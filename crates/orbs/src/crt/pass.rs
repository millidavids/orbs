//! The full-screen pass.
//!
//! **This is not a port.** `court_wizard`'s version is a `ViewNode` wired into a
//! render graph, and Bevy 0.19 does not have one: `bevy_render::render_graph` is
//! gone and post-process passes are ordinary systems in the `Core2d` schedule.
//! §4 flagged the render-graph Rust as "Bevy's most volatile surface" and it was
//! right — the shader ported almost unchanged, the surrounding Rust did not
//! survive at all. It is much smaller this way.

use bevy::asset::{AssetServer, load_embedded_asset};
use bevy::core_pipeline::FullscreenShader;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::binding_types::{sampler, texture_2d, uniform_buffer};
use bevy::render::render_resource::{
    BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries, CachedRenderPipelineId,
    ColorTargetState, ColorWrites, DynamicUniformBuffer, FragmentState, Operations, PipelineCache,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderStages, TextureFormat, TextureSampleType,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery};
use bevy::render::view::ViewTarget;
use bevy::shader::Shader;

use super::settings::CrtUniform;

/// Layout, sampler, and the pipelines built for each target format.
///
/// Keyed on format rather than assuming one: Bevy 0.19 deprecated the notion of
/// a default texture format precisely because a view can be HDR or not, and
/// guessing wrong is a validation error at draw time rather than a compile one.
#[derive(Resource)]
pub(super) struct CrtPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    shader: Handle<Shader>,
    fullscreen: FullscreenShader,
    by_format: HashMap<TextureFormat, CachedRenderPipelineId>,
}

impl CrtPipeline {
    /// The pipeline for `format`, queued on first sight.
    fn for_format(
        &mut self,
        format: TextureFormat,
        pipeline_cache: &PipelineCache,
    ) -> CachedRenderPipelineId {
        *self.by_format.entry(format).or_insert_with(|| {
            pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("orbs_crt".into()),
                layout: vec![self.layout.clone()],
                vertex: self.fullscreen.to_vertex_state(),
                fragment: Some(FragmentState {
                    shader: self.shader.clone(),
                    targets: vec![Some(ColorTargetState {
                        format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                    ..default()
                }),
                ..default()
            })
        })
    }
}

/// This frame's uniform, and where in the buffer it landed.
#[derive(Resource, Default)]
pub(super) struct CrtUniformBuffer {
    buffer: DynamicUniformBuffer<CrtUniform>,
    offset: u32,
}

/// The uniform, extracted from the main world each frame.
#[derive(Resource, Debug, Clone, Copy)]
pub(super) struct ExtractedCrt(pub(super) CrtUniform);

pub(super) fn init_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    fullscreen_shader: Res<FullscreenShader>,
    asset_server: Res<AssetServer>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "orbs_crt_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<CrtUniform>(true),
            ),
        ),
    );

    // Linear, unlike the glyph atlas: this samples an already-rasterised image,
    // and the barrel warp lands between texels by definition.
    let sampler = render_device.create_sampler(&SamplerDescriptor {
        label: Some("orbs_crt_sampler"),
        ..default()
    });

    commands.insert_resource(CrtPipeline {
        layout,
        sampler,
        shader: load_embedded_asset!(asset_server.as_ref(), "crt.wgsl"),
        fullscreen: fullscreen_shader.clone(),
        by_format: HashMap::new(),
    });
}

/// Upload this frame's settings.
pub(super) fn prepare(
    extracted: Option<Res<ExtractedCrt>>,
    mut uniform: ResMut<CrtUniformBuffer>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let Some(extracted) = extracted else {
        return;
    };
    uniform.buffer.clear();
    uniform.offset = uniform.buffer.push(&extracted.0);
    uniform.buffer.write_buffer(&render_device, &render_queue);
}

/// Draw the curved screen.
pub(super) fn crt_pass(
    view: ViewQuery<&ViewTarget>,
    pipeline_cache: Res<PipelineCache>,
    mut pipeline: ResMut<CrtPipeline>,
    uniform: Res<CrtUniformBuffer>,
    mut ctx: RenderContext,
) {
    let view_target = view.into_inner();

    let id = pipeline.for_format(view_target.main_texture_format(), &pipeline_cache);
    let Some(render_pipeline) = pipeline_cache.get_render_pipeline(id) else {
        return;
    };
    let Some(binding) = uniform.buffer.binding() else {
        return;
    };

    // Ping-pong: read the frame the cell grid drew, write the curved one.
    let post_process = view_target.post_process_write();

    let bind_group = ctx.render_device().create_bind_group(
        Some("orbs_crt_bind_group"),
        &pipeline_cache.get_bind_group_layout(&pipeline.layout),
        &BindGroupEntries::sequential((post_process.source, &pipeline.sampler, binding)),
    );

    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("orbs_crt"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: post_process.destination,
            depth_slice: None,
            resolve_target: None,
            ops: Operations::default(),
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    pass.set_render_pipeline(render_pipeline);
    pass.set_bind_group(0, &bind_group, &[uniform.offset]);
    pass.draw(0..3, 0..1);
}
