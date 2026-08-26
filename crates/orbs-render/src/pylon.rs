//! A course of wards across three stations — what the pylon looks like.
//!
//! Drawn beside the transcript whenever a course is drawn up, which is the rule
//! the archive's map and the lens's sheet both follow: **the picture is not
//! gated on a word**, because watching a bound spell solve one and solving it
//! yourself are different activities and only the second involves typing.
//!
//! # It carries nothing the readings lack
//!
//! Rule 2's line. Every ward drawn here is one the player could count off
//! `survey wellspring` and the two `potency` readings, and the tally under the
//! rule is what `survey pylon` says. What the picture adds is that all three
//! stations are legible **at once**, which is the whole of what makes a Hanoi
//! position readable — the same thing the ward's sheet adds by holding five
//! presses side by side.
//!
//! # A ward's magnitude is its width, never its colour
//!
//! §14 forbids meaning that lives only in hue, and here the constraint bites
//! harder than usual: the puzzle's one rule is *a greater ward will not rest
//! upon a lesser*, so if magnitude were a colour the rule would be invisible in
//! greyscale and invisible in a dump. A ward `n` wide is `n` cells of [`WARD`],
//! so the stack is a staircase and an illegal position would be a staircase with
//! a step the wrong way up. The tint is one colour for every ward and is pure
//! enrichment.

use crate::style::{Style, Tint};

/// The glyph a ward is drawn from. CP437 0xDB.
pub const WARD: char = '\u{2588}';

/// The floor of each station. CP437 0xC4.
pub const GROUND: char = '\u{2500}';

/// Arcane, which is what a ward is made of.
pub const TINT: Tint = Tint::Violet;

/// The tallest course the pylon will draw up.
///
/// **A painter's constant, not a copy of the sim's.** `tower::pylon::MOST` is
/// the world's rule; this is how many rows the picture reserves, and they are
/// the same number for the good reason that a board which could not draw the
/// tallest course would be a board that stopped working on a neglected tower.
/// `a_board_has_room_for_the_tallest_course` holds the two together from the
/// sim's side, which is the only side that can see both.
pub const TALLEST: usize = 7;

/// The stations, and what is stacked at each.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pylon {
    /// Each station's wards, bottom first, as magnitudes.
    ///
    /// A magnitude is `1..=height`, and `1` is the least. Bottom-first because
    /// that is the order the rule cares about: a station is legal exactly when
    /// its magnitudes descend.
    pub stations: [Vec<usize>; 3],
    /// What the stations are called, left to right.
    ///
    /// **Handed in by the sim, not held here.** `haul wellspring conduit` names
    /// a station by word, so a board whose columns are unlabelled is one a
    /// player has to count along before they can type — the lens's sheet paid
    /// for this exact omission. They are content (`tower::pylon::STATIONS`),
    /// which rule 6 keeps out of a painter.
    pub names: [&'static str; 3],
    /// How many wards this course has.
    pub height: usize,
    /// Hauls spent so far.
    pub hauls: u32,
    /// The tower's standing.
    ///
    /// **The denominator is not here, and was.** A `standing: u32` sat beside
    /// this saying what full would be, written by every constructor and read by
    /// nothing — the board draws no bar, so it had nothing to measure against.
    /// The rail is where a proportion is drawn (`Unit::Standing`), and it takes
    /// its own from `tower::STANDING`.
    pub integrity: u32,
    /// The line under the ground rule, already written.
    ///
    /// **Handed in rather than composed here**, and it is rule 6 rather than
    /// tidiness: `4 wards, 7 hauled, 62` is a sentence, and a sentence built out
    /// of literals in a painter is prose that cannot be hot-reloaded, cannot be
    /// re-registered, and would be the only authored English in this crate. The
    /// three numbers above it are still here because a *test* wants them; what
    /// gets drawn is this.
    pub tally: String,
}

impl Pylon {
    /// One station's column, in cells.
    ///
    /// Eleven, because `wellspring` is ten and a column with no gap beside it
    /// runs into its neighbour. It also has to hold the tallest ward, which is
    /// [`TALLEST`] cells wide — so the name is what sets this, not the wards.
    const COLUMN: u16 = 11;

    /// The whole board's width.
    pub const COLS: u16 = Self::COLUMN * 3;

    /// The header naming the stations.
    const HEAD: u16 = 1;

    /// The floor rule, and the tally beneath it.
    const FOOT: u16 = 2;

