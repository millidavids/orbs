//! §10.1's brewing loop: clear, charge, wield, draw off.
//!
//! `move`, `wield`, `stop`, `siphon`, `purge` and the `grimoire` that makes the
//! whole thing readable **before** the player commits an instrument to a route.
//! Everything here is scoped to where the player is standing — see
//! [`instrument`], which is the one lookup the pipeline uses and the reason
//! `purge` reaching across the tower was a defect rather than a feature.
//!
//! **No prose here.** Rule 6 and §12 put authored text in content files; these
//! emit facts and let a later layer wrap sentences around them.

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
/// A `Place` slot resolves to the **full path** (`/tower/laboratory/alembic`),
/// because §7 makes places nameable from anywhere and the scene registers them
/// by path. Nodes carry only their leaf, so the leaf is what this compares —
/// matching the whole string silently found nothing and turned every pipeline
/// command into "no such thing".
fn instrument(world: &World, path: &str) -> Option<(Entity, String)> {
    let leaf = crate::parser::leaf(path).to_owned();
    let cwd = world.resource::<Cwd>().0;
    let at = tower::children_of(world, cwd).into_iter().find(|node| {
        world.get::<tower::Fixture>(*node).is_some()
            && world
                .get::<tower::Name>(*node)
                .is_some_and(|held| held.0 == leaf)
    })?;
    // The leaf goes back out because everything downstream wants it: recipes are
    // keyed by instrument name, and prose that said
    // "the /tower/laboratory/mortar_and_pestle" would be reading a path aloud.
    Some((at, leaf))
}

/// Everywhere a component can be taken from, in the order it is looked for.
///
/// **The floor first**, because that is where `siphon` puts a product
/// (`tower::work` moves it to `cwd`) and §10.1's loop makes `siphon` mandatory —
/// so the thing a player is moving mid-pipeline is nearly always lying on the
/// bench, not still inside the instrument that made it.
///
/// **Then the instruments**, in raise order: what is left inside one is the
/// byproduct `siphon` deliberately did not take, or a charge not yet wielded.
/// **Busy instruments are skipped** — §10.1's lock covers taking as much as
/// putting, and without this a `move` could gut a run in flight, spend the Focus
/// slot for nothing and say not a word about it.
///
/// **The store last.** It is stock, and stock is the fallback.
fn reachable(world: &World, cwd: Entity) -> Vec<Entity> {
    let here = tower::children_of(world, cwd);

    // The floor: everything loose in the room.
    let mut order: Vec<Entity> = here
        .iter()
        .copied()
        .filter(|node| world.get::<tower::Fixture>(*node).is_none())
        .collect();

    // One partition rather than two filtered passes over a cached bool: the
    // instruments in raise order, then the stores.
    let (stores, instruments): (Vec<Entity>, Vec<Entity>) = here
        .into_iter()
        .filter(|node| world.get::<tower::Fixture>(*node).is_some())
        .partition(|node| world.get::<tower::Store>(*node).is_some());

    for node in instruments {
        // §10.1's lock covers taking as much as putting, and a scour is about to
        // despawn everything in there.
        if tower::busy(world, node).is_some() {
            continue;
        }
        order.extend(tower::children_of(world, node));
    }
    for node in stores {
        order.extend(tower::children_of(world, node));
    }
    order
}

