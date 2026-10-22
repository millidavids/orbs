//! The world's tick, what a surface asked for, and keeping the tower.

use super::Session;
use super::state::scribing;
use crate::carried::settings;

/// How often the tower writes itself out, in world ticks.
///
/// The Bevy build's `sim::persist` picks the same number: §8 asks for *"every N
/// ticks and on significant events"*, and a minute is the most a crash may cost
/// in a game whose slowest single action is 94 ticks. One cadence for both, or
/// a player moving between them loses four times as much in one.
const AUTOSAVE_TICKS: u64 = 60;

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
        if self.sim.tick().get().is_multiple_of(AUTOSAVE_TICKS) {
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
            scribing(self.held.driver, self.scribe.as_deref()),
        );
    }

    /// Act on what a surface asked for, and say whether the loop goes on.
    ///
    /// Not inside [`tick`](Self::tick) any more: the clock does not run before
    /// a tower is chosen, so neither does `tick`, and the menu could raise a
    /// swap nothing ever looked at — the lengths page stayed on screen for ever.
    /// Found by playing it, because a dump hosts no loop;
    /// `routing::the_threshold_reaches_a_tower` looks at it now. Once per pass
    /// rather than once per tick, so a swap or a `quit` lands on the frame the
    /// key arrived.
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
            if let Some(menu) = self.surfaces.menuing.as_mut()
                && menu.settings() != fresh.as_slice()
            {
                menu.show_settings(fresh);
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
            let mut menu = orbs_shell::Menu::at(orbs_shell::Stance::Threshold);
            menu.show_driver(self.held.driver);
            menu.show_settings(settings(&self.carrying()));
            self.surfaces.menuing = Some(menu);
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
        let Some(path) = self.kept.clone() else {
            self.save_failed = false;
            return;
        };
        let Err(error) = orbs_shell::write_save_to(&path, &self.sim.snapshot()) else {
            self.save_failed = false;
            return;
        };
        if !self.save_failed {
            self.save_failed = true;
            // The terminal is in raw mode and the screen is ours, so this cannot
            // go to stdout. `tracing` is where the other build's goes too.
            tracing::error!("the tower could not be written out: {error}");
            // And in voice: the terminal is in raw mode, so `tracing` reaches
            // nobody until the session is over.
            self.sim.say_save_failed();
        }
    }
}
