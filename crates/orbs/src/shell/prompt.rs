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
    DEEP_FOCUS_FLOOR, DisplayMode, FieldName, Frame, PROMPT, Pos, RecordKind, RecordView, Records,
    Rect, ScreenLayout, ScreenRequest, Span, Style, UtteranceKind,
};
use orbs_sim::Sim;

use super::input::Line;
use super::linear::Linear;
use super::screen::Screen;

/// Columns the telemetry pane shows.
///
/// `State` carries the one reading that is a word rather than a count. A row
/// without it leaves a gap, which is the record model's documented behaviour
/// rather than a special case — and putting `wide` in a `qty` column would be a
/// lie the model exists to make impossible.
const TELEMETRY: [FieldName; 3] = [FieldName::Name, FieldName::Quantity, FieldName::State];

/// Paint the session into `frame`.
///
/// **Two panes, from a real [`ScreenLayout`].** The game drew a single hand-built
/// rectangle until now, so §9's whole layout system — pane tiling, the sidebar,
/// Deep versus Wide focus — existed only in an example. Asking the layout for two
/// panes is what makes it something the binary exercises (§15's retroactive
/// gate), and it is also just the right screen: a session and a dashboard.
pub(crate) fn paint(
    frame: &mut Frame,
    sim: &Sim,
    line: &Line,
    screen: &Screen,
    linear: &mut Linear,
) {
    let grid = frame.size();
    // A second pane only where there is room for one. At the 80×22 floor a
    // secondary pane is a four-row strip (§9) — a border, a header and one row —
    // which is why §9 sets `DEEP_FOCUS_FLOOR` in the first place. Below it the
    // readings live in the session's border title instead, so nothing is lost;
    // above it they get a real table. Drag the window across that line and watch
    // the pane appear.
    let panes = if grid.fits(DEEP_FOCUS_FLOOR) { 2 } else { 1 };
    let layout = ScreenLayout::compute(&ScreenRequest {
        grid,
        main_panes: panes,
        sidebar_panes: 0,
        mode: screen.mode,
    });

    let main = layout.main();
    let first = main.first().copied().unwrap_or(Rect::EMPTY);
    // The same rectangle, not a second pane: the point of §14's stream is that
    // it says the same thing as the cells, and a comparison you make by pressing
    // one key is a comparison you actually make.
    if linear.showing() {
        super::linear::paint(linear, frame, sim, screen, first, panes == 1);
    } else {
        session(frame, sim, screen, first, panes == 1);
    }
    telemetry(
        frame,
        sim,
        screen,
        main.get(1).copied().unwrap_or(Rect::EMPTY),
    );
    input_line(frame, layout.input(), line);
}

/// The transcript: what was typed and what came back.
pub(super) fn session(
    frame: &mut Frame,
    sim: &Sim,
    screen: &Screen,
    pane: Rect,
    carry_readings: bool,
) {
    if pane.is_empty() {
        return;
    }
    let mut painter = frame.painter(pane);
    // The hint is here because this is the state a player gets stuck in: below
    // `DEEP_FOCUS_FLOOR` there is only one pane, and nothing on screen would
    // otherwise say that a second one exists or how to reach it. A key with no
    // affordance is a key nobody presses.
    let title = if carry_readings {
        format!(
            "session  tick {}  tier {}  {}x{}  {}  F4 {}",
            sim.tick().get(),
            screen.fidelity.map_or(0, |tier| tier.scale()),
            screen.grid.cols,
            screen.grid.rows,
            focus(screen),
            match screen.mode {
                DisplayMode::Deep => "wide",
                DisplayMode::Wide => "deep",
            },
        )
    } else {
        format!(
            "session  F4 {}",
            match screen.mode {
                DisplayMode::Deep => "wide",
                DisplayMode::Wide => "deep",
            }
        )
    };
    painter.border(pane, Some(&title), Style::DIM);

    let body = pane.inset(1);
    let records = sim.scrollback().records();
    // Only the tail fits. `iter().skip(n)` is O(1) here and stays `Clone`, which
    // is what `RecordView::draw` needs to measure and then draw.
    let skipped = records.len().saturating_sub(usize::from(body.rows));
    RecordView::prompt().draw(&mut painter, body, records.iter().skip(skipped));
}

/// What the orb and the tube are currently doing.
///
/// Drawn with [`RecordView::table`] rather than by formatting a string, so the
/// numbers stay numbers all the way to the cell that draws them — right-aligned
/// because the record said they were counts, not because this function did.
///
/// This is the playability gate for three subsystems at once (§15). A changing
/// **tick** is the only visible proof the sim runs; **tier and grid** make §9's
/// fidelity table something you walk through by dragging a window edge; and the
/// **focus mode** is §9's setting, which the design requires be overridable at
/// any time.
fn telemetry(frame: &mut Frame, sim: &Sim, screen: &Screen, pane: Rect) {
    if pane.is_empty() {
        return;
    }
    let mut painter = frame.painter(pane);
    painter.border(pane, Some("orb"), Style::DIM);

    let mut readings = Records::new();
    let mut row = |name: &str, value: u64| {
        readings
            .push(RecordKind::Status)
            .text(FieldName::Name, name)
            .count(FieldName::Quantity, value)
            .finish();
    };
    // Ordered by what a glance most wants, because in Wide focus this pane is a
    // four-row strip (§9) and the table truncates. `status` is the full answer;
    // this is the glance.
    row("tick", sim.tick().get());
    if let Some(tier) = screen.fidelity {
        row("tier", u64::from(tier.scale()));
    }
    row("cols", u64::from(screen.grid.cols));
    row("rows", u64::from(screen.grid.rows));
    row("logged", quantity(sim.scrollback().records().len()));
    row("queued", quantity(sim.pending().len()));
    row("seed", sim.seed());

    readings
        .push(RecordKind::Status)
        .text(FieldName::Name, "focus")
        .text(FieldName::State, focus(screen))
        .finish();

    RecordView::table(&TELEMETRY).draw(&mut painter, pane.inset(1), readings.iter());
}

/// §9's focus mode, as a word.
fn focus(screen: &Screen) -> &'static str {
    match screen.mode {
        DisplayMode::Deep => "deep",
        DisplayMode::Wide => "wide",
    }
}

fn quantity(count: usize) -> u64 {
    u64::try_from(count).unwrap_or(u64::MAX)
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
