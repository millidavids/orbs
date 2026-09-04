//! The Ley Line, read (DESIGN.md §11.5, §19).
//!
//! # The tower's line
//!
//! One list of stations on total experience. A **step** grants something and
//! passing it *is* the grant — there is nothing to decide and nothing to store,
//! since what a player has is a function of [`Experience`] against the authored
//! list. A **fork** offers its nodes, and `take` chooses one: the fork opening
//! costs nothing and taking a node closes it, which is what lets experience stay
//! unspendable (§11.5's *"accumulates and is never spent"*) while still
//! offering a choice.
//!
//! # Three lanes
//!
//! A fork's nodes are one each of provision, war and craft — more resources,
//! better combat, a faster orb — so a choice is always between kinds of play.
//! What each node grants, and its lane, is `tower::grant`'s: derived from the
//! id, one parser, no second table to drift. `Progression::check` refuses a
//! fork with two nodes in one lane.
//!
//! # What was here before
//!
//! This module was `tower::mastery`, and the branching tree it read is the Ley
//! Line's forks now. Mastery is the seven per-domain lines — see there.

use bevy_ecs::prelude::*;

use crate::content::Progression;

use super::Experience;
use super::grant::{Grant, Lane, granted};

/// Which fork nodes the orb has taken.
///
/// **A list of ids, not a set of flags.** An id is what the content file
/// authors, what a sentence is keyed by, and what a save carries; a `bool` per
/// node would have to be regenerated every time the line grew.
/// `Progression::check` refuses a duplicate id, so a name here means one node.
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

    /// Record one, if it is not held already.
    ///
    /// **Idempotent, and the caller does not have to check.** A double `take` is
    /// a player pressing Enter twice, not a bug worth a second refusal path —
    /// and a duplicate id here would make `steps_granted` count the same node
    /// twice and hand out a budget nobody bought.
    pub(crate) fn hold(&mut self, id: &str) {
        if !self.0.iter().any(|held| held == id) {
            self.0.push(id.to_owned());
        }
    }
}

/// Whether the orb has taken a node granting `grant`.
///
/// **The one question the gated words ask**, so they cannot come to disagree
/// about what "unlocked" means. `spell::budget` sums [`Grant::Steps`] instead,
/// because speed is a quantity and the others are a yes.
#[must_use]
pub fn holds(world: &World, grant: Grant) -> bool {
    world
        .resource::<Taken>()
        .ids()
        .iter()
        .any(|id| granted(id) == Some(grant))
}

/// Whether `id` is a fork node at all.
///
/// Derived from the grant, so a node cannot be takeable and worthless at once.
#[must_use]
pub fn is_real(id: &str) -> bool {
    granted(id).is_some()
}

/// What a node or a step is, right now.
///
/// **Three states, and the screen must draw them apart without colour** (§14).
/// The glyph carries it for anyone who can see it and a word carries it for
/// anyone who cannot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Held. A step that has been passed, or a node that was chosen.
    Taken,
    /// Reachable now: the fork is open and its choice is unspent.
    Open,
    /// Not yet. Either the total is short, or the fork already spent its choice.
    Locked,
}

impl Standing {
    /// The word a screen reader hears.
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
    /// panel asks both: an open fork node is unlocked and doing nothing, and a
    /// node whose sibling took the fork's one choice is unlocked and will never
    /// do anything.
    #[must_use]
    pub const fn active(self) -> bool {
        matches!(self, Self::Taken)
    }
}

/// One node on the line, as the screen draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The id, which is the content file's word and the prose key.
    pub id: String,
    /// The total that reaches it.
    pub at: u64,
    /// Where it stands against the tower's experience.
    pub standing: Standing,
    /// Whether the tower has earned enough to reach it at all.
    ///
    /// **Not derivable from [`standing`](Self::standing)**, which is why it is a
    /// field. `Locked` covers two situations — a total not yet reached, and a
    /// fork whose one choice a sibling already took — and they are the same
    /// glyph on purpose, because in both cases the node cannot be had. They are
    /// *not* the same sentence: one is *work more* and the other is *you chose
    /// otherwise*.
    pub unlocked: bool,
    /// Which lane it sits in. A step has none.
    pub lane: Option<Lane>,
}

impl Node {
    /// What a cursor holds to mean *this* node.
    ///
    /// **Not the id.** A step's id is what it *grants*, and eight stations grant
    /// concentration — so a cursor holding `concentration` would be pointing at
    /// eight nodes at once, and every one of them would draw aimed. The total is
    /// what tells them apart, and `Progression::check` makes the line ascend
    /// strictly, so no two stations share one.
    ///
    /// The id stays what it was: the content file's word, the prose key, and
    /// what `take` names. Only the *cursor* needed a finer identity.
    #[must_use]
    pub fn mark(&self) -> String {
        format!("{}:{}", self.at, self.id)
    }
}

