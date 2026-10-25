//! Every puzzle domain's per-puzzle answers, matched exhaustively over
//! [`Open`]: what it publishes, how a stopped one is let go, and a load's one
//! tidy — a puzzle the document carried and the load refused.
//!
//! A puzzle travels as a component and as its published readings, so a load
//! that refuses the component must tidy the readings. Exhaustive, so a new
//! `Open` is a compile error here until it says what it publishes. A whole
//! save is left as saved, since republishing issues new ids.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use super::readings::{reading, room_of};
use crate::parser::Verb;
use crate::tower::{self, circle::Glyph, maze, puzzle::Open};

/// Which of a fixture's own children belong to its puzzle.
enum OnFixture {
    /// Every one: its publisher clears the fixture wholesale.
    Any,
    /// Only these. The pylon's `integrity` is about the tower rather than a
    /// course, and the stacks keep the archive's fragments beside the errand.
    Named(&'static [&'static str]),
}

/// What a fixture says only while its puzzle is open.
struct Says {
    on_fixture: OnFixture,
    /// The room's `Role::Reading` nodes the puzzle publishes into.
    in_room: Vec<&'static str>,
}

fn says(open: Open) -> Says {
    match open {
        Open::Maze => Says {
            on_fixture: OnFixture::Named(&maze::Errand::ALL),
            in_room: maze::Way::ALL.map(maze::Way::word).to_vec(),
        },
        Open::Ward => Says {
            on_fixture: OnFixture::Any,
            in_room: tower::ward::SOCKETS.to_vec(),
        },
        Open::Binding => Says {
            on_fixture: OnFixture::Any,
            in_room: tower::lattice::COLUMNS.to_vec(),
        },
        Open::Course => Says {
            on_fixture: OnFixture::Named(&[tower::pylon::ODD]),
            in_room: tower::pylon::STATIONS.to_vec(),
        },
        Open::Beast => Says {
            on_fixture: OnFixture::Any,
            in_room: Glyph::ALL.map(Glyph::word).to_vec(),
        },
    }
}

/// Republish a fixture from its component, whoever is standing where.
fn publish(world: &mut World, open: Open, fixture: Entity) {
    match open {
        Open::Maze => super::research::publish(world, fixture),
        Open::Ward => super::scry::publish(world, fixture),
        Open::Binding => super::imbue::publish(world, fixture),
        Open::Course => super::muster::publish(world, fixture),
        Open::Beast => super::summon::publish(world, fixture),
    }
}

/// Republish `open`'s fixture where the player is standing.
pub(crate) fn refresh(world: &mut World, open: Open) {
    if let Some(fixture) = super::readings::fixture(world, open.operation()) {
        publish(world, open, fixture);
    }
}

/// Let the puzzle on `at` go, for `stop`, and say whether anything went.
pub(super) fn abandon(world: &mut World, open: Open, at: Entity) -> bool {
    match open {
        Open::Maze => {
            world.entity_mut(at).remove::<tower::Maze>();
            refresh(world, open);
            abandoned(world, at, "research_abandoned");
            true
        }
        // A course the same, because `muster_already` promises it: *"haul it
        // across, or stop the pylon"* is §6's way forward.
        Open::Course => {
            world.entity_mut(at).remove::<tower::Course>();
            refresh(world, open);
            abandoned(world, at, "muster_abandoned");
            true
        }
        // Without this `stop circle` said the circle was not working while `if
        // the circle is working` said it was.
        Open::Beast => super::summon::release(world, at),
        // A gap, not a decision: what letting an open ward or a part-bound charm
        // go costs is open in §19.
        Open::Ward | Open::Binding => false,
    }
}

/// Say that `stop` let the puzzle on `at` go, filed to its domain's log.
fn abandoned(world: &mut World, at: Entity, key: &str) {
    let source = world
        .get::<tower::Name>(at)
        .map(|name| name.0.clone())
        .unwrap_or_default();
    super::readings::filed(world, &source, Verb::Stop, key, &[], Role::Cost);
}

/// Clear what every puzzle fixture with no puzzle open still says about one.
pub(crate) fn settle(world: &mut World) {
    let looking: &World = world;
    let stale: Vec<(Open, Entity)> = Open::ALL
        .into_iter()
        .flat_map(|open| {
            fixtures(looking, open)
                .into_iter()
                .filter(move |fixture| {
                    Open::on(looking, *fixture).is_none() && says_anything(looking, open, *fixture)
                })
                .map(move |fixture| (open, fixture))
        })
        .collect();
    for (open, fixture) in stale {
        publish(world, open, fixture);
    }
}

/// Every fixture in the tower carrying `open`'s operation.
fn fixtures(world: &World, open: Open) -> Vec<Entity> {
    world
        .iter_entities()
        .filter(|entity| {
            entity
                .get::<tower::Operation>()
                .is_some_and(|operation| operation.0 == open.operation())
        })
        .map(|entity| entity.id())
        .collect()
}

/// Whether a fixture still says anything its puzzle would.
fn says_anything(world: &World, open: Open, fixture: Entity) -> bool {
    let says = says(open);
    let on_fixture =
        tower::children_of(world, fixture)
            .into_iter()
            .any(|child| match says.on_fixture {
                OnFixture::Any => true,
                OnFixture::Named(names) => world
                    .get::<tower::Name>(child)
                    .is_some_and(|name| names.contains(&name.0.as_str())),
            });
    on_fixture
        || room_of(world, fixture).is_some_and(|room| {
            says.in_room
                .iter()
                .filter_map(|name| reading(world, room, name))
                .any(|node| !tower::children_of(world, node).is_empty())
        })
}
