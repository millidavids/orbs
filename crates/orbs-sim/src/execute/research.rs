//! `divine` and `follow` — opening the stacks and threading them (§10, §19).
//!
//! A maze in the stacks, where `divine` was a duration with no decision in it
//! (§10). The stacks and the lectern are two fixtures, so neither `stop`
//! reaches the other.
//!
//! Opening takes no slot: a maze is walked for hundreds of ticks, and holding
//! the one production slot would starve every other spell — the athanor's
//! argument at `start`'s `HeatSource` branch.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::{Intent, Verb};
use crate::rng::{RngStream, Rngs};
use crate::session::Scrollback;
use crate::tower::{self, Maze, Sense, Square, Way};

/// How wide a maze is, in cells.
///
/// Larger than the smallest pane can draw: stacks you can take in whole are
/// not stacks. Not [`HEIGHT`], since a cell is twice as tall as it is wide.
const WIDTH: usize = 16;

/// How tall a maze is, in cells, and it is not the width.
///
/// Sixteen by eleven draws near square on 8×16 cells. A map wants
/// `2 × HEIGHT + 3` rows; a smaller pane pans (`orbs_render::maze::viewport`),
/// which reads as cut off when the grid is much larger.
const HEIGHT: usize = 11;

/// How wide the *grid* is, in squares: a wall each side of every cell.
///
/// `2 × WIDTH + 1`, and the picture is one character per square.
const SPAN_X: usize = 2 * WIDTH + 1;

/// How tall the grid is, in squares. `2 × HEIGHT + 1`.
const SPAN_Y: usize = 2 * HEIGHT + 1;

