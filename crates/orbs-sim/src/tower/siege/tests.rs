//! The siege's model, proved with no `World` at all (§5.1).
//!
//! **One file for the whole module, because most of these cross its seams.**
//! `every_state_a_siege_reaches_obeys_every_rule` walks a real siege and asserts
//! band invariants, reading arithmetic and completion together; splitting it per
//! sibling would mean four copies of the same loop. `tests/besieging.rs` proves
//! the *game* — the words, the schedule, the save.

use super::*;
use crate::rng::Rngs;
use crate::tower::dice::{Die, Effect, Roll};

fn rngs() -> Rngs {
    Rngs::from_seed(11)
}

/// A pool deep enough that quintessence never binds.
///
/// **Most of what is below is about bands, intents and rolls**, and a pool that
/// ran dry mid-test would make those fail for a reason they are not about. The
/// tests that *are* about the pool ask for a specific one, so the number here is
/// a way of saying "not the subject" rather than a balance figure.
const AMPLE: u32 = 999;

#[test]
fn a_siege_starts_with_the_contingent_the_king_assigns() {
    let siege = Siege::begin(&mut rngs(), AMPLE);
    assert_eq!(siege.garrison.count, ASSIGNED, "the baseline is not static");
    assert_eq!(siege.garrison.vigour, ASSIGNED * VIGOUR);
    assert!(siege.running());
    assert_eq!(siege.turns, 0);
}

#[test]
fn the_baseline_is_the_same_every_time_and_the_enemy_is_not() {
    // The design rests on this: *"the wizard's base troops will be static"*,
    // so the variables a player weighs are the enemy, the arsenal, and the
    // dice — never a contingent that quietly changed.
    let mut enemies = std::collections::BTreeSet::new();
    for seed in 0..40 {
        let siege = Siege::begin(&mut Rngs::from_seed(seed), AMPLE);
        assert_eq!(siege.garrison.count, ASSIGNED);
        enemies.insert(siege.enemy.count);
    }
    assert!(enemies.len() > 1, "every enemy was identical: {enemies:?}");
}

#[test]
fn a_round_advances_the_clock_and_nothing_else_does() {
    let mut rngs = rngs();
    let mut siege = Siege::begin(&mut rngs, AMPLE);
    // Staging is free and moves no round — §5.0's "no per-command tick
    // cost", which is what lets a player think.
    siege.stage("mending", Effect::Bonus(2));
    siege.reinforce(1);
    siege.heal(3);
    assert_eq!(siege.turns, 0, "acting on your turn advanced the round");
    siege.resolve(&mut rngs);
    assert_eq!(siege.turns, 1);
}

#[test]
fn a_volley_draws_the_garrisons_dice_and_throws_them_away() {
    // **The determinism trap this domain is most likely to fall into.** The
    // garrison does not strike back on a volley, and the obvious way to
    // write that is to skip its rolls — which would make the *number of
    // draws* depend on the intent, silently reordering every later roll for
    // any seed where a volley came up. §19 records the same shape in
    // `drift`. So the rolls are drawn, then discarded.
    //
    // A volley and an advance genuinely consume *different* counts, because
    // a volley sends fewer attackers — an earlier version of this test
    // asserted they matched, which was a claim about the wrong thing.
    let mut siege = Siege::begin(&mut rngs(), AMPLE);
    siege.intent = Intent::Volley;

    let attackers = Intent::Volley.attackers(siege.enemy.count);
    let defenders = siege.garrison.count;

    let mut fought = rngs();
    let round = siege.resolve(&mut fought);
    assert!(round.answered.is_empty(), "a volley was answered");
    assert_eq!(round.dealt, 0, "a volley cost the enemy something");

    // The same stream, advanced by hand through every draw the round should
    // have taken: each attacker, each defender (drawn and discarded), and
    // the intent for the next round.
    let mut counted = rngs();
    for _ in 0..(attackers + defenders) {
        Roll::new(Die::D20, AGAINST).resolve(&mut counted);
    }
    let _ = Intent::drawn(&mut counted);

    assert_eq!(
        Roll::new(Die::D20, AGAINST).resolve(&mut fought),
        Roll::new(Die::D20, AGAINST).resolve(&mut counted),
        "an unanswered volley consumed {attackers} + {defenders} draws plus \
         an intent, or the garrison's dice were skipped rather than discarded",
    );
}

#[test]
fn staged_modifiers_are_spent_on_the_round_they_were_bought_for() {
    let mut rngs = rngs();
    let mut siege = Siege::begin(&mut rngs, AMPLE);
    siege.stage("mending", Effect::Bonus(5));
    assert_eq!(siege.garrison_roll().bonus(), 5);
    siege.resolve(&mut rngs);
    assert_eq!(
        siege.garrison_roll().bonus(),
        0,
        "a staged modifier outlived its round, so only turn one would matter",
    );
}

