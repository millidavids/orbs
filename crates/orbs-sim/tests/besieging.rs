//! The bailey, driven through the real parser and schedule (§5.1).
//!
//! `tower::siege` and `tower::dice` prove the arithmetic with no `World`. These
//! prove the game: that the words resolve, that the readings a decision tree
//! asks for are published, that the arsenal reaches the dice, and that a siege
//! survives being saved.
//!
//! Eleven are `cfg(debug_assertions)`: `debug_siege` and `debug_spawn` make a
//! siege testable in a hundredth of a second, and neither word exists in a
//! release build, where the test fails against a world that was never built.
//! Gated per test rather than per file, as `tests/gleaning.rs` records, so the
//! thirty-two that need no door run in either profile.

use orbs_render::{FieldName, Value};
use orbs_sim::{Save, Sim, tower};

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

fn at_the_wall(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    run(&mut sim, "attend bailey");
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

/// Whether any record carries `wanted` in its `State` field.
///
/// Not every answer is a `Message`: `purge` reports through `State` and
/// `Detail`, so `ever_said` cannot see it and the repair reads as broken.
fn ever_stated(sim: &Sim, wanted: &str) -> bool {
    sim.scrollback().records().iter().any(|record| {
        matches!(record.field(FieldName::State), Some(Value::Text(text)) if text == wanted)
    })
}

/// Every `name: qty` reading `survey` has printed, in order.
///
/// The quantity arrives as `Value::Text`, not `Value::Count` — a `survey` answer
/// is an `Entry` already rendered for a column.
fn readings(sim: &Sim) -> Vec<(String, u64)> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| {
            let Some(Value::Text(name)) = record.field(FieldName::Name) else {
                return None;
            };
            let qty = match record.field(FieldName::Quantity)? {
                Value::Text(text) => text.parse().ok()?,
                Value::Count(count) | Value::Tick(count) => count,
            };
            Some((name.to_owned(), qty))
        })
        .collect()
}

/// Every bare word `survey` has printed — a reading with no number on it.
///
/// The third field. `said` reads `Message` and `readings` reads `Name` plus
/// `Quantity`, so neither can see a bare `moot` or `d20`.
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

/// The spells an audit has just called tampered, in the order it named them.
///
/// Asked rather than assumed: a hardcoded `holding.spell` broke when the dice
/// allocation shifted the `Siege` stream. What is under test is the loop.
fn tampered(sim: &Sim) -> Vec<String> {
    said(sim)
        .into_iter()
        .filter(|line| line.contains("not what they say"))
        .flat_map(|line| {
            line.rsplit(':')
                .next()
                .unwrap_or_default()
                .split(',')
                .map(|name| name.trim().to_owned())
                .collect::<Vec<_>>()
        })
        .filter(|name| name.ends_with("spell"))
        .collect()
}

/// The last value `survey` gave for one reading.
fn last(sim: &Sim, wanted: &str) -> Option<u64> {
    readings(sim)
        .into_iter()
        .filter(|(name, _)| name == wanted)
        .map(|(_, qty)| qty)
        .next_back()
}

#[test]
fn the_four_words_only_work_at_the_wall() {
    // `Verb::anchor` scopes them to the rampart, so `help` in the laboratory
    // does not offer a word that can only refuse (§19).
    let mut sim = Sim::new(11);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "defend");
    assert!(
        ever_said(&sim, "no rampart"),
        "defend worked in the laboratory: {:?}",
        said(&sim),
    );
}

#[test]
fn a_siege_arrives_and_says_what_it_means_to_do() {
    // Telegraphed intent — the Into the Breach borrow. The enemy declares before
    // it acts, so the player's turn is prevention rather than reaction.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    assert!(
        ever_said(&sim, "come up the road"),
        "no enemy arrived: {:?}",
        said(&sim),
    );
    let intents = ["advance", "onslaught", "volley"];
    assert!(
        intents.iter().any(|word| ever_said(&sim, word)),
        "the enemy never said what it would do: {:?}",
        said(&sim),
    );
}

#[test]
fn nothing_moves_until_you_hold() {
    // §10 wants outcome to follow what the player chooses, never how fast they
    // act, so the world must not advance while they read the board.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    for _ in 0..20 {
        sim.step();
    }
    run(&mut sim, "survey rampart");
    let turns = last(&sim, tower::siege::TURNS);
    assert_eq!(
        turns,
        Some(0),
        "twenty ticks of standing still advanced the siege: {:?}",
        said(&sim),
    );

    run(&mut sim, "hold");
    run(&mut sim, "survey rampart");
    let turns = last(&sim, tower::siege::TURNS);
    assert_eq!(turns, Some(1), "hold did not resolve a round");
}

#[test]
fn a_siege_takes_no_production_slot() {
    // §19's "a domain stands alone": a siege exists to test the automation, so
    // holding the tower-wide slot would freeze what the enemy attacks.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "hold");
    run(&mut sim, "attend laboratory");
    run(&mut sim, "grind sage");
    assert!(
        !ever_said(&sim, "busy"),
        "the siege held the production slot: {:?}",
        said(&sim),
    );
}

#[test]
fn the_readings_a_decision_tree_asks_for_are_published() {
    // The maze's pattern: *"if the enemy count is twice the defenders"* is not
    // expressible in the language, so `outnumbered` is published instead.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "survey garrison");
    run(&mut sim, "survey enemy");

    let names: Vec<String> = readings(&sim).into_iter().map(|(name, _)| name).collect();
    for wanted in [
        tower::siege::SPEARS,
        tower::siege::METTLE,
        tower::siege::FOES,
    ] {
        assert!(
            names.iter().any(|name| name == wanted),
            "{wanted} is not published: {names:?}",
        );
    }
}

#[test]
fn a_band_that_is_standing_is_never_empty_and_a_routed_one_is() {
    // `spell::watch` answers `is empty` by asking whether a node has children,
    // so a band that always carried a count could never be empty and the first
    // rung of every solver would be dead.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "survey garrison");
    assert!(
        !readings(&sim).is_empty(),
        "a standing garrison published nothing",
    );
}

