//! A course of wards across three stations, and the one rule that governs it.
//!
//! DESIGN.md §10 gives defense *"command pressure at 1 Hz, ward placement"* and
//! then says outright that the column is a table rather than a design — and that
//! this domain is *"the one that can still fail the rule"*, because pressure at 1
//! Hz is a reflex mechanic unless something makes it a decision. ROADMAP made
//! finding that something head-of-phase work.
//!
//! **It is the Tower of Hanoi**, and the choice dissolves the problem rather than
//! working around it. There is no clock in it at all: a course waits for ever,
//! every ward is on screen, and the only thing that can go wrong is picking the
//! wrong pair of stations. Outcome follows *what the player chooses given
//! readable state*, which is §10.1's rule stated almost word for word.
//!
//! # The fiction, and why it is not masonry
//!
//! Raw arcane energy wells up in the [`WELLSPRING`]; the wizard draws it through
//! the [`CONDUIT`] a ward at a time and assembles it into the [`BARRIER`]. **The
//! walls of the tower are stone and the wards are not** — what he is shoring up
//! is a magical defence, and the three stations are places a ward can rest on
//! the way there.
//!
//! §19 records the reskin. It was `barbican`/`bastion`/`redoubt` for one
//! version, which committed the whole domain to a fortification metaphor that
//! *"hauling a ward from the barbican to the redoubt"* never made sense inside.
//! `hollow` was the first name for the source and could not be used: it scores
//! **834 against `follow`**, a verb the player types constantly one room over.
//!
//! # The one rule, and what it buys
//!
//! A greater ward will not rest upon a lesser. That single constraint has a
//! consequence worth stating plainly, because the whole domain is built on it:
//!
//! > **Between any two stations, exactly one haul is legal** — unless both are
//! > empty.
//!
//! So a player choosing a *pair* has already chosen a move, and a spell that
//! knows which pair it wants needs only to work out which way round it runs.
//! That is what [`POTENCY`] is published for, and it is the whole of what a spell
//! reads here. `a_haul_between_two_stations_is_unique` is the proof.
//!
//! # What erosion does, and why the parity is published
//!
//! A course is three to seven wards ([`LEAST`], [`MOST`]), and how many is
//! decided by how far the tower's integrity has slipped — see
//! [`erosion`](super::erode). Seven wards is 127 hauls against three wards'
//! seven, so neglect is expensive without ever being ruinous, which is §11.5's
//! *"never ruinous, only slower"*.
//!
//! The height also decides **which way the cyclic solution runs**, and that is
//! why [`ODD`] is a reading. Move the least ward one way round the three stations
//! for an even course and the other way for an odd one; get it backwards and the
//! course assembles at the wrong station. A spell that could not ask would have
//! to be two spells, and a height that never varied would let a player hard-code
//! the answer and stop reading anything — which is why
//! [`height_for`](super::height_for) jitters it.

use bevy_ecs::prelude::*;

/// Where raw arcane energy wells up, and where a course is drawn from.
pub const WELLSPRING: &str = "wellspring";

/// What a ward is drawn through on its way to the barrier.
pub const CONDUIT: &str = "conduit";

/// Where the wards are assembled, and what the tower is defended by.
pub const BARRIER: &str = "barrier";

/// The three stations, source first.
///
/// A course is drawn up at the first and belongs at the last; the middle one is
/// the spare. **Nothing here depends on that** — a `Course` addresses stations
/// by index — but the names were chosen so the fiction and the puzzle agree:
/// energy wells up, passes through, and is assembled.
pub const STATIONS: [&str; 3] = [WELLSPRING, CONDUIT, BARRIER];

/// Where a course is drawn up.
pub const START: usize = 0;

/// Where it belongs.
pub const GOAL: usize = 2;

/// The shortest course the pylon will draw up: seven hauls.
pub const LEAST: usize = 3;

/// The tallest: 127 hauls.
///
/// **Held against the painter's own reservation** by
/// `a_board_has_room_for_the_tallest_course`, from this side, because
/// `orbs-render` may not depend on this crate and so cannot check it from the
/// other.
pub const MOST: usize = orbs_render::TALLEST;

