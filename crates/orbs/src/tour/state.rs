//! The tour as the window holds it: which scene, and how far through.

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use orbs_shell::tour::{Scene, Timeline};

use super::shots::Shots;

/// Seconds a scene stays once its steps are done, so their last effect is seen.
const REST: f32 = 1.5;

/// Seconds a brisk scene stays once its steps are done, if its hold is already
/// over: a surface opens on the frame after the tick it was asked on.
const BRISK_REST: f32 = 0.25;

/// The scenes, the one showing, and its steps.
#[derive(Resource)]
pub(crate) struct Tour {
    scenes: Vec<Scene>,
    at: usize,
    /// Begin `at` on the next `First`.
    due: bool,
    /// The scene's steps, and its clock.
    pub(super) timeline: Timeline,
    /// A person pressed a key: no more steps, no moving on.
    yours: bool,
    /// The release owed for the last key the tour pressed.
    pub(super) owed: Option<KeyboardInput>,
    /// The virtual clock was paused at `begin`, so what staging cost is not
    /// caught up; it starts again after the next frame's time update.
    pub(super) paused: bool,
    /// Screenshots, when the tour is taking them.
    pub(super) shots: Option<Shots>,
    /// Scenes rest only a moment after their steps: the tests'.
    brisk: bool,
}

impl Tour {
    /// `scenes`, the first due.
    pub(crate) fn new(scenes: Vec<Scene>, shots: Option<Shots>, brisk: bool) -> Self {
        Self {
            scenes,
            at: 0,
            due: true,
            timeline: Timeline::new(Vec::new()),
            yours: false,
            owed: None,
            paused: false,
            shots,
            brisk,
        }
    }

    /// The scene showing, or about to.
    pub(crate) fn scene(&self) -> &Scene {
        &self.scenes[self.at]
    }

    /// Which scene, from nought.
    pub(crate) const fn at(&self) -> usize {
        self.at
    }

    /// Whether a scene is waiting to begin.
    pub(crate) const fn is_due(&self) -> bool {
        self.due
    }

    /// The scene due, now begun: its steps start from nought.
    pub(super) fn begun(&mut self) -> Scene {
        self.due = false;
        self.yours = false;
        self.owed = None;
        let scene = self.scene().clone();
        self.timeline = Timeline::new(scene.steps.clone());
        scene
    }

    /// Whether a person has taken the scene.
    pub(crate) const fn is_yours(&self) -> bool {
        self.yours
    }

    /// A person pressed a key. A tour taking screenshots ignores it: the
    /// window takes the focus as it opens, and one stray key would stall it.
    pub(super) const fn take_over(&mut self) {
        if self.shots.is_none() {
            self.yours = true;
        }
    }

    /// The next scene, or this one again at the end.
    pub(super) fn forward(&mut self) {
        self.at = (self.at + 1).min(self.scenes.len() - 1);
        self.due = true;
    }

    /// The scene before, or this one again at the start.
    pub(super) const fn back(&mut self) {
        self.at = self.at.saturating_sub(1);
        self.due = true;
    }

    /// Whether this is the playlist's last scene.
    pub(crate) const fn is_last(&self) -> bool {
        self.at + 1 >= self.scenes.len()
    }

    /// Whether the scene has had its time: its steps done, its hold, or a rest
    /// after them, gone, and every shot it asked for taken.
    pub(crate) fn is_over(&self) -> bool {
        let Some(done) = self.timeline.done_at() else {
            return false;
        };
        // Brisk keeps the hold: a brew can land in it, and `ends` says so.
        let rest = if self.brisk { BRISK_REST } else { REST };
        let length = self.scene().hold.max(done + rest);
        let shot = self.shots.as_ref().is_none_or(|shots| !shots.pending());
        !self.yours && !self.due && shot && self.timeline.clock() >= length
    }

    /// The window's title while the scene shows.
    pub(super) fn title(&self) -> String {
        let scene = self.scene();
        format!(
            "O.R.B.S. tour {}/{} {}: {}",
            self.at + 1,
            self.scenes.len(),
            scene.name,
            scene.about
        )
    }
}

/// Run condition: a scene is waiting to begin.
pub(crate) fn scene_due(tour: Option<Res<Tour>>) -> bool {
    tour.is_some_and(|tour| tour.is_due())
}

/// Move on once a scene has had its time, unless it is the last.
pub(super) fn advance(mut tour: ResMut<Tour>) {
    if tour.is_over() && !tour.is_last() {
        tour.forward();
    }
}
