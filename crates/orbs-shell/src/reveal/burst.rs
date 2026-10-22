//! How much of the newest burst of output has arrived.

use bevy_ecs::prelude::*;
use orbs_render::Records;

use super::pace::Pace;

/// The longest one burst of output may take to arrive, in seconds.
///
/// Without a ceiling `peruse orb.log` takes minutes and stops being flavour.
/// Past this the rate rises to fit, so a long dump prints fast while a one-line
/// echo still types itself.
pub(super) const LONGEST: f32 = 1.4;

/// How much of the newest output has arrived.
///
/// Whole characters plus a fraction rather than a float count: the budget stays
/// an integer at every observation point, so no `f32` is ever cast to one.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Reveal {
    /// How many records the stream held when this burst began, or `None` before
    /// the first look.
    pub(super) seen: Option<usize>,
    /// The first record of the current burst. Everything before it is whole.
    pub(super) from: usize,
    /// Characters of the burst on screen.
    pub(super) shown: u16,
    /// Characters the burst contains.
    pub(super) total: u16,
    /// The fraction of the next character that has arrived.
    pub(super) carry: f32,
}

impl Reveal {
    /// One frame: notice new output in `records`, then let it arrive at `pace` —
    /// or land it whole when `motion` says nothing may move.
    ///
    /// Both builds' whole use of this in one call, so they cannot come to type at
    /// different times. `motion` is `None` where a caller has no switch, which
    /// types as if one were on.
    pub fn drive(&mut self, records: &Records, delta: f32, motion: Option<bool>, pace: Pace) {
        let drawn = records.drawn_len();
        // Counted only when the stream grew, since it walks the whole stream.
        let cells = match self.seen {
            Some(seen) if drawn > seen => tail_cells(records, seen),
            _ => 0,
        };
        self.observe(drawn, cells);
        // Text typing itself is motion: when nothing may move it lands whole.
        if motion == Some(false) {
            self.finish();
            return;
        }
        self.advance_at(delta, pace);
    }

    /// Note that the stream now holds `records` records, the newest of which
    /// carry `cells` characters between them.
    ///
    /// A burst begins when the record count grows, and any burst in flight is
    /// abandoned rather than queued — queueing would let a fast player build a
    /// backlog that never drains. The first look begins none: what was on screen
    /// before anyone watched is not news, and a tower opened from the menu
    /// typed its history out again.
    pub(super) const fn observe(&mut self, records: usize, cells: u16) {
        let Some(seen) = self.seen else {
            self.seen = Some(records);
            return;
        };
        self.seen = Some(records);
        if records <= seen {
            // The stream can also shrink — a test, a future `clear` — and
            // resyncing beats revealing from a stale index forever.
            return;
        }
        self.from = seen;
        self.shown = 0;
        self.total = cells;
        self.carry = 0.0;
    }

    /// Let `delta` seconds pass, at the default pace.
    #[cfg(test)]
    pub(super) fn advance(&mut self, delta: f32) {
        self.advance_at(delta, Pace::default());
    }

    /// Let `delta` seconds pass at `pace`.
    pub(super) fn advance_at(&mut self, delta: f32, pace: Pace) {
        if self.is_settled() {
            return;
        }
        let Some(characters) = pace.characters() else {
            self.finish();
            return;
        };
        self.carry += self.rate(characters) * delta;
        while self.carry >= 1.0 && self.shown < self.total {
            self.carry -= 1.0;
            self.shown += 1;
        }
        if self.shown >= self.total {
            self.carry = 0.0;
        }
    }

    /// Put all of it on screen now.
    ///
    /// Any keystroke does this, so the reveal is never a mechanic: §14's
    /// players do not pay for skipping it.
    pub const fn finish(&mut self) {
        self.shown = self.total;
        self.carry = 0.0;
    }

    /// Take a stream of `records` drawn records as already on screen, whole.
    ///
    /// For a change in what is drawn rather than in what was said: echoes put
    /// back by the `echo` row are not news, and must not type themselves.
    pub const fn settle_at(&mut self, records: usize) {
        self.seen = Some(records);
        self.finish();
    }

    /// Whether everything has arrived.
    #[must_use]
    pub const fn is_settled(&self) -> bool {
        self.shown >= self.total
    }

    /// What to hand [`RecordView::revealing`](orbs_render::RecordView::revealing),
    /// given how many records the pane skipped off the top of the stream.
    ///
    /// `None` once everything has arrived, so a settled screen pays nothing.
    #[must_use]
    pub fn budget(&self, skipped: usize) -> Option<(usize, u32)> {
        if self.is_settled() {
            return None;
        }
        Some((self.from.saturating_sub(skipped), u32::from(self.shown)))
    }

    /// Characters per second for the burst in flight.
    ///
    /// A rate, not a clamp on elapsed time: a long burst prints uniformly
    /// faster rather than slowly and then snapping.
    fn rate(&self, characters: f32) -> f32 {
        characters.max(f32::from(self.total) / LONGEST)
    }
}

/// Characters in the drawn records added since `from`.
///
/// `from` counts drawn records, as `observe` does, so skipping the whole stream
/// would land past the burst whenever an echo or a spell's record is hidden.
/// Measured through `to_line`, which is what the view draws.
pub(super) fn tail_cells(records: &Records, from: usize) -> u16 {
    let counted: usize = records
        .drawn()
        .skip(from)
        .map(|record| record.to_line().chars().count())
        .sum();
    u16::try_from(counted).unwrap_or(u16::MAX)
}
