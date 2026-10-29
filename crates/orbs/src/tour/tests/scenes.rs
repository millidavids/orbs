//! What a scene starts from: a tower kept nowhere, the defaults, the clock,
//! the boot card and the threshold.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::{SECOND, TWO, app, reach, run, scenes, tour};
use crate::shell::Standing;
use crate::sim::{Kept, Tower};

#[test]
fn a_tour_tower_is_kept_nowhere() {
    let mut app = app(scenes(TWO));
    assert_eq!(app.world().resource::<Kept>().path(), None);
    run(&mut app, 12 * SECOND);
    assert_eq!(
        tour(&app).at(),
        1,
        "the tour never reached its second scene"
    );
    assert_eq!(app.world().resource::<Kept>().path(), None);
    assert_eq!(
        app.world().resource::<Tower>().sim().domain().as_deref(),
        Some("archive")
    );
}

#[test]
fn settings_go_back_to_the_defaults_between_scenes() {
    // A scene's own setting, and one a key changed during it: neither outlives
    // it. Motion and the linear stream run headless; the tube, the phosphor
    // and the sight have no systems here, and their reset is the window's to
    // show.
    let mut app = app(scenes(
        "[[scene]]\nname = \"test.still\"\nabout = \"a\"\nhold = 0.5\n\
         settings = { motion = \"off\" }\nsteps = [{ keys = \"<f5>\" }]\n\
         [[scene]]\nname = \"test.after\"\nabout = \"b\"\nhold = 0.5\n",
    ));
    let still = |app: &App| app.world().resource::<crate::shell::Stillness>().still();
    let linear = |app: &App| app.world().resource::<orbs_shell::Linear>().showing();
    assert!(still(&app), "the scene's own setting did not take");
    let mut pressed = false;
    while tour(&app).at() == 0 {
        app.update();
        pressed |= linear(&app);
    }
    assert!(pressed, "F5 never reached the linear stream");
    reach(&mut app, 1);
    assert!(!still(&app), "motion stayed off into the next scene");
    assert!(
        !linear(&app),
        "the linear stream stayed up into the next scene"
    );
}

#[test]
fn a_scenes_first_tick_comes_a_second_after_it_appears() {
    // Whatever the last scene left owing on the fixed clock.
    let mut app = app(scenes(
        "[[scene]]\nname = \"test.one\"\nabout = \"a\"\nhold = 0.7\n\
         [[scene]]\nname = \"test.two\"\nabout = \"b\"\nsetup = \"meditate 600\"\n",
    ));
    reach(&mut app, 1);
    let tick = |app: &App| app.world().resource::<Tower>().sim().tick().get();
    let begun = tick(&app);
    // The frame after `begin` is as long as its staging took.
    let frame = |app: &mut App, length: Duration| {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(length));
    };
    frame(&mut app, Duration::from_secs(3));
    app.update();
    frame(&mut app, Duration::from_secs(1) / SECOND);
    run(&mut app, SECOND - 10);
    assert_eq!(
        tick(&app),
        begun,
        "a tick came before the scene's first second"
    );
    run(&mut app, 20);
    assert!(
        tick(&app) > begun,
        "no tick came after the scene's first second"
    );
}

#[test]
fn a_boot_scene_ends_live_and_still_means_still() {
    let card = |still: bool| {
        format!(
            "[[scene]]\nname = \"test.card\"\nabout = \"the card\"\nboot = true\n\
             settings = {{ motion = \"{}\" }}\n",
            if still { "off" } else { "on" }
        )
    };
    for still in [false, true] {
        let mut app = app(scenes(&card(still)));
        let mut closed = false;
        for _ in 0..15 * SECOND {
            app.update();
            closed |=
                app.world().resource::<orbs_shell::Boot>().stage() == orbs_shell::Stage::Close;
        }
        assert!(
            app.world().resource::<orbs_shell::Boot>().is_live(),
            "the replayed card never handed over"
        );
        assert_eq!(closed, !still, "motion {} and the card closing", !still);
    }
}

#[test]
fn a_second_threshold_reopens_without_complaint() {
    // The latch is the tower's: a later threshold scene is a new one.
    let mut app = app(scenes(
        "[[scene]]\nname = \"test.one\"\nabout = \"a\"\ntower = \"threshold\"\nhold = 0.5\n\
         [[scene]]\nname = \"test.mid\"\nabout = \"b\"\nhold = 0.5\n\
         [[scene]]\nname = \"test.two\"\nabout = \"c\"\ntower = \"threshold\"\nhold = 0.5\n",
    ));
    let opened = |app: &App| {
        app.world()
            .resource::<crate::shell::ThresholdOpened>()
            .is_set()
    };
    let mut between = 0;
    for _ in 0..3 * SECOND {
        app.update();
        if tour(&app).at() == 1 && !tour(&app).is_due() {
            between += 1;
            assert!(!opened(&app), "a new tower kept the last one's latch");
        }
    }
    assert!(between > 0, "the tour never showed the scene between");
    assert_eq!(tour(&app).at(), 2);
    assert!(app.world().resource::<Standing>().is_open());
    assert!(opened(&app));
}
