//! Registration for the cell renderer.

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};
use orbs_render::Frame;

use super::atlas::{self, GlyphAtlas};
use super::blink::{self, Blink};
use super::grid;
use super::palette::{self, Phosphor};
use crate::crt::Tube;
use crate::shell::Screen;
use crate::sim::Tower;

/// Marks the single entity the whole grid is drawn as.
#[derive(Component)]
struct CellGrid;

/// The frame being painted, reused every frame.
///
/// `Frame`'s own documentation says to reset rather than reallocate — the grid
/// is 120×45 and a siege redraws it every frame. The vertex buffers live in
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
        // What the player last chose; `F2` writes it. An unknown name falls
        // through to the default. `named` covers hidden themes too.
        if let Some(chosen) = orbs_shell::settings::get(crate::shell::THEME)
            && let Some(phosphor) = palette::named(&chosen)
        {
            return Self(phosphor);
        }
        // Deliberately `ALL[0]` rather than the constant by name: the list's
        // order is what `F2` cycles, and a default that is not the thing the
        // cycle starts from makes the first keypress do nothing visible.
        Self(palette::ALL[0])
    }
}

/// Builds the glyph atlas and draws the Frame through it.
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Canvas>()
            .init_resource::<Theme>()
            .init_resource::<super::Contrast>()
            .init_resource::<super::weight::Weight>()
            .init_resource::<Tube>()
            .init_resource::<Blink>()
            .add_systems(PreStartup, build_atlas)
            .add_systems(Startup, spawn_grid.after(build_atlas))
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
                    // The background only moves when the theme does.
                    tint_background.run_if(resource_changed::<Theme>),
                    // ...and the strokes only when the weight does. Never on
                    // the frame it was inserted: `build_atlas` drew that one.
                    atlas::restroke
                        .run_if(atlas_ready)
                        .run_if(resource_changed::<super::weight::Weight>)
                        .run_if(not(resource_added::<super::weight::Weight>)),
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
                    .after(crate::shell::ShellSystems::Input)
                    // ...and after the animations, or a burst of output can draw
                    // complete and then rewind on the following frame. See
                    // `ShellSystems::Drive`.
                    .after(crate::shell::ShellSystems::Drive),
            );

        // Read once at startup rather than polling the environment sixty times a
        // second forever.
        if std::env::var_os("ORBS_CAPTURE").is_some() {
            app.init_resource::<AutoCapture>();
        }
    }
}

/// Build the atlas before anything can want it.
fn build_atlas(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    theme: Res<Theme>,
    weight: Res<super::weight::Weight>,
) {
    let atlas = atlas::build(&mut images, *weight);

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
    // One setting seen twice — see `crt::tube::cycle`.
    orbs_shell::settings::set(crate::shell::THEME, theme.0.name);
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
const fn atlas_ready(atlas: Option<Res<GlyphAtlas>>) -> bool {
    atlas.is_some()
}

/// Keep the room behind the orb matching the active phosphor.
fn tint_background(theme: Res<Theme>, mut clear: ResMut<ClearColor>) {
    clear.0 = theme.0.background.into();
}

/// Everything the shell keeps between frames, as one dependency.
///
/// One `SystemParam`, since these are one dependency: what `shell::View` is
/// built from.
#[derive(bevy::ecs::system::SystemParam)]
struct ShellState<'w> {
    line: Res<'w, crate::shell::Line>,
    offered: Res<'w, crate::shell::Offered>,
    ghost: Res<'w, crate::shell::Ghost>,
    panel: Res<'w, crate::shell::Panel>,
    scroll: Res<'w, crate::shell::Scroll>,
    panes: Res<'w, crate::shell::PaneTransition>,
    /// `ResMut` because this is the one thing here that has to see the frame
    /// *before* it is blanked: a crossing departs from last frame's screen, and
    /// `Frame::reset` below is the moment it stops existing.
    passing: ResMut<'w, crate::shell::Passing>,
    reveal: Res<'w, crate::shell::Reveal>,
    bench: Res<'w, crate::shell::Bench>,
    climb: Res<'w, crate::shell::Climb>,
    /// `ResMut` because the editor's viewport follows its caret, and how many
    /// lines fit is a fact only the painter has — see `Editor::scroll_to`.
    editing: ResMut<'w, crate::shell::Editing>,
    /// `ResMut` only to reach `get_mut`; the painter tells this surface nothing.
    weaving: ResMut<'w, crate::shell::Loom>,
    /// The orb's menu. `Res`, because unlike the two above it there is nothing
    /// here for the painter to hand back.
    menuing: Res<'w, crate::shell::Standing>,
    /// The manual. `ResMut` for the editor's reason: a chapter is wrapped to the
    /// pane and scrolled within it, and only the painter knows how wide and how
    /// tall the pane is.
    reading_manual: ResMut<'w, crate::shell::Reading>,
    /// Whether the arrows are walking the archive's stacks.
    ///
    /// `Res`, not `ResMut`: this one owns no pane, so there is nothing for the
    /// painter to hand back to it.
    walk: Res<'w, crate::shell::Walk>,
    /// The installation, while the tower is its practice tower.
    installing: Res<'w, crate::shell::Installing>,
}

