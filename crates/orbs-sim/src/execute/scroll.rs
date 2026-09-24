//! Scrolls, and what spending one does.
//!
//! Four walks of the stacks assemble into a scroll at the lectern (§10, §19);
//! this is the other half, the scroll having had a name and no use.
//!
//! Spent with `wield` rather than a word of its own: spending a scroll is
//! *setting a thing going*, so the verb learned a second argument kind
//! ([`NounKind::Workable`]) instead of the tower gaining a twenty-third word.
//!
//! Rule 6, as everywhere in `execute`: facts here, sentences looked up by key.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Prose, Recipes};
use crate::parser::{Intent, NounKind, Verb};
use crate::rng::{RngStream, Rngs};
use crate::session::Scrollback;
use crate::tower::maze::{Errand, Maze};
use crate::tower::{self, Cwd};

use super::missing;

/// How many spoils a gleaning errand scatters.
///
/// Five against the four a scroll costs, profitable on purpose: gleaning keeps
/// scrolls in circulation and makes automating the maze worth doing. The walk's
/// hundreds of ticks and the *drawn* rather than chosen replacement stop it
/// running away.
///
/// A placeholder, and the largest balance exposure in the feature —
/// `orbs-balance` is still a stub, so nothing would catch it drifting.
const SPOILS: usize = 5;

/// A scroll the archive can assemble, and what it does.
///
/// A table of facts, not prose — the shape of `Verb::canonical` and
/// [`Errand::word`]. The words are authored in `recipes.toml`; this says which
/// the game can act on, and `every_scroll_the_lectern_makes_can_be_spent` fails
/// the build if the two disagree. A scroll with no arm here would be drawn,
/// carried, wielded and refused, which reads like a bug in the verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scroll {
    /// Sets the open stacks to gather rather than to escape.
    Gleaning,
    /// Halves what is left of the run in hand.
    Quickening,
    /// Puts a base reagent the laboratory has never had on its shelf.
    Verdant,
}

impl Scroll {
    /// Every scroll that has an effect behind it.
    pub const ALL: [Self; 3] = [Self::Gleaning, Self::Quickening, Self::Verdant];

    /// The name this is authored and carried under.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Gleaning => "gleaning-scroll",
            Self::Quickening => "quickening-scroll",
            Self::Verdant => "verdant-scroll",
        }
    }

    /// The scroll called `name`, if the game has one.
    #[must_use]
    pub fn of(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|scroll| scroll.word() == name)
    }
}

/// The scroll this `wield` spends, if it spends one rather than charging a tool.
///
/// The one expression of that question, for [`spend`] below and for
/// `tower::spell::block`, which must *not* charge a spend to the production
/// slot. Asking it twice went wrong: `begins_work` is keyed on the verb alone,
/// so the slot tier saw `Verb::Wield` and made a scripted `wield
/// quickening-scroll` wait out the brew it was meant to hurry, burn `PATIENCE`
/// and give up. A typed line never reaches that tier, so only one of the two
/// disagreeing paths was ever looked at.
///
/// Branches on the argument's *kind*, not its name, so this is not a string
/// comparison in the executor: `Recipes::kind_of` decides what a scroll is,
/// from the content file.
pub(crate) fn spending(intent: &Intent) -> Option<&str> {
    intent
        .arguments
        .first()
        .filter(|argument| argument.kind == NounKind::Scroll)
        .map(|argument| argument.value.as_str())
}

/// Spend the scroll `intent` names.
///
/// Returns whether the argument was a scroll at all, so `wield` can fall through
/// to its instrument lookup.
pub(super) fn spend(intent: &Intent, world: &mut World) -> bool {
    let Some(name) = spending(intent) else {
        return false;
    };
    let name = name.to_owned();

    // Where the player is standing. §7, as for a reagent: you can only spend
    // what is to hand.
    let cwd = world.resource::<Cwd>().0;
    let Some(holder) = super::pipeline::holder(world, cwd, &name) else {
        missing(Verb::Wield, &name, world);
        return true;
    };

    let Some(scroll) = Scroll::of(&name) else {
        // Authored but unimplemented: the lint below stops this shipping, and
        // saying so beats acknowledging in silence.
        refuse(world, &name, "scroll_inert");
        return true;
    };

    let spent = match scroll {
        Scroll::Gleaning => glean(world, &name),
        Scroll::Quickening => quicken(world, &name),
        Scroll::Verdant => flower(world, &name),
    };
    if !spent {
        return true;
    }

    // Taken only once the effect landed: consuming it on a refusal is the trap
    // §7 refuses — four walks of the stacks spent on a sentence.
    tower::take(world, holder, &name, 1);
    true
}

