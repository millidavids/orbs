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
    Burn, Depiction, DisplayMode, Fidelity, FieldName, Frame, GridSize, Grind, Intensity, Outcome,
    Painter, Pos, Presentation, RecordKind, RecordView, Records, Rect, Role, ScreenLayout,
    ScreenRequest, Sift, Span, Steep, Style, UtteranceKind,
};

/// The wizard's name is world state (`orbs_sim::Wizard`), which this crate does
/// not know about — so a stand-in stands in.
const DEMO_PROMPT: &str = "orbs $ ";

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

    let records = brewing_log();
    let views = records_screen(GridSize::new(80, 22), &records);
    show("Records — one stream, three views (§7)", &views);
    speak(&views);
    exemption(&records);

    let session = session();
    let prompt = prompt_screen(GridSize::new(80, 22), &session);
    show("The prompt — every outcome §6 can produce", &prompt);
    speak(&prompt);

    let editor = editor_screen(GridSize::new(80, 22));
    show("The spell editor at the 80×22 floor (§8)", &editor);
    speak(&editor);

    burning();
    grinding();
    steeping();
    worst_case();
}

/// The balneum mariae: a vessel of liquid, rolling as it digests.
///
/// **The only place the roil can be looked at as text.** Every other instrument
/// carries some of its motion in glyphs, so `ORBS_DUMP` shows it moving; this one
/// carries *all* of its motion in colour by design — the glyph is `█` at every
/// fill and every phase — so a dump of it is a solid bar that proves nothing.
/// Printed as ramp steps, the roil is visible.
fn steeping() {
    const ROWS: u16 = 12;
    // Four ticks, which is exactly one beat of the bath's own tempo — long
    // enough that every cell has had the chance to turn over once.
    let tick = 1.0 / orbs_render::FLIP_HZ;

    println!("\n── The balneum mariae — 5/8 digested ──\n");
    println!("    █ liquid, every cell, always.  a still  b stirred  c rolling");
    println!("    Bubbles are lighter *colour*, never a second glyph — a mark in the");
    println!("    water would be a mark of something, and a vessel holds one substance.");
    println!("    So the level survives with the colour thrown away: solid against");
    println!("    blank, the strongest join there is.\n");
    println!("    Each column is one rise. Read a letter and find it one row higher");
    println!("    in the column to its right — that is a bubble on its way up.\n");

    let mut columns = Vec::new();
    for step in 0u16..6 {
        // One *rise* per column, so the translation is visible across the page.
        let phase = f32::from(step * orbs_render::RISE_EVERY) * tick;
        let mut frame = Frame::new(GridSize::new(2, ROWS));
        let area = Rect::new(0, 0, 2, ROWS);
        frame.painter(area).bath_meter_upward(
            area,
            5,
            8,
            Steep {
                phase,
                motion: orbs_render::Motion::Bubbling,
                ..Steep::default()
            },
        );
        columns.push(frame);
    }

    for row in 0..ROWS {
        let mut line = String::from("    ");
        for frame in &columns {
            for col in 0..2 {
                let cell = frame.cell(Pos::new(col, row));
                line.push(cell.map_or(' ', |cell| depiction_mark(cell.style.depicted())));
            }
            line.push_str("  ");
        }
        println!("{line}");
    }
    println!("\n    0   1   2   3   4   5   ← rises, three shared-clock ticks each");
    println!("    A bubble climbs a cell every three ticks — one flash a second, so");
    println!("    unlike the fire this needs no §19 exemption. It is also the slowest");
    println!("    tempo on the panel, which is the signature a gentle heat should have.");

    // ...and the three states the sim reports no meter for, which is where a
    // vessel picture earns its keep: all three would otherwise draw nothing.
    println!("\n    charged        fouled         ready");
    let resting = [(0u32, false), (0, true), (8, false)];
    let mut frames = Vec::new();
    for (done, spent) in resting {
        let mut frame = Frame::new(GridSize::new(2, ROWS));
        let area = Rect::new(0, 0, 2, ROWS);
        frame.painter(area).bath_meter_upward(
            area,
            done,
            8,
            Steep {
                phase: 3.0,
                spent,
                ..Steep::default()
            },
        );
        frames.push(frame);
    }
    for row in 0..ROWS {
        let mut line = String::from("    ");
        for frame in &frames {
            for col in 0..2 {
                let cell = frame.cell(Pos::new(col, row));
                line.push(cell.map_or(' ', |cell| cell.glyph));
            }
            line.push_str("             ");
        }
        println!("{}", line.trim_end());
    }
    println!("\n    A charged vessel is a shallow layer, not an empty one — a bar");
    println!("    reading *how far along* would draw it blank, which is exactly what");
    println!("    an empty instrument looks like. Sediment (▓, still, two cells deep)");
    println!("    lies too low to be mistaken for a level.");
}

