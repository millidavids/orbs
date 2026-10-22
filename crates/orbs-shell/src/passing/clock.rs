//! When a crossing runs: the duration, the floor, and the kept screen.

use bevy_ecs::prelude::Resource;
use orbs_render::{Crossing, Frame, Kept, Passage, Toward};

use super::showing::Showing;
use crate::prompt::Crossed;

/// How long a whole crossing takes: out, then in.
///
/// Raised from seven frames by looking: that read as a flicker, not travel.
/// It must stay under [`FLOOR`], which §19 makes the bound on any animation.
pub(super) const DURATION: f32 = 0.5;

// Asserted rather than remembered: the floor bounds the rate, and a duration
// that reached it would quietly take over that job.
const _: () = assert!(
    DURATION < FLOOR,
    "a crossing must finish before the next one may begin",
);

/// The shortest time between two crossings *starting*, in seconds.
///
/// One world tick, and a photosensitivity bound (§19): one flash a second
/// against WCAG 2.3.1's three. A crossing the keyboard makes faster than that —
/// `F5` mashed — cuts instead.
pub(super) const FLOOR: f32 = 1.0;

/// A screen leaving and the next arriving.
#[derive(Resource, Debug)]
pub struct Passing {
    /// What the pane was showing when this last settled.
    showing: Option<Showing>,
    /// Seconds into the current crossing, or `None` when nothing is moving.
    pub(super) elapsed: Option<f32>,
    /// Seconds since the last crossing began. See [`FLOOR`].
    since: f32,
    /// The shape the current crossing runs by. Which regions move is the
    /// painter's to say, since it put them there.
    pub(super) shape: Passage,
    /// The screen being left.
    pub(super) kept: Kept,
    /// The regions that screen was using: the other half of the snapshot. See
    /// [`remember`](Self::remember).
    regions: Crossed,
    /// Whether the tower has been woken out of the boot card yet. Latched,
    /// because [`wake`](Self::wake) is called every live frame.
    pub(super) woken: bool,
    /// Whether the crossing in flight is that one — the only one that moves the
    /// tower rail.
    pub(super) waking: bool,
    /// Whether the effect runs at all — the motion switch, this session.
    pub(super) enabled: bool,
    /// Whether `ORBS_PASSAGE` permits it at all, read once.
    pub(super) permitted: bool,
}

impl Default for Passing {
    fn default() -> Self {
        Self::new(crate::dump::passage_permitted())
    }
}

impl Passing {
    /// A clock that crosses only if `permitted` — `ORBS_PASSAGE`'s answer,
    /// which [`Default`] reads.
    #[must_use]
    pub fn new(permitted: bool) -> Self {
        Self {
            showing: None,
            elapsed: None,
            // The floor starts elapsed, or the first screen change of a session
            // — a player walking out of the laboratory — would be refused.
            since: FLOOR,
            shape: Passage::Wipe,
            kept: Kept::default(),
            regions: Crossed::default(),
            woken: false,
            waking: false,
            enabled: permitted,
            permitted,
        }
    }

    /// One live frame: wake out of boot, let `delta` pass, and look at what the
    /// pane shows.
    ///
    /// The window's `drive_passing`, as one call, so both builds take the same
    /// three steps in the same order. The motion switch is not here: hand it over
    /// first with `advance(0.0, motion)`, which in the window is another system
    /// that reads the tube.
    pub fn drive(&mut self, delta: f32, showing: &Showing) {
        self.wake();
        self.advance(delta, None);
        self.observe(showing);
    }

    /// Note what the pane is showing, starting a crossing if it changed.
    ///
    /// The first `Showing` ever seen settles rather than crosses, or the frame
    /// after boot would cross out of nothing into the laboratory and move every
    /// `scripts/dumps.sh` baseline. `Reveal`'s first look settles the same way.
    pub fn observe(&mut self, showing: &Showing) {
        let Some(was) = self.showing.as_ref() else {
            self.showing = Some(showing.clone());
            return;
        };
        let Some(change) = was.change(showing) else {
            return;
        };
        // A change during a crossing does not restart it: the arriving half
        // already draws whatever is live. The floor stops a held key or a mashed
        // `F5` from strobing the pane.
        if self.enabled && self.elapsed.is_none() && self.since >= FLOOR && !self.kept.is_empty() {
            self.shape = change.passage();
            // An ordinary crossing, so the rail stays where it is.
            self.waking = false;
            self.elapsed = Some(0.0);
            self.since = 0.0;
        }
        self.showing = Some(showing.clone());
    }

