//! A recipe you do not know yet — unmakeable, unnameable, unreadable.
//!
//! All three, and **all three flipping on the same tick**, is the property this
//! file exists for. The subtle half is the third: a secret potion's `recall_`
//! page is a prose key, and `Topics` is snapshotted once at construction so a
//! prose reload cannot change what a phrase resolves to — so a naive gate leaves
//! `recall <secret>` answering from tick 0 with every other test green, because
//! the recipe still refuses to fire.

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

/// The first secret the file reveals, which is what `debug_learn` bare takes.
const FIRST: &str = "mending";

/// **Gated, unlike its neighbours in this file.**
///
/// `debug_learn` is `cfg(debug_assertions)` and there is deliberately no way to
/// force a find — the roll is the mechanic — so the only honest alternative to
/// the door is solving wards until one spills a recipe, which is hours. A door
/// in a release build is an ordinary unresolvable line rather than an error, so
/// without this the test ran against a world where nothing had been learned and
/// failed saying *"an unfound recipe read as ready to run"*, which is true and
/// names nothing about why.
#[cfg(debug_assertions)]
#[test]
fn a_secret_is_unmakeable_unnameable_and_unreadable_until_it_is_found() {
    let mut sim = Sim::new(1);

    // **Unreadable.** `recall mending` must not resolve — it falls through to
    // the bare overview, which is what an unknown topic does.
    run(&mut sim, &format!("recall {FIRST}"));
    assert!(
        !said(&sim).iter().any(|line| line.contains("knits what")),
        "the page answered before the recipe was found",
    );

    // **Unmakeable.** The alembic holds what the recipe wants and does nothing.
    run(&mut sim, "attend laboratory");
    run(&mut sim, "kindle charcoal");
    run(&mut sim, "debug_spawn potash");
    run(&mut sim, "distil potash");
    sim.step_n(40);
    // **The refusal, not the absence of the word.** `recall mending` above is
    // echoed verbatim, so a test asking merely whether the name appears anywhere
    // catches its own earlier line — which is a test that fails for the right
    // reason on the wrong evidence.
    assert!(
        said(&sim)
            .iter()
            .any(|line| line.contains("can do nothing with potash")),
        "the alembic did not refuse a recipe nobody has found: {:?}",
        said(&sim),
    );
    assert!(
        !said(&sim)
            .iter()
            .any(|line| line.contains("yields mending")),
        "an unfound recipe fired",
    );

    // ...and all three flip together.
    run(&mut sim, "debug_learn");
    run(&mut sim, &format!("recall {FIRST}"));
    assert!(
        said(&sim).iter().any(|line| line.contains("knits what")),
        "the page did not open after the recipe was found",
    );
}

#[test]
fn everything_that_shipped_before_still_works_on_a_tower_that_has_found_nothing() {
    // **The half that would be catastrophic and silent.** If the gate caught an
    // ordinary recipe, the whole laboratory would stop and every existing
    // See-it line with it. One clarity, end to end, on a tower that knows
    // nothing beyond what it started with.
    let mut sim = Sim::new(1);
    for line in [
        "attend laboratory",
        "kindle charcoal",
        "grind sage",
        "meditate 9",
        "empty mortar_and_pestle",
        "digest ground-sage",
        "meditate 14",
        "siphon balneum_mariae",
        "grind rock-salt",
        "meditate 9",
        "empty mortar_and_pestle",
        "mix sage-tincture with ground-salt",
        "meditate 12",
        "distil clarified-draught",
        "meditate 60",
    ] {
        run(&mut sim, line);
    }
    assert_eq!(sim.experience(), 16, "the flagship brew stopped working");
}

#[test]
fn a_verdant_scroll_never_offers_a_secret_as_an_endless_herb() {
    // **The regression the design was written around.** `execute::scroll`
    // derives a base reagent as *"in the vocabulary and made by nothing"*. If
    // `Recipes::outputs` were filtered by what the player has learned, an
    // undiscovered potion would drop out of "made" and become a candidate for
    // an inexhaustible shelf — the same class of defect §19 records shipping
    // once, when every byproduct read as a herb.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    for _ in 0..5 {
        run(&mut sim, "debug_spawn verdant-scroll");
        run(&mut sim, "wield verdant-scroll");
        sim.step_n(2);
    }
    run(&mut sim, "survey dispensary");

    let listed = sim
        .scrollback()
        .records()
        .iter()
        .filter_map(|record| match record.field(FieldName::Name) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect::<Vec<_>>();
    for secret in ["mending", "dreaming", "vigour"] {
        assert!(
            !listed.contains(&secret.to_owned()),
            "a verdant scroll shelved `{secret}`, which nobody has discovered",
        );
    }
}

/// **Gated, unlike its neighbours in this file.**
///
/// `debug_learn` is `cfg(debug_assertions)` and there is deliberately no way to
/// force a find — the roll is the mechanic — so the only honest alternative to
/// the door is solving wards until one spills a recipe, which is hours. A door
/// in a release build is an ordinary unresolvable line rather than an error, so
/// without this the test ran against a world where nothing had been learned and
/// failed saying *"an unfound recipe read as ready to run"*, which is true and
/// names nothing about why.
#[cfg(debug_assertions)]
#[test]
fn the_panel_says_fouled_rather_than_charged_for_a_recipe_nobody_knows() {
    // Honest rather than coy: the player is holding something that makes
    // nothing, as far as they know. `charged` would promise a run that can never
    // start, which is the exact lie the state column exists to remove.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "debug_spawn potash");
    run(&mut sim, "move potash to alembic");
    sim.step_n(2);

    let alembic = sim
        .instruments()
        .into_iter()
        .find(|instrument| instrument.name == "alembic")
        .expect("the laboratory has an alembic");
    assert_eq!(
        alembic.state,
        orbs_sim::tower::State::Fouled,
        "an unfound recipe read as ready to run",
    );
}
