//! Screen layout — where the panes, the sidebar, and the input line go.
//!
//! DESIGN.md §9 fixes the shape:
//!
//! - The main window holds every open pane, all fully rendered and fully
//!   functional, up to multiplex capacity.
//! - The rail holds every domain, minimised. Awareness only, not commandable.
//! - One input line, always at the bottom.
//!
//! The rail was a sidebar. §9 wrote the second as *"minimised to a single line"*
//! and this laid it out as full-width rows above the input line; it was never
//! reachable with two panes (§19). The shape it opened into is a thin vertical
//! column on the right, one box per domain, because a row holds a name *or* a
//! state *or* a spell where a box holds all three. Everything else §9 argued for
//! survives: awareness only, never commandable, and it yields before the main
//! window does.
//!
//! Layout is computed from a grid size, not from pixels, so it is identical
//! under both frontends — which is what makes §9's parity rule enforceable:
//! *"pane count and content are identical at every fidelity tier and window
//! size."* Since the grid became [`GRID`](crate::GRID) that is nearly free under
//! Bevy, where a resize changes the size of a cell and not the number of them.
//! It still earns its keep for `orbs-tui` and `ORBS_GRID`.

use crate::geometry::{GridSize, Rect};
use crate::tiling;

/// The most panes the main window ever holds. §9: "four panes is the cap."
pub const MAX_MAIN_PANES: usize = 4;

/// One box per domain you work in — six of them.
///
/// Not §10's seven: spellcraft is a kind of play rather than a room, and the
/// bailey is a room you fight in rather than tend. The grimoire was the seventh
/// until it left `DOMAINS` (§19) and this did not follow, so the rail divided
/// itself into seven slots, drew six, and left a five-row hole; `fits_rail`
/// measured the same phantom and dropped the rail between 39 and 43 rows, where
/// a terminal player sits.
///
/// `orbs-sim`'s `DOMAINS` is the authority and this crate cannot see it (rule 1
/// runs the other way). `orbs-shell` depends on both and pins them together in
/// `rail::tests::the_rail_has_exactly_one_box_per_domain`, so the next domain
/// added or removed is a compile-time conversation.
pub const MAX_PANES: usize = 6;

// The caps again as pane counts. The public constants are `usize` because they
// are array lengths; layout arithmetic is `u16` because grid coordinates are.
// The const assertions keep the two spellings from drifting apart.
const MAIN_CAP: u16 = 4;
const SIDEBAR_CAP: u16 = 6;
const _: () = assert!(MAIN_CAP as usize == MAX_MAIN_PANES);
const _: () = assert!(SIDEBAR_CAP as usize == MAX_PANES);

/// Columns the rail takes off the right of the main window.
///
/// Sixteen, checked rather than eyeballed: inset one each side leaves 14 for
/// content, against `laboratory` (10) plus a mark, `►tending` at 8 and
/// `alembic 22t` at 11. The main window keeps 104 of the fixed 120.
///
/// The longest name shrank to 10 with the sanctum's rename (§19) and the number
/// did not move: sixteen is set by `alembic 22t` and the unbuilt domains, and
/// narrowing the rail would reflow every screen in `scripts/dumps.sh` for one
/// spare column.
pub const RAIL_COLS: u16 = 16;

/// Rows the rail keeps at its foot for the readings that are not per-domain.
///
/// A separator plus `tick`, `held`, `scale`, the grid and the focus mode — the
/// three playability gates §15 named plus the two §9 requires be readable. These
/// were the telemetry pane's and cannot go to `status`, which lives in the sim
/// and may not see a window (rules 1 and 2).
pub const RAIL_FOOT_ROWS: u16 = 6;

/// Rows the foot's content needs: a rule, then `tick`, `held`, `scale`, `grid`
/// and `focus`.
///
/// Separate from [`RAIL_FOOT_ROWS`] so the assertion below is not tautological:
/// it read `assert!(RAIL_FOOT_ROWS >= 6)` against a constant defined as `6`, so
/// lowering the budget to 4 would have passed and silently dropped `focus`.
pub const RAIL_FOOT_CONTENT: u16 = 6;

// The budget against what the foot actually writes. `rail::readings` bails at
// `row >= at.bottom()`, so it would find out by dropping its last row in
// silence.
const _: () = assert!(RAIL_FOOT_ROWS >= RAIL_FOOT_CONTENT);

