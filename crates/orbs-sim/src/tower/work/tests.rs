//! §10.1's loop, driven through a real [`Sim`].
//!
//! Together rather than split across `slot`, `produce` and `triage`: almost
//! every one of these types a command and steps, so what they exercise is the
//! *loop* — which is the thing the split was careful not to break apart.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind};

use super::{PURGE_TICKS, Working, in_flight, occupied};
use crate::Sim;
use crate::parser::{NounKind, Verb};
use crate::tick::Tick;
use crate::tower::node::{Cwd, Name, Nameable, NodeId, NodeIds, children_of};

/// Everything nameable from where the player is standing.
fn here(sim: &mut Sim) -> Vec<String> {
    let cwd = sim.world_mut().resource::<Cwd>().0;
    children_of(sim.world_mut(), cwd)
        .into_iter()
        .filter_map(|node| sim.world_mut().get::<Name>(node).map(|name| name.0.clone()))
        .collect()
}

/// Put a reagent inside `place`, the way `move` does.
///
/// Resolves `place` among the children of where the player is standing,
/// which for the laboratory's instruments is exactly one level.
fn stock(sim: &mut Sim, place: &str, name: &str) {
    let world = sim.world_mut();
    let cwd = world.resource::<Cwd>().0;
    let at = children_of(world, cwd)
        .into_iter()
        .find(|node| world.get::<Name>(*node).is_some_and(|n| n.0 == place))
        .expect("the place is here");
    let id = world.resource_mut::<NodeIds>().issue();
    world.spawn((
        id,
        Name(name.to_owned()),
        Nameable(NounKind::Reagent),
        ChildOf(at),
    ));
}

/// Clear an instrument and wait for the scouring to land.
///
/// Purging is Triage work now (§9, §11.5) rather than instant, so a test
/// that clears and immediately looks inside is reading the instrument
/// mid-scour.
fn scour(sim: &mut Sim, instrument: &str) {
    sim.submit(&format!("purge {instrument}"));
    sim.step();
    sim.step_n(PURGE_TICKS);
}

/// Run one stage: charge the instrument and set it going.
fn stage(sim: &mut Sim, thing: &str, instrument: &str) {
    sim.submit(&format!("move {thing} to {instrument}"));
    sim.step();
    sim.submit(&format!("wield {instrument}"));
    sim.step();
}

/// Take `thing` through `instrument` and collect what comes out.
fn run(sim: &mut Sim, thing: &str, instrument: &str) {
    stage(sim, thing, instrument);
    sim.step_n(20);
    // `empty`, not `siphon` then `purge`: `siphon` is retired (§19) and
    // scouring alone would destroy the product this helper exists to keep.
    // `empty` shelves everything and frees the tool in one move, and
    // `reachable` searches the store, so the next stage finds it.
    sim.submit(&format!("empty {instrument}"));
    sim.step();
}

#[test]
fn purging_a_charged_alembic_takes_what_is_in_it_and_leaves_the_alembic() {
    // An instrument that has something in it — purging an *empty* one proves
    // nothing about what a purge takes.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    stock(&mut sim, "alembic", "phlegm");

    scour(&mut sim, "alembic");

    let laboratory = here(&mut sim);
    assert!(
        laboratory.iter().any(|name| name == "alembic"),
        "the alembic itself was purged: {laboratory:?}"
    );

    sim.submit("attend alembic");
    sim.step();
    assert!(
        here(&mut sim).is_empty(),
        "the phlegm survived a purge of the alembic holding it"
    );
}

#[test]
fn purging_an_instrument_empties_it_rather_than_destroying_it() {
    // Deleting the alembic would leave a laboratory that cannot distil, from a
    // verb §7 calls everyday maintenance. Instruments are safe by being places,
    // not `Protected` — that tier refuses outright and is for the root, the
    // live domains and the dispensary.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim.submit("move sage to mortar_and_pestle");
    sim.step();

    scour(&mut sim, "mortar_and_pestle");

    assert!(
        here(&mut sim)
            .iter()
            .any(|name| name == "mortar_and_pestle"),
        "the mortar was destroyed rather than emptied"
    );

    sim.submit("attend mortar_and_pestle");
    sim.step();
    assert!(here(&mut sim).is_empty(), "it kept its contents");
}

