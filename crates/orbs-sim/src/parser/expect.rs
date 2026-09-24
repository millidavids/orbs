//! What may come next here — one answer, three surfaces.
//!
//! The prompt's Tab listing, the prompt's ghost and the spell editor's Tab all
//! ask the same question: *given this line and this caret, what could go there?*
//!
//! One function, not three: §19 records that shape going wrong three times, each
//! a second opinion about something the sim already knew. It is load-bearing
//! inside this file too — `verb_of` goes through the real
//! [`match_phrase`](super::resolve::match_phrase) because a private single-word
//! scan disagreed with `resolve` twice.
//!
//! The scribing guide is a fourth surface and is not this. It asks *what should
//! I teach you*, so it lists canonical names through
//! [`spell_vocabulary`](crate::spell_vocabulary), where this offers all three of
//! §6's registers and Tab completing `l` to `ls` and `look` is the point. They
//! share everything that could drift: `may_issue`/`is_live` (see [`spell_may`]),
//! `Scene::offers`, and [`SpellWord::shape`].
//!
//! It does not answer whether a word is any good, only what the grammar permits,
//! in a stable order — the language's own words as taught, verbs alphabetical,
//! nouns in scene order. Permitted is a real filter and the half that took the
//! work: `until` never opens a line, `end` needs something open and `else` needs
//! an `if` directly above, so [`control`] takes the block stack rather than the
//! partial word. Offering a word the orb then refuses is §15's dead end.

use core::ops::Range;

use orbs_render::{Lexeme, char_index};

use super::question::State as SpellState;
use super::scene::Scene;
use super::spellword::SpellWord;
use super::verb::{NounKind, Verb};
use super::vocabulary::SYNONYMS;

/// Where the caret is standing.
///
/// `spell` is the editor rather than the prompt, and not cosmetic: the language
/// has nine words of its own the prompt cannot run, and the prompt has §6's
/// numbered answer a spell never sees.
#[derive(Debug, Clone, Copy)]
pub struct Situation<'a> {
    /// The live vocabulary — for a spell, the domain the file belongs to, which
    /// is not always the room the player is standing in.
    pub scene: &'a Scene,
    /// Whether this line is a line of a spell rather than a command.
    pub spell: bool,
    /// Whether §6's numbered prompt is waiting for a digit.
    ///
    /// Turns the answer off entirely: the orb wants a number, and offering a
    /// verb there — or ghosting one — is §15's dead end. It lives in the
    /// situation rather than in callers, or three of the four surfaces would
    /// each have to remember to.
    pub prompt_open: bool,
    /// The blocks open above this line, innermost last.
    ///
    /// From [`open_blocks`]. Empty at the prompt, which has no lines above it
    /// and cannot run a block anyway.
    pub open: &'a [SpellWord],
    /// The sets `for each` may walk here — `way` in the archive, `socket` and
    /// `sigil` in the lens.
    ///
    /// A field because it is not derivable from the [`Scene`]: a set is declared
    /// by the fixture and read by `tower::groups_at`, and `scene_at` never
    /// registers one as a `Noun`. Answering `for each ` from the scene offers
    /// the room's *contents* — `stacks`, `cabinet`, `north` — a list where every
    /// entry is wrong and every right answer missing, contradicting `recall
    /// scripting`, the only place a player learns sets exist.
    ///
    /// Empty at the prompt, which cannot run `for`.
    pub sets: &'a [String],
}

/// Why something is being offered.
///
/// Not [`Lexeme`] under another name, though for one version it was. `is`,
/// `has`, `be` and `each` are fixed in position by the grammar — as much
/// scaffolding as `if` — yet none is a [`Lexeme::Control`], because `lex`
/// reserves that for [`SpellWord`] and a completer must not disagree with the
/// painter about how a word is drawn.
///
/// Two facts, then, with [`kind`](Self::kind) deriving one from the other here
/// rather than at each caller — `complete`'s header records what two fields per
/// construction site cost last time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// One of the language's own words — `repeat`, `if`, `end`.
    Word,
    /// A verb the tower answers to.
    Verb,
    /// Something the tower has — a reagent, an instrument, a place.
    Thing,
    /// A word the grammar fixes in this position — `is`, `has`, `be`, `each`.
    Grammar,
    /// One of the three states a place reports, or a spelling of one.
    State,
    /// A reading a place publishes.
    Reading,
}

impl Reason {
    /// How a surface should draw it — the painter's answer, not a second one.
    #[must_use]
    pub const fn kind(self) -> Lexeme {
        match self {
            Self::Word => Lexeme::Control,
            Self::Verb => Lexeme::Verb,
            // A grammar word, a state and a reading are all ordinary words to
            // `lex`, which classifies only what it can see on the line.
            Self::Thing | Self::Grammar | Self::State | Self::Reading => Lexeme::Name,
        }
    }
}

/// One thing that could go where the caret is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expected {
    /// What a list shows and what a Tab inserts, which are the same string.
    pub text: String,
    /// Why it is here — so a surface can group it, head it, or explain it
    /// without asking the grammar a second time.
    pub why: Reason,
    /// What follows it, for a surface with room to say — `<count>`, `place`.
    ///
    /// Empty where nothing follows, which is most nouns and two of the nine
    /// control words. The *signature*, not [`text`](Self::text) twice — it came
    /// from a table `orbs-shell` was keeping in parallel with the grammar.
    pub shape: &'static str,
}

impl Expected {
    /// How to draw it. See [`Reason::kind`].
    #[must_use]
    pub const fn kind(&self) -> Lexeme {
        self.why.kind()
    }

    /// A candidate with nothing following it, which is most of them.
    fn plain(text: impl Into<String>, why: Reason) -> Self {
        Self {
            text: text.into(),
            why,
            shape: "",
        }
    }
}

