//! Records become cells here — and nowhere else.
//!
//! This module is the whole claim of architectural rule 4 made concrete:
//! *presentation is a view over the record*. Nothing here adds information. A
//! table decides widths, alignment, and which columns to show; it cannot invent
//! a value, and it cannot suppress one from the screen-reader stream, because
//! the linear form comes from [`Record::speak`] rather than from anything drawn.
//!
//! Three consequences fall out rather than being implemented:
//!
//! - **Linearised tables are correct by construction** (§14). Every row speaks
//!   `label: value` because the labels live on the record, so a view physically
//!   cannot forget them or reorder them away from their values.
//! - **A truncated column costs a sighted player and nobody else.** The row's
//!   speech is built before any width is applied, so a narrow pane is a visual
//!   constraint and never an informational one — the same rule
//!   [`Painter::span`] follows.
//! - **A missing field draws as a gap.** §8.1 names malformed record boundaries
//!   as a structural sabotage signature; a record short a field leaves a hole in
//!   its row without any special case being written for it.

use crate::geometry::{Pos, Rect};
use crate::paint::Painter;
use crate::record::field::FieldName;
use crate::record::kind::RecordKind;
use crate::record::stream::Record;
use crate::style::Style;

/// Cells between adjacent columns.
const COLUMN_GAP: usize = 2;

/// How a run of records becomes cells.
#[derive(Debug, Clone, Copy)]
pub struct RecordView<'a> {
    mode: Mode<'a>,
    reveal: Option<Reveal>,
}

/// How much of the newest output has arrived.
///
/// See [`RecordView::revealing`]. Counted in **characters of content**, so the
/// marker and the prompt do not consume the budget — they are drawn as soon as
/// any of their record is.
#[derive(Debug, Clone, Copy)]
struct Reveal {
    /// Records before this index in the drawn run are already whole.
    after: usize,
    /// Characters still to arrive.
    cells: u32,
}

#[derive(Debug, Clone, Copy)]
enum Mode<'a> {
    Table {
        columns: &'a [FieldName],
        header: bool,
    },
    Lines,
    Prompt {
        prompt: &'a str,
    },
}

impl<'a> RecordView<'a> {
    /// A column-aligned table over the named fields, with a header row.
    ///
    /// Fields a record does not carry are left blank; fields not named here are
    /// still spoken, because the row's speech is the whole record.
    #[must_use]
    pub const fn table(columns: &'a [FieldName]) -> Self {
        Self {
            mode: Mode::Table {
                columns,
                header: true,
            },
            reveal: None,
        }
    }

    /// The same table with no header row.
    #[must_use]
    pub const fn without_header(self) -> Self {
        match self.mode {
            Mode::Table { columns, .. } => Self {
                mode: Mode::Table {
                    columns,
                    header: false,
                },
                reveal: self.reveal,
            },
            Mode::Lines | Mode::Prompt { .. } => self,
        }
    }

    /// Draw only the first `cells` characters of everything after record
    /// `after`, as if it were arriving over a wire.
    ///
    /// Records before `after` are drawn whole; from there on the budget runs out
    /// mid-line and the rest of the run is blank. **Rows are unaffected** — a
    /// record still occupies exactly the height it will occupy when it finishes,
    /// so nothing below it moves as it arrives. A reveal that reflowed the pane
    /// would fight the caller's own arithmetic for which records fit.
    ///
    /// # It costs the sim nothing, and must not
    ///
    /// This is presentation: the records are all already there, and a caller
    /// that never calls this sees the finished screen. The reason that matters
    /// is that waiting on output is meant to be a reason to automate, and the
    /// moment it were a *modelled* cost the balance harness would have to
    /// simulate typewriter delays, offline catch-up would owe animation time,
    /// and a player who turns the animation off for motion or attention reasons
    /// would gain a competitive advantage — the inversion of §9's parity rule,
    /// where a setting must never become a difficulty choice.
    ///
    /// Speech is unaffected in a different way: a record announces only once it
    /// is **whole**, so a listener hears complete records in stream order and
    /// never half of one.
    #[must_use]
    pub const fn revealing(self, after: usize, cells: u32) -> Self {
        Self {
            reveal: Some(Reveal { after, cells }),
            ..self
        }
    }

