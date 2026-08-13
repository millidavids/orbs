//! The archive's labyrinth (DESIGN.md §10, §19).
//!
//! # The world remembers, so the spell does not have to
//!
//! §8's language has no variables, no counters and no numeric comparison, and a
//! condition can only ask whether a named thing is present. A maze solved by
//! *searching* would therefore be the one room in the game that permanently
//! defeats pillar 3 — you could never teach the orb to do it.
//!
//! Unless the **maze** holds the search's state. Trémaux's algorithm needs no
//! memory beyond marks in the passages: enter a passage and mark it, turn back
//! at a junction you have seen before, never take a passage marked twice. So the
//! cells mark themselves, the maze publishes what is adjacent as ordinary nodes,
//! and a solver becomes a rule rather than a search:
//!
//! ```text
//! repeat 400
//!   if north has exit
//!     tread north
//!   end
//!   if north has passage and not north has walked
//!     tread north
//!   end
//!   ...
//! end
//! ```
//!
//! That is depth-first search, performed physically — the marks are the visited
//! set, and turning back the way you came is the stack pop, because the reading
//! head *is* the stack pointer. It needs no grammar change, which is the
//! strongest defence §19's refusal of numeric comparison has: the language did
//! not need to grow, the world needed to remember.
//!
//! # The reading moves, never the wizard
//!
//! §7's filesystem is already the game's space and `attend` is already how you
//! move through it. A second spatial system would be a second answer to "where
//! am I". So the player stands in the archive throughout and it is the *reading*
//! that threads the labyrinth — which is also why a spell can work one, since
//! `may_issue` forbids a spell from walking.

use bevy_ecs::prelude::*;

/// What the maze can say about a direction.
///
/// **A closed vocabulary, offered by the scene whether or not a maze is open.**
/// A solver's `if` names these words at the moment it is *cast*, which is
/// exactly when none of them is true of anything — see `tower::scene_at`, where
/// registering them is the single change that makes the whole design work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sense {
    /// The way is open and unwalked.
    Passage,
    /// There is no way through.
    Wall,
    /// Walked once. Trémaux's first mark.
    Walked,
    /// Walked twice, and never to be entered again.
    Twice,
    /// The way out.
    Exit,
}

/// The way the reading last came from.
///
/// **A second fact about a direction, not a fifth [`Sense`].** A way can be
/// `walked` *and* the way you came, and the two answer different questions — so
/// this is raised as an extra child rather than replacing the reading.
///
/// # Without it there is no correct solver
///
/// The four-tier ladder — exit, unwalked, walked, twice — reads like Trémaux and
/// is not, because Trémaux's rule is *"when you arrive at a junction you have
/// seen before **by the passage you came along**, turn back"*, and nothing in the
/// language could say which passage that was. It solved 7×7 mazes carved by a
/// recursive backtracker and nothing harder: at 16×16 it finished four runs in
/// eight, and on the denser mazes Prim's carves it cycled for ever on eleven in
/// twelve. Not slow — cycling, at a junction where two ways read alike and a
/// fixed compass order sent it back where it came from.
///
/// With this word the same ladder solves every maze tried, both generators, both
/// sizes, in at most 708 steps. It is one word and it is the difference between
/// the archive being automatable and not.
pub const BACK: &str = "back";

impl Sense {
    /// Every reading, as the scene offers them.
    pub const ALL: [&'static str; 5] = ["passage", "wall", "walked", "twice", "exit"];

    /// The word this reading answers to.
    ///
    /// **Not prose.** One word naming a state is the parser's kind of fact, and
    /// §19 already settles that the tables emit facts and never sentences. What
    /// the orb *says* about a reading lives in `prose.toml`.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Passage => "passage",
            Self::Wall => "wall",
            Self::Walked => "walked",
            Self::Twice => "twice",
            Self::Exit => "exit",
        }
    }
}

/// The four ways out of a cell.
///
/// **Absolute, not relative.** Relative directions would need a heading, which
/// is state the maze would have to keep and a spell would have to reason about —
/// and Trémaux needs no heading at all, while solving looped mazes that the
/// wall-follower a heading would enable cannot. A heading is a later rung on the
/// ladder (§19), bought rather than assumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
    /// Up the grid.
    North,
    /// Rightward.
    East,
    /// Down the grid.
    South,
    /// Leftward.
    West,
}

