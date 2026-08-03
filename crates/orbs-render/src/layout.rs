//! Screen layout — where the panes, the sidebar, and the input line go.
//!
//! DESIGN.md §9 fixes the shape:
//!
//! - The **main window** holds every open pane, all fully rendered and fully
//!   functional, up to multiplex capacity.
//! - The **sidebar** holds every other unlocked pane, minimised to a single line.
//!   Awareness only, not commandable.
//! - **One input line, always at the bottom.**
//!
//! Layout is computed from a grid size, not from pixels, so it is identical
//! under both frontends. That is what makes §9's parity rule enforceable rather
//! than aspirational: *"pane count and content are identical at every fidelity
//! tier and window size."*

use crate::geometry::{GridSize, Rect};
use crate::tiling;

/// The most panes the main window ever holds. §9: "four panes is the cap."
pub const MAX_MAIN_PANES: usize = 4;

/// One pane per domain, seven domains (§10).
pub const MAX_PANES: usize = 7;

// The caps again as pane counts. The public constants are `usize` because they
// are array lengths; layout arithmetic is `u16` because grid coordinates are.
// The const assertions keep the two spellings from drifting apart.
const MAIN_CAP: u16 = 4;
const SIDEBAR_CAP: u16 = 7;
const _: () = assert!(MAIN_CAP as usize == MAX_MAIN_PANES);
const _: () = assert!(SIDEBAR_CAP as usize == MAX_PANES);

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
/// **Provisional.** §4 makes establishing this one of Phase 0's deliverables:
/// *"It must also establish the minimum window at which tier 2 is offered at
/// all."* Derived, pending that test, from four panes needing to stay usable —
/// 100×28 leaves roughly 50×13 per pane against the 60×15 the design calls
/// comfortable. The player can override the default either way at any time.
pub const DEEP_FOCUS_FLOOR: GridSize = GridSize::new(100, 28);

/// How extra panes are shown once multiplexing is engaged (§9).
///
/// A **setting, not a heuristic**. Both modes grant identical capacity, panes,
/// information, and synergies; only the rendering differs. If strips ever showed
/// less, the setting would become a difficulty choice and a player who needs
/// large text would be paying for it in capability.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayMode {
    /// Fidelity rises one step; every pane is drawn at full size in a grid.
    #[default]
    Deep,
    /// Fidelity stays put; the focused pane keeps its size and the rest become
    /// compact strips. Suits large text, small windows, handhelds, TVs.
    Wide,
}

impl DisplayMode {
    /// The default mode for a grid.
    ///
    /// Window size and font scale are proxies for visual acuity, not
    /// measurements of it, so this only ever picks a *default* — §9 requires the
    /// player be able to override it at any time, including mid-siege.
    #[must_use]
    pub fn default_for(grid: GridSize) -> Self {
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
    /// Unlocked panes minimised to the sidebar. Clamped to [`MAX_PANES`].
    pub sidebar_panes: u8,
    /// How the main window is divided.
    pub mode: DisplayMode,
}

impl ScreenRequest {
    /// A request for a single main pane and no sidebar — the opening state, and
    /// the shape of the boot report (§4).
    #[must_use]
    pub fn single(grid: GridSize) -> Self {
        Self {
            grid,
            main_panes: 1,
            sidebar_panes: 0,
            mode: DisplayMode::default_for(grid),
        }
    }
}

/// Where everything goes, in absolute grid coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenLayout {
    main: [Rect; MAX_MAIN_PANES],
    main_len: usize,
    sidebar: [Rect; MAX_PANES],
    sidebar_len: usize,
    input: Rect,
}

impl Default for ScreenLayout {
    fn default() -> Self {
        Self {
            main: [Rect::EMPTY; MAX_MAIN_PANES],
            main_len: 0,
            sidebar: [Rect::EMPTY; MAX_PANES],
            sidebar_len: 0,
            input: Rect::EMPTY,
        }
    }
}

