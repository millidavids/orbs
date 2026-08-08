//! Fidelity tiers — how many cells a window is worth.
//!
//! DESIGN.md §4 and §9: cell size is an **integer** multiple of the 8×16 bitmap
//! cell, because bitmap fonts must scale by whole pixels to stay crisp.
//! Tier 1 is the largest scale that still meets the 80×22 floor; engaging
//! multiplexing drops one step, roughly doubling the available cells so four
//! panes fit at full function.
//!
//! This lives in `orbs-render` rather than in the Bevy frontend because the
//! *grid* is what layout is computed against, and layout is this crate's job.
//! The frontend supplies window pixels and receives a [`GridSize`]; what it does
//! with the leftover pixels — letterboxing, centring — is its own business.
//!
//! The terminal frontend has no use for any of this: a terminal is whatever size
//! the user made it, so `orbs-tui` reports its grid directly and never consults
//! a tier.

use core::num::NonZeroU8;

use crate::geometry::GridSize;

/// Width of one bitmap cell at scale 1, in pixels.
pub const CELL_WIDTH: u16 = 8;

/// Height of one bitmap cell at scale 1, in pixels.
pub const CELL_HEIGHT: u16 = 16;

/// The declared minimum grid. All layout is authored against this floor.
pub const MIN_GRID: GridSize = GridSize::new(80, 22);

/// An integer scale factor on the 8×16 cell.
///
/// Larger scale means bigger glyphs and fewer cells. Non-zero by construction —
/// a scale of zero would be a cell of no pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fidelity(NonZeroU8);

impl Fidelity {
    /// Scale 1 — one screen pixel per font pixel. The finest tier there is.
    pub const FINEST: Self = Self(NonZeroU8::MIN);

    /// A fidelity at an explicit scale.
    #[must_use]
    pub const fn new(scale: NonZeroU8) -> Self {
        Self(scale)
    }

    /// The scale factor.
    #[must_use]
    pub const fn scale(self) -> u8 {
        self.0.get()
    }

    /// The size of one cell in pixels at this scale.
    #[must_use]
    pub fn cell_pixels(self) -> (u16, u16) {
        let scale = u16::from(self.scale());
        (
            CELL_WIDTH.saturating_mul(scale),
            CELL_HEIGHT.saturating_mul(scale),
        )
    }

    /// The grid a window of `window` pixels yields at this scale.
    ///
    /// Leftover pixels are discarded; the frontend decides whether to letterbox
    /// or centre what remains.
    #[must_use]
    pub fn grid(self, window: (u32, u32)) -> GridSize {
        let (cell_width, cell_height) = self.cell_pixels();
        let cols = window.0 / u32::from(cell_width);
        let rows = window.1 / u32::from(cell_height);
        GridSize::new(
            u16::try_from(cols).unwrap_or(u16::MAX),
            u16::try_from(rows).unwrap_or(u16::MAX),
        )
    }

    /// The grid the default tier aims for, when the window can afford it.
    ///
    /// [`MIN_GRID`] is a **floor**, not a target. Choosing the largest scale that
    /// merely clears it landed every window on 80×22 with the biggest cells that
    /// would fit — a 4K display and a 720p one got the same amount of text, and
    /// the text on the 4K one was enormous.
    ///
    /// This is [`DEEP_FOCUS_FLOOR`](crate::DEEP_FOCUS_FLOOR) deliberately: aiming
    /// at it means a default window can host §9's two-pane Deep focus, so `F4` is
    /// available rather than needing a bigger window first.
    pub const PREFERRED_GRID: GridSize = crate::DEEP_FOCUS_FLOOR;

    /// The default tier for a window: the largest scale that still gives a
    /// comfortable grid, falling back to the largest that clears [`MIN_GRID`].
    ///
    /// `None` means the window is too small to host the game at all — below
    /// roughly 640×352 — which is a real answer the frontend must handle rather
    /// than a case to clamp away.
    #[must_use]
    pub fn tier_one(window: (u32, u32)) -> Option<Self> {
        // Aim high, settle for the floor. A window that cannot reach the
        // preferred grid still gets the biggest text it can carry, which is the
        // small-window case `MIN_GRID` exists for.
        Self::largest_meeting(window, Self::PREFERRED_GRID)
            .or_else(|| Self::largest_meeting(window, MIN_GRID))
    }

