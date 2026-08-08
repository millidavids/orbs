//! What a burning meter looks like.
//!
//! The athanor is the one instrument that is literally a fire (DESIGN.md §10.1),
//! and its meter reports fuel **remaining** where every other instrument reports
//! ticks elapsed — see [`Meter`](orbs_sim::tower::Meter). So its bar drains, and
//! drawing it as fire gets the right story for free: the flame shrinks and the
//! smoke above it grows.
//!
//! Nothing here knows what a second is. A frontend owns the clock and passes a
//! phase, exactly as it does for [`tween`](crate::tween) — this decides only what
//! a cell *is*, never when.
//!
//! # The shape of a fire
//!
//! Hottest at the **base**, mellowing toward the tip — a fuel bed glows where the
//! fuel is, not where it has already burned. So heat falls with height, and the
//! bottom half of the flame is **solid `█` throughout**, carrying every bit of
//! its motion in hue. `▓` only begins about halfway up and grows commoner toward
//! the tip, which is where a real flame breaks up. Sparks come off the top and
//! rise through the plume, cooling as they go.
//!
//! # Four properties this module is built to hold
//!
//! **No cell changes faster than [`FLIP_HZ`], by construction.** A cell's
//! appearance is a function of its **tick index**, and the tick index advances
//! [`FLIP_HZ`] times a second and no faster, whatever the noise does.
//!
//! **[`FLIP_HZ`] is 6 Hz, and that is inside the photosensitive band.** §14 and
//! the CRT port both fixed 3–30 Hz as the band to stay out of, and this is a
//! deliberate, recorded departure from it — DESIGN.md §19. The short version:
//! the band exists for flashes covering a substantial part of the visual field
//! (W3C puts the threshold at a quarter of it), and this element is a bar two
//! cells wide. The blanket floor was calibrated on the *whole tube* flickering at
//! 19.1 Hz, which is three orders of magnitude more area. The mitigations below
//! are what make the exemption defensible rather than merely convenient — if any
//! of them is removed, the rate has to come back down with it.
//!
//! **Nothing turns over together, and each step is small.** Every cell's tick
//! boundaries are offset by a fixed fraction of a tick ([`stagger`]), so a change
//! is always a few cells out of thirty rather than the strip as a whole; and the
//! ramp the base shifts along is deliberately compressed, so a flip is a hue step
//! rather than an on/off flash. Whole-field modulation is the hazard, not motion.
//!
//! **The fill boundary survives with no colour at all.** The cell ahead of the
//! flame front is **always blank** — that pin is absolute, and sparks are barred
//! from it. Behind the front is `█` or `▓`, so the join is at worst 75% against
//! nothing at all.
//!
//! That is the strongest join the alphabet can draw, and it arrived by a route
//! worth recording: the empty track *was* a solid field of `░`, and giving that
//! glyph up to the smoke — where it is now the last of a puff pittering out —
//! left the background empty as a side effect. The rule had been relaxed once
//! already (from `█`/`░` to `█`-or-`▓` against `░`, a 3:1 coverage step, to let
//! `▓` reach the flame tip); this bought back more than that cost. The join that
//! was never allowed is `▓` against `▒` — one dither step, which the CRT's
//! phosphor bloom erases, and the meter's value is read off this join.
//!
//! **It is deterministic.** The noise is an integer hash of position and tick,
//! never an RNG, so the same phase draws the same fire — which is what keeps
//! `ORBS_DUMP` reproducible and keeps this out of `orbs-sim`'s seeded streams
//! altogether.

use crate::pulse::{
    PRIME_SPARK, TICKS_PER_CYCLE, drift_noise, hash, noise, rising, shade, shade2, shared_tick,
    tick_of,
};
use crate::style::{Density, Depiction, Heat};
use crate::tween;

/// One in this many plume positions throws a spark.
///
/// Sparse on purpose: a plume with a spark in every other cell is a dotted line,
/// not a fire. At 17 a bar's worth of plume carries one or two in flight.
const SPARK_ODDS: u32 = 17;

/// How many cells a puff of smoke rises before it loses one step of body.
///
/// Sets how far the plume reaches: a puff starts with `0..=7` of body and goes
/// out when it runs out, so the last of it is gone around `8 × THIN_EVERY` cells
/// up. At three that is roughly twenty — longer than any bar the panel draws, so
/// the plume fades because it is fading rather than because it hit a ceiling.
const THIN_EVERY: u16 = 3;

/// How much body a puff needs to show at all on a **cold** hearth.
///
/// Against a `0..=7` vigour, and rising a step per cell, so the wisp tapers off
/// over about three: 37% of the bottom cell, then 25%, then 12%, then nothing.
/// High on purpose — this is a hearth that has gone out, and anything more reads
/// as a fire someone forgot to draw.
const COLD_FLOOR: u16 = 5;

/// How much hotter the flare runs at the instant of ignition, in ramp steps.
///
/// Added to every cell's warmth and decayed to nothing, so `kindle` lands as a
/// fire catching rather than as a bar changing value.
///
/// **It pushes the ramp up; it does not flatten it.** At four, the bottom half
/// saturates at `Core` while the tip only reaches `Blaze` — the flame stays a
/// flame with a cooler tip, and gets brighter. Six would take every cell to
/// `Core` and the whole bar would go one flat colour for half a second, which
/// reads as a UI flash rather than as something catching light.
const FLARE_BOOST: u16 = 4;

/// How much likelier a spark is while the flare lasts.
///
/// A fire catching throws a shower; this is what makes the moment read as an
/// event rather than as the steady state arriving early.
const FLARE_SPARKS: u32 = 4;

