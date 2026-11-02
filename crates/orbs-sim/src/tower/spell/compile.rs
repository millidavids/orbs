//! Reading a spell's text into something the orb can run.
//!
//! §8's loose phrasing is resolved at **cast**, into the program and never into
//! the file: a rewriter that understands part of a line writes the rest out of
//! existence (§19). Cast, not save, so names are fixed in the world the spell is
//! about to run in.
//!
//! An abbreviation is accepted, never a typo or a coin flip: nobody is watching a
//! spell resolve, so it is held to [`SPELL_SIMILARITY`](super::SPELL_SIMILARITY),
//! not §6's prompt floor.

use bevy_ecs::prelude::*;

use crate::parser::{Mode, NounKind, Scene, analyse};

use super::names::{Unplaced, clearly, fix};
use super::program::{Block, Complaint, Draft, Kind, Program, Step};
use super::speech::Refused;

/// Whether one line stands on its own as a spell **statement**.
///
/// What a reader's output must pass before it may replace a line. Three
/// complaints are earned only for being alone — a lone `if`, `end` or `else` —
/// so `complaints.is_empty()` would refuse most control flow. A command line
/// answers `false`: the prompt's reader owns it.
///
/// Parsing is not understanding: `if the alembic has finished` passes, as
/// *holds a thing called finished*. A reader must also account for every word.
#[must_use]
pub fn reads_cleanly(line: &str) -> bool {
    /// What a line earns for having no block around it.
    const ALONE: [&str; 3] = ["spell_unclosed", "spell_stray_end", "spell_stray_else"];

    let draft = super::program::read(std::slice::from_ref(&line.to_owned()));
    if !draft
        .complaints
        .iter()
        .all(|complaint| ALONE.contains(&complaint.key))
    {
        return false;
    }
    // `end` and `else` produce no step, and a call opens on no spell word.
    crate::parser::spell_word(line).is_some()
        || draft
            .body
            .first()
            .is_some_and(|step| !matches!(step.kind, super::program::Kind::Command(_)))
}

/// The lines a spell compiles from: its reading, or its text if it has none.
///
/// One place for casting, the mid-flight reload and a restored save. Line by
/// line, because a reading is used only where it was read from the text now at
/// its place — see `Read::compiled`.
#[must_use]
pub fn source(world: &World, node: Entity) -> Vec<String> {
    let held = world
        .get::<crate::tower::Held>(node)
        .map(|held| held.0.clone())
        .unwrap_or_default();
    match world.get::<crate::tower::Read>(node) {
        Some(read) => held
            .iter()
            .enumerate()
            .map(|(at, line)| read.compiled(at, line).to_owned())
            .collect(),
        None => held,
    }
}

/// Read `lines` as a program, with every name resolved against the domain at
/// `from`.
///
/// The only way a [`Program`] is made: `program::read` returns a [`Draft`],
/// which nothing can run, so no future call site can skip the resolution.
#[must_use]
pub fn compile(world: &World, from: Entity, lines: &[String]) -> Program {
    let at = crate::tower::domain_of(world, from).unwrap_or(from);
    let scene = crate::tower::scene_at(world, at);
    let known = world.resource::<crate::content::Recipes>().vocabulary();
    let draft = super::program::read(lines);
    let mut complaints = draft.complaints;
    // First: a bound name otherwise reaches `fix` looking like a missing place.
    let bound = super::program::bindings(&draft.body);
    // Against the text, since the spell defines its parts; `interpret` asks the
    // same (§19).
    let defined: Vec<(String, usize)> = super::program::parts(&draft.body)
        .into_iter()
        .map(|(name, takes)| (name.to_owned(), takes))
        .collect();
    check_calls(&draft.body, &defined, &mut complaints);
    let for_the_tower = super::whole::at_the_tower(world, at);
    check_commands(&draft.body, &scene, &bound, for_the_tower, &mut complaints);
    check_learned(world, &draft.body, &mut complaints);
    let body = resolved(draft.body, &scene, &known, &bound, &mut complaints);
    // Down the file, however the faults were found.
    complaints.sort_by_key(|complaint| complaint.line);
    Program::new(body, complaints)
}

