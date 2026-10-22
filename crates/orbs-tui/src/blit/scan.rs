//! Every East Asian Ambiguous glyph a painter can draw has a narrow stand-in —
//! held by reading the painters' source, so the next one cannot arrive unseen.
//!
//! The table was kept by hand and missed `∞`, then `▲` and `←`: each found by a
//! person, one at a time. Here rather than in `orbs-render` because the answer
//! is this frontend's private table and nothing else consumes it.

use std::path::{Path, PathBuf};

use super::narrow::substituted;

/// CP437's Ambiguous glyphs, less the box drawing and blocks `SUBSTITUTIONS`
/// leaves out on its documented bet. Measured with `unicodedata`.
///
/// `§` is left out: it is how a message cites DESIGN.md, in assertions and the
/// dump's own headings, and never something a painter draws.
const CP437_AMBIGUOUS: &str = "♥♣♠•○♂♀♪↕¶↑↓→←∟↔▲▼üéàêèìæÆòùáíóúªº¿½¼¡αßΓπΣστΦΘΩδ∞φε∩≡±≥≤÷≈°·√ⁿ²■";

/// The source a frame's glyphs come from: the painters, the sim's words, and
/// the content the sim reads.
const ROOTS: [&str; 4] = [
    "../orbs-render/src",
    "../orbs-shell/src",
    "../orbs-sim/src",
    "../orbs-sim/content",
];

#[test]
fn every_ambiguous_glyph_a_painter_draws_is_substituted() {
    let table = substituted();
    let mut missing = Vec::new();
    for root in ROOTS {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(root);
        let mut files = Vec::new();
        gather(&root, &mut files);
        // A root that yields nothing passes without looking — the shape of
        // `orbs-shell/tests/boundaries.rs`'s same guard.
        assert!(!files.is_empty(), "{} yielded no files", root.display());
        for file in files {
            let text = std::fs::read_to_string(&file).expect("a source file reads");
            let glyphs = if file.extension().is_some_and(|ext| ext == "toml") {
                toml_text(&text)
            } else {
                rust_literals(&text)
            };
            for (line, glyph) in glyphs {
                if CP437_AMBIGUOUS.contains(glyph) && !table.contains(&glyph) {
                    missing.push(format!("{}:{line} {glyph:?}", file.display()));
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "Ambiguous glyphs drawn with no narrow stand-in — add them to \
         `SUBSTITUTIONS`:\n{}",
        missing.join("\n"),
    );
}

#[test]
fn the_measured_list_is_cp437() {
    // A typo here is a glyph nothing can draw, which no scan would ever find.
    for glyph in CP437_AMBIGUOUS.chars() {
        assert!(orbs_render::is_renderable(glyph), "{glyph:?} is not CP437");
    }
}

#[test]
fn the_probe_is_a_glyph_that_can_actually_widen() {
    // A Neutral probe reports narrow on every terminal ever built: an
    // instrument wired to a constant.
    let probe = crate::term::PROBE;
    assert!(
        CP437_AMBIGUOUS.contains(probe),
        "{probe:?} is not Ambiguous, so the narrow fallback is unreachable"
    );
    assert!(
        substituted().contains(&probe),
        "{probe:?} would fire with nothing to narrow"
    );
}

/// Every `.rs` and `.toml` under `dir`, less tests and the CP437 table itself.
fn gather(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if name != "tests" {
                gather(&path, into);
            }
        } else if (name.ends_with(".rs") || name.ends_with(".toml"))
            && name != "tests.rs"
            && name != "cp437.rs"
        {
            into.push(path);
        }
    }
}

/// The characters inside string and char literals, by line, with `\u{…}`
/// decoded — `∞` arrives as `"\u{221e}"` and a scan of the raw text missed it.
///
/// Stops at a module-level `#[cfg(test)]`, and skips a line that prints to
/// stdout: neither reaches a frame.
fn rust_literals(text: &str) -> Vec<(usize, char)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut kept = String::new();
    for (at, line) in lines.iter().enumerate() {
        if line.trim() == "#[cfg(test)]"
            && lines[at + 1..]
                .iter()
                .find(|next| !next.trim().is_empty())
                .is_some_and(|next| next.trim_start().starts_with("mod "))
        {
            break;
        }
        if !line.contains("println!(") {
            kept.push_str(line);
        }
        kept.push('\n');
    }

    let chars: Vec<char> = kept.chars().collect();
    let mut found = Vec::new();
    let (mut at, mut line) = (0, 1);
    while at < chars.len() {
        match chars[at] {
            '\n' => line += 1,
            '/' if chars.get(at + 1) == Some(&'/') => {
                while at < chars.len() && chars[at] != '\n' {
                    at += 1;
                }
                continue;
            }
            '"' => {
                at += 1;
                while at < chars.len() && chars[at] != '"' {
                    if chars[at] == '\\' {
                        at = escape(&chars, at, line, &mut found);
                        continue;
                    }
                    if chars[at] == '\n' {
                        line += 1;
                    }
                    found.push((line, chars[at]));
                    at += 1;
                }
            }
            '\'' => {
                if chars.get(at + 1) == Some(&'\\') {
                    at = escape(&chars, at + 1, line, &mut found);
                } else if chars.get(at + 2) == Some(&'\'') {
                    found.push((line, chars[at + 1]));
                    at += 2;
                }
                // Otherwise a lifetime, which holds no glyph.
            }
            _ => {}
        }
        at += 1;
    }
    found
}

/// Step over an escape starting at `at` (the `\`), keeping a decoded `\u{…}`.
fn escape(chars: &[char], at: usize, line: usize, found: &mut Vec<(usize, char)>) -> usize {
    if chars.get(at + 1) == Some(&'u') && chars.get(at + 2) == Some(&'{') {
        let close = chars[at..]
            .iter()
            .position(|&c| c == '}')
            .map_or(chars.len(), |end| at + end);
        let hex: String = chars[at + 3..close].iter().collect();
        if let Some(glyph) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
            found.push((line, glyph));
        }
        return close + 1;
    }
    at + 2
}

/// Everything a TOML line says outside a comment.
fn toml_text(text: &str) -> Vec<(usize, char)> {
    let mut found = Vec::new();
    for (index, row) in text.lines().enumerate() {
        let mut quoted = false;
        for glyph in row.chars() {
            if glyph == '"' {
                quoted = !quoted;
            }
            if glyph == '#' && !quoted {
                break;
            }
            found.push((index + 1, glyph));
        }
    }
    found
}
