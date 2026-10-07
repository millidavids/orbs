//! What the tower has done, in all, and how much of it a spell did (§19,
//! *number go up*).
//!
//! Read from the tally, never kept: a ledger row is a tally key with a label,
//! so it needs nothing saved and cannot disagree with a mastery station that
//! counts the same thing.

use bevy_ecs::prelude::*;

use super::tally::{Tally, by_spell};
use crate::content::Progression;

/// One lifetime count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The row's id; `ledger_<id>` is its label.
    pub id: String,
    /// The room that shows it, if one does.
    pub domain: Option<String>,
    /// How many, in all.
    pub count: u32,
    /// How many of those a spell did.
    pub by_spell: u32,
}

/// Every row `progression.toml` authors, in its order, counted.
#[must_use]
pub fn ledger(world: &World) -> Vec<Row> {
    let tally = world.resource::<Tally>();
    world
        .resource::<Progression>()
        .ledger()
        .iter()
        .map(|entry| counted(entry, tally))
        .collect()
}

/// The first row `room` keeps — what its column shows — counted alone.
#[must_use]
pub fn ledger_of(world: &World, room: &str) -> Option<Row> {
    let tally = world.resource::<Tally>();
    world
        .resource::<Progression>()
        .ledger()
        .iter()
        .find(|entry| entry.domain.as_deref() == Some(room))
        .map(|entry| counted(entry, tally))
}

fn counted(entry: &crate::content::LedgerEntry, tally: &Tally) -> Row {
    let key = entry.counts.key();
    Row {
        id: entry.id.clone(),
        domain: entry.domain.clone(),
        count: tally.count(&key),
        by_spell: tally.count(&by_spell(&key)),
    }
}
