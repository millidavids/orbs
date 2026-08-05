//! What the orb is doing between the window opening and the tower answering.
//!
//! DESIGN.md §4 makes *the tower's* boot a status report reflecting real world
//! state, and that report is built and shipped (`orbs_sim::tower::boot`). This
//! is the **machine** waking up in front of it: the prompt appearing, the frame
//! drawing itself, and the dependencies the game is made of reporting in.
//!
//! It opens on **black and nothing else**. A CRT strike shipped here first — a
//! flash and a sweeping band, then a flash alone — and neither survived being
//! looked at: the game is a wizard finding a computer inside a scrying orb, and
//! an orb does not power on like a monitor.
//!
//! Two screens, deliberately, and they must not read as one thing twice. The
//! split is visual as well as sequential — the POST is a centred title card with
//! no pane and no `name qty state` columns, and the pane border does not exist
//! until [`Stage::Frame`] draws it. What follows the card is the tower's report,
//! already sitting in the scrollback where `Sim::new` put it.
//!
//! # It is wall-clock, so it lives here and not in the sim
//!
//! Architectural rule 3: the sim advances only through `step()`, and nothing it
//! can observe may depend on how long a frame took. A boot animation is entirely
//! frame-rate driven, so it is frontend state driven by `Time` — the same
//! arrangement `render::blink` uses and for the same reason.
//!
//! # Nothing here touches a Bevy resource
//!
//! `shell::dump` runs before `App::new()` and builds no `App` at all, which §19
//! records as deliberate. A boot screen that could only be painted from inside a
//! system would be a screen `ORBS_DUMP` could never show, so painting takes a
//! stage, a progress value and a `Frame` and nothing else.

use core::time::Duration;

use bevy::prelude::Resource;

/// The stages, in order, with how long each lasts.
///
/// About fourteen seconds all told. That is long for a fifth relaunch, which is
/// what the skip is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Stage {
    /// A dark tube. Nothing has happened yet.
    ///
    /// **The screen simply opens black.** A flash-and-strike shipped here first,
    /// then a flash alone, and neither earned its place: the game is a wizard
    /// looking into a scrying orb, and an orb does not power on like a monitor.
    /// What is left is a beat of dark before the prompt, which reads as the orb
    /// being *found* rather than switched on.
    Dark,
    /// The prompt appears, with its caret.
    Prompt,
    /// The pane border draws itself, one cell at a time.
    Frame,
    /// Dependencies report in.
    Post,
    /// The game.
    Live,
}

impl Stage {
    /// Every stage before [`Stage::Live`], in order.
    pub(crate) const SEQUENCE: [Self; 4] = [Self::Dark, Self::Prompt, Self::Frame, Self::Post];

    /// How long this stage lasts.
    ///
    /// **Paced to be read, not to be got past.** The first version ran the whole
    /// sequence in 4.4 s and the stages that animate — the frame drawing itself,
    /// the dependencies reporting — went by faster than anyone could follow.
    /// Four times slower is the difference between a flicker and a screen.
    ///
    /// [`Stage::Dark`] is the exception and is *short*: it was 1.6 s when a
    /// flash punctuated the end of it, and with the flash gone it is a screen
    /// that does nothing, which is indistinguishable from a slow launch. Long
    /// enough to be a beat, not long enough to be a wait.
    ///
    /// The total is still long, which is what the skip is for: any key, and
    /// `ORBS_BOOT=0` for a session that never wants it.
    pub(crate) const fn duration(self) -> Duration {
        Duration::from_millis(match self {
            Self::Dark => 600,
            Self::Prompt => 1200,
            Self::Frame => 3200,
            Self::Post => 9000,
            Self::Live => 0,
        })
    }

    /// The stage after this one.
    pub(crate) const fn next(self) -> Self {
        match self {
            Self::Dark => Self::Prompt,
            Self::Prompt => Self::Frame,
            Self::Frame => Self::Post,
            Self::Post | Self::Live => Self::Live,
        }
    }

    /// Whether the tower's clock may run.
    ///
    /// **Not cosmetic.** `tower::drift` rolls once per tick, so a sim left
    /// running through boot advances its RNG stream by a wall-clock- and
    /// skip-dependent number of draws: the same seed would produce a different
    /// world depending on how long the animation took and whether anyone
    /// skipped it. The player would also read a tick-0 report beside a telemetry
    /// pane saying tick 4.
    pub(crate) const fn world_runs(self) -> bool {
        matches!(self, Self::Live)
    }

    /// Whether the pane border is on screen yet.
    pub(crate) const fn has_frame(self) -> bool {
        matches!(self, Self::Frame | Self::Post | Self::Live)
    }

