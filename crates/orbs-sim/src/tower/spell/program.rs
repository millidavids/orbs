//! A spell's lines, read as a program.
//!
//! # Derived, never stored
//!
//! [`Held`](crate::tower::Held) stays the single source of truth and this is a
//! **view** over it, rebuilt whenever a spell is cast. Two design commitments
//! are line-anchored and would break if the program became the truth:
//!
//! - §8's hot-reload: *"only lines the player actually changed are re-resolved
//!   by name; untouched lines stay bound by ID."*
//! - §8.1's script-text sabotage surface: *"a flag altered, a target changed,
//!   **a line reordered**."* An enemy mutates the text, and the program has to
//!   be whatever that text now means.
//!
//! # Malformed spells run
//!
//! §8 fixes both ends of this and leaves exactly one gap to fill. A spell
//! **cannot be refused at save** — *"`bind` always succeeds"*, and *"a draft you
//! cannot save is a dead end"* — and it **cannot halt at cast**, because the
//! failure taxonomy is titled *"scripts always log and never halt."*
//!
//! So an unmatched `repeat` is **closed at end of file**, an unmatched `end` is
//! dropped, and each is reported once naming the line. The spell runs. That is
//! the only answer both rules allow.

use crate::parser::{Condition, SpellWord, spell_argument, spell_word};

/// One thing a spell does, and the line of the file it came from.
///
/// The line is carried rather than derived because the program is a **tree** and
/// the file is a list: blank lines and comments are not steps at all, and a step
/// inside two blocks is three path elements deep with no arithmetic relating
/// that to a line number. The editor draws a marker beside the line a running
/// spell is on, and a spell that says *"line 3"* about the wrong line is worse
/// than one that says nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// Which line of the spell this came from, counting from one.
    pub line: usize,
    /// What it does.
    pub kind: Kind,
}

/// What a step does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// A command line, run through the ordinary dispatch.
    Command(String),
    /// Hold until something the spell names happens.
    ///
    /// §8's smallest control structure, and per the Autonauts precedent the one
    /// that gives a non-programmer conditional behaviour without a condition
    /// vocabulary: it needs no comparison, no truthiness, only a noun.
    Wait(String),
    /// Spend a number of ticks doing nothing.
    ///
    /// **A count, where [`Wait`](Self::Wait) is a noun**, and the two are not
    /// versions of each other: a wait ends when the world says so and a bide
    /// ends when the spell's own arithmetic says so. The menagerie is what
    /// needs the second, because *when* is the puzzle there and a blocking wait
    /// would hand that decision back to the world (§19).
    ///
    /// **A literal, and only ever a literal.** It was `Delay::Ticks(u32) |
    /// Delay::Reading(String)`, and `bide until` — the reading form — was the
    /// whole of why the menagerie's solver was trivial: a spell that reads its
    /// delay off the world computes nothing. Every count in the language is a
    /// literal again, as `repeat 5` and `has 4 fragment` always were.
    Bide(u32),
    /// Do the enclosed steps, `times` of them, or for ever if `None`.
    Repeat {
        /// How many times, or `None` for an unbounded loop.
        ///
        /// Unbounded is safe because the execution budget bounds a tick: a
        /// `repeat` with no `wait` in it spends its budget and stops, so it
        /// wastes itself rather than hanging the game.
        times: Option<u32>,
        /// The question that ends it, if it is bounded by one instead.
        ///
        /// **Asked before the first pass and again at the end of each**, which
        /// is Autonauts' rule and makes `repeat until <already true>` run zero
        /// times rather than one. A do-while would be the other choice and it is
        /// the wrong one: a guard that cannot prevent the first pass is not a
        /// guard, and the first pass is where a spell does damage.
        ///
        /// **Never set alongside `times`.** Two bounds on one loop is a
        /// semantics nobody asked for and a thing to teach; `repeat 5 until X`
        /// is refused at parse, naming the line.
        until: Option<crate::parser::Condition>,
        /// What to do each time.
        body: Block,
    },
    /// Do one branch or the other, depending on the tower.
    If {
        /// The question, or `None` if the orb could not read it — in which case
        /// the branch is **not taken**, and the line is reported once. Guessing
        /// would be worse: a condition the player did not write, deciding what
        /// their laboratory does while they are elsewhere.
        condition: Option<Condition>,
        /// What to do when it holds.
        body: Block,
        /// What to do when it does not. Empty unless there is an `else`.
        otherwise: Block,
    },
    /// Bind a name to a place, for the lines after it — `let best be north`.
    ///
    /// **One line, no block.** It is the accumulator half of *"follow the way
    /// with the fewest marks"*: something to compare against and then act on.
    Let {
        /// The word the spell will use.
        name: String,
        /// What it stands for — a name, resolved against the room at cast like
        /// every other, unless it is itself a name the spell binds.
        value: String,
    },
    /// Take the oldest name out of a satchel and bind it —
    /// `pull note from satchel`.
    ///
    /// **[`Let`](Self::Let)'s shape with the value coming from the world**, and
    /// the two are the whole of what writes to `vars`. It is a control word
    /// rather than a verb for exactly that reason: dispatch hands back records,
    /// so a `pull` verb could empty the satchel and would have nowhere to put
    /// what it took.
    ///
    /// **It yields while the satchel is empty** and never reaches `PATIENCE` —
    /// a consumer caught up with its producer is a working pipeline, not a
    /// fault. `bide`'s road, and `run::pull` says the rest.
    Pull {
        /// The word the spell will use for whatever comes out.
        name: String,
        /// Which satchel — a place, resolved against the room at cast.
        from: String,
    },
    /// Do the enclosed steps once for each member of a set — `for each way`.
    ///
    /// The cursor is bound to `group` itself, so the body reads
    /// `if way has spoil`. See [`SpellWord::For`]
    /// for why it is not `it`.
    Each {
        /// The set to walk, and the name the cursor takes.
        group: String,
        /// What to do for each of them.
        body: Block,
    },
    /// A named run of lines — `part gathering()`, `part between(here, there)`.
    ///
    /// **A definition, so reaching it executes nothing.** The runner steps past
    /// this exactly as a reader's eye does; the body runs only where a
    /// [`Call`](Self::Call) says so. It stays a step of the tree rather than
    /// being hoisted into a table beside it, because the tree is what the save,
    /// `interpret` and the editor's gutter all address by **line** — a
    /// definition lifted out of the body is a run of lines with no place in the
    /// file it came from.
    Part {
        /// What the part is called, without its parentheses.
        name: String,
        /// The names its arguments arrive under, in order.
        ///
        /// **These are the part's whole store**, not additions to the caller's:
        /// [`Descent`](super::Descent) keeps the caller's bindings and the part
        /// opens with only these. Empty for `part gathering()`, which is the
        /// shape every spell shipped before parameters existed.
        params: Vec<String>,
        /// What it does.
        body: Block,
    },
    /// Do a part — `gathering()`, `between(wellspring, near)`.
    ///
    /// Carries the **name** rather than a path to the definition, which is what
    /// lets a spell be edited while it runs (§8): a definition that moves up the
    /// file is still the same part, and a path would point at whatever took its
    /// place.
    Call {
        /// Which part.
        name: String,
        /// The names handed to it, in order, exactly as written.
        ///
        /// **Resolved against the caller's store where the call runs**, not
        /// here — `between(wellspring, near)` passes whatever `near` stands for
        /// at that moment, and a literal stands for itself. One level, which is
        /// the rule a bound name follows everywhere else in the language.
        args: Vec<String>,
    },
    /// Set a part running as a second cursor and carry on —
    /// `alongside gathering()`.
    ///
    /// **[`Call`](Self::Call)'s fields and none of its waiting.** A call
    /// suspends the caller onto a [`Descent`](super::Descent) and resumes it
    /// when the part returns; this starts a
    /// [`Strand`](super::Strand) on the part and leaves the caller exactly where
    /// it stands. The forked cursor has an empty stack because nothing is
    /// waiting for it: running off the end ends the strand and no more.
    ///
    /// Arguments resolve in the caller's store at the moment of the fork, which
    /// is a call's rule and has to be — a part's brackets are the whole of what
    /// it can see (§19), and a cursor that could read the caller's bindings
    /// *while the caller kept changing them* would be worse than the shared
    /// store that decision removed.
    Alongside {
        /// Which part to set running.
        name: String,
        /// The names handed to it, in order, exactly as written.
        args: Vec<String>,
    },
}

/// A run of steps.
pub type Block = Vec<Step>;

/// Something the orb had to fix to make sense of the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Complaint {
    /// Which line, counting from one.
    pub line: usize,
    /// The prose key for what was wrong.
    pub key: &'static str,
}

