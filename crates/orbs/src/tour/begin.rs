//! A scene put in front of the player: the menu's swap, minus the save.

use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use orbs_shell::Boot;

use super::state::Tour;
use crate::shell::{
    Installing, OpenManualMessage, SettingMessage, Standing, SubmittedMessage, SwapMessage,
    apply_setting, default_settings, reset_for_swap,
};
use crate::sim::{Kept, Tower};

/// Begin the scene that is due.
///
/// `menuing::swap`'s order, without its first step: nothing is written,
/// because a tour keeps nothing.
pub(super) fn begin(world: &mut World) {
    let Some(scene) = world
        .get_resource_mut::<Tour>()
        .map(|mut tour| tour.begun())
    else {
        return;
    };

    // Nothing the last scene queued reaches this one: a swap left in the queue
    // would replace the tower raised below.
    clear::<SwapMessage>(world);
    clear::<SettingMessage>(world);
    clear::<SubmittedMessage>(world);
    clear::<OpenManualMessage>(world);
    if let Some(mut held) = world.get_resource_mut::<ButtonInput<KeyCode>>() {
        held.reset_all();
    }

    reset_for_swap(world);
    world.resource_mut::<Standing>().close();

    let staged = orbs_shell::tour::stage(&scene, orbs_shell::wizard().as_deref());
    world.insert_resource(Tower::raised(staged.sim));
    world.insert_resource(Kept::at(None));
    world.insert_resource(if staged.threshold {
        orbs_shell::Threshold::Waiting
    } else {
        orbs_shell::Threshold::Playing
    });
    if let Some(install) = staged.install {
        world.insert_resource(Installing::up(install));
    }

    // From the defaults every scene, or an `F3` in one carries into the next.
    for row in default_settings() {
        apply_setting(world, &row.key, &row.value);
    }
    for (key, value) in &scene.settings {
        apply_setting(world, key, value);
    }
    // After the settings, since a card's stillness is decided as it is built.
    let boot = if scene.boot {
        let mut tubes = world.query::<&crate::crt::CrtSettings>();
        let moving = crate::shell::moving(
            tubes.iter(world),
            world.get_resource::<crate::shell::Stillness>(),
        );
        Boot::new().held_still(!moving.unwrap_or(true))
    } else {
        Boot::finished()
    };
    world.insert_resource(boot);
    if let Some([pressure, fault]) = scene.strain {
        world.insert_resource(crate::crt::Strain::held(pressure, fault));
    }

    // The scene's first tick comes a second after it appears, whatever staging
    // cost: the overstep goes, and the frame that pays for the staging is not
    // caught up.
    if let Some(mut fixed) = world.get_resource_mut::<Time<Fixed>>() {
        fixed.discard_overstep(Duration::MAX);
    }
    if let Some(mut clock) = world.get_resource_mut::<Time<Virtual>>() {
        clock.pause();
    }
    let mut tour = world.resource_mut::<Tour>();
    tour.paused = true;
    let number = tour.at() + 1;
    if let Some(shots) = tour.shots.as_mut() {
        shots.begin(number, &scene);
    }
    let title = tour.title();
    info!("{title}");
    let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
    for mut window in windows.iter_mut(world) {
        window.title.clone_from(&title);
    }
}

/// Start the virtual clock `begin` paused, once a time update has seen it
/// paused: that frame's delta is what staging cost, and it is swallowed.
pub(super) fn resume(mut tour: ResMut<Tour>, mut clock: ResMut<Time<Virtual>>) {
    if tour.paused {
        clock.unpause();
        tour.paused = false;
    }
}

/// Drop every queued `T`.
fn clear<T: Message>(world: &mut World) {
    if let Some(mut queued) = world.get_resource_mut::<Messages<T>>() {
        queued.clear();
    }
}
