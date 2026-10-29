//! The simulation, and the single entry point frontends drive it through.
//!
//! Frontends are *callers*, not hosts: they construct a [`Sim`] and call
//! [`Sim::step`], never their own scheduler, so the Bevy build, the terminal
//! build and the balance harness run one code path and produce identical
//! results.

use bevy_ecs::prelude::*;
use orbs_render::{Outcome, Presentation, RecordKind};

use crate::content::{Fuels, Prose, Recipes, Spells};
use crate::execute::run_pending;
use crate::parser::{
    Analysis, Confidence, Mode, ParseLog, ParseRecord, Resolution, Scene, analyse, report,
};
use crate::rng::Rngs;
use crate::schedule::new_sim_schedule;
use crate::session::{Choices, Pending, Scrollback, Skip, Submission, Submissions, Wizard};
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
    ///
    /// An open tower and the curve as authored — what every test, dump and
    /// balance policy measures against. A fresh game is [`begun`](Self::begun).
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self::build(seed, false, crate::content::Length::Baseline, |_| {}).reported()
    }

    /// Create a simulation that begins as a laboratory and nothing else.
    ///
    /// What a station opens is shut until it is reached (§11.5,
    /// `tower::Opened::start`). Travels in the save, since sealed and open
    /// towers of one seed diverge at the first `attend archive`.
    #[must_use]
    pub fn sealed(seed: u64) -> Self {
        Self::build(seed, true, crate::content::Length::Baseline, |_| {}).reported()
    }

    /// An open tower at a chosen length — what the balance harness measures.
    ///
    /// `new` is this at [`Baseline`](crate::content::Length::Baseline). A policy
    /// needs every room open, so this is how a played length gets measured.
    #[must_use]
    pub fn measured(seed: u64, length: crate::content::Length) -> Self {
        Self::build(seed, false, length, |_| {}).reported()
    }

    /// The tower a fresh *game* builds: sealed, and as long as the player asked.
    ///
    /// One constructor, since a game is sealed *and* has a length.
    #[must_use]
    pub fn begun(seed: u64, length: crate::content::Length) -> Self {
        Self::build(seed, true, length, |_| {}).reported()
    }

    /// A fresh game that has said nothing yet: the installation's practice tower.
    ///
    /// [`begun`](Self::begun) without the opening report, so the report arrives
    /// once, in the tower raised when the installation ends. Never kept. See
    /// DESIGN.md §19, *The orb is installed before the tower wakes*.
    #[must_use]
    pub fn sandbox(seed: u64, length: crate::content::Length) -> Self {
        Self::build(seed, true, length, |_| {})
    }

    /// Create a simulation and populate its schedule.
    ///
    /// The schedule is never exposed afterwards. A frontend must not call this:
    /// it is for tests that observe the ordering from outside (`tower::scene`),
    /// and a frontend registering systems would be a different game (§13).
    ///
    /// # Panics
    ///
    /// If the built-in content files disagree — an authoring error no input
    /// can reach, as `load::builtin` treats one.
    #[must_use]
    pub fn with_schedule(seed: u64, build: impl FnOnce(&mut Schedule)) -> Self {
        Self::build(seed, false, crate::content::Length::Baseline, build).reported()
    }

    /// The tower's opening report, which every constructor but
    /// [`sandbox`](Self::sandbox) makes. Last, after `rebuild`, because the
    /// report lists only what is open.
    fn reported(mut self) -> Self {
        tower::report(&mut self.world);
        self
    }

    /// The one construction behind every constructor here.
    fn build(
        seed: u64,
        sealed: bool,
        length: crate::content::Length,
        build: impl FnOnce(&mut Schedule),
    ) -> Self {
        let mut world = Self::bare(seed, length);
        let (commands, schedule, scene) = Self::schedules(build);

        // The tower is raised before the first tick, so tick 0 already has a
        // world to name.
        tower::raise(&mut world);
        // Sealed before anything reads the rooms: the boot report lists only
        // what is open, and the scene is built from what is not sealed.
        if sealed {
            let start = tower::Opened::start(
                world.resource::<Recipes>(),
                world.resource::<crate::content::Charms>(),
                world.resource::<crate::content::Spendables>(),
                world.resource::<crate::content::Progression>(),
            );
            world.insert_resource(start);
            world.insert_resource(tower::Sealing(true));
        }
        // The walls say how they stand from tick 0, since `erode` publishes
        // only on a change and an absent reading is nought. Here, because
        // `tower::build` knows no resources.
        if let Some(pylon) = tower::pylon::fixture(&world) {
            crate::execute::publish_pylon(&mut world, pylon);
        }
        // And what each die costs, for the same reason.
        crate::execute::publish_dice(&mut world);
        // ...and the standing garrison on the roster, before the first tick so a
        // fresh tower and one reloaded from its save carry the same nodes.
        tower::siege::roll_call(&mut world);
        // ...and the forge's, by path, since `Cwd` is the root here.
        //
        // `seal` last, after the publishers create their nodes, as `restore`
        // does, or a fresh tower and its reload differ to sabotage.
        tower::seal(&mut world);
        tower::rebuild(&mut world);

        Self {
            world,
            commands,
            schedule,
            scene,
        }
    }

    /// Every resource a world needs, and no tower in it.
    ///
    /// One list for raising a world and loading one, since two constructions
    /// are two games (§13). Not a `Default`: an unseeded world is broken.
    ///
    /// # Panics
    ///
    /// As [`with_schedule`](Self::with_schedule) documents.
    fn bare(seed: u64, length: crate::content::Length) -> World {
        let mut world = World::new();
        world.insert_resource(Rngs::from_seed(seed));
        world.insert_resource(Tick::default());
        world.init_resource::<Scene>();
        world.init_resource::<NodeIds>();
        world.init_resource::<Scrollback>();
        world.init_resource::<Pending>();
        world.init_resource::<Submissions>();
        world.init_resource::<Skip>();
        world.init_resource::<tower::Cooling>();
        world.init_resource::<ParseLog>();
        world.init_resource::<Wizard>();
        world.init_resource::<Choices>();
        // What each domain wants noticed, for §9's rail. `attend` is the only
        // thing that clears, so a mark survives a save as the state it
        // describes does.
        world.init_resource::<crate::tower::Marks>();
        // What the player has found. Written only by a broken ward, so it
        // replays from `(seed, submissions)` like everything else.
        world.init_resource::<crate::tower::Learned>();
        // The compiled-in default, so a headless `Sim` needs no filesystem
        // (rule 6, rule 8). A frontend swaps it with `set_prose`.
        world.init_resource::<Prose>();
        // The manual, on the same terms, swappable with `ORBS_CONTENT`. It
        // registers no parser nouns, which is why it is a second resource
        // rather than a prefix in `prose.toml` — see `content::manual`.
        world.init_resource::<crate::content::Manual>();
        // The manual's subjects, fixed here. They are parser nouns, so reading
        // them live from `Prose` would let a hot reload change what a phrase
        // resolves to — see `tower::Topics`.
        let topics = crate::tower::Topics::of(world.resource::<Prose>());
        world.insert_resource(topics);
        // Recipes are *not* hot-reloadable, unlike prose: they reach decisions,
        // so a mid-session swap would break replay from `(seed, submissions)`
        // unless the content were versioned with it.
        world.init_resource::<Recipes>();
        world.init_resource::<Fuels>();
        // What the arsenal is worth in a siege. Recipes' tier, not materials':
        // how many troops a scroll is worth changes what a round does, so a
        // mid-session swap would break replay.
        world.init_resource::<crate::content::Spendables>();
        // What a scroll does in a room, for the same reason.
        world.init_resource::<crate::content::Scrolls>();
        world.init_resource::<crate::content::Charms>();
        // Materials *are* hot-reloadable in principle: a tint is read by the
        // instrument panel and nothing else, so no verb branches on it. Beside
        // the two above because that is where content lives, not because it
        // shares their constraint.
        world.init_resource::<crate::content::Materials>();
        // Recipes' tier again: a spell is nothing *but* decisions, so a reload
        // would break replay. Read once here — the player's own edits go
        // through the world, not through this.
        world.init_resource::<Spells>();
        // Recipes' tier too, since a swap would break replay. Checked against
        // the recipes, so loaded after them; stretched first, so `ascends`
        // checks the curve the world uses — which every length keeps.
        let curve = crate::content::Progression::default().stretched(length);
        // Kept beside the curve it produced, so a save can say what length this
        // game is without inferring it back out of the thresholds.
        world.insert_resource(length);
        // What may be priced: anything that runs. The recipes' instruments plus
        // the fixtures that carry a verb and transform nothing — the athanor,
        // and the `stacks`, which earns per walk finished and has no recipe to
        // be found by. The recipes alone left both unpriceable.
        let mut instruments = world.resource::<Recipes>().instruments();
        instruments.extend(tower::operated());
        instruments.sort_unstable();
        instruments.dedup();
        {
            let recipes = world.resource::<Recipes>();
            let charms = world.resource::<crate::content::Charms>();
            let outputs = recipes.outputs();
            let gated = recipes.gated();
            let charms: Vec<&str> = charms.names().collect();
            let demons: Vec<&str> = world
                .resource::<crate::content::Spendables>()
                .demons()
                .collect();
            let catalogue = crate::content::Catalogue {
                instruments: &instruments,
                outputs: &outputs,
                gated: &gated,
                charms: &charms,
                demons: &demons,
            };
            if let Err(error) = curve.check(&catalogue) {
                panic!("the built-in content is authored with the crate: {error}");
            }
        }
        world.insert_resource(curve);
        world.init_resource::<tower::Experience>();
        world.init_resource::<tower::Renown>();
        world.init_resource::<tower::siege::Petitioned>();
        world.init_resource::<tower::Cultivation>();
        world.init_resource::<tower::siege::Garrison>();
        world.init_resource::<tower::supply::Supplies>();
        world.init_resource::<tower::Stores>();
        world.init_resource::<tower::Throughput>();
        // Everything open, the tower every test, dump and policy uses. A fresh
        // *game* starts sealed — see `Sim::sealed`.
        let opened = tower::Opened::all(
            world.resource::<Recipes>(),
            world.resource::<crate::content::Charms>(),
            world.resource::<crate::content::Spendables>(),
        );
        world.insert_resource(opened);
        world.init_resource::<tower::Sealing>();
        world.init_resource::<tower::Tally>();
        world.init_resource::<tower::mastery::Reached>();
        // Whole, by `Default`. A tower is not built already crumbling.
        world.init_resource::<tower::Integrity>();
        // Full, as `Integrity` opens whole, or the forge is inert for twelve
        // minutes. After `Integrity`, which the ceiling reads.
        world.init_resource::<tower::Taken>();
        let ceiling = tower::ceiling(&world);
        world.insert_resource(tower::Quintessence::new(ceiling));
        world.init_resource::<crate::execute::Opening>();
        world.init_resource::<crate::execute::Reloaded>();
        world.init_resource::<crate::execute::Unfurling>();
        world.init_resource::<crate::execute::Quitting>();
        world.init_resource::<crate::execute::Menuing>();
        world.init_resource::<crate::execute::Weaving>();
        world.init_resource::<crate::execute::Wandering>();
        world.init_resource::<tower::spell::Caller>();

        world
    }

    /// The three passes a tick runs, in the order they run.
    ///
    /// Written once, beside [`bare`](Self::bare): the order is load-bearing, and
    /// a loaded world must run the passes a raised one does.
    fn schedules(build: impl FnOnce(&mut Schedule)) -> (Schedule, Schedule, Schedule) {
        // Its own schedule, run before the caller's. Bevy promises nothing
        // about systems with no constraint between them, so insertion order
        // would leave a domain system added through `build` free to observe the
        // queue drained or not. A separate pass is unambiguous by construction.
        let mut commands = new_sim_schedule();
        commands.add_systems(run_pending);

        let mut schedule = new_sim_schedule();
        // `burn` before `finish`: a fire that runs out on the tick a heated
        // stage lands is cold *after* that stage completes, since the run was
        // committed when it started (§10.1).
        //
        // `spell::advance` before `finish`, or a script gets a free tick no
        // manual player does (§19). `spell::stand` last, or a held spell gets
        // two goes at the budget in one tick.
        schedule.add_systems(
            (
                tower::spell::advance,
                tower::burn,
                tower::finish,
                // Draws twice a tick from `RngStream::Threat`, and was two
                // systems: calm-layer log drift is gone (§19) and its draw
                // stayed, so every existing replay holds.
                tower::substitution,
                // After the roll, and it draws nothing: a lie settling is a
                // clock reading, so appending it perturbs no replay.
                tower::settling,
                // The same licence: a barrier wearing down compares two ticks,
                // and the sanctum's one draw is in `height_for` inside `muster`
                // rather than in a system.
                tower::erode,
                // No menagerie system: a beast waits for ever (§19). This one
                // draws nothing, and runs after `erode`, since the ceiling is a
                // function of integrity (§19).
                tower::regenerate,
                // The same licence: a body deserting is a modulo on the tick,
                // and paused while a siege runs (§19, *Supply*).
                tower::siege::desert,
                // The same licence: a calm supply draws on a modulo of the
                // ticks since its word, and paused in a siege (§19, *Supply*).
                crate::execute::drain_scrolls,
                // The same licence, and load-bearing rather than tidy: without
                // a system saying a charm lapsed, `ebbing` would arrive only
                // when a forge verb ran, and the maintenance spell is built
                // entirely on it arriving.
                crate::execute::lapse_charms,
                // `lapse_charms`'s reason: a spell keeping its stores up needs
                // `thin` to arrive alone. It writes only on a change, so an
                // idle tick issues no `NodeId` (§19).
                tower::stocktake,
                // `stocktake`'s rule for the supplies and the roster: each
                // writes only on a change, and runs only when what it reads
                // moved.
                tower::supply::gauge.run_if(
                    resource_exists_and_changed::<tower::supply::Supplies>
                        .or_eager(tower::supply::restocked),
                ),
                tower::siege::roll_call.run_if(
                    resource_exists_and_changed::<tower::siege::Garrison>
                        .or_eager(resource_exists_and_changed::<tower::ley::Taken>),
                ),
                tower::spell::stand,
            )
                .chain(),
        );
        build(&mut schedule);

        // A pass of its own, since the topsort ran callers' systems before
        // `rebuild` and frontends could flip differently (§13). Last, so the
        // scene names the world as the tick left it.
        let mut scene = new_sim_schedule();
        scene.add_systems(tower::rebuild);

        (commands, schedule, scene)
    }

    /// Read this world out as a save document.
    ///
    /// `&self`, so a save cannot move the archetypes it measures. At a tick
    /// boundary (§8), straight after [`step`](Self::step); the file is
    /// `orbs_shell::save`'s.
    #[must_use]
    pub fn snapshot(&self) -> crate::save::Save {
        crate::save::capture(&self.world)
    }

    /// Build a world from a save document.
    ///
    /// Raises the tower and applies the save over it (`crate::save::restore`),
    /// so the result stands exactly where the saved world stood —
    /// `tests/persistence.rs` holds it.
    ///
    /// # Panics
    ///
    /// As [`with_schedule`](Self::with_schedule) does: the built-in content is
    /// authored with the crate.
    #[must_use]
    pub fn restored(save: &crate::save::Save) -> Self {
        // The saved length, or `bare` installs the authored curve — and the
        // completeness lint walks components, not resources.
        let mut world = Self::bare(save.world.seed, save.world.length);
        let (commands, schedule, scene) = Self::schedules(|_| {});

        // Raised before the save is applied, never instead of it.
        tower::raise(&mut world);
        crate::save::restore(&mut world, save);
        tower::rebuild(&mut world);
        // Only a document that disagreed with itself changes here: a puzzle
        // refused on load leaves readings behind that nothing clears until the
        // next `summon` or `muster`. One pass over every `puzzle::Open`, so a
        // restore that starts refusing is already covered.
        crate::execute::settle_puzzles(&mut world);

        // No `tower::report`: a restored world is the saved world exactly.
        // *You are back* is `resumed`'s, since the gap needs a wall clock.

        Self {
            world,
            commands,
            schedule,
            scene,
        }
    }

    /// Say that this tower was resumed, and how long it was dark.
    ///
    /// The words are the sim's (rule 6), the gap the frontend's (§19). Not in
    /// [`restored`](Self::restored), which must equal the save byte for byte.
    /// `away` is seconds, or `None` where the machine would not say.
    pub fn say_resumed(&mut self, away: Option<u64>) {
        let prose = self.world.resource::<Prose>().clone();
        let mut scrollback = self.world.resource_mut::<Scrollback>();
        let records = scrollback.records_mut();
        records
            .push(RecordKind::Message)
            .text(orbs_render::FieldName::Message, &prose.line("resumed", &[]))
            .finish();

        if let Some(away) = away {
            let span = crate::tick::span(away);
            records
                .push(RecordKind::Message)
                .text(
                    orbs_render::FieldName::Message,
                    &prose.line("resumed_away", &[("span", &span)]),
                )
                .role(orbs_render::Role::Normal)
                .finish();
        }
    }

    /// Say that the tower could not be written out.
    ///
    /// A record, since the stream is the output (§3). The caller says it once,
    /// not every sixty-tick attempt.
    pub fn say_save_failed(&mut self) {
        self.say_save("save_failed", &[], orbs_render::Role::Danger);
    }

    /// Say that the tower is kept in slot `slot`, not the one it was raised
    /// for, because another orb wrote that one first.
    pub fn say_save_moved(&mut self, slot: usize) {
        let slot = slot.to_string();
        self.say_save("save_moved", &[("count", &slot)], orbs_render::Role::Normal);
    }

    /// Say that a save was there and could not be read.
    ///
    /// A tower that did not come back is owed a reason on screen.
    pub fn say_save_unreadable(&mut self) {
        self.say_save("save_unreadable", &[], orbs_render::Role::Danger);
    }

    /// One line about the tower's file.
    fn say_save(&mut self, key: &str, args: &[(&str, &str)], role: orbs_render::Role) {
        let message = self.world.resource::<Prose>().line(key, args);
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Message)
            .text(orbs_render::FieldName::Message, &message)
            .role(role)
            .finish();
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
    /// Advances no world time: it echoes now and queues any resolved command
    /// for the next tick (see [`session`](crate::session), §19). A blank line
    /// does nothing, as in every shell.
    pub fn submit(&mut self, line: &str) {
        if line.trim().is_empty() {
            return;
        }

        // A bare digit answers a numbered prompt (§6), before parsing, since no
        // verb is a digit.
        if !self.world.resource::<Choices>().is_empty()
            && let Ok(choice) = line.trim().parse::<usize>()
        {
            self.choose(line, choice);
            return;
        }
        // Anything else walks away from the question. §6 forbids a modal
        // prompt, so leaving one unanswered must cost nothing.
        self.world.resource_mut::<Choices>().clear();

        // A tester's door, and only in a build a tester runs. Matched exactly
        // and before the parser: no debug word is in §6's vocabulary, so the
        // fuzzy matcher must never see one and `Verb::ALL` must never grow one.
        // In a release binary this is an ordinary unresolvable line.
        #[cfg(debug_assertions)]
        if let Some(word) = crate::execute::DebugWord::read(line) {
            crate::execute::open_door(&mut self.world, line, word);
            return;
        }
        #[cfg(debug_assertions)]
        if let Some(order) = crate::execute::spell_order(line) {
            self.debug_spell(line, &order);
            return;
        }

        // `analyse`, which keeps every scored reading for the trace (§6);
        // `resolve` is the same work with the candidates dropped.
        let tick = *self.world.resource::<Tick>();
        let analysis = analyse(line, self.world.resource::<Scene>(), Mode::Calm);
        self.world.resource_mut::<ParseLog>().push(ParseRecord::new(
            tick.get(),
            line,
            Mode::Calm,
            &analysis,
        ));
        let resolution = analysis.resolution;

        // Cloned because `report` needs the prose while `Scrollback` is borrowed
        // mutably, and both live in the same world. It is one line's worth of
        // lookup on a keystroke, not per frame.
        let prose = self.world.resource::<Prose>().clone();
        let mut scrollback = self.world.resource_mut::<Scrollback>();
        let records = scrollback.records_mut();
        records
            .push(RecordKind::Input)
            .text(orbs_render::FieldName::Message, line)
            .finish();
        report(line, &resolution, &prose, records);

        self.world.resource_mut::<Submissions>().push(tick, line);
        match resolution {
            Resolution::Resolved { intent, .. } => {
                self.world.resource_mut::<Pending>().push(intent);
            }
            // Hold the readings so the numbers the player just saw mean
            // something when they type one.
            Resolution::Ambiguous { candidates } => {
                let readings = candidates.into_iter().map(|c| c.intent).collect();
                self.world.resource_mut::<Choices>().offer(line, readings);
            }
            Resolution::Incomplete { .. }
            | Resolution::TakesNothing { .. }
            | Resolution::Elsewhere { .. }
            | Resolution::InSpell { .. }
            | Resolution::Unresolved { .. } => {}
        }
    }

    /// Submit a line, letting `augur` read it if the orb cannot (§6).
    ///
    /// The three tiers in one place. A frontend calls this instead of
    /// [`submit`](Self::submit) when it has a reader; everything without one
    /// keeps calling `submit` and behaves exactly as it always has.
    ///
    /// 1. The orb reads it — [`is_literal`](crate::parser::is_literal) for what
    ///    text alone settles, and
    ///    [`Analysis::reads_outright`](crate::parser::Analysis::reads_outright)
    ///    for a typed verb whose reading accounted for every word. Either way
    ///    the line goes to `submit` untouched, so `Elsewhere`, `InSpell`,
    ///    `Incomplete` and the numbered prompt all survive.
    /// 2. The augury reads it, and [`submit_divined`](Self::submit_divined) runs
    ///    what it decided.
    /// 3. Nobody reads it, and `submit` answers with §6's suggestions.
    ///
    /// Tier one is decided first, so nothing that resolves can regress. The
    /// double `analyse` is pure and under a millisecond, and keeps `submit`
    /// meaning one thing.
    pub fn submit_reading(&mut self, line: &str, augur: &dyn crate::Augur) {
        if crate::parser::is_literal(line) {
            self.submit(line);
            return;
        }
        let analysis = analyse(line, self.world.resource::<Scene>(), Mode::Calm);
        if analysis.reads_outright() {
            self.submit(line);
            return;
        }
        // The first reading that resolves, and the room decides which: `run
        // night_watch` is `invoke` or `wield` depending on what `night_watch`
        // *is*, and only the scene knows. Trying them here keeps that judgement
        // with `analyse`, the deterministic half.
        let scene = self.world.resource::<Scene>().clone();
        let readings: Vec<String> = augur
            .read(line)
            .into_iter()
            .take(crate::augur::MAX_READINGS)
            .collect();

        // A command that runs, if any of them does — and of those, one that
        // uses every word it was handed before one that leaves some over. See
        // `parser::reading_to_run`.
        let runs = crate::parser::reading_to_run(&readings, &scene, Mode::Calm).cloned();
        if let Some(echo) = runs {
            self.submit_divined(line, &echo);
            return;
        }

        // Failing that, a deliberate refusal beats a shrug: `grind sage` in a
        // room with no mortar is `Elsewhere`, and §19 wants that because *"I do
        // not know that word"* would lie about a word the game taught next
        // door. Requiring a reading to *run* threw it away.
        let answers = readings
            .iter()
            .find(|echo| {
                matches!(
                    analyse(echo, &scene, Mode::Calm).resolution,
                    Resolution::Elsewhere { .. }
                        | Resolution::Incomplete { .. }
                        | Resolution::TakesNothing { .. }
                )
            })
            .cloned();
        if let Some(echo) = answers {
            self.submit_divined(line, &echo);
            return;
        }

        // Nothing the reader offered means anything here, in any sense. §6's
        // suggestions are a better answer than a command that cannot run.
        self.submit(line);
    }

    /// [`submit_reading`](Self::submit_reading) with the reader there is, or
    /// [`submit`](Self::submit) with none — what every frontend's prompt does.
    pub fn submit_with(&mut self, line: &str, reader: Option<&dyn crate::Augur>) {
        match reader {
            Some(augur) => self.submit_reading(line, augur),
            None => self.submit(line),
        }
    }

    /// Run a line the augury read, rather than one the orb read (§6).
    ///
    /// The augury's entry point, once neither [`crate::parser::is_literal`] nor
    /// [`Analysis::reads_outright`] settles the line and a model expanded it
    /// into the canonical `echo`.
    ///
    /// `echo` is what is analysed, recorded and replayed, and `line` only
    /// shown: a model need not match across GPUs, so it runs once, here — the
    /// determinism boundary ([`Submission::Divined`]). The echo carries
    /// [`Confidence::Divined`], so it draws `≈` and an inferred `purge`
    /// confirms.
    pub fn submit_divined(&mut self, line: &str, echo: &str) {
        if line.trim().is_empty() || echo.trim().is_empty() {
            return;
        }
        // A sentence walks away from an open question, the same as any other
        // line that is not the digit answering it.
        self.world.resource_mut::<Choices>().clear();

        let tick = *self.world.resource::<Tick>();
        let analysis = analyse(echo, self.world.resource::<Scene>(), Mode::Calm);

        // Traced against what the player typed: a session sifted for what the
        // augury was asked is sifted for their words. The canonical form is in
        // the `echo` column beside it.
        let resolution = match analysis.resolution {
            Resolution::Resolved { intent, .. } => Resolution::Resolved {
                intent,
                confidence: Confidence::Divined,
            },
            other => other,
        };
        let traced = Analysis {
            resolution: resolution.clone(),
            candidates: analysis.candidates,
        };
        let mut record = ParseRecord::new(tick.get(), line, Mode::Calm, &traced);
        // Stamped, not derived from the confidence, which only `Resolved`
        // carries: the augury's failures are the export's best rows.
        record.divined = true;
        self.world.resource_mut::<ParseLog>().push(record);

        let prose = self.world.resource::<Prose>().clone();
        let mut scrollback = self.world.resource_mut::<Scrollback>();
        let records = scrollback.records_mut();
        records
            .push(RecordKind::Input)
            .text(orbs_render::FieldName::Message, line)
            .finish();
        report(line, &resolution, &prose, records);

        self.world
            .resource_mut::<Submissions>()
            .divined(tick, line, echo);
        match resolution {
            Resolution::Resolved { intent, .. } => {
                self.world.resource_mut::<Pending>().push_divined(intent);
            }
            // No numbered prompt from a divined reading (§6): the augury acts
            // on its best reading and offers correction, where stopping to ask
            // is the interrogation it exists to remove.
            Resolution::Ambiguous { .. }
            | Resolution::Incomplete { .. }
            | Resolution::TakesNothing { .. }
            | Resolution::Elsewhere { .. }
            | Resolution::InSpell { .. }
            | Resolution::Unresolved { .. } => {}
        }
    }

    /// Save a spell out of the editor.
    ///
    /// A file the player wrote is a decision, so it is in [`Submissions`]; the
    /// keystrokes are not, which is why the editor is the frontend's. Queued,
    /// like `submit`. The typed lines are recorded, so a better reader cannot
    /// change an old replay.
    pub fn write_spell(&mut self, name: &str, lines: &[String]) {
        self.write_spell_reading(name, lines, &crate::augur::Verbatim);
    }

    /// Write a spell out, with a reader for the lines the orb cannot read.
    ///
    /// [`write_spell`](Self::write_spell) is this with a reader that abstains on
    /// everything, so a build with none is the game exactly as it was.
    ///
    /// `lines` goes to [`Held`](crate::tower::Held) byte-exact (§19), and the
    /// reading to [`Read`](crate::tower::Read) beside it, what `compile`
    /// compiles. Read once on the way in, by the frontend's reader (rules 1
    /// and 3); unchanged text keeps its reading.
    pub fn write_spell_reading(
        &mut self,
        name: &str,
        lines: &[String],
        scrivener: &dyn crate::Scrivener,
    ) {
        let filename = crate::content::with_extension(name);
        let read = self.reading_of(&filename, lines, scrivener);
        self.queue_write(&filename, lines, read, scrivener.identity());
    }

    /// Record the save and queue it, with the reading already settled.
    ///
    /// Split out so replay can reach it: a replayed `Wrote` carries its reading
    /// and must not derive a new one.
    fn queue_write(&mut self, filename: &str, lines: &[String], read: Vec<String>, by: u64) {
        let tick = *self.world.resource::<Tick>();
        self.world
            .resource_mut::<Submissions>()
            .wrote(tick, filename, lines, &read, by);
        self.world
            .resource_mut::<Pending>()
            .write(filename.to_owned(), lines.to_vec(), read, by);
    }

    /// Each line as the orb reads it, reusing what this reader already read.
    ///
    /// The newest reading, queued or landed — the queue first, since a write
    /// lands a tick late — so the save and the editor share one lookup.
    fn reading_of(
        &self,
        filename: &str,
        lines: &[String],
        scrivener: &dyn crate::Scrivener,
    ) -> Vec<String> {
        let by = scrivener.identity();
        let newest = self
            .world
            .resource::<Pending>()
            .written(filename)
            .or_else(|| {
                // The same walk `spell` does, for the reading rather than the text.
                self.world
                    .iter_entities()
                    .find(|entity| {
                        entity.get::<tower::Nameable>().map(|kind| kind.0)
                            == Some(crate::parser::NounKind::Script)
                            && entity
                                .get::<tower::Name>()
                                .is_some_and(|node| node.0 == filename)
                    })
                    .and_then(|entity| entity.get::<tower::Read>().cloned())
            });
        lines
            .iter()
            .map(|line| {
                // A blank line and a comment are not statements and there is
                // nothing for a reader to say about them.
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    return line.clone();
                }
                // Unchanged text keeps its reading — this reader's, which makes
                // autosave affordable and switching the reader reach every
                // line. See `tower::Read::by`.
                if let Some(read) = newest.as_ref().and_then(|newest| newest.kept(line, by)) {
                    return read.to_owned();
                }
                scrivener.read(line).unwrap_or_else(|| line.clone())
            })
            .collect()
    }

    /// Take a mastery node, on the next tick.
    ///
    /// [`write_spell`](Self::write_spell)'s shape: the id, recorded and queued.
    /// The world re-checks every rule (`execute::weave`).
    pub fn take(&mut self, id: &str) {
        let tick = *self.world.resource::<Tick>();
        self.world.resource_mut::<Submissions>().took(tick, id);
        self.world
            .resource_mut::<Pending>()
            .take_node(id.to_owned());
    }

    /// Which mastery nodes the orb has taken, in the order it took them.
    #[must_use]
    pub fn taken(&self) -> &[String] {
        self.world.resource::<tower::Taken>().ids()
    }

    /// What a spell holds, if the tower has one by that name.
    ///
    /// The extension is optional — `morning` and `morning.spell` are the same
    /// spell everywhere a player can name one.
    #[must_use]
    pub fn spell(&self, name: &str) -> Option<Vec<String>> {
        let wanted = crate::content::with_extension(name);
        self.world
            .iter_entities()
            .find(|entity| {
                entity.get::<tower::Nameable>().map(|kind| kind.0)
                    == Some(crate::parser::NounKind::Script)
                    && entity
                        .get::<tower::Name>()
                        .is_some_and(|node| node.0 == wanted)
            })
            .and_then(|entity| entity.get::<tower::Held>().map(|held| held.0.clone()))
    }

    /// Everything the player has earned by working (§11.5).
    #[must_use]
    pub fn experience(&self) -> u64 {
        self.world.resource::<tower::Experience>().get()
    }

    /// What the tower is known for, which is the one number that can fall.
    #[must_use]
    pub fn renown(&self) -> u64 {
        self.world.resource::<tower::Renown>().get()
    }

    /// How the tower's defences stand, out of [`tower::STANDING`].
    ///
    /// Falls on its own (`tower::erode`) and is put back by finishing a course
    /// in the sanctum — the only resource in the game that goes down.
    #[must_use]
    pub fn integrity(&self) -> u32 {
        self.world.resource::<tower::Integrity>().get()
    }

    /// Whether the node `named` in `room` is carrying `reading`.
    ///
    /// A spell's `if`, asked directly and without a tick, from the root — so
    /// it answers about rooms nobody stands in, where a bound solver is.
    #[must_use]
    pub fn holds_reading(&self, room: &str, named: &str, reading: &str) -> bool {
        let world = &self.world;
        let name_of = |node| {
            world
                .get::<tower::Name>(node)
                .map(|name| name.0.as_str())
                .unwrap_or_default()
        };
        let under = |node| tower::children_of(world, node);
        let Some(tower) = under(tower::root(world)).into_iter().next() else {
            return false;
        };
        under(tower)
            .into_iter()
            .filter(|node| name_of(*node) == room)
            .flat_map(under)
            .filter(|node| name_of(*node) == named)
            .flat_map(under)
            .any(|held| name_of(held) == reading)
    }

    /// How many spells the orb can hold at once.
    ///
    /// Derived from [`experience`](Self::experience), so it is a reading rather
    /// than a second piece of state — see `tower::experience`.
    #[must_use]
    pub fn concentration(&self) -> usize {
        tower::concentration(&self.world)
    }

    /// The ward's sheet, where the player is standing.
    ///
    /// Reads `Cwd`, as [`stacks`](Self::stacks) does, so the picture cannot
    /// follow the player out of the room.
    #[must_use]
    pub fn ward(&self) -> Option<orbs_render::Board> {
        self.world
            .get::<tower::Ward>(self.here_with::<tower::Ward>()?)
            .map(tower::Ward::view)
    }

    /// The course itself, for a caller that needs to ask it questions.
    ///
    /// For `orbs-balance`, which asks `Course::between` so the algorithm has
    /// one copy; [`pylon`](Self::pylon) is the painter's view.
    #[must_use]
    pub fn course(&self) -> Option<&tower::Course> {
        self.world
            .get::<tower::Course>(self.here_with::<tower::Course>()?)
    }

    /// The child of where the player stands that carries `C`, if any.
    ///
    /// One walk for every room-scoped view. `tower::pylon::fixture` is the
    /// tower-wide question and stays separate: a solver's readings keep up
    /// from another room, and a picture must not.
    fn here_with<C: bevy_ecs::component::Component>(&self) -> Option<bevy_ecs::entity::Entity> {
        tower::here_with::<C>(&self.world)
    }

    /// The course drawn up in the sanctum, if the player is looking at it.
    ///
    /// Reads `Cwd` for the reason [`ward`](Self::ward) does: the picture cannot
    /// outrun the readings by following the player out of the room.
    #[must_use]
    pub fn pylon(&self) -> Option<orbs_render::Pylon> {
        let standing = self.world.resource::<tower::Integrity>().get();
        // Through `course`, not a second walk of `Cwd`: a view is a course plus
        // a sentence, and only the sentence is here.
        let course = self.course()?;
        // The board's own line, written here: `orbs-render` holds no authored
        // English (rule 6), so the sentence comes from the same prose key the
        // reader hears — one spelling, and hot-reloadable like the rest.
        let tally = self.prose().line(
            "pylon_tally",
            &[
                ("quantity", &course.height().to_string()),
                ("name", &course.hauls().to_string()),
            ],
        );
        Some(course.view(standing, tally))
    }

    /// The lattice open in the forge, if the player is looking at it.
    ///
    /// Reads `Cwd` for the reason [`pylon`](Self::pylon) does: the picture
    /// cannot outrun the readings by following the player out of the room.
    #[must_use]
    pub fn lattice(&self) -> Option<orbs_render::LatticeBoard> {
        let node = self.here_with::<tower::lattice::Binding>()?;
        let binding = self.world.get::<tower::lattice::Binding>(node)?;
        // The board's own line, written here: `orbs-render` holds no authored
        // English (rule 6), so it comes from the prose key the reader hears.
        let tally = self.prose().line(
            "lattice_tally",
            &[
                ("name", &binding.kind),
                ("quantity", &binding.spent.to_string()),
            ],
        );
        let title = self.prose().line("lattice_title", &[]);
        Some(orbs_render::LatticeBoard {
            columns: tower::lattice::COLUMNS
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
            glyphs: binding.lattice.glyphs().to_vec(),
            snapped: (0..tower::lattice::WIDTH)
                .map(|column| binding.lattice.snapped(column))
                .collect(),
            width: tower::lattice::WIDTH,
            residue: binding.lattice.residue().to_vec(),
            tally,
            title,
        })
    }

    /// The siege being fought in the bailey, if the player is looking at it.
    ///
    /// Reads `Cwd`, as [`pylon`](Self::pylon) does. A finished siege still
    /// draws, so the postmortem stays.
    #[must_use]
    pub fn rampart(&self) -> Option<orbs_render::Rampart> {
        let rampart = self.here_with::<tower::Siege>()?;
        let siege = self.world.get::<tower::Siege>(rampart)?;
        // The board's own line, written here: `orbs-render` holds no authored
        // English (rule 6), so it comes from the prose key the reader hears.
        let tally = self.prose().line(
            "siege_tally",
            &[
                ("quantity", &siege.turns.to_string()),
                ("name", &siege.enemy.count.to_string()),
            ],
        );
        // What the arsenal is drawing onto this wall, and at which rate.
        let flowing = self
            .world
            .resource::<tower::supply::Supplies>()
            .iter()
            .filter(|(_, open)| open.scope == tower::supply::Scope::Siege)
            .map(|(name, open)| {
                (
                    name.to_owned(),
                    u32::try_from(open.running + 1).unwrap_or(u32::MAX),
                )
            })
            .collect();
        Some(siege.view(
            tally,
            self.world.resource::<tower::Quintessence>().get(),
            flowing,
        ))
    }

    /// The beast waiting at the menagerie's circle, if the player is looking at it.
    ///
    /// [`course`](Self::course)'s reason: the harness reads glyphs every step,
    /// and building the board for it was its hot path.
    #[must_use]
    pub fn beast(&self) -> Option<&tower::circle::Beast> {
        self.world
            .get::<tower::circle::Beast>(self.here_with::<tower::circle::Beast>()?)
    }

    /// The board for the beast waiting at the menagerie's circle, if the player
    /// is looking at it.
    ///
    /// Reads `Cwd`, as [`pylon`](Self::pylon) does.
    #[must_use]
    pub fn circle(&self) -> Option<orbs_render::Circle> {
        Some(tower::circle::view(self.beast()?, self.prose()))
    }

    /// Every domain at a glance — what §9's rail draws.
    ///
    /// Derived here, not in a frontend (rule 2). Every domain, in a fixed
    /// order, built or not — see [`tower::briefs`].
    #[must_use]
    pub fn briefs(&self) -> Vec<tower::Brief> {
        tower::briefs(&self.world)
    }

    /// The spells the orb is holding, named, in the order the tower keeps them.
    ///
    /// §8 surfaces concentration, and a player at capacity asks *which one*.
    #[must_use]
    pub fn bound(&self) -> Vec<String> {
        tower::spell::held(&self.world)
    }

    /// Every spell running, with where, on how many cursors, and what holds it.
    #[must_use]
    pub fn running_spells(&self) -> Vec<tower::Cast> {
        tower::running_spells(&self.world)
    }

    /// Which domain `name` was written for, if the tower has it.
    ///
    /// A component, not the text, so only this can say whether a save from
    /// another room re-homed a spell.
    #[must_use]
    pub fn spell_domain(&self, name: &str) -> Option<String> {
        let wanted = crate::content::with_extension(name);
        self.world
            .iter_entities()
            .find(|entity| {
                entity.get::<tower::Nameable>().map(|kind| kind.0)
                    == Some(crate::parser::NounKind::Script)
                    && entity
                        .get::<tower::Name>()
                        .is_some_and(|node| node.0 == wanted)
            })
            .and_then(|entity| entity.get::<tower::Domain>().map(|domain| domain.0.clone()))
    }

    /// How the orb reads `lines`, if they were a spell for `domain`.
    ///
    /// The editor's half of *report it, do not rewrite it*: told before cast,
    /// by the sim, so there is one parser (rule 2). Pure, and over the buffer,
    /// so it tracks the screen.
    #[must_use]
    pub fn read_spell(&self, domain: &str, lines: &[String]) -> Vec<crate::tower::spell::Reading> {
        crate::tower::spell::interpret(&self.world, domain, lines, lines)
    }

    /// The same, with a reader for the lines the orb cannot read itself.
    ///
    /// [`read_spell`](Self::read_spell) is this with none, so a build without a
    /// reader shows exactly what it always did.
    ///
    /// The save's reading, by
    /// [`write_spell_reading`](Self::write_spell_reading)'s lookup, so the
    /// editor and the runner agree; a line edited since is read now.
    #[must_use]
    pub fn read_spell_with(
        &self,
        name: &str,
        domain: &str,
        lines: &[String],
        scrivener: &dyn crate::Scrivener,
    ) -> Vec<crate::tower::spell::Reading> {
        let read = self.reading_of(&crate::content::with_extension(name), lines, scrivener);
        crate::tower::spell::interpret(&self.world, domain, lines, &read)
    }

    /// Which line of `name` a running invocation is on, if one is running.
    ///
    /// The editor's marker, so a running spell is not edited blind.
    #[must_use]
    pub fn running_line(&self, name: &str) -> Option<u64> {
        let wanted = crate::content::with_extension(name);
        self.world
            .iter_entities()
            .filter(|entity| {
                entity
                    .get::<tower::Name>()
                    .is_some_and(|node| node.0 == wanted)
            })
            .find_map(|entity| entity.get::<tower::spell::Running>())
            .and_then(tower::spell::line_of)
    }

    /// A spell the orb has been asked to open, if any.
    ///
    /// Takes rather than reads; see [`Opening`](crate::execute::Opening).
    pub fn opening(&mut self) -> Option<crate::execute::Request> {
        self.world.resource_mut::<crate::execute::Opening>().take()
    }

    /// Whether a `scribe` is waiting, without taking it.
    ///
    /// A peek, since `opening`'s `&mut` stamps the change tick and leaves
    /// `resource_changed::<Tower>` true for ever.
    #[must_use]
    pub fn has_opening(&self) -> bool {
        self.world
            .resource::<crate::execute::Opening>()
            .is_pending()
    }

    /// Whether an `unfurl` is waiting, without taking it. See [`Sim::has_opening`].
    #[must_use]
    pub fn is_unfurling(&self) -> bool {
        self.world.resource::<crate::execute::Unfurling>().pending()
    }

    /// Whether `unfurl` has asked for the transcript to take the keyboard.
    ///
    /// Takes rather than reads, as [`Sim::opening`] does.
    pub fn unfurling(&mut self) -> bool {
        self.world
            .resource_mut::<crate::execute::Unfurling>()
            .take()
    }

    /// Whether a `quit` is waiting, without taking it. See [`Sim::has_opening`].
    #[must_use]
    pub fn is_quitting(&self) -> bool {
        self.world.resource::<crate::execute::Quitting>().pending()
    }

    /// Whether `quit` has asked for the session to end.
    ///
    /// Takes rather than reads. What leaving means is the frontend's.
    pub fn quitting(&mut self) -> bool {
        self.world.resource_mut::<crate::execute::Quitting>().take()
    }

    /// Whether the orb has asked *are you sure* and is waiting to be told again.
    ///
    /// Peeked, never taken: it is state a screen can draw, not a request.
    #[must_use]
    pub fn is_asking_to_quit(&self) -> bool {
        self.world
            .resource::<crate::execute::Quitting>()
            .is_asking()
    }

    /// Whether a `menu` is waiting, without taking it. See [`Sim::has_opening`].
    #[must_use]
    pub fn has_menuing(&self) -> bool {
        self.world.resource::<crate::execute::Menuing>().pending()
    }

    /// Whether `menu` has asked for the orb's menu.
    ///
    /// Takes rather than reads, like the handshakes above it.
    pub fn menuing(&mut self) -> bool {
        self.world.resource_mut::<crate::execute::Menuing>().take()
    }

    /// Whether `weave` has asked for the progression screen.
    ///
    /// Peeked rather than taken, so a system can decide whether to run without
    /// stamping the resource's change tick — see [`Sim::has_opening`].
    #[must_use]
    pub fn has_weaving(&self) -> bool {
        self.world.resource::<crate::execute::Weaving>().pending()
    }

    /// Take `weave`'s pending request, if there is one.
    pub fn weaving(&mut self) -> bool {
        self.world.resource_mut::<crate::execute::Weaving>().take()
    }

    /// Walk the archive's stacks one cell, now (§10, §19).
    ///
    /// Its own entry point, since a maze walked at 1 Hz is a wait: the reading
    /// moves and no tick is consumed. Recorded as [`Submission::Walked`], which
    /// replays after its tick's step; a tick never holds a walk and a typed
    /// line, as the prompt is dead while the arrows have the maze.
    ///
    /// Returns whether the stacks were open to walk at all.
    pub fn walk(&mut self, way: tower::Way) -> bool {
        let Some(lectern) = crate::execute::stacks(&self.world) else {
            return false;
        };
        if self.world.get::<tower::Maze>(lectern).is_none() {
            return false;
        }
        let tick = *self.world.resource::<Tick>();
        self.world
            .resource_mut::<Submissions>()
            .walked(tick, way.word());
        // The same body `follow` runs, so a hand-walked maze and a spell-walked
        // one cannot disagree about a wall or about what the exit is worth.
        crate::execute::tread(&mut self.world, way);
        true
    }

    /// Apply one recorded submission, with this sim already on its tick.
    ///
    /// The one replay driver. A variant means *when it ran*:
    ///
    /// - [`Typed`](Submission::Typed) was queued during its tick and executes at
    ///   the start of the next.
    /// - [`Wrote`](Submission::Wrote) is the same: a save queues like any effect.
    /// - [`Walked`](Submission::Walked) already ran, during its tick, after that
    ///   tick's step — because [`Sim::walk`] does not wait for a clock.
    ///
    /// Standing on the recorded tick is correct for each.
    pub fn replay(&mut self, submission: Submission) {
        match submission {
            Submission::Typed(line) => self.submit(&line),
            // The model does not run again: the canonical form it settled on is
            // what replays, so a session stays reproducible on a machine whose
            // GPU would have read the player's words differently.
            Submission::Divined { line, echo } => self.submit_divined(&line, &echo),
            // The reading is applied, never re-derived: re-running the reader
            // would make a replay depend on a model being present and
            // identical. `read` equals `lines` for every session nothing read.
            Submission::Wrote {
                name,
                lines,
                read,
                by,
            } => self.queue_write(&name, &lines, read, by),
            Submission::Took(id) => self.take(&id),
            Submission::Walked(word) => {
                if let Some(way) = tower::Way::named(&word) {
                    self.walk(way);
                }
            }
        }
    }

    /// Whether `wander` has asked for the arrow keys (§10, §19).
    #[must_use]
    pub fn has_wandering(&self) -> bool {
        self.world.resource::<crate::execute::Wandering>().pending()
    }

    /// Take `wander`'s pending request, if there is one.
    pub fn wandering(&mut self) -> bool {
        self.world
            .resource_mut::<crate::execute::Wandering>()
            .take()
    }

    /// The Ley Line, against what the tower has earned (§11.5).
    #[must_use]
    pub fn ley_line(&self) -> Vec<tower::Station> {
        tower::ley_line(&self.world)
    }

    /// The last total the Ley Line is authored to.
    #[must_use]
    pub fn scale(&self) -> u64 {
        tower::scale(&self.world)
    }

    /// Where the tower's total stands against the Ley Line's next station.
    #[must_use]
    pub fn toward_station(&self) -> tower::Toward {
        tower::ley::toward(&self.world)
    }

    /// Where renown stands against the next rank.
    #[must_use]
    pub fn toward_rank(&self) -> tower::Toward {
        tower::renown::toward(&self.world)
    }

    /// What the tower is called now, if it has earned a name.
    #[must_use]
    pub fn rank(&self) -> Option<String> {
        tower::renown::rank(&self.world)
    }

    /// One mastery line per domain, in `DOMAINS` order.
    #[must_use]
    pub fn mastery(&self) -> Vec<tower::Line> {
        tower::mastery(&self.world)
    }

    /// Which mastery stations the tower has reached, in order.
    #[must_use]
    pub fn reached(&self) -> &[String] {
        self.world.resource::<tower::mastery::Reached>().ids()
    }

    /// Whether the tower has opened `key` — a room, a recipe, a charm, the wall.
    #[must_use]
    pub fn has_opened(&self, key: &str) -> bool {
        self.world.resource::<tower::Opened>().has(key)
    }

    /// Whether the room named may be entered.
    #[must_use]
    pub fn is_open(&self, domain: &str) -> bool {
        self.world.resource::<tower::Opened>().is_open(domain)
    }

    /// Whether this tower began sealed — part of its recorded start.
    #[must_use]
    pub fn began_sealed(&self) -> bool {
        self.world.resource::<tower::Sealing>().0
    }

    /// The room the player is standing in, by name — `None` at `/tower` or
    /// above it, where no work happens.
    ///
    /// The room, not the leaf: the alembic is in the laboratory. `location` is
    /// the path.
    #[must_use]
    pub fn domain(&self) -> Option<String> {
        let cwd = self.world.resource::<tower::Cwd>().0;
        tower::domain_of(&self.world, cwd)
            .and_then(|node| self.world.get::<tower::Name>(node))
            .map(|name| name.0.clone())
    }

    /// How many times `key` has been counted — `made:clarity`, `potion`,
    /// `at:stacks`, `event:figure`.
    #[must_use]
    pub fn tally(&self, key: &str) -> u32 {
        self.world.resource::<tower::Tally>().count(key)
    }

    /// Every ledger row, counted: what the tower has done in all, and how much
    /// of it a spell did (§19, *number go up*).
    #[must_use]
    pub fn ledger(&self) -> Vec<tower::LedgerRow> {
        tower::ledger(&self.world)
    }

    /// The row `room`'s column shows, counted alone.
    #[must_use]
    pub fn ledger_of(&self, room: &str) -> Option<tower::LedgerRow> {
        tower::ledger_of(&self.world, room)
    }

    /// Every open room's rate and what it waits on — `status rates`, and what
    /// the balance sweep reports.
    #[must_use]
    pub fn yields(&self) -> Vec<tower::Yield> {
        tower::yields(&self.world)
    }

    /// What holds the tower's one production slot.
    #[must_use]
    pub fn slot(&self) -> Option<tower::Slot> {
        tower::slot(&self.world)
    }

    /// How fast `key` has been made, from its gaps. A forecast reads this.
    #[must_use]
    pub fn pace_of(&self, key: &str) -> Option<tower::Pace> {
        self.world.resource::<tower::Throughput>().pace(key)
    }

    /// How fast experience has been earned, lately.
    #[must_use]
    pub fn earning(&self) -> Option<tower::Pace> {
        self.world.resource::<tower::Throughput>().earning()
    }

    /// Queue a tester's `debug_spell`.
    ///
    /// Runs now and records the write, since [`write_spell`](Self::write_spell)
    /// pushes a `Wrote` — see `execute::debug_spell`. Not in the parse trace.
    #[cfg(debug_assertions)]
    fn debug_spell(&mut self, line: &str, order: &crate::execute::SpellOrder) {
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Input)
            .text(orbs_render::FieldName::Message, line)
            .finish();
        if let Some((name, lines)) = crate::execute::run_spell_order(&mut self.world, order) {
            self.write_spell(&name, &lines);
        }
    }

    /// Answer a numbered prompt.
    ///
    /// Not in the parse trace: a digit is not a phrasing, but evidence for the
    /// ambiguous line (§15).
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

    // No `scrollback_mut`: a frontend writing to the log directly would make a
    // session replay cannot reproduce (§13). Tests use `Sim::world_mut`.

    /// Leave echoes that only repeat the player's line out of the transcript.
    ///
    /// A view preference that writes no record, so replay is untouched.
    /// Touches the world only when it changes, so a frontend may ask every
    /// frame.
    pub fn set_terse_echo(&mut self, terse: bool) {
        if self.scrollback().records().terse_echo() != terse {
            self.world
                .resource_mut::<Scrollback>()
                .records_mut()
                .set_terse_echo(terse);
        }
    }

    /// What is in flight, if anything (§5.0).
    ///
    /// One at a time: §11.5 opens at multiplex capacity 1 and §9's fourth
    /// invariant reserves that slot for the action's whole duration.
    ///
    /// `iter_entities`, since a query needs `&mut World` and a paint holds the
    /// sim shared. It walks resource entities too (Bevy 0.19), harmless while
    /// no resource can carry `Working`.
    #[must_use]
    pub fn working(&self) -> Option<tower::Working> {
        self.world
            .iter_entities()
            .find_map(|entity| entity.get::<tower::Working>().copied())
    }

    /// The instruments where the player is standing, for §10.1's panel.
    ///
    /// Empty anywhere but the laboratory, so the panel is a property of where
    /// you are.
    #[must_use]
    pub fn instruments(&self) -> Vec<tower::Instrument> {
        tower::instruments(&self.world)
    }

    /// The stacks the player is standing over, if they are open (§10, §19).
    ///
    /// Relative to `Cwd`, as [`Sim::instruments`] is. Built fresh, once a
    /// tick, which is cheap at 1 Hz.
    #[must_use]
    pub fn stacks(&self) -> Option<orbs_render::Stacks> {
        let stacks = crate::execute::stacks(&self.world)?;
        self.world.get::<tower::Maze>(stacks).map(tower::Maze::view)
    }

    /// The readings the orb is waiting for the player to pick between (§6).
    #[must_use]
    pub fn choices(&self) -> &Choices {
        self.world.resource::<Choices>()
    }

    /// Replace the orb's voice — CLAUDE.md rule 6's hot reload.
    ///
    /// A frontend owns the watcher (rules 3 and 8) and calls this between
    /// steps. Replay-safe, since no line reaches a decision;
    /// [`Topics`](crate::tower::Topics) is snapshotted at construction, so a
    /// new subject needs a relaunch and new text does not.
    pub fn set_prose(&mut self, prose: Prose) {
        self.world.insert_resource(prose);
    }

    /// The orb's voice, for the handful of lines a frontend must speak.
    ///
    /// For a screen the sim has no record for, like §9's "window too small",
    /// so it is still rule 6's prose.
    #[must_use]
    pub fn prose(&self) -> &Prose {
        self.world.resource::<Prose>()
    }

    /// Replace the manual, for `ORBS_CONTENT`.
    ///
    /// [`set_prose`](Self::set_prose)'s pair: rule 6 wants text edited without
    /// a recompile.
    pub fn set_manual(&mut self, manual: crate::content::Manual) {
        self.world.insert_resource(manual);
    }

    /// The manual a reader is drawn from.
    #[must_use]
    pub fn manual(&self) -> &crate::content::Manual {
        self.world.resource::<crate::content::Manual>()
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
    /// §3's high-threat tonal register, reachable directly so the typefaces can
    /// be seen. The content never changes (§3), only the face, and log lines
    /// refuse the eldritch one.
    pub fn set_register(&mut self, register: Presentation) {
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .set_register(register);
    }

    /// What the prompt reads, before the caret.
    ///
    /// `<name> $ `. Composed here rather than in a view because the name is
    /// world state (see [`Wizard`]) and both frontends must show the same one.
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
    /// What the prompt shows, and so what tells a player which commands will
    /// resolve.
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
    /// §15's raw material: with eight testers 84% against 86% is noise, and
    /// *which* readings lost is not.
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
    pub const fn world(&self) -> &World {
        &self.world
    }

    /// Mutable access to the world, for setup: inserting resources, spawning the
    /// initial tower, applying a loaded save.
    ///
    /// This is a deliberate escape hatch. Mutating the world *between* steps is
    /// fine; anything that makes two runs from the same seed diverge is not.
    pub const fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::RngStream;
    use rand::RngExt;

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
