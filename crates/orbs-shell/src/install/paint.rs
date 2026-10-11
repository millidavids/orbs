//! The installation's strip, across the top of the pane where the gauges sit.
//!
//! Rule 2: every word is `prose.toml`'s and every mark is [`Install`]'s. This
//! decides where they go, and what a short pane gives up first.

use std::cmp::Ordering;

use orbs_render::{Painter, Pos, Rect, Span, Style, UtteranceKind};
use orbs_sim::Prose;

use super::state::{Install, Step, WAKE};

/// Rows the transcript keeps under the strip. Below this the strip yields a
/// row, as the road does: the player's own last line outranks the lesson.
const KEEP: u16 = 6;

/// Between the title and the count, and between checklist entries.
const GAP: u16 = 2;

/// Between the count and `skip ends it`.
const WIDE_GAP: u16 = 3;

/// A row of the strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Title,
    Checklist,
    First,
    Second,
    Try,
    Gap,
}

/// Top to bottom.
const DRAWN: [Row; 6] = [
    Row::Title,
    Row::Checklist,
    Row::First,
    Row::Second,
    Row::Try,
    Row::Gap,
];

/// What a short pane keeps, most needed first. The checklist goes early: the
/// title's count already says how far along the player is.
const KEPT: [Row; 6] = [
    Row::Title,
    Row::First,
    Row::Try,
    Row::Second,
    Row::Checklist,
    Row::Gap,
];

/// The rows the strip took, and what is left for the rest of the pane.
#[derive(Debug, Clone, Copy)]
pub struct Split {
    /// The strip, or empty when there is no room for any of it.
    pub area: Rect,
    /// The body below it.
    pub rest: Rect,
}

/// Take the top of `body` for the strip.
#[must_use]
pub fn split(body: Rect) -> Split {
    let rows = u16::try_from(kept(body.rows.saturating_sub(KEEP)).count()).unwrap_or(0);
    Split {
        area: Rect::new(body.col, body.row, body.cols, rows),
        rest: Rect::new(
            body.col,
            body.row.saturating_add(rows),
            body.cols,
            body.rows.saturating_sub(rows),
        ),
    }
}

/// The rows that fit in `room`, in the order they are drawn.
fn kept(room: u16) -> impl Iterator<Item = Row> {
    let all: &'static [Row] = &KEPT;
    let keeping = &all[..usize::from(room).min(KEPT.len())];
    DRAWN.into_iter().filter(move |row| keeping.contains(row))
}

/// How a step stands, for its mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Done,
    Now,
    Waiting,
}

/// Everything the strip says, before it is placed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Said {
    title: String,
    /// What may sit at the title's right edge, widest first: as drawn, and the
    /// row as read. Empty once every step is done.
    right: Vec<(String, String)>,
    checklist: Vec<(Mark, String)>,
    /// The checklist as one sentence, and the cells it takes drawn.
    checklist_read: String,
    checklist_cols: u16,
    first: String,
    second: String,
    /// The word `try`, the line, and the row as read.
    attempt: (String, String, String),
    ok: String,
}

impl Said {
    /// The strip with `done` steps finished.
    pub(super) fn new(done: usize, prose: &Prose) -> Self {
        let title = prose.line("install_title", &[]);
        let ok = prose.line("install_ok", &[]);
        let now = prose.line("install_now", &[]);
        let checklist: Vec<(Mark, String)> = Step::ALL
            .into_iter()
            .enumerate()
            .map(|(at, step)| {
                let mark = match at.cmp(&done) {
                    Ordering::Less => Mark::Done,
                    Ordering::Equal => Mark::Now,
                    Ordering::Greater => Mark::Waiting,
                };
                (
                    mark,
                    prose.line(&format!("install_{}_name", step.key()), &[]),
                )
            })
            .collect();
        let checklist_read = checklist
            .iter()
            .map(|(mark, name)| match mark {
                Mark::Done => format!("{name} {ok}"),
                Mark::Now => format!("{name} {now}"),
                Mark::Waiting => name.clone(),
            })
            .collect::<Vec<_>>()
            .join(", ");
        // `[`, the mark as drawn, `] `, the name: `install_ok` is prose, so its
        // width is a writer's.
        let checklist_cols = checklist
            .iter()
            .map(|(mark, name)| {
                cells(marked(&ok, *mark).0)
                    .saturating_add(3)
                    .saturating_add(cells(name))
            })
            .fold(0_u16, |sum, cells| {
                sum.saturating_add(cells).saturating_add(GAP)
            })
            .saturating_sub(GAP);
        let (first, second, right, line) = match Step::ALL.get(done) {
            Some(step) => {
                let key = step.key();
                let count = prose.line(
                    "install_step",
                    &[
                        ("count", &done.saturating_add(1).to_string()),
                        ("quantity", &Step::ALL.len().to_string()),
                    ],
                );
                let skip = prose.line("install_skip", &[]);
                let wide = " ".repeat(usize::from(WIDE_GAP));
                (
                    prose.line(&format!("install_{key}_1"), &[]),
                    prose.line(&format!("install_{key}_2"), &[]),
                    vec![
                        (
                            format!("{count}{wide}{skip}"),
                            format!("{title}, {count}, {skip}"),
                        ),
                        (count.clone(), format!("{title}, {count}")),
                    ],
                    prose.line(&format!("install_{key}_try"), &[]),
                )
            }
            None => (
                prose.line("install_done_1", &[]),
                prose.line("install_done_2", &[]),
                Vec::new(),
                WAKE.to_owned(),
            ),
        };
        let word = prose.line("install_try", &[]);
        let read = format!("{word} {line}");
        Self {
            title,
            right,
            checklist,
            checklist_read,
            checklist_cols,
            first,
            second,
            attempt: (word, line, read),
            ok,
        }
    }
}

