//! The satchel: a queue of names, in the tower rather than in a spell.
//!
//! §8's spells could not tell each other anything. Two of them run at once
//! already — `invoke` from inside a spell inserts a second `Running` and
//! `run::advance` steps every one of them each tick, each with its own budget —
//! but they shared no channel, so the concurrency was two solitary loops rather
//! than a pipeline. This is the channel.
//!
//! # Why a node, and not a field on `Running`
//!
//! **State lives in the tower in this game.** A queue held inside a spell would
//! be reachable by exactly one spell, invisible to `survey`, and absent from
//! every instrument the project uses to look at itself. A node is reachable by
//! two spells *and* by two cursors of one spell, it saves through the same path
//! every other node does, and `survey satchel` shows what is waiting — which is
//! more than can be said for any other part of a running spell.
//!
//! # A component, not children
//!
//! Every other counted thing in the tower is children plus [`Stock`], and this
//! deliberately is not. A queue is an **ordered multiset**: `skyward` may be
//! queued twice and the order is the entire point, and `Stock` collapses
//! duplicates into a count and despawns at nought. So the names ride a
//! `VecDeque` on the node, and `spell::watch` answers `is empty` from here
//! rather than from `children_of`.
//!
//! [`Stock`]: super::Stock

use bevy_ecs::prelude::*;
use std::collections::VecDeque;

/// What the satchel is called, in every room that has one.
///
/// One word for all of them, because a spell is written for a domain and the
/// satchel it means is the one where it stands — the same rule `<room>.log`
/// follows from the other side, where the name changes and the meaning does not.
pub const SATCHEL: &str = "satchel";

/// The heading `survey satchel` puts over the queue.
///
/// A **table entry, not prose** — the same exemption `NounKind::label` has, and
/// for the same reason: every other section heading in a `survey` is a noun kind
/// read off that const table, and this is the one section whose rows are not
/// nodes. Keeping it beside them means the listing has one vocabulary.
pub const QUEUED: &str = "queued";

/// The most names one satchel will hold.
///
/// **A cap rather than a refusal to grow**, and it is a runaway guard rather
/// than a balance number: a producer that queues faster than its consumer pulls
/// is the ordinary shape of a pipeline falling behind, and without a bound it
/// grows the *save* by a line a tick until nothing can read it. `MAX_PARTS`
/// makes the same argument for a descent, and a queued name is cheaper than a
/// descent but arrives faster.
///
/// Sixteen is well past any pipeline the game has: the menagerie's whole figure
/// is twelve syllables, so a producer that ran the entire chart ahead of its
/// consumer would still fit.
pub const DEPTH: usize = 16;

/// An ordered queue of names, on the node that is the satchel.
///
/// Oldest first: [`take`](Satchel::take) pops the front and
/// [`put`](Satchel::put) pushes the back, which is what makes it a queue rather
/// than a pile. A pile is what the rest of the tower already has.
#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct Satchel(VecDeque<String>);

impl Satchel {
    /// Put a name on the back.
    ///
    /// **Refuses at [`DEPTH`] rather than dropping the oldest**, and the
    /// direction matters: a queue that silently forgot its front would hand the
    /// consumer a syllable out of order, which looks exactly like a solver with
    /// a timing bug. Refusing is visible — the caller says so, and the pipeline
    /// stalls where it actually went wrong.
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
    /// **This is what `if the satchel is empty` reads**, and it is the reason
    /// the queue is a component rather than children: `spell::watch` answers
    /// emptiness with `children_of(...).is_empty()`, and a satchel has none
    /// either way.
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

    /// Read one back, **clamped to [`DEPTH`]**.
    ///
    /// A hand-edited save is a supported thing to have (§15), so the cap is
    /// applied on the way in as well as on the way out — otherwise the one
    /// guard against an unbounded queue is one a text editor can step over.
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
        for name in ["skyward", "earthward", "skyward"] {
            assert!(satchel.put(name));
        }
        // **A name twice, in the order it was put.** This is the whole reason
        // the queue is not children plus `Stock`: a count would say `skyward 2`
        // and lose which one comes first.
        assert_eq!(satchel.len(), 3);
        assert_eq!(satchel.peek(), Some("skyward"));
        assert_eq!(satchel.take().as_deref(), Some("skyward"));
        assert_eq!(satchel.take().as_deref(), Some("earthward"));
        assert_eq!(satchel.take().as_deref(), Some("skyward"));
        assert_eq!(satchel.take(), None);
        assert!(satchel.is_empty());
    }

    /// **Refused at the cap, and the front is kept.** Dropping the oldest would
    /// hand the consumer a name out of order, which is indistinguishable from a
    /// solver whose timing is wrong.
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
