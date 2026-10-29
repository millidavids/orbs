//! The pass itself: its pipeline per format, its uniform, and the draw.

use std::marker::PhantomData;

use bevy::asset::AssetServer;
use bevy::core_pipeline::FullscreenShader;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::binding_types::{sampler, texture_2d, uniform_buffer};
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::{
    BindGroupLayoutDescriptor, CachedRenderPipelineId, ColorTargetState, ColorWrites,
    DynamicBindGroupEntries, DynamicBindGroupLayoutEntries, DynamicUniformBuffer, FragmentState,
    Operations, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
    ShaderType, TextureFormat, TextureSampleType,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery};
use bevy::render::view::ViewTarget;
use bevy::shader::Shader;

use super::History;

/// The uniform a full-screen pass draws with, and what is particular to it.
pub(crate) trait Fullscreen: ShaderType + WriteInto + Send + Sync + 'static {
    /// What its pipeline, layout, sampler, bind group and pass are called, for
    /// a GPU trace.
    const LABEL: &'static str;

    /// Its fragment shader. Loaded where its module embedded it: an embedded
    /// asset's path is the module's, so `load_embedded_asset!` cannot be here.
    fn shader(assets: &AssetServer) -> Handle<Shader>;

    /// Whether this frame's value changes anything. A pass that would multiply
    /// by an identity is skipped, upload and all; the tube always draws, since
    /// its shader returns early when it is off.
    fn wanted(&self) -> bool {
        true
    }

    /// Whether the pass also reads the frame it drew last time, at binding 3 —
    /// the afterglow's. See [`History`].
    const HISTORY: bool = false;
}

/// Layout, sampler, and the pipelines built for each target format.
///
/// Keyed on format rather than assuming one: Bevy 0.19 deprecated the notion of
/// a default texture format precisely because a view can be HDR or not, and
/// guessing wrong is a validation error at draw time rather than a compile one.
#[derive(Resource)]
pub(crate) struct Pipeline<U: Fullscreen> {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    shader: Handle<Shader>,
    fullscreen: FullscreenShader,
    by_format: HashMap<TextureFormat, CachedRenderPipelineId>,
    uniform: PhantomData<fn() -> U>,
}

impl<U: Fullscreen> Pipeline<U> {
    /// The pipeline for `format`, queued on first sight.
    fn for_format(
        &mut self,
        format: TextureFormat,
        pipeline_cache: &PipelineCache,
    ) -> CachedRenderPipelineId {
        *self.by_format.entry(format).or_insert_with(|| {
            pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
                label: Some(U::LABEL.into()),
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
#[derive(Resource)]
pub(crate) struct UniformBuffer<U: Fullscreen> {
    buffer: DynamicUniformBuffer<U>,
    offset: u32,
}

impl<U: Fullscreen> Default for UniformBuffer<U> {
    fn default() -> Self {
        Self {
            buffer: DynamicUniformBuffer::default(),
            offset: 0,
        }
    }
}

/// The uniform, extracted from the main world each frame.
#[derive(Resource)]
pub(crate) struct Extracted<U: Fullscreen>(pub(crate) U);

/// Build the layout and sampler once the render device exists.
pub(crate) fn init_pipeline<U: Fullscreen>(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    fullscreen_shader: Res<FullscreenShader>,
    asset_server: Res<AssetServer>,
) {
    // The frame, its sampler and the uniform; the kept frame after them.
    let mut entries = DynamicBindGroupLayoutEntries::sequential(
        ShaderStages::FRAGMENT,
        (
            texture_2d(TextureSampleType::Float { filterable: true }),
            sampler(SamplerBindingType::Filtering),
            uniform_buffer::<U>(true),
        ),
    );
    if U::HISTORY {
        entries =
            entries.extend_sequential((texture_2d(TextureSampleType::Float { filterable: true }),));
    }
    let layout = BindGroupLayoutDescriptor::new(format!("{}_layout", U::LABEL), &entries);

    // The default sampler: this samples an already-rasterised image, and the
    // tube's barrel warp lands between texels by definition.
    let sampler = render_device.create_sampler(&SamplerDescriptor {
        label: Some(&format!("{}_sampler", U::LABEL)),
        ..default()
    });

    commands.insert_resource(Pipeline::<U> {
        layout,
        sampler,
        shader: U::shader(asset_server.as_ref()),
        fullscreen: fullscreen_shader.clone(),
        by_format: HashMap::new(),
        uniform: PhantomData,
    });
}

/// Upload this frame's uniform, unless it would change nothing.
pub(crate) fn prepare<U: Fullscreen>(
    extracted: Option<Res<Extracted<U>>>,
    mut uniform: ResMut<UniformBuffer<U>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let Some(extracted) = extracted else {
        return;
    };
    if !extracted.0.wanted() {
        return;
    }
    uniform.buffer.clear();
    uniform.offset = uniform.buffer.push(&extracted.0);
    uniform.buffer.write_buffer(&render_device, &render_queue);
}

/// Draw the frame so far through the shader.
pub(crate) fn pass<U: Fullscreen>(
    view: ViewQuery<&ViewTarget>,
    extracted: Option<Res<Extracted<U>>>,
    pipeline_cache: Res<PipelineCache>,
    mut pipeline: ResMut<Pipeline<U>>,
    uniform: Res<UniformBuffer<U>>,
    mut history: Option<ResMut<History>>,
    mut ctx: RenderContext,
) {
    if extracted.is_some_and(|extracted| !extracted.0.wanted()) {
        // Still cuts and keeps nothing: a trail resumed from a frame left
        // behind when motion stopped would be a picture of the past.
        if U::HISTORY
            && let Some(history) = history.as_deref_mut()
        {
            history.forget();
        }
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
    // Every early return before the ping-pong below: after it, a pass that
    // drew nothing leaves the next one reading an empty texture.
    let kept = if U::HISTORY {
        let Some(history) = history.as_deref_mut() else {
            return;
        };
        Some(history.ensure(
            ctx.render_device(),
            view_target.main_texture().size(),
            view_target.main_texture_format(),
        ))
    } else {
        None
    };

    // Ping-pong: read the frame drawn so far, write this pass's.
    let post_process = view_target.post_process_write();

    let label = format!("{}_bind_group", U::LABEL);
    let layout = pipeline_cache.get_bind_group_layout(&pipeline.layout);
    let mut entries =
        DynamicBindGroupEntries::sequential((post_process.source, &pipeline.sampler, binding));
    if let Some((_, previous)) = &kept {
        entries = entries.extend_sequential((previous,));
    }
    let bind_group = ctx
        .render_device()
        .create_bind_group(Some(label.as_str()), &layout, &entries);

    {
        let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some(U::LABEL),
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

    // What this pass drew becomes the next frame's history. Same encoder, so
    // after the draw; same size and format, which `ensure` made sure of.
    if let Some((texture, _)) = &kept {
        ctx.command_encoder().copy_texture_to_texture(
            post_process.destination_texture.as_image_copy(),
            texture.as_image_copy(),
            post_process.destination_texture.size(),
        );
    }
}
