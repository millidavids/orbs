//! What a mortar and pestle looks like, working.
//!
//! A mortar *reduces* rather than fills, so the bar is one lump of material seen
//! edge-on: the block gives way at its underside, the pieces snow down through a
//! working gap, and they collect as a coarse bed at the bottom.
//!
//! Material is conserved on screen. The underside sits a fixed gap above the
//! bed, so the block is *consumed* rather than pushed — the bar is always full
//! of something, and how much of it is broken is the reading.
//!
//! ```text
//!   charged        working          ...later          ready
//!    ██              ██               ██               ▓▓
//!    ██              ██               ██               ▓▓
//!    ██              ██               ██               ▓▓
//!    ██              ██  ← block      ▓▓  ← crumbling  ▓▓
//!    ██              ▓▓  ← crumbling  ░                ▓▓
//!    ██              ░ ▒              ▒░               ▓▓
//!    ██              ░░  ← falling    ▓▓               ▓▓
//!    ██              ▓▓  ← collected  ▓▓               ▓▓
//!    ██              ▓▓               ▓▓               ▓▓
//! ```
//!
//! The ends are not special cases: `charged` is nothing collected — a solid bar
//! of `█` — and `ready` is everything collected, held until the tool is emptied.
//!
//! Four things this holds:
//!
//! - Four shades, four states of one substance: `█` whole, `▒` and `░` in the
//!   air, `▓` broken and settled. No *tool* — a pestle `■` read as an object
//!   visiting the bar rather than as material changing state.
//! - The value survives with no colour: the cell above the bed holds `░` or
//!   air, never `▒`, so the join is 75% against 25% rather than one dither step
//!   the CRT's bloom erases.
//! - The fall is slower than the clock — one cell every [`FALL_EVERY`] ticks,
//!   1.5 flashes a second — so unlike the fire it needs no exemption.
//! - Deterministic: an integer hash of position, never an RNG, so `ORBS_DUMP`
//!   stays reproducible and this stays out of `orbs-sim`'s seeded streams.
//!
//! No [`Depiction`](crate::Depiction), where the fire needed one: [`Intensity`]
//! is "the base hue at three weights" (§4) and this picture is that channel
//! doing its job, where orange and yellow are hues the green phosphor does not
//! contain. Greyscale is the test — four shades are four *glyphs* and survive
//! it; the flame goes flat.

use crate::pulse::{TICKS_PER_CYCLE, drift_noise, falling, salted, shade, shared_tick};
use crate::style::{Intensity, Style};
use crate::tween;

/// Cells of open air between the block's underside and the collected bed, where
/// broken material is in flight. Fixed, so the block is eaten as the bed rises.
///
/// Four rather than three: at three there is one cell of clear travel, which
/// reads as flickering in place rather than as *falling*.
const FALL_GAP: u16 = 4;

/// Ticks the debris takes to fall one cell.
///
/// Two, not one: at one cell per tick the gap turns over six times a second, 3
/// flashes and squarely on the photosensitive floor. The pieces streak, too.
const FALL_EVERY: u16 = 2;

/// Cells the debris falls in one whole cycle — the period its coordinate wraps
/// on, so the boundary is one more ordinary fall step rather than a tear.
///
/// It must divide exactly, or the last fall is a fraction of a cell and the
/// seam comes back. Checked rather than trusted: [`FALL_EVERY`] at three is an
/// obvious thing to try and would tear a frame every twenty-four seconds.
const FALL_SPAN: u16 = TICKS_PER_CYCLE / FALL_EVERY;
const _: () = assert!(
    TICKS_PER_CYCLE.is_multiple_of(FALL_EVERY),
    "FALL_EVERY must divide the cycle, or the fall tears at the wrap",
);

/// How much of the gap carries a piece at any moment.
///
/// Against a `0..=3` draw, so at two roughly half the cells are occupied. Denser
/// and it is a column of rubble rather than something falling through air.
const DEBRIS: u16 = 2;