    /// One line per record, every field in emit order, space separated.
    ///
    /// What a log pane and the orb's own speech want: prose and log lines carry
    /// their structure in their wording, not in their columns.
    #[must_use]
    pub const fn lines() -> RecordView<'static> {
        RecordView {
            mode: Mode::Lines,
            reveal: None,
        }
    }

    /// The command-line surface: a marker, then the line.
    ///
    /// [`RecordView::lines`] is not sufficient here. Every record the parser
    /// emits carries one canonical command form, so through a line view an
    /// unresolved input draws as a column of bare words and `meditate` draws as
    /// `meditate count` — neither of which is the prompt DESIGN.md §6 describes.
    ///
    /// This view draws two extra channels, both derived from the record and
    /// neither of them prose:
    ///
    /// - the [`Record::marker`] glyph, so a suggestion cannot be mistaken for
    ///   the command that will run;
    /// - the player's own [`RecordKind::Input`] lines with the shell prompt in
    ///   front of them, so the transcript reads as a session.
    ///
    /// Intensity comes from [`Record::style`], which derives it the same way.
    ///
    /// `prompt` is what the player's own lines are drawn behind — the wizard's
    /// name, which is world state rather than a rendering choice, so it arrives
    /// as an argument instead of living here as a constant.
    #[must_use]
    pub const fn prompt(prompt: &str) -> RecordView<'_> {
        RecordView {
            mode: Mode::Prompt { prompt },
            reveal: None,
        }
    }

    /// Rows `records` would need at `cols` wide, without drawing anything.
    ///
    /// A line view is no longer one row per record — a listing packs across the
    /// pane — so a caller that wants the *newest* records has to ask rather than
    /// count. Before this existed the transcript assumed one row each, and every
    /// packed listing left that many blank rows at the bottom of the pane while
    /// dropping the same number of records off the top.
    ///
    /// Unbounded in the record count, so a caller holding a long stream should
    /// pass a window rather than the whole of it.
    #[must_use]
    pub fn height<'r>(&self, cols: u16, records: impl Iterator<Item = Record<'r>> + Clone) -> u16 {
        let prompt = match self.mode {
            // A table is one row per record plus the header, always.
            Mode::Table { header, .. } => {
                let rows = to_cols(records.count());
                return if header { rows.saturating_add(1) } else { rows };
            }
            Mode::Lines => None,
            Mode::Prompt { prompt } => Some(prompt),
        };

        let indent = indent_for(prompt);
        let (mut rows, mut rest) = (0u16, records);
        let mut at_run_start = true;
        let mut first = true;
        loop {
            let run = rest.clone();
            let Some(record) = rest.next() else { break };
            // The blank row `draw_lines` puts before each command but the first.
            // Measured here too, or the caller's "which records fit" arithmetic
            // disagrees with what is drawn and the pane scrolls by a row a frame.
            if opens_with_a_gap(record.kind()) && !first {
                rows = rows.saturating_add(1);
            }
            first = false;
            // Only ever planned at a run's first record. A run that declines
            // declines for its whole length, and re-asking at each of its records
            // would walk the remainder every time — quadratic in the run, on a
            // path that runs per frame.
            if at_run_start
                && record.kind().tiles()
                && let Some(plan) = Tiling::plan(cols, indent, run)
            {
                for _ in 1..plan.count {
                    rest.next();
                }
                rows = rows.saturating_add(plan.rows());
                continue;
            }
            at_run_start = !record.kind().tiles();
            rows = rows.saturating_add(wrapped_rows(&record, cols, prompt));
        }
        rows
    }

    /// Draw `records` into `area`, returning the number of rows used.
    ///
    /// The iterator must be `Clone` because a table measures its columns in one
    /// pass and draws them in a second; that is what keeps a per-frame `Vec` of
    /// rows out of the render path.
    pub fn draw<'r>(
        &self,
        painter: &mut Painter<'_>,
        area: Rect,
        records: impl Iterator<Item = Record<'r>> + Clone,
    ) -> u16 {
        let mut painter = painter.sub(area);
        let area = painter.area();
        if area.is_empty() {
            return 0;
        }
        match self.mode {
            // A table has no reveal. Its rows are a readout of state rather than
            // output arriving, and a half-drawn column of numbers reads as a bug.
            Mode::Table { columns, header } => {
                draw_table(&mut painter, area, columns, header, records)
            }
            Mode::Lines => draw_lines(&mut painter, area, records, None, self.reveal),
            Mode::Prompt { prompt } => {
                draw_lines(&mut painter, area, records, Some(prompt), self.reveal)
            }
        }
    }
}

/// How much of a record may be drawn on this pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arrival {
    /// All of it, and it may speak.
    Whole,
    /// The first `n` characters, and it stays silent until it is whole.
    Partial(u32),
}

impl Arrival {
    /// What is left of `text` under this budget, and how much of the budget it
    /// used.
    fn clip(self, text: &str) -> (&str, u32) {
        match self {
            Self::Whole => (text, 0),
            Self::Partial(cells) => {
                let visible = crate::arriving(text, cells);
                let used = if visible.len() == text.len() {
                    to_cells(text.chars().count())
                } else {
                    cells
                };
                (visible, used)
            }
        }
    }
}

/// A character count as a reveal budget, saturating rather than wrapping.
fn to_cells(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// Cells reserved in front of a line for its marker.
const MARKER_WIDTH: u16 = 2;

fn draw_table<'r>(
    painter: &mut Painter<'_>,
    area: Rect,
    columns: &[FieldName],
    header: bool,
    records: impl Iterator<Item = Record<'r>> + Clone,
) -> u16 {
    if columns.is_empty() {
        return 0;
    }

    let mut widths: Vec<usize> = columns
        .iter()
        .map(|column| {
            if header {
                column.label().chars().count()
            } else {
                0
            }
        })
        .collect();
    for record in records.clone() {
        for (width, column) in widths.iter_mut().zip(columns) {
            if let Some(value) = record.field(*column) {
                *width = (*width).max(value.width());
            }
        }
    }

    let mut row = area.row;
    if header {
        // Drawn silently. Every row below speaks its own `label: value` pairs
        // (§14), so the header carries no information a reader is missing — and
        // announcing it would make every table open with a redundant recital of
        // words the listener is about to hear on each line anyway.
        for (index, column) in columns.iter().enumerate() {
            if let Some(at) = column_start(area, &widths, index) {
                painter.glyphs(at, column.label(), Style::DIM);
            }
        }
        row = row.saturating_add(1);
    }

    let mut speech = String::new();
    for record in records {
        if row >= area.bottom() {
            break;
        }
        speech.clear();
        record.speak(&mut speech);
        painter.announce(record.kind().utterance(), record.role(), &speech);

        let style = record.style();
        for (index, column) in columns.iter().enumerate() {
            let Some(value) = record.field(*column) else {
                continue;
            };
            let Some(at) = column_start(Rect { row, ..area }, &widths, index) else {
                continue;
            };
            // Numbers align right within their column; text aligns left. The
            // view can make that choice only because the record kept the number
            // a number instead of emitting it pre-rendered.
            let at = if value.is_numeric() {
                let pad = widths[index].saturating_sub(value.width());
                Pos::new(at.col.saturating_add(to_cols(pad)), at.row)
            } else {
                at
            };
            value.with_str(|text| painter.glyphs(at, text, style));
        }
        row = row.saturating_add(1);
    }
    row.saturating_sub(area.row)
}

