//! One screen leaving and the next arriving, in glyphs.
//!
//! `attend forge` replaces the laboratory's instruments with the forge's lattice
//! between one frame and the next. §19 recorded that defect once already, for
//! pane *geometry* — *"a pane appearing between one frame and the next reads as
//! a glitch"* — and [`crate::ScreenLayout::transition`] is the answer it got.
//! This is the same answer for pane *content*.
//!
//! # What a crossing is, and what it is not
//!
//! It is the **glyphs** moving. It is not a flash, not a tube effect, not a
//! stinger. §19 cut the boot strike twice — the sweeping band went first *"for
//! reading as a fault rather than as a tube coming on"* — and the consequence is
//! recorded there as *"no part of this game flashes any more"*. A transition that
//! drew attention to the screen rather than to the content would be that mistake
//! made again.
//!
//! # Nothing here knows what a second is
//!
//! [`Crossing`] is the counterpart of [`Burn`](crate::Burn),
//! [`Grind`](crate::Grind), [`Steep`](crate::Steep) and [`Stir`](crate::Stir):
//! plain data with one `f32`, handed over by whichever frontend owns the clock.
//! The division is `tween`'s and `paint`'s and `layout`'s, restated.
//!
//! # The two endpoints are identities
//!
//! `progress` of `0.0` draws the kept cells exactly and `1.0` draws the live
//! frame exactly. Everything else rests on that: it is what makes a settled
//! crossing indistinguishable from no crossing, which is what keeps every
//! `scripts/dumps.sh` baseline still.
//!
//! # The safety property, which is not the fire's
//!
//! `pulse` — the clock the instruments animate on — takes a **recorded
//! exemption** to the 3–30 Hz
//! photosensitive band, conditional on three things — nothing turning over
//! together, each step being small, and the element staying small. A crossing
//! meets none of those the same way, so it does not lean on that argument. It
//! makes its own — **two of them, one per family**, because the shapes here do
//! two different things to a cell:
//!
//! - The shapes that **erode** — [`Passage::Wipe`] and [`Passage::Furl`] — hold
//!   it per cell. A cell's coverage moves one way and reverses once: on the way
//!   out a glyph decays `▓ → ▒ → ░ → blank` and never brightens, and on the way
//!   in it builds back. §19: *"a flash is a **pair** of opposing changes"*, so a
//!   whole crossing is exactly one flash.
//! - The shape that **moves** — [`Passage::Gather`] — cannot, and neither can any
//!   motion: a glyph travelling past a cell lights it and darkens it again, which
//!   the per-cell rule counts as a flash and which is plainly not one. It holds a
//!   *field* property instead: everything it can light is inside an [`envelope`]
//!   that shrinks monotonically to a point and grows back, so the lit **area**
//!   never oscillates however individual cells behave.
//!
//! The second is the honest reading of the band, which is about flashes covering
//! a substantial share of the visual field — and it is what `pulse` says from the
//! other side: *"whole-field modulation is the hazard; motion is not."* §19
//! records the per-cell rule being applied to all three at first, and why that
//! was too strict rather than merely inconvenient.
//!
//! How often a crossing may *start* is the remaining question either way. That is
//! the shell's, and it answers it with a floor of one world tick.

use crate::cell::Cell;
use crate::geometry::{Pos, Rect};
use crate::tween;

/// How many cells trail the seam, decaying.
///
/// Three, because that is how many rungs the shade ramp has. A wake of one is a
/// hard edge that reads as a rectangle being drawn over the screen; a wake as
/// long as the ramp reads as the content coming apart, which is the thing being
/// depicted.
const WAKE: u16 = 3;

/// The shade ramp, most covered first.
///
/// CP437's three shading glyphs, and the reason a crossing reads as *ASCII*
/// rather than as a black rectangle sliding about. Every one of them is in the
/// repertoire by construction — see [`cp437`] — and
/// `every_glyph_a_crossing_writes_is_drawable` holds it.
const RAMP: [char; 3] = ['▓', '▒', '░'];

/// Where the middle of a crossing falls: the screen has left, the next has not
/// arrived.
const MIDPOINT: f32 = 0.5;

