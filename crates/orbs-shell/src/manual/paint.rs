//! The manual, as cells.
//!
//! Rule 2: this decides what appears and where, and a frontend decides only how
//! a cell is drawn.
//!
//! `Focus::takes_the_pane` surfaces get no `F5` mirror (§14's hole, recorded in
//! `focus.rs`), so the frame's own `Speech` is all a screen reader gets — and a
//! manual is nothing but sentences. [`Painter::span`] and
//! [`Painter::paragraph`] announce as they draw, and
//! `a_chapter_reaches_the_linear_stream` holds it.
//!
//! [`Painter::span`]: orbs_render::Painter::span
//! [`Painter::paragraph`]: orbs_render::Painter::paragraph

use orbs_render::{Frame, Painter, Pos, Rect, Span, Style};
use orbs_sim::Prose;

use super::state::{Reader, Showing};

/// The smallest pane the manual can honestly be drawn in.
///
/// Two borders, the title, a blank, some of the page, a blank, the line, and a
/// row for a complaint. Below this it says so rather than drawing a chapter one
/// row at a time, as the menu and the weave do.
const MIN_ROWS: u16 = 10;
const MIN_COLS: u16 = 34;

/// The caret's lead-in, and what the typed line is indented by.
const LEAD: &str = "> ";

/// Draw the manual into `pane`.
///
/// `reader` is `&mut` for `Editor`'s reason: the viewport follows the content
/// and only the painter knows how tall the pane is. See [`Reader::measured`].
pub fn paint(frame: &mut Frame, reader: &mut Reader, pane: Rect, prose: &Prose) {
    if pane.is_empty() {
        return;
    }
    let caret = {
        let mut painter = frame.painter(pane);
        let area = painter.area();
        painter.border(area, Some(&prose.line("manual_title", &[])), Style::DIM);
        let inner = area.inset(1);
        if inner.is_empty() {
            return;
        }
        if area.rows < MIN_ROWS || area.cols < MIN_COLS {
            painter.paragraph(inner, &Span::new(&prose.line("manual_too_small", &[])));
            None
        } else {
            Some(body(&mut painter, reader, inner, prose))
        }
    };
    frame.set_cursor(caret);
}

/// The manual's rows, and where the caret ends up.
fn body(painter: &mut Painter<'_>, reader: &mut Reader, inner: Rect, prose: &Prose) -> Pos {
    // The line and its complaint sit at the bottom; everything above is the
    // page. Two rows are always spent, which is what `MIN_ROWS` reserves.
    let line_row = inner.row.saturating_add(inner.rows.saturating_sub(2));
    let page = Rect {
        rows: inner.rows.saturating_sub(3),
        ..inner
    };

    match reader.showing().clone() {
        Showing::Contents => contents(painter, reader, page, prose),
        Showing::Chapter { at, top } => chapter(painter, reader, at, top, page, prose),
    }

    painter.span(
        Pos::new(inner.col, line_row),
        &Span::new(&format!("{LEAD}{}", reader.command())),
    );
    if let Some(missing) = reader.complaint() {
        painter.span(
            Pos::new(inner.col, line_row.saturating_add(1)),
            &Span::new(&prose.line("manual_unknown", &[("detail", missing)]))
                .with_style(Style::DIM),
        );
    }

    // Clamped to the pane, for `menu/paint.rs`'s reason and by the same rule
    // `sheet.rs` has always used.
    let typed = LEAD
        .chars()
        .count()
        .saturating_add(reader.command().chars().count());
    Pos::new(
        inner
            .col
            .saturating_add(u16::try_from(typed).unwrap_or(u16::MAX))
            .min(inner.col.saturating_add(inner.cols).saturating_sub(1)),
        line_row,
    )
}

/// Every chapter, by name.
fn contents(painter: &mut Painter<'_>, reader: &mut Reader, page: Rect, prose: &Prose) {
    let mut y = page.row;
    painter.span(
        Pos::new(page.col, y),
        &Span::new(&prose.line("manual_contents", &[])).with_style(Style::DIM),
    );
    y = y.saturating_add(2);

    // The list pages like a chapter does. It used to stop at the bottom of the
    // pane, silently losing chapters once the book outgrew it.
    let showing = usize::from(page.rows.saturating_sub(2));
    let top = reader.contents_top();
    let entries: Vec<String> = reader
        .book()
        .iter()
        .map(|entry| format!("{:<10}{}", entry.name, entry.title))
        .collect();
    for entry in entries.iter().skip(top).take(showing) {
        painter.span(Pos::new(page.col.saturating_add(2), y), &Span::new(entry));
        y = y.saturating_add(1);
    }

    reader.measured(showing, entries.len());

    if entries.len() > top.saturating_add(showing) {
        painter.span(
            Pos::new(page.col, page.row.saturating_add(page.rows)),
            &Span::new(&prose.line("manual_more", &[])).with_style(Style::DIM),
        );
    }
}

