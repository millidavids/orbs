//! The command line, and everything the orb has said.
//!
//! Every glyph comes from a [`Record`](orbs_render::Record) the sim emitted —
//! nothing is composed at paint time (rule 4, from the drawing end).
//!
//! The scrollback is painted from its tail, which keeps a screen reader and a
//! sighted player level: `Speech` describes one frame, so describing more than
//! the screen shows would break rule 2. History is `Records`'s job.

use orbs_render::{
    Crossing, DisplayMode, Frame, Painter, Pos, RecordView, Rect, ScreenLayout, ScreenRequest,
    Span, Style, Toward, UtteranceKind,
};
use orbs_sim::Sim;

use super::editor::Editor;
use super::line::Line;
use super::linear::Linear;
use super::menu::Menu;
use super::reveal::Reveal;
use super::screen::Screen;
use super::tapestry::Tapestry;
use super::transition::PaneTransition;

/// Everything a frame is drawn from, borrowed for the one call.
///
/// A struct, so no two `&str`-ish arguments can be transposed; `Frame` and
/// `Linear` stay apart as what `paint` writes. Not `Copy`: a viewport only the
/// painter can size makes some fields `&mut`.
pub struct View<'a> {
    /// The world, for everything the frame says.
    pub sim: &'a Sim,
    /// What is being typed.
    pub line: &'a Line,
    /// Grid, tier and focus mode.
    pub screen: &'a Screen,
    /// Where the pane animation has reached.
    pub panes: &'a PaneTransition,
    /// How much of the newest output has arrived.
    pub reveal: &'a Reveal,
    /// What Tab last offered, and where a cycle has reached.
    pub offered: &'a super::offering::Offered,
    /// The inline suggestion trailing the caret.
    pub ghost: &'a str,
    /// §10.1's instruments, as the sim last reported them.
    pub panel: &'a super::glance::Panel,
    /// How far back through the transcript the player is looking.
    pub scroll: &'a super::scrollback::Scroll,
    /// Where the athanor's fire has reached, and whether it burns at all.
    pub bench: &'a super::bench::Bench,
    /// How far each changing number has rolled.
    pub climb: &'a super::climb::Climb,
    /// The spell being edited, if the player is in the editor.
    ///
    /// `&mut`, because the editor's viewport follows the caret and only the
    /// painter knows how tall the pane is — see `Editor::scroll_to`.
    pub editing: Option<&'a mut Editor>,
    /// The weave screen, if the player has it open.
    ///
    /// Read-only, unlike `editing`: this surface has no viewport that follows a
    /// caret, so the painter has nothing to tell it.
    pub weaving: Option<&'a Tapestry>,
    /// Whether the arrow keys are walking the archive's stacks.
    ///
    /// A flag, since this surface holds no state: it decides whether the maze
    /// is drawn over the pane or beside the transcript.
    pub walking: bool,
    /// The orb's menu, if `quit` has opened it.
    ///
    /// Read-only, like `weaving`, and for the same reason: it holds a typed line
    /// and nothing that follows a viewport.
    pub menuing: Option<&'a Menu>,
    /// The manual, if it is open.
    ///
    /// The second `&mut`, for the editor's exact reason: a chapter is wrapped to
    /// the pane's width and scrolled within it, and only the painter knows how
    /// wide and how tall the pane is — see
    /// [`ManualReader::measured`](crate::ManualReader::measured).
    pub reading_manual: Option<&'a mut crate::ManualReader>,
    /// The installation, while the tower being drawn is its practice tower.
    pub installing: Option<&'a crate::install::Install>,
}

/// Paint the session into `frame`.
///
/// From a real `ScreenLayout`, already interpolated (see [`PaneTransition`]),
/// so every rectangle is wherever the motion has reached this frame.
pub fn paint(
    frame: &mut Frame,
    linear: &mut Linear,
    passing: &mut super::passing::Passing,
    view: View<'_>,
) {
    // The one place a screen is handed over, around `paint_view`'s four exits.
    // `passing` is a parameter, the one thing here that outlives the frame.
    // Only crossed if the kept screen still describes this grid.
    let crossing = passing.crossing().filter(|_| passing.describes(frame));
    let crossed = paint_view(frame, linear, view);

    let Some(crossing) = crossing else {
        // Nothing moving, so this screen's regions are what the next crossing
        // will have to leave *from* — and only this function knows them.
        passing.remember(crossed);
        return;
    };
    // The union of both screens' regions. Sized from the arriving screen alone,
    // a laboratory leaving for a room with no instrument panel would find an
    // empty block and cut rather than leave. `Kept::cell` reads blank outside
    // its own rectangle, so the wider region draws correctly either way.
    let regions = crossed.union(passing.remembered());
    for (area, toward) in regions.moving() {
        frame.cross(area, Crossing { toward, ..crossing }, Some(passing.kept()));
    }
    // The rail, and only out of boot. See `Crossed::rail`: every other crossing
    // leaves it standing. Arriving out of the card it is not on screen yet, so
    // it pushes in from the edge it lives against.
    if passing.is_waking() {
        frame.cross(
            regions.rail,
            Crossing {
                toward: Toward::Right,
                ..crossing
            },
            Some(passing.kept()),
        );
    }
}

