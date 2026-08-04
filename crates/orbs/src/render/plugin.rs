//! Registration for the cell renderer.

use bevy::camera::{Projection, ScalingMode};
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};
use bevy::window::WindowResized;
use orbs_render::Frame;

use super::atlas::{self, GlyphAtlas};
use super::blink::{self, Blink};
use super::grid;
use super::palette::{self, Phosphor};
use crate::crt::CellSize;
use crate::shell::Screen;
use crate::sim::Tower;

/// Marks the single entity the whole grid is drawn as.
#[derive(Component)]
struct CellGrid;

/// The frame being painted, reused every frame.
///
/// `Frame`'s own documentation says to reset rather than reallocate — the grid
/// reaches 160×45 and a siege redraws it every frame. The vertex buffers live in
/// the mesh itself; see [`super::grid`].
#[derive(Resource, Default)]
pub(crate) struct Canvas {
    pub(crate) frame: Frame,
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
            .init_resource::<CellSize>()
            .init_resource::<Blink>()
            .add_systems(PreStartup, build_atlas)
            .add_systems(Startup, spawn_grid.after(build_atlas))
            .add_systems(Startup, fit_camera.after(spawn_grid))
            .add_systems(
                Update,
                (
                    // The two keys are guarded on boot; the automatic capture is
                    // not, so a scripted screenshot can still catch the sequence.
                    cycle_theme
                        .run_if(input_just_pressed(KeyCode::F2))
                        .run_if(crate::boot::booted),
                    capture
                        .run_if(input_just_pressed(KeyCode::F12))
                        .run_if(crate::boot::booted),
                    auto_capture.run_if(resource_exists::<AutoCapture>),
                    // The projection only needs revisiting when the window
                    // changes; every Update system carries a guard (CLAUDE.md).
                    fit_camera.run_if(on_message::<WindowResized>),
                    // The background only moves when the theme does.
                    tint_background.run_if(resource_changed::<Theme>),
                    blink::tick,
                    // Typing must not hide what is being typed: a caret caught
                    // mid-blink when a key lands reads as dropped input. Guarded
                    // like every other keyed system — during boot a keystroke is
                    // a skip, and waking a caret that is not on screen yet is at
                    // best pointless.
                    blink::wake
                        .run_if(on_message::<bevy::input::keyboard::KeyboardInput>)
                        .run_if(crate::boot::booted),
                    // Chained below, so the mesh is always built from the frame
                    // this frame painted rather than the previous one's.
                    repaint.run_if(atlas_ready),
                    rasterise.run_if(atlas_ready),
                )
                    .chain()
                    .after(crate::shell::track_window)
                    // The frame that receives a keystroke is the frame that
                    // draws it. Without this edge the paint is unordered against
                    // the input systems, which is a one-frame lag nobody sees
                    // and a screenshot nobody can reproduce.
                    .after(crate::shell::ShellSystems::Input),
            );

        // Read once at startup rather than polling the environment sixty times a
        // second forever.
        if std::env::var_os("ORBS_CAPTURE").is_some() {
            app.init_resource::<AutoCapture>();
        }
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

/// Present only when `ORBS_CAPTURE` was set at startup.
#[derive(Resource, Default)]
struct AutoCapture;

/// Take one screenshot automatically.
///
/// §4 says work is done when it has been *looked at*, and a renderer that cannot
/// be checked without a human at the keyboard is one nobody checks. This makes
/// "did it actually draw" answerable from a script.
fn auto_capture(mut commands: Commands, mut frames: Local<u32>) {
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

/// Keep the room behind the orb matching the active phosphor.
fn tint_background(theme: Res<Theme>, mut clear: ResMut<ClearColor>) {
    clear.0 = theme.0.background.into();
}

/// Paint the screen into the `Frame`.
///
/// Split from [`rasterise`] rather than being one `redraw`, because they are two
/// jobs meeting at the Frame boundary and sharing only the [`Canvas`]: this one
/// decides *what the screen says* and reads the world to do it; that one turns
/// cells into triangles and reads nothing but the frame, the theme and the
/// caret. `clippy.toml`'s argument threshold is what forced the question, and
/// the answer it wanted — a system whose parameters had stopped being one
/// dependency list.
fn repaint(
    screen: Res<Screen>,
    tower: Res<Tower>,
    line: Res<crate::shell::Line>,
    panes: Res<crate::shell::PaneTransition>,
    reveal: Res<crate::shell::Reveal>,
    boot: Option<Res<crate::boot::Boot>>,
    mut linear: ResMut<crate::shell::Linear>,
    mut canvas: ResMut<Canvas>,
) {
    let frame = &mut canvas.frame;
    frame.reset(screen.grid);

    let booting = boot.filter(|boot| !boot.is_live());
    if let Some(boot) = booting {
        // The orb waking up. It paints the parts of the screen that exist yet
        // and nothing else, so `Dark` really is dark — see `boot::stage`.
        crate::shell::paint_booting(frame, tower.sim(), &screen, boot.stage(), boot.progress());
    } else if screen.is_hostable() {
        crate::shell::paint(
            frame,
            tower.sim(),
            &line,
            &screen,
            &mut linear,
            &panes,
            &reveal,
        );
    } else {
        // `Screen::is_hostable` documents this as a real state to render, not a
        // reason to stop drawing. Blanking the mesh left the player looking at an
        // empty rectangle with no idea why.
        crate::shell::paint_too_small(frame);
    }
}

/// Turn the `Frame`'s cells into the one mesh that draws them.
fn rasterise(
    screen: Res<Screen>,
    theme: Res<Theme>,
    blink: Res<Blink>,
    canvas: Res<Canvas>,
    mut cell: ResMut<CellSize>,
    mut meshes: ResMut<Assets<Mesh>>,
    grid_mesh: Option<Single<&Mesh2d, With<CellGrid>>>,
    mut drew_something: Local<bool>,
) {
    let Some(grid_mesh) = grid_mesh else {
        return;
    };

    // **An empty screen must not rebuild the mesh.** `Assets::get_mut` marks the
    // asset changed whether or not anything is written, so a blank frame
    // re-uploaded a zero-vertex mesh every frame — and Bevy 0.19's slab
    // allocator answers that with a stream of `use-after-free: attempted to copy
    // element data for an unallocated key`. The boot sequence is the first thing
    // in this game to hold a blank screen for more than one frame, and it
    // produced ~250 of them in 1.3 seconds.
    //
    // The `Local` is what keeps this correct rather than merely quiet: the
    // transition *into* blank still writes once, so a screen that empties really
    // does clear instead of leaving the last mesh on the tube.
    let nothing = canvas.frame.is_blank() && canvas.frame.cursor().is_none();
    if nothing && !*drew_something {
        return;
    }
    *drew_something = !nothing;

    let Some(mut mesh) = meshes.get_mut(&grid_mesh.0) else {
        return;
    };

    let scale = screen.fidelity.map_or(1, |tier| u16::from(tier.scale()));

    // §9: the CRT's scanline and grille frequencies re-derive against the active
    // cell size. Publishing it here is what keeps them in step through a
    // fidelity-tier change.
    *cell = CellSize {
        width: f32::from(orbs_render::CELL_WIDTH) * f32::from(scale),
        height: f32::from(orbs_render::CELL_HEIGHT) * f32::from(scale),
    };

    grid::build(&canvas.frame, &theme.0, scale, blink.showing(), &mut mesh);
}
