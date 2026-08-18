//! The Frame boundary, exercised through the public API.
//!
//! These are the rules the boundary exists to enforce, rather than the unit
//! behaviour of any one type. Each names the design clause it protects, because
//! a test that fails without saying what it was defending gets deleted.

use orbs_render::{
    Cell, DisplayMode, Frame, GridSize, Intensity, Pos, Presentation, Rect, Role, ScreenLayout,
    ScreenRequest, Span, Style, UtteranceKind, cp437,
};

fn frame(cols: u16, rows: u16) -> Frame {
    Frame::new(GridSize::new(cols, rows))
}

fn spoken(frame: &Frame) -> Vec<(UtteranceKind, Role, String)> {
    frame
        .speech()
        .utterances()
        .map(|utterance| (utterance.kind, utterance.role, utterance.text.to_owned()))
        .collect()
}

fn glyphs(frame: &Frame, row: u16) -> String {
    frame
        .row(row)
        .expect("row exists")
        .iter()
        .map(|cell| cell.glyph)
        .collect()
}

// ---------------------------------------------------------------------------
// Clipping — a pane cannot corrupt its neighbour
// ---------------------------------------------------------------------------

#[test]
fn a_painter_cannot_write_outside_its_region() {
    let mut frame = frame(20, 3);

    // Two side-by-side panes; the left one is handed text far too long for it.
    frame
        .painter(Rect::new(0, 0, 10, 3))
        .span(Pos::new(0, 1), &Span::new("0123456789ABCDEFGHIJ"));
    frame
        .painter(Rect::new(10, 0, 10, 3))
        .span(Pos::new(10, 1), &Span::new("right"));

    assert_eq!(glyphs(&frame, 1), "0123456789right     ");
}

#[test]
fn a_span_starting_left_of_the_region_is_consumed_not_shifted() {
    let mut frame = frame(12, 1);
    frame
        .painter(Rect::new(4, 0, 4, 1))
        .span(Pos::new(0, 0), &Span::new("abcdefghijkl"));

    // 'e','f','g','h' land in columns 4..8; nothing slides right.
    assert_eq!(glyphs(&frame, 0), "    efgh    ");
}

#[test]
fn a_sub_painter_is_clipped_to_its_parent() {
    let mut frame = frame(20, 3);
    let mut pane = frame.painter(Rect::new(0, 0, 10, 3));
    let mut inner = pane.sub(Rect::new(5, 0, 100, 3));

    assert_eq!(inner.area(), Rect::new(5, 0, 5, 3));
    inner.span(Pos::new(5, 0), &Span::new("overlong"));

    assert_eq!(glyphs(&frame, 0), "     overl          ");
}

// ---------------------------------------------------------------------------
// Linearisation — DESIGN.md §14
// ---------------------------------------------------------------------------

#[test]
fn structure_is_silent_and_content_speaks() {
    let mut frame = frame(20, 5);
    let area = Rect::new(0, 0, 20, 5);
    let mut painter = frame.painter(area);

    painter.clear();
    painter.fill(Rect::new(0, 4, 20, 1), '─', Style::DIM);
    painter.border(area, Some("laboratory"), Style::DIM);
    painter.span(Pos::new(2, 2), &Span::new("the brew settles"));

    // The border, the rule, and the blanking contribute nothing to speak; the
    // title and the line of prose do.
    assert_eq!(
        spoken(&frame),
        [
            (
                UtteranceKind::Heading,
                Role::Normal,
                "laboratory".to_owned()
            ),
            (
                UtteranceKind::Text,
                Role::Normal,
                "the brew settles".to_owned()
            ),
        ]
    );
}

#[test]
fn truncated_text_is_spoken_in_full() {
    // A narrow pane is a visual constraint. Withholding the rest of the sentence
    // from a screen reader would make it an informational one.
    let mut frame = frame(8, 1);
    frame.painter(Rect::new(0, 0, 8, 1)).span(
        Pos::new(0, 0),
        &Span::new("the east wall has been breached"),
    );

    assert_eq!(glyphs(&frame, 0), "the east");
    assert_eq!(
        frame
            .speech()
            .utterances()
            .next()
            .expect("one utterance")
            .text,
        "the east wall has been breached"
    );
}

