//! The clock a screen leaves and arrives on.
//!
//! [`orbs_render::passage`] decides what a crossing *looks* like; this decides
//! *when* — the division `tween`, `bench` and `reveal` already draw, and the
//! reason nothing in `orbs-render` knows what a second is.
//!
//! It is [`PaneTransition`](crate::PaneTransition)'s twin and deliberately its
//! shape: that one animates a pane's **geometry**, this one animates a pane's
//! **content**. They are two clocks because they answer to different events —
//! `F4` and the multiplex move rectangles, `attend` and `wander` move screens —
//! and one clock doing both would have to interleave them.
//!
//! # What a crossing is between
//!
//! There is no `Screen` enum in this crate. Which view draws is **never
//! stored**: it is re-derived every frame by the branch chain in
//! [`prompt::paint`](crate::paint) from four independent pieces of state. So
//! this watches a derived value, [`Showing`], whose three fields are each chosen
//! against a specific false positive — see there.
//!
//! # It is off when the tube is off
//!
//! §14 requires motion be disableable and `court_wizard` ships a health warning
//! for motion sickness. The crossing rides the frontend's own motion switch,
//! passed to [`Passing::advance`] rather than queried here, exactly as
//! [`Bench`](crate::Bench) does — and `crt::settings` documents at length why
//! that distinction is load-bearing.
//!
//! **With motion off there is nothing kept and no crossing to freeze.** Not "a
//! quieter animation": DESIGN.md §19 records three animations that
//! independently learned *"a picture may never be able to vanish"*, because
//! reduce-motion pinned a phase and froze a blank. A crossing frozen at its
//! midpoint is an empty pane for the session.

use bevy_ecs::prelude::Resource;
use orbs_render::{Crossing, Frame, Kept, Passage, Toward};

use super::focus::{Focus, Open};
use super::glance::Panel;
use super::prompt::Crossed;

/// How long a whole crossing takes: out, then in.
///
/// **Settled by looking, and it went up.** It shipped at `0.24` — the number
/// [`PaneTransition`](crate::PaneTransition) uses for a pane arriving, borrowed
/// on the argument that a crossing should be *felt rather than watched*. Played,
/// that was wrong in a way no test could have said: a seam crossing a hundred
/// columns in seven frames reads as a flicker rather than as travel, and a
/// motion nobody can follow is decoration rather than a passage. §19 made the
/// same correction to the boot sequence, four times over, and recorded the
/// judgement — *"a boot sequence nobody can read is worse than one that takes a
/// beat."*
///
/// At half a second the seam moves about seven columns a frame at 60 Hz, which
/// is travel a person can follow across a pane.
///
/// **It is bounded on both sides, and neither bound is taste.** §19 caps any
/// animation at one world tick — *"the longest an animation can run and still
/// finish before anything it describes can change"* — and [`FLOOR`] is that same
/// tick. So this must stay **under** the floor, or a crossing would still be
/// running when the next one became permissible and crossings would be paced by
/// their own length instead of by the bound with the safety argument attached.
/// Half of it leaves as much margin again.
const DURATION: f32 = 0.5;

// The invariant the two constants above share, asserted rather than remembered:
// the floor is what bounds the rate, and a duration that reached it would
// quietly take over that job.
const _: () = assert!(
    DURATION < FLOOR,
    "a crossing must finish before the next one may begin",
);

/// The shortest time between two crossings *starting*, in seconds.
///
/// **One world tick, and it is the photosensitivity bound rather than a taste
/// one.** A crossing moves a cell's coverage one way and then back, which §19
/// counts as exactly one flash — *"a flash is a pair of opposing changes"* — so
/// what has to be bounded is not the shape but how often one may begin.
///
/// One per second against WCAG 2.3.1's limit of three, with the margin spent on
/// the thing that is actually uncertain: how much of the field a crossing
/// covers. It costs nothing real, because §5.0 turns the world at 1 Hz and a
/// room change cannot arrive faster than this. What it can refuse is a crossing
/// the *keyboard* makes — `F5` mashed, a tool opened and shut — and those cut,
/// which is what they did before any of this existed.
const FLOOR: f32 = 1.0;

