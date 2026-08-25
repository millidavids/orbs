//! What the verbs do, driven through a real [`Sim`].
//!
//! Together rather than split three ways with the bodies: every one of these
//! submits a line and steps, so what they exercise is the *dispatch* — the seam
//! `execute` is — rather than any one module's internals.

use orbs_render::{FieldName, Outcome, RecordKind, Role, Value};

use super::{MAX_MEDITATE, is_live};
use crate::Sim;
use crate::parser::Verb;
use crate::session::Skip;
use crate::tick::Tick;

/// Type a line and let the tick that runs it pass.
fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

/// Everything nameable from where the player is standing.
fn here(sim: &mut Sim) -> Vec<String> {
    let cwd = sim.world_mut().resource::<crate::tower::Cwd>().0;
    crate::tower::children_of(sim.world_mut(), cwd)
        .into_iter()
        .filter_map(|node| {
            sim.world_mut()
                .get::<crate::tower::Name>(node)
                .map(|name| name.0.clone())
        })
        .collect()
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
    // `siphon retort` resolves and acknowledges; `recall brewing` does not
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
        // §5.0 makes issuing instant and the action slow, so `wield` is
        // legitimately silent while an instrument works. Stepping rather
        // than `meditate`-ing keeps this measuring one verb at a time.
        // Long enough for the slowest recipe in `content/recipes.toml`.
        for _ in 0..40 {
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
        Verb::Survey => ("laboratory", "survey laboratory"),
        Verb::Peruse => ("laboratory", "peruse laboratory.log"),
        Verb::Sift => ("laboratory", "sift decoct laboratory.log"),
        Verb::Status => ("tower", "status"),
        Verb::Recall => ("tower", "recall brewing"),
        Verb::Verify => ("laboratory", "verify laboratory.log"),
        Verb::Undo => ("tower", "undo"),
        Verb::Meditate => ("tower", "meditate 1"),
        Verb::Move => ("laboratory", "move sage to mortar_and_pestle"),
        Verb::Wield => ("laboratory", "wield mortar_and_pestle"),
        // §10.1's per-instrument verbs. Each names its material, not its tool —
        // and each only resolves where that tool stands, so the place matters
        // more here than for anything else in this table.
        Verb::Grind => ("laboratory", "grind sage"),
        Verb::Digest => ("laboratory", "digest husks"),
        Verb::Mix => ("laboratory", "mix sage and rock-salt"),
        Verb::Distil => ("laboratory", "distil sage"),
        Verb::Kindle => ("laboratory", "kindle charcoal"),
        Verb::Stop => ("laboratory", "stop mortar_and_pestle"),
        Verb::Empty => ("laboratory", "empty mortar_and_pestle"),
        // An instrument, not a vessel: §10.1 puts the product in the thing
        // that made it. The mortar is empty here, and the refusal that
        // yields is still the world answering rather than acknowledging.
        Verb::Purge => ("laboratory", "purge laboratory.log"),
        // No argument: `divine` takes `NOTHING` since it opens the stacks
        // rather than deciphering a named fragment. The parser fixtures were
        // migrated when the signature changed and this one was missed, so it had
        // been feeding a noun to a verb with no slot for it.
        Verb::Research => ("archive", "research"),
        Verb::Scribe => ("tower", "scribe night_watch"),
        // **A spell the tower actually has.** It named `night_watch`, which does
        // not exist — harmless while `bind` was dark and a test of nothing the
        // moment it went live: the refusal would have been *"there is no such
        // spell"* rather than the verb doing its work.
        //
        // It still refuses here, at concentration 0, and that **is** the verb
        // working: this table asks whether the world answered, and *"the orb
        // cannot hold a spell yet"* is the world answering.
        Verb::Bind => ("tower", "bind first_light"),
        // **A spell that exists, for the reason `bind` above names** — and this
        // one drifted the other way. `invoke night_watch` was *unresolved* while
        // `first_light.spell` was the only script in the grimoire, so the parser
        // said so and that counted as the world answering. Shelving the dev
        // ladders in a debug build put four scripts there, and a name matching
        // none of them well became **ambiguous** instead: a numbered prompt of
        // `Echo` records, which this filters out, so the verb looked dead.
        //
        // §19 records the same drift at `brew`, and CLAUDE.md's rule from it — if
        // you want an ambiguity fixture, reach for `purge` deliberately rather
        // than depending on how many nouns happen to exist.
        Verb::Invoke => ("tower", "invoke first_light"),
        Verb::Unfurl => ("tower", "unfurl"),
        Verb::Quit => ("tower", "quit"),
        Verb::Weave => ("tower", "weave"),
        Verb::Follow => ("archive", "follow north"),
        Verb::Wander => ("archive", "wander"),
        // The lens (§10). `probe` opens a reading and presses it in one word, so
        // it needs no setup; `dial` before one is open refuses with *"probe
        // first"*, which is the world answering and is what this table asks for.
        Verb::Probe => ("lens", "probe"),
        Verb::Dial => ("lens", "dial first alum"),
    }
}

