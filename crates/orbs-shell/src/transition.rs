//! Panes arriving and leaving over time.
//!
//! A pane used to appear between one frame and the next: `paint` derived the
//! pane count from the grid every frame, so crossing
//! [`DEEP_FOCUS_FLOOR`](orbs_render::DEEP_FOCUS_FLOOR) or pressing `F4` swapped
//! one screen for another with nothing in between.
//!
//! Purely how cells are drawn, never what appears — architectural rule 2. Every
//! pane the settled layout would show is still shown; only the geometry moves,
//! and `orbs-tui` is free to ignore all of it.
//!
//! What is stored is the pane *count*, not the rectangles. §19 fixed the grid,
//! so the original reason — a rectangle going stale when `F4` resized the whole
//! grid — is gone, and what is left is simply the smaller state: two numbers
//! rather than eight rectangles, re-derived every frame.
//!
//! The siege multiplex is what will move the count from two to four (see
//! `plugin::PANES`).

use bevy_ecs::prelude::Resource;
use orbs_render::{DisplayMode, GridSize, ScreenLayout, ScreenRequest};

/// How long a pane takes to arrive or leave.
///
/// Motion rather than a glitch, but never in the way of the next keystroke.
/// Dragging a window across `DEEP_FOCUS_FLOOR` retriggers it, so it must stay
/// well inside the time a person spends dragging.
const DURATION: f32 = 0.22;

/// Panes mid-flight.
#[derive(Resource, Debug, Clone, Copy)]
pub struct PaneTransition {
    from: u8,
    to: u8,
    elapsed: f32,
}

impl Default for PaneTransition {
    fn default() -> Self {
        Self::settled(1)
    }
}

impl PaneTransition {
    /// Nothing moving, showing `panes`.
    #[must_use]
    pub const fn settled(panes: u8) -> Self {
        Self {
            from: panes,
            to: panes,
            elapsed: DURATION,
        }
    }

    // `panes()` was here and went with the telemetry pane: the rail answers its
    // one question now, and `PANES` is 1. It comes back with multiplexing.

    /// Whether the panes have arrived.
    #[must_use]
    pub fn is_settled(&self) -> bool {
        self.from == self.to || self.elapsed >= DURATION
    }

    /// Aim at a new pane count.
    ///
    /// Reversal, not restart: dragging a window back and forth across the floor
    /// retargets repeatedly, and restarting each time is a strobe. Aiming back
    /// at where a transition came from runs the same motion backwards.
    pub fn retarget(&mut self, panes: u8) {
        if panes == self.to {
            return;
        }
        if panes == self.from && !self.is_settled() {
            self.elapsed = (DURATION - self.elapsed).max(0.0);
            std::mem::swap(&mut self.from, &mut self.to);
            return;
        }
        self.from = self.to;
        self.to = panes;
        self.elapsed = 0.0;
    }

    /// Let `delta` seconds pass.
    pub fn advance(&mut self, delta: f32) {
        if self.is_settled() {
            self.elapsed = DURATION;
            return;
        }
        self.elapsed = (self.elapsed + delta).min(DURATION);
    }

