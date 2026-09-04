//! The bailey, scripted — and the arithmetic a siege spell is written with.
//!
//! `tests/besieging.rs` proves the **game**: that the words resolve, that a
//! round pays, that a siege survives a save. `tests/solvers.rs` proves the
//! shipped spells run. Neither asks the question this file does: **can a person
//! write a siege spell at all**, and is every state they would need to write one
//! actually emitted?
//!
//! # Three claims, and the first is the one that catches the next bug
//!
//! 1. **Every reading the domain declares is reachable.** `siege::readings()` is
//!    what `recall scripting` teaches and what the scene registers, so a word in
//!    it that no state ever publishes is a word the manual offers and the world
//!    never answers — `if the coffer has quintessence` reading nought for ever,
//!    with the whole suite green. That shipped: the die prices were raised only
//!    by `defend::publish`, which nothing calls until a bailey verb runs, so a
//!    fresh tower priced every die at nothing and the affordability guard every
//!    solver ships answered *yes* on an empty pool.
//! 2. **A hand-written spell can use each of them**, compiled through the real
//!    editor and run through the real schedule.
//! 3. **The arithmetic is exact**, against numbers this domain fixes rather than
//!    a maze's luck: the pool opens at 24 and a `d20` costs 5, so `plus`,
//!    `double` and the comparison have known answers rather than derived ones.

use orbs_render::{FieldName, Value};
use orbs_sim::{Sim, tower};

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

fn at_the_wall(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    run(&mut sim, "attend bailey");
    sim
}

/// Every `Message` the orb has said.
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

