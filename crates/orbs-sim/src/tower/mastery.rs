//! The two tracks, read (DESIGN.md §11.5, §19).
//!
//! # Two shapes, one currency
//!
//! The **Ley Line** is a straight path: predefined steps in order, and passing
//! one *is* the grant. There is no choice in it and nothing to store — what a
//! player has is a function of [`Experience`](super::Experience) against the
//! authored list, the same way `concentration` already was.
//!
//! **Mastery** branches. A tier opens when the total passes it and gives exactly
//! **one** of its nodes, so the tier opening is the grant and which node is taken
//! is the decision. That is what lets experience stay unspendable (§11.5's
//! *"accumulates and is never spent"*) while still offering a choice — the
//! ROADMAP's own acceptance line for the upgrade tree.
//!
//! # Nothing here takes anything, yet
//!
//! Every authored node is a marker. There is no mutator in this module and no
//! `Submission` variant behind it, because a screen that cannot change the world
//! needs neither — and adding a mutator with nothing to grant would be building
//! the replay leg, the confirm and the irreversibility before anything could
//! exercise them. `take` lands with the first real node.
//!
//! What *is* here is the reading, because that is what the screen draws and what
//! the first real node will be checked against.

use bevy_ecs::prelude::*;

use crate::content::Progression;

use super::Experience;

/// Which mastery nodes the orb has taken.
///
/// **A list of ids, not a set of flags.** An id is what the content file
/// authors, what a sentence is keyed by, and what a save would carry; a
/// `bool` per node would have to be regenerated every time the tree grew.
/// `Progression::check` refuses a duplicate id, so a name here means one node.
///
/// Empty and stays empty in this version — nothing can be taken yet.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Taken(Vec<String>);

impl Taken {
    /// Put a set of taken nodes back, for a save.
    pub(crate) fn restore(&mut self, ids: Vec<String>) {
        self.0 = ids;
    }

    /// The ids taken, in the order they were taken.
    #[must_use]
    pub fn ids(&self) -> &[String] {
        &self.0
    }
}

/// What a node or a step is, right now.
///
/// **Three states, and the screen must draw them apart without colour** (§14).
/// The glyph carries it for anyone who can see it and `FieldName::State` carries
/// the word for anyone who cannot — a reader hearing "`○` concentration 1" has
/// been told nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Held. A ley-line step that has been passed, or a node that was chosen.
    Taken,
    /// Reachable now: the tier is open and its choice is unspent.
    Open,
    /// Not yet. Either the total is short, or the tier already spent its choice.
    Locked,
}

impl Standing {
    /// The word a screen reader hears.
    ///
    /// Not prose — one word naming a state, which §19 settles is the parser's
    /// kind of fact rather than a sentence. The *sentence* about a node lives in
    /// `prose.toml` keyed by its id.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Taken => "taken",
            Self::Open => "open",
            Self::Locked => "locked",
        }
    }

    /// Whether what this node grants is in effect right now.
    ///
    /// **A different question from whether it is reachable**, and the details
    /// panel asks both: an open Mastery node is unlocked and doing nothing, and
    /// a node whose sibling took the tier's one choice is unlocked and will
    /// never do anything.
    #[must_use]
    pub const fn active(self) -> bool {
        matches!(self, Self::Taken)
    }
}

/// One row of either track, as the screen draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The id, which is the content file's word and the prose key.
    pub id: String,
    /// The total that opens it.
    pub at: u64,
    /// Where it stands against the tower's experience.
    pub standing: Standing,
    /// Whether the tower has earned enough to reach it at all.
    ///
    /// **Not derivable from [`standing`](Self::standing)**, which is why it is a
    /// field. `Locked` covers two different situations — a total not yet reached,
    /// and a tier whose one choice a sibling already took — and they are the same
    /// glyph on purpose, because in both cases the node cannot be had. They are
    /// *not* the same sentence: one is *work more* and the other is *you chose
    /// otherwise*, and a details panel that could not tell them apart would be
    /// telling the player to go and earn something they have already earned.
    pub unlocked: bool,
}

/// The Ley Line, in order, against what the tower has earned.
///
/// Every step is `Taken` or `Locked` and never `Open`: passing one *is* taking
/// it, so there is no moment where a step is reachable and unheld.
#[must_use]
pub fn ley_line(world: &World) -> Vec<Node> {
    let earned = world.resource::<Experience>().get();
    world
        .resource::<Progression>()
        .ley_line()
        .iter()
        .map(|step| Node {
            id: step.grants.clone(),
            at: step.at,
            standing: if earned >= step.at {
                Standing::Taken
            } else {
                Standing::Locked
            },
            unlocked: earned >= step.at,
        })
        .collect()
}

/// Mastery's tiers, in order, each with its nodes.
///
/// A tier whose total has not been reached holds only `Locked` nodes. A tier
/// that is open holds `Open` nodes until one is taken, after which that one is
/// `Taken` and **its siblings are `Locked`** — the choice is spent, and the
/// screen has to show that it was spent rather than that the others were never
/// there.
#[must_use]
pub fn mastery(world: &World) -> Vec<Vec<Node>> {
    let earned = world.resource::<Experience>().get();
    let taken = world.resource::<Taken>();
    world
        .resource::<Progression>()
        .mastery()
        .iter()
        .map(|tier| {
            let spent = tier
                .nodes
                .iter()
                .any(|node| taken.ids().iter().any(|held| held == node));
            tier.nodes
                .iter()
                .map(|node| Node {
                    id: node.clone(),
                    at: tier.at,
                    standing: standing_of(node, tier.at, earned, spent, taken),
                    unlocked: earned >= tier.at,
                })
                .collect()
        })
        .collect()
}