/// The shape a screen leaves and arrives by.
///
/// **The motion names the kind of change**, which is §19's *"tempo is the
/// organising principle"* applied to screens: a room, a tool, a re-reading of the
/// same screen. Every one has a caller, because §15's rule is that an API with no
/// callers is unshaped.
///
/// **None of them translates a glyph**, and that is one decision rather than
/// three. A cell showing a procession of source glyphs as text slides past
/// flickers between covered and blank, and the safety argument this whole module
/// rests on is that a cell's coverage reverses *once*. `Furl`'s implementation
/// carries the long form of that argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Passage {
    /// A seam crosses the region; ahead of it the old screen, behind it nothing.
    ///
    /// A room change. The most legible of the three, which is why it has the
    /// crossing a player meets in the first ten minutes: it is the one that must
    /// never read as a fault.
    #[default]
    Wipe,
    /// The same seam, with the rows staggered so the region peels.
    ///
    /// `F5`'s linear mirror. The same content re-read rather than different
    /// content, and a motion that comes apart in layers says so.
    Furl,
    /// Every glyph flies to the middle and out again from it.
    ///
    /// A full-pane tool opening or shutting — the maze, the editor, the weave
    /// screen. Going *into* something rather than beside it, and the one shape
    /// that genuinely **moves** a glyph rather than eroding it.
    ///
    /// It is a scale about the region's centre: on the way out the screen
    /// shrinks into a point, on the way in the next one grows out of that point.
    /// Read per destination it is the inverse — a cell shows whatever source
    /// position it has *zoomed away from* — which is what keeps it a pure
    /// function like the other two rather than a pass that has to sort
    /// overlapping writes.
    Gather,
}

impl Passage {
    /// The erosion this shape is, or `None` if it moves glyphs instead.
    ///
    /// **What keeps [`cell_at`] from having to refuse a `Gather`.** It used to
    /// take a whole [`Passage`] and `unreachable!` on the one variant it cannot
    /// answer for — a panic on a *rendering* path, which is the trade this crate
    /// declines everywhere else: [`Cell::new`] substitutes a glyph it cannot draw
    /// precisely because *"a visibly wrong character in one cell is a better
    /// failure than a panic mid-siege"*. Splitting the two families out makes the
    /// call impossible to write rather than merely wrong to write.
    pub(crate) const fn erosion(self) -> Option<Erosion> {
        match self {
            Self::Wipe => Some(Erosion::Wipe),
            Self::Furl => Some(Erosion::Furl),
            Self::Gather => None,
        }
    }
}

/// A shape that wears a screen away in place, rather than moving it.
///
/// The two families answer different questions — an erosion asks *"how far
/// through is this cell"*, a convergence asks *"where does this cell read
/// from"* — and this is the half [`cell_at`] can answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Erosion {
    /// See [`Passage::Wipe`].
    Wipe,
    /// See [`Passage::Furl`].
    Furl,
}

/// Which way the motion runs.
///
/// A plain `bool` would do the arithmetic and would read as `true` at the call
/// site, which is what [`crate::ScreenRequest`]'s named fields exist to avoid.
///
/// **[`Passage::Gather`] ignores it**, because both of its edges move at once and
/// there is no direction left to name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Toward {
    /// Out by the right edge, and back in from it.
    ///
    /// The instrument panel and the domain boards, which already sit against
    /// that edge with the transcript to their left — so they leave by the side
    /// they are on rather than crossing the text on the way out.
    #[default]
    Right,
    /// Out by the top edge, and back in from it.
    ///
    /// The gauges and the road, which are the strip along the top.
    Up,
}

impl Toward {
    /// How far a seam must travel to clear the region.
    const fn span(self, area: Rect) -> u16 {
        match self {
            Self::Right => area.cols,
            Self::Up => area.rows,
        }
    }
}

/// A crossing in flight.
#[derive(Debug, Clone, Copy)]
pub struct Crossing {
    /// The shape.
    pub passage: Passage,
    /// Which way it runs.
    pub toward: Toward,
    /// How far through the whole crossing, `0.0..=1.0`. Out for the first half,
    /// in for the second.
    pub progress: f32,
}

impl Crossing {
    /// Whether the old screen is still leaving.
    #[must_use]
    pub fn is_leaving(self) -> bool {
        self.sane() < MIDPOINT
    }

    /// How far through this *half*, `0.0..=1.0`.
    fn local(self) -> f32 {
        let progress = self.sane();
        if progress < MIDPOINT {
            progress / MIDPOINT
        } else {
            (progress - MIDPOINT) / MIDPOINT
        }
    }

    /// How large the screen is drawn, for [`Passage::Gather`]: `1.0` is its own
    /// size and `0.0` is a point at the region's centre.
    ///
    /// Runs down through the leaving half and back up through the arriving one,
    /// so the same number describes a screen collapsing into the middle and the
    /// next one growing out of it.
    fn scale(self) -> f32 {
        if self.is_leaving() {
            1.0 - self.local()
        } else {
            self.local()
        }
    }

