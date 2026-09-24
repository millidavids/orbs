//! The archive's stacks, as a picture (DESIGN.md §10, §19).
//!
//! `orbs-sim` depends on *this* crate, never the other way round, so the type a
//! frontend draws lives here and the sim builds one — the arrangement
//! `Instrument`'s [`Wash`](crate::Wash) already has. What crosses is a
//! *description*, so the sim's `Maze` keeps its cells private.
//!
//! One character per square. The maze was cells with the walls *between* them,
//! drawn `2w+1` across, so one step moved the reading two characters and read
//! as two. A wall is a square of its own now.
//!
//! The whole maze is drawn and the fog is gone (§19). The trade: a sighted
//! player sees more than the linear stream carries, the asymmetry §14 exists to
//! prevent. What survives it is that a spell still solves the maze from the
//! four `survey` readings alone.
//!
//! Unwalked floor draws as nothing. The walls around it are on screen, so a
//! corridor is a gap in them; a glyph there says what the wall already says,
//! which is what the `·` was and why it went.

use crate::geometry::Rect;
use crate::style::Style;

/// One square of the grid.
///
/// A square, not a cell with four walls: the old form drew `2w+1` across, so a
/// one-cell step moved the reading two characters.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Square {
    /// Whether this is solid.
    pub wall: bool,
    /// How many times the reading has entered. Trémaux's mark.
    pub marks: u8,
}

/// The stacks as a frontend needs to draw them.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Stacks {
    /// Squares, row-major, already fogged.
    pub squares: Vec<Square>,
    /// How wide, in squares.
    pub width: u16,
    /// Where the reading is, as an index into `squares`.
    pub at: usize,
    /// Where the way out is, when there is one.
    ///
    /// `Option` because a maze set to gather withdraws the way out entirely —
    /// an `Ω` a spell cannot see and a player cannot use would be the only mark
    /// here that lies.
    pub exit: Option<usize>,
    /// Squares still holding something to pick up.
    ///
    /// Empty for an ordinary walk. Indices into `squares`, like `at` and `exit`.
    pub spoils: Vec<usize>,
}

impl Stacks {
    /// The picture's size in character cells.
    ///
    /// One character per square, so a step of one square is a step of one
    /// character.
    #[must_use]
    pub fn size(&self) -> (u16, u16) {
        let width = self.width.max(1);
        let height = u16::try_from(self.squares.len())
            .unwrap_or(u16::MAX)
            .div_euclid(width);
        (width, height)
    }

    /// Whether the square one step from the reading is floor.
    ///
    /// `way` indexes the sim's `Way::ALL` — north, east, south, west. This crate
    /// cannot name that type (the dependency runs the other way), so the index
    /// is the shared vocabulary.
    #[must_use]
    pub fn open(&self, way: usize) -> bool {
        let width = usize::from(self.width.max(1));
        let (x, y) = (self.at % width, self.at / width);
        let next = match way {
            0 => y.checked_sub(1).map(|y| y * width + x),
            1 => (x + 1 < width).then_some(self.at + 1),
            2 => Some(self.at + width),
            _ => x.checked_sub(1).map(|x| y * width + x),
        };
        next.and_then(|next| self.squares.get(next))
            .is_some_and(|square| !square.wall)
    }

    /// Floor the reading has stood in, against floor there is.
    ///
    /// The same pair the lectern's panel meter reports, so the full-pane view
    /// needs no second definition of *explored*. Walls are not counted: they
    /// are most of the grid and none of the walk.
    #[must_use]
    pub fn explored(&self) -> (usize, usize) {
        let floor = self.squares.iter().filter(|square| !square.wall);
        floor.fold((0, 0), |(walked, total), square| {
            (walked + usize::from(square.marks > 0), total + 1)
        })
    }
}

/// A wall the reading has proved is there.
pub(crate) const WALL: char = '█';
/// Walked once. Trémaux's first mark.
pub(crate) const ONCE: char = '▒';
/// Walked twice, and finished with.
pub(crate) const TWICE: char = '░';
/// The reading itself.
pub(crate) const HEAD: char = '☼';
/// The way out.
pub(crate) const EXIT: char = 'Ω';
/// Something scattered through the maze, waiting to be picked up.
///
/// In CP437 at 0x04, checked by `ALPHABET` below — `Cell::new` substitutes
/// silently, and a drawing is the hardest place to notice a smudge.
pub(crate) const SPOIL: char = '♦';