/// The mortar and pestle: a block broken down, across a fall and a lifecycle.
///
/// Six consecutive frames of the shared clock — one world tick — so the debris
/// falls three cells, which is the whole depth of the working gap.
fn grinding() {
    const ROWS: u16 = 16;
    let tick = 1.0 / orbs_render::FLIP_HZ;

    println!("\n── The mortar and pestle — 3/8 broken down ──\n");
    println!("    █ whole   ▓ broken and settled   ▒░ in pieces, in the air");
    println!("    The block is eaten from below: the gap between it and the bed is");
    println!("    fixed, so the bar stays full of material and what changes is state.\n");

    let mut columns = Vec::new();
    for step in 0u16..6 {
        let phase = f32::from(step) * tick;
        let mut frame = Frame::new(GridSize::new(2, ROWS));
        let area = Rect::new(0, 0, 2, ROWS);
        frame.painter(area).grind_meter_upward(
            area,
            3,
            8,
            Grind {
                phase,
                working: true,
                ..Grind::default()
            },
        );
        columns.push((orbs_render::fallen_cells(phase), frame));
    }

    for row in 0..ROWS {
        let mut line = String::from("    ");
        for (_, frame) in &columns {
            for col in 0..2 {
                line.push(
                    frame
                        .cell(Pos::new(col, row))
                        .map_or(' ', |cell| cell.glyph),
                );
            }
            line.push_str("     ");
        }
        println!("{line}");
    }
    let heights: Vec<String> = columns
        .iter()
        .map(|(fallen, _)| format!("{:<7}", format!("v{fallen}")))
        .collect();
    println!("    {}", heights.join(""));

    // The whole lifecycle. **Four of these five report no meter at all** — the
    // sim has a quantity only while a run is going — so every one of them was a
    // blank row before this picture existed.
    //
    // The pour is sampled three times because it is an *edge*: the frame a
    // reagent enters an empty bowl, gone a tick later.
    println!("\n── the same bowl, pouring through fouled ──\n");
    for (label, done, working, spent, load) in [
        ("pouring", 0u32, false, false, 1.0),
        ("pouring", 0, false, false, 0.5),
        ("charged", 0, false, false, 0.0),
        ("working", 3, true, false, 0.0),
        ("working", 6, true, false, 0.0),
        ("ready", 8, false, false, 0.0),
        ("fouled", 0, false, true, 0.0),
    ] {
        let mut frame = Frame::new(GridSize::new(2, ROWS));
        let area = Rect::new(0, 0, 2, ROWS);
        frame.painter(area).grind_meter_upward(
            area,
            done,
            8,
            Grind {
                phase: tick * 3.0,
                working,
                spent,
                load,
                ..Grind::default()
            },
        );
        let column: String = (0..ROWS)
            .rev()
            .map(|row| frame.cell(Pos::new(0, row)).map_or(' ', |cell| cell.glyph))
            .collect();
        println!("    {label:<9} {done}/8  bottom |{column}| top");
    }

    // Horizontal, which is what a tall narrow pane gets. The picture needs no
    // adjusting for it: "down" is simply "toward the collected end", and the
    // shade ramp carries the same four states on either axis.
    println!("\n── ...and horizontal, where down means leftward ──\n");
    for step in 0u16..6 {
        let phase = f32::from(step) * tick;
        let mut frame = Frame::new(GridSize::new(40, 1));
        let area = Rect::new(0, 0, 40, 1);
        frame.painter(area).grind_meter(
            area,
            3,
            8,
            Grind {
                phase,
                working: true,
                ..Grind::default()
            },
        );
        let row: String = (0..40)
            .map(|col| frame.cell(Pos::new(col, 0)).map_or(' ', |cell| cell.glyph))
            .collect();
        println!("    v{}  {row}", orbs_render::fallen_cells(phase));
    }
}

