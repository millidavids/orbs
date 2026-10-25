//! What the orb says when it wakes.
//!
//! §4's status report on real world state, not a splash screen: built by
//! walking the world, since a report that could go stale is a lie the player
//! reads first. It names what is bound, §8.1's defence against forgotten
//! automation, and ends with the verbs, §15's scaffold tutorial. Names, not
//! sentences; the prose around them is a content file's.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::node::{Cwd, Name, children_of};
use super::sabotage::poisoned;
use crate::rng::Rngs;
use crate::session::Scrollback;
use crate::tick::Tick;

/// Report the tower's condition.
///
/// Called once, at the end of `Sim::new`, before any tick — so what a player
/// reads first is the world as it was handed to them.
pub fn report(world: &mut World) {
    let root = world.resource::<Cwd>().0;
    let seed = world.resource::<Rngs>().master_seed();
    let tick = world.resource::<Tick>().get();

    // One row per open domain — the glance that says which surface to `verify`.
    // A sealed room is left to the rail's dark box.
    let domains: Vec<(String, usize, bool)> = children_of(world, root)
        .into_iter()
        .filter(|branch| super::sealed_room_of(world, *branch).is_none())
        .map(|branch| {
            let name = world
                .get::<Name>(branch)
                .map_or_else(String::new, |name| name.0.clone());
            let holdings = children_of(world, branch);
            let tampered = holdings.iter().any(|node| poisoned(world, *node));
            (name, holdings.len(), tampered)
        })
        .collect();

    let mut scrollback = world.resource_mut::<Scrollback>();
    let records = scrollback.records_mut();

    records
        .push(RecordKind::Status)
        .text(FieldName::Name, "orb")
        .count(FieldName::Tick, tick)
        .count(FieldName::Quantity, seed)
        .text(FieldName::State, "cold start")
        .finish();

    for (name, holdings, tampered) in domains {
        records
            .push(RecordKind::Status)
            .text(FieldName::Name, &name)
            .count(FieldName::Quantity, holdings as u64)
            .text(FieldName::State, if tampered { "tampered" } else { "ok" })
            .role(if tampered {
                Role::Danger
            } else {
                Role::Success
            })
            .finish();
    }

    // §8.1: bound scripts are counted here so automation cannot be forgotten.
    let bound = crate::tower::spell::held(world).len();
    let mut scrollback = world.resource_mut::<Scrollback>();
    let records = scrollback.records_mut();
    records
        .push(RecordKind::Status)
        .text(FieldName::Name, "bound")
        .count(FieldName::Quantity, bound as u64)
        .finish();

    // The scaffold tutorial (§15): the words that work, through
    // `execute::offered`, which `recall`'s overview shares and argues.
    let offered = crate::execute::offered(world);
    let mut scrollback = world.resource_mut::<Scrollback>();
    let records = scrollback.records_mut();
    for verb in offered {
        let mut entry = records.push(RecordKind::Entry);
        entry = entry.text(FieldName::Name, verb.canonical());
        // An empty field is not an absent one: it would speak as "kind:" and
        // then nothing.
        if !verb.signature_label().is_empty() {
            entry = entry.text(FieldName::Kind, verb.signature_label());
        }
        entry.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;
    use crate::execute::{is_gated, is_live, offered};
    use crate::parser::Verb;

    fn rows(sim: &Sim, kind: RecordKind) -> Vec<String> {
        sim.scrollback()
            .records()
            .iter()
            .filter(|record| record.kind() == kind)
            .filter_map(|record| record.field(FieldName::Name))
            .map(|value| value.with_str(str::to_owned))
            .collect()
    }

    #[test]
    fn the_orb_reports_the_world_it_actually_has() {
        // §4: a status report reflecting real world state.
        let sim = Sim::new(1);
        let named = rows(&sim, RecordKind::Status);
        assert!(named.iter().any(|name| name == "laboratory"));
        assert!(named.iter().any(|name| name == "archive"));
        assert!(named.iter().any(|name| name == "bound"));
    }

    #[test]
    fn it_names_every_working_verb_and_no_others() {
        // Live, ungated, and unanchored, since the report is written at the
        // tower root, where `grind` is not a word (§7).
        let sim = Sim::new(1);
        let listed = rows(&sim, RecordKind::Entry);
        // The rule restated as well as the shared function compared: asserting
        // only against `execute::offered` would prove the report calls what it
        // calls.
        let shared: Vec<&str> = offered(sim.world())
            .into_iter()
            .map(Verb::canonical)
            .collect();
        assert_eq!(shared, listed, "the report and `execute::offered` disagree");

        for verb in Verb::ALL {
            let wanted = is_live(verb) && verb.anchor().is_none() && !is_gated(verb, sim.world());
            assert_eq!(
                listed.iter().any(|name| name == verb.canonical()),
                wanted,
                "{} is offered by the boot report but {}",
                verb.canonical(),
                if wanted { "should be" } else { "should not be" },
            );
        }
    }

    #[test]
    fn a_tampered_domain_says_so_before_anything_is_typed() {
        // §8.1's forgotten-automation defence, one surface up: the first thing a
        // player sees names the surface worth inspecting.
        let mut sim = Sim::new(1);
        let laboratory = children_of(sim.world(), sim.world().resource::<Cwd>().0)[0];
        let log = children_of(sim.world(), laboratory)[5];
        super::super::sabotage::poison(sim.world_mut(), log);

        sim.world_mut()
            .resource_mut::<Scrollback>()
            .records_mut()
            .clear();
        report(sim.world_mut());

        assert!(
            sim.scrollback()
                .records()
                .iter()
                .any(|record| record.role() == Role::Danger),
            "a tampered tower booted clean",
        );
    }
}
