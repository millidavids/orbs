//! `weave` — looking at what the work has bought (DESIGN.md §11.5, §19).
//!
//! # Progression had no surface at all
//!
//! `status` printed `experience 20` and `concentration 1` and that was the whole
//! of it: two numbers with nothing saying what they are for, what is next, or
//! what it costs. Concentration 1 — §11.5's *"moment the game becomes the game it
//! advertises"* — arrived as a single line and was never chosen.
//!
//! So the way in is a word, exactly as it is for [`unfurl`](super::unfurl). This
//! hands a frontend screen the keyboard; that screen draws the two tracks
//! ([`tower::mastery`](crate::tower::mastery)) and the total they are measured
//! against.
//!
//! # What the sim owns, and what it does not
//!
//! Only the *decision to open it*. Where a cursor sits and which track is being
//! looked at are facts about a pane, and rule 2 keeps panes out of this crate —
//! the same split `unfurl` makes, for the same reason. Looking is not a world
//! event, so `(seed, submissions)` replays identically whether or not anybody
//! opened the screen.
//!
//! # Taking a node
//!
//! This comment used to say nothing here takes one, because every authored node
//! was a marker — and it named exactly what the first real one would need: *"a
//! mutator, a `Submission` variant and a queued effect on a tick boundary, the
//! shape `Sim::write_spell` already has."* [`grant`] is that, and `steps_1` is
//! the node that earned it: the menagerie cannot be automated at one instruction
//! a tick, so a second step is the first thing in the game worth buying.
//!
//! **Markers still refuse in voice.** `tower::mastery::is_real` is what separates
//! them, derived from the grant so a node cannot be takeable and worthless at
//! once — and the screen asks it *before* sending, so a marker never reaches a
//! tick boundary at all.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tower;

/// A request to open the weave screen.
///
/// **Taken rather than read**, for the reason `Opening` and `Unfurling` both
/// give: the verb asks once, and a frontend polling a persistent flag would
/// reopen the screen every frame — including the frame after the player closed
/// it.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Weaving(bool);

impl Weaving {
    /// Ask for the screen.
    pub const fn ask(&mut self) {
        self.0 = true;
    }

    /// Whether a request is waiting, without taking it.
    ///
    /// The peek exists because taking needs `&mut`, and a system reaching for it
    /// stamps the resource's change tick merely by asking — which leaves its own
    /// `resource_changed` run condition true for ever. That defect is recorded
    /// against `open_requested`; this is the same pair of accessors.
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

/// Take a mastery node, on the tick after the screen asked for it.
///
/// **Every guard is re-asked here.** The screen refuses a locked node, a marker
/// and a spent tier before it sends — and a queued effect that trusted the
/// screen's arithmetic would be a second opinion about the rules, which is
/// exactly how a screen and a world come to disagree. §19 records that shape
/// going wrong repeatedly; it is cheap to ask twice and the world is the answer.
///
/// Silent when it refuses, because the screen already said so in voice. What it
/// says on success is a record, so `sift` and the log see the decision.
pub(super) fn grant(world: &mut World, id: &str) {
    if !tower::mastery::is_real(id) {
        return;
    }
    let open = tower::mastery(world)
        .into_iter()
        .flatten()
        .any(|node| node.id == id && node.standing == tower::Standing::Open);
    if !open {
        return;
    }
    world.resource_mut::<tower::Taken>().hold(id);

    let message = world
        .resource::<Prose>()
        .line("weave_took", &[("name", id)]);
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

    // **It answers, and it has to.** `a_dark_verb_only_acknowledges_and_a_live_one_does_not`
    // asserts every live verb emits a record that is more than its own bare
    // name, and `unfurl` — whose entire job is also handing a frontend surface
    // the keyboard — sets the precedent with `unfurl_begins`.
    //
    // What it says is the number, because that is the one fact a player who
    // typed this wanted and the transcript keeps it after the screen is closed.
    let earned = tower::Experience::get(*world.resource::<tower::Experience>());
    let next = tower::next(world);
    let key = if next.is_some() {
        "weave_begins"
    } else {
        "weave_begins_topped_out"
    };
    // **`quantity` is the earned total, matching `FieldName::Quantity` below.**
    // They were the other way round — the placeholder named `quantity` carried
    // the *next threshold* while the field named `Quantity` carried the total —
    // which is a trap for whoever authors the next line in this file.
    //
    // The threshold rides in `detail` rather than in a field of its own, and
    // deliberately: it is a fact about `progression.toml`, constant for the whole
    // session, not a fact about this moment. A record says what happened; a
    // machine reading the log back derives the rest from the content, exactly as
    // it does for a recipe's duration.
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
    use super::*;
    use crate::Sim;
    use orbs_render::Value;

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

        let said = messages(&sim);
        assert!(
            said.iter().any(|line| line.contains("16")),
            "it did not name what is next: {said:?}",
        );
    }

    #[test]
    fn a_spell_cannot_seize_the_screen() {
        // **The soft-lock.** `weave` opens a whole screen, which is `unfurl`'s
        // objection with more of the window behind it — inside a `repeat` a
        // spell re-seizes it faster than Escape can give it back, on the orb's
        // clock rather than the player's. Driven through a real cast rather than
        // by calling `may_issue`, because the list is a `matches!` and nothing
        // makes the compiler check it.
        // **Stood in the laboratory before the spell is written**, or it is
        // written for the root and the `attend` ends the invocation before the
        // line under test ever runs — which passes for the wrong reason.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.write_spell("grab", &["weave".to_owned()]);
        sim.step();
        sim.submit("invoke grab");
        sim.step_n(4);

        assert!(!sim.has_weaving(), "a spell opened the screen");
        assert!(
            messages(&sim)
                .iter()
                .any(|line| line.contains("will not do")),
            "it was refused silently: {:?}",
            messages(&sim),
        );
    }

    #[test]
    fn a_topped_out_tower_is_not_told_to_work_toward_nothing() {
        // `next` is `None` once every authored threshold is passed, and the
        // sentence about what is next would then name **0** — a target that
        // reads as reached and is not a target at all.
        let mut sim = Sim::new(1);
        crate::tower::credit(sim.world_mut(), 10_000);
        sim.submit("weave");
        sim.step();

        let said = messages(&sim);
        assert!(
            !said.iter().any(|line| line.contains(" 0")),
            "it offered a threshold of nothing: {said:?}",
        );
    }
}
