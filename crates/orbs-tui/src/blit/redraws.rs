//! What a redraw writes, and where: the diff, a resize, and the screen as a
//! terminal shows it.

use orbs_render::{Frame, GridSize, Pos, Span, Style};

use super::Screen;
use super::tests::drawn;

fn frame_saying(text: &str) -> Frame {
    let grid = GridSize::new(20, 3);
    let mut frame = Frame::new(grid);
    frame
        .painter(grid.to_rect())
        .span(Pos::new(0, 0), &Span::new(text).with_style(Style::NORMAL));
    frame
}

/// A wide-ambiguous terminal still gets one column a glyph: the circle's `▲`
/// and `←` keep the marks row under its numbers.
#[test]
fn a_wide_terminal_keeps_the_circle_in_its_columns() {
    let grid = GridSize::new(6, 1);
    let mut frame = Frame::new(grid);
    frame
        .painter(grid.to_rect())
        .span(Pos::new(0, 0), &Span::new("1▲2←3"));
    let written = drawn(&mut Screen::new(grid, false), &frame);
    assert_eq!(as_seen(&written, grid)[0].trim_end(), "1x2{3");
}

#[test]
fn an_unchanged_frame_writes_no_cells() {
    // A second identical frame costs a cursor placement and nothing else —
    // over `ssh`, a game rather than a smear.
    let mut screen = Screen::new(GridSize::new(20, 3), true);
    let frame = frame_saying("clarity");

    let first = drawn(&mut screen, &frame);
    assert!(first.contains('c'), "the first paint drew nothing");

    let second = drawn(&mut screen, &frame);
    assert!(
        !second.contains('c'),
        "an unchanged frame redrew its cells: {second:?}",
    );
}

#[test]
fn a_changed_tint_redraws_a_cell_whose_glyph_did_not_change() {
    // Tint is a region, so the flask's band changes colour under an
    // unchanged `█` and a `Cell` diff leaves the old colour on screen.
    use orbs_render::{Tint, Wash};

    let grid = GridSize::new(20, 3);
    let mut screen = Screen::new(grid, true);

    let mut plain = Frame::new(grid);
    plain
        .painter(grid.to_rect())
        .span(Pos::new(0, 0), &Span::new("███"));
    let _ = drawn(&mut screen, &plain);

    let mut tinted = Frame::new(grid);
    tinted
        .painter(grid.to_rect())
        .span(Pos::new(0, 0), &Span::new("███"));
    tinted.set_tint(grid.to_rect(), Wash::plain(Tint::Green));

    let second = drawn(&mut screen, &tinted);
    assert!(
        second.contains('█'),
        "the wash changed and nothing was redrawn: {second:?}",
    );
}

#[test]
fn a_resize_wipes_what_the_new_frame_leaves_blank() {
    // Shrinking a *full* screen is the failure, and growing a sparse one
    // misses it: the buffer says "all blank", so cells the new frame leaves
    // blank compare equal and are skipped. Asserted on the wipe, because no
    // per-cell writing undoes a glyph nobody writes over.
    let wide = GridSize::new(40, 3);
    let mut screen = Screen::new(wide, true);

    let mut full = Frame::new(wide);
    full.painter(wide.to_rect())
        .fill(wide.to_rect(), '█', Style::NORMAL);
    let first = drawn(&mut screen, &full);
    assert!(!first.contains("\u{1b}[2J"), "the opening frame wiped");
    assert!(first.contains('█'), "the first paint drew nothing");

    // Now the same screen, narrower and nearly empty.
    let narrow = GridSize::new(20, 3);
    let mut sparse = Frame::new(narrow);
    sparse
        .painter(narrow.to_rect())
        .span(Pos::new(0, 0), &Span::new("clarity"));

    let after = drawn(&mut screen, &sparse);
    assert!(
        after.contains("\u{1b}[2J"),
        "a resize left the old screen underneath: {after:?}",
    );
}

