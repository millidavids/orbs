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

use bevy_ecs::prelude::{Entity, World};
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

/// What the tower answers, and every name in the question it could not place.
///
/// # Three answers, because two were a silent bug
///
/// §6's matcher is deliberately not involved: a fuzzy answer here would decide
/// what a laboratory does while nobody is watching, and the names arrive
/// **already resolved**, fixed against the room when the spell was cast
/// (`spell::compile`).
///
/// So a name that does not match is not a near miss to be guessed at — it is a
/// place the tower does not have, which is §8's *Referent missing* and not the
/// same thing as the answer being no. It returned `false` for both, and
/// `if mortar is empty` — written before an `if` resolved its names — answered
/// no for ever and took the `else` every time. That reads exactly like the
/// condition being inverted, which is how it was reported.
///
/// # Strict, so an `or` cannot answer over a name nobody can find
///
/// One missing place makes the **whole** question unanswerable, even where the
/// other half of an `or` already said yes. The alternative — three-valued logic,
/// answering from the half that resolved — was considered and rejected (§19):
/// an instrument that has stopped existing is §8.1's substitution surface, and a
/// spell carrying on over it is exactly the thing that must not pass quietly.
/// The runner runs **neither** branch when this answers `None`, so a question
/// nobody can answer decides nothing.
///
/// The names come back with the answer rather than being logged here: this is a
/// `&World` read with no opinion about prose, and the runner needs them to say
/// which word it could not place.
#[must_use]
pub fn holds(world: &World, condition: &Condition) -> (Option<bool>, Vec<String>) {
    let mut missing = Vec::new();
    let answer = ask(world, condition, &mut missing);
    (answer, missing)
}

/// One question, gathering the places it could not find as it goes.
fn ask(world: &World, condition: &Condition, missing: &mut Vec<String>) -> Option<bool> {
    match condition {
        Condition::Not(inner) => ask(world, inner, missing).map(|answer| !answer),
        // **Every operand is asked, even after one has failed.** Short-circuiting
        // would report the first bad name and hide the second, so a player fixing
        // a question would be sent back for the next one — and §8.1's rule is
        // that the culprit is never anonymous, not that one culprit is enough.
        //
        // **And one `None` makes the whole thing `None`**, even where the rest
        // would have settled it — an `and` with a false operand beside a missing
        // name, an `or` with a true one. That is the strict rule, chosen over
        // three-valued logic deliberately (§19): a name the tower cannot place is
        // a spell that has stopped describing the world it runs in, and carrying
        // on over it is what must not pass quietly.
        Condition::All(items) => {
            let answers = every(world, items, missing)?;
            Some(answers.into_iter().all(|answer| answer))
        }
        Condition::Any(items) => {
            let answers = every(world, items, missing)?;
            Some(answers.into_iter().any(|answer| answer))
        }
        // **A named child, and deliberately not `tower::holdings`.** `holdings`
        // skips `Nameable(NounKind::Sense)`, which is right for a shelf and fatal
        // here: every maze reading is a `Sense` child — `passage`, `wall`,
        // `walked`, `twice`, `exit`, `back`, `spoil` and the errand word all come
        // from `tower::raise_reading`. Asking `holdings` would answer *no* to
        // `if north has passage` for ever and delete the archive's whole
        // automation pillar, while looking like reuse. `tower::build`'s own note
        // on `raise_reading` says it outright: *"a named child, which is the one
        // read the language has."*
        //
        // The count then comes from the node's own `Stock`, which is where a
        // quantity lives when there is one.
        Condition::Has {
            place,
            thing,
            count,
            bound,
        } => {
            let at = find(world, place, missing)?;
            // **The other side is asked first, and unconditionally.** A
            // comparison against a place the tower lacks must report that name
            // even when this side already settles the answer — the same rule
            // `every` follows for connectives, and the same reason: §8.1 wants
            // the culprit named, not the first culprit.
            let want = match count {
                crate::parser::Quantity::Count(count) => *count,
                crate::parser::Quantity::Elsewhere(other) => {
                    let there = find(world, other, missing)?;
                    many_at(world, there, thing)
                }
            };
            let many = many_at(world, at, thing);
            // **Strict against another place, inclusive against a number**, and
            // the asymmetry is English rather than an inconsistency: `has 2 or
            // fewer marks` includes two and `has fewer marks than east` does
            // not. `Quantity::Elsewhere`'s doc records that the third
            // comparative — *at least as many* — is deliberately absent, because
            // `not … fewer … than` already says it.
            let strict = matches!(count, crate::parser::Quantity::Elsewhere(_));
            Some(match bound {
                crate::parser::Bound::AtLeast if strict => many > want,
                crate::parser::Bound::AtMost if strict => many < want,
                crate::parser::Bound::AtLeast => many >= want,
                crate::parser::Bound::AtMost => many <= want,
                crate::parser::Bound::Exactly => many == want,
            })
        }
        // **The panel's word, not `busy()`.** The two are the same answer for the
        // four instruments that consume Focus and differ for the athanor, whose
        // fire is `Burning` rather than `Working` — so `if athanor is idle` was
        // yes while it burned. See [`State::is_busy`](tower::State::is_busy).
        //
        // `Empty` stays a question about **contents** rather than the panel's
        // `State::Empty`: the athanor never reports that word — it is cold, or
        // charged, or alight — and reading it from the panel would make
        // `if athanor is empty` false however bare it was.
        Condition::Is { place, state } => {
            let at = find(world, place, missing)?;
            Some(match state {
                SpellState::Idle => !tower::state_at(world, at).is_busy(),
                SpellState::Working => tower::state_at(world, at).is_busy(),
                // **A satchel answers from its queue, not from its children.**
                // It has none either way — the names ride a `VecDeque` on the
                // node, because a queue is an ordered multiset and `Stock`
                // collapses duplicates and has no order — so without this arm
                // `if the satchel is empty` is *true of a full satchel*, for
                // ever and silently. That is the shape §19 records twice already
                // (`is idle` in the menagerie, `is empty` in the lens): a guard
                // that answers before the loop can run, and a spell that does
                // nothing without saying so.
                SpellState::Empty => world.get::<tower::Satchel>(at).map_or_else(
                    || tower::children_of(world, at).is_empty(),
                    tower::Satchel::is_empty,
                ),
            })
        }
    }
}

