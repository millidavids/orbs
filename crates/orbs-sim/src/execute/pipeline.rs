//! §10.1's brewing loop: clear, charge, wield, draw off.
//!
//! `move`, `wield`, the instruments' own verbs, `empty`, `stop` and `purge`.
//! Everything here is scoped to where the player stands — see [`instrument`],
//! the one lookup the pipeline uses, and why `purge` reaching across the tower
//! was a defect.
//!
//! No prose: rule 6 and §12 put authored text in content files; these emit facts
//! for a later layer to wrap sentences around.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Prose, Recipes};
use crate::parser::{Intent, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Cwd};

use super::navigate::{find_place, root};
use super::{acknowledge, missing};

/// A place inside the current one, by name — §10.1's instruments.
///
/// Compared by leaf, since a `Place` slot resolves to the full path (§7) and a
/// node carries only its leaf.
fn instrument(world: &World, path: &str) -> Option<(Entity, String)> {
    let leaf = crate::parser::leaf(path).to_owned();
    let cwd = world.resource::<Cwd>().0;
    let at = tower::children_of(world, cwd).into_iter().find(|node| {
        world.get::<tower::Fixture>(*node).is_some()
            && world
                .get::<tower::Name>(*node)
                .is_some_and(|held| held.0 == leaf)
    })?;
    // The leaf goes back out: recipes are keyed by instrument name, and prose
    // saying "the /tower/laboratory/mortar_and_pestle" would read a path aloud.
    Some((at, leaf))
}

/// Everywhere a component can be taken from, in the order it is looked for.
///
/// Instruments first, in raise order, since mid-pipeline the thing moved is
/// still in the tool that made it (§19); busy ones skipped (§10.1). Then the
/// store, then the arsenal — nameable from every room, so findable from every
/// room, and last so a reagent here outranks a carried one.
///
/// The order lives in `tower::reach`, so a spell at cast and a verb at
/// execution agree.
fn reachable(world: &World, cwd: Entity) -> Vec<Entity> {
    tower::reach::look(world)
        .scope(tower::reach::Scope::Fetch(cwd))
        .candidates()
}

/// Somewhere a `move` can name: an instrument here, or the arsenal.
///
/// The arsenal is the one place reachable from anywhere, and takes finished
/// work only (`tower::keep`). Tried after the instruments, so a fixture called
/// `arsenal` still means its own. Both ends of a `move` ask this.
fn addressed(world: &World, path: &str) -> Option<(Entity, String)> {
    instrument(world, path).or_else(|| {
        let leaf = crate::parser::leaf(path);
        let keep = tower::keep(world)?;
        (world
            .get::<tower::Name>(keep)
            .is_some_and(|name| name.0 == leaf))
        .then(|| (keep, leaf.to_owned()))
    })
}

