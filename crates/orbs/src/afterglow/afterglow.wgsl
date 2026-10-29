// Afterglow — a lit phosphor lingers for a fraction of a second (§19, *the
// tower speaks up*).
//
// Before the tube, so a trail curves with the glass and gets its scan lines;
// after it, the dark room around the picture would glow too. The frame drawn
// here is copied into `previous_texture` for the next.

// What Bevy's fullscreen vertex shader hands on, written out: Bevy 0.20 gives a
// `.wgsl` file to wgpu as it is, with no `#import` to resolve.
struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Afterglow {
    // What a lit pixel keeps of itself next frame.
    decay: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> afterglow: Afterglow;
@group(0) @binding(3) var previous_texture: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let now = textureSample(screen_texture, texture_sampler, in.uv);
    let before = textureSample(previous_texture, texture_sampler, in.uv);
    // Only ever fades: a pixel is what it is now, or what it was dimmed, the
    // brighter. Nothing is added that was not on screen a moment ago.
    return vec4<f32>(max(now.rgb, before.rgb * afterglow.decay), now.a);
}
