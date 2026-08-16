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

use super::line::Line;
use crate::sim::Tower;

/// What Tab last offered, and which of them is in the line.
///
/// A resource rather than a record, and cleared on the next keystroke: readline
/// lists on ambiguity, but the log is the sim's and a Tab press is not something
/// a replay could reproduce.
#[derive(Resource, Debug, Default)]
pub(crate) struct Offered {
    /// The candidates, in the order the completer offered them.
    pub(crate) options: Vec<String>,
    /// Which one repeated Tab has reached, so the list can mark it.
    ///
    /// Without this the list is a wall of equal-looking words while the line
    /// changes underneath it, and the player has no way to see where they are in
    /// the cycle — which is the whole affordance.
    pub(crate) current: Option<usize>,
}

impl Offered {
    /// Whether there is anything to draw.
    pub(crate) const fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    /// Forget the listing.
    pub(crate) fn clear(&mut self) {
        self.options.clear();
        self.current = None;
    }
}

/// The inline suggestion trailing the caret.
///
/// **Recomputed on change, not per frame.** [`Line::ghost`] walks the history,
/// then runs the whole completer — which filters every synonym, builds an owned
/// `String` per candidate, sorts, dedups, and collects a `Vec<char>` into a
/// `String` for the common prefix. That ran at 60 Hz off inputs that change on a
/// keystroke or a tick, so ~59 frames in 60 rebuilt a string identical to the one
/// already on screen.
///
/// It stays a pure function of `(line, scene, prompt_open)` — this holds the
/// result, and the run condition names exactly what invalidates it, so there is
/// no second copy able to drift from the line it trails.
#[derive(Resource, Debug, Default)]
pub(crate) struct Ghost(pub(crate) String);

/// Recompute the suggestion.
pub(crate) fn suggest(mut ghost: ResMut<Ghost>, line: Res<Line>, tower: Res<Tower>) {
    let sim = tower.sim();
    ghost.0 = line.ghost(sim.scene(), !sim.choices().is_empty());
}

/// How far back through the transcript the player has scrolled.
///
/// **In records, not rows.** The transcript already finds its tail by
/// binary-searching for the smallest *record* skip whose measured height fits the
/// pane, because a record is not one row — a wrapped message is several, a tiled
/// listing packs many into few, and every command opens with a blank line. Rows
/// would need a second, different measure of the same stream; records reuse the
/// one that is already there and already correct.
///
/// Zero is the bottom, which is where it returns on every submission: you typed
/// something, so you want to see what it did.
#[derive(Resource, Debug, Default)]
pub(crate) struct Scroll {
    /// Records held back from the newest end.
    back: usize,
    /// Whether the transcript has the keyboard — see [`Scroll::is_reading`].
    reading: bool,
}

impl Scroll {
    /// How far back the view is.
    pub(crate) const fn back(&self) -> usize {
        self.back
    }

    /// Whether the transcript currently has the keyboard.
    ///
    /// # Why a verb turns this on
    ///
    /// `PageUp` has scrolled since the transcript existed, and nothing said so:
    /// the border advertises `PgDn newest` only once you are *already* scrolled
    /// back, so the affordance announced itself exclusively to players who had
    /// found it. In a game with no mouse and no menus, that is no affordance at
    /// all — hence `unfurl` (§19), and hence this: the word puts the keys on
    /// screen, which is what the player keeps once they stop needing the word.
    ///
    /// It is the **third** thing that can own the keyboard, after the prompt and
    /// the editor. Which one consumes a keystroke is decided in
    /// [`type_into_line`] rather than by a set of run conditions that had to
    /// stay complements — see there for why declining a key means running.
    pub(crate) const fn is_reading(&self) -> bool {
        self.reading
    }

    /// Take the keyboard, and start from where the view already is.
    pub(crate) const fn read(&mut self) {
        self.reading = true;
    }

    /// Give the keyboard back to the prompt.
    ///
    /// **Escape alone, and it does not scroll anywhere.** Leaving reading mode
    /// is not the same act as returning to the newest output — a player who read
    /// back and pressed Escape wants to type, not to lose their place — so
    /// `PgDn` still walks forward and this only hands the keys over. It is the
    /// same meaning Escape has in the editor: step out of the mode you are in.
    pub(crate) const fn stop_reading(&mut self) {
        self.reading = false;
    }

    /// Whether the player is looking at history rather than at the newest output.
    pub(crate) const fn is_back(&self) -> bool {
        self.back > 0
    }

    /// Return to the newest output.
    pub(crate) const fn rewind(&mut self) {
        self.back = 0;
    }

