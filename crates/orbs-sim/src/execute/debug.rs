//! A door for a tester, in the builds a tester runs.
//!
//! # Why this is not a verb
//!
//! `debug_spawn` puts reagents on the shelf so a state that takes forty ticks of
//! grinding can be reached in one line. It is deliberately **not** a
//! [`Verb`](crate::parser::Verb), and every reason is about the vocabulary it
//! would otherwise join:
//!
//! - §6.1's tutorial lists the verbs that *work*, and its most important metric
//!   is the dead-end rate. A word that only exists in some builds would be
//!   counted, offered, and then absent from the one the player has.
//! - §6's naming pass exists so no two verbs collide, and `Verb::ALL` is walked
//!   by the completion table, `recall`, and a test that drives every verb through
//!   a real `Sim`. A variant that comes and goes with a `cfg` makes all of those
//!   differ between builds.
//! - It is not fuzzy-matched, because a tester typing `debug_spaw` should be told
//!   so rather than have the tower quietly change under them.
//!
//! So it is matched **exactly**, before the parser sees the line, in the same
//! shape [`SpellWord`](crate::parser::SpellWord) uses and for the same stated
//! reason: *"checked before the fuzzy matcher ever sees the line"*. The parser's
//! vocabulary does not know it exists.
//!
//! # What a release build has
//!
//! No code. The module is `cfg(debug_assertions)`, so the word, the parse, the
//! queue variant and the effect are all absent — a player who types it gets the
//! ordinary *"nothing here answers to that"*, because as far as that build is
//! concerned nothing does.
//!
//! **Four lines of prose do ship**, and that is not an oversight to fix.
//! `prose.toml` is `include_str!`'d whole, so `debug_spawn_done` and its three
//! neighbours are in the binary as text no code can reach. Moving them out to
//! string literals to save four lines would cost rule 6's uniformity — the width
//! lint, the shouting lint and the CP437 lint all read that file — for something
//! nobody can invoke. `the_word_does_nothing_in_a_release_build` is what makes
//! the distinction hold: the strings are dead weight, and the door is shut.
//!
//! **One consequence worth stating**: a session that used it does not replay in a
//! release build. `Submissions` records the typed line like any other, and a
//! release build reading it back resolves nothing. Debug sessions replay in debug
//! builds, which is where they were recorded.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Fuels, Prose, Recipes};
use crate::session::Scrollback;
use crate::tower::{self, Store};

/// The word, matched exactly and never advertised.
///
/// Underscored on purpose: no verb in §6's vocabulary has an underscore, so this
/// cannot be reached by a typo of a real word, and it reads as a tool rather than
/// as part of the game.
pub const SPAWN: &str = "debug_spawn";

/// What a tester asked for, if this line is a spawn order at all.
///
/// `debug_spawn ground-sage` is one; `debug_spawn ground-sage 5` is five. A count
/// that is not a number is **refused rather than defaulted** — `debug_spawn sage
/// lots` meaning one sage is the kind of quiet reinterpretation this whole
/// subsystem spent a week removing.
#[must_use]
pub fn order(line: &str) -> Option<Order> {
    let rest = line.trim().strip_prefix(SPAWN)?;
    // `debug_spawnage` is not this word. Anything after it must be whitespace.
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let mut words = rest.split_whitespace();
    let Some(name) = words.next() else {
        // Bare, which is how you ask what there is.
        return Some(Order::List);
    };
    let count = match words.next() {
        None => 1,
        // **Zero is not a count, it is a request for nothing.** `stock::give`
        // would spawn a node holding `Counted(0)`, and `stock::take` documents
        // the invariant that breaks: *"a pile that reaches zero is despawned, so
        // *is there any* stays the same question it always was — a node
        // existing."* A nought-node answers `has ash` yes for ever while nothing
        // can ever be taken from it, which is precisely a world state the game
        // cannot otherwise reach — the thing this tool refuses to create.
        Some(count) => count.parse().ok().filter(|count| *count > 0)?,
    };
    // **Every word read, or none of them** — the rule the question grammar next
    // door is built on, and it belongs here for the same reason: `debug_spawn
    // sage 2 3` meaning two sage is a line half-obeyed.
    if words.next().is_some() {
        return None;
    }
    Some(Order::Spawn {
        name: name.to_owned(),
        count,
    })
}

/// What the word was asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    /// Put `count` of `name` on the shelf.
    Spawn {
        /// What to make.
        name: String,
        /// How many.
        count: u32,
    },
    /// Say what can be made.
    List,
}

