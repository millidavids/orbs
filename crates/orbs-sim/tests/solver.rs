//! The acceptance test for the archive's maze: **can a player teach the orb to
//! solve one?**
//!
//! This file exists before the generator, the verb or the picture, because the
//! whole design rests on one claim — that Trémaux's algorithm is expressible in
//! §8's language *without* the language gaining a variable, because the maze
//! holds the search's state. If that is false the archive is the one room
//! automation can never reach, and nothing else about it is worth building.
//!
//! Most of what it pins is the **cast**. A condition's names are resolved when a
//! spell is cast (`spell::compile`), against the room as it is at that instant —
//! and a name it cannot place nulls the whole condition, so the `if` runs
//! neither half. No cell is `walked` at the moment a solver is cast, which is
//! precisely the moment its names have to resolve. `tower::scene_at` registering
//! the readings is what makes that work, and this is what would catch it being
//! removed.
//!
//! **And one of them pins the walk**, which for a while nothing did. Every test
//! here checked that the sentences survive being read, and *survives the cast*
//! was quietly standing in for *reaches the exit* — two different claims, only
//! one of them tested. It was not idle worry: the obvious flat ladder parses
//! perfectly, casts clean, walks two cells and then oscillates for ever. See
//! [`four_way_solver`].

use orbs_render::{FieldName, RecordKind, Value};
use orbs_sim::Sim;

/// The rules of a Trémaux solver, as a player would write them.
///
/// Only north and east, because a corridor is all this needs and a full
/// four-way solver is twelve more lines saying the same thing. What is under
/// test is whether the *sentences* survive the cast.
fn solver() -> Vec<String> {
    [
        "repeat 40",
        "if north has exit",
        "follow north",
        "end",
        "if east has exit",
        "follow east",
        "end",
        "if east has passage and not east has walked",
        "follow east",
        "end",
        "if north has passage and not north has twice",
        "follow north",
        "end",
        "end",
    ]
    .iter()
    .map(|line| (*line).to_owned())
    .collect()
}

