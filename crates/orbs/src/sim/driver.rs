//! Driving the simulation from the frontend.
//!
//! Architectural rule 3: the frontend is a caller, not a host. Bevy's scheduler
//! runs one system, [`advance`], which calls [`Sim::step`] once; the sim's own
//! single-threaded schedule does the rest, in its own deterministic order.
//!
//! One tick is one real second (DESIGN.md §5.0), so the driver lives in
//! `FixedUpdate` at 1 Hz — frame rate must never change how fast the world
//! moves.

use bevy::prelude::*;
use orbs_render::Presentation;
use orbs_sim::Sim;

/// The simulated tower, owned by the frontend.
#[derive(Resource)]
pub(crate) struct Tower(Sim);

/// The readers that answer lines the orb cannot read itself (§6), if any.
///
/// Both registers under one setting, loaded once by
/// [`orbs_shell::Readers::from_environment`] for both builds.
#[derive(Resource, Default)]
pub(crate) struct Readers {
    /// The two readers, loaded once.
    loaded: orbs_shell::Readers,
    /// Whether the player wants them consulted.
    driver: orbs_shell::Driver,
}

impl Readers {
    /// Whatever `ORBS_AUGURY` and `ORBS_SCRIVENER` asked for.
    ///
    /// Every reader is chosen in `orbs_shell`, including the trained one: a dump
    /// builds no `App`, so a reader living in a Bevy resource is invisible to
    /// `scripts/dumps.sh`.
    pub(crate) fn from_environment() -> Self {
        // Never under `cargo test`: weights are a gitignored build artefact, so
        // a reader installed here would make every test that adds `SimPlugin`
        // pass or fail on whether someone ran the trainer. A test that wants one
        // installs it with [`Readers::holding`].
        if cfg!(test) {
            return Self::default();
        }
        Self {
            loaded: orbs_shell::Readers::from_environment(),
            driver: orbs_shell::settings::driver(),
        }
    }

    /// The reader, for handing to the sim — or nothing, if `plain` is chosen.
    pub(crate) fn reader(&self) -> Option<&dyn orbs_sim::Augur> {
        self.loaded.reader(self.driver)
    }

    /// The spell reader, on exactly the same terms.
    pub(crate) fn scrivener(&self) -> Option<&dyn orbs_sim::Scrivener> {
        self.loaded.scrivener(self.driver)
    }

    /// Read lines this way from now on.
    ///
    /// The menu has already written the choice down; this is what makes it true
    /// for the session already running.
    pub(crate) const fn drive(&mut self, driver: orbs_shell::Driver) {
        self.driver = driver;
    }

    /// Which driver is in effect, for the menu to mark.
    pub(crate) const fn driver(&self) -> orbs_shell::Driver {
        self.driver
    }

    /// A reader chosen directly rather than from the environment.
    ///
    /// For the one seam a dump cannot reach: `ORBS_DUMP` builds no `App`, so
    /// `scripts/dumps.sh` never exercises the Bevy message that carries a typed
    /// line to `Sim::submit_reading`.
    #[cfg(test)]
    pub(crate) fn holding(reader: Box<dyn orbs_sim::Augur>) -> Self {
        Self {
            loaded: orbs_shell::Readers::new(Some(reader), None),
            driver: orbs_shell::Driver::Augury,
        }
    }

    /// A spell reader chosen directly rather than from the environment.
    #[cfg(test)]
    pub(crate) fn copying(scribe: Box<dyn orbs_sim::Scrivener>) -> Self {
        Self {
            loaded: orbs_shell::Readers::new(None, Some(scribe)),
            driver: orbs_shell::Driver::Augury,
        }
    }
}

impl Tower {
    /// An open tower from a master seed — every room, for the tests that need
    /// one and never for a player.
    #[cfg(test)]
    pub(crate) fn new(seed: u64) -> Self {
        Self(Sim::new(seed))
    }

    /// The tower a fresh game builds, for the tests that want a player's start.
    ///
    /// Sealed unless `ORBS_SEALED=0` — a fresh game is a laboratory and nothing
    /// else (§11.5). `orbs_shell::fresh` is the one reader of the switch, so the
    /// two builds cannot disagree; a launch reaches it through
    /// `orbs_shell::tower_from`.
    #[cfg(test)]
    pub(crate) fn fresh(seed: u64) -> Self {
        Self(orbs_shell::fresh(
            seed,
            true,
            orbs_sim::content::Length::Medium,
        ))
    }

