//! Lines the orb answers and never offers (DESIGN.md §3, §19 *Both tongues*).
//!
//! A shell's reflexes and the old jokes, matched before the matcher sees the
//! line: `sudo` fuzzed to `study` and so to `research`, and `exit` asked about
//! `scribe`. A pointer names the orb's own word, standing in for a shell synonym
//! the readers would have to be retrained to learn. A joke is §3's comedy, and
//! is silent in a tense moment.

use super::normalise::{Tokens, Word, skip_leading_filler};

/// What an aside is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Discovered, never pushed, and never said in a tense moment (§3).
    Joke,
    /// Another shell's word, answered with the orb's.
    Pointer,
}

/// A line the orb answers without running anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Aside {
    /// The prose key it is answered with.
    pub key: &'static str,
    /// A joke or a pointer.
    pub kind: Kind,
    /// Matched as the line's opening word, with the rest of the line after it.
    pub opening: bool,
}

/// Whole lines, matched once folded as the matcher folds a line.
const WHOLE: [(&str, &str, Kind); 16] = [
    // Colossal Cave's magic word, which does exactly what it did there.
    ("xyzzy", "aside_xyzzy", Kind::Joke),
    ("plugh", "aside_plugh", Kind::Joke),
    ("hello sailor", "aside_hello_sailor", Kind::Joke),
    ("make me a sandwich", "aside_sandwich", Kind::Joke),
    ("sudo make me a sandwich", "aside_sudo_sandwich", Kind::Joke),
    ("rm -rf /", "aside_rm_root", Kind::Joke),
    ("exit", "aside_exit", Kind::Pointer),
    ("ps", "aside_ps", Kind::Pointer),
    ("top", "aside_top", Kind::Pointer),
    ("whoami", "aside_whoami", Kind::Pointer),
    ("pwd", "aside_pwd", Kind::Pointer),
    ("clear", "aside_clear", Kind::Pointer),
    ("reboot", "aside_reboot", Kind::Pointer),
    ("shutdown", "aside_shutdown", Kind::Pointer),
    ("fsck", "aside_fsck", Kind::Pointer),
    ("date", "aside_date", Kind::Pointer),
];

/// Words that answer for the whole line when they open it. `kill <spell>` once
/// read as `invoke <spell>`, which starts the thing asked to be ended.
const LEADING: [(&str, &str, Kind); 2] = [
    ("sudo", "aside_sudo", Kind::Joke),
    ("kill", "aside_kill", Kind::Pointer),
];

/// The aside `input` is, if it is one. Whole lines first, so `sudo make me a
/// sandwich` is not `sudo`'s.
///
/// Folded as the matcher folds, or `exit.` and `please sudo hold` missed the
/// table and went back to being fuzzed.
#[must_use]
pub fn aside(input: &str) -> Option<Aside> {
    among(&Tokens::split(input).words())
}