/// One station of the line: a step, or a fork with its siblings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Station {
    /// The total that reaches it.
    pub at: u64,
    /// Whether this is a choice.
    pub fork: bool,
    /// The step's one node, or the fork's siblings in lane order.
    pub nodes: Vec<Node>,
    /// What passing it opens.
    pub opens: Vec<String>,
}

/// The Ley Line, in order, against what the tower has earned.
///
/// A step is `Taken` or `Locked` and never `Open`: passing one *is* taking it.
/// A fork whose total has not been reached holds only `Locked` nodes; an open
/// one holds `Open` nodes until one is taken, after which that one is `Taken`
/// and **its siblings are `Locked`** — the choice is spent, and the screen has
/// to show that it was spent rather than that the others were never there.
#[must_use]
pub fn ley_line(world: &World) -> Vec<Station> {
    let earned = world.resource::<Experience>().get();
    let taken = world.resource::<Taken>();
    world
        .resource::<Progression>()
        .ley_line()
        .iter()
        .map(|station| {
            let unlocked = earned >= station.at;
            let nodes = if station.is_fork() {
                let spent = station
                    .nodes
                    .iter()
                    .any(|node| taken.ids().iter().any(|held| held == node));
                let mut nodes: Vec<Node> = station
                    .nodes
                    .iter()
                    .map(|id| Node {
                        id: id.clone(),
                        at: station.at,
                        standing: standing_of(id, unlocked, spent, taken),
                        unlocked,
                        lane: granted(id).map(Grant::lane),
                    })
                    .collect();
                // **Lane order, then the file's.** A stable sort, so two nodes
                // of one lane keep the author's order until the rule that
                // forbids them lands.
                nodes.sort_by_key(|node| node.lane);
                nodes
            } else {
                vec![Node {
                    id: station.grants.clone().unwrap_or_default(),
                    at: station.at,
                    standing: if unlocked {
                        Standing::Taken
                    } else {
                        Standing::Locked
                    },
                    unlocked,
                    lane: None,
                }]
            };
            Station {
                at: station.at,
                fork: station.is_fork(),
                nodes,
                opens: station.opens.clone(),
            }
        })
        .collect()
}

/// Where one fork node stands.
fn standing_of(node: &str, unlocked: bool, spent: bool, taken: &Taken) -> Standing {
    if taken.ids().iter().any(|held| held == node) {
        Standing::Taken
    } else if unlocked && !spent {
        Standing::Open
    } else {
        Standing::Locked
    }
}

/// Open what every station between `before` and `after` opens.
///
/// **The other half of "passing a step *is* the grant".** What a step grants is
/// derived from the total and needs no writing down; what it *opens* is a set
/// the world holds, so crossing has to write it. Without this the grimoire and
/// the forge — the two rooms no deed reaches — were shut for ever in a sealed
/// tower, and the forge line's charm with them.
///
/// Called from [`credit`](super::credit), which is the only thing that moves
/// the total, so every path that earns arrives here. A half-open range: a
/// station exactly at `before` was crossed by an earlier call and must not be
/// announced twice — and `opening` refuses to say a second time in any case.
pub(crate) fn cross(world: &mut World, before: u64, after: u64) {
    let opens = opens_between(world, before, after);
    if opens.is_empty() {
        return;
    }
    super::opened::opening(world, &opens);
}

/// Open everything the line has already passed, without saying so.
///
/// **A restore, not an earning.** The total and what passing a station opened
/// are two records of one fact, and only the second is written down — so a
/// document written before a station carried its `opens`, or by a build where
/// nothing applied them, comes back with a room shut that the tower paid for
/// long ago. Nothing would ever open it: the crossing happens once, at the
/// edge, and that edge is in the past.
///
/// **Silent**, for [`Experience::restore`](super::Experience)'s reason: a load
/// that congratulated you on work you did yesterday would be reporting a lie in
/// voice. It runs before `seal`, so a room this opens gets its markers cleared
/// with the rest.
pub(crate) fn caught_up(world: &mut World) {
    let earned = world.resource::<Experience>().get();
    for key in opens_between(world, 0, earned) {
        super::opened::open(world, &key);
    }
}

/// What the stations in `(before, after]` open, in line order.
fn opens_between(world: &World, before: u64, after: u64) -> Vec<String> {
    world
        .resource::<Progression>()
        .ley_line()
        .iter()
        .filter(|station| station.at > before && station.at <= after)
        .flat_map(|station| station.opens.iter().cloned())
        .collect()
}