/// A spell's text, read as a shape — before any name in it has been resolved.
///
/// **Not runnable, and the type is what says so.** A draft's questions still
/// hold the words the player typed (`the mortar`, `the shelf`), which
/// [`holds`](super::holds) compares exactly and would find nothing for. Running
/// one would give a spell whose every condition answered "there is no such
/// place" — the exact silent failure §19 records as *"it looked exactly like the
/// condition being inverted"*.
///
/// [`compile`](super::compile()) turns one into a [`Program`], and nothing else
/// can. That is a weaker guarantee than it sounds — `Program::new` is reachable
/// from anywhere in this module — but it is the one that matters, because the
/// callers that would otherwise reach for the parser directly (`invoke`,
/// `scribe`'s reload) are *outside* it and now cannot.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Draft {
    /// What it does.
    pub body: Block,
    /// What the orb had to fix, if anything. **Never a refusal** — see the
    /// module docs.
    pub complaints: Vec<Complaint>,
}

/// A spell, as something that can be run: read, and with its names resolved.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Program {
    body: Block,
    complaints: Vec<Complaint>,
}

impl Program {
    /// Assemble one. `pub(super)` so [`compile`](mod@super::compile) is the only
    /// route from text to something runnable.
    #[must_use]
    pub(super) const fn new(body: Block, complaints: Vec<Complaint>) -> Self {
        Self { body, complaints }
    }

    /// What it does.
    #[must_use]
    pub const fn body(&self) -> &Block {
        &self.body
    }

    /// What the orb had to fix to read it.
    #[must_use]
    pub fn complaints(&self) -> &[Complaint] {
        &self.complaints
    }
}

/// A block being read, and what it will become when its `end` arrives.
enum Open {
    /// The spell itself. Never popped, which is what makes a stray `end` a
    /// complaint rather than a panic.
    Spell,
    Repeat {
        times: Option<u32>,
        until: Option<Condition>,
    },
    If {
        condition: Option<Condition>,
        /// The `then` branch, once an `else` has moved it aside. `None` while
        /// still reading the first half.
        taken: Option<Block>,
    },
    Each {
        group: String,
    },
    Part {
        name: String,
        params: Vec<String>,
    },
}

/// One entry on the block stack: where it opened, what it is, what is in it.
///
/// Deliberately **not** the obvious word for a stack entry: `tests/boundaries.rs`
/// forbids the four layout type names anywhere under `orbs-sim/src` — rule 2 —
/// and it matches by *substring*, so the guard is unarguable rather than clever.
/// A parser's stack entry is a false positive, and renaming costs less than
/// weakening the one test that keeps the sim away from the painter. (This
/// sentence cannot name the word either, which is the guard working.)
struct Nesting {
    line: usize,
    kind: Open,
    body: Block,
    /// Opened by `else if`, so one `end` closes this **and** what it hangs from.
    ///
    /// An `else if` chain is one construct to the player — four rungs and one
    /// `end` — and a tree of nested `if`s to the runner. This is the only thing
    /// that has to be remembered to keep those two views apart, and it is why the
    /// desugaring needs no new [`Kind`] and no runner change at all.
    chained: bool,
}

/// Read `lines` as a shape, resolving nothing.
///
/// `pub(super)` deliberately: the way in from outside this module is
/// [`compile`](mod@super::compile), which does this and then fixes the names. See
/// [`Draft`].
#[must_use]
pub(super) fn read(lines: &[String]) -> Draft {
    let mut complaints = Vec::new();
    let mut open = vec![Nesting {
        line: 0,
        kind: Open::Spell,
        body: Vec::new(),
        chained: false,
    }];

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let at = index + 1;

        match spell_word(trimmed) {
            Some(SpellWord::Repeat) => {
                let argument = spell_argument(trimmed);
                // **One bound per loop.** `repeat` alone is unbounded, `repeat 5`
                // counts, `repeat until X` asks — and `repeat 5 until X` is two
                // bounds whose interaction would be a rule to teach for a shape
                // nobody reaches for. Refused by name rather than resolved by
                // precedence, which is the same call `debug_spawn` makes about a
                // count it cannot read.
                let counted = count_of(argument);
                // **The word anywhere, not just in front.** `repeat 5 until X`
                // has `until` in second place, so looking only at the head would
                // read it as `repeat 5` and drop the rest in silence — the quiet
                // reinterpretation this file refuses everywhere else.
                let mentions = argument
                    .split_whitespace()
                    .any(|word| word.eq_ignore_ascii_case(SpellWord::Until.canonical()));
                let guard = guard_of(argument);
                // **A bound the orb cannot read makes the loop run nought times,
                // not for ever.** `(None, None)` is `Loop::Repeat(None)`, which
                // `step_past` loops unbounded — so a typo in a guard produced the
                // exact opposite of what `spell_two_bounds` and
                // `spell_unreadable_until` both say, and worse than either. A
                // player who mistypes one word got an infinite loop and a message
                // reading *"the repeat stops"*.
                //
                // `Some(0)` is stepped past by the runner without entering, which
                // is the same treatment `repeat 0` gets and is what the prose
                // describes.
                let refused = (Some(0), None);
                let (times, until) = if mentions && counted.is_some() {
                    complaints.push(Complaint {
                        line: at,
                        key: "spell_two_bounds",
                    });
                    refused
                } else if mentions && guard.is_none() {
                    // Either the question would not parse, or `until` is buried
                    // somewhere it cannot be read from. Both are a bound the
                    // player wrote and the orb cannot use, and an unbounded loop
                    // is not a safe thing to guess at.
                    complaints.push(Complaint {
                        line: at,
                        key: "spell_unreadable_until",
                    });
                    refused
                } else {
                    (counted, guard)
                };
                open.push(Nesting {
                    line: at,
                    kind: Open::Repeat { times, until },
                    body: Vec::new(),
                    chained: false,
                });
            }
            Some(SpellWord::Until) => complaints.push(Complaint {
                line: at,
                key: "spell_stray_until",
            }),
            Some(SpellWord::If) => {
                let condition = crate::parser::condition(spell_argument(trimmed));
                if condition.is_none() {
                    // Unreadable, and **not guessed at**. A condition the player
                    // did not write would decide what their laboratory does
                    // while they are elsewhere; the branch simply never runs.
                    complaints.push(Complaint {
                        line: at,
                        key: "spell_unreadable_if",
                    });
                }
                open.push(Nesting {
                    line: at,
                    kind: Open::If {
                        condition,
                        taken: None,
                    },
                    body: Vec::new(),
                    chained: false,
                });
            }
            Some(SpellWord::Else) => {
                // **`else if` is a chained `if`, not a new word.** The ladder is
                // the shape every solver in the game is written in — the ward's
                // four rungs, the maze's twenty-four — and without this each rung
                // nests one deeper and pays for an `end` at the bottom. Half of
                // `threading` is `end` and `else`: 49 lines of 98.
                //
                // What follows `else` is read as an `if` line, so there is one
                // reader for a condition and one set of complaint keys. A
                // desugaring rather than a `Kind`, so the runner, `step_past`,
                // `guard_answers`, the save format and `interpret` all need
                // nothing: what they see is the nested tree that was always
                // written by hand.
                let tail = spell_argument(trimmed).trim();
                let chaining = spell_word(tail) == Some(SpellWord::If);

                // Only an `if` has an other half. Anywhere else it is a line the
                // orb cannot place, kept as a complaint rather than silently
                // starting a second branch on a loop.
                match open.last_mut() {
                    Some(Nesting {
                        kind: Open::If { taken, .. },
                        body,
                        ..
                    }) if taken.is_none() => {
                        *taken = Some(std::mem::take(body));
                    }
                    _ => {
                        complaints.push(Complaint {
                            line: at,
                            key: "spell_stray_else",
                        });
                        continue;
                    }
                }

                if chaining {
                    let condition = crate::parser::condition(spell_argument(tail));
                    if condition.is_none() {
                        // The same treatment a plain `if` gets, from the same
                        // reader — an unreadable rung runs neither half rather
                        // than being guessed at.
                        complaints.push(Complaint {
                            line: at,
                            key: "spell_unreadable_if",
                        });
                    }
                    open.push(Nesting {
                        line: at,
                        kind: Open::If {
                            condition,
                            taken: None,
                        },
                        body: Vec::new(),
                        chained: true,
                    });
                }
            }
            Some(SpellWord::End) => {
                if open.len() == 1 {
                    // A close with nothing open. Dropped, and said once.
                    complaints.push(Complaint {
                        line: at,
                        key: "spell_stray_end",
                    });
                    continue;
                }
                // **One `end` closes the whole chain.** Each `else if` pushed a
                // frame; the player wrote one block and owes one close, so the
                // unwind continues while the frame it just shut was chained.
                while let Some(frame) = open.pop() {
                    let chained = frame.chained;
                    close(&mut open, frame);
                    if !chained || open.len() == 1 {
                        break;
                    }
                }
            }
            Some(SpellWord::Wait) => {
                let wanted = strip_filler(spell_argument(trimmed));
                push(&mut open, at, Kind::Wait(wanted));
            }
            Some(SpellWord::Bide) => {
                let argument = strip_filler(spell_argument(trimmed));
                match count_of(&argument) {
                    Some(ticks) => push(&mut open, at, Kind::Bide(ticks)),
                    // **A bare word is a complaint again.** It used to compile to
                    // `Delay::Reading`, so `bide until` read the delay off the
                    // world and the menagerie's solver had no arithmetic left in
                    // it. Refusing it here is what puts the counting back, and it
                    // catches `bide sage` — a typo that answered `Endless` and
                    // bided `u32::MAX` — as the same complaint rather than as
                    // four billion ticks of silence.
                    None => complaints.push(Complaint {
                        line: at,
                        key: "spell_unreadable_bide",
                    }),
                }
            }
            Some(SpellWord::Let) => match binding(spell_argument(trimmed)) {
                Some((name, value)) => push(&mut open, at, Kind::Let { name, value }),
                // **A complaint, not a guess.** `set best` names nothing to bind
                // and `set to north` binds nothing — either read as the other
                // would be the orb writing a line the player did not, which is
                // what every refusal in this file is protecting against.
                None => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_let",
                }),
            },
            // **`strip_filler` here, and `let` above deliberately without it.**
            // `spell_argument` does not strip — `bide` calls it separately and
            // this has to as well, or `pull note from satchel` arrives as three
            // words and is refused. It cost one See-it line to find, and the
            // failure is the quiet kind: a complaint on a line that reads
            // perfectly.
            //
            // `let` must *not* strip, because its keyword is ` be ` and
            // `binding` splits the raw text on it. Stripping first would leave
            // the value shorn of a `the` the player typed and then match a name
            // that never appeared in the file.
            Some(SpellWord::Pull) => match pulled(&strip_filler(spell_argument(trimmed))) {
                Some((name, from)) => push(&mut open, at, Kind::Pull { name, from }),
                // `let`'s refusal, for `let`'s reason: `pull note` names no
                // satchel and `pull satchel` binds nothing, and reading either
                // as the other is the orb writing a line the player did not.
                None => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_pull",
                }),
            },
            Some(SpellWord::For) => match walked(spell_argument(trimmed)) {
                Some(group) => open.push(Nesting {
                    line: at,
                    kind: Open::Each { group },
                    body: Vec::new(),
                    chained: false,
                }),
                // **No block is opened**, so the `end` the player wrote below is
                // a stray one and says so. Opening an unnamed block instead
                // would swallow the body into a loop over nothing, silently.
                None => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_for",
                }),
            },
            Some(SpellWord::Part) => match part_signature(spell_argument(trimmed)) {
                Some((name, params)) => open.push(Nesting {
                    line: at,
                    kind: Open::Part { name, params },
                    body: Vec::new(),
                    chained: false,
                }),
                // **No block is opened**, exactly as an unreadable `for each`
                // opens none — so the `end` below is a stray one and says so.
                // Opening an unnamed part instead would swallow the body into
                // something nothing can ever call.
                None => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_part",
                }),
            },
            // **A fork is a call with a word in front of it**, so it reuses the
            // same parser rather than growing a second one — `alongside
            // between(a b)` has to be the same complaint as `between(a b)`, or
            // the two spellings of one mistake read differently.
            Some(SpellWord::Alongside) => match call_of(spell_argument(trimmed)) {
                Some(Some((name, args))) => push(&mut open, at, Kind::Alongside { name, args }),
                Some(None) => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_call",
                }),
                // **No brackets at all**, which is its own complaint: `alongside
                // gathering` reads as a bare name, and a bare name is the one
                // thing a call may not be. Saying so beats reading it as a call
                // the player did not punctuate.
                None => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_alongside",
                }),
            },
            // A call is punctuation, so it arrives here rather than through
            // `spell_word` — there is no ninth control word to match.
            None if let Some(called) = call_of(trimmed) => match called {
                Some((name, args)) => push(&mut open, at, Kind::Call { name, args }),
                // `between(a b)`, or a bracket nothing closed. Reading it as a
                // bare call would drop a word the player wrote — the quiet
                // reinterpretation this file refuses everywhere.
                None => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_call",
                }),
            },
            None => push(&mut open, at, Kind::Command(trimmed.to_owned())),
        }
    }

    // **Unmatched opens are closed here**, innermost first, each reported once.
    // §8 leaves no other option: the save could not refuse and the cast may not
    // halt, so the spell runs as if the player had finished typing it.
    while open.len() > 1 {
        let Some(frame) = open.pop() else {
            break;
        };
        // A chained frame is half of a construct someone else opened, so the
        // complaint belongs to the `if` at the top of the ladder and is made
        // once — not once per rung.
        if !frame.chained {
            complaints.push(Complaint {
                line: frame.line,
                key: "spell_unclosed",
            });
        }
        close(&mut open, frame);
    }

    // **No `expect`.** The outermost block cannot be popped by the loops
    // above — both are guarded on `len() > 1` — but writing that as a panic
    // would be a claim the compiler cannot check and a crash if it ever
    // stopped being true. An empty spell is a real thing anyway.
    let mut body = open.pop().map(|frame| frame.body).unwrap_or_default();
    settle_parts(&mut body, &mut complaints);

    Draft { body, complaints }
}

