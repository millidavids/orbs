//! The menagerie's circle, driven through the real parser and the real schedule.
//!
//! `tower::circle`'s proofs walk every beast the table can draw and prove the
//! model; these prove the *game* — that the words resolve, the readings publish,
//! a call costs the tower nothing, a hold pays what `progression.toml` says, and
//! the shipped search holds a beast through `invoke` rather than in a Rust copy
//! of its loop.

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;

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

/// Whether anything said so far contains `needle`.
fn ever_said(sim: &Sim, needle: &str) -> bool {
    said(sim).iter().any(|line| line.contains(needle))
}

/// Standing in the menagerie, with nothing done yet.
fn at_the_circle(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    run(&mut sim, "attend menagerie");
    sim
}

/// Where a glyph stands on the board, by the word it is limned with.
fn standing(sim: &Sim, glyph: &str) -> Option<String> {
    sim.circle()?
        .lines
        .into_iter()
        .find(|line| line.glyph == glyph)
        .map(|line| line.humour)
}

#[test]
fn a_beast_arrives_and_the_draw_is_not_a_call() {
    let mut sim = at_the_circle(3);
    run(&mut sim, "summon");

    let Some(board) = sim.circle() else {
        panic!("summon drew no beast: {:?}", said(&sim));
    };
    assert!(board.answer.is_none(), "the draw was counted as a call");
    assert!(ever_said(&sim, "a beast gathers at the circle"));
    // What a spell reads: the temper's fervour on the circle, and every glyph at
    // the opening.
    assert!(sim.holds_reading("menagerie", "circle", "fervour"));
    for glyph in ["keystone", "sunwise", "widdershins"] {
        assert!(
            sim.holds_reading("menagerie", glyph, "yoke"),
            "the {glyph} does not publish its humour",
        );
    }
}

#[test]
fn limn_names_a_humour_or_steps_to_the_next() {
    let mut sim = at_the_circle(3);
    run(&mut sim, "summon");

    run(&mut sim, "limn keystone heed");
    assert_eq!(standing(&sim, "keystone").as_deref(), Some("heed"));
    assert!(sim.holds_reading("menagerie", "keystone", "heed"));
    assert!(!sim.holds_reading("menagerie", "keystone", "yoke"));

    // Bare: the next humour round the six, which is how a spell tries what it
    // cannot name.
    run(&mut sim, "limn keystone");
    assert_eq!(standing(&sim, "keystone").as_deref(), Some("eschew"));
    assert!(ever_said(&sim, "keystone is limned eschew"));

    // **Humour first is the same limn** — no word is both, so a player who
    // thinks of the humour first may type it first.
    run(&mut sim, "limn oppose sunwise");
    assert_eq!(standing(&sim, "sunwise").as_deref(), Some("oppose"));
}

/// **A limn republishes its own glyph and nothing else.** The temper and the other
/// two glyphs have not moved, and a bound search limns hundreds of times a beast:
/// re-raising all four readings each time said three of them again with new ids.
#[test]
fn a_limn_leaves_every_other_reading_where_it_stood() {
    let mut sim = at_the_circle(3);
    run(&mut sim, "summon");
    let children = |sim: &Sim, path: &str| {
        let world = sim.world();
        orbs_sim::tower::find_by_path(world, path)
            .map(|node| orbs_sim::tower::children_of(world, node))
            .unwrap_or_default()
    };
    let circle = children(&sim, "/tower/menagerie/circle");
    let sunwise = children(&sim, "/tower/menagerie/sunwise");
    let keystone = children(&sim, "/tower/menagerie/keystone");
    assert!(
        !circle.is_empty() && !sunwise.is_empty(),
        "nothing published"
    );

    run(&mut sim, "limn keystone heed");
    assert_eq!(children(&sim, "/tower/menagerie/circle"), circle);
    assert_eq!(children(&sim, "/tower/menagerie/sunwise"), sunwise);
    assert_ne!(
        children(&sim, "/tower/menagerie/keystone"),
        keystone,
        "the limned glyph was not republished"
    );
    assert!(sim.holds_reading("menagerie", "keystone", "heed"));
}

