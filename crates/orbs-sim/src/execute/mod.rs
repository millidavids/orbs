//! What a command actually does.
//!
//! The seam the domains plug into: `dispatch` holds the `match`, and each
//! verb's body lives with its concern. A dark verb acknowledges and does
//! nothing. No prose: these emit facts, and content files hold the sentences
//! (rule 6).

mod audit;
#[cfg(debug_assertions)]
mod debug;
#[cfg(debug_assertions)]
mod debug_spell;
mod defend;
mod dispatch;
mod files;
mod handshake;
mod imbue;
mod muster;
mod navigate;
mod pipeline;
mod price;
mod queue;
mod quit;
mod readings;
mod recall;
mod research;
mod scribe;
mod scroll;
mod scry;
mod settle;
mod status;
mod summon;
mod unfurl;
mod wander;
mod weave;

#[cfg(test)]
mod tests;

#[cfg(debug_assertions)]
pub use debug::{Asking, DebugWord, Order as SpawnOrder, open as open_door};
#[cfg(debug_assertions)]
pub use debug_spell::{
    Order as SpellOrder, SPELL, order as spell_order, run as run_spell_order, spells as dev_spells,
};
pub use dispatch::{
    LOG, MAX_MEDITATE, execute_one, is_gated, is_live, offered, run_pending, spell_expect,
    spell_vocabulary,
};
// For `tower::spell`'s `Domain`.
pub use navigate::find_domain;
// The pylon's republish by node, for `tower::erode`, which runs on a tick when
// the player may be standing anywhere.
pub(crate) use muster::publish as publish_pylon;
// The bailey's republish by `Cwd`, for `debug_siege`.
#[cfg(debug_assertions)]
pub(crate) use defend::refresh as refresh_rampart;
// The die prices, raised once at construction by `Sim::bare`.
pub(crate) use defend::publish_dice;
pub(crate) use imbue::{land as land_fall, lapse as lapse_charms};
pub(crate) use research::{stacks, tread};
pub(crate) use scribe::Reloaded;
pub use scribe::{Opening, Request, write};
/// Named for `every_scroll_the_lectern_makes_can_be_spent`.
pub use scroll::Scroll;
pub use summon::TROOP;
// A puzzle's republish by `Cwd`, for a debug word that changed it; and a
// load's tidy of every puzzle.
#[cfg(debug_assertions)]
pub(crate) use settle::refresh;
pub(crate) use settle::settle as settle_puzzles;

pub use audit::land_sweep;
pub use handshake::{Handshake, Menuing, Unfurling, Wandering, Weaving};
pub use quit::Quitting;
pub(crate) use scroll::drain as drain_scrolls;
// `spell::block` asks what `wield` asks, or a scripted spend takes a slot.
pub(crate) use scroll::spending;
pub use scry::land as land_probe;

use dispatch::{acknowledge, missing};
