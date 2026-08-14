//! `divine` and `follow` — opening a labyrinth and threading it (§10, §19).
//!
//! # What `divine` became
//!
//! It held the production slot for twelve ticks, consumed nothing, produced
//! nothing, and could be run on the same sigil for ever — §10's *"commands with
//! a duration and no decision content"*, which is the thing that column exists
//! to prevent. It now opens a maze on the lectern.
//!
//! **Opening takes no slot.** A maze is walked for hundreds of ticks, and
//! `CAPACITY` is 1 with `PATIENCE` at 120 — a solver holding the tower's one
//! production slot would starve every other spell into `spell_gave_up`, which is
//! precisely the bind-it-and-go-and-brew case the whole design sells. Reading is
//! not a *run*; the same argument `start` makes for the athanor at its
//! `HeatSource` branch.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::{Intent, Verb};
use crate::rng::{RngStream, Rngs};
use crate::session::Scrollback;
use crate::tower::{self, Cwd, Maze, Square, Way};

/// How wide a maze is, in cells.
///
/// **Sixteen, and it is deliberately larger than the smallest pane can draw.**
/// It was seven, chosen so the picture fitted everywhere; the maze that produced
/// was small enough to read at a glance and over in a few dozen steps. A
/// labyrinth you can take in whole is not one.
///
/// See [`HEIGHT`], which is *not* this: character cells are twice as tall as
/// they are wide, so a maze with equal counts draws as a portrait rectangle.
const WIDTH: usize = 16;

/// How **tall** a maze is, in cells, and it is not the width.
///
/// **A character cell is 8×16 pixels — twice as tall as it is wide** — so a grid
/// square in characters is a tall rectangle on screen, and a maze with equal
/// counts draws as a portrait one. Sixteen by eleven is 33×23 characters — 264
/// by 368 pixels, against the 264 by 528 an equal-count maze would draw.
///
/// **It has come down twice, and the second time was clipping rather than
/// taste.** Whatever a pane cannot fit is shown as a window that pans (see
/// `orbs_render::maze::viewport`), which is right at a small grid and reads as
/// *the bottom is cut off* at a large one. The block a map wants is
/// `2 × HEIGHT + 3` rows — 25 here — so a session pane with fewer than that
/// shows part of the maze rather than all of it.
const HEIGHT: usize = 11;

/// How wide the *grid* is, in squares: a wall each side of every cell.
///
/// `2 × WIDTH + 1`, and the picture is one character per square.
const SPAN_X: usize = 2 * WIDTH + 1;

/// How tall the grid is, in squares. `2 × HEIGHT + 1`.
const SPAN_Y: usize = 2 * HEIGHT + 1;

/// `research` — open a labyrinth on the lectern.
pub(super) fn research(intent: &Intent, world: &mut World) {
    let _ = intent;
    let Some(lectern) = lectern(world) else {
        say(world, Verb::Research, "research_nowhere", &[], Role::Danger);
        return;
    };
    if world.get::<Maze>(lectern).is_some() {
        say(world, Verb::Research, "research_already", &[], Role::Cost);
        return;
    }

    let maze = generate(world);
    world.entity_mut(lectern).insert(maze);
    refresh(world);
    say(world, Verb::Research, "research_opens", &[], Role::Success);
}

/// `follow <way>` — move the reading one cell.
pub(super) fn follow(intent: &Intent, world: &mut World) {
    let Some(way) = intent
        .arguments
        .first()
        .and_then(|argument| named(&argument.value))
    else {
        say(world, Verb::Follow, "follow_nowhere", &[], Role::Danger);
        return;
    };
    tread(world, way);
}

