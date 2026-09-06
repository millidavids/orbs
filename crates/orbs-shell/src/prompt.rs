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
    Crossing, DisplayMode, Frame, Painter, Pos, RecordView, Rect, ScreenLayout, ScreenRequest,
    Span, Style, Toward, UtteranceKind,
};
use orbs_sim::Sim;

use super::editor::Editor;
use super::line::Line;
use super::linear::Linear;
use super::reveal::Reveal;
use super::screen::Screen;
use super::tapestry::Tapestry;
use super::transition::PaneTransition;

/// Everything a frame is drawn from, borrowed for the one call.
///
/// A struct rather than ten positional parameters: `paint` grew one for the Tab
/// listing, one for the cached ghost and one for the cached panel, and by then
/// three of its arguments were `&str`-ish and adjacent — the kind of signature
/// where transposing two compiles cleanly. `Frame` and `Linear` stay separate
/// because they are the two things `paint` writes.
///
/// **No longer `Copy`**, and passed by value rather than by reference. The
/// editor's viewport follows its caret, and how many lines fit is a fact only
/// the painter has — so `editing` is the one `&mut` field, and the whole struct
/// moves rather than being duplicated.
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
    /// `&mut` alone in this struct, because the editor's viewport follows the
    /// caret and only the painter knows how tall the pane is — see
    /// `Editor::scroll_to`. Everything else here is read-only and stays so.
    pub editing: Option<&'a mut Editor>,
    /// The weave screen, if the player has it open.
    ///
    /// Read-only, unlike `editing`: this surface has no viewport that follows a
    /// caret, so the painter has nothing to tell it.
    pub weaving: Option<&'a Tapestry>,
    /// Whether the arrow keys are walking the archive's stacks.
    ///
    /// **A flag rather than a borrow**, unlike the two above it, because this
    /// surface holds no state of its own — the maze is the sim's and arrives
    /// through `panel`, and where the arrows are pointed is `Walk`'s. All this
    /// says is who owns the keyboard, which is what decides whether the maze is
    /// drawn over the pane or beside the transcript.
    pub walking: bool,
}

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
pub fn paint(
    frame: &mut Frame,
    linear: &mut Linear,
    passing: &mut super::passing::Passing,
    view: View<'_>,
) {
    // **A wrapper, because `paint_view` has three early returns and a
    // fall-through branch.** A crossing applied at each of them would be four
    // copies of one rule, and the surface that got added without its copy would
    // simply not animate — the failure `focus::Focus` was extracted to stop, one
    // level down. This is the single place a screen is handed over.
    //
    // `passing` is a parameter rather than a [`View`] field, and `&mut`: it is
    // the only thing here that outlives the frame, and `paint_view` has no
    // business with it at all.
    //
    // **Only crossed if the kept screen still describes this grid.** A resize
    // mid-crossing would otherwise blit last frame's cells at this frame's
    // coordinates, which is the defect `orbs-tui`'s shadow buffer already
    // records for itself.
    let crossing = passing.crossing().filter(|_| passing.describes(frame));
    let crossed = paint_view(frame, linear, view);

    let Some(crossing) = crossing else {
        // Nothing moving, so this screen's regions are what the next crossing
        // will have to leave *from* — and only this function knows them.
        passing.remember(crossed);
        return;
    };
    // **The union of both screens' regions.** Sized from the arriving screen
    // alone, a laboratory leaving for a room with no instrument panel would find
    // an empty block and cut rather than leave. `Kept::cell` reads blank outside
    // its own rectangle, which is what makes the wider region draw correctly for
    // whichever screen did not have it.
    let regions = crossed.union(passing.remembered());
    for (area, toward) in regions.moving() {
        frame.cross(area, Crossing { toward, ..crossing }, Some(passing.kept()));
    }
    // **The rail, and only out of boot.** See `Crossed::rail`: every other
    // crossing leaves it standing because it is the one thing on screen that is
    // not about the room you are in. Arriving out of the card it is not on
    // screen yet, so it pushes in from the edge it lives against.
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

/// The regions a crossing moves, each by its own edge.
///
/// **Two, or one.** A room change moves the strip along the top and the block
/// down the side and leaves the transcript between them standing; a surface that
/// takes the whole pane moves the lot. Nothing else is ever crossed — the border,
/// its title, the tower rail and the prompt hold still through all of it.
/// How many regions one screen change can move.
///
/// The strip along the top, and the block's two slabs. See [`Crossed::parts`].
const MOST: usize = 3;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Crossed {
    /// The regions that move, each with the edge it leaves by.
    ///
    /// **Three, and an array rather than named fields.** It was `strip` and
    /// `block`, which assumed the block was one rectangle — and it is two
    /// whenever the layout puts the instrument panel across the top *and* a board
    /// down the side, which `Along::of` does on any pane taller than it is wide.
    /// See [`slabs`], where collapsing those two into one is recorded along with
    /// what it cost.
    ///
    /// Slot 0 is the gauges and the road; slots 1 and 2 are the block's two
    /// slabs. Fixed slots, so [`union`](Self::union) can pair them up across two
    /// screens without having to match rectangles to each other.
    parts: [(Rect, Toward); MOST],
    /// The whole interior, when a surface replaced it. Overrides the two above.
    whole: Rect,
    /// The tower rail, which moves for **one** crossing only.
    ///
    /// Every ordinary crossing leaves it standing on purpose: it is awareness
    /// rather than a view, it is drawn on every branch including the modal ones,
    /// and a player deep in the spell editor still gets told the forge caught
    /// fire. Taking it away for a room change would be taking away the one thing
    /// on screen that is not about the room.
    ///
    /// Arriving out of the boot card is the exception, because there it is not
    /// on screen yet — it pushes in from the right as the tower opens. Carried
    /// here always and used only when `Passing::is_waking`.
    rail: Rect,
}

impl Crossed {
    /// The regions of two screens, together.
    ///
    /// A crossing has to cover whatever *either* side put on screen: the
    /// laboratory's panel has to be able to leave for a forge that has none, and
    /// the forge's lattice has to be able to arrive over a laboratory that had
    /// none. [`Kept::cell`](orbs_render::Kept::cell) reads blank outside its own
    /// rectangle, so the half that never had the region simply has nothing there.
    fn union(self, other: Self) -> Self {
        // Slot by slot — the two screens' slabs mean the same thing in the same
        // place, so the directions match by construction and only the rectangles
        // have to be joined.
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
    /// **`whole` is not a third part, it is the other two's replacement**, and
    /// running it alongside them is a defect with a picture: a maze opening over
    /// a laboratory unioned the session's strip and block into the interior and
    /// drew *three* gathers, each converging on a different centre, so the screen
    /// came apart into three piles instead of one. It contains both by
    /// construction, so it stands in for both.
    ///
    /// Empty rectangles are dropped rather than crossed: a short pane refuses the
    /// road, an empty room draws no panel, and a zero-area region is a divide
    /// waiting to happen in the shapes.
    fn moving(self) -> impl Iterator<Item = (Rect, Toward)> {
        // `Toward` means nothing to the shape a whole-pane change uses — a
        // `Gather` converges from every side at once — so the value paired with
        // it here is the default rather than a claim.
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
    } = view;
    let grid = frame.size();
    // The prompt spends a second row so it can be drawn at double size — see
    // `orbs_render::INPUT_ROWS`, which is a constant now that a resize cannot
    // change how many cells there are to spend.
    let input_rows = orbs_render::INPUT_ROWS;
    // A Tab listing takes a row **from the layout**, so the pane above shrinks by
    // one for as long as it is up. Drawing it at `input.row - 1` instead put it
    // exactly on the session pane's bottom border, because `compute` hands the
    // main tiler everything up to `above_input` — so completing a word erased the
    // border and it came back when the listing went. Reserving the row is what a
    // terminal does anyway: the list pushes the transcript up rather than
    // scribbling on it.
    let listing = u16::from(!offered.is_empty());
    let layout = panes.layout(grid, screen.mode, input_rows.saturating_add(listing));
    let main = layout.main();
    let first = main.first().copied().unwrap_or(Rect::EMPTY);

    // **The rail carries the readings, and the border title carries them only
    // when there is no rail.** This used to ask `panes.panes() == 1`, which was
    // right while a second *pane* held the telemetry: below `DEEP_FOCUS_FLOOR`
    // there was no second pane, so the readings fell back into the session's
    // border title.
    //
    // With `PANES` at 1 that test is permanently true, and the title would have
    // gone to its long form at every size — duplicating, in the one place the
    // player is always looking, the five rows the rail is already showing. The
    // question is now the one it was always standing in for: **is there anywhere
    // else for the readings to be?**
    let carry_readings = layout.rail().is_empty();

    // The same rectangle, not a second pane: the point of §14's stream is that
    // it says the same thing as the cells, and a comparison you make by pressing
    // one key is a comparison you actually make.
    // **The editor takes the session pane while it is open.** Not a pane of its
    // own: §9's pane count is a progression axis, and an editor that added one
    // would hand the player a second pane for free — the thing the multiplex
    // track sells. You go and write, and while you are writing that is what the
    // window shows.
    // **The weave screen takes it on the same terms**, and is checked first
    // only because the two cannot both be open: `weave` is refused from a spell
    // and the prompt is dead while either has the keyboard, so there is no route
    // that opens one over the other. An order is still needed, and the newer
    // surface losing silently would be the harder bug to see.
    // **The rail is drawn whatever the pane is doing**, which is what makes it
    // awareness rather than a view: a player deep in the spell editor still gets
    // told the forge caught fire. Each modal branch below returns early, so this
    // is hoisted above all four rather than repeated inside them — it was four
    // copies of the telemetry call before, and a fifth surface would have made
    // it five.
    let rail = || (layout.rail(), layout.rail_boxes(), layout.rail_foot());

    // **The interior, not the pane.** A crossing never touches the border: the
    // title is the game's one continuously-visible statement of place (§7 — paths
    // are places), and a box coming apart reads as the *machine* breaking rather
    // than the screen changing, which is exactly why §19 cut the tube strike.
    let interior = first.inset(1);
    // The three surfaces that replace the whole pane move all of it —
    // `Focus::takes_the_pane` names the same set, and `Passing` asks it there.
    let whole = Crossed::all(interior, layout.rail());

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

    // **The maze, when the arrows have it.** Third of three modal branches, and
    // it cannot coexist with either above: `wander` needs the prompt to be
    // typed, and the prompt is dead while the editor or the loom has the keys.
    //
    // The map itself is *not* gated on this — `session` draws it beside the
    // transcript whenever a maze is open, which is what makes a spell's solving
    // watchable. What this branch adds is the pane, and only for the player who
    // is walking it themselves.
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
    // The ghost arrives already computed. It stays a pure function of the line,
    // the scene and whether a prompt is open — `shell::input::suggest` owns the
    // one call and names what invalidates it, so there is no second copy able to
    // disagree with the line it trails, and it stops being rebuilt on the ~59
    // frames in 60 where none of its inputs moved.
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
/// **Not a record.** There is deliberately no `scrollback_mut` (§13): the
/// scrollback *is* the log, and a frontend writing into it produces a session
/// `(seed, submissions)` cannot replay — a Tab press is not a submission and
/// never will be. So this is transient Frame content, gone on the next
/// keystroke, and §3's "unlogged output is forbidden" is about the orb's output
/// rather than the shell's own affordances.
fn candidates(frame: &mut Frame, area: Rect, offered: &super::offering::Offered) {
    if offered.is_empty() || area.is_empty() {
        return;
    }
    let row = area.row;
    let mut painter = frame.painter(area);

    // The one the cycle has reached is drawn bright against the rest, because
    // repeated Tab changes the *line* and the list has to say which of them the
    // line now holds. Without it the player is walking a wall of equal-looking
    // words with no idea where they are.
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

    // Spoken once, as a sentence, rather than a span per candidate — a reader
    // hearing eight separate utterances a keypress learns nothing. §19: a visual
    // constraint must not become an informational one, and neither may a visual
    // *affordance* — so the mark a sighted player sees is said aloud too.
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
/// Takes no Bevy resources, because `shell::dump` builds no `App` (§19).
/// Takes no `Sim`: the boot screen draws nothing from the world. It used to, for
/// a prompt that is no longer painted here, and the parameter survived as a
/// `let _ = sim;` that two call sites still threaded an argument through for.
/// Takes no `Screen` either, for the same reason one step on: it wanted a
/// fidelity tier to size the input line, and there is no longer such a thing.
pub fn paint_booting(frame: &mut Frame, stage: crate::stage::Stage, progress: f32, engine: &str) {
    let grid = frame.size();
    let layout = ScreenLayout::compute(&ScreenRequest::single(grid));
    let pane = layout.main().first().copied().unwrap_or(Rect::EMPTY);

    if stage.has_frame() {
        let mut painter = frame.painter(pane);
        // **Untitled.** `Painter::border` announces its title as a heading, and
        // a border drawing itself one cell at a time would speak a pane that is
        // not there yet. The title arrives with the game.
        painter.border_revealed(pane, Style::DIM, stage.frame_progress(progress));
    }

    crate::post::paint(frame, stage, progress, engine);

    // **And then the whole card leaves.** `Stage::Close` draws the finished
    // screen and takes it away by the leaving half of a `Gather` — the same
    // motion `wander` and the editor use, because this is the same event: a
    // surface that had the whole pane giving it back.
    //
    // **Over the pane's interior, and the box stays.** Folding the whole screen
    // reads better in isolation and worse in sequence: the border would go and
    // then come straight back, because the game draws one too. Left standing, it
    // *is* the game's pane — the rail pushes it narrower from the right as the
    // tower opens, which is the frame becoming the main panel rather than being
    // replaced by one.
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

    // **No prompt during boot.** It used to type itself here, caret and all,
    // before the frame drew — an input line offered on a screen where nothing
    // can be typed, since every keyed system is gated on `booted`. The first
    // affordance the game showed was one that did not work.
}

/// The transcript: what was typed and what came back.
///
/// Returns the two regions a crossing moves — the strip along the top and the
/// block down the side — and **not** the transcript between them, which did not
/// change and is what a room change deliberately leaves standing.
///
/// They are returned rather than re-derived because the splits below are the
/// only thing that knows. The panel runs down the *side* at the grid the game
/// draws, so neither region is a function of the pane alone, and the boards below
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
    // **The pane says where you are**, because the pane is the thing that shows
    // a place (§7: paths are places). The prompt below cannot: §9 puts one input
    // line beneath however many panes are open, so it serves all of them and can
    // claim to be standing in none.
    //
    // This is also the only thing on screen that tells a player *which commands
    // will work* — the essences live in `/tower/laboratory`, so that is where
    // `decoct` resolves. See `orbs_sim::tower::rebuild`.
    //
    // The `F4` hint rides along because below `DEEP_FOCUS_FLOOR` there is only
    // one pane and nothing would otherwise say a second exists. A key with no
    // affordance is a key nobody presses.
    let switch = match screen.mode {
        DisplayMode::Deep => "wide",
        DisplayMode::Wide => "deep",
    };
    // **Scrolled-back is a state the pane has to declare**, and it *replaces* the
    // `F4` hint rather than crowding in beside it — the title is already near the
    // width a border gives at the 80x22 floor, and while the player is reading
    // history the way back is the more urgent affordance.
    //
    // New output keeps arriving while they read; it must, or a completion they
    // did not cause would be lost. So without this the newest line on screen
    // simply is not the newest line and nothing says so. It goes in the title
    // because `Painter::border` announces one as a heading, which is what makes a
    // reader hear it too (§14).
    //
    // Plain ASCII, deliberately: an arrow glyph is CP437 0x18 and an em-dash is
    // not in the repertoire at all — one shipped once and drew as `?`.
    // **Reading outranks scrolled-back, which outranks the focus key.** Once the
    // transcript has the keyboard the player needs the way out more than
    // anything else on the row — and the way out is the one thing no other
    // screen has taught them. `unfurl_keys` is authored (rule 6) rather than a
    // literal here, because it is a sentence a player reads.
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
    // and a meter is the only thing on screen that says how much of it is left.
    // §14 names progress bars specifically, and `Painter::progress` had lived in
    // an example since the Frame boundary landed because nothing had a duration
    // to show.
    // §10.1's instrument panel: a **permanent fixture** at the top of the pane,
    // above the transcript, whenever the player is standing in the laboratory.
    // Twice now a piece of laboratory state has been reported as a bug because
    // the only way to see it was to touch it — a bar says it continuously.
    // It follows the shape of the pane: down the side when the pane is wider
    // than it is tall, across the top when it is taller than wide.
    // §11.5's road: the room's own mastery line, one row under the title,
    // **before** the panel and the boards divide what is left — so it sits
    // under the title whichever way the panel runs, and a short pane loses the
    // road rather than the transcript.
    // §11.5's two standings, above the road and above everything: the only two
    // numbers on screen that are about the tower's whole life rather than the
    // room in front of you. Taken first so they sit at the very top, and they
    // yield before the road does — see `gauges::split`.
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
    // **The strip along the top**, which is the two rows above that are about
    // the tower's whole life rather than about the room. It leaves upward, by the
    // edge it sits against. Measured as what `body` lost rather than as the sum
    // of the two splits, because either of them can refuse at a short pane and a
    // sum of refusals is a rectangle nothing drew in.
    //
    // Only ever rows — the gauges and the road span the full width — so the
    // second slab is empty and is dropped.
    let [strip, _] = slabs(interior, body);
    let after_strip = body;

    let instruments = panel.instruments.as_slice();
    let split = super::panel::split(body, instruments);
    super::panel::paint(&mut painter, split, instruments, &panel.domain, bench);
    body = split.rest;

    // §10's map, beside the panel and inboard of it. **Second, deliberately:**
    // whichever split runs first takes its slice from the whole body and hands
    // on the rest, so the second is the one whose refusal can fire — and an
    // instrument row is load-bearing where a map is a convenience. Running this
    // first would also put it outboard of the panel, which is the wrong side.
    let map = super::stacks::split(body, panel.stacks.as_ref());
    if let Some(maze) = panel.stacks.as_ref() {
        super::stacks::paint(&mut painter, map.area, maze, sim.prose());
    }
    body = map.rest;

    // The lens's sheet, on the same terms and in the same slot. **The two cannot
    // both be present**, because a maze lives in the archive and a ward in the
    // lens and the player stands in one room — so this is not a third claim on
    // the columns, it is the same claim made by whichever domain is open. It
    // still splits after the panel, and it still refuses rather than clipping.
    let sheet = super::board::split(body, panel.ward.as_ref());
    if let Some(ward) = panel.ward.as_ref() {
        super::board::paint(&mut painter, sheet.area, ward, sheet.presses, sim.prose());
    }
    body = sheet.rest;

    // The sanctum's board, third and last on exactly the same terms. Three
    // pictures, three rooms, one player — so the columns are claimed once
    // whichever domain is open, and never twice.
    let course = super::pylon::split(body, panel.pylon.as_ref());
    if let Some(standing) = panel.pylon.as_ref() {
        super::pylon::paint(&mut painter, course.area, standing, sim.prose());
    }
    body = course.rest;

    // The menagerie's board, fourth and last, on exactly the same terms. Four
    // pictures, four rooms, one player — so the columns are claimed once
    // whichever domain is open, and never twice.
    let figure = super::chant::split(body, panel.figure.as_ref());
    if let Some(running) = panel.figure.as_ref() {
        super::chant::paint(&mut painter, figure.area, running, sim.prose());
    }
    body = figure.rest;

    // The bailey's board, fifth and last, on exactly the same terms. Five
    // pictures, five rooms, one player — so the columns are claimed once
    // whichever domain is open, and never twice.
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
    // **The block down the side**, which is the panel and whichever one of the
    // seven domain boards is open — everything the room put on screen. It leaves
    // rightward, by the edge `panel::split` already puts it against, so it goes
    // out past the tower rail rather than across the transcript.
    //
    // Taken after the whole chain rather than unioned split by split: only one
    // board can be present at a time and each takes from what the last left, so
    // the difference is the block and a union would be the same rectangle spelled
    // seven ways.
    // **Two slabs, not one.** The panel can run across the top while a board
    // claims columns down the side, and what the transcript gave up is then an L
    // — see `slabs`, and what treating it as one rectangle cost.
    let block = slabs(after_strip, body);

    // The tower-wide production meter stays: it is the *pool*, not an
    // instrument, and it is what says the slot is spent wherever it was spent.
    // Skipped in the laboratory, where the panel already draws that instrument's
    // own bar and a second copy of it would be the same fact twice.
    // `instruments.is_empty()` **first**: `Sim::working` walks every entity in
    // the world with a dynamic component lookup and almost never short-circuits,
    // and as the left operand it ran on every frame in the one place its result
    // is thrown away — the laboratory, where `instruments` is non-empty.
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

    // **What the player did, not what their spells did.** A spell emits exactly
    // what the same commands typed by hand emit — which is right, and which
    // buried the pane: a `repeat` loop pushes several records every few ticks for
    // as long as it runs, and the player's own last line scrolled off in seconds.
    //
    // Nothing is lost and nothing is stored twice. A log is already a view over
    // this one stream (§3, rule 4), so `peruse laboratory.log` reads the very
    // records this declines to draw — see `FieldName::Spell`.
    //
    // A closure rather than a collected `Vec`: `RecordView` measures the tail
    // several times per frame and needs a `Clone` iterator, and the scrollback
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
    // Binary search rather than a walk. `height` is **non-increasing** in the
    // skip — restoring an older record adds to a run's count and can only widen
    // its columns, so it can only cost rows — which makes "the smallest skip
    // that still fits" a monotone predicate. Walking it re-measured the whole
    // tail per step, which is quadratic in a per-frame path; this is about five
    // measurements at the 80×22 floor.
    //
    // **The upper bound is `len`, not `len - rows`.** A record is not one row:
    // `RecordView` opens every `Input` after the first with a blank line, and a
    // wrapped message costs more still. `len - rows` therefore is not a skip
    // that is known to fit, and a binary search whose predicate is false at its
    // own upper bound converges on a value that overflows the pane — dropping
    // the *newest* records off the bottom, out of the linear stream as well as
    // the cells. Skipping everything is height 0, which always fits, at the cost
    // of about one extra probe.
    // **`take(visible)` before the skip**, so scrolling back is the same search
    // over a shorter stream rather than a second way of choosing what to draw.
    // The tail of the first `visible` records *is* the view when scrolled.
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
    // the finished output will need. Sizing the pane against half-arrived text
    // would make it reflow as the rest turned up.
    //
    // **No reveal while scrolled back.** The typewriter reveals the *newest*
    // output, which is precisely what is off screen — applying it would hold back
    // the last line of a page of history for a reason the player cannot see.
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
/// **Two, because what a body gives up is an L and not a rectangle.** This
/// returned one rect — the bounding box of the edges that moved — on the
/// reasoning that an L "cannot be produced because the strip is measured before
/// the boards begin". That was wrong, and the way it was wrong wiped the one
/// thing this whole design exists to keep: `panel::split` puts the instrument
/// panel **across the top** whenever `Along::of` finds the pane taller than it
/// is wide, and `stacks::split` claims columns from the right whatever the shape.
/// Both at once is exactly the L, and the bounding box of a full-width top slab
/// and a full-height right slab is *the entire body* — transcript included. At
/// `ORBS_GRID=80x45` a room change erased it.
///
/// So the slabs are kept apart, and they are **disjoint by construction**: the
/// rows one takes span the full width, and the columns the other takes span only
/// the rows the transcript kept. Overlapping them would double-cross the corner,
/// and on the arriving half the second pass would read the first pass's output as
/// its source.
///
/// Each leaves by the edge it sits against, which is now a fact about *which
/// slab it is* rather than a guess from its shape — the `edge` helper that made
/// that guess is gone with the bounding box that needed it.
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

// **The telemetry pane was here, and the tower rail replaced it** (§19, Phase 2).
//
// It drew nine developer readings — tick, held, scale, cols, rows, logged,
// queued, seed, focus — into fifty-eight columns of the second main pane, and it
// was built as a playability gate rather than as something a player wants. Five
// of those rows moved to the rail's foot (`shell::rail::readings`); the other
// four were already in `status`, which is the full answer where the rail is the
// glance.
//
// What the pane bought is not lost: `tick` still proves the sim runs and `scale`
// against a fixed `grid` is still what a window drag walks through. What it cost
// was §9's second pane, spent on the orb rather than on a domain.

/// §9's focus mode, as a word.
///
/// Kept after the telemetry pane went, because the **border title** still carries
/// the readings on any grid too small for the rail — see `carry_readings`. That
/// fallback is the whole reason the readings are not simply the rail's: a player
/// at the authoring floor has no rail and still needs to know the sim is ticking.
const fn focus(screen: &Screen) -> &'static str {
    screen.mode.word()
}

/// Re-draw the visible line one lexical run at a time, silently.
///
/// # The whole line is lexed and only a window is drawn
///
/// `parser::lex` classifies partly by **position**: a line opening with `#` is
/// one comment run, one opening with a control word is a control line, and
/// otherwise the first word is the verb. Lexing the visible slice of a scrolled
/// line would apply all three to a mid-line fragment. [`Line::window_starts`]
/// exists so it does not.
///
/// The case where that reaches a cell is a **scrolled comment** — dim throughout
/// when lexed whole, and coming apart into ordinary words when lexed from the
/// window. The tidier example, a window starting on a word that names a verb,
/// draws identically either way, because `Verb` and `Name` both weigh `Normal`.
/// Worth knowing before assuming a change here is visible.
///
/// # Silent, because the row has already been spoken
///
/// The `Input` span above draws *and* announces the visible text. These runs
/// re-draw the same glyphs at their own weight with `glyphs`, which draws and
/// says nothing — so §14's stream carries one utterance for the line rather than
/// one per word. A half-typed command recited a word at a time is the `0.3.23`
/// defect, and it is rebuilt every frame.
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
        // **The prompt's arbitration, not the editor's.** `repeat` and
        // `gathering()` are words a spell runs and the prompt does not, so
        // drawing them as scaffolding would say the orb knew a word it will
        // refuse. The same goes for the question grammar: `is` and `idle` are
        // words a *spell* line turns on, and a prompt cannot ask a question.
        // See `lexing::at_prompt`.
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
    // Two rows would mean **double-size glyphs**, not a spare row: at 2× a
    // glyph fills two cells across as well as down, so the text is written into
    // *half* the columns and the frontend draws it into all of them. That
    // halving is why the region lives on the `Frame` rather than in the
    // renderer — it changes what fits.
    //
    // **`INPUT_ROWS` is 1, so this is false today.** Inferred from the rect
    // rather than read from the constant, because the Tab listing splits these
    // rows and the prompt should follow what it is actually given.
    let big = area.rows >= 2;
    let row = area.row;
    let width = if big { area.cols / 2 } else { area.cols };

    let mut painter = frame.painter(Rect::new(area.col, row, width, 1));
    let prompt = painter.glyphs(Pos::new(area.col, row), prompt, Style::DIM);

    let room = width.saturating_sub(prompt);
    let (visible, caret) = line.viewport(room);
    let at = area.col.saturating_add(prompt);
    // One span for the whole line rather than a glyph run: the linear stream
    // should carry what is being typed, tagged `Input` so a reader can filter
    // the partial line out. It is re-spoken every frame — which is correct raw
    // material and wrong to recite verbatim, hence the tag.
    //
    // **Exactly one, and the highlighting below must not add a second.** §14's
    // stream is rebuilt every frame, so a run-per-word would recite a half-typed
    // command one word at a time — the `0.3.23` defect, which is why the runs
    // go on with the silent `glyphs` over the top of what this already drew.
    painter.span(
        Pos::new(at, row),
        &Span::new(visible).with_kind(UtteranceKind::Input),
    );
    highlight(&mut painter, Pos::new(at, row), line, room);

    // The suggestion, after the caret and dim. **Spoken**, with its own kind so
    // a reader can filter it: the half-typed line above is spoken, and §19's
    // Frame-boundary rule is that *"truncation is visual only — a narrow pane is
    // a visual constraint and must not become an informational one."* A ghost
    // drawn and never announced is exactly that asymmetry.
    // **After the runs**, so nothing over-draws it. A ghost is only offered with
    // the caret at the end of the line, so it sits past the last run — but
    // "sits past" is an invariant of `Line::ghost`, and depending on someone
    // else's invariant for a drawing order that costs nothing to get right is
    // how the next change breaks it.
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
/// Its one sentence comes from `content/prose.toml` (rule 6), not from a literal
/// here: a string in Rust is invisible to the `ORBS_CONTENT` watcher, to the
/// width and CP437 lints in `prose.rs`, and to a writer grepping `content/`. The
/// sibling `boot/screen.rs` states the rule this was breaking — *"names and
/// facts, no sentences… there is not a sentence in this file."*
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

    /// A word only a spell can run is **not** drawn as scaffolding here.
    ///
    /// `lex` is lexical, so it reads `repeat` as a control word wherever it
    /// finds one. The prompt cannot run it, and drawing it bright would say the
    /// orb knew a word it is about to refuse.
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

    /// The **whole** line is lexed, not the visible slice of it.
    ///
    /// `lex` classifies partly by position — a line opening with `#` is one
    /// comment run, a line opening with a control word is a control line — so a
    /// scrolled line lexed from its window would classify a mid-line fragment as
    /// a line start.
    ///
    /// **A comment is the case where that reaches a cell**, and it is the only
    /// one: `Verb` and `Name` both weigh `Normal`, so a window that re-read its
    /// first word as the verb would draw identically. A scrolled comment does
    /// not — whole-line it is dim throughout, window-only it comes apart into
    /// ordinary words. Written against that rather than against the tidier
    /// example, because the tidier example passes either way and proves nothing.
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

    /// The line is spoken **once**, however many runs it is drawn in.
    ///
    /// §14's stream is rebuilt every frame, so a run-per-word would recite a
    /// half-typed command one word at a time — the `0.3.23` defect, at the one
    /// surface a player types at constantly.
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
        // **The defect this is here for wiped the transcript.** `panel::split`
        // puts the instrument panel across the top of a pane taller than it is
        // wide while a board still claims columns from the right, so what the
        // transcript gives up is an L — and one rectangle covering an L is the
        // whole body. At `ORBS_GRID=80x45` a room change erased every line of
        // history on screen.
        let body = Rect::new(0, 0, 40, 20);
        // Four rows off the top and eight columns off the right.
        let left = Rect::new(0, 4, 32, 16);

        let [(rows, up), (cols, right)] = slabs(body, left);
        assert_eq!(rows, Rect::new(0, 0, 40, 4), "the top slab");
        assert_eq!(up, Toward::Up, "a row slab leaves upward");
        assert_eq!(cols, Rect::new(32, 4, 8, 16), "the side slab");
        assert_eq!(right, Toward::Right, "a column slab leaves rightward");

        // **Disjoint**, which is what keeps the corner from being crossed twice —
        // on the arriving half a second pass would read the first pass's output.
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
