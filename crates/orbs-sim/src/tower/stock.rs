//! How much of a thing a place holds.
//!
//! # One node per kind, with a count — not one node per unit
//!
//! A node per unit would make five sage five entities, five `survey` rows and
//! five names the parser has to tell apart, all called the same thing. The count
//! lives on the one node instead, which keeps every existing lookup — `find the
//! child named sage` — working exactly as it did.
//!
//! # Endless is a component state, not a special name
//!
//! §11.5 wants the laboratory to always have something to do, and a tower whose
//! sage runs out after one grind does not. So the base reagents are **endless**:
//! taking from them never depletes them.
//!
//! **A variant rather than a `sage`/`rock-salt`/`charcoal` name check**, for the
//! reason `tower::Role` already records — six sites once branched on `name ==
//! ATHANOR` with nothing binding them together. Which reagents are inexhaustible
//! is a property of the tower's stock, decided once where it is built, and §10's
//! five further domains will each have their own.

use bevy_ecs::prelude::*;

use super::{Name, Nameable, NodeIds};
use crate::parser::NounKind;

/// How many of a reagent one node stands for.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stock {
    /// A base reagent: the tower always has more.
    ///
    /// Not "a very large number". A count that merely started high would still
    /// tick down on the panel and still end, and the promise this exists to make
    /// is that it does not.
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
/// **The shape [`Recipes::matching`](crate::content::Recipes::matching) needs**,
/// and the one place it is derived. Three callers built the name list by hand and
/// all three dropped the count on the floor, which is invisible until a recipe
/// wants more than one of something — see [`Recipe::count`](crate::content::Recipe::count).
///
/// [`Endless`](Stock::Endless) reports [`u32::MAX`]: the tower always has more, so
/// no count can be short of it.
/// `&World`, not `&mut`: `contents` is `children_of` with a mutable borrow it
/// does not use, and demanding one here would keep this out of `panel::read`,
/// which is the caller that most needs it to be the same rule.
///
/// # A reading is not stock
///
/// [`Sense`](crate::parser::NounKind::Sense) children are skipped, and the rule
/// is general rather than a special case for the one place it bites. The maze
/// publishes what it can see as named children — `passage`, `back`, `spoil`, and
/// the lectern's own errand — and the lectern is an **instrument**, so a reading
/// counted here would enter the multiset `Recipes::matching` compares: four
/// fragments plus one word is not four fragments, the recipe would stop
/// matching, and the panel would read `fouled` for a lectern with nothing wrong
/// with it. Nothing anywhere wants a word to be a unit of something.
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
/// A pile that reaches zero is despawned, so "is there any" stays the same
/// question it always was — a node existing — and `if the mortar is empty` does
/// not have to learn about counts.
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
/// **Merging is the point.** Without it, grinding twice leaves two nodes both
/// called `ground-sage` in the dispensary — two `survey` rows for one thing, and
/// a name the parser has to choose between arbitrarily. That was already true
/// before counts existed; it simply had no way to show.
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

/// Put an **inexhaustible** pile of `named` in `place`.
///
/// The shape `build` gives the three the tower starts with, reached at run time:
/// a base reagent unlocked mid-game has to be as endless as one the laboratory
/// opened with, or the same word means a different thing depending on when it
/// arrived — and a recipe written against it would work for a while and then
/// stop.
///
/// Idempotent, and the one that already exists wins. `give` would have converted
/// nothing: it returns early on an `Endless` node precisely so a second gift
/// cannot make an inexhaustible pile exhaustible.
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
