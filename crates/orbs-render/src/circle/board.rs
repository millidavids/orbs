//! The board itself: what a beast at the circle is handed as, and the rows it
//! draws.

use crate::style::{Style, Tint};

/// A lit cell. The lattice's sun, because it means the same thing.
pub const LIT: char = '☼';

/// A dark one.
pub const DARK: char = '·';

/// Under a row the circle answered differently from the temper. CP437 0x1E.
pub const BALKS: char = '▲';

/// Between a glyph's humour and what it is given. CP437 0x1B.
pub const GIVEN: char = '←';

/// The rule between the wiring and the table. CP437 0xC4.
pub const RULE: char = '─';

/// The one hue on the board — the menagerie's, as the troop's is.
pub const TINT: Tint = Tint::Violet;

/// Before a sense a glyph is given turned over. ASCII, so CP437 0x7E.
pub const TURNED: char = '~';

/// One thing a glyph is given: a sense, or another glyph, and whether its wire
/// is turned.
///
/// One struct, not two lists side by side, so a name and its turn cannot come
/// apart when a line is built.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Given {
    /// What it is called: a sense's name, or a glyph's.
    pub name: String,
    /// Whether the wire hands it over turned — lit when the sense is dark, dark
    /// when it is lit.
    pub turned: bool,
}

/// One glyph of the circle, as a line of the board.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Line {
    /// The glyph's name: the word `limn` takes.
    pub glyph: String,
    /// The humour it is limned with.
    pub humour: String,
    /// What it is given — two senses, or the two other glyphs.
    pub given: Vec<Given>,
}

/// A beast at the circle, as the board needs it.
///
/// Every word is handed in, because this crate may not depend on `orbs-sim` and
/// the words are content: the senses' names are prose, and the glyphs and
/// humours are the parser's.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Circle {
    /// The glyphs, in the order they are drawn: the two outer ones, then the
    /// keystone they feed.
    pub lines: Vec<Line>,
    /// The senses' names, in the table's order.
    pub senses: Vec<String>,
    /// Which senses are lit on each row, `senses[s][row]`.
    pub lit: Vec<Vec<bool>>,
    /// The temper — what every row must answer.
    pub temper: Vec<bool>,
    /// What the circle answered when last called, or `None` before the first
    /// call.
    pub answer: Option<Vec<bool>>,
    /// The labels of the temper's row and the answer's, already written.
    pub labels: [String; 2],
    /// The line at the foot, already written.
    ///
    /// Handed in rather than composed here — rule 6: *"2 calls, 2 rows balk"* is
    /// a sentence, and one built from literals in a painter is prose nobody can
    /// reload.
    pub tally: String,
}

/// A board row's cells: the glyph, its style, and an optional tint.
pub type Cells = Vec<(char, Style, Option<Tint>)>;

impl Circle {
    /// The width every label takes, `widdershins` and a space.
    pub(super) const LABEL: usize = 12;

    /// The board's width, whatever is on it.
    ///
    /// The longest line is the keystone's — `keystone    oppose ← sunwise,
    /// widdershins` is 41 — and three spare columns keep a long sense name from
    /// running into the border.
    pub const COLS: u16 = 44;

    /// The whole circle's three lines of wiring, the rule, the numbers, three
    /// senses, the temper, the answer, the marks under it, and the tally — the
    /// most a board is asked to hold.
    const HEIGHT: u16 = 12;

    /// The board's size in character cells, border excluded.
    ///
    /// Fixed, never sized to what is standing: a board growing a row the first
    /// time the beast was called would move the transcript under the player's
    /// eye as they read the answer. The same for a lesser circle, which draws
    /// one line of wiring and two senses and leaves the foot blank, so the
    /// transcript does not move the day the whole circle opens either.
    #[must_use]
    pub const fn size() -> (u16, u16) {
        (Self::COLS, Self::HEIGHT)
    }

    /// One row of the board, as cells. Blank past what the circle holds and
    /// inside the footprint; `None` past the footprint.
    ///
    /// Laid out from what it is handed — as many wiring lines as
    /// [`lines`](Self::lines), as many sense rows as [`senses`](Self::senses) —
    /// so the whole circle and the lesser one are one painter.
    #[must_use]
    pub fn row(&self, index: usize) -> Option<Cells> {
        if index >= usize::from(Self::HEIGHT) {
            return None;
        }
        let wiring = self.lines.len();
        let senses = self.senses.len();
        // Where the senses start: under the wiring, the rule and the numbers.
        let table = wiring + 2;
        let row = if index < wiring {
            self.wiring_row(index)
        } else if index == wiring {
            text(
                &RULE.to_string().repeat(usize::from(Self::COLS)),
                Style::DIM,
            )
        } else if index == wiring + 1 {
            self.numbers_row()
        } else if index < table + senses {
            let sense = index - table;
            let name = self.senses.get(sense).map_or("", String::as_str);
            let lit = self.lit.get(sense).map(Vec::as_slice);
            self.table_row(name, lit, Style::DIM, None)
        } else if index == table + senses {
            self.table_row(&self.labels[0], Some(&self.temper), Style::NORMAL, None)
        } else if index == table + senses + 1 {
            self.table_row(
                &self.labels[1],
                self.answer.as_deref(),
                Style::NORMAL,
                Some(TINT),
            )
        } else if index == table + senses + 2 {
            self.marks_row()
        } else if index == table + senses + 3 {
            text(&self.tally, Style::DIM)
        } else {
            Vec::new()
        };
        Some(fitted(row))
    }

