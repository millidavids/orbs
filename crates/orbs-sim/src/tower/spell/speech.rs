//! What a spell may say: one rule, asked at cast, in the editor, by the guide
//! and when the line runs, so no two of them can disagree (§19, *The tower's
//! spell*).

use crate::parser::{NounKind, Resolution, Verb, verb_of_word};

/// Whether a script may issue this verb at all.
///
/// Each one refused is a hazard, not nonsense. `execute::is_live` is the wrong
/// instrument: it answers *does this verb work*, which only overlaps.
#[must_use]
pub const fn may_issue(verb: Verb) -> bool {
    !matches!(
        verb,
        // A spell is written *for* a domain; walking would make canonicalising
        // an `if` undecidable.
        Verb::Attend
            // `Sim::step` drains `Skip` in a loop: an hour inside one `step()`.
            | Verb::Meditate
            // Opens the editor, so the player's next keystrokes land in it.
            | Verb::Scribe
            // Command-anchored (§6), with no meaning from a script.
            | Verb::Undo
            | Verb::Bind
            // `unfurl`, `weave` and `wander` take the keyboard; inside a
            // `repeat` they re-seize it faster than Escape gives it back.
            // `follow`, `summon` and `limn` take no keys and stay.
            | Verb::Unfurl
            | Verb::Weave
            | Verb::Wander
            // A bound spell re-casts off the end, so it would close the game on
            // the orb's clock.
            | Verb::Quit
            // Takes the pane and the keys, and a bound spell would reopen it
            // every lap.
            | Verb::Menu
    )
}

/// Whether a spell may say `verb`. One for the whole tower starts and stops
/// spells and nothing else: the work stays the rooms'.
#[must_use]
pub const fn may_say(verb: Verb, for_the_tower: bool) -> bool {
    may_issue(verb) && (!for_the_tower || matches!(verb, Verb::Invoke | Verb::Stop))
}

/// Why a spell may not say a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Refused {
    /// A word no spell may say: see [`may_issue`].
    Forbidden,
    /// A line a tower spell may not say: see [`may_say`].
    TowerOnly,
}

impl Refused {
    /// Why this verb is refused, whatever it names.
    const fn verb(verb: Verb, for_the_tower: bool) -> Option<Self> {
        if !may_issue(verb) {
            Some(Self::Forbidden)
        } else if !may_say(verb, for_the_tower) {
            Some(Self::TowerOnly)
        } else {
            None
        }
    }

    /// Why this reading of a line is refused.
    pub(super) fn reading(resolution: &Resolution, for_the_tower: bool) -> Option<Self> {
        match resolution {
            Resolution::Resolved { intent, .. } => {
                // `stop forge`: a verb it may say, of what it may not touch.
                let stops_a_tool = intent.verb == Verb::Stop
                    && intent
                        .arguments
                        .first()
                        .is_none_or(|argument| argument.kind != NounKind::Script);
                Self::verb(intent.verb, for_the_tower)
                    .or_else(|| (for_the_tower && stops_a_tool).then_some(Self::TowerOnly))
            }
            // A room's verb at `/tower`: nothing is missing, the line is in the
            // wrong spell.
            Resolution::Elsewhere { .. } if for_the_tower => Some(Self::TowerOnly),
            Resolution::Ambiguous { .. }
            | Resolution::Incomplete { .. }
            | Resolution::TakesNothing { .. }
            | Resolution::Elsewhere { .. }
            | Resolution::InSpell { .. }
            | Resolution::Unresolved { .. } => None,
        }
    }

    /// Why a line that cannot be resolved yet is refused, read from its first
    /// word: its arguments are bindings, or a spell not written.
    pub(super) fn opening(line: &str, for_the_tower: bool) -> Option<Self> {
        let verb = line.split_whitespace().next().and_then(verb_of_word)?;
        Self::verb(verb, for_the_tower)
    }

    /// The key at cast, where a complaint has no `{detail}` to fill.
    pub(super) const fn at_cast(self) -> &'static str {
        match self {
            Self::Forbidden => "spell_forbidden_line",
            Self::TowerOnly => "spell_tower_only_line",
        }
    }

    /// The key for a line being read or run, which names the line.
    pub(super) const fn of_line(self) -> &'static str {
        match self {
            Self::Forbidden => "spell_forbidden",
            Self::TowerOnly => "spell_tower_only",
        }
    }
}