/// What the pane is showing. Two of these differing is a crossing.
///
/// **Three fields rather than one, and each earns its place by a false positive
/// it prevents.** A fourth would need the same argument.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Showing {
    /// The surface that has taken the whole pane, if one has.
    ///
    /// [`Focus::takes_the_pane`], **not the whole `Focus`**. `Focus::Reading`
    /// would fire a crossing on `PgUp` and `Focus::Chant` on `chorus` — and
    /// `focus.rs` is explicit that a chant *"takes the keys and nothing else"*
    /// and deliberately keeps the transcript on screen. Neither replaces the
    /// pane, so neither is a screen change.
    tool: Option<Focus>,
    /// The room. [`Panel::room`], which is the room and not the leaf.
    room: String,
    /// Whether `F5`'s linear mirror is up.
    ///
    /// It is not a `Focus` variant at all — `prompt::paint` branches on
    /// `linear.showing()` separately — so without this the one surface that
    /// replaces the pane without taking the keyboard could never cross.
    mirrored: bool,
}

impl Showing {
    /// What the frontends' own state adds up to.
    ///
    /// One function so the two builds cannot disagree about what a screen change
    /// is, which is `focus::Focus::of`'s argument and this is the same shape.
    #[must_use]
    pub fn of(open: Open, panel: &Panel, mirrored: bool) -> Self {
        let focus = Focus::of(open);
        let tool = focus.takes_the_pane().then_some(focus);
        Self {
            tool,
            room: panel.room.clone(),
            // **Only when a tool is not already holding the pane**, which is the
            // order `prompt::paint_view` branches in: the editor, the weave
            // screen and the maze all return before it ever asks whether the
            // mirror is up. `F5` is not gated on focus, so pressing it inside the
            // editor flipped this and peeled the editor apart and back together
            // over a screen that had not changed at all.
            mirrored: mirrored && tool.is_none(),
        }
    }

    /// What kind of change this is, if it is one.
    ///
    /// **Ordered, because two can differ at once** — `wander` in the archive
    /// changes nothing else, but a spell that walks and then opens a maze can
    /// move both in one tick. The pane-replacing kinds win, because a shape that
    /// spared the transcript while the transcript was being replaced would draw a
    /// crossing around a hole with nothing in it.
    fn change(&self, next: &Self) -> Option<Change> {
        if self.tool != next.tool {
            Some(Change::Tool)
        } else if self.mirrored != next.mirrored {
            Some(Change::Mirror)
        } else if self.room != next.room {
            Some(Change::Room)
        } else {
            None
        }
    }
}

/// What kind of screen change is being crossed.
///
/// The motion names the kind, which is §19's *"tempo is the organising
/// principle"* applied to screens: what is running should be legible from across
/// the room without reading a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Change {
    /// `attend` somewhere else. The boards change; the transcript does not.
    Room,
    /// A full-pane surface opened or shut — `wander`, `edit`, `weave`.
    Tool,
    /// `F5`'s linear mirror, on or off. The same content, re-read.
    Mirror,
}

impl Change {
    /// The shape this kind of change leaves by.
    const fn passage(self) -> Passage {
        match self {
            Self::Room => Passage::Wipe,
            Self::Tool => Passage::Gather,
            Self::Mirror => Passage::Furl,
        }
    }
}

