//! Screenshots of a tour: the window as a PNG, and the frame beside it as text,
//! which is what can be checked exactly where the picture is judged.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use orbs_shell::tour::Scene;

use super::state::Tour;
use crate::render::Canvas;

/// The switch naming a directory to take screenshots into. With it the tour
/// ignores a person's keys and leaves once the last scene is over.
pub(crate) const SHOTS: &str = "ORBS_TOUR_SHOTS";

/// Where screenshots go, and which are still to take.
pub(crate) struct Shots {
    dir: PathBuf,
    /// Seconds into the scene still to shoot at, the soonest last.
    due: Vec<f32>,
    /// The scene's place in the playlist, from one, and its name and about.
    number: usize,
    name: String,
    about: String,
    /// Screenshots asked for, and those on disk.
    asked: usize,
    taken: usize,
    /// One line per screenshot on disk.
    index: Vec<String>,
    /// Every file name given out, so no shot writes over another.
    stems: BTreeSet<String>,
    finished: bool,
}

impl Shots {
    /// Screenshots into `dir`, made if it is missing.
    ///
    /// # Errors
    ///
    /// When `dir` cannot be made.
    pub(crate) fn in_dir(dir: &Path) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        Ok(Self {
            dir: dir.to_owned(),
            due: Vec::new(),
            number: 0,
            name: String::new(),
            about: String::new(),
            asked: 0,
            taken: 0,
            index: Vec::new(),
            stems: BTreeSet::new(),
            finished: false,
        })
    }

    /// Whether a shot is still to take in this scene.
    pub(super) const fn pending(&self) -> bool {
        !self.due.is_empty()
    }

    /// A file name for a shot at `clock`, never one already given.
    fn stem(&mut self, clock: f32) -> String {
        let base = format!("{:02}-{}-{clock:05.2}s", self.number, self.name);
        let mut stem = base.clone();
        let mut again = 1;
        while !self.stems.insert(stem.clone()) {
            again += 1;
            stem = format!("{base}-{again}");
        }
        stem
    }

    /// A scene begun: its own shots are due, any left from the last are not.
    pub(super) fn begin(&mut self, number: usize, scene: &Scene) {
        self.number = number;
        self.name.clone_from(&scene.name);
        self.about.clone_from(&scene.about);
        self.due.clear();
        self.after(0.0, &scene.shot_times());
    }

    /// Shots at `offsets` seconds after `clock`.
    pub(super) fn after(&mut self, clock: f32, offsets: &[f32]) {
        self.due.extend(offsets.iter().map(|offset| clock + offset));
        self.due.sort_by(|a, b| b.total_cmp(a));
    }
}

/// Run condition: the tour is taking screenshots.
pub(super) fn shooting(tour: Option<Res<Tour>>) -> bool {
    tour.is_some_and(|tour| tour.shots.is_some())
}

/// What a screenshot's line in the index says, carried to its capture.
#[derive(Component)]
pub(super) struct Shot(String);

/// Take a shot due by now, after the frame has been painted. One a frame:
/// Bevy keeps one screenshot of a window per frame and drops the rest, whose
/// capture would then never come.
pub(super) fn take(mut commands: Commands, mut tour: ResMut<Tour>, canvas: Option<Res<Canvas>>) {
    let clock = tour.timeline.clock();
    let Some(shots) = tour.shots.as_mut() else {
        return;
    };
    if shots.due.last().is_none_or(|at| *at > clock) {
        return;
    }
    shots.due.pop();
    let stem = shots.stem(clock);
    if let Some(canvas) = &canvas {
        let text = orbs_shell::describe(&canvas.frame);
        if let Err(error) = std::fs::write(shots.dir.join(format!("{stem}.txt")), text) {
            warn!("tour: the frame for {stem} was not written: {error}");
        }
    }
    let line = format!("{stem}.png  {}  {clock:.2}s  {}", shots.name, shots.about);
    commands
        .spawn((Screenshot::primary_window(), Shot(line)))
        .observe(save_to_disk(shots.dir.join(format!("{stem}.png"))))
        .observe(noted);
    shots.asked += 1;
}

/// A screenshot is on disk: its line in the index, and whether the picture was
/// one flat colour, which a window that never composited gives.
fn noted(captured: On<ScreenshotCaptured>, lines: Query<&Shot>, mut tour: ResMut<Tour>) {
    let line = lines
        .get(captured.entity)
        .map(|shot| shot.0.clone())
        .unwrap_or_default();
    let flat = captured.image.data.as_deref().is_some_and(is_flat);
    if let Some(shots) = tour.shots.as_mut() {
        shots
            .index
            .push(if flat { format!("{line}  BLANK") } else { line });
        shots.taken += 1;
    }
}

/// Whether every pixel is the first one, within a step of rounding.
fn is_flat(pixels: &[u8]) -> bool {
    let (pixels, _) = pixels.as_chunks::<4>();
    let Some(first) = pixels.first() else {
        return true;
    };
    pixels.iter().all(|pixel| {
        pixel
            .iter()
            .zip(first)
            .all(|(channel, start)| channel.abs_diff(*start) <= 2)
    })
}

/// Leave once the last scene is over and every screenshot is on disk, with the
/// index written.
pub(super) fn finish(mut tour: ResMut<Tour>, mut exit: MessageWriter<AppExit>) {
    let over = tour.is_last() && tour.is_over();
    let Some(shots) = tour.shots.as_mut() else {
        return;
    };
    if shots.finished || !over || shots.taken < shots.asked {
        return;
    }
    shots.finished = true;
    shots.index.sort();
    let mut index = shots.index.join("\n");
    index.push('\n');
    if let Err(error) = std::fs::write(shots.dir.join("index.txt"), index) {
        warn!("tour: the index was not written: {error}");
    }
    info!(
        "tour: {} screenshots in {}",
        shots.taken,
        shots.dir.display()
    );
    exit.write(AppExit::Success);
}
