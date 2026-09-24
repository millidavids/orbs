//! The intervals themselves: what is in flight, and what may start.
//!
//! Stored as an interval, not a countdown: §8 wants in-flight actions to be
//! *"first-class serialisable entities with start and completion ticks"*, and
//! comparing against the clock is idempotent, survives `meditate` running
//! hundreds of ticks inside one `step`, and makes the progress fraction a pure
//! function of the tick.
//!
//! Two pools, and two types. §9 reserves the production slot for a manual action
//! and grants a per-pane triage slot beside it so short work still runs during a
//! brew. [`Triaging`] is a separate component rather than a flag on [`Working`]
//! because [`in_flight`] and [`CAPACITY`] must not see it: one shared type means
//! every counter has to remember to filter, and the first that forgot would
//! refuse a purge during a brew.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tick::Tick;
use crate::tower::node::{Name, NodeId};

// `DECOCT_TICKS` lived here. It went with `decoct` (§19): a brew is no longer
// one duration but four, and each is a recipe's own `ticks` in
// `content/recipes.toml` — which is where the balance CLI will sweep them, and
// where a writer can see them all at once instead of hunting constants.

/// How long a decipherment takes. Shorter than a brew — §10 makes archive the
/// domain played most, and the one the player returns to between other work.
pub const DIVINE_TICKS: u64 = 12;

/// How long clearing an instrument takes.
///
/// §11.5's Triage band is 10–30 s and this sits well under it: at the band's own
/// numbers, clearing four instruments is two minutes of a loop whose point is
/// not feeling like a chore. A placeholder the balance CLI sweeps, and the one
/// most likely to move once a whole brew can be timed end to end.
pub const PURGE_TICKS: u64 = 4;

/// How many production actions may be in flight at once.
///
/// §11.5's opening multiplex capacity. Raising it is a progression unlock rather
/// than a tuning knob — §9 keeps panes and capacity separate.
pub const CAPACITY: usize = 1;

/// Work in progress.
#[derive(Component, Debug, Clone, Copy)]
pub struct Working {
    /// What is being done.
    pub verb: Verb,
    /// What it is being done to.
    pub subject: NodeId,
    /// When it started.
    pub started: Tick,
    /// When it lands.
    pub ends: Tick,
}

impl Working {
    /// Ticks elapsed and ticks total, for a meter.
    #[must_use]
    pub fn progress(&self, now: Tick) -> (u64, u64) {
        let total = self.ends.get().saturating_sub(self.started.get());
        let done = now.get().saturating_sub(self.started.get()).min(total);
        (done, total)
    }

    /// Whether `now` has reached the end.
    #[must_use]
    pub const fn finished(&self, now: Tick) -> bool {
        now.get() >= self.ends.get()
    }
}

/// Short work, running beside a production action.
///
/// §9 grants one triage slot per pane alongside the production slot, so a
/// 6-minute brew and a 20-second purge run together. `purge byproduct` is the
/// only occupant so far.
#[derive(Component, Debug, Clone, Copy)]
pub struct Triaging {
    /// When it started.
    pub started: Tick,
    /// When it lands.
    pub ends: Tick,
}

impl Triaging {
    /// Whether `now` has reached the end.
    #[must_use]
    pub const fn finished(&self, now: Tick) -> bool {
        now.get() >= self.ends.get()
    }
}

/// Whether the tower's production slot is free, and what holds it.
#[must_use]
pub fn occupied(world: &mut World) -> Option<(Verb, Entity)> {
    in_flight(world).into_iter().next()
}

/// Everything in flight, in insertion order.
///
/// §9's per-pane cap was amended in Phase 1 so the laboratory may hold as many
/// production actions as it has instruments, but they come from the same
/// tower-wide pool, which is what this counts. At [`CAPACITY`] 1 that is one
/// action anywhere, so starting the mortar means you are not deciphering.
#[must_use]
pub fn in_flight(world: &mut World) -> Vec<(Verb, Entity)> {
    world
        .query::<(Entity, &Working)>()
        .iter(world)
        .map(|(entity, working)| (working.verb, entity))
        .collect()
}