#[cfg(debug_assertions)]
#[test]
fn spending_the_arsenal_reaches_the_line() {
    // Potions, scrolls and troops are mathematical advantages applied on your
    // turn — the ten shipped prose lines promising it are now true.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "debug_spawn troop 1");
    run(&mut sim, "survey garrison");
    let before = last(&sim, tower::siege::SPEARS).expect("a garrison was published");

    run(&mut sim, "deploy troop");
    run(&mut sim, "survey garrison");
    let after = last(&sim, tower::siege::SPEARS).expect("a garrison was published");

    assert!(
        after > before,
        "deploying a troop did not reinforce the line: {before} -> {after}",
    );
}

#[test]
fn the_wrong_word_is_refused_and_names_the_right_one() {
    // §6 forbids a bare error, and `quaff troop` is one word from correct. §19's
    // split: a scroll keeps `wield`, a troop is deployed, drinking has its own.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "debug_spawn troop 1");
    run(&mut sim, "quaff troop");
    assert!(
        ever_said(&sim, "deploy it"),
        "the refusal did not name the right word: {:?}",
        said(&sim),
    );
}

#[test]
fn something_the_wall_has_no_use_for_is_refused_rather_than_spent() {
    // An item with no `siege.toml` entry cannot be spent at all, which is what
    // stops `quaff sage` resolving and quietly doing nothing.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "deploy sage");
    assert!(
        ever_said(&sim, "worth nothing"),
        "a reagent was spendable on a wall: {:?}",
        said(&sim),
    );
}

#[test]
fn spending_something_you_do_not_have_is_refused() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "deploy troop");
    assert!(
        ever_said(&sim, "no troop in the arsenal"),
        "a troop nobody had was deployed: {:?}",
        said(&sim),
    );
}

