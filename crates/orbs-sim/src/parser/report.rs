//! The parser's answer, as records.
//!
//! The first producer on the boundary architectural rule 4 describes:
//! *commands emit records; presentation is a view over the record.* Everything
//! downstream — the scrollback pane, a screen reader, `sift`, the balance
//! harness — reads what this writes, and none of them reads a rendered string.
//!
//! # No prose lives here
//!
//! Rule 6 and §12's second mitigation put all authored prose in hot-reloadable
//! content files, never in Rust. So this module emits **only facts already in
//! the [`Resolution`]** — a canonical command form, a verb name, the category a
//! slot wants. The sentence wrapped around them (*"did you mean"*, *"I need a
//! file"*) is composed by a content file in Phase 1, from these fields.
//!
//! # Every record says which of these it is
//!
//! [`FieldName::Outcome`] is not decoration. A numbered disambiguation prompt is
//! **selectable** — §6 has the player answer it with a number — and a suggestion
//! list is not; a forced echo needs a correction affordance and a clear one does
//! not. All four look alike as text, so a view that had only the text could not
//! draw any of those differences.
//!
//! | Resolution | Records | `Outcome` |
//! |---|---|---|
//! | `Resolved` | one — what will run | [`Outcome::Resolved`], or [`Outcome::Forced`] under siege |
//! | `Incomplete` | one — the command so far, plus the category of the empty slot | [`Outcome::Incomplete`] |
//! | `Ambiguous` | one per tied reading, best first | [`Outcome::Candidate`] |
//! | `Unresolved` | one naming the input, then one per suggestion | [`Outcome::Unresolved`], then [`Outcome::Suggestion`] |
//!
//! §6 makes the echo canonical *arcane* whatever register was typed, because the
//! canonical form is the one players absorb. §14 requires it be tagged as
//! metadata so a reader can suppress it without losing output — which is what
//! [`RecordKind::Echo`] does, via
//! [`UtteranceKind::Echo`](orbs_render::UtteranceKind).

use orbs_render::{FieldName, Outcome, RecordKind, Records};

use super::intent::{Confidence, Resolution};

/// Write `resolution` to `records`.
///
/// `input` is the raw line the player typed. [`Resolution::Unresolved`] does not
/// carry it and the orb cannot say *"I do not know that word"* without naming
/// the word, so it is threaded in rather than reconstructed.
///
/// Always emits **at least one record**: §6 forbids a bare error, and a parser
/// that resolved nothing and suggested nothing would otherwise produce silence —
/// the one outcome the design rules out.
///
/// Roles stay [`Role::Normal`](orbs_render::Role) throughout. The accent triad
/// is danger, cost, and success (§4), and a parser needing one more word is none
/// of those — spending an accent here would dilute the three signals that have
/// to read instantly during a siege.
pub fn report(input: &str, resolution: &Resolution, records: &mut Records) {
    match resolution {
        Resolution::Resolved { intent, confidence } => {
            let outcome = match confidence {
                Confidence::Clear => Outcome::Resolved,
                Confidence::Forced => Outcome::Forced,
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
            // The command as far as it got, so the prompt can show the player
            // their own words back rather than starting over. §19 records the
            // defect where rebuilding this from scratch dropped resolved slots.
            let mut sofar = String::from(verb.canonical());
            for argument in filled {
                sofar.push(' ');
                sofar.push_str(&argument.value);
            }
            records
                .push(RecordKind::Echo)
                .outcome(Outcome::Incomplete)
                .text(FieldName::Message, &sofar)
                .text(FieldName::Kind, missing.label())
                .finish();
        }
        Resolution::Ambiguous { candidates } if !candidates.is_empty() => {
            for candidate in candidates {
                records
                    .push(RecordKind::Echo)
                    .outcome(Outcome::Candidate)
                    .text(FieldName::Message, &candidate.intent.echo())
                    .finish();
            }
        }
        // An `Ambiguous` with nothing in it is a parser defect, not a state the
        // player can reach on purpose. Falling through to the same floor as
        // `Unresolved` keeps the "always at least one record" guarantee true
        // without inventing a fifth outcome for a bug.
        Resolution::Ambiguous { .. } => unresolved(input, records),
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
        report(input, &resolve(input, scene, Mode::Calm), &mut records);
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
        // §6: a siege never blocks on a prompt, so the correction moves into the
        // echo. A view cannot draw that affordance from the text alone, so the
        // flag has to survive the trip through the record.
        //
        // Built directly rather than resolved from input: whether any *current*
        // phrase happens to tie under siege is a property of the vocabulary, and
        // a test that silently stops exercising `Forced` when a synonym moves is
        // a test that asserts nothing.
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
        // The defect this outcome field exists for: `Resolved`, `Ambiguous` and
        // `Unresolved` all carry one canonical form in `Message`, so without the
        // annotation a view could not tell a command that will run from one the
        // parser could not read at all.
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
