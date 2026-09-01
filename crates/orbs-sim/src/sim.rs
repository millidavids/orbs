//! The simulation, and the single entry point frontends drive it through.
//!
//! Frontends are *callers*, not hosts: they construct a [`Sim`] and call
//! [`Sim::step`]. No frontend's own scheduler ever drives the world. This is what
//! keeps the Bevy build, the terminal build, and the balance harness running the
//! identical code path — and therefore producing identical results.

use bevy_ecs::prelude::*;
use orbs_render::{Outcome, Presentation, RecordKind};

use crate::content::{Fuels, Prose, Recipes, Spells};
use crate::execute::run_pending;
use crate::parser::{Mode, ParseLog, ParseRecord, Resolution, Scene, analyse, report};
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
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self::with_schedule(seed, |_| {})
    }

    /// Create a simulation and populate its schedule.
    ///
    /// The schedule is built here and never exposed afterwards, so systems cannot
    /// be added behind the sim's back at runtime.
    ///
    /// # A frontend must not call this
    ///
    /// It exists for tests that need to observe the schedule's ordering
    /// guarantees from outside — see `tower::scene`, where a caller's system had
    /// been running *before* the scene rebuild and the topsort was within its
    /// rights to do it.
    ///
    /// Domain systems belong inside [`Sim::new`]. §13 is explicit about why: if
    /// the Bevy build, `orbs-tui` and `orbs-balance` each registered their own,
    /// they would be three different games, and *"if the live game and the CLI
    /// harness diverged, we would not find out until Phase 9."* The seam is left
    /// open because closing it would cost the ordering test its only handle, not
    /// because a frontend may reach through it.
    ///
    /// # Panics
    ///
    /// If the built-in content files disagree with each other — today, if
    /// `progression.toml` does not price every instrument `recipes.toml` names.
    /// Both ship inside the binary, so this is a build-time authoring error that
    /// no input can reach, and it fails the same way `load::builtin` fails a
    /// malformed file rather than starting a tower whose work is worth nothing.
    #[must_use]
    pub fn with_schedule(seed: u64, build: impl FnOnce(&mut Schedule)) -> Self {
        let mut world = Self::bare(seed);
        let (commands, schedule, scene) = Self::schedules(build);

        // The tower is raised before the first tick, so tick 0 already has a
        // world to name.
        tower::raise(&mut world);
        // **The walls have to say how they stand from tick 0.** `integrity` is a
        // reading on the pylon, published by `erode` when the number moves — and
        // the number does not move for thirty ticks, so a tower nobody had
        // touched had no reading at all. `survey pylon` printed nothing, and
        // worse: `many_at` answers an absent child with **nought**, so a spell
        // asking `if the pylon has fewer than 60 integrity` fired on a whole
        // barrier for the first half-minute of every session.
        //
        // Raising it here rather than in `tower::build` because `build` names
        // things and knows no resources, and this is a number.
        if let Some(pylon) = tower::pylon::fixture(&world) {
            crate::execute::publish_pylon(&mut world, pylon);
        }
        // **And what each die costs, for exactly the same reason one line up.**
        // A die's price is a fact about the die rather than about a fight, and it
        // was raised only by `defend::publish` — which nothing calls until the
        // first bailey verb. So `survey d20` on a fresh tower answered *"the d20
        // holds nothing"*, and because `many_at` reads an absent child as nought
        // the affordability guard every solver ships — `not the coffer has fewer
        // quintessence than the d20` — compared nought against nought and
        // answered **yes, afford it** on a tower with no pool at all.
        //
        // The sanctum paid for this once already; the comment above is its scar.
        crate::execute::publish_dice(&mut world);
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
    /// # Why this is a function rather than the top of [`with_schedule`]
    ///
    /// Because there are two ways to reach a world now — raising a new tower and
    /// loading a saved one — and §13's whole argument is that two constructions
    /// of one world are two different games. The list below is thirty-odd lines
    /// of `init_resource` and is exactly where an omission would hide: a save
    /// that built its own world and forgot one entry would start a tower missing
    /// something no test asks about. So both callers get this list, and there is
    /// only one of it.
    ///
    /// It is deliberately *not* a `Default`: it takes the seed, and a world with
    /// an unseeded RNG is not a lesser world but a broken one.
    ///
    /// # Panics
    ///
    /// As [`with_schedule`](Self::with_schedule) documents.
    fn bare(seed: u64) -> World {
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
        world.init_resource::<crate::execute::Answered>();
        world.init_resource::<crate::execute::Chorusing>();
        world.init_resource::<crate::execute::Patient>();
        world.init_resource::<ParseLog>();
        world.init_resource::<Wizard>();
        world.init_resource::<Choices>();
        // What each domain wants noticed, for §9's rail. Empty at tick 0 and
        // latched by whatever raises one; `attend` is the only thing that
        // clears, so a mark survives a save exactly as the state it describes
        // does.
        world.init_resource::<crate::tower::Marks>();
        // What the player has found. Empty at tick 0 and written only by a
        // broken ward, so it is reproducible from `(seed, submissions)` exactly
        // as everything else in the world is.
        world.init_resource::<crate::tower::Learned>();
        // The compiled-in default, so a headless `Sim` needs no filesystem
        // (rule 6, rule 8). A frontend swaps it with `set_prose`.
        world.init_resource::<Prose>();
        // The manual's subjects, fixed here. They are parser nouns, so reading
        // them live from `Prose` would let a hot reload change what a phrase
        // resolves to — see `tower::Topics`.
        let topics = crate::tower::Topics::of(world.resource::<Prose>());
        world.insert_resource(topics);
        // Recipes are **not** hot-reloadable, unlike prose: they reach
        // decisions, so swapping them mid-session would break replay from
        // `(seed, submissions)` unless the content were versioned with it.
        world.init_resource::<Recipes>();
        world.init_resource::<Fuels>();
        // What the arsenal is worth in a siege. **Recipes' tier, not
        // materials'**: these reach decisions — how many troops a scroll is
        // worth changes what a round does — so swapping them mid-session would
        // break replay from `(seed, submissions)` the same way a recipe swap
        // would.
        world.init_resource::<crate::content::Spendables>();
        // Materials **are** hot-reloadable in principle, unlike the two above:
        // a tint is read by the instrument panel and by nothing else, so no verb
        // branches on it and swapping it mid-session cannot change what the
        // world does. Installed here beside them because that is where content
        // lives, not because it shares their constraint.
        world.init_resource::<crate::content::Materials>();
        // Spells sit in the same tier as recipes and for the same reason: a
        // spell is nothing *but* decisions, so a reload would break replay from
        // `(seed, submissions)`. Read once, here, and never again — the player's
        // own edits go through the world, not through this.
        world.init_resource::<Spells>();
        // The progression curve is in the recipes' tier, not the materials':
        // a weight decides what a run earns and a threshold gates a verb, so
        // both reach decisions and a mid-session swap would break replay.
        //
        // **Checked against the recipes, which is why it loads after them.** Its
        // keys are instrument names and nothing in Rust knows what those are —
        // `materials.toml` validates against an enum and had no such ordering.
        // The panic is the same one `load::builtin` uses for a malformed file,
        // and for the same reason: authoring the two files to disagree is a
        // build-time error that `the_builtin_curve_prices_every_instrument`
        // fails on first.
        let curve = crate::content::Progression::default();
        // **What may be priced: anything that runs.** The recipes' instruments,
        // plus the fixtures that carry a verb and transform nothing — the
        // athanor, and the `stacks`, which earns for every walk finished and
        // has no recipe to be found by. Checking against the recipes alone made
        // the second impossible to price and the first impossible to price
        // *ever*.
        let mut instruments = world.resource::<Recipes>().instruments();
        instruments.extend(tower::operated());
        instruments.sort_unstable();
        instruments.dedup();
        if let Err(error) = curve.check(&instruments) {
            panic!("the built-in content is authored with the crate: {error}");
        }
        world.insert_resource(curve);
        world.init_resource::<tower::Experience>();
        // Whole, by `Default`. A tower is not built already crumbling.
        world.init_resource::<tower::Integrity>();
        world.init_resource::<tower::Taken>();
        world.init_resource::<crate::execute::Opening>();
        world.init_resource::<crate::execute::Reloaded>();
        world.init_resource::<crate::execute::Unfurling>();
        world.init_resource::<crate::execute::Quitting>();
        world.init_resource::<crate::execute::Weaving>();
        world.init_resource::<crate::execute::Wandering>();
        world.init_resource::<tower::spell::Caller>();

        world
    }

    /// The three passes a tick runs, in the order they run.
    ///
    /// Split out beside [`bare`](Self::bare) and for the same reason: the system
    /// order below is load-bearing — two of these draw from one RNG stream and
    /// the comments say what reordering them would cost — so there may be
    /// exactly one place it is written down. A loaded world runs the same three
    /// passes as a raised one or it is a different game.
    fn schedules(build: impl FnOnce(&mut Schedule)) -> (Schedule, Schedule, Schedule) {
        // Its **own** schedule, run before the caller's. Adding `run_pending`
        // to the same schedule and relying on insertion order would be an
        // ambiguity, not an ordering: Bevy makes no promise about systems with
        // no constraint between them, and a domain system added through `build`
        // could observe the queue either drained or not. A separate pass is
        // unambiguous by construction and needs no set for callers to remember.
        let mut commands = new_sim_schedule();
        commands.add_systems(run_pending);

        let mut schedule = new_sim_schedule();
        // `burn` before `finish`: a fire that runs out on the same tick a heated
        // stage lands should be cold *after* that stage completes, not before —
        // the run was already committed when it started (§10.1), and ordering it
        // the other way would make a completion depend on which system Bevy
        // happened to sort first.
        // `spell::advance` **before** `finish`: a spell must see the world as
        // the previous tick left it rather than racing the completion of the run
        // it is waiting on. Running it after would let a script start the next
        // stage on the same tick the previous one landed, which is a free tick
        // no manual player gets — §8's speed advantage arriving by accident, and
        // arriving at concentration 0 where §19 says nothing may.
        // `spell::stand` **last**, so a bound spell that ran off the end this
        // tick is cast again on the next one rather than inside the same pass.
        // Standing it up before `advance` would give a held spell two goes at
        // the budget in one tick — §8's speed advantage arriving by the back
        // door, at concentration 1 where §19 says nothing may.
        schedule.add_systems(
            (
                tower::spell::advance,
                tower::burn,
                tower::finish,
                tower::drift,
                // **After `drift`, and the order is load-bearing.** Both draw
                // once per tick from `RngStream::Threat`, so which one goes
                // first decides which value each sees — and a schedule that
                // reordered them would silently change every existing replay.
                // Appended, never inserted, exactly as a stream index is.
                tower::substitution,
                // **After the roll, and it draws nothing.** A lie settling is a
                // clock reading, so it cannot perturb `RngStream::Threat` — which
                // is what lets it be appended here without touching a replay.
                tower::settling,
                // **The same licence, for the same reason.** A barrier wearing
                // down is a comparison of two ticks; the sanctum's one draw is
                // in `height_for`, on `RngStream::Battlements`, and happens
                // inside `muster` rather than in a system. So this is appended
                // here without shifting a single existing replay.
                tower::erode,
                // **The same licence again, and this one is the clearest case
                // of it.** A syllable landing unanswered is a clock reading and
                // nothing else: the menagerie's one draw is `Chant::draw`, on
                // `RngStream::Menagerie`, and it happens inside `summon` rather
                // than in a system. So this is appended here without shifting a
                // single existing replay.
                //
                // **After `erode`**, so a chant that collapses on the same tick
                // the barrier wears sees the barrier the tick left it, not the
                // one it started with.
                crate::execute::lapse_chant,
                tower::spell::stand,
            )
                .chain(),
        );
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

        (commands, schedule, scene)
    }

    /// Read this world out as a save document.
    ///
    /// **Takes `&self`**, which is the guarantee rather than a courtesy: saving
    /// cannot perturb the world it is describing. A `&mut` here could build a
    /// `QueryState`, which registers components and moves archetypes, and a save
    /// that changed the thing it measured would make `tests/persistence.rs`
    /// measure itself.
    ///
    /// **Take it at a tick boundary**, which §8 makes the rule: *"instruction
    /// dispatch is atomic within a tick; saves are permitted only at tick
    /// boundaries."* Immediately after [`step`](Self::step) is that moment, and
    /// it is where both frontends take theirs.
    ///
    /// Writing the result to a file is a frontend's — see `orbs_shell::save`.
    /// This crate never touches the filesystem.
    #[must_use]
    pub fn snapshot(&self) -> crate::save::Save {
        crate::save::capture(&self.world)
    }

    /// Build a world from a save document.
    ///
    /// Raises the tower first and then applies the save over it, which is what
    /// lets a save written before a domain existed open into a tower that has
    /// one. `crate::save::restore` carries the reasoning.
    ///
    /// The result is a world standing exactly where the saved one stood: the
    /// same tick, the same eight random-stream positions, the same work in
    /// flight. Stepping it and stepping the world that wrote it produces the
    /// same two worlds, which is the property `tests/persistence.rs` holds.
    ///
    /// # Panics
    ///
    /// As [`with_schedule`](Self::with_schedule) does, and for the same reason:
    /// the built-in content is authored with the crate.
    #[must_use]
    pub fn restored(save: &crate::save::Save) -> Self {
        let mut world = Self::bare(save.world.seed);
        let (commands, schedule, scene) = Self::schedules(|_| {});

        // Raised before the save is applied, never instead of it.
        tower::raise(&mut world);
        crate::save::restore(&mut world, save);
        tower::rebuild(&mut world);

        // **No `tower::report` here, unlike `with_schedule`**, and the round-trip
        // test is what settled it. `report` pushes §4's condition report onto the
        // record stream — which `restore` has just rebuilt from the save — so a
        // loaded world would carry thirty-odd records the world that wrote it
        // never had, and `snapshot(loaded) != snapshot(saved)` on the first tick.
        //
        // The contract is worth more than the banner: **a restored world is the
        // saved world, exactly.** Saying *"you are back, and the orb was dark for
        // three hours"* is a frontend's line anyway — the away stamp is the
        // frontend's, because the sim has no wall clock and must not acquire one.
        //
        // What a player actually sees on waking is better than a boot report: the
        // save carries the tail of the stream, so the transcript is the screen
        // they left.

        Self {
            world,
            commands,
            schedule,
            scene,
        }
    }

    /// Say that this tower was resumed, and how long it was dark.
    ///
    /// # Why the sim says it and the frontend measures it
    ///
    /// Rule 6 puts prose in content files, so the words are the sim's; §19
    /// forbids the sim reading a wall clock, so the *gap* is the frontend's.
    /// This is where the two meet — the caller hands over a number of seconds it
    /// read from the machine, and the orb finds the sentence.
    ///
    /// # Why it is not inside [`restored`](Self::restored)
    ///
    /// Because a restored world must be **the saved world, exactly**: a banner
    /// pushed there would put records in the loaded stream that the world which
    /// wrote it never had, and `tests/persistence.rs` compares the two documents
    /// byte for byte. Saying so is a thing a *session* does, not a thing a world
    /// is, which is the same line `quit` draws.
    ///
    /// `away` is seconds, or `None` where the machine would not say — a save
    /// written before the stamp existed, or a clock that has gone backwards.
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
    /// # Why this is a record and not a `status` line
    ///
    /// Because §3 makes the record stream *the* output: a status line is a view
    /// over records, and a thing that never becomes one is a thing `sift`, the
    /// log, and the screen reader all cannot see. A player who never types
    /// `status` would also never learn — and there is no `save` verb (§19), so
    /// they have no reason to look.
    ///
    /// The caller says this **once**. A save is attempted every sixty ticks and
    /// a read-only directory fails every one of them, so a line per attempt is
    /// sixty an hour — which is the noise §19 deleted the editor's per-save
    /// announcement over.
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

        // **A tester's door, and only in a build a tester runs.** Matched
        // exactly and checked before the parser, in the same shape spell words
        // use — `debug_spawn` is not in §6's vocabulary, so the fuzzy matcher
        // must never see it and `Verb::ALL` must never grow it. Absent from a
        // release binary entirely; there, this is an ordinary unresolvable line.
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
            | Resolution::Elsewhere { .. }
            | Resolution::InSpell { .. }
            | Resolution::Unresolved { .. } => {}
        }
    }

    /// Save a spell out of the editor.
    ///
    /// **The third entry point, and the last one.** [`submit`](Self::submit)
    /// takes a line the player typed; this takes a file the player wrote. Both
    /// are *decisions*, which is the test for what belongs in
    /// [`Submissions`](crate::session::Submissions) and therefore in a replay —
    /// and the keystrokes that built the buffer are not, which is why the editor
    /// itself lives in the frontend beside the prompt's own line editor.
    ///
    /// # What lands, and when
    ///
    /// Like `submit`, this does **not** advance world time. It records the
    /// submission immediately and queues the write for the next tick, because
    /// `session` is explicit that effects land on a tick boundary through
    /// `Pending` — a world mutated from inside an input call produces a session
    /// that `(seed, submissions)` cannot reproduce. `scene::rebuild` runs per
    /// tick anyway, so a new spell is nameable from the tick after it is saved
    /// either way.
    ///
    /// The **typed** lines are what gets recorded, not the canonical form they
    /// become. A replay re-derives the canonicalisation, so improving the
    /// canonicaliser cannot silently make an old session replay into a different
    /// world.
    pub fn write_spell(&mut self, name: &str, lines: &[String]) {
        let filename = crate::content::with_extension(name);
        let tick = *self.world.resource::<Tick>();
        self.world
            .resource_mut::<Submissions>()
            .wrote(tick, &filename, lines);
        self.world
            .resource_mut::<Pending>()
            .write(filename, lines.to_vec());
    }

    /// Take a mastery node, on the next tick.
    ///
    /// [`write_spell`](Self::write_spell)'s shape, one screen along: the choice
    /// is recorded as a [`Submission`](crate::session::Submission) and queued as
    /// an effect, so it lands on a tick boundary like everything else a player
    /// decides. **The id, not the keystrokes** — aiming the cursor changes no
    /// state the world can see.
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

    /// How the tower's defences stand, out of [`tower::STANDING`].
    ///
    /// Falls on its own (`tower::erode`) and is put back by finishing a course
    /// in the sanctum. **The only resource in the game that goes down.**
    #[must_use]
    pub fn integrity(&self) -> u32 {
        self.world.resource::<tower::Integrity>().get()
    }

    /// Whether the node `named` in `room` is carrying `reading`.
    ///
    /// **The question a spell's `if` asks, asked directly.** Every other route
    /// to a published reading goes through `survey`, which costs a tick — and
    /// the menagerie's aperture moves on every tick, so a test that surveyed
    /// four syllables in turn would be looking at four different moments and
    /// could miss the aperture entirely. It did: `next` shipped unpublished and
    /// four surveys in a row all said *"holds nothing"*, which reads exactly
    /// like the feature being absent and exactly like the feature working.
    ///
    /// Walked from the root rather than from `Cwd`, so it answers about a room
    /// nobody is standing in — which is the case a bound solver is always in.
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
    /// **Derived from [`experience`](Self::experience)**, so this is a reading
    /// rather than a second piece of state — see `tower::experience`.
    #[must_use]
    pub fn concentration(&self) -> usize {
        tower::concentration(&self.world)
    }

    /// The ward's sheet, where the player is standing.
    ///
    /// **Reads `Cwd`, exactly as [`stacks`](Self::stacks) does**: a frontend
    /// asking for the board gets one only where the player could `survey prism`
    /// themselves, so the picture cannot outrun the readings by following them
    /// out of the room.
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
    /// **`orbs-balance` is the caller**, and the reason it is not
    /// [`pylon`](Self::pylon) is that a *view* is what a painter needs and a
    /// *course* is what a solver needs. The harness rotates through
    /// `pylon::cycle` and asks `Course::between` which way each haul runs, which
    /// keeps one copy of the algorithm in the game rather than two — see
    /// `drive::haul_one`.
    #[must_use]
    pub fn course(&self) -> Option<&tower::Course> {
        self.world
            .get::<tower::Course>(self.here_with::<tower::Course>()?)
    }

    /// The child of where the player stands that carries `C`, if any.
    ///
    /// **One walk, because this was three.** `course`, `pylon` and
    /// `debug_course` each wrote out the same `children_of(cwd).find(...)`, in
    /// one file, differing only in the component — which is the shape
    /// [`tower::reach`] was extracted to stop for *names*. The same argument
    /// applies to components: a room-scoped lookup is a rule about where a
    /// player is standing, and three copies of it are three chances to disagree
    /// about that.
    ///
    /// Room-scoped on purpose. `tower::pylon::fixture` is the tower-wide
    /// question and is deliberately separate — a bound solver's readings must
    /// keep up while the player is in another room, and a picture must not.
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
        // **Through `course`, not a second walk of `Cwd`.** These were the same
        // three lines twice in one file — and `debug_course` made it three —
        // which is the shape `tower::reach` was extracted to stop one layer
        // down. A view is a course plus a sentence; only the sentence is here.
        let course = self.course()?;
        // **The board's own line, written here.** `orbs-render` holds no
        // authored English (rule 6), so the sentence under the floor rule is
        // composed from the same prose key the reader hears — one spelling, one
        // place, and hot-reloadable like every other line in the game.
        let tally = self.prose().line(
            "pylon_tally",
            &[
                ("quantity", &course.height().to_string()),
                ("name", &course.hauls().to_string()),
            ],
        );
        Some(course.view(standing, tally))
    }

    /// The siege being fought in the bailey, if the player is looking at it.
    ///
    /// Reads `Cwd` for the reason [`pylon`](Self::pylon) does: the picture
    /// cannot outrun the readings by following the player out of the room.
    ///
    /// **A finished siege still draws.** `settle` leaves the board up so the
    /// last thing that happened stays readable — a board that vanished on the
    /// winning round would take the postmortem with it.
    #[must_use]
    pub fn rampart(&self) -> Option<orbs_render::Rampart> {
        let rampart = self.here_with::<tower::Siege>()?;
        let siege = self.world.get::<tower::Siege>(rampart)?;
        // The board's own line, written here: `orbs-render` holds no authored
        // English (rule 6), so the sentence under the rule is composed from the
        // same prose key the reader hears.
        let tally = self.prose().line(
            "siege_tally",
            &[
                ("quantity", &siege.turns.to_string()),
                ("name", &siege.enemy.count.to_string()),
            ],
        );
        Some(siege.view(tally))
    }

    /// The figure being sung in the menagerie, if the player is looking at it.
    ///
    /// Reads `Cwd` for the reason [`pylon`](Self::pylon) does: the picture cannot
    /// outrun the readings by following the player out of the room.
    ///
    /// **The lane names and glyphs travel with it**, which is what stops the
    /// painter from having to know what a syllable is called — `orbs-render` may
    /// never depend on `orbs-sim`, and §19 records the lens's sheet shipping
    /// unlabelled because that was got the other way round.
    #[must_use]
    pub fn figure(&self) -> Option<orbs_render::Figure> {
        let circle = crate::execute::circle_at(&self.world)?;
        let chant = self.world.get::<tower::Chant>(circle)?;
        let lanes = tower::Syllable::ALL
            .into_iter()
            .map(|one| (one.glyph(), one.word()))
            .collect::<Vec<_>>();
        let lane_of = |wanted: tower::Syllable| {
            tower::Syllable::ALL
                .into_iter()
                .position(|one| one == wanted)
                .unwrap_or_default()
        };
        let coming = chant
            .chart()
            .iter()
            .skip(chant.at())
            .take(orbs_render::AHEAD)
            .map(|one| lane_of(*one))
            .collect();
        let missed = chant.tally().1;
        // **The board's own line, written here.** `orbs-render` holds no
        // authored English (rule 6), so the sentence under the rule is composed
        // from a prose key — one spelling, one place, hot-reloadable.
        let tally = self.prose().line(
            "chant_tally",
            &[
                ("quantity", &chant.remaining().to_string()),
                ("name", &missed.to_string()),
            ],
        );
        // Oldest first, so the pegs read left to right as the figure was sung —
        // and taken from the sequence rather than rebuilt from counts, which
        // would draw a run that never happened.
        let sung = chant.sung().to_vec();
        Some(orbs_render::Figure {
            coming,
            lanes,
            sung,
            // The same number the `until` reading publishes, so a hand player
            // and a solver are reading one fact about one moment.
            until: chant.until(),
            // **The same number the tally above is written from**, so the board
            // and the spoken line cannot disagree — which they did.
            remaining: chant.remaining(),
            tally,
        })
    }

    /// Every domain at a glance — what §9's rail draws.
    ///
    /// **Derived here rather than in a frontend**, because rule 2 lets a
    /// frontend decide only how a cell is drawn: `orbs-tui` must be able to draw
    /// the same rail without re-deriving *what is happening in the forge*, and
    /// two derivations are two answers that can disagree.
    ///
    /// Always seven, in a fixed order, including the rooms the tower has not
    /// built yet — see [`tower::briefs`].
    #[must_use]
    pub fn briefs(&self) -> Vec<tower::Brief> {
        tower::briefs(&self.world)
    }

    /// The spells the orb is holding, named, in the order the tower keeps them.
    ///
    /// §8 wants concentration *"surfaced in `status` and in the sidebar — never a
    /// quiet log line"*, and a count alone cannot answer the question a player at
    /// capacity is actually asking, which is **which one**.
    #[must_use]
    pub fn bound(&self) -> Vec<String> {
        tower::spell::held(&self.world)
    }

    /// Which domain `name` was written for, if the tower has it.
    ///
    /// **The fact that survives the file being the player's.** A spell's home is
    /// what its lines are read against at cast, and it is a component rather than
    /// anything in the text — so nothing about the text can tell you whether a
    /// save from the archive re-homed a laboratory spell. This can.
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
    /// # The editor's half of "report it, do not rewrite it"
    ///
    /// Nothing rewrites a spell any more, so the only way a player learns what
    /// the orb heard — or that it heard nothing — is to be **told**. Being told
    /// at cast is true and late; this is the same answer at the moment they can
    /// act on it.
    ///
    /// It is the sim's decision rather than the frontend's (rule 2): what a line
    /// means is the spell language, and a frontend working it out for itself
    /// would be a second parser to keep in step with this one.
    ///
    /// Pure, and over the **buffer** rather than the saved file, so the answer
    /// tracks what is on screen rather than what was last written.
    #[must_use]
    pub fn read_spell(&self, domain: &str, lines: &[String]) -> Vec<crate::tower::spell::Reading> {
        crate::tower::spell::interpret(&self.world, domain, lines)
    }

    /// Which line of `name` a running invocation is on, if one is running.
    ///
    /// **What the editor draws its marker from.** A spell being edited while it
    /// runs is the loop this whole surface exists for, and a buffer that does
    /// not say where the orb has reached is a buffer you are editing blind.
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
    /// **Takes** rather than reads: `scribe` asks once, and a frontend polling a
    /// persistent flag would reopen the editor every frame. See
    /// [`Opening`](crate::execute::Opening).
    pub fn opening(&mut self) -> Option<crate::execute::Request> {
        self.world.resource_mut::<crate::execute::Opening>().take()
    }

    /// Whether a `scribe` is waiting, without taking it.
    ///
    /// **So a frontend can ask before it mutates.** `opening` takes `&mut self`,
    /// so a system holding `ResMut<Tower>` stamps the resource's change tick
    /// merely by *asking* — which leaves `resource_changed::<Tower>` true for
    /// ever and quietly returns every system gated on it to 60 Hz, including the
    /// two whose doc comments exist to say they must not be.
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
    /// **Takes** rather than reads, for the same reason [`Sim::opening`] does: a
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
    /// **Takes** rather than reads, like the four handshakes above it. What
    /// leaving *means* is entirely a frontend's — dropping a window, or leaving
    /// raw mode and putting the terminal back — and the two have nothing in
    /// common, so the sim says only that it was asked for.
    pub fn quitting(&mut self) -> bool {
        self.world.resource_mut::<crate::execute::Quitting>().take()
    }

    /// Whether `weave` has asked for the progression screen.
    ///
    /// **Peeked** rather than taken, so a system can decide whether to run
    /// without stamping the resource's change tick — see [`Sim::has_opening`],
    /// which exists for a defect this pair prevents.
    #[must_use]
    pub fn has_weaving(&self) -> bool {
        self.world.resource::<crate::execute::Weaving>().pending()
    }

    /// Take `weave`'s pending request, if there is one.
    pub fn weaving(&mut self) -> bool {
        self.world.resource_mut::<crate::execute::Weaving>().take()
    }

    /// Walk the archive's stacks one cell, **now** (§10, §19).
    ///
    /// # The third entry point, and why the tick was the wrong clock
    ///
    /// [`Sim::submit`] queues and [`Sim::step`] advances, and for two phases
    /// those were the only two doors. An arrow key went through `submit`, which
    /// meant a step landed on the next tick — a second away — and a maze walked
    /// at 1 Hz is not a minigame, it is a wait. Queueing the presses gave the
    /// player their keys back and did not make the maze any faster to walk.
    ///
    /// So this is a door of its own, and it is the *narrowest* one that answers
    /// the problem: it moves the reading and nothing else. **No tick is
    /// consumed** — no brew advances, no fire burns down, no spell runs — so
    /// walking a maze by hand costs world time only in the sense that the player
    /// is standing there doing it.
    ///
    /// # Replay is not weakened, and the reason is the recording
    ///
    /// A typed line is recorded against the tick it was *queued* on and executes
    /// at the start of the next; this executes immediately, so it lands after
    /// that tick's step. Both are exact, and they are told apart by
    /// [`Submission::Walked`](crate::session::Submission::Walked) rather than by
    /// a driver having to guess: replay a tick, then apply the walks recorded
    /// against it in list order. A tick can never hold both kinds, because the
    /// prompt is dead while the arrows have the maze.
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
        // **The same body `follow` runs**, so a hand-walked maze and a
        // spell-walked one cannot disagree about a wall or about what reaching
        // the exit is worth.
        crate::execute::tread(&mut self.world, way);
        true
    }

    /// Apply one recorded submission, with this sim already on its tick.
    ///
    /// **The replay driver, owned here rather than written out by each caller.**
    /// It was three hand-written `match`es across two test files and a third in
    /// the session tests, which is three chances to disagree about what a
    /// variant means — and what a variant means is precisely *when it ran*:
    ///
    /// - [`Typed`](Submission::Typed) was queued during its tick and executes at
    ///   the start of the next.
    /// - [`Wrote`](Submission::Wrote) is the same: a save queues like any effect.
    /// - [`Walked`](Submission::Walked) already ran, during its tick, after that
    ///   tick's step — because [`Sim::walk`] does not wait for a clock.
    ///
    /// Standing on the recorded tick and calling this is correct for all three,
    /// and adding a fourth kind is now a change in one place that the compiler
    /// insists on rather than three it does not.
    pub fn replay(&mut self, submission: Submission) {
        match submission {
            Submission::Typed(line) => self.submit(&line),
            Submission::Wrote { name, lines } => self.write_spell(&name, &lines),
            Submission::Took(id) => self.take(&id),
            Submission::Sang(word) => {
                if let Some(syllable) = tower::Syllable::from_word(&word) {
                    self.sing(syllable);
                }
            }
            Submission::Walked(word) => {
                if let Some(way) = tower::Way::ALL.into_iter().find(|way| way.word() == word) {
                    self.walk(way);
                }
            }
        }
    }

    /// Answer the syllable at the aperture by hand.
    ///
    /// **The fourth entry point**, and [`walk`](Self::walk)'s twin: it reaches
    /// the world without waiting for a tick boundary, because a chant is played
    /// on a key and a key that queued would arrive after the beat it was
    /// answering. Going through `submit` was tried for the maze three times and
    /// §19 records every version being some flavour of too slow; here it would
    /// not merely be slow, it would be *always wrong*.
    ///
    /// **No sub-tick phase, and none is wanted.** The sim grades on the tick a
    /// press arrives in — [`tower::chant::WINDOW`] is two ticks wide — so the
    /// only thing that crosses this boundary is *which syllable*. That is what
    /// keeps `orbs-sim` clockless (rules 1 and 3) and what makes the recorded
    /// [`Submission::Sang`](crate::session::Submission::Sang) exact rather than
    /// a float somebody has to argue about.
    ///
    /// Returns whether there was a chant to answer at all.
    pub fn sing(&mut self, syllable: tower::Syllable) -> bool {
        let Some(circle) = crate::execute::circle_at(&self.world) else {
            return false;
        };
        if self.world.get::<tower::Chant>(circle).is_none() {
            return false;
        }
        let tick = *self.world.resource::<Tick>();
        self.world
            .resource_mut::<Submissions>()
            .sang(tick, syllable.word());
        // **The same body the typed path runs**, which is `Sim::walk`'s rule: a
        // hand-sung chant and a scripted one cannot disagree about what a strike
        // is worth.
        crate::execute::strike_syllable(&mut self.world, circle, syllable);
        true
    }

    /// Whether a chant waits for the singer rather than for the clock (§14).
    #[must_use]
    pub fn is_patient(&self) -> bool {
        self.world.resource::<crate::execute::Patient>().is_set()
    }

    /// Flip that, and say what it became.
    pub fn set_patient(&mut self) -> bool {
        self.world
            .resource_mut::<crate::execute::Patient>()
            .toggle()
    }

    /// Whether `chorus` has asked for the arrow keys.
    #[must_use]
    pub fn has_chorusing(&self) -> bool {
        self.world.resource::<crate::execute::Chorusing>().pending()
    }

    /// Take `chorus`'s pending request, if there is one.
    pub fn chorusing(&mut self) -> bool {
        self.world
            .resource_mut::<crate::execute::Chorusing>()
            .take()
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
    pub fn ley_line(&self) -> Vec<tower::Node> {
        tower::ley_line(&self.world)
    }

    /// Mastery's tiers, in order, each with its nodes.
    #[must_use]
    pub fn mastery(&self) -> Vec<Vec<tower::Node>> {
        tower::mastery(&self.world)
    }

    /// Queue a tester's `debug_spawn`, on the next tick like everything else.
    ///
    /// Recorded in the scrollback and in `Submissions` — so a debug session
    /// replays in a debug build — but **not** in the parse trace, for the reason
    /// [`choose`](Self::choose) gives below: `debug_spawn` is not a phrasing, and
    /// counting it as one would dilute §15's first metric with inputs that were
    /// never a test of the parser. It is not even in the vocabulary being
    /// measured.
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
    /// **One prologue, because there were two and they were byte-identical.**
    /// `debug_ward` and `debug_course` each wrote out the same four steps —
    /// read the tick, echo the line, push a `Submission` so replay stays honest,
    /// walk `Cwd`'s children for a component — differing only in the type. A
    /// change to the protocol had to be made twice, and a fix applied to one was
    /// invisible in the other.
    ///
    /// The `Submission` is the load-bearing step: a debug word reaches the world
    /// without going through `submit`'s parser, so a replay that did not see it
    /// would diverge from the session that recorded it.
    ///
    /// Returns the node carrying `C`, or `None` — what each word does with it,
    /// and what it says when there is nothing, is the word's own business.
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
    /// **Straight into `Taken`, skipping `weave::grant`'s `Standing::Open`
    /// check** — which is the whole point, since that check is the threshold.
    /// Everything downstream is the real thing: `spell::budget` sums it,
    /// `is_gated` reads it, and `compile::check_learned` sees exactly what a
    /// played tower would.
    ///
    /// **A marker is refused**, because a tower holding one is a state the game
    /// cannot reach and so not one worth testing from — `debug_learn`'s rule
    /// about a name that is not a secret, applied here.
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
            .mastery()
            .iter()
            .flat_map(|tier| &tier.nodes)
            .filter(|node| tower::mastery::is_real(node))
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
            // **Republished, unlike `debug_ward`.** A ward's readings are
            // rewritten by the press that follows it; a course's are rewritten
            // by the *haul* that follows, and the two `potency` readings this
            // just moved are what a solver reads to decide which haul that is.
            // Left stale, the next line of a dump would be hauling at a board
            // that no longer exists.
            crate::execute::refresh_pylon(&mut self.world);
            return;
        }

        // **A shortcut that finds nothing says so.** It was silent, so typing it
        // before `muster` — or outside the sanctum, since the pylon is found
        // through `Cwd` — echoed the word, did nothing, and left the *next* line
        // to report the trouble: `haul` answering "there is nothing drawn to
        // move" reads as the haul being wrong rather than the shortcut. Every
        // other debug word in `submit` reports what it did or refused.
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

    /// Thin the enemy to one, so the next `hold` is the round that ends it.
    #[cfg(debug_assertions)]
    fn debug_siege(&mut self, line: &str) {
        if let Some(node) = self.debug_shortcut::<tower::Siege>(line)
            && let Some(mut siege) = self.world.get_mut::<tower::Siege>(node)
            && siege.running()
        {
            siege.give_away();
            // **Republished**, for `debug_course`'s reason rather than
            // `debug_ward`'s: a siege's readings are rewritten by the *round*
            // that follows, so `foes`, `outnumbered` and `massed` would all be
            // describing the enemy that arrived rather than the one left — and a
            // decision tree reads exactly those.
            crate::execute::refresh_rampart(&mut self.world);
            return;
        }

        // **A shortcut that finds nothing says so**, which is `debug_course`'s
        // rule: silent, it would leave the *next* line to report the trouble,
        // and `hold` answering "nothing is at the wall" reads as the hold being
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
    /// **Runs now rather than queueing**, like `debug_spell` and unlike
    /// `debug_spawn`: nothing about it is a world action with a duration, and a
    /// tester wants the next line of their dump to see the result.
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
    /// **Runs now rather than queueing, and records the write rather than the
    /// line.** Both are departures from [`debug_spawn`](Self::debug_spawn) and
    /// both are the same reason: the effect *is* a write, and
    /// [`write_spell`](Self::write_spell) already pushes a `Wrote` into
    /// `Submissions`. Recording the typed line as well would push two
    /// submissions for one input, and a replay would re-match the word and push
    /// two more.
    ///
    /// Recording only the write is also the stronger guarantee — a replay
    /// reproduces the **lines that ran**, even if `dev_spells.toml` is edited
    /// afterwards, where a recorded name would silently pick up the new text.
    ///
    /// Not in the parse trace, for the reason `debug_spawn` gives: it is not a
    /// phrasing, and §15's first metric measures phrasings.
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

    // There is deliberately no `scrollback_mut`. §3 makes this stream *the* log —
    // the transcript, the file a player `peruse`s, and what `orbs-balance`
    // replays — so a frontend writing a line into it directly would produce a
    // session that replaying `(seed, submissions)` cannot reproduce, which is
    // what §13 exists to stop. Commands write through
    // [`run_pending`](crate::execute::run_pending) on a tick boundary or they do
    // not write, and a test that needs to reach past that says so by going
    // through [`Sim::world_mut`] and its escape-hatch contract.

    /// What is in flight, if anything (§5.0).
    ///
    /// One at a time: §11.5 opens at multiplex capacity 1 and §9's fourth
    /// invariant reserves that slot for the action's whole duration.
    ///
    /// # Why `iter_entities` rather than a query
    ///
    /// A query needs `&mut World` to build its `QueryState`, and this takes
    /// `&self` because a **paint** calls it — the frontend holds the sim shared.
    /// The filter is by component, so it is correct; it is the *breadth* that is
    /// unfortunate.
    ///
    /// **Bevy 0.19 made that breadth wider.** Resources are components now, kept
    /// on dedicated entities, so this walks those too. Nothing here can carry
    /// `Working`, so the answer is unchanged — but a future broad walk that
    /// filters on something a resource *could* have would silently include them,
    /// which is the trap worth knowing about before writing the next one.
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
    /// **`None` everywhere but an archive with a maze open**, and for the same
    /// reason [`Sim::instruments`] is empty outside the laboratory: it goes
    /// through the stacks, which are found relative to `Cwd`. So the map is a
    /// property of where the player is, and a frontend cannot carry it out of
    /// the room and show something `survey` would not.
    ///
    /// Built fresh rather than cached, and called once a tick by the frontend's
    /// panel — a 49-cell `Vec` at 1 Hz, against `Instrument`'s recorded
    /// objection to allocating *per frame*, which is a different rate entirely.
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
    /// hosted; rule 8: no async here) and calls this **between** steps, so a
    /// reload lands on a tick boundary and never mid-schedule.
    ///
    /// # Replay
    ///
    /// Safe. No line reaches a decision — prose is presentation over a record
    /// that was already built, so `(seed, submissions)` still replays to the
    /// same world. **Recipes will not have this property**, and when they arrive
    /// the content they came from has to be versioned into the submission log.
    ///
    /// That claim was **false while `recall_` keys fed the scene**: every one is
    /// a `NounKind::Topic`, so renaming one mid-session changed what the parser
    /// resolves. [`Topics`](crate::tower::Topics) is snapshotted at construction
    /// and deliberately not touched here, which is what makes the paragraph above
    /// true again — a new manual subject needs a relaunch, its text does not.
    pub fn set_prose(&mut self, prose: Prose) {
        self.world.insert_resource(prose);
    }

    /// The orb's voice, for the handful of lines a **frontend** must speak.
    ///
    /// Almost nothing needs this: prose belongs on a record, and a frontend
    /// drawing a record gets the sentence with it. The exception is a screen the
    /// sim has no record for — §9's "window too small" — which is still authored
    /// prose and still rule 6's, so it is read from here rather than written as a
    /// literal in the Bevy crate.
    #[must_use]
    pub fn prose(&self) -> &Prose {
        self.world.resource::<Prose>()
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
    /// DESIGN.md §3's high-threat tonal register. In Phase 8 this is driven by
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
