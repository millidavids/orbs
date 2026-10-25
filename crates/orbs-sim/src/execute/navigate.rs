//! Getting around, and looking at what is there.
//!
//! §7: *"The directory tree **is** the tower."* [`attend`] is also what makes a
//! domain's contents nameable.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::parser::{Intent, NounKind, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Cwd};

use super::{acknowledge, missing};

/// Go somewhere.
///
/// The scene holds every place but only the belongings of where you stand, so
/// this is how a player reaches a command's nouns. See [`tower::rebuild`].
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

    // A compass bearing is not a room, though a way is a `Place` so a spell
    // can ask about it (§19).
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

    // Walking in is the only thing that clears the rail's mark (§9): the room
    // says it now. A timer would clear while the player made tea.
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
/// The place, when named, is honoured, so the echo and the listing agree (§6).
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

    // A satchel answers in queue order, which the sort below would destroy,
    // and its names are not children (`tower::satchel`). Empty falls through
    // to `survey_bare`.
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
                // Only what is stocked, or `arsenal.log` reads as spent — the
                // filter `stocktake` and the save migration use.
                let supply =
                    (keep && stock.is_some()).then(|| tower::freshness_of(world, &name).word());
                Some((name, kind.label(), stock, supply))
            })
            .collect();

    // Grouped by kind, then by name, so it scans. Here, not in the frontend,
    // since §14's linear stream is the same sequence.
    here.sort_by(|left, right| left.1.cmp(right.1).then_with(|| left.0.cmp(&right.0)));

    // An empty place still answers (§6), here rather than per domain, since
    // an untouched socket and a scoured instrument are the same shape.
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
    // A heading per kind, and a row of name and amount, so the view can line
    // them up (`Tiling`).
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
/// One sentence for every door, naming the room the thing is in.
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
/// Exported for `tower::spell`'s `Domain`, so there is one answer to "which
/// place is that", full path or leaf.
pub fn find_domain(world: &World, named: &str) -> Option<Entity> {
    find_place(world, tower::root(world), named)
}

/// The spell `target` names, wherever it is kept.
///
/// Global, because `tower::scene` names every `.spell` from anywhere:
/// nameable and findable are one rule. The extension is optional, as
/// [`with_extension`](crate::content::with_extension) makes it.
pub(super) fn find_script(world: &World, target: &str) -> Option<Entity> {
    tower::reach::look(world)
        .scope(tower::reach::Scope::Tower)
        .kind(NounKind::Script)
        .naming(tower::reach::Naming::Script)
        .find(target)
}
