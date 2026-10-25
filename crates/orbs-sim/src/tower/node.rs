//! What a place in the tower is.
//!
//! DESIGN.md §7: *"The directory tree **is** the tower. Navigation is diegetic;
//! paths are places."* So a node is an entity, the tree is `ChildOf`/`Children`,
//! and the world model stays ECS throughout (architectural rule 1).
//!
//! Anything a player can see is derived by walking [`Children`], never a global
//! query: archetype order moves when a component is added, and since §6
//! resolves ties to the first registered, a brew starting would silently change
//! what a phrase means.

use bevy_ecs::prelude::*;

use crate::parser::NounKind;

/// A stable identity for a node, independent of this run.
///
/// [`Entity`] means nothing across a save or a rebuilt world, and §8 resolves
/// bound references by stable id, so this existed from the start.
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
    /// Make sure the next identity issued is above `id`.
    ///
    /// A save restores each node's own [`NodeId`], so the counter has to clear
    /// the highest of them or the next node the tower spawns is handed an
    /// identity something already has.
    pub(crate) const fn wind_past(&mut self, id: u64) {
        if self.0 <= id {
            self.0 = id + 1;
        }
    }

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
/// `recall clarity` resolves because clarity is an essence, and stops resolving
/// the moment it is not.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nameable(pub NounKind);

/// A node the player may not destroy.
///
/// §7 guards catastrophic targets in character; the sentence is a content
/// file's.
#[derive(Component, Debug, Clone, Copy)]
pub struct Protected;

/// A place that is furniture in a room rather than somewhere you travel to.
///
/// §10.1's instruments: real places whose contents are nameable from the room,
/// so `move husks from alembic to dispensary` can be typed. §19's *name only
/// what is where you are* still holds, because a domain is never a fixture.
#[derive(Component, Debug, Clone, Copy)]
pub struct Fixture;

/// The fixture that burns fuel for the ones that need heat — §10.1's athanor.
///
/// A marker, not six sites comparing names that the compiler cannot keep in
/// step.
#[derive(Component, Debug, Clone, Copy)]
pub struct HeatSource;

/// The operation an instrument performs, named as its own verb.
///
/// `grind sage` charges and starts in one by naming the operation, as the
/// domain talks. Declared on the instrument, so a new domain coins verbs without
/// touching the executor.
#[derive(Component, Debug, Clone, Copy)]
pub struct Operation(pub crate::parser::Verb);

/// A shelf of stock rather than an instrument — §10.1's dispensary.
///
/// Two things read it: `reachable` searches it **last** (stock is the fallback),
/// and the panel leaves it out, because a row that reads `charged` from the first
/// tick to the last teaches the eye to skip the panel.
#[derive(Component, Debug, Clone, Copy)]
pub struct Store;

/// Where finished work is kept, and the one place in the tower you can reach
/// from anywhere — §10.1's arsenal.
///
/// Not a [`Store`], which a `move` falls back to and would capture stray
/// reagents; finished work only. A narrow hole in §7 — see `tower::keep`.
#[derive(Component, Debug, Clone, Copy)]
pub struct Keep;

/// One of the four ways the archive's reading can go — see `tower::maze`.
///
/// A `NounKind::Place`, so `if north has passage` can be written, that is not
/// somewhere you go: `attend` refuses on it, and the panel leaves it off.
#[derive(Component, Debug, Clone, Copy)]
pub struct Reading;

/// One of a set a spell's `for each` walks — `for each way`, `for each socket`.
///
/// Declared (`build::Branch::group`), since [`Reading`] spans several sets in
/// one room. Singular, because it names the cursor: `for each way` binds `way`.
#[derive(Component, Debug, Clone)]
pub struct Grouped(pub String);

/// Everything at `node` belonging to the set `group`, in the order it was
/// raised.
///
/// Raise order through [`children_of`], never query order, or a rebuilt world
/// would walk a set differently.
#[must_use]
pub fn group_at(world: &World, node: Entity, group: &str) -> Vec<Entity> {
    // The arsenal's set is the only one walked wherever you stand, since its
    // contents are nameable from everywhere; every other set is in the room.
    let node = if group == super::stores::STORE {
        match super::keep(world) {
            Some(arsenal) => arsenal,
            None => return Vec::new(),
        }
    } else {
        node
    };
    children_of(world, node)
        .into_iter()
        .filter(|child| {
            world
                .get::<Grouped>(*child)
                .is_some_and(|Grouped(named)| named == group)
        })
        .collect()
}

/// Every set name `node` has children for, in raise order and without repeats.
///
/// What the manual lists and what `for each` will accept — derived rather than
/// written down twice, so a domain that declares a set gets it in both places.
#[must_use]
pub fn groups_at(world: &World, node: Entity) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for child in children_of(world, node) {
        if let Some(Grouped(named)) = world.get::<Grouped>(child)
            && !names.iter().any(|already| already == named)
        {
            names.push(named.clone());
        }
    }
    names
}

