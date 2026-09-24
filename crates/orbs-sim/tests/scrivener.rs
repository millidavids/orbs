//! What a reader may do to a `.spell`, and what it may never do.
//!
//! The prompt's reader answers a line the player is watching; this one answers
//! a line that runs unattended for hours, so the guarantees are stricter. The
//! first is that the file is still the player's (§19).

use std::sync::atomic::{AtomicUsize, Ordering};

use orbs_sim::{Scrivener, Sim, tower};

/// A reader that rewrites every line it is given, and counts how often it is
/// asked.
///
/// Deliberately wrong about everything: abstaining and never being consulted
/// look identical from outside.
struct Shouty {
    asked: AtomicUsize,
}

impl Shouty {
    const fn new() -> Self {
        Self {
            asked: AtomicUsize::new(0),
        }
    }

    fn asked(&self) -> usize {
        self.asked.load(Ordering::Relaxed)
    }
}

impl Scrivener for Shouty {
    fn read(&self, line: &str) -> Option<String> {
        self.asked.fetch_add(1, Ordering::Relaxed);
        Some(format!("survey {line}"))
    }

    /// Every `Shouty` answers alike, so they share one — which is what lets a
    /// fresh one count what a kept reading saved it.
    fn identity(&self) -> u64 {
        0x5_4077
    }
}

/// The spell node, once a queued write has landed.
fn spell_node(sim: &Sim) -> bevy_ecs::entity::Entity {
    sim.world()
        .iter_entities()
        .find(|entity| {
            entity
                .get::<tower::Name>()
                .is_some_and(|name| name.0 == "morning.spell")
        })
        .map(|entity| entity.id())
        .expect("the spell was written")
}

fn written(sim: &Sim) -> Vec<String> {
    sim.world()
        .get::<tower::Held>(spell_node(sim))
        .cloned()
        .expect("held")
        .0
}

fn compiled_from(sim: &Sim) -> Vec<String> {
    tower::spell::source(sim.world(), spell_node(sim))
}

fn lines(of: &[&str]) -> Vec<String> {
    of.iter().map(|line| (*line).to_owned()).collect()
}

#[test]
fn the_file_stays_the_players_however_loudly_it_is_read() {
    // The rule the whole shape exists to keep (§19): a reader that rewrites
    // every line must still leave `Held` byte-exact.
    let mut sim = Sim::new(3);
    let typed = lines(&["crush the sage", "then have a rest"]);
    sim.write_spell_reading("morning", &typed, &Shouty::new());
    sim.step();

    assert_eq!(written(&sim), typed, "the orb rewrote the player's file");
}

#[test]
fn the_reading_is_what_compiles() {
    // ...and the other half: a reading that reached nothing would be a cache
    // with no consumer.
    let mut sim = Sim::new(3);
    sim.write_spell_reading("morning", &lines(&["crush the sage"]), &Shouty::new());
    sim.step();

    assert_eq!(
        compiled_from(&sim),
        lines(&["survey crush the sage"]),
        "the program was built from the text rather than the reading",
    );
}

#[test]
fn with_no_reader_the_reading_is_the_text() {
    // The identity case: every build without a reader must compile what it
    // always did. `Sim::write_spell` is `write_spell_reading` with a reader
    // that answers nothing, so there is no second code path.
    let mut sim = Sim::new(3);
    let typed = lines(&["grind sage", "bide 10"]);
    sim.write_spell("morning", &typed);
    sim.step();

    assert_eq!(written(&sim), typed);
    assert_eq!(compiled_from(&sim), typed, "an unread spell was not itself");
}

#[test]
fn a_line_that_did_not_change_is_not_read_again() {
    // The editor writes the buffer out after every pause in the typing, so a
    // whole-file reading would pay for every line on every keystroke — 436µs a
    // line for the trained reader. Hence the per-line fingerprint.
    let mut sim = Sim::new(3);
    let reader = Shouty::new();

    sim.write_spell_reading(
        "morning",
        &lines(&["crush the sage", "wait a bit"]),
        &reader,
    );
    sim.step();
    assert_eq!(reader.asked(), 2, "both lines should have been read once");

    // One line edited; the other is untouched text and keeps its reading.
    sim.write_spell_reading(
        "morning",
        &lines(&["crush the sage", "wait a long while"]),
        &reader,
    );
    sim.step();
    assert_eq!(
        reader.asked(),
        3,
        "an unchanged line was read again — the cache is not per line",
    );
    assert_eq!(
        compiled_from(&sim),
        lines(&["survey crush the sage", "survey wait a long while"]),
        "the kept reading and the fresh one did not both land",
    );
}

