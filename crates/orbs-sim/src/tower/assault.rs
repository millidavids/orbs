//! What a siege does to the tower behind you (DESIGN.md §5.1, §8.1).
//!
//! §5.1 splits aberrations in two, and the split is the design:
//!
//! | | **Nuisance (idle)** | **Adversarial (siege)** |
//! |---|---|---|
//! | Source | Environmental, random | An intelligent enemy |
//! | Surfaces | Environmental only — never scripts, schedules or logs | All four (§8.1) |
//! | Response | Repair occupies a pane | Diagnose and repair under pressure |
//!
//! Siege-only, on a resolved round and never a tick, so the calm layer is safe
//! (pillar 4) and an open siege waits for the player (§19).
//!
//! Two surfaces reach a script — its text and its clock — so the response is
//! debugging, and `verify` is the diagnosis. The round says it sabotaged without
//! saying what, so finding the lie is the decision.

use bevy_ecs::prelude::*;

use super::node::{Held, Name, NodeId};
use super::sabotage::{Log, Poisoned};
use super::spell::Bound;
use crate::rng::{RngStream, Rngs};

/// How often a round sabotages something, as one chance in this many — capped
/// so repairs cannot spiral (§5.1).
pub const ODDS: u32 = 3;

/// A bound spell whose schedule an enemy has retimed.
///
/// The subtlest surface: the text is perfect, and only `verify` finds it.
#[derive(Component, Debug, Clone)]
pub struct Retimed {
    /// How many extra ticks each of its steps now costs.
    pub drag: u64,
}

/// A spell whose text an enemy has rewritten.
///
/// The true lines are kept, as `Substituted` does: misdirection, not theft, and
/// what makes the repair a repair.
#[derive(Component, Debug, Clone)]
pub struct Rewritten {
    /// What the player actually wrote.
    pub was: Vec<String>,
    /// How the orb read it — what ran before the strike, and what a repair puts
    /// back beside [`was`](Self::was).
    pub read: super::Read,
}

/// Which adversarial surface a round reached, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reached {
    /// A spell's text was rewritten.
    Script,
    /// A bound spell's clock was dragged.
    Clock,
    /// A domain's log was poisoned.
    Log,
}

/// Roll for sabotage, and do it. Called once per resolved round.
///
/// One draw, first and unconditionally; the target is that roll's quotient,
/// never a second draw (§19's `drift` defect). What it breaks stays broken
/// until the player purges it (§19).
pub fn strike(world: &mut World) -> Option<Reached> {
    let roll: u32 = {
        let mut rngs = world.resource_mut::<Rngs>();
        // Wide, so the quotient has room to choose a target.
        rand::RngExt::random_range(rngs.stream(RngStream::Siege), 0..u32::from(u16::MAX))
    };

    if !roll.is_multiple_of(ODDS) {
        return None;
    }
    let choice = roll / ODDS;

    // Only a spell with a corruptible line, or `verify` would find nothing.
    let mut scripts = pool::<(With<Held>, Without<Rewritten>)>(world);
    scripts.retain(|node| {
        world
            .get::<Held>(*node)
            .is_some_and(|held| held.0.iter().any(|line| corruptible(line)))
    });
    let surfaces: Vec<(Reached, Vec<Entity>)> = [
        (
            Reached::Clock,
            pool::<(With<Bound>, Without<Retimed>)>(world),
        ),
        (Reached::Script, scripts),
        (Reached::Log, pool::<(With<Log>, Without<Poisoned>)>(world)),
    ]
    .into_iter()
    .filter(|(_, nodes)| !nodes.is_empty())
    .collect();
    if surfaces.is_empty() {
        return None;
    }

    // The surface first, then the node: a fixed order starved the log, reached
    // only once every spell was rewritten.
    let choice = choice as usize;
    let (reached, nodes) = &surfaces[choice % surfaces.len()];
    let node = nodes[(choice / surfaces.len()) % nodes.len()];
    // A `shielded` node is struck and holds — skipped after the choice, so the
    // same roll reaches the same node.
    if held(world, node) {
        return None;
    }
    match reached {
        Reached::Clock => {
            // From the part of the roll the surface and node did not use, so one
            // draw still covers it.
            let drag = (choice / surfaces.len() / nodes.len() % 3) as u64 + 1;
            world.entity_mut(node).insert(Retimed { drag });
            super::poison(world, node);
        }
        Reached::Script => rewrite(world, node),
        Reached::Log => super::poison(world, node),
    }
    Some(*reached)
}