/// Move the reading one cell, and say what happened.
///
/// **The one body, called from two clocks.** `follow` reaches it at the start of
/// a tick like every other command; `Sim::walk` reaches it the instant an arrow
/// is pressed. A second copy for the immediate path is exactly the thing rule 2
/// forbids a frontend from having, and it would be no better inside the sim —
/// the two would disagree about a wall, or about what a solve is worth, and only
/// one of them would be tested.
pub(crate) fn tread(world: &mut World, way: Way) {
    let Some(lectern) = lectern(world) else {
        say(world, Verb::Follow, "research_nowhere", &[], Role::Danger);
        return;
    };
    let Some(mut maze) = world.get_mut::<Maze>(lectern) else {
        say(world, Verb::Follow, "follow_unopened", &[], Role::Cost);
        return;
    };

    if !maze.tread(way) {
        // **A wall costs the step and nothing else.** §7's *"destruction is a
        // tool, not a trap"* applies to a wrong turn too: a solver that walked
        // into a wall and lost the maze would make every unmapped fragment a
        // gamble, and a bad rule should be *slow*, not ruinous.
        say(
            world,
            Verb::Follow,
            "follow_wall",
            &[("name", way.word())],
            Role::Cost,
        );
        return;
    }
    if maze.solved() {
        // **Removed first, then refreshed.** The other order leaves the four
        // ways holding the solved maze's last readings for ever: `survey north`
        // answers `passage` with no labyrinth open, and a bound solver reads
        // them, fires its `follow` tier every lap, and is told *"research first"*
        // for the rest of its `repeat`. `pipeline::stop` has always had this
        // order; this path did not.
        world.entity_mut(lectern).remove::<Maze>();
        refresh(world);
        yield_fragment(world, lectern);
        return;
    }
    refresh(world);
    // **Logged, not drawn.** The map has just moved the reading, so a line
    // saying so is the same fact twice — and a maze is hundreds of steps, so the
    // second copy is what scrolls the player's own typing off the pane. `peruse
    // archive.log` still has every one of them; see `RecordBuilder::quiet`.
    //
    // The wall refusal above is *not* quiet, deliberately: nothing moves, so the
    // map reports nothing, and a press that did nothing has to say so (§6).
    quietly(
        world,
        Verb::Follow,
        "follow_done",
        &[("name", way.word())],
        Role::Normal,
    );
}

/// Write what the maze can see into the four readings.
///
/// **Mutated in place, never respawned.** `heat.rs` records that spawning across
/// ticks issues `NodeId`s at a rate depending on how the ticks were consumed, so
/// a live-watched run and a `meditate`-collapsed one produce different worlds
/// from one seed. Despawning and re-spawning twenty nodes a step would be that
/// hazard with more nodes; the four ways are raised once by `build` and only
/// their contents change.
pub fn refresh(world: &mut World) {
    let Some(lectern) = lectern(world) else {
        return;
    };
    let readings: Vec<(Way, Option<&'static str>, bool)> = {
        let maze = world.get::<Maze>(lectern);
        Way::ALL
            .into_iter()
            .map(|way| {
                (
                    way,
                    maze.map(|maze| maze.reading(way).word()),
                    maze.is_some_and(|maze| maze.came() == Some(way)),
                )
            })
            .collect()
    };

    for (way, word, came) in readings {
        let Some(node) = find_reading(world, way) else {
            continue;
        };
        for held in tower::children_of(world, node) {
            world.entity_mut(held).despawn();
        }
        if let Some(word) = word {
            tower::raise_reading(world, node, word);
        }
        // **A second child, not a replacement.** A way can be `walked` and the
        // way you came at once, and a solver asks both — see `maze::BACK`.
        if came {
            tower::raise_reading(world, node, tower::maze::BACK);
        }
    }
}