/// How many regions one screen change can move.
///
/// The strip along the top, and the block's two slabs — see [`Crossed::parts`].
/// A room change moves the strip and the block and leaves the transcript
/// between them standing; a surface taking the whole pane moves the lot.
/// Nothing else is ever crossed: the border, its title, the tower rail and the
/// prompt hold still.
const MOST: usize = 3;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Crossed {
    /// The regions that move, each with the edge it leaves by.
    ///
    /// Slot 0 is the gauges and the road, 1 and 2 the block's two slabs (see
    /// [`slabs`]). Fixed, so [`union`](Self::union) pairs them across screens.
    parts: [(Rect, Toward); MOST],
    /// The whole interior, when a surface replaced it. Overrides the two above.
    whole: Rect,
    /// The tower rail, which moves for one crossing only.
    ///
    /// Awareness, not a view, so ordinary crossings leave it standing. Out of
    /// the boot card it pushes in from the right (`Passing::is_waking`).
    rail: Rect,
}

impl Crossed {
    /// The regions of two screens, together.
    ///
    /// A crossing has to cover whatever *either* side put on screen — the
    /// laboratory's panel leaving for a forge with none, the forge's lattice
    /// arriving over a laboratory with none.
    /// [`Kept::cell`](orbs_render::Kept::cell) reads blank outside its own
    /// rectangle, so the half that never had the region has nothing there.
    fn union(self, other: Self) -> Self {
        // Slot by slot: the two screens' slabs mean the same thing in the same
        // place, so the directions match and only the rectangles need joining.
        let mut parts = self.parts;
        for (slot, (area, _)) in parts.iter_mut().zip(other.parts) {
            slot.0 = slot.0.union(area);
        }
        Self {
            parts,
            whole: self.whole.union(other.whole),
            rail: self.rail.union(other.rail),
        }
    }

    /// A surface that replaced everything inside the border.
    const fn all(interior: Rect, rail: Rect) -> Self {
        Self {
            parts: [(Rect::EMPTY, Toward::Up); MOST],
            whole: interior,
            rail,
        }
    }

    /// What to cross, and which way each part goes.
    ///
    /// `whole` replaces the parts rather than joining them, or one crossing
    /// gathers to three centres. Empty rectangles are dropped.
    fn moving(self) -> impl Iterator<Item = (Rect, Toward)> {
        // `Toward` means nothing to a whole-pane change — a `Gather` converges
        // from every side — so the value paired here is a default, not a claim.
        let mut parts = self.parts;
        if !self.whole.is_empty() {
            parts = [(Rect::EMPTY, Toward::Up); MOST];
            parts[0] = (self.whole, Toward::Right);
        }
        parts.into_iter().filter(|(area, _)| !area.is_empty())
    }
}

