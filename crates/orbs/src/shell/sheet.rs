//! Drawing the spell editor.
//!
//! # Why this is here and not in `orbs-render`
//!
//! Rule 2 says `orbs-render` decides **what appears and where**, and it does:
//! `ScreenLayout` hands this function a rectangle and it draws inside it. What
//! `orbs-render` does *not* contain is any screen painter at all — `Painter` is
//! a primitive (`span`, `border`, `glyphs`), its `[dependencies]` is
//! deliberately empty, and every real surface in the game is drawn here in the
//! frontend. The instrument panel beside this file is the precedent:
//! informational content, not decoration, painted on this side.
//!
//! # Everything here speaks
//!
//! §14 forbids information carried only visually, so the buffer is drawn with
//! [`Painter::span`] rather than [`Painter::glyphs`] — the former goes to the
//! linear stream and the latter does not. That is also why the caret's position
//! is *stated* on the status row rather than being only a cell the frontend
//! brightens: a screen reader has to be able to find it.

use orbs_render::{Frame, Intensity, Painter, Pos, Rect, Role, Span, Style};
use orbs_sim::Prose;

use super::Editor;
use super::editor::{Complaint, Mode};

/// How wide the line-number gutter is: a marker column, three digits, a space.
///
/// Five cells, unchanged, but split differently. It was four digits and a space,
/// with the running-line marker replacing the space — which put the marker on
/// the *right* of the number, between it and the code. Three digits still spells
/// every line of any spell anyone will write, and reserving a sixth column would
/// cost the 80-cell floor the one thing that is scarce there.
const GUTTER: u16 = 5;

/// The marker column, and what goes in it.
///
/// **Left of the number, so the number does not move.** With the marker on the
/// right the digits shifted a column the instant an invocation reached the line,
/// and a gutter that twitches while you read it is worse than no marker.
///
/// `»` is CP437 0xAF — see [`title`] for the em-dash this file already lost.
const MARKER: char = '\u{bb}';

/// The smallest pane an editor can honestly be drawn in.
///
/// Two border rows, one status row, and at least one line of the spell — plus a
/// gutter and room for a word. Below this the answer is to say so rather than
/// draw something misleading; the buffer is untouched either way, so growing the
/// window recovers it.
const MIN_ROWS: u16 = 4;
const MIN_COLS: u16 = GUTTER + 8;

/// Draw the editor into `pane`.
///
/// Returns where the caret belongs, so the caller can place it on the frame. The
/// caret's **position** is information (rule 2); only its blink is the
/// frontend's own.
pub(crate) fn paint(
    frame: &mut Frame,
    editor: &mut Editor,
    pane: Rect,
    prose: &Prose,
) -> Option<(u16, u16)> {
    if pane.is_empty() || pane.rows < MIN_ROWS || pane.cols < MIN_COLS {
        return None;
    }
    let mut painter = frame.painter(pane);
    let area = painter.area();
    painter.border(area, Some(&title(editor, prose)), Style::DIM);

    // Inside the border: one row of chrome at the bottom for the status line.
    let text_rows = area.rows.saturating_sub(3);
    editor.scroll_to(text_rows as usize);

    let text_col = area.col.saturating_add(1).saturating_add(GUTTER);
    let width = area.cols.saturating_sub(2 + GUTTER);
    let (caret_row, caret_column) = editor.caret();
    // Which buffer row the orb is executing, zero-based to match `index`. A
    // spell's lines are numbered from one everywhere a player sees them.
    let running = editor
        .running_line()
        .and_then(|line| usize::try_from(line).ok())
        .and_then(|line| line.checked_sub(1));
    let mut caret = None;

    for row in 0..text_rows {
        let index = editor.top() + row as usize;
        if index >= editor.len() {
            break;
        }
        let y = area.row.saturating_add(1).saturating_add(row);
        let here = running == Some(index);

        // **The marker gets its own column, left of the number.** Drawn as a
        // separate span because it carries a different style — `Role::Success`,
        // the theme's completion accent, which is green in every theme where
        // green would read and yellow in the one where it would not. Merging it
        // into the number would either tint the digits or lose the colour.
        if here {
            painter.span(
                Pos::new(area.col.saturating_add(1), y),
                &Span::new(&MARKER.to_string()).with_style(
                    Style::default()
                        .with_role(Role::Success)
                        .with_intensity(Intensity::Bright),
                ),
            );
        }
        // The number is a fact about which line you are on, so it is spoken with
        // the line rather than drawn beside it — `1 attend laboratory` is what a
        // screen reader should hear, not a bare command with no position. It
        // sits at the same column whether or not the marker is beside it.
        painter.span(
            Pos::new(area.col.saturating_add(2), y),
            &Span::new(&format!("{:>3} ", index + 1)).with_style(if here {
                Style::default()
            } else {
                Style::DIM
            }),
        );
        let line = Span::new(editor.visible(index, width));
        painter.span(
            Pos::new(text_col, y),
            &if here {
                line.with_style(Style::default().with_intensity(Intensity::Bright))
            } else {
                line
            },
        );

        if index == caret_row && editor.mode() == Mode::Editing {
            // Clamped: a caret past the right edge would land on the border, or
            // off the frame entirely.
            // **`width - 1`, because `width` is a count and this is an index.**
            // The last text cell is `text_col + width - 1`; clamping to `width`
            // put the caret on `area.col + area.cols - 1`, which is the pane's
            // right border — so a full-width line parked a blinking caret on the
            // box-drawing glyph, the exact case the comment below claims to
            // prevent.
            let column = u16::try_from(caret_column)
                .unwrap_or(u16::MAX)
                .min(width.saturating_sub(1));
            caret = Some((text_col.saturating_add(column), y));
        }
    }

    status(&mut painter, editor, area, prose);

    // **In command state the caret belongs on the command row**, because that is
    // where typing goes. Leaving it in the buffer would be the same lie as
    // drawing the prompt while the editor holds the keyboard — a caret is the
    // one thing on screen whose whole job is saying where the next character
    // lands, and rule 2 makes its *position* information rather than decoration.
    if editor.mode() == Mode::Command {
        let typed = u16::try_from(editor.command().chars().count()).unwrap_or(u16::MAX);
        caret = Some((
            area.col
                .saturating_add(3)
                .saturating_add(typed)
                .min(area.col.saturating_add(area.cols).saturating_sub(2)),
            area.row.saturating_add(area.rows).saturating_sub(2),
        ));
    }
    caret
}

