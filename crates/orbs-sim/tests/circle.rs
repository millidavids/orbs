//! The circle as a whole domain: what a player can see is enough, what the world
//! says always agrees with what the beast is, and one seed is one world.
//!
//! `tower::circle`'s proofs walk every circuit and prove the model; `taming.rs`
//! drives the verbs and the spells, `lesser.rs` the sealed tower's first beasts.
//! This file holds the claims that span all of them — the ones a regression in any
//! one layer would break without that layer's own tests noticing.

use std::collections::BTreeSet;

use orbs_render::{Circle, FieldName, Value};
use orbs_sim::Sim;
use orbs_sim::tower::circle::{Glyph, Humour};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

/// Every sentence the tower has said so far.
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

fn at_the_circle(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    run(&mut sim, "attend menagerie");
    sim
}

/// Whether the player is standing in the menagerie.
fn here_is_menagerie(sim: &Sim) -> bool {
    sim.location() == "/tower/menagerie"
}

/// What each glyph is given and whether turned, by glyph.
type Wires = Vec<(String, Vec<(String, bool)>)>;

/// A board as something comparable: what each glyph is given and turned, and
/// the temper. Two boards equal here are the same question.
fn question(board: &Circle) -> (Wires, Vec<bool>) {
    let lines = board
        .lines
        .iter()
        .map(|line| {
            let given = line
                .given
                .iter()
                .map(|given| (given.name.clone(), given.turned))
                .collect();
            (line.glyph.clone(), given)
        })
        .collect();
    (lines, board.temper.clone())
}

/// A limning that answers the board, found by reading **only the board**: the
/// names each glyph is given and whether turned, which senses each column lights,
/// and the temper row. Nothing from the model but the humours' own rules.
fn solve_from_board(board: &Circle) -> Option<Vec<(String, Humour)>> {
    let sense = |name: &str| board.senses.iter().position(|one| one == name);
    let input = |given: &orbs_render::CircleGiven, row: usize| {
        let lit = sense(&given.name)
            .and_then(|index| board.lit.get(index))
            .and_then(|column| column.get(row).copied())
            .unwrap_or(false);
        lit != given.turned
    };
    let rows = board.temper.len();
    let answers = |humours: &[Humour]| {
        (0..rows).all(|row| {
            let lit = match (board.lines.as_slice(), humours) {
                ([keystone_line], [keystone]) => keystone.answer(
                    input(&keystone_line.given[0], row),
                    input(&keystone_line.given[1], row),
                ),
                ([sunwise, widdershins, _], [keystone, outer, inner]) => keystone.answer(
                    outer.answer(input(&sunwise.given[0], row), input(&sunwise.given[1], row)),
                    inner.answer(
                        input(&widdershins.given[0], row),
                        input(&widdershins.given[1], row),
                    ),
                ),
                _ => return false,
            };
            Some(&lit) == board.temper.get(row)
        })
    };
    match board.lines.as_slice() {
        [_] => Humour::ALL
            .into_iter()
            .find(|keystone| answers(&[*keystone]))
            .map(|keystone| vec![("keystone".to_owned(), keystone)]),
        [sunwise, widdershins, _] => Humour::ALL.into_iter().find_map(|keystone| {
            Humour::ALL.into_iter().find_map(|outer| {
                Humour::ALL.into_iter().find_map(|inner| {
                    answers(&[keystone, outer, inner]).then(|| {
                        vec![
                            ("keystone".to_owned(), keystone),
                            (sunwise.glyph.clone(), outer),
                            (widdershins.glyph.clone(), inner),
                        ]
                    })
                })
            })
        }),
        _ => None,
    }
}