/// Set the open stacks to gather, and scatter what they have to gather.
///
/// Returns whether the scroll was spent.
fn glean(world: &mut World, name: &str) -> bool {
    let Some(stacks) = super::research::stacks(world) else {
        refuse(world, name, "scroll_nowhere");
        return false;
    };
    if world.get::<Maze>(stacks).is_none() {
        refuse(world, name, "scroll_unopened");
        return false;
    }
    if world
        .get::<Maze>(stacks)
        .is_some_and(|maze| maze.errand() != Errand::Way)
    {
        refuse(world, name, "scroll_already");
        return false;
    }

    // Drawn in one go, on the archive's own stream. Growing it as the maze was
    // walked would tie the draw to how the ticks were consumed, so a
    // live-watched run and a `meditate`-collapsed one would differ from one
    // seed — `heat.rs` records the hazard, `Maze::new` the rule.
    // `RngStream::Archive` because it is an archive roll; §19 records the shard
    // draw once rolling `Yield` and changing a player's brew yields.
    let mut floor = world
        .get::<Maze>(stacks)
        .map(Maze::scatterable)
        .unwrap_or_default();
    if floor.len() < SPOILS {
        refuse(world, name, "scroll_no_room");
        return false;
    }
    let mut spoils = Vec::with_capacity(SPOILS);
    {
        let mut rngs = world.resource_mut::<Rngs>();
        let rng = rngs.stream(RngStream::Archive);
        for _ in 0..SPOILS {
            let at = rand::Rng::random_range(rng, 0..floor.len());
            spoils.push(floor.swap_remove(at));
        }
    }
    // Sorted, so the picture and the log read the same on every run whatever
    // order the draw returned them in.
    spoils.sort_unstable();

    if let Some(mut maze) = world.get_mut::<Maze>(stacks) {
        maze.set_errand(Errand::Glean, spoils);
    }
    // The four ways and the stacks both have something new to say.
    super::research::refresh(world);

    let message = world.resource::<Prose>().line(
        "scroll_gleaning",
        &[("name", name), ("count", &SPOILS.to_string())],
    );
    // No `Detail`: a view draws it *in front of* the message, so the scroll's
    // own name would print before the sentence explaining it. `Name` and
    // `State` instead, both facts a `sift` can find.
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, name)
        .text(FieldName::State, Errand::Glean.word().unwrap_or_default())
        .text(FieldName::At, "stacks")
        .count(FieldName::Quantity, SPOILS as u64)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
    true
}

/// Set the laboratory working at double speed for a while.
///
/// Returns whether the scroll was spent.
///
/// It never refuses for want of something to hurry: the first version halved
/// the run in hand and refused when nothing ran, so *quicken, then brew* — the
/// obvious play — was the one thing it could not do, and four walks of the
/// stacks bought a single stage rather than a stretch of work worth lining up.
/// So it sets [`Quickened`](tower::Quickened) on the laboratory for
/// [`QUICKENED_TICKS`](tower::QUICKENED_TICKS), and `tower::begin` reads it
/// when a run starts — where §10.1 checks the athanor, and for the same reason:
/// a run starting inside the window stays short even if the window closes.
///
/// What is already running is hurried too — one rule, not two, or wielding the
/// scroll mid-brew would look like it had done nothing. Halved from *now*
/// rather than from the start: halving the whole interval refunds time already
/// spent and, past half way, lands the end in the past. Set once, never
/// re-derived per tick, which keeps `meditate 60` identical to sixty
/// `meditate 1`s.
fn quicken(world: &mut World, name: &str) -> bool {
    let Some(shelf) = tower::home(world, SAGE) else {
        refuse(world, name, "scroll_nowhere");
        return false;
    };
    let Some(domain) = tower::domain_of(world, shelf) else {
        refuse(world, name, "scroll_nowhere");
        return false;
    };

    let now = *world.resource::<crate::Tick>();
    world.entity_mut(domain).insert(tower::Quickened {
        from: now,
        ticks: tower::QUICKENED_TICKS,
    });

    // Whatever the room is already doing, at the new rate. `in_flight` is the
    // production pool and `CAPACITY` is 1, so at most one run, and only if it
    // is in the room the scroll just quickened.
    let hurried = tower::in_flight(world)
        .into_iter()
        .map(|(_, at)| at)
        .find(|at| tower::domain_of(world, *at) == Some(domain));
    if let Some(at) = hurried
        && let Some(mut working) = world.get_mut::<tower::Working>(at)
        // Only ever earlier. `tower::hurried_from` carries the reason, beside
        // the rate it shares with `hastened`; it was written out here a second
        // time, which is one rule in two places.
        && let Some(ends) = tower::hurried_from(now, working.ends)
    {
        working.ends = ends;
    }

    let place = world
        .get::<tower::Name>(domain)
        .map_or_else(String::new, |name| name.0.clone());
    let message = world.resource::<Prose>().line(
        "scroll_quickening",
        &[
            ("name", name),
            ("source", &place),
            ("count", &tower::QUICKENED_TICKS.to_string()),
        ],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, name)
        .text(FieldName::At, &place)
        .count(FieldName::Remaining, tower::QUICKENED_TICKS)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
    true
}