/// What the mortar is doing, beyond how much is broken.
///
/// The counterpart to [`Burn`](crate::Burn), and smaller: a grind has no flare,
/// because `grind` starts it on a tick boundary with nothing to announce.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Grind {
    /// Elapsed seconds from the frontend's own clock. Nothing here knows what a
    /// second is; this is only ever compared against itself.
    pub phase: f32,
    /// Whether the block is being worked.
    ///
    /// `false` is a bowl at rest — loaded or finished depending on how much is
    /// broken. The material settles: no gap, nothing in the air.
    pub working: bool,
    /// How far through the current world tick, `0.0..1.0`.
    ///
    /// Why a bar fills faster than the world turns: the sim reports whole ticks
    /// at 1 Hz (§5.0) but a duration action is *continuous*, so the bed creeps
    /// between samples, closer to the truth than the sample. `stop` mid-tick is
    /// the one wrong prediction, and it corrects next frame. A frontend supplies
    /// the fraction, as it supplies [`Burn::flare`](crate::Burn).
    pub advance: f32,
    /// How recently a reagent went in: `1.0` at the instant of loading, decaying
    /// to `0.0` over one world tick.
    ///
    /// Drives the pour — the block builds up from the floor as this decays. A
    /// load is otherwise invisible, `Empty` and `Charged` both reporting no
    /// meter, so the bar would simply *be* a solid block.
    ///
    /// `move` reaches it; `grind` does not, `grind sage` being the move and the
    /// wield in one command (§19), so the bowl never rests at `Charged`. The
    /// See-it line uses `move sage to mortar_and_pestle`.
    pub load: f32,
    /// Whether what is in the bowl is only leavings.
    ///
    /// `State::Fouled` — the husks a run left behind. A sparse scatter rather
    /// than a solid block: the narrow layout has no state word, so a fouled
    /// bowl drawn as a loaded one says ready when it is jammed (§10.1).
    pub spent: bool,
}

/// One cell of a mortar's bar.
///
/// `step` counts from the end the bar fills from — leftward horizontal, upward
/// vertical — so the two orientations cannot drift apart. `filled` is what
/// `filled_of` computed, so the mortar and the plain meter agree on the value.
pub(crate) fn cell(lane: u16, step: u16, filled: u16, total: u16, work: Grind) -> (char, Style) {
    // What has been broken and settled: the reading, and the one thing here that
    // has to survive with the colour thrown away.
    if step < filled {
        return ('▓', Style::NORMAL);
    }

    // At rest the material sits as it is: no working gap, nothing being broken.
    if !work.working {
        // Leavings, which must not read as a loaded bowl — see `Grind::spent`.
        if work.spent {
            return leavings(lane, step);
        }
        // Pouring in: the block builds up from the floor as `load` decays, so a
        // reagent arriving is something the eye can catch. Floored at one cell,
        // the flare's lesson — at zero height the sparse pieces above can all
        // miss and the bowl reads *empty* on the arrival frame.
        let risen = tween::mix(0, total, 1.0 - work.load).max(1);
        if step < risen {
            return whole(lane, step);
        }
        // The last of it still falling. Above that, the bowl is not full yet.
        if step < risen.saturating_add(FALL_GAP) {
            return piece(lane, step, shared_tick(work.phase), false);
        }
        return (' ', Style::NORMAL);
    }

    // The block's underside, a fixed gap above the bed: it recedes as the bed
    // rises, so the block is eaten rather than pushed and the bar stays full.
    let underside = filled.saturating_add(FALL_GAP).min(total);
    if step >= underside {
        // The face is where it is giving way — part broken already, and where
        // the pieces in the gap below came from.
        return if step == underside {
            ('▓', Style::BRIGHT)
        } else {
            whole(lane, step)
        };
    }

    // In the air. The cell against the bed takes only the fine grade: `▓`
    // against `▒` is one dither step and the value is read off that join, so
    // the coarse pieces land a cell higher.
    piece(lane, step, shared_tick(work.phase), step == filled)
}

/// A cell of the unbroken block.
///
/// Faintly uneven rather than a flat wall of `█` — at two cells wide a uniform
/// block reads as a rectangle rather than as a thing. It does not move.
fn whole(lane: u16, step: u16) -> (char, Style) {
    // `Normal` against `Bright`, not `Dim`: the ends of the ramp are the widest
    // step there is, and on a tinted bar that is one lump of rock in two
    // colours — two substances rather than one with facets.
    let intensity = if shade(drift_noise(lane, step)) == 0 {
        Intensity::Normal
    } else {
        Intensity::Bright
    };
    ('█', Style::NORMAL.with_intensity(intensity))
}

