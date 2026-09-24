//! What the orb is doing between the window opening and the tower answering.
//!
//! DESIGN.md §4 makes *the tower's* boot a status report reflecting real world
//! state, and that report is built and shipped (`orbs_sim::tower::boot`). This
//! is the machine waking up in front of it: the frame drawing itself, and the
//! dependencies reporting in.
//!
//! No prompt on any of these screens: every keyed system is gated on `booted`,
//! so one that typed itself here was an affordance that did not work.
//!
//! It opens on black and nothing else. A CRT strike shipped here first and did
//! not survive being looked at — an orb does not power on like a monitor.
//!
//! Two screens, and the split is visual as well as sequential: the POST is a
//! centred title card with no pane, and the pane border does not exist until
//! [`Stage::Post`] draws it. What follows is the tower's report, already in the
//! scrollback where `Sim::new` put it.
//!
//! Wall-clock, so it lives here and not in the sim (rule 3): frontend state
//! driven by `Time`, as `render::blink` is.
//!
//! Nothing here touches a Bevy resource, because `shell::dump` builds no `App`
//! (§19). Painting takes a stage, a progress value and a `Frame`.

use core::time::Duration;

use bevy_ecs::prelude::Resource;

/// The stages, in order, with how long each lasts.
///
/// Just under thirteen seconds all told, and it runs unless the player has said
/// not to — §4's sticky skip, which [`Boot::default`] consults (§19).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    /// A dark tube. Nothing has happened yet.
    ///
    /// A beat of dark before the frame draws reads as the orb being *found*
    /// rather than switched on.
    Dark,
    // A `Prompt` stage that typed an input line lived here, and went: nothing
    // can be typed during boot. Removing only the drawing would have left
    // 1.2 s of black indistinguishable from a slow launch.
    /// The card: the border draws itself while the logo prints and the
    /// dependencies report in.
    ///
    /// One stage, because they happen at once. A separate border stage made the
    /// opening two waits in a row.
    Post,
    /// The card leaving: everything inside the box flies into the middle.
    ///
    /// The handover, given a beat of its own rather than the between-frames cut
    /// §19 spent an interlude removing from every other screen change. It is
    /// [`Passage::Gather`](orbs_render::Passage)'s leaving half.
    ///
    /// The box stays: the border the card drew is the pane the game arrives in,
    /// and the tower rail narrows it as it pushes in from the right.
    Close,
    /// The game.
    Live,
}

impl Stage {
    /// Every stage before [`Stage::Live`], in order.
    pub const SEQUENCE: [Self; 3] = [Self::Dark, Self::Post, Self::Close];

    /// How much of the card's time the border spends closing.
    ///
    /// The four runs cover a quarter of the cells one continuous line did, so
    /// the box is whole while the logo is still arriving.
    pub const FRAME_SHARE: f32 = 0.22;

    /// How long this stage lasts.
    ///
    /// Paced to be read, not got past: at 4.4 s for the sequence the animated
    /// stages went by faster than anyone could follow. [`Stage::Dark`] is the
    /// exception and is short, being a screen doing nothing.
    ///
    /// `ORBS_BOOT=0` skips the whole thing, since every "see it" pass would
    /// otherwise pay for it. No player-facing skip.
    #[must_use]
    pub const fn duration(self) -> Duration {
        Duration::from_millis(match self {
            Self::Dark => 600,
            // 9 s, then 5.5: §19's four-times-slower correction was made when
            // the card read as one slow event. It is three legible events in
            // sequence now, so the pauses between them only held it open. The
            // letters still get about a quarter-second each.
            Self::Post => 5500,
            // Half a crossing's length: the card spends only the *leaving* half
            // here, because this is the tower opening, not a room changing.
            Self::Close => 500,
            Self::Live => 0,
        })
    }