/// The fewest rows a rail box can occupy and still say anything.
///
/// A name, a state, the detail, the spell, and the rule that closes the box.
/// Below that the rail is dropped rather than squeezed — §9's rule that the
/// minimised half yields and the main window never does — and its readings fall
/// back into the session border's title.
///
/// Five, and it has been three and then four. Each raise was the same defect one
/// row up: a squeezed box keeps its name and its state and silently drops the
/// `►spell` line, the one row telling a player that room is automated. It went
/// unnoticed because the Bevy grid is fixed at 120×45, where each box gets five;
/// a terminal's grid is whatever size the window is, and `scripts/tui.sh start
/// 177 38` is a size a person actually sits at.
pub const MIN_RAIL_BOX: u16 = 5;

/// The fewest columns the main window may be left with before the rail yields.
///
/// A pane narrower than this cannot host the instrument panel *and* a
/// transcript, and §9's main window is *"fully rendered and fully functional"* —
/// so the awareness column is what goes.
const MIN_MAIN_COLS: u16 = 60;

/// Rows a Wide-focus strip occupies.
///
/// First-pass. The Phase 0 worst-case legibility test (§4) is what settles it,
/// and it is a tuning constant, not a design decision.
pub const STRIP_ROWS: u16 = 4;

/// The fewest rows a pane can occupy and still be worth drawing: a top border,
/// one row of content, a bottom border.
pub const MIN_PANE_ROWS: u16 = 3;

/// The smallest grid on which Deep focus is offered by default.
///
/// Provisional. §4 makes establishing this one of Phase 0's deliverables:
/// *"It must also establish the minimum window at which tier 2 is offered at
/// all."* Derived, pending that test, from four panes needing to stay usable —
/// 100×28 leaves roughly 50×13 per pane against the 60×15 the design calls
/// comfortable. The player can override the default either way at any time.
pub const DEEP_FOCUS_FLOOR: GridSize = GridSize::new(100, 28);

/// How extra panes are shown once multiplexing is engaged (§9).
///
/// A setting, not a heuristic. Both modes grant identical capacity, panes,
/// information and synergies; only the rendering differs. If strips showed less,
/// the setting would become a difficulty choice and a player who needs large
/// text would pay for it in capability.
///
/// It no longer changes the size of the text, and §9 is superseded there: deep
/// focus used to raise fidelity a step, and §19 fixed the grid at
/// [`GRID`](crate::GRID), which has room for four panes either way. That leaves
/// a debt — wide focus was §9's large-text affordance and the game has no other,
/// so §19 names a font-scale setting as the replacement it owes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayMode {
    /// Every pane is drawn at full size, tiled in a grid.
    #[default]
    Deep,
    /// The focused pane keeps its size and the rest become compact strips.
    /// Suits small windows, handhelds and TVs.
    Wide,
}

impl DisplayMode {
    /// Both of them, in the order a settings page offers them.
    pub const ALL: [Self; 2] = [Self::Deep, Self::Wide];

    /// The mode `word` names, whole.
    ///
    /// The pair of [`word`](Self::word), so a settings file that says `wide`
    /// reads back as one — the words *are* the format, and there is no second
    /// spelling to drift from.
    #[must_use]
    pub fn named(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.word() == word)
    }

    /// The word for this mode, as the rail and the border title both print it.
    ///
    /// Two private copies lived in one crate — `prompt::focus` and
    /// `rail::focus`, on screen together with nothing binding them, so renaming
    /// a mode would have updated one and left the other saying something else.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Deep => "deep",
            Self::Wide => "wide",
        }
    }

    /// The other mode — what `F4` would give you.
    ///
    /// A property of the mode, not of a `Screen`. It lived on `Screen`, so a
    /// frontend holding only a `DisplayMode` had to build one with a sentinel
    /// `window: (0, 0)` — and that sentinel means *"has the player chosen a mode
    /// yet?"* elsewhere, so a later method consulting `window` would answer for
    /// a screen that does not exist.
    #[must_use]
    pub const fn flipped(self) -> Self {
        match self {
            Self::Deep => Self::Wide,
            Self::Wide => Self::Deep,
        }
    }

    /// The default mode for a grid.
    ///
    /// Window size and font scale are proxies for visual acuity, not
    /// measurements of it, so this only ever picks a *default* — §9 requires the
    /// player be able to override it at any time, including mid-siege.
    #[must_use]
    pub const fn default_for(grid: GridSize) -> Self {
        if grid.fits(DEEP_FOCUS_FLOOR) {
            Self::Deep
        } else {
            Self::Wide
        }
    }
}

