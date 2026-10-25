//! The page the orb's menu and its manual are both drawn on: a bordered pane
//! with a typed line at its foot.

use orbs_render::{Frame, Painter, Pos, Rect, Span, Style};

/// The caret's lead-in, and what the typed line is indented by.
pub(crate) const LEAD: &str = "> ";

/// Draw a bordered page titled `title` into `pane` — or `too_small` in it, when
/// it is under `least` rows and columns — and set the caret `body` hands back.
///
/// The caret is set after the painter is done, the way `sheet::paint` hands one
/// back: a `Painter` holds the `Frame`, so the two cannot be reached at once.
/// None when the pane is too small to type into: a caret is the game's one
/// promise about where a keystroke lands, and drawing it over a message that
/// says there is no room would be a lie about exactly that.
pub(crate) fn paint(
    frame: &mut Frame,
    pane: Rect,
    title: &str,
    least: (u16, u16),
    too_small: &str,
    body: impl FnOnce(&mut Painter<'_>, Rect) -> Pos,
) {
    if pane.is_empty() {
        return;
    }
    let caret = {
        let mut painter = frame.painter(pane);
        let area = painter.area();
        painter.border(area, Some(title), Style::DIM);
        let inner = area.inset(1);
        if inner.is_empty() {
            return;
        }
        if area.rows < least.0 || area.cols < least.1 {
            painter.paragraph(inner, &Span::new(too_small));
            None
        } else {
            Some(body(&mut painter, inner))
        }
    };
    frame.set_cursor(caret);
}

/// The typed line at `row`, with `complaint` under it, and where the caret goes.
///
/// Clamped to the pane, which `sheet.rs` has always done: a long enough line
/// put the caret past the right border — into the tower rail at 120×45, and off
/// the grid a few characters later.
pub(crate) fn typed_line(
    painter: &mut Painter<'_>,
    inner: Rect,
    row: u16,
    command: &str,
    complaint: Option<&str>,
) -> Pos {
    painter.span(
        Pos::new(inner.col, row),
        &Span::new(&format!("{LEAD}{command}")),
    );
    if let Some(complaint) = complaint {
        painter.span(
            Pos::new(inner.col, row.saturating_add(1)),
            &Span::new(complaint).with_style(Style::DIM),
        );
    }
    let typed = LEAD.chars().count().saturating_add(command.chars().count());
    Pos::new(
        inner
            .col
            .saturating_add(u16::try_from(typed).unwrap_or(u16::MAX))
            .min(inner.col.saturating_add(inner.cols).saturating_sub(1)),
        row,
    )
}
