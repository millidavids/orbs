//! The clock and the noise every animated instrument shares.
//!
//! §10.1 puts five instruments on one panel and each gets its own picture — a
//! fire, a grind, and three more to come. What they must *not* each get is their
//! own idea of how fast a cell may change, because that is the one number with a
//! safety argument attached (DESIGN.md §19) and five copies of it is five places
//! to get it wrong.
//!
//! So the tick grid lives here and the pictures live in [`fire`](crate::fire),
//! [`grind`](crate::grind) and their successors. A picture decides what a cell
//! *is*; this decides when it may become something else.
//!
//! # The rate is the safety property
//!
//! **No cell changes faster than [`FLIP_HZ`], by construction.** Every animation
//! is a function of the tick index this hands out, and the tick index advances
//! [`FLIP_HZ`] times a second and no faster, whatever any noise does.
//!
//! [`FLIP_HZ`] is 6 Hz, which is inside the 3–30 Hz photosensitive band, by
//! deliberate and recorded exemption — the band is about flashes covering a
//! substantial share of the visual field and these are bars two cells wide. See
//! DESIGN.md §19 for the argument and the three mitigations it is conditional
//! on. An instrument wanting a *slower* rhythm than this simply takes several
//! ticks per beat, which is what the mortar's stroke does; nothing needs a
//! faster one.
//!
//! # Nothing here is random
//!
//! The noise is an integer hash of position and tick, never an RNG, so the same
//! phase draws the same picture. That is what keeps `ORBS_DUMP` reproducible and
//! keeps every instrument's animation out of `orbs-sim`'s seeded streams.

use crate::tween;

/// How often any one cell of an animated instrument may change, in hertz.
///
/// **Not a tuning number.** See this module's header and DESIGN.md §19. Raising
/// it, widening the bars, or removing a mitigation named there reopens the
/// question for every instrument at once.
pub const FLIP_HZ: f32 = 6.0;

/// How long before the animations repeat themselves.
///
/// A period is needed at all so the phase can be wrapped: a frontend accumulates
/// `Time::delta_secs` forever, and an `f32` counting real seconds loses its
/// fraction after a few hours at the keyboard — the fire would visibly coarsen
/// during a long session and nothing would say why.
///
/// 24 s is long enough that the loop is not something an eye finds, and it is a
/// whole number of [`FLIP_HZ`] ticks so the wrap lands on a tick boundary.
///
/// **A tick boundary is not automatically seamless**, and this said it was.
/// Consecutive ticks are uncorrelated hashes for an ordinary cell, so the join
/// there looks like every other one — but a *travelling* pattern correlates them
/// on purpose, and for those the wrap was a 143-cell jump that re-randomised the
/// picture in a single frame every twenty-four seconds. `rising` and `falling`
/// are what close it; take a drift coordinate from them rather than subtracting
/// a tick by hand.
pub const CYCLE_SECS: f32 = 24.0;

/// How long one cell holds an appearance.
pub(crate) const TICK_SECS: f32 = 1.0 / FLIP_HZ;

/// Ticks in a cycle. Exact for the constants above: 24 × 6.
pub(crate) const TICKS_PER_CYCLE: u16 = 144;

/// Which tick `phase` falls in for one cell, offset by that cell's [`stagger`].
///
/// **This is the whole of the photosensitivity guarantee.** Appearance is a
/// function of the value this returns, and this advances [`FLIP_HZ`] times a
/// second, so no cell can flicker faster whatever the noise does.
pub(crate) fn tick_of(phase: f32, lane: u16, step: u16) -> u16 {
    at(phase + stagger(lane, step) * TICK_SECS)
}

/// The tick every cell agrees on.
///
/// For patterns that must *translate* rather than churn — the fire's plume, the
/// mortar's stroke. A staggered clock makes a travelling pattern land a tick
/// early on half its cells and the motion washes out; measured, it did.
pub(crate) fn shared_tick(phase: f32) -> u16 {
    at(phase)
}

/// The tick a given moment falls in.
///
/// The `% TICKS_PER_CYCLE` is not cosmetic: [`tween::mix`] rounds rather than
/// truncates, so a phase just short of the cycle end yields `TICKS_PER_CYCLE`
/// itself. Left alone that is an extra tick half a tick long, and the wrap
/// through it would be two transitions inside one tick period — the one place a
/// cell could exceed the rate [`tick_of`] exists to cap.
fn at(moment: f32) -> u16 {
    let wrapped = moment.rem_euclid(CYCLE_SECS);
    // The only `f32`-to-integer conversion here, and it goes through the one
    // function in this crate allowed to make it — see `tween::mix`.
    tween::mix(0, TICKS_PER_CYCLE, wrapped / CYCLE_SECS) % TICKS_PER_CYCLE
}

