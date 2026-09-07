//! Keystrokes reaching the line, and the state a paint reads back.
//!
//! The line *itself* is [`Line`] — buffer, caret, history,
//! Tab and the ghost, none of which needs a window. This file is the Bevy half:
//! the one system that turns a `KeyboardInput` into an edit, and the three
//! resources holding what a frame would otherwise recompute 60 times a second.
//!
//! # Read `text`, not `logical_key`
//!
//! The spacebar's logical key is [`Key::Space`], **not** `Key::Character(" ")`,
//! on every platform winit supports. A buffer built by matching
//! `Key::Character` therefore silently loses the space bar, and `look around`,
//! `sift march feed.log` — every multi-word command in DESIGN.md §6.1 — becomes
//! untypeable. Nothing in a test suite catches that; it dies on the first
//! keystroke a person types.
//!
//! [`KeyboardInput::text`] carries the correct locale-mapped character for every
//! layout, handles the Windows dead-key case where one press yields two
//! characters, and is `Some(" ")` for space. `logical_key` is consulted for
//! exactly two things: Enter and Backspace.
//!
//! # `text` is not safe to append
//!
//! It contains control characters — winit documents Enter as `Some("\r")`, and
//! Tab and Escape arrive the same way. A control character in the buffer would
//! reach [`Painter::put_str`](orbs_render::Painter), occupy a cell, and draw
//! **nothing**, because the renderer skips glyphs outside the CP437 repertoire.
//! The caret would drift away from the text with no visible cause.
//!
//! So every character is filtered through [`orbs_render::is_renderable`], which
//! is the repertoire itself rather than an approximation of it: `is_ascii_graphic`
//! would also throw away the accented range the font can draw.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::prelude::*;
use bevy::window::WindowFocused;

use orbs_shell::{Focus, Ghost, Line, Offered, Open, Panel, Scroll};

use crate::sim::Tower;

/// Keys a surface was still holding when it handed the keyboard back.
///
/// # The bug this exists for
///
/// You walk the archive's maze with the arrows and press Escape. The maze lets go
/// on that frame — but your finger is still on the arrow, and **key repeat keeps
/// delivering**. By the next frame the prompt owns the keyboard again, an arrow at
/// the prompt means *recall history*, and the newest entry is the `wander` that
/// opened the maze. So leaving the maze put the word back in the prompt.
///
/// It is not a maze bug: all four surfaces hand the keyboard back the same way, so
/// escaping the editor or the weave screen on a held arrow does the same thing.
///
/// # Why a set of keys rather than a quiet frame
///
/// Swallowing everything for a frame or two would be a race against the player's
/// key-repeat rate, which is a setting on their machine. What is actually wrong is
/// narrower and exact: **the prompt is being handed the *middle* of a keystroke
/// whose press it never saw.** So it drops events for exactly those keys, and
/// exactly until they are released — a fresh press afterwards is a real one and
/// gets through. `chord_is_stale` reasons about ghost modifiers the same way, and
/// for the same reason.
#[derive(Resource, Debug, Default)]
pub(crate) struct HeldOver {
    /// Physical keys whose press went to another surface.
    keys: std::collections::HashSet<KeyCode>,
    /// Whether another surface owned the keyboard on the previous frame.
    owned: bool,
}

impl HeldOver {
    /// Whether this key's press belonged to somebody else.
    pub(crate) fn swallows(&self, key: KeyCode) -> bool {
        self.keys.contains(&key)
    }
}

