//! What `ORBS_TOUR` asks of the binary before there is an `App`.

use std::path::PathBuf;

use bevy::app::AppExit;
use orbs_shell::tour::{Catalogue, NEEDS_DEBUG, requested};

use super::plugin::TourPlugin;
use super::shots::SHOTS;

/// Switches that would outrank a scene, named when they are set.
const OUTRANKING: [&str; 8] = [
    "ORBS_PASSAGE",
    "ORBS_STRAIN",
    "ORBS_CRT",
    "ORBS_SIGHT",
    "ORBS_MOTION",
    "ORBS_AFTERGLOW",
    "ORBS_FIRE",
    "ORBS_SEED",
];

/// What the tour asks of `main`.
pub(crate) enum Ask {
    /// Nothing: there is no tour.
    Nothing,
    /// End here: a listing, a refusal, or a name that is no scene.
    Exit(AppExit),
    /// Play these scenes.
    Play(TourPlugin),
}

/// What `ORBS_TOUR` asks for, decided before there is any window.
pub(crate) fn before_the_app() -> Ask {
    let Some(terms) = requested() else {
        return Ask::Nothing;
    };
    if cfg!(not(debug_assertions)) {
        eprintln!("{NEEDS_DEBUG}");
        return Ask::Exit(AppExit::error());
    }
    let catalogue = match Catalogue::chosen() {
        Ok(catalogue) => catalogue,
        Err(why) => {
            eprintln!("{why}");
            return Ask::Exit(AppExit::error());
        }
    };
    if terms == "list" {
        print!("{}", catalogue.listing());
        return Ask::Exit(AppExit::Success);
    }
    let scenes = match catalogue.select(&terms) {
        Ok(scenes) => scenes.into_iter().cloned().collect(),
        Err(why) => {
            eprintln!("{why}");
            return Ask::Exit(AppExit::error());
        }
    };
    for switch in OUTRANKING {
        if std::env::var_os(switch).is_some() {
            eprintln!("tour: {switch} is set, and outranks what the scenes ask for");
        }
    }
    let shots = std::env::var_os(SHOTS).map(PathBuf::from);
    Ask::Play(TourPlugin::new(scenes, shots))
}
