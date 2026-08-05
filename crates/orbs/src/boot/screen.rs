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

/// What the logo says, one character per entry in [`GLYPHS`].
const NAME: &str = "O.R.B.S.";

/// Where each character of [`NAME`] sits in [`ART`], as `(first column, width)`.
///
/// The letterforms are **column-separable** — no glyph shares a column with its
/// neighbour — which is what lets the logo be printed a character at a time
/// rather than appearing whole. The letters are eight cells wide except `O`,
/// which is nine, and the full stops are three; nothing about that is regular
/// enough to compute, so it is a table, and `the_glyph_table_tiles_the_art`
/// holds it to covering the art exactly with no gap and no overlap.
const GLYPHS: [(usize, usize); 8] = [
    (0, 9),
    (9, 3),
    (12, 8),
    (20, 3),
    (23, 8),
    (31, 3),
    (34, 8),
    (42, 3),
];

/// The game's own name, in the block glyphs CP437 has for exactly this.
///
/// Every character is in the repertoire — full block plus the double box-drawing
/// set — which `nothing_it_draws_is_outside_the_font` holds it to. 45 cells wide,
/// so it fits §4's 80-column floor with sixteen columns either side.
///
/// The first four rows are shorter than the last two: only the full stops reach
/// the bottom line, and trailing spaces are not padded on. Every column index in
/// [`GLYPHS`] is therefore beyond the end of some row, which [`columns`] treats
/// as empty rather than as an error.
const ART: [&str; 6] = [
    r" ██████╗    ██████╗    ██████╗    ███████╗",
    r"██╔═══██╗   ██╔══██╗   ██╔══██╗   ██╔════╝",
    r"██║   ██║   ██████╔╝   ██████╔╝   ███████╗",
    r"██║   ██║   ██╔══██╗   ██╔══██╗   ╚════██║",
    r"╚██████╔╝██╗██║  ██║██╗██████╔╝██╗███████║██╗",
    r" ╚═════╝ ╚═╝╚═╝  ╚═╝╚═╝╚═════╝ ╚═╝╚══════╝╚═╝",
];

/// What the POST reports, in the order it reports them.
///
/// The studio first, then what the orb is built out of. The game's own version
/// is not among them: [`ART`] is the game saying its name, and a version line
/// under a six-row logo would be the only small text on the card.
fn reported() -> [String; 3] {
    [
        STUDIO.to_owned(),
        format!("rust {}", env!("ORBS_RUSTC")),
        format!("bevy {BEVY}"),
    ]
}

/// Leader dots between a label and its `ok`.
///
/// The part that types. Five is enough to read as waiting and few enough that
/// the line does not become mostly punctuation.
const DOTS: usize = 5;

/// What each line ends with once its dots have finished.
const DONE: &str = "ok";

/// How much of a line's slot is spent typing its dots.
///
/// The rest is the pause after `ok` appears — *"a slight pause between each
/// line"*, which is what stops three lines reading as one paragraph that
/// happens to arrive in pieces.
const TYPING_SHARE: f32 = 0.6;

/// How much of the stage the logo takes to print itself.
///
/// The rest belongs to the report lines. Enough that the letters land as
/// separate events rather than as a stutter, and not so much that the card has
/// nothing to say for half its life.
const LOGO_SHARE: f32 = 0.3;

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
/// # What types and what does not
///
/// **The words do not type; the dots do.** A name arriving one letter at a time
/// reads as a slow machine, which is the opposite of the point — a POST line
/// should read as *this thing is being checked*. So the label lands whole, its
/// leader dots fill the way a progress indicator fills, and `ok` snaps in behind
/// them. Then a pause, and the next line.
///
/// The logo prints a **character at a time** as well, six rows at once — `O`,
/// then the full stop, then `R`, and so on. The letterforms are column-separable
/// ([`GLYPHS`]), which is the only reason that is possible.
pub(crate) fn paint(frame: &mut Frame, stage: Stage, progress: f32) {
    if !matches!(stage, Stage::Post) {
        return;
    }

    let area = frame.area();
    let lines = reported();
    // Logo, a blank row, then the report. Centred as a block, with no pane
    // around it: §4's tower report is a table inside a border, and this must not
    // read as the same screen arriving twice.
    let height = to_row(ART.len() + lines.len() + 1);
    let top = area.rows.saturating_sub(height) / 2;

    let mut painter = frame.painter(area);
    // Centred as a **block**, on the widest row. The rows are not all the same
    // length — only the full stops reach the bottom line — so centring each one
    // on its own width shears the letterforms apart by a column.
    let logo_width = ART.iter().map(|row| row.chars().count()).max().unwrap_or(0);
    let logo_col = area
        .col
        .saturating_add(area.cols.saturating_sub(to_row(logo_width)) / 2);

    let printed = usize::try_from(arrived_cells(
        (progress / LOGO_SHARE).clamp(0.0, 1.0),
        GLYPHS.len(),
    ))
    .unwrap_or(GLYPHS.len());

    for &(start, width) in GLYPHS.iter().take(printed) {
        for (index, row) in ART.iter().enumerate() {
            painter.glyphs(
                Pos::new(
                    logo_col.saturating_add(to_row(start)),
                    top.saturating_add(to_row(index)),
                ),
                columns(row, start, width),
                Style::BRIGHT,
            );
        }
    }
    // The logo is one utterance, not six rows of block glyphs, and it says only
    // as much of the name as is on screen. §14: what a reader hears is what the
    // screen says.
    if printed > 0 {
        painter.announce(
            UtteranceKind::Heading,
            Style::BRIGHT.role,
            orbs_render::arriving(NAME, to_cells(printed)),
        );
    }

    // Labels padded to a common width so every `ok` lands in one column — the
    // shape a POST has, and the reason the dots read as a leader rather than as
    // punctuation somebody typed.
    let widest = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    let first = top.saturating_add(to_row(ART.len() + 1));

    for (index, label) in lines.iter().enumerate() {
        let Some(state) = line_at(progress, index, lines.len()) else {
            break;
        };
        let text = format!(
            "{label:widest$} {dots:<DOTS$} {done}",
            dots = ".".repeat(state.dots),
            done = if state.finished { DONE } else { "" },
        );
        // Positioned by the *finished* width, not the visible one, so a line
        // fills from where it will end up rather than sliding as it grows.
        let at = centred(area, &text, first.saturating_add(to_row(index)));
        painter.span(
            at,
            &Span::new(text.trim_end())
                .with_style(Style::NORMAL)
                .with_kind(UtteranceKind::TableRow),
        );
    }
}