/// Every bare word `survey` has printed — a reading with no number on it.
fn words(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| match record.field(FieldName::Name) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// Write a spell into the bailey and cast it, then hand back the log.
///
/// **Through `write` and `invoke`, never by hand.** A test that pokes a
/// `Program` into the world proves the runner works and says nothing about
/// whether a person could have got there — and *"can a person write this"* is
/// the whole question here.
fn cast(sim: &mut Sim, name: &str, lines: &[&str]) {
    let body: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    sim.write_spell(name, &body);
    sim.step();
    run(sim, &format!("invoke {name}"));
}

/// Whether a spell the orb was given reads clean.
///
/// The complaints are the thing to assert on: a line the compiler cannot read is
/// reported once per cast, and a spell that silently does nothing looks
/// identical to one that ran.
fn complaints(sim: &Sim) -> Vec<String> {
    said(sim)
        .into_iter()
        .filter(|line| {
            line.contains("means nothing")
                || line.contains("cannot read")
                || line.contains("is not a")
                || line.contains("no part of this")
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 1. Every declared reading is one the world actually publishes
// ---------------------------------------------------------------------------

/// **The lint that would have caught the die prices.**
///
/// `siege::readings()` is the domain's contract with the language: `scene_at`
/// registers every word in it unconditionally, so all of them *compile* in a
/// spell whether or not anything ever publishes them. That is deliberate — a
/// spell must compile before the siege it asks about exists — and it means the
/// declaration is the **only** thing standing between a word and silence.
///
/// So this walks a real siege through the states that produce each one and
/// asserts the word actually turns up. A reading nothing can publish is worse
/// than a missing one: `many_at` answers absent with nought, so the spell gets a
/// confident wrong number rather than an error.
#[test]
fn every_reading_the_domain_declares_is_one_some_state_reaches() {
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    // Sixty seeds, because several readings are conditional on how a fight goes:
    // `few` and `hurt` need a worn line, `outnumbered` a big enemy, `routed` a
    // broken one, and no single seed produces all of them.
    for seed in 0..60 {
        let mut sim = at_the_wall(seed);
        run(&mut sim, "defend");

        for _ in 0..40 {
            // Survey everything a spell can name, every round, and collect.
            for place in [
                tower::siege::RAMPART,
                tower::siege::GARRISON,
                tower::siege::ENEMY,
                tower::siege::COFFER,
            ] {
                run(&mut sim, &format!("survey {place}"));
            }
            for area in tower::siege::Area::ALL {
                run(&mut sim, &format!("survey {}", area.word()));
            }
            for die in tower::siege::POOL {
                run(&mut sim, &format!("survey {}", die.word()));
            }
            // Pledge what we can, **and survey the areas again afterwards** —
            // `ceiling` is raised only while something is on an area and `hold`
            // clears the pledges, so surveying before the pledge (as this did)
            // never sees it at all.
            for die in tower::siege::POOL {
                run(&mut sim, &format!("pledge {} buckler", die.word()));
            }
            for area in tower::siege::Area::ALL {
                run(&mut sim, &format!("survey {}", area.word()));
            }
            let ended = words(&sim).iter().any(|w| w == tower::siege::LIFTED);
            if ended {
                break;
            }
            run(&mut sim, "hold");
        }
        seen.extend(words(&sim));
    }

    let missing: Vec<&str> = tower::siege::readings()
        .into_iter()
        .filter(|word| !seen.contains(*word))
        .collect();
    assert!(
        missing.is_empty(),
        "the domain declares readings no state ever publishes, so a spell asking \
         for one gets nought for ever: {missing:?}",
    );
}

/// **A die's price is there before the first siege**, which it was not.
///
/// It was raised by `defend::publish`, and nothing calls that until a bailey verb
/// runs — so on a fresh tower `survey d20` answered *"the d20 holds nothing"* and
/// `not the coffer has fewer quintessence than the d20` compared nought against
/// nought and said **yes, afford it**, with no siege and no pool at all.
#[test]
fn a_die_is_priced_before_any_siege_has_been_fought() {
    let sim = Sim::new(11);
    let mut sim = sim;
    run(&mut sim, "attend bailey");
    for die in tower::siege::POOL {
        run(&mut sim, &format!("survey {}", die.word()));
    }
    let priced = sim
        .scrollback()
        .records()
        .iter()
        .filter(|record| {
            matches!(record.field(FieldName::Name),
                Some(Value::Text(name)) if name == tower::siege::QUINTESSENCE)
        })
        .count();
    assert_eq!(
        priced,
        tower::siege::POOL.len(),
        "not every die is priced on a tower that has never fought: {:?}",
        said(&sim),
    );
}

// ---------------------------------------------------------------------------
// 2. A person can write a spell with them
// ---------------------------------------------------------------------------

/// **Each new reading, in a spell a person typed.**
///
/// Compiling is not using: `scene_at` registers every declared word, so a spell
/// naming one always compiles. What this asserts is that the guard actually
/// *fires* — the spell acts, and the log says so.
#[test]
fn a_hand_written_spell_can_read_the_odds_and_the_pool() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");

    cast(
        &mut sim,
        "weighing",
        &[
            // `aim` is the enemy's chance to tell, which was drawn on the board
            // and askable by nobody until it was published.
            "if the enemy has more than 30 aim",
            "    pledge d20 to buckler",
            "end",
            // ...and the pool, read as a number.
            "if the coffer has more than 10 quintessence",
            "    pledge d8 to succour",
            "end",
            "hold",
        ],
    );
    for _ in 0..20 {
        sim.step();
    }

    assert!(complaints(&sim).is_empty(), "{:?}", complaints(&sim));
    let log = said(&sim);
    assert!(
        log.iter().any(|line| line.contains("behind the buckler")),
        "the `aim` rung never fired: {log:?}",
    );
    assert!(
        log.iter().any(|line| line.contains("behind the succour")),
        "the `quintessence` rung never fired: {log:?}",
    );
}

/// **The affordability guard, in the spelling the shipped solvers use.**
///
/// `not … fewer … than` is the language's route to *at least as many*, and the
/// affirmative is wrong rather than merely clumsy — a comparison against a place
/// is strict, so `has more quintessence than the d20` refuses the die you can
/// exactly afford. This asserts the double negative behaves.
#[test]
fn the_affordability_guard_pledges_while_it_can_and_stops_when_it_cannot() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");

    cast(
        &mut sim,
        "thrift",
        &[
            "repeat until the rampart has lifted",
            "    for each die",
            "        if not the coffer has fewer quintessence than die",
            "            pledge die to buckler",
            "        end",
            "    end",
            "    hold",
            "end",
        ],
    );
    for _ in 0..600 {
        sim.step();
    }

    assert!(complaints(&sim).is_empty(), "{:?}", complaints(&sim));
    let log = said(&sim);
    let pledged = log
        .iter()
        .filter(|line| line.contains("goes behind") || line.contains("goes to"))
        .count();
    let short = log
        .iter()
        .filter(|line| line.contains("would take"))
        .count();
    assert!(pledged > 0, "the guard never let a pledge through: {log:?}");
    assert_eq!(
        short, 0,
        "the guard let through a pledge it could not pay for, so the spelling is \
         backwards: {log:?}",
    );
}

/// **A spell can tell a win from a loss, which is the one thing `lifted` does
/// not say.**
///
/// `repeat until the rampart has lifted` is the loop guard — it exists because
/// `until the enemy has routed` can never come true when the *garrison* is the
/// side that broke, and the shipped solver spun for ever on a loss until it was
/// added. But `lifted` says only *over*, so the outcome is read afterwards by
/// asking **which band is routed**.
///
/// Both directions are asserted, because a test that only ever wins would pass
/// against a domain that could not express the loss at all — and losing is the
/// case the reading was added for.
#[cfg(debug_assertions)]
#[test]
fn a_spell_can_tell_which_way_a_finished_siege_went() {
    // A win: `debug_siege` leaves the enemy one round from breaking.
    let mut won = at_the_wall(11);
    run(&mut won, "defend");
    run(&mut won, "debug_siege");
    for _ in 0..6 {
        run(&mut won, "hold");
    }
    // **The rampart, not a band** — `lifted` is a fact about the *siege*, which
    // is the whole reason it exists rather than `routed` doing the job.
    run(&mut won, "survey rampart");
    run(&mut won, "survey enemy");
    run(&mut won, "survey garrison");
    let after = words(&won);
    assert!(
        after.iter().any(|w| w == tower::siege::LIFTED),
        "the siege did not end: {after:?}",
    );
    assert!(
        after.iter().any(|w| w == tower::siege::ROUTED),
        "nobody broke, so there is no outcome to read: {after:?}",
    );

    // The enemy is the broken side, and the garrison is not — which is exactly
    // what `if the enemy has routed` after the loop asks.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "debug_siege");
    cast(
        &mut sim,
        "aftermath",
        &[
            "repeat until the rampart has lifted",
            "    hold",
            "end",
            "if the enemy has routed",
            "    survey coffer",
            "end",
            "if the garrison has routed",
            "    survey garrison",
            "end",
        ],
    );
    for _ in 0..80 {
        sim.step();
    }
    assert!(complaints(&sim).is_empty(), "{:?}", complaints(&sim));
    assert!(
        said(&sim).iter().any(|line| line.contains("the wall")),
        "the siege never settled: {:?}",
        said(&sim),
    );
}

// ---------------------------------------------------------------------------
// 3. The arithmetic, against numbers this domain fixes
// ---------------------------------------------------------------------------

/// Whether a one-line question is true, right now, in this world.
///
/// Cast as a spell that says something when the guard holds — because a
/// condition is only observable through what it *does*, and a test that reached
/// into `watch::holds` would be testing the evaluator rather than the language a
/// player writes.
fn asks(sim: &mut Sim, name: &str, question: &str) -> bool {
    // **Counted before and after, never read off the tail.** A `survey rampart`
    // inside the guard writes a `turns` row, so the question held exactly when
    // one more of them exists than did a moment ago — which does not depend on
    // how much else the orb happened to say in between.
    let turns = |sim: &Sim| {
        sim.scrollback()
            .records()
            .iter()
            .filter(|record| {
                matches!(record.field(FieldName::Name),
                    Some(Value::Text(name)) if name == tower::siege::TURNS)
            })
            .count()
    };
    let before = turns(sim);
    cast(
        sim,
        name,
        &[&format!("if {question}"), "    survey rampart", "end"],
    );
    for _ in 0..8 {
        sim.step();
    }
    assert!(
        complaints(sim).is_empty(),
        "the orb could not read {question:?}: {:?}",
        complaints(sim),
    );
    turns(sim) > before
}

/// **The pool is 24 and a `d20` costs 5, so the comparison has a known answer.**
///
/// `tower::spell::watch`'s own tests weigh a maze, where which way was walked
/// more is a fact about the seed. Here both sides are fixed by the domain, so a
/// wrong answer is a wrong *number* rather than a wrong world.
#[test]
fn the_comparison_answers_what_the_numbers_say() {
    let base = tower::ceiling_for(tower::STANDING, 0);
    let d20 = tower::siege::cost_of(orbs_sim::tower::dice::Die::D20);
    assert_eq!(base, 24, "the fixture's premise moved");
    assert_eq!(d20, 5, "the fixture's premise moved");

    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");

    // 24 is not fewer than 5.
    assert!(
        !asks(
            &mut sim,
            "afford_one",
            "the coffer has fewer quintessence than the d20",
        ),
        "a full pool read as too poor for a d20",
    );

    // ...and the same question flips when the pool is short.
    //
    // **Set directly rather than spent down over rounds**, and that is the
    // sharper test rather than the lazier one. It used to pledge everything for
    // three rounds, which emptied the pool *exactly* — an arithmetic
    // coincidence that stopped holding the moment `hold` began granting
    // `REGEN_PER_ROUND` back, and would have stopped again on any tuning pass.
    // Worse, a stronger garrison now ends the fight sooner, so the loop could
    // run out of siege before it ran out of quintessence and the test would fail
    // for a reason that has nothing to do with the comparison.
    //
    // What is being asked is whether `fewer … than` reads two numbers correctly
    // at the boundary. So put the number there.
    let d20_cost = tower::siege::cost_of(orbs_sim::tower::dice::Die::D20);
    sim.world_mut()
        .insert_resource(tower::Quintessence::new(d20_cost - 1));
    // **A verb, because the coffer's readings are published by one.** Setting
    // the resource changes what the tower holds and nothing else; `publish` runs
    // on `defend`, `pledge` and `hold`. A `d6` costs one, so this leaves the
    // pool short of a `d20` either way and is the cheapest thing that republishes.
    run(&mut sim, "pledge d6 buckler");
    assert!(
        asks(
            &mut sim,
            "afford_two",
            "the coffer has fewer quintessence than the d20",
        ),
        "an empty pool read as able to pay for a d20",
    );
}

/// **`plus` and `double`, evaluated rather than merely parsed.**
///
/// The garrison opens at six spears against an enemy of five to nine, so
/// `double the garrison` is twelve — more than any enemy this domain draws. That
/// makes the two directions of the comparison assertable without knowing the
/// seed.
#[test]
fn double_and_plus_are_worth_what_they_say() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");

    // The enemy is at most nine and twice the garrison is twelve.
    assert!(
        !asks(
            &mut sim,
            "twice_one",
            "the enemy has more spears than double the garrison",
        ),
        "the enemy outnumbered twice the garrison on the opening round",
    );
    // ...and it is more than the garrison alone, or `outnumbered` would be a lie.
    assert!(
        asks(
            &mut sim,
            "twice_two",
            "the enemy has fewer spears than double the garrison",
        ),
        "`double` did not double",
    );

    // `plus` shifts a threshold by exactly what it says. The garrison is six, so
    // it is not more than itself plus nought and is not more than itself plus
    // anything.
    assert!(
        !asks(
            &mut sim,
            "plus_one",
            "the garrison has more spears than the garrison plus 0",
        ),
        "a band was strictly more than itself",
    );
    assert!(
        asks(
            &mut sim,
            "plus_two",
            "the enemy has fewer spears than the garrison plus 6",
        ),
        "`plus 6` did not raise the threshold above any enemy this domain draws",
    );
}

/// **`plus 0` is the identity, which is how `strict` is caught reading the
/// variant instead of the grammar.**
///
/// Derived from the `Quantity` variant, the expression forms all fall to
/// *inclusive* while a bare place is *strict* — so `than the garrison` and `than
/// the garrison plus 0` would disagree at equality, which is exactly where a
/// comparison is asked.
#[test]
fn adding_nothing_changes_no_answer() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");

    for (bare, padded) in [
        (
            "the garrison has more spears than the enemy",
            "the garrison has more spears than the enemy plus 0",
        ),
        (
            "the garrison has fewer spears than the enemy",
            "the garrison has fewer spears than the enemy plus 0",
        ),
    ] {
        let one = asks(&mut sim, "pad_a", bare);
        let two = asks(&mut sim, "pad_b", padded);
        assert_eq!(
            one, two,
            "`plus 0` changed the answer to {bare:?}, so strictness is reading \
             the variant rather than the grammar",
        );
    }
}

