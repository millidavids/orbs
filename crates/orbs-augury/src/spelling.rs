//! The spell register: what a `.spell` line means, and how one is put back
//! together.
//!
//! # The same question over a different set of answers
//!
//! The prompt asks *which verb, and which words are its argument*. A spell line
//! asks *which statement, and which words are its argument* — and everything
//! between those two sentences is identical, so this shares the encoder, the
//! vocabulary, the batching and the trainer with [`crate::model`]. What differs
//! is one head's width and one corpus.
//!
//! # A class is a **template**, not a spell word
//!
//! At the prompt a canonical command names its own class: the head of `grind
//! sage` is `grind` and `Verb::ALL` has a row for it. A spell statement does not.
//! `if {place} is idle`, `if {place} is empty` and `if {place} has {reagent}` are
//! one word between them, and the first two carry the same number of slots — so
//! neither the word nor the tagging says which shape to build.
//!
//! So the classes are the entries of `content/spellings.toml`, in file order,
//! plus one for *this line is a command*. [`assemble`] then has a shape to fill
//! rather than a string to reverse-engineer.
//!
//! # Propose, then check twice
//!
//! A reader's answer never replaces a line on its own. The assembler builds a
//! candidate and then asks two questions, and **both** have to hold:
//!
//! - does it stand on its own as a statement — `spell::reads_cleanly`;
//! - does it account for **every word the player wrote** that carries meaning.
//!
//! The second is the one that matters. `if the alembic has finished` parses
//! perfectly and means *"holds a thing called finished"*, so a re-parse alone
//! would wave through the exact class this feature exists to fix. §19 records
//! four rewriter defects of precisely that kind.

use std::sync::OnceLock;

use orbs_sim::content::Phrasings;
use orbs_sim::parser::{NounKind, SYNONYMS, SpellWord, Verb};

/// Every canonical shape the spell head chooses between, in file order.
///
/// Read once from `content/spellings.toml` — the same file the corpus is built
/// from, so a template added there becomes a class without anything here
/// changing. That also means **adding an entry invalidates trained weights**,
/// exactly as adding a word invalidates the vocabulary, and
/// [`Scribe::load`](crate::Scribe::load) checks the head's width for that reason.
pub fn shapes() -> &'static [String] {
    static SHAPES: OnceLock<Vec<String>> = OnceLock::new();
    SHAPES.get_or_init(|| {
        Phrasings::spellings()
            .entries()
            .iter()
            .map(|entry| entry.canonical.clone())
            .collect()
    })
}

/// How many statements the spell head chooses between.
///
/// Every shape plus one: **a line that is a command**, which the prompt's own
/// reader owns. A spell body is mostly commands, so a head that could not say so
/// would have to call them all `wait`.
#[must_use]
pub fn kinds() -> usize {
    shapes().len() + 1
}

/// The class meaning *this line is a command, not control flow*.
#[must_use]
pub fn command() -> usize {
    shapes().len()
}

/// The shape a class index names, or [`None`] for [`command`].
#[must_use]
pub fn shape_of(class: usize) -> Option<&'static str> {
    shapes().get(class).map(String::as_str)
}

/// The sets a spell can walk, as `spellings.toml` teaches them.
///
/// What a `{group}` slot is filled against. A closed list, so a span the tagger
/// marks is a set only if it is one of these — or one with the `s` English puts
/// on it, which is how *"work through the ways"* says `way`.
pub fn groups() -> &'static [String] {
    static GROUPS: OnceLock<Vec<String>> = OnceLock::new();
    GROUPS.get_or_init(|| Phrasings::spellings().groups().to_vec())
}

/// Which tag slot a noun of `kind` is written into, in the spell register.
///
/// # By kind, not by position — and the prompt keeps position
///
/// The prompt numbers a slot by where its argument sits in the canonical,
/// because `move {reagent} {place}` has to know which of two spans is the thing
/// and which the destination. That was this register's rule too, and it made the
/// tagger's job contradict itself: a place was slot 0 in `if {place} is idle`
/// and slot 1 in `let {name} be {place}` — and in every `move` the command class
/// is taught from. The tagger runs before the class is decided and without it,
/// so it put both words of *"nook from now on is the alembic"* in slot 0, and
/// `let` read 25% of its holdout while `pull`, whose one slot is never a place,
/// read 71%.
///
/// No spell shape carries two slots of one kind — a test holds that — so the
/// kind alone can say which: a place is always 0, a reagent always 1, and a word
/// of the player's own, a name or a set, always 2. The tagger learns one thing
/// about a place, everywhere it meets one.
#[must_use]
pub const fn slot_for(kind: NounKind) -> usize {
    match kind {
        NounKind::Place => 0,
        NounKind::Reagent => 1,
        _ => 2,
    }
}

/// The slot a shape's `{marker}` is read from: [`slot_for`] on its kind.
fn slot_of_marker(marker: &str) -> Option<usize> {
    let kind = match marker {
        "{place}" => NounKind::Place,
        "{reagent}" => NounKind::Reagent,
        "{name}" | "{group}" => NounKind::Name,
        _ => return None,
    };
    Some(slot_for(kind))
}

/// A shape's arguments in the order it names them, taken from the tagger's
/// slots by kind — [`None`] when the tagger left one of them empty.
///
/// `placed` is indexed by slot, empties included; see [`slot_for`].
#[must_use]
pub fn ordered(class: usize, placed: &[Option<String>]) -> Option<Vec<String>> {
    shape_of(class)?
        .split_whitespace()
        .filter(|token| token.starts_with('{'))
        .map(|marker| placed.get(slot_of_marker(marker)?)?.clone())
        .collect()
}

/// The numbers a spell can be told, written out.
///
/// **A count must be *in* the line.** `bide 10` is only ever offered for a line
/// that says ten — as a digit or as one of these — because inventing a number
/// for *"hang about a while"* would be **generation**, and nothing in this
/// feature generates. A line with no number in it is left exactly as written.
///
/// # `one` is not here, and that is deliberate
///
/// It is the number English spends least on counting: *"one at a time"*, *"one
/// by one"*, *"for every one of the ways"*, *"each one"*. Two of the four
/// phrasings `for each` is measured on use it that way, and with `one` in this
/// table [`accounts_for`] demanded a `1` in the reading — so the right answer,
/// `for each way`, could never pass, and `bide 1` could and did. A count of one
/// is `bide 1` or `repeat 1`, which a player writes as a digit or not at all.
///
/// A const table rather than prose, for `Verb::canonical`'s reason: it is
/// parser vocabulary, not writing.
const NUMBERS: [(&str, u32); 20] = [
    ("zero", 0),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
    ("ten", 10),
    ("eleven", 11),
    ("twelve", 12),
    ("dozen", 12),
    ("fifteen", 15),
    ("twenty", 20),
    ("thirty", 30),
    ("forty", 40),
    ("fifty", 50),
    ("hundred", 100),
    ("score", 20),
];

/// The number `line` names, if it names one.
#[must_use]
pub fn count_in(line: &str) -> Option<u32> {
    line.split_whitespace().find_map(|word| {
        let word = word.trim_matches(|ch: char| !ch.is_alphanumeric());
        word.parse::<u32>().ok().or_else(|| {
            let folded = word.to_lowercase();
            NUMBERS
                .iter()
                .find(|(name, _)| *name == folded)
                .map(|(_, value)| *value)
        })
    })
}

/// Words a canonical statement is built out of, which the player need not have
/// written.
///
/// **The other half of *a count must be in the line*.** A shape's literal words
/// are grammar the orb supplies — `is`, `has`, `be` — and grammar may be
/// invented, because *"once the mortar falls quiet"* contains neither `is` nor
/// `idle` and is still exactly `if mortar_and_pestle is idle`. A literal that is
/// neither grammar nor a spell word has to appear in the line, or the shape is
/// not offered.
///
/// # Every name is a slot now
///
/// Three shapes used to carry a name as a literal — `let tool be {place}`, `pull
/// note from {place}`, `for each way` — and this rule was what stopped *"name
/// the alembic hammer"* coming back as `let tool be alembic`. The price was that
/// `tool`, `note` and `way` were the only names those shapes could ever offer.
/// The name is a `{name}` slot and the set a `{group}` slot now, filled from the
/// player's own words, and the rule is left guarding the next template that
/// writes a name down.
///
/// # `satchel` is not here, and was for one measurement
///
/// Every satchel in the tower is called [`SATCHEL`](orbs_sim::tower::SATCHEL),
/// so supplying the word invents no name — but it also removed the only evidence
/// that a line was about a satchel at all. *"somebody left the clarity out"*
/// came back `pull clarity from satchel`. The word has to be in the line, as it
/// is in every phrasing of `pull` the corpus has.
const GRAMMAR: [&str; 11] = [
    "is", "be", "has", "from", "each", "idle", "empty", "not", "and", "or", "to",
];

