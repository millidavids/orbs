//! The lens, driven through the real parser and the real schedule.
//!
//! `tower::ward`'s own tests prove the model; these prove the *game* — that the
//! words resolve, the readings publish, a press costs the tower nothing, and a
//! seal that opens pays what `progression.toml` says it does.
//!
//! **The ladder here is the one `debug_spell breaking` will write**, in Rust:
//! walk the sockets and the sigils, keep what gains, and never revert — because
//! §8's language has no variables and a spell cannot remember what a socket held.
//! If this cannot solve a ward through `dial` and `probe`, no spell can either.

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;
use orbs_sim::content::Recipes;

/// Six sigils and four sockets, spelled out — a test that imported them from the
/// sim could not catch a rename that broke the *player's* vocabulary.
const SIGILS: [&str; 6] = ["nitre", "alum", "borax", "quartz", "pewter", "ochre"];
const SOCKETS: [&str; 4] = ["first", "second", "third", "fourth"];

/// Ticks to let a press settle.
///
/// **A press is instant now**, so this buys nothing a press needs — it is kept
/// because these tests also let *other* work land between presses, and a helper
/// that stepped nought would make a future duration invisible here rather than
/// failing loudly.
const PRESS: u64 = 1;

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

/// Press, wait for the answer, and say which way the ward moved.
fn press(sim: &mut Sim) -> String {
    let before = said(sim).len();
    run(sim, "probe");
    sim.step_n(PRESS);
    said(sim)
        .into_iter()
        .skip(before)
        .find(|line| line.contains("aligned") || line.contains("seal gives"))
        .unwrap_or_default()
}

#[test]
fn a_ward_opens_on_the_first_probe_and_publishes_what_it_answered() {
    let mut sim = Sim::new(3);
    run(&mut sim, "attend lens");

    let answer = press(&mut sim);
    assert!(
        answer.contains("aligned"),
        "the first probe said nothing about the ward: {answer:?}",
    );
    assert!(
        said(&sim).iter().any(|line| line.contains("far orb")),
        "probing with no reading open did not open one",
    );

    // The readings a spell asks for, through the same `survey` a player types.
    run(&mut sim, "survey prism");
    let listed = said(&sim);
    assert!(
        listed.iter().any(|line| line.contains("aligned"))
            || sim.scrollback().records().iter().any(|record| matches!(
                record.field(FieldName::Name),
                Some(Value::Text(text)) if text == "aligned"
            )),
        "the prism published no readings",
    );
}

#[test]
fn a_blind_ladder_breaks_a_ward_through_the_real_verbs() {
    // **The domain's whole automation claim, end to end.** No deduction, no
    // revert, no memory beyond what the world publishes — and it must still
    // finish, because the ward ratchets and a settled socket is refused.
    for seed in [1u64, 3, 7, 11, 17] {
        let mut sim = Sim::new(seed);
        run(&mut sim, "attend lens");

        let mut answer = press(&mut sim);
        let mut tried: Vec<(usize, usize)> = Vec::new();
        let mut presses = 1;

        while !answer.contains("seal gives") && presses < 120 {
            let Some((socket, sigil)) = (0..SOCKETS.len())
                .flat_map(|socket| (0..SIGILS.len()).map(move |sigil| (socket, sigil)))
                .find(|pair| !tried.contains(pair))
            else {
                tried.clear();
                continue;
            };
            tried.push((socket, sigil));

            let before = said(&sim).len();
            run(
                &mut sim,
                &format!("dial {} {}", SOCKETS[socket], SIGILS[sigil]),
            );
            // A dial the ward refused — settled, or already there. Nothing was
            // spent, so try the next rung without pressing.
            let turned = said(&sim)
                .into_iter()
                .skip(before)
                .any(|line| line.contains("turns to"));
            if !turned {
                continue;
            }

            answer = press(&mut sim);
            presses += 1;
            if answer.contains("closer") {
                tried.clear();
            }
        }

        assert!(
            answer.contains("seal gives"),
            "seed {seed} survived {presses} presses: {answer:?}",
        );
        assert!(
            sim.experience() > 0,
            "seed {seed} broke a seal and earned nothing",
        );
    }
}

