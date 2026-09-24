//! `alongside` — two cursors in one spell (DESIGN.md §8).
//!
//! Not that concurrency exists: `invoke` from inside a spell already inserted a
//! second `Running` without blocking the caller. What `alongside` adds is both
//! halves of a pipeline in one file, so these tests are about the cursor model
//! rather than parallelism.

use orbs_render::{FieldName, Value};
// `Save` is the debug-gated tests' — `debug_take` buys the second cursor, and
// that word does not exist in a release build.
#[cfg(debug_assertions)]
use orbs_sim::Save;
use orbs_sim::Sim;

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

/// The menagerie with §8's channel and its second cursor both bought.
///
/// `debug_take` rather than earning both tiers — that is a laboratory's
/// afternoon in a file about the spell runner. The gate itself is
/// [`the_second_cursor_is_bought_at_the_loom`].
fn in_the_menagerie() -> Sim {
    let mut sim = Sim::new(11);
    run(&mut sim, "attend menagerie");
    run(&mut sim, "debug_take satchel_1");
    run(&mut sim, "debug_take cursors_1");
    sim
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

fn ever_said(sim: &Sim, needle: &str) -> bool {
    said(sim).iter().any(|line| line.contains(needle))
}

fn write(sim: &mut Sim, name: &str, lines: &[&str]) {
    let lines: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    sim.write_spell(name, &lines);
    sim.step();
}

/// A second cursor sells ergonomics, not power: two spells already run at once
/// at concentration 0, ungated. What `cursors_1` unlocks is both halves of a
/// pipeline in one file. Both sides are asserted — `alongside` says what to buy
/// until it is bought, and the two-spell road was open throughout.
#[cfg(debug_assertions)]
#[test]
fn the_second_cursor_is_bought_at_the_loom() {
    let mut sim = Sim::new(11);
    run(&mut sim, "attend menagerie");
    run(&mut sim, "debug_take satchel_1");

    write(
        &mut sim,
        "early",
        &[
            "part filling()",
            "    queue heed",
            "end",
            "alongside filling()",
        ],
    );
    run(&mut sim, "invoke early");
    sim.step_n(6);
    assert!(
        ever_said(&sim, "cannot be in two places yet"),
        "an unbought fork said nothing: {:?}",
        said(&sim),
    );
    assert!(
        !ever_said(&sim, "heed goes in the satchel"),
        "an unbought fork ran anyway: {:?}",
        said(&sim),
    );

    // And the two-spell road is open without it, which is why the loom labels
    // this node as ergonomics.
    write(&mut sim, "filler", &["queue yoke"]);
    write(&mut sim, "caller", &["invoke filler", "bide 3"]);
    run(&mut sim, "invoke caller");
    sim.step_n(10);
    assert!(
        ever_said(&sim, "yoke goes in the satchel"),
        "two spells could not run together: {:?}",
        said(&sim),
    );
}

/// The test the whole refactor is for. The consumer is cursor 0 and blocks on
/// an empty satchel; the producer is cursor 1. Before the split,
/// `Progress::Blocked` ended the whole *entity's* tick, so the producer was
/// never reached and the spell sat until `PATIENCE` — deadlock by construction.
#[cfg(debug_assertions)]
#[test]
fn a_blocked_cursor_yields_only_itself() {
    let mut sim = in_the_menagerie();
    write(
        &mut sim,
        "both",
        &[
            "part filling()",
            "    queue heed",
            "    queue yoke",
            "end",
            // The consumer runs first and finds nothing, every tick, until the
            // forked producer has put something in.
            "alongside filling()",
            "repeat 2",
            "    pull note from satchel",
            "    survey note",
            "end",
        ],
    );
    run(&mut sim, "invoke both");
    sim.step_n(30);

    assert!(
        ever_said(&sim, "heed"),
        "the consumer never got the first name: {:?}",
        said(&sim),
    );
    assert!(
        ever_said(&sim, "yoke"),
        "the consumer never got the second: {:?}",
        said(&sim),
    );
    assert!(
        !ever_said(&sim, "gave up"),
        "the pipeline deadlocked into a fault: {:?}",
        said(&sim),
    );
}

/// The spell ends when every cursor has, not when the first does. The caller
/// runs off the end at once while the forked part works; before [`ended`] the
/// off-the-end path called `finish` straight away and took the fork with it.
#[cfg(debug_assertions)]
#[test]
fn a_spell_lasts_as_long_as_its_longest_cursor() {
    let mut sim = in_the_menagerie();
    write(
        &mut sim,
        "outlast",
        &[
            "part filling()",
            "    bide 6",
            "    queue heed",
            "end",
            "alongside filling()",
        ],
    );
    run(&mut sim, "invoke outlast");
    sim.step_n(20);

    assert!(
        ever_said(&sim, "heed goes in the satchel"),
        "the caller running out killed the fork: {:?}",
        said(&sim),
    );
}

/// A forked cursor keeps its own bindings, and the caller's are untouched.
///
/// A part's brackets are the whole of what it can see (§19), now applied to a
/// cursor that goes on running while the caller does.
#[cfg(debug_assertions)]
#[test]
fn a_forked_cursor_binds_its_own_names() {
    let mut sim = in_the_menagerie();
    write(
        &mut sim,
        "scoped",
        &[
            "part filling(what)",
            "    let note be yoke",
            "    queue what",
            "end",
            "let note be heed",
            "alongside filling(spurn)",
            "bide 4",
            "queue note",
        ],
    );
    run(&mut sim, "invoke scoped");
    sim.step_n(20);

    // The part queued what it was handed; the caller queued its own `note`,
    // which the part's `let` must not have moved.
    assert!(
        ever_said(&sim, "spurn goes in the satchel"),
        "the fork did not get its argument: {:?}",
        said(&sim),
    );
    assert!(
        ever_said(&sim, "heed goes in the satchel"),
        "the fork overwrote the caller's binding: {:?}",
        said(&sim),
    );
}

/// `MAX_STRANDS` is a runaway guard, not a balance number: `alongside` inside a
/// `repeat` forks one a lap, and a strand is heavier than a descent because each
/// also spends its own budget.
#[cfg(debug_assertions)]
#[test]
fn a_runaway_fork_is_bounded_and_says_so() {
    let mut sim = in_the_menagerie();
    write(
        &mut sim,
        "runaway",
        &[
            "part filling()",
            "    bide 60",
            "end",
            "repeat 20",
            "    alongside filling()",
            "end",
        ],
    );
    run(&mut sim, "invoke runaway");
    sim.step_n(40);

    assert!(
        ever_said(&sim, "too many already running"),
        "an unbounded fork was allowed: {:?}",
        said(&sim),
    );
}

/// A fork of a part that does not exist is the same complaint a call gets.
#[test]
fn forking_a_part_that_is_not_there_is_refused_by_name() {
    let mut sim = in_the_menagerie();
    write(&mut sim, "missing", &["alongside nowhere()"]);
    run(&mut sim, "invoke missing");
    sim.step_n(4);

    assert!(ever_said(&sim, "no part of this spell"), "{:?}", said(&sim),);
}

/// `alongside` without brackets is its own complaint, not a command.
#[test]
fn a_fork_without_brackets_says_what_is_missing() {
    let mut sim = in_the_menagerie();
    write(
        &mut sim,
        "bare",
        &[
            "part filling()",
            "    queue heed",
            "end",
            "alongside filling",
        ],
    );
    run(&mut sim, "invoke bare");
    sim.step_n(6);

    assert!(
        ever_said(&sim, "alongside wants a part"),
        "{:?}",
        said(&sim),
    );
}

/// Two cursors interleave the same way on every run (rule 3): `step_one` steps
/// strands in `Vec` order, each spending its whole budget before the next.
///
/// Budget 2, because batch and round-robin are identical at one step a tick and
/// a test at the shipped budget would pin neither (§19).
#[cfg(debug_assertions)]
#[test]
fn two_cursors_interleave_the_same_way_at_two_steps_a_tick() {
    fn interleaving(seed: u64) -> Vec<String> {
        let mut sim = Sim::new(seed);
        for line in ["attend laboratory", "kindle charcoal"] {
            run(&mut sim, line);
        }
        run(&mut sim, "debug_spawn clarified-draught 4");
        for _ in 0..3 {
            run(&mut sim, "distil clarified-draught");
            sim.step_n(60);
            run(&mut sim, "empty alembic");
        }
        sim.take("steps_1");
        sim.step();
        assert_eq!(orbs_sim::tower::spell::budget(sim.world()), 2);

        run(&mut sim, "attend menagerie");
        // `steps_1` was earned the real way above, because the second step is
        // what is under test; the channel is not, so it is shortcut.
        run(&mut sim, "debug_take satchel_1");
        run(&mut sim, "debug_take cursors_1");
        write(
            &mut sim,
            "weave_two",
            &[
                "part filling()",
                "    queue heed",
                "    queue yoke",
                "    queue spurn",
                "    queue mirror",
                "end",
                "alongside filling()",
                "repeat 4",
                "    pull note from satchel",
                "    survey note",
                "end",
            ],
        );
        run(&mut sim, "invoke weave_two");
        sim.step_n(30);
        said(&sim)
    }

    let once = interleaving(11);
    let twice = interleaving(11);
    assert_eq!(once, twice, "two runs of one seed interleaved differently");
    assert!(
        once.iter().any(|line| line.contains("mirror")),
        "the pipeline never ran through: {once:?}",
    );
}

/// The rail counts what else is running and `status` names it. The rail once
/// kept only the first spell per domain, so a second `invoke` — or a fork — was
/// invisible; and a `►both +2` a player cannot expand is worse than no marker.
/// Both come from one walk of `Running` (`tower::running_spells`), so the pair
/// is asserted in one world and cannot drift apart.
#[cfg(debug_assertions)]
#[test]
fn the_rail_counts_every_cursor_and_status_names_them() {
    let mut sim = in_the_menagerie();
    write(
        &mut sim,
        "both",
        &[
            "part filling()",
            "    repeat 30",
            "        queue heed",
            "        bide 2",
            "    end",
            "end",
            "alongside filling()",
            "repeat 30",
            "    pull note from satchel",
            "    bide 2",
            "end",
        ],
    );
    write(&mut sim, "second", &["repeat 30", "    bide 2", "end"]);
    run(&mut sim, "invoke both");
    run(&mut sim, "invoke second");
    sim.step_n(3);

    // Three cursors here: the forked spell's two and the second spell's one.
    let menagerie = sim
        .briefs()
        .into_iter()
        .find(|brief| brief.name == "menagerie")
        .expect("the menagerie is built");
    assert_eq!(
        menagerie.running, 3,
        "the rail counted {} of three cursors",
        menagerie.running,
    );
    assert!(menagerie.spell.is_some(), "the rail named no spell at all");

    // ...and `status` says which, with the fork's cursor count on it.
    run(&mut sim, "status");
    let shown = sim
        .scrollback()
        .records()
        .iter()
        .filter_map(|record| match record.field(FieldName::Detail) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        shown.iter().any(|line| line.contains("on 2 cursors")),
        "status did not say the fork had two: {shown:?}",
    );
    assert_eq!(
        shown
            .iter()
            .filter(|line| line.contains("menagerie"))
            .count(),
        2,
        "status listed the wrong number of spells: {shown:?}",
    );
}

/// The shipped forked solver, cast for real, and it must stay optimal.
/// `coursing` is `holding` split down the middle: one cursor queues the next
/// pair of stations, the other hauls between them. A wrong cycle still
/// finishes, at the wrong station, so the assertion is the haul count —
/// `2^n - 1`, the optimum `tower::pylon` proves.
#[cfg(debug_assertions)]
#[test]
fn the_shipped_forked_solver_is_still_optimal() {
    let mut sim = Sim::new(11);
    run(&mut sim, "attend sanctum");
    run(&mut sim, "debug_take satchel_1");
    run(&mut sim, "debug_take cursors_1");
    run(&mut sim, "invoke coursing");
    sim.step_n(600);

    let closing = said(&sim)
        .into_iter()
        .find(|line| line.contains("the last ward seats"))
        .expect("the forked solver never finished the course");
    // `2^n - 1` for a course of n, and `muster` draws three to seven.
    let hauled: u32 = closing
        .split_whitespace()
        .find_map(|word| word.parse().ok())
        .expect("the closing line named no count");
    assert!(
        [7, 15, 31, 63, 127].contains(&hauled),
        "{hauled} hauls is not 2^n - 1, so the cycle is wrong: {closing:?}",
    );
}

/// Both cursors survive a save: state the world holds and the document does not
/// resets silently when a player comes back.
#[cfg(debug_assertions)]
#[test]
fn a_forked_spell_comes_back_with_both_cursors() {
    let mut sim = in_the_menagerie();
    write(
        &mut sim,
        "both",
        &[
            "part filling()",
            "    bide 20",
            "    queue heed",
            "end",
            "alongside filling()",
            "bide 40",
            "queue yoke",
        ],
    );
    run(&mut sim, "invoke both");
    // Far enough in that both cursors are mid-`bide` and neither has emitted.
    sim.step_n(6);

    let text = sim.snapshot().to_toml().expect("a save renders");
    let mut restored = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));
    restored.step_n(80);

    assert!(
        ever_said(&restored, "heed goes in the satchel"),
        "the forked cursor did not come back: {:?}",
        said(&restored),
    );
    assert!(
        ever_said(&restored, "yoke goes in the satchel"),
        "the caller's cursor did not come back: {:?}",
        said(&restored),
    );
}
