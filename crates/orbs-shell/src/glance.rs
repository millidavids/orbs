//! What the orb last noticed about the room, held between ticks.
//!
//! Four questions the painters ask every frame and the world answers once a
//! second. Asking the sim directly from a painter is what this replaced, and the
//! cost was not theoretical: `Sim::briefs` alone walks every built room, builds
//! two `QueryState`s, and runs `instruments_in` per room — at 60 Hz, for a table
//! that changes at 1 Hz.
//!
//! [`Panel::refresh`] is the whole of it, and it is a method rather than a
//! system so that a frontend without an ECS can call it too.

use bevy_ecs::prelude::Resource;
use orbs_sim::Sim;

/// §10.1's instrument panel, as the sim last reported it.
///
/// **Rebuilt on tick, not per frame.** `tower::instruments` walks the room's
/// children, then each fixture's children, cloning a `String` per name and
/// allocating a `Vec` per lookup — roughly twenty allocations for the
/// laboratory's five instruments, at 60 Hz, for state that changes at most once a
/// second. The sim is the authority either way; this is where the answer waits
/// between ticks.
#[derive(Resource, Debug, Default)]
pub struct Panel {
    /// The instruments where the player is standing.
    pub instruments: Vec<orbs_sim::tower::Instrument>,
    /// That place's leaf name, for the panel's spoken summary.
    pub domain: String,
    /// The stacks the player is standing over, if they are open.
    ///
    /// **On the same tick clock as the instruments, and it belongs here for the
    /// same reason.** A 49-cell `Vec` rebuilt at 60 Hz would be the allocation
    /// this resource exists to stop; rebuilt once a second it is exactly as
    /// fresh as the world it describes, because the world moves at 1 Hz too.
    pub stacks: Option<orbs_render::Stacks>,
    /// The ward the player is standing over, if a reading is open.
    ///
    /// Beside the stacks and on the same clock, for the same reason: it is a
    /// description of a world that moves at 1 Hz, so rebuilding it per frame
    /// would allocate sixty times for one change.
    pub ward: Option<orbs_render::Board>,
    /// The course the player is standing over, if one is up.
    ///
    /// Beside the other two and on the same clock. Only one of the three can
    /// ever be present, because they belong to three different rooms and the
    /// player stands in one.
    pub pylon: Option<orbs_render::Pylon>,
    /// The figure being sung, if one is running.
    ///
    /// Beside the other three and on the same clock. Only one of the four can
    /// ever be present, because they belong to four different rooms and the
    /// player stands in one.
    pub figure: Option<orbs_render::Figure>,
    /// The siege being fought, if one is.
    ///
    /// Beside the other four and on the same clock. Only one of the five can
    /// ever be present, because they belong to five different rooms and the
    /// player stands in one.
    pub rampart: Option<orbs_render::Rampart>,
    /// The forge's open lattice, if the player is standing at it.
    pub lattice: Option<orbs_render::LatticeBoard>,
    /// Every domain at a glance, for §9's rail.
    ///
    /// **Here rather than asked from the painter, and it is the most expensive of
    /// the four.** `Sim::briefs` walks every top-level room, builds two
    /// `QueryState`s, and runs `instruments_in` — with its per-recipe `matching`
    /// and `gathering` allocations — once per built room. Called from `rail::paint`
    /// it ran all of that at 60 Hz for data that changes at 1 Hz, which is
    /// precisely what this resource was created to stop.
    pub briefs: Vec<orbs_sim::tower::Brief>,
}

impl Panel {
    /// Re-read the panel from the world.
    ///
    /// **A method, not a system.** Every frontend needs this exact set of five
    /// questions asked in this exact order once a tick; only the Bevy build has
    /// a `resource_changed` guard to hang it on, and a guard is not a reason for
    /// the answer to live somewhere a terminal cannot reach.
    pub fn refresh(&mut self, sim: &Sim) {
        self.instruments = sim.instruments();
        self.domain = orbs_sim::parser::leaf(&sim.location()).to_owned();
        self.stacks = sim.stacks();
        self.ward = sim.ward();
        self.pylon = sim.pylon();
        self.figure = sim.figure();
        self.rampart = sim.rampart();
        self.lattice = sim.lattice();
        self.briefs = sim.briefs();
    }
}
