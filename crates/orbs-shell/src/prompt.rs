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
/// A struct rather than ten positional parameters: `paint` ended up with three
/// adjacent `&str`-ish arguments, where transposing two compiles cleanly.
/// `Frame` and `Linear` stay separate because they are what `paint` writes.
///
/// Not `Copy`: the editor's viewport follows its caret and only the painter
/// knows how many lines fit, so there are `&mut` fields and the struct moves.
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
    /// A flag rather than a borrow, because this surface holds no state of its
    /// own — the maze arrives through `panel` and the arrows are `Walk`'s. All
    /// it says is who owns the keyboard, which decides whether the maze is drawn
    /// over the pane or beside the transcript.
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
}

/// Paint the session into `frame`.
///
/// Two panes, from a real `ScreenLayout`. The game drew one hand-built rectangle
/// until now, so §9's layout system — pane tiling, the sidebar, Deep versus Wide
/// focus — existed only in an example; asking for two panes is what makes the
/// binary exercise it (§15's retroactive gate), and it is the right screen
/// anyway: a session and a dashboard.
///
/// The layout arrives already interpolated (see [`PaneTransition`]), so every
/// rectangle here is wherever that motion has reached this frame.
pub fn paint(
    frame: &mut Frame,
    linear: &mut Linear,
    passing: &mut super::passing::Passing,
    view: View<'_>,
) {
    // A wrapper, because `paint_view` has three early returns and a
    // fall-through: a crossing at each would be four copies of one rule, and the
    // surface added without its copy would simply not animate. This is the
    // single place a screen is handed over. `passing` is a parameter rather than
    // a [`View`] field, and `&mut`, because it is the only thing here that
    // outlives the frame.
    //
    // Only crossed if the kept screen still describes this grid — a resize
    // mid-crossing would blit last frame's cells at this frame's coordinates,
    // the defect `orbs-tui`'s shadow buffer records for itself.
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
    /// An array rather than named fields. It was `strip` and `block`, which
    /// assumed the block was one rectangle — and it is two whenever the layout
    /// puts the panel across the top *and* a board down the side, which
    /// `Along::of` does on any pane taller than wide. See [`slabs`].
    ///
    /// Slot 0 is the gauges and the road, slots 1 and 2 the block's two slabs.
    /// Fixed, so [`union`](Self::union) can pair them across two screens without
    /// matching rectangles to each other.
    parts: [(Rect, Toward); MOST],
    /// The whole interior, when a surface replaced it. Overrides the two above.
    whole: Rect,
    /// The tower rail, which moves for one crossing only.
    ///
    /// Every ordinary crossing leaves it standing: it is awareness rather than a
    /// view, drawn on every branch including the modal ones, so a player deep in
    /// the spell editor still gets told the forge caught fire. Taking it away
    /// for a room change would take away the one thing on screen that is not
    /// about the room.
    ///
    /// Out of the boot card it is not on screen yet, so it pushes in from the
    /// right. Carried here always, used only when `Passing::is_waking`.
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
    /// `whole` is not a third part, it is the other two's replacement, and it
    /// contains both by construction. Running it alongside them drew *three*
    /// gathers each converging on a different centre, so a maze opening over a
    /// laboratory came apart into three piles instead of one.
    ///
    /// Empty rectangles are dropped rather than crossed: a short pane refuses
    /// the road, an empty room draws no panel, and a zero-area region is a
    /// divide waiting to happen in the shapes.
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
        editing,
        weaving,
        walking,
        menuing,
        reading_manual,
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

    // The rail carries the readings; the border title carries them only when
    // there is no rail. This asked `panes.panes() == 1`, which was right while a
    // second pane held the telemetry — but with `PANES` at 1 that is permanently
    // true, and the title would have gone to its long form at every size,
    // duplicating the rail's five rows where the player is always looking. The
    // question is now the one it stood in for: is there anywhere else for the
    // readings to be?
    let carry_readings = layout.rail().is_empty();

    // The same rectangle, not a second pane: §14's stream says the same thing as
    // the cells, and a comparison made by pressing one key is one you make.
    //
    // The editor takes the session pane while it is open, not a pane of its own:
    // §9's pane count is a progression axis, and an editor adding one would hand
    // the player the thing the multiplex track sells. The weave screen takes it
    // on the same terms, checked first only because the two cannot both be open
    // — an order is still needed, and the newer surface losing silently would be
    // the harder bug to see.
    //
    // The rail is drawn whatever the pane is doing, which makes it awareness
    // rather than a view. Each modal branch returns early, so it is hoisted
    // above all four rather than copied into each.
    let rail = || (layout.rail(), layout.rail_boxes(), layout.rail_foot());

    // The interior, not the pane. A crossing never touches the border: the title
    // is the one continuously-visible statement of place (§7 — paths are
    // places), and a box coming apart reads as the *machine* breaking rather
    // than the screen changing, which is why §19 cut the tube strike.
    let interior = first.inset(1);
    // The three surfaces that replace the whole pane move all of it —
    // `Focus::takes_the_pane` names the same set, and `Passing` asks it there.
    let whole = Crossed::all(interior, layout.rail());

    // The menu is checked before the other three: they are surfaces of a tower
    // and this is the way out of one, so if two could be up together the way out
    // is what the player meant — `Focus::of`'s order, for the same reason. No
    // rail beside it: the rail is the tower's telemetry and this screen is
    // stepping out of the tower, where a forge catching fire is not news a
    // player can act on.
    //
    // The manual comes before the menu, because it opens over it. Every other
    // branch is exclusive; this pair is not, and the order is `Focus::of`'s,
    // which is why that chain and this one are kept in step.
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

    // The maze, when the arrows have it. It cannot coexist with either branch
    // above: `wander` needs the prompt typed, and the prompt is dead while the
    // editor or the loom has the keys.
    //
    // The map itself is not gated on this — `session` draws it beside the
    // transcript whenever a maze is open, which makes a spell's solving
    // watchable. This branch adds the pane, for the player walking it.
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
/// Not a record. There is deliberately no `scrollback_mut` (§13): the scrollback
/// *is* the log, and a frontend writing into it makes a session `(seed,
/// submissions)` cannot replay — a Tab press is not a submission. So this is
/// transient Frame content, and §3's "unlogged output is forbidden" is about the
/// orb's output rather than the shell's own affordances.
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
/// Takes no Bevy resources, because `shell::dump` builds no `App` (§19); no
/// `Sim`, because the boot screen draws nothing from the world; and no `Screen`,
/// which wanted a fidelity tier to size the input line and there is no longer
/// such a thing.
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

    // And then the whole card leaves. `Stage::Close` draws the finished screen
    // and takes it away by the leaving half of a `Gather` — the motion `wander`
    // and the editor use, because it is the same event: a surface that had the
    // whole pane giving it back.
    //
    // Over the pane's interior, and the box stays. Folding the whole screen
    // reads better in isolation and worse in sequence — the border would go and
    // come straight back, because the game draws one too. Left standing it *is*
    // the game's pane, narrowed from the right as the rail opens.
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
/// Returns the regions a crossing moves — the strip along the top and the block
/// down the side — and not the transcript between them, which did not change and
/// is what a room change deliberately leaves standing.
///
/// Returned rather than re-derived because the splits below are the only thing
/// that knows: neither region is a function of the pane alone, and the boards
/// take their slices in a fixed order from whatever is left.
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
) -> [(Rect, Toward); MOST] {
    if pane.is_empty() {
        return [(Rect::EMPTY, Toward::Up); MOST];
    }
    let mut painter = frame.painter(pane);
    // The pane says where you are, because the pane is what shows a place (§7:
    // paths are places). The prompt cannot: §9 puts one input line beneath
    // however many panes are open, so it stands in none of them. It is also the
    // only thing on screen saying *which commands will work* — the essences live
    // in `/tower/laboratory`, so that is where `decoct` resolves. See
    // `orbs_sim::tower::rebuild`.
    //
    // The `F4` hint rides along because below `DEEP_FOCUS_FLOOR` there is one
    // pane and nothing else would say a second exists.
    let switch = match screen.mode {
        DisplayMode::Deep => "wide",
        DisplayMode::Wide => "deep",
    };
    // Scrolled-back is a state the pane declares, and it replaces the `F4` hint
    // rather than crowding in beside it: the title is already near the width a
    // border gives at the 80x22 floor, and the way back is more urgent. New
    // output keeps arriving while the player reads — it must, or a completion
    // they did not cause would be lost — so without this the newest line on
    // screen is not the newest and nothing says so. In the title, because
    // `Painter::border` announces one as a heading and a reader hears it (§14).
    //
    // Plain ASCII, deliberately: an arrow glyph is CP437 0x18 and an em-dash is
    // not in the repertoire at all — one shipped once and drew as `?`.
    //
    // Reading outranks scrolled-back, which outranks the focus key: once the
    // transcript has the keyboard the way out is what the player needs, and no
    // other screen has taught it. `unfurl_keys` is authored (rule 6) because it
    // is a sentence a player reads.
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

    // §5.0's economy, made visible: an action occupies its slot for a duration,
    // and a meter is the only thing on screen saying how much is left. §14 names
    // progress bars, and `Painter::progress` lived in an example until something
    // had a duration to show.
    //
    // §10.1's instrument panel is a permanent fixture — twice a piece of
    // laboratory state was reported as a bug because the only way to see it was
    // to touch it. It follows the shape of the pane: down the side when wider
    // than tall, across the top when taller than wide.
    //
    // §11.5's road is the room's mastery line, one row under the title and taken
    // before the panel and boards divide what is left, so a short pane loses the
    // road rather than the transcript. Its two standings go above it — the only
    // numbers about the tower's whole life rather than the room in front of you
    // — and they yield before the road does. See `gauges::split`.
    let gauges = super::gauges::split(body);
    super::gauges::paint(
        &mut painter,
        gauges.area,
        panel.station,
        panel.rankward,
        panel.rank.as_deref(),
        sim.prose(),
    );
    body = gauges.rest;

    let road = super::road::split(body, panel.line.as_ref());
    if let Some(line) = panel.line.as_ref() {
        super::road::paint(&mut painter, road.area, line, sim.prose());
    }
    body = road.rest;
    // The strip along the top: the two rows about the tower's whole life rather
    // than the room. It leaves upward, by the edge it sits against. Measured as
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
        super::rampart::paint(&mut painter, siege.area, fighting, sim.prose());
    }
    body = siege.rest;

    // The forge's board, sixth and last, on exactly the same terms.
    let binding = super::lattice::split(body, panel.lattice.as_ref());
    if let Some(open) = panel.lattice.as_ref() {
        super::lattice::paint(&mut painter, binding.area, open, sim.prose());
    }
    body = binding.rest;
    // The block down the side: the panel and whichever domain board is open —
    // everything the room put on screen. It leaves rightward, by the edge
    // `panel::split` puts it against, so it goes out past the tower rail rather
    // than across the transcript.
    //
    // Taken after the whole chain rather than unioned split by split: only one
    // board is present at a time and each takes from what the last left, so the
    // difference is the block. Two slabs, not one — the panel can run across the
    // top while a board claims columns down the side, and what the transcript
    // gives up is then an L. See `slabs`.
    let block = slabs(after_strip, body);

    // The tower-wide production meter stays: it is the *pool* rather than an
    // instrument, and says the slot is spent wherever it was spent. Skipped in
    // the laboratory, where the panel already draws that bar.
    //
    // `instruments.is_empty()` first: `Sim::working` walks every entity with a
    // dynamic component lookup and rarely short-circuits, so as the left operand
    // it ran every frame in the one place its result is thrown away.
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

    // What the player did, not what their spells did. A spell emits what the
    // same commands typed by hand emit, which is right and which buried the
    // pane: a `repeat` loop pushes records every few ticks, so the player's own
    // last line scrolled off in seconds. Nothing is lost or stored twice — a log
    // is already a view over this one stream (§3, rule 4), so `peruse
    // laboratory.log` reads the records this declines to draw. See
    // `FieldName::Spell`.
    //
    // A closure rather than a collected `Vec`: `RecordView` measures the tail
    // several times a frame and needs a `Clone` iterator, and the scrollback
    // grows without bound.
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
    // Binary search rather than a walk: `height` is non-increasing in the skip —
    // restoring an older record can only widen a run's columns — making "the
    // smallest skip that still fits" monotone. Walking re-measured the whole
    // tail per step, quadratic in a per-frame path; this is about five
    // measurements at the 80×22 floor.
    //
    // The upper bound is `len`, not `len - rows`. A record is not one row —
    // `RecordView` opens every `Input` after the first with a blank line — so
    // `len - rows` is not known to fit, and a search whose predicate is false at
    // its own upper bound converges on a value that overflows the pane, dropping
    // the *newest* records off the bottom. Skipping everything is height 0.
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
/// Two, because what a body gives up is an L and not a rectangle. This returned
/// one rect — the bounding box of the edges that moved — and that wiped the
/// transcript: `panel::split` puts the panel across the top of a pane taller
/// than wide, `stacks::split` claims columns from the right whatever the shape,
/// and the bounding box of those two slabs is *the entire body*. At
/// `ORBS_GRID=80x45` a room change erased it.
///
/// So the slabs are kept apart, and disjoint by construction: one takes rows
/// spanning the full width, the other columns spanning only the rows the
/// transcript kept. Overlapping would double-cross the corner, where the
/// arriving half's second pass would read the first pass's output.
///
/// Each leaves by the edge it sits against, which is a fact about *which slab it
/// is* rather than a guess from its shape.
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