/// **Gated.** `debug_ward` makes the answer whatever the aperture holds, and
/// `debug_spell` writes the solver ladder — both `cfg(debug_assertions)`, and
/// both unresolvable lines in a release build rather than errors. Without this
/// the test ran against a ward nobody had broken and a spell that was never
/// written, and failed saying the seed could not bind anything.
///
/// The honest alternative is brute-forcing a 360-code ward per seed, which
/// `a_blind_ladder_breaks_a_ward_through_the_real_verbs` already does through
/// the real verbs and in either profile — so what is gated here is the
/// *shortcut*, not the coverage.
#[cfg(debug_assertions)]
#[test]
fn the_solver_spell_keeps_solving_and_never_goes_quiet() {
    // **The test above is a *reference* ladder, written in Rust, and it does not
    // run the spell.** It has a `tried.clear()` the shipped spell had no way to
    // express, so it was proving a Rust loop terminates while the thing a player
    // actually casts went unmeasured. `debug_spell breaking` is what this runs.
    //
    // **A faucet that stops is the failure mode**, not a crash: a ward solver that
    // runs out of moves presses an unchanged aperture for ever, holds the tower's
    // one production slot and earns nothing. So the assertion is on the *second*
    // half of a long run earning as much as the first — a plateau is what a stall
    // looks like from outside.
    //
    // `MAX_MEDITATE` is 3600, so a longer wait must be several commands. One
    // `meditate 9600` silently becomes 3600, which is how a first pass at this
    // "found" a plateau that was the cap.
    //
    // **Bound, not invoked, and the difference is new.** `repeat until the prism
    // is idle` solves the ward in front of it and exits — with a press in flight
    // for twelve ticks the guard never saw the solve, so an invocation lapped by
    // accident. An instant press lands the solve before the guard is asked, so an
    // invocation is now one ward and the faucet is the *binding*, which re-casts a
    // spell that has run off the end. That is the honest shape: an invocation is an
    // act, a binding is standing automation.
    for seed in [1u64, 3, 11] {
        let mut sim = Sim::new(seed);
        run(&mut sim, "attend lens");
        // Two solves buy the first concentration slot. `debug_ward` makes the
        // answer whatever the aperture holds, so the next press breaks it.
        for _ in 0..2 {
            run(&mut sim, "probe");
            run(&mut sim, "debug_ward");
            run(&mut sim, "probe");
        }
        assert!(
            sim.concentration() > 0,
            "seed {seed} could not bind anything"
        );

        run(&mut sim, "debug_spell breaking");
        run(&mut sim, "bind breaking");

        let start = sim.experience();
        run(&mut sim, "meditate 3600");
        let half = sim.experience() - start;
        run(&mut sim, "meditate 3600");
        let full = sim.experience() - start;

        assert!(half > 0, "seed {seed} earned nothing in the first hour");
        let second = full - half;
        assert!(
            second * 2 >= half,
            "seed {seed} earned {half} in its first hour and {second} in its \
             second — the solver has run out of moves and is pressing for ever",
        );
    }
}

#[test]
fn a_bare_dial_turns_a_socket_and_publishes_what_is_left() {
    // **The affordance the whole four-rung ladder rests on.** A spell has no
    // variables, so it cannot name the sigil a socket has not tried — `dial
    // <socket>` asks the ward instead. `Ward::untried` is the model's half; this
    // is the half a spell can actually see, and a count the ward kept and never
    // published would leave every ladder blind while the unit tests passed.
    let mut sim = Sim::new(3);
    run(&mut sim, "attend lens");
    run(&mut sim, "probe");
    sim.step_n(PRESS);

    let before = untried(&mut sim, 0);
    assert_eq!(before, 6, "a fresh socket has every sigil left");

    // **Spending a candidate is the claim, not moving the aperture.** The opening
    // aperture already holds `nitre` in the first socket, so the first bare dial
    // there turns nothing and still has to count — a ladder that only counted
    // *movement* would aim at that socket for ever.
    run(&mut sim, "dial first");
    assert!(
        !said(&sim)
            .last()
            .is_some_and(|line| line.contains("round them all")),
        "a fresh socket refused a bare dial: {:?}",
        said(&sim).last(),
    );
    assert_eq!(
        untried(&mut sim, 0),
        before - 1,
        "a bare dial did not spend a candidate",
    );

    // ...and it does turn one when the sigil is not already there.
    run(&mut sim, "dial second");
    run(&mut sim, "dial second");
    assert!(
        said(&sim).iter().any(|line| line.contains("turns to")),
        "no bare dial ever moved a socket: {:?}",
        said(&sim),
    );
}

