//! The words a save spells its enums with.
//!
//! Every one of these is an **explicit table with a round-trip test**, never a
//! `#[derive(Serialize)]` on the enum itself and never an index. Two reasons,
//! and the second is the one that bites:
//!
//! - A derive spells a variant with its Rust name, so renaming `Reagent` in the
//!   parser would silently invalidate every save on disk. The names here are a
//!   *format*, and they are allowed to disagree with the code.
//! - [`NounKind::label`](crate::parser::NounKind::label) already exists and is
//!   **not injective** — `File` and `Readable` both say `"file"`, `Place`,
//!   `Stoppable` and `Workable` all say `"place"`. It is a word for a player,
//!   not an identity, and reading a save back through it would quietly change
//!   what a node is.
//!
//! [`Presentation`] gets the same treatment for a different reason: it lives in
//! `orbs-render`, whose `[dependencies]` is deliberately empty, so it cannot
//! derive `Serialize` and a mapping has to live somewhere. Here is somewhere.
//!
//! `Verb` is the exception that proves the rule: it has a canonical name that is
//! already unique (`canonical_names_are_unique` in `parser::verb`) and already
//! player-facing, so the save spells it the way the player types it.

use orbs_render::{FieldName, Presentation, RecordKind, Role};

use crate::parser::{NounKind, Verb};
use crate::tower::maze::Errand;
use crate::tower::{Mark, Way};

/// Every [`NounKind`], with the word a save spells it with.
///
/// Exhaustive by construction: `every_noun_kind_has_a_word` matches on one and
/// fails to compile if a variant is added without a word here.
const KINDS: [(NounKind, &str); 19] = [
    (NounKind::Place, "place"),
    (NounKind::File, "file"),
    (NounKind::Pattern, "pattern"),
    (NounKind::Topic, "topic"),
    (NounKind::Essence, "essence"),
    (NounKind::Reagent, "reagent"),
    (NounKind::Vessel, "vessel"),
    (NounKind::Scroll, "scroll"),
    (NounKind::Script, "script"),
    (NounKind::Count, "count"),
    (NounKind::Name, "name"),
    (NounKind::Sense, "sense"),
    (NounKind::Readable, "readable"),
    (NounKind::Portable, "portable"),
    (NounKind::Workable, "workable"),
    (NounKind::Stoppable, "stoppable"),
    (NounKind::Command, "command"),
    (NounKind::Any, "any"),
    (NounKind::Subject, "subject"),
];

/// The word for a noun kind.
pub(super) fn kind_word(kind: NounKind) -> &'static str {
    KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map_or("any", |(_, word)| *word)
}

/// The noun kind a word names, if it names one.
pub(super) fn kind_from(word: &str) -> Option<NounKind> {
    KINDS
        .iter()
        .find(|(_, w)| *w == word)
        .map(|(kind, _)| *kind)
}

/// The word for a verb — the one the player types.
pub(super) const fn verb_word(verb: Verb) -> &'static str {
    verb.canonical()
}

/// The verb a canonical name belongs to.
pub(super) fn verb_from(word: &str) -> Option<Verb> {
    Verb::ALL.into_iter().find(|verb| verb.canonical() == word)
}

/// The word for a tonal register.
pub(super) const fn register_word(register: Presentation) -> &'static str {
    match register {
        Presentation::Plain => "plain",
        Presentation::Eldritch => "eldritch",
        Presentation::Tampered => "tampered",
    }
}

/// The register a word names. An unknown one falls back to plain, because a
/// save written by a build with a register this one has never heard of should
/// still open — a face is enrichment, and losing it costs no information.
pub(super) fn register_from(word: &str) -> Presentation {
    match word {
        "eldritch" => Presentation::Eldritch,
        "tampered" => Presentation::Tampered,
        _ => Presentation::Plain,
    }
}

/// The word for a compass bearing — the one a player types after `follow`.
pub(crate) const fn way_word(way: Way) -> &'static str {
    way.word()
}

/// The bearing a word names.
pub(crate) fn way_from(word: &str) -> Option<Way> {
    Way::ALL.into_iter().find(|way| way.word() == word)
}

/// The word for a maze's errand.
pub(crate) const fn errand_word(errand: Errand) -> &'static str {
    match errand {
        Errand::Way => "way",
        Errand::Glean => "glean",
    }
}

/// The errand a word names. An unknown one is the ordinary maze, which is the
/// only errand that can always be finished.
pub(crate) fn errand_from(word: &str) -> Errand {
    match word {
        "glean" => Errand::Glean,
        _ => Errand::Way,
    }
}