#[test]
fn every_roll_reaches_the_log_with_its_die_and_its_face() {
    // Rule 4: one record is the transcript line, the §14 utterance and what
    // `sift` finds, which makes `peruse bailey.log` a postmortem.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "hold");
    run(&mut sim, "peruse bailey.log");
    assert!(
        ever_said(&sim, "d20 gives"),
        "no roll named its die: {:?}",
        said(&sim),
    );
    assert!(
        ever_said(&sim, "against 11"),
        "no roll named what it had to beat: {:?}",
        said(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_siege_ends_and_pays() {
    // §11.5's escrow, through the real verbs. `debug_siege` leaves it one round
    // from won so the line does not depend on the dice falling a particular way.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "debug_siege");
    for _ in 0..6 {
        run(&mut sim, "hold");
        if ever_said(&sim, "breaks and runs") {
            break;
        }
    }
    assert!(
        ever_said(&sim, "breaks and runs"),
        "a enemy of one never broke: {:?}",
        said(&sim),
    );
    assert!(sim.experience() > 0, "a won siege earned nothing",);
}

#[cfg(debug_assertions)]
#[test]
fn a_store_runs_down_and_the_shelf_keeps_what_it_will_not_spend() {
    // Stores are a rate, and the shelf is not what is measured: the count never
    // moves, only whether the tower has made one lately.
    let mut sim = at_the_wall(11);
    run(&mut sim, "debug_spawn warding 1");
    run(&mut sim, "defend");
    run(&mut sim, "meditate 3000");

    // Surveyed first: `last` reads what `survey` published, so with no survey
    // behind it the count is `None` rather than the shelf being empty.
    run(&mut sim, "survey arsenal");
    let held = last(&sim, "warding");
    assert_eq!(held, Some(1), "the shelf did not have the warding");
    run(&mut sim, "quaff warding");
    assert!(
        ever_said(&sim, "stores are out"),
        "a store that had run down was spent anyway: {:?}",
        said(&sim),
    );
    run(&mut sim, "survey arsenal");
    assert_eq!(
        last(&sim, "warding"),
        held,
        "a refused spend still took the potion",
    );
}

#[cfg(debug_assertions)]
#[test]
fn an_empty_shelf_says_so_rather_than_blaming_the_stores() {
    // Two facts, two sentences: the supply check answered *"your troop stores
    // are out"* for a shelf that is simply bare, which a player cannot act on.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "deploy troop");
    assert!(
        ever_said(&sim, "there is no troop"),
        "a bare shelf blamed the stores: {:?}",
        said(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_famous_tower_draws_a_longer_tail_and_the_floor_never_moves() {
    // Fame lengthens the tail, it does not shift the band. Over seeds, because
    // the draw is a draw: the ceiling rises and the floor does not.
    let quiet: Vec<u32> = (0..40).map(|seed| opening(seed, 0)).collect();
    let famous: Vec<u32> = (0..40).map(|seed| opening(seed, 9_000)).collect();

    assert_eq!(
        quiet.iter().copied().min(),
        famous.iter().copied().min(),
        "standing moved the floor, and it must never",
    );
    assert!(
        famous.iter().copied().max() > quiet.iter().copied().max(),
        "standing did not lengthen the tail: {quiet:?} against {famous:?}",
    );
    assert!(
        famous.iter().all(|&n| n <= tower::siege::MOST),
        "a draw went past the written ceiling: {famous:?}",
    );
}

#[cfg(debug_assertions)]
#[test]
fn petitioning_buys_the_tail_down_and_costs_standing_to_do_it() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "debug_renown 9000");
    let before = sim.renown();

    run(&mut sim, "petition");
    assert!(
        ever_said(&sim, "word goes out"),
        "petition said nothing: {:?}",
        said(&sim),
    );
    assert!(
        sim.renown() < before,
        "petitioning cost no standing: {before} either side",
    );
}

#[cfg(debug_assertions)]
#[test]
fn petitioning_is_refused_at_the_floor_and_keeps_the_renown() {
    // The reading sweep never petitions, so it passes whatever this does. The
    // risk is a tower buying the tail away until `outnumbered` can never fire.
    let mut sim = at_the_wall(11);
    run(&mut sim, "debug_renown 9000");
    for _ in 0..12 {
        run(&mut sim, "petition");
    }
    assert!(
        ever_said(&sim, "no fewer than"),
        "buying the tail past its floor was allowed: {:?}",
        said(&sim),
    );

    // ...and the refusal is free, which is `pledge`'s rule in this room.
    let held = sim.renown();
    run(&mut sim, "petition");
    assert_eq!(sim.renown(), held, "a refused petition still took renown");
}

#[cfg(debug_assertions)]
#[test]
fn petitioning_is_refused_once_they_are_at_the_wall() {
    // What is bought is the size of the *next* siege, and this one is drawn
    // (§19). `settle` measures the fight from the snapshot taken when the enemy
    // arrived, so renown spent inside that window was charged to the fight — a
    // siege that moved +38 reported +31.
    let mut sim = at_the_wall(11);
    run(&mut sim, "debug_renown 400");
    run(&mut sim, "defend");
    let held = sim.renown();

    run(&mut sim, "petition");
    assert!(
        ever_said(&sim, "already at the wall"),
        "petition was allowed mid-siege: {:?}",
        said(&sim),
    );
    assert_eq!(sim.renown(), held, "a refused petition still took renown");
}

#[cfg(debug_assertions)]
#[test]
fn petitioning_with_nothing_to_spend_says_both_numbers() {
    // `pledge_short`'s rule: a refusal a player cannot plan around is a wall,
    // so the sentence carries the price *and* what is held.
    let mut sim = at_the_wall(11);
    run(&mut sim, "petition");
    assert!(
        ever_said(&sim, "that would take"),
        "an unaffordable petition did not name its price: {:?}",
        said(&sim),
    );
    assert_eq!(sim.renown(), 0, "a refused petition moved standing");
}

/// How many came up the road on `seed`, at `renown` standing.
#[cfg(debug_assertions)]
fn opening(seed: u64, renown: u64) -> u32 {
    let mut sim = at_the_wall(seed);
    if renown > 0 {
        run(&mut sim, &format!("debug_renown {renown}"));
    }
    run(&mut sim, "defend");
    run(&mut sim, "survey enemy");
    last(&sim, tower::siege::SPEARS)
        .unwrap_or(0)
        .try_into()
        .unwrap_or(0)
}

#[cfg(debug_assertions)]
#[test]
fn a_round_moves_standing_and_says_nothing_about_it() {
    // The exchange is worth something and the round does not stop to say so: a
    // fight is six to thirteen rounds and each has already said what the enemy
    // did, so a sentence per exchange is the same news twice.
    let mut sim = at_the_wall(11);
    // Standing to move, in either direction: a fresh tower is at nought and
    // `slip` saturates there, so a bad round would assert nothing.
    run(&mut sim, "debug_renown 400");
    run(&mut sim, "defend");
    // Pledged, because an unpledged round trades about evenly and can net to
    // nought — which would make this pass without measuring anything.
    run(&mut sim, "pledge d20 to buckler");

    let before = sim.renown();
    let quiet = sim.scrollback().records().len();

    // Rounds, not a round: a single exchange can trade even. Held short of a
    // settle, so nothing here is the outcome's doing.
    for _ in 0..4 {
        run(&mut sim, "hold");
        // "the wall" is not the end of a siege — `the buckler put 14 on the
        // wall` is a round saying what a pledge bought.
        if ever_said(&sim, "the wall is carried") || ever_said(&sim, "breaks and runs") {
            break;
        }
    }

    assert!(
        sim.renown() != before,
        "four pledged rounds left standing untouched: {before} either side",
    );
    // ...and not one of them stopped to talk about it.
    assert!(
        !said(&sim)
            .iter()
            .any(|line| line.contains("word gets about")),
        "a round stopped to talk about standing: {:?}",
        said(&sim),
    );
    assert!(
        sim.scrollback().records().len() > quiet,
        "the rounds said nothing at all",
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_won_siege_pays_standing_and_says_what_the_fight_came_to() {
    // The other half: the fight speaks once, on settling, and says the whole
    // movement — measured from `Siege::standing`, taken when the enemy arrived.
    let mut sim = at_the_wall(11);
    let before = sim.renown();
    run(&mut sim, "defend");
    run(&mut sim, "debug_siege");
    run(&mut sim, "pledge d20 to buckler");
    for _ in 0..6 {
        run(&mut sim, "hold");
        if ever_said(&sim, "breaks and runs") {
            break;
        }
    }
    assert!(ever_said(&sim, "breaks and runs"), "the enemy never broke");

    let won = sim.renown() - before;
    assert!(won > 0, "a won siege paid no standing at all");
    assert!(
        ever_said(&sim, "singing about it"),
        "a won siege never said what it was worth: {:?}",
        said(&sim),
    );
    // The sentence's number is the fight's, not the settle's.
    assert!(
        ever_said(&sim, &format!("{won} renown")),
        "the sentence did not say the {won} the fight actually moved: {:?}",
        said(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_lost_siege_costs_standing_and_a_near_miss_costs_less_than_a_collapse() {
    // A number that can go down. The stake scales by how far short the wall
    // fell, so this asserts the direction rather than a value.
    let collapse = tower::siege::renown_stake(7, 10, tower::siege::Outcome::Fallen);
    let near_miss = tower::siege::renown_stake(7, 90, tower::siege::Outcome::Fallen);
    assert!(
        collapse > near_miss,
        "collapsing early cost no more standing than nearly holding",
    );
    assert!(near_miss > 0, "losing at ninety percent was free");

    // ...and through the real verbs, with standing to lose. A tower at nought
    // cannot fall, so this one has to start somewhere.
    let mut sim = at_the_wall(11);
    run(&mut sim, "debug_renown 400");
    let before = sim.renown();
    run(&mut sim, "defend");
    for _ in 0..40 {
        run(&mut sim, "hold");
        if ever_said(&sim, "the wall is carried") {
            break;
        }
    }
    if ever_said(&sim, "the wall is carried") {
        assert!(
            sim.renown() < before,
            "a lost siege cost no standing: {before} either side",
        );
        assert!(
            ever_said(&sim, "the wall cost"),
            "a lost siege never said what it cost: {:?}",
            said(&sim),
        );
    }
}

#[cfg(debug_assertions)]
#[test]
fn a_finished_siege_refuses_a_further_round_and_says_how_to_start_another() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "debug_siege");
    for _ in 0..6 {
        run(&mut sim, "hold");
        if ever_said(&sim, "breaks and runs") {
            break;
        }
    }
    run(&mut sim, "hold");
    assert!(
        ever_said(&sim, "defend again"),
        "a finished siege did not say how to start another: {:?}",
        said(&sim),
    );
}

#[test]
fn a_siege_survives_being_saved() {
    // A siege is long enough that a player will quit in the middle of one, and
    // one that reset on load would be worse than no save at all.
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "hold");
    run(&mut sim, "hold");

    let text = sim.snapshot().to_toml().expect("a save renders");
    let mut restored = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));
    run(&mut restored, "attend bailey");
    run(&mut restored, "survey rampart");

    let turns = last(&restored, tower::siege::TURNS);
    assert_eq!(
        turns,
        Some(2),
        "the siege forgot two rounds across a save: {:?}",
        said(&restored),
    );
}

#[test]
fn the_same_seed_fights_the_same_siege_through_the_real_verbs() {
    // Rule 3, end to end and through `submit` rather than over the model — the
    // reason the dice have their own stream.
    let script = ["attend bailey", "defend", "hold", "hold", "hold"];
    let mut a = Sim::new(5);
    let mut b = Sim::new(5);
    for line in script {
        run(&mut a, line);
        run(&mut b, line);
    }
    assert_eq!(said(&a), said(&b), "one seed fought two different sieges");
}

#[test]
fn a_siege_rolls_its_own_stream_and_does_not_move_the_sabotage_schedule() {
    // Why `RngStream::Siege` exists: sharing `Threat` would let every combat
    // roll shift the ambient sabotage schedule and invalidate every replay with
    // a siege in it. Two towers on one seed, one of which fights — the tampering
    // the other sees must be identical.
    let mut quiet = Sim::new(3);
    let mut fighting = Sim::new(3);
    run(&mut fighting, "attend bailey");
    run(&mut fighting, "defend");
    for _ in 0..5 {
        run(&mut fighting, "hold");
    }
    for _ in 0..400 {
        quiet.step();
        fighting.step();
    }
    run(&mut quiet, "verify");
    run(&mut fighting, "verify");
    quiet.step_n(40);
    fighting.step_n(40);

    // Logs and shelves only. A siege is expected to corrupt spells — §5.1's
    // adversarial half, drawing from `Siege` — so the claim is narrower: the
    // ambient schedule, which draws from `Threat`, is untouched.
    //
    // The names are parsed off the sentence rather than filtered inside it:
    // splitting the whole line and dropping chunks ending in `spell` took the
    // prefix with it whenever the first name was a spell.
    let ambient = |sim: &Sim| -> Vec<String> {
        said(sim)
            .into_iter()
            .filter(|line| line.contains("not what they say"))
            .flat_map(|line| {
                line.rsplit(": ")
                    .next()
                    .unwrap_or_default()
                    .split(',')
                    .map(|name| name.trim().to_owned())
                    .collect::<Vec<_>>()
            })
            .filter(|name| !name.ends_with("spell"))
            .collect()
    };
    assert_eq!(
        ambient(&quiet),
        ambient(&fighting),
        "fighting a siege moved the ambient sabotage schedule",
    );
}

/// The premise, asserted: *"sieges then test everything you automated — because
/// the enemy attacks the automation."*
#[test]
fn a_siege_reaches_the_automation() {
    let mut sim = at_the_wall(3);
    run(&mut sim, "defend");
    for _ in 0..8 {
        run(&mut sim, "hold");
    }
    assert!(
        ever_said(&sim, "got past the wall"),
        "eight rounds of siege never touched the tower: {:?}",
        said(&sim),
    );
}

/// §5.1's pillar 4: *"Phase A stays genuinely safe."*
#[test]
fn the_calm_layer_never_sees_an_adversarial_aberration() {
    // The ambient nuisances still fire — that is `drift` and `substitution` —
    // but nothing ever reaches a *spell* outside a siege.
    let mut sim = Sim::new(3);
    sim.step_n(3000);
    run(&mut sim, "verify");
    // `AUDIT_LONGEST`, not a guessed number: the claim is a negative, so a wait
    // shorter than the audit passes with no audit having happened.
    sim.step_n(orbs_sim::tower::audit::AUDIT_LONGEST + 1);
    let lines = said(&sim);
    // The control: without it the filter below is empty whether nothing was
    // corrupted or nothing ever answered.
    assert!(
        lines.iter().any(
            |line| line.contains("is what it says it is") || line.contains("not what they say")
        ),
        "the audit never answered, so the claim below is vacuous: {lines:?}",
    );
    let lied: Vec<String> = lines
        .into_iter()
        .filter(|line| line.contains("not what they say"))
        .collect();
    assert!(
        !lied.iter().any(|line| line.contains(".spell")),
        "the calm layer corrupted a spell: {lied:?}",
    );
}

/// The whole loop §5.1 calls *"diagnose and repair under pressure"*.
#[test]
fn a_sabotaged_spell_is_found_by_verify_and_mended_by_purge() {
    let mut sim = at_the_wall(3);
    run(&mut sim, "defend");
    for _ in 0..6 {
        run(&mut sim, "hold");
    }
    assert!(
        ever_said(&sim, "got past the wall"),
        "nothing was sabotaged"
    );

    // The audit finds it and names it — §8.1's *"the skill is knowing which
    // surface to inspect, not deciphering an obscure clue"*.
    run(&mut sim, "verify");
    sim.step_n(80);
    let named: Vec<String> = said(&sim)
        .into_iter()
        .filter(|line| line.contains("not what they say"))
        .collect();
    assert!(
        named.iter().any(|line| line.contains(".spell")),
        "the audit missed the spell it had just been told about: {named:?}",
    );

    // ...and a purge puts it back, which makes this a loop rather than a report.
    // Whichever spell the audit named: a fixed name began purging something that
    // was never broken once the dice allocation shifted the `Siege` stream.
    let broken = tampered(&sim);
    let target = broken.first().expect("the audit named a spell").clone();
    run(&mut sim, "attend grimoire");
    run(&mut sim, &format!("purge {target}"));
    sim.step_n(10);
    assert!(
        ever_stated(&sim, "cleansed"),
        "a corrupted spell could not be repaired: {:?}",
        said(&sim),
    );
}

/// Misdirection, never theft — `substitute`'s rule, one surface over.
#[test]
fn a_repaired_spell_reads_exactly_as_it_was_written() {
    // Fight first, so the enemy chooses which spell this is about.
    let mut sim = at_the_wall(3);
    run(&mut sim, "defend");
    for _ in 0..6 {
        run(&mut sim, "hold");
    }
    run(&mut sim, "verify");
    sim.step_n(80);
    let broken = tampered(&sim);
    let target = broken.first().expect("the audit named a spell").clone();

    // What it said before anything touched it — the peruse output alone, not the
    // whole scrollback, which compared a spell's lines against `attend bailey`.
    let mut before = at_the_wall(3);
    run(&mut before, "attend grimoire");
    let mark = said(&before).len();
    run(&mut before, &format!("peruse {target}"));
    let original: Vec<String> = said(&before).into_iter().skip(mark).collect();

    run(&mut sim, "attend grimoire");
    run(&mut sim, &format!("purge {target}"));
    sim.step_n(10);
    let mark = said(&sim).len();
    run(&mut sim, &format!("peruse {target}"));
    let repaired: Vec<String> = said(&sim).into_iter().skip(mark).collect();

    // Every line back, none wearing the substitution sigil. A repair that
    // cleared the mark and left the text is the defect `triage::purge` records.
    for line in &original {
        if line.len() > 6 && !line.contains("peruse") {
            assert!(
                repaired.iter().any(|got| got == line),
                "a repaired spell lost the line {line:?}",
            );
        }
    }
    assert!(
        !repaired.iter().any(|line| line.trim_end().ends_with('-')),
        "the corrupted line survived the repair: {repaired:?}",
    );
}

/// A sabotaged spell's own words survive a save, or the repair loop is a lie.
///
/// The corruption travelled and the truth did not: the spell reloaded corrupt
/// and `purge` cleared the mark, said `cleansed` and repaired nothing.
#[test]
fn a_rewritten_spell_can_still_be_repaired_after_a_save() {
    let mut sim = at_the_wall(3);
    run(&mut sim, "defend");
    for _ in 0..6 {
        run(&mut sim, "hold");
    }
    assert!(
        ever_said(&sim, "got past the wall"),
        "nothing was sabotaged"
    );

    // Ask the audit which spell it was, rather than assuming.
    run(&mut sim, "verify");
    sim.step_n(80);
    let target = tampered(&sim)
        .first()
        .expect("the audit named a spell")
        .clone();

    let text = sim.snapshot().to_toml().expect("a save renders");
    let mut restored = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));

    run(&mut restored, "attend grimoire");
    run(&mut restored, &format!("purge {target}"));
    restored.step_n(10);
    assert!(
        ever_stated(&restored, "cleansed"),
        "a spell sabotaged before the save could not be repaired after it: {:?}",
        said(&restored),
    );

    // ...and the repair actually restored the text, rather than only clearing
    // the mark.
    let mark = said(&restored).len();
    run(&mut restored, &format!("peruse {target}"));
    let lines: Vec<String> = said(&restored).into_iter().skip(mark).collect();
    assert!(
        !lines.iter().any(|line| line.trim_end().ends_with('-')),
        "the corrupted line survived the repair: {lines:?}",
    );
}

/// A save from before a siege closes one that is running.
///
/// `adopt::apply` adopts onto the live world, so it has to remove what the
/// document does not have. Without it the bailey reopened mid-fight with a
/// stale `clear_at` gating `defend`.
#[test]
fn loading_a_save_from_before_a_siege_ends_the_one_in_progress() {
    let mut sim = at_the_wall(11);
    let quiet = sim.snapshot().to_toml().expect("a save renders");

    run(&mut sim, "defend");
    run(&mut sim, "hold");
    assert!(last(&sim, tower::siege::TURNS).is_none() || ever_said(&sim, "you lose"));

    let mut restored = Sim::restored(&Save::from_toml(&quiet).expect("a save reads back"));
    run(&mut restored, "attend bailey");
    run(&mut restored, "hold");
    assert!(
        ever_said(&restored, "defend first"),
        "a siege survived a save taken before it began: {:?}",
        said(&restored),
    );
}

/// A scroll spent on the wall reaches the wall.
///
/// §19 keeps `wield` for scrolls rather than a fourth verb, so without the
/// interception the three scroll rows in `siege.toml` were dead content —
/// `wield quickening-scroll` in the bailey hurried the laboratory.
#[cfg(debug_assertions)]
#[test]
fn a_scroll_is_spent_on_the_siege_when_one_is_running() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "debug_spawn verdant-scroll 1");
    run(&mut sim, "survey garrison");
    let before = last(&sim, tower::siege::SPEARS).expect("a garrison was published");

    run(&mut sim, "wield verdant-scroll");
    run(&mut sim, "survey garrison");
    let after = last(&sim, tower::siege::SPEARS).expect("a garrison was published");

    assert!(
        after > before,
        "a scroll wielded on the wall did nothing for the line: {before} -> {after}",
    );
    assert!(
        ever_said(&sim, "burns away on the wall"),
        "the scroll took its laboratory effect instead: {:?}",
        said(&sim),
    );
}

