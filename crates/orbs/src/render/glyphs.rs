//! Reading the shipped fonts into bitmaps.
//!
//! Two formats, because the two sources use different ones and both ship
//! verbatim so their checksums still match upstream (`assets/fonts/*/
//! PROVENANCE.md`):
//!
//! - **`.hex`** — unscii. `codepoint:hexbitmap` per line, two hex digits per
//!   row. The three [`Presentation`](orbs_render::Presentation) faces.
//! - **`.bdf`** — Spleen. Used only to fill the four glyphs every unscii face is
//!   missing, which is the job `assets/README.md` keeps it in the repo for.
//!
//! Everything here works in **codepage indices**, not Unicode. `cp437_index`
//! turns a `Cell`'s glyph into the index, and that index addresses the atlas
//! directly.

use orbs_render::{CELL_HEIGHT, CELL_WIDTH, cp437};

/// One glyph: one byte per row, most significant bit leftmost.
pub(crate) type Bitmap = [u8; CELL_HEIGHT as usize];

/// A blank glyph.
pub(crate) const BLANK: Bitmap = [0; CELL_HEIGHT as usize];

/// The 256 glyphs of one face, in codepage order.
pub(crate) type Face = [Bitmap; 256];

/// How tall the source glyphs are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Height {
    /// Drawn at 8×16. Used verbatim.
    Native,
    /// Drawn at 8×8. Each row is duplicated, as upstream's `doubleheight.pl`
    /// does, because a 16-row cell must be filled somehow and stretching keeps
    /// the family's proportions.
    Doubled,
}

/// Parse a `.hex` font into a codepage-ordered face.
///
/// Unicode-keyed input, codepage-indexed output: every slot the repertoire
/// defines is looked up by its Unicode scalar and stored at its codepage index.
/// Slots the font does not draw are left blank for [`fill_gaps`] to handle.
pub(crate) fn parse_hex(source: &str, height: Height) -> Face {
    let mut face = [BLANK; 256];
    for line in source.lines() {
        let Some((code, bits)) = line.split_once(':') else {
            continue;
        };
        let Ok(code) = u32::from_str_radix(code.trim(), 16) else {
            continue;
        };

        // Ask the repertoire directly rather than scanning a 256-slot table per
        // line. These files hold ~3,200 lines each and only 256 of them matter.
        let Some(glyph) = char::from_u32(code) else {
            continue;
        };
        let Some(index) = cp437::cp437_index(glyph) else {
            continue;
        };
        face[usize::from(index)] = decode_hex(bits.trim(), height);
    }
    face
}

fn decode_hex(bits: &str, height: Height) -> Bitmap {
    let mut out = BLANK;
    let source_rows = match height {
        Height::Native => CELL_HEIGHT as usize,
        Height::Doubled => CELL_HEIGHT as usize / 2,
    };

    for row in 0..source_rows {
        let Some(pair) = bits.get(row * 2..row * 2 + 2) else {
            break;
        };
        let Ok(byte) = u8::from_str_radix(pair, 16) else {
            break;
        };
        match height {
            Height::Native => out[row] = byte,
            Height::Doubled => {
                out[row * 2] = byte;
                out[row * 2 + 1] = byte;
            }
        }
    }
    out
}

/// Parse a codepage-indexed `.bdf` into a face.
///
/// Only enough BDF to read `ENCODING` and the bitmap rows. Spleen's CP437 file
/// is already indexed 0–255, so no Unicode round trip is needed.
pub(crate) fn parse_bdf(source: &str) -> Face {
    let mut face = [BLANK; 256];
    let (mut encoding, mut row, mut in_bitmap) = (None, 0usize, false);

    for line in source.lines() {
        let line = line.trim_end();
        if let Some(rest) = line.strip_prefix("ENCODING ") {
            encoding = rest
                .trim()
                .parse::<i64>()
                .ok()
                .filter(|c| (0..256).contains(c));
        } else if line == "BITMAP" {
            in_bitmap = true;
            row = 0;
        } else if line == "ENDCHAR" {
            in_bitmap = false;
            encoding = None;
        } else if in_bitmap
            && let (Some(code), Some(pair)) = (encoding, line.get(0..2))
            && let Ok(byte) = u8::from_str_radix(pair, 16)
            && row < CELL_HEIGHT as usize
            && let Ok(index) = usize::try_from(code)
        {
            face[index][row] = byte;
            row += 1;
        }
    }
    face
}

/// Codepage slots that draw nothing *by design*: space and no-break space.
///
/// Neither is a gap, and neither may be filled from a fallback — some fonts draw
/// a visible marker for no-break space, which would put ink on screen exactly
/// where the grid promises none.
pub(crate) const fn is_blank_by_design(index: u8) -> bool {
    // 0x20 space, 0xFF no-break space (U+00A0).
    matches!(index, 0x20 | 0xFF)
}

