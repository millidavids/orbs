//! The full-screen accommodation pass.
//!
//! Mirrors `crt::pass` closely and deliberately — same pipeline-per-format
//! shape, same ping-pong, same "absent only while the pipeline compiles" guards.
//! It is a second pass rather than more of the tube for the reasons in
//! `sight.wgsl`'s header: the tube early-returns when it is off, and this may
//! not.

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

use bevy::render::Extract;

use super::settings::{SightUniform, Vision};

/// Hand the setting to the render world.
///
/// **Reads [`Vision`] and nothing else.** There is no path from here to
/// `CrtSettings`, which is what makes "the tube cannot switch the accommodation
/// off" a property of the wiring rather than a promise in a comment.
pub(super) fn extract(vision: Extract<Res<Vision>>, mut commands: Commands) {
    commands.insert_resource(ExtractedSight(SightUniform::new(vision.0)));
}

/// Layout, sampler, and the pipelines built for each target format.
#[derive(Resource)]
pub(super) struct SightPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    shader: Handle<Shader>,
    fullscreen: FullscreenShader,
    by_format: HashMap<TextureFormat, CachedRenderPipelineId>,
}

impl SightPipeline {
    /// The pipeline for `format`, queued on first sight.
    fn for_format(
        &mut self,
        format: TextureFormat,
        pipeline_cache: &PipelineCache,
    ) -> CachedRenderPipelineId {
        *self.by_format.entry(format).or_insert_with(|| {
            pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("orbs_sight".into()),
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
pub(super) struct SightUniformBuffer {
    buffer: DynamicUniformBuffer<SightUniform>,
    offset: u32,
}

/// The uniform, extracted from the main world each frame.
#[derive(Resource, Debug, Clone, Copy)]
pub(super) struct ExtractedSight(pub(super) SightUniform);

pub(super) fn init_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    fullscreen_shader: Res<FullscreenShader>,
    asset_server: Res<AssetServer>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "orbs_sight_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<SightUniform>(true),
            ),
        ),
    );

    // Nearest would do — this samples one texel per fragment and warps nothing —
    // but the default matches the tube's and costs the same.
    let sampler = render_device.create_sampler(&SamplerDescriptor {
        label: Some("orbs_sight_sampler"),
        ..default()
    });

    commands.insert_resource(SightPipeline {
        layout,
        sampler,
        shader: load_embedded_asset!(asset_server.as_ref(), "sight.wgsl"),
        fullscreen: fullscreen_shader.clone(),
        by_format: HashMap::new(),
    });
}

/// Upload this frame's setting.
pub(super) fn prepare(
    extracted: Option<Res<ExtractedSight>>,
    mut uniform: ResMut<SightUniformBuffer>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let Some(extracted) = extracted else {
        return;
    };
    // **Skipped for the same reason the pass is.** `sight_pass` early-returns
    // when the setting changes nothing, so uploading in that case buys a buffer
    // write per frame for a value nothing reads — sixty a second, for every
    // player who has not asked for an accommodation. The pass's own guard is
    // what keeps the picture right; this is what keeps the ordinary case free.
    if !extracted.0.wanted() {
        return;
    }
    uniform.buffer.clear();
    uniform.offset = uniform.buffer.push(&extracted.0);
    uniform.buffer.write_buffer(&render_device, &render_queue);
}

/// Take the hue out.
pub(super) fn sight_pass(
    view: ViewQuery<&ViewTarget>,
    extracted: Option<Res<ExtractedSight>>,
    pipeline_cache: Res<PipelineCache>,
    mut pipeline: ResMut<SightPipeline>,
    uniform: Res<SightUniformBuffer>,
    mut ctx: RenderContext,
) {
    // **Skipped when it would change nothing.** A pass that always ran would
    // cost a fullscreen ping-pong on every frame for every player, to multiply
    // by an identity. This is the one place the accommodation is allowed to be
    // conditional, because the condition is its own setting and nothing else's.
    let Some(extracted) = extracted else {
        return;
    };
    if !extracted.0.wanted() {
        return;
    }

    let view_target = view.into_inner();

    let id = pipeline.for_format(view_target.main_texture_format(), &pipeline_cache);
    // Both are absent only on the first frames, while the pipeline compiles.
    let Some(render_pipeline) = pipeline_cache.get_render_pipeline(id) else {
        return;
    };
    let Some(binding) = uniform.buffer.binding() else {
        return;
    };

    // Ping-pong: read the frame the tube drew, write the one without hue.
    let post_process = view_target.post_process_write();

    let bind_group = ctx.render_device().create_bind_group(
        Some("orbs_sight_bind_group"),
        &pipeline_cache.get_bind_group_layout(&pipeline.layout),
        &BindGroupEntries::sequential((post_process.source, &pipeline.sampler, binding)),
    );

    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("orbs_sight"),
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