/// Everything that could go where the caret is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Expectation {
    /// The byte range of the line this replaces — the partial word.
    ///
    /// A range rather than an append, because §6's parser is deliberately fuzzy:
    /// a completer that can only *extend* what was typed is useless on a
    /// near-miss, and a near-miss is what this game expects.
    pub replaces: Range<usize>,
    /// The candidates, in a stable order — verbs alphabetical, nouns in scene
    /// order, control words in the language's own order.
    pub expected: Vec<Expected>,
}

impl Expectation {
    /// Whether there is anything to offer.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.expected.is_empty()
    }

    /// Just the strings, for a caller that wants nothing else.
    #[must_use]
    pub fn texts(&self) -> Vec<String> {
        self.expected.iter().map(|one| one.text.clone()).collect()
    }

    /// The text every candidate agrees on.
    ///
    /// GNU readline's `compute_lcd_of_matches`: Tab advances to the longest
    /// common prefix and lists only when that adds nothing.
    ///
    /// Two surfaces read this and must agree — Tab spends the prefix and the
    /// prompt's ghost draws exactly what Tab would take, so a second
    /// implementation would have the ghost promising what Tab did not do.
    #[must_use]
    pub fn common(&self) -> String {
        let mut rest = self.expected.iter();
        let Some(first) = rest.next() else {
            return String::new();
        };
        let mut shared: Vec<char> = first.text.chars().collect();
        for other in rest {
            let keep = shared
                .iter()
                .zip(other.text.chars())
                .take_while(|(a, b)| **a == *b)
                .count();
            shared.truncate(keep);
        }
        shared.into_iter().collect()
    }
}

/// What could go at `caret`.
#[must_use]
pub fn expect(line: &str, caret: usize, at: &Situation<'_>) -> Expectation {
    if at.prompt_open {
        return Expectation::default();
    }

    let upto = char_index(line, caret);
    let word = word_start(line, upto);
    let partial = &line[word..upto];
    let replaces = word..upto;

    // The first phrase is a verb — or, in a spell, one of the language's own
    // words. Anything later fills a slot of whichever it was.
    let leading = line[..word].trim();
    let expected = if leading.is_empty() {
        let mut out = if at.spell {
            control(partial, at.open)
        } else {
            Vec::new()
        };
        out.extend(verbs(partial, at.scene, at.spell));
        out
    } else if let Some(word) = at.spell.then(|| opening(leading)).flatten() {
        // A line opening with one of the language's own words is the language's
        // to finish, not a verb's. Falling through to `verb_of` left `if the
        // mortar_and_pestle is ` offering nothing at all.
        after(word, leading, partial, at)
    } else {
        match verb_of(leading, at.scene) {
            Some((verb, filled)) => nouns(verb, filled, partial, at.scene),
            None => Vec::new(),
        }
    };

    Expectation { replaces, expected }
}

/// The language's own word this line opens with, if it opens with one.
fn opening(leading: &str) -> Option<SpellWord> {
    let first = leading.split_whitespace().next()?;
    SpellWord::ALL
        .into_iter()
        .find(|word| word.canonical() == first.to_lowercase())
}

/// What the blocks open at this line allow to be written next.
///
/// A stack, not a count: `else` is legal only directly inside an `if`, and a
/// count cannot tell an `if` inside a `repeat` from a `repeat` inside an `if`.
/// `end` closes whatever is innermost, so it wants only a non-empty stack.
///
/// A bare `else` neither opens nor closes and still changes the answer, being
/// the *last* branch of its ladder — a second is `spell_stray_else`. Tracking
/// only opens and closes left the `if` on the stack after its else-branch had
/// started, so the guide went on offering `else`.
///
/// `else if` chains, which is why the two are told apart here rather than by the
/// word alone: `if A / else if B / else / end` is valid, and treating its middle
/// rung as final would forbid the spelling that saved `threading` forty-six
/// lines.
#[must_use]
pub fn open_blocks(before: &[String]) -> Vec<SpellWord> {
    let mut open = Vec::new();
    for line in before {
        let trimmed = line.trim();
        let Some(word) = opening(trimmed) else {
            continue;
        };
        match word {
            SpellWord::End => {
                open.pop();
            }
            SpellWord::Else if !chains(trimmed) => {
                // The ladder is finished. Keep the frame — an `end` still has to
                // close it — but stop calling it an `if`.
                if open.last() == Some(&SpellWord::If) {
                    open.pop();
                    open.push(SpellWord::Else);
                }
            }
            other if other.opens_block() => open.push(other),
            _ => {}
        }
    }
    open
}

/// Whether an `else` line carries an `if` — `else if <question>`.
fn chains(line: &str) -> bool {
    super::spellword::argument(line)
        .split_whitespace()
        .next()
        .is_some_and(|word| word.eq_ignore_ascii_case(SpellWord::If.canonical()))
}

/// The language's own words that may open a line **here**.
///
/// Not sorted: `wait`, `repeat`, `if`, `else`, `end` is the order they are
/// taught in and appear in; alphabetising puts `else` before `if`, which reads
/// as nonsense to somebody learning the shape.
///
/// Three of the nine are illegal at a line start, and listing all nine offered
/// three ways to make the orb refuse the line it had just suggested:
///
/// - `until` never opens a line — it is `repeat`'s guard, written on `repeat`'s
///   own line, and alone it is `spell_stray_until`.
/// - `end` with nothing to close is `spell_stray_end`.
/// - `else` needs an `if` *directly* above, which is why [`open_blocks`] keeps a
///   stack rather than a depth.
fn control(partial: &str, open: &[SpellWord]) -> Vec<Expected> {
    SpellWord::ALL
        .iter()
        .filter(|word| legal_here(**word, open))
        .filter(|word| word.canonical().starts_with(partial))
        .map(|word| Expected {
            text: word.canonical().to_owned(),
            why: Reason::Word,
            shape: word.shape(),
        })
        .collect()
}

