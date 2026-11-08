//! The orb never speaks as its maker (DESIGN.md §3, *Both at once*, rule 4).

use super::{Manual, Prose};

/// The developer's words, matched whole and in any case. `built` alone is a
/// tower's word, so only the two phrases that promise a feature.
const MAKERS: [&str; 5] = ["authored", "not built", "built yet", "tier", "game"];

/// The first of [`MAKERS`] in `text`, as a whole word or phrase.
fn maker_word(text: &str) -> Option<&'static str> {
    let lower = text.to_lowercase();
    MAKERS.into_iter().find(|word| {
        lower.match_indices(word).any(|(at, _)| {
            let before = lower[..at].chars().next_back();
            let after = lower[at + word.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        })
    })
}

#[test]
fn the_orb_never_speaks_as_its_maker() {
    for (key, line) in Prose::builtin().every_line() {
        if let Some(word) = maker_word(line) {
            panic!("{key} says '{word}', the maker's word rather than the orb's: {line:?}");
        }
    }
}

/// The handbook is the orb's too; `notices` names what it is built from, and
/// may say what it is.
#[test]
fn the_handbook_never_speaks_as_its_maker() {
    for chapter in Manual::builtin().chapters() {
        if chapter.name == "notices" {
            continue;
        }
        for text in std::iter::once(&chapter.title).chain(&chapter.lines) {
            if let Some(word) = maker_word(text) {
                panic!("{} says '{word}', the maker's word: {text:?}", chapter.name);
            }
        }
    }
}

#[test]
fn a_maker_word_is_found_only_whole() {
    assert_eq!(maker_word("a short game"), Some("game"));
    assert_eq!(maker_word("not built yet."), Some("not built"));
    assert_eq!(maker_word("the next Tier"), Some("tier"));
    for clean in ["endgame", "frontier", "games", "a tower built of stone"] {
        assert_eq!(maker_word(clean), None, "{clean:?}");
    }
}