impl ScreenLayout {
    /// Lay out a screen.
    ///
    /// Total, deliberately. A grid below [`crate::MIN_GRID`] produces a smaller
    /// or emptier layout rather than an error, because a sub-minimum grid is a
    /// normal runtime state and not a bug: the Bevy build passes through it
    /// while a window is being dragged, and a terminal user can shrink
    /// `orbs-tui` to any size at all. Refusing to run below the floor is a
    /// frontend policy — check [`GridSize::fits`] against [`crate::MIN_GRID`]
    /// and show a "window too small" screen — not something to enforce with a
    /// panic down here.
    ///
    /// Sidebar entries that do not fit are **dropped, not squeezed** — main
    /// panes are "fully rendered and fully functional" (§9) and the sidebar is
    /// awareness only, so the sidebar is what yields. The caller can compare
    /// `sidebar().len()` against what it asked for.
    #[must_use]
    pub fn compute(request: &ScreenRequest) -> Self {
        let mut layout = Self::default();
        let grid = request.grid;
        if grid.is_empty() {
            return layout;
        }

        // One input line, always at the bottom (§9).
        let above_input = grid.rows - 1;
        layout.input = Rect::new(0, above_input, grid.cols, 1);

        let main_panes = u16::from(request.main_panes).min(MAIN_CAP);

        // Reserve what the main window needs before the sidebar takes its share.
        let reserved = match request.mode {
            DisplayMode::Deep => MIN_PANE_ROWS.saturating_mul(tiling::deep_bands(main_panes)),
            DisplayMode::Wide => MIN_PANE_ROWS.saturating_add(main_panes.saturating_sub(1)),
        };

        let requested_sidebar = u16::from(request.sidebar_panes).min(SIDEBAR_CAP);
        let sidebar_rows = requested_sidebar.min(above_input.saturating_sub(reserved));

        // The sidebar sits directly above the input line, one row per pane.
        let sidebar_top = above_input - sidebar_rows;
        for index in 0..sidebar_rows {
            let Some(slot) = layout.sidebar.get_mut(usize::from(index)) else {
                break;
            };
            *slot = Rect::new(0, sidebar_top + index, grid.cols, 1);
            layout.sidebar_len += 1;
        }

        let main_area = Rect::new(0, 0, grid.cols, sidebar_top);
        layout.main_len = match request.mode {
            DisplayMode::Deep => tiling::deep(main_area, main_panes, &mut layout.main),
            DisplayMode::Wide => tiling::wide(main_area, main_panes, &mut layout.main),
        };

        layout
    }

    /// Fully rendered panes, in order.
    #[must_use]
    pub fn main(&self) -> &[Rect] {
        self.main.get(..self.main_len).unwrap_or(&[])
    }

    /// Minimised panes, top to bottom, one row each.
    #[must_use]
    pub fn sidebar(&self) -> &[Rect] {
        self.sidebar.get(..self.sidebar_len).unwrap_or(&[])
    }