/// Put a reading together: the shape `class` names, with the player's own words
/// in its slots.
///
/// `slots` is what the tagger found, in slot order. Returns [`None`] — meaning
/// *leave the line exactly as it is* — whenever the shape cannot be filled
/// honestly:
///
/// - fewer spans than the shape has slots;
/// - a shape with a count in it and no number anywhere in the line;
/// - a shape naming something the player did not (see the grammar list);
/// - a shape with nothing to fill — `else`, `end` — that the line never says
///   (see `said_by`);
/// - a candidate that does not stand on its own as a statement, or that drops a
///   word carrying meaning (see [`accounts_for`]).
///
/// # It never returns the line back
///
/// A reading identical to what was typed is [`None`], not `Some`. The caller
/// stores readings that *differ* — `Reading::was` is what makes a misreading
/// visible — and a reading that changed nothing would claim one had happened.
#[must_use]
pub fn assemble(class: usize, line: &str, slots: &[String]) -> Option<String> {
    let shape = shape_of(class)?;
    // A shape with nothing to fill has to be *said* — see `said_by`.
    if !fills_from_the_line(class) && !said_by(shape, line) {
        return None;
    }
    let mut spans = slots.iter();
    let mut built: Vec<String> = Vec::new();

    for token in shape.split_whitespace() {
        if token == "{group}" {
            built.push(set_named(spans.next()?)?);
        } else if token.starts_with('{') {
            built.push(named(spans.next()?)?);
        } else if token.parse::<u32>().is_ok() {
            built.push(count_in(line)?.to_string());
        } else {
            let folded = token.to_lowercase();
            let supplied = GRAMMAR.contains(&folded.as_str())
                || SpellWord::ALL
                    .iter()
                    .any(|word| word.canonical() == folded.as_str());
            if !supplied && !says(line, &folded) {
                return None;
            }
            built.push(folded);
        }
    }

    let reading = built.join(" ");
    if reading == line.trim() {
        return None;
    }
    (orbs_sim::tower::spell::reads_cleanly(&reading) && accounts_for(line, &reading))
        .then_some(reading)
}

/// What a tagged span actually names, with the filler trimmed off its ends.
///
/// **A span is the tagger's, and the tagger is 92% right.** `when nothing at all
/// is in the alembic` came back with `at alembic` in the slot, which assembles
/// into `if at alembic is empty` — a line that parses, means nothing, and is not
/// what anyone wrote. §6's filler list is the parser's own answer to which words
/// carry no reference, and using it here rather than a second list is the point.
///
/// [`None`] when nothing but filler was found: a slot the tagger filled with
/// `the` is a slot it did not fill.
fn named(span: &str) -> Option<String> {
    // **A comma is not part of a name, and neither is `'s`.** *"call it hammer,
    // meaning the alembic"* tags `hammer,`, and *"until the mortar's free"* tags
    // `mortar's` — binding either would name something no later line could
    // spell. `unfold` is what every other check in this file reads a line
    // through, so a span is read the same way.
    let unfolded = words_of(span);
    let mut words: Vec<&str> = unfolded.iter().map(String::as_str).collect();
    // A word spent on meaning is trimmed with the filler: the tagger ran *"keep
    // at it till the alembic is free"* into `till alembic`, and the name in that
    // is `alembic`. A negation trimmed here is still owed by `accounts_for`.
    let loose = |word: &&str| orbs_sim::parser::is_filler(word) || spent(word);
    while words.first().is_some_and(loose) {
        words.remove(0);
    }
    while words.last().is_some_and(loose) {
        words.pop();
    }
    // **One word, because every name the spell language takes is one.** `let`
    // binds a single word, a place answers to its leaf, a reagent is hyphenated —
    // so a span of two is two things the tagger ran together, not a name. Taking
    // it built `repeat until nook lectern is idle` out of *"let nook refer to the
    // lectern"*, and `if quill away is idle` out of a `pull`.
    //
    // **And never a word the checks count on**, which the trim above is also
    // what does: *"unless the alembic is free"* came back `let unless be
    // alembic` — one word, so a name. See `spent`.
    match words.as_slice() {
        [word] => Some((*word).to_owned()),
        _ => None,
    }
}

/// Whether `word` is spent on meaning — a negation, a join, a comparison, a
/// condition, a count, a spell word or the grammar a shape is built from — and
/// so is never a name.
///
/// Every one is a word [`accounts_for`] relies on surviving *as itself*. Bound
/// as a name it survives as a name, the check that wanted it is satisfied, and
/// what it meant is gone: `unless` became a variable, and the spell stopped
/// asking anything at all.
fn spent(word: &str) -> bool {
    NEGATIONS.contains(&word)
        || JOINS.contains(&word)
        || COMPARISONS.contains(&word)
        || CONDITIONS.contains(&word)
        || GRAMMAR.contains(&word)
        || count_in(word).is_some()
        || SpellWord::ALL.iter().any(|spell| spell.canonical() == word)
}

/// A set's name, from a span that says it in either number.
///
/// **Closed, unlike a name.** `for each` walks a set the room has, so a span is
/// offered only when it *is* one of [`groups`] or one of them with an `s` on —
/// *"work through the ways"* is `for each way`, and `for each ways` would parse
/// and then find no set at cast. A word that is no set at all is not offered:
/// `for each hammer` parses too, and walks nothing.
///
/// **And closed is what lets one be picked out of a longer span**, which a name
/// cannot be. The tagger ran *"for every one of the bands"* into `one bands`;
/// exactly one of those words is a set, so that is the set. Any word at all could
/// be a name, so [`named`] still refuses two.
fn set_named(span: &str) -> Option<String> {
    let sets: Vec<&String> = words_of(span)
        .iter()
        .filter_map(|word| {
            groups()
                .iter()
                .find(|group| *group == word || word.strip_suffix('s') == Some(group.as_str()))
        })
        .collect();
    match sets.as_slice() {
        [set] => Some((*set).clone()),
        _ => None,
    }
}

/// The words of `line` the game has no word for.
///
/// **In a spell those are nearly always names.** A word the vocabulary knows can
/// be spent on choosing a statement or a verb — *"once the mortar falls quiet"*
/// spends `falls` and `quiet` on `idle` — but a word it does not know arrived as
/// a hash bucket and gave the reader nothing to choose with. What is left for it
/// to be is something the player named, which is why a reading that drops one is
/// suspect. See `Scribe::reading` for what that is used to decide.
///
/// Filler and numbers are left out: filler is nothing, and a count is
/// [`accounts_for`]'s business.
#[must_use]
pub fn unknown_words(line: &str, vocabulary: &crate::Vocabulary) -> Vec<String> {
    words_of(line)
        .into_iter()
        .filter(|word| {
            !orbs_sim::parser::is_filler(word)
                && count_in(word).is_none()
                && !vocabulary.token(word).is_known()
        })
        .collect()
}

