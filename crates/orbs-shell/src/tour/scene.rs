//! One scene of the tour, as data: a tower, its setup, and what is then typed.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::expect::Expect;
use super::steps::Step;

/// The switch that asks for a tour: scene names, groups, `all`, or `list`.
pub const TOUR: &str = "ORBS_TOUR";

/// A scene lasts at least this long, in seconds, unless it says otherwise.
const HOLD: f32 = 5.0;

/// What `ORBS_TOUR` asked for, if anything.
#[must_use]
pub fn requested() -> Option<String> {
    crate::environment::switched(TOUR)
}

/// Which tower a scene starts from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Start {
    /// `Sim::new`: every room open, as every dump draws it.
    #[default]
    Open,
    /// A sealed tower, as a player's begins.
    Sealed,
    /// The practice tower, installing.
    Install,
    /// No tower yet: the orb's own menu.
    Threshold,
}

/// One scene: a tower, what is done to it at once, and what is then typed.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    /// `group.name`; a group selects every scene under it.
    pub name: String,
    /// What it shows, for the listing, the title and the index.
    pub about: String,
    /// Commands run at once, before the scene is shown.
    #[serde(default)]
    pub setup: String,
    /// The tower it starts from.
    #[serde(default)]
    pub tower: Start,
    /// A sealed tower's length, by name; the authored curve otherwise.
    #[serde(default)]
    pub length: Option<String>,
    /// The world's seed; the instrument's otherwise.
    #[serde(default)]
    pub seed: Option<u64>,
    /// Whether the boot card plays first.
    #[serde(default)]
    pub boot: bool,
    /// Seconds away: the tower is reopened from its own save and says so.
    #[serde(default)]
    pub resumed: Option<u64>,
    /// Keystrokes for the editor a setup `scribe` opened, as `ORBS_EDIT`.
    #[serde(default)]
    pub edit: Option<String>,
    /// Commands run after `edit`, as `ORBS_THEN`.
    #[serde(default)]
    pub then: Option<String>,
    /// Settings rows, by key, set for this scene only.
    #[serde(default)]
    pub settings: BTreeMap<String, String>,
    /// The glass's strain held at `[pressure, fault]`.
    #[serde(default)]
    pub strain: Option<[f32; 2]>,
    /// The scene shows a numbered question on purpose, so a line read as
    /// ambiguous is what it is for rather than a setup gone wrong.
    #[serde(default)]
    pub asks: bool,
    /// The least it lasts, in seconds.
    #[serde(default = "hold")]
    pub hold: f32,
    /// Seconds from its start to take a screenshot at.
    #[serde(default)]
    pub shots: Vec<f32>,
    /// What is typed once it is shown, in order.
    #[serde(default)]
    pub steps: Vec<Step>,
    /// What setup must reach.
    #[serde(default)]
    pub expect: Expect,
    /// What the steps must reach.
    #[serde(default)]
    pub ends: Expect,
}

const fn hold() -> f32 {
    HOLD
}

impl Scene {
    /// Whether a line read as `outcome` means the scene went wrong: anything
    /// unread, and an ambiguous line unless the scene [`asks`](Self::asks).
    #[must_use]
    pub const fn unread(&self, outcome: orbs_sim::parser::Outcome) -> bool {
        use orbs_sim::parser::Outcome;
        match outcome {
            Outcome::Ambiguous => !self.asks,
            Outcome::Incomplete | Outcome::Unresolved => true,
            _ => false,
        }
    }

    /// The tower's length, if it names one this build knows.
    #[must_use]
    pub fn length(&self) -> Option<orbs_sim::content::Length> {
        self.length
            .as_deref()
            .and_then(orbs_sim::content::Length::named)
    }

    /// Whether `term` selects it: its name, or a group above it.
    pub(super) fn is(&self, term: &str) -> bool {
        self.name == term
            || self
                .name
                .strip_prefix(term)
                .is_some_and(|rest| rest.starts_with('.'))
    }

    /// Why the window could not play it, if it could not: a step with nothing
    /// to press would hold it for good, and a level that is no number would
    /// reach the tube. An edited catalogue is read here too.
    pub(super) fn playable(&self) -> Result<(), String> {
        let name = &self.name;
        let seconds = |value: f32| (0.0..f32::INFINITY).contains(&value);
        let held = self.hold > 0.0 && self.hold.is_finite();
        if !held {
            return Err(format!("{name} holds {}s", self.hold));
        }
        if self
            .strain
            .is_some_and(|levels| levels.iter().any(|level| !(0.0..=1.0).contains(level)))
        {
            return Err(format!("{name} strains past the glass's range"));
        }
        if !self.shots.iter().copied().all(seconds) {
            return Err(format!("{name} shoots at a time there is none"));
        }
        for step in &self.steps {
            match step.strokes() {
                Ok(strokes) if strokes.is_empty() => {
                    return Err(format!("{name}: a step presses nothing"));
                }
                Err(why) => return Err(format!("{name}: {why}")),
                Ok(_) => {}
            }
            if !step.pause.is_none_or(seconds) || !step.shots.iter().copied().all(seconds) {
                return Err(format!("{name}: a step waits a time there is none"));
            }
        }
        Ok(())
    }

    /// When to take its screenshots, in seconds from its start: its own, or
    /// three across it when it has no steps to take them for it.
    #[must_use]
    pub fn shot_times(&self) -> Vec<f32> {
        if !self.shots.is_empty() || !self.steps.is_empty() {
            return self.shots.clone();
        }
        vec![1.0, self.hold / 2.0, self.hold - 0.5]
    }
}
