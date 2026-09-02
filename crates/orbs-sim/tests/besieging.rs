//! The bailey, driven through the real parser and schedule (§5.1).
//!
//! `tower::siege` and `tower::dice` prove the arithmetic with no `World` at all.
//! These prove the *game*: that the words resolve where they should, that the
//! readings a decision tree asks for are actually published, that spending the
//! arsenal reaches the dice, and that a siege survives being saved.

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
/// **Not every answer is a `Message`.** `purge` reports through `State` and
/// `Detail` with no message at all, so `ever_said` cannot see it — which read
/// exactly like the repair not working. Second time this file has been caught by
/// a helper that looks at one field; `readings` was the first.
fn ever_stated(sim: &Sim, wanted: &str) -> bool {
    sim.scrollback().records().iter().any(|record| {
        matches!(record.field(FieldName::State), Some(Value::Text(text)) if text == wanted)
    })
}

/// Every `name: qty` reading `survey` has printed, in order.
///
/// **The quantity arrives as `Value::Text`, not `Value::Count`.** A `survey`
/// answer is an `Entry` whose fields have already been rendered for a column, so
/// matching on `Count` finds nothing and reads exactly like the reading never
/// being published — which cost four of these tests a wrong diagnosis.
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
/// **The third helper in this file, and the third field.** `said` reads
/// `Message`, `readings` reads `Name` *plus* `Quantity`, and a bare reading like
/// `moot` or `d20` carries only a `Name` — so neither of the first two can see
/// one, and both report it as *not published* rather than *not looked for*.
/// Every time this file has been wrong about the game it has been a helper
/// reading one field.
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
/// **Asked rather than assumed.** These tests hardcoded `holding.spell`, and the
/// dice allocation shifted the `Siege` stream enough to change which spell the
/// enemy reached — so a repair test began purging something that was never
/// broken. What is under test is the *loop*, not the name.
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
    // `Verb::anchor` scopes them to the rampart, which is what stops `help` in
    // the laboratory offering a word that can only refuse — §19's debt that
    // `follow` and `wander` were waiting on.
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
    // **Telegraphed intent** — the Into the Breach borrow. The enemy declares
    // before it acts, which turns the player's turn into prevention rather than
    // reaction, and is what makes the arsenal a decision rather than a reflex.
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
    // **The whole accessibility position.** §10 wants outcome to follow what the
    // player chooses given readable state, *never how fast they act* — so the
    // world must not advance while they are reading the board. This is that,
    // asserted: the round counter does not move, however long you take.
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
    // **§19's *"a domain stands alone"*, and this domain needs it most.** A
    // siege exists to test the automation, so holding the tower-wide slot would
    // freeze the thing the enemy is supposed to attack. A grind must still work
    // while a siege is being fought.
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
    // The maze's pattern: the world publishes a derived word and the spell asks
    // for it. *"If the enemy count is twice that of defenders"* is not
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
    // **The rule the sanctum recorded and the satchel repeated.**
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

