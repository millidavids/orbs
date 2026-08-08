//! What a mortar and pestle looks like, working.
//!
//! # Breaking large things into small ones
//!
//! A mortar is not a tool that fills a container; it is a tool that *reduces*.
//! So the bar is one lump of material seen edge-on, and grinding eats it from
//! below: the solid block gives way at its underside, the pieces snow down
//! through a working gap, and they collect as a coarse bed at the bottom.
//!
//! Material is conserved on screen. The block's underside sits a fixed gap above
//! the bed, so as the bed rises the block is *consumed* rather than pushed — the
//! bar is always full of something, and how much of it is broken is the reading.
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
//! **Loaded** is that picture with nothing broken yet — a solid bar of `█`,
//! which is the clearest "there is something in here" the alphabet has. Its
//! ends are not special cases: `charged` is nothing collected, `ready` is
//! everything collected and held until the tool is emptied.
//!
//! # Four things this module is built to hold
//!
//! **The four shades are four states of one substance**, which is what makes the
//! picture legible without a legend: `█` whole, `▒` and `░` in pieces and in the
//! air, `▓` broken and settled. Nothing here is a *tool* — an earlier version put
//! a pestle `■` in the gap, and a glyph from outside the fill vocabulary reads as
//! an object visiting the bar rather than as the material changing state.
//!
//! **The value survives with no colour at all.** The cell directly above the bed
//! only ever holds `░` or air, never `▒`, so the join is 75% against 25% or
//! nothing. `▓` against `▒` is one dither step, which the CRT's phosphor bloom
//! erases, and the meter's value is read off this join.
//!
//! **The fall is slower than the clock.** One cell every [`FALL_EVERY`] ticks
//! rather than one per tick, which puts every cell in the gap at 1.5 flashes a
//! second — under the 3 Hz floor, so unlike the fire this needs no exemption.
//! It also simply looks more like falling: at the full rate the debris streaks.
//!
//! **It is deterministic.** An integer hash of position, never an RNG, so the
//! same phase draws the same fall — which keeps `ORBS_DUMP` reproducible and
//! keeps this out of `orbs-sim`'s seeded streams.
//!
//! # Why this needs no [`Depiction`](crate::Depiction) and the fire did
//!
//! The mortar varies [`Intensity`] where the athanor needed a whole new channel
//! invented for it, and the difference is not that one was done carelessly.
//!
//! [`Intensity`] *is* "the base hue at three weights" (§4) — the one channel
//! ordinary content is allowed to vary on, and the reason the accent triad stays
//! meaningful. A picture drawn entirely in the base hue at different weights is
//! that channel doing its job. The fire could not be: orange and yellow are hues
//! the green phosphor does not contain, so it needed a ramp of its own, and a
//! ramp of its own needed a channel that selects one without claiming to mean
//! anything.
//!
//! The test of the distinction is greyscale. Throw the colour away and the
//! mortar is unharmed — the four shades are four *glyphs*, and every property
//! this module holds is stated about them. Throw it away on the fire and the
//! flame and its sparks go flat, which is why `Depiction` carries no meaning:
//! there is nothing there to lose.

use crate::pulse::{TICKS_PER_CYCLE, drift_noise, falling, salted, shade, shared_tick};
use crate::style::{Intensity, Style};
use crate::tween;

/// Cells of open air between the block's underside and the collected bed.
///
/// The working gap: where broken material is in flight. Fixed, so the block is
/// eaten as the bed rises rather than squeezed.
///
/// Four rather than three. At three a piece is against the bed almost as soon as
/// it exists — there is one cell of clear travel between the block's face and
/// the fine-grade zone, which is not enough for the eye to read it as *falling*
/// rather than as flickering in place.
const FALL_GAP: u16 = 4;

/// Ticks the debris takes to fall one cell.
///
/// **Two, not one.** At one cell per tick the gap turns over six times a second,
/// which is 3 flashes and squarely on the photosensitive floor — the fire is
/// inside that band by a recorded exemption and this does not need to be. It
/// also reads better: at the full rate the pieces streak rather than fall.
const FALL_EVERY: u16 = 2;

