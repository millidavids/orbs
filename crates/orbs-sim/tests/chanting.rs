//! The menagerie, driven through the real parser and the real schedule.
//!
//! `tower::chant`'s own tests prove the figure; these prove the *game* — that
//! `summon` and `sing` reach it, that the readings a spell's `if` turns on are
//! published, and that a collapsed chant costs the barrier what it says it does.

use orbs_render::{FieldName, Value};
use orbs_sim::{Sim, tower};

/// The four syllables, spelled out — a test importing them from the sim could
/// not catch a rename that broke the *player's* vocabulary.
const SYLLABLES: [&str; 4] = ["skyward", "earthward", "leftward", "rightward"];

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

fn in_the_menagerie(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    run(&mut sim, "attend menagerie");
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

fn last(sim: &Sim) -> String {
    said(sim).last().cloned().unwrap_or_default()
}

/// Every field of every record, as one line each.
///
/// **`survey`'s answer is not a `Message`** — it emits `TableRow`s carrying a
/// `Name`, so [`said`] cannot see a published reading at all. `warding.rs`
/// records two tests passing for the wrong reason against that mistake.
fn shown(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .map(|record| {
            let mut line = String::new();
            for (_, value) in record.fields() {
                value.write(&mut line);
                line.push(' ');
            }
            line
        })
        .collect()
}

/// Which syllable the circle says is next, by surveying each in turn.
///
/// **Every survey is a tick**, and the aperture moves on every tick — so this
/// cannot be a loop over four surveys. It asks the *world* instead, which is
/// what a spell's `if` does too.
fn aperture(sim: &Sim) -> Option<&'static str> {
    SYLLABLES
        .into_iter()
        .find(|word| sim.holds_reading("menagerie", word, tower::chant::NEXT))
}

/// Wait for the syllable at the aperture to reach the rule, then sing it.
///
/// **The whole of what `PACE` changed.** A syllable is struck only on the tick
/// it lands, so answering the moment it appears is `Early` and costs it — which
/// is the mechanic, and which is why every test that sang immediately had to be
/// rewritten rather than retuned.
fn sing_on_the_beat(sim: &mut Sim) -> bool {
    let Some(word) = aperture(sim) else {
        return false;
    };
    wait_for_the_rule(sim);
    run(sim, &format!("sing {word}"));
    true
}

/// Step until the aperture syllable is on the rule.
///
/// **Read off the model, not off a reading**, and that is the point rather than
/// a convenience: the circle publishes no `until` any more, because a *spell*
/// that can read the clock does not have to count it. `Chant::until` still
/// answers — for the board, for `orbs-balance`, and here — so what a test does
/// to find the beat is exactly what a player's eyes do and not what a spell can.
fn wait_for_the_rule(sim: &mut Sim) {
    while sim.figure().is_some_and(|figure| figure.until > 0) {
        sim.step();
    }
}

#[test]
fn summon_draws_a_figure_and_says_how_long_it_is() {
    let mut sim = in_the_menagerie(1);
    run(&mut sim, "summon");
    assert!(
        last(&sim).contains(&tower::chant::LENGTH.to_string()),
        "{:?}",
        last(&sim),
    );
}

/// **The reading a spell's `if` turns on**, and the one that shipped absent.
///
/// `remaining` published from the first tick and no syllable ever carried
/// `next`, so a solver's every rung would have answered no and it would have
/// stood there for ever. **Nothing on the transcript said so** — the chant ran,
/// collapsed and wore the barrier exactly as if somebody were playing it, which
/// is why looking at a dump did not find it and only asking the world did.
#[test]
fn exactly_one_syllable_is_at_the_aperture() {
    let mut sim = in_the_menagerie(1);
    run(&mut sim, "summon");

    let claiming: Vec<_> = SYLLABLES
        .into_iter()
        .filter(|word| sim.holds_reading("menagerie", word, tower::chant::NEXT))
        .collect();
    assert_eq!(claiming.len(), 1, "{claiming:?} claim the aperture");
}

/// ...and it moves on when the syllable is answered.
#[test]
fn the_aperture_advances_and_the_count_falls() {
    let mut sim = in_the_menagerie(1);
    run(&mut sim, "summon");
    run(&mut sim, "survey circle");
    assert!(
        shown(&sim)
            .iter()
            .any(|line| line.contains(tower::chant::REMAINING)),
        "the circle published no count",
    );

    let first = aperture(&sim).expect("no syllable carries `next`");
    run(&mut sim, &format!("sing {first}"));
    assert!(aperture(&sim).is_some(), "the aperture went quiet");
}

