//! The machine's POST, drawn.
//!
//! **Names and facts, no sentences** — rule 6, and the line §19 already drew for
//! `Verb::canonical` and `tower/build.rs`. There is not a sentence in this file,
//! and the moment the splash wants one it belongs in Phase 1's content file
//! rather than here.
//!
//! # Why the versions are real
//!
//! `tower/boot.rs` exists because *"a boot report that could go stale would be a
//! lie the player reads first"*. A POST screen reporting invented version
//! numbers would be the same lie one screen earlier, and it costs nothing to
//! avoid: the game's version is its own package's, and Bevy's is the exact pin,
//! held to it by a test rather than by memory.
//!
//! The Rust version is the one that actually compiled the binary, captured by
//! `build.rs`. `env!("CARGO_PKG_RUST_VERSION")` was the obvious choice and is
//! **empty here** — this crate does not inherit `rust-version` — and the
//! workspace's `1.95` is a floor rather than the compiler in use, which
//! `rust-toolchain.toml` pins to something else again. Three numbers, only one
//! of them true.

use orbs_render::{Frame, Pos, Rect, Span, Style, UtteranceKind};

use super::stage::Stage;

/// The studio, and what the orb is made of.
///
/// Names only. The versions are resolved at compile time from the places that
/// actually know them — see the module docs.
const STUDIO: &str = "blackhearth games";

/// One dependency, as the POST reports it.
struct Part {
    name: &'static str,
    version: &'static str,
}

const PARTS: [Part; 3] = [
    Part {
        name: "rust",
        version: env!("ORBS_RUSTC"),
    },
    Part {
        name: "bevy",
        version: BEVY,
    },
    Part {
        name: "orbs",
        version: env!("CARGO_PKG_VERSION"),
    },
];

/// The exact Bevy pin, held to `Cargo.toml` by a test.
///
/// A const rather than a build script because the version is *pinned* — CLAUDE.md
/// commits to `=0.19.0` and upgrading it is a deliberate one-window act in Phase
/// 3c, so a number that can only change when someone edits the manifest is
/// exactly as live as it needs to be.
const BEVY: &str = "0.19.0";

/// How much of the ellipsis has arrived, in dots.
const DOTS: usize = 3;

/// Paint the boot screen for `stage`, `progress` of the way through it.
///
/// Takes no Bevy resources on purpose: `shell::dump` builds no `App`, so a screen
/// that needed one could never be dumped as text.
pub(crate) fn paint(frame: &mut Frame, stage: Stage, progress: f32) {
    if !matches!(stage, Stage::Post) {
        return;
    }

    let area = frame.area();
    let mut painter = frame.painter(area);

    // Centred, and with no pane around it. §4's tower report is a table inside a
    // border; this must not read as the same screen arriving twice, so it is
    // shaped like a title card instead.
    let rows = to_row(PARTS.len()).saturating_add(2);
    let top = area.rows.saturating_sub(rows) / 2;

    painter.span(
        centred(area, "O.R.B.S.", top),
        &Span::new("O.R.B.S.")
            .with_style(Style::BRIGHT)
            .with_kind(UtteranceKind::Heading),
    );
    painter.span(
        centred(area, STUDIO, top.saturating_add(1)),
        &Span::new(STUDIO)
            .with_style(Style::DIM)
            .with_kind(UtteranceKind::Text),
    );

    // Each part reports in turn, so the list fills rather than appearing whole.
    let arrived = arrived(progress, PARTS.len());
    for (index, part) in PARTS.iter().enumerate().take(arrived) {
        let last = index + 1 == arrived;
        let row = top.saturating_add(3).saturating_add(to_row(index));
        let line = if last && progress < 1.0 {
            format!("{} {} {}", part.name, part.version, ellipsis(progress))
        } else {
            format!("{} {} ok", part.name, part.version)
        };
        painter.span(
            centred(area, &line, row),
            &Span::new(&line)
                .with_style(if last && progress < 1.0 {
                    Style::DIM
                } else {
                    Style::NORMAL
                })
                .with_kind(UtteranceKind::TableRow),
        );
    }
}

