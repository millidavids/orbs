//! The simulation, and the single entry point frontends drive it through.
//!
//! Frontends are *callers*, not hosts: they construct a [`Sim`] and call
//! [`Sim::step`]. No frontend's own scheduler ever drives the world. This is what
//! keeps the Bevy build, the terminal build, and the balance harness running the
//! identical code path — and therefore producing identical results.

use bevy_ecs::prelude::*;
use orbs_render::{Outcome, Presentation, RecordKind};

use crate::execute::run_pending;
use crate::parser::{Mode, ParseLog, ParseRecord, Resolution, Scene, analyse, report};
use crate::rng::Rngs;
use crate::schedule::new_sim_schedule;
use crate::session::{Choices, Pending, Scrollback, Skip, Submissions, Wizard};
use crate::tick::Tick;
use crate::tower::{self, NodeIds};

/// A complete simulation: the world, its schedule, and its clock.
pub struct Sim {
    world: World,
    /// Commands the player queued, applied before the world moves.
    commands: Schedule,
    schedule: Schedule,
    /// What the player can name, rebuilt after the world has moved.
    scene: Schedule,
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
        world.init_resource::<Scene>();
        world.init_resource::<NodeIds>();
        world.init_resource::<Scrollback>();
        world.init_resource::<Pending>();
        world.init_resource::<Submissions>();
        world.init_resource::<Skip>();
        world.init_resource::<ParseLog>();
        world.init_resource::<Wizard>();
        world.init_resource::<Choices>();

        // Its **own** schedule, run before the caller's. Adding `run_pending`
        // to the same schedule and relying on insertion order would be an
        // ambiguity, not an ordering: Bevy makes no promise about systems with
        // no constraint between them, and a domain system added through `build`
        // could observe the queue either drained or not. A separate pass is
        // unambiguous by construction and needs no set for callers to remember.
        let mut commands = new_sim_schedule();
        commands.add_systems(run_pending);

        let mut schedule = new_sim_schedule();
        schedule.add_systems((tower::finish, tower::drift));
        build(&mut schedule);

        // A **third** pass, for the same reason `commands` is a first one:
        // putting `rebuild` in the same schedule as whatever `build` adds is an
        // ambiguity rather than an ordering, and Bevy's topsort was in fact
        // running the caller's systems first despite `rebuild` being inserted
        // first. If that flipped, the scene would go a tick stale — and since
        // every frontend passes a different `build` closure, the two graphs
        // could flip differently, which is exactly the game/harness divergence
        // §13 exists to prevent.
        //
        // Last, so the scene names the world as the tick left it.
        let mut scene = new_sim_schedule();
        scene.add_systems(tower::rebuild);

        // The tower is raised before the first tick, so tick 0 already has a
        // world to name.
        tower::raise(&mut world);
        tower::rebuild(&mut world);

        Self {
            world,
            commands,
            schedule,
            scene,
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
        // What the player asked for, what the world does about it, then what
        // the player can name afterwards. Three passes rather than ordered
        // systems in one — see `with_schedule`.
        self.commands.run(&mut self.world);
        self.schedule.run(&mut self.world);
        self.scene.run(&mut self.world);
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

        // A bare digit answers a numbered prompt (§6). Checked before parsing
        // because a digit is not a command and no verb takes one as its whole
        // input, so there is nothing to collide with — and without this the
        // prompt is rhetorical: it asks a question, the answer resolves as a
        // miss, and the player is in the dead end §15's gate weighs most.
        if !self.world.resource::<Choices>().is_empty()
            && let Ok(choice) = line.trim().parse::<usize>()
        {
            self.choose(line, choice);
            return;
        }
        // Anything else walks away from the question. §6 forbids a modal
        // prompt, so leaving one unanswered must cost nothing.
        self.world.resource_mut::<Choices>().clear();

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
        match resolution {
            Resolution::Resolved { intent, .. } => {
                self.world.resource_mut::<Pending>().push(intent);
            }
            // Hold the readings so the numbers the player just saw mean
            // something when they type one.
            Resolution::Ambiguous { candidates } => {
                let readings = candidates.into_iter().map(|c| c.intent).collect();
                self.world.resource_mut::<Choices>().offer(readings);
            }
            Resolution::Incomplete { .. } | Resolution::Unresolved { .. } => {}
        }
    }

    /// Answer a numbered prompt.
    ///
    /// Recorded in the scrollback but **not** in the parse trace: a digit is not
    /// a phrasing, and counting it as one would dilute §15's first metric with
    /// inputs that were never a test of the parser. What the gate wants is the
    /// *original* ambiguous line reaching its intended action, which the
    /// following selection is the evidence for.
    fn choose(&mut self, line: &str, choice: usize) {
        let picked = self.world.resource::<Choices>().pick(choice).cloned();

        let mut scrollback = self.world.resource_mut::<Scrollback>();
        let records = scrollback.records_mut();
        records
            .push(RecordKind::Input)
            .text(orbs_render::FieldName::Message, line)
            .finish();

        let Some(intent) = picked else {
            // A number outside the list. §6 forbids a dead end, so the question
            // stands rather than being silently dropped — the player can try
            // another number.
            records
                .push(RecordKind::Echo)
                .outcome(Outcome::Unresolved)
                .text(orbs_render::FieldName::Message, line)
                .finish();
            return;
        };

        records
            .push(RecordKind::Echo)
            .outcome(Outcome::Resolved)
            .text(orbs_render::FieldName::Message, &intent.echo())
            .finish();

        let tick = *self.world.resource::<Tick>();
        self.world.resource_mut::<Submissions>().push(tick, line);
        self.world.resource_mut::<Choices>().clear();
        self.world.resource_mut::<Pending>().push(intent);
    }

    /// Everything the player has said and been told.
    #[must_use]
    pub fn scrollback(&self) -> &Scrollback {
        self.world.resource::<Scrollback>()
    }

    /// What is in flight, if anything (§5.0).
    ///
    /// One at a time: §11.5 opens at multiplex capacity 1 and §9's fourth
    /// invariant reserves that slot for the action's whole duration.
    #[must_use]
    pub fn working(&self) -> Option<tower::Working> {
        self.world
            .iter_entities()
            .find_map(|entity| entity.get::<tower::Working>().copied())
    }

    /// The readings the orb is waiting for the player to pick between (§6).
    #[must_use]
    pub fn choices(&self) -> &Choices {
        self.world.resource::<Choices>()
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

    /// What the prompt reads, before the caret.
    ///
    /// `<name> $ `. Composed here rather than in a view because the name is
    /// world state — see [`Wizard`](crate::session::Wizard) — and because both
    /// frontends must show the same one.
    #[must_use]
    pub fn prompt(&self) -> String {
        format!("{} $ ", self.world.resource::<Wizard>().name())
    }

    /// Rename the wizard at the orb.
    pub fn rename(&mut self, name: &str) {
        self.world.resource_mut::<Wizard>().rename(name);
    }

    /// Where the player is standing, as a path (§7).
    ///
    /// What the prompt shows, and therefore what tells a player which commands
    /// will resolve: the essences are in `/tower/alembic`, so that is where
    /// `decoct` works.
    #[must_use]
    pub fn location(&self) -> String {
        self.world
            .get_resource::<tower::Cwd>()
            .map_or_else(String::new, |cwd| tower::path_of(&self.world, cwd.0))
    }

    /// Everything the player can currently name (§6).
    ///
    /// Rebuilt every tick from the tower, so it is the world as it is rather
    /// than as it was when a command table was written.
    #[must_use]
    pub fn scene(&self) -> &Scene {
        self.world.resource::<Scene>()
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
