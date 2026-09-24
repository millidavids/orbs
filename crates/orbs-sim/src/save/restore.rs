//! Putting a document back into a world.
//!
//! Raise first, then adopt. The caller has already run `tower::raise`, so the
//! tower exists before a word of the save is applied, and this pass reconciles:
//! a node the save knows is found by path and given its components, one the
//! tower lacks is spawned under its parent, and a *stock* pile the tower raised
//! that the save never heard of is despawned as a starting reagent the player
//! used up. Letting the save own the whole tree is simpler and wrong on
//! schedule — the domains that land later would be permanently missing from an
//! old save, and refusing it by format version deletes it.
//!
//! Nothing here draws from a random stream. `Ward::from_save` and
//! `Maze::from_save` read their state rather than rolling it, and this pass
//! calls no generator: a restore that re-dug the maze would move
//! `RngStream::Archive` and diverge the loaded world from the one that saved
//! it, on the same seed, for a reason nothing on screen could explain.
//!
//! This owns the world's *shape* — the clock, every stream position, the tree,
//! and the resources. What is true of each *node* is [`super::adopt`], which
//! needs the tree to already exist.
//!
//! §15 accepts hand-editing — *"hand-editing a TOML file only affects the
//! person doing it"* — so every lookup here is fallible and nothing panics. A
//! path that names nothing is skipped, a word that names no verb is dropped,
//! and a missing `Cwd` falls back to `/tower`, because a player standing
//! nowhere is a game with no prompt.

use bevy_ecs::prelude::*;
use orbs_render::Records;

use super::adopt;
use super::document::Save;
use super::naming;
use super::node::NodeSave;
use crate::content::{Charms, Progression, Recipes};
use crate::rng::{RngStream, Rngs};
use crate::session::{Scrollback, Wizard};
use crate::tick::Tick;
use crate::tower::{self, Cwd, Marks, Stock};