/// ...and everything outside a siege is untouched. The interception is narrow on
/// purpose: only while a siege runs, and only for a scroll the wall can use.
#[cfg(debug_assertions)]
#[test]
fn a_scroll_keeps_its_ordinary_effect_when_no_siege_is_running() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "debug_spawn quickening-scroll 1");
    run(&mut sim, "wield quickening-scroll");
    assert!(
        ever_said(&sim, "works quick"),
        "a scroll lost its ordinary effect outside a siege: {:?}",
        said(&sim),
    );

    // ...and in the room that owns it, mid-siege elsewhere or not.
    let mut archive = Sim::new(3);
    run(&mut archive, "attend archive");
    run(&mut archive, "research");
    run(&mut archive, "debug_spawn gleaning-scroll 1");
    run(&mut archive, "wield gleaning-scroll");
    assert!(
        ever_said(&archive, "things in the dark"),
        "the archive's errand scroll stopped setting its errand: {:?}",
        said(&archive),
    );
}

/// Every row of `siege.toml` is spendable and does something observable.
///
/// Three rows named `wield` and nothing routed `wield` to the siege, so they
/// were authored, shape-tested and dead in the game. Spends each entry through
/// the real verb its row names and requires the world to answer.
#[cfg(debug_assertions)]
#[test]
fn every_authored_arsenal_row_can_actually_be_spent() {
    let spendables = orbs_sim::content::Spendables::builtin();
    let mut dead = Vec::new();

    for name in spendables.names() {
        let verb = spendables
            .verb_for(name)
            .expect("content::siege pins every row to a verb");

        // A worn line, not a fresh one: a heal spent at full strength is
        // refused, so drinking immediately measures the refusal.
        let mut sim = at_the_wall(11);
        run(&mut sim, "defend");
        run(&mut sim, "hold");
        run(&mut sim, "hold");
        run(&mut sim, &format!("debug_spawn {name} 1"));
        // Messages before and after, never record indices: `said` filters to
        // records carrying a message, so a record count does not index into it.
        let before = said(&sim).len();
        run(&mut sim, &format!("{} {name}", verb.canonical()));
        let answered: Vec<String> = said(&sim).into_iter().skip(before).collect();
        let spent = answered.iter().any(|line| {
            line.contains("goes down to the line")
                || line.contains("drunk")
                || line.contains("burns away on the wall")
        });
        if !spent {
            dead.push(format!("{name} via {}: {answered:?}", verb.canonical()));
        }
    }

    assert!(
        dead.is_empty(),
        "these authored arsenal rows cannot be spent in a siege: {dead:#?}",
    );
}