    /// Where the motion has reached, eased.
    ///
    /// Smoothstep, not linear: a pane that starts and stops abruptly reads as a
    /// jump with frames in the middle.
    fn progress(&self) -> f32 {
        if self.is_settled() {
            return 1.0;
        }
        let t = (self.elapsed / DURATION).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// The screen as it is drawn this frame.
    ///
    /// The settled layout untouched when nothing is moving, so the common path
    /// costs one `compute`. Where a pane arrives *from* is
    /// [`ScreenLayout::transition`]'s business; this owns only the clock.
    #[must_use]
    pub fn layout(&self, grid: GridSize, mode: DisplayMode, input_rows: u16) -> ScreenLayout {
        let to = settled_layout(grid, mode, self.to, input_rows);
        if self.is_settled() {
            return to;
        }
        let from = settled_layout(grid, mode, self.from, input_rows);
        ScreenLayout::transition(&from, &to, mode, self.progress())
    }
}

/// The rail is always asked for, and the layout decides whether it fits.
///
/// Not a setting and not animated: `ScreenLayout::compute` drops it whole below
/// the grid that can host it (§9's "the minimised half yields"). A toggle here
/// would put the decision in two places.
fn settled_layout(grid: GridSize, mode: DisplayMode, panes: u8, input_rows: u16) -> ScreenLayout {
    ScreenLayout::compute(&ScreenRequest {
        grid,
        main_panes: panes,
        rail: true,
        mode,
        input_rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::Rect;

    /// The grid, which is the only one the game has now.
    const WIDE_GRID: GridSize = orbs_render::GRID;

    fn at(transition: &PaneTransition, mode: DisplayMode) -> Vec<Rect> {
        transition.layout(WIDE_GRID, mode, 1).main().to_vec()
    }

    #[test]
    fn a_settled_transition_is_exactly_the_settled_layout() {
        // Every frame not mid-flight must produce the rectangles the layout
        // would have produced on its own.
        for mode in [DisplayMode::Deep, DisplayMode::Wide] {
            for panes in [1, 2] {
                let settled = settled_layout(WIDE_GRID, mode, panes, 1);
                assert_eq!(at(&PaneTransition::settled(panes), mode), settled.main());
            }
        }
    }

    #[test]
    fn a_new_pane_arrives_from_the_right_rather_than_the_corner() {
        // Lerping from `Rect::EMPTY` puts the newborn pane at the top-left at
        // half height, overlapping the pane it should be appearing beside.
        let mut transition = PaneTransition::settled(1);
        transition.retarget(2);
        let early = at(&transition, DisplayMode::Deep);

        let arriving = early[1];
        assert!(
            arriving.col > WIDE_GRID.cols / 2,
            "the new pane started at column {}, not at the right edge",
            arriving.col,
        );
        let settled = settled_layout(WIDE_GRID, DisplayMode::Deep, 2, 1).main()[1];
        assert_eq!(arriving.row, settled.row, "it should not move vertically");
        assert_eq!(arriving.rows, settled.rows, "it is full height throughout");
    }

    #[test]
    fn no_pane_ever_leaves_the_grid() {
        // Transitional layouts break `tiling`'s no-gap guarantee deliberately;
        // a pane off the edge is cells written past the frame.
        for mode in [DisplayMode::Deep, DisplayMode::Wide] {
            let mut transition = PaneTransition::settled(1);
            transition.retarget(2);
            for step in 0..=40 {
                transition.advance(DURATION / 40.0);
                for rect in at(&transition, mode) {
                    assert!(
                        u32::from(rect.col) + u32::from(rect.cols) <= u32::from(WIDE_GRID.cols)
                            && u32::from(rect.row) + u32::from(rect.rows)
                                <= u32::from(WIDE_GRID.rows),
                        "{mode:?} step {step}: {rect:?} escaped {WIDE_GRID:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn it_always_arrives() {
        let mut transition = PaneTransition::settled(1);
        transition.retarget(2);
        assert!(!transition.is_settled());
        transition.advance(DURATION * 2.0);
        assert!(transition.is_settled());
        assert_eq!(at(&transition, DisplayMode::Deep).len(), 2);
    }

    #[test]
    fn reversing_mid_flight_resumes_rather_than_restarting() {
        // Restarting on each crossing snaps the pane back to zero width, which
        // is a strobe rather than an animation.
        let mut transition = PaneTransition::settled(1);
        transition.retarget(2);
        transition.advance(DURATION * 0.75);
        let three_quarters_in = transition.progress();

        transition.retarget(1);
        // The field directly, since `panes()` went with the telemetry pane.
        assert_eq!(transition.to, 1);
        assert!(
            transition.progress() < three_quarters_in,
            "reversing restarted the motion instead of resuming it",
        );
        assert!(!transition.is_settled(), "reversing finished instantly");
    }

    #[test]
    fn a_pane_leaving_is_the_arrival_run_backwards() {
        let mut transition = PaneTransition::settled(2);
        transition.retarget(1);
        transition.advance(DURATION / 2.0);

        let leaving = at(&transition, DisplayMode::Deep);
        assert_eq!(leaving.len(), 2, "the departing pane stopped being painted");
        let settled = settled_layout(WIDE_GRID, DisplayMode::Deep, 2, 1).main()[1];
        assert!(
            leaving[1].cols < settled.cols,
            "the departing pane never narrowed",
        );
    }

    #[test]
    fn real_layouts_meet_exactly_at_every_point_in_the_motion() {
        // `tween` owns the same property over synthetic rectangles; this one
        // runs it through the layout the game asks for, so a change in how
        // `compute` divides a grid cannot open a seam silently. A one-cell gap
        // reads as a bright seam of background, an overlap as a doubled border.
        let grid = GridSize::new(60, 8);
        let mut transition = PaneTransition::settled(1);
        transition.retarget(2);

        for step in 0..=32 {
            let panes = transition.layout(grid, DisplayMode::Deep, 1);
            let main = panes.main();
            assert_eq!(main.len(), 2, "step {step}");
            let (left, right) = (main[0], main[1]);
            assert_eq!(
                left.col + left.cols,
                right.col,
                "step {step}: {left:?} and {right:?} do not meet",
            );
            assert_eq!(
                right.col + right.cols,
                grid.cols,
                "step {step}: the pair does not fill the grid",
            );
            transition.advance(DURATION / 32.0);
        }
    }
}