/// What the hearth is doing, beyond how full it is.
///
/// A struct rather than three more arguments because these travel together
/// everywhere and always will: they are one description of a fire's state, and
/// [`Painter::fire_meter`](crate::Painter::fire_meter) would otherwise be at six
/// parameters before the next thing anyone wants to add to a flame.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Burn {
    /// Elapsed seconds from the frontend's own clock. Nothing here knows what a
    /// second is; this is only ever compared against itself.
    pub phase: f32,
    /// How recently it was lit: `1.0` at the instant of ignition, decaying to
    /// `0.0`.
    ///
    /// Drives the flare: the flame **grows up out of the base** as this decays,
    /// burning hard where it has just caught and settling behind itself, while
    /// the plume throws a shower of sparks. Fuel above the front is drawn as
    /// nothing — present, but not alight.
    ///
    /// So the bar animates *to* its value over the tick after `kindle` rather
    /// than snapping to it — **the one place the fire shows less than its
    /// value**, and the same trade
    /// [`RecordView::revealing`](crate::RecordView::revealing) makes for
    /// arriving text and
    /// [`ScreenLayout::transition`](crate::ScreenLayout::transition) makes for
    /// arriving panes. §14 is unharmed: the linear stream never sees the
    /// animation. DESIGN.md §19 records why it is worth paying.
    pub flare: f32,
    /// Whether there is fire at all.
    ///
    /// `false` is a **cold hearth**, not an empty bar: no flame at any fill, no
    /// sparks, and a wisp of smoke off the bottom. The athanor reports no meter
    /// once it is out (`State::Cold`), so this is the only thing distinguishing
    /// "gone out" from "burning, with nothing left".
    pub lit: bool,
}

/// One cell of a burning meter.
///
/// `step` counts from the end the bar fills from — leftward for a horizontal
/// meter, upward for a vertical one — so both orientations share this and cannot
/// drift apart. `filled` is what [`filled_of`](crate::paint) computed, so the
/// fire and the plain meter always agree about where the boundary is.
///
/// `guttering` is the exception to that agreement and the only one: a fire with
/// fuel left but not enough to fill a cell. See [`gutter`].
pub(crate) fn cell(
    lane: u16,
    step: u16,
    filled: u16,
    guttering: bool,
    burn: Burn,
) -> (char, Depiction) {
    if !burn.lit {
        return cold(lane, step, burn.phase);
    }

    // **The flame is only as tall as the fire has caught.** While the flare
    // climbs, the fuel above the front is there but not alight, and it is drawn
    // as *nothing*. Drawing it as dark flame instead — which is what "present
    // but unlit" literally is — filled the whole bar with dithered orange the
    // instant `kindle` landed, and the flare then read as a highlight sweeping
    // over an already-full bar rather than as a fire taking hold.
    //
    // At rest `caught` is `filled`, so every path below is the ordinary one and
    // nothing needs to know a flare exists.
    let front = caught(filled, burn.flare);

    if step < front {
        flame(lane, step, front, burn)
    } else if step == 0 && guttering {
        gutter(lane, burn.phase)
    } else {
        // **A guttering ember stands in for a lit cell.** Without this the plume
        // starts at `ahead == 0` on the ember's own cell, the pinned blank never
        // gets drawn, and the ember ends up directly against `▒` — the one join
        // forbidden everywhere else in this module. There is no *value* to
        // misread there, since nothing is filled, but a `▓` touching a `▒` is
        // muddy wherever it happens.
        let base = if guttering { 1 } else { front };
        plume(lane, step, step.saturating_sub(base), front, burn)
    }
}

/// The last of a fire, when the bar has rounded its fuel away.
///
/// **The one place the fire deliberately shows more than the plain meter would.**
/// A hearth with a few ticks left divides to zero cells, and a bar that draws
/// nothing says *out* — which is wrong, and wrong in the direction that matters,
/// because "still lit" and "cold" are the two states `kindle` distinguishes and
/// the whole panel exists to save the player from having to touch a thing to
/// learn its state.
///
/// Always `▓` and never `█`: a guttering ember is not a solid cell of fire, and
/// keeping the glyph distinct is what stops this reading as one cell of fill.
fn gutter(lane: u16, phase: f32) -> (char, Depiction) {
    let noise = noise(lane, 0, tick_of(phase, lane, 0));
    let heat = if shade(noise) >= 3 {
        Heat::Flame
    } else {
        Heat::Ember
    };
    ('▓', Depiction::flame(heat))
}

/// A hearth that has gone out: a wisp of smoke off the bottom and nothing else.
///
/// Tapered over about three cells rather than cut off at one, so it reads as
/// smoke thinning rather than as a row of dots. Everything above is clear — a
/// cold athanor is the one instrument that draws no track at all, which is what
/// makes it obviously different from a banked one at a glance.
fn cold(lane: u16, step: u16, phase: f32) -> (char, Depiction) {
    // **The wisp travels, so its coordinate has to survive the cycle wrap.**
    // See [`rising`].
    let drift = rising(step, shared_tick(phase), TICKS_PER_CYCLE);
    let vigour = drift_noise(lane, drift) >> 5;
    // **Never nothing at the very bottom.** `vigour` clears `COLD_FLOOR` about
    // three times in eight, so all four cells of a two-lane wisp could miss at
    // once — and with motion off the phase is pinned at zero, which freezes that
    // blank forever. A cold hearth drawing nothing at all is indistinguishable
    // from a row the panel forgot, which is precisely the invisible-state defect
    // §10.1's panel exists to remove.
    //
    // The third time this lesson has been paid for: `caught` floors the flame at
    // one cell so ignition is never a blank frame, and the pour floors the block
    // at one for the same reason. A picture that can vanish is not a picture.
    if step == 0 && lane == 0 {
        return ('░', Depiction::smoke(Density::Thin));
    }
    if vigour >= u32::from(COLD_FLOOR + step) {
        ('░', Depiction::smoke(Density::Thin))
    } else {
        (' ', Depiction::None)
    }
}