/// A cell's fixed offset into the tick, in `0.0..1.0`.
///
/// Why cells do not all turn over on the same instant. Fixed per cell rather
/// than per frame, so a cell's *rate* is still [`FLIP_HZ`] — only its phase
/// differs. Whole-field modulation is the hazard; a strip where every cell
/// changed together would be one however slow it was.
pub(crate) fn stagger(lane: u16, step: u16) -> f32 {
    let bits = hash(u32::from(lane).wrapping_mul(PRIME_LANE) ^ u32::from(step));
    // Widening, not a cast: `u16::from` and `f32::from` are both lossless, which
    // is what keeps this off the `cast_precision_loss` list.
    f32::from(low_bits(bits)) / 65536.0
}

/// Noise for one cell at one tick, `0..=255`.
pub(crate) fn noise(lane: u16, step: u16, tick: u16) -> u32 {
    let seed = hash(u32::from(lane).wrapping_mul(PRIME_LANE) ^ u32::from(step));
    hash(seed ^ u32::from(tick).wrapping_mul(PRIME_TICK)) >> 24
}

/// Noise for a travelling pattern, `0..=255` — **a function of position only**.
///
/// Separate from [`noise`] because the missing argument is the entire point: a
/// pattern that depends on the tick cannot translate, it can only churn. What
/// moves it is the *coordinate* it is sampled at, not the sample.
///
/// Take the coordinate from [`rising`] or [`falling`] rather than subtracting a
/// tick by hand — see those for why the cycle wrap tears otherwise.
pub(crate) fn drift_noise(lane: u16, drift: u16) -> u32 {
    hash(u32::from(lane).wrapping_mul(PRIME_TICK) ^ u32::from(drift)) >> 24
}

/// Where a pattern **travelling away from the bar's base** is sampled: the
/// fire's plume and its sparks, rising one cell per tick.
///
/// `travelled` is how many cells the pattern has moved since the cycle began,
/// and `span` is how many it moves in a whole cycle — for a pattern moving a
/// cell per tick that is [`TICKS_PER_CYCLE`], and for one moving every *n* ticks
/// it is `TICKS_PER_CYCLE / n`, which must divide exactly.
///
/// # The seam this closes
///
/// A translating picture is the one thing in this module that deliberately
/// **correlates** consecutive ticks — that is what makes it move rather than
/// churn — so it is also the only thing the cycle wrap can tear. [`shared_tick`]
/// runs `143 → 0`, and a coordinate computed as `step - tick` therefore jumps
/// 143 cells at that instant: the whole plume re-randomises once every 24
/// seconds, in one frame, for no reason on screen.
///
/// [`CYCLE_SECS`] used to claim there was no seam, and for [`noise`] there is
/// none — consecutive ticks *are* uncorrelated there, so the join looks like
/// every other tick boundary. For a drift the argument runs backwards, and the
/// tear was measured: 24 of 32 plume cells translated across the wrap against 31
/// of 32 everywhere else, and 26 of 40 for the mortar's debris.
///
/// Reducing the coordinate modulo `span` closes it, because `143` and `-1` are
/// then the same coordinate and the wrap is one more ordinary step.
pub(crate) const fn rising(step: u16, travelled: u16, span: u16) -> u16 {
    // `span - travelled` rather than a subtraction that could go negative. When
    // `travelled` is zero this is `span` itself, and the outer modulus takes it
    // back to zero.
    offset(step, span - travelled % span, span)
}

/// Where a pattern **travelling toward the bar's base** is sampled: the mortar's
/// debris, falling a cell every [`FALL_EVERY`](crate::grind) ticks.
///
/// The mirror of [`rising`], and it closes the same seam — see there.
pub(crate) const fn falling(step: u16, travelled: u16, span: u16) -> u16 {
    offset(step, travelled % span, span)
}

/// A position shifted along a pattern that repeats every `span` cells.
///
/// `step` is reduced too, so a bar longer than `span` repeats its texture rather
/// than running off the end of the pattern. No bar the panel draws comes close:
/// the widest is about 130 cells against a span of 144, and the only cells that
/// travel at all are the four of the mortar's working gap.
const fn offset(step: u16, shift: u16, span: u16) -> u16 {
    (step % span + shift) % span
}

/// Noise keyed to a third thing, for a picture that needs two independent
/// draws from the same position — the mortar's chunks against its dust.
pub(crate) fn salted(lane: u16, step: u16, salt: u32) -> u32 {
    hash(hash(u32::from(lane).wrapping_mul(PRIME_LANE) ^ u32::from(step)) ^ salt) >> 24
}

/// Noise as a small integer, `0..=3`.
pub(crate) fn shade(noise: u32) -> u16 {
    u16::try_from(noise >> 6).unwrap_or(0)
}

