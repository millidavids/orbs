//! Registration for the cell renderer.

use bevy::prelude::*;

use super::atlas::{self, GlyphAtlas};

/// Builds the glyph atlas and (once written) draws the Frame through it.
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, build_atlas);
    }
}

/// Build the atlas before anything can want it.
fn build_atlas(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let atlas = atlas::build(&mut images);

    info!(
        "glyph atlas {}x{} built: 3 faces x 256 glyphs, {} slots filled from the fallback font",
        atlas::WIDTH,
        atlas::HEIGHT,
        atlas.filled_from_fallback.iter().sum::<usize>(),
    );

    commands.insert_resource(atlas);
}

/// A frame drawn before the atlas exists would be a screen of holes.
#[expect(
    dead_code,
    reason = "consumed by the grid draw step; #[expect] fails once it is"
)]
pub(crate) fn atlas_ready(atlas: Option<Res<GlyphAtlas>>) -> bool {
    atlas.is_some()
}
