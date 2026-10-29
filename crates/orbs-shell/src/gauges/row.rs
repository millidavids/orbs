//! One gauge row: its strings laid out first, so the strip can measure it,
//! then drawn.

use orbs_render::{Fill, Painter, Pos, Rect, Span, Style};
use orbs_sim::{Ahead, Prose, Toward};

use crate::climb::Rolled;

/// Cells the label takes, gap included: `ley line` is eight and `renown` six.
pub(super) const LABEL: u16 = 9;

/// The widest a gauge is drawn, however wide the pane: past forty the eye
/// cannot tell a third full from a half.
const WIDEST: u16 = 40;

/// The narrowest bar there is: two brackets with one cell between them —
/// `Painter::gauge`'s own clip test.
const NARROWEST: u16 = 3;

/// Cells between a reading and its `+N`.
const PLUS_GAP: u16 = 2;

/// One row's strings, before anything is placed.
pub(super) struct Laid {
    label: String,
    toward: Toward,
    rolled: Rolled,
    /// The truth's reading. Every width is measured from it.
    reading: String,
    /// What is drawn now: right-aligned in the truth's width, and wider than it
    /// only while a fall or a crossing draws more digits than it ends on.
    drawn: String,
    /// The tower's title, if it has one; whether it fits is the draw's call.
    title: Option<String>,
    /// How long until the next tier, if there is a pace to say it from. It
    /// takes only blank space: never a cell of the bar, nor the room's column.
    forecast: Option<String>,
    spoken: String,
    /// The span the shown value is out of.
    span: u64,
}

/// Lay out one row, or nothing for a track nobody has measured — `0/1` and
/// `nothing more authored` would both be wrong for it.
///
/// `rolled` is where the number has got to. Cells come from it; speech, colour
/// and every width come from the truth, so a roll never moves the bar.
pub(super) fn lay(
    label: &str,
    toward: Toward,
    rolled: Rolled,
    name: Option<&str>,
    forecast: Option<&str>,
    prose: &Prose,
) -> Option<Laid> {
    // Relative both sides of the slash: `8/16`, not `8/10000`.
    let digits = |done: u64, span: u64| {
        prose.line(
            "gauge_toward",
            &[
                ("count", &done.to_string()),
                ("quantity", &span.to_string()),
            ],
        )
    };
    let reading = match toward.ahead {
        Ahead::Tier(_) => digits(toward.done, toward.span),
        Ahead::Nothing => prose.line("gauge_topped", &[]),
        Ahead::Unasked => return None,
    };
    // A crossing draws the tier it is leaving, in digits, even into the top.
    let span = rolled.span.unwrap_or(toward.span);
    let topped = matches!(toward.ahead, Ahead::Nothing) && rolled.span.is_none();
    let shown = if topped {
        reading.clone()
    } else {
        digits(rolled.shown, span)
    };
    let width = cells(&reading);

    // A topped track speaks what it draws: `Toward::FULL` is `1/1`, a fill and
    // not a count.
    let spoken = if matches!(toward.ahead, Ahead::Nothing) {
        prose.line("gauge_spoken_topped", &[("name", label)])
    } else {
        prose.line(
            "gauge_spoken",
            &[
                ("name", label),
                ("count", &toward.done.to_string()),
                ("quantity", &toward.span.to_string()),
            ],
        )
    };
    Some(Laid {
        label: label.to_owned(),
        toward,
        rolled,
        drawn: format!("{shown:>width$}"),
        reading,
        title: name.map(|name| prose.line(&format!("renown_{name}"), &[])),
        forecast: forecast.map(ToOwned::to_owned),
        spoken,
        span,
    })
}

impl Laid {
    /// Cells the row needs to draw a bar of `bar` with its reading and title.
    ///
    /// Not the forecast, which outranks nothing: the room's column is never
    /// dropped to make room for one.
    pub(super) fn needs(&self, bar: u16) -> u16 {
        let tail = u16::try_from(cells(&self.tail(&self.reading, false, true))).unwrap_or(u16::MAX);
        LABEL
            .saturating_add(bar)
            .saturating_add(1)
            .saturating_add(tail)
    }