/// A cell of the fire itself, `step` cells up from the base of `filled`.
///
/// **Hottest at the base.** An earlier version put the brightest cell at the
/// flame *front* on the reasoning that the front is where fuel is being consumed.
/// It read as a bar with a bright edge rather than as a fire: what an eye expects
/// is a glowing bed that mellows into its tip.
fn flame(lane: u16, step: u16, height: u16, burn: Burn) -> (char, Depiction) {
    let noise = noise(lane, step, tick_of(burn.phase, lane, step));

    // How far up the flame, in quarters. The whole shape is expressed against
    // this rather than against a cell count, so a two-cell fire and a
    // twenty-cell one have the same proportions — the athanor's bar spans both
    // within one burn, and a flare spans both within one second.
    //
    // `height` is how tall the flame *is*, which during a flare is how far it
    // has caught rather than how much fuel there is. So the gradient stretches
    // as the fire climbs instead of the fire filling in a pre-drawn gradient.
    let up = quarters(step, height);

    // Warm at the bottom, cool at the top, with the noise shifting each cell
    // through the ramp rather than settling it on one step. `up * 2` against a
    // noise of `0..=3` is what makes every height shift between two colours
    // instead of holding one: the base moves between `Blaze` and `Core`, the
    // tip between `Ember` and `Flame`.
    //
    // **The flare adds heat on top of the shape.** The fire being *shorter* is
    // what makes it climb (see `cell`); this is what makes the newly-caught part
    // of it burn hard and settle. Both fade together over one tick.
    let warmth = (3 - up) * 2 + shade(noise) + tween::mix(0, FLARE_BOOST, burn.flare);
    let heat = match warmth {
        0 | 1 => Heat::Ember,
        2..=4 => Heat::Flame,
        5..=7 => Heat::Blaze,
        _ => Heat::Core,
    };

    // **Solid for the bottom half, breaking up toward the tip.** `up >= 2` is
    // the halfway line; above it `▓` grows commoner, so the flame frays where a
    // real one does. A second draw from the same noise rather than a second
    // hash — cooler cells being the broken ones is the correlation you want.
    let glyph = if up >= 2 && shade2(noise) + up * 2 >= 7 {
        '▓'
    } else {
        '█'
    };
    (glyph, Depiction::flame(heat))
}

/// How far up the fuel the fire has caught, in cells — the flame's real height.
///
/// Climbs from the base to `filled` as `flare` decays, so `kindle` is a fire
/// taking hold rather than a full bar changing colour. Everything above it is
/// drawn as nothing: fuel that is present but has not lit.
///
/// **This is the one place the fire meter shows less than its value**, and only
/// for the single tick after ignition. It is the same trade
/// [`Reveal`](crate::RecordView::revealing) makes for arriving text and
/// [`ScreenLayout::transition`](crate::ScreenLayout::transition) makes for
/// arriving panes: a value animating *to* the truth reads better than one
/// teleporting to it, and the linear stream — which is what §14 actually
/// protects — never sees the animation at all.
///
/// **At least one cell, always.** At the exact instant of ignition the front
/// would otherwise be at zero and the bar would be empty for a frame, right when
/// the player is looking for something to have happened.
///
/// At rest (`flare == 0`) this is `filled`, so every path that uses it is the
/// ordinary one and nothing needs to special-case a hearth that is simply
/// burning.
fn caught(filled: u16, flare: f32) -> u16 {
    // **No fuel, no flame, floor or no floor.** The `max(1)` below exists so a
    // fire always shows *something* the instant it is lit; applied to an empty
    // bar it invents a cell of fire out of nothing, which put a flame on a spent
    // athanor and on every zero-valued reading the plain meter draws empty.
    if filled == 0 {
        return 0;
    }
    tween::mix(0, filled, 1.0 - flare).max(1)
}

/// How far `step` is through `total`, in quarters — `0..=3`.
///
/// Guards `total == 0` because `filled` can be zero on a spent athanor, and this
/// is reached before that is ruled out anywhere else.
fn quarters(step: u16, total: u16) -> u16 {
    if total == 0 {
        return 0;
    }
    (step.saturating_mul(4) / total).min(3)
}

/// A cell above the fire: a spark if one is passing, otherwise smoke.
///
/// `front` is the flame's real height — [`caught`], not `filled` — because that
/// is what the call site passes and what the spark test below means. The two
/// agree on the only question asked of them (`caught` is zero exactly when
/// `filled` is), so the old name was harmless and wrong, which is the kind that
/// survives longest.
fn plume(lane: u16, step: u16, ahead: u16, front: u16, burn: Burn) -> (char, Depiction) {
    // **The pinned boundary, and the one cell a spark may not occupy.** Empty,
    // which reads as the clear hot air a real fire has above it before the smoke
    // gathers — and which is also the strongest join the alphabet can draw. The
    // meter's value is read off it.
    if ahead == 0 {
        return (' ', Depiction::None);
    }
    // **No fire, no sparks.** A spent athanor is still `Burning` for the tick
    // before it goes out, and its bar is all plume; sparks coming off nothing
    // would say there is fuel left when the bar says there is none.
    if front > 0
        && let Some(spark) = spark(lane, step, ahead, burn)
    {
        return spark;
    }
    smoke(lane, step, ahead, burn.phase)
}