/// What to lay out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreenRequest {
    /// The grid to lay out against.
    pub grid: GridSize,
    /// Panes in the main window — the multiplex capacity in use. Clamped to
    /// [`MAX_MAIN_PANES`].
    pub main_panes: u8,
    /// Whether to lay out the rail — one box per domain, down the right.
    ///
    /// A flag rather than a count, because the rail always shows every domain
    /// (§10). One not built yet draws as a dim empty slot — the progression tell
    /// §9's *"two of seven at the start"* implies, which a variable count would
    /// hide.
    pub rail: bool,
    /// How the main window is divided.
    pub mode: DisplayMode,
    /// Rows the input line occupies. At least 1.
    ///
    /// Two means the prompt is drawn at double size into half the columns — see
    /// [`INPUT_ROWS`](crate::INPUT_ROWS), which is what the frontends pass. A
    /// caller that does not care passes 1 and gets a plain single row.
    pub input_rows: u16,
}

impl ScreenRequest {
    /// A request for a single main pane and no rail — the opening state, and
    /// the shape of the boot report (§4).
    ///
    /// Takes the grid rather than assuming [`GRID`](crate::GRID) because
    /// `ORBS_GRID` and the `screens` example lay out against the authoring floor,
    /// which is a grid no window produces.
    #[must_use]
    pub const fn single(grid: GridSize) -> Self {
        Self {
            grid,
            main_panes: 1,
            rail: false,
            mode: DisplayMode::default_for(grid),
            input_rows: crate::INPUT_ROWS,
        }
    }
}

/// Where everything goes, in absolute grid coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenLayout {
    main: [Rect; MAX_MAIN_PANES],
    main_len: usize,
    rail: Rect,
    boxes: [Rect; MAX_PANES],
    boxes_len: usize,
    foot: Rect,
    input: Rect,
}

impl Default for ScreenLayout {
    fn default() -> Self {
        Self {
            main: [Rect::EMPTY; MAX_MAIN_PANES],
            main_len: 0,
            rail: Rect::EMPTY,
            boxes: [Rect::EMPTY; MAX_PANES],
            boxes_len: 0,
            foot: Rect::EMPTY,
            input: Rect::EMPTY,
        }
    }
}

impl ScreenLayout {
    /// Lay out a screen.
    ///
    /// Total, deliberately. A grid below [`crate::MIN_GRID`] produces a smaller
    /// or emptier layout rather than an error: a sub-minimum grid is a normal
    /// runtime state, reached while a window is dragged or a terminal shrunk.
    /// Refusing below the floor is frontend policy — check [`GridSize::fits`]
    /// against [`crate::MIN_GRID`] and show a "window too small" screen.
    ///
    /// The rail is dropped whole, never squeezed: main panes are "fully rendered
    /// and fully functional" (§9) and the rail is awareness only, so
    /// [`rail`](Self::rail) is empty or it is [`RAIL_COLS`] wide.
    #[must_use]
    pub fn compute(request: &ScreenRequest) -> Self {
        let mut layout = Self::default();
        let grid = request.grid;
        if grid.is_empty() {
            return layout;
        }

        // One input line, always at the bottom (§9), inset one cell each side.
        //
        // The gutter is a safe margin: a curved tube distorts most at its
        // corners, where the warp, the vignette and the bezel compound, and the
        // Bevy frontend clipped the first glyph of `orbs:~$` clean off before
        // this existed. Where content may safely go is a layout decision, so it
        // is charged here rather than to the frontend that curves — `orbs-tui`
        // pays a cell it does not need, which beats divergent layouts that §9's
        // parity rule forbids.
        //
        // The rows come from [`INPUT_ROWS`](crate::INPUT_ROWS), which is 1, the
        // same size as the transcript above. Two would mean double-size glyphs
        // into half the columns.
        let input_rows = request.input_rows.max(1).min(grid.rows);
        let above_input = grid.rows - input_rows;
        layout.input = Rect::new(1, above_input, grid.cols.saturating_sub(2), input_rows);

        let main_panes = u16::from(request.main_panes).min(MAIN_CAP);

        // Columns, never rows, as the maze map does: taking rows would shorten
        // the transcript, and columns cost width the panes have to spare at 120.
        let rail_cols = if request.rail && fits_rail(grid.cols, above_input) {
            RAIL_COLS
        } else {
            0
        };

        let main_area = Rect::new(0, 0, grid.cols - rail_cols, above_input);
        layout.main_len = match request.mode {
            DisplayMode::Deep => tiling::deep(main_area, main_panes, &mut layout.main),
            DisplayMode::Wide => tiling::wide(main_area, main_panes, &mut layout.main),
        };

        if rail_cols > 0 {
            layout.lay_rail(Rect::new(main_area.cols, 0, rail_cols, above_input));
        }

        layout
    }