#[test]
fn one_command_charges_the_instrument_and_starts_it() {
    // §10.1's loop was four commands a stage, and the two in the middle are the
    // ones a player types most. `grind sage` is `move sage to
    // mortar_and_pestle` and `wield mortar_and_pestle`.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "grind sage");

    assert!(sim.working().is_some(), "the mortar never started");
    sim.step_n(20);
    run(&mut sim, "attend mortar_and_pestle");

    let inside = here(&mut sim);
    assert!(inside.contains(&"ground-sage".to_owned()), "{inside:?}");
}

#[test]
fn and_joins_two_reagents_into_one_command() {
    // `mix a and b` fills both of `Mix`'s slots. The `and` is dropped by
    // `normalise` exactly as `to` and `from` are, so the slots fill
    // positionally — the mechanism `move sage to mortar_and_pestle` already
    // used, rather than a conjunction node bolted on beside it.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "mix sage and rock-salt");

    run(&mut sim, "attend flask_and_rod");
    let inside = here(&mut sim);
    assert!(inside.contains(&"sage".to_owned()), "{inside:?}");
    assert!(inside.contains(&"rock-salt".to_owned()), "{inside:?}");
}

#[test]
fn empty_keeps_what_purge_would_destroy() {
    // §10.1's byproduct rule needs a way to clear an instrument that **keeps**
    // what was in it — husks are the mortar's leavings and the water bath's
    // input, so a loop that can only clear by destroying never finds route B.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "grind sage");
    sim.step_n(20);
    run(&mut sim, "empty mortar_and_pestle");

    run(&mut sim, "attend mortar_and_pestle");
    assert!(here(&mut sim).is_empty(), "the mortar kept something");

    run(&mut sim, "attend laboratory");
    run(&mut sim, "attend dispensary");
    let shelf = here(&mut sim);
    for wanted in ["ground-sage", "husks"] {
        assert!(
            shelf.iter().any(|name| name == wanted),
            "{wanted} was destroyed rather than shelved: {shelf:?}"
        );
    }
}

#[test]
fn empty_will_not_raid_a_running_instrument() {
    // The same lock every other pipeline verb answers to: taking the reagents
    // out from under a run spends the Focus slot and produces nothing.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "grind sage");
    assert!(sim.working().is_some(), "nothing is running");

    run(&mut sim, "empty mortar_and_pestle");
    run(&mut sim, "attend mortar_and_pestle");
    assert!(
        here(&mut sim).iter().any(|name| name == "sage"),
        "a running instrument was turned out"
    );
}