/// How many parts have reported by `progress`.
///
/// At least one immediately: a POST that shows nothing for its first third looks
/// like a hang rather than a boot.
///
/// Counted by comparison rather than by rounding a product, which keeps every
/// number here an integer — `orbs-render` confines its one float-to-integer
/// conversion to a single justified function, and a splash screen is not the
/// place to open a second front.
fn arrived(progress: f32, parts: usize) -> usize {
    let total = f32::from(to_row(parts));
    let reached = progress.clamp(0.0, 1.0) * total;
    (1..=parts)
        .filter(|index| reached > f32::from(to_row(index - 1)))
        .count()
        .clamp(1, parts)
}

/// The animated ellipsis, cycling within the current part.
fn ellipsis(progress: f32) -> &'static str {
    const DOTTED: [&str; DOTS + 1] = ["", ".", "..", "..."];
    /// Ellipsis steps across the whole stage: three full cycles of four.
    const STEPS: u16 = 12;

    let scaled = progress.clamp(0.0, 1.0) * f32::from(STEPS);
    let phase = (1..STEPS).filter(|step| scaled >= f32::from(*step)).count();
    DOTTED[phase % DOTTED.len()]
}

/// Where `text` starts if it is centred on `row`.
fn centred(area: Rect, text: &str, row: u16) -> Pos {
    let width = to_row(text.chars().count());
    let col = area.cols.saturating_sub(width) / 2;
    Pos::new(area.col.saturating_add(col), area.row.saturating_add(row))
}

fn to_row(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::{GridSize, is_renderable};

    #[test]
    fn every_version_it_reports_is_a_real_one() {
        // The whole argument for the screen existing at all: `tower/boot.rs` is
        // built by walking the world so it cannot go stale, and a POST in front
        // of it printing invented numbers would be the same lie one screen
        // earlier.
        for part in &PARTS {
            assert!(!part.version.is_empty(), "{} has no version", part.name);
            assert!(
                part.version
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit()),
                "{} reports {:?}, which is not a version",
                part.name,
                part.version,
            );
        }
    }

    #[test]
    fn the_bevy_version_is_the_one_the_manifest_pins() {
        // CLAUDE.md pins Bevy exactly and upgrading is a deliberate Phase 3c
        // act. This is what stops the splash drifting away from the manifest
        // silently when that window arrives.
        let manifest = include_str!("../../Cargo.toml");
        assert!(
            manifest.contains(&format!("\"={BEVY}\"")),
            "the splash says bevy {BEVY}, which Cargo.toml does not pin",
        );
    }

    #[test]
    fn nothing_it_draws_is_outside_the_font() {
        // §4 bounds every glyph to CP437, and a character the atlas has no cell
        // for occupies a column and draws nothing. The `screens` example already
        // found an em-dash this way in DESIGN.md's own boot text.
        for text in [STUDIO, "O.R.B.S."]
            .into_iter()
            .chain(PARTS.iter().map(|part| part.name))
            .chain(PARTS.iter().map(|part| part.version))
        {
            for glyph in text.chars() {
                assert!(
                    is_renderable(glyph),
                    "{glyph:?} in {text:?} cannot be drawn"
                );
            }
        }
    }

    #[test]
    fn something_is_reporting_from_the_first_frame() {
        // A POST that shows nothing for its first third reads as a hang.
        assert_eq!(arrived(0.0, 3), 1);
        assert_eq!(arrived(1.0, 3), 3);
    }

    #[test]
    fn it_draws_nothing_before_its_own_stage() {
        // The tube is dark and the frame is still drawing itself; a splash that
        // painted through those would be on screen before the screen was.
        for stage in [Stage::Dark, Stage::Strike, Stage::Prompt, Stage::Frame] {
            let mut frame = Frame::new(GridSize::new(80, 22));
            paint(&mut frame, stage, 0.5);
            assert!(
                frame.to_text().trim().is_empty(),
                "{stage:?} drew the splash",
            );
        }
    }

    #[test]
    fn the_card_is_centred_and_names_the_studio() {
        let mut frame = Frame::new(GridSize::new(80, 22));
        paint(&mut frame, Stage::Post, 1.0);
        let drawn = frame.to_text();
        assert!(drawn.contains("O.R.B.S."));
        assert!(drawn.contains(STUDIO));
        for part in &PARTS {
            assert!(drawn.contains(part.name), "{} did not report", part.name);
        }
    }
}
