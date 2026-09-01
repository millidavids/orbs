//! §8.1's two forms of `verify`, driven through the real parser and schedule.
//!
//! `tower::audit`'s own tests prove the curve and the cooldown arithmetic; these
//! prove the *game* — that a bare `verify` is the expensive audit, that it holds
//! the production slot while it runs, and that checking one surface costs you
//! that surface and not the other.
//!
//! # Why this is Phase 1 debt rather than Phase 8 work
//!
//! `sabotage::verify`'s doc has said the expensive form *"arrives with the
//! remaining surfaces in Phase 1"* since Phase 0, and Phase 1 closed without it.
//! Until then four instant checks audited the whole tower for nothing, which is
//! the collapse §8.1 names by name: *"'which surface do I inspect first' would
//! stop being a decision, and the four-surface model would collapse on move
//! one."*

use orbs_render::{FieldName, Value};
use orbs_sim::{Save, Sim};

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

fn in_the_laboratory() -> Sim {
    let mut sim = Sim::new(11);
    run(&mut sim, "attend laboratory");
    sim
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

fn ever_said(sim: &Sim, needle: &str) -> bool {
    said(sim).iter().any(|line| line.contains(needle))
}

/// Every `State` field the tower has emitted — where `verify`'s verdict lives.
fn verdicts(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter(|record| matches!(record.field(FieldName::Name), Some(Value::Text("verify"))))
        .filter_map(|record| match record.field(FieldName::State) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// **A bare `verify` is the audit**, which is the whole of what was missing.
///
/// §8.1 writes it `verify --all`; the parser has no flag syntax at all, so it is
/// spelled the way every other widening in this game is — bare is the wide
/// scope, exactly as `survey` and `recall` already are.
#[test]
fn a_bare_verify_audits_the_whole_tower() {
    let mut sim = in_the_laboratory();
    run(&mut sim, "verify");
    assert!(
        ever_said(&sim, "the whole tower"),
        "a bare verify did not start an audit: {:?}",
        said(&sim),
    );

    // ...and it lands with a verdict rather than a material.
    sim.step_n(40);
    assert!(
        verdicts(&sim)
            .iter()
            .any(|state| state == "sound" || state == "tampered"),
        "the audit never landed a verdict: {:?}",
        said(&sim),
    );
}

/// **Production-class, which is the price §8.1 sets.**
///
/// *"`verify --all` is Production-class, and its duration scales with the
/// tower."* So it answers to `CAPACITY` and an audit means you are not brewing —
/// §5.0's *"occupies the pane for its duration, during which you are not
/// brewing"* applied to the one command that would otherwise be free.
#[test]
fn an_audit_holds_the_production_slot() {
    let mut sim = in_the_laboratory();
    run(&mut sim, "verify");
    run(&mut sim, "grind sage");

    assert!(
        ever_said(&sim, "busy verifying"),
        "an audit did not contend for the slot: {:?}",
        said(&sim),
    );
    // **And the refusal names what holds it.** The run hangs on `/tower` rather
    // than on the nameless filesystem root, which it did first — and `work_busy`
    // then read *"the  is busy verifying"*, a refusal naming nothing.
    assert!(
        ever_said(&sim, "the tower is busy"),
        "the refusal did not name the tower: {:?}",
        said(&sim),
    );
}

/// **The cooldown rations a surface, and the other stays open.**
///
/// This is the decision §8.1 is buying: *"There are only four surfaces; four
/// free instant checks **are** `verify --all` by another name."* Checking a log
/// puts every log beyond reach, and the shelf is still there to check — so
/// *which surface do I inspect first* costs something.
#[test]
fn checking_one_surface_costs_that_surface_and_not_the_other() {
    let mut sim = in_the_laboratory();

    run(&mut sim, "verify laboratory.log");
    assert_eq!(verdicts(&sim).len(), 1, "the first check said nothing");

    // The same surface, refused — and the refusal names the wait, because §6
    // forbids a bare error.
    run(&mut sim, "verify laboratory.log");
    assert!(
        ever_said(&sim, "still reading the log"),
        "a second log check was free: {:?}",
        said(&sim),
    );
    assert_eq!(verdicts(&sim).len(), 1, "the refused check answered anyway");

    // ...and the world surface is untouched by it.
    run(&mut sim, "verify dispensary");
    assert_eq!(
        verdicts(&sim).len(),
        2,
        "checking a log cooled the shelf too: {:?}",
        said(&sim),
    );
}

/// The cooldown ends, and it ends on the tick it named.
#[test]
fn a_cooled_surface_comes_back() {
    let mut sim = in_the_laboratory();
    run(&mut sim, "verify laboratory.log");
    run(&mut sim, "verify laboratory.log");
    assert_eq!(verdicts(&sim).len(), 1);

    sim.step_n(orbs_sim::tower::audit::COOLING);
    run(&mut sim, "verify laboratory.log");
    assert_eq!(
        verdicts(&sim).len(),
        2,
        "the surface never came back: {:?}",
        said(&sim),
    );
}

/// **A cooldown a player could clear by quitting is not a cooldown.**
///
/// §19 calls that shape an exploit that then needs its own rule, so `Cooling`
/// travels in the save — and an absent row reads as *nothing is cooling*, which
/// is what a document written before this existed honestly says.
#[test]
fn a_cooldown_survives_a_save() {
    let mut sim = in_the_laboratory();
    run(&mut sim, "verify laboratory.log");

    let text = sim.snapshot().to_toml().expect("a save renders");
    let mut restored = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));

    run(&mut restored, "attend laboratory");
    run(&mut restored, "verify laboratory.log");
    assert!(
        ever_said(&restored, "still reading the log"),
        "quitting cleared the cooldown: {:?}",
        said(&restored),
    );
}

/// **A thing that can never be tampered with is never rationed.**
///
/// §8.1 rations *which surface do I inspect first*; an instrument is neither a
/// log nor a shelf, so a cooldown on it would be a wait with no information
/// behind it.
#[test]
fn checking_something_that_is_not_a_surface_costs_nothing() {
    let mut sim = in_the_laboratory();
    run(&mut sim, "verify mortar_and_pestle");
    run(&mut sim, "verify mortar_and_pestle");
    assert_eq!(
        verdicts(&sim).len(),
        2,
        "an instrument was rationed: {:?}",
        said(&sim),
    );
}

/// **The audit finds what a targeted check would have**, which is what makes it
/// worth its duration rather than a slower way to be told the same nothing.
#[cfg(debug_assertions)]
#[test]
fn an_audit_names_what_is_lying() {
    let mut sim = in_the_laboratory();
    run(&mut sim, "debug_swap");
    run(&mut sim, "verify");
    sim.step_n(40);

    // `debug_swap` renames a base reagent, so the audit reports it by name —
    // §8.1's *"the skill is knowing which surface to inspect, not deciphering an
    // obscure clue"*, once the audit has been paid for.
    assert!(
        verdicts(&sim).iter().any(|state| state == "tampered"),
        "the audit missed a swap it was standing next to: {:?}",
        said(&sim),
    );
}