/// The reading words a room can answer `has` with, in set order.
///
/// Keyed on the declared set, not [`Reading`], which once had the lens's page
/// teach the archive's words. A domain adding a set adds an arm here.
#[must_use]
pub fn readings_at(world: &World, node: Entity) -> Vec<&'static str> {
    let mut words: Vec<&'static str> = Vec::new();
    for set in groups_at(world, node) {
        for word in readings_of(&set) {
            if !words.contains(&word) {
                words.push(word);
            }
        }
    }
    words
}

/// What a set's members answer `has` with. See [`readings_at`].
fn readings_of(set: &str) -> Vec<&'static str> {
    match set {
        "way" => super::maze::readings(),
        "socket" | "sigil" => super::ward::readings(),
        "station" => super::pylon::readings(),
        // A glyph answers with the humour it is limned with, and a humour with
        // nothing — it is a word to name, not a thing that holds a reading.
        super::circle::GLYPHS => super::circle::Humour::ALL
            .into_iter()
            .map(super::circle::Humour::word)
            .collect(),
        // The forge's two sets. A column answers with `lit` and nothing else —
        // absence is what "dark" means here — and a charm answers with how long
        // it has and whether that is nearly out.
        "column" => vec!["lit"],
        "charm" => vec![super::charm::GRACED, super::charm::EBBING],
        // The arsenal's items answer with how well stocked the tower is —
        // `fresh`, `thin`, `spent` — which is what lets a spell keep its own
        // stores up: walk them, find what has gone thin, go make it. And with
        // their supply's `draw` and `drained` (§19, *Supply*).
        super::stores::STORE => {
            let mut words = super::stores::readings();
            words.extend(super::supply::readings());
            words
        }
        // The bailey's three have no arm, and that is a gap rather than a
        // decision: `siege::readings()` returns all eighteen words as one list,
        // and splitting them across `area`, `die` and `band` is a judgement
        // about which word belongs where.
        _ => Vec::new(),
    }
}

/// A file whose text is **stored**, rather than derived from the record stream.
///
/// A log is the record stream and has no contents of its own (§3); a `.spell`
/// is lines a player wrote, which stay as written. `peruse` reads this when
/// present and the stream when not.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Held(pub Vec<String>);

/// [`Held`], as the orb read it — the lines `spell::compile` actually compiles.
///
/// Derived and beside [`Held`], which stays byte-exact: save-time rewriting
/// destroyed the player's words (§19). `compile` reads this and only this.
/// Fingerprinted per line, since the editor writes after every pause; kept only
/// for the reader [`by`](Self::by) that made it. Saved and never re-derived,
/// because a GPU's reading does not replay (§19).
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Read {
    /// One line per line of [`Held`], canonical where the orb could read it.
    pub lines: Vec<String>,
    /// The hash of the [`Held`] line each of those was read from.
    pub of: Vec<u64>,
    /// The [`Scrivener::identity`](crate::Scrivener::identity) of the reader
    /// that made [`lines`](Self::lines), or [`None`] where no reader's answer
    /// can be vouched for.
    ///
    /// `None` is a spell no reader in this session read — authored, or restored
    /// from a save. Its lines compile and are not taken for what the reader in
    /// hand would say; the first write reads every line again.
    pub by: Option<u64>,
}

impl Read {
    /// `lines` as the reading of `held`, line for line, made by `by`.
    #[must_use]
    pub fn new(held: &[String], lines: Vec<String>, by: Option<u64>) -> Self {
        Self {
            lines,
            of: held.iter().map(|line| fingerprint_of(line)).collect(),
            by,
        }
    }

    /// `held` as its own reading — what a spell nobody read compiles.
    #[must_use]
    pub fn verbatim(held: &[String]) -> Self {
        Self::new(held, held.to_vec(), None)
    }

    /// The line at `at` as it compiles: its reading, if that was read from
    /// `held`, and `held` itself otherwise.
    ///
    /// Line by line, never all or nothing: every write keeps the two in step, so
    /// a mismatch comes from outside the game — a hand-edited save, most likely
    /// — and one edited line is no reason to discard every other reading.
    #[must_use]
    pub fn compiled<'a>(&'a self, at: usize, held: &'a str) -> &'a str {
        match self.lines.get(at) {
            Some(line) if self.of.get(at) == Some(&fingerprint_of(held)) => line,
            _ => held,
        }
    }

