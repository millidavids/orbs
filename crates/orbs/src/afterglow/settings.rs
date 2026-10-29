//! How long a lit phosphor lingers, frame by frame (§19, *the tower speaks up*).

use bevy::prelude::*;
use bevy::render::Extract;
use bevy::render::render_resource::ShaderType;

use crate::crt::{CrtSettings, Effect, Picture};
use crate::fullscreen::Extracted;

/// How long a trail takes to fall by a factor of *e* at full: under half after
/// one frame at 60 Hz and below one step of 255 in about 0.12 s — a soft fade,
/// not a ghost. Halved from 0.04 on the owner's look (§19).
const TAU: f32 = 0.02;

/// The longest frame a decay is computed over. A hitch must not bake a frame
/// into the history for its whole length.
const LONGEST_FRAME: f32 = 0.05;

/// The switch that turns it off for a scripted run, read once.
const SWITCH: &str = "ORBS_AFTERGLOW";

/// Whether the environment permits an afterglow at all.
#[derive(Resource, Debug, Clone, Copy)]
pub(super) struct Permitted(bool);

impl Default for Permitted {
    fn default() -> Self {
        Self(std::env::var(SWITCH).as_deref() != Ok("0"))
    }
}

/// The GPU-side value. Field order must match `afterglow.wgsl`; padded to
/// sixteen bytes as `SightUniform` is.
#[derive(ShaderType, Debug, Clone, Copy, PartialEq)]
pub(crate) struct AfterglowUniform {
    /// What a lit pixel keeps of itself next frame, `0.0` for nothing.
    pub(crate) decay: f32,
    pub(crate) _pad1: f32,
    pub(crate) _pad2: f32,
    pub(crate) _pad3: f32,
}

impl AfterglowUniform {
    /// The decay for a frame `seconds` long at `share` of full strength, or
    /// none when it should not run.
    pub(crate) fn new(running: bool, share: f32, seconds: f32) -> Self {
        let decay = if running && share > 0.0 {
            (-seconds.min(LONGEST_FRAME) / (TAU * share)).exp()
        } else {
            0.0
        };
        Self {
            decay,
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
        }
    }

    /// Whether the pass would change anything. When not, it is skipped and
    /// its history forgotten — see `fullscreen::History`.
    pub(crate) fn wanted(self) -> bool {
        self.decay > 0.0
    }
}

/// Hand the frame's decay to the render world.
///
/// Off with the tube, off with motion, and scaled by the `glow` row: bloom is
/// the phosphor in space and this is the phosphor in time, one row for both.
pub(super) fn extract(
    tube: Extract<Query<&CrtSettings>>,
    picture: Extract<Option<Res<Picture>>>,
    stillness: Extract<Option<Res<crate::shell::Stillness>>>,
    permitted: Extract<Res<Permitted>>,
    time: Extract<Res<Time>>,
    mut commands: Commands,
) {
    let on = tube.iter().next().is_some_and(|tube| tube.on);
    let still = stillness
        .as_deref()
        .is_some_and(|stillness| stillness.still());
    let share = picture
        .as_deref()
        .copied()
        .unwrap_or(Picture::FULL)
        .of(Effect::Glow)
        .share();
    commands.insert_resource(Extracted(AfterglowUniform::new(
        permitted.0 && on && !still,
        share,
        time.delta_secs(),
    )));
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f32 = 1.0 / 60.0;

    #[test]
    fn a_trail_is_a_soft_fade_not_a_ghost() {
        // The first draft lingered at two thirds for a frame and for 0.22 s in
        // all, and read as ghosting. A fade starts under half and is gone in
        // about a tenth of a second — and still lasts long enough to be seen.
        let decay = AfterglowUniform::new(true, 1.0, FRAME).decay;
        assert!(decay <= 0.5, "a trail starts at {decay} of what it was");
        let mut lit = 1.0_f32;
        let mut seconds = 0.0;
        while lit >= 1.0 / 255.0 {
            lit *= decay;
            seconds += FRAME;
            assert!(seconds < 0.13, "a trail outlived a soft fade");
        }
        assert!(seconds > 0.05, "a trail of {seconds}s is no trail");
    }

    #[test]
    fn off_is_skipped_not_multiplied() {
        // Each of the three inputs alone stops it.
        assert!(!AfterglowUniform::new(false, 1.0, FRAME).wanted());
        assert!(!AfterglowUniform::new(true, 0.0, FRAME).wanted());
        assert!(AfterglowUniform::new(true, 0.5, FRAME).wanted());
    }

    #[test]
    fn soft_is_shorter_than_full() {
        let soft = AfterglowUniform::new(true, 0.5, FRAME).decay;
        let full = AfterglowUniform::new(true, 1.0, FRAME).decay;
        assert!(soft < full);
    }

    #[test]
    fn the_camera_still_allows_copying_its_main_texture() {
        // The history is a copy of the frame this pass drew, and a copy needs
        // `COPY_SRC` on the source. Bevy's default has it today; a bump that
        // drops it fails here rather than as a validation error at draw.
        use bevy::camera::CameraMainTextureUsages;
        use bevy::render::render_resource::TextureUsages;
        assert!(
            CameraMainTextureUsages::default()
                .0
                .contains(TextureUsages::COPY_SRC)
        );
    }

    #[test]
    fn a_hitch_does_not_bake_into_the_history() {
        // A two-second frame decays as a short one would, not to nothing in one
        // go and not as if time had stood still.
        let hitch = AfterglowUniform::new(true, 1.0, 2.0).decay;
        let longest = AfterglowUniform::new(true, 1.0, LONGEST_FRAME).decay;
        assert!((hitch - longest).abs() < f32::EPSILON);
    }
}
