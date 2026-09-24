//! Reading a spell's text into something the orb can run.
//!
//! §8 asked for loose phrasing to be resolved *"at authoring time"*. That is met
//! here, but the moment is **cast** and the result lives in the program, not in
//! the file: a rewriter that understands part of a line writes the rest out of
//! existence (§19), and a rewritten file is not the player's any more.
//!
//! Cast beats save too — the names are fixed in the world the spell is about to
//! run in, and one that stops matching later is reported every casting (§8.1).
//!
//! The orb accepts an abbreviation, never a typo and never a coin flip. §6's
//! matcher is forgiving because the player is *there* to see the echo, where a
//! spell resolves with nobody watching, so it is held to [`SPELL_SIMILARITY`].

use bevy_ecs::prelude::*;

use crate::parser::{
    Condition, EXACT, Mode, NounKind, NounMatch, Resolution, Scene, Verb, analyse, leaf,
};

use super::program::{Block, Complaint, Draft, Kind, Program, Step};

/// How alike a name must be to what it names, inside a spell.
///
/// 850 is where the scale already divides. `fuzzy` scores a *prefix* of three
/// characters or more at `PREFIX_FLOOR` (850) plus coverage, so an abbreviation
/// is always at or above the line — `mortar` → `mortar_and_pestle` is 902 — and
/// a *typo*, scored by edit distance, only reaches it as a single wrong
/// character in a long word. *"Prefixes are intentional… typos are
/// accidental."*
///
/// The case that forced it: `similarity("ground-salt", "ground-sage")` is 819,
/// over the prompt's floor of 600, so a question written while the salt was
/// absent compiled into a question about the sage.
pub const SPELL_SIMILARITY: u32 = 850;

/// How far ahead of the runner-up a name must be to be the one meant.
///
/// A floor alone cannot help when two things are *equally* close: with both
/// products on the shelf, `ground` is a 931 prefix of `ground-sage` and of
/// `ground-salt` alike, and `Scene::best_match` would hand back whichever was
/// registered first. Command lines in a spell already refuse on ambiguity, so
/// this brings questions into line rather than inventing a policy for them.
///
/// It was `1` — a margin in name and a tie in behaviour. Against the shipped
/// vocabulary nothing lands between 1 and 54, so the two rules cannot be told
/// apart today; the ceiling is `sag` → `sage` at 67, a legitimate abbreviation
/// that has to keep working, so 50 is the room there is.
/// `the_margin_is_below_the_closest_call` measures that lead and fails when a
/// new name squeezes it.
pub const SPELL_MARGIN: u32 = 50;

/// Whether one line stands on its own as a spell **statement**.
///
/// The check a reader's output has to pass before it may replace a line. A
/// function rather than a `program::read` at each site, because three complaints
/// are earned *purely for being on one line* — `spell_unclosed` (a lone `if`,
/// `repeat`, `for` or `part`), `spell_stray_end` and `spell_stray_else` — and a
/// caller comparing `complaints.is_empty()` would refuse most of the control
/// flow the scrivener exists for.
///
/// A command line answers `false`: it is a statement the *prompt's* reader owns.
///
/// ⚠ Parsing is not understanding. `if the alembic has finished` passes this —
/// as *"holds a thing called finished"* — and means something else entirely, so
/// a reader's output must also account for every word the player wrote.
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
    // Two ways, because `end` and `else` produce no step: a body check alone
    // would call the two commonest words in the language unreadable. A call
    // takes the other route — `morning()` opens on no spell word.
    crate::parser::spell_word(line).is_some()
        || draft
            .body
            .first()
            .is_some_and(|step| !matches!(step.kind, super::program::Kind::Command(_)))
}

/// The lines a spell compiles from: its reading, or its text if it has none.
///
/// One place, because two would disagree: casting, the mid-flight reload and
/// restoring a save all ask this.
///
/// [`Read`](crate::tower::Read) is derived and byte-equal to
/// [`Held`](crate::tower::Held) unless something read the file. It falls back
/// when the component is absent, and line by line where a reading was not read
/// from the text now at its place (see `Read::compiled`) — comparing only line
/// counts meant a save with one hand-edited line compiled the reading of the
/// line that used to be there.
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
/// The only way a [`Program`] is made. `program::read` returns a [`Draft`],
/// which nothing can run and nothing can store on [`Running`](super::Running),
/// so a future call site cannot skip the resolution. Visibility would not do it:
/// both callers are already in this crate.
#[must_use]
pub fn compile(world: &World, from: Entity, lines: &[String]) -> Program {
    let at = crate::tower::domain_of(world, from).unwrap_or(from);
    let scene = crate::tower::scene_at(world, at);
    let known = world.resource::<crate::content::Recipes>().vocabulary();
    let draft = super::program::read(lines);
    let mut complaints = draft.complaints;
    // The names the spell binds, gathered first. They are lexical, and a bound
    // name reaches `fix` looking exactly like a place the room does not have.
    let bound = super::program::bindings(&draft.body);
    // Against the text rather than the room, since a part is a name the *spell*
    // defines. Asking it here keeps `interpret` and the cast agreeing (§19).
    let defined: Vec<(String, usize)> = super::program::parts(&draft.body)
        .into_iter()
        .map(|(name, takes)| (name.to_owned(), takes))
        .collect();
    check_calls(&draft.body, &defined, &mut complaints);
    check_commands(&draft.body, &scene, &bound, &mut complaints);
    check_learned(world, &draft.body, &mut complaints);
    let body = resolved(draft.body, &scene, &known, &bound, &mut complaints);
    // In line order, so what the orb says about a spell reads down the file
    // however the faults were found.
    complaints.sort_by_key(|complaint| complaint.line);
    Program::new(body, complaints)
}