#[test]
fn the_dispensary_refuses_to_be_purged() {
    // Not an instrument but the tower's only source of `sage` and `charcoal`,
    // so one `purge dispensary` used to make the tower unwinnable — against
    // §7's "destruction is a tool, not a trap".
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    scour(&mut sim, "dispensary");

    sim.submit("attend dispensary");
    sim.step();
    let left = here(&mut sim);
    assert!(
        left.iter().any(|name| name == "charcoal"),
        "the tower's only fuel was destroyed: {left:?}"
    );
    assert!(
        left.iter().any(|name| name == "sage"),
        "the tower's only sage was destroyed: {left:?}"
    );
}

#[test]
fn a_live_domain_refuses_to_be_purged_at_all() {
    // The other tier. `Protected` covers the root *and* the domains (§7).
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim.submit("purge laboratory");
    sim.step();

    sim.submit("survey");
    sim.step();
    assert!(
        here(&mut sim).iter().any(|name| name == "alembic"),
        "the laboratory was emptied, taking its instruments with it"
    );
}

#[test]
fn the_laboratory_holds_every_instrument_the_pipeline_needs() {
    // §10.1: four Focus-consuming instruments, the athanor as shared heat,
    // and a dispensary to hold materials between them.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();

    let here = here(&mut sim);
    for instrument in [
        "mortar_and_pestle",
        "balneum_mariae",
        "flask_and_rod",
        "alembic",
        "athanor",
        "dispensary",
    ] {
        assert!(
            here.contains(&instrument.to_owned()),
            "no {instrument}: {here:?}"
        );
    }
    // `crucible` went with §19's rename; `retort` stayed a vessel, which is
    // what keeps `NounKind::Vessel` from being emptied of nouns.
    assert!(!here.contains(&"crucible".to_owned()), "crucible survived");
    assert!(
        here.contains(&"retort".to_owned()),
        "retort is still a vessel"
    );
}

#[test]
fn an_instrument_takes_time_and_lands_by_itself() {
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    stage(&mut sim, "sage", "mortar_and_pestle");

    assert!(sim.working().is_some(), "nothing went in flight");
    sim.step_n(30);
    assert!(sim.working().is_none(), "it never landed");
}

#[test]
fn grinding_sage_yields_ground_sage_and_husks() {
    // §10.1: the instrument leaves both the product and the byproduct in
    // itself, which makes clearing the first move of the next loop.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    stage(&mut sim, "sage", "mortar_and_pestle");
    sim.step_n(30);

    sim.submit("attend mortar_and_pestle");
    sim.step();
    let inside = here(&mut sim);
    assert!(inside.contains(&"ground-sage".to_owned()), "{inside:?}");
    assert!(inside.contains(&"husks".to_owned()), "{inside:?}");
    assert!(!inside.contains(&"sage".to_owned()), "the sage survived");
}

#[test]
fn a_fouled_instrument_refuses_until_it_is_cleared() {
    // The product and byproduct from the last run are still in there, so no
    // recipe matches. This is the loop's clearing step, enforced.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    stage(&mut sim, "sage", "mortar_and_pestle");
    sim.step_n(30);

    // The product and the husks are both still in there, so the multiset
    // matches no recipe and the mortar will not start.
    sim.submit("wield mortar_and_pestle");
    sim.step();
    assert!(sim.working().is_none(), "a fouled mortar started anyway");

    // Clear it, fetch the salt that is still in the dispensary, and it works.
    scour(&mut sim, "mortar_and_pestle");
    stage(&mut sim, "rock-salt", "mortar_and_pestle");
    assert!(sim.working().is_some(), "a cleared mortar still refused");
}