/// Whether a codepage slot must carry ink.
///
/// False for slots that draw nothing by design ([`is_blank_by_design`]) and for
/// slots the repertoire refuses outright — index `0x00` is NUL, which
/// `cp437_index` rejects, so it can never reach a `Cell` and never needs a
/// glyph behind it.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "consumed by the grid draw step")
)]
pub(crate) fn needs_glyph(index: u8) -> bool {
    !is_blank_by_design(index) && cp437::cp437_index(cp437::cp437_glyph(index)).is_some()
}

/// Fill blank slots in `face` from `fallback`.
///
/// Every unscii face is missing `∙ ⌂ ⌐ ☼`. A hole in the atlas is a hole on
/// screen, and it would appear only for characters nobody thought to test, so
/// the documented fallback fills them rather than leaving them to chance.
///
/// Returns how many slots were filled.
pub(crate) fn fill_gaps(face: &mut Face, fallback: &Face) -> usize {
    let mut filled = 0;
    for index in 0..=u8::MAX {
        if is_blank_by_design(index) {
            continue;
        }
        let slot = usize::from(index);
        if face[slot] == BLANK && fallback[slot] != BLANK {
            face[slot] = fallback[slot];
            filled += 1;
        }
    }
    filled
}

/// Whether a bitmap would draw anything.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "consumed by the grid draw step")
)]
pub(crate) fn is_inked(bitmap: &Bitmap) -> bool {
    bitmap.iter().any(|row| *row != 0)
}

/// Whether a pixel is set. `column` counts from the left.
pub(crate) fn pixel(bitmap: &Bitmap, column: u32, row: u32) -> bool {
    let Ok(row) = usize::try_from(row) else {
        return false;
    };
    let Some(bits) = bitmap.get(row) else {
        return false;
    };
    if column >= u32::from(CELL_WIDTH) {
        return false;
    }
    bits >> (u32::from(CELL_WIDTH) - 1 - column) & 1 == 1
}

