//! Writing a spell down, and canonicalising it as it lands.
//!
//! DESIGN.md §8: *"`bind` resolves loose phrasing to canonical commands **at
//! authoring time** and stores the canonical form. A script executes later, in a
//! different world state, where live-state disambiguation is unavailable."*
//!
//! `bind` was the only authoring event when that was written. **Saving the
//! buffer is now that event** — the doc's own words are "authoring time" — so
//! canonicalisation happens here and `bind`/`invoke` both run text that is
//! already arcane. §19 records the move.
//!
//! # The diegetic half
//!
//! This is the beat §8 calls *the orb writes down what you meant*. A player
//! types `make a potion of clarity` into the buffer and the file says
//! `recall clarity`, because that is what the orb heard. It is the same lesson
//! the echo teaches at the prompt (§6), applied to a file that outlives the
//! moment.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Prose, with_extension};
use crate::parser::{Condition, Intent, Mode, NounKind, Resolution, Scene, Verb, analyse};
use crate::session::Scrollback;
use crate::tower::{self, Cwd, Domain, Held, Name, Nameable, NodeIds};

use super::navigate::find_script;

/// One line of a spell, after the orb has written it down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    /// What the file will hold.
    pub text: String,
    /// Whether the orb understood it.
    ///
    /// §8: *"`bind` always succeeds."* A line that resolved to nothing is kept
    /// **verbatim** and flagged, never dropped and never blocking — a draft you
    /// cannot save is a dead end (§6), and a writer mid-thought saves broken
    /// lines constantly.
    pub understood: bool,
}