    /// The tier a window showing exactly `grid` would be at.
    ///
    /// For a caller that has a grid and **no window** — `ORBS_DUMP`, which takes
    /// its grid from an environment variable. It used to pin
    /// `tier_one((1280, 720))` under a comment claiming the tier was inert, and
    /// that stopped being true when the prompt learned to spend a second row at a
    /// fine tier: every dump then reserved a magnified prompt and halved its own
    /// input viewport, whatever grid was asked for.
    ///
    /// Inverts the real path rather than guessing: build the window that `grid`
    /// implies at each scale and take the **finest** one
    /// [`tier_one`](Self::tier_one) agrees with, so the pair a dump reports is a
    /// pair the game can actually reach.
    ///
    /// **Finest, not coarsest**, and the difference is the whole point of the
    /// function. A grid is reachable at several scales — 100×30 is an 800×480
    /// window at scale 1 and a 6400×3840 one at scale 8 — and keeping the
    /// coarsest reported `tier 8` for a dump anyone would call small, then drew a
    /// **one-row** prompt where the same grid on a plausible window has the
    /// magnified two-row one. That is the layout `ORBS_DUMP` exists to check, so
    /// hiding it is the one answer worse than being wrong loudly. The finest
    /// scale is also the tighter screen, which is the same reason `DEFAULT_GRID`
    /// is §4's floor.
    #[must_use]
    pub fn for_grid(grid: GridSize) -> Option<Self> {
        (NonZeroU8::MIN.get()..=Self::SCALES)
            .filter_map(NonZeroU8::new)
            .map(Self)
            .find(|candidate| {
                let (cell_width, cell_height) = candidate.cell_pixels();
                let window = (
                    u32::from(grid.cols) * u32::from(cell_width),
                    u32::from(grid.rows) * u32::from(cell_height),
                );
                Self::tier_one(window) == Some(*candidate)
            })
    }

    /// The coarsest scale [`for_grid`](Self::for_grid) will consider.
    ///
    /// Scale 8 is a 64×128 pixel cell — an eight-inch glyph on a 4K panel. Past
    /// it the search is describing windows nobody has, and it needs *some* bound
    /// because the mapping from grid to window is one-to-many.
    const SCALES: u8 = 8;

    /// The largest scale whose grid still fits `wanted`.
    fn largest_meeting(window: (u32, u32), wanted: GridSize) -> Option<Self> {
        // The grid shrinks monotonically as scale rises, so the first failure is
        // one past the last candidate.
        let mut best = None;
        let mut scale = NonZeroU8::MIN;
        loop {
            let candidate = Self(scale);
            if !candidate.grid(window).fits(wanted) {
                return best;
            }
            best = Some(candidate);
            let Some(next) = scale.checked_add(1) else {
                return best;
            };
            scale = next;
        }
    }

    /// Rows the input line should occupy at this tier.
    ///
    /// The prompt is the one line a player reads on **every** frame, and a finer
    /// tier shrinks it along with everything else. Spending a second row at the
    /// finest scale holds its height in *pixels* where the tier above puts it,
    /// without a second cell size anywhere in the frame — one grid, one ratio,
    /// and the row count doing the work.
    ///
    /// # It may never grow when the tier gets finer
    ///
    /// **The threshold is scale 1, and it is derived rather than chosen.** A
    /// glyph is `16 × scale` pixels tall and doubling gives `32 × scale`, so
    /// doubling at scale `s` is only compensation — rather than magnification —
    /// while it stays within the tier above it:
    ///
    /// ```text
    /// 32s ≤ 16(s + 1)   ⇔   16s ≤ 16   ⇔   s ≤ 1
    /// ```
    ///
    /// Tiers step by one scale and row-doubling steps by two, and those only
    /// agree on the 2→1 step. The threshold was `scale ≤ 2` and overshot by 4/3
    /// at exactly one place: F4 on a 1440p window drops scale 3 → 2, and the
    /// prompt went **48 px → 64 px — bigger — while every other glyph halved**.
    ///
    /// A constant height is not available. The reachable heights are `16s` or
    /// `32s`: `{16, 32}`, `{32, 64}`, `{48, 96}`, `{64, 128}`, which share no
    /// value. So the property actually held is *monotone*: 32, 32, 48, 64 across
    /// scales 1 to 4, never rising as the tier gets finer.
    ///
    /// The cost is on the record. At scale 2 the prompt is now the transcript's
    /// own size, which is the complaint that moved this threshold to 2 in the
    /// first place — see DESIGN.md §19. Size was a blunt instrument for
    /// *distinguishability*, and it bought a prompt that grew when the screen
    /// got denser.
    #[must_use]
    pub const fn input_rows(self) -> u16 {
        if self.scale() <= 1 { 2 } else { 1 }
    }

