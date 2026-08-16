//! Writing into a frame.
//!
//! Every write goes through a [`Painter`] bounded to a rectangle, so painting
//! outside a pane is impossible rather than merely discouraged — a stale layout
//! draws less, never into a neighbour.
//!
//! # The split that matters
//!
//! The API divides into **content**, which speaks, and **structure**, which does
//! not:
//!
//! | Method | Cells | Speech |
//! |---|---|---|
//! | [`Painter::span`], [`Painter::paragraph`], [`Painter::progress`] | yes | yes |
//! | [`Painter::fill`], [`Painter::clear`], [`Painter::border`] | yes | no |
//! | [`Painter::announce`] | no | yes |
//!
//! Borders, rules, and padding carry no information, so speaking them would
//! drown the stream in `"┌──────┐"`. Everything else must speak, because §14
//! makes the linear stream a first-class view of the frame rather than a
//! debugging aid.
//!
//! [`Painter::border`] announces its title for exactly this reason: the title is
//! drawn *into* the structural border, so without the announcement a pane would
//! lose its identity in the linear stream.

use crate::bath;
use crate::cell::Cell;
use crate::cp437::box_drawing;
use crate::fire;
use crate::frame::Frame;
use crate::geometry::{Pos, Rect};
use crate::grind;
use crate::linear::UtteranceKind;
use crate::maze::{self, Stacks};
use crate::mix;
use crate::span::Span;
use crate::style::{Presentation, Role, Style, Wash};
use crate::wrap::Wrap;

/// A clipped writer into a region of a [`Frame`].
///
/// All positions are absolute grid coordinates, matching the rectangles
/// [`crate::ScreenLayout`] hands out.
#[derive(Debug)]
pub struct Painter<'a> {
    frame: &'a mut Frame,
    area: Rect,
}

impl<'a> Painter<'a> {
    pub(crate) fn new(frame: &'a mut Frame, area: Rect) -> Self {
        let area = area.intersection(frame.area());
        Self { frame, area }
    }

    /// The region this painter may write to.
    #[must_use]
    pub const fn area(&self) -> Rect {
        self.area
    }