/// The next total that reaches anything.
///
/// **The "what's next" line, and the whole reason a locked station is drawn at
/// all.** A screen that showed only what you have is a receipt; §11.5 wants the
/// player to know what the work is *for*. `None` means nothing more is authored
/// yet, which is not the same as being finished.
#[must_use]
pub fn next(world: &World) -> Option<u64> {
    let earned = world.resource::<Experience>().get();
    world
        .resource::<Progression>()
        .ley_line()
        .iter()
        .map(|station| station.at)
        .find(|at| *at > earned)
}

/// The last total the line is authored to, for a bar to be measured against.
///
/// **Derived, where it was a fixed hundred** — §19 said the hundred would
/// become derived once the curve reached it, and the curve now runs past it.
#[must_use]
pub fn scale(world: &World) -> u64 {
    world
        .resource::<Progression>()
        .ley_line()
        .last()
        .map_or(1, |station| station.at.max(1))
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

    /// The station at `at`.
    fn station(sim: &Sim, at: u64) -> Station {
        ley_line(sim.world())
            .into_iter()
            .find(|station| station.at == at)
            .unwrap_or_else(|| panic!("no station at {at}"))
    }

    #[test]
    fn a_cold_tower_has_taken_nothing_and_is_told_what_is_next() {
        let sim = Sim::new(1);
        let line = ley_line(sim.world());
        assert!(!line.is_empty(), "the ley line has no stations at all");
        assert!(
            line.iter()
                .flat_map(|station| &station.nodes)
                .all(|node| node.standing == Standing::Locked),
            "a cold tower has already passed a station: {line:?}",
        );
        assert_eq!(line[0].at, 16);
        assert!(!line[0].fork, "the first station is a choice");
        assert_eq!(next(sim.world()), Some(16), "nothing to work toward");
    }

    #[test]
    fn passing_a_step_is_taking_it() {
        // A step has no `Open`: there is nothing to decide, so it is held the
        // instant it is passed. That asymmetry with a fork is the whole
        // difference between the two kinds of station.
        let sim = earned(16);
        assert_eq!(station(&sim, 16).nodes[0].standing, Standing::Taken);
        assert_eq!(
            next(sim.world()),
            Some(24),
            "the next thing is the first fork"
        );
    }

    #[test]
    fn a_fork_opens_all_at_once_and_its_nodes_wait_to_be_chosen() {
        let below = earned(23);
        assert!(
            station(&below, 24)
                .nodes
                .iter()
                .all(|node| node.standing == Standing::Locked),
            "a fork opened early",
        );

        let at = earned(24);
        assert!(
            station(&at, 24)
                .nodes
                .iter()
                .all(|node| node.standing == Standing::Open),
            "the fork did not open",
        );
        assert!(
            station(&at, 40)
                .nodes
                .iter()
                .all(|node| node.standing == Standing::Locked),
            "the second fork opened with the first",
        );
    }

    #[test]
    fn taking_one_locks_the_rest_of_its_fork() {
        let mut sim = earned(24);
        let first = station(&sim, 24).nodes[0].id.clone();
        sim.world_mut().resource_mut::<Taken>().hold(&first);

        for node in station(&sim, 24).nodes {
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
        let mut sim = earned(24);
        let below = station(&sim, 40);
        assert!(!below.nodes[0].unlocked, "a fork at 40 was reachable at 24");
        assert!(!below.nodes[0].standing.active());

        let open = station(&sim, 24);
        assert!(open.nodes[0].unlocked, "an open fork was not unlocked");
        assert!(
            !open.nodes[0].standing.active(),
            "an unchosen node was in effect"
        );

        let first = open.nodes[0].id.clone();
        sim.world_mut().resource_mut::<Taken>().hold(&first);
        let spent = station(&sim, 24);
        assert!(
            spent.nodes[0].standing.active(),
            "the taken one is not in effect"
        );
        assert!(
            spent.nodes[1].unlocked,
            "the sibling stopped being unlocked"
        );
        assert!(!spent.nodes[1].standing.active());

        // A step answers both at once: passing it *is* taking it.
        let step = station(&sim, 16);
        assert!(step.nodes[0].unlocked && step.nodes[0].standing.active());
    }

    #[test]
    fn every_standing_and_every_lane_says_a_word_of_its_own() {
        let words: Vec<&str> = [Standing::Taken, Standing::Open, Standing::Locked]
            .into_iter()
            .map(Standing::word)
            .collect();
        assert_eq!(words, ["taken", "open", "locked"]);
        let lanes: Vec<&str> = [Lane::Provision, Lane::War, Lane::Craft]
            .into_iter()
            .map(Lane::word)
            .collect();
        assert_eq!(lanes, ["provision", "war", "craft"]);
    }

    #[test]
    fn the_scale_is_the_last_station() {
        // Concentration 8 — the soft ending — is the last thing authored.
        let sim = Sim::new(1);
        assert_eq!(scale(sim.world()), 10_000);
    }
}
