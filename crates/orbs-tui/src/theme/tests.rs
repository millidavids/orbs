use crossterm::style::Color;
use orbs_render::{Density, Depiction, Heat, Intensity, Lexeme, Roil, Role, Style, Tint, Wash};

use super::ink::{Ink, Weight};
use super::resolve::{accent, depicted, hue, resolve, syntax_colour, tinted};

#[test]
fn every_tint_resolves_to_its_own_hue() {
    // Eight names, eight distinct indices. Two tints sharing a colour would
    // make two materials indistinguishable in a bar that is all one glyph.
    let mut seen = Vec::new();
    for tint in Tint::ALL {
        let colour = hue(tint);
        assert!(
            !seen.contains(&colour),
            "{} shares a colour with something already mapped",
            tint.name(),
        );
        seen.push(colour);
    }
}

#[test]
fn the_three_accents_are_three_colours() {
    // §14: colour never carries meaning alone, but where it carries any it
    // must at least separate.
    let danger = accent(Role::Danger, Intensity::Normal);
    let cost = accent(Role::Cost, Intensity::Normal);
    let success = accent(Role::Success, Intensity::Normal);
    assert_ne!(danger, cost);
    assert_ne!(cost, success);
    assert_ne!(danger, success);
}

/// No step of the gauge ramp may wear an accent's ink. §4 and §14 reserve
/// the triad for meaning and a depiction carries none; it shipped once with
/// `GaugeLow` as `Color::Red` and `GaugeWhole` as `Color::Green`.
///
/// Weight is no escape: a bold green still reads as green, and several
/// terminals render bold-plus-dark as the bright index outright — so the
/// check is on the colour, not on the pair.
#[test]
fn the_gauge_ramp_never_wears_an_accents_ink() {
    let triad: Vec<_> = [Role::Danger, Role::Cost, Role::Success]
        .into_iter()
        .filter_map(|role| accent(role, Intensity::Normal).colour)
        .collect();

    let mut seen = Vec::new();
    for depiction in Depiction::ALL
        .into_iter()
        .filter(|depiction| depiction.is_gauge())
    {
        let ink = depicted(depiction).expect("a gauge step drew nothing");
        let colour = ink.colour.expect("a gauge step took no colour");
        assert!(
            !triad.contains(&colour),
            "{depiction:?} draws in {colour:?}, which is an accent's own ink",
        );
        assert!(
            !seen.contains(&ink),
            "{depiction:?} is indistinguishable from an earlier step",
        );
        seen.push(ink);
    }
    assert_eq!(seen.len(), 6, "the ramp is not six steps");
}

#[test]
fn ordinary_text_keeps_the_terminals_own_foreground() {
    // The base hue is the user's, and §4 lets ordinary text vary only on
    // intensity.
    for intensity in [Intensity::Dim, Intensity::Normal, Intensity::Bright] {
        let ink = accent(Role::Normal, intensity);
        assert_eq!(ink.colour, None, "ordinary text took a colour");
    }
    assert_eq!(accent(Role::Normal, Intensity::Dim).weight, Weight::Dim);
    assert_eq!(accent(Role::Normal, Intensity::Bright).weight, Weight::Bold);
}

#[test]
fn an_accent_ignores_intensity() {
    // The Bevy side asserts the same. Two frontends disagreeing about
    // whether a dim breach is dimmer is rule 2 broken where nobody looks.
    for role in [Role::Danger, Role::Cost, Role::Success] {
        let dim = accent(role, Intensity::Dim);
        let bright = accent(role, Intensity::Bright);
        assert_eq!(dim, bright, "{role:?} was diluted by intensity");
    }
}

#[test]
fn a_tint_declines_to_an_accent() {
    // "An accent is a signal; a tint is a hint. The signal wins."
    let wash = Wash::plain(Tint::Green);
    for role in [Role::Danger, Role::Cost, Role::Success] {
        assert!(
            tinted(wash, role, Intensity::Normal, Depiction::None).is_none(),
            "{role:?} was overpainted by a material's colour",
        );
    }
    assert!(tinted(wash, Role::Normal, Intensity::Normal, Depiction::None).is_some());
}

#[test]
fn a_tint_declines_to_fire_and_to_waste() {
    let wash = Wash::plain(Tint::Green);
    for depiction in Depiction::ALL {
        // Against `orbs-render`'s predicate, which the other frontend also
        // calls. This used to compare `tinted` against a copy of its own
        // rule while claiming to check the Bevy build.
        let declines = tinted(wash, Role::Normal, Intensity::Normal, depiction).is_none();
        assert_eq!(
            declines,
            depiction.declines_tint(),
            "{depiction:?} resolves a tint the shared rule declines, or the \
             other way round — the two frontends would draw it differently",
        );
    }
}

#[test]
fn a_mixture_takes_its_first_tint() {
    // The accepted degradation, stated so it cannot drift into a bug report.
    let mixed = Wash::mixing(Tint::Green, Tint::Bone);
    let plain = Wash::plain(Tint::Green);
    assert_eq!(
        tinted(mixed, Role::Normal, Intensity::Normal, Depiction::None),
        tinted(plain, Role::Normal, Intensity::Normal, Depiction::None),
    );
}

