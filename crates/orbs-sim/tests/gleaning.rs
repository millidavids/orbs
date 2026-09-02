//! Scrolls, and the errand one of them sets.
//!
//! The archive assembled a scroll from the day the lectern had a recipe, and
//! then did nothing with it — an object with a name, a colour and no use, which
//! §15 weighs as a dead end above the raw resolution rate. This is the file that
//! says the other half works.
//!
//! **Every claim here is driven through a real `Sim`**, in the shape
//! `tests/solver.rs` established: there is no public way to hand the world a
//! scroll, an errand or a fragment count, so a test that reached in and set one
//! would be testing a state the game cannot arrive at.

use orbs_render::{FieldName, Value};
// **Most of this file is `cfg(debug_assertions)`, and the reason is the scroll.**
//
// A `gleaning-scroll` is four fragments assembled at the lectern, and a fragment
// is one finished walk of the stacks — so the honest route to a single scroll is
// four solved mazes, several thousand ticks apiece. `debug_spawn` is the door
// that makes any of this testable at all, and it does not exist in a release
// build: the line is simply unresolvable there, so the scroll never appears and
// the test fails against a world that was never built.
//
// Gated per test rather than per file, which is this project's convention — the
// handful below that need no door still run in either profile. Where a fixture
// *can* be built without one it is (`arsenal`, `fetching` and `binding` all brew
// §10.1's chain from endless stock instead); a scroll cannot.

use orbs_sim::Sim;
use orbs_sim::parser::NounKind;

/// Standing in the archive holding the scroll this file is about.
///
/// **Spawned, not assembled, and the difference is the roll.** What four
/// fragments become is *drawn* from the scrolls the lectern knows, so a test that
/// assembled one and then spent it would be testing the draw as much as the
/// effect — and would start failing the day a third scroll is authored, on a
/// coin toss rather than on anything being wrong. `debug_spawn` names the one
/// under test; [`assembled`] is what exercises the other half.
///
/// A scroll's home is the arsenal, whose contents are reachable from every room
/// (`tower::keep`), so it is in hand in the archive without being carried there.
fn with_a_gleaning_scroll(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    sim.submit("attend archive");
    sim.step();
    sim.submit("debug_spawn gleaning-scroll");
    sim.step();
    sim
}