    /// The rows a board wants, whatever is on it.
    ///
    /// **Fixed, and deliberately not sized to the course in hand.** A board that
    /// shrank with the stack would move the ground line every haul, and the
    /// ground line is the one thing a player reads position against. Seven rows
    /// of sky above a three-ward course is the cost, and it is the right one.
    #[must_use]
    pub const fn rows() -> u16 {
        // `TALLEST` is 7. The cast is a const-fn away from `try_from`, and the
        // constant is right here — `Board::rows_for` takes the same licence for
        // the same reason.
        #[allow(clippy::cast_possible_truncation)]
        let tallest = TALLEST as u16;
        Self::HEAD + tallest + Self::FOOT
    }

    /// The board's size in character cells.
    #[must_use]
    pub const fn size() -> (u16, u16) {
        (Self::COLS, Self::rows())
    }

    // **There is no `viewport` here, and there was.** It answered *does the
    // bare board fit in this area*, and its only caller was the test that
    // tested it: `orbs_shell::pylon::split` asks a different question — a
    // bordered board, plus the gutter and the transcript floor beside it — and
    // gets the one thing the two share, [`size`](Self::size), from here. A
    // second fits-or-not that nothing consults is a rule waiting to disagree
    // with the one that ships. (`Board::viewport` in the lens is the sibling
    // this was copied from and is dead in the same way.)

    /// One row of the board, as glyphs and their styles.
    ///
    /// `None` past the end. Row 0 names the stations; rows `1..=TALLEST` are the
    /// stacks, drawn from the top of the picture down, so the last of them is
    /// the row resting on the floor.
    #[must_use]
    pub fn row(&self, index: usize) -> Option<Vec<(char, Style, Option<Tint>)>> {
        let index = u16::try_from(index).ok()?;
        if index >= Self::rows() {
            return None;
        }
        if index < Self::HEAD {
            return Some(self.header_row());
        }
        let body = index - Self::HEAD;
        if (body as usize) < TALLEST {
            return Some(self.ward_row(TALLEST - 1 - body as usize));
        }
        if body as usize == TALLEST {
            return Some(Self::ground_row());
        }
        Some(self.tally_row())
    }

    /// The board as plain text, for a dump and for tests.
    #[must_use]
    pub fn line(&self, index: usize) -> Option<String> {
        Some(self.row(index)?.into_iter().map(|(ch, ..)| ch).collect())
    }

    /// The station names, each centred over its column.
    fn header_row(&self) -> Vec<(char, Style, Option<Tint>)> {
        let mut row = Vec::with_capacity(Self::COLS as usize);
        for name in self.names {
            push(&mut row, &centred(name, Self::COLUMN), Style::NORMAL, None);
        }
        row
    }

    /// One storey of all three stacks.
    ///
    /// `level` counts from the floor: 0 is what a station is resting on.
    fn ward_row(&self, level: usize) -> Vec<(char, Style, Option<Tint>)> {
        let mut row = Vec::with_capacity(Self::COLS as usize);
        for station in &self.stations {
            match station.get(level) {
                Some(size) => {
                    let bar: String = std::iter::repeat_n(WARD, *size).collect();
                    push(
                        &mut row,
                        &centred(&bar, Self::COLUMN),
                        Style::NORMAL,
                        Some(TINT),
                    );
                }
                None => push(
                    &mut row,
                    &" ".repeat(Self::COLUMN as usize),
                    Style::DIM,
                    None,
                ),
            }
        }
        row
    }

    /// The floor the three stations rest on.
    fn ground_row() -> Vec<(char, Style, Option<Tint>)> {
        let mut row = Vec::with_capacity(Self::COLS as usize);
        for _ in 0..3 {
            let bar: String = std::iter::repeat_n(GROUND, Self::COLUMN as usize - 1).collect();
            push(&mut row, &format!("{bar} "), Style::DIM, None);
        }
        row
    }

    /// What the course has cost and what the tower is standing at.
    fn tally_row(&self) -> Vec<(char, Style, Option<Tint>)> {
        let mut row = Vec::with_capacity(Self::COLS as usize);
        push(
            &mut row,
            &centred(&self.tally, Self::COLS),
            Style::DIM,
            None,
        );
        row
    }
}

/// Append `text`'s characters, all in one style.
fn push(row: &mut Vec<(char, Style, Option<Tint>)>, text: &str, style: Style, tint: Option<Tint>) {
    for ch in text.chars() {
        row.push((ch, style, if ch == ' ' { None } else { tint }));
    }
}