/// A cell of a bowl holding nothing but what the last run left behind.
///
/// Sparse on purpose, never solid: a fouled mortar reports no meter, so before
/// this it drew the same full block as a loaded one (§10.1). Drawn from
/// position alone — leavings do not move.
fn leavings(lane: u16, step: u16) -> (char, Style) {
    match shade(salted(lane, step, LITTER)) {
        0 | 1 => (' ', Style::NORMAL),
        2 => ('░', Style::DIM),
        _ => ('▒', Style::DIM),
    }
}

/// A cell of the working gap: a falling piece, or air.
///
/// A function of the fall coordinate and nothing else, which is what makes it
/// translate rather than churn. Keying the hash to the tick as well — the
/// obvious thing for a moving picture — re-randomises the gap every step, as
/// the fire's plume found.
fn piece(lane: u16, step: u16, tick: u16, against_bed: bool) -> (char, Style) {
    // Wrapped on [`FALL_SPAN`] rather than added straight on: `fallen` runs
    // `71 → 0` at the cycle boundary, and the jump re-randomised the whole gap
    // one frame every twenty-four seconds. See `pulse::falling`.
    let fallen = falling(step, tick / FALL_EVERY, FALL_SPAN);
    let grade = shade(salted(lane, fallen, SALT));
    if grade < DEBRIS {
        return (' ', Style::NORMAL);
    }
    // Coarse pieces exist only clear of the bed — see [`cell`].
    if grade > DEBRIS && !against_bed {
        ('▒', Style::NORMAL)
    } else {
        ('░', Style::DIM)
    }
}

/// Keeps the fall's noise clear of the block's facets, which are drawn from the
/// same position with a different function.
const SALT: u32 = 0x51ED_2701;

/// The same, for the scatter of leavings — a third draw from a position the
/// block and the fall both also read.
const LITTER: u32 = 0x2F3B_9A17;

/// How far the debris has fallen at a given phase, in cells. For the example.
#[must_use]
pub fn fallen_cells(phase: f32) -> u16 {
    shared_tick(phase) / FALL_EVERY
}