/// Notice when the keyboard changes hands, and what was held when it did.
///
/// **Ungated, and it has to be.** The obvious place for this is inside
/// [`type_into_line`], which already reads all four surface states — but that
/// system is gated on `on_message::<KeyboardInput>`, so it never runs on the
/// frames where a surface owned the keyboard and nobody typed. The edge it needs
/// to see is exactly the one it cannot: the frame Escape arrives, the surface has
/// already let go, and there is no previous frame on record saying it ever held on.
///
/// It asks [`Surfaces::focus`], which is now the only thing that decides who owns
/// a keystroke — `type_into_line`'s comment predicted the four-term shape's
/// ceiling was five and that a single owner was worth building before the fifth
/// arrived, and `orbs_shell::Focus` is that owner. This function keeps its own
/// reason for existing, which is the *edge* rather than the state.
pub(crate) fn watch_focus(
    surfaces: Surfaces,
    held: Res<ButtonInput<KeyCode>>,
    mut over: ResMut<HeldOver>,
) {
    if surfaces.focus().is_elsewhere() {
        over.owned = true;
        return;
    }
    if over.owned {
        over.owned = false;
        over.keys = held.get_pressed().copied().collect();
    }
    // Pruned every frame rather than on release: a key that has been let go of is
    // no longer held, and pruning here is what lets the *next* deliberate press of
    // the same key through. Testing `pressed` at the point of use instead would
    // swallow that press too, for ever.
    over.keys.retain(|key| held.pressed(*key));
}

/// The five surfaces that can hold the keyboard, as one parameter.
///
/// **A `SystemParam` rather than five more parameters**, which is
/// `render::plugin`'s precedent one crate over — and here it is not tidiness:
/// the fifth surface took `type_into_line` to thirteen arguments and clippy
/// refuses at twelve. That limit is the same pressure `orbs_shell::focus`
/// answered one level down, arriving at the call site instead, so the fix is the
/// same shape: the *set* of surfaces is one thing, and it should be named once.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct Surfaces<'w> {
    editing: Res<'w, super::editing::Editing>,
    loom: Res<'w, super::Loom>,
    walk: Res<'w, super::Walk>,
    chorus: Res<'w, super::Chorus>,
    scroll: Res<'w, Scroll>,
    standing: Res<'w, super::Standing>,
}

impl Surfaces<'_> {
    /// Who the next keystroke belongs to.
    pub(crate) fn focus(&self) -> Focus {
        Focus::of(self.open())
    }

    /// What is open, before anything decides who wins.
    ///
    /// [`focus`](Self::focus) is the question the keyboard asks; this is the one
    /// [`Showing`](orbs_shell::Showing) asks, and they are not the same. A
    /// crossing cares which surface *replaced the pane*, not which surface would
    /// receive the next keystroke — `chorus` takes the keys and leaves the pane
    /// alone, and the transcript scrolled back takes them without changing
    /// anything at all.
    pub(crate) fn open(&self) -> Open {
        opened(
            &self.editing,
            &self.loom,
            &self.walk,
            &self.chorus,
            &self.scroll,
            &self.standing,
        )
    }
}

/// What is open, gathered for [`orbs_shell::Focus`].
///
/// **The questions are asked here and answered there.** This build's copy of the
/// ordering is gone: `orbs-tui` had already reduced it to one enum, and keeping a
/// second expression of the same rule is how the two frontends come to disagree
/// about who owns a keystroke — which is `orbs_shell::shortcuts`'s argument,
/// restated one module along.
const fn opened(
    editing: &super::editing::Editing,
    loom: &super::Loom,
    walk: &super::Walk,
    chorus: &super::Chorus,
    scroll: &Scroll,
    standing: &super::Standing,
) -> Open {
    Open {
        editing: editing.is_open(),
        weaving: loom.is_open(),
        walking: walk.is_open(),
        chorusing: chorus.is_open(),
        reading: scroll.is_reading(),
        menuing: standing.is_open(),
    }
}

/// Recompute the suggestion.
pub(crate) fn suggest(mut ghost: ResMut<Ghost>, line: Res<Line>, tower: Res<Tower>) {
    let sim = tower.sim();
    ghost.0 = line.ghost(sim.scene(), !sim.choices().is_empty());
}

/// Re-read the panel from the world.
///
/// The five questions are [`Panel::refresh`]'s; this is the `resource_changed`
/// guard they hang on, which is the only part a terminal has no use for.
pub(crate) fn refresh_panel(mut panel: ResMut<Panel>, tower: Res<Tower>) {
    panel.refresh(tower.sim());
}

/// A line the player finished.
#[derive(Message, Debug, Clone)]
pub(crate) struct SubmittedMessage {
    /// What they typed, verbatim.
    pub(crate) line: String,
}