/// The word for a rail mark.
pub(super) const fn mark_word(mark: Mark) -> &'static str {
    match mark {
        Mark::Fault => "fault",
        Mark::News => "news",
    }
}

/// The mark a word names, if it names one.
pub(super) fn mark_from(word: &str) -> Option<Mark> {
    match word {
        "fault" => Some(Mark::Fault),
        "news" => Some(Mark::News),
        _ => None,
    }
}

/// The word for a record's kind.
///
/// A record's kind decides how a view draws it and whether §3's corruption
/// exemption applies, so it is identity rather than decoration and gets a table
/// like the rest.
pub(super) const fn record_kind_word(kind: RecordKind) -> &'static str {
    match kind {
        RecordKind::Section => "section",
        RecordKind::Entry => "entry",
        RecordKind::LogLine => "log",
        RecordKind::ScriptLine => "script",
        RecordKind::Schedule => "schedule",
        RecordKind::Status => "status",
        RecordKind::Message => "message",
        RecordKind::Completion => "completion",
        RecordKind::Echo => "echo",
        RecordKind::Input => "input",
    }
}

/// Every record kind.
///
/// **`RecordKind::ALL`, not a copy.** The same trap [`FIELDS`] documents: a
/// hand-written list compiles unchanged when an eleventh kind arrives, and
/// `record_kind_from` then reads it back as `Message` while the round-trip test
/// iterates the stale array and passes.
const RECORD_KINDS: [RecordKind; RecordKind::ALL.len()] = RecordKind::ALL;

/// The kind a word names. An unknown one is a message, which draws plainly and
/// claims no exemption — the safest thing an unrecognised line can be.
pub(super) fn record_kind_from(word: &str) -> RecordKind {
    RECORD_KINDS
        .into_iter()
        .find(|kind| record_kind_word(*kind) == word)
        .unwrap_or(RecordKind::Message)
}

/// The word for a record's role.
pub(super) const fn role_word(role: Role) -> &'static str {
    match role {
        Role::Normal => "normal",
        Role::Danger => "danger",
        Role::Cost => "cost",
        Role::Success => "success",
    }
}

/// Every role.
const ROLES: [Role; 4] = [Role::Normal, Role::Danger, Role::Cost, Role::Success];

/// The role a word names, or the ordinary one.
pub(super) fn role_from(word: &str) -> Role {
    ROLES
        .into_iter()
        .find(|role| role_word(*role) == word)
        .unwrap_or(Role::Normal)
}

/// Every field a record can carry.
///
/// **`FieldName::ALL`, not a copy of it.** A hand-written list here would
/// compile happily when a sixteenth field arrived, `field_from` would return
/// `None` for it, `restore::stream` would drop it silently, and the round-trip
/// test below would iterate the stale array and pass. The one place the list
/// lives is `orbs-render`, and this borrows it.
const FIELDS: [FieldName; FieldName::ALL.len()] = FieldName::ALL;

/// The word for a field.
pub(super) const fn field_word(field: FieldName) -> &'static str {
    field.label()
}

/// The field a word names, if it names one. A field the save does not recognise
/// is dropped rather than guessed at: a line missing one field still reads, and
/// a line carrying a field under the wrong name reads wrong.
pub(super) fn field_from(word: &str) -> Option<FieldName> {
    FIELDS.into_iter().find(|field| field.label() == word)
}

#[cfg(test)]
mod tests {
    use super::*;
    // `Shift` is only spelled in the round-trip guard below; its two halves live
    // in `tower::ward`, beside the fields they serialise.
    use crate::tower::Shift;

    #[test]
    fn every_noun_kind_has_a_word_and_the_word_reads_back() {
        // The match is what makes this exhaustive: adding a variant to
        // `NounKind` fails to compile here until it is given a word above.
        for kind in KINDS.map(|(kind, _)| kind) {
            let word = match kind {
                NounKind::Place
                | NounKind::File
                | NounKind::Pattern
                | NounKind::Topic
                | NounKind::Essence
                | NounKind::Reagent
                | NounKind::Vessel
                | NounKind::Scroll
                | NounKind::Script
                | NounKind::Count
                | NounKind::Name
                | NounKind::Sense
                | NounKind::Readable
                | NounKind::Portable
                | NounKind::Workable
                | NounKind::Stoppable
                | NounKind::Command
                | NounKind::Any
                | NounKind::Subject => kind_word(kind),
            };
            assert_eq!(kind_from(word), Some(kind), "{kind:?} did not read back");
        }
    }

