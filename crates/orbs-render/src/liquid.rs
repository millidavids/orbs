//! How a vessel of liquid moves.
//!
//! Two instruments hold liquid — the balneum mariae and the flask — and they
//! must agree about what motion *means*, or the panel says two different things
//! with the same picture. So the vocabulary lives here and both draw from it.
//!
//! Bubbles mean heat and nothing else does. [`Motion::Bubbling`] is the only
//! state whose pattern travels, and it belongs to a bath over a lit athanor: the
//! flask is never heated so never bubbles, and a bath whose fire has gone out
//! stops at once, which is the panel saying the athanor died without a word.
//! §10.1 makes *"light it once, get both heated stages done inside that window"*
//! the central timing decision — this is that window, visible.
//!
//! Everything else shifts in place. [`Motion::Stirring`] and
//! [`Motion::Drifting`] are one stationary shimmer at two rates: liquid moving
//! but not driven from below. A finished vessel still shifts, because liquid
//! does.
//!
//! A stationary shimmer takes the staggered clock and a travelling one must not.
//! `pulse::tick_of` folds each cell's offset into the tick, which stops a strip
//! turning over as one field; a rising pattern cannot use it, because cell `s+1`
//! would sit half a tick behind cell `s` and the rise would wash out into churn.
//! It does not need to either, since a translation moves brightness along the
//! strip.
//!
//! Tempo is the signature:
//!
//! | Motion | Ticks per step | Flashes | Reads as |
//! |---|---|---|---|
//! | [`Motion::Standing`] | — | 0 | not started |
//! | [`Motion::Drifting`] | [`DRIFT_EVERY`] | 0.33 Hz | done, or gone cold |
//! | [`Motion::Stirring`] | [`STIR_EVERY`] | 0.6 Hz | being worked |
//! | [`Motion::Bubbling`] | [`RISE_EVERY`] | 1 Hz | being worked **over a fire** |
//!
//! Every one is under §14's 3 Hz floor, so unlike the athanor's fire no part of
//! this needs a photosensitivity exemption.

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
/// It must divide exactly, or the last rise of a cycle is a fraction of a cell
/// and the wrap tears. Checked rather than trusted, as
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
/// Four states, and the top one is about the athanor: bubbles mean heat, so
/// `Bubbling` against `Stirring` is whether the fire is lit — the laboratory
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
    /// Bubbles rising — a vessel being worked *over a lit athanor*.
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
        // Travelling, so the shared clock and a wrapped drift coordinate.
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
        // Stationary, so the staggered clock, which stops the whole vessel
        // turning over on one instant.
        Motion::Stirring => shift(lane, step, phase, STIR_EVERY, false),
        // The same shimmer, slower and shallower. Never reaches `Rolling`:
        // nothing about a vessel standing full is vigorous.
        Motion::Drifting => shift(lane, step, phase, DRIFT_EVERY, true),
    }
}

/// How many lengths of climb a bubble can be given.
///
/// Three, so one bubble in three is a short one, and the tallest gets
/// [`CARRY`] cells. What varies the *look* is that they differ from each other;
/// how far any of them actually gets is [`STAGE`].
const STAGES: u16 = 3;

/// How many cells one of those lengths is worth.
///
/// Two, where it was one: every bubble rises twice as far, so two cells, four,
/// or six. At one the whole vocabulary happened inside three cells of the face
/// and read as a fizz at the surface rather than as something leaving it.
const STAGE: u16 = 2;

/// How far above the surface a bubble gets before it is gone.
///
/// Six cells, which is two seconds at [`RISE_EVERY`] — long enough to read as
/// *leaving* and short enough that the air never fills up. A bubble is a thing
/// that pops, and one that climbed the whole bar would be a plume, which is the
/// athanor's picture and means something else.
pub const CARRY: u16 = STAGES * STAGE;

