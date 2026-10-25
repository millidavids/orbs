//! A spell's lines, read as a program.
//!
//! Derived, never stored: [`Held`](crate::tower::Held) is the truth and this a
//! view rebuilt at every cast, because §8's hot-reload and §8.1's sabotage both
//! need it to be whatever the text now means.
//!
//! Malformed spells run: §8 forbids refusing at save and halting at cast, so an
//! unmatched `repeat` is closed at end of file and an unmatched `end` dropped,
//! each reported once naming the line.

use crate::parser::{Condition, SpellWord, spell_argument, spell_word};

/// One thing a spell does, and the line of the file it came from.
///
/// The line is carried, not derived: a tree's path has no arithmetic to a line
/// number, and a wrong *"line 3"* is worse than none.
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
    /// Hold until something the spell names happens — §8's smallest control
    /// structure, a noun and no comparison.
    Wait(String),
    /// Spend a number of ticks doing nothing.
    ///
    /// A literal count, where a wait is a noun: the menagerie needs the spell's
    /// own arithmetic to decide *when* (§19).
    Bide(u32),
    /// Do the enclosed steps, `times` of them, or for ever if `None`.
    Repeat {
        /// How many times, or `None` for an unbounded loop — safe, because the
        /// execution budget bounds a tick.
        times: Option<u32>,
        /// The question that ends it, asked before the first pass too, so
        /// `repeat until <already true>` runs zero times.
        ///
        /// Never set alongside `times`: `repeat 5 until X` is refused at parse.
        until: Option<crate::parser::Condition>,
        /// What to do each time.
        body: Block,
    },
    /// Do one branch or the other, depending on the tower.
    If {
        /// The question, or `None` if the orb could not read it — then neither
        /// branch runs, and the line is reported once. A guess would be a
        /// condition the player did not write.
        condition: Option<Condition>,
        /// What to do when it holds.
        body: Block,
        /// What to do when it does not. Empty unless there is an `else`.
        otherwise: Block,
    },
    /// Bind a name to a place, for the lines after it — `let best be north`.
    Let {
        /// The word the spell will use.
        name: String,
        /// What it stands for — a name, resolved against the room at cast, unless
        /// it is itself a name the spell binds.
        value: String,
    },
    /// Take the oldest name out of a satchel and bind it —
    /// `pull note from satchel`.
    ///
    /// A control word rather than a verb, because dispatch hands back records and
    /// a verb would have nowhere to put what it took.
    Pull {
        /// The word the spell will use for whatever comes out.
        name: String,
        /// Which satchel — a place, resolved against the room at cast.
        from: String,
    },
    /// Do the enclosed steps once for each member of a set — `for each way`.
    ///
    /// The cursor is bound to `group` itself; [`SpellWord::For`] says why it is
    /// not `it`.
    Each {
        /// The set to walk, and the name the cursor takes.
        group: String,
        /// What to do for each of them.
        body: Block,
    },
    /// A named run of lines — `part gathering()`, `part between(here, there)`.
    ///
    /// Reaching it runs nothing. It stays a step of the tree, not a table beside
    /// it, because the save, `interpret` and the editor's gutter address by line.
    Part {
        /// What the part is called, without its parentheses.
        name: String,
        /// The names its arguments arrive under, in order: the part's whole
        /// store, since a [`Descent`](super::Descent) keeps the caller's.
        params: Vec<String>,
        /// What it does.
        body: Block,
    },
    /// Do a part — `gathering()`, `between(wellspring, near)`.
    ///
    /// By name, not path, so a definition moved by an edit while the spell runs
    /// is still the same part (§8).
    Call {
        /// Which part.
        name: String,
        /// The names handed to it, in order, exactly as written — resolved
        /// where the call runs, one level deep.
        args: Vec<String>,
    },
    /// Set a part running as a second cursor and carry on —
    /// `alongside gathering()`.
    ///
    /// A call that starts a [`Strand`](super::Strand) instead of suspending the
    /// caller. Arguments resolve at the fork, as a call's do (§19).
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
/// Not runnable, and the type says so: its questions still hold the words the
/// player typed, which [`holds`](super::holds) finds nothing for, so every
/// condition would read as inverted. Only [`compile`](super::compile()) makes a
/// [`Program`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Draft {
    /// What it does.
    pub body: Block,
    /// What the orb had to fix, if anything. Never a refusal.
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
    /// The spell itself. Never popped, which makes a stray `end` a complaint
    /// rather than a panic.
    Spell,
    Repeat {
        times: Option<u32>,
        until: Option<Condition>,
    },
    If {
        condition: Option<Condition>,
        /// The `then` branch, once an `else` has moved it aside.
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
/// Not the obvious name, which `tests/boundaries.rs` forbids by substring.
struct Nesting {
    line: usize,
    kind: Open,
    body: Block,
    /// Opened by `else if`, so one `end` closes this and what it hangs from —
    /// all that keeps a chain one construct to the player and nested `if`s to
    /// the runner.
    chained: bool,
}

/// Read `lines` as a shape, resolving nothing.
///
/// `pub(super)`: the way in from outside is [`compile`](mod@super::compile).
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
                // One bound per loop, refused by name rather than resolved by
                // precedence.
                let counted = count_of(argument);
                // Anywhere, not just in front: `repeat 5 until X` has it second.
                let mentions = argument
                    .split_whitespace()
                    .any(|word| word.eq_ignore_ascii_case(SpellWord::Until.canonical()));
                let guard = guard_of(argument);
                // An unreadable bound runs nought times, not for ever: one
                // mistyped word must not buy an infinite loop.
                let refused = (Some(0), None);
                let (times, until) = if mentions && counted.is_some() {
                    complaints.push(Complaint {
                        line: at,
                        key: "spell_two_bounds",
                    });
                    refused
                } else if mentions && guard.is_none() {
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
                    // Unreadable, and not guessed at: the branch never runs.
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
                // `else if` is a desugaring, not a word: the runner, the save and
                // `interpret` see the nesting a hand would write, one `end` saved
                // per rung of every solver's ladder.
                let tail = spell_argument(trimmed).trim();
                let chaining = spell_word(tail) == Some(SpellWord::If);

                // Only an `if` has an other half.
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
                    complaints.push(Complaint {
                        line: at,
                        key: "spell_stray_end",
                    });
                    continue;
                }
                // One `end` closes the whole chain: unwind while the frame just
                // shut was chained.
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
                    // A word is refused: `bide until` read its delay off the
                    // world, and `bide sage` bided `u32::MAX`.
                    None => complaints.push(Complaint {
                        line: at,
                        key: "spell_unreadable_bide",
                    }),
                }
            }
            Some(SpellWord::Let) => match binding(spell_argument(trimmed)) {
                Some((name, value)) => push(&mut open, at, Kind::Let { name, value }),
                // `set best` binds nothing and `set to north` names nothing.
                None => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_let",
                }),
            },
            // `strip_filler` here and not for `let`: `pull` is positional, while
            // `binding` splits the raw text on ` be ` and stripping would shear a
            // `the` off the value.
            Some(SpellWord::Pull) => match pulled(&strip_filler(spell_argument(trimmed))) {
                Some((name, from)) => push(&mut open, at, Kind::Pull { name, from }),
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
                // No block is opened, so its `end` is said as stray rather than
                // a loop over nothing swallowing the body.
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
                // No block, as for an unreadable `for each`.
                None => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_part",
                }),
            },
            // The call parser, so `alongside between(a b)` is the same complaint
            // as `between(a b)`.
            Some(SpellWord::Alongside) => match call_of(spell_argument(trimmed)) {
                Some(Some((name, args))) => push(&mut open, at, Kind::Alongside { name, args }),
                Some(None) => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_call",
                }),
                // A bare name, the one thing a call may not be.
                None => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_alongside",
                }),
            },
            // A call is punctuation, not a control word.
            None if let Some(called) = call_of(trimmed) => match called {
                Some((name, args)) => push(&mut open, at, Kind::Call { name, args }),
                // A bare call would drop a word the player wrote.
                None => complaints.push(Complaint {
                    line: at,
                    key: "spell_unreadable_call",
                }),
            },
            None => push(&mut open, at, Kind::Command(trimmed.to_owned())),
        }
    }

    // Unmatched opens closed innermost first, each reported once: the spell runs
    // as if the player had finished typing it.
    while open.len() > 1 {
        let Some(frame) = open.pop() else {
            break;
        };
        // A chain's complaint belongs to the `if` at its top, once.
        if !frame.chained {
            complaints.push(Complaint {
                line: frame.line,
                key: "spell_unclosed",
            });
        }
        close(&mut open, frame);
    }

    // No `expect`: an empty spell is a real thing.
    let mut body = open.pop().map(|frame| frame.body).unwrap_or_default();
    settle_parts(&mut body, &mut complaints);

    Draft { body, complaints }
}

