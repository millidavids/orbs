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
    pub(crate) aberration: f32,
    /// Phosphor bloom around lit glyphs.
    pub(crate) glow: f32,
    /// Mains hum.
    pub(crate) flicker: f32,
    /// Rounded bezel, in UV.
    pub(crate) corner_radius: f32,
    /// Drains colour. Reserved for world state — §4 wires this to failure.
    pub(crate) desaturation: f32,
    /// Additive flash colour. Reserved for world state — flash on breach.
    pub(crate) flash: LinearRgba,
}

impl CrtSettings {
    /// Everything off. What §14's accessibility toggle selects, and the
    /// baseline the Phase 0 legibility test compares against.
    pub(crate) const OFF: Self = Self {
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
        barrel: 0.10,
        scanline: 0.16,
        mask: 0.10,
        vignette: 0.45,
        vignette_radius: 0.90,
        aberration: 0.0015,
        glow: 0.85,
        flicker: 0.02,
        corner_radius: 0.028,
        desaturation: 0.0,
        flash: LinearRgba::NONE,
    };

    /// Peak threat: what the legibility test must be run against (§4).
    ///
    /// "Maximum flicker and vignette pulse" — the state in which a player still
    /// has to spot a one-character sabotage tell.
    pub(crate) const PEAK_THREAT: Self = Self {
        vignette: 0.85,
        flicker: 0.10,
        aberration: 0.0035,
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
    /// fidelity tier (§9).
    pub(crate) cell_width: f32,
    pub(crate) cell_height: f32,
    pub(crate) enabled: f32,
}

impl CrtUniform {
    pub(crate) fn new(settings: CrtSettings, time: f32, cell: (f32, f32)) -> Self {
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
            cell_width: cell.0,
            cell_height: cell.1,
            enabled: f32::from(u8::from(settings != CrtSettings::OFF)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_really_is_off() {
        // §14 requires the CRT be fully disableable. "Mostly off" is not a
        // setting a player with motion sickness can rely on.
        let uniform = CrtUniform::new(CrtSettings::OFF, 12.0, (32.0, 64.0));
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
    fn the_default_tube_is_on() {
        assert_eq!(
            CrtUniform::new(CrtSettings::DEFAULT, 0.0, (8.0, 16.0)).enabled,
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
        let uniform = CrtUniform::new(CrtSettings::DEFAULT, 0.0, (32.0, 64.0));
        assert_eq!(uniform.cell_width, 32.0);
        assert_eq!(uniform.cell_height, 64.0);
    }
}