/// The words of `line` a reading that can hold a name has to keep — see
/// [`holds_a_name`]: ones the game has no word for, that the tagger found as a
/// name, or that it passed over while finding none.
///
/// # A word the game does not know is a name — unless it is a typo
///
/// **Every reading with room for one is held to it, not only a command.** A
/// statement is built from the tagger's spans, so it was taken to be unable to
/// drop a name — and it could, whenever the tagger missed one: *"name the alembic bertha"* came
/// back `if alembic is empty`, *"call the alembic zeph for short"* `repeat until
/// alembic is idle`. Both parse, both run, and neither binds anything.
///
/// A typo arrives unknown too, and a reading that spends one on choosing its
/// shape — *"otherwsie"* as `else`, *"teh"* as nothing — has read it, not
/// dropped it. So a word one slip from a known one may be spent, **unless the
/// tagger marked it**: *"nook"* is a slip of *"look"*, and in *"let nook refer
/// to the lectern"* it is the name the tagger found, not a verb misspelt.
///
/// # Three kinds of unknown word that are never a name
///
/// Each was measured refusing correct readings in the first retrain:
///
/// - **A word spent on meaning** (see `spent`). *"whenever"* is in no template,
///   so it is unknown — and a reading that made it `if` read it.
/// - **A plural of the game's own word.** `areas` reaches the table only as
///   `{group}s`, so it was unknown, and `for each area` was refused for dropping
///   it: `for each` read 27% of its holdout.
/// - **Two letters, untagged.** *"abide ye ten ticks"*: nearly every two-letter
///   word is one slip from another, so `near` cannot judge one, and a two-letter
///   name the player means is one the tagger marks.
///
/// # The tagger's own name decides the words it passed over
///
/// A word the tagger skipped is taken for a name only while its slot for one
/// holds no unknown word. *"nickname the alembic nib"* put `nib` there, so
/// `nickname` is a verb the game lacks and not a second name; *"take the next
/// pip out of the satchel"* put `next` there — a word the game has — so `pip`,
/// which it skipped, is the name it missed. And a span of several words is the
/// tagger running things together (*"iterate over the bands"* came back
/// `iterate bands`), so only a span of one word is a name it found.
#[must_use]
pub fn names_in(
    line: &str,
    placed: &[Option<String>],
    vocabulary: &crate::Vocabulary,
) -> Vec<String> {
    let unknown = |word: &String| {
        !spent(word)
            && !vocabulary.token(word).is_known()
            && !word
                .strip_suffix('s')
                .is_some_and(|single| vocabulary.token(single).is_known())
    };
    let mut names: Vec<String> = placed
        .iter()
        .flatten()
        .filter_map(|span| {
            let words: Vec<String> = words_of(span)
                .into_iter()
                .filter(|word| !orbs_sim::parser::is_filler(word) && !spent(word))
                .collect();
            match words.as_slice() {
                [word] if unknown(word) => Some(word.clone()),
                _ => None,
            }
        })
        .collect();
    let found = placed
        .get(slot_for(NounKind::Name))
        .and_then(Option::as_ref)
        .is_some_and(|span| words_of(span).iter().any(unknown));
    if !found {
        for word in unknown_words(line, vocabulary) {
            if unknown(&word)
                && word.chars().count() > 2
                && !vocabulary.near(&word)
                && !names.contains(&word)
            {
                names.push(word);
            }
        }
    }
    names
}

/// Whether `reading` keeps every one of `words`.
#[must_use]
pub fn keeps(reading: &str, words: &[String]) -> bool {
    let kept = words_of(reading);
    words.iter().all(|word| kept.contains(word))
}

/// Whether a shape is built from something the line said — a span or a count.
///
/// `else` and `end` are not: they have no slot and no number, so they assemble
/// out of any line at all. `Scribe::reading` offers such a shape only as the
/// head's first choice, and never by walking down the ranking to it.
#[must_use]
pub fn fills_from_the_line(class: usize) -> bool {
    shape_of(class).is_some_and(|shape| {
        shape
            .split_whitespace()
            .any(|token| token.starts_with('{') || token.parse::<u32>().is_ok())
    })
}

/// Whether a reading of `class` holds a word of the player's own — a command's
/// argument, or a shape's `{name}` or `{group}` — and so has a name to drop.
///
/// **A shape with no room for a name is not asked to keep one**, and cannot
/// pass for a reading of a line that names something either, because it has to
/// be *said*: an `if` needs a line that asks, a `repeat until` one that loops, a
/// count its number and unit, `else` and `end` their own words. Holding every
/// reading to the names cost correct ones — *"once the alembic stops bubbling"*
/// has an unknown word and no name in it at all.
#[must_use]
pub fn holds_a_name(class: usize) -> bool {
    class == command()
        || shape_of(class)
            .is_some_and(|shape| shape.contains("{name}") || shape.contains("{group}"))
}

/// Whether `line` names the verb `reading` opens with — its canonical word, or
/// every word of one of its synonyms.
///
/// Asked of a **bare** command, one with no argument. Nothing from the line went
/// into it, so the only evidence it is a reading of this line at all is that the
/// line says the verb.
#[must_use]
pub fn names_its_verb(line: &str, reading: &str) -> bool {
    let said = words_of(line);
    let Some(head) = reading.split_whitespace().next() else {
        return false;
    };
    let Some(verb) = Verb::ALL.iter().find(|verb| verb.canonical() == head) else {
        return false;
    };
    said.iter().any(|word| word.as_str() == head)
        || SYNONYMS
            .iter()
            .filter(|synonym| synonym.verb == *verb)
            .any(|synonym| {
                synonym
                    .words
                    .iter()
                    .all(|word| said.iter().any(|seen| seen.as_str() == *word))
            })
}

/// Whether `line` contains `word`, ignoring case and punctuation.
fn says(line: &str, word: &str) -> bool {
    words_of(line).iter().any(|said| said == word)
}

/// A line as lower-case words, punctuation trimmed and contractions unfolded.
fn words_of(text: &str) -> Vec<String> {
    text.split_whitespace().flat_map(unfold).collect()
}

/// One written word as the words it stands for.
///
/// **A contraction is two words, and one of them can be `not`.** *"isn't"*
/// matched nothing in the negation rule, so *"when the alembic isn't free"*
/// dropped its negation silently. A word ending *n't* unfolds to its stem and
/// `not`; `'s`, `'re`, `'ll`, `'ve`, `'d` and `'m` are dropped, because none of
/// them is anything a spell can say — and dropping `'s` is also what turns the
/// span *mortar's* back into the name *mortar*. `_` and `-` are kept inside a
/// word: `mortar_and_pestle` and `rock-salt` are one name each.
fn unfold(word: &str) -> Vec<String> {
    // **The reader's fold, with one difference.** A word ending *n't* is two
    // words here and one row there: a check counts words, so *"isn't"* has to
    // leave both `is` and `not` behind, while the reader cannot split a word
    // without moving every span after it. Everything else is `fold_word`
    // exactly, so the two cannot disagree about what a word is.
    let lowered = word.replace('\u{2019}', "'").to_lowercase();
    let trimmed = lowered
        .trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '\'')
        .trim_start_matches('\'');
    if let Some(stem) = trimmed.strip_suffix("n't") {
        return [stem, "not"]
            .into_iter()
            .filter(|part| !part.is_empty())
            .map(str::to_owned)
            .collect();
    }
    let folded = crate::vocabulary::fold_word(word);
    if folded.is_empty() {
        Vec::new()
    } else {
        vec![folded]
    }
}

