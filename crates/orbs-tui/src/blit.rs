//! A [`Frame`] onto a terminal — this frontend's whole half of rule 2.
//!
//! `orbs-render` said what appears and where; `orbs-shell` said what the screen
//! is. All that is left is writing the cells out, and the only decisions here
//! are how a glyph is encoded and what colour it takes.
//!
//! Only the differences are written: a full 120×45 repaint is 5,400 cells of
//! escape sequences thirty times a second, which is nothing over a local pty
//! and a visible smear over `ssh`. A shadow buffer holds what is on screen.
//!
//! The diff key is `(glyph, ink)`, not `Cell`, which derives `PartialEq` and
//! would look right: a tint is a *region*, so a cell can be byte-identical
//! while the wash over it changed. The flask's `green+bone` band growing is one
//! `█` at a time, same glyph, new colour. Resolve first, then compare.

use std::io::Write;

use crossterm::style::{Attribute, Color, Print, SetAttribute, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, queue};
use orbs_render::{Frame, GridSize, Pos};

use crate::theme::{self, Ink, Weight};

/// What the terminal is currently showing, one entry per cell.
pub(crate) struct Screen {
    /// Row-major, `grid.area()` long.
    cells: Vec<(char, Ink)>,
    grid: GridSize,
    /// Whether this terminal draws the game's symbols one column wide.
    ///
    /// See [`crate::term::symbols_are_narrow`]. `false` swaps the at-risk glyphs
    /// for ASCII the terminal cannot disagree about.
    narrow: bool,
    /// Whether the terminal must be wiped before the next draw — the half of a
    /// resize the buffer alone cannot express. See [`Screen::resize`].
    wipe: bool,
}

/// A blank cell, as the shadow buffer starts.
const BLANK: (char, Ink) = (' ', Ink::PLAIN);

impl Screen {
    /// A shadow buffer for a terminal of this size.
    pub(crate) fn new(grid: GridSize, narrow: bool) -> Self {
        Self {
            cells: vec![BLANK; grid.area()],
            grid,
            narrow,
            // The alternate screen arrives blank, and a wipe here would put a
            // visible clear between entering it and the first paint.
            wipe: false,
        }
    }

    /// Forget everything, because the terminal is a different size or has been
    /// scribbled on.
    ///
    /// Every resize calls this: the diff is addressed by `(col, row)`, so a
    /// buffer sized for the old grid writes this frame's cells at last frame's
    /// coordinates.
    ///
    /// The terminal is wiped too, because "every cell is blank" is a lie about
    /// the screen: cells the new frame leaves blank compare equal and are
    /// skipped while the old glyph is still showing. Queued into the same flush
    /// as the redraw, so no frame in between is empty.
    pub(crate) fn resize(&mut self, grid: GridSize) {
        self.grid = grid;
        self.cells.clear();
        self.cells.resize(grid.area(), BLANK);
        self.wipe = true;
    }

    /// Write the cells that changed, and put the cursor where the Frame says.
    ///
    /// # Errors
    ///
    /// If the terminal cannot be written to.
    pub(crate) fn draw(&mut self, frame: &Frame, out: &mut impl Write) -> std::io::Result<()> {
        // A resize between poll and paint is normal; the Frame is the authority.
        if frame.size() != self.grid {
            self.resize(frame.size());
        }
        // In the same flush as the cells that follow: a wipe on its own write
        // would put one empty frame on screen.
        if std::mem::take(&mut self.wipe) {
            queue!(out, Clear(ClearType::All))?;
        }

        // Track what the terminal is already set to, so a run of cells in one
        // colour costs one escape sequence rather than one per cell.
        let mut pen: Option<Ink> = None;
        let mut at: Option<(u16, u16)> = None;

        for (row, cells) in frame.rows().enumerate() {
            let Ok(row) = u16::try_from(row) else {
                break;
            };
            for (col, cell) in cells.iter().enumerate() {
                let Ok(col) = u16::try_from(col) else {
                    break;
                };
                // Both side-tables, read per cell: a cell can be byte-identical
                // while the region over it changed.
                let here = Pos::new(col, row);
                let ink = theme::resolve(cell.style, frame.tint_at(here), frame.lit_at(here));
                let glyph = self.glyph(cell.glyph);
                let index = self.index(col, row);
                if self.cells[index] == (glyph, ink) {
                    continue;
                }
                self.cells[index] = (glyph, ink);

                // Only move when the last write did not leave us here.
                if at != Some((col, row)) {
                    queue!(out, cursor::MoveTo(col, row))?;
                }
                if pen != Some(ink) {
                    set_pen(out, ink)?;
                    pen = Some(ink);
                }
                queue!(out, Print(glyph))?;
                at = Some((col.saturating_add(1), row));
            }
        }

        // The real terminal cursor, which `orbs-render` keeps off `Cell` so the
        // input line works with line editing and a screen reader can track it.
        if let Some(pos) = frame.cursor() {
            queue!(out, cursor::MoveTo(pos.col, pos.row), cursor::Show)?;
        } else {
            queue!(out, cursor::Hide)?;
        }
        out.flush()
    }