/// **Rule 2's claim, made executable: the picture is enough.** For two hundred
/// seeds a solver reads the board and nothing else — not the model, not a
/// reading, not the row numbering, which it takes from the painted sense columns
/// — types the limning it finds, and calls the beast in once. Every beast is
/// held on that call, turned wires included; a board missing a fact would leave
/// some beast balking.
#[test]
fn the_board_alone_is_enough_to_hold_every_beast_in_one_call() {
    let mut turned = 0;
    for seed in 0..200 {
        let mut sim = at_the_circle(seed);
        run(&mut sim, "summon");
        let Some(board) = sim.circle() else {
            panic!("seed {seed} drew no beast");
        };
        turned += board
            .lines
            .iter()
            .flat_map(|line| &line.given)
            .filter(|given| given.turned)
            .count();
        let Some(limning) = solve_from_board(&board) else {
            panic!("seed {seed}: nothing on the board answers it: {board:?}");
        };
        for (glyph, humour) in limning {
            run(&mut sim, &format!("limn {glyph} {}", humour.word()));
        }
        run(&mut sim, "summon");
        assert!(
            sim.circle().is_none() && sim.tally("event:figure") == 1,
            "seed {seed}: the board's own answer balked: {:?}",
            said(&sim).last(),
        );
        assert!(
            said(&sim).iter().any(|line| line.contains("on call 1")),
            "seed {seed} was not held on the first call",
        );
    }
    assert!(
        turned > 150,
        "only {turned} turned wires in two hundred beasts"
    );
}

/// **A player meets different beasts in one game**: thirty held in a row from
/// one tower are at least twenty-five different questions.
#[cfg(debug_assertions)]
#[test]
fn one_tower_draws_many_different_beasts() {
    let mut sim = at_the_circle(181);
    let mut met = BTreeSet::new();
    for held in 0..30 {
        run(&mut sim, "summon");
        let Some(board) = sim.circle() else {
            panic!("beast {held} was not drawn");
        };
        met.insert(question(&board));
        run(&mut sim, "debug_circle");
        run(&mut sim, "summon");
        assert!(sim.circle().is_none(), "beast {held} was not held");
    }
    assert!(
        met.len() >= 25,
        "thirty beasts were only {} questions",
        met.len()
    );
}

/// **And different games meet different beasts**: the first beast of a hundred
/// towers is at least ninety different questions, and most carry a turned wire.
#[test]
fn a_hundred_towers_open_on_many_different_beasts() {
    let mut met = BTreeSet::new();
    let mut turned = 0;
    for seed in 0..100 {
        let mut sim = at_the_circle(seed);
        run(&mut sim, "summon");
        let Some(board) = sim.circle() else {
            panic!("seed {seed} drew no beast");
        };
        turned += usize::from(
            board
                .lines
                .iter()
                .flat_map(|line| &line.given)
                .any(|given| given.turned),
        );
        met.insert(question(&board));
    }
    assert!(
        met.len() >= 90,
        "a hundred towers opened on {} beasts",
        met.len()
    );
    assert!(
        turned >= 70,
        "only {turned} of a hundred first beasts had a turned wire"
    );
}

/// The commands the fuzz and the replay choose from — including words that are
/// wrong, in either order, and leaving the room.
fn a_command(rng: &mut ChaCha8Rng) -> String {
    const GLYPHS: [&str; 3] = ["keystone", "sunwise", "widdershins"];
    const HUMOURS: [&str; 6] = ["yoke", "spurn", "heed", "eschew", "oppose", "mirror"];
    // `debug_circle` is how a random walk holds beasts often enough to test holds,
    // and it exists only in a debug build.
    let shortcut = if cfg!(debug_assertions) {
        "debug_circle"
    } else {
        "summon"
    };
    let glyph = GLYPHS[rng.random_range(0..GLYPHS.len())];
    let humour = HUMOURS[rng.random_range(0..HUMOURS.len())];
    match rng.random_range(0..20) {
        0..=4 => "summon".to_owned(),
        5..=8 => format!("limn {glyph} {humour}"),
        9..=10 => format!("limn {glyph}"),
        11 => format!("limn {humour} {glyph}"),
        12 => format!("limn {glyph} {glyph}"),
        13 => "limn".to_owned(),
        14 => "stop circle".to_owned(),
        15 => "survey circle".to_owned(),
        16 => shortcut.to_owned(),
        17 => "attend laboratory".to_owned(),
        _ => "attend menagerie".to_owned(),
    }
}

