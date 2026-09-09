//! The three tiers — which lines a trained reader is allowed to see (§6).
//!
//! The augury is additive by construction: the deterministic pipeline answers
//! first and its answer wins whenever it has one, so no phrasing that resolves
//! today can resolve differently tomorrow. These are the claims that make that
//! true, written so they hold with no model in the room.

use orbs_sim::augur::{Augur, Fixture};
use orbs_sim::{Sim, parser};

/// A reader that answers everything, wrongly, and loudly.
///
/// **The instrument for proving tier one.** A test that a literal line is not
/// sent to a reader cannot be written with a reader that abstains — abstaining
/// and never being asked look identical from outside. This one always answers,
/// so being consulted at all is visible in the world.
struct Trap;

impl Augur for Trap {
    fn read(&self, _line: &str) -> Vec<String> {
        vec![String::from("attend archive")]
    }
}

/// A reader whose first answer never runs and whose second always does.
///
/// **The candidates rule, made visible.** A reader offers several because it
/// cannot tell `run {script}` from `run the {place}` — nothing in the sentence
/// says which. The caller must skip past what does not resolve rather than
/// taking the head of the list on faith.
struct Fussy;

impl Augur for Fussy {
    fn read(&self, _line: &str) -> Vec<String> {
        vec![
            // No such room, so this one cannot run.
            String::from("attend nowhere-at-all"),
            String::from("survey"),
        ]
    }
}

/// A reader that is never right and never speaks.
struct Silent;

impl Augur for Silent {
    fn read(&self, _line: &str) -> Vec<String> {
        Vec::new()
    }
}

/// Whether the newest reading was worked out rather than read.
fn divined(sim: &Sim) -> bool {
    sim.parse_log()
        .records()
        .last()
        .is_some_and(|record| record.divined)
}

/// What the orb echoed for the newest reading.
fn echo(sim: &Sim) -> Option<String> {
    sim.parse_log()
        .records()
        .last()
        .and_then(|record| record.echo.clone())
}

#[test]
fn a_command_typed_properly_never_reaches_a_reader() {
    // The mastery arc (§6) is players graduating to the canonical form, and it
    // must not pass through anything probabilistic. `Trap` answers every line,
    // so if any of these consulted it the echo would say `attend archive`.
    for line in [
        "survey",
        "attend laboratory",
        "recall brewing",
        "ls",
        "status",
    ] {
        let mut sim = Sim::new(0);
        sim.submit_reading(line, &Trap);
        assert!(!divined(&sim), "{line:?} was handed to a reader");
        assert_ne!(echo(&sim).as_deref(), Some("attend archive"), "{line:?}");
    }
}

#[test]
fn a_spell_word_and_a_prompt_answer_never_reach_a_reader() {
    // Both are answered before the parser and must be answered before a reader.
    // A digit routed to a model makes the numbered prompt rhetorical: it asks a
    // question and then reads the answer as a command.
    for line in ["repeat 3", "wait for the mortar_and_pestle", "1", "2"] {
        let mut sim = Sim::new(0);
        sim.submit_reading(line, &Trap);
        assert!(!divined(&sim), "{line:?} was handed to a reader");
    }
}

#[test]
fn a_typo_is_the_matchers_and_not_a_readers() {
    // §19's *"the player this game is built for"*. `clarty` reaches `clarity`
    // at 819 and the matcher is better at that than a model trained on
    // phrasings, so routing typos to a reader would hand it work already done.
    let mut sim = Sim::new(0);
    sim.submit_reading("recall clarty", &Trap);
    assert!(!divined(&sim));
    assert_eq!(echo(&sim).as_deref(), Some("recall clarity"));
}

#[test]
fn a_phrasing_the_orb_cannot_read_reaches_the_reader_and_runs() {
    // The feature, in one test. `turn the sage into powder` resolves to nothing
    // today; it grinds now.
    //
    // **Standing in the laboratory matters.** `grind` is anchored to the mortar
    // (§7), so the canonical command the reader produces is `Elsewhere` from
    // anywhere else — which is right, and would make this test measure the room
    // rather than the reader.
    let mut sim = Sim::new(0);
    sim.submit("attend laboratory");
    sim.step();

    sim.submit_reading("turn the sage into powder", &Fixture::worked());

    assert!(divined(&sim), "the reader was not consulted");
    assert_eq!(echo(&sim).as_deref(), Some("grind sage"));

    // ...and the transcript keeps the player's own words, not the command.
    let record = sim.parse_log().records().last().expect("a record");
    assert_eq!(record.input, "turn the sage into powder");
}

#[test]
fn the_trace_records_a_consultation_even_when_the_command_does_not_land() {
    // The augury read the sentence correctly and the command was refused —
    // `grind` is the mortar's word and nobody is standing at the mortar. That
    // row is the most interesting one in an export, because it is where either
    // the model or the room was wrong, and deriving `divined` from the
    // confidence would have dropped it silently.
    let mut sim = Sim::new(0);
    sim.submit_reading("turn the sage into powder", &Fixture::worked());

    assert!(
        divined(&sim),
        "a refused reading forgot it had been divined"
    );
    assert_eq!(echo(&sim).as_deref(), Some("grind"));
}