/// Every glyph the picture can put on the screen.
///
/// A list so a test can walk it, and needed nowhere else — the picture is drawn
/// here, so no frontend ever names a glyph.
#[cfg(test)]
const ALPHABET: [char; 7] = [' ', WALL, ONCE, TWICE, HEAD, EXIT, SPOIL];

/// What is at one coordinate of the picture.
///
/// `col` and `row` are relative to the picture's own top-left and index the
/// grid directly — a wall is a square, not a line between two.
pub(crate) fn cell(maze: &Stacks, col: u16, row: u16) -> (char, Style) {
    let width = maze.width.max(1);
    if col >= width {
        return (' ', Style::NORMAL);
    }
    let index = usize::from(row) * usize::from(width) + usize::from(col);
    let Some(square) = maze.squares.get(index) else {
        return (' ', Style::NORMAL);
    };

    // The head outranks everything, including the way out: once they are the
    // same square the maze is solved and the picture is gone.
    if index == maze.at {
        return (HEAD, Style::BRIGHT);
    }
    if square.wall {
        return (WALL, Style::DIM);
    }
    // Not a giveaway: `survey east` already answers `exit` from the same square.
    if maze.exit == Some(index) {
        return (EXIT, Style::SUCCESS);
    }
    // A spoil outranks the mark under it: it is what the walk is *for*, and
    // `survey east` already answers `spoil` from the square beside it.
    if maze.spoils.contains(&index) {
        return (SPOIL, Style::SUCCESS);
    }
    // Floor nobody has walked draws nothing: the walls around it are already on
    // screen, so the corridor is a gap in them (a `·` sat here once).
    match square.marks {
        0 => (' ', Style::NORMAL),
        1 => (ONCE, Style::NORMAL),
        _ => (TWICE, Style::DIM),
    }
}

