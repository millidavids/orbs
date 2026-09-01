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
//! different program wearing the game's clothes, so there are no brackets and no
//! operators — but there **are** connectives, because `if the mortar is idle and
//! the dispensary has sage` is a sentence anyone would write, and before this the
//! parser read the first half and threw the rest away.
//!
//! **And there are comparisons now, spelled out rather than punctuated.** `if the
//! cabinet has 2 or more fragment` is the sentence; `>=` is the different program.
//! This paragraph used to say there were none at all, which was true when the
//! only counted thing in the game reported itself in two buckets — see §19.
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

/// Which way a count is compared.
///
/// **Named rather than a `bool`**, because the call sites read it: `at_most:
/// false` at a construction site says nothing, and this is a comparison the
/// player wrote in words.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Bound {
    /// At least this many — `has 4 X`, and `has 4 or more X` said out loud.
    ///
    /// **The default, and the one a bare count means.** A guard asks *have I
    /// enough yet*, so `has 4 fragment` must stay true at five or the spell that
    /// spends four stops working the moment something gets ahead of it.
    #[default]
    AtLeast,
    /// At most this many — `has 4 or fewer X`.
    ///
    /// The direction that cannot be said any other way, and the reason
    /// comparators earn their place: *prefer the least-walked* is a rule the
    /// world can answer and an at-least count cannot ask.
    AtMost,
    /// This many and no other — `has exactly 4 X`, or `has = 4 X`.
    ///
    /// **The third bound, and it arrived with the symbols.** The words gave two
    /// directions and no way to say *this many*; `=` is a thing people write
    /// without being taught, and reading it as at-least would be the quiet
    /// reinterpretation §6 forbids. `not has exactly 4 X` is how the fourth
    /// question is asked, so there is no `!=` to learn.
    Exactly,
}