    /// The progress, in range and finite.
    ///
    /// **Both readers go through this, and that is the point.** A `NaN` compares
    /// false against everything, so [`is_leaving`](Self::is_leaving) testing the
    /// raw field answered *arriving* while [`local`](Self::local) answered zero —
    /// which together is "the new screen, none of it here yet", an empty pane
    /// that no state in the game can reach. `dump.rs` records the same class of
    /// defect for `ORBS_FIRE_PHASE`, where a bad value drew a screen the game
    /// could not produce, and calls it the one thing that tool must never do.
    ///
    /// Zero, so a bad number leaves the screen the game already had.
    const fn sane(self) -> f32 {
        if self.progress.is_finite() {
            self.progress.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// The cells of a region, kept — what a crossing departs from.
///
/// Held by the shell across frames and refilled by [`Frame::keep`] while nothing
/// is moving.
///
/// **The rect travels with the cells**, and it is what lets a caller ask *"is
/// this still a picture of the screen we have?"* — the shell keeps a whole grid
/// and compares, so a window resized mid-crossing invalidates it rather than
/// blitting last frame's cells at this frame's coordinates. `orbs-tui`'s shadow
/// buffer records the same hazard for itself, in the same words.
///
/// [`Frame::keep`]: crate::Frame::keep
#[derive(Debug, Default, Clone)]
pub struct Kept {
    area: Rect,
    cells: Vec<Cell>,
}

impl Kept {
    /// The region these cells were taken from.
    #[must_use]
    pub const fn area(&self) -> Rect {
        self.area
    }

    /// Whether anything is kept at all.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// The cell kept at `at`, or [`Cell::BLANK`] outside the kept region.
    ///
    /// Blank rather than `None`, and that is what makes a crossing between two
    /// differently-shaped regions work: the laboratory's tall panel leaves while
    /// the archive's short map arrives, the crossing runs over the union of the
    /// two, and the rows the old screen never had read as empty — which is what
    /// they were.
    #[must_use]
    pub fn cell(&self, at: Pos) -> Cell {
        let Some(index) = self.index(at) else {
            return Cell::BLANK;
        };
        self.cells.get(index).copied().unwrap_or(Cell::BLANK)
    }

    /// Refill from `cells`, a row-major block covering `area`.
    pub(crate) fn fill(&mut self, area: Rect, cells: impl Iterator<Item = Cell>) {
        self.area = area;
        // Cleared rather than reallocated: this is refilled on every settled
        // frame, and a 120×45 grid of 8-byte cells is 43 KiB that has no business
        // being handed back to the allocator sixty times a second.
        self.cells.clear();
        self.cells.extend(cells);
    }

    /// Forget everything.
    ///
    /// **What reduce-motion leaves behind**, which is why this is public where
    /// the refill is not: a player who has asked for no motion should
    /// not be paying 43 KiB to hold a screen that will never be crossed out of.
    /// The shell's `Passing::set_enabled` is the caller.
    pub fn clear(&mut self) {
        self.area = Rect::EMPTY;
        self.cells.clear();
    }

    fn index(&self, at: Pos) -> Option<usize> {
        if !self.area.contains(at) {
            return None;
        }
        let col = usize::from(at.col - self.area.col);
        let row = usize::from(at.row - self.area.row);
        Some(row * usize::from(self.area.cols) + col)
    }
}

/// What one cell of a crossing draws.
///
/// `at` is inside `area`; `source` is the kept cell on the way out and the live
/// cell on the way in. Pure, and a function of position and progress alone — no
/// RNG reaches it, which is what keeps `ORBS_DUMP` reproducible and keeps every
/// animation out of `orbs-sim`'s seeded streams. [`crate::pulse`] makes the same
/// argument at length.
pub(crate) fn cell_at(
    area: Rect,
    crossing: Crossing,
    erosion: Erosion,
    at: Pos,
    source: Cell,
) -> Cell {
    // Both erosions are the same wearing-away measured along a different ruler:
    // how far this cell is from where the seam starts, how far the seam has to
    // go, and when this cell's clock runs. Three numbers, one rule — which is
    // what keeps the safety property one argument rather than two.
    let (step, span, local) = match erosion {
        Erosion::Wipe => (
            along(area, crossing, at),
            crossing.toward.span(area),
            crossing.local(),
        ),
        Erosion::Furl => (
            along(area, crossing, at),
            crossing.toward.span(area),
            peeling(area, crossing, at),
        ),
    };
    // The seam travels the span *plus* the wake, or the last cell's decay would
    // be cut off mid-ramp and the half would end on a `▓` that never got to
    // leave.
    let travelled = tween::mix(0, span.saturating_add(WAKE), local);

    let struck = step < travelled;
    let past = step.saturating_add(WAKE) < travelled;

    if crossing.is_leaving() {
        // Behind the wake, gone; inside it, decaying; ahead of it, untouched.
        if past {
            Cell::BLANK
        } else if struck {
            eroded(source, travelled - step - 1)
        } else {
            source
        }
    } else if past {
        // The mirror: behind the wake the new screen has arrived, inside it, it
        // is still building, ahead of it there is nothing yet.
        source
    } else if struck {
        eroded(source, WAKE - (travelled - step))
    } else {
        Cell::BLANK
    }
}

/// Where a cell of a [`Passage::Gather`] takes its glyph from, or `None` for a
/// cell the screen has moved off.
///
/// **The inverse of the motion, which is what keeps this a pure function.** The
/// screen scales about the region's centre, so a glyph at `s` lands at
/// `centre + (s - centre) * scale` — a *scatter*, many sources onto one
/// destination, which a per-destination function cannot answer. Read backwards it
/// is a sample: the cell at `d` shows whatever was at
/// `centre + (d - centre) / scale`, and that is one position with one answer.
///
/// **The whole thing goes through [`tween::mix`]**, which is not incidental. It
/// is the crate's only `f32`-to-integer conversion and the reason the
/// justification for rounding is written once; `mix(centre, at, 1.0 / scale)` is
/// exactly `centre + (at - centre) / scale`, so this needs no arithmetic of its
/// own. Its two guards land the two edge cases for free: at `scale` of zero the
/// division is infinite, every offset cell rounds outside the region and reads
/// blank, and the centre cell's `0 × ∞` is a `NaN` that `mix` answers with the
/// centre itself — which is precisely *"everything has arrived at the middle"*.
pub(crate) fn sampled(area: Rect, crossing: Crossing, at: Pos) -> Option<Pos> {
    let (col, row) = centre(area);
    let reach = 1.0 / crossing.scale();
    Some(Pos::new(
        axis(col, at.col, reach, area.col, area.right())?,
        axis(row, at.row, reach, area.row, area.bottom())?,
    ))
}

/// One axis of a [`sampled`] position, or `None` once it has left the region.
///
/// **Bounded before it is rounded, and that ordering is the whole of it.**
/// [`tween::mix`] clamps into `u16`'s range, which is exactly right for a
/// coordinate and exactly wrong for a *test*: a region whose edge is the grid
/// origin catches a sample that has gone off the far side, because `-∞` clamps to
/// `0` and `0` is inside. That is not hypothetical — it drew the centre column
/// three rows tall at the beat, with the top two rows reading the same cell,
/// and `examples/screens` is what showed it.
///
/// So the offset is measured in `f32` and judged there, and `mix` is asked only
/// for the rounding once the answer is known to be a real position. The `NaN` is
/// the centre cell's `0 × ∞` and is the one sample that always survives —
/// everything has arrived at the middle, and the middle is what is there.
fn axis(centre: u16, at: u16, reach: f32, low: u16, high: u16) -> Option<u16> {
    let value = f32::from(centre) + (f32::from(at) - f32::from(centre)) * reach;
    if value.is_nan() {
        return Some(centre);
    }
    if value < f32::from(low) || value >= f32::from(high) {
        return None;
    }
    // **And again after rounding, which is not belt and braces.** `mix` rounds,
    // so anything in the last half-cell of the range passes the test above and
    // comes back one past the end — and `Kept` holds the **whole grid**, so a
    // sample one past the region reads whatever is really there rather than
    // blank. That is the pane's own border: a gather drew a full copy of the
    // bottom rule one row up, inside the interior, for the first and last tenth
    // of every `wander`.
    //
    // The `f32` test above still earns its place — it is what catches `-∞`
    // clamping to `0` — so this is a second question rather than a better one.
    let rounded = tween::mix(centre, at, reach);
    (rounded >= low && rounded < high).then_some(rounded)
}

/// The cell a [`Passage::Gather`] converges on.
///
/// Whole cells, because a glyph cannot be drawn half in one. An even-sided region
/// has no true middle and takes the lower of the two, which is arbitrary and only
/// has to be *consistent* — the same cell has to be the destination going out and
/// the origin coming back, or the screen would arrive from somewhere it did not
/// leave.
const fn centre(area: Rect) -> (u16, u16) {
    (
        area.col.saturating_add(area.cols / 2),
        area.row.saturating_add(area.rows / 2),
    )
}

/// The rectangle a [`Passage::Gather`] can still be drawing in.
///
/// **The safety property for a shape that moves rather than erodes.** Every cell
/// this can light is inside `centre ± half-extent × scale`, so the lit region is
/// *nested*: it shrinks monotonically to a point through the leaving half and
/// grows back monotonically through the arriving one. The field's lit area
/// therefore never oscillates, which is what the photosensitive band is actually
/// about — `pulse` says the same thing from the other side, *"whole-field
/// modulation is the hazard; motion is not."*
#[must_use]
pub fn envelope(area: Rect, crossing: Crossing) -> Rect {
    let scale = crossing.scale();
    let (col, row) = centre(area);
    let half = |extent: u16| tween::mix(0, extent / 2, scale);
    let (wide, tall) = (half(area.cols), half(area.rows));
    Rect::new(
        col.saturating_sub(wide),
        row.saturating_sub(tall),
        wide.saturating_mul(2).saturating_add(1),
        tall.saturating_mul(2).saturating_add(1),
    )
    .intersection(area)
}

/// How far a cell is from the edge the seam starts at.
///
/// **The two halves measure from opposite edges, and that is what makes a
/// screen leave and return the *same way*.** Going out toward the right, the
/// last content standing is against the right edge, so the seam starts at the
/// left; coming back in from the right, the first content to appear is against
/// the right edge, so the seam starts there instead. Measured the same way in
/// both halves, a screen would exit rightward and then arrive from the left,
/// which reads as two unrelated motions rather than as one thing going and
/// coming back.
///
/// One arithmetic line does it, rather than a second copy of the rule with its
/// comparisons flipped.
fn along(area: Rect, crossing: Crossing, at: Pos) -> u16 {
    let (step, span) = match crossing.toward {
        // Toward the right: out from the left edge, back in from the right.
        Toward::Right => (at.col.saturating_sub(area.col), area.cols),
        // Toward the top: out from the bottom edge, back in from the top.
        Toward::Up => (
            area.rows
                .saturating_sub(at.row.saturating_sub(area.row))
                .saturating_sub(1),
            area.rows,
        ),
    };
    if crossing.is_leaving() {
        step
    } else {
        span.saturating_sub(step).saturating_sub(1)
    }
}

/// [`Passage::Furl`]'s clock: each row a little behind the one above it.
///
/// A stagger rather than a slide. It peels, which is what the name was reaching
/// for, and it costs nothing that a translation would buy — the two screens are
/// the *same content re-read*, so there is nothing to carry across.
///
/// The offset comes from [`crate::pulse::stagger`] — an integer hash of position,
/// never an RNG — so the same progress draws the same picture and `ORBS_DUMP`
/// stays reproducible.
fn peeling(area: Rect, crossing: Crossing, at: Pos) -> f32 {
    // How much of the half is spent spreading the rows out. Past about a half the
    // last row barely moves before the beat arrives and the peel reads as a lag.
    const SPREAD: f32 = 0.4;
    let row = at.row.saturating_sub(area.row);
    let behind = crate::pulse::stagger(row, 0) * SPREAD;
    // Rescaled rather than clipped, so the last row still finishes exactly at the
    // end of its half — which is what keeps the endpoints identities.
    ((crossing.local() - behind) / (1.0 - SPREAD)).clamp(0.0, 1.0)
}

/// A cell part-way through the shade ramp. `depth` counts down from the glyph.
///
/// **A blank cell stays blank**, and that is a correctness rule rather than a
/// nicety. The safety property is that a cell's coverage moves in one direction
/// per half, and painting `▓` over empty space would raise the coverage of a cell
/// that had none — a brightening on the way *out*, which is the reversal the whole
/// argument turns on.
///
/// The style is carried through untouched and only the glyph changes. A cell
/// leaves in its own colour, which is what "this content is going" looks like;
/// dimming it as well would be a second statement about the same event, and
/// [`Depiction`](crate::Depiction) picks a ramp rather than a hue, so a flame's
/// last three cells stay a flame's.
fn eroded(source: Cell, depth: u16) -> Cell {
    if source.is_blank() {
        return Cell::BLANK;
    }
    let Some(glyph) = RAMP.get(usize::from(depth)) else {
        return source;
    };
    Cell::new(*glyph, source.style)
}

/// How covered a cell reads, `0..=4`.
///
/// The scale the safety property is stated on: blank, then the three ramp rungs,
/// then anything else. Test-only, because it is a claim *about* the ramp rather
/// than a thing any painter needs — but it lives here beside the ramp, so the two
/// cannot drift apart.
#[cfg(test)]
pub(crate) fn coverage(cell: Cell) -> u8 {
    if cell.is_blank() {
        return 0;
    }
    match RAMP.iter().position(|glyph| *glyph == cell.glyph) {
        // `▓` is index 0 and the most covered of the three, so the ramp is walked
        // backwards to put it nearest an ordinary glyph.
        Some(rung) => 3 - u8::try_from(rung).unwrap_or(0),
        None => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;

    const AREA: Rect = Rect::new(4, 2, 10, 3);

    /// Every shape, so a property is asserted of the vocabulary rather than of
    /// whichever member of it was written first.
    const SHAPES: [Passage; 3] = [Passage::Wipe, Passage::Furl, Passage::Gather];

    fn crossing(progress: f32) -> Crossing {
        shaped(Passage::Wipe, progress)
    }

    const fn shaped(passage: Passage, progress: f32) -> Crossing {
        Crossing {
            passage,
            toward: Toward::Right,
            progress,
        }
    }

    fn lit(glyph: char) -> Cell {
        Cell::new(glyph, Style::NORMAL)
    }

    /// One cell of any shape, resolved the way `Frame::cross` resolves it.
    ///
    /// The two families answer different questions — an erosion asks *"how far
    /// through is this cell"*, a gather asks *"where does this cell read from"* —
    /// so a property swept across all three has to ask each in its own terms.
    /// Every source here is the same glyph, so *which* in-region position a
    /// gather lands on cannot matter, only whether it lands on one.
    fn drawn(area: Rect, crossing: Crossing, at: Pos, source: Cell) -> Cell {
        match crossing.passage.erosion() {
            Some(erosion) => cell_at(area, crossing, erosion, at, source),
            None => sampled(area, crossing, at).map_or(Cell::BLANK, |_| source),
        }
    }

    /// Every cell of `AREA`, left to right and top to bottom.
    fn cells() -> impl Iterator<Item = Pos> {
        (AREA.row..AREA.bottom())
            .flat_map(|row| (AREA.col..AREA.right()).map(move |col| Pos::new(col, row)))
    }

    #[test]
    fn the_endpoints_are_identities() {
        // Everything else in this module rests on this: a crossing that has not
        // started and one that has finished must be indistinguishable from no
        // crossing at all, or a settled `Passage` would move every `dumps.sh`
        // baseline in the repository.
        for passage in SHAPES {
            for at in cells() {
                let source = lit('x');
                for progress in [0.0, 1.0] {
                    assert_eq!(
                        drawn(AREA, shaped(passage, progress), at, source),
                        source,
                        "{passage:?} at {at:?}, progress {progress}",
                    );
                }
            }
        }
    }

    #[test]
    fn the_midpoint_is_all_but_empty() {
        // The beat between the two screens. It is what makes the second half read
        // as an arrival rather than as a cross-fade of two texts, which on a cell
        // grid is illegible.
        //
        // **All but**, because a `Gather` converges rather than erodes: at the
        // beat everything has arrived at the middle, so the one cell it arrived
        // at is still lit. That is the shape working, not a screen failing to
        // vanish — and one cell of a hundred is the beat either way.
        for passage in SHAPES {
            let standing = cells()
                .filter(|at| !drawn(AREA, shaped(passage, MIDPOINT), *at, lit('x')).is_blank())
                .count();
            let most = usize::from(passage == Passage::Gather);
            assert!(
                standing <= most,
                "{passage:?} left {standing} cells standing at the beat",
            );
        }
    }

    #[test]
    fn an_eroding_cell_reverses_direction_once() {
        // **The photosensitivity property for the shapes that erode**, asserted
        // rather than argued. §19: "a flash is a *pair* of opposing changes", so
        // one reversal is one flash — and one flash per crossing is what lets the
        // shell bound the rate with a single floor rather than a rule per shape.
        //
        // `Gather` is held to `a_gather_never_lights_outside_its_envelope`
        // instead, and the two are the same argument at different scales: what
        // the band is about is the **field** modulating, and a shape that moves
        // glyphs makes a cell flicker without the field doing anything of the
        // kind. `pulse` says it from the other side — *"whole-field modulation is
        // the hazard; motion is not."*
        //
        // Sampled far above any frame rate, so a transient between two frames
        // cannot hide. `pulse::harness` makes the same argument for the
        // instruments.
        const STEPS: u16 = 480;
        for passage in [Passage::Wipe, Passage::Furl] {
            for at in cells() {
                let mut reversals = 0;
                let mut rising: Option<bool> = None;
                let mut last = coverage(drawn(AREA, shaped(passage, 0.0), at, lit('x')));
                for step in 1..=STEPS {
                    let progress = f32::from(step) / f32::from(STEPS);
                    let now = coverage(drawn(AREA, shaped(passage, progress), at, lit('x')));
                    if now != last {
                        let up = now > last;
                        if rising.is_some_and(|was| was != up) {
                            reversals += 1;
                        }
                        rising = Some(up);
                    }
                    last = now;
                }
                assert!(
                    reversals <= 1,
                    "{passage:?} at {at:?} reversed {reversals} times \
                     — that is {reversals} flashes",
                );
            }
        }
    }

    #[test]
    fn a_gather_never_samples_outside_its_region() {
        // **Sources, not destinations** — which is the hole
        // `a_gather_never_lights_outside_its_envelope` left, because it asserts
        // where a gather *writes* and this asserts where it *reads*. `Kept` holds
        // the whole grid, so one cell past the region is not blank, it is the
        // pane border: the rounded coordinate escaped by half a cell and a
        // gather drew a copy of the bottom rule inside the interior.
        const STEPS: u16 = 400;
        for step in 0..=STEPS {
            let progress = f32::from(step) / f32::from(STEPS);
            let crossing = shaped(Passage::Gather, progress);
            for at in cells() {
                if let Some(from) = sampled(AREA, crossing, at) {
                    assert!(
                        AREA.contains(from),
                        "sampled {from:?} outside {AREA:?} at {progress}",
                    );
                }
            }
        }
    }

    #[test]
    fn a_gather_never_lights_outside_its_envelope() {
        // **The field-level property, which is the one the band is actually
        // about.** A gather moves glyphs, so a single cell may light and go dark
        // several times as the screen passes over it — but everything it can
        // light is inside a rectangle that shrinks to a point and grows back, so
        // the *lit area* never oscillates. That is what "no flash" means for a
        // shape that moves.
        //
        // Nested, too: the envelope at a later step of the leaving half is inside
        // the one before it, which is what makes the shrink monotone rather than
        // merely bounded.
        const STEPS: u16 = 240;
        let mut last = envelope(AREA, shaped(Passage::Gather, 0.0));
        for step in 0..=STEPS {
            let progress = f32::from(step) / f32::from(STEPS);
            let crossing = shaped(Passage::Gather, progress);
            let now = envelope(AREA, crossing);

            for at in cells() {
                if !drawn(AREA, crossing, at, lit('x')).is_blank() {
                    assert!(now.contains(at), "{at:?} lit outside {now:?} at {progress}");
                }
            }
            // Nested *within* the leaving half — each step inside the one
            // before. The two halves are mirror images, so the envelope grows
            // again across the beat and the arriving half is the same claim
            // reversed.
            if step > 0 && crossing.is_leaving() {
                assert_eq!(
                    now.intersection(last),
                    now,
                    "the envelope grew while the screen was leaving, at {progress}",
                );
            }
            last = now;
        }
    }

    #[test]
    fn a_gather_ends_on_the_middle_and_starts_there_again() {
        // The shape's whole claim, and the reason the middle has to be the *same*
        // cell in both halves: a screen that left by one point and arrived from
        // another would read as two motions rather than one thing going and
        // coming back.
        let (col, row) = centre(AREA);
        let middle = Pos::new(col, row);

        // Everything but the middle has gone by the beat...
        let beat = shaped(Passage::Gather, MIDPOINT);
        assert!(!drawn(AREA, beat, middle, lit('x')).is_blank());
        for at in cells().filter(|at| *at != middle) {
            assert!(
                drawn(AREA, beat, at, lit('x')).is_blank(),
                "{at:?} was still lit at the beat",
            );
        }

        // ...and the far corners are the first to go and the last to return.
        let corner = Pos::new(AREA.col, AREA.row);
        assert!(drawn(AREA, shaped(Passage::Gather, 0.35), corner, lit('x')).is_blank());
        assert!(!drawn(AREA, shaped(Passage::Gather, 0.35), middle, lit('x')).is_blank());
        assert!(drawn(AREA, shaped(Passage::Gather, 0.65), corner, lit('x')).is_blank());
        assert!(!drawn(AREA, shaped(Passage::Gather, 0.65), middle, lit('x')).is_blank());
    }

    #[test]
    fn a_furl_peels_rather_than_leaving_as_one_row() {
        // If every row left together this would be a `Wipe` with a different
        // name. The stagger is the shape.
        let mid = shaped(Passage::Furl, 0.2);
        let column: Vec<_> = (AREA.row..AREA.bottom())
            .map(|row| drawn(AREA, mid, Pos::new(AREA.col, row), lit('x')))
            .collect();
        assert!(
            column.iter().any(|cell| *cell != column[0]),
            "every row furled in step: {column:?}",
        );
    }

    #[test]
    fn the_wake_decays_rather_than_cutting() {
        // A hard edge reads as a rectangle being drawn over the screen. The ramp
        // is what makes it read as the content coming apart.
        let mut seen = Vec::new();
        for step in 0..=60u16 {
            let progress = f32::from(step) / 60.0 * MIDPOINT;
            let cell = drawn(
                AREA,
                crossing(progress),
                Pos::new(AREA.col, AREA.row),
                lit('x'),
            );
            if !seen.contains(&cell.glyph) {
                seen.push(cell.glyph);
            }
        }
        assert_eq!(seen, vec!['x', '▓', '▒', '░', ' '], "the ramp was skipped");
    }

    #[test]
    fn a_blank_cell_never_brightens_on_its_way_out() {
        // The rule `eroded` exists for. Painting the ramp over empty space would
        // raise a cell's coverage while the screen was *leaving*, which is the
        // reversal the safety argument turns on — so it would break the property
        // above rather than merely look odd.
        for passage in SHAPES {
            for at in cells() {
                for step in 0..=40u16 {
                    let progress = f32::from(step) / 40.0;
                    assert_eq!(
                        drawn(AREA, shaped(passage, progress), at, Cell::BLANK),
                        Cell::BLANK,
                        "{passage:?} at {at:?}, progress {progress}",
                    );
                }
            }
        }
    }

    #[test]
    fn every_glyph_a_crossing_writes_is_drawable() {
        // `Cell::new` substitutes anything outside the repertoire, so this cannot
        // fail silently — but a ramp glyph that had to be substituted would draw
        // `?` across the whole wake, and the CP437 table is the sort of thing an
        // edit reaches for its shading characters.
        for glyph in RAMP {
            assert!(
                crate::cp437::is_renderable(glyph),
                "{glyph} is outside CP437"
            );
        }
    }

    #[test]
    fn an_upward_wipe_is_a_rightward_one_turned() {
        // One arithmetic line decides the axis, so what has to hold is that it
        // really is the same motion turned — not a second copy of the rule with
        // its comparisons transposed, which is how the two would come to
        // disagree. A square region, so the two axes are directly comparable.
        const SQUARE: Rect = Rect::new(0, 0, 6, 6);
        let upward = Crossing {
            toward: Toward::Up,
            ..crossing(0.3)
        };
        for row in SQUARE.row..SQUARE.bottom() {
            for col in SQUARE.col..SQUARE.right() {
                // Rightward measures from the left edge; upward from the bottom.
                // The same cell of the motion is the transpose, flipped in the
                // axis that runs backwards.
                let across = Pos::new(SQUARE.right() - 1 - row, col);
                assert_eq!(
                    drawn(SQUARE, upward, Pos::new(col, row), lit('x')),
                    drawn(SQUARE, crossing(0.3), across, lit('x')),
                    "at ({col}, {row})",
                );
            }
        }
    }

    #[test]
    fn a_screen_returns_by_the_edge_it_left_by() {
        // **The half that reads wrong if it is measured the same way as the
        // other.** Going out toward the right the last content standing is
        // against the right edge; coming back in from the right, the first
        // content to appear must be against the same edge. Measured identically
        // in both halves a screen exits right and arrives from the left, which
        // reads as two unrelated motions.
        // Coverage rather than glyphs: near the beat the surviving edge is inside
        // its own wake rather than intact, and *"more covered than the far
        // side"* is the claim — "still whole" would be a claim about the wake's
        // length instead.
        let edge = Pos::new(AREA.right() - 1, AREA.row);
        let far = Pos::new(AREA.col, AREA.row);
        let at = |progress: f32, pos| coverage(drawn(AREA, crossing(progress), pos, lit('x')));

        // Late in the leaving half: the right edge is the survivor.
        assert!(at(0.42, edge) > at(0.42, far), "it left by the wrong side");
        // Early in the arriving half: the right edge is the first one back.
        assert!(at(0.58, edge) > at(0.58, far), "it came back the wrong way");
    }

    #[test]
    fn a_bad_progress_draws_the_screen_it_had() {
        // The useful answer to a `NaN` is the screen the game already has, not a
        // pane of noise. `ORBS_PASSAGE_AT` is typed by hand and `dump.rs` records
        // what a `NaN` phase did to the fire.
        for at in cells() {
            let source = lit('x');
            assert_eq!(
                drawn(AREA, crossing(f32::NAN), at, source),
                source,
                "at {at:?}",
            );
        }
    }

    #[test]
    fn kept_cells_outside_the_region_read_blank() {
        // What makes a crossing between two differently-shaped regions work.
        let mut kept = Kept::default();
        kept.fill(Rect::new(0, 0, 2, 1), [lit('a'), lit('b')].into_iter());
        assert_eq!(kept.cell(Pos::new(1, 0)), lit('b'));
        assert_eq!(kept.cell(Pos::new(2, 0)), Cell::BLANK);
        assert_eq!(kept.cell(Pos::new(0, 9)), Cell::BLANK);
    }

    #[test]
    fn keeping_reuses_the_allocation() {
        // Refilled on every settled frame — 43 KiB at a 120×45 grid, which has no
        // business reaching the allocator sixty times a second.
        let mut kept = Kept::default();
        kept.fill(Rect::new(0, 0, 64, 16), std::iter::repeat_n(lit('x'), 1024));
        let capacity = kept.cells.capacity();
        kept.fill(Rect::new(0, 0, 64, 16), std::iter::repeat_n(lit('y'), 1024));
        assert_eq!(kept.cells.capacity(), capacity);
    }
}
