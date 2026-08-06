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
    pub const fn issue(&mut self) -> NodeId {
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

/// A place that is furniture in a room rather than somewhere you travel to.
///
/// §10.1's instruments. A fixture is a real place — `attend alembic` and
/// `survey alembic` both work — but its **contents are nameable from the room it
/// stands in**, because someone in the laboratory can plainly reach the sage in
/// the mortar. That is what makes the pipeline typable: `move husks from alembic
/// to dispensary` has to be able to name `husks`.
///
/// It does not weaken §19's *"you can only name what is where you are"*. That
/// rule stops you acting on another **domain** at a distance, and a domain is
/// never a fixture — the marker is exactly the line between the two.
#[derive(Component, Debug, Clone, Copy)]
pub struct Fixture;

/// The fixture that burns fuel for the ones that need heat — §10.1's athanor.
///
/// **A marker, not `name == ATHANOR`.** Six sites compared a `Name` against a
/// `&'static str` to decide whether a fixture behaves like the rest, with nothing
/// binding them together: no test, no type. §10 puts five more domains in Phase
/// 3a, and the day a second room gets a forge, `wield forge` starts an ordinary
/// run with no recipe instead of lighting it. That is six edits the compiler
/// never asks for; this is one component it does.
#[derive(Component, Debug, Clone, Copy)]
pub struct HeatSource;

/// The operation an instrument performs, named as its own verb.
///
/// §10.1's loop is four commands a stage, and the two in the middle — charge it,
/// start it — are the ones a player types most. `grind sage` collapses them by
/// naming the *operation* instead of the tool, which is also how the domain
/// talks: you grind sage, you do not move sage into a mortar and then operate
/// the mortar.
///
/// **A component, not a table of names.** The alternative is a
/// `match verb { Grind => "mortar_and_pestle", … }` somewhere in the executor,
/// which is the same name-string dispatch the athanor and the dispensary were
/// just moved off — six sites branching on a `&'static str` with nothing binding
/// them together. Here the instrument declares what it does, in the one place
/// instruments are declared, and §10's five further domains can coin their own
/// verbs without touching the executor at all.
#[derive(Component, Debug, Clone, Copy)]
pub struct Operation(pub crate::parser::Verb);

/// A shelf of stock rather than an instrument — §10.1's dispensary.
///
/// Two things read it: `reachable` searches it **last** (stock is the fallback),
/// and the panel leaves it out, because a row that reads `charged` from the first
/// tick to the last teaches the eye to skip the panel.
#[derive(Component, Debug, Clone, Copy)]
pub struct Store;

/// A file whose text is **stored**, rather than derived from the record stream.
///
/// # Why this is not how `orb.log` works, and must not become it
///
/// §3 forbids unlogged output, which makes the record stream *the* log: `orb.log`
/// is the whole of it and `laboratory.log` is that same stream filtered by where
/// each line happened. Neither has contents of its own, and that is what keeps a
/// search working when the eldritch renderer corrupts the display — the filter
/// runs over field values and never over anything a view put on screen.
///
/// A `.spell` is the opposite kind of thing: lines a player wrote, which stay
/// exactly as written until the player changes them. Deriving one would mean
/// inventing records for it; filtering the stream to find it would mean a spell
/// could be edited by something the orb happened to say.
///
/// So `peruse` reads this when it is present and falls back to the stream when
/// it is not, and the two paths stay separate. **`Name` alone does not say which
/// kind a file is** — the component does.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Held(pub Vec<String>);

/// The domain a spell is written for.
///
/// # Data on the node, not a directory
///
/// The obvious shape is `/grimoire/laboratory/morning.spell`, and it breaks the
/// tower: a directory spawns as a [`NounKind::Place`](crate::parser::NounKind),
/// so a `laboratory` under the grimoire collides on the leaf with
/// `/tower/laboratory` — `find_place` matches a full path *or* a bare leaf, so
/// `attend laboratory` would become a walk-order coin flip between a room and a
/// folder, and `every_place_leaf_is_unique` fails, which is the test that exists
/// *"rather than the echo quietly starting to lie."*
///
/// What was wanted is which domain a spell is **for**, and that is a fact about
/// the spell rather than about where its bytes live.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Domain(pub String);

/// The leaf name of `place`, for [`FieldName::At`](orbs_render::FieldName::At).
///
/// One helper rather than a `world.get::<Name>(...).map_or_else(...)` at each of
/// eleven emit sites, because those eleven already disagreed once about *which
/// field* the instrument went in and the fix is worth nothing if they can drift
/// again on *what it is called*.
#[must_use]
pub fn where_at(world: &World, place: Entity) -> String {
    world
        .get::<Name>(place)
        .map_or_else(String::new, |name| name.0.clone())
}

/// The domain `node` belongs to, if it is in one.
///
/// A **domain** is a child of the tower — `laboratory`, `archive` — as opposed
/// to a fixture standing inside one, or the tower itself, or the grimoire.
/// Walking up rather than comparing names, so §10's five further domains need no
/// edit here.
///
/// Returns `None` at `/tower` and at `/grimoire`: neither is somewhere work
/// happens, which is what makes `scribe` there a question with no answer.
#[must_use]
pub fn domain_of(world: &World, node: Entity) -> Option<Entity> {
    let root = filesystem_root(world, node);
    let tower = children_of(world, root).into_iter().next()?;

    let mut at = node;
    loop {
        let parent = world.get::<ChildOf>(at).map(ChildOf::parent)?;
        if parent == tower {
            return Some(at);
        }
        at = parent;
    }
}

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

/// The topmost node of the tree `node` hangs from.
///
/// # Why this is one function and not five
///
/// The walk itself is four lines, and it was written out four times — once in
/// `execute::navigate` as `root`, once inside `tower::scene` as `root_of`, once
/// inline in `execute::files` to find a domain log, and once per test that
/// wanted "the root" and reached for [`Cwd`] instead.
///
/// That last shortcut is the dangerous one, because **`Cwd` is where the player
/// stands, not where the tree begins**, and the two are the same thing only for
/// as long as `/tower` is the only root. §8 puts the player's spells in a
/// `/grimoire` beside it rather than inside it; the moment that lands, every
/// test that walked from `Cwd` keeps passing while quietly covering half the
/// filesystem — including `every_place_leaf_is_unique`, whose whole purpose is
/// failing *"rather than the echo quietly starting to lie."*
///
/// So the walk lives here, beside [`path_of`] and [`children_of`], and the day
/// the root moves it is one function that learns about it.
#[must_use]
pub fn filesystem_root(world: &World, node: Entity) -> Entity {
    let mut at = node;
    while let Some(parent) = world.get::<ChildOf>(at).map(ChildOf::parent) {
        at = parent;
    }
    at
}

/// The topmost node of the tree the player is standing in.
///
/// See [`filesystem_root`] for why this is not simply [`Cwd`].
#[must_use]
pub fn root(world: &World) -> Entity {
    filesystem_root(world, world.resource::<Cwd>().0)
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