/// How many of `thing` are at `at`, where absent is nought and means it.
///
/// **One read, used by both sides of a comparison**, which is what makes
/// `north has fewer marks than east` an honest question rather than two notions
/// of counting placed side by side. It is also the arithmetic
/// `tower::build::raise_count` promises: *"`has 2 or more marks` is answered by
/// the same arithmetic that answers `has 4 fragment`, rather than by a second
/// notion of how many of something there is."*
///
/// **Absent is nought, and the other reading was refused.** A comparison needs
/// something to count, so an argument exists that `or fewer` should be false of
/// an absent thing — but then `if the dispensary has 2 or fewer sage` is **false
/// with no sage at all**, which is the one case a restock guard is written for.
/// A player who writes that sentence gets what it says.
///
/// A thing with no `Stock` is one of it: a reading, a file, a spell.
fn many_at(world: &World, at: Entity, thing: &str) -> u32 {
    // **A named child, and deliberately not `tower::holdings`** — see the note
    // on `Condition::Has` above. Every maze and ward reading is a
    // `Nameable(NounKind::Sense)` child, which `holdings` skips.
    let held = tower::children_of(world, at).into_iter().find(|held| {
        world
            .get::<tower::Name>(*held)
            .is_some_and(|name| name.0 == *thing)
    });
    match held.map(|node| world.get::<tower::Stock>(node)) {
        Some(Some(tower::Stock::Endless)) => u32::MAX,
        Some(Some(tower::Stock::Counted(units))) => *units,
        Some(None) => 1,
        None => 0,
    }
}