/// How great the ward at the top of a station is. A counted reading, on each.
///
/// **`potency`, not `heft`** — the first name weighed the ward, which is exactly
/// the physical framing the reskin moved away from (§19). Pure arcane energy has
/// magnitude and no mass.
///
/// **Published only while the station holds a ward**, and that is load-bearing
/// rather than tidy: `spell::watch` answers `is empty` by asking whether a node
/// has children, so a station that always carried a `potency` could never be
/// empty and the first two rungs of every solver would be dead. It is the maze's
/// rule for `marks` on an unwalked way, arrived at from the opposite direction.
pub const POTENCY: &str = "potency";

/// Whether this course has an odd number of wards. A bare reading, on the pylon.
pub const ODD: &str = "odd";

/// How the tower's defences stand. A counted reading, on the pylon.
pub const INTEGRITY: &str = "integrity";

/// Every word a spell may ask this domain for.
///
/// The lens's `readings()` is the shape: three words registered unconditionally
/// in `tower::scene`, whether or not a course is open, because a spell compiles
/// at **cast** — when none of them is true of anything — and `bind::stand`
/// recasts every lap.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    vec![POTENCY, ODD, INTEGRITY]
}

/// Which station a name means, by leaf.
///
/// By leaf because a resolved `Place` argument arrives as
/// `/tower/sanctum/wellspring` — `ward::socket_of` has the same shape for the
/// same reason.
#[must_use]
pub fn station_of(name: &str) -> Option<usize> {
    let leaf = crate::parser::leaf(name);
    STATIONS.iter().position(|station| *station == leaf)
}

/// The pylon, wherever it is and whoever is standing where.
///
/// **Not `Cwd`**, unlike `execute::muster`'s own finder: this is what
/// [`erode`](super::erode) uses, and erosion happens on a tick whether the
/// player is in the sanctum, the laboratory or nowhere at all. `Sim::working`
/// walks entities for the same reason and with the same cost.
#[must_use]
pub fn fixture(world: &World) -> Option<Entity> {
    world.iter_entities().find_map(|entity| {
        entity
            .get::<super::Operation>()
            .is_some_and(|operation| operation.0 == crate::parser::Verb::Muster)
            .then(|| entity.id())
    })
}

/// Why a haul was not made.
///
/// Three, and each names something different for the player to do about it —
/// which is §6's rule that a refusal names the way forward.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// One station named twice.
    Same,
    /// Nothing at the station the ward would leave.
    Bare,
    /// The ward would rest upon a lesser one. **The puzzle's only real refusal.**
    Greater,
}

/// A course of wards, mid-solve.
///
/// Fields are private on `Ward`'s precedent: a `Course` has no secret in it the
/// way a ward's `code` does, but the invariant *every station descends* is one
/// that only [`haul`](Self::haul) can be trusted to keep, and a public
/// `stations` would put it within reach of anything.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Course {
    /// Each station's wards, bottom first. Magnitudes, where 1 is the least.
    stations: [Vec<usize>; 3],
    /// How many wards this course was drawn up with.
    height: usize,
    /// Hauls spent.
    hauls: u32,
}

impl Course {
    /// Draw up a course of `height` wards at [`START`].
    ///
    /// Clamped to `LEAST..=MOST` rather than trusted: the height comes from an
    /// integrity deficit and a jitter, and a course of nought wards would be
    /// solved the moment it was drawn.
    #[must_use]
    pub fn new(height: usize) -> Self {
        let height = height.clamp(LEAST, MOST);
        let mut stations = [const { Vec::new() }; 3];
        stations[START] = (1..=height).rev().collect();
        Self {
            stations,
            height,
            hauls: 0,
        }
    }

    /// How many wards this course has.
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Whether that is an odd number of them.
    #[must_use]
    pub const fn is_odd(&self) -> bool {
        !self.height.is_multiple_of(2)
    }