/// Enforce the two rules a definition has, after the tree is built.
///
/// A pass of its own rather than a check inside [`read`], because both rules are
/// about a definition's **place among the others** and neither can be answered
/// while the block that holds it is still open.
///
/// - **Top-level only.** [`tree`] looks no deeper, so a `part` inside a `repeat`
///   would be a run of lines nothing could ever call — and silently, since the
///   runner steps past a definition wherever it finds one. Dropped and said.
/// - **One name, one part.** Two definitions sharing a name make `tree` answer
///   with whichever is written first, so the second is a block the player wrote
///   and the orb will never run. The same call `progression.toml` makes about a
///   duplicate node id, for the same reason: two entries under one name mean
///   naming either names both.
fn settle_parts(body: &mut Block, complaints: &mut Vec<Complaint>) {
    // Kept in writing order, so the *first* definition of a name is the one that
    // survives — which is what `tree` would have answered with anyway.
    let mut named: Vec<String> = Vec::new();
    body.retain(|step| {
        let Kind::Part { name, .. } = &step.kind else {
            return true;
        };
        if named.iter().any(|already| already == name) {
            complaints.push(Complaint {
                line: step.line,
                key: "spell_repeated_part",
            });
            return false;
        }
        named.push(name.clone());
        true
    });

    // Anything deeper than the top level, cut out and reported once each.
    for step in body.iter_mut() {
        strip_nested(&mut step.kind, complaints);
    }
}

/// Remove any definition below the top level, reporting each once.
fn strip_nested(kind: &mut Kind, said: &mut Vec<Complaint>) {
    let inner: Vec<&mut Block> = match kind {
        Kind::Repeat { body, .. } | Kind::Each { body, .. } | Kind::Part { body, .. } => {
            vec![body]
        }
        Kind::If {
            body, otherwise, ..
        } => vec![body, otherwise],
        Kind::Command(_)
        | Kind::Wait(_)
        | Kind::Bide(_)
        | Kind::Let { .. }
        | Kind::Pull { .. }
        | Kind::Call { .. }
        | Kind::Alongside { .. } => Vec::new(),
    };
    for block in inner {
        block.retain(|step| {
            let is_part = matches!(step.kind, Kind::Part { .. });
            if is_part {
                said.push(Complaint {
                    line: step.line,
                    key: "spell_nested_part",
                });
            }
            !is_part
        });
        for step in block.iter_mut() {
            strip_nested(&mut step.kind, said);
        }
    }
}

/// What an open block on the runner's stack is.
///
/// A plain count was enough while `repeat` was the only block. An `if` has
/// **two** bodies, so the path has to say which one execution went into — and
/// walking back out of a branch pops one more path element than walking out of a
/// loop does. Recording the kind is what keeps those two exits from being one
/// piece of arithmetic that is right for one of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loop {
    /// A `repeat`, with the turns it has left.
    Repeat(Option<u32>),
    /// A branch of an `if`. Runs once, and its path carries which half.
    Branch,
    /// A `for each`, with the index of the member currently bound.
    ///
    /// # An index, not the members themselves
    ///
    /// The set is re-read from the world at the top of every pass, and this says
    /// only how far along it the loop has got. Two reasons, and the second is the
    /// one that decides it:
    ///
    /// - a `Loop` is `Copy` and travels to a save as **one integer per open
    ///   block**, which the save format's own note prefers to a tagged table;
    /// - a spell runs in a live world, so a set that changes under it should be
    ///   walked as it now is. Carrying a snapshot would have `for each way`
    ///   iterate a maze that has since closed.
    Each(u32),
}

