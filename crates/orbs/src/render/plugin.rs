//! Registration for the cell renderer.

use bevy::camera::{Projection, ScalingMode};
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};
use orbs_render::Frame;

use super::atlas::{self, GlyphAtlas};
use super::grid::{self, GridBuffers};
use super::palette::{self, Phosphor};
use crate::shell::Screen;
use crate::sim::Tower;

/// Marks the single entity the whole grid is drawn as.
#[derive(Component)]
struct CellGrid;

/// The frame being painted, and the vertex buffers it becomes.
///
/// Both are reused every frame. `Frame`'s own documentation says to reset rather
/// than reallocate — the grid reaches 160×45 and a siege redraws it every frame.
#[derive(Resource, Default)]
pub(crate) struct Canvas {
    pub(crate) frame: Frame,
    buffers: GridBuffers,
}

/// The active phosphor theme (§4). A setting, never earned.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct Theme(pub(crate) Phosphor);

impl Default for Theme {
    fn default() -> Self {
        Self(palette::MUTED_VIOLET)
    }
}

/// Builds the glyph atlas and draws the Frame through it.
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Canvas>()
            .init_resource::<Theme>()
            .add_systems(PreStartup, build_atlas)
            .add_systems(Startup, spawn_grid.after(build_atlas))
            .add_systems(
                Update,
                (
                    cycle_theme.run_if(input_just_pressed(KeyCode::F2)),
                    capture.run_if(input_just_pressed(KeyCode::F12)),
                    auto_capture,
                    fit_camera,
                    redraw.run_if(atlas_ready),
                )
                    .chain()
                    .after(crate::shell::track_window),
            );
    }
}

/// Build the atlas before anything can want it.
fn build_atlas(mut commands: Commands, mut images: ResMut<Assets<Image>>, theme: Res<Theme>) {
    let atlas = atlas::build(&mut images);

    info!(
        "glyph atlas {}x{} built: 3 faces x 256 glyphs, {} slots filled from the fallback font; \
         theme {}",
        atlas::WIDTH,
        atlas::HEIGHT,
        atlas.filled_from_fallback.iter().sum::<usize>(),
        theme.0.name,
    );

    commands.insert_resource(atlas);
}

/// One entity, one mesh, one draw call for the entire screen.
fn spawn_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    atlas: Res<GlyphAtlas>,
) {
    let material = materials.add(ColorMaterial {
        texture: Some(atlas.image.clone()),
        // The atlas carries coverage in alpha, so blending is what turns a
        // rectangle into a glyph rather than a solid tile.
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    });

    commands.spawn((
        CellGrid,
        Mesh2d(meshes.add(grid::empty_mesh())),
        MeshMaterial2d(material),
    ));
}

/// Step through the phosphor themes.
///
/// §4: all three are cosmetic and none is unlockable — art direction should not
/// be gated behind the economy. A real settings screen replaces this.
fn cycle_theme(mut theme: ResMut<Theme>) {
    let next = palette::ALL
        .iter()
        .position(|candidate| candidate.name == theme.0.name)
        .map_or(0, |index| (index + 1) % palette::ALL.len());
    theme.0 = palette::ALL[next];
    info!("phosphor: {}", theme.0.name);
}

/// Frames to wait before the automatic capture, so the first drawn frame is not
/// caught mid-setup.
const CAPTURE_AT: u32 = 30;

/// Take one screenshot automatically when `ORBS_CAPTURE` is set.
///
/// §4 says work is done when it has been *looked at*, and a renderer that cannot
/// be checked without a human at the keyboard is one nobody checks. This makes
/// "did it actually draw" answerable from a script.
fn auto_capture(mut commands: Commands, mut frames: Local<u32>) {
    if std::env::var_os("ORBS_CAPTURE").is_none() {
        return;
    }
    *frames += 1;
    if *frames == CAPTURE_AT {
        capture(commands.reborrow());
    }
}

/// Save a screenshot. §4 says work is done when it has been *looked at*.
fn capture(mut commands: Commands) {
    let path = "orbs-screenshot.png";
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
    info!("screenshot -> {path}");
}

/// A frame drawn before the atlas exists would be a screen of holes.
fn atlas_ready(atlas: Option<Res<GlyphAtlas>>) -> bool {
    atlas.is_some()
}

/// Make one world unit one **physical** pixel.
///
/// Bevy's default 2D projection works in logical pixels, which on a 2× display
/// would stretch every glyph across four physical pixels — a blurred bitmap
/// font, which §4 calls out as the thing legibility cannot survive. Fixing the
/// projection to the physical size keeps the integer cell scale honest all the
/// way to the framebuffer.
fn fit_camera(
    window: Option<Single<&Window, With<bevy::window::PrimaryWindow>>>,
    camera: Option<Single<&mut Projection, With<Camera2d>>>,
) {
    let (Some(window), Some(mut projection)) = (window, camera) else {
        return;
    };
    // A window wider than 65535 physical pixels is not a case worth carrying
    // arithmetic for; clamping keeps the conversion exact.
    let width = u16::try_from(window.physical_width())
        .unwrap_or(u16::MAX)
        .max(1);
    let height = u16::try_from(window.physical_height())
        .unwrap_or(u16::MAX)
        .max(1);

    if let Projection::Orthographic(orthographic) = &mut **projection {
        orthographic.scaling_mode = ScalingMode::Fixed {
            width: f32::from(width),
            height: f32::from(height),
        };
    }
}

/// Repaint the Frame and rebuild the mesh.
fn redraw(
    screen: Res<Screen>,
    theme: Res<Theme>,
    tower: Res<Tower>,
    mut canvas: ResMut<Canvas>,
    mut meshes: ResMut<Assets<Mesh>>,
    grid_mesh: Option<Single<&Mesh2d, With<CellGrid>>>,
    mut clear: ResMut<ClearColor>,
) {
    clear.0 = theme.0.background.into();

    let Some(grid_mesh) = grid_mesh else {
        return;
    };
    let Some(mut mesh) = meshes.get_mut(&grid_mesh.0) else {
        return;
    };
    if !screen.is_hostable() {
        // Below the floor there is nothing honest to draw; §4's minimum is a
        // real constraint, not a target to squeeze under.
        *mesh = grid::empty_mesh();
        return;
    }

    let Canvas { frame, buffers } = &mut *canvas;
    frame.reset(screen.grid);
    crate::shell::paint(frame, tower.tick().get(), tower.seed());

    let scale = screen.fidelity.map_or(1, |tier| u16::from(tier.scale()));
    grid::build(frame, &theme.0, scale, buffers, &mut mesh);
}
