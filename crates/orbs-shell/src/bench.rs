//! The clock §10.1's instruments animate on.
//!
//! The frontend half of [`fire`](orbs_render::Burn) and
//! [`grind`](orbs_render::Grind): those decide what a cell *is*, this decides
//! *when*. Driven from `Time`, never from the tick — the sim must not be able
//! to observe it, or replay would depend on how long a frame took.
//!
//! One clock, not one per instrument: they share `orbs_render`'s tick grid,
//! which is where the photosensitivity guarantee lives. What differs is the
//! *view* this hands out — [`Bench::burn`], [`Bench::grind`].
//!
//! It is off when the tube is off. §14 requires motion be disableable, and
//! nothing else in the game flashes, so a shimmering meter on the laboratory's
//! permanent fixture needs a switch a player can reach. It rides the frontend's
//! own motion switch, passed to [`Bench::advance`] and read explicitly rather
//! than derived — `crt::settings` has why. F3 cycles to `OFF`, and that stops
//! the fire too.
//!
//! `ORBS_FIRE=0` is for scripted runs, not for players; the per-effect toggle
//! belongs to the settings item in `ROADMAP.md`.

use bevy_ecs::prelude::Resource;
use orbs_render::{Burn, Grind, Motion, Steep, Stir};
use orbs_sim::tower::{Craft, State};

/// How long a fire takes to catch, in seconds.
///
/// One world tick (DESIGN.md §5.0): the longest an animation can run and still
/// be over before anything it describes can change. It is also about as slow as
/// it can be without `kindle` reading as the panel lagging.
const FLARE_SECS: f32 = 1.0;

/// How long a bowl takes to fill, in seconds. One world tick, for the flare's
/// reason: the pour is over before the next thing happens to what went in.
const LOAD_SECS: f32 = 1.0;

/// Where every instrument's animation has reached, in seconds.
#[derive(Resource, Debug)]
pub struct Bench {
    /// Elapsed time, wrapped into `orbs_render`'s own cycle.
    phase: f32,
    /// Seconds left of the flare `kindle` set off, counting down.
    flare: f32,
    /// Seconds left of the pour a reagent entering the mortar set off.
    load: f32,
    /// Whether the mortar held anything last frame — the edge
    /// [`Bench::observe_bowl`] watches for.
    was_charged: bool,
    /// Whether the athanor was alight last frame — the edge
    /// [`Bench::observe_hearth`] watches for.
    was_lit: bool,
    /// How far through the current world tick, `0.0..1.0`.
    ///
    /// Read from the fixed scheduler rather than counted here: a second copy
    /// drifts the moment the clock hitches, and drift shows as a bar reaching
    /// the next cell off the tick it is meant to land on. The sim ticks in
    /// `FixedUpdate`, so this *is* that tick's position by construction.
    advance: f32,
    /// Whether the effect runs at all. See the module header.
    enabled: bool,
    /// Whether `ORBS_FIRE` permits any motion, read once.
    ///
    /// Separate from [`Self::enabled`], which the tube's switch drives every
    /// frame. An env var cannot change during a run, and reading it per frame
    /// takes the process environment lock and allocates for a constant.
    permitted: bool,
}

impl Default for Bench {
    fn default() -> Self {
        Self {
            phase: 0.0,
            flare: 0.0,
            // Starts lit: a tower that opens with the athanor already burning
            // must not flare on its first frame, and the edge watched for is
            // off-to-on.
            was_lit: true,
            load: 0.0,
            // Starts charged, for `was_lit`'s reason: a tower that opens with
            // something in the mortar must not pour it in again.
            was_charged: true,
            advance: 0.0,
            // Read once at startup rather than polling the environment sixty
            // times a second forever, exactly as `ORBS_CAPTURE` is.
            enabled: permitted(),
            permitted: permitted(),
        }
    }
}

/// Whether `ORBS_FIRE` permits motion. See [`Bench::permitted`].
fn permitted() -> bool {
    std::env::var(FIRE).as_deref() != Ok("0")
}

