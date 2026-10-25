//! The satchel, driven through the real parser and the real schedule.
//!
//! `queue` and `pull` reach it, a spell hands another a name, a caught-up
//! consumer waits. One name in every room, so most of this asks from each
//! (§19).

mod common;

use common::{ever_said, run, said, write};
#[cfg(debug_assertions)]
use common::{reloaded, words};
use orbs_sim::Sim;

/// A tower standing in `room` with §8's channel bought.
///
/// `debug_take` grants the real node; the gate is
/// [`the_channel_is_bought_at_the_loom`](the_channel_is_bought_at_the_loom)'s.
fn in_room(room: &str) -> Sim {
    let mut sim = Sim::new(11);
    run(&mut sim, &format!("attend {room}"));
    run(&mut sim, "debug_take satchel_1");
    sim
}

#[cfg(debug_assertions)]
fn last(sim: &Sim) -> String {
    said(sim).last().cloned().unwrap_or_default()
}

/// The channel is bought, and until it is the words say where to buy it.
/// `queue` refuses in voice, which is `bind`'s shape and reason: a word that
/// works and is not yet available is different from one that is broken. `pull`
/// is a control word with nobody to answer, so its report is a complaint at
/// cast. The refusal names the *loom* rather than the satchel, because *"there
/// is no satchel here"* sends a player looking round the room.
#[cfg(debug_assertions)]
#[test]
fn the_channel_is_bought_at_the_loom() {
    let mut sim = Sim::new(11);
    run(&mut sim, "attend menagerie");

    run(&mut sim, "queue heed");
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
    run(&mut sim, "queue heed");
    assert!(ever_said(&sim, "1 waiting"), "{:?}", last(&sim));
}

#[cfg(debug_assertions)]
#[test]
fn a_queued_name_is_read_back_in_the_order_it_went_in() {
    let mut sim = in_room("menagerie");
    for word in ["heed", "yoke", "heed"] {
        run(&mut sim, &format!("queue {word}"));
    }
    run(&mut sim, "survey satchel");

    // A name twice, and the order kept — the whole reason the queue is a
    // component rather than children plus `Stock`, which would report `heed 2`
    // and lose which came first.
    let rows = words(&sim);
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
        ["heed", "yoke", "heed"],
        "the satchel did not read back as a queue: {rows:?}",
    );
}

/// Every domain has its own, and naming one from another room reaches the local
/// one or nothing. The first See-it line for this feature failed here: the scene
/// registered all six by path, §6's matcher accepts a last segment, so `satchel`
/// resolved to whichever was registered first — `queue` filled the menagerie's
/// and `survey satchel` read the laboratory's, one line apart.
#[cfg(debug_assertions)]
#[test]
fn each_room_has_its_own_satchel() {
    let mut sim = in_room("menagerie");
    run(&mut sim, "queue heed");
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
    let rows = words(&sim);
    assert!(
        rows.iter().any(|row| row == "heed"),
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
    run(&mut sim, "queue heed");
    run(&mut sim, "queue yoke");
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

/// The two halves, and neither is a copy of the other: one spell queues and a
/// second pulls, with nothing between them but the node — which is what §8 could
/// not do before. Both run at once because `invoke` from inside a spell inserts
/// a second `Running` and the caller does not block.
#[cfg(debug_assertions)]
#[test]
fn one_spell_hands_another_spell_a_name() {
    let mut sim = in_room("menagerie");
    write(
        &mut sim,
        "filling",
        &["queue heed", "queue yoke", "queue spurn"],
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
    // The consumer named all three, which is the channel carrying content rather
    // than merely emptying — a `pull` that bound nothing would drain the satchel
    // just as thoroughly. Read off the *message*, not `FieldName::Name`: a
    // `survey` stamps its own verb there, so `listed` reports three `survey`s
    // and never the word they surveyed.
    for word in ["heed", "yoke", "spurn"] {
        assert!(
            ever_said(&sim, word),
            "{word} never reached the consumer: {:?}",
            said(&sim),
        );
    }
}

/// A consumer caught up with its producer waits, and is not a fault. `wait`
/// gives up after `PATIENCE` ticks at `Role::Danger`, latching `‼` on the rail;
/// a `pull` must not, because a pipeline momentarily dry is its ordinary state
/// and a fault light that fires then is one nobody reads.
///
/// 400 ticks against a `PATIENCE` of 120, so the run is well past the point a
/// `wait` would have given up.
#[test]
fn a_pull_on_an_empty_satchel_waits_without_latching_a_fault() {
    let mut sim = in_room("menagerie");
    run(&mut sim, "queue heed");
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

/// The shipped pair, cast, since an uncast dev spell rots (§19). `bide 2` is
/// under test too: `ordering` starts a tick late, and the loop would run zero
/// times over an empty satchel.
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

/// What is waiting survives a save, in order — `ChantSave`'s lesson: a component
/// the world holds and the document does not is state that silently resets when
/// a player comes back.
#[cfg(debug_assertions)]
#[test]
fn a_satchel_full_of_names_survives_a_save() {
    let mut sim = in_room("menagerie");
    for word in ["heed", "yoke", "heed"] {
        run(&mut sim, &format!("queue {word}"));
    }

    let mut restored = reloaded(&sim.snapshot());

    run(&mut restored, "attend menagerie");
    run(&mut restored, "survey satchel");
    let rows = words(&restored);
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
        ["heed", "yoke", "heed"],
        "the queue did not come back in order: {rows:?}",
    );
}
