//! The simulation, and the single entry point frontends drive it through.
//!
//! Frontends are *callers*, not hosts: they construct a [`Sim`] and call
//! [`Sim::step`]. No frontend's own scheduler ever drives the world. This is what
//! keeps the Bevy build, the terminal build, and the balance harness running the
//! identical code path — and therefore producing identical results.

use bevy_ecs::prelude::*;

use crate::rng::Rngs;
use crate::schedule::new_sim_schedule;
use crate::tick::Tick;

/// A complete simulation: the world, its schedule, and its clock.
pub struct Sim {
    world: World,
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

        let mut schedule = new_sim_schedule();
        build(&mut schedule);

        Self { world, schedule }
    }

    /// Advance the world by exactly one tick.
    ///
    /// The clock advances *before* systems run, so a system observing `Tick(1)`
    /// is doing the work of the first tick rather than reporting the tick it just
    /// finished.
    pub fn step(&mut self) {
        let next = self.world.resource::<Tick>().next();
        self.world.insert_resource(next);
        self.schedule.run(&mut self.world);
    }

    /// Advance by `n` ticks.
    pub fn step_n(&mut self, n: u64) {
        for _ in 0..n {
            self.step();
        }
    }

    #[must_use]
    pub fn tick(&self) -> Tick {
        *self.world.resource::<Tick>()
    }

    #[must_use]
    pub fn seed(&self) -> u64 {
        self.world.resource::<Rngs>().master_seed()
    }

    #[must_use]
    pub fn world(&self) -> &World {
        &self.world
    }

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
        assert_eq!(Sim::new(1).tick(), Tick(0));
    }

    #[test]
    fn step_advances_the_clock() {
        let mut sim = Sim::new(1);
        sim.step();
        assert_eq!(sim.tick(), Tick(1));
        sim.step_n(9);
        assert_eq!(sim.tick(), Tick(10));
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
