//! What an `if` asks the tower, read from a line and written back out.
//!
//! Type, reader and writer share a file so the round-trip is one module's
//! invariant.
//!
//! No brackets and no operators — §6 says a player types what they mean.
//! Connectives there are, and comparisons spelled out in words (§19).
//!
//! `not` binds tighter than `and`, which binds tighter than `or`. Where that is
//! not enough, `either … or …` is a bracket made of words:
//!
//! | typed | means |
//! |---|---|
//! | `a and b or c` | `(a and b) or c` |
//! | `either a or b and c` | `(a or b) and c` |
//! | `a and either b or c` | `a and (b or c)` |
//!
//! `both … and …` mirrors it and changes no meaning, but refusing a word
//! someone would type is §6's dead end. It earns its keep in the *writer*, the
//! only way to stop a nested `All` flattening into its parent.
//!
//! Everything must be read, or nothing is: the parser this replaces took the
//! first `is`, read one word after it, and dropped the rest. A question that
//! does not consume every token returns `None`, the line is kept as typed, and
//! the orb says which line it could not read.

use super::normalise::{Tokens, is_filler};
use super::verb::NounKind;

/// Which way a count is compared.
///
/// Named rather than a `bool`: `at_most: false` says nothing at a call site.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Bound {
    /// At least this many — `has 4 X`, and `has 4 or more X` said out loud.
    ///
    /// The default. A guard asks *have I enough yet*, so `has 4 fragment` must
    /// stay true at five.
    #[default]
    AtLeast,
    /// At most this many — `has 4 or fewer X`.
    ///
    /// Sayable no other way: *prefer the least-walked* is a rule an at-least
    /// count cannot ask.
    AtMost,
    /// This many and no other — `has exactly 4 X`, or `has = 4 X`.
    ///
    /// Arrived with the symbols; reading the untaught `=` as at-least would be
    /// the quiet reinterpretation §6 forbids. `not has exactly 4 X` asks the
    /// fourth question, so there is no `!=` to learn.
    Exactly,
}

/// Every way a bound can be written, longest match first.
///
/// One table, so reader and writer cannot disagree. Order matters: `<=` before
/// `<`, and `or fewer` shares its first token with `or more`.
///
/// `at least` is here without its `at`, which is §6 filler
/// (`normalise::FILLER`) and stripped before a question is ever read.
const BOUNDS: &[(&[&str], Bound, i64)] = &[
    // Words. `more than 2` is *three or more*, which is why the offsets exist:
    // one shape in the type, several in English.
    (&["or", "more"], Bound::AtLeast, 0),
    (&["or", "fewer"], Bound::AtMost, 0),
    (&["or", "less"], Bound::AtMost, 0),
    (&["least"], Bound::AtLeast, 0),
    (&["most"], Bound::AtMost, 0),
    (&["exactly"], Bound::Exactly, 0),
    (&["more", "than"], Bound::AtLeast, 1),
    (&["greater", "than"], Bound::AtLeast, 1),
    (&["fewer", "than"], Bound::AtMost, -1),
    (&["less", "than"], Bound::AtMost, -1),
    // Symbols, for people who would rather write them. `>=` before `>`.
    (&[">="], Bound::AtLeast, 0),
    (&["=<"], Bound::AtMost, 0),
    (&["<="], Bound::AtMost, 0),
    (&["=="], Bound::Exactly, 0),
    (&["="], Bound::Exactly, 0),
    (&[">"], Bound::AtLeast, 1),
    (&["<"], Bound::AtMost, -1),
];

/// What a count is compared *against*.
///
/// The far side was once a typed number only, so *"the way with the fewest
/// marks"* was inexpressible. `Quantity` rather than `Value`, which is the
/// record field type `watch` imports beside this.
///
/// An expression tree in word notation; §19 records the reversal from refusing
/// arithmetic outright. What holds the ceiling:
///
/// - Words, never symbols — `plus` and `double`, so there is nothing to
///   parenthesise.
/// - One operator: `A - n > B` is `A > B + n`, and the other direction has no
///   clean word — `less` is shipped grammar in `BOUNDS`, `minus` scores 667
///   against `minute`.
/// - No precedence table and no brackets: an expression sits only on the
///   **far** side of a comparative, reads strictly left to right, and cannot
///   contain a comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quantity {
    /// A number the player wrote — `has 4 fragment`.
    Count(u32),
    /// What another place holds of the **same thing** — `has fewer marks than
    /// east`.
    ///
    /// Strictness rides on the bound: English's three comparatives land on the
    /// three [`Bound`]s, and *at least as many* is `not north has fewer marks
    /// than east`, the route the docs above give for `!=`.
    ///
    /// The place is resolved like every other name, so one the tower lacks is
    /// §8's *Referent missing* rather than an answer of no.
    Elsewhere(String),
    /// What another place holds of a **different** thing — `has fewer
    /// quintessence than the d20 has cost`.
    ///
    /// [`Elsewhere`](Self::Elsewhere) compares one reading in two places, so it
    /// cannot ask this when the two sides are named differently.
    Of {
        /// Where to look.
        place: String,
        /// What to look for there.
        thing: String,
    },
    /// Twice what is inside — `than double the garrison`.
    ///
    /// `outnumbered` answered *"the enemy count is twice that of defenders"*
    /// only while the ratio was fixed at two. A word rather than a `*`; see the
    /// module doc.
    Doubled(Box<Quantity>),
    /// What is inside, and `by` more — `than the enemy has mettle plus 6`.
    ///
    /// The only operator: subtraction is `A > B + n` read from the other end.
    Plus {
        /// What to add to.
        of: Box<Quantity>,
        /// How much.
        by: u32,
    },
}

