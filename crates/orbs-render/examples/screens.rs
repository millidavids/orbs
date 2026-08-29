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
    Burn, Depiction, DisplayMode, FieldName, Frame, GRID, GridSize, Grind, Intensity, Outcome,
    PICTURE, Painter, Pos, Presentation, RecordKind, RecordView, Records, Rect, Role, ScreenLayout,
    ScreenRequest, Sift, Span, Square, Stacks, Steep, Style, UtteranceKind,
};

/// The wizard's name is world state (`orbs_sim::Wizard`), which this crate does
/// not know about — so a stand-in stands in.
const DEMO_PROMPT: &str = "orbs $ ";

fn main() {
    tier_table();
    lint_prose();

    let boot = boot_report(GridSize::new(80, 22));
    show(
        "Boot report — 80×22, the authoring floor (DESIGN.md §4)",
        &boot,
    );
    speak(&boot);

    // **The grid the game draws**, both times. These used to be two different
    // grids because the focus mode changed the fidelity tier and so the cell
    // count; §19 fixed the grid, so the pair is now what it always claimed to
    // be — the same screen divided two ways.
    let deep = siege(GRID, DisplayMode::Deep);
    show("Siege — Deep focus, the 120×45 grid", &deep);

    let wide = siege(GRID, DisplayMode::Wide);
    show("Siege — Wide focus, the same 120×45 grid", &wide);
    speak(&wide);

    parity(&deep, &wide);

    let sheet = ward(GRID);
    show("Ward — the lens's sheet, part broken (§10)", &sheet);
    speak(&sheet);

    let course = pylon(GRID);
    show("Pylon — a course of wards, part hauled (§10)", &course);
    speak(&course);

    let figure = chant(GRID);
    show("Figure — a chant, two ticks from the rule (§10)", &figure);
    speak(&figure);

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

    // **48×18 — narrower *and* shorter than this surface will now draw at.** A
    // tiled session pane is 60 columns since the grid was fixed at 120×45, and
    // the height is `loom::MIN_ROWS`, below which the real painter refuses
    // rather than drawing something misleading. Kept at 48 deliberately: it was
    // the real width and is now a margin, and a surface authored against the
    // tighter number keeps working when the grid is next revisited.
    let weave = weave_screen(GridSize::new(48, 18));
    show(
        "The weave screen at the width a tiled pane gives it (§11.5)",
        &weave,
    );
    speak(&weave);

    // 35×35 — the block the archive's map takes, at its natural size. Three
    // states, because the fog is the whole mechanic and one of them is not
    // enough to see it. The grid is 33 squares across (`2 × 16 + 1`), and the
    // snake through it is 511 squares long.
    for (caption, walked, gleaning) in [
        ("unopened — one mark in the dark (§10)", 0, false),
        ("part walked — a lit region growing out of it", 120, false),
        (
            "all but the last square — once, twice, and the way out",
            511,
            false,
        ),
        // The errand a scroll sets. **No `Ω` anywhere in this one**, which is
        // the whole difference: the walk ends when the five `♦` are gathered,
        // so a way out would be a mark on screen that nothing answers to.
        ("set to glean — five spoils, and no way out", 120, true),
    ] {
        let map = stacks_screen(GridSize::new(35, 35), walked, gleaning);
        show(&format!("The stacks, {caption}"), &map);
    }

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

    distilling();
}