    /// The game arriving out of the boot card.
    ///
    /// An arriving half only: `Stage::Close` has already taken the card away, so
    /// this starts at the midpoint. The furniture pushes in from the edges it
    /// lives against — the rail from the right, the gauges from the top.
    ///
    /// Called every live frame and does something once. Latching is cheaper than
    /// making a system remember, and cannot fire twice on a hitch the way an
    /// edge test could.
    pub fn wake(&mut self) {
        if self.woken {
            return;
        }
        self.woken = true;
        if !self.enabled {
            return;
        }
        self.shape = Passage::Wipe;
        self.waking = true;
        self.elapsed = Some(DURATION * 0.5);
        self.since = 0.0;
    }

    /// Whether the crossing in flight is the one that arrives out of boot.
    ///
    /// The rail moves for this one and for nothing else.
    #[must_use]
    pub const fn is_waking(&self) -> bool {
        self.waking && self.elapsed.is_some()
    }

    /// Let `delta` seconds pass, and learn whether motion is on.
    ///
    /// `motion` is `None` where a caller has no switch to hand over, which leaves
    /// whatever was last set rather than defaulting to off. A missing switch is
    /// not a switch set to off, and `shell::motion` records what treating them
    /// alike cost the fire.
    pub fn advance(&mut self, delta: f32, motion: Option<bool>) {
        if let Some(on) = motion {
            // The cached read, not a fresh one: `ORBS_PASSAGE=0` is a scripted
            // run's answer and the motion switch must not undo it.
            let permitted = self.permitted;
            self.set_enabled(on && permitted);
        }
        let delta = delta.max(0.0);
        self.since = (self.since + delta).min(FLOOR);
        let Some(elapsed) = self.elapsed else {
            return;
        };
        let elapsed = elapsed + delta;
        self.elapsed = (elapsed < DURATION).then_some(elapsed);
    }

    /// Turn the crossing on or off.
    ///
    /// Turning it off drops the kept screen as well as the crossing: a `Kept` is
    /// 43 KiB at the grid the game draws, and a player who has asked for no
    /// motion should not be paying for one.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.elapsed = None;
            self.waking = false;
            self.kept.clear();
        }
    }

    /// Whether nothing is moving.
    #[must_use]
    pub const fn is_settled(&self) -> bool {
        self.elapsed.is_none()
    }

    /// Keep this frame, for the next crossing to depart from.
    ///
    /// A no-op while a crossing runs, or it overwrites its own source with its
    /// own output. Called by a frontend just before `Frame::reset` blanks the
    /// screen, the last moment the previous frame exists. The whole grid, so
    /// [`Kept::area`] can say whether it still pictures the screen.
    pub fn keep(&mut self, frame: &Frame) {
        if self.enabled && self.is_settled() {
            frame.keep(frame.area(), &mut self.kept);
        }
    }

    /// Whether the kept screen still describes the frame being painted.
    ///
    /// A grid that changed under a crossing invalidates it — a window resize,
    /// `ORBS_GRID`, a terminal dragged narrower.
    #[must_use]
    pub fn describes(&self, frame: &Frame) -> bool {
        self.kept.area() == frame.area()
    }

    /// The crossing to draw this frame, if one is running.
    #[must_use]
    pub fn crossing(&self) -> Option<Crossing> {
        let elapsed = self.elapsed?;
        Some(Crossing {
            passage: self.shape,
            // Toward the tower rail, which is where the rooms are listed.
            // `Gather` ignores it, and says so.
            toward: Toward::Right,
            progress: (elapsed / DURATION).clamp(0.0, 1.0),
        })
    }

    /// The screen being left.
    #[must_use]
    pub const fn kept(&self) -> &Kept {
        &self.kept
    }

    /// Note the regions the settled screen is using, for the next crossing to
    /// leave from.
    ///
    /// The cells are [`keep`](Self::keep)'s, from the frontend before the frame
    /// is blanked; these are the rectangles, from the painter after it has
    /// drawn, because only it knows where a screen put its boards.
    pub(crate) const fn remember(&mut self, regions: Crossed) {
        self.regions = regions;
    }

    /// The regions the screen being left was using.
    #[must_use]
    pub(crate) const fn remembered(&self) -> Crossed {
        self.regions
    }
}