/// Codepoints upstream unscii does not draw, which the fallback must supply.
///
/// Pinned rather than tolerated: if upstream fills one in, the test fails and
/// says a hand-drawn glyph can be dropped.
#[cfg(test)]
pub(crate) const UNSCII_GAPS: [char; 4] = ['∙', '⌂', '⌐', '☼'];

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn assets() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets")
    }

    fn read(relative: &str) -> String {
        let path = assets().join(relative);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
    }

    // ------------------------------------------------------------------
    // The shipped assets, read through the parsers that actually ship.
    //
    // These lived in `orbs-render/tests/` with two hand-rolled duplicate
    // parsers. `orbs-render` defines the repertoire and never reads a font;
    // `orbs` is the only crate that parses one. Testing the assets anywhere
    // else meant a second implementation that could drift from this one.
    // ------------------------------------------------------------------

    #[test]
    fn the_spleen_fallback_is_a_complete_codepage() {
        let face = parse_bdf(&read("fonts/spleen/spleen-8x16-ibm-437.bdf"));
        for index in 0..=u8::MAX {
            if !needs_glyph(index) {
                continue;
            }
            assert!(
                is_inked(&face[usize::from(index)]),
                "spleen slot {index:#04x} is blank"
            );
        }
    }

    #[test]
    fn every_unscii_face_covers_the_repertoire_apart_from_the_known_gaps() {
        let faces = [
            ("unscii-16.hex", Height::Native),
            ("unscii-8-fantasy.hex", Height::Doubled),
            ("unscii-8-mcr.hex", Height::Doubled),
        ];
        let mut gaps: Vec<u8> = UNSCII_GAPS
            .iter()
            .filter_map(|glyph| cp437::cp437_index(*glyph))
            .collect();
        gaps.sort_unstable();

        for (name, height) in faces {
            let face = parse_hex(&read(&format!("fonts/unscii/{name}")), height);
            let mut missing = Vec::new();
            for index in 0..=u8::MAX {
                if needs_glyph(index) && !is_inked(&face[usize::from(index)]) {
                    missing.push(index);
                }
            }
            assert_eq!(missing, gaps, "{name} gaps changed");
        }
    }

    #[test]
    fn the_three_faces_share_metrics_and_never_disagree_on_coverage() {
        // A face swap must never turn a drawable character into a hole.
        let plain = parse_hex(&read("fonts/unscii/unscii-16.hex"), Height::Native);
        let fantasy = parse_hex(&read("fonts/unscii/unscii-8-fantasy.hex"), Height::Doubled);
        let mcr = parse_hex(&read("fonts/unscii/unscii-8-mcr.hex"), Height::Doubled);

        for index in 0..256 {
            assert_eq!(
                is_inked(&plain[index]),
                is_inked(&fantasy[index]),
                "slot {index:#04x} differs between plain and fantasy"
            );
            assert_eq!(
                is_inked(&plain[index]),
                is_inked(&mcr[index]),
                "slot {index:#04x} differs between plain and mcr"
            );
        }
    }

    #[test]
    fn the_two_special_registers_agree_on_box_drawing() {
        let fantasy = parse_hex(&read("fonts/unscii/unscii-8-fantasy.hex"), Height::Doubled);
        let mcr = parse_hex(&read("fonts/unscii/unscii-8-mcr.hex"), Height::Doubled);

        for glyph in ['┌', '─', '┐', '│', '└', '┘', '├', '┤', '┬', '┴', '┼', '█']
        {
            let index = usize::from(cp437::cp437_index(glyph).expect("in the repertoire"));
            assert_eq!(
                fantasy[index], mcr[index],
                "{glyph:?} differs between the two special faces"
            );
        }
    }

    /// The licence guard.
    ///
    /// `unscii-16-full` merges GPL Unifont and is one word away in name from the
    /// CC0 `unscii-16` we ship. Whatever the project's own licence, taking it
    /// would mean shipping a file whose terms and checksums are not the ones
    /// `PROVENANCE.md` records. This fails the build rather than the submission.
    #[test]
    fn the_gpl_licensed_variant_is_not_in_the_repository() {
        let directory = assets().join("fonts/unscii");
        let entries = std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("cannot list {}: {error}", directory.display()));

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            assert!(
                !name.contains("unscii-16-full") && !name.contains("unifont"),
                "{name} carries different licence terms than PROVENANCE.md records"
            );
        }
    }

    #[test]
    fn provenance_and_licences_travel_with_the_fonts() {
        for relative in [
            "README.md",
            "fonts/unscii/PROVENANCE.md",
            "fonts/unscii/LICENSE.md",
            "fonts/spleen/PROVENANCE.md",
            "fonts/spleen/LICENSE",
        ] {
            assert!(
                Path::new(&assets().join(relative)).is_file(),
                "assets/{relative} is missing"
            );
        }
    }

    #[test]
    fn native_rows_are_taken_verbatim() {
        // 'A' at U+0041, two rows given, rest blank.
        let face = parse_hex("00041:FF00", Height::Native);
        let glyph = face[usize::from(b'A')];
        assert_eq!(glyph[0], 0xFF);
        assert_eq!(glyph[1], 0x00);
    }

    #[test]
    fn doubled_rows_are_duplicated() {
        let face = parse_hex("00041:FF0F", Height::Doubled);
        let glyph = face[usize::from(b'A')];
        assert_eq!(glyph[0], 0xFF);
        assert_eq!(glyph[1], 0xFF, "row 0 should occupy two rows");
        assert_eq!(glyph[2], 0x0F);
        assert_eq!(glyph[3], 0x0F);
    }

    #[test]
    fn unicode_input_lands_at_the_codepage_index() {
        // U+2591 is the light shade, CP437 0xB0.
        let face = parse_hex("02591:AA55AA55AA55AA55", Height::Doubled);
        assert!(is_inked(&face[0xB0]));
        assert!(!is_inked(&face[0xB1]));
    }

    #[test]
    fn gaps_are_filled_from_the_fallback_and_counted() {
        let mut face = [BLANK; 256];
        let mut fallback = [BLANK; 256];
        fallback[0xF9] = [0x18; CELL_HEIGHT as usize];
        fallback[0x7F] = [0x24; CELL_HEIGHT as usize];

        assert_eq!(fill_gaps(&mut face, &fallback), 2);
        assert!(is_inked(&face[0xF9]));
        assert_eq!(fill_gaps(&mut face, &fallback), 0, "already filled");
    }

    #[test]
    fn a_filled_glyph_is_never_overwritten_by_the_fallback() {
        let mut face = [BLANK; 256];
        face[0x41] = [0x11; CELL_HEIGHT as usize];
        let mut fallback = [BLANK; 256];
        fallback[0x41] = [0x99; CELL_HEIGHT as usize];

        fill_gaps(&mut face, &fallback);
        assert_eq!(face[0x41][0], 0x11, "the face's own glyph must win");
    }

    #[test]
    fn blank_by_design_slots_are_never_filled() {
        // Space and no-break space draw nothing on purpose. Some fonts draw a
        // visible marker for NBSP; taking it would put ink where the grid
        // promises none.
        let mut face = [BLANK; 256];
        let mut fallback = [BLANK; 256];
        fallback[0x20] = [0xFF; CELL_HEIGHT as usize];
        fallback[0xFF] = [0xFF; CELL_HEIGHT as usize];

        assert_eq!(fill_gaps(&mut face, &fallback), 0);
        assert!(!is_inked(&face[0x20]));
        assert!(!is_inked(&face[0xFF]));
    }

    #[test]
    fn pixels_read_most_significant_bit_leftmost() {
        let mut glyph = BLANK;
        glyph[0] = 0b1000_0001;
        assert!(pixel(&glyph, 0, 0));
        assert!(!pixel(&glyph, 1, 0));
        assert!(pixel(&glyph, 7, 0));
        assert!(!pixel(&glyph, 8, 0), "outside the cell");
        assert!(!pixel(&glyph, 0, 99), "outside the cell");
    }

    #[test]
    fn bdf_bitmaps_land_at_their_encoding() {
        let source = "\
STARTCHAR test
ENCODING 65
BITMAP
FF
0F
ENDCHAR
";
        let face = parse_bdf(source);
        assert_eq!(face[65][0], 0xFF);
        assert_eq!(face[65][1], 0x0F);
    }
}
