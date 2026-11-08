//! Lines the orb answers and never runs (DESIGN.md §3, §19 *Both tongues*).

mod common;

use common::{at_the_wall, ever_said, run, said};
use orbs_render::Presentation;
use orbs_sim::parser::{AsideKind, Mode, Resolution, Verb, analyse, every_aside};
use orbs_sim::{Prose, Sim};

fn line(key: &str) -> String {
    Prose::builtin().line(key, &[])
}

#[test]
fn every_aside_is_answered_and_runs_nothing() {
    for (said_line, aside) in every_aside() {
        let mut sim = Sim::new(1);
        run(&mut sim, "attend laboratory");
        let before = said(&sim).len();
        run(&mut sim, said_line);
        sim.step_n(3);
        let after = &said(&sim)[before..];
        assert!(
            after.iter().any(|text| *text == line(aside.key)),
            "{said_line:?} was not answered with {}: {after:?}",
            aside.key,
        );
        assert!(
            after.iter().all(|text| !text.contains("the orb takes up")),
            "{said_line:?} ran something: {after:?}",
        );
    }
}

#[test]
fn kill_never_starts_the_spell_it_names() {
    // The reader once read `kill first_light` as `invoke first_light`.
    let mut sim = Sim::new(1);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "kill first_light");
    sim.step_n(5);
    assert!(ever_said(&sim, &line("aside_kill")), "{:?}", said(&sim));
    assert!(!ever_said(&sim, "first_light.spell"), "{:?}", said(&sim));
}

#[test]
fn sudo_no_longer_reaches_a_verb() {
    let sim = Sim::new(1);
    let analysis = analyse("sudo grind sage", sim.scene(), Mode::Calm);
    assert!(matches!(analysis.resolution, Resolution::Aside { .. }));
    assert!(analysis.reads_outright(), "an aside went to the reader");
}

#[test]
fn the_register_alone_never_quiets_the_prompt() {
    // `F7` moves the register and no journal records it, so a replay would say
    // something else if the prompt listened to it. The menu may; it writes
    // nothing into the record.
    let mut sim = Sim::new(1);
    sim.set_register(Presentation::Eldritch);
    run(&mut sim, "xyzzy");
    assert!(ever_said(&sim, &line("aside_xyzzy")), "{:?}", said(&sim));
}

#[test]
fn a_joke_is_quiet_in_a_siege_and_a_pointer_is_not() {
    let mut sim = at_the_wall(1);
    run(&mut sim, "defend");
    run(&mut sim, "xyzzy");
    run(&mut sim, "exit");
    assert!(!ever_said(&sim, &line("aside_xyzzy")), "{:?}", said(&sim));
    assert!(ever_said(&sim, &line("aside_exit")), "{:?}", said(&sim));

    // What follows `sudo` is offered in its place, and never run.
    let before = said(&sim).len();
    run(&mut sim, "sudo hold");
    let after = &said(&sim)[before..];
    assert!(!after.contains(&line("aside_sudo")), "{after:?}");
    assert!(
        after.iter().any(|text| text == "hold"),
        "hold was not offered: {after:?}"
    );
    assert!(
        !after.iter().any(|text| text.starts_with("they ")),
        "a round resolved: {after:?}"
    );
}

/// What a line said in a siege comes to, against the scene where it was said.
fn in_a_siege(said: &str) -> orbs_sim::parser::Analysis {
    let sim = Sim::new(1);
    analyse(said, &sim.scene().clone().besieged(true), Mode::Calm)
}

#[test]
fn a_quiet_joke_offers_what_the_matcher_would_have() {
    let analysis = in_a_siege("sudo status");
    assert!(
        matches!(
            &analysis.resolution,
            Resolution::Aside { quiet: Some(offered), .. } if offered == &[Verb::Status]
        ),
        "{:?}",
        analysis.resolution,
    );
    // Still the orb's to answer: a reader handed it would run `status`.
    assert!(analysis.reads_outright());
}

#[test]
fn a_quiet_joke_offers_nothing_a_table_answers() {
    // `exit` alone fuzzed to `scribe`, and `sudo` to `research`.
    for said in ["sudo exit", "sudo sudo", "sudo wait"] {
        let analysis = in_a_siege(said);
        assert!(
            matches!(
                &analysis.resolution,
                Resolution::Aside { quiet: Some(offered), .. } if offered.is_empty()
            ),
            "{said:?}: {:?}",
            analysis.resolution,
        );
    }
}

#[test]
fn an_aside_said_in_calm_is_never_scored() {
    // Scored, `kill first_light` filled the parse log with `invoke` readings
    // the orb never considered.
    let sim = Sim::new(1);
    for said in ["kill first_light", "sudo grind sage", "xyzzy"] {
        let analysis = analyse(said, sim.scene(), Mode::Calm);
        assert!(analysis.candidates.is_empty(), "{said:?} was scored");
        assert!(
            matches!(analysis.resolution, Resolution::Aside { quiet: None, .. }),
            "{said:?}"
        );
    }
}

#[test]
fn a_pointer_is_said_in_a_siege() {
    for (said, aside) in every_aside().filter(|(_, aside)| aside.kind == AsideKind::Pointer) {
        let analysis = in_a_siege(said);
        assert_eq!(
            analysis.resolution,
            Resolution::Aside { aside, quiet: None },
            "{said:?}",
        );
    }
}
