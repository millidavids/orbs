//! What the verbs do, driven through a real [`Sim`].
//!
//! Together rather than split with the bodies: every one submits a line and
//! steps, so what they exercise is the dispatch rather than any one module's
//! internals.

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
    // `is_live` is a hand-written restatement of `execute`'s match, and
    // `tower::boot`'s scaffold tutorial believes it.
    //
    // A live verb must reach the world from typed input, not merely have a
    // match arm — §15's playability gate as an assertion.
    //
    // A dark verb must do no work of its own. Darkness has three shapes —
    // acknowledged, unresolved, hijacked — equally empty for a player, so the
    // check is what they share: the verb's own name never appearing as a
    // completion.
    for verb in Verb::ALL {
        let mut sim = Sim::new(1);
        // §7 scopes a domain's belongings to inside it, so stand where the
        // noun is or this measures the scoping rule rather than the verb.
        let (place, line) = sample(verb);
        run(&mut sim, &format!("attend {place}"));

        let before = sim.scrollback().records().len();
        run(&mut sim, line);
        // §5.0 makes issuing instant and the action slow, so `wield` is
        // silent while an instrument works. Stepping rather than `meditate`
        // keeps this to one verb at a time; 40 covers the slowest recipe.
        for _ in 0..40 {
            sim.step();
        }

        let after: Vec<_> = sim
            .scrollback()
            .records()
            .iter()
            .skip(before)
            // Input and echo are the front half of the loop; what is under
            // test is whether the world answered.
            .filter(|record| !matches!(record.kind(), RecordKind::Input | RecordKind::Echo))
            .map(|record| (record.kind(), record.to_line()))
            .collect();

        let bare = [(RecordKind::Completion, verb.canonical().to_owned())];
        if is_live(verb) {
            assert!(!after.is_empty(), "{line} is called live and never ran");
            assert_ne!(after, bare, "{line} is called live and only heard");
        } else {
            // Every command names itself in the leading `Name` field, so a
            // completion opening with this verb's word and running past it is
            // that verb reporting work. A different verb is the hijack case.
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
/// Written out per verb rather than generated: `sift` takes two slots and
/// `attend` names a place it is not standing in, so a generator would grow
/// those cases anyway, less legibly.
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
        // §10.1's per-instrument verbs: each names its material, not its tool,
        // and only resolves where that tool stands.
        Verb::Grind => ("laboratory", "grind sage"),
        // `sage`, not `husks`: the starting tower holds no grinding byproducts,
        // so `digest husks` named nothing and only ran because a reading that
        // explained nothing still resolved as the bare verb (§19).
        Verb::Digest => ("laboratory", "digest sage"),
        Verb::Mix => ("laboratory", "mix sage and rock-salt"),
        Verb::Distil => ("laboratory", "distil sage"),
        Verb::Kindle => ("laboratory", "kindle charcoal"),
        Verb::Stop => ("laboratory", "stop mortar_and_pestle"),
        Verb::Empty => ("laboratory", "empty mortar_and_pestle"),
        // An instrument, not a vessel: §10.1 puts the product in the thing that
        // made it. The mortar is empty, and that refusal is still the world
        // answering.
        Verb::Purge => ("laboratory", "purge laboratory.log"),
        // No argument: `divine` opens the stacks rather than deciphering a
        // named fragment, and this fixture was missed when the signature
        // changed, feeding a noun to a verb with no slot for it.
        Verb::Research => ("archive", "research"),
        Verb::Scribe => ("tower", "scribe night_watch"),
        // A spell the tower actually has. `night_watch` does not exist, so once
        // `bind` went live the refusal was *"no such spell"* rather than the
        // verb working. It still refuses at concentration 0, but *"the orb
        // cannot hold a spell yet"* is the world answering.
        Verb::Bind => ("tower", "bind first_light"),
        // A spell that exists, for the reason `bind` above names. This drifted
        // the other way: with four dev ladders shelved, `invoke night_watch`
        // turned from unresolved into ambiguous — a numbered prompt of `Echo`
        // records, which this filters out, so the verb looked dead. Reach for
        // `purge` when you want an ambiguity fixture (§19).
        Verb::Invoke => ("tower", "invoke first_light"),
        Verb::Unfurl => ("tower", "unfurl"),
        Verb::Quit => ("tower", "quit"),
        Verb::Menu => ("tower", "menu"),
        Verb::Weave => ("tower", "weave"),
        Verb::Follow => ("archive", "follow north"),
        Verb::Wander => ("archive", "wander"),
        // The lens (§10). `probe` needs no setup; `dial` before one is open
        // refuses with *"probe first"*, which is the world answering.
        Verb::Probe => ("lens", "probe"),
        Verb::Dial => ("lens", "dial first alum"),
        // The sanctum (§10). `haul` before a course is drawn refuses with
        // *"muster first"*, which is the world answering.
        Verb::Muster => ("sanctum", "muster"),
        Verb::Haul => ("sanctum", "haul wellspring barrier"),
        // The menagerie (§10). `limn` before a beast is waiting refuses with
        // *"summon first"*, which is the world answering.
        Verb::Summon => ("menagerie", "summon"),
        Verb::Limn => ("menagerie", "limn keystone heed"),
        // The satchel (§8). Every domain has one, so any room would do; the
        // menagerie is where the pipeline this exists for is written.
        Verb::Queue => ("menagerie", "queue heed"),
        // The bailey (§5.1). The other three before an enemy arrives refuse
        // with *"defend first"*, which is the world answering.
        Verb::Defend => ("bailey", "defend"),
        Verb::Petition => ("bailey", "petition"),
        Verb::Deploy => ("bailey", "deploy troop"),
        // `clarity`, not `mending`: `mending` is `secret = true` and not in the
        // vocabulary until the lens spills it, so the parser refuses before
        // dispatch and the verb looks dead (§19).
        Verb::Quaff => ("bailey", "quaff clarity"),
        Verb::Hold => ("bailey", "hold"),
        // `pledge` before a siege is open refuses with *"defend first"*, which
        // is the world answering.
        Verb::Pledge => ("bailey", "pledge d20 buckler"),
        // The forge's three. `snap` and `anneal` before a lattice is open
        // refuse with *"imbue first"*, which is the world answering.
        Verb::Imbue => ("forge", "imbue mortar_and_pestle hurried"),
        Verb::Snap => ("forge", "snap apex"),
        Verb::Anneal => ("forge", "anneal"),
    }
}

