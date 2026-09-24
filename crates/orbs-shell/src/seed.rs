//! The seed a world is built from: fixed for an instrument, its own for a game.
//!
//! Read outside the sim, always: the wall clock is not deterministic, and a
//! world that must replay identically from its seed never reaches for one. The
//! frontend chooses a number once, the sim is built from it, and the save
//! carries it. The POST card prints it, so a tester's report says which world.
//!
//! Two readers, and which one a caller wants is the whole question:
//!
//! - [`seed`] — `ORBS_SEED`, or [`SEED`]. Every instrument: `ORBS_DUMP`, the
//!   `screens` example, `scripts/dumps.sh`, the play harness, every See-it line.
//!   A dump drawing a different world each run could not be diffed.
//! - [`new_game_seed`] — `ORBS_SEED`, or a seed of the game's own. A game a
//!   player starts. Before `0.15.5` every player's tower was seed `0x0B5` — the
//!   same stacks, the same beasts, the same sieges.

use std::time::{SystemTime, UNIX_EPOCH};

/// The world's seed when nothing overrides it.
///
/// A hex word rather than a round number, so a seed that appears in a bug report
/// is recognisable as *the default* rather than as something the reporter chose.
pub const SEED: u64 = 0x0B5;

/// How many seeds a new game chooses among: six decimal digits.
///
/// Short enough to read off the POST card and type back, which is what a tester
/// does with a world worth reporting — `ORBS_SEED=482913`. A million towers is
/// variety no player will exhaust, and determinism needs no more.
pub const SEEDS: u64 = 1_000_000;

/// The environment variable a person names a seed with.
const VAR: &str = "ORBS_SEED";

/// The switch that makes the Bevy build take a screenshot — an instrument, so a
/// game begun under it keeps [`SEED`].
const CAPTURE: &str = "ORBS_CAPTURE";

/// The seed, or `ORBS_SEED`'s if it names a number — for an instrument.
///
/// A See-it affordance, not a setting. Anything the world generates is one
/// seed's worth of evidence per run, and one sample cannot show a distribution:
/// three dumps of the same maze looked like proof that randomising it had
/// failed, and were three copies of one seed.
///
/// Tests sweep seeds directly through `Sim::new` and always could. This is the
/// same reach from outside the binary, so a person can look rather than trust a
/// test — which is the whole of §15's gate.
#[must_use]
pub fn seed() -> u64 {
    chosen().unwrap_or(SEED)
}

/// The seed a new game is built from: `ORBS_SEED` if it names a number, else a
/// seed of its own from the wall clock.
///
/// A screenshot is an instrument, so with `ORBS_CAPTURE` set this is [`SEED`] —
/// a reproducible screenshot is why the switch exists. An `ORBS_SEED` naming no
/// number is treated as unset, so the game still gets a seed of its own.
#[must_use]
pub fn new_game_seed() -> u64 {
    if let Some(named) = chosen() {
        return named;
    }
    if std::env::var_os(CAPTURE).is_some() {
        return SEED;
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    fresh_seed(nanos)
}

/// What `ORBS_SEED` names, if it names a number.
fn chosen() -> Option<u64> {
    std::env::var(VAR)
        .ok()
        .and_then(|value| value.trim().parse().ok())
}

/// A seed from a moment, below [`SEEDS`].
///
/// Pure, so it is testable without the clock. The nanoseconds fold through
/// splitmix64 so two games begun a moment apart are not neighbouring seeds, and
/// the result is taken below [`SEEDS`] so it reads in six digits.
#[must_use]
pub fn fresh_seed(nanos: u128) -> u64 {
    let low = u64::try_from(nanos & u128::from(u64::MAX)).unwrap_or(0);
    let high = u64::try_from(nanos >> 64).unwrap_or(0);
    mix(low ^ mix(high)) % SEEDS
}

/// splitmix64's finaliser: every input bit moves about half the output bits.
const fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_moment_gives_one_seed_and_the_next_moment_another() {
        let now = 1_789_010_728_123_456_789_u128;
        assert_eq!(fresh_seed(now), fresh_seed(now));
        assert_ne!(fresh_seed(now), fresh_seed(now + 1));
    }

    /// Six digits and spread across them — a thousand moments a nanosecond apart
    /// land on nearly a thousand different seeds, all below the bound.
    #[test]
    fn seeds_from_nearby_moments_are_spread_and_short() {
        let start = 1_789_010_728_000_000_000_u128;
        let seeds: std::collections::BTreeSet<u64> =
            (0..1000).map(|offset| fresh_seed(start + offset)).collect();
        assert!(seeds.len() > 990, "only {} distinct seeds", seeds.len());
        assert!(seeds.iter().all(|seed| *seed < SEEDS));
        assert!(
            seeds.iter().any(|seed| *seed > SEEDS / 2)
                && seeds.iter().any(|seed| *seed < SEEDS / 2),
            "the seeds bunch at one end",
        );
    }

    /// The high half of the moment counts too, so a clock far from 1970 does not
    /// fold two different moments onto one seed through the low bits alone.
    #[test]
    fn the_high_half_of_a_moment_changes_the_seed() {
        let low = 42_u128;
        assert_ne!(fresh_seed(low), fresh_seed(low | (1 << 64)));
    }
}
