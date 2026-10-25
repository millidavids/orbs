//! What a water bath looks like, working.
//!
//! The bar is the liquid in the vessel, rising as the extraction proceeds
//! (§10.1).
//!
//! ```text
//!   charged        working          ...later          ready
//!                                                      ██
//!                                                      ██
//!                                     ██               ██
//!                    ██               ██               ██
//!                    ██               ██               ██
//!    ██              ██               ██               ██
//! ```
//!
//! The level is never zero, so a charged bath is not an empty vessel. All
//! motion is colour over solid `█`, so the value survives greyscale (§14). How
//! it moves is [`liquid`]'s; this decides where the liquid is. An integer
//! hash, never an RNG.

use crate::liquid;
pub(crate) use crate::liquid::Motion;
use crate::style::Depiction;

/// How little of a vessel a charged one shows.
///
/// One cell, a floor, since a picture that can vanish is not one — as
/// [`fire`](crate::fire)'s `caught`.
const FLOOR: u16 = 1;

/// How deep the settled grit of a spent run lies.
///
/// Two cells, and never more: sediment must not read as a bath holding a little
/// liquid, and the thing that separates them is that this does not move and
/// stops well short of any real fill.
const SILT: u16 = 2;

/// What the bath is doing, beyond how full it is.
///
/// The counterpart to [`Burn`](crate::Burn) and [`Grind`](crate::Grind), and
/// smaller than either: a bath has no flare because nothing about it is sudden,
/// and no pour because its charged state is already a picture — a shallow layer
/// standing still, which is what a loaded vessel looks like.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Steep {
    /// Elapsed seconds from the frontend's own clock. Nothing here knows what a
    /// second is; this is only ever compared against itself.
    pub phase: f32,
    /// What the vessel is doing.
    pub motion: Motion,
    /// How far through the current world tick, `0.0..1.0`.
    ///
    /// As [`Grind::advance`](crate::Grind).
    pub advance: f32,
    /// Whether the vessel holds nothing but what the last run left.
    ///
    /// `State::Fouled`: still grit, too low to be a level, since the narrow
    /// panel has no state word to tell it from charged.
    pub spent: bool,
    /// Whether a run has settled something in the bottom of the vessel.
    ///
    /// Under the liquid, not instead of it: sediment settles *within* the
    /// liquid, which keeps the value boundary exactly where `filled_of` put it.
    /// A band that displaced it would move the reading by its own depth.
    pub leavings: bool,
    /// Whether bubbles *break* at the surface rather than merely reaching it.
    ///
    /// The alembic's harder boil: the top cell reaches the brightest step, and
    /// above it where above is [`upward`](Self::upward).
    pub breaking: bool,
    /// Whether the bar runs upward, so that past the fill is *above* it.
    ///
    /// The one layout dependence: sideways, escaping bubbles would sit beside
    /// the athanor's sparks and mean something else.
    pub upward: bool,
}

