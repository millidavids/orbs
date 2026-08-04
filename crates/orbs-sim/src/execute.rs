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
        Verb::Decoct => work(intent, world, tower::DECOCT_TICKS),
        Verb::Divine => work(intent, world, tower::DIVINE_TICKS),
        Verb::Purge => purge(intent, world),
        Verb::Verify => verify(intent, world),
        _ => acknowledge(intent.verb, world),
    }
}

/// Whether the orb does this verb's work yet, or only says it heard.
///
/// Lives beside `execute` rather than on [`Verb`] because this *is* the dispatch
/// in that function, read as data — the parser knows the whole §6.1 vocabulary
/// and should keep knowing it, while what the world can currently act on is a
/// fact about this module and changes as phases land.
///
/// Written out rather than probed, because a match arm is not data. The
/// disagreement it invites is caught by
/// `a_dark_verb_only_acknowledges_and_a_live_one_does_not`, which drives all
/// sixteen through a real [`Sim`](crate::Sim) and fails the moment the two drift
/// apart.
///
/// # Why anything reads this
///
/// §15's scaffold tutorial names the vocabulary so the gate measures the parser
/// rather than a tester's guesswork. Naming a verb that only acknowledges would
/// walk that tester straight into a **dead end** — the metric §15 calls more
/// important than the raw resolution rate — so the report names what works, and
/// this is where "works" is written down.
///
/// One of the six is worse than a dead end. Phase 0 has no scripts in it, so
/// `bind`'s only slot is unfillable, and the documented `find`/`bind` collision
/// in the parser's vocabulary table then hands the line to `sift` unopposed:
/// `bind night_watch` searches the session log and reports success.
/// That resolution is correct — it stops being reachable the moment Phase 1 puts
/// a script in scope — but a tutorial that offers `bind` today is teaching a line
/// that silently runs a different command.
#[must_use]
pub const fn is_live(verb: Verb) -> bool {
    matches!(
        verb,
        Verb::Attend
            | Verb::Survey
            | Verb::Meditate
            | Verb::Status
            | Verb::Peruse
            | Verb::Sift
            | Verb::Decoct
            | Verb::Divine
            | Verb::Purge
            | Verb::Verify
    )
}

/// Start something that takes time.
///
/// §5.0: issuing is free and instant; the *action* occupies a slot for its
/// duration, and that concurrency is the whole economy. The subject must be
/// where the player is standing — §7 puts the essences in the alembic, which is
/// why `decoct` resolves there and nowhere else.
fn work(intent: &Intent, world: &mut World, ticks: u64) {
    let Some(target) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(intent.verb, world);
        return;
    };

    let cwd = world.resource::<Cwd>().0;
    let Some(subject) = tower::children_of(world, cwd)
        .into_iter()
        .find(|node| {
            world
                .get::<tower::Name>(*node)
                .is_some_and(|n| n.0 == target)
        })
        .and_then(|node| world.get::<tower::NodeId>(node).copied())
    else {
        missing(intent.verb, &target, world);
        return;
    };

    tower::begin(world, cwd, intent.verb, subject, ticks);
}