/// The border title: the spell, where it runs, whether it is unsaved, and where
/// a running invocation has reached.
///
/// **No em-dash.** CP437 has no `—`, so one drawn here comes out as `?` — the
/// same defect the `screens` example caught in §4's own boot text. The
/// separator is a word, which reads better anyway: a spell is *in* a domain.
///
/// The running line is said here as well as marked in the gutter, because
/// `Painter::border` pushes a title to the speech stream as a `Heading` and a
/// `»` in a column is exactly the kind of visual-only fact §14 forbids.
fn title(editor: &Editor, prose: &Prose) -> String {
    let dirty = if editor.is_dirty() { " *" } else { "" };
    let mut title = format!("{} in {}{dirty}", editor.name(), editor.domain());
    if let Some(line) = editor.running_line() {
        title.push(' ');
        title.push_str(&prose.line("editor_at_line", &[("count", &line.to_string())]));
    }
    title
}

/// The bottom row: the `:` line if one is open, a complaint if one is pending,
/// and otherwise the way out.
///
/// **It always says something.** §6 forbids a dead end, and this is the game's
/// first modal surface — a player who does not know `:q` has nowhere else to
/// find out, so the row that could have been blank is the row that tells them.
fn status(painter: &mut Painter<'_>, editor: &Editor, area: Rect, prose: &Prose) {
    let y = area.row.saturating_add(area.rows).saturating_sub(2);
    let width = usize::from(area.cols.saturating_sub(2));

    // **The row that must never be blank**, and in command state it is the
    // whole interface. §6 forbids a dead end, and this is the game's first modal
    // surface: a player who does not know the words has nowhere else to find
    // them, so the row that could have said nothing is the row that says them.
    //
    // A complaint outranks the help, and a half-typed word outranks both — you
    // should always be able to see what you are typing.
    let (text, style) = match (editor.mode(), editor.complaint()) {
        (Mode::Command, _) if !editor.command().is_empty() => {
            (format!("> {}", editor.command()), Style::default())
        }
        (_, Some(Complaint::Unknown(word))) => (
            prose.line("editor_unknown", &[("detail", word)]),
            Style::default().with_role(Role::Danger),
        ),
        (Mode::Command, None) => (prose.line("editor_words", &[]), Style::DIM),
        (Mode::Editing, None) => (prose.line("editor_editing", &[]), Style::DIM),
    };

    let position = editor.position();
    // The position is pinned right and the message takes what is left. At the
    // 80-cell floor that is ~68 cells, which `editor_words` is written to fit.
    let room =
        u32::try_from(width.saturating_sub(position.chars().count() + 1)).unwrap_or(u32::MAX);
    painter.span(
        Pos::new(area.col.saturating_add(1), y),
        &Span::new(orbs_render::arriving(&text, room)).with_style(style),
    );

    let at = area
        .col
        .saturating_add(area.cols)
        .saturating_sub(1)
        .saturating_sub(u16::try_from(position.chars().count()).unwrap_or(0));
    painter.span(
        Pos::new(at, y),
        &Span::new(&position).with_style(Style::DIM.with_intensity(Intensity::Dim)),
    );
}
