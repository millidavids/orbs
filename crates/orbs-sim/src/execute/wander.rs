//! `wander` — hand the arrow keys the archive's stacks (§10, §19).
//!
//! It buys the keys and nothing else: the map draws whenever a maze is open,
//! so this opens no surface. The sim owns only the request; an arrow becomes
//! an ordinary `follow <way>` submission, so hand and spell are one path.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use super::Wandering;
use crate::parser::Verb;
use crate::tower::Maze;

/// `wander` — walk the stacks by hand.
pub(super) fn wander(world: &mut World) {
    // §6 forbids a refusal that does not say what would have worked, and these
    // are the two a player reaches: wrong place, or too early.
    let Some(lectern) = super::stacks(world) else {
        say(world, "wander_nowhere", Role::Danger);
        return;
    };
    if world.get::<Maze>(lectern).is_none() {
        say(world, "wander_unopened", Role::Cost);
        return;
    }

    world.resource_mut::<Wandering>().ask();
    say(world, "wander_opens", Role::Success);
}

/// One authored line about the walking, filed as the archive's other lines are.
fn say(world: &mut World, key: &str, role: Role) {
    super::research::say(world, Verb::Wander, key, &[], role);
}

#[cfg(test)]
mod tests {
    use crate::Sim;
    use crate::testing::said;

    /// A wizard standing in the archive with the stacks open.
    fn opened() -> Sim {
        let mut sim = Sim::new(1);
        sim.submit("attend archive");
        sim.step();
        sim.submit("research");
        sim.step();
        sim
    }

    #[test]
    fn it_asks_once_rather_than_every_frame() {
        let mut sim = opened();
        sim.submit("wander");
        sim.step();

        assert!(sim.has_wandering(), "the request was not made");
        assert!(sim.wandering(), "the request could not be taken");
        assert!(
            !sim.wandering(),
            "a taken request came back, which would seize the keyboard again \
             on the frame after the player let it go",
        );
    }

    #[test]
    fn there_is_nothing_to_walk_until_a_maze_is_open() {
        // Typed too early, so the refusal has to name the way in (§6).
        let mut sim = Sim::new(1);
        sim.submit("attend archive");
        sim.step();
        sim.submit("wander");
        sim.step();

        assert!(
            !sim.has_wandering(),
            "the arrows were given a maze that is not there"
        );
        assert!(
            said(&sim).iter().any(|line| line.contains("research")),
            "the refusal did not say what would have worked: {:?}",
            said(&sim),
        );
    }

    #[test]
    fn there_is_nothing_to_walk_outside_the_archive() {
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("wander");
        sim.step();

        assert!(!sim.has_wandering());
        // The stacks, where the maze lives since it moved off the lectern —
        // and §6 wants the refusal to name what the laboratory has not got.
        assert!(
            said(&sim).iter().any(|line| line.contains("stacks")),
            "the refusal did not say what was missing: {:?}",
            said(&sim),
        );
    }

    #[test]
    fn walking_and_letting_go_are_in_the_archive_log() {
        let mut sim = opened();
        for line in ["wander", "stop stacks"] {
            sim.submit(line);
            sim.step();
        }
        let before = said(&sim).len();
        sim.submit("peruse archive.log");
        sim.step();
        let log = said(&sim).split_off(before);
        for line in ["you step into the stacks", "you let the stacks close"] {
            assert!(
                log.iter().any(|said| said.contains(line)),
                "archive.log is missing {line:?}: {log:?}",
            );
        }
    }

    #[test]
    fn a_spell_cannot_seize_the_keyboard() {
        // Driven, since `may_issue` is a `matches!`. A spell that wanders in a
        // `repeat` would race Escape for the keyboard.
        let mut sim = opened();
        sim.write_spell("threading", &["wander".to_owned()]);
        sim.step();
        sim.submit("invoke threading");
        sim.step_n(4);

        assert!(
            !sim.has_wandering(),
            "a spell took the arrow keys away from the player",
        );
        assert!(
            said(&sim).iter().any(|line| line.contains("will not")),
            "the spell was refused silently: {:?}",
            said(&sim),
        );
    }
}
