//! The terminal's one motion switch, and the clocks that answer to it.

use orbs_shell::Showing;

use super::Session;

impl Session {
    /// Whether anything may move: the `motion` row, which is the whole switch
    /// here — there is no tube beside it. `Some` always, so every clock believes
    /// it, where the window has a `None` for *no camera yet*.
    pub(super) const fn moving(&self) -> Option<bool> {
        Some(self.held.motion)
    }

    /// Advance the fire, the crossing, the rolling numbers and the text typing
    /// itself, by `delta`.
    ///
    /// The window's `motion::advance`, `drive_passing` and `drive_reveal` in one
    /// place and in their order: the switch reaches the crossing before its
    /// clock moves. `overstep` is how far into the next world tick this falls.
    pub(super) fn animate(&mut self, delta: f32, overstep: f32) {
        let moving = self.moving();
        // `ORBS_FIRE=0` still wins underneath the row.
        self.bench
            .advance(delta, overstep.clamp(0.0, 1.0), moving, &self.panel);
        // Not before the tower is up: the crossing's first act is the tower
        // arriving out of the boot card, which is the frame that hands over.
        if !self.boot.is_live() {
            return;
        }
        self.passing.advance(0.0, moving);
        let showing = Showing::of(
            self.surfaces.held_open(&self.scroll),
            &self.panel,
            self.linear.showing(),
        );
        self.passing.drive(delta, &showing);
        self.climb.advance(delta, moving, &self.panel);
        self.reveal.drive(
            self.sim.scrollback().records(),
            delta,
            moving,
            self.held.pace,
        );
    }
}