/// Four fragments on the lectern, wielded — whatever that happens to yield.
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
    // **The lint that keeps the content file and the executor honest**, and the
    // same shape `progression.toml`'s `grants` check has: a word nothing
    // implements gives the step nothing, and the file reads as deliberate.
    //
    // A scroll authored with no arm behind it would be drawn, carried, wielded
    // and refused — which reads to a player exactly like a bug in `wield`, and
    // costs four walks of the stacks to find out.
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
    // The kind, not just the name: `Recipes::kind_of` is what `debug_spawn` and
    // `produce::transmute` both ask, and a scroll that came out as a `Reagent`
    // would be `move`-able, `grind`-able and unwieldable — the wrong-kind state
    // §19 already records `debug_spawn` refusing to create.
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

    // **No way out at all, rather than one that does nothing.** A solver's top
    // rung is `if <way> has exit`; an inert exit would have it walk onto that
    // square, find the walk not over, and take the same rung from the same place
    // for ever. Withdrawing the word is what makes the errand a behaviour change
    // instead of a trap — and the picture has to agree, or the map offers a way
    // out the readings do not.
    assert!(
        after.exit.is_none(),
        "a gleaning maze still draws a way out"
    );
    assert_eq!(
        after.spoils.len(),
        5,
        "five spoils, against a scroll's four"
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
    // **The hazard that is not hypothetical.** The errand is published on the
    // lectern as a named child, exactly as a way publishes `passage` — and the
    // lectern is an *instrument*, so a child counted as stock would enter the
    // multiset `Recipes::matching` compares. Four fragments plus one word is not
    // four fragments: the recipe stops matching, and the panel reads `fouled`
    // for a lectern with nothing wrong with it.
    //
    // `tower::holdings` skipping `NounKind::Sense` is the fix, and this is what
    // holds it. Driven rather than asserted on `holdings`, because what has to
    // keep working is the *recipe*.
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
    // **The point of the whole mechanic**, and the reason the errand is a named
    // child rather than a `State` variant: one solver, written once, that reads
    // what it is walking for. The condition has to resolve at **cast**, which is
    // when there is no errand on — `tower::scene_at` chaining `Errand::ALL` onto
    // the readings is the single line that makes that work, exactly as it does
    // for `passage` and `back`.
    let mut sim = with_a_gleaning_scroll(7);
    sim.submit("research");
    sim.step();

    // **`verify`, because it names its target and does nothing else.** Both
    // branches have to be visible without changing the world — a branch that
    // walked or stopped something would make the second casting a different
    // experiment from the first.
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

    // **Deltas, not totals.** The setup ran `wield lectern`, and a completion
    // files the instrument under `Source` too — so a raw count starts at one and
    // the first assertion would fail on the fixture rather than on the feature.
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
const fn since(before: (usize, usize), after: (usize, usize)) -> (usize, usize) {
    (after.0 - before.0, after.1 - before.1)
}

/// How often each of the two branches has run, by what it verified.
///
/// `Source` is where `verify` files its **target** — `Name` is the verb, which
/// is the same on both branches. Reading the wrong one counted nothing at all
/// and looked exactly like the condition failing.
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
fn a_scroll_with_nothing_to_work_on_is_kept() {
    // §7's *"destruction is a tool, not a trap"*, applied to a thing four
    // walks paid for: a refusal that had already spent the scroll would be
    // the worst trade in the game, and it would be invisible until the player
    // went looking for it.
    let mut sim = with_a_gleaning_scroll(11);
    sim.submit("wield gleaning-scroll");
    sim.step();

    assert!(
        messages(&sim)
            .iter()
            .any(|line| line.contains("research first")),
        "the refusal did not name the way forward: {:?}",
        messages(&sim),
    );

    // Still there, and still spendable once the stacks are open.
    sim.submit("research");
    sim.step();
    sim.submit("wield gleaning-scroll");
    sim.step();
    let maze = sim.stacks().expect("the stacks are open");
    assert_eq!(maze.spoils.len(), 5, "the refusal had eaten the scroll");
}

#[cfg(debug_assertions)]
#[test]
fn spending_a_scroll_takes_no_production_slot() {
    // `CAPACITY` is 1, and `wield` is the verb that fills it — so if spending a
    // scroll went through `tower::begin` it would be refused whenever anything
    // was running, which is exactly when a player reaches for one. Branching
    // before `start` is what avoids that; `begins_work` is `const fn(Verb)` and
    // could not have.
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
        5,
        "the scroll was refused for want of a production slot: {:?}",
        messages(&sim),
    );
}

// **Both of the next two are `cfg(debug_assertions)`**, because they read the
// ladder from `execute::dev_spells`, which a release build does not have — the
// whole point of `debug_spell` being a tester's door. Without the gate the test
// *binary* fails to compile in release, which `cargo test --workspace` never
// notices and `cargo check --release --all-targets` does.
#[cfg(debug_assertions)]
#[test]
fn gathering_the_last_spoil_ends_the_walk() {
    // The errand's own completion rule. Walked by hand rather than solved,
    // because what is under test is *what ends it* and not whether a ladder can
    // find its way — `tests/solver.rs` owns the second question.
    let mut sim = with_a_gleaning_scroll(17);
    sim.submit("research");
    sim.step();
    sim.submit("wield gleaning-scroll");
    sim.step();

    // Walk the whole maze, taking whatever is nearest. A gleaning maze publishes
    // no exit, so this cannot end early by arriving somewhere.
    //
    // **Five spoils is a much longer walk than one exit**, and the budget says
    // so: the ladder is twenty-four rungs, a step costs a tick (`SCRIPT_BUDGET`
    // is 1), and five scattered squares means crossing the maze five times over.
    // `tests/solver.rs` pins 6500 ticks for reaching one exit; this is that
    // several times, with room.
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
    // Five spoils and the walk's own completion: `finish_walk` pays only
    // for the last step when that step also gathered, so five is five.
    assert_eq!(
        fragments(&sim),
        5,
        "a gleaning run paid the wrong number of fragments",
    );
}

