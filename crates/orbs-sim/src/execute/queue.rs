//! `queue` — put a name in the satchel (DESIGN.md §8, [`tower::satchel`]).
//!
//! The push half of §8's channel between two spells. The pull half is `pull`,
//! and it lives in the spell language rather than here: pulling **binds a
//! name**, and `let` is the only shape in the language that does one of those.
//! A verb cannot bind, so the pair is asymmetric by construction rather than by
//! preference — see [`SpellWord::Pull`](crate::parser::SpellWord::Pull).
//!
//! What this being a verb buys is the thing the mechanic would otherwise have no
//! way to show: a player can load a satchel by hand at the prompt and watch a
//! consumer spell drain it, one name a tick, with nothing else running.
//!
//! # It takes no production slot
//!
//! The lens's argument, and the menagerie's. A producer that has to hold the
//! tower's one slot could never run beside the consumer it feeds, which is the
//! entire point of the channel. `spell::block::begins_work` carries the
//! exemption.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::{Intent, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Cwd, Satchel};

/// Put one name on the back of the satchel where the player is standing.
pub(super) fn queue(intent: &Intent, world: &mut World) {
    // **Refused in voice before anything else is asked**, which is `bind`'s
    // shape: the word works, and it says what would make it work. A gate that
    // reported *"there is no satchel here"* would send the player looking round
    // the room for a thing the loom holds.
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
    // **The leaf, for the reason `limn` and `haul` both take one.** A resolved
    // place arrives as its full path, so a satchel loaded with
    // `/tower/menagerie/heed` would hand the consumer a word `limn` cannot read
    // — and the echo would print the tower's internals at a player.
    let name = crate::parser::leaf(named).to_owned();

    let Some(mut satchel) = world.get_mut::<Satchel>(at) else {
        say(world, "queue_nowhere", &[], Role::Cost);
        return;
    };
    if !satchel.put(&name) {
        // **Full is a refusal that names the depth**, not a silent drop. A queue
        // that forgot its front to make room would hand the consumer a name out
        // of order, which is indistinguishable from a solver whose timing is
        // wrong — and the pipeline would go on looking healthy while producing
        // rubbish. See `Satchel::put`.
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

// # `queue` takes a name and never a reading, and that was built and withdrawn
//
// **The tempting version resolves a reading to whatever carries it** — `queue
// onward`, while the menagerie was a chant, meaning *"put whatever is one behind
// the rule into the satchel"*. It was written, it worked, and it was a cheat:
// resolving the reading moved the identification into the world, and the pair
// of spells then did at one step a tick what the solver was meant not to. That
// is `bide until` in a new hat — the world answering the question the spell is
// supposed to — and it is refused for any reading, in any room.
//
// What the satchel is for is the channel — a name one spell puts down and
// another picks up — which nothing in §8 could do before and which
// `tests/satchel.rs` proves on its own terms.

fn say(world: &mut World, key: &str, args: &[(&str, &str)], role: Role) {
    let message = world.resource::<Prose>().line(key, args);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Queue.canonical())
        .text(FieldName::Message, &message)
        .text(FieldName::Source, tower::SATCHEL)
        .role(role)
        .finish();
}