fn draw_lines<'r>(
    painter: &mut Painter<'_>,
    area: Rect,
    records: impl Iterator<Item = Record<'r>> + Clone,
    prompt: Option<&str>,
    reveal: Option<Reveal>,
) -> u16 {
    let (mut drawn, mut speech) = (String::new(), String::new());
    let mut row = area.row;
    let mut rest = records;
    // See `RecordView::height`: a declined run must not be re-planned at each of
    // its records, or the measuring is quadratic in the run's length.
    let mut at_run_start = true;
    // The reading column, for as long as a run of readings lasts.
    let (mut readings, mut at_readings_start) = (None::<Readings>, true);
    let (mut index, mut budget) = (0usize, reveal.map(|reveal| reveal.cells));
    loop {
        if row >= area.bottom() {
            break;
        }
        // Everything before the reveal's starting record is already on screen;
        // from there on, whatever budget is left. Rows are unaffected either
        // way — see `RecordView::revealing`.
        let arrival = match (reveal, budget) {
            (Some(reveal), Some(left)) if index >= reveal.after => Arrival::Partial(left),
            _ => Arrival::Whole,
        };
        // Cloned before the record is taken, so a tiling run can be measured
        // from its own first row rather than needing a second pass over the
        // stream or a `Vec` in the render path.
        let run = rest.clone();
        let Some(record) = rest.next() else { break };

        // The sub-area is what is *left* of the pane, not the pane moved down.
        // `Rect { row, ..area }` keeps the full row count, so `bottom()` slides
        // with the run and a listing starting partway down draws past the pane —
        // where `Painter` clips the cells but not the speech, and a reader hears
        // rows nobody can see.
        let remaining = Rect {
            row,
            rows: area.bottom().saturating_sub(row),
            ..area
        };
        if at_run_start
            && record.kind().tiles()
            && let Some((rows, packed, spent)) =
                draw_tiled(painter, remaining, prompt, run, arrival)
        {
            // `record` was the first of them.
            for _ in 1..packed {
                rest.next();
            }
            row = row.saturating_add(rows);
            index += packed;
            if let Some(left) = budget.as_mut() {
                *left = left.saturating_sub(spent);
            }
            continue;
        }
        // **Planned once per run, at its first record.** The same rule
        // `Tiling::plan` follows and for the same reason: re-measuring the
        // remainder at every record is quadratic in the run, on a path that
        // runs per frame. A run that is not a run of readings plans `None` and
        // stays `None` until a record of another kind ends it.
        if record.kind() == RecordKind::Status {
            if readings.is_none() && at_readings_start {
                readings = Readings::plan(std::iter::once(record).chain(rest.clone()));
            }
            at_readings_start = false;
        } else {
            readings = None;
            at_readings_start = true;
        }

        at_run_start = !record.kind().tiles();

        // A blank row before each command but the first, so one exchange does
        // not run into the next. The transcript is a wall of short lines and the
        // prompt is the only thing separating them; without this, reading back
        // three commands means finding the prompts by eye.
        //
        // Drawn rather than spoken: §14's stream is whole records in order, and
        // an utterance for "nothing" is noise a listener cannot skip.
        if opens_with_a_gap(record.kind()) && index > 0 {
            row = row.saturating_add(1);
            if row >= area.bottom() {
                break;
            }
        }
        index += 1;

        drawn.clear();
        // Content only: an annotation drawn as text would put an internal token
        // — `"resolved survey"` — on screen for a player to read.
        //
        // **The brackets are the view's, not the record's.** A section carries
        // the bare word so `sift reagent` finds it and a screen reader hears a
        // heading rather than punctuation; `[reagent]` is how this surface draws
        // one, and drawing is what a view is for (rule 4).
        // **A listing keeps its `=` even when it is not tiled.** A run of one
        // does not pack — and scrolling clips a run, so the last visible entry
        // of a listing was stacking and coming out as `charcoal ∞` while the
        // rows above it read `charcoal = ∞`. One entry of a table is still a row
        // of that table. `drawn_form` is what `wrapped_rows` measures.
        drawn.push_str(&drawn_form(&record));
        // **A run of readings gets its values in a column** — §19 asked for this
        // outright: *"a status row wants its value column aligned with the one
        // above it."* `tick 4` over `concentration 0` is two numbers in two
        // places, and the whole reason to print six of them together is to be
        // read down.
        //
        // Leaders here and nowhere else in the transcript, because this is the
        // one listing whose far column is **right**-aligned — which is the only
        // shape a leader suits. Against a left-aligned column the run length is
        // set by the near column's raggedness and the eye lands on the dots;
        // that is why the verb listing takes a plain gap and this does not.
        //
        // **The row count cannot change**, which is what makes this safe to do
        // in the draw alone: a reading is one row before and after, and the
        // padded form is refused outright if it would not fit on one. So
        // `wrapped_rows` measuring the unpadded string still gets the right
        // answer, and the height/draw agreement §19 keeps recording defects
        // against is not touched.
        if let Some(plan) = readings.as_ref() {
            let width = wrap_width(area.cols, lead_for(&record, prompt), record.kind());
            if let Some(aligned) = plan.row(&record, width) {
                drawn.clear();
                drawn.push_str(&aligned);
            }
        }
        speech.clear();
        record.speak(&mut speech);

        let (visible, spent) = arrival.clip(&drawn);
        if let Some(left) = budget.as_mut() {
            *left = left.saturating_sub(spent);
        }
        // Whether *this record* finished, not whether a reveal is running: a
        // budget large enough to cover the line leaves it as complete as no
        // budget at all, and it must speak like one.
        let complete = visible.len() == drawn.len();
        // The row is still consumed. A record that has not started arriving
        // occupies the space it will fill, so nothing below it shifts as it
        // does — and the caller's arithmetic for which records fit stays true.
        if !complete && visible.is_empty() {
            row = row.saturating_add(1);
            continue;
        }

        let style = record.style();
        let mut col = area.col;
        if let Some(prompt) = prompt {
            // The marker and the prompt are drawn silently. Both restate what
            // the linear stream already carries — an utterance's kind says it
            // is `Input`, and a reader filtering by outcome reads the
            // annotation — so speaking them would be saying it twice.
            if record.kind() == RecordKind::Input {
                col = col.saturating_add(painter.glyphs(Pos::new(col, row), prompt, style));
            } else if record.kind() == RecordKind::Section {
                // No marker and no lead: see `lead_for`.
            } else {
                if let Some(marker) = record.marker() {
                    painter.glyphs(Pos::new(col, row), marker.encode_utf8(&mut [0; 4]), style);
                }
                col = col.saturating_add(MARKER_WIDTH);
            }
        }

        // Wrapped, not clipped — see `wrapped_rows`, which measures with the same
        // iterator so the two cannot disagree about the row cost.
        //
        // **The whole record speaks once, on its first row.** §14's stream is
        // whole records in stream order; a listener hearing one utterance per
        // wrapped fragment would have to reassemble a sentence the screen shows
        // whole, and would hear a *different number of things* depending on how
        // wide the window happens to be. Continuations draw silently.
        let width = wrap_width(area.cols, lead_for(&record, prompt), record.kind());
        let mut spoke = false;
        for fragment in crate::wrap::Wrap::new(visible, width) {
            if row >= area.bottom() {
                break;
            }
            let at = if spoke {
                col.saturating_add(CONTINUATION)
            } else {
                col
            };
            if complete && !spoke {
                let mut span = crate::span::Span::new(fragment)
                    .with_style(style)
                    .with_kind(record.kind().utterance())
                    // The *whole* line, however it was broken up to fit.
                    .with_spoken(&speech);
                // The marker and the intensity are both silent channels. Tagging
                // the utterance is what stops `xyzzy` from linearising as three
                // identical lines, with a listener unable to tell the error from
                // the offers.
                if let Some(outcome) = record.outcome() {
                    span = span.with_outcome(outcome);
                }
                let drawn = painter.span(Pos::new(at, row), &span);
                // **A heading is ruled off, on its own row.** Beside the words
                // rather than under them, so the hierarchy costs no row on a
                // page that already fills the floor exactly.
                //
                // Drawn here rather than in `drawn_form` because a rule's length
                // is the *pane's* and `drawn_form` returns a width-independent
                // string that `wrapped_rows` measures. Putting it there would
                // make the measure and the draw disagree, which §19 records
                // scrolling the transcript by a row a frame.
                //
                // Silent, like every rule in the game: a reader hearing a run of
                // box-drawing between sections gets noise where a sighted player
                // gets separation for free. The heading's own span already said
                // what the section is.
                if record.kind() == RecordKind::Section {
                    let after = at.saturating_add(drawn).saturating_add(1);
                    let stop = area.col.saturating_add(rule_stop(area.cols));
                    if stop > after {
                        painter.rule(Pos::new(after, row), stop.saturating_sub(after), Style::DIM);
                    }
                }
            } else {
                // `glyphs` rather than a `Span` with empty speech: an empty
                // override means *no override*, so the span would announce its
                // visible text and a listener would hear a prefix that grows
                // every frame. §14's stream is whole records in stream order — a
                // record announces when it finishes arriving, and stays silent
                // until then.
                painter.glyphs(Pos::new(at, row), fragment, style);
            }
            spoke = true;
            row = row.saturating_add(1);
        }
        if !spoke {
            row = row.saturating_add(1);
        }
    }
    row.saturating_sub(area.row)
}