impl Quantity {
    /// This written back as the far side of a comparative, if it is one.
    ///
    /// `None` for [`Count`](Self::Count): a count arrives from `BOUNDS` and
    /// everything else from `COMPARATIVES`, so this and
    /// [`strict`](Self::strict) are the same question.
    fn far(&self) -> Option<String> {
        match self {
            Self::Count(_) => None,
            Self::Elsewhere(place) => Some(place.clone()),
            Self::Of { place, thing } => Some(format!("{place} has {thing}")),
            Self::Doubled(of) => Some(format!("double {}", of.far()?)),
            Self::Plus { of, by } => Some(format!("{} plus {by}", of.far()?)),
        }
    }

    /// Whether the comparison excludes equality.
    ///
    /// A world read on the far side is strict, a number the player typed is
    /// inclusive. Derived from the grammar, not the variant: `matches!(self,
    /// Elsewhere(_))` would make `than the d20` and `than the d20 has
    /// quintessence` disagree at equality.
    #[must_use]
    pub const fn strict(&self) -> bool {
        !matches!(self, Self::Count(_))
    }

    /// Every place and thing named inside, in reading order.
    ///
    /// One walk for the whole tree, so a name nested inside `double the enemy
    /// has mettle plus 6` resolves exactly as a bare one does.
    fn names<'a>(&'a self, visit: &mut impl FnMut(NounKind, &'a str)) {
        match self {
            Self::Count(_) => {}
            Self::Elsewhere(place) => visit(NounKind::Place, place),
            Self::Of { place, thing } => {
                visit(NounKind::Place, place);
                visit(NounKind::Any, thing);
            }
            Self::Doubled(of) | Self::Plus { of, .. } => of.names(visit),
        }
    }

    /// Rewrite every name inside, leaving one alone where `rename` declines.
    fn rename(&mut self, rename: &mut impl FnMut(NounKind, &str) -> Option<String>) {
        match self {
            Self::Count(_) => {}
            Self::Elsewhere(place) => {
                if let Some(found) = rename(NounKind::Place, place) {
                    *place = found;
                }
            }
            Self::Of { place, thing } => {
                if let Some(found) = rename(NounKind::Place, place) {
                    *place = found;
                }
                if let Some(found) = rename(NounKind::Any, thing) {
                    *thing = found;
                }
            }
            Self::Doubled(of) | Self::Plus { of, .. } => of.rename(rename),
        }
    }
}

impl Default for Quantity {
    /// One, which is what a bare `has sage` has always meant.
    fn default() -> Self {
        Self::Count(1)
    }
}

/// Every way one place's count is compared against another's.
///
/// A table beside [`BOUNDS`], for the same reason. The third column closes the
/// phrase — `more marks **than** east` — and is what [`Reader::span`] stops at,
/// so the thing's name ends where the comparison's second half begins.
const COMPARATIVES: &[(&[&str], Bound, &str)] = &[
    (&["more"], Bound::AtLeast, "than"),
    (&["fewer"], Bound::AtMost, "than"),
    (&["less"], Bound::AtMost, "than"),
    (&["as", "many"], Bound::Exactly, "as"),
    (&["as", "much"], Bound::Exactly, "as"),
];

/// The words a thing's name may not run through.
///
/// A span that ate `than` would give `more marks than east` a thing called
/// *"marks than east"*. Joining this list is a permanent reservation: nothing
/// in the tower may ever be named one of them. `double` is not here — it is
/// consumed ahead of the place it modifies.
pub(super) const STOPPERS: &[&str] = &["is", "has", "than", "as", "plus"];