    /// Divide the rail into one box per domain, with the readings beneath.
    ///
    /// Every box is the same height and the remainder goes to the foot, not
    /// `tiling::deep`'s leftovers-to-the-earliest: a rail box is closed by a
    /// drawn rule, so an uneven box puts one separator a row below the other
    /// five and reads as a defect. Slack at the foot is invisible.
    fn lay_rail(&mut self, rail: Rect) {
        self.rail = rail;
        let inside = rail.inset(1);
        if inside.is_empty() {
            return;
        }

        let cap = u16::try_from(MAX_PANES).unwrap_or(SIDEBAR_CAP);
        let least = RAIL_FOOT_ROWS.min(inside.rows);
        let each = (inside.rows - least) / cap;
        let body = each * cap;
        let foot_rows = inside.rows - body;
        self.foot = Rect::new(inside.col, inside.row + body, inside.cols, foot_rows);

        let mut row = inside.row;
        for slot in &mut self.boxes {
            if each == 0 {
                break;
            }
            *slot = Rect::new(inside.col, row, inside.cols, each);
            row += each;
            self.boxes_len += 1;
        }
    }

    /// A layout partway between two others.
    ///
    /// `t` runs 0 → 1 and is the caller's to own: nothing in `orbs-render` knows
    /// what a second is, and a frontend wanting no animation never calls this.
    /// Where a pane arrives *from* is geometry, which is why it is here.
    ///
    /// A transitional layout is not a tiled one. [`compute`](Self::compute)
    /// promises no gaps, no overlap and no zero-area panes; this suspends all
    /// three, because a pane arriving *is* zero-area for a frame and a pane
    /// leaving *is* a gap closing. What holds instead: no pane leaves the span
    /// of its own two endpoints, so nothing escapes a grid both endpoints fitted.
    ///
    /// The sidebar and the input line are taken from `to` untouched — §9 puts
    /// one input line below however many panes are open, so it does not move.
    #[must_use]
    pub fn transition(from: &Self, to: &Self, mode: DisplayMode, t: f32) -> Self {
        let mut layout = *to;
        layout.main_len = crate::tween::panes(from.main(), to.main(), mode, t, &mut layout.main);
        layout
    }

    /// Fully rendered panes, in order.
    #[must_use]
    pub fn main(&self) -> &[Rect] {
        self.main.get(..self.main_len).unwrap_or(&[])
    }

    /// The whole rail, borders included. Empty when it did not fit.
    #[must_use]
    pub const fn rail(&self) -> Rect {
        self.rail
    }

    /// One box per domain, top to bottom, inside the rail's border.
    #[must_use]
    pub fn rail_boxes(&self) -> &[Rect] {
        self.boxes.get(..self.boxes_len).unwrap_or(&[])
    }

    /// The readings beneath the boxes — what the telemetry pane used to carry.
    #[must_use]
    pub const fn rail_foot(&self) -> Rect {
        self.foot
    }

    /// The input line.
    #[must_use]
    pub const fn input(&self) -> Rect {
        self.input
    }
}

