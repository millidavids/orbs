//! Crossings no clock produced — `ORBS_PASSAGE_AT`.
//!
//! A dump builds no `App` and advances no `Time`, so without these every
//! crossing draws at progress zero. `Bench::set_flare` exists for the same
//! reason.

use orbs_render::{Frame, Passage};

use super::clock::{DURATION, Passing};

impl Passing {
    /// Pose a crossing at `progress` through it.
    pub fn pose(&mut self, progress: f32, passage: Passage) {
        // `ORBS_PASSAGE=0` outranks `ORBS_PASSAGE_AT`, as `ORBS_FIRE` outranks
        // `ORBS_FIRE_PHASE`: a pose that defeated the off switch would make
        // `dumps.sh` depend on which of the two a capture line mentioned.
        if !self.permitted {
            return;
        }
        self.enabled = true;
        self.shape = passage;
        self.elapsed = Some(progress.clamp(0.0, 1.0) * DURATION);
    }

    /// Pose the crossing that arrives out of boot — `ORBS_PASSAGE_AT=wake:…`.
    ///
    /// `progress` runs across the arriving half, all this crossing has: `0.0` is
    /// the empty screen the card left behind, `1.0` the tower.
    pub fn pose_wake(&mut self, progress: f32) {
        if !self.permitted {
            return;
        }
        self.enabled = true;
        self.woken = true;
        self.waking = true;
        self.shape = Passage::Wipe;
        self.elapsed = Some(DURATION * (0.5 + progress.clamp(0.0, 1.0) * 0.5));
    }

    /// Keep `frame` whatever the clock is doing.
    ///
    /// The posed counterpart to [`keep`](Self::keep), which refuses while a
    /// crossing runs. A dump paints the screen being left, calls this, then runs
    /// the last command and paints again.
    pub fn pose_kept(&mut self, frame: &Frame) {
        frame.keep(frame.area(), &mut self.kept);
    }
}