/// Carry out `order`, on the tick boundary like every other effect.
pub fn run(world: &mut World, order: &Order) {
    match order {
        Order::List => say(
            world,
            "debug_spawn_known",
            &known(world).join(", "),
            Role::Normal,
        ),
        Order::Spawn { name, count } => spawn(world, name, *count),
    }
}

/// Put `count` of `name` on the shelf where the player is standing.
fn spawn(world: &mut World, name: &str, count: u32) {
    // **Known names only.** Spawning `xyzzy` would put a node in the tower that
    // no recipe, no instrument and no `survey` row knows what to do with — a
    // world state the game cannot otherwise reach, which is the opposite of what
    // a testing tool is for.
    let Some(known) = known(world)
        .into_iter()
        .find(|word| word.eq_ignore_ascii_case(name))
    else {
        say(world, "debug_spawn_unknown", name, Role::Danger);
        return;
    };

    // The dispensary, not the floor and not wherever the player happens to be:
    // §19 fixed that there is one place things go when they leave a tool, and
    // `move` can only reach a fixture.
    let Some(shelf) = dispensary(world) else {
        say(world, "debug_spawn_nowhere", &known, Role::Danger);
        return;
    };
    // **The kind the laboratory would have given it.** A potion is an `Essence`
    // and everything else is crafting stock (`work::produce`), so spawning
    // `clarity` as a reagent would put a node in the tower with the right name
    // and the wrong kind — a thing no `distil` could have made and no slot will
    // take. That is precisely the state this refuses to create, arriving through
    // the check meant to prevent it.
    let kind = world.resource::<Recipes>().kind_of(&known);
    tower::give(world, shelf, &known, kind, count);
    say(world, "debug_spawn_done", &known, Role::Success);
}

/// The tower's shelf, wherever the player is standing.
///
/// **Walked from the root rather than read out of the current room.** A tester
/// setting up a state should not have to be standing in the right place first —
/// and the point of the tool is to skip the forty ticks of walking and grinding
/// that reaching the state would otherwise cost, which a `cwd` check would put
/// straight back.
///
/// `None` only if the tower has no `Store` at all, which `build::raise` always
/// makes. It reports rather than panicking: a missing dispensary is a build bug,
/// and a sentence naming it is more use to whoever caused it than a stack trace
/// from a debug command.
fn dispensary(world: &World) -> Option<Entity> {
    let mut stack = vec![tower::root(world)];
    while let Some(node) = stack.pop() {
        if world.get::<Store>(node).is_some() {
            return Some(node);
        }
        stack.extend(tower::children_of(world, node));
    }
    None
}

/// Every name that can be spawned: everything the recipes name, and everything
/// that burns.
///
/// The union rather than either half. `Recipes::vocabulary` is *"every name any
/// recipe can produce or consume"* and misses fuel, because the athanor
/// transforms nothing and so has no recipe — which is exactly the reagent a
/// tester reaches for first.
fn known(world: &World) -> Vec<String> {
    let mut names: Vec<String> = world
        .resource::<Recipes>()
        .vocabulary()
        .into_iter()
        .map(str::to_owned)
        .collect();
    names.extend(world.resource::<Fuels>().names().map(str::to_owned));
    names.sort_unstable();
    names.dedup();
    names
}

/// Say what happened. §3 forbids unlogged output, and a debug tool is not
/// exempt — a tester who cannot see that the spawn failed is a tester chasing
/// the wrong bug.
fn say(world: &mut World, key: &str, detail: &str, role: Role) {
    let message = world
        .resource::<Prose>()
        .line(key, &[("name", detail), ("detail", detail)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, SPAWN)
        .text(FieldName::Message, &message)
        .role(role)
        .finish();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_word_is_matched_exactly() {
        assert_eq!(order("debug_spawn"), Some(Order::List));
        assert_eq!(
            order("debug_spawn sage"),
            Some(Order::Spawn {
                name: "sage".to_owned(),
                count: 1,
            }),
        );
        assert_eq!(
            order("  debug_spawn ground-sage 5  "),
            Some(Order::Spawn {
                name: "ground-sage".to_owned(),
                count: 5,
            }),
        );

        // Not this word, and not guessed at: a tester who mistypes it should be
        // told by the ordinary parser rather than have the tower change.
        for other in [
            "debug_spawnage",
            "debug spawn sage",
            "spawn sage",
            "debug_spaw sage",
            "grind sage",
        ] {
            assert_eq!(order(other), None, "{other:?} was read as a spawn");
        }

        // A count that is not a count. Defaulting to one would be the quiet
        // reinterpretation this subsystem exists to refuse.
        assert_eq!(order("debug_spawn sage lots"), None);
        assert_eq!(order("debug_spawn sage 2 3"), None);
    }
}