/// Cells a marked line's text is inset by, which a listing matches so a pane
/// does not appear to change its left edge partway down.
const fn indent_for(prompt: Option<&str>) -> u16 {
    if prompt.is_some() { MARKER_WIDTH } else { 0 }
}

/// Rows a record's drawn line takes once wrapped.
///
/// **A line view wraps rather than clips.** It used to draw one row per record
/// and cut whatever did not fit, which is silent data loss on the surface §14
/// calls the game's primary output: a refusal naming two long reagents lost its
/// verb, and `grimoire`'s own instructions — the one command whose entire job is
/// telling a player what to type — lost the thing to type. A pane gives about 46
/// cells once its border and §10.1's instrument panel are taken out, and
/// `sage-tincture + ground-salt -> clarified-draught` is 47.
///
/// Measured here and drawn by [`draw_lines`] from the same [`crate::Wrap`], so the two
/// cannot disagree about how many rows a record costs — the failure that left the
/// transcript with blank rows at the bottom while it dropped history off the top.
fn wrapped_rows(record: &Record<'_>, cols: u16, prompt: Option<&str>) -> u16 {
    // The same three arguments the draw uses, so the measure cannot narrow a
    // record the draw sets wide.
    let width = wrap_width(cols, lead_for(record, prompt), record.kind());
    if width == 0 {
        return 1;
    }
    // **Measured as it is drawn, not as `to_line` renders it.** `draw_lines`
    // wraps a `Section` in `[…]` and binds a counted `Entry` with ` = `, each
    // two cells wider than the plain join — so a row within two cells of the
    // wrap point measured one row and drew two, and the binary search above
    // picked a skip whose measured height fit while the drawn one overflowed,
    // pushing the newest record off the bottom of the pane and out of §14's
    // stream. Exactly the failure `wrapped_rows` was written to prevent.
    let line = drawn_form(record);
    let rows = crate::wrap::Wrap::new(&line, width).count();
    u16::try_from(rows).unwrap_or(u16::MAX).max(1)
}

/// A record's text exactly as the stacked path draws it.
///
/// One definition, so the measure and the draw cannot disagree about two
/// characters — see [`wrapped_rows`].
fn drawn_form(record: &Record<'_>) -> String {
    if record.kind() == RecordKind::Section {
        // **The brackets are gone, and the rule replaced them.** `[reagent]`
        // read as a config file — which is the whole complaint this pass
        // answers — and brackets are a weak heading anyway: they say *this is
        // set apart* without saying what it is set apart from. A rule to a
        // fixed column says it, and costs no row because it is drawn beside the
        // heading rather than under it. See `draw_lines`.
        //
        // It stays the view's decision either way: the record carries the bare
        // word so `sift reagent` finds it and a reader hears a heading rather
        // than punctuation.
        let mut out = String::new();
        record.write_line(&mut out);
        return out;
    }
    amount_of(record).unwrap_or_else(|| record.to_line())
}