/// Whether `word` may open a line with these blocks open. See [`control`].
fn legal_here(word: SpellWord, open: &[SpellWord]) -> bool {
    match word {
        SpellWord::Until => false,
        SpellWord::End => !open.is_empty(),
        SpellWord::Else => open.last() == Some(&SpellWord::If),
        _ => true,
    }
}

/// What may follow the language's own word, once the line has opened with it.
///
/// `wait` takes a thing, not a state, though the feature was specified the other
/// way. `program.rs` stores `Kind::Wait(<name>)` and `run.rs` resolves it by
/// scanning the record stream for an event naming that thing; no state is read,
/// and `be` is not filler — so `wait for the mortar_and_pestle to be idle` waits
/// on a thing called `mortar_and_pestle be idle`, matches nothing, burns
/// `PATIENCE` and latches a fault on the rail.
///
/// The state set belongs to `is`. Offering it after `wait` would break this
/// function's own rule: never teach a line the runner refuses.
fn after(word: SpellWord, leading: &str, partial: &str, at: &Situation<'_>) -> Vec<Expected> {
    let scene = at.scene;
    let all: Vec<String> = leading
        .split_whitespace()
        .skip(1)
        .map(str::to_lowercase)
        .collect();
    let rest: Vec<&str> = all
        .iter()
        .map(String::as_str)
        .filter(|word| !super::is_filler(word))
        .collect();

    let offered = match word {
        // A count is a number and completes to nothing, so the only word here is
        // the guard — `repeat until <question>`, the bound a loop should have.
        SpellWord::Repeat => match rest.split_first() {
            None => vec![Expected::plain("until", Reason::Grammar)],
            Some((&"until", asked)) => question(asked, scene),
            Some(_) => Vec::new(),
        },
        SpellWord::If | SpellWord::Until => question(&rest, scene),
        // A count and nothing else: no guard, no second form. `bide` is one
        // number, which is what makes it readable arithmetic on the page.
        SpellWord::Bide => Vec::new(),
        // `else if` chains, and a bare `else` takes nothing.
        SpellWord::Else => match rest.split_first() {
            None => vec![Expected::plain("if", Reason::Word)],
            Some((&"if", asked)) => question(asked, scene),
            Some(_) => Vec::new(),
        },
        // A thing, and never a state. See this function's header.
        SpellWord::Wait => things(scene),
        // `let <name> be <place>` — the name is the player's to coin, so the
        // only thing to offer at that position is nothing.
        SpellWord::Let => match rest.len() {
            0 => Vec::new(),
            1 => vec![Expected::plain("be", Reason::Grammar)],
            _ => places(scene),
        },
        // `pull <name> from <place>` — `let`'s shape, name coined the same way,
        // so nothing to offer there either.
        //
        // `from` is filler: `rest` has already dropped it, so one word in hand
        // puts the caret at the place, and offering `from` would offer a word
        // the parser throws away. `let`'s `be` survives normalisation and
        // carries the grammar, so it is offered.
        SpellWord::Pull => match rest.len() {
            0 => Vec::new(),
            _ => places(scene),
        },
        // Nothing, like `part`. What follows is a call naming a part *this file*
        // defines, which the scene knows nothing about; the editor's guide shows
        // the shape (`<name>(...)`), and the room's nouns would be a list with
        // no right answer in it — `for each`'s mistake, below.
        SpellWord::Alongside => Vec::new(),
        // The sets the fixture declares — `way`, `socket`, `sigil` — never the
        // room's contents. Offering `stacks`, `cabinet`, `north` was a list with
        // no right answer in it, contradicting `recall scripting`, the one page
        // that teaches sets exist.
        SpellWord::For => match rest.split_first() {
            None => vec![Expected::plain("each", Reason::Grammar)],
            Some((&"each", _)) => at
                .sets
                .iter()
                .map(|set| Expected::plain(set.as_str(), Reason::Thing))
                .collect(),
            Some(_) => Vec::new(),
        },
        // A part is named by whoever writes it, and `end` closes a block.
        SpellWord::Part | SpellWord::End => Vec::new(),
    };

    starting(offered, partial)
}

/// What may come next inside a question, given the words of it so far.
///
/// The grammar is `<place> is <state>` or `<place> has [no] [<count>] <thing>`,
/// and [`STOPPERS`](super::question) fixes `is` and `has` as the two hinges.
/// Where a comparative's far side begins, if the line has reached one.
///
/// The opener is found first and its closer after it. Searching for the closers
/// alone matched the *opening* `as` of `as many … as`, so `has as many ` was
/// answered with the far side's own words until the comparative closed — wrong
/// on all three surfaces at once, and the test that shipped with it only
/// exercised `than`.
///
/// The gap in the middle is the other discriminator, the same one
/// `question::eat_comparative` uses: nothing sits between the comparative and
/// its closer in the *number* form `more than 1 fragment`, where `more marks
/// than east` compares against a place.
fn far_side_begins(asked: &[&str]) -> Option<usize> {
    let mut at = 0;
    while at < asked.len() {
        let (width, closer) = match asked[at] {
            "more" | "fewer" | "less" => (1, "than"),
            "as" if asked
                .get(at + 1)
                .is_some_and(|word| matches!(*word, "many" | "much")) =>
            {
                (2, "as")
            }
            _ => {
                at += 1;
                continue;
            }
        };
        let after = at + width;
        let found = asked[after..].iter().position(|word| *word == closer)?;
        // Nothing between the two is `more than 1 fragment`, which is a count
        // and not a far side at all.
        return (found > 0).then_some(after + found + 1);
    }
    None
}

