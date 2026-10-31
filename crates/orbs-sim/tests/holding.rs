//! What the orb holds: every spell bound, and every spell a bound one starts.
//!
//! A child of a held spell runs while the player is away, so it holds a
//! concentration as its parent does, or one concentration and an `invoke`
//! would run the whole tower (§19, *The tower's spell*). Brewed from endless
//! stock, so it runs in release.

mod common;

use common::{brew_one, ever_said, run_all, said};
use orbs_sim::Sim;

/// A laboratory that has earned `levels` concentration by brewing.
fn holding(levels: usize) -> Sim {
    let mut sim = Sim::new(1);
    run_all(&mut sim, &["attend laboratory", "kindle charcoal"]);
    for _ in 0..16 {
        if sim.concentration() >= levels {
            break;
        }
        brew_one(&mut sim);
    }
    assert_eq!(
        sim.concentration(),
        levels,
        "the setup did not reach concentration {levels}: {:?}",
        said(&sim),
    );
    sim
}

/// `outer` invokes `inner`, whose lines are `inner`.
fn nested(sim: &mut Sim, inner: &[&str]) {
    let inner: Vec<String> = inner.iter().map(|line| (*line).to_owned()).collect();
    sim.write_spell("inner", &inner);
    sim.step();
    sim.write_spell(
        "outer",
        &[
            "invoke inner".to_owned(),
            "empty mortar_and_pestle".to_owned(),
        ],
    );
    sim.step();
}

fn grinds(sim: &Sim) -> usize {
    said(sim)
        .iter()
        .filter(|line| line.contains("yields ground-sage"))
        .count()
}

fn refusals(sim: &Sim) -> usize {
    said(sim)
        .iter()
        .filter(|line| line.contains("the orb is full"))
        .count()
}

#[test]
fn a_held_spell_may_invoke_another_and_keep_it() {
    // `Bound` is worn by the parent alone: a child gets `Running` and no
    // `Bound`, because it is a step of something held rather than held itself.
    // Asking about the component ended the child the tick the player walked
    // out. §8 permits nesting to depth 3.
    let mut sim = holding(2);
    nested(&mut sim, &["grind sage"]);
    let before = grinds(&sim);

    run_all(&mut sim, &["bind outer", "attend archive"]);
    sim.step_n(60);

    assert!(
        !ever_said(&sim, "needed you there"),
        "the nested spell was ended by the player leaving: {:?}",
        said(&sim),
    );
    // Counted from the bind: brewing up to the second concentration ground
    // sage of its own.
    assert!(
        grinds(&sim) > before + 1,
        "the nested spell never worked: {} grinds",
        grinds(&sim) - before,
    );
}

#[test]
fn a_full_orb_refuses_what_a_held_spell_invokes_and_names_it() {
    let mut sim = holding(1);
    nested(&mut sim, &["grind sage"]);
    let before = grinds(&sim);

    run_all(&mut sim, &["bind outer", "attend archive"]);
    sim.step_n(6);

    assert!(
        ever_said(
            &sim,
            "outer.spell line 1: the orb is full, holding outer.spell. not cast"
        ),
        "the refusal did not name what is held: {:?}",
        said(&sim),
    );
    assert_eq!(grinds(&sim), before, "the refused spell ran anyway");
    assert_eq!(sim.held(), ["outer.spell"], "the refused spell was held");
}

#[test]
fn a_full_orb_is_said_once_a_line_not_once_a_lap() {
    let mut sim = holding(1);
    nested(&mut sim, &["grind sage"]);

    run_all(&mut sim, &["bind outer", "attend archive"]);
    sim.step_n(120);

    assert_eq!(
        refusals(&sim),
        1,
        "a held spell said its refusal every lap: {:?}",
        said(&sim),
    );
}

#[test]
fn an_invocation_by_hand_is_free_however_full_the_orb_is() {
    // A hand invocation is how a spell is tested before it is given a slot, so
    // what it starts costs nothing either: it dies when the player walks out.
    let mut sim = holding(1);
    sim.write_spell("tending", &["bide 200".to_owned()]);
    sim.step();
    nested(&mut sim, &["grind sage"]);
    let before = grinds(&sim);

    run_all(&mut sim, &["bind tending", "invoke outer"]);
    sim.step_n(20);

    assert_eq!(
        refusals(&sim),
        0,
        "a hand invocation was charged: {:?}",
        said(&sim)
    );
    assert!(
        grinds(&sim) > before,
        "the invoked child never worked: {:?}",
        said(&sim)
    );
    assert_eq!(sim.held(), ["tending.spell"]);
}

#[test]
fn the_orb_counts_every_spell_it_holds() {
    let mut sim = holding(2);
    nested(&mut sim, &["bide 200"]);
    sim.write_spell("morning", &["survey".to_owned()]);
    sim.step();

    run_all(&mut sim, &["bind outer"]);
    sim.step_n(3);
    assert_eq!(sim.held(), ["inner.spell", "outer.spell"]);

    run_all(&mut sim, &["bind morning"]);
    assert!(
        ever_said(&sim, "the orb holds 2: inner.spell, outer.spell"),
        "a third was not refused naming both: {:?}",
        said(&sim),
    );
}

#[test]
fn binding_what_the_orb_already_holds_costs_nothing() {
    // The child is held already, so taking it up as a binding of its own is
    // not one more.
    let mut sim = holding(2);
    nested(&mut sim, &["bide 200"]);

    run_all(&mut sim, &["bind outer"]);
    sim.step_n(3);
    run_all(&mut sim, &["bind inner"]);

    assert!(
        ever_said(&sim, "takes up inner.spell"),
        "binding a held child was refused: {:?}",
        said(&sim),
    );
    assert_eq!(sim.held(), ["inner.spell", "outer.spell"]);
}
