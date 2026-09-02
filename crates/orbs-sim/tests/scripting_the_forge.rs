//! The forge, scripted — and whether the lattice can be solved by a spell at all.
//!
//! `tests/enchanting.rs` proves the **game**: that every charm reaches the number
//! it is meant to change and stops when it lapses. `tower::lattice` proves the
//! **puzzle**, exhaustively, over all 512 boards. `tests/solvers.rs` proves the
//! shipped spells run. None of them asks the question this file does: **can a
//! person write a forge spell at all**, and is every state they would need to
//! write one actually emitted?
//!
//! # Why this domain needs the question asked harder than the others
//!
//! Every other room can be scripted by a rule that reads one thing and acts. The
//! forge cannot: the residue identifies the answer only if you *hold the table*,
//! and the table is eight rungs of compound conditions across three separate
//! subjects. So there are three distinct ways this could be unscriptable and the
//! whole suite still be green —
//!
//! 1. the three columns never publish some residue, so a rung is dead;
//! 2. `and` does not chain across subjects, so the rung cannot be written;
//! 3. the reading is published but not while a spell can see it.
//!
//! Each gets a test. The first is the one that would rot silently: `many_at`
//! answers an **absent** reading with nought, so a rung asking about a residue
//! nothing publishes gets a confident wrong answer rather than an error.

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
/// **By path, not by `reach::look`.** A bare `look` is §7's plain rule — inside
/// where you stand — so from the forge it finds nothing, which is the whole
/// point of the forge reaching the tower and the trap a test written in the
/// laboratory does not see.
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

/// **The lint the bailey's own version of this file exists because of.**
///
/// `charm::readings()` is the domain's contract with the language: `scene_at`
/// registers every word in it *unconditionally*, so all of them compile in a
/// spell whether or not anything ever publishes them. That is deliberate — a
/// spell must compile before the lattice it asks about exists — and it means the
/// declaration is the **only** thing standing between a word and silence.
#[test]
fn every_reading_the_domain_declares_is_one_some_state_reaches() {
    let mut seen: Vec<String> = Vec::new();

    // Enough seeds to draw every residue: each board leaves one of eight, and
    // sixty draws makes missing all of any one vanishingly unlikely — which is
    // exactly the claim, since a residue nothing draws is a dead rung.
    for seed in 0..60u64 {
        // **One lattice per charm, each in its own tower.** A charm's own word
        // is published on the lattice while a binding of that kind is open, and
        // `imbue` refuses a second while one is running — so a single sim can
        // only ever reach one of the five, which is how four of them read as
        // unreachable on the first pass.
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
        // the five. **Laid rather than solved for**, and the distinction
        // matters: what is being asked here is whether the *publisher* emits the
        // word, not whether the puzzle is solvable five times — that is
        // `the_shipped_table_is_the_answer_the_arithmetic_gives`'s question and
        // it is answered over all 512 boards.
        let mut bound = at_the_forge(seed);
        let mortar = mortar(&bound);
        let now = *bound.world().resource::<orbs_sim::Tick>();
        let held = tower::Charmed(
            Kind::ALL
                .into_iter()
                .map(|kind| tower::Charm {
                    kind,
                    from: now,
                    // Inside the ebbing band, which is the only state that
                    // publishes `ebbing` — the maintenance rung's whole word.
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

/// **All eight residues are reachable**, which is what makes eight rungs worth
/// writing rather than three.
///
/// A residue that never comes up is a rung a player would write, test by hand,
/// and never see fire — and it would look like the rung was wrong.
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

/// **The rung the whole table is made of, compiled through the real editor.**
///
/// Three subjects joined by `and`. If `Condition::All` did not chain across
/// independent subjects this would not compile, and the eight-rung table — and
/// with it the entire domain's scriptability — would be unwritable.
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
/// **Both, because they fail differently.** A missing `column` set makes the
/// generic solver unwritable; a missing `charm` set makes the *maintenance*
/// spell unwritable, and that is Phase 9's fourth box.
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

/// **`recall scripting` teaches this room's words**, which is the only place a
/// player finds out what they may ask.
///
/// The lens shipped a version of this teaching the *archive's* words, because
/// the gate asked `Role::Reading` rather than the room's own sets. So this asks
/// what the page actually says.
#[test]
fn the_scripting_page_teaches_the_forges_own_words() {
    let mut sim = at_the_forge(3);
    run(&mut sim, "recall scripting");
    // **Every field, not just `Message`.** The page's sets and its nameable
    // words are `TableRow`s — a `Message`-only reading of it came back with the
    // page's one prose line and nothing else, and read as *the page teaches
    // nothing* when it teaches all of it.
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

/// **The eight rungs, checked against the model rather than against themselves.**
///
/// This is the test that would catch the table being transcribed wrong — the
/// single most likely defect in the domain, because the rungs are written by
/// hand in TOML and nothing else compares them to the arithmetic.
///
/// It builds the table the shipped solver holds, then asks `tower::lattice` for
/// the true answer on every one of the 512 boards and compares.
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
/// **The end-to-end claim.** Everything above could hold and the domain still be
/// unscriptable if the readings were published a tick too late, or the loop
/// guard were wrong, or `anneal` refused inside a spell. Across several seeds,
/// so it is not one board's luck.
#[test]
fn a_spell_holding_the_table_binds_a_charm_on_every_seed() {
    for seed in [0, 3, 11, 17, 42] {
        let mut sim = at_the_forge(seed);
        run(&mut sim, "invoke forging");
        // **Generous, and it has to be.** `SCRIPT_BUDGET` is one step a tick, so
        // an eight-rung ladder costs eight ticks to walk before it snaps
        // anything — a loop that ran out of ticks would look exactly like a
        // table that does not work.
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
/// **The half that is easy to get wrong.** `ebbing` is published only while a
/// charm is running *and* nearly done, so a loop guarded on it alone would never
/// start — a maintenance spell that maintains nothing, silently, for ever.
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

    // **Run it down into the band by arithmetic, not by a guessed number.** A
    // literal `meditate 560` was right for one `lasts` and one solver speed and
    // wrong the moment either moved — and its failure reads as *the charm never
    // ebbs* rather than as *the fixture overshot*.
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
/// **The trap, stated as a test.** `many_at` answers an absent reading with
/// nought, so the failure mode of publishing a charm's count unconditionally is
/// not an error — it is `graced = 0` reading as *a charm that is here and has no
/// time left*, which is a different sentence from *no charm*.
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
/// The other half of absent-is-nought, and the one the eight-rung table rests
/// on: if a dark column published anything at all, every rung would match.
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
