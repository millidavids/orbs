//! The shipped font and the repertoire this crate defines must agree.
//!
//! `cp437_index()` hands the frontend an atlas index. If the atlas does not have
//! a glyph at that index, the screen gets a hole — and it gets one only for the
//! characters nobody tested, which in this game means the box-drawing and the
//! eldritch punctuation rather than the letters.
//!
//! These tests read the real asset, so they also fail when it goes missing, gets
//! swapped for a different size, or is replaced by a build that is not a complete
//! codepage. See `assets/fonts/spleen/PROVENANCE.md`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use orbs_render::{CELL_HEIGHT, CELL_WIDTH, cp437, cp437_index};

fn font_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fonts/spleen/spleen-8x16-ibm-437.bdf")
}

struct Bdf {
    width: u32,
    height: u32,
    glyphs: BTreeMap<u32, Vec<String>>,
}

/// Enough BDF to answer "is there a glyph here, and how tall is it".
fn parse(source: &str) -> Bdf {
    let mut glyphs = BTreeMap::new();
    let (mut width, mut height) = (0, 0);
    let (mut encoding, mut rows, mut in_bitmap) = (None, Vec::new(), false);

    for line in source.lines() {
        let line = line.trim_end();
        if let Some(rest) = line.strip_prefix("FONTBOUNDINGBOX ") {
            let mut parts = rest.split_whitespace();
            width = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
            height = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("ENCODING ") {
            encoding = rest.trim().parse::<i64>().ok();
        } else if line == "BITMAP" {
            in_bitmap = true;
            rows = Vec::new();
        } else if line == "ENDCHAR" {
            if let Some(code) = encoding.take().filter(|c| *c >= 0) {
                let code = u32::try_from(code).expect("non-negative");
                glyphs.insert(code, std::mem::take(&mut rows));
            }
            in_bitmap = false;
        } else if in_bitmap {
            rows.push(line.to_owned());
        }
    }

    Bdf {
        width,
        height,
        glyphs,
    }
}

fn font() -> Bdf {
    let path = font_path();
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    parse(&source)
}

#[test]
fn the_font_asset_is_present_and_is_eight_by_sixteen() {
    let bdf = font();
    assert_eq!(
        (bdf.width, bdf.height),
        (u32::from(CELL_WIDTH), u32::from(CELL_HEIGHT)),
        "the atlas cell size must match what Fidelity scales by"
    );
}

#[test]
fn the_font_is_a_complete_codepage() {
    // 256 glyphs, encodings 0..=255, no gaps. A partial codepage would render
    // holes only for the characters nobody thought to test.
    let bdf = font();
    let missing: Vec<u32> = (0..256).filter(|c| !bdf.glyphs.contains_key(c)).collect();
    assert!(
        missing.is_empty(),
        "codepage slots with no glyph: {missing:?}"
    );
    assert_eq!(bdf.glyphs.len(), 256);
}

#[test]
fn every_glyph_is_sixteen_rows_tall() {
    let bdf = font();
    for (code, rows) in &bdf.glyphs {
        assert_eq!(
            rows.len(),
            usize::from(CELL_HEIGHT),
            "glyph {code:#04x} has {} rows",
            rows.len()
        );
    }
}

#[test]
fn every_character_the_frame_can_hold_has_a_glyph_in_the_atlas() {
    // The load-bearing one: closes the loop between cp437_index() and the asset.
    let bdf = font();
    for code in 0..=u8::MAX {
        let glyph = cp437::cp437_glyph(code);
        let Some(index) = cp437_index(glyph) else {
            continue; // control characters are refused before they reach a Cell
        };
        assert!(
            bdf.glyphs.contains_key(&u32::from(index)),
            "{glyph:?} maps to atlas index {index:#04x}, which has no glyph"
        );
    }
}

#[test]
fn the_box_drawing_the_game_is_built_from_is_not_blank() {
    // Every pane border, table, and progress bar is drawn with these. A present
    // but empty glyph would pass the completeness test and still draw nothing.
    use orbs_render::cp437::box_drawing::{
        BOTTOM_LEFT, BOTTOM_RIGHT, HORIZONTAL, TOP_LEFT, TOP_RIGHT, VERTICAL,
    };

    let bdf = font();
    for glyph in [
        TOP_LEFT,
        TOP_RIGHT,
        BOTTOM_LEFT,
        BOTTOM_RIGHT,
        HORIZONTAL,
        VERTICAL,
        '░',
        '▒',
        '▓',
        '█',
    ] {
        let index = cp437_index(glyph).expect("in the repertoire");
        let rows = bdf.glyphs.get(&u32::from(index)).expect("in the atlas");
        let inked = rows.iter().any(|row| row.chars().any(|c| c != '0'));
        assert!(inked, "{glyph:?} is present but draws nothing");
    }
}
