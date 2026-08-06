//! The tower: the directory tree that *is* the world (DESIGN.md §7).

mod boot;
mod build;
mod heat;
mod node;
mod panel;
mod sabotage;
mod scene;
mod work;

pub use boot::report;
pub use build::raise;
pub use heat::{
    ATHANOR, Banked, Burning, banked, burn, burning, damp, find as find_athanor, kindle, lit,
    refuse_cold,
};
pub use node::{
    Cwd, Fixture, HeatSource, Name, Nameable, NodeId, NodeIds, Operation, Protected, Store,
    children_of, path_of,
};
pub use panel::{Instrument, Meter, State, instruments};
pub use sabotage::{Log, Poisoned, drift, emit_lines, poison, poisoned, verify};
pub use scene::{Topics, rebuild};
pub use work::{
    Busy, CAPACITY, DIVINE_TICKS, PURGE_TICKS, Product, Triaging, Working, begin, busy, contents,
    finish, in_flight, occupied, purge, refuse_busy, siphon, stop,
};
