//! The clock §10.1's instruments animate on.
//!
//! The frontend half of [`fire`](orbs_render::Burn) and
//! [`grind`](orbs_render::Grind) and whatever the remaining three instruments
//! get — those decide what a cell *is*, this decides *when*, which is the
//! division `tween` and `reveal` already draw. Driven from `Time`, never from
//! the tick: the sim must not be able to observe it, or replay would depend on
//! how long a frame took.
//!
//! **One clock, not one per instrument.** They share a phase because they share
//! `orbs_render`'s tick grid, and that grid is where the photosensitivity
//! guarantee lives — five clocks would be five places to get the one number with
//! a safety argument attached wrong. What differs per instrument is the *view*
//! this hands out: [`Bench::burn`] for a hearth, [`Bench::grind`] for a mortar.
//!
//! # It is off when the tube is off
//!
//! §14 requires motion be disableable — `court_wizard` ships a health warning for
//! motion sickness and this inherits the obligation — and `ROADMAP.md` records
//! that after the boot strike was cut, **nothing in the game flashes**. A
//! shimmering meter on the permanent fixture of the laboratory screen puts that
//! back, so it needs a switch a player can actually reach.
//!
//! It rides the frontend's own motion switch — the Bevy build's `CrtSettings`,
//! passed to [`Bench::advance`] rather than queried here — read explicitly and
//! never derived. `crt::settings` documents at length why that distinction is
//! load-bearing: `enabled` was once inferred from "are all the effects zero", so
//! the moment anything drove `flash` the barrel, scanlines and grille came back
//! for a player who had turned them off. F3 cycles to `OFF`; that is the switch,
//! and it stops the fire too.
//!
//! `ORBS_FIRE=0` is for scripted runs, not for players. The per-effect toggle
//! belongs to `ROADMAP.md`'s Phase 13 settings item, beside persisted CRT-off and
//! reduce-motion, rather than to a debug affordance nobody maintains.

use bevy_ecs::prelude::Resource;
use orbs_render::{Burn, Grind, Motion, Steep, Stir};
use orbs_sim::tower::{Craft, State};

/// How long a fire takes to catch, in seconds.
///
/// **One world tick** (DESIGN.md §5.0 makes a tick one real second). The bar
/// redraws at frame rate and the world advances at 1 Hz, so a tick is the
/// longest an animation can run and still be over before anything it describes
/// can change — the flare finishes exactly as the fuel it is burning ticks down
/// for the first time.
///
/// It is also about as slow as it can be without reading as the panel lagging.
/// The front has to visibly climb, which needs long enough to see; much past a
/// tick and `kindle` stops feeling instant.
const FLARE_SECS: f32 = 1.0;

/// How long a bowl takes to fill, in seconds.
///
/// **One world tick**, for the same reason the flare is: the bar redraws at
/// frame rate and the world advances at 1 Hz, so a tick is the longest an
/// animation can run and still finish before anything it describes can change.
/// A reagent that went in is in; the pour is over before the next thing happens
/// to it.
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
    /// **Read from the fixed scheduler rather than counted here.** `Bench`
    /// already accumulates wall-clock for [`Self::phase`], and taking the
    /// fraction of that would be a second copy of the tick's position that
    /// drifts from the real one the moment the clock hitches or `Time<Fixed>`
    /// catches up several steps at once — and drift shows as a bar that reaches
    /// the next cell just before or after the tick it is meant to land on.
    ///
    /// The sim ticks in `FixedUpdate` (`sim::plugin`), so this *is* that tick's
    /// position, by construction rather than by agreement.
    advance: f32,
    /// Whether the effect runs at all. See the module header.
    enabled: bool,
    /// Whether `ORBS_FIRE` permits any motion, read **once**.
    ///
    /// Separate from [`Self::enabled`], which the tube's own switch drives every
    /// frame. Keeping the environment's answer here is what lets `advance`
    /// combine the two without asking the operating system again: an env var
    /// cannot change during a run, and reading it per frame takes the process's
    /// environment lock and allocates a `String` for a constant.
    permitted: bool,
}

