//! One keystroke: the function keys, the surfaces, and the prompt.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::Session;
use super::state::scribing;
use crate::keymap::{as_key, escape_prefixed};
use crate::surfaces::Owner;
use orbs_shell::Raising;

impl Session {
    /// Take one keystroke. Returns `false` when the player asked to leave.
    pub(crate) fn typed(&mut self, event: KeyEvent) -> bool {
        // Any keystroke puts the whole answer on screen, as the window's
        // `finish_reveal` does: waiting must never be something a player is made
        // to do.
        self.reveal.finish();
        // `\x1b` and a byte in one read parse as `Alt+<byte>`, so split them.
        // Lossless with no enhancement flags, where only that pair makes
        // `Alt+Char`; a real Alt chord on a special key is CSI and left alone.
        if let Some(rest) = escape_prefixed(&event) {
            if !self.typed(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)) {
                return false;
            }
            // One level deep and no further: `rest` has Alt cleared.
            return self.typed(rest);
        }
        // `Ctrl-H` is Backspace on many terminals (`stty erase ^H`), so it is
        // translated here, before the chord guard, for every surface.
        let event = if event.code == KeyCode::Char('h')
            && event.modifiers.contains(KeyModifiers::CONTROL)
        {
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)
        } else {
            event
        };
        // F10 leaves, from anywhere — the Bevy build's binding, ungated there
        // too. Not Escape: with a text field on screen, Escape gets a player out
        // of something *smaller*, and all four surfaces use it that way. A
        // player who learns one build's exit must not be trapped in the other's.
        if event.code == KeyCode::F(10) {
            return false;
        }
        // The two that write about a session are gated on there being one: `F6`
        // exports a parse trace and `F7` mutates the sim, and at the threshold
        // both act on a scratch world the first swap throws away — *"pressed at
        // the menu it writes `orbs-parse.tsv` for a session that has not
        // happened"*. `F4`, `F5` and `F10` stay ungated for that build's reason:
        // they are about the screen, the menu is on it, and a way out that only
        // works once you are in is not one.
        if self.threshold.is_waiting() && matches!(event.code, KeyCode::F(6) | KeyCode::F(7)) {
            return true;
        }
        // The rest of the function keys, ungated by surface, exactly as the Bevy
        // build has them: `input_just_pressed` in `Update` with no surface
        // condition, so `F5` works from inside the editor and must here too.
        if self.shortcut(event.code) {
            return true;
        }
        // The chord guard, and it is one field read. Under winit this needs
        // `HeldOver`, `Quiet` and `chord_is_stale` — a press/release model, key
        // repeat, and a ghost modifier left behind when a key-up went to the
        // screenshot overlay — but a terminal carries the modifiers on the
        // event itself. Alt is deliberately absent, as on the other side: AltGr
        // is how European layouts type `@`, `#` and `\`.
        let chord = event
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER);
        if chord {
            // Raw mode makes no SIGINT, so it is ours to answer. No `Cmd+←/→`
            // here: macOS terminals send Home and End on `Fn+←/→`.
            return !matches!(event.code, KeyCode::Char('c' | 'd'));
        }

        // The transcript pages from behind any surface, so before the
        // dispatch — but not under the manual, which answers these keys, nor
        // at the threshold, as the Bevy build's guards.
        let total = self.sim.scrollback().records().drawn_len();
        match event.code {
            KeyCode::PageUp | KeyCode::PageDown
                if self.surfaces.reading.is_some() || self.threshold.is_waiting() => {}
            KeyCode::PageUp => {
                self.scroll.page(self.page, true, total);
                return true;
            }
            KeyCode::PageDown => {
                self.scroll.page(self.page, false, total);
                return true;
            }
            _ => {}
        }

        let owner = self.surfaces.owner(&self.scroll);
        if owner == Owner::Prompt {
            // The prompt has the keys — unless a surface only just gave them up
            // and the player has not let go yet.
            if self.held_over.swallows(&event) {
                return true;
            }
        } else {
            self.held_over.handed_over();
        }
        if owner != Owner::Prompt {
            self.surfaces.typed(
                owner,
                event.code,
                &mut self.sim,
                &mut self.scroll,
                scribing(&self.readers, self.held.driver),
            );
            // `Sim::walk` spends no tick (§19), so the panel is refreshed by
            // hand, as the Bevy build's `resource_changed::<Tower>` does — all
            // of it, since a walk can pick up a fragment or solve the maze.
            if owner == Owner::Maze {
                self.panel.refresh(&self.sim);
            }
            // A surface may have opened another — `scribe` from the weave
            // screen cannot happen, but a save can close the editor and hand
            // the prompt back on the same keystroke.
            self.open_surfaces();
            return true;
        }

        let Some(key) = as_key(event.code) else {
            return true;
        };
        if let Some(finished) =
            orbs_shell::apply(&key, &mut self.line, &mut self.offered, &self.sim)
        {
            // Not a tick: `submit` echoes now and queues for the next `step`,
            // which keeps the echo instant while effects stay tick-aligned
            // (§19). `plain` leaves the reader loaded and does not ask it — see
            // `Session::held`.
            let reader = self.readers.reader(self.held.driver);
            // During an installation the line goes through it, and one that
            // wakes the tower asks `run` for the real one and reaches no world.
            if let Some(install) = self.install.as_mut() {
                if let Some(raise) = install.submit(&mut self.sim, &finished, reader) {
                    self.surfaces.swapping = Some(Raising::Installed(raise));
                    return true;
                }
            } else {
                self.sim.submit_with(&finished, reader);
            }
            self.scroll.rewind();
            // `unfurl` and `wander` answer on the tick they are typed, so the
            // surface they ask for must be taken before the next keystroke.
            self.open_surfaces();
        }
        self.ghost = self
            .line
            .ghost(self.sim.scene(), !self.sim.choices().is_empty());
        true
    }

    /// The function keys that are neither the prompt nor a surface.
    ///
    /// Returns whether this key was one of them; the border advertises `F4`
    /// (§19). The rules are `orbs-shell`'s.
    fn shortcut(&mut self, code: KeyCode) -> bool {
        match code {
            // Flip the focus mode. Visibly this only moves the border's own hint
            // until multiplexing returns the second pane in Phase 11a — with one
            // pane both tilings are identical, which §19 records as deliberate,
            // and the Bevy build is equally inert and equally bound. Through
            // `DisplayMode::flipped`, so the two cannot disagree about what the
            // other mode is.
            KeyCode::F(4) => {
                self.held.mode = self.held.mode.flipped();
                // Written down, which this build did not do. `F4` and the `focus`
                // row are one setting seen twice, and the key that changed it
                // wrote nothing — so this page and the window's opening mode
                // both disagreed with what the player had just pressed.
                orbs_shell::settings::set(
                    orbs_shell::settings::FOCUS,
                    orbs_render::DisplayMode::word(self.held.mode),
                );
            }
            // §14's linear stream — the same pane, described instead of drawn.
            // The terminal build is the one DESIGN.md calls the cheapest route
            // to screen-reader support, so it is the last that should lack it.
            KeyCode::F(5) => self.linear.toggle(),
            // Silent on success, like the Bevy build: it logs, and a log in raw
            // mode would scribble across the screen. The file appearing is the
            // confirmation. A failure must not take the session down — the
            // tester whose run it was recording is still playing.
            KeyCode::F(6) => {
                if let Err(error) = orbs_shell::export_trace(&self.sim) {
                    tracing::warn!("parse trace -> {} failed: {error}", orbs_shell::TRACE_PATH);
                }
            }
            // §3's registers: invisible here (`theme`), but the same change to
            // the same world, which rule 2 permits.
            KeyCode::F(7) => {
                orbs_shell::cycle_register(&mut self.sim);
            }
            // §14's greyscale, as the window binds it. About the screen, so
            // ungated like `F4` and `F5`, and written down like `F4`: the key and
            // the `sight` row are one setting seen twice.
            KeyCode::F(8) => {
                let mut held = self.carrying();
                held.sight = held.sight.next();
                orbs_shell::settings::set(orbs_shell::settings::SIGHT, held.sight.name());
                self.hold(held);
            }
            _ => return false,
        }
        true
    }
}
