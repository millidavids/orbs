//! Registration for the window shell.

use bevy::app::AppExit;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::WindowResized;

use super::preview::log_frame;
use super::screen::{Screen, spawn_camera, track_window};

/// The window, the camera, and the grid the window resolves to.
pub struct ShellPlugin;

impl Plugin for ShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Screen>()
            .add_systems(Startup, (spawn_camera, track_window).chain())
            .add_systems(
                Update,
                (
                    track_window.run_if(on_message::<WindowResized>),
                    quit.run_if(input_just_pressed(KeyCode::Escape)),
                ),
            )
            .add_systems(FixedUpdate, log_frame.after(crate::sim::advance));
    }
}

/// Leave the orb.
fn quit(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}
