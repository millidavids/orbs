//! What the parser decided, and how sure it was.
//!
//! DESIGN.md §6 closes with *"the parser must explain itself."* That is why a
//! [`Candidate`] carries its component scores rather than one opaque number: the
//! Phase 0 gate needs to act on the *clustering* of failures, which means knowing
//! whether a miss was the verb or the argument.

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

impl Argument {
    /// The value as the echo shows it.
    ///
    /// **Only a place is shortened.** Applying [`leaf`] to every argument
    /// truncated free text at a slash: `sift march/north laboratory.log` echoed
    /// `sift north laboratory.log`, teaching a command that searches for a
    /// different string than the one that ran. `arguments.rs` states the
    /// invariant this broke — *"free text: whatever was typed, in the case and
    /// punctuation they typed it in"* — and the echo is what §6 says players
    /// learn the vocabulary from.
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
    /// **Compacted**, so an unfilled optional slot leaves no gap — read a
    /// specific slot with [`slot`](Self::slot) rather than by index.
    pub arguments: Vec<Argument>,
}

impl Intent {
    /// The argument that filled signature slot `index`, if any.
    ///
    /// `arguments` drops the `None`s that `Filled::slots` deliberately keeps —
    /// *"positional rather than compacted, so a later slot resolving while an
    /// earlier one does not cannot silently renumber the arguments"* — and
    /// `execute::carry` then rebuilt the mapping by matching on **slice length**:
    /// two arguments meant the source was skipped, three meant it was named.
    ///
    /// That reintroduced the renumbering hazard one layer up, and it held only by
    /// accident: `move` is the sole signature with an optional slot, and it
    /// happens to sit between two required ones. The next verb with an optional
    /// argument would inherit an arity match that is not equivalent to a
    /// positional read, and the mis-mapping is silent — `move` would hand the
    /// destination to the source.
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
/// Places resolve to full paths, so a three-argument `move` echoed
/// `move charcoal /tower/laboratory/dispensary /tower/laboratory/athanor` — 74
/// characters against the ~60 a pane gives at the 80×22 floor, and the
/// destination was simply **clipped off** the one line whose job is saying where
/// a thing went.
///
/// The leaf is also what the player typed and what §7 says they say: *"a place
/// answers to its full path; §6's matcher also accepts the last segment, which
/// is what makes `attend laboratory` reach `/tower/laboratory` — players say the
/// place, not the path."*
///
/// **Only the last segment**, never a middle one. `score_against`
/// (`super::scene`) matches a phrase against the full name or the leaf and
/// nothing between, so echoing `laboratory/alembic` would teach a form the
/// parser rejects — and the echo teaching a typeable command is the single thing
/// §6 asks of it. Leaves staying unique is enforced by
/// `every_place_leaf_is_unique`, not hoped for.
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
    /// The verb is known but a required argument is not, and the parser cannot
    /// offer a list because the slot takes free text or a number.
    ///
    /// §6 forbids a bare error, and the alternative here is worse than one: a
    /// verb that matched at full score falling through to
    /// [`Resolution::Unresolved`] makes the orb answer "I do not know that word"
    /// and then suggest the word just typed.
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
    /// A real verb, but not one this place answers to.
    ///
    /// §7 scopes an instrument's verb to where the instrument is, so `mix` in
    /// the archive resolves to nothing — and *"I do not know that word"* would be
    /// a lie about a word the game taught the player in the room next door.
    /// §6 forbids a bare error and this is the same rule one step further in:
    /// the useful answer names the verb and says where it is not.
    Elsewhere {
        /// The verb they meant.
        verb: Verb,
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
            | Self::Elsewhere { .. }
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
