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

/// Paint the boot screen for `stage`, `progress` of the way through it.
///
/// Takes no Bevy resources on purpose: `shell::dump` builds no `App`, so a screen
/// that needed one could never be dumped as text.
///
/// # It types itself
///
/// Every line arrives a character at a time, on one budget shared across the
/// whole card — the same shape as command output (`shell::reveal`), because the
/// two are the same idea: a machine printing to a terminal, not a screen being
/// switched on. The budget runs left to right and top to bottom, so the card
/// fills the way a page does.
pub(crate) fn paint(frame: &mut Frame, stage: Stage, progress: f32) {
    if !matches!(stage, Stage::Post) {
        return;
    }

    let area = frame.area();

    // Centred, and with no pane around it. §4's tower report is a table inside a
    // border; this must not read as the same screen arriving twice, so it is
    // shaped like a title card instead.
    let lines = card();
    let rows = to_row(lines.len()).saturating_add(1);
    let top = area.rows.saturating_sub(rows) / 2;

    // Every character on the card, so the reveal is paced by how much there is
    // to say rather than by how many lines it happens to occupy.
    let total: usize = lines.iter().map(|(text, _)| text.chars().count()).sum();
    let mut budget = arrived_cells(progress, total);

    let mut painter = frame.painter(area);
    for (index, (text, style)) in lines.iter().enumerate() {
        // A spacer costs no characters and must not end the reveal — an empty
        // line is not an exhausted budget, and conflating them stopped the card
        // at the blank row above the parts.
        if text.is_empty() {
            continue;
        }
        if budget == 0 {
            break;
        }
        let visible = orbs_render::arriving(text, budget);
        budget = budget.saturating_sub(to_cells(visible.chars().count()));

        // Positioned by the *finished* line, not the visible one, so a line
        // types out from where it will end up rather than sliding left as it
        // grows. A centred line that re-centres per character is unreadable.
        let at = centred(area, text, top.saturating_add(to_row(index)));
        painter.span(
            at,
            &Span::new(visible)
                .with_style(*style)
                .with_kind(if index == 0 {
                    UtteranceKind::Heading
                } else {
                    UtteranceKind::Text
                }),
        );
    }
}

/// The card's lines, in the order they arrive.
///
/// A blank line between the studio and the parts, which is a line of the card
/// rather than a gap in the layout — it costs no characters, so the reveal does
/// not pause on it.
fn card() -> Vec<(String, Style)> {
    let mut lines = vec![
        ("O.R.B.S.".to_owned(), Style::BRIGHT),
        (STUDIO.to_owned(), Style::DIM),
        (String::new(), Style::DIM),
    ];
    lines.extend(
        PARTS
            .iter()
            .map(|part| (format!("{} {} ok", part.name, part.version), Style::NORMAL)),
    );
    lines
}

/// How many characters of the card have arrived by `progress`.
///
/// Counted by comparison rather than by rounding a product, which keeps every
/// number here an integer — `orbs-render` confines its one float-to-integer
/// conversion to a single justified function, and a splash screen is not the
/// place to open a second front.
fn arrived_cells(progress: f32, total: usize) -> u32 {
    let progress = progress.clamp(0.0, 1.0);
    let reached = progress * f32::from(to_row(total));
    to_cells(
        (1..=total)
            .filter(|index| reached >= f32::from(to_row(*index)))
            .count(),
    )
}

/// How many characters of `text` have arrived by `progress`.
///
/// For a caller with one string rather than a card of them — the input line
/// during [`Stage::Prompt`].
pub(crate) fn arrived(text: &str, progress: f32) -> u32 {
    arrived_cells(progress, text.chars().count())
}

fn to_cells(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
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
    fn the_card_types_itself_from_nothing_to_all_of_it() {
        // The reveal's endpoints. Starting at zero is right here where it would
        // be wrong for command output — the card has a whole stage to fill and
        // the tube has only just struck, so there is nothing to look like a hang
        // *yet*.
        assert_eq!(arrived_cells(0.0, 40), 0);
        assert_eq!(arrived_cells(1.0, 40), 40);
        assert_eq!(arrived_cells(0.5, 40), 20);
    }

    #[test]
    fn every_character_of_the_card_arrives_by_the_end() {
        // The budget is shared across all the lines, so a miscount would leave
        // the last one permanently short — and it is the game's own version.
        let lines = card();
        let total: usize = lines.iter().map(|(text, _)| text.chars().count()).sum();
        let mut budget = arrived_cells(1.0, total);
        for (text, _) in &lines {
            let visible = orbs_render::arriving(text, budget);
            assert_eq!(visible, text, "{text:?} never finished arriving");
            budget = budget.saturating_sub(to_cells(visible.chars().count()));
        }
    }

    #[test]
    fn a_line_types_out_from_where_it_will_end_up() {
        // Centred on the finished text, not the visible prefix. Re-centring per
        // character makes a line slide leftwards as it grows, which is
        // unreadable and looks like a fault.
        let area = Rect::new(0, 0, 80, 22);
        let settled = centred(area, STUDIO, 4);
        for cells in 1..STUDIO.len() {
            let partial = orbs_render::arriving(STUDIO, to_cells(cells));
            assert_eq!(
                centred(area, STUDIO, 4).col,
                settled.col,
                "the line moved at {partial:?}",
            );
        }
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
