//! Registration only (CLAUDE.md).
//!
//! The system that turns a request into a playing sound is `ringing`, with the
//! message and the switch it reads. This file wires them up and nothing else.

use bevy::audio::AddAudioSource as _;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use super::bed::{self, Humming};
use super::cues::{self, Heard, Voices};
use super::keys;
use super::ringing::{self, RingMessage, Saying};
use super::synth::Cue;

/// The orb's voice: synthesised cues, and the hum under them.
pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<Cue>()
            .add_message::<RingMessage>()
            .init_resource::<Heard>()
            .init_resource::<Humming>()
            .init_resource::<Saying>()
            .init_resource::<super::Levels>()
            .add_systems(Startup, cues::voice_the_orb)
            .add_systems(
                Update,
                (
                    // The watermark first, then the keys, then the playing, so a
                    // cue written this frame is played this frame. The
                    // alternative is a frame of latency on a keystroke, which is
                    // audible in a way a frame of latency on a glyph is not.
                    cues::ring_for_records.run_if(resource_exists::<crate::sim::Tower>),
                    // CLAUDE.md: every `Update` system has a `run_if`. This one
                    // reads `KeyboardInput` and had none — harmless, and the one
                    // system in the module out of step with the rule.
                    //
                    // And `booted`, because the boot card takes no keys: nothing
                    // reaches the prompt until the sequence is live, so a click
                    // during it is the orb answering a keystroke it ignored,
                    // which reads as the card being skippable when it
                    // deliberately is not (§19).
                    keys::ring_for_keys
                        .run_if(on_message::<KeyboardInput>)
                        .run_if(crate::boot::booted),
                    ringing::ring.run_if(resource_exists::<Voices>),
                    // The bed is not a cue and does not go through `ring`: it
                    // loops, and it is one entity for the life of a register.
                    bed::keep_humming.run_if(resource_exists::<crate::sim::Tower>),
                )
                    .chain(),
            )
            // Every way out, the way `keep_on_the_way_out` does it: `F10`, the
            // verb and the window's close button all end in an `AppExit`, and
            // a hum still playing while the window tears down is a click.
            .add_systems(Last, bed::quieten.run_if(on_message::<AppExit>));
    }
}

