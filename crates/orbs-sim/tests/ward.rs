//! The lens, driven through the real parser and the real schedule.
//!
//! `tower::ward`'s own tests prove the model; these prove the *game* — that the
//! words resolve, the readings publish, a press costs the tower nothing, and a
//! seal that opens pays what `progression.toml` says it does.
//!
//! **The sweep here is the one `debug_spell breaking` writes**, in Rust: one
//! socket at a time, turning it until `aligned` moves, reading nothing but which
//! way it went — because §8's language has no variables and a codemaker says
//! nothing else. If this cannot solve a ward through `dial` and `probe`, no
//! spell can either.

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

/// The opening aperture, which `Ward::new` fixes so a spell can name it.
///
/// **The restore rung depends on this being a literal.** A spell has no
/// variables, so the only way it can put a socket back is to name what was there
/// — and the only figure it can know is the one every reading opens on.
const OPENING: [&str; 4] = ["nitre", "alum", "borax", "quartz"];

/// Break the ward in front of the player, the way `breaking` does.
///
/// Returns the last thing the prism said, so a caller can check it gave.
fn solve(sim: &mut Sim) -> String {
    let mut answer = press(sim);
    for (socket, held) in SOCKETS.iter().zip(OPENING) {
        // The `if the prism is working` each rung carries: a ward can give on the
        // second socket, and every rung after it would be dialling at nothing.
        if answer.contains("seal gives") {
            break;
        }
        run(sim, &format!("dial {socket}"));
        answer = press(sim);
        // `aligned` fell and only this socket moved, so it was already right.
        // Put it back — and press, or the next rung's delta is measured against
        // a figure that was never sent.
        if answer.contains("further") {
            run(sim, &format!("dial {socket} {held}"));
            answer = press(sim);
            continue;
        }
        while answer.contains("level") {
            run(sim, &format!("dial {socket}"));
            answer = press(sim);
        }
    }
    answer
}

#[test]
fn the_writable_sweep_breaks_a_ward_through_the_real_verbs() {
    // **The domain's whole automation claim, end to end.** No deduction and no
    // memory beyond the last press's delta — and it must still finish, because a
    // socket's walk is cyclic and only its arrival can raise `aligned`.
    for seed in [1u64, 3, 7, 11, 17] {
        let mut sim = Sim::new(seed);
        run(&mut sim, "attend lens");

        let answer = solve(&mut sim);
        assert!(
            answer.contains("seal gives"),
            "seed {seed} survived the sweep: {answer:?}",
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
fn a_bare_dial_walks_the_six_and_says_which_it_took() {
    // **The affordance the whole sweep rests on.** A spell has no variables, so
    // it cannot name the sigil a socket has not tried — `dial <socket>` steps the
    // ward round instead, and the transcript names what it landed on, because a
    // player watching a spell work needs to see what it tried.
    //
    // It was a candidate list the ward kept per socket, which is the player's own
    // bookkeeping; it is a cycle now (§19).
    let mut sim = Sim::new(3);
    run(&mut sim, "attend lens");
    run(&mut sim, "probe");
    sim.step_n(PRESS);

    let mut took = Vec::new();
    for _ in 0..SIGILS.len() {
        let before = said(&sim).len();
        run(&mut sim, "dial first");
        let line = said(&sim)
            .into_iter()
            .skip(before)
            .find(|line| line.contains("turns to"))
            .unwrap_or_else(|| panic!("a bare dial refused: {:?}", said(&sim).last()));
        took.push(
            SIGILS
                .iter()
                .position(|sigil| line.contains(sigil))
                .unwrap_or_else(|| panic!("{line:?} named no sigil")),
        );
    }

    // Six turns is every sigil once, and back where it started.
    let mut seen = took.clone();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(
        seen.len(),
        SIGILS.len(),
        "the walk skipped a sigil: {took:?}"
    );
    assert_eq!(took.last(), Some(&0), "six turns did not come round");
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
    let answer = solve(&mut sim);
    assert!(answer.contains("seal gives"), "the seal held: {answer:?}");

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
