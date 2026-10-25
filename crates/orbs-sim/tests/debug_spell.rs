//! `debug_spell` — a known-good spell written out, in the builds a tester runs.
//!
//! The door is shut in release, and one input records one submission — only
//! the `Wrote`, unlike `debug_spawn`, or a replay doubles.

mod common;

use common::attended;
use orbs_sim::Sim;
#[cfg(debug_assertions)]
use orbs_sim::session::Submission;

/// A wizard standing in the archive, which is where these spells are written for.
fn archive() -> Sim {
    attended(1, "archive")
}

/// Everything the orb said, on one line for a failure message.
fn said(sim: &Sim) -> String {
    common::said(sim).join(" | ")
}

// ---------------------------------------------------------------------------
// The door, from the side a debug build cannot see
// ---------------------------------------------------------------------------

#[cfg(not(debug_assertions))]
#[test]
fn the_word_does_nothing_in_a_release_build() {
    // The module is `cfg(debug_assertions)`, so in a release binary there is no
    // word, no parse and no content — a player who types it gets the ordinary
    // unresolvable line, because as far as this build is concerned nothing
    // answers to it. This is the only test that can say so.
    let mut sim = archive();
    sim.submit("debug_spell threading");
    sim.step();
    assert!(
        !said(&sim).contains("grimoire remembers"),
        "a release build wrote a dev spell: {}",
        said(&sim),
    );
}

#[cfg(not(debug_assertions))]
#[test]
fn a_release_shelf_holds_only_the_shipped_spells() {
    // A release build must not ship the answers §12 wants found. Named, since
    // `execute::dev_spells` does not exist here.
    let sim = archive();
    for name in ["threading", "breaking", "assembling"] {
        assert!(
            sim.spell(name).is_none(),
            "a release build shipped the dev spell {name}",
        );
    }
    assert!(
        sim.spell("first_light").is_some(),
        "the shipped spell is missing, so this asserts nothing",
    );
}

// ---------------------------------------------------------------------------
// One input, one submission
// ---------------------------------------------------------------------------

#[cfg(debug_assertions)]
#[test]
fn one_input_records_exactly_one_submission_and_it_is_the_write() {
    // The defect this shape exists to avoid: mirroring `debug_spawn` exactly
    // would push the typed line and the `Wrote` that `write_spell` pushes — two
    // entries for one input — and `Sim::replay` would re-enter `submit`,
    // re-match the word, and push two more.
    let mut sim = archive();
    let before = sim.submissions().all().len();
    sim.submit("debug_spell threading");
    sim.step();

    let recorded: Vec<Submission> = sim
        .submissions()
        .all()
        .iter()
        .skip(before)
        .map(|(_, submission)| submission.clone())
        .collect();

    assert_eq!(
        recorded.len(),
        1,
        "one input recorded {} submissions: {recorded:?}",
        recorded.len(),
    );
    assert!(
        matches!(recorded[0], Submission::Wrote { .. }),
        "the write is what should be recorded, not the typed line: {:?}",
        recorded[0],
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_session_that_used_it_replays_to_the_same_world() {
    // The world replays, the word's chatter does not: only the write is
    // recorded, so a replay holds even after `dev_spells.toml` changes.
    let mut live = archive();
    live.submit("debug_spell threading");
    live.step();
    live.submit("research");
    live.step_n(40);

    let mut replayed = common::replayed(&live, Sim::new(1));

    let world_lines = |sim: &Sim| -> Vec<String> {
        common::said(sim)
            .into_iter()
            .filter(|line| !line.contains("debug_spell") && !line.contains("grimoire remembers"))
            .collect()
    };
    assert_eq!(
        world_lines(&replayed),
        world_lines(&live),
        "the replayed world diverged",
    );

    // And the spell is really there, rather than merely not having complained:
    // the replay must be able to cast what the live session wrote.
    replayed.submit("invoke threading");
    replayed.step();
    assert!(
        said(&replayed).contains("threading"),
        "the replay could not cast the spell it wrote: {}",
        said(&replayed),
    );
}

// ---------------------------------------------------------------------------
// What it writes, and where it refuses
// ---------------------------------------------------------------------------

#[cfg(debug_assertions)]
#[test]
fn the_dev_spells_are_on_the_shelf_from_the_first_tick() {
    // Shelved at construction in a debug build, since a tester casts them.
    // Each carries its file's `Domain`. Through `Sim::spell`, which can see
    // what the transcript cannot.
    let sim = archive();
    for (name, spell) in orbs_sim::execute::dev_spells().iter() {
        let held = sim
            .spell(name)
            .unwrap_or_else(|| panic!("{name} is not on the shelf"));
        assert_eq!(held, spell.lines, "{name} was shelved with the wrong lines");
    }

    // Readable and castable without `debug_spell` having run at all.
    let mut sim = archive();
    sim.submit("peruse threading.spell");
    sim.step();
    assert!(
        said(&sim).contains("repeat"),
        "a shelved dev spell could not be read: {}",
        said(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn it_refuses_in_a_room_the_spell_is_not_written_for() {
    // `scribe::write` homes a new spell to where the player stands, so writing
    // an archive spell from the laboratory would produce a file whose every line
    // fails to resolve — a broken spell reported as written down.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim.submit("debug_spell threading");
    sim.step();
    assert!(
        said(&sim).contains("archive"),
        "the refusal should name the room: {}",
        said(&sim),
    );
    assert!(
        !said(&sim).contains("grimoire remembers"),
        "it wrote the spell in the wrong room anyway: {}",
        said(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_name_it_does_not_know_is_refused_rather_than_guessed() {
    let mut sim = archive();
    sim.submit("debug_spell nonesuch");
    sim.step();
    assert!(
        said(&sim).contains("nonesuch"),
        "the refusal should quote the name: {}",
        said(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn bare_lists_what_there_is() {
    let mut sim = archive();
    sim.submit("debug_spell");
    sim.step();
    let said = said(&sim);
    for (name, _) in orbs_sim::execute::dev_spells().iter() {
        assert!(said.contains(name), "{name} was not offered: {said}");
    }
}
