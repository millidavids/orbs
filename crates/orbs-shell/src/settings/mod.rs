//! What the player has chosen about the orb itself, kept between sessions.
//!
//! `store` is the file, `values` is what a settings page is made of, and `keys`
//! is the shell's own questions of it — one per setting the shell itself needs
//! an answer to, which today is the driver and the sticky boot skip.
//!
//! Nothing is kept until a frontend calls [`keep`], and no test binary does —
//! which is what stops a `cargo test` reading and writing the developer's own
//! settings. `store` has the reason.
//!
//! Everything else is the frontends': they build the [`Row`]s from what they
//! actually have, apply what comes back, and write it here. See `values` for why
//! the menu is handed data rather than asking a type.

mod keys;
mod store;
mod values;

pub use keys::{DRIVER, FOCUS, SKIP, driver, set_driver, skips_boot};
pub(crate) use store::FILE;
pub use store::{get, keep, load, set};
pub use values::{Category, LEVELS, OFF, ON, Row, SWITCH, is_on, loudness, switched};