/// A scrap of fire riding the plume, if one is at this cell.
///
/// **A spark keeps a fixed identity for its whole life.** Its `drift` coordinate
/// is constant while it travels — `step` and the tick rise together — so one hash
/// settles both whether it exists and how far it gets, and it simply moves. That
/// is why there is no stationary cutoff here: a lifetime read off the *travelling*
/// coordinate rides along with the spark, where one read off `ahead` would be a
/// fixed ceiling that every spark died at and would read as a hard edge.
fn spark(lane: u16, step: u16, ahead: u16, burn: Burn) -> Option<(char, Depiction)> {
    // Wrapped through [`rising`], or every spark in flight is replaced by a
    // different one at the cycle boundary — see there.
    let drift = rising(step, shared_tick(burn.phase), TICKS_PER_CYCLE);
    let seed = hash(u32::from(lane).wrapping_mul(PRIME_SPARK) ^ u32::from(drift));

    // A fire catching throws a shower, and then settles to the odd one. Stepped
    // rather than interpolated: the odds are an integer modulus, and easing one
    // would mean sparks blinking in and out as the divisor crossed each value
    // instead of a burst that thins.
    let odds = if burn.flare > 0.25 {
        SPARK_ODDS / FLARE_SPARKS
    } else {
        SPARK_ODDS
    };
    if !seed.is_multiple_of(odds) {
        return None;
    }

    // How high this one gets before it goes out: 2..=8 cells, its own for life.
    let life = 2 + u16::try_from((seed / SPARK_ODDS) % 7).unwrap_or(0);
    if ahead > life {
        return None;
    }

    // Cooling and shrinking as it climbs. `∙` has body, `°` is a ring with the
    // heat gone out of its middle, `·` is the last of it.
    Some(match quarters(ahead, life) {
        0 => ('∙', Depiction::spark(Heat::Core)),
        1 => ('∙', Depiction::spark(Heat::Blaze)),
        2 => ('°', Depiction::spark(Heat::Flame)),
        _ => ('·', Depiction::spark(Heat::Ember)),
    })
}

