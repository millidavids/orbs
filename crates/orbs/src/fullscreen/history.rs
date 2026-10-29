//! The frame an afterglow pass drew last, kept for its next.

use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};
use bevy::render::renderer::RenderDevice;

/// The frame a [`Fullscreen::HISTORY`](super::Fullscreen::HISTORY) pass drew
/// last, kept for its next.
///
/// One texture, owned, not `TextureCache`'s: that hands back any free texture
/// of the descriptor, so one frame's history could be another's. Created
/// zeroed, which reads as black, so a new one starts with no trail; recreated
/// on any change of size or format, because a copy between unequal textures is
/// a validation error at draw rather than compile time.
#[derive(Resource, Default)]
pub(crate) struct History {
    kept: Option<Kept>,
}

struct Kept {
    texture: Texture,
    view: TextureView,
    size: Extent3d,
    format: TextureFormat,
}

impl History {
    /// The kept frame, made fresh unless one of this size and format exists.
    pub(super) fn ensure(
        &mut self,
        device: &RenderDevice,
        size: Extent3d,
        format: TextureFormat,
    ) -> (Texture, TextureView) {
        let kept = match self.kept.take() {
            Some(kept) if kept.size == size && kept.format == format => kept,
            _ => {
                let texture = device.create_texture(&TextureDescriptor {
                    label: Some("orbs_history"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format,
                    usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let view = texture.create_view(&TextureViewDescriptor::default());
                Kept {
                    texture,
                    view,
                    size,
                    format,
                }
            }
        };
        let handed = (kept.texture.clone(), kept.view.clone());
        self.kept = Some(kept);
        handed
    }

    /// Drop the kept frame, so a pass that resumes starts from black rather
    /// than from whatever was on screen when it stopped.
    pub(super) fn forget(&mut self) {
        self.kept = None;
    }
}