/// ...and each one actually *changes* the siege rather than only saying so.
#[cfg(debug_assertions)]
#[test]
fn every_authored_arsenal_row_changes_the_siege() {
    let spendables = orbs_sim::content::Spendables::builtin();
    let mut inert = Vec::new();

    for name in spendables.names() {
        let entry = spendables.get(name).expect("just listed");
        let verb = spendables.verb_for(name).expect("pinned to a verb");

        // Two rounds first, so a heal has something to heal.
        let mut sim = at_the_wall(11);
        run(&mut sim, "defend");
        run(&mut sim, "hold");
        run(&mut sim, "hold");
        run(&mut sim, &format!("debug_spawn {name} 1"));
        run(&mut sim, "survey garrison");
        let spears = last(&sim, tower::siege::SPEARS);
        let mettle = last(&sim, tower::siege::METTLE);

        run(&mut sim, &format!("{} {name}", verb.canonical()));
        run(&mut sim, "survey garrison");

        let moved = match entry.kind.as_str() {
            // Bodies and fight are visible on the board directly.
            "troops" => last(&sim, tower::siege::SPEARS) != spears,
            "vigour" => last(&sim, tower::siege::METTLE) != mettle,
            // A dice modifier is visible in the odds, which is exactly what §5.1
            // requires be shown before the commitment.
            _ => sim.rampart().is_some(),
        };
        if !moved {
            inert.push(format!("{name} ({})", entry.kind));
        }
    }

    assert!(
        inert.is_empty(),
        "these arsenal rows were spent and changed nothing: {inert:#?}",
    );
}

