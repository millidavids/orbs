//! Driving the simulation from the frontend.
//!
//! Architectural rule 3: **the frontend is a caller, not a host.** Bevy's
//! scheduler never runs sim systems. It runs exactly one system, [`advance`],
//! which calls [`Sim::step`] once. Everything inside the sim runs on the sim's
//! own single-threaded schedule, in its own deterministic order.
//!
//! One tick is one real second (DESIGN.md §5.0), so the driver lives in
//! `FixedUpdate` at 1 Hz rather than in `Update`. Frame rate must never change
//! how fast the world moves.

use bevy::prelude::*;
use orbs_render::Presentation;
use orbs_sim::Sim;

/// The simulated tower, owned by the frontend.
#[derive(Resource)]
pub(crate) struct Tower(Sim);

/// The readers that answer lines the orb cannot read itself (§6), if any.
///
/// **Both registers, under one setting.** The prompt's reader and the spell's
/// are two models over two answer spaces, and a player who has said *"read what
/// I mean"* has said it about their whole session — a second toggle for spells
/// would be one nobody finds and one that can disagree with the first.
///
/// `orbs_shell::augury` and `orbs_shell::scrivener` are the single readers of
/// the two switches, so this build and the terminal one cannot disagree about
/// what either means.
#[derive(Resource, Default)]
pub(crate) struct Readers {
    /// The prompt's reader, if the player wants one.
    ///
    /// **Loaded once and kept even while `plain` is chosen.** Switching drivers
    /// is a menu choice and must take effect on the next line typed; rebuilding
    /// a reader on the way out of a settings page would put a file read on a
    /// keystroke, and dropping it would make turning the setting back on cost
    /// one.
    reader: Option<Box<dyn orbs_sim::Augur>>,
    /// The spell reader, on the same terms.
    scribe: Option<Box<dyn orbs_sim::Scrivener>>,
    /// Whether the player wants them consulted.
    driver: orbs_shell::Driver,
}

impl Readers {
    /// Whatever `ORBS_AUGURY` asked for.
    ///
    /// **Every reader is chosen in `orbs_shell::augury`, including the trained
    /// one.** Answering `model` here instead put it somewhere `ORBS_DUMP` could
    /// never reach — a dump builds no `App`, so a reader living in a Bevy
    /// resource is invisible to `scripts/dumps.sh`, which is exactly the
    /// blindness the fixture augur exists to prevent. The shell takes an
    /// optional `orbs-augury` behind a feature instead, and `orbs-tui` leaves it
    /// off.
    pub(crate) fn from_environment() -> Self {
        // **Never under `cargo test`, now that a reader is the default.** Weights
        // are a gitignored build artefact, so a reader installed here would make
        // every test that adds `SimPlugin` behave one way on a machine that has
        // trained and another on a fresh clone — a suite that passes or fails on
        // whether someone ran the trainer is worse than no suite. A test that
        // wants one installs it with [`Readers::holding`].
        if cfg!(test) {
            return Self::default();
        }
        Self {
            reader: orbs_shell::augury(),
            scribe: orbs_shell::scrivener(),
            driver: orbs_shell::settings::driver(),
        }
    }

    /// The reader, for handing to the sim — or nothing, if `plain` is chosen.
    ///
    /// **Two switches, and they answer different questions.** `ORBS_AUGURY` says
    /// what this *build* has to offer and is a developer's override; the driver
    /// is the player's, and it decides whether what is on offer gets consulted.
    /// `off` leaves nothing to gate, so the setting is simply moot there.
    pub(crate) fn reader(&self) -> Option<&dyn orbs_sim::Augur> {
        match self.driver {
            orbs_shell::Driver::Plain => None,
            orbs_shell::Driver::Augury => self.reader.as_deref(),
        }
    }

