//! Registration for the afterglow.

use bevy::asset::embedded_asset;
use bevy::core_pipeline::tonemapping::tonemapping;
use bevy::core_pipeline::{Core2d, Core2dSystems};
use bevy::prelude::*;
use bevy::render::{ExtractSchedule, Render, RenderApp, RenderStartup, RenderSystems};

use super::settings::{AfterglowUniform, Permitted, extract};
use crate::fullscreen::{History, UniformBuffer, init_pipeline, pass, prepare};

/// A lit phosphor lingers for a fraction of a second (§19, *the tower speaks
/// up*). Enrichment: it adds nothing that was not on screen a moment ago.
pub struct AfterglowPlugin;

impl Plugin for AfterglowPlugin {
    fn build(&self, app: &mut App) {
        // Compiled in, not loaded from disk: §13 ships no loose asset directory.
        embedded_asset!(app, "afterglow.wgsl");
        // Read once at startup rather than polling the environment every frame.
        app.init_resource::<Permitted>();

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<UniformBuffer<AfterglowUniform>>()
            .init_resource::<History>()
            .add_systems(ExtractSchedule, extract)
            .add_systems(RenderStartup, init_pipeline::<AfterglowUniform>)
            .add_systems(
                Render,
                prepare::<AfterglowUniform>.in_set(RenderSystems::Prepare),
            )
            // Before the tube: the trail is the phosphor's, under the glass, so
            // it curves and is scan-lined with the glyphs. `in_set` for the
            // tube's reason (`crt/plugin.rs`).
            .add_systems(
                Core2d,
                pass::<AfterglowUniform>
                    .in_set(Core2dSystems::PostProcess)
                    .after(tonemapping)
                    .before(crate::crt::CrtPass),
            );
    }
}