/// The athanor's meter, burning, at several phases.
///
/// **What this can and cannot check.** `orbs-render` has an empty
/// `[dependencies]` and this example prints text, so it can show neither the
/// flame ramp's colours nor the assembled instrument panel — `orbs::shell::panel`
/// is private to the Bevy crate. What it does show is the thing that is
/// genuinely at risk and that no unit test displays: whether the glyph pattern
/// reads as *fire* rather than as noise, and whether the `█`/`░` join stays
/// findable while everything around it moves.
///
/// The colours need eyes on a window. See CLAUDE.md's See-it lines.
fn burning() {
    // A drained athanor: enough plume to see, enough fire to read.
    const ROWS: u16 = 16;
    const REMAINING: u32 = 5;
    const CAPACITY: u32 = 16;

    println!("\n── The athanor, burning — {REMAINING}/{CAPACITY} fuel, four phases ──\n");

    // Vertical, as the side panel draws it (§10.1), two cells wide.
    //
    // **One tick apart**, so consecutive columns are consecutive frames of the
    // animation rather than an arbitrary sample of it — that is what makes the
    // plume's drift and the sparks' rise visible as motion in a still dump.
    let tick = 1.0 / orbs_render::FLIP_HZ;
    let burn = |phase: f32| Burn {
        phase,
        flare: 0.0,
        lit: true,
    };
    let mut columns = Vec::new();
    for step in 0u16..4 {
        let phase = f32::from(step) * tick;
        let mut frame = Frame::new(GridSize::new(2, ROWS));
        let area = Rect::new(0, 0, 2, ROWS);
        frame
            .painter(area)
            .fire_meter_upward(area, REMAINING, CAPACITY, burn(phase));
        columns.push((phase, frame));
    }

    let glyph_at = |frame: &Frame, col: u16, row: u16| {
        frame
            .cell(Pos::new(col, row))
            .map_or(' ', |cell| cell.glyph)
    };

    for row in 0..ROWS {
        let mut line = String::from("    ");
        for (_, frame) in &columns {
            for col in 0..2 {
                line.push(glyph_at(frame, col, row));
            }
            line.push_str("     ");
        }
        println!("{line}");
    }
    let labels: Vec<String> = columns
        .iter()
        .map(|(phase, _)| format!("{phase:<7.2}"))
        .collect();
    println!("    {}", labels.join(""));

    // Horizontal, as the top panel draws it when the pane is taller than wide.
    println!();
    for step in [0u16, 1] {
        let phase = f32::from(step) * tick;
        let mut frame = Frame::new(GridSize::new(48, 1));
        let area = Rect::new(0, 0, 48, 1);
        frame
            .painter(area)
            .fire_meter(area, REMAINING, CAPACITY, burn(phase));
        let row: String = (0..48).map(|col| glyph_at(&frame, col, 0)).collect();
        println!("    {phase:.2}  {row}");
    }

    // The states the same bar can be in — a running game shows one at a time,
    // and the flare is over in a second.
    //
    // **Printed twice, as glyphs and as heat, and the heat is the point.** The
    // flare is *entirely* a colour event: a front of ignition climbing from the
    // base, with the glyphs deliberately identical throughout so the meter never
    // misreports fuel. A dump showing only glyphs would print every frame of it
    // the same and prove nothing, which is worse than not showing it at all.
    // The digits are the ramp step, `1` coolest to `4` hottest — watch the `1`s
    // give way from the left, which is the bottom of the bar.
    println!("\n── kindling: cold, the flare climbing, guttering ──\n");
    println!("    glyphs: █ ▓ fire   ░ ▒ smoke   ∙ ° · sparks");
    println!("    heat:   1..4 ramp step, * spark, . : smoke, space nothing\n");

    // Four ticks of cold, because the wisp is sparse by design — at any one
    // instant the bottom cell is clear 60% of the time, and a single sample of
    // it prints an empty column and looks like a bug.
    for step in 0u16..4 {
        let phase = f32::from(step) * tick;
        show_hearth(
            &format!("cold {step}"),
            0,
            1,
            Burn {
                phase,
                flare: 0.0,
                lit: false,
            },
            ROWS,
        );
    }

    // **The flare, sampled across its life rather than at one instant.** It is a
    // front climbing from the base, so a single frame of it says nothing about
    // whether it climbs — the heat column is where you watch the `1`s give way.
    for fifth in 0..=5u16 {
        let flare = 1.0 - f32::from(fifth) / 5.0;
        show_hearth(
            &format!("flare {flare:.1}"),
            CAPACITY,
            CAPACITY,
            Burn {
                phase: tick,
                flare,
                lit: true,
            },
            ROWS,
        );
    }

    // Fuel the bar rounds away: one tick left of six hundred, which divides to
    // zero cells. The ember is what says "still lit" rather than "out".
    show_hearth("guttering", 1, 600, burn(tick), ROWS);
}

