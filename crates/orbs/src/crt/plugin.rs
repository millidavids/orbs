//! Registration for the CRT.

use bevy::asset::embedded_asset;
use bevy::core_pipeline::tonemapping::tonemapping;
use bevy::core_pipeline::{Core2d, Core2dSystems};
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::{ExtractSchedule, Render, RenderApp, RenderStartup, RenderSystems};

use super::settings::{CrtSettings, CrtUniform};
use super::tube::{CrtPass, cycle, extract};
use crate::fullscreen::{UniformBuffer, init_pipeline, pass, prepare};

/// The curved phosphor screen (DESIGN.md §4).
pub struct CrtPlugin;

impl Plugin for CrtPlugin {
    fn build(&self, app: &mut App) {
        // Compiled in, not loaded from disk: §13 ships no loose asset directory.
        embedded_asset!(app, "crt.wgsl");

        app.add_plugins(ExtractComponentPlugin::<CrtSettings>::default())
            .init_resource::<super::Picture>()
            // Guarded on boot like every other key: during the sequence, a
            // keystroke means skip. See `boot::plugin::skip`.
            .add_systems(
                Update,
                cycle
                    .run_if(input_just_pressed(KeyCode::F3))
                    .run_if(crate::boot::booted),
            )
            // Every frame, for the ramp; it writes only on a change. After the
            // panel is refreshed, which is the one thing it reads.
            .add_systems(
                Update,
                super::strain::notice
                    .after(crate::shell::ShellSystems::Drive)
                    .run_if(resource_exists::<orbs_shell::Panel>)
                    .run_if(resource_exists::<super::Strain>),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<UniformBuffer<CrtUniform>>()
            .add_systems(ExtractSchedule, extract)
            .add_systems(RenderStartup, init_pipeline::<CrtUniform>)
            .add_systems(Render, prepare::<CrtUniform>.in_set(RenderSystems::Prepare))
            // `in_set` is load-bearing: `.after(tonemapping)` alone leaves the
            // pass unordered against `main_pass_2d` and `upscaling`, and the
            // tube flashed. The 0.20 chain holds since every pass takes
            // `RenderContext`, which always conflicts.
            .add_systems(
                Core2d,
                pass::<CrtUniform>
                    .in_set(Core2dSystems::PostProcess)
                    .in_set(CrtPass)
                    .after(tonemapping),
            );
    }
}
