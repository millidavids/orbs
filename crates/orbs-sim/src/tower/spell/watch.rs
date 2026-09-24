//! What a spell can wait on, read off the record stream.
//!
//! §3 forbids unlogged output, so the record stream *is* the log and every
//! consequence is already on it. A spell watching for "the mortar finished"
//! reads the records the player does — §8.1's *no automation driven by hidden
//! state*. Forging the event and forging the evidence are the same act, which
//! is what makes `verify` meaningful.
//!
//! This needed [`FieldName::At`] first: eleven emit sites put the instrument in
//! whichever field was nearest, and `Name` meant a verb, an instrument or a
//! product depending on who wrote the line.

use bevy_ecs::prelude::{Entity, World};
use orbs_render::{FieldName, Record, RecordKind, Value};

use crate::parser::{Condition, SpellState};
use crate::tower::{self, Cwd};

/// Something that happened somewhere.
///
/// Three strings rather than an enum of event kinds: a player writes `wait for
/// the mortar` or `wait for ground-sage` and the orb works out which sense they
/// meant (§6). An enum would ask the player to know a taxonomy nobody taught.
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
    /// Either the place or the subject, because both are natural. Case-folded
    /// through the helper `sift` uses, so a search and a wait cannot disagree.
    #[must_use]
    pub fn names(&self, wanted: &str) -> bool {
        let wanted = crate::parser::leaf(wanted);
        orbs_render::contains_ignoring_case(&self.at, wanted)
            || orbs_render::contains_ignoring_case(&self.what, wanted)
    }
}

/// The event `record` reports, if it reports one.
///
/// A completion that says where it happened. An echo, a status row or the orb
/// speaking did not *occur*, so a spell waiting on one would be waiting on the
/// log rather than on the world.
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
/// Three answers, because two were a silent bug: `false` for a name the tower
/// lacks made `if mortar is empty` answer no for ever and take the `else` every
/// time, which reads like an inverted condition. §8's *Referent missing*
/// instead. §6's matcher is not involved — the names arrive already resolved
/// against the room the spell was cast in (`spell::compile`).
///
/// Strict rather than three-valued (§19): one missing place makes the whole
/// question unanswerable even where the other half said yes, and the runner
/// runs neither branch on `None`.
///
/// The names come back with the answer rather than being logged here: this is a
/// `&World` read with no opinion about prose.
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
        // Every operand is asked even after one has failed: §8.1's rule is that
        // the culprit is never anonymous, not that one culprit is enough.
        //
        // One `None` makes the whole thing `None`, even where the rest would
        // have settled it — strict rather than three-valued (§19).
        Condition::All(items) => {
            let answers = every(world, items, missing)?;
            Some(answers.into_iter().all(|answer| answer))
        }
        Condition::Any(items) => {
            let answers = every(world, items, missing)?;
            Some(answers.into_iter().any(|answer| answer))
        }
        // A named child, and deliberately not `tower::holdings`, which skips
        // `Nameable(NounKind::Sense)` — every maze reading is a `Sense` child
        // from `tower::raise_reading`, so `holdings` would answer *no* to
        // `if north has passage` for ever. The count then comes from the node's
        // own `Stock`.
        Condition::Has {
            place,
            thing,
            count,
            bound,
        } => {
            let at = find(world, place, missing)?;
            // Asked unconditionally: a comparison against a place the tower
            // lacks must report that name even when this side settles the
            // answer (§8.1).
            let want = worth(world, count, thing, missing)?;
            let many = many_at(world, at, thing);
            // Strict against a world read, inclusive against a number: English,
            // since `has 2 or fewer marks` includes two and `has fewer marks
            // than east` does not.
            //
            // Asked of the grammar, not of the variant: `matches!(count,
            // Elsewhere(_))` would drop the three expression variants to
            // inclusive, so `plus 0` would change a sentence's meaning.
            let strict = count.strict();
            Some(match bound {
                crate::parser::Bound::AtLeast if strict => many > want,
                crate::parser::Bound::AtMost if strict => many < want,
                crate::parser::Bound::AtLeast => many >= want,
                crate::parser::Bound::AtMost => many <= want,
                crate::parser::Bound::Exactly => many == want,
            })
        }
        // The panel's word, not `busy()`: the two agree for the four instruments
        // that consume Focus and differ for the athanor, whose fire is `Burning`
        // rather than `Working` — so `if athanor is idle` was yes while it
        // burned. See [`State::is_busy`](tower::State::is_busy).
        //
        // `Empty` stays a question about contents rather than the panel's
        // `State::Empty`, which the athanor never reports, so reading it there
        // would make `if athanor is empty` false however bare it was.
        Condition::Is { place, state } => {
            let at = find(world, place, missing)?;
            Some(match state {
                SpellState::Idle => !tower::state_at(world, at).is_busy(),
                SpellState::Working => tower::state_at(world, at).is_busy(),
                // A satchel answers from its queue, not its children: a queue
                // is an ordered multiset and `Stock` collapses duplicates.
                // Without this arm `if the satchel is empty` is true of a full
                // satchel, silently (§19).
                SpellState::Empty => world.get::<tower::Satchel>(at).map_or_else(
                    || tower::children_of(world, at).is_empty(),
                    tower::Satchel::is_empty,
                ),
            })
        }
    }
}