/// Whether `reading` accounts for every word of `line` that carries meaning.
///
/// **The rule a re-parse cannot express**, and the one that stops a plausible
/// misreading. A dropped `or` clause, a dropped `not`, a commented-out line read
/// as a command — all three parse perfectly and mean something else. §19 records
/// four rewriter defects of exactly that shape, each *"fixed by another special
/// case in the rewriter"* until the rewriter was deleted.
///
/// # Two classes of word, not every word
///
/// Demanding that *every* word survive is wrong and was tried: `wait ten ticks`
/// becomes `bide 10`, and neither `wait` nor `ticks` appears in it — a statement
/// word is spent on choosing the statement and a unit is spent on nothing. A
/// rule that strict refuses every correct reading.
///
/// What it checks instead is the two classes whose loss **silently changes the
/// meaning**:
///
/// - **numbers** — `ten` must reach the reading as `10`, and must not reach it
///   as `30`. Checked on every reading.
/// - **joins** — `and`, `or`, `but`, `except`. Two things where the reading has
///   room for one: a dropped `or` halves a condition, and *"grind the sage and
///   then wait ten ticks"* read as `bide 10` runs the wait with the grind gone.
/// - **comparisons** — `until`, `fewer`, `more`, on a condition.
/// - **negations** — `not`, `no`, `never`, `unless`, and every *n't*. A dropped
///   one inverts the spell and still parses.
/// - **conditions** — `if`, `when`, `whenever` and the rest, on a reading that
///   asks nothing: a command has nowhere to put *when*, so it runs regardless.
/// - **units** — a count in ticks is a wait and a count in times a repeat, so
///   neither may become the other.
///
/// # Where each is excused, and why each excuse is narrow
///
/// Every excuse here was earned by a correct reading the flat rule refused, and
/// the first version of one was earned by a wrong reading it let through.
///
/// - **`else` and `end` cannot lose a clause**, because they carry nothing:
///   *"or else"* and *"and that is that"* spend their connective on nothing.
///   `bide 10` and `repeat 3` carry a count and are *not* excused — a join into
///   one of those is a second thing dropped.
/// - **A command is not held to the joins.** *"take the husks out and throw them
///   away"* is `discard husks`, one action in two verbs, and the command language
///   has no `and` for the word to have meant. It *is* held to the negations:
///   it has no `not` either, so *"never grind the sage"* is not a command.
/// - **`else` spends a negation**, because it is one: *"and if it isn't"*.
/// - **A negation of an opposite.** *"not busy"* **is** `idle`. The first version
///   excused any negation whose next word the reading lacked, and let *"not
///   free"* through as `idle` — a synonym, not an antonym. Now only a word listed
///   as the opposite of the state the reading asserts excuses one; see
///   `OPPOSITES`.
///
/// A *span* is never dropped, because the assembler builds the line from the
/// spans the tagger marked and the grammar list stops one being invented. A
/// shape further down the ranking can have no slot for a noun the line says,
/// though, so a thing the tower has — a substance or a place — and a verb's
/// own word are checked: see the body.
#[must_use]
pub fn accounts_for(line: &str, reading: &str) -> bool {
    // **`till` and `til` are `until`**, as it is written in a hurry. Counted as
    // themselves they were no comparison at all, so *"keep at it till the
    // alembic is free"* could lose its bound unremarked — and counted as
    // themselves but *listed*, the right reading, `repeat until`, would lose.
    let said: Vec<String> = words_of(line)
        .into_iter()
        .map(|word| match word.as_str() {
            "till" | "til" => "until".to_owned(),
            _ => word,
        })
        .collect();
    let kept = words_of(reading);
    let counted = |words: &[String], wanted: &str| -> usize {
        words.iter().filter(|word| word.as_str() == wanted).count()
    };
    // Each at least as many times as it was written: `a or b or c` losing one
    // `or` is the same defect as losing them all.
    let dropped = |set: &[&str]| {
        set.iter()
            .any(|word| counted(&kept, word) < counted(&said, word))
    };

    let statement = orbs_sim::parser::spell_word(reading);
    // `else` and `end` carry nothing but their own word, and their phrasings
    // spend a connective on nothing at all — *"or else"*, *"and that is that"*.
    let bare = statement.is_some() && kept.len() == 1;

    // **A join, on anything that carries something.** Not only a condition:
    // *"grind the sage and then wait ten ticks"* became `bide 10`, a count with
    // no condition in it, and ran the wait with the grind gone. A command is the
    // exception, because *"take the husks out and throw them away"* is one
    // action phrased with two verbs and the command language has no `and`.
    if statement.is_some() && !bare && dropped(&JOINS) {
        return false;
    }
    if has_a_condition(reading, &kept) && dropped(&COMPARISONS) {
        return false;
    }
    // **A negation, everywhere but `else`** — which *is* a negation, so
    // *"and if it isn't"* spends its `not` on choosing the shape.
    if statement != Some(SpellWord::Else) && !negations_kept(&said, &kept, statement.is_some()) {
        return false;
    }
    // **A condition, on anything that does not ask one** — except `else`, which
    // spends one on choosing its shape as it spends a negation: *"if that
    // fails"*. A command, a count or a binding has nowhere to put *when*, so a
    // reading of that shape runs regardless: *"whenever the alembic has sage"*
    // came back `distil sage`, and *"after the alembic has finished its work"*
    // `wield alembic`. See `conditional` for where each word counts.
    if !asks(reading) && statement != Some(SpellWord::Else) && conditional(line) {
        return false;
    }
    // **A loop is said, never supplied.** `until` is a spell word, so the
    // assembler may write it whether or not the line has it — and *"after the
    // alembic has finished its work"* came back `repeat until alembic is idle`,
    // a loop where the line asked once. Every phrasing the corpus teaches for it
    // says `until` or `while`, and *"as long as"* is `while` in three words.
    let inverts = said.iter().any(|word| word == "while" || word == "whilst")
        || said
            .windows(2)
            .any(|pair| pair[0] == "as" && pair[1] == "long");
    let loops = inverts || said.iter().any(|word| word == "until");
    if asks(reading) && statement == Some(SpellWord::Repeat) && !loops {
        return false;
    }
    // **A condition is said, never supplied**, for the same reason: `if` is a
    // spell word the assembler may write whatever the line says. Lines that
    // name something came back as conditions — *"name the alembic bertha"* as
    // `if alembic is empty`, *"pull out ozzy from the satchel"* as `if ozzy is
    // empty` — and every one of them asks nothing.
    if statement == Some(SpellWord::If) && !conditional(line) {
        return false;
    }
    // **A statement leaves no command behind.** *"when the alembic is idle,
    // grind the sage"* is two statements, and `if alembic is idle` is the first
    // with the grind gone. A verb's own word that the statement is never taught
    // with is a second thing where the reading has room for one — while
    // *"hold for ten ticks"* spends `hold` on `bide`, which is how it is taught.
    // `else` and `end` too: *"never grind the sage"* is no `else`.
    if let Some(opens) = statement
        && said.iter().enumerate().any(|(at, word)| {
            Verb::ALL
                .iter()
                .any(|verb| verb.canonical() == word.as_str())
                && used_as_a_verb(&said, at)
                && !kept.contains(word)
                && !taught_with(opens).contains(word)
        })
    {
        return false;
    }
    // **A statement drops nothing the tower has** — no substance the content
    // names, no place the tower raises. A shape further down the ranking can
    // have no slot for one: *"when the alembic has a bit of sage"* came back `if
    // alembic is idle` — the tagger missed the sage, `if {place} has {reagent}`
    // could not be built, and the next shape could. A shape with no slot at all
    // has room for none: `stop alembic`, a canonical command, came back `end`.
    if statement.is_some()
        && said
            .iter()
            .any(|word| things().contains(word) && !kept.contains(word))
    {
        return false;
    }
    // **An opposite said outright is the reading inverted.** *"loop while the
    // alembic is busy"* came back `if alembic is idle`, with no negation to drop
    // and so nothing above to notice. A loop's `while` is the exception, since
    // it inverts: *"keep at it while the alembic is busy"* is `repeat until
    // alembic is idle`, and taught so.
    if statement.is_some()
        && !(statement == Some(SpellWord::Repeat) && inverts)
        && contradicted(&said, &kept)
    {
        return false;
    }
    // **A count keeps its unit.** *"hang on 5 ticks"* came back `repeat 5`: the
    // number reached the reading and the ticks it was counting did not.
    let counts_in = |units: &[&str]| said.iter().any(|word| units.contains(&word.as_str()));
    match statement {
        Some(SpellWord::Repeat) if !asks(reading) && counts_in(&TICKS) => return false,
        Some(SpellWord::Bide) if counts_in(&TIMES) => return false,
        _ => {}
    }

    // ...and every number the player named, as the digit it becomes.
    said.iter()
        .filter_map(|word| count_in(word))
        .all(|count| kept.iter().any(|held| *held == count.to_string()))
}

/// Words that mean the opposite of a state a condition asks about.
///
/// **What makes *"not busy"* `idle` and *"not free"* not.** The first version of
/// the negation rule excused any negation whose next word was missing from the
/// reading, on the argument that the reading must have answered it with an
/// antonym. It was answering with a *synonym* just as often: *"when the alembic
/// isn't free"* became `if alembic is idle` and the spell ran on exactly the
/// ticks it was written to skip. A negation is excused only for a word listed
/// here as the opposite of the state the reading asserts.
///
/// A const table rather than prose, for `NUMBERS`' reason: it is parser
/// vocabulary, not writing.
const OPPOSITES: [(&str, &[&str]); 2] = [
    (
        "idle",
        &[
            "busy", "working", "running", "occupied", "use", "work", "going",
        ],
    ),
    (
        "empty",
        &[
            "full",
            "loaded",
            "stocked",
            "thing",
            "anything",
            "something",
        ],
    ),
];

/// Whether every negation `said` holds survives into `kept`, or negated an
/// opposite of the state `kept` asserts.
///
/// `statement` is whether `kept` is a spell statement. A command has no way to
/// say `not` at all, so it is never excused: *"never grind the sage"* read as
/// `grind sage` is the whole spell inverted.
fn negations_kept(said: &[String], kept: &[String], statement: bool) -> bool {
    let opposites: Vec<&str> = if statement {
        OPPOSITES
            .iter()
            .filter(|(state, _)| kept.iter().any(|word| word == state))
            .flat_map(|(_, words)| words.iter().copied())
            .collect()
    } else {
        Vec::new()
    };
    // Whether the negation at `at` negated an opposite, which the reading
    // answers with its state rather than by keeping the `not`.
    let excused = |at: usize| {
        said.iter()
            .skip(at + 1)
            .find(|word| !PAST.contains(&word.as_str()))
            .is_some_and(|next| opposites.contains(&next.as_str()))
    };

    NEGATIONS.iter().all(|negation| {
        // `never` and `unless` are never excused: neither is ever the other half
        // of an antonym.
        let owed = said
            .iter()
            .enumerate()
            .filter(|(at, word)| {
                word.as_str() == *negation && !(matches!(*negation, "not" | "no") && excused(*at))
            })
            .count();
        kept.iter()
            .filter(|word| word.as_str() == *negation)
            .count()
            >= owed
    })
}

/// Words that join two things where a reading has room for one.
const JOINS: [&str; 4] = ["and", "or", "but", "except"];

/// Words that compare, which only a condition can carry. `till` and `til` are
/// read as `until` before these are counted — see [`accounts_for`].
const COMPARISONS: [&str; 3] = ["until", "fewer", "more"];