    /// One step finer — the tier Deep-focus multiplexing engages (§9).
    ///
    /// `None` at scale 1: there is no smaller whole-pixel step, so Deep focus is
    /// simply unavailable on that window and Wide focus serves instead.
    #[must_use]
    pub fn deep(self) -> Option<Self> {
        NonZeroU8::new(self.scale() - 1).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One row of the tier table in DESIGN.md §9.
    fn assert_tiers(window: (u32, u32), one: (u8, GridSize), two: (u8, GridSize)) {
        let tier_one = Fidelity::tier_one(window).expect("window supports the floor");
        assert_eq!(tier_one.scale(), one.0, "tier 1 scale at {window:?}");
        assert_eq!(tier_one.grid(window), one.1, "tier 1 grid at {window:?}");

        let tier_two = tier_one.deep().expect("a finer step exists");
        assert_eq!(tier_two.scale(), two.0, "tier 2 scale at {window:?}");
        assert_eq!(tier_two.grid(window), two.1, "tier 2 grid at {window:?}");
    }

    /// The table in DESIGN.md §9, verbatim. If this fails, either the formula
    /// drifted or the design's own numbers are wrong — both are worth stopping
    /// for.
    #[test]
    fn the_design_tier_table_reproduces_exactly() {
        assert_tiers(
            (1920, 1080),
            (2, GridSize::new(120, 33)),
            (1, GridSize::new(240, 67)),
        );
        assert_tiers(
            (2560, 1440),
            (3, GridSize::new(106, 30)),
            (2, GridSize::new(160, 45)),
        );
    }

    #[test]
    fn the_finest_tier_has_no_finer_step_and_needs_none() {
        // 1280×720's row of the table. Scale 1 is the smallest whole-pixel cell,
        // so Deep focus engages at the same fidelity — and does not need a finer
        // one, because 160×45 already carries four panes. Tier 2 exists to *buy
        // cells*; a window that has them already needs nothing bought.
        let tier = Fidelity::tier_one((1280, 720)).expect("supported");
        assert_eq!(tier.scale(), 1);
        assert_eq!(tier.grid((1280, 720)), GridSize::new(160, 45));
        assert_eq!(tier.deep(), None);
        assert!(tier.grid((1280, 720)).fits(crate::DEEP_FOCUS_FLOOR));
    }

    #[test]
    fn tier_one_always_meets_the_floor() {
        for width in (640..3840).step_by(97) {
            for height in (352..2160).step_by(89) {
                let window = (width, height);
                if let Some(tier) = Fidelity::tier_one(window) {
                    assert!(
                        tier.grid(window).fits(MIN_GRID),
                        "tier 1 at {window:?} fell below the floor"
                    );
                }
            }
        }
    }

    #[test]
    fn tier_one_is_the_largest_scale_that_stays_comfortable() {
        // The next step coarser must fail the *preferred* grid — otherwise a
        // bigger glyph was available and not taken, which is the direction §9
        // wants erred in.
        let window = (1920, 1080);
        let tier = Fidelity::tier_one(window).expect("supported");
        let coarser = Fidelity::new(NonZeroU8::new(tier.scale() + 1).expect("non-zero"));
        assert!(!coarser.grid(window).fits(Fidelity::PREFERRED_GRID));
    }

    #[test]
    fn a_window_too_small_to_be_comfortable_still_gets_the_floor() {
        // The preferred grid is an aim, not a second floor. A window that cannot
        // reach it must still host the game with the biggest text it can carry.
        let window = (700, 400);
        let tier = Fidelity::tier_one(window).expect("hosts the floor");
        assert!(tier.grid(window).fits(MIN_GRID));
        assert!(!tier.grid(window).fits(Fidelity::PREFERRED_GRID));
    }

    /// The prompt's glyph height at a tier, in physical pixels.
    fn prompt_pixels(tier: Fidelity) -> u16 {
        let (_, cell) = tier.cell_pixels();
        cell * tier.input_rows()
    }

    #[test]
    fn the_prompt_never_grows_as_the_tier_gets_finer() {
        // **The property that replaced a range nobody chose.** This asserted
        // only `(32..=64).contains(&height)`, which every tier satisfies — and a
        // *jump inside that band* is exactly the defect it was meant to catch:
        // F4 on a 1440p window dropped scale 3 → 2 and the prompt went 48 px to
        // 64 px, growing while every other glyph on screen halved.
        //
        // A constant height is not reachable at all (see `input_rows`), so what
        // is held instead is monotonicity: finer tier, never a taller prompt.
        for scale in 1..8u8 {
            // `1..8` is never zero, so the `let ... else { continue }` this used
            // to open with could not take its branch — a guard that reads like
            // one and skips nothing.
            let finer = Fidelity::new(NonZeroU8::new(scale).expect("non-zero"));
            let coarser = Fidelity::new(NonZeroU8::new(scale + 1).expect("non-zero"));
            assert!(
                prompt_pixels(finer) <= prompt_pixels(coarser),
                "{}× has a {}px prompt against {}×'s {}px — it grew",
                finer.scale(),
                prompt_pixels(finer),
                coarser.scale(),
                prompt_pixels(coarser),
            );
        }
    }

    #[test]
    fn the_prompt_is_never_smaller_than_the_text_it_sits_under() {
        // The other half, and the reason `input_rows` exists at all: the one
        // line read on every frame must not be the hardest thing on screen to
        // read.
        //
        // **Asserted against an absolute floor, not against the text.** The old
        // version compared `prompt_pixels(tier)` with `tier.cell_pixels().1` —
        // and `prompt_pixels` *is* that height times `input_rows()`, which is
        // never less than one. `cell * n >= cell` cannot fail, so the test held
        // nothing and would have sat green through `input_rows` returning 1 at
        // every tier, which is the whole defect.
        //
        // What actually has to hold is that the prompt never falls below two
        // native rows. That is the readability floor the mechanism exists for:
        // at the finest tier the transcript's own text *is* one native row, and
        // a prompt matching it would be the smallest thing on screen.
        let floor = 2 * CELL_HEIGHT;
        for scale in 1..8u8 {
            let tier = Fidelity::new(NonZeroU8::new(scale).expect("non-zero"));
            assert!(
                prompt_pixels(tier) >= floor,
                "{scale}×: a {}px prompt, under the {floor}px floor",
                prompt_pixels(tier),
            );
        }
    }

    #[test]
    fn f4_never_enlarges_the_prompt_on_any_real_window() {
        // The bug as a player met it. **Deep focus is the half that was wrong**:
        // it drops a tier below the default, so it is the finest scale a window
        // actually reaches, and 1440p is the window where the jump happened.
        for window in [(1280u32, 720u32), (1920, 1080), (2560, 1440), (3840, 2160)] {
            let out = Fidelity::tier_one(window).expect("supported");
            let Some(deep) = out.deep() else { continue };
            assert!(
                prompt_pixels(deep) <= prompt_pixels(out),
                "{window:?}: F4 takes the prompt from {}px to {}px",
                prompt_pixels(out),
                prompt_pixels(deep),
            );
        }
    }

    #[test]
    fn a_window_below_the_floor_has_no_tier() {
        // 80x22 at scale 1 needs 640x352.
        assert_eq!(Fidelity::tier_one((639, 352)), None);
        assert_eq!(Fidelity::tier_one((640, 351)), None);
        assert!(Fidelity::tier_one((640, 352)).is_some());
    }

    #[test]
    fn deep_focus_is_unavailable_at_the_finest_scale() {
        assert_eq!(Fidelity::FINEST.deep(), None);
    }

    #[test]
    fn deep_focus_yields_strictly_more_cells() {
        let window = (1920, 1080);
        let one = Fidelity::tier_one(window).expect("supported");
        let two = one.deep().expect("a finer step exists");
        assert!(two.grid(window).area() > one.grid(window).area());
    }
}
