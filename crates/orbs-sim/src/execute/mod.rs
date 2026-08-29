//! What a command actually does.
//!
//! The seam the domains plug into. §10.1's brewing loop is live; the remaining
//! dark verbs acknowledge and do nothing, which is honest rather than lazy — a
//! verb whose domain does not exist has nothing to do, and §6 already guarantees
//! the player was told what the orb understood.
//!
//! Split by concern rather than kept whole. `execute.rs` reached 987 lines
//! holding three unrelated things — the pipeline, navigation, and the log — with
//! only the sixteen-arm `match` touching all of them, so every phase's work
//! landed in the same file and a brewing change conflicted with a log change for
//! no semantic reason.
//!
//! | Module | Verbs |
//! |---|---|
//! | `dispatch` | the `match`, `meditate`, `status`, and the two records every verb can need |
//! | `pipeline` | `move`, `wield`, `stop`, `siphon`, `purge`, `divine` — §10.1's loop |
//! | `recall` | `recall` — §6.1's manual, read *before* the loop |
//! | `navigate` | `attend`, `survey` — §7's places |
//! | `files` | `peruse`, `sift`, `verify` — §3's log |
//!
//! **No prose in any of them.** Rule 6 and §12 put authored text in content
//! files; these emit facts and let a later layer wrap sentences around them.

#[cfg(debug_assertions)]
mod debug;
#[cfg(debug_assertions)]
mod debug_spell;
mod dispatch;
mod files;
mod muster;
mod navigate;
mod pipeline;
mod queue;
mod quit;
mod readings;
mod recall;
mod research;
mod scribe;
mod scroll;
mod scry;
mod sing;
mod unfurl;
mod wander;
mod weave;

#[cfg(test)]
mod tests;

#[cfg(debug_assertions)]
pub use debug::{
    COURSE, LEARN, Order as SpawnOrder, SPAWN, SWAP, TAKE, WARD, giveaway, lesson,
    order as spawn_order, shortcut, swapping, taking,
};
#[cfg(debug_assertions)]
pub use debug_spell::{
    Order as SpellOrder, SPELL, order as spell_order, run as run_spell_order, spells as dev_spells,
};
pub use dispatch::{
    LOG, MAX_MEDITATE, execute_one, is_gated, is_live, offered, run_pending, spell_expect,
    spell_vocabulary,
};
// Crate-internal: `Sim::stacks` needs it and `divine` is a private module, so
// the re-export is what makes it nameable rather than what makes it public.
pub use navigate::find_domain;
// The sanctum's republish, for the two paths that change what the pylon has to
// say without going through `haul`. `refresh_pylon` reads `Cwd` and is
// `debug_course`'s; `publish_pylon` takes the node and is `tower::erode`'s,
// which runs on a tick when the player may be standing anywhere.
pub(crate) use muster::publish as publish_pylon;
pub(crate) use muster::refresh as refresh_pylon;
pub(crate) use research::{stacks, tread};
pub(crate) use scribe::Reloaded;
pub use scribe::{Opening, Request, write};
/// Named so `every_scroll_the_lectern_makes_can_be_spent` can ask whether a word
/// authored in `recipes.toml` has anything behind it. Nothing else outside this
/// module needs it — `wield` reaches the effect through `spend`.
pub use scroll::Scroll;
pub(crate) use sing::{Answered, circle_at, lapse as lapse_chant, strike as strike_syllable};
pub use sing::{Chorusing, Patient, TROOP};
// Crate-internal, and the reason is the defect it closed: `spell::block` has to
// ask the same question `wield` asks, or a scripted spend is charged a
// production slot the typed one is not.
pub use quit::Quitting;
pub(crate) use scroll::spending;
pub use scry::land as land_probe;
pub use unfurl::Unfurling;
pub use wander::Wandering;
pub use weave::Weaving;

use dispatch::{acknowledge, missing};
