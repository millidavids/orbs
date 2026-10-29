//! Reading a spell's text into something the orb can run.
//!
//! §8's loose phrasing is resolved at **cast**, into the program and never into
//! the file: a rewriter that understands part of a line writes the rest out of
//! existence (§19). Cast, not save, so names are fixed in the world the spell is
//! about to run in.
//!
//! An abbreviation is accepted, never a typo or a coin flip: nobody is watching a
//! spell resolve, so it is held to [`SPELL_SIMILARITY`], not §6's prompt floor.

use bevy_ecs::prelude::*;

use crate::parser::{
    Condition, EXACT, Mode, NounKind, NounMatch, Resolution, Scene, Verb, analyse, leaf,
};

use super::program::{Block, Complaint, Draft, Kind, Program, Step};

/// How alike a name must be to what it names, inside a spell.
///
/// 850 is `PREFIX_FLOOR`, where the scale already divides: any prefix of three
/// or more clears it, and a typo only as one wrong character in a long word.
/// At the prompt's 600, a question about absent salt compiled as one about sage.
pub const SPELL_SIMILARITY: u32 = 850;

/// How far ahead of the runner-up a name must be to be the one meant.
///
/// `ground` is an equal prefix of `ground-sage` and `ground-salt`, and
/// `best_match` would pick whichever registered first. The ceiling is `sag` →
/// `sage` at 67; `the_margin_is_below_the_closest_call_the_vocabulary_makes`
/// fails when a new name squeezes it.
pub const SPELL_MARGIN: u32 = 50;

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
    check_commands(&draft.body, &scene, &bound, &mut complaints);
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
            _ => {}
        }
    }
}

