//! §8.1's **world state** surface — a reagent swapped, and how it is found.
//!
//! Both channels, since §8.1 requires both:
//!
//! - a tell only `verify` could find makes looking pointless
//! - a tell only looking could find is a puzzle a screen reader cannot play

mod common;

use common::{run, said};
use orbs_render::{FieldName, Value};
use orbs_sim::Sim;

/// Every field of every record, for the assertions that read a `State`.
fn fields(sim: &Sim, wanted: FieldName) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| match record.field(wanted) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// Step until the *ambient* swap fires, and answer with what it renamed.
///
/// Not `debug_swap`, which is debug-only and not the surface; the real roll
/// costs under a second. Returns the pile, since the ambient roll exempts
/// fuel and the shortcut does not.
fn wait_for_a_swap(sim: &mut Sim) -> String {
    for _ in 0..20_000 {
        sim.step();
        // Asked of the world, not of the transcript: `fields` reads the record
        // stream, which names a pile only once something has surveyed it, so a
        // helper watching that would wait for ever on a tower nobody looked at.
        if let Some(was) = sim
            .snapshot()
            .nodes
            .into_iter()
            .find_map(|node| node.substituted.map(|lie| lie.was))
        {
            return was;
        }
    }
    panic!("no ambient swap in 20000 ticks, so this test would assert nothing");
}

#[test]
fn a_swapped_reagent_is_both_visible_and_verifiable() {
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    wait_for_a_swap(&mut sim);

    // Channel one: it is on screen. `survey` lists the shelf and one pile is not
    // what it was — §8.1's structural signature, carried by the *name* rather
    // than by a colour.
    run(&mut sim, "survey dispensary");
    let listed = fields(&sim, FieldName::Name);
    assert!(
        listed.iter().any(|name| name.ends_with('-')),
        "nothing on the shelf reads as substituted: {listed:?}",
    );

    // Channel two: one command finds it and names it — §5.1's one-command
    // diagnosis, and §8.1's rule that the skill is knowing which surface to
    // inspect rather than deciphering a clue once you have.
    run(&mut sim, "verify dispensary");
    assert!(
        said(&sim)
            .iter()
            .any(|line| line.contains("not what it says")),
        "verify did not name the substitution: {:?}",
        said(&sim),
    );
    assert!(
        fields(&sim, FieldName::State)
            .iter()
            .any(|state| state == "tampered"),
        "verify reported the shelf sound",
    );
}

#[test]
fn a_sound_shelf_still_reads_sound() {
    // The half that would make the surface useless: if everything verified as
    // tampered, the command would carry no information at all.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "verify dispensary");

    assert!(
        fields(&sim, FieldName::State)
            .iter()
            .any(|state| state == "sound"),
        "an untouched shelf did not read sound",
    );
    assert!(
        !said(&sim)
            .iter()
            .any(|line| line.contains("not what it says")),
        "an untouched shelf named a substitution",
    );
}

#[test]
fn a_substitution_stops_a_spell_that_named_the_reagent() {
    // §8.1's stated tell: *"substituted entities fail ID check → `Referent
    // missing`"*. The sabotage is felt as a spell that stopped working, and
    // finding out *why* is what scrying exists for.
    //
    // Swap first, then write the spell naming what was really hit, or it
    // proves nothing.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");

    let was = wait_for_a_swap(&mut sim);
    // `grind` takes a mortar input, and the exempted fuel is what the ambient
    // swap will never hand back — so whatever it took, this is a real operation
    // on it. `purge` first, so the *pile* is what stops the spell rather than a
    // poisoned surface refusing it for another reason.
    let before = said(&sim).len();
    sim.write_spell("tending", &[format!("grind {was}")]);
    sim.step();
    run(&mut sim, "invoke tending");
    sim.step_n(6);

    let after: Vec<String> = said(&sim).into_iter().skip(before).collect();
    assert!(
        !after.iter().any(|line| line.contains("dispensary to")),
        "the mortar took a reagent that is no longer called that: {after:?}",
    );
    assert!(
        !after.is_empty(),
        "the spell said nothing at all, so this asserts nothing",
    );
}

