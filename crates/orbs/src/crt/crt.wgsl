// The orb: a curved phosphor screen in a dark room.
//
// Ported from court_wizard's crt_effect.wgsl, changed three ways (§4, §9, §19):
//
//   * Scanlines and grille are fractions of the cell size, so they land alike
//     in every glyph instead of beating into moiré; the `scan_period` and
//     `stripe` clamps hold the continuous scale.
//   * The tube is the 4:3 picture, not the window: every shaped term is in
//     tube space, and outside it is the dark room.
//   * No channel-change effect, the wrong metaphor for a scrying orb.
//
// Legibility is the product: every term can go to zero, and the whole is
// disableable (§14).

// What Bevy's fullscreen vertex shader hands on, written out: Bevy 0.20 gives a
// `.wgsl` file to wgpu as it is, with no `#import` to resolve.
struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct CrtUniform {
    barrel: f32,
    scanline: f32,
    mask: f32,
    vignette: f32,
    vignette_radius: f32,
    aberration: f32,
    glow: f32,
    flicker: f32,
    corner_radius: f32,
    desaturation: f32,
    flash_r: f32,
    flash_g: f32,
    flash_b: f32,
    flash: f32,
    time: f32,
    // Physical pixels per cell. Everything periodic derives from these.
    cell_width: f32,
    cell_height: f32,
    // The picture's share of the window on each axis. Everything shaped derives
    // from these. One of the two is always 1.0.
    fill_x: f32,
    fill_y: f32,
    // Where the picture's middle sits from the window's, in UV. Zero but for a
    // whole fit on an odd window.
    shift_x: f32,
    shift_y: f32,
    enabled: f32,
}
@group(0) @binding(2) var<uniform> crt: CrtUniform;

/// How often the hum band crosses the screen, in hertz. Deliberately far below
/// the 3 Hz floor of the photosensitive band (§14).
const HUM_ROLL_HZ: f32 = 0.22;

/// The tube's width over its height — 4:3, and fixed, because the grid is.
///
/// Hardcoded, since `orbs_render::GRID`'s const assertion holds it at 8:3.
const GRID_ASPECT: f32 = 4.0 / 3.0;

/// How far inside the tube face the picture sits, as a fraction of the screen.
///
/// The dark surround between glass and phosphor. Zero is the smallest safe
/// value; below it cuts cells off. At a 2560-wide window:
///
/// | Value | Surround | Safe |
/// |---|---|---|
/// | `0.000` | 31px | yes — the minimum |
/// | `0.020` | 50px | yes |
/// | `0.080` | 124px | yes |
///
/// Tune freely upward. See [`barrel`] for why downward is a correctness matter.
const OVERSCAN: f32 = 0.02;

/// Pincushion the image outward from the centre as a curved tube does, and inset
/// it far enough that the whole grid survives the curve.
///
/// No cell may be lost (rule 2, §19). A radial warp cannot keep a rectangle, and
/// pinning the corners cuts the edge midpoints, so the picture scales outward
/// past the screen and the mask paints the dark room.
fn barrel(uv: vec2<f32>, strength: f32) -> vec2<f32> {
    let centred = uv - vec2<f32>(0.5);
    let warp = centred * (1.0 + strength * dot(centred, centred));
    return warp * (1.0 + OVERSCAN) + vec2<f32>(0.5);
}

/// How much of the window the tube's glass covers, in UV.
fn tube_fill() -> vec2<f32> {
    return max(vec2<f32>(crt.fill_x, crt.fill_y), vec2<f32>(0.0001));
}

/// The glass's top-left corner, in UV: centred, then moved by the shift the
/// camera's whole-pixel origin made.
fn tube_corner() -> vec2<f32> {
    return (vec2<f32>(1.0) - tube_fill()) * 0.5 + vec2<f32>(crt.shift_x, crt.shift_y);
}

/// Window UV to **tube space**: `0..1` spans the 4:3 picture, and the letterbox
/// bars are the region outside it.
fn to_tube(uv: vec2<f32>) -> vec2<f32> {
    return (uv - tube_corner()) / tube_fill();
}

