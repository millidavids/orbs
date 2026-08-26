//! The sanctum, driven through the real parser and the real schedule.
//!
//! `tower::pylon`'s own tests prove the model and `tower::erosion`'s prove the
//! curve; these prove the *game* — that the words resolve, the readings publish,
//! a haul costs the tower nothing, and a finished course pays what
//! `progression.toml` says it does.
//!
//! **The solver here is the one `dev_spells.toml` ships**, run through `invoke`
//! rather than reimplemented. A Rust copy of the cyclic algorithm would prove
//! the algorithm, which `tower::pylon` already does; what is worth proving
//! here is that §8's language can *express* it — no variables in the loop body,
//! a part it has to call three times with different bindings, and a parity it
//! has to read off the world.

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;

/// The three stations, spelled out — a test that imported them from the sim could
/// not catch a rename that broke the *player's* vocabulary.
const STATIONS: [&str; 3] = ["wellspring", "conduit", "barrier"];

/// The shipped solver's name.
const SOLVER: &str = "holding";

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
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

/// Whether anything said so far contains `needle`.
fn ever_said(sim: &Sim, needle: &str) -> bool {
    said(sim).iter().any(|line| line.contains(needle))
}

/// Every field of every record, as one line each.
///
/// **`survey`'s answer is not a `Message`.** It emits `TableRow`s carrying a
/// `Name` and a `Quantity`, so [`said`] — which reads the message field — cannot
/// see a published reading at all. Two tests here were written against `said`
/// and passed for the wrong reason before they failed for the right one.
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

/// Whether any record carries `needle` in any of its fields.
fn ever_shown(sim: &Sim, needle: &str) -> bool {
    shown(sim).iter().any(|line| line.contains(needle))
}

/// Standing in the sanctum, with nothing done yet.
fn in_the_sanctum(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    run(&mut sim, "attend sanctum");
    sim
}

#[test]
fn a_course_goes_up_and_publishes_what_a_spell_can_ask() {
    let mut sim = in_the_sanctum(1);
    run(&mut sim, "muster");
    assert!(ever_said(&sim, "well up in the wellspring"));

    // The wellspring holds the whole course, so it has a `potency` — the smallest
    // ward, which is on top. The other two hold nothing and so publish nothing,
    // which is what makes `is empty` answerable about them.
    run(&mut sim, "survey wellspring");
    assert!(
        ever_shown(&sim, "potency"),
        "the station holding the course published no potency",
    );

    let before = shown(&sim).len();
    run(&mut sim, "survey barrier");
    let after: Vec<String> = shown(&sim).into_iter().skip(before).collect();
    assert!(
        !after.iter().any(|line| line.contains("potency")),
        "an empty station published a potency, so `is empty` can never be true of it",
    );
}

#[test]
fn the_barrier_says_how_it_stands_before_anything_has_happened() {
    // **The defect this is here for, and it was the worse of the two.** The
    // `integrity` reading was published by `erode`, which only writes when the
    // number moves — so for the first thirty ticks of every session there was no
    // reading at all. `survey pylon` printed nothing, and `watch::many_at`
    // answers an absent child with **nought**, so `if the pylon has fewer than
    // 60 integrity` was true of a barrier in perfect repair.
    //
    // A guard that fires hardest when nothing is wrong is the worst shape a
    // guard can have, and no test that mustered first could ever have seen it.
    let sim = in_the_sanctum(1);
    assert_eq!(sim.integrity(), 100);

    let mut sim = sim;
    let before = shown(&sim).len();
    run(&mut sim, "survey pylon");
    let after: Vec<String> = shown(&sim).into_iter().skip(before).collect();
    assert!(
        after
            .iter()
            .any(|line| line.contains("integrity") && line.contains("100")),
        "an untouched tower publishes no integrity at all: {after:?}",
    );
}

#[test]
fn a_haul_is_instant_and_takes_no_slot() {
    // **The decision this domain shares with the lens.** A course is a hundred
    // hauls; one that held the tower's single production slot would starve every
    // other spell into `spell_gave_up`, and a bound solver would be an
    // alternative to brewing rather than something that runs beside it.
    let mut sim = in_the_sanctum(1);
    run(&mut sim, "muster");
    assert!(
        sim.working().is_none(),
        "mustering took the production slot"
    );

    run(&mut sim, "haul wellspring barrier");
    assert!(sim.working().is_none(), "a haul took the production slot");
    assert!(ever_said(
        &sim,
        "drawn from the wellspring into the barrier"
    ));
}