/// Draw the strip into the rows [`split`] took.
pub fn paint(painter: &mut Painter<'_>, area: Rect, install: &Install) {
    if area.is_empty() {
        return;
    }
    let said = install.said();
    let mut y = area.row;
    for row in kept(area.rows) {
        let at = Pos::new(area.col, y);
        match row {
            Row::Title => title(painter, area, at, said),
            Row::Checklist => checklist(painter, area, at, said),
            Row::First => {
                painter.span(at, &Span::new(&said.first));
            }
            Row::Second => {
                painter.span(at, &Span::new(&said.second));
            }
            Row::Try => attempt(painter, at, said),
            Row::Gap => {}
        }
        y = y.saturating_add(1);
    }
}

fn cells(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// The title, and the count pinned to the right edge where it fits.
///
/// `skip ends it` yields before the count does, which says where the player
/// is. Neither is drawn into the title's cells.
fn title(painter: &mut Painter<'_>, area: Rect, at: Pos, said: &Said) {
    let written = painter.glyphs(at, &said.title, Style::BRIGHT);
    let room = area.cols.saturating_sub(written.saturating_add(GAP));
    // One sentence for the row, saying only what is drawn.
    let read = match said.right.iter().find(|(drawn, _)| cells(drawn) <= room) {
        Some((drawn, read)) => {
            let col = area.right().saturating_sub(cells(drawn));
            painter.glyphs(Pos::new(col, at.row), drawn, Style::DIM);
            read
        }
        None => &said.title,
    };
    painter.announce(UtteranceKind::Heading, Style::BRIGHT.role, read);
}

/// A checklist entry's mark, and the styles of the mark and the name.
const fn marked(ok: &str, mark: Mark) -> (&str, Style, Style) {
    match mark {
        Mark::Done => (ok, Style::SUCCESS, Style::DIM),
        Mark::Now => (">>", Style::BRIGHT, Style::BRIGHT),
        Mark::Waiting => ("  ", Style::DIM, Style::DIM),
    }
}

/// `[ok] listening  [>>] sight  [  ] paths`, or nothing where it would not fit.
fn checklist(painter: &mut Painter<'_>, area: Rect, at: Pos, said: &Said) {
    if said.checklist_cols > area.cols {
        return;
    }
    let mut col = at.col;
    for (mark, name) in &said.checklist {
        let (glyph, mark_style, name_style) = marked(&said.ok, *mark);
        col = col.saturating_add(painter.glyphs(Pos::new(col, at.row), "[", Style::DIM));
        col = col.saturating_add(painter.glyphs(Pos::new(col, at.row), glyph, mark_style));
        col = col.saturating_add(painter.glyphs(Pos::new(col, at.row), "] ", Style::DIM));
        col = col.saturating_add(painter.glyphs(Pos::new(col, at.row), name, name_style));
        col = col.saturating_add(GAP);
    }
    painter.announce(UtteranceKind::Text, Style::DIM.role, &said.checklist_read);
}

/// `try  attend laboratory`, and `try  wake` once every step is done.
fn attempt(painter: &mut Painter<'_>, at: Pos, said: &Said) {
    let (word, line, read) = &said.attempt;
    let written = painter.glyphs(at, word, Style::DIM);
    painter.glyphs(
        Pos::new(at.col.saturating_add(written).saturating_add(GAP), at.row),
        line,
        Style::BRIGHT,
    );
    painter.announce(UtteranceKind::Text, Style::NORMAL.role, read);
}
