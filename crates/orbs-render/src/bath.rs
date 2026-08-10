//! What a water bath looks like, working.
//!
//! # A vessel filling with tincture
//!
//! The balneum mariae sits over the athanor and digests gently (DESIGN.md
//! §10.1), and what a gentle digestion *does* is draw something out of a reagent
//! and into a liquid. So the bar is **the liquid in the vessel**: it starts as a
//! shallow layer, rises as the extraction proceeds, and stands full until the
//! tincture is taken.
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
//! **That the level is never zero is the whole reason it is drawn this way.** A
//! bar reading *how far along* would draw a charged bath as an empty vessel,
//! which is indistinguishable from an empty one — the invisible-state defect
//! §10.1's panel exists to remove, reached from a new direction. The mortar
//! solved the same problem the same way: its bar is what is in the bowl rather
//! than how much has been done to it.
//!
//! # All of the motion is in the colour
//!
//! **One glyph, never two.** Liquid is `█` from the floor of the vessel to its
//! surface, at every fill and every phase; what moves is how hard each cell is
//! rolling. Doing that in colour alone buys three things:
//!
//! - **The value survives with the colour thrown away** (§14), and it does so
//!   trivially rather than by argument — solid against blank is the strongest
//!   join the alphabet has, the same one the fire's flame front is pinned to.
//! - **Nothing is ambiguous at the boundary.** A glyph that meant *bubble* would
//!   also have to mean *not quite full*, on the one cell the reading is taken
//!   from.
//! - **It is legible at one cell.** The panel's horizontal layout gives each
//!   instrument a single row, and a picture whose motion is textural needs
//!   height to read. This one does not.
//!
//! **How it moves lives in [`liquid`](crate::liquid)**, shared with the flask,
//! because two vessels must not say different things with the same picture.
//! The distinction that module exists for is this instrument's: a bath
//! **bubbles** only while the athanor is lit, and shifts in place when it is not
//! — so the fire going out is visible on the bath without a word being read,
//! which is §10.1's central timing decision made legible.
//!
//! This module decides only *where the liquid is*.
//!
//! # It is deterministic
//!
//! An integer hash of position and tick, never an RNG, so the same phase draws
//! the same bath — which keeps `ORBS_DUMP` reproducible and keeps this out of
//! `orbs-sim`'s seeded streams.

use crate::liquid;
pub(crate) use crate::liquid::Motion;
use crate::style::Depiction;

/// How little of a vessel a charged one shows.
///
/// The reagent is in and nothing has been drawn out of it yet. One cell, because
/// this is the *floor* rather than a reading — the meter says zero and the
/// picture still has to be a picture. `caught` in [`fire`](crate::fire) and the
/// pour in [`grind`](crate::grind) each record the same lesson: a picture that
/// can vanish is not a picture.
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
    /// Why the level can rise faster than the world turns — see
    /// [`Grind::advance`](crate::Grind), which documents the argument in full.
    /// The sim's whole-tick count is a *sample* of a continuous duration, so
    /// interpolating between samples is closer to the truth than holding the
    /// last one.
    pub advance: f32,
    /// Whether the vessel holds nothing but what the last run left.
    ///
    /// `State::Fouled` — sediment. Drawn as a still band of grit lying too low
    /// to be a level, because **the two must not look alike**: at the panel's
    /// narrow layout there is no state word on screen, so a fouled bath drawn as
    /// a charged one is the panel saying an instrument is ready when it is
    /// jammed.
    pub spent: bool,
    /// Whether a run has settled something in the bottom of the vessel.
    ///
    /// **Under the liquid, not instead of it.** Sediment drops out of a
    /// digestion as it goes, so it is there while the bath works and still
    /// there when it finishes — and it settles *within* the liquid, which is
    /// what keeps the value boundary exactly where `filled_of` put it. A band
    /// that displaced the liquid would move the reading by its own depth.
    pub leavings: bool,
    /// Whether bubbles **break** at the surface rather than merely reaching it.
    ///
    /// The alembic's signature, and the only thing separating its picture from
    /// the balneum's. Distilling is a harder boil than a gentle digestion, so
    /// the topmost cell of liquid reaches the brightest step on the bubble beat
    /// — a break at the face.
    ///
    /// **At the surface, and above it where above is up** — see
    /// [`upward`](Self::upward).
    pub breaking: bool,
    /// Whether the bar runs upward, so that past the fill is *above* it.
    ///
    /// # The one thing in this picture that depends on the layout
    ///
    /// Bubbles escaping into the air above the face were asked for, and were
    /// twice refused before that: the panel has two layouts, and in the
    /// horizontal one *above* becomes **rightward** — putting the marks on the
    /// row directly over the athanor's, whose sparks and smoke occupy exactly
    /// that region past *its* fill. Two adjacent rows, sparse marks past the
    /// fill on both, meaning different things.
    ///
    /// In the **upward** layout that objection does not apply: each instrument
    /// is a column, so the athanor's sparks are in the column *beside* the
    /// alembic rather than the row below it, and the air above the alembic's
    /// face is the alembic's own. So the bubbles are drawn there and nowhere
    /// else, and the picture legitimately differs by orientation — because what
    /// is *adjacent* to it differs by orientation, which is the only thing the
    /// objection was ever about.
    ///
    /// The two orientations otherwise share every cell of this, which is the
    /// property `cell` exists to keep. This is the exception, and it is one flag
    /// wide.
    pub upward: bool,
}

