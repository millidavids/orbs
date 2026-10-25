//! The words a spell has that the prompt does not.
//!
//! A second table, matched exactly and before the fuzzy matcher sees the line:
//! in §6's forgiving vocabulary `wait` was a `meditate` synonym and `repeat`
//! reaches `revert`. The prompt still answers for them, as
//! [`Resolution::InSpell`](super::Resolution::InSpell) — a real word in the
//! wrong place — or a word learned in the editor destroys something at the
//! prompt.
//!
//! A spell word beats a verb synonym outright, so a new one is swept against
//! the whole vocabulary: `set` once stopped reaching `dial` (§19).

/// A word that means something in a spell and nothing at the prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpellWord {
    /// Hold here until something happens — §8's smallest control structure,
    /// conditional behaviour without a condition vocabulary.
    Wait,
    /// Do the block again.
    Repeat,
    /// Do the block only if the tower is a certain way.
    If,
    /// The other half of an `if`.
    Else,
    /// Close a block, whatever opened it.
    End,
    /// Bound a `repeat` by a question instead of a number.
    ///
    /// Never the first word of a line. Earns its place by deleting guessed
    /// bounds like `repeat 20000`.
    Until,
    /// Give a name to a place, so a later line can say it — `let best be north`.
    ///
    /// A variable holds a name and nothing else, resolved against the room at
    /// cast (§8), so a word the room cannot place is a fault. Not `set … to`:
    /// `set` is `dial`'s, and `to` is §6 filler.
    Let,
    /// Do the block once for each of something — `for each way`.
    ///
    /// The cursor is named after the group, so the body says `if way has
    /// spoil`. Not `it`, which is §6 filler.
    For,
    /// Name a run of lines so the rest of the spell can say it —
    /// `part gathering()`.
    ///
    /// §10's *composition*, living in the spell that uses it (§19). A call is
    /// punctuation, `gathering()`, not another word — the one place symbols are
    /// canonical. `rite`, `call`, `step`, `make` and `form` all collide.
    Part,
    /// Do nothing for a number of ticks.
    ///
    /// [`Wait`](Self::Wait) takes a thing; this is the one count the world does
    /// not hand a spell (§19). Not `rest` (`meditate`'s), `tarry` or `pause`.
    Bide,
    /// Take the oldest name out of a satchel and bind it —
    /// `pull note from satchel`.
    ///
    /// A word, not a verb, because it binds a name, which only the language can
    /// do; the push half is the verb `queue`. It yields while empty and never
    /// reaches [`PATIENCE`](crate::tower::spell::PATIENCE): a consumer caught up
    /// with its producer is not a fault. Not `draw`, `take`, `lift` or `pop`.
    Pull,
    /// Set a part running as a second cursor and carry on —
    /// `alongside gathering()`.
    ///
    /// A fork, not a call: it starts a [`Strand`](crate::tower::spell::Strand)
    /// with its own position, bindings and budget, seeing only its brackets
    /// (§19). The spell ends when every cursor has. Not `fork` (962 against
    /// `for`), `split` or `beside`.
    Alongside,
}

impl SpellWord {
    /// Every one, for the naming pass and for the prompt's answer.
    pub const ALL: [Self; 12] = [
        Self::Wait,
        Self::Repeat,
        Self::If,
        Self::Else,
        Self::End,
        Self::Until,
        Self::Let,
        Self::For,
        Self::Part,
        Self::Bide,
        Self::Pull,
        Self::Alongside,
    ];