    /// The tower `new` or a wake raised, practice or real, as
    /// `orbs_shell::Raise` decided for both builds.
    ///
    /// Not `fresh` with an argument: `fresh` lets `ORBS_LENGTH` speak, while this
    /// length is an answer the player just typed.
    pub(crate) const fn raised(begun: Sim) -> Self {
        Self(begun)
    }

    /// Hand a line to the installation, which submits it or wakes the tower.
    ///
    /// Through `Install::submit` rather than [`submit_with`](Self::submit_with)
    /// and a check beside it, so both frontends and the dump take one path.
    pub(crate) fn submit_installing(
        &mut self,
        install: &mut orbs_shell::Install,
        line: &str,
        augur: Option<&dyn orbs_sim::Augur>,
    ) -> Option<orbs_shell::Raise> {
        install.submit(&mut self.0, line, augur)
    }

    /// Say the tower could not be written out.
    pub(crate) fn say_save_failed(&mut self) {
        self.0.say_save_failed();
    }

    /// [`submit_with`](Self::submit_with) with no reader, for the tests.
    #[cfg(test)]
    pub(crate) fn submit(&mut self, line: &str) {
        self.0.submit(line);
    }

    /// Hand a finished line to the sim, letting `augur` read it if the orb cannot.
    ///
    /// Echoes now and queues for the next `step()`, advancing nothing. The
    /// reader is no part of a tower, so an argument; see
    /// [`Sim::submit_reading`](orbs_sim::Sim::submit_reading).
    pub(crate) fn submit_with(&mut self, line: &str, augur: Option<&dyn orbs_sim::Augur>) {
        self.0.submit_with(line, augur);
    }

    /// Take a mastery node the weave screen chose.
    ///
    /// A third verb-shaped method rather than `sim_mut` — see
    /// [`Tower::opening`] for why. Like `submit_with` it queues, so the choice lands
    /// on the next tick rather than off a tick boundary.
    pub(crate) fn take(&mut self, id: &str) {
        self.0.take(id);
    }

    /// The world, for painting.
    pub(crate) const fn sim(&self) -> &Sim {
        &self.0
    }

    /// Leave echoes that only repeat the line out of the transcript, or not.
    ///
    /// A verb-shaped method for `take`'s reason. It writes no record.
    pub(crate) fn set_terse_echo(&mut self, terse: bool) {
        self.0.set_terse_echo(terse);
    }

    /// Take the spell `scribe` asked the editor to open, if any.
    ///
    /// Narrow on purpose: a general `sim_mut` would let any system reach the
    /// world outside a tick boundary, which rule 3 and `Sim::world_mut`'s
    /// contract both exist to stop.
    pub(crate) fn opening(&mut self) -> Option<orbs_sim::Request> {
        self.0.opening()
    }

    /// Whether `unfurl` has asked for the transcript to take the keyboard.
    ///
    /// Shaped like [`Tower::opening`]: the sim owns the decision, the frontend
    /// the scroll position. It takes rather than reads, so the mode is entered
    /// once per word rather than every frame.
    pub(crate) fn unfurling(&mut self) -> bool {
        self.0.unfurling()
    }

    /// Whether `quit` has asked for the session to end.
    ///
    /// The fifth, and the only one whose answer is not a surface: leaving is an
    /// `AppExit` here and raw mode restored in the terminal build, so the sim
    /// only records that it was asked for.
    pub(crate) fn quitting(&mut self) -> bool {
        self.0.quitting()
    }

    /// Whether either request is waiting, without mutating.
    ///
    /// `opening`/`unfurling` take `&mut self`, so asking through `ResMut<Tower>`
    /// stamps the change tick, leaving `resource_changed::<Tower>` true for ever
    /// — which put `refresh_panel` and `suggest` back to running every frame.
    /// Peek with this; reach for `&mut` only once there is something to take.
    pub(crate) fn has_opening(&self) -> bool {
        self.0.has_opening()
    }

    /// See [`Tower::has_opening`].
    pub(crate) fn is_unfurling(&self) -> bool {
        self.0.is_unfurling()
    }

    /// Whether `quit` is waiting, without mutating.
    ///
    /// The fifth handshake shipped without its peek: `quit_requested` reached
    /// for `ResMut` unconditionally, stamping `Tower` every frame, and its own
    /// run condition is `resource_changed::<Tower>`. See [`Tower::has_opening`]
    /// for the same regression the first time.
    pub(crate) fn is_quitting(&self) -> bool {
        self.0.is_quitting()
    }