/// Carry a reagent from one place to another (§10.1).
///
/// The source slot is optional and disambiguating: with one lot of `sage` in the
/// room, `move sage to mortar_and_pestle` is unambiguous; with `husks` in two
/// instruments, naming the source is what tells them apart.
pub(super) fn carry(intent: &Intent, world: &mut World) {
    // **By slot, not by arity.** `move` is `<thing> [from <source>] <to>`, and
    // this used to `match` on the argument slice's *length* — two meant the
    // source was skipped, three meant it was named. `Filled::slots` is positional
    // precisely so that inference is never needed ("a later slot resolving while
    // an earlier one does not cannot silently renumber the arguments"), and
    // `Intent::arguments` compacting the `None`s away is what made it look
    // necessary. It held only because `move` is the one signature with an
    // optional slot, sitting between two required ones.
    let Some(thing) = intent.slot(0).map(|argument| argument.value.clone()) else {
        acknowledge(Verb::Move, world);
        return;
    };
    let source = intent.slot(1).map(|argument| argument.value.clone());
    let Some(destination) = intent.slot(2).map(|argument| argument.value.clone()) else {
        acknowledge(Verb::Move, world);
        return;
    };

    let Some((to, destination)) = instrument(world, &destination) else {
        missing(Verb::Move, &destination, world);
        return;
    };

    // An instrument mid-something will not be charged. §10.1's lock is the whole
    // reason different recipes need different scripts, and it covers a scour as
    // well as a run: charging an instrument four ticks before a purge empties it
    // destroys the reagent.
    if let Some(why) = tower::busy(world, to) {
        tower::refuse_busy(world, Verb::Move, to, why);
        return;
    }

    // Where to look. A named source is that place and nothing else; with none
    // named, §10.1's own order — see `reachable`.
    let cwd = world.resource::<Cwd>().0;
    let haystack: Vec<Entity> = match &source {
        Some(source) => match instrument(world, source) {
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
        None => reachable(world, cwd),
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

    // Where it actually came from, which is the half the player could not see.
    // Named *before* the move, because after it the parent is the destination.
    let origin = world
        .get::<ChildOf>(node)
        .map(ChildOf::parent)
        .and_then(|parent| world.get::<tower::Name>(parent))
        .map_or_else(String::new, |name| name.0.clone());

    world.entity_mut(node).insert(ChildOf(to));
    // **A moved thing is no longer a product.** `siphon` clears this marker when
    // it collects, but a `move` re-parented it intact — so the destination read
    // `ready` on the panel before anything had been wielded, and `siphon` there
    // handed the freshly-delivered *input* straight back out, emptying the
    // charge. It marks "this instrument made this", which stops being true the
    // moment it is carried somewhere else.
    world.entity_mut(node).remove::<tower::Product>();

    let message = world.resource::<Prose>().line(
        "move_done",
        &[
            ("name", &thing),
            ("origin", &origin),
            ("path", &destination),
        ],
    );
    let mut scrollback = world.resource_mut::<Scrollback>();
    scrollback
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, &thing)
        // `Origin`, not `Source`. `read_file` keys **domain logs** by where a
        // record happened, and `Source` is the emitting instrument — a move
        // carrying `Source = "dispensary"` would claim to have happened there.
        .text(FieldName::Origin, &origin)
        .text(FieldName::Path, &destination)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
}

/// Set an instrument working on what is in it (§10.1).
pub(super) fn wield(intent: &Intent, world: &mut World) {
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
/// `grind sage` is `move sage to mortar_and_pestle` and `wield mortar_and_pestle`
/// — the two commands a stage that a player types most. The instrument is not
/// named because the **verb** names it: each one declares its own operation (see
/// [`Operation`](crate::tower::Operation)), so this finds the fixture that does
/// this rather than looking a name up in a table.
///
/// The reagents are found by §10.1's own search order — the floor, then the
/// instruments, then the store — which is the same [`reachable`] a bare `move`
/// uses. That is deliberate: two ways of saying one thing must not disagree
/// about *which* sage they meant.
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

    // **Every reagent is found before any of them moves.** A half-charged
    // instrument left by a command that then refused is worse than the command
    // not running: the player has to work out what went in and take it back out
    // before the loop will start again.
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

    // Each charge still reports **what moved and from where**. §19 settled that
    // for `move` — "the echo should always print what was done, what it was done
    // to, where it came from and where it went to" — and collapsing two commands
    // into one is no reason to stop saying it. It is also the only thing that
    // tells the player *which* sage the search order picked.
    for node in carrying {
        charge(world, node, at, &name);
    }

    start(world, at, &name, verb);
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

    world.entity_mut(node).insert(ChildOf(at));
    // See `carry`: a moved thing is no longer this instrument's product.
    world.entity_mut(node).remove::<tower::Product>();

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
/// Shared by [`wield`] and [`operate`] so the two cannot disagree about what a
/// charged instrument does — the recipe lookup, the heat check and the refusals
/// are one body, and only *how the instrument was named* differs.
fn start(world: &mut World, at: Entity, name: &str, verb: Verb) {
    // The heat source is not a stage: wielding it means lighting it, and it takes
    // no Focus slot at all (§10.1).
    if world.get::<tower::HeatSource>(at).is_some() {
        tower::kindle(world, at);
        return;
    }

    let holding: Vec<String> = tower::contents(world, at)
        .into_iter()
        .filter_map(|node| world.get::<tower::Name>(node).map(|held| held.0.clone()))
        .collect();

    // §10.1: the *material's state* decides what an instrument can do, so a
    // refusal has to say what is in there — otherwise "nothing happens" is
    // indistinguishable from "you loaded the wrong thing".
    let Some((ticks, heat)) = world
        .resource::<Recipes>()
        .matching(name, &holding)
        .map(|recipe| (recipe.ticks, recipe.heat))
    else {
        let key = if holding.is_empty() {
            "wield_empty"
        } else {
            "wield_no_recipe"
        };
        let listed = holding.join(", ");
        let message = world
            .resource::<Prose>()
            .line(key, &[("name", name), ("detail", &listed)]);
        let mut scrollback = world.resource_mut::<Scrollback>();
        scrollback
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, verb.canonical())
            .text(FieldName::Path, name)
            // What it holds is a *fact*, so it goes in `State` and not `Detail`.
            // `Detail` is secondary prose subordinate to `Message`, and a view
            // that honours prose draws it — putting the contents there printed
            // them in front of the sentence written to explain them.
            .text(FieldName::State, &listed)
            .text(FieldName::Message, &message)
            .role(Role::Cost)
            .finish();
        return;
    };

    // Heat is checked **here**, at the start, and the run then completes even if
    // the fire dies under it (§10.1). Pausing would be a countdown; spoiling
    // would cost progress. The pressure to batch heated stages comes from the
    // burn being time-based, not from a risk of losing the work.
    //
    // The flag is the *recipe's*, read from `recipes.toml` — see `Recipe::heat`.
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
    // The **verb the player used** goes on the `Working`, so a refusal while it
    // runs says "busy grinding" rather than "busy wielding" — see
    // `Verb::participle` and `work_busy`.
    tower::begin(world, at, verb, subject, ticks);
}

/// Turn everything an instrument holds out into the store (§10.1).
///
/// **The counterpart of `purge`, and the difference is the byproduct rule.**
/// `purge` destroys what you did not mean to make; this keeps it. §10.1 says
/// every byproduct has at least one use — husks are the mortar's leavings *and*
/// the water bath's input — so a loop that can only clear by destroying is a loop
/// that never finds the second route to a draught.
///
/// Instant, like [`carry`], because that is what it is: a `move` of everything at
/// once rather than a named thing at a time. It does not take §9's triage slot
/// the way `purge` does — the four ticks a scour costs are the price of
/// *destroying*, and paying them to put something on a shelf would be a toll
/// rather than a cost.
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
        if let Some(held) = world.get::<tower::Name>(node) {
            turned_out.push(held.0.clone());
        }
        world.entity_mut(node).insert(ChildOf(store));
        // See `carry`: what leaves an instrument is no longer its product.
        world.entity_mut(node).remove::<tower::Product>();
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
/// `moved` rides as a **fact** in `State` rather than only inside the sentence,
/// so `sift husks orb.log` finds the line that carried them. It is not `Detail`:
/// a record holding prose draws its `Detail` *in front of* the message, which is
/// the trap `wield`'s refusal fell into.
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

/// Collect what an instrument made (§10.1).
pub(super) fn siphon(intent: &Intent, world: &mut World) {
    let Some(name) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Siphon, world);
        return;
    };
    match instrument(world, &name) {
        Some((at, _)) => {
            tower::siphon(world, at);
        }
        None => missing(Verb::Siphon, &name, world),
    }
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
    match instrument(world, &name) {
        // Stopping the athanor damps the fire and **banks** what has not burnt,
        // which is what makes `stop athanor` at the end of a script loop worth
        // writing (§10.1).
        Some((at, _)) if world.get::<tower::HeatSource>(at).is_some() => {
            tower::damp(world, at);
        }
        Some((at, _)) => {
            tower::stop(world, at);
        }
        None => missing(Verb::Stop, &name, world),
    }
}

