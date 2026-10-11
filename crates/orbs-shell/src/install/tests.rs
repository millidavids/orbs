//! The installation, driven over a real practice tower.
//!
//! The lesson is typed rather than read past, so the tests type it: each step's
//! own `try` line, from `prose.toml`, has to finish that step and no other.

use std::path::PathBuf;

use orbs_sim::content::Length;
use orbs_sim::parser::Outcome;
use orbs_sim::{Augur, Fixture, Sim};

use super::paint::split;
use super::state::{Install, Raise, SKIP, Step, WAKE, from_value};

fn raise() -> Raise {
    Raise {
        keep: None,
        length: Length::Baseline,
        seed: 3,
        wizard: None,
    }
}

fn begun() -> (Sim, Install) {
    let begun = raise().begin(true);
    (begun.sim, begun.install.expect("installing"))
}

/// A line at the prompt, then the tick the clock would give it.
fn typed(sim: &mut Sim, install: &mut Install, line: &str) -> Option<Raise> {
    typed_with(sim, install, line, None)
}

fn typed_with(
    sim: &mut Sim,
    install: &mut Install,
    line: &str,
    augur: Option<&dyn Augur>,
) -> Option<Raise> {
    let woke = install.submit(sim, line, augur);
    if woke.is_none() {
        sim.step();
        install.observe(sim);
    }
    woke
}

fn attempt(sim: &Sim, step: Step) -> String {
    sim.prose()
        .line(&format!("install_{}_try", step.key()), &[])
}

/// Every step before `step`, finished by its own line.
fn reached(step: Step) -> (Sim, Install) {
    let (mut sim, mut install) = begun();
    for earlier in Step::ALL.into_iter().take_while(|each| *each != step) {
        let line = attempt(&sim, earlier);
        typed(&mut sim, &mut install, &line);
    }
    assert_eq!(install.step(), Some(step));
    (sim, install)
}

#[test]
fn every_step_is_finished_by_the_line_it_shows() {
    let (mut sim, mut install) = begun();
    for (at, step) in Step::ALL.into_iter().enumerate() {
        assert_eq!(install.step(), Some(step));
        let line = attempt(&sim, step);
        assert!(typed(&mut sim, &mut install, &line).is_none());
        assert_eq!(
            install.step(),
            Step::ALL.get(at + 1).copied(),
            "{line:?} did not finish {step:?} alone",
        );
    }
    assert!(install.is_complete());
}

#[test]
fn one_line_finishes_one_step() {
    // A shell's `cd` moves the player and is not the arcane register, so it
    // would finish `speech` too if the record it left were judged twice.
    let (mut sim, mut install) = reached(Step::Paths);
    typed(&mut sim, &mut install, "cd laboratory");
    assert_eq!(install.step(), Some(Step::Speech));

    // `overview` is `status` in plain words.
    typed(&mut sim, &mut install, "overview");
    assert_eq!(install.step(), Some(Step::Report));

    // `look` in a room finishes `sight`, then `paths` holds already, and
    // `speech` waits for a line of its own.
    let (mut sim, mut install) = reached(Step::Sight);
    typed(&mut sim, &mut install, "attend laboratory");
    assert_eq!(install.step(), Some(Step::Sight));
    typed(&mut sim, &mut install, "look");
    assert_eq!(install.step(), Some(Step::Speech));
}

#[test]
fn two_lines_before_one_look_are_each_judged() {
    // Two submits in one frame reach `observe` together.
    let (mut sim, mut install) = begun();
    sim.submit("help");
    sim.submit("survey");
    sim.step();
    install.observe(&sim);
    assert_eq!(install.step(), Some(Step::Paths));
}