/// What the far side of a comparison comes to.
///
/// Recursive, because the far side is a small expression: `double the enemy has
/// mettle plus 6` is a tree of world reads, doublings and additions. Every name
/// inside is resolved unconditionally — §8.1 wants the culprit named.
///
/// Saturating throughout: both a count off a node and a number the player typed
/// are reachable with values that overflow in a debug build, and a question
/// that panicked would take the tower down over a sentence.
fn worth(
    world: &World,
    count: &crate::parser::Quantity,
    thing: &str,
    missing: &mut Vec<String>,
) -> Option<u32> {
    match count {
        crate::parser::Quantity::Count(count) => Some(*count),
        crate::parser::Quantity::Elsewhere(other) => {
            let there = find(world, other, missing)?;
            Some(many_at(world, there, thing))
        }
        // The far side's own reading, all this variant adds: `has fewer
        // quintessence than the d20 has cost` asks a different word over there.
        crate::parser::Quantity::Of { place, thing } => {
            let there = find(world, place, missing)?;
            Some(many_at(world, there, thing))
        }
        crate::parser::Quantity::Doubled(of) => {
            Some(worth(world, of, thing, missing)?.saturating_mul(2))
        }
        crate::parser::Quantity::Plus { of, by } => {
            Some(worth(world, of, thing, missing)?.saturating_add(*by))
        }
    }
}