    const fn index(&self, col: u16, row: u16) -> usize {
        row as usize * self.grid.cols as usize + col as usize
    }

    /// The glyph this terminal can actually draw in one column.
    const fn glyph(&self, glyph: char) -> char {
        if self.narrow { glyph } else { narrowed(glyph) }
    }
}

/// An ASCII stand-in for a glyph a wide-ambiguous terminal would give two
/// columns to.
///
/// Only the symbols the painters actually use are listed; anything else is left
/// alone.
///
/// The borders and the bars stay out although `─ │ ═ ║ ╔ ╗ ╚ ╝` and `█ ▓ ▒` all
/// measure East Asian Ambiguous (only `░` is Neutral), on the bet that a
/// terminal widening box drawing would break every full-screen program on the
/// system. So the wide-ambiguous path is partial: `term::PROBE` firing means
/// the symbols are safe, not the screen.
///
/// Every substitution is distinct (§14): the glyph carries the identity where
/// the colour cannot. The first version mapped `•` and `·` both to `.`, merging
/// *taken* into *locked* on a surface made entirely of those glyphs, and
/// invisibly — the row still lines up.
/// `every_substitution_is_its_own_character` holds the map injective.
const SUBSTITUTIONS: [(char, char); 24] = [
    // The rail's fault marker and its "this room is automated" arrow.
    ('‼', '!'),
    ('►', '>'),
    ('◄', '<'),
    // The ward's six sigils.
    ('☼', '*'),
    ('♂', 'm'),
    ('♀', 'f'),
    ('♦', 'd'),
    ('♠', 's'),
    // The three states, kept three: filled, hollow, faint — taken from open
    // from locked, aligned from astray from loose.
    ('•', '+'),
    ('○', 'o'),
    ('·', '.'),
    ('■', '#'),
    // The maze's way out, and the transcript's outcome markers. `»` is
    // deliberately not `>`: that is `►`, and they share a screen.
    ('Ω', 'U'),
    ('√', 'v'),
    ('→', '-'),
    ('≈', '~'),
    ('¿', '?'),
    ('»', '}'),
    // The fire's sparks, and the boot card's furniture.
    ('∙', ','),
    ('°', '^'),
    ('⌂', 'A'),
    ('⌐', '='),
    ('∟', 'L'),
    // Endless stock, and it was missing: `Stock::label` draws `∞` for every
    // inexhaustible pile, and it is Ambiguous, so it took two columns and
    // shifted the row. `8` for the shape; nothing else claims it.
    ('∞', '8'),
];

const fn narrowed(glyph: char) -> char {
    // One table, not two literals: as a match of 23 arms beside an array of 21,
    // `⌐` and `∟` were covered by no test at all.
    let mut index = 0;
    while index < SUBSTITUTIONS.len() {
        if SUBSTITUTIONS[index].0 == glyph {
            return SUBSTITUTIONS[index].1;
        }
        index += 1;
    }
    glyph
}

/// Every glyph the table covers — derived, not retyped.
///
/// `●` is deliberately absent: not in CP437, so `Cell::new` replaces it with `?`
/// before a frame reaches here (§19). A stand-in for a glyph that cannot occur
/// is a row nobody can reach.
#[cfg(test)]
fn substituted() -> Vec<char> {
    SUBSTITUTIONS.iter().map(|(from, _)| *from).collect()
}