/// One chapter, wrapped, from `top`.
fn chapter(
    painter: &mut Painter<'_>,
    reader: &mut Reader,
    at: usize,
    top: usize,
    page: Rect,
    prose: &Prose,
) {
    // Wrapped here because only here knows the width — which is why the manual
    // is not in `prose.toml`, whose lines are drawn as written.
    //
    // Cached on `(chapter, width)`: this used to re-wrap two thousand words
    // sixty times a second while the reader sat still. Only a resize or a
    // `restock` changes a chapter, and both move the key.
    let width = page.cols.saturating_sub(2).max(1);
    let title = reader.wrap_for(at, width);
    let Some(title) = title else {
        return;
    };
    let rows = reader.wrapped();

    let mut y = page.row;
    painter.span(
        Pos::new(page.col, y),
        &Span::new(&title).with_style(Style::DIM),
    );
    y = y.saturating_add(2);

    let showing = usize::from(page.rows.saturating_sub(2));
    let height = rows.len();
    for row in rows.iter().skip(top).take(showing) {
        if !row.is_empty() {
            painter.span(Pos::new(page.col.saturating_add(2), y), &Span::new(row));
        }
        y = y.saturating_add(1);
    }

    // After drawing, so the clamp sees the height this width produced.
    reader.measured(showing, height);

    // Only when there is: a pager that always claimed more would be the border
    // hint that lied.
    if height > top.saturating_add(showing) {
        painter.span(
            Pos::new(page.col, page.row.saturating_add(page.rows)),
            &Span::new(&prose.line("manual_more", &[])).with_style(Style::DIM),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::GridSize;

    fn book() -> Vec<orbs_sim::content::Chapter> {
        orbs_sim::content::Manual::builtin().chapters().to_vec()
    }

    #[test]
    fn a_chapter_reaches_the_linear_stream() {
        // `takes_the_pane` means no `F5` mirror, so the frame's own speech is
        // all a screen reader gets (§14).
        let mut frame = Frame::new(GridSize::new(100, 36));
        let area = frame.area();
        let mut reader = Reader::of(book());
        reader.type_text("orbs");
        reader.enter();
        paint(&mut frame, &mut reader, area, &orbs_sim::Prose::builtin());

        let spoken: Vec<&str> = frame
            .speech()
            .utterances()
            .map(|utterance| utterance.text)
            .collect();
        assert!(
            spoken.iter().any(|said| said.contains("wizard")),
            "the chapter was drawn but not spoken: {spoken:?}",
        );
    }

    /// Everything the frame has on it, as one string.
    fn drawn(frame: &Frame) -> String {
        frame
            .speech()
            .utterances()
            .map(|utterance| utterance.text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn every_chapter_in_the_book_is_reachable_from_the_contents_at_the_floor() {
        // At §4's 80x22 floor the contents showed fifteen chapters and stopped,
        // losing the generated ones and the licences — §15's worst dead end, a
        // page nothing tells you is there.
        let prose = orbs_sim::Prose::builtin();
        let mut reader = Reader::of(book());
        let mut seen = String::new();
        // Paged the way a player pages, with a bound so a broken clamp fails
        // rather than hangs.
        for _ in 0..40 {
            let mut frame = Frame::new(GridSize::new(80, 22));
            let area = frame.area();
            paint(&mut frame, &mut reader, area, &prose);
            seen.push_str(&drawn(&frame));
            let step = reader.page();
            reader.scroll(step, true);
        }
        for chapter in book() {
            assert!(
                seen.contains(&chapter.name),
                "`{}` is in the book and cannot be reached from the contents",
                chapter.name,
            );
        }
    }

    #[test]
    fn the_contents_says_there_is_more_only_when_there_is() {
        // A pager claiming more at the bottom is a key that does nothing, which
        // is worse than no hint.
        let prose = orbs_sim::Prose::builtin();
        let more = prose.line("manual_more", &[]);

        let mut short = Reader::of(book());
        let mut frame = Frame::new(GridSize::new(80, 22));
        let area = frame.area();
        paint(&mut frame, &mut short, area, &prose);
        assert!(
            drawn(&frame).contains(&more),
            "nineteen chapters at the floor, and it claimed they all fit",
        );

        let mut tall = Reader::of(book());
        let mut frame = Frame::new(GridSize::new(80, 60));
        let area = frame.area();
        paint(&mut frame, &mut tall, area, &prose);
        assert!(
            !drawn(&frame).contains(&more),
            "the whole book is on screen and it asked for pgdn",
        );
    }

    #[test]
    fn coming_back_from_a_chapter_keeps_your_place_in_the_contents() {
        // `contents_top` is on the reader rather than in `Showing::Contents`
        // for this.
        let prose = orbs_sim::Prose::builtin();
        let mut reader = Reader::of(book());
        let mut frame = Frame::new(GridSize::new(80, 22));
        let area = frame.area();
        paint(&mut frame, &mut reader, area, &prose);
        reader.scroll(reader.page(), true);
        let was = reader.contents_top();
        assert!(was > 0, "the contents did not scroll at all");

        reader.type_text("orbs");
        reader.enter();
        reader.escape();
        assert_eq!(
            reader.contents_top(),
            was,
            "reading a chapter put the contents back at the top",
        );
    }

    #[test]
    fn a_long_line_keeps_the_caret_inside_the_pane() {
        // Unclamped, a long enough typed line put the caret past the right
        // border and then off the grid. `sheet.rs` clamped; this did not.
        let prose = orbs_sim::Prose::builtin();
        let mut frame = Frame::new(GridSize::new(120, 45));
        let area = frame.area();
        let mut reader = Reader::of(book());
        reader.type_text(&"z".repeat(400));
        paint(&mut frame, &mut reader, area, &prose);
        let caret = frame.cursor().expect("a caret");
        assert!(
            caret.col < area.col + area.cols,
            "the caret is at column {} on a {}-wide pane",
            caret.col,
            area.cols,
        );
    }

    #[test]
    fn a_pane_too_small_says_so_rather_than_drawing_a_chapter_a_row_at_a_time() {
        let mut frame = Frame::new(GridSize::new(20, 6));
        let area = frame.area();
        let mut reader = Reader::of(book());
        paint(&mut frame, &mut reader, area, &orbs_sim::Prose::builtin());
        assert_eq!(frame.cursor(), None, "it offered a caret it cannot honour");
    }
}