impl Way {
    /// Every way, in the order the sense nodes are raised.
    pub const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    /// The word a player types, and the sense node's name.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::North => "north",
            Self::East => "east",
            Self::South => "south",
            Self::West => "west",
        }
    }

    /// The way back.
    #[must_use]
    pub const fn back(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }
}

/// One square of the grid: a wall, or floor the reading may have walked.
///
/// **A square, not a cell with four walls.** The maze used to be 7×7 cells with
/// the walls *between* them, which draws as `2w+1` characters — so one step took
/// the reading two characters across the picture and read, correctly, as moving
/// two spaces at a time. Making the wall a square of its own means a step is a
/// step: the corridor between two cells is somewhere you stand.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Square {
    /// Whether this is solid. Nothing walks through it and nothing marks it.
    pub wall: bool,
    /// How many times the reading has entered. Trémaux's mark.
    pub marks: u8,
}

/// A labyrinth, and where the reading has reached in it.
///
/// **A component, not a node per square.** Spawning one entity each would flood
/// `survey`, the parser's scene and `NodeId`s with nodes the player can never
/// name — and §19 records archive order changing what a phrase resolves to as a
/// bug that no test catches.
///
/// It is the first component in the game not reconstructible from names and
/// `NodeId`s, which is a debt against §8's *"in-flight state is first-class
/// serialisable"* and is recorded as one in §19.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Maze {
    /// Squares, row-major.
    squares: Vec<Square>,
    /// How wide, in squares.
    width: usize,
    /// Where the reading is, as an index into `squares`.
    at: usize,
    /// Where the way out is.
    exit: usize,
    /// The way back to where the reading last stood.
    came: Option<Way>,
}

impl Maze {
    /// A maze from its squares, with the reading at `at`.
    ///
    /// **Built in one go, never over ticks.** `heat.rs` records that spawning
    /// across ticks issues `NodeId`s at a rate depending on how the ticks were
    /// consumed, so a live-watched run and a `meditate`-collapsed one produce
    /// different worlds from one seed. A maze that grew as it was walked would
    /// have the same hazard.
    #[must_use]
    pub fn new(mut squares: Vec<Square>, width: usize, at: usize, exit: usize) -> Self {
        // **The starting square counts as walked**, because it has been: the
        // reading is standing in it. Left unmarked, the first step back reports
        // the way you came as an unwalked `passage` — and a Trémaux solver would
        // take it, walk to the start, and loop there for ever.
        if let Some(square) = squares.get_mut(at) {
            square.marks = 1;
        }
        Self {
            squares,
            width,
            at,
            exit,
            came: None,
        }
    }

    /// Where the reading is.
    #[must_use]
    pub const fn at(&self) -> usize {
        self.at
    }

    /// The way back to where the reading last stood, if it has moved.
    ///
    /// See [`BACK`], which is the word this is published as.
    #[must_use]
    pub const fn came(&self) -> Option<Way> {
        self.came
    }

    /// How wide, in squares.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Where the way out is.
    #[must_use]
    pub const fn exit(&self) -> usize {
        self.exit
    }

    /// Every square, row-major.
    #[must_use]
    pub fn squares(&self) -> &[Square] {
        &self.squares
    }

    /// Floor walked, against floor there is.
    ///
    /// **The only honest meter a labyrinth has.** A brew knows its duration
    /// before it starts; a maze does not — how long it takes is what the player's
    /// rule decides. What *can* be reported is how much of it has been seen, and
    /// that only ever grows, which is what a bar has to do.
    ///
    /// Walls are not counted. They are most of the grid and none of the walk, so
    /// counting them would peg the bar near a third for ever.
    #[must_use]
    pub fn explored(&self) -> (u64, u64) {
        let floor = self.squares.iter().filter(|square| !square.wall);
        let (walked, total) = floor.fold((0, 0), |(walked, total), square| {
            (walked + u64::from(square.marks > 0), total + 1)
        });
        (walked, total)
    }

