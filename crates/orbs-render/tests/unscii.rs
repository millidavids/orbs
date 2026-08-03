//! The three faces behind [`Presentation`], and the licence boundary they sit on.
//!
//! `assets/fonts/unscii/` holds one face per `Presentation` variant. These tests
//! assert the properties the renderer will rely on — identical metrics, identical
//! repertoire, correct native sizes — and, just as importantly, that the **GPL**
//! sibling file has not crept in.
//!
//! See `assets/fonts/unscii/PROVENANCE.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use orbs_render::{CELL_HEIGHT, cp437};

/// Glyphs upstream does not draw, which we must supply before shipping.
///
/// Pinned rather than tolerated: if upstream fills one in, this test fails and
/// tells us we can drop a hand-drawn glyph.
const KNOWN_GAPS: [char; 4] = ['∙', '⌂', '⌐', '☼'];

fn assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/unscii")
}

/// A `.hex` font: `codepoint:hexbitmap` per line. Two hex digits per row, so the
/// string length says whether a glyph is 8×8 or 8×16.
fn load(name: &str) -> BTreeMap<u32, String> {
    let path = assets().join(name);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    source
        .lines()
        .filter_map(|line| {
            let (code, bits) = line.split_once(':')?;
            Some((
                u32::from_str_radix(code.trim(), 16).ok()?,
                bits.trim().to_owned(),
            ))
        })
        .collect()
}

/// `doubleheight.pl`: duplicate each of the eight rows.
fn double(bits: &str) -> String {
    (0..8)
        .map(|row| bits.get(row * 2..row * 2 + 2).unwrap_or("00").repeat(2))
        .collect()
}

fn faces() -> [(&'static str, BTreeMap<u32, String>); 3] {
    [
        ("unscii-16.hex", load("unscii-16.hex")),
        ("unscii-8-fantasy.hex", load("unscii-8-fantasy.hex")),
        ("unscii-8-mcr.hex", load("unscii-8-mcr.hex")),
    ]
}

/// Every codepoint a [`orbs_render::Frame`] can hold, minus the NUL slot.
fn repertoire() -> BTreeSet<u32> {
    (1..=u8::MAX)
        .map(|index| u32::from(cp437::cp437_glyph(index)))
        .collect()
}

#[test]
fn all_three_faces_are_present() {
    for (name, glyphs) in faces() {
        assert!(!glyphs.is_empty(), "{name} is empty or missing");
    }
}

#[test]
fn plain_is_natively_eight_by_sixteen() {
    // The whole reason unscii-16 was chosen over a stretched unscii-8: Plain
    // carries the ~88k-word prose budget and deserves the real resolution.
    let glyphs = load("unscii-16.hex");
    let bits = glyphs.get(&u32::from('A')).expect("'A' exists");
    assert_eq!(
        bits.len(),
        usize::from(CELL_HEIGHT) * 2,
        "unscii-16 should be 16 rows of two hex digits"
    );
}

#[test]
fn the_special_registers_are_eight_by_eight_and_double_cleanly() {
    for name in ["unscii-8-fantasy.hex", "unscii-8-mcr.hex"] {
        let glyphs = load(name);
        let bits = glyphs.get(&u32::from('A')).expect("'A' exists");
        assert_eq!(bits.len(), 16, "{name} should be 8 rows of two hex digits");
        assert_eq!(
            double(bits).len(),
            usize::from(CELL_HEIGHT) * 2,
            "{name} does not double to 16 rows"
        );
    }
}

#[test]
fn every_face_covers_the_repertoire_apart_from_the_known_gaps() {
    let wanted = repertoire();
    let gaps: BTreeSet<u32> = KNOWN_GAPS.iter().map(|c| u32::from(*c)).collect();

    for (name, glyphs) in faces() {
        let present: BTreeSet<u32> = glyphs.keys().copied().collect();
        let missing: BTreeSet<u32> = wanted.difference(&present).copied().collect();
        assert_eq!(
            missing,
            gaps,
            "{name} gaps changed: {:?}",
            missing
                .iter()
                .filter_map(|c| char::from_u32(*c))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn the_three_faces_share_a_repertoire() {
    // A face swap must never be able to turn a drawable character into a hole.
    let [(_, plain), (_, fantasy), (_, mcr)] = faces();
    let wanted = repertoire();

    for code in &wanted {
        let in_plain = plain.contains_key(code);
        assert_eq!(
            in_plain,
            fantasy.contains_key(code),
            "{:?} is in one face but not fantasy",
            char::from_u32(*code)
        );
        assert_eq!(
            in_plain,
            mcr.contains_key(code),
            "{:?} is in one face but not mcr",
            char::from_u32(*code)
        );
    }
}

#[test]
fn the_two_special_registers_agree_on_box_drawing() {
    // Borders inside a tampered or eldritch region must not disagree with each
    // other, or a pane would change shape depending on which register it is in.
    let fantasy = load("unscii-8-fantasy.hex");
    let mcr = load("unscii-8-mcr.hex");

    for glyph in ['┌', '─', '┐', '│', '└', '┘', '├', '┤', '┬', '┴', '┼', '█']
    {
        let code = u32::from(glyph);
        assert_eq!(
            fantasy.get(&code),
            mcr.get(&code),
            "{glyph:?} differs between the two special faces"
        );
    }
}

/// The licence guard.
///
/// `unscii-16-full` is **GPL**, one character away in name from the CC0
/// `unscii-16` we do ship, and merging it in would place a copyleft obligation
/// on a commercial product. This fails the build rather than the submission.
#[test]
fn the_gpl_licensed_variant_is_not_in_the_repository() {
    let directory = assets();
    let entries = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("cannot list {}: {error}", directory.display()));

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        assert!(
            !name.contains("unscii-16-full") && !name.contains("unifont"),
            "{name} is GPL (see PROVENANCE.md) and must not ship in this product"
        );
    }
}

/// Provenance is the point of the directory; a font without it is unshippable.
#[test]
fn provenance_and_licence_travel_with_the_fonts() {
    for name in ["PROVENANCE.md", "LICENSE.md"] {
        assert!(
            Path::new(&assets().join(name)).is_file(),
            "assets/fonts/unscii/{name} is missing"
        );
    }
}
