//! The tower: the directory tree that *is* the world (DESIGN.md §7).

mod boot;
mod build;
mod experience;
mod heat;
mod home;
mod keep;
mod mastery;
pub mod maze;
mod node;
mod panel;
mod sabotage;
mod scene;
mod stock;
mod work;

pub use boot::report;
pub use build::{operated, raise, raise_reading};
pub use experience::{Experience, concentration, credit, worth};
pub use heat::{
    ATHANOR, Banked, Burning, banked, burn, burning, damp, find as find_athanor, kindle, lit,
    refuse_cold,
};
pub use home::home;
pub use keep::{ARSENAL, admits, keep, keeping, kept};
pub use mastery::{Node, Standing, Taken, ley_line, mastery, next};
pub use maze::{Maze, Sense, Square, Way};
pub use node::{
    Cwd, Domain, Fixture, HeatSource, Held, Keep, Name, Nameable, NodeId, NodeIds, Operation,
    Protected, Reading, Store, children_of, domain_of, filesystem_root, path_of, root, where_at,
};
pub use panel::{Craft, Instrument, Meter, State, instruments, state_at};
pub use sabotage::{Log, Poisoned, drift, emit_lines, poison, poisoned, verify};
pub use scene::{Topics, rebuild, scene_at};
pub use stock::{Stock, give, give_endless, held, holdings, take};
pub mod spell;
pub use work::{
    Busy, CAPACITY, DIVINE_TICKS, PURGE_TICKS, Product, QUICKENED_BY, QUICKENED_TICKS, Quickened,
    Triaging, Working, begin, busy, contents, finish, in_flight, occupied, purge, quickened,
    refuse_busy, stop,
};