/// Put a base reagent the laboratory has never had on its shelf.
///
/// Returns whether the scroll was spent.
///
/// One herb per scroll, not all of them: unlocking the lot would leave the
/// lectern assembling a scroll with nothing left to give, which is the dead end
/// this item exists to close. One each means the first three are all worth
/// having and the laboratory visibly grows three times.
///
/// What is unlockable is derived, never listed. A *base reagent* is one the
/// vocabulary knows and no recipe produces — sage and rock-salt are the two the
/// tower opens with — so authoring a new one in `recipes.toml` makes it
/// unlockable the same tick, with no arm to add here; a hand-kept list is a
/// promise kept until somebody is busy. Fuel is excluded: the athanor's
/// charcoal passes that test and is not a herb, and unlocking what the
/// laboratory already burns gives the player nothing.
///
/// Alphabetical: `Recipes::vocabulary` is sorted, so which herb arrives first
/// is fixed for every seed. A roll here would stack a second draw on the
/// lectern's, and §19 has refused one layer of that in the archive's yield —
/// what the player chooses is when to spend it, not what they get.
fn flower(world: &mut World, name: &str) -> bool {
    let Some(shelf) = tower::home(world, SAGE) else {
        refuse(world, name, "scroll_nowhere");
        return false;
    };
    let Some(herb) = locked(world, shelf).first().cloned() else {
        refuse(world, name, "scroll_already_flowered");
        return false;
    };

    let kind = world.resource::<Recipes>().kind_of(&herb);
    tower::give_endless(world, shelf, &herb, kind);

    let message = world
        .resource::<Prose>()
        .line("scroll_verdant", &[("name", name), ("detail", &herb)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, name)
        .text(FieldName::State, &herb)
        .text(FieldName::At, "dispensary")
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
    true
}

/// The base reagent the tower opens with, and the yardstick for where they live.
const SAGE: &str = "sage";

/// Base reagents the laboratory has not been given yet, alphabetically.
///
/// *Base* is "the vocabulary knows it and nothing in the tower makes it". The
/// first version asked only `Recipes::outputs` — a recipe's `output`, not its
/// `leaves` — so every byproduct read as a herb and four scrolls put `dregs`,
/// `ash` and a `fragment` on the shelf as inexhaustible stock; one dump showed
/// it and no test would have said a word. So *made* is everything with a
/// source: a recipe's output, a recipe's leaving, a fuel, and what a fuel
/// leaves behind when it burns.
///
/// And *the laboratory's*, by [`tower::home`], which keeps the archive's
/// `fragment` out without naming it: a fragment is base by every test above and
/// simply not a herb. Asking the rule that already knows which room a thing
/// belongs to beats a second list here.
fn locked(world: &World, shelf: Entity) -> Vec<String> {
    let recipes = world.resource::<Recipes>();
    let fuels = world.resource::<crate::content::Fuels>();

    let mut made: Vec<String> = recipes.outputs().into_iter().map(str::to_owned).collect();
    for instrument in recipes.instruments() {
        made.extend(
            recipes
                .for_instrument(instrument)
                .iter()
                .filter_map(|recipe| recipe.leaves.clone()),
        );
    }
    for fuel in fuels.names() {
        made.push(fuel.to_owned());
        if let Some(burnt) = fuels.get(fuel) {
            made.push(burnt.leaves.clone());
        }
    }

    recipes
        .vocabulary()
        .into_iter()
        .filter(|name| !made.iter().any(|made| made == name))
        .filter(|name| tower::home(world, name) == Some(shelf))
        .filter(|name| tower::held(world, shelf, name).is_none())
        .map(str::to_owned)
        .collect()
}

/// Say why the scroll was not spent, and leave it in the player's hands.
fn refuse(world: &mut World, name: &str, key: &str) {
    let message = world.resource::<Prose>().line(key, &[("name", name)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Message)
        .text(FieldName::Name, name)
        .text(FieldName::Message, &message)
        .role(Role::Cost)
        .finish();
}