/// Every way a bound can be written, longest match first.
///
/// **A table, so the reader and the writer cannot disagree**, and so a new
/// spelling is one row rather than an arm in each. Order matters: `<=` must be
/// tried before `<`, and `or fewer` shares its first token with `or more`.
///
/// **`at least` is here without its `at`.** That word is §6 filler
/// (`normalise::FILLER`) and is stripped before a question is ever read, which
/// is exactly why `has at least 2 marks` used to silently become `has marks` —
/// the same swallow counting was added to close, left open for the spelling
/// nobody had tried.
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
/// # The one thing the language could not say
///
/// Every quantity in the tower is read the same way — a named child, and its
/// `Stock` — which is what lets `has 2 or more marks` and `has 4 fragment` be one
/// piece of arithmetic (`tower::build::raise_count` says so outright). What was
/// missing was the **other side**: a comparison could only ever name a number the
/// player typed, so *"the way with the fewest marks"* — the sentence `threading`
/// is 52 hand-unrolled lines for — was inexpressible.
///
/// **Named `Quantity` rather than `Value`.** `orbs_render::Value` is the record
/// field type and `watch` imports it beside this; two things called `Value` in
/// one file is a rename waiting to happen.
///
/// # An expression tree with a hard depth cap, in word notation
///
/// **This paragraph used to refuse arithmetic outright** — *"no arithmetic here
/// and no nesting: a world read on one side and a number or one other world read
/// on the other"* — and §19 records the reversal rather than quietly
/// contradicting it. Weighing the siege's pool against what a die costs is the
/// thing that broke it: the tower now has a resource whose whole point is being
/// compared, and *"the world publishes a derived word"* stops scaling when the
/// question is `is this one worth more than that one`.
///
/// **What is honest about the new shape.** Once [`Of`](Self::Of) exists this
/// *is* a tree: `Doubled` and `Plus` wrap it, so the old paragraph's own example
/// — `north has marks + 1 than east` — is expressible, as `Plus` with words
/// instead of punctuation. Calling that "not an expression tree" would describe
/// the notation and not the shape.
///
/// **What holds the ceiling instead**, and each of these is load-bearing:
///
/// - **Words, never symbols.** `plus` and `double`, so there is nothing to
///   parenthesise and §6's *"a player types what they mean"* survives.
/// - **One operator, and subtraction is deliberately absent.** `A - n > B` is
///   `A > B + n`, so one word covers both directions — and there is no clean word
///   for the other: `less` is already shipped grammar in `BOUNDS` and `minus`
///   scores 667 against `minute`.
/// - **No precedence table**, because there is nothing to disambiguate: an
///   expression sits only on the **far** side of a comparative, reads strictly
///   left to right, and cannot contain a comparison.
/// - **No brackets**, which follows from the above rather than being a separate
///   rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quantity {
    /// A number the player wrote — `has 4 fragment`.
    Count(u32),
    /// What another place holds of the **same thing** — `has fewer marks than
    /// east`.
    ///
    /// # Strictness rides on the bound, so there is no fourth spelling
    ///
    /// English gives three comparatives and they land exactly on the three
    /// [`Bound`]s: `more … than` is strictly more, `fewer … than` strictly
    /// fewer, `as many … as` equal. *At least as many* is deliberately absent —
    /// `not north has fewer marks than east` already says it, which is the same
    /// route the docs above give for `!=` rather than teaching an operator.
    ///
    /// The place is a **name**, resolved like every other, so a comparison
    /// against somewhere the tower does not have is §8's *Referent missing* and
    /// stops the question rather than answering it.
    Elsewhere(String),
    /// What another place holds of a **different** thing — `has fewer
    /// quintessence than the d20 has cost`.
    ///
    /// The variant the siege asked for. [`Elsewhere`](Self::Elsewhere) compares
    /// one reading in two places, which cannot ask *is what I hold less than what
    /// this costs* when the two are named differently on each side.
    Of {
        /// Where to look.
        place: String,
        /// What to look for there.
        thing: String,
    },
    /// Twice what is inside — `than double the garrison`.
    ///
    /// **The user's own motivating example** (*"if the enemy count is twice that
    /// of defenders"*), which the readings answered with a published word
    /// (`outnumbered`) for as long as the ratio was fixed at two. It is a word
    /// rather than a `*` for the reason the module doc gives.
    Doubled(Box<Quantity>),
    /// What is inside, and `by` more — `than the enemy has mettle plus 6`.
    ///
    /// **The only operator, and it is enough for both directions.** Subtraction
    /// is `A > B + n` read from the other end; see the module doc for why there
    /// is no word for it.
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
    /// **`None` for [`Count`](Self::Count), which is the whole discriminator.**
    /// A count arrives from `BOUNDS` (`has 2 or fewer marks`) and everything
    /// else from `COMPARATIVES` (`than …`) — so *"does this render a far
    /// side"* and *"did the player write a comparative"* are one question, and
    /// [`strict`](Self::strict) below is the same question again.
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
    /// **Derived from the grammar, not from the variant**, and the distinction
    /// is a real defect avoided. Written as `matches!(self, Elsewhere(_))` the
    /// three variants above would all fall to *inclusive*, so `than the d20` and
    /// `than the d20 has quintessence` — two spellings of one question — would
    /// disagree at equality, and `plus 0` would change a sentence's meaning.
    ///
    /// Asked this way there is one rule: **a world read on the far side is
    /// strict, a number the player typed is inclusive.** `has 2 or fewer marks`
    /// includes two; `has fewer marks than east` does not. That is English, and
    /// it is why there is no *at least as many* — `not … fewer … than` says it.
    #[must_use]
    pub const fn strict(&self) -> bool {
        !matches!(self, Self::Count(_))
    }

    /// Every place and thing named inside, in reading order.
    ///
    /// One walk for the whole tree, so a name nested inside `double the enemy
    /// has mettle plus 6` is resolved and reported exactly as a bare one is.
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
/// A table beside [`BOUNDS`] and for the same reason: the reader and the writer
/// read one list, so a new spelling is a row rather than an arm in each.
///
/// The third column is the word that closes the phrase — `more marks **than**
/// east`, `as many marks **as** east`. It is what [`Reader::span`] stops at, so
/// the thing's name ends where the comparison's second half begins.
const COMPARATIVES: &[(&[&str], Bound, &str)] = &[
    (&["more"], Bound::AtLeast, "than"),
    (&["fewer"], Bound::AtMost, "than"),
    (&["less"], Bound::AtMost, "than"),
    (&["as", "many"], Bound::Exactly, "as"),
    (&["as", "much"], Bound::Exactly, "as"),
];

