//! The laboratory's instrument panel — §10.1's permanent fixture.
//!
//! Four bars, drawn beside or above the transcript whenever the player is
//! standing in the laboratory. DESIGN.md §10.1: *"with four instruments running
//! you watch and respond"*, and a transcript that scrolls the state away is not
//! something anyone can watch.
//!
//! # Why it earns its space
//!
//! Twice a piece of laboratory state was reported as a bug because the only way
//! to see it was to touch it: banked fuel read as *"stopping got rid of the
//! charcoal"*, and a fouled instrument read as *"it will not start"*. Both were
//! answered with a sentence in `content/prose.toml`. A sentence tells you once,
//! when you ask. A bar tells you continuously, without asking.
//!
//! # It follows the shape of the pane
//!
//! A pane wider than it is tall gets the panel **down its side**, with bars that
//! grow upward; a pane taller than it is wide gets it **across the top**, with
//! bars that grow rightward. Cells are twice as tall as they are wide
//! ([`orbs_render::CELL_WIDTH`] against [`orbs_render::CELL_HEIGHT`]), so "wider
//! than tall" is measured in *pixels* — an 80×22 grid is a wide rectangle, not a
//! tall one.
//!
//! # §14
//!
//! Four bars drawn with `progress` would push four utterances **per frame**,
//! against §14's *"progress announcements: completion only"*. The bars are drawn
//! with `meter`, which is silent, and the panel says **one** line naming only
//! what is doing something.

use orbs_render::cp437::box_drawing;
use orbs_render::{CELL_HEIGHT, CELL_WIDTH, Painter, Pos, Rect, Style, UtteranceKind};
use orbs_sim::tower::{Instrument, State};

/// Cells an instrument's name gets when the panel runs across the top.
///
/// `mortar_and_pestle` is **17** and the column reserves 18, because the last
/// cell is the gap before the state word. Reserving exactly 17 drew
/// `mortar_and_pestl` — a name the player cannot type, in the panel that exists
/// to tell them what to type.
const NAME: u16 = 18;

/// Cells for the state word. `scouring` is the longest.
const STATE: u16 = 9;

/// Cells one instrument's column takes when the panel runs down the side.
///
/// Two for the abbreviation, one for the gap.
const COLUMN: u16 = 3;

/// Which way the panel runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Along {
    /// Down the side, bars growing upward. A pane wider than it is tall.
    Side,
    /// Across the top, bars growing rightward. A pane taller than it is wide.
    Top,
}

impl Along {
    /// The way a pane of this shape wants it.
    fn of(area: Rect) -> Self {
        let wide = u32::from(area.cols) * u32::from(CELL_WIDTH);
        let tall = u32::from(area.rows) * u32::from(CELL_HEIGHT);
        if wide > tall { Self::Side } else { Self::Top }
    }
}

/// Where the panel sits, which way it runs, and what is left for the transcript.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Split {
    /// The panel's own rectangle. Empty when there is nothing to draw.
    pub area: Rect,
    /// Which way it runs — **carried, not re-derived.**
    ///
    /// `paint` used to read this back out of the panel's shape, by testing
    /// `rows == instruments.len() + 1`. A `Side` panel is full pane height, and
    /// full pane height can equal that too: five instruments and a six-row body
    /// made a vertical strip read as horizontal, so `top` drew 17-character names
    /// into a 16-column column. The function that *chose* the direction is the
    /// one that knows it.
    pub along: Along,
    /// What the transcript gets.
    pub rest: Rect,
}

/// The panel's footprint, and what is left for the transcript.
///
/// Both rectangles are empty when there is nothing to draw or no room to draw
/// it, so a caller can hand `rest` straight on.
pub(crate) fn split(area: Rect, instruments: &[Instrument]) -> Split {
    let nothing = Split {
        area: Rect::EMPTY,
        along: Along::Top,
        rest: area,
    };
    if instruments.is_empty() {
        return nothing;
    }
    let count = u16::try_from(instruments.len()).unwrap_or(u16::MAX);

    let along = Along::of(area);
    let (panel, rest) = match along {
        Along::Side => {
            // Columns, plus one cell for the rule between panel and transcript.
            let width = count.saturating_mul(COLUMN).saturating_add(1);
            // Refuse rather than squeeze: a panel with no room for its bars is
            // worse than no panel, because it looks like the instruments are
            // idle.
            if area.cols <= width.saturating_add(20) || area.rows < 4 {
                return nothing;
            }
            let rest = Rect::new(area.col, area.row, area.cols - width, area.rows);
            let panel = Rect::new(area.col + area.cols - width, area.row, width, area.rows);
            (panel, rest)
        }
        Along::Top => {
            // A row each, plus one for the rule.
            let height = count.saturating_add(1);
            if area.rows <= height.saturating_add(3) {
                return nothing;
            }
            let panel = Rect::new(area.col, area.row, area.cols, height);
            let rest = Rect::new(area.col, area.row + height, area.cols, area.rows - height);
            (panel, rest)
        }
    };
    Split {
        area: panel,
        along,
        rest,
    }
}

