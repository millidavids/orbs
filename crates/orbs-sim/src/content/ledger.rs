//! What the ledger counts (DESIGN.md §19, *number go up*).
//!
//! Authored in `progression.toml` as `[[ledger]]`, one row per lifetime count a
//! player can look at: what each room has made in all, and a few things the
//! tower has done. A row's label is prose, keyed by its id.

use serde::Deserialize;

use super::deed::Named;

/// A count with no number attached.
///
/// A [`Deed`](super::Deed)'s spellings without the count — `"potions"`,
/// `"scrolls"`, `{ made = "clarity" }`, `{ at = "prism" }`,
/// `{ event = "figure" }` — read against the same tally keys and held to the
/// same check. A `Deed` asks *how many*, and reusing it would make an author
/// write a number that means nothing.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Counted {
    /// `"potions"` or `"scrolls"`, of any kind.
    Kind(Kind),
    /// One product, at any instrument.
    Made {
        /// The product.
        made: String,
    },
    /// Runs finished at one instrument.
    At {
        /// The instrument, by the name `[earns]` prices it under.
        at: String,
    },
    /// Something a room does that is not a run.
    Event {
        /// One of [`EVENTS`](crate::tower::EVENTS).
        event: String,
    },
}

/// The two things counted by kind rather than by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Potions, of any kind.
    Potions,
    /// Scrolls, of any kind.
    Scrolls,
}

impl Counted {
    /// The tally key this counts — the one [`Deed::key`](super::Deed::key)
    /// writes for the same spelling.
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Self::Kind(Kind::Potions) => "potion".to_owned(),
            Self::Kind(Kind::Scrolls) => "scroll".to_owned(),
            Self::Made { made } => format!("made:{made}"),
            Self::At { at } => format!("at:{at}"),
            Self::Event { event } => format!("event:{event}"),
        }
    }

    /// Whether this names something the game counts.
    ///
    /// # Errors
    ///
    /// A sentence naming what is wrong, for `Progression::check` to wrap.
    pub fn check(&self, outputs: &[&str], instruments: &[&str]) -> Result<(), String> {
        let named = match self {
            Self::Kind(_) => Named::Kind,
            Self::Made { made } => Named::Made(made),
            Self::At { at } => Named::At(at),
            Self::Event { event } => Named::Event(event),
        };
        named.check(outputs, instruments)
    }
}

/// One row of the ledger.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// The id: a decision, not prose. `ledger_<id>` is the label a player reads.
    pub id: String,
    /// The room it belongs to, where the room shows it as it climbs. None for a
    /// count about the whole tower, which only `status` shows.
    #[serde(default)]
    pub domain: Option<String>,
    /// What is counted.
    pub counts: Counted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Holder {
        counts: Counted,
    }

    fn parse(text: &str) -> Result<Counted, toml::de::Error> {
        toml::from_str::<Holder>(text).map(|holder| holder.counts)
    }

    #[test]
    fn every_spelling_reads_the_key_its_deed_reads() {
        let pairs = [
            ("counts = \"potions\"", "potion"),
            ("counts = \"scrolls\"", "scroll"),
            ("counts = { made = \"clarity\" }", "made:clarity"),
            ("counts = { at = \"prism\" }", "at:prism"),
            ("counts = { event = \"figure\" }", "event:figure"),
        ];
        for (text, key) in pairs {
            assert_eq!(
                parse(text).map(|counted| counted.key()).ok(),
                Some(key.to_owned())
            );
        }
    }

    #[test]
    fn a_count_with_a_number_is_a_deed_not_a_ledger_row() {
        assert!(parse("counts = { potions = 5 }").is_err());
        assert!(parse("counts = { at = \"prism\", times = 3 }").is_err());
        assert!(parse("counts = \"elixirs\"").is_err());
    }

    #[test]
    fn a_name_nothing_counts_is_refused() {
        let outputs = ["clarity"];
        let instruments = ["prism"];
        let check = |text: &str| {
            parse(text).is_ok_and(|counted| counted.check(&outputs, &instruments).is_ok())
        };
        assert!(check("counts = { made = \"clarity\" }"));
        assert!(!check("counts = { made = \"elixir\" }"));
        assert!(!check("counts = { at = \"kettle\" }"));
        assert!(!check("counts = { event = \"picnic\" }"));
    }
}
