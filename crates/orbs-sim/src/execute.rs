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

use crate::parser::{Intent, Verb};
use crate::rng::Rngs;
use crate::session::{Pending, Scrollback, Skip};
use crate::tick::Tick;

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
        Verb::Meditate => meditate(intent, world),
        Verb::Status => status(world),
        Verb::Peruse => peruse(intent, world),
        Verb::Sift => sift(intent, world),
        _ => acknowledge(intent.verb, world),
    }
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

/// Re-emit the log.
fn peruse(intent: &Intent, world: &mut World) {
    if !names_the_log(intent) {
        acknowledge(Verb::Peruse, world);
        return;
    }
    // Snapshot first: the stream being read is the stream being written to.
    let lines = messages(world, None);
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
    if !names_the_log(intent) {
        acknowledge(Verb::Sift, world);
        return;
    }
    let lines = messages(world, Some(&pattern));
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
