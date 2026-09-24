//! Keystrokes reaching the line, and the state a paint reads back.
//!
//! The line *itself* is [`Line`] — buffer, caret, history, Tab and the ghost,
//! none of which needs a window. This file is the Bevy half: the one system that
//! turns a `KeyboardInput` into an edit, and the three resources holding what a
//! frame would otherwise recompute 60 times a second.
//!
//! Read `text`, not `logical_key`: the spacebar's logical key is [`Key::Space`],
//! not `Key::Character(" ")`, on every platform winit supports, so a buffer
//! built by matching `Key::Character` silently loses the space bar and every
//! multi-word command in DESIGN.md §6.1 becomes untypeable.
//! [`KeyboardInput::text`] carries the locale-mapped character for every layout
//! and handles the Windows dead-key case. `logical_key` is consulted for Enter
//! and Backspace.
//!
//! `text` is not safe to append: it holds control characters — winit documents
//! Enter as `Some("\r")` — and one in the buffer would reach
//! [`Painter::put_str`](orbs_render::Painter), occupy a cell and draw nothing,
//! drifting the caret from the text. So every character is filtered through
//! [`orbs_render::is_renderable`], the repertoire itself: `is_ascii_graphic`
//! would also throw away the accented range the font can draw.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::prelude::*;
use bevy::window::WindowFocused;

use orbs_shell::{Focus, Ghost, Line, Offered, Open, Panel, Scroll};

use crate::sim::Tower;

/// Keys a surface was still holding when it handed the keyboard back.
///
/// Escape out of the archive's maze with a finger still on an arrow and key
/// repeat keeps delivering: by the next frame the prompt owns the keyboard, an
/// arrow there means *recall history*, and leaving put the `wander` that opened
/// the maze back in the prompt. All four surfaces do this.
///
/// A set of keys rather than a quiet frame, because swallowing everything for a
/// frame or two races the player's key-repeat rate. What is wrong is exact: the
/// prompt is handed the *middle* of a keystroke whose press it never saw, so it
/// drops events for those keys until they are released.
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
/// Ungated, and it has to be: [`type_into_line`] is gated on
/// `on_message::<KeyboardInput>`, so it never runs on the frames where a surface
/// owned the keyboard and nobody typed — and the edge that matters is the frame
/// Escape arrives.
///
/// Who owns a keystroke is [`Surfaces::focus`]'s answer; what this keeps is the
/// *edge* rather than the state.
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
    // Pruned every frame rather than on release: pruning here is what lets the
    // *next* deliberate press of the same key through. Testing `pressed` at the
    // point of use instead would swallow that press too, for ever.
    over.keys.retain(|key| held.pressed(*key));
}

/// The surfaces that can hold the keyboard, as one parameter.
///
/// A `SystemParam` rather than one parameter each: a fifth surface took
/// `type_into_line` to thirteen arguments and clippy refuses at twelve. The
/// *set* of surfaces is one thing and should be named once (§19).
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct Surfaces<'w> {
    editing: Res<'w, super::editing::Editing>,
    loom: Res<'w, super::Loom>,
    walk: Res<'w, super::Walk>,
    scroll: Res<'w, Scroll>,
    standing: Res<'w, super::Standing>,
    reading_manual: Res<'w, super::Reading>,
}

impl Surfaces<'_> {
    /// Who the next keystroke belongs to.
    pub(crate) fn focus(&self) -> Focus {
        Focus::of(self.open())
    }

    /// What is open, before anything decides who wins.
    ///
    /// [`focus`](Self::focus) is the keyboard's question; this is
    /// [`Showing`](orbs_shell::Showing)'s. A crossing cares which surface
    /// *replaced the pane*, not which would take the next keystroke.
    pub(crate) fn open(&self) -> Open {
        opened(
            &self.editing,
            &self.loom,
            &self.walk,
            &self.scroll,
            &self.standing,
            &self.reading_manual,
        )
    }
}