/// Destroy something where you stand.
///
/// §7 makes destruction *"useful, everyday, and scriptable"* rather than a trap,
/// and §9's per-pane **triage** slot is why it still runs during a brew: short
/// work is not what the production slot is for.
fn purge(intent: &Intent, world: &mut World) {
    let Some(target) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Purge, world);
        return;
    };

    let cwd = world.resource::<Cwd>().0;
    let found = tower::children_of(world, cwd).into_iter().find(|node| {
        world
            .get::<tower::Name>(*node)
            .is_some_and(|n| n.0 == target)
    });

    // `purge` takes `NounKind::Any`, so it reaches **places** too — and it has
    // to, or §7's guard never fires: aiming at a live domain reported "no such
    // thing" instead of the orb refusing, and those are very different answers
    // to give someone.
    let found = found.or_else(|| {
        let root = root(world);
        find_place(world, root, &target)
    });

    match found {
        Some(node) => tower::purge(world, node),
        None => missing(Verb::Purge, &target, world),
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
/// `orb.log` is the whole stream — §3: unlogged output is forbidden, so the
/// record stream *is* the log. A **domain** log is that same stream filtered by
/// who wrote each line, which is what `FieldName::Source` is for: one stream
/// read several ways rather than several streams that can disagree.
///
/// A file with nothing in it reads back as a count of zero, which is a true
/// answer. It used to report success having read nothing, which is worse than an
/// error — §6 forbids a bare error so a player is never left guessing, and a
/// cheerful completion over an unread file leaves them guessing anyway.
fn peruse(intent: &Intent, world: &mut World) {
    // Snapshot first: the stream being read is the stream being written to.
    let tampered = tampered_source(world, intent);
    let lines = read_file(world, intent, None);
    emit(world, Verb::Peruse, &lines, tampered);
}

/// Filter a file.
///
/// The first pipe stage, on real records. §7 calls this *"the only model that
/// survives the eldritch renderer corrupting output"* — matching runs over field
/// values, never over anything a view put on screen, so a narrow window cannot
/// change what a search returns, and a **poisoned** log still yields its text
/// because §3 damages only the rendering.
fn sift(intent: &Intent, world: &mut World) {
    let Some(pattern) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Sift, world);
        return;
    };
    let tampered = tampered_source(world, intent);
    let lines = read_file(world, intent, Some(&pattern));
    emit(world, Verb::Sift, &lines, tampered);
}

/// Report whether a surface has been interfered with (§8.1).
///
/// The command-detectable half of every tell. It is what makes the visual
/// signature a *speed bonus for observant players rather than a requirement* —
/// and what makes sabotage playable at all without sight.
fn verify(intent: &Intent, world: &mut World) {
    let Some(target) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Verify, world);
        return;
    };
    match here_or_place(world, &target) {
        Some(node) => tower::verify(world, node),
        None => missing(Verb::Verify, &target, world),
    }
}

/// The node `target` names: something where the player stands, or a place.
fn here_or_place(world: &mut World, target: &str) -> Option<Entity> {
    let cwd = world.resource::<Cwd>().0;
    tower::children_of(world, cwd)
        .into_iter()
        .find(|node| {
            world
                .get::<tower::Name>(*node)
                .is_some_and(|n| n.0 == target)
        })
        .or_else(|| {
            let root = root(world);
            find_place(world, root, target)
        })
}

/// Which file an intent names, if any.
fn named_file(intent: &Intent) -> Option<&str> {
    intent
        .arguments
        .iter()
        .map(|argument| argument.value.as_str())
        .find(|value| value == &LOG || value.ends_with(".log"))
}

/// Whether the named file has been poisoned (§8.1).
fn tampered_source(world: &mut World, intent: &Intent) -> bool {
    named_file(intent).is_some_and(|file| {
        here_or_place(world, file).is_some_and(|node| tower::poisoned(world, node))
    })
}

/// The lines a file holds, optionally filtered.
///
/// Owned because the borrow has to end before anything can be written back into
/// the same stream — reading and writing one log is the normal case here.
fn read_file(world: &World, intent: &Intent, pattern: Option<&str>) -> Vec<String> {
    let Some(file) = named_file(intent) else {
        return Vec::new();
    };
    // A domain log is the stream filtered by who wrote it.
    let source = (file != LOG).then(|| file.trim_end_matches(".log").to_owned());
    let sift = pattern.map(Sift::new);

    world
        .resource::<Scrollback>()
        .records()
        .iter()
        // Never its own output, or each run would match everything the last one
        // emitted and the stream would double every time.
        .filter(|record| record.kind() != RecordKind::LogLine)
        .filter(|record| match &source {
            Some(source) => {
                record.field(FieldName::Source) == Some(orbs_render::Value::Text(source))
            }
            None => true,
        })
        .filter(|record| sift.as_ref().is_none_or(|sift| record.matches(sift)))
        .map(|record| record.to_line())
        .collect()
}