#[test]
fn a_resize_forgets_everything() {
    // Addressed by (col, row), so a carried buffer writes this frame's
    // cells at last frame's coordinates.
    let mut screen = Screen::new(GridSize::new(20, 3), true);
    let frame = frame_saying("clarity");
    let _ = drawn(&mut screen, &frame);

    let wider = GridSize::new(40, 3);
    let mut moved = Frame::new(wider);
    moved
        .painter(wider.to_rect())
        .span(Pos::new(0, 0), &Span::new("clarity"));

    let after = drawn(&mut screen, &moved);
    assert!(
        after.contains('c'),
        "a resize kept a stale shadow buffer: {after:?}",
    );
}

/// Reconstruct the grid from what was written to the terminal.
///
/// Follows `MoveTo` and `Print` and ignores every colour sequence.
fn as_seen(written: &str, grid: GridSize) -> Vec<String> {
    let mut cells = vec![' '; grid.area()];
    let (mut col, mut row) = (0u16, 0u16);
    let mut rest = written;
    while let Some(start) = rest.find('\u{1b}') {
        for glyph in rest[..start].chars() {
            if let Some(slot) = cells.get_mut(row as usize * grid.cols as usize + col as usize) {
                *slot = glyph;
            }
            col = col.saturating_add(1);
        }
        let after = &rest[start + 1..];
        let end = after
            .find(|c: char| c.is_ascii_alphabetic())
            .map_or(after.len(), |i| i + 1);
        let sequence = &after[..end];
        // `ESC [ <row> ; <col> H` — one-based, and the only one that moves us.
        if let Some(body) = sequence.strip_prefix('[').and_then(|s| s.strip_suffix('H')) {
            let mut parts = body.split(';');
            row = parts
                .next()
                .and_then(|n| n.parse::<u16>().ok())
                .unwrap_or(1)
                - 1;
            col = parts
                .next()
                .and_then(|n| n.parse::<u16>().ok())
                .unwrap_or(1)
                - 1;
        }
        rest = &after[end..];
    }
    for glyph in rest.chars() {
        if let Some(slot) = cells.get_mut(row as usize * grid.cols as usize + col as usize) {
            *slot = glyph;
        }
        col = col.saturating_add(1);
    }
    cells
        .chunks(grid.cols as usize)
        .map(|row| row.iter().collect())
        .collect()
}

#[test]
fn the_terminal_shows_the_frame_and_nothing_else() {
    // Rule 2 as an assertion: every glyph the Frame holds reaches the
    // terminal at the Frame's own coordinates, and no other glyph appears.
    // Checked against `Frame::to_text`, so a rasteriser cannot pass by
    // agreeing only with itself.
    use orbs_shell::{Bench, Line, Linear, Offered, PaneTransition, Panel, Reveal};

    let grid = GridSize::new(120, 45);
    let mut sim = orbs_sim::Sim::new(3);
    for line in ["attend laboratory", "kindle charcoal", "grind sage"] {
        sim.submit(line);
        sim.step();
    }

    // The running loop's own constructor, so this cannot check a frame
    // drawn in a mode the terminal is never in.
    let screen = orbs_shell::Screen::windowless(grid, None);
    let mut panel = Panel::default();
    panel.refresh(&sim);

    let mut frame = Frame::new(grid);
    let mut linear = Linear::default();
    orbs_shell::paint(
        &mut frame,
        &mut linear,
        &mut orbs_shell::Passing::default(),
        orbs_shell::View {
            sim: &sim,
            line: &Line::default(),
            screen: &screen,
            panes: &PaneTransition::settled(1),
            reveal: &Reveal::default(),
            offered: &Offered::default(),
            ghost: "",
            panel: &panel,
            scroll: &orbs_shell::Scroll::default(),
            bench: &Bench::default(),
            climb: &orbs_shell::Climb::default(),
            editing: None,
            weaving: None,
            walking: false,
            menuing: None,
            reading_manual: None,
            installing: None,
        },
    );

    // `narrow`, so no substitution runs: this is about placement.
    let written = drawn(&mut Screen::new(grid, true), &frame);
    let expected: Vec<String> = frame.to_text().lines().map(str::to_owned).collect();
    let seen = as_seen(&written, grid);
    assert_eq!(seen.len(), expected.len(), "wrong number of rows");
    for (row, (drawn, wanted)) in seen.iter().zip(&expected).enumerate() {
        assert_eq!(
            drawn.trim_end(),
            wanted.trim_end(),
            "row {row} reached the terminal differently from the Frame",
        );
    }
}