/// The alembic: the same vessel, boiling hard enough to throw bubbles out of it.
///
/// **The one part of the liquid picture that is a glyph rather than a colour**,
/// so unlike the roil above it survives a text dump — which is why this section
/// is short. What it is here for is the *sweep*: a bubble appears above the
/// face, climbs a cell or three, and goes out, and one still frame cannot show
/// that.
fn distilling() {
    const ROWS: u16 = 12;
    let tick = 1.0 / orbs_render::FLIP_HZ;

    println!("\n── The alembic — the same bath, boiling ──\n");
    println!("    █ liquid   ° a bubble that got out   · the last of it\n");
    println!("    A distillation is a harder boil than a digestion, so some of what");
    println!("    rises through the liquid breaks the surface and leaves. The face");
    println!("    itself is never marked: the level is read off solid-against-blank,");
    println!("    and a bubble on that cell would put the picture and the value at");
    println!("    odds on the one cell the value comes from.\n");

    let mut columns = Vec::new();
    for step in 0u16..8 {
        let phase = f32::from(step * orbs_render::RISE_EVERY) * tick;
        let mut frame = Frame::new(GridSize::new(2, ROWS));
        let area = Rect::new(0, 0, 2, ROWS);
        frame.painter(area).bath_meter_upward(
            area,
            5,
            12,
            Steep {
                phase,
                motion: orbs_render::Motion::Bubbling,
                breaking: true,
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
                line.push(cell.map_or(' ', |cell| cell.glyph));
            }
            line.push_str("  ");
        }
        println!("{}", line.trim_end());
    }
    println!("\n    0   1   2   3   4   5   6   7   ← rises, three ticks each");
    println!("    Only over a lit athanor, and only where the bar runs **upward**.");
    println!("    In the side-by-side layout *above* is rightward, which would put");
    println!("    these on the row over the athanor's own sparks — see `Steep::upward`.");
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
/// The real painter is `orbs_shell::sheet`. If this layout stops fitting, that
/// one has the same problem.
///
/// # What it deliberately does not show is the scribing guide
///
/// The guide needs a `Sim` — it lists the verbs of the spell's *domain* — so it
/// cannot be reached from here at all, and a hand-built replica of it would be a
/// second opinion about the vocabulary rather than a check on the layout.
///
/// It is also **absent at 80×22 in the real editor**, which is the size drawn
/// here: the guide yields whole below 97 columns rather than cramping itself in.
/// So this replica is accurate at the one size it draws, and `scripts/tui.sh` is
/// the gate for the guide.
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
    // **The editor's real vocabulary**, which this said nothing of for three
    // versions: `save` and `discard` were never words — the buffer writes itself
    // out a beat after the typing stops — and `guide` and `interpret` both
    // arrived without this replica hearing about it.
    let status = grid.rows.saturating_sub(2);
    painter.span(
        Pos::new(1, status),
        &Span::new("edit  guide  interpret  quit").with_style(Style::DIM),
    );
    painter.span(
        Pos::new(grid.cols.saturating_sub(5), status),
        &Span::new("3:11").with_style(Style::DIM),
    );

    frame.set_cursor(Some(Pos::new(1 + GUTTER + 10, 3)));
    frame
}