    /// The stage after this one.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Dark => Self::Post,
            Self::Post => Self::Close,
            Self::Close | Self::Live => Self::Live,
        }
    }

    /// Whether the tower's clock may run.
    ///
    /// Not cosmetic: `tower::drift` rolls once per tick, so a sim running
    /// through boot advances its RNG stream by a wall-clock-dependent number of
    /// draws and the same seed builds a different world.
    #[must_use]
    pub const fn world_runs(self) -> bool {
        matches!(self, Self::Live)
    }

    /// Whether the pane border is on screen yet.
    ///
    /// [`Stage::Close`] still draws it: the box has to be on screen to shrink
    /// with everything else in it.
    #[must_use]
    pub const fn has_frame(self) -> bool {
        matches!(self, Self::Post | Self::Close | Self::Live)
    }

    /// How far through the card's departure, or `None` if it is not leaving.
    ///
    /// Handed to [`Passage::Gather`](orbs_render::Passage) as the *leaving*
    /// half, so it runs `0.0` to the midpoint across this stage and the card is
    /// a single cell by the end of it.
    #[must_use]
    pub fn closing(self, progress: f32) -> Option<f32> {
        matches!(self, Self::Close).then(|| progress.clamp(0.0, 1.0))
    }

    /// How far the border has closed, given how far this stage has run.
    ///
    /// Its own clock inside the card's, so the box finishes early and the logo
    /// keeps going. Always whole once the game arrives.
    #[must_use]
    pub fn frame_progress(self, progress: f32) -> f32 {
        match self {
            Self::Post => (progress / Self::FRAME_SHARE).clamp(0.0, 1.0),
            _ => 1.0,
        }
    }
}

/// Where the boot sequence has reached.
#[derive(Resource, Debug, Clone, Copy)]
pub struct Boot {
    stage: Stage,
    elapsed: Duration,
}

impl Default for Boot {
    fn default() -> Self {
        // `ORBS_BOOT=0` lands straight in the game, so a "see it" pass on
        // anything else does not pay the wait.
        if std::env::var("ORBS_BOOT").is_ok_and(|value| value == "0") {
            return Self::finished();
        }
        // §4's sticky skip, set once in `settings`, `habits` (§19). `ORBS_BOOT`
        // outranks it: a setting that could override the switch would make a
        // capture depend on whose machine it ran on.
        if crate::settings::skips_boot() {
            return Self::finished();
        }
        Self::new()
    }
}

