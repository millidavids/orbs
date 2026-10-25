//! Putting a different tower in front of the player.

use orbs_shell::{Boot, Raised, Raising, Threshold};

use super::Session;
use crate::term;

impl Session {
    /// The tower a surface asked for, if one did.
    pub(crate) const fn take_swap(&mut self) -> Option<Raising> {
        self.surfaces.swapping.take()
    }

    /// A session on the tower `asked` names — or `None`, and the player stays
    /// here, when it cannot be opened. Live content files that would not load
    /// are added to `missed`.
    ///
    /// Read before the old session is dropped, so a save that will not open
    /// leaves the player where they were rather than in a half-built world. A
    /// new game gets a seed of its own (`orbs_shell::new_game_seed`).
    pub(crate) fn raise(&mut self, asked: Raising, missed: &mut Vec<&'static str>) -> Option<Self> {
        // The tower being left was written before this ran (`run`), so a load
        // of the slot in hand reads what was just written rather than its last
        // autosave — or is refused, if that write failed.
        let raised = asked.raise(orbs_shell::Swapping {
            kept: self.kept.as_deref(),
            written: !self.save_failed,
            seed: orbs_shell::new_game_seed(),
            wizard: orbs_shell::wizard_named(self.held.name.as_deref()),
            installs: self.held.installs,
        });
        let Raised {
            begun,
            missed: more,
        } = match raised {
            Ok(raised) => raised,
            Err((path, why)) => {
                // Left where they were, on the menu that asked, which says why.
                if let Some(menu) = self.surfaces.menuing.as_mut() {
                    menu.refused(&path, why);
                }
                return None;
            }
        };
        missed.extend(more);

        // A whole new `Session`, the terminal's answer to the other build's
        // eighteen-resource reset: every derived field goes back to its default
        // because it is a new struct, so none can be forgotten. The grid is
        // re-read in case the terminal was resized while the menu was up, and
        // the threshold is crossed here because this is the one place a tower
        // the player asked for arrives.
        let mut next = Self::new(
            begun.sim,
            begun.keep,
            // Already booted: the orb woke once and the player has been typing
            // at its menu since. See `Session::new`.
            Boot::finished(),
            Threshold::Playing,
            // Carried from the session being replaced, not re-read from the file
            // — see `Carried`. The file is a best-effort write, so re-reading
            // turned §14's linear stream off on the way into a tower with
            // nothing said.
            self.carrying(),
            term::grid().unwrap_or(self.grid),
            self.screen.narrow(),
            self.engine.clone(),
        );
        next.install = begun.install;
        // The new shadow buffer believes the terminal is blank, which only the
        // first session's is: unwiped, the menu's words showed through a tower.
        next.screen.resize(next.grid);
        Some(next)
    }
}