/// The maze is solved: a fragment, and what the work was worth.
fn yield_fragment(world: &mut World, lectern: Entity) {
    tower::give(
        world,
        lectern,
        FRAGMENT,
        crate::parser::NounKind::Fragment,
        1,
    );

    let earned = tower::worth(world, "lectern");
    let message = world
        .resource::<Prose>()
        .line("research_solved", &[("name", FRAGMENT)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Research.canonical())
        .text(FieldName::Detail, FRAGMENT)
        .text(FieldName::At, "lectern")
        .count(FieldName::Quantity, earned)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
    tower::credit(world, earned);
}

/// What a solved maze yields. Four make a scroll.
///
/// # One name, and no roll
///
/// It was four — `shard-of-dawn`, `-noon`, `-dusk`, `-night` — drawn uniformly,
/// and two things were wrong with that. **Collecting a set was coupon-collector
/// attrition**: 4·(1 + ½ + ⅓ + ¼) ≈ 8.3 solves on average for one scroll, with
/// no decision anywhere in it, because you could not aim for the one you lacked
/// and a maze whose shard you already held was worth as much as one you did not.
/// That is §10's objection to the *old* `divine` — a duration with no decision
/// content — reappearing one level up, in the collection loop instead of the
/// command. And **nothing in the game ever said what one was**: no prose, no
/// `recall` topic, four invented names standing in for a design decision.
///
/// Specific fragments for specific spells is the intended shape and will want
/// distinct names again. Until those spells exist, one name is the honest
/// placeholder — and it takes the `Yield` roll out of the archive entirely.
const FRAGMENT: &str = "fragment";

/// A maze, from the archive's own stream.
///
/// **Built in one go.** A maze that grew as it was walked would issue `NodeId`s
/// at a rate depending on how the ticks were consumed — `heat.rs`'s hazard.
fn generate(world: &mut World) -> Maze {
    let mut rngs = world.resource_mut::<Rngs>();
    let rng = rngs.stream(RngStream::Archive);
    let cells = WIDTH * HEIGHT;

    // **Carved as cells, laid out as squares.** The spanning tree is easiest to
    // reason about one cell at a time — every cell reachable, exactly one path
    // between any two, no loops — but the *grid* has to hold the walls, or a step
    // is two characters on screen and reads as moving two spaces at a time. So
    // the tree is carved here and stamped into the square grid below.
    let mut grid = vec![
        Square {
            wall: true,
            marks: 0,
        };
        SPAN_X * SPAN_Y
    ];
    // Every cell's own square is floor.
    for index in 0..cells {
        grid[square_of(index)].wall = false;
    }

    // **Randomised Prim's, not a recursive backtracker.** A backtracker carves
    // one long path and only turns when it has to, so its mazes are a few very
    // long corridors with the odd stub — easy to read at a glance and easy to
    // walk. Prim's grows the maze outward from everywhere at once, which gives
    // short passages, frequent junctions and many small dead ends: the tight,
    // busy texture a labyrinth is supposed to have.
    //
    // The result is still a spanning tree — every cell reachable, exactly one
    // path between any two, no loops — which is the floor Trémaux is measured
    // against and what `tests/solver.rs` sweeps.
    // **Seeded at a random cell, not at the top-left one.** Prim's grows
    // outward from its seed, so the seed is the centre of a radial structure —
    // and growing every maze from cell 0 put that centre in the same place as
    // the start, every time. The tree was random and the *walk* was not: it
    // always began by unwinding the oldest, straightest part of the maze.
    // Decoupling the seed from both endpoints is what makes two mazes with the
    // same start and exit still read differently.
    let seed = rand::Rng::random_range(rng, 0..cells);
    let mut seen = vec![false; cells];
    seen[seed] = true;
    // Cells adjacent to the maze but not yet in it, each paired with the cell it
    // would be joined to. Duplicates are fine: one is taken and the rest fail
    // the `seen` check and are dropped.
    let mut frontier: Vec<(usize, usize)> = (0..4)
        .filter_map(|index| neighbour(seed, index).map(|next| (seed, next)))
        .collect();

    while !frontier.is_empty() {
        let pick = rand::Rng::random_range(rng, 0..frontier.len());
        let (from, next) = frontier.swap_remove(pick);
        if seen[next] {
            continue;
        }
        seen[next] = true;
        // The square *between* the two cells becomes floor: that is the wall
        // coming down, and it is somewhere the reading now stands on its way.
        //
        // Two adjacent cells sit exactly two squares apart — `2` across or
        // `2 * span` down — so their indices always average to the square
        // between them, with nothing to round.
        let here = square_of(from);
        let there = square_of(next);
        grid[(here + there) / 2].wall = false;

        frontier.extend(
            (0..4)
                .filter_map(|index| neighbour(next, index))
                .filter(|beyond| !seen[*beyond])
                .map(|beyond| (next, beyond)),
        );
    }

    // The reading starts in one of the four corners, uniformly — it was always
    // the top-left — and the way out is drawn against a weighting that leans on
    // the corner diagonally opposite it.
    let start = CORNERS[rand::Rng::random_range(rng, 0..CORNERS.len())];
    let exit = drawn_away_from(rng, start);
    Maze::new(grid, SPAN_X, square_of(cell_at(start)), square_of(exit))
}

/// The four corner cells, in a fixed order.
///
/// Fixed because the *order* is part of the replay: a roll into this table has
/// to mean the same corner for a given seed for ever.
const CORNERS: [(usize, usize); 4] = [
    (0, 0),
    (WIDTH - 1, 0),
    (0, HEIGHT - 1),
    (WIDTH - 1, HEIGHT - 1),
];

/// How hard the way out is pulled toward the corner opposite the reading.
///
/// The weight of a cell is `(reach - away + 1)^PULL`, where `away` is its
/// Manhattan distance from that corner — so the corner itself is worth
/// `26³ = 17576` and a cell beside the reading is worth `1`.
///
/// **Cubed, and the exponent was measured rather than liked.** Over this 16×11
/// grid it puts the way out within 8 steps of the opposing corner 63% of the
/// time, with a median of 7 and a 13% tail past 12:
///
/// | `PULL` | within 4 | within 8 | median |
/// |---|---|---|---|
/// | 2 | 22% | 52% | 8 |
/// | **3** | **29%** | **63%** | **7** |
/// | 4 | 37% | 72% | 6 |
/// | 6 | 51% | 85% | 4 |
///
/// Two is barely a lean — the exit lands mid-maze as often as far from it, which
/// is *"the errand is always the same length"* wearing different clothes. Six is
/// the old fixed corner with extra steps. Three keeps the walk long enough to be
/// worth taking while leaving the destination genuinely unpredictable.
///
/// It is a **weighting and not a rule**, which is the whole point: every cell
/// the maze reaches can win.
const PULL: u32 = 3;

/// A cell's index from its coordinates.
const fn cell_at((x, y): (usize, usize)) -> usize {
    y * WIDTH + x
}

/// A cell drawn at random, weighted toward the corner opposite `from`.
///
/// **Every cell can win.** The old exit was the bottom-right square, always, and
/// paired with a top-left start it made one journey the player took over and
/// over — the maze changed and the errand did not. This keeps the errand long,
/// because a short one is not worth walking, without making it identical.
///
/// Integer-only, like everything else the sim scores with: a weighting resolved
/// in floats would make the draw depend on rounding, and §3 requires a seed to
/// replay to the same labyrinth on every machine.
fn drawn_away_from(rng: &mut rand_chacha::ChaCha8Rng, from: (usize, usize)) -> usize {
    let toward = (WIDTH - 1 - from.0, HEIGHT - 1 - from.1);
    let reach = (WIDTH - 1) + (HEIGHT - 1);

    // Weight 0 for the cell the reading is standing on, so the way out is never
    // underfoot. Excluded here rather than rejected afterwards, which would be a
    // loop that can in principle spin.
    let weight = |cell: usize| -> u64 {
        let (x, y) = (cell % WIDTH, cell / WIDTH);
        if (x, y) == from {
            return 0;
        }
        let away = toward.0.abs_diff(x) + toward.1.abs_diff(y);
        u64::try_from(reach - away + 1).unwrap_or(1).pow(PULL)
    };

    let cells = WIDTH * HEIGHT;
    let total: u64 = (0..cells).map(weight).sum();
    let mut roll = rand::Rng::random_range(rng, 0..total.max(1));
    for cell in 0..cells {
        let share = weight(cell);
        if roll < share {
            return cell;
        }
        roll -= share;
    }
    // Unreachable while `total` is the sum of the same weights, and the corner
    // is the honest answer if the arithmetic ever stops agreeing with itself.
    cell_at(toward)
}

/// Where cell `index` lives in the square grid.
const fn square_of(index: usize) -> usize {
    (2 * (index / WIDTH) + 1) * SPAN_X + 2 * (index % WIDTH) + 1
}

/// The cell `index` steps to from `at`, if the grid has one.
fn neighbour(at: usize, index: usize) -> Option<usize> {
    let (x, y) = (at % WIDTH, at / WIDTH);
    let (x, y) = match index {
        0 => (x, y.checked_sub(1)?),
        1 => (x + 1, y),
        2 => (x, y + 1),
        _ => (x.checked_sub(1)?, y),
    };
    (x < WIDTH && y < HEIGHT).then_some(y * WIDTH + x)
}

/// The way a player named.
fn named(word: &str) -> Option<Way> {
    let leaf = crate::parser::leaf(word);
    Way::ALL.into_iter().find(|way| way.word() == leaf)
}

/// The lectern, if the player is standing where one is.
///
/// **Reads `Cwd`, which is what makes `Sim::labyrinth` honest.** A frontend
/// asking for the map gets one only where the player could `survey` the ways
/// themselves — so the picture cannot outrun the readings by following the
/// player out of the room.
pub(crate) fn lectern(world: &World) -> Option<Entity> {
    let cwd = world.resource::<Cwd>().0;
    tower::children_of(world, cwd).into_iter().find(|node| {
        world
            .get::<tower::Operation>(*node)
            .is_some_and(|operation| operation.0 == Verb::Research)
    })
}

/// One of the four readings, by way.
fn find_reading(world: &World, way: Way) -> Option<Entity> {
    let cwd = world.resource::<Cwd>().0;
    tower::children_of(world, cwd).into_iter().find(|node| {
        world.get::<tower::Reading>(*node).is_some()
            && world
                .get::<tower::Name>(*node)
                .is_some_and(|name| name.0 == way.word())
    })
}

/// One authored line about the reading.
///
/// **The verb is passed in, and filing everything under `divine` was a real
/// defect.** Both verbs in this module used one `say`, so every one of `follow`'s
/// completions and refusals went into the log named `divine` — and `sift follow
/// orb.log` returned the echo of the typed line and *not* what happened, for the
/// archive's most-used word. Rule 4 makes the record the source and every view a
/// reading of it; a record filed under the wrong verb is that source lying.
fn say(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    let message = world.resource::<Prose>().line(key, args);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Message, &message)
        .role(role)
        .finish();
}