/// Canonicalise `lines` as if run in the domain at `from`.
///
/// # One scene, because a spell does not walk
///
/// §10.1's per-instrument verbs only resolve where their instrument is (§7,
/// `Scene::offers`), so `grind sage` is a phrase in the laboratory and nowhere
/// else. A spell is written **for** a domain and runs there, so there is exactly
/// one scene to resolve every line against.
///
/// This replaced a simulated cwd that walked through the spell's own `attend`
/// lines, and the replacement is a **precondition for control flow** rather than
/// a simplification. A walk executes a `repeat` body once at authoring time and
/// N times at run time; worse, with an `if` it is *undecidable* — you cannot
/// know at save time which branch was taken, so you cannot know which room line
/// 9 belongs to. §8 fixes canonicalisation at authoring time, so the walk and
/// control flow could never both exist.
#[must_use]
pub fn canonicalise(world: &World, from: Entity, lines: &[String]) -> Vec<Written> {
    // The domain, or the place itself if it is not in one — a fixture resolves
    // to the room it stands in, which is where its verbs live.
    let at = tower::domain_of(world, from).unwrap_or(from);
    let scene = tower::scene_at(world, at);
    let mut out = Vec::with_capacity(lines.len());
    // How many blocks are open, so the orb can indent its fair copy.
    let mut depth = 0usize;

    for line in lines {
        // Where this line sits, and where the next one starts. **The rule lives
        // in `parser::spellword`**, not here, because the editor indents as you
        // type and the two must agree — a buffer that indents one way and a file
        // that indents another makes every save look like it moved your work.
        let (here, next) = crate::parser::indent_around(line, depth);
        depth = next;

        // Comments are the player's, kept exactly. A spell is a file someone
        // reads back, and stripping the space they left in it is not the orb's
        // business.
        //
        // A **blank** line is emitted empty rather than as typed, because the
        // editor's auto-indent leaves the caret's level behind on a line the
        // player never put anything on — trailing spaces that draw as nothing
        // and would otherwise be written into the file.
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.push(Written {
                text: String::new(),
                understood: true,
            });
            continue;
        }
        if trimmed.starts_with('#') {
            out.push(Written {
                text: line.clone(),
                understood: true,
            });
            continue;
        }

        // **A control word is kept exactly, and the orb re-indents it.**
        //
        // This is the half of the collision that would have been permanent.
        // `analyse` now answers a spell word with `Resolution::InSpell` rather
        // than guessing — but before it did, `wait for the mortar` resolved to
        // `scribe mortar` and canonicalisation ran **at save**, so the file
        // would have held a command the player never wrote, with a cheerful
        // *"written down"* on the way past.
        //
        // The indentation is the orb's, not the player's: `anchored` strips
        // leading whitespace from every command line anyway, so a hand-indented
        // spell could not round-trip. Re-deriving it from block depth makes the
        // file the orb's fair copy — which is what §8 says a saved spell is.
        if let Some(word) = crate::parser::spell_word(line) {
            // ...**except for the names inside an `if`**, which are resolved
            // against the room the way a command's argument is. See
            // `written_condition`: the word itself is still verbatim.
            let (text, understood) = if word == crate::parser::SpellWord::If {
                written_condition(&scene, trimmed)
            } else {
                (trimmed.to_owned(), true)
            };
            out.push(Written {
                text: format!("{}{text}", crate::parser::INDENT.repeat(here)),
                understood,
            });
            continue;
        }

        let resolution = analyse(line, &scene, Mode::Calm).resolution;

        // **A line that names another spell is kept exactly as typed.**
        //
        // §8's own worked example is `night_watch` invoking `brew_clarity`, and
        // a player writes those in whichever order they think of them — so the
        // spell being named routinely does not exist yet when the line naming it
        // is saved. Two ways that goes wrong, and this catches both:
        //
        // - **It resolves to the wrong spell.** The parser weights the verb
        //   double (`resolve::score`), so a perfect `invoke` with a meaningless
        //   argument still clears `MIN_SIMILARITY` — `invoke two` became
        //   `invoke first_light.spell`, written permanently into the file. A
        //   wrong answer wearing a right one's clothes.
        // - **It resolves to nothing**, and the line is flagged unreadable for
        //   naming a spell the player is about to write.
        //
        // A spell name is durable and nameable from anywhere, so there is no
        // live-state ambiguity for §8's rule to protect against here. The name
        // resolves when the line *runs*, by which time the spell exists.
        if names_a_spell(&resolution) {
            out.push(Written {
                text: verbatim(here, line),
                understood: true,
            });
            continue;
        }

        let Resolution::Resolved { intent, .. } = resolution else {
            // Unresolved, ambiguous or out of scope. Kept as typed and flagged;
            // the orb proposes a correction elsewhere rather than refusing here.
            out.push(Written {
                text: verbatim(here, line),
                understood: false,
            });
            continue;
        };

        // **`attend` is flagged, not refused.** §8 fixes that saving always
        // succeeds and this codebase records why — *"a draft you cannot save is
        // a dead end (§6), and a writer mid-thought saves broken lines
        // constantly."* So the line is kept exactly as typed and marked
        // unreadable, which is the same treatment a typo gets, and the runner
        // refuses it again when the spell is cast.
        if intent.verb == Verb::Attend {
            out.push(Written {
                text: verbatim(here, line),
                understood: false,
            });
            continue;
        }

        // **The orb never writes down fewer reagents than it was given.**
        if dropped_argument(world, line, &intent) {
            out.push(Written {
                text: verbatim(here, line),
                understood: false,
            });
            continue;
        }

        out.push(Written {
            text: format!(
                "{}{}",
                crate::parser::INDENT.repeat(here),
                written_line(&intent)
            ),
            understood: true,
        });
    }

    out
}

