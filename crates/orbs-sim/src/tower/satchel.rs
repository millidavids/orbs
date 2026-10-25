//! The satchel: a queue of names, in the tower rather than in a spell.
//!
//! §8's channel between two spells. A node, so two spells or two cursors reach
//! it and `survey` shows it. A component, not children with [`Stock`], because
//! a queue is an ordered multiset and `Stock` collapses duplicates.
//!
//! [`Stock`]: super::Stock

use bevy_ecs::prelude::*;
use std::collections::VecDeque;

/// What the satchel is called, in every room that has one.
///
/// One word for all of them: a spell is written for a domain and means the
/// satchel where it stands — the same rule `<room>.log` follows from the other
/// side.
pub const SATCHEL: &str = "satchel";

/// The heading `survey satchel` puts over the queue.
///
/// A table entry, not prose — the same exemption `NounKind::label` has: every
/// other heading in a `survey` is a noun kind read off that const table, and
/// this is the one section whose rows are not nodes.
pub const QUEUED: &str = "queued";

/// The most names one satchel will hold.
///
/// A runaway guard, `MAX_PARTS`'s argument: a producer outrunning its consumer
/// grows the save a line a tick. Well past any pipeline the game has.
pub const DEPTH: usize = 16;

/// An ordered queue of names, on the node that is the satchel.
///
/// Oldest first: [`take`](Satchel::take) pops the front and
/// [`put`](Satchel::put) pushes the back, which makes it a queue rather than
/// the pile the rest of the tower has.
#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct Satchel(VecDeque<String>);

impl Satchel {
    /// Put a name on the back.
    ///
    /// Refuses at [`DEPTH`] rather than dropping the oldest, which would look
    /// like a solver with a timing bug.
    pub fn put(&mut self, name: &str) -> bool {
        if self.0.len() >= DEPTH {
            return false;
        }
        self.0.push_back(name.to_owned());
        true
    }

    /// Take the name off the front, if there is one.
    pub fn take(&mut self) -> Option<String> {
        self.0.pop_front()
    }

    /// What is on the front, without taking it.
    #[must_use]
    pub fn peek(&self) -> Option<&str> {
        self.0.front().map(String::as_str)
    }

    /// Whether nothing is waiting.
    ///
    /// What `if the satchel is empty` reads, since a satchel has no children.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// How many names are waiting.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Every name, oldest first — what `survey satchel` draws.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }

    /// The queue as a save writes it, oldest first.
    #[must_use]
    pub fn to_save(&self) -> Vec<String> {
        self.0.iter().cloned().collect()
    }

    /// Read one back, clamped to [`DEPTH`].
    ///
    /// A hand-edited save is supported (§15), so the cap applies on the way in
    /// too — otherwise a text editor steps over the one guard against an
    /// unbounded queue.
    #[must_use]
    pub fn from_save(names: &[String]) -> Self {
        Self(names.iter().take(DEPTH).cloned().collect())
    }
}

/// The satchel where the player is standing, if that room has one.
#[must_use]
pub fn fixture(world: &World, room: Entity) -> Option<Entity> {
    super::children_of(world, room)
        .into_iter()
        .find(|node| world.get::<Satchel>(*node).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_is_a_queue_and_not_a_pile() {
        let mut satchel = Satchel::default();
        assert!(satchel.is_empty());
        for name in ["heed", "yoke", "heed"] {
            assert!(satchel.put(name));
        }
        // A name twice, in the order it was put — a count would say `heed 2`
        // and lose which one comes first.
        assert_eq!(satchel.len(), 3);
        assert_eq!(satchel.peek(), Some("heed"));
        assert_eq!(satchel.take().as_deref(), Some("heed"));
        assert_eq!(satchel.take().as_deref(), Some("yoke"));
        assert_eq!(satchel.take().as_deref(), Some("heed"));
        assert_eq!(satchel.take(), None);
        assert!(satchel.is_empty());
    }

    /// Refused at the cap, keeping the front: dropping the oldest hands the
    /// consumer a name out of order, which looks like a solver with bad timing.
    #[test]
    fn a_full_satchel_refuses_rather_than_forgetting_its_front() {
        let mut satchel = Satchel::default();
        for i in 0..DEPTH {
            assert!(satchel.put(&format!("n{i}")), "refused below the cap");
        }
        assert!(!satchel.put("one-too-many"));
        assert_eq!(satchel.len(), DEPTH);
        assert_eq!(satchel.peek(), Some("n0"), "the front was forgotten");
    }

    #[test]
    fn a_satchel_survives_a_save_in_order() {
        let mut satchel = Satchel::default();
        for name in ["leftward", "rightward", "leftward"] {
            satchel.put(name);
        }
        assert_eq!(Satchel::from_save(&satchel.to_save()), satchel);
    }

    /// A hand-edited save is supported, so the cap holds on the way in too.
    #[test]
    fn a_save_cannot_smuggle_a_longer_queue_in() {
        let long: Vec<String> = (0..DEPTH * 2).map(|i| format!("n{i}")).collect();
        assert_eq!(Satchel::from_save(&long).len(), DEPTH);
    }
}