/// `attend` + `place` → `attend <place>`, the form a listing shows a verb in.
///
/// **Composed here, not in the sim.** A slot's brackets are a drawing decision
/// exactly as `[reagent]` was (rule 4): the record carries `attend` and `place`
/// as separate fields so `sift attend` finds one and a reader hears two labelled
/// values, and this is how the surface draws them.
///
/// The brackets are doing real work rather than decorating. Tiled bare, a
/// two-word entry runs into its neighbour — `distil reagent  kindle reagent` —
/// because the gap between entries is the same two spaces as the gap *inside*
/// one. `>` terminates the cell, so the eye finds the boundary. It is also the
/// spelling `recall scripting` already teaches for `repeat <count>`.
/// **A shape that brackets its own slots is not bracketed again.** A verb's
/// record carries one bare slot name (`attend` + `place`), so this supplies the
/// pair; a *control word* carries a whole shape from `SpellWord::shape`, and
/// those spell their own — `<name> be <place>`, `each <set>`, `<name>(...)`.
/// Wrapping them drew `let <<name> be <place>>` and `wait <<thing>>`, the one
/// double bracket in the game, three rows above `queue <name>` on the same page
/// for the comparison.
///
/// **The test is "does it contain a `<`", not "is it wrapped in one".** Two of
/// the shapes are neither — `each <set>` opens with a word and `<name>(...)`
/// closes with a paren — so anything narrower leaves half of them doubled, which
/// is how the first attempt at this went.
///
/// The rule above is preserved rather than excepted: a cell has to end in
/// something the eye can find, or a tiled run reads as one entry. `>` does it,
/// and so does `)`.
fn signature_of(record: &Record<'_>) -> String {
    let mut out = String::new();
    if let Some(name) = record.field(FieldName::Name) {
        name.write(&mut out);
    }
    if let Some(slot) = record.field(FieldName::Kind) {
        let mut drawn = String::new();
        slot.write(&mut drawn);
        out.push(' ');
        if drawn.contains('<') {
            out.push_str(&drawn);
        } else {
            out.push('<');
            out.push_str(&drawn);
            out.push('>');
        }
    }
    out
}

/// Whether every record of a run carries a description.
///
/// A run is described or it is not; a half-described one would draw some rows
/// with a second column and some without, which is the ragged shape §3 reserves
/// for sabotage.
fn all_described<'r>(run: impl Iterator<Item = Record<'r>>) -> bool {
    let mut any = false;
    for record in run.take_while(|record| record.kind().tiles()) {
        if record.field(FieldName::Detail).is_none() {
            return false;
        }
        any = true;
    }
    any
}

/// Whether a record of this kind opens with a blank row above it.
///
/// **Two kinds, one rule, and it must be asked in both `height` and
/// `draw_lines`.** A blank row is the cheapest separator a fixed grid has —
/// stronger than an indent, cheaper than a rule, and the first thing to reach
/// for before either. `Input` has had one since the transcript existed, so one
/// exchange does not run into the next.
///
/// `Section` earns the same for the same reason one level down: a ruled heading
/// says *a new thing starts here*, and a heading pressed against the last row of
/// the previous listing says it while looking like part of it. The rule and the
/// gap answer different halves — the rule names the boundary, the gap gives the
/// eye somewhere to land.
///
/// **Not free, and worth saying what it costs.** The laboratory's page carries
/// eight sections, so it is seven more rows on a page that is already longer than
/// the 80×22 floor's nineteen — which stopped being an exact fit when the listing
/// gained descriptions. §19 records that the fit is gone; this makes a scrolling
/// page longer rather than breaking one that fitted.
const fn opens_with_a_gap(kind: RecordKind) -> bool {
    matches!(kind, RecordKind::Input | RecordKind::Section)
}

/// The column a section's rule runs to, given the pane.
///
/// **A fixed stop, not the pane edge, and this was measured rather than
/// picked.** Drawn to the edge the rules were the strongest marks on screen and
/// the content went to mush — the squint test's classic failure. Stopping every
/// rule at the same column makes the headings read as a *set* and leaves the
/// content dominant.
///
/// Never more than half the pane, so a narrow transcript does not end up mostly
/// rule: 34 cells at the laboratory's 86, 31 at the 80x22 floor's 62.
const fn rule_stop(cols: u16) -> u16 {
    const FURTHEST: u16 = 34;
    let half = cols / 2;
    if half < FURTHEST { half } else { FURTHEST }
}

/// Cells before a record's own text begins.
///
/// **Per record, because an `Input` starts after the whole prompt** and
/// everything else after the marker. Measuring both at the marker width — which
/// is what a single `indent` did — makes `height` believe a typed line has a
/// dozen more cells than it draws into, so a long command measures one row and
/// draws two. The pane then scrolls by a row a frame, which is the failure the
/// tiling plan is shared to prevent and would have been reintroduced here.
fn lead_for(record: &Record<'_>, prompt: Option<&str>) -> u16 {
    match prompt {
        None => 0,
        Some(prompt) if record.kind() == RecordKind::Input => {
            u16::try_from(prompt.chars().count()).unwrap_or(u16::MAX)
        }
        // **A heading sits at the margin, so what follows reads as under it.**
        // Every record took the same lead, which put `[what it does]` flush with
        // its own body and made a manual page a wall of text rather than
        // sections. Outdenting the heading is the same shape as indenting the
        // content and costs no cells; `survey`'s `[place]` gets it too.
        Some(_) if record.kind() == RecordKind::Section => 0,
        Some(_) => MARKER_WIDTH,
    }
}

/// Cells a wrapped line may use, leaving room for a continuation's indent.
///
/// Every row of a record is wrapped to the *same* width, including the first, so
/// [`wrapped_rows`] and [`draw_lines`] cannot count differently. The first row
/// gives up [`CONTINUATION`] cells it could have used; that is the price of the
/// two staying in step, and it buys the indent that makes a wrapped line read as
/// part of the line above rather than as a new one.
const fn wrap_width(cols: u16, lead: u16, kind: RecordKind) -> u16 {
    let available = cols.saturating_sub(lead).saturating_sub(CONTINUATION);
    // **Prose takes a measure; everything else takes the pane.**
    //
    // A line past about seventy characters stops scanning and starts being
    // *searched*: the eye loses its place on the return sweep, which is why
    // typography has put the comfortable measure at 45-75 characters since long
    // before anyone had a terminal. The transcript is 86 cells in the laboratory
    // and 102 in the grimoire, so the widest room was setting prose 40% past the
    // top of that range.
    //
    // **Only `Message`.** The three diagnostic surfaces — a log, a script
    // listing, a schedule — are `is_diagnostic` precisely because their job is
    // *fidelity*: a `.spell` line wrapped at 68 inside an 86-cell pane would be
    // a line the game had reformatted, on the one surface where what is on
    // screen must be what is in the file. Listings are columns rather than
    // sentences and have their own arithmetic. A `Section` never reaches this
    // measure because no heading is nineteen characters, let alone sixty-eight.
    //
    // At the 80x22 floor the body is 62 cells, so this never binds there: it
    // narrows the wide rooms and leaves the tight one exactly as it was.
    if matches!(kind, RecordKind::Message) && available > MEASURE {
        MEASURE
    } else {
        available
    }
}

