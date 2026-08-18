//! The ward as a sheet of paper — what a code-breaker keeps beside them.
//!
//! Every press, with what it answered, and the aperture as it stands. Drawn
//! beside the transcript whenever a reading is open, which is what makes a bound
//! solver watchable — the same rule the archive's map follows, and for the same
//! reason: **the picture is not gated on a word**, because watching and doing are
//! different activities.
//!
//! # It carries nothing the readings lack
//!
//! Rule 2's line, and the maze's fog is the precedent: every row here is one
//! press's answer, which `probe` already said in the transcript and `survey
//! prism` will say again. A player with squared paper could keep this themselves,
//! which is the test §19 applies to the map — *"a player with squared paper could
//! have drawn it themselves"*.
//!
//! What it adds is **history**, and only history. Five presses are five lines the
//! transcript has already scrolled past; a sheet holds them side by side, which
//! is the whole of what makes deduction possible without a notepad. It infers
//! nothing, because the sim it reads from infers nothing (`tower::ward`).
//!
//! # Glyphs, never colour alone
//!
//! §14 forbids meaning that lives only in hue. Each sigil is a distinct CP437
//! glyph and the tint is enrichment on top, so the board survives greyscale and
//! survives a dump — which is where it will mostly be looked at.

use crate::geometry::Rect;
use crate::style::{Style, Tint};

/// One press, and what came back.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Attempt {
    /// The figure sent, as indices into the sigil table.
    pub figure: [usize; 4],
    /// Right sigil, right socket.
    pub aligned: u32,
    /// Right sigil, wrong socket.
    pub astray: u32,
}

