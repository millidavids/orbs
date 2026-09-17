//! What a mastery station asks for (DESIGN.md §11.5).
//!
//! A deed is a count of something done in a room — a potion brewed, a walk of
//! the stacks finished, a beast held — and a station on a domain's line is
//! reached when its count is met. Authored in `progression.toml`, read against
//! [`Tally`](crate::tower::Tally).
//!
//! # Five spellings, one key
//!
//! `done = "clarity"` · `{ potions = 5 }` · `{ scrolls = 1 }` ·
//! `{ at = "stacks", times = 3 }` · `{ event = "figure", times = 1 }`. Each
//! names exactly one tally key, and the key is the whole of what the sim reads —
//! the spellings exist so an author writes *"brew a clarity"* rather than a
//! namespaced string, and so a typo in one is a load failure rather than a
//! station nothing can reach.
//!
//! # Not a second currency
//!
//! A deed counts things done; it is never a number that accrues per run the way
//! experience does. §11.5 keeps experience the one unspendable total, and a
//! per-domain total beside it would be a second curve to balance and a second
//! number to save. The user's two examples — *"a certain number of potions"* and
//! *"certain potions"* — are one count each, which is what this is.

use serde::Deserialize;

/// One thing a station asks to have been done.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Deed {
    /// `done = "clarity"` — make this once, at any instrument.
    Made(String),
    /// `done = { potions = n }` — that many potions, of any kind.
    Potions {
        /// How many.
        potions: u32,
    },
    /// `done = { scrolls = n }` — that many scrolls assembled.
    Scrolls {
        /// How many.
        scrolls: u32,
    },
    /// `done = { at = "stacks", times = n }` — runs finished at an instrument.
    At {
        /// The instrument, by the name `[earns]` prices it under.
        at: String,
        /// How many. One, if unsaid.
        #[serde(default = "once")]
        times: u32,
    },
    /// `done = { event = "figure", times = n }` — something a room does that is
    /// not a run: a beast held, a siege settled, a spell bound.
    Event {
        /// One of [`EVENTS`](crate::tower::EVENTS).
        event: String,
        /// How many. One, if unsaid.
        #[serde(default = "once")]
        times: u32,
        /// `fixed = true` — the same count at every length.
        ///
        /// **For a lesson, not a grind**, which is [`Made`](Deed::Made)'s reason
        /// for never stretching, given a count. `menagerie_2` opens the whole
        /// circle after five lesser beasts, and five is the lesson's size: each
        /// lesser temper about once. Ramped as its place on the line, it asked
        /// eleven at the default length and thirty-five at the longest — a
        /// tutorial that grew with the game it was teaching (§19).
        #[serde(default)]
        fixed: bool,
    },
}

/// The default for a count: once.
const fn once() -> u32 {
    1
}

impl Deed {
    /// This deed as the `index`th of `count` on its line, at `length`.
    ///
    /// **[`Made`](Self::Made) has no count and is returned untouched** — it is
    /// *make this thing once*, a reveal gate rather than a grind, and there is no
    /// field in it to stretch. That is a property of the variant rather than an
    /// exemption someone has to remember.
    ///
    /// Every other variant stretches its own count, except an event marked
    /// `fixed`, which is a lesson's size rather than a grind's. The `at` and
    /// `event` names are what the tally is keyed by and are never touched.
    #[must_use]
    pub fn stretched(&self, length: crate::content::Length, index: usize, count: usize) -> Self {
        let grown = |n: u32| -> u32 {
            u32::try_from(length.stretch(u64::from(n), index, count)).unwrap_or(u32::MAX)
        };
        match self {
            Self::Made(name) => Self::Made(name.clone()),
            Self::Potions { potions } => Self::Potions {
                potions: grown(*potions),
            },
            Self::Scrolls { scrolls } => Self::Scrolls {
                scrolls: grown(*scrolls),
            },
            Self::At { at, times } => Self::At {
                at: at.clone(),
                times: grown(*times),
            },
            Self::Event {
                event,
                times,
                fixed,
            } => Self::Event {
                event: event.clone(),
                times: if *fixed { *times } else { grown(*times) },
                fixed: *fixed,
            },
        }
    }
}