#[test]
fn a_working_instrument_will_not_be_charged_or_restarted() {
    // §10.1's lock: it is what makes different recipes need different
    // scripts.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    stage(&mut sim, "sage", "mortar_and_pestle");

    sim.submit("move rock-salt to mortar_and_pestle");
    sim.step();

    sim.submit("attend mortar_and_pestle");
    sim.step();
    assert!(
        !here(&mut sim).contains(&"rock-salt".to_owned()),
        "the salt was loaded into a working mortar"
    );
}

#[test]
fn an_instrument_being_scoured_refuses_to_be_charged_or_wielded() {
    // §10.1's lock covers a scour as well as a run, and only `Working` was
    // guarded — so a reagent moved in behind a `purge` was despawned by it
    // four ticks later, silently, having spent the one production slot.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim.submit("purge mortar_and_pestle");
    sim.step();

    sim.submit("move sage to mortar_and_pestle");
    sim.step();
    sim.submit("attend mortar_and_pestle");
    sim.step();
    assert!(
        here(&mut sim).is_empty(),
        "a reagent was carried into an instrument being scoured"
    );

    // ...and the sage is still where it was, not swallowed.
    sim.submit("attend laboratory");
    sim.step();
    sim.submit("attend dispensary");
    sim.step();
    assert!(
        here(&mut sim).iter().any(|name| name == "sage"),
        "the sage was consumed by a move that should have been refused"
    );
}

#[test]
fn a_finished_instrument_releases_the_slot_before_it_is_collected() {
    // Otherwise a capacity-1 player who walks away from a finished mortar
    // can never start anything again — a soft-lock reachable in the first
    // ten minutes.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    stage(&mut sim, "sage", "mortar_and_pestle");
    sim.step_n(30);

    assert!(sim.working().is_none(), "the slot is still held");
    stage(&mut sim, "rock-salt", "flask_and_rod");
    // Nothing to make of salt alone, but the *slot* was free to try.
    assert!(
        occupied(sim.world_mut()).is_none(),
        "the pool never freed up"
    );
}

#[test]
fn the_tower_has_one_production_slot_not_one_per_domain() {
    // §11.5 opens at multiplex capacity 1 and §9's invariant 4 reserves it for
    // the whole duration, so working occupies the tower. §10.1's instruments
    // amended the per-pane cap, but the pool stayed global.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    stage(&mut sim, "sage", "mortar_and_pestle");

    sim.submit("attend archive");
    sim.step();
    sim.submit("research");
    sim.step();

    assert_eq!(
        sim.working().map(|working| working.verb),
        Some(Verb::Wield),
        "the second action displaced the first",
    );
}

#[test]
fn progress_is_a_function_of_the_clock() {
    // Stored as an interval rather than a countdown, so `meditate` running
    // hundreds of ticks inside one `step` cannot desynchronise it.
    let working = Working {
        verb: Verb::Wield,
        subject: NodeId::from_raw(0),
        started: Tick::new(10),
        ends: Tick::new(30),
    };
    assert_eq!(working.progress(Tick::new(10)), (0, 20));
    assert_eq!(working.progress(Tick::new(20)), (10, 20));
    assert_eq!(working.progress(Tick::new(30)), (20, 20));
    // Past the end it clamps rather than overrunning the meter.
    assert_eq!(working.progress(Tick::new(99)), (20, 20));
}

#[test]
fn meditating_through_a_stage_lands_it_exactly_once() {
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    stage(&mut sim, "sage", "mortar_and_pestle");

    sim.submit("meditate 60");
    sim.step();

    assert!(sim.working().is_none());
    // A landed stage is the one completion naming what came out of it.
    let landed = sim
        .scrollback()
        .records()
        .iter()
        .filter(|record| record.kind() == RecordKind::Completion)
        .filter(|record| {
            record
                .field(FieldName::Name)
                .is_some_and(|value| value.with_str(|name| name == "ground-sage"))
        })
        .count();
    assert_eq!(landed, 1, "a completion fired more than once");
}

