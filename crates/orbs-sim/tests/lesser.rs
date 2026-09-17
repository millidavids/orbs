//! The lesser circle: a sealed tower's first beasts, the keystone alone over two
//! senses, until `menagerie_2` opens the whole circle (§19).
//!
//! **Driven through real verbs in a sealed tower**, because the lesser circle
//! exists only there — `Sim::new` is the open tower every other test, dump and
//! balance policy uses, and it must never draw one.

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;
use orbs_sim::save::Save;

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

/// A sealed tower with the menagerie open, standing in it.
#[cfg(debug_assertions)]
fn sealed_at_the_circle(seed: u64) -> Sim {
    let mut sim = Sim::sealed(seed);
    run(&mut sim, "debug_reach lens_1");
    run(&mut sim, "attend menagerie");
    assert_eq!(sim.location(), "/tower/menagerie", "{:?}", said(&sim));
    sim
}

/// **The claim the box makes, end to end.** Five lesser beasts held, the
/// second station reached, the whole circle opened and said so — and the sixth
/// beast is a whole one.
#[cfg(debug_assertions)]
#[test]
fn a_sealed_towers_first_beasts_are_lesser_and_the_sixth_is_whole() {
    let mut sim = sealed_at_the_circle(3);
    for held in 0..5 {
        run(&mut sim, "summon");
        let Some(board) = sim.circle() else {
            panic!("beast {held} was not drawn: {:?}", said(&sim));
        };
        assert_eq!(board.lines.len(), 1, "beast {held} is not lesser");
        assert_eq!(board.temper.len(), 4, "beast {held} has eight rows");
        assert!(!sim.has_opened("circle"), "opened after {held} holds");
        run(&mut sim, "debug_circle");
        run(&mut sim, "summon");
        assert!(sim.circle().is_none(), "beast {held} was not held");
    }
    assert!(ever_said(&sim, "a lesser beast gathers"));
    assert!(
        sim.has_opened("circle"),
        "five holds did not open the whole circle"
    );
    assert!(
        ever_said(&sim, "the circle opens whole"),
        "{:?}",
        said(&sim)
    );

    run(&mut sim, "summon");
    let Some(board) = sim.circle() else {
        panic!("the sixth beast was not drawn");
    };
    assert_eq!(board.lines.len(), 3, "the sixth beast is lesser");
    assert_eq!(board.temper.len(), 8);
}

/// **Five at every length a player can choose.** `Sim::sealed` runs the curve
/// as authored, and the second station on a line is ramped by length — so a test
/// only at the authored curve passed while a medium game asked eleven lesser
/// beasts and a long one thirty-five. `fixed` is what holds it at five.
#[cfg(debug_assertions)]
#[test]
fn the_whole_circle_opens_after_five_lesser_beasts_at_every_length() {
    use orbs_sim::content::Length;
    for length in Length::OFFERED {
        let mut sim = Sim::begun(3, length);
        run(&mut sim, "debug_reach lens_1");
        run(&mut sim, "attend menagerie");
        for _ in 0..5 {
            assert!(
                !sim.has_opened("circle"),
                "{length:?} opened the circle early"
            );
            run(&mut sim, "summon");
            run(&mut sim, "debug_circle");
            run(&mut sim, "summon");
        }
        assert!(
            sim.has_opened("circle"),
            "{length:?} held five lesser beasts and the circle stayed shut",
        );
    }
}

/// **Both shipped searches hold a lesser beast, of every temper, and latch no
/// fault.** Written for the whole circle, both spend most of a lesser search on
/// `limn`s the circle refuses as dark — `taming` holds anyway because its
/// keystone steps every thirty-six calls, `winnowing` because its ladder's rungs
/// all hold the keystone's answer. Every sealed game's first menagerie is this
/// case, so a change that let a cost refusal stop a spell would break both where
/// no open-tower test could see it.
#[cfg(debug_assertions)]
#[test]
fn both_shipped_searches_hold_every_lesser_beast_without_a_fault() {
    let mut tempers = std::collections::BTreeMap::new();
    for seed in 0..200 {
        if tempers.len() == 5 {
            break;
        }
        let mut drawn = sealed_at_the_circle(seed);
        run(&mut drawn, "summon");
        let Some(board) = drawn.circle() else {
            panic!("seed {seed} drew no lesser beast");
        };
        tempers.entry(board.temper).or_insert(seed);
    }
    assert_eq!(tempers.len(), 5, "200 seeds drew only {tempers:?}");

    for (temper, seed) in tempers {
        for spell in ["taming", "winnowing"] {
            let mut sim = sealed_at_the_circle(seed);
            run(&mut sim, &format!("invoke {spell}"));
            let mut ticks = 0;
            while sim.tally("event:figure") == 0 {
                assert!(
                    ticks < 3000,
                    "{spell} held no lesser beast {temper:?} in 3000 ticks",
                );
                sim.step();
                ticks += 1;
            }
            let faults: Vec<&str> = sim
                .briefs()
                .into_iter()
                .filter(|brief| brief.mark == Some(orbs_sim::tower::Mark::Fault))
                .map(|brief| brief.name)
                .collect();
            assert!(faults.is_empty(), "{spell} latched a fault: {faults:?}");
        }
    }
}