/// [`say`], into the log only — for a step the map has already drawn.
///
/// The two differ by one call and are kept apart so the *choice* is visible at
/// the emit site: a reader of `tread` can see which of its three outcomes the
/// pane shows without going and looking at a flag somewhere else.
///
/// # It files the record, which [`say`] never had to
///
/// **A quiet record that no log holds is not logged at all**, and §3 forbids
/// that outright. `files::in_domain` decides a domain's log by matching
/// `Source`, `Path` or `Origin` against the domain and everything standing in
/// it — and the archive's completions set none of the three, so `peruse
/// archive.log` was empty after a walk long before this made the steps quiet.
///
/// Drawn records got away with it because the pane is a second surface. This one
/// has no second surface, so the lectern goes in `Source` — an instrument, as
/// `work::produce` and `work::slot` already file theirs — and the log the player
/// is told to read actually holds the steps.
fn quietly(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    let message = world.resource::<Prose>().line(key, args);
    let at = lectern(world)
        .and_then(|lectern| world.get::<tower::Name>(lectern))
        .map(|name| name.0.clone());

    let mut records = world.resource_mut::<Scrollback>();
    let mut builder = records
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Message, &message)
        .role(role)
        .quiet();
    if let Some(at) = &at {
        builder = builder.text(FieldName::Source, at);
    }
    builder.finish();
}