/// Cells the debris falls in one whole cycle — the period its coordinate wraps
/// on, so the cycle boundary is one more ordinary fall step rather than a tear.
///
/// **It must divide exactly**, or the last fall of a cycle is a fraction of a
/// cell and the seam comes back. Checked here rather than trusted, because
/// changing [`FALL_EVERY`] to three is an obvious thing to try and the damage
/// would be one torn frame every twenty-four seconds.
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
/// The counterpart to [`Burn`](crate::Burn), and deliberately smaller: a grind
/// has no equivalent of the fire's flare because `grind` starts it on a tick
/// boundary with nothing to announce.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Grind {
    /// Elapsed seconds from the frontend's own clock. Nothing here knows what a
    /// second is; this is only ever compared against itself.
    pub phase: f32,
    /// Whether the block is being worked.
    ///
    /// `false` is a bowl at rest — **loaded** or **finished** depending on how
    /// much is broken, and both are pictures the panel could not draw at all
    /// before, because the sim reports no meter for either. At rest the material
    /// simply settles: no gap, nothing in the air.
    pub working: bool,
    /// How far through the current world tick, `0.0..1.0`.
    ///
    /// **Why a bar can fill faster than the world turns.** The sim advances at
    /// 1 Hz (§5.0) and reports whole ticks, so a meter read straight off it
    /// jumps once a second. But a duration action is *continuous* — an eight-tick
    /// grind is eight seconds of work, and at three and a half seconds it really
    /// is seven-sixteenths done. `3/8` is the sim's 1 Hz **sample** of that, not
    /// the thing itself.
    ///
    /// So the bed creeps between samples, and the drawn value is closer to the
    /// truth than the sample is rather than further from it. `stop` mid-tick is
    /// the one case where the prediction was wrong, and it corrects on the next
    /// frame.
    ///
    /// Nothing here knows what a tick is either — a frontend supplies the
    /// fraction, exactly as it supplies [`Burn::flare`](crate::Burn) and the
    /// pane tween's progress.
    pub advance: f32,
    /// How recently a reagent went in: `1.0` at the instant of loading, decaying
    /// to `0.0` over one world tick.
    ///
    /// Drives the pour — the block builds **up from the floor of the bowl** as
    /// this decays, with a little material still falling above it. The mirror of
    /// the grind, which takes the same block apart from underneath.
    ///
    /// A load is otherwise invisible: `Empty` and `Charged` are both states the
    /// sim reports no meter for, so without this the bar simply *is* a solid
    /// block one frame having been nothing the frame before.
    ///
    /// **`move` reaches it; `grind` does not, and that is right.** `grind sage`
    /// is the move and the wield in one command (§19), so the bowl goes
    /// `Empty → Working` inside a single tick and never rests at `Charged`. The
    /// frontend still arms the pour on that edge and this simply ignores it,
    /// because a pour drawn there would be a beat the world did not have — the
    /// material would appear to settle before work it has already started.
    ///
    /// So the pour is what `move sage to mortar_and_pestle` looks like: a bowl
    /// filling and then standing until something is done with it. That is the
    /// case it was asked for, and the See-it line uses that verb.
    pub load: f32,
    /// Whether what is in the bowl is only leavings.
    ///
    /// `State::Fouled` — the husks a run left behind, which will not start
    /// anything. Drawn as a sparse scatter rather than a solid block, because
    /// **the two must not look alike**: at the panel's narrow layout there is no
    /// state word on screen, so a fouled bowl drawn as a loaded one is the panel
    /// saying an instrument is ready when it is jammed. That was one of the two
    /// defects §10.1's panel was built to remove.
    pub spent: bool,
}