    /// This reading laid over `held`: every line kept that was read from the
    /// text now at its place, and the rest as written.
    ///
    /// [`by`](Self::by) survives only if every line did, because a line put
    /// back as written is not something that reader said.
    #[must_use]
    pub fn aligned(&self, held: &[String]) -> Self {
        let lines: Vec<String> = held
            .iter()
            .enumerate()
            .map(|(at, line)| self.compiled(at, line).to_owned())
            .collect();
        let whole = self.lines.len() == held.len()
            && held
                .iter()
                .enumerate()
                .all(|(at, line)| self.of.get(at) == Some(&fingerprint_of(line)));
        Self::new(held, lines, if whole { self.by } else { None })
    }

    /// What the reader `by` made of `line`, if this holds that reading.
    ///
    /// By text, not by position. A reader is asked about one line at a time and
    /// answers from nothing else, so a line's reading goes wherever the line
    /// does — and a line typed above it no longer sends every line below back to
    /// the reader.
    #[must_use]
    pub fn kept(&self, line: &str, by: u64) -> Option<&str> {
        if self.by != Some(by) {
            return None;
        }
        let hash = fingerprint_of(line);
        self.of
            .iter()
            .position(|of| *of == hash)
            .and_then(|at| self.lines.get(at))
            .map(String::as_str)
    }
}

/// One line's fingerprint — `save::fingerprint`, which is the one hash.
fn fingerprint_of(line: &str) -> u64 {
    crate::save::fingerprint(&[line.to_owned()])
}

/// The domain a spell is written for.
///
/// Data on the node, not a directory, which would be a second `laboratory`
/// place and make `attend laboratory` a coin flip.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Domain(pub String);

/// The leaf name of `place`, for [`FieldName::At`](orbs_render::FieldName::At).
///
/// One helper for eleven emit sites, which once disagreed about the field.
#[must_use]
pub fn where_at(world: &World, place: Entity) -> String {
    world
        .get::<Name>(place)
        .map_or_else(String::new, |name| name.0.clone())
}

/// The domain `node` belongs to, if it is in one.
///
/// A domain is a child of the tower, found by walking up. `None` at `/tower` and
/// `/grimoire`, where no work happens.
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
/// One function, not five: the four-line walk was written out four times, and
/// once per test that wanted "the root" and reached for [`Cwd`]. That shortcut
/// is the dangerous one — `Cwd` is where the player stands, not where the tree
/// begins, and with `/grimoire` beside `/tower` every test that walked from
/// `Cwd` keeps passing while covering half the filesystem.
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

/// The child of where the player stands that carries `C`, if any.
#[must_use]
pub fn here_with<C: Component>(world: &World) -> Option<Entity> {
    let cwd = world.resource::<Cwd>().0;
    children_of(world, cwd)
        .into_iter()
        .find(|node| world.get::<C>(*node).is_some())
}

/// The path of the node carrying `id`, if the tower still holds one.
///
/// A save needs it because a [`NodeId`] is a counter in spawn order, stable only
/// while `build`'s tables are — so a save keyed on one is invalidated by a phase
/// that adds a domain. The document spells every reference as a path instead;
/// this is one direction of that translation and [`find_by_path`] the other.
#[must_use]
pub fn path_of_id(world: &World, id: NodeId) -> Option<String> {
    first_in_tree(world, |entity| {
        (world.get::<NodeId>(entity) == Some(&id)).then(|| path_of(world, entity))
    })
}

/// The first answer `seen` gives, walking the whole tree depth-first in
/// insertion order.
#[must_use]
pub fn first_in_tree<T>(world: &World, mut seen: impl FnMut(Entity) -> Option<T>) -> Option<T> {
    let root = filesystem_root(world, world.resource::<Cwd>().0);
    walk(world, root, &mut seen)
}

/// The node at `path`, if there is one.
#[must_use]
pub fn find_by_path(world: &World, path: &str) -> Option<Entity> {
    // An empty path names nothing, and used to name the root: `split('/')` over
    // `""` yields no segments, so the walk returned where it started and every
    // "this path is gone" guard resolved to the filesystem root instead.
    if path.split('/').all(str::is_empty) {
        return None;
    }
    let mut at = filesystem_root(world, world.resource::<Cwd>().0);
    for segment in path.split('/').filter(|part| !part.is_empty()) {
        at = children_of(world, at).into_iter().find(|child| {
            world
                .get::<Name>(*child)
                .is_some_and(|name| name.0 == segment)
        })?;
    }
    Some(at)
}

/// Depth-first, in `Children` order, stopping at the first answer.
///
/// The ordering rule this module opens with: insertion order, never a global
/// query, because archetype order is not insertion order and §6 resolves noun
/// ties to whichever was registered first.
fn walk<T>(world: &World, from: Entity, seen: &mut impl FnMut(Entity) -> Option<T>) -> Option<T> {
    if let Some(found) = seen(from) {
        return Some(found);
    }
    children_of(world, from)
        .into_iter()
        .find_map(|child| walk(world, child, seen))
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
