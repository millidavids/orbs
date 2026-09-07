//! What the function keys do, what a finished line does, and how a session ends.
//!
//! Split out of `plugin.rs`, which CLAUDE.md restricts to registration. These
//! are the acts that belong to no surface: they work from inside the editor, the
//! weave screen and the maze alike, which is why they are gathered rather than
//! filed under whichever screen happens to be open.
//!
//! **The rules behind them are `orbs_shell::shortcuts`'s**, so the two builds
//! cannot disagree about where `Tampered` sits in the register cycle or what the
//! trace file is called.

use bevy::prelude::*;

use super::input::SubmittedMessage;
use crate::sim::Tower;

/// Where the parse trace is written.
///
/// TSV beside the binary, per §19: no dependency, survives `grep`, pastes into a
/// spreadsheet. One row per *candidate*, not per input, because the gate needs
/// to know whether a miss was the verb or the argument.
use orbs_shell::TRACE_PATH;

/// Hand a finished line to the sim.
///
/// The sim resolves it immediately and queues any command for the next tick —
/// see `orbs_sim::session` for why those are two different clocks.
pub(super) fn submit(
    mut lines: MessageReader<SubmittedMessage>,
    mut tower: ResMut<Tower>,
    mut scroll: ResMut<orbs_shell::Scroll>,
) {
    for submitted in lines.read() {
        tower.submit(&submitted.line);
        // Back to the newest output. The player acted; what they want to see is
        // what it did, and leaving them in history to watch their own command
        // scroll past off screen is the one place terminal convention is wrong
        // here — a terminal has no orb answering on its own clock.
        scroll.rewind();
    }
}

/// Write the parse trace to disk.
pub(super) fn export_trace(tower: Res<Tower>) {
    match orbs_shell::export_trace(tower.sim()) {
        Ok(summary) => info!("{summary}"),
        // A failed export must not take the session down with it — the tester
        // whose run it was recording is still playing.
        Err(error) => warn!("parse trace -> {TRACE_PATH} failed: {error}"),
    }
}

/// Step the orb's tonal register.
///
/// Type a command with the register on and the echo comes back in a different
/// face; then `peruse orb.log` and the log lines come back **plain**, because §3
/// exempts the diagnostic surfaces from the eldritch treatment and only from
/// that one. A sabotage tell is not exempt anywhere, which is the asymmetry the
/// whole disjointness rule buys.
pub(super) fn cycle_register(mut tower: ResMut<Tower>) {
    info!("register: {:?}", tower.cycle_register());
}

/// `F9` — a chant that waits for the singer rather than for the clock (§14).
///
/// **The one accommodation the menagerie needs**, because it is the one domain
/// whose puzzle is timed. It reaches the same ceiling: a patient chant and a
/// played one both yield what was sung correctly, so what the key removes is
/// the dimension reflex and speech cannot serve and nothing else.
pub(super) fn toggle_patient(mut tower: ResMut<Tower>) {
    info!("patient chant: {}", tower.toggle_patient());
}

/// Leave the orb, because `F10` was pressed.
pub(super) fn quit(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

/// Leave the orb, because the word was typed **and confirmed**.
///
/// **The sim asks first.** `quit` once puts the question, `quit` again answers
/// it, and any other command answers *no* — so `Quitting::take` is only ever
/// true for a decision the player made twice. `execute::quit` has why leaving is
/// the one word in the game that asks: it cannot be undone, waited out or
/// repeated away, and the tower is written on the way.
///
/// The take-once handshake beside `scribe`, `unfurl`, `weave`, `wander` and
/// `menu`: the sim records the decision and the frontend decides what leaving
/// *means*. Here it is an `AppExit`; in the terminal build it is raw mode being
/// put back.
pub(super) fn quit_requested(mut tower: ResMut<Tower>, mut exit: MessageWriter<AppExit>) {
    // **Peeked before it is taken**, exactly as the other handshakes are.
    // `quitting` needs `&mut`, and reaching through `ResMut` for it stamps
    // `Tower`'s change tick — and this system's own run condition is
    // `resource_changed::<Tower>`, so from the first frame Tower changed it
    // re-armed itself for ever and dragged `refresh_panel`, `suggest` and the
    // five `open_requested` systems back to frame rate with it. `Panel::refresh`
    // calls `Sim::briefs`, which walks every built room and builds two
    // `QueryState`s per call.
    //
    // `editing::open_requested`'s comment records this happening once already.
    // The peek that stops it was added with `quit` and then called by nothing.
    if !tower.is_quitting() {
        return;
    }
    if tower.quitting() {
        exit.write(AppExit::Success);
    }
}
