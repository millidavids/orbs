//! What the orb says when it wakes.
//!
//! §4's status report on real world state, not a splash screen: built by
//! walking the world, since a report that could go stale is a lie the player
//! reads first. It names what is bound, §8.1's defence against forgotten
//! automation, and ends with the verbs, §15's scaffold tutorial, and a line
//! composed from the rows (§19, *Both tongues*).

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::node::{Cwd, Name, children_of};
use super::sabotage::poisoned;
use crate::content::Prose;
use crate::session::Scrollback;

/// Report the tower's condition.
///
/// Called once, at the end of `Sim::new`, before any tick — so what a player
/// reads first is the world as it was handed to them. Every row is a name and
/// one value, so the run draws as readings, `§4`'s leaders. The tick is always
/// nought here and the seed is `status`'s, so neither is said.
pub fn report(world: &mut World) {
    let root = world.resource::<Cwd>().0;

    // One row per open domain — the glance that says which surface to `verify`.
    // A sealed room is left to the rail's dark box.
    let domains: Vec<(String, bool)> = children_of(world, root)
        .into_iter()
        .filter(|branch| super::sealed_room_of(world, *branch).is_none())
        .map(|branch| {
            let name = world
                .get::<Name>(branch)
                .map_or_else(String::new, |name| name.0.clone());
            let tampered = children_of(world, branch)
                .iter()
                .any(|node| poisoned(world, *node));
            (name, tampered)
        })
        .collect();
    let warned = domains.iter().filter(|(_, tampered)| *tampered).count();
    // A shut grimoire is the rail's too: its name is earned.
    let spells = super::grimoire(world)
        .filter(|grimoire| super::sealed_room_of(world, *grimoire).is_none())
        .map(|grimoire| written(world, grimoire));

    let mut scrollback = world.resource_mut::<Scrollback>();
    let records = scrollback.records_mut();

    records
        .push(RecordKind::Status)
        .text(FieldName::Name, "orb")
        .text(FieldName::State, "cold start")
        .finish();

    for (name, tampered) in domains {
        records
            .push(RecordKind::Status)
            .text(FieldName::Name, &name)
            .text(FieldName::State, if tampered { "tampered" } else { "ok" })
            .role(if tampered {
                Role::Danger
            } else {
                Role::Success
            })
            .finish();
    }
    if let Some(spells) = spells {
        records
            .push(RecordKind::Status)
            .text(FieldName::Name, super::GRIMOIRE)
            .count(FieldName::Quantity, spells as u64)
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

    // §4's closing line, from the rows above: a warning counted for each room
    // found tampered.
    let prose = world.resource::<Prose>();
    let closing = match warned {
        0 => prose.line("boot_warm", &[]),
        many => prose.line(
            &prose.counted_key("boot_warned", many as u64),
            &[("count", &many.to_string())],
        ),
    };
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Message)
        .text(FieldName::Message, &closing)
        .finish();
}

/// How many spells the grimoire holds that a player's would. A debug build
/// shelves the tester spells beside them, and the report counted 22 to a
/// player's one.
fn written(world: &World, grimoire: Entity) -> usize {
    let testers = testers();
    children_of(world, grimoire)
        .into_iter()
        .filter(|spell| {
            world
                .get::<Name>(*spell)
                .is_none_or(|name| !testers.contains(&name.0))
        })
        .count()
}

/// The tester spells' file names.
#[cfg(debug_assertions)]
fn testers() -> Vec<String> {
    crate::execute::dev_spells()
        .iter()
        .map(|(name, _)| crate::content::with_extension(name))
        .collect()
}

/// A release build shelves none.
#[cfg(not(debug_assertions))]
const fn testers() -> Vec<String> {
    Vec::new()
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
        let warned = sim.prose().line("boot_warned_one", &[("count", "1")]);
        assert!(
            sim.scrollback().records().iter().any(|record| record
                .field(FieldName::Message)
                .is_some_and(|said| said.with_str(|text| text == warned))),
            "the closing line did not count the tampered room",
        );
    }

    #[test]
    fn the_report_closes_warm_and_counts_the_grimoire() {
        let sim = Sim::new(1);
        let records = sim.scrollback().records();
        let last = records.iter().last().expect("a report");
        assert_eq!(
            last.field(FieldName::Message)
                .map(|said| said.with_str(str::to_owned)),
            Some(sim.prose().line("boot_warm", &[])),
        );
        // What a player's grimoire holds, whatever this build shelved beside it.
        let shipped = sim
            .world()
            .resource::<crate::content::Spells>()
            .iter()
            .count();
        assert!(shipped > 0, "the grimoire holds no spells");
        assert_eq!(grimoire_row(&sim), Some(shipped.to_string()));
    }

    #[test]
    fn a_shut_grimoire_is_not_named() {
        let sim = Sim::sealed(1);
        let grimoire = super::super::grimoire(sim.world()).expect("a grimoire");
        assert!(
            super::super::sealed_room_of(sim.world(), grimoire).is_some(),
            "the sealed tower opened its grimoire, so this proves nothing",
        );
        assert_eq!(grimoire_row(&sim), None);
    }

    /// The report's grimoire count, if it has a row.
    fn grimoire_row(sim: &Sim) -> Option<String> {
        sim.scrollback()
            .records()
            .iter()
            .filter(|record| record.kind() == RecordKind::Status)
            .find(|record| {
                record
                    .field(FieldName::Name)
                    .is_some_and(|name| name.with_str(|text| text == super::super::GRIMOIRE))
            })
            .and_then(|record| {
                record
                    .field(FieldName::Quantity)
                    .map(|count| count.with_str(str::to_owned))
            })
    }
}
