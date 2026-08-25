//! Reading a spell's text into something the orb can run.
//!
//! # Why the names are fixed here and not at save
//!
//! DESIGN.md §8 asked for loose phrasing to be resolved *"at authoring time"*,
//! because *"a script executes later, in a different world state, where
//! live-state disambiguation is unavailable"*. That requirement is right and
//! this is where it is met — but the moment is **cast**, not save, and the
//! result lives in the program rather than in the file.
//!
//! The file was the wrong place for it. A rewriter that understands part of a
//! line writes the rest out of existence, which §19 records happening four
//! separate times; and a file that is rewritten is not the player's any more.
//! The program has been *"derived, never stored"* since it existed, so putting
//! the reading there costs nothing and closes the class.
//!
//! Cast is also a better moment than save. The names are fixed once, in the
//! world the spell is about to run in, with the player standing there having
//! just typed `invoke` — and a name that stops matching later is **reported**
//! every time the question is asked, which is §8.1's substitution surface
//! working as designed.
//!
//! # The orb accepts an abbreviation, never a typo, and never a coin flip
//!
//! §6's matcher is deliberately forgiving because the player is *there*: the
//! echo says what it heard, and a wrong guess costs one line. A spell resolves
//! with nobody watching, so it is held to a higher bar — [`SPELL_SIMILARITY`].

use bevy_ecs::prelude::*;

use crate::parser::{
    Condition, EXACT, Mode, NounKind, NounMatch, Resolution, Scene, Verb, analyse, leaf,
};

use super::program::{Block, Complaint, Draft, Kind, Program, Step};

/// How alike a name must be to what it names, inside a spell.
///
/// # 850, and the shape of the scale decides it
///
/// `fuzzy` scores a **prefix** of three characters or more at `PREFIX_FLOOR`
/// (850) plus coverage, so an abbreviation is *always* at or above this line:
/// `mortar` → `mortar_and_pestle` is 902, `balneum` → `balneum_mariae` is 925.
/// A **typo** is scored by edit distance, and only reaches 850 when it is a
/// single character in a long word.
///
/// So this is not an arbitrary threshold — it is the boundary between the two
/// things `fuzzy` already treats differently, and its own docs name them:
/// *"prefixes are intentional… typos are accidental."* Inside a spell the orb
/// accepts the first and refuses the second.
///
/// The case that forced it: `similarity("ground-salt", "ground-sage")` is 819,
/// comfortably over the prompt's floor of 600. A question written while the salt
/// happened to be absent would have compiled into a question about the sage —
/// the file saying one thing and the running spell asking another, with nothing
/// on screen to show it.
pub const SPELL_SIMILARITY: u32 = 850;

/// How far ahead of the runner-up a name must be to be the one meant.
///
/// A floor alone cannot help when two things are *equally* close: with both
/// products on the shelf, `ground` is a 931 prefix of `ground-sage` and of
/// `ground-salt` alike, and `Scene::best_match` would hand back whichever was
/// registered first. That is a coin flip deciding what a laboratory does.
///
/// Command lines in a spell already refuse on ambiguity — `run_line` takes only
/// `Resolution::Resolved` — so this brings questions into line rather than
/// inventing a policy for them.
///
/// # 50, and why it was 1
///
/// It was `1`, which meant *"refuse only on an exact tie"* — a margin in name
/// and a tie in behaviour. Measured against the shipped vocabulary that is not
/// wrong yet, because every reading is either a dead heat or a landslide:
///
/// | typed | best | runner-up | lead |
/// |---|---|---|---|
/// | `mortar` | 902 `mortar_and_pestle` | 250 `charcoal` | 652 |
/// | `ground` | 931 `ground-salt` | 931 `ground-sage` | **0** |
/// | `ground-sale` | 910 `ground-salt` | 910 `ground-sage` | **0** |
/// | `sag` | 962 `sage` | 895 `sage-husks` | 67 |
/// | `ground-sag` | 986 `ground-sage` | 819 `ground-salt` | 167 |
///
/// Nothing lands between 1 and 54, so the two rules cannot be told apart today.
/// **One reagent named close to another and they part company**: a twenty-point
/// lead would resolve in silence, which is the failure the tie rule exists to
/// prevent, arriving through the constant that was supposed to prevent it.
///
/// The ceiling is `sag` at 67 — a legitimate abbreviation that has to keep
/// working — so 50 is the room there is. `the_margin_is_below_the_closest_call`
/// measures that lead against the real vocabulary and fails when a new name
/// squeezes it, which is the point: the next person to add a reagent finds out
/// from a test rather than from a spell quietly doing the wrong thing.
pub const SPELL_MARGIN: u32 = 50;