// The telemetry pane was here, and the tower rail replaced it (§19, Phase 2).
//
// It drew nine developer readings into fifty-eight columns of the second main
// pane, built as a playability gate rather than as something a player wants.
// Five moved to the rail's foot (`shell::rail::readings`); the other four were
// already in `status`, the full answer where the rail is the glance.
//
// What it bought is not lost — `tick` still proves the sim runs, and `scale`
// against a fixed `grid` is what a window drag walks through. What it cost was
// §9's second pane, spent on the orb rather than on a domain.

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
/// The whole line is lexed and only a window drawn. `parser::lex` classifies
/// partly by position — a line opening with `#` is one comment run, one opening
/// with a control word is a control line, otherwise the first word is the verb —
/// so lexing the visible slice of a scrolled line would apply all three to a
/// mid-line fragment. [`Line::window_starts`] exists so it does not.
///
/// A scrolled comment is the only case that reaches a cell: dim throughout when
/// lexed whole, coming apart into ordinary words when lexed from the window. A
/// window starting on a verb draws identically either way, because `Verb` and
/// `Name` both weigh `Normal`.
///
/// Silent, because the row has already been spoken: the `Input` span above draws
/// *and* announces the visible text, and these runs re-draw the same glyphs with
/// `glyphs`, which says nothing. §14's stream is rebuilt every frame, so one
/// utterance per word would be the `0.3.23` defect.
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
    // Two rows would mean double-size glyphs, not a spare row: at 2× a glyph
    // fills two cells across as well as down, so the text is written into *half*
    // the columns and the frontend draws it into all of them. That halving is
    // why the region lives on the `Frame` rather than in the renderer.
    //
    // `INPUT_ROWS` is 1, so this is false today. Inferred from the rect rather
    // than the constant, because the Tab listing splits these rows and the
    // prompt should follow what it is actually given.
    let big = area.rows >= 2;
    let row = area.row;
    let width = if big { area.cols / 2 } else { area.cols };

    let mut painter = frame.painter(Rect::new(area.col, row, width, 1));
    let prompt = painter.glyphs(Pos::new(area.col, row), prompt, Style::DIM);

    let room = width.saturating_sub(prompt);
    let (visible, caret) = line.viewport(room);
    let at = area.col.saturating_add(prompt);
    // One span for the whole line rather than a glyph run: the linear stream
    // carries what is being typed, tagged `Input` so a reader can filter the
    // partial line out. It is re-spoken every frame, which is correct raw
    // material and wrong to recite verbatim — hence the tag.
    //
    // Exactly one, and the highlighting below must not add a second: a
    // run-per-word would recite a half-typed command a word at a time, the
    // `0.3.23` defect. Hence the silent `glyphs` over what this already drew.
    painter.span(
        Pos::new(at, row),
        &Span::new(visible).with_kind(UtteranceKind::Input),
    );
    highlight(&mut painter, Pos::new(at, row), line, room);

    // The suggestion, after the caret and dim. Spoken, with its own kind so a
    // reader can filter it: §19's Frame-boundary rule is that *"a narrow pane is
    // a visual constraint and must not become an informational one"*, and a
    // ghost drawn but never announced is that asymmetry.
    //
    // After the runs, so nothing over-draws it. A ghost is only offered with the
    // caret at the end of the line, so it sits past the last run — but that is
    // `Line::ghost`'s invariant, and leaning on someone else's invariant for a
    // drawing order that costs nothing is how the next change breaks it.
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
/// `Screen::is_hostable` documents this as a real state to render rather than a
/// reason to stop drawing: blanking the mesh left the player looking at an empty
/// rectangle with no idea why.
///
/// Its one sentence comes from `content/prose.toml` (rule 6): a string in Rust
/// is invisible to the `ORBS_CONTENT` watcher, to the width and CP437 lints in
/// `prose.rs`, and to a writer grepping `content/`.
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