#[test]
fn limn_refuses_in_voice_and_names_the_way_forward() {
    let mut sim = at_the_circle(3);
    run(&mut sim, "limn keystone heed");
    assert!(ever_said(&sim, "summon first"), "{:?}", said(&sim));

    run(&mut sim, "summon");
    run(&mut sim, "limn keystone sunwise");
    assert!(ever_said(&sim, "is not a humour"), "{:?}", said(&sim));

    // **A humour the orb cannot read is asked about, never dropped.** Bare
    // `limn keystone` steps the glyph, so dropping the word ran a different
    // command and spoiled the circle a player was reasoning about.
    run(&mut sim, "limn keystone xyzzy");
    assert_eq!(standing(&sim, "keystone").as_deref(), Some("yoke"));
    assert!(!ever_said(&sim, "keystone is limned"), "{:?}", said(&sim));

    // **The word that is wrong is the one named**: a humour first, then a place
    // that is no glyph.
    run(&mut sim, "limn heed laboratory");
    assert!(
        ever_said(&sim, "laboratory is not a glyph"),
        "{:?}",
        said(&sim)
    );
    assert!(!ever_said(&sim, "heed is not a glyph"), "{:?}", said(&sim));
}

#[test]
fn a_wrong_call_balks_and_pays_nothing() {
    let mut sim = at_the_circle(3);
    run(&mut sim, "summon");
    let before = sim.experience();

    // The opening is never a solution — the table refuses any beast the unlimned
    // circle already holds — so this call must balk.
    run(&mut sim, "summon");
    assert!(ever_said(&sim, "the beast balks at"), "{:?}", said(&sim));
    assert_eq!(sim.experience(), before, "a balk paid something");
    assert_eq!(sim.tally("event:figure"), 0);
    let Some(board) = sim.circle() else {
        panic!("a balk sent the beast away");
    };
    assert!(board.answer.is_some(), "the board does not show the answer");
    assert!(!board.balking().is_empty(), "a balk marked no rows");
}

/// **A call is never spent on a guess.** `summon` takes no argument and resolves
/// in any menagerie, so it is what a reader falls back to when a sentence defeats
/// it — *"cycle the widdershins"* came back as a call against par with nothing
/// limned. A divined `summon` still draws a beast, which is free; it does not
/// call one in, and the same word typed does.
#[test]
fn a_divined_summon_draws_but_never_spends_a_call() {
    let mut sim = at_the_circle(3);
    sim.submit_divined("bring me a beast", "summon");
    sim.step();
    assert!(
        sim.circle().is_some(),
        "a divined summon drew nothing: {:?}",
        said(&sim)
    );

    sim.submit_divined("cycle the widdershins", "summon");
    sim.step();
    assert!(ever_said(&sim, "on a guess"), "{:?}", said(&sim));
    let Some(board) = sim.circle() else {
        panic!("a guess sent the beast away");
    };
    assert!(board.answer.is_none(), "a divined summon spent a call");
    assert!(!ever_said(&sim, "the beast balks at"), "{:?}", said(&sim));

    run(&mut sim, "summon");
    assert!(ever_said(&sim, "on call 1"), "{:?}", said(&sim));
}

#[test]
#[cfg(debug_assertions)]
fn a_hold_within_par_pays_four_troops_and_the_whole_price() {
    let mut sim = at_the_circle(3);
    run(&mut sim, "summon");
    let before = sim.experience();
    run(&mut sim, "debug_circle");
    run(&mut sim, "summon");

    assert!(sim.circle().is_none(), "a hold left the beast waiting");
    assert!(
        ever_said(&sim, "the circle holds. 4 troops answer"),
        "{:?}",
        said(&sim),
    );
    assert_eq!(
        sim.experience(),
        before + 8,
        "a hold within par did not pay 8"
    );
    assert_eq!(sim.tally("event:figure"), 1, "the hold was not counted");
    assert_eq!(sim.tally("made:troop"), 1, "the troops were not a making");
    // Held means clear: a spell's `if the circle is working` must go false.
    assert!(!sim.holds_reading("menagerie", "circle", "fervour"));
}

