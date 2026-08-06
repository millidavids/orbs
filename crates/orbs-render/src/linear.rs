//! Linearisation — the screen-reader view of a frame.
//!
//! DESIGN.md §14 makes this architectural rather than optional: *"linearised
//! representation for panes, tables, and progress bars; eldritch messages logged
//! and carrying authored linear variants; echo tagged as metadata."* It is built
//! now because it is the part that cannot be retrofitted — reading the cell grid
//! back row by row yields box-drawing characters and column-aligned fragments,
//! not sentences, and no amount of later cleverness recovers the structure that
//! was never recorded.
//!
//! The stream is captured on every frame regardless of whether a screen reader
//! is attached. Making it conditional would mean the path is exercised only by
//! the players least able to report that it broke; capturing always costs a few
//! kilobytes of memcpy per frame into a reused arena, and means every test sees
//! it.
//!
//! # Storage
//!
//! One `String` arena plus a `Vec` of ranges, both cleared and refilled each
//! frame. A frame at the largest supported grid holds a few hundred utterances,
//! so a `String` per node would be a few hundred allocations at 60 Hz for no
//! reason.

use crate::record::Outcome;
use crate::style::Role;

/// What a piece of linearised output *is*, so a reader can pace and filter it.
///
/// Deliberately not `#[non_exhaustive]`, for the same reason as [`Role`]: every
/// consumer is in-workspace, and a missed match arm should be a compile error
/// rather than a silently unspoken line.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UtteranceKind {
    /// Names the region the following utterances belong to — a pane title, a
    /// section break. The structural equivalent of a border, which is itself
    /// silent.
    Heading,
    /// Ordinary prose or a single line of output.
    #[default]
    Text,
    /// One row of a table, already flattened to `label: value` form by the
    /// caller. A reader must never have to reconstruct columns from spacing.
    TableRow,
    /// A meter, spoken as a description rather than drawn as a bar.
    Progress,
    /// The parser's canonical echo (§6). Metadata, and verbosity-gated —
    /// tagging it is what lets a reader suppress it without losing output.
    Echo,
    /// What the player typed.
    Input,
    /// What the orb thinks they are about to type.
    ///
    /// The inline suggestion after the caret, and the candidates Tab offers.
    /// **Spoken, not silent** — the half-typed line is spoken, and §19's
    /// Frame-boundary rule is that a visual constraint must not become an
    /// informational one, so a suggestion a sighted player can see and a reader
    /// cannot would be exactly that. Its own kind so verbosity can drop it:
    /// re-offered every keystroke, it is the most repetitive thing on screen.
    Hint,
    /// A duration-action finishing.
    ///
    /// §14 announces **completions only**: at endgame a player may have ~25
    /// in-flight actions, and announcing progress updates would be unusable.
    Completion,
}

/// One item in the linear stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Utterance<'a> {
    /// What this item is.
    pub kind: UtteranceKind,
    /// What it means.
    ///
    /// Carried separately from the text because colour must never be the sole
    /// carrier of meaning (§14) — a reader with no pixels still learns that a
    /// line reports a breach rather than a completion.
    pub role: Role,
    /// How the command that produced this concluded, if it said.
    ///
    /// Carried for the same reason as [`Utterance::role`], and it is not
    /// optional decoration: at the prompt the difference between "this is what
    /// will run", "I do not know that word", and "you could try this" is
    /// otherwise a **marker glyph and a brightness** — two channels a listener
    /// has neither of. Without this, `xyzzy` linearises as three identical
    /// lines and a screen-reader player cannot tell an error from an offer, nor
    /// a selectable candidate from a command about to execute.
    pub outcome: Option<Outcome>,
    /// The text to speak.
    ///
    /// For eldritch content this is the *authored linear variant*, not the
    /// corrupted visual text (§3).
    pub text: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Node {
    kind: UtteranceKind,
    role: Role,
    outcome: Option<Outcome>,
    start: usize,
    end: usize,
}

/// The linear stream for one frame, in the order content was painted.
#[derive(Debug, Default, Clone)]
pub struct Speech {
    text: String,
    nodes: Vec<Node>,
}

impl Speech {
    /// An empty stream.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every utterance, in paint order.
    pub fn utterances(&self) -> impl Iterator<Item = Utterance<'_>> {
        self.nodes.iter().map(|node| Utterance {
            kind: node.kind,
            role: node.role,
            outcome: node.outcome,
            text: &self.text[node.start..node.end],
        })
    }

    /// How many utterances the frame produced.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the frame produced nothing to speak.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The whole stream as one newline-separated string.
    ///
    /// A diagnostic and test view. A real reader wants [`Speech::utterances`],
    /// because the kinds are what make filtering and pacing possible.
    #[must_use]
    pub fn to_transcript(&self) -> String {
        let mut out = String::with_capacity(self.text.len() + self.nodes.len());
        for utterance in self.utterances() {
            out.push_str(utterance.text);
            out.push('\n');
        }
        out
    }

    /// Append an utterance.
    pub(crate) fn push(&mut self, kind: UtteranceKind, role: Role, text: &str) {
        self.push_with(kind, role, None, text);
    }

    /// Append an utterance carrying how its command concluded.
    pub(crate) fn push_with(
        &mut self,
        kind: UtteranceKind,
        role: Role,
        outcome: Option<Outcome>,
        text: &str,
    ) {
        let start = self.text.len();
        self.text.push_str(text);
        self.nodes.push(Node {
            kind,
            role,
            outcome,
            start,
            end: self.text.len(),
        });
    }

    /// Empty the stream, keeping both allocations for the next frame.
    pub(crate) fn clear(&mut self) {
        self.text.clear();
        self.nodes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Speech {
        let mut speech = Speech::new();
        speech.push(UtteranceKind::Heading, Role::Normal, "laboratory");
        speech.push(UtteranceKind::Text, Role::Danger, "the ward has failed");
        speech.push(UtteranceKind::Completion, Role::Success, "haste decocted");
        speech
    }

    #[test]
    fn utterances_come_back_in_paint_order() {
        let speech = sample();
        let texts: Vec<_> = speech.utterances().map(|u| u.text).collect();
        assert_eq!(
            texts,
            ["laboratory", "the ward has failed", "haste decocted"]
        );
    }

    #[test]
    fn kind_and_role_survive_the_arena() {
        let speech = sample();
        let breach = speech.utterances().nth(1).expect("second utterance");
        assert_eq!(breach.kind, UtteranceKind::Text);
        assert_eq!(breach.role, Role::Danger);
    }

    #[test]
    fn clearing_keeps_capacity_for_the_next_frame() {
        let mut speech = sample();
        let text_capacity = speech.text.capacity();
        let node_capacity = speech.nodes.capacity();

        speech.clear();

        assert!(speech.is_empty());
        assert_eq!(speech.text.capacity(), text_capacity);
        assert_eq!(speech.nodes.capacity(), node_capacity);
    }

    #[test]
    fn empty_utterances_do_not_swallow_their_neighbours() {
        let mut speech = Speech::new();
        speech.push(UtteranceKind::Text, Role::Normal, "before");
        speech.push(UtteranceKind::Text, Role::Normal, "");
        speech.push(UtteranceKind::Text, Role::Normal, "after");

        let texts: Vec<_> = speech.utterances().map(|u| u.text).collect();
        assert_eq!(texts, ["before", "", "after"]);
    }
}
