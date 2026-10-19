//! Which rate a supply may run or be raised to — pure, so the rules are tested
//! without a world (§19, *Supply*).

use super::open::Scope;
use crate::content::Rates;
use crate::tower::Freshness;

/// What one more word does to a supply: the rate it raises to, or why not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Raise {
    /// It runs at this rate now.
    To(usize),
    /// Nothing made lately, so nothing flows.
    Spent,
    /// Made only once lately, so only the slowest rate runs.
    Thin,
    /// The arsenal holds too few for that rate.
    Short {
        /// What that rate needs held.
        opens: u32,
    },
    /// It already runs as fast as it can.
    Fastest,
}

/// Why a word on a supply did nothing: a prose key, and what it fills in
/// beside the item's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The line to say.
    pub key: &'static str,
    /// Its fields other than `name`.
    pub args: Vec<(&'static str, String)>,
}

impl Refusal {
    /// A refusal that says only the item's name.
    #[must_use]
    pub const fn bare(key: &'static str) -> Self {
        Self {
            key,
            args: Vec::new(),
        }
    }

    /// Every field, `name` first, as `Prose::line` takes them.
    #[must_use]
    pub fn fields<'a>(&'a self, name: &'a str) -> Vec<(&'a str, &'a str)> {
        let mut all = vec![("name", name)];
        all.extend(
            self.args
                .iter()
                .map(|(field, value)| (*field, value.as_str())),
        );
        all
    }
}

impl Raise {
    /// The rate it raises to, or why one more word was refused.
    ///
    /// The short refusal speaks of the arsenal: what it `held`, and what it
    /// must hold once the `drawn` this round already took is counted.
    ///
    /// # Errors
    ///
    /// Every answer but [`To`](Self::To), as the line that says why.
    pub fn into_rate(self, held: u32, drawn: u32) -> Result<usize, Refusal> {
        match self {
            Self::To(at) => Ok(at),
            Self::Spent => Err(Refusal::bare("spend_stale")),
            Self::Thin => Err(Refusal::bare("supply_thin")),
            Self::Fastest => Err(Refusal::bare("supply_fastest")),
            Self::Short { opens } => Err(Refusal {
                key: "supply_short",
                args: vec![
                    ("count", held.to_string()),
                    ("quantity", opens.saturating_sub(drawn).to_string()),
                ],
            }),
        }
    }
}

/// The line for a supply that ran low at a draw: dry or slower, stale or
/// short, on the wall or in a room.
#[must_use]
pub const fn ran_low(scope: Scope, slowed: Slowed, dry: bool) -> &'static str {
    match (scope, slowed, dry) {
        (Scope::Siege, Slowed::Stale, true) => "supply_stale_drained",
        (Scope::Siege, Slowed::Short, true) => "supply_drained",
        (Scope::Siege, Slowed::Stale, false) => "supply_stale_stepped",
        (Scope::Siege, Slowed::Short, false) => "supply_stepped",
        (Scope::Calm, Slowed::Stale, true) => "calm_stale_drained",
        (Scope::Calm, Slowed::Short, true) => "calm_drained",
        (Scope::Calm, Slowed::Stale, false) => "calm_stale_stepped",
        (Scope::Calm, Slowed::Short, false) => "calm_stepped",
    }
}

/// Why a supply is not running at its chosen rate, for the line that says so.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slowed {
    /// Too few held for a faster rate, or for any.
    Short,
    /// Made too seldom lately: thin runs only the slowest, spent nothing.
    Stale,
}

/// Why `held` and `freshness` hold a supply below rate `above`.
///
/// Stale when freshness alone forbids it, so a player told it ran dry while
/// the arsenal is full is told to make more instead.
#[must_use]
pub fn slowed(rates: &Rates, held: u32, freshness: Freshness, above: usize) -> Slowed {
    let fresh_would = running(rates, held, Freshness::Fresh, above);
    let now = running(rates, held, freshness, above);
    if fresh_would > now {
        Slowed::Stale
    } else {
        Slowed::Short
    }
}