fn question(asked: &[&str], scene: &Scene) -> Vec<Expected> {
    // A comparative's far side is its own little question: after `than` the
    // words belong to it, not the near side. Without this the hinge below found
    // the *first* `has` and kept offering the near side's things through `than
    // the d20 has …`, where the answer is certainly a reading over there.
    if let Some(begins) = far_side_begins(asked) {
        let far = &asked[begins..];
        // Straight after the closer: a place, or the one word that scales one.
        if far.is_empty() {
            let mut out = vec![Expected::plain("double", Reason::Grammar)];
            out.extend(places(scene));
            return out;
        }
        // The far side's own `has`, then what it may ask for there.
        if let Some(at) = far.iter().position(|word| *word == "has") {
            return if far.len() == at + 1 {
                let mut out = things(scene);
                out.extend(readings(scene));
                out
            } else {
                vec![Expected::plain("plus", Reason::Grammar)]
            };
        }
        // A place is named and nothing else yet: the two words that may follow.
        return vec![
            Expected::plain("has", Reason::Grammar),
            Expected::plain("plus", Reason::Grammar),
        ];
    }

    let hinge = asked.iter().position(|word| matches!(*word, "is" | "has"));
    let Some(at) = hinge else {
        // No hinge yet: either the place has not been named, or it has and the
        // question word is what is wanted.
        return if asked.is_empty() {
            places(scene)
        } else {
            vec![
                Expected::plain("is", Reason::Grammar),
                Expected::plain("has", Reason::Grammar),
            ]
        };
    };

    match asked[at] {
        // A closed vocabulary of eight — the answer the feature was asked for,
        // at `is`, where it belongs.
        "is" => SpellState::WORDS
            .iter()
            .map(|word| Expected::plain(*word, Reason::State))
            .collect(),
        // `has` takes an optional `no`, an optional count, and a thing or a
        // reading. A count is a number and completes to nothing.
        //
        // Things first, readings after: `Errand::ALL` and the maze's senses
        // chain onto every scene, so a laboratory spell asking `has ` was
        // offered `passage`, `wall` and `spoil` above its own reagents.
        _ => {
            let mut out = Vec::new();
            if asked.len() == at + 1 {
                out.push(Expected::plain("no", Reason::Grammar));
            }
            out.extend(things(scene));
            out.extend(readings(scene));
            out
        }
    }
}

/// The places a question may name.
fn places(scene: &Scene) -> Vec<Expected> {
    of_kind(scene, NounKind::Place, Reason::Thing)
}

/// The readings a place publishes — what `has` asks about.
fn readings(scene: &Scene) -> Vec<Expected> {
    of_kind(scene, NounKind::Sense, Reason::Reading)
}

/// Everything the tower *has* that a spell can name.
///
/// `Command` is excluded. `scene_at` registers every one-word verb synonym as a
/// `NounKind::Command` — about a hundred: `spy`, `peek`, `try`, `light`, `walk`
/// — so the fuzzy matcher cannot turn a word the game knows into a noun. None is
/// a thing, yet `if the cabinet has ` listed the lot, and `wait ` offered one,
/// which scans the record stream for something that never arrives, burns
/// `PATIENCE` and latches a fault on the rail.
///
/// `Sense` is excluded too: a reading is offered by [`readings`], after these,
/// because the scene's things are the room's and its readings are not.
fn things(scene: &Scene) -> Vec<Expected> {
    scene
        .nouns()
        .iter()
        .filter(|noun| !matches!(noun.kind, NounKind::Sense | NounKind::Command))
        .map(|noun| Expected::plain(super::intent::leaf(&noun.name), Reason::Thing))
        .collect()
}

/// The scene's nouns of one kind, in scene order.
fn of_kind(scene: &Scene, kind: NounKind, why: Reason) -> Vec<Expected> {
    scene
        .nouns()
        .iter()
        .filter(|noun| noun.kind == kind)
        .map(|noun| Expected::plain(super::intent::leaf(&noun.name), why))
        .collect()
}

/// Keep only what the player has already started typing.
///
/// Applied once at the end rather than inside each arm above, so a new arm
/// cannot forget it — which is how a completer starts offering `idle` to
/// somebody who has typed `bu`.
fn starting(offered: Vec<Expected>, partial: &str) -> Vec<Expected> {
    if partial.is_empty() {
        return offered;
    }
    offered
        .into_iter()
        .filter(|one| one.text.starts_with(partial))
        .collect()
}

/// Every phrase that could be the verb being typed.
///
/// Canonical names *and* synonyms, including the multi-word ones: §6 claims all
/// three registers reach the same command, and a `words.len() == 1` filter here
/// made `go to`, `look for`, `get rid of`, `what's here`, `how do i` and `take
/// it back` invisible to Tab.
///
/// Two filters in a spell, not one. `Scene::offers` asks whether the verb's
/// fixture stands in this domain; [`may_issue`](crate::tower::spell::may_issue)
/// asks whether a spell may issue it *at all*. `attend`, `meditate`, `scribe`,
/// `unfurl`, `weave` and `wander` pass the first and fail the second, so without
/// it the editor offers six words the runner refuses — §15's dead end, taught
/// deliberately.
fn verbs(partial: &str, scene: &Scene, spell: bool) -> Vec<Expected> {
    let mut out: Vec<(String, Verb)> = SYNONYMS
        .iter()
        // Tab must not offer a word the parser would refuse. A per-instrument
        // verb out of its domain is exactly that — see `Scene::offers`.
        .filter(|entry| scene.offers(entry.verb))
        .filter(|entry| !spell || spell_may(entry.verb))
        .map(|entry| (entry.words.join(" "), entry.verb))
        .filter(|(phrase, _)| phrase.starts_with(partial))
        .collect();
    out.sort();
    out.dedup();
    out.into_iter()
        .map(|(text, verb)| Expected {
            text,
            why: Reason::Verb,
            shape: verb.signature_label(),
        })
        .collect()
}