/// Paint the session, and say what a crossing would cover.
fn paint_view(frame: &mut Frame, linear: &mut Linear, view: View<'_>) -> Crossed {
    let View {
        sim,
        line,
        screen,
        panes,
        reveal,
        offered,
        ghost,
        panel,
        scroll,
        bench,
        climb,
        editing,
        weaving,
        walking,
        menuing,
        reading_manual,
        installing,
    } = view;
    let grid = frame.size();
    // The prompt spends a second row so it can be drawn at double size — see
    // `orbs_render::INPUT_ROWS`, which is a constant now that a resize cannot
    // change how many cells there are to spend.
    let input_rows = orbs_render::INPUT_ROWS;
    // A Tab listing takes a row from the layout, so the pane above shrinks while
    // it is up. At `input.row - 1` it landed on the session pane's bottom border
    // — `compute` hands the main tiler everything up to `above_input` — so
    // completing a word erased the border. Reserving the row is what a terminal
    // does anyway: the list pushes the transcript up rather than scribbling.
    let listing = u16::from(!offered.is_empty());
    let layout = panes.layout(grid, screen.mode, input_rows.saturating_add(listing));
    let main = layout.main();
    let first = main.first().copied().unwrap_or(Rect::EMPTY);

    // The border title carries the readings only when there is no rail.
    let carry_readings = layout.rail().is_empty();

    // The same rectangle, not a second pane, so one key compares them (§14).
    // A surface takes the session pane rather than adding one, since §9's pane
    // count is a progression axis. The rail is hoisted above every modal
    // branch, being awareness rather than a view.
    let rail = || (layout.rail(), layout.rail_boxes(), layout.rail_foot());

    // The interior, not the pane. A crossing never touches the border: the title
    // is the one continuously-visible statement of place (§7 — paths are
    // places), and a box coming apart reads as the *machine* breaking rather
    // than the screen changing, which is why §19 cut the tube strike.
    let interior = first.inset(1);
    // The three surfaces that replace the whole pane move all of it —
    // `Focus::takes_the_pane` names the same set, and `Passing` asks it there.
    let whole = Crossed::all(interior, layout.rail());

    // `Focus::of`'s order, kept in step with it: the manual over the menu over
    // the tower's surfaces. No rail beside the menu, which steps out of the
    // tower.
    if let Some(reader) = reading_manual {
        super::manual::paint(frame, reader, first, sim.prose());
        return whole;
    }

    if let Some(menu) = menuing {
        super::menu::paint(frame, menu, first, sim.prose());
        return whole;
    }

    if let Some(tapestry) = weaving {
        super::loom::paint(frame, tapestry, first, sim.prose());
        let (at, boxes, foot) = rail();
        super::rail::paint(frame, sim, screen, at, boxes, foot, &panel.briefs);
        return whole;
    }

    if let Some(editor) = editing {
        if let Some((col, row)) = super::sheet::paint(frame, editor, first, sim.prose()) {
            frame.set_cursor(Some(Pos::new(col, row)));
        }
        let (at, boxes, foot) = rail();
        super::rail::paint(frame, sim, screen, at, boxes, foot, &panel.briefs);
        // No prompt row: the prompt is dead while the editor has the keyboard
        // (`editing::not_editing`), and drawing a caret it cannot accept a
        // keystroke into is the clearest possible lie about where typing goes.
        return whole;
    }

    // The maze pane, when the arrows have it. The map itself draws beside the
    // transcript whenever a maze is open, so a spell's solving is watchable.
    if walking && let Some(maze) = panel.stacks.as_ref() {
        let mut painter = frame.painter(first);
        super::stacks::paint_alone(&mut painter, first, maze, sim.prose());
        let (at, boxes, foot) = rail();
        super::rail::paint(frame, sim, screen, at, boxes, foot, &panel.briefs);
        // No prompt row and no caret, exactly as the editor leaves none: every
        // keystroke is discarded while this is open, and a caret is the game's
        // one promise about where typing lands.
        frame.set_cursor(None);
        return whole;
    }

    let crossed = if linear.showing() {
        super::linear::paint(
            linear,
            frame,
            sim,
            screen,
            first,
            carry_readings,
            panel,
            scroll,
            bench,
            installing,
        );
        // `F5`'s mirror is the fourth surface that replaces the pane, and the one
        // no `Focus` variant names — so it is spelled out here rather than
        // derived, and `Showing::mirrored` is its counterpart in the clock.
        whole
    } else {
        let parts = session(
            frame,
            sim,
            screen,
            first,
            carry_readings,
            reveal,
            panel,
            scroll,
            bench,
            climb,
            installing,
        );
        Crossed {
            parts,
            whole: Rect::EMPTY,
            rail: layout.rail(),
        }
    };
    let (at, boxes, foot) = rail();
    super::rail::paint(frame, sim, screen, at, boxes, foot, &panel.briefs);
    // The ghost arrives already computed. `shell::input::suggest` owns the one
    // call and names what invalidates it, so no second copy can disagree with
    // the line it trails and it is not rebuilt on the ~59 frames in 60 where
    // none of its inputs moved.
    let (list_area, prompt_area) = split_input(layout.input(), listing);
    candidates(frame, list_area, offered);

    input_line(frame, prompt_area, line, &sim.prompt(), ghost);
    crossed
}

/// The reserved rows, divided into the Tab listing and the prompt proper.
const fn split_input(input: Rect, listing: u16) -> (Rect, Rect) {
    if listing == 0 || input.rows <= listing {
        return (Rect::EMPTY, input);
    }
    (
        Rect::new(input.col, input.row, input.cols, listing),
        Rect::new(
            input.col,
            input.row + listing,
            input.cols,
            input.rows - listing,
        ),
    )
}