/// The switch that turns every instrument's motion off for a scripted run.
///
/// Named rather than spelled inline, as every other `ORBS_*` is: a rename
/// should not have to hunt bare literals.
const FIRE: &str = "ORBS_FIRE";

impl Bench {
    /// How a painter should draw a hearth that is `lit` or not.
    ///
    /// Zero phase and zero flare when the effect is off: `orbs_render`'s side
    /// is a pure function of this, so a frozen `Burn` is a frozen picture.
    #[must_use]
    pub fn burn(&self, lit: bool) -> Burn {
        if !self.enabled {
            return Burn {
                phase: 0.0,
                flare: 0.0,
                lit,
            };
        }
        Burn {
            phase: self.phase,
            flare: (self.flare / FLARE_SECS).clamp(0.0, 1.0),
            lit,
        }
    }

    /// How a painter should draw a mortar that is `working` or at rest.
    ///
    /// The same clock as [`Self::burn`] and the same reduce-motion switch. A
    /// mortar at rest is a *picture*, not an absence, so motion off leaves a
    /// stroke that does not swing rather than a fallback.
    #[must_use]
    pub fn grind(&self, working: bool, spent: bool) -> Grind {
        Grind {
            phase: if self.enabled { self.phase } else { 0.0 },
            working: working && self.enabled,
            // Zero with motion off snaps the bar to whole ticks: the sim's own
            // sampling rate rather than a smoothed guess at it.
            advance: if self.enabled { self.advance } else { 0.0 },
            load: if self.enabled {
                (self.load / LOAD_SECS).clamp(0.0, 1.0)
            } else {
                0.0
            },
            spent,
        }
    }

    /// How a painter should draw a bath that is `working` or standing.
    ///
    /// The same clock as [`Self::burn`] and [`Self::grind`], and the same
    /// reduce-motion switch.
    ///
    /// No pour, deliberately: the load is one timer on one edge
    /// (`observe_bowl`, which finds its instrument by [`Craft::Grinding`]), so
    /// sharing it would make charging the mortar pour the bath. The bath is a
    /// *picture* at rest, so a load is visible without an animation.
    #[must_use]
    pub const fn steep(
        &self,
        motion: Motion,
        spent: bool,
        leavings: bool,
        breaking: bool,
    ) -> Steep {
        Steep {
            phase: if self.enabled { self.phase } else { 0.0 },
            // Reduce-motion stills it, whichever motion it was in — a settling
            // bath is a *slower* animation, not an exemption from the switch.
            motion: if self.enabled {
                motion
            } else {
                Motion::Standing
            },
            // Zero with motion off snaps to whole ticks — the honest fallback,
            // exactly as [`Self::grind`] documents.
            advance: if self.enabled { self.advance } else { 0.0 },
            spent,
            leavings,
            breaking,
            // The painter's, not the bench's: `bath_meter` and its upward twin
            // each set this from the orientation they are.
            upward: false,
        }
    }

    /// How a painter should draw a flask that is `working` or standing.
    ///
    /// The same clock and switch as its siblings. A finished flask is
    /// *combined*, not still warm, so there is nothing for it to go on doing.
    #[must_use]
    pub const fn stir(&self, motion: Motion, spent: bool, leavings: bool) -> Stir {
        Stir {
            phase: if self.enabled { self.phase } else { 0.0 },
            motion: if self.enabled {
                motion
            } else {
                Motion::Standing
            },
            advance: if self.enabled { self.advance } else { 0.0 },
            spent,
            leavings,
        }
    }

    /// Note whether the athanor is alight, and catch the moment it lights.
    ///
    /// `None` means "no athanor on screen", not "out": the panel carries only
    /// the room the player is in, so walking out and back would flare.
    pub const fn observe_hearth(&mut self, lit: Option<bool>) {
        let Some(lit) = lit else {
            return;
        };
        if lit && !self.was_lit {
            self.flare = FLARE_SECS;
        }
        self.was_lit = lit;
    }

