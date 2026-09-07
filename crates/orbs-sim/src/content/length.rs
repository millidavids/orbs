//! How long a game is, chosen when it begins (DESIGN.md §11.5, §19).
//!
//! # Why a game has a length at all
//!
//! The shipped curve tops out at 10,000 experience, which is roughly five
//! thousand hand-played commands — **reachable by hand**. That makes playing
//! manually barely worse than automating, which undercuts pillar 3: *automation
//! is progression*. A tower you can finish by typing is a tower that never has to
//! teach you to write a spell.
//!
//! So the curve stretches, and how far is a choice the player makes once.
//!
//! # The head is anchored and the tail is stretched
//!
//! **A flat multiplier would break the thing this exists to serve.**
//! `progression.toml` on the first station: *"The player does the whole loop by
//! hand once, and the reward is not having to do it again — which is pillar 3's
//! promise landing as a mechanic rather than as a premise."* Six times that is
//! six clarities brewed by hand before the first spell slot arrives; twenty-five
//! times is twenty-five, against §12's *"a non-terminal player reaches hour two
//! unaided"*.
//!
//! So the stretch **ramps**: nothing at the first station, full at the last.
//!
//! ```text
//! factor(i) = 1 + (k - 1) * (i / (n - 1))^2
//! ```
//!
//! Quadratic rather than linear because the early game is where the ramp has to
//! be gentlest — at medium the first four ley stations move 16 → 16, 24 → 24,
//! 40 → 43, 56 → 67, and the last moves 10,000 → 60,000. The second station does
//! not move at all once the integer division has had its way, which is the ramp
//! doing exactly what it is for.
//!
//! **It also dissolves the room-reveal problem without a list.** A first draft
//! named five stations to exempt and three of the five were wrong; and because a
//! mastery line is *sequential*, exempting one station does not protect it when a
//! scaled station precedes it. An index-anchored ramp needs no exemptions: every
//! line's first station is unchanged by construction.
//!
//! # The tiers are the curve, not the clock
//!
//! Each is defined by **what the last station reads**, which is a fact. What that
//! costs in hours is for `orbs-balance` to measure: §19 records the tower's
//! automated rate as an open question — *"either the line's top or additivity is
//! wrong"* — and a tier defined in hours would be a guess wearing a fact's
//! clothes. Nothing here settles that question.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

/// How long a game runs.
///
/// **Serialised by name**, so a save reads `length = "medium"` rather than a
/// number whose meaning would drift the moment a tier is retuned.
///
/// **A `Resource` as well as a save field**, because the stretch is applied once
/// at construction and the world then holds only its *result* — so without this
/// nothing could answer *what length is this game* at save time, and the answer
/// would have to be reverse-engineered from the curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Resource)]
#[serde(rename_all = "lowercase")]
pub enum Length {
    /// The curve exactly as authored. **What every tower had before length
    /// existed**, and what a dump runs at so its baselines do not move.
    Baseline,
    /// A brisk game.
    Short,
    /// The default a new game takes.
    #[default]
    Medium,
    /// The long haul.
    Long,
}

impl Length {
    /// Every length, in the order a menu offers them.
    ///
    /// [`Baseline`](Self::Baseline) is deliberately absent: it is what the
    /// content already says, kept for the dump and for a save written before any
    /// of this, and offering it to a player would be offering "the numbers we
    /// happened to author first" as a difficulty.
    pub const OFFERED: [Self; 3] = [Self::Short, Self::Medium, Self::Long];