/// A cell of the smoke itself, `ahead` cells past the front.
fn smoke(lane: u16, step: u16, ahead: u16, phase: f32) -> (char, Depiction) {
    // **The plume drifts, and it needs a shared clock to do it.** Subtracting
    // the tick from the position translates the pattern one cell per tick, away
    // from the fire — upward in a vertical meter, rightward in a horizontal one.
    //
    // This is the one place `stagger` is deliberately *not* used, and the reason
    // is that the two properties genuinely conflict. A translation only reads as
    // one if neighbouring cells step together; staggered, cell `s+1` is half the
    // time a tick behind cell `s`, the shift never lines up, and measurement
    // says so — the drift washed out to 78504 aligned against 78640 in place,
    // which is a coin toss. The term was doing nothing but costing a hash.
    //
    // Safety survives the exchange because [`FLIP_HZ`] is unaffected — a smoke
    // cell still changes at the capped rate and no faster — and because what
    // moves together here is a dim glyph on the dim half of the bar. The
    // **flame** keeps its stagger, and that is the half where a synchronised
    // flicker would be bright enough to matter.
    //
    // **And it wraps through [`rising`] rather than by plain subtraction**,
    // because the cycle boundary is the one place a *correlated* sequence of
    // ticks can tear — measured at 24 of 32 cells translating across the wrap
    // against 31 of 32 elsewhere. See `pulse::rising`.
    let drift = rising(step, shared_tick(phase), TICKS_PER_CYCLE);

    // **The pattern is a function of `drift` and nothing else**, which is what
    // makes it translate rather than merely churn. Passing the tick to the hash
    // as well — the obvious thing, since every other sample here is
    // position-and-time — re-randomises the whole plume every tick and the
    // motion vanishes. It measured as a coin toss.
    let vigour = u16::try_from(drift_noise(lane, drift) >> 5).unwrap_or(0);

    // **`ahead` is both the cell's height and the puff's age, and they are the
    // same number.** A puff at height `h` left the fire `h` ticks ago — it rose
    // one cell per tick to get there — so thinning it by `ahead` is not the
    // stationary threshold that killed an earlier version of the drift. The puff
    // ages *because* it travels, which is what fading is.
    match vigour.saturating_sub(ahead / THIN_EVERY) {
        // Gone. Most of a plume is air, and this is what buys the top of the bar
        // back: the empty track used to be a solid wall of `░`.
        0 => (' ', Depiction::None),
        // Pittering out — the last of a puff before it disperses.
        1..=4 => ('░', Depiction::smoke(Density::Thin)),
        // Still has body, near the fire that made it.
        _ => ('▒', Depiction::smoke(Density::Thick)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse::harness::{RATE, sweep};
    use crate::pulse::{CYCLE_SECS, FLIP_HZ, TICK_SECS};

    /// A hearth alight at `phase`, with no flare in it.
    fn lit(phase: f32) -> Burn {
        Burn {
            phase,
            flare: 0.0,
            lit: true,
        }
    }

    /// Every cell of a bar of `steps` with `filled` alight, at one phase.
    fn bar(steps: u16, filled: u16, phase: f32) -> Vec<(char, Depiction)> {
        (0..steps)
            .map(|s| cell(0, s, filled, false, lit(phase)))
            .collect()
    }

    #[test]
    fn no_cell_outruns_the_declared_flip_rate() {
        // **The rate is inside the photosensitive band by exemption, so this
        // test changed shape rather than changed number.** It used to assert
        // `FLIP_HZ < 3.0`; the exemption (DESIGN.md §19) rests on the element
        // being a two-cell bar rather than the tube, so what has to be held now
        // is that the rate is *bounded and known* — the quantiser is the only
        // thing standing between 6 Hz and whatever the noise felt like doing.
        //
        // The ceiling is 30 Hz because that is the far edge of the band: past it
        // the hazard argument changes entirely and this file should not be where
        // that gets decided quietly.
        const { assert!(FLIP_HZ <= 8.0, "past what the §19 exemption argued for") }
        const { assert!(FLIP_HZ > 0.5, "the flip rate is slow enough to look dead") }

        // Walk every cell of a bar through a whole cycle at 240 Hz and count how
        // often each actually changes.
        for step in 0..24u16 {
            let mut changes = 0u16;
            let mut last = cell(0, step, 10, false, lit(0.0));
            for n in 1..RATE * 24 {
                let now = cell(0, step, 10, false, lit(f32::from(n) / f32::from(RATE)));
                if now != last {
                    changes += 1;
                    last = now;
                }
            }
            let hz = f32::from(changes) / CYCLE_SECS;
            assert!(
                hz <= FLIP_HZ,
                "cell {step} changed at {hz} Hz, over the {FLIP_HZ} cap",
            );
        }
    }

    #[test]
    fn cells_do_not_all_turn_over_together() {
        // Whole-field modulation is the hazard the rate cap alone does not
        // cover: a strip where every cell changed on the same instant would be
        // a flash however slow. `stagger` is what prevents it, so this fails if
        // it is ever removed as "an offset that makes no difference".
        //
        // **Measured on the flame, not on the whole bar.** The plume is
        // deliberately *un*staggered — a translation only reads as one if
        // neighbouring cells step together, and `smoke` documents the exchange
        // at length — so half the bar turning over at once is the design rather
        // than the defect, and the exchange is safe because what moves together
        // there is a dim glyph on the dim half.
        //
        // The old version swept all twenty cells against a threshold of ten and
        // passed with nine. That is not a margin; it is a coin landing the right
        // way, and re-keying the plume's drift was enough to tip it.
        let (steps, filled) = (20u16, 10u16);
        let mut busiest = 0;
        let mut previous = bar(steps, filled, 0.0);
        for phase in sweep().skip(1) {
            let now = bar(steps, filled, phase);
            let changed = previous
                .iter()
                .zip(&now)
                .take(usize::from(filled))
                .filter(|(before, after)| before != after)
                .count();
            busiest = busiest.max(changed);
            previous = now;
        }

        // If the flame's cells were synchronised, a handful of frames would
        // carry all of the change and the rest none — so it is the *busiest*
        // frame that tells.
        assert!(
            busiest < usize::from(filled) / 2,
            "{busiest} of {filled} flame cells changed on one frame — it flashes",
        );
    }

    #[test]
    fn the_fill_boundary_survives_in_greyscale() {
        // §14: no meaning carried by colour alone. The meter's *value* is read
        // off this one join, so it has to be findable with the hue thrown away,
        // at every phase and every fill.
        //
        // **The lit side is `█` or `▓`; the dark side is blank.** The lit side
        // was pinned to `█` alone until `▓` was allowed to reach the flame tip —
        // a 3:1 coverage step where the plain meter draws 4:1. Handing `░` over
        // to the smoke then emptied the track and made the join 75%-or-100%
        // against nothing, which is stronger than either. The dark side is
        // pinned absolutely, sparks included.
        for phase in sweep() {
            for filled in 1..20u16 {
                let cells = bar(20, filled, phase);
                let lit = cells[usize::from(filled) - 1].0;
                assert!(
                    lit == '█' || lit == '▓',
                    "phase {phase}, filled {filled}: the lit side of the join was {lit:?}",
                );
                if filled < 20 {
                    assert_eq!(
                        cells[usize::from(filled)].0,
                        ' ',
                        "phase {phase}, filled {filled}: the dark side of the join was not clear",
                    );
                }
            }
        }
    }

    /// A hearth that has gone out.
    fn out(phase: f32) -> Burn {
        Burn {
            phase,
            flare: 0.0,
            lit: false,
        }
    }

    #[test]
    fn a_cold_hearth_is_never_a_blank_column() {
        // **The failure this exists for.** `vigour` clears `COLD_FLOOR` about
        // three times in eight, and the wisp is only three cells tall, so every
        // draw in a narrow bar could miss at once — and reduce-motion pins the
        // phase at zero, which freezes that blank for the session. A cold
        // hearth drawing nothing is indistinguishable from a row the panel
        // forgot, and "you cannot tell what state a thing is in without touching
        // it" is the complaint §10.1's panel was built to answer.
        //
        // Swept over the phase because the wisp travels: a version floored only
        // at one phase would leave the others able to vanish.
        let out = |phase: f32| Burn {
            phase,
            flare: 0.0,
            lit: false,
        };
        for phase in sweep() {
            for steps in [1u16, 2, 4, 16] {
                let any = (0..steps).any(|s| cell(0, s, 0, false, out(phase)).0 != ' ');
                assert!(
                    any,
                    "a cold hearth drew nothing at phase {phase} ({steps} cells)"
                );
            }
        }
    }

    #[test]
    fn a_cold_hearth_smokes_at_the_bottom_and_nowhere_else() {
        // What `State::Cold` draws instead of a bar. It has to be *visible* —
        // this is the only thing telling a player the athanor is out rather
        // than that the panel forgot to draw it — and it has to be slight, or a
        // dead hearth reads as a lit one.
        let (mut bottom, mut above) = (0u32, 0u32);
        for phase in sweep() {
            for step in 0..16u16 {
                if cell(0, step, 0, false, out(phase)).0 == ' ' {
                    continue;
                }
                if step < 4 {
                    bottom += 1;
                } else {
                    above += 1;
                }
            }
        }
        assert!(bottom > 0, "a cold hearth showed nothing at all");
        assert_eq!(above, 0, "cold smoke climbed the whole bar: {above} cells");

        // Never fire, at any phase — that is the whole distinction.
        for phase in sweep() {
            for step in 0..16u16 {
                let (glyph, depiction) = cell(0, step, 0, false, out(phase));
                assert!(
                    depiction.is_smoke() || depiction == Depiction::None,
                    "a cold hearth drew {glyph:?}",
                );
            }
        }
    }

    #[test]
    fn a_guttering_ember_keeps_its_clear_air() {
        // A hearth with fuel the bar rounded away shows one faint ember, and the
        // cell above it is the same pinned blank a real flame front gets. Without
        // it the ember sits against `▒` — `▓` touching `▒` is the one join this
        // module forbids everywhere else.
        for phase in sweep() {
            let (glyph, depiction) = cell(0, 0, 0, true, lit(phase));
            assert_eq!(glyph, '▓', "the ember was not a guttering one");
            assert!(
                matches!(depiction, Depiction::FlameEmber | Depiction::FlameBody),
                "the ember was too hot: {depiction:?}",
            );
            assert_eq!(
                cell(0, 1, 0, true, lit(phase)).0,
                ' ',
                "the ember lost its clear air",
            );
        }

        // ...and with no fuel at all there is no ember, only plume.
        for phase in sweep() {
            assert_ne!(cell(0, 0, 0, false, lit(phase)).0, '▓');
        }
    }

    #[test]
    fn a_catching_fire_burns_harder_than_a_settled_one() {
        // The other half of the flare, and the half a glyph dump cannot show:
        // what has *just* caught burns at the top of the ramp and settles behind
        // itself. Growth alone would give a fire that grew at its resting
        // brightness, which reads as a bar filling rather than as fuel taking.
        let filled = 16u16;
        let hottest = |flare: f32| {
            let burn = Burn {
                phase: 0.3,
                flare,
                lit: true,
            };
            // Measured over the part that has caught — the rest is not fire yet,
            // so averaging it in would just be measuring the growth again.
            let front = caught(filled, flare);
            (0..front)
                .filter(|step| cell(0, *step, filled, false, burn).1 == Depiction::FlameCore)
                .count()
                * 100
                / usize::from(front)
        };

        assert!(
            hottest(1.0) > hottest(0.0) + 25,
            "ignition barely registered: {}% at Core against {}% settled",
            hottest(1.0),
            hottest(0.0),
        );

        // ...and it does not flatten. A settled flame keeps a cool tip, and so
        // does a flaring one — a bar at one uniform colour reads as a UI flash
        // rather than as something catching light.
        let flaring = Burn {
            phase: 0.3,
            flare: 0.3,
            lit: true,
        };
        let tip = caught(filled, 0.3) - 1;
        assert!(
            cell(0, tip, filled, false, flaring).1 != Depiction::FlameCore,
            "the flare flattened the flame to one colour",
        );
    }

    #[test]
    fn the_flame_grows_into_the_bar_rather_than_filling_it_at_once() {
        // **The fuel above the front is drawn as nothing.** Drawing it as dark
        // flame is what "present but not alight" literally is, and it filled the
        // whole bar with dithered orange the instant `kindle` landed — the flare
        // then read as a highlight sweeping over an already-full bar rather than
        // as a fire taking hold.
        let filled = 16u16;
        let alight = |flare: f32| {
            let burn = Burn {
                phase: 0.3,
                flare,
                lit: true,
            };
            (0..filled)
                .filter(|step| cell(0, *step, filled, false, burn).1.is_flame())
                .count()
        };

        // It grows, and it ends up exactly as tall as the fuel.
        let (start, middle, end) = (alight(1.0), alight(0.5), alight(0.0));
        assert!(
            start < middle && middle < end,
            "the flame did not grow: {start} lit at ignition, {middle} halfway, {end} settled",
        );
        assert_eq!(start, 1, "ignition lit more than the base");
        assert_eq!(
            end,
            usize::from(filled),
            "the flame never reached the top of its fuel",
        );

        // ...and what is above the front is *empty*, not dim fire.
        let igniting = Burn {
            phase: 0.3,
            flare: 0.9,
            lit: true,
        };
        for step in 3..filled {
            let (glyph, depiction) = cell(0, step, filled, false, igniting);
            assert!(
                !depiction.is_flame(),
                "cell {step} was drawn as fire before it caught: {glyph:?}",
            );
        }
    }

    #[test]
    fn a_settled_fire_is_the_full_height_of_its_fuel() {
        // The invariant the growth above must not cost: once the flare is over,
        // the fire meter and the plain meter agree cell for cell about where the
        // value is. `paint` has the cross-check against `meter` itself; this
        // pins the piece that lives here.
        for filled in [1u16, 5, 16] {
            for phase in [0.0, 0.4, 7.3] {
                let lit_cells = (0..filled)
                    .filter(|step| cell(0, *step, filled, false, lit(phase)).1.is_flame())
                    .count();
                assert_eq!(lit_cells, usize::from(filled), "at phase {phase}");
            }
        }
    }

    #[test]
    fn the_plume_thins_to_nothing_as_it_rises() {
        // What "pittering out" has to mean if the empty track is going to stay
        // empty: smoke near the fire, less of it higher, and none at the top —
        // rather than the solid wall of `░` this replaced.
        let (steps, filled) = (40u16, 4u16);
        let mut ink = [0u32; 40];
        for phase in sweep() {
            for step in filled..steps {
                if cell(0, step, filled, false, lit(phase)).0 != ' ' {
                    ink[usize::from(step)] += 1;
                }
            }
        }

        let near: u32 = ink[5..10].iter().sum();
        let far: u32 = ink[20..25].iter().sum();
        let top: u32 = ink[34..40].iter().sum();
        assert!(
            near > far,
            "the plume does not thin: {near} near, {far} far"
        );
        assert_eq!(top, 0, "smoke reached the top of the bar: {top} cells");
    }

    #[test]
    fn the_fire_is_brightest_at_its_base() {
        // The inversion. An earlier version put the hottest cell at the flame
        // *front*, which read as a bar with a bright edge rather than as a fire.
        // Averaged over a cycle because every cell shifts through the ramp — the
        // point is the gradient, not any one frame.
        let (filled, mut base, mut tip) = (16u16, 0u32, 0u32);
        let rank = |depiction| match depiction {
            Depiction::FlameCore => 3,
            Depiction::FlameBlaze => 2,
            Depiction::FlameBody => 1,
            _ => 0,
        };
        for phase in sweep() {
            base += rank(cell(0, 0, filled, false, lit(phase)).1);
            tip += rank(cell(0, filled - 1, filled, false, lit(phase)).1);
        }
        assert!(
            base > tip * 2,
            "the base ({base}) is not decisively hotter than the tip ({tip})",
        );
    }

    #[test]
    fn the_bottom_half_is_solid_and_the_top_frays() {
        // The glyph rule, stated as the two halves it divides the flame into:
        // below the midpoint every cell is `█` at every phase, so all of the
        // motion down there is colour; above it `▓` appears and grows commoner
        // toward the tip.
        let filled = 16u16;
        let mut frayed = [0u32; 16];
        for phase in sweep() {
            for step in 0..filled {
                if cell(0, step, filled, false, lit(phase)).0 == '▓' {
                    frayed[usize::from(step)] += 1;
                }
            }
        }

        for (step, count) in frayed.iter().enumerate().take(usize::from(filled) / 2) {
            assert_eq!(*count, 0, "cell {step} frayed below the halfway line");
        }
        assert!(
            frayed[usize::from(filled) - 1] > frayed[usize::from(filled) / 2 + 1],
            "fraying does not increase toward the tip: {frayed:?}",
        );
    }

    #[test]
    fn sparks_rise_out_of_the_fire_and_go_out() {
        // They have to exist, they have to move, and they have to stop — a spark
        // that rose forever would be a dotted line up the pane.
        let (steps, filled) = (40u16, 6u16);
        let is_spark = |glyph: char| glyph == '∙' || glyph == '°' || glyph == '·';

        let mut seen = 0u32;
        let mut highest = 0u16;
        for phase in sweep() {
            for step in filled..steps {
                if is_spark(cell(0, step, filled, false, lit(phase)).0) {
                    seen += 1;
                    highest = highest.max(step - filled);
                }
            }
        }
        assert!(seen > 0, "no sparks at all");
        assert!(highest >= 3, "sparks never got clear of the fire");
        assert!(
            highest <= 8,
            "a spark rose {highest} cells — it never went out"
        );

        // Never on the pinned cell, whatever else happens.
        for phase in sweep() {
            let glyph = cell(0, filled, filled, false, lit(phase)).0;
            assert_eq!(glyph, ' ', "a spark landed on the boundary cell");
        }
    }

    #[test]
    fn flame_and_smoke_never_share_a_glyph() {
        // Belt to the boundary pin's braces: away from the join the two regions
        // still have to be told apart with no colour, or a bar read in greyscale
        // is a row of noise.
        for phase in sweep() {
            for (index, (glyph, depiction)) in bar(30, 12, phase).into_iter().enumerate() {
                if depiction.is_flame() {
                    assert!(
                        glyph == '█' || glyph == '▓',
                        "cell {index} at {phase} burned as {glyph:?}",
                    );
                } else if depiction.is_smoke() {
                    assert!(
                        glyph == '░' || glyph == '▒',
                        "cell {index} at {phase} smoked as {glyph:?}",
                    );
                } else if depiction.is_spark() {
                    // Sparks are the one thing allowed a third vocabulary — they
                    // are small marks rather than fill, so they cannot be
                    // mistaken for either region however they are coloured.
                    assert!(
                        glyph == '∙' || glyph == '°' || glyph == '·',
                        "cell {index} at {phase} sparked as {glyph:?}",
                    );
                } else {
                    // Air. Most of a plume is, now that the empty track is not a
                    // solid field of `░` — but it has to be *actually* empty, or
                    // the depiction and the glyph disagree about whether there
                    // is anything there.
                    assert_eq!(
                        glyph, ' ',
                        "cell {index} at {phase} is a picture of nothing but draws {glyph:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn the_same_phase_always_draws_the_same_fire() {
        // What keeps `ORBS_DUMP` reproducible, and the reason this hashes rather
        // than reaching for an RNG.
        for phase in [0.0, 0.37, 4.0, 23.9] {
            assert_eq!(bar(24, 11, phase), bar(24, 11, phase));
        }
    }

    #[test]
    fn the_cycle_wraps_without_a_seam() {
        // **The plume has to keep *travelling* through the wrap**, which is a
        // stronger claim than the one this test used to make and the reason it
        // caught nothing. The old version compared `CYCLE_SECS - 0.01` against
        // `0.0` — and `shared_tick` rounds, so `23.99` *is* tick 0. It was
        // comparing a tick against itself and could not have failed.
        //
        // The real hazard is specific to a drift. Everywhere else consecutive
        // ticks are uncorrelated hashes, so the wrap looks like any other
        // boundary; a translating pattern is deliberately correlated, so the
        // `143 → 0` step used to jump its coordinate 143 cells and re-randomise
        // the whole plume in one frame, once every twenty-four seconds. It
        // measured at 24 of 32 cells translating across the wrap against 31 of
        // 32 elsewhere. See `pulse::rising`.
        let wrap = carried(TICKS_PER_CYCLE - 1);
        let interior: Vec<usize> = (10..20u16).map(carried).collect();
        let worst = interior.iter().copied().min().unwrap_or(0);
        assert!(
            wrap >= worst,
            "the plume tore at the wrap: {wrap} cells carried against a worst \
             ordinary tick of {worst} ({interior:?})",
        );
    }

    #[test]
    fn the_plume_travels_through_every_tick_boundary() {
        // The wrap above is one boundary out of 144, and singling one out is how
        // the seam survived the *old* test. Every boundary, so a future change
        // to `CYCLE_SECS`, `FLIP_HZ` or the span arithmetic has nowhere to hide.
        //
        // **Judged against the plume's own typical tick**, not against a fixed
        // number. A floor loose enough to be robust to the noise is loose enough
        // to sit under a torn frame — measured, it was: 21 cells carried at the
        // seam against a two-thirds floor of 19. What a tear actually looks like
        // is one tick well below the rest, so the rest is the yardstick.
        let every: Vec<usize> = (0..TICKS_PER_CYCLE).map(carried).collect();
        let mut sorted = every.clone();
        sorted.sort_unstable();
        let median = sorted[sorted.len() / 2];
        for (tick, carried) in every.iter().enumerate() {
            assert!(
                *carried >= PLUME_FLOOR,
                "the plume churned rather than rising at tick {tick}: \
                 {carried} cells carried, floor {PLUME_FLOOR}",
            );
            assert!(
                *carried + 3 >= median,
                "the plume tore at tick {tick}: {carried} cells carried against \
                 a median tick's {median}",
            );
        }
    }

    /// The bar the two drift tests measure on: enough plume above the flame that
    /// a translation has somewhere to go.
    const DRIFT_BAR: (u16, u16) = (36, 4);

    /// The first plume cell that carries — the pinned blank sits at `filled`, so
    /// the translation starts above it.
    const DRIFT_FROM: usize = DRIFT_BAR.1 as usize + 2;

    /// Cells the measurement covers.
    const DRIFT_CELLS: usize = DRIFT_BAR.0 as usize - 1 - DRIFT_FROM;

    /// Two thirds of them. Far below the ~97% a translating plume actually
    /// manages and far above the ~40% a re-randomised one does, so it separates
    /// the two without pinning the noise.
    const PLUME_FLOOR: usize = 2 * DRIFT_CELLS / 3;

    /// How many plume cells at `tick` reappear one cell **further up** at
    /// `tick + 1`, which is what the plume rising means.
    fn carried(tick: u16) -> usize {
        let (steps, filled) = DRIFT_BAR;
        let now = bar(steps, filled, f32::from(tick) / FLIP_HZ);
        let next = bar(steps, filled, f32::from(tick + 1) / FLIP_HZ);
        (DRIFT_FROM..usize::from(steps) - 1)
            .filter(|s| now[*s] == next[s + 1])
            .count()
    }

    #[test]
    fn a_full_bar_is_all_fire_and_an_empty_one_all_smoke() {
        // The athanor drains to nothing and is kindled back to full, so both
        // ends are states a player sees every session.
        //
        // **The empty end must not spark**, and that is a claim about the world
        // rather than about glyphs: a spent athanor is still `Burning` for the
        // tick before it goes out, so sparks coming off it would say there is
        // fuel left at the exact moment the bar says there is none. It may show
        // the last of its smoke, and mostly shows nothing.
        for phase in sweep() {
            for (glyph, depiction) in bar(16, 16, phase) {
                assert!(depiction.is_flame(), "{glyph:?}");
            }
            for (glyph, depiction) in bar(16, 0, phase) {
                assert!(
                    depiction.is_smoke() || depiction == Depiction::None,
                    "a spent athanor threw {glyph:?}",
                );
            }
        }
    }

    #[test]
    fn the_plume_drifts_rather_than_boiling_in_place() {
        // Smoke hashed on its own position would seethe. The pattern has to
        // translate away from the fire, which is what `drift` buys — so a later
        // frame should look like an earlier one shifted, not like a fresh one.
        // Summed over many phases rather than judged on one pair: `stagger`
        // deliberately puts neighbouring cells on different tick boundaries, so
        // any single frame is a translation *with jitter* and the tendency is
        // the honest thing to measure.
        let steps = 40u16;
        let filled = 4u16;
        let at = |phase: f32| -> Vec<char> {
            (0..steps)
                .map(|s| cell(0, s, filled, false, lit(phase)).0)
                .collect()
        };

        let plume = usize::from(filled) + 1;
        let last = usize::from(steps) - 1;
        let (mut aligned, mut in_place) = (0usize, 0usize);
        for phase in sweep() {
            let first = at(phase);
            let later = at(phase + TICK_SECS);
            aligned += (plume..last).filter(|&i| first[i] == later[i + 1]).count();
            in_place += (plume..last).filter(|&i| first[i] == later[i]).count();
        }
        assert!(
            aligned > in_place,
            "the plume did not move: {aligned} aligned vs {in_place} in place",
        );
    }
}