#[test]
fn meditating_stalls_at_a_stage_boundary_rather_than_advancing_the_pipeline() {
    // ROADMAP writes this into the item. Auto-advancing would delete the
    // decision the minigame exists to create, so a long `meditate` lands the
    // running stage and stops — it does not carry the product onward.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    stage(&mut sim, "sage", "mortar_and_pestle");

    sim.submit("meditate 600");
    sim.step();

    sim.submit("attend mortar_and_pestle");
    sim.step();
    let inside = here(&mut sim);
    assert!(
        inside.contains(&"ground-sage".to_owned()),
        "the stage never landed: {inside:?}"
    );
    // Still sitting in the mortar ten minutes later, waiting for a player.
    assert!(sim.working().is_none(), "something advanced by itself");
}

#[test]
fn stopping_an_instrument_leaves_what_it_holds() {
    // §10.1: `stop` cancels with a refund. The inputs stay exactly where
    // they are; what is lost is the time.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    stage(&mut sim, "sage", "mortar_and_pestle");
    assert!(sim.working().is_some());

    sim.submit("stop mortar_and_pestle");
    sim.step();
    assert!(sim.working().is_none(), "it kept working");

    sim.submit("attend mortar_and_pestle");
    sim.step();
    let inside = here(&mut sim);
    assert_eq!(inside, vec!["sage".to_owned()], "the sage was not refunded");
}

#[test]
fn the_next_stage_takes_the_product_and_leaves_the_byproduct() {
    // What retired `siphon` (§19): the per-instrument verbs reach into an idle
    // tool, so the drawing-off step had stopped doing anything.
    //
    // The byproduct still stays behind. Telling product from waste by name
    // would mean the laboratory deciding which reagents are rubbish, and §10.1
    // refuses — every byproduct is some other recipe's input.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    stage(&mut sim, "sage", "mortar_and_pestle");
    sim.step_n(20);

    sim.submit("digest ground-sage");
    sim.step();

    sim.submit("attend balneum_mariae");
    sim.step();
    assert!(
        here(&mut sim).contains(&"ground-sage".to_owned()),
        "the next stage could not reach into the mortar",
    );

    sim.submit("attend mortar_and_pestle");
    sim.step();
    let inside = here(&mut sim);
    assert_eq!(
        inside,
        vec!["husks".to_owned()],
        "the husks should still be fouling it: {inside:?}"
    );
}

#[test]
fn a_working_instrument_will_not_give_up_its_charge() {
    // §10.1's lock covers taking as much as putting: `reachable` skips busy
    // instruments, so a stage cannot be robbed of what it is working on.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    stage(&mut sim, "sage", "mortar_and_pestle");

    sim.submit("digest sage");
    sim.step();
    sim.submit("attend mortar_and_pestle");
    sim.step();
    assert!(
        here(&mut sim).contains(&"sage".to_owned()),
        "a running instrument was robbed of its charge",
    );
}

#[test]
fn a_purge_runs_during_a_brew_because_triage_is_a_different_slot() {
    // §9: *"a 6-minute brew occupies the laboratory's production slot while a
    // 20-second purge can still run in its triage slot."* Folding the two
    // together would refuse a purge because something else is brewing.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();

    // A brew in the production slot...
    stage(&mut sim, "sage", "mortar_and_pestle");
    assert!(sim.working().is_some(), "nothing is brewing");

    // ...and a scouring of a *different* instrument started while it runs.
    // §10.1's lock means the brewing one refuses on its own account.
    sim.submit("move rock-salt to alembic");
    sim.step();
    sim.submit("purge alembic");
    sim.step();

    assert_eq!(
        in_flight(sim.world_mut()).len(),
        1,
        "the purge took a production slot of its own"
    );

    sim.step_n(PURGE_TICKS);
    sim.submit("attend alembic");
    sim.step();
    assert!(
        here(&mut sim).is_empty(),
        "the purge never landed during the brew"
    );
}

