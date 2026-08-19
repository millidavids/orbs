//! The game, played through a real terminal.
//!
//! # The third layer, and what belongs in it
//!
//! `orbs-sim`'s tests prove the rules and `orbs-render`'s prove the picture.
//! Neither presses a key. This runs the actual binary inside a real terminal,
//! types at it, and reads the screen back — so it is the only layer that
//! exercises the event loop, the redraw diff, keyboard ownership between five
//! surfaces, the clocks measured off `Time`, and colour as a terminal resolves
//! it.
//!
//! Every defect found in this frontend was found by a person looking at a
//! screen, and every one was invisible to the other two layers: a resize that
//! scrambled the display, a fire ramp climbing to white, `F4` drawn on the
//! border and bound to nothing, a width probe that could never fire, a maze map
//! a whole second behind the arrow keys.
//!
//! # One target, on purpose
//!
//! Cargo runs test *binaries* one after another and only parallelises *within*
//! one, so nine files would be slower than this single target with nine modules.
//! `tests/playing/main.rs` is the layout that achieves it: cargo auto-discovers
//! `tests/*/main.rs`, names the target `playing`, and leaves the siblings as
//! modules rather than compiling each as a target of its own.
//!
//! ```bash
//! scripts/play.sh                                        # all of it
//! cargo test -p orbs-tui --test playing -- --ignored routing::
//! ```
//!
//! # Why every scenario is `#[ignore]`d
//!
//! The gate runs after every step and this takes minutes, not seconds. It is
//! `orbs-balance`'s standing instead: **run it after anything that touches the
//! sim, the shell, or the loop.**

mod archive;
mod brewing;
mod carrying;
mod keys;
mod lens;
mod play;
mod presentation;
mod routing;
mod session;
mod spells;