/// An `if` line, with the names in its question resolved.
///
/// # The one line the orb was not writing down
///
/// Every other line goes through §6's matcher at save, which is what turns
/// `grind the sage` into `grind sage`. A control word was kept verbatim — and so
/// was its whole tail, including the place the question is about. `holds`
/// compares that name **exactly**, so `if mortar is empty` never found
/// `mortar_and_pestle`: it answered no on every pass and took the `else` for
/// ever, which reads precisely like an inverted condition and is how it was
/// reported.
///
/// So the question is canonicalised like anything else, and the player sees it:
/// `peruse` reads back `if mortar_and_pestle is empty`, which is the same lesson
/// the echo teaches at the prompt (§6) applied to a file that outlives the
/// moment.
///
/// A name that resolves to nothing is **kept as typed and flagged**, never
/// guessed at. §8 fixes that saving always succeeds, and the runner says which
/// place it could not find every time the question is asked.
fn written_condition(scene: &Scene, line: &str) -> (String, bool) {
    let Some(condition) = crate::parser::condition(crate::parser::spell_argument(line)) else {
        // Unreadable as a question at all. Kept whole, and `spell_unreadable_if`
        // says so when the spell is cast.
        return (line.to_owned(), false);
    };

    // **`Place`, not `Any`.** A question is about somewhere, and letting it
    // reach a reagent would resolve `if sage is empty` to the sage rather than
    // saying there is no such place to ask about.
    let place = named(scene, NounKind::Place, condition.place());
    let (resolved, thing_known) = match &condition {
        Condition::Has { thing, .. } => {
            let found = named(scene, NounKind::Any, thing);
            (
                Condition::Has {
                    place: place
                        .clone()
                        .unwrap_or_else(|| condition.place().to_owned()),
                    thing: found.clone().unwrap_or_else(|| thing.clone()),
                },
                found.is_some(),
            )
        }
        Condition::Is { state, .. } => (
            Condition::Is {
                place: place
                    .clone()
                    .unwrap_or_else(|| condition.place().to_owned()),
                state: *state,
            },
            true,
        ),
    };

    (
        format!("if {}", crate::parser::write_condition(&resolved)),
        place.is_some() && thing_known,
    )
}

/// What `named` is called canonically, if the room has anything by that name.
fn named(scene: &Scene, kind: NounKind, named: &str) -> Option<String> {
    let words: Vec<&str> = named.split_whitespace().collect();
    scene
        .best_match(kind, &words)
        .map(|found| crate::parser::leaf(&found.name).to_owned())
}

/// A line the orb keeps as the player typed it, at the indent its block gives it.
///
/// **Verbatim in text, never in whitespace.** Four branches keep a line's words —
/// a spell naming another spell, an unreadable line, `attend`, and a dropped
/// reagent — and every one of them used to write `line.trim()`, which keeps the
/// words and throws the indentation away. Inside a block that reads as the line
/// falling out of it:
///
/// ```text
/// repeat
///     if mortar_and_pestle is empty
/// grind sage                          ← flagged, so un-indented
///     else
/// ```
///
/// Structurally harmless — blocks are delimited by `repeat`/`if`/`else`/`end`
/// and never by layout — and alarming to look at, which is worse than harmless
/// in a file whose whole job is being read back. It is also the exact thing this
/// module already says about control words: *"the indentation is the orb's, not
/// the player's."* It was true of one branch and not the other four.
fn verbatim(here: usize, line: &str) -> String {
    format!("{}{}", crate::parser::INDENT.repeat(here), line.trim())
}

/// Whether canonicalising `line` would throw away a reagent the player named.
///
/// # The line that got shorter every time you opened it
///
/// §10.1's per-instrument verbs take their reagent **optionally**, so bare
/// `grind` recharges a mortar that is already loaded — deliberate, and the whole
/// reason those verbs beat `move` + `wield`. The consequence nobody traced: with
/// the sage spent, `grind sage` resolves to `grind` with no arguments at all,
/// and the orb wrote `grind` down as though that were what it heard. Reported as
/// *"grind sage is truncated to grind when I close and reopen the editor"* —
/// because `quit` saves, so every visit re-canonicalised the file against
/// whatever happened to be on the shelf at that moment.
///
/// That is not the orb writing down what you meant. `grind` and `grind sage` are
/// different commands, and the line quietly became the other one.
///
/// # Why a reagent, and not any dropped word
///
/// Canonicalisation is *supposed* to discard words — `make a potion of clarity`
/// becomes `recall clarity`, and `look around` becomes `survey`. A sweep for
/// "did any word vanish" flags both of those, and they are the game's flagship
/// plain-English phrasings.
///
/// The distinguishing fact is that a reagent's **name** is a fixed property of
/// the recipes while its **presence** is not: `sage` names something whether or
/// not any is on the shelf, and `around` names nothing ever. So the question is
/// whether a word the orb dropped is in [`Recipes::vocabulary`], and the answer
/// does not move when the laboratory does.
fn dropped_argument(world: &World, line: &str, intent: &Intent) -> bool {
    // Something filled, so nothing was lost — this is `recall clarity`'s case,
    // where `potion` is phrasing and `clarity` is the argument.
    if !intent.arguments.is_empty() {
        return false;
    }
    let known = world.resource::<crate::content::Recipes>().vocabulary();
    line.split_whitespace()
        .skip(1)
        .map(str::to_lowercase)
        .any(|word| known.iter().any(|name| name.eq_ignore_ascii_case(&word)))
}

