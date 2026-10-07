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
        spoken,
        span,
    })
}

impl Laid {
    /// Cells the row needs to draw a bar of `bar` with its reading and title.
    pub(super) fn needs(&self, bar: u16) -> u16 {
        let tail = u16::try_from(cells(&self.with_title(&self.reading, true))).unwrap_or(u16::MAX);
        LABEL
            .saturating_add(bar)
            .saturating_add(1)
            .saturating_add(tail)
    }

    fn with_title(&self, reading: &str, titled: bool) -> String {
        match self.title.as_ref().filter(|_| titled) {
            Some(title) => format!("{reading}  {title}"),
            None => reading.to_owned(),
        }
    }

    /// Draw it into `at`.
    pub(super) fn draw(&self, painter: &mut Painter<'_>, at: Rect) {
        let start = at.col.saturating_add(LABEL);
        let room = at.right().saturating_sub(start);
        let bar_after = |tail: &str| {
            let width = u16::try_from(cells(tail)).unwrap_or(u16::MAX);
            room.saturating_sub(width.saturating_add(1)).min(WIDEST)
        };

        // The title yields before the bar does: a bar alone still says where the
        // tower stands, and a title alone is a word in a gap.
        let titled = bar_after(&self.with_title(&self.reading, true)) >= NARROWEST;
        let bar = bar_after(&self.with_title(&self.reading, titled));

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
        let tail = self.with_title(&self.drawn, titled);
        let truth = self.with_title(&self.reading, titled);
        let mut said = Span::new(&tail).with_style(Style::DIM);
        if tail != truth {
            said = said.with_spoken(&truth);
        }
        let after = start.saturating_add(bar).saturating_add(1);
        painter.span(Pos::new(after, at.row), &said);

        // A gain's `+N`, in the blank after the reading and never measured into
        // it. Silent: renown's earning is silent, and a run's own sentence
        // already says what it paid.
        if let Some((gain, weight)) = self.rolled.plus {
            let plus = format!("+{gain}");
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
    if let Some(laid) = lay(label, toward, rolled, name, prose) {
        laid.draw(painter, at);
    }
}

/// How many cells `text` takes.
pub(super) fn cells(text: &str) -> usize {
    text.chars().count()
}