    /// A painter for a sub-region, clipped to this one.
    pub fn sub(&mut self, area: Rect) -> Painter<'_> {
        Painter {
            area: area.intersection(self.area),
            frame: &mut *self.frame,
        }
    }

    /// Draw a span on one row, truncating at the region's right edge.
    ///
    /// Returns the number of cells written.
    ///
    /// The **full** text is recorded for the linear stream even when the visual
    /// form is truncated. A narrow pane is a visual constraint; withholding the
    /// rest of the sentence from a screen reader would make it an informational
    /// one.
    ///
    /// # Panics
    ///
    /// In debug builds, if the span is [`Presentation::Eldritch`] without an
    /// authored spoken variant. DESIGN.md §3 requires one, and the alternative
    /// to catching it here is shipping a tonal register that screen-reader
    /// players cannot hear.
    pub fn span(&mut self, at: Pos, span: &Span<'_>) -> u16 {
        debug_assert!(
            span.style().presentation != Presentation::Eldritch || span.has_spoken_variant(),
            "eldritch span with no authored spoken variant: {:?}",
            span.text()
        );
        self.frame.speech_mut().push_with(
            span.kind(),
            span.style().role,
            span.outcome(),
            span.spoken_text(),
        );
        self.put_str(at, span.text(), span.style(), u16::MAX)
    }

    /// Draw a span as word-wrapped prose filling `area`.
    ///
    /// Honours embedded newlines, including blank lines. Returns the number of
    /// rows used.
    ///
    /// # Panics
    ///
    /// As [`Painter::span`].
    pub fn paragraph(&mut self, area: Rect, span: &Span<'_>) -> u16 {
        debug_assert!(
            span.style().presentation != Presentation::Eldritch || span.has_spoken_variant(),
            "eldritch paragraph with no authored spoken variant: {:?}",
            span.text()
        );
        self.frame.speech_mut().push_with(
            span.kind(),
            span.style().role,
            span.outcome(),
            span.spoken_text(),
        );

        let area = area.intersection(self.area);
        if area.is_empty() {
            return 0;
        }

        let mut row = area.row;
        for line in span.text().split('\n') {
            if row >= area.bottom() {
                break;
            }
            if line.is_empty() {
                row = row.saturating_add(1);
                continue;
            }
            for wrapped in Wrap::new(line, area.cols) {
                if row >= area.bottom() {
                    break;
                }
                self.put_str(Pos::new(area.col, row), wrapped, span.style(), area.right());
                row = row.saturating_add(1);
            }
        }
        row.saturating_sub(area.row)
    }

    /// Draw a meter as a bar, and record `spoken` as its description.
    ///
    /// Integer-only: progress in this game is elapsed ticks against a duration
    /// (DESIGN.md §5.0), and keeping floats out of the render path keeps a
    /// deterministic sim rendering deterministically. `done` is clamped to
    /// `total`; a `total` of zero draws an empty bar.
    ///
    /// `spoken` is what a reader hears — `"east wall integrity 34 percent"`, not
    /// a row of block glyphs. §14 names progress bars specifically.
    pub fn progress(&mut self, area: Rect, done: u32, total: u32, style: Style, spoken: &str) {
        // Spoken *before* the clip test, deliberately: a meter scrolled out of
        // its pane is still a fact a listener needs, and §14's whole contract is
        // that the linear stream does not depend on what happened to fit.
        self.frame
            .speech_mut()
            .push(UtteranceKind::Progress, style.role, spoken);
        self.meter(area, done, total, style);
    }

    /// Draw a meter as a bar, silently.
    ///
    /// The same glyphs as [`Painter::progress`] with **nothing said**. For a
    /// panel of standing meters — §10.1's five instruments — where speaking each
    /// one per frame would bury the stream in furniture, and §14 asks for
    /// *"progress announcements: completion only"*.
    ///
    /// A caller using this owes the listener one summary utterance covering the
    /// panel, which is what [`Painter::announce`] is for. Drawing meters and
    /// saying nothing at all would be a screen a reader cannot see.
    ///
    /// Integer-only: progress in this game is elapsed ticks against a duration
    /// (DESIGN.md §5.0), and keeping floats out of the render path keeps a
    /// deterministic sim rendering deterministically. `done` is clamped to
    /// `total`; a `total` of zero draws an empty bar.
    ///
    /// **It fills the whole rectangle**, as [`Painter::meter_upward`] does with
    /// its own. This used to draw a single row however tall the rect was, which
    /// [`Painter::fire_meter`] and [`Painter::grind_meter`] do not — so the same
    /// two-row bar would be two rows thick for one instrument and one row thick
    /// for the next. Every caller passes a single row today; what this fixes is
    /// the four painters being able to disagree about it tomorrow.
    pub fn meter(&mut self, area: Rect, done: u32, total: u32, style: Style) {
        let area = area.intersection(self.area);
        if area.is_empty() {
            return;
        }

        let filled = filled_of(area.cols, done, total);
        self.fill(area, '░', style);
        self.fill(Rect::new(area.col, area.row, filled, area.rows), '█', style);
    }

    /// The same meter, drawn as a column that fills **upward**.
    ///
    /// A level reads as growing from the floor, which is what §10.1's side panel
    /// wants when the pane is taller than it is wide.
    ///
    /// It lives here rather than in a frontend because the alternative already
    /// happened: the Bevy panel re-derived the fill arithmetic and hand-drew the
    /// `█`/`░` pair, so one instrument panel drew its bar two different ways
    /// depending on which way the pane had split. Changing the glyphs or the
    /// clamping in `meter` would have left the side panel on the old ones, and
    /// a player pressing F4 would see the same five instruments in two bar
    /// vocabularies.
    pub fn meter_upward(&mut self, area: Rect, done: u32, total: u32, style: Style) {
        let area = area.intersection(self.area);
        if area.is_empty() {
            return;
        }

        let filled = filled_of(area.rows, done, total);
        for step in 0..area.rows {
            let row = area.bottom().saturating_sub(1).saturating_sub(step);
            let glyph = if step < filled { '█' } else { '░' };
            self.fill(Rect::new(area.col, row, area.cols, 1), glyph, style);
        }
    }

    /// [`Painter::meter`], drawn as a fire.
    ///
    /// For the athanor, which is the one instrument that *is* one (§10.1). Its
    /// meter reports fuel remaining rather than ticks elapsed, so the bar drains
    /// — the flame shrinks and the plume above it grows, with nothing here
    /// arranging for that.
    ///
    /// **Silent, and the fill boundary is exactly [`Painter::meter`]'s.** Both
    /// take their length from the same `filled_of`, so the two can never disagree
    /// about where the value is — and the cell behind the flame front is always
    /// `█` with the cell ahead of it always `░`, so the join reads with no colour
    /// at all. §14: the meter's *value* must never be carried by hue.
    ///
    /// [`FLIP_HZ`](crate::FLIP_HZ) caps how fast any one cell may change, and
    /// does so by construction. 3–30 Hz is the photosensitive band.
    ///
    /// `phase` is elapsed seconds from the frontend's own clock. Nothing here
    /// knows what a second is.
    pub fn fire_meter(&mut self, area: Rect, done: u32, total: u32, burn: fire::Burn) {
        self.burning(area, done, total, burn, Runs::Rightward);
    }

    /// [`Painter::meter_upward`], drawn as a fire.
    ///
    /// The orientation the side panel uses, and the one the metaphor was built
    /// for: the plume rises. See [`Painter::fire_meter`].
    pub fn fire_meter_upward(&mut self, area: Rect, done: u32, total: u32, burn: fire::Burn) {
        self.burning(area, done, total, burn, Runs::Upward);
    }

    /// [`Painter::meter`], drawn as a mortar being worked.
    ///
    /// **The bar is what is in the bowl**, not how far along: chunks become
    /// powder, the boundary is where the pestle works, and the whole bar carries
    /// something at every moment. So a *loaded* mortar and a *finished* one are
    /// this same picture at nothing-ground and everything-ground — which is what
    /// lets the panel draw two states the sim reports no meter for at all.
    ///
    /// Silent, and the value boundary is exactly [`Painter::meter`]'s: both take
    /// their length from the same `filled_of`, and the cell against the face is
    /// always `░` so the join reads with no colour at all.
    ///
    /// **Nothing here is a tool.** An earlier version put a pestle `■` in the
    /// working gap, and a mark from outside the fill vocabulary reads as an
    /// object visiting the bar rather than as the material changing state. The
    /// four shades are four states of one substance, which is the whole picture —
    /// and it is direction-neutral for free, which the pestle needed an argument
    /// for: §10.1's panel turns horizontal whenever the pane is taller than it is
    /// wide.
    pub fn grind_meter(&mut self, area: Rect, done: u32, total: u32, work: grind::Grind) {
        self.grinding(area, done, total, work, Runs::Rightward);
    }

    /// [`Painter::meter_upward`], drawn as a mortar being worked.
    ///
    /// The orientation the side panel uses, and the one the pestle's swing was
    /// built for — though the glyph is chosen so the horizontal form reads too.
    /// See [`Painter::grind_meter`].
    pub fn grind_meter_upward(&mut self, area: Rect, done: u32, total: u32, work: grind::Grind) {
        self.grinding(area, done, total, work, Runs::Upward);
    }

    /// [`Painter::meter`], drawn as a water bath.
    ///
    /// For the balneum mariae (§10.1), which digests gently over the athanor.
    /// **The bar is the liquid in the vessel**, not how far along: a charged bath
    /// is a shallow layer and a finished one is full, so the two states the sim
    /// reports no meter for are this same picture at its ends rather than
    /// special cases.
    ///
    /// Silent, and the value boundary is exactly [`Painter::meter`]'s: both take
    /// their length from the same `filled_of`. The join is solid against blank —
    /// **all** of the bath's motion is in its colour, so the level survives
    /// greyscale with nothing to argue about.
    pub fn bath_meter(&mut self, area: Rect, done: u32, total: u32, work: bath::Steep) {
        // **The orientation is the painter's to know, not the caller's.** A
        // caller that had to set `upward` itself could set it wrong, and the one
        // thing it decides — whether bubbles break into the air above the face —
        // would then be drawn sideways. See [`bath::Steep::upward`].
        let work = bath::Steep {
            upward: false,
            ..work
        };
        self.steeping(area, done, total, work, Runs::Rightward);
    }

    /// [`Painter::meter_upward`], drawn as a water bath.
    ///
    /// The orientation the side panel uses, and the one a level reads best in.
    /// See [`Painter::bath_meter`].
    pub fn bath_meter_upward(&mut self, area: Rect, done: u32, total: u32, work: bath::Steep) {
        let work = bath::Steep {
            upward: true,
            ..work
        };
        self.steeping(area, done, total, work, Runs::Upward);
    }

    /// [`Painter::fire_meter`] and its upward twin, which differ only in which
    /// way the bar runs.
    fn burning(&mut self, area: Rect, done: u32, total: u32, burn: fire::Burn, runs: Runs) {
        let area = area.intersection(self.area);
        if area.is_empty() {
            return;
        }
        let filled = filled_of(runs.steps(area), done, total);
        let guttering = guttering(filled, done, burn);
        for (step, lane, at) in runs.cells(area) {
            let (glyph, depiction) = fire::cell(lane, step, filled, guttering, burn);
            self.frame.set(
                at,
                Cell::new(glyph, Style::NORMAL.with_depiction(depiction)),
            );
        }
    }

    /// [`Painter::grind_meter`] and its upward twin. See [`Painter::burning`].
    fn grinding(&mut self, area: Rect, done: u32, total: u32, work: grind::Grind, runs: Runs) {
        let area = area.intersection(self.area);
        if area.is_empty() {
            return;
        }
        let steps = runs.steps(area);
        let filled = filling(steps, done, total, work.working, work.advance);
        for (step, lane, at) in runs.cells(area) {
            let (glyph, style) = grind::cell(lane, step, filled, steps, work);
            self.frame.set(at, Cell::new(glyph, style));
        }
    }

    /// [`Painter::meter`], drawn as a flask combining two things.
    ///
    /// **Takes the two ingredients' colours**, unlike every other picture
    /// painter, because the bar is three regions rather than one: the two inputs
    /// shrinking and the mixture growing. It writes all three into the frame's
    /// tint table itself — a caller cannot, because only this knows where the
    /// bands fall at a given fill.
    pub fn mix_meter(
        &mut self,
        area: Rect,
        done: u32,
        total: u32,
        work: mix::Stir,
        inputs: [Option<Wash>; 2],
    ) {
        self.stirring(area, done, total, work, inputs, Runs::Rightward);
    }

    /// [`Painter::meter_upward`], drawn as a flask. See [`Painter::mix_meter`].
    pub fn mix_meter_upward(
        &mut self,
        area: Rect,
        done: u32,
        total: u32,
        work: mix::Stir,
        inputs: [Option<Wash>; 2],
    ) {
        self.stirring(area, done, total, work, inputs, Runs::Upward);
    }

    /// Both flask painters.
    fn stirring(
        &mut self,
        area: Rect,
        done: u32,
        total: u32,
        work: mix::Stir,
        inputs: [Option<Wash>; 2],
        runs: Runs,
    ) {
        let area = area.intersection(self.area);
        if area.is_empty() {
            return;
        }
        let steps = runs.steps(area);
        // The creep applies only while it is being worked; a finished flask has
        // no duration in progress to sample between.
        let working = work.motion == mix::Motion::Stirring;
        let filled = filling(steps, done, total, working, work.advance);

        // The three bands, painted as tint regions. Collected as spans of `step`
        // and converted to rectangles, because `Runs` owns the mapping from a
        // step to a position and nothing else should re-derive it.
        let mut band_of = vec![None; usize::from(steps)];
        for (step, lane, at) in runs.cells(area) {
            let (glyph, depiction, band) = mix::cell(lane, step, filled, steps, work);
            if lane == 0 {
                band_of[usize::from(step)] = band;
            }
            self.frame.set(
                at,
                Cell::new(glyph, Style::NORMAL.with_depiction(depiction)),
            );
        }
        self.wash_bands(area, runs, &band_of, inputs);
    }

    /// Tint each of the flask's three bands with what it is made of.
    ///
    /// **Later regions win** (see [`Frame::tint_at`](crate::Frame::tint_at)), so
    /// these are painted in order and each simply overwrites the last where they
    /// meet — no gap arithmetic, and a band that has shrunk to nothing writes
    /// nothing because its rectangle is empty.
    fn wash_bands(
        &mut self,
        area: Rect,
        runs: Runs,
        band_of: &[Option<mix::Band>],
        inputs: [Option<Wash>; 2],
    ) {
        let mixed = match (inputs[0], inputs[1]) {
            // **Already a mixture: use it, do not blend it again.** A *finished*
            // flask holds its product and its dregs, so blending the two would
            // average `clarified-draught` with `brown` and the bar would change
            // colour at the instant the run completed — the exact discontinuity
            // authoring the draughts as mixtures exists to remove. The product
            // carries the answer; this only has to not overwrite it.
            (Some(first), _) if first.with.is_some() => Some(first),
            // **The primaries, averaged.** Combining two things that are each
            // already a mixture is not something §10.1's recipes do, and
            // averaging four families would give mud rather than a colour a
            // player could name.
            (Some(first), Some(second)) => Some(Wash::mixing(first.tint, second.tint)),
            // One ingredient in, or one untinted: there is nothing to average,
            // so the mixture simply takes whichever colour is present.
            (Some(only), None) | (None, Some(only)) => Some(only),
            (None, None) => None,
        };
        let washes = [
            (mix::Band::First, inputs[0]),
            (mix::Band::Second, inputs[1]),
            (mix::Band::Mixed, mixed),
        ];
        for (band, wash) in washes {
            let Some(wash) = wash else { continue };
            let steps: Vec<u16> = band_of
                .iter()
                .enumerate()
                .filter(|(_, held)| **held == Some(band))
                .filter_map(|(step, _)| u16::try_from(step).ok())
                .collect();
            let (Some(first), Some(last)) = (steps.first(), steps.last()) else {
                continue;
            };
            self.frame.set_tint(runs.span(area, *first, *last), wash);
        }
    }

    /// [`Painter::grind_meter`] and its upward twin, drawn as a water bath.
    fn steeping(&mut self, area: Rect, done: u32, total: u32, work: bath::Steep, runs: Runs) {
        let area = area.intersection(self.area);
        if area.is_empty() {
            return;
        }
        // **The creep applies while it is *working* and not while it is
        // settling.** `filling`'s justification is a duration in progress —
        // "an eight-tick digest is eight seconds of work" — and a finished bath
        // has none, so there is nothing to sample between.
        let working = work.motion == bath::Motion::Bubbling;
        let filled = filling(runs.steps(area), done, total, working, work.advance);
        for (step, lane, at) in runs.cells(area) {
            let (glyph, depiction) = bath::cell(lane, step, filled, work);
            self.frame.set(
                at,
                Cell::new(glyph, Style::NORMAL.with_depiction(depiction)),
            );
        }
    }

    /// Draw this region in a material's colour family.
    ///
    /// Silent and structural: a tint is a hint over `survey`, never a carrier
    /// (see [`Tint`](crate::Tint)), so it writes nothing to the linear stream —
    /// what a listener needs is the instrument's name and state, which the panel
    /// says once for all of them.
    ///
    /// Clipped to this painter's own region, so a tint cannot colour a
    /// neighbouring pane any more than a glyph can reach one.
    pub fn tint(&mut self, area: Rect, wash: Wash) {
        let area = area.intersection(self.area);
        self.frame.set_tint(area, wash);
    }

    /// Draw the stacks, centred in `area` at their natural size.
    ///
    /// Returns whether anything was drawn — false only for a region with no room
    /// at all. A region too small for the whole maze gets a **window onto it,
    /// centred on the reading**, which pans as the reading walks; see
    /// `maze::viewport` for why that replaced refusing outright.
    ///
    /// **Structural and silent**, like [`Painter::fill`] and the instrument
    /// meters. What a listener needs is the four readings and how much has been
    /// walked, and both are already in the panel's one utterance — a second
    /// continuous announcement would be the *"progress announcements: completion
    /// only"* rule (§14) broken by the very surface that most wants to break it.
    pub fn stacks(&mut self, area: Rect, maze: &Stacks) -> bool {
        let area = area.intersection(self.area);
        let Some((at, from_x, from_y)) = maze::viewport(maze, area) else {
            return false;
        };
        for row in 0..at.rows {
            for col in 0..at.cols {
                let (glyph, style) =
                    maze::cell(maze, col.saturating_add(from_x), row.saturating_add(from_y));
                self.put_cell(
                    Pos::new(at.col.saturating_add(col), at.row.saturating_add(row)),
                    glyph,
                    style,
                );
            }
        }
        true
    }

    /// Speak something with no visual form of its own.
    ///
    /// For information a sighted player reads from structure — a pane's title in
    /// its border, a column header, a layout grouping.
    pub fn announce(&mut self, kind: UtteranceKind, role: Role, text: &str) {
        self.frame.speech_mut().push(kind, role, text);
    }

    /// Draw text whose meaning a neighbouring span already carries. Structural:
    /// writes no speech.
    ///
    /// For a row drawn in several styles — `battlements ....... [ DEGRADED ]`,
    /// where the label is base hue, the leader is dim, and only the bracket takes
    /// the danger accent. One [`Painter::span`] announces the row as a whole;
    /// the remaining runs are drawn with this so the reader hears one sentence
    /// rather than three fragments.
    ///
    /// Returns the number of cells written.
    ///
    /// **This is not a silent [`Painter::span`].** Text drawn here is invisible
    /// to a screen reader, so use it only where a span or [`Painter::announce`]
    /// on the same row has already said what the row means.
    pub fn glyphs(&mut self, at: Pos, text: &str, style: Style) -> u16 {
        self.put_str(at, text, style, u16::MAX)
    }

    /// Fill a region with one glyph. Structural: writes no speech.
    pub fn fill(&mut self, area: Rect, glyph: char, style: Style) {
        let area = area.intersection(self.area);
        let cell = Cell::new(glyph, style);
        for row in area.row..area.bottom() {
            for col in area.col..area.right() {
                self.frame.set(Pos::new(col, row), cell);
            }
        }
    }

    /// Blank the painter's whole region.
    pub fn clear(&mut self) {
        self.fill(self.area, ' ', Style::NORMAL);
    }

    /// Draw a single-line box, optionally titled.
    ///
    /// The box itself is structural and silent; the title is announced as a
    /// [`UtteranceKind::Heading`], which is what gives the pane an identity in
    /// the linear stream.
    ///
    /// A region narrower or shorter than two cells draws nothing.
    pub fn border(&mut self, area: Rect, title: Option<&str>, style: Style) {
        let area = area.intersection(self.area);
        if area.cols < 2 || area.rows < 2 {
            return;
        }

        let left = area.col;
        let right = area.right() - 1;
        let top = area.row;
        let bottom = area.bottom() - 1;

        for col in left..=right {
            self.put_cell(Pos::new(col, top), box_drawing::HORIZONTAL, style);
            self.put_cell(Pos::new(col, bottom), box_drawing::HORIZONTAL, style);
        }
        for row in top..=bottom {
            self.put_cell(Pos::new(left, row), box_drawing::VERTICAL, style);
            self.put_cell(Pos::new(right, row), box_drawing::VERTICAL, style);
        }
        self.put_cell(Pos::new(left, top), box_drawing::TOP_LEFT, style);
        self.put_cell(Pos::new(right, top), box_drawing::TOP_RIGHT, style);
        self.put_cell(Pos::new(left, bottom), box_drawing::BOTTOM_LEFT, style);
        self.put_cell(Pos::new(right, bottom), box_drawing::BOTTOM_RIGHT, style);

        let Some(title) = title else {
            return;
        };
        self.frame
            .speech_mut()
            .push(UtteranceKind::Heading, style.role, title);

        // ` title ` inset one cell from the top-left corner, stopping short of
        // the far corner so the box never breaks.
        let mut col = left.saturating_add(1);
        for text in [" ", title, " "] {
            col = col.saturating_add(self.put_str(Pos::new(col, top), text, style, right));
        }
    }

    /// The same box, drawn only `progress` of the way round its perimeter.
    ///
    /// Silent and untitled: a border arriving a cell at a time has no identity
    /// to announce yet, and [`Painter::border`] speaks its title as a heading —
    /// which would put a pane into the linear stream before the pane exists.
    ///
    /// # Four corners at once
    ///
    /// It grows from **all four corners simultaneously**, each running clockwise
    /// along its own side, and they meet on the corners together. `progress` at
    /// or above 1.0 is exactly [`border`](Self::border) with no title.
    ///
    /// This replaced a single line from the top-left. That reads well and takes
    /// four times as long as it needs to: at the 80×22 floor one line is 202
    /// cells where the longest of four sides is 79, so the box closes in a
    /// little over a third of the time for the same per-cell pace.
    ///
    /// **Each side is drawn `progress` of its own length**, not at a shared
    /// cells-per-second. The sides are different lengths, so a shared pace would
    /// have the short ones finish early and sit waiting — and the shape would
    /// stop being four lines racing and start being two that had already parked.
    ///
    /// Where a frontend gets `progress` from is its own business — nothing in
    /// this crate knows what a second is. The *path* is geometry, which is why
    /// it is here.
    pub fn border_revealed(&mut self, area: Rect, style: Style, progress: f32) {
        let area = area.intersection(self.area);
        if area.cols < 2 || area.rows < 2 {
            return;
        }
        let progress = progress.clamp(0.0, 1.0);
        let left = area.col;
        let right = area.right().saturating_sub(1);
        let top = area.row;
        let bottom = area.bottom().saturating_sub(1);
        // Each side is its own length, and each is drawn `progress` of the way
        // along — so all four close on the corners together rather than the
        // short sides finishing early and waiting.
        let across = area.cols.saturating_sub(1);
        let down = area.rows.saturating_sub(1);

        // Top-left, rightwards along the top.
        self.reveal_side(
            (left..right).map(move |col| {
                let glyph = if col == left {
                    box_drawing::TOP_LEFT
                } else {
                    box_drawing::HORIZONTAL
                };
                (Pos::new(col, top), glyph)
            }),
            across,
            progress,
            style,
        );
        // Top-right, downwards.
        self.reveal_side(
            (top..bottom).map(move |row| {
                let glyph = if row == top {
                    box_drawing::TOP_RIGHT
                } else {
                    box_drawing::VERTICAL
                };
                (Pos::new(right, row), glyph)
            }),
            down,
            progress,
            style,
        );
        // Bottom-right, leftwards along the bottom.
        self.reveal_side(
            ((left + 1)..=right).rev().map(move |col| {
                let glyph = if col == right {
                    box_drawing::BOTTOM_RIGHT
                } else {
                    box_drawing::HORIZONTAL
                };
                (Pos::new(col, bottom), glyph)
            }),
            across,
            progress,
            style,
        );
        // Bottom-left, upwards.
        self.reveal_side(
            ((top + 1)..=bottom).rev().map(move |row| {
                let glyph = if row == bottom {
                    box_drawing::BOTTOM_LEFT
                } else {
                    box_drawing::VERTICAL
                };
                (Pos::new(left, row), glyph)
            }),
            down,
            progress,
            style,
        );
    }

    /// Draw `progress` of the way along one side of a revealing box.
    ///
    /// `len` is passed rather than counted because the iterator is consumed
    /// drawing it, and `mix` needs the total before the first cell is placed.
    fn reveal_side(
        &mut self,
        cells: impl Iterator<Item = (Pos, char)>,
        len: u16,
        progress: f32,
        style: Style,
    ) {
        // `mix` is the crate's one float-to-integer conversion, and it already
        // carries the justification for it — see `tween`.
        let mut budget = crate::tween::mix(0, len, progress);
        for (at, glyph) in cells {
            if budget == 0 {
                return;
            }
            self.put_cell(at, glyph, style);
            budget -= 1;
        }
    }

    fn put_cell(&mut self, at: Pos, glyph: char, style: Style) {
        if self.area.contains(at) {
            self.frame.set(at, Cell::new(glyph, style));
        }
    }

    /// Write `text` rightwards from `at`, clipped to the region and to
    /// `right_limit`. Returns cells written.
    fn put_str(&mut self, at: Pos, text: &str, style: Style, right_limit: u16) -> u16 {
        if at.row < self.area.row || at.row >= self.area.bottom() {
            return 0;
        }

        let limit = self.area.right().min(right_limit);
        let mut col = at.col.max(self.area.col);
        // Glyphs falling left of the region are consumed, not shifted right.
        let skipped = usize::from(col.saturating_sub(at.col));

        let mut written = 0u16;
        for glyph in text.chars().skip(skipped) {
            if col >= limit {
                break;
            }
            self.frame
                .set(Pos::new(col, at.row), Cell::new(glyph, style));
            col = col.saturating_add(1);
            written = written.saturating_add(1);
        }
        written
    }
}