/// The ward as a frontend needs to draw it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Board {
    /// Every press so far, oldest first.
    pub attempts: Vec<Attempt>,
    /// What the next press will send.
    pub aperture: [usize; 4],
    /// Which sockets are held, and refuse a dial.
    pub settled: [bool; 4],
    /// Presses each sigil has taken part in.
    pub marks: [u32; 6],
    /// What the sockets are called, left to right.
    ///
    /// **Handed in by the sim, not held here.** `dial second borax` names a socket
    /// by word, so a sheet whose columns are unlabelled is one a player has to
    /// count along before they can type — and the words are content
    /// (`tower::ward::SOCKETS`), which rule 6 keeps out of a painter.
    pub sockets: [&'static str; 4],
    /// What the sigils are called, in [`SIGILS`] order.
    ///
    /// The legend's half of the same problem, and the worse half: without it the
    /// board shows `♦` and the player has no way to learn that is `pewter` except
    /// by reading it off the transcript.
    pub sigils: [&'static str; 6],
}

/// A sigil's glyph.
///
/// **Six distinct shapes**, so the board reads in greyscale and in a dump. Each
/// is verified against [`cp437::is_renderable`](crate::cp437::is_renderable); a
/// seventh sigil would need a seventh glyph and the compiler will not say so,
/// which is what the test below is for.
pub const SIGILS: [char; 6] = ['☼', '○', '♂', '♀', '♦', '♠'];

/// A sigil's tint, in the same order.
///
/// Enrichment only — the glyph already carries the identity. A frontend that
/// resolves no tints draws a legible board.
pub const TINTS: [Tint; 6] = [
    Tint::Gold,
    Tint::Bone,
    Tint::Grey,
    Tint::Brown,
    Tint::Violet,
    Tint::Red,
];

/// A socket the ward has settled.
///
/// A settled socket refuses a dial (`tower::ward`), so the mark is not
/// decoration — it is the one thing on the board that changes what a player may
/// *do*.
///
/// `■` is CP437 0xFE. `▪` is the obvious choice and is **not** in the
/// repertoire — the same class of miss §19 records finding in DESIGN.md's own
/// boot text, caught here by the test below rather than by a player.
const HELD: char = '■';
/// A socket still free to turn.
const LOOSE: char = '·';

/// The pegs an answer is drawn with.
///
/// `•` is CP437 0x07 and `○` is 0x09 — the filled one for a sigil in its own
/// socket, the hollow one for a sigil in the wrong socket. `●` is **not** in
/// CP437 and was the obvious first choice.
const ALIGNED: char = '•';
const ASTRAY: char = '○';

/// Cells the left gutter takes: a press number, right-aligned, then a space.
///
/// `u16` throughout, like every other measurement in this crate — the row builders
/// take `usize::from` where they need to index, which is a widening and cannot
/// truncate. Declaring these `usize` and casting the total the other way is what
/// `cast_possible_truncation` correctly objects to.
const GUTTER: u16 = 3;

/// Cells one socket's column takes, name included.
///
/// `second` and `fourth` are the longest socket words at six, and one cell of gap
/// keeps two glyphs from reading as a pair.
const SOCKET: u16 = 7;

/// Cells one peg takes: the peg, then a space.
const PEG: u16 = 2;

impl Board {
    /// Columns the board wants.
    ///
    /// # It was ten, and ten was unreadable
    ///
    /// The first sheet packed four glyphs, two spaces and four pegs into ten
    /// cells — `☼○♂♀  ••○ ` — in a pane 104 wide. Correct, compact, and a wall of
    /// symbols: nothing said which column was which socket, so a player had to
    /// count along before they could type `dial second borax`, and nothing said
    /// which sigil `♦` was at all.
    ///
    /// Fixed at 39, because every row is the same shape — which is what lets a
    /// reader compare two presses by looking down a column, and is the whole
    /// reason a sheet beats scrollback.
    pub const COLS: u16 = GUTTER + SOCKET * 4 + PEG * 4;

    /// The most presses a sheet shows at once.
    ///
    /// **A cap, because a sheet with no cap disappears.** `Ward::history` grows
    /// once per press and a blind ladder averages 23 with a measured worst of 51 —
    /// past the pane's height the whole picture was refused, so it vanished with
    /// no explanation part-way through exactly the long solve it exists to make
    /// watchable.
    ///
    /// Twelve is what a deducing player ever needs to see: the worst hand-played
    /// solve over all 360 codes is six presses, so a person's whole reading fits
    /// twice over. Beyond that the older rows are the ladder's, and they are in
    /// `lens.log` — a sheet is a working surface, not an archive.
    pub const SHOWN: usize = 12;

    /// The presses the sheet draws: the most recent [`SHOWN`](Self::SHOWN).
    ///
    /// **The recent end, not the first.** What a deduction needs is what has
    /// happened lately, and a sheet that showed the opening twelve presses of a
    /// fifty-press walk would be a sheet frozen at the beginning.
    #[must_use]
    pub fn showing(&self) -> &[Attempt] {
        let from = self.attempts.len().saturating_sub(Self::SHOWN);
        self.attempts.get(from..).unwrap_or(&[])
    }

    /// Rows above the presses: the socket-name header.
    const HEAD: u16 = 1;

    /// Rows below the presses: a rule, the aperture, a gap, and two legend rows.
    const FOOT: u16 = 5;

    /// Rows it wants: the header, a row per press shown, and the foot.
    #[must_use]
    pub fn rows(&self) -> u16 {
        let shown = u16::try_from(self.showing().len()).unwrap_or(u16::MAX);
        shown.saturating_add(Self::HEAD).saturating_add(Self::FOOT)
    }

    /// The board's size in character cells.
    #[must_use]
    pub fn size(&self) -> (u16, u16) {
        (Self::COLS, self.rows())
    }

    /// One row of the sheet, as glyphs and their styles.
    ///
    /// `None` past the end. Row `attempts.len()` is the rule, and the one after
    /// it is the aperture — so the sheet reads top to bottom as *what I have
    /// tried*, then *what I will try next*.
    #[must_use]
    pub fn row(&self, index: usize) -> Option<Vec<(char, Style, Option<Tint>)>> {
        let shown = self.showing();
        let count = shown.len();
        let index = u16::try_from(index).ok()?;

        if index < Self::HEAD {
            return Some(self.header_row());
        }
        let body = index - Self::HEAD;
        if usize::from(body) < count {
            // **Numbered from the whole history, not from the sheet.** A capped
            // sheet shows the last twelve of fifty-one presses, and numbering those
            // `1..12` would say the solve had just begun.
            let first = self.attempts.len().saturating_sub(count);
            let at = usize::from(body);
            return Some(Self::attempt_row(shown.get(at)?, first + at + 1));
        }

        match usize::from(body) - count {
            0 => Some(Self::rule_row()),
            1 => Some(self.aperture_row()),
            2 => Some(blank()),
            3 => Some(self.legend_row(0)),
            4 => Some(self.legend_row(3)),
            _ => None,
        }
    }

    /// The socket names, over the columns they label.
    fn header_row(&self) -> Vec<(char, Style, Option<Tint>)> {
        let mut row = text(&" ".repeat(usize::from(GUTTER)), Style::DIM);
        for name in self.sockets {
            row.extend(text(&centred(name, SOCKET), Style::DIM));
        }
        row.extend(text(&centred("answer", PEG * 4), Style::DIM));
        row
    }

    fn rule_row() -> Vec<(char, Style, Option<Tint>)> {
        std::iter::repeat_n(('─', Style::DIM, None), usize::from(Self::COLS)).collect()
    }

    fn attempt_row(attempt: &Attempt, number: usize) -> Vec<(char, Style, Option<Tint>)> {
        let mut row = text(&format!("{number:>2} "), Style::DIM);
        for sigil in attempt.figure {
            row.extend(sigil_cell(sigil));
        }

        // **Pegs, not numbers**, and the count is the length of a run rather
        // than a digit — which is what makes two rows comparable at a glance
        // instead of read one at a time.
        for slot in 0..4u32 {
            let (glyph, style) = if slot < attempt.aligned {
                (ALIGNED, Style::SUCCESS)
            } else if slot < attempt.aligned + attempt.astray {
                (ASTRAY, Style::NORMAL)
            } else {
                (' ', Style::DIM)
            };
            row.push((glyph, style, None));
            row.push((' ', Style::DIM, None));
        }
        row
    }

    fn aperture_row(&self) -> Vec<(char, Style, Option<Tint>)> {
        // `→` is CP437 0x1A. It marks the row that has not been pressed yet, which
        // is the one distinction the sheet's shape cannot make on its own now that
        // the rows above it are numbered.
        let mut row = text(" → ", Style::NORMAL);
        for sigil in self.aperture {
            row.extend(sigil_cell(sigil));
        }

        // The settle marks sit under the pegs, in the same four columns, so a
        // held socket lines up with the peg that held it.
        for socket in 0..4usize {
            let held = self.settled.get(socket).copied().unwrap_or(false);
            row.push(if held {
                (HELD, Style::SUCCESS, None)
            } else {
                (LOOSE, Style::DIM, None)
            });
            row.push((' ', Style::DIM, None));
        }
        row
    }

    /// Three sigils and their names, so a glyph can be typed.
    fn legend_row(&self, from: usize) -> Vec<(char, Style, Option<Tint>)> {
        let mut row = text(" ", Style::DIM);
        for at in from..(from + 3) {
            let glyph = SIGILS.get(at).copied().unwrap_or('?');
            let name = self.sigils.get(at).copied().unwrap_or("");
            row.push((glyph, Style::NORMAL, TINTS.get(at).copied()));
            row.extend(text(&format!(" {name:<8}"), Style::DIM));
        }
        row.truncate(usize::from(Self::COLS));
        row.resize(usize::from(Self::COLS), (' ', Style::DIM, None));
        row
    }

    /// Every glyph of a row, for a test or a dump that wants the line.
    #[must_use]
    pub fn line(&self, index: usize) -> Option<String> {
        Some(
            self.row(index)?
                .into_iter()
                .map(|(glyph, _, _)| glyph)
                .collect(),
        )
    }

    /// Where the board goes inside `area`, or `None` if it will not fit.
    ///
    /// **Refuses rather than truncating**, which is the map's rule: a ward drawn
    /// short is not a smaller ward, it is a wrong one — a row missing its pegs
    /// says a press answered nothing.
    #[must_use]
    pub fn viewport(&self, area: Rect) -> Option<Rect> {
        let (cols, rows) = self.size();
        (area.cols >= cols && area.rows >= rows).then(|| Rect::new(area.col, area.row, cols, rows))
    }
}

/// A row of plain cells from a string.
fn text(from: &str, style: Style) -> Vec<(char, Style, Option<Tint>)> {
    from.chars().map(|glyph| (glyph, style, None)).collect()
}

/// A whole row of nothing, for the gap above the legend.
fn blank() -> Vec<(char, Style, Option<Tint>)> {
    std::iter::repeat_n((' ', Style::DIM, None), usize::from(Board::COLS)).collect()
}

/// One sigil, centred in its socket's column.
///
/// Centred rather than left-aligned so the glyph sits under the middle of the name
/// above it — which is what makes a column read as a column.
fn sigil_cell(sigil: usize) -> Vec<(char, Style, Option<Tint>)> {
    let glyph = SIGILS.get(sigil).copied().unwrap_or('?');
    let tint = TINTS.get(sigil).copied();
    let pad = usize::from((SOCKET - 1) / 2);
    let mut cell = text(&" ".repeat(pad), Style::DIM);
    cell.push((glyph, Style::NORMAL, tint));
    cell.extend(text(&" ".repeat(usize::from(SOCKET) - pad - 1), Style::DIM));
    cell
}

/// `text` centred in `width`, padded with spaces and cut if it will not fit.
fn centred(from: &str, width: u16) -> String {
    let width = usize::from(width);
    let count = from.chars().count();
    if count >= width {
        return from.chars().take(width).collect();
    }
    let left = (width - count) / 2;
    let mut out = " ".repeat(left);
    out.push_str(from);
    out.push_str(&" ".repeat(width - count - left));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Board {
        Board {
            attempts: vec![
                Attempt {
                    figure: [0, 1, 2, 3],
                    aligned: 1,
                    astray: 2,
                },
                Attempt {
                    figure: [0, 1, 3, 2],
                    aligned: 2,
                    astray: 1,
                },
            ],
            aperture: [0, 1, 3, 2],
            settled: [true, false, false, false],
            marks: [2, 2, 2, 2, 0, 0],
            sockets: NAMED_SOCKETS,
            sigils: NAMED_SIGILS,
        }
    }

    /// `tower::ward::SOCKETS`, which this crate may not depend on.
    const NAMED_SOCKETS: [&str; 4] = ["first", "second", "third", "fourth"];
    /// `tower::ward::SIGILS`, in [`SIGILS`] order.
    const NAMED_SIGILS: [&str; 6] = ["nitre", "alum", "borax", "quartz", "pewter", "ochre"];

    #[test]
    fn every_sigil_glyph_is_one_cp437_can_draw() {
        // The board is mostly looked at in a dump, and a glyph the repertoire
        // lacks is the failure §19 records finding in DESIGN.md's own boot text.
        for glyph in SIGILS {
            assert!(
                crate::cp437::is_renderable(glyph),
                "{glyph:?} is not in the repertoire",
            );
        }
        for glyph in [HELD, LOOSE, ALIGNED, ASTRAY] {
            assert!(
                crate::cp437::is_renderable(glyph),
                "{glyph:?} is not drawable",
            );
        }
    }

    #[test]
    fn the_six_sigils_are_six_different_shapes() {
        // §14: meaning may never live in hue alone. If two sigils shared a
        // glyph the board would be unreadable in greyscale and in every dump,
        // and the tints would be carrying the whole identity.
        let mut seen = SIGILS.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), SIGILS.len(), "two sigils draw the same");
    }

    #[test]
    fn every_row_is_exactly_as_wide_as_the_board_says() {
        // **The property the whole sheet rests on.** Rows are compared by looking
        // down a column, so one row a cell short shifts every peg beneath it and
        // silently makes two presses look alike. A `centred` that rounded the other
        // way, or a legend name a character longer, breaks exactly this.
        let board = board();
        for index in 0..usize::from(board.rows()) {
            let row = board
                .row(index)
                .expect("a row inside the board's own count");
            assert_eq!(
                row.len(),
                usize::from(Board::COLS),
                "row {index} is {} cells, not {}: {:?}",
                row.len(),
                Board::COLS,
                board.line(index),
            );
        }
        assert!(
            board.row(usize::from(board.rows())).is_none(),
            "the sheet ran past its own end",
        );
    }

    #[test]
    fn the_header_names_the_sockets_a_dial_would_name() {
        // Without this the player counts columns to work out that the third glyph
        // is `third` — and `dial` takes the word, not the number.
        let board = board();
        let head = board.line(0).expect("a header row");
        for socket in board.sockets {
            assert!(
                head.contains(socket),
                "{socket:?} is not on the sheet: {head:?}"
            );
        }
    }

    #[test]
    fn a_press_row_is_numbered_and_reads_as_a_figure_then_its_pegs() {
        let board = board();
        let first = board.line(1).expect("the first press has a row");
        assert_eq!(
            first, " 1    ☼      ○      ♂      ♀   • ○ ○   ",
            "{first:?}"
        );
    }

    #[test]
    fn the_aperture_follows_a_rule_and_carries_the_settle_marks() {
        let board = board();
        let rule = board
            .line(3)
            .expect("a rule between tries and the aperture");
        assert!(rule.chars().all(|glyph| glyph == '─'), "{rule:?}");

        let row = board.line(4).expect("the aperture has a row");
        assert_eq!(row, " →    ☼      ○      ♀      ♂   ■ · · · ", "{row:?}");
    }

    #[test]
    fn the_legend_names_every_sigil_beside_its_glyph() {
        // **The half a bare board could not teach at all.** `♦` is `pewter` and
        // nothing on screen said so, so a player could read the sheet perfectly and
        // still not know what to type.
        let board = board();
        let legend = format!(
            "{}{}",
            board.line(6).expect("a legend row"),
            board.line(7).expect("a second legend row"),
        );
        for (at, name) in board.sigils.iter().enumerate() {
            assert!(legend.contains(name), "{name:?} is not in the legend");
            assert!(
                legend.contains(SIGILS[at]),
                "{:?} is not in the legend",
                SIGILS[at],
            );
        }
    }

    #[test]
    fn a_board_refuses_an_area_it_would_have_to_be_cut_to_fit() {
        // The map's rule: a picture drawn short is a wrong picture, not a small
        // one. Better to draw nothing and let the transcript have the columns.
        let board = board();
        let (cols, rows) = board.size();
        assert!(board.viewport(Rect::new(0, 0, cols, rows)).is_some());
        assert!(board.viewport(Rect::new(0, 0, cols - 1, rows)).is_none());
        assert!(board.viewport(Rect::new(0, 0, cols, rows - 1)).is_none());
    }

    #[test]
    fn a_long_walk_still_has_a_sheet() {
        // **The failure this cap exists for.** `split` refuses whole rather than
        // clipping, so an uncapped sheet vanished with no explanation once the
        // presses outgrew the pane — part-way through the long solve the picture
        // is *for*. A blind ladder averages 23 presses and its measured worst is
        // 51.
        let mut board = board();
        board.attempts = (0..51)
            .map(|n| Attempt {
                figure: [0, 1, 2, 3],
                aligned: n % 4,
                astray: 0,
            })
            .collect();

        assert_eq!(board.showing().len(), Board::SHOWN);
        let shown = u16::try_from(Board::SHOWN).expect("twelve rows fit a u16");
        assert_eq!(board.rows(), shown + Board::HEAD + Board::FOOT);
        assert!(
            board.viewport(Rect::new(0, 0, 60, 40)).is_some(),
            "a fifty-press walk lost its sheet",
        );

        // ...and it is the *recent* end. A sheet frozen at the opening twelve
        // presses of a fifty-press walk would be showing the wrong twelve.
        let last = board
            .line(usize::from(Board::HEAD) + Board::SHOWN - 1)
            .expect("the newest press");
        assert!(
            last.contains("• •"),
            "the last row is not the newest press: {last:?}",
        );
        // **Numbered from the history, not from the sheet.** Numbering the last
        // twelve of fifty-one `1..12` would say the solve had just begun.
        assert!(
            last.starts_with("51 "),
            "the sheet renumbered a capped history: {last:?}",
        );
    }

    #[test]
    fn a_board_with_no_presses_is_still_a_sheet() {
        // Reachable the tick a reading opens, before the press lands.
        let fresh = Board {
            aperture: [0, 1, 2, 3],
            sockets: NAMED_SOCKETS,
            sigils: NAMED_SIGILS,
            ..Board::default()
        };
        assert_eq!(
            fresh.rows(),
            Board::HEAD + Board::FOOT,
            "a header, a rule, the aperture and the legend",
        );
        for index in 0..usize::from(fresh.rows()) {
            assert!(fresh.row(index).is_some(), "row {index} is missing");
        }
        assert!(fresh.row(usize::from(fresh.rows())).is_none());
    }
}