/// Begin work at `place`, on `subject`.
///
/// Refuses when the production slot is taken, naming what holds it — §5.0's
/// *"repairing the rats occupies the laboratory pane for its duration, during
/// which you are not brewing"* only bites if the refusal says so.
pub fn begin(world: &mut World, place: Entity, verb: Verb, subject: NodeId, ticks: u64) -> bool {
    // An instrument already busy refuses on its own account, whatever the pool
    // looks like — §10.1's lock is what makes different recipes need different
    // scripts. Scouring counts: starting a run on an instrument a purge is about
    // to empty loses the reagent and the slot both.
    if let Some(why) = busy(world, place) {
        refuse_busy(world, verb, place, why);
        return false;
    }

    if in_flight(world).len() >= CAPACITY
        && let Some((holder, at)) = occupied(world)
    {
        let name = world
            .get::<Name>(at)
            .map_or_else(String::new, |name| name.0.clone());
        // The fields carry the facts (rule 4); the sentence is authored in
        // `content/prose.toml` (rule 6). Without it this drew as three bare
        // values — `decoct decoct laboratory` — naming what held the slot
        // without ever saying the player had been refused.
        let message = world.resource::<Prose>().line(
            "work_busy",
            &[("source", &name), ("state", holder.participle())],
        );
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, verb.canonical())
            .text(FieldName::State, holder.canonical())
            .text(FieldName::Source, &name)
            .text(FieldName::Message, &message)
            .role(Role::Cost)
            .finish();
        return false;
    }

    // Speed is read here and nowhere else, following heat: §10.1 checks the
    // athanor when a run *begins* and lets it finish even if the fire dies. A
    // run started inside a quickened window stays short, which keeps `Working`
    // an interval set once — what `meditate` idempotence rests on. Per tick it
    // would be the countdown §19 refused, wearing a multiplier.
    let ticks = super::quicken::hastened(world, place, ticks);
    // `haste` is the orb's speed, never the tool's (§19): a run a *spell* issued
    // lands sooner by the tiers taken, and a player's own run does not. Read
    // here with the charm, and for its reason — an interval set once.
    let ticks = if bidder(world).is_some() {
        let sooner = crate::tower::grant::haste_percent(world);
        (ticks.saturating_mul(100u64.saturating_sub(sooner)) / 100).max(1)
    } else {
        ticks
    };
    let now = *world.resource::<Tick>();
    world.entity_mut(place).insert(Working {
        verb,
        subject,
        started: now,
        ends: Tick::new(now.get().saturating_add(ticks)),
    });
    // Who asked for this run, carried on the instrument so the completion can be
    // credited to them ticks later: a spell charges the mortar and the mortar
    // yields on its own schedule, in a different system.
    //
    // Set or cleared, never just set. An insert with no matching clear left the
    // credit on a cancelled run, so the player's own next run on that tool
    // inherited it and the transcript filtered their completion out entirely —
    // they typed a command, waited twenty seconds, and nothing appeared.
    match bidder(world) {
        Some(spell) => {
            world.entity_mut(place).insert(Bidden(spell));
        }
        None => {
            world.entity_mut(place).remove::<Bidden>();
        }
    }
    true
}

/// Which spell asked for the run an instrument is doing, if a spell did.
///
/// Removed when the run lands — see `land::finish`. Nothing reads it in
/// between: it exists only to survive the gap between charging an instrument and
/// its completion arriving.
#[derive(Component, Debug, Clone)]
pub struct Bidden(pub String);

/// The spell currently being credited for what the world emits, if any.
pub(crate) fn bidder(world: &World) -> Option<String> {
    world
        .resource::<Scrollback>()
        .records()
        .attributed()
        .map(ToOwned::to_owned)
}

/// Why an instrument will not accept a command right now.
///
/// Both states, always together. `Working` was guarded for and `Triaging` was
/// not, so a scour on a charged instrument could despawn the inputs of a run
/// that began in between — reagent gone, nothing produced, the slot spent.
/// Returning them from one function stops the next caller checking half.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Busy {
    /// A run is in flight.
    Working,
    /// A scour is in flight.
    Scouring,
}