/// Apply a save over a freshly raised tower.
///
/// The order is load-bearing in one place: [`adopt::references`] runs after
/// [`stream`], because a spell whose text has moved under it says so in a
/// record, and a record pushed before the stream is rebuilt is thrown away by
/// the rebuild.
pub(crate) fn restore(world: &mut World, save: &Save) {
    clock_and_rolls(world, save);
    tree(world, save);
    progress(world, save);
    stream(world, save);
    adopt::references(world, save);
    prompt(world, save);
    // What both tracks have already passed is opened again, silently. Only the
    // *reached* half is written down, so a document from before a station
    // carried its `opens` comes back with the thing shut and nothing to open it
    // — both tracks apply an `opens` exactly once. Silent for
    // `Experience::restore`'s reason, except what a mastery station reached only
    // by this load opens, which the player has never been told
    // (`mastery::caught_up`).
    tower::ley::caught_up(world);
    tower::mastery::caught_up(world);
    // After the tree, the progress and the catching up: a shut room's markers
    // derive from all three — which nodes exist, which rooms the document calls
    // open, and which the line has since opened.
    world.insert_resource(tower::Sealing(save.world.sealed));
    tower::seal(world);
    // A taken node above its fork's total is kept. When a node moves to a later
    // station — `cursors_1` went from 40 to 400 with the lanes (§19) — an older
    // document holds it below the new total, and so does a tester's
    // `debug_take`. Both are consistent, because `ley_line` derives *spent* by
    // membership: nothing the player earned is taken away, and a tester's
    // shortcut survives the round-trip `tests/strands.rs` holds it to.
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

    // Anything raised that the save does not name and the player could have
    // destroyed. Stock and spells are the two kinds a command can remove —
    // without this, `purge` on a shipped `first_light.spell` is undone by the
    // next load.
    //
    // `Protected` and `Fixture` are the exemption, and are what makes adopting
    // worth doing: a room or instrument the save never heard of is a domain a
    // later phase added, and despawning it would undo the upgrade the player
    // just bought. The cost: a *destructible* thing shipped in a later build is
    // swept the first time an older save is opened.
    //
    // Collected first and re-checked inside the loop — `Children` is
    // `linked_spawn` in bevy_ecs 0.19, so despawning a parent takes its
    // descendants and a later entry can already be gone.
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

    // Wound past every id the save carried, or the next node the tower issues
    // collides with one just restored — two nodes with one identity, and
    // `spell::advance`'s ordering key stops being an ordering.
    let highest = save.nodes.iter().map(|node| node.id).max();
    if let Some(highest) = highest {
        world.resource_mut::<tower::NodeIds>().wind_past(highest);
    }

    // `Children` order is rebuilt to the document's, and it is not cosmetic: §6
    // resolves noun ties to whichever was registered first, so anything a player
    // sees comes from walking `Children` in insertion order.
    //
    // A renamed node is what breaks it. `sabotage::substitute` renames a pile in
    // place — `sage` becomes `sage-` — so its saved path matches nothing raised,
    // it is spawned and appended, and from then on an ambiguous phrase resolves
    // to a different noun than it did before the save. Re-inserting `ChildOf` in
    // document order costs one hook per node at ~50 nodes and settles every
    // cause, not only this one.
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
    // Silent, like every other restore here: `earn` says a sentence, and a load
    // must not congratulate the player on yesterday's work.
    world
        .resource_mut::<tower::Renown>()
        .restore(progress.renown);
    // The allowance a player has already paid for, kept across a save: renown
    // was spent on it, and losing it to a quit would be losing the renown.
    world
        .resource_mut::<tower::siege::Petitioned>()
        .restore(progress.petitioned);
    // Absent means whole, not nothing: a save written before the sanctum says
    // nothing about integrity, and nought would hand every returning player a
    // tower worn to the ground (`ProgressSave::integrity`).
    world
        .resource_mut::<tower::Integrity>()
        .restore(progress.integrity.unwrap_or(tower::STANDING));
    // Before the ceiling is read, because the ceiling depends on it: `pool_<n>`
    // and `floor_<n>` sit on it, and `grant::tiers` answers nought for a
    // resource not there yet rather than panicking — so a ceiling read first
    // came back as if the orb had taken nothing. The reader is deliberately
    // forgiving, which makes the order the thing that has to be right.
    world
        .resource_mut::<tower::Taken>()
        .restore(progress.taken.clone());
    // Absent means the ceiling, for integrity's reason: a save from before the
    // pool came up says nothing about it, and nought would hand a returning
    // player an inert forge and a siege that cannot pledge. Read after
    // integrity, because the ceiling is a function of it.
    //
    // Restored faithfully, never clamped. The ceiling caps *regeneration* and
    // nothing else, so a pool above it is a legal state a live world reaches by
    // wearing down while full — clamping made a round-trip lose a point, caught
    // by `a_loaded_tower_keeps_running_the_same_world`, which is why that test
    // drives a *lived* world rather than a fresh one.
    let ceiling = tower::ceiling(world);
    world.insert_resource(tower::Quintessence::new(
        progress.quintessence.unwrap_or(ceiling),
    ));
    // An absent row is *nothing is cooling*, which is what a save written before
    // §8.1's rationing existed honestly says — `Cooling::from_save` takes a
    // short list for that reason.
    world.insert_resource(tower::Cooling::from_save(&progress.cooling));
    world
        .resource_mut::<tower::Learned>()
        .restore(progress.learned.clone(), progress.fruitless);
    world
        .resource_mut::<tower::Tally>()
        .restore(progress.tally.clone());
    // Absent means full, not empty. A store's standing is a *rate*, so an empty
    // map reads as *nothing made lately* — every store out, and a returning
    // player unable to spend from a shelf they filled. A save written before
    // stores existed is stamped at the tick it loads into instead; `integrity`
    // one field up is the precedent.
    match progress.stores.clone() {
        // A document this build wrote. Taken at face value, empty or not — a
        // tower that has made nothing lately is a real state and must survive a
        // reload unchanged, or the round trip is not idempotent.
        Some(stores) => world.resource_mut::<tower::Stores>().restore(stores),
        // Written before stores existed — stamped now rather than read as out.
        None => {
            let now = world.resource::<crate::tick::Tick>().get();
            // Stock only: `keeping` returns every child of the arsenal,
            // `arsenal.log` included, and stamping a log put a store on a file.
            let kept: Vec<String> = tower::keeping(world)
                .into_iter()
                .filter(|node| world.get::<tower::Stock>(*node).is_some())
                .filter_map(|node| world.get::<tower::Name>(node).map(|name| name.0.clone()))
                .collect();
            for named in kept {
                for _ in 0..tower::FRESH_AT {
                    world.resource_mut::<tower::Stores>().made(&named, now);
                }
            }
        }
    }
    world
        .resource_mut::<tower::mastery::Reached>()
        .restore(progress.reached.clone());
    // Absent means everything open, for `integrity`'s reason: a save from before
    // anything could be shut was a tower standing in all seven rooms, and an
    // empty set would seal six of them on load. `Sim::restored` builds the world
    // open already, so only a document that says otherwise changes it.
    //
    // Unioned with what no station opens, never simply replaced. That set is
    // derived from this build's content (`Opened::start`), so a charm, a gated
    // product or a room this build ships *ungated* is in it and an older
    // document cannot name it — replacing outright shut it for the life of every
    // existing save, which is the authoritative-save failure this header warns
    // of, arriving through capabilities instead of nodes.
    if let Some(opened) = &progress.opened {
        let ungated = tower::Opened::start(
            world.resource::<Recipes>(),
            world.resource::<Charms>(),
            world.resource::<Progression>(),
        );
        let keys: Vec<String> = opened
            .iter()
            .cloned()
            .chain(ungated.keys().map(ToOwned::to_owned))
            .collect();
        world.resource_mut::<tower::Opened>().restore(keys);
    }
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
        // Per record, not inherited from the stream. `set_register` decides what
        // *new* records are spoken in; a saved one carries its own, and §8.1's
        // poisoned-log tell is exactly that — `emit_lines` stamps
        // `Presentation::Tampered` on every third row, so dropping this answered
        // `verify` with *tampered* and drew the log with a plain face.
        //
        // Safe beside `spoken`: `finish` asserts an eldritch record carries an
        // authored linear variant, and the same assertion fired at capture.
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
/// Its own step, after the scene is rebuilt: `analyse` takes the `Scene`
/// *resource*, and at the top of a restore that is still the empty default
/// (`tower::raise` builds none), so re-asking there resolved against a world
/// that named nothing.
///
/// (`spell::compile` is unaffected — `tower::scene_at` computes a scene from
/// the world rather than reading the resource.)
fn prompt(world: &mut World, save: &Save) {
    tower::rebuild(world);

    // The readings are not in the document — an `Intent` would pin the format
    // against every parser change — so the line is re-analysed here. §19 settled
    // that the parser's tie-break draws no randomness, which is what reproduces
    // the same list rather than a similar one. A reading the restored scene can
    // no longer offer is one the player could not have acted on either, so a
    // shorter list is the honest answer.
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