/// The columns `start..start + width` of `row`, by character.
///
/// Borrowed rather than collected: this runs for every glyph of every row of
/// every frame the logo is printing, and the art is `&'static str`. A column
/// past the end of a short row yields nothing, which is what makes the four
/// unpadded rows work.
fn columns(row: &str, start: usize, width: usize) -> &str {
    let begin = row
        .char_indices()
        .nth(start)
        .map_or(row.len(), |(index, _)| index);
    let end = row
        .char_indices()
        .nth(start + width)
        .map_or(row.len(), |(index, _)| index);
    row.get(begin..end).unwrap_or("")
}

/// How far line `index` of `count` has got at `progress`.
///
/// `None` before the line's turn. The report begins once the logo has finished
/// printing, and each line owns an equal share of what is left:
/// [`TYPING_SHARE`] of it filling the dots, the rest holding `ok` on screen
/// before the next one starts.
fn line_at(progress: f32, index: usize, count: usize) -> Option<Line> {
    let after_logo = ((progress.clamp(0.0, 1.0) - LOGO_SHARE) / (1.0 - LOGO_SHARE)).clamp(0.0, 1.0);
    if progress.clamp(0.0, 1.0) < LOGO_SHARE {
        return None;
    }
    let span = 1.0 / f32::from(to_row(count).max(1));
    let start = f32::from(to_row(index)) * span;
    if after_logo < start {
        return None;
    }
    let local = ((after_logo - start) / span).clamp(0.0, 1.0);
    let typing = (local / TYPING_SHARE).clamp(0.0, 1.0);
    Some(Line {
        dots: usize::try_from(arrived_cells(typing, DOTS)).unwrap_or(DOTS),
        // `ok` lands the instant the dots do, not gradually — the line has
        // finished being checked, and a two-letter word fading in would be the
        // only thing on the card that was still arriving.
        finished: typing >= 1.0,
    })
}