/// The step at `pc`, if the path resolves.
///
/// A path rather than an index: `[0]` is the first step, `[2, 1]` the second
/// step inside the third. Blocks made a single line number insufficient the
/// moment they arrived.
///
/// **An `if` costs two path elements**, not one: the branch, then the step
/// within it. `[2, 0, 1]` is the second step of the `then` half of step three.
#[must_use]
pub fn at<'a>(body: &'a Block, pc: &[usize]) -> Option<&'a Step> {
    let (&first, rest) = pc.split_first()?;
    let step = body.get(first)?;
    if rest.is_empty() {
        return Some(step);
    }
    match &step.kind {
        // **One path element, exactly like a `repeat`.** A `for each` has one
        // body and no second half, so the arithmetic that walks out of it is the
        // loop's rather than the branch's — see [`Loop::Each`].
        Kind::Repeat { body, .. } | Kind::Each { body, .. } => at(body, rest),
        Kind::If {
            body, otherwise, ..
        } => {
            let (&branch, inner) = rest.split_first()?;
            at(if branch == 0 { body } else { otherwise }, inner)
        }
        // **A definition is not descended into from the body it sits in.** It is
        // reached by name through [`tree`], which is what lets a call find it
        // after an edit has moved it — so a path that points *into* one is a
        // path nothing builds, and answering `None` says so.
        Kind::Command(_)
        | Kind::Wait(_)
        | Kind::Bide(_)
        | Kind::Let { .. }
        | Kind::Pull { .. }
        | Kind::Part { .. }
        | Kind::Call { .. }
        | Kind::Alongside { .. } => None,
    }
}

/// The body a frame walks: the spell's own, or a named part's.
///
/// **Found by name at every step, never cached.** A part is a run of lines in a
/// file the player may be editing while it runs (§8), so the answer to *which
/// lines is `gathering`* has to be asked of the text as it now is. A path or an
/// index taken at the call would point at whatever moved into its place.
///
/// Definitions are **top-level only** — see `spell_nested_part` — so this looks
/// no deeper, and a part inside a `repeat` is unreachable by design rather than
/// by oversight.
#[must_use]
pub fn tree<'a>(body: &'a Block, part: Option<&str>) -> Option<&'a Block> {
    let Some(wanted) = part else {
        return Some(body);
    };
    body.iter().find_map(|step| match &step.kind {
        Kind::Part { name, body, .. } if name == wanted => Some(body),
        _ => None,
    })
}

/// The names one part takes its arguments under, if it is defined at all.
///
/// **`None` and an empty list are different answers**, which is why this is not
/// folded into [`parts`]: no such part is a missing name the orb says once, and
/// a part taking nothing is `gathering()` working normally. Found by name at
/// every call for the reason [`tree`] is — a definition that moves while the
/// spell runs is still the same part.
#[must_use]
pub fn signature(body: &Block, part: &str) -> Option<Vec<String>> {
    body.iter().find_map(|step| match &step.kind {
        Kind::Part { name, params, .. } if name == part => Some(params.clone()),
        _ => None,
    })
}

/// Every part the spell defines and how many names it takes, in writing order.
///
/// The count travels with the name because the only two questions anyone asks
/// of this list are *is there such a part* and *does this call fit it*, and
/// answering the second from a second walk of the tree is how two expressions of
/// one rule come to disagree.
#[must_use]
pub fn parts(body: &Block) -> Vec<(&str, usize)> {
    body.iter()
        .filter_map(|step| match &step.kind {
            Kind::Part { name, params, .. } => Some((name.as_str(), params.len())),
            _ => None,
        })
        .collect()
}

/// Move `pc` past the step it points at, closing and repeating blocks as needed.
///
/// Returns `false` when the program has run out — which is the only way a spell
/// finishes, since §8's taxonomy forbids halting on a failure.
///
/// # The loop stack is why a save needs more than a `pc`
///
/// Each open `repeat` has a count on `loops`, outermost first. Restoring a
/// suspended spell from a `pc` alone would resume every enclosing loop from its
/// first iteration — §8 requires in-flight state be serialisable, and a spell
/// inside a loop is exactly that.
/// `again` is asked whether a block that has just run off the end may go round,
/// with the path pointing at the block's own step and the [`Loop`] that was
/// popped. A `repeat` with no `until` answers yes; a `for each` answers whether
/// the set still has a member after the one just finished.
///
/// **It takes the popped loop as well as the path**, because those two questions
/// need different things: the guard needs the step (to find its `until`) and the
/// set needs the index (to know which member is next). This is the only thing in
/// the walker that consults the world, and it is a closure rather than a
/// `&World` parameter because `step_past` is otherwise pure and its test callers
/// have no world to give it.
pub fn step_past(
    body: &Block,
    pc: &mut Vec<usize>,
    loops: &mut Vec<Loop>,
    mut again: impl FnMut(&[usize], Loop) -> bool,
) -> bool {
    if pc.is_empty() {
        return false;
    }
    if let Some(last) = pc.last_mut() {
        *last += 1;
    }

    // Walk out of any block the path has just run off the end of. Each one
    // either goes round again or is finished with.
    while at(body, pc).is_none() {
        if pc.len() == 1 {
            // Off the end of the outermost block: the spell is done.
            return false;
        }
        pc.pop();
        match loops.pop() {
            // Unbounded, or more to go: back to the top of this block — **and
            // only if the guard still says so**. `again` is asked with the path
            // pointing at the `repeat` itself, which is how the caller finds the
            // step and its `until`; a loop with no guard answers yes and behaves
            // exactly as it always did.
            //
            // **Asked after the count, never before it.** `repeat 3 until X` is
            // refused at parse, so the two never meet here — but short-circuiting
            // keeps an exhausted loop from putting a question to the world on its
            // way out, which would be a read nobody asked for.
            Some(Loop::Repeat(left))
                if left.is_none_or(|turns| turns > 1) && again(pc, Loop::Repeat(left)) =>
            {
                loops.push(Loop::Repeat(left.map(|turns| turns - 1)));
                pc.push(0);
                return true;
            }
            // **The set is asked, not counted here.** A `for each` walks a live
            // world, so whether there is another member is a question for the
            // tick the loop laps on rather than for the tick it started — see
            // [`Loop::Each`].
            Some(Loop::Each(index)) if again(pc, Loop::Each(index)) => {
                loops.push(Loop::Each(index + 1));
                pc.push(0);
                return true;
            }
            // A branch runs once, and cost **two** path elements going in — the
            // half, then the step. Popping only one would leave the path
            // pointing at the other half of the `if` and run it as well.
            Some(Loop::Branch) => {
                pc.pop();
                if let Some(last) = pc.last_mut() {
                    *last += 1;
                }
            }
            // Finished looping. Step past the `repeat` itself and check again,
            // because the block *containing* it may also have ended.
            _ => {
                if let Some(last) = pc.last_mut() {
                    *last += 1;
                }
            }
        }
    }
    true
}

/// Every name a spell binds, anywhere in it.
///
/// # Lexical, and deliberately not scoped
///
/// A `set` inside an `if` binds a name the lines after the `end` can still say,
/// and a `for each` cursor outlives its loop holding the last member. Both are
/// the simple reading, and the simple reading is the right one here: §8's
/// language has no declarations, so a player who writes `set best to north`
/// inside a branch and reads `best` below it means what they wrote. Scoping
/// would be a rule to teach and a rule to get wrong, for a program that fits on
/// a screen.
///
/// The list is what [`compile`](mod@super::compile) uses to tell a variable from a
/// place the room does not have — a distinction it cannot otherwise make, since
/// both are words that resolve to nothing at cast.
#[must_use]
pub(super) fn bindings(body: &Block) -> Vec<String> {
    let mut names = Vec::new();
    gather(body, &mut names);
    names
}

fn gather(body: &Block, names: &mut Vec<String>) {
    for step in body {
        match &step.kind {
            Kind::Let { name, .. } => {
                if !names.iter().any(|already| already == name) {
                    names.push(name.clone());
                }
            }
            Kind::Each { group, body } => {
                if !names.iter().any(|already| already == group) {
                    names.push(group.clone());
                }
                gather(body, names);
            }
            Kind::Repeat { body, .. } => gather(body, names),
            Kind::If {
                body, otherwise, ..
            } => {
                gather(body, names);
                gather(otherwise, names);
            }
            // **A part's parameters and its `let`s are gathered too, even
            // though they are scoped to it.** This list has exactly one job —
            // stopping `check_commands` and `interpret` resolving a line that
            // names a variable against the room — and for that a name too many
            // is harmless where a name too few is a working line painted red.
            //
            // So it is deliberately the *file's* names rather than any one
            // frame's: a part's `here` reaches this list, and the only cost is
            // that a caller writing `follow here` outside the part is quoted
            // rather than resolved. The runner is the thing that scopes (see
            // [`Descent::vars`](super::Descent)); this is a lint's input.
            Kind::Part { params, body, .. } => {
                for param in params {
                    if !names.iter().any(|already| already == param) {
                        names.push(param.clone());
                    }
                }
                gather(body, names);
            }
            // **A `pull` binds a name and is gathered as one.** It is `let`'s
            // arm in every way that matters here — the lint's whole job is to
            // stop a line naming a variable being resolved against the room, and
            // `sing note` after `pull note from satchel` is exactly that line.
            // Missing it would paint a working line red.
            Kind::Pull { name, .. } => {
                if !names.iter().any(|already| already == name) {
                    names.push(name.clone());
                }
            }
            Kind::Command(_)
            | Kind::Wait(_)
            | Kind::Bide(_)
            | Kind::Call { .. }
            | Kind::Alongside { .. } => {}
        }
    }
}