/// What a round can reach on one surface, in an order a restored world agrees
/// with.
fn pool<F: bevy_ecs::query::QueryFilter>(world: &mut World) -> Vec<Entity> {
    // Never a shut room's: a strike there latches a rail mark on a box drawn
    // dark, a fault nobody can walk in and find.
    let mut nodes: Vec<(Entity, String, NodeId)> = world
        .query_filtered::<(Entity, &Name, &NodeId), (F, Without<super::Sealed>)>()
        .iter(world)
        .map(|(entity, name, id)| (entity, name.0.clone(), *id))
        .collect();
    // Name, then `NodeId`: archetype order differs between a lived world and a
    // rebuilt one, and a name alone ties when two rooms hold one spell.
    nodes.sort_unstable_by(|(_, a, x), (_, b, y)| a.cmp(b).then(x.cmp(y)));
    nodes.into_iter().map(|(entity, _, _)| entity).collect()
}

/// Whether a live `shielded` charm is holding this node.
///
/// The clock, never `With<Charmed>`: a charm is an interval with no expiry
/// system, so a component test would shield a spell for ever after its first.
fn held(world: &World, node: Entity) -> bool {
    super::charmed(world, node, super::charm::Kind::Shielded)
}

/// Corrupt one line of a spell, keeping what it said.
///
/// One line, not the file: a spell rewritten wholesale is one the player throws
/// away, while one line changed is one they have to *find*.
fn rewrite(world: &mut World, node: Entity) {
    let Some(held) = world.get::<Held>(node).cloned() else {
        return;
    };
    if held.0.is_empty() {
        return;
    }
    // Only lines with something to misdirect, walked from the tick's position.
    // No draw: the tick is already replayed state.
    let candidates: Vec<usize> = (0..held.0.len())
        .filter(|at| corruptible(&held.0[*at]))
        .collect();
    if candidates.is_empty() {
        return;
    }
    let from = usize::try_from(world.resource::<crate::tick::Tick>().get()).unwrap_or(0);
    let at = candidates[from % candidates.len()];

    let mut lines = held.0.clone();
    // The last word: `grind sage-` parses, runs and quietly does nothing, where
    // a struck line would fault and give the game away.
    lines[at] = corrupt(&lines[at]);

    // `Read` too, since that is what compiles — and that one line of it, so a
    // repair restores what ran rather than what was typed.
    let reading = world.get::<super::Read>(node).map_or_else(
        || super::Read::verbatim(&held.0),
        |read| read.aligned(&held.0),
    );
    let mut read = reading.lines.clone();
    read[at] = lines[at].clone();
    let corrupted = super::Read::new(&lines, read, reading.by);
    world.entity_mut(node).insert(Rewritten {
        was: held.0,
        read: reading,
    });
    world.entity_mut(node).insert(Held(lines));
    world.entity_mut(node).insert(corrupted);
    super::poison(world, node);
}

/// Append the substitution sigil to a line's last word.
///
/// A line with no argument is left alone, since suffixing `end` breaks the
/// parse. Split at the word's own offset, so the player's spacing survives.
fn corrupt(line: &str) -> String {
    let mut words = line.split_whitespace();
    let Some(last) = words.next_back() else {
        return line.to_owned();
    };
    if words.next().is_none() {
        return line.to_owned();
    }
    let lie = super::sabotage::claimed(last);
    let Some(at) = line.rfind(last) else {
        return line.to_owned();
    };
    format!("{}{lie}", &line[..at])
}

/// Whether a line has an argument worth corrupting.
///
/// A corrupted line must parse, run and quietly do nothing. So the last word is
/// refused when it is a count, a state (`SpellState::WORDS`), bracketed, or the
/// set of a `for each` — each breaks the line rather than lying to it.
fn corruptible(line: &str) -> bool {
    // A comment compiles to nothing, so corrupting one changes nothing.
    if line.trim_start().starts_with('#') {
        return false;
    }
    let words: Vec<&str> = line.split_whitespace().collect();
    if words.len() < 2 {
        return false;
    }
    let last = words[words.len() - 1];
    // A trailing number is a count, and `repeat 3-` does not parse.
    if last.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    // `for each way` — the set is the word before the last. Asked of
    // `particle()` rather than the literal.
    if crate::parser::SpellWord::For.particle() == Some(words[words.len() - 2]) {
        return false;
    }
    !crate::parser::SpellState::WORDS.contains(&last) && !last.contains(['(', ')'])
}

