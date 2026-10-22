//! The rows, read from what holds each setting.

use bevy::prelude::*;

use orbs_shell::settings::{self, Category, Pair, Row, SWITCH, switched};

use super::held::{CRT, ELEVEN, FOCUS, LINEAR, Naming, READING, SIGHT, Skipping, THEME, VOICE};
use crate::crt::CrtSettings;
use crate::render::{Theme, palette};
use crate::sight::Vision;

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
    installs: Option<Res<'w, crate::shell::Installs>>,
    stillness: Option<Res<'w, crate::shell::Stillness>>,
    picture: Option<Res<'w, crate::crt::Picture>>,
    contrast: Option<Res<'w, crate::render::Contrast>>,
    weight: Option<Res<'w, crate::render::Weight>>,
    paced: Option<Res<'w, crate::shell::revealing::Paced>>,
    echoing: Option<Res<'w, crate::shell::echoing::Echoing>>,
    kinds: Option<Res<'w, crate::sound::Kinds>>,
    away: Option<Res<'w, crate::shell::Away>>,
    naming: Option<Res<'w, Naming>>,
}

/// Everything a settings row is read from, already asked.
///
/// Fields rather than arguments, so a new row is a name rather than one more
/// `None` in a list nobody can tell apart.
#[derive(Clone, Copy, Default)]
struct Readings<'a> {
    crt: Option<&'a CrtSettings>,
    theme: Option<&'a Theme>,
    vision: Option<&'a Vision>,
    linear: Option<&'a orbs_shell::Linear>,
    screen: Option<&'a orbs_shell::Screen>,
    driver: orbs_shell::Driver,
    levels: Option<crate::sound::Levels>,
    skipping: Option<Skipping>,
    installs: Option<crate::shell::Installs>,
    stillness: Option<crate::shell::Stillness>,
    picture: Option<crate::crt::Picture>,
    contrast: Option<crate::render::Contrast>,
    weight: Option<crate::render::Weight>,
    paced: Option<crate::shell::revealing::Paced>,
    echoing: Option<crate::shell::echoing::Echoing>,
    kinds: Option<crate::sound::Kinds>,
    away: Option<crate::shell::Away>,
    naming: Option<&'a Naming>,
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
        rows(Readings {
            crt: self.tubes.iter().next(),
            theme: self.theme.as_deref(),
            vision: self.vision.as_deref(),
            linear: self.linear.as_deref(),
            screen: self.screen.as_deref(),
            driver: self.driver(),
            levels: self.levels.as_deref().copied(),
            skipping: self.skipping.as_deref().copied(),
            installs: self.installs.as_deref().copied(),
            stillness: self.stillness.as_deref().copied(),
            picture: self.picture.as_deref().copied(),
            contrast: self.contrast.as_deref().copied(),
            weight: self.weight.as_deref().copied(),
            paced: self.paced.as_deref().copied(),
            echoing: self.echoing.as_deref().copied(),
            kinds: self.kinds.as_deref().copied(),
            away: self.away.as_deref().copied(),
            naming: self.naming.as_deref(),
        })
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
    rows(Readings::default())
}

