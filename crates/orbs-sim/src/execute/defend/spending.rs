//! Opening the arsenal's supplies onto the wall (§5.1, §19 *Supply*).
//!
//! No single item does anything. `quaff warding` opens the warding supply when
//! the arsenal holds enough for its slowest rate, the same word again raises it
//! one rate, and `stop warding` closes it. Each round it draws on its own —
//! `drawing` is that half. What each item is worth is authored in
//! `siege.toml`, never written in Rust.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use super::drawing::{draw_now, freshness, hand_back};
use super::shared::{fixture, say};
use crate::content::SupplyKind;
use crate::parser::{Intent, Verb};
use crate::tower::supply::{self, Refusal, Scope, Supplies, Word};
use crate::tower::{self, siege, siege::Siege};

/// Open a supply on the wall, or raise the one that is open by a rate.
///
/// One body for `deploy`, `quaff` and a scroll's `wield`, which differ only in
/// what they may open.
pub(super) fn spend(intent: &Intent, world: &mut World, verb: Verb) {
    let Some(rampart) = fixture(world) else {
        say(world, verb, "defend_nowhere", &[], Role::Cost);
        return;
    };
    let Some(siege) = world.get::<Siege>(rampart) else {
        say(world, verb, "hold_unopened", &[], Role::Cost);
        return;
    };
    if !siege.running() {
        say(world, verb, "hold_over", &[], Role::Cost);
        return;
    }

    let Some(argument) = intent.arguments.first() else {
        say(world, verb, "spend_incomplete", &[], Role::Cost);
        return;
    };
    let named = crate::parser::leaf(&argument.value).to_owned();

    // Authored, not inferred (rule 6): an item with no entry cannot be opened,
    // which stops `quaff sage` quietly doing nothing.
    let Some(entry) = world
        .resource::<crate::content::Spendables>()
        .get(&named)
        .cloned()
    else {
        say(
            world,
            verb,
            "spend_useless",
            &[("name", &named)],
            Role::Cost,
        );
        return;
    };
    // The wrong word is its own refusal, and it names the right one (§6).
    let wanted = world
        .resource::<crate::content::Spendables>()
        .verb_for(&named);
    if wanted != Some(verb) {
        let wanted = wanted.map_or("nothing", Verb::canonical).to_owned();
        say(
            world,
            verb,
            "spend_wrong_word",
            &[("name", &named), ("detail", &wanted)],
            Role::Cost,
        );
        return;
    }

    // "There is none" first, and a different fact from too few: a name the
    // arsenal does not hold has never been made, or waits on another shelf.
    if let Some(refusal) = supply::barred(world, &named, Scope::Siege) {
        refuse(world, verb, &named, &refusal);
        return;
    }
    // Ending a siege is the player's call, never a spell's (§19). After "there
    // is none", so a spell with an empty shelf is told that.
    if entry.kind == SupplyKind::Still
        && world
            .get_resource::<tower::spell::Caller>()
            .is_some_and(|caller| caller.0.is_some())
    {
        say(
            world,
            verb,
            "still_by_hand",
            &[("name", &named)],
            Role::Cost,
        );
        return;
    }

    let fresh = freshness(world, rampart, &named);
    let word = match supply::word(world, &named, &entry.rates, fresh, Scope::Siege) {
        Ok(word) => word,
        Err(refusal) => {
            refuse(world, verb, &named, &refusal);
            return;
        }
    };
    let at = word.at();

    // Stillness draws once, at opening, and ends the fight.
    if entry.kind == SupplyKind::Still {
        draw_now(world, rampart, &named, at);
        still(world, rampart, &named);
        return;
    }

    let drew = draw_now(world, rampart, &named, at);
    super::publish::publish(world, rampart);
    supply::gauge(world);
    let count = entry.rates.get(at).map_or(0, |rate| rate.draws).to_string();
    let key = match (word, drew) {
        (Word::Raised(_), _) => "supply_raised",
        (_, true) => "supply_open",
        (_, false) => "supply_open_waits",
    };
    say(
        world,
        verb,
        key,
        &[("name", &named), ("count", &count)],
        Role::Success,
    );
}

/// Say why a word on `named` did nothing.
fn refuse(world: &mut World, verb: Verb, named: &str, refusal: &Refusal) {
    say(world, verb, refusal.key, &refusal.fields(named), Role::Cost);
}

/// End the siege where it stands, handing back what the cancelled round was
/// given — `still` is a supply drawn once.
fn still(world: &mut World, rampart: Entity, named: &str) {
    let pledges = world
        .get_mut::<Siege>(rampart)
        .map(|mut siege| siege.still().0)
        .unwrap_or_default();
    // No penalty: the pledged quintessence returns, and so does every roll
    // modifier this round drew.
    let pledged: u32 = pledges
        .iter()
        .map(|pledge| siege::cost_of(pledge.die))
        .sum();
    let ceiling = tower::ceiling(world);
    world
        .resource_mut::<tower::Quintessence>()
        .restore(pledged, ceiling);
    hand_back(world, named);
    super::publish::publish(world, rampart);
    say(
        world,
        Verb::Quaff,
        "quaff_stilled",
        &[("name", named)],
        Role::Success,
    );
    super::report::settle(world, rampart, siege::Outcome::Stilled);
}

/// `stop <item>` on a supply — shut it. Returns whether `named` was one.
///
/// What this round already drew stands; the supply simply draws no more.
pub(in crate::execute) fn stop(world: &mut World, named: &str) -> bool {
    let named = crate::parser::leaf(named);
    let is_supply = world
        .resource::<crate::content::Spendables>()
        .get(named)
        .is_some();
    if !is_supply {
        return false;
    }
    let (key, role) = match world.resource_mut::<Supplies>().close(named) {
        Some(open) if open.scope == Scope::Calm => ("calm_stopped", Role::Success),
        Some(_) => ("supply_stopped", Role::Success),
        None => ("supply_not_open", Role::Cost),
    };
    say(world, Verb::Stop, key, &[("name", named)], role);
    if let Some(rampart) = fixture(world) {
        super::publish::publish(world, rampart);
    }
    supply::gauge(world);
    true
}

/// Shut every supply the siege had open — it is over.
pub(super) fn close_siege(world: &mut World) {
    world.resource_mut::<Supplies>().close_all(Scope::Siege);
    supply::gauge(world);
}
