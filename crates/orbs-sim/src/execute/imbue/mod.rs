//! The forge: §10's Enchanting, and the last of the seven domains.
//!
//! A charm is bound by a **lattice** of glyphs that must all be lit. `imbue`
//! names a tool and a charm and opens one; `snap` turns a column and the glyphs
//! beside it; `anneal` lets the lattice cascade and binds the charm if every
//! glyph holds. What a charm *does* once bound is `tower::charm`; what it costs
//! is authored in `content/forge.toml`.
//!
//! # The seam is *when*, not *what*
//!
//! The bailey's split, and the same argument: one file per verb would put three
//! copies of *find the lattice, refuse if there is none* in three places.
//!
//! | | |
//! |---|---|
//! | [`verbs`] | what a player types |
//! | [`publish`] | what a spell can then ask about — the half the scripting rests on |
//! | [`shared`] | the lattice, and the one-line `say` all three use |

mod publish;
mod shared;
mod verbs;

pub(crate) use publish::lapse;
pub(crate) use verbs::land;
pub(super) use verbs::{anneal, imbue, snap};
