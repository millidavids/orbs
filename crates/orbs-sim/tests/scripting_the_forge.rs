//! The forge, scripted — and whether the lattice can be solved by a spell at all.
//!
//! `tests/enchanting.rs` proves the game, `tower::lattice` the puzzle over all
//! 512 boards, `tests/solvers.rs` that the shipped spells run. None asks whether
//! a person could *write* a forge spell, or whether every state they would need
//! is emitted.
//!
//! The forge's answer needs the whole table — eight rungs of compound conditions
//! across three subjects — so three things could make it unscriptable with the
//! suite still green: a residue nothing publishes, `and` not chaining across
//! subjects, or a reading published too late for a spell to see. Each gets a
//! test. The first rots silently: `many_at` answers an absent reading with
//! nought, so a dead rung gets a confident wrong answer rather than an error.

use orbs_render::{FieldName, Value};
use orbs_sim::tower::charm::Kind;
use orbs_sim::tower::lattice::{COLUMNS, Lattice};
use orbs_sim::{Sim, tower};

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

fn at_the_forge(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    run(&mut sim, "attend forge");
    sim
}

/// The mortar, from wherever the player is standing.
///
/// By path, not `reach::look`: a bare `look` is §7's inside-where-you-stand
/// rule, so from the forge it finds nothing.
fn mortar(sim: &Sim) -> bevy_ecs::entity::Entity {
    tower::find_by_path(sim.world(), "/tower/laboratory/mortar_and_pestle")
        .expect("the laboratory has a mortar")
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

/// Write a spell into the forge and cast it.
///
/// Through `write` and `invoke`, never by poking a `Program` into the world —
/// that proves the runner works, not that a person could have got there.
fn cast(sim: &mut Sim, name: &str, lines: &[&str]) {
    let body: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    sim.write_spell(name, &body);
    sim.step();
    run(sim, &format!("invoke {name}"));
}

/// Whether a spell the orb was given reads clean.
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

/// `scene_at` registers every word in `charm::readings()` unconditionally, so a
/// spell compiles before the lattice it asks about exists. The declaration is
/// then the only thing standing between a word and silence.
#[test]
fn every_reading_the_domain_declares_is_one_some_state_reaches() {
    let mut seen: Vec<String> = Vec::new();

    // Enough seeds to draw every residue: each board leaves one of eight, so
    // sixty draws makes missing one vanishingly unlikely.
    for seed in 0..60u64 {
        // One lattice per charm, each in its own tower: `imbue` refuses a
        // second while one is running, so a single sim reaches only one of the
        // five.
        for kind in Kind::ALL {
            let mut sim = at_the_forge(seed);
            run(
                &mut sim,
                &format!("imbue mortar_and_pestle {}", kind.word()),
            );
            for column in COLUMNS {
                run(&mut sim, &format!("survey {column}"));
            }
            run(&mut sim, "survey lattice");
            seen.extend(words(&sim));
        }

        // ...and every charm actually laid, so `graced` is reached on each of
        // the five. Laid rather than solved for: the question is whether the
        // publisher emits the word, not whether the puzzle solves five times.
        let mut bound = at_the_forge(seed);
        let mortar = mortar(&bound);
        let now = *bound.world().resource::<orbs_sim::Tick>();
        let held = tower::Charmed(
            Kind::ALL
                .into_iter()
                .map(|kind| tower::Charm {
                    kind,
                    from: now,
                    // The only state that publishes `ebbing`, which is the
                    // maintenance rung's whole word.
                    ticks: tower::charm::EBBING_AT,
                })
                .collect(),
        );
        bound.world_mut().entity_mut(mortar).insert(held);
        run(&mut bound, "attend forge");
        for kind in Kind::ALL {
            run(&mut bound, &format!("survey {}", kind.word()));
        }
        seen.extend(words(&bound));
    }

    let missing: Vec<&str> = tower::charm::readings()
        .into_iter()
        .filter(|word| !seen.iter().any(|got| got == word))
        .collect();
    assert!(
        missing.is_empty(),
        "the domain declares readings no state ever publishes, so a spell asking \
         for one gets nought for ever: {missing:?}",
    );
}

/// All eight residues are reachable. One that never comes up is a rung a player
/// writes, never sees fire, and takes for broken.
#[test]
fn every_residue_the_table_has_a_rung_for_actually_comes_up() {
    let mut seen: std::collections::BTreeSet<Vec<bool>> = std::collections::BTreeSet::new();
    for bits in 0..(1u64 << 9) {
        seen.insert(Lattice::from_bits(bits).residue().to_vec());
    }
    assert_eq!(
        seen.len(),
        8,
        "the eight rungs of every forge solver do not cover eight distinct \
         residues: {seen:?}",
    );
}

// ---------------------------------------------------------------------------
// 2. A person can write the questions
// ---------------------------------------------------------------------------

/// The rung the whole table is made of, compiled through the real editor: three
/// subjects joined by `and`. If `Condition::All` did not chain across
/// independent subjects, the eight-rung table would be unwritable.
#[test]
fn a_three_subject_rung_compiles_and_runs() {
    let mut sim = at_the_forge(11);
    run(&mut sim, "imbue mortar_and_pestle hurried");
    cast(
        &mut sim,
        "reading",
        &[
            "if the apex has lit and the belt has no lit and the hem has lit",
            "    snap apex",
            "end",
        ],
    );
    for _ in 0..8 {
        sim.step();
    }
    assert!(
        complaints(&sim).is_empty(),
        "a three-subject rung did not compile: {:?}",
        complaints(&sim),
    );
}

/// Each of the five charm words is a question a spell can ask.
#[test]
fn every_charm_is_a_word_a_spell_can_ask_about() {
    for kind in Kind::ALL {
        let mut sim = at_the_forge(3);
        cast(
            &mut sim,
            &format!("asking_{}", kind.word()),
            &[
                &format!("if the {} has graced", kind.word()),
                "    survey lattice",
                "end",
            ],
        );
        for _ in 0..6 {
            sim.step();
        }
        assert!(
            complaints(&sim).is_empty(),
            "a spell cannot ask about {}: {:?}",
            kind.word(),
            complaints(&sim),
        );
    }
}

/// `for each column` and `for each charm` walk the sets the room declares.
///
/// Both, because they fail differently: no `column` set makes the generic
/// solver unwritable, no `charm` set the maintenance spell.
#[test]
fn the_rooms_two_sets_are_walkable() {
    for set in ["column", "charm"] {
        let mut sim = at_the_forge(3);
        run(&mut sim, "imbue mortar_and_pestle hurried");
        cast(
            &mut sim,
            &format!("walking_{set}"),
            &[&format!("for each {set}"), "    survey lattice", "end"],
        );
        for _ in 0..12 {
            sim.step();
        }
        assert!(
            complaints(&sim).is_empty(),
            "`for each {set}` does not compile: {:?}",
            complaints(&sim),
        );
    }
}

/// `recall scripting` teaches this room's words — the only place a player finds
/// out what they may ask. The lens once taught the *archive's* words, because
/// the gate asked `Role::Reading` rather than the room's own sets.
#[test]
fn the_scripting_page_teaches_the_forges_own_words() {
    let mut sim = at_the_forge(3);
    run(&mut sim, "recall scripting");
    // Every field, not just `Message`: the page's sets and nameable words are
    // `TableRow`s, so a `Message`-only reading finds one prose line and reads
    // as though the page teaches nothing.
    let mut page: Vec<String> = Vec::new();
    for record in sim.scrollback().records().iter() {
        for (_, value) in record.fields() {
            if let Value::Text(text) = value {
                page.push(text.to_owned());
            }
        }
    }
    let page = page.join(" ");
    for word in ["column", "charm"] {
        assert!(
            page.contains(word),
            "`recall scripting` in the forge never names the {word} set: {page}",
        );
    }
}

// ---------------------------------------------------------------------------
// 3. The table is right, and a spell holding it solves the puzzle
// ---------------------------------------------------------------------------

/// The shipped solver's table against `tower::lattice`'s true answer on all 512
/// boards. The rungs are hand-written TOML and nothing else compares them to
/// the arithmetic, so a transcription error would survive everything else.
#[test]
fn the_shipped_table_is_the_answer_the_arithmetic_gives() {
    // The rungs of `forging`, as `(residue, columns to snap)`. Read off the
    // spell rather than derived, so a change to one and not the other fails.
    let table: [([bool; 3], &[usize]); 8] = [
        ([true, true, true], &[]),
        ([true, true, false], &[1, 2]),
        ([true, false, true], &[0, 1, 2]),
        ([true, false, false], &[0]),
        ([false, true, true], &[0, 1]),
        ([false, true, false], &[0, 2]),
        ([false, false, true], &[2]),
        ([false, false, false], &[1]),
    ];

    for bits in 0..(1u64 << 9) {
        let lattice = Lattice::from_bits(bits);
        let residue = lattice.residue();
        let (_, wanted) = table
            .iter()
            .find(|(key, _)| *key == residue)
            .expect("the table does not cover this residue");

        let mut played = Lattice::from_bits(bits);
        for column in *wanted {
            played.snap(*column);
        }
        assert!(
            played.settle(),
            "the shipped table answers residue {residue:?} with {wanted:?}, \
             which does not light board {bits:#b}",
        );
    }
}

/// ...and the same table, written as a spell, solves a real lattice in the game.
///
/// The end-to-end claim: everything above can hold and the domain still be
/// unscriptable if a reading lands a tick late, the loop guard is wrong, or
/// `anneal` refuses inside a spell. Several seeds, not one board's luck.
#[cfg(debug_assertions)]
#[test]
fn a_spell_holding_the_table_binds_a_charm_on_every_seed() {
    for seed in [0, 3, 11, 17, 42] {
        let mut sim = at_the_forge(seed);
        run(&mut sim, "invoke forging");
        // Generous: `SCRIPT_BUDGET` is one step a tick, so an eight-rung ladder
        // costs eight ticks before it snaps anything, and running out of ticks
        // looks exactly like a table that does not work.
        for _ in 0..200 {
            sim.step();
        }
        assert!(
            tower::charmed(sim.world(), mortar(&sim), Kind::Hurried),
            "seed {seed}: the shipped solver never bound a charm — said {:?}",
            said(&sim),
        );
    }
}

/// The maintenance spell's cold-start rung fires on a tower that has never
/// enchanted anything.
///
/// `ebbing` is published only while a charm is running *and* nearly done, so a
/// loop guarded on it alone would never start, and maintain nothing for ever.
#[cfg(debug_assertions)]
#[test]
fn the_maintenance_spell_starts_from_a_tower_with_no_charm_at_all() {
    let mut sim = at_the_forge(3);
    run(&mut sim, "invoke tending_forge");
    for _ in 0..200 {
        sim.step();
    }
    let mortar = mortar(&sim);
    assert!(
        tower::charmed(sim.world(), mortar, Kind::Hurried),
        "the maintenance spell did not start from nothing: {:?}",
        said(&sim),
    );
}

/// ...and it fires again when the charm is nearly out, which is the other half.
#[cfg(debug_assertions)]
#[test]
fn the_maintenance_spell_renews_a_charm_that_is_ebbing() {
    let mut sim = at_the_forge(3);
    run(&mut sim, "invoke tending_forge");
    for _ in 0..200 {
        sim.step();
    }
    let mortar = mortar(&sim);
    let first = tower::charm_left(sim.world(), mortar, Kind::Hurried);
    assert!(first > 0, "nothing was bound to renew");

    // By arithmetic, not a guessed number: a literal `meditate 560` was right
    // for one `lasts` and one solver speed, and its failure read as *the charm
    // never ebbs* rather than *the fixture overshot*.
    let into_the_band = first.saturating_sub(tower::charm::EBBING_AT / 2);
    run(&mut sim, &format!("meditate {into_the_band}"));
    let ebbing = tower::charm_left(sim.world(), mortar, Kind::Hurried);
    assert!(
        ebbing > 0 && ebbing <= tower::charm::EBBING_AT,
        "the charm is not in the ebbing band: {ebbing} left",
    );

    run(&mut sim, "invoke tending_forge");
    for _ in 0..200 {
        sim.step();
    }
    assert!(
        tower::charm_left(sim.world(), mortar, Kind::Hurried) > ebbing,
        "an ebbing charm was not renewed: {:?}",
        said(&sim),
    );
}

// ---------------------------------------------------------------------------
// 4. Absent is nought, which is where a wrong answer would be plausible
// ---------------------------------------------------------------------------

/// A charm never laid publishes nothing, so `has no graced` is true of it.
///
/// `many_at` answers an absent reading with nought, so an unconditional count
/// is `graced = 0`: *here and out of time*, not *no charm*.
#[test]
fn a_charm_never_laid_publishes_nothing_at_all() {
    let mut sim = at_the_forge(3);
    run(&mut sim, "survey whetted");
    let shown = words(&sim);
    assert!(
        !shown.iter().any(|word| word == tower::charm::GRACED),
        "a charm nobody has laid reports how long it has left: {shown:?}",
    );
    assert!(
        !shown.iter().any(|word| word == tower::charm::EBBING),
        "a charm nobody has laid reports as ebbing: {shown:?}",
    );
}

/// A column publishes `lit` only where its residue glyph is alight.
///
/// The other half of absent-is-nought: if a dark column published anything,
/// every rung would match.
#[test]
fn a_dark_column_publishes_nothing() {
    let mut sim = at_the_forge(11);
    run(&mut sim, "imbue mortar_and_pestle hurried");

    let lattice = tower::reach::look(sim.world())
        .kind(orbs_sim::parser::NounKind::Place)
        .find("lattice")
        .expect("the forge has a lattice");
    let residue = sim
        .world()
        .get::<tower::lattice::Binding>(lattice)
        .expect("a lattice is open")
        .lattice
        .residue();

    for (index, column) in COLUMNS.iter().enumerate() {
        let mut asking = at_the_forge(11);
        run(&mut asking, "imbue mortar_and_pestle hurried");
        run(&mut asking, &format!("survey {column}"));
        let lit = words(&asking).iter().any(|word| word == "lit");
        assert_eq!(
            lit, residue[index],
            "the {column} column says {lit} where the model says {}",
            residue[index],
        );
    }
}