/// One cell of a bath's bar.
///
/// `step` counts from the end the bar fills from — leftward for a horizontal
/// meter, upward for a vertical one — so both orientations share this and cannot
/// drift apart. `filled` is what `filled_of` computed, so the bath and the plain
/// meter always agree about where the value is.
///
/// Returns a [`Depiction`] rather than a [`Style`](crate::Style), following
/// [`fire`](crate::fire): a depiction is dropped on any accented cell by
/// [`Style::depicted`](crate::Style::depicted), so a picture built out of a
/// `Style` has to remember never to set a role. One that cannot express a role
/// cannot forget.
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

    // **Never nothing.** The meter reads zero for a charged bath — the sim
    // reports no quantity for it at all — and a vessel drawn empty is
    // indistinguishable from an empty one. See [`FLOOR`].
    let level = filled.max(FLOOR);
    if step >= level {
        // **The air above the face**, where a hard enough boil throws something
        // into it. Only the alembic (`breaking`), only over a lit athanor
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

    // **Settled waste, lying under the liquid.** Sediment drops out of a
    // digestion as it goes, so it is there while the bath works and still there
    // when it finishes — and it settles *within* the liquid rather than
    // displacing it, which is what keeps the value boundary exactly where
    // `filled_of` put it.
    //
    // Only once the vessel has a floor *and* a body: in a bath one cell deep
    // there is no bottom to rest in, and a band that swallowed the whole reading
    // would say the run was fouled when it has barely started.
    if work.leavings && step < SILT && level > SILT {
        return ('▓', Depiction::Sediment);
    }

    let mut roil = liquid::roil(lane, step, work.phase, work.motion);
    // **The break at the face.** The alembic distils, which is a harder boil
    // than the balneum's digestion — so its topmost cell of liquid goes to the
    // brightest step wherever a bubble reached it. Only while bubbling, which
    // means only over a lit athanor; a still surface does not break.
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
        // A **bath**: `working` sets neither `breaking` nor `upward`, so nothing
        // leaves the liquid here. The alembic's air is not empty and its version
        // of this property is `a_bubble_never_touches_the_reading`.
        // §14, and the easiest version of it any instrument has: the level is
        // read off solid-against-blank, which is the strongest join the alphabet
        // has and the one the fire's flame front is also pinned to. Nothing in
        // the liquid varies but its colour, so throwing the colour away costs
        // the picture nothing at all.
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
        // **The defect this picture is shaped around.** The sim reports no meter
        // for a charged bath, so `stand_in` hands it `0/1` — and a bar reading
        // *how far along* would draw nothing, which is exactly what an empty
        // instrument looks like. "You cannot tell what state a thing is in
        // without touching it" is the complaint §10.1's panel was built to
        // answer, and drawing it that way would put it straight back.
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
        // **`Standing` only**, which is the state the rule was written about: an
        // instrument that has not begun must not move, or a player waits for
        // something that has not started. A *finished* bath is a deliberate
        // exception — see `a_finished_bath_settles_rather_than_freezing`.
        //
        // Swept over every fill, because the sibling lesson in `grind` was
        // learned by asserting this at one end of a range and finding the middle
        // moved.
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
        // **§14, and the property the whole picture is pinned to.** The level is
        // solid-against-blank; a mark that reached the face — or that was `█` —
        // would put the value and the picture at odds on the one cell the value
        // is read from. Swept over every fill, because a boundary bug is a bug
        // at one fill and invisible at the rest.
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
        // **What "the bubbles rise faster than the brew" means as a property.**
        // The level climbs `bar / ticks` cells a tick and the bar grows with the
        // window, so no recipe duration can outrun a bubble at every size — at
        // the shipped ones it did not come close, and what that looks like is
        // bubbles being swallowed by the liquid they just left.
        //
        // So the air is measured from the **face**: the same bubbles stand at
        // the same heights above it whatever the level is, which means a rising
        // level carries them rather than catching them. Asserted by holding the
        // phase and moving the fill — if any of this were anchored in absolute
        // space, these would differ.
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
