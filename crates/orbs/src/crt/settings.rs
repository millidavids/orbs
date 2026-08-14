//! What the tube is doing, and how hard.
//!
//! DESIGN.md §4 lists the effects and §14 requires the whole thing be
//! **disableable** — `court_wizard` ships a health warning for motion sickness and
//! this inherits the obligation. Every field reaches zero, and
//! [`CrtSettings::OFF`] is one value away.

use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::ShaderType;

/// Put this on the camera to curve the picture.
#[derive(Component, ExtractComponent, Debug, Clone, Copy, PartialEq)]
pub(crate) struct CrtSettings {
    /// Whether the tube is on at all.
    ///
    /// **Explicit, and deliberately not inferred from "are all the effects
    /// zero".** §14 makes this the accessibility switch, and a switch that
    /// something else can flip back on is not a switch.
    ///
    /// `enabled` used to be derived as `settings != OFF`. Both [`flash`] and
    /// [`desaturation`] are documented right here as *reserved for world state*,
    /// so the moment anything drove one — a breach flash, threat desaturation,
    /// the Phase 0.5 boot strike — the settings stopped equalling `OFF`, and
    /// barrel, scanlines, grille and vignette all came back for a player who had
    /// turned them off for motion sickness.
    ///
    /// [`flash`]: Self::flash
    /// [`desaturation`]: Self::desaturation
    pub(crate) on: bool,
    /// How far the tube bulges. 0 is a flat panel.
    pub(crate) barrel: f32,
    /// Depth of the dark line between scan rows.
    pub(crate) scanline: f32,
    /// Strength of the aperture grille.
    pub(crate) mask: f32,
    /// How much the corners fall off.
    pub(crate) vignette: f32,
    /// Where the fall-off begins, as a fraction of the half-diagonal.
    pub(crate) vignette_radius: f32,
    /// Convergence error towards the edges.
    ///
    /// Kept very low. This is the effect that most directly costs legibility —
    /// it puts a coloured fringe on every stroke — and a tube reads as a tube
    /// long before the fringe becomes something the eye has to work past.
    pub(crate) aberration: f32,
    /// Phosphor bloom around lit glyphs.
    ///
    /// Restrained on purpose: glow is haze, and haze is the thing a reader has
    /// to look *through*. A tight, dim halo says phosphor; a wide bright one
    /// just says out of focus.
    pub(crate) glow: f32,
    /// Depth of the slow hum band that rolls down the tube.
    ///
    /// Not a blink: see `crt.wgsl`. Whole-screen modulation at CRT frequencies
    /// lands in the photosensitive band, so the hum is spatial and slow.
    pub(crate) flicker: f32,
    /// Rounded bezel, as a fraction of the tube's **shorter** axis.
    ///
    /// Aspect-corrected in the shader, so the corners come out round rather than
    /// stretched along the tube's 4:3. Expressed against the short axis because
    /// that is the one the radius can actually run out of: at 0.5 the corners
    /// meet and the tube is a stadium.
    pub(crate) corner_radius: f32,
    /// Drains colour. Reserved for world state — §4 wires this to failure.
    pub(crate) desaturation: f32,
    /// Additive flash colour. Reserved for world state — flash on breach.
    pub(crate) flash: LinearRgba,
}

impl CrtSettings {
    /// Everything off. What §14's accessibility toggle selects, and the
    /// baseline the Phase 0 legibility test compares against.
    ///
    /// Belt and braces: `on: false` alone would do it, since the shader returns
    /// the untouched sample before reading another field. The zeros stay so that
    /// a future path which forgets to check `on` still draws nothing.
    pub(crate) const OFF: Self = Self {
        on: false,
        barrel: 0.0,
        scanline: 0.0,
        mask: 0.0,
        vignette: 0.0,
        vignette_radius: 1.0,
        aberration: 0.0,
        glow: 0.0,
        flicker: 0.0,
        corner_radius: 0.0,
        desaturation: 0.0,
        flash: LinearRgba::NONE,
    };

    /// A tube in a dark room, tuned so text stays readable.
    ///
    /// Deliberately restrained: §4 says legibility is the product, and the
    /// values that look best on a title card are not the values you can read a
    /// siege log through. The Phase 0 worst-case legibility test is what settles
    /// these, so treat them as a starting point rather than a result.
    pub(crate) const DEFAULT: Self = Self {
        on: true,
        barrel: 0.10,
        scanline: 0.32,
        mask: 0.10,
        vignette: 0.45,
        vignette_radius: 0.90,
        aberration: 0.0005,
        glow: 0.5,
        flicker: 0.10,
        // **The largest radius that loses no cell**, and it is a computed bound
        // rather than a taste (§19: compute the constant, do not reason about
        // it). The mask is evaluated on the texture coordinate, so a cell's own
        // grid position is what gets tested; the binding one is the session
        // pane's top-left border corner at `(0, 0)`, and the bound is `0.0278`.
        // Anything larger eats it — at `0.09` the arc reached the `d` of the
        // prompt, which is what a screenshot showed and no amount of looking at
        // the number would have.
        //
        // This is barely a change: it was `0.028`, a hair *over* the bound and
        // getting away with it because the radius was being applied to the
        // **unwarped** tube, whose corners lie outside the picture — so it
        // rounded nothing at any value and the picture kept a hard 90° corner.
        // Now that it bites, the bound has to actually hold.
        corner_radius: 0.027,
        desaturation: 0.0,
        flash: LinearRgba::NONE,
    };