#[test]
fn spending_the_arsenal_reaches_the_line() {
    // Step 4's whole claim: potions, scrolls and troops are pure mathematical
    // advantages applied on your turn, and this is what retires the ten shipped
    // prose lines saying *"a siege will be what spends them"*.
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
    // §6 forbids a bare error, and a player who typed `quaff troop` is one word
    // from correct. The split is §19's: a scroll keeps `wield`, a troop is
    // deployed, and drinking has its own word.
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
    // **Rule 4 doing its job.** The same record is the transcript line, the §14
    // utterance and what `sift` finds — which is what makes `peruse bailey.log`
    // a genuine postmortem rather than a list of outcomes, and what lets a
    // player see whether a potion earned its place.
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
    // Rule 3, end to end and through `submit` rather than over the model. This
    // is the genre's own regression pattern and the reason the dice have their
    // own stream.
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
    // **The reason `RngStream::Siege` exists.** Sharing `Threat` was the
    // tempting shortcut and the worst option: every combat roll would shift the
    // ambient sabotage schedule, moving the seeds `scripts/play.sh` chose and
    // invalidating every replay with a siege in it.
    //
    // Two towers on one seed, one of which fights: the tampering the *other*
    // sees must be identical.
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

    // **Logs and shelves only.** A siege *is* expected to corrupt spells — that
    // is §5.1's adversarial half and it draws from `Siege` — so the claim here
    // is narrower than "the two towers report the same thing": it is that the
    // **ambient** schedule, which draws from `Threat`, is untouched.
    //
    // This assertion was written before the adversarial surfaces existed and
    // compared the whole list, so step 7 broke it by working. The fix is to say
    // what was always meant rather than to widen the tolerance.
    // **The names, parsed off the sentence rather than filtered inside it.** The
    // first version split the whole line on `", "` and dropped any chunk ending
    // in `spell` — which took the *prefix* with it whenever the first name was a
    // spell, so the two sides were compared with one of them missing its opening
    // clause. Take the list, then filter the list.
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

/// **The premise, asserted.** *"Sieges then test everything you automated —
/// because the enemy attacks the automation."* Six domains produce; this is the
/// one thing that consumes, and until step 7 the last clause was unbuilt.
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
    // **`AUDIT_LONGEST`, not a guessed number.** The claim here is a *negative*,
    // so a wait shorter than the audit passes it with no audit having happened —
    // and `ticks_for` grows with the shelf, which a debug build stocks with every
    // dev spell. The neighbouring test waits 80 and would have gone the same way.
    sim.step_n(orbs_sim::tower::audit::AUDIT_LONGEST + 1);
    let lines = said(&sim);
    // The control. Without it the filter below can be empty because nothing was
    // corrupted *or* because nothing ever answered, and the assertion cannot
    // tell those apart — which is the whole failure mode of asserting a negative.
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

    // ...and a purge puts it back, which is what makes this a loop rather than
    // a report. **Whichever spell the audit named**, not a hardcoded one: the
    // dice allocation shifted the `Siege` stream and the enemy started reaching
    // a different spell, so a fixed name began purging something that was never
    // broken.
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

/// **Misdirection, never theft** — `substitute`'s rule, one surface over.
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

    // What it said before anything touched it — **the peruse output alone**, not
    // the whole scrollback, which is what made this compare a spell's lines
    // against `attend bailey`.
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

    // **Every line back, and none of them wearing the substitution sigil.**
    // A repair that cleared the mark and left the text is the defect
    // `triage::purge` records; a repair that lost a line would be theft.
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

/// **A sabotaged spell's own words survive a save**, or the repair loop is a lie.
///
/// The corruption travelled and the truth did not: the spell reloaded corrupt,
/// still `Poisoned`, and `purge` cleared the mark, said `cleansed` and repaired
/// nothing. That is exactly the defect `triage::purge`'s two repair lines exist
/// to prevent, arriving through the save instead.
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

/// **A save from before a siege closes one that is running.**
///
/// `adopt::apply` adopts onto the *live* world, so it has to remove what the
/// document does not have — its own doc says so for `Stock` and `Substituted`.
/// Without it the bailey reopened mid-fight against a siege the save had never
/// heard of, with a stale `clear_at` gating `defend`.
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

/// **A scroll spent on the wall reaches the wall.**
///
/// §19 keeps `wield` for scrolls rather than giving them a fourth verb, so the
/// same word has to mean *set this going* in two places. Without the
/// interception the three scroll rows in `siege.toml` were dead content:
/// `wield quickening-scroll` in the bailey hurried the *laboratory*, and `quaff`
/// refused and pointed the player at it.
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

/// **...and everything outside a siege is untouched**, which is the half that
/// could regress silently. The interception is narrow on purpose: only while a
/// siege is running, and only for a scroll the wall can actually use.
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

/// **Every row of `siege.toml` is spendable and does something observable.**
///
/// The matrix that would have caught the scrolls: three rows named `wield`, and
/// nothing routed `wield` to the siege — so they were authored, tested for
/// *shape* by `content::siege`'s own tests, and dead in the game. A table test
/// that only reads the table cannot see that.
///
/// This spends each entry through the **real verb** its row names and requires
/// the world to answer.
#[test]
fn every_authored_arsenal_row_can_actually_be_spent() {
    let spendables = orbs_sim::content::Spendables::builtin();
    let mut dead = Vec::new();

    for name in spendables.names() {
        let verb = spendables
            .verb_for(name)
            .expect("content::siege pins every row to a verb");

        // **A worn line, not a fresh one.** A heal spent at full strength is
        // refused now, so a fixture that opens a siege and drinks immediately
        // measures the refusal rather than the potion.
        let mut sim = at_the_wall(11);
        run(&mut sim, "defend");
        run(&mut sim, "hold");
        run(&mut sim, "hold");
        run(&mut sim, &format!("debug_spawn {name} 1"));
        // **Messages before and after, never record indices.** `said` filters to
        // the records that carry a message, so a count of *records* does not
        // index into it — which is how this first read back an empty list and
        // reported every row as dead.
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

/// **The cadence holds, and it is the number the whole economy rests on.**
///
/// Without it `defend` is free and `orbs-balance` measured 4.70 experience a
/// tick against clarity's 0.140 — thirty-three times the flagship.
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

/// **A potion that would do nothing is kept, not drunk.**
///
/// Not an error — §6's bare error is a different thing — but the silent loss of
/// the one resource the domain exists to make you weigh, and the next siege
/// arrives on `CADENCE` whether or not you have anything left.
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

/// **Replay: the same seed and the same submissions reach the same siege.**
///
/// Rule 3 through `Sim::replay` rather than by re-typing, which is the genre's
/// own regression pattern and the one the plan named.
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

    // The established idiom — `tests/progression.rs` and `tests/debug_spawn.rs`
    // both replay this way. A submission carries the tick it was made on, so the
    // replay walks the clock forward to meet each one rather than assuming one
    // command a tick.
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

/// **A siege replayed through the refusal, which is the path that changed.**
///
/// Making a pledge refusable changes *how many dice roll* in a round, and a die
/// that never rolls is a draw that never happens — so this is where a
/// determinism defect would land. The existing replay test above never runs the
/// pool dry, so it exercises the path that was always there.
///
/// The script spends the pool down and then keeps pledging into an empty one, so
/// the run contains successes, `Short` refusals and the rounds after them. If the
/// refusal ever drew, or ever skipped a draw it should have taken, the two runs
/// would disagree about every roll from that point.
///
/// **Six rounds, where three used to do.** A resolved round now grants
/// `REGEN_PER_ROUND` back against a full allocation's eight, so the pool drains
/// six a round rather than eight. The number of rounds here is arithmetic over
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

/// **The pool is a function of the world, not of the stream.**
///
/// Two towers at the same integrity and the same experience open with the same
/// pool, whatever their seeds — which is what makes it safe for `defend` to read
/// it *before* the two opening draws. A pool that varied with the seed would be
/// randomness wearing arithmetic's clothes.
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

/// **A bonus bought for a round the garrison does not swing in is refused.**
///
/// `staged` clears when the round resolves, so a `warding` bought before a
/// volley is spent on rolls that never happen — the same silent loss as a heal
/// at full strength, one intent over.
///
/// **Bodies are not refused**, and that distinction is the point: a volley still
/// hits you. What it takes away is your answer, not their attack. The shipped
/// `answering` solver had that backwards in its first draft and lost a siege the
/// other three won.
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

/// **Three dice against four areas**, so the board can never be covered.
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

    // ...and the round gives it back. **The die is not what is scarce — the
    // round is.**
    run(&mut sim, "hold");
    let mark = words(&sim).len();
    run(&mut sim, "survey coffer");
    let back: Vec<String> = words(&sim).into_iter().skip(mark).collect();
    assert!(
        back.contains(&"d20".to_owned()),
        "the die never came back: {back:?}",
    );
}

/// **A die cannot be pledged twice**, and the refusal says where it went.
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

/// **The range is shown before the commitment** — §5.1's fairness rule carried
/// from a roll to an allocation.
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

/// **The one mistake the domain warns about, before it is made.**
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

/// **A succour actually heals a wounded line**, which `mend`'s old cap made
/// impossible: it capped at `count * VIGOUR`, and `wound` derives `count` back
/// down from `vigour`, so the cap *was* the band's current strength.
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

/// **Every die the pool holds has a node, and every node has a price.**
///
/// The bailey's dice and areas are hardcoded `Branch` names in `tower::build`,
/// and `publish` reaches them with `let Some(node) = … else { continue }` — so a
/// die added to [`siege::POOL`] without a matching branch is **pledgeable,
/// nameable, listed by `readings()`, and prices at nought**, with the whole
/// suite green. A spell asking `if the coffer has fewer quintessence than the
/// d12` would then read its cost as absent, which is zero, and answer that it
/// can afford anything.
///
/// This is the shape `every_material_has_a_home_a_move_can_reach` and
/// `every_self_anchored_verb_is_declared_by_a_fixture` already forbid elsewhere.
/// The bailey shipped without the equivalent; this is it.
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