/// A **second** small integer from the same noise, `0..=3`.
///
/// Different bits rather than a second hash: a picture's colour and its glyph
/// often want to be correlated but not identical — the flame's cooler cells
/// being the frayed ones is right, cooler cells being frayed *every* time is a
/// gradient with a hard edge in it.
pub(crate) fn shade2(noise: u32) -> u16 {
    u16::try_from((noise >> 4) & 0b11).unwrap_or(0)
}

/// The low 16 bits, as the widest integer that converts to `f32` losslessly.
fn low_bits(bits: u32) -> u16 {
    u16::try_from(bits & 0xFFFF).unwrap_or(0)
}

pub(crate) const PRIME_LANE: u32 = 0x9E37_79B9;
pub(crate) const PRIME_TICK: u32 = 0x85EB_CA6B;
pub(crate) const PRIME_SPARK: u32 = 0xC2B2_AE35;

/// `lowbias32` — a well-known integer mixer with no dependency and no state.
///
/// An RNG would have been the obvious reach and is the wrong one: an animation
/// must draw the same way for the same phase, or `ORBS_DUMP` stops being
/// reproducible and the See-it gate stops being evidence.
pub(crate) const fn hash(seed: u32) -> u32 {
    let mut bits = seed;
    bits ^= bits >> 16;
    bits = bits.wrapping_mul(0x7feb_352d);
    bits ^= bits >> 15;
    bits = bits.wrapping_mul(0x846c_a68b);
    bits ^= bits >> 16;
    bits
}

/// What every picture's tests sweep with.
///
/// The two numbers below were copied into [`fire`](crate::fire) and
/// [`grind`](crate::grind) verbatim, and they are not arbitrary — they are how
/// finely a cycle has to be sampled before a one-frame transient can hide, which
/// is the same question for every instrument. Two copies is two places to relax
/// it without noticing.
#[cfg(test)]
pub(crate) mod harness {
    use super::CYCLE_SECS;

    /// How finely the sweeps sample a cycle.
    ///
    /// `u16` throughout: every integer here reaches `f32` by `From`, never by
    /// `as`, which keeps the tests off the same cast lints the modules avoid.
    const STEPS: u16 = 2400;

    /// How finely a rate check samples, in hertz. Far above any frame rate, so
    /// a transient between two frames cannot hide.
    pub(crate) const RATE: u16 = 240;

    /// Phases across a whole cycle, finely enough to catch a transient.
    pub(crate) fn sweep() -> impl Iterator<Item = f32> {
        (0..STEPS).map(|n| f32::from(n) * (CYCLE_SECS / f32::from(STEPS)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flip_rate_is_bounded_and_known() {
        // The rate is inside the photosensitive band by exemption (DESIGN.md
        // §19), so what has to be held is that it is bounded and deliberate —
        // the quantiser is the only thing between 6 Hz and whatever a noise
        // function felt like doing. Eight is past what the exemption argued for.
        const { assert!(FLIP_HZ <= 8.0, "past what the §19 exemption argued for") }
        const { assert!(FLIP_HZ > 0.5, "the flip rate is slow enough to look dead") }
        // The cycle must be a whole number of ticks, or the wrap lands mid-tick
        // and one cell gets two transitions inside one tick period.
        //
        // **Exact, not within a tolerance.** This compared against `f32::EPSILON`
        // — which is the gap between 1.0 and its successor, about 128 times
        // *tighter* than one representable step at a magnitude of 144. So the
        // tolerance was neither the slack it looked like nor a deliberate bound;
        // it passed because the product happens to be exact. "A whole number of
        // ticks" admits no slack anyway: half a tick of error is the entire
        // defect.
        let ticks = CYCLE_SECS * FLIP_HZ;
        assert!(
            ticks.floor() == ticks && ticks == f32::from(TICKS_PER_CYCLE),
            "{CYCLE_SECS}s at {FLIP_HZ}Hz is {ticks} ticks, not {TICKS_PER_CYCLE} whole ones",
        );
    }

    #[test]
    fn the_tick_never_leaves_its_cycle() {
        for n in 0..2000u16 {
            let phase = f32::from(n) * (CYCLE_SECS / 500.0);
            assert!(shared_tick(phase) < TICKS_PER_CYCLE, "at phase {phase}");
        }
    }

    #[test]
    fn staggers_are_spread_rather_than_clustered() {
        // If every cell drew the same offset this would silently become a
        // synchronised clock, which is the hazard the rate cap alone misses.
        let mut buckets = [0u32; 4];
        for lane in 0..2u16 {
            for step in 0..64u16 {
                // Through `mix`, like every other float-to-integer step in this
                // crate — see `tween`.
                let slot = tween::mix(0, 3, stagger(lane, step));
                buckets[usize::from(slot)] += 1;
            }
        }
        assert!(
            buckets.iter().all(|count| *count > 10),
            "staggers cluster: {buckets:?}",
        );
    }
}