/// Carry a reagent from one place to another (§10.1).
///
/// The source slot is optional and disambiguating: with one lot of `sage` in the
/// room, `move sage to mortar_and_pestle` is unambiguous; with `husks` in two
/// instruments, naming the source is what tells them apart.
pub(super) fn carry(intent: &Intent, world: &mut World) {
    // By slot, not by arity: `Filled::slots` is positional so inference is
    // never needed, whatever `Intent::arguments` compacting the `None`s away
    // suggests.
    let Some(thing) = intent.slot(0).map(|argument| argument.value.clone()) else {
        acknowledge(Verb::Move, world);
        return;
    };
    let source = intent.slot(1).map(|argument| argument.value.clone());
    let Some(destination) = intent.slot(2).map(|argument| argument.value.clone()) else {
        acknowledge(Verb::Move, world);
        return;
    };

    let Some((to, destination)) = addressed(world, &destination) else {
        missing(Verb::Move, &destination, world);
        return;
    };

    // An instrument mid-something will not be charged. §10.1's lock covers a
    // scour as well as a run: charging four ticks before a purge empties it
    // destroys the reagent.
    if let Some(why) = tower::busy(world, to) {
        tower::refuse_busy(world, Verb::Move, to, why);
        return;
    }

    // Where to look. A named source is that place and nothing else; with none
    // named, §10.1's own order — see `reachable`.
    let cwd = world.resource::<Cwd>().0;
    let haystack: Vec<Entity> = match &source {
        Some(source) => match addressed(world, source) {
            Some((from, _)) => {
                // §10.1's lock covers taking as much as putting: an instrument
                // mid-something will not be raided. Naming one refuses outright.
                if let Some(why) = tower::busy(world, from) {
                    tower::refuse_busy(world, Verb::Move, from, why);
                    return;
                }
                tower::children_of(world, from)
            }
            None => {
                missing(Verb::Move, source, world);
                return;
            }
        },
        // Never the destination itself: `reachable` walks it too, so `move
        // charcoal to athanor` found the athanor's own charcoal and reported
        // moving it to itself.
        None => reachable(world, cwd)
            .into_iter()
            .filter(|node| world.get::<ChildOf>(*node).map(ChildOf::parent) != Some(to))
            .collect(),
    };

    let Some(node) = haystack.into_iter().find(|node| {
        world
            .get::<tower::Name>(*node)
            .is_some_and(|held| held.0 == thing)
            && world.get::<tower::Fixture>(*node).is_none()
    }) else {
        missing(Verb::Move, &thing, world);
        return;
    };

    // The arsenal's door, checked before anything moves: finished work only
    // (`tower::admits`), and a half-done `move` leaves the player guessing.
    if world.get::<tower::Keep>(to).is_some() && !tower::admits(world, node) {
        let message = world
            .resource::<Prose>()
            .line("move_unkept", &[("name", &thing), ("path", &destination)]);
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Message)
            .text(FieldName::Name, &thing)
            .text(FieldName::Path, &destination)
            .text(FieldName::Message, &message)
            .role(Role::Cost)
            .finish();
        return;
    }

    // Where it actually came from, which is the half the player could not see.
    // Named *before* the move, because after it the parent is the destination.
    let origin = world
        .get::<ChildOf>(node)
        .map(ChildOf::parent)
        .and_then(|parent| world.get::<tower::Name>(parent))
        .map_or_else(String::new, |name| name.0.clone());

    let units = carried(world, node, to);
    hand(world, node, to, units);

    let key = if units > 1 {
        "move_done_all"
    } else {
        "move_done"
    };
    let message = world.resource::<Prose>().line(
        key,
        &[
            ("name", &thing),
            ("origin", &origin),
            ("path", &destination),
            ("count", &units.to_string()),
        ],
    );
    let mut scrollback = world.resource_mut::<Scrollback>();
    scrollback
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, &thing)
        // `Origin`, not `Source`: `read_file` keys domain logs by `Source`, so
        // a move carrying `Source = "dispensary"` would claim to happen there.
        .text(FieldName::Origin, &origin)
        .text(FieldName::Path, &destination)
        .count(FieldName::Quantity, u64::from(units))
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
}

/// Set an instrument working on what is in it (§10.1).
pub(super) fn wield(intent: &Intent, world: &mut World) {
    // A siege first (§19): `wield quickening-scroll` in the bailey used to
    // hurry the laboratory. Only while a siege runs and only for a scroll the
    // wall can use, so `wield mortar_and_pestle` is untouched.
    if super::defend::wielded(intent, world) {
        return;
    }

    // A scroll next, before `start`: spending one takes no production slot, so
    // it is not refused mid-brew, which is when a player reaches for one.
    // `begins_work` is `const fn(Verb)` and cannot see the argument.
    if super::scroll::spend(intent, world) {
        return;
    }

    let Some(name) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Wield, world);
        return;
    };

    let Some((at, name)) = instrument(world, &name) else {
        missing(Verb::Wield, &name, world);
        return;
    };
    start(world, at, &name, Verb::Wield);
}