#[test]
fn eldritch_output_speaks_its_authored_variant() {
    // §3: eldritch achieves its effect through diction and cadence in the linear
    // channel, never through the glyph damage a reader cannot perceive.
    let mut frame = frame(40, 1);
    let eldritch = Style::NORMAL.with_presentation(Presentation::Eldritch);

    frame.painter(Rect::new(0, 0, 40, 1)).span(
        Pos::new(0, 0),
        &Span::new("t h e   p r o m p t   a n s w e r s")
            .with_style(eldritch)
            .with_spoken("The prompt answers. It should not."),
    );

    assert!(glyphs(&frame, 0).starts_with("t h e"));
    assert_eq!(
        frame
            .speech()
            .utterances()
            .next()
            .expect("one utterance")
            .text,
        "The prompt answers. It should not."
    );
}

#[test]
fn a_progress_bar_speaks_a_description_rather_than_its_glyphs() {
    // §14 names progress bars specifically. A row of block characters is not
    // consumable as speech.
    let mut frame = frame(10, 1);
    frame.painter(Rect::new(0, 0, 10, 1)).progress(
        Rect::new(0, 0, 10, 1),
        34,
        100,
        Style::DANGER,
        "east wall integrity 34 percent",
    );

    assert_eq!(glyphs(&frame, 0), "███░░░░░░░");
    assert_eq!(
        spoken(&frame),
        [(
            UtteranceKind::Progress,
            Role::Danger,
            "east wall integrity 34 percent".to_owned()
        )]
    );
}

#[test]
fn role_reaches_the_linear_stream_without_a_single_pixel() {
    // §14: no meaning conveyed by colour alone.
    let mut frame = frame(30, 3);
    let mut painter = frame.painter(Rect::new(0, 0, 30, 3));
    painter.span(
        Pos::new(0, 0),
        &Span::new("ward failed").with_style(Style::DANGER),
    );
    painter.span(
        Pos::new(0, 1),
        &Span::new("42 mana").with_style(Style::COST),
    );
    painter.span(
        Pos::new(0, 2),
        &Span::new("haste decocted")
            .with_style(Style::SUCCESS)
            .with_kind(UtteranceKind::Completion),
    );

    let roles: Vec<_> = frame.speech().utterances().map(|u| u.role).collect();
    assert_eq!(roles, [Role::Danger, Role::Cost, Role::Success]);
    assert_eq!(
        frame.speech().utterances().last().expect("three").kind,
        UtteranceKind::Completion
    );
}

#[test]
fn announcements_speak_without_drawing() {
    let mut frame = frame(10, 2);
    frame.painter(Rect::new(0, 0, 10, 2)).announce(
        UtteranceKind::Heading,
        Role::Normal,
        "bound scripts",
    );

    assert_eq!(frame.to_text(), "          \n          \n");
    assert_eq!(frame.speech().len(), 1);
}

#[test]
fn resetting_a_frame_clears_the_linear_stream_too() {
    // Speech that survived a reset would be spoken twice, or attributed to the
    // wrong tick.
    let mut frame = frame(10, 2);
    frame
        .painter(Rect::new(0, 0, 10, 2))
        .span(Pos::ORIGIN, &Span::new("first"));
    assert_eq!(frame.speech().len(), 1);

    frame.reset(GridSize::new(10, 2));
    assert!(frame.speech().is_empty());
    assert_eq!(frame.to_text(), "          \n          \n");
}

// ---------------------------------------------------------------------------
// Styling — no colour, ever
// ---------------------------------------------------------------------------

#[test]
fn style_survives_onto_the_cell_it_was_painted_with() {
    let mut frame = frame(10, 1);
    let style = Style::DANGER
        .with_intensity(Intensity::Bright)
        .with_presentation(Presentation::Tampered);

    frame
        .painter(Rect::new(0, 0, 10, 1))
        .span(Pos::ORIGIN, &Span::new("x").with_style(style));

    let cell = frame.cell(Pos::ORIGIN).expect("painted");
    assert_eq!(cell.style, style);
    assert_eq!(cell.style.role, Role::Danger);
    assert_eq!(cell.style.intensity, Intensity::Bright);
    assert_eq!(cell.style.presentation, Presentation::Tampered);
}

