//! What work is worth, and what it buys (DESIGN.md §11.5).
//!
//! Authored in `content/progression.toml`, not here (rule 6). This module knows
//! the *shape* of the curve and nothing about its numbers.
//!
//! # It reaches decisions, so it is not hot-reloadable
//!
//! The opposite of [`Materials`](super::Materials) next door, and for the reason
//! that module states: a tint is read by the panel and by nothing else, so
//! swapping it cannot change what the world does. A **weight** changes what a
//! run earns and a **threshold** gates a verb, so both are decisions — and
//! `Sim::new` is explicit that content reaching a decision loads once, because
//! otherwise `(seed, submissions)` stops replaying.
//!
//! # An instrument nobody can name is an error
//!
//! The keys under `[earns]` are checked against `recipes.toml`'s own, and a name
//! matching none of them fails the *load*. An instrument missing from this table
//! earns nothing, which looks exactly like an instrument someone has deliberately
//! priced at zero — the same indistinguishable-typo problem `materials.toml`
//! records paying for once, and `Recipe::heat` and `craft_of` before it.
//!
//! **Validated against other content**, which is new: `materials.toml` checks
//! its colours against a Rust enum, and nothing here can. So the check takes the
//! recipes as an argument rather than reaching for a resource, which keeps it a
//! pure function and lets `Sim::new` decide the order.

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use serde::Deserialize;

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/progression.toml");

/// The file's name, for an error a writer can act on.
const FILE: &str = "progression.toml";

/// The archive's key: `divine` is a domain's work rather than an instrument's,
/// so it is the one name under `[earns]` with no recipes behind it.
pub const DIVINE: &str = "divine";

/// What work is worth, and what it buys.
///
/// **`Default` is the built-in table, not an empty one.** `Fuels` and
/// `Materials` both carry the same hand-written impl, and `Materials` records
/// why: a derived `Default` gives an empty map, `Sim` installs it with
/// `init_resource`, and every run silently earns nothing while the file on disk
/// is perfectly correct.
#[derive(Debug, Clone, Deserialize, Resource)]
pub struct Progression {
    /// What a completed run is worth, by what did it.
    earns: BTreeMap<String, u64>,
    /// What experience buys.
    concentration: Levels,
}

/// The thresholds for one track.
#[derive(Debug, Clone, Deserialize)]
struct Levels {
    /// The total needed for each level, in order.
    levels: Vec<u64>,
}

impl Default for Progression {
    fn default() -> Self {
        Self::builtin()
    }
}

impl Progression {
    /// The curve compiled into the binary.
    ///
    /// # Panics
    ///
    /// If the built-in file is malformed — a build-time authoring error, covered
    /// by `the_builtin_file_parses`.
    #[must_use]
    pub fn builtin() -> Self {
        super::load::builtin(FILE, BUILTIN)
    }