/// **One seed and one list of commands is one world**, however the commands
/// wander — and a tower saved half way and loaded carries on as the one that
/// never stopped.
#[test]
fn one_seed_and_one_list_of_commands_is_one_world_through_a_save() {
    for seed in [3, 181, 4242] {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let commands: Vec<String> = (0..80).map(|_| a_command(&mut rng)).collect();

        let mut played = Sim::new(seed);
        let mut twin = Sim::new(seed);
        for command in &commands[..40] {
            run(&mut played, command);
            run(&mut twin, command);
            assert_eq!(
                played.circle(),
                twin.circle(),
                "seed {seed} after {command:?}"
            );
        }
        let mut loaded = Sim::restored(&played.snapshot());
        for command in &commands[40..] {
            run(&mut played, command);
            run(&mut twin, command);
            run(&mut loaded, command);
            assert_eq!(
                played.circle(),
                twin.circle(),
                "seed {seed} after {command:?}"
            );
            assert_eq!(
                played.circle(),
                loaded.circle(),
                "seed {seed}: the loaded tower parted from the played one at {command:?}",
            );
        }
        let text = |sim: &Sim| sim.snapshot().to_toml().unwrap_or_default();
        assert_eq!(
            text(&played),
            text(&twin),
            "seed {seed}: two runs, two worlds"
        );
        assert_eq!(
            played.tally("event:figure"),
            loaded.tally("event:figure"),
            "seed {seed}: the loaded tower held a different number of beasts",
        );
    }
}

/// What the world says about a glyph's humour, read from the root so it answers
/// wherever the player stands.
fn humours_published(sim: &Sim, glyph: &str) -> Vec<Humour> {
    Humour::ALL
        .into_iter()
        .filter(|humour| sim.holds_reading("menagerie", glyph, humour.word()))
        .collect()
}

