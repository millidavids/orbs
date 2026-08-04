//! What a command actually does.
//!
//! The seam the domains plug into. Phase 0's brewing and archive items replace
//! the bodies here with real world effects; what exists now is the subset that
//! makes already-built subsystems **reachable from the running game** — the
//! retroactive playability pass in DESIGN.md §15.
//!
//! | Verb | Does | Makes visible |
//! |---|---|---|
//! | `meditate <n>` | Lets `n` ticks pass | The world clock and the seeded, replayable spine |
//! | `status` | Reports world state as records | The record model drawn as a table by the game rather than by an example |
//! | `peruse orb.log` | Re-emits the scrollback | §3's "the stream **is** the log" |
//! | `sift <pattern> orb.log` | Filters it | §7's "pipes operate on records, never rendered text" |
//!
//! Everything else acknowledges and does nothing. That is honest rather than
//! lazy: a verb whose domain does not exist has nothing to do, and §6 already
//! guarantees the player was told what the orb understood.
//!
//! **No prose here.** Rule 6 and §12 put authored text in content files; these
//! emit facts and let a later layer wrap sentences around them.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role, Sift};

use crate::parser::{Intent, NounKind, Verb};
use crate::rng::Rngs;
use crate::session::{Pending, Scrollback, Skip};
use crate::tick::Tick;
use crate::tower::{self, Cwd};

/// The name the scrollback answers to.
///
/// §3: unlogged output is forbidden, so the record stream *is* the log. Giving
/// it a filename is not a debugging affordance — it is the same object the
/// player will `peruse` and pipe once the filesystem exists, reachable early.
pub const LOG: &str = "orb.log";

/// The most ticks one `meditate` may pass.
///
/// An hour of world time. A cap exists because the loop runs inside a single
/// `step()`: at ~microseconds a tick this is imperceptible, and an uncapped
/// count typed by a curious player should not be able to stall a frame.
pub const MAX_MEDITATE: u64 = 3600;

/// Run everything the player queued since the last tick.
///
/// Exclusive: a command reads and writes whatever its domain touches, and
/// enumerating that as system parameters now would be a guess about domains that
/// do not exist yet.
pub fn run_pending(world: &mut World) {
    let queued = world.resource_mut::<Pending>().drain();
    for intent in queued {
        execute(&intent, world);
    }
}

/// A count as a record value, saturating rather than wrapping.
fn quantity(count: usize) -> u64 {
    u64::try_from(count).unwrap_or(u64::MAX)
}

fn execute(intent: &Intent, world: &mut World) {
    match intent.verb {
        Verb::Attend => attend(intent, world),
        Verb::Survey => survey(intent, world),
        Verb::Meditate => meditate(intent, world),
        Verb::Status => status(world),
        Verb::Peruse => peruse(intent, world),
        Verb::Sift => sift(intent, world),
        _ => acknowledge(intent.verb, world),
    }
}

/// Go somewhere.
///
/// §7: *"Navigation is diegetic; paths are places."* Moving is also what makes a
/// domain's contents nameable at all — the scene holds every place but only the
/// belongings of where you stand, so `attend` is how a player reaches the nouns
/// a brewing command needs. See [`tower::rebuild`](crate::tower::rebuild).
fn attend(intent: &Intent, world: &mut World) {
    let Some(target) = intent.arguments.first().map(|argument| &argument.value) else {
        acknowledge(Verb::Attend, world);
        return;
    };

    let root = root(world);
    let Some(node) = find_place(world, root, target) else {
        missing(Verb::Attend, target, world);
        return;
    };

    world.insert_resource(Cwd(node));
    let path = tower::path_of(world, node);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Attend.canonical())
        .text(FieldName::Path, &path)
        .role(Role::Success)
        .finish();
}