#[test]
fn an_instruments_verb_is_only_a_word_where_the_instrument_is() {
    // §7's rule applied to verbs. `mix` means the flask and rod, and there is
    // none in the archive — so the parser never considers the word there, which
    // is what stops one domain's vocabulary capturing another's typos as §10's
    // five further domains land.
    //
    // **But it still answers.** `grind` is two edits from `find`, which `sift`
    // claims, so scoping it without this would have *created* a silent
    // misreading — `grind sage` in the archive offered `sift sage archive.log`.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend archive");

    for line in ["grind sage", "mix sage and rock-salt", "distil sage"] {
        let before = sim.scrollback().records().len();
        run(&mut sim, line);
        let records = sim.scrollback();
        let new: Vec<_> = records.records().iter().skip(before).collect();

        // **The outcome and the fields, not the sentence.** This matched the
        // substring `"nothing here"` and broke when the refusal improved to name
        // the missing fixture — a test depending on wording is the thing rule 6
        // puts prose in a file to prevent. `Outcome::Unresolved` plus the fixture
        // in `Source` is the same claim, held where it cannot drift.
        assert!(
            new.iter()
                .any(|record| record.outcome() == Some(Outcome::Unresolved)),
            "{line:?} did not say it was the wrong room: {new:?}"
        );
        assert!(
            new.iter().any(|record| matches!(
                record.field(FieldName::Source),
                Some(Value::Text(fixture)) if !fixture.is_empty()
            )),
            "{line:?} refused without naming the tool the archive has not got: {new:?}"
        );

        let said: Vec<String> = new.iter().map(orbs_render::Record::to_line).collect();
        assert!(
            !said.iter().any(|line| line.contains("sift")),
            "{line:?} drifted to another verb: {said:?}"
        );
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
fn a_spell_is_read_as_a_script_and_a_log_as_a_log() {
    // **§3 names three diagnostic surfaces and this one had no producer.** Both
    // reads go through one `emit_lines`, so the only thing separating them is
    // the kind it is handed — and for four phases it was handed `LogLine` for
    // everything. A spell listing then linearised as `row  tick: 11, message:
    // follow north`: a table row that is not one, under a world clock that is
    // not one, for every line of a file the player wrote themselves.
    let mut sim = Sim::new(1);
    run(&mut sim, "peruse first_light.spell");

    let script: Vec<_> = sim
        .scrollback()
        .records()
        .iter()
        .filter(|record| record.kind() == RecordKind::ScriptLine)
        .collect();
    assert!(!script.is_empty(), "a spell read back as no script lines");
    assert!(
        sim.scrollback()
            .records()
            .iter()
            .all(|record| record.kind() != RecordKind::LogLine),
        "a spell read back as log output",
    );

    // §14: a script line is the player's own sentence, so it speaks as itself
    // rather than as `label: value`. The number survives — it is how they say
    // which line they mean — and nothing else does.
    let mut spoken = String::new();
    script[0].speak(&mut spoken);
    assert!(
        !spoken.contains("message:") && !spoken.contains("tick"),
        "a script line still speaks as a fielded row: {spoken:?}",
    );
}

#[test]
fn reading_the_log_does_not_sweep_up_the_last_spell_it_listed() {
    // The doubling guard, which used to name one kind because there was only
    // one. A listing is written back into the same stream it reads, so a
    // `ScriptLine` left unfiltered means `peruse orb.log` reports the whole of
    // whatever spell was last opened — as log lines, again, growing each time.
    let mut sim = Sim::new(1);
    run(&mut sim, "peruse first_light.spell");
    run(&mut sim, "peruse orb.log");

    let log: Vec<String> = sim
        .scrollback()
        .records()
        .iter()
        .filter(|record| record.kind() == RecordKind::LogLine)
        .map(|record| record.to_line())
        .collect();

    assert!(
        !log.is_empty(),
        "the log read back empty, so this asserts nothing"
    );
    assert!(
        !log.iter().any(|line| line.contains("kindle charcoal")),
        "the log swallowed the spell listing before it: {log:?}",
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
    // `/tower/archive` and then showed the laboratory — and §6 makes the echo
    // the thing players learn the vocabulary from.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
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

    // The lectern, which is the archive's own and stands in it. It used to be
    // `sigil-iv`, one of three artefacts the archive kept on its floor after
    // `research` stopped consuming them (§19) — an instrument is the better
    // subject anyway, because it is what the room is *for*.
    assert!(listed.iter().any(|name| name == "lectern"), "{listed:?}");
    assert!(!listed.iter().any(|name| name == "clarity"), "{listed:?}");
}

#[test]
fn survey_with_no_place_still_lists_where_you_are() {
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
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
    assert!(listed.iter().any(|name| name == "retort"), "{listed:?}");
    assert!(listed.iter().any(|name| name == "alembic"), "{listed:?}");
}

#[test]
fn every_sift_hit_shows_why_it_matched() {
    // `matches` runs over **all** fields (rule 4) while `to_line` draws a
    // prose record as its sentence alone — so a hit on `Name = "wield"` drew
    // as "the balneum_mariae is empty", a filter that looks broken because
    // the term is nowhere in the result.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "wield balneum_mariae");
    run(&mut sim, "sift wield orb.log");

    let hits: Vec<String> = sim
        .scrollback()
        .records()
        .iter()
        .filter(|record| record.kind() == RecordKind::LogLine)
        .map(|record| record.to_line())
        .collect();

    assert!(!hits.is_empty(), "sift found nothing");
    assert!(
        hits.iter()
            .all(|line| line.to_lowercase().contains("wield")),
        "a hit does not show the term it matched: {hits:?}"
    );
}

#[test]
fn a_domain_log_holds_what_happened_in_that_domain() {
    // **This was permanently empty.** The filter compared `FieldName::Source`
    // against `"laboratory"`, and nothing in §10.1's pipeline writes that:
    // `transmute` puts the *instrument* there and `carry` writes no `Source`
    // at all. So `peruse laboratory.log` returned nothing after a full brew,
    // and the one test that covered it passed by never brewing first.
    let mut sim = Sim::new(1);
    for line in [
        "attend laboratory",
        "move sage to mortar_and_pestle",
        "wield mortar_and_pestle",
    ] {
        run(&mut sim, line);
    }
    sim.step_n(20);

    let before = sim.scrollback().records().len();
    run(&mut sim, "peruse laboratory.log");
    let lines: Vec<String> = sim
        .scrollback()
        .records()
        .iter()
        .skip(before)
        .filter(|record| record.kind() == RecordKind::LogLine)
        .map(|record| record.to_line())
        .collect();

    assert!(
        lines.iter().any(|line| line.contains("mortar_and_pestle")),
        "the laboratory's log does not mention its own instrument: {lines:?}"
    );
}

#[test]
fn an_empty_file_reads_as_empty_rather_than_as_success() {
    // Reporting a cheerful completion over an unread file is worse than an
    // error: §6 forbids a bare error so a player is never left guessing, and
    // false confidence leaves them guessing anyway.
    //
    // A **search that matches nothing**, rather than a log that is empty. It
    // used to `peruse laboratory.log` and pass because that log was *always*
    // empty — the domain filter compared a field nothing writes — so this
    // asserted the bug rather than the behaviour it was written for. The
    // zero-length read is the thing under test, and a sift is the reliable
    // way to reach one now that a domain log fills up.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    let before = sim.scrollback().records().len();
    run(&mut sim, "sift zzznothinghere laboratory.log");

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