/// **What the world says always agrees with what the beast is**, whatever a
/// player types. Sixty towers of three hundred commands each, chosen at random —
/// right and wrong words, either order, leaving the room and coming back,
/// letting beasts go, holding them — and after every one:
///
/// - **`fervour` is published exactly while a beast waits**;
/// - **each glyph carries exactly its humour** while it is part of the circle,
///   and nothing when no beast waits or the glyph is dark;
/// - **the circle never takes the production slot** — no `Working` anywhere,
///   since nothing else in these towers works;
/// - **the rail says the menagerie is working exactly while a beast waits**;
/// - **a call's answer row is what the glyphs as they stand answer**, and a call
///   counts once — calls never go down while the same beast waits;
/// - **a hold is counted exactly once**, and only when a beast went away on a
///   `summon` that was not the one that drew it.
#[test]
fn what_the_world_says_always_agrees_with_the_beast() {
    // **Counted, so the fuzz cannot pass by never reaching what it guards.**
    let (mut holds, mut answers, mut calls, mut away) = (0, 0, 0, 0);
    // Fifty open towers, and in a debug build ten sealed ones: those draw lesser
    // beasts until five holds open the whole circle, so both shapes are fuzzed
    // and so is the moment one gives way to the other.
    let sealed = if cfg!(debug_assertions) { 10 } else { 0 };
    for seed in 0..50 + sealed {
        let mut rng = ChaCha8Rng::seed_from_u64(1000 + seed);
        let mut sim = if seed < 50 {
            at_the_circle(seed)
        } else {
            let mut sim = Sim::sealed(seed);
            run(&mut sim, "debug_reach lens_1");
            run(&mut sim, "attend menagerie");
            sim
        };
        let mut previous_calls = None;
        for step in 0..300 {
            let command = a_command(&mut rng);
            let waiting_before = here_is_menagerie(&sim) && sim.beast().is_some();
            let held_before = sim.tally("event:figure");
            run(&mut sim, &command);
            let context = || format!("seed {seed}, step {step}, {command:?}");

            // The beast itself is only visible from the room it waits in; the
            // readings are visible from anywhere.
            let in_room = here_is_menagerie(&sim);
            let beast = if in_room { sim.beast().cloned() } else { None };
            away += usize::from(!in_room);

            assert!(
                sim.working().is_none(),
                "{}: a production slot was taken",
                context()
            );

            if in_room {
                let fervour = sim.holds_reading("menagerie", "circle", "fervour");
                assert_eq!(fervour, beast.is_some(), "{}: fervour disagrees", context());
                for glyph in Glyph::ALL {
                    let published = humours_published(&sim, glyph.word());
                    match &beast {
                        Some(beast) if beast.shape().lights(glyph) => assert_eq!(
                            published,
                            vec![beast.humour(glyph)],
                            "{}: {} carries the wrong humour",
                            context(),
                            glyph.word(),
                        ),
                        _ => assert!(
                            published.is_empty(),
                            "{}: {} carries {published:?} with nothing to limn",
                            context(),
                            glyph.word(),
                        ),
                    }
                }
                let menagerie = sim
                    .briefs()
                    .into_iter()
                    .find(|brief| brief.name == "menagerie")
                    .map(|brief| brief.state);
                assert_eq!(
                    menagerie == Some(orbs_sim::tower::State::Working),
                    beast.is_some(),
                    "{}: the rail disagrees with the circle",
                    context(),
                );
                if let (Some(beast), Some(board)) = (&beast, sim.circle()) {
                    if let Some(answer) = &board.answer {
                        let expected = beast.shape().answer(beast.glyphs());
                        let drawn: Vec<bool> =
                            (0..beast.rows()).map(|row| expected.lit(row)).collect();
                        if command == "summon" {
                            assert_eq!(answer, &drawn, "{}: the answer row is wrong", context());
                            answers += 1;
                        }
                    }
                    if let Some(before) = previous_calls
                        && waiting_before
                    {
                        assert!(beast.calls() >= before, "{}: calls went down", context());
                        if command == "summon" {
                            assert_eq!(
                                beast.calls(),
                                before + 1,
                                "{}: a call miscounted",
                                context()
                            );
                            calls += 1;
                        }
                    }
                }
            }

            let held_now = sim.tally("event:figure");
            assert!(
                held_now <= held_before + 1,
                "{}: two holds in one command",
                context()
            );
            if held_now == held_before + 1 {
                assert!(
                    waiting_before && beast.is_none(),
                    "{}: a hold was counted with no beast going",
                    context(),
                );
                assert!(
                    command == "summon",
                    "{}: something other than a call held a beast",
                    context(),
                );
                holds += 1;
            }
            previous_calls = beast.as_ref().map(orbs_sim::tower::circle::Beast::calls);
        }
    }
    println!("{holds} holds, {answers} answer rows, {calls} calls, {away} steps away");
    assert!(holds >= 50, "only {holds} holds in the whole fuzz");
    assert!(answers >= 500, "only {answers} answer rows checked");
    assert!(calls >= 300, "only {calls} calls counted");
    assert!(away >= 500, "only {away} commands left the room");
}

/// **Summoning takes nothing from any other room's draws.** Ten beasts drawn and
/// held before the archive is walked leave the archive's stacks exactly the maze
/// the same seed draws without them — the per-subsystem stream, held end to end.
#[cfg(debug_assertions)]
#[test]
fn ten_beasts_move_no_other_rooms_draws() {
    let stacks = |summons: usize| {
        let mut sim = at_the_circle(11);
        for _ in 0..summons {
            run(&mut sim, "summon");
            run(&mut sim, "debug_circle");
            run(&mut sim, "summon");
        }
        run(&mut sim, "attend archive");
        run(&mut sim, "research");
        sim.stacks()
    };
    let untouched = stacks(0);
    assert!(untouched.is_some(), "the archive drew no stacks");
    assert_eq!(stacks(10), untouched, "ten beasts moved the archive's maze");
}
