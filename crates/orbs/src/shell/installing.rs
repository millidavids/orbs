//! The installation in the Bevy shell: holding it, and watching the practice
//! tower.
//!
//! [`Install`] is `orbs-shell`'s and knows nothing about Bevy. While it is
//! `Some`, the `Tower` *is* the practice tower, so typing reaches it the
//! ordinary way: `commanding::submit` hands each line to [`Install::submit`],
//! and the swap raises the real tower when it wakes. DESIGN.md §19, *The orb is
//! installed before the tower wakes*.

use bevy::prelude::*;
use orbs_shell::Install;

use crate::sim::Tower;

/// The installation, while the tower is its practice tower.
///
/// On `shell_resources!`, so every swap clears it: the wake, a load from the
/// menu, and another `new`, which sets a fresh one after the reset.
#[derive(Resource, Debug, Default)]
pub(crate) struct Installing(Option<Install>);

impl Installing {
    /// An installation under way.
    pub(crate) const fn begun(install: Install) -> Self {
        Self(Some(install))
    }

    /// The installation, if one is under way.
    pub(crate) const fn get(&self) -> Option<&Install> {
        self.0.as_ref()
    }

    /// The installation, to hand a line.
    pub(crate) const fn get_mut(&mut self) -> Option<&mut Install> {
        self.0.as_mut()
    }
}

/// Whether a new tower is installed before it wakes. Read once, as
/// `setting::Skipping` is, and off under `cargo test` so no test reaching `new`
/// depends on the developer's shell.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Installs(pub(crate) bool);

impl Default for Installs {
    fn default() -> Self {
        if cfg!(test) {
            return Self(false);
        }
        Self(orbs_shell::install_wanted(orbs_shell::settings::installs()))
    }
}

/// Run condition: an installation is under way.
pub(crate) fn installing(installing: Res<Installing>) -> bool {
    installing.0.is_some()
}

/// Finish the steps the practice tower now shows were done.
///
/// On every change to the tower, not only on a line: `attend` is judged by
/// where the player stands, and that moves on the tick after it is typed.
pub(crate) fn follow_install(tower: Res<Tower>, mut installing: ResMut<Installing>) {
    if let Some(install) = installing.0.as_mut() {
        install.observe(tower.sim());
    }
}
