// The accommodation pass — DESIGN.md §14, ROADMAP Phase 13.
//
// **The last thing that touches a pixel**, and that placement is the whole
// design. §19 originally said "after the phosphor and before the barrel"; there
// is no such place. `crt.wgsl` applies the barrel *first* — it decides the
// sampling coordinate — and three later terms put hue back into a pixel that had
// none: the aperture grille attenuates R, G and B by different amounts per
// column, the chromatic aberration is a deliberate coloured fringe, and the
// breach flash is additive. A greyscale pass upstream of any of those would be
// undone by them, and "no hue anywhere" would be false as drawn.
//
// It is a **separate pass** rather than more of `crt.wgsl` for a reason that is
// about switches rather than tidiness: that shader early-returns when the tube
// is off, so a filter living below the guard would be turned off by F3. §19:
// "a switch inferred from the absence of something is not a switch."

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct Sight {
    // 1.0 for greyscale, 0.0 for none. A float rather than a `u32` because
    // uniform padding rules make a lone `u32` the awkward case, and because the
    // next thing this grows is a strength.
    greyscale: f32,
    _pad0: f32,
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

    return vec4<f32>(mix(source.rgb, vec3<f32>(luma), sight.greyscale), source.a);
}