/// Which way a picture bar runs, and the axis mapping that follows from it.
///
/// **The four picture painters had this written out four times**, with the two
/// axes swapped between the pairs — four places for the fill direction to go
/// wrong independently, in a module whose own `meter_upward` doc records the
/// panel once drawing one instrument two ways depending on which way the pane
/// had split.
///
/// `step` always counts from the end the bar fills from and `lane` always runs
/// across its thickness, so [`fire::cell`] and [`grind::cell`] can be written
/// once for both orientations — which is the property this exists to keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Runs {
    /// A row that fills from the left. The panel's `Top` layout.
    Rightward,
    /// A column that fills from the bottom. The panel's `Side` layout, and the
    /// one the fire's metaphor was built for: the plume rises.
    Upward,
}

impl Runs {
    /// How long the bar is, in cells.
    const fn steps(self, area: Rect) -> u16 {
        match self {
            Self::Rightward => area.cols,
            Self::Upward => area.rows,
        }
    }

    /// The rectangle covering steps `first..=last`, across the bar's full width.
    ///
    /// For a caller that needs a *region* rather than cells — the flask's three
    /// bands, which are tinted separately. It lives here so the step-to-position
    /// mapping stays in one place; a caller computing the rectangle itself would
    /// be the fifth copy of the fill direction, which is what this type exists
    /// to prevent.
    const fn span(self, area: Rect, first: u16, last: u16) -> Rect {
        let depth = last.saturating_sub(first).saturating_add(1);
        match self {
            Self::Rightward => {
                Rect::new(area.col.saturating_add(first), area.row, depth, area.rows)
            }
            // Upward, so the *last* step is the topmost row.
            Self::Upward => Rect::new(
                area.col,
                area.bottom().saturating_sub(1).saturating_sub(last),
                area.cols,
                depth,
            ),
        }
    }

