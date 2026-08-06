//! What a spell can wait on, read off the record stream.
//!
//! # Events are records, and there is no second stream
//!
//! §3 forbids unlogged output, so the record stream **is** the log — the
//! scrollback, the file a player `peruse`s, and what `sift` filters are one
//! stream read several ways. Every consequence in the game is already on it.
//!
//! So a spell watching for "the mortar finished" reads the same records the
//! player does, and the strongest argument for that is §8.1's: **no automation
//! can be driven by hidden state.** A private event bus would let a spell react
//! to something the player cannot see or audit, and log poisoning — a *keystone*
//! mechanic — would have nothing to bite on. Here, forging the event and forging
//! the evidence are the same act, which is what makes `verify` meaningful.
//!
//! # Why this needed [`FieldName::At`] first
//!
//! Eleven emit sites put the instrument in whichever field was nearest, and
//! `Name` meant a verb, an instrument, a product or a list of products depending
//! on who wrote the line. That is survivable while a person is reading; it is
//! not once a **spell** is, because *"has the mortar finished?"* has to be one
//! question with one answer.

use bevy_ecs::prelude::World;
use orbs_render::{FieldName, Record, RecordKind, Value};

use crate::parser::{Condition, SpellState};
use crate::tower::{self, Cwd};

/// Something that happened somewhere.
///
/// Deliberately three strings rather than an enum of event kinds. A player
/// writes `wait for the mortar` or `wait for ground-sage` — they name a **thing**
/// and the orb works out which of its senses they meant, which is §6's whole
/// posture applied one layer in. An enum would make the spell language ask the
/// player to know a taxonomy the game never taught them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// Where it happened — an instrument, always. [`FieldName::At`].
    pub at: String,
    /// What it concerned: a product, a reagent, a verb.
    pub what: String,
    /// The condition it left behind: `collected`, `cold`, `emptied`.
    pub state: String,
}

impl Event {
    /// Whether this is the thing a spell named.
    ///
    /// Matched against **either** the place or the subject, because both are
    /// natural: `wait for the mortar` names where, `wait for ground-sage` names
    /// what. Case-folded through the same helper `sift` uses, so a search and a
    /// wait cannot disagree about what counts as a match.
    #[must_use]
    pub fn names(&self, wanted: &str) -> bool {
        let wanted = crate::parser::leaf(wanted);
        orbs_render::contains_ignoring_case(&self.at, wanted)
            || orbs_render::contains_ignoring_case(&self.what, wanted)
    }
}

/// The event `record` reports, if it reports one.
///
/// **A completion that says where it happened.** Everything else — an echo, a
/// status row, the orb speaking — is not something that *occurred* in the
/// laboratory, and a spell waiting on one would be waiting on the log rather
/// than on the world.
#[must_use]
pub fn watch(record: &Record<'_>) -> Option<Event> {
    if record.kind() != RecordKind::Completion {
        return None;
    }
    let at = text(record, FieldName::At)?;
    Some(Event {
        at,
        what: text(record, FieldName::Name).unwrap_or_default(),
        state: text(record, FieldName::State).unwrap_or_default(),
    })
}

/// Whether the tower currently answers `condition` yes, or `None` if the place
/// it asks about is not here.
///
/// # Three answers, because two were a silent bug
///
/// §6's matcher is deliberately not involved: a fuzzy answer here would decide
/// what a laboratory does while nobody is watching, and the place name arrives
/// **already canonical**, resolved against the room when the spell was saved
/// (`scribe::written_condition`).
///
/// So a name that does not match is not a near miss to be guessed at — it is a
/// place the tower does not have, which is §8's *Referent missing* and not the
/// same thing as the answer being no. It returned `false` for both, and
/// `if mortar is empty` — written before an `if` was canonicalised — answered no
/// for ever and took the `else` every time. That reads exactly like the
/// condition being inverted, which is how it was reported.
#[must_use]
pub fn holds(world: &World, condition: &Condition) -> Option<bool> {
    let cwd = world.resource::<Cwd>().0;
    let find = |named: &str| {
        let leaf = crate::parser::leaf(named).to_owned();
        tower::children_of(world, cwd).into_iter().find(|node| {
            world
                .get::<tower::Name>(*node)
                .is_some_and(|name| name.0 == leaf)
        })
    };

    let at = find(condition.place())?;
    Some(match condition {
        Condition::Has { thing, .. } => tower::children_of(world, at).into_iter().any(|held| {
            world
                .get::<tower::Name>(held)
                .is_some_and(|name| name.0 == *thing)
        }),
        Condition::Is { state, .. } => match state {
            SpellState::Idle => tower::busy(world, at).is_none(),
            SpellState::Working => tower::busy(world, at).is_some(),
            SpellState::Empty => tower::children_of(world, at).is_empty(),
        },
    })
}

fn text(record: &Record<'_>, field: FieldName) -> Option<String> {
    match record.field(field)? {
        Value::Text(text) => Some(text.to_owned()),
        // A number is never a thing a spell waits *on*. `wait for 3` is not a
        // sentence, and treating a count as a name would let `qty: 3` match a
        // spell waiting for something called 3.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    /// Every event a session produced, in order.
    fn events(sim: &Sim) -> Vec<Event> {
        sim.scrollback()
            .records()
            .iter()
            .filter_map(|record| watch(&record))
            .collect()
    }

    #[test]
    fn a_finished_grind_is_an_event_naming_the_mortar_and_what_it_made() {
        let mut sim = Sim::new(1);
        for line in [
            "attend laboratory",
            "kindle charcoal",
            "grind sage",
            "meditate 30",
        ] {
            sim.submit(line);
            sim.step();
        }

        let events = events(&sim);
        assert!(
            events.iter().any(|event| event.names("mortar_and_pestle")),
            "nothing named the mortar: {events:?}",
        );
        assert!(
            events.iter().any(|event| event.names("ground-sage")),
            "nothing named what it made: {events:?}",
        );
    }

    #[test]
    fn a_spell_can_name_the_place_or_the_thing() {
        // `wait for the mortar` and `wait for ground-sage` are both natural, and
        // a player should not have to know which sense the orb keeps.
        let event = Event {
            at: "mortar_and_pestle".to_owned(),
            what: "ground-sage".to_owned(),
            state: "husks".to_owned(),
        };
        assert!(event.names("mortar_and_pestle"));
        assert!(event.names("ground-sage"));
        assert!(!event.names("alembic"));
    }

    #[test]
    fn the_echo_of_a_command_is_not_an_event() {
        // A spell waits on the **world**, not on the log. An echo, a status row
        // and the orb speaking are all records; none of them happened anywhere.
        let mut sim = Sim::new(1);
        sim.submit("status");
        sim.step();
        assert!(
            events(&sim).is_empty(),
            "something that did not happen was reported as an event",
        );
    }
}
