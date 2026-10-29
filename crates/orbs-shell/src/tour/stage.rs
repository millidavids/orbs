//! A scene's tower, built and set up: what the window shows at its start and
//! what the dump prints of it.

use orbs_sim::Sim;
use orbs_sim::content::Length;

use super::catalogue::Catalogue;
use super::scene::{Scene, Start};
use crate::install::Install;

/// A scene's world, ready to show.
pub struct Staged {
    /// The tower.
    pub sim: Sim,
    /// The installation over it, for a practice tower.
    pub install: Option<Install>,
    /// Whether it is the threshold's scratch world, with no tower raised.
    pub threshold: bool,
    /// Whether the boot card plays over it first.
    pub boot: bool,
    /// The setup lines the world could not read, gathered before a resume
    /// restores the tower and its log starts empty.
    pub unread: Vec<String>,
}

/// `scene`'s tower, with its setup run.
///
/// Built directly rather than through `environment::fresh`, so no switch in
/// the environment outranks the scene. Setup lines are read with no reader:
/// they are exact commands, and a capture of one must not depend on weights.
#[must_use]
pub fn stage(scene: &Scene, wizard: Option<&str>) -> Staged {
    let seed = scene.seed.unwrap_or_else(crate::seed);
    let (mut sim, mut install) = match scene.tower {
        Start::Open | Start::Threshold => (Sim::new(seed), None),
        Start::Sealed => match scene.length() {
            None | Some(Length::Baseline) => (Sim::sealed(seed), None),
            Some(length) => (Sim::begun(seed, length), None),
        },
        Start::Install => crate::install::pose_for_dump(seed, wizard),
    };
    if install.is_none()
        && let Some(name) = wizard
    {
        sim.rename(name);
    }
    poured(&mut sim);

    crate::script::drive(&mut sim, &mut install, &scene.setup, None);
    // A scene's `edit` ends by saving and closing, which a test holds: an
    // editor left open here would be lost, since `stage` hands back a world.
    if let Some(edit) = &scene.edit
        && let Some(editor) = crate::script::open(&mut sim)
    {
        let _closed = crate::script::type_into(&mut sim, editor, edit, None);
    }
    if let Some(then) = &scene.then {
        // A save lands on the next tick, as in the dump.
        sim.step();
        crate::script::drive(&mut sim, &mut install, then, None);
    }
    let unread = sim
        .parse_log()
        .records()
        .iter()
        .filter(|record| scene.unread(record.outcome))
        .map(|record| record.input.clone())
        .collect();
    if let Some(away) = scene.resumed {
        sim = Sim::restored(&sim.snapshot());
        poured(&mut sim);
        sim.say_resumed(Some(away));
    }
    Staged {
        sim,
        install,
        threshold: scene.tower == Start::Threshold,
        boot: scene.boot,
        unread,
    }
}

/// `ORBS_CONTENT`'s prose, as a swap pours it into a new world.
fn poured(sim: &mut Sim) {
    for file in crate::prose::load(sim) {
        tracing::warn!(
            "{} is set but {file} did not load; drawing the built-in text",
            crate::prose::CONTENT_DIR
        );
    }
}

/// The first scene `ORBS_TOUR` names, staged for the dump to print; `None`
/// when no tour was asked for.
///
/// # Errors
///
/// Why there is no such scene, or no debug build to stage it in.
pub fn for_dump(wizard: Option<&str>) -> Result<Option<Staged>, String> {
    let Some(terms) = super::scene::requested() else {
        return Ok(None);
    };
    if cfg!(not(debug_assertions)) {
        return Err(NEEDS_DEBUG.to_owned());
    }
    let catalogue = Catalogue::chosen()?;
    let chosen = catalogue.select(&terms)?;
    Ok(chosen.first().map(|scene| stage(scene, wizard)))
}

/// What a release build says to `ORBS_TOUR`: setups use the debug words, which
/// only a debug build has.
pub const NEEDS_DEBUG: &str = "the tour needs a debug build: its scenes use the debug words";