/// Words that invert what follows them.
const NEGATIONS: [&str; 4] = ["not", "no", "never", "unless"];

/// Words a negation reaches past to what it negates — *no longer* busy, *not
/// in* use, *not at* work, *not a* thing.
const PAST: [&str; 5] = ["longer", "in", "at", "a", "the"];

/// Words that say a state's opposite outright, needing no negation to mean it.
///
/// Narrower than `OPPOSITES`, which also lists what only a negation makes an
/// opposite — *"not in use"*, *"not a thing"* — and which, said bare, are no
/// state at all: *"keep going"* is not busy.
const CONTRARY: [(&str, &[&str]); 2] = [
    ("idle", &["busy", "working", "running", "occupied"]),
    ("empty", &["full", "loaded", "stocked"]),
];

/// Words that make what follows depend on the tower. See [`conditional`] for
/// where each one counts.
const CONDITIONS: [&str; 12] = [
    "if", "when", "whenever", "unless", "until", "till", "til", "while", "whilst", "once", "after",
    "should",
];

/// What a wait is counted in.
const TICKS: [&str; 2] = ["tick", "ticks"];

/// What a repeat is counted in.
const TIMES: [&str; 4] = ["times", "rounds", "passes", "goes"];

/// Words that say nothing a spell could run, beside §6's filler — what *"that
/// is it"* and *"and there we are"* are made of.
const NOTHING_SAID: [&str; 8] = ["is", "are", "so", "there", "here", "we", "now", "then"];

/// Whether `line` makes something depend on the tower.
///
/// **Where each word stands is the whole of this**, because every one of them
/// has an ordinary use that asks nothing. *"if you would"* is manners and *"a
/// while"* a length of time; `once`, `after` and `should` ask only where they
/// open a clause — *"once the alembic is free"*, *"should sage lie within"* —
/// and are an adverb, a preposition and a modal everywhere else: *"sample
/// once"*, *"one socket after another"*, *"the alembic should stop"*, all of
/// them lines the corpus teaches as commands. The rest ask wherever they stand.
fn conditional(line: &str) -> bool {
    let raw: Vec<&str> = line.split_whitespace().collect();
    let words: Vec<String> = raw
        .iter()
        .map(|word| crate::vocabulary::fold_word(word))
        .collect();
    // **A question about the tower asks**, whatever follows it: *"does the
    // alembic have sage? then"* came back `distil sage`. The openers of a yes-or-no
    // question only — *"could you grind the sage"* is a request, and *"have nook
    // be the alembic"* an order, so neither `could` nor `have` is one.
    if words.first().is_some_and(|word| {
        matches!(
            word.as_str(),
            "is" | "are" | "does" | "has" | "was" | "were"
        )
    }) {
        return true;
    }
    words.iter().enumerate().any(|(at, word)| {
        let before = at.checked_sub(1).map(|was| words[was].as_str());
        let after = words.get(at + 1).map(String::as_str);
        // A clause opens at the line's start, after a comma or a colon, or
        // after a word that joins one clause to the next.
        let opens = at == 0
            || raw[at - 1].ends_with([',', ';', ':'])
            || before.is_some_and(|word| JOINS.contains(&word) || word == "then");
        match (word.as_str(), after) {
            // **Openers the corpus teaches as phrases**, where no one word
            // asks: *"the moment the alembic is empty"* came back `empty
            // alembic`, the condition gone and the `empty` kept. `the second`
            // only where it opens, since elsewhere it counts things.
            ("as", Some("soon" | "long")) | ("the", Some("moment" | "instant" | "minute")) => true,
            ("the", Some("second")) => opens,
            ("in", Some("the")) if words.get(at + 2).map(String::as_str) == Some("event") => opens,
            // Openers the corpus teaches that are no condition word elsewhere:
            // *"provided the alembic is idle"*, *"supposing the flask is free"*.
            ("provided" | "providing" | "supposing", _) => opens,
            ("if", _) => after != Some("you"),
            ("while" | "whilst", _) => before != Some("a"),
            ("once" | "after" | "should", _) => opens,
            (other, _) => CONDITIONS.contains(&other),
        }
    })
}

/// Whether `said` names outright the opposite of a state `kept` asserts.
///
/// A loop's `while` inverts what follows it, and the caller excuses that —
/// see [`accounts_for`].
fn contradicted(said: &[String], kept: &[String]) -> bool {
    let against: Vec<&str> = CONTRARY
        .iter()
        .filter(|(state, _)| kept.iter().any(|word| word == state))
        .flat_map(|(_, words)| words.iter().copied())
        .collect();
    said.iter()
        .enumerate()
        .any(|(at, word)| against.contains(&word.as_str()) && !negated(said, at))
}

/// Whether a negation stands before `said[at]`, reached past `PAST`.
///
/// **`nothing` and its kin count here**, and not in `NEGATIONS`: *"nothing
/// running"* is not running, but *"the flask has nothing left"* spends its
/// `nothing` on `empty` — so they excuse a state without being owed by one.
/// So does a word that ends what follows it: *"stopped working"* is idle.
fn negated(said: &[String], at: usize) -> bool {
    said[..at]
        .iter()
        .rev()
        .find(|word| !PAST.contains(&word.as_str()))
        .is_some_and(|word| {
            NEGATIONS.contains(&word.as_str())
                || matches!(
                    word.as_str(),
                    "nothing"
                        | "none"
                        | "nobody"
                        | "nought"
                        | "stopped"
                        | "stops"
                        | "stop"
                        | "finished"
                        | "ceased"
                        | "quit"
                        | "done"
                )
        })
}

/// Whether the verb's word at `said[at]` is used as a verb, not as a noun —
/// *"come to a stop"*, *"get hold of the quill"*.
fn used_as_a_verb(said: &[String], at: usize) -> bool {
    /// Words that make the next one a noun.
    const DETERMINERS: [&str; 9] = [
        "a", "an", "the", "some", "my", "its", "this", "that", "your",
    ];

    let before = at.checked_sub(1).map(|was| said[was].as_str());
    !before.is_some_and(|word| DETERMINERS.contains(&word))
        && said.get(at + 1).map(String::as_str) != Some("of")
}

/// Everything the tower has a name for — every substance the content names,
/// and every place the corpus's tower raises. No shape spends one on choosing
/// itself, so none is a word a statement may drop.
fn things() -> &'static [String] {
    static THINGS: OnceLock<Vec<String>> = OnceLock::new();
    THINGS.get_or_init(|| {
        let recipes = orbs_sim::content::Recipes::builtin();
        let materials = orbs_sim::content::Materials::builtin();
        let scene = orbs_sim::content::corpus_scene();
        let places = scene
            .nouns()
            .iter()
            .filter(|noun| noun.kind == NounKind::Place)
            .filter_map(|noun| noun.name.rsplit('/').next())
            .map(str::to_lowercase);
        recipes
            .vocabulary()
            .into_iter()
            .chain(materials.names())
            .map(str::to_owned)
            .chain(places)
            .collect()
    })
}

/// Every word the corpus teaches the statements opening on `word` with.
///
/// What a statement may spend a verb's word on: *"hold for ten ticks"* is how
/// `bide` is taught, so `hold` there is no second command. The `say` lines
/// only — a holdout is measured against, never learned from.
fn taught_with(word: SpellWord) -> &'static [String] {
    static TAUGHT: OnceLock<Vec<(SpellWord, Vec<String>)>> = OnceLock::new();
    let taught = TAUGHT.get_or_init(|| {
        let mut by: Vec<(SpellWord, Vec<String>)> = Vec::new();
        for entry in Phrasings::spellings().entries() {
            let Some(opens) = orbs_sim::parser::spell_word(&entry.canonical) else {
                continue;
            };
            let words: Vec<String> = entry.say.iter().flat_map(|line| words_of(line)).collect();
            match by.iter_mut().find(|(spell, _)| *spell == opens) {
                Some((_, known)) => known.extend(words),
                None => by.push((opens, words)),
            }
        }
        by
    });
    taught
        .iter()
        .find(|(spell, _)| *spell == word)
        .map_or(&[], |(_, words)| words.as_slice())
}

/// Whether `reading` asks something of the tower — an `if`, or a `repeat
/// until`.
fn asks(reading: &str) -> bool {
    match orbs_sim::parser::spell_word(reading) {
        Some(SpellWord::If) => true,
        Some(SpellWord::Repeat) => reading
            .split_whitespace()
            .nth(1)
            .is_some_and(|word| word == SpellWord::Until.canonical()),
        _ => false,
    }
}

