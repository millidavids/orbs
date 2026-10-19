//! Which supplies are open, and how far each has drawn (§19, *Supply*).

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;

use crate::tower;

/// Where a supply draws: into a siege's rounds, or over the calm layer's ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// A draw a round, on the wall.
    Siege,
    /// A draw every few hundred ticks, in a room.
    Calm,
}

impl Scope {
    /// The word a save writes.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Siege => "siege",
            Self::Calm => "calm",
        }
    }

    /// The scope a save's word names.
    #[must_use]
    pub fn named(word: &str) -> Option<Self> {
        [Self::Siege, Self::Calm]
            .into_iter()
            .find(|scope| scope.word() == word)
    }
}

/// One open supply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Open {
    /// Where it draws.
    pub scope: Scope,
    /// The rate the player chose — one word raises it one.
    pub chosen: usize,
    /// The rate the last draw ran at, which draining can step below `chosen`.
    pub running: usize,
    /// The tick it opened.
    pub since: u64,
    /// How many the current draw took. Nought when it was skipped.
    pub drawn: u32,
    /// What the current draw applied, so a raise mid-turn adds the difference.
    pub applied: u32,
}

/// Every open supply, by item. One an item, across the tower.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Supplies {
    open: BTreeMap<String, Open>,
    /// Supplies stopped since their last draw, with what it took: on the wall
    /// until the round ends, in a room until the next draw would have come.
    /// Opened again before then, one starts from that rather than drawing and
    /// applying a whole ration twice.
    stopped: BTreeMap<String, Open>,
}

impl Supplies {
    /// The supply open for `name`, if one is.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Open> {
        self.open.get(name)
    }

    /// The same, to change.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Open> {
        self.open.get_mut(name)
    }

    /// Open or replace the supply for `name`.
    pub fn open(&mut self, name: &str, open: Open) {
        self.open.insert(name.to_owned(), open);
    }

    /// Shut the supply for `name`, and say what it was. One that has drawn is
    /// remembered until its next draw would have come.
    pub fn close(&mut self, name: &str) -> Option<Open> {
        let open = self.open.remove(name)?;
        if open.drawn > 0 {
            self.stopped.insert(name.to_owned(), open);
        }
        Some(open)
    }

    /// What `name` drew before it was stopped, if it was stopped since.
    #[must_use]
    pub fn stopped(&self, name: &str) -> Option<&Open> {
        self.stopped.get(name)
    }

    /// Forget what was stopped and take it back, as `name` opens again or its
    /// draw comes round.
    pub fn reopen(&mut self, name: &str) -> Option<Open> {
        self.stopped.remove(name)
    }

    /// A new round: nothing on the wall has drawn for it, stopped or open. A
    /// supply that then runs dry is shut with nothing drawn, so it is not
    /// remembered as stopped and cannot reopen on last round's ration.
    pub fn new_round(&mut self) {
        self.stopped.retain(|_, open| open.scope != Scope::Siege);
        for open in self.open.values_mut() {
            if open.scope == Scope::Siege {
                open.drawn = 0;
                open.applied = 0;
            }
        }
    }

    /// A new draw for `name`'s calm supply: nothing taken for it yet.
    pub fn new_period(&mut self, name: &str) {
        if let Some(open) = self.open.get_mut(name) {
            open.drawn = 0;
            open.applied = 0;
        }
    }

    /// Shut every supply drawing in `scope`.
    pub fn close_all(&mut self, scope: Scope) {
        self.open.retain(|_, open| open.scope != scope);
        self.stopped.retain(|_, open| open.scope != scope);
    }

    /// Every open supply, by name.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Open)> {
        self.open.iter().map(|(name, open)| (name.as_str(), open))
    }

    /// Every supply stopped this round, by name, for a save.
    pub fn iter_stopped(&self) -> impl Iterator<Item = (&str, &Open)> {
        self.stopped
            .iter()
            .map(|(name, open)| (name.as_str(), open))
    }

    /// The names drawing in `scope`, alphabetically.
    #[must_use]
    pub fn names(&self, scope: Scope) -> Vec<String> {
        self.open
            .iter()
            .filter(|(_, open)| open.scope == scope)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// The names stopped in `scope` since their last draw, alphabetically.
    #[must_use]
    pub fn stopped_names(&self, scope: Scope) -> Vec<String> {
        self.stopped
            .iter()
            .filter(|(_, open)| open.scope == scope)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Put supplies back, for a save.
    pub(crate) fn restore(
        &mut self,
        open: impl IntoIterator<Item = (String, Open)>,
        stopped: impl IntoIterator<Item = (String, Open)>,
    ) {
        self.open = open.into_iter().collect();
        self.stopped = stopped.into_iter().collect();
    }
}

/// How many of `name` the arsenal holds. Supplies draw from there and nowhere
/// else, the one room reachable from every other.
#[must_use]
pub fn held_in_arsenal(world: &World, name: &str) -> u32 {
    tower::keep(world)
        .and_then(|arsenal| tower::held(world, arsenal, name))
        .map_or(0, |stock| match stock {
            tower::Stock::Counted(count) => count,
            tower::Stock::Endless => u32::MAX,
        })
}
