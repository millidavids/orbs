//! The tower: the directory tree that *is* the world (DESIGN.md §7).

mod boot;
mod build;
mod node;
mod sabotage;
mod scene;
mod work;

pub use boot::report;
pub use build::raise;
pub use node::{Cwd, Name, Nameable, NodeId, NodeIds, Protected, children_of, path_of};
pub use sabotage::{Log, Poisoned, drift, emit_lines, poison, poisoned, verify};
pub use scene::rebuild;
pub use work::{CAPACITY, DECOCT_TICKS, DIVINE_TICKS, Working, begin, finish, occupied, purge};
