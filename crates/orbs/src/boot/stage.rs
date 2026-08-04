//! What the orb is doing between the window opening and the tower answering.
//!
//! DESIGN.md §4 makes *the tower's* boot a status report reflecting real world
//! state, and that report is built and shipped (`orbs_sim::tower::boot`). This
//! is the **machine** waking up in front of it: the tube striking, the frame
//! drawing itself, and the dependencies the game is made of reporting in.
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
/// Roughly four seconds all told. That is long for a fifth relaunch, which is
/// what the skip is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Stage {
    /// A dark tube. Nothing has happened yet.
    Dark,
    /// The strike: one flash, then one roll pass. See `crt::strike`.
    Strike,
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
    pub(crate) const SEQUENCE: [Self; 5] = [
        Self::Dark,
        Self::Strike,
        Self::Prompt,
        Self::Frame,
        Self::Post,
    ];

    /// How long this stage lasts.
    ///
    /// [`Stage::Strike`]'s 0.9 s is a **safety constraint, not a taste one** —
    /// see `crt::strike`, which spends two general flashes inside it and must
    /// stay under three per second.
    pub(crate) const fn duration(self) -> Duration {
        Duration::from_millis(match self {
            Self::Dark => 400,
            Self::Strike => 900,
            Self::Prompt => 300,
            Self::Frame => 800,
            Self::Post => 2000,
            Self::Live => 0,
        })
    }

    /// The stage after this one.
    pub(crate) const fn next(self) -> Self {
        match self {
            Self::Dark => Self::Strike,
            Self::Strike => Self::Prompt,
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
        !matches!(self, Self::Dark | Self::Strike)
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

    #[test]
    fn it_reaches_the_game_on_its_own() {
        let mut boot = Boot::new();
        let total: Duration = Stage::SEQUENCE.iter().map(|stage| stage.duration()).sum();
        boot.advance(total);
        assert!(boot.is_live(), "the sequence never finished");
    }

    #[test]
    fn it_passes_through_every_stage_in_order() {
        let mut boot = Boot::new();
        let mut seen = vec![boot.stage()];
        for _ in 0..200 {
            boot.advance(Duration::from_millis(50));
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
    fn the_strike_is_long_enough_for_what_it_spends() {
        // Cross-checked against `crt::strike`'s flash budget, which is where the
        // arithmetic lives. Stated here too because this is the number someone
        // would shorten to make the boot feel snappier, and it is the one number
        // in the sequence that is not a taste call.
        assert!(
            Stage::Strike.duration() >= Duration::from_millis(900),
            "the strike is shorter than its flash budget allows",
        );
    }
}
