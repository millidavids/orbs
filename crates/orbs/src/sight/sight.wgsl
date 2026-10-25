// The accommodation pass — DESIGN.md §14, ROADMAP Phase 13.
//
// The last thing to touch a pixel, since the tube's grille, aberration and
// flash put hue back (§19). A separate pass, since `crt.wgsl` returns early
// with the tube off, and F3 must not switch this off too.

// What Bevy's fullscreen vertex shader hands on, written out: Bevy 0.20 gives a
// `.wgsl` file to wgpu as it is, with no `#import` to resolve.
struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Sight {
    // 1.0 for greyscale, 0.0 for none. A float rather than a `u32` because
    // uniform padding rules make a lone `u32` the awkward case, and because the
    // next thing this grows is a strength.
    greyscale: f32,
    // 1.0 as authored; lower while the window is in the background and the
    // player asked the glass to cloud (`away`).
    brightness: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> sight: Sight;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let source = textureSample(screen_texture, texture_sampler, in.uv);

    // **Rec.709, matching `palette::luminance`.** `crt.wgsl`'s own
    // `desaturation` term uses NTSC (0.299, 0.587, 0.114) and is left alone —
    // it is reserved for world state, it lives below the tube's off-switch, and
    // a greyscale built on it would put the player in a grey the accent triad
    // was never proved separable in. The guarantee and the picture have to agree
    // about which grey they mean.
    //
    // The view format is sRGB, so `textureSample` has already decoded and the
    // values here are linear — which is the space these weights are defined for.
    // No gamma pair: `court_wizard`'s shader applies one and would be applying
    // the curve twice here.
    let luma = dot(source.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));

    let seen = mix(source.rgb, vec3<f32>(luma), sight.greyscale);
    return vec4<f32>(seen * sight.brightness, source.a);
}