/// Whether this reading is a spell naming another spell.
///
/// **`Incomplete` counts.** `invoke brew_clarity` where `brew_clarity` does not
/// exist yet leaves the `Script` slot unfillable, which is exactly the case that
/// has to be kept verbatim — refusing to canonicalise only the readings that
/// *did* resolve would miss the ones this exists for.
const fn names_a_spell(resolution: &Resolution) -> bool {
    let verb = match resolution {
        Resolution::Resolved { intent, .. } => intent.verb,
        Resolution::Incomplete { verb, .. } => *verb,
        _ => return false,
    };
    matches!(verb, Verb::Invoke | Verb::Bind)
}

/// One line, as the orb writes it down.
///
/// # There is no `#id` annotation, and §8's example is why
///
/// §8 asks for referents bound *"by stable entity ID, not by name or path"*, so
/// that a destroyed-and-rebuilt thing fails the check and the spell reports
/// `Referent missing` rather than acting on a replacement the enemy swapped in.
/// That is §8.1's substitution surface and it is a real requirement.
///
/// **It is a requirement about things that can be destroyed and rebuilt**, and
/// §8's own example names one: `ward --upon north_gate#7f2a`. A gate is torn
/// down by a siege and put back by the player, so the rebuilt gate is a new
/// entity and the anchor is the only way to tell.
///
/// Nothing in the laboratory is like that. `purge` on a place **scours** it
/// rather than despawning it — *"an instrument is safe by being a place"* — and
/// the tower's four despawns are spent fuel, an instrument's contents, and loose
/// reagents. The mortar's identity is fixed for the life of the world, so
/// `siphon mortar_and_pestle#5` was an anchor against a substitution that cannot
/// happen, written into every spell and read by nothing.
///
/// So the annotation waits for a class of referent that can actually be
/// replaced. When wards and gates arrive with Phase 2's sabotage surfaces, this
/// is where it goes back — and there will be something to test it against, which
/// there is not now.
fn written_line(intent: &Intent) -> String {
    let mut out = String::from(intent.verb.canonical());
    for argument in &intent.arguments {
        out.push(' ');
        out.push_str(argument.display());
    }
    out
}