/// Charge an instrument and start it, in one command (§10.1).
///
/// `grind sage` is `move` then `wield`. The fixture is found by its declared
/// [`Operation`](crate::tower::Operation), and reagents by the [`reachable`] a
/// bare `move` uses, so both agree on which sage.
pub(super) fn operate(intent: &Intent, world: &mut World) {
    let verb = intent.verb;
    let cwd = world.resource::<Cwd>().0;

    let Some(at) = tower::children_of(world, cwd).into_iter().find(|node| {
        world
            .get::<tower::Operation>(*node)
            .is_some_and(|operation| operation.0 == verb)
    }) else {
        // No instrument here does this. Naming the verb rather than a tool is
        // the honest report — the player did not name one.
        missing(verb, verb.canonical(), world);
        return;
    };
    let name = world
        .get::<tower::Name>(at)
        .map_or_else(String::new, |name| name.0.clone());

    // §10.1's lock, before anything moves.
    if let Some(why) = tower::busy(world, at) {
        tower::refuse_busy(world, verb, at, why);
        return;
    }

    // Every reagent is found before any of them moves: a half-charged instrument
    // leaves the player working out what went in and taking it back out.
    let mut carrying = Vec::new();
    for argument in &intent.arguments {
        let haystack = reachable(world, cwd);
        let Some(node) = haystack.into_iter().find(|node| {
            !carrying.contains(node)
                && world
                    .get::<tower::Name>(*node)
                    .is_some_and(|held| held.0 == argument.value)
                && world.get::<tower::Fixture>(*node).is_none()
        }) else {
            missing(verb, &argument.value, world);
            return;
        };
        carrying.push(node);
    }

    // Each charge still reports what moved and from where (§19), which is also
    // the only thing telling the player which sage the search order picked.
    for node in carrying {
        charge(world, node, at, &name);
    }

    start(world, at, &name, verb);
}

/// Move `units` of whatever `node` is into `to` if it is stock, the whole thing
/// if it is not.
///
/// An endless pile yields units without shrinking; a vessel, a file and a
/// spell simply move. One function for `move` and charging, or an endless pile
/// gets re-parented and consumed.
fn hand(world: &mut World, node: Entity, to: Entity, units: u32) {
    let Some(thing) = world.get::<tower::Name>(node).map(|name| name.0.clone()) else {
        return;
    };
    // A moved thing is no longer a product, or the destination reads `ready`
    // before anything was wielded there. `give` spawns without the marker.
    let Some(&kind) = world.get::<tower::Nameable>(node).map(|kind| &kind.0) else {
        return;
    };
    if world.get::<tower::Stock>(node).is_none() {
        world.entity_mut(node).insert(ChildOf(to));
        world.entity_mut(node).remove::<tower::Product>();
        return;
    }
    if let Some(from) = world.get::<ChildOf>(node).map(ChildOf::parent) {
        tower::take(world, from, &thing, units);
    }
    tower::give(world, to, &thing, kind, units);
}

/// How much of `node` a `move` to `to` carries.
///
/// The whole batch onto a shelf, the arsenal or a store, since a supply opens
/// on how much the arsenal holds (§19, *Supply*). One unit into an instrument,
/// which a run spends; one from an endless pile.
fn carried(world: &World, node: Entity, to: Entity) -> u32 {
    let shelf = world.get::<tower::Keep>(to).is_some() || world.get::<tower::Store>(to).is_some();
    match world.get::<tower::Stock>(node) {
        Some(tower::Stock::Counted(count)) if shelf => *count,
        _ => 1,
    }
}

/// Move one reagent into `at`, saying where it came from.
fn charge(world: &mut World, node: Entity, at: Entity, destination: &str) {
    let thing = world
        .get::<tower::Name>(node)
        .map_or_else(String::new, |name| name.0.clone());
    // Named *before* the move, because after it the parent is the destination.
    let origin = world
        .get::<ChildOf>(node)
        .map(ChildOf::parent)
        .and_then(|parent| world.get::<tower::Name>(parent))
        .map_or_else(String::new, |name| name.0.clone());

    hand(world, node, at, 1);

    let message = world.resource::<Prose>().line(
        "move_done",
        &[("name", &thing), ("origin", &origin), ("path", destination)],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, &thing)
        .text(FieldName::Origin, &origin)
        .text(FieldName::Path, destination)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
}

