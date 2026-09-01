//! Every shipped dev spell, run against the game it claims to solve.
//!
//! # The gap this closes
//!
//! `dev_spells.toml` ships ten solvers and, until this file, **nothing ran any
//! of them.** They were reached only by See-it lines and by whichever integration
//! test happened to `invoke` one — so a spell could stop compiling, spin for
//! ever, or silently do nothing, with the whole gate green. §19 records exactly
//! that happening twice: `chanting` stopped compiling and was noticed by a
//! person, and `besieging` shipped with a `repeat until` guard that could never
//! come true on a loss.
//!
//! # What a solver has to do to pass here
//!
//! Four things, and the middle two are the ones a broken spell fails:
//!
//! 1. **Compile clean** — no line the orb cannot read, which is what a renamed
//!    reading or a withdrawn word breaks.
//! 2. **Terminate** — the loop guard must actually become true. A `repeat until`
//!    that cannot be satisfied spins one instruction a tick for ever, quietly.
//! 3. **Do its work** — a spell that runs and achieves nothing passes 1 and 2.
//! 4. **Latch no fault** — `Mark::Fault` means the orb gave up or could not read
//!    a line, and a shipped solver must never earn one.
//!
//! # Why the budget matters here and nowhere else
//!
//! `SCRIPT_BUDGET` is 1 until the weave grants more, and two of these solvers are
//! *meant* to fail at one step a tick — `chanting` is the menagerie's whole
//! progression hook (§19). So each spell declares what it needs rather than the
//! file assuming one number.

#![cfg(debug_assertions)]

use orbs_render::{FieldName, Value};
use orbs_sim::{Sim, tower};

/// A solver, and what it takes to judge it.
struct Solver {
    /// The spell, as `dev_spells.toml` names it.
    name: &'static str,
    /// Where to stand, and what to have, before casting.
    setup: &'static [&'static str],
    /// How long to give it.
    ticks: u64,
    /// The log its work lands in.
    ///
    /// **A spell's own records go to the log, never the transcript** — §19:
    /// `prompt.rs` draws *"what the player did, not what their spells did"*, so
    /// a test looking for a solver's output on the pane finds nothing and looks
    /// broken. This is where the evidence actually is.
    log: &'static str,
    /// A phrase the world says once it has worked, or `None` for *anything*.
    ///
    /// **A sentence the world emits, never one the spell contains.** Matching
    /// the spell's own text would pass for a spell that was merely cast.
    ///
    /// `None` where the domain's success is not one sentence — a chant that
    /// collapses has still *run*, and the menagerie's solver is meant to fall
    /// short at one step a tick (§19).
    did: Option<&'static str>,
}

