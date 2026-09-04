//! `verify` bare — §8.1's expensive audit, started and landed.
//!
//! The model is [`tower::audit`]; this is the half that
//! takes the production slot and says what it found.
//!
//! # It is Production-class, which is the whole price
//!
//! §8.1: *"`verify --all` is Production-class, and its duration scales with the
//! tower."* So it goes through [`tower::begin`] and answers to `CAPACITY` — an
//! audit means you are not brewing, which is §5.0's *"repairing the rats
//! occupies the laboratory pane for its duration, during which you are not
//! brewing"* applied to the one command that would otherwise be free.
//!
//! **Not the triage slot**, which is where `purge` runs. A scour is the answer
//! to a problem you have already found; an audit is the looking, and §8.1 prices
//! the looking against the making on purpose.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tower::{self, Surface, audit};

/// Where the audit's run hangs — the named room, not the nameless root.
const TOWER: &str = "/tower";

/// Begin a full audit of the tower.
pub(super) fn sweep(world: &mut World) {
    let ticks = audit::ticks_for(held(world), world.resource::<Scrollback>().records().len());

    // **`/tower`, not `tower::root`.** That one is the *filesystem* root and is
    // deliberately nameless (`build::raise`: *"a root without a `Name`
    // contributes nothing and the paths stay `/tower/laboratory`"*), so hanging
    // the run there made `work_busy` read *"the  is busy verifying"* — a refusal
    // naming nothing, which is the one thing that sentence exists to avoid.
    let Some(at) = tower::find_by_path(world, TOWER) else {
        return;
    };
    let Some(id) = world.get::<tower::NodeId>(at).copied() else {
        return;
    };

    // **The tower node holds the run**, because the audit is of the tower rather
    // than of any instrument. `in_flight` queries `Working` wherever it sits, so
    // this counts against the same tower-wide pool a grind does — which is the
    // contention §8.1 is buying.
    if !tower::begin(world, at, Verb::Verify, id, ticks) {
        return;
    }

    let message = world
        .resource::<Prose>()
        .line("verify_sweep_begins", &[("quantity", &ticks.to_string())]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Verify.canonical())
        .text(FieldName::Message, &message)
        .count(FieldName::Remaining, ticks)
        .role(Role::Cost)
        .finish();
}

/// Report what a finished audit found — `land::finish`'s fourth kind of work.
///
/// **A press is the third and this is the fourth**, for the same reason: it
/// makes no material, so `transmute` is wrong, and what it says depends on what
/// it found, so the generic completion sentence is wrong too.
///
/// It **names what is tampered**, which is §8.1's design rule — *"the skill is
/// knowing which surface to inspect, not deciphering an obscure clue"* — so once
/// the audit has been paid for, the answer is the thing itself.
pub fn land_sweep(world: &mut World) {
    let found = tampered(world);
    let message = if found.is_empty() {
        world.resource::<Prose>().line("verify_sweep_sound", &[])
    } else {
        world
            .resource::<Prose>()
            .line("verify_sweep_found", &[("detail", &found.join(", "))])
    };
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Status)
        .text(FieldName::Name, Verb::Verify.canonical())
        .text(
            FieldName::State,
            if found.is_empty() {
                "sound"
            } else {
                "tampered"
            },
        )
        .text(FieldName::Message, &message)
        .role(if found.is_empty() {
            Role::Success
        } else {
            Role::Danger
        })
        .finish();
}

/// Say that a surface is still cooling, and how long is left.
///
/// **A refusal that names the wait**, never a silent no: §6 forbids a bare
/// error, and a player told only *"not yet"* would have no way to plan the one
/// decision this cooldown exists to create.
///
/// **The surface rides `Kind`, and `State` is reserved for a verdict.** That is
/// rule 4 rather than tidiness: `verify`'s answer is `sound` or `tampered`, and
/// a refusal writing `log` into the same field makes the two indistinguishable
/// to `sift`, to the §14 stream and to any test counting verdicts — which is how
/// this was found, by a test that counted two answers where one had been given.
pub(super) fn say_cooling(world: &mut World, surface: Surface, left: u64) {
    let message = world.resource::<Prose>().line(
        "verify_cooling",
        &[("state", surface.word()), ("quantity", &left.to_string())],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Verify.canonical())
        .text(FieldName::Kind, surface.word())
        .text(FieldName::Message, &message)
        .count(FieldName::Remaining, left)
        .role(Role::Cost)
        .finish();
}

/// How many spells the orb is holding — one half of the audit's cost.
fn held(world: &mut World) -> usize {
    world.query::<&tower::spell::Bound>().iter(world).count()
}

/// Everything in the tower that is currently lying, by name.
///
/// **Sorted, and by name rather than by entity.** An ECS query has no order
/// worth relying on, and a report whose rows moved between two runs of one seed
/// would fail the lockstep test that pins a snapshot as a complete description —
/// the same rule `sabotage::drift` follows when it picks a target.
fn tampered(world: &mut World) -> Vec<String> {
    let mut found: Vec<String> = world
        .query::<(Entity, &tower::Name)>()
        .iter(world)
        .map(|(entity, name)| (entity, name.0.clone()))
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|(entity, _)| tower::poisoned(world, *entity))
        .map(|(_, name)| name)
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}
