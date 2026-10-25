//! The world's tick, what a surface asked for, and keeping the tower.

use super::Session;
use super::state::scribing;
use crate::carried::settings;

impl Session {
    /// Advance the world by one tick, and re-read what the painters cache.
    ///
    /// `Panel` is rebuilt here rather than every frame for the reason the Bevy
    /// build guards it on `resource_changed::<Tower>`: it walks every room and
    /// allocates per recipe, for a table that changes at 1 Hz, and a hand-rolled
    /// loop has no change detection. Returns `false` when the session is over.
    pub(crate) fn tick(&mut self) -> bool {
        self.sim.step();
        // `attend` is judged by where the player stands, which moves here.
        if let Some(install) = self.install.as_mut() {
            install.observe(&self.sim);
        }
        // §8 permits a save at a tick boundary and nowhere else, and this is one
        // — `step` has returned, so `Pending` is drained and `Skip` is spent.
        if orbs_shell::autosave_due(self.sim.tick().get()) {
            self.keep();
        }
        // `quit` lands here, not at `submit`, and the delay is not worth
        // engineering away: every verb's *effect* runs at the next `step` (§19),
        // so quitting gets the same beat as every other word. The flag is only
        // set by a confirmed `quit` — the sim asks first — so nothing else to
        // check.
        if self.sim.quitting() {
            return false;
        }
        self.panel.refresh(&self.sim);
        // A verb may have asked for a surface, and an open one may have been
        // closed from under the player by a spell. `menu`'s handshake is taken
        // in here, by `Surfaces::open`.
        self.open_surfaces();
        self.surfaces.tick(&self.sim);
        if !self.answered() {
            return false;
        }
        self.ghost = self
            .line
            .ghost(self.sim.scene(), !self.sim.choices().is_empty());
        true
    }

    /// Take whatever surface a verb has asked for.
    ///
    /// One call site's worth of arguments in one place, because there are three:
    /// the tick, a keystroke a surface answered, and a line submitted. Three
    /// copies of seven arguments, which the settings rows made eight — and how
    /// one came to hand over a different list.
    pub(super) fn open_surfaces(&mut self) {
        let held = self.carrying();
        self.surfaces.open(
            &mut self.sim,
            &mut self.scroll,
            self.page,
            held,
            scribing(&self.readers, self.held.driver),
        );
    }

    /// Act on what a surface asked for, and say whether the loop goes on.
    ///
    /// Once per pass, not in [`tick`](Self::tick), which does not run before a
    /// tower (`routing::the_threshold_reaches_a_tower`).
    pub(crate) fn answered(&mut self) -> bool {
        // Taken here rather than in the surface, for `Surfaces::driving`'s
        // reason: the reader belongs to the session. Unlike leaving and swapping
        // it does not end the loop — it changes how the next line is read.
        if let Some(driver) = self.surfaces.driving.take() {
            self.held.driver = driver;
        }
        // The settings this build holds. `install` takes effect at the next
        // `new`, which is the next time it is asked.
        if let Some((setting, value)) = self.surfaces.setting.take() {
            let mut held = self.carrying();
            held.apply(&setting, &value);
            // The drawn count moves with the echo row, so a place held in drawn
            // records would no longer name the same line, and echoes put back
            // are not news to type. The window's `keep_echo`.
            if held.echo != self.held.echo {
                self.sim.set_terse_echo(held.echo);
                self.reveal
                    .settle_at(self.sim.scrollback().records().drawn_len());
                self.scroll.rewind();
            }
            self.hold(held);
        }
        // The settings page says what is in effect, and stops the moment a
        // function key changes something behind it: the rows are a snapshot
        // taken when the menu opened and `F4`/`F5` are ungated. Without this
        // (`setting::follow_the_keys` in the Bevy build) `F5` left the row
        // saying `off` while the pane drew the linear stream. Only while a
        // settings page is up and only when something moved — the comparison is
        // what keeps it off the per-frame path.
        if self
            .surfaces
            .menuing
            .as_ref()
            .is_some_and(orbs_shell::Menu::showing_settings)
        {
            let fresh = settings(&self.carrying());
            if let Some(menu) = self.surfaces.menuing.as_mut() {
                menu.follow(fresh);
            }
        }

        // The net under a menu that cannot be closed. `Stance::Threshold` makes
        // `Outcome::Close` unreachable, so this should never fire — but if
        // anything did close it the player would be stranded at the prompt of a
        // scratch world that never ticks and is never kept. The Bevy build has
        // had `open_at_the_threshold` since the feature shipped.
        if self.threshold.is_waiting() && self.surfaces.menuing.is_none() {
            tracing::error!(
                "the threshold's menu was closed and had to be put back up; \
                 something reached `MenuOutcome::Close` at `Stance::Threshold`",
            );
            self.surfaces.menuing = Some(orbs_shell::Menu::open_at(
                orbs_shell::Stance::Threshold,
                self.held.driver,
                settings(&self.carrying()),
            ));
        }

        // The menu's own `quit` leaves the loop the same way, and so does a
        // swap; `run` decides which of the three it was.
        !(self.surfaces.leaving || self.surfaces.swapping.is_some())
    }

    /// Write the tower out, and complain once if it will not go.
    ///
    /// Once, for the reason the other build gives: there is no `save` verb, so a
    /// silent failure is a whole session lost with nothing said — but a save is
    /// attempted every sixty ticks, so a line per attempt is sixty an hour.
    pub(crate) fn keep(&mut self) {
        // This tower's own path, never `save_path()` — see `Session::kept`.
        if orbs_shell::keep(&self.sim, self.kept.as_deref(), &mut self.save_failed) {
            // And in voice: the terminal is in raw mode, so `tracing` reaches
            // nobody until the session is over.
            self.sim.say_save_failed();
        }
    }
}
