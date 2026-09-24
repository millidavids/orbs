//! Variables and sets: `let … be …`, and `for each …`.
//!
//! §10 gives Spellcraft *"composition — build spells from components"*, and the
//! language had nothing to compose with: no way to hold an answer and no way to
//! say *"each of these"*. `threading` is what that cost — six tiers unrolled by
//! four ways, 52 lines, because *"the way with the fewest marks"* was
//! inexpressible.
//!
//! Driven through a real `Sim`, as `tests/solver.rs` does: there is no public
//! way to put a value in a spell's store, so a test that reached in would pin a
//! state the game cannot reach.

use orbs_sim::Sim;

/// A tower standing in the archive with a maze open.
///
/// The one room with a set worth walking (`for each way`) and a quantity worth
/// comparing (`marks`).
fn in_the_stacks(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    for line in ["attend archive", "research"] {
        sim.submit(line);
        sim.step();
    }
    sim
}

/// Write `lines` as a spell, cast it, and let it run for `ticks`.
fn cast(sim: &mut Sim, name: &str, lines: &[&str], ticks: u64) {
    let lines: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    sim.write_spell(name, &lines);
    sim.step();
    sim.submit(&format!("invoke {name}"));
    sim.step();
    sim.step_n(ticks);
}

/// Every word the session has put on the record stream, one entry per record.
///
/// Every text field, not just `Message`: a `survey` answers with a `Heading`
/// and a `TableRow`, so a messages-only helper saw an empty screen and three
/// of these tests failed against a working game.
fn said(sim: &Sim) -> Vec<String> {
    use orbs_render::{FieldName, Value};
    sim.scrollback()
        .records()
        .iter()
        .map(|record| {
            FieldName::ALL
                .iter()
                .filter_map(|field| match record.field(*field) {
                    Some(Value::Text(text)) => Some(text.to_owned()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

/// Whether anything said contains `needle`.
fn anything_said(sim: &Sim, needle: &str) -> bool {
    said(sim).iter().any(|line| line.contains(needle))
}

#[test]
fn a_bound_name_stands_for_a_place_in_a_command() {
    // The smallest claim there is: `let bowl be mortar_and_pestle` and then
    // `grind sage` into `bowl`.
    //
    // In the laboratory, because its refusals name the instrument: a walk that
    // lands is quiet by design (§19), so the archive gives nothing to match on.
    // Emptying an empty mortar names the mortar, and only a substitution gets
    // you that sentence.
    let mut sim = Sim::new(3);
    sim.submit("attend laboratory");
    sim.step();
    cast(
        &mut sim,
        "pointing",
        &["let bowl be mortar_and_pestle", "empty bowl"],
        6,
    );

    assert!(
        anything_said(&sim, "mortar_and_pestle"),
        "`empty bowl` never reached the mortar, so the bound name did not stand \
         for it: {:?}",
        said(&sim),
    );
    assert!(
        !anything_said(&sim, "no bowl") && !anything_said(&sim, "bowl within"),
        "the tower was asked about something called `bowl`: {:?}",
        said(&sim),
    );
}

#[test]
fn a_bound_name_stands_for_a_place_in_a_question() {
    // The other half, and the one the accumulator needs: a variable has to be
    // usable where a *place* stands in a condition, on both sides of it.
    // Positive and seed-independent, where the first draft was neither — it
    // asked whether two names took the *same* branch, which is satisfied when
    // neither does.
    let mut sim = Sim::new(3);
    sim.submit("attend laboratory");
    sim.step();
    cast(
        &mut sim,
        "asking",
        &[
            "let bowl be mortar_and_pestle",
            "if bowl is idle",
            "survey grimoire",
            "end",
            "if the dispensary has more sage than bowl",
            "survey arsenal",
            "end",
        ],
        12,
    );

    // `first_light`, not `spell`: the cast echoes `invoke asking.spell`, so a
    // needle of `spell` is in the transcript before the question is asked and
    // the test passed with the substitution disabled.
    assert!(
        anything_said(&sim, "first_light"),
        "`if bowl is idle` decided nothing, so the bound name never reached the \
         question: {:?}",
        said(&sim),
    );
    // A `survey` of the fresh tower's empty arsenal says nothing, so this asks
    // the other way round: the spell reached the end rather than stopping at an
    // unanswerable line.
    assert!(
        !anything_said(&sim, "there is no arsenal"),
        "a comparison against a bound place could not find it: {:?}",
        said(&sim),
    );
}

#[test]
fn a_for_each_walks_every_member_of_its_set() {
    // Four, because the archive publishes four ways. The set is declared by
    // `build::Branch::group` alone — `Role::Reading` also covers the lens's
    // sockets and sigils, which is why the group is a fact of its own.
    //
    // Counted by what the loop *says*: four passes is four surveys.
    let mut sim = in_the_stacks(3);
    let before = said(&sim).len();
    cast(
        &mut sim,
        "counting",
        &["for each way", "survey way", "end"],
        12,
    );

    let headings = said(&sim)
        .into_iter()
        .skip(before)
        .filter(|line| line.split_whitespace().any(|word| word == "reading"))
        .count();
    assert_eq!(
        headings, 4,
        "a `for each way` should have surveyed all four ways",
    );
}

#[test]
fn a_for_each_over_a_set_the_room_lacks_walks_nothing() {
    // The lens has sockets and the archive does not. A loop over a set that is
    // not here must step *past* rather than into — descending puts the path
    // somewhere `at` cannot resolve, which the runner reads as the end of the
    // spell, so the lines after it never run.
    //
    // Both lines must be ones that always speak: `survey cabinet` on an empty
    // shelf answers nothing, so an assertion on it passes and fails for reasons
    // unrelated to the loop.
    let mut sim = in_the_stacks(3);
    cast(
        &mut sim,
        "elsewhere",
        &["for each socket", "follow north", "end", "survey grimoire"],
        10,
    );

    assert!(
        !anything_said(&sim, "north"),
        "the body of a loop over a set the room does not have was run",
    );
    assert!(
        anything_said(&sim, "spell"),
        "the line after an empty `for each` never ran, so the loop was entered \
         rather than stepped past",
    );
}

#[test]
fn a_cursor_and_an_accumulator_pick_the_least_walked_way() {
    // The sentence the language could not say, and the reason for this work:
    // `threading` is 52 hand-unrolled lines because a comparison could name a
    // number and not a place, and nothing could hold the answer.
    //
    // Driven as `dev_spells.toml`'s `roaming` writes it, and asserted on the
    // one thing that cannot happen by accident: a fragment in the cabinet is a
    // maze walked end to end.
    let mut sim = in_the_stacks(3);
    cast(
        &mut sim,
        "roaming_here",
        &[
            "repeat until the stacks is idle",
            "let best be north",
            "for each way",
            "if way has no wall",
            "let best be way",
            "end",
            "end",
            "for each way",
            "if way has no wall and no back",
            "let best be way",
            "end",
            "end",
            "for each way",
            "if way has no wall and no back and fewer marks than best",
            "let best be way",
            "end",
            "end",
            "follow best",
            "end",
        ],
        7_200,
    );

    sim.submit("survey cabinet");
    sim.step();
    assert!(
        anything_said(&sim, "fragment") || fragments(&sim) > 0,
        "a 15-line solver did not walk out of the maze",
    );
}

/// How many fragments the archive has won.
fn fragments(sim: &Sim) -> usize {
    use orbs_render::{FieldName, Value};
    sim.scrollback()
        .records()
        .iter()
        .filter(|record| {
            matches!(record.field(FieldName::Name), Some(Value::Text(name)) if name == "fragment")
        })
        .count()
}

#[test]
fn a_half_written_binding_refuses_the_line_rather_than_guessing() {
    // `let best` binds nothing and `let be north` names nothing. Either read as
    // the other is the orb writing down a line the player did not (§19).
    let sim = Sim::new(3);
    for half in ["let best", "let be north", "let the best way be north"] {
        let readings = sim.read_spell("archive", &[half.to_owned()]);
        assert!(
            readings[0].fault.is_some(),
            "{half:?} was read as something rather than refused",
        );
    }
}

#[test]
fn a_spell_line_whose_verb_takes_nothing_says_so_rather_than_naming_a_referent() {
    // The prompt names the verb and the words it could not use. A spell used to
    // answer §8's *Referent missing*, which is the one thing not wrong with the
    // line: `status report` hands a verb words it has no slot for.
    let sim = Sim::new(3);
    let readings = sim.read_spell("archive", &["status report".to_owned()]);
    let fault = readings[0]
        .fault
        .as_ref()
        .expect("`status report` was read as a command");
    assert_eq!(fault.key, "spell_takes_nothing");
    assert_eq!(fault.detail.as_deref(), Some("report"));
}

#[test]
fn a_for_without_its_particle_opens_no_block() {
    // `for way` is missing the word that makes the sentence one, so no block
    // opens and the `end` below it is a stray. Opening an unnamed block would
    // swallow the body into a loop over nothing, silently.
    let sim = Sim::new(3);
    let readings = sim.read_spell(
        "archive",
        &[
            "for way".to_owned(),
            "survey cabinet".to_owned(),
            "end".to_owned(),
        ],
    );
    assert!(readings[0].fault.is_some(), "`for way` was read as a loop");
    assert!(
        readings[2].fault.is_some(),
        "the `end` closed something, so a block was opened after all",
    );
}

#[test]
fn the_orb_quotes_a_bound_name_rather_than_resolving_it() {
    // `follow best` read back as `follow west`: the fuzzy matcher finding the
    // nearest place in the room. The runner substitutes before the parser sees
    // the line and was always right; `interpret` was the liar.
    let sim = Sim::new(3);
    let readings = sim.read_spell(
        "archive",
        &["let best be north".to_owned(), "follow best".to_owned()],
    );
    assert_eq!(
        readings[1].heard, "follow best",
        "the orb resolved a name that will not be known until the line runs",
    );
    assert!(
        readings[1].fault.is_none(),
        "naming a variable was reported as a fault",
    );
}

#[test]
fn a_bound_name_is_not_a_place_the_tower_is_missing() {
    // A variable reaches `compile` looking exactly like a place the room does
    // not have. Reporting it as one puts `spell_nowhere` on every correct `for
    // each` in the game — and it is `Role::Danger`, so it would latch the
    // rail's fault mark on a working spell.
    let sim = Sim::new(3);
    let readings = sim.read_spell(
        "archive",
        &[
            "for each way".to_owned(),
            "if way has passage".to_owned(),
            "follow way".to_owned(),
            "end".to_owned(),
            "end".to_owned(),
        ],
    );
    for reading in &readings {
        assert!(
            reading.fault.is_none(),
            "a correct loop was faulted: {reading:?}",
        );
    }
}

#[test]
fn a_binding_survives_the_orb_being_closed_and_opened() {
    // §8 requires in-flight state be serialisable, and a store is in-flight
    // state. Written by hand because the completeness lint keys on `TypeId`, so
    // a new *field* on an existing component is invisible to it.
    //
    // The observable has to be the *value*: looking for `best` in the document
    // found the spell's own saved text, and counting records found a run that
    // emits either way. So the bound *pair* in the file, and a refusal naming
    // the mortar — an unbound `bowl` says *"there is no bowl within reach"*.
    let mut sim = Sim::new(3);
    sim.submit("attend laboratory");
    sim.step();
    cast(
        &mut sim,
        "keeping",
        &[
            "let bowl be mortar_and_pestle",
            "repeat 400",
            "empty bowl",
            "end",
        ],
        6,
    );

    let save = sim.snapshot();
    let text = save.to_toml().expect("a snapshot writes");
    assert!(
        text.contains(r#"bowl = "mortar_and_pestle""#),
        "the store did not reach the document: no bound pair in\n{text}",
    );

    let mut back = Sim::restored(&save);
    let before = said(&back).len();
    back.step_n(8);
    let after: Vec<String> = said(&back).into_iter().skip(before).collect();
    assert!(
        after.iter().any(|line| line.contains("mortar_and_pestle")),
        "the restored spell stopped naming the mortar, so its binding did not \
         travel: {after:?}",
    );
}