    /// How many columns the table has: one per way the senses can be lit.
    const fn columns(&self) -> usize {
        self.temper.len()
    }

    /// The board as plain text, for a dump and for tests.
    #[must_use]
    pub fn line(&self, index: usize) -> Option<String> {
        Some(
            self.row(index)?
                .into_iter()
                .map(|(glyph, ..)| glyph)
                .collect(),
        )
    }

    /// The rows the last call balked at, counting from one — the numbers the
    /// board draws over its columns, so a reader and a sighted player name the
    /// same row. Empty before the first call.
    #[must_use]
    pub fn balking(&self) -> Vec<usize> {
        let Some(answer) = self.answer.as_ref() else {
            return Vec::new();
        };
        (0..self.columns())
            .filter(|row| answer.get(*row) != self.temper.get(*row))
            .map(|row| row + 1)
            .collect()
    }

    /// One glyph's line: its name, its humour, and what it is given.
    fn wiring_row(&self, index: usize) -> Cells {
        let Some(line) = self.lines.get(index) else {
            return Vec::new();
        };
        let mut cells = Vec::new();
        push(&mut cells, &label(&line.glyph), Style::DIM, None);
        push(
            &mut cells,
            &format!("{:<7}", line.humour),
            Style::BRIGHT,
            Some(TINT),
        );
        // A turned wire is marked on its name, where the eye is when it reads
        // what a glyph is given — not in a legend the board has no row for.
        let given: Vec<String> = line
            .given
            .iter()
            .map(|given| {
                if given.turned {
                    format!("{TURNED}{}", given.name)
                } else {
                    given.name.clone()
                }
            })
            .collect();
        push(
            &mut cells,
            &format!("{GIVEN} {}", given.join(", ")),
            Style::DIM,
            None,
        );
        cells
    }

    /// The column numbers, one to the table's last, over the cells.
    fn numbers_row(&self) -> Cells {
        let mut cells = Vec::new();
        push(&mut cells, &" ".repeat(Self::LABEL), Style::DIM, None);
        for row in 1..=self.columns() {
            push(&mut cells, &format!("{row} "), Style::DIM, None);
        }
        cells
    }

    /// A label, then a cell for each row — blank when there is nothing to draw.
    fn table_row(
        &self,
        name: &str,
        lit: Option<&[bool]>,
        style: Style,
        tint: Option<Tint>,
    ) -> Cells {
        let mut cells = Vec::new();
        push(&mut cells, &label(name), Style::DIM, None);
        for row in 0..self.columns() {
            let glyph = match lit.and_then(|lit| lit.get(row)) {
                Some(true) => LIT,
                Some(false) => DARK,
                None => ' ',
            };
            push(&mut cells, &format!("{glyph} "), style, tint);
        }
        cells
    }

    /// A mark under each row the last call balked at.
    fn marks_row(&self) -> Cells {
        let balking = self.balking();
        let mut cells = Vec::new();
        push(&mut cells, &" ".repeat(Self::LABEL), Style::DIM, None);
        for row in 1..=self.columns() {
            let glyph = if balking.contains(&row) { BALKS } else { ' ' };
            push(&mut cells, &format!("{glyph} "), Style::DANGER, None);
        }
        cells
    }
}

/// A row's name, exactly [`Circle::LABEL`] cells wide: clipped to leave one
/// blank before the cells, and padded.
///
/// Clipped, not only padded, because the senses' names and the two labels are
/// prose a writer can reload: a name of twelve or more pushed its row's cells
/// right while the numbers and marks stayed put, putting a mark under the wrong
/// row — the one failure this board most has to avoid.
fn label(name: &str) -> String {
    let clipped: String = name.chars().take(Circle::LABEL - 1).collect();
    format!("{clipped:<width$}", width = Circle::LABEL)
}

/// A row of plain text in one style.
fn text(line: &str, style: Style) -> Cells {
    let mut cells = Vec::new();
    push(&mut cells, line, style, None);
    cells
}

/// Append `text`'s characters as cells.
fn push(cells: &mut Cells, text: &str, style: Style, tint: Option<Tint>) {
    cells.extend(text.chars().map(|glyph| (glyph, style, tint)));
}

/// Pad or clip a row to exactly [`Circle::COLS`].
fn fitted(mut cells: Cells) -> Cells {
    let width = usize::from(Circle::COLS);
    cells.truncate(width);
    while cells.len() < width {
        cells.push((' ', Style::DIM, None));
    }
    cells
}
