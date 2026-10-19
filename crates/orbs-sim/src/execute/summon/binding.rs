//! The circle's second use: a held beast bound as a demon (§19, *Supply*).
//!
//! A bare `summon` that holds refills the standing garrison. `summon golem`
//! binds the same hold into the arsenal instead, as a batch of demons that
//! `deploy` opens on the wall like any supply. Once the garrison is full, that
//! is the better use of a beast, and balancing the two is the menagerie's game.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use super::verbs::say;
use crate::content::{Prose, Spendables};
use crate::parser::{Intent, Verb};
use crate::tower;

/// The demon `intent` names, `None` when it names none, or `Err` once refused.
///
/// Asked before anything is drawn or called, so a refusal costs no call.
pub(super) fn named(intent: &Intent, world: &mut World) -> Result<Option<String>, ()> {
    let Some(argument) = intent.arguments.first() else {
        return Ok(None);
    };
    let name = crate::parser::leaf(&argument.value).to_owned();
    let summoned = world
        .resource::<Spendables>()
        .get(&name)
        .is_some_and(|entry| entry.summoned.is_some());
    let opened = world
        .resource::<tower::Opened>()
        .has(&tower::opened::demon_key(&name));
    if summoned && opened {
        return Ok(Some(name));
    }
    say(
        world,
        Verb::Summon,
        "summon_not_a_demon",
        &[("name", &name)],
        Role::Cost,
    );
    Err(())
}

/// Bind a held beast as `demon`, into the arsenal. Returns what to tell the
/// player, and whether anything reached a shelf: with no arsenal nothing does,
/// and the sentence says so rather than counting demons that went nowhere.
pub(super) fn bind(world: &mut World, demon: &str, within_par: bool) -> (String, bool) {
    let count = world
        .resource::<Spendables>()
        .get(demon)
        .and_then(|entry| entry.summoned)
        .map_or(0, |summoned| {
            if within_par {
                summoned.yields
            } else {
                summoned.past_par
            }
        });
    let Some(arsenal) = tower::home(world, demon) else {
        let line = world
            .resource::<Prose>()
            .line("summon_unkept", &[("name", demon)]);
        return (line, false);
    };
    let kind = tower::kind_of(world, demon);
    tower::give(world, arsenal, demon, kind, count);
    let line = world.resource::<Prose>().line(
        "summon_bound",
        &[("count", &count.to_string()), ("name", demon)],
    );
    (line, true)
}