/// Where one mastery node stands.
fn standing_of(node: &str, at: u64, earned: u64, spent: bool, taken: &Taken) -> Standing {
    if taken.ids().iter().any(|held| held == node) {
        Standing::Taken
    } else if earned >= at && !spent {
        Standing::Open
    } else {
        Standing::Locked
    }
}

/// The next total that opens anything, and what it is.
///
/// **The "what's next" line, and the whole reason a locked tier is drawn at
/// all.** A screen that showed only what you have is a receipt; §11.5 wants the
/// player to know what the work is *for*. `None` means nothing more is authored
/// yet, which is not the same as being finished.
#[must_use]
pub fn next(world: &World) -> Option<u64> {
    let earned = world.resource::<Experience>().get();
    let curve = world.resource::<Progression>();
    let steps = curve.ley_line().iter().map(|step| step.at);
    let tiers = curve.mastery().iter().map(|tier| tier.at);
    steps.chain(tiers).filter(|at| *at > earned).min()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    /// A tower that has earned `total`, by the only door that grants any.
    fn earned(total: u64) -> Sim {
        let mut sim = Sim::new(1);
        super::super::credit(sim.world_mut(), total);
        sim
    }

    #[test]
    fn a_cold_tower_has_taken_nothing_and_is_told_what_is_next() {
        let sim = Sim::new(1);
        let line = ley_line(sim.world());
        assert_eq!(line.len(), 1);
        assert_eq!(line[0].standing, Standing::Locked);
        assert_eq!(line[0].at, 16);
        assert_eq!(next(sim.world()), Some(16), "nothing to work toward");
    }

    #[test]
    fn passing_a_step_is_taking_it() {
        // The Ley Line has no `Open`: there is nothing to decide, so a step is
        // held the instant it is passed. That asymmetry with Mastery is the
        // whole difference between the two tracks.
        let sim = earned(16);
        assert_eq!(ley_line(sim.world())[0].standing, Standing::Taken);
        assert_eq!(
            next(sim.world()),
            Some(24),
            "the next thing is the first mastery tier",
        );
    }

    #[test]
    fn a_tier_opens_all_at_once_and_its_nodes_wait_to_be_chosen() {
        let below = earned(23);
        let tiers = mastery(below.world());
        assert!(
            tiers[0]
                .iter()
                .all(|node| node.standing == Standing::Locked),
            "a tier opened early: {tiers:?}",
        );

        let at = earned(24);
        let tiers = mastery(at.world());
        assert!(
            tiers[0].iter().all(|node| node.standing == Standing::Open),
            "the tier did not open: {tiers:?}",
        );
        assert!(
            tiers[1]
                .iter()
                .all(|node| node.standing == Standing::Locked),
            "the second tier opened with the first: {tiers:?}",
        );
    }

    #[test]
    fn taking_one_locks_the_rest_of_its_tier() {
        // **Nothing in the game can reach this state yet**, which is exactly why
        // it is worth pinning now: the rule is what makes a tier a *choice*, and
        // the first real node will be checked against it rather than defining it.
        let mut sim = earned(24);
        let first = sim.world().resource::<Progression>().mastery()[0].nodes[0].clone();
        sim.world_mut()
            .resource_mut::<Taken>()
            .0
            .push(first.clone());

        let tier = mastery(sim.world()).remove(0);
        for node in &tier {
            let expected = if node.id == first {
                Standing::Taken
            } else {
                Standing::Locked
            };
            assert_eq!(node.standing, expected, "{}", node.id);
        }
    }

    #[test]
    fn unlocked_and_active_are_two_different_questions() {
        // **The pair the details panel asks**, and the reason `unlocked` is a
        // field rather than a reading of `standing`: `Locked` covers both a total
        // not yet reached and a tier whose one choice a sibling already took.
        // They draw the same on purpose — neither can be had — but one says
        // *work more* and the other says *you chose otherwise*, and a panel that
        // could not tell them apart would send a player to earn something they
        // have already earned.
        let mut sim = earned(24);
        let below = mastery(sim.world()).remove(1);
        assert!(!below[0].unlocked, "a tier at 40 was reachable at 24");
        assert!(!below[0].standing.active());

        let open = mastery(sim.world()).remove(0);
        assert!(open[0].unlocked, "an open tier was not unlocked");
        assert!(!open[0].standing.active(), "an unchosen node was in effect");

        // ...and once a sibling takes the tier, the other stays unlocked and
        // will never be active. That is the state a single word could not carry.
        let first = sim.world().resource::<Progression>().mastery()[0].nodes[0].clone();
        sim.world_mut().resource_mut::<Taken>().0.push(first);
        let spent = mastery(sim.world()).remove(0);
        assert!(spent[0].standing.active(), "the taken one is not in effect");
        assert!(spent[1].unlocked, "the sibling stopped being unlocked");
        assert!(!spent[1].standing.active());

        // The Ley Line answers both at once: passing a step *is* taking it.
        let step = &ley_line(sim.world())[0];
        assert!(step.unlocked && step.standing.active());
    }

    #[test]
    fn every_standing_says_a_word_of_its_own() {
        // §14: the glyph carries the state for anyone who can see it, and this
        // carries it for anyone who cannot. Three states, three words.
        let words: Vec<&str> = [Standing::Taken, Standing::Open, Standing::Locked]
            .into_iter()
            .map(Standing::word)
            .collect();
        assert_eq!(words, ["taken", "open", "locked"]);
    }
}