/// **A far-side reading the tower has never heard of is refused at cast**, which
/// is better than the nought I expected and worth pinning as the guarantee it is.
///
/// `nothing-is-named-this` is not a nameable word, so `compile::fix` leaves it
/// `Unplaced::Thing` and the whole question is unreadable — *"that question means
/// nothing. neither half runs"*. The far side inherits that from the near one
/// for free, because `Condition::rename` walks the whole tree; had it not, an
/// unknown word over there would have read as nought and answered confidently.
#[test]
fn an_unknown_reading_on_the_far_side_is_refused_rather_than_read_as_nought() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");

    cast(
        &mut sim,
        "unknown",
        &[
            "if the garrison has more spears than the enemy has nothing-is-named-this",
            "    survey rampart",
            "end",
        ],
    );
    for _ in 0..8 {
        sim.step();
    }
    assert!(
        complaints(&sim)
            .iter()
            .any(|line| line.contains("means nothing")),
        "an unknown far-side reading was accepted: {:?}",
        said(&sim),
    );
}

/// **A *declared* reading that is simply absent here reads as nought**, and the
/// operators do not know they are adding to nothing.
///
/// This is the case with no design fix, only the knowledge. `ceiling` is a real
/// word — it compiles anywhere, because `scene_at` registers the whole declared
/// set — and it is published on *areas*, never on a band. So asking a band for it
/// is nought, and `plus 3` over that is three.
///
/// Pinned rather than asserted away: a spell that asks the wrong node gets a
/// confident wrong number, which is why CLAUDE.md's rule is to test every new
/// comparison against an absent reading.
#[test]
fn the_arithmetic_over_an_absent_reading_is_arithmetic_over_nought() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");

    // The garrison opens at six spears; a band publishes no `ceiling` at all.
    assert!(
        asks(
            &mut sim,
            "absent_one",
            "the garrison has more spears than the enemy has ceiling",
        ),
        "an absent far-side reading did not count as nought",
    );
    // ...and `plus 3` over that absence is three, which six still beats.
    assert!(
        asks(
            &mut sim,
            "absent_two",
            "the garrison has more spears than the enemy has ceiling plus 3",
        ),
        "`plus` over an absent reading did not add to nought",
    );
    // ...but `plus 20` over it is twenty, which six does not.
    assert!(
        !asks(
            &mut sim,
            "absent_three",
            "the garrison has more spears than the enemy has ceiling plus 20",
        ),
        "`plus` over an absent reading was ignored rather than added to nought",
    );
}

/// **Absurd numbers saturate rather than panicking.**
///
/// `by` is a number the player typed and `double` multiplies a world read, so
/// both are reachable with values that would overflow in a debug build — and a
/// question that took the tower down over a sentence would be the worst possible
/// failure for a line a player is still editing.
#[test]
fn an_absurd_threshold_saturates_and_answers() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");

    assert!(
        !asks(
            &mut sim,
            "huge",
            "the garrison has more spears than the enemy plus 4294967295",
        ),
        "a saturating threshold answered yes",
    );
    // ...and the whole tree, doubled and added to, still answers.
    assert!(
        !asks(
            &mut sim,
            "huge_two",
            "the garrison has more spears than double the enemy plus 4294967290",
        ),
        "a doubled saturating threshold answered yes",
    );
}
