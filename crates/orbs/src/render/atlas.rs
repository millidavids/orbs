//! The glyph atlas — three faces, 256 glyphs each, in one texture.
//!
//! Layout is a 16×16 grid of cells per face, faces stacked vertically:
//!
//! ```text
//!            128 px
//!         ┌──────────┐
//!  256 px │  Plain   │   codepage index -> (index % 16, index / 16)
//!         ├──────────┤
//!  256 px │ Eldritch │   face -> vertical band
//!         ├──────────┤
//!  256 px │ Tampered │
//!         └──────────┘
//! ```
//!
//! One texture rather than three keeps the renderer to a single bind group, and
//! stacking by face means a cell's atlas coordinate is pure arithmetic on
//! `(cp437_index, Presentation)` with no lookup table.
//!
//! The texture is `R8Unorm` — coverage only. Colour is resolved per cell from
//! the phosphor theme, because `orbs-render` never emits one (architectural
//! rule 2).

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use orbs_render::{CELL_HEIGHT, CELL_WIDTH, Presentation};

use super::glyphs::{self, Face, Height};

/// Glyphs per row in the atlas.
const COLUMNS: u32 = 16;
/// Rows of glyphs per face.
const ROWS: u32 = 16;
/// Faces stacked vertically: plain, eldritch, tampered.
pub(crate) const FACES: u32 = 3;

/// Atlas width in pixels.
pub(crate) const WIDTH: u32 = COLUMNS * CELL_WIDTH as u32;
/// Height of one face's band, in pixels.
pub(crate) const BAND_HEIGHT: u32 = ROWS * CELL_HEIGHT as u32;
/// Atlas height in pixels.
pub(crate) const HEIGHT: u32 = BAND_HEIGHT * FACES;

// The fonts ship verbatim in `assets/`; see their PROVENANCE.md files.
const PLAIN_HEX: &str = include_str!("../../../../assets/fonts/unscii/unscii-16.hex");
const ELDRITCH_HEX: &str = include_str!("../../../../assets/fonts/unscii/unscii-8-fantasy.hex");
const TAMPERED_HEX: &str = include_str!("../../../../assets/fonts/unscii/unscii-8-mcr.hex");
const FALLBACK_BDF: &str = include_str!("../../../../assets/fonts/spleen/spleen-8x16-ibm-437.bdf");

/// The built atlas texture and the numbers needed to address it.
#[derive(Resource, Debug, Clone)]
pub(crate) struct GlyphAtlas {
    /// The `R8Unorm` coverage texture.
    #[expect(
        dead_code,
        reason = "consumed by the grid draw step; #[expect] fails once it is"
    )]
    pub(crate) image: Handle<Image>,
    /// How many blank slots the fallback font had to fill, per face.
    pub(crate) filled_from_fallback: [usize; FACES as usize],
}

/// Which vertical band a presentation draws from.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "consumed by the grid draw step")
)]
pub(crate) const fn band(presentation: Presentation) -> u32 {
    match presentation {
        Presentation::Plain => 0,
        Presentation::Eldritch => 1,
        Presentation::Tampered => 2,
    }
}

/// Build the atlas from the embedded fonts.
pub(crate) fn build(images: &mut Assets<Image>) -> GlyphAtlas {
    let fallback = glyphs::parse_bdf(FALLBACK_BDF);

    let mut faces = [
        glyphs::parse_hex(PLAIN_HEX, Height::Native),
        glyphs::parse_hex(ELDRITCH_HEX, Height::Doubled),
        glyphs::parse_hex(TAMPERED_HEX, Height::Doubled),
    ];

    let mut filled_from_fallback = [0; FACES as usize];
    for (index, face) in faces.iter_mut().enumerate() {
        filled_from_fallback[index] = glyphs::fill_gaps(face, &fallback);
    }

    let mut pixels = vec![0u8; (WIDTH * HEIGHT) as usize];
    for (face_index, face) in faces.iter().enumerate() {
        let band_top = u32::try_from(face_index).unwrap_or(0) * BAND_HEIGHT;
        blit(&mut pixels, face, band_top);
    }

    let mut image = Image::new(
        Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );

    // Nearest, always. A bitmap font filtered bilinearly is a blurred bitmap
    // font, and DESIGN.md §4 is explicit that legibility is the product.
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });

    GlyphAtlas {
        image: images.add(image),
        filled_from_fallback,
    }
}