#[test]
fn clearing_takes_time_rather_than_happening_on_the_keystroke() {
    // §11.5 puts "purge byproduct" in the Triage band; instant clearing made
    // the loop's opening move free, which a maintenance verb must not be.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim.submit("move sage to mortar_and_pestle");
    sim.step();
    sim.submit("purge mortar_and_pestle");
    sim.step();

    sim.submit("attend mortar_and_pestle");
    sim.step();
    assert!(
        !here(&mut sim).is_empty(),
        "the scouring finished on the keystroke"
    );

    sim.step_n(PURGE_TICKS);
    assert!(here(&mut sim).is_empty(), "the scouring never landed");
}

#[test]
fn the_same_goal_has_two_right_answers_depending_on_what_the_laboratory_holds() {
    // §10.1's exit criterion: the same recipe, two tower states, two different
    // right answers, and the difference readable from the records before you
    // act.
    //
    // Route A spends fresh sage. Route B spends the husks the mortar left
    // behind — free if you have been grinding, unavailable if you have not,
    // and slower. Both end in clarity, and `recall clarity` lists both.

    // --- Route A: fresh sage ------------------------------------------
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    run(&mut sim, "sage", "mortar_and_pestle");
    run(&mut sim, "rock-salt", "mortar_and_pestle");

    sim.submit("move charcoal to athanor");
    sim.step();
    sim.submit("wield athanor");
    sim.step();
    run(&mut sim, "ground-sage", "balneum_mariae");

    sim.submit("move sage-tincture to flask_and_rod");
    sim.step();
    sim.submit("move ground-salt to flask_and_rod");
    sim.step();
    sim.submit("wield flask_and_rod");
    sim.step();
    sim.step_n(20);

    // The draught is left where it was made: the bench is gone (§19), so a
    // finished product sits in the tool that made it until the next tool — or
    // `empty` — takes it.
    sim.submit("attend flask_and_rod");
    sim.step();
    assert!(
        here(&mut sim).contains(&"clarified-draught".to_owned()),
        "route A did not reach the draught"
    );

    // --- Route B: the husks a grind left behind ------------------------
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    // Grinding sage makes husks. Route B is what those husks are *for*.
    stage(&mut sim, "sage", "mortar_and_pestle");
    sim.step_n(20);
    // Straight out of the mortar: the byproduct goes to the next stage by name
    // with nothing drawn off first, and the ground-sage route B does not want
    // is scoured — `purge` doing the one job `empty` does not.
    sim.submit("move husks to balneum_mariae");
    sim.step();
    scour(&mut sim, "mortar_and_pestle");
    run(&mut sim, "rock-salt", "mortar_and_pestle");

    sim.submit("move charcoal to athanor");
    sim.step();
    sim.submit("wield athanor");
    sim.step();
    sim.submit("wield balneum_mariae");
    sim.step();
    sim.step_n(20);
    sim.submit("attend balneum_mariae");
    sim.step();
    assert!(
        here(&mut sim).contains(&"weak-tincture".to_owned()),
        "the husks did not become a tincture"
    );
    sim.submit("attend laboratory");
    sim.step();

    sim.submit("move weak-tincture to flask_and_rod");
    sim.step();
    sim.submit("move ground-salt to flask_and_rod");
    sim.step();
    sim.submit("wield flask_and_rod");
    sim.step();
    sim.step_n(20);
    sim.submit("attend flask_and_rod");
    sim.step();
    assert!(
        here(&mut sim).contains(&"clarified-draught".to_owned()),
        "route B did not reach the same draught"
    );
}

