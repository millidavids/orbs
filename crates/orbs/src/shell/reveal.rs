//! Output arriving over a wire instead of all at once.
//!
//! A command's answer used to appear whole on the frame the tick produced it.
//! Now it prints, the way a terminal attached to something slow prints.
//!
//! # It is presentation, and that is load-bearing
//!
//! Every record is in the [`Scrollback`](orbs_sim::Scrollback) the instant the
//! tick makes it; this draws fewer of their cells and nothing else. The sim
//! never learns a reveal happened, and nothing may make it — architectural rule
//! 2, for three concrete reasons:
//!
//! - `orbs-balance` would have to simulate typewriter delays to stay in step
//!   with the live game, which is the divergence §13 exists to prevent.
//! - Offline catch-up is ~29k `step()` calls and would owe animation time.
//! - §9's parity rule says a display setting must never become a difficulty
//!   choice. Make waiting a *modelled* cost and turning the animation off
//!   becomes a competitive advantage — the same failure pointed the other way,
//!   and aimed at exactly the players §14 exists for.
//!
//! The detriment it is meant to create arrives anyway, because it costs the
//! player seconds of attention and a player who automates stops spending them.
//! That lands in Phase 1: a bound script running twenty commands is not a person
//! watching twenty reveals.
//!
//! # The response is instant even though the reveal is not
//!
//! §19 settled this once already — *"a terminal that takes a second to answer
//! reads as broken"*, and §6 makes the echo the thing players learn the
//! vocabulary from. The first character lands on the same frame the record does.
//! Only the rest of it takes time.

use bevy::prelude::*;

/// Characters per second once a reveal is under way.
///
/// A real teletype ran about 10 (110 baud) and is unusable; a 9600-baud VT100
/// managed ~960 and is indistinguishable from instant. This is the useful
/// middle, and it is a number to settle by looking rather than by reasoning.
const RATE: f32 = 420.0;

/// The longest one burst of output may take to arrive, in seconds.
///
/// Without a ceiling, `peruse orb.log` on a long session would take minutes and
/// the reveal would stop being flavour and start being a wall. Past this the
/// effective rate rises to fit, so a long dump prints fast while a one-line echo
/// still types itself.
const LONGEST: f32 = 1.4;

/// How much of the newest output has arrived.
///
/// Whole characters plus a fraction, rather than a float count: it keeps the
/// budget an integer at every observation point and means no `f32` is ever cast
/// to one.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct Reveal {
    /// How many records the stream held when this burst began.
    seen: usize,
    /// The first record of the current burst. Everything before it is whole.
    from: usize,
    /// Characters of the burst on screen.
    shown: u16,
    /// Characters the burst contains.
    total: u16,
    /// The fraction of the next character that has arrived.
    carry: f32,
}

impl Reveal {
    /// Note that the stream now holds `records` records, the newest of which
    /// carry `cells` characters between them.
    ///
    /// A burst begins when the record count grows. Any burst still in flight is
    /// **abandoned rather than queued** — its records finish immediately and the
    /// new one starts. Queueing would let a fast player build a backlog that
    /// never drains, and the output on screen would fall further behind the
    /// world with every command.
    pub(crate) const fn observe(&mut self, records: usize, cells: u16) {
        if records <= self.seen {
            // The stream can also shrink — a test clearing it, a future `clear`
            // command — and resyncing is better than revealing from a stale
            // index forever.
            self.seen = records;
            return;
        }
        self.from = self.seen;
        self.seen = records;
        self.shown = 0;
        self.total = cells;
        self.carry = 0.0;
    }

