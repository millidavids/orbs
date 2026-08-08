//! What two things becoming one looks like.
//!
//! # The bar is the flask, and it is divided
//!
//! `flask_and_rod` is §10.1's only instrument that takes **two** inputs, and
//! that is the whole picture: the vessel starts as two unmixed bands, and the
//! mixture grows from the floor as both of them are used up.
//!
//! ```text
//!   charged        working          ...later          ready
//!    ██  b          ██  b            ██  b             ██
//!    ██  b          ██  b            ▓▓                ██
//!    ██  b          ▓▓               ██  a             ██   all
//!    ▓▓             ██  a            ██                ██   mixed
//!    ██  a          ██               ██  mixed         ██
//!    ██  a          ██  mixed        ██                ██
//!    ██  a          ██               ██                ██
//! ```
//!
//! Glyphs never change — `█` throughout, exactly as the bath. **All three bands
//! are told apart by colour and by nothing else**, which sounds like it breaks
//! §14 and does not: what the bar has to carry without colour is its *value*, and
//! the value is where the mixture ends. That boundary is the one thing here that
//! is also a glyph edge, because above it the vessel is `▓` and below it is `█`.
//!
//! # Two ingredients, and neither is favoured
//!
//! The unmixed remainder splits evenly, the lower band being the first input and
//! the upper the second. Both shrink at the same rate as the mixture climbs, so
//! at every moment the bar is full of material and what changes is how much of it
//! has combined — the same conservation the mortar holds, arrived at for the same
//! reason.
//!
//! **Order comes from the world, not from the recipe.** The bands are in the
//! order the flask holds its contents, which is the order `survey` lists them,
//! so what the player sees on the panel and what they read in the transcript
//! agree.
//!
//! # The mixture has texture
//!
//! It carries the bath's roil ([`Roil`]) — liquids being combined are not flat —
//! at the same slow tempo, so the two liquid instruments read as the same kind of
//! thing. The unmixed bands do **not** move: nothing is happening to them yet,
//! and motion there would say the whole vessel was working when only part of it
//! is.

use crate::liquid;
pub(crate) use crate::liquid::Motion;
use crate::style::Depiction;

/// Which band of the flask a cell belongs to.
///
/// Returned alongside the glyph so a painter can tint the three separately —
/// they are three *materials*, and the whole picture is that two of them are
/// becoming the third.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// The first input, nearest the mixture.
    First,
    /// The second input, above it.
    Second,
    /// What they have combined into, growing from the floor.
    Mixed,
}

/// What the flask is doing, beyond how far along it is.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Stir {
    /// Elapsed seconds from the frontend's own clock.
    pub phase: f32,
    /// What the mixture is doing.
    ///
    /// **Never [`Motion::Bubbling`]**, however hard it is working: nothing heats
    /// a flask, and bubbles are what heat looks like. See
    /// [`liquid`](crate::liquid).
    pub motion: Motion,
    /// How far through the current world tick, `0.0..1.0`. See
    /// [`Grind::advance`](crate::Grind).
    pub advance: f32,
    /// Whether the vessel holds nothing but the last run's dregs.
    pub spent: bool,
    /// Whether a run has settled dregs in the bottom of the vessel.
    ///
    /// See [`Steep::leavings`](crate::Steep) — the same band, for the same
    /// reason, drawn under the mixture rather than in place of it.
    pub leavings: bool,
}

/// How deep the dregs of a spent run lie. The bath's [`SILT`](crate::bath), and
/// the same reasoning: too low to be mistaken for a reading.
const LEES: u16 = 2;

/// The least of a vessel that a charged one shows. See
/// [`bath::FLOOR`](crate::bath).
const FLOOR: u16 = 1;

