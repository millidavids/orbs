//! A scroll's calm supply (§19, *Supply*).
//!
//! No single scroll does anything. `wield` opens the supply when the arsenal
//! holds enough for the slowest rate and draws at once; the word again raises it
//! a rate, paying only the difference, as on the wall. While open it draws every
//! `every` ticks from the word, used or not, so leaving one open is a cost.
//! Draining steps it down and then shuts it. It pauses while a siege runs, and
//! the siege takes `wield` while it does.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use super::spend::{Scroll, refuse, say, speak};
use crate::content::{Calm, Scrolls};
use crate::tower::supply::{self, Refusal, Scope, Supplies, Word};
use crate::tower::{self, siege};

/// `wield <scroll>` outside a siege: open its supply, or raise it a rate.
pub(super) fn open(world: &mut World, name: &str) {
    // Inert without both halves: an effect the game can act on, and a calm
    // table to draw by. The lints stop either shipping alone.
    let calm = world.resource::<Scrolls>().get(name).cloned();
    let Some(calm) = calm.filter(|_| Scroll::of(name).is_some()) else {
        refuse(world, name, "scroll_inert");
        return;
    };
    if let Some(refusal) = supply::barred(world, name, Scope::Calm) {
        turn_down(world, name, &refusal);
        return;
    }
    // Asked of the tower, not of the room: the wall takes `wield` only in the
    // bailey, so from anywhere else a siege would otherwise reach here.
    if siege::besieged(world) {
        refuse(world, name, "calm_besieged");
        return;
    }
    if let Some(key) = unready(world, name) {
        refuse(world, name, key);
        return;
    }

    let fresh = tower::freshness_of(world, name);
    let word = match supply::word(world, name, &calm.rates, fresh, Scope::Calm) {
        Ok(word) => word,
        Err(refusal) => {
            turn_down(world, name, &refusal);
            return;
        }
    };
    let at = word.at();
    let draws = calm.rates.get(at).map_or(0, |rate| rate.draws);
    let key = if matches!(word, Word::Raised(_)) {
        "calm_raised"
    } else {
        "calm_open"
    };
    say(
        world,
        key,
        name,
        &[
            ("count", &draws.to_string()),
            ("quantity", &calm.every.to_string()),
        ],
        Role::Success,
    );
    land(world, name, &calm, at, matches!(word, Word::Opened(_)));
    supply::gauge(world);
}

/// Say why a word on `name` did nothing.
fn turn_down(world: &mut World, name: &str, refusal: &Refusal) {
    speak(world, refusal.key, name, &refusal.fields(name), Role::Cost);
}

/// Why `name`'s effect could not land now, as a prose key, or `None` if it can.
/// Asked before a scroll is opened or drawn.
fn unready(world: &World, name: &str) -> Option<&'static str> {
    match Scroll::of(name)? {
        Scroll::Quickening => super::spend::laboratory(world)
            .is_none()
            .then_some("scroll_nowhere"),
        Scroll::Verdant => (!super::verdant::growing(world)).then_some("scroll_already_flowered"),
        Scroll::Gleaning => None,
    }
}

/// Draw every calm supply that is due.
///
/// Keyed on the clock from each one's word, so a watched hour and a
/// `meditate`-collapsed one draw alike. Paused while a siege runs: the draws a
/// siege covers are skipped, not owed. A supply stopped since its last draw is
/// forgotten where its next would have come, so opening it after pays again.
pub(crate) fn drain(world: &mut World) {
    let (open, stopped) = {
        let supplies = world.resource::<Supplies>();
        (
            supplies.names(Scope::Calm),
            supplies.stopped_names(Scope::Calm),
        )
    };
    if open.is_empty() && stopped.is_empty() {
        return;
    }
    for name in stopped {
        let since = world
            .resource::<Supplies>()
            .stopped(&name)
            .map(|was| was.since);
        if due(world, &name, since) {
            world.resource_mut::<Supplies>().reopen(&name);
        }
    }
    let drawing: Vec<String> = open
        .into_iter()
        .filter(|name| {
            let since = world
                .resource::<Supplies>()
                .get(name)
                .map(|open| open.since);
            due(world, name, since)
        })
        .collect();
    // Asked only on a tick something would draw: a siege is found by query.
    if drawing.is_empty() || siege::besieged(world) {
        return;
    }
    for name in drawing {
        draw(world, &name);
    }
    supply::gauge(world);
}

