//! Scrolls, and the errand one of them sets.
//!
//! The lectern assembled a scroll and nothing spent it — a name, a colour and no
//! use, which §15 weighs as a dead end. This file says the other half works.
//!
//! Every claim is driven through a real `Sim`: there is no public way to hand
//! the world a scroll, an errand or a fragment count, so a test that set one
//! would be testing a state the game cannot arrive at.

use orbs_render::{FieldName, Value};
// Most of this file is `cfg(debug_assertions)` because of the scroll: the honest
// route to one is four solved mazes, and `debug_spawn` — which makes it testable
// — does not exist in a release build, where the test would fail against a world
// that was never built.
//
// Gated per test rather than per file, this project's convention: the handful
// below that need no door still run in either profile.

use orbs_sim::Sim;
use orbs_sim::parser::NounKind;

/// Standing in the archive holding enough of the scroll this file is about to
/// open its supply: four, the slowest rate (§19, *Supply*).
///
/// Spawned, not assembled: what four fragments become is drawn, so assembling
/// one would start failing on a coin toss the day a third scroll is authored.
/// [`assembled`] exercises that half.
///
/// A scroll's home is the arsenal, reachable from every room (`tower::keep`).
fn with_a_gleaning_scroll(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    sim.submit("attend archive");
    sim.step();
    sim.submit("debug_spawn gleaning-scroll 4");
    sim.step();
    sim
}

/// Four fragments on the lectern, wielded — whatever that happens to yield.
///
/// The helpers carry the gate too: with `debug_assertions` off every caller is
/// gone and a release build reports this as dead code.
#[cfg(debug_assertions)]
fn assembled(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    sim.submit("attend archive");
    sim.step();
    sim.submit("debug_spawn fragment 4 lectern");
    sim.step();
    sim.submit("wield lectern");
    sim.step_n(25);
    sim
}

/// Every message the orb has said.
fn messages(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| record.field(FieldName::Message))
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// Every scroll the content says the lectern can make.
fn scrolls() -> Vec<String> {
    let recipes = orbs_sim::content::Recipes::builtin();
    recipes
        .outputs()
        .into_iter()
        .filter(|name| recipes.kind_of(name) == NounKind::Scroll)
        .map(str::to_owned)
        .collect()
}

#[test]
fn every_scroll_the_lectern_makes_can_be_spent() {
    // Keeps the content file and the executor honest: a scroll authored with no
    // arm behind it would be drawn, carried, wielded and refused, which reads
    // like a bug in `wield` and costs four walks of the stacks to find out.
    for name in scrolls() {
        assert!(
            orbs_sim::execute::Scroll::of(&name).is_some(),
            "`{name}` is authored in recipes.toml and nothing spends it",
        );
    }
    assert!(!scrolls().is_empty(), "the lectern makes no scroll at all");
}