#[cfg(test)]
mod tests {
    use crate::Sim;

    /// The reading and the way out of one seed's maze, as **cell** coordinates.
    fn ends(seed: u64) -> ((usize, usize), (usize, usize)) {
        let mut sim = Sim::new(seed);
        sim.submit("attend archive");
        sim.step();
        sim.submit("research");
        sim.step();
        let maze = sim.labyrinth().expect("research opens a labyrinth");

        // Square indices back to cells: `square_of` puts cell (x, y) at square
        // (2y + 1, 2x + 1), so the inverse halves each axis.
        let cell = |index: usize| {
            let span = usize::from(maze.width);
            ((index % span - 1) / 2, (index / span - 1) / 2)
        };
        (cell(maze.at), cell(maze.exit))
    }

    /// Enough seeds that a corner missed by chance is vanishingly unlikely: four
    /// corners over 120 draws, so each is expected 30 times.
    const SWEEP: u64 = 120;

    #[test]
    fn the_reading_starts_in_every_corner_across_seeds() {
        // It was the top-left one, always. Four corners uniformly is the first
        // half of making two labyrinths feel like two errands.
        let mut corners = std::collections::BTreeSet::new();
        for seed in 1..=SWEEP {
            let (start, _) = ends(seed);
            corners.insert(start);
        }
        assert_eq!(
            corners,
            super::CORNERS.iter().copied().collect(),
            "the reading does not reach every corner",
        );
    }

    #[test]
    fn the_way_out_is_not_one_fixed_square() {
        // The complaint, directly: the exit used to be the bottom-right cell on
        // every seed, so the journey was the same however the walls fell.
        let exits: std::collections::BTreeSet<_> = (1..=SWEEP).map(|seed| ends(seed).1).collect();
        assert!(
            exits.len() > 20,
            "the way out only lands in {} places across {SWEEP} seeds",
            exits.len(),
        );
    }

