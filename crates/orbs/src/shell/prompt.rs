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
    DisplayMode, FieldName, Frame, Pos, RecordKind, RecordView, Records, Rect, Span, Style,
    UtteranceKind,
};
use orbs_sim::Sim;

use super::input::Line;
use super::linear::Linear;
use super::reveal::Reveal;
use super::screen::Screen;
use super::transition::PaneTransition;

/// Columns the telemetry pane shows.
///
/// `State` carries the one reading that is a word rather than a count. A row
/// without it leaves a gap, which is the record model's documented behaviour
/// rather than a special case — and putting `wide` in a `qty` column would be a
/// lie the model exists to make impossible.
const TELEMETRY: [FieldName; 3] = [FieldName::Name, FieldName::Quantity, FieldName::State];

/// Paint the session into `frame`.
///
/// **Two panes, from a real `ScreenLayout`.** The game drew a single hand-built
/// rectangle until now, so §9's whole layout system — pane tiling, the sidebar,
/// Deep versus Wide focus — existed only in an example. Asking the layout for two
/// panes is what makes it something the binary exercises (§15's retroactive
/// gate), and it is also just the right screen: a session and a dashboard.
///
/// The layout arrives already interpolated: a pane appearing or leaving does so
/// over a fraction of a second (see [`PaneTransition`]), and every rectangle here
/// is wherever that motion has reached this frame.
pub(crate) fn paint(
    frame: &mut Frame,
    sim: &Sim,
    line: &Line,
    screen: &Screen,
    linear: &mut Linear,
    panes: &PaneTransition,
    reveal: &Reveal,
) {
    let grid = frame.size();
    let layout = panes.layout(grid, screen.mode);
    let main = layout.main();
    let first = main.first().copied().unwrap_or(Rect::EMPTY);

    // **The target's pane count, not the interpolated one.** Below
    // `DEEP_FOCUS_FLOOR` the readings live in the session's border title; above
    // it they get a real telemetry pane. Deciding that from where the animation
    // has *reached* would pop the title from its long form to its short one
    // partway through the motion. Switching once, when the motion starts, reads
    // as part of the same movement — and for the quarter-second a pane is
    // leaving, the readings are briefly in both places, which is the harmless
    // direction to be wrong in.
    let carry_readings = panes.panes() == 1;

    // The same rectangle, not a second pane: the point of §14's stream is that
    // it says the same thing as the cells, and a comparison you make by pressing
    // one key is a comparison you actually make.
    if linear.showing() {
        super::linear::paint(linear, frame, sim, screen, first, carry_readings);
    } else {
        session(frame, sim, screen, first, carry_readings, reveal);
    }
    if let Some(second) = main.get(1) {
        telemetry(frame, sim, screen, *second);
    }
    input_line(frame, layout.input(), line, &sim.prompt());
}

/// The transcript: what was typed and what came back.
pub(super) fn session(
    frame: &mut Frame,
    sim: &Sim,
    screen: &Screen,
    pane: Rect,
    carry_readings: bool,
    reveal: &Reveal,
) {
    if pane.is_empty() {
        return;
    }
    let mut painter = frame.painter(pane);
    // **The pane says where you are**, because the pane is the thing that shows
    // a place (§7: paths are places). The prompt below cannot: §9 puts one input
    // line beneath however many panes are open, so it serves all of them and can
    // claim to be standing in none.
    //
    // This is also the only thing on screen that tells a player *which commands
    // will work* — the essences live in `/tower/alembic`, so that is where
    // `decoct` resolves. See `orbs_sim::tower::rebuild`.
    //
    // The `F4` hint rides along because below `DEEP_FOCUS_FLOOR` there is only
    // one pane and nothing would otherwise say a second exists. A key with no
    // affordance is a key nobody presses.
    let switch = match screen.mode {
        DisplayMode::Deep => "wide",
        DisplayMode::Wide => "deep",
    };
    let title = if carry_readings {
        format!(
            "{}  tick {}  tier {}  {}x{}  {}  F4 {switch}",
            sim.location(),
            sim.tick().get(),
            screen.fidelity.map_or(0, |tier| tier.scale()),
            screen.grid.cols,
            screen.grid.rows,
            focus(screen),
        )
    } else {
        format!("{}  F4 {switch}", sim.location())
    };
    painter.border(pane, Some(&title), Style::DIM);

    let mut body = pane.inset(1);

    // §5.0's economy, made visible: an action occupies its slot for a duration,
    // and a meter is the only thing on screen that says how much of it is left.
    // §14 names progress bars specifically, and `Painter::progress` had lived in
    // an example since the Frame boundary landed because nothing had a duration
    // to show.
    if let Some(working) = sim.working() {
        let (done, total) = working.progress(sim.tick());
        let row = Rect::new(body.col, body.bottom().saturating_sub(1), body.cols, 1);
        body = Rect::new(body.col, body.row, body.cols, body.rows.saturating_sub(1));

        // The meter is drawn; the *sentence* is what a reader hears. §14 wants a
        // description rather than a row of block glyphs, and `progress` takes
        // both at the same call site so one cannot be written without the other.
        let spoken = format!("{} {} of {} ticks", working.verb.canonical(), done, total,);
        painter.progress(row, to_u32(done), to_u32(total), Style::COST, &spoken);
    }

    let records = sim.scrollback().records();
    let prompt = sim.prompt();
    let mut view = RecordView::prompt(&prompt);
    // Only the tail fits. `iter().skip(n)` is O(1) here and stays `Clone`, which
    // is what `RecordView::draw` needs to measure and then draw.
    //
    // One row per record is a *floor*, not the height: a listing packs across
    // the pane, so this skip always fits and usually wastes the difference.
    // Widening it is what puts real history in the rows tiling frees up — a
    // session that had five blank rows and dropped its own opening shows both.
    //
    // Binary search rather than a walk. `height` is **non-increasing** in the
    // skip — restoring an older record adds to a run's count and can only widen
    // its columns, so it can only cost rows — which makes "the smallest skip
    // that still fits" a monotone predicate. Walking it re-measured the whole
    // tail per step, which is quadratic in a per-frame path; this is about five
    // measurements at the 80×22 floor.
    let (mut narrowest, mut widest) = (0, records.len().saturating_sub(usize::from(body.rows)));
    while narrowest < widest {
        let candidate = narrowest + (widest - narrowest) / 2;
        if view.height(body.cols, records.iter().skip(candidate)) <= body.rows {
            widest = candidate;
        } else {
            narrowest = candidate + 1;
        }
    }
    // Measured *before* the reveal is applied, so the tail that fits is the one
    // the finished output will need. Sizing the pane against half-arrived text
    // would make it reflow as the rest turned up.
    if let Some((after, cells)) = reveal.budget(narrowest) {
        view = view.revealing(after, cells);
    }
    view.draw(&mut painter, body, records.iter().skip(narrowest));
}

/// A tick count as a meter value, saturating rather than wrapping.
fn to_u32(ticks: u64) -> u32 {
    u32::try_from(ticks).unwrap_or(u32::MAX)
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
fn input_line(frame: &mut Frame, area: Rect, line: &Line, prompt: &str) {
    if area.is_empty() {
        return;
    }
    let mut painter = frame.painter(area);
    let prompt = painter.glyphs(area.origin(), prompt, Style::DIM);

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