    #[test]
    fn the_kind_words_are_unique() {
        // `NounKind::label` is not injective and that is the whole reason this
        // table exists. If two entries here ever collide, a save would read a
        // node back as the wrong kind of thing.
        let mut words: Vec<&str> = KINDS.iter().map(|(_, word)| *word).collect();
        words.sort_unstable();
        let before = words.len();
        words.dedup();
        assert_eq!(before, words.len(), "two noun kinds share a word");
    }

    #[test]
    fn the_label_a_player_reads_would_not_have_worked() {
        // Pinned rather than merely asserted in a comment: if `label` ever
        // becomes injective this test fails and the table can be deleted.
        assert_eq!(NounKind::File.label(), NounKind::Readable.label());
        assert_ne!(
            kind_word(NounKind::File),
            kind_word(NounKind::Readable),
            "the save's own words must still tell them apart",
        );
    }

    #[test]
    fn every_verb_reads_back_from_its_canonical_name() {
        for verb in Verb::ALL {
            assert_eq!(verb_from(verb_word(verb)), Some(verb), "{verb:?}");
        }
    }

    /// Adding a variant to any of these must not compile until it has a word.
    ///
    /// The arrays above are hand-written where the upstream enum publishes no
    /// `ALL` — and a hand-written array is the trap this whole module exists to
    /// avoid, because it compiles unchanged when a variant arrives and the
    /// reverse lookup then silently returns the fallback. An **exhaustive
    /// match** is what closes it: this fails to build until the new variant is
    /// listed, which is the same instrument
    /// `every_noun_kind_has_a_word_and_the_word_reads_back` uses one screen up.
    #[test]
    fn no_variant_can_arrive_without_a_word() {
        for role in ROLES {
            let word = match role {
                Role::Normal | Role::Danger | Role::Cost | Role::Success => role_word(role),
            };
            assert_eq!(role_from(word), role, "{role:?}");
        }
        for mark in [Mark::Fault, Mark::News] {
            let word = match mark {
                Mark::Fault | Mark::News => mark_word(mark),
            };
            assert_eq!(mark_from(word), Some(mark), "{mark:?}");
        }
        for errand in [Errand::Way, Errand::Glean] {
            let word = match errand {
                Errand::Way | Errand::Glean => errand_word(errand),
            };
            assert_eq!(errand_from(word), errand, "{errand:?}");
        }
        for register in [
            Presentation::Plain,
            Presentation::Eldritch,
            Presentation::Tampered,
        ] {
            let word = match register {
                Presentation::Plain | Presentation::Eldritch | Presentation::Tampered => {
                    register_word(register)
                }
            };
            assert_eq!(register_from(word), register);
        }
        // `Shift` is the ward's, and its two halves live in `tower::ward` beside
        // the fields they serialise — so it is covered here and nowhere else.
        for shift in [Shift::Gained, Shift::Held, Shift::Lost] {
            let word = match shift {
                Shift::Gained | Shift::Held | Shift::Lost => shift.word(),
            };
            assert_eq!(Shift::named(word), Some(shift), "{shift:?}");
        }
    }

    #[test]
    fn the_small_enums_read_back() {
        for way in Way::ALL {
            assert_eq!(way_from(way_word(way)), Some(way), "{way:?}");
        }
        for errand in [Errand::Way, Errand::Glean] {
            assert_eq!(errand_from(errand_word(errand)), errand, "{errand:?}");
        }
        for mark in [Mark::Fault, Mark::News] {
            assert_eq!(mark_from(mark_word(mark)), Some(mark), "{mark:?}");
        }
        for register in [
            Presentation::Plain,
            Presentation::Eldritch,
            Presentation::Tampered,
        ] {
            assert_eq!(register_from(register_word(register)), register);
        }
    }

    #[test]
    fn the_record_vocabulary_reads_back() {
        for kind in RECORD_KINDS {
            assert_eq!(record_kind_from(record_kind_word(kind)), kind, "{kind:?}");
        }
        for role in ROLES {
            assert_eq!(role_from(role_word(role)), role, "{role:?}");
        }
        for field in FIELDS {
            assert_eq!(field_from(field_word(field)), Some(field), "{field:?}");
        }
    }

    #[test]
    fn the_field_labels_are_still_unique() {
        // `field_word` borrows `FieldName::label` instead of restating it, which
        // is only safe while the labels are distinct. `NounKind::label` is the
        // cautionary example three tests up.
        let mut words: Vec<&str> = FIELDS.iter().map(|f| f.label()).collect();
        words.sort_unstable();
        let before = words.len();
        words.dedup();
        assert_eq!(before, words.len(), "two record fields share a label");
    }
}