    /// Note whether the mortar holds anything, and catch the moment it fills.
    ///
    /// `None` means "no mortar on screen", not "empty" —
    /// [`Self::observe_hearth`]'s rule and reason.
    ///
    /// The edge is *empty to holding*. `Charged → Working → Ready` are not
    /// loads, and neither is `Ready → Fouled`: nothing went in.
    pub const fn observe_bowl(&mut self, charged: Option<bool>) {
        let Some(charged) = charged else {
            return;
        };
        if charged && !self.was_charged {
            self.load = LOAD_SECS;
        }
        self.was_charged = charged;
    }

    /// Carry the phase forward by `seconds`.
    ///
    /// Wrapped, not accumulated: an `f32` counting real seconds has lost its
    /// fraction by the third hour, and the fire would coarsen and stop with
    /// nothing on screen to say why. Separate from [`Bench::advance`] so it is
    /// testable without an `App`.
    ///
    /// Forward only, because `ORBS_FIRE_PHASE` is a number a person types and a
    /// negative one ran both decays in reverse — drawing an ignition and a load
    /// no state in the game could produce.
    pub fn tick(&mut self, seconds: f32) {
        let seconds = seconds.max(0.0);
        self.phase = (self.phase + seconds).rem_euclid(orbs_render::CYCLE_SECS);
        self.flare = (self.flare - seconds).max(0.0);
        self.load = (self.load - seconds).max(0.0);
    }

    /// Turn the effect on or off. See the module header.
    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Note how far through the world tick the frame is. See [`Self::advance`].
    pub const fn set_advance(&mut self, advance: f32) {
        self.advance = advance;
    }

    /// Put the pour part-way through, for a dump. See [`Self::set_flare`].
    pub fn set_load(&mut self, fraction: f32) {
        self.load = fraction.clamp(0.0, 1.0) * LOAD_SECS;
    }

    /// Put the flare part-way through, for a dump.
    ///
    /// A dump advances no `Time` and runs no systems, so it never observes an
    /// ignition and the flare would be permanently zero. `ORBS_FLARE` is its
    /// See-it line; see `shell::dump`.
    pub fn set_flare(&mut self, fraction: f32) {
        self.flare = fraction.clamp(0.0, 1.0) * FLARE_SECS;
    }

    /// Carry the bench forward one frame.
    ///
    /// `motion` is whether the frontend allows movement at all. `None` leaves
    /// whatever `ORBS_FIRE` said: a missing switch is not a switch set to off,
    /// and defaulting to off would blank the fire before a camera spawns.
    ///
    /// A method, not a system, for
    /// [`Panel::refresh`](super::glance::Panel::refresh)'s reason: every
    /// frontend advances these clocks in this order every frame.
    pub fn advance(
        &mut self,
        delta: f32,
        overstep: f32,
        motion: Option<bool>,
        panel: &super::glance::Panel,
    ) {
        if let Some(on) = motion {
            // The cached read, not a fresh one: this used to take the process
            // environment lock and allocate per frame for a constant.
            let permitted = self.permitted;
            self.set_enabled(on && permitted);
        }
        // Where the world is between steps, on the clock the sim runs on — a
        // self-counted copy would disagree the moment the clock hitched.
        self.set_advance(overstep);
        self.observe_hearth(hearth(panel));
        self.observe_bowl(bowl(panel));
        self.tick(delta);
    }
}

/// Whether the mortar in the current room holds anything, if there is one.
///
/// Found by [`Craft`], not by name: the sim says what an instrument does
/// (`tower::panel::craft_of`), so no frontend spells `"mortar_and_pestle"`.
fn bowl(panel: &super::glance::Panel) -> Option<bool> {
    panel
        .instruments
        .iter()
        .find(|instrument| instrument.craft == Craft::Grinding)
        .map(|instrument| instrument.state != State::Empty)
}

