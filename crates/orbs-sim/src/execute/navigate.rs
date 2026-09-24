//! Getting around, and looking at what is there.
//!
//! §7: *"The directory tree **is** the tower. Navigation is diegetic; paths are
//! places."* [`attend`] is also what makes a domain's contents nameable — the
//! scene holds every place but only the belongings of where you stand — so this
//! is the module every other verb depends on to have somewhere to act.
//!
//! No prose here: rule 6 and §12 put authored text in content files, so these
//! emit facts and a later layer wraps sentences around them.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::parser::{Intent, NounKind, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Cwd};

use super::{acknowledge, missing};

/// Go somewhere.
///
/// §7: *"Navigation is diegetic; paths are places."* Moving is also what makes a
/// domain's contents nameable at all — the scene holds every place but only the
/// belongings of where you stand, so `attend` is how a player reaches the nouns
/// a brewing command needs. See [`tower::rebuild`].
pub(super) fn attend(intent: &Intent, world: &mut World) {
    let Some(target) = intent.arguments.first().map(|argument| &argument.value) else {
        acknowledge(Verb::Attend, world);
        return;
    };

    let root = root(world);
    let Some(node) = find_place(world, root, target) else {
        missing(Verb::Attend, target, world);
        return;
    };

    // A room the tower has not opened yet refuses in voice (§11.5). Asked of the
    // node, not the name: `attend stacks` reaches the archive by a name that is
    // not the archive's.
    if let Some(room) = tower::sealed_room_of(world, node) {
        refuse_sealed(world, Verb::Attend, &room);
        return;
    }

    // A compass bearing is not a room. The archive's four ways have to be
    // `NounKind::Place` — the only kind a spell's place question resolves
    // against, so `if north has passage` cannot be written otherwise — which
    // also made them somewhere you could stand (§19).
    if world.get::<tower::Reading>(node).is_some() {
        let message = world
            .resource::<crate::content::Prose>()
            .line("attend_reading", &[]);
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, Verb::Attend.canonical())
            .text(FieldName::Message, &message)
            .role(Role::Cost)
            .finish();
        return;
    }

    world.insert_resource(Cwd(node));

    // Walking in is the only thing that clears the rail's mark. §9's minimised
    // half says *something happened over there*; once you are in the room the
    // room says it, and a marker that outlived the visit is a light nobody can
    // turn off. A timer would clear while the player was making tea.
    if let Some(name) = world.get::<tower::Name>(node).map(|name| name.0.clone()) {
        tower::clear_mark(world, &name);
    }

    let path = tower::path_of(world, node);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Attend.canonical())
        .text(FieldName::Path, &path)
        .role(Role::Success)
        .finish();
}