    /// Hauls spent so far.
    #[must_use]
    pub const fn hauls(&self) -> u32 {
        self.hauls
    }

    /// The magnitude of the ward at the top of `station`, if it holds one.
    #[must_use]
    pub fn top(&self, station: usize) -> Option<usize> {
        self.stations.get(station)?.last().copied()
    }

    /// How many wards are resting at `station`.
    #[must_use]
    pub fn depth(&self, station: usize) -> usize {
        self.stations.get(station).map_or(0, Vec::len)
    }

    /// Whether the whole course is assembled at [`GOAL`].
    #[must_use]
    pub const fn solved(&self) -> bool {
        self.stations[GOAL].len() == self.height
    }

    /// Carry the topmost ward from one station to another.
    ///
    /// Returns the magnitude that moved. The three refusals are exactly the
    /// three ways this can fail, and none of them changes the course.
    ///
    /// # Errors
    ///
    /// [`Refused::Same`], [`Refused::Bare`] or [`Refused::Greater`].
    pub fn haul(&mut self, from: usize, to: usize) -> Result<usize, Refused> {
        if from == to {
            return Err(Refused::Same);
        }
        let size = self.top(from).ok_or(Refused::Bare)?;
        if self.top(to).is_some_and(|resting| resting < size) {
            return Err(Refused::Greater);
        }
        self.stations[from].pop();
        self.stations[to].push(size);
        self.hauls = self.hauls.saturating_add(1);
        Ok(size)
    }

    /// Which way the one legal haul between two stations runs, if there is one.
    ///
    /// `None` only when both are empty, or when they are the same station.
    /// **This is the fact the whole domain rests on** and it is worth having as
    /// a function even though nothing in the game calls it: it is what the unit
    /// tests check the rule against, and what the cyclic solver below walks.
    #[must_use]
    pub fn between(&self, a: usize, b: usize) -> Option<(usize, usize)> {
        if a == b {
            return None;
        }
        match (self.top(a), self.top(b)) {
            (None, None) => None,
            (None, Some(_)) => Some((b, a)),
            (Some(_), None) => Some((a, b)),
            (Some(here), Some(there)) => Some(if here < there { (a, b) } else { (b, a) }),
        }
    }

    /// Assemble the whole course but the least ward, which waits at the spare.
    ///
    /// One `haul conduit barrier` from finished. See `execute::debug::COURSE`
    /// for why: the completion is the interesting half and 127 hauls is not a
    /// See-it line. **The hauls already spent are kept**, so the record the
    /// finish emits quotes a real number rather than one.
    #[cfg(debug_assertions)]
    pub fn give_away(&mut self) {
        const SPARE: usize = 1;
        self.stations = [const { Vec::new() }; 3];
        self.stations[GOAL] = (2..=self.height).rev().collect();
        self.stations[SPARE] = vec![1];
    }

    /// What a frontend needs to draw this.
    ///
    /// `tally` is the already-written line under the floor rule — see
    /// `orbs_render::Pylon::tally` for why a painter may not compose it.
    #[must_use]
    pub fn view(&self, integrity: u32, tally: String) -> orbs_render::Pylon {
        orbs_render::Pylon {
            stations: self.stations.clone(),
            tally,
            // The words, so the board can be typed from. `Board::sockets` is the
            // precedent and the reason: `orbs-render` may not depend on this
            // crate, and an unlabelled column is one a player counts along.
            names: STATIONS,
            height: self.height,
            hauls: self.hauls,
            integrity,
        }
    }

    /// Read a course out for a save.
    pub(crate) fn to_save(&self) -> crate::save::CourseSave {
        crate::save::CourseSave {
            stations: self.stations.iter().map(Clone::clone).collect(),
            height: self.height,
            hauls: self.hauls,
        }
    }