    /// Whether the reading has reached the way out.
    #[must_use]
    pub const fn solved(&self) -> bool {
        self.at == self.exit
    }

    /// The square `way` leads to, if there is one.
    fn beyond(&self, from: usize, way: Way) -> Option<usize> {
        let (x, y) = (from % self.width, from / self.width);
        let height = self.squares.len() / self.width.max(1);
        let (x, y) = match way {
            Way::North => (x, y.checked_sub(1)?),
            Way::South => (x, y + 1),
            Way::East => (x + 1, y),
            Way::West => (x.checked_sub(1)?, y),
        };
        (x < self.width && y < height).then_some(y * self.width + x)
    }

    /// What the reading can say about `way` from where it stands.
    ///
    /// **`Exit` outranks everything**, including a wall — a solver's first rule
    /// is *if the way out is there, take it*, and a reading that reported the
    /// exit as `walked` would make that rule fire on the wrong square.
    #[must_use]
    pub fn reading(&self, way: Way) -> Sense {
        let Some(beyond) = self.beyond(self.at, way) else {
            return Sense::Wall;
        };
        let Some(square) = self.squares.get(beyond) else {
            return Sense::Wall;
        };
        if square.wall {
            return Sense::Wall;
        }
        if beyond == self.exit {
            return Sense::Exit;
        }
        match square.marks {
            0 => Sense::Passage,
            1 => Sense::Walked,
            _ => Sense::Twice,
        }
    }

    /// Move the reading one square, marking where it arrives.
    ///
    /// Returns whether it moved. **The maze does the marking**, which is what
    /// lets a spell hold a rule rather than a memory.
    pub fn tread(&mut self, way: Way) -> bool {
        if matches!(self.reading(way), Sense::Wall) {
            return false;
        }
        let Some(beyond) = self.beyond(self.at, way) else {
            return false;
        };
        self.at = beyond;
        self.came = Some(way.back());
        if let Some(square) = self.squares.get_mut(beyond) {
            square.marks = square.marks.saturating_add(1);
        }
        true
    }