/// Write `lines` into the spell called `name`, creating it if it is not there.
///
/// Returns the node, so a caller can report on what it just wrote.
pub fn write(world: &mut World, name: &str, lines: &[String]) -> Entity {
    let filename = with_extension(name);
    let existing = find_script(world, &filename);

    // **The spell's own domain, not where the player is standing.** A save that
    // resolved against the player's position would re-home a laboratory spell
    // the moment it was saved from the archive — and the lines would come back
    // flagged unreadable, because `grind` is not a word there.
    let from = existing
        .and_then(|node| world.get::<Domain>(node).cloned())
        .and_then(|Domain(named)| super::navigate::find_place(world, tower::root(world), &named))
        .unwrap_or_else(|| world.resource::<Cwd>().0);

    let written = canonicalise(world, from, lines);
    let text: Vec<String> = written.iter().map(|line| line.text.clone()).collect();
    let domain = tower::domain_of(world, from).unwrap_or(from);
    let domain_name = world
        .get::<Name>(domain)
        .map_or_else(String::new, |name| name.0.clone());

    let node = existing.unwrap_or_else(|| {
        // New spell. It goes in `/grimoire` — the one place a spell lives — so a
        // player who writes one from the laboratory does not leave it on the
        // bench.
        let grimoire = grimoire(world);
        let id = world.resource_mut::<NodeIds>().issue();
        let node = world
            .spawn((
                id,
                Name(filename.clone()),
                Nameable(crate::parser::NounKind::Script),
            ))
            .id();
        world.entity_mut(node).insert(ChildOf(grimoire));
        node
    });

    world
        .entity_mut(node)
        .insert((Held(text.clone()), Domain(domain_name)));

    // **A running spell takes the change now**, not on its next casting.
    //
    // §8 says *"reloads queue to the next tick boundary, so a file cannot change
    // under a script mid-execution"* — and this **is** a tick boundary: a save
    // is queued through `Pending` like every other effect, so the program is
    // swapped between steps and never inside one.
    //
    // The position is kept as it stands. Nothing else is honest: matching old
    // lines to new ones is a diff, and a diff that guesses wrong moves a running
    // spell to a line the player did not point it at. Editing *below* the marker
    // behaves exactly as expected; editing above it shifts what runs next, which
    // is the same thing that happens when you edit a script somebody is reading
    // aloud from.
    if world.get::<crate::tower::spell::Running>(node).is_some() {
        let program = crate::tower::spell::parse(&text);
        if let Some(mut running) = world.get_mut::<crate::tower::spell::Running>(node) {
            running.program = program;
        }
        // Off the end after the swap is how a spell ends anyway — `at` returns
        // nothing and the runner finishes it — so a spell shortened past its own
        // marker stops cleanly rather than needing a case here.
        let message = world
            .resource::<Prose>()
            .line("spell_reloaded", &[("name", &filename)]);
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, Verb::Invoke.canonical())
            .text(FieldName::Path, &filename)
            .text(FieldName::Message, &message)
            .role(Role::Cost)
            .finish();
    }

    say_written(world, &filename, &written);
    node
}

/// The name of the domain the player is standing in, if they are in one.
///
/// `None` at `/tower` and at `/grimoire` — neither is somewhere work happens,
/// which is exactly what makes `scribe` there a question with no answer.
fn domain_here(world: &World) -> Option<String> {
    let cwd = world.resource::<Cwd>().0;
    let domain = tower::domain_of(world, cwd)?;
    world.get::<Name>(domain).map(|name| name.0.clone())
}

/// `/grimoire`, which is where a spell lives.
///
/// Found by walking rather than cached: a `Entity` handle stored anywhere would
/// be the thing `tower::node` warns is meaningless across a save.
fn grimoire(world: &World) -> Entity {
    let root = tower::root(world);
    tower::children_of(world, root)
        .into_iter()
        .find(|node| {
            world
                .get::<Name>(*node)
                .is_some_and(|name| name.0 == "grimoire")
        })
        .unwrap_or(root)
}

/// Report what was written, and how much of it the orb understood.
fn say_written(world: &mut World, filename: &str, written: &[Written]) {
    let unclear = written.iter().filter(|line| !line.understood).count();
    let key = if unclear == 0 {
        "scribe_done"
    } else {
        "scribe_unclear"
    };
    let message = world.resource::<Prose>().line(
        key,
        &[
            ("name", filename),
            ("count", &written.len().to_string()),
            ("detail", &unclear.to_string()),
        ],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Scribe.canonical())
        .text(FieldName::Path, filename)
        .count(FieldName::Quantity, written.len() as u64)
        .text(FieldName::Message, &message)
        .role(if unclear == 0 {
            Role::Success
        } else {
            Role::Cost
        })
        .finish();
}

