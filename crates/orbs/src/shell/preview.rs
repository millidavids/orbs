//! Interim: the Frame, printed to the log.
//!
//! **Stands in for the cell-grid renderer** (Phase 0 item 4). It exists so the
//! whole pipeline — sim tick, layout, painting, linearisation — is observable
//! end to end inside the real binary before the GPU path is written, rather than
//! only in `orbs-render`'s tests.
//!
//! Delete this module when the renderer lands. Nothing else depends on it.

use bevy::prelude::*;
use orbs_render::{Frame, Pos, Rect, ScreenLayout, ScreenRequest, Span, Style, UtteranceKind};

use crate::shell::screen::Screen;
use crate::sim::Tower;

/// The frame, reused across ticks.
///
/// `Frame`'s own documentation says to reset rather than reallocate: the grid
/// reaches 160x45 and a siege redraws it every frame. This module is the shape
/// the real cell renderer will be written from, so it should model the right
/// habit rather than the convenient one.
#[derive(Resource, Default)]
pub(crate) struct Canvas(Frame);

/// Log one painted frame per world tick.
pub(crate) fn log_frame(screen: Res<Screen>, tower: Res<Tower>, mut canvas: ResMut<Canvas>) {
    if !screen.is_hostable() {
        return;
    }

    let frame = &mut canvas.0;
    frame.reset(screen.grid);
    let layout = ScreenLayout::compute(&ScreenRequest::single(screen.grid));
    let pane = layout.main().first().copied().unwrap_or(Rect::EMPTY);

    let mut painter = frame.painter(pane);
    painter.border(pane, Some("orbs"), Style::DIM);

    let body = pane.inset(2);
    let mut inner = painter.sub(body);
    inner.span(
        body.origin(),
        &Span::new("O.R.B.S. -- cell renderer pending").with_style(Style::BRIGHT),
    );
    inner.span(
        Pos::new(body.col, body.row + 2),
        &Span::new(&format!("tick   {}", tower.tick().get())),
    );
    inner.span(
        Pos::new(body.col, body.row + 3),
        &Span::new(&format!("seed   {:#x}", tower.seed())).with_style(Style::DIM),
    );
    inner.span(
        Pos::new(body.col, body.row + 4),
        &Span::new(&format!("grid   {}x{}", screen.grid.cols, screen.grid.rows))
            .with_style(Style::DIM),
    );

    let input = layout.input();
    frame.painter(input).span(
        input.origin(),
        &Span::new("orbs:~$ ").with_kind(UtteranceKind::Input),
    );
    frame.set_cursor(Some(Pos::new(input.col + 8, input.row)));

    info!("\n{}", frame.to_text());
}
