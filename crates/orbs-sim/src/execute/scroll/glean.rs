//! What a gleaning supply does: while it is open the stacks gather rather than
//! escape (§19, *Supply*).
//!
//! A maze `research` opens is set with the running rate's spoils and no way
//! out, and one already open is set the same as the supply opens. The draws are
//! the upkeep of that, used or not, so a player opens it for a maze and stops
//! it after.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::spend::{Scroll, say};
use crate::content::{Prose, Scrolls};
use crate::rng::{RngStream, Rngs};
use crate::session::Scrollback;
use crate::tower;
use crate::tower::maze::{Errand, Maze};
use crate::tower::supply::{Scope, Supplies};

/// The spoils a maze opened now is set with, if a gleaning supply is open.
pub(in crate::execute) fn gathering(world: &World) -> Option<usize> {
    let name = Scroll::Gleaning.word();
    let open = world
        .resource::<Supplies>()
        .get(name)
        .filter(|open| open.scope == Scope::Calm)?;
    let worth = world
        .resource::<Scrolls>()
        .get(name)?
        .rates
        .get(open.running)?
        .worth;
    usize::try_from(worth).ok().filter(|spoils| *spoils > 0)
}

/// As the supply opens, set the open maze to gather `spoils`, or say which
/// stacks will.
pub(super) fn set_open(world: &mut World, name: &str, spoils: u32) {
    let open = archive_stacks(world)
        .and_then(|stacks| Some((stacks, world.get::<Maze>(stacks)?.errand())));
    let key = match open {
        // Gathering already: what this opens for is the next maze.
        Some((_, Errand::Glean)) => "scroll_gleaning_gathering",
        Some((stacks, Errand::Way)) => {
            if gather(world, stacks, usize::try_from(spoils).unwrap_or(0)) {
                return;
            }
            "scroll_gleaning_waits"
        }
        None => "scroll_gleaning_waits",
    };
    say(
        world,
        key,
        name,
        &[("count", &spoils.to_string())],
        Role::Success,
    );
}

/// The archive's stacks, wherever the player stands.
///
/// Not `research::stacks`, which reads `Cwd`: a supply opened from the
/// laboratory still finds the maze.
fn archive_stacks(world: &World) -> Option<Entity> {
    let shelf = tower::home(world, "fragment")?;
    super::super::research::stacks_in(world, tower::domain_of(world, shelf)?)
}

/// Set the maze in `stacks` to gather `spoils`, scattered on floor nobody has
/// walked, or as many as it has left. Returns whether it was set.
///
/// Drawn in one go, on the archive's stream, so how the ticks were consumed
/// cannot change it.
pub(in crate::execute) fn gather(world: &mut World, stacks: Entity, spoils: usize) -> bool {
    let Some(mut floor) = world
        .get::<Maze>(stacks)
        .filter(|maze| maze.errand() == Errand::Way)
        .map(Maze::scatterable)
    else {
        return false;
    };
    let spoils = spoils.min(floor.len());
    if spoils == 0 {
        return false;
    }
    let mut drawn = Vec::with_capacity(spoils);
    {
        let mut rngs = world.resource_mut::<Rngs>();
        let rng = rngs.stream(RngStream::Archive);
        for _ in 0..spoils {
            let at = rand::RngExt::random_range(rng, 0..floor.len());
            drawn.push(floor.swap_remove(at));
        }
    }
    // Sorted, so the picture and the log read the same whatever order the draw
    // returned them in.
    drawn.sort_unstable();

    if let Some(mut maze) = world.get_mut::<Maze>(stacks) {
        maze.set_errand(Errand::Glean, drawn);
    }
    // The four ways and the stacks both have something new to say, published
    // on these stacks rather than the room the player stands in.
    super::super::research::publish(world, stacks);

    let name = Scroll::Gleaning.word();
    let message = world
        .resource::<Prose>()
        .line("scroll_gleaning", &[("count", &spoils.to_string())]);
    // No `Detail`: a view draws it *in front of* the message. `Name` and `State`
    // instead, both facts a `sift` can find.
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, name)
        .text(FieldName::State, Errand::Glean.word().unwrap_or_default())
        .text(FieldName::At, "stacks")
        // What puts it in `archive.log` (`files::in_domain`).
        .text(FieldName::Source, "stacks")
        .count(FieldName::Quantity, spoils as u64)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
    true
}