#[test]
#[cfg(debug_assertions)]
fn a_hold_past_par_pays_three_troops_and_three_quarters() {
    let mut sim = at_the_circle(3);
    run(&mut sim, "summon");
    // Four calls on the opening, every one a balk: one past par before the hold.
    for _ in 0..4 {
        run(&mut sim, "summon");
    }
    let before = sim.experience();
    run(&mut sim, "debug_circle");
    run(&mut sim, "summon");

    assert!(
        ever_said(&sim, "the circle holds. 3 troops answer"),
        "{:?}",
        said(&sim),
    );
    assert_eq!(
        sim.experience(),
        before + 6,
        "past par did not pay three quarters"
    );
}

/// **A hold says what reached a shelf.** With no arsenal to keep them, the
/// sentence counted four troops that went nowhere — the record lying about the
/// world, and the player unable to tell the hold had been lost.
#[test]
#[cfg(debug_assertions)]
fn a_hold_with_no_arsenal_does_not_announce_troops() {
    let mut sim = at_the_circle(3);
    let Some(arsenal) = orbs_sim::tower::keep(sim.world_mut()) else {
        panic!("an open tower has no arsenal");
    };
    sim.world_mut()
        .entity_mut(arsenal)
        .remove::<orbs_sim::tower::Keep>();
    run(&mut sim, "summon");
    run(&mut sim, "debug_circle");
    run(&mut sim, "summon");
    assert!(ever_said(&sim, "the circle holds"), "{:?}", said(&sim));
    assert!(ever_said(&sim, "no troops come"), "{:?}", said(&sim));
    assert!(!ever_said(&sim, "troops answer"), "{:?}", said(&sim));
}

#[test]
fn stop_lets_the_beast_go_and_the_circle_agrees() {
    let mut sim = at_the_circle(3);
    run(&mut sim, "summon");
    run(&mut sim, "stop circle");

    assert!(ever_said(&sim, "slips away unheld"), "{:?}", said(&sim));
    assert!(sim.circle().is_none());
    // **Both halves**, which is the disagreement the pylon's arm records: the
    // verb said stopped and the reading said still working.
    assert!(!sim.holds_reading("menagerie", "circle", "fervour"));
    assert!(!ever_said(&sim, "not working"), "{:?}", said(&sim));
}

/// **Neither word takes the production slot** — §19 records this exemption
/// being forgotten for the sanctum. A spell's `summon` queued behind a brew would
/// wait on a slot it never uses.
#[test]
#[cfg(debug_assertions)]
fn summoning_takes_no_production_slot() {
    // The plain half first: neither word schedules anything.
    let mut idle = at_the_circle(3);
    run(&mut idle, "summon");
    run(&mut idle, "limn keystone heed");
    run(&mut idle, "summon");
    assert!(
        idle.working().is_none(),
        "the circle took the tower's one production slot",
    );

    // ...and the half that bit the sanctum: a spell's `summon` beside a brew.
    let mut sim = Sim::new(3);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "kindle charcoal");
    run(&mut sim, "debug_spawn clarified-draught 1");
    run(&mut sim, "distil clarified-draught");
    assert!(
        sim.working().is_some(),
        "the brew that should hold the slot did not start",
    );

    run(&mut sim, "attend menagerie");
    sim.write_spell(
        "calling",
        &["summon".to_owned(), "limn keystone heed".to_owned()],
    );
    sim.step();
    run(&mut sim, "invoke calling");
    sim.step_n(6);

    assert!(
        ever_said(&sim, "a beast gathers at the circle"),
        "a spell's summon waited on the brew: {:?}",
        said(&sim),
    );
    assert!(sim.holds_reading("menagerie", "keystone", "heed"));
}

