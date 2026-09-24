//! Writing a spell down — and, deliberately, doing nothing else to it.
//!
//! The rewriter that canonicalised each line as it landed is gone: it worked on
//! whole lines and wrote the rest out of existence — `if a has x or b has x`
//! saved as `if a has x`, one of four such bugs in §19. `Held` holds exactly
//! what was typed, and the reading happens at cast, where the program is
//! already *"derived, never stored"*
//! ([`spell::compile`](crate::tower::spell::compile)). §8's requirement is met
//! at a better moment: names are fixed in the world the spell is about to run
//! in, and one that stops matching is reported rather than guessed at.
//!
//! §8's *the orb writes down what you meant* moved to `interpret` in the
//! editor, where the player asks for it and can see a wrong reading before it
//! runs.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Prose, with_extension};
use crate::parser::{Intent, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Cwd, Domain, Held, Name, Nameable, NodeIds};

use super::navigate::find_script;

/// Write `lines` into the spell called `name`, creating it if it is not there.
///
/// Returns the node, so a caller can report on what it just wrote.
///
/// A save says nothing. The buffer writes itself out after every pause in the
/// typing, so the old *"{name}: {count} lines, written down"* filled one
/// editing session's transcript with sixteen copies of itself. Nothing was lost
/// with it: the runner names an unreadable line individually, with its number,
/// when the spell is cast (`spell_missing`, `spell_unreadable_if`,
/// `spell_forbidden`), and the editor marks it as it is written.
pub fn write(world: &mut World, name: &str, lines: &[String], read: &[String], by: u64) -> Entity {
    let filename = with_extension(name);
    let existing = find_script(world, &filename);

    // The spell's own domain, not where the player is standing: the domain is
    // what its lines are read against at cast, so re-homing a laboratory spell
    // saved from the archive would make `grind` stop being a word.
    let from = existing
        .and_then(|node| world.get::<Domain>(node).cloned())
        .and_then(|Domain(named)| super::navigate::find_place(world, tower::root(world), &named))
        .unwrap_or_else(|| world.resource::<Cwd>().0);
    let domain = tower::domain_of(world, from).unwrap_or(from);
    let domain_name = world
        .get::<Name>(domain)
        .map_or_else(String::new, |name| name.0.clone());

    let node = existing.unwrap_or_else(|| {
        // New spell. It goes in `/grimoire` — the one place a spell lives — so
        // one written from the laboratory is not left on the bench.
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

    // `lines`, exactly as given — the line the whole rewrite was for. `read`
    // lands beside it and is what `compile` compiles; it equals `lines`
    // wherever nothing read them, so the default is the identity case and the
    // game is what it was. `by` is whose reading it is, which the next write
    // asks before keeping any of it.
    world.entity_mut(node).insert((
        Held(lines.to_vec()),
        tower::Read::new(lines, read.to_vec(), Some(by)),
        Domain(domain_name),
    ));

    // A running spell takes the change now, not on its next casting. §8's
    // *"reloads queue to the next tick boundary, so a file cannot change under
    // a script mid-execution"* holds: a save is queued through `Pending` like
    // every other effect, so the program swaps between steps, never inside one.
    //
    // The position is kept as it stands, because matching old lines to new ones
    // is a diff and a wrong guess moves a running spell to a line the player
    // did not point it at.
    if world.get::<crate::tower::spell::Running>(node).is_some() {
        // Through `compile`, like every other way a program is made: a swap
        // that skipped name resolution would leave a running spell asking about
        // words the tower does not use. Its complaints are not said here —
        // `invoke` says them once at cast, and a save happens after every pause
        // in the typing. Nothing is hidden by that: the editor marks the line
        // and counts it on the status row (`Sim::read_spell`), and the runner
        // still names a missing place every casting, because `said` is cleared
        // just below.
        //
        // `read`, not `lines` — the reading is what a program is built from,
        // and this reload must agree with the cast that will follow it.
        let program = crate::tower::spell::compile(world, from, read);
        if let Some(mut running) = world.get_mut::<crate::tower::spell::Running>(node) {
            running.program = program;
            // The new text is a new set of lines, so what was reported about the
            // old one is not what needs saying about this one.
            running.said.clear();
        }
        // And the held copy, which outlives the run between laps: the rationing
        // that survives a lap would survive a fixed typo too, and the bound
        // spell would never say so again.
        if let Some(mut bound) = world.get_mut::<crate::tower::spell::Bound>(node) {
            bound.said.clear();
        }
        // A spell shortened past its own marker needs no case here: `at`
        // returns nothing and the runner finishes it, which is how a spell ends
        // anyway.
        //
        // Said once per session, not once per save — see [`Reloaded`]. The swap
        // above happens every time; only the sentence is rationed.
        if world.resource_mut::<Reloaded>().first() {
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
    }

    node
}

/// Whether this editing session has already been told the orb took a change up.
///
/// Once per session, not once per save. `spell_reloaded` is worth saying —
/// picking a change up between steps is safe, invisible and otherwise
/// indistinguishable from being ignored — but the buffer saves after every
/// pause in the typing, so per-save is the same sentence every few seconds.
///
/// Reset by [`scribe`], the only way an editing session begins, so it is
/// derived from the submission stream alone and a replay says the same things
/// in the same order.
#[derive(Resource, Debug, Default)]
pub(crate) struct Reloaded(bool);

impl Reloaded {
    /// Begin a session with the notice unsaid.
    const fn begin(&mut self) {
        self.0 = false;
    }

    /// Whether this is the reload to speak, marking it spoken.
    const fn first(&mut self) -> bool {
        !std::mem::replace(&mut self.0, true)
    }
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

/// Ask the frontend to open a spell for editing.
///
/// A request rather than a buffer: the editor's buffer lives in the frontend,
/// beside the prompt's own line editor, because its keystrokes reach no
/// decision and enter neither `Submissions` nor `ParseLog`. Rule 2 also keeps
/// layout out of this crate (`tests/boundaries.rs` enforces it by *substring*,
/// so even this sentence may not name the type), so a buffer here could not
/// know its own pane height or own its viewport.
///
/// The sim owns the *decision*: which spell, and what it currently holds. The
/// frontend takes this, edits, and calls `Sim::write_spell` on `:w`.
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

    // A new spell takes the domain you are standing in; an existing one keeps
    // the one it was written for, so reopening `morning` from the archive edits
    // the laboratory spell rather than quietly re-homing it.
    let domain = match existing.and_then(|node| world.get::<Domain>(node).cloned()) {
        Some(Domain(named)) => named,
        None => {
            let Some(here) = domain_here(world) else {
                // The first verb to refuse on *where you are*. §6 forbids a
                // bare error, so the sentence teaches: a spell is written for a
                // place, and `/tower` is not one you work in.
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
    // A fresh session, so the reload notice below is worth saying once more.
    world.resource_mut::<Reloaded>().begin();

    // A running spell opens and says so. The program is derived at cast, so
    // saving over it cannot disturb the copy in flight (§8: *"a file cannot
    // change under a script mid-execution"*) — safe, and unsaid
    // indistinguishable from being ignored.
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
    fn what_is_typed_is_what_is_kept() {
        // Loose phrasing, a comment, a blank line, hand indentation that
        // disagrees with the block depth, an unreadable line and a forward
        // reference — all of it comes back exactly as it went in.
        let typed = [
            "# the morning round",
            "",
            "make a potion of clarity",
            "        grind the sage   ",
            "if the mortar is idle and the dispensary has sage",
            "xyzzy plugh",
            "invoke brew_clarity",
            "end",
        ];
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();

        assert_eq!(written(&mut sim, "kept", &typed), typed);
    }

    #[test]
    fn saving_the_same_buffer_again_changes_nothing() {
        // The reported complaint, as a property: `quit` saves, so a spell was
        // re-read against whatever was on the shelf each time it was opened,
        // and lines got shorter.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        let typed = [
            "grind the sage",
            "if the mortar is idle",
            "grind sage",
            "end",
        ];

        let first = written(&mut sim, "steady", &typed);
        for _ in 0..5 {
            let again = written(&mut sim, "steady", &typed);
            assert_eq!(again, first, "the file moved under a save");
        }
        assert_eq!(first, typed);
    }

    #[test]
    fn a_reagent_the_shelf_has_run_out_of_is_still_written_down() {
        // §19's *"`grind sage` is truncated to `grind` when I close and reopen
        // the editor"*: §10.1's verbs take their reagent optionally, so with the
        // sage spent it resolved to bare `grind` and the orb wrote that down.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("grind sage");
        sim.step_n(20);

        assert_eq!(written(&mut sim, "keep", &["grind sage"]), ["grind sage"]);
    }

    #[test]
    fn a_saved_spell_keeps_its_domain_wherever_it_is_saved_from() {
        // Asserted on the domain itself, not on the text: lines now come back
        // unmangled whatever the domain says, so the old test would have passed
        // through the bug it was written for.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        written(&mut sim, "brewing", &["grind sage"]);

        sim.submit("attend archive");
        sim.step();
        written(
            &mut sim,
            "brewing",
            &["grind sage", "empty mortar_and_pestle"],
        );

        assert_eq!(
            sim.spell_domain("brewing").as_deref(),
            Some("laboratory"),
            "saving from elsewhere re-homed the spell",
        );
    }

    #[test]
    fn a_spell_cannot_be_written_where_no_work_happens() {
        // The first refusal about where you are rather than what you named. §6
        // forbids a bare error: a player at `/tower` who has not found a domain
        // does not know there are any.
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
    fn scribing_an_existing_spell_opens_it_rather_than_blanking_it() {
        // The commonest `scribe` reopens one you already have. An empty buffer
        // would destroy the spell on the next save, unnoticed until it stopped
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
        // `opening` takes: a frontend polling a persistent flag would reopen the
        // editor every frame and the player could never leave it.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("scribe morning");
        sim.step();

        assert!(sim.opening().is_some());
        assert!(sim.opening().is_none(), "the request was not consumed");
    }
}
