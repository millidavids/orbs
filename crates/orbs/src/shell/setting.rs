//! What this build can set, and what setting it does.
//!
//! [`Row`] is data and the menu draws it; this is the half that knows what a
//! phosphor theme is, how many there are, and which one is on. `values.rs` in
//! `orbs-shell` carries the argument for the split at length — the short of it
//! is that the two builds do not have the same settings, and the menu must not
//! be the place that knows which.
//!
//! Every row is read *from the thing it sets*, never from the file
//! (`Menu::show_driver`'s rule): what a page shows is what is in effect. Built
//! from `orbs-settings.toml` it would show a session choices it did not make — a
//! `ORBS_SAVE=off` run has no file and still has a tube, and `ORBS_CRT=off`
//! outranks whatever is written down.
//!
//! A function key writes back, so `F3` and the tube page are one setting seen
//! twice. The keys still cycle, because that is faster than a menu, and call
//! [`remember`] on the way past.

use bevy::prelude::*;

use orbs_shell::settings::{self, Category, Row, SWITCH, is_on, switched};

use crate::crt::CrtSettings;
use crate::render::{Theme, palette};
use crate::sight::Vision;

/// The word the tube's own state is set by.
pub(crate) const CRT: &str = "crt";
/// The word the phosphor is set by.
pub(crate) const THEME: &str = "theme";
/// The word §14's hue accommodation is set by.
pub(crate) const SIGHT: &str = "sight";
/// The word the linear stream is set by.
///
/// `orbs-shell`'s, because both builds bind `F5` and both must write the same
/// key.
pub(crate) const LINEAR: &str = orbs_shell::LINEAR_SETTING;
/// The word the parse driver is set by — see [`Row::key`] for why it is not
/// `driver`, which is what the file says.
pub(crate) const READING: &str = "reading";
/// The word §9's focus mode is set by.
///
/// `orbs-shell`'s, because both builds bind `F4` and `Screen::default` reads it
/// for both.
pub(crate) const FOCUS: &str = settings::FOCUS;
/// The word the cue volume is set by.
pub(crate) const VOICE: &str = crate::sound::VOICE;
/// The word the bed's volume is set by.
pub(crate) const HUM: &str = crate::sound::HUM;

/// Whether §4's sticky boot skip is on.
///
/// The one setting that had no resource, which cost twice. Reading the *file*
/// through `settings::skips_boot()` meant a blocking read and a TOML parse per
/// frame — `follow_the_keys` rebuilds rows while the menu is open, and at the
/// threshold it cannot be closed, so the game parsed `orbs-settings.toml` sixty
/// times a second, invisibly to every instrument. It also meant a silent no-op
/// wherever the write did not land: the row flipped, nothing applied it, and the
/// next rebuild snapped it back without complaint.
///
/// `Boot::default` still reads the file, once, at startup. Nothing else does.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct Skipping(bool);

impl Skipping {
    /// Whether the boot sequence is skipped.
    pub(crate) const fn on(self) -> bool {
        self.0
    }
}

impl Default for Skipping {
    fn default() -> Self {
        Self(settings::skips_boot())
    }
}

/// Everything a settings row is read from.
///
/// One `SystemParam` because two systems open the menu — `menuing`'s for the
/// `menu` verb and `thresholding`'s for the screen before a tower — and a second
/// copy of this list is a settings page that differs by which door you came
/// through. `input::Surfaces` is the precedent.
///
/// Every field is optional for that module's reason: half the tests here build
/// the shell alone, and a bare `Res` fails parameter validation in one.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct Settable<'w, 's> {
    /// The tube, which is a component on the camera rather than a resource.
    tubes: Query<'w, 's, &'static CrtSettings>,
    theme: Option<Res<'w, Theme>>,
    vision: Option<Res<'w, Vision>>,
    linear: Option<Res<'w, orbs_shell::Linear>>,
    screen: Option<Res<'w, orbs_shell::Screen>>,
    readers: Option<Res<'w, crate::sim::Readers>>,
    levels: Option<Res<'w, crate::sound::Levels>>,
    skipping: Option<Res<'w, Skipping>>,
}

impl Settable<'_, '_> {
    /// Which driver is in effect.
    pub(crate) fn driver(&self) -> orbs_shell::Driver {
        self.readers
            .as_ref()
            .map_or_else(Default::default, |readers| readers.driver())
    }

    /// What this frontend can set, and what each is set to now.
    ///
    /// Handed to the menu when it opens. Only what the Bevy build actually has,
    /// which is nearly everything; `orbs-tui` builds a shorter list and shows
    /// fewer pages.
    pub(crate) fn rows(&self) -> Vec<Row> {
        rows(
            self.tubes.iter().next(),
            self.theme.as_deref(),
            self.vision.as_deref(),
            self.linear.as_deref(),
            self.screen.as_deref(),
            self.driver(),
            self.levels.as_deref().copied(),
            self.skipping.as_deref().copied(),
        )
    }
}

