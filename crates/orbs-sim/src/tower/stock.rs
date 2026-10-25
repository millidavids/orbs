//! How much of a thing a place holds.
//!
//! One node per kind with a count, not one per unit, which would be five `survey`
//! rows and five names to tell apart. Endless is a variant, not a name check,
//! decided where the stock is built: a base reagent never depletes (§11.5).

use bevy_ecs::prelude::*;

use super::{Name, Nameable, NodeIds};
use crate::parser::NounKind;

/// How many of a reagent one node stands for.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stock {
    /// A base reagent: the tower always has more.
    ///
    /// Not a large count, which would still tick down and end.
    Endless,
    /// A made thing: this many, and no more until more is made.
    Counted(u32),
}

impl Stock {
    /// What to draw beside the name.
    ///
    /// `∞` is CP437 0xEC, so the tube can draw it — the same check the em-dash
    /// failed in §4's boot text.
    #[must_use]
    pub fn label(self) -> String {
        match self {
            Self::Endless => "\u{221e}".to_owned(),
            Self::Counted(count) => count.to_string(),
        }
    }

    /// Whether `wanted` can be taken from this.
    #[must_use]
    pub const fn has(self, wanted: u32) -> bool {
        match self {
            Self::Endless => true,
            Self::Counted(count) => count >= wanted,
        }
    }
}

/// The thing called `named` directly inside `place`, if it is there.
#[must_use]
pub fn find(world: &World, place: Entity, named: &str) -> Option<Entity> {
    super::reach::look(world)
        .scope(super::reach::Scope::In(place))
        .find(named)
}

/// What `place` holds: each name once, with how many units of it are there.
///
/// The one derivation of what
/// [`Recipes::matching`](crate::content::Recipes::matching) needs; hand-built
/// copies dropped the count. [`Endless`](Stock::Endless) reports [`u32::MAX`].
/// `&World`, so `panel::read` can share the rule. A
/// [`Sense`](crate::parser::NounKind::Sense) child is skipped, or a published
/// word would stop a recipe matching.
#[must_use]
pub fn holdings(world: &World, place: Entity) -> Vec<(String, u32)> {
    super::children_of(world, place)
        .into_iter()
        .filter_map(|node| {
            if world.get::<super::Nameable>(node).map(|kind| kind.0)
                == Some(crate::parser::NounKind::Sense)
            {
                return None;
            }
            let name = world.get::<Name>(node)?.0.clone();
            let units = match world.get::<Stock>(node).copied() {
                Some(Stock::Endless) => u32::MAX,
                Some(Stock::Counted(count)) => count,
                // Not stock at all — a file, a spell. Present, and one of it.
                None => 1,
            };
            Some((name, units))
        })
        .collect()
}

/// How much of `named` is in `place`.
#[must_use]
pub fn held(world: &World, place: Entity, named: &str) -> Option<Stock> {
    find(world, place, named).and_then(|node| world.get::<Stock>(node).copied())
}

/// Take `wanted` of `named` out of `place`, and say whether it was there.
///
/// A pile at zero is despawned, so "is there any" is still whether a node
/// exists.
pub fn take(world: &mut World, place: Entity, named: &str, wanted: u32) -> bool {
    let Some(node) = find(world, place, named) else {
        return false;
    };
    match world.get::<Stock>(node).copied() {
        // No count at all: a place, a file, a spell. Not stock, and not takeable
        // by this route — `move` on one of those is a different verb's business.
        None => false,
        Some(Stock::Endless) => true,
        Some(Stock::Counted(count)) if count >= wanted => {
            if count == wanted {
                world.entity_mut(node).despawn();
            } else if let Some(mut stock) = world.get_mut::<Stock>(node) {
                *stock = Stock::Counted(count - wanted);
            }
            true
        }
        Some(Stock::Counted(_)) => false,
    }
}

/// Put `wanted` of `named` into `place`, merging with whatever is there.
///
/// Merging, or grinding twice leaves two nodes the parser cannot tell apart.
pub fn give(world: &mut World, place: Entity, named: &str, kind: NounKind, wanted: u32) -> Entity {
    if let Some(node) = find(world, place, named) {
        match world.get::<Stock>(node).copied() {
            // Endless stays endless: adding to what is already inexhaustible
            // changes nothing, and turning it into a count would quietly make it
            // exhaustible.
            Some(Stock::Endless) => return node,
            Some(Stock::Counted(count)) => {
                if let Some(mut stock) = world.get_mut::<Stock>(node) {
                    *stock = Stock::Counted(count.saturating_add(wanted));
                }
                return node;
            }
            None => {}
        }
    }

    let id = world.resource_mut::<NodeIds>().issue();
    let node = world
        .spawn((
            id,
            Name(named.to_owned()),
            Nameable(kind),
            Stock::Counted(wanted),
        ))
        .id();
    world.entity_mut(node).insert(ChildOf(place));
    node
}

/// Put an inexhaustible pile of `named` in `place`.
///
/// `build`'s shape at run time, so a base reagent unlocked mid-game is as
/// endless as the first three. Idempotent; the existing pile wins.
pub fn give_endless(world: &mut World, place: Entity, named: &str, kind: NounKind) -> Entity {
    if let Some(node) = find(world, place, named) {
        if let Some(mut stock) = world.get_mut::<Stock>(node) {
            *stock = Stock::Endless;
        }
        return node;
    }
    let node = give(world, place, named, kind, 1);
    if let Some(mut stock) = world.get_mut::<Stock>(node) {
        *stock = Stock::Endless;
    }
    node
}