/// Descend into a `repeat`, recording how many times it should run.
pub fn enter(pc: &mut Vec<usize>, loops: &mut Vec<Loop>, times: Option<u32>) {
    loops.push(Loop::Repeat(times));
    pc.push(0);
}

/// Descend into a `for each`, at its first member.
///
/// One path element, like a `repeat` and unlike an `if`: there is one body and
/// no half to record.
pub fn enter_each(pc: &mut Vec<usize>, loops: &mut Vec<Loop>) {
    loops.push(Loop::Each(0));
    pc.push(0);
}

/// Descend into one half of an `if`.
///
/// Pushes the half **and** the step within it, which is what makes an `if` two
/// path elements deep — see [`at`].
pub fn enter_branch(pc: &mut Vec<usize>, loops: &mut Vec<Loop>, taken: bool) {
    loops.push(Loop::Branch);
    pc.push(usize::from(!taken));
    pc.push(0);
}

/// Add a step to whichever block is currently open.
fn push(open: &mut [Nesting], line: usize, kind: Kind) {
    if let Some(frame) = open.last_mut() {
        frame.body.push(Step { line, kind });
    }
}

/// Turn a finished frame into a step of the block that encloses it.
///
/// `Open::Spell` cannot reach here — both callers guard on `len() > 1` — but it
/// is folded in rather than panicked on, because a claim the compiler cannot
/// check is a crash waiting for the day it stops being true.
fn close(open: &mut [Nesting], frame: Nesting) {
    let line = frame.line;
    let kind = match frame.kind {
        Open::Repeat { times, until } => Kind::Repeat {
            times,
            until,
            body: frame.body,
        },
        Open::If { condition, taken } => match taken {
            // An `else` was seen, so what accumulated after it is the other half.
            Some(body) => Kind::If {
                condition,
                body,
                otherwise: frame.body,
            },
            None => Kind::If {
                condition,
                body: frame.body,
                otherwise: Vec::new(),
            },
        },
        Open::Each { group } => Kind::Each {
            group,
            body: frame.body,
        },
        Open::Part { name, params } => Kind::Part {
            name,
            params,
            body: frame.body,
        },
        Open::Spell => return,
    };
    push(open, line, kind);
}

/// `between(here, there)` → the name and the names its arguments arrive under.
///
/// **The parentheses are optional on a definition only while it takes nothing.**
/// A heading is already unambiguous — `part` says what the line is — so
/// demanding them for `part gathering` would be ceremony; the moment there is a
/// parameter list there is nowhere else to put it. At a call site they are the
/// entire notation and are always required.
///
/// `None` for a name that is empty, is more than one word, or has a parameter
/// list the orb cannot read — an unclosed bracket, an empty slot from a trailing
/// comma, a parameter that is a phrase rather than a name. `part gather the
/// sage` is a sentence rather than a heading, and reading it as one would set
/// aside a body under a name nothing can call.
///
/// **A repeated parameter is refused here rather than shadowed.** `part
/// between(here, here)` would bind the second over the first and leave the
/// caller's first argument unreachable, which is the quiet reinterpretation this
/// file refuses everywhere.
fn part_signature(argument: &str) -> Option<(String, Vec<String>)> {
    let trimmed = argument.trim();
    let Some((head, rest)) = trimmed.split_once('(') else {
        // No brackets at all: the whole heading is the name, and it must be one
        // word. This is the `part gathering` shape.
        let mut words = trimmed.split_whitespace();
        let name = words.next()?;
        if words.next().is_some() || name.contains(')') {
            return None;
        }
        return Some((name.to_lowercase(), Vec::new()));
    };
    let inside = rest.strip_suffix(')')?;
    let name = head.trim();
    if name.is_empty() || name.split_whitespace().count() != 1 || name.contains(')') {
        return None;
    }
    let params = parameters(inside)?;
    Some((name.to_lowercase(), params))
}

/// `here, there` → the names, lowercased. `None` if any slot is not one name.
///
/// Empty text is no parameters, which is what `()` means. Everything else must
/// be comma-separated single words: a trailing comma leaves an empty slot and is
/// refused rather than dropped, because a player who wrote one meant to type
/// another name.
fn parameters(inside: &str) -> Option<Vec<String>> {
    if inside.trim().is_empty() {
        return Some(Vec::new());
    }
    let mut names = Vec::new();
    for slot in inside.split(',') {
        let mut words = slot.split_whitespace();
        let name = words.next()?;
        if words.next().is_some() || name.contains('(') || name.contains(')') {
            return None;
        }
        let name = name.to_lowercase();
        if names.contains(&name) {
            return None;
        }
        names.push(name);
    }
    Some(names)
}

/// A part's name and the names handed to it — what [`call_of`] answers with.
type Call = (String, Vec<String>);

/// `between(wellspring, near)` → the part it calls and what it hands over.
///
/// Three answers, not two: `None` for a line that is not a call, `Some(None)`
/// for one that is a call and is malformed, and `Some(Some((name, args)))` for a
/// good one. Collapsing the middle into "not a call" would send `between(a b)`
/// to the command resolver, which would report that the tower has no
/// `between(a b)` — true, unhelpful, and about the wrong thing.
///
/// **The line must *end* with the bracket**, which is what `strip_suffix` says
/// and is the rule `lexeme::call_runs` has to match: `gathering() # note` is a
/// command, not a call with something after it.
///
/// **Whether the count is right is not asked here**, and deliberately: that
/// needs the definition, which is a fact about the file rather than about the
/// line. `compile::check_calls` answers it beside *is there such a part at all*,
/// so both arrive as one report about the spell.
///
/// **An argument may repeat where a parameter may not.** `between(here, here)`
/// as a *call* is two slots given the same name, which is ordinary; as a
/// heading it would be one name shadowing another, which is not.
pub(super) fn call_of(line: &str) -> Option<Option<Call>> {
    let trimmed = line.trim();
    let (head, rest) = trimmed.split_once('(')?;
    let inside = rest.strip_suffix(')')?;
    let name = head.trim();
    if name.is_empty() || name.split_whitespace().count() != 1 {
        return Some(None);
    }
    let Some(args) = arguments(inside) else {
        return Some(None);
    };
    Some(Some((name.to_lowercase(), args)))
}

/// `wellspring, near` → the names handed over, lowercased.
///
/// [`parameters`]'s rule minus the distinctness one — see [`call_of`].
fn arguments(inside: &str) -> Option<Vec<String>> {
    if inside.trim().is_empty() {
        return Some(Vec::new());
    }
    inside
        .split(',')
        .map(|slot| {
            let mut words = slot.split_whitespace();
            let name = words.next()?;
            if words.next().is_some() || name.contains('(') || name.contains(')') {
                return None;
            }
            Some(name.to_lowercase())
        })
        .collect()
}

/// `best be north` → the name and what it stands for.
///
/// # `be`, and not `to`
///
/// `to` is on §6's filler list — `move sage to mortar` fills two slots
/// positionally and the preposition carries nothing — so a keyword `to` would be
/// invisible to every reader in the parser except this one, which sees the text
/// before normalisation. `be` carries the same sentence with none of that, and
/// `let best be north` is the English either way.
///
/// **Both halves are required.** `let best` has nothing to bind and `let be
/// north` has no name; either read as the other is the orb writing a line the
/// player did not.
fn binding(argument: &str) -> Option<(String, String)> {
    let (name, value) = argument.split_once(" be ")?;
    let (name, value) = (name.trim(), value.trim());
    // A name is **one word**. `let the best way be north` would otherwise bind
    // something no later line could spell, since a use site is matched word by
    // word.
    if name.is_empty() || value.is_empty() || name.split_whitespace().count() != 1 {
        return None;
    }
    Some((name.to_lowercase(), value.to_owned()))
}