/// Forget every held key when the window loses focus.
///
/// **The bug this exists for.** A key's release event goes to whoever has focus.
/// Press `Cmd+Shift+Ctrl+4` on macOS and the screenshot overlay takes the window
/// away mid-chord, so the release for `Cmd` and `Ctrl` is delivered to *it* —
/// and `ButtonInput` here believes they are still down. Forever. Every keystroke
/// after that hits [`type_into_line`]'s chord guard and is dropped, and the
/// prompt is dead with nothing on screen to say why.
///
/// Any modal the operating system throws up does this: screenshots, Spotlight,
/// mission control, a notification stealing focus. The fix is not to enumerate
/// them but to distrust held state across a focus boundary, which is the only
/// moment the release could have gone missing.
pub(crate) fn forget_held_keys(
    mut focus: MessageReader<WindowFocused>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    if focus.read().any(|event| !event.focused) {
        keys.reset_all();
    }
}

/// Whether a keystroke that produced `text` proves the held chord is a ghost.
///
/// # Why a focus hook was never going to be enough
///
/// [`forget_held_keys`] assumed a stolen window is *observable*. Bevy makes the
/// same assumption and nothing else: `WindowFocused(false)` is the only thing
/// that reaches `check_keyboard_focus_lost`, which is the only thing that writes
/// `KeyboardFocusLost`, which is the only thing that calls `release_all`. Every
/// recovery path in the engine hangs off that one event.
///
/// **And Bevy 0.19 drops winit's `ModifiersChanged`**, which is the OS telling
/// you what is *actually* held — so when a key-up is swallowed without a focus
/// event, there is no mechanism anywhere to notice. The modifier is down for the
/// rest of the session, every keystroke hits the guard, and the field is dead
/// with nothing on screen to say why. `Cmd+Shift+Ctrl+4` does exactly this: the
/// screenshot overlay takes the keys and gives back no focus change.
///
/// So the recovery cannot be another focus hook. It is this: **the OS decides
/// what is text.** If a chord were really in force, macOS would interpret the
/// key as a command and hand us no text at all; Windows and X11 hand back a
/// control character, which [`Line::insert`] and the editor both already filter.
/// Text arriving *is* the proof that nothing is being held — so the held state
/// is stale, and saying so unsticks it on the first character typed.
///
/// Platform-independent, needs no event that may never come, and cannot make a
/// text field unusable: the worst case is one keystroke behaving as though the
/// chord had been released, which is what actually happened.
pub(crate) fn chord_is_stale(quiet: f32) -> bool {
    quiet >= STALE_AFTER
}

/// How long the keyboard must be silent before a held chord is disbelieved.
///
/// **Generous, because a false positive types a letter into the prompt.** A real
/// chord is pressed and used inside a fraction of a second; this is the gap left
/// by an overlay that held the keyboard for as long as it took a person to drag
/// a screenshot rectangle.
const STALE_AFTER: f32 = 2.0;

/// How long the keyboard was silent before the keystroke being handled now.
#[derive(Resource, Debug, Default)]
pub(crate) struct Quiet {
    /// The gap the text fields read this frame.
    gap: f32,
    /// Silence accumulated since the last key, which becomes the next `gap`.
    since: f32,
}

impl Quiet {
    /// The gap before the keystroke being handled now.
    pub(crate) const fn gap(&self) -> f32 {
        self.gap
    }

    /// Declare the keyboard to have been silent for `seconds`.
    ///
    /// For tests: `MinimalPlugins` brings `TimePlugin`, which rewrites `Time`
    /// from its own clock every frame, so a test cannot advance the silence by
    /// advancing `Time`. This drives the one value the guard actually reads.
    #[cfg(test)]
    pub(crate) const fn silent_for(&mut self, seconds: f32) {
        // The accumulator, not the published gap: `watch_quiet` runs first and
        // publishes `since` into `gap` on the frame keys arrive, so setting
        // `gap` here would be overwritten before anything read it.
        self.since = seconds;
    }
}

