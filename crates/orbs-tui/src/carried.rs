//! The player's settings: what a swap carries across, and the rows that show them.

use orbs_render::DisplayMode;
use orbs_shell::Linear;

/// What a first launch would show, with no session to ask.
///
/// For the dump, which builds no session — and the only answer available to a
/// capture, since `scripts/dumps.sh` runs with `ORBS_SAVE=off`.
#[must_use]
pub(crate) fn default_settings() -> Vec<orbs_shell::settings::Row> {
    settings(&Carried {
        driver: orbs_shell::Driver::default(),
        linear: false,
        mode: DisplayMode::Wide,
        installs: orbs_shell::settings::installs(),
        skip: orbs_shell::settings::skips_boot(),
        motion: orbs_shell::settings::motion_wanted(),
        echo: orbs_shell::settings::terse_echo(),
        name: orbs_shell::settings::chosen_name(),
        pace: orbs_shell::settings::pace(),
        // Plain, as the window's dump defaults are: a capture's row must not
        // move with an exported `ORBS_SIGHT`.
        sight: orbs_shell::Sight::Plain,
    })
}

/// What this build can set, and what each is set to now.
///
/// Shorter than the window's: no tube, phosphor or sound, so no rows for them
/// (§15, `settings::values`).
pub(crate) fn settings(held: &Carried) -> Vec<orbs_shell::settings::Row> {
    use orbs_shell::settings::{Category, ECHO, ECHOES, Row, rows};

    vec![
        // First on the page, as in the window. Here it takes the hue out of the
        // terminal's own colours.
        rows::sight(held.sight),
        rows::linear(held.linear),
        // The row alone, since there is no tube here to be the other switch.
        rows::motion(held.motion),
        Row::new(
            Category::Access,
            ECHO,
            ECHOES[usize::from(held.echo)],
            &ECHOES,
        ),
        rows::reading(held.driver),
        rows::focus(held.mode),
        // From what the session holds, never the file: with nowhere to write,
        // the refresh in `Session::answered` rebuilt this from the file and
        // snapped `skip on` straight back to `off`.
        rows::skip(held.skip),
        rows::install(held.installs),
        rows::pace(held.pace),
        rows::name(held.name.as_deref()),
    ]
}

/// What a swap carries across, because it is the player's and not the tower's.
///
/// `reset_for_swap`'s exceptions, for a build that makes a new `Session` per
/// swap, so the file is not their only memory (§14). Also the rows' source,
/// so the page shows what is in effect.
///
/// `Default` is for tests; a launch starts from [`Carried::remembered`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Carried {
    pub(crate) driver: orbs_shell::Driver,
    /// Whether §14's linear stream is showing. A bool rather than the `Linear`
    /// itself, which owns a scratch `Frame` worth re-allocating per session.
    pub(crate) linear: bool,
    pub(crate) mode: DisplayMode,
    /// Whether a new tower is installed before it wakes.
    pub(crate) installs: bool,
    /// Whether the boot sequence is passed over.
    pub(crate) skip: bool,
    /// Whether anything may move — §14's switch, with no tube beside it here.
    pub(crate) motion: bool,
    /// Whether an echo that only repeats the line is left out.
    pub(crate) echo: bool,
    /// The name a new tower is raised with, if the player chose one.
    pub(crate) name: Option<String>,
    /// How fast text types itself.
    pub(crate) pace: orbs_shell::Pace,
    /// Whether every hue is taken out.
    pub(crate) sight: orbs_shell::Sight,
}

impl Carried {
    /// The reader this session consults.
    pub(crate) const fn driver(&self) -> orbs_shell::Driver {
        self.driver
    }

    /// Honour a setting the menu has written down.
    ///
    /// The one place this build does. `reading` never arrives here: the menu
    /// answers it with `MenuOutcome::Drive`, as it does in the Bevy build.
    pub(crate) fn apply(&mut self, setting: &str, value: &str) {
        let on = orbs_shell::settings::is_on(value);
        match setting {
            orbs_shell::LINEAR_SETTING => self.linear = on,
            orbs_shell::settings::INSTALL => self.installs = on,
            orbs_shell::settings::SKIP => self.skip = on,
            orbs_shell::settings::MOTION => self.motion = on,
            orbs_shell::settings::ECHO => self.echo = value == orbs_shell::settings::ECHOES[1],
            // Asked at the next `new`; a tower already standing keeps its name.
            orbs_shell::settings::NAME => {
                self.name = orbs_shell::settings::checked_name(value)
                    .ok()
                    .filter(|name| !name.is_empty());
            }
            orbs_shell::settings::FOCUS => {
                if let Some(mode) = DisplayMode::named(value) {
                    self.mode = mode;
                }
            }
            orbs_shell::settings::PACE => {
                self.pace = orbs_shell::Pace::named(value).unwrap_or_default();
            }
            orbs_shell::settings::SIGHT => {
                if let Some(sight) = orbs_shell::Sight::parse(value) {
                    self.sight = sight;
                }
            }
            _ => {}
        }
    }

    /// What a first launch starts from: the file, read once.
    ///
    /// §9 opens Wide when nothing is remembered — the mode the Bevy build takes
    /// from `Screen::for_window` — and honours the player's choice otherwise.
    /// `focus` was write-only here until `0.16.12`: this build hard-coded `Wide`
    /// and wrote nothing, against `settings::FOCUS`.
    pub(crate) fn remembered() -> Self {
        Self {
            driver: orbs_shell::settings::driver(),
            linear: Linear::default().showing(),
            mode: orbs_shell::settings::get(orbs_shell::settings::FOCUS)
                .and_then(|word| DisplayMode::named(&word))
                .unwrap_or(DisplayMode::Wide),
            installs: orbs_shell::install_wanted(orbs_shell::settings::installs()),
            skip: orbs_shell::settings::skips_boot(),
            // `ORBS_MOTION` seeds it; the row wins after.
            motion: orbs_shell::settings::motion_wanted(),
            echo: orbs_shell::settings::terse_echo(),
            name: orbs_shell::settings::chosen_name(),
            pace: orbs_shell::settings::pace(),
            // `ORBS_SIGHT`, then the file.
            sight: orbs_shell::Sight::seeded(),
        }
    }
}

#[cfg(test)]
mod rows {
    use super::{Carried, settings};

    #[test]
    fn every_row_is_honoured() {
        // A row this build writes nowhere is §19's dead row. `skip` was one in
        // reverse: read back from the file, so with nowhere to write it snapped
        // to `off`. `reading` is exempt — the menu answers it with `Drive`.
        let held = Carried::default();
        for row in settings(&held) {
            if row.key == orbs_shell::settings::DRIVER {
                continue;
            }
            // A row with no list takes what it is given — the name.
            let next = row.next().unwrap_or("Morgana");
            let mut after = held.clone();
            after.apply(&row.key, next);
            let shown = settings(&after)
                .into_iter()
                .find(|again| again.key == row.key)
                .expect("the row went away");
            // The name row shows what a new tower would take, and an exported
            // `ORBS_WIZARD` outranks the setting there.
            let named =
                (row.key == orbs_shell::settings::NAME).then(|| orbs_shell::new_wizard(Some(next)));
            assert_eq!(
                shown.value,
                named.as_deref().unwrap_or(next),
                "`{}` was set and its row still says `{}`",
                row.word,
                shown.value,
            );
        }
    }
}