/// Whether a spell may issue this verb at all.
///
/// The same pair [`spell_vocabulary`](crate::spell_vocabulary) applies, and the
/// two must not drift: that builds the guide's teaching list, this the editor's
/// completion, so a word Tab offers and the guide never explains is the shape
/// §19 records going wrong three times.
///
/// They differ in one thing, deliberately: this iterates the synonym table so
/// all three of §6's registers complete, where the guide lists canonical names
/// only — `ls`, `look`, `go to` and `what's here` side by side is the overwhelm
/// it exists to prevent.
const fn spell_may(verb: Verb) -> bool {
    // `is_live` is *"nobody built this"* and `may_issue` is *"a spell may not"*.
    // Both are dead ends and a completer must offer neither.
    crate::execute::is_live(verb) && crate::tower::spell::may_issue(verb)
}

/// The verb a line has already named, and how many argument slots follow it.
///
/// Matched through [`match_phrase`](super::resolve::match_phrase) — the same
/// function `resolve` ranks with — rather than through
/// [`resolve`](mod@super::resolve) itself: a bare `wield ` has no argument yet, so a
/// full resolution comes back `Incomplete` and yields no intent, exactly when
/// completion is most wanted.
///
/// The real matcher is what keeps Tab and Enter agreeing. A private single-word
/// scan disagreed twice: multi-word phrases were invisible, so `look for `
/// offered places for `Survey` while the line resolves to `sift`, whose slot is
/// free text and must offer nothing; and its `max_by_key` took the *last*
/// maximum where `resolve` applies `named_exactly` → score → `verb_order`, so a
/// tie offered one verb's arguments and ran another's.
fn verb_of(leading: &str, scene: &Scene) -> Option<(Verb, usize)> {
    let split = super::normalise::Tokens::split(leading);
    let all = split.words();
    let words = &all[super::normalise::skip_leading_filler(&all)..];

    let (verb, consumed) = SYNONYMS
        .iter()
        .filter(|entry| scene.offers(entry.verb))
        .filter_map(|entry| {
            super::resolve::match_phrase(entry, words)
                .map(|(score, span)| (score, entry.verb, span))
        })
        .max_by(|a, b| {
            // Score, then the longer phrase, then table order. `match_phrase`
            // clamps its `PHRASE_BONUS` at `fuzzy::EXACT`, so a perfectly typed
            // `look for` scores what a perfectly typed `look` does, and without
            // the span tie-break the two-word reading loses to whichever came
            // first.
            a.0.cmp(&b.0)
                .then_with(|| a.2.cmp(&b.2))
                .then_with(|| verb_order(b.1).cmp(&verb_order(a.1)))
        })
        .map(|(_, verb, span)| (verb, span))?;

    // Filler between the verb and the caret is not a slot: `move sage to ` has
    // filled one, not two, and counting `to` would offer the third slot's nouns
    // for the second.
    let filled = super::normalise::strip_filler(&words[consumed..]).len();
    Some((verb, filled))
}

/// A verb's position in [`Verb::ALL`], mirroring `resolve`'s stable tie-break.
fn verb_order(verb: Verb) -> usize {
    Verb::ALL
        .iter()
        .position(|&other| other == verb)
        .unwrap_or(usize::MAX)
}

/// Every noun that fits the slot the caret is in.
fn nouns(verb: Verb, filled: usize, partial: &str, scene: &Scene) -> Vec<Expected> {
    let signature = verb.signature();
    let Some(slot) = signature.get(filled).or_else(|| signature.last()) else {
        return Vec::new();
    };

    // Free text completes nothing: a pattern is whatever the player is searching
    // for and a count is a number, so the scene's nouns would lie about what the
    // slot accepts.
    if matches!(slot.kind, NounKind::Pattern | NounKind::Count) {
        return Vec::new();
    }

    // `Name` is the exception among the free-text kinds. It cannot *resolve*
    // against the scene — a spell being coined does not exist — but the
    // commonest `scribe` reopens one that does, so completion offers the spells
    // that exist while resolution still accepts anything typed.
    let offered = if slot.kind == NounKind::Name {
        NounKind::Script
    } else {
        slot.kind
    };

    scene
        .nouns()
        .iter()
        .filter(|noun| offered.accepts(noun.kind))
        .filter_map(|noun| {
            // A place is shown and typed as its leaf (§7: players say the place,
            // not the path) — through the same helper the echo uses, so Tab
            // inserts exactly the form the echo will show back.
            let leaf = super::intent::leaf(&noun.name);
            leaf.starts_with(partial)
                .then(|| Expected::plain(leaf, Reason::Thing))
        })
        .collect()
}

