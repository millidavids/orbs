//! The afterglow's full-screen pass: `fullscreen`'s, with a history.

use bevy::asset::{AssetServer, load_embedded_asset};
use bevy::prelude::*;
use bevy::shader::Shader;

use super::settings::AfterglowUniform;

impl crate::fullscreen::Fullscreen for AfterglowUniform {
    const LABEL: &'static str = "orbs_afterglow";
    const HISTORY: bool = true;

    fn shader(assets: &AssetServer) -> Handle<Shader> {
        load_embedded_asset!(assets, "afterglow.wgsl")
    }

    fn wanted(&self) -> bool {
        AfterglowUniform::wanted(*self)
    }
}