/// Ask the frontend to open a spell for editing.
///
/// # Why this is a request rather than a buffer
///
/// The editor's buffer lives in the **frontend**, beside the prompt's own line
/// editor. Keystrokes reach no decision, never enter `Submissions` and never
/// enter `ParseLog` — which is exactly the test by which line editing was put
/// there in the first place — and rule 2 keeps layout out of this crate
/// entirely (`tests/boundaries.rs` enforces it by *substring*, so even this
/// sentence may not name the type), which means a buffer here could not know
/// its own pane height or own its viewport.
///
/// What the sim owns is the *decision*: which spell, and what it currently
/// holds. The frontend takes this, edits, and calls `Sim::write_spell` on `:w`.
#[derive(Resource, Debug, Default, Clone)]
pub struct Opening(Option<Request>);

/// A spell the orb has been asked to open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The file, with its extension.
    pub name: String,
    /// Its lines, or empty for a spell being created.
    pub lines: Vec<String>,
    /// Whether it had to be created.
    pub fresh: bool,
    /// Which domain it is written for. Shown in the editor, because a spell that
    /// does not say where it runs is a spell you cannot read.
    pub domain: String,
}

impl Opening {
    /// Ask for `request` to be opened.
    pub fn ask(&mut self, request: Request) {
        self.0 = Some(request);
    }

    /// Take the pending request, if there is one.
    ///
    /// Taking rather than reading: an editor opens once per `scribe`, and a
    /// frontend that polled a persistent flag would reopen it every frame.
    pub const fn take(&mut self) -> Option<Request> {
        self.0.take()
    }

    /// Whether something is waiting to be opened.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.0.is_some()
    }
}

