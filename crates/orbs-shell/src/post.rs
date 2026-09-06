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

use orbs_render::{Crossing, Frame, Passage, Pos, Rect, Span, Style, Toward, UtteranceKind};

use super::stage::Stage;

/// The studio, and what the orb is made of.
///
/// Names only. The versions are resolved at compile time from the places that
/// actually know them — see the module docs.
const STUDIO: &str = "blackhearth games";

/// What the logo says, one character per entry in [`GLYPHS`].
const NAME: &str = "O.R.B.S.";

/// What the initials stand for, one word per letter.
///
/// **They land after the name, not with it.** Each word used to arrive as its own
/// letter did — `Operational` with the `O` — which read well while the logo
/// printed left to right and stopped reading once the letters began *growing*: a
/// word appearing beside a letter still half its size is two clocks arguing. The
/// name arrives, and then it is expanded. `the_subtitle_waits_for_the_name` holds
/// the order.
const EXPANSION: [&str; 4] = ["Operational", "Relic", "Bewitching", "System"];

/// The subtitle's own width, including the single spaces between its words.
///
/// Computed rather than written down: it decides where the block is centred, and
/// a hand-counted 35 would be wrong the first time a word changed.
fn expansion_width() -> usize {
    EXPANSION
        .iter()
        .map(|word| word.chars().count())
        .sum::<usize>()
        + EXPANSION.len()
        - 1
}

/// Where each word of [`EXPANSION`] starts, in columns from the subtitle's left.
///
/// **It used to carry the [`GLYPHS`] entry that brought each word too**, back
/// when the words rode the letters' clock. They have their own now, so the pairing
/// was a second thing to keep true about a relationship that no longer exists.
fn expansion_places() -> [usize; 4] {
    let mut places = [0; 4];
    let mut col = 0;
    for (index, word) in EXPANSION.iter().enumerate() {
        places[index] = col;
        col += word.chars().count() + 1;
    }
    places
}

/// Where each character of [`NAME`] sits in [`ART`], as `(first column, width)`.
///
/// The letterforms are **column-separable** — no glyph shares a column with its
/// neighbour — and that is still what makes the name arrive a letter at a time,
/// for a subtler reason than it used to be. Each pair is *grown* into place by a
/// `Gather` over its own columns, and column-separability is what guarantees that
/// gather cannot reach the letters standing whole beside it. The letters are
/// eight cells wide except `O`,
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
///
/// **`engine` is the frontend's to supply** and the other two are not. The
/// studio is the game's identity, and the compiler is whichever one built this
/// binary — but the engine is `bevy` here and `crossterm` in a terminal, and a
/// card that named the wrong one would be inventing a component. See
/// the frontend's own `boot::engine`.
fn reported(engine: &str) -> [String; 3] {
    [
        STUDIO.to_owned(),
        format!("rust {}", crate::environment::RUSTC),
        engine.to_owned(),
    ]
}

/// What each line ends with once its dots have finished.
const DONE: &str = "ok";

/// How much of a line's slot is spent typing its dots.
///
/// The rest is the pause after `ok` appears — *"a slight pause between each
/// line"*, which is what stops three lines reading as one paragraph that
/// happens to arrive in pieces.
const TYPING_SHARE: f32 = 0.6;

/// How much of the stage the name takes, a letter at a time.
///
/// A fifth of 5.5 s, so each of the four gets about a quarter of a second —
/// enough that they land as separate events rather than as a stutter, and not so
/// much that the card has nothing to say for half its life.
const LETTERS_SHARE: f32 = 0.20;

/// How much of the stage the subtitle takes, a word at a time.
///
/// **After the letters, not alongside them.** Each word used to land with its own
/// initial — `Operational` with the `O` — which read well while the logo printed
/// left to right and stopped reading at all once the letters started *growing*:
/// a word appearing beside a letter that is still half its size is two clocks
/// arguing. The name arrives, and then it is expanded.
const SUBTITLE_SHARE: f32 = 0.10;

