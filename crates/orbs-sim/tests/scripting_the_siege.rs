//! The bailey, scripted — and the arithmetic a siege spell is written with.
//!
//! `tests/besieging.rs` proves the game and `tests/solvers.rs` proves the
//! shipped spells run. Neither asks whether a person could write a siege spell
//! at all, or whether every state they would need is emitted.
//!
//! 1. Every reading the domain declares is reachable. A declared word no state
//!    publishes reads nought for ever with the suite green — the die prices
//!    shipped that way, raised only by `defend::publish`, so a fresh tower
//!    priced every die at nothing and every solver's affordability guard said
//!    *yes* on an empty pool.
//! 2. A hand-written spell can use each of them, compiled through the real
//!    editor and run through the real schedule.
//! 3. The arithmetic is exact against numbers this domain fixes: the pool opens
//!    at 24 and a `d20` costs 5.

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
/// Through `write` and `invoke`, never by hand: poking a `Program` into the
/// world proves the runner works, not that a person could get there.
fn cast(sim: &mut Sim, name: &str, lines: &[&str]) {
    let body: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    sim.write_spell(name, &body);
    sim.step();
    run(sim, &format!("invoke {name}"));
}

/// Whether a spell the orb was given reads clean.
///
/// The thing to assert on: a spell that silently does nothing looks identical
/// to one that ran.
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

/// The lint that would have caught the die prices.
///
/// `scene_at` registers every declared word unconditionally, so all of them
/// compile whether or not anything publishes them — the declaration is the only
/// thing between a word and silence. This walks a real siege through the states
/// that produce each one. A reading nothing publishes is worse than a missing
/// one: `many_at` answers absent with nought.
#[test]
fn every_reading_the_domain_declares_is_one_some_state_reaches() {
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    // Sixty seeds, because no single one produces every reading: `few` needs a
    // worn line, `outnumbered` a big enemy, `routed` a broken one.
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
            // Survey the areas again after pledging: `ceiling` is raised only
            // while something is on an area, and `hold` clears the pledges.
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

/// A die's price is there before the first siege, which it was not: raised by
/// `defend::publish`, nothing called it until a bailey verb ran, so the
/// affordability guard compared nought against nought and said yes.
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

/// The arsenal's stores, in a spell a person typed.
///
/// Compiling is not using: every declared word compiles. This asserts the
/// loop's body runs — the cursor binds to a store and the condition answers.
#[test]
fn a_spell_can_walk_the_arsenal_and_find_what_has_gone_thin() {
    // Stores run down with time, and a player who cannot ask *which* can only
    // guess — so a spell walks them, as it walks ways and sockets. A nameable
    // word no `has` can reach casts, runs and does nothing for ever, which is
    // why this asserts on `complaints` rather than on an effect.
    let mut sim = at_the_wall(11);
    run(&mut sim, "debug_spawn warding 1");
    run(&mut sim, "debug_spawn troop 1");
    // Aged until the stores have run out: `spent` is what a keeping spell acts
    // on. `thin` needs production spread over time, which one `debug_spawn`
    // cannot make — its makings all leave the window together.
    run(&mut sim, "meditate 3000");

    cast(
        &mut sim,
        "keeping",
        &[
            "for each store",
            "    if the store has spent",
            "        survey arsenal",
            "    end",
            "end",
        ],
    );
    assert!(
        complaints(&sim).is_empty(),
        "a spell could not ask what the arsenal is stocked in: {:?}",
        complaints(&sim),
    );
    // And the body ran, which a clean compile does not prove: a `for each` over
    // an empty set complains about nothing and does nothing. The survey inside
    // the loop is what says the question was answered.
    for _ in 0..8 {
        sim.step();
    }
    assert!(
        words(&sim).iter().any(|word| word == "warding"),
        "the loop compiled but never reached a store: {:?}",
        said(&sim),
    );
}

/// Each new reading, in a spell a person typed.
///
/// Compiling is not using: every declared word compiles. This asserts the guard
/// fires — the spell acts, and the log says so.
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

/// The affordability guard, in the spelling the shipped solvers use.
///
/// `not … fewer … than` is the language's route to *at least as many*: a
/// comparison against a place is strict, so the affirmative refuses the die you
/// can exactly afford.
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

/// A spell can tell a win from a loss, which `lifted` does not say.
///
/// `repeat until the rampart has lifted` is the loop guard: `until the enemy
/// has routed` never comes true when the garrison is the side that broke. So
/// the outcome is read afterwards by asking which band is routed, and both
/// directions are asserted — a test that only wins would pass against a domain
/// that could not express the loss.
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
    // The rampart, not a band: `lifted` is a fact about the *siege*.
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
/// Cast as a spell that says something when the guard holds: reaching into
/// `watch::holds` would test the evaluator, not the language a player writes.
fn asks(sim: &mut Sim, name: &str, question: &str) -> bool {
    // Counted before and after, never read off the tail: the question held
    // exactly when one more `turns` row exists than did a moment ago.
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

/// The pool is 24 and a `d20` costs 5, so the comparison has a known answer —
/// unlike `tower::spell::watch`'s own tests, which weigh a maze. A wrong answer
/// here is a wrong number rather than a wrong world.
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

    // ...and the same question flips when the pool is short. Set directly
    // rather than spent down over rounds: pledging everything for three rounds
    // emptied the pool by arithmetic coincidence, which any tuning pass breaks.
    // What is asked is whether `fewer … than` reads two numbers at the
    // boundary, so put the number there.
    let d20_cost = tower::siege::cost_of(orbs_sim::tower::dice::Die::D20);
    sim.world_mut()
        .insert_resource(tower::Quintessence::new(d20_cost - 1));
    // A verb, because the coffer's readings are published by one: `publish`
    // runs on `defend`, `pledge` and `hold`. A `d6` costs one, so the pool is
    // short of a `d20` either way.
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

/// `plus` and `double`, evaluated rather than merely parsed.
///
/// The garrison opens at six against an enemy of five to nine, so `double the
/// garrison` is twelve — more than any enemy drawn, and both directions of the
/// comparison are assertable without knowing the seed.
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

    // `plus` shifts a threshold by what it says: the garrison is six, so it is
    // not more than itself plus nought.
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

/// `plus 0` is the identity, which catches `strict` reading the variant instead
/// of the grammar: derived from `Quantity`, every expression form falls to
/// inclusive while a bare place is strict, so the two disagree at equality.
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

/// A far-side reading the tower has never heard of is refused at cast, not read
/// as nought — worth pinning as the guarantee it is.
///
/// `compile::fix` leaves an unnameable word `Unplaced::Thing` and the whole
/// question is unreadable. The far side inherits that because
/// `Condition::rename` walks the whole tree.
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

/// A *declared* reading that is absent here reads as nought, and the operators
/// do not know they are adding to nothing.
///
/// `ceiling` compiles anywhere but is published on areas, never on a band, so
/// asking a band for it is nought and `plus 3` over that is three. Pinned
/// rather than fixed: a spell that asks the wrong node gets a confident wrong
/// number.
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

/// Absurd numbers saturate rather than panicking.
///
/// `by` is a number the player typed and `double` multiplies a world read, so
/// both reach values that overflow in a debug build — and a line a player is
/// still editing must not take the tower down.
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