/// A full four-way solver, as a player would eventually write one.
///
/// **Five tiers**: the way out; a square nobody has walked; the least-walked way
/// that is *not* the way you came; and, last, the way you came. That last pair is
/// what makes it a solver rather than something that resembles one — see
/// [`orbs_sim::tower::maze::BACK`]. Without it the ladder cycles for ever at a
/// junction where two ways read alike and the compass order sends it back where
/// it came from; it solved 7x7 backtracker mazes and nothing harder.
///
/// **`else`, not sixteen `if`s in a row, and the difference is the whole test.**
/// A flat ladder walks two cells and then oscillates for ever: the passage tier
/// steps into a fresh cell, and four lines later the *same lap* reads the cell
/// just left as `walked` and steps straight back. Guarding the retreat behind
/// *nowhere new to go* moves the pendulum down a tier rather than removing it —
/// the `walked` rung then steps back and the `twice` rung returns.
///
/// The rule a ladder is *read* as carrying — "the first line that matches, and
/// then stop" — is the thing `else` says and a sequence of `if`s does not. One
/// move per lap, by construction.
///
/// Fifty-odd lines, and every one of them costs a tick. That is the price the
/// design says a player can optimise, stated as a number rather than a promise.
fn four_way_solver(laps: u32) -> Vec<String> {
    let ways = ["north", "east", "south", "west"];
    // Five rungs, and the last two are the whole difference between a solver and
    // a thing that looks like one. `back` is *not* a reading in the same axis —
    // a way can be `walked` and the way you came at once — so the middle rungs
    // exclude it and the bottom rung is the retreat.
    let ladder: Vec<(String, &str)> = ["exit", "passage"]
        .into_iter()
        .flat_map(|reading| ways.map(|way| (format!("{way} has {reading}"), way)))
        .chain(["walked", "twice"].into_iter().flat_map(|reading| {
            ways.map(|way| (format!("{way} has {reading} and not {way} has back"), way))
        }))
        .chain(ways.map(|way| (format!("{way} has back"), way)))
        .collect();

    let mut lines = vec![format!("repeat {laps}")];
    for (condition, way) in &ladder {
        lines.push(format!("if {condition}"));
        lines.push(format!("follow {way}"));
        lines.push("else".to_owned());
    }
    // One `end` per `else` opened, then one for the `repeat`.
    lines.extend(std::iter::repeat_n("end".to_owned(), ladder.len() + 1));
    lines
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

#[test]
fn a_solver_survives_the_cast_with_every_condition_intact() {
    // **The claim the design rests on.** `compile::fix` nulls a condition naming
    // a thing it cannot place — a guard §19 added because `has ground-slat`
    // answered "no" for ever — and it cannot tell a word for a not-yet-existing
    // state from a typo. Without the readings in the scene, every line below
    // compiles to a branch that takes neither half, and a bound solver would
    // walk into a wall for ever with nothing on screen saying why.
    let mut sim = Sim::new(1);
    sim.submit("attend archive");
    sim.step();

    let reading = sim.read_spell("archive", &solver());
    let faults: Vec<String> = reading
        .iter()
        .filter(|line| line.fault.is_some())
        .map(|line| line.heard.clone())
        .collect();

    assert!(
        faults.is_empty(),
        "the orb could not read {} of the solver's lines: {faults:?}",
        faults.len(),
    );
}

#[test]
fn the_readings_are_offered_before_any_maze_exists() {
    // **Stable is the point.** `bind::stand` recasts a held spell every lap, so
    // a solver must compile identically each time. A vocabulary that came and
    // went with the maze would make a bound spell work on some laps and not on
    // others — the worst failure available, because it looks like the maze.
    let mut sim = Sim::new(1);
    sim.submit("attend archive");
    sim.step();

    for word in ["passage", "wall", "walked", "twice", "exit"] {
        let line = format!("if north has {word}");
        let reading = sim.read_spell(
            "archive",
            &[line.clone(), "survey".to_owned(), "end".to_owned()],
        );
        assert!(
            reading[0].fault.is_none(),
            "{line:?} did not resolve with no maze open: {:?}",
            reading[0],
        );
    }
}

#[test]
fn a_typo_among_the_readings_is_still_refused() {
    // The guard must keep working. Offering a vocabulary is not the same as
    // giving up on typos, and `walkd` has to stay a fault or the readings have
    // bought their resolution by disabling the thing that made them necessary.
    let mut sim = Sim::new(1);
    sim.submit("attend archive");
    sim.step();

    let reading = sim.read_spell(
        "archive",
        &[
            "if north has walkd".to_owned(),
            "survey".to_owned(),
            "end".to_owned(),
        ],
    );
    assert!(
        reading[0].fault.is_some(),
        "a misspelled reading was accepted: {:?}",
        reading[0],
    );
}

#[test]
fn the_solver_is_writable_and_castable_end_to_end() {
    // Cast rather than merely read: `interpret` and `compile` are the same door,
    // but a spell that reads clean and then refuses to run would be the two
    // halves disagreeing, which §19 records happening twice.
    let mut sim = Sim::new(1);
    sim.submit("attend archive");
    sim.step();
    sim.write_spell("threading", &solver());
    sim.step();
    sim.submit("invoke threading");
    sim.step_n(4);

    let said = messages(&sim);
    assert!(
        !said.iter().any(|line| line.contains("cannot read")),
        "the cast reported an unreadable line: {said:?}",
    );
    assert!(
        said.iter().any(|line| line.contains("threading")),
        "the spell never started: {said:?}",
    );
}

#[test]
fn a_solver_walks_a_generated_maze_to_the_exit() {
    // **The claim the other tests only gesture at.** Every one of them pins the
    // *cast*; none walks a step, so "you can teach the orb to solve a maze" was
    // an assumption with a test-shaped hole under it. This is the one that says
    // the language is sufficient — not that the sentences parse, but that the
    // rules they encode actually reach the exit.
    // **Swept, not sampled.** One seed is one maze, and a policy that happens to
    // suit one layout is exactly the failure a single fixture cannot see.
    let mut worst = 0;
    for seed in 1..=12 {
        let mut sim = Sim::new(seed);
        sim.submit("attend archive");
        sim.step();
        sim.write_spell("threading", &four_way_solver(4_000));
        sim.step();
        sim.submit("research");
        sim.step();
        sim.submit("invoke threading");

        let mut ticks = 0;
        while ticks < CEILING && !messages(&sim).iter().any(|line| line.contains("fragment")) {
            sim.step();
            ticks += 1;
        }
        assert!(
            ticks < CEILING,
            "seed {seed}: a solver ran for {CEILING} ticks and never reached the exit",
        );
        worst = worst.max(ticks);

        // **And it lands on the archive's shelf.** What the stacks give up is
        // stock, so it goes where stock goes and the player moves four of them
        // into the lectern to assemble. Asked of `tower::home` rather than named
        // here, because `debug_spawn` asks the same rule — and a fragment that
        // was in one place when won and another when spawned is the split this
        // closed.
        let world = sim.world();
        let shelf =
            orbs_sim::tower::home(world, "fragment").expect("a fragment has nowhere to live");
        assert!(
            orbs_sim::tower::holdings(world, shelf)
                .iter()
                .any(|(name, units)| name == "fragment" && *units > 0),
            "seed {seed}: the exit paid its fragment somewhere else",
        );
    }

    // **A ceiling on the cost, not just on the outcome.** A solver that arrives
    // eventually is not the same claim as one a player would wait for, and this
    // is what would notice the language getting slower rather than broken.
    assert!(
        worst <= BUDGET,
        "the slowest maze took {worst} ticks, over the {BUDGET} this pins",
    );
}

/// How long a solver is given before it is called stuck.
const CEILING: u64 = 40_000;

/// The slowest a swept maze may be, in ticks.
///
/// **6500 against an observed worst of 5123**, over the twelve seeds above.
/// Loose enough that a differently-shaped maze does not fail it, tight enough to
/// notice the language getting slower — which is the only way this number is
/// worth having, since a test that merely says *eventually* would pass on a
/// solver nobody would wait for.
///
/// It has climbed twice and both times for a reason worth the ticks. 900 while
/// the walls lived *between* cells; 1600 once a wall became a square of its own,
/// because a corridor square is a step too; and 9000 at 16x16 with a denser
/// carve, which is about five times the floor. Roughly two hours of world time
/// per fragment for a bound solver — paid while the player is elsewhere, which
/// is what a bound spell is for, and not paid at all by a player walking it by
/// hand at the speed of their own keyboard.
const BUDGET: u64 = 6_500;

#[test]
fn a_solved_maze_leaves_no_readings_behind() {
    // **The order of two lines.** `refresh` ran before the `Maze` was removed,
    // so the four ways kept the solved maze's last readings for ever: `survey
    // north` answered `passage` with nothing open, and a bound solver read them,
    // fired its `follow` tier every lap and was told *"research first"* for the
    // rest of its `repeat` — a spell stuck in a loop with the world telling it
    // the truth and the scene lying.
    let mut sim = Sim::new(1);
    sim.submit("attend archive");
    sim.step();
    sim.write_spell("threading", &four_way_solver(4_000));
    sim.step();
    sim.submit("research");
    sim.step();
    sim.submit("invoke threading");
    let mut ticks = 0;
    while ticks < CEILING && !messages(&sim).iter().any(|line| line.contains("fragment")) {
        sim.step();
        ticks += 1;
    }
    sim.submit("stop threading");
    sim.step();

    for way in ["north", "east", "south", "west"] {
        let before = messages(&sim).len();
        sim.submit(&format!("survey {way}"));
        sim.step();
        let said: Vec<String> = messages(&sim).split_off(before);
        assert!(
            !said.iter().any(|line| line.contains("passage")
                || line.contains("walked")
                || line.contains("twice")),
            "{way} still reads from stacks that are gone: {said:?}",
        );
    }
}

#[test]
fn following_files_its_record_under_follow() {
    // Rule 4 makes the record the source and every view a reading of it, so a
    // record filed under the wrong verb is that source lying. Both verbs in the
    // module shared one `say`, which stamped `divine` on all of `follow`'s
    // completions — `sift follow orb.log` found the echo of the typed line and
    // not what happened, for the archive's most-used word.
    let mut sim = Sim::new(1);
    sim.submit("attend archive");
    sim.step();
    sim.submit("research");
    sim.step();
    sim.submit("follow east");
    sim.step();

    let named: Vec<String> = sim
        .scrollback()
        .records()
        .iter()
        .filter(|record| record.kind() == RecordKind::Completion)
        .filter_map(|record| record.field(FieldName::Name))
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .collect();
    assert!(
        named.iter().any(|name| name == "follow"),
        "no completion was filed under `follow`: {named:?}",
    );
}

#[test]
fn a_lectern_collecting_fragments_is_not_reported_as_fouled() {
    // `Fouled` means *it will not start*, and the panel exists to stop exactly
    // that confusion. The lectern's only recipe is an exact match on four
    // fragments, so one, two or three of them matched nothing and fell through
    // to it — the panel telling a player mid-collection that their instrument
    // was spoiled.
    let mut sim = Sim::new(1);
    sim.submit("attend archive");
    sim.step();
    let lectern = lectern_of(&sim);
    orbs_sim::tower::give(sim.world_mut(), lectern, "fragment", fragment_kind(), 2);
    sim.step();

    let state = sim
        .instruments()
        .into_iter()
        .find(|instrument| instrument.name == "lectern")
        .map(|instrument| instrument.state);
    assert_eq!(
        state,
        Some(orbs_sim::tower::State::Gathering),
        "two of four fragments read as something other than collecting",
    );
}

/// The archive's lectern.
/// The kind a fragment is, **asked rather than written down here**.
///
/// A test that hard-codes it can drift from `research::give_fragment`, and then
/// it is setting up a state the game cannot reach — which is exactly what
/// happened while the maze gave `NounKind::Fragment` and `debug_spawn` gave
/// `Reagent` for the same word (§19).
fn fragment_kind() -> orbs_sim::parser::NounKind {
    orbs_sim::content::Recipes::builtin().kind_of("fragment")
}

fn lectern_of(sim: &Sim) -> bevy_ecs::entity::Entity {
    let world = sim.world();
    let cwd = world.resource::<orbs_sim::Cwd>().0;
    orbs_sim::children_of(world, cwd)
        .into_iter()
        .find(|node| {
            world
                .get::<orbs_sim::Name>(*node)
                .is_some_and(|name| name.0 == "lectern")
        })
        .expect("the archive has no lectern")
}

#[test]
fn four_fragments_on_the_lectern_become_a_scroll() {
    // **The assembly half needed no new mechanism.** `Recipes::matching` is an
    // multiset over what an instrument holds *with a count per name*, so four
    // fragments in the lectern and a `wield` is a recipe firing — the same door
    // every laboratory stage goes through. This is the test that says so,
    // because a recipe authored and never fired is a recipe nobody knows works.
    let mut sim = Sim::new(1);
    sim.submit("attend archive");
    sim.step();
    // **Placed the way solving places them.** `debug_spawn` puts stock in the
    // dispensary, which is in the *laboratory* — and §7 keeps a domain's
    // belongings unreachable from another room, so a tester cannot carry one
    // across. This is the same `give` the exit calls, four times over.
    let lectern = {
        let world = sim.world();
        let cwd = world.resource::<orbs_sim::Cwd>().0;
        orbs_sim::children_of(world, cwd)
            .into_iter()
            .find(|node| {
                world
                    .get::<orbs_sim::Name>(*node)
                    .is_some_and(|name| name.0 == "lectern")
            })
            .expect("the archive has no lectern")
    };
    orbs_sim::tower::give(sim.world_mut(), lectern, "fragment", fragment_kind(), 4);

    sim.submit("wield lectern");
    sim.step_n(30);

    // **Any scroll, not one name.** What the lectern makes is *drawn* from the
    // scrolls it knows, so pinning a name here would make this test fail the day
    // a second one is authored — and it would be failing on the roll working.
    // What has to hold is that four fragments become *a scroll*.
    let scrolls = orbs_sim::content::Recipes::builtin();
    let scrolls: Vec<&str> = scrolls
        .outputs()
        .into_iter()
        .filter(|name| scrolls.kind_of(name) == orbs_sim::parser::NounKind::Scroll)
        .collect();
    assert!(!scrolls.is_empty(), "nothing in the content is a scroll");
    assert!(
        messages(&sim)
            .iter()
            .any(|line| scrolls.iter().any(|scroll| line.contains(scroll))),
        "four fragments made no scroll: {:?}",
        messages(&sim),
    );
}

#[test]
fn a_way_is_not_a_room_and_the_stacks_can_be_abandoned() {
    // Two refusals that only exist because the maze had to borrow shapes from
    // elsewhere. The four ways are `NounKind::Place` — the only kind the place
    // half of a spell's question resolves against — which made them somewhere
    // you could stand, and §19 names walking into a compass bearing as the sign
    // the maze had become a second spatial system.
    let mut sim = Sim::new(1);
    sim.submit("attend archive");
    sim.step();
    sim.submit("attend north");
    sim.step();
    assert!(
        messages(&sim)
            .iter()
            .any(|line| line.contains("not a room")),
        "the reading was somewhere to stand: {:?}",
        messages(&sim),
    );

    // ...and `research` inserts no `Working`, so without its own branch `stop`
    // would find the stacks, do nothing, and say it had.
    //
    // **`stop stacks`, and it was `stop lectern`.** The maze has its own
    // instrument now, so `stop lectern` reaches an *assembly* and answers *"the
    // lectern is not working"* — which is the separation working rather than a
    // regression, and is what this test would otherwise have gone on asserting
    // the opposite of.
    sim.submit("research");
    sim.step();
    sim.submit("stop stacks");
    sim.step();
    assert!(
        messages(&sim).iter().any(|line| line.contains("close")),
        "the stacks could not be abandoned: {:?}",
        messages(&sim),
    );
    sim.submit("follow north");
    sim.step();
    assert!(
        messages(&sim)
            .iter()
            .any(|line| line.contains("research first")),
        "it was still open after being stopped: {:?}",
        messages(&sim),
    );
}
