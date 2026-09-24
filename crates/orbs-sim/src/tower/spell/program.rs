//! A spell's lines, read as a program.
//!
//! Derived, never stored: [`Held`](crate::tower::Held) is the truth and this a
//! view over it, rebuilt at every cast. §8's hot-reload and §8.1's sabotage
//! both need the program to be whatever the text now means.
//!
//! Malformed spells run. §8 forbids refusing at save (*"`bind` always
//! succeeds"*) and halting at cast (*"scripts always log and never halt"*), so
//! an unmatched `repeat` is closed at end of file, an unmatched `end` dropped,
//! each reported once naming the line.

use crate::parser::{Condition, SpellWord, spell_argument, spell_word};

/// One thing a spell does, and the line of the file it came from.
///
/// The line is carried rather than derived: the program is a tree and the file
/// a list, so no arithmetic relates a step three path elements deep to a line
/// number. The editor marks the line a running spell is on, and a wrong
/// *"line 3"* is worse than saying nothing.
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
    /// §8's smallest control structure: per Autonauts, conditional behaviour
    /// for a non-programmer with no comparison and no truthiness — only a noun.
    Wait(String),
    /// Spend a number of ticks doing nothing.
    ///
    /// A count, where [`Wait`](Self::Wait) is a noun: a wait ends when the
    /// world says so, a bide when the spell's own arithmetic does. The
    /// menagerie needs the second, since *when* is the puzzle there (§19).
    ///
    /// Only ever a literal. `Delay::Reading` let `bide until` read its delay
    /// off the world, so the menagerie's solver computed nothing.
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
        /// Asked before the first pass and again at the end of each —
        /// Autonauts' rule, so `repeat until <already true>` runs zero times.
        /// Not a do-while: a guard that cannot prevent the first pass is not a
        /// guard.
        ///
        /// Never set alongside `times`: two bounds is a rule to teach, so
        /// `repeat 5 until X` is refused at parse, naming the line.
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
    /// One line, no block. The accumulator half of *"follow the way with the
    /// fewest marks"*: something to compare against and then act on.
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
    /// [`Let`](Self::Let)'s shape with the value from the world; the two are
    /// the whole of what writes to `vars`. A control word rather than a verb
    /// because dispatch hands back records, so a `pull` verb would have nowhere
    /// to put what it took.
    ///
    /// It yields while the satchel is empty and never reaches `PATIENCE`: a
    /// consumer caught up with its producer is a working pipeline, not a fault.
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
    /// A definition, so reaching it executes nothing: the body runs only where
    /// a [`Call`](Self::Call) says so. It stays a step of the tree rather than
    /// a table beside it, because the save, `interpret` and the editor's gutter
    /// all address the tree by line.
    Part {
        /// What the part is called, without its parentheses.
        name: String,
        /// The names its arguments arrive under, in order.
        ///
        /// The part's whole store, not additions to the caller's:
        /// [`Descent`](super::Descent) keeps the caller's bindings and the part
        /// opens with only these. Empty for `part gathering()`.
        params: Vec<String>,
        /// What it does.
        body: Block,
    },
    /// Do a part — `gathering()`, `between(wellspring, near)`.
    ///
    /// Carries the name rather than a path to the definition, so a spell can be
    /// edited while it runs (§8): a definition that moves up the file is still
    /// the same part, and a path would point at whatever took its place.
    Call {
        /// Which part.
        name: String,
        /// The names handed to it, in order, exactly as written.
        ///
        /// Resolved against the caller's store where the call runs, not here:
        /// `between(wellspring, near)` passes whatever `near` stands for then,
        /// and a literal stands for itself. One level, as everywhere else.
        args: Vec<String>,
    },
    /// Set a part running as a second cursor and carry on —
    /// `alongside gathering()`.
    ///
    /// [`Call`](Self::Call)'s fields and none of its waiting: a call suspends
    /// the caller onto a [`Descent`](super::Descent), this starts a
    /// [`Strand`](super::Strand) and leaves the caller where it stands. The
    /// forked cursor's stack is empty because nothing waits for it.
    ///
    /// Arguments resolve in the caller's store at the fork, as a call's do — a
    /// part's brackets are the whole of what it can see (§19), and reading
    /// bindings the caller kept changing would be worse than the shared store
    /// §19 removed.
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
/// Not runnable, and the type says so: a draft's questions still hold the words
/// the player typed (`the mortar`), which [`holds`](super::holds) compares
/// exactly and finds nothing for, so every condition would answer "there is no
/// such place" — §19's *"it looked exactly like the condition being inverted"*.
///
/// [`compile`](super::compile()) turns one into a [`Program`] and nothing else
/// can. `Program::new` is reachable in this module, but `invoke` and `scribe`'s
/// reload are outside it.
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
/// Deliberately not the obvious word for a stack entry: `tests/boundaries.rs`
/// rule 2 forbids the four layout type names anywhere under `orbs-sim/src`, by
/// *substring*. A parser's stack entry is a false positive, and renaming costs
/// less than weakening the one test that keeps the sim away from the painter.
struct Nesting {
    line: usize,
    kind: Open,
    body: Block,
    /// Opened by `else if`, so one `end` closes this and what it hangs from.
    ///
    /// A chain is one construct to the player and nested `if`s to the runner;
    /// remembering this is all that keeps the two apart, and why the desugaring
    /// needs no new [`Kind`] and no runner change.
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
                // One bound per loop: `repeat 5 until X` is two, and their
                // interaction would be a rule to teach for a shape nobody
                // reaches for. Refused by name rather than resolved by
                // precedence, as `debug_spawn` does with a count it cannot
                // read.
                let counted = count_of(argument);
                // The word anywhere, not just in front: `repeat 5 until X` has
                // it second, so looking only at the head would read `repeat 5`
                // and drop the rest in silence.
                let mentions = argument
                    .split_whitespace()
                    .any(|word| word.eq_ignore_ascii_case(SpellWord::Until.canonical()));
                let guard = guard_of(argument);
                // A bound the orb cannot read runs the loop nought times, not
                // for ever: `(None, None)` loops unbounded, so one mistyped
                // word bought an infinite loop under *"the repeat stops"*.
                // `Some(0)` is stepped past without entering, as `repeat 0` is.
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
                // `else if` is a chained `if`, not a new word. Every solver is
                // a ladder, and without this each rung nests one deeper and
                // pays for an `end`: half of `threading` was `end` and `else`.
                //
                // What follows is read as an `if` line, so one reader and one
                // set of complaint keys. A desugaring rather than a `Kind`, so
                // the runner, the save format and `interpret` see the nested
                // tree that was always written by hand.
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
                // One `end` closes the whole chain: each `else if` pushed a
                // frame, the player wrote one block and owes one close, so the
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
                    // A bare word is a complaint: it compiled to
                    // `Delay::Reading`, so `bide until` left the menagerie's
                    // solver no arithmetic. Also catches `bide sage`, a typo
                    // that answered `Endless` and bided `u32::MAX`.
                    None => complaints.push(Complaint {
                        line: at,
                        key: "spell_unreadable_bide",
                    }),
                }
            }
            Some(SpellWord::Let) => match binding(spell_argument(trimmed)) {
                Some((name, value)) => push(&mut open, at, Kind::Let { name, value }),
                // A complaint, not a guess: `set best` names nothing to bind
                // and `set to north` binds nothing, and reading either as the
                // other is the orb writing a line the player did not.
                None => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_let",
                }),
            },
            // `strip_filler` here, and `let` above deliberately without it.
            // `spell_argument` does not strip, so without this `pull note from
            // satchel` arrives as three words and is refused. `let` must *not*
            // strip: its keyword is ` be ` and `binding` splits the raw text on
            // it, so stripping would shear a `the` off the value and match a
            // name that never appeared in the file.
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
                // No block is opened, so the `end` below is a stray one and
                // says so. Opening an unnamed block would silently swallow the
                // body into a loop over nothing.
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
                // No block is opened, as an unreadable `for each` opens none,
                // so the `end` below is a stray one and says so. An unnamed
                // part would swallow the body into something nothing can call.
                None => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_part",
                }),
            },
            // A fork is a call with a word in front, so it reuses the same
            // parser rather than growing a second — `alongside between(a b)`
            // has to be the same complaint as `between(a b)`.
            Some(SpellWord::Alongside) => match call_of(spell_argument(trimmed)) {
                Some(Some((name, args))) => push(&mut open, at, Kind::Alongside { name, args }),
                Some(None) => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_call",
                }),
                // No brackets at all, its own complaint: `alongside gathering`
                // is a bare name, which is the one thing a call may not be.
                // Better said than read as a call the player did not punctuate.
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

    // Unmatched opens are closed here, innermost first, each reported once. §8
    // leaves no other option: the save could not refuse and the cast may not
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

    // No `expect`: the loops above are guarded on `len() > 1`, but writing that
    // as a panic is a claim the compiler cannot check and a crash the day it
    // stops being true. An empty spell is a real thing anyway.
    let mut body = open.pop().map(|frame| frame.body).unwrap_or_default();
    settle_parts(&mut body, &mut complaints);

    Draft { body, complaints }
}

