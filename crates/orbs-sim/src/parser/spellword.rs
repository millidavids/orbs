//! The words a spell has that the prompt does not.
//!
//! # Why these are a second table, and an exactly-matched one
//!
//! §6's vocabulary is fuzzy-matched, three-register, and forgiving, and a naming
//! pass exists to stop two words meaning two things. Control words cannot join
//! it: `wait` is already a `meditate` synonym, `repeat` reaches `revert`, and
//! `end` means nothing at all. Adding them fuzzily would put five new collisions
//! into a vocabulary whose whole naming pass is about not having any.
//!
//! So they are **spell-only**, matched **exactly**, and checked before the
//! fuzzy matcher ever sees the line — the same shape as the editor's `:wq`
//! table, and for the same stated reason: *"one merged list would resolve `w`
//! and `wq` by whichever was listed first."*
//!
//! # And the prompt has to answer for them
//!
//! That leaves a hole §6 forbids, and it was not hypothetical. Before this
//! existed, `wait for the mortar` typed at the prompt **opened the editor on a
//! new empty `mortar.spell`**: `for` and `the` are filler, `wait` is a
//! `Meditate` synonym whose `Count` slot `mortar` cannot fill, so the reading
//! lost to `scribe <Name>`, which takes free text. `repeat 3` resolved to
//! `undo`.
//!
//! A player learns a word in the editor, types it at the prompt, and destroys
//! something. So the prompt knows these words by name and says what they are —
//! [`Resolution::InSpell`](super::Resolution::InSpell), which is
//! [`Elsewhere`](super::Resolution::Elsewhere) one step further: *a real word,
//! in the wrong place, answered honestly* rather than guessed at.

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
    /// **Last of the four, and deliberately.** It is the only one needing a
    /// vocabulary for *state* rather than for what just happened, and the
    /// Autonauts precedent is that a wait gives a non-programmer conditional
    /// behaviour without any of that. A player reaches `if` having already
    /// solved most of what they wanted with `wait`.
    If,
    /// The other half of an `if`.
    Else,
    /// Close a block, whatever opened it.
    End,
    /// Bound a `repeat` by a question instead of a number.
    ///
    /// **Never the first word of a line.** It is `repeat`'s argument —
    /// `repeat until the stacks is idle` — and a line beginning with it is a
    /// real word in the wrong place, which is exactly what [`SpellWord`] exists
    /// to answer honestly rather than let the fuzzy matcher guess at.
    ///
    /// The sixth control word, and the count is defended rather than spent
    /// (`verb.rs` makes the same argument about verbs). It earns its place by
    /// *deleting* something: every loop that wanted "until" had to guess a
    /// bound instead, and the shipped solver still says `repeat 20000` because
    /// the language could not say what it meant.
    Until,
    /// Give a name to a place, so a later line can say it — `let best be north`.
    ///
    /// # A variable holds a **name**, and that is the ceiling
    ///
    /// Not a number, not a list, not an expression: it binds one word to
    /// another, and everywhere a name may stand the bound word stands for it.
    /// That is what *"follow the way with the fewest marks"* needs — an
    /// accumulator you compare against and then act on — and it is the smallest
    /// thing that gives it.
    ///
    /// The value is resolved against the room **at cast**, like every other name
    /// in a spell (§8), so `let m be mortar` binds `mortar_and_pestle` and a
    /// word the room cannot place is a fault rather than a variable holding a
    /// typo.
    ///
    /// # `let … be`, and it was `set … to` for an afternoon
    ///
    /// **`set` is already a `dial` synonym** (`vocabulary.rs`, shell register),
    /// and a spell word is matched *before* the fuzzy matcher — so `set second
    /// borax` at the prompt stopped reaching the lens and answered *"that is a
    /// spell word"* instead. This module's own opening paragraph names three
    /// collisions it refused to add; this would have been a fourth, and a
    /// shipped verb's would at that.
    ///
    /// `be` rather than `to` for a second reason of the same kind: `to` is on
    /// §6's filler list, so a keyword `to` is invisible to every reader in the
    /// parser except the one that gets the text before normalisation.
    Let,
    /// Do the block once for each of something — `for each way`.
    ///
    /// **The cursor is named after the group**, so `for each way` binds `way`
    /// and the body says `if way has spoil`. No second syntax, and nothing to
    /// learn beyond the group's own word.
    ///
    /// `it` was the obvious cursor and cannot be used: `it` is on §6's filler
    /// list, so `follow it` is stripped to `follow` before anything sees it.
    For,
    /// Name a run of lines so the rest of the spell can say it —
    /// `part gathering()`.
    ///
    /// # Why `part`, measured rather than chosen
    ///
    /// §10 calls this *"composition — build spells from components"* and the
    /// design calls the component a **part**; a different word here would give
    /// one phase two names for one idea.
    ///
    /// **A part lives in the spell that uses it**, and that is settled: a spell
    /// is contained to a single `.spell` file (§19), so this word names the
    /// whole of composition rather than the near half of it.
    ///
    /// It is also the word that survived a sweep. Every candidate was scored
    /// against every word the game knows, and the obvious ones are all inside
    /// `MIN_SIMILARITY`'s typo band — `rite` scores **800** against `write`,
    /// `call` **750** against `wall`, `step` **750** against `stop`, `make`
    /// **750** against `take`, and `form` **750** against `for`, which is a
    /// control word already. `part` scores 500, its nearest neighbours being
    /// `cast` and `east`, and it is neither filler nor anything the vocabulary
    /// already spends.
    ///
    /// `to` and `set` were refused before the sweep ran: `to` is filler, and
    /// `set` is a live `dial` synonym — [`Let`](Self::Let) records what taking
    /// that one cost for an afternoon.
    ///
    /// # A call is punctuation, not a ninth word
    ///
    /// `gathering()` is the call, and the parentheses are the whole notation.
    /// The alternative was a bare name, which needs a rule — *a part may not be
    /// called something the tower already has a word for* — to stop `grind`
    /// meaning two things; and a second keyword would have cost a word in a
    /// language that argues its count one entry at a time. Punctuation costs
    /// neither, and it is the one place symbols are **canonical** rather than
    /// merely accepted: §19's comparison spellings write back as words because a
    /// word exists to write back to, and here none does.
    Part,
    /// Do nothing for a number of ticks.
    ///
    /// **The tenth word, and the menagerie is what earned it.** Nothing in the
    /// language counted ticks: [`Wait`](Self::Wait) takes a *thing* and blocks by
    /// scanning the record stream, which is the right shape for *"hold until the
    /// mortar is free"* and the wrong one for a chant. A blocking wait would let
    /// the world decide when to sing, and deciding when is the entire puzzle
    /// (§19) — so the delay has to be a number the spell works out and writes
    /// down.
    ///
    /// # `rest` was the first name, and it is a `meditate` synonym
    ///
    /// A spell word is matched **before** the fuzzy matcher, so claiming `rest`
    /// would have stopped `rest 20` reaching `meditate` at the prompt — the
    /// collision §19 records `set` causing against `dial`, which is why that
    /// entry says to sweep a proposed word against the *whole* vocabulary rather
    /// than against the other spell words.
    ///
    /// Four alternatives were then asserted clean without being swept and two of
    /// them were not: `tarry` scores **800** against `carry`, a live `move`
    /// synonym, and `pause` **667** against `peruse`. `bide` is clean, is one
    /// syllable, and sits in the register of `muster` and `kindle`.
    Bide,
    /// Take the oldest name out of a satchel and bind it —
    /// `pull note from satchel`.
    ///
    /// # Why this is a control word and `queue` is a verb
    ///
    /// **Because it binds a name, and only the language can do that.**
    /// [`Let`](Self::Let) is the other one, and the two are the whole of what
    /// puts a word in `vars`. A verb runs through `execute::dispatch`, which
    /// hands back records and touches nothing a spell is holding — so a `pull`
    /// verb could empty the satchel and would have nowhere to put what it took.
    ///
    /// The asymmetry is worth stating plainly rather than apologising for: the
    /// push half changes the world, which is a verb's job and lets a player load
    /// a satchel by hand; the pull half changes the *spell*, which is a control
    /// word's.
    ///
    /// # It yields while empty, and never reaches `PATIENCE`
    ///
    /// A consumer that has caught up with its producer is the ordinary state of
    /// a working pipeline, not a fault. `wait` gives up after
    /// [`PATIENCE`](crate::tower::spell::PATIENCE) ticks and latches `‼` on the
    /// rail; this takes `bide`'s road instead — *"a spell waiting for ever is a
    /// fault; a spell counting to three is doing what it was written to do"* —
    /// and says nothing on the transcript. What bounds it is the work running
    /// out: a chant ends, the loop's guard goes true, and the spell finishes.
    ///
    /// # The word
    ///
    /// Swept on both axes against every verb, synonym, control word and reading.
    /// `pull` is clean on similarity and `pul` is a free prefix.
    ///
    /// **`draw` was the first choice and is wrong in the prose**, not in the
    /// parser: the game already spends that word on producing a *new* random
    /// thing three times over — `Chant::draw`, `muster` *"draws a course"*,
    /// `summon` *"draw a fresh figure"*. A queue hands back something that was
    /// already there, which is the opposite. `take` is `weave`'s, `lift` scores
    /// 750 against both `sift` and `list`, and `pop` is clean and reads as
    /// jargon in a game whose other words are `muster` and `kindle`.
    Pull,
    /// Set a part running as a second cursor and carry on —
    /// `alongside gathering()`.
    ///
    /// # What it is, against what already existed
    ///
    /// **Two spells already run at once.** `invoke` from inside a spell inserts
    /// a second `Running` and the caller does not block; `run::advance` steps
    /// every one of them each tick with its own budget. So this does not add
    /// concurrency — it adds concurrency *within one file*, which is the
    /// difference between a pipeline you can read on one screen and a pipeline
    /// split across two.
    ///
    /// Stating that plainly is deliberate. It would be easy to sell this as new
    /// capability and it is not: what it sells is that the producer and the
    /// consumer of a satchel sit next to each other, share a name, and are
    /// edited together.
    ///
    /// # A fork is not a call
    ///
    /// `gathering()` suspends the caller until the part returns;
    /// `alongside gathering()` leaves the caller where it is and starts a
    /// **[`Strand`](crate::tower::spell::Strand)** on the part. The forked
    /// cursor has its own position, its own bindings and its own budget, and
    /// when it runs off the end it simply stops — nothing is waiting for it.
    /// The spell ends when every cursor has.
    ///
    /// Arguments work exactly as a call's, and for the same reason: a part's
    /// brackets are the whole of what it can see (§19), so a fork that could
    /// read the caller's store would be the shared-variables shape that decision
    /// removed.
    ///
    /// # The word
    ///
    /// Swept on both axes. `alongside` is clean on similarity and `alo` is a
    /// free prefix; `meanwhile` is equally clean and was the other candidate.
    /// `fork` scores **962** against `for`, which is a control word already, and
    /// `split` 600 against `spoil`; `beside` scores 667 against `bide`, which is
    /// the word most likely to sit on the line above it.
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
    /// The same table `recall`'s overview prints, and deliberately the same
    /// words: two answers to *what does `for` take* is one of them being wrong
    /// later. It lived in `orbs-shell` for one version, which put a painter in
    /// the business of knowing the grammar — the shape §19 records going wrong
    /// three times over.
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
            // **`(...)` rather than `()` or `(<name>, <name>)`.** A part takes
            // any number of names including none, and a shape that showed two
            // would read as a rule; one that showed none is what this said
            // before parameters existed, and it taught the wrong form.
            Self::Part => "<name>(...)",
            // `<count>`, the same word `repeat` uses, because it is the same
            // thing being counted from the player's side: a number of times
            // round against a number of ticks through.
            Self::Bide => "<count>",
            // **`from` is on §6's filler list and is written anyway**, exactly
            // as `let`'s `be` is not. The difference: `be` is load-bearing —
            // `wait for the mortar to be idle` is the trap it exists to stop —
            // where this is decoration a reader wants and the parser never sees.
            // `pull note satchel` is the same line.
            Self::Pull => "<name> from <place>",
            // **`<name>(...)`, the same shape `part` shows**, because what
            // follows is a call and the brackets are the notation rather than a
            // slot. A shape reading `<part>` would teach a bare name, which is
            // the one thing a call may not be.
            Self::Alongside => "<name>(...)",
            Self::Else | Self::End => "",
        }
    }

    /// Whether this word opens a block that an `end` must close.
    ///
    /// **`Until` does not**, even though it is always inside one: the block is
    /// `repeat`'s, and counting it here would want a second `end` for a loop
    /// with a guard on it. **`Let` does not either** — it is one line that binds
    /// one name, and nothing follows it that an `end` would close.
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
/// **Exact on the first word, case-folded.** Not fuzzy: `waat` is a typo the
/// orb should say it cannot read rather than guess at, because guessing wrong
/// here rewrites the player's file — canonicalisation runs at save.
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
/// **after** it starts at.
///
/// # One rule, two callers
///
/// The orb re-indents a spell when it writes it down, and the editor indents as
/// you type. Those must agree — a buffer that indents one way and a saved file
/// that indents another makes every save look like it moved your work. So the
/// rule is here, in the crate that owns the spell language, and both fold it.
///
/// `end` and `else` sit level with the `if` or `repeat` they belong to rather
/// than with the body between them: an `else` indented as body reads as a step
/// *inside* the branch it ends, which is the opposite of what it does.
///
/// A blank line or a comment leaves the depth alone. That is what lets the
/// editor put the caret at the body's indent on an empty line inside a block.
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
        // **Exact, deliberately.** Fuzzy-matching here would let a typo rewrite
        // the player's file, because canonicalisation runs at save — and a
        // `waat` silently becoming `wait` is a line they never wrote.
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