/// The archive's map, drawn by the painter the game actually uses (§10, §19).
///
/// **Not a replica, unlike [`weave_screen`].** The weave screen is drawn in
/// `orbs`, which this crate cannot reach, so that one is redrawn by hand and can
/// drift. The maze picture lives *here*, so this calls straight into
/// [`Painter::stacks`] and cannot disagree with the game about a single
/// square.
///
/// `walked` is how far along a snake through the grid the reading has gone,
/// which is enough to exercise every glyph: fog, the two marks, and the way out.
/// `gleaning` sets the errand a scroll sets — spoils scattered ahead of the
/// reading, and no way out at all.
fn stacks_screen(grid: GridSize, walked: usize, gleaning: bool) -> Frame {
    let (span_x, span_y): (usize, usize) = (33, 23);
    let mut squares = vec![
        Square {
            wall: true,
            marks: 0,
        };
        span_x * span_y
    ];

    // A boustrophedon corridor: along an odd row, down through the wall at the
    // end, back along the next. Every floor square reachable, which is what lets
    // one number choose how much is lit.
    let mut path: Vec<usize> = Vec::new();
    for band in 0..(span_y / 2) {
        let row = 2 * band + 1;
        let rightward = band % 2 == 0;
        for step in 1..span_x - 1 {
            let col = if rightward { step } else { span_x - 1 - step };
            path.push(row * span_x + col);
        }
        if band + 1 < span_y / 2 {
            let turn = if rightward { span_x - 2 } else { 1 };
            path.push((row + 1) * span_x + turn);
        }
    }
    for index in &path {
        squares[*index].wall = false;
    }

    // Walk it, marking as the maze does — the first stretch twice, so the
    // finished-with glyph appears rather than being a claim in a doc comment.
    for (step, index) in path.iter().take(walked).enumerate() {
        squares[*index].marks = if step < walked / 3 { 2 } else { 1 };
    }
    let at = path
        .get(walked.saturating_sub(1))
        .copied()
        .unwrap_or(path[0]);
    squares[at].marks = squares[at].marks.max(1);

    // Spread across the floor the reading has *not* reached, which is where the
    // sim scatters them — a spoil on a walked square would already be gathered.
    let spoils: Vec<usize> = if gleaning {
        path.iter()
            .skip(walked)
            .step_by(70)
            .take(5)
            .copied()
            .collect()
    } else {
        Vec::new()
    };

    let maze = Stacks {
        squares,
        width: u16::try_from(span_x).unwrap_or(u16::MAX),
        at,
        exit: (!gleaning).then(|| *path.last().unwrap_or(&0)),
        spoils,
    };

    let mut frame = Frame::new(grid);
    let area = Rect::new(0, 0, grid.cols, grid.rows);
    let mut painter = frame.painter(area);
    painter.border(area, Some("stacks"), Style::DIM);
    painter.stacks(area.inset(1), &maze);
    frame
}