/// The rate a draw runs at: the fastest no faster than `chosen` that `held`
/// still meets and `freshness` allows. `None` is drained.
#[must_use]
pub fn running(rates: &Rates, held: u32, freshness: Freshness, chosen: usize) -> Option<usize> {
    let top = match freshness {
        Freshness::Spent => return None,
        Freshness::Thin => 0,
        Freshness::Fresh => chosen,
    }
    .min(rates.len().saturating_sub(1));
    (0..=top)
        .rev()
        .find(|at| rates.get(*at).is_some_and(|rate| held >= rate.opens))
}

/// What one more word does, from the rate `current`ly running — `None` when
/// the supply is shut.
///
/// `held` is what the arsenal holds plus what this round already drew, so a
/// raise mid-turn is not refused for the ration it has just taken.
#[must_use]
pub fn may_raise(rates: &Rates, held: u32, freshness: Freshness, current: Option<usize>) -> Raise {
    let next = current.map_or(0, |at| at + 1);
    let Some(rate) = rates.get(next) else {
        return Raise::Fastest;
    };
    match freshness {
        Freshness::Spent => return Raise::Spent,
        Freshness::Thin if next > 0 => return Raise::Thin,
        _ => {}
    }
    if held < rate.opens {
        return Raise::Short { opens: rate.opens };
    }
    Raise::To(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Rate;

    fn warding() -> Rates {
        Rates::try_from(vec![
            Rate {
                opens: 6,
                draws: 2,
                worth: 2,
            },
            Rate {
                opens: 12,
                draws: 3,
                worth: 3,
            },
            Rate {
                opens: 20,
                draws: 4,
                worth: 4,
            },
        ])
        .expect("a valid table")
    }

    #[test]
    fn draining_steps_down_then_closes() {
        let rates = warding();
        assert_eq!(running(&rates, 25, Freshness::Fresh, 2), Some(2));
        assert_eq!(running(&rates, 19, Freshness::Fresh, 2), Some(1));
        assert_eq!(running(&rates, 7, Freshness::Fresh, 2), Some(0));
        assert_eq!(running(&rates, 5, Freshness::Fresh, 2), None, "drained");
    }

    #[test]
    fn a_draw_never_runs_faster_than_chosen() {
        assert_eq!(running(&warding(), 25, Freshness::Fresh, 0), Some(0));
    }

    #[test]
    fn a_thin_store_runs_only_the_slowest_rate() {
        let rates = warding();
        assert_eq!(running(&rates, 25, Freshness::Thin, 2), Some(0));
        assert_eq!(may_raise(&rates, 25, Freshness::Thin, None), Raise::To(0));
        assert_eq!(may_raise(&rates, 25, Freshness::Thin, Some(0)), Raise::Thin);
    }

    #[test]
    fn a_spent_store_cannot_open() {
        let rates = warding();
        assert_eq!(running(&rates, 25, Freshness::Spent, 0), None);
        assert_eq!(may_raise(&rates, 25, Freshness::Spent, None), Raise::Spent);
    }

    #[test]
    fn one_word_raises_one_rate_and_the_fastest_refuses() {
        let rates = warding();
        assert_eq!(may_raise(&rates, 25, Freshness::Fresh, None), Raise::To(0));
        assert_eq!(
            may_raise(&rates, 25, Freshness::Fresh, Some(0)),
            Raise::To(1)
        );
        assert_eq!(
            may_raise(&rates, 25, Freshness::Fresh, Some(1)),
            Raise::To(2)
        );
        assert_eq!(
            may_raise(&rates, 25, Freshness::Fresh, Some(2)),
            Raise::Fastest
        );
    }

    #[test]
    fn too_few_held_is_short_and_says_how_many() {
        let rates = warding();
        assert_eq!(
            may_raise(&rates, 5, Freshness::Fresh, None),
            Raise::Short { opens: 6 }
        );
        assert_eq!(
            may_raise(&rates, 11, Freshness::Fresh, Some(0)),
            Raise::Short { opens: 12 }
        );
    }
}
