//! The menu, as cells.
//!
//! Rule 2: this decides what appears and where, and a frontend decides only how
//! a cell is drawn. Everything here is authored prose or something
//! [`Menu`](super::Menu) was handed.

use orbs_render::{Frame, Painter, Pos, Rect, Span, Style};
use orbs_sim::Prose;
use orbs_sim::content::Length;

use super::state::{Complaint, Menu, Page};
use super::words::offered;
use crate::settings::Category;

/// [`crate::save::SLOTS`] as rows.
///
/// Written out rather than cast, and asserted equal in
/// `the_floor_fits_the_tallest_page` — `as` from `usize` is a truncation clippy
/// will not take on faith, and this is the one number the floor below depends
/// on.
const SLOT_ROWS: u16 = 6;

/// Rows the tallest page needs: the play page, which is one row per slot, then
/// a blank, `new`, `abandon` and the way back.
const TALLEST: u16 = SLOT_ROWS + 4;

/// The smallest pane this can honestly be drawn in.
///
/// Below it the pane says it is too small, since a choice you cannot see is one
/// you do not have. Sized for the tallest page, so the menu never stops fitting
/// as you move.
const MIN_ROWS: u16 = 7 + TALLEST;

/// How many settings rows a page has room for at the floor.
///
/// Rows, a blank, the how-to line, and the way back — so three of the page's
/// height is furniture.
///
/// Published because the rows are the frontend's: a `Row` is assembled from what
/// that build has, so this crate cannot count them and the frontend can, against
/// this. See `no_settings_page_is_taller_than_the_menu_can_draw`.
pub const SETTINGS_ROWS: u16 = TALLEST - 3;
const MIN_COLS: u16 = 34;

/// Draw the menu into `pane`.
pub fn paint(frame: &mut Frame, menu: &Menu, pane: Rect, prose: &Prose) {
    // Two forms, because the advice differs: over a tower the answer is `F4`,
    // since one pane is taller than two. At the threshold there is no second
    // pane to fold away, so saying `F4` would be a dead end on the only screen
    // there is.
    let too_small = if menu.stance().is_threshold() {
        "menu_too_small_threshold"
    } else {
        "menu_too_small"
    };
    crate::framed::paint(
        frame,
        pane,
        &prose.line("menu_title", &[]),
        (MIN_ROWS, MIN_COLS),
        &prose.line(too_small, &[]),
        |painter, inner| body(painter, menu, inner, prose),
    );
}

/// The menu's rows, and where the caret ends up.
fn body(painter: &mut Painter<'_>, menu: &Menu, inner: Rect, prose: &Prose) -> Pos {
    let lead = match menu.page() {
        // The top page says where you are standing; the other three need not,
        // because "which tower?" and "how long a game?" are the same question
        // over a tower and in front of none.
        Page::Choices if menu.stance().is_threshold() => "menu_lead_threshold",
        Page::Choices => "menu_lead",
        Page::Play => "menu_play_lead",
        Page::Lengths => "menu_new_lead",
        Page::Settings => "menu_settings_lead",
        Page::Setting(page) => return_lead(page),
    };
    let mut y = inner.row;
    painter.span(
        Pos::new(inner.col, y),
        &Span::new(&prose.line(lead, &[])).with_style(Style::DIM),
    );
    y = y.saturating_add(2);

    y = match menu.page() {
        Page::Choices => choices(painter, menu, inner, y, prose),
        Page::Play => listing(painter, menu, inner, y, prose),
        Page::Lengths => lengths(painter, inner, y, prose),
        Page::Settings => categories(painter, menu, inner, y, prose),
        Page::Setting(page) => rows(painter, menu, page, inner, y, prose),
    };
    y = y.saturating_add(1);

    // The line, with a caret. Unlike the weave — which has a browsing mode and
    // draws none — there is exactly one thing to do here and it is typing, so
    // the caret is always honest.
    let complaint = menu.complaint().map(|complaint| said(complaint, prose));
    crate::framed::typed_line(painter, inner, y, menu.command(), complaint.as_deref())
}

/// The top page's words — the ones this stance offers.
///
/// Drawn from the same list [`word`] matches against, so the listing and the
/// vocabulary cannot disagree. Filtering only here would leave `resume` typeable
/// at a threshold that does not draw it.
///
/// [`word`]: super::words::word
fn choices(painter: &mut Painter<'_>, menu: &Menu, inner: Rect, mut y: u16, prose: &Prose) -> u16 {
    for (name, _) in offered(menu.stance()) {
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&gloss(name, menu, prose)),
        );
        y = y.saturating_add(1);
    }
    y
}

