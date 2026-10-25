//! A recipe you do not know yet — unmakeable, unnameable, unreadable.
//!
//! All three flip on one tick. The subtle one is `recall <secret>`, whose
//! `Topics` are snapshotted at construction.

mod common;

#[cfg(debug_assertions)]
use common::said;
use common::{run, words};
use orbs_sim::Sim;

/// The first secret the file reveals, which is what `debug_learn` bare takes.
#[cfg(debug_assertions)]
const FIRST: &str = "mending";

/// Gated, since `debug_learn` is debug-only and nothing forces a find.
#[cfg(debug_assertions)]
#[test]
fn a_secret_is_unmakeable_unnameable_and_unreadable_until_it_is_found() {
    let mut sim = Sim::new(1);

    // Unreadable: `recall mending` must not resolve, falling through to the bare
    // overview the way an unknown topic does.
    run(&mut sim, &format!("recall {FIRST}"));
    assert!(
        !said(&sim).iter().any(|line| line.contains("knits what")),
        "the page answered before the recipe was found",
    );

    // Unmakeable: the alembic holds what the recipe wants and does nothing.
    run(&mut sim, "attend laboratory");
    run(&mut sim, "kindle charcoal");
    run(&mut sim, "debug_spawn potash");
    run(&mut sim, "distil potash");
    sim.step_n(40);
    // The refusal, not the absence of the word: `recall mending` above is echoed
    // verbatim, so a test asking whether the name appears anywhere catches its
    // own earlier line.
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
            .any(|line| line.contains("yields") && line.contains("mending")),
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
    // The half that would be catastrophic and silent: a gate catching an
    // ordinary recipe stops the whole laboratory and every See-it line with it.
    // One clarity, end to end, on a tower that has learned nothing.
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
    // A secret must stay "made" in `Recipes::outputs`, or it reads as a base
    // reagent for an endless shelf (§19). Cultivated until every herb flowers.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "debug_spawn verdant-scroll 40");
    run(&mut sim, "wield verdant-scroll");
    run(&mut sim, "wield verdant-scroll");
    run(&mut sim, "meditate 1800");
    run(&mut sim, "survey dispensary");

    let listed = words(&sim);
    for secret in ["mending", "dreaming", "vigour"] {
        assert!(
            !listed.contains(&secret.to_owned()),
            "a verdant scroll shelved `{secret}`, which nobody has discovered",
        );
    }
}

/// Gated, since `debug_learn` is debug-only and nothing forces a find.
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