/// Read `lines` as a program, with every name resolved against the domain at
/// `from`.
///
/// **The only way a [`Program`] is made.** `program::read` returns a [`Draft`],
/// which nothing can run and nothing can store on [`Running`](super::Running) —
/// so a future call site cannot skip the resolution by reaching for the parser
/// directly. Visibility would not have done that job: both callers are already
/// inside this crate.
#[must_use]
pub fn compile(world: &World, from: Entity, lines: &[String]) -> Program {
    let at = crate::tower::domain_of(world, from).unwrap_or(from);
    let scene = crate::tower::scene_at(world, at);
    let known = world.resource::<crate::content::Recipes>().vocabulary();
    let draft = super::program::read(lines);
    let mut complaints = draft.complaints;
    // **The names the spell binds, gathered before anything is resolved.** They
    // are lexical — every `set` and every `for each` in the file — so this is a
    // read of the text rather than a fact about the world, and it has to happen
    // first: a bound name reaches `fix` looking exactly like a place the room
    // does not have, and would be reported as one on every cast.
    let bound = super::program::bindings(&draft.body);
    // **Before names are resolved, and against the text rather than the room.**
    // A part is a name the *spell* defines, so whether a call can be made is a
    // question about the file and not about the tower — and asking it here means
    // `interpret` and the cast agree, which §19 records two expressions of one
    // rule failing to do twice.
    let defined: Vec<String> = super::program::parts(&draft.body)
        .into_iter()
        .map(str::to_owned)
        .collect();
    check_calls(&draft.body, &defined, &mut complaints);
    check_commands(&draft.body, &scene, &bound, &mut complaints);
    let body = resolved(draft.body, &scene, &known, &bound, &mut complaints);
    // In line order, so what the orb says about a spell reads down the file
    // however the faults were found.
    complaints.sort_by_key(|complaint| complaint.line);
    Program::new(body, complaints)
}