#[cfg(test)]
mod tests {
    use super::super::cues::Voice;
    use super::*;
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key as LogicalKey, KeyboardInput};

    /// An app with the two writers and the message, and nothing that needs a
    /// sound card.
    ///
    /// The run conditions are the plugin's own, deliberately: a test app that
    /// guarded differently would be testing a system the game does not run.
    fn app() -> App {
        let mut app = App::new();
        app.add_message::<KeyboardInput>()
            .add_message::<RingMessage>()
            .init_resource::<Heard>()
            .add_systems(
                Update,
                (
                    cues::ring_for_records.run_if(resource_exists::<crate::sim::Tower>),
                    keys::ring_for_keys,
                ),
            );
        app
    }

    /// Every cue asked for since the last update.
    fn rung(app: &App) -> Vec<Voice> {
        app.world()
            .resource::<Messages<RingMessage>>()
            .iter_current_update_messages()
            .map(|RingMessage(voice)| *voice)
            .collect()
    }

    fn press(app: &mut App, key: LogicalKey) {
        app.world_mut().write_message(KeyboardInput {
            key_code: bevy::input::keyboard::KeyCode::KeyA,
            logical_key: key,
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    #[test]
    fn a_real_keystroke_rings() {
        // A real `KeyboardInput` at the real system. §19 records the menu
        // shipping a keyboard defect through four green dump latches, and a
        // dump presses no key — so anything on the input path is tested here or
        // it is not tested.
        let mut app = app();
        press(&mut app, LogicalKey::Character("a".into()));
        press(&mut app, LogicalKey::Enter);
        app.update();
        assert_eq!(rung(&app), vec![Voice::Key, Voice::Enter]);
    }

    #[test]
    fn a_held_key_does_not_stutter() {
        // Held Backspace repeats at the OS rate, which is faster than a cue is
        // long: without the guard it is a buzz rather than a click.
        let mut app = app();
        app.world_mut().write_message(KeyboardInput {
            key_code: bevy::input::keyboard::KeyCode::Backspace,
            logical_key: LogicalKey::Backspace,
            state: ButtonState::Pressed,
            text: None,
            repeat: true,
            window: Entity::PLACEHOLDER,
        });
        app.update();
        assert!(rung(&app).is_empty(), "a key repeat rang");
    }

    #[test]
    fn the_transcript_a_tower_was_loaded_with_does_not_all_play_at_once() {
        // The watermark's whole reason: a restored save opens with a tail of
        // records already in the stream, and a watermark starting at nought
        // would play somebody's entire last session at them on load.
        let mut app = app();
        let mut tower = crate::sim::Tower::fresh(1);
        // Through the real path, so these are records of the kind and role the
        // cue table actually reads.
        for line in ["attend laboratory", "survey", "kindle athanor"] {
            tower.submit(line);
            tower.step();
        }
        let before = tower.sim().scrollback().records().sequence();
        assert!(before > 0, "the tower made no records to be quiet about");
        app.insert_resource(tower);

        app.update();
        assert!(
            rung(&app).is_empty(),
            "the tower it opened with played itself",
        );

        // ...and the *next* thing that happens is still heard. A watermark that
        // suppressed the tail by going silent would pass the line above.
        {
            let mut tower = app.world_mut().resource_mut::<crate::sim::Tower>();
            tower.submit("wibble");
            tower.step();
        }
        app.update();
        assert!(
            !rung(&app).is_empty(),
            "nothing was heard after the load either",
        );
    }

    #[test]
    fn a_keystroke_reaches_an_actual_audio_player() {
        // The whole chain, through the real plugin: `add_audio_source`, the
        // startup table, the key reader, the message, and a spawned
        // `AudioPlayer<Cue>`. The tests above stop at the message, and a message
        // nobody turns into a sound is §15's API with no callers.
        //
        // No sound card is needed: `AudioPlugin` warns and goes inert without
        // one, which is what CI is, and the entity is spawned either way.
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            // `InputPlugin` is what registers `KeyboardInput`, and `SoundPlugin`
            // reads it — the same dependency every other key reader in this
            // build has on `DefaultPlugins`.
            bevy::input::InputPlugin,
            bevy::asset::AssetPlugin::default(),
            bevy::audio::AudioPlugin::default(),
            SoundPlugin,
        ));
        app.update();
        press(&mut app, LogicalKey::Character("a".into()));
        app.update();
        app.update();

        let mut players = app.world_mut().query::<&AudioPlayer<Cue>>();
        assert!(
            players.iter(app.world()).next().is_some(),
            "a keystroke reached no audio player",
        );
    }

    #[test]
    fn a_tower_arriving_mid_session_does_not_play_its_whole_transcript() {
        // The swap case, which `now <= last` never caught. The orb wakes at the
        // threshold, whose scratch world pushes a handful of records, so the
        // watermark sits low; loading a save jumps the sequence into the
        // thousands and `Records::since` skipped nothing. `Heard` is in
        // `shell_resources!` now, so a swap blanks it — this holds the other
        // half, that a discontinuity nothing reset still cannot flood.
        let mut app = app();
        let mut scratch = crate::sim::Tower::fresh(1);
        scratch.submit("survey");
        scratch.step();
        app.insert_resource(scratch);
        app.update();
        rung(&app);

        // A different tower, far further along, arriving with no reset at all.
        let mut carried = crate::sim::Tower::fresh(2);
        for line in [
            "attend laboratory",
            "kindle athanor",
            "grind sage",
            "wibble",
        ] {
            carried.submit(line);
            carried.step();
        }
        app.insert_resource(carried);
        app.update();
        assert!(
            rung(&app).len() <= super::super::cues::burst(),
            "a tower arriving played {} cues at once",
            rung(&app).len(),
        );
    }

    #[test]
    fn the_bed_stops_when_the_orb_does() {
        // A looping source that outlives the window is a click on the way out,
        // and on some backends a warning per frame after it. `quieten` is on
        // `AppExit` for `keep_on_the_way_out`'s reason: three of the four ways
        // out never touch the `Quitting` flag.
        let mut app = App::new();
        app.add_systems(Update, bed::quieten);
        let entity = app.world_mut().spawn(super::bed::Bed).id();
        app.update();
        assert!(
            app.world().get_entity(entity).is_err(),
            "the hum survived the orb being put down",
        );
    }
}
