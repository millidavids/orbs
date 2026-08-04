//! The simulation, and the single entry point frontends drive it through.
//!
//! Frontends are *callers*, not hosts: they construct a [`Sim`] and call
//! [`Sim::step`]. No frontend's own scheduler ever drives the world. This is what
//! keeps the Bevy build, the terminal build, and the balance harness running the
//! identical code path — and therefore producing identical results.

use bevy_ecs::prelude::*;
use orbs_render::{Presentation, RecordKind};

use crate::execute::run_pending;
use crate::parser::{Mode, NounKind, ParseLog, ParseRecord, Resolution, Scene, analyse, report};
use crate::rng::Rngs;
use crate::schedule::new_sim_schedule;
use crate::session::{Pending, Scrollback, Skip, Submissions};
use crate::tick::Tick;

/// A complete simulation: the world, its schedule, and its clock.
pub struct Sim {
    world: World,
    /// Commands the player queued, applied before the world moves.
    commands: Schedule,
    schedule: Schedule,
}

impl Sim {
    /// Create a simulation from a master seed, with an empty schedule.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self::with_schedule(seed, |_| {})
    }

    /// Create a simulation and populate its schedule.
    ///
    /// The schedule is built here and never exposed afterwards, so systems cannot
    /// be added behind the sim's back at runtime.
    #[must_use]
    pub fn with_schedule(seed: u64, build: impl FnOnce(&mut Schedule)) -> Self {
        let mut world = World::new();
        world.insert_resource(Rngs::from_seed(seed));
        world.insert_resource(Tick::default());
        // §3's log, nameable from the moment the game starts: the record
        // stream *is* the log, so `peruse orb.log` and `sift <pattern> orb.log`
        // are real commands rather than debug affordances.
        world.insert_resource(Scene::new().with(NounKind::File, crate::execute::LOG));
        world.init_resource::<Scrollback>();
        world.init_resource::<Pending>();
        world.init_resource::<Submissions>();
        world.init_resource::<Skip>();
        world.init_resource::<ParseLog>();

        // Its **own** schedule, run before the caller's. Adding `run_pending`
        // to the same schedule and relying on insertion order would be an
        // ambiguity, not an ordering: Bevy makes no promise about systems with
        // no constraint between them, and a domain system added through `build`
        // could observe the queue either drained or not. A separate pass is
        // unambiguous by construction and needs no set for callers to remember.
        let mut commands = new_sim_schedule();
        commands.add_systems(run_pending);

        let mut schedule = new_sim_schedule();
        build(&mut schedule);

        Self {
            world,
            commands,
            schedule,
        }
    }

    /// Advance the world by exactly one tick.
    ///
    /// The clock advances *before* systems run, so a system observing `Tick::new(1)`
    /// is doing the work of the first tick rather than reporting the tick it just
    /// finished.
    pub fn step(&mut self) {
        self.advance();
        // A command may have asked for time to pass — `meditate 30`. Running it
        // out here rather than in the caller is what keeps the Bevy build, the
        // terminal build and the balance harness passing time identically.
        // `Skip` is capped where it is requested, so this terminates.
        while self.world.resource_mut::<Skip>().take() {
            self.advance();
        }
    }

    /// One tick, exactly.
    fn advance(&mut self) {
        let next = self.world.resource::<Tick>().next();
        self.world.insert_resource(next);
        // What the player asked for, then what the world does about it. Two
        // passes rather than two ordered systems — see `with_schedule`.
        self.commands.run(&mut self.world);
        self.schedule.run(&mut self.world);
    }

    /// Advance by `n` ticks.
    pub fn step_n(&mut self, n: u64) {
        for _ in 0..n {
            self.step();
        }
    }

    /// Take a line the player typed.
    ///
    /// The **second** entry point, and the only other one. Unlike
    /// [`Sim::step`] it does not advance world time: it echoes immediately and
    /// queues any resolved command for the next tick. See
    /// [`session`](crate::session) for why the two clocks are split, and
    /// DESIGN.md §19 for the rule it is measured against.
    ///
    /// A blank line does nothing at all, as in every shell. `report` is right to
    /// refuse silence at its own layer — §6 forbids a bare error — but "the
    /// player pressed Enter on an empty prompt" is not an error to report.
    pub fn submit(&mut self, line: &str) {
        if line.trim().is_empty() {
            return;
        }

        // `analyse` rather than `resolve`: it keeps every scored reading, which
        // is what §6's *"the parser must explain itself"* means in practice and
        // what the Phase 0 gate needs to cluster failures by cause rather than
        // count them. Deterministic and identical work — `resolve` is `analyse`
        // with the candidates dropped.
        let tick = *self.world.resource::<Tick>();
        let analysis = analyse(line, self.world.resource::<Scene>(), Mode::Calm);
        self.world.resource_mut::<ParseLog>().push(ParseRecord::new(
            tick.get(),
            line,
            Mode::Calm,
            &analysis,
        ));
        let resolution = analysis.resolution;

        let mut scrollback = self.world.resource_mut::<Scrollback>();
        let records = scrollback.records_mut();
        records
            .push(RecordKind::Input)
            .text(orbs_render::FieldName::Message, line)
            .finish();
        report(line, &resolution, records);

        self.world.resource_mut::<Submissions>().push(tick, line);
        if let Resolution::Resolved { intent, .. } = resolution {
            self.world.resource_mut::<Pending>().push(intent);
        }
    }

    /// Everything the player has said and been told.
    #[must_use]
    pub fn scrollback(&self) -> &Scrollback {
        self.world.resource::<Scrollback>()
    }

    /// Commands resolved but not yet run.
    #[must_use]
    pub fn pending(&self) -> &Pending {
        self.world.resource::<Pending>()
    }

    /// The register the orb is currently speaking in.
    #[must_use]
    pub fn register(&self) -> Presentation {
        self.world.resource::<Scrollback>().records().register()
    }

    /// Change the register everything said from now on is spoken in.
    ///
    /// DESIGN.md §3's high-threat tonal register. In Phase 2 this is driven by
    /// threat rather than set by hand; until the threat system exists it is
    /// reachable directly, which is what makes the three typefaces and §3's
    /// corruption exemption something a person can see rather than something an
    /// example prints.
    ///
    /// Nothing about the *content* changes — §3: *"the renderer corrupts it; the
    /// model records it faithfully."* Only the face a frontend draws with does,
    /// and log lines refuse the eldritch one however this is set.
    pub fn set_register(&mut self, register: Presentation) {
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .set_register(register);
    }

    /// Every reading the parser scored this session.
    ///
    /// §6: *"the parser must explain itself."* This is the raw material for the
    /// Phase 0 gate, which acts on the clustering of failures by cause rather
    /// than on the aggregate — 8 testers over 15 minutes has wide confidence
    /// intervals, so 84% against 86% is noise and *which* readings lost is not.
    #[must_use]
    pub fn parse_log(&self) -> &ParseLog {
        self.world.resource::<ParseLog>()
    }

    /// Every line submitted, with the tick it landed on. A replay needs this and
    /// the seed, and nothing else.
    #[must_use]
    pub fn submissions(&self) -> &Submissions {
        self.world.resource::<Submissions>()
    }

    /// The current world time.
    #[must_use]
    pub fn tick(&self) -> Tick {
        *self.world.resource::<Tick>()
    }

    /// The master seed this world was built from.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.world.resource::<Rngs>().master_seed()
    }

    /// Read-only access to the world, for frontends rendering current state.
    #[must_use]
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Mutable access to the world, for setup: inserting resources, spawning the
    /// initial tower, applying a loaded save.
    ///
    /// This is a deliberate escape hatch. Mutating the world *between* steps is
    /// fine; anything that makes two runs from the same seed diverge is not.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::RngStream;
    use rand::Rng;

    #[derive(Resource, Debug, Default, PartialEq, Clone)]
    struct Trace {
        ticks_seen: Vec<u64>,
        rolls: Vec<u64>,
    }

    fn record(mut trace: ResMut<Trace>, tick: Res<Tick>, mut rngs: ResMut<Rngs>) {
        trace.ticks_seen.push(tick.get());
        let roll: u64 = rngs.stream(RngStream::Threat).random();
        trace.rolls.push(roll);
    }

    fn sim_with_recorder(seed: u64) -> Sim {
        let mut sim = Sim::with_schedule(seed, |s| {
            s.add_systems(record);
        });
        sim.world_mut().init_resource::<Trace>();
        sim
    }

    #[test]
    fn starts_at_tick_zero() {
        assert_eq!(Sim::new(1).tick(), Tick::new(0));
    }

    #[test]
    fn step_advances_the_clock() {
        let mut sim = Sim::new(1);
        sim.step();
        assert_eq!(sim.tick(), Tick::new(1));
        sim.step_n(9);
        assert_eq!(sim.tick(), Tick::new(10));
    }

    #[test]
    fn systems_observe_the_tick_they_are_working() {
        let mut sim = sim_with_recorder(1);
        sim.step_n(3);
        assert_eq!(sim.world().resource::<Trace>().ticks_seen, vec![1, 2, 3]);
    }

    #[test]
    fn identical_seeds_produce_identical_runs() {
        // The load-bearing property: replay, offline/online parity, and the
        // balance harness matching the live game all reduce to this test.
        let mut a = sim_with_recorder(0xC0FFEE);
        let mut b = sim_with_recorder(0xC0FFEE);
        a.step_n(256);
        b.step_n(256);
        assert_eq!(
            a.world().resource::<Trace>(),
            b.world().resource::<Trace>(),
            "identical seeds diverged"
        );
    }

    #[test]
    fn different_seeds_produce_different_runs() {
        let mut a = sim_with_recorder(1);
        let mut b = sim_with_recorder(2);
        a.step_n(32);
        b.step_n(32);
        assert_ne!(
            a.world().resource::<Trace>().rolls,
            b.world().resource::<Trace>().rolls
        );
    }

    #[test]
    fn stepping_is_resumable() {
        // Offline accrual replays the same schedule in bulk on return; running
        // 100 ticks in one call must equal 100 ticks in two.
        let mut whole = sim_with_recorder(42);
        whole.step_n(100);

        let mut split = sim_with_recorder(42);
        split.step_n(60);
        split.step_n(40);

        assert_eq!(
            whole.world().resource::<Trace>(),
            split.world().resource::<Trace>()
        );
    }
}