#[test]
fn a_reader_that_abstains_leaves_the_game_exactly_as_it_was() {
    // §6 forbids a bare error, and abstaining must reach the same suggestions a
    // tower with no reader at all would.
    let mut with = Sim::new(0);
    with.submit_reading("xyzzy plugh", &Silent);

    let mut without = Sim::new(0);
    without.submit("xyzzy plugh");

    assert!(!divined(&with));
    assert_eq!(
        with.parse_log()
            .records()
            .last()
            .map(|r| r.suggestions.clone()),
        without
            .parse_log()
            .records()
            .last()
            .map(|r| r.suggestions.clone()),
    );
}

#[test]
fn a_reader_cannot_change_a_reading_the_orb_was_sure_of() {
    // **The additive claim, stated as a property rather than an argument.**
    // Every synonym of every verb, submitted both ways, with a reader standing
    // by that would answer differently — and wherever the orb read the line
    // outright, it reads it identically.
    //
    // **Not every synonym qualifies, and that is the router working.** Bare
    // `edit` reaches `verify` only because `edit` is two edits from `audit` and
    // scores exactly `MIN_SIMILARITY`; the exact `scribe` reading loses because
    // its `Name` slot is free text and cannot be enumerated. A 600 is a guess,
    // and a guess is precisely what a reader is allowed to improve on. Nothing
    // is lost either way: a reader that abstains falls through to the same
    // `verify`, so §19's *"a released word does not stop resolving"* holds.
    let mut offered = 0;
    for verb in parser::Verb::ALL {
        for (_, phrase) in parser::synonyms_of(verb) {
            let mut plain = Sim::new(0);
            plain.submit(&phrase);

            let mut watched = Sim::new(0);
            let sure =
                parser::analyse(&phrase, watched.scene(), parser::Mode::Calm).reads_outright();
            watched.submit_reading(&phrase, &Trap);

            if sure {
                assert_eq!(
                    echo(&watched),
                    echo(&plain),
                    "{phrase:?} read differently with a reader present"
                );
                assert!(!divined(&watched), "{phrase:?} was handed to a reader");
            } else {
                offered += 1;
            }
        }
    }

    // A canary rather than a threshold: if this ever reached most of the
    // vocabulary, the router would have stopped deferring to the matcher and
    // this test would still be green without it.
    assert!(
        offered * 4 < parser::SYNONYMS.len(),
        "{offered} of {} synonyms are being offered to a reader",
        parser::SYNONYMS.len()
    );
}

#[test]
fn a_word_the_orb_only_guesses_at_still_resolves_when_no_reader_answers() {
    // The other half of the case above. `edit` is a shipped synonym, and §19's
    // rule is that a released word does not stop resolving — so the fallback
    // has to reach exactly where it always did.
    let mut plain = Sim::new(0);
    plain.submit("edit");

    let mut silent = Sim::new(0);
    silent.submit_reading("edit", &Silent);

    assert_eq!(echo(&silent), echo(&plain));
}

#[test]
fn the_first_reading_that_runs_is_the_one_taken() {
    // **Why the seam returns a list at all.** A reader cannot see the world, so
    // it offers what the sentence might mean and the room decides. Taking the
    // head of the list on faith would run nothing and fall through to §6's
    // suggestions, which is the behaviour this replaced.
    let mut sim = Sim::new(0);
    sim.submit_reading("what is about the place", &Fussy);

    assert!(divined(&sim), "the reader was not consulted");
    assert_eq!(echo(&sim).as_deref(), Some("survey"));
}

#[test]
fn a_reader_whose_readings_all_fail_leaves_the_game_as_it_was() {
    // Offering four commands that cannot run is not better than offering none.
    struct Hopeless;
    impl Augur for Hopeless {
        fn read(&self, _line: &str) -> Vec<String> {
            vec![
                String::from("attend nowhere-at-all"),
                String::from("peruse nothing.log"),
            ]
        }
    }

    let mut with = Sim::new(0);
    with.submit_reading("xyzzy plugh", &Hopeless);

    let mut without = Sim::new(0);
    without.submit("xyzzy plugh");

    assert!(
        !divined(&with),
        "a reading that cannot run was taken anyway"
    );
    assert_eq!(echo(&with), echo(&without));
}

#[test]
fn a_divined_reading_never_raises_a_numbered_prompt() {
    // "It acts, and never stops to ask." A reader whose command is ambiguous
    // drops rather than interrogating — the interrogation is what the augury
    // exists to remove.
    let augur = Fixture::new().reading("get rid of things", "purge");
    let mut sim = Sim::new(0);
    sim.submit_reading("get rid of things", &augur);

    assert!(
        sim.choices().asked().is_none(),
        "a divined reading opened a numbered prompt"
    );
}
