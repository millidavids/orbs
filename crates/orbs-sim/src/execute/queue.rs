//! `queue` — put a name in the satchel (DESIGN.md §8, [`tower::satchel`]).
//!
//! The push half of §8's channel; the pull half binds a name, so it is the
//! spell word [`SpellWord::Pull`](crate::parser::SpellWord::Pull). A verb, so a
//! player can load a satchel by hand and watch a consumer drain it. No
//! production slot (`spell::block::begins_work`), or a producer could never run
//! beside its consumer.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use crate::parser::{Intent, Verb};
use crate::tower::{self, Cwd, Satchel};

/// Put one name on the back of the satchel where the player is standing.
pub(super) fn queue(intent: &Intent, world: &mut World) {
    // Refused in voice first, `bind`'s shape: it says what would make it work,
    // rather than sending the player looking for a satchel the loom holds.
    if !tower::holds(world, tower::Grant::Satchel) {
        say(world, "queue_unlearned", &[], Role::Cost);
        return;
    }
    let room = world.resource::<Cwd>().0;
    let Some(at) = tower::satchel::fixture(world, room) else {
        say(world, "queue_nowhere", &[], Role::Cost);
        return;
    };

    let Some(named) = intent.arguments.first().map(|one| &one.value) else {
        say(world, "queue_incomplete", &[], Role::Cost);
        return;
    };
    // The leaf, as `limn` and `haul` take, or the consumer gets a full path it
    // cannot read.
    let name = crate::parser::leaf(named).to_owned();

    let Some(mut satchel) = world.get_mut::<Satchel>(at) else {
        say(world, "queue_nowhere", &[], Role::Cost);
        return;
    };
    if !satchel.put(&name) {
        // Full is a refusal naming the depth, never a silent drop that hands
        // the consumer names out of order. See `Satchel::put`.
        let depth = tower::satchel::DEPTH.to_string();
        say(world, "queue_full", &[("quantity", &depth)], Role::Cost);
        return;
    }
    let waiting = satchel.len().to_string();
    say(
        world,
        "queue_put",
        &[("name", &name), ("quantity", &waiting)],
        Role::Success,
    );
}

// `queue` takes a name, never a reading: resolving a reading to what carries it
// was built and withdrawn as a cheat that did the solver's work (§19). The
// satchel is the channel, which `tests/satchel.rs` proves.

fn say(world: &mut World, key: &str, args: &[(&str, &str)], role: Role) {
    super::readings::filed(world, tower::SATCHEL, Verb::Queue, key, args, role);
}