/// Whether `line` says the slotless `shape` — `else` or `end`.
///
/// **They assemble out of any line at all**, having no slot and no count, so
/// the head ranking one first was the only evidence there was: *"do it a few
/// times"* came back `else`. The line has to say it now — a word beginning as
/// one of the stems below, so that *"otherwsie"* and *"endeth"* still count —
/// or, for `end`, nothing at all but filler: *"that's it"* and *"and that's
/// that"* have nothing in them to run.
///
/// `every_phrasing_of_a_slotless_shape_says_it` holds `spellings.toml` to this.
fn said_by(shape: &str, line: &str) -> bool {
    /// What begins a word that says `else`.
    const ELSE: [&str; 5] = ["else", "other", "fail", "instead", "none"];
    /// What begins a word that says `end`.
    const END: [&str; 7] = ["end", "finish", "done", "clos", "stop", "wrap", "enough"];

    let said = words_of(line);
    let begins = |stems: &[&str]| {
        said.iter()
            .any(|word| stems.iter().any(|stem| word.starts_with(stem)))
    };
    match shape {
        "else" => begins(&ELSE) || said.iter().any(|word| NEGATIONS.contains(&word.as_str())),
        "end" => {
            begins(&END)
                || said.iter().any(|word| word == "all" || word == "lot")
                || said.iter().all(|word| {
                    orbs_sim::parser::is_filler(word) || NOTHING_SAID.contains(&word.as_str())
                })
        }
        _ => true,
    }
}

