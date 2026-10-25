//! The full-screen accommodation pass: `fullscreen`'s, with sight's shader.
//!
//! A second pass rather than more of the tube for the reasons in `sight.wgsl`'s
//! header: the tube early-returns when it is off, and this may not.

use bevy::asset::{AssetServer, load_embedded_asset};
use bevy::prelude::*;
use bevy::render::Extract;
use bevy::shader::Shader;

use super::settings::{SightUniform, Vision};
use crate::fullscreen::Extracted;

/// Hand the setting to the render world.
///
/// Reads [`Vision`] and the window's absence, and nothing else. There is no path
/// from here to `CrtSettings`, which makes "the tube cannot switch the
/// accommodation off" a property of the wiring rather than a promise in a
/// comment.
pub(super) fn extract(
    vision: Extract<Res<Vision>>,
    absence: Extract<Option<Res<crate::shell::Absence>>>,
    mut commands: Commands,
) {
    let brightness = absence
        .as_deref()
        .map_or(1.0, |absence| absence.brightness());
    commands.insert_resource(Extracted(SightUniform::new(vision.0, brightness)));
}

impl crate::fullscreen::Fullscreen for SightUniform {
    const LABEL: &'static str = "orbs_sight";

    fn shader(assets: &AssetServer) -> Handle<Shader> {
        load_embedded_asset!(assets, "sight.wgsl")
    }

    /// Skipped when it would change nothing: a pass that always ran would cost
    /// a fullscreen ping-pong every frame for every player, to multiply by an
    /// identity. The one place the accommodation may be conditional, because
    /// the condition is its own setting and nothing else's.
    fn wanted(&self) -> bool {
        SightUniform::wanted(*self)
    }
}
