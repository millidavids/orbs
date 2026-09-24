//! What a command actually does.
//!
//! The seam the domains plug into. §10.1's brewing loop is live; the remaining
//! dark verbs acknowledge and do nothing, which is honest rather than lazy — a
//! verb whose domain does not exist has nothing to do, and §6 already guarantees
//! the player was told what the orb understood.
//!
//! Split by concern rather than kept whole: `execute.rs` reached 987 lines
//! holding the pipeline, navigation and the log, with only the `match` touching
//! all three.
//!
//! | Module | Verbs |
//! |---|---|
//! | `dispatch` | the `match`, `meditate`, `status`, and the two records every verb can need |
//! | `pipeline` | `move`, `wield`, `stop`, `siphon`, `purge`, `divine` — §10.1's loop |
//! | `recall` | `recall` — §6.1's manual, read *before* the loop |
//! | `navigate` | `attend`, `survey` — §7's places |
//! | `files` | `peruse`, `sift`, `verify` — §3's log |
//!
//! No prose in any of them: rule 6 and §12 put authored text in content files,
//! and these emit facts for a later layer to wrap sentences around.

mod audit;
#[cfg(debug_assertions)]
mod debug;
#[cfg(debug_assertions)]
mod debug_spell;
mod defend;
mod dispatch;
mod files;
mod imbue;
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
mod settle;
mod summon;
mod unfurl;
mod wander;
mod weave;

#[cfg(test)]
mod tests;

#[cfg(debug_assertions)]
pub use debug::{
    Asking, CIRCLE, COURSE, LEARN, Order as SpawnOrder, REACH, SIEGE, SPAWN, SWAP, TAKE, WARD,
    beckoned, beleaguered, giveaway, lesson, order as spawn_order, reaching, shortcut, standing,
    swapping, taking,
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
// Debug-only, and say so rather than warn: `debug_course` and `debug_siege` are
// the only things that reach a republish by `Cwd` from outside its own domain,
// so a release build found both re-exports unused. `muster::refresh` itself
// stays ungated — `stop` abandoning a course calls it.
#[cfg(debug_assertions)]
pub(crate) use muster::refresh as refresh_pylon;
// The bailey's republish. Only the `Cwd` form is re-exported — `defend`'s own
// callers take the node directly, and the siege has no tick-driven republish
// the way the sanctum does.
#[cfg(debug_assertions)]
pub(crate) use defend::refresh as refresh_rampart;
// ...and the die prices, which are raised once at construction rather than on a
// round. `Sim::bare` is the only caller, beside the pylon's for the same reason.
pub(crate) use defend::publish_dice;
pub(crate) use imbue::{land as land_fall, lapse as lapse_charms};
pub(crate) use research::{stacks, tread};
pub(crate) use scribe::Reloaded;
pub use scribe::{Opening, Request, write};
/// Named so `every_scroll_the_lectern_makes_can_be_spent` can ask whether a word
/// authored in `recipes.toml` has anything behind it. Nothing else outside this
/// module needs it — `wield` reaches the effect through `spend`.
pub use scroll::Scroll;
pub use summon::TROOP;
// The circle's republish by `Cwd`, for `debug_circle`, which limns the glyphs
// without going through `limn`. Debug-only for `refresh_pylon`'s reason.
#[cfg(debug_assertions)]
pub(crate) use summon::refresh as refresh_circle;
// A load whose puzzle was refused but whose readings were not, for every puzzle.
pub(crate) use settle::settle as settle_puzzles;
// Crate-internal, and the reason is the defect it closed: `spell::block` has to
// ask the same question `wield` asks, or a scripted spend is charged a
// production slot the typed one is not.
pub use audit::land_sweep;
pub use quit::{Menuing, Quitting};
pub(crate) use scroll::spending;
pub use scry::land as land_probe;
pub use unfurl::Unfurling;
pub use wander::Wandering;
pub use weave::Weaving;

use dispatch::{acknowledge, missing};
