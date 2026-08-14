// The orb: a curved phosphor screen in a dark room.
//
// Ported from court_wizard's crt_effect.wgsl, with three changes the design
// requires (DESIGN.md §4, §9):
//
//   * Scanline and aperture-grille frequencies derive from the **cell size**,
//     not from a hardcoded 1080-line reference. §9: they "must re-derive against
//     the active cell size, or the tier change produces exactly the moiré §4
//     identifies as the top legibility hazard". Both are fractions of a cell, so
//     the pattern lands on the same place in every glyph at every window size
//     instead of beating against the stems. They are no longer *integer*
//     fractions: §19 fixed the grid and made the cell scale continuous, so a
//     1080p window is 1.5x and the grille stripe is a pixel and a half. The
//     clamps at `scan_period` and `stripe` are what hold that together, and
//     neither engages above `MIN_SCALE`.
//   * **The tube is the 4:3 picture, not the window.** The camera letterboxes a
//     fixed 960x720 picture with `ScalingMode::AutoMin`, so what arrives here is
//     the whole window with bars down one axis. Every *shaped* term below — the
//     barrel curve, the vignette, the rounded bezel, the edge mask — is measured
//     in **tube space**, the picture remapped to 0..1, and everything outside it
//     is the dark room the monitor stands in.
//
//     Doing it in window space instead, which is how this shipped for one
//     version, curves the bars along with the phosphor: the "monitor" is then
//     whatever rectangle the player dragged, and on a wide window it is a
//     letterbox-shaped tube nobody ever built. This comment used to say the grid
//     filled the window and the aspect was the monitor's; §19 reversed the first
//     and this restores the second.
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
    // The picture's share of the window on each axis. Everything shaped derives
    // from these. One of the two is always 1.0.
    fill_x: f32,
    fill_y: f32,
    enabled: f32,
}
@group(0) @binding(2) var<uniform> crt: CrtUniform;

/// How often the hum band crosses the screen, in hertz. Deliberately far below
/// the 3 Hz floor of the photosensitive band (§14).
const HUM_ROLL_HZ: f32 = 0.22;

/// The tube's width over its height — 4:3, and fixed, because the grid is.
///
/// Hardcoded rather than passed: `orbs_render::GRID` is a compile-time constant
/// with a const assertion holding it on the 8:3 column-to-row line, so this
/// cannot drift without that assertion failing the build first.
const GRID_ASPECT: f32 = 4.0 / 3.0;

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

/// Where the tube's glass sits in the window, as a rectangle in UV.
///
/// The picture is centred by `ScalingMode::AutoMin` — `viewport_origin` defaults
/// to the middle — so the origin is derived rather than passed.
fn tube_fill() -> vec2<f32> {
    return max(vec2<f32>(crt.fill_x, crt.fill_y), vec2<f32>(0.0001));
}

/// Window UV to **tube space**: `0..1` spans the 4:3 picture, and the letterbox
/// bars are the region outside it.
fn to_tube(uv: vec2<f32>) -> vec2<f32> {
    let fill = tube_fill();
    return (uv - (vec2<f32>(1.0) - fill) * 0.5) / fill;
}