    /// The input line.
    #[must_use]
    pub const fn input(&self) -> Rect {
        self.input
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fidelity::MIN_GRID;

    fn request(cols: u16, rows: u16, main: u8, side: u8, mode: DisplayMode) -> ScreenRequest {
        ScreenRequest {
            grid: GridSize::new(cols, rows),
            main_panes: main,
            sidebar_panes: side,
            mode,
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
            for side in 0..=6u8 {
                let req = request(80, 22, panes, side, DisplayMode::Deep);
                let layout = ScreenLayout::compute(&req);
                assert_eq!(layout.input(), Rect::new(0, 21, 80, 1));
            }
        }
    }

    /// DESIGN.md §9: tier 2 at 1080p gives four panes at "roughly 60x15 each".
    #[test]
    fn four_panes_at_tier_two_match_the_design_figure() {
        let layout = ScreenLayout::compute(&request(120, 33, 4, 3, DisplayMode::Deep));

        assert_eq!(layout.main().len(), 4);
        for pane in layout.main() {
            assert_eq!(pane.cols, 60);
            assert!(
                (14..=15).contains(&pane.rows),
                "{pane:?} is not roughly 15 rows"
            );
        }
        assert_tiles_exactly(&layout, Rect::new(0, 0, 120, 29));
    }

    #[test]
    fn panes_tile_the_main_window_without_gaps_or_overlap() {
        for panes in 1..=4u8 {
            for side in 0..=3u8 {
                let layout =
                    ScreenLayout::compute(&request(120, 33, panes, side, DisplayMode::Deep));
                let sidebar_rows = u16::try_from(layout.sidebar().len()).expect("small");
                assert_tiles_exactly(&layout, Rect::new(0, 0, 120, 32 - sidebar_rows));
            }
        }
    }

    #[test]
    fn a_short_final_band_spans_the_full_width() {
        let layout = ScreenLayout::compute(&request(120, 33, 3, 0, DisplayMode::Deep));
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
    fn the_sidebar_sits_directly_above_the_input_line() {
        let layout = ScreenLayout::compute(&request(80, 22, 2, 3, DisplayMode::Deep));
        let sidebar = layout.sidebar();
        assert_eq!(sidebar.len(), 3);
        assert_eq!(sidebar[0], Rect::new(0, 18, 80, 1));
        assert_eq!(sidebar[1], Rect::new(0, 19, 80, 1));
        assert_eq!(sidebar[2], Rect::new(0, 20, 80, 1));
        assert_eq!(layout.input().row, 21);
    }

    /// All seven domains at once — four multiplexed, three minimised — on the
    /// declared 80×22 floor. §9 says every unlocked pane is visible somewhere;
    /// if that did not fit at the floor, the floor would be wrong.
    #[test]
    fn the_full_pane_complement_fits_at_the_declared_floor() {
        let sidebar = u8::try_from(MAX_PANES - MAX_MAIN_PANES).expect("small");
        let layout = ScreenLayout::compute(&request(
            MIN_GRID.cols,
            MIN_GRID.rows,
            u8::try_from(MAX_MAIN_PANES).expect("small"),
            sidebar,
            DisplayMode::Deep,
        ));

        assert_eq!(layout.main().len(), MAX_MAIN_PANES);
        assert_eq!(layout.sidebar().len(), usize::from(sidebar));
        for pane in layout.main() {
            assert!(pane.rows >= MIN_PANE_ROWS, "{pane:?} is unusably short");
        }
    }

    #[test]
    fn the_sidebar_yields_before_the_main_window_does() {
        // Well below the floor — a terminal the user shrank. Six minimised panes
        // no longer fit above four full ones.
        let layout = ScreenLayout::compute(&request(80, 12, 4, 6, DisplayMode::Deep));

        assert_eq!(layout.main().len(), 4, "main panes must survive");
        assert!(
            layout.sidebar().len() < 6,
            "the sidebar should have yielded"
        );
        for pane in layout.main() {
            assert!(pane.rows >= MIN_PANE_ROWS, "{pane:?} is unusably short");
        }
    }

    #[test]
    fn wide_focus_gives_one_full_pane_and_compact_strips() {
        let layout = ScreenLayout::compute(&request(80, 22, 4, 0, DisplayMode::Wide));
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
            let deep = ScreenLayout::compute(&request(120, 33, panes, 2, DisplayMode::Deep));
            let wide = ScreenLayout::compute(&request(120, 33, panes, 2, DisplayMode::Wide));
            assert_eq!(deep.main().len(), wide.main().len());
            assert_eq!(deep.sidebar().len(), wide.sidebar().len());
        }
    }

    #[test]
    fn capacity_is_capped_at_four() {
        let layout = ScreenLayout::compute(&request(120, 33, 7, 0, DisplayMode::Deep));
        assert_eq!(layout.main().len(), MAX_MAIN_PANES);
    }

    #[test]
    fn a_degenerate_grid_lays_out_without_panicking() {
        for (cols, rows) in [(0, 0), (1, 1), (4, 2), (80, 1)] {
            let layout = ScreenLayout::compute(&request(cols, rows, 4, 6, DisplayMode::Deep));
            assert!(layout.main().len() <= MAX_MAIN_PANES);
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