/// One hearth, printed as glyphs and again as heat. See [`burning`].
fn show_hearth(label: &str, done: u32, total: u32, burn: Burn, rows: u16) {
    let mut frame = Frame::new(GridSize::new(2, rows));
    let area = Rect::new(0, 0, 2, rows);
    frame
        .painter(area)
        .fire_meter_upward(area, done, total, burn);

    let read = |as_heat: bool| -> String {
        (0..rows)
            .rev()
            .map(|row| {
                frame.cell(Pos::new(0, row)).map_or(' ', |cell| {
                    if as_heat {
                        depiction_mark(cell.style.depicted())
                    } else {
                        cell.glyph
                    }
                })
            })
            .collect()
    };
    println!("    {label:<10} |{}|  heat |{}|", read(false), read(true));
}

/// A depiction as one character, so a text dump can show colour it cannot draw.
///
/// **This is the only See-it there is for the bath's roil.** All of that
/// instrument's motion is in its colour — the glyph is `█` at every fill and
/// every phase, by design — so `ORBS_DUMP` shows a solid bar and proves nothing.
/// Printed as ramp steps, the roil is text.
const fn depiction_mark(depiction: Depiction) -> char {
    match depiction {
        Depiction::SparkEmber
        | Depiction::SparkBody
        | Depiction::SparkBlaze
        | Depiction::SparkCore => '*',
        Depiction::SmokeThin => '.',
        Depiction::SmokeThick => ':',
        Depiction::FlameEmber => '1',
        Depiction::FlameBody => '2',
        Depiction::FlameBlaze => '3',
        Depiction::FlameCore => '4',
        Depiction::LiquidStill => 'a',
        Depiction::LiquidStirred => 'b',
        Depiction::LiquidRolling => 'c',
        Depiction::Sediment => ',',
        Depiction::None => ' ',
    }
}