/// Whether `name`, opened at `since`, draws on this tick.
fn due(world: &World, name: &str, since: Option<u64>) -> bool {
    let now = world.resource::<crate::tick::Tick>().get();
    let every = world
        .resource::<Scrolls>()
        .get(name)
        .map_or(0, |calm| calm.every);
    since.is_some_and(|since| every > 0 && now > since && (now - since).is_multiple_of(every))
}

/// `name`'s supply draws again on the clock: at the fastest rate up to the
/// chosen one that the arsenal and freshness allow, or shut if none.
///
/// Silent unless it steps down or runs dry.
fn draw(world: &mut World, name: &str) {
    let Some(calm) = world.resource::<Scrolls>().get(name).cloned() else {
        return;
    };
    let Some(open) = world.resource::<Supplies>().get(name).copied() else {
        return;
    };
    world.resource_mut::<Supplies>().new_period(name);
    let held = supply::held_in_arsenal(world, name);
    let fresh = tower::freshness_of(world, name);
    // Stale or short: the line names which, since the answer to one is making
    // more and to the other holding more.
    let slowed = supply::slowed(&calm.rates, held, fresh, open.chosen);
    let Some(at) = supply::running(&calm.rates, held, fresh, open.chosen) else {
        world.resource_mut::<Supplies>().close(name);
        refuse(world, name, supply::ran_low(Scope::Calm, slowed, true));
        return;
    };
    // Nothing is taken for an effect that cannot land: §7's trap, a scroll
    // spent on a sentence.
    if let Some(key) = unready(world, name) {
        world.resource_mut::<Supplies>().close(name);
        refuse(world, name, key);
        return;
    }
    if at < open.running {
        let draws = calm.rates.get(at).map_or(0, |rate| rate.draws);
        say(
            world,
            supply::ran_low(Scope::Calm, slowed, false),
            name,
            &[
                ("count", &draws.to_string()),
                ("quantity", &calm.every.to_string()),
            ],
            Role::Cost,
        );
    }
    land(world, name, &calm, at, false);
}

/// Take `name`'s ration at rate `at` and let it act, beyond what this period
/// already applied.
///
/// `opening` is the word's own first draw, which hurries a run in flight or
/// sets an open maze to gather. A raise keeps the clock, and the window it sets
/// runs to the next draw.
fn land(world: &mut World, name: &str, calm: &Calm, at: usize, opening: bool) {
    let Some(rate) = calm.rates.get(at).copied() else {
        return;
    };
    let ration = supply::ration(world, name, at, rate.draws);
    let worth = u32::try_from(rate.worth).unwrap_or(0);
    supply::applied(world, name, worth);

    match Scroll::of(name) {
        // One tick past the next draw, so the next window overlaps this one and
        // no run starts in a gap between them.
        Some(Scroll::Quickening) => {
            let ticks = until_next(world, name, calm.every) + 1;
            super::quicken::window(world, name, u64::from(worth), ticks, opening);
        }
        Some(Scroll::Gleaning) if opening => super::glean::set_open(world, name, worth),
        Some(Scroll::Verdant) => {
            let more = worth.saturating_sub(ration.applied);
            super::verdant::grow(world, name, more, calm.flowers_at.unwrap_or(0));
        }
        _ => {}
    }
}

/// Ticks from now to `name`'s next draw: a whole `every` on the tick of a draw.
fn until_next(world: &World, name: &str, every: u64) -> u64 {
    let now = world.resource::<crate::tick::Tick>().get();
    let since = world
        .resource::<Supplies>()
        .get(name)
        .map_or(now, |open| open.since);
    if every == 0 {
        return 0;
    }
    every - now.saturating_sub(since) % every
}