/// `note from satchel` → the name to bind, and the satchel to take it from.
///
/// **`from` is already gone by here.** It is on §6's filler list, so
/// `spell_argument` hands over `note satchel` — two words, positional. That is
/// why this splits on whitespace where [`binding`] splits on a keyword: `let`'s
/// `be` survives normalisation and carries the grammar, and `from` is decoration
/// a reader wants. `pull note satchel` is the same line and is accepted.
///
/// **Not checked against the world here**, like every other name in a [`Draft`]:
/// a satchel the room does not have is `compile`'s to catch, in the room.
fn pulled(argument: &str) -> Option<(String, String)> {
    let mut words = argument.split_whitespace();
    let name = words.next()?;
    let from = words.next()?;
    // Two words and no more. `pull a b c` is a shape the language does not have
    // and must not read as one, which is `walked`'s rule one function down.
    if words.next().is_some() {
        return None;
    }
    Some((name.to_lowercase(), from.to_owned()))
}

/// `each way` → the set to walk.
///
/// The particle is required and carries nothing:
/// [`SpellWord::particle`](crate::parser::SpellWord::particle) says why.
///
/// **Not checked against the world here.** `read` resolves nothing — that is the
/// whole of what [`Draft`] means — so a set the room does not have is caught by
/// [`compile`](mod@super::compile), in the room, where every other name is.
fn walked(argument: &str) -> Option<String> {
    let mut words = argument.split_whitespace();
    let particle = SpellWord::For.particle()?;
    if !words.next()?.eq_ignore_ascii_case(particle) {
        return None;
    }
    let group = words.next()?;
    // One word, and nothing after it. `for each way and socket` is two sets,
    // which is a shape the language does not have and must not read as one.
    if words.next().is_some() {
        return None;
    }
    Some(group.to_lowercase())
}

/// `repeat 3` → `Some(3)`; a bare `repeat` → `None`.
///
/// A word that is not a number is **not** an error, and it used to be this
/// function's job to say why — the doc here named `repeat until the mortar` as
/// *"a reasonable thing to try"* that was read as an unbounded loop because the
/// language had no `until`. It has one now, so that case is handled properly
/// above and this stays lenient only for the rest.
fn count_of(argument: &str) -> Option<u32> {
    argument.split_whitespace().next()?.parse().ok()
}

/// What follows `until` in a `repeat`'s argument, if it opens with one.
///
/// **The word must be first.** `repeat until X` is the shape; anything else with
/// `until` buried in it is a question the player wrote oddly, and finding the
/// word anywhere would make `repeat 3 until` and `repeat the until room` both
/// mean something. §6's rule is that the orb says what it could not read.
fn after_until(argument: &str) -> Option<&str> {
    // **Case-folded, because every other control word is.** `spell_word`
    // lowercases the first word, so `Repeat Until the mortar is idle` is
    // recognised as a repeat — and this then failed to see its own keyword,
    // which made `mentions` (case-insensitive) and this disagree: the player was
    // told the question meant nothing, for a line differing from a working one by
    // a capital letter.
    let keyword = SpellWord::Until.canonical();
    let head = argument.get(..keyword.len())?;
    if !head.eq_ignore_ascii_case(keyword) {
        return None;
    }
    let rest = &argument[keyword.len()..];
    // `untilX` is not this word.
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    Some(rest.trim())
}

/// The question bounding a `repeat`, if it is bounded by one.
fn guard_of(argument: &str) -> Option<Condition> {
    crate::parser::condition(after_until(argument)?)
}

