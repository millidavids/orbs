//! What the orb says when it wakes.
//!
//! DESIGN.md §4 specifies a **status report reflecting real world state**, not a
//! splash screen: the boot text is the tower answering for itself, and every line
//! of it is something the player could have asked for. That is why it is built
//! by walking the world rather than written down — a boot report that could go
//! stale would be a lie the player reads first.
//!
//! §8.1 gives it a second job: *"forgotten bound scripts"* are answered by the
//! **boot report + named in sabotage logs + `verify --all`**. Listing what runs
//! unattended is an ambient defence against automation you no longer remember
//! building, and it costs a line.
//!
//! # It doubles as the scaffold tutorial
//!
//! §15 wants a throwaway onboarding so the gate *"measures the parser rather
//! than the absence of onboarding"* — a tester who does not know a single verb
//! is testing their guesswork, not the vocabulary. So the report ends by naming
//! the verbs.
//!
//! **Names, not sentences.** Rule 6 and §12 keep authored prose in content files,
//! and §19 already set the line: the parser's tables emit facts and
//! `Verb::canonical` is a const table nobody calls a violation. A list of verbs
//! is that same list. The sentence wrapped around it — §4's *"the orb warms to
//! your touch"* — is composed by a content file in Phase 1 from exactly these
//! records.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::node::{Cwd, Name, children_of};
use super::sabotage::poisoned;
use crate::execute::is_live;
use crate::parser::Verb;
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

    // One row per domain, carrying what it holds and whether it is sound. §8.1's
    // `verify` answers the same question one surface at a time; this is the
    // glance that tells you which surface to ask about.
    let domains: Vec<(String, usize, bool)> = children_of(world, root)
        .into_iter()
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

    // §8.1: bound scripts are named here so automation cannot be forgotten. The
    // script engine is Phase 1, so the honest report is that there are none —
    // an absent section would leave a player unable to tell "none" from "not
    // shown", and this line is what a Phase 1 script slots into.
    records
        .push(RecordKind::Status)
        .text(FieldName::Name, "bound")
        .count(FieldName::Quantity, 0)
        .finish();

    // The scaffold tutorial (§15). Facts, not a lesson: these are the words that
    // work, which is the smallest thing that stops the gate measuring a tester's
    // guesswork instead of the vocabulary.
    //
    // *Work*, not *exist*. Six of §6.1's sixteen only acknowledge in Phase 0, and
    // offering those to a tester would spend the gate's most important metric —
    // the dead-end rate — on things nobody built yet. See
    // [`is_live`](crate::execute::is_live) for which, and for the one that is
    // worse than a dead end.
    //
    // A **domain's own** verbs are left out. The boot report is written before
    // the player has gone anywhere, and `grind` is not a word at the tower root
    // — offering it there is the dead end this filter exists to avoid, one step
    // further in. `recall brewing` is what teaches them, from inside the
    // laboratory where they work.
    for verb in Verb::ALL
        .into_iter()
        .filter(|verb| is_live(*verb) && !verb.is_operation())
    {
        let mut entry = records.push(RecordKind::Entry);
        entry = entry.text(FieldName::Name, verb.canonical());
        // `status` and `undo` take nothing, and an empty field is not the same
        // as an absent one: it draws as trailing blanks and speaks as a labelled
        // silence — "kind:" followed by nothing at all.
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
        // §4: a status report reflecting real world state. Built by walking the
        // tower, so it cannot go stale — a boot report that could would be a lie
        // the player reads first.
        let sim = Sim::new(1);
        let named = rows(&sim, RecordKind::Status);
        assert!(named.iter().any(|name| name == "laboratory"));
        assert!(named.iter().any(|name| name == "archive"));
        assert!(named.iter().any(|name| name == "bound"));
    }

    #[test]
    fn it_names_every_working_verb_and_no_others() {
        // §15's scaffold tutorial. A tester who knows no verb at all is testing
        // their guesswork rather than the vocabulary — but a tester sent after a
        // verb nobody has built yet is testing Phase 1, and `bind` would have
        // them read a `sift` of the session log as a success.
        // A domain's own verbs are **also** left out, for the same reason one
        // step further in: the report is written at the tower root, and `grind`
        // is not a word there (§7). Sending a tester after it would be the exact
        // dead end this list exists to avoid.
        let sim = Sim::new(1);
        let listed = rows(&sim, RecordKind::Entry);
        for verb in Verb::ALL {
            let offered = is_live(verb) && !verb.is_operation();
            assert_eq!(
                listed.iter().any(|name| name == verb.canonical()),
                offered,
                "{} is offered by the boot report but {}",
                verb.canonical(),
                if offered {
                    "should be"
                } else {
                    "should not be"
                },
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
