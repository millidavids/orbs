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
    let body = resolved(draft.body, &scene, &known, &mut complaints);
    // In line order, so what the orb says about a spell reads down the file
    // however the faults were found.
    complaints.sort_by_key(|complaint| complaint.line);
    Program::new(body, complaints)
}

/// Every condition in a block, with its names fixed.
fn resolved(body: Block, scene: &Scene, known: &[&str], complaints: &mut Vec<Complaint>) -> Block {
    body.into_iter()
        .map(|Step { line, kind }| Step {
            line,
            kind: match kind {
                Kind::Repeat { times, body } => Kind::Repeat {
                    times,
                    body: resolved(body, scene, known, complaints),
                },
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
                        .map(|condition| fix(condition, scene, known))
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
                        body: resolved(body, scene, known, complaints),
                        otherwise: resolved(otherwise, scene, known, complaints),
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
fn fix(condition: &mut Condition, scene: &Scene, known: &[&str]) -> Vec<Unplaced> {
    let mut unplaced = Vec::new();
    condition.rename(&mut |kind, name| {
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
    let structural = super::program::read(lines).complaints;

    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let at = index + 1;
            let mut reading = one(line, &scene, &known);
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
fn one(line: &str, scene: &Scene, known: &[&str]) -> Reading {
    let trimmed = line.trim();
    let verbatim = |fault: Option<Fault>| Reading {
        line: 0,
        heard: trimmed.to_owned(),
        fault,
    };

    // A blank line and a comment are the player's own, and the orb has nothing
    // to say about either.
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return verbatim(None);
    }

    if let Some(word) = crate::parser::spell_word(trimmed) {
        if word != crate::parser::SpellWord::If {
            return verbatim(None);
        }
        let Some(mut question) = crate::parser::condition(crate::parser::spell_argument(trimmed))
        else {
            return verbatim(Some(Fault {
                key: "spell_unreadable_if",
                detail: None,
            }));
        };
        // A name the room cannot place: the one thing the file used to reveal by
        // being rewritten, and the reason this surface exists at all. **Asked of
        // `fix`**, which is the only thing that knows the rule — see there.
        let fault = fault_of(&fix(&mut question, scene, known));
        return Reading {
            line: 0,
            heard: format!("if {}", crate::parser::write_condition(&question)),
            fault,
        };
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
const fn names_a_spell(resolution: &Resolution) -> bool {
    let verb = match resolution {
        Resolution::Resolved { intent, .. } => intent.verb,
        Resolution::Incomplete { verb, .. } => *verb,
        _ => return false,
    };
    matches!(verb, Verb::Invoke | Verb::Bind)
}
