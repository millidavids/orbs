//! The whole catalogue, played: every scene's steps through the real keyboard
//! path, every `ends` held, and every frame showing all it says.

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use orbs_render::{GRID, GridSize, MIN_GRID, PICTURE};
use orbs_shell::tour::{Catalogue, Scene, Surface};

use super::{SECOND, Tour, app, tour};
use crate::render::{Canvas, repaint};
use crate::shell::{Editing, Loom, Reading, Screen, ShellSystems, Standing, Walk};
use crate::sim::{Kept, Tower};

#[test]
fn the_settings_a_scene_names_are_real_rows() {
    // Here, not in `orbs-shell`: the tube's and the theme's rows are this
    // build's.
    let rows = crate::shell::default_settings();
    for scene in Catalogue::builtin().scenes() {
        for (key, value) in &scene.settings {
            let row = rows.iter().find(|row| row.key == *key);
            assert!(
                row.is_some_and(|row| row.values.contains(value)),
                "{}: {key} = {value} is not a setting this build has",
                scene.name
            );
        }
    }
}

/// Whether `surface` is up in `world`.
fn up(world: &World, surface: Surface) -> bool {
    match surface {
        Surface::Editor => world.resource::<Editing>().is_open(),
        Surface::Weave => world.resource::<Loom>().is_open(),
        Surface::Maze => world.resource::<Walk>().is_open(),
        Surface::Menu => world.resource::<Standing>().is_open(),
        Surface::Manual => world.resource::<Reading>().is_open(),
        Surface::Install => world.resource::<crate::shell::Installing>().is_open(),
        Surface::Threshold => world.resource::<orbs_shell::Threshold>().is_waiting(),
    }
}

/// What of `scene`'s `ends` falls short in `world`.
fn shortfall(world: &World, scene: &Scene) -> Vec<String> {
    let sim = world.resource::<Tower>().sim();
    let mut short = scene.ends.shortfall(sim);
    if let Some(surface) = scene.ends.surface
        && !up(world, surface)
    {
        short.push(format!("no {surface:?} is up"));
    }
    short.extend(
        sim.parse_log()
            .records()
            .iter()
            .filter(|record| scene.unread(record.outcome))
            .map(|record| format!("{:?} was not read", record.input)),
    );
    short
}

/// What the frames of each scene could not fit, by scene, and how many frames
/// of what size were looked at, so a check that finds nothing has looked.
#[derive(Resource, Default)]
struct Cut {
    clipped: BTreeSet<(usize, String)>,
    frames: BTreeMap<(u16, u16), u32>,
}

fn cut(canvas: Res<Canvas>, tour: Res<Tour>, mut cut: ResMut<Cut>) {
    for clip in canvas.frame.clipped() {
        cut.clipped.insert((tour.at(), clip.text.clone()));
    }
    if !canvas.frame.speech().is_empty() {
        let size = canvas.frame.size();
        *cut.frames.entry((size.cols, size.rows)).or_default() += 1;
    }
}

#[test]
fn every_scene_plays_its_steps_and_ends_where_it_says() {
    played(GRID);
}

#[test]
fn every_scene_plays_as_well_on_the_smallest_grid() {
    // Where a pane is narrowest: what fits at the window's grid may not here.
    played(MIN_GRID);
}

/// The catalogue end to end at `grid`: every `ends` held, no tower kept, and
/// every frame painted showing all that it says.
fn played(grid: GridSize) {
    let all = Catalogue::builtin().scenes().to_vec();
    let total = all.len();
    let mut app = app(all.clone());
    // Painted as the window paints, with nothing to rasterise it.
    app.insert_resource(Screen {
        grid,
        window: (u32::from(PICTURE.0), u32::from(PICTURE.1)),
        ..Screen::default()
    })
    .init_resource::<Canvas>()
    .init_resource::<Cut>()
    .add_systems(
        Update,
        (repaint, cut)
            .chain()
            .after(ShellSystems::Input)
            .after(ShellSystems::Drive),
    );
    let mut ended = 0;
    // Generous: a minute a scene, of which a brisk tour uses seconds.
    let budget = 60 * SECOND * u32::try_from(total).unwrap_or(u32::MAX);
    for _ in 0..budget {
        let showing = tour(&app).at();
        app.update();
        let now = tour(&app);
        let over = now.is_due() || (now.is_last() && now.is_over());
        if !over {
            continue;
        }
        let scene = &all[showing];
        let short = shortfall(app.world(), scene);
        assert!(short.is_empty(), "{}: {}", scene.name, short.join("; "));
        // The installation's wake swaps through the menu's own door.
        assert_eq!(
            app.world().resource::<Kept>().path(),
            None,
            "{} left a tower kept somewhere",
            scene.name
        );
        ended += 1;
        if showing + 1 == total {
            break;
        }
    }
    assert_eq!(ended, total, "the tour did not play every scene");
    assert!(
        app.world().resource::<Messages<AppExit>>().is_empty(),
        "a scene ended the game"
    );
    let looked = app.world().resource::<Cut>();
    assert_eq!(
        looked.frames.keys().collect::<Vec<_>>(),
        [&(grid.cols, grid.rows)],
        "the frames painted were not all {grid:?}"
    );
    let cut: Vec<String> = looked
        .clipped
        .iter()
        .map(|(at, text)| format!("  {}: {text:?}", all[*at].name))
        .collect();
    assert!(
        cut.is_empty(),
        "at {}×{} the screen cut what it says:\n{}",
        grid.cols,
        grid.rows,
        cut.join("\n")
    );
}