/// A screen leaving and the next arriving.
#[derive(Resource, Debug)]
pub struct Passing {
    /// What the pane was showing when this last settled.
    showing: Option<Showing>,
    /// Seconds into the current crossing, or `None` when nothing is moving.
    elapsed: Option<f32>,
    /// Seconds since the last crossing began. See [`FLOOR`].
    since: f32,
    /// The shape the current crossing runs by. Set when one starts; read until
    /// it settles.
    ///
    /// **The shape only, not which regions move.** That used to be here too, and
    /// it was a second answer to a question `prompt::paint` already has to
    /// settle: it is the painter that knows whether a surface replaced the pane,
    /// because it is the painter that put it there. Two expressions of one rule
    /// is how they come to disagree — §19 is full of it.
    shape: Passage,
    /// The screen being left.
    kept: Kept,
    /// The regions that screen was using.
    ///
    /// Beside [`kept`](Self::kept) because they are the two halves of one
    /// snapshot: its cells and where its parts were. See
    /// [`remember`](Self::remember).
    ///
    /// **The painter's own type, not three rectangles in a tuple.** It was the
    /// latter, and `View`'s doc names that exact hazard one module over — *"the
    /// kind of signature where transposing two compiles cleanly"*. Three
    /// same-typed positional values, passed through two functions, is the shape
    /// that warning is about.
    regions: Crossed,
    /// Whether the tower has been woken out of the boot card yet.
    ///
    /// Latched rather than edged, because [`wake`](Self::wake) is called every
    /// frame the game is live and must happen exactly once — the first of them.
    woken: bool,
    /// Whether the crossing in flight is that one.
    ///
    /// It is the only crossing that moves the **tower rail**, which every other
    /// one deliberately leaves standing: the rail is awareness and is drawn on
    /// every branch, so a room change must not take it away. Arriving out of
    /// boot is the one moment it is not there yet.
    waking: bool,
    /// Whether the effect runs at all — the tube's switch, this session.
    enabled: bool,
    /// Whether `ORBS_PASSAGE` permits it at all.
    ///
    /// Cached rather than re-read, exactly as [`Bench`](crate::Bench) caches
    /// `ORBS_FIRE`: polling the environment sixty times a second forever to
    /// answer a question settled at startup.
    permitted: bool,
}

impl Default for Passing {
    fn default() -> Self {
        Self {
            showing: None,
            elapsed: None,
            // **The floor starts elapsed.** Seeded at zero, the very first
            // screen change of a session would be refused — and the first one is
            // a player walking out of the laboratory, which is the crossing this
            // whole thing is for.
            since: FLOOR,
            shape: Passage::Wipe,
            kept: Kept::default(),
            regions: Crossed::default(),
            woken: false,
            waking: false,
            enabled: permitted(),
            permitted: permitted(),
        }
    }
}

/// Whether `ORBS_PASSAGE` permits crossings. See [`Passing::permitted`].
fn permitted() -> bool {
    crate::dump::passage_permitted()
}