/// ...and back, to sample the screen texture the tube coordinate names.
fn to_window(tube: vec2<f32>) -> vec2<f32> {
    let fill = tube_fill();
    return tube * fill + (vec2<f32>(1.0) - fill) * 0.5;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Every textureSample must happen before any non-uniform branch.
    //
    // The whole pipeline runs in tube space: warp there, then map back to
    // window UV at the moment of sampling. A fragment out in the bars lands well
    // outside `0..1` here, so `inside` below zeroes it and the bars come out as
    // the dark room for free — no separate mask, and no branch.
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

    // Phosphor bloom: four cardinal taps *half* a glyph pixel away, so the halo
    // hugs the stroke instead of spreading into a haze the eye has to look
    // through. It still scales with the fidelity tier.
    //
    // Two earlier values were wrong in the same direction. A third of a cell
    // ghosted outright — every line of text had a visible duplicate below it. A
    // whole glyph pixel stopped ghosting but left the text soft, which §4 does
    // not permit: legibility is the product.
    //
    // In **window** UV, unlike everything else here, because it is added to a
    // sampling coordinate rather than to a shape: `cell_width` is already
    // physical pixels and `dims` is the physical window, so the ratio is right
    // without passing through the tube.
    let dims = vec2<f32>(textureDimensions(screen_texture));
    let spread = vec2<f32>(crt.cell_width, crt.cell_height) / (16.0 * dims);
    let up = textureSample(screen_texture, texture_sampler, clamp(at + vec2<f32>(0.0, spread.y), vec2<f32>(0.0), vec2<f32>(1.0)));
    let down = textureSample(screen_texture, texture_sampler, clamp(at - vec2<f32>(0.0, spread.y), vec2<f32>(0.0), vec2<f32>(1.0)));
    let left = textureSample(screen_texture, texture_sampler, clamp(at - vec2<f32>(spread.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0)));
    let right = textureSample(screen_texture, texture_sampler, clamp(at + vec2<f32>(spread.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0)));

    // The picture with nothing done to it, for `F3` off.
    //
    // **Not `sample_g`**, which is warped: `barrel` applies `OVERSCAN` even at
    // zero strength, and out in the bars the tube coordinate clamps to the edge
    // of the glass — so "off" would have shown the outermost pixel column
    // smeared across the letterbox. Sampling the window straight is what off
    // should mean anyway.
    let plain = textureSample(screen_texture, texture_sampler, in.uv);

    // --- sampling done; branching is safe ---

    if crt.enabled < 0.5 {
        return plain;
    }

    // Beyond the curved edge there is no screen, only the dark room.
    //
    // **One boundary, with rounded corners** — a rounded-box signed distance
    // over the warped coordinate. It was two: an axis-wise rectangle here, plus
    // a separate corner mask measured against the *unwarped* tube. Those are
    // different rectangles, because the warp insets the picture from the glass,
    // so the corner mask rounded a corner outside the visible area and the
    // picture kept a hard 90° angle however the radius was tuned. Taking a
    // screenshot and looking at a corner is what found that; no amount of
    // reasoning about the radius would have.
    //
    // Scaled by the aspect so the corners are **round rather than elliptical**:
    // the tube is 4:3, and a radius in its UV is stretched along with it.
    let aspect = vec2<f32>(GRID_ASPECT, 1.0);
    let half = vec2<f32>(0.5) * aspect;
    let from_middle = abs(curved - vec2<f32>(0.5)) * aspect;
    let quadrant = from_middle - (half - vec2<f32>(crt.corner_radius));
    // The full rounded-box SDF. The `min(max(...), 0)` term is what makes it a
    // true distance *inside* the box as well as outside — without it the sign is
    // right but the magnitude is zero everywhere inside, and the edge falls off
    // over the whole face instead of over a pixel.
    let sdf = length(max(quadrant, vec2<f32>(0.0)))
        + min(max(quadrant.x, quadrant.y), 0.0)
        - crt.corner_radius;

    // The band is deliberately tiny, and straddles the boundary so the arc is
    // antialiased rather than stair-stepped. 0.0015 of the short axis is under
    // two physical pixels at 1080p — where 0.004 was ten pixels at 2560 wide, a
    // third of a cell, enough to dim the first column of the input line.
    let inside = 1.0 - smoothstep(-0.0015, 0.0015, sdf);

    var colour = vec3<f32>(sample_r.r, sample_g.g, sample_b.b) * inside;

    // Phosphor glow, weighted by how bright the neighbourhood already is, so
    // lit glyphs bloom and the background stays black.
    let neighbours = (up.rgb + down.rgb + left.rgb + right.rgb) * 0.25;
    let brightness = max(neighbours.r, max(neighbours.g, neighbours.b));
    colour += neighbours * brightness * crt.glow * inside;

    // Physical pixels **within the tube**, which is what the periodic terms
    // below count in. `curved` is tube space, so the window's own size does not
    // enter into it — only the picture's, which is `fill` of the window.
    let pixel = curved * tube_fill() * dims;

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

    // Vignette, then the rounded bezel — **both in tube space**, so they are the
    // glass darkening at its own edge and the monitor's own corners. Measured
    // against `in.uv` they were the *window's* edge and corners: on a wide
    // window the vignette closed in over the bars while the picture sat evenly
    // lit, and the rounded corners were nowhere near the tube.
    let radial = length(tube - vec2<f32>(0.5));
    let vignette = smoothstep(crt.vignette_radius, crt.vignette_radius - 0.45, radial);
    colour *= mix(1.0, vignette, crt.vignette);

    // The bezel is not a second mask any more — `inside` above is the rounded
    // boundary, applied at the top where the dark room is decided.

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
