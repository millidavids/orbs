//! The game, played through a real terminal.
//!
//! The third layer: the real binary in a real terminal, typed at and read
//! back — the event loop, the redraw diff, keyboard ownership, the clocks and
//! terminal colour. One target with modules, since cargo parallelises only
//! within one.
//!
//! ```bash
//! scripts/play.sh                                        # all of it
//! cargo test -p orbs-tui --test playing -- --ignored routing::
//! ```
//!
//! `#[ignore]`d, taking minutes; run it after anything touching the sim, the
//! shell or the loop.

mod archive;
mod auditing;
mod bailey;
mod besieged;
mod brewing;
mod carrying;
mod demons;
mod driving;
mod forge;
mod garrison;
mod installing;
mod kept;
mod keys;
mod launch;
mod ledger;
mod lens;
mod menagerie;
mod motion;
mod play;
mod presentation;
mod probes;
mod readers;
mod reading;
mod refusals;
mod routing;
mod sanctum;
mod satchel;
mod sealed;
mod session;
mod settings;
mod sight;
mod spells;
mod standing;
mod supply;
mod swapping;
mod threshold;
mod tmux;
mod transcript;
mod unfurling;