    /// Every cell as `(step, lane, position)`.
    fn cells(self, area: Rect) -> impl Iterator<Item = (u16, u16, Pos)> {
        let (steps, lanes) = match self {
            Self::Rightward => (area.cols, area.rows),
            Self::Upward => (area.rows, area.cols),
        };
        (0..steps).flat_map(move |step| {
            (0..lanes).map(move |lane| {
                let at = match self {
                    Self::Rightward => {
                        Pos::new(area.col.saturating_add(step), area.row.saturating_add(lane))
                    }
                    Self::Upward => Pos::new(
                        area.col.saturating_add(lane),
                        area.bottom().saturating_sub(1).saturating_sub(step),
                    ),
                };
                (step, lane, at)
            })
        })
    }
}

/// How many of `steps` a meter fills.
///
/// Widened so a long duration cannot overflow the multiply, and clamped so a
/// finished meter never overruns its own track.
fn filled_of(steps: u16, done: u32, total: u32) -> u16 {
    if total == 0 {
        return 0;
    }
    let scaled = u64::from(steps) * u64::from(done.min(total)) / u64::from(total);
    u16::try_from(scaled).unwrap_or(steps)
}

/// [`filled_of`], crept forward by `advance` of the way to the next reading.
///
/// **For bars that fill.** The sim turns at 1 Hz (§5.0) and reports whole ticks,
/// so a meter read straight off it moves once a second in one jump. A duration
/// action is continuous underneath that — the tick count is a *sample* of it —
/// so drawing between samples is closer to the truth rather than further from
/// it. See [`Grind::advance`](crate::Grind::advance).
///
/// A bar that *drains* would need the reading at `done - 1`; the athanor's fuel
/// is the one of those and it does not use this.
/// How much of a mortar's bar is settled bed — crept only if it is *running*.
///
/// **A bar creeps only while there is a next sample to creep toward.** At rest
/// the panel has no quantity from the sim at all (`Charged`, `Fouled` and
/// `Ready` all report `meter: None`), so `shell::panel` stands one in — and a
/// stand-in of `0/1` fed to [`creeping`] predicts `1/1`, i.e. **the whole bar**.
/// The result was an idle bowl grinding itself to completion and snapping back
/// once a second, and a jammed one reading as `ready` for most of every second:
/// exactly the misreport §10.1's panel exists to remove.
///
/// The creep's own justification is what rules it out here — "an eight-tick
/// grind is eight seconds of work" is a claim about a *duration in progress*.
/// Nothing is in progress in a bowl at rest, so there is nothing to sample
/// between.
///
/// Takes `working` and `advance` loose rather than a `Grind`, because the bath
/// wants the identical rule and is not one. The two instruments' structs are
/// deliberately different shapes — a bath has no pour and no leavings in flight —
/// and the thing they share is this arithmetic, not a type.
fn filling(steps: u16, done: u32, total: u32, working: bool, advance: f32) -> u16 {
    if working {
        creeping(steps, done, total, advance)
    } else {
        filled_of(steps, done, total)
    }
}