// ---------------------------------------------------------------------------
// The alphabet — DESIGN.md §4
// ---------------------------------------------------------------------------

#[test]
fn every_glyph_a_frame_can_hold_is_in_the_repertoire() {
    // The Bevy frontend looks each cell up in an 8x16 CP437 atlas. A glyph with
    // no atlas entry is a hole in the screen.
    let mut frame = frame(24, 2);
    let mut painter = frame.painter(Rect::new(0, 0, 24, 2));
    painter.border(Rect::new(0, 0, 24, 2), Some("Σ ░▒▓"), Style::DIM);
    painter.span(Pos::new(1, 1), &Span::new("漢字\tand emoji 😀"));

    for row in frame.rows() {
        for cell in row {
            assert!(
                cp437::is_renderable(cell.glyph),
                "{:?} has no CP437 glyph",
                cell.glyph
            );
        }
    }
}

#[test]
fn unrenderable_input_is_substituted_rather_than_dropped() {
    let mut frame = frame(6, 1);
    frame
        .painter(Rect::new(0, 0, 6, 1))
        .span(Pos::ORIGIN, &Span::new("a漢b\tc"));

    // One cell per source character: substitution must not shift the row.
    assert_eq!(
        glyphs(&frame, 0),
        format!("a{r}b{r}c ", r = cp437::REPLACEMENT)
    );
}

// ---------------------------------------------------------------------------
// A whole screen
// ---------------------------------------------------------------------------

#[test]
fn a_full_screen_linearises_in_paint_order() {
    // **The game's own grid, and it was an arbitrary 120×33.** The rail needs
    // `MIN_RAIL_BOX` rows per domain across seven domains, so a short grid drops
    // it entirely — and a test that asserts rail speech on a grid with no rail is
    // asserting nothing. 120×45 is the only grid the game has (§19).
    let grid = GridSize::new(120, 45);
    let layout = ScreenLayout::compute(&ScreenRequest {
        grid,
        main_panes: 2,
        rail: true,
        mode: DisplayMode::Deep,
        input_rows: 1,
    });

    let mut frame = Frame::new(grid);

    for (pane, title) in layout.main().iter().zip(["laboratory", "battlements"]) {
        let mut painter = frame.painter(*pane);
        painter.border(*pane, Some(title), Style::DIM);
        painter.paragraph(
            pane.inset(1),
            &Span::new("the laboratory seethes and will not settle"),
        );
    }

    for (slot, name) in layout.rail_boxes().iter().zip(["archive", "menagerie"]) {
        frame.painter(*slot).span(
            slot.origin(),
            &Span::new(name).with_kind(UtteranceKind::Heading),
        );
    }

    let input = layout.input();
    frame.painter(input).span(
        input.origin(),
        &Span::new("orbs:~$ ").with_kind(UtteranceKind::Input),
    );
    frame.set_cursor(Some(Pos::new(input.col + 8, input.row)));

    let texts: Vec<_> = frame
        .speech()
        .utterances()
        .map(|utterance| utterance.text)
        .collect();
    assert_eq!(
        texts,
        [
            "laboratory",
            "the laboratory seethes and will not settle",
            "battlements",
            "the laboratory seethes and will not settle",
            "archive",
            "menagerie",
            "orbs:~$ ",
        ]
    );

    // Column 9, not 8: the input line is inset one cell from the bottom-left
    // corner, where a curved tube distorts most. See `ScreenLayout::compute`.
    assert_eq!(frame.cursor(), Some(Pos::new(9, 44)));
    assert_eq!(frame.cell(Pos::ORIGIN), Some(&Cell::new('┌', Style::DIM)));
}