impl Boot {
    /// A boot sequence about to start.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stage: Stage::Dark,
            elapsed: Duration::ZERO,
        }
    }

    /// A boot sequence already over — what a dump and a skipped launch both get.
    #[must_use]
    pub const fn finished() -> Self {
        Self {
            stage: Stage::Live,
            elapsed: Duration::ZERO,
        }
    }

    /// The stage on screen.
    #[must_use]
    pub const fn stage(self) -> Stage {
        self.stage
    }

    /// Whether the game has arrived.
    #[must_use]
    pub const fn is_live(self) -> bool {
        matches!(self.stage, Stage::Live)
    }

    /// How far through the current stage, 0 → 1.
    #[must_use]
    pub fn progress(self) -> f32 {
        let total = self.stage.duration();
        if total.is_zero() {
            return 1.0;
        }
        (self.elapsed.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
    }

    /// Let `delta` pass, moving through as many stages as it covers.
    ///
    /// Catching up in whole stages rather than one per frame, so a stall does
    /// not strand the sequence — as `render::blink` catches its phase up.
    pub fn advance(&mut self, delta: Duration) {
        if self.is_live() {
            return;
        }
        self.elapsed = self.elapsed.saturating_add(delta);
        while !self.is_live() && self.elapsed >= self.stage.duration() {
            self.elapsed -= self.stage.duration();
            self.stage = self.stage.next();
        }
    }

    // A keystroke-driven `skip()` lived here and went (§19): the first thing a
    // player does to the game should not be dismissing it. §4's *sticky* skip
    // stands — `settings::skips_boot()` answers, `Boot::default` asks.
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How long the whole sequence lasts.
    fn total() -> Duration {
        Stage::SEQUENCE.iter().map(|stage| stage.duration()).sum()
    }

    #[test]
    fn it_reaches_the_game_on_its_own() {
        let mut boot = Boot::new();
        boot.advance(total());
        assert!(boot.is_live(), "the sequence never finished");
    }

    #[test]
    fn it_passes_through_every_stage_in_order() {
        let mut boot = Boot::new();
        let mut seen = vec![boot.stage()];
        // Against the sequence's own length, so retiming a stage cannot
        // silently stop this reaching the end.
        let step = Duration::from_millis(50);
        let limit = total().saturating_add(step);
        let mut elapsed = Duration::ZERO;
        while elapsed <= limit {
            boot.advance(step);
            elapsed = elapsed.saturating_add(step);
            if seen.last() != Some(&boot.stage()) {
                seen.push(boot.stage());
            }
        }
        let mut expected = Stage::SEQUENCE.to_vec();
        expected.push(Stage::Live);
        assert_eq!(seen, expected);
    }

    #[test]
    fn one_long_stall_does_not_strand_it_mid_sequence() {
        // A hitch is likeliest at startup — atlas built, shader compiled — and
        // one stage per frame would still be in `Dark` long after.
        let mut boot = Boot::new();
        boot.advance(Duration::from_secs(30));
        assert!(boot.is_live());
    }

    #[test]
    fn the_world_is_stopped_until_the_game_arrives() {
        // `tower::drift` rolls once per tick, so ticks during boot advance the
        // RNG stream by a wall-clock-dependent amount.
        for stage in Stage::SEQUENCE {
            assert!(!stage.world_runs(), "{stage:?} let the clock run");
        }
        assert!(Stage::Live.world_runs());
    }

    #[test]
    fn a_finished_boot_is_the_same_state_as_a_watched_one() {
        // Both skips select `finished()`, which must land in the state sitting
        // through the sequence reaches, or dump and game draw different worlds.
        let finished = Boot::finished();

        let mut waited = Boot::new();
        waited.advance(Duration::from_secs(30));

        assert_eq!(finished.stage(), waited.stage());
        assert!(finished.is_live() && waited.is_live());
    }

    #[test]
    fn the_screen_opens_black_and_does_not_dwell_there() {
        // `Dark` is a screen doing nothing, and for long enough that is
        // indistinguishable from a slow launch. A beat, not a wait.
        assert!(
            Stage::Dark.duration() <= Duration::from_millis(900),
            "the black screen outstays a beat",
        );
        assert!(
            Stage::Dark.duration() >= Duration::from_millis(200),
            "there is no beat at all",
        );
    }

    #[test]
    fn the_sequence_opens_on_nothing_at_all() {
        // The sequence starts by *appearing* rather than by an effect
        // announcing it.
        assert!(!Stage::Dark.has_frame());
        assert!(Stage::Post.has_frame());
    }

    #[test]
    fn the_border_and_the_card_start_together() {
        // They used to be two waits in a row: a box drawing itself against
        // nothing, and only then a name.
        assert_eq!(Stage::Post.frame_progress(0.0), 0.0);
        assert!(
            Stage::Post.frame_progress(Stage::FRAME_SHARE / 2.0) > 0.0,
            "the border had not started while the card was running",
        );
        // ...and whole well before the card is done, so it frames the name.
        assert_eq!(Stage::Post.frame_progress(Stage::FRAME_SHARE), 1.0);
        assert!(
            Stage::Post.frame_progress(0.5) >= 1.0,
            "the border was still drawing halfway through the card",
        );
    }

    #[test]
    fn no_stage_offers_a_prompt() {
        // Nothing can be typed during boot, so the stage that drew an input
        // line went. This used to count the stages, which fired when `Close`
        // was added and would have missed a returning `Prompt` beside it.
        assert!(
            !Stage::SEQUENCE.iter().any(|stage| stage.world_runs()),
            "a boot stage let the world run",
        );
        assert!(
            !Stage::SEQUENCE.contains(&Stage::Live),
            "the game is not a boot stage",
        );
    }

    #[test]
    fn the_sequence_is_the_path_next_actually_walks() {
        // Two expressions of one order: `SEQUENCE` is what a dump and the tests
        // iterate, `next` is what the clock follows. `Close` reached the enum
        // and `next` before it reached here.
        let mut walked = Vec::new();
        let mut stage = Stage::SEQUENCE[0];
        while !stage.world_runs() {
            walked.push(stage);
            stage = stage.next();
        }
        assert_eq!(walked, Stage::SEQUENCE, "the two orders disagree");
        assert_eq!(stage, Stage::Live, "the walk did not end at the game");
    }

    #[test]
    fn the_animated_stages_are_slow_enough_to_follow() {
        // At 4.4 s for the whole sequence it was over before it could be read.
        assert!(
            Stage::Post.duration() >= Duration::from_secs(3),
            "the card is too quick to read",
        );
        // The border is a share rather than a stage, so it needs its own check.
        assert!(
            Stage::Post.duration().mul_f32(Stage::FRAME_SHARE) >= Duration::from_millis(1200),
            "the border closes too fast to watch",
        );
    }
}