/// Drop §6's filler words from what a `wait` was given.
///
/// `wait for the mortar` names the mortar. Going through the parser's own filler
/// table rather than a second list, so `for` and `the` mean the same thing here
/// as they do everywhere else.
fn strip_filler(argument: &str) -> String {
    argument
        .split_whitespace()
        .filter(|word| !crate::parser::is_filler(word))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &[&str]) -> Vec<String> {
        text.iter().map(|line| (*line).to_owned()).collect()
    }

    /// What a block *does*, with the source lines set aside.
    fn kinds(block: &Block) -> Vec<Kind> {
        block.iter().map(|step| step.kind.clone()).collect()
    }

    #[test]
    fn a_flat_spell_is_a_list_of_commands() {
        let program = read(&lines(&["kindle charcoal", "grind sage"]));
        assert_eq!(
            kinds(&program.body),
            [
                Kind::Command("kindle charcoal".to_owned()),
                Kind::Command("grind sage".to_owned()),
            ],
        );
        assert!(program.complaints.is_empty());
    }

    #[test]
    fn wait_names_the_thing_with_the_filler_gone() {
        // `wait for the mortar` names the mortar. Through §6's own filler table,
        // so `the` means here what it means everywhere.
        let program = read(&lines(&["wait for the mortar"]));
        assert_eq!(kinds(&program.body), [Kind::Wait("mortar".to_owned())]);
    }

    #[test]
    fn repeat_encloses_what_it_repeats() {
        let program = read(&lines(&[
            "repeat 3",
            "grind sage",
            "wait for mortar",
            "end",
        ]));
        let Kind::Repeat { times, until, body } = &program.body[0].kind else {
            panic!("not a repeat: {:?}", program.body);
        };
        assert_eq!(*times, Some(3));
        assert_eq!(*until, None, "a counted repeat carries no guard");
        assert_eq!(
            kinds(body),
            [
                Kind::Command("grind sage".to_owned()),
                Kind::Wait("mortar".to_owned()),
            ],
        );
        // The lines are the file's, not the block's — line 1 is the `repeat`.
        assert_eq!(program.body[0].line, 1);
        assert_eq!(body[0].line, 2);
        assert_eq!(body[1].line, 3);
    }

    #[test]
    fn a_bare_repeat_is_unbounded_and_that_is_safe() {
        // Safe because the execution budget bounds a tick — a `repeat` with no
        // `wait` in it spends its budget and stops, wasting itself rather than
        // hanging the game.
        let program = read(&lines(&["repeat", "kindle charcoal", "end"]));
        let Kind::Repeat { times, .. } = &program.body[0].kind else {
            panic!("not a repeat: {:?}", program.body);
        };
        assert_eq!(*times, None);
    }

    #[test]
    fn a_repeat_is_bounded_by_a_number_or_a_question_and_never_both() {
        // **One bound per loop**, refused by name rather than resolved by
        // precedence. `repeat 5 until X` read as `repeat 5` would drop half of
        // what the player wrote in silence.
        let program = read(&lines(&[
            "repeat until the mortar is working",
            "grind sage",
            "end",
        ]));
        let Kind::Repeat { times, until, .. } = &program.body[0].kind else {
            panic!("not a repeat: {:?}", program.body);
        };
        assert_eq!(*times, None, "a guarded repeat carries no count");
        assert!(until.is_some(), "the guard was not read");

        let both = read(&lines(&[
            "repeat 5 until the mortar is working",
            "grind sage",
            "end",
        ]));
        assert!(
            both.complaints
                .iter()
                .any(|complaint| complaint.key == "spell_two_bounds"),
            "two bounds passed unremarked: {:?}",
            both.complaints,
        );
    }

    #[test]
    fn a_bound_the_orb_cannot_read_is_said_rather_than_dropped() {
        // A question that will not parse, and `until` buried where it cannot be
        // read from. Both are a bound the player wrote and the orb cannot use,
        // and an unbounded loop is not a safe thing to guess at.
        for text in ["repeat until xyzzy plugh", "repeat the until room"] {
            let program = read(&lines(&[text, "grind sage", "end"]));
            assert!(
                program
                    .complaints
                    .iter()
                    .any(|complaint| complaint.key == "spell_unreadable_until"),
                "{text:?} passed unremarked: {:?}",
                program.complaints,
            );
        }
    }

    #[test]
    fn until_on_its_own_line_belongs_to_nothing() {
        // It is `repeat`'s argument, never a line of its own — and a real word in
        // the wrong place is answered, not guessed at.
        let program = read(&lines(&["until the mortar is working", "grind sage"]));
        assert!(
            program
                .complaints
                .iter()
                .any(|complaint| complaint.key == "spell_stray_until"),
            "a stray until passed unremarked: {:?}",
            program.complaints,
        );
    }

    #[test]
    fn blocks_nest() {
        let program = read(&lines(&[
            "repeat 2",
            "repeat 3",
            "grind sage",
            "end",
            "kindle charcoal",
            "end",
        ]));
        let Kind::Repeat { body: outer, .. } = &program.body[0].kind else {
            panic!("not a repeat");
        };
        assert_eq!(outer.len(), 2, "{outer:?}");
        assert!(matches!(outer[0].kind, Kind::Repeat { .. }));
        assert!(matches!(outer[1].kind, Kind::Command(_)));
    }

    #[test]
    fn an_unclosed_repeat_is_closed_at_the_end_and_said_once() {
        // §8 leaves exactly one answer: the save cannot refuse (*"bind always
        // succeeds"*) and the cast may not halt (*"scripts always log and never
        // halt"*), so the spell runs as if the player had finished typing it.
        let program = read(&lines(&["repeat 2", "grind sage"]));

        assert!(
            matches!(program.body[0].kind, Kind::Repeat { .. }),
            "the unclosed block was dropped rather than closed",
        );
        assert_eq!(program.complaints.len(), 1);
        assert_eq!(program.complaints[0].line, 1, "it named the wrong line");
        assert_eq!(program.complaints[0].key, "spell_unclosed");
    }

    #[test]
    fn a_stray_end_is_dropped_and_said_once() {
        let program = read(&lines(&["grind sage", "end"]));
        assert_eq!(
            kinds(&program.body),
            [Kind::Command("grind sage".to_owned())]
        );
        assert_eq!(program.complaints.len(), 1);
        assert_eq!(program.complaints[0].line, 2);
        assert_eq!(program.complaints[0].key, "spell_stray_end");
    }

    #[test]
    fn comments_and_blank_lines_are_not_steps() {
        let program = read(&lines(&["# the morning round", "", "grind sage"]));
        assert_eq!(program.body.len(), 1);
    }

    /// Every command a program runs, in order, with a step budget.
    ///
    /// Walks a [`Draft`] rather than a [`Program`], because what is under test
    /// here is the *shape* — `at`, `step_past` and the branch arithmetic — and
    /// none of it looks at a name. Resolution is `compile`'s, and it has its own
    /// tests against a real world.
    fn run(program: &Draft, budget: usize) -> Vec<String> {
        run_with(program, budget, true)
    }

    #[test]
    fn a_repeat_runs_its_body_the_stated_number_of_times() {
        let program = read(&lines(&["repeat 3", "grind sage", "end", "survey"]));
        assert_eq!(
            run(&program, 20),
            ["grind sage", "grind sage", "grind sage", "survey"],
        );
    }

    #[test]
    fn nested_repeats_multiply() {
        let program = read(&lines(&[
            "repeat 2",
            "repeat 3",
            "grind sage",
            "end",
            "end",
        ]));
        assert_eq!(run(&program, 30).len(), 6, "2 × 3 should be six grinds");
    }

    #[test]
    fn a_repeat_once_is_the_body_once() {
        let program = read(&lines(&["repeat 1", "grind sage", "end", "survey"]));
        assert_eq!(run(&program, 20), ["grind sage", "survey"]);
    }

    #[test]
    fn an_unbounded_repeat_never_runs_out() {
        // Safe because the caller's budget bounds a tick — the loop wastes
        // itself rather than hanging the game.
        //
        // One step of the budget goes on **entering** the block, which is what
        // makes that guard real: a body that spent nothing would otherwise spin
        // for ever inside one tick.
        let program = read(&lines(&["repeat", "grind sage", "end"]));
        assert_eq!(run(&program, 5).len(), 4, "5 steps: 1 to enter, 4 to run");
        assert_eq!(run(&program, 40).len(), 39, "and it never finishes");
    }

    #[test]
    fn an_empty_repeat_body_does_not_spin_for_ever() {
        // `repeat 2` with nothing in it must finish rather than loop on an empty
        // block — the path runs off the end immediately and has to keep walking
        // out, which is why closing a block re-checks rather than returning.
        let program = read(&lines(&["repeat 2", "end", "survey"]));
        assert_eq!(run(&program, 20), ["survey"]);
    }

    /// The world these tests pretend to have: every guard says go round, and
    /// every set has exactly two members.
    ///
    /// **Two, not one and not none**, because the arithmetic that can be wrong
    /// is the lap: a set of one enters and leaves without ever exercising
    /// `step_past`'s `Each` arm, and a set of none never enters at all.
    fn a_set_of_two(_: &[usize], popped: Loop) -> bool {
        match popped {
            Loop::Each(index) => index + 1 < 2,
            Loop::Repeat(_) | Loop::Branch => true,
        }
    }

    /// Run a program, taking `holds` for every condition.
    fn run_with(program: &Draft, budget: usize, holds: bool) -> Vec<String> {
        let mut pc = vec![0];
        let mut loops = Vec::new();
        let mut out = Vec::new();
        for _ in 0..budget {
            match at(&program.body, &pc).map(|step| &step.kind) {
                None => break,
                Some(Kind::Repeat { times, body, .. }) => {
                    if body.is_empty() {
                        if !step_past(&program.body, &mut pc, &mut loops, a_set_of_two) {
                            break;
                        }
                    } else {
                        enter(&mut pc, &mut loops, *times);
                    }
                    continue;
                }
                Some(Kind::If {
                    body, otherwise, ..
                }) => {
                    let half = if holds { body } else { otherwise };
                    if half.is_empty() {
                        if !step_past(&program.body, &mut pc, &mut loops, a_set_of_two) {
                            break;
                        }
                    } else {
                        enter_branch(&mut pc, &mut loops, holds);
                    }
                    continue;
                }
                // **A set of two, always**, so the walker needs no world. The
                // runner reads the real one; what is under test here is the
                // path arithmetic, which does not care what a member is called.
                Some(Kind::Each { body, .. }) => {
                    if body.is_empty() {
                        if !step_past(&program.body, &mut pc, &mut loops, a_set_of_two) {
                            break;
                        }
                    } else {
                        enter_each(&mut pc, &mut loops);
                    }
                    continue;
                }
                // **Stepped past, exactly as the runner does.** A definition is
                // not run where it stands, and this walker has to agree about
                // that or the path arithmetic it exists to test would be
                // measured against a different program.
                Some(Kind::Part { .. }) => {
                    if !step_past(&program.body, &mut pc, &mut loops, a_set_of_two) {
                        break;
                    }
                    continue;
                }
                // A leaf here, because this walker has **no frame stack** — what
                // a call does is the runner's, and `tests.rs` drives that
                // through a real `Sim`. Emitting the name keeps a call visible
                // in the shape tests without pretending it was entered.
                Some(Kind::Call { name, args }) => {
                    out.push(format!("{name}({})", args.join(", ")));
                }
                // A leaf here for the same reason a call is: what a fork does is
                // the runner's, and this walker has no cursors of its own.
                Some(Kind::Alongside { name, args }) => {
                    out.push(format!("alongside {name}({})", args.join(", ")));
                }
                Some(Kind::Command(line)) => out.push(line.clone()),
                Some(Kind::Wait(what)) => out.push(format!("wait {what}")),
                Some(Kind::Bide(ticks)) => out.push(format!("bide {ticks}")),
                Some(Kind::Let { name, value }) => out.push(format!("set {name} {value}")),
                Some(Kind::Pull { name, from }) => out.push(format!("pull {name} from {from}")),
            }
            if !step_past(&program.body, &mut pc, &mut loops, a_set_of_two) {
                break;
            }
        }
        out
    }

    #[test]
    fn an_if_reads_as_a_question_about_a_place() {
        let program = read(&lines(&["if the dispensary has sage", "grind sage", "end"]));
        let Kind::If { condition, .. } = &program.body[0].kind else {
            panic!("not an if: {:?}", program.body);
        };
        assert_eq!(
            condition.as_ref(),
            Some(&crate::parser::Condition::Has {
                place: "dispensary".to_owned(),
                thing: "sage".to_owned(),
                // `has sage` *is* `has 1 sage` — the default that keeps every
                // spell written before counting existed meaning what it meant.
                count: crate::parser::Quantity::Count(1),
                bound: crate::parser::Bound::AtLeast,
            }),
        );
    }

    #[test]
    fn an_if_takes_one_half_and_carries_on_past_both() {
        // **The arithmetic most likely to be wrong.** A branch costs two path
        // elements going in, so walking out has to pop two — pop one and the
        // path lands on the *other* half and runs it as well, which looks like
        // an `if` that executes both sides.
        let program = read(&lines(&[
            "if the dispensary has sage",
            "grind sage",
            "else",
            "kindle charcoal",
            "end",
            "survey",
        ]));

        assert_eq!(run_with(&program, 20, true), ["grind sage", "survey"]);
        assert_eq!(run_with(&program, 20, false), ["kindle charcoal", "survey"]);
    }

    #[test]
    fn an_if_with_no_else_skips_to_what_follows() {
        let program = read(&lines(&[
            "if the mortar is idle",
            "grind sage",
            "end",
            "survey",
        ]));
        assert_eq!(run_with(&program, 20, true), ["grind sage", "survey"]);
        assert_eq!(run_with(&program, 20, false), ["survey"]);
    }

    #[test]
    fn an_if_inside_a_repeat_runs_every_turn() {
        let program = read(&lines(&[
            "repeat 3",
            "if the mortar is idle",
            "grind sage",
            "end",
            "end",
        ]));
        assert_eq!(run_with(&program, 40, true).len(), 3);
        assert!(run_with(&program, 40, false).is_empty());
    }

    #[test]
    fn a_question_the_orb_cannot_read_answers_no() {
        // Guessing would be worse than refusing: a condition the player did not
        // write, deciding what their laboratory does while they are elsewhere.
        let program = read(&lines(&["if the moon is gibbous", "grind sage", "end"]));
        let Kind::If { condition, .. } = &program.body[0].kind else {
            panic!("not an if");
        };
        assert!(condition.is_none());
        assert_eq!(program.complaints[0].key, "spell_unreadable_if");
        assert!(run_with(&program, 20, false).is_empty());
    }

    /// **`bide` takes a number and nothing else**, and it did not.
    ///
    /// A bare word compiled to `Delay::Reading`, which is how `bide until` read
    /// its delay off the circle — the menagerie's whole puzzle answered by the
    /// world instead of by the author. Withdrawing the form is what puts the
    /// arithmetic back, and it closes a second hole with it: `bide sage` in the
    /// laboratory was a plausible typo that resolved to an *endless* pile and
    /// bided `u32::MAX`.
    ///
    /// **Nothing anywhere tested `bide` before this** — not the word, not the
    /// count, not the reading form the domain was built on. That is why the
    /// shipped solver could stop compiling with the whole suite green.
    #[test]
    fn bide_takes_a_count_and_a_bare_word_is_refused() {
        for line in ["bide until", "bide sage", "bide"] {
            let program = read(&lines(&[line]));
            let keys: Vec<&str> = program.complaints.iter().map(|c| c.key).collect();
            assert_eq!(keys, ["spell_unreadable_bide"], "{line:?} was accepted");
        }
        let program = read(&lines(&["bide 3"]));
        assert!(program.complaints.is_empty(), "{:?}", program.complaints);
        assert_eq!(program.body[0].kind, Kind::Bide(3));
    }

    #[test]
    fn an_else_with_no_if_is_reported_rather_than_starting_a_branch() {
        let program = read(&lines(&["repeat 2", "grind sage", "else", "end"]));
        assert_eq!(program.complaints[0].key, "spell_stray_else");
        assert_eq!(run_with(&program, 20, true).len(), 2, "the loop still ran");
    }

    /// The same block with every line number flattened, for comparing trees.
    ///
    /// Two spellings of one program sit on different lines by construction, so a
    /// bare `assert_eq!` on the bodies compares the thing that is *supposed* to
    /// differ. This compares the thing that is not.
    fn shape(body: &Block) -> Block {
        body.iter()
            .map(|step| Step {
                line: 0,
                kind: match &step.kind {
                    Kind::Repeat { times, until, body } => Kind::Repeat {
                        times: *times,
                        until: until.clone(),
                        body: shape(body),
                    },
                    Kind::If {
                        condition,
                        body,
                        otherwise,
                    } => Kind::If {
                        condition: condition.clone(),
                        body: shape(body),
                        otherwise: shape(otherwise),
                    },
                    other => other.clone(),
                },
            })
            .collect()
    }

    #[test]
    fn else_if_is_exactly_the_nesting_it_saves_writing() {
        // **The whole claim, as an equality.** `else if` adds no `Kind` and no
        // runner change; it is a desugaring, so the tree it builds must be the
        // one a player gets today by nesting and paying for the `end`s. If these
        // two ever differ, the shorter form has become a second language.
        let ladder = read(&lines(&[
            "if the mortar is idle",
            "grind sage",
            "else if the alembic is idle",
            "distil clarified-draught",
            "else",
            "survey",
            "end",
        ]));
        let by_hand = read(&lines(&[
            "if the mortar is idle",
            "grind sage",
            "else",
            "if the alembic is idle",
            "distil clarified-draught",
            "else",
            "survey",
            "end",
            "end",
        ]));

        // **Compared without line numbers, and that difference is not a defect.**
        // The two spellings occupy different lines and each `Step` carries its
        // own, because §8.1's contract is that the log names the line the player
        // wrote. What must match is the tree.
        assert_eq!(
            shape(&ladder.body),
            shape(&by_hand.body),
            "the ladder is not the nesting",
        );
        assert!(ladder.complaints.is_empty(), "{:?}", ladder.complaints);
        assert!(by_hand.complaints.is_empty(), "{:?}", by_hand.complaints);

        // ...and it is two lines shorter for two rungs, which is the point: the
        // saving is one `end` per rung, and `threading` has twenty-four rungs.
        assert_eq!(7 + 2, 9, "the two spellings above are 7 lines against 9");
    }

    #[test]
    fn a_ladder_of_rungs_closes_with_one_end() {
        // The ward solver's shape. Four rungs nested four deep by hand, closed by
        // four stacked `end`s; here, one.
        let program = read(&lines(&[
            "if the first has 1 or more untried",
            "dial first",
            "else if the second has 1 or more untried",
            "dial second",
            "else if the third has 1 or more untried",
            "dial third",
            "else if the fourth has 1 or more untried",
            "dial fourth",
            "else",
            "wait",
            "end",
        ]));

        assert!(
            program.complaints.is_empty(),
            "one end did not close the ladder: {:?}",
            program.complaints,
        );
        assert_eq!(
            program.body.len(),
            1,
            "a ladder is one step at the top level"
        );

        // Four rungs means four nested `if`s, and the innermost `else` holds the
        // fallback. Walked rather than asserted shallowly, because the bug this
        // guards is a chain that closes one frame too few.
        let mut depth = 0;
        let mut here = &program.body[0].kind;
        while let Kind::If { otherwise, .. } = here {
            depth += 1;
            match otherwise.first().map(|step| &step.kind) {
                Some(next @ Kind::If { .. }) => here = next,
                _ => break,
            }
        }
        assert_eq!(depth, 4, "the ladder is not four rungs deep: {program:?}");
    }

    #[test]
    fn a_ladder_runs_down_to_the_rung_that_answers() {
        // `run_with` answers every question the same way, so `false` walks the
        // whole chain to the final `else` — which is exactly the path a ladder
        // exists to make cheap, and the one an unclosed chain would truncate.
        let program = read(&lines(&[
            "if the mortar is idle",
            "grind sage",
            "else if the alembic is idle",
            "distil clarified-draught",
            "else",
            "survey",
            "end",
        ]));
        assert_eq!(run_with(&program, 20, false), ["survey"]);
        assert_eq!(run_with(&program, 20, true), ["grind sage"]);
    }

    #[test]
    fn an_unreadable_rung_is_said_once_and_the_ladder_still_closes() {
        // A rung the orb cannot read gets the same treatment a plain `if` does —
        // it runs neither half rather than being guessed at — and it must not
        // also swallow the `end`.
        let program = read(&lines(&[
            "if the mortar is idle",
            "grind sage",
            "else if xyzzy plugh",
            "survey",
            "end",
        ]));
        let keys: Vec<&str> = program.complaints.iter().map(|c| c.key).collect();
        assert_eq!(keys, ["spell_unreadable_if"], "{:?}", program.complaints);
    }

    #[test]
    fn an_unclosed_ladder_is_one_complaint_rather_than_one_per_rung() {
        // A chain is one construct to the player, so the missing `end` is one
        // mistake — reported against the `if` at the top of it.
        let program = read(&lines(&[
            "if the mortar is idle",
            "grind sage",
            "else if the alembic is idle",
            "survey",
            "else if the flask_and_rod is idle",
            "status",
        ]));
        let keys: Vec<&str> = program.complaints.iter().map(|c| c.key).collect();
        assert_eq!(keys, ["spell_unclosed"], "{:?}", program.complaints);
        assert_eq!(program.complaints[0].line, 1, "blamed the wrong line");
    }

    #[test]
    fn else_if_without_an_if_is_still_a_stray_else() {
        // The chain may only hang off an `if`. On a `repeat` it is the same
        // misplaced word it always was, and it must not quietly open a branch.
        let program = read(&lines(&[
            "repeat 2",
            "grind sage",
            "else if the mortar is idle",
            "survey",
            "end",
        ]));
        let keys: Vec<&str> = program.complaints.iter().map(|c| c.key).collect();
        assert_eq!(keys, ["spell_stray_else"], "{:?}", program.complaints);
        assert!(
            matches!(program.body[0].kind, Kind::Repeat { .. }),
            "the loop stopped being a loop",
        );
    }

    #[test]
    fn a_misordered_close_is_still_a_program() {
        // `end` closes whatever is innermost, so a misordered pair cannot be
        // detected by counting — the honest consequence of one closing word,
        // taken knowingly for learnability. What must not happen is a panic or a
        // dropped body.
        let program = read(&lines(&["repeat 2", "grind sage", "end", "end"]));
        assert!(matches!(program.body[0].kind, Kind::Repeat { .. }));
        assert_eq!(program.complaints.len(), 1, "the stray end went unreported");
    }
}
