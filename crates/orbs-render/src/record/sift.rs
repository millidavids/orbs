//! Filtering records — what `sift` and a pipe stage actually do.
//!
//! DESIGN.md §7: *"Pipes and `grep` operate on records, never on rendered
//! text."* The distinction is easy to nod at and easy to lose, so it is worth
//! being exact about what "rendered text" means here. Matching runs against a
//! field's **own value**. It never sees:
//!
//! - column padding, alignment, or a truncated tail — a match must not depend on
//!   how wide the pane happened to be;
//! - box drawing, headers, or any other structure a view added;
//! - eldritch corruption — §3 keeps the model faithful and corrupts only the
//!   rendering, so a sifted term finds the message the orb *meant* even while
//!   the tube is showing it damaged.
//!
//! The last one is why the design calls this *"the only model that survives the
//! eldritch renderer corrupting output"*. Search over rendered text would go
//! blind exactly when threat is highest.
//!
//! The authored linear variant is deliberately not searched either: it is a
//! rendering of the record for a screen reader, so matching it would make the
//! result set differ between players.

use crate::record::field::FieldName;
use crate::record::stream::{Record, Records};

/// A filter over records.
///
/// The field restriction is a capability of the *model*, not a player-facing
/// flag — §6's verb table gives `sift` the signature `<pattern> <source>` and
/// nothing more. Views and the balance harness need to ask "does this record's
/// state field say spoiled"; whether a player ever gets a flag for it is a
/// Phase 1 decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sift<'a> {
    pattern: &'a str,
    field: Option<FieldName>,
}

impl<'a> Sift<'a> {
    /// Match `pattern` anywhere in any field.
    ///
    /// The pattern is kept **verbatim**, punctuation and case intact. §6 tokenises
    /// player input into a matching form and a raw form precisely so a search
    /// term reaches this point unmangled; `sift ERROR feed.log` searching for
    /// `error` was a real defect and is recorded as one in §19.
    #[must_use]
    pub const fn new(pattern: &'a str) -> Self {
        Self {
            pattern,
            field: None,
        }
    }

    /// Restrict matching to one field.
    #[must_use]
    pub const fn in_field(self, field: FieldName) -> Self {
        Self {
            field: Some(field),
            ..self
        }
    }

    /// The pattern as the player typed it.
    #[must_use]
    pub const fn pattern(&self) -> &'a str {
        self.pattern
    }

    /// Which field this is restricted to, if any.
    #[must_use]
    pub const fn field(&self) -> Option<FieldName> {
        self.field
    }
}

impl Record<'_> {
    /// Whether this record satisfies `sift`.
    ///
    /// Case-insensitive over ASCII. §6's whole posture is that phrasing should
    /// not be a puzzle, and a player who types `sift SPOILED` and is told the
    /// laboratory is clean has been lied to by a technicality. Case folding is
    /// limited to ASCII on purpose: the CP437 repertoire's accented glyphs have
    /// no single correct fold, and guessing would make matches depend on which
    /// glyph an author reached for.
    #[must_use]
    pub fn matches(&self, sift: &Sift<'_>) -> bool {
        self.fields()
            .filter(|(name, _)| sift.field().is_none_or(|wanted| *name == wanted))
            .any(|(_, value)| value.with_str(|text| contains_ignoring_case(text, sift.pattern())))
    }
}

impl Records {
    /// Every record satisfying `sift`, in emit order.
    ///
    /// The output of a pipe stage. Returning an iterator rather than a
    /// collection is what lets stages compose without a `Vec` per stage — and
    /// the result is still `Clone`, so a view can measure it and then draw it.
    pub fn sift<'a>(&'a self, sift: &'a Sift<'a>) -> impl Iterator<Item = Record<'a>> + Clone {
        self.iter().filter(move |record| record.matches(sift))
    }
}

/// Substring search, folding ASCII case.
///
/// Byte windows are safe on UTF-8 here: a continuation byte is always `>= 0x80`
/// and can never equal an ASCII byte, so a match can neither straddle nor split
/// a character boundary.
fn contains_ignoring_case(haystack: &str, needle: &str) -> bool {
    let (haystack, needle) = (haystack.as_bytes(), needle.as_bytes());
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_folds_but_only_over_ascii() {
        assert!(contains_ignoring_case("SPOILED", "spoiled"));
        assert!(contains_ignoring_case("the ward HOLDS", "holds"));
        // `É` and `é` are distinct code points; folding them would be a guess.
        assert!(!contains_ignoring_case("Élan", "élan"));
    }

    #[test]
    fn a_multibyte_haystack_never_splits_a_character() {
        // `░` is 0xE2 0x96 0x91 — no byte of it can equal an ASCII byte.
        assert!(contains_ignoring_case("░░ready░░", "ready"));
        assert!(!contains_ignoring_case("░░░", "a"));
    }

    #[test]
    fn an_empty_pattern_matches_everything() {
        // `sift "" feed.log` is the identity stage, which is what a player who
        // has not finished typing has typed.
        assert!(contains_ignoring_case("anything", ""));
    }
}
