//! `weight heavy` — every stroke a pixel wider.
//!
//! The VGA's own bold: each row of a glyph OR'd with itself one pixel to the
//! right. Generated from the shipped fonts, so there is no second font to
//! license and nothing to keep in step with the first.

use bevy::prelude::*;
use orbs_shell::settings::Pair;

use super::glyphs::Face;

/// How heavy the strokes are — `settings` → `access` → `weight`. Anything but
/// `heavy` is the font as it ships.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Weight(bool);

impl From<bool> for Weight {
    fn from(heavy: bool) -> Self {
        Self(heavy)
    }
}

impl Pair for Weight {
    const KEY: &'static str = "weight";
    const WORDS: [&'static str; 2] = ["normal", "heavy"];

    fn raised(self) -> bool {
        self.0
    }
}

impl Default for Weight {
    fn default() -> Self {
        Self::remembered()
    }
}

/// The codepage's shades and box drawing, left as they are.
///
/// ░▒▓ are distinct states of the fire, the smoke and the sediment, and a
/// one-pixel smear turns ▒ into █. Box drawing is left too, so a pane's
/// border still meets its corners.
const UNTOUCHED: core::ops::RangeInclusive<usize> = 0xB0..=0xDF;

/// Thicken every glyph in `face` but the shades and the box drawing.
pub(crate) fn embolden(face: &mut Face) {
    for (index, glyph) in face.iter_mut().enumerate() {
        if UNTOUCHED.contains(&index) {
            continue;
        }
        for row in glyph.iter_mut() {
            *row |= *row >> 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::atlas;
    use super::*;

    #[test]
    fn no_two_glyphs_become_one() {
        // A heavy weight that merged two letters would carry less than the
        // Frame does, which rule 2 forbids.
        let normal = atlas::faces(Weight::named("normal"));
        let heavy = atlas::faces(Weight::named("heavy"));
        for (face, (plain, bold)) in normal.iter().zip(&heavy).enumerate() {
            for first in 0..256 {
                for second in (first + 1)..256 {
                    if plain[first] != plain[second] {
                        assert_ne!(
                            bold[first], bold[second],
                            "face {face}: {first:#04x} and {second:#04x} are one glyph at heavy",
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_tampered_glyph_still_reads_as_tampered() {
        // §8.1's tells are glyph substitution: plain and tampered must stay
        // apart wherever they were apart.
        let normal = atlas::faces(Weight::named("normal"));
        let heavy = atlas::faces(Weight::named("heavy"));
        let (plain, tampered) = (0, 2);
        for index in 0..256 {
            if normal[plain][index] != normal[tampered][index] {
                assert_ne!(
                    heavy[plain][index], heavy[tampered][index],
                    "{index:#04x} is the same plain and tampered at heavy",
                );
            }
        }
    }

    #[test]
    fn the_shades_are_untouched_and_a_letter_is_thicker() {
        let normal = atlas::faces(Weight::named("normal"));
        let heavy = atlas::faces(Weight::named("heavy"));
        for shade in [0xB0, 0xB1, 0xB2, 0xDB] {
            assert_eq!(
                normal[0][shade], heavy[0][shade],
                "{shade:#04x} was smeared"
            );
        }
        let lit = |glyph: &[u8; 16]| glyph.iter().map(|row| row.count_ones()).sum::<u32>();
        let letter = usize::from(b'l');
        assert!(
            lit(&heavy[0][letter]) > lit(&normal[0][letter]),
            "`l` did not thicken"
        );
    }
}
