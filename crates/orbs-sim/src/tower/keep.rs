//! The arsenal: where finished work is kept, and the one room you can reach from
//! any other.
//!
//! §7 scopes naming to where you are standing, which left no route for a thing
//! made in one domain and spent in another. The exemption is narrow: what
//! reaches everywhere is the arsenal's *contents*, and it refuses stock at the
//! door, so acting on a domain you are not in is still Phase 10's unlock.
//!
//! Nameable is not enough — every lookup that can name what is in here has to
//! reach it, or a word resolves at full confidence and reports "no such thing".
//! That is [`kept`], with three callers: `pipeline::reachable`,
//! `pipeline::purge` and `files::here_or_place`.

use bevy_ecs::prelude::*;

use super::node::{Keep, Nameable, children_of, root};
use crate::parser::NounKind;

/// The arsenal's name, for the one place `build` needs to spell it.
pub const ARSENAL: &str = "arsenal";

/// The tower's arsenal, if it has one.
///
/// Walked from the root rather than read out of `Cwd`: this is the one place
/// whose contents do not depend on where the player is standing.
#[must_use]
pub fn keep(world: &World) -> Option<Entity> {
    let mut stack = vec![root(world)];
    while let Some(node) = stack.pop() {
        if world.get::<Keep>(node).is_some() {
            return Some(node);
        }
        stack.extend(children_of(world, node));
    }
    None
}

/// Everything the arsenal holds, wherever the player is.
///
/// In one place so the three lookups cannot disagree — hand-built copies of
/// *what is in this instrument* all dropped the stock count (§19).
#[must_use]
pub fn keeping(world: &World) -> Vec<Entity> {
    keep(world).map_or_else(Vec::new, |keep| children_of(world, keep))
}

/// What the arsenal holds under `named`, if anything.
#[must_use]
pub fn kept(world: &World, named: &str) -> Option<Entity> {
    // By leaf: an argument may arrive as a path (§7) and a node carries only its
    // last segment. See `tower::reach` for the axis this is one setting of.
    super::reach::look(world)
        .scope(super::reach::Scope::Arsenal)
        .find(crate::parser::leaf(named))
}

/// Whether the arsenal will take `node`.
///
/// Finished work only: an [`Essence`](NounKind::Essence) or a
/// [`Scroll`](NounKind::Scroll). A reagent, a fragment, a file or a vessel is
/// refused at the door and stays where it is.
///
/// Asked of the kind, never of the name — telling finished work from stock by
/// name would mean the tower deciding which reagents are waste, which §10.1
/// refuses.
#[must_use]
pub fn admits(world: &World, node: Entity) -> bool {
    matches!(
        world.get::<Nameable>(node).map(|kind| kind.0),
        Some(NounKind::Essence | NounKind::Scroll)
    )
}