/// ...and back, to sample the screen texture the tube coordinate names.
fn to_window(tube: vec2<f32>) -> vec2<f32> {
    return tube * tube_fill() + tube_corner();
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Every textureSample must happen before any non-uniform branch.
    //
    // Warp in tube space, map back to window UV to sample. A fragment in the
    // bars lands outside `0..1`, so `inside` zeroes it with no branch.
    let tube = to_tube(in.uv);
    let curved = barrel(tube, crt.barrel);
    let safe = clamp(curved, vec2<f32>(0.0), vec2<f32>(1.0));
    let at = to_window(safe);

    // Chromatic aberration: red and blue drift apart towards the edges, as a
    // real tube's convergence does. In tube space, so the drift is radial from
    // the middle of the glass rather than the middle of the window.
    let from_centre = safe - vec2<f32>(0.5);
    let drift = from_centre * crt.aberration * tube_fill();
    let sample_r = textureSample(screen_texture, texture_sampler, clamp(at + drift, vec2<f32>(0.0), vec2<f32>(1.0)));
    let sample_g = textureSample(screen_texture, texture_sampler, at);
    let sample_b = textureSample(screen_texture, texture_sampler, clamp(at - drift, vec2<f32>(0.0), vec2<f32>(1.0)));

    // Phosphor bloom: four taps half a glyph pixel away, so the halo hugs the
    // stroke; further ghosted or softened the text (§4). In window UV, being
    // added to a sampling coordinate, not a shape.
    let dims = vec2<f32>(textureDimensions(screen_texture));
    let spread = vec2<f32>(crt.cell_width, crt.cell_height) / (16.0 * dims);
    let up = textureSample(screen_texture, texture_sampler, clamp(at + vec2<f32>(0.0, spread.y), vec2<f32>(0.0), vec2<f32>(1.0)));
    let down = textureSample(screen_texture, texture_sampler, clamp(at - vec2<f32>(0.0, spread.y), vec2<f32>(0.0), vec2<f32>(1.0)));
    let left = textureSample(screen_texture, texture_sampler, clamp(at - vec2<f32>(spread.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0)));
    let right = textureSample(screen_texture, texture_sampler, clamp(at + vec2<f32>(spread.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0)));

    // The picture with nothing done to it, for `F3` off.
    //
    // Not the warped `sample_g`, which would smear the edge across the bars.
    let plain = textureSample(screen_texture, texture_sampler, in.uv);

    // --- sampling done; branching is safe ---

    if crt.enabled < 0.5 {
        return plain;
    }

    // Beyond the curved edge there is no screen, only the dark room.
    //
    // One rounded-box boundary over the warped coordinate, since a mask on the
    // unwarped tube rounds a corner nobody sees. Scaled by the aspect, so the
    // corners are round, not elliptical.
    let aspect = vec2<f32>(GRID_ASPECT, 1.0);
    let half = vec2<f32>(0.5) * aspect;
    let from_middle = abs(curved - vec2<f32>(0.5)) * aspect;
    let quadrant = from_middle - (half - vec2<f32>(crt.corner_radius));
    // The `min(max(...), 0)` term makes it a true distance inside the box too.
    let sdf = length(max(quadrant, vec2<f32>(0.0)))
        + min(max(quadrant.x, quadrant.y), 0.0)
        - crt.corner_radius;

    // A tiny band straddling the edge, antialiasing the arc in under two pixels
    // at 1080p without dimming the first column.
    let inside = 1.0 - smoothstep(-0.0015, 0.0015, sdf);

    var colour = vec3<f32>(sample_r.r, sample_g.g, sample_b.b) * inside;

    // Phosphor glow, weighted by how bright the neighbourhood already is, so
    // lit glyphs bloom and the background stays black.
    let neighbours = (up.rgb + down.rgb + left.rgb + right.rgb) * 0.25;
    let brightness = max(neighbours.r, max(neighbours.g, neighbours.b));
    colour += neighbours * brightness * crt.glow * inside;

    // Physical pixels within the tube, which the periodic terms count in.
    let pixel = curved * tube_fill() * dims;

    // Scanlines, one every eighth of a cell, so the pattern sits alike in every
    // glyph.
    let scan_period = max(crt.cell_height / 8.0, 2.0);
    let scan_phase = sin(pixel.y * 6.28318 / scan_period);
    colour *= 1.0 - crt.scanline * (0.5 + 0.5 * scan_phase);

    // Aperture grille. One R, G, or B stripe per *glyph pixel* — cell_width / 8
    // — so a stripe never straddles a stem edge, which is what produces moiré.
    let stripe = max(crt.cell_width / 8.0, 1.0);
    let triad = fract(pixel.x / (stripe * 3.0)) * 3.0;
    let dim = 1.0 - crt.mask;
    let grille = vec3<f32>(
        select(dim, 1.0, triad < 1.0),
        select(dim, 1.0, triad >= 1.0 && triad < 2.0),
        select(dim, 1.0, triad >= 2.0),
    );
    colour *= grille;

    // Mains hum as a rolling band, not a blink: a flicker fast enough to read as
    // hum sits in the photosensitive band or aliases (§14, §19).
    let roll = fract(curved.y - crt.time * HUM_ROLL_HZ);
    let band = smoothstep(0.0, 0.25, roll) * smoothstep(0.5, 0.25, roll);
    colour *= 1.0 - crt.flicker * band;

    // The vignette, in tube space, so it is the glass's own edge.
    let radial = length(tube - vec2<f32>(0.5));
    let vignette = smoothstep(crt.vignette_radius, crt.vignette_radius - 0.45, radial);
    colour *= mix(1.0, vignette, crt.vignette);

    // Wired to world state later: flash on breach, desaturate as things fail.
    if crt.flash > 0.0 {
        colour += vec3<f32>(crt.flash_r, crt.flash_g, crt.flash_b) * crt.flash * inside;
    }

    if crt.desaturation > 0.0 {
        let luma = dot(colour, vec3<f32>(0.299, 0.587, 0.114));
        colour = mix(colour, vec3<f32>(luma), crt.desaturation);
    }

    return vec4<f32>(colour, 1.0);
}