/// A run of readings, measured so their values share a column.
///
/// # Why this is not `Tiling`, and not a widening of `tiles()`
///
/// A reading is a **sequence**: `tick` then `seed` then `experience`, in an
/// order the emitter chose. `RecordKind::tiles` is the *unordered* question and
/// `only_unordered_rows_tile` pins it to `Entry` alone, so widening it to reach
/// this would say something false about what a `Status` row is.
///
/// # The shape test is strict, and every clause of it earns its place
///
/// `RecordKind::Status` has five emit sites and only one of them is the `status`
/// command. The cold-start report is a **contiguous run of seven** drawn in the
/// transcript at every launch — `orb` carries a tick and a state, each domain
/// carries a state, `bound` carries neither — and `verify` carries a source and
/// a state and no quantity at all. A plan that fired on those would put
/// `cold start` and `ok` under a column headed by a number.
///
/// So: two or more records, every one of them carrying **exactly** a name and a
/// numeric quantity. The boot report is mixed and fails on its first row; a
/// `verify` is a run of one.
#[derive(Debug, Clone, Copy)]
struct Readings {
    /// Cells the widest name takes.
    name: usize,
    /// Cells the widest value takes, so the digits end in one column.
    value: usize,
}

impl Readings {
    /// The plan for a run starting here, or `None` if it is not a run of readings.
    fn plan<'r>(run: impl Iterator<Item = Record<'r>>) -> Option<Self> {
        let (mut name, mut value, mut count) = (0usize, 0usize, 0usize);
        let mut text = String::new();
        for record in run.take_while(|record| record.kind() == RecordKind::Status) {
            let mut fields = record.fields();
            let (Some((FieldName::Name, label)), Some((FieldName::Quantity, amount)), None) =
                (fields.next(), fields.next(), fields.next())
            else {
                return None;
            };
            if !amount.is_numeric() {
                return None;
            }
            text.clear();
            label.write(&mut text);
            name = name.max(text.chars().count());
            text.clear();
            amount.write(&mut text);
            value = value.max(text.chars().count());
            count += 1;
        }
        // One reading is not a column, and padding it would only move it right.
        (count >= 2).then_some(Self { name, value })
    }

    /// One reading, laid out — or `None` if it would not fit on its row.
    ///
    /// Refusing rather than wrapping is what keeps the row count identical to
    /// the unpadded form the measure counted.
    fn row(&self, record: &Record<'_>, width: u16) -> Option<String> {
        let mut fields = record.fields();
        let (Some((FieldName::Name, label)), Some((FieldName::Quantity, amount))) =
            (fields.next(), fields.next())
        else {
            return None;
        };
        let (mut name, mut value) = (String::new(), String::new());
        label.write(&mut name);
        amount.write(&mut value);

        // ` name ....... value `: one space either side of the leader so the
        // dots never touch either, which is what `post.rs` already does and the
        // whole reason a leader is legible at all.
        let dots = (self.name + 2 + 1).saturating_sub(name.chars().count() + 1);
        let mut out = name;
        out.push(' ');
        for _ in 0..dots {
            out.push('.');
        }
        out.push(' ');
        for _ in value.chars().count()..self.value {
            out.push(' ');
        }
        out.push_str(&value);
        (out.chars().count() <= usize::from(width)).then_some(out)
    }
}

/// The widest a sentence is set, whatever the pane can hold.
///
/// Sixty-eight, from the middle of typography's 45-75 band. Not a pane width
/// and not derived from one — the surplus becomes margin, which is what a wide
/// room buys instead of a longer line.
const MEASURE: u16 = 68;

/// How far a wrapped line's continuations are indented.
const CONTINUATION: u16 = 2;

/// How a run of [tiling](RecordKind::tiles) records packs across a pane.
///
/// Split out from the drawing so [`RecordView::height`] and [`draw_tiled`]
/// cannot disagree about it. They did not have to: a pane that measures one way
/// and draws another leaves blank rows at the bottom while dropping history off
/// the top, which is what the transcript did the first time this was looked at.
#[derive(Debug, Clone, Copy)]
struct Tiling {
    indent: u16,
    /// Cells from the start of one column to the start of the next.
    stride: u16,
    per_row: u16,
    count: usize,
    /// Width of the name column, when the run reads as `name = value`.
    ///
    /// `None` when nothing in the run has a second field — a listing of places
    /// is names and nothing else, and an ` = ` with nothing after it would be a
    /// column of punctuation.
    named: Option<u16>,
    /// One entry per row, with what it does beside it.
    ///
    /// **Tiling suits a set of single words and fails for phrases.** `survey`
    /// lists names — `sage`, `alembic` — and packing them is right. `help` lists
    /// *grammar*, and two words in a cell run into the two words in the next.
    /// So a run whose records carry a description takes one row each and gets a
    /// second column, and a run without one still tiles.
    ///
    /// It rides the tiling plan rather than being a second mechanism because
    /// that plan exists *specifically* so `height` and `draw` cannot disagree
    /// about a run — the property a parallel path would have to re-earn.
    describing: bool,
}

/// A listing entry written as `name = amount`, if it is one.
///
/// `None` for everything else, which is every record that is not a row of a
/// counted listing — those keep the plain juxtaposition [`Record::write_line`]
/// gives them.
fn amount_of(record: &Record<'_>) -> Option<String> {
    if record.kind() != RecordKind::Entry {
        return None;
    }
    let amount = record.field(FieldName::Quantity)?;
    let (_, name) = record.content().next()?;

    let mut out = String::new();
    name.write(&mut out);
    out.push_str(BINDS);
    amount.write(&mut out);
    // **A listed thing may also be in a *state*, and it has to be drawn.** The
    // arsenal says how well stocked the tower is in each row — `fresh`, `thin`,
    // `spent` — and a column that reached the linear stream but never the pane
    // is §14 backwards: the screen-reader user informed and the sighted player
    // not. It rides after the amount because the amount is what the row is
    // *about*, and the state is a note on it.
    if let Some(state) = record.field(FieldName::State) {
        out.push_str(STATES);
        state.write(&mut out);
    }
    Some(out)
}