/// A question a spell can ask about the tower.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    /// `the dispensary has sage`, or `the cabinet has 4 fragment`.
    Has {
        /// Where to look.
        place: String,
        /// What to look for.
        thing: String,
        /// How many of it are wanted.
        ///
        /// One unless a number was written — `has sage` *is* `has 1 sage`. At
        /// least, never exactly: a guard asks *"have I enough yet"*. Before
        /// this field the number was silently swallowed (§19).
        ///
        /// A [`Quantity`], so the other side can be the world too;
        /// `Quantity::Count(1)` is exactly the old bare `u32`.
        count: Quantity,
        /// Which side of `count` satisfies it.
        ///
        /// `AtLeast` with nought collapses to `has no X` where the question is
        /// read: "at least nought" is a guard that always fires. `AtMost` with
        /// nought is a real comparison.
        bound: Bound,
    },
    /// `the mortar is idle`.
    Is {
        /// What to ask about.
        place: String,
        /// The state it should be in.
        state: State,
    },
    /// `not the mortar is idle`, and the two spellings that mean it:
    /// `the mortar is not idle`, `the dispensary has no sage`.
    Not(Box<Condition>),
    /// `and` — every one of them.
    All(Vec<Condition>),
    /// `or` — any one of them.
    Any(Vec<Condition>),
}

impl Condition {
    /// Every **place** the question asks about, outermost first.
    ///
    /// Separate from things because they fail differently: a missing place is
    /// §8's *Referent missing*, a missing thing is an answer of no — `if the
    /// dispensary has ground-sage` asks it *before* there is any.
    #[must_use]
    pub fn places(&self) -> Vec<&str> {
        self.names(NounKind::Place)
    }

    /// Every **thing** the question looks for.
    #[must_use]
    pub fn things(&self) -> Vec<&str> {
        self.names(NounKind::Any)
    }

    fn names(&self, wanted: NounKind) -> Vec<&str> {
        let mut out = Vec::new();
        self.walk(&mut |kind, name| {
            if kind == wanted {
                out.push(name);
            }
        });
        out
    }

    /// Visit every name in the tree, in the order it was written.
    fn walk<'a>(&'a self, visit: &mut impl FnMut(NounKind, &'a str)) {
        match self {
            // The compared-against one is a place too, failing as the subject
            // does — §8's *Referent missing*, not an answer of no.
            Self::Has {
                place,
                thing,
                count,
                ..
            } => {
                visit(NounKind::Place, place);
                count.names(visit);
                visit(NounKind::Any, thing);
            }
            Self::Is { place, .. } => visit(NounKind::Place, place),
            Self::Not(inner) => inner.walk(visit),
            Self::All(items) | Self::Any(items) => {
                for item in items {
                    item.walk(visit);
                }
            }
        }
    }

    /// Rewrite every name through `rename`, leaving it alone where that returns
    /// `None`.
    ///
    /// A name the room cannot place stays as the player wrote it, so the runner
    /// reports the word they typed. See
    /// [`compile`](crate::tower::spell::compile).
    pub fn rename(&mut self, rename: &mut impl FnMut(NounKind, &str) -> Option<String>) {
        match self {
            Self::Has {
                place,
                thing,
                count,
                ..
            } => {
                if let Some(found) = rename(NounKind::Place, place) {
                    *place = found;
                }
                count.rename(rename);
                if let Some(found) = rename(NounKind::Any, thing) {
                    *thing = found;
                }
            }
            Self::Is { place, .. } => {
                if let Some(found) = rename(NounKind::Place, place) {
                    *place = found;
                }
            }
            Self::Not(inner) => inner.rename(rename),
            Self::All(items) | Self::Any(items) => {
                for item in items {
                    item.rename(rename);
                }
            }
        }
    }
}

/// A state an instrument can be asked about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Not running anything.
    Idle,
    /// Mid-run.
    Working,
    /// Holding nothing at all.
    Empty,
}

impl State {
    /// The word as it is written in a spell.
    #[must_use]
    pub const fn canonical(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Empty => "empty",
        }
    }

    /// Every spelling [`read`](Self::read) accepts, canonical first within each
    /// state.
    ///
    /// For completion, which must *offer* the vocabulary and not only recognise
    /// it — all eight, because §6 says a register is not second class. A
    /// `match` cannot be enumerated, so
    /// `every_state_word_the_list_offers_is_one_the_language_reads` stops the
    /// two drifting.
    pub const WORDS: [&'static str; 8] = [
        "idle", "free", "still", "working", "busy", "running", "empty", "bare",
    ];

    /// The state `word` names, if it names one.
    ///
    /// Matched exactly: fuzzy would put `is idle` and `is empty` one typo apart
    /// on a decision nobody is watching.
    #[must_use]
    pub fn read(word: &str) -> Option<Self> {
        match word {
            "idle" | "free" | "still" => Some(Self::Idle),
            "working" | "busy" | "running" => Some(Self::Working),
            "empty" | "bare" => Some(Self::Empty),
            _ => None,
        }
    }
}

/// The words that join two questions, and the two that bracket one.
const CONNECTIVES: [&str; 2] = ["and", "or"];

/// Whether `word` joins two halves of a question.
fn is_connective(word: &str) -> bool {
    CONNECTIVES.contains(&word)
}