    /// Peak threat: what the legibility test must be run against (§4).
    ///
    /// "Maximum flicker and vignette pulse" — the state in which a player still
    /// has to spot a one-character sabotage tell.
    pub(crate) const PEAK_THREAT: Self = Self {
        vignette: 0.85,
        flicker: 0.22,
        aberration: 0.0012,
        desaturation: 0.25,
        ..Self::DEFAULT
    };
}

impl Default for CrtSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The GPU-side mirror. Field order must match `crt.wgsl`.
#[derive(ShaderType, Debug, Clone, Copy)]
pub(crate) struct CrtUniform {
    pub(crate) barrel: f32,
    pub(crate) scanline: f32,
    pub(crate) mask: f32,
    pub(crate) vignette: f32,
    pub(crate) vignette_radius: f32,
    pub(crate) aberration: f32,
    pub(crate) glow: f32,
    pub(crate) flicker: f32,
    pub(crate) corner_radius: f32,
    pub(crate) desaturation: f32,
    pub(crate) flash_r: f32,
    pub(crate) flash_g: f32,
    pub(crate) flash_b: f32,
    pub(crate) flash: f32,
    pub(crate) time: f32,
    /// Physical pixels per cell. Everything periodic in the shader divides
    /// these, so the pattern lands identically inside every glyph at every
    /// window size (§9).
    pub(crate) cell_width: f32,
    pub(crate) cell_height: f32,
    /// The picture's share of the window on each axis — see
    /// [`Tube`](crate::crt::Tube). Everything *shaped* is measured against
    /// these, so the tube is the 4:3 picture and the letterbox bars are the dark
    /// room around it rather than part of the glass.
    pub(crate) fill_x: f32,
    pub(crate) fill_y: f32,
    pub(crate) enabled: f32,
}