/// The window onto the maze that `area` can show, and where it starts.
///
/// Returns the rectangle on screen to draw into, and the maze coordinate of its
/// top-left corner.
///
/// It pans rather than refusing. Handing back nothing unless the whole picture
/// fitted was fine at 15 squares; at 33 the map vanished from every small pane.
/// So a pane too small gets a window centred on the reading, clamped inside the
/// maze. When the whole picture fits, nothing moves.
#[must_use]
pub(crate) fn viewport(maze: &Stacks, area: Rect) -> Option<(Rect, u16, u16)> {
    let (cols, rows) = maze.size();
    let width = cols.min(area.cols);
    let height = rows.min(area.rows);
    if width == 0 || height == 0 {
        return None;
    }

    // Where the reading is, in maze coordinates.
    let across = maze.width.max(1);
    let at = u32::try_from(maze.at).unwrap_or(u32::MAX);
    let head_x = u16::try_from(at % u32::from(across)).unwrap_or(0);
    let head_y = u16::try_from(at / u32::from(across)).unwrap_or(0);

    // Centred on the head, then pulled back inside the maze's own bounds. When
    // the window is the whole maze both terms are zero and this is a no-op.
    let from_x = head_x.saturating_sub(width / 2).min(cols - width);
    let from_y = head_y.saturating_sub(height / 2).min(rows - height);

    let onto = Rect::new(
        area.col + (area.cols - width) / 2,
        area.row + (area.rows - height) / 2,
        width,
        height,
    );
    Some((onto, from_x, from_y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cp437;

    /// A `span` × `span` grid of solid wall, with nothing seen.
    fn solid(span: u16) -> Stacks {
        Stacks {
            squares: vec![
                Square {
                    wall: true,
                    marks: 0,
                };
                usize::from(span) * usize::from(span)
            ],
            width: span,
            at: 0,
            exit: Some(usize::from(span) * usize::from(span) - 1),
            spoils: Vec::new(),
        }
    }

    /// The picture as one string per row, for reading a whole maze at once.
    fn drawn(maze: &Stacks) -> Vec<String> {
        let (cols, rows) = maze.size();
        (0..rows)
            .map(|row| (0..cols).map(|col| cell(maze, col, row).0).collect())
            .collect()
    }

    #[test]
    fn every_glyph_the_picture_can_draw_is_in_the_repertoire() {
        // `Cell::new` substitutes silently, so a glyph outside CP437 becomes a
        // smudge rather than a failure, and nothing about it looks wrong.
        for glyph in ALPHABET {
            assert!(
                cp437::is_renderable(glyph),
                "{glyph:?} has no CP437 glyph and would draw as a substitute",
            );
        }
    }

    #[test]
    fn one_square_is_one_character() {
        // Why a wall is a square: cells with walls between them drew `2w+1`
        // across, so one step moved the reading two characters.
        let maze = solid(15);
        assert_eq!(maze.size(), (15, 15));
        assert_eq!(drawn(&maze).len(), 15);
        assert_eq!(drawn(&maze)[0].chars().count(), 15);
    }

    #[test]
    fn the_whole_maze_is_drawn_and_only_the_walking_is_not() {
        // The fog is gone (§19): every wall is on screen from the moment the
        // maze opens, so walking changes the marks and nothing else.
        let mut maze = solid(5);
        maze.at = 6;
        maze.squares[6] = Square {
            wall: false,
            marks: 1,
        };
        maze.squares[7] = Square {
            wall: false,
            marks: 0,
        };
        let rows = drawn(&maze);
        assert_eq!(rows[0], "█████", "the outer wall was hidden");
        assert_eq!(rows[1], "█☼ ██", "an unwalked way should be a gap in wall");
    }

    #[test]
    fn a_walked_path_draws_as_one_unbroken_run() {
        // A path is a solid run with the wall it was cut through either side.
        let mut maze = solid(7);
        maze.at = 8;
        maze.exit = Some(12);
        for index in 8..=12 {
            maze.squares[index] = Square {
                wall: false,
                marks: 1,
            };
        }
        assert_eq!(drawn(&maze)[1], "█☼▒▒▒Ω█", "the path has holes in it");
        assert_eq!(drawn(&maze)[0], "███████", "the maze is not drawn whole");
    }

    #[test]
    fn the_marks_are_told_apart_and_the_finished_ones_recede() {
        // Telling `once` from `twice` is how a player debugs a looping solver.
        let mut maze = solid(5);
        maze.at = 5;
        maze.exit = Some(24);
        for (index, marks) in [(5, 1), (6, 1), (7, 7)] {
            maze.squares[index] = Square { wall: false, marks };
        }
        assert_eq!(cell(&maze, 1, 1).0, ONCE);
        assert_eq!(
            cell(&maze, 2, 1).0,
            TWICE,
            "a mark beyond two lost its word"
        );
    }

    #[test]
    fn floor_beside_the_path_but_never_walked_draws_nothing() {
        // The `·` that used to sit here said what the gap in the wall says.
        let mut maze = solid(3);
        maze.at = 4;
        maze.exit = Some(8);
        maze.squares[4] = Square {
            wall: false,
            marks: 1,
        };
        maze.squares[5] = Square {
            wall: false,
            marks: 0,
        };
        assert_eq!(cell(&maze, 2, 1).0, ' ', "an unwalked way drew something");
    }

    #[test]
    fn a_picture_that_fits_is_centred_and_still() {
        // A maze small enough to show whole must not drift under a walking
        // reading.
        let mut maze = solid(15);
        maze.at = 7 * 15 + 7;
        assert_eq!(
            viewport(&maze, Rect::new(2, 3, 17, 17)),
            Some((Rect::new(3, 4, 15, 15), 0, 0)),
            "a picture with room to spare was not centred and still",
        );
    }

    #[test]
    fn a_pane_too_small_gets_a_window_on_the_reading() {
        // Handing back nothing meant the map vanished from every small pane
        // once the maze grew.
        let mut maze = solid(33);
        maze.at = 20 * 33 + 20;
        let (onto, from_x, from_y) =
            viewport(&maze, Rect::new(0, 0, 11, 9)).expect("no window at all");
        assert_eq!(onto, Rect::new(0, 0, 11, 9));
        // Centred on the head: 20 - 11/2 = 15, and 20 - 9/2 = 16.
        assert_eq!((from_x, from_y), (15, 16));
    }

    #[test]
    fn the_window_never_runs_off_the_edge_of_the_maze() {
        // Clamped, so a reading in a corner sees maze rather than emptiness
        // beside it.
        let mut maze = solid(33);
        maze.at = 0;
        assert_eq!(
            viewport(&maze, Rect::new(0, 0, 11, 9)).map(|w| (w.1, w.2)),
            Some((0, 0))
        );
        maze.at = 32 * 33 + 32;
        assert_eq!(
            viewport(&maze, Rect::new(0, 0, 11, 9)).map(|w| (w.1, w.2)),
            Some((22, 24)),
            "the far corner did not pull the window back inside",
        );
    }

    #[test]
    fn walls_are_not_counted_as_somewhere_to_go() {
        // Most of the grid is wall, so counting it would peg the meter near a
        // third before the reading had walked anywhere.
        let mut maze = solid(3);
        maze.squares[4].wall = false;
        maze.squares[4].marks = 1;
        maze.squares[5].wall = false;
        assert_eq!(maze.explored(), (1, 2));
    }
}
