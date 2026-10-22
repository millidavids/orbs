//! Putting a different tower in front of the player.

use orbs_shell::{Boot, Threshold};

use super::Session;
use crate::surfaces::Swap;
use crate::term;

impl Session {
    /// The tower a surface asked for, if one did.
    pub(crate) const fn take_swap(&mut self) -> Option<Swap> {
        self.surfaces.swapping.take()
    }

    /// A session on the tower `asked` names — or `None`, and the player stays
    /// here, when it cannot be opened.
    ///
    /// Read before the old session is dropped, so a save that will not open
    /// leaves the player where they were rather than in a half-built world. A
    /// new game gets a seed of its own (`orbs_shell::new_game_seed`).
    pub(crate) fn raise(&mut self, asked: Swap) -> Option<Self> {
        let mut installing = None;
        let (raised, kept) = match asked {
            // The wizard's name travels in the `Raise`: `Sim::begun` takes none,
            // and a new game here once came up as `orbs $ `.
            Swap::Begin { keep, length } => {
                let begun = orbs_shell::Raise {
                    keep,
                    length,
                    seed: orbs_shell::new_game_seed(),
                    // The name the session holds, not the file's: a name chosen
                    // where nothing can be written still names the tower.
                    wizard: orbs_shell::wizard_named(self.held.name.as_deref()),
                }
                .begin(self.held.installs);
                installing = begun.install;
                (begun.sim, begun.keep)
            }
            Swap::Installed(raise) => {
                let woken = raise.wake();
                (woken.sim, woken.keep)
            }
            // The tower being left was written before this ran (`run`), so a
            // load of the slot in hand reads what was just written rather than
            // its last autosave — or is refused, if that write failed.
            Swap::Load(path) => {
                let written = !self.save_failed;
                match orbs_shell::loaded(&path, self.kept.as_deref(), written) {
                    Ok(loaded) => (loaded, Some(path)),
                    Err(why) => {
                        // Left where they were, on the menu that asked, which
                        // says why.
                        tracing::error!("the tower in {} was not opened: {why:?}", path.display());
                        if let Some(menu) = self.surfaces.menuing.as_mut() {
                            menu.refused(&path, why);
                        }
                        return None;
                    }
                }
            }
        };

        // `ORBS_CONTENT` again, on the tower that just arrived. It was applied
        // once, to the scratch sim built before the loop — and the swap is the
        // *only* path to a tower, so a writer with the switch set silently got
        // the `include_str!` text back for every line and manual chapter, from
        // their fast no-GPU target. That is CLAUDE.md rule 6's whole workflow.
        let mut raised = raised;
        for file in orbs_shell::load(&mut raised) {
            tracing::warn!("content: {file} did not load; running on the built-in text");
        }

        // A whole new `Session`, the terminal's answer to the other build's
        // eighteen-resource reset: every derived field goes back to its default
        // because it is a new struct, so none can be forgotten. The grid is
        // re-read in case the terminal was resized while the menu was up, and
        // the threshold is crossed here because this is the one place a tower
        // the player asked for arrives.
        let mut next = Self::new(
            raised,
            kept,
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
        next.install = installing;
        // The new shadow buffer believes the terminal is blank, which only the
        // first session's is: unwiped, the menu's words showed through a tower.
        next.screen.resize(next.grid);
        Some(next)
    }
}
