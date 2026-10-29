//! The catalogue held to what it claims.

use std::collections::BTreeSet;

use orbs_sim::tower::{Craft, State};

use super::*;

#[test]
fn the_catalogue_reads_and_every_name_is_its_own() {
    let catalogue = Catalogue::builtin();
    let mut seen = BTreeSet::new();
    for scene in catalogue.scenes() {
        let name = scene.name.as_str();
        assert!(seen.insert(name), "{name} is in the catalogue twice");
        assert!(
            name.contains('.')
                && name.chars().all(|c| c.is_ascii_lowercase()
                    || c.is_ascii_digit()
                    || matches!(c, '.' | '_')),
            "{name} is not `group.name`",
        );
        assert!(!scene.about.is_empty(), "{name} says nothing about itself");
        assert!(
            scene.hold > 0.0 && scene.hold <= 30.0,
            "{name} holds {}s",
            scene.hold
        );
        for shot in scene.shot_times() {
            assert!((0.0..30.0).contains(&shot), "{name} shoots at {shot}s");
        }
        for step in &scene.steps {
            for shot in &step.shots {
                assert!((0.0..10.0).contains(shot), "{name} shoots at +{shot}s");
            }
            // The orb asks before it leaves, and a second `quit` would end the
            // tour: only the scene that shows the question types it, once.
            assert!(
                step.line.as_deref() != Some("quit") || name == "orb.quit",
                "{name} types `quit` at the prompt",
            );
        }
        if let Some(edit) = &scene.edit {
            let last = edit
                .replace("\\n", "\n")
                .lines()
                .map(str::trim)
                .rfind(|segment| !segment.is_empty())
                .map(str::to_owned);
            assert!(
                matches!(last.as_deref(), Some("quit" | "wq")),
                "{name}'s `edit` leaves the editor open, and setup would lose it",
            );
        }
        if scene.length.is_some() {
            assert_eq!(
                scene.tower,
                Start::Sealed,
                "{name}: only a sealed tower has a length"
            );
            assert!(scene.length().is_some(), "{name} names no length there is");
        }
    }
}

#[test]
fn a_scene_the_window_could_not_play_does_not_read() {
    let scene = |extra: &str| format!("[[scene]]\nname = \"t.t\"\nabout = \"t\"\n{extra}\n");
    for unplayable in [
        "steps = [{ keys = \"\" }]",
        "steps = [{ pause = 2.0 }]",
        "steps = [{ keys = \"<rigth>\" }]",
        "strain = [nan, 0.0]",
        "hold = 0.0",
        "shots = [inf]",
    ] {
        assert!(
            Catalogue::read(&scene(unplayable)).is_err(),
            "{unplayable} read"
        );
    }
    assert!(Catalogue::read(&scene("steps = [{ keys = \"<f3>\" }]")).is_ok());
}

#[test]
fn every_scene_reaches_what_it_expects() {
    for scene in Catalogue::builtin().scenes() {
        let mut staged = stage(scene, None);
        assert!(
            staged.unread.is_empty(),
            "{}: setup could not read {:?}",
            scene.name,
            staged.unread
        );
        let short = scene.expect.shortfall(&staged.sim);
        assert!(short.is_empty(), "{}: {}", scene.name, short.join("; "));
        if let Some(surface) = scene.expect.surface {
            assert!(
                asked_for(&mut staged, surface),
                "{}: setup left no {surface:?} up",
                scene.name
            );
        }
    }
}

