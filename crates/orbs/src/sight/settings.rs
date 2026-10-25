//! What the accommodation is set to, and the uniform it becomes.
//!
//! The setting itself — its names, `F8`'s order, how it opens — is
//! `orbs_shell::Sight`, which the terminal reads too. This is the GPU half.

use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use orbs_shell::Sight;

/// The player's chosen accommodation (DESIGN.md §14).
///
/// A resource rather than a component on the camera, unlike `CrtSettings`: it is
/// a *setting*, one per session, and nothing in the world may write it.
/// `CrtSettings` is driven by world state (§4) and this must never be.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct Vision(pub(crate) Sight);

/// What the shader is told.
///
/// Field order must match `sight.wgsl`'s `Sight` struct — the hand-maintained
/// contract `CrtUniform` carries. Nothing checks it, and a mismatch is a silent
/// wrong picture rather than a compile error.
#[derive(ShaderType, Debug, Clone, Copy, PartialEq)]
pub(crate) struct SightUniform {
    /// 1.0 to take the hue out, 0.0 to leave it.
    pub(crate) greyscale: f32,
    /// How bright the glass is drawn, 1.0 as authored — `away`'s cloud.
    pub(crate) brightness: f32,
    pub(crate) _pad1: f32,
    pub(crate) _pad2: f32,
}

impl SightUniform {
    /// Build the uniform for a setting, and the glass's brightness.
    ///
    /// Takes no `CrtSettings`, so the tube's own off-switch cannot reach it —
    /// an accommodation the tube could turn off is a defect §19 already
    /// records once. The cloud is here for the same reason: it must hold with
    /// the tube off.
    pub(crate) const fn new(sight: Sight, brightness: f32) -> Self {
        Self {
            greyscale: match sight {
                Sight::Plain => 0.0,
                Sight::Greyscale => 1.0,
            },
            brightness,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }

    /// Whether this pass would change anything.
    ///
    /// The pass is skipped when it would not, so the ordinary case costs no
    /// fullscreen ping-pong at all.
    pub(crate) fn wanted(self) -> bool {
        self.greyscale > 0.0 || self.brightness < 1.0
    }
}

/// Step through the accommodations — `F8`.
///
/// The `F2`/`F3` shape, and here for the same reason §14 gives for those: an
/// accommodation a player cannot reach is not an accommodation. Phase 13's
/// settings screen replaces all three.
pub(super) fn cycle(mut vision: ResMut<Vision>) {
    let next = vision.0.next();
    vision.0 = next;
    info!("sight: {}", next.name());
    // The key and the settings page are one setting seen twice — see
    // `crt::tube::cycle`. An accommodation that did not survive a relaunch
    // would be the worst one to leave un-sticky.
    orbs_shell::settings::set(orbs_shell::settings::SIGHT, next.name());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_is_the_identity_and_costs_nothing() {
        let uniform = SightUniform::new(Sight::Plain, 1.0);
        assert_eq!(uniform.greyscale, 0.0);
        assert!(
            !uniform.wanted(),
            "the ordinary case must not run a fullscreen pass"
        );
    }

    #[test]
    fn greyscale_takes_all_of_it() {
        assert_eq!(SightUniform::new(Sight::Greyscale, 1.0).greyscale, 1.0);
        assert!(SightUniform::new(Sight::Greyscale, 1.0).wanted());
    }

    #[test]
    fn a_clouded_glass_runs_the_pass_on_a_plain_tube() {
        // `away dim` must hold with no accommodation set, or it would only
        // ever cloud a greyscale tube.
        assert!(SightUniform::new(Sight::Plain, 0.6).wanted());
    }

    // "The tube cannot switch the accommodation off" is structural — no
    // `CrtSettings` is in scope of `new` or `extract` — and SEEING-IT's
    // greyscale matrix is its evidence.
}
