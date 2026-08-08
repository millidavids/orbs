//! The tower: the directory tree that *is* the world (DESIGN.md §7).

mod boot;
mod build;
mod heat;
mod node;
mod panel;
mod sabotage;
mod scene;
mod stock;
mod work;

pub use boot::report;
pub use build::raise;
pub use heat::{
    ATHANOR, Banked, Burning, banked, burn, burning, damp, find as find_athanor, kindle, lit,
    refuse_cold,
};
pub use node::{
    Cwd, Domain, Fixture, HeatSource, Held, Name, Nameable, NodeId, NodeIds, Operation, Protected,
    Store, children_of, domain_of, filesystem_root, path_of, root, where_at,
};
pub use panel::{Craft, Instrument, Meter, State, instruments};
pub use sabotage::{Log, Poisoned, drift, emit_lines, poison, poisoned, verify};
pub use scene::{Topics, rebuild, scene_at};
pub use stock::{Stock, give, held, take};
pub mod spell;
pub use work::{
    Busy, CAPACITY, DIVINE_TICKS, PURGE_TICKS, Product, Triaging, Working, begin, busy, contents,
    finish, in_flight, occupied, purge, refuse_busy, stop,
};