#[test]
fn a_step_is_not_finished_by_the_wrong_thing() {
    let (mut sim, mut install) = begun();
    typed(&mut sim, &mut install, "status");
    typed(&mut sim, &mut install, "frobnicate the moon");
    assert_eq!(install.step(), Some(Step::Listening));

    let (mut sim, mut install) = reached(Step::Speech);
    // The arcane form, an `Incomplete` plain word, and a numbered answer.
    typed(&mut sim, &mut install, "attend tower");
    typed(&mut sim, &mut install, "go");
    typed(&mut sim, &mut install, "attend laboratory");
    typed(&mut sim, &mut install, "purge");
    assert!(!sim.choices().is_empty(), "`purge` asked nothing");
    typed(&mut sim, &mut install, "1");
    assert_eq!(install.step(), Some(Step::Speech));

    typed(&mut sim, &mut install, "look");
    assert_eq!(install.step(), Some(Step::Report));
}

#[test]
fn a_reading_counts_only_when_it_resolved() {
    // `status` takes nothing, so this reading is `TakesNothing`: kept, stamped
    // `divined`, and it ran nothing.
    let reader = Fixture::new()
        .reading("xyzzy plugh", "status laboratory")
        .reading("xyzzy home", "attend tower");

    let (mut sim, mut install) = reached(Step::Speech);
    typed_with(&mut sim, &mut install, "xyzzy plugh", Some(&reader));
    let last = sim.parse_log().records().last().expect("a record");
    assert!(last.divined, "the reader was not consulted: {last:?}");
    assert_ne!(last.outcome, Outcome::Resolved);
    assert_eq!(install.step(), Some(Step::Speech));

    typed_with(&mut sim, &mut install, "xyzzy home", Some(&reader));
    assert_eq!(install.step(), Some(Step::Report));
}

#[test]
fn skip_wakes_from_any_step_and_is_never_submitted() {
    for step in Step::ALL {
        let (mut sim, mut install) = reached(step);
        let before = sim.submissions().all().len();
        let woke = install.submit(&mut sim, "  Skip ", None);
        assert_eq!(woke.as_ref(), Some(install.raise()), "at {step:?}");
        assert_eq!(sim.submissions().all().len(), before, "at {step:?}");
    }
}

#[test]
fn when_done_wake_wakes_and_a_bare_enter_does_not() {
    // Before then `wake` is a line like any other, and reaches the tower.
    let (mut sim, mut install) = begun();
    let before = sim.submissions().all().len();
    assert!(typed(&mut sim, &mut install, WAKE).is_none());
    assert_eq!(sim.submissions().all().len(), before + 1);

    for step in Step::ALL {
        let line = attempt(&sim, step);
        typed(&mut sim, &mut install, &line);
    }
    assert!(install.is_complete());
    // A bare Enter is a habit, and a player still practising keeps the tower.
    assert!(typed(&mut sim, &mut install, "").is_none());
    // A question left unanswered costs nothing (§6): another line still runs.
    let before = sim.submissions().all().len();
    assert!(typed(&mut sim, &mut install, "survey").is_none());
    assert_eq!(sim.submissions().all().len(), before + 1);
    assert!(install.submit(&mut sim, " Wake ", None).is_some());
    assert_eq!(sim.submissions().all().len(), before + 1);
}

#[test]
fn the_strip_names_the_words_it_listens_for() {
    let sim = Sim::new(0);
    let names = |key: &str, word: &str| {
        sim.prose()
            .line(key, &[])
            .split(|c: char| !c.is_ascii_alphabetic())
            .any(|each| each == word)
    };
    assert!(
        names("install_skip", SKIP),
        "install_skip does not say {SKIP}"
    );
    assert!(
        names("install_done_2", WAKE),
        "install_done_2 does not say {WAKE}"
    );
}

#[test]
fn waking_hands_back_what_new_asked_for() {
    let raise = Raise {
        keep: Some(PathBuf::from("/nowhere/orbs-save-3.toml")),
        length: Length::Long,
        seed: 41,
        wizard: Some("morgan".to_owned()),
    };
    let begun = raise.clone().begin(true);
    assert_eq!(begun.keep, None, "a practice tower was kept");
    let (mut sim, mut install) = (begun.sim, begun.install.expect("installing"));
    let woke = install.submit(&mut sim, "skip", None);
    assert_eq!(woke, Some(raise.clone()));

    let woken = raise.wake_beside(None);
    assert_eq!(woken.keep, raise.keep, "an empty slot is kept as chosen");
    assert!(woken.install.is_none());
    let tower = woken.sim;
    assert_eq!(tower.seed(), sim.seed());
    assert_eq!(tower.prompt(), sim.prompt());
    assert_eq!(tower.tick().get(), 0);
    assert!(tower.submissions().all().is_empty());
}

