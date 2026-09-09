//! The menu, as cells.
//!
//! Rule 2: this decides what appears and where, and a frontend decides only how
//! a cell is drawn. Everything here is authored prose or something
//! [`Menu`](super::Menu) was handed.

use orbs_render::{Frame, Painter, Pos, Rect, Span, Style};
use orbs_sim::Prose;
use orbs_sim::content::Length;

use super::state::{Complaint, Driver, Menu, Page};
use super::words::WORDS;

/// [`crate::save::SLOTS`] as rows.
///
/// Written out rather than cast, and asserted equal in
/// `the_floor_fits_the_tallest_page` — `as` from `usize` is a truncation clippy
/// will not take on faith, and this is the one number the floor below depends
/// on.
const SLOT_ROWS: u16 = 6;

/// Rows the tallest page needs: the saves listing, which is one row per slot,
/// then a blank and the way back.
const TALLEST: u16 = SLOT_ROWS + 2;

/// The smallest pane this can honestly be drawn in.
///
/// Two borders, the lead, a blank, the page, a blank, the line, and a row under
/// it for a complaint. Below this it says the pane is too small rather than
/// drawing a menu with a choice missing from it — the answer the weave already
/// gives, and the same reason: **a choice you cannot see is one you do not
/// have**, and one of these choices is how you get out.
///
/// **Sized for the tallest page rather than the one showing**, so the menu does
/// not fit when you open it and stop fitting when you ask for the listing.
const MIN_ROWS: u16 = 7 + TALLEST;
const MIN_COLS: u16 = 34;

/// The caret's lead-in, and what the typed line is indented by.
const LEAD: &str = "> ";

/// Draw the menu into `pane`.
///
/// **The caret is set after the painter is done**, the way `sheet::paint` hands
/// one back: a `Painter` holds the `Frame`, so the two cannot be reached at
/// once.
pub fn paint(frame: &mut Frame, menu: &Menu, pane: Rect, prose: &Prose) {
    if pane.is_empty() {
        return;
    }
    let caret = {
        let mut painter = frame.painter(pane);
        let area = painter.area();
        painter.border(area, Some(&prose.line("menu_title", &[])), Style::DIM);
        let inner = area.inset(1);
        if inner.is_empty() {
            return;
        }
        if area.rows < MIN_ROWS || area.cols < MIN_COLS {
            painter.paragraph(inner, &Span::new(&prose.line("menu_too_small", &[])));
            None
        } else {
            Some(body(&mut painter, menu, inner, prose))
        }
    };
    // `None` when the pane is too small to type into: a caret is the game's one
    // promise about where a keystroke lands, and drawing it over a message that
    // says there is no room would be a lie about exactly that.
    frame.set_cursor(caret);
}

/// The menu's rows, and where the caret ends up.
fn body(painter: &mut Painter<'_>, menu: &Menu, inner: Rect, prose: &Prose) -> Pos {
    let lead = match menu.page() {
        Page::Choices => "menu_lead",
        Page::Saves => "menu_saves_lead",
        Page::Lengths => "menu_new_lead",
        Page::Options => "menu_options_lead",
    };
    let mut y = inner.row;
    painter.span(
        Pos::new(inner.col, y),
        &Span::new(&prose.line(lead, &[])).with_style(Style::DIM),
    );
    y = y.saturating_add(2);

    y = match menu.page() {
        Page::Choices => choices(painter, inner, y, prose),
        Page::Saves => listing(painter, menu, inner, y, prose),
        Page::Lengths => lengths(painter, inner, y, prose),
        Page::Options => drivers(painter, menu, inner, y, prose),
    };
    y = y.saturating_add(1);

    // The line, with a caret. Unlike the weave — which has a browsing mode and
    // draws none — there is exactly one thing to do here and it is typing, so
    // the caret is always honest.
    painter.span(
        Pos::new(inner.col, y),
        &Span::new(&format!("{LEAD}{}", menu.command())),
    );

    if let Some(complaint) = menu.complaint() {
        painter.span(
            Pos::new(inner.col, y.saturating_add(1)),
            &Span::new(&said(complaint, prose)).with_style(Style::DIM),
        );
    }

    let typed = LEAD
        .chars()
        .count()
        .saturating_add(menu.command().chars().count());
    Pos::new(
        inner
            .col
            .saturating_add(u16::try_from(typed).unwrap_or(u16::MAX)),
        y,
    )
}

/// The top page's words.
fn choices(painter: &mut Painter<'_>, inner: Rect, mut y: u16, prose: &Prose) -> u16 {
    for (name, _) in WORDS {
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line(&format!("menu_word_{name}"), &[])),
        );
        y = y.saturating_add(1);
    }
    y
}

/// The towers the orb is keeping, one row each.
fn listing(painter: &mut Painter<'_>, menu: &Menu, inner: Rect, mut y: u16, prose: &Prose) -> u16 {
    if menu.saves().is_empty() {
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line("menu_saves_none", &[])),
        );
        y = y.saturating_add(1);
    }
    for save in menu.saves() {
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line(
                "menu_saves_row",
                &[
                    ("count", &save.slot.to_string()),
                    ("name", &save.wizard),
                    ("detail", save.length.word()),
                    ("quantity", &save.experience.to_string()),
                ],
            )),
        );
        y = y.saturating_add(1);
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

/// How the orb reads a line, and which way it is set.
///
/// **The chosen one is marked rather than merely listed.** A settings page that
/// does not say what is currently true is a list of things you might already
/// have done, which is §15's dead end in miniature.
fn drivers(painter: &mut Painter<'_>, menu: &Menu, inner: Rect, mut y: u16, prose: &Prose) -> u16 {
    for driver in Driver::ALL {
        let mark = if driver == menu.driver() {
            "menu_driver_on"
        } else {
            "menu_driver_off"
        };
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line(
                mark,
                &[(
                    "detail",
                    &prose.line(&format!("menu_driver_{}", driver.word()), &[]),
                )],
            )),
        );
        y = y.saturating_add(1);
    }
    back(painter, inner, y.saturating_add(1), prose)
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
fn said(complaint: &Complaint, prose: &Prose) -> String {
    match complaint {
        Complaint::Unknown(word) => prose.line("menu_unknown", &[("detail", word)]),
        Complaint::Empty(slot) => prose.line("menu_empty", &[("count", &slot.to_string())]),
        Complaint::Full => prose.line("menu_full", &[]),
        Complaint::Unkept => prose.line("menu_unkept", &[]),
    }
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

        // The options page is the newest and the shortest; it is asserted for
        // the same reason as the others, so a third driver cannot quietly
        // outgrow the floor.
        let drivers = u16::try_from(Driver::ALL.len()).expect("two of them");
        assert!(TALLEST >= drivers + 2, "the options page does not fit");
    }
}
