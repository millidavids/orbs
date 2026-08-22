//! `bind` — and the half of it that lives in `invoke`.
//!
//! **The pair is the point.** What concentration buys is a spell that survives
//! you walking out of the room, and that is only a thing you can buy because an
//! *invocation* does not. Neither test means anything without the other, so they
//! are the same test written twice from opposite ends.

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;

/// Submit each line and give the world a tick to do it.
fn run(sim: &mut Sim, lines: &[&str]) {
    for line in lines {
        sim.submit(line);
        sim.step();
    }
}

/// Every message the orb has said.
fn messages(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| record.field(FieldName::Message))
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// Whether anything the orb said contains `needle`.
fn mentioned(sim: &Sim, needle: &str) -> bool {
    messages(sim).iter().any(|line| line.contains(needle))
}

/// A laboratory with a spell called `tending` and the concentration to hold it.
///
/// **Earned, not granted.** There is no public way to hand the sim experience
/// and there should not be — the number is what a player works for, so a test
/// that skipped the work would be testing a state the game cannot reach. Two
/// distillations are 16, which is the cheapest honest route.
///
/// The *reagent* used to be spawned, which made this whole file fail under
/// `cargo test --release`: `debug_spawn` is `cfg(debug_assertions)`, and the
/// line was gated but the test was not — so in release the draughts never
/// appeared, both distillations refused, and nine tests failed on the assertion
/// below saying the setup had not reached a slot. It had not, and nothing said
/// why.
///
/// §10.1's chain needs no door at all — sage, rock-salt and charcoal are all
/// `Holding::endless` — so both draughts are brewed now and `bind` is tested in
/// the build that ships it.
fn ready() -> Sim {
    let mut sim = Sim::new(1);
    run(&mut sim, &["attend laboratory", "kindle charcoal"]);
    for _ in 0..2 {
        brew_one(&mut sim);
    }
    assert_eq!(
        sim.concentration(),
        1,
        "the setup did not reach a slot: {:?}",
        messages(&sim),
    );

    sim.write_spell(
        "tending",
        &[
            "grind sage".to_owned(),
            "empty mortar_and_pestle".to_owned(),
        ],
    );
    sim.step();
    sim
}

/// §10.1's five stages, once, ending in a distilled `clarity`.
///
/// **Scoured before use, never after fouling** — `orbs-balance`'s rule. A second
/// lap otherwise refuses in silence: `mix` leaves the flask charged and `distil`
/// leaves the alembic so, and a charged instrument will not take a second load.
/// That is the same thing a player has to do, which is the point.
fn brew_one(sim: &mut Sim) {
    run(
        sim,
        &[
            "empty mortar_and_pestle",
            "empty balneum_mariae",
            "empty flask_and_rod",
            "empty alembic",
            "grind sage",
            "meditate 9",
            "empty mortar_and_pestle",
            "digest ground-sage",
            "meditate 14",
            "grind rock-salt",
            "meditate 9",
            "empty mortar_and_pestle",
            "mix sage-tincture with ground-salt",
            "meditate 12",
            "distil clarified-draught",
        ],
    );
    sim.step_n(60);
}

/// How many grinds have completed.
fn grinds(sim: &Sim) -> usize {
    messages(sim)
        .iter()
        .filter(|line| line.contains("yields ground-sage"))
        .count()
}

#[test]
fn the_orb_cannot_hold_a_spell_until_it_has_been_taught_to() {
    // **The first refusal in the game about what you have earned**, and §11.5
    // calls buying past it the game's turn. §6 forbids a bare error, so the
    // sentence has to teach twice: that holding a spell is a thing at all, and
    // that working is what buys it.
    let mut sim = Sim::new(1);
    run(&mut sim, &["attend laboratory", "bind first_light"]);

    assert!(
        mentioned(&sim, "cannot hold a spell yet"),
        "{:?}",
        messages(&sim),
    );
    assert!(
        mentioned(&sim, "work, and it will learn"),
        "the refusal did not say what to do about it: {:?}",
        messages(&sim),
    );
    assert_eq!(sim.concentration(), 0);
}