/// What is open, gathered for [`orbs_shell::Focus`].
///
/// The questions are asked here and answered there. A second expression of one
/// rule is how two frontends come to disagree about who owns a keystroke —
/// `orbs_shell::shortcuts`'s argument, one module along.
const fn opened(
    editing: &super::editing::Editing,
    loom: &super::Loom,
    walk: &super::Walk,
    scroll: &Scroll,
    standing: &super::Standing,
    reading_manual: &super::Reading,
) -> Open {
    Open {
        editing: editing.is_open(),
        weaving: loom.is_open(),
        walking: walk.is_open(),
        reading: scroll.is_reading(),
        menuing: standing.is_open(),
        reading_manual: reading_manual.is_open(),
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
/// A key's release goes to whoever has focus, so `Cmd+Shift+Ctrl+4` on macOS
/// hands the screenshot overlay the release for `Cmd` and `Ctrl` and
/// `ButtonInput` here believes they are still down, for ever — every later
/// keystroke hits [`type_into_line`]'s chord guard with nothing saying why.
///
/// Any OS modal does this, so the fix is to distrust held state across a focus
/// boundary rather than to enumerate them.
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
/// A focus hook was never going to be enough: [`forget_held_keys`] assumes a
/// stolen window is observable, and Bevy 0.19 drops winit's `ModifiersChanged`
/// — so a key-up swallowed without a focus event leaves the modifier down for
/// the rest of the session. `Cmd+Shift+Ctrl+4` does exactly that.
///
/// So the OS decides what is text: under a real chord macOS hands back none at
/// all, and Windows and X11 a control character [`Line::insert`] already
/// filters. Text arriving is the proof that nothing is held.
///
/// Platform-independent, needs no event that may never come, and its worst case
/// is one keystroke behaving as though the chord had been released.
pub(crate) fn chord_is_stale(quiet: f32) -> bool {
    quiet >= STALE_AFTER
}

/// How long the keyboard must be silent before a held chord is disbelieved.
///
/// Generous, because a false positive types a letter into the prompt. A real
/// chord is pressed and used inside a fraction of a second; this is the gap an
/// overlay leaves while a person drags a screenshot rectangle.
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
    /// every frame, so a test cannot advance the silence by advancing `Time`.
    #[cfg(test)]
    pub(crate) const fn silent_for(&mut self, seconds: f32) {
        // The accumulator, not the published gap: `watch_quiet` runs first and
        // publishes `since` into `gap`, so setting `gap` here would be
        // overwritten before anything read it.
        self.since = seconds;
    }
}

/// Advance the silence, and hand it to the text fields when a key arrives.
///
/// Two fields, because resetting on arrival would erase what the readers need:
/// a frame with keys publishes the silence that preceded them and starts
/// counting again; a frame without keys just counts.
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
    // Discarded here, not gated out by a run condition: every reader carries its
    // own cursor, so a system that does not run while another surface has the
    // keyboard leaves the keystrokes queued and they all arrive at once.
    // Clearing the cursor is what actually throws a keystroke away.
    //
    // The ordering itself lives once, in `orbs_shell::focus`; what stays here is
    // the discarding, which genuinely differs.
    if surfaces.focus().is_elsewhere() {
        keys.clear();
        return;
    }
    // Set when a keystroke proves the held chord is a ghost, and acted on after
    // the loop — `held` is borrowed for the duration of it.
    let mut stale = false;
    let stale_chord = chord_is_stale(quiet.gap());
    // Chords are commands, not text. Alt is deliberately not in this list: AltGr
    // is how European layouts type `@`, `#` and `\`. The consequence is that
    // `Alt+B`/`Alt+F` — readline's word motion — insert characters rather than
    // moving; word motion is deferred with `Ctrl+R`, `Delete`, `Ctrl+U` and
    // `Ctrl+W`.
    let chord = held.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);

    // ...except the editing chords, which a Mac keyboard has no other key for:
    // there is no Home or End, and `Cmd+←/→` is what every text field on the
    // platform does. An allow-list rather than a hole — the guard exists because
    // `Cmd+Enter` was submitting lines.
    let editing = held.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
        && !held.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    for event in keys.read() {
        // `repeat` is deliberately not filtered: held Backspace should delete
        // more than one character, which is what every text field does.
        if event.state != ButtonState::Pressed {
            continue;
        }
        // A keystroke whose press went to another surface, still repeating.
        // Dropped before anything else so it cannot retire the Tab listing
        // either: it is a finger that has not lifted yet.
        if over.swallows(event.key_code) {
            continue;
        }
        // Any keystroke retires the last Tab listing — it answered a question
        // the player has already moved past. A Tab cycle ends with it too.
        //
        // Inside the press filter, because winit sends a `KeyboardInput` for the
        // release as well: at the top of the function, lifting the Tab key
        // cleared the list, so a listing lived for the ~50ms a finger was down.
        //
        // `orbs_shell::apply` does this too, and neither copy is redundant: the
        // shared one covers every key that reaches the prompt, this one the keys
        // that *do not* — a chorded keystroke `continue`s below and never gets
        // there. Both are idempotent, so the overlap costs nothing.
        if !matches!(event.logical_key, Key::Tab) {
            offered.clear();
            line.end_cycle();
        }
        // The chord guard covers every key, `Enter` and `Backspace` included:
        // they used to bypass it, so `Cmd+Enter` submitted the line and
        // `Ctrl+Backspace` ate a character. ...unless the keyboard has been
        // silent long enough that the held chord cannot be real — see
        // `chord_is_stale`.
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
        // What a keystroke *means* is `orbs-shell`'s; only finding it is ours.
        // Everything above is winit — press versus release, key repeat, held
        // modifiers, a ghost chord — which a terminal has none of.
        let Some(key) = pressed(event) else {
            continue;
        };
        if let Some(finished) = orbs_shell::apply(&key, &mut line, &mut offered, tower.sim()) {
            submitted.write(SubmittedMessage { line: finished });
        }
    }

    // Cleared after the loop, so the rest of this frame's keys are judged by the
    // same rule as the one that exposed the ghost — otherwise the character that
    // got through looks like a fluke, which is worse than a steady failure.
    if stale {
        held.reset_all();
    }
}

/// One winit keystroke, as the shared prompt understands it.
///
/// `event.text`, not `logical_key`, for the default arm: the spacebar's logical
/// key is [`Key::Space`], *not* `Key::Character(" ")`, so matching
/// `Key::Character` loses the space bar and every multi-word command in §6.1.
/// `logical_key` is consulted for the named keys only.
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
        // Missing for a whole version: `orbs_shell::Key` gained
        // `PageUp`/`PageDown` and the pane drew *pgdn for more*, but neither
        // frontend built one. winit reports `text: None` for a named key, so the
        // default arm's `?` dropped them silently — a `_ =>` fallthrough cannot
        // fail to compile when an enum grows.
        Key::PageUp => orbs_shell::Key::PageUp,
        Key::PageDown => orbs_shell::Key::PageDown,
        _ => orbs_shell::Key::Text(event.text.as_ref()?.to_string()),
    })
}

// `answering` — whether a line is a digit answering a numbered prompt, read
// before `submit` clears `Choices` — moved to `orbs_shell::keys` with the key
// table it guards. It is §6's rule, and a second copy here would be a second
// answer to it.
