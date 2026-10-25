//! The forge's puzzle: a lattice of glyphs that must all be lit.
//!
//! §10's *sequence* for Enchanting: snapping a glyph flips it and its
//! orthogonal neighbours, and the charm sets when every glyph is alight.
//!
//! Lights Out, settled by the top row since every later press is forced — so
//! the domain addresses three columns, not nine cells. Measured over all 512
//! boards, and tested below: each has a unique solution, the eight openings
//! give eight distinct residues, and the residue-to-answer table is universal,
//! which is what lets a spell hold it.
//!
//! Height 5 is out by arithmetic: 3×5's null space is non-trivial, so not every
//! board solves. 3×4 and 3×6 would work, each with its own table.

/// How wide a lattice is.
pub const WIDTH: usize = 3;

/// How tall a lattice is.
///
/// Three for now. Four and six are the other heights that work — see the module
/// header — and each would need its own rung in a solver's table.
pub const HEIGHT: usize = 3;

/// How many glyphs a lattice holds.
pub const CELLS: usize = WIDTH * HEIGHT;

/// The words `snap` takes, left to right.
///
/// Three because the noun space has no more: nine cell names did not survive
/// the sweep (`crown` 800 against `brown`, `base` 750 against `bare`, `tier`
/// 600 against `tower`).
pub const COLUMNS: [&str; WIDTH] = ["apex", "belt", "hem"];

/// What is open on the forge's lattice: a charm, a tool, and the puzzle.
///
/// Saved as itself, `Siege`'s decision: a second shape would be a copy that can
/// disagree. The tool is a path, since an entity means nothing across a save.
#[derive(bevy_ecs::prelude::Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Binding {
    /// Which charm, by the word a player typed.
    pub kind: String,
    /// The tool it will be laid on, by path.
    pub tool: String,
    /// The puzzle.
    pub lattice: Lattice,
    /// What has been spent on settles so far, for the tally.
    #[serde(default)]
    pub spent: u32,
}

/// A glyph lattice, part-solved.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Lattice {
    /// The draw. Never mutated — a failed settle springs back to it, which makes
    /// each attempt an independent read of the same board rather than a walk
    /// that can be got lost in.
    opening: [bool; CELLS],
    /// Which columns are snapped for the attempt being built.
    pressed: [bool; WIDTH],
    /// Settles spent. What the escrow charges for.
    attempts: u32,
    /// Whether the last settle lit every glyph.
    set: bool,
}

impl Lattice {
    /// A lattice from the low [`CELLS`] bits of `bits`, lit where set.
    ///
    /// Bits of one draw, never one draw each, so widening the lattice moves no
    /// seed. No rejection loop: every board of this size solves.
    #[must_use]
    pub fn from_bits(bits: u64) -> Self {
        let mut opening = [false; CELLS];
        for (index, glyph) in opening.iter_mut().enumerate() {
            *glyph = (bits >> index) & 1 == 1;
        }
        Self {
            opening,
            pressed: [false; WIDTH],
            attempts: 0,
            set: false,
        }
    }

    /// Flip one glyph and its orthogonal neighbours.
    fn strike(board: &mut [bool; CELLS], row: usize, column: usize) {
        let mut flip = |row: usize, column: usize| {
            if row < HEIGHT && column < WIDTH {
                board[row * WIDTH + column] = !board[row * WIDTH + column];
            }
        };
        flip(row, column);
        flip(row + 1, column);
        flip(row, column + 1);
        if row > 0 {
            flip(row - 1, column);
        }
        if column > 0 {
            flip(row, column - 1);
        }
    }

    /// The board after `presses` on the top row and the forced cascade below it.
    ///
    /// The cascade presses under every dark glyph, top to bottom, settling each
    /// row as it leaves.
    #[must_use]
    fn cascade(opening: &[bool; CELLS], presses: &[bool; WIDTH]) -> [bool; CELLS] {
        let mut board = *opening;
        for (column, snapped) in presses.iter().enumerate() {
            if *snapped {
                Self::strike(&mut board, 0, column);
            }
        }
        for row in 1..HEIGHT {
            for column in 0..WIDTH {
                if !board[(row - 1) * WIDTH + column] {
                    Self::strike(&mut board, row, column);
                }
            }
        }
        board
    }

    /// What the lattice looks like right now — the draw, with what is snapped.
    ///
    /// Without the cascade, or the board would show an outcome not yet
    /// committed to.
    #[must_use]
    pub fn glyphs(&self) -> [bool; CELLS] {
        let mut board = self.opening;
        for (column, snapped) in self.pressed.iter().enumerate() {
            if *snapped {
                Self::strike(&mut board, 0, column);
            }
        }
        board
    }