#[test]
fn the_readings_say_what_the_numbers_say() {
    let mut siege = Siege::begin(&mut rngs(), AMPLE);
    siege.garrison = Band::new(6);
    siege.enemy = Band::new(6);
    assert!(!siege.few() && !siege.hurt() && !siege.outnumbered());

    siege.garrison = Band::new(2);
    assert!(siege.few(), "two troops is not few");
    assert!(siege.outnumbered(), "six against two is not outnumbered");

    siege.garrison = Band::new(4);
    siege.garrison.vigour = 8; // half of 4 * 3
    assert!(siege.hurt(), "a band at half vigour is not hurt");
}

#[test]
fn a_routed_band_reads_as_neither_few_nor_hurt() {
    // A broken side is not a *thin* side, and a decision tree that treated
    // them alike would keep pouring potions into nobody.
    let mut siege = Siege::begin(&mut rngs(), AMPLE);
    siege.garrison = Band::new(0);
    assert!(siege.garrison.routed());
    assert!(!siege.few());
    assert!(!siege.hurt());
    assert!(!siege.outnumbered());
}

#[test]
fn wounding_drops_a_troop_only_when_its_vigour_is_gone() {
    let mut band = Band::new(3); // 9 vigour
    band.wound(1);
    assert_eq!(band.count, 3, "one hit felled a whole troop");
    band.wound(2);
    assert_eq!((band.count, band.vigour), (2, 6));
    band.wound(6);
    assert!(band.routed());
}

#[test]
fn mending_never_raises_a_band_above_full() {
    let mut band = Band::new(3);
    band.wound(4);
    band.mend(100, 3 * VIGOUR);
    assert_eq!(band.vigour, 3 * VIGOUR, "a potion overfilled a band");
    assert_eq!(band.count, 3, "troops did not come back with the fight");
}

/// **A wounded band can actually be healed**, which it could not be.
///
/// `mend` capped at `count * VIGOUR` and `wound` derives `count` down from
/// `vigour`, so the cap *was* the band's current strength: a line at 14 of
/// 18 could take back two points and no more, whatever it was given. It made
/// `succour` nearly inert and the `mending` potion a third of what it read
/// as — and the arsenal matrix test passed throughout, because the number
/// did move, by one.
#[test]
fn a_wounded_band_takes_back_what_it_is_given() {
    let mustered = 6;
    let ceiling = mustered * VIGOUR;
    let mut band = Band::new(mustered);
    band.wound(9);
    assert_eq!(band.vigour, 9);

    band.mend(6, ceiling);
    assert_eq!(band.vigour, 15, "a wounded band could not take back six");
    assert_eq!(band.count, 5, "the troops did not come back with the fight");

    // ...and still never past what it was mustered at.
    band.mend(100, ceiling);
    assert_eq!(band.vigour, ceiling);
    assert_eq!(band.count, mustered);
}

#[test]
fn every_siege_ends_and_says_how() {
    // A siege that could run for ever is a faucet that never closes, which
    // is this domain's version of the failure mode `scrying` names.
    for seed in 0..60 {
        let mut rngs = Rngs::from_seed(seed);
        let mut siege = Siege::begin(&mut rngs, AMPLE);
        let mut rounds = 0;
        while siege.running() && rounds < 200 {
            siege.resolve(&mut rngs);
            rounds += 1;
        }
        assert!(
            !siege.running(),
            "seed {seed} never ended after {rounds} rounds",
        );
        assert!(siege.outcome.is_some());
    }
}

#[test]
fn completion_is_what_the_enemy_lost() {
    let mut siege = Siege::begin(&mut rngs(), AMPLE);
    siege.arrived = 8;
    siege.enemy = Band::new(8);
    assert_eq!(siege.completion(), 0);
    siege.enemy = Band::new(4);
    assert_eq!(siege.completion(), 50);
    siege.enemy = Band::new(0);
    assert_eq!(siege.completion(), 100);
}

#[test]
fn the_same_seed_fights_the_same_siege() {
    // Rule 3, end to end — the genre's own regression pattern.
    let mut ra = Rngs::from_seed(7);
    let mut rb = Rngs::from_seed(7);
    let mut a = Siege::begin(&mut ra, AMPLE);
    let mut b = Siege::begin(&mut rb, AMPLE);
    while a.running() {
        assert_eq!(a.resolve(&mut ra), b.resolve(&mut rb));
    }
    assert_eq!(a, b);
}

