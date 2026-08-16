//! Scrolls, and what spending one does.
//!
//! Four walks of the stacks assemble into a scroll at the lectern (§10, §19), and
//! until now that was where the archive stopped: an object with a name, a colour
//! and no use. This is the other half.
//!
//! # `wield`, not a word of its own
//!
//! `verb.rs`'s vocabulary test argues the case and refuses the alternative in
//! advance — *"22 is a number to defend, not a budget to spend: the next word
//! added here needs an argument of this shape"*. Spending a scroll is *setting a
//! thing going*, which is what `wield` already means, so the verb learned a
//! second argument kind ([`NounKind::Workable`](crate::parser::NounKind::Workable))
//! instead of the tower gaining a twenty-third word.
//!
//! # No prose here
//!
//! Rule 6, as everywhere else in `execute`: this emits facts and looks its
//! sentences up by key.

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
/// **Five against the four a scroll costs, and profitable on purpose.** Gleaning
/// is what keeps scrolls in circulation and makes automating the maze worth
/// doing; the walk's own hundreds of ticks and the fact that the replacement is
/// *drawn* rather than chosen are what stop it running away.
///
/// A placeholder like every other number here, and the largest balance exposure
/// in the feature: `orbs-balance` is still a stub, so nothing would catch this
/// drifting. If it runs hot, this is the one line that changes.
const SPOILS: usize = 5;