    /// The reading, then the forecast and the title where each is wanted.
    fn tail(&self, reading: &str, forecast: bool, titled: bool) -> String {
        let mut out = reading.to_owned();
        for part in [
            self.forecast.as_ref().filter(|_| forecast),
            self.title.as_ref().filter(|_| titled),
        ]
        .into_iter()
        .flatten()
        {
            out.push_str("  ");
            out.push_str(part);
        }
        out
    }

    /// Draw it into `at`.
    pub(super) fn draw(&self, painter: &mut Painter<'_>, at: Rect) {
        let start = at.col.saturating_add(LABEL);
        let room = at.right().saturating_sub(start);
        let bar_after = |tail: &str, after: u16| {
            let width = u16::try_from(cells(tail)).unwrap_or(u16::MAX);
            room.saturating_sub(width.saturating_add(after).saturating_add(1))
                .min(WIDEST)
        };
        let plus = self
            .rolled
            .plus
            .map(|(gain, weight)| (format!("+{gain}"), weight));
        let gained = plus.as_ref().map_or(0, |(plus, _)| {
            PLUS_GAP.saturating_add(u16::try_from(cells(plus)).unwrap_or(u16::MAX))
        });

        // The title yields before the bar does: a bar alone still says where the
        // tower stands, and a title alone is a word in a gap. The forecast
        // yields before either, and before a gain's `+N`: drawn only where the
        // bar keeps every cell and the `+N` still fits after it.
        let titled = bar_after(&self.tail(&self.reading, false, true), 0) >= NARROWEST;
        let bar = bar_after(&self.tail(&self.reading, false, titled), 0);
        let forecast = self.forecast.is_some()
            && bar_after(&self.tail(&self.reading, true, titled), gained) == bar;

        // Called even when it cannot draw, so §14's stream does not depend on
        // what fit. No accent: the warming ramp is a depiction, and an accent
        // would paint it flat. The hue is the truth's, or forty cells would turn
        // over together on every step; a crossing's one flash is the exception.
        let count = |value: u64| u32::try_from(value).unwrap_or(u32::MAX);
        painter.gauge_hued(
            Rect::new(start, at.row, bar, 1),
            count(self.rolled.shown),
            count(self.span),
            self.rolled
                .hue
                .unwrap_or_else(|| Fill::of(count(self.toward.done), count(self.toward.span))),
            Style::default(),
            &self.spoken,
        );

        // No label without a bar: a lone `ley line` says less than nothing.
        if bar < NARROWEST {
            return;
        }
        painter.span(
            Pos::new(at.col, at.row),
            &Span::new(&self.label).with_style(Style::DIM),
        );
        let tail = self.tail(&self.drawn, forecast, titled);
        let truth = self.tail(&self.reading, forecast, titled);
        let mut said = Span::new(&tail).with_style(Style::DIM);
        if tail != truth {
            said = said.with_spoken(&truth);
        }
        let after = start.saturating_add(bar).saturating_add(1);
        painter.span(Pos::new(after, at.row), &said);

        // A gain's `+N`, in the blank after the reading and never measured into
        // it. Silent: renown's earning is silent, and a run's own sentence
        // already says what it paid.
        if let Some((plus, weight)) = plus {
            let col = after
                .saturating_add(u16::try_from(cells(&tail)).unwrap_or(u16::MAX))
                .saturating_add(PLUS_GAP);
            let end = u32::from(col) + u32::try_from(cells(&plus)).unwrap_or(u32::MAX);
            if end <= u32::from(at.right()) {
                painter.glyphs(
                    Pos::new(col, at.row),
                    &plus,
                    Style::default().with_intensity(weight),
                );
            }
        }
    }
}

/// One label, one gauge, one reading, and optionally a name after it.
#[cfg(test)]
pub(super) fn row(
    painter: &mut Painter<'_>,
    at: Rect,
    label: &str,
    toward: Toward,
    rolled: Rolled,
    name: Option<&str>,
    prose: &Prose,
) {
    if let Some(laid) = lay(label, toward, rolled, name, None, prose) {
        laid.draw(painter, at);
    }
}

/// How many cells `text` takes.
pub(super) fn cells(text: &str) -> usize {
    text.chars().count()
}
