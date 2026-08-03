//! Interim screen content.
//!
//! Stands in for the real surfaces — boot report, panes, input line — until the
//! domains exist. The renderer calls [`paint`] once per frame; everything here
//! is ordinary `orbs-render` painting with no frontend knowledge in it, which is
//! the shape every real screen will take.

use orbs_render::{Frame, Pos, Rect, ScreenLayout, ScreenRequest, Span, Style, UtteranceKind};

/// Paint the current screen into `frame`.
///
/// §4's boot report reflects real world state rather than a mock-up, and this
/// stand-in keeps that habit: the tick shown is the tick the sim is on.
pub(crate) fn paint(frame: &mut Frame, tick: u64, seed: u64) {
    let grid = frame.size();
    let layout = ScreenLayout::compute(&ScreenRequest::single(grid));
    let pane = layout.main().first().copied().unwrap_or(Rect::EMPTY);

    let mut painter = frame.painter(pane);
    painter.border(pane, Some("O.R.B.S."), Style::DIM);

    let body = pane.inset(2);
    let mut inner = painter.sub(body);
    inner.span(
        body.origin(),
        &Span::new("O.R.B.S. -- cell renderer online").with_style(Style::BRIGHT),
    );
    inner.span(
        Pos::new(body.col, body.row + 2),
        &Span::new(&format!("tick {tick}  seed {seed:#x}")).with_style(Style::DIM),
    );
    inner.span(
        Pos::new(body.col, body.row + 4),
        &Span::new("ward failed").with_style(Style::DANGER),
    );
    inner.span(
        Pos::new(body.col, body.row + 5),
        &Span::new("42 mana").with_style(Style::COST),
    );
    inner.span(
        Pos::new(body.col, body.row + 6),
        &Span::new("haste decocted").with_style(Style::SUCCESS),
    );
    inner.progress(
        Rect::new(body.col, body.row + 8, body.cols.min(24), 1),
        34,
        100,
        Style::DANGER,
        "east wall integrity 34 percent",
    );

    let input = layout.input();
    frame.painter(input).span(
        input.origin(),
        &Span::new("orbs:~$ ").with_kind(UtteranceKind::Input),
    );
    frame.set_cursor(Some(Pos::new(input.col + 8, input.row)));
}

/// What to show when the window cannot host the 80×22 floor.
///
/// `Screen::is_hostable` calls this "a real state to render", and §4's minimum is
/// a declared constraint rather than something to squeeze under. Drawing nothing
/// leaves the player looking at an empty rectangle with no idea why, so say it —
/// and say it in whatever space there actually is.
pub(crate) fn paint_too_small(frame: &mut Frame) {
    let area = frame.area();
    if area.is_empty() {
        return;
    }

    let grid = frame.size();
    let mut painter = frame.painter(area);
    painter.paragraph(
        area,
        &Span::new(&format!(
            "the orb cannot focus.\n\nthis window shows {}x{} cells.\nit needs {}x{}.",
            grid.cols,
            grid.rows,
            orbs_render::MIN_GRID.cols,
            orbs_render::MIN_GRID.rows,
        ))
        .with_style(Style::DANGER),
    );
}
