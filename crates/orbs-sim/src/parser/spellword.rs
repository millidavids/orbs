//! The words a spell has that the prompt does not.
//!
//! A second table, matched exactly. §6's vocabulary is fuzzy and forgiving, and
//! control words cannot join it: `wait` is already a `meditate` synonym,
//! `repeat` reaches `revert`, `end` means nothing at all. So they are
//! spell-only, checked before the fuzzy matcher sees the line — the editor's
//! `:wq` table's shape, and its reason: *"one merged list would resolve `w` and
//! `wq` by whichever was listed first."*
//!
//! The prompt has to answer for them, or §6's hole is real: `wait for the
//! mortar` typed at the prompt used to open the editor on a new empty
//! `mortar.spell`, and `repeat 3` resolved to `undo`. A player learns a word in
//! the editor, types it at the prompt, and destroys something. So the prompt
//! names them — [`Resolution::InSpell`](super::Resolution::InSpell), a real word
//! in the wrong place, answered rather than guessed at.

/// A word that means something in a spell and nothing at the prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpellWord {
    /// Hold here until something happens. §8's smallest control structure, and
    /// the one that gives a non-programmer conditional behaviour without a
    /// condition vocabulary.
    Wait,
    /// Do the block again.
    Repeat,
    /// Do the block only if the tower is a certain way.
    ///
    /// Last of the four, deliberately: it is the only one needing a vocabulary
    /// for *state* rather than for what just happened, and the Autonauts
    /// precedent is that a wait covers most of it first.
    If,
    /// The other half of an `if`.
    Else,
    /// Close a block, whatever opened it.
    End,
    /// Bound a `repeat` by a question instead of a number.
    ///
    /// Never the first word of a line: it is `repeat`'s argument, so a line
    /// beginning with it is a real word in the wrong place — what [`SpellWord`]
    /// exists to answer.
    ///
    /// The sixth control word, and the count is defended rather than spent. It
    /// earns its place by *deleting* something: every loop that wanted "until"
    /// had to guess a bound, and the shipped solver still says `repeat 20000`.
    Until,
    /// Give a name to a place, so a later line can say it — `let best be north`.
    ///
    /// A variable holds a name and nothing else: not a number, not a list, not
    /// an expression. That is the smallest thing *"follow the way with the
    /// fewest marks"* needs — an accumulator to compare against and act on.
    ///
    /// The value resolves against the room at cast, like every other name in a
    /// spell (§8), so `let m be mortar` binds `mortar_and_pestle` and a word the
    /// room cannot place is a fault rather than a variable holding a typo.
    ///
    /// It was `set … to` for an afternoon. `set` is already a `dial` synonym and
    /// a spell word is matched before the fuzzy matcher, so `set second borax`
    /// stopped reaching the lens — a fourth collision, and a shipped verb's.
    /// `be` rather than `to` because `to` is on §6's filler list, so a keyword
    /// `to` is invisible to every reader but the one that sees the text before
    /// normalisation.
    Let,
    /// Do the block once for each of something — `for each way`.
    ///
    /// The cursor is named after the group, so `for each way` binds `way` and
    /// the body says `if way has spoil` — nothing to learn beyond the group's
    /// own word. `it` cannot be the cursor: it is on §6's filler list, so
    /// `follow it` is stripped to `follow` before anything sees it.
    For,
    /// Name a run of lines so the rest of the spell can say it —
    /// `part gathering()`.
    ///
    /// §10 calls this *"composition — build spells from components"* and the
    /// design calls the component a part; a different word would give one phase
    /// two names for one idea. A part lives in the spell that uses it, since a
    /// spell is one `.spell` file (§19), so the word names the whole of
    /// composition rather than half of it.
    ///
    /// It also survived a sweep against every word the game knows: `rite` scores
    /// 800 against `write`, `call` 750 against `wall`, `step` 750 against
    /// `stop`, `make` 750 against `take`, `form` 750 against `for`, a control
    /// word already. `part` scores 500, against `cast` and `east`. `to` and
    /// `set` were refused before the sweep — filler, and a live `dial` synonym
    /// that [`Let`](Self::Let) records the cost of.
    ///
    /// A call is punctuation, not a ninth word: `gathering()`'s brackets are the
    /// whole notation. A bare name would need a rule — *a part may not be called
    /// something the tower already has a word for* — and a second keyword would
    /// cost a word in a language that argues its count one entry at a time. It
    /// is the one place symbols are canonical rather than merely accepted, since
    /// §19's comparison spellings write back to a word and here none exists.
    Part,
    /// Do nothing for a number of ticks.
    ///
    /// The tenth word, earned by the menagerie's chant. Nothing else counted
    /// ticks: [`Wait`](Self::Wait) takes a *thing* and blocks by scanning the
    /// record stream, right for *"hold until the mortar is free"* and wrong for
    /// a chant, whose puzzle was deciding *when* (§19). The chant is gone and
    /// the word stays — a delay a spell works out is still the one count the
    /// world does not hand it.
    ///
    /// `rest` was the first name and is a `meditate` synonym: a spell word is
    /// matched before the fuzzy matcher, so it would have stopped `rest 20`
    /// reaching `meditate` at the prompt — the collision §19 records `set`
    /// causing against `dial`, and why a proposed word is swept against the
    /// whole vocabulary. `tarry` then scored 800 against `carry` and `pause` 667
    /// against `peruse`; `bide` is clean, one syllable, and in the register of
    /// `muster` and `kindle`.
    Bide,
    /// Take the oldest name out of a satchel and bind it —
    /// `pull note from satchel`.
    ///
    /// A control word rather than a verb because it binds a name, which only the
    /// language can do: [`Let`](Self::Let) is the other, and the two are the
    /// whole of what puts a word in `vars`. A verb runs through
    /// `execute::dispatch`, which touches nothing a spell is holding, so a
    /// `pull` verb could empty the satchel and have nowhere to put what it took.
    /// The push half changes the world, which is a verb's job and lets a player
    /// load a satchel by hand; the pull half changes the *spell*.
    ///
    /// It yields while empty and never reaches
    /// [`PATIENCE`](crate::tower::spell::PATIENCE): a consumer caught up with
    /// its producer is the ordinary state of a working pipeline, not a fault, so
    /// it takes `bide`'s road — *"a spell waiting for ever is a fault; a spell
    /// counting to three is doing what it was written to do"*. What bounds it is
    /// the work running out.
    ///
    /// Swept on both axes: `pull` is clean on similarity and `pul` is a free
    /// prefix. `draw` was the first choice and is wrong in the prose — the game
    /// spends it on producing a *new* random thing three times over, and a queue
    /// hands back something already there. `take` is `weave`'s, `lift` scores
    /// 750 against `sift` and `list`, and `pop` reads as jargon beside `muster`
    /// and `kindle`.
    Pull,
    /// Set a part running as a second cursor and carry on —
    /// `alongside gathering()`.
    ///
    /// Two spells already run at once: `invoke` from inside a spell inserts a
    /// second `Running` without blocking the caller. So this adds no concurrency
    /// — it adds concurrency *within one file*, which is what puts a satchel's
    /// producer and consumer next to each other, sharing a name and edited
    /// together.
    ///
    /// A fork is not a call. `gathering()` suspends the caller until the part
    /// returns; `alongside gathering()` leaves the caller where it is and starts
    /// a [`Strand`](crate::tower::spell::Strand) with its own position, bindings
    /// and budget. It stops when it runs off the end, and the spell ends when
    /// every cursor has. Arguments work as a call's, because a part's brackets
    /// are the whole of what it can see (§19) — a fork reading the caller's
    /// store would be the shared-variables shape that decision removed.
    ///
    /// Swept on both axes: `alongside` is clean and `alo` is a free prefix;
    /// `meanwhile` was the other candidate. `fork` scores 962 against `for`, a
    /// control word already, `split` 600 against `spoil`, and `beside` 667
    /// against `bide`, the word most likely to sit on the line above it.
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

    /// What follows the word, for a listing that has to show how it is used.
    ///
    /// The same table `recall`'s overview prints: two answers to *what does
    /// `for` take* is one of them being wrong later. It lived in `orbs-shell`
    /// for one version, which put a painter in the business of knowing the
    /// grammar.
    ///
    /// `else` and `end` take nothing, and the empty string is the honest answer
    /// rather than an absence a caller has to special-case.
    #[must_use]
    pub const fn shape(self) -> &'static str {
        match self {
            Self::Repeat => "<count>",
            Self::Until | Self::If => "<question>",
            Self::Wait => "<thing>",
            Self::Let => "<name> be <place>",
            Self::For => "each <set>",
            // `(...)` rather than `()` or `(<name>, <name>)`: a part takes any
            // number of names, so showing two would read as a rule and showing
            // none taught the wrong form.
            Self::Part => "<name>(...)",
            // `<count>`, the same word `repeat` uses, because it is the same
            // thing being counted from the player's side: a number of times
            // round against a number of ticks through.
            Self::Bide => "<count>",
            // `from` is on §6's filler list and is written anyway, unlike
            // `let`'s `be`, which is load-bearing. This is decoration a reader
            // wants and the parser never sees: `pull note satchel` is the same
            // line.
            Self::Pull => "<name> from <place>",
            // The same shape `part` shows, because what follows is a call and
            // the brackets are the notation. `<part>` would teach a bare name,
            // the one thing a call may not be.
            Self::Alongside => "<name>(...)",
            Self::Else | Self::End => "",
        }
    }

    /// Whether this word opens a block that an `end` must close.
    ///
    /// `Until` does not, though it is always inside one: the block is `repeat`'s,
    /// and counting it here would want a second `end`. `Let` does not either —
    /// one line binding one name, with nothing after it to close.
    #[must_use]
    pub const fn opens_block(self) -> bool {
        matches!(self, Self::Repeat | Self::If | Self::For | Self::Part)
    }

    /// The word that must follow this one, if the grammar fixes it.
    ///
    /// `for each way` and never `for way`: the second word carries no
    /// information and is required anyway, because `for` alone reads as the
    /// preposition it is everywhere else in English and the sentence would be
    /// ambiguous to a person long before it was to the parser.
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
/// Spaces rather than a tab, because the editor filters control characters out
/// of the buffer (`control_characters_never_reach_the_buffer`) and a spell is a
/// file a player reads back at a fixed cell width.
pub const INDENT: &str = "    ";

/// How `line` sits in its blocks: the depth it prints at, and the depth the line
/// after it starts at.
///
/// One rule, two callers — the orb re-indents a spell when it writes it down and
/// the editor indents as you type. A buffer and a saved file that disagree make
/// every save look like it moved your work, so the rule lives in the crate that
/// owns the language.
///
/// `end` and `else` sit level with the `if` or `repeat` they belong to: an
/// `else` indented as body reads as a step *inside* the branch it ends.
///
/// A blank line or a comment leaves the depth alone, which lets the editor put
/// the caret at the body's indent on an empty line inside a block.
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
/// `wait for the mortar` → `the mortar`; `repeat 3` → `3`. Filler is **not**
/// stripped here — the argument goes to §6's own normaliser when it is resolved,
/// so there is one answer to what `the` means.
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
        // Fuzzy-matching here would let a typo rewrite the player's file:
        // canonicalisation runs at save, so `waat` becoming `wait` is a line
        // they never wrote.
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
        // The reason these live in their own table. If a word were both, the
        // same line would mean one thing in a spell and another at the prompt —
        // and the prompt's answer (`Resolution::InSpell`) would be a lie.
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