/// Say so about every word the loom has not granted yet.
///
/// `pull` and `alongside` are control words with nobody to answer at run time,
/// so the report is made here, before casting. A complaint, not a refusal: the
/// rest of the spell runs (§8).
fn check_learned(world: &World, body: &Block, complaints: &mut Vec<Complaint>) {
    use crate::tower::{Grant, holds};
    let satchel = holds(world, Grant::Satchel);
    let cursors = holds(world, Grant::Cursors);
    if satchel && cursors {
        return;
    }
    walk_learned(body, satchel, cursors, complaints);
}

fn walk_learned(body: &Block, satchel: bool, cursors: bool, complaints: &mut Vec<Complaint>) {
    for Step { line, kind } in body {
        match kind {
            Kind::Pull { .. } if !satchel => complaints.push(Complaint {
                line: *line,
                key: "pull_unlearned",
            }),
            Kind::Alongside { .. } if !cursors => complaints.push(Complaint {
                line: *line,
                key: "alongside_unlearned",
            }),
            Kind::Repeat { body, .. } | Kind::Each { body, .. } | Kind::Part { body, .. } => {
                walk_learned(body, satchel, cursors, complaints);
            }
            Kind::If {
                body, otherwise, ..
            } => {
                walk_learned(body, satchel, cursors, complaints);
                walk_learned(otherwise, satchel, cursors, complaints);
            }
            Kind::Command(_)
            | Kind::Wait(_)
            | Kind::Bide(_)
            | Kind::Let { .. }
            | Kind::Pull { .. }
            | Kind::Call { .. }
            | Kind::Alongside { .. } => {}
        }
    }
}

/// Say so about every call naming a part the spell does not define.
///
/// At cast rather than at the call, which inside a branch may be never.
/// `parts::called` answers again for a definition deleted while the spell runs.
pub(super) fn check_calls(
    body: &Block,
    defined: &[(String, usize)],
    complaints: &mut Vec<Complaint>,
) {
    for Step { line, kind } in body {
        match kind {
            // A fork is the same mistake as a call.
            Kind::Call { name, args } | Kind::Alongside { name, args } => {
                match defined.iter().find(|(part, _)| part == name) {
                    None => complaints.push(Complaint {
                        line: *line,
                        key: "spell_no_such_part",
                    }),
                    // Positional, so a short call leaves a parameter that would
                    // resolve against the room and quietly do the wrong thing.
                    Some((_, wanted)) if *wanted != args.len() => {
                        complaints.push(Complaint {
                            line: *line,
                            key: "spell_call_arity",
                        });
                    }
                    Some(_) => {}
                }
            }
            Kind::Repeat { body, .. } | Kind::Each { body, .. } | Kind::Part { body, .. } => {
                check_calls(body, defined, complaints);
            }
            Kind::If {
                body, otherwise, ..
            } => {
                check_calls(body, defined, complaints);
                check_calls(otherwise, defined, complaints);
            }
            Kind::Command(_)
            | Kind::Wait(_)
            | Kind::Bide(_)
            | Kind::Let { .. }
            | Kind::Pull { .. } => {}
        }
    }
}

/// Read each command's **verb** at cast, and say so when it is one a spell may
/// not issue.
///
/// Only the verb: a spell makes its own inputs, so at cast its arguments may not
/// exist yet, and freezing a whole `Intent` would break every pipeline (§19).
/// `may_issue` is a security boundary, asked here as well as when the line runs,
/// so a forbidden line in an untaken branch is still said. A variable holds
/// nothing until the line runs, but the verb beside it is already the verb.
fn check_commands(
    body: &Block,
    scene: &Scene,
    bound: &[String],
    for_the_tower: bool,
    complaints: &mut Vec<Complaint>,
) {
    for Step { line, kind } in body {
        match kind {
            Kind::Command(text) => {
                let refused = if names_a_binding(text, bound) {
                    Refused::opening(text, for_the_tower)
                } else {
                    Refused::reading(&analyse(text, scene, Mode::Calm).resolution, for_the_tower)
                };
                if let Some(refused) = refused {
                    complaints.push(Complaint {
                        line: *line,
                        key: refused.at_cast(),
                    });
                }
            }
            Kind::Repeat { body, .. } | Kind::Each { body, .. } | Kind::Part { body, .. } => {
                check_commands(body, scene, bound, for_the_tower, complaints);
            }
            Kind::If {
                body, otherwise, ..
            } => {
                check_commands(body, scene, bound, for_the_tower, complaints);
                check_commands(otherwise, scene, bound, for_the_tower, complaints);
            }
            // No verb in a bide. A `pull` names a place in the spell's room,
            // while `scene` is the player's, so checking it here would fault a
            // good line whenever a bound spell worked elsewhere (§19).
            Kind::Wait(_)
            | Kind::Bide(_)
            | Kind::Let { .. }
            | Kind::Pull { .. }
            | Kind::Call { .. }
            | Kind::Alongside { .. } => {}
        }
    }
}