/// Every solver this file drives.
///
/// **Not derived from `dev_spells.toml`**, deliberately: what counts as *done*
/// is different for each domain and cannot be read off the file. A spell added
/// there and not here is caught by
/// [`every_shipped_solver_is_driven_by_this_file`].
const SOLVERS: &[Solver] = &[
    // --- the bailey (§5.1). Four solvers, one domain, and the differences
    // between them are the point rather than duplication: `besieging` is the
    // ladder, `answering` reads the telegraph, `sparing` hoards, `bulwark`
    // composes with parts.
    Solver {
        name: "besieging",
        setup: &[
            "attend bailey",
            "debug_spawn troop 6",
            "debug_spawn warding 6",
        ],
        ticks: 900,
        log: "bailey.log",
        did: Some("the wall"),
    },
    Solver {
        name: "answering",
        setup: &[
            "attend bailey",
            "debug_spawn troop 6",
            "debug_spawn warding 6",
        ],
        ticks: 900,
        log: "bailey.log",
        did: Some("the wall"),
    },
    // **The one that spends the far side's arithmetic.** `for each die`, a
    // comparison naming a different reading over there, and `double` — none of
    // which any other shipped solver uses, so without this row the grammar has
    // no worked example anything runs.
    Solver {
        name: "sparingly",
        setup: &[
            "attend bailey",
            "debug_spawn troop 6",
            "debug_spawn warding 6",
        ],
        ticks: 900,
        log: "bailey.log",
        did: Some("the wall"),
    },
    Solver {
        name: "sparing",
        setup: &[
            "attend bailey",
            "debug_spawn troop 6",
            "debug_spawn warding 6",
        ],
        ticks: 900,
        log: "bailey.log",
        did: Some("the wall"),
    },
    // The two that allocate dice. **`did` is a pledge landing**, not a win: a
    // spell that fights well and never pledges passes every other check here,
    // and that is exactly what `warding_off` did for its first draft — `if area
    // is empty and no moot` mixes `is` with an elided `has no`, so the guard
    // never fired and it pledged nothing, in all seventeen seeds.
    Solver {
        name: "steadfast",
        setup: &["attend bailey"],
        ticks: 900,
        log: "bailey.log",
        did: Some("goes behind"),
    },
    Solver {
        name: "warding_off",
        setup: &["attend bailey"],
        ticks: 900,
        log: "bailey.log",
        did: Some("goes behind"),
    },
    Solver {
        name: "bulwark",
        setup: &[
            "attend bailey",
            "debug_spawn troop 6",
            "debug_spawn warding 6",
        ],
        ticks: 900,
        log: "bailey.log",
        did: Some("the wall"),
    },
    // --- the sanctum.
    Solver {
        name: "holding",
        setup: &["attend sanctum"],
        ticks: 600,
        log: "sanctum.log",
        did: Some("barrier"),
    },
    Solver {
        name: "coursing",
        setup: &[
            "attend sanctum",
            "debug_take satchel_1",
            "debug_take cursors_1",
        ],
        ticks: 600,
        log: "sanctum.log",
        did: Some("wellspring"),
    },
    // --- the archive. **`repeat 200` in `assembling` is ~500 ticks**, which is
    // why this one is not given the same budget as the bailey's.
    Solver {
        name: "assembling",
        setup: &["attend archive", "debug_spawn fragment 4"],
        ticks: 900,
        log: "archive.log",
        did: Some("scroll"),
    },
    Solver {
        name: "threading",
        setup: &["attend archive", "research"],
        ticks: 7200,
        log: "archive.log",
        did: Some("the reading goes"),
    },
    Solver {
        name: "roaming",
        setup: &["attend archive", "research"],
        ticks: 7200,
        log: "archive.log",
        did: Some("the reading goes"),
    },
    // --- the lens.
    Solver {
        name: "breaking",
        setup: &["attend lens"],
        ticks: 3600,
        log: "lens.log",
        did: None,
    },
    // --- the menagerie. **`did: None`, and that is the domain working.** A
    // chant at one step a tick collapses rather than finishing (§19) — it is the
    // weave's progression hook — so what is asserted is that it *ran*, not that
    // it won.
    Solver {
        name: "chanting",
        setup: &["attend menagerie"],
        ticks: 400,
        log: "menagerie.log",
        did: None,
    },
    // --- the laboratory's two-spell channel. `ordering` is a producer that
    // stops after three queues; `milling` invokes it and grinds what it left.
    Solver {
        name: "ordering",
        setup: &["attend laboratory", "debug_take satchel_1"],
        ticks: 60,
        log: "laboratory.log",
        did: Some("satchel"),
    },
    Solver {
        name: "milling",
        setup: &["attend laboratory", "debug_take satchel_1"],
        ticks: 200,
        log: "laboratory.log",
        did: Some("ground-sage"),
    },
];

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