/// **What the domain is for**, and the shape of bug it would ship with: a
/// world-reading step resolved through where the *player* stands answers about
/// the wrong room, and fails only when automation is doing what automation is
/// for (§19, `0.5.7`).
#[test]
#[cfg(debug_assertions)]
fn a_bound_search_holds_beasts_while_the_player_stands_elsewhere() {
    let mut sim = at_the_circle(3);
    // **Earned, not granted.** Two holds within par are sixteen, which is what
    // a slot costs.
    for _ in 0..2 {
        run(&mut sim, "summon");
        run(&mut sim, "debug_circle");
        run(&mut sim, "summon");
    }
    assert!(
        sim.concentration() > 0,
        "two holds did not earn a slot, at {} experience",
        sim.experience(),
    );

    run(&mut sim, "bind taming");
    run(&mut sim, "attend laboratory");
    let held = sim.tally("event:figure");
    sim.step_n(1800);
    assert!(
        sim.tally("event:figure") > held,
        "a bound search held nothing while the player was in the laboratory",
    );
}

/// How many ticks a shipped search takes to hold its first beast.
fn ticks_to_hold(sim: &mut Sim, spell: &str) -> u64 {
    run(sim, &format!("invoke {spell}"));
    let start = sim.tick().get();
    for _ in 0..2000 {
        if sim.tally("event:figure") > 0 {
            return sim.tick().get() - start;
        }
        sim.step();
    }
    panic!("the search held nothing in 2000 ticks: {:?}", said(sim));
}

/// **The pair, and either half alone passes against a spell that never works.**
/// The search holds at one step a tick — every room automates from the first —
/// and a second step is worth something real here: fewer ticks to the same
/// beast.
#[test]
#[cfg(debug_assertions)]
fn the_search_holds_at_one_step_and_sooner_at_two() {
    let mut one = at_the_circle(11);
    let slow = ticks_to_hold(&mut one, "taming");

    let mut two = at_the_circle(11);
    run(&mut two, "debug_take steps_1");
    let fast = ticks_to_hold(&mut two, "taming");

    assert!(
        fast < slow,
        "a second step a tick did not speed the search: {fast} against {slow}",
    );
}

/// **`winnowing` through the engine, on beasts from every rung it has.** The
/// model's proofs walk every beast through a Rust copy of its loop; this casts
/// the real spell, so a ladder rung that never matches — a misspelt reading, a
/// part that never returns — is caught where a copy would agree with itself.
///
/// Each rung holds its beast without one refusal, since every `limn` in the
/// spell is guarded. **A seed is found for every lit-row count** rather than
/// hoping a range of seeds reaches them, so *every rung* is true by construction.
#[test]
fn the_winnowing_search_holds_on_every_rung() {
    let mut rungs = std::collections::BTreeMap::new();
    for seed in 0..400 {
        if rungs.len() == 7 {
            break;
        }
        let mut drawn = at_the_circle(seed);
        run(&mut drawn, "summon");
        let Some(board) = drawn.circle() else {
            panic!("seed {seed}: summon drew no beast");
        };
        let lit = board.temper.iter().filter(|lit| **lit).count();
        rungs.entry(lit).or_insert(seed);
    }
    assert_eq!(rungs.len(), 7, "400 seeds reached only {rungs:?}");

    for (lit, seed) in rungs {
        let mut sim = at_the_circle(seed);
        ticks_to_hold(&mut sim, "winnowing");
        sim.step_n(60);
        run(&mut sim, "peruse menagerie.log");
        let refused: Vec<String> = said(&sim)
            .into_iter()
            .filter(|line| line.contains("summon first") || line.contains("not a humour"))
            .collect();
        assert!(refused.is_empty(), "lit on {lit}, seed {seed}: {refused:?}");
    }
}

/// **Both searches hold a beast drawn with every turned mask there is**, through
/// the engine. A turned wire is part of the beast, not of the limning, so neither
/// spell had to change — and this is where that is proven with the real spells
/// rather than their Rust copies: a seed is found for each of the nine masks, and
/// each spell holds that seed's first beast without a refusal.
#[test]
fn both_searches_hold_a_beast_under_every_turned_mask() {
    let mut masks = std::collections::BTreeMap::new();
    for seed in 0..400 {
        if masks.len() == orbs_sim::tower::circle::Turned::ALL.len() {
            break;
        }
        let mut drawn = at_the_circle(seed);
        run(&mut drawn, "summon");
        let Some(beast) = drawn.beast() else {
            panic!("seed {seed}: summon drew no beast");
        };
        masks.entry(beast.shape().turned()).or_insert(seed);
    }
    assert_eq!(masks.len(), 9, "400 seeds drew only {masks:?}");

    for (mask, seed) in masks {
        for spell in ["taming", "winnowing"] {
            let mut sim = at_the_circle(seed);
            ticks_to_hold(&mut sim, spell);
            sim.step_n(60);
            run(&mut sim, "peruse menagerie.log");
            let refused: Vec<String> = said(&sim)
                .into_iter()
                .filter(|line| line.contains("summon first") || line.contains("not a humour"))
                .collect();
            assert!(refused.is_empty(), "{spell} under {mask:?}: {refused:?}");
        }
    }
}