#[cfg(debug_assertions)]
#[test]
fn one_ladder_solves_a_maze_walked_for_its_exit_and_one_set_to_gather() {
    // **The claim the errand was built for, finally asserted.** It has been true
    // since the spoil rung was written and it was a *doc comment*: `sweeper`
    // said "one solver for both errands" and was only ever pointed at a gleaning
    // maze. A regression that broke the exit half — the `exit` tier reordered, a
    // reading renamed — would have left this file green.
    //
    // What makes it one ladder rather than two is that both halves below run the
    // *same* `threading()` text, which is the file `debug_spell` hands a tester.
    //
    // **Four seeds, and the step budgets are the lever rather than the count.**
    // `tests/solver.rs` carries the wide sweep for the exit errand at 12 seeds;
    // this asks the narrower question — *does one text serve both* — so four is
    // enough to catch an ordering mistake.
    //
    // What costs is the *headroom*, not the seeds: a step runs whether the walk
    // is over or not, so an exit solve measured at ~5100 ticks against a
    // `step_n(20_000)` throws away three quarters of the run, four times over.
    // These are ~2.5x the measured worst, which is margin for a slower seed
    // without paying for eight walks nobody takes.
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

        // The gathering errand: no way out at all, five things to pick up.
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
            5,
            "seed {seed}: the same ladder did not gather a maze set to gather",
        );
    }
}

/// How many fragments the archive's shelf holds.
#[cfg(debug_assertions)]
fn fragments(sim: &Sim) -> u32 {
    // **Wherever the rule says they live**, rather than a room written down
    // here. A walk pays into the archive's shelf and `debug_spawn` puts one in
    // the same place because both ask `tower::home`; a test naming the room
    // instead is a third opinion, and the third opinion is the one that drifts.
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
/// **The same text, not a second copy of it.** This was a `sweeper()` that built
/// the ladder here while `tests/solver.rs` built its own — two expressions of one
/// algorithm that had to agree and nothing made them, which is the shape §19 keeps
/// recording as the cause of defects. Reading `dev_spells.toml` makes *the ladder
/// under test* and *the ladder a tester is handed* the same twenty-four rungs by
/// construction: a change to one is a change to both, and this file fails if the
/// shipped ladder stops working.
///
/// `tests/solver.rs` keeps a ladder of its own **on purpose** — see the note there.
/// It pins a tick budget, and the four always-false `spoil` rungs this one carries
/// would blow it by ~2800 ticks while proving nothing about the exit walk.
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
    // **The errand is a child node, so it can outlive the thing it describes.**
    // §19 already records this exact shape once: the four ways kept a solved
    // maze's readings for ever, so `survey north` answered `passage` with no
    // stacks open and a bound solver was told *"research first"* for the rest
    // of its `repeat`. A stale `gleaning` would be worse, because nothing on
    // screen contradicts it — every later cast of a mode-aware spell would take
    // the gathering branch of a maze that is walked for its exit.
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

    // And the stacks open again for their exit, not still gathering.
    sim.submit("research");
    sim.step();
    let maze = sim.stacks().expect("research opens the stacks");
    assert!(maze.exit.is_some(), "the new maze inherited the errand");
    assert!(!asks_gleaning(&mut sim), "so did the word");
}

/// What a spell would be told if it asked, right now.
///
/// Driven through a real cast rather than read off the world, because what has
/// to be true is the *answer a spell gets* — `watch::ask` resolving a named
/// child is the mechanism, and asserting on the child would skip it.
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

/// Standing in the laboratory with a long distillation running and a scroll.
///
/// The alembic, because 56 ticks is the longest run in the game and a halving is
/// unmistakable against it — a mortar's eight would be hard to tell from a
/// rounding.
fn distilling(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    for line in [
        "attend laboratory",
        "kindle charcoal",
        "debug_spawn clarified-draught",
        "debug_spawn quickening-scroll",
        "distil clarified-draught",
    ] {
        sim.submit(line);
        sim.step();
    }
    sim
}

