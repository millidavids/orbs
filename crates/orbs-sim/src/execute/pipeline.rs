//! §10.1's brewing loop: clear, charge, wield, draw off.
//!
//! `move`, `wield`, `stop`, `siphon`, `purge` and the `recall` that makes the
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
/// **The instruments first**, in raise order — because the thing a player is
/// moving mid-pipeline is the output of the last stage, and it is still inside
/// the tool that made it. **This is what retired `siphon`** (§19): the loop used
/// to be `move`/`wield`/`siphon`, so a stage's output had to be drawn onto the
/// bench before the next tool could have it. Reaching into an idle instrument
/// means `digest ground-sage` advances the pipeline on its own, and drawing off
/// first was a step that had stopped doing anything.
///
/// **Busy instruments are skipped** — §10.1's lock covers taking as much as
/// putting, and without this a `move` could gut a run in flight, spend the Focus
/// slot for nothing and say not a word about it.
///
/// **The store last.** It is stock, and stock is the fallback.
///
/// # There is no floor
///
/// A tier used to come first here for things lying loose in the room. Nothing
/// can be there any more: `siphon` was the only thing that ever put a reagent on
/// the floor, and `move`'s destination resolves through [`instrument`], which
/// finds fixtures only. The bench and the shelf are one place now, and it is the
/// dispensary.
/// **The order itself lives in `tower::reach`**, because it is a rule about the
/// world rather than about this file: a spell that resolves a name at cast and a
/// verb body that looks it up at execution have to agree, and two copies of a
/// search order are two answers to *what does `digest ground-sage` pick up*.
///
/// **The arsenal is last, wherever it is.** Its contents are nameable from every
/// room (`tower::keep`), so they have to be *findable* from every room or `move
/// clarity to flask_and_rod` resolves at full confidence and then reports "no
/// such thing" — §15's dead end, arriving through the exemption that exists to
/// remove one. Last is the point: a reagent in the room always outranks one
/// carried, so adding it cannot change what an existing command picks up.
fn reachable(world: &World, cwd: Entity) -> Vec<Entity> {
    tower::reach::look(world)
        .scope(tower::reach::Scope::Fetch(cwd))
        .candidates()
}

/// Somewhere a `move` can name: an instrument here, or the arsenal.
///
/// **The arsenal from anywhere, and it is the only place that gets this.**
/// [`instrument`] wants a `Fixture` child of `cwd`, and a domain is neither —
/// which is why nothing in the tower could be carried between rooms at all
/// before there was an arsenal. `tower::keep` states the exemption and why it is
/// narrow; what makes it safe rather than a repeal of §7 is that the arsenal
/// takes finished work only, so it cannot become the room everything ends up in.
///
/// Tried **after** the instruments, so a room that ever raises a fixture called
/// `arsenal` still means its own.
///
/// **Both ends of a `move` ask this**, and that is the correction: it was the
/// destination's lookup alone, so `move clarity to arsenal` worked and `move
/// clarity from arsenal to alembic` answered *"there is no /tower/arsenal within
/// reach"* — a name that resolves at full confidence and then reports itself
/// unreachable, which is §15's worst dead end and precisely what `tower::keep`'s
/// *"nameable is not enough"* note enumerates. Three lookups learned the
/// exemption and the fourth did not.
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