#[test]
#[cfg(debug_assertions)]
fn a_scripted_course_runs_while_the_laboratory_is_busy() {
    // **The claim the whole domain rests on, and it shipped false.** `muster`
    // and `haul` are in `Verb::is_operation` — which is the *scope* question,
    // and what keeps them out of the tower-wide vocabulary — and
    // `spell::block::begins_work` reads that same predicate to decide whether a
    // scripted line must wait on the tower's one production slot. The `Dial`
    // exemption beside them was written for exactly this and these two were left
    // out of it.
    //
    // The symptom: `holding.spell waits: the alembic is distilling`, forever, and
    // then `spell_gave_up` at `PATIENCE` with a fault latched on the rail. Every
    // other test here runs in an idle tower, so none of them could see it.
    let mut sim = Sim::new(1);
    for line in [
        "attend laboratory",
        "kindle charcoal",
        "debug_spawn clarified-draught",
        "distil clarified-draught",
    ] {
        run(&mut sim, line);
    }
    assert!(sim.working().is_some(), "the alembic never started");

    run(&mut sim, "attend sanctum");
    run(&mut sim, &format!("invoke {SOLVER}"));
    sim.step_n(150);

    assert!(
        ever_said(&sim, "drawn from the"),
        "a bound solver hauled nothing while the laboratory was busy",
    );
    assert!(
        !ever_said(&sim, "waits: the alembic"),
        "the sanctum queued behind a brew — it takes no production slot",
    );
}

#[test]
fn the_rail_says_the_barrier_and_never_the_course() {
    // **The rail's own recorded defect, arriving for the third time.** The
    // pylon's meter was wards-still-to-haul while a course stood, and
    // `detail_of` prints a meter's *remainder* — so the sanctum read `py 4` and
    // counted **down** as a solver won, which to a player who last saw `py 100`
    // is a barrier about to fail. §19 records the same shape at `st 350t` and
    // `pr 4t`: *"two of the three built domains were glanceably wrong"*.
    //
    // Worse here than there, because the number did not merely mislead — it
    // *vanished* into course progress exactly while a bound solver was working,
    // which is the one time the player is in another room and glancing.
    let detail = |sim: &Sim| {
        sim.briefs()
            .into_iter()
            .find(|brief| brief.name == "sanctum")
            .and_then(|brief| brief.detail)
            .expect("the sanctum says nothing at all")
    };

    let mut sim = in_the_sanctum(1);
    let whole = detail(&sim);
    assert!(
        whole.contains("100") && whole.contains('%'),
        "a whole barrier does not read as a percentage: {whole:?}",
    );

    // Wear it, then draw a course: the reading must still be the barrier, and it
    // must have gone *down* from 100 rather than jumping to a ward count.
    sim.step_n(1200);
    run(&mut sim, "muster");
    let working = detail(&sim);
    assert!(
        working.contains('%'),
        "a drawn course took the rail's barrier reading away: {working:?}",
    );
    let standing: u32 = working
        .trim_end_matches('%')
        .split_whitespace()
        .next_back()
        .and_then(|word| word.parse().ok())
        .expect("the detail carries no number");
    assert_eq!(
        standing,
        sim.integrity(),
        "the rail and the resource disagree: {working:?}",
    );
    assert!(
        standing < 100 && standing > 10,
        "{standing} is a ward count, not a worn barrier",
    );
}

#[test]
fn the_one_rule_is_refused_in_voice_and_costs_nothing() {
    let mut sim = in_the_sanctum(1);
    run(&mut sim, "muster");
    run(&mut sim, "haul wellspring barrier");

    // The greater ward onto the lesser: the puzzle's only real refusal.
    run(&mut sim, "haul wellspring barrier");
    assert!(ever_said(&sim, "will not rest upon the lesser"));

    // ...and it changed nothing, which is §11.5's "never ruinous, only slower".
    //
    // **Read through `shown`, not `said`.** This asked `said` for `potency = 2`
    // and could never have failed: `survey` emits `TableRow`s with no `Message`
    // at all, and even `shown` renders the pair as `potency 2`. The claim in the
    // comment was untested.
    let before = shown(&sim).len();
    run(&mut sim, "survey barrier");
    let after: Vec<String> = shown(&sim).into_iter().skip(before).collect();
    assert!(
        after
            .iter()
            .any(|line| line.contains("potency") && line.contains('1')),
        "the barrier holds no ward at all: {after:?}",
    );
    assert!(
        !after
            .iter()
            .any(|line| line.contains("potency") && line.contains('2')),
        "the refused haul moved the ward anyway: {after:?}",
    );
}