/// The byte index where the word ending at `upto` begins.
fn word_start(line: &str, upto: usize) -> usize {
    line[..upto].rfind(char::is_whitespace).map_or(0, |at| {
        at + line[at..].chars().next().map_or(1, char::len_utf8)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tower() -> Scene {
        Scene::new()
            .with(NounKind::Place, "/tower/laboratory")
            .with(NounKind::Place, "/tower/laboratory/mortar_and_pestle")
            .with(NounKind::Reagent, "sage")
            .with(NounKind::Sense, "clarity")
    }

    /// A line of a spell, with nothing open above it.
    fn spelling(line: &str) -> Expectation {
        inside(line, &[])
    }

    /// ...and one written inside the blocks named.
    fn inside(line: &str, open: &[SpellWord]) -> Expectation {
        let scene = tower();
        let sets = [String::from("way")];
        expect(
            line,
            line.chars().count(),
            &Situation {
                scene: &scene,
                spell: true,
                prompt_open: false,
                open,
                sets: &sets,
            },
        )
    }

    fn typing(line: &str) -> Expectation {
        let scene = tower();
        expect(
            line,
            line.chars().count(),
            &Situation {
                scene: &scene,
                spell: false,
                prompt_open: false,
                open: &[],
                sets: &[],
            },
        )
    }

    /// A spell line may open with one of the language's own words.
    ///
    /// The prompt cannot run them, so `complete` never offered them and the
    /// editor's Tab would have taught that `repeat` is not a word.
    #[test]
    fn a_spell_offers_the_languages_own_words_and_the_prompt_does_not() {
        let found = spelling("re").texts();
        assert!(found.contains(&"repeat".to_owned()), "{found:?}");

        let prompt = typing("re").texts();
        assert!(
            !prompt.contains(&"repeat".to_owned()),
            "the prompt offered a word it cannot run: {prompt:?}",
        );
    }

    /// ...and the verbs are still there beside them.
    ///
    /// A spell line is `repeat 3` *or* `grind sage`; offering only the
    /// scaffolding would be worse than offering only the verbs.
    #[test]
    fn a_spell_offers_verbs_as_well_as_words() {
        let found = spelling("su").texts();
        assert!(found.contains(&"survey".to_owned()), "{found:?}");
    }

    /// Control words come first, in the order they are taught — not
    /// alphabetical, which puts `else` before `if`.
    ///
    /// Six of the nine, at the top of a spell: `until` never opens a line, and
    /// `else` and `end` need something open above them.
    #[test]
    fn the_languages_words_lead_and_keep_their_own_order() {
        let found = spelling("").expected;
        let words: Vec<&str> = found
            .iter()
            .take_while(|one| one.why == Reason::Word)
            .map(|one| one.text.as_str())
            .collect();
        assert_eq!(
            words,
            // `SpellWord::ALL`'s order. A word added anywhere but the end
            // reorders what the editor's guide and the prompt's completion both
            // offer, and the two have agreed since the language had four words.
            [
                "wait",
                "repeat",
                "if",
                "let",
                "for",
                "part",
                "bide",
                "pull",
                "alongside",
            ],
            "the language's words reordered or stopped leading",
        );
        assert!(
            found.iter().any(|one| one.why == Reason::Verb),
            "the verbs went missing behind them",
        );
    }

    /// Three of the nine are illegal at a line start, and it depends where.
    /// Listing all nine gave three ways to make the orb refuse the line it had
    /// just suggested.
    #[test]
    fn a_word_that_cannot_open_a_line_here_is_not_offered() {
        // `until` is `repeat`'s guard, written on `repeat`'s own line. Alone it
        // is `spell_stray_until`, so it never opens a line anywhere.
        for open in [&[][..], &[SpellWord::Repeat][..], &[SpellWord::If][..]] {
            assert!(
                !inside("", open).texts().contains(&"until".to_owned()),
                "`until` was offered as a line opener with {open:?} open",
            );
        }

        // `end` needs something to close.
        assert!(!spelling("").texts().contains(&"end".to_owned()));
        assert!(
            inside("", &[SpellWord::Repeat])
                .texts()
                .contains(&"end".to_owned()),
        );

        // `else` needs an `if` directly inside one, which is why the stack is a
        // stack and not a depth.
        assert!(
            inside("", &[SpellWord::If])
                .texts()
                .contains(&"else".to_owned()),
        );
        assert!(
            !inside("", &[SpellWord::If, SpellWord::Repeat])
                .texts()
                .contains(&"else".to_owned()),
            "`else` was offered inside a `repeat` that is inside the `if`",
        );
    }

    /// The stack is read off the lines above, and `end` pops it.
    #[test]
    fn the_blocks_open_above_a_line_are_counted_from_it() {
        let lines: Vec<String> = ["repeat 3", "if the mortar_and_pestle is idle", "end"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert_eq!(open_blocks(&lines), [SpellWord::Repeat]);
        assert_eq!(open_blocks(&lines[..2]), [SpellWord::Repeat, SpellWord::If]);
        assert!(open_blocks(&[]).is_empty());
    }

    /// `is` offers the states and `wait` does not.
    ///
    /// The feature was specified as offering them after `wait … to be`, which
    /// does not work: `wait` stores a *thing* and resolves it by scanning the
    /// record stream, so `wait for the mortar_and_pestle to be idle` waits on a
    /// thing called `mortar_and_pestle be idle`, matches nothing, and burns
    /// `PATIENCE` into a fault on the rail.
    #[test]
    fn the_states_belong_to_is_and_not_to_wait() {
        let states = spelling("if the mortar_and_pestle is ").texts();
        assert_eq!(
            states,
            [
                "idle", "free", "still", "working", "busy", "running", "empty", "bare"
            ],
            "the state vocabulary moved",
        );

        let waiting = spelling("wait for the mortar_and_pestle to be ").texts();
        assert!(
            !waiting.iter().any(|word| word == "idle"),
            "`wait` was offered a state it cannot use: {waiting:?}",
        );
    }

    /// The far side is its own little question, and the hinge is the last one.
    ///
    /// `question` found the *first* `is`/`has`, so everything after `than … has`
    /// was answered against the near side — the room's things in the one
    /// position where the answer is certainly a reading over there, and wrong on
    /// all three surfaces at once.
    #[test]
    fn the_far_side_of_a_comparison_completes_as_its_own_question() {
        // Straight after the closer: somewhere to compare against, or the word
        // that doubles one.
        let far = spelling("if the mortar_and_pestle has fewer sage than ").texts();
        assert!(far.contains(&"double".to_owned()), "{far:?}");
        assert!(
            far.contains(&"mortar_and_pestle".to_owned()),
            "the far side offered no place: {far:?}",
        );

        // A place is named. The trailing space is the convention that turns
        // *explain this word* into *what may come next*, as it does everywhere.
        let after =
            spelling("if the mortar_and_pestle has fewer sage than the laboratory ").texts();
        assert_eq!(
            after,
            vec!["has".to_owned(), "plus".to_owned()],
            "the far side offered the wrong continuation",
        );

        // ...and after the far side's own `has`, what may be asked *there*.
        let reading =
            spelling("if the mortar_and_pestle has fewer sage than the laboratory has ").texts();
        assert!(
            reading.contains(&"clarity".to_owned()),
            "the far side's `has` did not offer a reading: {reading:?}",
        );

        // Named, so the only thing left is the operator.
        let operator =
            spelling("if the mortar_and_pestle has fewer sage than the laboratory has clarity ")
                .texts();
        assert_eq!(operator, vec!["plus".to_owned()]);

        // `double` changes none of that, which makes it a prefix rather than a
        // fourth state to thread through.
        let doubled =
            spelling("if the mortar_and_pestle has fewer sage than double the laboratory ").texts();
        assert_eq!(doubled, vec!["has".to_owned(), "plus".to_owned()]);

        // `as many … as` opens with the word that closes it, and searching
        // backwards for the closer made the near side's own things unreachable
        // from here until the closing `as`.
        let opening = spelling("if the mortar_and_pestle has as many ").texts();
        assert!(
            opening.contains(&"sage".to_owned()),
            "the opening `as` was read as a closer: {opening:?}",
        );
        assert!(
            !opening.contains(&"plus".to_owned()),
            "the far side answered a line still on the near one: {opening:?}",
        );
        // ...and once it really is closed, the far side answers.
        let closed = spelling("if the mortar_and_pestle has as many sage as ").texts();
        assert!(closed.contains(&"double".to_owned()), "{closed:?}");

        // The number form is not a far side: nothing sits between the
        // comparative and its closer in `more than 2 …`, the spelling `BOUNDS`
        // has read since counting arrived.
        let counted = spelling("if the mortar_and_pestle has more than 2 ").texts();
        assert!(
            counted.contains(&"sage".to_owned()),
            "a count was read as a comparison against a place: {counted:?}",
        );
    }

    /// A question is walked one hinge at a time.
    #[test]
    fn a_question_offers_a_place_then_a_hinge_then_an_answer() {
        // Nothing named yet: the places.
        let places = spelling("if ").texts();
        assert!(
            places.contains(&"mortar_and_pestle".to_owned()),
            "{places:?}"
        );
        assert!(
            !places.contains(&"sage".to_owned()),
            "a reagent is not a place to ask about: {places:?}",
        );

        // Named: the two words that hinge a question.
        assert_eq!(spelling("if the mortar_and_pestle ").texts(), ["is", "has"]);

        // ...and a partial narrows it, through the one filter every arm shares.
        assert_eq!(spelling("if the mortar_and_pestle i").texts(), ["is"]);

        // `has` asks about a reading or a thing, and takes an optional `no`.
        let has = spelling("if the mortar_and_pestle has ").texts();
        assert_eq!(has.first().map(String::as_str), Some("no"));
        assert!(has.contains(&"sage".to_owned()), "{has:?}");
    }

    /// `repeat` offers its guard, and `else` chains into another `if`.
    #[test]
    fn a_word_that_takes_another_word_offers_it() {
        assert_eq!(spelling("repeat ").texts(), ["until"]);
        assert_eq!(inside("else ", &[SpellWord::If]).texts(), ["if"]);
        assert_eq!(spelling("for ").texts(), ["each"]);
        assert_eq!(spelling("let mine ").texts(), ["be"]);

        // `repeat until ` is a question like any other.
        let asked = spelling("repeat until ").texts();
        assert!(asked.contains(&"mortar_and_pestle".to_owned()), "{asked:?}");

        // A part is named by whoever writes it; there is nothing to offer.
        assert!(spelling("part ").is_empty());
    }

    /// `for each ` walks a set, and the room's contents are not sets.
    ///
    /// A set is declared by the fixture and read by `groups_at`; `scene_at`
    /// never registers one as a noun. Answering from the scene offered
    /// `mortar_and_pestle` and `sage` — every entry wrong, every right answer
    /// missing — and contradicted `recall scripting`, the only page saying sets
    /// exist.
    #[test]
    fn for_each_offers_a_set_and_not_the_rooms_contents() {
        assert_eq!(spelling("for each ").texts(), ["way"]);

        let found = spelling("for each ").texts();
        assert!(
            !found.contains(&"mortar_and_pestle".to_owned()),
            "the room's contents were offered as sets: {found:?}",
        );
    }

    /// A verb word is not a thing, so nothing offers one as one.
    ///
    /// `scene_at` registers every one-word synonym as a `NounKind::Command` so
    /// the fuzzy matcher cannot turn a word the game knows into a noun. `wait `
    /// offering one is a spell that scans the record stream for something that
    /// never arrives, burns `PATIENCE` and latches a fault.
    #[test]
    fn a_verb_word_is_never_offered_as_a_thing() {
        let scene = Scene::new()
            .with(NounKind::Reagent, "sage")
            .with(NounKind::Command, "grind")
            .with(NounKind::Command, "walk");
        let sets: [String; 0] = [];
        let asked = |line: &str| {
            expect(
                line,
                line.chars().count(),
                &Situation {
                    scene: &scene,
                    spell: true,
                    prompt_open: false,
                    open: &[],
                    sets: &sets,
                },
            )
            .texts()
        };

        assert_eq!(asked("wait "), ["sage"], "a verb word was waitable");
        assert!(
            !asked("if the cabinet has ").contains(&"walk".to_owned()),
            "a verb word was offered as something a place could hold",
        );
    }

    /// A second bare `else` is not offered, and `else if` still chains.
    ///
    /// `open_blocks` tracked only opens and closes, so the `if` stayed on the
    /// stack after its else-branch had started and the guide went on offering
    /// `else`.
    #[test]
    fn a_ladder_that_has_had_its_else_is_not_offered_another() {
        let ladder = |lines: &[&str]| {
            let owned: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
            let open = open_blocks(&owned);
            inside("", &open).texts().contains(&"else".to_owned())
        };

        assert!(
            ladder(&["if the mortar_and_pestle is idle"]),
            "an open `if`"
        );
        assert!(
            !ladder(&["if the mortar_and_pestle is idle", "else"]),
            "a finished ladder was offered a second `else`",
        );
        // `else if` chains; forbidding it would outlaw the spelling that took
        // `threading` from 98 lines to 52.
        assert!(
            ladder(&[
                "if the mortar_and_pestle is idle",
                "else if the laboratory is idle",
            ]),
            "`else if` was read as the end of its ladder",
        );
        // ...and `end` still closes the frame the `else` left open.
        let closed: Vec<String> = ["if the mortar_and_pestle is idle", "else"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert_eq!(open_blocks(&closed).len(), 1, "the frame went missing");
        assert!(
            inside("", &open_blocks(&closed))
                .texts()
                .contains(&"end".to_owned()),
            "nothing was left to close",
        );
    }

    /// A spell is offered only verbs it is allowed to issue.
    ///
    /// `attend` passes `Scene::offers` — its fixture is the tower itself — and
    /// fails `may_issue`, because a spell is written *for* a domain and does not
    /// walk. Teaching a line the runner refuses is worse than offering nothing.
    #[test]
    fn a_spell_is_not_offered_a_verb_it_may_not_issue() {
        let prompt = typing("att").texts();
        assert!(prompt.contains(&"attend".to_owned()), "{prompt:?}");

        let spell = spelling("att").texts();
        assert!(
            !spell.contains(&"attend".to_owned()),
            "a spell was offered a verb it may not issue: {spell:?}",
        );
    }

    /// Each candidate carries how it is used, from the grammar's own table.
    ///
    /// This lived in `orbs-shell` as a second copy — a painter keeping its own
    /// answer to *what does `for` take*, which its own comment called out.
    #[test]
    fn a_candidate_carries_the_shape_that_follows_it() {
        let words = spelling("").expected;
        let repeat = words.iter().find(|one| one.text == "repeat");
        assert_eq!(repeat.map(|one| one.shape), Some("<count>"));

        // `end` takes nothing, and the empty string is the honest answer rather
        // than an absence every caller has to special-case. Reached from inside
        // a block, because with nothing open it is not offered at all.
        let closing = inside("", &[SpellWord::Repeat]).expected;
        let end = closing.iter().find(|one| one.text == "end");
        assert_eq!(end.map(|one| one.shape), Some(""));

        // A verb's is its signature label — the same one `help` lists.
        let verb = words.iter().find(|one| one.text == "survey");
        assert_eq!(verb.map(|one| one.shape), Some("place"));
    }

    /// The shared prefix is as far as everyone agrees.
    ///
    /// readline's `compute_lcd_of_matches`. Two surfaces read it — Tab spends
    /// it, the prompt's ghost draws what Tab would take — so a second
    /// implementation, very nearly written in `orbs-shell`, would have the ghost
    /// promising something Tab did not do.
    #[test]
    fn the_common_prefix_is_as_far_as_everyone_agrees() {
        // Two places share `m`... but only one starts with `mo`, so the whole
        // name is common.
        assert_eq!(typing("wield m").common(), "mortar_and_pestle");

        // With nothing typed, the laboratory's places share nothing.
        let both = typing("wield ");
        assert!(both.expected.len() > 1);
        assert_eq!(both.common(), "");

        // Nothing offered agrees on nothing, rather than panicking.
        assert_eq!(typing("xyzzy zz").common(), "");
    }

    /// An argument slot is the same answer in both, because it is the same verb.
    ///
    /// The one thing a spell must *not* get differently: `grind sage` means
    /// `grind sage` wherever it is written.
    #[test]
    fn an_argument_slot_answers_the_same_in_a_spell_as_at_the_prompt() {
        assert_eq!(spelling("wield mo"), typing("wield mo"));
        assert_eq!(spelling("wield mo").texts(), ["mortar_and_pestle"]);
    }

    /// Each candidate says what it is, so a surface need not ask again.
    #[test]
    fn a_candidate_carries_its_own_kind() {
        let nouns = typing("wield mo").expected;
        assert_eq!(nouns.first().map(Expected::kind), Some(Lexeme::Name));

        let verbs = typing("surv").expected;
        assert_eq!(verbs.first().map(Expected::kind), Some(Lexeme::Verb));

        // A grammar word is scaffolding to the *guide* and an ordinary word to
        // the *painter* — why `why` and `kind` are two facts rather than one.
        let hinge = spelling("if the mortar_and_pestle ").expected;
        let is = hinge.first().expect("a hinge");
        assert_eq!(is.why, Reason::Grammar);
        assert_eq!(is.kind(), Lexeme::Name);
    }

    /// The numbered prompt closes it, in a spell too.
    ///
    /// A spell never sees §6's prompt, but the flag is answered before anything
    /// else is looked at, and a surface that passes both is asking for nothing.
    #[test]
    fn nothing_is_expected_while_a_numbered_prompt_is_open() {
        let scene = tower();
        for spell in [false, true] {
            let found = expect(
                "sur",
                3,
                &Situation {
                    scene: &scene,
                    spell,
                    prompt_open: true,
                    open: &[],
                    sets: &[],
                },
            );
            assert!(found.is_empty(), "offered something at a numbered prompt");
        }
    }
}