/// A scroll the archive can assemble, and what it does.
///
/// **A table of facts, not prose** — the same shape as `Verb::canonical` and
/// [`Errand::word`]. The words themselves are authored in `recipes.toml`; this
/// says which of them the game can act on, and
/// `every_scroll_the_lectern_makes_can_be_spent` fails the build if the two
/// disagree. A scroll in the content file with no arm here would be drawn,
/// carried, wielded and refused — which reads exactly like a bug in the verb.
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
/// **The one expression of that question**, and it has two callers on two paths:
/// [`spend`] below, and `tower::spell::block`, which must *not* charge a spend to
/// the production slot. Asking it twice is what went wrong — `begins_work` is
/// keyed on the verb alone, so the slot tier saw `Verb::Wield` and made a
/// scripted `wield quickening-scroll` wait out the very brew it was meant to
/// hurry, burn `PATIENCE` and give up. A typed line never reaches that tier, so
/// the two paths disagreed and only one of them was ever looked at.
///
/// Branching on the argument's **kind** rather than on its name is what keeps
/// this from being a string comparison in the executor: `Recipes::kind_of`
/// decides what a scroll is, from the content file.
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

    // Where the scroll is, which is where the player is standing. §7 governs
    // this exactly as it governs a reagent: you can only spend what is to hand.
    let cwd = world.resource::<Cwd>().0;
    let Some(holder) = super::pipeline::holder(world, cwd, &name) else {
        missing(Verb::Wield, &name, world);
        return true;
    };

    let Some(scroll) = Scroll::of(&name) else {
        // Authored but unimplemented: the lint below is what stops this
        // shipping, and saying so beats acknowledging in silence.
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

    // **Taken only once the effect landed.** A refusal that had already consumed
    // the scroll would be the trap §7 refuses — four walks of the stacks spent
    // on a sentence explaining why nothing happened.
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

    // **Drawn in one go, on the archive's own stream.** A maze that grew as it
    // was walked would issue its squares at a rate depending on how the ticks
    // were consumed, so a live-watched run and a `meditate`-collapsed one would
    // differ from one seed — `heat.rs` records the hazard and `Maze::new` the
    // rule. `RngStream::Archive` because this is an archive roll: §19 records
    // the shard draw once rolling `Yield` and changing a player's brew yields.
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
    // **No `Detail`**, which `produce::transmute` records the reason for one
    // field along: it is prose a view draws *in front of* the message, so the
    // scroll's own name would be printed before the sentence written to explain
    // it. The scroll is the `Name`, the errand it set is a `State`, and both are
    // facts a `sift` can find.
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
/// # It never refuses for want of something to hurry
///
/// The first version halved what was left of the run in hand and refused when
/// nothing was running — which made it unusable at exactly the moment a player
/// would reach for one. *Quicken the laboratory, then brew* is the obvious play
/// and it was the one thing the scroll could not do. Four walks of the stacks also
/// bought a single stage, where a window buys a stretch of work and rewards
/// lining it up.
///
/// So it sets [`Quickened`](tower::Quickened) on the laboratory for
/// [`QUICKENED_TICKS`](tower::QUICKENED_TICKS), and `tower::begin` reads it when
/// a run starts — the same moment §10.1 checks the athanor, and for the same
/// reason: a run that starts inside the window stays short even if the window
/// closes under it.
///
/// # And what is already running is hurried too
///
/// One rule, not two. The state means *this room works at double speed*, and a
/// run in flight is something the room is doing — so leaving it alone would make
/// wielding the scroll mid-brew look like it had done nothing. Halved from
/// **now** rather than from the start: halving the whole interval would refund
/// time already spent and, past the half-way point, land the end in the past.
///
/// Set once, never re-derived per tick, which is what keeps `meditate 60`
/// identical to sixty `meditate 1`s.
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
    // production pool and `CAPACITY` is 1, so this is at most one run — and it
    // is only hurried if it is in the room the scroll just quickened.
    let hurried = tower::in_flight(world)
        .into_iter()
        .map(|(_, at)| at)
        .find(|at| tower::domain_of(world, *at) == Some(domain));
    if let Some(at) = hurried
        && let Some(mut working) = world.get_mut::<tower::Working>(at)
    {
        // **Only ever earlier**, and the guard is not belt-and-braces. `commands`
        // runs before `tower::finish` in `Sim::advance`, so a scroll spent on the
        // exact tick a run would land sees `left == 0` — and `(0 / 2).max(1)` is
        // 1, which pushed `ends` a tick *past* where it already was. A scroll
        // that makes the thing it hurries land later is the one outcome it must
        // never have, and `max(1)` was there to stop a one-tick run becoming a
        // no-tick one, which is a different case entirely.
        let left = working.ends.get().saturating_sub(now.get());
        if left > 1 {
            working.ends = crate::Tick::new(now.get() + (left / tower::QUICKENED_BY).max(1));
        }
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
/// # One herb per scroll, not all of them
///
/// The plan said *unlock the reagents* and one scroll doing all of it would have
/// been a single dud draw for ever after — the lectern would go on assembling a
/// scroll with nothing left to give, which is the dead end this whole item
/// exists to close. One each means the first three are all worth having and the
/// laboratory visibly grows three times.
///
/// # What is unlockable is derived, never listed
///
/// A **base reagent** is one the vocabulary knows and no recipe produces: sage
/// and rock-salt are the two the tower opens with, and anything else in that set
/// is a herb waiting for a shelf. So authoring a new one in `recipes.toml` makes
/// it unlockable the same tick, with no arm to add here — the same rule
/// `tower::home` follows, for the same reason a hand-kept list is a promise kept
/// until somebody is busy.
///
/// Fuel is excluded: the athanor's charcoal is a base reagent by that test and is
/// not a herb, and unlocking what the laboratory already burns would give a
/// player nothing.
///
/// **Alphabetical, and that is a decision.** `Recipes::vocabulary` is sorted, so
/// which herb arrives first is fixed for every seed — a *roll* here would be a
/// second draw stacked on the lectern's, and §19 has already refused one layer of
/// that in the archive's yield. What the player chooses is when to spend it, not
/// what they get.
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
/// *Base* is "the vocabulary knows it and **nothing in the tower makes it**", and
/// getting that wrong is visible rather than subtle: the first version asked only
/// `Recipes::outputs`, which is a recipe's `output` and not its `leaves` — so
/// every byproduct in the game read as a herb, and four scrolls put `dregs`,
/// `ash` and a `fragment` on the shelf as inexhaustible stock. It took one dump
/// to see and no test would have said a word.
///
/// So *made* is everything with a source: a recipe's output, a recipe's leaving,
/// a fuel, and what a fuel leaves behind when it burns.
///
/// And *the laboratory's*, by [`tower::home`] — which is what keeps the archive's
/// `fragment` out without naming it. A fragment is a base reagent by every test
/// above; it is simply not a herb, and the rule that already knows which room a
/// thing belongs to is the one to ask rather than a second list here.
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