#[test]
fn each_of_the_three_refusals_names_a_different_way_forward() {
    let mut sim = in_the_sanctum(1);

    // Before a course is up.
    run(&mut sim, "haul wellspring barrier");
    assert!(ever_said(&sim, "muster first"));

    run(&mut sim, "muster");
    run(&mut sim, "muster");
    assert!(ever_said(&sim, "already drawn"));

    run(&mut sim, "haul wellspring wellspring");
    assert!(ever_said(&sim, "where it already is"));

    run(&mut sim, "haul conduit barrier");
    assert!(ever_said(&sim, "nothing is resting at the conduit"));

    // A place that exists and is not a station.
    run(&mut sim, "haul wellspring pylon");
    assert!(ever_said(&sim, "is not a station"));
}

#[test]
fn the_words_mean_nothing_in_the_laboratory() {
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "muster");
    assert!(
        !ever_said(&sim, "well up in the wellspring"),
        "a course went up in the laboratory",
    );
}

#[test]
#[cfg(debug_assertions)]
fn the_shipped_solver_finishes_a_course_in_the_optimal_number_of_hauls() {
    // **The acceptance test for the whole domain.** §8's language has no
    // arguments to a part and no per-descent variables, so the only way to write
    // the cyclic solution is the one `holding` takes: bind `here` and `there`
    // before each call and let the part read them. If this stops finishing, the
    // language has lost something the domain depends on.
    //
    // Optimality is the sharp half. A wrong cycle direction still *finishes* —
    // in the conduit — and a ladder that guessed would take more hauls, so
    // the count is what catches both.
    for seed in [1, 3, 11] {
        let mut sim = in_the_sanctum(seed);
        run(&mut sim, &format!("invoke {SOLVER}"));
        sim.step_n(4000);

        let height = said(&sim)
            .iter()
            .find_map(|line| {
                // `muster_opens` leads with the count: `4 wards well up in…`.
                line.strip_suffix(", greatest beneath")
                    .filter(|line| line.contains("well up in the wellspring"))
                    .and_then(|line| line.split_whitespace().next())
                    .and_then(|count| count.parse::<u32>().ok())
            })
            .expect("the solver never mustered anything");

        let hauls = said(&sim)
            .iter()
            .find_map(|line| {
                line.strip_prefix("the last ward seats. ")
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|count| count.parse::<u32>().ok())
            })
            .unwrap_or_else(|| panic!("seed {seed}: the course of {height} never finished"));

        assert_eq!(
            hauls,
            (1u32 << height) - 1,
            "seed {seed}: a course of {height} took {hauls} hauls, not the optimal count",
        );
    }
}

#[test]
#[cfg(debug_assertions)]
fn a_bound_solver_holds_the_barrier_while_the_player_stands_elsewhere() {
    // **What the domain is for**, and the bug it would otherwise ship with: the
    // lens published its readings from `Cwd`, so a bound solver working while
    // the player was in the laboratory froze on its first move. `publish` taking
    // the pylon entity is what stops that here, and this is what proves it.
    let mut sim = in_the_sanctum(3);

    // **Earned, not counted.** Concentration costs 16 experience and a course
    // pays one or two, so the number of courses is a function of
    // `progression.toml` — a loop with a hard count would go red every time
    // somebody tuned the yield, which is the wrong thing for this test to be
    // sensitive to. The cap is a runaway guard, not a budget.
    for _ in 0..40 {
        if sim.concentration() > 0 {
            break;
        }
        run(&mut sim, &format!("invoke {SOLVER}"));
        sim.step_n(600);
    }
    assert!(
        sim.concentration() > 0,
        "forty courses did not earn a slot to bind into, at {} experience",
        sim.experience(),
    );

    run(&mut sim, &format!("bind {SOLVER}"));
    run(&mut sim, "attend laboratory");

    let before = sim.experience();
    sim.step_n(3600);
    assert!(
        sim.experience() > before,
        "a bound solver stopped earning the moment the player left the room",
    );

    // ...and the barrier is being held rather than merely worked on. An hour of
    // erosion is 120 points against a barrier of 100, so anything above nothing
    // means the spell is outrunning the decay.
    assert!(
        sim.integrity() > 0,
        "the barrier fell to nothing with a solver bound to hold it",
    );
}

