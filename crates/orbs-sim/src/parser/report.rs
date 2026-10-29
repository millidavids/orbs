//! The parser's answer, as records.
//!
//! Rule 4: everything downstream — the pane, a screen reader, `sift`, the
//! harness — reads these, never a rendered string. No prose: only facts already
//! in the [`Resolution`], for a content file to compose around (rule 6).
//!
//! [`FieldName::Outcome`] is not decoration: a numbered prompt is selectable
//! where a suggestion list is not, and the four look alike as text.
//!
//! | Resolution | Records | `Outcome` |
//! |---|---|---|
//! | `Resolved` | one — what will run | [`Outcome::Resolved`], or [`Outcome::Forced`] under siege |
//! | `Incomplete` | one — the command so far, plus the category of the empty slot | [`Outcome::Incomplete`] |
//! | `TakesNothing` | one — the verb, and a sentence naming the words it could not use | [`Outcome::Unresolved`] |
//! | `Ambiguous` | one per tied reading, best first | [`Outcome::Candidate`] |
//! | `Unresolved` | one naming the input, then one per suggestion | [`Outcome::Unresolved`], then [`Outcome::Suggestion`] |
//!
//! The echo is arcane whatever was typed (§6), and tagged
//! [`RecordKind::Echo`] so a reader can suppress it (§14).

use orbs_render::{FieldName, Outcome, RecordKind, Records};

use super::intent::{Confidence, Resolution};
use crate::content::Prose;

/// Write `resolution` to `records`.
///
/// `input` is the raw line, which [`Resolution::Unresolved`] does not carry
/// and the orb must name. Always at least one record (§6). Roles stay
/// [`Role::Normal`](orbs_render::Role): the accent triad is danger, cost and
/// success (§4), and a parser needing a word is none of them. [`Prose`] is for
/// the outcomes that need an authored sentence.
pub fn report(input: &str, resolution: &Resolution, prose: &Prose, records: &mut Records) {
    match resolution {
        Resolution::Resolved { intent, confidence } => {
            // `Divined` draws as `Forced` does: one fact, one marker.
            let outcome = match confidence {
                Confidence::Clear => Outcome::Resolved,
                Confidence::Forced | Confidence::Divined => Outcome::Forced,
            };
            records
                .push(RecordKind::Echo)
                .outcome(outcome)
                .text(FieldName::Message, &intent.echo())
                .finish();
        }
        Resolution::Incomplete {
            verb,
            missing,
            filled,
            ..
        } => {
            // The command as far as it got, in leaves as the echo is, so the
            // prompt shows the player their own words back.
            let mut sofar = String::from(verb.canonical());
            for argument in filled {
                sofar.push(' ');
                sofar.push_str(argument.display());
            }
            records
                .push(RecordKind::Echo)
                .outcome(Outcome::Incomplete)
                .text(FieldName::Message, &sofar)
                .text(FieldName::Kind, missing.label())
                .finish();
        }
        // A verb that takes nothing, handed words anyway: the verb as a fact,
        // the sentence the content's. Drawn as `Unresolved`, since nothing ran.
        Resolution::TakesNothing { verb, extra, .. } => {
            // Or takes only a closed few, which it names (`Verb::narrows`).
            let (key, only) = verb
                .narrows()
                .map_or(("verb_takes_nothing", String::new()), |only| {
                    ("verb_takes_only", only.join(" or "))
                });
            let message = prose.line(
                key,
                &[
                    ("name", verb.canonical()),
                    ("words", extra),
                    ("detail", &only),
                ],
            );
            records
                .push(RecordKind::Echo)
                .outcome(Outcome::Unresolved)
                .text(FieldName::Name, verb.canonical())
                .text(FieldName::Message, &message)
                .finish();
        }
        Resolution::Ambiguous { candidates } if !candidates.is_empty() => {
            // Numbered from one, as a fact about the option: a view drawing a
            // window onto the scrollback may not hold the whole list.
            for (index, candidate) in candidates.iter().enumerate() {
                let choice = u64::try_from(index + 1).unwrap_or(u64::MAX);
                records
                    .push(RecordKind::Echo)
                    .outcome(Outcome::Candidate)
                    .count(FieldName::Choice, choice)
                    .text(FieldName::Message, &candidate.intent.echo())
                    .finish();
            }
        }
        // An empty `Ambiguous` is a parser defect; the `Unresolved` floor keeps
        // "at least one record" true without a fifth outcome.
        Resolution::Ambiguous { .. } => unresolved(input, records),
        // A real verb, in a room that does not answer to it: the verb as a
        // fact, the sentence from `content/prose.toml`.
        Resolution::Elsewhere { verb } => {
            // And the fixture, which tells the player where to go — from
            // `tower::fixture_of`, so a new domain reads right on arrival.
            let (key, fixture) = match crate::tower::fixture_of(*verb) {
                Some(fixture) => ("verb_needs_fixture", fixture),
                None => ("verb_elsewhere", ""),
            };
            let message = prose.line(key, &[("name", verb.canonical()), ("source", fixture)]);
            let mut record = records
                .push(RecordKind::Echo)
                .outcome(Outcome::Unresolved)
                .text(FieldName::Name, verb.canonical())
                .text(FieldName::Message, &message);
            if !fixture.is_empty() {
                record = record.text(FieldName::Source, fixture);
            }
            record.finish();
        }
        // A word the editor taught them, typed here. Naming it and saying where
        // it belongs — §6 forbids a bare error, and this word is one the game
        // itself put in their hands.
        Resolution::InSpell { word } => {
            let message = prose.line("word_in_spell", &[("name", word.canonical())]);
            records
                .push(RecordKind::Echo)
                .outcome(Outcome::Unresolved)
                .text(FieldName::Name, word.canonical())
                .text(FieldName::Message, &message)
                .finish();
        }
        Resolution::Unresolved { suggestions } => {
            unresolved(input, records);
            for verb in suggestions {
                records
                    .push(RecordKind::Echo)
                    .outcome(Outcome::Suggestion)
                    .text(FieldName::Message, verb.canonical())
                    .finish();
            }
        }
    }
}

