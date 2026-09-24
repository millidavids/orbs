//! The archive's stacks (DESIGN.md §10, §19).
//!
//! The world remembers, so the spell does not have to. §8's language has no
//! variables, so a maze solved by *searching* would defeat pillar 3 unless the
//! maze holds the search's state. Trémaux needs no memory beyond marks in the
//! passages: mark what you enter, turn back at a junction you have seen, prefer
//! the least-walked way. So the cells mark themselves and a solver is a rule:
//!
//! ```text
//! repeat until the stacks is idle
//!   if north has exit
//!     follow north
//!   else
//!   if north has passage
//!     follow north
//!   else
//!   if north has 1 or fewer marks and not north has back and not north has wall
//!     follow north
//!   ...
//! end
//! ```
//!
//! That is depth-first search performed physically: the marks are the visited
//! set, turning back is the stack pop, and the reading head *is* the stack
//! pointer. The comparison grammar added no memory — it stopped a `u8` being
//! read through the two-value lens of `walked` and `twice` (§19).
//!
//! The reading moves, never the wizard: §7's filesystem is already the game's
//! space, so a second spatial system would be a second answer to "where am I".
//! It is also why a spell can work the stacks at all, `may_issue` forbidding a
//! spell from walking.

use bevy_ecs::prelude::*;

/// What the maze can say about a direction.
///
/// A closed vocabulary, offered by the scene whether or not a maze is open: a
/// solver's `if` names these words at *cast*, which is exactly when none of
/// them is true of anything. See `tower::scene_at`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sense {
    /// The way is open and unwalked.
    Passage,
    /// There is no way through.
    Wall,
    /// The way out.
    Exit,
}

/// The way the reading last came from.
///
/// A second fact about a direction, not a fifth [`Sense`]: a way can be `walked`
/// *and* the way you came.
///
/// Without it there is no correct solver — Trémaux turns back *by the passage
/// you came along*, and nothing else could name it. The four-tier ladder solved
/// 7×7 backtracker mazes and nothing harder: four runs in eight at 16×16, and
/// eleven in twelve cycled for ever on Prim's denser mazes. With this word the
/// same ladder solves every maze tried in at most 708 steps.
pub const BACK: &str = "back";

/// A way whose next square holds something worth picking up.
///
/// A second fact about a direction, exactly as [`BACK`] is, and outside
/// [`Sense::ALL`] for the same reason: a way can be `passage` *and* hold a
/// spoil, and a solver asks both.
pub const SPOIL: &str = "spoil";

/// How many times the square a way leads to has been walked.
///
/// The only reading that carries a number. It replaced `walked` and `twice`, two
/// buckets over a `u8` the maze had all along, so a ladder could not prefer the
/// less-trodden of two ways. Outside [`Sense::ALL`] like [`BACK`] and [`SPOIL`].
///
/// Published from one, never nought, a pile at zero being despawned everywhere
/// else in the tower. Absence answers nought to a comparison, which makes
/// `has 1 or fewer marks` true of unwalked floor with no node saying so.
pub const MARKS: &str = "marks";

/// Every word a way can be asked about, whether or not a maze is open.
///
/// One list, the scene and the maze having to speak the same vocabulary. Built
/// inline in `scene_at`, a reading dropped from the chain compiles, casts clean
/// and answers no for ever.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    Sense::ALL
        .into_iter()
        .chain([BACK, SPOIL, MARKS])
        .chain(Errand::ALL)
        .collect()
}

/// What the stacks are being walked *for*.
///
/// The maze's modifier, and what a scroll changes. Published on the **stacks**
/// as an ordinary named child, so a spell asks `if the stacks has gleaning`
/// through the `has` question it already has, and one solver handles both.
///
/// Not a `State` variant: `State` is read by `is_busy`, `bar_of` and
/// `is working`/`is idle` at once, and an errand is not a state of the
/// instrument.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Errand {
    /// Reach the way out. What the stacks always open as.
    #[default]
    Way,
    /// Gather everything scattered through it. **There is no way out while this
    /// is on** — see [`Maze::reading`], which stops publishing [`Sense::Exit`].
    Glean,
}