    /// Put one back.
    ///
    /// **Clamps rather than trusts**, on `Ward::from_save`'s precedent: a
    /// hand-edited save with eight wards at one station would otherwise draw off
    /// the end of a board that reserves seven rows.
    pub(crate) fn from_save(save: &crate::save::CourseSave) -> Option<Self> {
        let mut stations = [const { Vec::new() }; 3];
        for (station, saved) in stations.iter_mut().zip(save.stations.iter()) {
            station.clone_from(saved);
        }
        let height = stations.iter().map(Vec::len).sum::<usize>();

        // **Every way a course can be malformed, refused rather than repaired.**
        // The first version filtered magnitudes it did not like and then clamped
        // the count *up* to `LEAST` — so a save that lost a ward came back as a
        // three-ward course holding two, `solved()` could never be true, `finish`
        // never fired, and `muster` answered "a course is already drawn" for the
        // rest of the session. A jammed room is worse than an empty one, and
        // `adopt` simply raises no course when this says no.
        //
        // **`save.height` is what makes the check possible**, and it was dead
        // weight before: every ward stays in the course for its whole life, so
        // the count and the raised height can only disagree if the file is
        // wrong. That is precisely a checksum, and the field's own doc claimed
        // the two "cannot disagree" while nothing compared them.
        if height != save.height || !(LEAST..=MOST).contains(&height) {
            return None;
        }
        // ...and the magnitudes are exactly `1..=height`, each once.
        let mut every: Vec<usize> = stations.iter().flatten().copied().collect();
        every.sort_unstable();
        if every != (1..=height).collect::<Vec<_>>() {
            return None;
        }
        // **The invariant the type's doc says only `haul` can be trusted to
        // keep.** A hand-edited station that ascends is an illegal position, and
        // sorting it would be inventing a course the player never had.
        if stations
            .iter()
            .any(|station| station.windows(2).any(|pair| pair[0] <= pair[1]))
        {
            return None;
        }

        Some(Self {
            stations,
            height,
            hauls: save.hauls,
        })
    }
}