/// Set the terminal's pen to `ink`.
///
/// `Attribute::Reset` first, because dim and bold are not opposites: SGR 22
/// clears both and there is no "not dim" that leaves bold alone.
fn set_pen(out: &mut impl Write, ink: Ink) -> std::io::Result<()> {
    queue!(out, SetAttribute(Attribute::Reset))?;
    match ink.weight {
        Weight::Dim => queue!(out, SetAttribute(Attribute::Dim))?,
        Weight::Plain => {}
        Weight::Bold => queue!(out, SetAttribute(Attribute::Bold))?,
    }
    queue!(out, SetForegroundColor(ink.colour.unwrap_or(Color::Reset)))
}

#[cfg(test)]
mod tests {
    use super::*;

    use orbs_render::{Span, Style};

    /// Draw a frame into a `Vec<u8>` and hand back what was written.
    fn drawn(screen: &mut Screen, frame: &Frame) -> String {
        let mut out = Vec::new();
        screen.draw(frame, &mut out).expect("a Vec cannot fail");
        String::from_utf8(out).expect("crossterm writes utf-8")
    }

    fn frame_saying(text: &str) -> Frame {
        let grid = GridSize::new(20, 3);
        let mut frame = Frame::new(grid);
        frame
            .painter(grid.to_rect())
            .span(Pos::new(0, 0), &Span::new(text).with_style(Style::NORMAL));
        frame
    }

    /// A lit run reaches the wire as a colour.
    ///
    /// End to end, because every link is somewhere else: `parser::lexeme`
    /// classifies, `sheet` registers the region, `Frame` stores it, `theme`
    /// picks the colour and this writes it. A break anywhere looks like a spell
    /// before any of it existed.
    #[test]
    fn a_lit_run_is_written_in_its_own_colour() {
        let grid = GridSize::new(20, 3);
        let mut frame = Frame::new(grid);
        frame.painter(grid.to_rect()).span(
            Pos::new(0, 0),
            &Span::new("repeat").with_style(Style::NORMAL),
        );
        frame.lit(
            orbs_render::Rect::new(0, 0, 6, 1),
            orbs_render::Lexeme::Control,
        );

        let mut screen = Screen::new(grid, true);
        let written = drawn(&mut screen, &frame);
        // `38;5;13`, not `35`: crossterm writes every named colour through the
        // 256-colour form, so a reader written against `3x` sees no colour at
        // all and reports the feature missing.
        assert!(
            written.contains("\x1b[38;5;13m"),
            "a control word reached the terminal with no colour on it:\n{written:?}",
        );
    }

    #[test]
    fn an_unchanged_frame_writes_no_cells() {
        // A second identical frame costs a cursor placement and nothing else —
        // over `ssh`, a game rather than a smear.
        let mut screen = Screen::new(GridSize::new(20, 3), true);
        let frame = frame_saying("clarity");

        let first = drawn(&mut screen, &frame);
        assert!(first.contains('c'), "the first paint drew nothing");

        let second = drawn(&mut screen, &frame);
        assert!(
            !second.contains('c'),
            "an unchanged frame redrew its cells: {second:?}",
        );
    }

    #[test]
    fn a_changed_tint_redraws_a_cell_whose_glyph_did_not_change() {
        // Tint is a region, so the flask's band changes colour under an
        // unchanged `█` and a `Cell` diff leaves the old colour on screen.
        use orbs_render::{Tint, Wash};

        let grid = GridSize::new(20, 3);
        let mut screen = Screen::new(grid, true);

        let mut plain = Frame::new(grid);
        plain
            .painter(grid.to_rect())
            .span(Pos::new(0, 0), &Span::new("███"));
        let _ = drawn(&mut screen, &plain);

        let mut tinted = Frame::new(grid);
        tinted
            .painter(grid.to_rect())
            .span(Pos::new(0, 0), &Span::new("███"));
        tinted.set_tint(grid.to_rect(), Wash::plain(Tint::Green));

        let second = drawn(&mut screen, &tinted);
        assert!(
            second.contains('█'),
            "the wash changed and nothing was redrawn: {second:?}",
        );
    }

