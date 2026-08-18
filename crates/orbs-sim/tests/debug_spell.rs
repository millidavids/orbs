//! `debug_spell` — a known-good spell written out, in the builds a tester runs.
//!
//! Two properties nothing else can hold. **The door is shut in release**, which
//! only a `--release` run can check because the code is `cfg`'d away; and **one
//! input records one submission**, which only a look at `Submissions` can check
//! because the scrollback shows one line either way.
//!
//! That second one is the whole reason this word does not simply mirror
//! `debug_spawn`. `debug_spawn` records the typed line and re-runs it on replay;
//! `write_spell` already records a `Wrote`. Doing both would push two
//! submissions for one input and a replay would push two more.

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;
#[cfg(debug_assertions)]
use orbs_sim::session::Submission;

/// A wizard standing in the archive, which is where these spells are written for.
fn archive() -> Sim {
    let mut sim = Sim::new(1);
    sim.submit("attend archive");
    sim.step();
    sim
}

fn messages(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| record.field(FieldName::Message))
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

fn said(sim: &Sim) -> String {
    messages(sim).join(" | ")
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
    // **The other half of shelving them**, and the half that matters to a player.
    // `raise_grimoire` puts the dev ladders on the shelf under
    // `cfg(debug_assertions)`; if that guard were ever dropped, a release build
    // would ship a working maze solver, a working ward solver and the answer to
    // the archive's central puzzle — which §12 wants the player to find, and
    // `dev_spells.toml`'s own header says is the whole reason it is not
    // `spells.toml`.
    //
    // Named rather than derived: `execute::dev_spells` does not exist in this
    // build, so there is nothing to iterate. That absence is the point.
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
    // **The defect this shape exists to avoid.** Mirroring `debug_spawn`
    // exactly would push the typed line *and* the `Wrote` that `write_spell`
    // pushes — two entries for one input — and `Sim::replay` would then re-enter
    // `submit`, re-match the word, and push two more.
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
    // **The world replays, the tester's console chatter does not**, and that is
    // the shape rather than a shortfall. Only the write is recorded, so a replay
    // re-writes the spell without re-running the word — the two lines
    // `debug_spell` said to the tester are absent, and every line the *world*
    // produced is identical.
    //
    // That is the right side to be on. `debug_spawn` records its typed line and
    // therefore replays its own announcement; the cost is that its replay is
    // pinned to the tool still existing and still meaning the same thing. Here
    // the recorded lines are the spell itself, so a replay reproduces what ran
    // even after `dev_spells.toml` is edited.
    let mut live = archive();
    live.submit("debug_spell threading");
    live.step();
    live.submit("research");
    live.step_n(40);

    let mut replayed = Sim::new(1);
    for (tick, submission) in live.submissions().all().to_vec() {
        while replayed.tick() < tick {
            replayed.step();
        }
        replayed.replay(submission);
    }
    while replayed.tick() < live.tick() {
        replayed.step();
    }

    let world_lines = |sim: &Sim| -> Vec<String> {
        messages(sim)
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
    // **Reversed deliberately.** This used to assert the opposite — that a dev
    // ladder reached the grimoire only when `debug_spell` wrote it — on the
    // argument that a tester looking at what the game does should not see
    // scaffolding. In practice the first thing a tester does with one is cast it,
    // and making them type `debug_spell breaking` first was a step that taught
    // nothing. They are shelved at construction now, in a debug build only.
    //
    // A shelved spell carries its **own** `Domain` from the file, which is why
    // this can skip the room check `debug_spell` needs: `scribe::write` homes a
    // *new* spell to where the player stands, and nothing here is new.
    // **`Sim::spell`, not the transcript.** The version of this that asserted the
    // opposite read `said(&sim)` — the sentences said so far — which contains no
    // spell name either way, so it passed against a grimoire holding every ladder.
    // An absence test that cannot see the thing it forbids is not a test.
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
