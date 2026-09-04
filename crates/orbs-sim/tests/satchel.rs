//! The satchel, driven through the real parser and the real schedule.
//!
//! `tower::satchel`'s own tests prove the queue; these prove the *game* — that
//! `queue` and `pull` reach it, that a spell can hand another spell a name, and
//! that a consumer which has caught up with its producer waits rather than
//! faulting.
//!
//! # What is worth proving here and nowhere else
//!
//! **The satchel is the one fixture that exists in every domain under one
//! name.** That makes it the one place where *nameable* and *findable* can come
//! apart room by room — §19 records paying for that twice already — so most of
//! this file is asking the same question from different rooms.

use orbs_render::{FieldName, Value};
// `Save` and the two helpers below belong to the debug-gated tests, which is
// most of this file: `debug_take` buys the channel and `debug_spawn` fills the
// satchel, and neither word exists in a release build.
#[cfg(debug_assertions)]
use orbs_sim::Save;
use orbs_sim::Sim;

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

/// A tower standing in `room` with §8's channel bought.
///
/// **`debug_take`, not three distillations.** `satchel_1` is a mastery node and
/// the honest road to it is 24 experience — about two hundred ticks of the
/// laboratory, in a file about the menagerie. That is the setup cost
/// `debug_spawn` and `debug_learn` already exist to skip, and the shortcut grants
/// the *real* node, so everything downstream sees what a played tower would.
///
/// The gate itself is proved by
/// [`the_channel_is_bought_at_the_loom`](the_channel_is_bought_at_the_loom),
/// which is the one test here that must not take it.
fn in_room(room: &str) -> Sim {
    let mut sim = Sim::new(11);
    run(&mut sim, &format!("attend {room}"));
    run(&mut sim, "debug_take satchel_1");
    sim
}