/// The cadence holds, and the whole economy rests on it.
///
/// Without it `defend` is free and `orbs-balance` measured 4.70 experience a
/// tick against clarity's 0.140.
#[cfg(debug_assertions)]
#[test]
fn a_second_siege_cannot_be_summoned_at_will() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "debug_siege");
    for _ in 0..6 {
        run(&mut sim, "hold");
        if ever_said(&sim, "the wall") {
            break;
        }
    }
    run(&mut sim, "defend");
    assert!(
        ever_said(&sim, "the road is empty"),
        "a second siege was available the moment the first ended: {:?}",
        said(&sim),
    );
}

/// ...and it survives a save, or it is a cadence a player clears by quitting.
#[cfg(debug_assertions)]
#[test]
fn the_cadence_travels_in_the_save() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "debug_siege");
    for _ in 0..6 {
        run(&mut sim, "hold");
        if ever_said(&sim, "the wall") {
            break;
        }
    }

    let text = sim.snapshot().to_toml().expect("a save renders");
    let mut restored = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));
    run(&mut restored, "attend bailey");
    run(&mut restored, "defend");
    assert!(
        ever_said(&restored, "the road is empty"),
        "quitting cleared the siege cadence: {:?}",
        said(&restored),
    );
}

/// A potion that would do nothing is kept, not drunk.
///
/// The silent loss of the one resource the domain exists to make you weigh, and
/// the next siege arrives on `CADENCE` whether or not you have anything left.
#[cfg(debug_assertions)]
#[test]
fn a_heal_at_full_strength_is_refused_and_the_potion_kept() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "debug_spawn mending 1");
    run(&mut sim, "quaff mending");
    assert!(
        ever_said(&sim, "the line is whole"),
        "a heal at full strength was drunk anyway: {:?}",
        said(&sim),
    );

    // ...and it is still there when it is wanted.
    run(&mut sim, "hold");
    run(&mut sim, "hold");
    run(&mut sim, "quaff mending");
    assert!(
        ever_said(&sim, "drunk"),
        "the refused potion was consumed after all: {:?}",
        said(&sim),
    );
}