/// How far through the one run in flight the tower is, as `(done, total)`.
fn meter(sim: &Sim) -> Option<(u64, u64)> {
    sim.instruments()
        .into_iter()
        .find_map(|instrument| instrument.meter)
        .map(|meter| (meter.done, meter.total))
}

#[cfg(debug_assertions)]
#[test]
fn quickening_halves_what_is_left_and_the_run_lands_early() {
    // **The interval is the whole effect.** `Working { started, ends }` is what
    // `land::finish` compares against and what the meter is derived from, so
    // moving `ends` carries to the completion, the panel and every frontend with
    // nothing else changed.
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

    // **Halved from *now*, not from the start.** Ten ticks in on a 56-tick run
    // leaves 46, so the total becomes 10 + 23 = 33. Halving the whole interval
    // would refund time already spent, and past the half-way point would put the
    // end in the past.
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
    // `meditate` collapses hundreds of ticks inside one `step`, and three
    // modules state that in-flight state must be identical either way. A halving
    // re-derived per tick would shrink the run geometrically and this is what
    // would say so.
    let mut watched = distilling(2);
    let mut skipped = distilling(2);
    // **`meditate 9`, for ten ticks.** The `step` that runs the command advances
    // the clock itself and *then* drains the skip, so `meditate n` costs `n + 1`
    // — which is the semantics, not a rounding, and getting it wrong here would
    // have compared two different moments and blamed the scroll.
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
    // **What the run said, not how much was said.** `meditate` narrates itself,
    // so the skipped arm has extra lines that belong to the command rather than
    // to the brew — counting them would measure the fixture. What has to match is
    // the completion.
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
fn quickening_with_nothing_running_makes_the_next_run_short() {
    // **The play the first version could not do.** Quicken the laboratory, then
    // brew, is the obvious way to spend a scroll — and it refused for want of
    // something to hurry, which made four walks of the stacks unusable at exactly
    // the moment a player reaches for one.
    let mut sim = Sim::new(3);
    for line in [
        "attend laboratory",
        "debug_spawn quickening-scroll",
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
    // **Speed follows heat.** §10.1 checks the athanor when a run *begins* and
    // lets it finish even if the fire dies under it, because pausing would be the
    // countdown §19 refused. A run whose duration changed as the window expired
    // would be that countdown wearing a multiplier — and would break the interval
    // `meditate` idempotence rests on.
    //
    // **The window has to actually close**, which is the whole difficulty and
    // which the first version of this test quietly skipped: it meditated 25 ticks
    // against a 120-tick window and asserted a length that nothing had any reason
    // to move. So the run starts *near the end* of the window and the boundary is
    // crossed while it is still going.
    let mut sim = Sim::new(5);
    for line in [
        "attend laboratory",
        "debug_spawn quickening-scroll",
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
fn quickened(sim: &Sim) -> bool {
    let world = sim.world();
    let shelf = orbs_sim::tower::home(world, "sage").expect("sage has nowhere to live");
    orbs_sim::tower::quickened(world, shelf)
}

#[cfg(debug_assertions)]
#[test]
fn quickening_also_hurries_what_is_already_running() {
    // One rule, not two: the state means *this room works at double speed*, and a
    // run in flight is something the room is doing. Leaving it alone would make
    // wielding the scroll mid-brew look like it had done nothing.
    let mut sim = distilling(7);
    sim.step_n(10);
    let (_, before) = meter(&sim).expect("the alembic is running");

    sim.submit("wield quickening-scroll");
    sim.step();
    let (_, after) = meter(&sim).expect("the alembic is still running");

    // **Halved from now, not from the start.** Ten ticks into 56 leaves 46, so
    // the total becomes 10 + 23 = 33; halving the whole interval would refund
    // time already spent and, past the half-way point, put the end in the past.
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
        "debug_spawn quickening-scroll",
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
    // **The one duration in the tower that did not ask.** `Triaging` was inserted
    // with a raw `PURGE_TICKS`, so §19's *"everything the room starts inside that
    // window takes half as long"* was false for a scour and nothing said so —
    // `hastened` had exactly one caller and this was the path around it.
    //
    // The *ratio* rather than the two numbers: `PURGE_TICKS` is a placeholder the
    // balance CLI sweeps, and a test naming 4 and 2 would fail on a tuning pass
    // that changed nothing about this rule.
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
    // **`Cooling`'s rule, one component over.** Nothing ever removes `Quickened`
    // — that is what an interval means, and `quickened` simply reads false once
    // it has run out — so a bare capture wrote a dead span into *every* autosave
    // from the first scroll a player ever spent, for ever after.
    //
    // The control matters as much as the case: without it this passes against a
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

    // Past `QUICKENED_TICKS`, which is 300.
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
fn shelved(sim: &Sim) -> Vec<String> {
    let world = sim.world();
    let shelf = orbs_sim::tower::home(world, "sage").expect("sage has nowhere to live");
    orbs_sim::tower::holdings(world, shelf)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

#[cfg(debug_assertions)]
#[test]
fn a_verdant_scroll_puts_one_herb_on_the_shelf_and_the_fourth_is_refused() {
    // **One each, not all three.** A scroll that unlocked everything would leave
    // the lectern assembling a thing with nothing left to give — a dud draw for
    // ever, which is the dead end this whole item exists to close. Three are all
    // worth having and the laboratory visibly grows three times.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim.submit("debug_spawn verdant-scroll 4");
    sim.step();

    let before = shelved(&sim);
    assert!(
        !before.iter().any(|name| name == "mugwort"),
        "the herbs were on the shelf before anything unlocked them: {before:?}",
    );

    // **Alphabetical, and fixed for every seed.** A roll here would be a second
    // draw stacked on the lectern's; what the player chooses is when to spend.
    for want in ["amber", "mugwort", "valerian"] {
        sim.submit("wield verdant-scroll");
        sim.step();
        assert!(
            shelved(&sim).iter().any(|name| name == want),
            "spending a scroll did not put `{want}` on the shelf: {:?}",
            shelved(&sim),
        );
    }

    // **Only the herbs.** The first version asked `Recipes::outputs`, which is a
    // recipe's `output` and not its `leaves` — so every byproduct in the game
    // read as unlockable and four scrolls shelved `dregs`, `ash` and a
    // `fragment` as inexhaustible stock. No test said a word; one dump did.
    for never in ["dregs", "ash", "husks", "phlegm", "fragment", "potash"] {
        assert!(
            !shelved(&sim).iter().any(|name| name == never),
            "`{never}` is not a herb and was unlocked as one",
        );
    }

    let before = messages(&sim).len();
    sim.submit("wield verdant-scroll");
    sim.step();
    assert!(
        messages(&sim)
            .into_iter()
            .skip(before)
            .any(|line| line.contains("every herb")),
        "the fourth scroll did not say why it did nothing: {:?}",
        messages(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn an_unlocked_herb_is_as_endless_as_one_the_tower_opened_with() {
    // A base reagent that ran out would make a recipe written against it work
    // for a while and then stop — the same word meaning a different thing
    // depending on when it arrived.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim.submit("debug_spawn verdant-scroll");
    sim.step();
    sim.submit("wield verdant-scroll");
    sim.step();

    let world = sim.world();
    let shelf = orbs_sim::tower::home(world, "sage").expect("sage has nowhere to live");
    assert_eq!(
        orbs_sim::tower::held(world, shelf, "amber"),
        Some(orbs_sim::tower::Stock::Endless),
        "an unlocked herb is exhaustible",
    );
}

#[cfg(debug_assertions)]
#[test]
fn the_ported_herbs_reach_a_potion() {
    // **Driven the whole way**, because a route that reads well in `recall` and
    // cannot be walked is a table rather than content — which is exactly what
    // the lectern's `dust` recipe turned out to be.
    let mut sim = Sim::new(1);
    for line in [
        "attend laboratory",
        "debug_spawn verdant-scroll 3",
        "wield verdant-scroll",
        "wield verdant-scroll",
        "wield verdant-scroll",
        "kindle charcoal",
        "grind mugwort",
    ] {
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
            .any(|line| line.contains("yields insight")),
        "the ported route does not reach its potion: {:?}",
        messages(&sim),
    );
}