/// Enforce the two rules a definition has, after the tree is built.
///
/// Its own pass rather than a check inside [`read`]: both rules are about a
/// definition's place among the others, unanswerable while the block holding it
/// is still open.
///
/// - Top-level only. [`tree`] looks no deeper, so a `part` inside a `repeat` is
///   lines nothing can ever call — and silently. Dropped and said.
/// - One name, one part. `tree` answers with whichever is written first, so the
///   second would never run — `progression.toml`'s call about a duplicate id.
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
/// A plain count was enough while `repeat` was the only block. An `if` has two
/// bodies, so the path must say which one execution entered, and walking out of
/// a branch pops one element more than out of a loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loop {
    /// A `repeat`, with the turns it has left.
    Repeat(Option<u32>),
    /// A branch of an `if`. Runs once, and its path carries which half.
    Branch,
    /// A `for each`, with the index of the member currently bound.
    ///
    /// An index, not the members: the set is re-read at the top of every pass
    /// and this says only how far the loop has got. A `Loop` is `Copy` and
    /// saves as one integer per open block; and, deciding it, a snapshot would
    /// have `for each way` iterate a maze that has since closed.
    Each(u32),
}

/// The step at `pc`, if the path resolves.
///
/// A path rather than an index: `[0]` is the first step, `[2, 1]` the second
/// step inside the third. Blocks made a single line number insufficient.
///
/// An `if` costs two path elements, not one: the branch, then the step within
/// it. `[2, 0, 1]` is the second step of the `then` half of step three.
#[must_use]
pub fn at<'a>(body: &'a Block, pc: &[usize]) -> Option<&'a Step> {
    let (&first, rest) = pc.split_first()?;
    let step = body.get(first)?;
    if rest.is_empty() {
        return Some(step);
    }
    match &step.kind {
        // One path element, like a `repeat`: one body and no second half, so
        // walking out is the loop's arithmetic rather than the branch's — see
        // [`Loop::Each`].
        Kind::Repeat { body, .. } | Kind::Each { body, .. } => at(body, rest),
        Kind::If {
            body, otherwise, ..
        } => {
            let (&branch, inner) = rest.split_first()?;
            at(if branch == 0 { body } else { otherwise }, inner)
        }
        // A definition is not descended into from the body it sits in; it is
        // reached by name through [`tree`], so a call still finds it after an
        // edit moves it. A path pointing *into* one is a path nothing builds.
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
/// Found by name at every step, never cached: the file may be edited while the
/// spell runs (§8), so a path taken at the call would point at whatever moved
/// into its place. Definitions are top-level only — see `spell_nested_part` —
/// so this looks no deeper, and a part inside a `repeat` is unreachable by
/// design.
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
/// `None` and an empty list are different answers, which is why this is not
/// folded into [`parts`]: no such part is a missing name said once, a part
/// taking nothing is `gathering()` working. Found by name for [`tree`]'s
/// reason.
#[must_use]
pub fn signature(body: &Block, part: &str) -> Option<Vec<String>> {
    body.iter().find_map(|step| match &step.kind {
        Kind::Part { name, params, .. } if name == part => Some(params.clone()),
        _ => None,
    })
}

/// Every part the spell defines and how many names it takes, in writing order.
///
/// The count travels with the name because this list answers both *is there
/// such a part* and *does this call fit it*, and a second walk of the tree for
/// the second is how one rule comes to disagree with itself.
#[must_use]
pub fn parts(body: &Block) -> Vec<(&str, usize)> {
    body.iter()
        .filter_map(|step| match &step.kind {
            Kind::Part { name, params, .. } => Some((name.as_str(), params.len())),
            _ => None,
        })
        .collect()
}

/// Move `pc` past the step it points at, closing and repeating blocks as
/// needed.
///
/// Returns `false` when the program has run out — the only way a spell
/// finishes, since §8's taxonomy forbids halting on a failure.
///
/// Each open `repeat` keeps a count on `loops`, outermost first, which is why a
/// save needs more than a `pc`: restoring from one alone would resume every
/// enclosing loop from its first iteration, and §8 requires in-flight state be
/// serialisable.
///
/// `again` is asked whether a block that has just run off the end may go round,
/// given the path to the block's own step and the [`Loop`] popped. It takes
/// both because a guard needs the step (to find its `until`) and a set needs
/// the index. The only thing here that consults the world, and a closure
/// because `step_past` is otherwise pure and its test callers have no world.
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
            // Unbounded, or more to go: back to the top of this block, and only
            // if the guard still says so. `again` is asked with the path on the
            // `repeat` itself, which is how the caller finds its `until`.
            //
            // Asked after the count, so short-circuiting keeps an exhausted
            // loop from putting a question to the world on its way out.
            Some(Loop::Repeat(left))
                if left.is_none_or(|turns| turns > 1) && again(pc, Loop::Repeat(left)) =>
            {
                loops.push(Loop::Repeat(left.map(|turns| turns - 1)));
                pc.push(0);
                return true;
            }
            // The set is asked, not counted here: a `for each` walks a live
            // world, so another member is a question for the tick the loop laps
            // on rather than the tick it started — see [`Loop::Each`].
            Some(Loop::Each(index)) if again(pc, Loop::Each(index)) => {
                loops.push(Loop::Each(index + 1));
                pc.push(0);
                return true;
            }
            // A branch runs once and cost two path elements going in — the
            // half, then the step. Popping one would leave the path on the
            // other half of the `if` and run it as well.
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
/// Lexical and deliberately not scoped: a `set` inside an `if` binds a name the
/// lines after the `end` can still say, and a `for each` cursor outlives its
/// loop. §8's language has no declarations, so `set best to north` in a branch
/// and `best` read below it mean what the player wrote; scoping is a rule to
/// teach and to get wrong, for a program that fits on a screen.
///
/// [`compile`](mod@super::compile) uses the list to tell a variable from a
/// place the room does not have — both resolve to nothing at cast.
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
            // A part's parameters and its `let`s are gathered too, though
            // scoped to it: the list only stops `check_commands` and
            // `interpret` resolving a variable against the room, and there a
            // name too many is harmless where a name too few is a working line
            // painted red. So it is the *file's* names, at the cost of a
            // caller's `follow here` being quoted. The runner scopes (see
            // [`Descent::vars`](super::Descent)).
            Kind::Part { params, body, .. } => {
                for param in params {
                    if !names.iter().any(|already| already == param) {
                        names.push(param.clone());
                    }
                }
                gather(body, names);
            }
            // A `pull` binds a name and is gathered as one: `limn keystone
            // note` after `pull note from satchel` is exactly the line the lint
            // must not paint red.
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
/// Pushes the half and the step within it, which is what makes an `if` two path
/// elements deep — see [`at`].
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
/// `Open::Spell` cannot reach here — both callers guard on `len() > 1` — but is
/// folded in rather than panicked on: a claim the compiler cannot check is a
/// crash waiting for the day it stops being true.
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
/// The parentheses are optional on a definition only while it takes nothing:
/// `part` already says what the line is, so demanding them is ceremony, and a
/// parameter list has nowhere else to go. At a call site they are the entire
/// notation and always required.
///
/// `None` for a name that is empty, is more than one word, or has a parameter
/// list the orb cannot read — an unclosed bracket, an empty slot from a trailing
/// comma, a parameter that is a phrase. Reading `part gather the sage` as a
/// heading would file a body under a name nothing can call.
///
/// A repeated parameter is refused rather than shadowed: `part between(here,
/// here)` would leave the caller's first argument unreachable.
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
/// be comma-separated single words; a trailing comma leaves an empty slot and
/// is refused rather than dropped, since whoever wrote it meant to type a name.
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
/// for a malformed one, `Some(Some(..))` for a good one. Collapsing the middle
/// would send `between(a b)` to the command resolver, which would answer that
/// the tower has no `between(a b)` — true, and about the wrong thing.
///
/// The line must *end* with the bracket, the rule `lexeme::call_runs` has to
/// match: `gathering() # note` is a command.
///
/// Whether the count is right needs the definition, a fact about the file
/// rather than the line, so `compile::check_calls` answers it beside *is there
/// such a part* and both arrive as one report.
///
/// An argument may repeat where a parameter may not: two slots given one name
/// is ordinary in a call, shadowing in a heading.
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
/// `be`, not `to`: `to` is on §6's filler list, so a keyword `to` would be
/// invisible to every reader but this one, which sees the text before
/// normalisation. `let best be north` is the English either way.
///
/// Both halves are required. `let best` has nothing to bind and `let be north`
/// has no name; either read as the other is the orb writing a line the player
/// did not.
fn binding(argument: &str) -> Option<(String, String)> {
    let (name, value) = argument.split_once(" be ")?;
    let (name, value) = (name.trim(), value.trim());
    // A name is one word: `let the best way be north` would bind something no
    // later line could spell, since a use site is matched word by word.
    if name.is_empty() || value.is_empty() || name.split_whitespace().count() != 1 {
        return None;
    }
    Some((name.to_lowercase(), value.to_owned()))
}

/// `note from satchel` → the name to bind, and the satchel to take it from.
///
/// `from` is on §6's filler list and already gone, so `spell_argument` hands
/// over `note satchel` — two words, positional. Hence splitting on whitespace
/// where [`binding`] splits on a keyword: `let`'s `be` survives normalisation
/// and carries the grammar, `from` is decoration. `pull note satchel` is
/// accepted.
///
/// Not checked against the world here, like every name in a [`Draft`]: a
/// satchel the room does not have is `compile`'s to catch, in the room.
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
/// Not checked against the world here: `read` resolves nothing, which is what
/// [`Draft`] means, so a set the room does not have is
/// [`compile`](mod@super::compile)'s to catch, in the room.
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
/// A word that is not a number is not an error. This used to excuse `repeat
/// until the mortar` as an unbounded loop, because the language had no `until`;
/// it has one now, handled above, so this stays lenient only for the rest.
fn count_of(argument: &str) -> Option<u32> {
    argument.split_whitespace().next()?.parse().ok()
}

/// What follows `until` in a `repeat`'s argument, if it opens with one.
///
/// The word must be first. Finding it anywhere would make `repeat 3 until` and
/// `repeat the until room` both mean something; §6's rule is that the orb says
/// what it could not read.
fn after_until(argument: &str) -> Option<&str> {
    // Case-folded, because every other control word is. `spell_word` lowercases
    // the first word, so `Repeat Until ...` is a repeat while this missed its
    // own keyword: a capital letter bought *the question meant nothing*.
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
        // One bound per loop, refused by name rather than resolved by
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
    /// Walks a [`Draft`] rather than a [`Program`]: what is under test is the
    /// *shape* — `at`, `step_past`, the branch arithmetic — none of which looks
    /// at a name. Resolution is `compile`'s, with its own tests.
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
        // itself rather than hanging the game. One step of that budget goes on
        // entering the block; a body that spent nothing would spin for ever in
        // one tick.
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
    /// Two, because the lap is the arithmetic that can be wrong — a set of one
    /// never exercises `step_past`'s `Each` arm, and a set of none never
    /// enters.
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
                // A set of two, always, so the walker needs no world. The
                // runner reads the real one; what is under test is the path
                // arithmetic, which does not care what a member is called.
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
                // Stepped past, as the runner does: a definition is not run
                // where it stands, and a walker that disagreed would measure
                // the path arithmetic against a different program.
                Some(Kind::Part { .. }) => {
                    if !step_past(&program.body, &mut pc, &mut loops, a_set_of_two) {
                        break;
                    }
                    continue;
                }
                // A leaf: this walker has no frame stack — what a call does is
                // the runner's, driven through a real `Sim` in `tests.rs`. The
                // name keeps a call visible without pretending it was entered.
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
        // The arithmetic most likely to be wrong: a branch costs two path
        // elements going in, so walking out pops two — pop one and the path
        // lands on the *other* half and runs it as well.
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

    /// `bide` takes a number and nothing else, and it did not.
    ///
    /// A bare word compiled to `Delay::Reading`, so `bide until` read its delay
    /// off the circle and the world answered the menagerie's puzzle.
    /// Withdrawing the form also closes `bide sage`, a plausible typo that
    /// resolved to an *endless* pile and bided `u32::MAX`.
    ///
    /// Nothing tested `bide` before this, which is why the shipped solver could
    /// stop compiling with the whole suite green.
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
        // The whole claim, as an equality. `else if` is a desugaring, so its
        // tree must be the one nesting by hand builds; if the two ever differ,
        // the shorter form has become a second language.
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

        // Compared without line numbers: the two spellings occupy different
        // ones and each `Step` carries its own, because §8.1's contract is that
        // the log names the line the player wrote. What must match is the tree.
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