/// Whether the athanor in the current room is alight, if there is one.
///
/// Found by [`Craft`], exactly as [`bowl`] is. Searching by *state* worked only
/// while no other instrument reported `Burning` — a forge with a `Burning`
/// state would have flared the athanor when it caught.
///
/// `Charged` answers `None`: an athanor loaded but unlit is neither alight nor
/// out, and [`Bench::observe_hearth`] leaves the previous answer standing.
fn hearth(panel: &super::glance::Panel) -> Option<bool> {
    panel
        .instruments
        .iter()
        .find(|instrument| instrument.craft == Craft::Heating)
        .and_then(|instrument| match instrument.state {
            State::Burning => Some(true),
            State::Banked | State::Cold => Some(false),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_phase_wraps_instead_of_growing_without_bound() {
        // A session lasts hours; an `f32` counting seconds does not. The failure
        // is silent and gradual, which is the kind worth a test.
        let mut bench = Bench::default();
        for _ in 0..10_000 {
            bench.tick(1.0 / 60.0);
        }
        assert!(
            bench.phase < orbs_render::CYCLE_SECS,
            "{} escaped the cycle",
            bench.phase,
        );
        // ...and it is still moving, rather than pinned at a bound.
        let before = bench.burn(true).phase;
        bench.tick(1.0 / 60.0);
        assert_ne!(before, bench.burn(true).phase);
    }

    #[test]
    fn kindling_flares_and_the_flare_burns_out() {
        // Both halves are the test: it has to start on the edge `kindle` lands
        // on, and it has to stop, or the athanor sits at peak brightness.
        let mut bench = Bench::default();
        bench.observe_hearth(Some(false));
        assert_eq!(bench.burn(true).flare, 0.0, "a cold hearth was flaring");

        bench.observe_hearth(Some(true));
        assert_eq!(bench.burn(true).flare, 1.0, "kindle did not catch");

        bench.tick(FLARE_SECS / 2.0);
        let half = bench.burn(true).flare;
        assert!(
            half > 0.0 && half < 1.0,
            "the flare jumped rather than decayed: {half}"
        );

        bench.tick(FLARE_SECS);
        assert_eq!(bench.burn(true).flare, 0.0, "the flare never went out");

        // Still burning next frame is not a fresh ignition.
        bench.observe_hearth(Some(true));
        assert_eq!(bench.burn(true).flare, 0.0, "a steady bench re-flared");
    }

    #[test]
    fn winding_the_clock_backwards_kindles_nothing() {
        // `tick` decays by subtraction, so a negative `ORBS_FIRE_PHASE` ran it
        // in reverse and manufactured a flare and a pour out of nothing — a
        // dump drawing a screen the game cannot reach looks like evidence.
        let mut bench = Bench::default();
        bench.tick(-5.0);
        assert_eq!(bench.burn(true).flare, 0.0, "running back lit the hearth");
        assert_eq!(bench.grind(false, false).load, 0.0, "...and poured a bowl");
        assert_eq!(bench.burn(true).phase, 0.0, "...and moved the clock");
    }

    #[test]
    fn walking_out_of_the_laboratory_does_not_relight_the_fire() {
        // `None` is "no athanor on screen", not "out": without this, every trip
        // back to the laboratory would set off a flare.
        let mut bench = Bench::default();
        bench.observe_hearth(Some(true));
        bench.tick(FLARE_SECS * 2.0);
        assert_eq!(bench.burn(true).flare, 0.0);

        bench.observe_hearth(None); // walked out
        bench.observe_hearth(None);
        bench.observe_hearth(Some(true)); // walked back, still burning
        assert_eq!(
            bench.burn(true).flare,
            0.0,
            "coming home re-lit a bench that never went out",
        );
    }

    #[test]
    fn turning_it_off_freezes_the_phase_and_says_so() {
        // Both halves matter: a disabled bench must stop moving *and* fall back
        // to the plain meter, and the second is what `is_lit` carries.
        let mut bench = Bench::default();
        bench.tick(3.0);
        assert!(bench.grind(true, false).working);
        assert_ne!(bench.burn(true).phase, 0.0);

        bench.set_enabled(false);
        assert!(!bench.grind(true, false).working);
        assert_eq!(bench.burn(true).phase, 0.0);

        // Time still passes; it simply does not reach the screen. The flare is
        // frozen with it — a `kindle` during reduce-motion must not flash.
        bench.observe_hearth(Some(false));
        bench.observe_hearth(Some(true));
        bench.tick(5.0);
        assert_eq!(bench.burn(true).phase, 0.0, "a disabled bench moved");
        assert_eq!(bench.burn(true).flare, 0.0, "a disabled bench flared");
    }
}