/// The words a thing's name may not run through.
///
/// `is` and `has` were the first two and the module docs say why. `than` and
/// `as` join a comparison to its second half, so a span that ate them would give
/// `more marks than east` a thing called *"marks than east"* — the same silent
/// swallow, one grammar wider.
///
/// **`plus` is the fifth, and joining this list is a permanent reservation.**
/// Nothing in the tower may ever be named any of them: a reading or a material
/// called `plus` would be unnameable, because a span stops before it rather than
/// eating it. That is the price of the operator and it is paid once — `double`
/// is *not* here, because it is consumed ahead of the place it modifies rather
/// than being something a span could run through.
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
        /// **One unless a number was written**, so every spell that predates
        /// counting keeps its exact meaning — `has sage` *is* `has 1 sage`.
        ///
        /// **At least, never exactly.** A guard asks *"have I enough yet"*;
        /// `if the cabinet has 4 fragment` must stay true at five or the spell
        /// that assembles a scroll stops working the moment it gets ahead.
        ///
        /// The number was **silently swallowed** before this field existed:
        /// `has 4 fragment` parsed as `has fragment`, with no fault, and
        /// `interpret` showed the shorter question. That is §19's *"the orb
        /// writes down a shorter command than it heard"* arriving through the
        /// one surface built to catch it.
        ///
        /// **A [`Quantity`], so the other side can be the world too.** It was a
        /// bare `u32` and `Quantity::Count(1)` is exactly what that meant, so
        /// every spell written against the old shape asks the same question.
        count: Quantity,
        /// Which side of `count` satisfies it.
        ///
        /// **`AtLeast` with a count of nought is unrepresentable**, and that is
        /// enforced where the question is read rather than by the type: it
        /// collapses to `has no X`, because "at least nought" is satisfied by an
        /// empty shelf and is therefore a guard that always fires. `AtMost` with
        /// nought is a real comparison and means exactly what it says.
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
            // **Both places, and the compared-against one is a place.** It
            // resolves and fails exactly as the subject does — a comparison
            // against somewhere the tower lacks is §8's *Referent missing*, not
            // an answer of no.
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
    /// **Leaving it alone is the honest failure**, not substituting a guess: a
    /// name the room cannot place stays exactly as the player wrote it, so
    /// `holds` finds nothing and the runner reports the word they actually
    /// typed. See [`compile`](crate::tower::spell::compile).
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
    /// For completion, which must be able to *offer* the vocabulary and not only
    /// recognise it. All eight rather than the canonical three: §6's claim is
    /// that a register is not second class, and a player who writes `busy`
    /// should get the same help as one who writes `working`.
    ///
    /// Held beside [`read`](Self::read) rather than derived from it, because a
    /// `match` cannot be enumerated — and
    /// `every_state_word_the_list_offers_is_one_the_language_reads` is what
    /// stops the two drifting.
    pub const WORDS: [&'static str; 8] = [
        "idle", "free", "still", "working", "busy", "running", "empty", "bare",
    ];

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