    #[test]
    fn a_resize_wipes_what_the_new_frame_leaves_blank() {
        // Shrinking a *full* screen is the failure, and growing a sparse one
        // misses it: the buffer says "all blank", so cells the new frame leaves
        // blank compare equal and are skipped. Asserted on the wipe, because no
        // per-cell writing undoes a glyph nobody writes over.
        let wide = GridSize::new(40, 3);
        let mut screen = Screen::new(wide, true);

        let mut full = Frame::new(wide);
        full.painter(wide.to_rect())
            .fill(wide.to_rect(), '█', Style::NORMAL);
        let first = drawn(&mut screen, &full);
        assert!(!first.contains("\u{1b}[2J"), "the opening frame wiped");
        assert!(first.contains('█'), "the first paint drew nothing");

        // Now the same screen, narrower and nearly empty.
        let narrow = GridSize::new(20, 3);
        let mut sparse = Frame::new(narrow);
        sparse
            .painter(narrow.to_rect())
            .span(Pos::new(0, 0), &Span::new("clarity"));

        let after = drawn(&mut screen, &sparse);
        assert!(
            after.contains("\u{1b}[2J"),
            "a resize left the old screen underneath: {after:?}",
        );
    }

    #[test]
    fn a_resize_forgets_everything() {
        // Addressed by (col, row), so a carried buffer writes this frame's
        // cells at last frame's coordinates.
        let mut screen = Screen::new(GridSize::new(20, 3), true);
        let frame = frame_saying("clarity");
        let _ = drawn(&mut screen, &frame);

        let wider = GridSize::new(40, 3);
        let mut moved = Frame::new(wider);
        moved
            .painter(wider.to_rect())
            .span(Pos::new(0, 0), &Span::new("clarity"));

        let after = drawn(&mut screen, &moved);
        assert!(
            after.contains('c'),
            "a resize kept a stale shadow buffer: {after:?}",
        );
    }

    #[test]
    fn a_wide_terminal_gets_glyphs_it_can_draw_in_one_column() {
        // Or the table makes the problem it exists to solve.
        for glyph in substituted() {
            let swapped = narrowed(glyph);
            assert!(
                swapped.is_ascii_graphic(),
                "{glyph:?} was swapped for {swapped:?}, which is not plain ASCII",
            );
            assert_ne!(swapped, glyph, "{glyph:?} was left as itself");
        }
    }

    #[test]
    fn the_width_probe_is_a_glyph_this_table_substitutes() {
        // `term::PROBE` asks the question and this table is the only thing that
        // acts on the answer, so a probe glyph absent from here fires correctly
        // and changes nothing on screen.
        let probe = crate::term::PROBE;
        assert!(
            substituted().contains(&probe),
            "{probe:?} is what the terminal is measured with, but this table \
             leaves it alone — so a wide terminal would be detected and then \
             drawn to exactly as if it were narrow",
        );
    }

    #[test]
    fn every_substitution_is_its_own_character() {
        // §14. Two glyphs sharing a stand-in merges two meanings invisibly, the
        // row still lining up: `•` and `·` both went to `.`, making a *taken*
        // node identical to a *locked* one.
        let mut seen: Vec<char> = Vec::new();
        for glyph in substituted() {
            let swapped = narrowed(glyph);
            assert!(
                !seen.contains(&swapped),
                "{glyph:?} was swapped for {swapped:?}, which another glyph already uses",
            );
            seen.push(swapped);
        }
    }

    #[test]
    fn the_states_a_surface_is_made_of_stay_distinct() {
        // The rule above aimed at the two surfaces made *entirely* of these
        // glyphs. `loom.rs`: taken / open / locked. `board.rs`: held / loose /
        // aligned / astray.
        let weave = ['\u{2022}', '\u{25cb}', '\u{b7}'].map(narrowed);
        assert_eq!(
            weave.iter().collect::<std::collections::HashSet<_>>().len(),
            3,
            "two progression states became the same character: {weave:?}",
        );

        let ward = ['\u{25a0}', '\u{b7}', '\u{2022}', '\u{25cb}'].map(narrowed);
        assert_eq!(
            ward.iter().collect::<std::collections::HashSet<_>>().len(),
            4,
            "two ward states became the same character: {ward:?}",
        );
    }

    #[test]
    fn the_wards_six_sigils_stay_six() {
        // `board.rs`: *"six glyphs, never six colours"*. A terminal that
        // widens them still has to hand back six distinguishable things.
        let sigils = orbs_render::SIGILS.map(narrowed);
        assert_eq!(
            sigils
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            orbs_render::SIGILS.len(),
            "two sigils became the same character: {sigils:?}",
        );
    }