/// One cell of a mortar's bar.
///
/// `step` counts from the end the bar fills from — leftward for a horizontal
/// meter, upward for a vertical one — so both orientations share this and cannot
/// drift apart. `filled` is what `filled_of` computed, so the mortar and the
/// plain meter always agree about where the value is.
pub(crate) fn cell(lane: u16, step: u16, filled: u16, total: u16, work: Grind) -> (char, Style) {
    // What has been broken and settled. The reading, and the one thing here
    // that has to survive with the colour thrown away.
    if step < filled {
        return ('▓', Style::NORMAL);
    }

    // At rest the material sits as it is: no working gap, nothing being broken.
    if !work.working {
        // Leavings, which must not read as a loaded bowl — see `Grind::spent`.
        if work.spent {
            return leavings(lane, step);
        }
        // **Pouring in.** The block builds up from the floor as `load` decays,
        // so a reagent arriving is something the eye can catch rather than a
        // solid bar appearing between two frames. At rest this is the whole bar
        // and every branch below it is unreachable.
        //
        // **Floored at one cell**, the same lesson the fire's flare recorded: at
        // the instant of loading the height is zero, the pieces above it are
        // sparse enough to all miss, and the bowl reads *empty* on the one frame
        // the player is looking for something to have arrived in it.
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

    // The block's underside, a fixed gap above the bed. It recedes as the bed
    // rises, so the block is eaten rather than pushed and the bar stays full of
    // material throughout.
    let underside = filled.saturating_add(FALL_GAP).min(total);
    if step >= underside {
        // The face is where it is giving way — part broken already, and that is
        // where the pieces in the gap below came from.
        return if step == underside {
            ('▓', Style::BRIGHT)
        } else {
            whole(lane, step)
        };
    }

    // In the air. **The cell against the bed takes only the fine grade**: `▓`
    // against `▒` is one dither step and the meter's value is read off that
    // join, so the coarse pieces land a cell higher.
    piece(lane, step, shared_tick(work.phase), step == filled)
}

/// A cell of the unbroken block.
///
/// Faintly uneven rather than a flat wall of `█` — a lump of rock has facets, and
/// at two cells wide a perfectly uniform block reads as a filled rectangle rather
/// than as a thing. It does not move: nothing about the block is animated except
/// where it is being broken.
fn whole(lane: u16, step: u16) -> (char, Style) {
    // **`Normal` against `Bright`, not `Dim` against `Bright`.** The block used
    // the two *ends* of the ramp, which is the widest step it can make — on a
    // tinted bar that is one lump of rock in two visibly different colours,
    // which reads as two substances rather than as one with facets. Adjacent
    // steps say the same thing quietly.
    let intensity = if shade(drift_noise(lane, step)) == 0 {
        Intensity::Normal
    } else {
        Intensity::Bright
    };
    ('█', Style::NORMAL.with_intensity(intensity))
}

/// A cell of a bowl holding nothing but what the last run left behind.
///
/// **Sparse on purpose, and never solid.** A fouled mortar reports no meter, so
/// before this it drew the same full block as a loaded one — and at the panel's
/// narrow layout there is no state word to tell them apart. "A fouled instrument
/// read as *it will not start*" is one of the two complaints §10.1's panel was
/// built to answer, and drawing it as ready re-created that with pictures.
///
/// Still, and drawn from position alone: leavings do not move.
fn leavings(lane: u16, step: u16) -> (char, Style) {
    match shade(salted(lane, step, LITTER)) {
        0 | 1 => (' ', Style::NORMAL),
        2 => ('░', Style::DIM),
        _ => ('▒', Style::DIM),
    }
}

/// A cell of the working gap: a falling piece, or air.
///
/// **The pattern is a function of the fall coordinate and nothing else**, which
/// is what makes it translate rather than churn. Keying the hash to the tick as
/// well — the obvious thing, since it is a moving picture — re-randomises the
/// whole gap every step and the fall stops reading as motion at all. The fire's
/// plume learned this the same way, and the measurement said coin toss.
fn piece(lane: u16, step: u16, tick: u16, against_bed: bool) -> (char, Style) {
    // Wrapped on [`FALL_SPAN`] rather than added straight on: `fallen` runs
    // `71 → 0` at the cycle boundary, and a coordinate that jumped 71 cells
    // there re-randomised the whole gap in one frame every twenty-four seconds.
    // Measured at 26 of 40 gap cells translating across the wrap against 36 of
    // 40 elsewhere. See `pulse::falling`.
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
/// **Not public**, unlike [`fallen_cells`] beside it. It was exported on the
/// symmetry of the two names and has never had a caller outside this file — an
/// API with no callers is unshaped (§15), and a `pub` one in a workspace crate
/// is a promise nobody asked for.
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
            // The creep is `paint`'s and reaches this module only as a different
            // `filled`; the pour and the leavings are the at-rest picture.
            // Nothing in a *working* bowl needs any of the three.
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
        // §14. The bed is `▓` and the cell against it is `░` or air — 75%
        // against 25% or nothing. The join this must never become is `▓`
        // against `▒`: one dither step, which the CRT's bloom erases, and the
        // meter's value is read off exactly here.
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
        // Material is conserved on screen: the bar stays full of *something*,
        // and what changes is how much of it is broken. A block that kept its
        // height and rode upward would leave a growing hole at the top and the
        // bowl would look like it was emptying rather than grinding.
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

        // ...and the gap between block and bed stays the same size throughout,
        // which is what "eaten rather than pushed" means. Counted as the cells
        // in flight — air and pieces — because the block's *face* is a `▓` it
        // shares a glyph with the bed, so "first `█`" is a cell too high.
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
        // well as the position cannot translate, only re-randomise. Measured as
        // a tendency across the cycle, because neighbouring cells do not all
        // turn over on the same instant.
        // **Measured inside the gap only.** An earlier version swept the whole
        // bar and the static block drowned the signal — a wall of `█` matches
        // itself shifted or unshifted, and the two boundary cells then tipped
        // the count the wrong way. What is being claimed is about the four cells
        // that are actually in flight.
        //
        // The bed-adjacent cell is excluded because it is *deliberately* not a
        // translation: coarse pieces render fine down there, so the value's join
        // survives greyscale. See [`cell`].
        let filled = 4u16;
        let interior = usize::from(filled) + 2..usize::from(filled + FALL_GAP);
        let at = |phase: f32| -> Vec<char> {
            bar(16, filled, working(phase))
                .into_iter()
                .map(|(glyph, _)| glyph)
                .collect()
        };

        // **Sampled exactly on ticks, not on a uniform sweep.** `shared_tick`
        // *rounds*, so a phase landing on a half-tick can go either way and
        // adding one fall step then advances the tick by one instead of two —
        // arithmetic in the test rather than a fault in the fall, and it cost
        // two confusing failures to pin down. `tick / FLIP_HZ` lands dead on the
        // tick, where rounding has nothing to decide. Note that a *centre*,
        // `(tick + 0.5) / FLIP_HZ`, is the boundary, not the safe point.
        for tick in 0..TICKS_PER_CYCLE {
            let phase = f32::from(tick) / crate::FLIP_HZ;
            let first = at(phase);
            let later = at(phase + fall_secs());
            for i in interior.clone() {
                // Down the bar is *toward* the bed, so a piece at `s` arrives at
                // `s - 1` exactly one fall step later.
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
        // **The gap above is two cells at one fill**, and that is exactly how
        // the seam survived it: `fallen` ran `71 → 0` at the boundary, jumping
        // the coordinate 71 cells and re-scattering the whole gap in one frame
        // every twenty-four seconds — and with only two cells asserted at one
        // fill, a 14% coincidence was enough to keep the suite green. It was.
        //
        // Sweeping the fill moves the gap to twenty different places in the bar,
        // which is twenty independent draws instead of two. The sibling test
        // `a_bowl_at_rest_is_perfectly_still` records the same lesson in the
        // same file: a property asserted at one end of a range is not asserted.
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
                // The gap's interior, excluding the bed-adjacent cell — which is
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
        // **No tool glyph.** An earlier version put a pestle `■` in the gap, and
        // a mark from outside the fill vocabulary reads as an object visiting
        // the bar rather than as the material changing state. The four shades
        // are four states of one substance, which is the whole picture.
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
        // The two states the sim reports no meter for. Neither is a special
        // case — they are this picture at nothing-broken and everything-broken.
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
        // meter, so without this the bar simply *is* a solid block one frame
        // having been nothing the frame before.
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

        // **Never blank at the instant of loading.** The flare recorded this
        // lesson first: a height of zero with sparse pieces above it can miss
        // every cell, and the bowl reads empty on the one frame the player is
        // looking for something to have arrived in it.
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
        // drew the same solid block a charged one does — and at the side layout
        // there is no state word on screen to tell them apart. "A fouled
        // instrument read as *it will not start*" is one of the two complaints
        // §10.1's panel was built to answer.
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
        // Motion on an idle tool reads as still working, which is the one thing
        // the panel must never say — a player would wait for something that had
        // already happened.
        //
        // **Every fill, not one.** This checked only a *finished* bowl, which is
        // uniformly `▓` and therefore still whatever the code does. A **loaded**
        // one re-scattered once a second for as long as that version lived, and
        // the test sat green through all of it: a property asserted at one end
        // of a range is not asserted.
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
        // **Two different numbers, and mixing them up is easy.** A *transition*
        // is one change; a *flash* — what the 3–30 Hz band is about — is a pair
        // of opposing ones, so the flash rate is half the transition rate. An
        // earlier version of this asserted transitions against the 3 Hz floor,
        // failed, and the failure was the units rather than the design.
        //
        // The fall being slower than the clock is what buys this: at one cell
        // per tick the gap turns over six times a second and the mortar would
        // need the fire's exemption too.
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
