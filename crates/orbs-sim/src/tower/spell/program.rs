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
    /// Do the enclosed steps, `times` of them, or for ever if `None`.
    Repeat {
        /// How many times, or `None` for an unbounded loop.
        ///
        /// Unbounded is safe because the execution budget bounds a tick: a
        /// `repeat` with no `wait` in it spends its budget and stops, so it
        /// wastes itself rather than hanging the game.
        times: Option<u32>,
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
/// [`compile`](super::compile) turns one into a [`Program`], and nothing else
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
    /// Assemble one. `pub(super)` so [`compile`](super::compile) is the only
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
    },
    If {
        condition: Option<Condition>,
        /// The `then` branch, once an `else` has moved it aside. `None` while
        /// still reading the first half.
        taken: Option<Block>,
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
}

/// Read `lines` as a shape, resolving nothing.
///
/// `pub(super)` deliberately: the way in from outside this module is
/// [`compile`](super::compile), which does this and then fixes the names. See
/// [`Draft`].
#[must_use]
pub(super) fn read(lines: &[String]) -> Draft {
    let mut complaints = Vec::new();
    let mut open = vec![Nesting {
        line: 0,
        kind: Open::Spell,
        body: Vec::new(),
    }];

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let at = index + 1;

        match spell_word(trimmed) {
            Some(SpellWord::Repeat) => open.push(Nesting {
                line: at,
                kind: Open::Repeat {
                    times: count_of(spell_argument(trimmed)),
                },
                body: Vec::new(),
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
                });
            }
            Some(SpellWord::Else) => {
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
                    _ => complaints.push(Complaint {
                        line: at,
                        key: "spell_stray_else",
                    }),
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
                if let Some(frame) = open.pop() {
                    close(&mut open, frame);
                }
            }
            Some(SpellWord::Wait) => {
                let wanted = strip_filler(spell_argument(trimmed));
                push(&mut open, at, Kind::Wait(wanted));
            }
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
        complaints.push(Complaint {
            line: frame.line,
            key: "spell_unclosed",
        });
        close(&mut open, frame);
    }

    Draft {
        // **No `expect`.** The outermost block cannot be popped by the loops
        // above — both are guarded on `len() > 1` — but writing that as a panic
        // would be a claim the compiler cannot check and a crash if it ever
        // stopped being true. An empty spell is a real thing anyway.
        body: open.pop().map(|frame| frame.body).unwrap_or_default(),
        complaints,
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
        Kind::Repeat { body, .. } => at(body, rest),
        Kind::If {
            body, otherwise, ..
        } => {
            let (&branch, inner) = rest.split_first()?;
            at(if branch == 0 { body } else { otherwise }, inner)
        }
        Kind::Command(_) | Kind::Wait(_) => None,
    }
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
pub fn step_past(body: &Block, pc: &mut Vec<usize>, loops: &mut Vec<Loop>) -> bool {
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
            // Unbounded, or more to go: back to the top of this block.
            Some(Loop::Repeat(None)) => {
                loops.push(Loop::Repeat(None));
                pc.push(0);
                return true;
            }
            Some(Loop::Repeat(Some(left))) if left > 1 => {
                loops.push(Loop::Repeat(Some(left - 1)));
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

/// Descend into a `repeat`, recording how many times it should run.
pub fn enter(pc: &mut Vec<usize>, loops: &mut Vec<Loop>, times: Option<u32>) {
    loops.push(Loop::Repeat(times));
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
        Open::Repeat { times } => Kind::Repeat {
            times,
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
        Open::Spell => return,
    };
    push(open, line, kind);
}

/// `repeat 3` → `Some(3)`; a bare `repeat` → `None`.
///
/// A word that is not a number is **not** an error: `repeat until the mortar`
/// would be a reasonable thing to try, and reading it as an unbounded loop is
/// closer to what was meant than refusing the line.
fn count_of(argument: &str) -> Option<u32> {
    argument.split_whitespace().next()?.parse().ok()
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
        let Kind::Repeat { times, body } = &program.body[0].kind else {
            panic!("not a repeat: {:?}", program.body);
        };
        assert_eq!(*times, Some(3));
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

    /// Run a program, taking `holds` for every condition.
    fn run_with(program: &Draft, budget: usize, holds: bool) -> Vec<String> {
        let mut pc = vec![0];
        let mut loops = Vec::new();
        let mut out = Vec::new();
        for _ in 0..budget {
            match at(&program.body, &pc).map(|step| &step.kind) {
                None => break,
                Some(Kind::Repeat { times, body }) => {
                    if body.is_empty() {
                        if !step_past(&program.body, &mut pc, &mut loops) {
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
                        if !step_past(&program.body, &mut pc, &mut loops) {
                            break;
                        }
                    } else {
                        enter_branch(&mut pc, &mut loops, holds);
                    }
                    continue;
                }
                Some(Kind::Command(line)) => out.push(line.clone()),
                Some(Kind::Wait(what)) => out.push(format!("wait {what}")),
            }
            if !step_past(&program.body, &mut pc, &mut loops) {
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

    #[test]
    fn an_else_with_no_if_is_reported_rather_than_starting_a_branch() {
        let program = read(&lines(&["repeat 2", "grind sage", "else", "end"]));
        assert_eq!(program.complaints[0].key, "spell_stray_else");
        assert_eq!(run_with(&program, 20, true).len(), 2, "the loop still ran");
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
