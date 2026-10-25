//! `weave` — looking at what the work has bought (DESIGN.md §11.5, §19).
//!
//! The way in is a word, as for [`unfurl`](super::unfurl): it hands a screen
//! the keyboard to draw the Ley Line ([`tower::ley`]) and the mastery lines
//! ([`mod@crate::tower::mastery`]). The sim owns only the decision to open it,
//! so a replay is the same whether or not anybody looked.
//!
//! [`grant`] takes a fork node on a tick boundary. A mastery station is reached
//! by its deed, never taken, and the world re-asks every guard here.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::Weaving;
use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tower;

/// Take a fork node, on the tick after the screen asked for it.
///
/// Every guard is re-asked, since the screen's arithmetic is not the rules
/// (§19). Silent when it refuses, as the screen already said so; success is a
/// record.
pub(super) fn grant(world: &mut World, id: &str) {
    if !tower::is_real(id) {
        return;
    }
    let open = tower::ley_line(world)
        .into_iter()
        .filter(|station| station.fork)
        .flat_map(|station| station.nodes)
        .any(|node| node.id == id && node.standing == tower::Standing::Open);
    if !open {
        return;
    }
    world.resource_mut::<tower::Taken>().hold(id);

    // The node's own sentence rides in the record, so the log says what was
    // bought and not only that something was.
    let detail = world
        .resource::<Prose>()
        .line(&format!("weave_node_{id}"), &[]);
    let message = world
        .resource::<Prose>()
        .line("weave_took", &[("name", id), ("detail", &detail)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Weave.canonical())
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
}

/// Open the weave screen.
pub(super) fn weave(world: &mut World) {
    world.resource_mut::<Weaving>().ask();

    // It answers, as every live verb must, with the number the transcript
    // keeps after the screen closes.
    let earned = tower::Experience::get(*world.resource::<tower::Experience>());
    let next = tower::next(world);
    let key = if next.is_some() {
        "weave_begins"
    } else {
        "weave_begins_topped_out"
    };
    // `quantity` is the earned total. The threshold rides in `detail`: a fact
    // about `progression.toml`, not about this moment.
    let message = world.resource::<Prose>().line(
        key,
        &[
            ("quantity", &earned.to_string()),
            ("detail", &next.unwrap_or_default().to_string()),
        ],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Weave.canonical())
        .count(FieldName::Quantity, earned)
        .text(FieldName::Message, &message)
        .role(Role::Normal)
        .finish();
}

#[cfg(test)]
mod tests {
    use crate::Sim;
    use crate::testing::said;

    #[test]
    fn it_asks_once_rather_than_every_frame() {
        // The `Opening` rule: a frontend that polled a persistent flag would
        // reopen the screen on the frame after the player closed it.
        let mut sim = Sim::new(1);
        sim.submit("weave");
        sim.step();

        assert!(sim.has_weaving());
        assert!(sim.weaving(), "the request was not there to take");
        assert!(!sim.weaving(), "it was still there after being taken");
    }

    #[test]
    fn it_says_what_the_player_asked_for() {
        // §6 forbids a bare fact where a sentence would teach, and the sentence
        // outlives the screen — a player who closes it still has the number in
        // the transcript.
        let mut sim = Sim::new(1);
        sim.submit("weave");
        sim.step();

        let said = said(&sim);
        assert!(
            said.iter().any(|line| line.contains("16")),
            "it did not name what is next: {said:?}",
        );
    }

    #[test]
    fn a_spell_cannot_seize_the_screen() {
        // A spell in a `repeat` would re-seize the screen faster than Escape.
        // Through a real cast, since `may_issue` is a `matches!`; in the
        // laboratory, or the spell is the root's.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.write_spell("grab", &["weave".to_owned()]);
        sim.step();
        sim.submit("invoke grab");
        sim.step_n(4);

        assert!(!sim.has_weaving(), "a spell opened the screen");
        assert!(
            said(&sim).iter().any(|line| line.contains("will not do")),
            "it was refused silently: {:?}",
            said(&sim),
        );
    }

    #[test]
    fn a_topped_out_tower_is_not_told_to_work_toward_nothing() {
        // `next` is `None` once every authored threshold is passed, and the
        // sentence about what is next would then name 0 — a target that reads
        // as already reached.
        let mut sim = Sim::new(1);
        crate::tower::credit(sim.world_mut(), 10_000);
        sim.submit("weave");
        sim.step();

        let said = said(&sim);
        assert!(
            !said.iter().any(|line| line.contains(" 0")),
            "it offered a threshold of nothing: {said:?}",
        );
    }
}