/// A count moved by a strict comparator's offset.
///
/// `more than 2` is *three or more*. Strictness lives here rather than in
/// everything that answers a bound; counts are whole, so nothing is lost.
///
/// The ends clamp differently: `fewer than 0` floors to *none*, a question the
/// world can answer, while `more than u32::MAX` saturates — clamped to nought
/// it would be a typo that always fires.
fn shift(count: u32, offset: i64) -> u32 {
    let shifted = i64::from(count) + offset;
    u32::try_from(shifted).unwrap_or(if shifted < 0 { 0 } else { u32::MAX })
}

/// Write a question back out, as a player could have typed it.
///
/// Round-trips: `condition(&write_condition(&q)) == Some(q)`, held by the
/// property tests, so the orb can quote a question back without inventing a
/// second notation.
#[must_use]
pub fn write_condition(condition: &Condition) -> String {
    match condition {
        // The count is written only when it is not one, and `or more` never:
        // the bare spelling is canonical. `or fewer` is written, nothing else
        // saying it.
        //
        // One comparative arm, `COMPARATIVES` pairing each bound with its
        // closer. Guarded on `far()` rather than a variant, so every world-read
        // shape shares it.
        Condition::Has {
            place,
            thing,
            count,
            bound,
        } if count.far().is_some() => {
            let other = count.far().unwrap_or_default();
            // Falls back rather than panicking: crashing the game over a badly
            // extended table is worse than writing the equality form.
            // `a_comparative_exists_for_every_bound` holds the table complete.
            let (words, closer) = COMPARATIVES
                .iter()
                .find(|(.., row, _)| row == bound)
                .map_or((&["as", "many"][..], "as"), |(words, _, closer)| {
                    (words, *closer)
                });
            format!("{place} has {} {thing} {closer} {other}", words.join(" "))
        }
        Condition::Has {
            place,
            thing,
            count: Quantity::Count(1),
            bound: Bound::AtLeast,
        } => format!("{place} has {thing}"),
        // Nought at-least keeps its words: the reader collapses `has 0 X` but
        // not `has 0 or more X`, so the short spelling round-tripped to the
        // negation of what was typed.
        Condition::Has {
            place,
            thing,
            count: Quantity::Count(0),
            bound: Bound::AtLeast,
        } => format!("{place} has 0 or more {thing}"),
        Condition::Has {
            place,
            thing,
            count: Quantity::Count(count),
            bound: Bound::AtLeast,
        } => format!("{place} has {count} {thing}"),
        Condition::Has {
            place,
            thing,
            count: Quantity::Count(count),
            bound: Bound::AtMost,
        } => format!("{place} has {count} or fewer {thing}"),
        // Words, not the symbol that may have been typed: the fair copy reads
        // for a player who has never seen an operator.
        Condition::Has {
            place,
            thing,
            count: Quantity::Count(count),
            bound: Bound::Exactly,
        } => format!("{place} has exactly {count} {thing}"),
        // Unreachable rather than `unreachable!`: only a `Quantity` variant
        // added without a `far()` arm reaches here, and the round-trip test
        // catches that without a panic at a player's expense.
        Condition::Has {
            place,
            thing,
            count: _,
            bound: _,
        } => format!("{place} has {thing}"),
        Condition::Is { place, state } => format!("{place} is {}", state.canonical()),
        Condition::Not(inner) => format!("not {}", bracketed(inner, Around::Not)),
        Condition::All(items) => join(items, "and", Around::All),
        Condition::Any(items) => join(items, "or", Around::Any),
    }
}

/// What a sub-question is being written inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Around {
    Not,
    All,
    Any,
}

fn join(items: &[Condition], word: &str, around: Around) -> String {
    items
        .iter()
        .map(|item| bracketed(item, around))
        .collect::<Vec<_>>()
        .join(&format!(" {word} "))
}

/// One sub-question, with a bracket word if it needs one.
///
/// Only where precedence does not already say it. A bracket is needed for a
/// child that would flatten into its parent (`All` in `All`, `Any` in `Any`) or
/// bind too loosely (`Any` in `All`, anything compound under `not`).
fn bracketed(item: &Condition, around: Around) -> String {
    let written = write_condition(item);
    let needed = matches!(
        (around, item),
        (
            Around::Not | Around::All,
            Condition::All(_) | Condition::Any(_)
        ) | (Around::Any, Condition::Any(_))
    );
    if !needed {
        return written;
    }
    match item {
        Condition::All(_) => format!("both {written}"),
        _ => format!("either {written}"),
    }
}

/// Read the tail of an `if` as a question.
///
/// `None` for anything it cannot make sense of, including anything it can only
/// *partly* make sense of. A guess here decides what a laboratory does while
/// nobody is watching.
#[must_use]
pub fn condition(tail: &str) -> Option<Condition> {
    // Refused, not truncated: `Tokens::split` caps at `MAX_INPUT` characters
    // and `MAX_WORDS` words, which here would silently shorten a question to
    // its first thirty-two words.
    if tail.chars().count() > super::normalise::MAX_INPUT
        || tail.split_whitespace().count() > super::normalise::MAX_WORDS
    {
        return None;
    }

    // Filler goes, except `and` — §6 filler because `mix sage-tincture and
    // ground-salt` fills two slots positionally, but a question is the one
    // place it carries meaning.
    let tokens = Tokens::split(tail);
    let words: Vec<&str> = tokens
        .words()
        .iter()
        .map(|word| word.matching)
        .filter(|word| *word == "and" || !is_filler(word))
        .collect();

    let mut reader = Reader {
        words: &words,
        at: 0,
        subject: None,
    };
    let question = reader.disjunction()?;
    // Everything, or nothing. See the module docs.
    if reader.at != words.len() {
        return None;
    }
    Some(question)
}