#[test]
fn an_invocation_ends_when_you_walk_out_and_a_binding_does_not() {
    // **The pair, and the whole of what concentration buys.** Nothing enforced
    // the first half before this: the domain is fixed at cast and the player's
    // position was never read again, so leaving an invocation running was free —
    // and `bind` had nothing left to sell.
    let mut invoked = ready();
    run(&mut invoked, &["invoke tending", "attend archive"]);
    invoked.step_n(40);

    assert!(
        mentioned(&invoked, "needed you there"),
        "the invocation survived the player leaving: {:?}",
        messages(&invoked),
    );
    let left_running = grinds(&invoked);

    let mut bound = ready();
    run(&mut bound, &["bind tending", "attend archive"]);
    bound.step_n(40);

    assert!(
        !mentioned(&bound, "needed you there"),
        "a bound spell stopped when the player left: {:?}",
        messages(&bound),
    );
    assert!(
        grinds(&bound) > left_running,
        "bound {} grinds, invoked {left_running} — holding bought nothing",
        grinds(&bound),
    );
}

#[test]
fn a_held_spell_stands_rather_than_finishing() {
    // A slot held by a spell that has run out is a slot held by nothing, and
    // *"walk away, come back to work done"* would be false for every spell not
    // already wrapped in a `repeat`. `tending` is two lines and no loop.
    let mut sim = ready();
    run(&mut sim, &["bind tending"]);
    sim.step_n(60);

    assert!(
        grinds(&sim) > 1,
        "it ran once and stopped: {} grinds",
        grinds(&sim),
    );
}

#[test]
fn standing_up_again_says_nothing() {
    // One line per lap is the noise §19 cut back from the editor's saves, and a
    // held spell laps every few ticks for as long as it is held. The work it
    // does still reports itself.
    let mut sim = ready();
    run(&mut sim, &["bind tending"]);
    sim.step_n(60);

    // **Once**, for the `bind` the player typed — and not again for the dozen
    // laps that followed it. The first is a thing they did; the rest are the
    // held spell being held.
    let begun = messages(&sim)
        .iter()
        .filter(|line| line.contains("takes up"))
        .count();
    assert_eq!(begun, 1, "the laps announced themselves: {begun} lines");
    assert!(grinds(&sim) > 1, "nothing ran, so nothing was silent");

    // **And no finish either**, which was the other end of the same noise: the
    // recast was made silent and the lap it recast still said *"tending.spell is
    // finished"* — a sentence contradicted a tick later, once per lap, with no
    // beginning to match it. A held spell has not finished; it is being held.
    let done = messages(&sim)
        .iter()
        .filter(|line| line.contains("is finished"))
        .count();
    assert_eq!(done, 0, "a lap called itself a finish: {done} lines");
}

#[test]
fn a_bad_name_in_a_held_spell_is_said_once_and_not_once_a_lap() {
    // **The rationing and the standing collided.** `Running::said` holds a bad
    // name to one report per line per casting, and `stand` casts again every
    // time the spell runs off the end — so clearing `said` at every cast put the
    // rationing back to square one twice a second, which is the exact failure it
    // exists to prevent arriving through the fix for a different one.
    let mut sim = ready();
    sim.write_spell(
        "wrong",
        &[
            "if the mortr is idle".to_owned(),
            "survey".to_owned(),
            "end".to_owned(),
        ],
    );
    sim.step();
    run(&mut sim, &["bind wrong"]);
    sim.step_n(60);

    let complained = messages(&sim)
        .iter()
        .filter(|line| line.contains("mortr"))
        .count();
    assert_eq!(
        complained, 1,
        "the bad name was reported {complained} times across 60 ticks",
    );
}

#[test]
fn binding_a_running_invocation_takes_it_up_where_it_stands() {
    // §8 makes `invoke` the way to **test** a spell before committing a slot, so
    // `invoke x` then `bind x` is the sequence the design recommends — and `bind`
    // cast again unconditionally, silently discarding everything the test had
    // done and restarting at line 1.
    let mut sim = ready();
    run(&mut sim, &["invoke tending"]);
    sim.step_n(3);
    let mid = sim
        .running_line("tending.spell")
        .expect("the invocation never started");

    run(&mut sim, &["bind tending"]);
    assert_eq!(
        sim.running_line("tending.spell"),
        Some(mid),
        "binding restarted the run: {:?}",
        messages(&sim),
    );

    // ...and it is genuinely held now, not merely left running.
    run(&mut sim, &["attend archive"]);
    sim.step_n(20);
    assert!(
        !mentioned(&sim, "needed you there"),
        "it was taken up and then walked out on anyway: {:?}",
        messages(&sim),
    );
}