    /// Let `delta` seconds pass.
    pub(crate) fn advance(&mut self, delta: f32) {
        if self.is_settled() {
            return;
        }
        self.carry += self.rate() * delta;
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
    /// Any keystroke does this, which is what keeps the reveal from ever being a
    /// mechanic: a player who does not want to wait never waits, and the ones
    /// who most need that — §14's — are not paying for it in capability.
    pub(crate) const fn finish(&mut self) {
        self.shown = self.total;
        self.carry = 0.0;
    }

    /// Whether everything has arrived.
    pub(crate) const fn is_settled(&self) -> bool {
        self.shown >= self.total
    }

    /// How many records were on screen when the current burst began.
    ///
    /// What the caller measures the next burst's length from, so the two agree
    /// about where one ends and the next starts.
    pub(crate) const fn settled_len(&self) -> usize {
        self.seen
    }

    /// What to hand [`RecordView::revealing`](orbs_render::RecordView::revealing),
    /// given how many records the pane skipped off the top of the stream.
    ///
    /// `None` once everything has arrived, so a settled screen pays nothing.
    pub(crate) fn budget(&self, skipped: usize) -> Option<(usize, u32)> {
        if self.is_settled() {
            return None;
        }
        Some((self.from.saturating_sub(skipped), u32::from(self.shown)))
    }

    /// Characters per second for the burst in flight.
    ///
    /// Expressed as a rate rather than as a clamp on the elapsed time, so a long
    /// burst is uniformly faster instead of printing slowly and then snapping.
    fn rate(&self) -> f32 {
        RATE.max(f32::from(self.total) / LONGEST)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A burst of `cells` characters, just arrived.
    fn burst(cells: u16) -> Reveal {
        let mut reveal = Reveal::default();
        reveal.observe(1, cells);
        reveal
    }

    #[test]
    fn nothing_to_reveal_is_already_settled() {
        // The common case by a wide margin: most frames have no new output, and
        // they must cost nothing and hand the view no budget at all.
        assert!(Reveal::default().is_settled());
        assert_eq!(Reveal::default().budget(0), None);
    }

    #[test]
    fn the_first_character_is_available_on_the_frame_the_record_arrives() {
        // §19: "a terminal that takes a second to answer reads as broken", and
        // §6 makes the echo the teaching mechanism. The reveal may take time;
        // the response may not.
        let mut reveal = burst(40);
        reveal.advance(1.0 / 60.0);
        let (_, cells) = reveal.budget(0).expect("mid-reveal");
        assert!(cells >= 1, "nothing had arrived after a frame");
    }

    #[test]
    fn a_long_burst_still_finishes_inside_the_ceiling() {
        // Without this a `peruse` of a long session takes minutes. The rate
        // rises to fit rather than the reveal being truncated.
        let mut reveal = burst(u16::MAX);
        let mut elapsed = 0.0;
        while !reveal.is_settled() && elapsed < 10.0 {
            reveal.advance(1.0 / 60.0);
            elapsed += 1.0 / 60.0;
        }
        assert!(reveal.is_settled(), "it never finished");
        assert!(
            elapsed <= LONGEST + 0.1,
            "{elapsed}s is a wall, not flavour"
        );
    }

    #[test]
    fn a_short_burst_still_types_itself() {
        // The other end of the same rule: the ceiling must not make a one-line
        // echo instant, or there is no animation at all on the most common case.
        let mut reveal = burst(20);
        reveal.advance(1.0 / 60.0);
        assert!(!reveal.is_settled(), "a short line arrived all at once");
    }

    #[test]
    fn a_keystroke_finishes_it() {
        let mut reveal = burst(500);
        reveal.advance(1.0 / 60.0);
        assert!(!reveal.is_settled());
        reveal.finish();
        assert!(reveal.is_settled());
        assert_eq!(reveal.budget(0), None);
    }

    #[test]
    fn a_second_burst_abandons_the_first_rather_than_queueing_behind_it() {
        // Two ticks landing close together must not stack. Queueing would let a
        // fast player build a backlog, and what is on screen would fall further
        // behind the world with every command.
        let mut reveal = Reveal::default();
        reveal.observe(3, 300);
        reveal.advance(0.05);
        reveal.observe(5, 40);

        assert_eq!(reveal.total, 40, "the new burst inherited the old length");
        assert_eq!(reveal.shown, 0, "the new burst started part-way through");
        assert_eq!(reveal.from, 3, "the first burst's records are not settled");
    }

    #[test]
    fn a_shrinking_stream_resyncs_instead_of_revealing_from_a_stale_index() {
        let mut reveal = Reveal::default();
        reveal.observe(10, 100);
        reveal.finish();
        reveal.observe(2, 0);
        assert!(reveal.is_settled());

        reveal.observe(4, 30);
        assert_eq!(reveal.from, 2, "the burst began from a record that is gone");
    }

    #[test]
    fn the_budget_is_relative_to_what_the_pane_actually_draws() {
        // The pane skips the head of the stream to show the tail, and the view
        // indexes from the first record it is handed — not from the first record
        // that exists.
        let mut reveal = Reveal::default();
        // A settled session of 14 records, then a burst adding six more.
        reveal.observe(14, 200);
        reveal.finish();
        reveal.observe(20, 50);
        reveal.advance(0.01);

        // The pane shows the last 12, so the burst begins at drawn record 2.
        let (after, _) = reveal.budget(12).expect("mid-reveal");
        assert_eq!(after, 2, "the reveal started at the wrong drawn record");
        // The pane skipped past the burst's start: everything drawn is new.
        let (after, _) = reveal.budget(99).expect("mid-reveal");
        assert_eq!(
            after, 0,
            "skipping past the burst should reveal from the top"
        );
    }
}