/// Every operand's answer, or `None` if any of them had none.
///
/// **Collected before it is folded**, so every operand is asked even once the
/// answer is settled. Short-circuiting would report the first bad name and hide
/// the second, and a player fixing a question would be sent back for the next
/// one — §8.1's rule is that the culprit is never anonymous, not that one
/// culprit is enough.
fn every(world: &World, items: &[Condition], missing: &mut Vec<String>) -> Option<Vec<bool>> {
    items
        .iter()
        .map(|item| ask(world, item, missing))
        .collect::<Vec<_>>()
        .into_iter()
        .collect()
}

/// The place `named`, or `None` with the name noted as one the tower lacks.
///
/// Named **once**, however many operands mention it: a question repeating a bad
/// name would otherwise say the same sentence twice about one mistake.
fn find(world: &World, named: &str, missing: &mut Vec<String>) -> Option<Entity> {
    let cwd = world.resource::<Cwd>().0;
    let leaf = crate::parser::leaf(named).to_owned();
    let found = tower::children_of(world, cwd).into_iter().find(|node| {
        world
            .get::<tower::Name>(*node)
            .is_some_and(|name| name.0 == leaf)
    });
    if found.is_none() && !missing.iter().any(|already| *already == named) {
        missing.push(named.to_owned());
    }
    found
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

    /// Whether the tower answers `place is state` yes, now.
    fn asking(sim: &Sim, place: &str, state: SpellState) -> Option<bool> {
        holds(
            sim.world(),
            &Condition::Is {
                place: place.to_owned(),
                state,
            },
        )
        .0
    }

    #[test]
    fn a_burning_athanor_is_working_rather_than_idle() {
        // **Reported from a spell**: `if athanor is idle` fired while it was
        // burning charcoal, which is not what anybody reads that word to mean.
        //
        // The cause is a deliberate decision one layer down. The fire is
        // `Burning` and pointedly **not** `Working`, because the athanor takes no
        // Focus and nothing counting the production pool may see it — so
        // `busy()`, which reads `Working` and `Triaging`, could not see it
        // either. Both words were wrong at once: idle said yes, working said no,
        // and the panel beside them drew `at burning` the whole time.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();

        assert_eq!(
            asking(&sim, "athanor", SpellState::Idle),
            Some(true),
            "a cold athanor is idle",
        );

        sim.submit("kindle charcoal");
        sim.step();

        assert_eq!(
            asking(&sim, "athanor", SpellState::Idle),
            Some(false),
            "the fire was invisible to the word that asks about it",
        );
        assert_eq!(
            asking(&sim, "athanor", SpellState::Working),
            Some(true),
            "the two words disagreed about the same fire",
        );
    }

    #[test]
    fn a_damped_athanor_is_idle_again() {
        // `Banked` is fuel put by, not work in progress. A spell waiting for the
        // athanor to be free must not wait on a damped one for ever.
        let mut sim = Sim::new(1);
        for line in ["attend laboratory", "kindle charcoal", "stop athanor"] {
            sim.submit(line);
            sim.step();
        }

        assert_eq!(asking(&sim, "athanor", SpellState::Idle), Some(true));
    }

    #[test]
    fn an_instrument_mid_run_is_still_working() {
        // The other four instruments answer exactly as they did through
        // `busy()`, which is what makes the fix above a widening rather than a
        // replacement.
        let mut sim = Sim::new(1);
        for line in ["attend laboratory", "grind sage"] {
            sim.submit(line);
            sim.step();
        }

        assert_eq!(
            asking(&sim, "mortar_and_pestle", SpellState::Working),
            Some(true),
        );
        assert_eq!(
            asking(&sim, "mortar_and_pestle", SpellState::Idle),
            Some(false),
        );
    }

    /// A maze, walked so the four ways carry different mark counts.
    fn walked(sim: &mut Sim) {
        for line in [
            "attend archive",
            "research",
            "follow south",
            "follow north",
            "follow south",
        ] {
            sim.submit(line);
            sim.step();
        }
    }

    /// Whether the tower answers `place has <cmp> thing than other` yes, now.
    fn comparing(
        sim: &Sim,
        place: &str,
        thing: &str,
        bound: crate::parser::Bound,
        other: &str,
    ) -> Option<bool> {
        holds(
            sim.world(),
            &Condition::Has {
                place: place.to_owned(),
                thing: thing.to_owned(),
                count: crate::parser::Quantity::Elsewhere(other.to_owned()),
                bound,
            },
        )
        .0
    }

    #[test]
    fn a_comparison_reads_both_sides_off_the_same_published_count() {
        // **The step the whole language overhaul rests on**, and it is small
        // because the tower was already shaped for it: `raise_count` puts a
        // maze's `marks` on `Stock` so that *"`has 2 or more marks` is answered
        // by the same arithmetic that answers `has 4 fragment`"*. This makes the
        // **other** side of that arithmetic a world read too.
        //
        // §8.1 is why it reads the published `Sense` children rather than
        // `Maze::marks`: forging the event and forging the evidence have to stay
        // the same act, or log poisoning has nothing to bite on.
        use crate::parser::Bound;
        let mut sim = Sim::new(1);
        walked(&mut sim);

        // South was walked twice and north once, whichever maze this seed drew.
        let south = super::many_at(
            sim.world(),
            find(sim.world(), "south", &mut Vec::new()).expect("a way south"),
            "marks",
        );
        let north = super::many_at(
            sim.world(),
            find(sim.world(), "north", &mut Vec::new()).expect("a way north"),
            "marks",
        );
        assert_ne!(
            south, north,
            "the fixture walked the two ways the same number of times, so it \
             cannot tell a working comparison from one that always says no",
        );

        let (fewer, more) = if south < north {
            ("south", "north")
        } else {
            ("north", "south")
        };
        assert_eq!(
            comparing(&sim, fewer, "marks", Bound::AtMost, more),
            Some(true),
            "{fewer} has fewer marks than {more} and the tower said otherwise",
        );
        assert_eq!(
            comparing(&sim, more, "marks", Bound::AtMost, fewer),
            Some(false),
            "the comparison answered the same either way round",
        );
        assert_eq!(
            comparing(&sim, more, "marks", Bound::AtLeast, fewer),
            Some(true),
        );
    }

    #[test]
    fn a_comparison_against_itself_is_equal_and_not_more() {
        // **Strict, which is what the English says.** `north has more marks than
        // north` is false; `as many … as` is what asks the other question. The
        // asymmetry against `has 2 or more marks` — which *does* include two —
        // is English rather than an inconsistency, and `Quantity::Elsewhere`
        // records it.
        use crate::parser::Bound;
        let mut sim = Sim::new(1);
        walked(&mut sim);

        assert_eq!(
            comparing(&sim, "south", "marks", Bound::Exactly, "south"),
            Some(true),
        );
        assert_eq!(
            comparing(&sim, "south", "marks", Bound::AtLeast, "south"),
            Some(false),
            "a place had strictly more marks than itself",
        );
        assert_eq!(
            comparing(&sim, "south", "marks", Bound::AtMost, "south"),
            Some(false),
        );
    }

    #[test]
    fn a_comparison_against_a_place_the_tower_lacks_answers_nothing() {
        // The far side resolves like the near one, so §8's *Referent missing*
        // applies to it — a spell comparing against a room that is not there has
        // stopped describing the world it runs in, and must not carry on.
        use crate::parser::Bound;
        let mut sim = Sim::new(1);
        walked(&mut sim);

        let (answer, missing) = holds(
            sim.world(),
            &Condition::Has {
                place: "north".to_owned(),
                thing: "marks".to_owned(),
                count: crate::parser::Quantity::Elsewhere("gatehouse".to_owned()),
                bound: Bound::AtMost,
            },
        );
        assert_eq!(answer, None);
        assert!(
            missing.iter().any(|name| name == "gatehouse"),
            "the name it could not place went unreported: {missing:?}",
        );
    }

    #[test]
    fn a_question_about_a_place_the_tower_does_not_have_is_not_an_answer() {
        // Three answers, not two — the distinction `holds` exists for. Kept
        // beside the states because it is the same call and the same `Option`.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();

        assert_eq!(asking(&sim, "gatehouse", SpellState::Idle), None);
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