/// What a top-page word says it does, in this stance.
///
/// A `_threshold` key, looked for and fallen through — `Prose::counted`'s shape,
/// and here for its reason: a word that reads wrong in one stance is authored
/// twice and everything else stays one line. `new` is the word it was written
/// for, since *"raise another"* is a small lie in front of no tower.
fn gloss(name: &str, menu: &Menu, prose: &Prose) -> String {
    if menu.stance().is_threshold() {
        let keyed = format!("menu_word_{name}_threshold");
        if prose.has(&keyed) {
            return prose.line(&keyed, &[]);
        }
    }
    prose.line(&format!("menu_word_{name}"), &[])
}

/// The towers the orb is keeping, one row each — and the way to raise another.
///
/// Both answers on one page; `new` is drawn even with no towers, a first
/// launch's only move.
fn listing(painter: &mut Painter<'_>, menu: &Menu, inner: Rect, mut y: u16, prose: &Prose) -> u16 {
    // Only when there *could* be towers: a session that keeps none says so
    // through `Complaint::Unkept` below, and *no towers yet* on top of that
    // reads as a first launch. See `Menu::keeps_towers`.
    if menu.saves().is_empty() && menu.keeps_towers() {
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line("menu_saves_none", &[])).with_style(Style::DIM),
        );
        y = y.saturating_add(1);
    }
    for save in menu.saves() {
        // A slot this build cannot read is drawn as that rather than left out.
        // Dropping it made the menu say two false things at once: the slot
        // looked empty, and its number answered *"there is no tower in 3"*
        // about a file sitting right there.
        let row = match &save.held {
            Some(held) => prose.line(
                "menu_saves_row",
                &[
                    ("count", &save.slot.to_string()),
                    ("name", &held.wizard),
                    ("detail", held.length.word()),
                    ("quantity", &held.experience.to_string()),
                ],
            ),
            None => prose.line(
                "menu_saves_unreadable",
                &[("count", &save.slot.to_string())],
            ),
        };
        let span = Span::new(&row);
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &if save.is_readable() {
                span
            } else {
                span.with_style(Style::DIM)
            },
        );
        y = y.saturating_add(1);
    }
    y = y.saturating_add(1);
    painter.span(
        Pos::new(inner.col.saturating_add(2), y),
        &Span::new(&prose.line("menu_word_new", &[])),
    );
    // `abandon` is drawn only when there is something to abandon — the one place
    // this page hides a word it still answers to. It needs a slot number, so
    // offering it over an empty listing offers a sentence with no argument.
    if !menu.saves().is_empty() {
        y = y.saturating_add(1);
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line("menu_word_abandon", &[])).with_style(Style::DIM),
        );
    }
    back(painter, inner, y.saturating_add(1), prose)
}

/// How long a new game should be.
fn lengths(painter: &mut Painter<'_>, inner: Rect, mut y: u16, prose: &Prose) -> u16 {
    for length in Length::OFFERED {
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line(&format!("menu_length_{}", length.word()), &[])),
        );
        y = y.saturating_add(1);
    }
    back(painter, inner, y.saturating_add(1), prose)
}

/// Which parts of the orb there are to set.
///
/// Only the pages this frontend has anything on, which is how *unsupported* is
/// expressed — see [`Menu::pages`](super::Menu::pages). `orbs-tui` has no tube,
/// so it offers no `tube` page rather than a page of controls that do nothing.
fn categories(
    painter: &mut Painter<'_>,
    menu: &Menu,
    inner: Rect,
    mut y: u16,
    prose: &Prose,
) -> u16 {
    for page in menu.pages() {
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line(&format!("menu_settings_{}", page.word()), &[])),
        );
        y = y.saturating_add(1);
    }
    back(painter, inner, y.saturating_add(1), prose)
}

/// One page of settings, each row saying what it is set to.
///
/// The value is on the row rather than a page below it: a settings page that
/// does not say what is currently true is a list of things you might already
/// have done, and a page *per setting* would be a third level for a question
/// with two answers.
fn rows(
    painter: &mut Painter<'_>,
    menu: &Menu,
    page: Category,
    inner: Rect,
    mut y: u16,
    prose: &Prose,
) -> u16 {
    for row in menu.rows(page) {
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line(
                "menu_setting_row",
                &[
                    ("name", &prose.line(&format!("menu_set_{}", row.word), &[])),
                    ("state", &row.value),
                ],
            )),
        );
        y = y.saturating_add(1);
    }
    y = y.saturating_add(1);
    painter.span(
        Pos::new(inner.col.saturating_add(2), y),
        &Span::new(&prose.line("menu_setting_how", &[])).with_style(Style::DIM),
    );
    back(painter, inner, y.saturating_add(1), prose)
}

/// The lead line for one settings page.
const fn return_lead(page: Category) -> &'static str {
    match page {
        Category::Sound => "menu_sound_lead",
        Category::Tube => "menu_tube_lead",
        Category::Picture => "menu_picture_lead",
        Category::Access => "menu_access_lead",
        Category::Habits => "menu_habits_lead",
    }
}