/// A cursor over a question's words, and what the last clause was about.
struct Reader<'a> {
    words: &'a [&'a str],
    at: usize,
    /// The place the last comparison asked about, and how it asked.
    ///
    /// What makes `the dispensary has sage and charcoal` two questions about
    /// one shelf. On the reader so precedence applies to the short form too.
    subject: Option<(String, Asking)>,
}

/// Which question a subject was being asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Asking {
    Has,
    Is,
}

/// What [`Reader::eat_comparative`] found.
///
/// Three answers, not an `Option`: an unclosed comparative — `north has fewer
/// marks` — must refuse the line rather than become `north has marks`, which
/// answers yes where the player's answers no (§19).
enum Comparative {
    /// No comparative word here; the words ahead are something else.
    Absent,
    /// `fewer marks than east` — the bound, the thing, and what to compare
    /// against.
    ///
    /// The third field is the whole far side, not a place name: flattening
    /// `double the enemy has mettle plus 6` back to a string would mean parsing
    /// it twice.
    Read(Bound, String, Quantity),
    /// A comparative with nothing to compare against.
    Unclosed,
}

impl<'a> Reader<'a> {
    fn peek(&self) -> Option<&'a str> {
        self.words.get(self.at).copied()
    }

    fn take(&mut self) -> Option<&'a str> {
        let word = self.peek()?;
        self.at += 1;
        Some(word)
    }

    fn eat(&mut self, word: &str) -> bool {
        if self.peek() == Some(word) {
            self.at += 1;
            return true;
        }
        false
    }

    /// A leading number, if the next word is one — `has 4 fragment`.
    ///
    /// Only called on the `has` side; `is 4 idle` is not a sentence. A number
    /// too large for a `u32` falls through to [`span`](Self::span) and becomes
    /// part of a name nothing is called, so §8.1 gets its culprit — saturating
    /// would make a typo a guard that never fires.
    fn eat_count(&mut self) -> Option<u32> {
        let count = self.peek()?.parse().ok()?;
        self.at += 1;
        Some(count)
    }

    /// The [`BOUNDS`] row written at `at`, if one is — words and length.
    fn bound_at(&self, at: usize) -> Option<(usize, Bound, i64)> {
        BOUNDS.iter().find_map(|(words, bound, offset)| {
            let matched = words
                .iter()
                .enumerate()
                .all(|(step, word)| self.words.get(at + step) == Some(word));
            matched.then_some((words.len(), *bound, *offset))
        })
    }

    /// A symbol written against its number — `>=2`, with no space.
    ///
    /// Invisible to a token-at-a-time reader, the whole thing arriving as one
    /// word. Longest operator first, so `>=2` is not read as `>` against `=2`.
    fn glued(&self) -> Option<(Bound, i64, u32)> {
        let word = self.peek()?;
        BOUNDS.iter().find_map(|(words, bound, offset)| {
            let [symbol] = words else { return None };
            let rest = word.strip_prefix(symbol)?;
            Some((*bound, *offset, rest.parse().ok()?))
        })
    }

    /// The count and which side of it satisfies, however it was written.
    ///
    /// `has 2 marks`, `has at least 2 marks`, `has 2 or more marks`,
    /// `has more than 1 mark`, `has >= 2 marks`, `has >=2 marks`,
    /// `has exactly 2 marks` — one question, seven ways of asking it.
    ///
    /// Returns whether a bound was *written*: a bare `has 0 X` collapses to
    /// `has no X` and an explicit `has 0 or more X` must not, the player having
    /// asked for the vacuous question by name.
    ///
    /// The postfix is read here because the `or` must be claimed before the
    /// disjunction parser sees it, and this is the only place that can:
    /// `disjunction` → `conjunction` → `comparison` → here. Whole row or
    /// nothing, so `has 2 sage or the mortar is idle` still reads as one.
    fn eat_bounded_count(&mut self) -> Option<(u32, Bound, bool)> {
        if let Some((bound, offset, count)) = self.glued() {
            self.at += 1;
            return Some((shift(count, offset), bound, true));
        }
        let start = self.at;
        let prefix = self.bound_at(self.at);
        if let Some((words, ..)) = prefix {
            self.at += words;
        }
        let Some(count) = self.eat_count() else {
            // A bound with no number is not a question. Put the words back so
            // `span` sees them and the line is refused naming what it read.
            self.at = start;
            return None;
        };
        if let Some((_, bound, offset)) = prefix {
            return Some((shift(count, offset), bound, true));
        }
        if let Some((words, bound, offset)) = self.bound_at(self.at) {
            self.at += words;
            return Some((shift(count, offset), bound, true));
        }
        Some((count, Bound::AtLeast, false))
    }

    /// A comparison against another place — `fewer marks than east`.
    ///
    /// A whole phrase or not at all: `north has more marks` without the `than`
    /// is a shelf holding *"more marks"*, so the cursor goes back and the
    /// all-or-nothing rule refuses the line.
    ///
    /// The thing is read here, not by the caller, because the comparative sits
    /// on the wrong side of it (`fewer marks than`, not `marks fewer than`).
    fn eat_comparative(&mut self) -> Comparative {
        let start = self.at;
        let Some((bound, closer)) = COMPARATIVES.iter().find_map(|(words, bound, closer)| {
            let matched = words
                .iter()
                .enumerate()
                .all(|(step, word)| self.words.get(self.at + step) == Some(word));
            matched.then(|| {
                self.at += words.len();
                (*bound, *closer)
            })
        }) else {
            return Comparative::Absent;
        };

        let thing = self.span();
        self.at += thing.len();
        // Nothing between the comparative and its closer is the *number* form:
        // `more than 1 fragment` is a `BOUNDS` spelling sharing its first word
        // with `more marks than east`, and the gap is the discriminator.
        if thing.is_empty() {
            self.at = start;
            return Comparative::Absent;
        }
        if !self.eat(closer) {
            self.at = start;
            return Comparative::Unclosed;
        }

        let Some(other) = self.far_side() else {
            self.at = start;
            return Comparative::Unclosed;
        };
        Comparative::Read(bound, thing.join(" "), other)
    }

    /// The far side of a comparative — `[double] <place> [has <thing>] [plus n]`.
    ///
    /// Left to right, and it cannot contain a comparison, so there is nothing
    /// to bracket. `double` binds to the whole of what follows.
    ///
    /// The place reads to a [`STOPPERS`] word, not to the next connective,
    /// which gave `than the d20 has quintessence` a place called *"d20 has
    /// quintessence"*. Multi-word names still work; `balneum mariae` contains
    /// no stopper.
    fn far_side(&mut self) -> Option<Quantity> {
        let doubled = self.eat("double");

        let place = self.span();
        self.at += place.len();
        if place.is_empty() {
            return None;
        }
        let place = place.join(" ");

        // `has <thing>` names a *different* reading over there. Without it the
        // far side can only ask about the same word as the near side.
        let mut quantity = if self.eat("has") {
            let thing = self.span();
            self.at += thing.len();
            if thing.is_empty() {
                return None;
            }
            Quantity::Of {
                place,
                thing: thing.join(" "),
            }
        } else {
            Quantity::Elsewhere(place)
        };

        if doubled {
            quantity = Quantity::Doubled(Box::new(quantity));
        }

        // `plus` last, so it applies to the whole of what came before:
        // `double the enemy plus 2` is `(2 x enemy) + 2`.
        if self.eat("plus") {
            let by = self.eat_count()?;
            quantity = Quantity::Plus {
                of: Box::new(quantity),
                by,
            };
        }

        Some(quantity)
    }

    fn disjunction(&mut self) -> Option<Condition> {
        let mut items = vec![self.conjunction()?];
        while self.eat("or") {
            items.push(self.conjunction()?);
        }
        Some(gather(items, Condition::Any))
    }

    fn conjunction(&mut self) -> Option<Condition> {
        let mut items = vec![self.negation()?];
        while self.eat("and") {
            items.push(self.negation()?);
        }
        Some(gather(items, Condition::All))
    }

    fn negation(&mut self) -> Option<Condition> {
        if self.eat("not") {
            return Some(Condition::Not(Box::new(self.negation()?)));
        }
        self.atom()
    }

    /// A bracketed group, or a plain comparison.
    ///
    /// The bracket words take `negation`s, not whole sub-questions: with a full
    /// disjunction the inner one swallows the following `and`, so `either a or
    /// b and c` means `a or (b and c)` and `either` groups nothing.
    fn atom(&mut self) -> Option<Condition> {
        for (word, joiner, gathered) in [
            (
                "either",
                "or",
                Condition::Any as fn(Vec<Condition>) -> Condition,
            ),
            (
                "both",
                "and",
                Condition::All as fn(Vec<Condition>) -> Condition,
            ),
        ] {
            if !self.eat(word) {
                continue;
            }
            let mut items = vec![self.negation()?];
            while self.eat(joiner) {
                items.push(self.negation()?);
            }
            // `either a` with no `or`, or `both a or b`, is a bracket that
            // never closed. Refused rather than read as the question inside.
            if items.len() < 2 {
                return None;
            }
            return Some(gather(items, gathered));
        }
        self.comparison()
    }

    /// One clause: `<name> is <state>` or `<name> has <thing>` — or the short
    /// form, which is the tail of one that came before.
    fn comparison(&mut self) -> Option<Condition> {
        if let Some(shared) = self.shared() {
            return Some(shared);
        }

        let start = self.at;
        while let Some(word) = self.peek() {
            if matches!(word, "is" | "has") {
                break;
            }
            // A subject's own name still may not run through a comparison
            // word — `than east has sage` names nothing on the left.
            if matches!(word, "than") {
                return None;
            }
            // A connective before the question word means there was no question
            // here at all — `a and b is idle` names nothing on the left.
            if is_connective(word) || matches!(word, "not" | "either" | "both") {
                return None;
            }
            self.at += 1;
        }
        // A name may be several words (`balneum mariae`), which is why this
        // scans rather than taking one token.
        let name = self.words.get(start..self.at)?.join(" ");
        let asking = match self.take()? {
            "is" => Asking::Is,
            _ => Asking::Has,
        };
        if name.is_empty() {
            return None;
        }
        self.subject = Some((name.clone(), asking));
        self.operand(&name, asking)
    }

    /// The short form: an operand with the subject left off.
    ///
    /// `the dispensary has sage and charcoal`. Only taken when the span ahead
    /// carries no `is`/`has` of its own, so `has sage and the mortar is idle`
    /// stays two clauses rather than a shelf holding a mortar.
    fn shared(&mut self) -> Option<Condition> {
        let (name, asking) = self.subject.clone()?;
        // As far as the next connective, not the next question word: stopping
        // where `span` does would make every operand look like a short one.
        let ahead = self.to_connective();
        if ahead.is_empty() || ahead.iter().any(|word| matches!(*word, "is" | "has")) {
            return None;
        }
        self.operand(&name, asking)
    }

    /// One operand of a clause, negation included, up to the next connective.
    fn operand(&mut self, name: &str, asking: Asking) -> Option<Condition> {
        let negated = match asking {
            // `has no sage`.
            Asking::Has => self.eat("no"),
            // `is not idle`.
            Asking::Is => self.eat("not"),
        };
        // Before the count, and they cannot collide: a comparative opens with a
        // word, `eat_bounded_count` needs a digit, and either puts the cursor
        // back. This one first because it reads the thing itself.
        if asking == Asking::Has {
            match self.eat_comparative() {
                Comparative::Absent => {}
                Comparative::Unclosed => return None,
                Comparative::Read(bound, thing, count) => {
                    let condition = Condition::Has {
                        place: name.to_owned(),
                        thing,
                        count,
                        bound,
                    };
                    return Some(if negated {
                        Condition::Not(Box::new(condition))
                    } else {
                        condition
                    });
                }
            }
        }

        // After the negation and before the span, the only place it can go, or
        // `span` takes the digit as the first word of the thing's name —
        // `has 4 fragment` meaning a thing called *"4 fragment"*.
        let counted = match asking {
            Asking::Has => self.eat_bounded_count(),
            Asking::Is => None,
        };
        let (counted, bound, written) = match counted {
            Some((count, bound, written)) => (Some(count), bound, written),
            None => (None, Bound::AtLeast, false),
        };
        let span = self.span();
        if span.is_empty() {
            return None;
        }
        self.at += span.len();

        // `has 0 sage` is `has no sage` with a digit: "at least nought" is a
        // guard that always fires. Only when no bound was written — the
        // explicit `has 0 or more sage` asks the vacuous question by name.
        let bare_nought = counted == Some(0) && bound == Bound::AtLeast && !written;
        let negated = negated || bare_nought;
        let condition = match asking {
            Asking::Has => Condition::Has {
                place: name.to_owned(),
                thing: span.join(" "),
                count: Quantity::Count(if bare_nought { 1 } else { counted.unwrap_or(1) }),
                bound,
            },
            Asking::Is => Condition::Is {
                place: name.to_owned(),
                // A closed vocabulary, one word only: `is idle empty` is not a
                // sentence.
                state: State::read(span.first()?).filter(|_| span.len() == 1)?,
            },
        };
        Some(if negated {
            Condition::Not(Box::new(condition))
        } else {
            condition
        })
    }

    /// The words from here to the next connective **or question word**, without
    /// consuming them.
    ///
    /// `is` and `has` end a span, or a line missing an `and` gives `the
    /// dispensary has sage the mortar is idle` a shelf holding *"sage mortar is
    /// idle"*. The words left unconsumed make the all-or-nothing rule refuse
    /// the line instead.
    ///
    /// Nothing in the tower is named `is` or `has`.
    fn span(&self) -> Vec<&'a str> {
        self.to_connective()
            .into_iter()
            .take_while(|word| !STOPPERS.contains(word))
            .collect()
    }

    /// The words from here to the next connective, without consuming them.
    ///
    /// The wider of the two lookaheads: what [`shared`](Self::shared) reads to
    /// decide whether the operand ahead brings its own question.
    fn to_connective(&self) -> Vec<&'a str> {
        self.words[self.at..]
            .iter()
            .copied()
            .take_while(|word| !is_connective(word))
            .collect()
    }
}

