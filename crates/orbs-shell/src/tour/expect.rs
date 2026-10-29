//! What a scene says it reaches, checked against the world it reached.
//!
//! A world that turns a command down says so in prose, with no mark a check
//! could read, so a setup that parsed and did nothing is caught here or
//! nowhere.

use std::collections::BTreeMap;

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;
use orbs_sim::tower::State;
use serde::Deserialize;

/// What must hold.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expect {
    /// The room the player stands in.
    #[serde(default)]
    pub room: Option<String>,
    /// Instruments here, by name, in a state their picture moves in.
    #[serde(default)]
    pub working: Vec<String>,
    /// Instruments here, by name, each in the state named: `{ alembic =
    /// "charged" }`. For a state that does not move, which `working` cannot
    /// claim.
    #[serde(default)]
    pub states: BTreeMap<String, String>,
    /// What has the screen.
    #[serde(default)]
    pub surface: Option<Surface>,
    /// Words the orb has said.
    #[serde(default)]
    pub said: Option<String>,
}

/// A surface a scene can claim is up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Surface {
    /// The spell editor.
    Editor,
    /// The weave screen.
    Weave,
    /// The archive's maze, with the arrows.
    Maze,
    /// The orb's menu.
    Menu,
    /// The manual.
    Manual,
    /// The installation's strip.
    Install,
    /// The threshold, before any tower.
    Threshold,
}

/// Whether an instrument in `state` is one whose picture moves.
#[must_use]
pub const fn moving(state: State) -> bool {
    matches!(state, State::Working | State::Scouring | State::Burning)
}

impl Expect {
    /// What of this falls short in `sim`, one sentence each; empty when it
    /// all holds. The surface is the caller's, since each frontend holds its
    /// own.
    #[must_use]
    pub fn shortfall(&self, sim: &Sim) -> Vec<String> {
        let mut short = Vec::new();
        if let Some(room) = &self.room {
            let here = sim.domain();
            if here.as_deref() != Some(room.as_str()) {
                short.push(format!("stands in {here:?}, not {room}"));
            }
        }
        let instruments = sim.instruments();
        let state_of = |name: &str| {
            instruments
                .iter()
                .find(|instrument| instrument.name == name)
                .map(|instrument| instrument.state)
        };
        for name in &self.working {
            match state_of(name) {
                Some(state) if moving(state) => {}
                Some(state) => short.push(format!("the {name} is {}, not working", state.label())),
                None => short.push(format!("there is no {name} here")),
            }
        }
        for (name, want) in &self.states {
            match state_of(name) {
                Some(state) if state.label() == want => {}
                Some(state) => short.push(format!("the {name} is {}, not {want}", state.label())),
                None => short.push(format!("there is no {name} here")),
            }
        }
        if let Some(words) = &self.said
            && !said(sim, words)
        {
            short.push(format!("the orb never said {words:?}"));
        }
        short
    }
}

/// Whether anything the orb has said contains `words`.
fn said(sim: &Sim, words: &str) -> bool {
    sim.scrollback().records().iter().any(|record| {
        matches!(record.field(FieldName::Message), Some(Value::Text(text)) if text.contains(words))
    })
}
