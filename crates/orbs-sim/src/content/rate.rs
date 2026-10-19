//! How fast a supply draws, and what each speed is worth (§19, *Supply*).
//!
//! No produced item does anything alone. An item authors one to three rates:
//! a held count to run at, a ration drawn each time, and what that draw buys.
//! The lints below are the rule, checked at load, so a content edit cannot
//! make a single item worth something.

use serde::Deserialize;

/// One speed a supply can run at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rate {
    /// How many the arsenal must hold to run at this rate.
    pub opens: u32,
    /// How many each draw takes.
    pub draws: u32,
    /// What a draw buys: a bonus, a heal, a room, a refill. Nought for a switch.
    #[serde(default)]
    pub worth: i32,
}

/// Every rate an item has, slowest first.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Vec<Rate>")]
pub struct Rates(Vec<Rate>);

impl Rates {
    /// The rate at `index`, slowest being nought.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Rate> {
        self.0.get(index)
    }

    /// The slowest rate, which says when an item is drained.
    #[must_use]
    pub fn slowest(&self) -> &Rate {
        &self.0[0]
    }

    /// How many rates there are.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Never true: a supply with no rate is refused at load.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Every rate, slowest first.
    pub fn iter(&self) -> impl Iterator<Item = &Rate> {
        self.0.iter()
    }
}

impl TryFrom<Vec<Rate>> for Rates {
    type Error = String;

    fn try_from(rates: Vec<Rate>) -> Result<Self, Self::Error> {
        let Some(first) = rates.first() else {
            return Err("a supply needs at least one rate".to_owned());
        };
        // No pile of one does anything.
        if first.opens < 2 {
            return Err(format!(
                "the slowest rate opens at {}; a single item may not do anything",
                first.opens
            ));
        }
        for rate in &rates {
            // A draw that took the whole threshold would empty the shelf on the
            // first round, and one that took nothing would be free.
            if rate.draws == 0 || rate.draws >= rate.opens {
                return Err(format!(
                    "a rate draws {} of {}; it must draw at least one and fewer than it opens at",
                    rate.draws, rate.opens
                ));
            }
        }
        for pair in rates.windows(2) {
            let (slower, faster) = (pair[0], pair[1]);
            if faster.opens <= slower.opens || faster.draws < slower.draws {
                return Err(format!(
                    "a faster rate must open higher and draw no fewer: {slower:?} then {faster:?}"
                ));
            }
            // By size, so a bonus may be negative: a stronger curse at the
            // faster rate is more, not less. Every rate points one way.
            if (faster.worth < 0) != (slower.worth < 0) && faster.worth != 0 && slower.worth != 0 {
                return Err(format!(
                    "a supply's rates must all help or all hurt: {slower:?} then {faster:?}"
                ));
            }
            if faster.worth.unsigned_abs() < slower.worth.unsigned_abs() {
                return Err(format!(
                    "a faster rate gives less: {slower:?} then {faster:?}"
                ));
            }
            // Never more per item: speed costs efficiency, so raising a rate is
            // a decision rather than the obvious move.
            let per_faster = u64::from(faster.worth.unsigned_abs()) * u64::from(slower.draws);
            let per_slower = u64::from(slower.worth.unsigned_abs()) * u64::from(faster.draws);
            if per_faster > per_slower {
                return Err(format!(
                    "a faster rate is worth more per item: {slower:?} then {faster:?}"
                ));
            }
        }
        Ok(Self(rates))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rate(opens: u32, draws: u32, worth: i32) -> Rate {
        Rate {
            opens,
            draws,
            worth,
        }
    }

    #[test]
    fn a_single_item_never_opens_a_supply() {
        assert!(Rates::try_from(vec![rate(1, 1, 3)]).is_err());
        assert!(Rates::try_from(vec![rate(2, 1, 3)]).is_ok());
    }

    #[test]
    fn no_rate_draws_its_whole_threshold_or_nothing() {
        assert!(Rates::try_from(vec![rate(4, 4, 1)]).is_err());
        assert!(Rates::try_from(vec![rate(4, 0, 1)]).is_err());
    }

    #[test]
    fn every_rate_steps_up() {
        assert!(Rates::try_from(vec![rate(6, 2, 2), rate(6, 3, 3)]).is_err());
        assert!(Rates::try_from(vec![rate(6, 2, 2), rate(12, 1, 2)]).is_err());
        assert!(Rates::try_from(vec![rate(6, 2, 2), rate(12, 3, 1)]).is_err());
        assert!(Rates::try_from(vec![rate(6, 2, 2), rate(12, 3, 3)]).is_ok());
    }

    #[test]
    fn a_faster_rate_never_gives_more_per_item() {
        // 3 for 2 is 1.5 an item; 5 for 3 is 1.67, which would make speed free.
        assert!(Rates::try_from(vec![rate(6, 2, 3), rate(12, 3, 5)]).is_err());
        assert!(Rates::try_from(vec![rate(6, 2, 3), rate(12, 3, 4)]).is_ok());
    }

    #[test]
    fn nothing_is_refused_as_empty() {
        assert!(Rates::try_from(Vec::new()).is_err());
    }

    #[test]
    fn a_curse_is_measured_by_its_size() {
        // A stronger curse at the faster rate is more, not less.
        assert!(Rates::try_from(vec![rate(6, 2, -2), rate(12, 3, -3)]).is_ok());
        assert!(Rates::try_from(vec![rate(6, 2, -2), rate(12, 3, -5)]).is_err());
        // ...and every rate points one way.
        assert!(Rates::try_from(vec![rate(6, 2, -2), rate(12, 3, 3)]).is_err());
    }
}