/// [`Settable::rows`] with the world already asked.
///
/// Split out so it can be *called with anything*, which is what a test wants —
/// `crt::chosen` and `save::chosen` are the precedent and the reason.
fn rows(read: Readings) -> Vec<Row> {
    let themes: Vec<&str> = palette::ALL.iter().map(|phosphor| phosphor.name).collect();
    let theme = read
        .theme
        .map_or(palette::ALL[0].name, |theme| theme.0.name);
    let vision = read
        .vision
        .map_or(crate::sight::Sight::Plain, |vision| vision.0);
    let linear = read.linear.is_some_and(orbs_shell::Linear::showing);
    let mode = read
        .screen
        .map_or(orbs_render::DisplayMode::Wide, |screen| screen.mode);
    let fit = read
        .screen
        .map_or_else(orbs_shell::Fit::default, |screen| screen.fit);
    let levels = read.levels.unwrap_or_default();
    let still = read.stillness.unwrap_or_default().still();
    let picture = read.picture.unwrap_or_default();
    let kinds = read.kinds.unwrap_or_default();
    let strengths = crate::crt::Strength::ALL.map(crate::crt::Strength::word);
    let mut rows = vec![
        // `eleven` is `full`, and the orb says so.
        Row::new(Category::Sound, VOICE, levels.voice, &settings::LEVELS).secretly(
            ELEVEN,
            "full",
            Some("menu_secret_eleven"),
        ),
    ];
    // How loud is `voice`; whether each kind of sound plays at all is a switch
    // of its own, under the volume.
    rows.extend(crate::sound::Kind::ALL.map(|kind| {
        Row::new(
            Category::Sound,
            kind.word(),
            switched(kinds.allows(kind)),
            &SWITCH,
        )
    }));
    rows.extend([
        Row::new(
            Category::Tube,
            CRT,
            read.crt
                .map_or(crate::crt::TUBE_STATES[0].0, |crt| crate::crt::named(crt)),
            &crate::crt::TUBE_STATES.map(|(name, _)| name),
        ),
        // The phosphors by number, never offered: `theme p3` is amber, and
        // `theme p7` is a radar tube no list shows.
        palette::NUMBERED.iter().fold(
            Row::new(Category::Tube, THEME, theme, &themes),
            |row, (number, phosphor)| row.secretly(number, phosphor, Some("menu_secret_phosphor")),
        ),
        Row::new(
            Category::Tube,
            settings::SCALE,
            fit.word(),
            &orbs_shell::Fit::ALL.map(orbs_shell::Fit::word),
        ),
        Row::new(
            Category::Access,
            SIGHT,
            vision.name(),
            &crate::sight::Sight::ALL.map(crate::sight::Sight::name),
        ),
        Row::new(Category::Access, LINEAR, switched(linear), &SWITCH),
        // Its own switch, and never one that can bring motion back: `moving`
        // asks this *and* the tube, so `motion on` with the tube off is still.
        Row::new(
            Category::Access,
            settings::MOTION,
            switched(!still),
            &SWITCH,
        ),
        read.contrast.unwrap_or_default().row(Category::Access),
        read.weight.unwrap_or_default().row(Category::Access),
        read.echoing.unwrap_or_default().row(Category::Access),
        // `reading` to a player, `driver` in the file: the word says what it
        // does and the key is what `0.13.17` wrote, so no file migrates. See
        // `Row::key`.
        Row::new(
            Category::Habits,
            READING,
            read.driver.word(),
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
            switched(read.skipping.unwrap_or_default().on()),
            &SWITCH,
        ),
        // From the resource, for `skip`'s reason. With none — the dump — it is
        // the file's answer, which on a first launch is on.
        Row::new(
            Category::Habits,
            settings::INSTALL,
            switched(
                read.installs
                    .map_or_else(settings::installs, |installs| installs.0),
            ),
            &SWITCH,
        ),
        Row::new(
            Category::Habits,
            settings::PACE,
            read.paced.unwrap_or_default().0.word(),
            &orbs_shell::Pace::ALL.map(orbs_shell::Pace::word),
        ),
        read.away.unwrap_or_default().row(Category::Habits),
        // No list: it takes what it is given, and shows what a new tower would
        // take.
        Row::new(
            Category::Habits,
            settings::NAME,
            &orbs_shell::new_wizard(
                read.naming
                    .map_or_else(settings::chosen_name, |naming| naming.0.clone())
                    .as_deref(),
            ),
            &[],
        ),
    ]);
    // One row per effect, each a ceiling on whatever state `crt` chose.
    rows.extend(crate::crt::Effect::ALL.map(|effect| {
        Row::new(
            Category::Picture,
            effect.word(),
            picture.of(effect).word(),
            &strengths,
        )
    }));
    rows
}