impl Errand {
    /// The word this errand is published and named by, if it has one.
    ///
    /// `Way` has none: it is what the stacks already are, and a word for the
    /// absence of a modifier is a permanent child saying nothing. A spell writes
    /// `if not the stacks has gleaning` instead.
    #[must_use]
    pub const fn word(self) -> Option<&'static str> {
        match self {
            Self::Way => None,
            Self::Glean => Some("gleaning"),
        }
    }

    /// Every word an errand can be published as, for the scene to offer.
    pub const ALL: [&'static str; 1] = ["gleaning"];
}

impl Sense {
    /// Every reading, as the scene offers them. Three, where there were five:
    /// `walked` and `twice` were buckets over `Square::marks` and are [`MARKS`]
    /// now, compared with `or more` / `or fewer`.
    pub const ALL: [&'static str; 3] = ["passage", "wall", "exit"];

    /// The word this reading answers to.
    ///
    /// Not prose: the tables emit facts rather than sentences (§19). What the
    /// orb *says* about a reading is in `prose.toml`.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Passage => "passage",
            Self::Wall => "wall",
            Self::Exit => "exit",
        }
    }
}

/// The four ways out of a cell.
///
/// Absolute, not relative: a heading is state, and Trémaux needs none — while
/// solving the loops the wall-follower a heading enables cannot (§19).
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
/// A square, not a cell with four walls. Walls *between* cells draw as `2w+1`
/// characters, so one step moved the reading two characters across the picture.
/// Making the wall a square means a step is a step.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Square {
    /// Whether this is solid. Nothing walks through it and nothing marks it.
    pub wall: bool,
    /// How many times the reading has entered. Trémaux's mark.
    pub marks: u8,
}

/// The stacks, and where the reading has reached in them.
///
/// A component, not a node per square: one entity each would flood `survey`, the
/// parser's scene and `NodeId`s with nodes the player can never name (§19).
///
/// The first component not reconstructible from names and `NodeId`s, a debt
/// against §8's serialisable in-flight state (§19).
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
    /// What this walk is for.
    errand: Errand,
    /// Squares still holding something to pick up, in the order they were
    /// scattered.
    ///
    /// Empty under [`Errand::Way`], which has nothing to gather. A `Vec` rather
    /// than a set because the order is part of the replay: it is written once,
    /// from one draw, and only ever shrinks.
    spoils: Vec<usize>,
}

impl Maze {
    /// A maze from its squares, with the reading at `at`.
    ///
    /// Built in one go, never over ticks: spawning across ticks issues `NodeId`s
    /// at a rate depending on how the ticks were consumed, so a live-watched run
    /// and a `meditate`-collapsed one differ from one seed (`heat.rs`).
    #[must_use]
    pub fn new(mut squares: Vec<Square>, width: usize, at: usize, exit: usize) -> Self {
        // The starting square counts as walked, because it has been. Unmarked,
        // the first step back reports the way you came as an unwalked
        // `passage`, and a Trémaux solver loops at the start for ever.
        if let Some(square) = squares.get_mut(at) {
            square.marks = 1;
        }
        Self {
            squares,
            width,
            at,
            exit,
            came: None,
            errand: Errand::Way,
            spoils: Vec::new(),
        }
    }

    /// Set this maze an errand, and what it has to gather to finish it.
    ///
    /// In one go, for `Maze::new`'s reason: scattering spoils as the walk went
    /// would issue them at a rate depending on tick consumption.
    pub fn set_errand(&mut self, errand: Errand, spoils: Vec<usize>) {
        self.errand = errand;
        self.spoils = spoils;
    }