/// Enforce a definition's two rules, which need the finished tree.
///
/// Top-level only, because [`tree`] looks no deeper and a nested `part` is lines
/// nothing can call. One name, one part, because `tree` answers with the first.
/// Each offender is dropped and said.
fn settle_parts(body: &mut Block, complaints: &mut Vec<Complaint>) {
    // In writing order, so the first definition survives — `tree`'s answer.
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
/// The kind matters: an `if` has two bodies, so leaving a branch pops one path
/// element more than leaving a loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loop {
    /// A `repeat`, with the turns it has left.
    Repeat(Option<u32>),
    /// A branch of an `if`. Runs once, and its path carries which half.
    Branch,
    /// A `for each`, with the index of the member currently bound.
    ///
    /// An index, not a snapshot of the members: the set is re-read every pass,
    /// so a loop over a maze that has closed stops.
    Each(u32),
}

/// The step at `pc`, if the path resolves.
///
/// An `if` costs two path elements, the branch and the step within it:
/// `[2, 0, 1]` is the second step of the `then` half of step three.
#[must_use]
pub fn at<'a>(body: &'a Block, pc: &[usize]) -> Option<&'a Step> {
    let (&first, rest) = pc.split_first()?;
    let step = body.get(first)?;
    if rest.is_empty() {
        return Some(step);
    }
    match &step.kind {
        Kind::Repeat { body, .. } | Kind::Each { body, .. } => at(body, rest),
        Kind::If {
            body, otherwise, ..
        } => {
            let (&branch, inner) = rest.split_first()?;
            at(if branch == 0 { body } else { otherwise }, inner)
        }
        // A definition is reached by name through [`tree`], never descended
        // into from where it sits.
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
/// Found by name at every step, never cached, because the file may be edited
/// while the spell runs (§8). Top level only — see `spell_nested_part`.
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
/// `None` and empty are different answers: no such part, against a part that
/// takes nothing.
#[must_use]
pub fn signature(body: &Block, part: &str) -> Option<Vec<String>> {
    body.iter().find_map(|step| match &step.kind {
        Kind::Part { name, params, .. } if name == part => Some(params.clone()),
        _ => None,
    })
}

/// Every part the spell defines and how many names it takes, in writing order.
///
/// The count travels with the name so *is there such a part* and *does this call
/// fit it* come from one walk.
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
/// Returns `false` when the program has run out, the only way a spell finishes.
/// `again` is asked whether a block that just ran off the end goes round, given
/// the path to its step and the [`Loop`] popped; a closure, so this stays pure
/// and its tests need no world.
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

    // Walk out of every block the path has just run off the end of.
    while at(body, pc).is_none() {
        if pc.len() == 1 {
            return false;
        }
        pc.pop();
        match loops.pop() {
            // The count first, so an exhausted loop asks the world nothing.
            Some(Loop::Repeat(left))
                if left.is_none_or(|turns| turns > 1) && again(pc, Loop::Repeat(left)) =>
            {
                loops.push(Loop::Repeat(left.map(|turns| turns - 1)));
                pc.push(0);
                return true;
            }
            // Asked on the tick the loop laps, because it walks a live world.
            Some(Loop::Each(index)) if again(pc, Loop::Each(index)) => {
                loops.push(Loop::Each(index + 1));
                pc.push(0);
                return true;
            }
            // A branch cost two elements going in; popping one would run the
            // other half as well.
            Some(Loop::Branch) => {
                pc.pop();
                if let Some(last) = pc.last_mut() {
                    *last += 1;
                }
            }
            // Finished: step past the block and check again, since the one
            // containing it may have ended too.
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
/// Lexical, not scoped: a `set` in a branch is still readable after its `end`,
/// because scoping is a rule to teach for a program that fits on a screen.
/// [`compile`](mod@super::compile) uses it to tell a variable from a missing
/// place.
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
            // A part's names too, though the runner scopes them: here a name too
            // many is harmless, and a name too few paints a working line red.
            Kind::Part { params, body, .. } => {
                for param in params {
                    if !names.iter().any(|already| already == param) {
                        names.push(param.clone());
                    }
                }
                gather(body, names);
            }
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

/// Descend into a `for each`, at its first member — one path element, like a
/// `repeat`.
pub fn enter_each(pc: &mut Vec<usize>, loops: &mut Vec<Loop>) {
    loops.push(Loop::Each(0));
    pc.push(0);
}

/// Descend into one half of an `if`: the half, then the step — see [`at`].
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
/// `Open::Spell` cannot arrive, but is folded in rather than panicked on.
fn close(open: &mut [Nesting], frame: Nesting) {
    let line = frame.line;
    let kind = match frame.kind {
        Open::Repeat { times, until } => Kind::Repeat {
            times,
            until,
            body: frame.body,
        },
        Open::If { condition, taken } => match taken {
            // After an `else`, what accumulated is the other half.
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
/// Brackets are optional on a definition that takes nothing; at a call site they
/// are the whole notation. `None` for anything not one name with a readable list
/// — a phrase would file a body under a name nothing can call. A repeated
/// parameter is refused, since it would make an argument unreachable.
fn part_signature(argument: &str) -> Option<(String, Vec<String>)> {
    let trimmed = argument.trim();
    let Some((head, rest)) = trimmed.split_once('(') else {
        // `part gathering`: one word, no brackets.
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
/// A trailing comma's empty slot is refused: whoever wrote it meant a name.
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
/// `None` is not a call, `Some(None)` a malformed one: collapsing them would send
/// `between(a b)` to the command resolver. The line must end with the bracket,
/// `lexeme::call_runs`'s rule. Arity is a fact about the file, so
/// `compile::check_calls` answers it. An argument may repeat; a parameter may not.
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
/// [`parameters`]'s rule minus distinctness.
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
/// `be`, not `to`: `to` is §6 filler, invisible to every other reader. Both
/// halves are required.
fn binding(argument: &str) -> Option<(String, String)> {
    let (name, value) = argument.split_once(" be ")?;
    let (name, value) = (name.trim(), value.trim());
    // One word, because a use site is matched word by word.
    if name.is_empty() || value.is_empty() || name.split_whitespace().count() != 1 {
        return None;
    }
    Some((name.to_lowercase(), value.to_owned()))
}

/// `note from satchel` → the name to bind, and the satchel to take it from.
///
/// Positional, because `from` is §6 filler and already gone, so `pull note
/// satchel` is accepted. The satchel is `compile`'s to check, in the room.
fn pulled(argument: &str) -> Option<(String, String)> {
    let mut words = argument.split_whitespace();
    let name = words.next()?;
    let from = words.next()?;
    if words.next().is_some() {
        return None;
    }
    Some((name.to_lowercase(), from.to_owned()))
}

/// `each way` → the set to walk.
///
/// The particle is required and carries nothing — see
/// [`SpellWord::particle`](crate::parser::SpellWord::particle). The set is
/// [`compile`](mod@super::compile)'s to check, in the room.
fn walked(argument: &str) -> Option<String> {
    let mut words = argument.split_whitespace();
    let particle = SpellWord::For.particle()?;
    if !words.next()?.eq_ignore_ascii_case(particle) {
        return None;
    }
    let group = words.next()?;
    // `for each way and socket` is two sets, a shape the language does not have.
    if words.next().is_some() {
        return None;
    }
    Some(group.to_lowercase())
}

/// `repeat 3` → `Some(3)`; a bare `repeat` → `None`.
fn count_of(argument: &str) -> Option<u32> {
    argument.split_whitespace().next()?.parse().ok()
}

/// What follows `until` in a `repeat`'s argument, if it opens with one.
///
/// First only, so `repeat the until room` is something the orb says it could
/// not read (§6).
fn after_until(argument: &str) -> Option<&str> {
    // Case-folded, as `spell_word` folds every other control word.
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

/// Drop §6's filler words from what a `wait` was given, through the parser's own
/// table.
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
        // The file's lines, not the block's.
        assert_eq!(program.body[0].line, 1);
        assert_eq!(body[0].line, 2);
        assert_eq!(body[1].line, 3);
    }

    #[test]
    fn a_bare_repeat_is_unbounded_and_that_is_safe() {
        let program = read(&lines(&["repeat", "kindle charcoal", "end"]));
        let Kind::Repeat { times, .. } = &program.body[0].kind else {
            panic!("not a repeat: {:?}", program.body);
        };
        assert_eq!(*times, None);
    }

    #[test]
    fn a_repeat_is_bounded_by_a_number_or_a_question_and_never_both() {
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
        // read from.
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
    /// Over a [`Draft`]: what is under test is the shape, which looks at no name.
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
        // One step of the budget goes on entering the block.
        let program = read(&lines(&["repeat", "grind sage", "end"]));
        assert_eq!(run(&program, 5).len(), 4, "5 steps: 1 to enter, 4 to run");
        assert_eq!(run(&program, 40).len(), 39, "and it never finishes");
    }

    #[test]
    fn an_empty_repeat_body_does_not_spin_for_ever() {
        let program = read(&lines(&["repeat 2", "end", "survey"]));
        assert_eq!(run(&program, 20), ["survey"]);
    }

    /// The world these tests pretend to have: every guard says go round, and
    /// every set has two members — one never laps, none never enters.
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
                // Stepped past, as the runner does.
                Some(Kind::Part { .. }) => {
                    if !step_past(&program.body, &mut pc, &mut loops, a_set_of_two) {
                        break;
                    }
                    continue;
                }
                // Leaves: calls and forks are the runner's, tested through a
                // real `Sim` in `tests.rs`.
                Some(Kind::Call { name, args }) => {
                    out.push(format!("{name}({})", args.join(", ")));
                }
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
                // `has sage` is `has 1 sage`.
                count: crate::parser::Quantity::Count(1),
                bound: crate::parser::Bound::AtLeast,
            }),
        );
    }

    #[test]
    fn an_if_takes_one_half_and_carries_on_past_both() {
        // A branch costs two path elements going in; pop one and the other half
        // runs too.
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
        let program = read(&lines(&["if the moon is gibbous", "grind sage", "end"]));
        let Kind::If { condition, .. } = &program.body[0].kind else {
            panic!("not an if");
        };
        assert!(condition.is_none());
        assert_eq!(program.complaints[0].key, "spell_unreadable_if");
        assert!(run_with(&program, 20, false).is_empty());
    }

    /// `bide` takes a number and nothing else: a word once read its delay off
    /// the world, and the typo `bide sage` bided `u32::MAX`.
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
        // If the two trees ever differ, the shorter form has become a second
        // language.
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

        // Without line numbers, which differ by construction.
        assert_eq!(
            shape(&ladder.body),
            shape(&by_hand.body),
            "the ladder is not the nesting",
        );
        assert!(ladder.complaints.is_empty(), "{:?}", ladder.complaints);
        assert!(by_hand.complaints.is_empty(), "{:?}", by_hand.complaints);

        assert_eq!(7 + 2, 9, "the two spellings above are 7 lines against 9");
    }

    #[test]
    fn a_ladder_of_rungs_closes_with_one_end() {
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

        // Walked, because the bug guarded is a chain that closes one frame too
        // few.
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
        // `false` walks the whole chain to the final `else`.
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
        // `end` closes whatever is innermost, so misordering cannot be detected
        // by counting; what must not happen is a panic or a dropped body.
        let program = read(&lines(&["repeat 2", "grind sage", "end", "end"]));
        assert!(matches!(program.body[0].kind, Kind::Repeat { .. }));
        assert_eq!(program.complaints.len(), 1, "the stray end went unreported");
    }
}