/// What a first launch would show, with no world to ask.
///
/// For the dump, which builds no `App` and so has no tube to read, and for
/// tests. The honest answer as well as the only available one: `dumps.sh` runs
/// with `ORBS_SAVE=off`, so there is no settings file either, and this is what a
/// player sees on a first launch.
#[must_use]
pub(crate) fn defaults() -> Vec<Row> {
    rows(
        None,
        None,
        None,
        None,
        None,
        orbs_shell::Driver::default(),
        None,
        None,
    )
}

/// [`Settable::rows`] with the world already asked.
///
/// Split out so it can be *called with anything*, which is what a test wants —
/// `crt::chosen` and `save::chosen` are the precedent and the reason.
fn rows(
    crt: Option<&CrtSettings>,
    theme: Option<&Theme>,
    vision: Option<&Vision>,
    linear: Option<&orbs_shell::Linear>,
    screen: Option<&orbs_shell::Screen>,
    driver: orbs_shell::Driver,
    levels: Option<crate::sound::Levels>,
    skipping: Option<Skipping>,
) -> Vec<Row> {
    let themes: Vec<&str> = palette::ALL.iter().map(|phosphor| phosphor.name).collect();
    let theme = theme.map_or(palette::ALL[0].name, |theme| theme.0.name);
    let vision = vision.map_or(crate::sight::Sight::Plain, |vision| vision.0);
    let linear = linear.is_some_and(orbs_shell::Linear::showing);
    let mode = screen.map_or(orbs_render::DisplayMode::Wide, |screen| screen.mode);
    let levels = levels.unwrap_or_default();
    vec![
        // Two volumes rather than one, because the reasons to turn each down
        // differ: the hum is atmosphere some find tiring within a minute, and
        // the cues are §14's ambient channel, which is the part a screen-reader
        // player is using. See `sound::levels`.
        Row::new(Category::Sound, VOICE, levels.voice, &settings::LEVELS),
        Row::new(Category::Sound, HUM, levels.hum, &settings::LEVELS),
        Row::new(
            Category::Tube,
            CRT,
            crt.map_or(crate::crt::TUBE_STATES[0].0, |crt| crate::crt::named(crt)),
            &crate::crt::TUBE_STATES.map(|(name, _)| name),
        ),
        Row::new(Category::Tube, THEME, theme, &themes),
        Row::new(
            Category::Access,
            SIGHT,
            vision.name(),
            &crate::sight::Sight::ALL.map(crate::sight::Sight::name),
        ),
        Row::new(Category::Access, LINEAR, switched(linear), &SWITCH),
        // Motion is not a row, deliberately rather than deferred: §14 wants a
        // player who asks for no motion to get none, and `motion::advance`
        // already turns the crossings, the fire and the flare off with the tube.
        // A second switch could be turned on while the tube said off, which is
        // the one state §14 must not be able to reach.
        //
        // A row that reduced motion *without* the tube is a real thing to want
        // and is not a checkbox: it needs the crossings separated from the
        // phosphor, which is a change to `Passing::enabled`'s one input. Named
        // here rather than half-built.
        // `reading` to a player, `driver` in the file: the word says what it
        // does and the key is what `0.13.17` wrote, so no file migrates. See
        // `Row::key`.
        Row::new(
            Category::Habits,
            READING,
            driver.word(),
            &orbs_shell::Driver::ALL.map(orbs_shell::Driver::word),
        )
        .keyed(settings::DRIVER),
        Row::new(
            Category::Habits,
            FOCUS,
            mode.word(),
            &orbs_render::DisplayMode::ALL.map(orbs_render::DisplayMode::word),
        ),
        // §4 has owed this since Phase 0.5: *"skip is a sticky setting, not a
        // per-launch keypress"*, and §19 removed the keypress rather than keep
        // it, because the first thing a player does to the game should not be
        // dismissing it.
        // From the resource, never the file — see [`Skipping`] for the two
        // defects that came of this row reading `orbs-settings.toml` directly.
        Row::new(
            Category::Habits,
            settings::SKIP,
            switched(skipping.unwrap_or_default().on()),
            &SWITCH,
        ),
    ]
}

