//! A [`Frame`] onto a terminal — this frontend's whole half of rule 2.
//!
//! `orbs-render` said what appears and where; `orbs-shell` said what the screen
//! is. All that is left is writing the cells out, and the only decisions here
//! are how a glyph is encoded and what colour it takes.
//!
//! # Only what changed
//!
//! A full 120×45 repaint is 5,400 cells of escape sequences thirty times a
//! second. Over a local pty that is nothing; over `ssh` it is a visible smear.
//! So a shadow buffer holds what is on screen and only the differences are
//! written.
//!
//! **The diff key is `(glyph, ink)`, not `Cell`.** `Cell` derives `PartialEq`
//! and comparing it would look right and be wrong in a way only this game's
//! screens show: a **tint is a region**, so a cell can be byte-identical while
//! the wash over it changed. The flask's `green+bone` band growing is exactly
//! that — one `█` at a time, the same glyph, a different colour. Resolve first,
//! then compare.

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
    /// Whether the terminal must be wiped before the next draw.
    ///
    /// See [`Screen::resize`] — this is the half of a resize the buffer alone
    /// cannot express.
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
            // The alternate screen arrives blank, so the opening frame needs no
            // wipe — and paying for one would put a visible clear between
            // entering it and the first paint.
            wipe: false,
        }
    }

    /// Forget everything, because the terminal is a different size or has been
    /// scribbled on.
    ///
    /// **Every resize calls this.** The diff is addressed by `(col, row)`, so a
    /// buffer sized for the old grid would write this frame's cells at last
    /// frame's coordinates — which is not a smear but a scramble.
    ///
    /// # Resetting the buffer is only half of it
    ///
    /// The first version did just that, and it was wrong in the way a diff is
    /// always wrong: **it told the truth about the buffer and a lie about the
    /// screen.** Saying "every cell is blank" makes every cell the new frame
    /// leaves blank compare *equal*, so it is skipped — and the glyph the
    /// terminal is still showing there stays. Shrink a full screen and the
    /// result is a scramble of two layouts, because only the cells that happen
    /// to be non-blank get overwritten.
    ///
    /// So the terminal is wiped too, and then "everything is blank" is true of
    /// both. The wipe is queued into the same flush as the redraw, so there is
    /// no frame in between for anyone to see it empty.
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
        // **In the same flush as the cells that follow.** A wipe on its own
        // write would put one empty frame on screen, which is the flicker this
        // whole shadow buffer exists to avoid.
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
                let ink = theme::resolve(cell.style, frame.tint_at(Pos::new(col, row)));
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

        // **The real terminal cursor, from the Frame.** `orbs-render` keeps this
        // off `Cell` for exactly this: it is what makes the input line work with
        // the user's own line editing, and what a screen reader tracks.
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
/// Only the symbols the painters actually use are listed. Anything not named
/// here is left alone.
///
/// # The borders and the bars are left out, and not for the reason once given
///
/// This used to say box drawing and the block elements *"are Neutral or
/// Ambiguous-but-universally-narrow"*. The first half is false: measured,
/// `─ │ ═ ║ ╔ ╗ ╚ ╝` and `█ ▓ ▒` are **all** East Asian Ambiguous — only `░` is
/// Neutral — so under `ambiguous = wide` every border rule, every meter bar and
/// the whole boot logo take two columns while the 23 symbols below are swapped
/// to one.
///
/// They stay out anyway, on the *second* half of that claim, which is the one
/// that was doing the work: a terminal that widened box drawing would break
/// every full-screen program on the system, so in practice they are drawn
/// narrow even in that mode. That is an empirical bet rather than a property of
/// the code page, and it is worth stating as one — swapping `─` for `-` would
/// wreck the borders this exists to keep straight, and a half-substituted
/// screen is worse than an honestly-unhandled one.
///
/// **So the wide-ambiguous path is partial**, and `term::PROBE` firing means the
/// symbols are safe rather than that the screen is. A terminal that really does
/// widen box drawing is not supported, and that is now written down.
///
/// # Every substitution is distinct, and that is §14
///
/// **The first version of this table collapsed three meanings into `.`**, and it
/// is worth being specific about how bad that was. `loom.rs` draws a progression
/// node as `•` taken, `○` open, `·` locked; `board.rs` draws a ward socket as
/// `■` held, `·` loose, `•` aligned, `○` astray. Mapping `•` and `·` both to `.`
/// made *taken* indistinguishable from *locked*, and *aligned* from *loose* — on
/// the two surfaces whose entire content is those glyphs.
///
/// That is precisely the failure §14 forbids: **the glyph carries the identity**
/// here, because the colour cannot. A fallback that merges two glyphs is worse
/// than no fallback, because the row still lines up and nothing looks wrong.
///
/// So the map is **injective**, and `every_substitution_is_its_own_character`
/// holds it that way — which is the test that was missing when this was written.
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
    // **The three states, kept three.** Filled, hollow, faint — a taken node
    // from an open one from a locked one, and an aligned peg from an astray one
    // from a loose socket.
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
    // **Endless stock, and it was missing.** `Stock::label` draws `∞` for every
    // inexhaustible pile, so `survey dispensary` prints three of them on an
    // ordinary screen — and it is Ambiguous, so on the very terminal the probe
    // exists to detect it takes two columns and shifts the row. `8` for the
    // shape, and nothing else claims it.
    ('∞', '8'),
];