/// The spell editor, at the size where it is tightest (§8).
///
/// **Hand-built from literals**, like `boot_report` and `siege` beside it: this
/// example lives in `orbs-render`, whose `[dependencies]` is deliberately empty,
/// so it cannot reach `orbs`'s editor or `orbs-sim`'s spells. What it checks is
/// the thing that is genuinely at risk — that a gutter, a border, a filename, a
/// status line and a caret position all fit in 80 columns with room left for a
/// spell, and that every one of them **speaks**.
///
/// The real painter is `orbs::shell::sheet`. If this layout stops fitting, that
/// one has the same problem.
fn editor_screen(grid: GridSize) -> Frame {
    const GUTTER: u16 = 5;
    let mut frame = Frame::new(grid);
    let area = Rect::new(0, 0, grid.cols, grid.rows);
    let mut painter = frame.painter(area);
    painter.border(area, Some("night_watch.spell *"), Style::DIM);

    let lines = [
        "attend laboratory#2",
        "kindle charcoal",
        "grind sage",
        "siphon mortar_and_pestle",
        "empty mortar_and_pestle",
    ];
    for (index, line) in lines.iter().enumerate() {
        let y = 1 + u16::try_from(index).unwrap_or(0);
        painter.span(
            Pos::new(1, y),
            &Span::new(&format!("{:>4} ", index + 1)).with_style(Style::DIM),
        );
        painter.span(Pos::new(1 + GUTTER, y), &Span::new(line));
    }

    // The row that must never be blank: §6 forbids a dead end, and this is the
    // game's first modal surface — a player who does not know the words has
    // nowhere else to find them, so in command state this row *is* the whole
    // interface.
    let status = grid.rows.saturating_sub(2);
    painter.span(
        Pos::new(1, status),
        &Span::new("edit  save  quit  discard").with_style(Style::DIM),
    );
    painter.span(
        Pos::new(grid.cols.saturating_sub(5), status),
        &Span::new("3:11").with_style(Style::DIM),
    );

    frame.set_cursor(Some(Pos::new(1 + GUTTER + 10, 3)));
    frame
}

// ---------------------------------------------------------------------------
// §15's worst-case legibility test
// ---------------------------------------------------------------------------

/// The hardest screen the game can produce, and the window that produces it.
///
/// DESIGN.md §15 asks for *"tier 2 at minimum supported window, four panes,
/// siege in progress, peak-threat CRT, eldritch active, tester must spot a
/// single-character sabotage tell"* — and, in the same breath, for the item to
/// **establish the minimum window at which tier 2 is offered**. That number is
/// derived here rather than written down: §19's standing lesson from four failed
/// attempts at the CRT overscan is *compute the constant, do not reason about
/// it*.
///
/// Two of the six conditions are not in a [`Frame`] and cannot be. Peak-threat
/// CRT and the phosphor are frontend enrichment — rule 2 — so they are read on
/// the running game at this window with `F3`, and what this screen establishes
/// is that everything *informational* survives at the smallest glyph the game
/// ever draws. The final judgement is a human one; this prepares it and cannot
/// make it.
fn worst_case() {
    let window = minimum_window_for_tier_two();
    let one = Fidelity::tier_one(window).expect("the floor fits by construction");
    let two = one.deep().expect("tier two by construction");
    let grid = two.grid(window);
    let (cell_width, cell_height) = two.cell_pixels();

    println!("\nWorst-case legibility — DESIGN.md §15\n");
    println!(
        "  minimum window offering tier 2   {}x{}",
        window.0, window.1
    );
    println!(
        "    tier 1  {}x cell -> {:?}",
        one.scale(),
        one.grid(window)
    );
    println!(
        "    tier 2  {two_scale}x cell -> {grid:?}",
        two_scale = two.scale()
    );
    println!("  glyph at tier 2                  {cell_width}x{cell_height} physical pixels");
    println!("  a one-character tell is          {cell_width} pixels wide\n");

    let frame = siege(grid, DisplayMode::Deep);
    show("Worst case — four panes, tier 2, siege, eldritch", &frame);

    // §8.1's structural tell, at the smallest glyph the game draws. The forged
    // line differs from the genuine one by a single space, which is the whole
    // point: if it does not survive to the Frame here it survives nowhere.
    let drawn = frame.to_text();
    let genuine = drawn.contains("03:14 purge ok");
    let forged = drawn.contains("03:14  purge ok");
    println!(
        "  the tell: genuine line {}, forged line {} — {}",
        if genuine { "drawn" } else { "MISSING" },
        if forged { "drawn" } else { "MISSING" },
        if genuine && forged {
            "one space apart, spot it"
        } else {
            "THE TEST CANNOT BE RUN"
        },
    );

    // §14 and rule 2: whatever the eye has to work for, the linear stream says
    // plainly. Peak-threat CRT and eldritch substitution both act on the drawn
    // form only, so a reader loses nothing at this window that they had at any
    // other — which is the property that makes the visual test safe to fail.
    let spoken = frame
        .speech()
        .utterances()
        .find(|utterance| utterance.text.contains("door"))
        .map(|utterance| utterance.text);
    println!(
        "  eldritch pane speaks:            {}",
        spoken.unwrap_or("MISSING"),
    );
    println!("\n  Remaining, and human: the CRT at peak threat over this grid.");
    println!(
        "  Size the window to {}x{}, F4 into Deep focus, F3 to peak threat, F7 for eldritch.",
        window.0, window.1,
    );
}