/// Say so about every call naming a part the spell does not define.
///
/// **At cast rather than at the call**, which is the difference between a player
/// finding out when they write the spell and finding out on whichever tick the
/// line is reached — possibly never, if it is inside a branch. `run::called`
/// still answers for it, because a definition can be deleted while the spell
/// runs (§8), but by then it is a report about an edit rather than about a typo.
///
/// Recursive, because a call can be anywhere a command can.
fn check_calls(body: &Block, defined: &[String], complaints: &mut Vec<Complaint>) {
    for Step { line, kind } in body {
        match kind {
            Kind::Call { name } => {
                if !defined.iter().any(|part| part == name) {
                    complaints.push(Complaint {
                        line: *line,
                        key: "spell_no_such_part",
                    });
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
            Kind::Command(_) | Kind::Wait(_) | Kind::Let { .. } => {}
        }
    }
}

/// Read each command's **verb** at cast, and say so when it is one a spell may
/// not issue.
///
/// # Why the verb is knowable here and the arguments are not
///
/// A spell makes its own inputs. `digest ground-sage` is written above the line
/// that produces any, so at cast the room has none and `analyse` drops the
/// argument — `interpret` reads that line back as bare `digest`. **Freezing a
/// whole `Intent` at cast would therefore break every pipeline spell in the
/// game, silently**, which is what §19 records this pass being scoped down from.
///
/// The **verb** survives, because a verb is offered by the fixture standing in
/// the room rather than by what is on the shelf: `mix`, `distil` and `digest`
/// all read back at cast with their arguments gone and their verb intact. So the
/// verb is the part that can be checked before the line ever runs.
///
/// # `may_issue` is a security boundary, and it was answered too late
///
/// `run_line` asks it when the line is **reached**, which for a line inside a
/// branch may be never and for a bound spell may be hours after it was written.
/// A `meditate 3600` sitting in an untaken branch said nothing at all — and its
/// own doc calls a scripted `meditate` a hazard, because `Sim::step` drains
/// `Skip` in a while-loop and an hour of world time runs inside one step.
///
/// Asked here as well, so the answer arrives when the spell is cast. **As well,
/// not instead**: `run_line` keeps its check, because a boundary with one guard
/// is a boundary that a future caster can walk around, and its doc already
/// records `quit` being missed from the list once.
fn check_commands(body: &Block, scene: &Scene, bound: &[String], complaints: &mut Vec<Complaint>) {
    for Step { line, kind } in body {
        match kind {
            Kind::Command(text) => {
                // **A line naming something the spell binds is left alone.** A
                // variable holds nothing until the line runs, so resolving one
                // here would read `follow way` as a `follow` with no bearing and
                // report a fault about a line that is perfectly good.
                if names_a_binding(text, bound) {
                    continue;
                }
                let Resolution::Resolved { intent, .. } =
                    crate::parser::analyse(text, scene, Mode::Calm).resolution
                else {
                    continue;
                };
                if !super::run::may_issue(intent.verb) {
                    // **Its own key, not `run_line`'s.** A complaint is filled
                    // with `name` and `count` and nothing else, so the runtime
                    // key's `{detail}` would reach the player unsubstituted —
                    // which is the orb saying `'{detail}'` out loud.
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
            Kind::Wait(_) | Kind::Let { .. } | Kind::Call { .. } => {}
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
                // **A guard's names are fixed against the room exactly as an
                // `if`'s are.** It is the same condition grammar answered by the
                // same `watch::holds`, so a name it could not place has to fail
                // the same way — otherwise `repeat until the mortr is idle` would
                // resolve quietly at cast and then never end.
                Kind::Repeat {
                    times,
                    mut until,
                    body,
                } => {
                    let unplaced = until
                        .as_mut()
                        .map(|condition| fix(condition, scene, known, bound))
                        .unwrap_or_default();
                    // **Nought turns, not unbounded.** Dropping the guard and
                    // leaving `times` at `None` — which is what a *guarded*
                    // repeat carries — turns a loop the orb could not read into
                    // one that never stops, which is the opposite of what
                    // `spell_unreadable_until` tells the player.
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
                    // **A word that names nothing at all makes the question
                    // unreadable**, rather than a question about a thing that
                    // happens not to be here — and it is said at cast, like the
                    // faults the parser finds, because it is the same kind of
                    // fault. See [`fix`] for the difference, which is the whole
                    // of why a typo cannot answer quietly.
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
                // **The body is resolved, and forgetting that is silent.** A
                // `for each` whose block never went through this would run with
                // every name exactly as typed — so `if way has spoil` would ask
                // about a *thing* called `spoil` the room never resolved, and a
                // misspelt line inside a loop would be the one place in the
                // language where a typo answered no for ever.
                Kind::Each { group, body } => Kind::Each {
                    group,
                    body: resolved(body, scene, known, bound, complaints),
                },
                // **The value is a name and is resolved like one**, so
                // `set m to mortar` binds `mortar_and_pestle`. A name the spell
                // itself binds is left alone — `set best to way` is the whole
                // point of an accumulator inside a `for each`.
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

/// Resolve one question's names in place, answering whether it can be asked at
/// all.
///
/// # A place and a thing fail differently, and that is the whole rule
///
/// A **place** that cannot be found makes the question *unanswerable*: §8's
/// *Referent missing*. It is left as the player typed it, so `holds` finds
/// nothing and the runner names the word — every cast, because the tower may
/// have changed since.
///
/// A **thing** that cannot be found is usually just an answer of no.
/// `if the dispensary has ground-sage` is the commonest question in the game and
/// it is asked *before* there is any — treating an absent product as a fault
/// would break the loop the whole feature exists for.
///
/// So a thing is looked for in the room first, and failing that in
/// [`Recipes::vocabulary`](crate::content::Recipes::vocabulary) — every name any
/// recipe can produce or consume, whether or not one exists right now. That
/// distinction is not new: `scribe` used the same list, for the same stated
/// reason, *"a reagent's **name** is a fixed property of the recipes while its
/// **presence** is not"*.
///
/// **A word in neither is a typo, and returns `false` here**: the question is
/// unreadable, said once at cast, and neither branch runs. It cannot be left to
/// answer no, which is what it did while this returned nothing — `has
/// ground-slat` was indistinguishable from `has ground-salt` on an empty shelf,
/// for ever, in silence. That is the whole complaint this work started from,
/// wearing its other face.
///
/// # It reports what it could not place, rather than being asked twice
///
/// **This returned a bare `bool` and the editor re-derived the rest**, which is
/// two expressions of one rule and they disagreed on both halves: a thing found
/// only in the recipes (`ash`, `phlegm` — every byproduct) counted as placed
/// here and unplaced there, so `interpret` painted a working line red; and a
/// place was re-checked against `NounKind::Any`, so `if sage is idle` was
/// reported clean and then failed at run time. The rule is here; nothing else
/// gets to have an opinion about it.
/// # Every name, not the first
///
/// It stopped at the first failure, so `if the mortr is idle and the dispensary
/// has sagg` reported `mortr` and said nothing at all about `sagg`. That is the
/// rule `watch::every` states from the other end — *"§8.1's rule is that the
/// culprit is never anonymous, not that one culprit is enough"* — and the
/// compile half and the runtime half disagreeing on it is how a player fixes one
/// typo, casts again, and is told about the next one.
fn fix(
    condition: &mut Condition,
    scene: &Scene,
    known: &[&str],
    bound: &[String],
) -> Vec<Unplaced> {
    let mut unplaced = Vec::new();
    condition.rename(&mut |kind, name| {
        // **A name the spell binds is left exactly as written and is not a
        // fault.** It stands for a place that will be known when the line runs
        // and is not one now, so resolving it here is impossible and reporting
        // it would put `spell_nowhere` on every correct `for each` in the game.
        //
        // Left alone rather than substituted, because the *value* is what gets
        // resolved — at the `set` that binds it, in this same pass.
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
/// **A typo outranks a missing place**, because the two are not the same size of
/// wrong: a place the tower does not have leaves a question that stands and
/// cannot be answered, and a word that names nothing leaves a question that
/// cannot be asked. Every offender of the winning kind is named.
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
///
/// The two fail differently — see [`fix`] — and the difference decides both what
/// the orb says and whether the question can be asked at all, so it is a type
/// rather than a flag and a comment.
enum Unplaced {
    /// A place the tower does not have. The question stands and is unanswerable;
    /// the runner names it every casting, because the tower may change.
    Place(String),
    /// A word that names nothing at all — not in the room, not in the recipes.
    /// A typo, and the question cannot be asked.
    Thing(String),
}

/// What `name` clearly names in `scene`, or `None` if it is not clear.
///
/// Three ways to be unclear, and all three answer the same: nothing is near
/// enough; the best is a typo rather than an abbreviation; or two things are
/// equally close and picking one would be a guess.
fn clearly(scene: &Scene, kind: NounKind, name: &str) -> Option<String> {
    let words: Vec<&str> = name.split_whitespace().collect();
    let found = scene.candidates(kind, &words);
    let best = found.first()?;
    if best.score < SPELL_SIMILARITY {
        return None;
    }
    let winner = leaf(&best.name);

    // **Two entries for one word are one candidate, not a tie.** §6.1 registers
    // a `Topic` beside every reagent so `recall ground-sage` reads the manual —
    // which put `ground-sage` in the scene twice, at identical scores, and made
    // the runner-up a copy of the winner. `ground-sag` was refused as ambiguous
    // between a thing and its own manual entry.
    //
    // Found by the test that measures the margin against the real vocabulary,
    // which reported a name beating *itself* by nothing.
    let runner_up = found.iter().find(|next| leaf(&next.name) != winner);

    // An exact match is never a coin flip, whatever else is near it: `ground-sage`
    // means `ground-sage` even with `ground-salt` on the shelf beside it.
    if best.score < EXACT
        && runner_up.is_some_and(|next| best.score.saturating_sub(next.score) < SPELL_MARGIN)
    {
        return None;
    }
    Some(winner.to_owned())
}

/// Every distinct name a scene could answer `name` with, best first.
///
/// Exposed for the tie rule above — `best_match` keeps only the winner, which is
/// exactly the information a coin flip has to be detected from.
#[must_use]
pub fn candidates(scene: &Scene, kind: NounKind, name: &str) -> Vec<NounMatch> {
    let words: Vec<&str> = name.split_whitespace().collect();
    scene.candidates(kind, &words)
}

/// Read `lines` without resolving anything — for the parser's own tests, and for
/// the editor, which reports on a buffer rather than running it.
///
/// Returns a [`Draft`] rather than a [`Program`] for the reason [`compile`]
/// gives: the type is what keeps an unresolved program from being run.
#[must_use]
pub fn read(lines: &[String]) -> Draft {
    super::program::read(lines)
}

/// One line of a spell, as the orb reads it.
///
/// # The rewriter, turned the right way round
///
/// This is the work `scribe::canonicalise` used to do, and very nearly the same
/// code — with one difference that is the whole point: it **returns** the
/// reading instead of writing it into the player's file. The thing that was
/// dangerous as a mutation is exactly what is wanted as a report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    /// Which line, counting from one, as the player sees it.
    pub line: usize,
    /// What the orb hears: the canonical command, the resolved question, or the
    /// line itself where it is the player's own (a comment, a blank).
    pub heard: String,
    /// Why the orb cannot read it, if it cannot.
    pub fault: Option<Fault>,
}

/// What is wrong with a line, in a form prose can be built from.
///
/// Deliberately **not** [`Complaint`](super::Complaint), which carries a prose
/// key and nothing else. Half of what this reports is a *name* — the word the
/// room could not place — and a `&'static str` has nowhere to put it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    /// The prose key for what is wrong.
    pub key: &'static str,
    /// The word at fault, where there is one.
    pub detail: Option<String>,
}

/// Read `lines` as a spell for `domain`, line by line, without running anything.
///
/// What the editor draws `interpret` and its marks from. Pure, and over the
/// **buffer** rather than the saved file, so the answer tracks what is on screen.
///
/// A line the orb understands reads back as what it heard; one it does not is
/// returned as typed, with the fault beside it. Nothing here changes the buffer:
/// that is the difference between this and the rewriter it is descended from.
#[must_use]
pub fn interpret(world: &World, domain: &str, lines: &[String]) -> Vec<Reading> {
    let Some(at) = crate::execute::find_domain(world, domain) else {
        return lines
            .iter()
            .enumerate()
            .map(|(index, line)| Reading {
                line: index + 1,
                heard: line.trim().to_owned(),
                fault: Some(Fault {
                    key: "spell_homeless",
                    detail: Some(domain.to_owned()),
                }),
            })
            .collect();
    };
    let scene = crate::tower::scene_at(world, at);
    let known = world.resource::<crate::content::Recipes>().vocabulary();

    // Faults about the *shape* of the file — a block nothing closed, a stray
    // `end` — belong to a line but are found by reading the whole thing.
    let draft = super::program::read(lines);
    let mut structural = draft.complaints;
    // **The same check the cast makes, on the same tree.** `interpret` and the
    // runner disagreeing about a line is §19's recurring defect in this file;
    // a call to a part nobody defined is exactly the kind of fault this surface
    // exists to show before it runs.
    let defined: Vec<String> = super::program::parts(&draft.body)
        .into_iter()
        .map(str::to_owned)
        .collect();
    check_calls(&draft.body, &defined, &mut structural);
    // **The whole file's bindings, for every line of it.** A `set` on line 9 is
    // a name line 2 may already say — `bindings` is deliberately not scoped, and
    // this surface has to agree with the runner about that or it would paint a
    // working line red.
    let bound = super::program::bindings(&draft.body);

    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let at = index + 1;
            let mut reading = one(line, &scene, &known, &bound);
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
        fault,
    };
    // The orb's reading where it differs from the text — a call written
    // `gathering ()` is heard as `gathering()`, which is the form the player has
    // to type and the one this surface is for showing them.
    let verbatim_as = |heard: &str, fault: Option<Fault>| Reading {
        line: 0,
        heard: heard.to_owned(),
        fault,
    };

    // A blank line and a comment are the player's own, and the orb has nothing
    // to say about either.
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return verbatim(None);
    }

    // **A call, before the command resolver sees it — and after the language's
    // own words, which is the order `read` uses.**
    //
    // `gathering()` is not a word the tower has, so falling through to the
    // resolver would report *"no such thing"* about a line that is perfectly
    // good; this surface exists to show a wrong resolution, not to invent one.
    // Whether the part is *defined* is a fault about the file, and arrives from
    // `check_calls` with the structural ones.
    //
    // The `spell_word` guard is what stops a **definition** being read as a
    // malformed call. `call_name("part gathering()")` splits at the first `(`,
    // finds a two-word head, and answers `Some(None)` — *a call with something
    // in front of it* — so without this the canonical `part <name>()` fell into
    // `spell_part_takes_nothing` and `interpret` reported the one form the
    // language teaches as a line the orb could not read. `read` got it right
    // because it reaches `call_name` only in `spell_word`'s `None` arm, so the
    // spell compiled and ran while the surface built to catch bad lines lied
    // about it.
    if crate::parser::spell_word(trimmed).is_none()
        && let Some(called) = super::program::call_name(trimmed)
    {
        return match called {
            Some(name) => verbatim_as(&format!("{name}()"), None),
            None => verbatim(Some(Fault {
                key: "spell_part_takes_nothing",
                detail: None,
            })),
        };
    }

    if let Some(word) = crate::parser::spell_word(trimmed) {
        // **`repeat until` carries the same grammar an `if` does**, resolved by
        // the same `fix`, so it belongs on this surface for the same reason. It
        // was excluded, which left `interpret` — the one place a wrong resolution
        // can be seen *before* it runs — covering half the language: a guard with
        // a misspelt place came back exactly as typed, with no fault and no
        // change to the "cannot read" count.
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
        // A name the room cannot place: the one thing the file used to reveal by
        // being rewritten, and the reason this surface exists at all. **Asked of
        // `fix`**, which is the only thing that knows the rule — see there.
        let fault = fault_of(&fix(&mut question, scene, known, bound));
        return Reading {
            line: 0,
            heard: format!("{lead} {}", crate::parser::write_condition(&question)),
            fault,
        };
    }

    // **A line naming something the spell binds is quoted, never resolved.**
    // `follow best` read back as **`follow west`** — the fuzzy matcher finding
    // the nearest place in the room, which is the one thing `best` is certainly
    // not. The runner is right (it substitutes the bound value before `analyse`
    // ever sees the line); this surface was the only liar, which is the exact
    // shape of the bug it exists to catch, one grammar wider.
    //
    // Verbatim is the honest answer rather than a shortcut: what the orb hears
    // is *"follow whatever `best` is"*, and it cannot know that until the line
    // runs. The same call `names_a_spell` makes below, for the same reason.
    if trimmed
        .split_whitespace()
        .any(|word| bound.iter().any(|held| held.eq_ignore_ascii_case(word)))
    {
        return verbatim(None);
    }

    // A command, read the way the runner will read it — through the same
    // `analyse` a typed line goes through, against the spell's own room.
    let resolution = analyse(line, scene, Mode::Calm).resolution;

    // **A line naming another spell is quoted, never resolved.** §8's own worked
    // example is `night_watch` invoking `brew_clarity`, and a player writes those
    // in whichever order they think of them — so the spell being named routinely
    // does not exist yet. The parser weights the verb double (`resolve::score`),
    // so a perfect `invoke` with a meaningless argument still clears
    // `MIN_SIMILARITY`: `invoke not_written_yet` read back as
    // `invoke first_light.spell`, which is the orb telling the player, with
    // confidence, that it will run a spell they did not name.
    //
    // That is the bug §19 records being written *into the file* once already.
    // The file is safe now; this is the same wrong answer on the surface that
    // replaced it, which is worse in one way — a rewritten file could at least
    // be read back and disbelieved.
    if names_a_spell(&resolution) {
        return verbatim(None);
    }

    let Resolution::Resolved { intent, .. } = resolution else {
        return verbatim(Some(Fault {
            key: "spell_missing",
            detail: Some(trimmed.to_owned()),
        }));
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
        fault: None,
    }
}

/// Whether this reading is a spell naming another spell.
///
/// **`Incomplete` counts.** `invoke brew_clarity` where `brew_clarity` does not
/// exist yet leaves the `Script` slot unfillable, which is exactly the case this
/// exists for — reporting only the readings that *did* resolve would miss it.
///
/// Lifted out of the rewriter this replaced, where it guarded the file. It now
/// guards what the orb *says* about the file, which is the only surface left
/// that can get this wrong.
fn names_a_spell(resolution: &Resolution) -> bool {
    let verb = match resolution {
        Resolution::Resolved { intent, .. } => intent.verb,
        Resolution::Incomplete { verb, .. } => *verb,
        // **Ambiguous counts too, and this was the hole.** A forward reference
        // matches *no* spell well, so which resolution it produces depends on how
        // many spells happen to exist: with one on the shelf `invoke
        // not_written_yet` resolved (the verb is weighted double) and was quoted;
        // with four it ties between them and came back `Ambiguous`, which fell
        // through to `spell_missing` and called a forward reference a fault.
        //
        // Shelving the dev ladders in a debug build is what made four, but the
        // defect was always there — a player with four spells of their own would
        // have found it. The rule is *this line names a spell*, and a tie between
        // spells is still that.
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