/// The weave screen at the width it actually gets (§11.5).
///
/// **48 columns, not 80.** `DEEP_FOCUS_FLOOR` is 100×28 and panes tile side by
/// side above it, so the session pane is half the grid — the 80-column floor is
/// the *widest* single-pane case. Every sentence on this screen is authored
/// against the number below, and this is the cheapest place to find out when one
/// stops fitting, because it builds a Frame without a sim.
///
/// The real painter is `orbs::shell::loom`. If this stops fitting, so has that.
fn weave_screen(grid: GridSize) -> Frame {
    let mut frame = Frame::new(grid);
    let area = Rect::new(0, 0, grid.cols, grid.rows);
    let mut painter = frame.painter(area);
    painter.border(area, Some("weave"), Style::DIM);

    // **Progression runs rightward**, and the screen says so three times: the
    // bar fills right, the Ley Line runs right, and Mastery's tiers run right.
    // A tier's siblings stack *downward*, which is the other axis and the other
    // meaning — rightward is progress, downward is a choice.
    let label = "24 of 100";
    let width = grid
        .cols
        .saturating_sub(u16::try_from(label.len()).unwrap_or(9) + 3);
    painter.progress(
        Rect::new(1, 1, width, 1),
        24,
        100,
        Style::default().with_role(Role::Success),
        "24 experience of 100",
    );
    painter.span(Pos::new(width + 2, 1), &Span::new(label));

    // **The Ley Line is one line with its steps standing on it**, drawn across
    // the same cells the bar above uses — so a step at 16 stands one sixth along
    // and the fill either has reached it or has not. The two rows are one
    // picture, which is why the bar's scale is a fixed hundred.
    painter.span(
        Pos::new(1, 3),
        &Span::new("ley line").with_style(Style::DIM),
    );
    painter.rule(Pos::new(1, 4), width, Style::DIM);
    let at = 2 + u16::try_from(16 * u32::from(width.saturating_sub(3)) / 100).unwrap_or(0);
    painter.glyphs(Pos::new(at - 1, 4), "[", Style::DIM);
    painter.glyphs(Pos::new(at, 4), "\u{2022}", Style::default());
    painter.glyphs(Pos::new(at + 1, 4), "]", Style::DIM);
    // **The glyph is drawn silently and the state is said as a word**, which is
    // the §14 property this screen exists to check: `Painter::span` would push
    // `•` itself into the stream and tell a listener nothing. The real painter
    // does exactly this — see `loom::glyph`.
    painter.announce(UtteranceKind::TableRow, Role::Normal, "16: taken");
    painter.span(Pos::new(at - 1, 5), &Span::new("16").with_style(Style::DIM));

    // **Mastery is placed at cost too**, on the same cells: one trunk forking
    // into the first tier, then a line from each node to *its own* successor —
    // which is what makes it a tree rather than two rows of unrelated marks.
    // `«»` marks the aimed node, and it is Bright as well: the frame survives
    // greyscale, the brightness is what the eye finds first.
    painter.span(Pos::new(1, 7), &Span::new("mastery").with_style(Style::DIM));
    let along = |cost: u32| 2 + u16::try_from(cost * u32::from(width - 3) / 100).unwrap_or(0);
    painter.rule(Pos::new(1, 8), along(24) - 3, Style::DIM);
    painter.glyphs(Pos::new(along(24) - 2, 8), "\u{252c}", Style::DIM);
    painter.glyphs(Pos::new(along(24) - 2, 9), "\u{2514}", Style::DIM);
    for row in [8u16, 9] {
        painter.rule(
            Pos::new(along(24) + 2, row),
            along(40) - along(24) - 3,
            Style::DIM,
        );
    }
    for (cost, glyph) in [(24u32, "\u{25cb}"), (40, "\u{b7}")] {
        let x = along(cost);
        for row in [8u16, 9] {
            // The aimed one is the lower node of the first tier.
            let (open, close) = if cost == 24 && row == 9 {
                ("\u{ab}", "\u{bb}")
            } else {
                ("[", "]")
            };
            painter.glyphs(Pos::new(x - 1, row), open, Style::DIM);
            painter.glyphs(Pos::new(x, row), glyph, Style::default());
            painter.glyphs(Pos::new(x + 1, row), close, Style::DIM);
            let state = if cost == 24 { "open" } else { "locked" };
            painter.announce(
                UtteranceKind::TableRow,
                Role::Normal,
                &format!("{cost}: {state}"),
            );
        }
        painter.span(
            Pos::new(x - 1, 10),
            &Span::new(&cost.to_string()).with_style(Style::DIM),
        );
    }

    // **The details panel**, bottom right: what the aimed node is, what it
    // costs, and the two facts that are not the same fact. *Unlocked* is whether
    // it can be reached; *active* is whether what it grants is in effect. A
    // mastery node can be unlocked and idle because nobody chose it, or unlocked
    // and idle for ever because a sibling took the tier's one choice.
    let panel = Rect::new(
        grid.cols.saturating_sub(31),
        grid.rows.saturating_sub(7),
        30,
        5,
    );
    painter.border(panel, Some("details"), Style::DIM);
    painter.span(
        Pos::new(panel.col + 1, panel.row + 1),
        &Span::new("not taught yet"),
    );
    painter.span(
        Pos::new(panel.col + 1, panel.row + 2),
        &Span::new("costs 24").with_style(Style::DIM),
    );
    painter.span(
        Pos::new(panel.col + 1, panel.row + 3),
        &Span::new("unlocked").with_style(Style::default().with_role(Role::Success)),
    );
    painter.span(
        Pos::new(panel.col + panel.cols - 9, panel.row + 3),
        &Span::new("inactive").with_style(Style::DIM),
    );

    // The row that must never be blank: §6 forbids a dead end, and at the
    // command line this row is the whole interface.
    painter.span(
        Pos::new(1, grid.rows.saturating_sub(2)),
        &Span::new("ley  mastery  take  quit").with_style(Style::DIM),
    );
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
/// **establish the minimum window at which tier 2 is offered**.
///
/// **The question changed with §19's fixed grid and the answer is now easier.**
/// There are no tiers: the grid is [`GRID`] on every window, so the worst case
/// is not "the smallest window that can still host four panes" but simply four
/// panes, because that is the same screen everywhere. What the window decides is
/// the *glyph*, and the hard case there is the smallest one the game will draw
/// rather than refuse — [`minimum_window`], derived rather than written down,
/// because §19's standing lesson from four failed attempts at the CRT overscan
/// is *compute the constant, do not reason about it*.
///
/// Two of the six conditions are not in a [`Frame`] and cannot be. Peak-threat
/// CRT and the phosphor are frontend enrichment — rule 2 — so they are read on
/// the running game at this window with `F3`, and what this screen establishes
/// is that everything *informational* survives at the smallest glyph the game
/// ever draws. The final judgement is a human one; this prepares it and cannot
/// make it.
fn worst_case() {
    let window = minimum_window();
    let grid = GRID;
    let scale = orbs_render::scale_for(window);
    let cell_width = f32::from(orbs_render::CELL_WIDTH) * scale;
    let cell_height = f32::from(orbs_render::CELL_HEIGHT) * scale;

    println!("\nWorst-case legibility — DESIGN.md §15\n");
    println!(
        "  smallest window the game will draw on   {}x{}",
        window.0, window.1
    );
    println!(
        "  the picture, always                     {}x{} px -> {}x{} cells",
        PICTURE.0, PICTURE.1, grid.cols, grid.rows,
    );
    println!(
        "  glyph there                             {cell_width}x{cell_height} physical pixels"
    );
    println!("  a one-character tell is                 {cell_width} pixels wide\n");

    let frame = siege(grid, DisplayMode::Deep);
    show(
        "Worst case — four panes, smallest glyph, siege, eldritch",
        &frame,
    );

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

/// The smallest window the game will draw on rather than refuse.
///
/// Below [`MIN_SCALE`] the frontend paints a "window too small" card instead —
/// minification drops strokes out of an 8×16 bitmap rather than shrinking it —
/// so this is the boundary, and the glyph here is the smallest one a player can
/// be asked to read.
///
/// A search rather than a constant anyone should retype, and searched **per
/// axis**: [`scale_for`](orbs_render::scale_for) takes the smaller of the two
/// ratios, so the smallest qualifying window is the pair of per-axis minima and
/// searching both at once would be a slower way to the same number.
fn minimum_window() -> (u32, u32) {
    /// Past any window a 2026 desktop will present, and small enough to search
    /// exhaustively in microseconds.
    const LIMIT: u32 = 8192;

    let drawable = |window: (u32, u32)| orbs_render::scale_for(window) >= orbs_render::MIN_SCALE;

    let width = (1..=LIMIT)
        .find(|width| drawable((*width, LIMIT)))
        .expect("some width is drawable");
    let height = (1..=LIMIT)
        .find(|height| drawable((LIMIT, *height)))
        .expect("some height is drawable");
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
            "sanctum",
            "DEGRADED",
            Role::Danger,
            Some("barrier integrity 34%"),
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

/// Four domain panes under attack, with every domain on the rail beside them.
fn siege(grid: GridSize, mode: DisplayMode) -> Frame {
    let layout = ScreenLayout::compute(&ScreenRequest {
        grid,
        main_panes: 4,
        rail: true,
        mode,
        input_rows: 1,
    });

    let mut frame = Frame::new(grid);
    let panes: [(&str, fn(&mut Painter<'_>, Rect)); 4] = [
        ("laboratory", laboratory),
        ("sanctum", sanctum),
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

    // The rail is awareness only — never commandable (§9). One box per domain,
    // and the last two are rooms the tower has not built yet: a dim rule with no
    // name, so the seven slots keep fixed positions while the game grows into
    // them.
    let rail = layout.rail();
    if !rail.is_empty() {
        let mut painter = frame.painter(rail);
        painter.border(rail, Some("tower"), Style::DIM);
        let known = [
            ("laboratory", "working"),
            ("archive", "working"),
            ("lens", "probing"),
            ("grimoire", "idle"),
            ("forge", "idle"),
        ];
        for (index, slot) in layout.rail_boxes().iter().enumerate() {
            let Some((name, state)) = known.get(index) else {
                painter.glyphs(
                    slot.origin(),
                    &"·".repeat(usize::from(slot.cols)),
                    Style::DIM,
                );
                continue;
            };
            painter.span(
                slot.origin(),
                &Span::new(name)
                    .with_style(Style::NORMAL)
                    .with_kind(UtteranceKind::Heading),
            );
            painter.glyphs(
                Pos::new(slot.col, slot.row.saturating_add(1)),
                &format!("  {state}"),
                Style::DIM,
            );
        }
        let foot = layout.rail_foot();
        painter.rule(foot.origin(), foot.cols, Style::DIM);
        painter.span(
            Pos::new(foot.col, foot.row.saturating_add(1)),
            &Span::new("tick  4210").with_style(Style::DIM),
        );
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
fn sanctum(painter: &mut Painter<'_>, area: Rect) {
    painter.span(
        area.origin(),
        &Span::new("barrier").with_style(Style::NORMAL),
    );
    painter.progress(
        Rect::new(area.col, area.row + 1, area.cols.min(20), 1),
        34,
        100,
        Style::DANGER,
        "barrier integrity 34 percent",
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

/// A ward part-broken — the sheet a code-breaker keeps beside them (§10).
///
/// **Every glyph here has to be one CP437 can draw**, and this example is where
/// that gets caught: the board's own tests assert the repertoire, but only a
/// rendered frame shows whether four sigils and four pegs read as two columns or
/// as a smear. `▪` failed the first pass and became `■`.
fn ward(grid: GridSize) -> Frame {
    let board = orbs_render::Board {
        attempts: vec![
            orbs_render::Attempt {
                figure: [0, 1, 2, 3],
                aligned: 0,
                astray: 2,
            },
            orbs_render::Attempt {
                figure: [0, 4, 2, 3],
                aligned: 1,
                astray: 2,
            },
            // **A repeated sigil, and four pegs.** Both are things this sheet
            // could not show before: a code may hold a sigil twice now (§19), so
            // a figure may too — and `aligned + astray` reaching the full width
            // is the widest the peg column ever gets. These three rows answer a
            // real code, `quartz pewter pewter borax`.
            orbs_render::Attempt {
                figure: [4, 4, 2, 3],
                aligned: 1,
                astray: 3,
            },
        ],
        aperture: [4, 4, 2, 3],
        // `tower::ward`'s own words. The sim hands these through `Ward::view`; an
        // example has no sim, so it repeats them — and this is the surface where a
        // header wider than its column, or a legend that runs into the border,
        // shows up as a picture rather than as a passing assertion.
        sockets: ["first", "second", "third", "fourth"],
        sigils: ["nitre", "alum", "borax", "quartz", "pewter", "ochre"],
    };

    let layout = ScreenLayout::compute(&ScreenRequest::single(grid));
    let mut frame = Frame::new(grid);
    let pane = layout.main()[0];
    let mut painter = frame.painter(pane);
    painter.border(pane, Some("lens"), Style::DIM);

    let (cols, rows) = board.size();
    let at = Rect::new(pane.col + 2, pane.row + 2, cols + 2, rows + 2);
    painter.border(at, Some("ward"), Style::DIM);
    let inside = at.inset(1);
    for row in 0..inside.rows {
        let Some(cells) = board.row(usize::from(row)) else {
            break;
        };
        for (col, (glyph, style, tint)) in cells.into_iter().enumerate() {
            let Ok(col) = u16::try_from(col) else { break };
            let cell = Pos::new(inside.col + col, inside.row + row);
            painter.glyphs(cell, &glyph.to_string(), style);
            if let Some(tint) = tint {
                painter.tint(
                    Rect::new(cell.col, cell.row, 1, 1),
                    orbs_render::Wash::plain(tint),
                );
            }
        }
    }
    painter.announce(
        UtteranceKind::Progress,
        Role::Normal,
        "3 pressed, last 2 aligned 1 astray, 1 held",
    );
    frame
}

/// A course of wards, part-drawn (§10, `tower::pylon`).
///
/// **The one surface where a Hanoi position can actually be judged.** Every
/// other check on this board is an assertion about widths; only a rendered frame
/// shows whether three centred stacks of blocks read as three stacks or as a
/// smear, and whether the staircase makes the rule — *a greater ward will not
/// rest upon a lesser* — visible without a word of explanation.
///
/// The position is a real one and is **checked rather than invented**: it is
/// where `cycle(4)` actually stands after seven hauls, with the greatest ward
/// alone in the wellspring and the other three stacked in the conduit. The first
/// version drew `[4,3] / [2,1] / []`, a legal position the solver never passes
/// through — and since this screen is the one place a Hanoi position is judged by
/// eye, a wrong literal here reads as a solver bug.
fn pylon(grid: GridSize) -> Frame {
    let course = orbs_render::Pylon {
        stations: [vec![4], vec![3, 2, 1], Vec::new()],
        // `tower::pylon`'s own words. The sim hands these through
        // `Course::view`; an example has no sim, so it repeats them — and this
        // is the surface where a header wider than its column shows up as a
        // picture rather than as a passing assertion.
        names: ["wellspring", "conduit", "barrier"],
        height: 4,
        hauls: 7,
        integrity: 62,
        // What `prose.toml`'s `pylon_tally` renders to. An example has no prose,
        // so it repeats the line — the same dodge the sockets and sigils above
        // take, and the same reason: this is where a line too wide for its box
        // shows up as a picture rather than as an assertion.
        tally: "4 wards, 7 hauled".to_owned(),
    };

    let layout = ScreenLayout::compute(&ScreenRequest::single(grid));
    let mut frame = Frame::new(grid);
    let pane = layout.main()[0];
    let mut painter = frame.painter(pane);
    painter.border(pane, Some("sanctum"), Style::DIM);

    let (cols, rows) = orbs_render::Pylon::size();
    let at = Rect::new(pane.col + 2, pane.row + 2, cols + 2, rows + 2);
    painter.border(at, Some("pylon"), Style::DIM);
    let inside = at.inset(1);
    for row in 0..inside.rows {
        let Some(cells) = course.row(usize::from(row)) else {
            break;
        };
        for (col, (glyph, style, tint)) in cells.into_iter().enumerate() {
            let Ok(col) = u16::try_from(col) else { break };
            let cell = Pos::new(inside.col + col, inside.row + row);
            painter.glyphs(cell, &glyph.to_string(), style);
            if let Some(tint) = tint {
                painter.tint(
                    Rect::new(cell.col, cell.row, 1, 1),
                    orbs_render::Wash::plain(tint),
                );
            }
        }
    }
    painter.announce(
        UtteranceKind::Progress,
        Role::Normal,
        "4 wards, 7 hauled, the walls at 62",
    );
    frame
}

/// A figure part-sung, two ticks from the rule (§10, `tower::chant`).
///
/// **The one surface where the timing can actually be judged.** Every other
/// check on this board asserts a width; only a rendered frame shows whether the
/// gap between the rule and the nearest syllable reads as *approach* — and that
/// gap is the whole mechanic, because a press counts on the last two ticks of it
/// and nowhere else.
///
/// `until` is **2**, deliberately: at nought the nearest syllable sits on the
/// rule and the board looks static again, which is exactly what it looked like
/// for a whole approach when `Figure` carried no `until` at all. A screen drawn
/// at the one value that hides the bug would be worse than no screen.
fn chant(grid: GridSize) -> Frame {
    let figure = orbs_render::Figure {
        // Lane indices into `lanes` below, nearest first.
        coming: vec![1, 3, 0, 2],
        // `tower::chant`'s own words and glyphs. The sim hands these through
        // `Sim::figure`; an example has no sim, so it repeats them — and this is
        // the surface where a header wider than its column shows up as a picture
        // rather than as a passing assertion.
        lanes: vec![
            ('\u{25C4}', "leftward"),
            ('\u{25B2}', "skyward"),
            ('\u{25BC}', "earthward"),
            ('\u{25BA}', "rightward"),
        ],
        sung: vec![true, true, false, true],
        until: 2,
        remaining: 4,
        // What `prose.toml`'s `chant_tally` renders to.
        tally: "4 to come, 1 missed".to_owned(),
    };

    let layout = ScreenLayout::compute(&ScreenRequest::single(grid));
    let mut frame = Frame::new(grid);
    let pane = layout.main()[0];
    let mut painter = frame.painter(pane);
    painter.border(pane, Some("menagerie"), Style::DIM);

    let (cols, rows) = orbs_render::Figure::size();
    let at = Rect::new(pane.col + 2, pane.row + 2, cols + 2, rows + 2);
    painter.border(at, Some("figure"), Style::DIM);
    let inside = at.inset(1);
    for row in 0..inside.rows {
        let Some(cells) = figure.row(usize::from(row)) else {
            break;
        };
        for (col, (glyph, style, tint)) in cells.into_iter().enumerate() {
            let Ok(col) = u16::try_from(col) else { break };
            let cell = Pos::new(inside.col + col, inside.row + row);
            painter.glyphs(cell, &glyph.to_string(), style);
            if let Some(tint) = tint {
                painter.tint(
                    Rect::new(cell.col, cell.row, 1, 1),
                    orbs_render::Wash::plain(tint),
                );
            }
        }
    }
    // **`next` first**, which is what makes the room playable by ear: a reader
    // has no rows, so a line leading with a tally would leave them nothing to
    // act on.
    painter.announce(
        UtteranceKind::Progress,
        Role::Normal,
        "skyward next, 4 to come, 1 missed",
    );
    frame
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
        "barrier integrity 34% [ DEGRADED ]",
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
    // What replaced §9's fidelity table (§19). It used to have a grid per
    // window; now the grid is the same in every row and the *glyph* is what
    // moves, which is the whole change in one printout. The bars are the 4:3
    // letterbox — a row with none of them is a window that is already 4:3.
    println!("\nThe picture — one grid, and how big a cell of it is (DESIGN.md §19)\n");
    println!(
        "  the grid is always {}×{}, i.e. {PICTURE:?} px\n",
        GRID.cols, GRID.rows
    );
    for window in [
        (1280u32, 720u32),
        (1920, 1080),
        (2560, 1440),
        (3840, 2160),
        (1366, 768),
        (960, 720),
        (800, 600),
    ] {
        let scale = orbs_render::scale_for(window);
        if scale < orbs_render::MIN_SCALE {
            println!(
                "  {:>5}×{:<5}  {scale:.3}× — too small to draw",
                window.0, window.1
            );
            continue;
        }
        let (across, down) = (f32::from(PICTURE.0) * scale, f32::from(PICTURE.1) * scale);
        println!(
            "  {:>5}×{:<5}  {scale:.3}× → {}×{} px glyph, picture {across:.0}×{down:.0}, \
             bars {:.0}×{:.0}",
            window.0,
            window.1,
            f32::from(orbs_render::CELL_WIDTH) * scale,
            f32::from(orbs_render::CELL_HEIGHT) * scale,
            // Clamped off zero, so an exactly-4:3 window prints `0` rather than
            // the `-0` a float subtraction leaves behind.
            ((orbs_render::pixels(window.0) - across) / 2.0).max(0.0),
            ((orbs_render::pixels(window.1) - down) / 2.0).max(0.0),
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