#[test]
fn neglect_musters_a_taller_course_than_a_kept_tower_does() {
    // The whole of what erosion does today, through the real verbs. Two towers
    // on the same seed so the height difference is the deficit and not the
    // jitter.
    let mut kept = in_the_sanctum(11);
    run(&mut kept, "muster");

    let mut neglected = in_the_sanctum(11);
    neglected.step_n(3600);
    run(&mut neglected, "muster");

    let height_of = |sim: &Sim| {
        said(sim)
            .iter()
            .find_map(|line| {
                // `muster_opens` leads with the count: `4 wards well up in…`.
                line.strip_suffix(", greatest beneath")
                    .filter(|line| line.contains("well up in the wellspring"))
                    .and_then(|line| line.split_whitespace().next())
                    .and_then(|count| count.parse::<u32>().ok())
            })
            .expect("no course was mustered")
    };

    assert!(
        height_of(&neglected) > height_of(&kept),
        "an hour of neglect mustered {} wards against a kept tower's {}",
        height_of(&neglected),
        height_of(&kept),
    );
}

#[test]
fn the_barrier_wears_down_and_a_finished_course_puts_it_back() {
    let mut sim = in_the_sanctum(1);
    assert_eq!(sim.integrity(), 100);

    sim.step_n(1200);
    let worn = sim.integrity();
    assert!(worn < 100, "twenty minutes left the barrier untouched");

    run(&mut sim, "muster");
    run(&mut sim, "debug_course");
    run(&mut sim, "haul conduit barrier");
    assert!(
        sim.integrity() > worn,
        "a finished course put nothing back: {worn} then {}",
        sim.integrity(),
    );
    assert!(ever_said(&sim, "the barrier gains"));
}

#[test]
fn the_integrity_a_spell_reads_keeps_up_with_the_one_the_rail_draws() {
    // **The defect this is here for.** `erode` wears a resource down; the
    // pylon carries a *reading* of it, and the first version never
    // republished — so `survey pylon` and every `if the pylon has fewer than
    // n integrity` went on reporting a whole wall while the rail counted down
    // beside them. Two surfaces, one number, and only the one a spell reads was
    // wrong.
    let mut sim = in_the_sanctum(1);
    sim.step_n(900);

    let standing = sim.integrity();
    assert!(standing < 100);

    let before = shown(&sim).len();
    run(&mut sim, "survey pylon");
    let after: Vec<String> = shown(&sim).into_iter().skip(before).collect();
    assert!(
        after
            .iter()
            .any(|line| line.contains("integrity") && line.contains(&standing.to_string())),
        "the published reading does not say {standing}, which is what the tower stands at: {after:?}",
    );
}

#[test]
fn every_station_answers_to_its_own_name_and_to_no_other() {
    // **This asserted nothing at all.** `said(&sim).len() >= before` is monotone
    // and trivially true, and `is not a station` is a *haul* refusal that
    // `survey` cannot produce — in a test that issues no hauls. What it is meant
    // to prove is that each name reaches its own station and no other, so it
    // hauls a ward onto each in turn and reads the potency back.
    let mut sim = in_the_sanctum(1);
    run(&mut sim, "muster");

    // Ward 1 walks all three, and each station reports it while it is there.
    for (from, to) in [
        (STATIONS[0], STATIONS[1]),
        (STATIONS[1], STATIONS[2]),
        (STATIONS[2], STATIONS[0]),
    ] {
        run(&mut sim, &format!("haul {from} {to}"));
        assert!(
            ever_said(&sim, &format!("into the {to}")),
            "`haul {from} {to}` did not reach the {to}",
        );

        let before = shown(&sim).len();
        run(&mut sim, &format!("survey {to}"));
        let after: Vec<String> = shown(&sim).into_iter().skip(before).collect();
        assert!(
            after
                .iter()
                .any(|line| line.contains("potency") && line.contains('1')),
            "the ward went somewhere other than the {to}: {after:?}",
        );
    }
}