/// One cell of a flask's bar.
///
/// `step` counts from the end the bar fills from, `total` is the whole bar, and
/// `filled` is what `filled_of` computed — so the flask and the plain meter
/// always agree about where the value is.
pub(crate) fn cell(
    lane: u16,
    step: u16,
    filled: u16,
    total: u16,
    work: Stir,
) -> (char, Depiction, Option<Band>) {
    // Leavings first, exactly as the bath: a spent vessel is not a partly-mixed
    // one, and every branch below would draw it as material.
    if work.spent {
        return if step < LEES {
            ('▓', Depiction::Sediment, None)
        } else {
            (' ', Depiction::None, None)
        };
    }

    // **Never nothing.** A charged flask reports no meter, so without a floor it
    // would draw an empty vessel — which is what an *empty* flask looks like.
    let mixed = filled.max(FLOOR).min(total);
    if work.leavings && step < LEES && mixed > LEES {
        return ('▓', Depiction::Sediment, Some(Band::Mixed));
    }
    if step < mixed {
        return (
            '█',
            Depiction::liquid(liquid::roil(lane, step, work.phase, work.motion)),
            Some(Band::Mixed),
        );
    }
    if step >= total {
        return (' ', Depiction::None, None);
    }

    // What is left splits evenly, the first input under the second. The halves
    // are computed from the *remainder* so both shrink together as the mixture
    // climbs, rather than one being eaten before the other is touched.
    let remaining = total - mixed;
    let boundary = mixed + remaining.div_ceil(2);
    // **`▓` at the top of the first band.** The bands are told apart by colour,
    // which §14 will not let carry the *value* — so the one boundary that is the
    // reading gets a glyph too. This is that boundary seen from below: the
    // mixture's surface is where `█` meets `▓`.
    let band = if step < boundary {
        Band::First
    } else {
        Band::Second
    };
    ('▓', Depiction::None, Some(band))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse::harness::sweep;

    fn working(phase: f32) -> Stir {
        Stir {
            phase,
            motion: Motion::Stirring,
            ..Stir::default()
        }
    }

    fn bar(total: u16, filled: u16, work: Stir) -> Vec<(char, Depiction, Option<Band>)> {
        (0..total)
            .map(|s| cell(0, s, filled, total, work))
            .collect()
    }

    #[test]
    fn the_vessel_is_always_full_of_something() {
        // The mortar's conservation rule, and the flask's for the same reason:
        // what changes is how much has *combined*, not how much is there. A bar
        // that emptied as it worked would say the flask was losing material.
        for phase in [0.0, 3.0, 11.5] {
            for filled in 0..=16u16 {
                let cells = bar(16, filled, working(phase));
                assert!(
                    cells.iter().all(|(glyph, _, _)| *glyph != ' '),
                    "the flask had a hole in it at fill {filled}",
                );
            }
        }
    }

    #[test]
    fn the_mixture_grows_from_the_floor_and_both_inputs_shrink() {
        // The picture the whole instrument is for. Neither input may be consumed
        // before the other is touched — that would read as one reagent being
        // used up and then the next, which is not what combining is.
        let count = |filled: u16, want: Band| {
            bar(16, filled, working(4.0))
                .into_iter()
                .filter(|(_, _, band)| *band == Some(want))
                .count()
        };
        let (early, late) = (4u16, 12u16);
        assert!(
            count(early, Band::Mixed) < count(late, Band::Mixed),
            "the mixture did not grow",
        );
        for input in [Band::First, Band::Second] {
            assert!(
                count(early, input) > count(late, input),
                "{input:?} did not shrink as the mixture grew",
            );
        }
        // ...and they shrink together rather than one at a time.
        for filled in 0..14u16 {
            let (first, second) = (count(filled, Band::First), count(filled, Band::Second));
            assert!(
                first.abs_diff(second) <= 1,
                "the two inputs are {first} and {second} at fill {filled} — one \
                 is being used up before the other",
            );
        }
    }

    #[test]
    fn the_value_survives_in_greyscale() {
        // §14. Three bands told apart by colour is fine; the *reading* being
        // told apart by colour is not. The mixture's surface is `█` against `▓`
        // — a whole coverage step — at every fill.
        for phase in sweep() {
            for filled in 1..16u16 {
                let cells = bar(16, filled, working(phase));
                assert_eq!(
                    cells[usize::from(filled) - 1].0,
                    '█',
                    "the mixture's top cell was not solid",
                );
                assert_eq!(
                    cells[usize::from(filled)].0,
                    '▓',
                    "the cell above the mixture was not the unmixed shade",
                );
            }
        }
    }

    #[test]
    fn a_charged_flask_is_never_an_empty_one() {
        // The bath's lesson, and the same trap: the sim reports no meter for a
        // charged instrument, so a bar reading *how far along* draws nothing.
        for motion in [Motion::Standing, Motion::Stirring, Motion::Drifting] {
            let cells = bar(
                16,
                0,
                Stir {
                    phase: 2.0,
                    motion,
                    ..Stir::default()
                },
            );
            assert!(cells.iter().any(|(_, _, band)| *band == Some(Band::Mixed)));
        }
    }

    #[test]
    fn only_the_mixture_moves() {
        // Motion in the unmixed bands would say the whole vessel was working
        // when only part of it is — and on an instrument whose entire subject is
        // *which part has combined*, that is the one thing it must not say.
        let at = |phase: f32| {
            bar(16, 6, working(phase))
                .into_iter()
                .filter(|(_, _, band)| *band != Some(Band::Mixed))
                .collect::<Vec<_>>()
        };
        let first = at(0.0);
        for phase in sweep() {
            assert_eq!(at(phase), first, "an unmixed band moved at {phase}");
        }

        // ...and the mixture does, or it is not an animation.
        let mixture = |phase: f32| {
            bar(16, 6, working(phase))
                .into_iter()
                .filter(|(_, _, band)| *band == Some(Band::Mixed))
                .collect::<Vec<_>>()
        };
        let still = mixture(0.0);
        assert!(
            sweep().any(|phase| mixture(phase) != still),
            "the mixture never moved",
        );
    }

    #[test]
    fn a_flask_at_rest_is_perfectly_still() {
        for filled in [0u16, 1, 8, 16] {
            let at = |phase: f32| {
                bar(
                    16,
                    filled,
                    Stir {
                        phase,
                        ..Stir::default()
                    },
                )
            };
            let first = at(0.0);
            for phase in sweep() {
                assert_eq!(at(phase), first, "a resting flask moved at {phase}");
            }
        }
    }

    #[test]
    fn leavings_never_look_like_a_charged_flask() {
        let spent = Stir {
            phase: 3.0,
            spent: true,
            ..Stir::default()
        };
        let dregs = bar(16, 0, spent);
        assert!(dregs.iter().any(|(glyph, _, _)| *glyph != ' '));
        assert!(dregs.iter().all(|(glyph, _, _)| *glyph != '█'));
        assert_ne!(
            dregs,
            bar(
                16,
                0,
                Stir {
                    phase: 3.0,
                    ..Stir::default()
                }
            ),
        );
    }
}
