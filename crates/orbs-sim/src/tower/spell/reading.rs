//! The editor's view of a spell: each line as the orb reads it, without running
//! anything.

use bevy_ecs::prelude::*;

use crate::parser::{Mode, NounKind, Resolution, Scene, Verb, analyse};

use super::compile::{check_calls, names_a_binding};
use super::names::{fault_of, fix};
use super::speech::Refused;

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
/// Not a [`Complaint`](super::Complaint), which has nowhere to put the name at
/// fault.
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
    let for_the_tower = super::whole::at_the_tower(world, at);

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
            let mut reading = one(heard, &scene, &known, &bound, for_the_tower);
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
fn one(
    line: &str,
    scene: &Scene,
    known: &[&str],
    bound: &[String],
    for_the_tower: bool,
) -> Reading {
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
        // The fair copy keeps a spell's bare name, as the player writes it.
        question.rename(&mut |kind, name| {
            (kind == NounKind::Stoppable)
                .then(|| name.strip_suffix(".spell").map(str::to_owned))
                .flatten()
        });
        return Reading {
            line: 0,
            heard: format!("{lead} {}", crate::parser::write_condition(&question)),
            was: None,
            fault,
        };
    }

    let refusal = |refused: Refused| {
        verbatim(Some(Fault {
            key: refused.of_line(),
            detail: Some(trimmed.to_owned()),
        }))
    };
    // Quoted, its verb still read as the runner reads it.
    let quoted =
        || Refused::opening(trimmed, for_the_tower).map_or_else(|| verbatim(None), refusal);

    // A line naming a binding is quoted, never resolved: `follow best` once read
    // back as `follow west`, the one place `best` is certainly not.
    if names_a_binding(trimmed, bound) {
        return quoted();
    }

    // As the runner reads it: the same `analyse`, in the spell's own room.
    let resolution = analyse(line, scene, Mode::Calm).resolution;
    if let Some(refused) = Refused::reading(&resolution, for_the_tower) {
        return refusal(refused);
    }

    // A spell naming another is quoted: the one named often does not exist yet,
    // and the matcher would name a spell the player did not (§19).
    if names_a_spell(&resolution) {
        return quoted();
    }

    let intent = match resolution {
        Resolution::Resolved { intent, .. } => intent,
        // `muster the troops` names no missing referent: the verb takes nothing.
        Resolution::TakesNothing { verb, extra, .. } => {
            return verbatim(Some(Fault {
                key: super::line::takes_nothing_key(verb),
                detail: Some(extra),
            }));
        }
        Resolution::Ambiguous { .. }
        | Resolution::Incomplete { .. }
        | Resolution::Elsewhere { .. }
        | Resolution::InSpell { .. }
        | Resolution::Aside { .. }
        | Resolution::Unresolved { .. } => {
            return verbatim(Some(Fault {
                key: "spell_missing",
                detail: Some(trimmed.to_owned()),
            }));
        }
    };
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
        Resolution::TakesNothing { .. }
        | Resolution::Elsewhere { .. }
        | Resolution::InSpell { .. }
        | Resolution::Aside { .. }
        | Resolution::Unresolved { .. } => return false,
    };
    matches!(verb, Verb::Invoke | Verb::Bind)
}
