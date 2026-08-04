//! The command line, and everything the orb has said.
//!
//! Replaces the stand-in screen this crate shipped while the domains did not
//! exist. Every glyph here comes from a [`Record`](orbs_render::Record) the sim
//! emitted — nothing is composed at paint time, which is architectural rule 4
//! seen from the drawing end.
//!
//! The scrollback is painted from its **tail**. That keeps a screen reader and a
//! sighted player exactly level: `Speech` is a per-frame description of the
//! screen, so describing more than the screen shows would break rule 2 in the
//! direction nobody checks. History is `Records`'s job and always was — it is
//! the log a player will `peruse`.

use orbs_render::{
    Frame, PROMPT, Pos, RecordView, Rect, ScreenLayout, ScreenRequest, Span, Style, UtteranceKind,
};
use orbs_sim::Sim;

use super::input::Line;

/// Paint the session into `frame`.
pub(crate) fn paint(frame: &mut Frame, sim: &Sim, line: &Line) {
    let grid = frame.size();
    let layout = ScreenLayout::compute(&ScreenRequest::single(grid));
    let pane = layout.main().first().copied().unwrap_or(Rect::EMPTY);

    // The tick and seed live in the border title. `Painter::border` announces a
    // title as a heading, so world time reaches the linear stream without
    // spending a row — and a changing tick here is the visible proof that the
    // sim is running at all.
    let title = format!(
        "O.R.B.S.  tick {}  seed {:#x}",
        sim.tick().get(),
        sim.seed(),
    );
    let mut painter = frame.painter(pane);
    painter.border(pane, Some(&title), Style::DIM);

    let body = pane.inset(1);
    let records = sim.scrollback().records();
    // Only the tail fits. `iter().skip(n)` is O(1) here and stays `Clone`, which
    // is what `RecordView::draw` needs to measure and then draw.
    let skipped = records.len().saturating_sub(usize::from(body.rows));
    RecordView::prompt().draw(&mut painter, body, records.iter().skip(skipped));

    input_line(frame, layout.input(), line);
}

/// Draw the prompt and what is being typed into it.
fn input_line(frame: &mut Frame, area: Rect, line: &Line) {
    if area.is_empty() {
        return;
    }
    let mut painter = frame.painter(area);
    let prompt = painter.glyphs(area.origin(), PROMPT, Style::DIM);

    let (visible, caret) = line.viewport(area.cols.saturating_sub(prompt));
    // One span for the whole line rather than a glyph run: the linear stream
    // should carry what is being typed, tagged `Input` so a reader can filter
    // the partial line out. It is re-spoken every frame — which is correct raw
    // material and wrong to recite verbatim, hence the tag.
    painter.span(
        Pos::new(area.col.saturating_add(prompt), area.row),
        &Span::new(visible).with_kind(UtteranceKind::Input),
    );
    frame.set_cursor(Some(Pos::new(
        area.col.saturating_add(prompt).saturating_add(caret),
        area.row,
    )));
}

/// The window cannot host the game.
///
/// `Screen::is_hostable` documents this as a real state to render rather than a
/// reason to stop drawing: blanking the mesh left the player looking at an empty
/// rectangle with no idea why.
pub(crate) fn paint_too_small(frame: &mut Frame) {
    let area = frame.area();
    if area.is_empty() {
        return;
    }
    let mut painter = frame.painter(area);
    painter.paragraph(
        area,
        &Span::new("the orb needs a larger window").with_style(Style::DANGER),
    );
    frame.set_cursor(None);
}