/// `research` — open a way into the stacks.
pub(super) fn research(intent: &Intent, world: &mut World) {
    let _ = intent;
    let Some(stacks) = stacks(world) else {
        say(world, Verb::Research, "research_nowhere", &[], Role::Danger);
        return;
    };
    if world.get::<Maze>(stacks).is_some() {
        say(world, Verb::Research, "research_already", &[], Role::Cost);
        return;
    }

    let maze = generate(world);
    world.entity_mut(stacks).insert(maze);
    publish(world, stacks);
    say(world, Verb::Research, "research_opens", &[], Role::Success);
    // A gleaning supply open sets it gathering before a step is taken.
    if let Some(spoils) = super::scroll::gathering(world) {
        super::scroll::gather(world, stacks, spoils);
    }
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
/// One body, two clocks: `follow` at the start of a tick, `Sim::walk` the
/// instant an arrow is pressed.
pub(crate) fn tread(world: &mut World, way: Way) {
    let Some(stacks) = stacks(world) else {
        say(world, Verb::Follow, "research_nowhere", &[], Role::Danger);
        return;
    };
    let Some(mut maze) = world.get_mut::<Maze>(stacks) else {
        say(world, Verb::Follow, "follow_unopened", &[], Role::Cost);
        return;
    };

    let spoils_before = maze.spoils().len();
    if !maze.tread(way) {
        // A wall costs the step and nothing else: a bad rule should be slow,
        // not ruinous (§7).
        say(
            world,
            Verb::Follow,
            "follow_wall",
            &[("name", way.word())],
            Role::Cost,
        );
        return;
    }
    // Asked of the maze, not counted by the caller: `Maze::tread` collects the
    // spoil the reading arrives on, so a typed `follow` and an arrow through
    // `Sim::walk` pick up on one rule rather than two.
    let gathered = spoils_before > maze.spoils().len();
    let solved = maze.solved();

    if gathered {
        // Each spoil is a fragment, paid the moment it is picked up rather than
        // banked to the end: the panel then counts up as the walk goes, and a
        // gleaning run abandoned half way is worth what it actually did.
        give_fragment(world, stacks);
    }
    if solved {
        // Removed first, then refreshed, or the ways keep the solved maze's
        // readings and a bound solver follows them for ever.
        world.entity_mut(stacks).remove::<Maze>();
        publish(world, stacks);
        finish_walk(world, stacks, gathered);
        return;
    }
    publish(world, stacks);
    // Logged, not drawn (`RecordBuilder::quiet`): the map already said it. The
    // wall refusal is not quiet, since nothing moved (§6).
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
/// Mutated in place, never respawned — `heat.rs`'s `NodeId` hazard. Takes the
/// stacks rather than reading `Cwd`, since a load tidies every puzzle from
/// wherever the player saved.
pub(crate) fn publish(world: &mut World, stacks: Entity) {
    let room = super::readings::room_of(world, stacks);
    let readings: Vec<(Way, Option<&'static str>, bool, bool, Option<u8>)> = {
        let maze = world.get::<Maze>(stacks);
        Way::ALL
            .into_iter()
            .map(|way| {
                (
                    way,
                    maze.and_then(|maze| maze.reading(way)).map(Sense::word),
                    maze.is_some_and(|maze| maze.came() == Some(way)),
                    maze.is_some_and(|maze| maze.spoil(way)),
                    // Nought is not published, as everywhere else; unwalked
                    // floor is `passage`.
                    maze.and_then(|maze| maze.marks(way)).filter(|&n| n > 0),
                )
            })
            .collect()
    };

    for (way, word, came, spoil, marks) in readings {
        let Some(node) = room.and_then(|room| super::readings::reading(world, room, way.word()))
        else {
            continue;
        };
        for held in tower::children_of(world, node) {
            world.entity_mut(held).despawn();
        }
        if let Some(word) = word {
            tower::raise_reading(world, node, word);
        }
        // A second child, not a replacement: a way can be `walked` and the way
        // you came at once, and a solver asks both — see `maze::BACK`.
        if came {
            tower::raise_reading(world, node, tower::maze::BACK);
        }
        // And a third, for the same reason: a spoil nearly always sits in a
        // `passage`, so replacing the reading would cost a solver the one word
        // that says whether the square has been walked.
        if spoil {
            tower::raise_reading(world, node, tower::maze::SPOIL);
        }
        // And how often the square beyond has been walked, as a number, so a
        // spell can prefer the less-trodden way.
        if let Some(count) = marks {
            tower::raise_count(world, node, tower::maze::MARKS, u32::from(count));
        }
    }

    republish_errand(world, stacks);
}

/// Put the maze's errand on the stacks, as a word a spell can ask for.
///
/// A named child, like a way's reading, so `if the stacks has gleaning` needs
/// no new grammar. Only the errand's own children are cleared: the stacks also
/// hold fragments, which `holdings` must not count a reading among.
fn republish_errand(world: &mut World, stacks: Entity) {
    let wanted = world
        .get::<Maze>(stacks)
        .and_then(|maze| maze.errand().word());

    for node in tower::children_of(world, stacks) {
        let named = world
            .get::<tower::Name>(node)
            .is_some_and(|name| tower::maze::Errand::ALL.contains(&name.0.as_str()));
        if named && world.get::<tower::Name>(node).map(|name| name.0.as_str()) != wanted {
            world.entity_mut(node).despawn();
        }
    }

    let Some(word) = wanted else {
        return;
    };
    let already = tower::children_of(world, stacks)
        .into_iter()
        .any(|node| world.get::<tower::Name>(node).is_some_and(|n| n.0 == word));
    if !already {
        tower::raise_reading(world, stacks, word);
    }
}

/// One fragment onto the archive's shelf.
///
/// Silent: the map shows it (§19). Onto the cabinet by `tower::home`'s rule,
/// since stock lives on a shelf; the stacks only if the archive has none.
fn give_fragment(world: &mut World, stacks: Entity) {
    // The kind asked of `Recipes::kind_of`, as `debug_spawn` does, so one
    // word is never two kinds that `stock::give` silently merges.
    let kind = world
        .resource::<crate::content::Recipes>()
        .kind_of(FRAGMENT);
    let shelf = tower::home(world, FRAGMENT).unwrap_or(stacks);
    // A `bountiful` stacks pays twice — the charm read off the thing doing the
    // work, not the shelf.
    let paid = if tower::charmed(world, stacks, tower::charm::Kind::Bountiful) {
        2
    } else {
        1
    };
    tower::give(world, shelf, FRAGMENT, kind, paid);
}

/// The walk is over: what it was worth, said once.
///
/// `gathered` is whether this last step also picked something up, which is what
/// tells the two errands apart at the end: a [`Way`](tower::maze::Errand::Way)
/// ends by arriving somewhere, a [`Glean`](tower::maze::Errand::Glean) by taking
/// the last spoil — and the second has already had its fragment.
fn finish_walk(world: &mut World, stacks: Entity, gathered: bool) {
    if !gathered {
        give_fragment(world, stacks);
    }

    let earned = tower::worth(world, "stacks");
    let message = world
        .resource::<Prose>()
        .line("research_solved", &[("name", FRAGMENT)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Research.canonical())
        .text(FieldName::Detail, FRAGMENT)
        .text(FieldName::At, "stacks")
        .text(FieldName::Source, "stacks")
        .count(FieldName::Quantity, earned)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
    // `making`, because the maze shelved a fragment. It reached `tower::give` a
    // dozen lines up and still answered `sold()` with no, so the archive was
    // the one room that stocked itself for no standing at all.
    tower::done(world, &tower::Work::at("stacks").making(FRAGMENT), earned);
}

/// What a solved maze yields. Four make a scroll.
///
/// One name, and no roll: four drawn uniformly was coupon-collector attrition,
/// ~8.3 solves a scroll with no decision in it (§19). Distinct names return
/// with the spells that want them.
const FRAGMENT: &str = "fragment";

/// A maze, from the archive's own stream.
///
/// Built in one go: a maze that grew as it was walked would issue `NodeId`s at
/// a rate depending on how the ticks were consumed — `heat.rs`'s hazard.
fn generate(world: &mut World) -> Maze {
    let mut rngs = world.resource_mut::<Rngs>();
    let rng = rngs.stream(RngStream::Archive);
    let cells = WIDTH * HEIGHT;

    // Carved as cells, laid out as squares. The spanning tree is easiest to
    // reason about one cell at a time, but the *grid* has to hold the walls or
    // a step is two characters on screen and reads as moving two spaces.
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

    // Randomised Prim's: short passages, frequent junctions, small dead ends —
    // the busy texture the stacks want, where a backtracker carves long
    // corridors. Still a spanning tree, which `tests/solver.rs` sweeps.
    //
    // Seeded at a random cell, so the radial centre is not always the start.
    let seed = rand::RngExt::random_range(rng, 0..cells);
    let mut seen = vec![false; cells];
    seen[seed] = true;
    // Cells adjacent to the maze but not yet in it, each paired with the cell it
    // would be joined to. Duplicates are fine: one is taken and the rest fail
    // the `seen` check and are dropped.
    let mut frontier: Vec<(usize, usize)> = (0..4)
        .filter_map(|index| neighbour(seed, index).map(|next| (seed, next)))
        .collect();

    while !frontier.is_empty() {
        let pick = rand::RngExt::random_range(rng, 0..frontier.len());
        let (from, next) = frontier.swap_remove(pick);
        if seen[next] {
            continue;
        }
        seen[next] = true;
        // The square between the two cells becomes floor. Adjacent cells are
        // two squares apart, so their indices average to it exactly.
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
    let start = CORNERS[rand::RngExt::random_range(rng, 0..CORNERS.len())];
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
/// A cell weighs `(reach - away + 1)^PULL`, `away` its Manhattan distance from
/// that corner. Cubed, as measured: the way out lands within 8 of the corner
/// 63% of the time and every cell can still win. §19, "`execute/` (`0.20.13`)"
/// has the table.
const PULL: u32 = 3;

/// A cell's index from its coordinates.
const fn cell_at((x, y): (usize, usize)) -> usize {
    y * WIDTH + x
}

/// A cell drawn at random, weighted toward the corner opposite `from`.
///
/// Every cell can win, so the errand stays long without being one journey
/// over and over. Integer-only, so a seed replays on every machine (§3).
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
    let mut roll = rand::RngExt::random_range(rng, 0..total.max(1));
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
    Way::named(crate::parser::leaf(word))
}

/// The stacks, if the player is standing where one is.
///
/// Reads `Cwd`, so a frontend gets the map only where the player could
/// `survey` the ways themselves.
pub(crate) fn stacks(world: &World) -> Option<Entity> {
    super::readings::fixture(world, Verb::Research)
}

/// The stacks in `room`, if it has them.
pub(crate) fn stacks_in(world: &World, room: Entity) -> Option<Entity> {
    super::readings::beside(world, room, Verb::Research)
}

/// One authored line about the reading, under the verb that caused it.
pub(super) fn say(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    file(world, verb, key, args, role, false);
}

/// [`say`], into the log only — for a step the map has already drawn.
///
/// Apart from [`say`] so the choice is visible at the emit site.
fn quietly(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    file(world, verb, key, args, role, true);
}

/// The record both write, with the stacks in `Source`: that is what puts it in
/// `archive.log` (`files::in_domain`).
fn file(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role, quiet: bool) {
    let message = world.resource::<Prose>().line(key, args);
    let at = stacks(world)
        .and_then(|stacks| world.get::<tower::Name>(stacks))
        .map(|name| name.0.clone());

    let mut records = world.resource_mut::<Scrollback>();
    let mut builder = records
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Message, &message)
        .role(role);
    if quiet {
        builder = builder.quiet();
    }
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
        let maze = sim.stacks().expect("research opens the stacks");

        // Square indices back to cells: `square_of` puts cell (x, y) at square
        // (2y + 1, 2x + 1), so the inverse halves each axis.
        let cell = |index: usize| {
            let span = usize::from(maze.width);
            ((index % span - 1) / 2, (index / span - 1) / 2)
        };
        (
            cell(maze.at),
            cell(maze.exit.expect("fresh stacks are walked for their exit")),
        )
    }

    /// Enough seeds that a corner missed by chance is vanishingly unlikely: four
    /// corners over 120 draws, so each is expected 30 times.
    const SWEEP: u64 = 120;

    #[test]
    fn the_reading_starts_in_every_corner_across_seeds() {
        // It was the top-left one, always. Four corners uniformly is the first
        // half of making two walks feel like two errands.
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
        // Both halves, since either alone is a plausible wrong fix: tighter and
        // the exit is the corner again, looser and the errand is three steps.
        // The margins fit `PULL = 3`'s figures (63% within 8, 13% past 12).
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
        // The map is the report. "Quiet" means not drawn, never not emitted
        // (§3), so both halves are asserted.
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
    fn the_same_seed_still_draws_the_same_stacks() {
        // Three new rolls went into the archive's stream. §3 makes replay
        // architectural, so the property that matters is not *which* maze a seed
        // draws but that it keeps drawing that one.
        for seed in [1, 7, 99] {
            assert_eq!(ends(seed), ends(seed), "seed {seed} is not reproducible");
        }
    }
}