/// Run whatever the instrument's contents make of it.
///
/// Shared by [`wield`] and [`operate`] so the two cannot disagree: only how the
/// instrument was named differs.
fn start(world: &mut World, at: Entity, name: &str, verb: Verb) {
    // The heat source is not a stage: wielding it means lighting it, and it takes
    // no Focus slot at all (§10.1).
    if world.get::<tower::HeatSource>(at).is_some() {
        tower::kindle(world, at);
        return;
    }

    let holding = tower::holdings(world, at);

    // §10.1: the material's state decides what an instrument can do, so a
    // refusal has to say what is in there.
    let Some((ticks, heat, cost)) = world
        .resource::<Recipes>()
        .matching(name, &holding, &tower::known(world))
        .map(|recipe| (recipe.ticks, recipe.heat, recipe.quintessence))
    else {
        let key = if holding.is_empty() {
            "wield_empty"
        } else {
            "wield_no_recipe"
        };
        let listed = holding
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        refuse(
            world,
            verb,
            name,
            key,
            &[("detail", &listed)],
            Some(&listed),
        );
        return;
    };

    // Checked at the start; the run completes even if the fire dies under it
    // (§10.1). The flag is the recipe's — see `Recipe::heat`.
    if heat {
        let cwd = world.resource::<Cwd>().0;
        let lit = tower::find_athanor(world, cwd).is_some_and(|fire| tower::lit(world, fire));
        if !lit {
            tower::refuse_cold(world, name);
            return;
        }
    }

    let Some(subject) = world.get::<tower::NodeId>(at).copied() else {
        missing(verb, name, world);
        return;
    };
    if let Some((key, fields)) = super::price::refusal(world, cost) {
        let fields: Vec<(&str, &str)> = fields
            .iter()
            .map(|(field, value)| (*field, value.as_str()))
            .collect();
        refuse(world, verb, name, key, &fields, None);
        return;
    }

    // The verb the player used goes on the `Working`, so a refusal says "busy
    // grinding" rather than "busy wielding" — see `Verb::participle`.
    if tower::begin(world, at, verb, subject, ticks) {
        super::price::charge(world, cost);
    }
}

/// Refuse a run, saying why, at `Role::Cost`.
///
/// `state` is a fact a pipe can read — `State`, since `Detail` draws in front
/// of the message.
fn refuse(
    world: &mut World,
    verb: Verb,
    name: &str,
    key: &str,
    fields: &[(&str, &str)],
    state: Option<&str>,
) {
    let mut fields = fields.to_vec();
    fields.push(("name", name));
    let message = world.resource::<Prose>().line(key, &fields);
    let mut scrollback = world.resource_mut::<Scrollback>();
    let mut record = scrollback
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Path, name);
    if let Some(state) = state {
        record = record.text(FieldName::State, state);
    }
    record
        .text(FieldName::Message, &message)
        .role(Role::Cost)
        .finish();
}

/// Turn everything an instrument holds out into the store (§10.1).
///
/// `purge`'s counterpart, which keeps what it clears: every byproduct has a
/// use (§10.1). Instant, like [`carry`] — a scour's ticks are the price of
/// destroying, not of shelving.
pub(super) fn empty(intent: &Intent, world: &mut World) {
    let Some(name) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Empty, world);
        return;
    };
    let Some((at, name)) = instrument(world, &name) else {
        missing(Verb::Empty, &name, world);
        return;
    };

    // §10.1's lock. Turning out an instrument mid-run would take the reagents
    // out from under it, which is what `busy` exists to stop everywhere else.
    if let Some(why) = tower::busy(world, at) {
        tower::refuse_busy(world, Verb::Empty, at, why);
        return;
    }

    let cwd = world.resource::<Cwd>().0;
    let Some(store) = tower::children_of(world, cwd)
        .into_iter()
        .find(|node| world.get::<tower::Store>(*node).is_some())
    else {
        say_empty(world, "empty_nowhere", &name, "", "", Role::Cost);
        return;
    };
    let store_name = world
        .get::<tower::Name>(store)
        .map_or_else(String::new, |name| name.0.clone());

    // Emptying the store into itself is a no-op worth saying out loud rather
    // than performing.
    if store == at {
        say_empty(world, "empty_is_store", &name, &store_name, "", Role::Cost);
        return;
    }

    let held = tower::contents(world, at);
    if held.is_empty() {
        say_empty(world, "empty_already", &name, &store_name, "", Role::Normal);
        return;
    }

    let mut turned_out = Vec::new();
    for node in held {
        let Some(held) = world.get::<tower::Name>(node).map(|name| name.0.clone()) else {
            continue;
        };
        let kind = world
            .get::<tower::Nameable>(node)
            .map_or(crate::parser::NounKind::Reagent, |kind| kind.0);
        turned_out.push(held.clone());

        // Poured onto the pile, not stood beside it: reparenting left two
        // `ground-sage` rows for the parser to choose between arbitrarily.
        match world.get::<tower::Stock>(node).copied() {
            Some(tower::Stock::Counted(count)) => {
                world.entity_mut(node).despawn();
                tower::give(world, store, &held, kind, count);
            }
            // Pouring an endless pile into the store would silently make the
            // store endless. Dropped rather than merged, as a spent unit is.
            Some(tower::Stock::Endless) => {
                world.entity_mut(node).despawn();
            }
            // Not stock: it moves whole, and stops being a product on the way.
            None => {
                world.entity_mut(node).insert(ChildOf(store));
                world.entity_mut(node).remove::<tower::Product>();
            }
        }
    }

    let listed = turned_out.join(", ");
    say_empty(
        world,
        "empty_done",
        &name,
        &store_name,
        &listed,
        Role::Success,
    );
}