/// Draw the panel where [`split`] put it.
///
/// `domain` is where the player is standing, for the spoken summary — passed in
/// rather than written here, because a frontend must not be the thing that
/// decides a place name (rule 2). It was the literal `"laboratory"`, next to a
/// pane title drawn from the real location: the moment §10's Phase 3a adds a
/// second instrumented room, a sighted player would read `/tower/workshop` in the
/// border while a screen-reader user heard "laboratory: forge burning".
pub(crate) fn paint(
    painter: &mut Painter<'_>,
    split: Split,
    instruments: &[Instrument],
    domain: &str,
) {
    if instruments.is_empty() || split.area.is_empty() {
        return;
    }
    match split.along {
        Along::Side => side(painter, split.area, instruments),
        Along::Top => top(painter, split.area, instruments),
    }
    speak(painter, instruments, domain);
}

/// Down the side: a column per instrument, bars growing upward.
fn side(painter: &mut Painter<'_>, area: Rect, instruments: &[Instrument]) {
    // The rule sits on the panel's first column, between it and the transcript.
    painter.fill(
        Rect::new(area.col, area.row, 1, area.rows),
        box_drawing::VERTICAL,
        Style::DIM,
    );

    let body = Rect::new(area.col + 1, area.row, area.cols - 1, area.rows);
    for (index, instrument) in instruments.iter().enumerate() {
        let at = body
            .col
            .saturating_add(u16::try_from(index).unwrap_or(0).saturating_mul(COLUMN));
        if at >= body.right() {
            break;
        }
        let style = style_of(instrument.state);
        painter.glyphs(Pos::new(at, body.row), &instrument.short, style);

        // Bars grow up from the bottom, which is what a level reads as, and are
        // as wide as the label above them — a one-cell bar under a two-cell
        // abbreviation reads as a stray column rather than as that tool's meter.
        let Some(meter) = instrument.meter else {
            continue;
        };
        let width = (COLUMN - 1).min(body.right().saturating_sub(at));
        let bar = Rect::new(at, body.row + 1, width, body.rows.saturating_sub(1));
        painter.meter_upward(bar, to_u32(meter.done), to_u32(meter.total), style);
    }
}

/// Across the top: a row per instrument, bars growing rightward.
fn top(painter: &mut Painter<'_>, area: Rect, instruments: &[Instrument]) {
    for (index, instrument) in instruments.iter().enumerate() {
        let row = area.row.saturating_add(u16::try_from(index).unwrap_or(0));
        if row >= area.bottom() {
            break;
        }
        let style = style_of(instrument.state);
        painter.glyphs(
            Pos::new(area.col, row),
            orbs_render::arriving(&instrument.name, u32::from(NAME) - 1),
            style,
        );
        painter.glyphs(
            Pos::new(area.col.saturating_add(NAME), row),
            instrument.state.label(),
            style,
        );

        let used = NAME.saturating_add(STATE);
        if let Some(meter) = instrument.meter
            && area.cols > used
        {
            let bar = Rect::new(
                area.col.saturating_add(used),
                row,
                area.cols.saturating_sub(used),
                1,
            );
            painter.meter(bar, to_u32(meter.done), to_u32(meter.total), style);
        }
    }

    // The rule sits on the panel's last row, between it and the transcript.
    painter.fill(
        Rect::new(area.col, area.bottom().saturating_sub(1), area.cols, 1),
        box_drawing::HORIZONTAL,
        Style::DIM,
    );
}

/// One utterance for the whole panel, naming only what is doing something.
///
/// Four per frame is what §14 forbids, and "four instruments idle" is not news a
/// listener needs repeated.
fn speak(painter: &mut Painter<'_>, instruments: &[Instrument], domain: &str) {
    use core::fmt::Write as _;

    // One buffer written into, rather than a `format!` per instrument plus a
    // `Vec<String>` plus a `join` plus an outer `format!` — six allocations a
    // frame for a sentence that changes at most once a second.
    // **`Charged` is spoken.** Only the two states that mean *nothing is there*
    // are dropped. The `Top` layout draws a state word per row, so filtering a
    // loaded instrument out of the utterance made a word a sighted player reads
    // one a listener never hears — §19's rule that a visual constraint must not
    // become an informational one, in the direction nobody checks. It is also the
    // one filtered state that is *actionable*: charged means wield it.
    let mut spoken = String::new();
    for instrument in instruments
        .iter()
        .filter(|instrument| !matches!(instrument.state, State::Empty | State::Cold))
    {
        if spoken.is_empty() {
            spoken.push_str(domain);
            spoken.push_str(": ");
        } else {
            spoken.push_str(", ");
        }
        // Infallible into a `String`; the `Result` is `fmt`'s signature, not a
        // failure mode.
        let _ = write!(spoken, "{} {}", instrument.name, instrument.state.label());
    }
    if !spoken.is_empty() {
        painter.announce(UtteranceKind::Progress, Style::DIM.role, &spoken);
    }
}

