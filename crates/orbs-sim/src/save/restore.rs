//! Putting a document back into a world.
//!
//! # Raise first, then adopt
//!
//! The caller has already run `tower::raise`, so the tower exists before a word
//! of the save is applied. This pass then reconciles: a node the save knows is
//! **found by path** and given its components; one it knows and the tower does
//! not is spawned under its parent; and a *stock* pile the tower raised that the
//! save has never heard of is despawned, because that is a starting reagent the
//! player used up.
//!
//! The alternative — letting the save own the whole tree — is simpler and wrong
//! in a way that arrives on schedule. Five more domains land between here and
//! Phase 9a, and an authoritative save would open into a tower permanently
//! lacking them. Refusing the old save by format version is not an answer,
//! because it deletes it.
//!
//! # Nothing here draws from a random stream
//!
//! `Ward::from_save` and `Maze::from_save` read their state rather than rolling
//! it, and this pass calls no generator. A restore that re-dug the maze would
//! move `RngStream::Archive` and change every later roll in the session — which
//! would show up as the loaded world diverging from the one that saved it, on
//! the same seed, for a reason nothing on screen could explain.
//!
//! # Where the rest of it is
//!
//! This owns the world's **shape** — the clock, every stream position, the
//! tree, and the resources. What is true of each *node* is [`super::adopt`], and
//! the seam between them is real rather than a line count: everything there
//! needs the tree to already exist.
//!
//! # What a bad file may do
//!
//! §15 accepts hand-editing: *"hand-editing a TOML file only affects the person
//! doing it."* So every lookup here is fallible and nothing panics. A path that
//! names nothing is skipped, a word that names no verb is dropped, and a `Cwd`
//! that has gone missing falls back to `/tower` — a player standing nowhere is
//! a game with no prompt.

use bevy_ecs::prelude::*;
use orbs_render::Records;

use super::adopt;
use super::document::Save;
use super::naming;
use super::node::NodeSave;
use crate::rng::{RngStream, Rngs};
use crate::session::{Scrollback, Wizard};
use crate::tick::Tick;
use crate::tower::{self, Cwd, Marks, Stock};

/// Apply a save over a freshly raised tower.
///
/// The order is load-bearing in one place: [`spells`] runs **after**
/// [`stream`], because a spell whose text has moved under it says so in a
/// record, and a record pushed before the stream is rebuilt would be thrown
/// away by the rebuild.
pub(crate) fn restore(world: &mut World, save: &Save) {
    clock_and_rolls(world, save);
    tree(world, save);
    progress(world, save);
    stream(world, save);
    adopt::references(world, save);
    prompt(world, save);
}

/// The clock, and where each random stream stood.
fn clock_and_rolls(world: &mut World, save: &Save) {
    world.insert_resource(Tick::new(save.world.tick));

    let mut positions = [0_u128; RngStream::COUNT];
    for (slot, text) in positions.iter_mut().zip(&save.rng.positions) {
        *slot = text.parse().unwrap_or(0);
    }
    world.insert_resource(Rngs::restore(save.world.seed, positions));
}