/// One report line, mid-flight.
struct Line {
    dots: usize,
    finished: bool,
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
        for line in reported().iter().skip(1) {
            let version = line.split_whitespace().nth(1).unwrap_or("");
            assert!(!version.is_empty(), "{line:?} has no version");
            assert!(
                version.chars().next().is_some_and(|c| c.is_ascii_digit()),
                "{line:?} reports {version:?}, which is not a version",
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
        for text in ART.iter().map(|row| (*row).to_string()).chain(reported()) {
            for glyph in text.chars() {
                assert!(
                    is_renderable(glyph),
                    "{glyph:?} in {text:?} cannot be drawn"
                );
            }
        }
    }

    #[test]
    fn the_glyph_table_tiles_the_art() {
        // The table is hand-written because the letterforms are not a regular
        // width — `O` is nine cells, the others eight, the full stops three.
        // Hand-written means it can drift from the art, and the failure would be
        // silent: a glyph drawn one column off, or a sliver never drawn at all.
        let width = ART.iter().map(|row| row.chars().count()).max().unwrap_or(0);
        let mut expected = 0;
        for (start, glyph) in GLYPHS {
            assert_eq!(start, expected, "a gap or overlap at column {start}");
            expected = start + glyph;
        }
        assert_eq!(expected, width, "the table covers {expected} of {width}");
        assert_eq!(
            GLYPHS.len(),
            NAME.chars().count(),
            "the table and the name disagree about how many characters there are",
        );
    }

    #[test]
    fn no_glyph_shares_a_column_with_its_neighbour() {
        // What makes printing the logo a character at a time possible at all. If
        // two letterforms overlapped, revealing one would draw part of the next.
        for (index, &(start, width)) in GLYPHS.iter().enumerate() {
            for row in ART {
                let slice = columns(row, start, width);
                assert!(
                    slice.chars().count() <= width,
                    "glyph {index} spills past its {width} columns",
                );
            }
        }
    }

    #[test]
    fn the_logo_prints_one_character_at_a_time() {
        // Left to right, and every character eventually. The whole point of the
        // column table.
        let mut seen = Vec::new();
        for step in 0..=60u16 {
            let progress = f32::from(step) / 60.0;
            let printed = usize::try_from(arrived_cells(
                (progress / LOGO_SHARE).clamp(0.0, 1.0),
                GLYPHS.len(),
            ))
            .unwrap_or(0);
            if seen.last() != Some(&printed) {
                seen.push(printed);
            }
        }
        assert_eq!(seen.first(), Some(&0), "the logo was there from the start");
        assert_eq!(seen.last(), Some(&GLYPHS.len()), "it never finished");
        assert!(
            seen.windows(2).all(|pair| pair[1] == pair[0] + 1),
            "it printed in jumps rather than one at a time: {seen:?}",
        );
    }

    #[test]
    fn the_report_waits_for_the_logo() {
        // Otherwise the card is two things happening at once, which is the one
        // shape a POST never has.
        let count = reported().len();
        assert!(
            line_at(LOGO_SHARE - 0.01, 0, count).is_none(),
            "the first line started while the logo was still printing",
        );
        assert!(
            line_at(LOGO_SHARE + 0.01, 0, count).is_some(),
            "the report never started",
        );
    }

    #[test]
    fn the_logo_fits_the_floor_with_room_to_spare() {
        // §4's floor is 80 columns. A logo that clipped there would clip on the
        // window the game *opens* at, which is the one grid it is guaranteed to
        // be looked at on.
        let widest = ART.iter().map(|row| row.chars().count()).max().unwrap_or(0);
        assert!(widest <= 72, "the logo is {widest} cells wide");
    }

    #[test]
    fn the_words_land_whole_and_only_the_dots_fill() {
        // The shape of the whole card: a POST line reads as *this is being
        // checked*, and a name arriving one letter at a time reads as a slow
        // machine instead. The label is never partial at any point in the run.
        for step in 0..=60u16 {
            let progress = f32::from(step) / 60.0;
            for index in 0..reported().len() {
                let Some(line) = line_at(progress, index, reported().len()) else {
                    continue;
                };
                assert!(line.dots <= DOTS, "more dots than the line has");
                assert!(
                    !line.finished || line.dots == DOTS,
                    "ok landed before the dots finished",
                );
            }
        }
    }

    /// Overall stage progress for a point `fraction` of the way through the
    /// report, which only begins once the logo has finished printing.
    fn into_report(fraction: f32) -> f32 {
        LOGO_SHARE + (1.0 - LOGO_SHARE) * fraction
    }

    #[test]
    fn each_line_waits_its_turn_and_finishes_by_the_end() {
        let count = reported().len();
        assert!(
            line_at(into_report(0.0), 1, count).is_none(),
            "line 2 started at once",
        );
        assert!(
            line_at(into_report(0.0), 0, count).is_some(),
            "line 1 never started",
        );
        for index in 0..count {
            let line = line_at(1.0, index, count).expect("every line runs");
            assert_eq!(line.dots, DOTS, "line {index} never filled");
            assert!(line.finished, "line {index} never reported ok");
        }
    }

    #[test]
    fn there_is_a_pause_after_a_line_finishes() {
        // "A slight pause between each line" — without it three lines read as
        // one paragraph that happens to arrive in pieces.
        let count = reported().len();
        let span = 1.0 / f32::from(to_row(count));
        let just_after = into_report(span * TYPING_SHARE + 0.001);
        let line = line_at(just_after, 0, count).expect("line one");
        assert!(line.finished, "the first line had not finished");
        // Still on line one — the next has not begun.
        assert!(
            line_at(just_after, 1, count).is_none(),
            "the next line started with no pause",
        );
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
    fn the_finished_card_names_the_studio_and_everything_it_is_built_from() {
        let mut frame = Frame::new(GridSize::new(80, 22));
        paint(&mut frame, Stage::Post, 1.0);
        let drawn = frame.to_text();
        for line in reported() {
            let label = line.split_whitespace().next().unwrap_or("");
            assert!(drawn.contains(label), "{label} did not report");
        }
        assert!(drawn.contains(DONE), "nothing reported ok");
        assert!(
            drawn.contains(&".".repeat(DOTS)),
            "the leaders never filled"
        );
    }

    #[test]
    fn the_logo_speaks_as_a_name_rather_than_as_block_glyphs() {
        // §14: what a reader hears is what the screen says, and what it says
        // here is the game's name — not six rows of full-block characters.
        let mut frame = Frame::new(GridSize::new(80, 22));
        paint(&mut frame, Stage::Post, 1.0);
        assert!(
            frame
                .speech()
                .utterances()
                .any(|utterance| utterance.text == "O.R.B.S."),
            "the logo did not say its own name",
        );
    }
}