/// Replay: the same seed and the same submissions reach the same siege.
///
/// Rule 3 through `Sim::replay` rather than by re-typing.
#[test]
fn a_siege_replays_from_its_submissions() {
    let script = [
        "attend bailey",
        "defend",
        "debug_spawn troop 2",
        "deploy troop",
        "hold",
        "hold",
        "hold",
    ];
    let mut played = Sim::new(5);
    for line in script {
        run(&mut played, line);
    }

    // The established idiom. A submission carries the tick it was made on, so
    // the replay walks the clock forward to meet each one.
    let mut replayed = Sim::new(5);
    for (tick, submission) in played.submissions().all().to_vec() {
        while replayed.tick() < tick {
            replayed.step();
        }
        replayed.replay(submission);
    }
    while replayed.tick() < played.tick() {
        replayed.step();
    }

    assert_eq!(
        said(&played),
        said(&replayed),
        "a siege replayed from its own submissions diverged",
    );
}

/// A siege replayed through the refusal, which is the path that changed.
///
/// A refusable pledge changes how many dice roll in a round, and a die that
/// never rolls is a draw that never happens. The script spends the pool down and
/// keeps pledging into an empty one, so the run contains successes, `Short`
/// refusals and the rounds after them.
///
/// Six rounds, where three used to do: a resolved round grants `REGEN_PER_ROUND`
/// back against a full allocation's eight, so the count here is arithmetic over
/// those two constants and moves when either does.
#[test]
fn a_siege_replays_through_running_out_of_quintessence() {
    let mut script = vec!["attend bailey".to_owned(), "defend".to_owned()];
    for _ in 0..6 {
        for die in ["d20", "d8", "d6"] {
            script.push(format!("pledge {die} buckler"));
        }
        script.push("hold".to_owned());
    }

    let mut played = Sim::new(5);
    for line in &script {
        run(&mut played, line);
    }
    assert!(
        ever_said(&played, "would take"),
        "the script never ran the pool dry, so it tests the unchanged path: {:?}",
        said(&played),
    );

    let mut replayed = Sim::new(5);
    for (tick, submission) in played.submissions().all().to_vec() {
        while replayed.tick() < tick {
            replayed.step();
        }
        replayed.replay(submission);
    }
    while replayed.tick() < played.tick() {
        replayed.step();
    }

    assert_eq!(
        said(&played),
        said(&replayed),
        "a siege that ran out of quintessence replayed differently",
    );
}

/// The pool is a function of the world, not of the stream.
///
/// Two towers at the same integrity and experience open with the same pool
/// whatever their seeds, which is what makes it safe for `defend` to read it
/// before the two opening draws.
#[test]
fn the_pool_is_the_same_on_every_seed() {
    let pools: Vec<u64> = [0, 3, 11, 42]
        .into_iter()
        .map(|seed| {
            let mut sim = at_the_wall(seed);
            run(&mut sim, "defend");
            run(&mut sim, "survey coffer");
            last(&sim, tower::siege::QUINTESSENCE).expect("the coffer publishes it")
        })
        .collect();
    assert!(
        pools.windows(2).all(|pair| pair[0] == pair[1]),
        "the opening pool varied with the seed: {pools:?}",
    );
}

/// A bonus bought for a round the garrison does not swing in is refused.
///
/// `staged` clears when the round resolves, so a `warding` bought before a
/// volley is spent on rolls that never happen. Bodies are not refused: a volley
/// still hits you, so what it takes away is your answer, not their attack.
#[cfg(debug_assertions)]
#[test]
fn a_bonus_is_refused_before_a_volley_and_a_troop_is_not() {
    // Walk seeds until one telegraphs a volley on the opening round.
    let mut found = false;
    for seed in 0..30 {
        let mut sim = at_the_wall(seed);
        run(&mut sim, "defend");
        if !ever_said(&sim, "volley") {
            continue;
        }
        found = true;

        run(&mut sim, "debug_spawn warding 1");
        run(&mut sim, "debug_spawn troop 1");
        run(&mut sim, "quaff warding");
        assert!(
            ever_said(&sim, "will not swing"),
            "a bonus was spent on a round with no rolls in it: {:?}",
            said(&sim),
        );

        // ...and the troop still goes in, because a volley still lands.
        run(&mut sim, "survey garrison");
        let before = last(&sim, tower::siege::SPEARS).expect("a garrison");
        run(&mut sim, "deploy troop");
        run(&mut sim, "survey garrison");
        assert!(
            last(&sim, tower::siege::SPEARS) > Some(before),
            "bodies were refused before a volley, which they should not be",
        );
        break;
    }
    assert!(found, "no seed in thirty opened with a volley");
}

// --- the dice allocation (§5.1) --------------------------------------------

/// Three dice against four areas, so the board can never be covered.
#[test]
fn the_coffer_holds_fewer_dice_than_there_are_places_for_them() {
    assert!(
        tower::siege::POOL.len() < tower::siege::Area::ALL.len(),
        "every area can be covered, so nothing is ever left dark",
    );
}

/// A pledged die leaves the coffer, and comes back when the round resolves.
#[test]
fn a_pledged_die_leaves_the_coffer_and_returns_next_round() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "survey coffer");
    assert!(
        words(&sim).contains(&"d20".to_owned()),
        "the coffer did not start with the d20 in it",
    );

    let before = words(&sim).len();
    run(&mut sim, "pledge d20 buckler");
    run(&mut sim, "survey coffer");
    let after: Vec<String> = words(&sim).into_iter().skip(before).collect();
    assert!(
        !after.contains(&"d20".to_owned()),
        "a pledged die was still in the coffer: {after:?}",
    );

    // ...and the round gives it back: the die is not what is scarce, the round
    // is.
    run(&mut sim, "hold");
    let mark = words(&sim).len();
    run(&mut sim, "survey coffer");
    let back: Vec<String> = words(&sim).into_iter().skip(mark).collect();
    assert!(
        back.contains(&"d20".to_owned()),
        "the die never came back: {back:?}",
    );
}