/// One item is itself; several are the group, flattened.
///
/// A one-item `All` would give `a` and `a and b` different shapes for the same
/// meaning; `and` and `or` being associative, the `Any` inside an `Any` that
/// `either a or either b or c` builds would give one question two trees. Only
/// same-kind nesting flattens — an `Any` inside an `All` is the grouping
/// `either` exists for.
fn gather(mut items: Vec<Condition>, group: fn(Vec<Condition>) -> Condition) -> Condition {
    if items.len() == 1 {
        return items.remove(0);
    }
    let joining_with_and = matches!(group(Vec::new()), Condition::All(_));
    let mut flat = Vec::with_capacity(items.len());
    for item in items {
        match item {
            Condition::All(inner) if joining_with_and => flat.extend(inner),
            Condition::Any(inner) if !joining_with_and => flat.extend(inner),
            other => flat.push(other),
        }
    }
    group(flat)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The question `text` asks, written back out — the shape the tables use.
    fn read(text: &str) -> Option<String> {
        condition(text).as_ref().map(write_condition)
    }

    /// Every state word the completer offers is one the language reads.
    ///
    /// Two lists of one vocabulary went wrong three times (§19). Both
    /// directions: a word missing from [`State::WORDS`] is one the editor
    /// silently refuses to help with.
    #[test]
    fn every_state_word_the_list_offers_is_one_the_language_reads() {
        for word in State::WORDS {
            assert!(
                State::read(word).is_some(),
                "`{word}` is offered and the language does not read it",
            );
        }

        // The canonical three lead their own runs, which is what makes the
        // offered list read as three answers rather than eight.
        for state in [State::Idle, State::Working, State::Empty] {
            assert!(
                State::WORDS.contains(&state.canonical()),
                "{state:?}'s canonical spelling is not offered at all",
            );
        }

        // And nothing is offered twice.
        let mut sorted = State::WORDS.to_vec();
        sorted.sort_unstable();
        let mut unique = sorted.clone();
        unique.dedup();
        assert_eq!(sorted, unique, "a state word is offered twice");
    }

    #[test]
    fn a_question_is_read_and_written_the_same_way_round() {
        assert_eq!(
            read("the mortar is idle").as_deref(),
            Some("mortar is idle"),
        );
        assert_eq!(
            read("the dispensary has sage").as_deref(),
            Some("dispensary has sage"),
        );
    }

    #[test]
    fn everything_must_be_read_or_nothing_is() {
        // The bug this module exists for: the old parser took the first `is`,
        // read one word after it, and dropped the rest, so this line became
        // `mortar is idle` in the player's file.
        assert_eq!(read("the mortar is idle the athanor is working"), None);
        assert_eq!(read("the mortar is idle rubbish"), None);
        assert_eq!(read("the mortar is idle and"), None);
    }

    #[test]
    fn either_is_a_bracket_and_both_is_a_courtesy() {
        // The row that fails if the bracket words take whole sub-questions:
        // the inner disjunction swallows the `and`.
        //
        // Not `a`, `b`, `c` — `a` is on §6's filler list.
        assert_eq!(
            condition("either mortar is idle or flask is idle and alembic is idle"),
            Some(Condition::All(vec![
                Condition::Any(vec![idle("mortar"), idle("flask")]),
                idle("alembic"),
            ])),
        );
        assert_eq!(
            read("either mortar is idle or flask is idle and alembic is idle").as_deref(),
            Some("either mortar is idle or flask is idle and alembic is idle"),
        );
        // `both` changes no meaning: with it and without it are the same tree.
        assert_eq!(
            condition("both mortar is idle and flask is idle or alembic is idle"),
            condition("mortar is idle and flask is idle or alembic is idle"),
        );
    }

    #[test]
    fn a_subject_carries_to_the_next_operand() {
        assert_eq!(
            condition("the dispensary has sage and charcoal"),
            Some(Condition::All(vec![
                has("dispensary", "sage"),
                has("dispensary", "charcoal"),
            ])),
        );
        // ...but not past an operand that brought its own.
        assert_eq!(
            condition("the dispensary has sage and the mortar is idle"),
            Some(Condition::All(vec![
                has("dispensary", "sage"),
                idle("mortar"),
            ])),
        );
    }

    fn idle(place: &str) -> Condition {
        Condition::Is {
            place: place.to_owned(),
            state: State::Idle,
        }
    }

    fn has(place: &str, thing: &str) -> Condition {
        counted(place, thing, 1)
    }

    /// `has <count> <thing>` — the same question with a number written on it.
    fn counted(place: &str, thing: &str, count: u32) -> Condition {
        Condition::Has {
            place: place.to_owned(),
            thing: thing.to_owned(),
            count: Quantity::Count(count),
            bound: Bound::AtLeast,
        }
    }
}
