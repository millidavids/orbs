//! What an `if` asks the tower, read from a line and written back out.
//!
//! # One file, because the two halves must agree
//!
//! The type, the reader and the writer are one concern rather than three. A
//! question the writer emits must be a question the reader accepts — that
//! round-trip is the property the tests lean on hardest, and splitting the pair
//! across files would make it a contract between modules instead of an
//! invariant inside one.
//!
//! # No punctuation, and no precedence to learn the hard way
//!
//! §6's posture is that a player types what they mean. `if count(sage) > 0` is a
//! different program wearing the game's clothes, so there are no comparisons and
//! no brackets — but there **are** connectives, because `if the mortar is idle
//! and the dispensary has sage` is a sentence anyone would write, and before
//! this the parser read the first half and threw the rest away.
//!
//! `not` binds tighter than `and`, which binds tighter than `or`, which is what
//! every language does and what most people expect. Where that is not enough,
//! `either … or …` is a bracket made of words:
//!
//! | typed | means |
//! |---|---|
//! | `a and b or c` | `(a and b) or c` |
//! | `either a or b and c` | `(a or b) and c` |
//! | `a and either b or c` | `a and (b or c)` |
//!
//! `both … and …` is the mirror of it. It never changes what a question means —
//! `and` already binds tighter — but a player who reaches for it gets what they
//! meant, and refusing a word someone would reasonably type is the dead end §6
//! forbids. It earns its keep in the *writer*, where it is the only way to stop
//! a nested `All` flattening into its parent.
//!
//! # Everything must be read, or nothing is
//!
//! **The rule this module exists for.** The parser it replaces took the first
//! `is` it found, read one word after it, and discarded the rest of the line —
//! so `if the mortar is idle and the athanor is working` became
//! `if mortar is idle`, permanently, in the player's file. Here, a question that
//! does not consume every token it was given is not a question: it returns
//! `None`, the line is kept exactly as typed, and the orb says which line it
//! could not read.

use super::normalise::{Tokens, is_filler};
use super::verb::NounKind;