#[cfg(debug_assertions)]
#[test]
fn four_fragments_become_a_scroll_that_is_a_scroll() {
    // The kind, not just the name: a scroll that came out as a `Reagent` would
    // be `move`-able, `grind`-able and unwieldable (§19).
    let sim = assembled(1);
    let made: Vec<String> = scrolls()
        .into_iter()
        .filter(|scroll| {
            messages(&sim)
                .iter()
                .any(|line| line.contains(scroll.as_str()))
        })
        .collect();
    assert_eq!(
        made.len(),
        1,
        "the lectern made {made:?}, wanted one scroll"
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_gleaning_errand_scatters_spoils_and_withdraws_the_way_out() {
    let mut sim = with_a_gleaning_scroll(3);
    sim.submit("research");
    sim.step();
    let before = sim.stacks().expect("research opens the stacks");
    assert!(before.exit.is_some(), "a fresh maze has a way out");
    assert!(before.spoils.is_empty(), "a fresh maze has nothing in it");

    sim.submit("wield gleaning-scroll");
    sim.step();
    let after = sim.stacks().expect("the stacks are still open");

    // No way out at all, rather than one that does nothing: a solver's top rung
    // is `if <way> has exit`, so an inert exit would have it walk onto that
    // square and take the same rung for ever. The picture has to agree too.
    assert!(
        after.exit.is_none(),
        "a gleaning maze still draws a way out"
    );
    assert_eq!(
        after.spoils.len(),
        4,
        "four spoils, the slowest rate's worth"
    );

    // On floor nobody has walked, so spending the scroll on a half-explored maze
    // is not a partial refund.
    for spoil in &after.spoils {
        assert!(*spoil != after.at, "a spoil was dropped under the reading");
        assert!(
            after.squares[*spoil].marks == 0,
            "a spoil was dropped on walked floor",
        );
        assert!(!after.squares[*spoil].wall, "a spoil was dropped in a wall");
    }
}

#[test]
fn the_lectern_still_assembles_with_an_errand_published_on_it() {
    // The errand is a named child on the lectern, and the lectern is an
    // instrument — so a child counted as stock enters the multiset
    // `Recipes::matching` compares, the recipe stops matching, and the panel
    // reads `fouled` for a lectern with nothing wrong with it. `tower::holdings`
    // skipping `NounKind::Sense` is the fix; driven, because what has to keep
    // working is the recipe.
    let mut sim = with_a_gleaning_scroll(5);
    sim.submit("research");
    sim.step();
    sim.submit("wield gleaning-scroll");
    sim.step();

    sim.submit("debug_spawn fragment 4 lectern");
    sim.step();
    sim.submit("wield lectern");
    sim.step_n(25);

    let made = messages(&sim)
        .iter()
        .filter(|line| scrolls().iter().any(|scroll| line.contains(scroll)))
        .count();
    assert!(
        made >= 2,
        "the lectern stopped assembling once an errand was on it: {:?}",
        messages(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_spell_can_ask_which_errand_the_stacks_are_on() {
    // Why the errand is a named child rather than a `State` variant: one solver
    // that reads what it is walking for. The condition has to resolve at cast,
    // when there is no errand on — `tower::scene_at` chaining `Errand::ALL` onto
    // the readings is what makes that work.
    let mut sim = with_a_gleaning_scroll(7);
    sim.submit("research");
    sim.step();

    // `verify`, because it names its target and does nothing else: a branch that
    // changed the world would make the second casting a different experiment.
    sim.write_spell(
        "asking",
        &[
            "if stacks has gleaning".to_owned(),
            "verify stacks".to_owned(),
            "else".to_owned(),
            "verify north".to_owned(),
            "end".to_owned(),
        ],
    );
    sim.step();

    // Deltas, not totals: a completion files the instrument under `Source` too,
    // so a raw count starts at one and fails on the fixture.
    let before = verified(&sim);

    // Cast with no errand on: the `else` branch.
    sim.submit("invoke asking");
    sim.step_n(8);
    let plain = since(before, verified(&sim));
    assert_eq!(
        plain,
        (0, 1),
        "with no errand on, the spell did not take the `else`",
    );

    let before = verified(&sim);
    sim.submit("wield gleaning-scroll");
    sim.step();
    sim.submit("invoke asking");
    sim.step_n(8);
    let gleaning = since(before, verified(&sim));
    assert_eq!(
        gleaning,
        (1, 0),
        "with the errand on, the spell did not take the `if`",
    );
}

/// What happened between two readings of [`verified`].
#[cfg(debug_assertions)]
const fn since(before: (usize, usize), after: (usize, usize)) -> (usize, usize) {
    (after.0 - before.0, after.1 - before.1)
}

/// How often each of the two branches has run, by what it verified.
///
/// `Source` is where `verify` files its target; `Name` is the verb, the same on
/// both branches. Reading the wrong one looks like the condition failing.
#[cfg(debug_assertions)]
fn verified(sim: &Sim) -> (usize, usize) {
    let named = |wanted: &str| {
        sim.scrollback()
            .records()
            .iter()
            .filter(|record| record.field(FieldName::Source) == Some(Value::Text(wanted)))
            .count()
    };
    (named("stacks"), named("north"))
}

#[cfg(debug_assertions)]
#[test]
fn gleaning_opened_before_research_sets_the_next_maze_gathering() {
    // Open first, then research, is the order a player reaches for, and the
    // one-shot scroll refused it. The supply is a mode: while it is open, a
    // maze opened gathers.
    let mut sim = with_a_gleaning_scroll(11);
    sim.submit("wield gleaning-scroll");
    sim.step();
    assert!(
        messages(&sim)
            .iter()
            .any(|line| line.contains("the next stacks you open will gather 4")),
        "opening with no maze did not say what it will do: {:?}",
        messages(&sim),
    );

    sim.submit("research");
    sim.step();
    let maze = sim.stacks().expect("the stacks are open");
    assert_eq!(maze.spoils.len(), 4, "the maze opened walking for its exit");
    assert!(maze.exit.is_none(), "a gathering maze still has a way out");
}

#[cfg(debug_assertions)]
#[test]
fn gleaning_opened_from_another_room_sets_the_open_maze() {
    // The open maze was found through `Cwd`, so from the laboratory the supply
    // missed it and drew a scroll every 300 ticks for nothing.
    let mut sim = with_a_gleaning_scroll(11);
    sim.submit("research");
    sim.step();
    sim.submit("attend laboratory");
    sim.step();
    sim.submit("wield gleaning-scroll");
    sim.step();
    sim.submit("attend archive");
    sim.step();
    let maze = sim.stacks().expect("the stacks are open");
    assert_eq!(maze.spoils.len(), 4, "{:?}", messages(&sim));
    assert!(
        maze.exit.is_none(),
        "the open maze still walks for its exit"
    );
}

#[cfg(debug_assertions)]
#[test]
fn gleaning_opened_over_a_gathering_maze_says_it_gathers_already() {
    // It promised the next stacks would gather, as if the open ones did not.
    let mut sim = with_a_gleaning_scroll(11);
    for line in [
        "debug_spawn gleaning-scroll 4",
        "research",
        "wield gleaning-scroll",
        "stop gleaning-scroll",
        // Past where the next draw would have come, so the word opens afresh.
        "meditate 300",
        "wield gleaning-scroll",
    ] {
        sim.submit(line);
        sim.step();
    }
    assert!(
        messages(&sim)
            .iter()
            .any(|line| line.contains("the open stacks gather already")),
        "{:?}",
        messages(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn one_gleaning_scroll_sets_nothing_gathering() {
    // The owner's rule: no single item does anything.
    let mut sim = Sim::new(11);
    for line in [
        "attend archive",
        "debug_spawn gleaning-scroll 3",
        "research",
        "wield gleaning-scroll",
    ] {
        sim.submit(line);
        sim.step();
    }
    let maze = sim.stacks().expect("the stacks are open");
    assert!(
        maze.spoils.is_empty(),
        "three scrolls set the maze gathering"
    );
    assert!(
        messages(&sim)
            .iter()
            .any(|line| line.contains("flows from 4")),
        "the refusal did not say how many it wants: {:?}",
        messages(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn spending_a_scroll_takes_no_production_slot() {
    // `CAPACITY` is 1 and `wield` fills it, so going through `tower::begin`
    // would refuse a scroll whenever anything was running — exactly when a
    // player reaches for one. Branching before `start` is what avoids it.
    let mut sim = with_a_gleaning_scroll(13);
    sim.submit("research");
    sim.step();
    // Something in flight: another four fragments, assembling.
    sim.submit("debug_spawn fragment 4 lectern");
    sim.step();
    sim.submit("wield lectern");
    sim.step();

    sim.submit("wield gleaning-scroll");
    sim.step();

    let maze = sim.stacks().expect("the stacks are open");
    assert_eq!(
        maze.spoils.len(),
        4,
        "the scroll was refused for want of a production slot: {:?}",
        messages(&sim),
    );
}

// Both of the next two are `cfg(debug_assertions)` because they read the ladder
// from `execute::dev_spells`, which a release build does not have. Without the
// gate the test binary fails to compile in release — which `cargo test` never
// notices and `cargo check --release --all-targets` does.
#[cfg(debug_assertions)]
#[test]
fn gathering_the_last_spoil_ends_the_walk() {
    // The errand's own completion rule. What is under test is what ends the
    // walk, not whether a ladder can find its way — `tests/solver.rs` owns that.
    let mut sim = with_a_gleaning_scroll(17);
    sim.submit("research");
    sim.step();
    sim.submit("wield gleaning-scroll");
    sim.step();

    // Walk the whole maze, taking whatever is nearest. A gleaning maze publishes
    // no exit, so this cannot end early by arriving somewhere.
    //
    // Four spoils is a much longer walk than one exit: four scattered squares
    // means crossing the maze four times over, against `tests/solver.rs`'s 6500
    // ticks for reaching one.
    sim.write_spell("sweeping", &threading());
    sim.step();
    sim.submit("invoke sweeping");
    // ~2.5x the measured walk, not eight times it. A step runs whether the walk
    // is over or not, so the surplus is pure suite time.
    sim.step_n(22_000);

    assert!(
        sim.stacks().is_none(),
        "the walk did not end when the spoils ran out",
    );
    // Four spoils and the walk's own completion: `finish_walk` pays only
    // for the last step when that step also gathered, so four is four.
    assert_eq!(
        fragments(&sim),
        4,
        "a gleaning run paid the wrong number of fragments",
    );
}

#[cfg(debug_assertions)]
#[test]
fn one_ladder_solves_a_maze_walked_for_its_exit_and_one_set_to_gather() {
    // "One solver for both errands" was a doc comment, and `sweeper` was only
    // ever pointed at a gleaning maze — a regression in the exit half would have
    // left this file green. Both halves below run the same `threading()` text,
    // the file `debug_spell` hands a tester.
    //
    // Four seeds: `tests/solver.rs` carries the wide sweep at 12, and this asks
    // the narrower question. What costs is the headroom rather than the count —
    // a step runs whether the walk is over or not, so these budgets are ~2.5x
    // the measured worst rather than four times it.
    const SEEDS: [u64; 4] = [3, 11, 17, 23];
    const EXIT_TICKS: u64 = 13_000;
    const GLEAN_TICKS: u64 = 22_000;

    for seed in SEEDS {
        // The ordinary errand: a way out, and one fragment for reaching it.
        let mut exit = Sim::new(seed);
        exit.submit("attend archive");
        exit.step();
        exit.submit("research");
        exit.step();
        exit.write_spell("threading", &threading());
        exit.step();
        exit.submit("invoke threading");
        exit.step_n(EXIT_TICKS);
        assert!(
            fragments(&exit) >= 1,
            "seed {seed}: the ladder never reached the way out",
        );

        // The gathering errand: no way out at all, four things to pick up.
        let mut glean = with_a_gleaning_scroll(seed);
        glean.submit("research");
        glean.step();
        glean.submit("wield gleaning-scroll");
        glean.step();
        glean.write_spell("threading", &threading());
        glean.step();
        glean.submit("invoke threading");
        glean.step_n(GLEAN_TICKS);
        assert_eq!(
            fragments(&glean),
            4,
            "seed {seed}: the same ladder did not gather a maze set to gather",
        );
    }
}

/// How many fragments the archive's shelf holds.
#[cfg(debug_assertions)]
fn fragments(sim: &Sim) -> u32 {
    // Wherever the rule says they live: a walk and `debug_spawn` both ask
    // `tower::home`, and a test naming the room is a third opinion that drifts.
    let world = sim.world();
    let shelf = orbs_sim::tower::home(world, "fragment").expect("a fragment has nowhere to live");
    orbs_sim::tower::holdings(world, shelf)
        .into_iter()
        .filter(|(name, _)| name == "fragment")
        .map(|(_, units)| units)
        .sum()
}

/// The ladder a tester gets from `debug_spell`, read from the content file.
///
/// The same text, not a second copy: two expressions of one algorithm that had
/// to agree and nothing made them (§19). Reading `dev_spells.toml` makes the
/// ladder under test and the one a tester is handed the same by construction.
///
/// `tests/solver.rs` keeps its own on purpose: it pins a tick budget, and the
/// four always-false `spoil` rungs here would blow it by ~2800 ticks.
#[cfg(debug_assertions)]
fn threading() -> Vec<String> {
    orbs_sim::execute::dev_spells()
        .iter()
        .find(|(name, _)| *name == "threading")
        .map(|(_, spell)| spell.lines.clone())
        .expect("dev_spells.toml has no threading ladder")
}

#[cfg(debug_assertions)]
#[test]
fn abandoning_a_gleaning_maze_takes_the_word_with_it() {
    // The errand is a child node, so it can outlive what it describes — the
    // shape §19 records for the four ways keeping a solved maze's readings. A
    // stale `gleaning` would be worse: nothing on screen contradicts it, and
    // every later cast would take the gathering branch.
    let mut sim = with_a_gleaning_scroll(19);
    sim.submit("research");
    sim.step();
    sim.submit("wield gleaning-scroll");
    sim.step();
    assert!(asks_gleaning(&mut sim), "the errand was never published");

    sim.submit("stop stacks");
    sim.step();
    assert!(
        !asks_gleaning(&mut sim),
        "the word outlived the stacks it described",
    );

    // And with the supply rolled up, the stacks open again for their exit, not
    // still gathering. Open, a supply would set the new maze gathering too.
    sim.submit("stop gleaning-scroll");
    sim.step();
    sim.submit("research");
    sim.step();
    let maze = sim.stacks().expect("research opens the stacks");
    assert!(maze.exit.is_some(), "the new maze inherited the errand");
    assert!(!asks_gleaning(&mut sim), "so did the word");
}

/// What a spell would be told if it asked, right now.
///
/// Driven through a real cast: the mechanism is `watch::ask` resolving a named
/// child, and asserting on the child would skip it.
#[cfg(debug_assertions)]
fn asks_gleaning(sim: &mut Sim) -> bool {
    let before = verified(sim);
    sim.write_spell(
        "probe",
        &[
            "if stacks has gleaning".to_owned(),
            "verify stacks".to_owned(),
            "else".to_owned(),
            "verify north".to_owned(),
            "end".to_owned(),
        ],
    );
    sim.step();
    sim.submit("invoke probe");
    sim.step_n(8);
    since(before, verified(sim)).0 > 0
}

/// Standing in the laboratory with a long distillation running and four
/// quickening scrolls, the slowest rate's float.
///
/// The alembic, because 56 ticks is the longest run in the game and a halving is
/// unmistakable against it.
#[cfg(debug_assertions)]
fn distilling(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    for line in [
        "attend laboratory",
        "kindle charcoal",
        "debug_spawn clarified-draught",
        "debug_spawn quickening-scroll 4",
        "distil clarified-draught",
    ] {
        sim.submit(line);
        sim.step();
    }
    sim
}

/// How far through the one run in flight the tower is, as `(done, total)`.
#[cfg(debug_assertions)]
fn meter(sim: &Sim) -> Option<(u64, u64)> {
    sim.instruments()
        .into_iter()
        .find_map(|instrument| instrument.meter)
        .map(|meter| (meter.done, meter.total))
}

#[cfg(debug_assertions)]
#[test]
fn quickening_halves_what_is_left_and_the_run_lands_early() {
    // The interval is the whole effect: `Working { started, ends }` is what
    // `land::finish` compares and what the meter derives from, so moving `ends`
    // carries to the completion, the panel and every frontend.
    let mut sim = distilling(1);
    sim.step_n(10);
    let (_, before) = meter(&sim).expect("the alembic is running");

    sim.submit("wield quickening-scroll");
    sim.step();
    let (_, after) = meter(&sim).expect("the alembic is still running");
    assert!(
        after < before,
        "the run was not shortened: {before} then {after}",
    );

    // Halved from now, not from the start: ten ticks into 56 leaves 46, so the
    // total becomes 10 + 23 = 33. Halving the whole interval would refund time
    // already spent.
    assert_eq!(after, 33, "halved from the wrong point: {after}");

    sim.step_n(30);
    assert!(
        messages(&sim).iter().any(|line| line.contains("clarity")),
        "the quickened run did not land: {:?}",
        messages(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_quickened_run_is_the_same_however_the_ticks_are_taken() {
    // `meditate` collapses hundreds of ticks inside one `step`, and in-flight
    // state must be identical either way: a halving re-derived per tick would
    // shrink the run geometrically.
    let mut watched = distilling(2);
    let mut skipped = distilling(2);
    // `meditate 9`, for ten ticks: the `step` that runs the command advances the
    // clock itself and then drains the skip, so `meditate n` costs `n + 1`.
    for _ in 0..10 {
        watched.step();
    }
    skipped.submit("meditate 9");
    skipped.step();

    for sim in [&mut watched, &mut skipped] {
        sim.submit("wield quickening-scroll");
        sim.step();
    }
    assert_eq!(meter(&watched), meter(&skipped), "the two clocks disagree");

    for _ in 0..40 {
        watched.step();
    }
    skipped.submit("meditate 39");
    skipped.step();
    // What the run said, not how much was said: `meditate` narrates itself, so
    // counting the skipped arm's lines would measure the fixture.
    let landed = |sim: &Sim| {
        messages(sim)
            .into_iter()
            .filter(|line| line.contains("clarity"))
            .collect::<Vec<_>>()
    };
    assert!(!landed(&watched).is_empty(), "the watched run never landed");
    assert_eq!(
        landed(&watched),
        landed(&skipped),
        "a quickened run finished differently for the two clocks",
    );
    assert_eq!(meter(&watched), meter(&skipped), "and left different state");
}

#[cfg(debug_assertions)]
#[test]
fn opening_quickening_records_the_room_and_the_window() {
    // Rule 4: where and for how long are fields a `sift` can find, not only
    // words in a sentence. Three hundred to the next draw, and one over.
    let mut sim = distilling(3);
    sim.submit("wield quickening-scroll");
    sim.step();
    let remaining = sim.scrollback().records().iter().find_map(|record| {
        (record.field(FieldName::Name) == Some(Value::Text("quickening-scroll"))
            && record.field(FieldName::At) == Some(Value::Text("laboratory")))
        .then(|| record.field(FieldName::Remaining))
        .flatten()
    });
    assert_eq!(remaining, Some(Value::Count(301)));
}

#[cfg(debug_assertions)]
#[test]
fn quickening_with_nothing_running_makes_the_next_run_short() {
    // Quicken the laboratory, then brew, is the obvious way to spend a scroll —
    // and it refused for want of something to hurry.
    let mut sim = Sim::new(3);
    for line in [
        "attend laboratory",
        "debug_spawn quickening-scroll 4",
        "wield quickening-scroll",
        "kindle charcoal",
        "debug_spawn clarified-draught",
        "distil clarified-draught",
    ] {
        sim.submit(line);
        sim.step();
    }

    // 56 ticks of distillation, started inside the window.
    let (_, total) = meter(&sim).expect("the alembic is running");
    assert_eq!(total, 28, "the run did not start quick: {total}");
}

#[cfg(debug_assertions)]
#[test]
fn a_run_started_in_the_window_stays_short_when_it_closes() {
    // Speed follows heat: §10.1 checks the athanor when a run begins and lets it
    // finish even if the fire dies under it, because pausing would be the
    // countdown §19 refused.
    //
    // The window has to actually close, which the first version quietly skipped
    // by meditating 25 ticks against a 120-tick window. So the run starts near
    // the end of the window and the boundary is crossed while it is going.
    let mut sim = Sim::new(5);
    for line in [
        "attend laboratory",
        "debug_spawn quickening-scroll 4",
        "wield quickening-scroll",
        "kindle charcoal",
        "debug_spawn clarified-draught",
    ] {
        sim.submit(line);
        sim.step();
    }

    // Most of the way through the window. Charcoal burns 600, so the fire holds.
    sim.submit("meditate 280");
    sim.step();
    sim.submit("distil clarified-draught");
    sim.step();

    let (_, total) = meter(&sim).expect("the alembic is running");
    assert_eq!(total, 28, "the run did not start quick: {total}");
    assert!(quickened(&sim), "the window closed before the run began");

    // Over the boundary, with the run still going.
    sim.submit("meditate 20");
    sim.step();
    assert!(
        !quickened(&sim),
        "the window did not close — this test is measuring nothing again",
    );
    let (_, after) = meter(&sim).expect("the alembic is still running");
    assert_eq!(after, 28, "the run's length moved under it: {after}");

    // ...and it lands at the short length rather than the long one.
    sim.submit("meditate 10");
    sim.step();
    assert!(
        messages(&sim).iter().any(|line| line.contains("clarity")),
        "the quickened run did not land inside its shortened interval",
    );
}

/// Whether the laboratory is working at double speed right now.
#[cfg(debug_assertions)]
fn quickened(sim: &Sim) -> bool {
    let world = sim.world();
    let shelf = orbs_sim::tower::home(world, "sage").expect("sage has nowhere to live");
    orbs_sim::tower::quickened(world, shelf)
}

#[cfg(debug_assertions)]
#[test]
fn quickening_also_hurries_what_is_already_running() {
    // One rule, not two: the state means this room works at double speed, and a
    // run in flight is something the room is doing.
    let mut sim = distilling(7);
    sim.step_n(10);
    let (_, before) = meter(&sim).expect("the alembic is running");

    sim.submit("wield quickening-scroll");
    sim.step();
    let (_, after) = meter(&sim).expect("the alembic is still running");

    // Halved from now, not from the start: ten ticks into 56 leaves 46, so the
    // total becomes 10 + 23 = 33.
    assert_eq!(
        after, 33,
        "halved from the wrong point: {before} then {after}"
    );
}

/// A fouled mortar in a laboratory that is, or is not, quickened.
///
/// The mortar rather than the alembic: `PURGE_TICKS` is four, so the whole
/// effect is two ticks, and the fixture has to be exact rather than roomy.
#[cfg(debug_assertions)]
fn scouring(quickened: bool) -> Sim {
    let mut sim = Sim::new(1);
    for line in [
        "attend laboratory",
        "debug_spawn quickening-scroll 4",
        "grind sage",
    ] {
        sim.submit(line);
        sim.step();
    }
    // The grind lands, leaving husks — which is the thing there is to scour.
    sim.step_n(9);
    if quickened {
        sim.submit("wield quickening-scroll");
        sim.step();
    }
    sim.submit("purge mortar_and_pestle");
    sim.step();
    sim
}

/// Ticks from a scour beginning to the mortar reporting itself clean.
#[cfg(debug_assertions)]
fn scoured_after(sim: &mut Sim) -> u64 {
    for tick in 1..40 {
        sim.step();
        if messages(sim).iter().any(|line| line.contains("clean")) {
            return tick;
        }
    }
    panic!("the scour never finished");
}

#[cfg(debug_assertions)]
#[test]
fn a_quickened_room_scours_quickly_too() {
    // `Triaging` was inserted with a raw `PURGE_TICKS`, so §19's rule was false
    // for a scour and nothing said so. The ratio rather than the two numbers:
    // `PURGE_TICKS` is a placeholder the balance CLI sweeps.
    let plain = scoured_after(&mut scouring(false));
    let quick = scoured_after(&mut scouring(true));
    assert!(
        quick < plain,
        "a quickened scour took as long as a plain one: {plain} then {quick}",
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_window_that_has_closed_is_not_written_into_the_save() {
    // `Cooling`'s rule, one component over: nothing removes `Quickened`, so a
    // bare capture wrote a dead span into every autosave after the first scroll.
    // The control matters as much as the case — without it this passes against a
    // capture that has stopped writing the window at all.
    let mut sim = distilling(1);
    sim.submit("wield quickening-scroll");
    sim.step();
    assert!(
        sim.snapshot()
            .to_toml()
            .expect("a save writes")
            .contains("quickened"),
        "an open window is not in the save at all — the case below is vacuous",
    );

    // Past the window: four scrolls draw once, run dry at the next draw, and
    // the last window closes a tick later, at 301.
    sim.submit("meditate 400");
    sim.step();
    assert!(
        !sim.snapshot()
            .to_toml()
            .expect("a save writes")
            .contains("quickened"),
        "a closed window is still being written into the save",
    );
}

/// What the laboratory's shelf holds, by name.
#[cfg(debug_assertions)]
fn shelved(sim: &Sim) -> Vec<String> {
    let world = sim.world();
    let shelf = orbs_sim::tower::home(world, "sage").expect("sage has nowhere to live");
    orbs_sim::tower::holdings(world, shelf)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

/// Standing in the laboratory with a verdant supply opened on `count` scrolls,
/// raised `raises` times.
#[cfg(debug_assertions)]
fn cultivating(count: u32, raises: usize) -> Sim {
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim.submit(&format!("debug_spawn verdant-scroll {count}"));
    sim.step();
    for _ in 0..=raises {
        sim.submit("wield verdant-scroll");
        sim.step();
    }
    sim
}

/// How far the next herb has grown.
#[cfg(debug_assertions)]
fn grown(sim: &Sim) -> u32 {
    sim.world()
        .resource::<orbs_sim::tower::Cultivation>()
        .grown()
}

/// Whether `herb` is on the laboratory's shelf.
#[cfg(debug_assertions)]
fn has(sim: &Sim, herb: &str) -> bool {
    shelved(sim).iter().any(|name| name == herb)
}

#[cfg(debug_assertions)]
#[test]
fn verdant_flowers_once_enough_is_drawn_and_keeps_its_progress() {
    // Five scrolls: two draws grow 2, and four are needed to draw a third.
    let mut sim = cultivating(5, 0);
    sim.submit("meditate 700");
    sim.step();
    assert!(!has(&sim, "amber"), "two draws flowered a herb");
    assert_eq!(
        grown(&sim),
        2,
        "the growth was lost when the supply ran dry"
    );
    assert!(
        messages(&sim).iter().any(|line| line.contains("runs dry")),
        "{:?}",
        messages(&sim),
    );

    // Restocked, the third draw flowers it.
    sim.submit("debug_spawn verdant-scroll 4");
    sim.step();
    sim.submit("wield verdant-scroll");
    sim.step();
    assert!(has(&sim, "amber"), "the kept growth did not count");
    assert_eq!(grown(&sim), 0);
}

#[cfg(debug_assertions)]
#[test]
fn a_faster_rate_flowers_sooner() {
    let mut slow = cultivating(9, 0);
    // Nine: the word draws one, and the raise wants eight still held.
    let mut fast = cultivating(9, 1);
    // One at the word, and the raise grows only the difference: two, as the
    // faster rate's first draw would.
    assert_eq!(grown(&slow), 1);
    assert_eq!(grown(&fast), 2);
    // The next draw, on the first word's clock, grows one more in each: seven
    // held is short of the faster rate, so it steps down. Three flowers.
    for sim in [&mut slow, &mut fast] {
        sim.submit("meditate 298");
        sim.step();
    }
    assert!(!has(&slow, "amber"), "two slow draws flowered a herb");
    assert!(
        has(&fast, "amber"),
        "the faster rate did not flower on its second draw"
    );
}

#[cfg(debug_assertions)]
#[test]
fn herbs_flower_alphabetically_then_the_supply_rolls_itself_up() {
    // Fixed for every seed: a roll here would be a second draw stacked on the
    // lectern's.
    let mut sim = cultivating(40, 1);
    sim.submit("meditate 1500");
    sim.step();
    let flowered: Vec<String> = messages(&sim)
        .into_iter()
        .filter(|line| line.contains("flowers on the shelf"))
        .collect();
    assert_eq!(
        flowered,
        [
            "amber flowers on the shelf, and stays",
            "mugwort flowers on the shelf, and stays",
            "valerian flowers on the shelf, and stays",
        ],
    );
    assert!(
        messages(&sim)
            .iter()
            .any(|line| line.contains("every herb has flowered")),
        "{:?}",
        messages(&sim),
    );
    let open = sim
        .world()
        .resource::<orbs_sim::tower::supply::Supplies>()
        .get("verdant-scroll")
        .is_some();
    assert!(!open, "the supply went on drawing with nothing to grow");

    // Only the herbs. The first version asked `Recipes::outputs`, not `leaves`,
    // and shelved `dregs`, `ash` and a `fragment` as endless stock.
    for never in ["dregs", "ash", "husks", "phlegm", "fragment", "potash"] {
        assert!(!has(&sim, never), "`{never}` is not a herb and flowered");
    }

    // And a word with nothing left to grow draws nothing.
    let held = orbs_sim::tower::supply::held_in_arsenal(sim.world(), "verdant-scroll");
    sim.submit("wield verdant-scroll");
    sim.step();
    assert_eq!(
        orbs_sim::tower::supply::held_in_arsenal(sim.world(), "verdant-scroll"),
        held,
        "a scroll was drawn with nothing to grow",
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_herb_the_swap_has_renamed_is_not_grown_again() {
    // A reagent swap renames sage to `sage-` for a while. Asked by name, the
    // shelf held no sage, and verdant grew it as a new herb.
    let mut sim = cultivating(40, 1);
    sim.submit("meditate 1500");
    sim.step();
    let world = sim.world_mut();
    let shelf = orbs_sim::tower::home(world, "sage").expect("sage has nowhere to live");
    let sage = orbs_sim::tower::children_of(world, shelf)
        .into_iter()
        .find(|node| {
            world
                .get::<orbs_sim::tower::Name>(*node)
                .is_some_and(|name| name.0 == "sage")
        })
        .expect("sage is on the shelf");
    orbs_sim::tower::substitute(world, sage, "sage-");

    sim.submit("wield verdant-scroll");
    sim.step();
    assert!(
        !messages(&sim)
            .iter()
            .any(|line| line.contains("sage flowers")),
        "a swapped sage was grown as a new herb",
    );
    assert!(
        messages(&sim)
            .iter()
            .any(|line| line.contains("the shelf holds every herb")),
        "{:?}",
        messages(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn sage_stays_endless_and_a_flowered_herb_stays_flowered() {
    // A base reagent that ran out would make a recipe written against it work
    // for a while and then stop.
    let mut sim = cultivating(8, 1);
    sim.submit("meditate 2000");
    sim.step();
    let world = sim.world();
    let shelf = orbs_sim::tower::home(world, "sage").expect("sage has nowhere to live");
    for herb in ["sage", "amber"] {
        assert_eq!(
            orbs_sim::tower::held(world, shelf, herb),
            Some(orbs_sim::tower::Stock::Endless),
            "{herb} is exhaustible",
        );
    }
}

#[cfg(debug_assertions)]
#[test]
fn growth_travels_in_the_save() {
    let mut sim = cultivating(5, 0);
    sim.submit("meditate 301");
    sim.step();
    assert_eq!(grown(&sim), 2);
    let text = sim.snapshot().to_toml().expect("a save writes");
    let loaded = Sim::restored(&orbs_sim::Save::from_toml(&text).expect("a save reads"));
    assert_eq!(grown(&loaded), 2, "growth was lost to a quit");
}

#[cfg(debug_assertions)]
#[test]
fn the_ported_herbs_reach_a_potion() {
    // Driven the whole way: a route that reads well in `recall` and cannot be
    // walked is a table rather than content, which the `dust` recipe was.
    // Amber on the first draw after the raise, mugwort on the next.
    let mut sim = cultivating(20, 1);
    for line in ["meditate 600", "kindle charcoal", "grind mugwort"] {
        sim.submit(line);
        sim.step();
    }
    sim.step_n(9);
    for (line, wait) in [
        ("empty mortar_and_pestle", 1),
        ("digest ground-mugwort", 14),
        ("grind amber", 9),
        ("empty mortar_and_pestle", 1),
        ("mix mugwort-tincture with powdered-amber", 12),
        ("distil keen-draught", 45),
    ] {
        sim.submit(line);
        sim.step_n(wait);
    }

    assert!(
        messages(&sim)
            .iter()
            .any(|line| line.contains("yields") && line.contains("insight")),
        "the ported route does not reach its potion: {:?}",
        messages(&sim),
    );
}
