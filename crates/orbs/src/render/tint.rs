//! What a material's colour family looks like.
//!
//! The colours a [`Tint`] resolves to. `orbs-render` says *this region holds
//! something green*; this is the only place that decides what green is — the
//! same rule 2 boundary [`ember`](super::ember) sits on.
//!
//! # One base per tint, and the ramp is derived
//!
//! Eight tints at three steps is twenty-four colours, and twenty-four
//! hand-picked numbers is twenty-four chances for a ramp to sag in the middle or
//! inv&#101;rt. So each tint is **one** colour and the steps are scalings of it:
//! monotonic by construction, and eight numbers to tune rather than
//! twenty-four.
//!
//! The scalings were solved rather than chosen. [`DIM`] began at `0.66`, which
//! put brown's dimmest step at 2.78:1 against the tightest background — under
//! the 3.0 floor `even_dim_text_stays_visible` holds everything to. `0.74` clears
//! it at 3.28:1 with room, and the test below is what keeps it clear.
//!
//! # Three steps, serving two different pictures
//!
//! The same ramp answers both instruments that use it, because both ask the same
//! question three ways:
//!
//! | Picture | Step 0 | Step 1 | Step 2 |
//! |---|---|---|---|
//! | The mortar, which varies [`Intensity`] | `Dim` | `Normal` | `Bright` |
//! | The bath, which varies [`Roil`](orbs_render::Roil) | `Still` | `Stirred` | `Rolling` |
//!
//! That is not a coincidence worth being clever about — it is why a material's
//! colour is expressible as a ramp at all. A picture that needed four steps would
//! need its own table.

use bevy::prelude::*;
use orbs_render::{Depiction, Intensity, Tint, Wash};

use super::palette::rgb;

/// How far below the base the dimmest step sits.
///
/// **Narrow on purpose.** These began at `0.74`/`1.22`, which put one ramp step
/// at 1.92× the luminance of the last — a *material* changing shade that hard
/// reads as two substances rather than as one catching the light. At
/// `0.86`/`1.10` a step is 1.39×: still plainly visible, no longer a strobe on a
/// bar you are meant to glance at.
///
/// Narrowing is free on contrast, and that is worth stating because it looks
/// like it should cost: the tightest ratio is always the *dim* step against the
/// background, and raising the floor raises it. 3.28:1 became 4.17:1.
const DIM: f32 = 0.86;

/// How far above the base the brightest step sits.
///
/// Clamped per channel, so a tint that is already near white simply stops rather
/// than wrapping — which is what keeps `bone` a colour instead of a flat white.
const BRIGHT: f32 = 1.10;

/// A tint's three steps, from its one authored colour.
const fn ramp(base: Srgba) -> [Srgba; 3] {
    [scaled(base, DIM), base, scaled(base, BRIGHT)]
}

/// One channel-wise scaling, clamped at white.
const fn scaled(base: Srgba, by: f32) -> Srgba {
    const fn lift(channel: f32, by: f32) -> f32 {
        let raised = channel * by;
        if raised > 1.0 { 1.0 } else { raised }
    }
    Srgba::new(
        lift(base.red, by),
        lift(base.green, by),
        lift(base.blue, by),
        1.0,
    )
}

/// The base colour of each family.
///
/// **Kept apart from one another as well as visible.** The closest pair is
/// `grey`/`violet`; anything nearer would make two materials indistinguishable
/// on a bar two cells wide, which is the whole point of the channel.
const fn base_of(tint: Tint) -> Srgba {
    match tint {
        // **Muted, and the most-looked-at tint in the game.** Sage is the first
        // reagent a player grinds and the one every tutorial path runs through,
        // so it is on screen more than the other seven together. A saturated
        // green at that duty cycle reads as a highlight rather than as a herb.
        Tint::Green => rgb(0.46, 0.62, 0.42),
        Tint::Brown => rgb(0.66, 0.50, 0.30),
        Tint::Grey => rgb(0.66, 0.68, 0.72),
        Tint::Gold => rgb(0.92, 0.78, 0.34),
        Tint::Violet => rgb(0.74, 0.56, 0.95),
        Tint::Red => rgb(0.93, 0.38, 0.38),
        Tint::Blue => rgb(0.42, 0.70, 0.94),
        // **A cool off-white, not a cream.** This is rock salt before anything
        // else, and salt is crystalline — the warm bone it started as read as
        // old paper next to the gold of a finished potion, which are the two
        // pale things on the panel and the two that most need telling apart.
        Tint::Bone => rgb(0.90, 0.90, 0.86),
    }
}