/// Every sentence the tower has said so far.
fn said(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| match record.field(FieldName::Message) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

fn ever_said(sim: &Sim, needle: &str) -> bool {
    said(sim).iter().any(|line| line.contains(needle))
}

#[cfg(debug_assertions)]
fn last(sim: &Sim) -> String {
    said(sim).last().cloned().unwrap_or_default()
}

/// Every `Name` field the tower has emitted, which is where `survey`'s rows are.
///
/// **`survey` emits `TableRow`s, not `Message`s**, so [`said`] cannot see its
/// answer at all — the diagnostic CLAUDE.md records costing three experiments in
/// the menagerie, and `warding.rs` records two tests passing for the wrong
/// reason against the same mistake.
#[cfg(debug_assertions)]
fn listed(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| match record.field(FieldName::Name) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

fn write(sim: &mut Sim, name: &str, lines: &[&str]) {
    let lines: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    sim.write_spell(name, &lines);
    sim.step();
}

/// **The channel is bought, and until it is the words say where to buy it.**
///
/// `queue` refuses in voice — `bind`'s shape, and for `bind`'s reason: a word
/// that works and is not yet available is a different thing from a word that is
/// broken. `pull` is a control word with nobody to answer, so its report is a
/// complaint at cast, which is where `interpret` shows it.
///
/// The refusal names the **loom** rather than the satchel. Saying *"there is no
/// satchel here"* would send a player looking round the room for a thing the
/// progression tree holds.
#[cfg(debug_assertions)]
#[test]
fn the_channel_is_bought_at_the_loom() {
    let mut sim = Sim::new(11);
    run(&mut sim, "attend menagerie");

    run(&mut sim, "queue skyward");
    assert!(
        ever_said(&sim, "not learned to carry a satchel"),
        "an unbought queue worked: {:?}",
        last(&sim),
    );

    write(&mut sim, "early", &["pull note from satchel"]);
    run(&mut sim, "invoke early");
    sim.step_n(4);
    assert!(
        ever_said(&sim, "cannot pull yet"),
        "an unbought pull said nothing: {:?}",
        said(&sim),
    );

    // ...and once it is bought, the same two lines work.
    run(&mut sim, "debug_take satchel_1");
    run(&mut sim, "queue skyward");
    assert!(ever_said(&sim, "1 waiting"), "{:?}", last(&sim));
}

#[cfg(debug_assertions)]
#[test]
fn a_queued_name_is_read_back_in_the_order_it_went_in() {
    let mut sim = in_room("menagerie");
    for word in ["skyward", "earthward", "skyward"] {
        run(&mut sim, &format!("queue {word}"));
    }
    run(&mut sim, "survey satchel");

    // **A name twice, and the order kept.** This is the whole reason the queue
    // is a component rather than children plus `Stock`: a count would report
    // `skyward 2` and lose which came first.
    let rows = listed(&sim);
    let queued: Vec<&String> = rows
        .iter()
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    assert_eq!(
        queued,
        ["skyward", "earthward", "skyward"],
        "the satchel did not read back as a queue: {rows:?}",
    );
}

/// **Every domain has its own**, and naming one from another room reaches the
/// local one or nothing.
///
/// The first See-it line written for this feature failed here: the scene
/// registered all six satchels by path, §6's matcher accepts a last segment, and
/// `satchel` therefore resolved to whichever was registered first — so `queue`
/// filled the menagerie's and `survey satchel` read the *laboratory's* and
/// reported it empty, one line apart.
#[cfg(debug_assertions)]
#[test]
fn each_room_has_its_own_satchel() {
    let mut sim = in_room("menagerie");
    run(&mut sim, "queue skyward");
    assert!(ever_said(&sim, "1 waiting"), "{:?}", last(&sim));

    run(&mut sim, "attend laboratory");
    run(&mut sim, "survey satchel");
    assert!(
        ever_said(&sim, "holds nothing"),
        "the laboratory saw the menagerie's queue: {:?}",
        last(&sim),
    );

    run(&mut sim, "queue sage");
    run(&mut sim, "attend menagerie");
    run(&mut sim, "survey satchel");
    let rows = listed(&sim);
    assert!(
        rows.iter().any(|row| row == "skyward"),
        "the menagerie lost what was queued in it: {rows:?}",
    );
    assert!(
        !rows.iter().any(|row| row == "sage"),
        "the laboratory's queue leaked into the menagerie: {rows:?}",
    );
}

/// The arsenal is the Keep and has no satchel, and the refusal says so.
#[cfg(debug_assertions)]
#[test]
fn the_arsenal_has_no_satchel_and_says_so() {
    let mut sim = in_room("arsenal");
    run(&mut sim, "queue clarity");
    assert!(ever_said(&sim, "no satchel here"), "{:?}", last(&sim),);
}

/// A spell takes what a hand put in, and binds it to a name it can then use.
#[test]
fn a_spell_pulls_what_a_hand_queued_and_can_name_it() {
    let mut sim = in_room("menagerie");
    run(&mut sim, "queue skyward");
    run(&mut sim, "queue earthward");
    write(
        &mut sim,
        "drain",
        &[
            "repeat 2",
            "    pull note from satchel",
            "    survey note",
            "end",
        ],
    );
    run(&mut sim, "invoke drain");
    sim.step_n(12);

    run(&mut sim, "survey satchel");
    assert!(
        ever_said(&sim, "holds nothing"),
        "the spell did not drain the satchel: {:?}",
        said(&sim),
    );
}

/// **The two halves, and neither is a copy of the other.**
///
/// One spell queues and a second pulls, with nothing between them but the node —
/// which is the whole of what §8 could not do before. Both run at once because
/// `invoke` from inside a spell inserts a second `Running` and the caller does
/// not block; the satchel is what gives them something to say to each other.
#[cfg(debug_assertions)]
#[test]
fn one_spell_hands_another_spell_a_name() {
    let mut sim = in_room("menagerie");
    write(
        &mut sim,
        "filling",
        &["queue skyward", "queue earthward", "queue leftward"],
    );
    write(
        &mut sim,
        "draining",
        &[
            "invoke filling",
            "repeat 3",
            "    pull note from satchel",
            "    survey note",
            "end",
        ],
    );
    run(&mut sim, "invoke draining");
    sim.step_n(30);

    run(&mut sim, "survey satchel");
    assert!(
        ever_said(&sim, "holds nothing"),
        "the pipeline did not run dry: {:?}",
        said(&sim),
    );
    // **The consumer named all three**, which is the channel carrying content
    // rather than merely emptying — a `pull` that bound nothing would drain the
    // satchel just as thoroughly.
    //
    // Read off the *message*, not off `FieldName::Name`: a `survey` stamps its
    // own verb there, so `listed` reports three `survey`s and never the word
    // they surveyed. That is `warding.rs`'s two-tests-passing-for-the-wrong-
    // reason again, and it is why this file keeps both readers.
    for word in ["skyward", "earthward", "leftward"] {
        assert!(
            ever_said(&sim, word),
            "{word} never reached the consumer: {:?}",
            said(&sim),
        );
    }
}

/// **A consumer caught up with its producer waits, and is not a fault.**
///
/// `wait` gives up after `PATIENCE` ticks at `Role::Danger`, which latches `‼`
/// on the rail; a `pull` must not, because a pipeline momentarily dry is its
/// ordinary state and a fault light that fires then is a fault light nobody
/// reads. `bide`'s rule — *"a spell waiting for ever is a fault; a spell
/// counting to three is doing what it was written to do."*
///
/// **400 ticks, against a `PATIENCE` of 120**, so the run is well past the point
/// a `wait` would have given up.
#[test]
fn a_pull_on_an_empty_satchel_waits_without_latching_a_fault() {
    let mut sim = in_room("menagerie");
    run(&mut sim, "queue skyward");
    write(
        &mut sim,
        "drain",
        &[
            "repeat 3",
            "    pull note from satchel",
            "    survey note",
            "end",
        ],
    );
    run(&mut sim, "invoke drain");
    sim.step_n(400);

    assert!(
        !ever_said(&sim, "gave up"),
        "an empty satchel was treated as a fault: {:?}",
        said(&sim),
    );
    let broken = sim
        .briefs()
        .into_iter()
        .any(|brief| brief.name == "menagerie" && brief.mark == Some(orbs_sim::tower::Mark::Fault));
    assert!(!broken, "the rail marked the room broken for waiting");
}

/// A name the room cannot place is a fault, not a wait — and it is said.
#[cfg(debug_assertions)]
#[test]
fn pulling_from_something_that_is_not_a_satchel_says_so() {
    let mut sim = in_room("menagerie");
    write(&mut sim, "wrong", &["pull note from circle", "survey note"]);
    run(&mut sim, "invoke wrong");
    sim.step_n(6);

    assert!(
        ever_said(&sim, "circle"),
        "a pull from a non-satchel said nothing: {:?}",
        said(&sim),
    );
}

/// `pull` wants two words, and one is a complaint rather than a guess.
#[test]
fn a_malformed_pull_is_refused_by_name() {
    let mut sim = in_room("menagerie");
    write(&mut sim, "broken", &["pull note"]);
    run(&mut sim, "invoke broken");
    sim.step_n(4);

    assert!(ever_said(&sim, "pull wants a name"), "{:?}", said(&sim),);
}

/// **The shipped pair, cast for real** — `milling` invokes `ordering` and grinds
/// what it left behind.
///
/// A dev spell nobody casts is prose, and §19 records exactly that costing this
/// project a shipped `chanting.spell` that had stopped compiling while the whole
/// gate stayed green. These two are the worked example of a channel between two
/// spells, so they are cast here rather than admired in a TOML file.
///
/// **`bide 2` is the line under test as much as the pipeline is.** `advance`
/// snapshots the running list, so `ordering` starts on the tick *after* it is
/// invoked — and the guard is `repeat until the satchel is empty`, which is true
/// of an empty one. Without the pause the loop runs zero times and the spell
/// ends having done nothing, silently. Three yields is what says it did not.
#[cfg(debug_assertions)]
#[test]
fn the_shipped_pair_hands_a_work_list_between_two_spells() {
    let mut sim = in_room("laboratory");
    run(&mut sim, "invoke milling");
    sim.step_n(80);

    let ground = said(&sim)
        .iter()
        .filter(|line| line.contains("yields ground-"))
        .count();
    assert_eq!(
        ground,
        3,
        "the pair ground {ground} of three loads: {:?}",
        said(&sim),
    );
    run(&mut sim, "survey satchel");
    assert!(
        ever_said(&sim, "holds nothing"),
        "the work list was not drained: {:?}",
        last(&sim),
    );
}

/// **What is waiting survives a save**, in order, which is `ChantSave`'s lesson:
/// a component the world holds and the document does not is state that silently
/// resets when a player comes back.
#[cfg(debug_assertions)]
#[test]
fn a_satchel_full_of_names_survives_a_save() {
    let mut sim = in_room("menagerie");
    for word in ["skyward", "earthward", "skyward"] {
        run(&mut sim, &format!("queue {word}"));
    }

    let text = sim.snapshot().to_toml().expect("a save renders");
    let mut restored = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));

    run(&mut restored, "attend menagerie");
    run(&mut restored, "survey satchel");
    let rows = listed(&restored);
    let queued: Vec<&String> = rows
        .iter()
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    assert_eq!(
        queued,
        ["skyward", "earthward", "skyward"],
        "the queue did not come back in order: {rows:?}",
    );
}