/// The tree: adopt what is there, spawn what is not, clear what was used up.
fn tree(world: &mut World, save: &Save) {
    // Every path the save knows, so the sweep below can tell a pile the player
    // consumed from a fixture a later phase added.
    let known: std::collections::BTreeSet<&str> =
        save.nodes.iter().map(|node| node.path.as_str()).collect();

    let mut placed: Vec<Entity> = Vec::with_capacity(save.nodes.len());
    for node in &save.nodes {
        let entity = tower::find_by_path(world, &node.path).unwrap_or_else(|| spawn(world, node));
        adopt::apply(world, entity, node);
        // The node's own id, restored rather than re-issued. See `NodeSave::id`:
        // `spell::advance` orders running spells by it, so a world that re-issued
        // them would interleave two player-written spells differently.
        world
            .entity_mut(entity)
            .insert(tower::NodeId::from_raw(node.id));
        placed.push(entity);
    }

    // **Anything raised that the save does not name, and that the player could
    // have destroyed.** Stock and spells are the two kinds a command can remove
    // — `purge` on a shipped `first_light.spell` is undone by the next load
    // without this, in a release build.
    //
    // `Protected` and `Fixture` are the exemption, and they are what makes
    // adopting worth doing: a room or an instrument the save has never heard of
    // is a domain a later phase added, and despawning one would undo the
    // upgrade the player just bought. The cost is stated rather than hidden — a
    // *destructible* thing shipped in a later build (a new spell on the shelf)
    // is swept the first time an older save is opened.
    //
    // Collected first and re-checked inside the loop: `Children` is
    // `linked_spawn` in bevy_ecs 0.19, so despawning a parent takes its
    // descendants, and a later entry in this list can already be gone.
    for entity in every_node(world) {
        if world.get_entity(entity).is_err() {
            continue;
        }
        let destructible =
            world.get::<Stock>(entity).is_some() || world.get::<tower::Held>(entity).is_some();
        let spared = world.get::<tower::Protected>(entity).is_some()
            || world.get::<tower::Fixture>(entity).is_some();
        if destructible && !spared && !known.contains(tower::path_of(world, entity).as_str()) {
            world.entity_mut(entity).despawn();
        }
    }

    // **The counter is wound past every id the save carried**, or the next node
    // the tower issues collides with one just restored — two nodes with one
    // identity, and `spell::advance`'s ordering key stops being an ordering.
    let highest = save.nodes.iter().map(|node| node.id).max();
    if let Some(highest) = highest {
        world.resource_mut::<tower::NodeIds>().wind_past(highest);
    }

    // **`Children` order is rebuilt to the document's**, and it is not cosmetic.
    // `tower::node` opens by saying anything a player can see must come from
    // walking `Children` in insertion order, because §6 resolves noun ties to
    // whichever was registered first.
    //
    // A renamed node is what breaks it. `sabotage::substitute` renames a pile in
    // place — `sage` becomes `sage-` — so on the next load its saved path matches
    // nothing raised, it is *spawned* and appended, and the raised `sage` is
    // swept above. The pile leaves its slot and lands last, and from then on an
    // ambiguous phrase resolves to a different noun than it did before the save.
    //
    // Re-inserting `ChildOf` in document order costs one relationship hook per
    // node at ~50 nodes and settles it for every cause, not only this one.
    for (entity, node) in placed.iter().zip(&save.nodes) {
        let (parent, _) = split(&node.path);
        if let Some(under) = tower::find_by_path(world, &parent) {
            world.entity_mut(*entity).insert(ChildOf(under));
        }
    }
}

/// Create a node the tower does not have, under the parent its path names.
///
/// Capture walks parents before children, so the parent is always in place by
/// the time this is reached. A path whose parent is somehow missing lands at the
/// filesystem root rather than nowhere — visible, and recoverable by hand.
fn spawn(world: &mut World, node: &NodeSave) -> Entity {
    let (parent, name) = split(&node.path);
    let under = tower::find_by_path(world, &parent)
        .unwrap_or_else(|| tower::filesystem_root(world, world.resource::<Cwd>().0));
    let id = world.resource_mut::<tower::NodeIds>().issue();
    let kind = naming::kind_from(&node.kind).unwrap_or(crate::parser::NounKind::Any);
    world
        .spawn((id, tower::Name(name), tower::Nameable(kind), ChildOf(under)))
        .id()
}

/// `/tower/laboratory/sage` becomes `/tower/laboratory` and `sage`.
fn split(path: &str) -> (String, String) {
    path.rsplit_once('/').map_or_else(
        || (String::new(), path.to_owned()),
        |(parent, name)| (parent.to_owned(), name.to_owned()),
    )
}

/// Every node in the tree, in `Children` order.
fn every_node(world: &World) -> Vec<Entity> {
    let root = tower::filesystem_root(world, world.resource::<Cwd>().0);
    let mut found = Vec::new();
    let mut stack = tower::children_of(world, root);
    while let Some(entity) = stack.pop() {
        found.push(entity);
        stack.extend(tower::children_of(world, entity));
    }
    found
}

