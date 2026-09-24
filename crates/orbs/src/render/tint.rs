//! What a material's colour family looks like.
//!
//! The colours a [`Tint`] resolves to. `orbs-render` says *this region holds
//! something green*; this is the only place that decides what green is — the
//! same rule 2 boundary [`ember`](super::ember) sits on.
//!
//! One base per tint, with the ramp derived: eight tints at three steps would be
//! twenty-four hand-picked numbers and twenty-four chances for a ramp to sag. So
//! each tint is one colour and the steps are scalings of it — monotonic by
//! construction, eight numbers to tune. The scalings were solved, not chosen:
//! [`DIM`] began at `0.66`, which put brown's dimmest step at 2.78:1, under the
//! 3.0 floor `even_dim_text_stays_visible` holds everything to.
//!
//! Three steps serve both instruments, because both ask one question three ways:
//!
//! | Picture | Step 0 | Step 1 | Step 2 |
//! |---|---|---|---|
//! | The mortar, which varies [`Intensity`] | `Dim` | `Normal` | `Bright` |
//! | The bath, which varies [`Roil`](orbs_render::Roil) | `Still` | `Stirred` | `Rolling` |
//!
//! A picture that needed four steps would need its own table.

use bevy::prelude::*;
use orbs_render::{Depiction, Intensity, Tint, Wash};

use super::palette::rgb;

/// How far below the base the dimmest step sits.
///
/// Narrow on purpose. At `0.74`/`1.22` a step was 1.92× the last, which reads as
/// two substances rather than one catching the light; `0.86`/`1.10` is 1.39×.
/// Narrowing is free on contrast — the tightest ratio is the dim step against
/// the background, so raising the floor raises it (3.28:1 became 4.17:1).
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
/// Kept apart from one another as well as visible: the closest pair is
/// `grey`/`violet`, and nearer is two materials indistinguishable on a bar two
/// cells wide.
const fn base_of(tint: Tint) -> Srgba {
    match tint {
        // Muted, and the most-looked-at tint in the game: sage is on screen
        // more than the other seven together, and a saturated green at that
        // duty cycle reads as a highlight rather than a herb.
        Tint::Green => rgb(0.46, 0.62, 0.42),
        Tint::Brown => rgb(0.66, 0.50, 0.30),
        Tint::Grey => rgb(0.66, 0.68, 0.72),
        Tint::Gold => rgb(0.92, 0.78, 0.34),
        Tint::Violet => rgb(0.74, 0.56, 0.95),
        Tint::Red => rgb(0.93, 0.38, 0.38),
        Tint::Blue => rgb(0.42, 0.70, 0.94),
        // A cool off-white, not a cream: this is rock salt, and the warm bone
        // it started as read as old paper beside a finished potion's gold.
        Tint::Bone => rgb(0.90, 0.90, 0.86),
    }
}

/// The colour a wash starts from — one family, or two averaged.
///
/// An average, not a third hand-picked colour: the eye reads a colour between
/// its neighbours as a mixture of them, where an authored third would read as a
/// substitution. Averaged over the *bases* rather than the resolved steps, so
/// the mixed band derives its own three steps and roils like any other liquid.
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
/// Never an accent: §4 reserves the triad for meaning and a tint means nothing
/// on its own, so a `Fouled` instrument's red label stays red. The same rule
/// [`Style::depicted`](orbs_render::Style::depicted) enforces for pictures, and
/// it needs enforcing twice because they are two channels.
///
/// Never a fire either — the athanor burns orange on every tube (§19), and fuel
/// has a tint in the data that the flame simply does not consult.
///
/// What it does colour is the material: the mortar through [`Intensity`], the
/// bath through [`Depiction`].
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

    // The shared rule, asked once, before the ramp. The arms below and
    // `orbs-tui`'s resolver each used to carry their own copy of it, with
    // nothing making the two agree.
    if depiction.declines_tint() {
        return None;
    }

    let ramp = ramp(combined(wash));
    match depiction {
        // The bath: its own three steps, which happen to be this ramp's.
        Depiction::LiquidStill => Some(ramp[0]),
        Depiction::LiquidStirred => Some(ramp[1]),
        Depiction::LiquidRolling => Some(ramp[2]),
        // Waste declines the tint, so waste always looks like waste: grit in
        // the contents' own colour reads as more of the same substance rather
        // than what has dropped out of it. Falls through to `ember::SEDIMENT`.
        Depiction::Sediment => None,
        // A gauge is not a material either, and stands at the top of the pane:
        // a fill tinted by whatever was brewing would say the bar meant
        // something about sage. `declines_tint` returns above.
        Depiction::GaugeFaint
        | Depiction::GaugeLow
        | Depiction::GaugeMiddle
        | Depiction::GaugeHigh
        | Depiction::GaugeNear
        | Depiction::GaugeWhole => None,
        // The fire is not a material. See this function's docs.
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
        // One ramp per tint against four themes, so each clears the floor on
        // all four backgrounds.
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
        // A channel whose values cannot be told apart is not a channel, and two
        // cells is all the side layout gives an instrument.
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
        // §4's rule, enforced a second time because this is a second channel:
        // a fouled instrument's label stays red while its bar is brown husks.
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
        // The athanor burns orange on every tube (§19), and fuel is a material
        // with a tint like any other — so this is the only thing stopping a
        // hearth full of charcoal drawing grey flames.
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
                // `Sediment` is deliberately not on this list: waste declines
                // the tint so it always looks like waste. See `resolve`.
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
        // Authored data selects a tint by name, and an unknown name must be a
        // reported error rather than a silent fallback — `content::material`
        // reports it; this is the half that refuses to guess.
        for tint in Tint::ALL {
            assert_eq!(Tint::from_name(tint.name()), Some(tint));
            assert_eq!(Tint::from_name(&tint.name().to_uppercase()), Some(tint));
        }
        assert_eq!(Tint::from_name("chartreuse"), None);
        assert_eq!(Tint::from_name(""), None);
    }
}
