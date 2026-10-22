//! One keystroke: the function keys, the surfaces, and the prompt.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::Session;
use super::state::scribing;
use crate::keymap::{as_key, escape_prefixed};
use crate::surfaces::{Owner, Swap};

impl Session {
    /// Take one keystroke. Returns `false` when the player asked to leave.
    pub(crate) fn typed(&mut self, event: KeyEvent) -> bool {
        // Any keystroke puts the whole answer on screen, as the window's
        // `finish_reveal` does: waiting must never be something a player is made
        // to do.
        self.reveal.finish();
        // Escape with a letter behind it is two keystrokes and this build threw
        // the first away: crossterm reads the pty buffer in one syscall, and
        // `\x1b` plus any byte in the same read parses as `Alt+<that byte>` —
        // so leaving the spell editor and typing `quit` fast enough put `quit`
        // in the buffer as a line of the spell.
        //
        // Splitting it apart is lossless only here: with no keyboard-enhancement
        // flags (this build pushes none, see the `Repeat` note in `play`)
        // crossterm builds `Alt+Char` from an `\x1b <byte>` pair and nothing
        // else. A *real* Alt chord on a special key arrives CSI-encoded and is
        // left alone, which matters — `Alt+Left` is word-left in plenty of
        // terminals. The game binds none anyway, and the guard below excludes
        // Alt so AltGr can type `@`, `#` and `\`; on Linux AltGr composes in the
        // terminal and never reaches this branch.
        if let Some(rest) = escape_prefixed(&event) {
            if !self.typed(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)) {
                return false;
            }
            // One level deep and no further: `rest` has Alt cleared.
            return self.typed(rest);
        }
        // `Ctrl-H` is Backspace on a great many terminals and the chord guard
        // below was eating it: `stty erase ^H` (PuTTY's default) sends `0x08`,
        // crossterm turns every `0x01..=0x1A` byte into `Char(letter) +
        // CONTROL`, and §6 makes this a game played entirely by typing — so a
        // typo became uncorrectable short of clearing the line. Translated here
        // rather than special-cased in the guard, because every surface reads
        // `KeyCode::Backspace` and none should have to know this; the cost is a
        // deliberate `Ctrl-H`, which the game does not bind.
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
            // Raw mode means the terminal does not turn this into SIGINT, so it
            // is ours to answer — and a full-screen application that cannot be
            // left by the one chord everybody tries is a trap.
            //
            // One accepted divergence: the Bevy build lets `Cmd+←/→` mean
            // Home/End, *"which a Mac keyboard has no other key for"*. Every
            // macOS emulator sends real Home and End on `Fn+←/→`, and a second
            // chord path would be a second place to forget a surface.
            return !matches!(event.code, KeyCode::Char('c' | 'd'));
        }

        // Exactly one surface takes a keystroke: two consuming a key is
        // invisible until a player types `:wq` and finds it in their history.
        //
        // The transcript scrolls without `unfurl` and from behind a surface —
        // the word exists because the *key* could not be discovered, not
        // because the key needs permission. Answered before the surface
        // dispatch, which is where this was wrong: below it the editor, the
        // weave screen and the maze swallowed both keys through their `_ => {}`,
        // while the Bevy build gates `scroll_back`/`scroll_forward` on `booted`
        // alone.
        //
        // Except the manual, the one surface that answers these keys itself;
        // without the guard PgDn pages the chapter *and* the transcript under
        // it. The Bevy build's is `manualling::not_reading_the_manual`. Nor at
        // the threshold, where the transcript is a scratch world's behind a menu
        // — the Bevy build gates these on `playing`.
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
                scribing(self.held.driver, self.scribe.as_deref()),
            );
            // The one keystroke that reaches the world without a tick.
            // `Sim::walk` is the third entry point (§19): an arrow moves the
            // reading *now* and spends no world time, so it lands between the
            // ticks `Panel` is rebuilt on — and `panel.stacks` is what the
            // inline map and `wander`'s pane draw from, so without this the map
            // runs up to a second stale, invisibly to every test that drives
            // `Sim` directly. The Bevy build needs no such line: `walk` takes
            // `Tower` by `&mut` and `refresh_panel` hangs on
            // `resource_changed::<Tower>`. This is that, by hand.
            //
            // The whole panel, not just `stacks`: a walk can pick up a fragment
            // and can solve the maze outright, which moves the cabinet's stock
            // and the archive's rail box with it.
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
            let reader = match self.held.driver {
                orbs_shell::Driver::Plain => None,
                orbs_shell::Driver::Augury => self.augury.as_deref(),
            };
            // During an installation the line goes through it, and one that
            // wakes the tower asks `run` for the real one and reaches no world.
            if let Some(install) = self.install.as_mut() {
                if let Some(raise) = install.submit(&mut self.sim, &finished, reader) {
                    self.surfaces.swapping = Some(Swap::Installed(raise));
                    return true;
                }
            } else {
                match reader {
                    Some(augur) => self.sim.submit_reading(&finished, augur),
                    None => self.sim.submit(&finished),
                }
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
    /// Returns whether this key was one of them. `F4` is why it exists at all:
    /// `prompt.rs` draws `F4 deep` into the session border every frame and the
    /// terminal build shipped without answering it — §19's *"the first
    /// affordance the game showed was one that did not work"* a second time. The
    /// rules behind them are `orbs-shell`'s, so the two builds cannot disagree
    /// about the register cycle or what the trace file is called.
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
            // §3's three tonal registers. Visibly inert here and bound anyway:
            // `Presentation` picks a *face* in the Bevy build's glyph atlas, and
            // a terminal's face is whatever the user set — one of this
            // frontend's accepted degradations (`theme.rs`). It still does the
            // same thing to the same world, and `F6`'s trace shows the register
            // move. A key that changed the world in one build and not the other
            // is the divergence worth avoiding; one that changes it invisibly is
            // the frontend limitation rule 2 permits.
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