/// What sits between a tiled entry's name and its value.
///
/// Spaced, so the `=` never touches either — the whole point of aligning the
/// column is that the eye can run down it.
const BINDS: &str = " = ";

/// What sits between a tiled entry's value and the state it is in.
///
/// One space and no glyph. `warding = 5 thin` reads as a note on the row;
/// a second `=` would claim the state is another value bound to the name, which
/// is the category error [`DESCRIBES`] records one constant down.
const STATES: &str = " ";

/// What sits between an entry and what it does.
///
/// Two spaces and no glyph. The second column is a *sentence about* the first,
/// not a value bound to it — an `=` would say `attend = go somewhere`, which is
/// the same category error §19 records for `stop = place`.
const DESCRIBES: &str = "  ";

impl Tiling {
    /// Plan the run beginning at `run`'s first record, or `None` to stack.
    ///
    /// The run ends at the first record that does not tile; `run` may continue
    /// past it.
    ///
    /// # Why it can decline
    ///
    /// A tiled row has no room for a per-record [marker](Record::marker), and a
    /// marker is information — the difference between a candidate and the
    /// command that will run. Rather than drop it, a run carrying any marker
    /// declines to tile and falls back to one record per line, where the marker
    /// column exists. Nothing that reaches here today carries one; this is what
    /// keeps that true by construction instead of by convention.
    fn plan<'r>(
        cols: u16,
        indent: u16,
        run: impl Iterator<Item = Record<'r>> + Clone,
    ) -> Option<Self> {
        let available = cols.saturating_sub(indent);
        if available == 0 {
            return None;
        }

        // **Measured field by field, not as a rendered line.** A listing reads
        // as `name = value`, and the `=` only lines up if every tile puts its
        // name in a column of one width and its value in a column of another.
        // Measuring the joined text gives one width for the pair, which packs
        // them tightly and leaves the eye nothing to run down.
        let (mut name_wide, mut value_wide, mut count) = (0usize, 0usize, 0usize);
        // The whole rendered line, for the runs that are not `name = amount`.
        // **Measuring only the name would set the stride too narrow for what is
        // actually drawn**, and the tiles would overlap: the cold-launch verb
        // listing came out as `attend plasurvey plaperuse filsift` — every entry
        // truncated by its neighbour.
        let mut line_wide = 0usize;
        let mut drawn = String::new();
        for record in run.clone().take_while(|record| record.kind().tiles()) {
            if record.marker().is_some() {
                return None;
            }
            let mut fields = record.content();
            let Some((_, name)) = fields.next() else {
                continue;
            };
            drawn.clear();
            name.write(&mut drawn);
            name_wide = name_wide.max(drawn.chars().count());
            // **An amount, specifically** — not "whatever the second field is".
            // A cold launch lists the verbs as `stop place`, meaning *stop takes
            // a place*, and binding those with an `=` turns a grammar into an
            // assignment: `stop = place` says the two are the same thing.
            //
            // How many of something there are is the one relation `=` reads
            // correctly, so that is the one it is used for. Everything else
            // keeps the juxtaposition it had.
            if let Some(value) = record.field(FieldName::Quantity) {
                drawn.clear();
                value.write(&mut drawn);
                // **The state is measured with the value it follows**, because
                // the draw writes them together — a stride set to the value
                // alone would be short by the width of `spent` and the tiles
                // would overlap, which is the `attend plasurvey` defect §19
                // records arriving by a third door.
                if let Some(state) = record.field(FieldName::State) {
                    drawn.push_str(STATES);
                    state.write(&mut drawn);
                }
                value_wide = value_wide.max(drawn.chars().count());
            }
            drawn.clear();
            // **`signature_of`, not `write_line`, and it must match the draw
            // below exactly.** A tiled verb draws as `attend <place>`; measuring
            // it as `attend place` would set the stride two cells short and the
            // tiles would overlap — the `attend plasurvey plaperuse filsift`
            // defect §19 already records, arriving by a different door.
            drawn.push_str(&signature_of(&record));
            line_wide = line_wide.max(drawn.chars().count());
            count += 1;
        }
        // One name is not a listing, and packing it would only move it right.
        if count < 2 || line_wide == 0 {
            return None;
        }

        // **A described run: one per row, and the signature column pinned to the
        // run's widest.** Planned here rather than anywhere else so the padding
        // the draw applies is the padding the measure counted.
        if all_described(run.clone()) {
            let signature = run
                .take_while(|record| record.kind().tiles())
                .map(|record| signature_of(&record).chars().count())
                .max()
                .unwrap_or(0);
            return Some(Self {
                indent,
                stride: available,
                per_row: 1,
                count,
                named: Some(to_cols(signature)),
                describing: true,
            });
        }

        let named = (value_wide > 0).then(|| to_cols(name_wide));
        let widest = if value_wide > 0 {
            name_wide + BINDS.len() + value_wide
        } else {
            line_wide
        };
        let stride = to_cols(widest.saturating_add(COLUMN_GAP));
        let per_row = available / stride;
        // Nothing gained, and stacking keeps the fallback in one place.
        //
        // **A tile never wraps.** `per_row` is a whole number of strides, and a
        // stride is the widest entry in the run — so an entry either has its own
        // column or the run stacks. There is no arithmetic here that can put
        // half a name at the end of a line.
        (per_row >= 2).then_some(Self {
            indent,
            stride,
            per_row,
            count,
            named,
            describing: false,
        })
    }

    /// Rows the whole run needs.
    fn rows(self) -> u16 {
        to_cols(self.count.div_ceil(usize::from(self.per_row)))
    }
}