#[test]
fn a_reading_is_kept_only_for_the_reader_that_made_it() {
    // The cache was keyed by the text alone, so a reading outlived the reader
    // that made it — saved again on `plain`, a spell went on compiling the
    // model's words.
    let mut sim = Sim::new(3);
    let typed = lines(&["crush the sage"]);
    sim.write_spell_reading("morning", &typed, &Shouty::new());
    sim.step();
    assert_eq!(compiled_from(&sim), lines(&["survey crush the sage"]));

    sim.write_spell("morning", &typed);
    sim.step();
    assert_eq!(
        compiled_from(&sim),
        typed,
        "a reading outlived the reader that made it",
    );

    let reader = Shouty::new();
    sim.write_spell_reading("morning", &typed, &reader);
    sim.step();
    assert_eq!(
        reader.asked(),
        1,
        "a copy nobody read passed for the reader's own answer",
    );
    assert_eq!(compiled_from(&sim), lines(&["survey crush the sage"]));
}

#[test]
fn interpret_judges_the_file_on_what_compiles() {
    // Blocks, calls and bindings were found in the text while each line was
    // judged on its reading, so `that is all` read as `end` and its `if` still
    // said nothing closed it.
    let sim = Sim::new(3);
    let reader = orbs_sim::Copyist::new()
        .reading("when the mortar is idle", "if mortar_and_pestle is idle")
        .reading("name the alembic hammer", "let hammer be alembic")
        .reading("that is all", "end");
    let typed = lines(&[
        "when the mortar is idle",
        "name the alembic hammer",
        "wield hammer",
        "that is all",
    ]);
    let reading = sim.read_spell_with("morning", "laboratory", &typed, &reader);
    for line in &reading {
        assert!(
            line.fault.is_none(),
            "line {} ({:?}) faults on a reading that compiles: {:?}",
            line.line,
            line.heard,
            line.fault,
        );
    }
}

#[test]
fn interpret_reads_nothing_the_save_already_read() {
    // It asked the reader about every line on every beat and ignored the reading
    // the node already held. It looks the lines up where the save does now — the
    // queued write, then the node — so the two are one reading.
    let mut sim = Sim::new(3);
    let reader = Shouty::new();
    let typed = lines(&["crush the sage", "wait a bit"]);
    sim.write_spell_reading("morning", &typed, &reader);
    let _ = sim.read_spell_with("morning", "laboratory", &typed, &reader);
    assert_eq!(
        reader.asked(),
        2,
        "the editor read again what the save had just read, before it landed",
    );

    sim.step();
    let _ = sim.read_spell_with("morning", "laboratory", &typed, &reader);
    assert_eq!(
        reader.asked(),
        2,
        "the editor read again what the spell already holds",
    );

    let edited = lines(&["crush the sage", "wait a long while"]);
    let _ = sim.read_spell_with("morning", "laboratory", &edited, &reader);
    assert_eq!(reader.asked(), 3, "an edited line should be read, and once");
}

#[test]
fn a_reading_comes_back_from_a_save_only_where_its_line_does() {
    // A save carried one reading per line by position, so a hand-edited line of
    // `held` compiled the reading of whatever used to be there. It now carries
    // the text each reading was made from.
    let mut sim = Sim::new(3);
    sim.write_spell_reading(
        "morning",
        &lines(&["crush the sage", "wait a bit"]),
        &Shouty::new(),
    );
    sim.step();

    let mut save = sim.snapshot();
    let node = save
        .nodes
        .iter_mut()
        .find(|node| node.path.ends_with("morning.spell"))
        .expect("the spell was saved");
    node.held = Some(lines(&["crush the sage", "grind sage"]));
    let text = save.to_toml().expect("a save renders");
    let loaded = Sim::restored(&orbs_sim::save::Save::from_toml(&text).expect("a save reads"));

    assert_eq!(
        compiled_from(&loaded),
        lines(&["survey crush the sage", "grind sage"]),
        "an edited line compiled a reading of the line it replaced",
    );
}