/// What `survey <socket>` reports as `untried`, read through the real verb.
///
/// Through `survey` rather than the model, because what a spell guards on is the
/// *published* reading — a count the ward held and never raised would satisfy an
/// assertion on `Ward` and leave every ladder blind.
fn untried(sim: &mut Sim, socket: usize) -> u32 {
    let name = SOCKETS[socket];
    run(sim, &format!("survey {name}"));
    // **Backwards from the newest**, rather than skipping a prefix measured before
    // the command: `Records` is a window, so an index taken earlier does not stay
    // pointing at the same record and `skip` quietly walked past the answer.
    let seen: Vec<u32> = sim
        .scrollback()
        .records()
        .iter()
        .filter_map(|record| match record.field(FieldName::Name) {
            // **`Quantity` is `Text`, not `Count`.** `survey` writes
            // `Stock::label()`, which is a string — endless stock draws `∞` and
            // has no number at all — so matching `Value::Count` silently read
            // every amount in the game as nought.
            Some(Value::Text("untried")) => Some(match record.field(FieldName::Quantity) {
                Some(Value::Text(count)) => count.parse().unwrap_or(0),
                _ => 0,
            }),
            _ => None,
        })
        .collect();
    seen.last().copied().unwrap_or(0)
}

#[test]
fn a_broken_seal_spills_a_log_that_only_says_true_things() {
    // **The yield, and the one way it could be quietly wrong.** Every line names
    // a real instrument doing something it can actually do — a template filled
    // from a free-for-all of names would print `digest sage`, which is plausible
    // and false, and a player who tried it would learn the wrong thing about
    // their own tower.
    let mut sim = Sim::new(3);
    run(&mut sim, "attend lens");
    let mut answer = press(&mut sim);
    let mut tried: Vec<(usize, usize)> = Vec::new();

    while !answer.contains("seal gives") {
        let Some((socket, sigil)) = (0..SOCKETS.len())
            .flat_map(|socket| (0..SIGILS.len()).map(move |sigil| (socket, sigil)))
            .find(|pair| !tried.contains(pair))
        else {
            tried.clear();
            continue;
        };
        tried.push((socket, sigil));
        let before = said(&sim).len();
        run(
            &mut sim,
            &format!("dial {} {}", SOCKETS[socket], SIGILS[sigil]),
        );
        if !said(&sim)
            .into_iter()
            .skip(before)
            .any(|line| line.contains("turns to"))
        {
            continue;
        }
        answer = press(&mut sim);
        if answer.contains("closer") {
            tried.clear();
        }
    }

    // The spill is **quiet**, so it is in the stream and not on the transcript.
    let drawn = sim.scrollback().records().drawn().count();
    let all = sim.scrollback().records().iter().count();
    assert!(all > drawn, "nothing was logged without being drawn");

    // Every stolen line is `<verb> <material>`, and both halves must be real.
    let verbs = ["grind", "digest", "mix", "distil", "wield"];
    let spilled: Vec<String> = said(&sim)
        .into_iter()
        .filter(|line| verbs.iter().any(|verb| line.starts_with(verb)))
        .collect();
    assert!(!spilled.is_empty(), "a broken seal spilled nothing");

    // **The pairs the tower could actually produce**, asked of `Recipes` rather
    // than listed here. The first version of this test checked the verb was in the
    // array above and that there were exactly two words — which is the *shape* of
    // the claim in the comment and not the claim. A generator changed to draw a
    // material at random would have printed `digest sage` and passed: plausible,
    // false, and the exact failure the comment warns about.
    let real: Vec<String> = {
        let recipes = sim.world().resource::<Recipes>();
        recipes
            .instruments()
            .into_iter()
            .flat_map(|instrument| {
                recipes
                    .for_instrument(instrument)
                    .iter()
                    .filter_map(move |recipe| {
                        let input = recipe.inputs().first().copied()?;
                        Some(format!("{instrument} {input}"))
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    };

    for line in &spilled {
        let mut words = line.split_whitespace();
        let (Some(verb), Some(material)) = (words.next(), words.next()) else {
            panic!("a spilled line is not a command: {line:?}");
        };
        assert!(verbs.contains(&verb), "{line:?} names no real verb");
        assert!(
            words.next().is_none(),
            "{line:?} is not one verb and one material",
        );
        assert!(
            real.iter().any(|pair| pair.ends_with(material)),
            "{line:?} names a material no instrument in the tower takes",
        );
    }
}

#[test]
fn a_press_is_instant_and_takes_no_slot() {
    // **ROADMAP's *"a read is not a brew"* is withdrawn**, and this is the
    // assertion that says so rather than leaving the old one to rot: a press used
    // to schedule twelve ticks through the production machinery and now answers on
    // the tick it is typed, taking nothing.
    //
    // Both halves matter. The *answer* landing at once is what a player feels; the
    // *slot* staying free is what lets a bound solver run beside a full brewing
    // loop with no contention at all, which is a balance fact and is in §19.
    let mut sim = Sim::new(3);
    run(&mut sim, "attend lens");

    let before = sim.experience();
    run(&mut sim, "probe");
    assert!(
        sim.working().is_none(),
        "a press took the production slot, so it is still competing with a brew",
    );
    assert!(
        said(&sim).iter().any(|line| line.contains("astray")),
        "the ward did not answer on the tick it was pressed: {:?}",
        said(&sim).last(),
    );

    // ...and it really did press, rather than answering out of nothing.
    assert!(
        sim.experience() >= before,
        "a press that costs nothing must still be a press",
    );
}

/// **Gated.** `debug_ward` makes the answer whatever the aperture holds, and
/// `debug_spell` writes the solver ladder — both `cfg(debug_assertions)`, and
/// both unresolvable lines in a release build rather than errors. Without this
/// the test ran against a ward nobody had broken and a spell that was never
/// written, and failed saying the seed could not bind anything.
///
/// The honest alternative is brute-forcing a 360-code ward per seed, which
/// `a_blind_ladder_breaks_a_ward_through_the_real_verbs` already does through
/// the real verbs and in either profile — so what is gated here is the
/// *shortcut*, not the coverage.
#[cfg(debug_assertions)]
#[test]
fn a_solver_keeps_working_after_the_player_walks_out() {
    // **The domain's own selling point, and nothing covered it.** `land` runs in
    // the tick schedule, outside `spell::run`'s domain swap — so a `refresh` that
    // resolved the prism from `Cwd` published nothing whenever the player was
    // elsewhere. Every reading froze at the last `dial`: a socket that had just
    // settled still read `loose`, the ladder dialled it, `seat` refused, and the
    // rung fired for ever.
    //
    // Every other test in this file stands in the lens for its whole run, which
    // is exactly why this went unseen.
    // **`bind`, not `invoke`, and the distinction is the whole test.** An
    // invocation ends when the player walks out — *"first_light.spell needed you
    // there. it stops"* — so only a binding is unattended, and only a binding
    // reaches the `land`-outside-the-domain-swap path at all.
    //
    // `bind` costs 16 experience and nothing grants it, so the wards are broken
    // by hand first: `debug_ward` hands the answer to the aperture, and a press
    // beyond par pays 6.
    let mut sim = Sim::new(3);
    run(&mut sim, "attend lens");
    while sim.experience() < 16 {
        run(&mut sim, "probe");
        sim.step_n(PRESS);
        run(&mut sim, "debug_ward");
        run(&mut sim, "probe");
        sim.step_n(PRESS);
    }

    run(&mut sim, "debug_spell breaking");
    run(&mut sim, "bind breaking");
    let earned = sim.experience();

    // Out of the room, and then a long wait with nobody watching.
    run(&mut sim, "attend laboratory");
    sim.step_n(900);

    assert!(
        sim.experience() > earned,
        "a bound solver broke nothing once the player left the room",
    );
    assert!(
        said(&sim).iter().any(|line| line.contains("seal gives")),
        "no seal gave while the player was in the laboratory",
    );
}

/// **Gated.** `debug_ward` makes the answer whatever the aperture holds, and
/// `debug_spell` writes the solver ladder — both `cfg(debug_assertions)`, and
/// both unresolvable lines in a release build rather than errors. Without this
/// the test ran against a ward nobody had broken and a spell that was never
/// written, and failed saying the seed could not bind anything.
///
/// The honest alternative is brute-forcing a 360-code ward per seed, which
/// `a_blind_ladder_breaks_a_ward_through_the_real_verbs` already does through
/// the real verbs and in either profile — so what is gated here is the
/// *shortcut*, not the coverage.
#[cfg(debug_assertions)]
#[test]
fn a_bound_solver_leaves_the_tower_a_share_of_its_one_slot() {
    // **It takes *none* of it now, and that is the assertion.** §19 refuses the
    // production slot to a maze because *"a solver holding the tower's one slot
    // would starve every other spell into `spell_gave_up`"*, and this measured a
    // bound `breaking` pressing twelve ticks out of roughly every thirteen —
    // contended but not starving. A press is instant and schedules nothing, so the
    // share is now nought and the question the test was written to answer has been
    // answered by removing the cost rather than by arithmetic.
    //
    // Kept rather than deleted: the *claim* it holds — the lens does not own the
    // slot — is the one that matters, and it is what a future decision to price a
    // press again would break.
    const SPAN: u32 = 600;

    let mut sim = Sim::new(3);
    run(&mut sim, "attend lens");
    // Bound, not invoked: an instant press lets the loop's guard see the solve, so
    // an invocation is one ward now and only a binding laps.
    for _ in 0..2 {
        run(&mut sim, "probe");
        run(&mut sim, "debug_ward");
        run(&mut sim, "probe");
    }
    run(&mut sim, "debug_spell breaking");
    run(&mut sim, "bind breaking");

    let mut busy = 0u32;
    for _ in 0..SPAN {
        if sim.working().is_some() {
            busy += 1;
        }
        sim.step();
    }

    let share = f64::from(busy) / f64::from(SPAN);
    assert!(
        sim.experience() > 0,
        "the solver broke nothing in {SPAN} ticks",
    );
    // What this holds is that the lens does **not** own the slot, so a brew beside
    // it is unaffected. It was `< 0.97` — contended but not starving — and is now
    // nought, because a press schedules nothing at all.
    assert_eq!(
        share,
        0.0,
        "a bound solver held the production slot {:.0}% of the time; a press is \
         supposed to cost the tower nothing",
        share * 100.0,
    );
}

#[test]
fn dialling_before_a_reading_is_open_says_what_to_do_instead() {
    // §15's dead-end rule: a refusal that names the way forward is why all of
    // the lens's words are live rather than gated.
    let mut sim = Sim::new(3);
    run(&mut sim, "attend lens");
    run(&mut sim, "dial first nitre");

    assert!(
        said(&sim).iter().any(|line| line.contains("probe first")),
        "dialling with nothing open did not name the way forward: {:?}",
        said(&sim),
    );
}

#[test]
fn the_lens_s_words_mean_nothing_in_the_laboratory() {
    // Both verbs are `is_operation`, so `Scene::offering` keeps them out of
    // every other room — which is what lets a three-verb domain cost the shared
    // vocabulary nothing. The answer must be *"not here"*, never *"no such
    // word"*, or the game would be denying a word it taught next door.
    let mut sim = Sim::new(3);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "probe");

    let lines = said(&sim);
    assert!(
        !lines.iter().any(|line| line.contains("far orb")),
        "a ward opened in the laboratory: {lines:?}",
    );
}