/// Advance the silence, and hand it to the text fields when a key arrives.
///
/// **Two fields, because resetting on arrival would erase the very thing the
/// readers need.** A frame with keys publishes the silence that preceded them
/// and starts counting again; a frame without keys just counts.
///
/// Ordered before the text fields, so the gap they read is the one in front of
/// this frame's keystroke.
pub(crate) fn watch_quiet(
    time: Res<Time>,
    mut keys: MessageReader<KeyboardInput>,
    mut quiet: ResMut<Quiet>,
) {
    if keys.read().next().is_some() {
        quiet.gap = quiet.since;
        quiet.since = 0.0;
    } else {
        quiet.since += time.delta_secs();
    }
}

/// Feed keystrokes into the line.
///
/// Runs in `Update`, never `FixedUpdate`: ticks are 1 Hz (§5.0) and typing
/// sampled at 1 Hz would be unusable. Nothing is lost by the split — Bevy runs
/// the fixed loop *before* `Update` in a frame, so a keystroke can never be
/// observed between two ticks of the same frame.
pub(crate) fn type_into_line(
    mut keys: MessageReader<KeyboardInput>,
    mut held: ResMut<ButtonInput<KeyCode>>,
    mut line: ResMut<Line>,
    tower: Res<Tower>,
    mut offered: ResMut<Offered>,
    mut submitted: MessageWriter<SubmittedMessage>,
    surfaces: Surfaces,
    quiet: Res<Quiet>,
    over: Res<HeldOver>,
) {
    // **Discarded here, not gated out by a run condition.** A message this
    // system never *reads* is still in the queue on the next frame, because
    // every reader carries its own cursor — so a system that simply does not run
    // while another surface has the keyboard leaves the keystrokes waiting, and
    // they all arrive at once the moment it runs again.
    //
    // That is not hypothetical: typing while the transcript was being read and
    // then pressing Escape put every one of those characters into the prompt,
    // and the test that found it had been written to check something else.
    // Clearing the cursor is what actually throws a keystroke away.
    //
    // **The refactor this comment used to promise has happened.** It said the
    // four-term shape's ceiling was five and that the fix — a single `Focus`
    // owner rather than a predicate per surface — was worth doing before the
    // fifth arrived. The ordering now lives once, in `orbs_shell::focus`, and
    // both frontends read it; what stays here is the *discarding*, which
    // genuinely differs between them and is explained above.
    if surfaces.focus().is_elsewhere() {
        keys.clear();
        return;
    }
    // Set when a keystroke proves the held chord is a ghost, and acted on after
    // the loop — `held` is borrowed for the duration of it.
    let mut stale = false;
    let stale_chord = chord_is_stale(quiet.gap());
    // Chords are commands, not text. Alt is deliberately **not** in this list:
    // AltGr is how European layouts type `@`, `#` and `\`, and guarding on it
    // would make those characters untypeable for the players who need them.
    //
    // The consequence, stated: `Alt+B`/`Alt+F` — readline's word motion — insert
    // characters rather than moving. Word motion is deferred with `Ctrl+R`,
    // `Delete`, `Ctrl+U` and `Ctrl+W`.
    let chord = held.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);

    // ...except the editing chords, which a Mac keyboard has no other key for:
    // there is no Home or End, and `Cmd+←/→` is what every text field on the
    // platform does. An allow-list rather than a hole — the guard exists because
    // `Cmd+Enter` was submitting lines, and that must stay true.
    let editing = held.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
        && !held.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    for event in keys.read() {
        // `repeat` is deliberately not filtered: held Backspace should delete
        // more than one character, which is what every text field does.
        if event.state != ButtonState::Pressed {
            continue;
        }
        // **A keystroke whose press went to another surface**, still repeating
        // after that surface let go. Dropped before anything else so it cannot
        // retire the Tab listing either — it is not the player answering the
        // prompt, it is a finger that has not lifted yet.
        if over.swallows(event.key_code) {
            continue;
        }
        // Any keystroke retires the last Tab listing — it answered a question
        // the player has already moved past.
        //
        // **Inside the press filter, and after it.** This ran at the top of the
        // function, which is gated on `on_message::<KeyboardInput>` — and winit
        // sends a `KeyboardInput` for the *release* too. So lifting the Tab key
        // re-entered here, cleared the list, and `continue`d past the release: a
        // listing that lived for the ~50ms a finger was down. `press` in the
        // tests only ever writes `Pressed`, which is why the suite was green.
        // A Tab cycle ends with it, for the same reason and in the same place:
        // the next Tab should start a fresh completion rather than resume one the
        // player has typed past.
        //
        // **`orbs_shell::apply` does this too, and neither copy is redundant.**
        // The shared one covers every key that reaches the prompt; this one
        // covers the keys that *do not* — a chorded keystroke `continue`s below
        // and never gets there, and `Ctrl+L` should still retire a listing the
        // player has moved past. Both are idempotent, so the overlap costs
        // nothing; deleting either changes behaviour, which is why this says so.
        if !matches!(event.logical_key, Key::Tab) {
            offered.clear();
            line.end_cycle();
        }
        // The chord guard covers **every** key, `Enter` and `Backspace`
        // included. They used to bypass it, so `Cmd+Enter` submitted the line
        // and `Ctrl+Backspace` ate a character — a chord the player aimed at
        // their operating system reaching into the prompt on the way past.
        // ...**unless the keyboard has been silent long enough that the held
        // chord cannot be real** — see `chord_is_stale`. A ghost from a
        // swallowed key-up outlives any gap; a chord a person is holding does
        // not.
        if chord && !stale_chord {
            // `Cmd+←/→` is Home/End on a keyboard that has neither.
            if editing {
                match &event.logical_key {
                    Key::ArrowLeft => line.home(),
                    Key::ArrowRight => line.end(),
                    _ => {}
                }
            }
            continue;
        }
        stale |= chord && stale_chord;
        // **What a keystroke *means* is `orbs-shell`'s**, and only finding it is
        // ours. Everything above this line is winit — press versus release, key
        // repeat, held modifiers, a ghost chord from a swallowed key-up — and a
        // terminal has none of it. Everything below would have been a second
        // line editor, which is the divergence that crate exists to stop.
        let Some(key) = pressed(event) else {
            continue;
        };
        if let Some(finished) = orbs_shell::apply(&key, &mut line, &mut offered, tower.sim()) {
            submitted.write(SubmittedMessage { line: finished });
        }
    }

    // The ghost is cleared *after* the loop, so the rest of this frame's keys
    // are judged by the same rule as the one that exposed it. Without this the
    // next keystroke would be guarded all over again — the character that got
    // through would look like a fluke, which is worse than a steady failure.
    if stale {
        held.reset_all();
    }
}

