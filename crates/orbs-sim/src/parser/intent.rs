//! What the parser decided, and how sure it was.
//!
//! DESIGN.md §6 closes with *"the parser must explain itself."* That is why a
//! [`Candidate`] carries its component scores rather than one opaque number: the
//! Phase 0 gate needs to act on the *clustering* of failures, which means knowing
//! whether a miss was the verb or the argument.

use super::verb::{NounKind, Verb};
use super::vocabulary::Register;

/// One filled argument slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument {
    /// The category the slot wanted.
    pub kind: NounKind,
    /// The canonical name it resolved to, or the raw text for a free-text slot.
    pub value: String,
}

/// A fully resolved command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intent {
    /// What to do.
    pub verb: Verb,
    /// Which dialect the player reached for. Recorded for the gate — this is the
    /// number that says whether the three-register bet paid off.
    pub register: Register,
    /// Filled slots, in signature order.
    pub arguments: Vec<Argument>,
}

impl Intent {
    /// The canonical arcane form, as the echo shows it.
    ///
    /// §6: whichever register is canonical is the one players absorb, so the
    /// echo always speaks arcane no matter what was typed.
    #[must_use]
    pub fn echo(&self) -> String {
        let mut out = String::from(self.verb.canonical());
        for argument in &self.arguments {
            out.push(' ');
            out.push_str(&argument.value);
        }
        out
    }
}

/// How certain the parser is about what it chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// One candidate won outright.
    Clear,
    /// Candidates were close, and a siege forbade a blocking prompt (§6), so the
    /// best-scoring one was taken. The echo must offer correction.
    Forced,
}

/// A scored interpretation of the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// What this interpretation would do.
    pub intent: Intent,
    /// How well the verb phrase matched.
    pub verb_score: u32,
    /// How well the arguments resolved against the world.
    pub argument_score: u32,
    /// The combined score candidates are ranked by.
    pub score: u32,
}

/// Whether a blocking disambiguation prompt is permitted.
///
/// §6: *"Disambiguation never blocks during a siege."* The numbered prompt is
/// modal and siege ticks advance on wall-clock, so a modal wait would make
/// ambiguous phrasing cost siege time — exactly the typing pressure §14 forbids.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The tower. A prompt may block.
    #[default]
    Calm,
    /// A siege. Take the best candidate and offer correction afterwards.
    Siege,
}

/// What the parser concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// A command to run.
    Resolved {
        /// What to do.
        intent: Intent,
        /// Whether it won outright or was forced by a siege.
        confidence: Confidence,
    },
    /// Several readings scored alike. Calm mode only — a siege forces instead.
    Ambiguous {
        /// The tied readings, best first.
        candidates: Vec<Candidate>,
    },
    /// Nothing scored. Never a bare error (§6) — always something to try.
    Unresolved {
        /// Verbs worth suggesting, best first.
        suggestions: Vec<Verb>,
    },
}

impl Resolution {
    /// The intent, if there is one to run.
    #[must_use]
    pub fn intent(&self) -> Option<&Intent> {
        match self {
            Self::Resolved { intent, .. } => Some(intent),
            Self::Ambiguous { .. } | Self::Unresolved { .. } => None,
        }
    }

    /// Whether this resolution produced something to execute.
    #[must_use]
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intent(verb: Verb, register: Register, values: &[(NounKind, &str)]) -> Intent {
        Intent {
            verb,
            register,
            arguments: values
                .iter()
                .map(|(kind, value)| Argument {
                    kind: *kind,
                    value: (*value).to_owned(),
                })
                .collect(),
        }
    }

    #[test]
    fn the_echo_is_arcane_whatever_was_typed() {
        // §6: "you type 'make a potion of clarity', the orb answers
        // 'decoct --essence=clarity', and months later you are typing 'decoct'."
        let plain = intent(
            Verb::Decoct,
            Register::Plain,
            &[(NounKind::Essence, "clarity")],
        );
        let shell = intent(
            Verb::Sift,
            Register::Shell,
            &[(NounKind::Pattern, "march"), (NounKind::File, "feed.log")],
        );

        assert_eq!(plain.echo(), "decoct clarity");
        assert_eq!(shell.echo(), "sift march feed.log");
    }

    #[test]
    fn an_argumentless_verb_echoes_bare() {
        assert_eq!(intent(Verb::Status, Register::Plain, &[]).echo(), "status");
    }

    #[test]
    fn only_resolved_outcomes_carry_an_intent() {
        let resolved = Resolution::Resolved {
            intent: intent(Verb::Status, Register::Arcane, &[]),
            confidence: Confidence::Clear,
        };
        assert!(resolved.is_resolved());
        assert!(resolved.intent().is_some());

        let unresolved = Resolution::Unresolved {
            suggestions: vec![Verb::Status],
        };
        assert!(!unresolved.is_resolved());
        assert!(unresolved.intent().is_none());
    }

    #[test]
    fn calm_is_the_default_mode() {
        assert_eq!(Mode::default(), Mode::Calm);
    }
}