    /// Reconstruct the grid from what was written to the terminal.
    ///
    /// Follows `MoveTo` and `Print` and ignores every colour sequence.
    fn as_seen(written: &str, grid: GridSize) -> Vec<String> {
        let mut cells = vec![' '; grid.area()];
        let (mut col, mut row) = (0u16, 0u16);
        let mut rest = written;
        while let Some(start) = rest.find('\u{1b}') {
            for glyph in rest[..start].chars() {
                if let Some(slot) = cells.get_mut(row as usize * grid.cols as usize + col as usize)
                {
                    *slot = glyph;
                }
                col = col.saturating_add(1);
            }
            let after = &rest[start + 1..];
            let end = after
                .find(|c: char| c.is_ascii_alphabetic())
                .map_or(after.len(), |i| i + 1);
            let sequence = &after[..end];
            // `ESC [ <row> ; <col> H` — one-based, and the only one that moves us.
            if let Some(body) = sequence.strip_prefix('[').and_then(|s| s.strip_suffix('H')) {
                let mut parts = body.split(';');
                row = parts
                    .next()
                    .and_then(|n| n.parse::<u16>().ok())
                    .unwrap_or(1)
                    - 1;
                col = parts
                    .next()
                    .and_then(|n| n.parse::<u16>().ok())
                    .unwrap_or(1)
                    - 1;
            }
            rest = &after[end..];
        }
        for glyph in rest.chars() {
            if let Some(slot) = cells.get_mut(row as usize * grid.cols as usize + col as usize) {
                *slot = glyph;
            }
            col = col.saturating_add(1);
        }
        cells
            .chunks(grid.cols as usize)
            .map(|row| row.iter().collect())
            .collect()
    }

    #[test]
    fn the_terminal_shows_the_frame_and_nothing_else() {
        // Rule 2 as an assertion: every glyph the Frame holds reaches the
        // terminal at the Frame's own coordinates, and no other glyph appears.
        // Checked against `Frame::to_text`, so a rasteriser cannot pass by
        // agreeing only with itself.
        use orbs_shell::{Bench, Line, Linear, Offered, PaneTransition, Panel, Reveal, Screen};

        let grid = GridSize::new(120, 45);
        let mut sim = orbs_sim::Sim::new(3);
        for line in ["attend laboratory", "kindle charcoal", "grind sage"] {
            sim.submit(line);
            sim.step();
        }

        // The running loop's own constructor, so this cannot check a frame
        // drawn in a mode the terminal is never in.
        let screen = Screen::windowless(grid, None);
        let mut panel = Panel::default();
        panel.refresh(&sim);

        let mut frame = Frame::new(grid);
        let mut linear = Linear::default();
        orbs_shell::paint(
            &mut frame,
            &mut linear,
            &mut orbs_shell::Passing::default(),
            orbs_shell::View {
                sim: &sim,
                line: &Line::default(),
                screen: &screen,
                panes: &PaneTransition::settled(1),
                reveal: &Reveal::default(),
                offered: &Offered::default(),
                ghost: "",
                panel: &panel,
                scroll: &orbs_shell::Scroll::default(),
                bench: &Bench::default(),
                editing: None,
                weaving: None,
                walking: false,
                menuing: None,
                reading_manual: None,
            },
        );

        let mut out = Vec::new();
        // `narrow`, so no substitution runs: this is about placement.
        super::Screen::new(grid, true)
            .draw(&frame, &mut out)
            .expect("a Vec cannot fail");
        let written = String::from_utf8(out).expect("crossterm writes utf-8");

        let expected: Vec<String> = frame.to_text().lines().map(str::to_owned).collect();
        let seen = as_seen(&written, grid);
        assert_eq!(seen.len(), expected.len(), "wrong number of rows");
        for (row, (drawn, wanted)) in seen.iter().zip(&expected).enumerate() {
            assert_eq!(
                drawn.trim_end(),
                wanted.trim_end(),
                "row {row} reached the terminal differently from the Frame",
            );
        }
    }

    #[test]
    fn the_substitutions_are_drawable_by_the_bevy_build_too() {
        // A stand-in outside CP437 is a glyph the *other* frontend cannot draw,
        // which is the boundary running backwards.
        for glyph in ['‼', '►', '☼', '♀', '♦', 'Ω', '√', '→'] {
            assert!(orbs_render::is_renderable(narrowed(glyph)));
        }
    }
}