/// How bright a resolved ink reads, for comparing steps of one ramp.
fn brightness(ink: Ink) -> u8 {
    let base = match ink.colour {
        Some(Color::DarkYellow | Color::DarkCyan | Color::DarkGrey) => 0,
        Some(Color::Yellow | Color::Cyan | Color::Grey) => 2,
        _ => 4,
    };
    base + match ink.weight {
        Weight::Dim => 0,
        Weight::Plain => 1,
        Weight::Bold => 2,
    }
}

#[test]
fn the_fire_is_orange_all_the_way_up() {
    // The athanor burns orange on every tube (§19). This ramp once ran
    // `DarkRed` → `White`: red at the cool end, colourless at the hot one.
    for heat in [Heat::Ember, Heat::Flame, Heat::Blaze, Heat::Core] {
        for depiction in [Depiction::flame(heat), Depiction::spark(heat)] {
            let ink = depicted(depiction).expect("fire is a picture of something");
            assert!(
                matches!(ink.colour, Some(Color::DarkYellow | Color::Yellow)),
                "{depiction:?} drew {:?}, which is not a fire",
                ink.colour,
            );
        }
    }
}

#[test]
fn every_ramp_climbs() {
    // A meter is read off where the colour steps, so a ramp that is not
    // monotonic cannot be read.
    let fire: Vec<_> = [Heat::Ember, Heat::Flame, Heat::Blaze, Heat::Core]
        .map(|heat| brightness(depicted(Depiction::flame(heat)).expect("fire")))
        .to_vec();
    assert!(
        fire.windows(2).all(|pair| pair[0] < pair[1]),
        "the flame ramp does not climb: {fire:?}",
    );

    let liquid: Vec<_> = [Roil::Still, Roil::Stirred, Roil::Rolling]
        .map(|roil| brightness(depicted(Depiction::liquid(roil)).expect("liquid")))
        .to_vec();
    assert!(
        liquid.windows(2).all(|pair| pair[0] < pair[1]),
        "the liquid ramp does not climb: {liquid:?}",
    );

    let smoke: Vec<_> = [Density::Thin, Density::Thick]
        .map(|density| brightness(depicted(Depiction::smoke(density)).expect("smoke")))
        .to_vec();
    assert!(
        smoke[0] < smoke[1],
        "the smoke ramp does not climb: {smoke:?}"
    );
}

#[test]
fn the_bath_is_teal_and_waste_is_not() {
    // The fire's opposite stays a colour, and waste stays tellable from the
    // smoke it sits under — `Sediment` and `SmokeThin` were both dark grey.
    for roil in [Roil::Still, Roil::Stirred, Roil::Rolling] {
        let ink = depicted(Depiction::liquid(roil)).expect("liquid");
        assert!(
            matches!(ink.colour, Some(Color::DarkCyan | Color::Cyan)),
            "{roil:?} drew {:?}, which is not the bath",
            ink.colour,
        );
    }
    assert_ne!(
        depicted(Depiction::Sediment),
        depicted(Depiction::smoke(Density::Thin)),
        "waste and thin smoke resolved identically",
    );
}

#[test]
fn a_role_never_reads_its_depiction_field_directly() {
    // An accented cell that also carries a depiction must resolve as the
    // accent, which only happens if `resolve` went through `depicted()`.
    let style = Style::DANGER.with_depiction(Depiction::FlameCore);
    assert_eq!(
        resolve(style, None, None),
        accent(Role::Danger, Intensity::Normal)
    );
}

/// The window's order: a lexeme outranks a depiction. No painter puts the two on
/// one cell, so this is the agreement the doc on `resolve` claims, held.
#[test]
fn a_lexeme_outranks_a_depiction_as_in_the_window() {
    let style = Style::NORMAL.with_depiction(Depiction::FlameCore);
    assert_eq!(
        resolve(style, None, Some(Lexeme::Control)).colour,
        syntax_colour(Lexeme::Control),
        "a depiction outranked a lexeme, where the window has it the other way",
    );
}

/// Greyscale keeps every ramp's weight, which is what carries the ramp once the
/// hue is gone, and leaves no hue behind.
#[test]
fn without_hue_keeps_every_weight() {
    for depiction in Depiction::ALL {
        let Some(ink) = depicted(depiction) else {
            continue;
        };
        let grey = ink.without_hue();
        assert_eq!(grey.weight, ink.weight, "{depiction:?} lost its weight");
        assert!(
            matches!(
                grey.colour,
                Some(Color::DarkGrey | Color::Grey | Color::White)
            ),
            "{depiction:?} kept a hue: {:?}",
            grey.colour,
        );
    }
    // The terminal's own foreground stays its own.
    assert_eq!(Ink::PLAIN.without_hue(), Ink::PLAIN);
}