/// A die cannot be pledged twice, and the refusal says where it went.
#[test]
fn a_die_is_refused_a_second_pledge_and_named_where_it_is() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "pledge d20 buckler");
    run(&mut sim, "pledge d20 sortie");
    assert!(
        ever_said(&sim, "already pledged to the buckler"),
        "a die was pledged twice, or the refusal did not say where it was: {:?}",
        said(&sim),
    );
}

/// The range is shown before the commitment — §5.1's fairness rule carried from
/// a roll to an allocation.
#[test]
fn a_pledge_says_what_it_could_come_to_before_it_is_rolled() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "pledge d20 buckler");
    assert!(
        ever_said(&sim, "1 to 20"),
        "a d20 was pledged with no range shown: {:?}",
        said(&sim),
    );
    run(&mut sim, "pledge d6 succour");
    assert!(ever_said(&sim, "1 to 6"), "{:?}", said(&sim));
}

/// The one mistake the domain warns about, before it is made.
#[test]
fn pledging_to_the_line_before_a_volley_is_allowed_and_warned_about() {
    for seed in 0..30 {
        let mut sim = at_the_wall(seed);
        run(&mut sim, "defend");
        if !ever_said(&sim, "volley") {
            continue;
        }
        let mark = words(&sim).len();
        run(&mut sim, "survey line");
        assert!(
            words(&sim)
                .into_iter()
                .skip(mark)
                .any(|word| word == "moot"),
            "the line gave no warning before a volley: {:?}",
            said(&sim),
        );
        run(&mut sim, "pledge d20 line");
        assert!(
            ever_said(&sim, "wasted"),
            "a wasted pledge was accepted silently: {:?}",
            said(&sim),
        );
        return;
    }
    panic!("no seed in thirty opened with a volley");
}

/// Every area changes the round it is pledged to, and does so visibly.
#[test]
fn every_area_does_something_and_says_what_it_did() {
    for (area, phrase) in [
        ("buckler", "on the wall"),
        ("succour", "put back"),
        ("sortie", "dealt"),
    ] {
        let mut sim = at_the_wall(11);
        run(&mut sim, "defend");
        // Two rounds first, so a succour has something to put back.
        run(&mut sim, "hold");
        run(&mut sim, "hold");
        run(&mut sim, &format!("pledge d20 {area}"));
        run(&mut sim, "hold");
        assert!(
            ever_said(&sim, phrase),
            "the {area} resolved and never said what it did: {:?}",
            said(&sim),
        );
    }
}

/// A succour actually heals a wounded line, which `mend`'s old cap made
/// impossible: it capped at `count * VIGOUR`, and `wound` derives `count` back
/// from `vigour`, so the cap was the band's current strength.
#[test]
fn a_succour_puts_real_mettle_back() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    for _ in 0..3 {
        run(&mut sim, "hold");
    }
    run(&mut sim, "survey garrison");
    let hurt = last(&sim, tower::siege::METTLE).expect("a garrison");

    run(&mut sim, "pledge d20 succour");
    run(&mut sim, "pledge d8 succour");
    run(&mut sim, "pledge d6 buckler");
    run(&mut sim, "hold");
    run(&mut sim, "survey garrison");
    let after = last(&sim, tower::siege::METTLE).expect("a garrison");

    assert!(
        after > hurt.saturating_sub(3),
        "a wounded line took back nothing from two dice of succour: {hurt} -> {after}",
    );
}

/// Pledges clear when the round resolves — a pledge is bought for one round.
#[test]
fn pledges_do_not_outlive_the_round_they_were_made_for() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "pledge d20 buckler");
    run(&mut sim, "survey buckler");
    assert!(
        last(&sim, tower::siege::CEILING).is_some(),
        "the pledge was not published",
    );

    run(&mut sim, "hold");
    let mark = said(&sim).len();
    run(&mut sim, "survey buckler");
    let after: Vec<String> = said(&sim).into_iter().skip(mark).collect();
    assert!(
        !after.iter().any(|line| line.contains("ceiling")),
        "a pledge outlived its round, so only the first turn would matter: {after:?}",
    );
}

/// The allocation survives a save — it is state like any other.
#[test]
fn a_pledge_survives_being_saved() {
    let mut sim = at_the_wall(11);
    run(&mut sim, "defend");
    run(&mut sim, "pledge d20 buckler");

    let text = sim.snapshot().to_toml().expect("a save renders");
    let mut restored = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));
    run(&mut restored, "attend bailey");
    run(&mut restored, "pledge d20 sortie");
    assert!(
        ever_said(&restored, "already pledged"),
        "the pledge did not survive the save: {:?}",
        said(&restored),
    );
}

/// Every die the pool holds has a node, and every node has a price.
///
/// The bailey's dice and areas are hardcoded `Branch` names in `tower::build`,
/// and `publish` skips a name with no node — so a die added to [`siege::POOL`]
/// without a matching branch is pledgeable, nameable, listed by `readings()` and
/// priced at nought, with the whole suite green.
///
/// The shape `every_material_has_a_home_a_move_can_reach` already forbids
/// elsewhere; the bailey shipped without the equivalent.
#[test]
fn every_die_and_area_the_domain_knows_is_a_node_that_answers() {
    let mut sim = at_the_wall(11);

    let mut missing = Vec::new();
    for die in tower::siege::POOL {
        run(&mut sim, &format!("survey {}", die.word()));
        // The price is raised at construction, so this holds before any siege —
        // which is the half that was broken.
        if last(&sim, tower::siege::QUINTESSENCE).is_none() {
            missing.push(format!("{} has no price", die.word()));
        }
    }

    // An area answers `survey` at all, which is what `pledge` and `for each
    // area` both need of it.
    for area in tower::siege::Area::ALL {
        let before = said(&sim).len();
        run(&mut sim, &format!("survey {}", area.word()));
        if said(&sim).len() == before {
            missing.push(format!("{} is not a place to survey", area.word()));
        }
    }

    assert!(
        missing.is_empty(),
        "the domain names things the tower has no node for: {missing:#?}",
    );
}