/// The accent an instrument's state draws in.
///
/// §3 keeps the triad meaningful: `Cost` for work under way and fuel being
/// spent, `Success` for something finished and waiting, plain for at rest.
const fn style_of(state: State) -> Style {
    match state {
        State::Working | State::Scouring | State::Burning => Style::COST,
        State::Ready => Style::SUCCESS,
        _ => Style::DIM,
    }
}

/// A tick count as a meter value, saturating rather than wrapping.
fn to_u32(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instruments(names: &[&str]) -> Vec<Instrument> {
        names
            .iter()
            .map(|name| Instrument {
                name: (*name).to_owned(),
                short: name.chars().take(2).collect(),
                state: State::Empty,
                meter: None,
            })
            .collect()
    }

    const LABORATORY: [&str; 5] = [
        "mortar_and_pestle",
        "balneum_mariae",
        "flask_and_rod",
        "alembic",
        "athanor",
    ];

    #[test]
    fn an_empty_laboratory_takes_no_space() {
        let split = split(Rect::new(0, 0, 80, 22), &[]);
        assert!(split.area.is_empty());
        assert_eq!(split.rest, Rect::new(0, 0, 80, 22));
    }

    #[test]
    fn a_wide_pane_puts_the_panel_down_the_side() {
        // 80×22 cells is 640×352 pixels — a wide rectangle, because cells are
        // twice as tall as they are wide. Measuring in cells would call this
        // tall and put the panel on top.
        let area = Rect::new(0, 0, 80, 22);
        assert_eq!(Along::of(area), Along::Side);

        let split = split(area, &instruments(&LABORATORY));
        assert_eq!(split.along, Along::Side);
        assert_eq!(split.area.rows, area.rows, "a side panel is full height");
        assert_eq!(split.rest.rows, area.rows);
        assert_eq!(
            split.area.cols + split.rest.cols,
            area.cols,
            "they tile the pane"
        );
    }

    #[test]
    fn a_tall_pane_puts_the_panel_across_the_top() {
        let area = Rect::new(0, 0, 40, 60);
        assert_eq!(Along::of(area), Along::Top);

        let split = split(area, &instruments(&LABORATORY));
        assert_eq!(split.along, Along::Top);
        assert_eq!(split.area.cols, area.cols, "a top panel is full width");
        assert_eq!(
            split.area.rows + split.rest.rows,
            area.rows,
            "they tile the pane"
        );
    }

    #[test]
    fn the_direction_is_carried_rather_than_read_back_from_the_shape() {
        // `paint` must not disagree with `split` about which way the panel runs,
        // or it draws columns into a strip laid out as rows. It used to re-derive
        // this from `rows == instruments.len() + 1` — and a **`Side`** panel is
        // full pane height, which for five instruments and a six-row body is
        // exactly six. Carrying the answer makes the ambiguity unrepresentable;
        // this pins the shape that used to trip it.
        let ambiguous = Rect::new(0, 0, 80, 6);
        let split = split(ambiguous, &instruments(&LABORATORY));
        assert_eq!(Along::of(ambiguous), Along::Side);
        if !split.area.is_empty() {
            assert_eq!(
                split.along,
                Along::Side,
                "a six-row body read back as a top panel"
            );
        }
    }

    #[test]
    fn a_pane_with_no_room_gets_no_panel_rather_than_a_squeezed_one() {
        // A panel with no room for its bars looks like four idle instruments,
        // which is worse than no panel at all.
        let split = split(Rect::new(0, 0, 24, 8), &instruments(&LABORATORY));
        assert!(split.area.is_empty());
        assert_eq!(split.rest.cols, 24);
    }

    #[test]
    fn every_instrument_name_survives_the_top_layout_whole() {
        // A truncated name is a name the player cannot type, in the panel whose
        // job is telling them what to type. `mortar_and_pestl` shipped once.
        for name in LABORATORY {
            assert_eq!(
                orbs_render::arriving(name, u32::from(NAME) - 1),
                name,
                "{name} is cut by its own column"
            );
        }
    }

    #[test]
    fn every_state_word_survives_its_column_whole() {
        for state in [
            State::Empty,
            State::Charged,
            State::Working,
            State::Scouring,
            State::Ready,
            State::Fouled,
            State::Burning,
            State::Banked,
            State::Cold,
        ] {
            let label = state.label();
            assert!(
                u16::try_from(label.len()).unwrap_or(u16::MAX) < STATE,
                "{label} fills its column with no gap before the bar"
            );
        }
    }
}