impl Default for Bench {
    fn default() -> Self {
        Self {
            phase: 0.0,
            flare: 0.0,
            // **Starts lit.** A tower that opens with the athanor already
            // burning — a save, or a dump that kindles before it draws — must
            // not flare on its first frame: the fire did not just catch, it was
            // already going. The edge this watches for is off-to-on, so seeding
            // it *on* means an athanor that is already alight never sets it off.
            was_lit: true,
            load: 0.0,
            // **Starts charged**, for the same reason `was_lit` starts lit: a
            // tower that opens with something already in the mortar must not
            // pour it in again on the first frame.
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
/// Named rather than spelled inline, following `shell::dump`'s convention for
/// every other `ORBS_*` — a rename should not have to find bare string literals
/// in a file away from the one documenting them.
const FIRE: &str = "ORBS_FIRE";

impl Bench {
    /// How a painter should draw a hearth that is `lit` or not.
    ///
    /// Zero phase and zero flare when the effect is off, which is what makes a
    /// disabled fire *still* — `orbs_render`'s side is a pure function of this,
    /// so a frozen `Burn` is a frozen picture rather than a special case anyone
    /// has to remember.
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
    /// mortar at rest is a *picture*, not an absence — a loaded bowl and a
    /// finished one — so unlike the fire there is nothing to fall back to when
    /// motion is off, only a stroke that does not swing.
    #[must_use]
    pub fn grind(&self, working: bool, spent: bool) -> Grind {
        Grind {
            phase: if self.enabled { self.phase } else { 0.0 },
            working: working && self.enabled,
            // **Zero with motion off snaps the bar to whole ticks**, which is
            // the honest fallback: a player who turned animation off gets the
            // sim's own sampling rate rather than a smoothed guess at it.
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
    /// **No pour, and that is a decision rather than an omission.** The mortar's
    /// own load
    /// is one timer driven by one edge (`observe_bowl`, which finds its
    /// instrument by [`Craft::Grinding`]), so handing it to the bath as well
    /// would mean charging the mortar poured the bath. A second edge would need
    /// its own field and its own `was_*`, and the bath does not need one: unlike
    /// the mortar it is a *picture* at rest, a shallow layer of liquid standing
    /// still, so a load is already visible without an animation to announce it.
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
            // **The painter's, not the bench's.** `bath_meter` and its upward
            // twin each set this from the orientation they *are*, so a caller
            // cannot get it wrong — and this side of the boundary has no idea
            // which way the pane runs anyway.
            upward: false,
        }
    }

    /// How a painter should draw a flask that is `working` or standing.
    ///
    /// The same clock and the same switch as its two siblings. It carries no
    /// motion enum of its own: a finished flask is *combined*, not still warm,
    /// so there is nothing for it to go on doing.
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
    /// **`None` means "no athanor on screen", which is not the same as "out".**
    /// The panel only carries the room the player is standing in, so walking out
    /// of the laboratory and back would otherwise read as the fire going out and
    /// re-lighting — a flare every time you came home.
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
    /// **`None` means "no mortar on screen"**, not "empty" — the same rule
    /// [`Self::observe_hearth`] follows, and for the same reason: the panel only
    /// carries the room the player is standing in, so without it walking back
    /// into the laboratory would pour the bowl again every time.
    ///
    /// The edge is *empty to holding*. `Charged → Working → Ready` are not
    /// loads, and neither is `Ready → Fouled` when the product is taken and the
    /// husks stay: nothing went in, something came out.
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
    /// **Wrapped, not accumulated.** `Time` runs for as long as the session
    /// does, and an `f32` counting real seconds has lost its fraction by the
    /// third hour — the fire would visibly coarsen and then stop, with nothing
    /// on screen to say why. `rem_euclid` keeps it inside the one cycle
    /// `orbs_render` defines, where the resolution is constant forever.
    ///
    /// Separate from [`Bench::advance`] so it is testable without an `App`.
    ///
    /// **Forward only.** `Time::delta_secs` never goes backwards, but
    /// `ORBS_FIRE_PHASE` is a number a person types, and a negative one ran the
    /// two decays in reverse: `flare - (-5.0)` is *five seconds of flare* on a
    /// hearth nobody kindled, and the same again for a pour into a bowl nothing
    /// entered. `ORBS_FIRE_PHASE=-1` therefore drew an ignition and a load that
    /// no state in the game could produce — a dump showing a screen the game
    /// cannot reach, which is the one thing this tool must never do.
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
    /// **A dump advances no `Time` and runs no systems**, so it never observes
    /// an ignition and the flare would be permanently zero — the one part of
    /// this effect with no See-it line at all. `ORBS_FLARE` is what gives it
    /// one; see `shell::dump`.
    pub fn set_flare(&mut self, fraction: f32) {
        self.flare = fraction.clamp(0.0, 1.0) * FLARE_SECS;
    }

    /// Carry the bench forward one frame.
    ///
    /// `motion` is whether the frontend is *allowing* movement at all — a CRT
    /// switch here, a `--no-motion` flag or a slow terminal elsewhere. `None`
    /// leaves whatever `ORBS_FIRE` said, because a missing switch is not the
    /// same as a switch set to off: defaulting to "off" would make the fire
    /// invisible for the frames before a camera spawns rather than merely unlit.
    ///
    /// **A method, not a system**, for the same reason
    /// [`Panel::refresh`](super::glance::Panel::refresh) is:
    /// every frontend advances these four clocks in this order every frame, and
    /// only one of them has a `Query` to find the switch with.
    pub fn advance(
        &mut self,
        delta: f32,
        overstep: f32,
        motion: Option<bool>,
        panel: &super::glance::Panel,
    ) {
        if let Some(on) = motion {
            // **`permitted` is the cached read, not a fresh one.**
            // `Bench::default` documents reading the environment once "rather
            // than polling it sixty times a second forever" — and this used to
            // do exactly that polling, taking the process environment lock and
            // allocating a `String` per frame for a value that cannot change
            // during a run.
            let permitted = self.permitted;
            self.set_enabled(on && permitted);
        }
        // Where the world is between steps. **The same clock the sim runs on**,
        // so it cannot disagree with the sim about when a tick lands — which a
        // second, self-counted copy would the moment the clock hitched.
        self.set_advance(overstep);
        self.observe_hearth(hearth(panel));
        self.observe_bowl(bowl(panel));
        self.tick(delta);
    }
}

/// Whether the mortar in the current room holds anything, if there is one.
///
/// **Found by [`Craft`], not by name** — the sim says what an instrument does
/// (`tower::panel::craft_of`) so a frontend never compares against the literal
/// `"mortar_and_pestle"`.
fn bowl(panel: &super::glance::Panel) -> Option<bool> {
    panel
        .instruments
        .iter()
        .find(|instrument| instrument.craft == Craft::Grinding)
        .map(|instrument| instrument.state != State::Empty)
}

/// Whether the athanor in the current room is alight, if there is one.
///
/// **Found by [`Craft`], exactly as [`bowl`] is.** This used to search by *state*
/// — the first instrument reporting `Burning`, `Banked` or `Cold` — on the
/// argument that no other instrument reports those three. That is true today and
/// is a property of a table in another crate: the moment §10's Phase 11a gives a
/// forge or a kiln a `Burning` state, this would find whichever the sim happened
/// to list first and flare the athanor when the forge caught. Asking what an
/// instrument *does* cannot go wrong that way, and it is the rule 2 answer
/// anyway.
///
/// `Charged` still answers `None`: an athanor loaded but unlit is neither alight
/// nor out, and [`Bench::observe_hearth`] leaves the previous answer standing —
/// which is what stops the state before `kindle` reading as the fire going out.
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
        // The activation effect. Both halves are the test: it has to bench on the
        // *edge* — the tick `kindle` lands on — and it has to stop, or the
        // athanor sits at peak brightness for the rest of the session.
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
        // `ORBS_FIRE_PHASE` is a number a person types, and `tick` decays both
        // the flare and the pour by subtraction — so a negative one ran them in
        // reverse and manufactured a full flare *and* a full pour out of a
        // hearth nobody kindled and a bowl nothing entered. A dump drawing a
        // screen the game cannot reach is the one failure this tool must not
        // have, because it looks exactly like evidence.
        let mut bench = Bench::default();
        bench.tick(-5.0);
        assert_eq!(bench.burn(true).flare, 0.0, "running back lit the hearth");
        assert_eq!(bench.grind(false, false).load, 0.0, "...and poured a bowl");
        assert_eq!(bench.burn(true).phase, 0.0, "...and moved the clock");
    }

    #[test]
    fn walking_out_of_the_laboratory_does_not_relight_the_fire() {
        // `None` is "no athanor on screen", which is not "out". The panel only
        // carries the room you are standing in, so without this every trip back
        // to the laboratory would set off a flare.
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