    /// The word a save writes and a player types.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Short => "short",
            Self::Medium => "medium",
            Self::Long => "long",
        }
    }

    /// The length called `word`, if there is one.
    #[must_use]
    pub fn named(word: &str) -> Option<Self> {
        let word = word.trim().to_ascii_lowercase();
        [Self::Baseline, Self::Short, Self::Medium, Self::Long]
            .into_iter()
            .find(|length| length.word() == word)
    }

    /// What the **last** station on a track reads at this length, against a
    /// track whose last station currently reads `authored`.
    ///
    /// **The tier is this number and nothing else.** Hours are swept, not
    /// promised — see the module doc.
    #[must_use]
    pub const fn last(self, authored: u64) -> u64 {
        match self {
            Self::Baseline => authored,
            Self::Short => authored * 3,
            Self::Medium => authored * 6,
            Self::Long => authored * 25,
        }
    }

    /// What the station at `index` of `count` is multiplied by.
    ///
    /// Nothing at the head, [`last`](Self::last)'s whole factor at the tail, and
    /// a quadratic ramp between. Returned as a numerator over `SCALE` — a
    /// private fixed-point denominator — so the arithmetic stays integral: a
    /// curve that depended on float rounding would replay differently on a
    /// different target.
    #[must_use]
    pub const fn ramp(self, index: usize, count: usize) -> u64 {
        if count <= 1 || index == 0 || matches!(self, Self::Baseline) {
            return SCALE;
        }
        // `k - 1` in SCALE units, times `(i / (n - 1))^2`, plus one.
        let last = self.last(SCALE);
        let over = last - SCALE;
        let span = (count - 1) as u64;
        let step = index as u64;
        SCALE + (over * step * step) / (span * span)
    }

    /// Stretch `authored`, the value at `index` of `count`.
    ///
    /// **Never below what was authored**, and never nought: a threshold that
    /// rounded down to zero would be a station reached before the game began, and
    /// a deed of nought is refused at load.
    #[must_use]
    pub const fn stretch(self, authored: u64, index: usize, count: usize) -> u64 {
        let stretched = (authored * self.ramp(index, count)) / SCALE;
        if stretched < authored {
            authored
        } else {
            stretched
        }
    }
}