/// **Only the keystone answers**: a dark glyph is refused as a cost and carries
/// no reading, so a spell asking what sunwise is limned with hears nothing.
#[cfg(debug_assertions)]
#[test]
fn a_lesser_circle_refuses_a_dark_glyph_and_publishes_only_the_keystone() {
    let mut sim = sealed_at_the_circle(3);
    run(&mut sim, "summon");
    run(&mut sim, "limn sunwise heed");
    assert!(
        ever_said(&sim, "sunwise is dark in a lesser circle"),
        "{:?}",
        said(&sim),
    );
    assert!(sim.holds_reading("menagerie", "keystone", "yoke"));
    assert!(sim.holds_reading("menagerie", "circle", "fervour"));
    for glyph in ["sunwise", "widdershins"] {
        assert!(
            !sim.holds_reading("menagerie", glyph, "yoke"),
            "the dark {glyph} publishes a humour",
        );
    }

    // Stepping the keystone is what a lesser search does, and it works.
    run(&mut sim, "limn keystone");
    assert!(sim.holds_reading("menagerie", "keystone", "spurn"));
}

/// **One troop and a quarter of the price** — within par, which is one call —
/// and the quarter of three quarters past it.
#[cfg(debug_assertions)]
#[test]
fn a_lesser_hold_pays_one_troop_and_a_quarter_of_the_price() {
    let mut sim = sealed_at_the_circle(3);
    run(&mut sim, "summon");
    let before = sim.experience();
    run(&mut sim, "debug_circle");
    run(&mut sim, "summon");
    assert!(
        ever_said(&sim, "the circle holds. 1 troop answers, on call 1"),
        "{:?}",
        said(&sim),
    );
    assert_eq!(sim.experience(), before + 2, "a lesser hold did not pay 2");

    run(&mut sim, "summon");
    run(&mut sim, "summon");
    let before = sim.experience();
    run(&mut sim, "debug_circle");
    run(&mut sim, "summon");
    assert!(ever_said(&sim, "on call 2"), "{:?}", said(&sim));
    assert_eq!(
        sim.experience(),
        before + 1,
        "past par did not pay a quarter of three quarters",
    );
}

/// **The open tower never draws one**, whatever the seed — every test, dump and
/// balance policy stands in it.
#[test]
fn an_open_tower_never_draws_a_lesser_beast() {
    for seed in 0..24 {
        let mut sim = Sim::new(seed);
        run(&mut sim, "attend menagerie");
        run(&mut sim, "summon");
        let lines = sim.circle().map(|board| board.lines.len());
        assert_eq!(lines, Some(3), "seed {seed} drew a lesser beast");
    }
}

#[cfg(debug_assertions)]
#[test]
fn a_lesser_beast_saved_mid_call_comes_back_lesser() {
    let mut sim = sealed_at_the_circle(5);
    run(&mut sim, "summon");
    // Called on the opening, which never holds, and then limned — so the beast
    // is saved with an answer and a glyph moved off where it arrived.
    run(&mut sim, "summon");
    run(&mut sim, "limn keystone oppose");
    let board = sim.circle();
    assert_eq!(board.as_ref().map(|board| board.lines.len()), Some(1));

    let text = sim.snapshot().to_toml().expect("a save renders");
    let save = Save::from_toml(&text).expect("a save reads back");
    let mut restored = Sim::restored(&save);
    run(&mut restored, "attend menagerie");
    assert_eq!(
        restored.circle(),
        board,
        "the lesser beast came back different"
    );
}