/// Whether `reading` is a statement that says something about the tower.
///
/// A spell word with an argument after it. `end` and `bide 10` are statements
/// with nothing to lose; a command belongs to the other reader entirely.
fn has_a_condition(reading: &str, kept: &[String]) -> bool {
    orbs_sim::parser::spell_word(reading).is_some()
        && kept.iter().skip(1).any(|word| {
            !GRAMMAR.contains(&word.as_str())
                && word.parse::<u32>().is_err()
                && !SpellWord::ALL
                    .iter()
                    .any(|spell| spell.canonical() == word.as_str())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The class index of the shape written exactly like this.
    fn class(canonical: &str) -> usize {
        shapes()
            .iter()
            .position(|shape| shape == canonical)
            .unwrap_or_else(|| panic!("{canonical:?} is not a shape in spellings.toml"))
    }

    #[test]
    fn a_command_is_its_own_class_and_a_shape_is_not() {
        assert_eq!(kinds(), shapes().len() + 1);
        assert_eq!(shape_of(command()), None);
        assert!(shapes().iter().any(|shape| shape == "end"));
        assert_eq!(shape_of(class("bide 10")), Some("bide 10"));
    }

    #[test]
    fn a_count_is_read_from_the_line_and_never_invented() {
        assert_eq!(count_in("wait ten ticks"), Some(10));
        assert_eq!(count_in("bide 30"), Some(30));
        assert_eq!(count_in("do that three times"), Some(3));
        // **The one that matters.** No number, no reading — a `bide` assembled
        // here would be a number the player never said.
        assert_eq!(count_in("hang about a while"), None);
        assert_eq!(count_in("wait a bit"), None);
        assert_eq!(assemble(class("bide 10"), "hang about a while", &[]), None);

        // ...and `one` is a word, not a count. Two of `for each`'s four measured
        // phrasings say it, and none of them means `bide 1`.
        assert_eq!(count_in("for every one of the ways"), None);
        assert!(accounts_for("one way at a time", "for each way"));
        assert_eq!(count_in("wait 1 tick"), Some(1));
    }

    #[test]
    fn a_shape_is_filled_from_the_spans_and_the_line() {
        assert_eq!(
            assemble(
                class("if {place} is idle"),
                "once the mortar_and_pestle falls quiet",
                &["mortar_and_pestle".to_owned()],
            )
            .as_deref(),
            Some("if mortar_and_pestle is idle"),
        );
        assert_eq!(
            assemble(class("bide 10"), "wait ten ticks", &[]).as_deref(),
            Some("bide 10"),
        );
        // A shape wanting two spans and given one is not a reading of this line.
        assert_eq!(
            assemble(
                class("if {place} has {reagent}"),
                "when the alembic has something",
                &["alembic".to_owned()],
            ),
            None,
        );
    }

    #[test]
    fn a_name_is_whatever_word_the_player_chose() {
        // **Any word, not only `tool`.** The shape used to carry `tool` as a
        // literal, so *"name the alembic hammer"* could only be refused.
        let named = |line: &str, slots: &[&str]| {
            let slots: Vec<String> = slots.iter().map(|slot| (*slot).to_owned()).collect();
            assemble(class("let {name} be {place}"), line, &slots)
        };
        assert_eq!(
            named("name the alembic hammer", &["hammer", "alembic"]).as_deref(),
            Some("let hammer be alembic"),
        );
        // ...and a comma is not part of it.
        assert_eq!(
            named(
                "call it hammer, meaning the alembic",
                &["hammer,", "alembic"]
            )
            .as_deref(),
            Some("let hammer be alembic"),
        );
        // A name is one word; the language binds nothing else.
        assert_eq!(
            named("the big hammer is the alembic", &["big hammer", "alembic"]),
            None
        );

        // `pull`'s source is supplied, because every satchel has the one name.
        assert_eq!(
            assemble(
                class("pull {name} from satchel"),
                "fetch herb out of the satchel",
                &["herb".to_owned()],
            )
            .as_deref(),
            Some("pull herb from satchel"),
        );
    }

    #[test]
    fn every_shape_reads_its_slots_by_kind_and_never_two_from_one() {
        // **What makes numbering by kind possible at all.** A shape with two
        // places would need two place slots, and the kind could no longer say
        // which is which — the day one is written, this is what says so.
        for shape in shapes() {
            let slots: Vec<usize> = shape
                .split_whitespace()
                .filter(|token| token.starts_with('{'))
                .map(|marker| {
                    slot_of_marker(marker)
                        .unwrap_or_else(|| panic!("{shape:?} has a slot of no known kind"))
                })
                .collect();
            let mut distinct = slots.clone();
            distinct.sort_unstable();
            distinct.dedup();
            assert_eq!(
                distinct.len(),
                slots.len(),
                "{shape:?} reads two arguments from one slot",
            );
        }

        // ...and the arguments come back in the shape's order, whatever order
        // the slots are numbered in.
        let binding = class("let {name} be {place}");
        assert_eq!(
            ordered(
                binding,
                &[Some("alembic".to_owned()), None, Some("nook".to_owned())]
            ),
            Some(vec!["nook".to_owned(), "alembic".to_owned()]),
        );
        assert_eq!(
            ordered(binding, &[Some("alembic".to_owned()), None, None]),
            None
        );
    }

    #[test]
    fn a_span_of_two_words_is_two_things_and_not_a_name() {
        // Both were offered, parsed, and meant nothing anyone wrote.
        assert_eq!(
            assemble(
                class("repeat until {place} is idle"),
                "let nook refer to the lectern",
                &["nook lectern".to_owned()],
            ),
            None,
        );
        assert_eq!(
            assemble(
                class("if {place} is idle"),
                "take quill away from the satchel",
                &["quill away".to_owned()],
            ),
            None,
        );
    }

    #[test]
    fn a_word_the_game_does_not_know_is_a_name_a_reading_must_keep() {
        // `nook` is a name: nothing in the game's content says it. `let`,
        // `refer` and `the` are all words the vocabulary has.
        let vocabulary = crate::Vocabulary::builtin();
        let unknown = unknown_words("let nook refer to the alembic", &vocabulary);
        assert!(unknown.contains(&"nook".to_owned()), "{unknown:?}");
        assert!(!unknown.contains(&"let".to_owned()), "{unknown:?}");
        assert!(!unknown.contains(&"the".to_owned()), "{unknown:?}");
        assert!(keeps("let nook be alembic", &unknown));
        assert!(!keeps("stop alembic", &unknown));
    }

    #[test]
    fn a_set_is_one_the_tower_has_in_either_number() {
        let walked =
            |line: &str, span: &str| assemble(class("for each {group}"), line, &[span.to_owned()]);
        assert_eq!(
            walked("take each charm in turn", "charm").as_deref(),
            Some("for each charm"),
        );
        // The plural reads back as the set — `for each ways` walks nothing.
        assert_eq!(
            walked("work through the ways", "ways").as_deref(),
            Some("for each way"),
        );
        // A word that is no set is not offered at all.
        assert_eq!(walked("go through each hammer", "hammer"), None);
        // A closed set can be picked out of a longer span; the tagger ran this
        // one into `one bands`.
        assert_eq!(
            walked("for every one of the bands", "one bands").as_deref(),
            Some("for each band"),
        );
    }

    #[test]
    fn only_a_reading_with_room_for_a_name_is_asked_to_keep_one() {
        assert!(holds_a_name(command()));
        assert!(holds_a_name(class("let {name} be {place}")));
        assert!(holds_a_name(class("pull {name} from satchel")));
        assert!(holds_a_name(class("for each {group}")));
        assert!(!holds_a_name(class("if {place} is idle")));
        assert!(!holds_a_name(class("bide 10")));
        assert!(!holds_a_name(class("else")));
    }

    #[test]
    fn nothing_offered_as_a_fallback_can_have_been_built_from_nothing() {
        // `else` and `end` assemble out of any line at all, so they may be the
        // head's first choice and never what the ranking falls to.
        assert!(!fills_from_the_line(class("else")));
        assert!(!fills_from_the_line(class("end")));
        assert!(fills_from_the_line(class("bide 10")));
        assert!(fills_from_the_line(class("if {place} is idle")));

        // A bare verb is a reading of a line only if the line says it.
        assert!(!names_its_verb("for every one of the bands", "status"));
        assert!(!names_its_verb("the stairs creak", "probe"));
        assert!(names_its_verb("have a look around", "survey"));
    }

    #[test]
    fn the_gate_lets_a_loose_line_through_and_keeps_a_statement_out() {
        // **`Scribe`'s first gate, from both sides.** It leaves alone anything
        // `spell::reads_cleanly` accepts, and `content/spellings.toml` is
        // authored so that no phrasing in it is such a line —
        // `a_spelling_is_never_a_line_the_language_already_reads` holds that end.
        for loose in [
            "when the alembic is free",
            "once the alembic has finished",
            "hang on ten ticks",
            "that is all",
        ] {
            assert!(
                !orbs_sim::tower::spell::reads_cleanly(loose),
                "{loose:?} is gated away from the reader",
            );
        }
        // ...and the counter-example, which is *not* a defect: `if the alembic
        // is not busy` is heard as `if not alembic is working` and already means
        // what it says, so a reader overriding it would be the rewriter again.
        for already in [
            "if the alembic is not busy",
            "if alembic is idle",
            "bide 10",
            "morning()",
        ] {
            assert!(
                orbs_sim::tower::spell::reads_cleanly(already),
                "{already:?} reaches the reader and needs no reading",
            );
        }
    }

    #[test]
    fn a_dropped_clause_is_not_accounted_for() {
        // §19's rewriter defect, verbatim: *"`if a has x or b has x` was saved as
        // `if a has x`"*. Both halves parse; only one is what was written.
        assert!(accounts_for(
            "when the alembic has sage",
            "if alembic has sage",
        ));
        assert!(!accounts_for(
            "when the alembic has sage or the retort has sage",
            "if alembic has sage",
        ));
    }

    #[test]
    fn a_dropped_negation_is_not_accounted_for() {
        // The other half of the same class, and worse: it inverts the spell.
        assert!(!accounts_for(
            "when the mortar_and_pestle is not idle",
            "if mortar_and_pestle is idle",
        ));
    }

    #[test]
    fn a_negation_answered_by_an_antonym_is_accounted_for() {
        // **The exemption, and it is not a loophole.** *"not busy"* is what the
        // language spells `idle`, and the `not` is spent on a word the reading
        // does not contain — which is exactly what separates it from the
        // inversion above, where the negated word comes straight through.
        assert!(accounts_for(
            "if the alembic is not busy",
            "if alembic is idle",
        ));
        assert!(accounts_for(
            "once the alembic is no longer busy",
            "if alembic is idle",
        ));
    }

    #[test]
    fn a_negation_is_kept_unless_it_named_the_opposite_of_what_is_read() {
        // *"not busy"* is `idle`; *"not free"* is the inversion the first rule
        // let through, because it excused any negated word the reading lacked.
        assert!(accounts_for(
            "when the alembic is not busy",
            "if alembic is idle"
        ));
        assert!(accounts_for(
            "when the alembic is not in use",
            "if alembic is idle"
        ));
        assert!(!accounts_for(
            "when the alembic isn't free",
            "if alembic is idle"
        ));
        assert!(!accounts_for(
            "when the alembic is never free",
            "if alembic is idle"
        ));
        assert!(!accounts_for(
            "unless the alembic is free",
            "if alembic is idle"
        ));
        assert!(!accounts_for(
            "when the alembic isn't empty",
            "if alembic is empty"
        ));
        // `else` *is* a negation, so it may spend one on choosing the shape.
        assert!(accounts_for("and if it isn't", "else"));
        // A command cannot say `not`, so a negated one is not a reading of it.
        assert!(!accounts_for("never grind the sage", "grind sage"));
        assert!(!accounts_for("do not empty the alembic", "empty alembic"));
    }

    #[test]
    fn two_things_joined_are_not_one_thing_read() {
        assert!(!accounts_for(
            "for each way except the north",
            "for each way"
        ));
        assert!(!accounts_for(
            "grind the sage and then wait ten ticks",
            "bide 10"
        ));
        // ...and a connective spent on nothing is still nothing.
        assert!(accounts_for("and that is that", "end"));
        assert!(accounts_for("or else", "else"));
    }

    #[test]
    fn a_contraction_is_the_words_it_stands_for() {
        assert_eq!(words_of("isn't"), vec!["is", "not"]);
        assert_eq!(
            words_of("the mortar's done."),
            vec!["the", "mortar", "done"]
        );
        assert_eq!(words_of("'til"), vec!["til"]);
        assert_eq!(words_of("rock-salt, please"), vec!["rock-salt", "please"]);
    }

    #[test]
    fn a_statement_with_no_argument_has_no_clause_to_lose() {
        // `or else` is `else`; `three more times` is `repeat 3`. Neither reading
        // carries anything the connective could have been about.
        assert!(accounts_for("or else", "else"));
        assert!(accounts_for("and that is that", "end"));
        assert!(accounts_for("three more times", "repeat 3"));
    }

    #[test]
    fn a_condition_does_not_disappear_into_a_reading_that_asks_nothing() {
        // All three were read, parsed, and ran on every tick they were written
        // to wait for.
        assert!(!accounts_for(
            "whenever the alembic has sage",
            "distil sage"
        ));
        assert!(!accounts_for(
            "after the alembic has finished its work",
            "wield alembic"
        ));
        assert!(!accounts_for(
            "should sage lie within the alembic",
            "distil sage"
        ));
        assert!(!accounts_for(
            "when the alembic is idle, wait ten ticks",
            "bide 10"
        ));
        assert!(accounts_for(
            "whenever the alembic has sage",
            "if alembic has sage"
        ));
        // ...and the same words asking nothing, each a line the corpus teaches.
        assert!(accounts_for("six times, if you would", "repeat 6"));
        assert!(accounts_for("let the sage soak a while", "digest sage"));
        assert!(accounts_for("one socket after another", "for each socket"));
        assert!(accounts_for("the alembic should stop", "stop alembic"));
        assert!(accounts_for("sample once", "probe"));
        assert!(accounts_for("if that fails", "else"));
    }

    #[test]
    fn every_phrasing_the_corpus_teaches_is_accounted_for_by_its_own_reading() {
        // **Every rule here is a claim about how spells are written**, so the
        // corpus is what they are held to. A phrasing its own canonical does not
        // account for is one the reader is taught to answer and then forbidden
        // to: the rules and the corpus disagree, and one of them is wrong.
        let spellings = Phrasings::spellings();
        let scene = orbs_sim::content::corpus_scene();
        let refused: Vec<String> = spellings
            .corpus_by_entry(&scene, orbs_sim::content::CORPUS_CAP)
            .into_iter()
            .chain(spellings.holdout_by_entry(&scene))
            .filter(|(_, example)| !accounts_for(&example.said, &example.canonical))
            .map(|(_, example)| format!("{:?} -> {:?}", example.said, example.canonical))
            .collect();
        assert!(
            refused.is_empty(),
            "{} phrasings their own reading does not account for: {refused:#?}",
            refused.len(),
        );
    }

    #[test]
    fn a_statement_leaves_no_command_behind_and_says_no_opposite() {
        assert!(!accounts_for(
            "when the alembic is idle, grind the sage",
            "if alembic is idle"
        ));
        assert!(!accounts_for(
            "if the alembic is free grind the sage",
            "if alembic is idle"
        ));
        // `hold` is how `bide` is taught, so it is spent there.
        assert!(accounts_for("hold for 20 ticks", "bide 20"));

        assert!(!accounts_for(
            "loop while the alembic is busy",
            "if alembic is idle"
        ));
        assert!(accounts_for(
            "loop while the alembic is busy",
            "repeat until alembic is idle"
        ));
        assert!(accounts_for(
            "when the alembic is not busy",
            "if alembic is idle"
        ));
        // A loop is said: `until` is not the assembler's to supply.
        assert!(!accounts_for(
            "after the alembic has finished its work",
            "repeat until alembic is idle",
        ));
        // A verb's word used as a noun is no second command, and `nothing`
        // negates what follows it.
        assert!(accounts_for(
            "when the alembic has come to a stop",
            "if alembic is idle"
        ));
        assert!(accounts_for(
            "if there is nothing running at the alembic",
            "if alembic is idle"
        ));
        // A substance the line names is not the next shape's to drop.
        assert!(!accounts_for(
            "when the alembic has a bit of sage",
            "if alembic is idle"
        ));
        // `else` and `end` have no room for a command or a thing either — and
        // `stop` is still how `end` is taught.
        assert!(!accounts_for("stop alembic", "end"));
        assert!(!accounts_for("never grind the sage", "else"));
        assert!(accounts_for("and stop", "end"));
        // A condition is said: lines that name something are not conditions,
        // and a question about the tower is one.
        assert!(!accounts_for(
            "name the alembic bertha",
            "if alembic is empty"
        ));
        assert!(!accounts_for(
            "pull out ozzy from the satchel",
            "if ozzy is empty"
        ));
        assert!(accounts_for(
            "does the alembic have sage? then",
            "if alembic has sage"
        ));
        assert!(!accounts_for(
            "does the alembic have sage? then",
            "distil sage"
        ));
        // *As long as* loops, and inverts as `while` does; *stopped working*
        // is not working.
        assert!(accounts_for(
            "carry on as long as the alembic is busy",
            "repeat until alembic is idle",
        ));
        assert!(accounts_for(
            "when the alembic has stopped working",
            "if alembic is idle"
        ));
        // ...and a phrase can ask as well as a word.
        assert!(!accounts_for(
            "the moment the alembic is empty",
            "empty alembic"
        ));
        assert!(!accounts_for("as soon as the flask is free", "wield flask"));
        assert!(accounts_for("survey the second shelf", "survey shelf"));
    }

    #[test]
    fn till_is_until_and_is_kept_as_one() {
        assert!(accounts_for(
            "keep at it till the alembic is free",
            "repeat until alembic is idle",
        ));
        assert!(!accounts_for(
            "keep at it till the alembic is free",
            "if alembic is idle",
        ));
    }

    #[test]
    fn a_count_keeps_its_unit() {
        assert!(!accounts_for("hang on 5 ticks", "repeat 5"));
        assert!(accounts_for("hang on 5 ticks", "bide 5"));
        assert!(!accounts_for("do it 5 times", "bide 5"));
        assert!(accounts_for("do it 5 times", "repeat 5"));
    }

    #[test]
    fn a_shape_with_nothing_to_fill_has_to_be_said() {
        assert_eq!(assemble(class("else"), "do it a few times", &[]), None);
        assert_eq!(assemble(class("end"), "do it a few times", &[]), None);
        assert_eq!(
            assemble(class("else"), "otherwise", &[]).as_deref(),
            Some("else")
        );
        assert_eq!(
            assemble(class("end"), "that's it", &[]).as_deref(),
            Some("end")
        );
    }

    #[test]
    fn every_phrasing_of_a_slotless_shape_says_it() {
        // **The table is a claim about the corpus**, so the corpus is what it is
        // held to: a phrasing of `else` or `end` the rule would refuse is one
        // the reader is taught and then never allowed to answer.
        for entry in Phrasings::spellings()
            .entries()
            .iter()
            .filter(|entry| !fills_from_the_line(class(&entry.canonical)))
        {
            for line in entry.say.iter().chain(&entry.holdout) {
                assert!(
                    said_by(&entry.canonical, line),
                    "{line:?} teaches {:?} and does not say it",
                    entry.canonical,
                );
            }
        }
    }

    #[test]
    fn a_name_is_never_a_word_the_checks_count_on() {
        let named = |line: &str, slots: &[&str]| {
            let slots: Vec<String> = slots.iter().map(|slot| (*slot).to_owned()).collect();
            assemble(class("let {name} be {place}"), line, &slots)
        };
        assert_eq!(
            named("unless the alembic is free", &["unless", "alembic"]),
            None
        );
        assert_eq!(named("not the alembic, surely", &["not", "alembic"]), None);
        assert_eq!(
            named("name the alembic hammer", &["hammer", "alembic"]).as_deref(),
            Some("let hammer be alembic"),
        );
    }

    #[test]
    fn a_reading_keeps_every_name_and_may_spend_a_typo() {
        let vocabulary = crate::Vocabulary::builtin();
        let place = |word: &str| vec![Some(word.to_owned()), None, None];

        let names = names_in("name the alembic bertha", &place("alembic"), &vocabulary);
        assert!(names.contains(&"bertha".to_owned()), "{names:?}");
        assert!(!keeps("if alembic is empty", &names));

        // A slip of a word the game has is a typo, and a reading may spend it.
        let typo = names_in("empty teh alembic", &place("alembic"), &vocabulary);
        assert!(typo.is_empty(), "{typo:?}");

        // ...unless the tagger marked it: then it is the name it found.
        let tagged = names_in(
            "let nook refer to the alembic",
            &[Some("alembic".to_owned()), None, Some("nook".to_owned())],
            &vocabulary,
        );
        assert!(tagged.contains(&"nook".to_owned()), "{tagged:?}");

        // The tagger's own name decides the words it passed over: `nib` was
        // found, so `nickname` is not a second name; `next` is a word the game
        // has, so the `pip` it skipped is the name it missed.
        let nib = names_in(
            "nickname the alembic nib",
            &[Some("alembic".to_owned()), None, Some("nib".to_owned())],
            &vocabulary,
        );
        assert_eq!(nib, vec!["nib".to_owned()]);
        let pip = names_in(
            "take the next pip out of the satchel",
            &[None, None, Some("next".to_owned())],
            &vocabulary,
        );
        assert!(pip.contains(&"pip".to_owned()), "{pip:?}");
        // ...and a span of two words is the tagger running things together.
        let bands = names_in(
            "iterate over the bands",
            &[None, None, Some("iterate bands".to_owned())],
            &vocabulary,
        );
        assert!(bands.is_empty(), "{bands:?}");

        // A condition word, a plural of a set, and an untagged two-letter word
        // are none of them names.
        for (line, placed) in [
            ("whenever the alembic has sage", place("alembic")),
            (
                "for every one of the areas",
                vec![None, None, Some("areas".to_owned())],
            ),
            ("abide ye ten ticks", vec![None, None, None]),
        ] {
            let names = names_in(line, &placed, &vocabulary);
            assert!(names.is_empty(), "{line:?} named {names:?}");
        }
    }

    #[test]
    fn a_number_word_is_accounted_for_by_its_digit() {
        assert!(accounts_for("wait ten ticks", "bide 10"));
        assert!(!accounts_for("wait ten ticks", "bide 30"));
    }

    #[test]
    fn a_commands_filler_and_is_not_a_conditions_and() {
        // The rule is a condition's. `and` joins two questions in an `if`; in a
        // command it is spent on nothing, and refusing there would cost correct
        // readings for a hazard the command language does not have.
        assert!(accounts_for(
            "take the husks out and throw them away",
            "discard husks",
        ));
        assert!(!accounts_for(
            "when the mortar_and_pestle is idle and the alembic is empty",
            "if mortar_and_pestle is idle",
        ));
    }

    #[test]
    fn a_reading_identical_to_the_line_is_no_reading() {
        // The caller stores readings that *differ*, because `Reading::was` is
        // what makes a misreading visible. One that changed nothing would claim
        // a reading happened.
        assert_eq!(assemble(class("end"), "end", &[]), None);
        assert_eq!(
            assemble(class("end"), "that is all", &[]).as_deref(),
            Some("end")
        );
    }

    #[test]
    fn every_shape_assembles_into_something_that_reads_cleanly() {
        // **The whole file, not a sample.** A template whose canonical the spell
        // language cannot parse would be a class the assembler can never emit,
        // and nothing else would say so.
        for (at, shape) in shapes().iter().enumerate() {
            let slots: Vec<String> = shape
                .split_whitespace()
                .filter(|token| token.starts_with('{'))
                .map(|token| match token {
                    "{group}" => groups().first().cloned().expect("a set to walk"),
                    "{name}" => "hammer".to_owned(),
                    _ => "mortar_and_pestle".to_owned(),
                })
                .collect();
            let line = format!("please {}", shape.replace(['{', '}'], ""));
            assert!(
                assemble(at, &line, &slots).is_some(),
                "{shape:?} cannot be assembled",
            );
        }
    }
}
