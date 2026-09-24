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
    /// An open tower — every room, gated recipe and charm — and the curve
    /// exactly as authored, which is what every test, dump, balance policy and
    /// the `screens` example measures against. A fresh *game* is
    /// [`sealed`](Self::sealed), and its length arrives through
    /// [`begun`](Self::begun) rather than here, or every pinned rate in the
    /// workspace would silently measure a different curve.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self::build(seed, false, crate::content::Length::Baseline, |_| {})
    }

    /// Create a simulation that begins as a laboratory and nothing else.
    ///
    /// The tower a fresh game builds (§11.5): what a station on either track
    /// opens is shut until it is reached, and everything nothing opens is open —
    /// see `tower::Opened::start`. Whether a tower began this way travels in its
    /// save, because a sealed and an open tower with the same seed and
    /// submissions diverge at the first `attend archive`.
    #[must_use]
    pub fn sealed(seed: u64) -> Self {
        Self::build(seed, true, crate::content::Length::Baseline, |_| {})
    }

    /// An open tower at a chosen length — what the balance harness measures.
    ///
    /// `new` is this at [`Baseline`](crate::content::Length::Baseline), and
    /// [`begun`](Self::begun) is a *game* and therefore sealed. A policy needs
    /// every room open, so neither can measure the curve a player at medium is
    /// actually climbing — and unmeasured, the shipped game runs a curve the
    /// instrument never sees.
    #[must_use]
    pub fn measured(seed: u64, length: crate::content::Length) -> Self {
        Self::build(seed, false, length, |_| {})
    }

    /// The tower a fresh *game* builds: sealed, and as long as the player asked.
    ///
    /// One constructor, because a fresh game is sealed *and* has a length and a
    /// `Sim::paced` beside `Sim::sealed` could not say both. `new` and `sealed`
    /// keep their signatures — over three hundred call sites between them, and
    /// none is a game.
    #[must_use]
    pub fn begun(seed: u64, length: crate::content::Length) -> Self {
        Self::build(seed, true, length, |_| {})
    }

    /// Create a simulation and populate its schedule.
    ///
    /// The schedule is built here and never exposed afterwards, so systems cannot
    /// be added behind the sim's back at runtime.
    ///
    /// A frontend must not call this. It is for tests that observe the
    /// schedule's ordering from outside — see `tower::scene`, where a caller's
    /// system ran *before* the scene rebuild and the topsort was within its
    /// rights. Domain systems belong inside [`Sim::new`]: if each frontend
    /// registered its own they would be three different games (§13). The seam is
    /// open only because closing it would cost the ordering test its handle.
    ///
    /// # Panics
    ///
    /// If the built-in content files disagree — today, if `progression.toml`
    /// does not price every instrument `recipes.toml` names. Both ship in the
    /// binary, so it is an authoring error no input can reach, and failing as
    /// `load::builtin` does beats a tower whose work is worth nothing.
    #[must_use]
    pub fn with_schedule(seed: u64, build: impl FnOnce(&mut Schedule)) -> Self {
        Self::build(seed, false, crate::content::Length::Baseline, build)
    }

    /// The one construction behind [`new`](Self::new), [`sealed`](Self::sealed)
    /// and [`with_schedule`](Self::with_schedule).
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
                world.resource::<crate::content::Progression>(),
            );
            world.insert_resource(start);
            world.insert_resource(tower::Sealing(true));
        }
        // The walls must say how they stand from tick 0. `erode` publishes
        // `integrity` only when the number moves, and it does not move for
        // thirty ticks — so `many_at` read the absent child as nought and `if
        // the pylon has fewer than 60 integrity` fired on a whole barrier.
        // Here rather than in `tower::build`, which knows no resources.
        if let Some(pylon) = tower::pylon::fixture(&world) {
            crate::execute::publish_pylon(&mut world, pylon);
        }
        // And what each die costs, for the same reason. A price is a fact about
        // the die rather than about a fight, but only `defend::publish` raised
        // it and nothing calls that until the first bailey verb — so every
        // solver's affordability guard compared nought against nought and
        // afforded it on a tower with no pool.
        crate::execute::publish_dice(&mut world);
        // ...and the forge's, for the same reason: an unpublished charm node
        // answers *nought* rather than *nothing*, so a maintenance spell's `if
        // the hurried has no graced` is true of a tool charmed right now.
        // `lapse_charms` rather than `refresh_lattice`, which resolves the forge
        // through `Cwd` — the root at construction, so it would do nothing.
        //
        // `seal` last, after every publisher above: it marks the nodes that
        // exist when it runs, and those calls *create* nodes. Sealing first left
        // `pylon/integrity` and each die's `quintessence` unmarked in a fresh
        // tower while the same tower reloaded from its save had them — `restore`
        // seals last — and two sabotage queries read that component.
        tower::seal(&mut world);
        tower::rebuild(&mut world);
        tower::report(&mut world);

        Self {
            world,
            commands,
            schedule,
            scene,
        }
    }

    /// Every resource a world needs, and no tower in it.
    ///
    /// A function rather than the top of [`Sim::with_schedule`] because there
    /// are two ways to reach a world — raising one and loading one — and §13
    /// holds that two constructions of one world are two different games. Thirty
    /// lines of `init_resource` is where an omission hides, so both callers get
    /// this one list.
    ///
    /// Deliberately *not* a `Default`: it takes the seed, and a world with an
    /// unseeded RNG is broken rather than lesser.
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
        // Recipes' tier too: a weight decides what a run earns and a threshold
        // gates a verb, so a mid-session swap would break replay.
        //
        // Checked against the recipes, hence loaded after them: its keys are
        // instrument names and nothing in Rust knows what those are, so
        // authoring the two files to disagree panics as `load::builtin` does.
        //
        // Stretched before it is checked, because `check`'s `ascends` gate has
        // to run against the curve the world will use. Every length survives it
        // — a strictly increasing sequence times a non-decreasing positive one
        // is strictly increasing.
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
            let catalogue = crate::content::Catalogue {
                instruments: &instruments,
                outputs: &outputs,
                gated: &gated,
                charms: &charms,
            };
            if let Err(error) = curve.check(&catalogue) {
                panic!("the built-in content is authored with the crate: {error}");
            }
        }
        world.insert_resource(curve);
        world.init_resource::<tower::Experience>();
        world.init_resource::<tower::Renown>();
        world.init_resource::<tower::siege::Petitioned>();
        world.init_resource::<tower::Stores>();
        // Everything open, the tower every test, dump and policy uses. A fresh
        // *game* starts sealed — see `Sim::sealed`.
        let opened = tower::Opened::all(
            world.resource::<Recipes>(),
            world.resource::<crate::content::Charms>(),
        );
        world.insert_resource(opened);
        world.init_resource::<tower::Sealing>();
        world.init_resource::<tower::Tally>();
        world.init_resource::<tower::mastery::Reached>();
        // Whole, by `Default`. A tower is not built already crumbling.
        world.init_resource::<tower::Integrity>();
        // Full, not empty, for the reason `Integrity` opens whole: the wizard
        // has tended this place for years, and nought would mean twelve minutes
        // of inert forge before the game had a decision in it. After
        // `Integrity` because the ceiling reads it, and before the ceiling is
        // read because that reads what the Ley Line's forks granted — a
        // courtesy, since the readers answer nought for a missing resource.
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
    /// Split out beside [`bare`](Self::bare) for the same reason: the order
    /// below is load-bearing — two systems draw from one RNG stream — so it is
    /// written down in one place. A loaded world runs the same three passes as a
    /// raised one or it is a different game.
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
        // `spell::advance` before `finish`, so a spell sees the world as the
        // previous tick left it; after, a script would start the next stage on
        // the tick the previous one landed — a free tick no manual player gets,
        // at concentration 0 where §19 says nothing may.
        //
        // `spell::stand` last, so a spell that ran off the end this tick is cast
        // again on the next rather than inside the same pass — otherwise a held
        // spell gets two goes at the budget in one tick.
        schedule.add_systems(
            (
                tower::spell::advance,
                tower::burn,
                tower::finish,
                tower::drift,
                // After `drift`, and load-bearing: both draw once per tick from
                // `RngStream::Threat`, so reordering them would silently change
                // every existing replay. Appended, never inserted.
                tower::substitution,
                // After the roll, and it draws nothing: a lie settling is a
                // clock reading, so appending it perturbs no replay.
                tower::settling,
                // The same licence: a barrier wearing down compares two ticks,
                // and the sanctum's one draw is in `height_for` inside `muster`
                // rather than in a system.
                tower::erode,
                // The menagerie has no system here and once had one: a beast at
                // the circle waits for ever, so there is nothing to advance
                // (§19), and its one draw is `Beast::draw` inside `summon`.
                //
                // The same licence for this one: quintessence coming back is a
                // modulo on the tick, and the forge's draw is in `imbue`.
                //
                // After `erode`, because the ceiling is a function of integrity:
                // on a tick where both fire the pool is capped against the
                // barrier as the wear left it — the ordering defect §19 records
                // the sanctum paying for twice.
                tower::regenerate,
                // The same licence, and load-bearing rather than tidy: without
                // a system saying a charm lapsed, `ebbing` would arrive only
                // when a forge verb ran, and the maintenance spell is built
                // entirely on it arriving.
                crate::execute::lapse_charms,
                // `lapse_charms`'s reason again: a store running down draws
                // nothing, and without a system saying so `thin` would arrive
                // only when a bailey verb ran, so a spell keeping its own
                // stores up could not work.
                //
                // It writes only when a word changes, so an idle tick issues no
                // `NodeId` and §19's *insertion order is the parse* holds
                // between a watched hour and a `meditate`-collapsed one.
                tower::stocktake,
                tower::spell::stand,
            )
                .chain(),
        );
        build(&mut schedule);

        // A third pass, for the reason `commands` is a first one: Bevy's topsort
        // was in fact running the caller's systems before `rebuild` despite the
        // insertion order, and a flip there leaves the scene a tick stale. Every
        // frontend passes a different `build`, so the two graphs could flip
        // differently — the game/harness divergence §13 exists to prevent.
        //
        // Last, so the scene names the world as the tick left it.
        let mut scene = new_sim_schedule();
        scene.add_systems(tower::rebuild);

        (commands, schedule, scene)
    }

    /// Read this world out as a save document.
    ///
    /// Takes `&self` as a guarantee: a `&mut` could build a `QueryState`, which
    /// registers components and moves archetypes, and a save that changed what
    /// it measured would make `tests/persistence.rs` measure itself.
    ///
    /// Take it at a tick boundary — §8: *"saves are permitted only at tick
    /// boundaries."* Immediately after [`step`](Self::step) is where both
    /// frontends take theirs.
    ///
    /// Writing it to a file is a frontend's (`orbs_shell::save`); this crate
    /// never touches the filesystem.
    #[must_use]
    pub fn snapshot(&self) -> crate::save::Save {
        crate::save::capture(&self.world)
    }

    /// Build a world from a save document.
    ///
    /// Raises the tower and applies the save over it, so a save written before a
    /// domain existed opens into a tower that has one — `crate::save::restore`
    /// carries the reasoning. The result stands exactly where the saved world
    /// stood: same tick, same eight stream positions, same work in flight, and
    /// `tests/persistence.rs` holds that stepping either gives the same world.
    ///
    /// # Panics
    ///
    /// As [`with_schedule`](Self::with_schedule) does: the built-in content is
    /// authored with the crate.
    #[must_use]
    pub fn restored(save: &crate::save::Save) -> Self {
        // The length comes out of the save: `bare` builds the curve, so a
        // restore that did not hand it the saved length would install the
        // *authored* one — thresholds re-derived, stations re-crossed, rooms
        // opened that should not be. `persistence.rs`'s completeness lint walks
        // components, not resources, so nothing would have caught it.
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

        // No `tower::report` here, unlike `with_schedule`: it would push §4's
        // condition report onto the stream `restore` just rebuilt, so a loaded
        // world would carry thirty-odd records the world that wrote it never
        // had. A restored world is the saved world, exactly.
        //
        // Saying *"you are back, and the orb was dark for three hours"* is a
        // frontend's line — the sim has no wall clock and must not acquire one —
        // and the save carries the tail of the stream anyway, so the transcript
        // is the screen they left.

        Self {
            world,
            commands,
            schedule,
            scene,
        }
    }

    /// Say that this tower was resumed, and how long it was dark.
    ///
    /// Rule 6 puts prose in content files, so the words are the sim's; §19
    /// forbids the sim reading a wall clock, so the *gap* is the frontend's.
    ///
    /// Not inside [`restored`](Self::restored), because a restored world must be
    /// the saved world exactly and `tests/persistence.rs` compares the two
    /// documents byte for byte. Saying so is a thing a *session* does.
    ///
    /// `away` is seconds, or `None` where the machine would not say — a save
    /// written before the stamp existed, or a clock gone backwards.
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
    /// A record rather than a `status` line: §3 makes the stream *the* output,
    /// so anything that never becomes a record is invisible to `sift`, the log
    /// and the screen reader — and with no `save` verb (§19) a player has no
    /// reason to look.
    ///
    /// The caller says this once. A save is attempted every sixty ticks, so a
    /// line per failed attempt is sixty an hour.
    pub fn say_save_failed(&mut self) {
        let message = self.world.resource::<Prose>().line("save_failed", &[]);
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Message)
            .text(orbs_render::FieldName::Message, &message)
            .role(orbs_render::Role::Danger)
            .finish();
    }

    /// Say that a save was there and could not be read.
    ///
    /// A first launch says nothing — there is nothing to say — but a tower that
    /// did not come back is owed a reason, and the log is not where a player
    /// looks.
    pub fn say_save_unreadable(&mut self) {
        let message = self.world.resource::<Prose>().line("save_unreadable", &[]);
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Message)
            .text(orbs_render::FieldName::Message, &message)
            .role(orbs_render::Role::Danger)
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
    /// The second entry point, and the only other one. Unlike [`Sim::step`] it
    /// does not advance world time: it echoes immediately and queues any
    /// resolved command for the next tick. See [`session`](crate::session) for
    /// why the two clocks are split, and DESIGN.md §19 for the rule it is
    /// measured against.
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

        // A tester's door, and only in a build a tester runs. Matched exactly
        // and before the parser: `debug_spawn` is not in §6's vocabulary, so
        // the fuzzy matcher must never see it and `Verb::ALL` must never grow
        // it. In a release binary this is an ordinary unresolvable line.
        #[cfg(debug_assertions)]
        if let Some(order) = crate::execute::spawn_order(line) {
            self.debug_spawn(line, order);
            return;
        }

        #[cfg(debug_assertions)]
        if let Some(order) = crate::execute::spell_order(line) {
            self.debug_spell(line, &order);
            return;
        }

        #[cfg(debug_assertions)]
        if crate::execute::giveaway(line) {
            self.debug_ward(line);
            return;
        }

        #[cfg(debug_assertions)]
        if crate::execute::shortcut(line) {
            self.debug_course(line);
            return;
        }

        #[cfg(debug_assertions)]
        if crate::execute::beleaguered(line) {
            self.debug_siege(line);
            return;
        }

        #[cfg(debug_assertions)]
        if crate::execute::beckoned(line) {
            self.debug_circle(line);
            return;
        }

        #[cfg(debug_assertions)]
        if crate::execute::swapping(line) {
            self.debug_swap(line);
            return;
        }

        #[cfg(debug_assertions)]
        if let Some(lesson) = crate::execute::lesson(line) {
            self.debug_learn(line, lesson.name.as_deref());
            return;
        }

        #[cfg(debug_assertions)]
        if let Some(id) = crate::execute::taking(line) {
            self.debug_take(line, id.as_deref());
            return;
        }

        #[cfg(debug_assertions)]
        if let Some(id) = crate::execute::reaching(line) {
            self.debug_reach(line, id.as_deref());
            return;
        }

        #[cfg(debug_assertions)]
        if let Some(total) = crate::execute::standing(line) {
            self.debug_renown(line, total);
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
    /// Tier one is decided before a reader is consulted, so nothing that
    /// resolves today can regress; `scripts/dumps.sh` is the `diff` that proves
    /// it. The double `analyse` — once to decide, once in `submit` to act — is
    /// pure and under a millisecond, where threading a half-finished analysis
    /// through the entry point every other caller uses would make `submit` mean
    /// two things.
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

    /// Run a line the augury read, rather than one the orb read (§6).
    ///
    /// The fifth entry point, and the augury's only one: the caller has
    /// established that neither [`crate::parser::is_literal`] nor
    /// [`Analysis::reads_outright`] settles the line, asked a model what it
    /// meant, and expanded the answer into the canonical command `echo`.
    ///
    /// The model runs once, here. `echo` is what is analysed, recorded and
    /// replayed; `line` is kept for the transcript and never re-read. That is
    /// the determinism boundary: [`analyse`] is pure and integer-scored, while
    /// re-deriving from the player's own words would run a model whose spans
    /// need not match across a GPU, a driver or a backend. See
    /// [`Submission::Divined`].
    ///
    /// Not [`submit`](Self::submit), because the transcript shows what the
    /// *player* wrote, the echo carries [`Confidence::Divined`] so it draws `≈`
    /// and the destructive guard can tell an inferred `purge` from a typed one,
    /// and the journal records both halves. A command that fails to resolve
    /// falls through as a typed one would; §6 forbids a bare error.
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
        // Stamped here, not derived from the confidence: only a `Resolved`
        // reading carries `Divined`, so deriving it would silently drop the
        // augury's failures — and an `Elsewhere` or `Unresolved` canonical
        // command is the most interesting row in the export. The consultation
        // is the fact worth recording, not its outcome.
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
    /// The third entry point, and the last. [`submit`](Self::submit) takes a
    /// line the player typed; this takes a file they wrote. Both are
    /// *decisions*, the test for what belongs in [`Submissions`] and therefore
    /// in a replay — the keystrokes that built the buffer are not, which is why
    /// the editor lives in the frontend. Like `submit`, it does not advance
    /// world time: a world mutated from inside an input call produces a session
    /// `(seed, submissions)` cannot reproduce, so the write is queued.
    ///
    /// The *typed* lines are recorded, not the canonical form they become, so
    /// improving the canonicaliser cannot make an old session replay into a
    /// different world.
    pub fn write_spell(&mut self, name: &str, lines: &[String]) {
        self.write_spell_reading(name, lines, &crate::augur::Verbatim);
    }

    /// Write a spell out, with a reader for the lines the orb cannot read.
    ///
    /// [`write_spell`](Self::write_spell) is this with a reader that abstains on
    /// everything, so a build with none is the game exactly as it was.
    ///
    /// What the player typed is never touched: `lines` goes to
    /// [`Held`](crate::tower::Held) byte-exact (§19), and the reading lands in
    /// [`Read`](crate::tower::Read) beside it — derived, discardable, rebuilt
    /// whenever a line changes, and what `spell::compile` compiles.
    ///
    /// Read once on the way in, not at cast and not in `step`: `orbs-sim` cannot
    /// depend on a model (rule 1) and a model on the tick spine would break
    /// replay (rule 3), so the reader is the frontend's —
    /// [`submit_reading`](Self::submit_reading)'s shape. Unchanged text keeps
    /// its reading, which matters because the editor saves after every pause.
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
    /// The newest reading of the file, queued or landed. A write lands on the
    /// next tick, so on the beat the editor saves and then reads its buffer the
    /// node still holds the older reading; asking the queue first reads a
    /// changed line once rather than twice, and makes the save's reading and the
    /// editor's one lookup.
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
    /// [`write_spell`](Self::write_spell)'s shape: recorded as a
    /// [`Submission`] and queued as an effect, so it lands on a tick boundary
    /// like everything else a player decides. The id, not the keystrokes —
    /// aiming the cursor changes no state the world can see.
    ///
    /// The world re-checks every rule before granting; see `execute::weave`.
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
    /// The question a spell's `if` asks, asked directly. Every other route to a
    /// published reading goes through `survey`, which costs a tick — so four
    /// surveys in a row read four different moments back when the menagerie's
    /// aperture moved every tick. A reading asked without a tick is the honest
    /// instrument.
    ///
    /// Walked from the root rather than from `Cwd`, so it answers about a room
    /// nobody is standing in — which is where a bound solver always is.
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
    /// Reads `Cwd`, as [`stacks`](Self::stacks) does: a frontend gets the board
    /// only where the player could `survey prism` themselves, so the picture
    /// cannot outrun the readings by following them out of the room.
    #[must_use]
    pub fn ward(&self) -> Option<orbs_render::Board> {
        let cwd = self.world.resource::<tower::Cwd>().0;
        tower::children_of(&self.world, cwd)
            .into_iter()
            .find_map(|node| self.world.get::<tower::Ward>(node))
            .map(tower::Ward::view)
    }

    /// The course itself, for a caller that needs to ask it questions.
    ///
    /// `orbs-balance` is the caller, and not [`pylon`](Self::pylon) because a
    /// *view* is what a painter needs and a *course* what a solver needs. The
    /// harness asks `Course::between` which way each haul runs, keeping one copy
    /// of the algorithm in the game — see `drive::haul_one`.
    #[must_use]
    pub fn course(&self) -> Option<&tower::Course> {
        self.world
            .get::<tower::Course>(self.here_with::<tower::Course>()?)
    }

    /// The child of where the player stands that carries `C`, if any.
    ///
    /// One walk, because this was three: `course`, `pylon` and `debug_course`
    /// each wrote out the same `children_of(cwd).find(...)`, and three copies of
    /// a rule about where the player stands are three chances to disagree — the
    /// shape [`tower::reach`] was extracted to stop for *names*.
    ///
    /// Room-scoped on purpose. `tower::pylon::fixture` is the tower-wide
    /// question and stays separate: a bound solver's readings must keep up while
    /// the player is in another room, and a picture must not.
    fn here_with<C: bevy_ecs::component::Component>(&self) -> Option<bevy_ecs::entity::Entity> {
        let cwd = self.world.resource::<tower::Cwd>().0;
        tower::children_of(&self.world, cwd)
            .into_iter()
            .find(|node| self.world.get::<C>(*node).is_some())
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
    /// Reads `Cwd` for the reason [`pylon`](Self::pylon) does: the picture
    /// cannot outrun the readings by following the player out of the room.
    ///
    /// A finished siege still draws: `settle` leaves the board up, since one
    /// that vanished on the winning round would take the postmortem with it.
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
        Some(siege.view(tally, self.world.resource::<tower::Quintessence>().get()))
    }

    /// The beast waiting at the menagerie's circle, if the player is looking at it.
    ///
    /// [`course`](Self::course)'s reason, one room over: a *view* is what a
    /// painter needs and a *beast* what a solver needs. `orbs-balance` reads
    /// where two glyphs stand on every step of `taming`, and building the whole
    /// board three times a step to find them was the harness's hot path.
    #[must_use]
    pub fn beast(&self) -> Option<&tower::circle::Beast> {
        self.world
            .get::<tower::circle::Beast>(self.here_with::<tower::circle::Beast>()?)
    }

    /// The board for the beast waiting at the menagerie's circle, if the player
    /// is looking at it.
    ///
    /// Reads `Cwd` for the reason [`pylon`](Self::pylon) does: the picture cannot
    /// outrun the readings by following the player out of the room.
    #[must_use]
    pub fn circle(&self) -> Option<orbs_render::Circle> {
        Some(tower::circle::view(self.beast()?, self.prose()))
    }

    /// Every domain at a glance — what §9's rail draws.
    ///
    /// Derived here rather than in a frontend: rule 2 lets a frontend decide
    /// only how a cell is drawn, and two derivations of *what is happening in
    /// the forge* are two answers that can disagree.
    ///
    /// Always seven, in a fixed order, including rooms the tower has not built
    /// yet — see [`tower::briefs`].
    #[must_use]
    pub fn briefs(&self) -> Vec<tower::Brief> {
        tower::briefs(&self.world)
    }

    /// The spells the orb is holding, named, in the order the tower keeps them.
    ///
    /// §8 wants concentration *"surfaced in `status` and in the sidebar — never a
    /// quiet log line"*, and a count cannot answer what a player at capacity is
    /// actually asking, which is *which one*.
    #[must_use]
    pub fn bound(&self) -> Vec<String> {
        tower::spell::held(&self.world)
    }

    /// Which domain `name` was written for, if the tower has it.
    ///
    /// The fact that survives the file being the player's. A spell's home is
    /// what its lines are read against at cast, and it is a component rather
    /// than anything in the text — so only this can say whether a save from the
    /// archive re-homed a laboratory spell.
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
    /// The editor's half of *"report it, do not rewrite it"*: nothing rewrites a
    /// spell, so a player learns what the orb heard only by being told, and
    /// being told at cast is true but late.
    ///
    /// The sim's decision rather than the frontend's (rule 2): what a line means
    /// is the spell language, and a frontend working it out would be a second
    /// parser to keep in step with this one.
    ///
    /// Pure, and over the *buffer* rather than the saved file, so the answer
    /// tracks what is on screen.
    #[must_use]
    pub fn read_spell(&self, domain: &str, lines: &[String]) -> Vec<crate::tower::spell::Reading> {
        crate::tower::spell::interpret(&self.world, domain, lines, lines)
    }

    /// The same, with a reader for the lines the orb cannot read itself.
    ///
    /// [`read_spell`](Self::read_spell) is this with none, so a build without a
    /// reader shows exactly what it always did.
    ///
    /// The save's reading, not a second one: each line's reading comes from the
    /// lookup [`write_spell_reading`](Self::write_spell_reading) makes — the
    /// queued write, then the node's `Read` — so the editor and the runner
    /// cannot differ about what a line means. A line edited since is read now,
    /// exactly as the next save would read it.
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
    /// What the editor draws its marker from. Editing a spell while it runs is
    /// the loop this surface exists for, and a buffer that does not say where
    /// the orb has reached is one you edit blind.
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
    /// Takes rather than reads: `scribe` asks once, and a frontend polling a
    /// persistent flag would reopen the editor every frame. See
    /// [`Opening`](crate::execute::Opening).
    pub fn opening(&mut self) -> Option<crate::execute::Request> {
        self.world.resource_mut::<crate::execute::Opening>().take()
    }

    /// Whether a `scribe` is waiting, without taking it.
    ///
    /// So a frontend can ask before it mutates. `opening` takes `&mut self`, so
    /// a system holding `ResMut<Tower>` stamps the change tick merely by
    /// *asking* — leaving `resource_changed::<Tower>` true for ever and
    /// returning every system gated on it to 60 Hz.
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
    /// Takes rather than reads, for the same reason [`Sim::opening`] does: a
    /// frontend polling a persistent flag would re-enter reading mode every
    /// frame, including the frame after the player pressed Escape to leave it.
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
    /// Takes rather than reads, like the handshakes above it. What leaving
    /// *means* is a frontend's — dropping a window, or putting the terminal
    /// back — so the sim says only that it was asked for.
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
    /// A third entry point, because the tick was the wrong clock: an arrow key
    /// through `submit` landed a step on the next tick, and a maze walked at
    /// 1 Hz is a wait rather than a minigame.
    ///
    /// The narrowest door that answers it: the reading moves and no tick is
    /// consumed, so walking a maze by hand costs world time only in the sense
    /// that the player is standing there doing it.
    ///
    /// Replay is not weakened. A typed line is recorded against the tick it was
    /// *queued* on and runs at the start of the next; this runs immediately, so
    /// it lands after that tick's step. [`Submission::Walked`] tells them apart:
    /// replay a tick, then apply the walks recorded against it in list order. A
    /// tick never holds both kinds, because the prompt is dead while the arrows
    /// have the maze.
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
    /// The replay driver, owned here rather than written out by each caller: it
    /// was three hand-written `match`es, three chances to disagree about what a
    /// variant means — and a variant means *when it ran*:
    ///
    /// - [`Typed`](Submission::Typed) was queued during its tick and executes at
    ///   the start of the next.
    /// - [`Wrote`](Submission::Wrote) is the same: a save queues like any effect.
    /// - [`Walked`](Submission::Walked) already ran, during its tick, after that
    ///   tick's step — because [`Sim::walk`] does not wait for a clock.
    ///
    /// Standing on the recorded tick is correct for all three, and a fourth kind
    /// is one change the compiler insists on rather than three it does not.
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
                if let Some(way) = tower::Way::ALL.into_iter().find(|way| way.word() == word) {
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
    /// The room, not the leaf: a player standing in the alembic is in the
    /// laboratory, and anything asking *which line, which panel, which log*
    /// wants the room. `location` is the path and `domain_of` the walk.
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

    /// Queue a tester's `debug_spawn`, on the next tick like everything else.
    ///
    /// Recorded in the scrollback and in `Submissions`, so a debug session
    /// replays — but not in the parse trace, for the reason
    /// [`choose`](Self::choose) gives below: it is not a phrasing, and counting
    /// it would dilute §15's first metric with inputs that never tested the
    /// parser.
    #[cfg(debug_assertions)]
    fn debug_spawn(&mut self, line: &str, order: crate::execute::SpawnOrder) {
        let tick = *self.world.resource::<Tick>();
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Input)
            .text(orbs_render::FieldName::Message, line)
            .finish();
        self.world.resource_mut::<Submissions>().push(tick, line);
        self.world.resource_mut::<Pending>().spawn(order);
    }

    /// Substitute a reagent on a shelf where the player is standing.
    ///
    /// Takes the first pile it finds by name, so a dump reaches the same one
    /// every time — `tower::node` records archetype order as a defect that
    /// changes what a phrase resolves to with nothing catching it.
    #[cfg(debug_assertions)]
    fn debug_swap(&mut self, line: &str) {
        let tick = *self.world.resource::<Tick>();
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Input)
            .text(orbs_render::FieldName::Message, line)
            .finish();
        self.world.resource_mut::<Submissions>().push(tick, line);

        let cwd = self.world.resource::<tower::Cwd>().0;
        let mut piles: Vec<(String, bevy_ecs::entity::Entity)> =
            tower::children_of(&self.world, cwd)
                .into_iter()
                .flat_map(|place| tower::children_of(&self.world, place))
                .filter(|node| self.world.get::<tower::Stock>(*node).is_some())
                .filter_map(|node| {
                    self.world
                        .get::<tower::Name>(node)
                        .map(|name| (name.0.clone(), node))
                })
                .collect();
        piles.sort_unstable();

        if let Some((name, node)) = piles.first() {
            let claimed = format!("{name}-");
            tower::substitute(&mut self.world, *node, &claimed);
        }
    }

    /// Echo a debug word, record it for replay, and find what it acts on.
    ///
    /// One prologue, because `debug_ward` and `debug_course` wrote out the same
    /// four steps byte for byte, differing only in the type — so a protocol
    /// change had to be made twice and a fix to one was invisible in the other.
    ///
    /// The `Submission` is the load-bearing step: a debug word reaches the world
    /// without going through `submit`'s parser, so a replay that did not see it
    /// would diverge from the session that recorded it.
    ///
    /// Returns the node carrying `C`, or `None` — what each word does with it is
    /// the word's own business.
    #[cfg(debug_assertions)]
    fn debug_shortcut<C: bevy_ecs::component::Component>(
        &mut self,
        line: &str,
    ) -> Option<bevy_ecs::entity::Entity> {
        let tick = *self.world.resource::<Tick>();
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Input)
            .text(orbs_render::FieldName::Message, line)
            .finish();
        self.world.resource_mut::<Submissions>().push(tick, line);
        self.here_with::<C>()
    }

    /// Hold a mastery node without earning the experience for it.
    ///
    /// Straight into `Taken`, skipping `weave::grant`'s `Standing::Open` check,
    /// since that check is the threshold. Everything downstream is the real
    /// thing: `spell::budget` sums it, `is_gated` reads it, and
    /// `compile::check_learned` sees exactly what a played tower would.
    ///
    /// A marker is refused: a tower holding one is a state the game cannot
    /// reach, so not one worth testing from.
    #[cfg(debug_assertions)]
    fn debug_take(&mut self, line: &str, id: Option<&str>) {
        let tick = *self.world.resource::<Tick>();
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Input)
            .text(orbs_render::FieldName::Message, line)
            .finish();
        self.world.resource_mut::<Submissions>().push(tick, line);

        let real: Vec<String> = self
            .world
            .resource::<crate::content::Progression>()
            .ley_line()
            .iter()
            .flat_map(|station| &station.nodes)
            .cloned()
            .collect();
        let said = match id {
            Some(id) if real.iter().any(|node| node == id) => {
                self.world.resource_mut::<tower::Taken>().hold(id);
                format!("the orb holds {id}")
            }
            Some(id) => format!("{id} grants nothing. one of: {}", real.join(", ")),
            None => format!("nodes: {}", real.join(", ")),
        };
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(orbs_render::FieldName::Message, &said)
            .finish();
    }

    /// Put renown at a total, for a tester, and say the title it crossed.
    ///
    /// Sets rather than adds, so one word goes both ways — a rank is lost by
    /// falling back through it, and reaching that otherwise means losing a siege
    /// on purpose. Bare, it says where the tower stands.
    #[cfg(debug_assertions)]
    fn debug_renown(&mut self, line: &str, total: crate::execute::Asking) {
        let tick = *self.world.resource::<Tick>();
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Input)
            .text(orbs_render::FieldName::Message, line)
            .finish();
        self.world.resource_mut::<Submissions>().push(tick, line);

        let standing = |world: &World| {
            let held = world.resource::<tower::Renown>().get();
            let title = tower::renown::rank(world);
            format!(
                "renown {held}, and the tower is {}",
                title.unwrap_or_else(|| "nothing yet".to_owned()),
            )
        };
        let said = match total {
            crate::execute::Asking::Set(total) => {
                tower::renown::set(&mut self.world, total);
                format!("renown is {total}")
            }
            crate::execute::Asking::Where => standing(&self.world),
            // Refused, and nothing touched, as `debug_take` and `debug_reach`
            // answer a bad argument. Reading it as nought zeroed the total and
            // said so — destroying what the tester had built, and looking like
            // it had worked.
            crate::execute::Asking::Unreadable => {
                format!("that is no number. {}", standing(&self.world))
            }
        };
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(orbs_render::FieldName::Message, &said)
            .finish();
    }

    /// Reach a mastery station without doing its deed.
    ///
    /// Every earlier station on the line too, because a line is walked in order
    /// and a tower with its third station reached and its first not is a state
    /// the game cannot reach. What each opens is opened and said, so a See-it
    /// line about a gated recipe sees what a played tower would.
    ///
    /// And the room the line stands in: `debug_reach archive_3` reached three
    /// stations inside a room the player could not enter — the archive is opened
    /// by `laboratory_1`, on a *different* line — so `survey cabinet` answered
    /// *"the archive is not yours yet"* while its own line read three of five.
    #[cfg(debug_assertions)]
    fn debug_reach(&mut self, line: &str, id: Option<&str>) {
        let tick = *self.world.resource::<Tick>();
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Input)
            .text(orbs_render::FieldName::Message, line)
            .finish();
        self.world.resource_mut::<Submissions>().push(tick, line);

        let stations: Vec<(String, String, Vec<String>)> = self
            .world
            .resource::<crate::content::Progression>()
            .mastery()
            .iter()
            .map(|milestone| {
                (
                    milestone.domain.clone(),
                    milestone.id.clone(),
                    milestone.opens.clone(),
                )
            })
            .collect();
        let ids: Vec<&str> = stations.iter().map(|(_, id, _)| id.as_str()).collect();
        let said = match id {
            Some(wanted) if ids.contains(&wanted) => {
                let domain = stations
                    .iter()
                    .find(|(_, id, _)| id == wanted)
                    .map(|(domain, _, _)| domain.clone())
                    .unwrap_or_default();
                self.debug_unseal(&domain, &stations);
                let mut reached_any = false;
                for (line_domain, station, opens) in &stations {
                    if *line_domain != domain {
                        continue;
                    }
                    if !self
                        .world
                        .resource::<tower::mastery::Reached>()
                        .has(station)
                    {
                        tower::mastery::reach(&mut self.world, line_domain, station, opens);
                        reached_any = true;
                    }
                    if station == wanted {
                        break;
                    }
                }
                if reached_any {
                    format!("the orb reaches {wanted}")
                } else {
                    format!("{wanted} is reached already")
                }
            }
            Some(wanted) => format!("{wanted} is no station. one of: {}", ids.join(", ")),
            None => format!("stations: {}", ids.join(", ")),
        };
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(orbs_render::FieldName::Message, &said)
            .finish();
    }

    /// Open the room `domain`, and whatever had to open first for it to.
    ///
    /// The chain, outermost first: a room is opened by a station on *another*
    /// room's line, which may itself stand in a shut room — the sanctum is the
    /// laboratory's third. Walked here rather than by hand in a See-it line.
    ///
    /// Where the tower's own line opens a room — the grimoire at 16, the forge
    /// at 56 — the key is applied directly rather than credited, because this
    /// word grants no experience and handing out fifty-six would move
    /// concentration, the pool and every reading derived from the total.
    #[cfg(debug_assertions)]
    fn debug_unseal(&mut self, domain: &str, stations: &[(String, String, Vec<String>)]) {
        let mut order: Vec<(String, String)> = Vec::new();
        let mut wanted = domain.to_owned();
        // Bounded by the rooms there are, so no authored chain can spin here.
        for _ in 0..=tower::DOMAINS.len() {
            if self.world.resource::<tower::Opened>().is_open(&wanted) {
                break;
            }
            let key = tower::domain_key(&wanted);
            let Some((line, id, _)) = stations.iter().find(|(_, _, opens)| opens.contains(&key))
            else {
                tower::open(&mut self.world, &key);
                break;
            };
            order.push((line.clone(), id.clone()));
            wanted = line.clone();
        }
        for (line, wanted) in order.into_iter().rev() {
            for (line_domain, station, opens) in stations {
                if *line_domain != line {
                    continue;
                }
                if !self
                    .world
                    .resource::<tower::mastery::Reached>()
                    .has(station)
                {
                    tower::mastery::reach(&mut self.world, line_domain, station, opens);
                }
                if *station == wanted {
                    break;
                }
            }
        }
    }

    /// Hand the open ward's answer to the aperture, so the next press breaks it.
    #[cfg(debug_assertions)]
    fn debug_ward(&mut self, line: &str) {
        if let Some(node) = self.debug_shortcut::<tower::Ward>(line)
            && let Some(mut ward) = self.world.get_mut::<tower::Ward>(node)
        {
            ward.give_away();
        }
    }

    /// Stack the standing course but its smallest ward, one haul from finished.
    #[cfg(debug_assertions)]
    fn debug_course(&mut self, line: &str) {
        if let Some(node) = self.debug_shortcut::<tower::Course>(line)
            && let Some(mut course) = self.world.get_mut::<tower::Course>(node)
        {
            course.give_away();
            // Republished, unlike `debug_ward`: a ward's readings are rewritten
            // by the press that follows, a course's by the *haul* — and a
            // solver reads the two `potency` readings this just moved to decide
            // which haul that is.
            crate::execute::refresh_pylon(&mut self.world);
            return;
        }

        // A shortcut that finds nothing says so. Silent, typing it before
        // `muster` — or outside the sanctum, since the pylon is found through
        // `Cwd` — left the *next* line to report the trouble, and `haul`
        // answering "there is nothing drawn to move" reads as the haul being
        // wrong rather than the shortcut.
        let message = self
            .world
            .resource::<crate::content::Prose>()
            .line("muster_nothing_to_give", &[]);
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(orbs_render::FieldName::Name, crate::execute::COURSE)
            .text(orbs_render::FieldName::Message, &message)
            .role(orbs_render::Role::Cost)
            .finish();
    }

    /// Limn the circle to one of the waiting beast's solutions, so the next
    /// `summon` holds it.
    #[cfg(debug_assertions)]
    fn debug_circle(&mut self, line: &str) {
        if let Some(node) = self.debug_shortcut::<tower::circle::Beast>(line)
            && let Some(mut beast) = self.world.get_mut::<tower::circle::Beast>(node)
            && let Some(solution) = beast.solution()
        {
            for (glyph, humour) in tower::circle::Glyph::ALL.into_iter().zip(solution) {
                beast.limn(glyph, humour);
            }
            // Republished, for `debug_course`'s reason: a spell reads the
            // glyphs' readings next, and stale they still name the opening.
            crate::execute::refresh_circle(&mut self.world);
            return;
        }

        // A shortcut that finds nothing says so (`debug_course`'s rule): silent,
        // it would leave the next `summon` to *draw* a beast instead.
        let message = self
            .world
            .resource::<crate::content::Prose>()
            .line("summon_nothing_to_give", &[]);
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(orbs_render::FieldName::Name, crate::execute::CIRCLE)
            .text(orbs_render::FieldName::Message, &message)
            .role(orbs_render::Role::Cost)
            .finish();
    }

    /// Thin the enemy to one, so the next `hold` is the round that ends it.
    #[cfg(debug_assertions)]
    fn debug_siege(&mut self, line: &str) {
        if let Some(node) = self.debug_shortcut::<tower::Siege>(line)
            && let Some(mut siege) = self.world.get_mut::<tower::Siege>(node)
            && siege.running()
        {
            siege.give_away();
            // Republished, for `debug_course`'s reason: a siege's readings are
            // rewritten by the *round* that follows, so `foes`, `outnumbered`
            // and `massed` would describe the enemy that arrived rather than
            // the one left — and a decision tree reads exactly those.
            crate::execute::refresh_rampart(&mut self.world);
            return;
        }

        // A shortcut that finds nothing says so (`debug_course`'s rule): silent,
        // `hold` answering "nothing is at the wall" reads as the hold being
        // wrong rather than the shortcut.
        let message = self
            .world
            .resource::<crate::content::Prose>()
            .line("defend_nothing_to_give", &[]);
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(orbs_render::FieldName::Name, crate::execute::SIEGE)
            .text(orbs_render::FieldName::Message, &message)
            .role(orbs_render::Role::Cost)
            .finish();
    }

    /// Learn a secret the lens would otherwise have to find.
    ///
    /// Runs now rather than queueing, like `debug_spell`: nothing about it is a
    /// world action with a duration, and a tester wants the next line of their
    /// dump to see the result.
    #[cfg(debug_assertions)]
    fn debug_learn(&mut self, line: &str, wanted: Option<&str>) {
        let tick = *self.world.resource::<Tick>();
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Input)
            .text(orbs_render::FieldName::Message, line)
            .finish();
        self.world.resource_mut::<Submissions>().push(tick, line);

        let found = crate::tower::learn(&mut self.world, wanted);
        let message = match &found {
            Some(name) => self
                .world
                .resource::<Prose>()
                .line("probe_found", &[("name", name)]),
            None => format!("{line}: no such secret, or every one is known"),
        };
        self.world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(orbs_render::FieldName::Name, crate::execute::LEARN)
            .text(orbs_render::FieldName::Message, &message)
            .role(if found.is_some() {
                orbs_render::Role::Success
            } else {
                orbs_render::Role::Cost
            })
            .finish();
    }

    /// Queue a tester's `debug_spell`.
    ///
    /// Runs now, and records the write rather than the line — both departures
    /// from [`debug_spawn`](Self::debug_spawn) for one reason: the effect *is* a
    /// write, and [`write_spell`](Self::write_spell) already pushes a `Wrote`,
    /// so recording the line too would push two submissions for one input.
    ///
    /// It is also the stronger guarantee — a replay reproduces the lines that
    /// ran, even if `dev_spells.toml` is edited afterwards, where a recorded
    /// name would silently pick up the new text.
    ///
    /// Not in the parse trace, for the reason `debug_spawn` gives.
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
    /// Recorded in the scrollback but not in the parse trace: a digit is not a
    /// phrasing, and §15's first metric wants the *original* ambiguous line
    /// reaching its intended action, which the selection is the evidence for.
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

    // There is deliberately no `scrollback_mut`. §3 makes this stream *the* log,
    // so a frontend writing a line into it directly would produce a session
    // that replaying `(seed, submissions)` cannot reproduce (§13). Commands
    // write through `run_pending` on a tick boundary or they do not write; a
    // test that must reach past that goes through `Sim::world_mut`.

    /// What is in flight, if anything (§5.0).
    ///
    /// One at a time: §11.5 opens at multiplex capacity 1 and §9's fourth
    /// invariant reserves that slot for the action's whole duration.
    ///
    /// `iter_entities` rather than a query because a query needs `&mut World`
    /// for its `QueryState` and a paint calls this with the sim shared. The
    /// filter is by component, so it is correct; the *breadth* is unfortunate,
    /// and Bevy 0.19 widened it — resources are components on dedicated
    /// entities, so this walks those too. Nothing here can carry `Working`, but
    /// a broad walk filtering on something a resource *could* have would
    /// silently include them.
    #[must_use]
    pub fn working(&self) -> Option<tower::Working> {
        self.world
            .iter_entities()
            .find_map(|entity| entity.get::<tower::Working>().copied())
    }

    /// The instruments where the player is standing, for §10.1's panel.
    ///
    /// Empty anywhere but the laboratory, which is what makes the panel a
    /// property of *where you are* rather than something a frontend decides to
    /// show.
    #[must_use]
    pub fn instruments(&self) -> Vec<tower::Instrument> {
        tower::instruments(&self.world)
    }

    /// The stacks the player is standing over, if they are open (§10, §19).
    ///
    /// `None` everywhere but an archive with a maze open, for the reason
    /// [`Sim::instruments`] is empty outside the laboratory: the stacks are
    /// found relative to `Cwd`, so a frontend cannot carry the map out of the
    /// room and show something `survey` would not.
    ///
    /// Built fresh rather than cached, and called once a tick — a 49-cell `Vec`
    /// at 1 Hz, where `Instrument`'s objection was to allocating per *frame*.
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
    /// A frontend owns the file watcher (rule 3: the sim is called, never
    /// hosted; rule 8: no async here) and calls this between steps, so a reload
    /// lands on a tick boundary and never mid-schedule.
    ///
    /// Replay is safe: no line reaches a decision, so `(seed, submissions)`
    /// still replays to the same world. Recipes will not have this property, and
    /// when they arrive the content has to be versioned into the submission log.
    ///
    /// It was false while `recall_` keys fed the scene, every one a
    /// `NounKind::Topic`. [`Topics`](crate::tower::Topics) is snapshotted at
    /// construction and not touched here, so a new manual subject needs a
    /// relaunch and its text does not.
    pub fn set_prose(&mut self, prose: Prose) {
        self.world.insert_resource(prose);
    }

    /// The orb's voice, for the handful of lines a frontend must speak.
    ///
    /// Almost nothing needs this: prose belongs on a record, and a frontend
    /// drawing one gets the sentence with it. The exception is a screen the sim
    /// has no record for — §9's "window too small" — which is still rule 6's
    /// prose rather than a literal in the Bevy crate.
    #[must_use]
    pub fn prose(&self) -> &Prose {
        self.world.resource::<Prose>()
    }

    /// Replace the manual, for `ORBS_CONTENT`.
    ///
    /// The pair of [`set_prose`](Self::set_prose) and for the same reason: rule
    /// 6 is not satisfied by `include_str!` alone, and ten thousand words that
    /// needed a recompile per edit is what rule 6 exists to prevent.
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
    /// DESIGN.md §3's high-threat tonal register, driven by threat in Phase 8
    /// and reachable directly until then, so the three typefaces and §3's
    /// corruption exemption are something a person can see.
    ///
    /// Nothing about the *content* changes — §3: *"the renderer corrupts it; the
    /// model records it faithfully."* Only the face a frontend draws with, and
    /// log lines refuse the eldritch one however this is set.
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
    /// What the prompt shows, and therefore what tells a player which commands
    /// will resolve: the essences are in `/tower/laboratory`, so that is where
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
