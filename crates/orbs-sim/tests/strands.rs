//! `alongside` — two cursors in one spell (DESIGN.md §8).
//!
//! # What is worth proving here
//!
//! **Not that concurrency exists.** Two spells already ran at once before any of
//! this: `invoke` from inside a spell inserts a second `Running` and the caller
//! does not block. What `alongside` adds is that the two halves of a pipeline
//! sit in one file — so these tests are about the *cursor* model rather than
//! about parallelism, and the sharpest of them is the one that would have
//! deadlocked.

use orbs_render::{FieldName, Value};
use orbs_sim::{Save, Sim};

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

/// The menagerie with §8's channel and its second cursor both bought.
///
/// **`debug_take`, not two tiers of earning.** `satchel_1` opens at 24 and
/// `cursors_1` at 40, which is a laboratory's afternoon in a file about the
/// spell runner. `satchel_tests`'s helper says the rest; the gate itself is
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

/// **A second cursor is bought, and it sells ergonomics rather than power.**
///
/// Worth stating in a test because it would be easy to believe otherwise: two
/// spells already run at once at concentration 0, ungated, because `invoke` from
/// inside a spell inserts a second `Running` and the caller does not block. What
/// `cursors_1` unlocks is both halves of a pipeline **in one file**.
///
/// So this asserts both sides: `alongside` says what to buy until it is bought,
/// and the two-spell road was open the whole time.
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
            "    queue skyward",
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
        !ever_said(&sim, "skyward goes in the satchel"),
        "an unbought fork ran anyway: {:?}",
        said(&sim),
    );

    // **And the two-spell road is open without it**, which is why this node is
    // honestly labelled as ergonomics on the loom.
    write(&mut sim, "filler", &["queue earthward"]);
    write(&mut sim, "caller", &["invoke filler", "bide 3"]);
    run(&mut sim, "invoke caller");
    sim.step_n(10);
    assert!(
        ever_said(&sim, "earthward goes in the satchel"),
        "two spells could not run together: {:?}",
        said(&sim),
    );
}

