//! Registration for the CRT.

use bevy::asset::embedded_asset;
use bevy::core_pipeline::Core2d;
use bevy::core_pipeline::tonemapping::tonemapping;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::{Extract, ExtractSchedule, Render, RenderApp, RenderStartup, RenderSystems};

use super::pass::{CrtUniformBuffer, ExtractedCrt, crt_pass, init_pipeline, prepare};
use super::settings::{CrtSettings, CrtUniform};

/// The curved phosphor screen (DESIGN.md §4).
pub struct CrtPlugin;

impl Plugin for CrtPlugin {
    fn build(&self, app: &mut App) {
        // Compiled in, not loaded from disk: §13 ships no loose asset directory.
        embedded_asset!(app, "crt.wgsl");

        app.add_plugins(ExtractComponentPlugin::<CrtSettings>::default())
            .add_systems(Update, cycle.run_if(input_just_pressed(KeyCode::F3)));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<CrtUniformBuffer>()
            .add_systems(ExtractSchedule, extract)
            .add_systems(RenderStartup, init_pipeline)
            .add_systems(Render, prepare.in_set(RenderSystems::Prepare))
            .add_systems(Core2d, crt_pass.after(tonemapping));
    }
}

/// Step through tube states: the tuned default, §4's peak-threat state, and off.
///
/// Peak threat has to be *reachable* rather than merely defined: §4 requires the
/// worst-case legibility test to run against "maximum flicker and vignette
/// pulse", and off is §14's accessibility requirement.
fn cycle(settings: Option<Single<&mut CrtSettings>>) {
    let Some(mut settings) = settings else {
        return;
    };
    let (next, name) = if **settings == CrtSettings::DEFAULT {
        (CrtSettings::PEAK_THREAT, "peak threat")
    } else if **settings == CrtSettings::PEAK_THREAT {
        (CrtSettings::OFF, "off")
    } else {
        (CrtSettings::DEFAULT, "default")
    };
    **settings = next;
    info!("crt: {name}");
}

/// Copy the settings across to the render world once per frame.
fn extract(
    settings: Extract<Query<&CrtSettings>>,
    time: Extract<Res<Time>>,
    cell: Extract<Res<CellSize>>,
    mut commands: Commands,
) {
    let Some(settings) = settings.iter().next() else {
        return;
    };
    commands.insert_resource(ExtractedCrt(CrtUniform::new(
        *settings,
        time.elapsed_secs(),
        (cell.width, cell.height),
    )));
}

/// Physical pixels per cell, published by the renderer for the CRT to read.
#[derive(Resource, Debug, Clone, Copy)]
pub struct CellSize {
    pub width: f32,
    pub height: f32,
}

impl Default for CellSize {
    fn default() -> Self {
        Self {
            width: f32::from(orbs_render::CELL_WIDTH),
            height: f32::from(orbs_render::CELL_HEIGHT),
        }
    }
}