/// Write one face's 256 glyphs into the pixel buffer at `band_top`.
fn blit(pixels: &mut [u8], face: &Face, band_top: u32) {
    for (index, glyph) in face.iter().enumerate() {
        let index = u32::try_from(index).unwrap_or(0);
        let left = (index % COLUMNS) * u32::from(CELL_WIDTH);
        let top = band_top + (index / COLUMNS) * u32::from(CELL_HEIGHT);

        for row in 0..u32::from(CELL_HEIGHT) {
            for column in 0..u32::from(CELL_WIDTH) {
                if !glyphs::pixel(glyph, column, row) {
                    continue;
                }
                let offset = (top + row) * WIDTH + left + column;
                if let Ok(offset) = usize::try_from(offset)
                    && let Some(texel) = pixels.get_mut(offset)
                {
                    *texel = u8::MAX;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build the faces without touching Bevy's asset system.
    fn faces() -> [Face; 3] {
        let fallback = glyphs::parse_bdf(FALLBACK_BDF);
        let mut faces = [
            glyphs::parse_hex(PLAIN_HEX, Height::Native),
            glyphs::parse_hex(ELDRITCH_HEX, Height::Doubled),
            glyphs::parse_hex(TAMPERED_HEX, Height::Doubled),
        ];
        for face in &mut faces {
            glyphs::fill_gaps(face, &fallback);
        }
        faces
    }

    #[test]
    fn the_atlas_is_the_size_the_layout_implies() {
        assert_eq!(WIDTH, 128);
        assert_eq!(BAND_HEIGHT, 256);
        assert_eq!(HEIGHT, 768);
    }

    #[test]
    fn every_face_draws_every_glyph_the_frame_can_hold() {
        // The whole point of the fallback. A blank slot here is a hole on screen
        // that shows up only for characters nobody tested.
        for (index, face) in faces().iter().enumerate() {
            for code in 0..=u8::MAX {
                if !glyphs::needs_glyph(code) {
                    continue;
                }
                let glyph = orbs_render::cp437::cp437_glyph(code);
                assert!(
                    glyphs::is_inked(&face[usize::from(code)]),
                    "face {index}: {glyph:?} ({code:#04x}) is blank"
                );
            }
        }
    }

    #[test]
    fn the_fallback_fills_exactly_the_four_documented_gaps() {
        let fallback = glyphs::parse_bdf(FALLBACK_BDF);
        for (name, source, height) in [
            ("plain", PLAIN_HEX, Height::Native),
            ("eldritch", ELDRITCH_HEX, Height::Doubled),
            ("tampered", TAMPERED_HEX, Height::Doubled),
        ] {
            let mut face = glyphs::parse_hex(source, height);
            let filled = glyphs::fill_gaps(&mut face, &fallback);
            assert_eq!(filled, 4, "{name} needed {filled} fallback glyphs, not 4");
        }
    }

    #[test]
    fn the_three_faces_are_actually_different() {
        // If two bands were identical, Presentation would be carrying no
        // information and the whole scheme would be silently inert.
        let [plain, eldritch, tampered] = faces();
        let letters: Vec<usize> = (b'a'..=b'z').map(usize::from).collect();

        let differs = |a: &Face, b: &Face| letters.iter().any(|i| a[*i] != b[*i]);
        assert!(
            differs(&plain, &eldritch),
            "plain and eldritch are identical"
        );
        assert!(
            differs(&plain, &tampered),
            "plain and tampered are identical"
        );
        assert!(
            differs(&eldritch, &tampered),
            "the special faces are identical"
        );
    }

    #[test]
    fn blitting_places_a_glyph_at_its_codepage_coordinate() {
        let mut pixels = vec![0u8; (WIDTH * HEIGHT) as usize];
        let mut face = [glyphs::BLANK; 256];
        // Codepage 0x11 -> column 1, row 1. Set its top-left pixel.
        face[0x11][0] = 0b1000_0000;
        blit(&mut pixels, &face, 0);

        let x = u32::from(CELL_WIDTH);
        let y = u32::from(CELL_HEIGHT);
        assert_eq!(pixels[(y * WIDTH + x) as usize], u8::MAX);
        assert_eq!(pixels[0], 0, "nothing should land at the origin");
    }

    #[test]
    fn bands_map_to_presentations_in_order() {
        assert_eq!(band(Presentation::Plain), 0);
        assert_eq!(band(Presentation::Eldritch), 1);
        assert_eq!(band(Presentation::Tampered), 2);
    }
}
