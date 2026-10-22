//! What reaches the wire: every escape, colour and weight, byte for byte.

use orbs_render::{Frame, GridSize, Pos, Span, Style};

use super::Screen;

/// Draw a frame into a `Vec<u8>` and hand back what was written.
///
/// Colour forced on, never off: crossterm reads `NO_COLOR` from the shell that
/// ran the tests, and every test here is about what a coloured terminal is
/// sent. The switch is global, so no test may turn it off.
pub(super) fn drawn(screen: &mut Screen, frame: &Frame) -> String {
    crossterm::style::force_color_output(true);
    let mut out = Vec::new();
    screen.draw(frame, &mut out).expect("a Vec cannot fail");
    String::from_utf8(out).expect("crossterm writes utf-8")
}

/// Every style channel the rasteriser writes, byte for byte.
///
/// A characterisation of the wire, so a change to the order or the encoding of
/// any escape is a decision somebody sees in a diff rather than a drift.
#[test]
fn a_frame_reaches_the_wire_byte_for_byte() {
    use orbs_render::{Lexeme, Rect, Tint, Wash};

    let grid = GridSize::new(12, 2);
    let mut frame = Frame::new(grid);
    {
        let mut painter = frame.painter(grid.to_rect());
        painter.span(Pos::new(0, 0), &Span::new("ab").with_style(Style::NORMAL));
        painter.span(Pos::new(2, 0), &Span::new("cd").with_style(Style::DIM));
        painter.span(Pos::new(4, 0), &Span::new("ef").with_style(Style::BRIGHT));
        painter.span(Pos::new(6, 0), &Span::new("!").with_style(Style::DANGER));
        painter.span(
            Pos::new(0, 1),
            &Span::new("repeat").with_style(Style::NORMAL),
        );
        painter.span(Pos::new(7, 1), &Span::new("██").with_style(Style::NORMAL));
    }
    frame.lit(Rect::new(0, 1, 6, 1), Lexeme::Control);
    frame.set_tint(Rect::new(7, 1, 2, 1), Wash::plain(Tint::Green));
    frame.set_cursor(Some(Pos::new(11, 1)));

    let written = drawn(&mut Screen::new(grid, true), &frame);
    assert_eq!(written, WIRE, "the wire changed:\n{written:?}");
}

const WIRE: &str = "\u{1b}[1;1H\u{1b}[0m\u{1b}[39mab\u{1b}[0m\u{1b}[39m\u{1b}[2mcd\u{1b}[0m\u{1b}[39m\u{1b}[1mef\u{1b}[0m\u{1b}[38;5;9m!\u{1b}[2;1H\u{1b}[0m\u{1b}[38;5;13mrepeat\u{1b}[2;8H\u{1b}[0m\u{1b}[38;5;10m██\u{1b}[2;12H\u{1b}[?25h";

/// `NO_COLOR` turns crossterm's colour into `ESC[m`, which is SGR 0: written
/// after the weight it wiped it, and weight is what survives a terminal with no
/// colour. Read off the bytes, since the switch that would show it is global.
#[test]
fn colour_comes_before_weight_so_no_color_cannot_erase_it() {
    use orbs_render::Depiction;

    let grid = GridSize::new(4, 1);
    let mut frame = Frame::new(grid);
    frame.painter(grid.to_rect()).span(
        Pos::new(0, 0),
        &Span::new("█").with_style(Style::NORMAL.with_depiction(Depiction::FlameEmber)),
    );
    let written = drawn(&mut Screen::new(grid, true), &frame);
    let colour = written
        .find("\x1b[38;5;3m")
        .expect("the ember took no colour");
    let weight = written.find("\x1b[2m").expect("the ember took no weight");
    assert!(
        colour < weight,
        "the weight was written before the colour that would erase it: {written:?}",
    );
}

/// `sight greyscale` sends no hue: every colour on the wire is one of the
/// terminal's greys, and the weights are still there.
#[test]
fn greyscale_writes_no_hue_and_keeps_weight() {
    use orbs_render::Depiction;

    let grid = GridSize::new(8, 1);
    let mut frame = Frame::new(grid);
    {
        let mut painter = frame.painter(grid.to_rect());
        painter.span(
            Pos::new(0, 0),
            &Span::new("█").with_style(Style::NORMAL.with_depiction(Depiction::FlameEmber)),
        );
        painter.span(Pos::new(2, 0), &Span::new("!").with_style(Style::DANGER));
        painter.span(Pos::new(4, 0), &Span::new("✓").with_style(Style::SUCCESS));
    }
    let mut screen = Screen::new(grid, true);
    screen.set_greyscale(true);
    let written = drawn(&mut screen, &frame);
    for code in written.split("\x1b[38;5;").skip(1) {
        let index: u8 = code
            .split('m')
            .next()
            .and_then(|n| n.parse().ok())
            .expect("an indexed colour");
        assert!(
            matches!(index, 7 | 8 | 15),
            "greyscale sent colour {index}:\n{written:?}",
        );
    }
    assert!(
        written.contains("\x1b[2m"),
        "the ember lost its weight with its hue"
    );
}

#[test]
fn toggling_greyscale_redraws_only_the_coloured_cells() {
    let grid = GridSize::new(8, 1);
    let mut frame = Frame::new(grid);
    {
        let mut painter = frame.painter(grid.to_rect());
        painter.span(Pos::new(0, 0), &Span::new("ab").with_style(Style::NORMAL));
        painter.span(Pos::new(3, 0), &Span::new("!").with_style(Style::DANGER));
    }
    let mut screen = Screen::new(grid, true);
    let _ = drawn(&mut screen, &frame);
    screen.set_greyscale(true);
    let after = drawn(&mut screen, &frame);
    assert!(after.contains('!'), "the danger mark kept its hue");
    assert!(
        !after.contains('a'),
        "plain text was redrawn for a change it does not have: {after:?}",
    );
}

/// A lit run reaches the wire as a colour.
///
/// End to end, because every link is somewhere else: `parser::lexeme`
/// classifies, `sheet` registers the region, `Frame` stores it, `theme`
/// picks the colour and this writes it. A break anywhere looks like a spell
/// before any of it existed.
#[test]
fn a_lit_run_is_written_in_its_own_colour() {
    let grid = GridSize::new(20, 3);
    let mut frame = Frame::new(grid);
    frame.painter(grid.to_rect()).span(
        Pos::new(0, 0),
        &Span::new("repeat").with_style(Style::NORMAL),
    );
    frame.lit(
        orbs_render::Rect::new(0, 0, 6, 1),
        orbs_render::Lexeme::Control,
    );

    let mut screen = Screen::new(grid, true);
    let written = drawn(&mut screen, &frame);
    // `38;5;13`, not `35`: crossterm writes every named colour through the
    // 256-colour form, so a reader written against `3x` sees no colour at
    // all and reports the feature missing.
    assert!(
        written.contains("\x1b[38;5;13m"),
        "a control word reached the terminal with no colour on it:\n{written:?}",
    );
}