/// `scribe <name>` — open a spell, creating it if it does not exist.
pub(super) fn scribe(intent: &Intent, world: &mut World) {
    let Some(argument) = intent.arguments.first() else {
        super::acknowledge(Verb::Scribe, world);
        return;
    };
    let filename = with_extension(&argument.value);

    let existing = find_script(world, &filename);
    let lines = existing.and_then(|node| world.get::<Held>(node).map(|held| held.0.clone()));
    let fresh = lines.is_none();

    // **A new spell takes the domain you are standing in.** An existing one
    // keeps the one it was written for, so reopening `morning` from the archive
    // edits the laboratory spell it has always been rather than quietly
    // re-homing it.
    let domain = match existing.and_then(|node| world.get::<Domain>(node).cloned()) {
        Some(Domain(named)) => named,
        None => {
            let Some(here) = domain_here(world) else {
                // **The first verb in the game that refuses on *where you are*.**
                // §6 forbids a bare error, so the sentence has to teach: a spell
                // is written for a place, and `/tower` is not one you work in.
                let message = world
                    .resource::<Prose>()
                    .line("scribe_nowhere", &[("name", &filename)]);
                world
                    .resource_mut::<Scrollback>()
                    .records_mut()
                    .push(RecordKind::Completion)
                    .text(FieldName::Name, Verb::Scribe.canonical())
                    .text(FieldName::Path, &filename)
                    .text(FieldName::Message, &message)
                    .role(Role::Danger)
                    .finish();
                return;
            };
            here
        }
    };

    world.resource_mut::<Opening>().ask(Request {
        name: filename.clone(),
        lines: lines.unwrap_or_default(),
        fresh,
        domain: domain.clone(),
    });

    // **A running spell opens, and says it is running.** The program is derived
    // when a spell is *cast*, so saving over it cannot disturb the copy in
    // flight (§8: *"a file cannot change under a script mid-execution"*) — which
    // is safe and, unsaid, indistinguishable from being ignored. The player
    // needs to know the change lands on the next cast.
    let running =
        existing.is_some_and(|node| world.get::<crate::tower::spell::Running>(node).is_some());
    let key = if running {
        "scribe_running"
    } else if fresh {
        "scribe_new"
    } else {
        "scribe_open"
    };
    let message = world
        .resource::<Prose>()
        .line(key, &[("name", &filename), ("source", &domain)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Scribe.canonical())
        .text(FieldName::Path, &filename)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
}

#[cfg(test)]
mod tests {
    use crate::Sim;

    /// The lines `name` holds after the world has had a tick to write them.
    fn written(sim: &mut Sim, name: &str, lines: &[&str]) -> Vec<String> {
        let lines: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
        sim.write_spell(name, &lines);
        sim.step();
        sim.spell(name).expect("the spell was not written")
    }

    #[test]
    fn the_orb_writes_down_what_you_meant() {
        // §8's diegetic beat, and the whole reason canonicalisation happens at
        // authoring time: the file holds what the orb *heard*, so a script that
        // runs a week later is not re-guessing a phrase against a world that has
        // moved.
        let mut sim = Sim::new(1);
        let lines = written(&mut sim, "morning", &["make a potion of clarity"]);
        assert_eq!(lines, ["recall clarity"]);
    }

    #[test]
    fn every_line_resolves_in_the_spells_own_domain() {
        // **The rule a spell is unusable without**, and the one that replaced a
        // walking position. `grind` is a word only where the mortar is (§7), so
        // a spell's lines have to resolve somewhere specific — and that
        // somewhere is the domain the spell was written for, not wherever the
        // player happens to be when they save it.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        let lines = written(&mut sim, "brewing", &["grind the sage", "empty the mortar"]);

        assert_eq!(lines, ["grind sage", "empty mortar_and_pestle"]);
    }

    #[test]
    fn a_saved_spell_keeps_its_domain_wherever_it_is_saved_from() {
        // Reopening `brewing` in the archive and saving must not re-home it —
        // and if it did, every line would come back flagged unreadable, because
        // `grind` is not a word there.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        written(&mut sim, "brewing", &["grind sage"]);

        sim.submit("attend archive");
        sim.step();
        let lines = written(
            &mut sim,
            "brewing",
            &["grind sage", "empty mortar_and_pestle"],
        );
        assert_eq!(
            lines,
            ["grind sage", "empty mortar_and_pestle"],
            "saving from elsewhere re-homed the spell",
        );
    }

    #[test]
    fn nothing_is_written_down_that_the_player_did_not_type() {
        // **The `#id` annotation is gone, and this is what keeps it gone.**
        //
        // §8 binds referents by stable ID so a destroyed-and-rebuilt thing fails
        // the check instead of being acted on — §8.1's substitution surface, and
        // a real requirement about things that *can* be rebuilt. §8's example is
        // `north_gate`, which a siege tears down and the player puts back.
        //
        // Nothing in the tower is like that: `purge` on a place **scours** it
        // rather than despawning it, so an instrument's identity is fixed for
        // the life of the world. `siphon mortar_and_pestle#5` was an anchor
        // against a substitution that cannot happen — written into every spell,
        // read by nothing, and noise in a file the player has to be able to read.
        //
        // It comes back with wards and gates, when there is something for it to
        // catch. Until then a saved line is the line, in the orb's words.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        let lines = written(
            &mut sim,
            "brewing",
            &["empty mortar_and_pestle", "grind sage"],
        );

        assert_eq!(lines, ["empty mortar_and_pestle", "grind sage"]);
        for line in &lines {
            assert!(!line.contains('#'), "an annotation crept back: {line:?}");
        }
    }

    #[test]
    fn attend_in_a_spell_is_flagged_and_kept_rather_than_refused() {
        // A spell does not walk. §8 fixes that saving always succeeds, so the
        // line is kept exactly as typed and marked unreadable — the same
        // treatment a typo gets — rather than the save being refused, which
        // would be the dead end §6 forbids.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        let lines = written(&mut sim, "wander", &["attend archive", "grind sage"]);

        assert_eq!(lines[0], "attend archive", "the line was rewritten");
        assert_eq!(lines[1], "grind sage", "the rest of the spell was affected");
    }

    #[test]
    fn a_spell_cannot_be_written_where_no_work_happens() {
        // The first refusal in the game about **where you are** rather than what
        // you named. §6 forbids a bare error, so the sentence teaches: a player
        // at `/tower` who has not found a domain yet does not know there are any.
        let mut sim = Sim::new(1);
        sim.submit("scribe morning");
        sim.step();

        assert!(sim.opening().is_none(), "the editor opened with no domain");
        let said: Vec<String> = sim
            .scrollback()
            .records()
            .iter()
            .filter_map(|record| record.field(orbs_render::FieldName::Message))
            .filter_map(|value| match value {
                orbs_render::Value::Text(text) => Some(text.to_owned()),
                _ => None,
            })
            .collect();
        assert!(
            said.iter()
                .any(|line| line.contains("go where the work is")),
            "{said:?}",
        );
    }

    #[test]
    fn a_spell_may_name_a_spell_that_does_not_exist_yet() {
        // **§8's own worked example is this case**: `night_watch` invokes
        // `brew_clarity`, and a player writes them in whichever order they think
        // of them. Canonicalising the name would resolve it against whatever
        // Script *is* in scope — and because the parser weights the verb double,
        // a perfect `invoke` with a meaningless argument still clears
        // `MIN_SIMILARITY`, so `invoke brew_clarity` was silently written into
        // the file as `invoke first_light.spell`.
        //
        // Corrupting one spell into a reference to a different one, permanently,
        // with a cheerful "written down" on the way past.
        let mut sim = Sim::new(1);
        let lines = written(&mut sim, "night_watch", &["invoke brew_clarity"]);
        assert_eq!(
            lines,
            ["invoke brew_clarity"],
            "the spell was re-pointed at whatever happened to be nearest",
        );

        // ...and it runs once the spell it names exists.
        let made = written(&mut sim, "brew_clarity", &["survey"]);
        assert_eq!(made, ["survey"]);
        sim.submit("invoke night_watch");
        sim.step_n(4);
        assert!(
            sim.scrollback()
                .records()
                .iter()
                .filter_map(|record| record.field(orbs_render::FieldName::Message))
                .any(|value| matches!(
                    value,
                    orbs_render::Value::Text(text) if text.contains("brew_clarity")
                )),
            "the forward reference never resolved at run time",
        );
    }

    #[test]
    fn a_line_the_orb_cannot_read_is_kept_exactly_and_never_refused() {
        // §8: *"bind always succeeds."* A draft you cannot save is a dead end
        // (§6), and a writer mid-thought saves broken lines constantly.
        let mut sim = Sim::new(1);
        let lines = written(&mut sim, "rough", &["xyzzy plugh", "survey"]);

        assert_eq!(lines[0], "xyzzy plugh", "the unreadable line was mangled");
        assert_eq!(lines[1], "survey");
    }

    #[test]
    fn blank_lines_and_comments_are_the_players_own() {
        let mut sim = Sim::new(1);
        let lines = written(&mut sim, "spaced", &["# the morning round", "", "survey"]);
        assert_eq!(lines, ["# the morning round", "", "survey"]);
    }

    #[test]
    fn scribing_an_existing_spell_opens_it_rather_than_blanking_it() {
        // The commonest `scribe` is reopening one you already have. Handing the
        // editor an empty buffer would silently destroy the spell on the next
        // save, and the player would have no way to tell until it stopped
        // running.
        let mut sim = Sim::new(1);
        sim.submit("scribe first_light");
        sim.step();

        let request = sim.opening().expect("the editor was never asked to open");
        assert_eq!(request.name, "first_light.spell");
        assert!(!request.fresh, "an existing spell reported itself as new");
        assert_eq!(
            request.lines,
            sim.spell("first_light").expect("the shipped spell"),
        );
    }

    #[test]
    fn scribing_a_new_name_opens_an_empty_page_for_the_domain_you_are_in() {
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("scribe morning");
        sim.step();

        let request = sim.opening().expect("the editor was never asked to open");
        assert_eq!(request.name, "morning.spell");
        assert_eq!(request.domain, "laboratory");
        assert!(request.fresh);
        assert!(request.lines.is_empty());
    }

    #[test]
    fn the_editor_is_asked_once_rather_than_every_frame() {
        // `opening` **takes**. A frontend polling a persistent flag would reopen
        // the editor on every frame and the player could never leave it.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("scribe morning");
        sim.step();

        assert!(sim.opening().is_some());
        assert!(sim.opening().is_none(), "the request was not consumed");
    }
}
