//! Registration for the accommodation pass.

use bevy::asset::embedded_asset;
use bevy::core_pipeline::{Core2d, Core2dSystems};
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::render::{ExtractSchedule, Render, RenderApp, RenderStartup, RenderSystems};

use super::pass::{SightUniformBuffer, extract, init_pipeline, prepare, sight_pass};
use super::settings::{Vision, cycle, seeded};

/// Take the hue out for players who need it (DESIGN.md §14).
pub struct SightPlugin;

impl Plugin for SightPlugin {
    fn build(&self, app: &mut App) {
        // Compiled in, not loaded from disk: §13 ships no loose asset directory.
        embedded_asset!(app, "sight.wgsl");

        app.insert_resource(Vision(seeded()))
            // **F8, because §14's rule is that a player can reach it.** The
            // phosphor got `F2` and the tube got `F3` for exactly this reason
            // and with exactly this justification — an interim key until Phase
            // 11's settings screen, because an accommodation reachable only
            // through an environment variable is not reachable by the person it
            // is for. A shipped build has no shell.
            //
            // Guarded on boot like every other key: during the sequence, a
            // keystroke means skip. See `boot::plugin::skip`.
            .add_systems(
                Update,
                cycle
                    .run_if(input_just_pressed(KeyCode::F8))
                    .run_if(crate::boot::booted),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<SightUniformBuffer>()
            .add_systems(ExtractSchedule, extract)
            .add_systems(RenderStartup, init_pipeline)
            .add_systems(Render, prepare.in_set(RenderSystems::Prepare))
            // **`.after(crt::CrtPass)`, and the `in_set` is load-bearing.**
            // `crt/plugin.rs` records what an edge to `tonemapping` alone cost:
            // unordered against `main_pass_2d` and `upscaling`, the pass landed
            // at a different point every frame and the tube flashed. This has
            // the same exposure and one more constraint — it must be *after* the
            // tube, because the grille, the aberration and the flash all put hue
            // back into a pixel that had none.
            .add_systems(
                Core2d,
                sight_pass
                    .in_set(Core2dSystems::PostProcess)
                    .after(crate::crt::CrtPass),
            );
    }
}
