//! Registration for the CRT.

use bevy::asset::embedded_asset;
use bevy::core_pipeline::tonemapping::tonemapping;
use bevy::core_pipeline::{Core2d, Core2dSystems};
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
            // Guarded on boot like every other key: during the sequence, a
            // keystroke means skip. See `boot::plugin::skip`.
            .add_systems(
                Update,
                (
                    cycle
                        .run_if(input_just_pressed(KeyCode::F3))
                        .run_if(crate::boot::booted),
                    // Not guarded — it *is* the boot sequence. Runs before
                    // `cycle` so a skip lands on a settled tube rather than one
                    // frozen mid-strike.
                    super::strike::drive.run_if(resource_exists::<crate::boot::Boot>),
                )
                    .chain(),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<CrtUniformBuffer>()
            .add_systems(ExtractSchedule, extract)
            .add_systems(RenderStartup, init_pipeline)
            .add_systems(Render, prepare.in_set(RenderSystems::Prepare))
            // `in_set` is load-bearing, not decoration. `Core2dSystems` chains
            // Prepass -> MainPass -> EarlyPostProcess -> PostProcess, and
            // `upscaling` runs `.after(PostProcess)`. A system that only says
            // `.after(tonemapping)` has an edge to tonemapping and to nothing
            // else — it is unordered against `main_pass_2d` and against
            // `upscaling`, so it lands at a different point every frame. That is
            // what made the tube flash: some frames it curved the grid, some it
            // ran before the grid was drawn, some after the blit to the
            // swapchain had already happened.
            .add_systems(
                Core2d,
                crt_pass
                    .in_set(Core2dSystems::PostProcess)
                    .after(tonemapping),
            );
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
    let (next, name) = after(**settings);
    **settings = next;
    info!("crt: {name}");
}

/// The tube state `current` steps to, and what to call it.
///
/// Split from the system so the one property that matters can be asserted:
/// **pressing F3 enough times reaches off.** §14 makes that an accessibility
/// requirement rather than a convenience, and a `Single<&mut ...>` system is not
/// something a unit test can drive.
fn after(current: CrtSettings) -> (CrtSettings, &'static str) {
    if current == CrtSettings::DEFAULT {
        (CrtSettings::PEAK_THREAT, "peak threat")
    } else if current == CrtSettings::PEAK_THREAT {
        (CrtSettings::OFF, "off")
    } else {
        (CrtSettings::DEFAULT, "default")
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressing_f3_reaches_off_and_comes_back() {
        // §14 requires the tube be disableable. That is a property of the
        // *cycle*, not of `CrtSettings::OFF` existing — a key that never arrives
        // at off is the same as no key at all.
        let mut state = CrtSettings::DEFAULT;
        let mut seen = Vec::new();
        for _ in 0..3 {
            let (next, name) = after(state);
            state = next;
            seen.push(name);
        }
        assert_eq!(seen, ["peak threat", "off", "default"]);
        assert_eq!(state, CrtSettings::DEFAULT, "the cycle did not close");
    }

    #[test]
    fn an_unrecognised_state_lands_somewhere_the_player_can_see() {
        // The fallback arm. Something world-driven will eventually leave the
        // settings equal to no preset — §4 wires vignette to threat and flash to
        // breach — and the wrong answer would be to leave the key doing nothing.
        let odd = CrtSettings {
            vignette: 0.61,
            ..CrtSettings::DEFAULT
        };
        assert_eq!(after(odd).0, CrtSettings::DEFAULT);
    }
}