fn creeping(steps: u16, done: u32, total: u32, advance: f32) -> u16 {
    let now = filled_of(steps, done, total);
    let next = filled_of(steps, done.saturating_add(1), total);
    now.saturating_add(crate::tween::mix(0, next.saturating_sub(now), advance))
}

/// Whether the fire has fuel the bar has rounded away.
///
/// The one state where the fire and the plain meter disagree, and deliberately:
/// a hearth with a handful of ticks left divides to zero cells, and a bar drawn
/// empty says *out*. "Still lit" against "cold" is exactly the distinction
/// `kindle` turns on, so losing it to integer division would be the panel
/// failing at the one job §10.1 gives it.
const fn guttering(filled: u16, done: u32, burn: fire::Burn) -> bool {
    burn.lit && filled == 0 && done > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::GridSize;

    /// A hearth alight at `phase`, with no flare in it.
    fn lit(phase: f32) -> fire::Burn {
        fire::Burn {
            phase,
            flare: 0.0,
            lit: true,
        }
    }

    /// The glyphs of a one-row bar, plain and burning, for the same reading.
    fn both(cols: u16, done: u32, total: u32, phase: f32) -> (Vec<char>, Vec<char>) {
        let area = Rect::new(0, 0, cols, 1);
        let read = |frame: &Frame| -> Vec<char> {
            (0..cols)
                .map(|col| frame.cell(Pos::new(col, 0)).map_or(' ', |cell| cell.glyph))
                .collect()
        };

        let mut plain = Frame::new(GridSize::new(cols, 1));
        plain.painter(area).meter(area, done, total, Style::NORMAL);

        let mut burning = Frame::new(GridSize::new(cols, 1));
        burning
            .painter(area)
            .fire_meter(area, done, total, lit(phase));

        (read(&plain), read(&burning))
    }

    #[test]
    fn every_meter_covers_the_rectangle_it_was_given() {
        // Four painters draw §10.1's bars and they must agree about *where*, or
        // the same rect is a two-row bar for one instrument and a one-row bar
        // for the next. `meter` was the odd one out: it drew a single row
        // however tall the rect was, while `meter_upward` filled its whole area
        // and both animated meters filled theirs.
        //
        // Caught only because nothing passes a tall rect today — which is
        // exactly how a divergence like this survives to the day something does.
        let area = Rect::new(1, 1, 8, 3);
        let painted = |draw: &dyn Fn(&mut Painter<'_>)| -> usize {
            let mut frame = Frame::new(GridSize::new(10, 5));
            draw(&mut frame.painter(area));
            (0..10)
                .flat_map(|col| (0..5).map(move |row| Pos::new(col, row)))
                .filter(|at| frame.cell(*at).is_some_and(|cell| cell.glyph != ' '))
                .count()
        };

        let cells = usize::from(area.cols) * usize::from(area.rows);
        let solid: [(&str, &dyn Fn(&mut Painter<'_>)); 4] = [
            ("meter", &|p| p.meter(area, 3, 8, Style::NORMAL)),
            ("meter_upward", &|p| {
                p.meter_upward(area, 3, 8, Style::NORMAL);
            }),
            ("grind_meter", &|p| {
                p.grind_meter(area, 3, 8, grind::Grind::default());
            }),
            ("grind_meter_upward", &|p| {
                p.grind_meter_upward(area, 3, 8, grind::Grind::default());
            }),
        ];
        for (name, draw) in solid {
            assert_eq!(painted(draw), cells, "{name} did not cover its rectangle");
        }

        // The two fire meters are the exception and it is the *point*: the plume
        // is mostly air, so a burning bar is deliberately not solid. What has to
        // hold for them is that nothing lands outside the rect.
        let sparse: [(&str, &dyn Fn(&mut Painter<'_>)); 2] = [
            ("fire_meter", &|p| p.fire_meter(area, 3, 8, lit(2.0))),
            ("fire_meter_upward", &|p| {
                p.fire_meter_upward(area, 3, 8, lit(2.0));
            }),
        ];
        for (name, draw) in sparse {
            let mut frame = Frame::new(GridSize::new(10, 5));
            draw(&mut frame.painter(area));
            let outside = (0..10)
                .flat_map(|col| (0..5).map(move |row| Pos::new(col, row)))
                .filter(|at| !area.contains(*at))
                .filter(|at| frame.cell(*at).is_some_and(|cell| cell.glyph != ' '))
                .count();
            assert_eq!(outside, 0, "{name} painted outside its rectangle");
        }
    }

    #[test]
    fn a_burning_meter_reads_the_same_value_as_a_plain_one() {
        // **The regression this file's own `meter_upward` doc warns about.** The
        // panel once re-derived the fill arithmetic and drew one instrument two
        // ways depending on which way the pane had split. A fire that computed
        // its own length would be the same defect wearing a costume: the bar
        // would say one thing with motion on and another with it off.
        //
        // Both take their length from `filled_of`, so this is a property rather
        // than a coincidence — and this is what keeps it one.
        // The two draw their empty halves differently — the plain meter lays
        // down a `░` track, the fire leaves it clear so the smoke has somewhere
        // to be — so the join is found as "the first cell that is not lit"
        // rather than by looking for one glyph in both.
        for total in [1u32, 7, 60, 600] {
            for done in [0, total / 3, total / 2, total - 1, total] {
                for tenths in [0u16, 3, 7, 13] {
                    let phase = f32::from(tenths) / 10.0;
                    let (plain, burning) = both(24, done, total, phase);

                    let edge = |row: &[char]| {
                        row.iter()
                            .position(|glyph| !matches!(glyph, '█' | '▓'))
                            .unwrap_or(row.len())
                    };
                    assert_eq!(
                        edge(&plain),
                        edge(&burning),
                        "{done}/{total} at phase {phase}: the fire moved the value",
                    );
                }
            }
        }
    }

    #[test]
    fn a_filling_bar_creeps_between_the_worlds_ticks() {
        // §5.0 turns the world at 1 Hz and reports whole ticks, so a meter read
        // straight off it moves once a second in one jump. A duration action is
        // continuous underneath that — the tick count is a *sample* — so the
        // crept value is closer to the truth than the sample is.
        //
        // The two ends have to be exact, or the bar drifts out of step with the
        // reading it is interpolating between: at zero it is the sim's own
        // answer, at one it is the answer the sim is about to give.
        for total in [6u32, 8, 60] {
            for done in 0..total {
                let (now, next) = (filled_of(32, done, total), filled_of(32, done + 1, total));
                assert_eq!(creeping(32, done, total, 0.0), now, "{done}/{total} at 0");
                assert_eq!(creeping(32, done, total, 1.0), next, "{done}/{total} at 1");

                // ...and it never runs backwards or overshoots in between.
                let mut last = now;
                for tenth in 0..=10u16 {
                    let at = creeping(32, done, total, f32::from(tenth) / 10.0);
                    assert!(at >= last && at <= next, "{done}/{total}: {at} out of band");
                    last = at;
                }
            }
        }
    }

    #[test]
    fn a_bowl_at_rest_does_not_creep() {
        // **The regression this file's own layer had no test for.** `grind`'s
        // `a_bowl_at_rest_is_perfectly_still` sweeps `phase` with `filled`
        // handed in directly — so it never reaches `creeping`, which lives up
        // here and is driven by `advance`, a different clock.
        //
        // `shell::panel` stands in a `0/1` meter for the states the sim reports
        // no quantity for, and `creeping` predicted `1/1` from it: **the whole
        // bar**. An idle bowl ground itself to completion and snapped back once
        // a second, and a fouled one read as `ready` for most of every second.
        let idle = grind::Grind {
            phase: 3.0,
            working: false,
            ..grind::Grind::default()
        };
        let area = Rect::new(0, 0, 2, 16);

        let mut first = Frame::new(GridSize::new(2, 16));
        first.painter(area).grind_meter_upward(area, 0, 1, idle);
        let settled = first.to_text();

        for tenth in 0..=10u16 {
            let mut frame = Frame::new(GridSize::new(2, 16));
            let advance = f32::from(tenth) / 10.0;
            frame
                .painter(area)
                .grind_meter_upward(area, 0, 1, grind::Grind { advance, ..idle });
            assert_eq!(
                frame.to_text(),
                settled,
                "a resting bowl moved {advance} of the way through a tick",
            );
        }
    }

    #[test]
    fn the_creep_cannot_run_past_a_finished_bar() {
        // The last tick is the one where predicting `done + 1` would read past
        // the end. `filled_of` clamps `done`, so the prediction is simply the
        // full bar and the creep has nowhere to go.
        for advance in [0.0, 0.5, 1.0] {
            assert_eq!(creeping(16, 8, 8, advance), 16);
            assert_eq!(creeping(16, 7, 8, 1.0), 16);
        }
    }

    #[test]
    fn a_fire_says_nothing_at_all() {
        // The whole basis on which `Depiction` is allowed to exist: it carries
        // no meaning, so it owes the linear stream nothing — and must therefore
        // *add* nothing to it. A frame that spoke differently with the fire on
        // would mean a listener's screen and a sighted player's had diverged,
        // which is the one thing §14 does not permit.
        //
        // Compared against the plain meter rather than against emptiness,
        // because `meter` is silent too and the point is that they match.
        let area = Rect::new(0, 0, 24, 1);

        let mut plain = Frame::new(GridSize::new(24, 1));
        plain.painter(area).meter(area, 5, 16, Style::NORMAL);

        let mut burning = Frame::new(GridSize::new(24, 1));
        burning.painter(area).fire_meter(area, 5, 16, lit(0.7));

        assert_eq!(
            plain.speech().to_transcript(),
            burning.speech().to_transcript(),
            "the fire spoke",
        );
        assert!(burning.speech().is_empty(), "a silent meter said something");
    }

    #[test]
    fn both_orientations_agree_about_where_the_fire_ends() {
        // `meter` fills rightward and `meter_upward` fills from the bottom, and
        // §10.1 picks between them by pane shape — so F4 must not change how
        // much fuel the athanor appears to have.
        let (across, up) = (Rect::new(0, 0, 16, 1), Rect::new(0, 0, 1, 16));

        let mut horizontal = Frame::new(GridSize::new(16, 1));
        horizontal
            .painter(across)
            .fire_meter(across, 5, 16, lit(0.4));
        let lit_across = (0..16)
            .filter(|col| {
                horizontal
                    .cell(Pos::new(*col, 0))
                    .is_some_and(|cell| cell.glyph == '█' || cell.glyph == '▓')
            })
            .count();

        let mut vertical = Frame::new(GridSize::new(1, 16));
        vertical.painter(up).fire_meter_upward(up, 5, 16, lit(0.4));
        let lit_up = (0..16)
            .filter(|row| {
                vertical
                    .cell(Pos::new(0, *row))
                    .is_some_and(|cell| cell.glyph == '█' || cell.glyph == '▓')
            })
            .count();

        assert_eq!(lit_across, lit_up, "the two orientations disagree");
        assert_eq!(lit_across, 5, "5 of 16 fuel should light 5 of 16 cells");
    }
}