/// A question a spell can ask about the tower.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    /// `the dispensary has sage`.
    Has {
        /// Where to look.
        place: String,
        /// What to look for.
        thing: String,
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
    /// Places and things are collected separately because they fail
    /// differently. A place that is not there makes the question unanswerable —
    /// §8's *Referent missing*, reported by name. A thing that is not there is
    /// simply an answer of no, which is the whole point of
    /// `if the dispensary has ground-sage`: the commonest spell in the game asks
    /// it *before* there is any.
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
            Self::Has { place, thing } => {
                visit(NounKind::Place, place);
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
    /// **Leaving it alone is the honest failure**, not substituting a guess: a
    /// name the room cannot place stays exactly as the player wrote it, so
    /// `holds` finds nothing and the runner reports the word they actually
    /// typed. See [`compile`](crate::tower::spell::compile).
    pub fn rename(&mut self, rename: &mut impl FnMut(NounKind, &str) -> Option<String>) {
        match self {
            Self::Has { place, thing } => {
                if let Some(found) = rename(NounKind::Place, place) {
                    *place = found;
                }
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

    /// The state `word` names, if it names one.
    ///
    /// A closed vocabulary, matched exactly. Fuzzy would put `is idle` and
    /// `is empty` one typo apart from each other on a decision nobody is
    /// watching.
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

/// Write a question back out, as a player could have typed it.
///
/// Round-trips: `condition(&write_condition(&q)) == Some(q)` for every question
/// this module can build, which the property tests hold it to. That is what lets
/// the orb quote a question back — in `interpret`, and in the line that names a
/// place it could not find — without inventing a second notation nobody has seen.
#[must_use]
pub fn write_condition(condition: &Condition) -> String {
    match condition {
        Condition::Has { place, thing } => format!("{place} has {thing}"),
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
/// **Only where precedence does not already say it.** `Any[All[a,b], c]` writes
/// as `a and b or c`, because `and` binds tighter and reading it back gives the
/// same tree — putting `both` there would be noise in the common case. What does
/// need a bracket is a child that would otherwise *flatten into its parent*
/// (`All` in `All`, `Any` in `Any`) or bind too loosely (`Any` in `All`, anything
/// compound under `not`).
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
/// `None` for anything it cannot make sense of — including anything it can only
/// *partly* make sense of. The caller keeps the line exactly as typed and says
/// which line it could not read; nothing here ever guesses, because a guess
/// decides what a laboratory does while nobody is watching.
#[must_use]
pub fn condition(tail: &str) -> Option<Condition> {
    // **A line too long to tokenise is refused, not truncated.** `Tokens::split`
    // caps at `MAX_INPUT` characters and `MAX_WORDS` words, which is right for a
    // typed command and would be the original bug all over again here: a
    // question quietly shortened to its first thirty-two words, with the rest
    // deciding nothing.
    if tail.chars().count() > super::normalise::MAX_INPUT
        || tail.split_whitespace().count() > super::normalise::MAX_WORDS
    {
        return None;
    }

    // Filler goes, **except `and`**, which is on §6's filler list because
    // `mix sage-tincture and ground-salt` fills two slots positionally. A
    // question is the one place it carries meaning, and names that contain it —
    // `mortar_and_pestle`, `flask_and_rod` — are single tokens, so nothing can
    // be swallowed.
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
    /// What makes `the dispensary has sage and charcoal` two questions about one
    /// shelf. Carried on the reader rather than handled inside `comparison` so
    /// that precedence applies to the short form exactly as it does to the long
    /// one: `has sage and charcoal or salt` groups the way `a and b or c` does,
    /// with no second rule to learn or to get wrong.
    subject: Option<(String, Asking)>,
}

/// Which question a subject was being asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Asking {
    Has,
    Is,
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
    /// **The bracket words take `negation`s, not whole sub-questions**, and that
    /// is what makes them brackets at all. Written the obvious way — `either`
    /// reading a full disjunction — the inner disjunction swallows the `and`
    /// that follows it, so `either a or b and c` means `a or (b and c)`: the
    /// grouping word changes nothing, which is the opposite of its purpose.
    /// Taking operands one level down makes `either` close at the first
    /// connective that is not `or`.
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
            // `either a` with no `or`, or `both a or b` with the wrong joiner, is
            // a bracket that never closed. Refused rather than quietly read as
            // the one question inside it.
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
    /// `the dispensary has sage and charcoal` — the second half is a question
    /// about the shelf, and the player did not repeat it. Only taken when the
    /// span ahead carries no `is`/`has` of its own, which is the whole rule:
    /// `has sage and the mortar is idle` is two clauses, not a shelf that holds
    /// a mortar.
    fn shared(&mut self) -> Option<Condition> {
        let (name, asking) = self.subject.clone()?;
        // **Looked at as far as the next connective**, not as far as the next
        // question word — which is what [`span`](Self::span) stops at, and would
        // make every operand look like a short one.
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
        let span = self.span();
        if span.is_empty() {
            return None;
        }
        self.at += span.len();

        let condition = match asking {
            Asking::Has => Condition::Has {
                place: name.to_owned(),
                thing: span.join(" "),
            },
            Asking::Is => Condition::Is {
                place: name.to_owned(),
                // A closed vocabulary, so anything else is not a question the
                // orb can answer — and one word only, because `is idle empty`
                // is not a sentence.
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
    /// **`is` and `has` end a span, and that is not a detail.** Without it the
    /// thing in `the dispensary has sage the mortar is idle` ran to the end of
    /// the line and the whole thing read as a shelf holding something called
    /// *"sage mortar is idle"* — a question that answers no for ever, from a line
    /// with a missing `and` in it. Stopping here leaves those words unconsumed,
    /// and the all-or-nothing rule then refuses the line and says so.
    ///
    /// Nothing in the tower is named `is` or `has`, and if anything ever is, it
    /// will need quoting long before it reaches a question.
    fn span(&self) -> Vec<&'a str> {
        self.to_connective()
            .into_iter()
            .take_while(|word| !matches!(*word, "is" | "has"))
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
/// A one-item `All` would make `a and b` and `a` different shapes for no
/// difference in meaning, and every test would have to know which it was
/// looking at.
///
/// **Flattened for the same reason.** `either a or either b or c` builds an
/// `Any` inside an `Any`, which means exactly what a flat one of three means —
/// `and` and `or` are associative — so keeping the nesting would give one
/// question two trees, and the writer would have to bracket a group that needs
/// no bracket. Only *same-kind* nesting flattens: an `Any` inside an `All` is
/// the grouping `either` exists for and stays.
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
        // **The bug this module exists for.** The parser before it took the
        // first `is`, read one word after it, and dropped the rest — so this
        // line became `mortar is idle` and the other half was written out of the
        // player's file, permanently and silently.
        assert_eq!(read("the mortar is idle the athanor is working"), None);
        assert_eq!(read("the mortar is idle rubbish"), None);
        assert_eq!(read("the mortar is idle and"), None);
    }

    #[test]
    fn either_is_a_bracket_and_both_is_a_courtesy() {
        // The one row that fails if the bracket words are given whole
        // sub-questions instead of operands: the inner disjunction swallows the
        // `and`, and `either` stops meaning anything.
        //
        // **Not `a`, `b`, `c`.** `a` is on §6's filler list, so the tidiest
        // names to write a grammar table with are the one set that cannot appear
        // in one — which cost an afternoon the first time.
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
        Condition::Has {
            place: place.to_owned(),
            thing: thing.to_owned(),
        }
    }
}
