// The orb: a curved phosphor screen in a dark room.
//
// Ported from court_wizard's crt_effect.wgsl, with three changes the design
// requires (DESIGN.md §4, §9):
//
//   * Scanline and aperture-grille frequencies derive from the **cell size**,
//     not from a hardcoded 1080-line reference. §9: they "must re-derive against
//     the active cell size, or the tier change produces exactly the moiré §4
//     identifies as the top legibility hazard". Both are integer fractions of a
//     cell, so the pattern lands on the same place in every glyph at every
//     fidelity tier instead of beating against the stems.
//   * No 16:9 letterbox. The grid fills the window; §4's aspect is whatever the
//     player's monitor is.
//   * No channel-change effect. §4's list does not include it, and a television
//     retuning is the wrong metaphor for a scrying orb.
//
// Legibility is the product, not an aesthetic. Every term here is scaled by a
// setting that can go to zero, and §14 requires the whole thing be disableable.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

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
    enabled: f32,
}
@group(0) @binding(2) var<uniform> crt: CrtUniform;

/// How often the hum band crosses the screen, in hertz. Deliberately far below
/// the 3 Hz floor of the photosensitive band (§14).
const HUM_ROLL_HZ: f32 = 0.22;

/// How far inside the tube face the picture sits, as a fraction of the screen.
///
/// The dark surround a real tube keeps between the glass and the phosphor.
/// Raising it insets the picture further and widens the surround. **Zero is the
/// smallest safe value**; below zero it stops being taste and starts cutting
/// cells off the edges. At a 2560-wide window:
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
/// **No cell may ever be lost.** Architectural rule 2 lets a frontend add
/// enrichment the other cannot reproduce *provided it carries no information
/// absent from the Frame* — a tube that swallows a column carries less. This
/// arithmetic is therefore a correctness matter, and it was wrong twice before
/// it was right (DESIGN.md §19).
///
/// A radial warp cannot map a rectangle onto a rectangle: whichever boundary
/// point is made exact, every other one moves the other way. Normalising so the
/// **corners** land exactly on the screen edge is the intuitive choice and it is
/// the one that cuts content — it pulled the edge midpoints in by 30.5px, a
/// whole cell at tier 4, slicing the left and right pane borders off at
/// mid-height.
///
/// So the picture is scaled *outward* instead, past the screen on every side.
/// The screen then runs out of texture near the edges and the mask below paints
/// the dark room — which costs nothing, and is what a tube looks like anyway.
fn barrel(uv: vec2<f32>, strength: f32) -> vec2<f32> {
    let centred = uv - vec2<f32>(0.5);
    let warp = centred * (1.0 + strength * dot(centred, centred));
    return warp * (1.0 + OVERSCAN) + vec2<f32>(0.5);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Every textureSample must happen before any non-uniform branch.
    let curved = barrel(in.uv, crt.barrel);
    let safe = clamp(curved, vec2<f32>(0.0), vec2<f32>(1.0));

    // Chromatic aberration: red and blue drift apart towards the edges, as a
    // real tube's convergence does.
    let from_centre = safe - vec2<f32>(0.5);
    let drift = from_centre * crt.aberration;
    let sample_r = textureSample(screen_texture, texture_sampler, clamp(safe + drift, vec2<f32>(0.0), vec2<f32>(1.0)));
    let sample_g = textureSample(screen_texture, texture_sampler, safe);
    let sample_b = textureSample(screen_texture, texture_sampler, clamp(safe - drift, vec2<f32>(0.0), vec2<f32>(1.0)));

    // Phosphor bloom: four cardinal taps *half* a glyph pixel away, so the halo
    // hugs the stroke instead of spreading into a haze the eye has to look
    // through. It still scales with the fidelity tier.
    //
    // Two earlier values were wrong in the same direction. A third of a cell
    // ghosted outright — every line of text had a visible duplicate below it. A
    // whole glyph pixel stopped ghosting but left the text soft, which §4 does
    // not permit: legibility is the product.
    let dims = vec2<f32>(textureDimensions(screen_texture));
    let spread = vec2<f32>(crt.cell_width, crt.cell_height) / (16.0 * dims);
    let up = textureSample(screen_texture, texture_sampler, clamp(safe + vec2<f32>(0.0, spread.y), vec2<f32>(0.0), vec2<f32>(1.0)));
    let down = textureSample(screen_texture, texture_sampler, clamp(safe - vec2<f32>(0.0, spread.y), vec2<f32>(0.0), vec2<f32>(1.0)));
    let left = textureSample(screen_texture, texture_sampler, clamp(safe - vec2<f32>(spread.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0)));
    let right = textureSample(screen_texture, texture_sampler, clamp(safe + vec2<f32>(spread.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0)));

    // --- sampling done; branching is safe ---

    if crt.enabled < 0.5 {
        return sample_g;
    }

    // Beyond the curved edge there is no screen, only the dark room.
    //
    // The threshold is deliberately tiny. `barrel` now normalises, so a sample
    // only lands outside [0,1] by a hair — but 0.004 of a 2560-wide window is
    // ten pixels, a third of a cell at tier 4, which was enough to dim the first
    // column of the input line to near-black even once the warp stopped pushing
    // it off entirely.
    let edge_x = smoothstep(0.0, 0.0005, curved.x) * smoothstep(0.0, 0.0005, 1.0 - curved.x);
    let edge_y = smoothstep(0.0, 0.0005, curved.y) * smoothstep(0.0, 0.0005, 1.0 - curved.y);
    let inside = edge_x * edge_y;

    var colour = vec3<f32>(sample_r.r, sample_g.g, sample_b.b) * inside;

    // Phosphor glow, weighted by how bright the neighbourhood already is, so
    // lit glyphs bloom and the background stays black.
    let neighbours = (up.rgb + down.rgb + left.rgb + right.rgb) * 0.25;
    let brightness = max(neighbours.r, max(neighbours.g, neighbours.b));
    colour += neighbours * brightness * crt.glow * inside;

    let pixel = curved * dims;

    // Scanlines, one every eighth of a cell — an integer fraction, so the
    // pattern sits identically inside every glyph at every fidelity tier.
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

    // Mains hum — as a slowly rolling band, not a blink.
    //
    // The original modulated whole-screen brightness by `sin(time * 120.0)`,
    // commented as "60Hz-ish". It is neither: 120 rad/s is **19.1 Hz**, which
    // sits in the middle of the 3–30 Hz band that provokes photosensitive
    // reactions, and on a dark screen it reads as the whole picture blinking.
    // Nor is there a correct frequency to substitute — anything fast enough to
    // pass for mains hum is above a 60 Hz display's Nyquist limit and aliases
    // into noise.
    //
    // So the hum rolls instead. A faint band drifting down the tube once every
    // few seconds is what a camera actually catches off a CRT, it carries the
    // same "this is a live phosphor screen" reading, and its temporal frequency
    // at any given pixel is well under 1 Hz.
    let roll = fract(curved.y - crt.time * HUM_ROLL_HZ);
    let band = smoothstep(0.0, 0.25, roll) * smoothstep(0.5, 0.25, roll);
    colour *= 1.0 - crt.flicker * band;

    // Vignette, then the rounded bezel.
    let radial = length(in.uv - vec2<f32>(0.5));
    let vignette = smoothstep(crt.vignette_radius, crt.vignette_radius - 0.45, radial);
    colour *= mix(1.0, vignette, crt.vignette);

    let corner = abs(in.uv - vec2<f32>(0.5));
    let quadrant = corner - (vec2<f32>(0.5) - vec2<f32>(crt.corner_radius));
    let sdf = length(max(quadrant, vec2<f32>(0.0))) - crt.corner_radius;
    colour *= 1.0 - smoothstep(0.0, 0.006, sdf);

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