/// Paint the screen into the `Frame`.
///
/// Split from [`rasterise`] at the Frame boundary: this decides what the screen
/// says, that turns cells into triangles.
fn repaint(
    screen: Res<Screen>,
    tower: Res<Tower>,
    mut shell: ShellState,
    boot: Option<Res<crate::boot::Boot>>,
    mut linear: ResMut<crate::shell::Linear>,
    mut canvas: ResMut<Canvas>,
) {
    let ShellState {
        line,
        offered,
        ghost,
        panel,
        scroll,
        panes,
        mut passing,
        reveal,
        bench,
        climb,
        ref mut editing,
        ref mut weaving,
        walk,
        menuing,
        ref mut reading_manual,
        installing,
    } = shell;
    let frame = &mut canvas.frame;
    // Before the reset, the last moment last frame exists. A crossing departs
    // from the screen it is replacing, and nothing can ask for that screen once
    // the line below has blanked it. Refused while a crossing is already
    // running, or it would overwrite its own source — see `Passing::keep`.
    passing.keep(frame);
    frame.reset(screen.grid);

    let booting = boot.filter(|boot| !boot.is_live());
    if let Some(boot) = booting {
        // The orb waking up. It paints the parts of the screen that exist yet
        // and nothing else, so `Dark` really is dark — see `boot::stage`.
        crate::shell::paint_booting(frame, boot.stage(), boot.progress(), crate::boot::engine());
    } else if screen.is_hostable() {
        crate::shell::paint(
            frame,
            &mut linear,
            &mut passing,
            crate::shell::View {
                sim: tower.sim(),
                line: &line,
                screen: &screen,
                panes: &panes,
                reveal: &reveal,
                offered: &offered,
                ghost: &ghost.0,
                panel: &panel,
                scroll: &scroll,
                bench: &bench,
                climb: &climb,
                editing: editing.get_mut(),
                weaving: weaving.get_mut().map(|screen| &*screen),
                walking: walk.is_open(),
                menuing: menuing.get(),
                reading_manual: reading_manual.get_mut(),
                installing: installing.get(),
            },
        );
    } else {
        // `Screen::is_hostable` documents this as a real state to render, not a
        // reason to stop drawing. Blanking the mesh left the player looking at an
        // empty rectangle with no idea why.
        crate::shell::paint_too_small(frame, tower.sim());
    }
}

/// Turn the `Frame`'s cells into the one mesh that draws them.
fn rasterise(
    screen: Res<Screen>,
    theme: Res<Theme>,
    contrast: Option<Res<super::Contrast>>,
    blink: Res<Blink>,
    canvas: Res<Canvas>,
    mut tube: ResMut<Tube>,
    mut meshes: ResMut<Assets<Mesh>>,
    grid_mesh: Option<Single<&Mesh2d, With<CellGrid>>>,
    mut drew_something: Local<bool>,
    mut lifted: Local<super::legibility::Lifted>,
) {
    let Some(grid_mesh) = grid_mesh else {
        return;
    };

    // An empty screen must not rebuild the mesh: `get_mut` marks it changed,
    // and Bevy's allocator answers a zero-vertex upload with `use-after-free`.
    // The `Local` still writes the transition into blank. A caret counts only
    // on the showing half of its blink.
    let caret = blink.showing() && canvas.frame.cursor().is_some();
    let nothing = canvas.frame.is_blank() && !caret;
    if nothing && !*drew_something {
        return;
    }
    *drew_something = !nothing;

    let Some(mut mesh) = meshes.get_mut(&grid_mesh.0) else {
        return;
    };

    // The only place a cell's physical size is known, so the CRT's scanlines
    // land on real pixels (§9). The fill tells the shader where the tube is,
    // so the bezel is the monitor's, not the window's (§19).
    let scale = screen.scale();
    let window = (
        orbs_render::pixels(screen.window.0),
        orbs_render::pixels(screen.window.1),
    );
    let picture = (
        f32::from(orbs_render::PICTURE.0) * scale,
        f32::from(orbs_render::PICTURE.1) * scale,
    );
    // Where the camera put the picture's middle, which `fit_projection` reads
    // from the same `Screen::middle`.
    let middle = screen.middle();
    let shift = |at: f32, side: f32| if side >= 1.0 { at / side - 0.5 } else { 0.0 };
    *tube = Tube {
        shift_x: shift(middle.0, window.0),
        shift_y: shift(middle.1, window.1),
        width: f32::from(orbs_render::CELL_WIDTH) * scale,
        height: f32::from(orbs_render::CELL_HEIGHT) * scale,
        // `max(1.0, …)` on the divisor, not a clamp on the result: before the
        // first `WindowResized` the window is `(0, 0)` and this would be a
        // division by zero published straight into a uniform.
        fill_x: (picture.0 / window.0.max(1.0)).clamp(0.0, 1.0),
        fill_y: (picture.1 / window.1.max(1.0)).clamp(0.0, 1.0),
    };

    let drawn = contrast.map_or(theme.0, |contrast| lifted.drawn(theme.0, *contrast));
    grid::build(&canvas.frame, &drawn, blink.showing(), &mut mesh);
}