/// Honour a setting the menu has just written down.
///
/// Now, not at the next launch: a player who turns the tube off and cannot see
/// it go off reasonably thinks nothing happened, which is the dead end §15
/// weighs heaviest arriving through a settings screen.
///
/// `skip` is deliberately absent: it is read by `Boot::default` at startup and
/// there is nothing to do to a sequence that has already finished.
pub(crate) fn apply(world: &mut World, setting: &str, value: &str) {
    match setting {
        CRT => {
            if let Some(state) = crate::crt::state_named(value) {
                let mut tubes = world.query::<&mut CrtSettings>();
                for mut settings in tubes.iter_mut(world) {
                    *settings = state;
                }
            }
        }
        THEME => {
            if let Some(phosphor) = palette::ALL
                .iter()
                .find(|phosphor| phosphor.name == value)
                .copied()
                && let Some(mut theme) = world.get_resource_mut::<Theme>()
            {
                theme.0 = phosphor;
            }
        }
        SIGHT => {
            if let Some(sight) = crate::sight::Sight::parse(value)
                && let Some(mut vision) = world.get_resource_mut::<Vision>()
            {
                vision.0 = sight;
            }
        }
        LINEAR => {
            if let Some(mut linear) = world.get_resource_mut::<orbs_shell::Linear>() {
                linear.show(is_on(value));
            }
        }
        FOCUS => {
            if let Some(mode) = orbs_render::DisplayMode::named(value)
                && let Some(mut screen) = world.get_resource_mut::<orbs_shell::Screen>()
            {
                screen.mode = mode;
            }
        }
        // `skip` has an arm now, for every other row's reason: nothing takes
        // effect this session, but the *row* has to become true or
        // `follow_the_keys` rebuilds it from a resource that never heard.
        settings::SKIP => {
            if let Some(mut skipping) = world.get_resource_mut::<Skipping>() {
                *skipping = Skipping(is_on(value));
            }
        }
        VOICE | HUM => {
            if let Some(mut levels) = world.get_resource_mut::<crate::sound::Levels>() {
                // `set` decides whether the word is one this build knows, so a
                // level from a later build's settings file leaves the current
                // one alone rather than muting the game.
                levels.set(setting, value);
            }
        }
        // `reading` never arrives here: the menu answers it with
        // `MenuOutcome::Drive`, which `menuing` acts on, because *stop
        // consulting a reader now* is not something a generic setting can say.
        _ => {}
    }
}

/// Write a setting down because a function key changed it.
///
/// The other half of *one setting seen twice*: `F3` cycles the tube, and this
/// stops the settings page disagreeing with it on the next launch.
pub(crate) fn remember(setting: &str, value: &str) {
    settings::set(setting, value);
}

/// Keep an open settings page telling the truth.
///
/// The rows are a snapshot taken once by `Standing::open`, and no function key
/// is gated on the menu being shut — so `F3` over an open tube page steps the
/// tube to `peak` while the row still says `default`. Worse than cosmetic:
/// typing `crt` then cycles from the stale value, lands back on `peak`, and the
/// keystroke does nothing anybody can see.
///
/// Refreshed rather than locking the keys out, because a menu is exactly where
/// somebody reaches for `F3`. Only while the menu is up and only when something
/// moved — the comparison keeps change detection on `Standing` quiet, so this
/// does not drag the painter to frame rate for a page nobody is touching.
pub(crate) fn follow_the_keys(settable: Settable, mut standing: ResMut<crate::shell::Standing>) {
    let fresh = settable.rows();
    let Some(menu) = standing.bypass_change_detection().get_mut() else {
        return;
    };
    if menu.settings() == fresh.as_slice() {
        return;
    }
    menu.show_settings(fresh);
    standing.set_changed();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_value_on_every_row_is_reachable_by_a_prefix() {
        // The lint that has to live here, because the values come from this
        // frontend's palette and `orbs-shell` cannot see them. `palette::ALL` is
        // `[amber, green, muted violet, monochrome]`, and while `value_named`
        // took the first prefix match, `theme m` meant muted violet and
        // monochrome — §14's accessibility phosphor — was unreachable by its
        // first letter. Now `m` is refused, and this holds that some prefix
        // still reaches every value.
        for row in defaults() {
            for value in &row.values {
                let reached = (1..=value.len())
                    .map(|len| &value[..len])
                    .any(|prefix| row.value_named(prefix) == Some(value.as_str()));
                assert!(
                    reached,
                    "`{}` on the `{}` row cannot be reached by any prefix of itself",
                    value, row.word,
                );
            }
        }
    }

    #[test]
    fn no_settings_page_is_taller_than_the_menu_can_draw() {
        // The check `orbs-shell` cannot make: its own version compared two
        // constants in the same file and could not fail. These rows are this
        // build's, assembled from what it has, and this crate is the only one
        // that can count them.
        //
        // `menu/paint.rs` refuses to draw below `MIN_ROWS`, sized for the
        // tallest page — so a page with more rows than fit is one whose last
        // setting a player at the 80×22 floor simply cannot see, which is §15's
        // dead end with a scrollbar-shaped hole where the fix would go.
        for page in orbs_shell::settings::Category::ALL {
            let rows = defaults().iter().filter(|row| row.page == page).count();
            assert!(
                rows <= orbs_shell::SETTINGS_ROWS as usize,
                "the `{}` page has {rows} rows and the menu draws {}",
                page.word(),
                orbs_shell::SETTINGS_ROWS,
            );
        }
    }

    #[test]
    fn no_row_offers_a_value_it_is_not_set_to_one_of() {
        // A page shows `[value]` from the row, and cycling starts from it — so a
        // current value outside its own list makes `Row::next` wrap from the
        // start rather than step, which reads as the first keystroke doing
        // nothing.
        for row in defaults() {
            assert!(
                row.values.contains(&row.value),
                "the `{}` row reads `{}`, which is not one of {:?}",
                row.word,
                row.value,
                row.values,
            );
        }
    }
}
