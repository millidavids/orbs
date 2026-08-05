//! What a place in the tower is.
//!
//! DESIGN.md §7: *"The directory tree **is** the tower. Navigation is diegetic;
//! paths are places."* So a node is an entity, the tree is `ChildOf`/`Children`,
//! and the world model stays ECS throughout (architectural rule 1).
//!
//! # Two orderings, and only one of them is safe
//!
//! Anything a player can see must be derived by walking [`Children`], which is a
//! `Vec<Entity>` in insertion order. It must **never** come from a global query:
//! archetype order is not insertion order, and an entity moves to a new table
//! whenever a component is added or removed. Starting a brew would therefore
//! reorder a listing — and because §6's noun matching resolves ties to whichever
//! noun was registered first, it would silently change which noun a phrase
//! resolves to and break replay from the same seed. No test would catch it.

use bevy_ecs::prelude::*;

use crate::parser::NounKind;

/// A stable identity for a node, independent of this run.
///
/// [`Entity`] is a generational index: deterministic within a run and meaningless
/// across a save or a rebuilt world. §8 requires bound references resolve *"by
/// stable entity ID, not by name or path"* and writes them into script files as
/// `north_gate#7f2a`, so the durable identity has to exist from the start —
/// retrofitting it once scripts and saves both depend on `Entity` would mean
/// rewriting both.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u64);

impl NodeId {
    /// The number a script file would carry.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// An identity from its number, for tests and for loading a save.
    #[must_use]
    pub const fn from_raw(id: u64) -> Self {
        Self(id)
    }
}

/// Hands out [`NodeId`]s in a deterministic sequence.
///
/// A counter, not a hash and not an RNG draw: two runs from the same seed must
/// assign the same identity to the same node, or replay stops meaning anything.
#[derive(Resource, Debug, Default)]
pub struct NodeIds(u64);

impl NodeIds {
    /// The next identity.
    pub fn issue(&mut self) -> NodeId {
        let id = NodeId(self.0);
        self.0 += 1;
        id
    }
}

/// What a node is called, as a single path segment.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Name(pub String);

/// The category a node answers to when the player names it (§6).
///
/// Separate from [`Name`] because resolution is grounded in *what a thing is*:
/// `decoct clarity` resolves because clarity is an essence, and stops resolving
/// the moment it is not.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nameable(pub NounKind);

/// A node the player may not destroy.
///
/// §7: catastrophic targets are guarded *"in character — the orb refuses,
/// memorably"*. The refusal is a fact this component supplies; the memorable
/// sentence is composed from it by a content file in Phase 1 (rule 6, §12).
#[derive(Component, Debug, Clone, Copy)]
pub struct Protected;

/// Where the player is standing.
///
/// §7: paths are places, so this is a place rather than a string. The path is
/// recomputed for display by [`path_of`], because a node's name can change and a
/// cached string cannot.
#[derive(Resource, Debug, Clone, Copy)]
pub struct Cwd(pub Entity);

/// The canonical path of `node`, walking up to the root.
///
/// Returns `/`-joined segments with a leading slash — `/tower/laboratory`.
#[must_use]
pub fn path_of(world: &World, node: Entity) -> String {
    let mut segments = Vec::new();
    let mut at = Some(node);
    while let Some(entity) = at {
        if let Some(name) = world.get::<Name>(entity) {
            segments.push(name.0.clone());
        }
        at = world.get::<ChildOf>(entity).map(ChildOf::parent);
    }
    segments.reverse();
    format!("/{}", segments.join("/"))
}

/// Every child of `node`, in the order they were spawned.
///
/// The safe ordering. See the module docs for why a query is not.
#[must_use]
pub fn children_of(world: &World, node: Entity) -> Vec<Entity> {
    world
        .get::<Children>(node)
        .map(|children| children.iter().collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_are_assigned_in_order_and_never_reused() {
        let mut ids = NodeIds::default();
        let assigned: Vec<u64> = (0..4).map(|_| ids.issue().get()).collect();
        assert_eq!(assigned, [0, 1, 2, 3]);
    }

    #[test]
    fn two_runs_assign_the_same_identities() {
        // Replay from a seed reproduces the world, so the same node must come
        // back with the same identity or a bound script would point elsewhere.
        let mut a = NodeIds::default();
        let mut b = NodeIds::default();
        for _ in 0..8 {
            assert_eq!(a.issue(), b.issue());
        }
    }
}
