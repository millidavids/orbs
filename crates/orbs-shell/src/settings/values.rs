//! What a settings page is made of: rows, and the words that set them.
//!
//! A [`Row`] is data the frontend fills — a word, its value, its values — since
//! the builds differ. *Unsupported* is fewer rows, and a page with none is not
//! offered (§15). A word cycles a setting; a word and value set it.

/// Which page of the settings a row belongs to.
///
/// Pages rather than one long list: `menu/paint.rs` sizes its floor for the
/// tallest page, and one list of everything would raise that floor past what
/// 80×22 — the smallest screen the game supports — can host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// How loud the orb is, and whether a key clicks.
    ///
    /// Arrived with the voice, not before it: a page of volumes for a game that
    /// makes no sound is the dead affordance §15 weighs heaviest.
    Sound,
    /// The glass: the tube, and the colour of the phosphor.
    Tube,
    /// Each of the tube's effects on its own, as a ceiling on the tube's state.
    Picture,
    /// §14's accommodations: hue, the linear stream, motion.
    Access,
    /// How the orb behaves: how it reads you, how it divides the screen, and
    /// whether it plays its boot sequence.
    Habits,
}

impl Category {
    /// Every page, in the order the settings page offers them.
    pub const ALL: [Self; 5] = [
        Self::Sound,
        Self::Tube,
        Self::Picture,
        Self::Access,
        Self::Habits,
    ];

    /// The word a player types for it.
    ///
    /// No two share a first letter, which `menu::words`' test holds along with
    /// every other page's vocabulary.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Sound => "sound",
            Self::Tube => "tube",
            Self::Picture => "picture",
            Self::Access => "access",
            Self::Habits => "habits",
        }
    }

    /// The page `typed` names, by unambiguous prefix.
    #[must_use]
    pub fn named(typed: &str) -> Option<Self> {
        crate::prefix::unique_prefix(Self::ALL.map(|page| (page.word(), page)), typed)
    }
}

/// One setting, as a settings page shows it.
///
/// Owned strings rather than `&'static str`, because a theme's name comes from
/// the frontend's palette and a value word can be a number. The cost is a
/// handful of allocations when a page opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Which page it is on.
    pub page: Category,
    /// The word a player types to reach it.
    pub word: String,
    /// What it is called in the settings file.
    ///
    /// [`word`](Self::word) unless [`keyed`](Self::keyed): the player types
    /// `reading`, the file says `driver`.
    pub key: String,
    /// What it is set to, as one of [`values`](Self::values).
    pub value: String,
    /// Every value it will take, in the order cycling visits them.
    pub values: Vec<String>,
    /// Words it answers to and never offers — §3's *discovered, never pushed*.
    pub secrets: Vec<Secret>,
}

/// A word a row answers to that it never lists, cycles or matches by prefix.
///
/// Typed whole or not at all, so `theme p` stays a question about the themes
/// the page shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Secret {
    /// What is typed after the row's word.
    pub word: String,
    /// What it sets the row to.
    pub value: String,
    /// The prose key of what the menu says back, if anything.
    pub remark: Option<String>,
}

impl Row {
    /// A row for `word`, currently `value`, choosing among `values`.
    #[must_use]
    pub fn new(page: Category, word: &str, value: &str, values: &[&str]) -> Self {
        Self {
            page,
            word: word.to_owned(),
            key: word.to_owned(),
            value: value.to_owned(),
            values: values.iter().map(|value| (*value).to_owned()).collect(),
            secrets: Vec::new(),
        }
    }

    /// Answer `word` with `value`, and say `remark` if there is one — without
    /// ever offering it.
    #[must_use]
    pub fn secretly(mut self, word: &str, value: &str, remark: Option<&str>) -> Self {
        self.secrets.push(Secret {
            word: word.to_owned(),
            value: value.to_owned(),
            remark: remark.map(ToOwned::to_owned),
        });
        self
    }

    /// The secret `typed` is, whole.
    #[must_use]
    pub fn secret(&self, typed: &str) -> Option<&Secret> {
        self.secrets.iter().find(|secret| secret.word == typed)
    }

    /// Keep this row under `key` rather than under its word.
    ///
    /// For a setting whose file name a player never sees and whose word they
    /// type — see [`key`](Self::key).
    #[must_use]
    pub fn keyed(mut self, key: &str) -> Self {
        self.key = key.to_owned();
        self
    }

    /// The value after this one, wrapping.
    ///
    /// Starts from whatever is set, so a value this build no longer offers still
    /// lands somewhere: a settings file written by a later build can name a
    /// theme this one does not have.
    #[must_use]
    pub fn next(&self) -> Option<&str> {
        if self.values.is_empty() {
            return None;
        }
        let at = self
            .values
            .iter()
            .position(|value| *value == self.value)
            .map_or(0, |at| (at + 1) % self.values.len());
        self.values.get(at).map(String::as_str)
    }