impl Deed {
    /// The tally key this deed reads.
    ///
    /// **One key per deed, and the key is the contract.** `Tally` is written by
    /// the completion seams under exactly these names, so an author's spelling
    /// and a seam's spelling meet here and nowhere else.
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Self::Made(name) => format!("made:{name}"),
            Self::Potions { .. } => "potion".to_owned(),
            Self::Scrolls { .. } => "scroll".to_owned(),
            Self::At { at, .. } => format!("at:{at}"),
            Self::Event { event, .. } => format!("event:{event}"),
        }
    }

    /// How many times the key must have been counted.
    #[must_use]
    pub const fn times(&self) -> u32 {
        match self {
            Self::Made(_) => 1,
            Self::Potions { potions: n } | Self::Scrolls { scrolls: n } => *n,
            Self::At { times, .. } | Self::Event { times, .. } => *times,
        }
    }

    /// Whether this deed names something the game can count.
    ///
    /// # Errors
    ///
    /// A sentence naming what is wrong, for `Progression::check` to wrap: a
    /// product no recipe makes, an instrument nothing runs at, an event nothing
    /// emits, or a count of nought — which would be met before the first tick
    /// and make the station a marker wearing a deed's name.
    pub fn check(&self, outputs: &[&str], instruments: &[&str]) -> Result<(), String> {
        if self.times() == 0 {
            return Err(format!("`{}` asks for nothing to be done", self.key()));
        }
        match self {
            Self::Made(name) if !outputs.contains(&name.as_str()) => Err(format!(
                "`{name}` is not something any recipe makes. One of: {}",
                outputs.join(", "),
            )),
            Self::At { at, .. } if !instruments.contains(&at.as_str()) => Err(format!(
                "`{at}` is not an instrument. One of: {}",
                instruments.join(", "),
            )),
            Self::Event { event, .. } if !crate::tower::EVENTS.contains(&event.as_str()) => {
                Err(format!(
                    "`{event}` is not something a room does. One of: {}",
                    crate::tower::EVENTS.join(", "),
                ))
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Holder {
        done: Deed,
    }

    fn parse(text: &str) -> Result<Deed, toml::de::Error> {
        toml::from_str::<Holder>(text).map(|holder| holder.done)
    }

    #[test]
    fn every_spelling_reads_as_one_key() {
        let cases = [
            ("done = \"clarity\"", "made:clarity", 1),
            ("done = { potions = 5 }", "potion", 5),
            ("done = { scrolls = 3 }", "scroll", 3),
            ("done = { at = \"stacks\" }", "at:stacks", 1),
            ("done = { at = \"stacks\", times = 4 }", "at:stacks", 4),
            ("done = { event = \"figure\" }", "event:figure", 1),
            (
                "done = { event = \"siege_won\", times = 5 }",
                "event:siege_won",
                5,
            ),
        ];
        for (text, key, times) in cases {
            let deed = parse(text).unwrap_or_else(|error| panic!("{text}: {error}"));
            assert_eq!(deed.key(), key, "{text}");
            assert_eq!(deed.times(), times, "{text}");
        }
    }

    /// **A lesson keeps its size at every length**, and a deed not marked keeps
    /// growing with the line — the pair, since either alone passes against a
    /// `stretched` that ignored the flag in one direction.
    #[test]
    fn a_fixed_event_is_the_same_count_at_every_length() {
        use crate::content::Length;
        let fixed = parse("done = { event = \"figure\", times = 5, fixed = true }")
            .unwrap_or_else(|error| panic!("{error}"));
        let grown = parse("done = { event = \"figure\", times = 5 }")
            .unwrap_or_else(|error| panic!("{error}"));
        for length in [Length::Short, Length::Medium, Length::Long] {
            assert_eq!(fixed.stretched(length, 1, 3).times(), 5, "{length:?}");
            assert!(grown.stretched(length, 1, 3).times() > 5, "{length:?}");
        }
        assert!(parse("done = { event = \"figure\", times = 5, fixd = true }").is_err());
    }

    #[test]
    fn a_misspelled_field_fails_rather_than_defaulting() {
        // **The trap `deny_unknown_fields` is for.** `{ at = "stacks", tims = 3 }`
        // would otherwise parse as `At { times: 1 }` — a station met three
        // times too early, with the file correct on its face.
        assert!(parse("done = { at = \"stacks\", tims = 3 }").is_err());
        assert!(parse("done = { potion = 5 }").is_err());
    }

    #[test]
    fn a_deed_names_things_the_game_can_count() {
        let outputs = ["clarity", "warding"];
        let instruments = ["stacks", "alembic"];
        assert!(
            Deed::Made("clarity".into())
                .check(&outputs, &instruments)
                .is_ok()
        );
        assert!(
            Deed::Made("clarty".into())
                .check(&outputs, &instruments)
                .is_err()
        );
        assert!(
            Deed::At {
                at: "stack".into(),
                times: 1,
            }
            .check(&outputs, &instruments)
            .is_err()
        );
        assert!(
            Deed::Event {
                event: "figure".into(),
                times: 1,
                fixed: false,
            }
            .check(&outputs, &instruments)
            .is_ok()
        );
        assert!(
            Deed::Event {
                event: "figures".into(),
                times: 1,
                fixed: false,
            }
            .check(&outputs, &instruments)
            .is_err()
        );
        assert!(
            Deed::Potions { potions: 0 }
                .check(&outputs, &instruments)
                .is_err(),
            "a deed of nought was accepted",
        );
    }
}