#[test]
fn a_slot_taken_while_installing_is_traded_for_a_free_one() {
    let dir = std::env::temp_dir().join(format!("orbs-install-slot-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).expect("a temp directory");
    let base = dir.join("orbs-save.toml");
    let two = crate::save::slot_path(&base, 2);
    let raise = Raise {
        keep: Some(two.clone()),
        ..raise()
    };
    let moved = Sim::new(0).prose().line("save_moved", &[("count", "1")]);
    let woken = raise.wake_beside(Some(&base));
    assert_eq!(woken.keep, Some(two.clone()));
    assert!(
        !said(&woken.sim, &moved),
        "a slot kept as chosen said it moved"
    );

    // A second orb wrote it while this one was installing.
    crate::save::write_to(&two, &Sim::new(5).snapshot()).expect("a save");
    let woken = raise.wake_beside(Some(&base));
    assert_eq!(woken.keep, Some(base));
    assert!(said(&woken.sim, &moved), "nothing said where it is kept");
    assert_eq!(raise.wake_beside(None).keep, None);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_tower_with_no_slot_left_says_it_will_not_be_kept() {
    let dir = std::env::temp_dir().join(format!("orbs-install-full-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).expect("a temp directory");
    let base = dir.join("orbs-save.toml");
    for slot in 1..=crate::save::SLOTS {
        let path = crate::save::slot_path(&base, slot);
        crate::save::write_to(&path, &Sim::new(5).snapshot()).expect("a save");
    }
    let full = Raise {
        keep: Some(crate::save::slot_path(&base, 2)),
        ..raise()
    };

    let woken = full.wake_beside(Some(&base));
    assert_eq!(woken.keep, None);
    let failed = woken.sim.prose().line("save_failed", &[]);
    assert!(
        said(&woken.sim, &failed),
        "nothing said it will not be kept"
    );
    // And nothing to say for a tower that never asked to be kept.
    let quiet = raise().wake_beside(Some(&base)).sim;
    assert!(!said(&quiet, &failed));

    std::fs::remove_dir_all(&dir).ok();
}

fn said(sim: &Sim, message: &str) -> bool {
    sim.scrollback().records().iter().any(|record| {
        matches!(
            record.field(orbs_render::FieldName::Message),
            Some(orbs_render::Value::Text(text)) if text == message
        )
    })
}

#[test]
fn every_step_has_its_prose() {
    let sim = Sim::new(0);
    let prose = sim.prose();
    let mut keys: Vec<String> = [
        "title", "step", "skip", "try", "ok", "now", "done_1", "done_2",
    ]
    .iter()
    .map(|key| format!("install_{key}"))
    .collect();
    for step in Step::ALL {
        for part in ["name", "1", "2", "try"] {
            keys.push(format!("install_{}_{part}", step.key()));
        }
    }
    for key in keys {
        assert!(prose.has(&key), "{key} is not in prose.toml");
    }
}

#[test]
fn the_strip_yields_rows_before_the_transcript_does() {
    use orbs_render::Rect;

    // The 80×22 floor's body, and one too short for the whole strip.
    assert_eq!(split(Rect::new(1, 1, 78, 18)).area.rows, 6);
    assert_eq!(split(Rect::new(1, 1, 78, 10)).area.rows, 4);
    assert_eq!(split(Rect::new(1, 1, 78, 6)).area.rows, 0);
}

#[test]
fn the_switch_outranks_the_setting_and_blank_does_not() {
    for setting in [true, false] {
        assert!(!from_value(Some("0"), setting));
        assert!(from_value(Some("1"), setting));
        assert_eq!(from_value(Some(""), setting), setting);
        assert_eq!(from_value(Some("  "), setting), setting);
        assert_eq!(from_value(Some("maybe"), setting), setting);
        assert_eq!(from_value(None, setting), setting);
    }
}