/// `text` centred in `width` cells, padded with spaces and never longer.
fn centred(text: &str, width: u16) -> String {
    let width = width as usize;
    let len = text.chars().count();
    if len >= width {
        return text.chars().take(width).collect();
    }
    let left = (width - len) / 2;
    let right = width - len - left;
    format!("{}{text}{}", " ".repeat(left), " ".repeat(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board(stations: [Vec<usize>; 3]) -> Pylon {
        let height = stations.iter().map(Vec::len).sum();
        Pylon {
            stations,
            names: ["wellspring", "conduit", "barrier"],
            height,
            hauls: 0,
            integrity: 100,
            tally: "3 wards, 0 hauled".to_owned(),
        }
    }

    #[test]
    fn every_glyph_the_board_draws_is_in_the_code_page() {
        // Listed rather than derived, for the reason the rail's own lint gives:
        // a painter's glyphs are the constants on the other side of these names,
        // and adding one without adding it here is the failure this cannot
        // catch. `▪` drew as `?` on the lens's sheet and `▸` on the rail; both
        // were found this way and both are in §19.
        for glyph in [WARD, GROUND, '─', ' '] {
            assert!(
                crate::cp437::is_renderable(glyph),
                "{glyph:?} (U+{:04X}) is not in CP437 and will draw as `?`",
                glyph as u32,
            );
        }
    }

    #[test]
    fn every_row_is_exactly_as_wide_as_the_board_says() {
        // The property the whole layout rests on: a row that came out short
        // would leave the transcript's border painted over by whatever was
        // underneath, and a long one would run into it.
        let board = board([vec![3, 2, 1], vec![], vec![]]);
        for index in 0..Pylon::rows() as usize {
            let row = board.row(index).expect("a row inside the board");
            assert_eq!(
                row.len(),
                Pylon::COLS as usize,
                "row {index} is {} cells, not {}",
                row.len(),
                Pylon::COLS,
            );
        }
        assert!(board.row(Pylon::rows() as usize).is_none());
    }

    #[test]
    fn a_ward_is_as_wide_as_it_is_big_and_the_stack_is_a_staircase() {
        // §14's rule made concrete. Size is width, so the illegal position is
        // the visibly wrong-way-up step and needs no colour to be seen.
        let board = board([vec![3, 2, 1], vec![], vec![]]);
        let ground = board
            .line(Pylon::rows() as usize - 3)
            .expect("the ground storey");
        let middle = board
            .line(Pylon::rows() as usize - 4)
            .expect("the middle storey");
        let top = board
            .line(Pylon::rows() as usize - 5)
            .expect("the top storey");

        assert_eq!(ground.matches(WARD).count(), 3);
        assert_eq!(middle.matches(WARD).count(), 2);
        assert_eq!(top.matches(WARD).count(), 1);
    }

    #[test]
    fn the_stack_is_drawn_from_the_ground_up() {
        // The bug this exists for: a picture drawn top-down puts a three-ward
        // course floating in the sky above its own ground line, which reads as
        // two wards missing rather than as a short stack.
        let board = board([vec![2, 1], vec![], vec![]]);
        let sky = board.line(1).expect("the top of the picture");
        assert!(
            !sky.contains(WARD),
            "a two-ward course reaches the top of a seven-row board: {sky:?}",
        );
        let lowest = board
            .line(Pylon::rows() as usize - 3)
            .expect("the ground storey");
        assert!(lowest.contains(WARD), "nothing is standing on the ground");
    }

    #[test]
    fn the_header_names_the_stations_a_haul_would_name() {
        // The lens's sheet shipped without this and a player had to count
        // columns before they could type. Same failure, caught by copying the
        // fix rather than the hole.
        let header = board([vec![1], vec![], vec![]]).line(0).expect("a header");
        for name in ["wellspring", "conduit", "barrier"] {
            assert!(header.contains(name), "{header:?} never names {name}");
        }
    }

    // The refuses-rather-than-truncates claim is `orbs_shell::pylon`'s to hold,
    // because `split` is the only thing that decides it — see
    // `a_pane_too_narrow_is_refused_whole_and_the_transcript_keeps_the_pane`
    // and `a_pane_too_short_is_refused_rather_than_clipped`.

    #[test]
    fn the_tallest_course_still_fits_between_the_names_and_the_floor() {
        // A ward is as wide as its magnitude, so the widest one has to fit a
        // column that was sized for a station's *name*. It does, with room to
        // spare — and if `TALLEST` ever rises past that this fails rather than
        // drawing a ward wider than its own station.
        let board = board([(1..=TALLEST).rev().collect(), vec![], vec![]]);
        for index in 0..Pylon::rows() as usize {
            assert_eq!(board.row(index).expect("a row").len(), Pylon::COLS as usize);
        }
        // The widest ward is the one on the ground, and a full course reaches
        // the top of the picture — so the two ends of the staircase pin both
        // that nothing is clipped and that nothing is drawn upside down.
        let widest = board
            .line(Pylon::rows() as usize - 3)
            .expect("the ground storey");
        assert_eq!(widest.matches(WARD).count(), TALLEST);
        let narrowest = board.line(1).expect("the top storey of a full course");
        assert_eq!(narrowest.matches(WARD).count(), 1);
    }
}