/// Start something that takes time.
///
/// §5.0: issuing is free and instant; the *action* occupies a slot for its
/// duration, and that concurrency is the whole economy. The subject must be
/// where the player is standing — §7 puts a domain's belongings in the domain,
/// which is why `divine` resolves in the archive and nowhere else.
pub(super) fn work(intent: &Intent, world: &mut World, ticks: u64) {
    let Some(target) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(intent.verb, world);
        return;
    };

    let cwd = world.resource::<Cwd>().0;
    let Some(subject) = tower::children_of(world, cwd)
        .into_iter()
        .find(|node| {
            world
                .get::<tower::Name>(*node)
                .is_some_and(|n| n.0 == target)
        })
        .and_then(|node| world.get::<tower::NodeId>(node).copied())
    else {
        missing(intent.verb, &target, world);
        return;
    };

    tower::begin(world, cwd, intent.verb, subject, ticks);
}

/// Destroy something where you stand.
///
/// §7 makes destruction *"useful, everyday, and scriptable"* rather than a trap,
/// and §9's per-pane **triage** slot is why it still runs during a brew: short
/// work is not what the production slot is for.
pub(super) fn purge(intent: &Intent, world: &mut World) {
    let Some(target) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Purge, world);
        return;
    };

    // **By leaf.** A `Place` argument resolves to the full path (§7), while a node
    // carries only its last segment — so comparing the two matched *nothing* for
    // an instrument, and every `purge alembic` reached its target through the
    // tower-wide fallback below instead. That is what made a destructive verb
    // work at a distance, and it looked like a deliberate fallback rather than
    // the only path that ever fired.
    let leaf = crate::parser::leaf(&target).to_owned();
    let cwd = world.resource::<Cwd>().0;
    let found = tower::children_of(world, cwd)
        .into_iter()
        .find(|node| world.get::<tower::Name>(*node).is_some_and(|n| n.0 == leaf));

    // `purge` takes `NounKind::Any`, so it reaches **places** too — and it has
    // to, or §7's guard never fires: aiming at a live domain reported "no such
    // thing" instead of the orb refusing, and those are very different answers
    // to give someone.
    //
    // **Only as far as something that refuses**, though. The fallback walks from
    // the root, and places resolve to full paths from anywhere (§7), so this let
    // a destructive verb work at a distance: standing in `/tower/archive`,
    // `purge mortar_and_pestle` scoured the laboratory's mortar and destroyed
    // what was in it. Every other pipeline verb goes through `instrument`, which
    // is scoped to `cwd`; this is that scope, with the guard's reach kept.
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
