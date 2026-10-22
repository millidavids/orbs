//! What the player has chosen about the orb itself, kept between sessions.
//!
//! `store` is the file, `values` is what a settings page is made of, and `keys`
//! is the shell's own questions of it — one per setting the shell itself needs
//! an answer to: the driver, the sticky boot skip and the installation.
//!
//! Nothing is kept until a frontend calls [`keep`], and no test binary does —
//! which is what stops a `cargo test` reading and writing the developer's own
//! settings. `store` has the reason.
//!
//! Everything else is the frontends': they build the [`Row`]s from what they
//! actually have, apply what comes back, and write it here. See `values` for why
//! the menu is handed data rather than asking a type.

mod keys;
mod name;
mod pair;
mod store;
mod values;

pub use keys::{
    DRIVER, ECHO, ECHOES, FOCUS, INSTALL, MOTION, MOTION_VAR, PACE, SCALE, SIGHT, SKIP, WARNED,
    driver, installs, motion_from, motion_wanted, moves, pace, set_driver, skips_boot, terse_echo,
    warn_once,
};
pub use name::{LONGEST_NAME, NAME, NameFault, checked_name, chosen_name};
pub use pair::Pair;
pub(crate) use store::FILE;
pub use store::{get, keep, load, set};
pub use values::{Category, LEVELS, OFF, ON, Row, SWITCH, Secret, is_on, loudness, switched};