    /// The bottom row a settle with **no** presses would leave.
    ///
    /// A property of the draw and the whole signal: eight openings, eight
    /// residues. What the columns publish and a solver's table is keyed on.
    #[must_use]
    pub fn residue(&self) -> [bool; WIDTH] {
        let settled = Self::cascade(&self.opening, &[false; WIDTH]);
        let mut out = [false; WIDTH];
        out.copy_from_slice(&settled[(HEIGHT - 1) * WIDTH..]);
        out
    }

    /// The one opening that lights this lattice — for the tests and nothing a
    /// player can reach, since working it out is the puzzle.
    #[must_use]
    pub fn solution(&self) -> [bool; WIDTH] {
        for choice in 0..(1u8 << WIDTH) {
            let mut presses = [false; WIDTH];
            for (column, press) in presses.iter_mut().enumerate() {
                *press = (choice >> column) & 1 == 1;
            }
            if Self::cascade(&self.opening, &presses)
                .iter()
                .all(|lit| *lit)
            {
                return presses;
            }
        }
        // Unreachable: every board of this size is solvable, and
        // `every_board_has_exactly_one_answer` holds it over all 512.
        [false; WIDTH]
    }

    /// Toggle a column's press. Snapping twice is snapping nothing.
    pub const fn snap(&mut self, column: usize) {
        if column < WIDTH {
            self.pressed[column] = !self.pressed[column];
        }
    }

    /// Whether `column` is snapped for the attempt being built.
    #[must_use]
    pub const fn snapped(&self, column: usize) -> bool {
        column < WIDTH && self.pressed[column]
    }

    /// Commit the attempt. `true` when every glyph came up lit.
    ///
    /// A failure springs back to the draw: a half-cascaded board cannot be
    /// reasoned about, and [`residue`](Self::residue) must mean the same thing
    /// on every attempt for one table to serve a loop.
    pub fn settle(&mut self) -> bool {
        self.attempts = self.attempts.saturating_add(1);
        self.set = Self::cascade(&self.opening, &self.pressed)
            .iter()
            .all(|lit| *lit);
        self.pressed = [false; WIDTH];
        self.set
    }

    /// Whether the charm has been bound.
    #[must_use]
    pub const fn set(&self) -> bool {
        self.set
    }