#[test]
fn a_spell_no_reader_changed_saves_no_reading() {
    // The file says nothing a load could work out alone: a spell that is its
    // own reading throughout writes no `read` at all.
    let mut sim = Sim::new(3);
    sim.write_spell("morning", &lines(&["grind sage", "bide 10"]));
    sim.step();
    let save = sim.snapshot();
    let node = save
        .nodes
        .iter()
        .find(|node| node.path.ends_with("morning.spell"))
        .expect("the spell was saved");
    assert_eq!(node.read, None, "an unread spell was written out twice");
}

#[test]
fn interpret_says_which_lines_it_read_and_what_they_were() {
    // A line read wrongly and a line read correctly both come back as plausible
    // canonical commands, so without `was` the player cannot tell a reading
    // happened at all (§19).
    let sim = Sim::new(3);
    let typed = lines(&["crush the sage", "# a note", "survey"]);
    let reading = sim.read_spell_with("morning", "laboratory", &typed, &Shouty::new());

    assert_eq!(reading[0].was.as_deref(), Some("crush the sage"));
    assert_eq!(
        reading[1].was, None,
        "a comment was read, and nothing may read one",
    );
    assert_eq!(
        reading[2].was.as_deref(),
        Some("survey"),
        "a line the reader changed must say so, canonical or not",
    );

    // ...and with nothing reading, every line is the player's own.
    let plain = sim.read_spell("laboratory", &typed);
    assert!(
        plain.iter().all(|line| line.was.is_none()),
        "an unread spell claimed to have been read",
    );
}

#[test]
fn what_a_reader_may_be_shown_is_decided_by_a_re_parse_and_not_by_a_fault() {
    // A fault gate is wrong in both directions: a lone `if` parses and faults
    // anyway, and a command never faults at all — including `stop there`
    // resolving to `stop third`, the misread the scrivener exists to fix.
    //
    // `spell::reads_cleanly` gets both right: it tolerates the three complaints
    // a line earns for being on its own, and answers `false` for a command.
    let sim = Sim::new(3);

    let statement = "if the mortar_and_pestle is not busy".to_owned();
    assert!(
        tower::spell::reads_cleanly(&statement),
        "a statement that parses is no longer recognised as one",
    );
    assert!(
        sim.read_spell("laboratory", std::slice::from_ref(&statement))[0]
            .fault
            .is_some(),
        "a lone `if` used to fault for opening a block nothing closes",
    );

    let command = "stop there".to_owned();
    assert!(!tower::spell::reads_cleanly(&command));
    let heard = &sim.read_spell("laboratory", std::slice::from_ref(&command))[0];
    println!("{command:?} -> {heard:?}");
    assert!(
        heard.fault.is_none(),
        "the matcher used to answer this confidently and wrongly",
    );
}

#[test]
fn the_sets_the_corpus_teaches_are_the_sets_the_tower_raises() {
    // Both directions: a set the tower never raises teaches the reader a `for
    // each` that walks nothing, and a set the corpus never names is one the
    // reader can never offer.
    let mut sim = Sim::new(3);
    sim.step();
    let mut raised: Vec<String> = sim
        .world()
        .iter_entities()
        .filter_map(|entity| entity.get::<tower::Grouped>().map(|group| group.0.clone()))
        .collect();
    // A store is grouped only once the arsenal holds something, so the set
    // exists in every real game and in none a test builds. Named from the
    // constant `stocktake` groups by, since `debug_spawn` is debug-only.
    raised.push(tower::STORE.to_owned());
    raised.sort();
    raised.dedup();
    let mut taught = orbs_sim::content::Phrasings::spellings().groups().to_vec();
    taught.sort();
    taught.dedup();
    assert_eq!(
        taught, raised,
        "the corpus and the tower disagree about which sets exist"
    );
}

#[test]
fn a_blank_line_and_a_comment_are_never_offered_to_a_reader() {
    // Neither is a statement, so there is nothing for a reader to say about one
    // — and a reader that answered a comment would turn a line the player
    // commented *out* into one the spell runs.
    let mut sim = Sim::new(3);
    let reader = Shouty::new();
    sim.write_spell_reading(
        "morning",
        &lines(&["", "# grind sage", "   ", "crush the sage"]),
        &reader,
    );
    sim.step();

    assert_eq!(reader.asked(), 1, "a blank or a comment reached the reader");
    assert_eq!(
        compiled_from(&sim),
        lines(&["", "# grind sage", "   ", "survey crush the sage"]),
    );
}
