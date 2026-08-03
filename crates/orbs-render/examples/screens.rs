//! Real screens, drawn through the real pipeline, dumped as text.
//!
//! ```text
//! cargo run -p orbs-render --example screens
//! ```
//!
//! Nothing here is a frontend. Every screen is built with the same public API
//! the Bevy and terminal builds will use, and printed by walking `Frame::rows()`
//! exactly as a rasteriser would. What you see is what the GPU cell renderer
//! will be handed.
//!
//! The last section is the point of the whole crate: the same screen in Deep
//! focus and in Wide focus, and the proof that both produce an identical linear
//! stream. DESIGN.md §9 calls that parity mandatory — *"if strips ever showed
//! less, the setting would become a difficulty choice."*

use orbs_render::{
    DisplayMode, Fidelity, Frame, GridSize, Intensity, Painter, Pos, Presentation, Rect, Role,
    ScreenLayout, ScreenRequest, Span, Style, UtteranceKind,
};

fn main() {
    tier_table();
    lint_prose();

    let boot = boot_report(GridSize::new(80, 22));
    show("Boot report — 80×22, tier 1 (DESIGN.md §4)", &boot);
    speak(&boot);

    let deep = siege(GridSize::new(120, 33), DisplayMode::Deep);
    show("Siege — Deep focus, 120×33 (tier 2 at 1080p)", &deep);

    let wide = siege(GridSize::new(80, 22), DisplayMode::Wide);
    show("Siege — Wide focus, 80×22 (tier 1, large text)", &wide);
    speak(&wide);

    parity(&deep, &wide);
}

// ---------------------------------------------------------------------------
// Screens
// ---------------------------------------------------------------------------

/// The status report from DESIGN.md §4, reflecting a damaged tower.
fn boot_report(grid: GridSize) -> Frame {
    let mut frame = Frame::new(grid);
    let layout = ScreenLayout::compute(&ScreenRequest::single(grid));
    let area = layout.main().first().copied().unwrap_or(Rect::EMPTY);

    let mut painter = frame.painter(area);
    // Note the ASCII dashes. DESIGN.md §4 writes this line with an em-dash, which
    // CP437 does not have — see `lint_prose` below.
    painter.span(
        Pos::new(2, 1),
        &Span::new("O.R.B.S. v0.9.3  --  cold start").with_style(Style::BRIGHT),
    );

    let systems = [
        ("scrying lens", "ok", Role::Success, None),
        ("ley-line uplink", "ok", Role::Success, None),
        ("grimoire index", "2841", Role::Normal, None),
        ("alembic", "ok", Role::Success, None),
        (
            "battlements",
            "DEGRADED",
            Role::Danger,
            Some("east_wall integrity 34%"),
        ),
        ("menagerie", "not found", Role::Normal, None),
        ("archive", "ok", Role::Success, None),
    ];

    let mut row = 3;
    for (label, status, role, detail) in systems {
        status_row(&mut painter, Pos::new(2, row), label, status, role);
        row += 1;
        if let Some(detail) = detail {
            painter.span(
                Pos::new(4, row),
                &Span::new(detail).with_style(Style::DANGER),
            );
            row += 1;
        }
    }

    // Listing bound scripts is the ambient defence against forgetting what runs
    // unattended (§8.1), not decoration.
    row += 1;
    painter.span(
        Pos::new(2, row),
        &Span::new("bound:").with_kind(UtteranceKind::Heading),
    );
    status_row(
        &mut painter,
        Pos::new(4, row + 1),
        "night_watch    dusk",
        "ok",
        Role::Success,
    );
    status_row(
        &mut painter,
        Pos::new(4, row + 2),
        "purge_cycle    hourly",
        "DRIFTED",
        Role::Danger,
    );

    painter.span(
        Pos::new(2, row + 4),
        &Span::new("3 warnings. the orb warms to your touch.").with_style(Style::DIM),
    );

    let input = layout.input();
    frame.painter(input).span(
        Pos::new(input.col, input.row),
        &Span::new("orbs:~$ ").with_kind(UtteranceKind::Input),
    );
    frame.set_cursor(Some(Pos::new(input.col + 8, input.row)));
    frame
}

