//! What a spell can ask about a charm.
//!
//! Declared, never derived from `Role::Reading`, which once had the lens teach
//! the archive's words.
//!
//! A spell holds a rule, not a memory (§8.1), so it cannot count ticks since it
//! imbued: [`EBBING`] is the world saying *nearly out*, which is what makes a
//! maintenance loop writable. Published only while true, since `is empty` and
//! `many_at` both read absence.

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
/// The word the eight-rung table is keyed on. It must be in [`readings`]: once
/// missing, every rung compiled only because a `recall_lit` page happened to
/// resolve it as a `Topic`.
pub const LIT: &str = "lit";

/// A charm with little enough left to be worth renewing.
///
/// The threshold is [`EBBING_AT`]. A spell's whole maintenance rung is
/// `if the mortar_and_pestle has ebbing` — no arithmetic, no memory.
pub const EBBING: &str = "ebbing";

/// Below how many ticks left a charm reads as [`EBBING`].
///
/// Wide enough that a spell at one step a tick reaches the imbue before the
/// charm lapses. A minute at 1 Hz; `orbs-balance` sweeps it.
pub const EBBING_AT: u64 = 60;

/// Every word this domain publishes about a charm.
///
/// Appended, never inserted: a noun tie resolves to the first registered.
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