    /// Parse a progression file.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of
    /// the expected shape.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        super::load::parse(FILE, text)
    }

    /// Check every `[earns]` key against the instruments that exist, and the
    /// thresholds against each other.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) naming the key and listing the real
    /// instruments. See the module header for why this is a load failure.
    pub fn check(&self, instruments: &[&str]) -> Result<(), super::ContentError> {
        // **Ascending, because [`concentration`](Self::concentration) counts
        // with `take_while` and stops at the first entry it cannot afford.**
        // `levels = [30, 16]` would gate level 2 behind 30 *and* never award
        // level 1 at 16 — a curve that reads as authored and behaves as neither,
        // with no verb refusing and nothing to look at. The sort is the meaning
        // of the list, so an unsorted one is a malformed file rather than an
        // unusual one.
        if let Some(pair) = self.concentration.levels.windows(2).find(|two| {
            let [first, second] = two else { return false };
            second <= first
        }) {
            return Err(super::ContentError::new(
                FILE,
                format!(
                    "concentration levels must ascend, and {} does not follow {}: {:?}",
                    pair[1], pair[0], self.concentration.levels,
                ),
            ));
        }

        let Some(unknown) = self
            .earns
            .keys()
            .find(|name| name.as_str() != DIVINE && !instruments.contains(&name.as_str()))
        else {
            return Ok(());
        };
        Err(super::ContentError::new(
            FILE,
            format!(
                "`{unknown}` earns experience and is not an instrument. One of: {}",
                instruments.join(", "),
            ),
        ))
    }

    /// What one completed run at `named` is worth.
    ///
    /// Zero for anything unlisted, which after [`check`](Self::check) can only
    /// be something with no recipes at all.
    #[must_use]
    pub fn earns(&self, named: &str) -> u64 {
        self.earns.get(named).copied().unwrap_or_default()
    }

    /// How many spells the orb can hold at `experience`.
    ///
    /// **Derived, never stored.** The level is a function of one number against
    /// this table, so a save carries the number and nothing can fall out of step
    /// with it — the same shape as a spell's `Program` being derived from its
    /// text rather than kept beside it.
    #[must_use]
    pub fn concentration(&self, experience: u64) -> usize {
        self.concentration
            .levels
            .iter()
            .take_while(|needed| **needed <= experience)
            .count()
    }

    /// What the next level of concentration costs, if there is one.
    ///
    /// The number a player is working toward. `None` at the top of the table,
    /// which is *"nothing more is authored yet"* rather than *"you are finished"*.
    #[must_use]
    pub fn next_concentration(&self, experience: u64) -> Option<u64> {
        self.concentration
            .levels
            .iter()
            .find(|needed| **needed > experience)
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_file_parses() {
        let curve = Progression::builtin();
        assert_eq!(curve.earns("mortar_and_pestle"), 1);
        assert_eq!(curve.earns("balneum_mariae"), 2);
        assert_eq!(curve.earns("flask_and_rod"), 4);
        assert_eq!(curve.earns("alembic"), 8);
        assert_eq!(curve.earns(DIVINE), 1);
    }

    #[test]
    fn one_clarity_is_the_first_threshold() {
        // **The number the whole curve is anchored to**, spelled out here rather
        // than trusted: grind sage, digest, grind salt, mix, distil. If a weight
        // moves and the threshold does not, this is what says so.
        let curve = Progression::builtin();
        let clarity = curve.earns("mortar_and_pestle")
            + curve.earns("balneum_mariae")
            + curve.earns("mortar_and_pestle")
            + curve.earns("flask_and_rod")
            + curve.earns("alembic");

        assert_eq!(clarity, 16);
        assert_eq!(curve.concentration(clarity - 1), 0, "it arrived early");
        assert_eq!(curve.concentration(clarity), 1, "one clarity is not enough");
        assert_eq!(curve.next_concentration(0), Some(16));
    }

    #[test]
    fn an_instrument_that_does_not_exist_fails_the_load() {
        // A key nobody can name earns nothing, which is indistinguishable from a
        // number somebody chose. `materials.toml` records paying for this once.
        let curve = Progression::builtin();
        assert!(curve.check(&["mortar_and_pestle"]).is_err());
        assert!(
            curve
                .check(&[
                    "mortar_and_pestle",
                    "balneum_mariae",
                    "flask_and_rod",
                    "alembic",
                ])
                .is_ok(),
            "the real instruments were rejected",
        );
    }

    #[test]
    fn levels_are_counted_rather_than_looked_up() {
        let curve: Progression = Progression::parse(
            "[earns]\nmortar_and_pestle = 1\n[concentration]\nlevels = [10, 30, 90]\n",
        )
        .expect("valid");

        for (experience, expected) in [(0, 0), (9, 0), (10, 1), (29, 1), (30, 2), (900, 3)] {
            assert_eq!(curve.concentration(experience), expected, "at {experience}");
        }
        assert_eq!(curve.next_concentration(10), Some(30));
        assert_eq!(curve.next_concentration(900), None, "past the last level");
    }

    #[test]
    fn a_curve_that_does_not_ascend_fails_the_load() {
        // **`take_while` stops at the first level it cannot afford**, so
        // `[30, 16]` would gate level 2 behind 30 *and* never award level 1 at
        // 16 — a table that reads as authored and behaves as neither, with no
        // verb refusing and nothing to look at. The sort is the meaning of the
        // list, so an unsorted one is malformed rather than unusual.
        let out_of_order = Progression::parse(
            "[earns]\nmortar_and_pestle = 1\n[concentration]\nlevels = [30, 16]\n",
        )
        .expect("valid TOML");
        assert_eq!(
            out_of_order.concentration(16),
            0,
            "the reading this refuses is not the one that was broken",
        );
        assert!(out_of_order.check(&["mortar_and_pestle"]).is_err());

        // A repeat is the same fault: a second level bought by the same number
        // is a level nobody can work toward.
        let repeated = Progression::parse(
            "[earns]\nmortar_and_pestle = 1\n[concentration]\nlevels = [16, 16]\n",
        )
        .expect("valid TOML");
        assert!(repeated.check(&["mortar_and_pestle"]).is_err());

        assert!(
            Progression::builtin()
                .check(&[
                    "mortar_and_pestle",
                    "balneum_mariae",
                    "flask_and_rod",
                    "alembic",
                ])
                .is_ok(),
            "the shipped curve does not ascend",
        );
    }
}