/// Say so about every word the loom has not granted yet.
///
/// `queue` is a verb and refuses in voice where it is typed; `pull` and
/// `alongside` are control words with nobody to answer, so the report comes from
/// here — where every other unreadable line is reported, before casting rather
/// than on whichever tick it is reached.
///
/// A complaint, so the rest of the spell still runs: §8 forbids refusing at save
/// and halting at cast, so a loop above a `pull` goes round doing the half it
/// can.
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
/// `run::called` still answers for it, since a definition can be deleted while
/// the spell runs (§8), but by then it is a report about an edit, not a typo.
///
/// Recursive, because a call can be anywhere a command can.
fn check_calls(body: &Block, defined: &[(String, usize)], complaints: &mut Vec<Complaint>) {
    for Step { line, kind } in body {
        match kind {
            // A fork is checked exactly as a call is: `alongside missing()` and
            // `missing()` are the same mistake, and a fork whose part does not
            // exist would start a cursor on nothing.
            Kind::Call { name, args } | Kind::Alongside { name, args } => {
                match defined.iter().find(|(part, _)| part == name) {
                    None => complaints.push(Complaint {
                        line: *line,
                        key: "spell_no_such_part",
                    }),
                    // Wrong count is its own complaint, not a missing part. A
                    // part takes names positionally, so `between(wellspring)`
                    // has nothing to put in `there` — and binding it to nothing
                    // leaves the body asking about a name that resolves against
                    // the room and does the wrong thing quietly.
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
/// The verb is knowable here and the arguments are not. A spell makes its own
/// inputs: `digest ground-sage` is written above the line that produces any, so
/// at cast the room has none and `analyse` drops the argument. Freezing a whole
/// `Intent` here would break every pipeline spell in the game (§19); the verb
/// survives because the fixture standing in the room offers it.
///
/// `may_issue` is a security boundary and was answered too late — `run_line`
/// asks it when the line is *reached*, so a `meditate 3600` in an untaken branch
/// said nothing, and `Sim::step` drains `Skip` in a while-loop.
///
/// Asked here as well, not instead: a boundary with one guard is one a future
/// caster can walk around.
fn check_commands(body: &Block, scene: &Scene, bound: &[String], complaints: &mut Vec<Complaint>) {
    for Step { line, kind } in body {
        match kind {
            Kind::Command(text) => {
                // A line naming something the spell binds is left alone: a
                // variable holds nothing until the line runs, so resolving one
                // here reads `follow way` as a `follow` with no bearing.
                if names_a_binding(text, bound) {
                    continue;
                }
                let Resolution::Resolved { intent, .. } =
                    crate::parser::analyse(text, scene, Mode::Calm).resolution
                else {
                    continue;
                };
                if !super::run::may_issue(intent.verb) {
                    // Its own key, not `run_line`'s: a complaint carries only
                    // `name` and `count`, so the runtime key's `{detail}` would
                    // reach the player unsubstituted.
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
            // `bide` is checked at parse: this asks whether a *verb* is one the
            // room offers, and a bide has no verb.
            //
            // `pull` is out for a sharper reason: it names a *place*, resolved
            // by `run::pull` in the room the spell stands in. `scene` is built
            // from where the **player** is, so checking it here would fault a
            // good line whenever a bound solver worked while the player was
            // elsewhere — §19's Cwd-versus-spell-room defect.
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
                // A guard's names are fixed exactly as an `if`'s are, or `repeat
                // until the mortr is idle` resolves quietly at cast and then
                // never ends.
                Kind::Repeat {
                    times,
                    mut until,
                    body,
                } => {
                    let unplaced = until
                        .as_mut()
                        .map(|condition| fix(condition, scene, known, bound))
                        .unwrap_or_default();
                    // Nought turns, not unbounded. Leaving `times` at `None` —
                    // what a *guarded* repeat carries — turns a loop the orb
                    // could not read into one that never stops.
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
                    // A word that names nothing makes the question unreadable,
                    // rather than a question about a thing that happens not to
                    // be here. See [`fix`] for the difference.
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
                // The body is resolved, and forgetting that is silent: a `for
                // each` whose block skipped this would run with every name as
                // typed, and a misspelt line inside a loop is the one place a
                // typo answers no for ever.
                Kind::Each { group, body } => Kind::Each {
                    group,
                    body: resolved(body, scene, known, bound, complaints),
                },
                // The value is a name and is resolved like one, so `set m to
                // mortar` binds `mortar_and_pestle`. A name the spell itself
                // binds is left alone — `set best to way` is an accumulator
                // inside a `for each`.
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
/// A *place* that cannot be found makes the question unanswerable (§8's
/// *Referent missing*). It is left as the player typed it, so `holds` finds
/// nothing and the runner names the word — every cast, because the tower may
/// have changed since.
///
/// A *thing* that cannot be found is usually just an answer of no: `if the
/// dispensary has ground-sage` is asked *before* there is any. So a thing is
/// looked for in the room first and then in
/// [`Recipes::vocabulary`](crate::content::Recipes::vocabulary) — every name any
/// recipe can produce or consume, present or not. A word in neither is a typo:
/// unreadable, said once at cast, and neither branch runs, because `has
/// ground-slat` left to answer no is indistinguishable from an empty shelf.
///
/// It reports what it could not place rather than being asked twice: as a bare
/// `bool` with the editor re-deriving the rest, a byproduct counted as placed
/// here and unplaced there, and a place re-checked against `NounKind::Any` read
/// clean and then failed at run time.
///
/// Every name, not the first: stopping at one, `if the mortr is idle and the
/// dispensary has sagg` said nothing about `sagg` (§8.1).
fn fix(
    condition: &mut Condition,
    scene: &Scene,
    known: &[&str],
    bound: &[String],
) -> Vec<Unplaced> {
    let mut unplaced = Vec::new();
    condition.rename(&mut |kind, name| {
        // A name the spell binds is left as written and is not a fault: it
        // stands for a place known only when the line runs. Left alone rather
        // than substituted, because the *value* is what gets resolved, at the
        // `set` that binds it.
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
/// A typo outranks a missing place: a place the tower does not have leaves a
/// question that stands and cannot be answered, where a word that names nothing
/// leaves one that cannot be asked. Every offender of the winning kind is named.
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
/// The two fail differently (see [`fix`]), and the difference decides whether
/// the question can be asked at all — so a type rather than a flag.
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

    // Two entries for one word are one candidate, not a tie. §6.1 registers a
    // `Topic` beside every reagent, which put `ground-sage` in the scene twice
    // at identical scores — so `ground-sag` was refused as ambiguous with its
    // own manual entry.
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
/// The rewriter turned the right way round: `scribe::canonicalise`'s work,
/// except that it *returns* the reading instead of writing it into the player's
/// file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    /// Which line, counting from one, as the player sees it.
    pub line: usize,
    /// What the orb hears: the canonical command, the resolved question, or the
    /// line itself where it is the player's own (a comment, a blank).
    pub heard: String,
    /// What the player wrote, when a reader turned it into something else.
    ///
    /// Without it a line read *correctly* and one read *wrongly* look identical
    /// in the editor. `None` where [`heard`](Self::heard) came from the line as
    /// typed, which is every line in a build with no reader.
    pub was: Option<String>,
    /// Why the orb cannot read it, if it cannot.
    pub fault: Option<Fault>,
}

/// What is wrong with a line, in a form prose can be built from.
///
/// Not [`Complaint`], which carries a prose key and nothing else: half of what
/// this reports is a *name*, and a `&'static str` has nowhere to put it.
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
/// A line the orb understands reads back as what it heard; one it does not is
/// returned as typed, with the fault beside it. Nothing here changes the buffer,
/// which is the difference between this and the rewriter it descends from.
///
/// `read` is each line as it would compile, looked up where the save does
/// (`Sim::read_spell_with`), so a line the save already read is not read again.
/// A line the reader changed carries the player's own text in [`Reading::was`].
///
/// Everything file-shaped is asked of the reading, not the text: judging each
/// line on its reading while finding blocks and bindings in the buffer made the
/// two disagree exactly where a reader had helped (§19).
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

    // Faults about the *shape* of the file — a block nothing closed, a stray
    // `end` — belong to a line but are found by reading the whole thing.
    let draft = super::program::read(&compiled);
    let mut structural = draft.complaints;
    // The same check the cast makes, on the same tree: `interpret` and the
    // runner disagreeing about a line is §19's recurring defect in this file.
    let defined: Vec<(String, usize)> = super::program::parts(&draft.body)
        .into_iter()
        .map(|(name, takes)| (name.to_owned(), takes))
        .collect();
    check_calls(&draft.body, &defined, &mut structural);
    // The whole file's bindings, for every line of it: a `set` on line 9 is a
    // name line 2 may already say, and disagreeing with the runner here paints a
    // working line red.
    let bound = super::program::bindings(&draft.body);

    lines
        .iter()
        .zip(&compiled)
        .enumerate()
        .map(|(index, (line, heard))| {
            let at = index + 1;
            // What the orb would compile, then what it makes of that.
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
    // The orb's reading where it differs from the text — a call written
    // `gathering ()` is heard as `gathering()`, which is the form the player has
    // to type and the one this surface is for showing them.
    let verbatim_as = |heard: &str, fault: Option<Fault>| Reading {
        line: 0,
        heard: heard.to_owned(),
        was: None,
        fault,
    };

    // A blank line and a comment are the player's own, and the orb has nothing
    // to say about either.
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return verbatim(None);
    }

    // A call, before the command resolver sees it and after the language's own
    // words, which is the order `read` uses. `gathering()` is not a word the
    // tower has, so falling through to the resolver would report *"no such
    // thing"* about a good line; whether the part is *defined* arrives from
    // `check_calls`.
    //
    // The `spell_word` guard stops a *definition* being read as a malformed
    // call: `call_name("part gathering()")` finds a two-word head and answers
    // `Some(None)`, so without it the canonical `part <name>()` was reported
    // unreadable here while `read` compiled and ran it.
    if crate::parser::spell_word(trimmed).is_none()
        && let Some(called) = super::program::call_of(trimmed)
    {
        return match called {
            // The arguments are written back as names, never resolved: what the
            // orb hears is *"do `between` with whatever `near` is"*, and it
            // cannot know that until the line runs.
            Some((name, args)) => verbatim_as(&format!("{name}({})", args.join(", ")), None),
            None => verbatim(Some(Fault {
                key: "spell_unreadable_call",
                detail: None,
            })),
        };
    }

    if let Some(word) = crate::parser::spell_word(trimmed) {
        // `repeat until` carries the same grammar an `if` does, resolved by the
        // same `fix`. Excluded, a guard with a misspelt place came back as
        // typed, with no fault and no change to the "cannot read" count.
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
        // A name the room cannot place: what the file used to reveal by being
        // rewritten, and the reason this surface exists. Asked of `fix`.
        let fault = fault_of(&fix(&mut question, scene, known, bound));
        return Reading {
            line: 0,
            heard: format!("{lead} {}", crate::parser::write_condition(&question)),
            was: None,
            fault,
        };
    }

    // A line naming something the spell binds is quoted, never resolved.
    // `follow best` read back as `follow west` — the fuzzy matcher finding the
    // nearest place in the room, which is the one thing `best` is certainly not.
    // Verbatim is the honest answer: what the orb hears is *"follow whatever
    // `best` is"*, and it cannot know until the line runs.
    if trimmed
        .split_whitespace()
        .any(|word| bound.iter().any(|held| held.eq_ignore_ascii_case(word)))
    {
        return verbatim(None);
    }

    // A command, read the way the runner will read it — through the same
    // `analyse` a typed line goes through, against the spell's own room.
    let resolution = analyse(line, scene, Mode::Calm).resolution;

    // A line naming another spell is quoted, never resolved: a player writes
    // `night_watch` and the `brew_clarity` it invokes in whichever order they
    // think of them (§8), so the spell being named routinely does not exist yet.
    // The parser weights the verb double, so `invoke not_written_yet` read back
    // as `invoke first_light.spell` — the orb naming a spell the player did not
    // (§19).
    if names_a_spell(&resolution) {
        return verbatim(None);
    }

    let intent = match resolution {
        Resolution::Resolved { intent, .. } => intent,
        // A verb that takes nothing says so here too. A spell answered *"nothing
        // here answers to …"*, which is not what is wrong with the line —
        // `muster the troops` names no missing referent.
        Resolution::TakesNothing { extra, .. } => {
            return verbatim(Some(Fault {
                key: "spell_takes_nothing",
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
/// `Incomplete` counts: `invoke brew_clarity` where `brew_clarity` does not
/// exist yet leaves the `Script` slot unfillable, which is the case this exists
/// for, so reporting only the readings that *did* resolve would miss it.
fn names_a_spell(resolution: &Resolution) -> bool {
    let verb = match resolution {
        Resolution::Resolved { intent, .. } => intent.verb,
        Resolution::Incomplete { verb, .. } => *verb,
        // Ambiguous counts too, and this was the hole: a forward reference
        // matches *no* spell well, so with four on the shelf `invoke
        // not_written_yet` tied between them and fell through to
        // `spell_missing`. The rule is *this line names a spell*, and a tie
        // between spells is still that.
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
