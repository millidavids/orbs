//! A [`Scrivener`] built from a table, for the surfaces a model cannot reach.
//!
//! [`Fixture`](super::Fixture)'s sibling, and it exists for the same reason:
//! `ORBS_DUMP` builds no `App`, `scripts/dumps.sh` captures surfaces as text, and
//! a trained reader's weights are a gitignored build artefact that changes on
//! every training run. A capture made against one could not be reproduced from a
//! clean checkout, so the *fixed* reader is what a capture pins — and CLAUDE.md
//! names the alternative precisely: *"a domain built without a block in it is one
//! this instrument is blind to, and the blindness looks exactly like stability."*

use super::Scrivener;

/// A scrivener that reads exactly the lines it was told about.
#[derive(Debug, Default, Clone)]
pub struct Copyist {
    readings: Vec<(String, String)>,
}

impl Copyist {
    /// A reader that knows nothing.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            readings: Vec::new(),
        }
    }

    /// Teach it that `line` means `canonical`.
    #[must_use]
    pub fn reading(mut self, line: &str, canonical: &str) -> Self {
        self.readings
            .push((line.trim().to_lowercase(), canonical.to_owned()));
        self
    }

    /// The handful of loose lines the captures pin.
    ///
    /// **Both registers, because that is the claim.** A command line and a
    /// control line are different output spaces — `grind` is a verb and `if` is
    /// not — and a fixture that only covered commands would let the harder half
    /// ship untested.
    ///
    /// Each is a line the deterministic pipeline genuinely cannot read, so a
    /// capture using one exercises the scrivener rather than the parser.
    /// `crush the sage` is deliberately **not** here: `crush` is a `grind`
    /// synonym, so the matcher already reads it and a fixture claiming it would
    /// take credit for work this feature did not do.
    #[must_use]
    pub fn worked() -> Self {
        Self::new()
            .reading("work the sage down", "grind sage")
            .reading("the sage wants crushing", "grind sage")
            .reading("let the ground-sage steep", "digest ground-sage")
            .reading("when the mortar is idle", "if mortar_and_pestle is idle")
            .reading("hang on ten ticks", "bide 10")
            .reading("do that again three times", "repeat 3")
            .reading("that is all", "end")
    }
}

impl Scrivener for Copyist {
    fn read(&self, line: &str) -> Option<String> {
        let wanted = line.trim().to_lowercase();
        self.readings
            .iter()
            .find(|(known, _)| *known == wanted)
            .map(|(_, canonical)| canonical.clone())
    }

    /// Its table, because the table is everything it will ever answer.
    fn identity(&self) -> u64 {
        let table: Vec<String> = self
            .readings
            .iter()
            .flat_map(|(line, canonical)| [line.clone(), canonical.clone()])
            .collect();
        crate::save::fingerprint(&table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_answers_only_what_it_was_told() {
        let copyist = Copyist::worked();
        assert_eq!(
            copyist.read("work the sage down").as_deref(),
            Some("grind sage"),
        );
        // **Abstaining is the common answer**, and a fixture that guessed would
        // be a worse instrument than none: a capture would then pin the guess.
        assert_eq!(copyist.read("grind sage"), None);
        assert_eq!(copyist.read("something else entirely"), None);
    }

    #[test]
    fn it_reads_control_flow_as_well_as_commands() {
        // The half a command-only fixture would leave untested.
        let copyist = Copyist::worked();
        assert_eq!(
            copyist.read("when the mortar is idle").as_deref(),
            Some("if mortar_and_pestle is idle"),
        );
        assert_eq!(
            copyist.read("hang on ten ticks").as_deref(),
            Some("bide 10")
        );
    }
}