/// One air cell in this many carries a bubble, before lifetime thins them.
///
/// Tuned by looking, at `screens`. Sparse enough that every mark is a thing
/// rather than a texture, and dense enough to read as a boil: at five, a
/// two-lane column showed a bubble about a third of the time and looked like a
/// stray artefact rather than an instrument working. With a life of one to
/// [`CARRY`] cells, this puts a little over one mark in the air at a time,
/// thinning with height.
const ESCAPE_ODDS: u32 = 3;

/// A bubble that has broken the surface, `ahead` cells above it.
///
/// [`roil`] already has bubbles going out at the surface, which is the balneum's
/// whole motion. The alembic adds that some get through — distilling is a harder
/// boil than a digestion. Gated on `breaking` and [`Motion::Bubbling`], so it
/// means what the rest of the vocabulary means: over a lit athanor.
///
/// It rides the liquid's clock, because an escaped bubble is the bubble that was
/// rising a moment ago: `rising` gives a coordinate constant for one bubble
/// while the tick and its height climb together, as `fire::spark` does. Its
/// lifetime is read off that travelling coordinate rather than a stationary one,
/// which would be a fixed ceiling every bubble died at.
///
/// `ahead`, not the absolute cell, so the surface can never overtake it. The
/// vessel fills while this runs, at a rate depending on the recipe and on a bar
/// that grows with the window, so a bubble anchored in absolute space is
/// swallowed by the liquid it just left whenever `bar / ticks` beats one cell
/// per [`RISE_EVERY`] — which at the shipped durations it did. No tick count
/// fixes that at every window size, so a bubble is defined as sitting so many
/// cells above the face.
///
/// Returns the [`Roil`] rather than a style: what colour a piece of liquid is
/// belongs with the liquid, and a bubble is liquid that has left.
pub(crate) fn escaping(lane: u16, ahead: u16, phase: f32) -> Option<(char, Roil)> {
    // The face itself is never a bubble: it is the cell the level is read off,
    // and a mark on it is the one place picture and value disagree.
    if ahead == 0 {
        return None;
    }
    let risen = shared_tick(phase) / RISE_EVERY;
    let drift = rising(ahead, risen, RISE_SPAN);
    // Divided out, not shifted. Presence and lifetime come from one hash and
    // must be taken from *independent* parts of it, as the fire's sparks do
    // (`seed / SPARK_ODDS % 7`). This copied the shape without the division:
    // `drift_noise` is eight bits wide, so `seed >> 8` was always zero and every
    // bubble lived exactly one cell — a blink one row above the face that never
    // rises, visible in `screens` and in nothing else.
    let seed = drift_noise(lane, drift);
    if !seed.is_multiple_of(ESCAPE_ODDS) {
        return None;
    }
    // How high this one gets, its own for life: one of [`STAGES`] lengths, each
    // [`STAGE`] cells — so two, four or six.
    let stage = 1 + u16::try_from((seed / ESCAPE_ODDS) % u32::from(STAGES)).unwrap_or(0);
    let life = stage * STAGE;
    if ahead > life {
        return None;
    }
    // Thinning as it goes. `°` is a ring with something still in it, `·` is the
    // last of it before it is gone — the fire's own two smallest marks, in the
    // The liquid's colours rather than the fire's, which keeps a bubble and a
    // spark apart on a screen that holds both.
    Some(if ahead * 2 <= life {
        ('°', Roil::Stirred)
    } else {
        ('·', Roil::Still)
    })
}

/// A shimmer that stays where it is, at `every` ticks a step.
///
/// Density is ordered as well as tempo, and has to be: a sweep of the cycle
/// counts how many cells move at once, so a `Drifting` vessel denser than a
/// `Bubbling` one reads as busier however slowly it changes. The first version
/// mapped two of four shades to `Stirred` for the calm case and measured 16,500
/// moving cells against bubbling's 10,400 — calmer by the clock, busier to the
/// eye.
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
        // Only ticks where the bar has something to say. A uniformly `Still` bar
        // satisfies `now[s] == next[s+1]` trivially and `Drifting` is sparse
        // enough that most ticks are exactly that, so a naive count scored it as
        // translating on 7 ticks while it stood still on all of them.
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
