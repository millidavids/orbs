//! `wander` — hand the arrow keys the archive's stacks (§10, §19).
//!
//! It buys the keys and nothing else. The map draws whenever a maze is open,
//! said or not — which is what makes watching a bound solver work. So unlike
//! `scribe` and `weave`, this opens no surface: it changes *who owns the
//! arrows*, and a player walking by hand sees the picture a spell walks.
//!
//! The sim owns only the request. Where the arrows point and whether the border
//! is lit are facts about a pane, and rule 2 keeps panes out of this crate. The
//! *steps* are not: an arrow becomes an ordinary `follow <way>` submission, so
//! by hand and by spell are one code path and replay from `(seed, submissions)`
//! is untouched by anybody having looked.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tower::Maze;

/// Whether `wander` has asked for the arrow keys.
///
/// A request, not a state, for the reason [`Opening`](super::Opening) and
/// [`Weaving`](super::Weaving) both give: the verb asks once, and a frontend
/// polling a persistent flag would seize the keyboard again on the frame after
/// the player let it go.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Wandering(bool);

impl Wandering {
    /// Ask for the arrows.
    pub const fn ask(&mut self) {
        self.0 = true;
    }

    /// Whether a request is waiting, without taking it.
    ///
    /// The peek exists because taking needs `&mut`, and a system reaching for it
    /// stamps the resource's change tick merely by asking — which leaves its own
    /// `resource_changed` run condition true for ever.
    #[must_use]
    pub const fn pending(&self) -> bool {
        self.0
    }

    /// Take the pending request, if there is one.
    pub const fn take(&mut self) -> bool {
        let asked = self.0;
        self.0 = false;
        asked
    }
}

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

/// One authored line about the walking.
fn say(world: &mut World, key: &str, role: Role) {
    let message = world.resource::<Prose>().line(key, &[]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Wander.canonical())
        .text(FieldName::Message, &message)
        .role(role)
        .finish();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;
    use orbs_render::Value;

    /// Every message the orb has said.
    fn messages(sim: &Sim) -> Vec<String> {
        sim.scrollback()
            .records()
            .iter()
            .filter_map(|record| record.field(FieldName::Message))
            .filter_map(|value| match value {
                Value::Text(text) => Some(text.to_owned()),
                _ => None,
            })
            .collect()
    }

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
            messages(&sim).iter().any(|line| line.contains("research")),
            "the refusal did not say what would have worked: {:?}",
            messages(&sim),
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
            messages(&sim).iter().any(|line| line.contains("stacks")),
            "the refusal did not say what was missing: {:?}",
            messages(&sim),
        );
    }

    #[test]
    fn a_spell_cannot_seize_the_keyboard() {
        // Driven, not asserted against `may_issue`: that list is a `matches!`
        // nothing makes the compiler check. `repeat 100 / wander` is a
        // soft-lock — the prompt is dead while the arrows have the maze, so
        // Escape would race the script for the keyboard.
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
            messages(&sim).iter().any(|line| line.contains("will not")),
            "the spell was refused silently: {:?}",
            messages(&sim),
        );
    }
}