/// List what is at a place, or here.
///
/// `Verb::Survey` takes an **optional** place, and dropping it made the echo and
/// the listing disagree: `survey archive` restated `/tower/archive` and then
/// showed the alembic. §6 makes the echo the thing players learn the vocabulary
/// from, so an echo that describes a different command than the one that ran is
/// worse than no echo at all.
fn survey(intent: &Intent, world: &mut World) {
    let at = match intent.arguments.first() {
        Some(argument) => {
            let root = root(world);
            match find_place(world, root, &argument.value) {
                Some(node) => node,
                None => {
                    missing(Verb::Survey, &argument.value, world);
                    return;
                }
            }
        }
        None => world.resource::<Cwd>().0,
    };

    let here: Vec<(String, &'static str)> = tower::children_of(world, at)
        .into_iter()
        .filter_map(|node| {
            let name = world.get::<tower::Name>(node)?.0.clone();
            let kind = world.get::<tower::Nameable>(node)?.0;
            Some((name, kind.label()))
        })
        .collect();

    let mut scrollback = world.resource_mut::<Scrollback>();
    let records = scrollback.records_mut();
    for (name, kind) in here {
        records
            .push(RecordKind::Entry)
            .text(FieldName::Name, &name)
            .text(FieldName::Kind, kind)
            .finish();
    }
}

/// The parser named something that has since stopped existing.
///
/// §8's failure taxonomy calls this a missing referent, and it is reachable
/// because resolution happens on submit while execution happens on the next
/// tick — enough of a gap for one command to remove what the next one names.
fn missing(verb: Verb, target: &str, world: &mut World) {
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Path, target)
        .role(Role::Danger)
        .finish();
}

/// The tower root.
fn root(world: &World) -> Entity {
    let mut at = world.resource::<Cwd>().0;
    while let Some(parent) = world.get::<ChildOf>(at).map(ChildOf::parent) {
        at = parent;
    }
    at
}

/// The place `target` names, by full path or by last segment (§7).
fn find_place(world: &World, from: Entity, target: &str) -> Option<Entity> {
    let mut stack = vec![from];
    while let Some(node) = stack.pop() {
        if world.get::<tower::Nameable>(node).map(|n| n.0) == Some(NounKind::Place)
            && (tower::path_of(world, node) == target
                || world
                    .get::<tower::Name>(node)
                    .is_some_and(|n| n.0 == target))
        {
            return Some(node);
        }
        stack.extend(tower::children_of(world, node));
    }
    None
}

/// Let time pass.
fn meditate(intent: &Intent, world: &mut World) {
    let count = intent
        .arguments
        .first()
        .and_then(|argument| argument.value.parse::<u64>().ok())
        .unwrap_or(1)
        .min(MAX_MEDITATE);

    world.resource_mut::<Skip>().request(count);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Meditate.canonical())
        .count(FieldName::Remaining, count)
        .finish();
}

/// Report what the world currently is.
///
/// Every value here belongs to a subsystem that until now had no way of being
/// seen from the game: the tick and seed are the determinism spine, and drawing
/// them as a table is the record model doing its job in the binary rather than
/// in an example.
fn status(world: &mut World) {
    let tick = world.resource::<Tick>().get();
    let seed = world.resource::<Rngs>().master_seed();
    let logged = world.resource::<Scrollback>().records().len();
    let queued = world.resource::<Pending>().len();

    let mut scrollback = world.resource_mut::<Scrollback>();
    let rows = scrollback.records_mut();
    for (name, value) in [
        ("tick", tick),
        ("seed", seed),
        ("logged", quantity(logged)),
        ("queued", quantity(queued)),
    ] {
        rows.push(RecordKind::Status)
            .text(FieldName::Name, name)
            .count(FieldName::Quantity, value)
            .finish();
    }
}

/// Read a file.
///
/// The domain logs are real [`NounKind::File`] nouns and hold nothing yet, so
/// they read back empty — a count of zero, which is a true answer.
///
/// They used to fall through to [`acknowledge`] and report **success having read
/// nothing**, which is worse than an error: §6 forbids a bare error precisely so
/// a player is never left guessing, and a cheerful completion over an unread
/// file leaves them guessing with false confidence. The branch was unreachable
/// until the tower gave the scene a second `File`.
fn peruse(intent: &Intent, world: &mut World) {
    // Snapshot first: the stream being read is the stream being written to.
    let lines = if names_the_log(intent) {
        messages(world, None)
    } else {
        Vec::new()
    };
    emit(world, Verb::Peruse, &lines);
}

/// Filter the log.
///
/// The first pipe stage, on real records. §7 calls this *"the only model that
/// survives the eldritch renderer corrupting output"* — matching runs over field
/// values, never over anything a view put on screen, so a narrow window cannot
/// change what a search returns.
fn sift(intent: &Intent, world: &mut World) {
    let Some(pattern) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Sift, world);
        return;
    };
    // A file with nothing in it matches nothing. Zero hits, honestly reported —
    // see [`peruse`] for why this is not an acknowledgement.
    let lines = if names_the_log(intent) {
        messages(world, Some(&pattern))
    } else {
        Vec::new()
    };
    emit(world, Verb::Sift, &lines);
}