    /// Whether the input line is on screen yet.
    pub(crate) const fn has_prompt(self) -> bool {
        !matches!(self, Self::Dark)
    }
}

/// Where the boot sequence has reached.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct Boot {
    stage: Stage,
    elapsed: Duration,
}

impl Boot {
    /// A boot sequence about to start.
    pub(crate) const fn new() -> Self {
        Self {
            stage: Stage::Dark,
            elapsed: Duration::ZERO,
        }
    }

    /// A boot sequence already over — what a dump and a skipped launch both get.
    pub(crate) const fn finished() -> Self {
        Self {
            stage: Stage::Live,
            elapsed: Duration::ZERO,
        }
    }

    /// The stage on screen.
    pub(crate) const fn stage(self) -> Stage {
        self.stage
    }

    /// Whether the game has arrived.
    pub(crate) const fn is_live(self) -> bool {
        matches!(self.stage, Stage::Live)
    }

    /// How far through the current stage, 0 → 1.
    pub(crate) fn progress(self) -> f32 {
        let total = self.stage.duration();
        if total.is_zero() {
            return 1.0;
        }
        (self.elapsed.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
    }

    /// Let `delta` pass, moving through as many stages as it covers.
    ///
    /// Catching up in whole stages rather than clamping to one per frame: a
    /// stall during boot should not leave the sequence stuck part-way, for the
    /// same reason `render::blink` catches its phase up in whole half-cycles.
    pub(crate) fn advance(&mut self, delta: Duration) {
        if self.is_live() {
            return;
        }
        self.elapsed = self.elapsed.saturating_add(delta);
        while !self.is_live() && self.elapsed >= self.stage.duration() {
            self.elapsed -= self.stage.duration();
            self.stage = self.stage.next();
        }
    }

    /// Go straight to the game.
    ///
    /// §4 asks for a **sticky** skip rather than a per-launch keypress, and
    /// sticky needs somewhere to persist it — which does not exist yet and
    /// arrives with §15's Phase 5 settings screen. Until then this is the
    /// keypress, which is the honest half-measure rather than a claim that the
    /// requirement is met.
    pub(crate) const fn skip(&mut self) {
        self.stage = Stage::Live;
        self.elapsed = Duration::ZERO;
    }
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
        // Run against the sequence's own length rather than a fixed step count,
        // so retiming a stage cannot silently stop this reaching the end.
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
        // A hitch during startup is exactly when this is most likely — the atlas
        // is being built and the shader compiled — and a sequence that advances
        // one stage per frame would still be in `Dark` well after the tube had
        // finished striking.
        let mut boot = Boot::new();
        boot.advance(Duration::from_secs(30));
        assert!(boot.is_live());
    }

    #[test]
    fn the_world_is_stopped_until_the_game_arrives() {
        // The determinism half: `tower::drift` rolls once per tick, so ticks
        // during boot would advance the RNG stream by a wall-clock-dependent
        // amount and the same seed would build a different world.
        for stage in Stage::SEQUENCE {
            assert!(!stage.world_runs(), "{stage:?} let the clock run");
        }
        assert!(Stage::Live.world_runs());
    }

    #[test]
    fn skipping_arrives_in_the_same_state_as_waiting() {
        let mut skipped = Boot::new();
        skipped.skip();

        let mut waited = Boot::new();
        waited.advance(Duration::from_secs(30));

        assert_eq!(skipped.stage(), waited.stage());
        assert!(skipped.is_live() && waited.is_live());
    }

    #[test]
    fn the_screen_opens_black_and_does_not_dwell_there() {
        // Nothing flashes any more, so `Dark` is a screen doing nothing — and a
        // screen doing nothing for long enough is indistinguishable from a slow
        // launch. A beat, not a wait.
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
    fn nothing_is_drawn_before_the_prompt() {
        // The whole of "just open to black": the first stage puts nothing on
        // screen, and the sequence starts by *appearing* rather than by an
        // effect announcing it.
        assert!(!Stage::Dark.has_prompt());
        assert!(!Stage::Dark.has_frame());
        assert!(Stage::Prompt.has_prompt());
    }

    #[test]
    fn the_animated_stages_are_slow_enough_to_follow() {
        // The reason for the pacing: at 4.4 s for the whole sequence, the frame
        // drawing itself and the dependencies reporting were both over before
        // they could be read. A stage that animates needs to be seconds, not
        // fractions of one.
        for stage in [Stage::Frame, Stage::Post] {
            assert!(
                stage.duration() >= Duration::from_secs(3),
                "{stage:?} is too quick to read",
            );
        }
    }
}