/// The way out, which every inner page ends with.
fn back(painter: &mut Painter<'_>, inner: Rect, y: u16, prose: &Prose) -> u16 {
    painter.span(
        Pos::new(inner.col.saturating_add(2), y),
        &Span::new(&prose.line("menu_word_back", &[])).with_style(Style::DIM),
    );
    y.saturating_add(1)
}

/// A refusal, as a sentence. §6: never a bare error.
///
/// [`Complaint::Abandoning`] is the one that is not a refusal — it is the
/// question `abandon` asks — and it is drawn here because it is drawn *where*
/// the refusals are and lasts exactly as long as one.
fn said(complaint: &Complaint, prose: &Prose) -> String {
    match complaint {
        Complaint::Unknown(word) => prose.line("menu_unknown", &[("detail", word)]),
        Complaint::Empty(slot) => prose.line("menu_empty", &[("count", &slot.to_string())]),
        Complaint::Unreadable(slot) => {
            prose.line("menu_unreadable", &[("count", &slot.to_string())])
        }
        Complaint::Abandoning(slot) => {
            prose.line("menu_abandoning", &[("count", &slot.to_string())])
        }
        Complaint::Unabandoned(slot) => {
            prose.line("menu_unabandoned", &[("count", &slot.to_string())])
        }
        Complaint::Full => prose.line("menu_full", &[]),
        Complaint::Unwritten => prose.line("menu_unwritten", &[]),
        Complaint::Unkept => prose.line("menu_unkept", &[]),
        Complaint::Warning => prose.line("menu_warning", &[]),
        Complaint::Unnamed(fault) => match fault {
            crate::settings::NameFault::Long => prose.line(
                "menu_name_long",
                &[("count", &crate::settings::LONGEST_NAME.to_string())],
            ),
            crate::settings::NameFault::Spaced => prose.line("menu_name_spaced", &[]),
            crate::settings::NameFault::Undrawable(at) => {
                prose.line("menu_name_undrawable", &[("count", &at.to_string())])
            }
        },
        Complaint::Remark(key) => prose.line(key, &[]),
        Complaint::Moon(reading) => moon(*reading, prose),
    }
}

/// What `moon` says: the phase, and the night if it is one worth naming.
fn moon(reading: crate::sky::Reading, prose: &Prose) -> String {
    use crate::sky::{Occasion, Phase, Reading};

    let Reading::Seen(phase, occasion) = reading else {
        return prose.line("moon_unseen", &[]);
    };
    let sky = match phase {
        Phase::New => prose.line("moon_new", &[]),
        Phase::Full => prose.line("moon_full", &[]),
        Phase::Waxing(nights) => prose.line("moon_waxing", &[("count", &nights.to_string())]),
        Phase::Waning(nights) => prose.line("moon_waning", &[("count", &nights.to_string())]),
    };
    let night = match occasion {
        Occasion::Ordinary => return sky,
        Occasion::Hallows => prose.line("moon_hallows", &[]),
        Occasion::Unlucky => prose.line("moon_unlucky", &[]),
    };
    prose.line("moon_and", &[("detail", &sky), ("name", &night)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::words::COUNT;

    #[test]
    fn the_floor_fits_the_tallest_page() {
        // `SLOT_ROWS` is `save::SLOTS` written as a `u16`; if they drift, the
        // listing grows a row the floor did not reserve and the last tower falls
        // off the bottom of a short pane.
        assert_eq!(usize::from(SLOT_ROWS), crate::save::SLOTS);
        const {
            assert!(
                TALLEST >= COUNT,
                "the top page is taller than the floor was sized for",
            );
        }
        let lengths = u16::try_from(Length::OFFERED.len()).expect("three of them");
        assert!(TALLEST >= lengths + 2, "the lengths page does not fit");

        // The settings page lists the categories, plus a blank and the way back.
        let pages = u16::try_from(Category::ALL.len()).expect("three of them");
        assert!(TALLEST >= pages + 2, "the settings page does not fit");
    }

    /// The floor holds for a settings page, whatever a frontend puts on one.
    ///
    /// The one page whose height is not ours, because the rows come from the
    /// frontend: eight phosphor themes are still *one* row, since the value
    /// cycles in place, but eight *settings* on one page would not fit. This
    /// names the ceiling so the day one does, it fails here.
    #[test]
    fn a_settings_page_may_hold_this_many_rows() {
        // The arithmetic only; the rows are checked in `orbs`
        // (`no_settings_page_is_taller_than_the_menu_can_draw`). A `const`
        // block, since a runtime `assert!` over constants tests nothing.
        const {
            assert!(
                SETTINGS_ROWS == TALLEST - 3,
                "the published capacity and the floor disagree",
            );
            assert!(
                SETTINGS_ROWS >= 5,
                "a settings page has room for fewer rows than a frontend is likely to build",
            );
        }
    }
}