/// Give a rewritten spell its own words back — `sabotage::restore`'s mirror,
/// which makes notice → `verify` → `purge` a loop.
pub fn unwrite(world: &mut World, node: Entity) -> bool {
    let Some(Rewritten { was, read }) = world.get::<Rewritten>(node).cloned() else {
        return false;
    };
    // Both back, or the tower runs the enemy's line while `peruse` shows the
    // player's.
    world.entity_mut(node).insert(Held(was));
    world.entity_mut(node).insert(read);
    world.entity_mut(node).remove::<Rewritten>();
    true
}

/// Give a retimed spell its own clock back.
pub fn untime(world: &mut World, node: Entity) -> bool {
    if world.get::<Retimed>(node).is_none() {
        return false;
    }
    world.entity_mut(node).remove::<Retimed>();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    /// Every line the enemy is allowed to touch must still parse.
    ///
    /// Asked of every line of every shipped solver, so a spell added to
    /// `dev_spells.toml` in an unmet shape fails here rather than in a player's
    /// siege. The old guard let through `part look()` and `is idle`.
    #[test]
    fn a_line_the_enemy_may_corrupt_still_reads_as_a_line() {
        let spells = crate::content::Spells::builtin();
        let mut broken = Vec::new();
        for (name, spell) in spells.iter() {
            for line in &spell.lines {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || !corruptible(line) {
                    continue;
                }
                let lied = corrupt(line);
                // A spell word still opens it — which `part`, `for each` and
                // `is <state>` broke.
                let before = crate::parser::spell_word(line);
                let after = crate::parser::spell_word(&lied);
                if before != after {
                    broken.push(format!("{name}: {line:?} -> {lied:?} changed its word"));
                }
                // ...and the corruption is one suffix on the last word, never a
                // reflow of the line.
                if !lied.starts_with(line.split_whitespace().next().unwrap_or_default()) {
                    broken.push(format!("{name}: {line:?} -> {lied:?} lost its head"));
                }
            }
        }
        assert!(
            broken.is_empty(),
            "the enemy would break these rather than lie to them: {broken:#?}",
        );
    }

    /// The lie is one word, and the player's own spacing survives it.
    #[test]
    fn corrupting_a_line_keeps_every_character_before_its_last_word() {
        let line = "    grind    sage";
        let lied = corrupt(line);
        assert!(
            lied.starts_with("    grind    "),
            "the enemy reflowed the line: {lied:?}",
        );
        assert!(lied.ends_with('-'), "nothing was corrupted: {lied:?}");
    }

    /// The three shapes that were getting through, by name.
    #[test]
    fn a_part_a_call_and_a_state_are_left_alone() {
        for line in [
            "part look()",
            "look()",
            "if the mortar_and_pestle is idle",
            "for each way",
            "repeat 3",
            "end",
        ] {
            assert!(
                !corruptible(line),
                "{line:?} would be corrupted, and it does not survive it",
            );
        }
        // ...and the ordinary case still is, or the guard ate the surface
        // rather than narrowing it.
        assert!(corruptible("grind sage"));
        assert!(corruptible("haul wellspring barrier"));
    }

    #[test]
    fn the_calm_layer_is_never_touched() {
        // Pillar 4: *"the enemy never touches scripts, schedules, or logs in
        // the calm layer."* Nothing here runs on a tick — only on a resolved
        // round, and `hold` is the only caller.
        let mut sim = Sim::new(11);
        sim.step_n(2000);
        let world = sim.world();
        let rewritten = world
            .iter_entities()
            .filter(bevy_ecs::world::EntityRef::contains::<Rewritten>)
            .count();
        let retimed = world
            .iter_entities()
            .filter(bevy_ecs::world::EntityRef::contains::<Retimed>)
            .count();
        assert_eq!(
            (rewritten, retimed),
            (0, 0),
            "two thousand calm ticks produced adversarial sabotage",
        );
    }

    #[test]
    fn a_rewritten_spell_keeps_what_it_said() {
        // Misdirection, not theft, which is what makes the repair loop a loop.
        let mut sim = Sim::new(11);
        sim.submit("attend archive");
        sim.step();
        sim.submit("debug_spell threading");
        sim.step();

        let node = sim
            .world()
            .iter_entities()
            .find(bevy_ecs::world::EntityRef::contains::<Held>)
            .map(|entity| entity.id())
            .expect("a spell was shelved");
        let before = sim.world().get::<Held>(node).cloned().expect("held").0;

        rewrite(sim.world_mut(), node);
        let after = sim.world().get::<Held>(node).cloned().expect("held").0;
        assert_ne!(before, after, "rewriting changed nothing");
        assert_eq!(
            sim.world().get::<Rewritten>(node).map(|r| r.was.clone()),
            Some(before.clone()),
            "the player's own words were not kept",
        );

        // The sabotage must reach the program, not only the file: touching
        // `Held` alone shows a broken line while the orb runs the clean one.
        assert_eq!(
            crate::tower::spell::source(sim.world(), node),
            after,
            "the enemy corrupted the file and not what runs",
        );

        assert!(unwrite(sim.world_mut(), node));
        assert_eq!(
            sim.world().get::<Held>(node).cloned().expect("held").0,
            before,
            "a repaired spell did not read as it was written",
        );
        assert_eq!(
            crate::tower::spell::source(sim.world(), node),
            before,
            "a repaired spell still runs the enemy's line",
        );
    }

    #[test]
    fn a_strike_corrupts_one_line_of_the_reading_and_a_repair_restores_all_of_it() {
        // Writing the whole corrupted text into `Read` sent a runnable spell
        // back to its player's loose words, and the repair put those back.
        let mut sim = Sim::new(11);
        let typed: Vec<String> = ["work the sage down", "hang on ten ticks", "grind sage"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        sim.write_spell_reading("morning", &typed, &crate::Copyist::worked());
        sim.step();
        let node = sim
            .world()
            .iter_entities()
            .find(|entity| {
                entity
                    .get::<Name>()
                    .is_some_and(|name| name.0 == "morning.spell")
            })
            .map(|entity| entity.id())
            .expect("the spell was written");
        let before = crate::tower::spell::source(sim.world(), node);
        assert_eq!(
            before,
            ["grind sage", "bide 10", "grind sage"],
            "the fixture did not read the spell",
        );

        rewrite(sim.world_mut(), node);
        let held = sim.world().get::<Held>(node).cloned().expect("held").0;
        let after = crate::tower::spell::source(sim.world(), node);
        let struck: Vec<usize> = (0..typed.len())
            .filter(|at| held[*at] != typed[*at])
            .collect();
        assert_eq!(struck.len(), 1, "one line is struck: {held:?}");
        for at in 0..typed.len() {
            let wanted = if struck.contains(&at) {
                &held[at]
            } else {
                &before[at]
            };
            assert_eq!(&after[at], wanted, "line {at} runs the wrong thing");
        }

        assert!(unwrite(sim.world_mut(), node));
        assert_eq!(
            crate::tower::spell::source(sim.world(), node),
            before,
            "the repair put back the text and not what ran",
        );
    }

    #[test]
    fn only_one_line_is_touched() {
        // Wholesale is a spell the player throws away; one line is one they
        // have to find.
        let mut sim = Sim::new(11);
        sim.submit("attend archive");
        sim.step();
        sim.submit("debug_spell threading");
        sim.step();
        let node = sim
            .world()
            .iter_entities()
            .find(bevy_ecs::world::EntityRef::contains::<Held>)
            .map(|entity| entity.id())
            .expect("a spell was shelved");
        let before = sim.world().get::<Held>(node).cloned().expect("held").0;

        rewrite(sim.world_mut(), node);
        let after = sim.world().get::<Held>(node).cloned().expect("held").0;

        let differing = before.iter().zip(&after).filter(|(a, b)| a != b).count();
        assert_eq!(differing, 1, "more than one line was rewritten");
        assert_eq!(before.len(), after.len(), "the spell changed length");
    }
}

#[cfg(test)]
mod dragging {
    use super::*;
    use crate::Sim;

    #[test]
    fn a_retimed_spell_loses_ticks_at_the_shipped_budget() {
        // Asked with an allowance of 4, this hid the defect: at the shipped
        // `SCRIPT_BUDGET` of 1, `allowance - drag` floored at one returns one
        // for every drag, so the surface cost an audit and changed nothing.
        let mut sim = Sim::new(11);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("invoke first_light");
        sim.step();

        let node = sim
            .world_mut()
            .query::<(Entity, &crate::tower::spell::Running)>()
            .iter(sim.world())
            .map(|(entity, _)| entity)
            .next();
        let Some(node) = node else { return };

        for drag in [1_u64, 2, 3] {
            sim.world_mut().entity_mut(node).insert(Retimed { drag });
            let over: Vec<usize> = (0..24)
                .map(|_| {
                    sim.step();
                    crate::tower::spell::dragged_for_test(sim.world(), node, 1)
                })
                .collect();
            let ran = over.iter().filter(|left| **left > 0).count();
            assert!(
                ran > 0,
                "a drag of {drag} halted the spell outright, which §8 forbids",
            );
            assert!(
                ran < over.len(),
                "a drag of {drag} cost nothing at a budget of one",
            );
        }
    }
}