impl CrtUniform {
    pub(crate) fn new(settings: CrtSettings, time: f32, tube: crate::crt::Tube) -> Self {
        Self {
            barrel: settings.barrel,
            scanline: settings.scanline,
            mask: settings.mask,
            vignette: settings.vignette,
            vignette_radius: settings.vignette_radius,
            aberration: settings.aberration,
            glow: settings.glow,
            flicker: settings.flicker,
            corner_radius: settings.corner_radius,
            desaturation: settings.desaturation,
            flash_r: settings.flash.red,
            flash_g: settings.flash.green,
            flash_b: settings.flash.blue,
            flash: settings.flash.alpha,
            time,
            cell_width: tube.width,
            cell_height: tube.height,
            fill_x: tube.fill_x,
            fill_y: tube.fill_y,
            enabled: f32::from(u8::from(settings.on)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tube at some cell size, filling its window. The fill is only read by
    /// the shader, so these tests care about the pair the uniform carries.
    fn tube(width: f32, height: f32) -> crate::crt::Tube {
        crate::crt::Tube {
            width,
            height,
            ..crate::crt::Tube::default()
        }
    }

    #[test]
    fn the_rounded_bezel_loses_no_cell() {
        // Architectural rule 2: a frontend may enrich what the other cannot
        // reproduce **provided it carries no information absent from the
        // Frame**, and a corner the tube has eaten carries less. The bezel is
        // the one CRT term that can delete a cell outright rather than dim it,
        // so its radius is a bound to hold rather than a number to like.
        //
        // This is `crt.wgsl`'s rounded-box SDF in Rust, evaluated at the four
        // corner cells of the grid. It is a second implementation of the same
        // arithmetic, which is the cost of the shader being untestable — but the
        // formula is six lines and the alternative is finding out by screenshot,
        // which is exactly how the `0.09` that ate the prompt's first letter got
        // as far as a window.
        let aspect = 4.0 / 3.0_f32;
        let grid = orbs_render::GRID;
        let radius = CrtSettings::DEFAULT.corner_radius;

        let outside = |col: u16, row: u16| {
            let at = (
                (f32::from(col) + 0.5) / f32::from(grid.cols),
                (f32::from(row) + 0.5) / f32::from(grid.rows),
            );
            let from_middle = ((at.0 - 0.5).abs() * aspect, (at.1 - 0.5).abs());
            let quadrant = (
                from_middle.0 - (0.5 * aspect - radius),
                from_middle.1 - (0.5 - radius),
            );
            let outer = quadrant.0.max(0.0).hypot(quadrant.1.max(0.0));
            outer + quadrant.0.max(quadrant.1).min(0.0) - radius
        };

        let (right, bottom) = (grid.cols - 1, grid.rows - 1);
        for (col, row) in [(0, 0), (right, 0), (0, bottom), (right, bottom)] {
            assert!(
                outside(col, row) <= 0.0,
                "the bezel eats the cell at ({col}, {row}): radius {radius} is too large",
            );
        }

        // ...and that it is the *largest* such radius, so the bound is known to
        // be tight rather than merely safe. A corner cell is what binds, and
        // 10% more radius must fail — otherwise the rounding is being left on
        // the table for no reason and nobody would notice.
        let generous = radius * 1.1;
        let slack = {
            let at = (0.5 / f32::from(grid.cols), 0.5 / f32::from(grid.rows));
            let from_middle = ((at.0 - 0.5).abs() * aspect, (at.1 - 0.5).abs());
            let quadrant = (
                from_middle.0 - (0.5 * aspect - generous),
                from_middle.1 - (0.5 - generous),
            );
            quadrant.0.max(0.0).hypot(quadrant.1.max(0.0)) + quadrant.0.max(quadrant.1).min(0.0)
                - generous
        };
        assert!(slack > 0.0, "the radius could be larger than {radius}");
    }

    #[test]
    fn off_really_is_off() {
        // §14 requires the CRT be fully disableable. "Mostly off" is not a
        // setting a player with motion sickness can rely on.
        let uniform = CrtUniform::new(CrtSettings::OFF, 12.0, tube(32.0, 64.0));
        assert_eq!(uniform.enabled, 0.0);
        for value in [
            uniform.barrel,
            uniform.scanline,
            uniform.mask,
            uniform.vignette,
            uniform.aberration,
            uniform.glow,
            uniform.flicker,
            uniform.corner_radius,
            uniform.desaturation,
            uniform.flash,
        ] {
            assert_eq!(value, 0.0);
        }
    }

    #[test]
    fn nothing_world_driven_can_switch_a_disabled_tube_back_on() {
        // The defect this replaced: `enabled` was `settings != OFF`, and both
        // `flash` and `desaturation` are documented as world-driven. Driving
        // either made the settings unequal to `OFF`, so `enabled` flipped to 1.0
        // and the whole tube — barrel, scanlines, grille, vignette — came back
        // for a player who had turned it off for motion sickness (§14).
        //
        // Checked field by field rather than through the toggle, because the
        // failure was never that someone pressed the wrong key. It was that
        // something else wrote a field.
        for lit in [
            CrtSettings {
                flash: LinearRgba::rgb(1.0, 1.0, 1.0),
                ..CrtSettings::OFF
            },
            CrtSettings {
                desaturation: 0.9,
                ..CrtSettings::OFF
            },
            CrtSettings {
                vignette: 0.85,
                ..CrtSettings::OFF
            },
        ] {
            assert_ne!(lit, CrtSettings::OFF, "the test is not testing anything");
            let uniform = CrtUniform::new(lit, 0.0, tube(8.0, 16.0));
            assert_eq!(uniform.enabled, 0.0, "{lit:?} switched the tube on");
        }
    }

    #[test]
    fn being_on_is_stated_rather_than_inferred_from_the_effects() {
        // A tube whose every effect happens to sit at zero is still *on* — it is
        // simply a flat, quiet one. Conflating the two is what let a single
        // written field resurrect the whole picture.
        let quiet = CrtSettings {
            on: true,
            ..CrtSettings::OFF
        };
        assert_eq!(CrtUniform::new(quiet, 0.0, tube(8.0, 16.0)).enabled, 1.0);
    }

    #[test]
    fn the_default_tube_is_on() {
        assert_eq!(
            CrtUniform::new(CrtSettings::DEFAULT, 0.0, tube(8.0, 16.0)).enabled,
            1.0
        );
    }

    #[test]
    fn peak_threat_degrades_legibility_in_every_channel_the_design_names() {
        // §4: the legibility test runs against "maximum flicker and vignette
        // pulse". If peak threat were gentler than the default in any of these,
        // the test would not be testing the worst case.
        let peak = CrtSettings::PEAK_THREAT;
        let normal = CrtSettings::DEFAULT;
        assert!(peak.flicker > normal.flicker);
        assert!(peak.vignette > normal.vignette);
        assert!(peak.aberration > normal.aberration);
        assert!(peak.desaturation > normal.desaturation);
    }

    #[test]
    fn the_cell_size_reaches_the_shader() {
        // §9: scanline and grille frequencies re-derive against the active cell
        // size. If the cell never arrived, they could not.
        let uniform = CrtUniform::new(CrtSettings::DEFAULT, 0.0, tube(32.0, 64.0));
        assert_eq!(uniform.cell_width, 32.0);
        assert_eq!(uniform.cell_height, 64.0);
    }
}