/// The floor: nothing resolved, and here is the word that did not.
fn unresolved(input: &str, records: &mut Records) {
    records
        .push(RecordKind::Echo)
        .outcome(Outcome::Unresolved)
        .text(FieldName::Message, input)
        .finish();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Intent, Mode, Register, Scene, Verb, resolve};
    use orbs_render::Value;

    fn records_for(input: &str, scene: &Scene) -> Records {
        let mut records = Records::new();
        report(
            input,
            &resolve(input, scene, Mode::Calm),
            &Prose::default(),
            &mut records,
        );
        records
    }

    /// Every record's `(outcome, message)` pair.
    fn outcomes(records: &Records) -> Vec<(String, String)> {
        records
            .iter()
            .map(|record| {
                let text = |name| {
                    record
                        .field(name)
                        .map_or_else(String::new, |value: Value<'_>| {
                            value.with_str(str::to_owned)
                        })
                };
                (text(FieldName::Outcome), text(FieldName::Message))
            })
            .collect()
    }

    #[test]
    fn a_resolved_command_echoes_its_canonical_arcane_form() {
        let scene = Scene::default();
        let records = records_for("look around", &scene);

        assert_eq!(records.len(), 1);
        assert_eq!(records.get(0).expect("echo").kind(), RecordKind::Echo);
        assert_eq!(
            outcomes(&records),
            [(Outcome::Resolved.as_str().to_owned(), "survey".to_owned())],
        );
    }

    #[test]
    fn a_forced_reading_is_flagged_so_a_view_can_offer_correction() {
        // A siege never blocks (§6), so the correction rides the echo and the
        // flag must survive the record. Built by hand, so a moved synonym
        // cannot stop this exercising `Forced`.
        let intent = Intent {
            verb: Verb::Survey,
            register: Register::Arcane,
            arguments: Vec::new(),
        };

        for (confidence, expected) in [
            (Confidence::Clear, Outcome::Resolved),
            (Confidence::Forced, Outcome::Forced),
        ] {
            let mut records = Records::new();
            report(
                "look around",
                &Resolution::Resolved {
                    intent: intent.clone(),
                    confidence,
                },
                &Prose::default(),
                &mut records,
            );
            assert_eq!(
                outcomes(&records),
                [(expected.as_str().to_owned(), "survey".to_owned())],
                "{confidence:?}",
            );
        }
    }

    #[test]
    fn an_empty_slot_names_the_category_that_would_fill_it() {
        // Never a bare error (§6), and never the word `Fragment` either.
        let records = records_for("meditate", &Scene::default());

        assert_eq!(
            outcomes(&records),
            [(
                Outcome::Incomplete.as_str().to_owned(),
                "meditate".to_owned()
            )],
        );
        assert_eq!(
            records.get(0).expect("echo").field(FieldName::Kind),
            Some(Value::Text("count")),
        );
    }

    #[test]
    fn a_failure_never_looks_like_a_success() {
        // All three carry one canonical form in `Message`; only the outcome
        // tells a command that will run from one that could not be read.
        let scene = Scene::default();
        let good = records_for("look around", &scene);
        let bad = records_for("xyzzy", &scene);

        assert_ne!(
            good.get(0).expect("echo").field(FieldName::Outcome),
            bad.get(0).expect("echo").field(FieldName::Outcome),
        );
        assert_eq!(
            bad.get(0).expect("echo").field(FieldName::Outcome),
            Some(Value::Text(Outcome::Unresolved.as_str())),
        );
    }

    #[test]
    fn a_suggestion_is_never_mistaken_for_a_selectable_candidate() {
        // §6 numbers ambiguous readings and has the player pick one. A "did you
        // mean" list is not selectable, and a view drawing numbers beside it
        // would promise an interaction that does nothing.
        let records = records_for("xyzzy", &Scene::default());
        for (outcome, _) in outcomes(&records).iter().skip(1) {
            assert_eq!(outcome, Outcome::Suggestion.as_str());
        }
        assert!(
            outcomes(&records)
                .iter()
                .all(|(outcome, _)| outcome != Outcome::Candidate.as_str()),
        );
    }

    #[test]
    fn nothing_at_all_still_leaves_a_record() {
        // §6 forbids a bare error, and silence is worse than one. An input that
        // resolves to nothing *and* suggests nothing must still say so.
        let scene = Scene::default();
        for input in ["", "   ", "xyzzy", "!!!!"] {
            let records = records_for(input, &scene);
            assert!(!records.is_empty(), "{input:?} produced silence");
            assert_eq!(
                records.get(0).expect("first").field(FieldName::Message),
                Some(Value::Text(input)),
                "{input:?} did not name what failed",
            );
        }
    }

    #[test]
    fn an_outcome_is_never_read_aloud() {
        // It is a machine annotation. A reader hearing "outcome: forced" has
        // been read an internal token; the correction affordance is the view's
        // job to draw from the same field.
        let scene = Scene::default();
        for input in ["look around", "meditate", "xyzzy"] {
            for record in records_for(input, &scene).iter() {
                let spoken = record.to_speech();
                for outcome in Outcome::ALL {
                    assert!(
                        !spoken.contains(outcome.as_str()),
                        "{input:?} spoke {outcome:?}"
                    );
                }
                assert!(!spoken.contains("outcome"), "{input:?}: {spoken:?}");
            }
        }
    }

    #[test]
    fn every_record_speaks_as_metadata_a_reader_can_suppress() {
        // §14 tags the echo as metadata precisely so verbosity settings can drop
        // it without dropping output. That works off the utterance kind.
        let scene = Scene::default();
        for input in ["look around", "meditate", "xyzzy"] {
            for record in records_for(input, &scene).iter() {
                assert_eq!(
                    record.kind().utterance(),
                    orbs_render::UtteranceKind::Echo,
                    "{input}",
                );
            }
        }
    }
}
