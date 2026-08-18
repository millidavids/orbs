//! §8.1's **world state** surface — a reagent swapped, and how it is found.
//!
//! The log surface has shipped since Phase 0; this is the second of the four,
//! and the one whose tell §8.1 states as *"substituted entities fail ID check →
//! `Referent missing`"*.
//!
//! Both channels are asserted here, because §8.1 requires both and each on its
//! own is a defect:
//!
//! - a tell only `verify` could find makes looking pointless
//! - a tell only looking could find is a puzzle a screen reader cannot play

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

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

#[test]
fn a_swapped_reagent_is_both_visible_and_verifiable() {
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "debug_swap");

    // **Channel one: it is on screen.** `survey` lists the shelf and one pile is
    // not what it was — the structural signature §8.1 asks for, perceptible to a
    // player who looks and carried by the *name* rather than by a colour.
    run(&mut sim, "survey dispensary");
    let listed = fields(&sim, FieldName::Name);
    assert!(
        listed.iter().any(|name| name.ends_with('-')),
        "nothing on the shelf reads as substituted: {listed:?}",
    );

    // **Channel two: one command finds it, and names it.** §5.1's one-command
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
    // **§8.1's stated tell**: *"substituted entities fail ID check → `Referent
    // missing`"*. This is what makes the world a surface worth inspecting — the
    // sabotage is felt as a spell that stopped working, and finding out *why* is
    // the thing scrying exists for.
    // **`kindle charcoal`, because `debug_swap` takes the shelf's first pile by
    // name and that is the charcoal.** A spell naming a reagent the swap did not
    // touch would pass this test while proving nothing — the first version wrote
    // `grind charcoal`, which fails on a tower with no sabotage at all because
    // charcoal is fuel and not a mortar input.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    sim.write_spell("tending", &["kindle charcoal".to_owned()]);
    sim.step();

    // It works before the swap, which is what makes the failure afterwards mean
    // something.
    run(&mut sim, "invoke tending");
    sim.step_n(4);
    assert!(
        said(&sim).iter().any(|line| line.contains("takes light")),
        "the spell did not work before the swap: {:?}",
        said(&sim),
    );

    let before = said(&sim).len();
    run(&mut sim, "stop athanor");
    sim.step_n(6);
    run(&mut sim, "debug_swap");
    run(&mut sim, "invoke tending");
    sim.step_n(6);

    let after: Vec<String> = said(&sim).into_iter().skip(before).collect();
    assert!(
        !after.iter().any(|line| line.contains("takes light")),
        "the fire lit from a reagent that is no longer called that: {after:?}",
    );
}

#[test]
fn the_swap_leaves_the_pile_where_it_was() {
    // **Sabotage, not theft.** §5.1 keeps environmental damage in the calm layer
    // and leaves *misdirection* as the thing to see through — and a swap that
    // deleted the stock could not be recovered, compared, or verified against
    // anything, which is the same argument §3 makes for a poisoned log being
    // re-emitted rather than rewritten.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "survey dispensary");
    let before = fields(&sim, FieldName::Name).len();

    run(&mut sim, "debug_swap");
    run(&mut sim, "survey dispensary");
    let after = fields(&sim, FieldName::Name).len();

    assert!(
        after > before,
        "the shelf lost a pile to a substitution rather than keeping it",
    );
}

#[test]
fn a_lie_nobody_catches_settles_back_to_the_truth() {
    // **The floor under an unattended tower**, and it is a fact about the *spell
    // language* rather than a kindness. A spell names things with literals, so it
    // can never say "purge whatever the dispensary is lying about" — it would have
    // to name `sage-`, a word nobody knew when the spell was written. So
    // `verify` → `purge` is reachable only by a person reading the screen.
    //
    // Without an expiry that made an automated tower terminal rather than
    // harassed: `orbs-balance` measured the standing grind loop falling from
    // 0.100/tick to 0.058 and staying there for the rest of the session.
    // The **last** verdict, not any of them: the point of this test is that the
    // second `verify` disagrees with the first, so `any` would pass on either.
    fn verdict(sim: &Sim) -> Option<String> {
        fields(sim, FieldName::State)
            .into_iter()
            .rfind(|state| state == "tampered" || state == "sound")
    }

    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "debug_swap");
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

    // **And the pile is usable again, not merely un-poisoned.** Reporting `sound`
    // while still carrying the lie would be the worse of the two bugs — a shelf
    // that verifies clean and refuses every recipe that names it.
    run(&mut sim, "survey dispensary");
    let names = fields(&sim, FieldName::Name);
    assert!(
        names.iter().any(|name| name == "charcoal"),
        "the pile settled without getting its name back: {names:?}",
    );
}

#[test]
fn the_ambient_swap_never_takes_the_fire() {
    // **Fuel is exempt, and `orbs-balance` is what found it.** A swap is honest
    // because it costs the spell that named the reagent and nothing half-made —
    // and charcoal is named by no recipe at all, it is the tower's power supply.
    // Swapping it stops every heated stage in every domain at once.
    //
    // It shipped taking the alphabetically-first endless pile, which *is*
    // `charcoal`, so the first swap of every session took the fire: clarity's rate
    // fell from 0.140 to 0.074 on every seed, and the suite was green throughout.
    //
    // Four hours across four seeds. `SWAP_INTERVAL` is one an hour, so this is
    // ~16 swaps' worth of rolls against a pool of two.
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