    /// The word as it is written in a spell.
    #[must_use]
    pub const fn canonical(self) -> &'static str {
        match self {
            Self::Wait => "wait",
            Self::Repeat => "repeat",
            Self::If => "if",
            Self::Else => "else",
            Self::End => "end",
            Self::Until => "until",
            Self::Let => "let",
            Self::For => "for",
            Self::Part => "part",
            Self::Bide => "bide",
            Self::Pull => "pull",
            Self::Alongside => "alongside",
        }
    }

    /// What follows the word, for a listing that shows how it is used — the
    /// table `recall`'s overview prints too. Empty where nothing follows.
    #[must_use]
    pub const fn shape(self) -> &'static str {
        match self {
            Self::Repeat => "<count>",
            Self::Until | Self::If => "<question>",
            Self::Wait => "<thing>",
            Self::Let => "<name> be <place>",
            Self::For => "each <set>",
            // Any number of names, so neither two nor none.
            Self::Part => "<name>(...)",
            Self::Bide => "<count>",
            // `from` is filler the parser drops, shown because a reader wants it.
            Self::Pull => "<name> from <place>",
            // A call, so the brackets are the notation.
            Self::Alongside => "<name>(...)",
            Self::Else | Self::End => "",
        }
    }

    /// Whether this word opens a block that an `end` must close.
    ///
    /// Not `Until`, which sits on `repeat`'s line, nor `Let`, one line.
    #[must_use]
    pub const fn opens_block(self) -> bool {
        matches!(self, Self::Repeat | Self::If | Self::For | Self::Part)
    }

    /// The word that must follow this one, if the grammar fixes it — `for
    /// each way`, since `for` alone reads as the preposition.
    #[must_use]
    pub const fn particle(self) -> Option<&'static str> {
        match self {
            Self::For => Some("each"),
            _ => None,
        }
    }
}

/// The spell word `line` begins with, if any.
///
/// Exact on the first word, case-folded. Not fuzzy: guessing wrong rewrites the
/// player's file, because canonicalisation runs at save.
#[must_use]
pub fn leading(line: &str) -> Option<SpellWord> {
    let first = line.split_whitespace().next()?.to_lowercase();
    SpellWord::ALL
        .into_iter()
        .find(|word| word.canonical() == first)
}

/// One level of indentation, as the orb writes it.
///
/// Spaces, because the editor filters control characters out of the buffer and
/// a spell is read back at a fixed cell width.
pub const INDENT: &str = "    ";

/// How `line` sits in its blocks: the depth it prints at, and the depth the line
/// after it starts at.
///
/// One rule for the orb writing a spell down and the editor indenting as you
/// type, or every save looks like it moved your work. `end` and `else` sit
/// level with their opener; a blank line or a comment leaves the depth alone.
#[must_use]
pub fn indent_around(line: &str, depth: usize) -> (usize, usize) {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return (depth, depth);
    }
    let Some(word) = leading(line) else {
        return (depth, depth);
    };
    // `else` does both: it closes the branch above and opens the one below.
    let here = if matches!(word, SpellWord::End | SpellWord::Else) {
        depth.saturating_sub(1)
    } else {
        depth
    };
    let next = if word.opens_block() || word == SpellWord::Else {
        here + 1
    } else {
        here
    };
    (here, next)
}

/// What a spell word was given, after the word itself.
///
/// `wait for the mortar` → `for the mortar`. Filler is **not** stripped here:
/// §6's normaliser does that at resolution, so there is one answer to what
/// `the` means.
#[must_use]
pub fn argument(line: &str) -> &str {
    let trimmed = line.trim();
    let after = trimmed.split_whitespace().next().map_or(0, str::len);
    trimmed[after..].trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spell_word_is_recognised_by_its_first_word_only() {
        assert_eq!(leading("wait for the mortar"), Some(SpellWord::Wait));
        assert_eq!(leading("  REPEAT 3  "), Some(SpellWord::Repeat));
        assert_eq!(leading("end"), Some(SpellWord::End));
        assert_eq!(leading("grind sage"), None);
    }

    #[test]
    fn a_near_miss_is_not_a_spell_word() {
        // Canonicalisation runs at save, so a guess would be a line they never
        // wrote.
        for typo in ["waat", "repeet", "ends", "wai"] {
            assert_eq!(leading(typo), None, "{typo:?} was guessed at");
        }
    }

    #[test]
    fn the_argument_is_whatever_followed_the_word() {
        assert_eq!(argument("wait for the mortar"), "for the mortar");
        assert_eq!(argument("repeat 3"), "3");
        assert_eq!(argument("end"), "");
    }

    #[test]
    fn no_spell_word_is_also_a_verb() {
        // Or one line would mean two things, and `Resolution::InSpell` would lie.
        for word in SpellWord::ALL {
            assert!(
                !crate::parser::Verb::ALL
                    .iter()
                    .any(|verb| verb.canonical() == word.canonical()),
                "{} is both a spell word and a verb",
                word.canonical(),
            );
        }
    }
}