/// Whether any word of `line` is a name the spell binds.
pub(super) fn names_a_binding(line: &str, bound: &[String]) -> bool {
    line.split_whitespace()
        .any(|word| bound.iter().any(|held| held.eq_ignore_ascii_case(word)))
}

/// Every condition in a block, with its names fixed.
fn resolved(
    body: Block,
    scene: &Scene,
    known: &[&str],
    bound: &[String],
    complaints: &mut Vec<Complaint>,
) -> Block {
    body.into_iter()
        .map(|Step { line, kind }| Step {
            line,
            kind: match kind {
                // Fixed as an `if`'s are, or a misspelt guard never ends.
                Kind::Repeat {
                    times,
                    mut until,
                    body,
                } => {
                    let unplaced = until
                        .as_mut()
                        .map(|condition| fix(condition, scene, known, bound))
                        .unwrap_or_default();
                    // Nought turns: a guarded repeat's `None` would make an
                    // unreadable loop one that never stops.
                    let mut turns = times;
                    if unplaced
                        .iter()
                        .any(|entry| matches!(entry, Unplaced::Thing(_)))
                    {
                        until = None;
                        turns = Some(0);
                        complaints.push(Complaint {
                            line,
                            key: "spell_unreadable_until",
                        });
                    }
                    Kind::Repeat {
                        times: turns,
                        until,
                        body: resolved(body, scene, known, bound, complaints),
                    }
                }
                Kind::If {
                    mut condition,
                    body,
                    otherwise,
                } => {
                    // A word naming nothing makes the question unreadable — see
                    // `names::fix`.
                    let unplaced = condition
                        .as_mut()
                        .map(|condition| fix(condition, scene, known, bound))
                        .unwrap_or_default();
                    if unplaced
                        .iter()
                        .any(|entry| matches!(entry, Unplaced::Thing(_)))
                    {
                        condition = None;
                        complaints.push(Complaint {
                            line,
                            key: "spell_unreadable_if",
                        });
                    }
                    Kind::If {
                        condition,
                        body: resolved(body, scene, known, bound, complaints),
                        otherwise: resolved(otherwise, scene, known, bound, complaints),
                    }
                }
                // Skipping the body would leave a typo inside a loop answering
                // no for ever.
                Kind::Each { group, body } => Kind::Each {
                    group,
                    body: resolved(body, scene, known, bound, complaints),
                },
                // `set m to mortar` binds `mortar_and_pestle`; a name the spell
                // binds itself is an accumulator and is left alone.
                Kind::Let { name, value } => {
                    let held = bound.iter().any(|held| held.eq_ignore_ascii_case(&value));
                    let found = if held {
                        Some(value.clone())
                    } else {
                        clearly(scene, NounKind::Any, &value)
                    };
                    if found.is_none() {
                        complaints.push(Complaint {
                            line,
                            key: "spell_unreadable_let",
                        });
                    }
                    Kind::Let {
                        name,
                        value: found.unwrap_or(value),
                    }
                }
                other => other,
            },
        })
        .collect()
}

/// Read `lines` without resolving anything — for the parser's tests, and for the
/// editor, which reports on a buffer rather than running it.
#[must_use]
pub fn read(lines: &[String]) -> Draft {
    super::program::read(lines)
}