#[test]
fn panes_drawn_from_a_layout_never_bleed_into_each_other() {
    // 120×45 for the reason above: the assertion below refuses to run without a
    // rail, and a short grid does not host one.
    let grid = GridSize::new(120, 45);
    let layout = ScreenLayout::compute(&ScreenRequest {
        grid,
        main_panes: 4,
        rail: true,
        mode: DisplayMode::Deep,
        input_rows: 1,
    });

    let mut frame = Frame::new(grid);
    // Every pane tries to overrun its bounds in both axes.
    let filler = "X".repeat(400);
    for pane in layout.main() {
        frame
            .painter(*pane)
            .paragraph(Rect::new(pane.col, pane.row, 400, 400), &Span::new(&filler));
    }

    let blank = |frame: &Frame, row: u16| {
        frame
            .row(row)
            .expect("row exists")
            .iter()
            .all(|cell| cell.is_blank())
    };

    // **The rail is checked by column, where the sidebar was checked by row.**
    // That is the whole shape change as a test: a rail shares every row with the
    // main window, so a pane bleeding rightwards would show up here and nowhere
    // else — and it is precisely the bleed a full-width sidebar could never have
    // caught.
    let rail = layout.rail();
    assert!(
        !rail.is_empty(),
        "the rail did not fit, so nothing is tested"
    );
    for row in rail.row..rail.bottom() {
        for col in rail.col..rail.right() {
            let cell = frame.cell(Pos::new(col, row));
            assert!(
                cell.is_none_or(|cell| cell.is_blank()),
                "a pane overran into the rail at {col},{row}",
            );
        }
    }
    assert!(
        blank(&frame, layout.input().row),
        "the input line was overrun"
    );
}

/// Every glyph a border draws, as a set of positions.
fn drawn(frame: &Frame) -> Vec<(u16, u16, char)> {
    let size = frame.size();
    (0..size.rows)
        .flat_map(|row| {
            (0..size.cols).filter_map(move |col| {
                frame
                    .row(row)
                    .and_then(|cells| cells.get(usize::from(col)).copied())
                    .filter(|cell| cell.glyph != ' ')
                    .map(|cell| (col, row, cell.glyph))
            })
        })
        .collect()
}

#[test]
fn a_finished_reveal_is_exactly_a_drawn_border() {
    // The property that lets the boot sequence hand over to the game without a
    // seam: the last frame of the animation and the first frame of the real
    // screen are the same cells. Nothing tested this at all while the reveal was
    // a single walk from the top-left, which is how it could be rewritten into
    // four runs with no failure to warn on a missed corner.
    let area = Rect::new(0, 0, 40, 12);

    let mut revealed = frame(40, 12);
    revealed
        .painter(area)
        .border_revealed(area, Style::DIM, 1.0);

    let mut plain = frame(40, 12);
    plain.painter(area).border(area, None, Style::DIM);

    assert_eq!(
        drawn(&revealed),
        drawn(&plain),
        "a fully revealed border is not the border",
    );
}

#[test]
fn a_reveal_grows_from_all_four_corners() {
    // **The point of the change.** A single line from the top-left leaves three
    // corners dark for most of the animation; four runs put every corner down in
    // the first few cells and close on the midpoints of nothing — they meet each
    // other. One tenth of the way in is early enough that a single walk could
    // not have reached even the second corner of a 40-cell box.
    let area = Rect::new(0, 0, 40, 12);
    let mut frame = frame(40, 12);
    frame.painter(area).border_revealed(area, Style::DIM, 0.10);

    let corners: [(u16, u16); 4] = [(0, 0), (39, 0), (39, 11), (0, 11)];
    for (col, row) in corners {
        let glyph = frame
            .row(row)
            .and_then(|cells| cells.get(usize::from(col)).copied())
            .map(|cell| cell.glyph);
        assert!(
            glyph.is_some_and(|glyph| glyph != ' '),
            "corner ({col}, {row}) had not started at one tenth",
        );
    }
}

#[test]
fn the_four_sides_of_a_reveal_close_together() {
    // Each side is drawn `progress` of **its own** length, so the short sides do
    // not finish early and sit waiting. Measured as "every side is partly drawn
    // and none is complete" at the halfway point — a shared cells-per-second
    // pace would have the 12-row sides done and the 40-column ones half done.
    let area = Rect::new(0, 0, 40, 12);
    let mut frame = frame(40, 12);
    frame.painter(area).border_revealed(area, Style::DIM, 0.5);

    let filled = |cells: &[(u16, u16, char)], on_side: fn(u16, u16) -> bool| -> usize {
        cells
            .iter()
            .filter(|(col, row, _)| on_side(*col, *row))
            .count()
    };
    let cells = drawn(&frame);
    let top = filled(&cells, |_, row| row == 0);
    let bottom = filled(&cells, |_, row| row == 11);
    let left = filled(&cells, |col, _| col == 0);
    let right = filled(&cells, |col, _| col == 39);

    assert!(top > 1 && top < 40, "the top is {top} of 40");
    assert!(bottom > 1 && bottom < 40, "the bottom is {bottom} of 40");
    assert!(left > 1 && left < 12, "the left is {left} of 12");
    assert!(right > 1 && right < 12, "the right is {right} of 12");
}