/// What Tab offered, on the row above the prompt.
///
/// Transient Frame content, never a record (§13); §3's logging rule is about
/// the orb's output, not the shell's affordances.
fn candidates(frame: &mut Frame, area: Rect, offered: &super::offering::Offered) {
    if offered.is_empty() || area.is_empty() {
        return;
    }
    let row = area.row;
    let mut painter = frame.painter(area);

    // The one the cycle has reached is drawn bright: repeated Tab changes the
    // *line*, so the list has to say which of them the line now holds, or the
    // player walks a wall of equal-looking words with no idea where they are.
    let mut at = area.col;
    for (index, option) in offered.options.iter().enumerate() {
        if at >= area.right() {
            break;
        }
        let current = offered.current == Some(index);
        let style = if current { Style::SUCCESS } else { Style::DIM };
        at = at.saturating_add(painter.glyphs(Pos::new(at, row), option, style));
        at = at.saturating_add(2);
    }

    // Spoken once as a sentence, not a span per candidate — eight utterances a
    // keypress teach a reader nothing. §19: a visual constraint must not become
    // an informational one, and nor may a visual *affordance*, so the mark a
    // sighted player sees is said aloud too.
    let spoken = offered
        .options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            if offered.current == Some(index) {
                format!("{option} (chosen)")
            } else {
                option.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    painter.announce(UtteranceKind::Hint, Style::DIM.role, &spoken);
}

/// Paint the screen as it exists partway through the boot sequence.
///
/// Draws only what has arrived: `Dark` paints nothing at all, so the screen
/// really does open black. The prompt types itself, the pane border then draws
/// itself a cell at a time, and the POST card follows.
///
/// No Bevy resources, since a dump builds no `App` (§19), and no `Sim`, since
/// the boot draws nothing from the world.
pub fn paint_booting(frame: &mut Frame, stage: crate::stage::Stage, progress: f32, engine: &str) {
    let grid = frame.size();
    let layout = ScreenLayout::compute(&ScreenRequest::single(grid));
    let pane = layout.main().first().copied().unwrap_or(Rect::EMPTY);

    if stage.has_frame() {
        let mut painter = frame.painter(pane);
        // Untitled: `Painter::border` announces its title as a heading, and a
        // border drawing itself a cell at a time would speak a pane that is not
        // there yet. The title arrives with the game.
        painter.border_revealed(pane, Style::DIM, stage.frame_progress(progress));
    }

    crate::post::paint(frame, stage, progress, engine);

    // The card leaves by a `Gather`'s leaving half, as any full-pane surface
    // does. The box stays: it *is* the game's pane, narrowed as the rail opens.
    if let Some(closing) = stage.closing(progress) {
        frame.fold(
            pane.inset(1),
            Crossing {
                passage: orbs_render::Passage::Gather,
                toward: Toward::Right,
                progress: closing * 0.5,
            },
        );
    }

    // No prompt during boot. It used to type itself here, caret and all, on a
    // screen where every keyed system is gated on `booted` — so the first
    // affordance the game showed was one that did not work.
}

/// The transcript: what was typed and what came back.
///
/// Returns the regions a crossing moves, never the transcript a room change
/// leaves standing — returned, since only these splits know them.
pub(super) fn session(
    frame: &mut Frame,
    sim: &Sim,
    screen: &Screen,
    pane: Rect,
    carry_readings: bool,
    reveal: &Reveal,
    panel: &super::glance::Panel,
    scroll: &super::scrollback::Scroll,
    bench: &super::bench::Bench,
    climb: &super::climb::Climb,
    installing: Option<&crate::install::Install>,
) -> [(Rect, Toward); MOST] {
    if pane.is_empty() {
        return [(Rect::EMPTY, Toward::Up); MOST];
    }
    let mut painter = frame.painter(pane);
    // The pane says where you are (§7), and so which commands will work; the
    // prompt stands under every pane. The `F4` hint rides along.
    let switch = match screen.mode {
        DisplayMode::Deep => "wide",
        DisplayMode::Wide => "deep",
    };
    // Scrolled-back replaces the `F4` hint in the title, since output keeps
    // arriving and the way back is more urgent; a title is heard as a heading
    // (§14). Plain ASCII, as an em-dash once drew as `?`. Reading outranks
    // scrolled-back, which outranks the focus key.
    let hint = if scroll.is_reading() {
        sim.prose().line("unfurl_keys", &[])
    } else if scroll.is_back() {
        "PgDn newest".to_owned()
    } else {
        format!("F4 {switch}")
    };
    let title = if carry_readings {
        format!(
            "{}  tick {}  {:.2}x  {}x{}  {}  {hint}",
            sim.location(),
            sim.tick().get(),
            screen.scale(),
            screen.grid.cols,
            screen.grid.rows,
            focus(screen),
        )
    } else {
        format!("{}  {hint}", sim.location())
    };
    painter.border(pane, Some(&title), Style::DIM);

    let interior = pane.inset(1);
    let mut body = interior;

    // A meter shows what an action's slot has left (§5.0). §10.1's panel
    // follows the pane's shape. §11.5's road is taken first, so a short pane
    // loses it before the transcript, with the tower's two standings above it
    // (`gauges::split`). The installation's strip takes their place.
    if let Some(install) = installing {
        let strip = crate::install::split(body);
        crate::install::paint(&mut painter, strip.area, install);
        body = strip.rest;
    } else {
        let gauges = super::gauges::split(body);
        super::gauges::paint(
            &mut painter,
            gauges.area,
            panel.station,
            panel.rankward,
            panel.rank.as_deref(),
            panel.ledger.as_ref(),
            climb,
            sim.prose(),
        );
        body = gauges.rest;

        let road = super::road::split(body, panel.line.as_ref());
        if let Some(line) = panel.line.as_ref() {
            super::road::paint(&mut painter, road.area, line, climb, sim.prose());
        }
        body = road.rest;
    }
    // The strip along the top: the standings and the road. It leaves upward, by
    // the edge it sits against. Measured as
    // what `body` lost rather than the sum of the two splits, because either can
    // refuse at a short pane and a sum of refusals is a rectangle nothing drew
    // in. Only ever rows, so the second slab is empty and is dropped.
    let [strip, _] = slabs(interior, body);
    let after_strip = body;

    let instruments = panel.instruments.as_slice();
    let split = super::panel::split(body, instruments);
    super::panel::paint(&mut painter, split, instruments, &panel.domain, bench);
    body = split.rest;

    // §10's map, beside the panel and inboard of it. Second, deliberately: the
    // first split takes from the whole body and hands on the rest, so the second
    // is the one whose refusal can fire — and an instrument row is load-bearing
    // where a map is a convenience. Running it first also puts it outboard.
    let map = super::stacks::split(body, panel.stacks.as_ref());
    if let Some(maze) = panel.stacks.as_ref() {
        super::stacks::paint(&mut painter, map.area, maze, sim.prose());
    }
    body = map.rest;

    // The lens's sheet, on the same terms and in the same slot. The two cannot
    // both be present — a maze is in the archive, a ward in the lens, and the
    // player stands in one room — so this is the same claim on the columns made
    // by whichever domain is open. Still after the panel, still refusing rather
    // than clipping.
    let sheet = super::board::split(body, panel.ward.as_ref());
    if let Some(ward) = panel.ward.as_ref() {
        super::board::paint(&mut painter, sheet.area, ward, sheet.presses, sim.prose());
    }
    body = sheet.rest;

    // The sanctum's board, third, on the same terms: one player, one room, so
    // the columns are claimed once whichever domain is open, and never twice.
    let course = super::pylon::split(body, panel.pylon.as_ref());
    if let Some(standing) = panel.pylon.as_ref() {
        super::pylon::paint(&mut painter, course.area, standing, sim.prose());
    }
    body = course.rest;

    // The menagerie's board, fourth, on the same terms.
    let circle = super::circle::split(body, panel.circle.as_ref());
    if let Some(waiting) = panel.circle.as_ref() {
        super::circle::paint(&mut painter, circle.area, waiting, sim.prose());
    }
    body = circle.rest;

    // The bailey's board, fifth, on the same terms.
    let siege = super::rampart::split(body, panel.rampart.as_ref());
    if let Some(fighting) = panel.rampart.as_ref() {
        super::rampart::paint(&mut painter, siege.area, fighting, climb, sim.prose());
    }
    body = siege.rest;

    // The forge's board, sixth and last, on exactly the same terms.
    let binding = super::lattice::split(body, panel.lattice.as_ref());
    if let Some(open) = panel.lattice.as_ref() {
        super::lattice::paint(&mut painter, binding.area, open, sim.prose());
    }
    body = binding.rest;
    // The block — everything the room put on screen — leaves by the edge
    // `panel::split` put it against. Taken after the whole chain, as two slabs
    // (`slabs`).
    let block = slabs(after_strip, body);

    // The tower-wide production meter, skipped where the panel draws it.
    // `instruments.is_empty()` first, since `Sim::working` walks every entity.
    if instruments.is_empty()
        && let Some(working) = sim.working()
    {
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

    // What the player did, not their spells, or a `repeat` buries the pane;
    // the log still has them (§3, `FieldName::Spell`). A closure, since
    // `RecordView` needs a `Clone` iterator over an unbounded stream.
    let drawn = || records.drawn();
    let total = records.drawn_len();

    // How much of the newest end the player has scrolled away from. Clamped to
    // leave at least one record, so paging to the top lands on the oldest line
    // rather than on an empty pane with no way to tell what happened.
    let held = scroll.back().min(total.saturating_sub(1));
    let visible = total - held;
    // Only the tail fits. `iter().skip(n)` is O(1) here and stays `Clone`, which
    // is what `RecordView::draw` needs to measure and then draw.
    //
    // A binary search, since `height` is non-increasing in the skip. The upper
    // bound is `len`, where height is 0, since a record is not one row and
    // `len - rows` may not fit.
    //
    // `take(visible)` before the skip, so scrolling back is the same search over
    // a shorter stream rather than a second way of choosing what to draw.
    let (mut narrowest, mut widest) = (0, visible);
    while narrowest < widest {
        let candidate = narrowest + (widest - narrowest) / 2;
        if view.height(body.cols, drawn().take(visible).skip(candidate)) <= body.rows {
            widest = candidate;
        } else {
            narrowest = candidate + 1;
        }
    }
    // Measured *before* the reveal is applied, so the tail that fits is the one
    // the finished output needs; sizing against half-arrived text would reflow
    // as the rest turned up.
    //
    // No reveal while scrolled back: the typewriter reveals the *newest* output,
    // which is exactly what is off screen, so applying it would hold back the
    // last line of a page of history for a reason the player cannot see.
    if !scroll.is_back()
        && let Some((after, cells)) = reveal.budget(narrowest)
    {
        view = view.revealing(after, cells);
    }
    view.draw(&mut painter, body, drawn().take(visible).skip(narrowest));
    [strip, block[0], block[1]]
}

/// What `after` gave up out of `before`, as up to two disjoint slabs.
///
/// Two, since what a body gives up can be an L, whose bounding box is the whole
/// body. Disjoint by construction, rows then columns, so no corner is crossed
/// twice; each leaves by the edge it sits against.
const fn slabs(before: Rect, after: Rect) -> [(Rect, Toward); 2] {
    if after.is_empty() {
        // The transcript got nothing, so the whole body is the block.
        return [(before, Toward::Up), (Rect::EMPTY, Toward::Right)];
    }
    let rows = if after.row > before.row {
        Rect::new(before.col, before.row, before.cols, after.row - before.row)
    } else if after.bottom() < before.bottom() {
        Rect::new(
            before.col,
            after.bottom(),
            before.cols,
            before.bottom() - after.bottom(),
        )
    } else {
        Rect::EMPTY
    };
    // Over `after`'s rows rather than `before`'s, which is what keeps the two
    // from meeting at the corner.
    let cols = if after.right() < before.right() {
        Rect::new(
            after.right(),
            after.row,
            before.right() - after.right(),
            after.rows,
        )
    } else if after.col > before.col {
        Rect::new(before.col, after.row, after.col - before.col, after.rows)
    } else {
        Rect::EMPTY
    };
    [(rows, Toward::Up), (cols, Toward::Right)]
}

/// A tick count as a meter value, saturating rather than wrapping.
fn to_u32(ticks: u64) -> u32 {
    u32::try_from(ticks).unwrap_or(u32::MAX)
}

// The telemetry pane's readings moved to the rail's foot and `status` (§19).

/// §9's focus mode, as a word.
///
/// Kept after the telemetry pane went, because the border title still carries
/// the readings on any grid too small for the rail (see `carry_readings`): a
/// player at the authoring floor has no rail and still needs to know it ticks.
const fn focus(screen: &Screen) -> &'static str {
    screen.mode.word()
}

/// Re-draw the visible line one lexical run at a time, silently.
///
/// The whole line is lexed and a window drawn ([`Line::window_starts`]), since
/// `parser::lex` classifies by position. Silent, since the `Input` span above
/// already spoke the row (§14).
fn highlight(painter: &mut Painter<'_>, at: Pos, line: &Line, room: u16) {
    let text = line.text();
    let (visible, _) = line.viewport(room);
    let from = line.window_starts(room);
    let upto = from.saturating_add(visible.len());

    for run in orbs_sim::parser::lex(text) {
        // Clip to the window rather than skipping: a run can straddle the edge
        // of a scrolled line, and dropping it would leave one word unhued for
        // no reason a player could see.
        let start = run.start.max(from);
        let end = run.end.min(upto);
        if start >= end {
            continue;
        }
        let Some(word) = text.get(start..end) else {
            continue;
        };
        let Some(before) = text.get(from..start) else {
            continue;
        };
        let Ok(offset) = u16::try_from(before.chars().count()) else {
            continue;
        };
        // The prompt's arbitration, not the editor's. `repeat` and `gathering()`
        // are words a spell runs and the prompt does not, so drawing them as
        // scaffolding would say the orb knew a word it will refuse. Same for the
        // question grammar: a prompt cannot ask a question. See
        // `lexing::at_prompt`.
        let honest = crate::lexing::at_prompt(run.kind);
        let col = at.col.saturating_add(offset);
        let drawn = painter.glyphs(
            Pos::new(col, at.row),
            word,
            crate::lexing::lit(Style::default(), honest),
        );
        painter.lit(Rect::new(col, at.row, drawn, 1), honest);
    }
}

/// Draw the prompt and what is being typed into it.
fn input_line(frame: &mut Frame, area: Rect, line: &Line, prompt: &str, ghost: &str) {
    if area.is_empty() {
        return;
    }
    // Two rows mean double-size glyphs in half the columns, which is why the
    // region lives on the `Frame`. False today; read from the rect, which the
    // Tab listing splits.
    let big = area.rows >= 2;
    let row = area.row;
    let width = if big { area.cols / 2 } else { area.cols };

    let mut painter = frame.painter(Rect::new(area.col, row, width, 1));
    let prompt = painter.glyphs(Pos::new(area.col, row), prompt, Style::DIM);

    let room = width.saturating_sub(prompt);
    let (visible, caret) = line.viewport(room);
    let at = area.col.saturating_add(prompt);
    // One span for the line, tagged `Input` so a reader can filter it; the
    // highlighting below is silent, so the line is spoken once.
    painter.span(
        Pos::new(at, row),
        &Span::new(visible).with_kind(UtteranceKind::Input),
    );
    highlight(&mut painter, Pos::new(at, row), line, room);

    // The suggestion, dim, spoken with its own kind (§19's Frame boundary).
    // After the runs, so nothing over-draws it, whatever `Line::ghost` promises.
    if !ghost.is_empty() {
        let at = area.col.saturating_add(prompt).saturating_add(caret);
        let room = width.saturating_sub(at.saturating_sub(area.col));
        painter.span(
            Pos::new(at, row),
            &Span::new(orbs_render::arriving(ghost, u32::from(room)))
                .with_style(Style::DIM)
                .with_kind(UtteranceKind::Hint),
        );
    }

    frame.set_cursor(Some(Pos::new(
        area.col.saturating_add(prompt).saturating_add(caret),
        row,
    )));
    if big {
        frame.set_magnified(Some(Rect::new(area.col, row, width, 1)));
    }
}

/// The window cannot host the game.
///
/// A real state to render (`Screen::is_hostable`), not a blank. Its sentence
/// is `content/prose.toml`'s (rule 6).
pub fn paint_too_small(frame: &mut Frame, sim: &Sim) {
    let area = frame.area();
    if area.is_empty() {
        return;
    }
    let line = sim.prose().line("window_too_small", &[]);
    let mut painter = frame.painter(area);
    painter.paragraph(area, &Span::new(&line).with_style(Style::DANGER));
    frame.set_cursor(None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::{GridSize, Intensity};

    /// The prompt row, drawn on its own — everything the highlighting needs and
    /// nothing that needs a `Sim`.
    fn typed(text: &str, width: u16) -> Frame {
        let mut frame = Frame::new(GridSize::new(width, 3));
        let area = Rect::new(0, 1, width, 1);
        input_line(&mut frame, area, &Line::typed(text), "> ", "");
        frame
    }

    /// The weight of the cell `word` starts at, given where the row starts.
    fn weight_of(frame: &Frame, at: u16) -> Intensity {
        frame
            .cell(Pos::new(at, 1))
            .expect("a painted cell")
            .style
            .intensity
    }

    /// The prompt hues what is typed into it, on the editor's three weights.
    #[test]
    fn what_is_typed_carries_the_same_weights_a_spell_does() {
        // `> ` is two cells, so the line starts at 2.
        let frame = typed("move the sage to laboratory", 60);
        assert_eq!(weight_of(&frame, 2), Intensity::Normal, "the verb");
        assert_eq!(weight_of(&frame, 2 + 5), Intensity::Dim, "`the` is filler");
        assert_eq!(weight_of(&frame, 2 + 9), Intensity::Normal, "`sage`");
    }

    /// A word only a spell can run is not drawn as scaffolding here.
    ///
    /// `lex` reads `repeat` as a control word wherever it finds one. The prompt
    /// cannot run it, and drawing it bright would say the orb knew a word it is
    /// about to refuse.
    #[test]
    fn the_prompt_does_not_hue_a_word_it_cannot_run() {
        let frame = typed("repeat 3", 60);
        assert_eq!(
            weight_of(&frame, 2),
            Intensity::Normal,
            "`repeat` drew as scaffolding at a prompt that cannot run it",
        );
        // The count is still a count — that much is true in both places.
        assert_eq!(weight_of(&frame, 2 + 7), Intensity::Normal);
    }

    /// The whole line is lexed, not the visible slice of it — see [`highlight`].
    ///
    /// Written against a scrolled comment because it is the only case that
    /// reaches a cell: `Verb` and `Name` both weigh `Normal`, so the tidier
    /// example passes either way and proves nothing.
    #[test]
    fn a_scrolled_line_is_lexed_from_its_real_beginning() {
        let text = "# a note long enough that the prompt has to scroll it sideways";
        let width = 20;
        let frame = typed(text, width);

        let line = Line::typed(text);
        let (visible, _) = line.viewport(width - 2);
        assert!(
            !visible.starts_with('#'),
            "the line did not scroll, so this test proves nothing: {visible:?}",
        );

        // Every visible cell belongs to the comment run, so every one is dim.
        // Lexed from the window instead, `it` would be filler and the words
        // either side of it ordinary names.
        for offset in 0..u16::try_from(visible.chars().count()).expect("a short window") {
            assert_eq!(
                weight_of(&frame, 2 + offset),
                Intensity::Dim,
                "cell {offset} of a scrolled comment was re-read as a fresh line",
            );
        }
    }

    /// The line is spoken once, however many runs it is drawn in.
    ///
    /// §14's stream is rebuilt every frame, so a run-per-word would recite a
    /// half-typed command a word at a time — the `0.3.23` defect, at the surface
    /// a player types at constantly.
    #[test]
    fn the_line_is_one_utterance_however_many_runs_it_takes() {
        let frame = typed("repeat until the stacks is idle", 60);
        let spoken: Vec<&str> = frame
            .speech()
            .utterances()
            .filter(|utterance| utterance.kind == UtteranceKind::Input)
            .map(|utterance| utterance.text)
            .collect();
        assert_eq!(
            spoken,
            ["repeat until the stacks is idle"],
            "the highlighting added an utterance per run",
        );
    }

    /// ...and the ghost survives the runs, because it is drawn after them.
    #[test]
    fn the_ghost_is_not_painted_over_by_the_highlighting() {
        let mut frame = Frame::new(GridSize::new(60, 3));
        input_line(
            &mut frame,
            Rect::new(0, 1, 60, 1),
            &Line::typed("grind"),
            "> ",
            " sage",
        );
        let hint = frame
            .speech()
            .utterances()
            .find(|utterance| utterance.kind == UtteranceKind::Hint);
        assert!(hint.is_some(), "the ghost went missing");
        assert_eq!(
            weight_of(&frame, 2 + 6),
            Intensity::Dim,
            "the ghost lost its weight to a run drawn over it",
        );
    }

    #[test]
    fn an_l_shaped_block_stays_two_slabs() {
        // The defect this is here for wiped the transcript — see [`slabs`]. At
        // `ORBS_GRID=80x45` a room change erased every line on screen.
        let body = Rect::new(0, 0, 40, 20);
        // Four rows off the top and eight columns off the right.
        let left = Rect::new(0, 4, 32, 16);

        let [(rows, up), (cols, right)] = slabs(body, left);
        assert_eq!(rows, Rect::new(0, 0, 40, 4), "the top slab");
        assert_eq!(up, Toward::Up, "a row slab leaves upward");
        assert_eq!(cols, Rect::new(32, 4, 8, 16), "the side slab");
        assert_eq!(right, Toward::Right, "a column slab leaves rightward");

        // Disjoint, which keeps the corner from being crossed twice — on the
        // arriving half a second pass would read the first pass's output.
        assert!(rows.intersection(cols).is_empty(), "the slabs overlap");
        // ...and between them they are exactly what was given up.
        assert_eq!(
            usize::from(rows.cols) * usize::from(rows.rows)
                + usize::from(cols.cols) * usize::from(cols.rows),
            usize::from(body.cols) * usize::from(body.rows)
                - usize::from(left.cols) * usize::from(left.rows),
            "the slabs do not cover what the transcript lost",
        );
    }

    #[test]
    fn a_plain_side_block_is_one_slab() {
        // The grid the game actually draws: the panel runs down the side and
        // nothing is taken off the top, so the row slab is empty and dropped.
        let body = Rect::new(0, 0, 100, 30);
        let [(rows, _), (cols, toward)] = slabs(body, Rect::new(0, 0, 90, 30));
        assert!(rows.is_empty(), "rows were taken that should not have been");
        assert_eq!(cols, Rect::new(90, 0, 10, 30));
        assert_eq!(toward, Toward::Right);
    }
}
