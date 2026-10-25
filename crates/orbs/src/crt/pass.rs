//! The tube's full-screen pass: `fullscreen`'s, with the tube's shader.

use bevy::asset::{AssetServer, load_embedded_asset};
use bevy::prelude::*;
use bevy::shader::Shader;

use super::settings::CrtUniform;

impl crate::fullscreen::Fullscreen for CrtUniform {
    const LABEL: &'static str = "orbs_crt";

    fn shader(assets: &AssetServer) -> Handle<Shader> {
        load_embedded_asset!(assets, "crt.wgsl")
    }
}