#[test]
fn the_odds_are_knowable_before_the_commitment() {
    // §5.1's fairness rule as an assertion: a player can always read the
    // chance before choosing, because `enemy_roll` composes without drawing.
    let siege = Siege::begin(&mut rngs(), AMPLE);
    assert!(siege.enemy_roll().chance() > 0);
    assert!(siege.garrison_roll().chance() > 0);
    // ...and a staged bonus visibly moves it, or the arsenal is decoration.
    let mut better = siege.clone();
    better.stage("mending", Effect::Bonus(4));
    assert!(
        better.garrison_roll().chance() > siege.garrison_roll().chance(),
        "a staged bonus did not improve the stated odds",
    );
}

#[test]
fn an_onslaught_is_worse_than_an_advance_and_a_volley_is_thinner() {
    let full = 8;
    assert_eq!(Intent::Advance.attackers(full), 8);
    assert_eq!(Intent::Onslaught.attackers(full), 8);
    assert_eq!(Intent::Volley.attackers(full), 4);
    assert!(Intent::Onslaught.edge() > Intent::Advance.edge());
    assert!(Intent::Advance.answered());
    assert!(!Intent::Volley.answered());
}

#[test]
fn a_lost_siege_still_pays_and_a_won_one_pays_more() {
    // §11.5's table, and the sentence under it: *"effort is never wasted;
    // only cynicism is."*
    let pool = 8 * ESCROW_PER_FOE;
    assert_eq!(escrow(8, 100, Outcome::Held), pool + pool / 2);
    // Lost at 60% keeps 60%.
    assert_eq!(escrow(8, 60, Outcome::Fallen), (pool * 60) / 100);
    // ...and bailing at nought still pays the floor, which is the whole
    // point of having one.
    assert_eq!(escrow(8, 0, Outcome::Fallen), (pool * ESCROW_FLOOR) / 100);
    // A win always beats a loss at the same completion.
    for completion in [0, 25, 60, 99, 100] {
        assert!(
            escrow(8, completion, Outcome::Held) > escrow(8, completion, Outcome::Fallen),
            "losing at {completion}% paid at least as well as winning",
        );
    }
}

#[test]
fn a_bigger_enemy_is_worth_more_so_abandoning_a_hard_one_is_never_the_play() {
    // The exploit the scaling exists to close: if the pool were flat, the
    // best move would be to abandon anything difficult and wait.
    assert!(escrow(9, 100, Outcome::Held) > escrow(5, 100, Outcome::Held));
    assert!(escrow(9, 50, Outcome::Fallen) > escrow(5, 50, Outcome::Fallen));
}

/// **Every state a siege can actually reach, checked against every rule.**
///
/// The `hurt` defect survived a green suite because the unit test that
/// covered it hand-built a band the game cannot produce — `Band::new(4)`
/// with `vigour = 8`, where the minimum at four troops is ten. A fixture
/// that constructs an impossible world proves nothing about the real one.
///
/// So this walks real sieges from real seeds and asserts the invariants at
/// **every intermediate state**, which is the only way a rule about worn
/// bands can be checked without inventing one.
#[test]
fn every_state_a_siege_reaches_obeys_every_rule() {
    for seed in 0..60 {
        let mut rngs = Rngs::from_seed(seed);
        let mut siege = Siege::begin(&mut rngs, AMPLE);
        let mut rounds = 0;

        loop {
            let where_ = format!("seed {seed}, round {rounds}: {siege:?}");

            // A band's count is derived from its vigour and can never
            // outrun it.
            for band in [siege.garrison, siege.enemy] {
                assert!(
                    band.count <= band.vigour.div_ceil(VIGOUR),
                    "a band has more standing than it has fight — {where_}",
                );
                assert_eq!(
                    band.count == 0,
                    band.vigour == 0,
                    "a band is standing with no fight, or has fight and nobody — {where_}",
                );
            }

            // Never mended above what it has ever had.
            assert!(
                siege.garrison.vigour <= siege.mustered * VIGOUR,
                "the garrison is stronger than it has ever been — {where_}",
            );
            assert!(
                siege.enemy.count <= siege.arrived,
                "the enemy grew — {where_}",
            );

            // The readings agree with the numbers they describe.
            assert_eq!(
                siege.few(),
                siege.garrison.count > 0 && siege.garrison.count <= THIN,
                "few disagrees with the count — {where_}",
            );
            assert_eq!(
                siege.outnumbered(),
                siege.garrison.count > 0 && siege.enemy.count >= siege.garrison.count * 2,
                "outnumbered disagrees with the counts — {where_}",
            );
            // **A routed side is neither few nor hurt**, or a decision tree
            // keeps pouring potions into nobody.
            if siege.garrison.routed() {
                assert!(
                    !siege.few() && !siege.hurt(),
                    "a broken line reads as thin — {where_}"
                );
            }

            assert!(
                siege.completion() <= 100,
                "completion ran past whole — {where_}"
            );

            if !siege.running() {
                break;
            }
            assert!(rounds < 400, "seed {seed} never ended");
            siege.resolve(&mut rngs);
            rounds += 1;
        }
    }
}

