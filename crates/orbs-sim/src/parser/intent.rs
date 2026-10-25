//! What the parser decided, and how sure it was.
//!
//! §6: *"the parser must explain itself"*, so a [`Candidate`] carries its
//! component scores — a miss can be told apart as the verb or the argument.

use super::verb::{NounKind, Verb};
use super::vocabulary::Register;

/// One filled argument slot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Argument {
    /// The category the slot wanted.
    pub kind: NounKind,
    /// The canonical name it resolved to, or the raw text for a free-text slot.
    pub value: String,
    /// Which signature slot this filled — see [`Intent::slot`].
    pub slot: usize,
}

// No `anchor` field: §8's bind-by-entity-ID guards a thing that can be
// destroyed and rebuilt, and nothing in the tower is yet. It returns with one.

impl Argument {
    /// The value as the echo shows it.
    ///
    /// Only a place is shortened: free text is echoed as typed, slashes and
    /// all, since the echo is how players learn (§6).
    #[must_use]
    pub fn display(&self) -> &str {
        if self.kind == NounKind::Place {
            leaf(&self.value)
        } else {
            &self.value
        }
    }
}

/// A fully resolved command.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Intent {
    /// What to do.
    pub verb: Verb,
    /// Which dialect the player reached for. Recorded for the gate — this is the
    /// number that says whether the three-register bet paid off.
    pub register: Register,
    /// Filled slots, in signature order.
    ///
    /// Compacted, so an unfilled optional slot leaves no gap — read a specific
    /// slot with [`slot`](Self::slot) rather than by index.
    pub arguments: Vec<Argument>,
}

impl Intent {
    /// The argument that filled signature slot `index`, if any.
    ///
    /// A positional read, never one by slice length: `arguments` is compacted,
    /// so an arity match silently renumbers once an optional slot is skipped.
    #[must_use]
    pub fn slot(&self, index: usize) -> Option<&Argument> {
        self.arguments
            .iter()
            .find(|argument| argument.slot == index)
    }
    /// The canonical arcane form, as the echo shows it.
    ///
    /// §6: whichever register is canonical is the one players absorb, so the
    /// echo always speaks arcane no matter what was typed.
    #[must_use]
    pub fn echo(&self) -> String {
        let mut out = String::from(self.verb.canonical());
        for argument in &self.arguments {
            out.push(' ');
            out.push_str(argument.display());
        }
        out
    }
}

/// A place argument as its last segment.
///
/// What players say (§7), and short enough that a three-place `move` fits the
/// 80×22 pane. Never a middle segment, which the matcher rejects;
/// `every_place_leaf_is_unique` holds the uniqueness this rests on.
#[must_use]
pub fn leaf(value: &str) -> &str {
    value.rsplit('/').next().unwrap_or(value)
}

/// How certain the parser is about what it chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// One candidate won outright.
    Clear,
    /// Candidates were close, and a siege forbade a blocking prompt (§6), so the
    /// best-scoring one was taken. The echo must offer correction.
    Forced,
    /// The orb worked the line out rather than reading it — the augury (§6).
    ///
    /// Renders as [`Forced`](Self::Forced) does — one `≈` for *the orb acted
    /// on its best reading*. What it adds is *why*, for the trace and for the
    /// destructive guard, so an inferred `purge` confirms.
    Divined,
}

impl Confidence {
    /// Whether the orb worked this out rather than reading it outright — one
    /// predicate, so the destructive guard and the trace agree.
    #[must_use]
    pub const fn is_divined(self) -> bool {
        matches!(self, Self::Divined)
    }
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
    /// Words this reading could not account for.
    ///
    /// Carried beside [`argument_score`](Self::argument_score) rather than
    /// folded into it, because the two route differently: see
    /// [`Analysis::reads_outright`](super::Analysis::reads_outright).
    pub leftover: usize,
}

/// Whether a blocking disambiguation prompt is permitted.
///
/// §6: *"Disambiguation never blocks during a siege"*, or ambiguous phrasing
/// costs siege time — the typing pressure §14 forbids.
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
    /// The verb is known but a required argument is not, and the parser cannot
    /// offer a list because the slot takes free text or a number.
    ///
    /// Not [`Resolution::Unresolved`], which would answer "I do not know that
    /// word" and suggest the word just typed.
    Incomplete {
        /// The verb that matched.
        verb: Verb,
        /// Which dialect the player reached for.
        register: Register,
        /// What the empty slot wants.
        missing: NounKind,
        /// Slots that did resolve, in signature order.
        filled: Vec<Argument>,
    },
    /// A verb that takes no argument, handed words it cannot use.
    ///
    /// [`Incomplete`](Self::Incomplete)'s other half, for a verb with no slot
    /// to name: §6 forbids the silent discard, so the orb names the verb and
    /// the words it could not use.
    TakesNothing {
        /// The verb that matched.
        verb: Verb,
        /// Which dialect the player reached for.
        register: Register,
        /// The words it was handed, as the player typed them.
        extra: String,
    },
    /// A real verb, but not one this place answers to.
    ///
    /// §7 scopes an instrument's verb to its room, and *"I do not know that
    /// word"* would lie about a word taught next door.
    Elsewhere {
        /// The verb they meant.
        verb: Verb,
    },
    /// A word that means something in a spell, typed at the prompt.
    ///
    /// [`Elsewhere`](Self::Elsewhere) for the prompt rather than the place.
    /// Guessing was destructive: `wait for the mortar` once opened an empty
    /// `mortar.spell`.
    InSpell {
        /// The word they used.
        word: super::SpellWord,
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
    pub const fn intent(&self) -> Option<&Intent> {
        match self {
            Self::Resolved { intent, .. } => Some(intent),
            Self::Ambiguous { .. }
            | Self::Incomplete { .. }
            | Self::TakesNothing { .. }
            | Self::Elsewhere { .. }
            | Self::InSpell { .. }
            | Self::Unresolved { .. } => None,
        }
    }

    /// Whether this resolution produced something to execute.
    #[must_use]
    pub const fn is_resolved(&self) -> bool {
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
                .enumerate()
                .map(|(slot, (kind, value))| Argument {
                    kind: *kind,
                    slot,
                    value: (*value).to_owned(),
                })
                .collect(),
        }
    }

    #[test]
    fn the_echo_is_arcane_whatever_was_typed() {
        // §6: "you type 'grind the sage', the orb answers
        // 'wield mortar_and_pestle', and months later you are typing 'wield'."
        let plain = intent(
            Verb::Wield,
            Register::Plain,
            &[(NounKind::Place, "mortar_and_pestle")],
        );
        let shell = intent(
            Verb::Sift,
            Register::Shell,
            &[(NounKind::Pattern, "march"), (NounKind::File, "feed.log")],
        );

        assert_eq!(plain.echo(), "wield mortar_and_pestle");
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