/// Whether any argument names the log.
fn names_the_log(intent: &Intent) -> bool {
    intent
        .arguments
        .iter()
        .any(|argument| argument.value == LOG)
}

/// The log's lines, optionally filtered, as owned text.
///
/// Owned because the borrow has to end before anything can be written back into
/// the same stream — reading and writing one log is the normal case here, not an
/// edge one.
fn messages(world: &World, pattern: Option<&str>) -> Vec<String> {
    let sift = pattern.map(Sift::new);
    world
        .resource::<Scrollback>()
        .records()
        .iter()
        .filter(|record| sift.as_ref().is_none_or(|sift| record.matches(sift)))
        .filter(|record| record.kind() != RecordKind::LogLine)
        .map(|record| record.to_line())
        .collect()
}

/// Write `lines` back as log output, under the verb that produced them.
fn emit(world: &mut World, verb: Verb, lines: &[String]) {
    let mut scrollback = world.resource_mut::<Scrollback>();
    let rows = scrollback.records_mut();
    rows.push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .count(FieldName::Quantity, quantity(lines.len()))
        .finish();
    for line in lines {
        rows.push(RecordKind::LogLine)
            .text(FieldName::Message, line)
            .finish();
    }
}

/// The orb understood, and has nothing to do about it yet.
fn acknowledge(verb: Verb, world: &mut World) {
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .role(Role::Success)
        .finish();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;
    use orbs_render::Value;

    /// Type a line and let the tick that runs it pass.
    fn run(sim: &mut Sim, line: &str) {
        sim.submit(line);
        sim.step();
    }

    fn drawn(sim: &Sim) -> Vec<String> {
        sim.scrollback()
            .records()
            .iter()
            .map(|record| record.to_line())
            .collect()
    }

    #[test]
    fn meditate_lets_the_world_clock_run() {
        // The determinism spine had no player-facing surface at all: the tick
        // advanced whether or not anyone was watching. This is the command that
        // makes it something you can do.
        let mut sim = Sim::new(1);
        run(&mut sim, "meditate 30");

        // One tick to run the command, thirty more that it asked for.
        assert_eq!(sim.tick(), Tick::new(31));
        assert_eq!(sim.world().resource::<Skip>().owed(), 0);
    }

    #[test]
    fn meditating_is_capped_so_a_typo_cannot_stall_a_frame() {
        let mut sim = Sim::new(1);
        run(&mut sim, "meditate 999999999");
        assert_eq!(sim.tick(), Tick::new(MAX_MEDITATE + 1));
    }

    #[test]
    fn the_same_seed_and_the_same_typing_still_land_on_the_same_tick() {
        // What `meditate` exists to make checkable by hand.
        let transcript = ["meditate 5", "status", "meditate 3"];
        let mut a = Sim::new(0xC0FFEE);
        let mut b = Sim::new(0xC0FFEE);
        for sim in [&mut a, &mut b] {
            for line in transcript {
                run(sim, line);
            }
        }
        assert_eq!(a.tick(), b.tick());
        assert_eq!(drawn(&a), drawn(&b));
    }

    #[test]
    fn status_reports_the_world_as_records() {
        let mut sim = Sim::new(0xB5);
        run(&mut sim, "status");

        let rows: Vec<_> = sim
            .scrollback()
            .records()
            .iter()
            .filter(|record| record.kind() == RecordKind::Status)
            .map(|record| {
                (
                    record
                        .field(FieldName::Name)
                        .map(|v| v.with_str(str::to_owned)),
                    record.field(FieldName::Quantity),
                )
            })
            .collect();

        assert_eq!(rows[0].0.as_deref(), Some("tick"));
        assert_eq!(rows[0].1, Some(Value::Count(1)));
        assert_eq!(rows[1].0.as_deref(), Some("seed"));
        assert_eq!(rows[1].1, Some(Value::Count(0xB5)));
        // Numbers, not pre-rendered strings — which is what lets a table
        // right-align them and the harness sum them.
        assert!(
            rows.iter()
                .all(|(_, value)| value.is_some_and(|v| v.is_numeric()))
        );
    }

    #[test]
    fn the_scrollback_is_a_file_you_can_read() {
        // §3: unlogged output is forbidden, so the record stream *is* the log.
        // Naming it makes that claim something a player can check.
        let mut sim = Sim::new(1);
        run(&mut sim, "status");
        let before = sim.scrollback().records().len();

        run(&mut sim, "peruse orb.log");
        assert!(
            sim.scrollback().records().len() > before,
            "peruse produced nothing",
        );
        assert!(
            sim.scrollback()
                .records()
                .iter()
                .any(|record| record.kind() == RecordKind::LogLine),
            "no log lines emitted",
        );
    }

    #[test]
    fn sift_filters_the_log_and_finds_only_what_matches() {
        let mut sim = Sim::new(1);
        run(&mut sim, "meditate 2");
        run(&mut sim, "status");
        run(&mut sim, "sift seed orb.log");

        let hits: Vec<String> = sim
            .scrollback()
            .records()
            .iter()
            .filter(|record| record.kind() == RecordKind::LogLine)
            .map(|record| record.to_line())
            .collect();

        assert!(!hits.is_empty(), "sift found nothing");
        assert!(
            hits.iter().all(|line| line.to_lowercase().contains("seed")),
            "sift returned a non-match: {hits:?}",
        );
    }

    #[test]
    fn survey_lists_the_place_it_echoed() {
        // The echo and the listing must describe the same command. `survey`
        // dropped its optional place, so `survey archive` restated
        // `/tower/archive` and then showed the alembic — and §6 makes the echo
        // the thing players learn the vocabulary from.
        let mut sim = Sim::new(1);
        run(&mut sim, "attend alembic");
        let before = sim.scrollback().records().len();
        run(&mut sim, "survey archive");

        let listed: Vec<String> = sim
            .scrollback()
            .records()
            .iter()
            .skip(before)
            .filter(|record| record.kind() == RecordKind::Entry)
            .filter_map(|record| record.field(FieldName::Name))
            .map(|value| value.with_str(str::to_owned))
            .collect();

        assert!(listed.iter().any(|name| name == "sigil-iv"), "{listed:?}");
        assert!(!listed.iter().any(|name| name == "clarity"), "{listed:?}");
    }

    #[test]
    fn survey_with_no_place_still_lists_where_you_are() {
        let mut sim = Sim::new(1);
        run(&mut sim, "attend alembic");
        let before = sim.scrollback().records().len();
        run(&mut sim, "look around");

        let listed: Vec<String> = sim
            .scrollback()
            .records()
            .iter()
            .skip(before)
            .filter(|record| record.kind() == RecordKind::Entry)
            .filter_map(|record| record.field(FieldName::Name))
            .map(|value| value.with_str(str::to_owned))
            .collect();
        assert!(listed.iter().any(|name| name == "clarity"), "{listed:?}");
    }

    #[test]
    fn an_empty_file_reads_as_empty_rather_than_as_success() {
        // The domain logs hold nothing yet. Reporting a cheerful completion over
        // an unread file is worse than an error: §6 forbids a bare error so a
        // player is never left guessing, and false confidence leaves them
        // guessing anyway. Unreachable until the tower gave the scene a second
        // `File`.
        let mut sim = Sim::new(1);
        run(&mut sim, "attend alembic");
        let before = sim.scrollback().records().len();
        run(&mut sim, "peruse alembic.log");

        let completion = sim
            .scrollback()
            .records()
            .iter()
            .skip(before)
            .find(|record| record.kind() == RecordKind::Completion)
            .expect("a completion");
        assert_eq!(
            completion.field(FieldName::Quantity),
            Some(Value::Count(0)),
            "an unread file reported content",
        );
        assert_ne!(completion.role(), Role::Success, "false success");
    }

    #[test]
    fn sifting_never_returns_its_own_output() {
        // The log is written to while it is read, so `sift` excludes prior log
        // lines. Without that, each run would match everything the last run
        // emitted and the stream would double every time.
        let mut sim = Sim::new(1);
        run(&mut sim, "status");

        let hits = |sim: &Sim| {
            sim.scrollback()
                .records()
                .iter()
                .filter(|record| record.kind() == RecordKind::LogLine)
                .count()
        };

        run(&mut sim, "sift tick orb.log");
        let first = hits(&sim);
        run(&mut sim, "sift tick orb.log");
        let second = hits(&sim) - first;

        assert!(first > 0, "the first sift found nothing to compare");
        // It may grow a little — each run adds its own input and echo, and those
        // contain the pattern. What it must not do is compound.
        assert!(
            second < first * 2,
            "sift is re-reading its own output: {first} then {second}",
        );
    }
}
