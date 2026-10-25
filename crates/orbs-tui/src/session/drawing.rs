//! The frame's clocks, and the screen they are painted onto.

use std::io::Write;
use std::time::Duration;

use super::Session;
use super::state::scribing;

impl Session {
    /// Advance the clocks the painters read, then draw.
    ///
    /// `overstep` is how far into the next world tick this frame falls.
    pub(crate) fn frame(
        &mut self,
        elapsed: Duration,
        overstep: f32,
        out: &mut impl Write,
    ) -> std::io::Result<()> {
        let delta = elapsed.as_secs_f32();
        // The sequence's own clock, advanced here and nowhere else. It walks
        // whole stages per call rather than one per frame, so a stall during
        // boot catches up instead of leaving the animation stuck part-way.
        self.boot.advance(elapsed);
        // Before the reveal counts drawn records, which the echo row changes.
        self.sim.set_terse_echo(self.held.echo);
        self.animate(delta, overstep);
        // After the keys, so a keystroke restarts the settle clock before it is
        // advanced — otherwise the frame a player types on counts toward a pause
        // they have not taken yet. §19's *"there is no `save`"*: stop typing and
        // the buffer writes itself out.
        self.surfaces.settle(
            delta,
            &mut self.sim,
            scribing(&self.readers, self.held.driver),
        );
        self.draw(out)
    }

    /// Ask the shell for a screen and put it on the terminal.
    pub(crate) fn draw(&mut self, out: &mut impl Write) -> std::io::Result<()> {
        // Every frame, because a swap brings a stream that shows every echo.
        // Touches the world only when the answer changes.
        self.sim.set_terse_echo(self.held.echo);
        // Before either reset, the last moment last frame exists: a crossing
        // departs from the screen it replaces. Refused while one runs.
        self.passing.keep(&self.frame);
        // The card and nothing else until the orb is found. Painted before the
        // layout below rather than over it: `paint_booting` computes its own
        // single-pane layout, and the game's screen has a rail, a prompt and a
        // caret that must not be on this one — §4's `Prompt` stage was removed
        // for that reason.
        if !self.boot.is_live() {
            self.frame.reset(self.grid);
            // The floor applies to the card too, and it did not: this returned
            // before the hostable check below, so a small terminal watched the
            // logo run off the right edge for thirteen seconds before the card
            // explained why. `§19` calls a shrunk terminal a normal runtime
            // state, and it is normal during boot as well.
            if !self.hostable() {
                orbs_shell::paint_too_small(&mut self.frame, &self.sim);
                self.frame.set_cursor(None);
                return self.screen.draw(&self.frame, out);
            }
            orbs_shell::paint_booting(
                &mut self.frame,
                self.boot.stage(),
                self.boot.progress(),
                &self.engine,
            );
            // No caret: nothing on this screen takes a keystroke, and a caret is
            // the game's one promise about where typing lands.
            self.frame.set_cursor(None);
            return self.screen.draw(&self.frame, out);
        }

        let screen = self.screen();

        // How far `PageUp` moves is measured from the screen it is about to
        // draw, so a resize changes the page on the same frame it changes the
        // pane. **Records, not rows** — see `orbs_shell::page_step`.
        self.page = orbs_shell::page_step(&screen, &self.sim, self.scroll.back());

        self.frame.reset(self.grid);
        if screen.is_hostable() {
            // Kept as said on the first frame that shows it, not when the menu
            // was built: a boot card quit early, or a terminal too small for
            // the menu, would have used it up unseen.
            if self.warns {
                self.warns = false;
                if let Some(menu) = self.surfaces.menuing.as_mut() {
                    menu.warn_once();
                }
            }
            orbs_shell::paint(
                &mut self.frame,
                &mut self.linear,
                &mut self.passing,
                orbs_shell::View {
                    sim: &self.sim,
                    line: &self.line,
                    screen: &screen,
                    panes: &self.panes,
                    reveal: &self.reveal,
                    offered: &self.offered,
                    ghost: &self.ghost,
                    panel: &self.panel,
                    scroll: &self.scroll,
                    bench: &self.bench,
                    climb: &self.climb,
                    // The editor's viewport follows its caret, and how many
                    // lines fit is a fact only the painter has — which is why
                    // this one field is `&mut`.
                    editing: self.surfaces.editing.as_mut(),
                    weaving: self.surfaces.weaving.as_ref(),
                    walking: self.surfaces.walking,
                    menuing: self.surfaces.menuing.as_ref(),
                    // `&mut` for the editor's reason, one surface over: a
                    // chapter is wrapped to the pane and scrolled within it, and
                    // only the painter knows how wide and how tall it is.
                    reading_manual: self.surfaces.reading.as_mut(),
                    installing: self.install.as_ref(),
                },
            );
        } else {
            orbs_shell::paint_too_small(&mut self.frame, &self.sim);
        }
        self.screen.draw(&self.frame, out)
    }
}
