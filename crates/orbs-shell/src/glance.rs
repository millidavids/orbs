//! What the orb last noticed about the room, held between ticks.
//!
//! What the painters ask every frame and the world answers once a tick, held
//! here so nothing is rebuilt at 60 Hz. [`Panel::refresh`] is a method, so a
//! frontend without an ECS can call it.

use bevy_ecs::prelude::Resource;
use orbs_sim::Sim;

/// §10.1's instrument panel, as the sim last reported it.
///
/// Rebuilt on tick, not per frame; the sim is the authority, and this is
/// where its answer waits.
#[derive(Resource, Debug, Default)]
pub struct Panel {
    /// The instruments where the player is standing.
    pub instruments: Vec<orbs_sim::tower::Instrument>,
    /// That place's leaf name, for the panel's spoken summary.
    pub domain: String,
    /// The room the player is standing in, which is not the same thing.
    ///
    /// At the alembic, `domain` is `alembic` and this is `laboratory` — what
    /// [`Passing`](crate::Passing) keys on, or `attend alembic` crosses.
    pub room: String,
    /// The stacks the player is standing over, if they are open.
    ///
    /// On the tick clock, as everything here is.
    pub stacks: Option<orbs_render::Stacks>,
    /// The ward the player is standing over, if a reading is open.
    ///
    /// Only one room's puzzle is ever present: the player stands in one.
    pub ward: Option<orbs_render::Board>,
    /// The course the player is standing over, if one is up.
    pub pylon: Option<orbs_render::Pylon>,
    /// The beast waiting at the menagerie's circle, if there is one.
    pub circle: Option<orbs_render::Circle>,
    /// The siege being fought, if one is.
    pub rampart: Option<orbs_render::Rampart>,
    /// The forge's open lattice, if the player is standing at it.
    pub lattice: Option<orbs_render::LatticeBoard>,
    /// Where the tower's total stands against the Ley Line's next station.
    ///
    /// On the tick clock, as everything here is.
    pub station: orbs_sim::Toward,
    /// Where renown stands against the next rank, and what the tower is called.
    pub rankward: orbs_sim::Toward,
    /// The tower's title, if it has earned one.
    pub rank: Option<String>,
    /// Experience, whole rather than toward a tier — what a gain's `+N` is
    /// measured on, since `station.done` drops at every crossing.
    pub experience: u64,
    /// Renown, whole, for the same reason.
    pub renown: u64,
    /// What the room the player is standing in has made in all: its first
    /// ledger row, which the gauge strip draws at its right edge.
    pub ledger: Option<orbs_sim::tower::LedgerRow>,
    /// The mastery line of the room the player is standing in, for the road
    /// under the pane's title (§11.5).
    ///
    /// On the tick clock.
    pub line: Option<orbs_sim::Line>,
    /// Every domain at a glance, for §9's rail.
    ///
    /// The most expensive answer here: `Sim::briefs` walks every room.
    pub briefs: Vec<orbs_sim::tower::Brief>,
}

impl Panel {
    /// Re-read the panel from the world.
    ///
    /// A method, not a system, so the terminal asks it too.
    pub fn refresh(&mut self, sim: &Sim) {
        self.instruments = sim.instruments();
        self.domain = orbs_sim::parser::leaf(&sim.location()).to_owned();
        // `None` above `/tower`, where no work happens — the leaf is the honest
        // fallback there, so `/tower` and the arsenal are still different places
        // to be standing.
        let domain = sim.domain();
        self.room = domain.clone().unwrap_or_else(|| self.domain.clone());
        self.stacks = sim.stacks();
        self.ward = sim.ward();
        self.pylon = sim.pylon();
        self.circle = sim.circle();
        self.rampart = sim.rampart();
        self.lattice = sim.lattice();
        self.briefs = sim.briefs();
        // The room, not the leaf: a player standing in the alembic is in the
        // laboratory, and the laboratory's line is the road they should see.
        self.line = domain
            .as_deref()
            .and_then(|domain| sim.mastery().into_iter().find(|line| line.domain == domain));
        self.station = sim.toward_station();
        self.rankward = sim.toward_rank();
        self.rank = sim.rank();
        self.experience = sim.experience();
        self.renown = sim.renown();
        // The room's count, as the road takes the room — none above `/tower`.
        self.ledger = domain.as_deref().and_then(|room| sim.ledger_of(room));
    }
}
