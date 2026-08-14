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
//! The texture is **white with coverage in alpha**, not a single-channel
//! coverage map. Bevy's stock `ColorMaterial` multiplies the sampled texel by
//! the vertex colour, so `(1, 1, 1, coverage) * (r, g, b, 1)` gives the glyph in
//! the cell's colour with no custom shader at all. An `R8Unorm` atlas would
//! sample as `(coverage, 0, 0, 1)` and tint the whole screen red.
//!
//! Colour itself is resolved per cell from the phosphor theme, because
//! `orbs-render` never emits one (architectural rule 2). See [`super::palette`].

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
    /// White RGB with coverage in alpha.
    pub(crate) image: Handle<Image>,
    /// How many blank slots the fallback font had to fill, per face.
    pub(crate) filled_from_fallback: [usize; FACES as usize],
}

/// The UV rectangle for a glyph, as `(u0, v0, u1, v1)`.
///
/// Half-texel insets are deliberately absent, and what that rests on has
/// narrowed. It used to be that sampling was Nearest *and* the cell grid was
/// integer-scaled, so no texel was ever interpolated at all. Magnification is
/// still Nearest, so an upscaled glyph is exact whether or not the scale is a
/// whole number — the sampler picks one texel per fragment and a cell edge falls
/// where it falls.
///
/// **Minification is the case with no inset to protect it**, because bilinear
/// there would reach a neighbouring glyph's column: the atlas packs 8×16 cells
/// edge to edge with no padding. That is survivable only because it is
/// unreachable above `MIN_SCALE` — below the floor the game draws the
/// "window too small" card and nothing else. Adding padding here is the price of
/// ever wanting a sub-native picture for real.
pub(crate) fn uv(index: u8, presentation: Presentation) -> (f32, f32, f32, f32) {
    let column = f32::from(u16::from(index) % 16);
    let row = f32::from(u16::from(index) / 16);
    let band = f32::from(u16::try_from(band(presentation)).unwrap_or(0));

    let cell_u = f32::from(CELL_WIDTH) / f32::from(u16::try_from(WIDTH).unwrap_or(1));
    let cell_v = f32::from(CELL_HEIGHT) / f32::from(u16::try_from(HEIGHT).unwrap_or(1));
    let band_v = f32::from(u16::try_from(BAND_HEIGHT).unwrap_or(0))
        / f32::from(u16::try_from(HEIGHT).unwrap_or(1));

    let u0 = column * cell_u;
    let v0 = band * band_v + row * cell_v;
    (u0, v0, u0 + cell_u, v0 + cell_v)
}

/// Which vertical band a presentation draws from.
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

    // RGBA: white everywhere, alpha carries coverage.
    let mut pixels = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
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
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    // **Nearest going up, Linear coming down**, and the asymmetry is the point.
    //
    // A bitmap font filtered bilinearly is a blurred bitmap font, and DESIGN.md
    // §4 is explicit that legibility is the product — so magnification, which is
    // every window at or above `MIN_SCALE`, stays Nearest.
    //
    // Minification is the opposite problem. Nearest *drops* whole source columns
    // rather than softening them, so an 8-pixel glyph squeezed into 6 loses two
    // of its strokes and which two depends on where the letter sits — the same
    // word becomes a different smear at every position. That is only reachable
    // below the floor, where all the game draws is the "window too small" card,
    // and a soft card beats a broken one. See `uv` for what has no inset.
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Linear,
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
                let texel = ((top + row) * WIDTH + left + column) * 4;
                if let Ok(texel) = usize::try_from(texel)
                    && let Some(rgba) = pixels.get_mut(texel..texel + 4)
                {
                    // White, fully opaque. Everything else stays transparent.
                    rgba.fill(u8::MAX);
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

    fn blank_atlas() -> Vec<u8> {
        vec![0u8; (WIDTH * HEIGHT * 4) as usize]
    }

    #[test]
    fn blitting_places_a_glyph_at_its_codepage_coordinate() {
        let mut pixels = blank_atlas();
        let mut face = [glyphs::BLANK; 256];
        // Codepage 0x11 -> column 1, row 1. Set its top-left pixel.
        face[0x11][0] = 0b1000_0000;
        blit(&mut pixels, &face, 0);

        let x = u32::from(CELL_WIDTH);
        let y = u32::from(CELL_HEIGHT);
        let texel = ((y * WIDTH + x) * 4) as usize;
        assert_eq!(&pixels[texel..texel + 4], &[u8::MAX; 4]);
        assert_eq!(&pixels[0..4], &[0; 4], "nothing should land at the origin");
    }

    #[test]
    fn unset_pixels_stay_fully_transparent() {
        // The material multiplies texel by vertex colour, so a glyph's empty
        // pixels must be alpha 0 — otherwise every cell draws a solid block.
        let mut pixels = blank_atlas();
        let mut face = [glyphs::BLANK; 256];
        face[0][0] = 0b1000_0000;
        blit(&mut pixels, &face, 0);

        assert_eq!(pixels[3], u8::MAX, "the set pixel should be opaque");
        assert_eq!(pixels[7], 0, "its neighbour should be transparent");
    }

    #[test]
    fn uvs_tile_the_atlas_without_overlapping() {
        // Every glyph must map to its own rectangle; an off-by-one here draws
        // the neighbouring character, which is the kind of bug that only shows
        // on the glyphs nobody tested.
        let (u0, v0, u1, v1) = uv(0, Presentation::Plain);
        assert!((u0 - 0.0).abs() < 1e-6 && (v0 - 0.0).abs() < 1e-6);
        assert!((u1 - 1.0 / 16.0).abs() < 1e-6);

        // Index 16 is the start of the second row of the first band.
        let (_, v0_row1, _, _) = uv(16, Presentation::Plain);
        assert!((v0_row1 - (v1 - v0)).abs() < 1e-6);

        // The same index in a later band sits exactly one band lower.
        let (_, plain_v, _, _) = uv(65, Presentation::Plain);
        let (_, eldritch_v, _, _) = uv(65, Presentation::Eldritch);
        assert!((eldritch_v - plain_v - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn every_uv_stays_inside_the_texture() {
        for index in 0..=u8::MAX {
            for presentation in [
                Presentation::Plain,
                Presentation::Eldritch,
                Presentation::Tampered,
            ] {
                let (u0, v0, u1, v1) = uv(index, presentation);
                assert!((0.0..=1.0).contains(&u0) && (0.0..=1.0).contains(&u1));
                assert!((0.0..=1.0).contains(&v0) && (0.0..=1.0).contains(&v1));
                assert!(u1 > u0 && v1 > v0);
            }
        }
    }

    #[test]
    fn bands_map_to_presentations_in_order() {
        assert_eq!(band(Presentation::Plain), 0);
        assert_eq!(band(Presentation::Eldritch), 1);
        assert_eq!(band(Presentation::Tampered), 2);
    }
}