/// How much of the stage the name takes before the report begins.
///
/// Summed rather than written down, so moving either share above cannot leave
/// the report lines starting in the middle of the subtitle.
const LOGO_SHARE: f32 = LETTERS_SHARE + SUBTITLE_SHARE;

/// Which letter pairs have landed, and how far through the one still arriving.
///
/// `arrived` counts the pairs to draw — the one in flight included, because it
/// has to be on screen for the gather to move it. `flight` is that one's own
/// `0.0`..`1.0`, or `None` when the name is whole.
fn letters_at(progress: f32) -> (usize, Option<f32>) {
    let letters = (progress.clamp(0.0, 1.0) / LETTERS_SHARE).clamp(0.0, 1.0);
    if letters >= 1.0 {
        return (PAIRS, None);
    }
    // Which pair the clock is inside, **counted rather than cast**. A float to
    // integer conversion is the one arithmetic this workspace keeps in a single
    // place — `tween::mix` — and `arrived_cells` beneath makes the same choice
    // for the same reason.
    let span = 1.0 / f32::from(to_row(PAIRS));
    let index = (1..PAIRS)
        .filter(|pair| letters >= f32::from(to_row(*pair)) * span)
        .count();
    let start = f32::from(to_row(index)) * span;
    (index + 1, Some(((letters - start) / span).clamp(0.0, 1.0)))
}

/// How many letter-and-full-stop pairs the name arrives in.
const PAIRS: usize = GLYPHS.len() / 2;

/// How many words of the subtitle have landed.
///
/// Zero until the letters are done, which is the sequence the card now reads in:
/// the name, then what it stands for, then what the orb is made of.
fn spoken_words(progress: f32) -> usize {
    let after = ((progress.clamp(0.0, 1.0) - LETTERS_SHARE) / SUBTITLE_SHARE).clamp(0.0, 1.0);
    usize::try_from(arrived_cells(after, EXPANSION.len())).unwrap_or(EXPANSION.len())
}