#[test]
fn one_command_charges_the_instrument_and_starts_it() {
    // §10.1's loop was four commands a stage. `grind sage` is `move sage to
    // mortar_and_pestle` plus `wield mortar_and_pestle`.
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
    // `normalise` drops `and` exactly as it drops `to` and `from`, so both of
    // `Mix`'s slots fill positionally rather than through a conjunction node.
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
    // §10.1's byproduct rule needs a clear that keeps: husks are the mortar's
    // leavings and the water bath's input, so clearing by destroying alone
    // never finds route B.
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
    // §7's rule applied to verbs: `mix` means the flask and rod, and the
    // archive has none, so one domain's vocabulary cannot capture another's
    // typos. But it still answers — `grind` is two edits from `find`, which
    // `sift` claims, so scoping alone offered `sift sage archive.log`.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend archive");

    for line in ["grind sage", "mix sage and rock-salt", "distil sage"] {
        let before = sim.scrollback().records().len();
        run(&mut sim, line);
        let records = sim.scrollback();
        let new: Vec<_> = records.records().iter().skip(before).collect();

        // The outcome and the fields, not the sentence: this matched the
        // substring `"nothing here"` and broke when the refusal improved, which
        // is what rule 6 puts prose in a file to prevent.
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
    // The determinism spine had no player-facing surface; this is the command
    // that makes the tick something you can advance on purpose.
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
    // Numbers, not pre-rendered strings, so a table can right-align them and
    // the harness can sum them.
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
    // §3 names three diagnostic surfaces and this one had no producer. Both
    // reads go through one `emit_lines`, which was handed `LogLine` for
    // everything — so a spell listing linearised as `row  tick: 11, message:
    // follow north`, a table row under a world clock that are neither.
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
    // rather than as `label: value`. Only the line number survives.
    let mut spoken = String::new();
    script[0].speak(&mut spoken);
    assert!(
        !spoken.contains("message:") && !spoken.contains("tick"),
        "a script line still speaks as a fielded row: {spoken:?}",
    );
}

#[test]
fn reading_the_log_does_not_sweep_up_the_last_spell_it_listed() {
    // A listing is written back into the same stream it reads, so a
    // `ScriptLine` left unfiltered means `peruse orb.log` reports the last
    // spell opened, as log lines, growing each time.
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
    // The echo and the listing must describe the same command — §6 makes the
    // echo what players learn the vocabulary from. `survey` dropped its
    // optional place, so `survey archive` then showed the laboratory.
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

    // The lectern is the archive's own instrument. It used to be `sigil-iv`, a
    // leftover from when `research` consumed artefacts (§19).
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
fn surveying_an_empty_place_still_answers() {
    // `survey` pushes every record inside the loop over what is there, so an
    // empty place answered with silence. The sanctum made that the common case
    // — a station publishes `potency` only while it holds a ward. Asserted on
    // a scoured mortar, because the hole is `survey`'s, not the sanctum's.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    let before = sim.scrollback().records().len();
    run(&mut sim, "survey mortar_and_pestle");

    let answered: Vec<String> = sim
        .scrollback()
        .records()
        .iter()
        .skip(before)
        .filter_map(|record| record.field(FieldName::Message))
        .map(|value| value.with_str(str::to_owned))
        .collect();
    assert!(
        answered.iter().any(|line| line.contains("holds nothing")),
        "an empty place answered with silence: {answered:?}",
    );
}

#[test]
fn every_sift_hit_shows_why_it_matched() {
    // `matches` runs over all fields (rule 4) while `to_line` draws a prose
    // record as its sentence alone, so a hit on `Name = "wield"` drew as "the
    // balneum_mariae is empty" — a filter that looks broken.
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
    // This was permanently empty: the filter compared `FieldName::Source`
    // against `"laboratory"`, which nothing in §10.1's pipeline writes —
    // `transmute` puts the instrument there and `carry` writes none.
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
    // A cheerful completion over an unread file is worse than an error: §6
    // forbids a bare error so a player is never left guessing, and false
    // confidence leaves them guessing anyway.
    //
    // A search that matches nothing, rather than an empty log: this used to
    // `peruse laboratory.log` and passed only because that log was always
    // empty, so it asserted the bug.
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
    // lines or each run matches the last one's output and the stream doubles.
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