/// List what is at a place, or here.
///
/// `Verb::Survey` takes an optional place, and dropping it made the echo and the
/// listing disagree: `survey archive` restated `/tower/archive` and showed the
/// laboratory. §6 makes the echo how players learn the vocabulary, so an echo
/// describing a different command is worse than no echo.
pub(super) fn survey(intent: &Intent, world: &mut World) {
    let at = match intent.arguments.first() {
        Some(argument) => {
            let root = root(world);
            match find_place(world, root, &argument.value) {
                Some(node) => node,
                None => {
                    missing(Verb::Survey, &argument.value, world);
                    return;
                }
            }
        }
        None => world.resource::<Cwd>().0,
    };
    // The same gate `attend` asks, so a room you cannot enter is not one you
    // can look into from the doorway either.
    if let Some(room) = tower::sealed_room_of(world, at) {
        refuse_sealed(world, Verb::Survey, &room);
        return;
    }

    // A satchel answers with its queue, in queue order. Everything below sorts
    // by kind then name, which destroys the one fact a queue holds: which is
    // next. The names are not children either (see `tower::satchel`), so the
    // walk below would report an empty shelf.
    //
    // Empty falls through rather than answering here, so `the satchel holds
    // nothing` comes from the same `survey_bare` line every bare place gets.
    if let Some(satchel) = world.get::<tower::Satchel>(at)
        && !satchel.is_empty()
    {
        let queued: Vec<String> = satchel.names().map(str::to_owned).collect();
        let mut scrollback = world.resource_mut::<Scrollback>();
        let records = scrollback.records_mut();
        records
            .push(RecordKind::Section)
            .text(FieldName::Kind, tower::satchel::QUEUED)
            .finish();
        for name in queued {
            records
                .push(RecordKind::Entry)
                .text(FieldName::Name, &name)
                .finish();
        }
        return;
    }

    // How well stocked the tower is, but only where the question means anything
    // (§19). Standing is about what the arsenal can still supply to a wall, so a
    // dispensary shelf of sage gets no word rather than one reading as a warning.
    let keep = tower::keep(world) == Some(at);
    let mut here: Vec<(String, &'static str, Option<String>, Option<&'static str>)> =
        tower::children_of(world, at)
            .into_iter()
            // A shut room is not listed, so the tower says *there is a
            // laboratory* and nothing about what a fresh player has not earned.
            // The rail's dark boxes make the same choice.
            .filter(|node| tower::sealed_room_of(world, *node).is_none())
            .filter_map(|node| {
                let name = world.get::<tower::Name>(node)?.0.clone();
                let kind = world.get::<tower::Nameable>(node)?.0;
                // Only stock is counted. A place, a file and a spell are each one
                // thing that is either there or not, and `x1` beside every row is a
                // column of noise.
                let stock = world.get::<tower::Stock>(node).map(|stock| stock.label());
                // Only what is stocked, and `arsenal.log` is why: asking a file
                // how well stocked the tower is in it printed `arsenal.log
                // spent`, a warning about a thing that cannot run out.
                // `stocktake` and the save migration filter the same way.
                let supply =
                    (keep && stock.is_some()).then(|| tower::supply_of(world, &name).word());
                Some((name, kind.label(), stock, supply))
            })
            .collect();

    // Grouped by kind, then by name: a tree walk put the two files either side
    // of the instruments, so the listing was something you read rather than
    // scanned. Sorted here rather than in the frontend because §14's linear
    // stream is the same sequence, and a view that re-ordered would carry an
    // ordering the record stream does not.
    here.sort_by(|left, right| left.1.cmp(right.1).then_with(|| left.0.cmp(&right.0)));

    // An empty place still answers. Every record below is pushed inside the
    // loop, so a childless place said nothing at all — §6 allows silence
    // nowhere. Unnoticed for four phases because the built rooms are never
    // empty; the sanctum made it the common case, since two of three stations
    // are bare for most of a solve.
    //
    // Said here rather than per-domain because the hole is `survey`'s: the
    // lens's untouched socket and a scoured instrument are the same shape.
    if here.is_empty() {
        let name = world
            .get::<tower::Name>(at)
            .map_or_else(String::new, |name| name.0.clone());
        let message = world
            .resource::<crate::content::Prose>()
            .line("survey_bare", &[("name", &name)]);
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, Verb::Survey.canonical())
            .text(FieldName::Message, &message)
            .finish();
        return;
    }

    let mut scrollback = world.resource_mut::<Scrollback>();
    let records = scrollback.records_mut();
    // A heading per kind, and the kind off every row: `sage reagent` said the
    // word once per line for no gain, and that repetition made a listing read as
    // a wall rather than a table. A row carries a name and, where there is one,
    // an amount — which is what lets the view line them up (`Tiling`).
    let mut section = "";
    for (name, kind, stock, supply) in here {
        if kind != section {
            section = kind;
            records
                .push(RecordKind::Section)
                .text(FieldName::Kind, kind)
                .finish();
        }
        let mut record = records.push(RecordKind::Entry).text(FieldName::Name, &name);
        if let Some(stock) = stock {
            record = record.text(FieldName::Quantity, &stock);
        }
        // A third column, arsenal only. `State` is what every other surface says
        // a derived condition with, so a spell and a screen reader meet this the
        // way they meet `ready` and `fouled`.
        if let Some(supply) = supply {
            record = record.text(FieldName::State, supply);
        }
        record.finish();
    }
}

/// Where the tree begins, which is not where the player stands.
pub(super) fn root(world: &World) -> Entity {
    tower::root(world)
}

/// Say that `room` is not the player's yet.
///
/// One sentence for every verb that reaches a shut room, so the refusal is the
/// same whichever door was tried — and it names the room, because a player who
/// typed `attend stacks` deserves to learn what the stacks are in.
fn refuse_sealed(world: &mut World, verb: Verb, room: &str) {
    let message = world
        .resource::<crate::content::Prose>()
        .line("room_sealed", &[("name", room)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Message, &message)
        .role(Role::Cost)
        .finish();
}

/// The place `target` names, by full path or by last segment (§7).
pub(super) fn find_place(world: &World, from: Entity, target: &str) -> Option<Entity> {
    tower::reach::look(world)
        .scope(tower::reach::Scope::Under(from))
        .kind(NounKind::Place)
        // §7: a place answers to its own name and to its whole path, because a
        // `Place` argument resolves to the path and the node carries the leaf.
        .naming(tower::reach::Naming::LeafOrPath)
        .find(target)
}

/// The place `named`, searched from the top of the tree.
///
/// The one lookup a spell's `Domain` needs, exported because the runner lives in
/// `tower::spell` and this is `execute`'s answer to "which place is that". A
/// second copy would be a second answer to a question `find_place` already
/// settles — including that a place answers to its full path *or* its leaf.
pub fn find_domain(world: &World, named: &str) -> Option<Entity> {
    find_place(world, tower::root(world), named)
}

/// The spell `target` names, wherever it is kept.
///
/// Nameable and findable are one rule. `tower::scene` registers every `.spell`
/// from the whole tree so a spell can be named from anywhere; nothing was doing
/// the matching half, and `peruse first_light.spell` resolved at `Clear`, found
/// no node, fell through to the record reader and reported `peruse 0`. The
/// lookup is global because the scene is global — change one and the other moves.
///
/// The extension is optional, because [`with_extension`](crate::content::with_extension)
/// makes `first_light` and `first_light.spell` the same spell everywhere else.
pub(super) fn find_script(world: &World, target: &str) -> Option<Entity> {
    tower::reach::look(world)
        .scope(tower::reach::Scope::Tower)
        .kind(NounKind::Script)
        .naming(tower::reach::Naming::Script)
        .find(target)
}
