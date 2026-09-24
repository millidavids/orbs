//! What a spell can ask about a charm.
//!
//! Declared, never derived from `Role::Reading`. `recall scripting` lists a
//! room's readings from `tower::readings_at`, and asking the marker instead
//! shipped a defect one domain over — the lens taught the archive's words,
//! because the marker covers every reading in the tower rather than a room's.
//!
//! Two words, and the second is what makes the phase's fourth box writable.
//! ROADMAP asks for *"a maintenance spell — `repeat until` keeps a buff alive
//! unattended"*, and a spell holds a rule rather than a memory (§8.1 forbids
//! automation driven by hidden state), so it cannot count ticks since it last
//! imbued: the world has to say *this charm is nearly out*. [`EBBING`] is that
//! word, in the maze's derived-word pattern.
//!
//! Published only while true. `spell::watch` answers `is empty` by asking
//! whether a node has children, and `many_at` answers an absent reading with
//! nought — so a tool that always carried `graced` could never be empty, and one
//! that always carried a count would report a charm it does not have.

/// §7's room for Enchanting. Fixed by the design and reserved in the rail's
/// seven slots since `brief.rs` was written.
pub const FORGE: &str = "forge";

/// The fixture a charm is bound on, and what `imbue` opens.
///
/// The word is the puzzle's own — a lattice of glyphs — and it came back clean
/// from a sweep that found almost nothing else did.
pub const LATTICE: &str = "lattice";

/// A tool with at least one live charm on it. Carries how many ticks are left.
///
/// The count is the longest live charm, not a sum: a tool under two charms is
/// graced until the last lapses, which is what a player watching the panel row
/// sees drain.
pub const GRACED: &str = "graced";

/// A column of the lattice whose residue glyph came up alight.
///
/// The word the eight-rung table is keyed on, and it was missing from
/// [`readings`] — so `scene_at` never registered it as a `Sense`, and every rung
/// compiled only because `prose.toml` happens to carry a `recall_lit` page that
/// resolves it as a `Topic`. Rename or retire that page and every solver in the
/// domain compiles to *"that question means nothing"*, goes dead, and still
/// passes `every_shipped_solver_does_its_work` by annealing blind.
///
/// The reachability lint could not have caught it either: that asks whether
/// every *declared* word is published, and this one was published and never
/// declared. The two directions fail differently.
pub const LIT: &str = "lit";

/// A charm with little enough left to be worth renewing.
///
/// The threshold is [`EBBING_AT`]. A spell's whole maintenance rung is
/// `if the mortar_and_pestle has ebbing` — no arithmetic, no memory.
pub const EBBING: &str = "ebbing";

/// Below how many ticks left a charm reads as [`EBBING`].
///
/// Wide enough that a spell at one step a tick can act on it. `SCRIPT_BUDGET` is
/// 1, so a solver that reads the rung, walks to the forge and imbues spends
/// several ticks getting there — a threshold of five would have the charm lapse
/// inside the loop that exists to keep it. Sixty is a minute at 1 Hz.
///
/// A placeholder like every duration here, and `orbs-balance` is what sweeps it.
pub const EBBING_AT: u64 = 60;

/// Every word this domain publishes about a charm.
///
/// Appended, never inserted — §6 resolves a noun tie to whichever was registered
/// first, so a word slipped into the middle silently re-resolves a name an
/// existing spell already uses. Six other sites state the same rule.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    let mut out = vec![GRACED, EBBING, LIT];
    out.extend(super::Kind::ALL.into_iter().map(super::Kind::word));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A duplicate would register one noun twice and make `recall scripting`
    /// list a word the room answers for only once.
    #[test]
    fn every_reading_the_domain_publishes_is_named_once() {
        let mut words = readings();
        let count = words.len();
        words.sort_unstable();
        words.dedup();
        assert_eq!(count, words.len(), "a charm reading is declared twice");
    }

    /// The charm words are readings in their own right — `if the mortar has
    /// hurried` is how a spell asks which charm it is under, and without this
    /// the words would be typeable at `imbue` and unaskable in a spell.
    #[test]
    fn every_charm_is_a_word_a_spell_can_ask_for() {
        let words = readings();
        for kind in super::super::Kind::ALL {
            assert!(
                words.contains(&kind.word()),
                "{} is not a reading, so no spell can ask about it",
                kind.word(),
            );
        }
    }
}