/// What the player has earned, found and chosen.
fn progress(world: &mut World, save: &Save) {
    let progress = &save.progress;
    if !progress.wizard.is_empty() {
        world.resource_mut::<Wizard>().rename(&progress.wizard);
    }
    world
        .resource_mut::<tower::Experience>()
        .restore(progress.experience);
    // **Absent means whole, not nothing.** A save written before the
    // sanctum existed says nothing about integrity, and defaulting a missing
    // field to nought would hand every returning player a tower worn to the
    // ground — see `ProgressSave::integrity`.
    world
        .resource_mut::<tower::Integrity>()
        .restore(progress.integrity.unwrap_or(tower::STANDING));
    // An absent row is *nothing is cooling*, which is what a save written before
    // §8.1's rationing existed honestly says — `Cooling::from_save` takes a
    // short list for that reason.
    world.insert_resource(tower::Cooling::from_save(&progress.cooling));
    world
        .resource_mut::<tower::Taken>()
        .restore(progress.taken.clone());
    world
        .resource_mut::<tower::Learned>()
        .restore(progress.learned.clone(), progress.fruitless);
    world.resource_mut::<Marks>().restore(
        progress
            .marks
            .iter()
            .filter_map(|mark| Some((mark.domain.clone(), naming::mark_from(&mark.mark)?))),
    );

    // A player standing nowhere is a game with no prompt, so a path that has
    // gone missing falls back to the tower rather than leaving `Cwd` as it was.
    if let Some(at) = progress
        .cwd
        .as_deref()
        .and_then(|path| tower::find_by_path(world, path))
        .or_else(|| tower::find_by_path(world, "/tower"))
    {
        world.insert_resource(Cwd(at));
    }
}

/// The tail of the record stream, and the sequence it sits at the end of.
fn stream(world: &mut World, save: &Save) {
    // What the tail is *not* carrying. `Records::resume` opens the stream there,
    // so `sequence` lands back where it was once the tail is pushed and
    // `dropped` reports the gap — which is what keeps a held spell's cursor
    // meaning what it meant.
    let dropped = save
        .world
        .sequence
        .saturating_sub(save.records.len() as u64);
    let mut records = Records::resume(dropped);
    records.set_register(
        save.progress
            .register
            .as_deref()
            .map_or(orbs_render::Presentation::Plain, naming::register_from),
    );

    for line in &save.records {
        let mut entry = records.push(naming::record_kind_from(&line.kind));
        if let Some(role) = line.role.as_deref() {
            entry = entry.role(naming::role_from(role));
        }
        if let Some(spoken) = line.spoken.as_deref() {
            entry = entry.spoken(spoken);
        }
        // **Asked for per record, not inherited from the stream.** `set_register`
        // above decides what *new* records are spoken in; a saved record carries
        // its own, and §8.1's poisoned-log tell is exactly that — `emit_lines`
        // stamps `Presentation::Tampered` on every third row, so a restore that
        // dropped this would answer `verify` with *tampered* and then draw the
        // log with a plain face. Captured and never read, until now.
        //
        // Safe beside `spoken`: `finish` asserts an eldritch record carries an
        // authored linear variant, and one that did not could never have been
        // written in the first place — the same assertion fired at capture.
        if let Some(register) = line.register.as_deref() {
            entry = entry.presentation(naming::register_from(register));
        }
        if line.quiet {
            entry = entry.quiet();
        }
        for (name, shape, value) in &line.fields {
            let Some(field) = naming::field_from(name) else {
                continue;
            };
            entry = match shape.as_str() {
                "count" => entry.count(field, value.parse().unwrap_or(0)),
                "tick" => entry.tick(field, value.parse().unwrap_or(0)),
                _ => entry.text(field, value),
            };
        }
        entry.finish();
    }

    *world.resource_mut::<Scrollback>().records_mut() = records;
}

/// Ask the open numbered question again.
///
/// **After the scene is rebuilt, which is why it is its own step.** `analyse`
/// takes the `Scene` *resource*, and at the top of a restore that resource is
/// still the empty default — `tower::raise` does not build one. Re-asking there
/// resolved against a world that named nothing, so the prompt came back with a
/// different list or none at all.
///
/// (`spell::compile` is not affected and was checked: `tower::scene_at` computes
/// a scene from the world rather than reading the resource.)
fn prompt(world: &mut World, save: &Save) {
    tower::rebuild(world);

    // **The numbered question, asked again.** The readings themselves are not in
    // the document — an `Intent` would pin the format against every parser
    // change — so the line is re-analysed here. §19 settled that the parser's
    // tie-break draws no randomness, which is what makes that reproduce the same
    // list rather than merely a similar one.
    //
    // A reading the restored scene can no longer offer is one the player could
    // not have acted on either, so a shorter list is the honest answer and an
    // empty one leaves no question open.
    if let Some(line) = save.progress.asked.as_deref() {
        let analysis = crate::parser::analyse(
            line,
            world.resource::<crate::parser::Scene>(),
            crate::parser::Mode::Calm,
        );
        if let crate::parser::Resolution::Ambiguous { candidates } = analysis.resolution {
            let readings = candidates.into_iter().map(|c| c.intent).collect();
            world
                .resource_mut::<crate::session::Choices>()
                .offer(line, readings);
        }
    }
}