/// The pairs the cyclic solution moves between, in order, for a course of `height`.
///
/// **This is the algorithm the shipped `holding` spell writes**, and it is here
/// so a test can prove the spell's shape terminates without going through the
/// interpreter. Even courses rotate one way round the three stations and odd
/// courses the other; the third pair is the same either way.
///
/// See `dev_spells.toml` for the same three lines in the spell language, and
/// `tower::pylon`'s module doc for why the parity has to be readable.
#[must_use]
pub const fn cycle(height: usize) -> [(usize, usize); 3] {
    const SPARE: usize = 1;
    if height.is_multiple_of(2) {
        [(START, SPARE), (START, GOAL), (SPARE, GOAL)]
    } else {
        [(START, GOAL), (START, SPARE), (GOAL, SPARE)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cyclic solution, driven through the model. Returns the hauls it took.
    fn solve(course: &mut Course) -> u32 {
        let pairs = cycle(course.height());
        // Bounded so a broken cycle fails as a wrong number rather than as a
        // hang. `2^MOST` is 128 and three pairs a pass, with slack.
        for _ in 0..1024 {
            for (a, b) in pairs {
                if course.solved() {
                    return course.hauls();
                }
                if let Some((from, to)) = course.between(a, b) {
                    course.haul(from, to).expect("the legal haul is legal");
                }
            }
        }
        panic!("the cycle never solved a course of {}", course.height());
    }

    #[test]
    fn a_haul_between_two_stations_is_unique() {
        // **The fact the whole domain rests on.** Checked over every reachable
        // position of a four-ward course rather than argued: for each pair, at
        // most one of the two directions is legal, and exactly one is unless
        // both stations are empty.
        let mut course = Course::new(4);
        let pairs = cycle(4);
        for _ in 0..64 {
            for (a, b) in [(0, 1), (0, 2), (1, 2)] {
                let forward = course.clone().haul(a, b).is_ok();
                let backward = course.clone().haul(b, a).is_ok();
                assert!(
                    !(forward && backward),
                    "both directions legal between {a} and {b}: {course:?}",
                );
                let empty = course.top(a).is_none() && course.top(b).is_none();
                assert_eq!(
                    forward || backward,
                    !empty,
                    "neither direction legal between {a} and {b}, and one is occupied: {course:?}",
                );
            }
            // Walk on, so the sweep sees positions rather than one position.
            for (a, b) in pairs {
                if let Some((from, to)) = course.between(a, b) {
                    course.haul(from, to).expect("the legal haul is legal");
                }
            }
        }
    }

    #[test]
    fn a_greater_ward_never_rests_upon_a_lesser() {
        // The one rule, checked as an invariant of the model rather than of any
        // one call: whatever a solve does, every station descends bottom to top
        // the whole way through.
        for height in LEAST..=MOST {
            let mut course = Course::new(height);
            let pairs = cycle(height);
            for _ in 0..1024 {
                if course.solved() {
                    break;
                }
                for (a, b) in pairs {
                    if let Some((from, to)) = course.between(a, b) {
                        course.haul(from, to).expect("the legal haul is legal");
                    }
                    for station in 0..3 {
                        let stack = &course.stations[station];
                        assert!(
                            stack.windows(2).all(|pair| pair[0] > pair[1]),
                            "station {station} is out of order at {stack:?}",
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_cyclic_solution_finishes_every_course_in_exactly_two_to_the_n_less_one() {
        // **The number is the proof.** The cyclic algorithm is optimal, so a
        // count that came out high would mean the pairs are in the wrong order
        // for that parity — the failure mode that is otherwise invisible,
        // because a wrong cycle still solves, just slowly and sometimes at the
        // wrong station.
        for height in LEAST..=MOST {
            let mut course = Course::new(height);
            let hauls = solve(&mut course);
            assert!(course.solved(), "a course of {height} never finished");
            assert_eq!(
                hauls,
                (1u32 << height) - 1,
                "a course of {height} took {hauls} hauls, not the optimal count",
            );
        }
    }

    #[test]
    fn the_parity_is_what_decides_the_cycle_and_getting_it_wrong_is_visible() {
        // Why `ODD` is published at all. Run a course under the *other* parity's
        // cycle: it still terminates, and it terminates at the wrong station —
        // so a spell that could not read the parity would quietly assemble the
        // tower's defences in the conduit and report success.
        let mut course = Course::new(4);
        let wrong = cycle(5);
        for _ in 0..1024 {
            if course.depth(GOAL) == course.height() || course.depth(1) == course.height() {
                break;
            }
            for (a, b) in wrong {
                if let Some((from, to)) = course.between(a, b) {
                    course.haul(from, to).expect("the legal haul is legal");
                }
            }
        }
        assert!(
            !course.solved(),
            "the wrong cycle solved a course, so the parity reading buys nothing",
        );
        assert_eq!(course.depth(1), 4, "it did not finish on the spare either");
    }

    #[test]
    fn a_fresh_course_waits_at_the_wellspring_and_is_not_solved() {
        let course = Course::new(5);
        assert_eq!(course.depth(START), 5);
        assert_eq!(course.depth(GOAL), 0);
        assert_eq!(course.top(START), Some(1), "the least ward is on top");
        assert!(!course.solved());
        assert!(course.is_odd());
    }

    #[test]
    fn the_three_refusals_are_the_three_ways_a_haul_can_fail() {
        let mut course = Course::new(3);
        assert_eq!(course.haul(START, START), Err(Refused::Same));
        assert_eq!(course.haul(1, GOAL), Err(Refused::Bare));
        course.haul(START, GOAL).expect("the smallest ward moves");
        assert_eq!(
            course.haul(START, GOAL),
            Err(Refused::Greater),
            "a two onto a one was allowed",
        );
        // ...and none of them cost a haul or moved anything.
        assert_eq!(course.hauls(), 1);
        assert_eq!(course.depth(GOAL), 1);
    }

    #[test]
    fn a_height_outside_the_band_is_clamped_rather_than_trusted() {
        assert_eq!(Course::new(0).height(), LEAST);
        assert_eq!(Course::new(99).height(), MOST);
    }

    #[test]
    fn the_line_under_the_floor_rule_fits_the_board_it_is_drawn_on() {
        // **This shipped truncated.** `pylon_tally` and `pylon_spoken` were one
        // key, and the sentence a reader wants — which names the defences too —
        // is longer than the board is wide, so the picture drew
        // `4 wards, 3 hauled, the wall` and stopped. Nothing failed: `centred`
        // truncates by design, because a row wider than the board would paint
        // over the transcript's border.
        //
        // The two are separate keys now, and this is what keeps the drawn one
        // short. `hauls` is unbounded — a player who flails can spend thousands
        // — so the check is against a number wider than any real solve rather
        // than against the optimal one.
        let prose = crate::content::Prose::builtin();
        let line = prose.line(
            "pylon_tally",
            &[("quantity", &MOST.to_string()), ("name", "99999")],
        );
        assert!(
            line.chars().count() <= orbs_render::Pylon::COLS as usize,
            "the tally is {} cells against a board {} wide: {line:?}",
            line.chars().count(),
            orbs_render::Pylon::COLS,
        );
    }

    #[test]
    fn a_board_has_room_for_the_tallest_course() {
        // Checked from this side because `orbs-render` may not depend on this
        // crate and so cannot see `MOST`. A painter reserving fewer rows than
        // the world can raise would clip the bottom of a neglected tower's
        // course — the state a player most needs to read.
        assert_eq!(MOST, orbs_render::TALLEST);
    }

    #[test]
    fn a_course_survives_the_round_trip_through_a_save() {
        let mut course = Course::new(5);
        for (a, b) in cycle(5) {
            if let Some((from, to)) = course.between(a, b) {
                course.haul(from, to).expect("the legal haul is legal");
            }
        }
        let there_and_back = Course::from_save(&course.to_save());
        assert_eq!(there_and_back, Some(course));
    }

    #[test]
    fn a_malformed_course_is_refused_rather_than_repaired() {
        // **Each of these came back as a *jammed* course before.** The count was
        // clamped up to `LEAST`, so a save missing a ward gave a three-ward
        // course holding two: `solved()` unreachable, `finish` never fired, and
        // `muster` refused for the rest of the session. Refusing raises no
        // course at all, and the player can simply muster.
        let whole = || Course::new(4).to_save();

        // A lost ward: three magnitudes where the height says four.
        let mut missing = whole();
        missing.stations = vec![vec![4, 3, 2], Vec::new(), Vec::new()];
        assert_eq!(
            Course::from_save(&missing),
            None,
            "a lost ward was tolerated"
        );

        // The checksum itself: the wards are fine, the raised height is not.
        let mut mismatched = whole();
        mismatched.height = 5;
        assert_eq!(
            Course::from_save(&mismatched),
            None,
            "the height and the wards disagreed and it was let through",
        );

        // Four wards, but not the four a course is made of.
        let mut repeated = whole();
        repeated.stations = vec![vec![4, 3, 2, 2], Vec::new(), Vec::new()];
        assert_eq!(
            Course::from_save(&repeated),
            None,
            "a repeated magnitude was tolerated",
        );

        // An illegal *position*: a station that ascends. Sorting it would be
        // inventing a course the player never had.
        let mut upside_down = whole();
        upside_down.stations = vec![vec![1, 2, 3, 4], Vec::new(), Vec::new()];
        assert_eq!(
            Course::from_save(&upside_down),
            None,
            "a greater ward resting on a lesser survived a reload",
        );

        // ...and the untouched one still loads.
        assert!(Course::from_save(&whole()).is_some());
    }

    #[test]
    fn the_names_are_three_distinct_stations_and_the_course_runs_between_the_ends() {
        assert_eq!(STATIONS.len(), 3);
        assert_ne!(START, GOAL);
        for (index, name) in STATIONS.iter().enumerate() {
            assert_eq!(station_of(name), Some(index));
            assert_eq!(
                station_of(&format!("/tower/sanctum/{name}")),
                Some(index),
                "a resolved place arrives as a path and must still name a station",
            );
        }
        assert_eq!(station_of("laboratory"), None);
    }
}