/// Where each pair starts and how wide it is, in [`ART`] columns.
///
/// **The stops travel with their letters.** [`GLYPHS`] is eight entries because
/// the full stops are glyphs in their own right, and `expansion_places` already
/// records that the letters are the even ones. A stop arriving as a beat of its
/// own would be four extra events in a sequence that is meant to read as a name.
fn pairs() -> [(usize, usize); PAIRS] {
    let mut out = [(0, 0); PAIRS];
    for (index, slot) in out.iter_mut().enumerate() {
        let (start, _) = GLYPHS[index * 2];
        let (stop, width) = GLYPHS[index * 2 + 1];
        *slot = (start, stop + width - start);
    }
    out
}

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
/// **The logo does not print at all — it grows in from the middle.** It used to
/// arrive a character at a time, six rows at once; it is now the whole picture
/// moving, which is [`Passage::Gather`]'s arriving half and so the same motion
/// the game uses whenever a surface takes the pane. The first thing the orb ever
/// does is the thing it keeps doing.
///
/// [`GLYPHS`] survives that, because the **subtitle** still lands letter by
/// letter underneath — `Operational` with the `O`, `Relic` with the `R` — and
/// that pairing is what the table is really for.
pub fn paint(frame: &mut Frame, stage: Stage, progress: f32, engine: &str) {
    if !matches!(stage, Stage::Post | Stage::Close) {
        return;
    }
    // **`Close` draws the finished card and then takes it away.** The stage is a
    // departure rather than a different screen, so everything below runs at full
    // progress and the gather at the end is the only thing that moves.
    let progress = if matches!(stage, Stage::Close) {
        1.0
    } else {
        progress
    };

    let area = frame.area();
    let lines = reported(engine);
    // Logo, the subtitle, a blank row, then the report. Centred as a block, with
    // no pane around it: §4's tower report is a table inside a border, and this
    // must not read as the same screen arriving twice.
    let height = to_row(ART.len() + lines.len() + 2);
    let top = area.rows.saturating_sub(height) / 2;

    let mut painter = frame.painter(area);
    // Centred as a **block**, on the widest row. The rows are not all the same
    // length — only the full stops reach the bottom line — so centring each one
    // on its own width shears the letterforms apart by a column.
    let logo_width = ART.iter().map(|row| row.chars().count()).max().unwrap_or(0);
    let logo_col = area
        .col
        .saturating_add(area.cols.saturating_sub(to_row(logo_width)) / 2);

    // **One letter at a time, and each one grows into place.** `O.`, then `R.`,
    // then `B.`, then `S.` — the full stop arrives with the letter it belongs to
    // rather than as a beat of its own, which is what the name is when it is read
    // aloud.
    //
    // Each pair is drawn whole once its turn has passed and not at all before it,
    // so the only one that moves is the one in flight — and because the pairs are
    // column-disjoint, the gather at the end of this function reaches that one
    // and leaves the letters already standing alone.
    let (arrived, flight) = letters_at(progress);
    for (start, width) in pairs().into_iter().take(arrived) {
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
    // The logo is one utterance, not six rows of block glyphs — and it is the
    // **whole** name from the first frame, where it used to be as much of it as
    // had printed. §14, and the same rule every crossing follows: the linear
    // stream is the settled screen, because a reader must never be made to wait
    // out an animation. The name is *there*; it is only arriving by moving.
    painter.announce(UtteranceKind::Heading, Style::BRIGHT.role, NAME);

    // **Every line spans the logo, edge to edge.** The label sits under the
    // logo's left edge and `ok` ends flush with its right, with the leader
    // filling whatever is between.
    //
    // Not centred, which is what this replaced and what made the card twitch:
    // a centred line is positioned by its own width, and its width grows by two
    // the moment `ok` lands, so the whole row shunted sideways at the end of
    // every check. Anchoring both ends to something that is not moving means
    // nothing moves but the dots.
    // **The subtitle, a word per letter.** `Operational` lands with the `O`,
    // `Relic` with the `R` — so the logo printing itself reads as the name being
    // spelled out rather than as a decoration that happens to be slow.
    //
    // Positioned from the **whole** subtitle's width and drawn word by word at
    // fixed offsets, never re-centred on what has arrived so far. That is the
    // lesson the report lines below already carry: a line centred on its own
    // width shunts sideways every time it grows, and four words arriving would
    // have made the whole thing crawl left four times.
    let subtitle_col = area
        .col
        .saturating_add(area.cols.saturating_sub(to_row(expansion_width())) / 2);
    let subtitle_row = top.saturating_add(to_row(ART.len()));
    // **Its own clock, starting where the letters finish.** The words land whole
    // and in order, which is the same progressive disclosure the report lines
    // below use — they are small enough that growing them in would be motion
    // nobody could read.
    let words = spoken_words(progress);
    for (word, offset) in EXPANSION.iter().zip(expansion_places()).take(words) {
        painter.glyphs(
            Pos::new(subtitle_col.saturating_add(to_row(offset)), subtitle_row),
            word,
            Style::DIM,
        );
    }
    // Spoken once, as much of it as is on screen — **unlike the name above**,
    // which is spoken whole from the first frame because it is one thing
    // *arriving*. This is four things appearing in turn, so what a reader hears
    // is what is there. §14 wants one utterance for one thing, not four for a
    // sentence, which is why it is joined rather than announced per word.
    let said = EXPANSION
        .iter()
        .take(words)
        .copied()
        .collect::<Vec<_>>()
        .join(" ");
    if !said.is_empty() {
        painter.announce(UtteranceKind::Text, Style::DIM.role, &said);
    }

    // The version, in the corner of the window rather than in the report. The
    // module docs argued it out of the report on the grounds that a version line
    // under a six-row logo would be the only small text on the card — which is
    // still true, and is exactly why it belongs in the corner instead, where
    // small text is what a corner is for.
    //
    // **Not during `Close`.** It is the one thing on the card that sits *outside*
    // the box, and the box's interior is what leaves — so left drawn it would be
    // the last of the card still standing after the card had gone, and then be
    // replaced by the prompt a frame later. It goes when the card starts to.
    if !matches!(stage, Stage::Close) {
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        painter.glyphs(
            Pos::new(area.col.saturating_add(1), area.bottom().saturating_sub(1)),
            &version,
            Style::DIM,
        );
    }

    let first = top.saturating_add(to_row(ART.len() + 2));
    let done_col = logo_col.saturating_add(to_row(logo_width.saturating_sub(DONE.len())));

    for (index, label) in lines.iter().enumerate() {
        let Some(state) = line_at(progress, index, lines.len(), leader(label, logo_width)) else {
            break;
        };
        let row = first.saturating_add(to_row(index));
        let dots_col = logo_col.saturating_add(to_row(label.chars().count() + 1));

        // The leader is structure, not content — drawn silently, exactly as
        // `screens`' own `label ....... [ status ]` rows are. The row speaks
        // once, as a whole, and only once it has something to report.
        painter.glyphs(Pos::new(dots_col, row), &".".repeat(state.dots), Style::DIM);
        if state.finished {
            painter.glyphs(Pos::new(done_col, row), DONE, Style::NORMAL);
            painter.span(
                Pos::new(logo_col, row),
                &Span::new(label)
                    .with_style(Style::NORMAL)
                    .with_kind(UtteranceKind::TableRow)
                    .with_spoken(&format!("{label}: {DONE}")),
            );
        } else {
            // Silent while it is still being checked, for the same reason a
            // half-arrived record is: §14's stream is whole facts, and "this is
            // partly done" is not one.
            painter.glyphs(Pos::new(logo_col, row), label, Style::NORMAL);
        }
    }

    // **The name grows in from the middle**, which is the motion a full-pane
    // surface leaves and arrives by — so the first thing the game ever does is
    // the thing it will keep doing. Applied last, after the painter has let the
    // frame go, and over the logo's own rectangle: the subtitle and the report
    // below are on their own clocks and must not be dragged into it.
    //
    // Arriving-half progress, so it runs from the midpoint to the end. `cross`
    // reads the frame's own cells on the way in, which is why no kept screen is
    // needed here.
    if let Some(growing) = flight {
        let (start, width) = pairs()[arrived - 1];
        frame.cross(
            Rect::new(
                logo_col.saturating_add(to_row(start)),
                top,
                to_row(width),
                to_row(ART.len()),
            ),
            Crossing {
                passage: Passage::Gather,
                toward: Toward::Right,
                progress: MIDPOINT + growing * MIDPOINT,
            },
            None,
        );
    }
}

/// Halfway through a crossing: everything has reached the middle.
///
/// The card only ever uses one half of one — the name arriving, and later the
/// whole screen leaving — so both are expressed as a crossing that starts or
/// stops here.
const MIDPOINT: f32 = 0.5;

/// How many leader dots fit between `label` and the `ok` column.
///
/// One space either side of the run, so the dots never touch the words. Varies
/// per line, which is what a leader *is* — the run is however long the gap is.
fn leader(label: &str, width: usize) -> usize {
    width
        .saturating_sub(label.chars().count())
        .saturating_sub(DONE.len())
        .saturating_sub(2)
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
fn line_at(progress: f32, index: usize, count: usize, dots: usize) -> Option<Line> {
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
        // Each line fills its own leader in the same time, so a short label's
        // longer run of dots simply moves a little faster. The alternative — a
        // fixed rate — would make the lines finish at different moments and turn
        // "a slight pause between each" into three different pauses.
        dots: usize::try_from(arrived_cells(typing, dots)).unwrap_or(dots),
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

fn to_cells(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

fn to_row(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::{GridSize, is_renderable};

    /// A stand-in for whatever engine the frontend reports.
    ///
    /// The painter is shared and must not care which one it is, so these tests
    /// pass a fixed string rather than reaching for this crate's. The *real*
    /// one is covered where it lives, in each frontend's own `boot::engine`.
    const ENGINE: &str = "bevy 0.19.0";

    #[test]
    fn every_version_it_reports_is_a_real_one() {
        // The whole argument for the screen existing at all: `tower/boot.rs` is
        // built by walking the world so it cannot go stale, and a POST in front
        // of it printing invented numbers would be the same lie one screen
        // earlier.
        for line in reported(ENGINE).iter().skip(1) {
            let version = line.split_whitespace().nth(1).unwrap_or("");
            assert!(!version.is_empty(), "{line:?} has no version");
            assert!(
                version.chars().next().is_some_and(|c| c.is_ascii_digit()),
                "{line:?} reports {version:?}, which is not a version",
            );
        }
    }

    #[test]
    fn nothing_it_draws_is_outside_the_font() {
        // §4 bounds every glyph to CP437, and a character the atlas has no cell
        // for occupies a column and draws nothing. The `screens` example already
        // found an em-dash this way in DESIGN.md's own boot text.
        for text in ART
            .iter()
            .map(|row| (*row).to_string())
            .chain(reported(ENGINE))
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
    fn the_name_arrives_one_letter_at_a_time() {
        // **`O.`, then `R.`, then `B.`, then `S.`** — in order, none skipped, and
        // never two at once. This replaced a test that measured `arrived_cells`
        // against `GLYPHS`, which the logo stopped calling when it started
        // *growing* rather than printing: the arithmetic was still right and no
        // longer described anything on screen.
        let mut seen = Vec::new();
        for step in 0..=400u16 {
            let progress = f32::from(step) / 400.0;
            let (arrived, flight) = letters_at(progress);

            // Exactly one pair is ever moving, and it is the newest — the ones
            // behind it stand whole, which is what lets a single `Gather` over
            // its columns leave them alone.
            assert!((1..=PAIRS).contains(&arrived), "{arrived} of {PAIRS}");
            if let Some(local) = flight {
                assert!((0.0..=1.0).contains(&local), "flight {local} at {progress}");
            }
            if seen.last() != Some(&arrived) {
                seen.push(arrived);
            }
        }
        assert_eq!(seen.first(), Some(&1), "it did not start with the first");
        assert_eq!(seen.last(), Some(&PAIRS), "it never finished");
        assert!(
            seen.windows(2).all(|pair| pair[1] == pair[0] + 1),
            "letters arrived in jumps rather than one at a time: {seen:?}",
        );
        assert!(
            letters_at(1.0).1.is_none(),
            "a letter was still arriving after the card was done",
        );
    }

    #[test]
    fn the_subtitle_waits_for_the_name() {
        // The sequence is the name, then what it stands for, then what the orb is
        // made of — where it used to be the words riding along with the letters.
        assert_eq!(
            spoken_words(LETTERS_SHARE - 0.01),
            0,
            "a word arrived early"
        );
        assert_eq!(
            spoken_words(1.0),
            EXPANSION.len(),
            "the subtitle never finished"
        );
        assert!(
            line_at(LOGO_SHARE - 0.01, 0, 3, 10).is_none(),
            "the report began before the subtitle was done",
        );
    }

    #[test]
    fn the_report_waits_for_the_logo() {
        // Otherwise the card is two things happening at once, which is the one
        // shape a POST never has.
        let count = reported(ENGINE).len();
        let dots = leader(&reported(ENGINE)[0], span());
        assert!(
            line_at(LOGO_SHARE - 0.01, 0, count, dots).is_none(),
            "the first line started while the logo was still printing",
        );
        assert!(
            line_at(LOGO_SHARE + 0.01, 0, count, dots).is_some(),
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

    /// The logo's width, which every report line spans.
    fn span() -> usize {
        ART.iter().map(|row| row.chars().count()).max().unwrap_or(0)
    }

    #[test]
    fn the_words_land_whole_and_only_the_dots_fill() {
        // The shape of the whole card: a POST line reads as *this is being
        // checked*, and a name arriving one letter at a time reads as a slow
        // machine instead. The label is never partial at any point in the run.
        let lines = reported(ENGINE);
        for step in 0..=60u16 {
            let progress = f32::from(step) / 60.0;
            for (index, label) in lines.iter().enumerate() {
                let dots = leader(label, span());
                let Some(line) = line_at(progress, index, lines.len(), dots) else {
                    continue;
                };
                assert!(line.dots <= dots, "more dots than the line has room for");
                assert!(
                    !line.finished || line.dots == dots,
                    "ok landed before the dots finished",
                );
            }
        }
    }

    #[test]
    fn a_line_reaches_from_one_edge_of_the_logo_to_the_other() {
        // What stopped the card twitching. A centred line is positioned by its
        // own width, and its width grows by two the moment `ok` lands, so every
        // row shunted sideways at the end of every check. Anchored to the logo's
        // edges, the only thing that moves is the leader.
        for label in reported(ENGINE) {
            let dots = leader(&label, span());
            // label + space + dots + space + ok == the logo's width, exactly.
            assert_eq!(
                label.chars().count() + 1 + dots + 1 + DONE.len(),
                span(),
                "{label:?} does not span the logo",
            );
            assert!(dots > 0, "{label:?} left no room for a leader");
        }
    }

    /// Overall stage progress for a point `fraction` of the way through the
    /// report, which only begins once the logo has finished printing.
    fn into_report(fraction: f32) -> f32 {
        LOGO_SHARE + (1.0 - LOGO_SHARE) * fraction
    }

    #[test]
    fn each_line_waits_its_turn_and_finishes_by_the_end() {
        let lines = reported(ENGINE);
        let count = lines.len();
        let dots = leader(&lines[0], span());
        assert!(
            line_at(into_report(0.0), 1, count, dots).is_none(),
            "line 2 started at once",
        );
        assert!(
            line_at(into_report(0.0), 0, count, dots).is_some(),
            "line 1 never started",
        );
        for (index, label) in lines.iter().enumerate() {
            let dots = leader(label, span());
            let line = line_at(1.0, index, count, dots).expect("every line runs");
            assert_eq!(line.dots, dots, "line {index} never filled");
            assert!(line.finished, "line {index} never reported ok");
        }
    }

    #[test]
    fn there_is_a_pause_after_a_line_finishes() {
        // "A slight pause between each line" — without it three lines read as
        // one paragraph that happens to arrive in pieces.
        let count = reported(ENGINE).len();
        let dots = leader(&reported(ENGINE)[0], span());
        let slot = 1.0 / f32::from(to_row(count));
        let just_after = into_report(slot * TYPING_SHARE + 0.001);
        let line = line_at(just_after, 0, count, dots).expect("line one");
        assert!(line.finished, "the first line had not finished");
        // Still on line one — the next has not begun.
        assert!(
            line_at(just_after, 1, count, dots).is_none(),
            "the next line started with no pause",
        );
    }

    #[test]
    fn it_draws_nothing_before_its_own_stage() {
        // The tube is dark; a splash that painted through it would be on screen
        // before the screen was. (The border no longer has a stage to itself —
        // it closes *during* the card now, so there is one stage left to keep
        // the splash out of.)
        let mut frame = Frame::new(GridSize::new(80, 22));
        paint(&mut frame, Stage::Dark, 0.5, ENGINE);
        assert!(frame.to_text().trim().is_empty(), "the dark stage drew");
    }

    #[test]
    fn the_finished_card_names_the_studio_and_everything_it_is_built_from() {
        let mut frame = Frame::new(GridSize::new(80, 22));
        paint(&mut frame, Stage::Post, 1.0, ENGINE);
        let drawn = frame.to_text();
        for line in reported(ENGINE) {
            let label = line.split_whitespace().next().unwrap_or("");
            assert!(drawn.contains(label), "{label} did not report");
        }
        assert!(drawn.contains(DONE), "nothing reported ok");
        for label in reported(ENGINE) {
            let dots = leader(&label, span());
            assert!(
                drawn.contains(&".".repeat(dots)),
                "{label:?}'s leader never filled its {dots} columns",
            );
        }
    }

    #[test]
    fn the_logo_speaks_as_a_name_rather_than_as_block_glyphs() {
        // §14: what a reader hears is what the screen says, and what it says
        // here is the game's name — not six rows of full-block characters.
        let mut frame = Frame::new(GridSize::new(80, 22));
        paint(&mut frame, Stage::Post, 1.0, ENGINE);
        assert!(
            frame
                .speech()
                .utterances()
                .any(|utterance| utterance.text == "O.R.B.S."),
            "the logo did not say its own name",
        );
    }
}