/// Draw a tiling run, returning the rows used, the records drawn, and how much
/// of `arrival`'s reveal budget it spent.
fn draw_tiled<'r>(
    painter: &mut Painter<'_>,
    area: Rect,
    prompt: Option<&str>,
    run: impl Iterator<Item = Record<'r>> + Clone,
    arrival: Arrival,
) -> Option<(u16, usize, u32)> {
    let plan = Tiling::plan(area.cols, indent_for(prompt), run.clone())?;
    let (indent, stride, per_row) = (plan.indent, plan.stride, plan.per_row);
    let tiling = run.take_while(|record| record.kind().tiles());

    let (mut drawn, mut speech) = (String::new(), String::new());
    let (mut placed, mut spent, mut left) = (
        0usize,
        0u32,
        match arrival {
            Arrival::Whole => None,
            Arrival::Partial(cells) => Some(cells),
        },
    );
    for record in tiling {
        let row = area
            .row
            .saturating_add(to_cols(placed / usize::from(per_row)));
        if row >= area.bottom() {
            break;
        }
        let col = area
            .col
            .saturating_add(indent)
            .saturating_add(to_cols(placed % usize::from(per_row)) * stride);

        // **Padded to the run's name column**, so every `=` in the listing sits
        // in the same place and the eye can run down it. Built as one string
        // rather than drawn in three pieces because the reveal clips by
        // character and speaks per record: splitting the tile would type the
        // value before the name on a narrow pane, and say each entry twice.
        drawn.clear();
        let mut bound = false;
        if plan.describing {
            // `attend <place>` padded to the run's widest, then what it does.
            // Two spaces and no `=`: the second column is a sentence about the
            // first, not a value bound to it.
            //
            // **No leader between them, and that is a rule rather than a
            // preference.** A leader bridges to a *right-aligned* column, where
            // the gap it spans is genuinely variable — which is why the boot
            // card's `name ....... ok` works. Here the description column is
            // left-aligned, so the run length is decided entirely by the
            // signature column's raggedness: the shortest verb would take a
            // dozen dots and the longest none at all, and the eye would land on
            // the dots instead of the words.
            drawn.push_str(&signature_of(&record));
            if let Some(named) = plan.named {
                for _ in drawn.chars().count()..usize::from(named) {
                    drawn.push(' ');
                }
            }
            drawn.push_str(DESCRIBES);
            if let Some(detail) = record.field(FieldName::Detail) {
                detail.write(&mut drawn);
            }
            // **`bound` stays false**, and it has to. It is what tells the draw
            // below to overdraw a dim ` = ` at the name column — and there is no
            // `=` in this form, so setting it stamped one over the first
            // character of every description: `grind <reagent> = rush a reagent
            // in the mortar`. Two spaces need no punctuation to recede.
        } else {
            match plan.named {
                Some(named) => {
                    let mut fields = record.content();
                    if let Some((_, name)) = fields.next() {
                        name.write(&mut drawn);
                        // A run may mix entries that have an amount with entries
                        // that do not — a room holds reagents *and* fixtures. One
                        // without gets its name and no trailing ` = ` to explain.
                        if let Some(value) = record.field(FieldName::Quantity) {
                            for _ in drawn.chars().count()..usize::from(named) {
                                drawn.push(' ');
                            }
                            drawn.push_str(BINDS);
                            value.write(&mut drawn);
                            bound = true;
                            // **A listed thing may also be in a state.** The
                            // arsenal says how well stocked the tower is in each
                            // row — `fresh`, `thin`, `spent` — and this is the
                            // path a *pane* takes; `amount_of` is the stacked
                            // one. Drawing it in only one of the two left the
                            // linear stream saying `state: thin` while the
                            // screen said nothing, which is §14 backwards.
                            if let Some(state) = record.field(FieldName::State) {
                                drawn.push_str(STATES);
                                state.write(&mut drawn);
                            }
                        }
                    }
                }
                // The same composition the measure used. Keeping the two
                // one call apart is what stops them drifting.
                None => drawn.push_str(&signature_of(&record)),
            }
        }
        speech.clear();
        record.speak(&mut speech);

        // Row-major, the order the run is drawn in, so a listing fills across
        // and then down exactly as a terminal would print it.
        let here = left.map_or(Arrival::Whole, Arrival::Partial);
        let (visible, used) = here.clip(&drawn);
        if let Some(left) = left.as_mut() {
            *left = left.saturating_sub(used);
        }
        spent = spent.saturating_add(used);
        let complete = visible.len() == drawn.len();

        if complete {
            // Spoken in stream order, one utterance per record, exactly as the
            // stacked path does — so §14's linear form is unchanged by the wrap.
            painter.span(
                Pos::new(col, row),
                &crate::span::Span::new(visible)
                    .with_style(record.style())
                    .with_kind(record.kind().utterance())
                    .with_spoken(&speech),
            );
            // The `=` is punctuation holding two facts apart, not a fact — so it
            // recedes, and the name and the value it separates do not. Overdrawn
            // rather than drawn as a third span: the span above already carried
            // the whole tile into the linear stream, and a second one here would
            // put ` = ` in it as an utterance of its own.
            if let Some(named) = plan.named.filter(|_| bound) {
                painter.glyphs(Pos::new(col.saturating_add(named), row), BINDS, Style::DIM);
            }
        } else if !visible.is_empty() {
            // Silent until whole — see the stacked path for why this is `glyphs`.
            painter.glyphs(Pos::new(col, row), visible, record.style());
        }
        placed += 1;
    }

    let rows = placed.div_ceil(usize::from(per_row));
    Some((to_cols(rows), placed, spent))
}

/// Where column `index` begins, or `None` if it starts past the right edge.
fn column_start(area: Rect, widths: &[usize], index: usize) -> Option<Pos> {
    let offset: usize = widths[..index].iter().sum::<usize>() + COLUMN_GAP * index;
    (offset < usize::from(area.cols))
        .then(|| Pos::new(area.col.saturating_add(to_cols(offset)), area.row))
}

/// A measured width as a column count, saturating rather than wrapping.
///
/// Anything that does not fit in a `u16` is off the far edge of any grid this
/// game will ever host, so clamping is the correct answer and not a fudge.
fn to_cols(width: usize) -> u16 {
    u16::try_from(width).unwrap_or(u16::MAX)
}