fn said(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| match record.field(FieldName::Message) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// Cast `solver` and let it work. Returns the sim for further questions.
fn drive(solver: &Solver, seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    for line in solver.setup {
        run(&mut sim, line);
    }
    run(&mut sim, &format!("invoke {}", solver.name));
    sim.step_n(solver.ticks);
    sim
}

#[test]
fn every_shipped_solver_compiles_without_a_complaint() {
    // **The failure this catches is silent.** A renamed reading or a withdrawn
    // word leaves the spell casting perfectly and every question answering
    // *"that question means nothing"* — the loop guard is then false from the
    // first evaluation and the spell ends having done nothing. §19 records
    // `besieging` shipping in exactly that state for one commit.
    for solver in SOLVERS {
        let sim = drive(solver, 11);
        let bad: Vec<String> = said(&sim)
            .into_iter()
            .filter(|line| line.contains("means nothing") || line.contains("cannot read"))
            .collect();
        assert!(
            bad.is_empty(),
            "{} has lines the orb cannot read: {bad:#?}",
            solver.name,
        );
    }
}

#[test]
fn every_shipped_solver_does_its_work() {
    // A spell that casts, runs and achieves nothing passes every check that only
    // asks whether it compiled.
    //
    // **The evidence is in the log, not the transcript** — §19: the pane draws
    // *"what the player did, not what their spells did"*, so a `repeat` loop
    // would otherwise push the player's own last line off screen in seconds.
    for solver in SOLVERS {
        let mut sim = drive(solver, 11);
        run(&mut sim, &format!("peruse {}", solver.log));
        let logged = said(&sim);
        match solver.did {
            Some(phrase) => assert!(
                logged.iter().any(|line| line.contains(phrase)),
                "{} ran for {} ticks and its log never said {phrase:?}",
                solver.name,
                solver.ticks,
            ),
            // No one sentence marks success here; that the log grew at all is
            // the claim. A spell that did nothing writes nothing.
            None => assert!(
                logged.len() > 4,
                "{} ran for {} ticks and wrote nothing to {}",
                solver.name,
                solver.ticks,
                solver.log,
            ),
        }
    }
}

#[test]
fn no_shipped_solver_latches_a_fault() {
    // `Mark::Fault` is the orb giving up or failing to read a line. A solver
    // shipped as a worked example must never earn one — it is the first thing a
    // player copying it would inherit.
    for solver in SOLVERS {
        let sim = drive(solver, 11);
        // A fault reaches the player through the rail, so that is where it is
        // read from — `Sim::briefs` is the same walk the rail paints.
        let faults: Vec<&'static str> = sim
            .briefs()
            .into_iter()
            .filter(|brief| brief.mark == Some(tower::Mark::Fault))
            .map(|brief| brief.name)
            .collect();
        assert!(
            faults.is_empty(),
            "{} latched a fault: {faults:?}\n{:#?}",
            solver.name,
            said(&sim),
        );
    }
}

#[test]
fn every_shipped_solver_terminates_rather_than_spinning() {
    // **A `repeat until` whose guard can never come true spins one instruction a
    // tick for ever, silently.** `besieging` shipped asking `the enemy has routed`
    // — true only on a *win* — so a lost siege left it looping and refusing
    // `hold` for the rest of the session.
    //
    // A stopped spell is one nothing is running any more. Given generous time,
    // every solver here should have run off the end.
    for solver in SOLVERS {
        let mut sim = drive(solver, 11);
        sim.step_n(solver.ticks * 2);
        assert!(
            sim.running_line(solver.name).is_none(),
            "{} was still running after {} ticks — its guard may never come true",
            solver.name,
            solver.ticks * 3,
        );
    }
}

#[test]
fn a_solver_reaches_the_same_end_from_the_same_seed() {
    // Rule 3, through the spell runner rather than over the model.
    for solver in SOLVERS {
        let a = said(&drive(solver, 7));
        let b = said(&drive(solver, 7));
        assert_eq!(
            a, b,
            "{} took two different runs from one seed",
            solver.name
        );
    }
}

#[test]
fn every_shipped_solver_is_driven_by_this_file() {
    // **The lint that keeps the list honest.** A solver added to
    // `dev_spells.toml` and not here is a shipped worked example nothing runs,
    // which is the state every spell in that file was in until this file existed.
    let shipped = orbs_sim::execute::dev_spells();
    let mut missed: Vec<String> = shipped
        .iter()
        .map(|(name, _)| name)
        .filter(|name| !SOLVERS.iter().any(|solver| solver.name == *name))
        .map(str::to_owned)
        .collect();
    missed.sort();
    assert!(
        missed.is_empty(),
        "these shipped solvers are driven by nothing: {missed:?}",
    );
}