/// A listing of `name = amount` entries, the way `survey` emits one.
fn shelf(records: &mut orbs_render::Records, stock: &[(&str, &str)]) {
    records
        .push(orbs_render::RecordKind::Section)
        .text(orbs_render::FieldName::Kind, "reagent")
        .finish();
    for (name, amount) in stock {
        records
            .push(orbs_render::RecordKind::Entry)
            .text(orbs_render::FieldName::Name, name)
            .text(orbs_render::FieldName::Quantity, amount)
            .finish();
    }
}

#[test]
fn a_listing_aligns_its_amounts_into_one_column() {
    // The whole point of the `=`: the eye runs down it. Names differ in length,
    // so the column only exists if every tile pads its name to the run's widest
    // — measuring the *rendered line* instead gives one width for the pair and
    // packs them tight, which is what this replaced.
    let mut records = orbs_render::Records::new();
    shelf(
        &mut records,
        &[("sage", "\u{221e}"), ("ground-sage", "2"), ("husks", "12")],
    );

    let mut frame = frame(60, 6);
    let area = Rect::new(0, 0, 60, 6);
    orbs_render::RecordView::lines().draw(&mut frame.painter(area), area, records.iter());

    let rows: Vec<String> = (0..6).map(|row| glyphs(&frame, row)).collect();
    // **Character positions, not byte offsets.** `∞` is three bytes, so
    // `match_indices` reports columns that drift by two per infinity on the row
    // — which looked exactly like a broken alignment and was a broken test.
    let bound: Vec<usize> = rows
        .iter()
        .flat_map(|row| {
            row.chars()
                .enumerate()
                .filter_map(|(at, glyph)| (glyph == '=').then_some(at))
        })
        .collect();
    assert!(bound.len() >= 3, "not every entry was bound: {rows:?}");
    // Every `=` sits at a column that is a whole number of strides from the
    // first — which is what "one column" means once the run wraps.
    assert!(
        bound
            .iter()
            .all(|at| (at - bound[0]).is_multiple_of(bound[1] - bound[0])),
        "the amounts did not line up: {bound:?} in {rows:?}",
    );
}

#[test]
fn a_listing_never_wraps_an_entry_across_two_lines() {
    // **The property the player asked for by name.** A tile is a whole stride
    // and `per_row` is a whole number of them, so an entry either gets its own
    // column or the run stacks — there is no arithmetic that can leave half a
    // name at the end of a row. Checked at several widths, because the failure
    // is a width-dependent off-by-one and one pane size would not find it.
    for cols in [24u16, 31, 40, 57, 80] {
        let mut records = orbs_render::Records::new();
        shelf(
            &mut records,
            &[
                ("sage", "\u{221e}"),
                ("ground-sage", "2"),
                ("husks", "12"),
                ("rock-salt", "\u{221e}"),
                ("charcoal", "\u{221e}"),
            ],
        );

        let mut frame = frame(cols, 10);
        let area = Rect::new(0, 0, cols, 10);
        orbs_render::RecordView::lines().draw(&mut frame.painter(area), area, records.iter());

        for row in 0..10 {
            let drawn = glyphs(&frame, row);
            let trimmed = drawn.trim_end();
            assert!(
                trimmed.chars().count() <= usize::from(cols),
                "a row overran {cols} columns: {trimmed:?}",
            );
            // A name split across the edge would leave the row ending mid-word
            // with the rest starting the next one. Every drawn name is whole.
            for name in ["ground-sage", "rock-salt", "charcoal", "husks", "sage"] {
                let broken = trimmed.ends_with(&name[..name.len() - 1]) && !trimmed.ends_with(name);
                assert!(!broken, "{name:?} was cut at {cols} columns: {trimmed:?}");
            }
        }
    }
}
