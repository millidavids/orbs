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

/// The tube's own pass, so another can be ordered after it.
///
/// A set rather than the system, because `crt_pass` takes two private types and
/// widening it to be nameable elsewhere would widen them too. `sight` is the
/// caller: §14's accommodation has to be the *last* thing to touch a pixel, and
/// three terms in this shader put hue back into one that had none.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CrtPass;

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
                cycle
                    .run_if(input_just_pressed(KeyCode::F3))
                    .run_if(crate::boot::booted),
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
                    .in_set(CrtPass)
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

/// The three states `F3` reaches, by the names `ORBS_CRT` takes.
///
/// One table, so a name and the state it selects cannot drift apart.
const STATES: [(&str, CrtSettings); 3] = [
    ("default", CrtSettings::DEFAULT),
    ("peak", CrtSettings::PEAK_THREAT),
    ("off", CrtSettings::OFF),
];

/// Which tube state to open with, from `ORBS_CRT`.
///
/// **This exists for a See-it line rather than for players.** `ORBS_CAPTURE`
/// presses no keys, so the state that matters most to §14 — the tube off — was
/// reachable only by a person at a keyboard, and *"turning the tube off does not
/// turn the accommodation off"* is exactly the property worth checking without
/// one. Phase 13's settings screen makes this ordinary.
pub(crate) fn seeded() -> CrtSettings {
    chosen(std::env::var("ORBS_CRT").ok().as_deref())
}

/// The rule [`seeded`] applies, without the environment.
///
/// Split out so it can be *tested*: a test that set `ORBS_CRT` would set it for
/// every other test in the binary. `save::chosen` is the precedent and the
/// reason, and `sight::chosen` is the sibling — the two must agree about case
/// and about blanks, or one accommodation switch behaves unlike the other.
fn chosen(value: Option<&str>) -> CrtSettings {
    // An exported-but-empty variable is *unset*, not a typo. `ORBS_CRT= orbs`
    // is the shell idiom for neutralising one, and a loop variable that came out
    // empty is the same thing arriving by accident; warning on either is noise
    // on every launch. `save::chosen` filters blanks for exactly this reason.
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return CrtSettings::default();
    };
    let Some((_, state)) = STATES
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(value))
    else {
        warn!("ORBS_CRT: no such tube state {value:?} — using the default");
        return CrtSettings::default();
    };
    *state
}

/// Copy the settings across to the render world once per frame.
fn extract(
    settings: Extract<Query<&CrtSettings>>,
    time: Extract<Res<Time>>,
    tube: Extract<Res<Tube>>,
    mut commands: Commands,
) {
    let Some(settings) = settings.iter().next() else {
        return;
    };
    commands.insert_resource(ExtractedCrt(CrtUniform::new(
        *settings,
        time.elapsed_secs(),
        **tube,
    )));
}

/// The tube's geometry in the window, published by the renderer for the CRT.
///
/// Two facts, and the shader needs both for the same reason: **the tube is the
/// 4:3 picture, not the window.** Everything periodic divides the cell size so
/// the pattern lands identically inside every glyph (§9); everything *shaped* —
/// the barrel curve, the vignette, the rounded bezel — is measured against
/// [`fill`](Self::fill_x), so the curve belongs to the monitor rather than to
/// whatever rectangle the player dragged.
///
/// This was `CellSize` and carried only the first pair. §19's fixed grid made
/// the window and the picture different rectangles for the first time, and a
/// shader that knew only about the window curved the letterbox bars along with
/// the phosphor.
#[derive(Resource, Debug, Clone, Copy)]
pub struct Tube {
    /// Physical pixels per cell, horizontally and vertically.
    pub width: f32,
    pub height: f32,
    /// The picture's share of the window on each axis, in `(0, 1]`.
    ///
    /// One of the two is always 1.0 — the picture fits inside on both axes and
    /// touches on at least one — and the other is what the bars eat. The picture
    /// is centred, so the shader derives its origin as `(1 - fill) / 2` rather
    /// than being told: `ScalingMode::AutoMin` centres on `viewport_origin`,
    /// which defaults to the middle.
    pub fill_x: f32,
    pub fill_y: f32,
}

impl Default for Tube {
    /// A window exactly the size of the picture: native cells, no bars.
    fn default() -> Self {
        Self {
            width: f32::from(orbs_render::CELL_WIDTH),
            height: f32::from(orbs_render::CELL_HEIGHT),
            fill_x: 1.0,
            fill_y: 1.0,
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

    /// `ORBS_CRT` and `ORBS_SIGHT` must agree about case and about blanks.
    ///
    /// They did not: this took `off` and refused `OFF`, where `Sight::parse`
    /// has always been case-insensitive. Two accommodation switches behaving
    /// differently is the kind of difference nobody reads a doc comment to
    /// discover.
    #[test]
    fn a_tube_state_is_named_the_way_a_sight_is() {
        for (name, state) in STATES {
            assert_eq!(chosen(Some(name)), state);
            assert_eq!(chosen(Some(&name.to_uppercase())), state);
            assert_eq!(chosen(Some(&format!("  {name}  "))), state);
        }
    }

    /// An exported-but-empty variable is unset, and must not warn.
    #[test]
    fn a_blank_reads_as_unset_rather_than_as_a_typo() {
        assert_eq!(chosen(None), CrtSettings::default());
        assert_eq!(chosen(Some("")), CrtSettings::default());
        assert_eq!(chosen(Some("   ")), CrtSettings::default());
    }

    #[test]
    fn a_typo_falls_back_to_the_default_tube() {
        assert_eq!(chosen(Some("of")), CrtSettings::default());
        assert_eq!(chosen(Some("peak threat")), CrtSettings::default());
    }
}