/// **12 → 13.** An open tower's document lists every key it had, and `circle`
/// was not one — so without the migration it would load drawing lesser beasts
/// for ever with nothing able to open the circle.
#[test]
fn an_open_towers_older_document_keeps_the_whole_circle() {
    let sim = Sim::new(1);
    let mut save = sim.snapshot();
    if let Some(opened) = save.progress.opened.as_mut() {
        opened.retain(|key| key != "circle");
    }
    let stamped = format!("format = {}", orbs_sim::save::FORMAT);
    let older = save
        .to_toml()
        .expect("a save renders")
        .replacen(&stamped, "format = 12", 1);

    let migrated = Save::from_toml(&older).expect("a format-12 save migrates");
    let mut restored = Sim::restored(&migrated);
    assert!(
        restored.has_opened("circle"),
        "the open tower lost its circle"
    );
    run(&mut restored, "attend menagerie");
    run(&mut restored, "summon");
    assert_eq!(restored.circle().map(|board| board.lines.len()), Some(3));
}

/// **A sealed one past the station catches up on its own** — the migration
/// leaves sealed towers to `mastery::caught_up`, and this is the case that
/// leaning on it has to be right about: a document that reached `menagerie_2`
/// without the key opens the whole circle on load, silently — the station was
/// reached, so its player was told when it was.
#[cfg(debug_assertions)]
#[test]
fn a_sealed_towers_older_document_past_the_station_opens_the_circle_on_load() {
    let mut sim = sealed_at_the_circle(3);
    run(&mut sim, "debug_reach menagerie_2");
    assert!(sim.reached().iter().any(|id| id == "menagerie_2"));
    let mut save = sim.snapshot();
    if let Some(opened) = save.progress.opened.as_mut() {
        opened.retain(|key| key != "circle");
    }
    let stamped = format!("format = {}", orbs_sim::save::FORMAT);
    let older = save
        .to_toml()
        .expect("a save renders")
        .replacen(&stamped, "format = 12", 1);

    let migrated = Save::from_toml(&older).expect("a format-12 save migrates");
    let restored = Sim::restored(&migrated);
    assert!(
        restored.has_opened("circle"),
        "a tower past menagerie_2 loaded without the whole circle",
    );
    assert!(
        !said(&restored)
            .iter()
            .skip(said(&sim).len())
            .any(|line| line.contains("opens whole")),
        "a load announced the circle opening again",
    );
}

/// **A document that met the station without reaching it reaches it on load.**
/// A medium tower saved seven holds into `menagerie_2`'s old ramped eleven has
/// met the fixed five — and `advance` runs after a completion, never on load,
/// so without catching up it drew lesser beasts until something unrelated
/// finished. Built here as a sealed tower with five holds and the station struck
/// out of the document, which is the same state.
#[cfg(debug_assertions)]
#[test]
fn a_document_whose_holds_meet_the_station_reaches_it_on_load() {
    let mut sim = sealed_at_the_circle(3);
    for _ in 0..5 {
        run(&mut sim, "summon");
        run(&mut sim, "debug_circle");
        run(&mut sim, "summon");
    }
    let mut save = sim.snapshot();
    save.progress.reached.retain(|id| id != "menagerie_2");
    if let Some(opened) = save.progress.opened.as_mut() {
        opened.retain(|key| key != "circle");
    }

    let saying = |sim: &Sim| {
        said(sim)
            .iter()
            .filter(|line| line.contains("opens whole"))
            .count()
    };
    let told_before = saying(&sim);
    let mut restored = Sim::restored(&save);
    assert!(
        restored.reached().iter().any(|id| id == "menagerie_2"),
        "five holds and the station stayed unreached on load",
    );
    assert!(
        restored.has_opened("circle"),
        "the whole circle stayed shut"
    );
    // **Said by the load, once** — a document short of the station never told
    // its player what a `~` wire is, and the next beast arrives with them. The
    // station itself is not congratulated.
    assert_eq!(
        saying(&restored),
        told_before + 1,
        "the load opened the circle without saying what a ~ is",
    );
    let before = said(&restored).len();
    run(&mut restored, "summon");
    assert_eq!(restored.circle().map(|board| board.lines.len()), Some(3));
    assert!(
        !said(&restored)
            .iter()
            .skip(before)
            .any(|line| line.contains("line advances") || line.contains("opens whole")),
        "the load's catching up was announced on the next command",
    );
}

/// **A sealed one is left alone** — short of `menagerie_2`, it draws lesser
/// beasts, which is the new rule rather than a loss.
#[test]
fn a_sealed_towers_older_document_is_not_given_the_circle() {
    let sim = Sim::sealed(1);
    let save = sim.snapshot();
    let stamped = format!("format = {}", orbs_sim::save::FORMAT);
    let older = save
        .to_toml()
        .expect("a save renders")
        .replacen(&stamped, "format = 12", 1);
    let migrated = Save::from_toml(&older).expect("a format-12 save migrates");
    assert!(!Sim::restored(&migrated).has_opened("circle"));
}
