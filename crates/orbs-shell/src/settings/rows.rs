//! The rows both frontends offer, each built one way from the value in effect.
//!
//! Where the value comes from is the frontend's — a Bevy resource, a field the
//! terminal's session carries — and the row is the shell's, so the two pages
//! cannot spell a setting differently.

use orbs_render::DisplayMode;

use super::keys::{DRIVER, FOCUS, INSTALL, MOTION, PACE, SIGHT, SKIP};
use super::name::NAME;
use super::values::{Category, Row, SWITCH, switched};
use crate::{Driver, Pace, Sight};

/// The word the parse driver is set by. `reading` to a player and `driver` in
/// the file: the word says what it does, and the key is what `0.13.17` wrote,
/// so no file migrates. See `Row::key`.
pub const READING: &str = "reading";

/// §14's hue accommodation.
#[must_use]
pub fn sight(sight: Sight) -> Row {
    Row::new(
        Category::Access,
        SIGHT,
        sight.name(),
        &Sight::ALL.map(Sight::name),
    )
}

/// §14's linear stream.
#[must_use]
pub fn linear(on: bool) -> Row {
    Row::new(
        Category::Access,
        crate::LINEAR_SETTING,
        switched(on),
        &SWITCH,
    )
}

/// Whether things move: crossings, rolls, the bench.
#[must_use]
pub fn motion(on: bool) -> Row {
    Row::new(Category::Access, MOTION, switched(on), &SWITCH)
}

/// How the orb reads a line it cannot read itself.
#[must_use]
pub fn reading(driver: Driver) -> Row {
    Row::new(
        Category::Habits,
        READING,
        driver.word(),
        &Driver::ALL.map(Driver::word),
    )
    .keyed(DRIVER)
}

/// §9's focus mode.
#[must_use]
pub fn focus(mode: DisplayMode) -> Row {
    Row::new(
        Category::Habits,
        FOCUS,
        mode.word(),
        &DisplayMode::ALL.map(DisplayMode::word),
    )
}

/// §4's sticky boot skip.
#[must_use]
pub fn skip(on: bool) -> Row {
    Row::new(Category::Habits, SKIP, switched(on), &SWITCH)
}

/// Whether a new tower is installed first.
#[must_use]
pub fn install(on: bool) -> Row {
    Row::new(Category::Habits, INSTALL, switched(on), &SWITCH)
}

/// How fast text types itself.
#[must_use]
pub fn pace(pace: Pace) -> Row {
    Row::new(
        Category::Habits,
        PACE,
        pace.word(),
        &Pace::ALL.map(Pace::word),
    )
}

/// The name a new tower is raised with. No list: it takes what it is given,
/// and shows what a new tower would take.
#[must_use]
pub fn name(chosen: Option<&str>) -> Row {
    Row::new(Category::Habits, NAME, &crate::new_wizard(chosen), &[])
}