#[test]
fn a_held_spell_may_invoke_another_and_keep_it() {
    // **The `Bound` component is worn by the parent alone.** A held spell that
    // `invoke`s another gives the child a `Running` and no `Bound` — the child is
    // not held, it is a step of something that is — so asking about the component
    // ended the child on the tick the player walked out, in exactly the
    // walk-away case `bind` exists to sell. §8 permits nesting to depth 3.
    let mut sim = ready();
    sim.write_spell("inner", &["grind sage".to_owned()]);
    sim.step();
    sim.write_spell(
        "outer",
        &[
            "invoke inner".to_owned(),
            "empty mortar_and_pestle".to_owned(),
        ],
    );
    sim.step();

    run(&mut sim, &["bind outer", "attend archive"]);
    sim.step_n(60);

    assert!(
        !mentioned(&sim, "needed you there"),
        "the nested spell was ended by the player leaving: {:?}",
        messages(&sim),
    );
    assert!(
        grinds(&sim) > 1,
        "the nested spell never worked: {} grinds",
        grinds(&sim),
    );
}

#[test]
fn a_spell_an_invocation_starts_dies_with_it() {
    // The other half of the rule above, and the reason it is a *flag* rather than
    // an exemption for everything nested: a child that outlived the parent the
    // player's own departure just ended would be unattended automation bought for
    // nothing, which is the whole of what `bind` sells.
    let mut sim = ready();
    sim.write_spell("inner", &["grind sage".to_owned()]);
    sim.step();
    sim.write_spell(
        "outer",
        &[
            "invoke inner".to_owned(),
            "empty mortar_and_pestle".to_owned(),
        ],
    );
    sim.step();

    run(&mut sim, &["invoke outer", "attend archive"]);
    sim.step_n(60);

    assert_eq!(
        sim.running_line("inner.spell"),
        None,
        "the nested spell outlived the invocation that started it: {:?}",
        messages(&sim),
    );
}

#[test]
fn a_second_spell_is_refused_and_names_what_is_held() {
    // At concentration 1 this is the sharpest decision in the game (§11.5), and
    // it cannot be made by a player who has to go and look up what they are
    // already holding.
    let mut sim = ready();
    sim.write_spell("morning", &["survey".to_owned()]);
    sim.step();
    run(&mut sim, &["bind tending", "bind morning"]);

    assert!(
        mentioned(&sim, "the orb holds 1: tending.spell"),
        "the refusal did not name what is held: {:?}",
        messages(&sim),
    );

    // ...and letting go is what makes room.
    run(&mut sim, &["stop tending", "bind morning"]);
    assert!(
        mentioned(&sim, "you let tending.spell go"),
        "{:?}",
        messages(&sim)
    );
    assert!(
        mentioned(&sim, "takes up morning.spell"),
        "the slot did not free: {:?}",
        messages(&sim),
    );
}

#[test]
fn letting_go_really_stops_it() {
    // **`stop` releases before it un-runs**, or `stand` would put the spell back
    // on the next tick and the player would watch nothing happen.
    let mut sim = ready();
    run(&mut sim, &["bind tending"]);
    sim.step_n(20);
    let before = grinds(&sim);

    run(&mut sim, &["stop tending"]);
    sim.step_n(40);

    assert_eq!(grinds(&sim), before, "it stood back up after being let go");
}

#[test]
fn the_boot_report_offers_bind_only_when_it_can_do_something() {
    // §15's scaffold names the words that **work**, and its most important
    // metric is the dead-end rate. A verb that can only refuse belongs off the
    // list — which at concentration 0 is every mention of `bind`.
    let sim = Sim::new(1);
    let listed: Vec<String> = sim
        .scrollback()
        .records()
        .iter()
        .filter(|record| record.kind() == orbs_render::RecordKind::Entry)
        .filter_map(|record| record.field(FieldName::Name))
        .map(|value| value.with_str(str::to_owned))
        .collect();

    assert!(
        !listed.iter().any(|name| name == "bind"),
        "a new player is sent after a verb that can only refuse: {listed:?}",
    );
    assert!(listed.iter().any(|name| name == "invoke"), "{listed:?}");
}
