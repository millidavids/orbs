//! What the orb is doing between the window opening and the tower answering.
//!
//! DESIGN.md §4 makes *the tower's* boot a status report reflecting real world
//! state, and that report is built and shipped (`orbs_sim::tower::boot`). This
//! is the **machine** waking up in front of it: the frame drawing itself, and
//! the dependencies the game is made of reporting in.
//!
//! **There is no prompt on any of these screens.** One typed itself here once,
//! caret and all, before the frame existed — an input line offered where nothing
//! can be typed, since every keyed system is gated on `booted`. The first
//! affordance the game showed was one that did not work.
//!
//! It opens on **black and nothing else**. A CRT strike shipped here first — a
//! flash and a sweeping band, then a flash alone — and neither survived being
//! looked at: the game is a wizard finding a computer inside a scrying orb, and
//! an orb does not power on like a monitor.
//!
//! Two screens, deliberately, and they must not read as one thing twice. The
//! split is visual as well as sequential — the POST is a centred title card with
//! no pane and no `name qty state` columns, and the pane border does not exist
//! until [`Stage::Post`] draws it. What follows the card is the tower's report,
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

use bevy_ecs::prelude::Resource;

/// The stages, in order, with how long each lasts.
///
/// Just under thirteen seconds all told, and it runs every time: the keypress
/// skip is gone (§19), and §4's *sticky* skip waits on Phase 11's settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    /// A dark tube. Nothing has happened yet.
    ///
    /// **The screen simply opens black.** A flash-and-strike shipped here first,
    /// then a flash alone, and neither earned its place: the game is a wizard
    /// looking into a scrying orb, and an orb does not power on like a monitor.
    /// What is left is a beat of dark before the frame draws, which reads as the
    /// orb being *found* rather than switched on.
    Dark,
    // A `Prompt` stage lived here, and the input line typed itself before the
    // frame drew. It is gone with the prompt it existed for: **nothing can be
    // typed during boot** — every keyed system is gated on `booted` — so an
    // input line on that screen was an affordance that did not work, offered
    // before anything else on screen did.
    //
    // Removing the drawing alone would have left the stage as 1.2 s of black
    // indistinguishable from a slow launch, which is the trap `Dark`'s own
    // duration note warns about.
    /// The card: the border draws itself while the logo prints and the
    /// dependencies report in.
    ///
    /// **One stage, because they happen at once.** The border used to have a
    /// stage of its own that finished before the card began, which made the
    /// opening two waits in a row — three and a bit seconds of a box drawing
    /// itself against nothing, and only then a name. They start together now,
    /// and the box closes while the logo is still spelling itself out.
    Post,
    /// The game.
    Live,
}

impl Stage {
    /// Every stage before [`Stage::Live`], in order.
    pub const SEQUENCE: [Self; 2] = [Self::Dark, Self::Post];

    /// How much of the card's time the border spends closing.
    ///
    /// The four runs cover a quarter of the cells one continuous line did, so
    /// this is a shorter *and* faster reveal than the stage it replaced: the box
    /// is whole while the logo is still arriving, which is the point — it frames
    /// the name rather than waiting for it.
    pub const FRAME_SHARE: f32 = 0.22;

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
    /// `ORBS_BOOT=0` skips the whole thing, and is a development affordance —
    /// boot runs once a launch and every "see it" pass would otherwise pay for
    /// it. There is no player-facing skip.
    #[must_use]
    pub const fn duration(self) -> Duration {
        Duration::from_millis(match self {
            Self::Dark => 600,
            Self::Post => 9000,
            Self::Live => 0,
        })
    }

    /// The stage after this one.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Dark => Self::Post,
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
    #[must_use]
    pub const fn world_runs(self) -> bool {
        matches!(self, Self::Live)
    }