    /// What this walk is for.
    #[must_use]
    pub const fn errand(&self) -> Errand {
        self.errand
    }

    /// Squares still holding something to pick up.
    #[must_use]
    pub fn spoils(&self) -> &[usize] {
        &self.spoils
    }

    /// Every square the reading could still be sent to gather from.
    ///
    /// Unwalked floor only, never where the reading stands: one dropped
    /// underfoot is collected on the frame it is scattered, and one on walked
    /// floor makes spending the scroll on a half-explored maze a partial refund.
    #[must_use]
    pub fn scatterable(&self) -> Vec<usize> {
        self.squares
            .iter()
            .enumerate()
            .filter(|(at, square)| !square.wall && square.marks == 0 && *at != self.at)
            .map(|(at, _)| at)
            .collect()
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
    /// The only honest meter: a brew knows its duration, a maze does not. How
    /// much has been seen only ever grows, which is what a bar has to do. Walls
    /// are most of the grid and none of the walk, so they are not counted.
    #[must_use]
    pub fn explored(&self) -> (u64, u64) {
        let floor = self.squares.iter().filter(|square| !square.wall);
        let (walked, total) = floor.fold((0, 0), |(walked, total), square| {
            (walked + u64::from(square.marks > 0), total + 1)
        });
        (walked, total)
    }

    /// Whether the walk is over — which is a different question per errand.
    ///
    /// Reaching the way out finishes a [`Way`](Errand::Way); gathering the last
    /// spoil finishes a [`Glean`](Errand::Glean). The two never both apply,
    /// because a gleaning maze publishes no exit.
    #[must_use]
    pub const fn solved(&self) -> bool {
        match self.errand {
            Errand::Way => self.at == self.exit,
            Errand::Glean => self.spoils.is_empty(),
        }
    }

    /// Whether the square beyond `way` holds something to pick up.
    ///
    /// The question [`SPOIL`] answers, and it rides beside the reading rather
    /// than replacing it — a spoil is nearly always in a `passage`, and a solver
    /// wants to know both.
    #[must_use]
    pub fn spoil(&self, way: Way) -> bool {
        self.beyond(self.at, way)
            .is_some_and(|beyond| self.spoils.contains(&beyond))
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
    /// `Exit` outranks everything, including a wall: a solver's first rule is
    /// *take the way out if it is there*, and an exit reported as `walked` fires
    /// it on the wrong square. A gleaning maze publishes no exit at all, or the
    /// top rung would walk onto it and take the same rung for ever.
    ///
    /// `None` means one thing: open, already walked, and not the way out. Every
    /// other absence is `Some(Sense::Wall)`, or *there is no such direction*
    /// would read as *walked floor*. A walked way says its
    /// [`marks`](Self::marks) count instead.
    #[must_use]
    pub fn reading(&self, way: Way) -> Option<Sense> {
        let Some(beyond) = self.beyond(self.at, way) else {
            return Some(Sense::Wall);
        };
        let Some(square) = self.squares.get(beyond) else {
            return Some(Sense::Wall);
        };
        if square.wall {
            return Some(Sense::Wall);
        }
        if beyond == self.exit && self.errand == Errand::Way {
            return Some(Sense::Exit);
        }
        (square.marks == 0).then_some(Sense::Passage)
    }

    /// How many times the square that way has been walked, if it is open.
    ///
    /// `None` for a wall: absence answers nought, so `has 1 or fewer marks`
    /// would otherwise send the reading into stone. Derived from
    /// [`reading`](Self::reading)'s lookup, so the two cannot disagree.
    #[must_use]
    pub fn marks(&self, way: Way) -> Option<u8> {
        let beyond = self.beyond(self.at, way)?;
        let square = self.squares.get(beyond)?;
        (!square.wall).then_some(square.marks)
    }

    /// Move the reading one square, marking where it arrives.
    ///
    /// Returns whether it moved. The maze does the marking, which lets a spell
    /// hold a rule rather than a memory. A spoil is picked up here, by the
    /// walking, so an arrow key and a bound solver collect on the same rule.
    pub fn tread(&mut self, way: Way) -> bool {
        // `Some(Wall)`, not `None`: a walked-open way reads `None`, and treating
        // that as impassable would stop the reading retracing its steps, which
        // is the `back` tier and the whole of what makes a solver terminate.
        if self.reading(way) == Some(Sense::Wall) {
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
        self.spoils.retain(|spoil| *spoil != beyond);
        true
    }

    /// The maze as a frontend draws it — whole. Fogged it carried nothing the
    /// linear stream lacked; entire, walking it is routing rather than feeling
    /// along a wall.
    ///
    /// The cost is §14's (§19): a sighted player sees more than a listener. A
    /// *spell* still solves from the four readings alone, and the cheap repair
    /// is a spoken bearing to the exit rather than a return to fog.
    #[must_use]
    pub fn view(&self) -> orbs_render::Stacks {
        let squares = self
            .squares
            .iter()
            .map(|square| orbs_render::Square {
                wall: square.wall,
                marks: square.marks,
            })
            .collect();

        orbs_render::Stacks {
            squares,
            width: u16::try_from(self.width).unwrap_or(u16::MAX),
            at: self.at,
            // The picture says what the readings do. A gleaning maze publishes
            // no `exit`, so drawing `Ω` would offer a way out that no spell can
            // see and no player can use.
            exit: (self.errand == Errand::Way).then_some(self.exit),
            spoils: self.spoils.clone(),
        }
    }
}
/// Reading a maze out into a save, and back.
///
/// Here rather than in `crate::save` for `Ward::to_save`'s reason: the fields
/// are private. It settles the debt above — `Maze` was the first component not
/// reconstructible from names and `NodeId`s (§8).
impl Maze {
    /// Everything a save needs to put this maze back.
    ///
    /// The walls go out as a picture and the marks as a sparse list: 33 × 23 is
    /// 759 squares, and an array-of-tables would be hundreds of lines of
    /// `{ wall = true, marks = 0 }` in a save §15 says stays readable.
    pub(crate) fn to_save(&self) -> crate::save::MazeSave {
        crate::save::MazeSave {
            width: self.width,
            at: self.at,
            exit: self.exit,
            came: self.came.map(|way| crate::save::way_word(way).to_owned()),
            errand: crate::save::errand_word(self.errand).to_owned(),
            spoils: self.spoils.clone(),
            walls: self
                .squares
                .chunks(self.width.max(1))
                .map(|row| {
                    row.iter()
                        .map(|square| if square.wall { '#' } else { '.' })
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n"),
            marks: self
                .squares
                .iter()
                .enumerate()
                .filter(|(_, square)| square.marks > 0)
                .map(|(at, square)| (at, square.marks))
                .collect(),
        }
    }

    /// Put one back.
    ///
    /// Draws no randomness, unlike the generator: the layout is read, not dug. A
    /// restore that re-dug would move `RngStream::Archive` and change every
    /// later roll in the session.
    pub(crate) fn from_save(save: &crate::save::MazeSave) -> Self {
        let squares: Vec<Square> = save
            .walls
            .lines()
            .flat_map(str::chars)
            .map(|glyph| Square {
                wall: glyph == '#',
                marks: 0,
            })
            .collect();

        // Range-checked, because §15 invites hand-editing. Nothing panics today,
        // but a reading standing outside its own grid is a maze `follow` cannot
        // move and `survey` says nothing about — a silently unplayable archive.
        let width = save.width.max(1);
        let squares: Vec<Square> = squares;
        let last = squares.len().saturating_sub(1);
        let mut maze = Self {
            width,
            at: save.at.min(last),
            exit: save.exit.min(last),
            came: save.came.as_deref().and_then(crate::save::way_from),
            errand: crate::save::errand_from(&save.errand),
            // A spoil outside the grid is unreachable, so it is dropped rather
            // than left as an errand the player can never finish.
            spoils: save
                .spoils
                .iter()
                .copied()
                .filter(|at| *at < squares.len())
                .collect(),
            squares,
        };
        for &(at, marks) in &save.marks {
            if let Some(square) = maze.squares.get_mut(at) {
                square.marks = marks;
            }
        }
        maze
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A straight corridor `len` squares long running east, walls either side,
    /// with the exit at the far end.
    ///
    /// A `len + 2` by 3 grid, so the corridor has real wall squares around it
    /// and `reading` answers `wall` north and south with no special case.
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
        assert_eq!(
            maze.reading(Way::West),
            Some(Sense::Wall),
            "walked off the edge",
        );
        assert_eq!(maze.reading(Way::North), Some(Sense::Wall));
        assert_eq!(maze.reading(Way::East), Some(Sense::Passage));
        // A wall has no count at all, which stops `1 or fewer marks` sending a
        // solver into stone: absence answers nought, so a wall reporting `0`
        // would satisfy every `or fewer` rung there is.
        assert_eq!(maze.marks(Way::West), None, "a wall reported a count");
        assert_eq!(maze.marks(Way::East), Some(0), "open floor has a count");
    }

    #[test]
    fn the_maze_counts_what_the_reading_walks() {
        // The whole design in one assertion: a spell has no memory, so if the
        // squares do not remember, Trémaux cannot be written and the archive is
        // the one room automation can never reach.
        let mut maze = corridor(4);
        assert_eq!(maze.reading(Way::East), Some(Sense::Passage));
        assert!(maze.tread(Way::East));
        assert_eq!(maze.marks(Way::West), Some(1), "the mark was lost");
        assert_eq!(
            maze.reading(Way::West),
            None,
            "walked floor is no longer a word",
        );

        assert!(maze.tread(Way::West));
        assert!(maze.tread(Way::East));
        assert_eq!(maze.marks(Way::West), Some(2));

        // And past the old ceiling, which is the point: `twice` could not tell
        // these apart and a ladder could not prefer the less-trodden way.
        assert!(maze.tread(Way::West));
        assert!(maze.tread(Way::East));
        assert_eq!(maze.marks(Way::West), Some(3));
    }

    #[test]
    fn the_exit_outranks_every_other_reading() {
        // A solver's first rule is *take the way out if it is there*, so an exit
        // reported as walked floor on a second pass fires it on the wrong
        // square. The one reading that outranks a tread count.
        let mut maze = corridor(2);
        assert_eq!(maze.reading(Way::East), Some(Sense::Exit));
        assert!(maze.tread(Way::East));
        assert!(maze.solved());
        assert!(maze.tread(Way::West));
        assert_eq!(maze.reading(Way::East), Some(Sense::Exit), "the exit moved",);
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
        // What the grid exists for: with walls *between* cells the picture was
        // `2w+1` across, so one step moved the reading two characters.
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
        // The fog is gone (§19): the view withheld exactly what the four
        // `survey` readings answer, and hands the maze over entire now.
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
        for sense in [Sense::Passage, Sense::Wall, Sense::Exit] {
            assert!(
                Sense::ALL.contains(&sense.word()),
                "{} is spoken and not offered",
                sense.word(),
            );
        }
        // And the three that are not `Sense`es, which is the half this test
        // missed when `back` and `spoil` were added: chained on separately, so
        // nothing here would have noticed one dropped.
        for word in [BACK, SPOIL, MARKS] {
            assert!(
                readings().contains(&word),
                "{word} rides outside Sense::ALL and fell off the chain",
            );
        }
        assert_eq!(
            readings().len(),
            Sense::ALL.len() + 3 + Errand::ALL.len(),
            "a reading is offered twice, or one went missing",
        );
    }
}
