//! A spell's highlighting, in sixteen colours.

use orbs_render::{Intensity, Lexeme, Role, Style};

use super::resolve::{accent, resolve, syntax_colour};

/// Every part of speech that departs from the base gets its own colour.
///
/// Seven categories and six colours: `Control` and `Call` share one, both
/// being the file's own scaffolding, and the `()` tells them apart.
#[test]
fn each_part_of_speech_that_departs_has_its_own_colour() {
    let departs = [
        Lexeme::Control,
        Lexeme::Verb,
        Lexeme::Number,
        Lexeme::Grammar,
        Lexeme::State,
    ];
    let mut seen = Vec::new();
    for kind in departs {
        let colour = syntax_colour(kind).expect("a departing kind has a colour");
        assert!(
            !seen.contains(&colour),
            "{kind:?} shares a colour with something already listed",
        );
        seen.push(colour);
    }
    assert_eq!(
        syntax_colour(Lexeme::Call),
        syntax_colour(Lexeme::Control),
        "a call is the file's own scaffolding and reads as such",
    );
}

/// ...and the ones that keep the reader's own foreground keep it.
///
/// A name is most of a spell, so colouring it leaves the departures with
/// nothing to depart from; comments and filler recede on the weight axis.
#[test]
fn the_base_hue_still_carries_most_of_a_spell() {
    for kind in [Lexeme::Name, Lexeme::Comment, Lexeme::Filler, Lexeme::None] {
        assert_eq!(
            syntax_colour(kind),
            None,
            "{kind:?} overrode the reader's own foreground",
        );
    }
    // The weight axis is doing the work for those two, and still is.
    assert_eq!(Lexeme::Comment.weight(), Intensity::Dim);
    assert_eq!(Lexeme::Filler.weight(), Intensity::Dim);
}

/// An accent outranks a syntax colour, as it outranks a tint.
#[test]
fn a_fault_outranks_the_highlighting_here_too() {
    assert_eq!(
        resolve(Style::DANGER, None, Some(Lexeme::Control)),
        accent(Role::Danger, Intensity::Normal),
        "a line the orb could not read went violet instead of red",
    );
}