    #[test]
    fn the_way_out_leans_toward_the_opposing_corner_without_being_pinned_to_it() {
        // **A weighting, not a rule**, and both halves are asserted because
        // either one alone is a plausible way to "fix" this into something
        // worse: pull it tighter and the exit is the corner again, loosen it and
        // the errand is sometimes three steps long.
        //
        // The bounds come from the weighting's own arithmetic rather than from
        // taste. At `PULL = 3` over a 16x11 grid the exact figures are 63%
        // within 8 of the opposing corner and 13% beyond 12; the margins here
        // are wide enough that 120 seeds will not trip them by luck, and narrow
        // enough to catch `PULL` being changed without meaning to.
        let reach = (super::WIDTH - 1) + (super::HEIGHT - 1);
        let (mut near, mut roaming) = (0, 0);
        for seed in 1..=SWEEP {
            let (start, exit) = ends(seed);
            assert_ne!(exit, start, "seed {seed} put the way out underfoot");

            let opposite = (super::WIDTH - 1 - start.0, super::HEIGHT - 1 - start.1);
            let away = opposite.0.abs_diff(exit.0) + opposite.1.abs_diff(exit.1);
            assert!(away <= reach, "seed {seed} left the grid");

            if away <= 8 {
                near += 1;
            }
            if away > 12 {
                roaming += 1;
            }
        }
        let sweep = usize::try_from(SWEEP).expect("small");
        assert!(
            near * 100 >= sweep * 50,
            "only {near} of {SWEEP} exits landed near the opposing corner — the pull is too weak",
        );
        assert!(
            roaming > 0,
            "every exit clustered on the opposing corner — this is a weighting, not a rule",
        );
    }

    #[test]
    fn a_step_is_logged_without_being_drawn() {
        // **The map is the report.** A maze is hundreds of steps, and a line per
        // step restating what the picture just drew is the same fact twice — the
        // copy that scrolled the player's own typing off the pane.
        //
        // Both halves are asserted. §3 forbids unlogged output, so "quiet" has
        // to mean *not drawn*, never *not emitted*: dropping the record would
        // pass a test that only checked the pane.
        let mut sim = Sim::new(3);
        sim.submit("attend archive");
        sim.step();
        sim.submit("research");
        sim.step();

        // One of the four lands, whichever way the walls fall on this seed.
        for way in ["north", "east", "south", "west"] {
            sim.submit(&format!("follow {way}"));
            sim.step();
        }

        let steps = |quiet_only: bool| {
            sim.scrollback()
                .records()
                .iter()
                .filter(|record| !quiet_only || record.is_quiet())
                .filter_map(|record| record.field(orbs_render::FieldName::Message))
                .filter(|value| value.with_str(|text| text.contains("the reading goes")))
                .count()
        };
        let walked = steps(false);
        assert!(walked > 0, "no way was open on this seed — pick another");
        assert_eq!(walked, steps(true), "a step reached the transcript");

        assert!(
            sim.scrollback()
                .records()
                .drawn()
                .filter_map(|record| record.field(orbs_render::FieldName::Message))
                .all(|value| value.with_str(|text| !text.contains("the reading goes"))),
            "`drawn` still yields the steps it is supposed to filter",
        );
    }

    #[test]
    fn a_wall_is_still_drawn() {
        // The other side of the rule, and the reason it is not "make `follow`
        // silent". A refusal moves nothing, so the map reports nothing at all —
        // a press that did nothing has to say so (§6).
        let mut sim = Sim::new(3);
        sim.submit("attend archive");
        sim.step();
        sim.submit("research");
        sim.step();
        for way in ["north", "east", "south", "west"] {
            sim.submit(&format!("follow {way}"));
            sim.step();
        }

        let walled = sim
            .scrollback()
            .records()
            .drawn()
            .filter_map(|record| record.field(orbs_render::FieldName::Message))
            .filter(|value| value.with_str(|text| text.contains("no way")))
            .count();
        assert!(walled > 0, "a wall was hushed along with the steps");
    }

    #[test]
    fn the_same_seed_still_draws_the_same_labyrinth() {
        // Three new rolls went into the archive's stream. §3 makes replay
        // architectural, so the property that matters is not *which* maze a seed
        // draws but that it keeps drawing that one.
        for seed in [1, 7, 99] {
            assert_eq!(ends(seed), ends(seed), "seed {seed} is not reproducible");
        }
    }
}