/// Write `lines` back as log output, damaged if the source was poisoned.
fn emit(world: &mut World, verb: Verb, lines: &[String], tampered: bool) {
    let mut scrollback = world.resource_mut::<Scrollback>();
    tower::emit_lines(scrollback.records_mut(), verb, lines, tampered);
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
    fn a_dark_verb_only_acknowledges_and_a_live_one_does_not() {
        // `is_live` is a hand-written restatement of `execute`'s match, and the
        // scaffold tutorial in `tower::boot` believes it. So drive all sixteen
        // through a real Sim and check both directions.
        //
        // A **live** verb must reach the world from typed input — not merely have
        // a match arm. That is §15's playability gate as an assertion: the arm
        // existing proves nothing if no line a player can type gets to it.
        //
        // A **dark** verb must do no work of its own. Three ways of being dark
        // all count, because they are equally empty for a player and asserting
        // one shape would be asserting which kind of unfinished a verb is:
        // `siphon retort` resolves and acknowledges; `grimoire brewing` does not
        // resolve at all, there being no Topic in the starting tower; and `bind
        // night_watch` is taken by `sift` through the collision documented in
        // `vocabulary`. So the check is on the verb's *own* name never appearing
        // as a completion, which is the thing all three have in common.
        for verb in Verb::ALL {
            let mut sim = Sim::new(1);
            // §7 makes a domain's belongings nameable only from inside it, so a
            // test standing at the root would be measuring the scoping rule
            // rather than the verb. Stand where the noun is first.
            let (place, line) = sample(verb);
            run(&mut sim, &format!("attend {place}"));

            let before = sim.scrollback().records().len();
            run(&mut sim, line);
            // §5.0 makes issuing instant and the action slow, so `decoct` is
            // legitimately silent for twenty ticks. Stepping rather than
            // `meditate`-ing keeps this measuring one verb at a time.
            for _ in 0..tower::DECOCT_TICKS.max(tower::DIVINE_TICKS) + 1 {
                sim.step();
            }

            let after: Vec<_> = sim
                .scrollback()
                .records()
                .iter()
                .skip(before)
                // The player's own line and the parser's restatement of it are
                // both the front half of the loop. What is under test is whether
                // the *world* answered.
                .filter(|record| !matches!(record.kind(), RecordKind::Input | RecordKind::Echo))
                .map(|record| (record.kind(), record.to_line()))
                .collect();

            let bare = [(RecordKind::Completion, verb.canonical().to_owned())];
            if is_live(verb) {
                assert!(!after.is_empty(), "{line} is called live and never ran");
                assert_ne!(after, bare, "{line} is called live and only heard");
            } else {
                // Every command names itself in the leading `Name` field, so a
                // completion opening with this verb's own word is that verb
                // reporting — and anything past the bare word is it reporting
                // work. Lines opening with a *different* verb are the hijack
                // case, and they are not this verb doing anything.
                let worked = after.iter().any(|(kind, drawn)| {
                    *kind == RecordKind::Completion
                        && drawn.starts_with(verb.canonical())
                        && drawn != verb.canonical()
                });
                assert!(
                    !worked,
                    "{line} is called dark and did something: {after:?}"
                );
            }
        }
    }

    /// Where to stand, and a whole command line that works from there.
    ///
    /// Written out per verb rather than generated from the signature, because
    /// `sift` takes two slots and `attend` names a place it is not standing in —
    /// a generator would have to grow those cases anyway, and less legibly.
    fn sample(verb: Verb) -> (&'static str, &'static str) {
        match verb {
            Verb::Attend => ("tower", "attend archive"),
            Verb::Survey => ("alembic", "survey alembic"),
            Verb::Peruse => ("alembic", "peruse alembic.log"),
            Verb::Sift => ("alembic", "sift decoct alembic.log"),
            Verb::Status => ("tower", "status"),
            Verb::Grimoire => ("tower", "grimoire brewing"),
            Verb::Verify => ("alembic", "verify alembic.log"),
            Verb::Undo => ("tower", "undo"),
            Verb::Meditate => ("tower", "meditate 1"),
            Verb::Decoct => ("alembic", "decoct clarity"),
            Verb::Siphon => ("alembic", "siphon retort"),
            Verb::Purge => ("alembic", "purge alembic.log"),
            Verb::Divine => ("archive", "divine sigil-iv"),
            Verb::Scribe => ("tower", "scribe night_watch"),
            Verb::Bind => ("tower", "bind night_watch"),
            Verb::Invoke => ("tower", "invoke night_watch"),
        }
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
        // The boot report (§4) is itself a run of `Status` rows, so this reads
        // only what the command added rather than what the orb had already said.
        let before = sim.scrollback().records().len();
        run(&mut sim, "status");

        let rows: Vec<_> = sim
            .scrollback()
            .records()
            .iter()
            .skip(before)
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