/// **The test the whole refactor is for.**
///
/// The consumer is cursor 0 and blocks on an empty satchel; the producer is
/// cursor 1 and is the only thing that can unblock it. Before the split,
/// `Progress::Blocked` returned from `step_one` and ended the whole *entity's*
/// tick — so the producer would never have been reached, on any tick, and the
/// spell would have sat there until `PATIENCE` with the queue permanently empty.
/// The deadlock was by construction, and it is the one failure mode this feature
/// exists to avoid.
#[test]
fn a_blocked_cursor_yields_only_itself() {
    let mut sim = in_the_menagerie();
    write(
        &mut sim,
        "both",
        &[
            "part filling()",
            "    queue skyward",
            "    queue earthward",
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
        ever_said(&sim, "skyward"),
        "the consumer never got the first name: {:?}",
        said(&sim),
    );
    assert!(
        ever_said(&sim, "earthward"),
        "the consumer never got the second: {:?}",
        said(&sim),
    );
    assert!(
        !ever_said(&sim, "gave up"),
        "the pipeline deadlocked into a fault: {:?}",
        said(&sim),
    );
}

/// **The spell ends when every cursor has, not when the first one does.**
///
/// The caller runs off the end almost immediately; the forked part is still
/// working. Before [`ended`] existed the off-the-end path called `finish`
/// straight away, which would have taken the forked cursor down with it.
#[test]
fn a_spell_lasts_as_long_as_its_longest_cursor() {
    let mut sim = in_the_menagerie();
    write(
        &mut sim,
        "outlast",
        &[
            "part filling()",
            "    bide 6",
            "    queue skyward",
            "end",
            "alongside filling()",
        ],
    );
    run(&mut sim, "invoke outlast");
    sim.step_n(20);

    assert!(
        ever_said(&sim, "skyward goes in the satchel"),
        "the caller running out killed the fork: {:?}",
        said(&sim),
    );
}

/// A forked cursor keeps its own bindings, and the caller's are untouched.
///
/// §19's rule for a part — *"a part's brackets are the whole of what it can
/// see"* — applied to a cursor that goes on running while the caller does. A
/// shared store here would be worse than it was for a call, because both sides
/// keep writing to it.
#[test]
fn a_forked_cursor_binds_its_own_names() {
    let mut sim = in_the_menagerie();
    write(
        &mut sim,
        "scoped",
        &[
            "part filling(what)",
            "    let note be earthward",
            "    queue what",
            "end",
            "let note be skyward",
            "alongside filling(leftward)",
            "bide 4",
            "queue note",
        ],
    );
    run(&mut sim, "invoke scoped");
    sim.step_n(20);

    // The part queued what it was handed; the caller queued its own `note`,
    // which the part's `let` must not have moved.
    assert!(
        ever_said(&sim, "leftward goes in the satchel"),
        "the fork did not get its argument: {:?}",
        said(&sim),
    );
    assert!(
        ever_said(&sim, "skyward goes in the satchel"),
        "the fork overwrote the caller's binding: {:?}",
        said(&sim),
    );
}

/// **`MAX_STRANDS`, and it is a runaway guard rather than a balance number.**
///
/// `alongside` inside a `repeat` forks one a lap. `MAX_PARTS` makes exactly this
/// argument for a descent — *"at one step a tick a runaway does not hang the
/// game, it grows the save"* — and a strand is heavier, because each also spends
/// its own budget.
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
            "    queue skyward",
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

/// **Two cursors interleave the same way on every run, at two steps a tick.**
///
/// Rule 3. `step_one` steps strands in `Vec` order, each spending its whole
/// budget before the next — which is `advance`'s law one level down, where every
/// `Running` spends its whole budget in `NodeId` order.
///
/// **Budget 2, deliberately.** Batch and round-robin are *identical* at one step
/// a tick, so a test written at the shipped budget would pass against either and
/// pin neither — §19 records that shape going wrong. This earns its second step
/// the way a player does.
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
        // **Earned `steps_1` the real way above, and shortcut the other two.**
        // The second step is the thing under test — batch and round-robin are
        // identical without it — so it comes from a tier the tower actually
        // opened. The channel is not under test here and would cost a second
        // afternoon.
        run(&mut sim, "debug_take satchel_1");
        run(&mut sim, "debug_take cursors_1");
        write(
            &mut sim,
            "weave_two",
            &[
                "part filling()",
                "    queue skyward",
                "    queue earthward",
                "    queue leftward",
                "    queue rightward",
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
        once.iter().any(|line| line.contains("rightward")),
        "the pipeline never ran through: {once:?}",
    );
}

/// **The rail counts what else is running, and `status` names it.**
///
/// The rail kept only the *first* spell per domain, so a second `invoke` in one
/// room was invisible — `►tending` whether one ran there or three — and after
/// `alongside` a fork was invisible for the same reason one level down. Both are
/// *"more automation than that line can name"*, so both are counted.
///
/// **And the count pointed at nothing until `status` grew a section.** A player
/// reading `►both +2` had no way to find out what the two were, which makes the
/// marker worse than none: it says something is there and refuses to say what.
/// So this asserts the pair, in one world — the glance and the answer are one
/// walk of `Running` (`tower::running_spells`) precisely so they cannot come to
/// disagree.
#[test]
fn the_rail_counts_every_cursor_and_status_names_them() {
    let mut sim = in_the_menagerie();
    write(
        &mut sim,
        "both",
        &[
            "part filling()",
            "    repeat 30",
            "        queue skyward",
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

/// **The shipped forked solver, cast for real, and it must be optimal.**
///
/// `coursing` is `holding` split down the middle: a cursor that works out which
/// pair of stations comes next and queues it, and a cursor that takes two names
/// out and hauls between them. Splitting a solver is only worth showing if the
/// split does not cost anything, so the assertion is the *number of hauls* —
/// `2^n - 1`, the same optimum `tower::pylon` proves and `holding` reaches.
///
/// **A wrong cycle still finishes**, at the wrong station, which is why the
/// count is the test and "it completed" is not. `warding.rs` makes the same
/// point about `holding`.
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

/// **Both cursors survive a save**, which is `ChantSave`'s lesson one struct
/// over: state the world holds and the document does not is state that silently
/// resets when a player comes back.
#[test]
fn a_forked_spell_comes_back_with_both_cursors() {
    let mut sim = in_the_menagerie();
    write(
        &mut sim,
        "both",
        &[
            "part filling()",
            "    bide 20",
            "    queue skyward",
            "end",
            "alongside filling()",
            "bide 40",
            "queue earthward",
        ],
    );
    run(&mut sim, "invoke both");
    // Far enough in that both cursors are mid-`bide` and neither has emitted.
    sim.step_n(6);

    let text = sim.snapshot().to_toml().expect("a save renders");
    let mut restored = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));
    restored.step_n(80);

    assert!(
        ever_said(&restored, "skyward goes in the satchel"),
        "the forked cursor did not come back: {:?}",
        said(&restored),
    );
    assert!(
        ever_said(&restored, "earthward goes in the satchel"),
        "the caller's cursor did not come back: {:?}",
        said(&restored),
    );
}
