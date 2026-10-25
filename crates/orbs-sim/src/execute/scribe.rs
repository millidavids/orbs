//! Writing a spell down — and, deliberately, doing nothing else to it.
//!
//! `Held` holds exactly what was typed; the reading happens at cast
//! ([`spell::compile`](crate::tower::spell::compile)), where names are fixed in
//! the world the spell runs in (§19). §8's *the orb writes down what you meant*
//! is the editor's `interpret`, where a wrong reading shows before it runs.

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
/// A save says nothing: the buffer saves after every pause, and the runner and
/// editor already name an unreadable line.
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

    // `lines` exactly as given; `read` beside it is what `compile` compiles,
    // equal to `lines` unless something read them. `by` is whose reading it
    // is, asked by the next write.
    world.entity_mut(node).insert((
        Held(lines.to_vec()),
        tower::Read::new(lines, read.to_vec(), Some(by)),
        Domain(domain_name),
    ));

    // A running spell takes the change now, between steps, since a save is
    // queued through `Pending` (§8). The position is kept: matching old lines
    // to new is a diff, and a wrong guess moves the spell.
    if world.get::<crate::tower::spell::Running>(node).is_some() {
        // Through `compile`, like every program, from `read`, so the reload
        // agrees with the next cast. Its complaints are `invoke`'s and the
        // editor's to say, not every save's.
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
        // A spell shortened past its marker just finishes. Said once per
        // session — see [`Reloaded`].
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
/// Once per session, since the buffer saves after every pause. Reset by
/// [`scribe`], so it derives from the submission stream and replays alike.
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
/// A request, not a buffer: keystrokes reach no decision, and layout stays out
/// of this crate (rule 2). The sim owns which spell and what it holds; the
/// frontend edits and calls `Sim::write_spell` on `:w`.
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
    /// Taking, not reading, so the editor opens once per `scribe`.
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

    // A running spell opens and says so: saving is safe (§8), and unsaid would
    // look like being ignored.
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
