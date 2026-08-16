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

impl Tower {
    /// A tower from a master seed.
    pub(crate) fn new(seed: u64) -> Self {
        Self(Sim::new(seed))
    }

    /// Hand a finished line to the sim.
    ///
    /// The **only** other way the frontend touches the world, and it does not
    /// advance it: `submit` echoes immediately and queues any resolved command
    /// for the next `step()`. See `orbs_sim::session`.
    pub(crate) fn submit(&mut self, line: &str) {
        self.0.submit(line);
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

    /// Save a spell out of the editor.
    ///
    /// The editor's whole contribution to the world. Keystrokes never reach the
    /// sim — see `shell::editor` — so this is the one call that makes an edit
    /// real, and it is a submission like any typed line.
    pub(crate) fn write_spell(&mut self, name: &str, lines: &[String]) {
        self.0.write_spell(name, lines);
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
        let next = match self.0.register() {
            Presentation::Plain => Presentation::Eldritch,
            Presentation::Eldritch => Presentation::Tampered,
            Presentation::Tampered => Presentation::Plain,
        };
        self.0.set_register(next);
        next
    }
}

/// Advance the world by exactly one tick.
///
/// `FixedUpdate` may run this several times in one frame after a stall, which is
/// correct: the world owes that time regardless of how long the GPU took.
pub(crate) fn advance(mut tower: ResMut<Tower>) {
    tower.0.step();
}
