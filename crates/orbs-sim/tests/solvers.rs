//! Every shipped dev spell, run against the game it claims to solve.
//!
//! `dev_spells.toml` ships ten solvers and until this file nothing ran any of
//! them, so one could stop compiling, spin for ever or silently do nothing with
//! the gate green. §19 records it happening twice.
//!
//! Four things a solver must do here, and the middle two are what a broken
//! spell fails: compile clean (a renamed reading or a withdrawn word breaks
//! that); terminate (a `repeat until` that cannot be satisfied spins one
//! instruction a tick for ever, quietly); do its work (running and achieving
//! nothing passes the first two); and latch no `Mark::Fault`.
//!
//! `SCRIPT_BUDGET` is 1 until the weave grants more, and a solver may be *meant*
//! to fall short at one step a tick — `tending_blindly` is — so each spell
//! declares what it needs rather than the file assuming one number.

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
    /// A spell's own records go to the log, never the transcript (§19), so a
    /// test looking for a solver's output on the pane finds nothing.
    log: &'static str,
    /// A phrase the world says once it has worked, or `None` for *anything*.
    ///
    /// A sentence the world emits, never one the spell contains — matching the
    /// spell's own text would pass for a spell that was merely cast.
    ///
    /// `None` where success is not one sentence: the lens's sweep breaks as
    /// many wards as the ticks allow, and no line says *done*.
    did: Option<&'static str>,
}

/// Every solver this file drives.
///
/// Not derived from `dev_spells.toml`: what counts as *done* differs per domain
/// and cannot be read off the file. A spell added there and not here is caught
/// by [`every_shipped_solver_is_driven_by_this_file`].
const SOLVERS: &[Solver] = &[
    // --- the bailey (§5.1). Four solvers, one domain: `besieging` is the
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
    // The one that spends the far side's arithmetic: `for each die`, a
    // comparison naming a different reading over there, and `double` — nothing
    // else shipped uses them, so without this row that grammar runs nowhere.
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
    // The two that allocate dice. `did` is a pledge landing, not a win: a spell
    // that fights well and never pledges passes every other check, which is
    // what `warding_off`'s first draft did — `if area is empty and no moot`
    // mixes `is` with an elided `has no`, so the guard never fired.
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
    // --- the forge.
    //
    // `did` is the charm binding: a spell that opens a lattice and snaps at it
    // for ever terminates and latches no fault while doing nothing. What says
    // the eight-rung table is right is a lattice actually lighting.
    Solver {
        name: "forging",
        setup: &["attend forge"],
        ticks: 200,
        log: "forge.log",
        did: Some("every glyph holds"),
    },
    // The maintenance spell, invoked rather than bound because `bind` costs
    // sixteen experience this fixture has not earned. One pass proves the
    // cold-start rung fires; the re-casting is `bind::stand`'s, tested there.
    Solver {
        name: "tending_forge",
        setup: &["attend forge"],
        ticks: 200,
        log: "forge.log",
        did: Some("every glyph holds"),
    },
    // The one meant to fall short, and the table says so rather than the file
    // assuming one outcome. It holds no residue table, so it anneals without
    // reading the board — one attempt in eight lights. It must still open a
    // binding and spend on it, which is the work. `chanting` is the precedent:
    // a shipped solver nothing ran, which stopped compiling, gate still green.
    Solver {
        name: "tending_blindly",
        setup: &["attend forge"],
        ticks: 200,
        log: "forge.log",
        did: Some("the glyphs rise"),
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
    // --- the archive. `repeat 200` in `assembling` is ~500 ticks, so it does
    // not get the bailey's budget.
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
    // --- the menagerie. `did` is a hold: the search tries every circle in
    // order, so a spell that summoned and limned for ever without one holding
    // passes everything else. The worst beast takes 656 steps at one a tick,
    // so 900 leaves room for it and the tail after.
    Solver {
        name: "taming",
        setup: &["attend menagerie"],
        ticks: 900,
        log: "menagerie.log",
        did: Some("the circle holds"),
    },
    // `winnowing` is the same claim with the logic applied: a ladder of rungs and
    // two parts, so the rung a seed's beast lands on is what is exercised. Its
    // worst beast is 90 calls where `taming`'s is 174, and 900 is kept for both.
    Solver {
        name: "winnowing",
        setup: &["attend menagerie"],
        ticks: 900,
        log: "menagerie.log",
        did: Some("the circle holds"),
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
    // A silent failure: a renamed reading leaves the spell casting perfectly
    // and every question answering *"that question means nothing"*, so the loop
    // guard is false from the first evaluation. §19 records `besieging`
    // shipping in that state for one commit.
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
    // A spell that casts, runs and achieves nothing passes every check that
    // only asks whether it compiled. The evidence is in the log, not the
    // transcript (§19).
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
    // A `repeat until` whose guard can never come true spins one instruction a
    // tick for ever. `besieging` shipped asking `the enemy has routed`, true
    // only on a win, so a lost siege left it looping for the session. Given
    // generous time, every solver here should have run off the end.
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
    // The lint that keeps the list honest: a solver added to `dev_spells.toml`
    // and not here is a shipped worked example nothing runs.
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