/// Four domain panes under attack, three more minimised to the sidebar.
fn siege(grid: GridSize, mode: DisplayMode) -> Frame {
    let layout = ScreenLayout::compute(&ScreenRequest {
        grid,
        main_panes: 4,
        sidebar_panes: 3,
        mode,
    });

    let mut frame = Frame::new(grid);
    let panes: [(&str, fn(&mut Painter<'_>, Rect)); 4] = [
        ("alembic", alembic),
        ("battlements", battlements),
        ("archive", archive),
        ("scrying", scrying),
    ];

    for (rect, (title, content)) in layout.main().iter().zip(panes) {
        let mut painter = frame.painter(*rect);
        painter.border(*rect, Some(title), Style::DIM);

        // The content area gets its OWN painter. Handing the pane's painter a
        // smaller rectangle would only be a suggestion — overflowing content
        // would eat the bottom border, which is exactly what happened the first
        // time this example ran. Clipping protects the boundary you establish,
        // not the one you meant.
        let inner = rect.inset(1);
        content(&mut painter.sub(inner), inner);
    }

    // Sidebar panes are awareness only — one line, not commandable (§9).
    for (rect, name) in layout
        .sidebar()
        .iter()
        .zip(["forge", "menagerie", "sanctum"])
    {
        let mut painter = frame.painter(*rect);
        painter.span(
            rect.origin(),
            &Span::new(name)
                .with_style(Style::DIM)
                .with_kind(UtteranceKind::Heading),
        );
        painter.glyphs(Pos::new(rect.col + 12, rect.row), "idle", Style::DIM);
    }

    let input = layout.input();
    frame.painter(input).span(
        input.origin(),
        &Span::new("orbs:~$ ward north").with_kind(UtteranceKind::Input),
    );
    frame.set_cursor(Some(Pos::new(input.col + 18, input.row)));
    frame
}

/// A duration-action in flight, with its meter.
fn alembic(painter: &mut Painter<'_>, area: Rect) {
    painter.span(area.origin(), &Span::new("decoct haste"));
    painter.progress(
        Rect::new(area.col, area.row + 1, area.cols.min(20), 1),
        252,
        372,
        Style::COST,
        "haste, 68 percent, 2 minutes remaining",
    );
    painter.span(
        Pos::new(area.col, area.row + 2),
        &Span::new("reagents: 3").with_style(Style::DIM),
    );
}

/// A breach. The accent triad's whole reason to exist.
fn battlements(painter: &mut Painter<'_>, area: Rect) {
    painter.span(
        area.origin(),
        &Span::new("east_wall").with_style(Style::NORMAL),
    );
    painter.progress(
        Rect::new(area.col, area.row + 1, area.cols.min(20), 1),
        34,
        100,
        Style::DANGER,
        "east wall integrity 34 percent",
    );
    painter.span(
        Pos::new(area.col, area.row + 2),
        &Span::new("WARD FAILED").with_style(Style::DANGER.with_intensity(Intensity::Bright)),
    );
}

/// A poisoned log. The genuine line and the forged one differ by one space —
/// the structural tell of §8.1, drawn here as `Presentation::Tampered`.
fn archive(painter: &mut Painter<'_>, area: Rect) {
    painter.span(
        area.origin(),
        &Span::new("purge_cycle log").with_kind(UtteranceKind::Heading),
    );
    painter.span(
        Pos::new(area.col, area.row + 1),
        &Span::new("03:14 purge ok").with_kind(UtteranceKind::TableRow),
    );
    painter.span(
        Pos::new(area.col, area.row + 2),
        &Span::new("03:14  purge ok")
            .with_style(Style::NORMAL.with_presentation(Presentation::Tampered))
            .with_kind(UtteranceKind::TableRow),
    );
}

/// The eldritch register. The drawn form is damaged; the spoken form is authored
/// separately and carries the same dread through diction (§3).
fn scrying(painter: &mut Painter<'_>, area: Rect) {
    painter.paragraph(
        area,
        &Span::new("t h e   d o o r   i s   o p e n")
            .with_style(Style::NORMAL.with_presentation(Presentation::Eldritch))
            .with_spoken("The door is open. It was not opened."),
    );
}

/// `label ....... [ status ]` — one utterance, three visual styles.
fn status_row(painter: &mut Painter<'_>, at: Pos, label: &str, status: &str, role: Role) {
    const BRACKET_COL: u16 = 30;

    // The row speaks once, as a flattened `label: value` (§14).
    painter.span(
        at,
        &Span::new(label)
            .with_kind(UtteranceKind::TableRow)
            .with_spoken(&format!("{label}: {status}")),
    );

    // The leader and the bracket are already accounted for by that utterance.
    let label_end = at.col + u16::try_from(label.chars().count()).unwrap_or(0) + 1;
    painter.fill(
        Rect::new(label_end, at.row, BRACKET_COL.saturating_sub(label_end), 1),
        '.',
        Style::DIM,
    );
    painter.glyphs(
        Pos::new(BRACKET_COL + 1, at.row),
        &format!("[ {status} ]"),
        Style::NORMAL.with_role(role),
    );
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// What a content pipeline will do to every prose data file at load time.
///
/// The first line is DESIGN.md §4's boot header verbatim. It does not survive
/// contact with the font, which is worth knowing before ~88k words are written
/// in an editor that produces smart quotes.
fn lint_prose() {
    println!("\nProse lint — CP437 repertoire (DESIGN.md §4, §13)\n");
    let samples = [
        "O.R.B.S. v0.9.3  —  cold start",
        "the wizard's alembic — untouched",
        "O.R.B.S. v0.9.3  --  cold start",
        "east_wall integrity 34% [ DEGRADED ]",
    ];
    for line in samples {
        match orbs_render::cp437::first_unrenderable(line) {
            Some((offset, glyph)) => {
                println!(
                    "  REJECT  byte {offset:>3}: {glyph:?} (U+{:04X})  {line}",
                    u32::from(glyph)
                );
            }
            None => println!("  ok                            {line}"),
        }
    }
}

fn tier_table() {
    println!("\nFidelity — window pixels to grid cells (DESIGN.md §9)\n");
    for window in [(1920u32, 1080u32), (2560, 1440), (1280, 720)] {
        let Some(one) = Fidelity::tier_one(window) else {
            println!("  {window:?}  below the 80×22 floor");
            continue;
        };
        let two = one.deep();
        let deep = two.map_or_else(
            || "unavailable".to_owned(),
            |tier| {
                let grid = tier.grid(window);
                format!("{}× → {}×{}", tier.scale(), grid.cols, grid.rows)
            },
        );
        let grid = one.grid(window);
        println!(
            "  {:>5}×{:<5}  tier 1: {}× → {}×{:<3}  tier 2: {}",
            window.0,
            window.1,
            one.scale(),
            grid.cols,
            grid.rows,
            deep
        );
    }
}

/// Print a frame exactly as a rasteriser would walk it.
fn show(title: &str, frame: &Frame) {
    let size = frame.size();
    let rule = "─".repeat(usize::from(size.cols));
    println!("\n\n{title}\n");
    println!("  ╭{rule}╮");
    for row in frame.rows() {
        let line: String = row.iter().map(|cell| cell.glyph).collect();
        println!("  │{line}│");
    }
    println!("  ╰{rule}╯");
    if let Some(cursor) = frame.cursor() {
        println!("  cursor at column {}, row {}", cursor.col, cursor.row);
    }
}

/// Print the same frame as a screen reader receives it.
fn speak(frame: &Frame) {
    println!("\n  Linearised — the same frame with no pixels at all:\n");
    for utterance in frame.speech().utterances() {
        let kind = format!("{:?}", utterance.kind);
        let role = match utterance.role {
            Role::Normal => String::new(),
            other => format!("({other:?}) "),
        };
        println!("    {kind:<10} {role}{}", utterance.text);
    }
}

/// §9's parity rule, checked rather than asserted in prose.
fn parity(deep: &Frame, wide: &Frame) {
    let of = |frame: &Frame| -> Vec<String> {
        frame
            .speech()
            .utterances()
            .map(|utterance| {
                format!(
                    "{:?}|{:?}|{}",
                    utterance.kind, utterance.role, utterance.text
                )
            })
            .collect()
    };

    println!("\n\nParity — Deep focus vs Wide focus (DESIGN.md §9)\n");
    println!(
        "  Deep 120×33: {} cells, {} utterances",
        deep.size().area(),
        deep.speech().len()
    );
    println!(
        "  Wide  80×22: {} cells, {} utterances",
        wide.size().area(),
        wide.speech().len()
    );

    assert_eq!(
        of(deep),
        of(wide),
        "the two display modes disagree about what the screen says"
    );
    println!("\n  Identical linear streams. Different rendering, same game.\n");
}