/// Whether a grid can host the rail without starving the main window.
///
/// Yes or no rather than a narrower rail: §9's minimised half yields and the
/// fully-functional half never does, so at the 80×22 authoring floor the rail is
/// absent and the session pane is whole.
fn fits_rail(cols: u16, above_input: u16) -> bool {
    let body = above_input.saturating_sub(2).saturating_sub(RAIL_FOOT_ROWS);
    let cap = u16::try_from(MAX_PANES).unwrap_or(SIDEBAR_CAP);
    cols >= RAIL_COLS + MIN_MAIN_COLS && body / cap >= MIN_RAIL_BOX
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewport::MIN_GRID;

    fn request(cols: u16, rows: u16, main: u8, rail: bool, mode: DisplayMode) -> ScreenRequest {
        ScreenRequest {
            grid: GridSize::new(cols, rows),
            main_panes: main,
            rail,
            mode,
            input_rows: 1,
        }
    }

    /// Every cell of the main window is covered exactly once, so panes never
    /// overlap and never leave a hole.
    fn assert_tiles_exactly(layout: &ScreenLayout, area: Rect) {
        let covered: usize = layout.main().iter().map(|r| r.size().area()).sum();
        assert_eq!(
            covered,
            area.size().area(),
            "main panes do not tile {area:?}"
        );

        for (index, a) in layout.main().iter().enumerate() {
            for b in &layout.main()[index + 1..] {
                assert!(a.intersection(*b).is_empty(), "{a:?} overlaps {b:?}");
            }
        }
    }

    #[test]
    fn the_input_line_is_always_the_bottom_row() {
        for panes in 0..=4u8 {
            for rail in [false, true] {
                let req = request(80, 22, panes, rail, DisplayMode::Deep);
                let layout = ScreenLayout::compute(&req);
                assert_eq!(layout.input(), Rect::new(1, 21, 78, 1));
            }
        }
    }

    #[test]
    fn the_input_line_keeps_clear_of_the_bottom_corners() {
        // A curved tube distorts most at the corners, and the Bevy frontend's
        // barrel warp clipped the first glyph of `orbs:~$` clean off — the tube
        // carrying less than the Frame, which rule 2 forbids. The gutter is the
        // layout's answer; see `compute`.
        for cols in [MIN_GRID.cols, 100, 160] {
            let layout = ScreenLayout::compute(&request(cols, 30, 2, true, DisplayMode::Deep));
            let input = layout.input();
            assert!(input.col >= 1, "{cols}: flush against the left edge");
            assert!(input.right() < cols, "{cols}: flush against the right edge",);
        }
    }

    #[test]
    fn a_grid_at_the_floor_still_has_an_input_line() {
        // The gutter must not be able to consume the row it insets.
        let layout = ScreenLayout::compute(&request(
            MIN_GRID.cols,
            MIN_GRID.rows,
            1,
            false,
            DisplayMode::Wide,
        ));
        assert!(!layout.input().is_empty());
    }

    /// DESIGN.md §9: tier 2 at 1080p gives four panes at "roughly 60x15 each".
    #[test]
    fn four_panes_at_tier_two_match_the_design_figure() {
        let layout = ScreenLayout::compute(&request(120, 33, 4, false, DisplayMode::Deep));

        assert_eq!(layout.main().len(), 4);
        for pane in layout.main() {
            assert_eq!(pane.cols, 60);
            // 16 now, where it was 14: the sidebar used to take three rows off
            // the top whether or not anything was in it, and the rail takes
            // columns instead.
            assert!(
                (14..=16).contains(&pane.rows),
                "{pane:?} is not roughly 15 rows"
            );
        }
        assert_tiles_exactly(&layout, Rect::new(0, 0, 120, 32));
    }

    #[test]
    fn panes_tile_the_main_window_without_gaps_or_overlap() {
        for panes in 1..=4u8 {
            for rail in [false, true] {
                let layout =
                    ScreenLayout::compute(&request(120, 33, panes, rail, DisplayMode::Deep));
                let taken = layout.rail().cols;
                assert_tiles_exactly(&layout, Rect::new(0, 0, 120 - taken, 32));
            }
        }
    }

    #[test]
    fn a_short_final_band_spans_the_full_width() {
        let layout = ScreenLayout::compute(&request(120, 33, 3, false, DisplayMode::Deep));
        let main = layout.main();
        assert_eq!(main.len(), 3);
        assert_eq!(main[0].cols, 60);
        assert_eq!(main[1].cols, 60);
        assert_eq!(
            main[2].cols, 120,
            "the lone pane in the last band should span"
        );
    }

    #[test]
    fn the_rail_takes_the_right_hand_columns_and_leaves_the_rows_alone() {
        // A rail costs the main window width and never height, so the transcript
        // is as long with it as without — the maze map's rule.
        let with = ScreenLayout::compute(&request(120, 45, 1, true, DisplayMode::Deep));
        let without = ScreenLayout::compute(&request(120, 45, 1, false, DisplayMode::Deep));

        assert_eq!(with.rail(), Rect::new(104, 0, RAIL_COLS, 44));
        assert_eq!(with.main()[0].rows, without.main()[0].rows, "rows moved");
        assert_eq!(with.main()[0].cols, 104);
        assert_eq!(without.main()[0].cols, 120);
        assert_eq!(with.input(), without.input(), "the prompt moved");
    }

    #[test]
    fn the_rail_holds_one_box_for_every_domain_and_they_tile_it() {
        // Seven, always — a domain that is not built yet draws as a dim slot,
        // which is the progression tell. If this ever returns fewer, the rail is
        // silently hiding a room.
        let layout = ScreenLayout::compute(&request(120, 45, 1, true, DisplayMode::Deep));
        let boxes = layout.rail_boxes();
        assert_eq!(boxes.len(), MAX_PANES);

        let inside = layout.rail().inset(1);
        for slot in boxes {
            assert_eq!(slot.col, inside.col, "a box left the rail");
            assert_eq!(slot.cols, inside.cols);
            assert!(
                slot.rows >= MIN_RAIL_BOX,
                "{slot:?} cannot say three things"
            );
        }
        for (index, a) in boxes.iter().enumerate() {
            for b in &boxes[index + 1..] {
                assert!(a.intersection(*b).is_empty(), "{a:?} overlaps {b:?}");
            }
        }

        let covered: u16 = boxes.iter().map(|slot| slot.rows).sum();
        assert_eq!(
            covered + layout.rail_foot().rows,
            inside.rows,
            "the boxes and the readings do not fill the rail",
        );
    }

    #[test]
    fn the_rail_yields_whole_rather_than_squeezing_the_main_window() {
        // §9's rule, and the reason there is no narrow fallback: at the authoring
        // floor the rail is *absent* and the session pane is whole. A rail drawn
        // three columns wide would be a worse answer than none.
        let floor = ScreenLayout::compute(&request(
            MIN_GRID.cols,
            MIN_GRID.rows,
            1,
            true,
            DisplayMode::Wide,
        ));
        assert!(floor.rail().is_empty(), "the rail squeezed in at 80x22");
        assert!(floor.rail_boxes().is_empty());
        assert_eq!(floor.main()[0].cols, MIN_GRID.cols, "the pane lost columns");

        // ...and never at the cost of a main pane, at any size between.
        for rows in 12..=45u16 {
            let layout = ScreenLayout::compute(&request(120, rows, 4, true, DisplayMode::Deep));
            assert_eq!(layout.main().len(), 4, "a main pane yielded at {rows} rows");
        }
    }

    #[test]
    fn wide_focus_gives_one_full_pane_and_compact_strips() {
        let layout = ScreenLayout::compute(&request(80, 22, 4, false, DisplayMode::Wide));
        let main = layout.main();

        assert_eq!(main.len(), 4);
        assert!(
            main.iter().all(|pane| pane.cols == 80),
            "strips span the width"
        );
        assert_eq!(main[1].rows, STRIP_ROWS);
        assert_eq!(main[2].rows, STRIP_ROWS);
        assert_eq!(main[3].rows, STRIP_ROWS);
        assert!(
            main[0].rows > STRIP_ROWS,
            "the focused pane should dominate"
        );
        assert_tiles_exactly(&layout, Rect::new(0, 0, 80, 21));
    }

    #[test]
    fn both_modes_grant_the_same_pane_count() {
        // §9: parity is mandatory. Only the rendering differs.
        for panes in 1..=4u8 {
            let deep = ScreenLayout::compute(&request(120, 33, panes, true, DisplayMode::Deep));
            let wide = ScreenLayout::compute(&request(120, 33, panes, true, DisplayMode::Wide));
            assert_eq!(deep.main().len(), wide.main().len());
            assert_eq!(deep.rail_boxes().len(), wide.rail_boxes().len());
            assert_eq!(deep.rail(), wide.rail(), "the rail followed the mode");
        }
    }

    #[test]
    fn capacity_is_capped_at_four() {
        let layout = ScreenLayout::compute(&request(120, 33, 7, false, DisplayMode::Deep));
        assert_eq!(layout.main().len(), MAX_MAIN_PANES);
    }

    #[test]
    fn a_degenerate_grid_lays_out_without_panicking() {
        for (cols, rows) in [(0, 0), (1, 1), (4, 2), (80, 1)] {
            let layout = ScreenLayout::compute(&request(cols, rows, 4, true, DisplayMode::Deep));
            assert!(layout.main().len() <= MAX_MAIN_PANES);
            assert!(layout.rail_boxes().len() <= MAX_PANES);
        }
    }

    #[test]
    fn deep_focus_is_the_default_only_on_a_large_enough_grid() {
        assert_eq!(
            DisplayMode::default_for(GridSize::new(120, 33)),
            DisplayMode::Deep
        );
        assert_eq!(DisplayMode::default_for(MIN_GRID), DisplayMode::Wide);
    }
}