impl Passing {
    /// Note what the pane is showing, starting a crossing if it changed.
    ///
    /// **The first `Showing` ever seen settles rather than crosses.** Otherwise
    /// the frame after the boot sequence would cross out of nothing into the
    /// laboratory, and every `scripts/dumps.sh` baseline in the repository would
    /// move. `Reveal::observe` resyncs the same way when its stream shrinks.
    pub fn observe(&mut self, showing: &Showing) {
        let Some(was) = self.showing.as_ref() else {
            self.showing = Some(showing.clone());
            return;
        };
        let Some(change) = was.change(showing) else {
            return;
        };
        // **A change during a crossing does not restart it**, and a change too
        // soon after one is refused. The arriving half already draws whatever is
        // live, so a change mid-arrival needs no new crossing at all; the floor
        // is what stops a held key or a mashed `F5` from strobing the pane.
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
    /// **An arriving half only.** `Stage::Close` has already taken the card away
    /// — the screen this departs from is gone, and there is nothing to leave —
    /// so this starts at the midpoint and runs to the end. The furniture pushes
    /// in from the edges it lives against: the tower rail from the right, the
    /// gauges and the road from the top.
    ///
    /// **Called every live frame and does something once.** The first frame the
    /// game paints *is* the boot edge, because everything here is gated on
    /// `booted`; latching is cheaper than making a system remember, and it
    /// cannot fire twice on a hitch the way an edge test could.
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

    /// Let `delta` seconds pass, and learn whether the tube is on.
    ///
    /// `motion` is `None` where a frontend has no switch to consult — a terminal
    /// has no CRT — which leaves whatever was last set rather than defaulting to
    /// off. A missing switch is not a switch set to off, and `shell::motion`
    /// records what treating them alike cost the fire.
    pub fn advance(&mut self, delta: f32, motion: Option<bool>) {
        if let Some(on) = motion {
            // **The cached read, not a fresh one**, and `Bench::advance` makes
            // the same point: `ORBS_PASSAGE=0` is a scripted run's answer and the
            // tube's switch must not be able to undo it.
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
    /// **A no-op while a crossing is running**, or it would overwrite its own
    /// source with its own output and eat itself. Called by a frontend just
    /// before `Frame::reset` blanks the screen, which is the last moment the
    /// previous frame exists at all.
    ///
    /// # The whole grid, not the pane
    ///
    /// The pane's interior is smaller and would be the obvious thing to keep. It
    /// is also a rectangle only [`prompt::paint`](crate::paint) knows, and this
    /// is called from a frontend before that runs — so keeping the pane would
    /// mean plumbing last frame's layout back out through a resource, and a
    /// stored rectangle is one that can go stale against the layout it came
    /// from.
    ///
    /// Keeping the grid costs 43 KiB against about 30, into a buffer that is
    /// reused, and it buys two things outright: [`Kept::area`] then answers *"is
    /// this a picture of the screen we still have?"* by itself, and a Tab
    /// listing shifting the pane by one row stops being a case at all.
    pub fn keep(&mut self, frame: &Frame) {
        if self.enabled && self.is_settled() {
            frame.keep(frame.area(), &mut self.kept);
        }
    }

    /// Whether the kept screen still describes the frame being painted.
    ///
    /// **A grid that changed under a crossing invalidates it**, and there are
    /// several ways to change one: a window resize, `ORBS_GRID`, a terminal
    /// dragged narrower. `orbs-tui`'s `Session::resized` records the same hazard
    /// for its own shadow buffer — *"the diff is addressed by `(col, row)`, so
    /// keeping it across a resize would write this frame's cells at last frame's
    /// coordinates."* A crossing would do exactly that.
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

    /// Pose a crossing that no clock produced — `ORBS_PASSAGE_AT`.
    ///
    /// A dump builds no `App` and advances no `Time`, so without this every
    /// crossing draws at progress zero for ever and the See-it line degrades to
    /// *"it compiles"*. `Bench::set_flare` exists for the same reason and says
    /// so.
    pub fn pose(&mut self, progress: f32, passage: Passage) {
        // `ORBS_PASSAGE=0` outranks `ORBS_PASSAGE_AT`, for the reason `ORBS_FIRE`
        // outranks `ORBS_FIRE_PHASE`: the off switch is what a scripted run sets,
        // and a pose that could defeat it would make `dumps.sh` depend on which
        // of the two a capture line happened to mention.
        if !self.permitted {
            return;
        }
        self.enabled = true;
        self.shape = passage;
        self.elapsed = Some(progress.clamp(0.0, 1.0) * DURATION);
    }

    /// Note the regions the settled screen is using, for the next crossing to
    /// leave from.
    ///
    /// The cells are [`keep`](Self::keep)'s and come from the frontend before the
    /// frame is blanked; these are the *rectangles* and come from the painter
    /// after it has drawn, because that is the only thing that knows where a
    /// screen put its boards. Two halves of one snapshot, taken a few lines
    /// apart for the same reason.
    pub(crate) const fn remember(&mut self, regions: Crossed) {
        self.regions = regions;
    }

    /// The regions the screen being left was using.
    ///
    /// `pub(crate)` with [`remember`](Self::remember), unlike the rest of this
    /// type: [`Crossed`] is the painter's private shape and a frontend has no
    /// business holding one. Both frontends hand a `Passing` to `paint` and it
    /// fills this in for them.
    #[must_use]
    pub(crate) const fn remembered(&self) -> Crossed {
        self.regions
    }

    /// Pose the crossing that arrives out of boot — `ORBS_PASSAGE_AT=wake:…`.
    ///
    /// `progress` runs across the **arriving half**, because that is all this
    /// crossing has: `0.0` is the empty screen the card left behind and `1.0` is
    /// the tower. A dump reaches it no other way — [`wake`](Self::wake) is
    /// called by a system, on the one frame the sequence hands over.
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

    /// Keep `frame` whatever the clock is doing — `ORBS_PASSAGE_AT`.
    ///
    /// The posed counterpart to [`keep`](Self::keep), which refuses while a
    /// crossing runs. A dump paints the screen being left, calls this, then runs
    /// the last command and paints again.
    pub fn pose_kept(&mut self, frame: &Frame) {
        frame.keep(frame.area(), &mut self.kept);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel(room: &str) -> Panel {
        Panel {
            room: room.to_owned(),
            ..Panel::default()
        }
    }

    fn showing(room: &str) -> Showing {
        Showing::of(Open::default(), &panel(room), false)
    }

    /// A `Passing` with a screen already kept and the floor already elapsed —
    /// what every frame of an ordinary session looks like.
    fn ready() -> Passing {
        let mut passing = Passing::default();
        passing.set_enabled(true);
        passing.pose_kept(&Frame::new(orbs_render::GridSize::new(4, 2)));
        passing.advance(FLOOR, None);
        passing
    }

    #[test]
    fn the_first_showing_settles() {
        // Otherwise the frame after boot crosses out of nothing into the
        // laboratory, and every `dumps.sh` baseline in the repository moves.
        let mut passing = ready();
        passing.observe(&showing("laboratory"));
        assert!(passing.is_settled());
        assert!(passing.crossing().is_none());
    }

    #[test]
    fn a_new_room_crosses() {
        let mut passing = ready();
        passing.observe(&showing("laboratory"));
        passing.observe(&showing("forge"));
        assert!(!passing.is_settled());
        // A `Wipe` is what a room change gets, and the shape is the whole of
        // what this resource decides — which regions move is the painter's, and
        // `prompt::Crossed` is where it is decided.
        assert_eq!(shape(&passing), Some(Passage::Wipe));
    }

    #[test]
    fn each_kind_of_change_has_its_own_shape() {
        // §19's *"tempo is the organising principle"*, applied to screens: what
        // is running should be legible from across the room without reading a
        // word. Three kinds of change, three motions, and the table is asserted
        // rather than left to the one place that writes it.
        let tool = Showing::of(
            Open {
                walking: true,
                ..Open::default()
            },
            &panel("archive"),
            false,
        );
        let mirror = Showing::of(Open::default(), &panel("archive"), true);
        for (next, expected) in [
            (showing("forge"), Passage::Wipe),
            (tool, Passage::Gather),
            (mirror, Passage::Furl),
        ] {
            let mut passing = ready();
            passing.observe(&showing("archive"));
            passing.observe(&next);
            assert_eq!(shape(&passing), Some(expected), "for {next:?}");
        }
    }

    #[test]
    fn a_tool_outranks_a_room_when_both_move() {
        // A spell can walk and then open a maze inside one tick, so both fields
        // differ at the next observation. Sparing the transcript there would draw
        // a crossing around a hole with nothing left in it.
        let mut passing = ready();
        passing.observe(&showing("laboratory"));
        passing.observe(&Showing::of(
            Open {
                walking: true,
                ..Open::default()
            },
            &panel("archive"),
            false,
        ));
        assert_eq!(shape(&passing), Some(Passage::Gather));
    }

    /// The shape a crossing is running by, if one is.
    fn shape(passing: &Passing) -> Option<Passage> {
        passing.crossing().map(|crossing| crossing.passage)
    }

    #[test]
    fn a_fixture_inside_the_room_is_not_a_crossing() {
        // `attend alembic` from the laboratory changes the *leaf* and not the
        // room, and `Panel::room` is what makes the two different questions.
        let mut passing = ready();
        passing.observe(&showing("laboratory"));
        passing.observe(&showing("laboratory"));
        assert!(passing.is_settled());
    }

    #[test]
    fn pgup_and_chorus_are_not_crossings() {
        // Neither takes the pane. `focus.rs` is explicit that a chant "takes the
        // keys and nothing else" and deliberately keeps the transcript, and the
        // transcript scrolling back does not replace anything at all.
        for open in [
            Open {
                reading: true,
                ..Open::default()
            },
            Open {
                chorusing: true,
                ..Open::default()
            },
        ] {
            let mut passing = ready();
            passing.observe(&showing("archive"));
            passing.observe(&Showing::of(open, &panel("archive"), false));
            assert!(passing.is_settled(), "{open:?} started a crossing");
        }
    }

    #[test]
    fn a_tool_takes_the_whole_pane() {
        let mut passing = ready();
        passing.observe(&showing("archive"));
        passing.observe(&Showing::of(
            Open {
                walking: true,
                ..Open::default()
            },
            &panel("archive"),
            false,
        ));
        assert!(!passing.is_settled());
        assert_eq!(
            shape(&passing),
            Some(Passage::Gather),
            "the maze flies into the middle rather than wiping",
        );
    }

    #[test]
    fn the_mirror_crosses_and_furls() {
        // `F5` is not a `Focus` variant, so without `Showing::mirrored` the one
        // surface that replaces the pane without taking the keyboard would never
        // cross at all.
        let mut passing = ready();
        passing.observe(&showing("lens"));
        passing.observe(&Showing::of(Open::default(), &panel("lens"), true));
        assert!(!passing.is_settled());
        assert_eq!(shape(&passing), Some(Passage::Furl));
    }

    #[test]
    fn a_crossing_cannot_begin_within_a_tick_of_the_last() {
        // The photosensitivity floor, asserted rather than argued. A held key or
        // a mashed `F5` would otherwise strobe the pane at four crossings a
        // second, and §19 counts each crossing as one flash.
        let mut passing = ready();
        passing.observe(&showing("laboratory"));
        passing.observe(&showing("forge"));
        assert!(!passing.is_settled());

        // The crossing finishes, but the floor has not.
        passing.advance(DURATION, None);
        assert!(passing.is_settled());
        passing.observe(&showing("lens"));
        assert!(
            passing.is_settled(),
            "a second crossing began inside the floor"
        );

        // ...and once the tick is up it may.
        passing.advance(FLOOR, None);
        passing.observe(&showing("sanctum"));
        assert!(!passing.is_settled());
    }

    #[test]
    fn a_change_mid_crossing_does_not_restart_it() {
        // The arriving half draws whatever is live, so a second change needs no
        // second crossing — and restarting would double the flash count that the
        // floor exists to bound.
        let mut passing = ready();
        passing.observe(&showing("laboratory"));
        passing.observe(&showing("forge"));
        passing.advance(DURATION / 2.0, None);
        let midway = passing.crossing().map(|crossing| crossing.progress);

        passing.observe(&showing("lens"));
        let after = passing.crossing().map(|crossing| crossing.progress);
        assert_eq!(midway, after, "the crossing restarted");
    }

    #[test]
    fn motion_off_keeps_nothing_and_starts_nothing() {
        // §14's requirement, and the `DESIGN.md:9182` trap with it: reduce-motion
        // must *cut*, never freeze a half-drawn screen. With nothing kept there
        // is no crossing to freeze.
        let mut passing = ready();
        passing.observe(&showing("laboratory"));
        passing.advance(0.0, Some(false));
        assert!(
            passing.kept().is_empty(),
            "a kept screen survived the switch"
        );

        passing.keep(&Frame::new(orbs_render::GridSize::new(4, 2)));
        assert!(
            passing.kept().is_empty(),
            "it kept a screen with motion off"
        );

        passing.observe(&showing("forge"));
        assert!(passing.is_settled(), "it crossed with motion off");
    }

    #[test]
    fn a_missing_switch_is_not_a_switch_set_to_off() {
        // A terminal has no CRT to consult and passes `None`. Defaulting to off
        // there would make the crossing invisible in that build for a reason
        // nobody asked for — the defect `shell::motion` records for the fire.
        let mut passing = ready();
        passing.advance(0.0, None);
        passing.observe(&showing("laboratory"));
        passing.observe(&showing("forge"));
        assert!(!passing.is_settled());
    }

    #[test]
    fn keeping_is_refused_while_a_crossing_runs() {
        // Or it overwrites its own source with its own output and eats itself.
        let mut passing = ready();
        passing.observe(&showing("laboratory"));
        passing.observe(&showing("forge"));

        let mut frame = Frame::new(orbs_render::GridSize::new(4, 2));
        let whole = frame.area();
        frame
            .painter(whole)
            .fill(whole, 'x', orbs_render::Style::NORMAL);
        passing.keep(&frame);

        assert!(
            passing.kept().cell(orbs_render::Pos::ORIGIN).is_blank(),
            "the crossing overwrote what it was departing from",
        );
    }

    #[test]
    fn a_crossing_ends_at_its_duration() {
        let mut passing = ready();
        passing.observe(&showing("laboratory"));
        passing.observe(&showing("forge"));
        passing.advance(DURATION - 0.01, None);
        assert!(!passing.is_settled());
        passing.advance(0.02, None);
        assert!(passing.is_settled());
        assert!(passing.crossing().is_none());
    }
}