/// One record about an emptying, with the sentence from `content/prose.toml`.
///
/// `moved` rides in `State`, so `sift husks orb.log` finds the line.
fn say_empty(world: &mut World, key: &str, name: &str, store: &str, moved: &str, role: Role) {
    let message = world
        .resource::<Prose>()
        .line(key, &[("name", name), ("path", store), ("state", moved)]);
    let mut scrollback = world.resource_mut::<Scrollback>();
    let mut record = scrollback
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Empty.canonical())
        .text(FieldName::Path, name)
        .text(FieldName::Origin, store)
        .text(FieldName::Message, &message);
    if !moved.is_empty() {
        record = record.text(FieldName::State, moved);
    }
    record.role(role).finish();
}

/// Cancel a working instrument (§10.1).
pub(super) fn stop(intent: &Intent, world: &mut World) {
    let Some(name) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Stop, world);
        return;
    };

    // A spell first: `stop` reached instruments only, so an invoked spell was
    // unstoppable. The two can never share a name, so the order is free.
    if tower::spell::stop_spell(world, &name) {
        return;
    }
    // Then a supply, which can never share a name with an instrument either:
    // `stop warding` shuts what the arsenal is drawing (§19, *Supply*).
    if super::defend::stop_supply(world, &name) {
        return;
    }

    match instrument(world, &name) {
        // Stopping the athanor damps the fire and banks what has not burnt,
        // which makes `stop athanor` worth writing at the end of a loop (§10.1).
        Some((at, _)) if world.get::<tower::HeatSource>(at).is_some() => {
            tower::damp(world, at);
        }
        Some((at, _)) => {
            // A puzzle inserts no `Working`, so it is abandoned here (§19) —
            // and then a run too, since one `stop` ends what is happening.
            let abandoned = tower::puzzle::Open::on(world, at)
                .is_some_and(|open| super::settle::abandon(world, open, at));
            // Nothing else to stop unless a run is also under way.
            if abandoned && world.get::<tower::Working>(at).is_none() {
                return;
            }
            tower::stop(world, at);
        }
        None => missing(Verb::Stop, &name, world),
    }
}

/// Destroy something where you stand.
///
/// §7 makes destruction *"useful, everyday, and scriptable"* rather than a
/// trap, and §9's per-pane triage slot is why it still runs during a brew.
pub(super) fn purge(intent: &Intent, world: &mut World) {
    let Some(target) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Purge, world);
        return;
    };

    // By leaf, or every `purge` falls through to the tower-wide walk below and
    // destroys at a distance.
    let leaf = crate::parser::leaf(&target).to_owned();
    let found = tower::reach::look(world)
        .find(&leaf)
        // ...and whatever the arsenal holds, nameable from every room (§7).
        .or_else(|| tower::kept(world, &leaf));

    // Places too, or §7's guard never fires — but from the root only as far as
    // something that refuses.
    let found = found.or_else(|| {
        let root = root(world);
        find_place(world, root, &target)
            .filter(|node| world.get::<tower::Protected>(*node).is_some())
    });

    match found {
        Some(node) => tower::purge(world, node),
        None => missing(Verb::Purge, &target, world),
    }
}