#[test]
fn a_whole_brew_runs_end_to_end_on_one_charcoal() {
    // The pipeline by hand, as a player would type it: four stages, each
    // fouling its instrument, each cleared before the next use.
    //
    // The two heated stages are not adjacent — the unheated `flask_and_rod`
    // sits between the bath and the still — so the efficient play is
    // light → bath → damp → combine → relight → distil, which this runs. A
    // charcoal outlasts a brew, so the damp is an optimisation rather than a
    // requirement, and this is the only test that exercises damp-and-relight.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();

    // Both unheated stages first, while the fire is still unlit — there is
    // no reason to be burning fuel during the grinding.
    stage(&mut sim, "sage", "mortar_and_pestle");
    sim.step_n(20);
    sim.submit("move ground-sage to balneum_mariae");
    sim.step();
    scour(&mut sim, "mortar_and_pestle");

    stage(&mut sim, "rock-salt", "mortar_and_pestle");
    sim.step_n(20);

    // Now light it, and get both heated stages done before it dies.
    sim.submit("move charcoal to athanor");
    sim.step();
    sim.submit("wield athanor");
    sim.step();

    sim.submit("wield balneum_mariae");
    sim.step();
    sim.step_n(14);

    // The bath is done and the flask needs no heat. Damp it — the fuel keeps.
    sim.submit("stop athanor");
    sim.step();

    sim.submit("move sage-tincture to flask_and_rod");
    sim.step();
    sim.submit("move ground-salt to flask_and_rod");
    sim.step();
    sim.submit("wield flask_and_rod");
    sim.step();
    sim.step_n(12);

    // Relight on what was banked, and distil.
    sim.submit("wield athanor");
    sim.step();
    sim.submit("move clarified-draught to alembic");
    sim.step();
    sim.submit("wield alembic");
    sim.step();
    // A distillation is 56 ticks (§19), the longest stage in the pipeline, and
    // this count is the recipe's rather than a round number that happened to
    // be enough.
    sim.step_n(60);

    sim.submit("attend alembic");
    sim.step();
    let inside = here(&mut sim);
    assert!(
        inside.contains(&"clarity".to_owned()),
        "no potion at the end of the pipeline: {inside:?}"
    );
    assert!(
        inside.contains(&"phlegm".to_owned()),
        "distilling left no byproduct: {inside:?}"
    );
}

#[test]
fn every_event_a_spell_can_wait_on_says_where_it_happened() {
    // Eleven emit sites disagreed about where the instrument went — `Name`,
    // `Path` or `Source` — which survives a person reading the log and not a
    // spell, because "wait until the mortar finishes" needs one answer.
    // `FieldName::At` is it, and this stops the next completion omitting it.
    let mut sim = Sim::new(1);
    for line in [
        "attend laboratory",
        "kindle charcoal",
        "grind sage",
        "meditate 30",
        "siphon mortar_and_pestle",
        "purge mortar_and_pestle",
        "meditate 20",
    ] {
        sim.submit(line);
        sim.step();
    }

    // Only the completions that are events. A refusal is not something a spell
    // waits on, so demanding `At` of one asserts a contract nothing needs.
    let events: Vec<String> = sim
        .scrollback()
        .records()
        .iter()
        .filter(|record| record.kind() == orbs_render::RecordKind::Completion)
        .filter(|record| record.field(orbs_render::FieldName::State).is_some())
        .filter_map(|record| match record.field(orbs_render::FieldName::At) {
            Some(orbs_render::Value::Text(at)) => Some(at.to_owned()),
            _ => record
                .field(orbs_render::FieldName::Name)
                .and_then(|value| match value {
                    orbs_render::Value::Text(text) => Some(format!("MISSING At: {text}")),
                    _ => None,
                }),
        })
        .collect();

    assert!(!events.is_empty(), "the brew produced no events at all");
    let missing: Vec<&String> = events
        .iter()
        .filter(|line| line.starts_with("MISSING"))
        .collect();
    assert!(
        missing.is_empty(),
        "completions a spell would wait on, with nowhere to read the place: {missing:?}",
    );
}