/// The colour a wash starts from — one family, or two averaged.
///
/// **An average, not a third colour picked by hand.** The flask's growing band
/// is two materials *becoming* one, and the eye reads a colour that sits between
/// its neighbours as a mixture of them. A separately-authored third colour would
/// read as a substitution — the vessel's contents being swapped rather than
/// combined — which is the opposite of the one thing that instrument does.
///
/// It is also why the average is taken of the **bases** rather than of the
/// resolved steps: the ramp is then derived from the mixture, so the mixed band
/// has its own three steps and roils in its own colour like any other liquid.
const fn combined(wash: Wash) -> Srgba {
    let first = base_of(wash.tint);
    match wash.with {
        None => first,
        Some(second) => {
            let second = base_of(second);
            Srgba::new(
                (first.red + second.red) / 2.0,
                (first.green + second.green) / 2.0,
                (first.blue + second.blue) / 2.0,
                1.0,
            )
        }
    }
}

/// The colour a tinted cell draws in, or `None` if the tint does not apply.
///
/// # What a tint may not paint over
///
/// **An accent, ever.** §4 reserves the triad strictly for meaning and a tint
/// means nothing on its own, so a `Fouled` instrument's red label stays red even
/// though its bar is full of brown husks. This is the same rule
/// [`Style::depicted`](orbs_render::Style::depicted) enforces for pictures, and
/// it has to be enforced twice because they are two channels.
///
/// **A fire.** The athanor burns orange on every tube by decision (§19) and a
/// tinted hearth would put the four-ramp problem straight back — a green fire
/// reads as a meter that changed colour, not as a fire. Fuel has a tint in the
/// data like everything else; the flame simply does not consult it.
///
/// What it *does* colour is the material itself: the mortar's block and dust
/// through [`Intensity`], the bath's liquid and sediment through
/// [`Depiction`].
pub(crate) const fn resolve(
    wash: Wash,
    role: orbs_render::Role,
    intensity: Intensity,
    depiction: Depiction,
) -> Option<Srgba> {
    // An accent is a signal; a tint is a hint. The signal wins.
    if !matches!(role, orbs_render::Role::Normal) {
        return None;
    }

    let ramp = ramp(combined(wash));
    match depiction {
        // The bath: its own three steps, which happen to be this ramp's.
        Depiction::LiquidStill => Some(ramp[0]),
        Depiction::LiquidStirred => Some(ramp[1]),
        Depiction::LiquidRolling => Some(ramp[2]),
        // **Waste declines the tint, so waste always looks like waste.**
        // Sediment lies in the bottom of a vessel while it works, under liquid
        // that is whatever colour its contents are — and settled grit that took
        // the *contents'* colour would read as more of the same substance
        // rather than as the thing that has dropped out of it. It falls through
        // to `ember::SEDIMENT`, one muted colour for every vessel.
        Depiction::Sediment => None,
        // **The fire is not a material.** See this function's docs.
        Depiction::FlameEmber
        | Depiction::FlameBody
        | Depiction::FlameBlaze
        | Depiction::FlameCore
        | Depiction::SmokeThin
        | Depiction::SmokeThick
        | Depiction::SparkEmber
        | Depiction::SparkBody
        | Depiction::SparkBlaze
        | Depiction::SparkCore => None,
        // The mortar, and anything else drawn as plain material: the cell's own
        // weight picks the step.
        Depiction::None => Some(match intensity {
            Intensity::Dim => ramp[0],
            Intensity::Normal => ramp[1],
            Intensity::Bright => ramp[2],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::super::palette::{ALL, contrast, luminance};
    use super::*;
    use orbs_render::Role;

    #[test]
    fn every_tint_climbs_and_reads_on_every_tube() {
        // One ramp per tint for four themes, so each has to clear the floor
        // against all four backgrounds — the same bar the fire and the bath's
        // default liquid are held to.
        for tint in Tint::ALL {
            let ramp = ramp(base_of(tint));
            for pair in ramp.windows(2) {
                assert!(
                    luminance(pair[0]) < luminance(pair[1]),
                    "{}'s ramp is not monotonic",
                    tint.name(),
                );
            }
            for theme in ALL {
                for (step, colour) in ramp.iter().enumerate() {
                    let ratio = contrast(*colour, theme.background);
                    assert!(
                        ratio >= 3.0,
                        "{} step {step} contrasts {ratio:.2}:1 on {}",
                        tint.name(),
                        theme.name,
                    );
                }
            }
        }
    }

    #[test]
    fn no_two_tints_look_alike() {
        // A channel whose values cannot be told apart is not a channel. Two cells
        // is all the side layout gives an instrument, so the separation has to
        // survive at a glance rather than under comparison.
        let apart = |a: Srgba, b: Srgba| {
            let (r, g, bl) = (a.red - b.red, a.green - b.green, a.blue - b.blue);
            (r * r + g * g + bl * bl).sqrt()
        };
        for (index, tint) in Tint::ALL.into_iter().enumerate() {
            for other in Tint::ALL.into_iter().skip(index + 1) {
                let gap = apart(base_of(tint), base_of(other));
                assert!(
                    gap > 0.2,
                    "{} and {} are {gap:.3} apart — indistinguishable on a \
                     two-cell bar",
                    tint.name(),
                    other.name(),
                );
            }
        }
    }

    #[test]
    fn a_tint_never_paints_over_an_accent() {
        // §4's rule, enforced a second time because this is a second channel.
        // A fouled instrument's label is red *and* its bar is full of brown
        // husks; the label must stay red.
        for tint in Tint::ALL {
            for role in [Role::Danger, Role::Cost, Role::Success] {
                assert_eq!(
                    resolve(Wash::plain(tint), role, Intensity::Normal, Depiction::None),
                    None,
                    "{} painted over {role:?}",
                    tint.name(),
                );
            }
            assert!(
                resolve(
                    Wash::plain(tint),
                    Role::Normal,
                    Intensity::Normal,
                    Depiction::None
                )
                .is_some(),
                "{} did not colour ordinary material",
                tint.name(),
            );
        }
    }

    #[test]
    fn a_tint_never_recolours_the_fire() {
        // The athanor burns orange on every tube by decision (§19). Fuel is a
        // material with a tint in the data like anything else, so this is the
        // only thing stopping a hearth full of charcoal drawing grey flames —
        // which would put the four-ramp problem back in a new costume.
        for tint in Tint::ALL {
            for depiction in Depiction::ALL {
                let is_fire = depiction.is_flame() || depiction.is_spark() || depiction.is_smoke();
                if is_fire {
                    assert_eq!(
                        resolve(
                            Wash::plain(tint),
                            Role::Normal,
                            Intensity::Normal,
                            depiction
                        ),
                        None,
                        "{} tinted {depiction:?}",
                        tint.name(),
                    );
                }
            }
        }
    }

    #[test]
    fn every_tint_resolves_at_every_step() {
        // `Depiction::ALL` is exhaustiveness-checked at its definition, so this
        // walks the whole channel rather than a list that can fall behind.
        for tint in Tint::ALL {
            for depiction in Depiction::ALL {
                // **`Sediment` is not on this list**, deliberately: settled
                // waste declines the tint so that waste always looks like waste
                // rather than like more of whatever the vessel held. See
                // `resolve`.
                let material = depiction.is_liquid() || depiction == Depiction::None;
                assert_eq!(
                    resolve(
                        Wash::plain(tint),
                        Role::Normal,
                        Intensity::Normal,
                        depiction
                    )
                    .is_some(),
                    material,
                    "{} on {depiction:?}",
                    tint.name(),
                );
            }
        }
    }

    #[test]
    fn a_name_round_trips_and_a_typo_does_not_resolve() {
        // Authored data selects a tint by name, and an unknown name has to be a
        // *reported* error rather than a silent fallback — a typo that quietly
        // drew the base hue would look exactly like a material nobody had
        // tinted yet. `content::material` is what reports it; this is the half
        // that refuses to guess.
        for tint in Tint::ALL {
            assert_eq!(Tint::from_name(tint.name()), Some(tint));
            assert_eq!(Tint::from_name(&tint.name().to_uppercase()), Some(tint));
        }
        assert_eq!(Tint::from_name("chartreuse"), None);
        assert_eq!(Tint::from_name(""), None);
    }
}
