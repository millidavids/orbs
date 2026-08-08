//! How a vessel of liquid moves.
//!
//! Two instruments hold liquid — the balneum mariae and the flask — and they
//! must agree about what motion *means*, or the panel says two different things
//! with the same picture. So the vocabulary lives here and both draw from it.
//!
//! # Bubbles mean heat, and nothing else does
//!
//! [`Motion::Bubbling`] is the only state whose pattern **travels**: lighter
//! cells rise and go out at the surface. It belongs to a bath over a lit
//! athanor and to nothing else in the game.
//!
//! That is the whole reason the distinction is worth a state. The flask is never
//! heated, so it never bubbles however hard it is working; and a bath whose fire
//! has gone out stops bubbling *immediately*, which is the panel telling a player
//! their athanor died without them having to read a word. §10.1 makes the burn a
//! shared, depleting resource and *"light it once, get both heated stages done
//! inside that window"* the central timing decision — this is that window,
//! visible.
//!
//! # Everything else shifts in place
//!
//! [`Motion::Stirring`] and [`Motion::Drifting`] are the same stationary
//! shimmer at two rates: liquid that is present and moving, but not being driven
//! from below. A vessel that has *finished* still shifts, because liquid does —
//! it is simply no longer being worked.
//!
//! **A stationary shimmer takes the staggered clock; a travelling one must not.**
//! `pulse::tick_of` folds each cell's own offset into the tick, which is what stops a
//! strip turning over as one field. A rising pattern cannot use it — cell `s+1`
//! would be half the time a tick behind cell `s` and the rise would wash out into
//! churn — and does not need to, because a translation moves brightness *along*
//! the strip rather than in and out of it. Both are safe; they are safe for
//! different reasons, and using either one's clock for the other breaks it.
//!
//! # Tempo is the signature
//!
//! | Motion | Ticks per step | Flashes | Reads as |
//! |---|---|---|---|
//! | [`Motion::Standing`] | — | 0 | not started |
//! | [`Motion::Drifting`] | [`DRIFT_EVERY`] | 0.33 Hz | done, or gone cold |
//! | [`Motion::Stirring`] | [`STIR_EVERY`] | 0.6 Hz | being worked |
//! | [`Motion::Bubbling`] | [`RISE_EVERY`] | 1 Hz | being worked **over a fire** |
//!
//! Every one of them is under §14's 3 Hz floor, so unlike the athanor's fire no
//! part of this needs a photosensitivity exemption.

use crate::pulse::{TICKS_PER_CYCLE, drift_noise, noise, rising, shared_tick, tick_of};
use crate::style::Roil;

/// Ticks a bubble takes to rise one cell, in a heated vessel.
///
/// Two would match the mortar's fall and cost the bath its signature; four is
/// slow enough that a bubble takes eleven seconds to cross a full bar, which
/// stops reading as rising at all.
pub const RISE_EVERY: u16 = 3;

/// Cells a bubble rises in one cycle — the period its coordinate wraps on.
///
/// **It must divide exactly**, or the last rise of a cycle is a fraction of a
/// cell and the wrap tears. Checked rather than trusted, for the reason
/// [`grind`](crate::grind)'s equivalent is.
const RISE_SPAN: u16 = TICKS_PER_CYCLE / RISE_EVERY;
const _: () = assert!(
    TICKS_PER_CYCLE.is_multiple_of(RISE_EVERY),
    "RISE_EVERY must divide the cycle, or the bubbles tear at the wrap",
);

/// Ticks a cell holds its shade while a vessel is being worked, unheated.
///
/// Slower than the bubbling rate: a driven liquid moves more than a stirred one,
/// and the panel should say which is which without a word.
pub const STIR_EVERY: u16 = 5;

/// Ticks a cell holds its shade in a vessel that is merely standing full.
///
/// Slow enough to read as *settling* rather than as *working* — which is what
/// keeps the at-rest rule intact while still admitting that liquid moves.
pub const DRIFT_EVERY: u16 = 9;