    /// The maze as a frontend draws it — **whole**.
    ///
    /// It used to fog: a square was handed over only once the reading had stood
    /// in it or beside it, which is exactly what the four `survey` readings
    /// answer, so the picture carried nothing the linear stream lacked. The maze
    /// is now given entire, which makes walking it routing rather than feeling
    /// along a wall.
    ///
    /// **The cost is §14's, and it is recorded in §19**: a sighted player sees
    /// more than a listener does. What survives it is that a *spell* still
    /// solves from the four readings alone, so the automation pillar is
    /// untouched — and the cheap repair, if it bites, is a spoken bearing to the
    /// exit rather than a return to fog.
    #[must_use]
    pub fn view(&self) -> orbs_render::Labyrinth {
        let squares = self
            .squares
            .iter()
            .map(|square| orbs_render::Square {
                wall: square.wall,
                marks: square.marks,
            })
            .collect();

        orbs_render::Labyrinth {
            squares,
            width: u16::try_from(self.width).unwrap_or(u16::MAX),
            at: self.at,
            exit: self.exit,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    /// A straight corridor `len` squares long running east, walls either side,
    /// with the exit at the far end.
    ///
    /// Laid out as a `len + 2` by 3 grid so the corridor has real wall squares
    /// around it — which is the point of squares, and what makes `reading`
    /// answer `wall` for north and south without a special case.
    fn corridor(len: usize) -> Maze {
        let span = len + 2;
        let mut squares = vec![
            Square {
                wall: true,
                marks: 0,
            };
            span * 3
        ];
        for step in 0..len {
            squares[span + 1 + step].wall = false;
        }
        Maze::new(squares, span, span + 1, span + len)
    }

    #[test]
    fn a_wall_is_a_wall_from_either_side() {
        let maze = corridor(3);
        assert_eq!(maze.reading(Way::West), Sense::Wall, "walked off the edge");
        assert_eq!(maze.reading(Way::North), Sense::Wall);
        assert_eq!(maze.reading(Way::East), Sense::Passage);
    }

    #[test]
    fn the_maze_marks_what_the_reading_walks() {
        // **The whole design in one assertion.** A spell has no memory, so if
        // the squares do not remember, Trémaux cannot be written and the archive
        // is the one room automation can never reach.
        let mut maze = corridor(4);
        assert_eq!(maze.reading(Way::East), Sense::Passage);
        assert!(maze.tread(Way::East));
        assert_eq!(maze.reading(Way::West), Sense::Walked, "the mark was lost");

        assert!(maze.tread(Way::West));
        assert!(maze.tread(Way::East));
        assert_eq!(
            maze.reading(Way::West),
            Sense::Twice,
            "a passage walked twice must say so, or Trémaux cannot terminate",
        );
    }

    #[test]
    fn the_exit_outranks_every_other_reading() {
        // A solver's first rule is *take the way out if it is there*. Reporting
        // the exit as `walked` on a second pass would fire that rule on the
        // wrong square.
        let mut maze = corridor(2);
        assert_eq!(maze.reading(Way::East), Sense::Exit);
        assert!(maze.tread(Way::East));
        assert!(maze.solved());
        assert!(maze.tread(Way::West));
        assert_eq!(maze.reading(Way::East), Sense::Exit, "the exit moved");
    }

    #[test]
    fn treading_a_wall_does_nothing() {
        let mut maze = corridor(3);
        let before = maze.at();
        assert!(!maze.tread(Way::North));
        assert_eq!(maze.at(), before);
    }

    #[test]
    fn a_step_is_one_square() {
        // **What the whole grid exists for.** With the walls *between* cells the
        // picture was `2w+1` across, so one step moved the reading two characters
        // and read as moving two spaces at a time.
        let mut maze = corridor(4);
        let before = maze.at();
        assert!(maze.tread(Way::East));
        assert_eq!(maze.at(), before + 1, "a step moved more than one square");
    }

    #[test]
    fn the_meter_counts_floor_and_not_wall() {
        // Wall is most of the grid and none of the walk, so counting it would
        // peg the bar near a third before the reading had gone anywhere.
        let maze = corridor(4);
        assert_eq!(maze.explored(), (1, 4), "the walls were counted");
    }

    #[test]
    fn every_way_has_a_back_and_it_is_an_involution() {
        for way in Way::ALL {
            assert_eq!(way.back().back(), way, "{}", way.word());
            assert_ne!(way.back(), way);
        }
    }

    #[test]
    fn the_view_hands_over_the_whole_maze() {
        // **The fog is gone** (§19). The view used to withhold every square the
        // reading had not stood in or beside, which is exactly what the four
        // `survey` readings answer; it now hands the maze over entire, so
        // walking it is routing rather than feeling along a wall.
        let maze = corridor(4);
        let view = maze.view();

        assert_eq!(view.squares.len(), maze.squares().len());
        assert!(
            view.squares[maze.at()].marks > 0,
            "the start was not stood in"
        );
        assert_eq!(
            view.squares[maze.at() + 2].marks,
            0,
            "a square nobody has walked came back marked",
        );
        assert!(
            !view.squares[maze.at() + 2].wall,
            "the corridor past the next square was withheld",
        );
    }

    #[test]
    fn walking_marks_the_square_it_arrives_in() {
        let mut maze = corridor(4);
        let start = maze.at();
        assert_eq!(maze.view().squares[start + 1].marks, 0);
        assert!(maze.tread(Way::East));
        assert_eq!(
            maze.view().squares[start + 1].marks,
            1,
            "the mark did not reach the view",
        );
    }

    #[test]
    fn the_scene_offers_a_word_for_every_reading() {
        // The vocabulary the scene registers and the vocabulary the maze speaks
        // are the same list, or a solver names a word the maze never says.
        for sense in [
            Sense::Passage,
            Sense::Wall,
            Sense::Walked,
            Sense::Twice,
            Sense::Exit,
        ] {
            assert!(
                Sense::ALL.contains(&sense.word()),
                "{} is spoken and not offered",
                sense.word(),
            );
        }
    }
}
