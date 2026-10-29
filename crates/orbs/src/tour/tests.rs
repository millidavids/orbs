//! The tour played in an `App` with no window: the real shell, the real
//! keyboard path, the real tick.

use std::time::Duration;

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::window::WindowEvent;
use orbs_shell::tour::{Catalogue, Scene};

use super::plugin::TourPlugin;
use super::state::Tour;
use crate::shell::ShellPlugin;
use crate::sim::SimPlugin;

mod catalogue;
mod keys;
mod scenes;

/// Frames a second, as the window runs.
const SECOND: u32 = 60;

fn scenes(text: &str) -> Vec<Scene> {
    Catalogue::read(text)
        .unwrap_or_else(|why| panic!("{why}"))
        .scenes()
        .to_vec()
}

fn app(scenes: Vec<Scene>) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::input::InputPlugin,
        bevy::window::WindowPlugin::default(),
        ShellPlugin,
        SimPlugin {
            seed: 1,
            wizard: None,
            persist: false,
            threshold: false,
        },
        crate::boot::BootPlugin,
        TourPlugin::brisk(scenes),
    ))
    .insert_resource(TimeUpdateStrategy::ManualDuration(
        Duration::from_secs(1) / SECOND,
    ));
    app.update();
    app
}

/// Run `frames` frames.
fn run(app: &mut App, frames: u32) {
    for _ in 0..frames {
        app.update();
    }
}

/// Run until the scene `at` has begun.
fn reach(app: &mut App, at: usize) {
    for _ in 0..60 * SECOND {
        if tour(app).at() == at && !tour(app).is_due() {
            return;
        }
        app.update();
    }
    panic!("the tour never reached scene {at}");
}

/// A person's key, as winit sends it: on both streams.
fn person(app: &mut App, key_code: KeyCode, state: ButtonState) {
    let key = KeyboardInput {
        key_code,
        logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
        state,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    };
    app.world_mut()
        .write_message(WindowEvent::KeyboardInput(key.clone()));
    app.world_mut().write_message(key);
}

fn tour(app: &App) -> &Tour {
    app.world().resource::<Tour>()
}

/// Two scenes: one that types, and one that only stands.
const TWO: &str = r#"
[[scene]]
name = "test.status"
about = "status typed"
steps = [{ type = "status" }, { type = "attend forge" }]

[[scene]]
name = "test.second"
about = "the second scene"
setup = "attend archive"
"#;
