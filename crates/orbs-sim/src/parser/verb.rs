//! The canonical command set.
//!
//! DESIGN.md §6.1 fixes the Phase 0 vocabulary at sixteen commands, and fixes
//! the naming rule that shapes them: **canonical verbs are one short word,
//! ideally ≤7 characters.** Players graduate to typing the canonical form, so it
//! is what expert players type all day — `auspicate --sign=march` would lose to
//! `grep march` every time, which would punish the exact progression the echo
//! mechanism exists to create.
//!
//! The canonical form is **arcane**. Whichever register is canonical is the one
//! players absorb, so making it arcane means the mastery arc is literally
//! learning to speak as a wizard.

/// What a command slot expects to be filled with.
///
/// This is what lets the parser resolve `clarity` against essences rather than
/// against every noun in the tower, and what lets it reject a plausible-sounding
/// argument in the wrong category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NounKind {
    /// A location in the tower. `attend /tower/alembic`.
    Place,
    /// A readable file. `peruse feed.log`.
    File,
    /// Free text matched against file contents. `sift march feed.log`.
    Pattern,
    /// A manual topic. `grimoire brewing`.
    Topic,
    /// An essence that can be brewed. `decoct clarity`.
    Essence,
    /// A vessel holding a finished brew. `decant alembic`.
    Vessel,
    /// A researchable fragment. `decipher sigil-iv`.
    Fragment,
    /// A script. `invoke night_watch`.
    Script,
    /// A count of ticks. `meditate 30`.
    Count,
    /// Anything nameable — `verify` and `purge` accept any surface.
    Any,
}

/// One argument position in a command's signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    /// What may fill it.
    pub kind: NounKind,
    /// Whether the command is incomplete without it.
    pub required: bool,
}

impl Slot {
    const fn required(kind: NounKind) -> Self {
        Self {
            kind,
            required: true,
        }
    }

    const fn optional(kind: NounKind) -> Self {
        Self {
            kind,
            required: false,
        }
    }
}

// Signatures are named constants rather than inline slice literals: a `&[...]`
// built from `const fn` calls is a temporary and does not get promoted to
// `'static`, so it cannot be returned from `signature()`.
const NOTHING: &[Slot] = &[];
const PLACE: &[Slot] = &[Slot::required(NounKind::Place)];
const PLACE_OPTIONAL: &[Slot] = &[Slot::optional(NounKind::Place)];
const FILE: &[Slot] = &[Slot::required(NounKind::File)];
const PATTERN_AND_FILE: &[Slot] = &[
    Slot::required(NounKind::Pattern),
    Slot::required(NounKind::File),
];
const TOPIC: &[Slot] = &[Slot::required(NounKind::Topic)];
const ANYTHING: &[Slot] = &[Slot::required(NounKind::Any)];
const COUNT: &[Slot] = &[Slot::required(NounKind::Count)];
const ESSENCE: &[Slot] = &[Slot::required(NounKind::Essence)];
const VESSEL: &[Slot] = &[Slot::required(NounKind::Vessel)];
const FRAGMENT: &[Slot] = &[Slot::required(NounKind::Fragment)];
const SCRIPT: &[Slot] = &[Slot::required(NounKind::Script)];

/// A canonical command.
///
/// Deliberately not `#[non_exhaustive]`, for the reason given on
/// [`crate::RngStream`]: every consumer is in-workspace, and adding a verb
/// should force every match — echo, help, the balance harness — to account for
/// it rather than silently falling through to a default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verb {
    /// Move to a place.
    Attend,
    /// List what is here.
    Survey,
    /// Read a file.
    Peruse,
    /// Filter for matches.
    Sift,
    /// Tower overview — the boot report.
    Status,
    /// The in-world manual.
    Grimoire,
    /// Detect tampering.
    Verify,
    /// Revert the last command.
    Undo,
    /// Fast-forward the clock.
    Meditate,
    /// Brew a potion.
    Decoct,
    /// Collect a finished potion.
    Siphon,
    /// Destroy waste or spoilage.
    Purge,
    /// Research a fragment.
    Divine,
    /// Author a script.
    Scribe,
    /// Attach a script to a trigger.
    Bind,
    /// Run a script or spell.
    Invoke,
}