/// The smallest window at which §9's tier 2 exists at all.
///
/// Tier 2 is [`Fidelity::deep`] of tier 1, and `deep` is `None` at scale 1 —
/// so the question is really "when does tier 1 stop being the finest scale",
/// and the answer is a search rather than a constant anyone should retype.
///
/// Searched per axis. A grid's columns depend only on the window's width and its
/// rows only on its height, so the smallest qualifying window is the pair of
/// per-axis minima; searching both at once would be a slower way to the same
/// number.
fn minimum_window_for_tier_two() -> (u32, u32) {
    /// Past any window a 2026 desktop will present, and small enough to search
    /// exhaustively in microseconds.
    const LIMIT: u32 = 8192;

    let offers_tier_two = |window: (u32, u32)| {
        Fidelity::tier_one(window)
            .and_then(Fidelity::deep)
            .is_some()
    };

    let width = (1..=LIMIT)
        .find(|width| offers_tier_two((*width, LIMIT)))
        .expect("some width offers tier 2");
    let height = (1..=LIMIT)
        .find(|height| offers_tier_two((LIMIT, *height)))
        .expect("some height offers tier 2");
    (width, height)
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
        ("laboratory", "ok", Role::Success, None),
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
        input_rows: 1,
    });

    let mut frame = Frame::new(grid);
    let panes: [(&str, fn(&mut Painter<'_>, Rect)); 4] = [
        ("laboratory", laboratory),
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

/// A session at the prompt, as records.
///
/// Built by hand rather than by driving the parser, because this crate does not
/// know the parser exists — which is the point. The view has to make these six
/// outcomes distinguishable from the record alone.
fn session() -> Records {
    let mut records = Records::new();
    let typed = |records: &mut Records, line: &str| {
        records
            .push(RecordKind::Input)
            .text(FieldName::Message, line)
            .finish();
    };
    let answer = |records: &mut Records, outcome: Outcome, message: &str| {
        records
            .push(RecordKind::Echo)
            .outcome(outcome)
            .text(FieldName::Message, message)
            .finish();
    };

    typed(&mut records, "look around");
    answer(&mut records, Outcome::Resolved, "survey");

    typed(&mut records, "meditate");
    records
        .push(RecordKind::Echo)
        .outcome(Outcome::Incomplete)
        .text(FieldName::Message, "meditate")
        .text(FieldName::Kind, "count")
        .finish();

    typed(&mut records, "brew clarity");
    answer(&mut records, Outcome::Candidate, "decoct clarity");
    answer(&mut records, Outcome::Candidate, "siphon clarity");

    typed(&mut records, "xyzzy");
    answer(&mut records, Outcome::Unresolved, "xyzzy");
    answer(&mut records, Outcome::Suggestion, "survey");
    answer(&mut records, Outcome::Suggestion, "scribe");

    typed(&mut records, "ward the north wall");
    answer(&mut records, Outcome::Forced, "ward north_gate");

    records
        .push(RecordKind::Completion)
        .text(FieldName::Name, "survey")
        .tick(FieldName::Tick, 1247)
        .role(Role::Success)
        .finish();

    records
}

/// The command line: every outcome §6 can produce, on one screen.
fn prompt_screen(grid: GridSize, records: &Records) -> Frame {
    let mut frame = Frame::new(grid);
    let layout = ScreenLayout::compute(&ScreenRequest::single(grid));
    let pane = layout.main().first().copied().unwrap_or(Rect::EMPTY);

    let mut painter = frame.painter(pane);
    painter.border(pane, Some("O.R.B.S.  tick 1247  seed 0xc0ffee"), Style::DIM);
    RecordView::prompt(DEMO_PROMPT).draw(&mut painter, pane.inset(1), records.iter());

    // The input line, mid-typing, with the caret where the frontend puts it.
    let input = layout.input();
    let mut painter = frame.painter(input);
    let prompt = painter.glyphs(input.origin(), DEMO_PROMPT, Style::DIM);
    let typed = "sift march ";
    painter.span(
        Pos::new(input.col + prompt, input.row),
        &Span::new(typed).with_kind(UtteranceKind::Input),
    );
    frame.set_cursor(Some(Pos::new(
        input.col + prompt + u16::try_from(typed.len()).unwrap_or(0),
        input.row,
    )));
    frame
}

/// A few minutes of the laboratory's life, as records.
///
/// One stream holds the directory listing, the log, and the orb speaking.
/// DESIGN.md §3 forbids unlogged output, so this *is* the log — the same rows a
/// pane draws, a reader hears, and a pipe stage filters.
fn brewing_log() -> Records {
    let mut records = Records::new();

    for (name, state, quantity) in [
        ("sage", "ready", 12u64),
        ("nightshade", "spoiled", 3),
        ("moonwater", "ready", 40),
        ("ash-of-vigil", "brewing", 1),
    ] {
        let role = if state == "spoiled" {
            Role::Danger
        } else {
            Role::Normal
        };
        records
            .push(RecordKind::Entry)
            .text(FieldName::Name, name)
            .text(FieldName::State, state)
            .count(FieldName::Quantity, quantity)
            .role(role)
            .finish();
    }

    for (tick, source, message, role) in [
        (
            1247u64,
            "laboratory",
            "decoction of clarity begun",
            Role::Normal,
        ),
        (
            1249,
            "laboratory",
            "nightshade spoiled in vessel 2",
            Role::Danger,
        ),
        (1251, "lens", "ley-line draw steady at 4", Role::Normal),
        (1254, "laboratory", "clarity decanted", Role::Success),
    ] {
        records
            .push(RecordKind::LogLine)
            .tick(FieldName::Tick, tick)
            .text(FieldName::Source, source)
            .text(FieldName::Message, message)
            .role(role)
            // Asked for on every log line, and refused on every one: §3 keeps
            // the diagnostic surfaces trustworthy as renderings. See the note
            // printed under this screen.
            .presentation(Presentation::Eldritch)
            .finish();
    }

    records
        .push(RecordKind::Message)
        .text(
            FieldName::Message,
            "s o m e t h i n g   i s   c o u n t i n g",
        )
        .presentation(Presentation::Eldritch)
        .spoken("something is counting")
        .finish();

    records
}

/// The same stream, drawn three ways.
fn records_screen(grid: GridSize, records: &Records) -> Frame {
    let mut frame = Frame::new(grid);
    let layout = ScreenLayout::compute(&ScreenRequest::single(grid));
    let area = layout.main().first().copied().unwrap_or(Rect::EMPTY);
    let mut painter = frame.painter(area);

    let third = area.rows / 3;
    let panes = [
        (Rect::new(area.col, area.row, area.cols, third), "ls"),
        (
            Rect::new(area.col, area.row + third, area.cols, third),
            "peruse laboratory.log",
        ),
        (
            Rect::new(
                area.col,
                area.row + third * 2,
                area.cols,
                area.rows - third * 2,
            ),
            "peruse laboratory.log | sift spoil",
        ),
    ];
    let inner = |pane: Rect| {
        Rect::new(
            pane.col + 2,
            pane.row + 1,
            pane.cols.saturating_sub(4),
            pane.rows.saturating_sub(2),
        )
    };

    // Each pane's border is drawn immediately before its own content, never all
    // three up front. The linear stream is captured in paint order, so bordering
    // everything first would give a reader three headings and then fourteen
    // unattributed rows — the `sift` results indistinguishable from the listing
    // they were filtered out of. §14's parity is an ordering property, not just
    // a completeness one.
    let spoiled = Sift::new("spoil");
    for (index, (pane, title)) in panes.into_iter().enumerate() {
        painter.border(pane, Some(title), Style::DIM);
        let inner = inner(pane);
        match index {
            // A table: the view picks columns and alignment. It cannot invent a
            // value, and it cannot keep one out of the linear stream.
            0 => RecordView::table(&[FieldName::Name, FieldName::State, FieldName::Quantity]).draw(
                &mut painter,
                inner,
                records
                    .iter()
                    .filter(|record| record.kind() == RecordKind::Entry),
            ),
            // Lines: same stream, no columns.
            1 => RecordView::lines().draw(
                &mut painter,
                inner,
                records
                    .iter()
                    .filter(|record| record.kind() != RecordKind::Entry),
            ),
            // A pipe stage. Matching runs over field values, never over anything
            // either pane above put on the screen.
            _ => RecordView::lines().draw(&mut painter, inner, records.sift(&spoiled)),
        };
    }

    frame
}

/// Print §3's corruption exemption, as the model actually resolves it.
fn exemption(records: &Records) {
    println!("\n  §3 — what each record above asked the renderer for, and got:\n");
    for record in records.iter() {
        let allowed = record.presentation();
        let note = match record.kind() {
            RecordKind::LogLine => "asked eldritch; diagnostic surface: refused",
            RecordKind::Message => "asked eldritch; tonal register: granted",
            _ => "asked nothing",
        };
        // Formatted first: a derived `Debug` ignores width specifiers, so
        // `{:<10?}` would silently print unpadded.
        let (kind, allowed) = (format!("{:?}", record.kind()), format!("{allowed:?}"));
        println!("    {kind:<9} -> {allowed:<9} {note}");
    }
    println!(
        "\n  A sabotage tell would survive that refusal. §8.1's structural signature\n  \
         lives on exactly the surfaces a player inspects, so suppressing it there\n  \
         would delete the signal; only the tonal register is exempt.\n"
    );
}

/// A duration-action in flight, with its meter.
fn laboratory(painter: &mut Painter<'_>, area: Rect) {
    // §10.1's instrument panel, in miniature: a standing meter per instrument,
    // drawn with `meter` rather than `progress` so five of them do not push five
    // utterances a frame. The panel owes one summary line instead — the last
    // call here is that debt paid, and §14 is why it exists.
    const PANEL: [(&str, &str, Option<(u32, u32)>); 5] = [
        ("mortar_and_pestle", "ready", None),
        ("balneum_mariae", "working", Some((7, 12))),
        ("flask_and_rod", "empty", None),
        ("alembic", "empty", None),
        // The one bar that drains: fuel remaining, not ticks elapsed.
        ("athanor", "burning", Some((22, 40))),
    ];

    // `glyphs`, not `span`: every row here is **silent**. Drawn with `span` the
    // rows spoke, and a Wide strip too short for the athanor then said one thing
    // less than Deep did — which `parity` catches and §9 forbids outright, since
    // a strip that shows less makes the display mode a difficulty choice.
    //
    // The panel's whole speech is the one summary below, which does not depend
    // on how many rows happened to fit.
    for (row, (name, state, meter)) in PANEL.into_iter().enumerate() {
        let row = area.row + u16::try_from(row).unwrap_or(0);
        if row >= area.bottom() {
            break;
        }
        painter.glyphs(Pos::new(area.col, row), name, Style::DIM);
        let at = area.col + 18;
        painter.glyphs(Pos::new(at, row), state, Style::DIM);
        if let Some((done, total)) = meter {
            let bar = Rect::new(at + 9, row, area.cols.saturating_sub(27), 1);
            painter.meter(bar, done, total, Style::COST);
        }
    }

    painter.announce(
        UtteranceKind::Progress,
        Role::Normal,
        "laboratory: mortar_and_pestle ready, balneum_mariae working, athanor burning",
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
        "the wizard's laboratory — untouched",
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
