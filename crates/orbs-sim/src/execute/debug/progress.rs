//! The words that hand a tester progress the game would make them earn.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use super::door::{said, voiced};
use super::word::{Asking, LEARN};
use crate::content::{Progression, Prose};
use crate::tower;

/// Hold a mastery node without earning the experience for it.
///
/// Straight into `Taken`, skipping the threshold; everything downstream is
/// real. A marker is refused, a state the game cannot reach.
pub(super) fn take(world: &mut World, id: Option<&str>) {
    let real: Vec<String> = world
        .resource::<Progression>()
        .ley_line()
        .iter()
        .flat_map(|station| &station.nodes)
        .cloned()
        .collect();
    let message = match id {
        Some(id) if real.iter().any(|node| node == id) => {
            world.resource_mut::<tower::Taken>().hold(id);
            format!("the orb holds {id}")
        }
        Some(id) => format!("{id} grants nothing. one of: {}", real.join(", ")),
        None => format!("nodes: {}", real.join(", ")),
    };
    said(world, &message);
}

/// Put renown at a total, for a tester, and say the title it crossed.
///
/// Sets rather than adds, so one word goes both ways. Bare, it says where the
/// tower stands.
pub(super) fn renown(world: &mut World, total: Asking) {
    let standing = |world: &World| {
        let held = world.resource::<tower::Renown>().get();
        let title = tower::renown::rank(world);
        format!(
            "renown {held}, and the tower is {}",
            title.unwrap_or_else(|| "nothing yet".to_owned()),
        )
    };
    let message = match total {
        Asking::Set(total) => {
            tower::renown::set(world, total);
            format!("renown is {total}")
        }
        Asking::Where => standing(world),
        // Refused, and nothing touched, as `debug_take` and `debug_reach` answer
        // a bad argument. Reading it as nought zeroed the total and said so —
        // destroying what the tester had built, and looking like it had worked.
        Asking::Unreadable => format!("that is no number. {}", standing(world)),
    };
    said(world, &message);
}

/// Reach a mastery station without doing its deed.
///
/// Every earlier station on the line too, and the room the line stands in,
/// which another line opens. What each opens is opened and said.
pub(super) fn reach(world: &mut World, id: Option<&str>) {
    let stations: Vec<(String, String, Vec<String>)> = world
        .resource::<Progression>()
        .mastery()
        .iter()
        .map(|milestone| {
            (
                milestone.domain.clone(),
                milestone.id.clone(),
                milestone.opens.clone(),
            )
        })
        .collect();
    let ids: Vec<&str> = stations.iter().map(|(_, id, _)| id.as_str()).collect();
    let message = match id {
        Some(wanted) if ids.contains(&wanted) => {
            let domain = stations
                .iter()
                .find(|(_, id, _)| id == wanted)
                .map(|(domain, _, _)| domain.clone())
                .unwrap_or_default();
            unseal(world, &domain, &stations);
            if walk(world, &stations, &domain, wanted) {
                format!("the orb reaches {wanted}")
            } else {
                format!("{wanted} is reached already")
            }
        }
        Some(wanted) => format!("{wanted} is no station. one of: {}", ids.join(", ")),
        None => format!("stations: {}", ids.join(", ")),
    };
    said(world, &message);
}

/// Reach every station on `line` up to and including `wanted`, and say
/// whether any was not reached already.
fn walk(
    world: &mut World,
    stations: &[(String, String, Vec<String>)],
    line: &str,
    wanted: &str,
) -> bool {
    let mut reached_any = false;
    for (line_domain, station, opens) in stations {
        if line_domain != line {
            continue;
        }
        if !world.resource::<tower::mastery::Reached>().has(station) {
            tower::mastery::reach(world, line_domain, station, opens);
            reached_any = true;
        }
        if station == wanted {
            break;
        }
    }
    reached_any
}

/// Open the room `domain`, and whatever had to open first for it to.
///
/// The chain, outermost first, since a room's key may stand in a shut room.
/// A key on the tower's own line is applied directly, not credited, so no
/// experience moves.
fn unseal(world: &mut World, domain: &str, stations: &[(String, String, Vec<String>)]) {
    let mut order: Vec<(String, String)> = Vec::new();
    let mut wanted = domain.to_owned();
    // Bounded by the rooms there are, so no authored chain can spin here.
    for _ in 0..=tower::DOMAINS.len() {
        if world.resource::<tower::Opened>().is_open(&wanted) {
            break;
        }
        let key = tower::domain_key(&wanted);
        let Some((line, id, _)) = stations.iter().find(|(_, _, opens)| opens.contains(&key)) else {
            tower::open(world, &key);
            break;
        };
        order.push((line.clone(), id.clone()));
        wanted = line.clone();
    }
    for (line, wanted) in order.into_iter().rev() {
        walk(world, stations, &line, &wanted);
    }
}

/// Learn a secret the lens would otherwise have to find.
///
/// Runs now, like `debug_spell`, so the next dump line sees it.
pub(super) fn learn(world: &mut World, line: &str, wanted: Option<&str>) {
    let found = tower::learn(world, wanted);
    let message = match &found {
        Some(name) => world
            .resource::<Prose>()
            .line("probe_found", &[("name", name)]),
        None => format!("{line}: no such secret, or every one is known"),
    };
    let role = if found.is_some() {
        Role::Success
    } else {
        Role::Cost
    };
    voiced(world, LEARN, &message, role);
}
