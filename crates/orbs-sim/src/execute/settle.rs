//! A load's one tidy: a puzzle the document carried and the load refused.
//!
//! Every puzzle travels twice — as a component on its fixture, and as the
//! readings it published, which are nodes of their own. A restore that refuses
//! the component brings the readings back without it: `survey keystone` names a
//! humour with nothing waiting, and `if the circle is empty` is false.
//!
//! One pass over [`Open::ALL`], matched exhaustively, which is `puzzle::Open`'s
//! own rule. The circle and the pylon each grew a `settle` of their own, and a
//! third puzzle would have needed a third nobody was reminded to write. A
//! variant added to `Open` is a compile error here until it says what its
//! readings are and what publishes them.
//!
//! A whole save is left exactly as saved. Republishing issues new ids, so a
//! fixture is touched only when it has no puzzle and still says something about
//! one, which no whole save can be.

use bevy_ecs::prelude::*;

use super::readings::{reading, room_of};
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