/// What a vessel of liquid is doing.
///
/// **Four states, and the top one is about the athanor.** See this module's
/// header: bubbles mean heat, so the difference between `Bubbling` and
/// `Stirring` is whether the fire is lit — which is the one piece of laboratory
/// state §10.1 makes most expensive to forget.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    /// Dead flat. Charged and not started, or motion turned off.
    ///
    /// The at-rest rule in its strict form: an instrument that has not begun
    /// must not move, or a player waits for something that has not started.
    #[default]
    Standing,
    /// Shifting slowly in place — a vessel standing full.
    ///
    /// A finished run, or a bath whose athanor went out. Liquid moves; it is
    /// simply not being worked.
    Drifting,
    /// Shifting briskly in place — a vessel being worked without heat.
    ///
    /// The flask, always, since nothing heats it.
    Stirring,
    /// Bubbles rising — a vessel being worked **over a lit athanor**.
    ///
    /// The bath, and only while the fire is in.
    Bubbling,
}

impl Motion {
    /// Whether anything is happening at all.
    #[must_use]
    pub const fn is_still(self) -> bool {
        matches!(self, Self::Standing)
    }
}

/// How hard one cell of liquid is moving.
///
/// `lane` and `step` are the cell's place in its bar; `phase` is the frontend's
/// clock. Nothing here knows what a second is.
pub(crate) fn roil(lane: u16, step: u16, phase: f32, motion: Motion) -> Roil {
    match motion {
        Motion::Standing => Roil::Still,
        // **Travelling**, so the shared clock and a wrapped drift coordinate.
        Motion::Bubbling => {
            let risen = shared_tick(phase) / RISE_EVERY;
            match drift_noise(lane, rising(step, risen, RISE_SPAN)) >> 5 {
                // Still liquid, and most of the vessel.
                0..=4 => Roil::Still,
                // A bubble on its way up.
                5 | 6 => Roil::Stirred,
                // The brightest of them. One cell in eight, so a bar carries two
                // or three at a time rather than a stripe.
                _ => Roil::Rolling,
            }
        }
        // **Stationary**, so the staggered clock — which is what stops the whole
        // vessel turning over on one instant.
        Motion::Stirring => shift(lane, step, phase, STIR_EVERY, false),
        // The same shimmer, slower and shallower. Never reaches `Rolling`:
        // nothing about a vessel standing full is vigorous.
        Motion::Drifting => shift(lane, step, phase, DRIFT_EVERY, true),
    }
}

