//! The tower: the directory tree that *is* the world (DESIGN.md §7).

mod boot;
mod brief;
mod build;
mod erosion;
mod experience;
mod heat;
mod home;
mod keep;
mod learned;
mod mastery;
pub mod maze;
mod node;
mod panel;
pub mod pylon;
pub mod reach;
mod sabotage;
mod scene;
mod stock;
pub mod ward;
mod work;

pub use boot::report;
pub use brief::{Brief, DOMAINS, Mark, Marks, briefs, clear_mark, mark, mark_fault_at};
pub use build::{declared, fixture_of, operated, raise, raise_count, raise_reading};
pub use erosion::{EROSION, Integrity, STANDING, erode, height_for, mend};
pub use experience::{Experience, concentration, credit, worth};
pub use heat::{
    ATHANOR, Ash, Banked, Burning, banked, burn, burning, damp, find as find_athanor, kindle, lit,
    refuse_cold,
};
pub use home::home;
pub use keep::{ARSENAL, admits, keep, keeping, kept};
pub use learned::{CERTAIN, Learned, discover, learn};
pub use mastery::{Node, Standing, Taken, ley_line, mastery, next};
pub use maze::{Maze, Sense, Square, Way};
pub use node::{
    Cwd, Domain, Fixture, Grouped, HeatSource, Held, Keep, Name, Nameable, NodeId, NodeIds,
    Operation, Protected, Reading, Store, children_of, domain_of, filesystem_root, find_by_path,
    group_at, groups_at, path_of, path_of_id, readings_at, root, where_at,
};
pub use panel::{Craft, Instrument, Meter, State, Unit, instruments, instruments_in, state_at};
pub use pylon::{Course, Refused, STATIONS};
pub use sabotage::{
    Log, Poisoned, Substituted, claimed, drift, emit_lines, poison, poisoned, restore, settling,
    substitute, substitution, verify,
};
pub use scene::{Topics, rebuild, scene_at};
pub use stock::{Stock, give, give_endless, held, holdings, take};
pub use ward::{SIGILS, SOCKETS, Shift, Ward};
pub mod spell;
pub use work::{
    Bidden, Busy, CAPACITY, DIVINE_TICKS, PURGE_TICKS, Product, QUICKENED_BY, QUICKENED_TICKS,
    Quickened, Triaging, Working, begin, busy, contents, finish, in_flight, occupied, purge,
    quickened, refuse_busy, stop,
};