/// Singing the syllable the circle names strikes it; anything else misses.
#[test]
fn the_named_syllable_strikes_and_another_misses() {
    let mut sim = in_the_menagerie(1);
    run(&mut sim, "summon");

    assert!(sing_on_the_beat(&mut sim));
    assert!(last(&sim).contains("rings true"), "{:?}", last(&sim));

    let at = aperture(&sim).expect("the aperture did not move on");
    let wrong = SYLLABLES
        .into_iter()
        .find(|word| *word != at)
        .expect("four syllables");
    wait_for_the_rule(&mut sim);
    run(&mut sim, &format!("sing {wrong}"));
    assert!(last(&sim).contains("falls wide"), "{:?}", last(&sim));
}

/// A whole figure sung right through yields troops and earns.
#[test]
fn a_figure_sung_cleanly_yields_troops_and_earns() {
    let mut sim = in_the_menagerie(1);
    run(&mut sim, "summon");
    for _ in 0..tower::chant::LENGTH {
        if !sing_on_the_beat(&mut sim) {
            break;
        }
    }

    assert!(
        ever_said(&sim, "the figure closes"),
        "the figure never closed: {:?}",
        said(&sim).last(),
    );
    assert!(sim.experience() > 0, "a whole chant earned nothing");
    run(&mut sim, "survey arsenal");
    assert!(
        shown(&sim).iter().any(|line| line.contains("troop")),
        "no troops reached the arsenal",
    );
}

/// **Silence collapses it, and the barrier pays exactly what the line says.**
#[test]
fn a_chant_left_unsung_collapses_and_wears_the_barrier() {
    let mut sim = in_the_menagerie(1);
    let before = sim.integrity();
    run(&mut sim, "summon");
    // **`PACE` ticks a syllable**, so running a figure out is no longer one
    // step per miss — the arithmetic is what changed when a tick stopped being
    // a syllable.
    for _ in 0..=tower::chant::TOLERANCE {
        for _ in 0..tower::chant::PACE {
            sim.step();
        }
    }

    let line = said(&sim)
        .into_iter()
        .find(|line| line.contains("comes apart"))
        .expect("the figure never collapsed");
    let taken = before - sim.integrity();
    assert_eq!(taken, tower::chant::WEAR);
    assert!(
        line.contains(&taken.to_string()),
        "the sentence quotes a number the barrier did not pay: {line:?}",
    );
}

/// §11.5 — never ruinous, only slower.
///
/// The barrier saturates at nought rather than wrapping, and the room is still
/// playable at the bottom, which is the half a `saturating_sub` alone does not
/// prove.
#[test]
fn no_number_of_collapsed_chants_makes_the_tower_unplayable() {
    let mut sim = in_the_menagerie(1);
    for _ in 0..60 {
        run(&mut sim, "summon");
        // **Stepped until the figure is actually gone**, never for a computed
        // number of ticks. The arithmetic version was one tick short — `summon`
        // spends the tick it gathers on — so a third of the chants never
        // collapsed and the barrier stopped at 30 with the test claiming the
        // saturation was broken. Waiting on the world cannot drift when `PACE`
        // moves again, and it moved once already.
        while sim.figure().is_some() {
            sim.step();
        }
    }
    assert_eq!(sim.integrity(), 0);
    run(&mut sim, "summon");
    assert!(ever_said(&sim, "gathers at the circle"), "{:?}", last(&sim));
}

/// **§14: the accommodation reaches the same ceiling.**
///
/// A patient chant waits for the singer rather than for the clock, so there is
/// no window and nothing is ever early. What it must *not* be is easier or
/// harder to score on — a patient chant and a played one both yield what was
/// sung correctly, and the setting removes only the dimension reflex and speech
/// cannot serve.
///
/// **The played half of this is the control**, and without it the test would
/// pass against a patient mode that yielded nothing at all.
#[test]
fn a_patient_chant_reaches_the_same_troops_as_a_played_one() {
    let mut played = in_the_menagerie(1);
    run(&mut played, "summon");
    for _ in 0..tower::chant::LENGTH {
        if !sing_on_the_beat(&mut played) {
            break;
        }
    }

    let mut patient = in_the_menagerie(1);
    assert!(patient.set_patient(), "the flag did not go on");
    run(&mut patient, "summon");
    // **No waiting at all**, which is the whole point: the syllable sits at the
    // rule until it is answered, so this is the same loop with the timing taken
    // out.
    for _ in 0..tower::chant::LENGTH {
        let Some(at) = aperture(&patient) else { break };
        run(&mut patient, &format!("sing {at}"));
    }

    assert!(
        ever_said(&played, "the figure closes"),
        "the control failed"
    );
    assert!(ever_said(&patient, "the figure closes"));
    assert_eq!(
        patient.experience(),
        played.experience(),
        "the accommodation is not worth the same as playing it",
    );
}