/// Say so about every call naming a part the spell does not define.
///
/// At cast rather than at the call, which inside a branch may be never.
/// `run::called` answers again for a definition deleted while the spell runs.
fn check_calls(body: &Block, defined: &[(String, usize)], complaints: &mut Vec<Complaint>) {
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
/// so a forbidden line in an untaken branch is still said.
fn check_commands(body: &Block, scene: &Scene, bound: &[String], complaints: &mut Vec<Complaint>) {
    for Step { line, kind } in body {
        match kind {
            Kind::Command(text) => {
                // A variable holds nothing until the line runs.
                if names_a_binding(text, bound) {
                    continue;
                }
                let Resolution::Resolved { intent, .. } =
                    crate::parser::analyse(text, scene, Mode::Calm).resolution
                else {
                    continue;
                };
                if !super::run::may_issue(intent.verb) {
                    // Its own key: the runtime one's `{detail}` has nothing to
                    // fill it here.
                    complaints.push(Complaint {
                        line: *line,
                        key: "spell_forbidden_line",
                    });
                }
            }
            Kind::Repeat { body, .. } | Kind::Each { body, .. } | Kind::Part { body, .. } => {
                check_commands(body, scene, bound, complaints);
            }
            Kind::If {
                body, otherwise, ..
            } => {
                check_commands(body, scene, bound, complaints);
                check_commands(otherwise, scene, bound, complaints);
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
fn names_a_binding(line: &str, bound: &[String]) -> bool {
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
                    // [`fix`].
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

/// Resolve one question's names in place, reporting every name it could not
/// place.
///
/// A missing *place* leaves the question unanswerable (§8's *Referent missing*),
/// named by the runner every cast. A missing *thing* is usually an answer of no
/// — `has ground-sage` is asked before there is any — so it is looked for in the
/// room, then in every name any recipe knows; in neither, it is a typo, and the
/// question cannot be asked. Every name, not the first (§8.1).
fn fix(
    condition: &mut Condition,
    scene: &Scene,
    known: &[&str],
    bound: &[String],
) -> Vec<Unplaced> {
    let mut unplaced = Vec::new();
    condition.rename(&mut |kind, name| {
        // A bound name stands for a place known only when the line runs; its
        // value was resolved at the `set`.
        if bound.iter().any(|held| held.eq_ignore_ascii_case(name)) {
            return None;
        }
        let found = match kind {
            NounKind::Place => clearly(scene, NounKind::Place, name),
            _ => clearly(scene, NounKind::Any, name).or_else(|| {
                known
                    .iter()
                    .find(|word| word.eq_ignore_ascii_case(name))
                    .map(|word| (*word).to_owned())
            }),
        };
        if found.is_none() {
            unplaced.push(match kind {
                NounKind::Place => Unplaced::Place(name.to_owned()),
                _ => Unplaced::Thing(name.to_owned()),
            });
        }
        found
    });
    unplaced
}

/// The one fault a list of unplaced names adds up to.
///
/// A typo outranks a missing place: one leaves a question that cannot be asked,
/// the other one that cannot be answered. Every offender of that kind is named.
fn fault_of(unplaced: &[Unplaced]) -> Option<Fault> {
    let typos: Vec<&str> = unplaced
        .iter()
        .filter_map(|entry| match entry {
            Unplaced::Thing(name) => Some(name.as_str()),
            Unplaced::Place(_) => None,
        })
        .collect();
    if !typos.is_empty() {
        return Some(Fault {
            key: "spell_unreadable_if",
            detail: Some(typos.join(", ")),
        });
    }
    let places: Vec<&str> = unplaced
        .iter()
        .filter_map(|entry| match entry {
            Unplaced::Place(name) => Some(name.as_str()),
            Unplaced::Thing(_) => None,
        })
        .collect();
    (!places.is_empty()).then(|| Fault {
        key: "spell_nowhere",
        detail: Some(places.join(", ")),
    })
}

/// A name in a question that the room could not place, and which kind it was.
enum Unplaced {
    /// A place the tower does not have: the question stands, unanswerable.
    Place(String),
    /// A word that names nothing, in the room or the recipes: a typo.
    Thing(String),
}

/// What `name` clearly names in `scene`, or `None` if it is not clear.
///
/// Unclear is nothing near enough, a typo rather than an abbreviation, or two
/// things equally close.
fn clearly(scene: &Scene, kind: NounKind, name: &str) -> Option<String> {
    let words: Vec<&str> = name.split_whitespace().collect();
    let found = scene.candidates(kind, &words);
    let best = found.first()?;
    if best.score < SPELL_SIMILARITY {
        return None;
    }
    let winner = leaf(&best.name);

    // Two entries for one word are not a tie: §6.1 registers a `Topic` beside
    // every reagent at the same score.
    let runner_up = found.iter().find(|next| leaf(&next.name) != winner);

    // An exact match is never a coin flip.
    if best.score < EXACT
        && runner_up.is_some_and(|next| best.score.saturating_sub(next.score) < SPELL_MARGIN)
    {
        return None;
    }
    Some(winner.to_owned())
}

/// Every distinct name a scene could answer `name` with, best first — what a
/// tie is detected from, which `best_match` throws away.
#[must_use]
pub fn candidates(scene: &Scene, kind: NounKind, name: &str) -> Vec<NounMatch> {
    let words: Vec<&str> = name.split_whitespace().collect();
    scene.candidates(kind, &words)
}

/// Read `lines` without resolving anything — for the parser's tests, and for the
/// editor, which reports on a buffer rather than running it.
#[must_use]
pub fn read(lines: &[String]) -> Draft {
    super::program::read(lines)
}

/// One line of a spell, as the orb reads it: the rewriter turned round, so it
/// returns the reading instead of writing it into the player's file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    /// Which line, counting from one, as the player sees it.
    pub line: usize,
    /// What the orb hears: the canonical command, the resolved question, or the
    /// line itself where it is the player's own (a comment, a blank).
    pub heard: String,
    /// What the player wrote, when a reader turned it into something else —
    /// without it a line read rightly and one read wrongly look alike.
    pub was: Option<String>,
    /// Why the orb cannot read it, if it cannot.
    pub fault: Option<Fault>,
}

/// What is wrong with a line, in a form prose can be built from.
///
/// Not a [`Complaint`], which has nowhere to put the name at fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    /// The prose key for what is wrong.
    pub key: &'static str,
    /// The word at fault, where there is one.
    pub detail: Option<String>,
}

/// Read `lines` as a spell for `domain`, line by line, without running anything.
///
/// What the editor draws `interpret` and its marks from, over the buffer rather
/// than the saved file. `read` is each line as it would compile, from the save.
/// Everything file-shaped is asked of the reading, not the text, or the two
/// disagree exactly where a reader helped (§19).
#[must_use]
pub fn interpret(world: &World, domain: &str, lines: &[String], read: &[String]) -> Vec<Reading> {
    let Some(at) = crate::execute::find_domain(world, domain) else {
        return lines
            .iter()
            .enumerate()
            .map(|(index, line)| Reading {
                line: index + 1,
                heard: line.trim().to_owned(),
                was: None,
                fault: Some(Fault {
                    key: "spell_homeless",
                    detail: Some(domain.to_owned()),
                }),
            })
            .collect();
    };
    let scene = crate::tower::scene_at(world, at);
    let known = world.resource::<crate::content::Recipes>().vocabulary();

    // What would compile: each line's reading, or the line where it has none.
    let compiled: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(at, line)| read.get(at).unwrap_or(line).clone())
        .collect();

    // A fault of shape belongs to a line but is found by reading the whole file.
    let draft = super::program::read(&compiled);
    let mut structural = draft.complaints;
    // The cast's check, on the same tree.
    let defined: Vec<(String, usize)> = super::program::parts(&draft.body)
        .into_iter()
        .map(|(name, takes)| (name.to_owned(), takes))
        .collect();
    check_calls(&draft.body, &defined, &mut structural);
    // The whole file's bindings for every line: a `set` on line 9 may be said on
    // line 2.
    let bound = super::program::bindings(&draft.body);

    lines
        .iter()
        .zip(&compiled)
        .enumerate()
        .map(|(index, (line, heard))| {
            let at = index + 1;
            let mut reading = one(heard, &scene, &known, &bound);
            reading.was = (heard != line).then(|| line.clone());
            reading.line = at;
            if reading.fault.is_none() {
                reading.fault = structural
                    .iter()
                    .find(|complaint| complaint.line == at)
                    .map(|complaint| Fault {
                        key: complaint.key,
                        detail: None,
                    });
            }
            reading
        })
        .collect()
}

/// One line, read.
fn one(line: &str, scene: &Scene, known: &[&str], bound: &[String]) -> Reading {
    let trimmed = line.trim();
    let verbatim = |fault: Option<Fault>| Reading {
        line: 0,
        heard: trimmed.to_owned(),
        was: None,
        fault,
    };
    // The orb's reading where it differs from the text: `gathering ()` is heard
    // as `gathering()`, the form the player has to type.
    let verbatim_as = |heard: &str, fault: Option<Fault>| Reading {
        line: 0,
        heard: heard.to_owned(),
        was: None,
        fault,
    };

    if trimmed.is_empty() || trimmed.starts_with('#') {
        return verbatim(None);
    }

    // A call before the command resolver, as `read` orders it — whether the part
    // exists comes from `check_calls`. Guarded on `spell_word`, or `part
    // gathering()` reads as a malformed call.
    if crate::parser::spell_word(trimmed).is_none()
        && let Some(called) = super::program::call_of(trimmed)
    {
        return match called {
            // Arguments stay names: what they stand for is known when the line
            // runs.
            Some((name, args)) => verbatim_as(&format!("{name}({})", args.join(", ")), None),
            None => verbatim(Some(Fault {
                key: "spell_unreadable_call",
                detail: None,
            })),
        };
    }

    if let Some(word) = crate::parser::spell_word(trimmed) {
        // `repeat until` carries an `if`'s grammar and is fixed by the same `fix`.
        let guarded = word == crate::parser::SpellWord::Repeat
            && crate::parser::spell_argument(trimmed)
                .split_whitespace()
                .next()
                .is_some_and(|first| {
                    first.eq_ignore_ascii_case(crate::parser::SpellWord::Until.canonical())
                });
        if word != crate::parser::SpellWord::If && !guarded {
            return verbatim(None);
        }
        let (lead, argument) = if guarded {
            let rest = crate::parser::spell_argument(trimmed);
            let after = rest
                .split_once(char::is_whitespace)
                .map_or("", |(_, tail)| tail);
            ("repeat until", after)
        } else {
            ("if", crate::parser::spell_argument(trimmed))
        };
        let Some(mut question) = crate::parser::condition(argument) else {
            return verbatim(Some(Fault {
                key: if guarded {
                    "spell_unreadable_until"
                } else {
                    "spell_unreadable_if"
                },
                detail: None,
            }));
        };
        let fault = fault_of(&fix(&mut question, scene, known, bound));
        return Reading {
            line: 0,
            heard: format!("{lead} {}", crate::parser::write_condition(&question)),
            was: None,
            fault,
        };
    }

    // A line naming a binding is quoted, never resolved: `follow best` once read
    // back as `follow west`, the one place `best` is certainly not.
    if trimmed
        .split_whitespace()
        .any(|word| bound.iter().any(|held| held.eq_ignore_ascii_case(word)))
    {
        return verbatim(None);
    }

    // As the runner reads it: the same `analyse`, in the spell's own room.
    let resolution = analyse(line, scene, Mode::Calm).resolution;

    // A spell naming another is quoted: the one named often does not exist yet,
    // and the matcher would name a spell the player did not (§19).
    if names_a_spell(&resolution) {
        return verbatim(None);
    }

    let intent = match resolution {
        Resolution::Resolved { intent, .. } => intent,
        // `muster the troops` names no missing referent: the verb takes nothing.
        Resolution::TakesNothing { verb, extra, .. } => {
            return verbatim(Some(Fault {
                key: super::run::takes_nothing_key(verb),
                detail: Some(extra),
            }));
        }
        _ => {
            return verbatim(Some(Fault {
                key: "spell_missing",
                detail: Some(trimmed.to_owned()),
            }));
        }
    };
    if !super::run::may_issue(intent.verb) {
        return verbatim(Some(Fault {
            key: "spell_forbidden",
            detail: Some(trimmed.to_owned()),
        }));
    }
    let mut heard = String::from(intent.verb.canonical());
    for argument in &intent.arguments {
        heard.push(' ');
        heard.push_str(argument.display());
    }
    Reading {
        line: 0,
        heard,
        was: None,
        fault: None,
    }
}

/// Whether this reading is a spell naming another spell.
///
/// `Incomplete` counts — a forward reference leaves the `Script` slot unfilled —
/// and so does a tie between spells, which a forward reference with four on the
/// shelf produces.
fn names_a_spell(resolution: &Resolution) -> bool {
    let verb = match resolution {
        Resolution::Resolved { intent, .. } => intent.verb,
        Resolution::Incomplete { verb, .. } => *verb,
        Resolution::Ambiguous { candidates } => {
            return !candidates.is_empty()
                && candidates
                    .iter()
                    .all(|candidate| matches!(candidate.intent.verb, Verb::Invoke | Verb::Bind));
        }
        _ => return false,
    };
    matches!(verb, Verb::Invoke | Verb::Bind)
}
