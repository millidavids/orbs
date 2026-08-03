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
    painter.border(area, Some("alembic"), Style::DIM);
    painter.span(Pos::new(2, 2), &Span::new("the brew settles"));

    // The border, the rule, and the blanking contribute nothing to speak; the
    // title and the line of prose do.
    assert_eq!(
        spoken(&frame),
        [
            (UtteranceKind::Heading, Role::Normal, "alembic".to_owned()),
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
    let grid = GridSize::new(120, 33);
    let layout = ScreenLayout::compute(&ScreenRequest {
        grid,
        main_panes: 2,
        sidebar_panes: 2,
        mode: DisplayMode::Deep,
    });

    let mut frame = Frame::new(grid);

    for (pane, title) in layout.main().iter().zip(["alembic", "battlements"]) {
        let mut painter = frame.painter(*pane);
        painter.border(*pane, Some(title), Style::DIM);
        painter.paragraph(
            pane.inset(1),
            &Span::new("the alembic seethes and will not settle"),
        );
    }

    for (row, name) in layout.sidebar().iter().zip(["archive", "menagerie"]) {
        frame.painter(*row).span(
            row.origin(),
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
            "alembic",
            "the alembic seethes and will not settle",
            "battlements",
            "the alembic seethes and will not settle",
            "archive",
            "menagerie",
            "orbs:~$ ",
        ]
    );

    assert_eq!(frame.cursor(), Some(Pos::new(8, 32)));
    assert_eq!(frame.cell(Pos::ORIGIN), Some(&Cell::new('┌', Style::DIM)));
}

#[test]
fn panes_drawn_from_a_layout_never_bleed_into_each_other() {
    let grid = GridSize::new(120, 33);
    let layout = ScreenLayout::compute(&ScreenRequest {
        grid,
        main_panes: 4,
        sidebar_panes: 3,
        mode: DisplayMode::Deep,
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

    // The sidebar and the input line, which nothing painted, are untouched.
    for row in layout.sidebar() {
        assert!(
            blank(&frame, row.row),
            "sidebar row {} was overrun",
            row.row
        );
    }
    assert!(
        blank(&frame, layout.input().row),
        "the input line was overrun"
    );
}