    /// The value `typed` names, by unambiguous prefix.
    ///
    /// Unambiguous, enforced; an exact match wins. The frontend's values are
    /// checked where the rows are built
    /// (`every_value_on_every_row_is_reachable_by_a_prefix`).
    #[must_use]
    pub fn value_named(&self, typed: &str) -> Option<&str> {
        crate::prefix::unique_prefix(
            self.values
                .iter()
                .map(|value| (value.as_str(), value.as_str())),
            typed,
        )
    }
}

/// The two words an on/off setting takes.
///
/// One spelling, shared, so `linear on` and `motion on` cannot come to disagree,
/// and so the file does not say `true` for some keys and `1` for others.
pub const ON: &str = "on";
/// The other one. See [`ON`].
pub const OFF: &str = "off";

/// [`ON`] and [`OFF`] as a list, for [`Row::new`].
pub const SWITCH: [&str; 2] = [ON, OFF];

/// How loud a thing is, quietest first.
///
/// Four words, since a typed menu has no slider. `off` first, the one people
/// look for; no two share a first letter.
pub const LEVELS: [&str; 4] = [OFF, "quiet", "half", "full"];

/// How loud [`LEVELS`] mean, as a multiplier.
///
/// Here rather than in the frontend so that a terminal build gaining audio one
/// day cannot pick different numbers for the same words. Perceived loudness is
/// closer to logarithmic than linear, which is why `half` is not 0.5.
#[must_use]
pub fn loudness(level: &str) -> f32 {
    match level {
        OFF => 0.0,
        "quiet" => 0.15,
        "half" => 0.4,
        _ => 1.0,
    }
}

/// `on` or `off`, as a switch row reports it.
#[must_use]
pub const fn switched(on: bool) -> &'static str {
    if on { ON } else { OFF }
}

/// Whether a value word means on.
#[must_use]
pub fn is_on(value: &str) -> bool {
    value == ON
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ambiguous_prefix_is_refused_rather_than_guessed() {
        // The shipped defect, with the real list that caused it.
        let row = Row::new(
            Category::Tube,
            "theme",
            "amber",
            &["amber", "green", "muted violet", "monochrome"],
        );
        assert_eq!(row.value_named("m"), None, "`m` guessed between two values");
        assert_eq!(row.value_named("mu"), Some("muted violet"));
        assert_eq!(row.value_named("mo"), Some("monochrome"));
        assert_eq!(row.value_named("a"), Some("amber"), "`a` is unambiguous");
        assert_eq!(row.value_named("zorb"), None);
    }

    #[test]
    fn an_exact_value_wins_over_a_longer_one_it_prefixes() {
        // Otherwise a value that is a prefix of another is unreachable, which is
        // the same defect one step along.
        let row = Row::new(Category::Habits, "when", "on", &["on", "once", "off"]);
        assert_eq!(row.value_named("on"), Some("on"));
        assert_eq!(row.value_named("onc"), Some("once"));
        assert_eq!(row.value_named("of"), Some("off"));
    }

    #[test]
    fn no_two_pages_share_a_first_letter() {
        // The prefix rule every other page in the menu keeps.
        let mut firsts: Vec<char> = Category::ALL
            .iter()
            .filter_map(|page| page.word().chars().next())
            .collect();
        firsts.sort_unstable();
        let before = firsts.len();
        firsts.dedup();
        assert_eq!(before, firsts.len(), "two settings pages share a letter");
    }

    #[test]
    fn a_page_is_named_by_its_shortest_prefix() {
        for page in Category::ALL {
            let first = &page.word()[..1];
            assert_eq!(Category::named(first), Some(page), "`{first}`");
            assert_eq!(Category::named(page.word()), Some(page));
        }
        assert_eq!(Category::named("zorb"), None);
    }

    #[test]
    fn cycling_wraps_and_a_stranger_value_still_lands_somewhere() {
        let row = Row::new(Category::Tube, "crt", "off", &["default", "peak", "off"]);
        assert_eq!(row.next(), Some("default"), "the last did not wrap");

        // A value this build does not offer: a later build's settings file can
        // name a theme this one has never heard of, and cycling from it must
        // still reach a real value.
        let stranger = Row::new(Category::Tube, "theme", "aurora", &["amber", "violet"]);
        assert_eq!(stranger.next(), Some("amber"));

        // A row with nothing to choose among cycles to nothing rather than
        // panicking — what a frontend with no values for a setting builds.
        let empty = Row::new(Category::Tube, "theme", "amber", &[]);
        assert_eq!(empty.next(), None);
    }

    #[test]
    fn a_value_is_named_by_its_shortest_prefix() {
        let row = Row::new(
            Category::Tube,
            "crt",
            "default",
            &["default", "peak", "off"],
        );
        assert_eq!(row.value_named("p"), Some("peak"));
        assert_eq!(row.value_named("off"), Some("off"));
        assert_eq!(row.value_named("zorb"), None);
    }

    #[test]
    fn the_switch_words_round_trip() {
        assert_eq!(switched(true), ON);
        assert_eq!(switched(false), OFF);
        assert!(is_on(switched(true)));
        assert!(!is_on(switched(false)));
        assert_eq!(SWITCH, [ON, OFF]);
    }
}