/// [`aside`], of a line already split.
pub(super) fn among(all: &[Word<'_>]) -> Option<Aside> {
    // Compared word by word, so a line the table does not hold costs no
    // allocation on its way to the matcher.
    let words = &all[skip_leading_filler(all)..];
    let folded = || words.iter().map(|word| word.matching);
    if let Some((_, key, kind)) = WHOLE.iter().find(|(said, ..)| said.split(' ').eq(folded())) {
        return Some(Aside {
            key,
            kind: *kind,
            opening: false,
        });
    }
    let first = words.first()?.matching;
    let (_, key, kind) = LEADING.iter().find(|(word, ..)| *word == first)?;
    Some(Aside {
        key,
        kind: *kind,
        opening: true,
    })
}

/// The aside `input` is when it is said alone: a whole line, or an opening
/// word with nothing after it. The menu's, which takes no command after one.
#[must_use]
pub fn alone(input: &str) -> Option<Aside> {
    aside(input).filter(|found| !found.opening || rest_of(input).is_empty())
}

/// What follows an opening-word aside, as typed: `hold`, of `sudo, hold`.
#[must_use]
pub fn rest_of(input: &str) -> String {
    let tokens = Tokens::split(input);
    let all = tokens.words();
    let opening = skip_leading_filler(&all);
    all.iter()
        .skip(opening + 1)
        .map(|word| word.raw)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Every line or opening word with its aside, whole lines first.
pub fn every() -> impl Iterator<Item = (&'static str, Aside)> {
    let whole = WHOLE.iter().map(|line| (line, false));
    let leading = LEADING.iter().map(|word| (word, true));
    whole.chain(leading).map(|((said, key, kind), opening)| {
        (
            *said,
            Aside {
                key,
                kind: *kind,
                opening,
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{Phrasings, Prose};
    use crate::parser::{SYNONYMS, Verb};

    /// Shadowed on purpose, each said in §19: a whole line that reaches a verb
    /// today with words to spare, or an opening word the readers were taught.
    const SHADOWS: [&str; 5] = [
        "make me a sandwich",
        "sudo make me a sandwich",
        "hello sailor",
        "rm -rf /",
        "kill",
    ];

    #[test]
    fn every_aside_has_a_line() {
        let prose = Prose::builtin();
        for (said, aside) in every() {
            assert!(
                prose.has(aside.key),
                "{said:?} answers with {}, which is not written",
                aside.key
            );
        }
    }

    #[test]
    fn a_whole_line_is_matched_before_its_opening_word() {
        assert_eq!(
            aside("sudo make me a sandwich").map(|a| a.key),
            Some("aside_sudo_sandwich")
        );
        assert_eq!(aside("sudo grind sage").map(|a| a.key), Some("aside_sudo"));
        assert_eq!(
            aside("  Kill   first_light ").map(|a| a.key),
            Some("aside_kill")
        );
        assert_eq!(
            aside("exit now"),
            None,
            "a whole line matched with words after it"
        );
        assert_eq!(aside("grind sage"), None);
    }

    #[test]
    fn an_aside_is_folded_as_the_matcher_folds_a_line() {
        for (said, key) in [
            ("exit.", "aside_exit"),
            ("ps?", "aside_ps"),
            ("Hello, sailor!", "aside_hello_sailor"),
            ("please make me a sandwich", "aside_sandwich"),
            ("please sudo hold", "aside_sudo"),
            ("sudo, hold", "aside_sudo"),
        ] {
            assert_eq!(aside(said).map(|a| a.key), Some(key), "{said:?}");
        }
        assert_eq!(rest_of("please sudo, hold the wall"), "hold the wall");
        assert_eq!(rest_of("sudo"), "");
    }

    #[test]
    fn alone_takes_an_opening_word_only_with_nothing_after_it() {
        assert_eq!(alone("sudo").map(|a| a.key), Some("aside_sudo"));
        assert_eq!(alone("sudo.").map(|a| a.key), Some("aside_sudo"));
        assert_eq!(alone("sudo hold"), None);
        assert_eq!(
            alone("sudo make me a sandwich").map(|a| a.key),
            Some("aside_sudo_sandwich")
        );
    }

    #[test]
    fn no_aside_takes_a_word_the_orb_already_has() {
        // A real synonym wins over an aside, so when one arrives this fails and
        // the aside it replaces goes.
        let phrases: Vec<String> = SYNONYMS
            .iter()
            .map(|synonym| synonym.words.join(" "))
            .collect();
        let canonical: Vec<&str> = Verb::ALL.iter().map(|verb| verb.canonical()).collect();
        let openers: Vec<&str> = SYNONYMS
            .iter()
            .filter_map(|synonym| synonym.words.first().copied())
            .collect();
        let taught: Vec<String> = Phrasings::builtin()
            .entries()
            .iter()
            .flat_map(|entry| entry.say.iter().chain(&entry.holdout))
            .cloned()
            .collect();
        let unshadowed = |said: &&str| !SHADOWS.contains(said);
        for said in WHOLE.iter().map(|(said, ..)| *said).filter(unshadowed) {
            assert!(
                !phrases.iter().any(|phrase| phrase == said),
                "{said:?} is a synonym"
            );
            assert!(!canonical.contains(&said), "{said:?} is a verb");
            assert!(
                !taught.iter().any(|line| line == said),
                "{said:?} is a phrasing the readers learn"
            );
        }
        for word in LEADING.iter().map(|(word, ..)| *word).filter(unshadowed) {
            assert!(!openers.contains(&word), "{word:?} opens a synonym");
            assert!(
                !taught
                    .iter()
                    .any(|line| line.split_whitespace().next() == Some(word)),
                "{word:?} opens a phrasing the readers learn",
            );
        }
    }
}
