//! A typed word naming one of a set by prefix — §6's forgiveness, written once
//! for the editor, the weave, the menu, the manual and the settings.

/// What `typed` names among `candidates`: a whole word outright, otherwise the
/// one candidate it prefixes, and nothing when two share the prefix — so a
/// list's order never decides what a letter meant.
///
/// Every vocabulary that reaches this holds that no two of its words share a
/// first letter, so any one letter is enough; the settings rows are the one set
/// that cannot promise it, and the reason the ambiguous case refuses.
pub(crate) fn unique_prefix<'a, T>(
    candidates: impl IntoIterator<Item = (&'a str, T)>,
    typed: &str,
) -> Option<T> {
    let mut found = None;
    let mut shared = false;
    for (word, value) in candidates {
        if word == typed {
            return Some(value);
        }
        if word.starts_with(typed) {
            shared |= found.is_some();
            found = found.or(Some(value));
        }
    }
    if shared { None } else { found }
}

/// The word that steps one level up, in the menu and the manual alike.
///
/// It is in no page's vocabulary and is tested before any of them, so each
/// vocabulary's tests keep every prefix of it free.
pub(crate) const BACK: &str = "back";

/// Whether `typed` is [`BACK`], by prefix either way: `bac` is a prefix of it,
/// and a player who typed the whole word and more is still asking for it.
pub(crate) fn is_back(typed: &str) -> bool {
    typed.starts_with(BACK) || BACK.starts_with(typed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: [(&str, u8); 4] = [("save", 1), ("settle", 2), ("set", 3), ("quit", 4)];

    #[test]
    fn a_whole_word_wins_and_a_shared_prefix_names_nothing() {
        assert_eq!(unique_prefix(WORDS, "set"), Some(3), "a whole word");
        assert_eq!(unique_prefix(WORDS, "sett"), Some(2), "one candidate");
        assert_eq!(unique_prefix(WORDS, "q"), Some(4));
        assert_eq!(unique_prefix(WORDS, "s"), None, "three share it");
        assert_eq!(unique_prefix(WORDS, "x"), None);
    }

    #[test]
    fn nothing_typed_names_only_a_set_of_one() {
        // Every caller refuses an empty line before it gets here; this pins
        // what would happen if one did not.
        assert_eq!(unique_prefix(WORDS, ""), None);
        assert_eq!(unique_prefix([("only", 7)], ""), Some(7));
    }
}