const fn narrowed(glyph: char) -> char {
    // **The table and the test's copy of it are one thing now.** They were two
    // literals a human kept aligned, and they had already drifted: the match had
    // 23 arms and the array 21, so `⌐` and `∟` were covered by no test at all.
    // That is precisely how the first version shipped `•` and `·` both mapping
    // to `.`, collapsing *taken* into *locked* and *aligned* into *loose* — the
    // defect the injectivity test exists to prevent, on arms the test could not
    // see.
    let mut index = 0;
    while index < SUBSTITUTIONS.len() {
        if SUBSTITUTIONS[index].0 == glyph {
            return SUBSTITUTIONS[index].1;
        }
        index += 1;
    }
    glyph
}

/// Every glyph the substitution table covers, for the tests below.
///
/// `●` is deliberately absent: it is **not in CP437**, so `Cell::new` replaces it
/// with `?` before a frame ever reaches here (§19 records it being rejected for
/// the progression tree). A stand-in for a glyph that cannot occur would be a
/// row in a table nobody can reach.
/// Every glyph the table covers — derived, not retyped.
#[cfg(test)]
fn substituted() -> Vec<char> {
    SUBSTITUTIONS.iter().map(|(from, _)| *from).collect()
}

/// Set the terminal's pen to `ink`.
///
/// `Attribute::Reset` first, because dim and bold are not opposites — SGR 22
/// clears both and there is no "not dim" that leaves bold alone. Resetting is
/// one sequence and cannot leave a cell wearing the previous run's weight.
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

    #[test]
    fn an_unchanged_frame_writes_no_cells() {
        // The whole point of the shadow buffer. A second identical frame should
        // cost a cursor placement and nothing else — over `ssh` this is the
        // difference between a readable game and a smear.
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
        // **The trap this file exists to avoid.** Tint is a region, so the
        // flask's mixture band changes colour under an unchanged `█`. A diff
        // keyed on `Cell` would call that no change and leave the old colour on
        // screen for ever.
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
        // **The case the first test of this missed.** It grew a nearly-empty
        // screen, where every cell that mattered was non-blank and therefore
        // redrawn anyway. Shrinking a *full* one is the failure: the buffer says
        // "all blank", so every cell the new frame leaves blank compares equal
        // and is skipped — while the terminal is still showing the old glyph.
        //
        // Asserted on the wipe rather than on the cells, because that is the
        // half the buffer cannot express: no amount of per-cell writing can undo
        // a glyph nobody writes over.
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
        // The buffer is addressed by (col, row). Carrying it across a resize
        // would write this frame's cells at last frame's coordinates.
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
        // Every substitution is one column, or the table is making the problem
        // it exists to solve.
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
        // **The two halves of the fallback, tied together.** `term::PROBE` asks
        // the terminal a question and this table is the only thing that acts on
        // the answer — so a probe glyph absent from here would fire correctly
        // and change nothing on screen, which is the same dead path as a probe
        // that cannot fire at all, arriving from the other side.
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
        // **§14, and the test that was missing when this table was written.**
        // Two glyphs sharing a stand-in merges two meanings, and does it
        // invisibly: the row still lines up and nothing looks wrong. The first
        // version sent `•` and `·` both to `.`, which made a *taken*
        // progression node identical to a *locked* one and an *aligned* ward peg
        // identical to a *loose* socket.
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
        // The general rule above, aimed at the two surfaces that are *entirely*
        // these glyphs — so a future edit to the table is checked against what
        // the glyphs actually mean rather than only against each other.
        //
        // `loom.rs`: taken / open / locked. `board.rs`: held / loose / aligned
        // / astray.
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
        // §14 again, and `board.rs` says it outright: *"six glyphs, never six
        // colours"*. A terminal that gives them two columns each still has to
        // hand back six things a player can tell apart.
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
    /// Follows `MoveTo` and `Print` and ignores every colour sequence, which is
    /// exactly the half of the output this is asserting about.
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
        // **The boundary claim for this frontend, as an assertion.** Rule 2 says
        // a frontend decides only how a cell is drawn — so every glyph the Frame
        // holds must reach the terminal at the Frame's own coordinates, and no
        // glyph the Frame does not hold may appear.
        //
        // It is checked against `Frame::to_text`, which is `orbs-render`'s own
        // reading of the same buffer. A rasteriser that agreed with itself and
        // not with that would be the whole failure this exists to catch.
        use orbs_shell::{Bench, Line, Linear, Offered, PaneTransition, Panel, Reveal, Screen};

        let grid = GridSize::new(120, 45);
        let mut sim = orbs_sim::Sim::new(3);
        for line in ["attend laboratory", "kindle charcoal", "grind sage"] {
            sim.submit(line);
            sim.step();
        }

        // **The same constructor the running loop uses**, so this test cannot
        // be checking a frame drawn in a mode the terminal is never in. The
        // three open-coded copies of this disagreed on exactly that field.
        let screen = Screen::windowless(grid, None);
        let mut panel = Panel::default();
        panel.refresh(&sim);

        let mut frame = Frame::new(grid);
        let mut linear = Linear::default();
        orbs_shell::paint(
            &mut frame,
            &mut linear,
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
            },
        );

        let mut out = Vec::new();
        // `narrow`, so no substitution runs: this test is about placement, and
        // the swap table has a test of its own.
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
        // A stand-in outside the CP437 repertoire would be a glyph the *other*
        // frontend cannot draw, which is the boundary running backwards.
        for glyph in ['‼', '►', '☼', '♀', '♦', 'Ω', '√', '→'] {
            assert!(orbs_render::is_renderable(narrowed(glyph)));
        }
    }
}
