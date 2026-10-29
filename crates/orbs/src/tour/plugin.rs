//! Registration for the tour.

use std::path::PathBuf;

use bevy::input::InputSystems;
use bevy::prelude::*;
use bevy::time::TimeSystems;
use orbs_shell::tour::Scene;

use super::begin::{begin, resume};
use super::shots::{self, Shots};
use super::state::{Tour, advance, scene_due};
use super::{inject, yours};

/// Plays a list of scenes in the window, one after another.
pub struct TourPlugin {
    scenes: Vec<Scene>,
    shots: Option<PathBuf>,
    brisk: bool,
}

impl TourPlugin {
    /// `scenes`, with screenshots into `shots` if it names a directory.
    pub(crate) const fn new(scenes: Vec<Scene>, shots: Option<PathBuf>) -> Self {
        Self {
            scenes,
            shots,
            brisk: false,
        }
    }

    /// Each scene over once its hold and steps are, with no rest after: the
    /// tests'.
    #[cfg(test)]
    pub(crate) const fn brisk(scenes: Vec<Scene>) -> Self {
        Self {
            scenes,
            shots: None,
            brisk: true,
        }
    }
}

impl Plugin for TourPlugin {
    fn build(&self, app: &mut App) {
        let shots = self
            .shots
            .as_deref()
            .and_then(|dir| match Shots::in_dir(dir) {
                Ok(shots) => Some(shots),
                Err(error) => {
                    error!(
                        "tour: no screenshots, {} could not be made: {error}",
                        dir.display()
                    );
                    None
                }
            });
        app.insert_resource(Tour::new(self.scenes.clone(), shots, self.brisk))
            // After the clock, so `begin` can stop it paying for the staging,
            // and `resume` start it once the frame that paid has been seen.
            .add_systems(
                First,
                (yours::watch, resume, begin.run_if(scene_due))
                    .chain()
                    .after(TimeSystems)
                    .run_if(resource_exists::<Tour>),
            )
            // Before `InputSystems`, so a pressed key is seen this frame.
            .add_systems(
                PreUpdate,
                inject::press
                    .before(InputSystems)
                    .run_if(resource_exists::<Tour>),
            )
            .add_systems(Update, advance.run_if(resource_exists::<Tour>))
            // After `Update`'s paint, so the text is the frame on screen.
            .add_systems(
                PostUpdate,
                (shots::take, shots::finish).chain().run_if(shots::shooting),
            );
    }
}
