//! Getting around, and looking at what is there.
//!
//! §7: *"The directory tree **is** the tower. Navigation is diegetic; paths are
//! places."* [`attend`] is also what makes a domain's contents nameable — the
//! scene holds every place but only the belongings of where you stand — so this
//! is the module every other verb depends on to have somewhere to act.
//!
//! **No prose here.** Rule 6 and §12 put authored text in content files; these
//! emit facts and let a later layer wrap sentences around them.

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
/// a brewing command needs. See [`tower::rebuild`](crate::tower::rebuild).
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

    // **A compass bearing is not a room.** The archive's four ways have to be
    // `NounKind::Place` — that is the only kind the place half of a spell's
    // question resolves against, so `if north has passage` cannot be written
    // otherwise — and being places made them somewhere you could stand. §19
    // names walking into one as the sign the maze had become a second spatial
    // system, which §7's filesystem already is.
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

    // **Walking in is what clears the rail's mark**, and it is the only thing
    // that does. §9's minimised half exists to say *something happened over
    // there*; once you are standing in the room, the room itself is saying it,
    // and a marker that outlived the visit would be a light nobody could turn
    // off. A timer was the alternative and is worse — it would clear while the
    // player was making tea, which is the case idle play is made of.
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
/// `Verb::Survey` takes an **optional** place, and dropping it made the echo and
/// the listing disagree: `survey archive` restated `/tower/archive` and then
/// showed the laboratory. §6 makes the echo the thing players learn the vocabulary
/// from, so an echo that describes a different command than the one that ran is
/// worse than no echo at all.
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

    // **A satchel answers with its queue, in queue order.** Everything below
    // sorts by kind and then by name, which is right for a room and wrong for
    // this: a queue's whole content is *which one is next*, and alphabetising it
    // would be presenting the one fact it holds in an order that destroys it.
    // The names are not children either — see `tower::satchel` for why — so the
    // walk below would find nothing and report an empty shelf.
    //
    // **Empty falls through** rather than answering here, so `the satchel holds
    // nothing` comes from the same `survey_bare` line every other bare place
    // gets. One sentence, one place.
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

    let mut here: Vec<(String, &'static str, Option<String>)> = tower::children_of(world, at)
        .into_iter()
        .filter_map(|node| {
            let name = world.get::<tower::Name>(node)?.0.clone();
            let kind = world.get::<tower::Nameable>(node)?.0;
            // Only stock is counted. A place, a file and a spell are each one
            // thing that is either there or not, and `x1` beside every row is a
            // column of noise.
            let stock = world.get::<tower::Stock>(node).map(|stock| stock.label());
            Some((name, kind.label(), stock))
        })
        .collect();

    // **Grouped by kind, then by name.** A room's contents arrive in whatever
    // order the tree was walked, which put the two files either side of the
    // instruments and made the listing something you read rather than scanned.
    // Sorted here rather than in the frontend because §14's linear stream is the
    // same sequence — a screen-reader user gets the grouping too, and a view
    // that re-ordered would be carrying an ordering the record stream does not.
    here.sort_by(|left, right| left.1.cmp(right.1).then_with(|| left.0.cmp(&right.0)));

    // **An empty place still answers.** Every record below is pushed *inside*
    // the loop, so a place with no children said nothing at all — the orb
    // meeting a typed command with total silence, which §6 does not allow
    // anywhere. It went unnoticed for four phases because the rooms that were
    // built are never empty; the sanctum made it the common case, since a
    // station publishes `potency` only while it holds a ward and two of the
    // three are bare for most of a solve. `survey barrier` is close to the
    // first thing anybody types in that room.
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
    // **A heading per kind, and the kind off every row.** `sage reagent` said
    // the word once per line for no gain, and that repetition is what made a
    // listing read as a wall rather than as a table. The rows carry a name and,
    // where there is one, an amount — which is what lets the view line the two
    // up in columns (`Tiling`).
    let mut section = "";
    for (name, kind, stock) in here {
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
        record.finish();
    }
}

/// Where the tree begins, which is not where the player stands.
pub(super) fn root(world: &World) -> Entity {
    tower::root(world)
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
/// # Nameable and findable are the same rule, and must not live apart
///
/// `tower::scene` registers every `.spell` from the whole tree, so a spell can be
/// *named* from anywhere — that is what keeps `invoke` usable outside the one
/// room with no laboratory in it. Nothing was doing the matching half, and the
/// failure was silent in the worst way: from the laboratory,
/// `peruse first_light.spell` resolved at **`Clear`** confidence, found no node,
/// fell through to the record-stream reader, and reported `peruse 0` — a
/// zero-line read of a file with three lines in it.
///
/// That is the symptom [`files`](super::files) already documents as the reason
/// resolution goes by kind, arriving from the other direction. The lookup is
/// global because the scene is global; change one and the other has to move.
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