#[test]
fn the_swap_leaves_the_pile_where_it_was() {
    // Sabotage, not theft: §5.1 keeps environmental damage in the calm layer and
    // leaves *misdirection* as the thing to see through, and a swap that deleted
    // the stock could not be recovered or verified against anything — §3's
    // argument for a poisoned log being re-emitted rather than rewritten.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "survey dispensary");
    let before = fields(&sim, FieldName::Name).len();

    // The real swap, not the shortcut: this one *passed* in release while
    // asserting nothing, because `debug_swap` was an unresolvable line and
    // `after > before` still held from a second `survey` listing the same shelf.
    wait_for_a_swap(&mut sim);
    run(&mut sim, "survey dispensary");
    let after = fields(&sim, FieldName::Name).len();

    assert!(
        after > before,
        "the shelf lost a pile to a substitution rather than keeping it",
    );
}

#[test]
fn a_lie_nobody_catches_settles_back_to_the_truth() {
    // A lie expires, since a spell cannot name `sage-` in advance, or an
    // automated tower stays crippled (0.100 to 0.058). The last verdict, since
    // the two must disagree.
    fn verdict(sim: &Sim) -> Option<String> {
        fields(sim, FieldName::State)
            .into_iter()
            .rfind(|state| state == "tampered" || state == "sound")
    }

    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    // The pile the ambient system actually took, which is not `debug_swap`'s:
    // that takes the alphabetically-first endless pile and the ambient half
    // exempts fuel, so charcoal is what it will never choose. The old version
    // asserted `charcoal` got its name back.
    let was = wait_for_a_swap(&mut sim);
    run(&mut sim, "verify dispensary");
    assert_eq!(
        verdict(&sim).as_deref(),
        Some("tampered"),
        "the swap did not land, so this test asserts nothing",
    );

    // Long enough to clear `WEARS_OFF`, which is private and deliberately so —
    // the claim is that it *does* wear off, not the number it wears off at.
    run(&mut sim, "meditate 400");
    run(&mut sim, "verify dispensary");
    assert_eq!(
        verdict(&sim).as_deref(),
        Some("sound"),
        "a lie nobody caught held for ever",
    );

    // And the pile is usable again, not merely un-poisoned: reporting `sound`
    // while still carrying the lie is the worse bug — a shelf that verifies
    // clean and refuses every recipe that names it.
    run(&mut sim, "survey dispensary");
    let names = fields(&sim, FieldName::Name);
    assert!(
        names.contains(&was),
        "the pile settled without getting its name back: wanted {was:?} in {names:?}",
    );
}

#[test]
fn the_ambient_swap_never_takes_the_fire() {
    // Fuel is exempt: a swapped charcoal stops every heated stage at once
    // (0.140 to 0.074). Four hours over four seeds, ~16 rolls.
    for seed in 0..4 {
        let mut sim = Sim::new(seed);
        for _ in 0..4 {
            sim.submit("meditate 3600");
            for _ in 0..3601 {
                sim.step();
            }
        }

        sim.submit("attend laboratory");
        sim.step();
        sim.submit("survey dispensary");
        sim.step();
        let names = fields(&sim, FieldName::Name);
        assert!(
            names.iter().any(|name| name == "charcoal"),
            "seed {seed} lost its fire to the ambient swap: {names:?}",
        );
    }
}

/// §5.1: the calm layer touches *"environmental only — never scripts,
/// schedules, or logs."* It poisoned every log in the tower within half an
/// hour until `0.17.0` (§19).
#[test]
fn the_calm_layer_never_poisons_a_log() {
    use bevy_ecs::prelude::With;
    use orbs_sim::tower::{Log, Name, Poisoned};
    // Seed 3 poisoned a log inside the first 300 ticks when it still could.
    let mut sim = Sim::new(3);
    sim.step_n(1800);
    let world = sim.world_mut();
    let lying: Vec<String> = world
        .query_filtered::<&Name, (With<Log>, With<Poisoned>)>()
        .iter(world)
        .map(|name| name.0.clone())
        .collect();
    assert!(lying.is_empty(), "the calm layer poisoned {lying:?}");
}