    /// Whether the pane border is on screen yet.
    #[must_use]
    pub const fn has_frame(self) -> bool {
        matches!(self, Self::Post | Self::Live)
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
        // `ORBS_BOOT=0` lands straight in the game. Boot happens once per launch,
        // so without this every "see it" pass on anything else costs a four-second
        // wait — and CLAUDE.md's warning about unmaintained debug affordances is
        // about inventing surfaces nobody uses, not about the one that makes the
        // gate cheap to run.
        if std::env::var("ORBS_BOOT").is_ok_and(|value| value == "0") {
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
    /// Catching up in whole stages rather than clamping to one per frame: a
    /// stall during boot should not leave the sequence stuck part-way, for the
    /// same reason `render::blink` catches its phase up in whole half-cycles.
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

    // There was a `skip()` here, driven by any keystroke. It is gone: the
    // sequence is short and it is *character*, and a keypress skip made the
    // first thing a player does to the game be dismissing it (§19).
    //
    // §4's **sticky** skip is a different thing and still stands — a remembered
    // setting for someone on their fortieth launch, not a per-launch keypress —
    // and it needs somewhere to persist, which arrives with §15's Phase 11
    // settings screen. `Boot::finished` is the state it will select.
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
    fn a_finished_boot_is_the_same_state_as_a_watched_one() {
        // `ORBS_BOOT=0` and §4's future sticky skip both select `finished()`,
        // and it has to land in exactly the state sitting through the sequence
        // reaches — otherwise the dump and the game are drawing different worlds.
        let finished = Boot::finished();

        let mut waited = Boot::new();
        waited.advance(Duration::from_secs(30));

        assert_eq!(finished.stage(), waited.stage());
        assert!(finished.is_live() && waited.is_live());
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
    fn the_sequence_opens_on_nothing_at_all() {
        // The whole of "just open to black": the first stage puts nothing on
        // screen, and the sequence starts by *appearing* rather than by an
        // effect announcing it.
        assert!(!Stage::Dark.has_frame());
        assert!(Stage::Post.has_frame());
    }

    #[test]
    fn the_border_and_the_card_start_together() {
        // **They used to be two waits in a row**: three and a bit seconds of a
        // box drawing itself against nothing, and only then a name. One stage
        // now, so the first frame of the card has both a border beginning and a
        // logo beginning.
        assert_eq!(Stage::Post.frame_progress(0.0), 0.0);
        assert!(
            Stage::Post.frame_progress(Stage::FRAME_SHARE / 2.0) > 0.0,
            "the border had not started while the card was running",
        );
        // ...and the box is whole well before the card is done, so it frames the
        // name rather than racing it to the end.
        assert_eq!(Stage::Post.frame_progress(Stage::FRAME_SHARE), 1.0);
        assert!(
            Stage::Post.frame_progress(0.5) >= 1.0,
            "the border was still drawing halfway through the card",
        );
    }

    #[test]
    fn no_stage_offers_a_prompt() {
        // Nothing can be typed during boot — every keyed system is gated on
        // `booted` — so an input line on those screens was an affordance that
        // did not work, offered before anything else on screen did. The stage
        // that existed to draw it went with it — as, later, did the one that
        // drew the border on its own.
        assert_eq!(Stage::SEQUENCE.len(), 2);
        assert!(
            !Stage::SEQUENCE.iter().any(|stage| stage.world_runs()),
            "a boot stage let the world run"
        );
    }

    #[test]
    fn the_animated_stages_are_slow_enough_to_follow() {
        // The reason for the pacing: at 4.4 s for the whole sequence, the frame
        // drawing itself and the dependencies reporting were both over before
        // they could be read. A stage that animates needs to be seconds, not
        // fractions of one.
        assert!(
            Stage::Post.duration() >= Duration::from_secs(3),
            "the card is too quick to read",
        );
        // The border is a share of it rather than a stage, so it needs checking
        // in its own right: a quarter of nine seconds is two, which is a box
        // drawing itself rather than a box appearing.
        assert!(
            Stage::Post.duration().mul_f32(Stage::FRAME_SHARE) >= Duration::from_millis(1200),
            "the border closes too fast to watch",
        );
    }
}
