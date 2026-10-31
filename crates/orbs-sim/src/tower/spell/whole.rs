//! The tower's spell: one spell homed at `/tower`, which watches every room and
//! starts and stops the rooms' spells (§19, *The tower's spell*).
//!
//! What it may say is narrow on purpose (`speech`). Every way of asking whether
//! something is the tower's is here, one per thing it is asked of.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tower::{self, Brief, Cwd, Domain, Name, NodeId, TOWER};

/// The concentration a tower spell is written from: one for itself, and one
/// for a room's spell to start.
pub const TOWER_CONCENTRATION: usize = 2;

/// `/tower`, the tower spell's home.
fn tower_node(world: &World) -> Option<Entity> {
    tower::find_by_path(world, "/tower")
}

/// Whether the node `at` is the tower itself.
pub(crate) fn at_the_tower(world: &World, at: Entity) -> bool {
    tower_node(world) == Some(at)
}

/// Whether a run homed at `at` is homed at the tower itself.
pub(crate) fn is_tower(world: &World, at: NodeId) -> bool {
    tower_node(world).and_then(|node| world.get::<NodeId>(node).copied()) == Some(at)
}

/// Whether `domain` is the name a spell for the tower is written under.
pub(crate) fn names_tower(domain: &str) -> bool {
    domain == TOWER
}

/// Whether `cwd` is in the tower: the tower, or anywhere in a room. Not the
/// grimoire, its sibling.
pub(crate) fn inside(world: &World, cwd: Entity) -> bool {
    at_the_tower(world, cwd) || tower::domain_of(world, cwd).is_some()
}

/// Whether `spell` is written for the tower.
pub(crate) fn for_the_tower(world: &World, spell: Entity) -> bool {
    world
        .get::<Domain>(spell)
        .is_some_and(|domain| names_tower(&domain.0))
}

/// Whether a tower spell may be written where the player stands: at `/tower`,
/// with the concentration to hold it and one thing it starts.
pub(crate) fn may_write(world: &World) -> bool {
    at_the_tower(world, world.resource::<Cwd>().0)
        && tower::concentration(world) >= TOWER_CONCENTRATION
}

/// Another tower spell running or held, if there is one: one runs the tower at
/// a time. The first by [`NodeId`], since a refusal names it.
pub(crate) fn running_elsewhere(world: &World, spell: Entity) -> Option<String> {
    let mut busy: Vec<(NodeId, String)> = tower_spells(world)
        .into_iter()
        .filter(|(node, _, _)| *node != spell)
        .filter(|(node, _, _)| {
            world.get::<super::Running>(*node).is_some()
                || world.get::<super::Bound>(*node).is_some()
        })
        .map(|(_, id, name)| (id, name))
        .collect();
    busy.sort_unstable_by_key(|(id, _)| *id);
    busy.into_iter().next().map(|(_, name)| name)
}

/// Refuse to start the tower spell `wanted` while another runs the tower,
/// naming it. Whether it refused; `verb` is the door the player came by.
pub(super) fn refused_while_taken(
    world: &mut World,
    spell: Entity,
    wanted: &str,
    verb: Verb,
) -> bool {
    if !for_the_tower(world, spell) {
        return false;
    }
    let Some(other) = running_elsewhere(world, spell) else {
        return false;
    };
    let message = world
        .resource::<Prose>()
        .line("spell_tower_taken", &[("name", wanted), ("detail", &other)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Path, wanted)
        .text(FieldName::Message, &message)
        .role(Role::Danger)
        .finish();
    true
}

/// The tower's spell at a glance, while one runs or is held, or its fault
/// stands: what the rail's foot and the tests read, since [`tower::briefs`]
/// walks only the rooms.
#[must_use]
pub fn brief(world: &World) -> Option<Brief> {
    let casts = tower::running_spells(world);
    let running: Vec<&tower::Cast> = casts
        .iter()
        .filter(|cast| names_tower(&cast.domain))
        .collect();
    // Between laps a held spell is not running, and the rail must not blink.
    let held = tower_spells(world)
        .into_iter()
        .find(|(node, _, _)| world.get::<super::Bound>(*node).is_some())
        .map(|(_, _, name)| name.trim_end_matches(".spell").to_owned());
    let spell = running.first().map(|cast| cast.spell.clone()).or(held);
    let mark = world
        .resource::<tower::Marks>()
        .iter()
        .find(|(domain, _)| names_tower(domain))
        .map(|(_, mark)| mark);
    if spell.is_none() && mark.is_none() {
        return None;
    }
    let wait = tower::wait_in(&[], TOWER, &casts);
    // What a reader hears for the rail's `►` row: the hold, when it is held,
    // as `status rates` says it.
    let prose = world.resource::<Prose>();
    let spoken = match (&wait, &spell) {
        (tower::Wait::Held { .. }, _) => Some(tower::wait_sentence(prose, &wait)),
        (_, Some(name)) => Some(prose.line("rates_tower", &[("name", name)])),
        (_, None) => None,
    };
    Some(Brief {
        name: TOWER,
        built: true,
        state: tower::State::Empty,
        wait,
        detail: None,
        detail_spoken: spoken,
        spell,
        running: running.iter().map(|cast| cast.cursors).sum(),
        mark,
        mastery: None,
    })
}

/// Every spell written for the tower: its node, its id, and its name.
fn tower_spells(world: &World) -> Vec<(Entity, NodeId, String)> {
    let Some(mut query) = world.try_query::<(Entity, &NodeId, &Name, &Domain)>() else {
        return Vec::new();
    };
    query
        .iter(world)
        .filter(|(_, _, _, domain)| names_tower(&domain.0))
        .map(|(node, id, name, _)| (node, *id, name.0.clone()))
        .collect()
}