/// **Across two dozen beasts it takes fewer ticks than `taming` on the same
/// ones**, which is the whole of what the spell claims. Not per beast: a beast
/// on the rung with every keystone can sit early in `taming`'s order and late in
/// this one's.
#[test]
fn the_winnowing_search_holds_sooner_than_taming() {
    let (mut winnowed, mut tamed) = (0, 0);
    for seed in 0..24 {
        winnowed += ticks_to_hold(&mut at_the_circle(seed), "winnowing");
        tamed += ticks_to_hold(&mut at_the_circle(seed), "taming");
    }
    assert!(
        winnowed < tamed,
        "winnowing took {winnowed} ticks over 24 beasts, taming {tamed}",
    );
}

#[test]
fn a_beast_saved_mid_search_comes_back_the_same() {
    let mut sim = at_the_circle(5);
    run(&mut sim, "summon");
    run(&mut sim, "limn widdershins oppose");
    run(&mut sim, "summon");
    let board = sim.circle();
    assert!(board.is_some());

    let restored = Sim::restored(&sim.snapshot());
    let mut restored = restored;
    run(&mut restored, "attend menagerie");
    assert_eq!(restored.circle(), board, "the beast came back different");
}

/// **A beast refused on load takes its readings with it.** The beast travels as a
/// component and its readings as nodes, so a hand-edited temper no circle
/// answers restored the readings and not the beast — `survey keystone` named a
/// humour with nothing waiting, and `is empty` was false.
#[test]
fn a_beast_refused_on_load_leaves_no_readings_behind() {
    let mut sim = at_the_circle(5);
    run(&mut sim, "summon");
    run(&mut sim, "limn keystone heed");
    let mut save = sim.snapshot();
    // The whole save first, so the assertions below are about the refusal and
    // not about a reading that never survives a load.
    assert!(Sim::restored(&save).holds_reading("menagerie", "keystone", "heed"));
    for node in &mut save.nodes {
        if let Some(beast) = node.beast.as_mut() {
            beast.temper = "11111111".to_owned();
        }
    }

    let restored = Sim::restored(&save);
    assert!(
        restored.circle().is_none(),
        "a temper no circle answers loaded"
    );
    assert!(
        !restored.holds_reading("menagerie", "circle", "fervour"),
        "the refused beast's fervour survived the load",
    );
    assert!(
        !restored.holds_reading("menagerie", "keystone", "heed"),
        "the refused beast's glyph survived the load",
    );
}

#[test]
fn the_same_seed_draws_the_same_beast() {
    let mut a = at_the_circle(17);
    let mut b = at_the_circle(17);
    run(&mut a, "summon");
    run(&mut b, "summon");
    assert_eq!(a.circle(), b.circle());
}

/// Every word `circle::readings` declares is one some state actually publishes
/// — the forge's lint, one room over. A declared reading nothing raises is a
/// word a spell can write and never see answered.
#[test]
fn every_reading_the_domain_declares_is_one_some_state_reaches() {
    let mut sim = at_the_circle(3);
    run(&mut sim, "summon");
    for word in orbs_sim::tower::circle::readings() {
        if word == orbs_sim::tower::circle::FERVOUR {
            assert!(sim.holds_reading("menagerie", "circle", word));
            continue;
        }
        run(&mut sim, &format!("limn sunwise {word}"));
        assert!(
            sim.holds_reading("menagerie", "sunwise", word),
            "`{word}` is declared and never published",
        );
    }
}