/// Seconds for the debris to fall one cell.
///
/// Not public, unlike [`fallen_cells`] beside it: an API with no callers is
/// unshaped (§15).
#[cfg(test)]
fn fall_secs() -> f32 {
    f32::from(FALL_EVERY) / crate::pulse::FLIP_HZ
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse::CYCLE_SECS;
    use crate::pulse::harness::{RATE, sweep};

    fn working(phase: f32) -> Grind {
        Grind {
            phase,
            working: true,
            // The creep is `paint`'s and arrives here only as a different
            // `filled`; the pour and the leavings are the at-rest picture.
            ..Grind::default()
        }
    }

    fn bar(total: u16, filled: u16, work: Grind) -> Vec<(char, Style)> {
        (0..total)
            .map(|s| cell(0, s, filled, total, work))
            .collect()
    }

    #[test]
    fn the_value_survives_in_greyscale() {
        // §14. The bed is `▓` and the cell against it `░` or air — 75% against
        // 25% or nothing. Never `▓` against `▒`: one dither step, which the
        // CRT's bloom erases, and the value is read off exactly here.
        for phase in sweep() {
            for filled in 1..15u16 {
                let cells = bar(16, filled, working(phase));
                assert_eq!(cells[usize::from(filled) - 1].0, '▓', "the bed gave way");
                let against = cells[usize::from(filled)].0;
                assert!(
                    against == '░' || against == ' ',
                    "the cell against the bed was {against:?}",
                );
            }
        }
    }

    #[test]
    fn the_block_is_eaten_from_below_rather_than_pushed() {
        // Material is conserved on screen: a block that kept its height and rode
        // upward would leave a hole at the top, and the bowl would look like it
        // was emptying rather than grinding.
        let whole_cells = |filled: u16| {
            bar(16, filled, working(3.0))
                .into_iter()
                .filter(|(glyph, _)| *glyph == '█')
                .count()
        };
        let (early, middle, late) = (whole_cells(1), whole_cells(7), whole_cells(12));
        assert!(
            early > middle && middle > late,
            "the block did not shrink: {early}, {middle}, {late}",
        );

        // ...and the gap stays the same size, which is what "eaten rather than
        // pushed" means. Counted as cells in flight, because the block's *face*
        // shares the bed's `▓` and "first `█`" is a cell too high.
        for filled in 1..11u16 {
            let cells = bar(16, filled, working(3.0));
            let flying = cells
                .iter()
                .skip(usize::from(filled))
                .take_while(|(glyph, _)| matches!(glyph, ' ' | '░' | '▒'))
                .count();
            assert_eq!(
                flying,
                usize::from(FALL_GAP),
                "the working gap moved at fill {filled}",
            );
        }
    }

    #[test]
    fn the_debris_falls_rather_than_churning() {
        // The lesson the fire's plume paid for: a pattern keyed to the tick as
        // well as the position cannot translate, only re-randomise.
        //
        // Measured inside the gap only — a wall of `█` matches itself shifted or
        // not, and drowns the signal. The bed-adjacent cell is excluded because
        // it is deliberately not a translation. See [`cell`].
        let filled = 4u16;
        let interior = usize::from(filled) + 2..usize::from(filled + FALL_GAP);
        let at = |phase: f32| -> Vec<char> {
            bar(16, filled, working(phase))
                .into_iter()
                .map(|(glyph, _)| glyph)
                .collect()
        };

        // Sampled exactly on ticks, not on a uniform sweep: `shared_tick`
        // *rounds*, so a phase on a half-tick advances the tick by one instead
        // of two — arithmetic in the test rather than a fault in the fall.
        for tick in 0..TICKS_PER_CYCLE {
            let phase = f32::from(tick) / crate::FLIP_HZ;
            let first = at(phase);
            let later = at(phase + fall_secs());
            for i in interior.clone() {
                // Down the bar is *toward* the bed, so a piece at `s` arrives
                // at `s - 1` one fall step later.
                assert_eq!(
                    first[i],
                    later[i - 1],
                    "cell {i} did not fall between {phase} and one step later",
                );
            }
        }
    }

    #[test]
    fn the_fall_carries_through_the_cycle_wrap() {
        // The gap is two cells at one fill, which is how the seam survived: a
        // 14% coincidence kept the suite green while `fallen` re-scattered the
        // gap one frame every twenty-four seconds. Sweeping the fill gives
        // twenty independent draws instead of two — a property asserted at one
        // end of a range is not asserted.
        let total = 24u16;
        for tick in 0..TICKS_PER_CYCLE {
            let phase = f32::from(tick) / crate::FLIP_HZ;
            let (mut carried, mut counted) = (0usize, 0usize);
            for filled in 1..21u16 {
                let at = |phase: f32| -> Vec<char> {
                    bar(total, filled, working(phase))
                        .into_iter()
                        .map(|(glyph, _)| glyph)
                        .collect()
                };
                let (first, later) = (at(phase), at(phase + fall_secs()));
                // The gap's interior, less the bed-adjacent cell, which is
                // deliberately not a translation. See the test above.
                for i in usize::from(filled) + 2..usize::from(filled + FALL_GAP) {
                    counted += 1;
                    carried += usize::from(first[i] == later[i - 1]);
                }
            }
            assert_eq!(
                carried, counted,
                "the debris re-scattered rather than falling at tick {tick}",
            );
        }
    }

    #[test]
    fn nothing_but_the_four_shades() {
        // No tool glyph: a pestle `■` in the gap read as an object visiting the
        // bar rather than as the material changing state.
        for phase in sweep() {
            for filled in [0u16, 5, 15] {
                for (glyph, _) in bar(16, filled, working(phase)) {
                    assert!(
                        matches!(glyph, ' ' | '░' | '▒' | '▓' | '█'),
                        "unexpected glyph {glyph:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn a_loaded_bowl_is_solid_and_a_finished_one_is_all_broken() {
        // The two states the sim reports no meter for: this picture at
        // nothing-broken and at everything-broken, not special cases.
        let idle = Grind {
            phase: 3.0,
            ..Grind::default()
        };
        assert!(
            bar(16, 0, idle).iter().all(|(glyph, _)| *glyph == '█'),
            "a loaded bowl was not a solid block",
        );
        assert!(
            bar(16, 16, idle).iter().all(|(glyph, _)| *glyph == '▓'),
            "a finished bowl was not all broken",
        );
    }

    #[test]
    fn a_reagent_pours_in_from_the_floor_of_the_bowl() {
        // A load is otherwise invisible: `Empty` and `Charged` both report no
        // meter, so the bar would simply *be* a solid block between two frames.
        let poured = |load: f32| {
            (0..16)
                .filter(|step| {
                    cell(
                        0,
                        *step,
                        0,
                        16,
                        Grind {
                            phase: 3.0,
                            load,
                            ..Grind::default()
                        },
                    )
                    .0 == '█'
                })
                .count()
        };

        let (start, half, done) = (poured(1.0), poured(0.5), poured(0.0));
        assert!(
            start < half && half < done,
            "the bowl did not fill: {start}, {half}, {done}",
        );
        assert_eq!(done, 16, "a settled bowl is not a full block");

        // Never blank at the instant of loading — the flare's lesson: a height
        // of zero with sparse pieces above it can miss every cell.
        for tenth in 0..=10u16 {
            let load = f32::from(tenth) / 10.0;
            let any = (0..16).any(|step| {
                cell(
                    0,
                    step,
                    0,
                    16,
                    Grind {
                        phase: 3.0,
                        load,
                        ..Grind::default()
                    },
                )
                .0 != ' '
            });
            assert!(any, "the bowl was empty at load {load}");
        }
    }

    #[test]
    fn leavings_never_look_like_a_loaded_bowl() {
        // The defect this exists for: a fouled mortar reports no meter, so it
        // drew the same solid block a charged one does, and the side layout has
        // no state word to tell them apart (§10.1).
        let spent = Grind {
            phase: 3.0,
            spent: true,
            ..Grind::default()
        };
        let loaded = Grind {
            phase: 3.0,
            ..Grind::default()
        };

        let junk = bar(16, 0, spent);
        assert!(
            junk.iter().all(|(glyph, _)| *glyph != '█'),
            "leavings drew as solid material",
        );
        assert!(
            junk.iter().any(|(glyph, _)| *glyph != ' '),
            "a fouled bowl looked empty",
        );
        assert_ne!(junk, bar(16, 0, loaded), "fouled and charged look alike");

        // ...and leavings do not move.
        for phase in sweep() {
            assert_eq!(
                bar(
                    16,
                    0,
                    Grind {
                        phase,
                        spent: true,
                        ..Grind::default()
                    }
                ),
                junk,
                "the leavings shifted at {phase}",
            );
        }
    }

    #[test]
    fn a_bowl_at_rest_is_perfectly_still() {
        // Motion on an idle tool reads as still working, so a player waits for
        // something that has already happened. Every fill, not one: this checked
        // only a *finished* bowl, uniformly `▓` and so still whatever the code
        // does, while a loaded one re-scattered once a second.
        for filled in [0u16, 1, 5, 11, 15, 16] {
            for spent in [false, true] {
                let at = |phase: f32| {
                    bar(
                        16,
                        filled,
                        Grind {
                            phase,
                            spent,
                            ..Grind::default()
                        },
                    )
                };
                let first = at(0.0);
                for phase in sweep() {
                    assert_eq!(
                        at(phase),
                        first,
                        "a resting bowl moved at {phase} (filled {filled}, spent {spent})",
                    );
                }
            }
        }
    }

    #[test]
    fn the_mortar_needs_no_photosensitivity_exemption() {
        // A *transition* is one change; a *flash*, which is what the 3–30 Hz
        // band is about, is a pair of opposing ones, so the flash rate is half
        // the transition rate. The fall being slower than the clock is what
        // buys this: at one cell per tick the mortar would need the fire's
        // exemption too.
        for step in 0..16u16 {
            let mut changes = 0u16;
            let mut last = cell(0, step, 5, 16, working(0.0));
            for n in 1..RATE * 24 {
                let now = cell(0, step, 5, 16, working(f32::from(n) / f32::from(RATE)));
                if now != last {
                    changes += 1;
                    last = now;
                }
            }
            let transitions = f32::from(changes) / CYCLE_SECS;
            assert!(
                transitions <= crate::FLIP_HZ,
                "cell {step} changed {transitions} times a second, over the shared cap",
            );
            assert!(
                transitions / 2.0 < 3.0,
                "cell {step} flashes at {} Hz, inside the band",
                transitions / 2.0,
            );
        }
    }

    #[test]
    fn the_same_phase_always_grinds_the_same_way() {
        for phase in [0.0, 0.37, 4.0, 23.9] {
            assert_eq!(bar(16, 5, working(phase)), bar(16, 5, working(phase)));
        }
    }
}