/// One cell of a bath's bar.
///
/// `step` counts from the filling end, so both orientations share this;
/// `filled` is `filled_of`'s. A [`Depiction`], which cannot express a role.
pub(crate) fn cell(lane: u16, step: u16, filled: u16, work: Steep) -> (char, Depiction) {
    // Leavings first: a spent vessel is not a shallow one, and every branch
    // below would draw it as liquid.
    if work.spent {
        return if step < SILT {
            ('▓', Depiction::Sediment)
        } else {
            (' ', Depiction::None)
        };
    }

    // Never nothing: the meter reads zero for a charged bath, and a vessel drawn
    // empty is indistinguishable from an empty one. See [`FLOOR`].
    let level = filled.max(FLOOR);
    if step >= level {
        // The air above the face, where a hard enough boil throws something into
        // it. Only the alembic (`breaking`), only over a lit athanor
        // (`Bubbling`), and only where above is up — see [`Steep::upward`].
        if work.breaking
            && work.upward
            && work.motion == Motion::Bubbling
            && let Some((glyph, roil)) = liquid::escaping(lane, step - level, work.phase)
        {
            return (glyph, Depiction::liquid(roil));
        }
        return (' ', Depiction::None);
    }

    // Settled waste, lying under the liquid rather than displacing it, which is
    // what keeps the value boundary exactly where `filled_of` put it.
    //
    // Only once the vessel has a floor *and* a body: in a bath one cell deep
    // there is no bottom to rest in, and a band that swallowed the whole reading
    // would say the run was fouled when it has barely started.
    if work.leavings && step < SILT && level > SILT {
        return ('▓', Depiction::Sediment);
    }

    let mut roil = liquid::roil(lane, step, work.phase, work.motion);
    // The break at the face: the alembic distils, a harder boil than a gentle
    // digestion, so its topmost cell goes to the brightest step wherever a
    // bubble reached it. Only while bubbling — a still surface does not break.
    if work.breaking && step + 1 == level && roil != crate::style::Roil::Still {
        roil = crate::style::Roil::Rolling;
    }
    ('█', Depiction::liquid(roil))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse::harness::sweep;

    fn working(phase: f32) -> Steep {
        Steep {
            phase,
            motion: Motion::Bubbling,
            ..Steep::default()
        }
    }

    fn bar(total: u16, filled: u16, work: Steep) -> Vec<(char, Depiction)> {
        (0..total).map(|s| cell(0, s, filled, work)).collect()
    }

    #[test]
    fn the_vessel_is_solid_to_its_surface_and_empty_above() {
        // A bath's join is solid against blank (§14); the alembic's is
        // `a_bubble_never_touches_the_reading`.
        for phase in sweep() {
            for filled in 1..16u16 {
                let cells = bar(16, filled, working(phase));
                for (step, (glyph, _)) in cells.iter().enumerate() {
                    let expected = if step < usize::from(filled) {
                        '█'
                    } else {
                        ' '
                    };
                    assert_eq!(
                        *glyph, expected,
                        "cell {step} of a bath filled {filled} at phase {phase}",
                    );
                }
            }
        }
    }

    #[test]
    fn a_charged_vessel_is_never_an_empty_one() {
        // The defect this picture is shaped around: the sim reports no meter for
        // a charged bath, so `stand_in` hands it `0/1`, and a bar reading *how
        // far along* draws nothing — which is what an empty instrument looks
        // like, and is the complaint §10.1's panel was built to answer.
        for phase in [0.0, 0.4, 7.3] {
            for motion in [Motion::Standing, Motion::Bubbling, Motion::Drifting] {
                let work = Steep {
                    phase,
                    motion,
                    ..Steep::default()
                };
                let cells = bar(16, 0, work);
                assert!(
                    cells.iter().any(|(glyph, _)| *glyph != ' '),
                    "a charged bath drew nothing at phase {phase} ({motion:?})",
                );
            }
        }
    }

    #[test]
    fn a_charged_bath_is_perfectly_still() {
        // An instrument not begun must not move. Every fill.
        for filled in [0u16, 1, 5, 11, 15, 16] {
            let at = |phase: f32| {
                bar(
                    16,
                    filled,
                    Steep {
                        phase,
                        motion: Motion::Standing,
                        ..Steep::default()
                    },
                )
            };
            let first = at(0.0);
            for phase in sweep() {
                assert_eq!(at(phase), first, "a charged bath moved at {phase}");
            }
        }
    }

    #[test]
    fn leavings_never_look_like_a_charged_vessel() {
        // A fouled bath reports no meter, so without its own picture it would
        // draw the same shallow layer a charged one does — and at the side
        // layout there is no state word on screen to tell them apart.
        let spent = Steep {
            phase: 3.0,
            spent: true,
            ..Steep::default()
        };
        let charged = Steep {
            phase: 3.0,
            ..Steep::default()
        };

        let grit = bar(16, 0, spent);
        assert!(
            grit.iter().any(|(glyph, _)| *glyph != ' '),
            "a fouled bath looked empty",
        );
        assert!(
            grit.iter().all(|(glyph, _)| *glyph != '█'),
            "sediment drew as liquid",
        );
        assert_ne!(grit, bar(16, 0, charged), "fouled and charged look alike");

        // ...and sediment does not move, whatever the phase or the fill says.
        for phase in sweep() {
            for filled in [0u16, 4, 16] {
                assert_eq!(
                    bar(
                        16,
                        filled,
                        Steep {
                            phase,
                            spent: true,
                            ..Steep::default()
                        }
                    ),
                    grit,
                    "the sediment stirred at {phase} (filled {filled})",
                );
            }
        }
    }

    #[test]
    fn the_same_phase_always_draws_the_same_bath() {
        for phase in [0.0, 0.37, 4.0, 23.9] {
            assert_eq!(bar(16, 5, working(phase)), bar(16, 5, working(phase)));
        }
    }

    /// An alembic distilling over a lit athanor, in the upward layout.
    fn distilling(phase: f32) -> Steep {
        Steep {
            phase,
            motion: Motion::Bubbling,
            breaking: true,
            upward: true,
            ..Steep::default()
        }
    }

    #[test]
    fn the_boil_throws_bubbles_into_the_air_above_the_face() {
        // The alembic's picture, and the thing that separates it from a bath at
        // more than one cell: a distillation is hard enough that some of what
        // rises gets *out*.
        let seen: Vec<(char, Depiction)> = sweep()
            .flat_map(|phase| {
                (8..16u16)
                    .map(move |step| cell(0, step, 8, distilling(phase)))
                    .collect::<Vec<_>>()
            })
            .filter(|(glyph, _)| *glyph != ' ')
            .collect();

        assert!(!seen.is_empty(), "nothing ever left the liquid");
        // In the liquid's own colours, never the fire's. The two share these
        // glyphs and sit in neighbouring columns, so the ramp is what tells a
        // bubble from a spark.
        for (glyph, depiction) in &seen {
            assert!(matches!(glyph, '°' | '·'), "{glyph:?} is not a bubble");
            assert!(
                matches!(
                    depiction,
                    Depiction::LiquidStill | Depiction::LiquidStirred | Depiction::LiquidRolling
                ),
                "a bubble drew in {depiction:?}, which is not liquid",
            );
        }
    }

    #[test]
    fn a_bubble_never_touches_the_reading() {
        // §14, and the property the whole picture is pinned to: the level is
        // solid-against-blank, so a mark reaching the face — or one that was
        // `█` — puts the value and the picture at odds on the one cell the value
        // is read from. Swept over every fill, because a boundary bug shows at
        // one fill and is invisible at the rest.
        for phase in sweep() {
            for filled in 1..16u16 {
                for step in 0..16u16 {
                    let (glyph, _) = cell(0, step, filled, distilling(phase));
                    if step < filled {
                        assert_eq!(glyph, '█', "the liquid broke up at {step} of {filled}");
                    } else {
                        assert_ne!(glyph, '█', "a bubble read as fill at {step} of {filled}");
                    }
                }
                // ...and the topmost liquid cell is still liquid, so the join is
                // exactly where `filled_of` put it.
                assert_eq!(cell(0, filled - 1, filled, distilling(phase)).0, '█');
            }
        }
    }

    #[test]
    fn nothing_escapes_from_a_vessel_that_is_not_a_boiling_alembic() {
        // Three gates, and each is a different picture being protected: the
        // balneum's gentle digestion (`breaking`), a vessel whose fire has gone
        // out (`Bubbling`), and the horizontal layout, where *above* would be
        // rightward and land on the athanor's row (`upward`).
        for phase in sweep() {
            for (name, work) in [
                (
                    "a balneum",
                    Steep {
                        breaking: false,
                        ..distilling(phase)
                    },
                ),
                (
                    "a cold alembic",
                    Steep {
                        motion: Motion::Drifting,
                        ..distilling(phase)
                    },
                ),
                (
                    "a still alembic",
                    Steep {
                        motion: Motion::Standing,
                        ..distilling(phase)
                    },
                ),
                (
                    "the sideways layout",
                    Steep {
                        upward: false,
                        ..distilling(phase)
                    },
                ),
            ] {
                for step in 8..16u16 {
                    assert_eq!(
                        cell(0, step, 8, work).0,
                        ' ',
                        "{name} threw a bubble at {step}, phase {phase}",
                    );
                }
            }
        }
    }

    #[test]
    fn the_rising_face_can_never_overtake_a_bubble() {
        // "The bubbles rise faster than the brew" as a property. A level that
        // outran them looks like bubbles being swallowed by the liquid they just
        // left, so the air is measured from the *face*: the same bubbles stand
        // at the same heights above it whatever the level, and a rising level
        // carries them rather than catching them. Asserted by holding the phase
        // and moving the fill — anchored in absolute space these would differ.
        for phase in sweep() {
            let air = |filled: u16| -> Vec<char> {
                (0..=crate::liquid::CARRY + 1)
                    .map(|ahead| cell(0, filled + ahead, filled, distilling(phase)).0)
                    .collect()
            };
            let first = air(1);
            for filled in 2..24u16 {
                assert_eq!(
                    air(filled),
                    first,
                    "the air above the face changed with the level at phase {phase}",
                );
            }
        }
    }

    #[test]
    fn a_bubble_pops_rather_than_climbing_out_of_the_vessel() {
        // A mark that kept going would stop being a bubble and start being a
        // plume — which the athanor next door already draws, and means something
        // else. `CARRY` is the ceiling and this is what holds it.
        for phase in sweep() {
            for step in 8..24u16 {
                let (glyph, _) = cell(0, step, 8, distilling(phase));
                if step - 8 > crate::liquid::CARRY {
                    assert_eq!(
                        glyph,
                        ' ',
                        "a bubble got {} cells up at phase {phase}",
                        step - 8,
                    );
                }
            }
        }
    }
}