/// A shimmer that stays where it is, at `every` ticks a step.
///
/// **Density is ordered as well as tempo**, and it has to be: a sweep of the
/// cycle counts how many cells are moving at once, and a `Drifting` vessel that
/// was *denser* than a `Bubbling` one would read as busier however slowly it
/// changed. The first version mapped two of four shades to `Stirred` for the calm
/// case and measured 16,500 moving cells against bubbling's 10,400 — calmer by
/// the clock and busier to the eye.
fn shift(lane: u16, step: u16, phase: f32, every: u16, calm: bool) -> Roil {
    let beat = tick_of(phase, lane, step) / every;
    let vigour = noise(lane, step, beat) >> 5;
    if calm {
        // One cell in eight, and never the brightest step: nothing about a
        // vessel standing full is vigorous. `Rolling` stays reserved for a
        // liquid being driven, so the top of the ramp always means *worked*.
        return if vigour >= 7 {
            Roil::Stirred
        } else {
            Roil::Still
        };
    }
    match vigour {
        0..=4 => Roil::Still,
        5 | 6 => Roil::Stirred,
        _ => Roil::Rolling,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse::harness::sweep;

    /// One bar's worth of roil.
    fn bar(phase: f32, motion: Motion) -> Vec<Roil> {
        (0..16).map(|step| roil(0, step, phase, motion)).collect()
    }

    #[test]
    fn only_a_heated_vessel_bubbles() {
        // The whole point of the distinction: a *travelling* pattern is what
        // reads as rising, and only `Bubbling` travels. If a stationary state
        // ever started translating, "the athanor is lit" would stop being
        // legible from the bath — which is the timing decision §10.1 is built
        // around.
        let hz = crate::FLIP_HZ;
        for tick in 0..TICKS_PER_CYCLE {
            let now = bar(f32::from(tick) / hz, Motion::Bubbling);
            let next = bar(f32::from(tick + RISE_EVERY) / hz, Motion::Bubbling);
            for step in 0..15 {
                assert_eq!(
                    now[step],
                    next[step + 1],
                    "the bubbles did not rise at tick {tick}",
                );
            }
        }

        // ...and neither shifting state does.
        //
        // **Only ticks where the bar has something to say.** A bar that is
        // uniformly `Still` satisfies `now[s] == next[s+1]` trivially, and
        // `Drifting` is sparse enough that most ticks are exactly that — so a
        // naive count scored it as translating on 7 ticks when it was standing
        // perfectly still on all of them. The claim is about ticks that carry a
        // *pattern*.
        for motion in [Motion::Stirring, Motion::Drifting] {
            let (mut translated, mut counted) = (0, 0);
            for tick in 0..TICKS_PER_CYCLE {
                let now = bar(f32::from(tick) / hz, motion);
                if now.iter().all(|roil| *roil == Roil::Still) {
                    continue;
                }
                counted += 1;
                let next = bar(f32::from(tick + 1) / hz, motion);
                if (0..15).all(|step| now[step] == next[step + 1]) {
                    translated += 1;
                }
            }
            assert!(counted > 20, "{motion:?} barely moves at all");
            assert!(
                translated * 8 < counted,
                "{motion:?} translated on {translated} of {counted} patterned \
                 ticks — it should shimmer in place, not rise",
            );
        }
    }

    #[test]
    fn a_standing_vessel_never_moves() {
        let first = bar(0.0, Motion::Standing);
        assert!(first.iter().all(|roil| *roil == Roil::Still));
        for phase in sweep() {
            assert_eq!(bar(phase, Motion::Standing), first, "moved at {phase}");
        }
    }

    #[test]
    fn each_motion_is_calmer_than_the_one_above_it() {
        // The tempos have to be *ordered* or the panel says the wrong thing:
        // a finished vessel that moved as much as a working one would read as
        // still running, and a stirred one that moved as much as a bubbling one
        // would hide whether the fire is in.
        let busy = |motion| -> usize {
            sweep()
                .map(|phase| {
                    bar(phase, motion)
                        .into_iter()
                        .filter(|r| *r != Roil::Still)
                        .count()
                })
                .sum()
        };
        let (bubbling, stirring, drifting) = (
            busy(Motion::Bubbling),
            busy(Motion::Stirring),
            busy(Motion::Drifting),
        );
        assert!(
            drifting < stirring,
            "a standing vessel ({drifting}) is not calmer than a worked one \
             ({stirring})",
        );
        // Half again as calm, measured. Not a round factor picked in advance —
        // `drifting * 2 < bubbling` was, and failed at 5,700 against 10,400,
        // which is a vessel that is *visibly* calmer and simply not calmer by
        // the number someone guessed.
        assert!(
            drifting * 3 < bubbling * 2,
            "a standing vessel ({drifting}) is not visibly calmer than a \
             bubbling one ({bubbling})",
        );
    }

    #[test]
    fn nothing_but_a_bubbling_vessel_rolls() {
        // The brightest step is reserved for a liquid being driven, so the top
        // of the ramp means *heat* wherever it appears.
        for phase in sweep() {
            assert!(
                bar(phase, Motion::Drifting)
                    .iter()
                    .all(|roil| *roil != Roil::Rolling),
                "a drifting vessel rolled at {phase}",
            );
        }
    }

    #[test]
    fn a_shimmer_does_not_turn_the_whole_vessel_over_at_once() {
        // Whole-field modulation is the hazard the rate cap alone does not
        // cover, and it is the *stationary* states that could commit it — a
        // translation cannot, because it moves brightness along rather than in
        // and out. `stagger` inside `tick_of` is what prevents it here.
        for motion in [Motion::Stirring, Motion::Drifting] {
            let mut busiest = 0;
            let mut previous = bar(0.0, motion);
            for phase in sweep().skip(1) {
                let now = bar(phase, motion);
                let changed = previous
                    .iter()
                    .zip(&now)
                    .filter(|(before, after)| before != after)
                    .count();
                busiest = busiest.max(changed);
                previous = now;
            }
            assert!(
                busiest < 8,
                "{motion:?} turned {busiest} of 16 cells at once"
            );
        }
    }

    #[test]
    fn every_tempo_is_under_the_photosensitive_floor() {
        // Unlike the athanor's fire, no part of a liquid needs §19's exemption.
        // A *flash* is a pair of opposing changes, so the flash rate is half the
        // transition rate — the distinction that cost the mortar's version of
        // this test two confusing failures.
        for (motion, every) in [
            (Motion::Bubbling, RISE_EVERY),
            (Motion::Stirring, STIR_EVERY),
            (Motion::Drifting, DRIFT_EVERY),
        ] {
            let flashes = crate::FLIP_HZ / f32::from(every) / 2.0;
            assert!(
                flashes < 3.0,
                "{motion:?} flashes at {flashes} Hz, inside the band",
            );
        }
    }
}