/// **`hurt` is reachable while the line is still standing**, which is the
/// whole of what the defect broke: measured against the *current* count it
/// implied `count < 3`, where `few` also fires and shadows it in an
/// `else if` ladder — so the `quaff` rung was dead and `siege.toml`'s
/// healing entries were unreachable.
#[test]
fn a_worn_but_standing_garrison_reads_hurt_without_reading_few() {
    let mut found = false;
    for seed in 0..60 {
        let mut rngs = Rngs::from_seed(seed);
        let mut siege = Siege::begin(&mut rngs, AMPLE);
        while siege.running() {
            if siege.hurt() && !siege.few() {
                assert!(
                    siege.garrison.count > THIN,
                    "the case is only reachable above the thin line",
                );
                found = true;
            }
            siege.resolve(&mut rngs);
        }
    }
    assert!(
        found,
        "no siege in sixty seeds ever read hurt without also reading few — \
         the quaff rung of every solver is dead code",
    );
}

/// **The decision, in one test: spending is finite and declining is free.**
#[test]
fn pledging_spends_the_pool_and_declining_costs_nothing() {
    let mut siege = Siege::begin(&mut rngs(), 6);
    assert_eq!(siege.quintessence, 6);

    // The d20 costs five of six.
    assert_eq!(
        siege.pledge(Die::D20, Area::Buckler),
        Pledged::Made { cost: 5 },
    );
    assert_eq!(siege.quintessence, 1);

    // ...which leaves the d8 unaffordable and the d6 just within reach.
    assert_eq!(
        siege.pledge(Die::D8, Area::Line),
        Pledged::Short { cost: 2 }
    );
    assert_eq!(
        siege.quintessence, 1,
        "a refused pledge took something anyway",
    );
    assert_eq!(
        siege.pledge(Die::D6, Area::Succour),
        Pledged::Made { cost: 1 }
    );
    assert_eq!(siege.quintessence, 0);
}

/// A die already behind something and a die you cannot pay for are different
/// answers — collapsing them would make the refusal say the wrong thing.
#[test]
fn a_spent_die_and_an_unaffordable_one_refuse_differently() {
    let mut siege = Siege::begin(&mut rngs(), AMPLE);
    assert!(matches!(
        siege.pledge(Die::D6, Area::Line),
        Pledged::Made { .. }
    ));
    assert_eq!(siege.pledge(Die::D6, Area::Buckler), Pledged::Spent);

    let mut broke = Siege::begin(&mut rngs(), 0);
    assert_eq!(
        broke.pledge(Die::D6, Area::Line),
        Pledged::Short { cost: 1 }
    );
}

/// **The pool survives the round; the pledges do not.** §11.5's *no
/// regeneration* — the dice come back and what paid for them does not.
#[test]
fn the_dice_come_back_each_round_and_the_pool_does_not() {
    let mut rngs = rngs();
    let mut siege = Siege::begin(&mut rngs, 12);
    assert!(matches!(
        siege.pledge(Die::D20, Area::Buckler),
        Pledged::Made { .. }
    ));
    assert_eq!(siege.quintessence, 7);

    siege.resolve(&mut rngs);
    assert_eq!(
        siege.coffer().len(),
        POOL.len(),
        "the dice did not come back",
    );
    assert_eq!(
        siege.quintessence, 7,
        "the pool regenerated, which §14 forbids",
    );
}

/// What the coffer offers a spell is what it can actually pay for.
#[test]
fn the_coffer_offers_only_what_can_be_paid_for() {
    let siege = Siege::begin(&mut rngs(), 2);
    assert_eq!(siege.coffer().len(), 3, "all three are still held");
    let afford: Vec<&str> = siege.affordable().iter().map(|d| d.word()).collect();
    assert_eq!(
        afford,
        vec!["d6", "d8"],
        "the d20 is held and unaffordable, and should not be offered",
    );
    assert!(siege.free(Die::D20), "the d20 is still unpledged");
    assert!(!siege.affords(Die::D20));
}

#[test]
fn every_reading_the_domain_publishes_is_named_once() {
    let mut seen = std::collections::BTreeSet::new();
    for word in readings() {
        assert!(seen.insert(word), "{word} is published twice");
    }
    // The intents are readings too — a spell asks `if the enemy has volley`.
    for intent in Intent::ALL {
        assert!(
            readings().contains(&intent.word()),
            "{} is not listed",
            intent.word()
        );
    }
}