    /// Move by `step` records, older or newer. Clamped at both ends.
    ///
    /// **`step` is a record count the caller measured**, not a row count. A row
    /// count is not a safe stand-in: a record costs *at least* one row, which
    /// caps records-per-page above rather than below, so paging by the pane's
    /// rows moved more than a screenful and dropped the lines in between. See
    /// `plugin::page_step`, which measures the page with the same `RecordView`
    /// the transcript is drawn with.
    pub(crate) fn page(&mut self, step: usize, older: bool, total: usize) {
        let step = step.max(1);
        self.back = if older {
            self.back.saturating_add(step).min(total)
        } else {
            self.back.saturating_sub(step)
        };
    }
}

/// §10.1's instrument panel, as the sim last reported it.
///
/// **Rebuilt on tick, not per frame.** `tower::instruments` walks the room's
/// children, then each fixture's children, cloning a `String` per name and
/// allocating a `Vec` per lookup — roughly twenty allocations for the
/// laboratory's five instruments, at 60 Hz, for state that changes at most once a
/// second. The sim is the authority either way; this is where the answer waits
/// between ticks.
#[derive(Resource, Debug, Default)]
pub(crate) struct Panel {
    /// The instruments where the player is standing.
    pub(crate) instruments: Vec<orbs_sim::tower::Instrument>,
    /// That place's leaf name, for the panel's spoken summary.
    pub(crate) domain: String,
    /// The stacks the player is standing over, if they are open.
    ///
    /// **On the same tick clock as the instruments, and it belongs here for the
    /// same reason.** A 49-cell `Vec` rebuilt at 60 Hz would be the allocation
    /// this resource exists to stop; rebuilt once a second it is exactly as
    /// fresh as the world it describes, because the world moves at 1 Hz too.
    pub(crate) stacks: Option<orbs_render::Stacks>,
}

/// Re-read the panel from the world.
pub(crate) fn refresh_panel(mut panel: ResMut<Panel>, tower: Res<Tower>) {
    let sim = tower.sim();
    panel.instruments = sim.instruments();
    panel.domain = orbs_sim::parser::leaf(&sim.location()).to_owned();
    panel.stacks = sim.stacks();
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
    editing: Res<super::editing::Editing>,
    loom: Res<super::Loom>,
    walk: Res<super::Walk>,
    scroll: Res<Scroll>,
    quiet: Res<Quiet>,
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
    // **Four terms, and the prediction this comment used to make has come
    // true.** It said the shape's ceiling was five and that the fix — a single
    // `Focus` owner rather than a predicate per surface — was worth doing before
    // the fifth arrived. `wander` is the fourth and it is the last one that goes
    // in here: a fifth surface refactors this first.
    //
    // The reason it is worth naming rather than living with is that each term is
    // a place to *forget*. A surface added without its term does not fail
    // loudly; it types into an invisible prompt while the player looks at
    // something else, and the characters arrive later.
    if editing.is_open() || loom.is_open() || walk.is_open() || scroll.is_reading() {
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
        match &event.logical_key {
            Key::Enter => {
                // A bare digit answering §6's numbered prompt is an answer, not
                // a phrasing, so it is not remembered. `answering` is sampled
                // from the world *before* `submit` clears `Choices`.
                let remember = !answering(&tower, line.text());
                let finished = line.take(remember);
                submitted.write(SubmittedMessage { line: finished });
            }
            Key::Backspace => line.backspace(),
            Key::Escape => line.clear(),
            Key::ArrowLeft => line.left(),
            Key::ArrowRight => line.right(),
            Key::ArrowUp => line.earlier(),
            Key::ArrowDown => line.later(),
            Key::Home => line.home(),
            Key::End => line.end(),
            Key::Tab => {
                let open = !tower.sim().choices().is_empty();
                // Candidates are **transient Frame content**, not a record.
                // There is deliberately no `scrollback_mut` (§13): a frontend
                // writing into the log makes a session that `(seed,
                // submissions)` cannot replay, and a Tab press is not a
                // submission.
                //
                // Assigned even when empty, so a Tab that *completes* a word
                // retires the listing that asked which word it was.
                offered.options = line.tab(tower.sim().scene(), open);
                offered.current = line.cycling();
            }
            _ => {
                let Some(text) = &event.text else {
                    continue;
                };
                line.insert(text);
            }
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

/// Whether this line is a digit answering a numbered prompt.
///
/// Read *before* the line is submitted, because `Sim::submit` clears `Choices`
/// the moment it takes a non-digit — so afterwards there is no way to tell an
/// answer from a command that happened to be a number.
fn answering(tower: &Tower, line: &str) -> bool {
    // Through the sim's own predicate, not a second copy of it. What counts as
    // an answer is §6's rule — a leading `#`, a `1)`, a range would all be
    // changes to it — and the sim already exports the canonical form.
    !tower.sim().choices().is_empty() && orbs_sim::parser::is_answer(line)
}