/// Where the thing called `named` is, among everything within reach.
///
/// The *place*, not the node: taking a unit goes through
/// [`tower::take`](crate::tower::take), which is keyed by where a thing is
/// standing. [`reachable`] decides what within reach means, so this obeys §10.1's
/// search order and its lock without a second opinion about either.
pub(super) fn holder(world: &World, cwd: Entity, named: &str) -> Option<Entity> {
    let node = reachable(world, cwd).into_iter().find(|node| {
        world
            .get::<tower::Name>(*node)
            .is_some_and(|name| name.0 == named)
    })?;
    world.get::<ChildOf>(node).map(ChildOf::parent)
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

    let Some((to, destination)) = addressed(world, &destination) else {
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
        // **Never the destination itself.** `reachable` walks every instrument
        // and store in the room, the destination included — so `move charcoal to
        // athanor` when the athanor already held charcoal found it *there*,
        // took a unit out and put a unit back, and reported `charcoal: athanor
        // to athanor`. A player could not add a second unit of anything an
        // instrument already had, and the command said it had worked.
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

    // **The arsenal's door, and it is checked before anything moves.** Finished
    // work only — see `tower::admits` for why the question is about the kind and
    // never about the name. Refusing here rather than after the fact is the rule
    // the multi-reagent charge above already follows: a half-done `move` leaves
    // the player working out what went where before anything will start again.
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

    hand(world, node, to);

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
    // **A scroll first, and it returns before `start`.** Spending one is not a
    // run: it takes no production slot, so it is not refused while a brew is in
    // flight — which is exactly when a player reaches for one — and it never
    // reaches `Verb::transmutes`, which `land::finish` reads. `begins_work` is
    // `const fn(Verb)` and cannot see the argument, so branching here rather
    // than there is what keeps a scroll out of the pool at all.
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

/// Move one of whatever `node` is into `to`.
///
/// # One unit if it is stock, the whole thing if it is not
///
/// A reagent is a pile with a count, so handing one over takes a unit and adds a
/// unit — and an endless pile yields its unit without shrinking, which is what
/// makes the base reagents the floor the laboratory stands on. A vessel, a file
/// and a spell are one of a kind and simply move.
///
/// **One function for `move` and for charging an instrument**, because they were
/// two copies of the same three lines and the copies drifted the moment counts
/// arrived: `move sage` took a unit while `grind sage` re-parented the endless
/// pile itself into the mortar — where the run consumed it and `empty` swept it
/// into the store, so the tower's inexhaustible sage was gone for good after one
/// grind. The second copy is what made the *first* fix look like it worked.
fn hand(world: &mut World, node: Entity, to: Entity) {
    let Some(thing) = world.get::<tower::Name>(node).map(|name| name.0.clone()) else {
        return;
    };
    // **A moved thing is no longer a product.** The marker means "this
    // instrument made this", which stops being true the moment it is carried
    // somewhere else — the destination used to read `ready` on the panel before
    // anything had been wielded there. `give` spawns or merges without it, so
    // the stock path clears it by construction.
    let Some(&kind) = world.get::<tower::Nameable>(node).map(|kind| &kind.0) else {
        return;
    };
    if world.get::<tower::Stock>(node).is_none() {
        world.entity_mut(node).insert(ChildOf(to));
        world.entity_mut(node).remove::<tower::Product>();
        return;
    }
    if let Some(from) = world.get::<ChildOf>(node).map(ChildOf::parent) {
        tower::take(world, from, &thing, 1);
    }
    tower::give(world, to, &thing, kind, 1);
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

    hand(world, node, at);

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

    let holding = tower::holdings(world, at);

    // §10.1: the *material's state* decides what an instrument can do, so a
    // refusal has to say what is in there — otherwise "nothing happens" is
    // indistinguishable from "you loaded the wrong thing".
    let Some((ticks, heat)) = world
        .resource::<Recipes>()
        .matching(name, &holding, world.resource::<tower::Learned>())
        .map(|recipe| (recipe.ticks, recipe.heat))
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
        let Some(held) = world.get::<tower::Name>(node).map(|name| name.0.clone()) else {
            continue;
        };
        let kind = world
            .get::<tower::Nameable>(node)
            .map_or(crate::parser::NounKind::Reagent, |kind| kind.0);
        turned_out.push(held.clone());

        // **Poured onto the pile, not stood beside it.** Reparenting the node
        // left a second `ground-sage` in the dispensary every time the mortar
        // was emptied — two rows under one name, and a name the parser then has
        // to choose between arbitrarily. That was already true before counts;
        // it simply had nothing to show it with.
        match world.get::<tower::Stock>(node).copied() {
            Some(tower::Stock::Counted(count)) => {
                world.entity_mut(node).despawn();
                tower::give(world, store, &held, kind, count);
            }
            // Endless in an instrument is not a thing the tower makes, and
            // pouring it into the store would silently make the store endless.
            // Dropped rather than merged, exactly as a spent unit is.
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

    // **A spell first.** `stop` used to reach instruments only, which made an
    // invoked spell impossible to call off — see `tower::spell::stop_spell`.
    // Spells are looked at before instruments because a spell and an instrument
    // can never share a name (one is a `.spell` in the grimoire, the other a
    // fixture), so the order is a preference between disjoint sets rather than a
    // tie-break that could surprise anyone.
    if tower::spell::stop_spell(world, &name) {
        return;
    }

    match instrument(world, &name) {
        // Stopping the athanor damps the fire and **banks** what has not burnt,
        // which is what makes `stop athanor` at the end of a script loop worth
        // writing (§10.1).
        Some((at, _)) if world.get::<tower::HeatSource>(at).is_some() => {
            tower::damp(world, at);
        }
        Some((at, _)) => {
            // **The stacks are abandoned, not merely un-run.** `divine` inserts
            // no `Working` — reading takes no production slot — so without this
            // `stop lectern` would find an instrument, do nothing, and say so:
            // the un-stoppable `divine` §19 records, still true and now harder
            // to see because the verb had changed underneath it.
            //
            // **And it does not `return`.** The lectern is the first instrument
            // that can be doing two things at once — assembling a scroll takes
            // twenty ticks of `Working`, and a maze can be open across all of
            // them — so stopping only the maze left the run going and made the
            // player type `stop lectern` a second time to reach it. One `stop`
            // ends what is happening here, whatever is happening.
            if world.get::<tower::Maze>(at).is_some() {
                world.entity_mut(at).remove::<tower::Maze>();
                super::research::refresh(world);
                let message = world
                    .resource::<crate::content::Prose>()
                    .line("research_abandoned", &[]);
                world
                    .resource_mut::<Scrollback>()
                    .records_mut()
                    .push(RecordKind::Completion)
                    .text(FieldName::Name, Verb::Stop.canonical())
                    .text(FieldName::Message, &message)
                    .role(Role::Cost)
                    .finish();
                // Nothing else to stop unless a run is also under way.
                if world.get::<tower::Working>(at).is_none() {
                    return;
                }
            }
            tower::stop(world, at);
        }
        None => missing(Verb::Stop, &name, world),
    }
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
    let found = tower::reach::look(world)
        .find(&leaf)
        // **...and whatever the arsenal holds.** It is nameable from every room,
        // so `purge` has to reach it from every room: a destructive verb that
        // resolves and then says "no such thing" is worse than one that refuses,
        // because the player cannot tell whether the thing is gone. Throwing away
        // a potion you no longer want is exactly the everyday maintenance §7
        // means by *"destruction is a tool, not a trap"*.
        .or_else(|| tower::kept(world, &leaf));

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