impl Busy {
    /// What the instrument is doing, as a word output can use.
    ///
    /// Public because the script runner reports a *wait* rather than a refusal
    /// and still has to say what it is waiting on — see `tower::spell::block`.
    /// The same word either way: a player and a spell looking at one busy mortar
    /// should not be told two different things about it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        self.state()
    }

    /// The `State` field value.
    const fn state(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Scouring => "scouring",
        }
    }

    /// The prose key for the refusal.
    const fn key(self) -> &'static str {
        match self {
            Self::Working => "wield_busy",
            Self::Scouring => "purge_already",
        }
    }
}

/// Whether `place` has something in flight.
#[must_use]
pub fn busy(world: &World, place: Entity) -> Option<Busy> {
    if world.get::<Working>(place).is_some() {
        Some(Busy::Working)
    } else if world.get::<Triaging>(place).is_some() {
        Some(Busy::Scouring)
    } else {
        None
    }
}

/// Refuse `verb` because `place` is mid-something, and say so.
///
/// One shape, one site. Hand-built at four call sites, three of them disagreed:
/// `purge`'s put the instrument in `Name` with no `Path`, `move`'s and `wield`'s
/// put the *verb* in `Name`. Rule 4 makes fields what `sift`, pipes and §14's
/// linearisation read, so `sift working orb.log` returned rows whose columns
/// meant different things.
pub fn refuse_busy(world: &mut World, verb: Verb, place: Entity, why: Busy) {
    let name = world
        .get::<Name>(place)
        .map_or_else(String::new, |name| name.0.clone());
    let message = world
        .resource::<Prose>()
        .line(why.key(), &[("name", &name), ("path", &name)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Path, &name)
        .text(FieldName::State, why.state())
        .text(FieldName::Message, &message)
        .role(Role::Cost)
        .finish();
}

/// Push one completion record about an instrument.
/// Say something happened, naming **what** and **where** separately.
///
/// `at` is not optional and not derivable from `name`, which is the whole point:
/// callers passed the instrument here sometimes and the *product* other times
/// (`siphon` says `ground-sage`), so nothing downstream could tell which. A
/// spell asking *"has the mortar finished?"* needs one field with one meaning —
/// see [`FieldName::At`].
pub(super) fn say(world: &mut World, name: &str, at: &str, state: &str, message: &str, role: Role) {
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, name)
        .text(FieldName::At, at)
        .text(FieldName::State, state)
        .text(FieldName::Message, message)
        .role(role)
        .finish();
}

/// Cancel a working instrument, leaving what it holds.
///
/// §10.1: *"a running tool is locked — `stop` cancels with a refund, or you
/// wait."* The refund is that the inputs stay exactly where they are; what is
/// lost is the time, which at capacity 1 was the tower's only slot.
pub fn stop(world: &mut World, place: Entity) -> bool {
    let name = world
        .get::<Name>(place)
        .map_or_else(String::new, |name| name.0.clone());
    // Asked once. It used to encode the answer as a prose key and recover it
    // with `key == "stop_done"`, so renaming the key in `content/prose.toml`
    // would leave `stopped` false for ever — every stop reporting `idle` while
    // the component *was* removed, with no compiler
    // complaint and no test on the returned bool.
    let stopped = world.get::<Working>(place).is_some();
    if stopped {
        world.entity_mut(place).remove::<Working>();
        // The run is over, so its credit is spent. Left behind, the next run on
        // this instrument — the player's own — would be attributed to whichever
        // spell started the one that was cancelled.
        world.entity_mut(place).remove::<Bidden>();
    }
    let (key, state, role) = if stopped {
        ("stop_done", "stopped", Role::Cost)
    } else {
        ("stop_idle", "idle", Role::Normal)
    };
    let message = world.resource::<Prose>().line(key, &[("name", &name)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Stop.canonical())
        .text(FieldName::Path, &name)
        .text(FieldName::State, state)
        .text(FieldName::Message, &message)
        .role(role)
        .finish();
    stopped
}