/// ...and it is not a way to score *more* either.
///
/// A patient chant cannot be sung wrong and still count: correctness is the one
/// axis left, so a wrong syllable still misses.
#[test]
fn a_patient_chant_still_has_to_be_sung_correctly() {
    let mut sim = in_the_menagerie(1);
    sim.set_patient();
    run(&mut sim, "summon");

    let at = aperture(&sim).expect("no syllable carries `next`");
    let wrong = SYLLABLES
        .into_iter()
        .find(|word| *word != at)
        .expect("four syllables");
    run(&mut sim, &format!("sing {wrong}"));
    assert!(last(&sim).contains("falls wide"), "{:?}", last(&sim));
}

/// A chant costs nothing to open, which is what makes the risk the price — and
/// what lets one run beside a brew rather than instead of it.
#[test]
fn summoning_takes_no_production_slot() {
    let mut sim = in_the_menagerie(1);
    run(&mut sim, "summon");
    assert!(
        sim.working().is_none(),
        "the circle took the tower's one production slot",
    );
}

/// Earn the first mastery tier, the way a player does.
///
/// `progression.rs`'s `at_the_first_tier`, one room over: there is no public way
/// to hand the sim experience (§11.5) and three distillations are 24.
#[cfg(debug_assertions)]
fn with_a_second_step(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    for line in ["attend laboratory", "kindle charcoal"] {
        run(&mut sim, line);
    }
    run(&mut sim, "debug_spawn clarified-draught 4");
    for _ in 0..3 {
        run(&mut sim, "distil clarified-draught");
        sim.step_n(60);
        run(&mut sim, "empty alembic");
    }
    assert!(sim.experience() >= 24, "the tier never opened");
    sim.take("steps_1");
    sim.step();
    assert_eq!(
        tower::spell::budget(sim.world()),
        2,
        "the second step was not bought",
    );
    run(&mut sim, "attend menagerie");
    sim
}

/// The shipped solver, cast for real, at both budgets.
///
/// **Nothing exercised this spell before**, and that is the defect this test is
/// really about. `bide until` was withdrawn — the delay used to be read off the
/// circle, so the spell computed nothing — and the whole suite stayed green
/// while the shipped file became a line the orb cannot read. A dev spell nobody
/// casts is prose.
///
/// **The two halves only mean anything together.** One of them alone would pass
/// against a spell that never works, or against one that always does; the pair
/// is the domain's progression hook stated as an assertion — *unautomatable at
/// one instruction a tick, solved outright at two*. §19 and `dev_spells.toml`
/// both claim that in words and neither could check it.
#[cfg(debug_assertions)]
#[test]
fn the_shipped_solver_needs_a_second_step_and_then_closes_the_figure() {
    let mut lean = in_the_menagerie(11);
    run(&mut lean, "invoke chanting");
    lean.step_n(300);
    assert!(
        ever_said(&lean, "comes apart"),
        "one step a tick solved the menagerie: {:?}",
        said(&lean),
    );

    let mut ready = with_a_second_step(11);
    run(&mut ready, "invoke chanting");
    ready.step_n(300);
    assert!(
        ever_said(&ready, "the figure closes"),
        "two steps a tick did not close it: {:?}",
        said(&ready),
    );
    assert!(
        !ever_said(&ready, "comes apart"),
        "it collapsed a figure on the way: {:?}",
        said(&ready),
    );
}

/// Both refusals name the way forward, which is what keeps the verbs off
/// `is_gated`.
#[test]
fn the_refusals_say_what_to_do_instead() {
    let mut sim = in_the_menagerie(1);
    run(&mut sim, "sing skyward");
    assert!(ever_said(&sim, "summon first"), "{:?}", last(&sim));

    run(&mut sim, "summon");
    run(&mut sim, "summon");
    assert!(last(&sim).contains("already gathered"), "{:?}", last(&sim));
}