    /// The spell reader, on exactly the same terms.
    pub(crate) fn scrivener(&self) -> Option<&dyn orbs_sim::Scrivener> {
        match self.driver {
            orbs_shell::Driver::Plain => None,
            orbs_shell::Driver::Augury => self.scribe.as_deref(),
        }
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
    /// **For the one seam a dump cannot reach.** `ORBS_DUMP` builds no `App`, so
    /// everything `scripts/dumps.sh` proves about a reader it proves about
    /// `Sim::submit_reading` — never about the Bevy message that carries a typed
    /// line to it.
    #[cfg(test)]
    pub(crate) fn holding(reader: Box<dyn orbs_sim::Augur>) -> Self {
        Self {
            reader: Some(reader),
            scribe: None,
            driver: orbs_shell::Driver::Augury,
        }
    }

    /// A spell reader chosen directly rather than from the environment.
    #[cfg(test)]
    pub(crate) fn copying(scribe: Box<dyn orbs_sim::Scrivener>) -> Self {
        Self {
            reader: None,
            scribe: Some(scribe),
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

    /// The tower a fresh game builds.
    ///
    /// **Sealed, unless `ORBS_SEALED=0`**: a fresh game is a laboratory and
    /// nothing else (§11.5), and `orbs_shell::fresh` is the one reader of the
    /// switch so this build and the terminal one cannot disagree about it.
    pub(crate) fn fresh(seed: u64) -> Self {
        Self(orbs_shell::fresh(
            seed,
            true,
            orbs_sim::content::Length::Medium,
        ))
    }

    /// The tower a *chosen* length builds, for a new game begun at the menu.
    ///
    /// **Not [`fresh`](Self::fresh) with an argument**, because the two differ in
    /// where the length comes from and that difference is the point: `fresh` is
    /// the game the binary starts with and lets `ORBS_LENGTH` speak, while this
    /// is a game the player has just been asked about and answered. An
    /// environment variable outranking an answer typed a second ago would be the
    /// wrong way round.
    ///
    /// Renamed here, unlike a restored tower: this **is** a new world, which is
    /// exactly the case `session::Wizard` says the environment may seed.
    pub(crate) fn begun(
        seed: u64,
        length: orbs_sim::content::Length,
        wizard: Option<&str>,
    ) -> Self {
        let mut tower = Self(Sim::begun(seed, length));
        if let Some(wizard) = wizard {
            tower.rename(wizard);
        }
        tower
    }

    /// The tower a save describes.
    ///
    /// **Not renamed afterwards.** `session::Wizard` is explicit that this is
    /// world state and *"a save outranks the environment"* — a frontend seeds
    /// the name from `USER` only when it is building a new world, or a player
    /// who renamed their wizard would be renamed back on every load.
    pub(crate) fn restored(save: &orbs_sim::Save) -> Self {
        Self(Sim::restored(save))
    }

    /// Say the tower was resumed, and how long it was dark.
    pub(crate) fn say_resumed(&mut self, away: Option<u64>) {
        self.0.say_resumed(away);
    }

    /// Say a save was there and could not be read.
    pub(crate) fn say_save_unreadable(&mut self) {
        self.0.say_save_unreadable();
    }

    /// Say the tower could not be written out.
    pub(crate) fn say_save_failed(&mut self) {
        self.0.say_save_failed();
    }

    /// Hand a finished line to the sim.
    ///
    /// The **only** other way the frontend touches the world, and it does not
    /// advance it: `submit` echoes immediately and queues any resolved command
    /// for the next `step()`. See `orbs_sim::session`.
    pub(crate) fn submit(&mut self, line: &str) {
        self.0.submit(line);
    }

    /// Hand a finished line to the sim, letting `augur` read it if the orb cannot.
    ///
    /// **A separate method rather than a field on `Tower`**, and a separate
    /// resource rather than a constructor argument, because a reader is not
    /// part of what a tower *is*: all four constructors build the same world
    /// whether or not one is installed, and threading an `Option` through each
    /// of them would say otherwise.
    ///
    /// `None` is [`submit`](Self::submit) exactly — see
    /// [`Sim::submit_reading`](orbs_sim::Sim::submit_reading) for the tiers.
    pub(crate) fn submit_with(&mut self, line: &str, augur: Option<&dyn orbs_sim::Augur>) {
        match augur {
            Some(augur) => self.0.submit_reading(line, augur),
            // Through [`submit`](Self::submit) rather than past it, so there is
            // one place this façade reaches the world with a typed line.
            None => self.submit(line),
        }
    }

    /// Take a mastery node the weave screen chose.
    ///
    /// **A third verb-shaped method, and not `sim_mut`** — the comment below
    /// says why, and a screen wanting to change the world is exactly the caller
    /// it is guarding against. Like `submit`, this queues: the choice lands on
    /// the next tick, so nothing reaches the world off a tick boundary.
    pub(crate) fn take(&mut self, id: &str) {
        self.0.take(id);
    }

    /// The world, for painting.
    pub(crate) const fn sim(&self) -> &Sim {
        &self.0
    }

    /// Take the spell `scribe` asked the editor to open, if any.
    ///
    /// Narrow on purpose. The frontend gets `&Sim` for painting and two
    /// verb-shaped methods for input; a general `sim_mut` would let any system
    /// reach the world outside a tick boundary, which is the thing rule 3 and
    /// `Sim::world_mut`'s contract both exist to stop.
    pub(crate) fn opening(&mut self) -> Option<orbs_sim::Request> {
        self.0.opening()
    }

    /// Whether `unfurl` has asked for the transcript to take the keyboard.
    ///
    /// The third of the narrow verb-shaped methods, and the same shape as
    /// [`Tower::opening`]: the sim owns the decision, the frontend owns the
    /// scroll position, and this takes rather than reads so the mode is entered
    /// once per word rather than every frame.
    pub(crate) fn unfurling(&mut self) -> bool {
        self.0.unfurling()
    }

    /// Whether `quit` has asked for the session to end.
    ///
    /// The fifth, and the only one whose answer is not a surface: leaving is an
    /// `AppExit` here and raw mode being put back in the terminal build, which
    /// is exactly why the sim only records that it was asked for.
    pub(crate) fn quitting(&mut self) -> bool {
        self.0.quitting()
    }

    /// Whether either request is waiting, **without** mutating.
    ///
    /// `opening`/`unfurling` take `&mut self`, so asking through `ResMut<Tower>`
    /// stamps the change tick — which left `resource_changed::<Tower>` true for
    /// ever and returned `refresh_panel` and `suggest` to running every frame,
    /// the exact thing their doc comments exist to prevent. Systems peek with
    /// this and only reach for `&mut` once there is something to take.
    pub(crate) fn has_opening(&self) -> bool {
        self.0.has_opening()
    }

    /// See [`Tower::has_opening`].
    pub(crate) fn is_unfurling(&self) -> bool {
        self.0.is_unfurling()
    }

    /// Whether `quit` is waiting, **without** mutating.
    ///
    /// The fifth handshake was the one that shipped without its peek: `Sim` grew
    /// this and nothing called it, while `quit_requested` reached for `ResMut`
    /// unconditionally — so it stamped `Tower` on every frame it ran, and since
    /// its own run condition is `resource_changed::<Tower>` it never stopped
    /// running. See [`Tower::has_opening`] for the same regression the first
    /// time.
    pub(crate) fn is_quitting(&self) -> bool {
        self.0.is_quitting()
    }

    /// Whether `menu` is waiting, **without** mutating. See
    /// [`Tower::has_opening`] for the regression the peek exists to prevent.
    pub(crate) fn has_menuing(&self) -> bool {
        self.0.has_menuing()
    }

    /// Whether `menu` has asked for the orb's own screen.
    pub(crate) fn menuing(&mut self) -> bool {
        self.0.menuing()
    }

    /// Whether `weave` has asked for the progression screen, **without**
    /// mutating. See [`Tower::has_opening`].
    pub(crate) fn has_weaving(&self) -> bool {
        self.0.has_weaving()
    }

    /// Take `weave`'s pending request, if there is one.
    pub(crate) fn weaving(&mut self) -> bool {
        self.0.weaving()
    }

    /// Whether `wander` has asked for the arrow keys, **without** mutating.
    /// See [`Tower::has_opening`].
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

    /// Whether `chorus` has asked for the arrow keys.
    pub(crate) fn has_chorusing(&self) -> bool {
        self.0.has_chorusing()
    }

    /// Take that request, if there is one.
    pub(crate) fn chorusing(&mut self) -> bool {
        self.0.chorusing()
    }

    /// Answer the syllable at the aperture, now. See [`orbs_sim::Sim::sing`].
    ///
    /// **Now, like `walk`.** A key that queued for the next tick would arrive
    /// after the beat it was answering, so it would not merely be slow — it
    /// would be wrong every time.
    pub(crate) fn sing(&mut self, syllable: orbs_sim::tower::Syllable) -> bool {
        self.0.sing(syllable)
    }

    /// Save a spell out of the editor.
    ///
    /// The editor's whole contribution to the world. Keystrokes never reach the
    /// sim — see `shell::editor` — so this is the one call that makes an edit
    /// real, and it is a submission like any typed line.
    #[cfg(test)]
    pub(crate) fn write_spell(&mut self, name: &str, lines: &[String]) {
        self.write_spell_with(name, lines, None);
    }

    /// ...with the reader that answers the lines the orb cannot read itself.
    ///
    /// **The reading is derived here and stored beside the text, never over
    /// it.** `Held` is byte-exact whatever a reader says; `Read` is what
    /// `spell::compile` compiles. A `None` reader is the identity — the two are
    /// then the same lines — so a build with no weights is the game exactly as
    /// it was.
    pub(crate) fn write_spell_with(
        &mut self,
        name: &str,
        lines: &[String],
        scrivener: Option<&dyn orbs_sim::Scrivener>,
    ) {
        self.0
            .write_spell_reading(name, lines, scrivener.unwrap_or(&orbs_sim::Verbatim));
    }

    /// Read the buffer of the spell called `name` the way the editor draws it,
    /// with a reader if there is one.
    pub(crate) fn read_spell_with(
        &self,
        name: &str,
        domain: &str,
        lines: &[String],
        scrivener: Option<&dyn orbs_sim::Scrivener>,
    ) -> Vec<orbs_sim::tower::spell::Reading> {
        self.0.read_spell_with(
            name,
            domain,
            lines,
            scrivener.unwrap_or(&orbs_sim::Verbatim),
        )
    }

    /// Replace the orb's authored voice (CLAUDE.md rule 6).
    ///
    /// Called from `content::reload` in `FixedUpdate`, so the swap lands on a
    /// tick boundary rather than part-way through a schedule.
    pub(crate) fn set_prose(&mut self, prose: orbs_sim::Prose) {
        self.0.set_prose(prose);
    }

    /// Name the wizard at the orb.
    pub(crate) fn rename(&mut self, name: &str) {
        self.0.rename(name);
    }

    /// Advance one tick, for a test that needs the world to have moved.
    ///
    /// **Test-only, and it stays that way.** In the game the sim is advanced
    /// from exactly one place — [`advance`], in `FixedUpdate` at 1 Hz — and rule
    /// 3 turns on that being true. A shipping caller of this would be a second
    /// driver, which is the thing the narrow surface above exists to prevent.
    #[cfg(test)]
    pub(crate) fn step(&mut self) {
        self.0.step();
    }

    /// Step the tonal register through its three treatments.
    ///
    /// A preview of §3's eldritch register, which Phase 8 drives from threat.
    /// It is here now because the three typefaces and §3's corruption exemption
    /// had no player-facing surface at all — they were proven by a `println!` in
    /// an example, which is not the same as having been looked at.
    pub(crate) fn cycle_register(&mut self) -> Presentation {
        // The order the key walks is `orbs_shell::cycle_register`'s, because
        // both frontends bind a key to it and two copies would eventually
        // disagree about where `Tampered` sits in the cycle.
        orbs_shell::cycle_register(&mut self.0)
    }

    /// §14's accommodation for the menagerie: a chant that waits.
    ///
    /// Through `orbs-shell` for `cycle_register`'s reason — both frontends bind
    /// a key to it, and two copies would eventually disagree.
    pub(crate) fn toggle_patient(&mut self) -> bool {
        orbs_shell::toggle_patient(&mut self.0)
    }
}

/// Advance the world by exactly one tick.
///
/// `FixedUpdate` may run this several times in one frame after a stall, which is
/// correct: the world owes that time regardless of how long the GPU took.
pub(crate) fn advance(mut tower: ResMut<Tower>) {
    tower.0.step();
}