    /// How many settles have been spent.
    #[must_use]
    pub const fn attempts(&self) -> u32 {
        self.attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every board this domain can draw, for the exhaustive proofs.
    fn every_board() -> impl Iterator<Item = Lattice> {
        (0..(1u64 << CELLS)).map(Lattice::from_bits)
    }

    fn answers(lattice: &Lattice) -> Vec<[bool; WIDTH]> {
        (0..(1u8 << WIDTH))
            .map(|choice| {
                let mut presses = [false; WIDTH];
                for (column, press) in presses.iter_mut().enumerate() {
                    *press = (choice >> column) & 1 == 1;
                }
                presses
            })
            .filter(|presses| {
                Lattice::cascade(&lattice.opening, presses)
                    .iter()
                    .all(|lit| *lit)
            })
            .collect()
    }

    /// No draw can be a dead end, and none is ambiguous — which is why `draw`
    /// has no rejection loop. If it fails, the lattice has been resized to
    /// something the arithmetic does not support, as 3×5 is.
    #[test]
    fn every_board_has_exactly_one_answer() {
        for lattice in every_board() {
            let found = answers(&lattice);
            assert_eq!(
                found.len(),
                1,
                "a board with {} answers: {:?}",
                found.len(),
                lattice.opening,
            );
            assert_eq!(
                found[0],
                lattice.solution(),
                "solution disagrees with the sweep"
            );
        }
    }

    /// The residue identifies the answer, which is what makes it worth
    /// publishing. Eight openings, eight residues, no two alike.
    #[test]
    fn the_eight_openings_leave_eight_different_residues() {
        for lattice in every_board() {
            let mut seen = Vec::new();
            for choice in 0..(1u8 << WIDTH) {
                let mut presses = [false; WIDTH];
                for (column, press) in presses.iter_mut().enumerate() {
                    *press = (choice >> column) & 1 == 1;
                }
                let settled = Lattice::cascade(&lattice.opening, &presses);
                seen.push(settled[(HEIGHT - 1) * WIDTH..].to_vec());
            }
            seen.sort();
            let total = seen.len();
            seen.dedup();
            assert_eq!(total, seen.len(), "two openings left the same residue");
        }
    }

    /// One table serves every board — the property a solver's eight rungs rest
    /// on. If the map from residue to answer varied with the draw, no fixed
    /// ladder could hold it and the domain would be unscriptable.
    #[test]
    fn the_residue_names_the_same_answer_on_every_board() {
        let mut table: std::collections::BTreeMap<Vec<bool>, [bool; WIDTH]> =
            std::collections::BTreeMap::new();
        for lattice in every_board() {
            let residue = lattice.residue().to_vec();
            let answer = lattice.solution();
            if let Some(known) = table.get(&residue) {
                assert_eq!(
                    *known, answer,
                    "residue {residue:?} wants {answer:?} here and {known:?} elsewhere",
                );
            } else {
                table.insert(residue, answer);
            }
        }
        assert_eq!(table.len(), 1 << WIDTH, "the table is not total");
    }

    /// Snapping twice is snapping nothing, so a fumbled press costs no attempt.
    #[test]
    fn a_column_snapped_twice_is_a_column_left_alone() {
        let mut lattice = Lattice::from_bits(0b1_0101_0101);
        let before = lattice.glyphs();
        lattice.snap(0);
        assert_ne!(lattice.glyphs(), before, "the first snap did nothing");
        lattice.snap(0);
        assert_eq!(lattice.glyphs(), before, "the second snap did not undo it");
        assert!(!lattice.snapped(0));
    }

    /// A failed settle leaves the board exactly as it was drawn.
    #[test]
    fn a_failed_settle_springs_back_to_the_draw() {
        let mut lattice = Lattice::from_bits(0b0_1010_1010);
        let drawn = lattice.glyphs();
        let wrong = if lattice.solution()[0] { 1 } else { 0 };
        lattice.snap(wrong);
        assert!(!lattice.settle() || lattice.set());
        if !lattice.set() {
            assert_eq!(
                lattice.glyphs(),
                drawn,
                "a failed attempt left the board moved"
            );
            assert_eq!(lattice.attempts(), 1);
        }
    }

    /// Playing the answer lights it, from every draw.
    #[test]
    fn the_answer_lights_every_board_in_one_settle() {
        for mut lattice in every_board() {
            for (column, press) in lattice.solution().into_iter().enumerate() {
                if press {
                    lattice.snap(column);
                }
            }
            assert!(lattice.settle(), "the answer did not light the lattice");
            assert_eq!(lattice.attempts(), 1);
        }
    }

    /// The chase clears every row but the last, which is why columns are enough
    /// to address. If it were false, the residue would stop identifying the
    /// answer — dark glyphs above the bottom row that no opening accounts for,
    /// and every solver's table reading a signal that means nothing.
    #[test]
    fn the_cascade_leaves_only_the_last_row_in_doubt() {
        for lattice in every_board() {
            for choice in 0..(1u8 << WIDTH) {
                let mut presses = [false; WIDTH];
                for (column, press) in presses.iter_mut().enumerate() {
                    *press = (choice >> column) & 1 == 1;
                }
                let settled = Lattice::cascade(&lattice.opening, &presses);
                for row in 0..HEIGHT - 1 {
                    for column in 0..WIDTH {
                        assert!(
                            settled[row * WIDTH + column],
                            "the cascade left ({row}, {column}) dark on board {:?}",
                            lattice.opening,
                        );
                    }
                }
            }
        }
    }

    /// A lattice survives being written down and read back.
    ///
    /// The whole thing, not a derived shape: a `Binding` travels as the
    /// component, so the draw, the presses and the count of falls must all
    /// round-trip. A board that came back with its presses cleared would be a
    /// player's turn quietly undone by quitting.
    #[test]
    fn a_part_worked_lattice_reads_back_as_itself() {
        let mut lattice = Lattice::from_bits(0b1_0110_1001);
        lattice.snap(0);
        lattice.snap(2);
        let text = toml::to_string(&lattice).expect("a lattice writes");
        let read: Lattice = toml::from_str(&text).expect("a lattice reads back");
        assert_eq!(read, lattice, "a lattice did not survive the trip");
        assert!(read.snapped(0) && read.snapped(2) && !read.snapped(1));
    }

    /// A strike is its own inverse, which is what makes the toggle a group.
    #[test]
    fn striking_the_same_glyph_twice_changes_nothing() {
        for row in 0..HEIGHT {
            for column in 0..WIDTH {
                let mut board = [false; CELLS];
                Lattice::strike(&mut board, row, column);
                Lattice::strike(&mut board, row, column);
                assert!(
                    board.iter().all(|lit| !lit),
                    "a strike is not its own inverse"
                );
            }
        }
    }
}