#[test]
fn the_scenes_cover_the_game() {
    // From claims the tests above and the in-App run verify, never from a list
    // of what a scene says it is about.
    let catalogue = Catalogue::builtin();
    let scenes = catalogue.scenes();
    let claims = || scenes.iter().flat_map(|scene| [&scene.expect, &scene.ends]);

    // Every room a player tends, and the bailey. The grimoire is no domain —
    // `domain_of` names none for it — so its scene can claim no room.
    let rooms: BTreeSet<&str> = claims().filter_map(|claim| claim.room.as_deref()).collect();
    for room in orbs_sim::tower::DOMAINS.iter().chain(&["bailey"]) {
        assert!(rooms.contains(room), "no scene stands in the {room}");
    }

    // Every craft, at work where a scene's setup leaves it; and every state an
    // instrument is in, which the rail and the panel each draw their own way.
    let mut crafts = BTreeSet::new();
    let mut states: BTreeSet<&str> = claims()
        .flat_map(|claim| claim.states.values().map(String::as_str))
        .collect();
    for scene in scenes
        .iter()
        .filter(|scene| !scene.expect.working.is_empty())
    {
        for instrument in stage(scene, None).sim.instruments() {
            if scene.expect.working.contains(&instrument.name) && moving(instrument.state) {
                crafts.insert(format!("{:?}", instrument.craft));
                states.insert(instrument.state.label());
            }
        }
    }
    for craft in [
        Craft::Grinding,
        Craft::Digesting,
        Craft::Combining,
        Craft::Distilling,
        Craft::Heating,
        Craft::Reading,
        Craft::Scrying,
        Craft::Warding,
        Craft::Imbuing,
        Craft::Summoning,
        Craft::Assembling,
    ] {
        assert!(
            crafts.contains(&format!("{craft:?}")),
            "no scene shows {craft:?} at work"
        );
    }
    for state in [
        State::Empty,
        State::Charged,
        State::Gathering,
        State::Working,
        State::Scouring,
        State::Ready,
        State::Fouled,
        State::Burning,
        State::Banked,
        State::Cold,
    ] {
        assert!(
            states.contains(state.label()),
            "no scene shows an instrument {}",
            state.label()
        );
    }

    // Every surface: claimed up, or waited on by a step, which never starts
    // if it never comes.
    let mut surfaces: Vec<Surface> = claims().filter_map(|claim| claim.surface).collect();
    for step in scenes.iter().flat_map(|scene| &scene.steps) {
        surfaces.extend(match step.when {
            Some(When::Editor) => Some(Surface::Editor),
            Some(When::Weave) => Some(Surface::Weave),
            Some(When::Maze) => Some(Surface::Maze),
            Some(When::Menu) => Some(Surface::Menu),
            Some(When::Manual) => Some(Surface::Manual),
            Some(When::Live) | None => None,
        });
    }
    for surface in [
        Surface::Editor,
        Surface::Weave,
        Surface::Maze,
        Surface::Menu,
        Surface::Manual,
        Surface::Install,
        Surface::Threshold,
    ] {
        assert!(
            surfaces.contains(&surface),
            "no scene opens the {surface:?}"
        );
    }

    // Every function key a person can press at the glass.
    let pressed: Vec<Stroke> = scenes
        .iter()
        .flat_map(|scene| &scene.steps)
        .flat_map(|step| step.strokes().unwrap_or_default())
        .collect();
    for key in [Named::F2, Named::F3, Named::F5, Named::F7, Named::F8] {
        assert!(
            pressed.contains(&Stroke::Named(key)),
            "no scene presses {key:?}"
        );
    }

    // The boot card moving and still, and the glass strained.
    let still = |scene: &&Scene| scene.settings.get("motion").map(String::as_str) == Some("off");
    assert!(scenes.iter().any(|scene| scene.boot && !still(&scene)));
    assert!(scenes.iter().any(|scene| scene.boot && still(&scene)));
    assert!(scenes.iter().any(|scene| scene.strain.is_some()));
}

#[test]
fn a_claim_the_world_does_not_hold_is_named() {
    // The check above is only as good as this: every field can fail.
    let catalogue = Catalogue::builtin();
    let grinding = catalogue.select("laboratory.grind").unwrap_or_default();
    let staged = stage(grinding[0], None);
    let wrong = Expect {
        room: Some("forge".to_owned()),
        working: vec!["alembic".to_owned(), "lattice".to_owned()],
        states: [("athanor".to_owned(), "cold".to_owned())].into(),
        surface: None,
        said: Some("zorb".to_owned()),
    };
    assert_eq!(
        wrong.shortfall(&staged.sim).len(),
        5,
        "{:?}",
        wrong.shortfall(&staged.sim)
    );
}

#[test]
fn a_resumed_scene_still_names_the_setup_it_could_not_read() {
    // A resume restores the tower, and a restored log starts empty.
    let catalogue = Catalogue::read(
        r#"
        [[scene]]
        name = "orb.lost"
        about = "a line that reads as nothing"
        setup = "attend laboratory; kindle zzz"
        resumed = 3600
        "#,
    )
    .unwrap_or_else(|why| panic!("{why}"));
    let staged = stage(&catalogue.scenes()[0], None);
    assert_eq!(staged.unread, ["kindle zzz"]);
}

/// Whether setup left `surface` asked for. Taken: the sim hands each over once.
fn asked_for(staged: &mut Staged, surface: Surface) -> bool {
    match surface {
        Surface::Editor => staged.sim.opening().is_some(),
        Surface::Weave => staged.sim.weaving(),
        Surface::Maze => staged.sim.wandering(),
        Surface::Menu => staged.sim.menuing(),
        Surface::Install => staged.install.is_some(),
        Surface::Threshold => staged.threshold,
        // Only a menu word opens it, which is a step.
        Surface::Manual => false,
    }
}

#[test]
fn a_term_selects_a_scene_a_group_or_everything() {
    let catalogue = Catalogue::builtin();
    let names = |terms: &str| -> Vec<String> {
        catalogue
            .select(terms)
            .unwrap_or_default()
            .into_iter()
            .map(|scene| scene.name.clone())
            .collect()
    };
    assert_eq!(names("forge.lattice"), ["forge.lattice"]);
    assert!(
        names("laboratory")
            .iter()
            .all(|name| name.starts_with("laboratory."))
    );
    assert!(names("laboratory").len() > 1);
    // A group is cut at a dot, never inside a word.
    assert!(names("lab").is_empty());
    assert_eq!(names("all").len(), catalogue.scenes().len());
    // Each scene once, in the order the terms asked.
    let asked = names("forge.lattice, laboratory.grind, forge");
    assert_eq!(asked[..2], ["forge.lattice", "laboratory.grind"]);
    assert_eq!(
        asked.iter().filter(|name| *name == "forge.lattice").count(),
        1
    );
    assert!(asked[2..].iter().all(|name| name.starts_with("forge.")));
    // A near miss names the scenes it was near; nothing near names the groups.
    let refused = catalogue.select("lattic").err().unwrap_or_default();
    assert!(refused.contains("forge.lattice"), "{refused}");
    let refused = catalogue.select("xyzzy").err().unwrap_or_default();
    assert!(refused.contains("laboratory"), "{refused}");
}