/// One winit keystroke, as the shared prompt understands it.
///
/// **`event.text`, not `logical_key`, for the default arm.** The spacebar's
/// logical key is [`Key::Space`], *not* `Key::Character(" ")`, on every platform
/// winit supports — so a buffer built by matching `Key::Character` silently loses
/// the space bar, and every multi-word command in §6.1 becomes untypeable.
/// Nothing in a test suite catches that; it dies on the first keystroke a person
/// types. `logical_key` is consulted for the named keys and nothing else.
pub(crate) fn pressed(event: &KeyboardInput) -> Option<orbs_shell::Key> {
    Some(match &event.logical_key {
        Key::Enter => orbs_shell::Key::Enter,
        Key::Backspace => orbs_shell::Key::Backspace,
        Key::Escape => orbs_shell::Key::Escape,
        Key::ArrowLeft => orbs_shell::Key::Left,
        Key::ArrowRight => orbs_shell::Key::Right,
        Key::ArrowUp => orbs_shell::Key::Up,
        Key::ArrowDown => orbs_shell::Key::Down,
        Key::Home => orbs_shell::Key::Home,
        Key::End => orbs_shell::Key::End,
        Key::Tab => orbs_shell::Key::Tab,
        _ => orbs_shell::Key::Text(event.text.as_ref()?.to_string()),
    })
}

// `answering` — whether a line is a digit answering a numbered prompt, read
// before `submit` clears `Choices` — moved to `orbs_shell::keys` with the key
// table it guards. It is §6's rule, and a second copy of it here would be a
// second answer to *"is this a phrasing worth remembering?"*.
