//! The arsenal: where finished work is kept, and the one room you can reach from
//! any other.
//!
//! # Why the tower needs one at all
//!
//! §7 scopes naming to where you are standing, and until now that held because
//! nothing a domain made was wanted anywhere else: the laboratory consumed its
//! own reagents, the archive its own fragments. A **finished** thing breaks that
//! — a potion is brewed in the laboratory to be used elsewhere, and a scroll is
//! assembled in the archive to be spent elsewhere — and there was no way to
//! carry one at all. `carry`'s destination lookup wants a `Fixture` child of
//! `cwd`; a domain is neither. So `move clarity to archive` could not resolve,
//! and neither could any route between two rooms.
//!
//! §10's remaining five domains all have the same problem waiting for them, so
//! this is the standing answer rather than a fix for one pair.
//!
//! # The exemption is narrow, and it is stated
//!
//! `tower::scene` records that acting on a domain you are not in is **Phase 10's**
//! unlock. This does not repeal that: what reaches everywhere is the arsenal's
//! *contents*, on exactly the terms places, spells and the maze's readings
//! already have. A spellbook you carry is not a shelf you walk to, and neither
//! is a bandolier.
//!
//! What the arsenal is **not** is a second dispensary. It refuses stock at the
//! door, so it cannot become the room where everything ends up: the laboratory's
//! shelf is still the fallback a `move` falls back to, and reagents still belong
//! to the domain that uses them.
//!
//! # Nameable is not enough
//!
//! Every lookup that can now *name* what is in here has to be able to *reach* it,
//! or a word resolves at full confidence and then reports "no such thing" —
//! §15's dead end, arriving through the affordance meant to remove one. That is
//! what [`kept`] is for, and it has three callers: `pipeline::reachable`,
//! `pipeline::purge` and `files::here_or_place`.

use bevy_ecs::prelude::*;

use super::node::{Keep, Nameable, children_of, root};
use crate::parser::NounKind;

/// The arsenal's name, for the one place `build` needs to spell it.
pub const ARSENAL: &str = "arsenal";

/// The tower's arsenal, if it has one.
///
/// Walked from the root rather than read out of `Cwd`, which is the whole point:
/// this is the one place whose contents do not depend on where the player is
/// standing.
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
/// **In one place, so the lookups cannot disagree.** Three of them ask —
/// `pipeline::reachable`, `pipeline::purge` and `files::here_or_place` — and
/// §19 records what happens when a rule like this has copies: three hand-built
/// versions of *what is in this instrument* all dropped the stock count, and
/// nothing noticed until a recipe wanted more than one of something.
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
/// **Finished work only**, which is the rule that keeps this from becoming a
/// second dispensary: an [`Essence`](NounKind::Essence) is §10.1's *quality* a
/// recipe yields and a [`Scroll`](NounKind::Scroll) is a thing you spend. A
/// reagent, a fragment, a file or a vessel is refused at the door and stays
/// where it is.
///
/// Asked of the **kind**, never of the name. Telling finished work from stock by
/// name would mean the tower deciding which reagents are waste, which §10.1
/// refuses outright — every byproduct is some other recipe's input, and the
/// judgement is exactly the thing the recipes are meant to keep changing.
#[must_use]
pub fn admits(world: &World, node: Entity) -> bool {
    matches!(
        world.get::<Nameable>(node).map(|kind| kind.0),
        Some(NounKind::Essence | NounKind::Scroll)
    )
}