/// How many of `thing` are at `at`, where absent is nought and means it.
///
/// One read, used by both sides of a comparison, so `north has fewer marks than
/// east` is one question rather than two notions of counting — the arithmetic
/// `tower::build::raise_count` promises.
///
/// Absent is nought: `or fewer` false of an absent thing would make `if the
/// dispensary has 2 or fewer sage` false with no sage at all, the one case a
/// restock guard is written for.
///
/// A thing with no `Stock` is one of it: a reading, a file, a spell.
fn many_at(world: &World, at: Entity, thing: &str) -> u32 {
    // A named child, not `tower::holdings` — see the note on `Condition::Has`.
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
/// Collected before it is folded, so every operand is asked even once the
/// answer is settled: §8.1 wants no anonymous culprit.
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
/// Named once, however many operands mention it, or a question repeating a bad
/// name would say the same sentence twice about one mistake.
fn find(world: &World, named: &str, missing: &mut Vec<String>) -> Option<Entity> {
    let cwd = world.resource::<Cwd>().0;
    let leaf = crate::parser::leaf(named).to_owned();
    let here = |at: Entity| {
        tower::children_of(world, at).into_iter().find(|node| {
            world
                .get::<tower::Name>(*node)
                .is_some_and(|name| name.0 == leaf)
        })
    };
    // The room first, then the arsenal — but only a *store*. `for each store`
    // binds a cursor to an arsenal item and the next line asks it a question,
    // so the arsenal has to be reachable from wherever the spell stands (§19).
    //
    // Narrowed to `Grouped(STORE)` because an unrestricted fallback silences
    // complaints for every other condition: a laboratory spell asking about an
    // absent clarity would bind to the arsenal's shelf and read false for ever.
    let found = here(cwd).or_else(|| {
        tower::keep(world).and_then(here).filter(|node| {
            world
                .get::<tower::Grouped>(*node)
                .is_some_and(|group| group.0 == tower::STORE)
        })
    });
    if found.is_none() && !missing.iter().any(|already| *already == named) {
        missing.push(named.to_owned());
    }
    found
}

fn text(record: &Record<'_>, field: FieldName) -> Option<String> {
    match record.field(field)? {
        Value::Text(text) => Some(text.to_owned()),
        // A number is never a thing a spell waits *on*: `qty: 3` would match a
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
        // Both are natural, and a player should not have to know which sense
        // the orb keeps.
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
        // `if athanor is idle` fired while it burned charcoal. The fire is
        // `Burning`, not `Working`, because the athanor takes no Focus — so
        // `busy()` could not see it either, and both words were wrong at once.
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
        // `Banked` is fuel put by, not work in progress, so a spell waiting for
        // a free athanor must not wait on a damped one for ever.
        let mut sim = Sim::new(1);
        for line in ["attend laboratory", "kindle charcoal", "stop athanor"] {
            sim.submit(line);
            sim.step();
        }

        assert_eq!(asking(&sim, "athanor", SpellState::Idle), Some(true));
    }

    #[test]
    fn an_instrument_mid_run_is_still_working() {
        // The other four answer exactly as they did through `busy()`, which
        // makes the fix above a widening rather than a replacement.
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

    /// Whether the tower answers a comparison whose far side is an expression.
    fn weighing(
        sim: &Sim,
        place: &str,
        thing: &str,
        bound: crate::parser::Bound,
        far: crate::parser::Quantity,
    ) -> Option<bool> {
        holds(
            sim.world(),
            &Condition::Has {
                place: place.to_owned(),
                thing: thing.to_owned(),
                count: far,
                bound,
            },
        )
        .0
    }

    /// The arithmetic, evaluated rather than merely parsed.
    ///
    /// `tests/questions.rs` proves the far side round-trips but never runs it,
    /// so a tree computing the wrong number passes every property there.
    #[test]
    fn the_far_side_of_a_comparison_does_its_arithmetic() {
        use crate::parser::{Bound, Quantity};
        let mut sim = Sim::new(1);
        walked(&mut sim);

        // Derived, never assumed: a walled way publishes no `marks`, so which
        // way the fixture walked more is a fact about the seed's maze.
        let marks = |way: &str| {
            super::many_at(
                sim.world(),
                find(sim.world(), way, &mut Vec::new()).expect("a way"),
                "marks",
            )
        };
        let (fewer, more) = if marks("south") < marks("north") {
            ("south", "north")
        } else {
            ("north", "south")
        };
        assert_ne!(
            marks(fewer),
            marks(more),
            "the fixture walked both ways alike, so this cannot tell a working \
             comparison from one that always says no",
        );

        // `plus` shifts the threshold by what it says: the busier way is
        // strictly ahead until the other is given the gap.
        let by = marks(more) - marks(fewer);
        assert_eq!(
            weighing(
                &sim,
                more,
                "marks",
                Bound::AtLeast,
                Quantity::Plus {
                    of: Box::new(Quantity::Elsewhere(fewer.to_owned())),
                    by: by - 1,
                },
            ),
            Some(true),
            "{more} is not more than {fewer} plus {}",
            by - 1,
        );
        assert_eq!(
            weighing(
                &sim,
                more,
                "marks",
                Bound::AtLeast,
                Quantity::Plus {
                    of: Box::new(Quantity::Elsewhere(fewer.to_owned())),
                    by,
                },
            ),
            Some(false),
            "the comparison is strict, so equal is not more",
        );

        // `double` is asked *before* `plus` wraps it — `double north plus 0`
        // and `double north` are one question.
        assert_eq!(
            weighing(
                &sim,
                more,
                "marks",
                Bound::AtMost,
                Quantity::Doubled(Box::new(Quantity::Elsewhere(fewer.to_owned()))),
            ),
            weighing(
                &sim,
                more,
                "marks",
                Bound::AtMost,
                Quantity::Plus {
                    of: Box::new(Quantity::Doubled(Box::new(Quantity::Elsewhere(
                        fewer.to_owned()
                    )))),
                    by: 0,
                },
            ),
            "`plus 0` changed the answer, so `strict` is reading the variant",
        );

        // A nested name is still reported: bailing at the first `None` would
        // hide it, which is the rule `every` follows one level up.
        let (answer, missing) = holds(
            sim.world(),
            &Condition::Has {
                place: more.to_owned(),
                thing: "marks".to_owned(),
                count: Quantity::Doubled(Box::new(Quantity::Of {
                    place: "nowhere-at-all".to_owned(),
                    thing: "marks".to_owned(),
                })),
                bound: Bound::AtLeast,
            },
        );
        assert_eq!(answer, None, "a question over a missing place was answered");
        assert!(
            missing.iter().any(|name| name == "nowhere-at-all"),
            "the nested name was not reported: {missing:?}",
        );
    }

    /// `Of` reads a different word over there, which is why it exists —
    /// `Elsewhere` can only ask the same reading on both sides.
    #[test]
    fn a_comparison_can_name_a_different_reading_on_the_far_side() {
        use crate::parser::{Bound, Quantity};
        let mut sim = Sim::new(1);
        walked(&mut sim);

        // Whichever way this seed's maze let the fixture walk: naming one by
        // hand would assert against the seed rather than the language.
        let marks = |way: &str| {
            super::many_at(
                sim.world(),
                find(sim.world(), way, &mut Vec::new()).expect("a way"),
                "marks",
            )
        };
        let walked = ["south", "north", "east", "west"]
            .into_iter()
            .find(|way| marks(way) > 0)
            .expect("the fixture walked somewhere");

        // Against a word not published there at all: absent is nought, so any
        // walked way has strictly more than none of it.
        assert_eq!(
            weighing(
                &sim,
                walked,
                "marks",
                Bound::AtLeast,
                Quantity::Of {
                    place: "north".to_owned(),
                    thing: "nothing-is-called-this".to_owned(),
                },
            ),
            Some(true),
            "an absent far-side reading did not count as nought",
        );

        // ...and it really is reading the *far* side's own word: asked for
        // `marks` over there instead, the same question answers differently.
        assert_eq!(
            weighing(
                &sim,
                walked,
                "marks",
                Bound::AtMost,
                Quantity::Of {
                    place: walked.to_owned(),
                    thing: "marks".to_owned(),
                },
            ),
            Some(false),
            "a way has strictly fewer marks than itself",
        );
    }

    #[test]
    fn a_comparison_reads_both_sides_off_the_same_published_count() {
        // `raise_count` puts a maze's `marks` on `Stock` so one arithmetic
        // answers both `has 2 or more marks` and `has 4 fragment`; this makes
        // the far side a world read too. It reads the published `Sense`
        // children rather than `Maze::marks` because of §8.1.
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
        // Strict, as the English says: `north has more marks than north` is
        // false. The asymmetry against `has 2 or more marks`, which does
        // include two, is English rather than an inconsistency.
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
        // applies to it too.
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
        // Three answers, not two — the distinction `holds` exists for.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();

        assert_eq!(asking(&sim, "gatehouse", SpellState::Idle), None);
    }

    #[test]
    fn the_echo_of_a_command_is_not_an_event() {
        // A spell waits on the world, not on the log. An echo, a status row and
        // the orb speaking are all records; none of them happened anywhere.
        let mut sim = Sim::new(1);
        sim.submit("status");
        sim.step();
        assert!(
            events(&sim).is_empty(),
            "something that did not happen was reported as an event",
        );
    }
}