impl Verb {
    /// Every verb in the Phase 0 vocabulary.
    pub const ALL: [Self; 16] = [
        Self::Attend,
        Self::Survey,
        Self::Peruse,
        Self::Sift,
        Self::Status,
        Self::Grimoire,
        Self::Verify,
        Self::Undo,
        Self::Meditate,
        Self::Decoct,
        Self::Siphon,
        Self::Purge,
        Self::Divine,
        Self::Scribe,
        Self::Bind,
        Self::Invoke,
    ];

    /// The longest a canonical verb may be.
    ///
    /// §6.1 wrote the rule as "one short word, ideally ≤7 characters". The
    /// Phase 0 naming pass settled the "ideally" at **8**: `grimoire` and
    /// `meditate` are the two most in-world names in the set and carry the
    /// game's identity, abbreviation covers the typing cost (`grim`, `medit`),
    /// and the two 8-character names that had *no* such defence — `decipher`
    /// and `inscribe` — were shortened instead.
    pub const MAX_CANONICAL_LEN: usize = 8;

    /// The arcane name — what the echo shows and what experts type.
    #[must_use]
    pub const fn canonical(self) -> &'static str {
        match self {
            Self::Attend => "attend",
            Self::Survey => "survey",
            Self::Peruse => "peruse",
            Self::Sift => "sift",
            Self::Status => "status",
            Self::Grimoire => "grimoire",
            Self::Verify => "verify",
            Self::Undo => "undo",
            Self::Meditate => "meditate",
            Self::Decoct => "decoct",
            Self::Siphon => "siphon",
            Self::Purge => "purge",
            Self::Divine => "divine",
            Self::Scribe => "scribe",
            Self::Bind => "bind",
            Self::Invoke => "invoke",
        }
    }

    /// What this command takes, in order.
    #[must_use]
    pub const fn signature(self) -> &'static [Slot] {
        match self {
            Self::Attend => PLACE,
            Self::Survey => PLACE_OPTIONAL,
            Self::Peruse => FILE,
            Self::Sift => PATTERN_AND_FILE,
            Self::Status | Self::Undo => NOTHING,
            Self::Grimoire => TOPIC,
            Self::Verify | Self::Purge => ANYTHING,
            Self::Meditate => COUNT,
            Self::Decoct => ESSENCE,
            Self::Siphon => VESSEL,
            Self::Divine => FRAGMENT,
            Self::Scribe | Self::Bind | Self::Invoke => SCRIPT,
        }
    }

    /// Whether the command destroys something and therefore confirms when the
    /// target is not routine (§6, *Undo*).
    #[must_use]
    pub const fn is_destructive(self) -> bool {
        matches!(self, Self::Purge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_vocabulary_is_the_sixteen_the_design_lists() {
        assert_eq!(Verb::ALL.len(), 16);
    }

    #[test]
    fn canonical_names_obey_the_length_rule() {
        // §6.1: canonical verbs are one short word. The canonical form is what
        // expert players type all day, so length is a real cost.
        for verb in Verb::ALL {
            let name = verb.canonical();
            assert!(
                name.len() <= Verb::MAX_CANONICAL_LEN,
                "{name} is {} characters",
                name.len()
            );
            assert!(!name.contains(' '), "{name} is not one word");
        }
    }

    #[test]
    fn canonical_names_are_unique() {
        let mut names: Vec<_> = Verb::ALL.iter().map(|verb| verb.canonical()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "two verbs share a canonical name");
    }

    #[test]
    fn every_verb_appears_in_all() {
        // Guards against adding a variant and forgetting the table.
        for verb in Verb::ALL {
            assert!(Verb::ALL.contains(&verb));
        }
        assert_eq!(
            Verb::ALL
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            Verb::ALL.len()
        );
    }

    #[test]
    fn commands_taking_no_arguments_have_empty_signatures() {
        assert!(Verb::Status.signature().is_empty());
        assert!(Verb::Undo.signature().is_empty());
    }

    #[test]
    fn sift_takes_a_pattern_then_a_source() {
        let signature = Verb::Sift.signature();
        assert_eq!(signature.len(), 2);
        assert_eq!(signature[0].kind, NounKind::Pattern);
        assert_eq!(signature[1].kind, NounKind::File);
        assert!(signature.iter().all(|slot| slot.required));
    }

    #[test]
    fn survey_works_with_no_argument() {
        let signature = Verb::Survey.signature();
        assert_eq!(signature.len(), 1);
        assert!(!signature[0].required);
    }
}