/// A count moved by a strict comparator's offset.
///
/// **`more than 2` is *three or more***, and the type carries two directions
/// rather than four so the strictness lives here instead of in every place that
/// answers one. Counts are whole, so this loses nothing.
///
/// Clamped at each end, and the two ends clamp differently **because clamping
/// both to nought inverts one of them**. `fewer than 0` is unsatisfiable as
/// written and becomes *none*, which is at least a question the world can answer.
/// `more than u32::MAX` is unsatisfiable too — but clamping *that* to nought
/// makes it "at least nought", satisfied by an empty shelf, so a typo that should
/// never fire would always fire. It saturates instead.
fn shift(count: u32, offset: i64) -> u32 {
    let shifted = i64::from(count) + offset;
    u32::try_from(shifted).unwrap_or(if shifted < 0 { 0 } else { u32::MAX })
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
        // **The count is written only when it is not one, and `or more` is never
        // written at all.** `has sage`, `has 1 sage` and `has 1 or more sage` are
        // one question with three spellings; the bare one is canonical, so
        // quoting either of the others back at a player is the orb inventing a
        // notation — the thing this function exists not to do.
        //
        // `or fewer` **is** written, because nothing else says it. That
        // asymmetry is visible in `interpret` and is the point: it shows which
        // direction is the default.
        // **The comparative, written the way it was read.** One arm rather than
        // three, because `COMPARATIVES` already pairs each bound with the word
        // that closes it — so a new spelling is still a row in that table and
        // cannot arrive here without a way back.
        // **Guarded on `far()` rather than matching one variant**, so the four
        // world-read shapes share the arm that already knew how to write a
        // comparative. A variant added to `Quantity` with a `far()` gets written
        // back for free; one added without gets caught by the round-trip test
        // rather than by a player.
        Condition::Has {
            place,
            thing,
            count,
            bound,
        } if count.far().is_some() => {
            let other = count.far().unwrap_or_default();
            // **Falls back rather than panicking**, and the fallback is a
            // question the reader accepts: every `Bound` has a row today, and a
            // writer that could crash the game over a table someone extended
            // badly is a worse answer than one that writes the equality form.
            // `a_comparative_exists_for_every_bound` is what actually holds the
            // table complete, in a test rather than at a player's expense.
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
        // **Nought at-least keeps its words, and that is a round-trip fix.**
        // `has 0 or more X` is deliberately *not* collapsed by the reader — the
        // player asked for the vacuous question by name — but writing it as
        // `has 0 X` handed it back to the reader, which *does* collapse that to
        // `not has X`. So `interpret`, the one surface built to show a mis-read
        // question, showed the **negation** of what was typed.
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
        // **Words, not the symbol that may have been typed.** `=` and `exactly`
        // are one question, and the orb's fair copy is the spelling a player who
        // has never seen an operator can still read — which is the whole of why
        // symbols are accepted at the door and not kept.
        Condition::Has {
            place,
            thing,
            count: Quantity::Count(count),
            bound: Bound::Exactly,
        } => format!("{place} has exactly {count} {thing}"),
        // **Unreachable, and written rather than `unreachable!`.** The arms
        // above cover every `Count` bound and the guarded one covers everything
        // `far()` renders, so the only way here is a `Quantity` variant added
        // without a `far()` arm. The round-trip test is what catches that; a
        // panic here would catch it at a player's expense instead, which is the
        // trade the comparative arm's own fallback already makes.
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

/// What [`Reader::eat_comparative`] found.
///
/// # Three answers, and the third is why this is not an `Option`
///
/// A comparative that opens and never closes — `north has fewer marks`, with no
/// `than` — must **refuse the line**, not fall through to the count path. Falling
/// through hands `fewer` to the thing's name, and the fuzzy resolution in
/// `spell::compile` then drops it: the question silently becomes
/// `north has marks`, which answers yes wherever the player's answers no.
///
/// That is §19's *"the orb writes down a shorter command than it heard"* — the
/// defect counting was added to close — reappearing one grammar wider. Here the
/// line is kept exactly as typed and the orb says which line it could not read,
/// which is this module's whole posture.
enum Comparative {
    /// No comparative word here; the words ahead are something else.
    Absent,
    /// `fewer marks than east` — the bound, the thing, and what to compare
    /// against.
    ///
    /// **The third field was a place name and is now the whole far side.** It
    /// had to become a [`Quantity`] when that side stopped being a bare name:
    /// `double the enemy has mettle plus 6` is a tree, and flattening it back to
    /// a string here would mean parsing it twice.
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
    /// **Only ever called on the `has` side.** `is 4 idle` is not a sentence,
    /// and letting a number through there would make `State::read` the thing
    /// that refused it, one layer too late to say why.
    ///
    /// A number too large for a `u32` is **not** a count and is left alone, so
    /// it falls through to [`span`](Self::span) and becomes part of the thing's
    /// name — a question about something called `99999999999999 sage`, which
    /// nothing is, so `compile` reports that name as one it cannot place and
    /// §8.1 gets its culprit. Saturating to `u32::MAX` instead would turn a
    /// typo into a guard that silently never fires.
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
    /// Natural to type and invisible to a token-at-a-time reader, since the
    /// whole thing arrives as one word. Longest operator first, so `>=2` is not
    /// read as `>` against `=2`.
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
    /// # Every spelling, because the ones it refused were swallowed
    ///
    /// `has 2 marks`, `has at least 2 marks`, `has 2 or more marks`,
    /// `has more than 1 mark`, `has >= 2 marks`, `has >=2 marks`,
    /// `has exactly 2 marks` — one question, seven ways of asking it. Before
    /// this only the first two existed and the rest **vanished silently**:
    /// `at` is §6 filler, so `at least 2 marks` lost its `at` and the remainder
    /// resolved down to `marks` with no fault raised. That is the same swallow
    /// counting was added to close, left open for the spelling nobody tried.
    ///
    /// Returns whether a bound was **written**, which the caller needs: a bare
    /// `has 0 X` collapses to `has no X`, and an explicit `has 0 or more X` must
    /// not, because the player asked for the vacuous question by name.
    ///
    /// # `or` is why a postfix is read at all
    ///
    /// `or` joins two halves of a question and [`span`](Self::span) stops dead
    /// at it, so `has 2 or more marks` ate the `2`, found an empty span and
    /// refused the line. The `or` has to be claimed before the disjunction
    /// parser sees it, and this is the only place that can happen:
    /// `disjunction` → `conjunction` → `comparison` → here, with it unconsumed.
    /// **Whole row or nothing**, so `has 2 sage or the mortar is idle` is still
    /// the disjunction it reads as.
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
    /// Read as a **whole phrase or not at all**, because half of one is a
    /// question with a different meaning rather than a question with a missing
    /// word: `north has more marks` without the `than` is a shelf holding
    /// something called *"more marks"*, which is nothing, and the all-or-nothing
    /// rule would then refuse the line and say so. Putting the cursor back is
    /// what lets that happen instead of a partial read.
    ///
    /// Returns the bound, the thing, and where to compare against — the thing
    /// is read **here** rather than by the caller, because the comparative sits
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
        // **Nothing between the comparative and its closer is the *number*
        // form**, not a comparison missing its subject: `more than 1 fragment`
        // is a spelling [`BOUNDS`] has always read, and it shares its first word
        // with `more marks than east`. The gap in the middle is the whole
        // discriminator, and getting it wrong refused two rows of the table that
        // has held every spelling since counting arrived.
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
    /// **Left to right, and it cannot contain a comparison**, which is what
    /// makes the absence of brackets and precedence a consequence rather than a
    /// rule. `double` binds to the whole of what follows it, because there is
    /// only ever one thing following it.
    ///
    /// **The place reads to a [`STOPPERS`] word now, not to the next
    /// connective.** That is the change the operators needed: `to_connective`
    /// took everything up to `and`/`or`, so `than the d20 has quintessence`
    /// yielded a place literally called *"d20 has quintessence"* — the silent
    /// swallow the stopper list exists to prevent, one grammar wider again.
    /// Multi-word names still work; `balneum mariae` contains no stopper.
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

        // **`plus` last, so it applies to the whole of what came before it.**
        // `double the enemy plus 2` is `(2 x enemy) + 2` — left to right, which
        // is the only reading available when there is nothing to bracket.
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
        // **Tried before the count**, and the two cannot collide: a comparative
        // opens with a word and `eat_bounded_count` needs a digit, so whichever
        // fails puts the cursor back untouched. This one first only because it
        // reads the thing itself, which the count path leaves to `span`.
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

        // **After the negation and before the span**, which is the only place it
        // can go: `span` stops at a connective or a question word and would
        // otherwise take the digit as the first word of the thing's name — which
        // is exactly what it did before counting existed, giving `has 4 fragment`
        // a thing called *"4 fragment"* that nothing is ever called.
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

        // **`has 0 sage` is `has no sage` with a digit**, and saying so here is
        // what stops it being a branch that always fires: "at least nought" is
        // satisfied by an empty shelf, so read literally it is the vacuous truth
        // a player least expects from a guard they wrote.
        //
        // **Only when no bound was written.** `has 0 or fewer sage` and
        // `has 0 or more sage` are real comparisons — the first asks for an empty
        // shelf and says so, the second asks the vacuous question by name — and
        // collapsing either would make the explicit spelling pointless.
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

    /// Every state word the completer offers is one the language reads.
    ///
    /// [`State::WORDS`] is held beside [`State::read`] because a `match` cannot
    /// be enumerated, and two lists of the same vocabulary is the shape §19
    /// records going wrong three times over. This is what stops them drifting —
    /// in both directions, since a word added to the `match` and not the list is
    /// a spelling the editor silently refuses to help with.
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