/// The fixed-point denominator [`Length::ramp`] returns over. Private, so its
/// doc link there is plain text — the ratio is the contract, not this number.
///
/// **Integer arithmetic throughout.** A curve computed in floats would be a
/// world that could replay differently on a different target, which is the one
/// property §13 will not spend.
const SCALE: u64 = 1_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_head_never_moves_at_any_length() {
        // **Pillar 3's landing point.** The first station is one clarity brewed
        // by hand, and the reward is the slot that means never doing it again.
        // Stretching it would make the tutorial the grind.
        for length in [Length::Short, Length::Medium, Length::Long] {
            assert_eq!(length.stretch(16, 0, 16), 16, "{length:?} moved the head");
        }
    }

    #[test]
    fn the_tail_reads_exactly_what_the_tier_promises() {
        // The tier *is* this number, so it has to be exact rather than near.
        assert_eq!(Length::Short.stretch(10_000, 15, 16), 30_000);
        assert_eq!(Length::Medium.stretch(10_000, 15, 16), 60_000);
        assert_eq!(Length::Long.stretch(10_000, 15, 16), 250_000);
    }

    #[test]
    fn baseline_is_the_curve_exactly_as_authored() {
        // What the dump runs at, and what a save written before length existed
        // reads as — so neither moves.
        for index in 0..16 {
            assert_eq!(Length::Baseline.stretch(640, index, 16), 640);
        }
    }

    #[test]
    fn a_stretched_track_still_ascends_strictly() {
        // **`Progression::check`'s load gate.** A track that stopped ascending
        // would panic at startup, so this is asserted rather than argued.
        let authored = [
            16, 24, 40, 56, 96, 160, 256, 400, 640, 1_000, 1_600, 2_500, 4_000, 6_400, 8_000,
            10_000,
        ];
        for length in [
            Length::Baseline,
            Length::Short,
            Length::Medium,
            Length::Long,
        ] {
            let stretched: Vec<u64> = authored
                .iter()
                .enumerate()
                .map(|(index, at)| length.stretch(*at, index, authored.len()))
                .collect();
            assert!(
                stretched.windows(2).all(|pair| pair[0] < pair[1]),
                "{length:?} broke strict ascent: {stretched:?}",
            );
        }
    }

    #[test]
    fn the_early_game_is_barely_touched() {
        // The ramp's whole point: the first four stations stay close to what
        // they are, so the tower opens at the pace it opens at now.
        let authored = [16, 24, 40, 56];
        let stretched: Vec<u64> = authored
            .iter()
            .enumerate()
            .map(|(index, at)| Length::Medium.stretch(*at, index, 16))
            .collect();
        assert_eq!(stretched, vec![16, 24, 43, 67]);
    }

    #[test]
    fn a_count_never_stretches_below_itself_or_to_nought() {
        // A deed of nought is refused at load, and a station reached before the
        // game began is not a station.
        for length in [
            Length::Baseline,
            Length::Short,
            Length::Medium,
            Length::Long,
        ] {
            for index in 0..8 {
                assert!(length.stretch(1, index, 8) >= 1);
            }
        }
    }

    #[test]
    fn stretching_never_touches_what_a_run_earns() {
        // **The failure that would cancel the feature in silence.** `earns` is
        // what a run is *worth*; the thresholds are what it is worth *against*.
        // Stretch both and you have multiplied numerator and denominator — the
        // curve looks longer, plays identically, and every rate `orbs-balance`
        // pins still passes, so nothing anywhere goes red.
        // Every instrument `[earns]` prices, so a name added there is covered by
        // the same assertion rather than by a second list nobody updates.
        const PRICED: [&str; 9] = [
            "mortar_and_pestle",
            "balneum_mariae",
            "flask_and_rod",
            "alembic",
            "stacks",
            "lectern",
            "prism",
            "pylon",
            "lattice",
        ];
        let authored = crate::content::Progression::builtin();
        for length in [Length::Short, Length::Medium, Length::Long] {
            let stretched = crate::content::Progression::builtin().stretched(length);
            for named in PRICED {
                assert_eq!(
                    authored.earns(named),
                    stretched.earns(named),
                    "{length:?} moved what a {named} run earns",
                );
            }
        }
    }

    #[test]
    fn every_rooms_line_is_ramped_on_its_own() {
        // Mastery is seven lines, not one. A station's ramp is its place on its
        // *room's* line — indexing the flat file would put the menagerie's first
        // deed a third of the way up the ramp for no reason a player could see.
        let stretched = crate::content::Progression::builtin().stretched(Length::Long);
        for domain in [
            "laboratory",
            "archive",
            "lens",
            "sanctum",
            "menagerie",
            "forge",
        ] {
            let first = stretched
                .mastery()
                .iter()
                .find(|stone| stone.domain == domain)
                .expect("every room has a line");
            let authored = crate::content::Progression::builtin();
            let was = authored
                .mastery()
                .iter()
                .find(|stone| stone.domain == domain)
                .expect("every room has a line");
            assert_eq!(
                first.done, was.done,
                "{domain}'s first station moved, so its room opens late",
            );
        }
    }

    #[test]
    fn a_length_round_trips_through_its_word() {
        for length in [
            Length::Baseline,
            Length::Short,
            Length::Medium,
            Length::Long,
        ] {
            assert_eq!(Length::named(length.word()), Some(length));
        }
        assert_eq!(Length::named("  MEDIUM  "), Some(Length::Medium));
        assert_eq!(Length::named("epic"), None);
    }
}

#[cfg(test)]
mod curve {
    use super::*;

    /// Print the shipped ley line at every length. `--nocapture` to read it.
    ///
    /// **Not an assertion.** The tiers are defined by the last station and the
    /// hours are swept, so what this is for is looking at the shape — which is
    /// the one thing a table of numbers in a plan cannot be checked against.
    #[test]
    fn the_shipped_line_at_every_length() {
        for length in [
            Length::Baseline,
            Length::Short,
            Length::Medium,
            Length::Long,
        ] {
            let curve = crate::content::Progression::builtin().stretched(length);
            let ats: Vec<u64> = curve.ley_line().iter().map(|station| station.at).collect();
            println!("{:9} {ats:?}", length.word());
        }
    }
}