    /// Whether `menu` is waiting, without mutating. See
    /// [`Tower::has_opening`] for the regression the peek exists to prevent.
    pub(crate) fn has_menuing(&self) -> bool {
        self.0.has_menuing()
    }

    /// Whether `menu` has asked for the orb's own screen.
    pub(crate) fn menuing(&mut self) -> bool {
        self.0.menuing()
    }

    /// Whether `weave` has asked for the progression screen, without mutating.
    /// See [`Tower::has_opening`].
    pub(crate) fn has_weaving(&self) -> bool {
        self.0.has_weaving()
    }

    /// Take `weave`'s pending request, if there is one.
    pub(crate) fn weaving(&mut self) -> bool {
        self.0.weaving()
    }

    /// Whether `wander` has asked for the arrow keys, without mutating. See
    /// [`Tower::has_opening`].
    pub(crate) fn has_wandering(&self) -> bool {
        self.0.has_wandering()
    }

    /// Take `wander`'s pending request, if there is one.
    pub(crate) fn wandering(&mut self) -> bool {
        self.0.wandering()
    }

    /// Walk the stacks one cell, now. See [`orbs_sim::Sim::walk`].
    pub(crate) fn walk(&mut self, way: orbs_sim::tower::Way) -> bool {
        self.0.walk(way)
    }

    /// Save a spell out of the editor.
    ///
    /// Keystrokes never reach the sim (see `shell::editor`), so this is the one
    /// call that makes an edit real — a submission like any typed line.
    #[cfg(test)]
    pub(crate) fn write_spell(&mut self, name: &str, lines: &[String]) {
        self.write_spell_with(name, lines, None);
    }

    /// ...with the reader that answers the lines the orb cannot read itself.
    ///
    /// The reading is stored beside the text, never over it: `Held` is
    /// byte-exact, `Read` is what `spell::compile` compiles. A `None` reader is
    /// the identity, so a build with no weights is the game as it was.
    pub(crate) fn write_spell_with(
        &mut self,
        name: &str,
        lines: &[String],
        scrivener: Option<&dyn orbs_sim::Scrivener>,
    ) {
        self.0
            .write_spell_reading(name, lines, scrivener.unwrap_or(&orbs_sim::Verbatim));
    }

    /// Replace the orb's authored voice (CLAUDE.md rule 6).
    ///
    /// Called from `content::reload` in `FixedUpdate`, so the swap lands on a
    /// tick boundary rather than part-way through a schedule.
    pub(crate) fn set_prose(&mut self, prose: orbs_sim::Prose) {
        self.0.set_prose(prose);
    }

    /// Replace the manual (CLAUDE.md rule 6).
    ///
    /// [`set_prose`](Self::set_prose)'s pair, on the same tick boundary. A
    /// reader already open keeps its chapter until the next keystroke
    /// re-assembles the book: the edit lands, the scroll position does not jump.
    pub(crate) fn set_manual(&mut self, manual: orbs_sim::content::Manual) {
        self.0.set_manual(manual);
    }

    /// Pour `ORBS_CONTENT` in, as every frontend does ([`orbs_shell::load`]),
    /// returning what did not load.
    pub(crate) fn load_content(&mut self) -> Vec<&'static str> {
        orbs_shell::load(&mut self.0)
    }

    /// Advance one tick, for a test that needs the world to have moved.
    ///
    /// Test-only, and it stays that way: rule 3 turns on the sim being advanced
    /// from exactly one place, [`advance`] in `FixedUpdate` at 1 Hz. A shipping
    /// caller would be a second driver.
    #[cfg(test)]
    pub(crate) fn step(&mut self) {
        self.0.step();
    }

    /// Step the tonal register through its three treatments.
    ///
    /// A preview of §3's eldritch register, which Phase 8 drives from threat.
    /// Here now because the three typefaces and §3's corruption exemption had no
    /// player-facing surface — they were proven by a `println!` in an example.
    pub(crate) fn cycle_register(&mut self) -> Presentation {
        // Order lives in `orbs_shell::cycle_register`: both frontends bind a key
        // to it, and two copies would disagree about where `Tampered` sits.
        orbs_shell::cycle_register(&mut self.0)
    }
}

/// Advance the world by exactly one tick.
///
/// `FixedUpdate` may run this several times in one frame after a stall, which is
/// correct: the world owes that time regardless of how long the GPU took.
pub(crate) fn advance(mut tower: ResMut<Tower>) {
    tower.0.step();
}
