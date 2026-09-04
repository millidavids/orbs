//! The tower: the directory tree that *is* the world (DESIGN.md §7).

pub mod assault;
pub mod audit;
mod boot;
mod brief;
mod build;
pub mod chant;
pub mod charm;
pub mod dice;
mod erosion;
mod experience;
pub mod grant;
mod heat;
mod home;
mod keep;
pub mod lattice;
mod learned;
pub mod ley;
pub mod mastery;
pub mod maze;
mod node;
pub mod opened;
mod panel;
pub mod pylon;
pub mod quintessence;
pub mod reach;
mod sabotage;
pub mod satchel;
mod scene;
pub mod siege;
mod stock;
mod tally;
pub mod ward;
mod work;

pub use assault::{Reached, Retimed, Rewritten};
pub use audit::{Cooling, Surface};
pub use boot::report;
pub use brief::{
    Brief, Cast, DOMAINS, Mark, Marks, briefs, clear_mark, mark, mark_fault_at, running_spells,
};
pub use build::{declared, fixture_of, operated, raise, raise_count, raise_reading};
pub use chant::{Chant, Strike, Syllable};
pub use charm::{Charm, Charmed, charm_left, charmed};
pub use dice::{Die, Effect, Landed, Modifier, Roll};
pub use erosion::{
    EROSION, Integrity, MENDED_PER_WARD, STANDING, erode, height_for, mend, mend_by, wear_by,
};
pub use experience::{Experience, concentration, credit, quintessence_steps, worth};
pub use grant::{Grant, Lane, granted, steps_granted};
pub use heat::{
    ATHANOR, Ash, Banked, Burning, banked, burn, burning, damp, find as find_athanor, kindle, lit,
    refuse_cold,
};
pub use home::home;
pub use keep::{ARSENAL, admits, keep, keeping, kept};
pub use learned::{CERTAIN, Learned, discover, learn};
pub use ley::{Node, Standing, Station, Taken, holds, is_real, ley_line, next, scale};
// `mastery::Reached` is reached through its module: the siege's `assault::Reached`
// already holds the bare name here.
pub use mastery::{Line, Progress, Stop, Walk, advance, mastery, progress};
pub use maze::{Maze, Sense, Square, Way};
pub use node::{
    Cwd, Domain, Fixture, Grouped, HeatSource, Held, Keep, Name, Nameable, NodeId, NodeIds,
    Operation, Protected, Reading, Store, children_of, domain_of, filesystem_root, find_by_path,
    group_at, groups_at, path_of, path_of_id, readings_at, root, where_at,
};
pub use opened::{
    Key as OpenKey, Known, Opened, Sealed, Sealing, charm_key, domain_key, known, open, recipe_key,
    seal, sealed_room_of,
};
pub use panel::{Craft, Instrument, Meter, State, Unit, instruments, instruments_in, state_at};
pub use pylon::{Course, Refused, STATIONS};
pub use quintessence::{
    PER_LEY_STEP, QUINTESSENCE_BASE, Quintessence, REGEN_PER_ROUND, REGEN_TICKS, ceiling,
    ceiling_for, regenerate,
};
pub use sabotage::{
    Log, Poisoned, Substituted, claimed, drift, emit_lines, poison, poisoned, restore, settling,
    substitute, substitution, verify, vigilant_interval,
};
pub use satchel::{SATCHEL, Satchel};
pub use scene::{Topics, rebuild, scene_at};
pub use siege::{Band, Intent, Outcome, Round, Siege};
pub use stock::{Stock, give, give_endless, held, holdings, take};
pub use tally::{BOUND, EVENTS, FIGURE, SECRET, SIEGE, SIEGE_WON, Tally, Work, done, note};
pub use ward::{SIGILS, SOCKETS, Shift, Ward};
pub mod spell;
pub use work::{
    Bidden, Busy, CAPACITY, DIVINE_TICKS, PURGE_TICKS, Product, QUICKENED_BY, QUICKENED_TICKS,
    Quickened, Triaging, Working, begin, busy, contents, finish, hurried_from, in_flight, occupied,
    purge, quickened, refuse_busy, stop,
};
