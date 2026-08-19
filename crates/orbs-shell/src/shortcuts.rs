//! What the function keys do — the handful that are neither the prompt nor a
//! surface.
//!
//! `F4` flips the focus mode, `F5` shows the linear stream, `F6` writes the
//! parse trace, `F7` cycles the tonal register. Each is a *rule* rather than a
//! binding: which key reaches it is a frontend's business, and the two disagree
//! already — a terminal has no `F12` screenshot and no `F2` phosphor to cycle.
//!
//! **They live here because they were the gap.** The terminal build shipped
//! binding only `F10`, while `prompt.rs` drew `F4 deep` into the session border
//! on every frame — a key the screen offered and the build did not answer, which
//! is §19's *"the first affordance the game showed was one that did not work"*
//! arriving a second time. Two of these were one-line rules living in the Bevy
//! frontend, and a second copy is how the next one would have gone missing.

use orbs_render::Presentation;
use orbs_sim::Sim;

/// Where `F6` writes the parse trace.
///
/// Beside the binary rather than in a temp directory: a tester who pressed the
/// key is going to attach the file to a report, and one they cannot find is one
/// that never gets attached.
pub const TRACE_PATH: &str = "orbs-parse.tsv";

/// Step the tonal register on.
///
/// §3's three registers, in the order `F7` walks them. **A rule, not a setting**
/// — it is a development affordance for looking at eldritch and tampered text
/// without waiting for the world to produce either, and Phase 11's settings
/// screen is where a player-facing version would live.
///
/// Returns what it became, for whatever the caller logs.
pub fn cycle_register(sim: &mut Sim) -> Presentation {
    let next = match sim.register() {
        Presentation::Plain => Presentation::Eldritch,
        Presentation::Eldritch => Presentation::Tampered,
        Presentation::Tampered => Presentation::Plain,
    };
    sim.set_register(next);
    next
}

/// Write the parse trace to [`TRACE_PATH`], and describe what was written.
///
/// # Errors
///
/// If the file cannot be written. **A failed export must not take the session
/// down with it** — the tester whose run it was recording is still playing — so
/// every caller reports and carries on.
pub fn export_trace(sim: &Sim) -> std::io::Result<String> {
    let log = sim.parse_log();
    std::fs::write(TRACE_PATH, log.to_tsv())?;
    Ok(format!(
        "parse trace -> {TRACE_PATH}: {} inputs, {} resolved, {} forced, {} ambiguous, \
         {} incomplete, {} unresolved",
        log.records().len(),
        log.resolved(),
        log.forced(),
        log.ambiguous(),
        log.incomplete(),
        log.unresolved(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_register_walks_all_three_and_comes_back() {
        // A cycle rather than a toggle: `F7` has to reach `Tampered`, which is
        // the one a tester presses it for.
        let mut sim = Sim::new(0);
        assert_eq!(sim.register(), Presentation::Plain);
        assert_eq!(cycle_register(&mut sim), Presentation::Eldritch);
        assert_eq!(cycle_register(&mut sim), Presentation::Tampered);
        assert_eq!(cycle_register(&mut sim), Presentation::Plain);
        assert_eq!(sim.register(), Presentation::Plain, "the sim disagreed");
    }
}
