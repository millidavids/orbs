//! Output arriving over a wire instead of all at once.
//!
//! A command's answer used to appear whole on the frame the tick produced it.
//! Now it prints, the way a terminal attached to something slow prints.
//!
//! Presentation only: every record is in the
//! [`Scrollback`](orbs_sim::Scrollback) the instant the tick makes it, and this
//! draws fewer of their cells. The sim never learns a reveal happened
//! (architectural rule 2) — `orbs-balance` would owe typewriter delays to stay
//! in step (§13), offline catch-up would owe animation time, and §9's parity
//! rule forbids a display setting becoming a difficulty choice.
//!
//! The cost it is meant to create arrives anyway: it spends the player's
//! attention, and a player who automates stops spending it.
//!
//! The first character lands on the frame the record does — a terminal that
//! takes a second to answer reads as broken (§19), and §6 makes the echo how
//! players learn the vocabulary. Only the rest takes time.

use bevy_ecs::prelude::*;

/// How fast text types itself — `settings` → `habits` → `pace`, in baud.
///
/// A teletype ran ~10 characters a second and is unusable; a 9600-baud VT100
/// managed ~960 and is indistinguishable from instant. The default was settled
/// by looking at 420, and is 4800 baud now that it has a name.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Pace {
    /// 120 characters a second: a line visibly typing.
    Baud1200,
    /// 480 characters a second.
    #[default]
    Baud4800,
    /// 960 characters a second, which reads as nearly instant.
    Baud9600,
    /// No reveal at all — what stillness forces.
    Instant,
}

impl Pace {
    /// All four, in the order the row cycles them.
    pub const ALL: [Self; 4] = [
        Self::Baud1200,
        Self::Baud4800,
        Self::Baud9600,
        Self::Instant,
    ];

    /// The word for it, on the page and in the file.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Baud1200 => "1200",
            Self::Baud4800 => "4800",
            Self::Baud9600 => "9600",
            Self::Instant => "instant",
        }
    }

    /// The pace `word` names, whole.
    #[must_use]
    pub fn named(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|pace| pace.word() == word)
    }

    /// Characters a second, or `None` for no reveal.
    ///
    /// A baud is a bit a second and a character ten of them on the wire.
    const fn characters(self) -> Option<f32> {
        match self {
            Self::Baud1200 => Some(120.0),
            Self::Baud4800 => Some(480.0),
            Self::Baud9600 => Some(960.0),
            Self::Instant => None,
        }
    }
}

/// The longest one burst of output may take to arrive, in seconds.
///
/// Without a ceiling `peruse orb.log` takes minutes and stops being flavour.
/// Past this the rate rises to fit, so a long dump prints fast while a one-line
/// echo still types itself.
const LONGEST: f32 = 1.4;

/// How much of the newest output has arrived.
///
/// Whole characters plus a fraction rather than a float count: the budget stays
/// an integer at every observation point, so no `f32` is ever cast to one.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Reveal {
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
    /// A burst begins when the record count grows, and any burst in flight is
    /// abandoned rather than queued — queueing would let a fast player build a
    /// backlog that never drains.
    pub const fn observe(&mut self, records: usize, cells: u16) {
        if records <= self.seen {
            // The stream can also shrink — a test, a future `clear` — and
            // resyncing beats revealing from a stale index forever.
            self.seen = records;
            return;
        }
        self.from = self.seen;
        self.seen = records;
        self.shown = 0;
        self.total = cells;
        self.carry = 0.0;
    }

    /// Let `delta` seconds pass, at the default pace.
    pub fn advance(&mut self, delta: f32) {
        self.advance_at(delta, Pace::default());
    }

    /// Let `delta` seconds pass at `pace`.
    pub fn advance_at(&mut self, delta: f32, pace: Pace) {
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
        self.seen = records;
        self.finish();
    }

    /// Whether everything has arrived.
    #[must_use]
    pub const fn is_settled(&self) -> bool {
        self.shown >= self.total
    }

    /// How many records were on screen when the current burst began.
    ///
    /// The caller measures the next burst's length from it.
    #[must_use]
    pub const fn settled_len(&self) -> usize {
        self.seen
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
        // Most frames have no new output: they must cost nothing.
        assert!(Reveal::default().is_settled());
        assert_eq!(Reveal::default().budget(0), None);
    }

    #[test]
    fn the_first_character_is_available_on_the_frame_the_record_arrives() {
        // §19: a terminal that takes a second to answer reads as broken. The
        // reveal may take time; the response may not.
        let mut reveal = burst(40);
        reveal.advance(1.0 / 60.0);
        let (_, cells) = reveal.budget(0).expect("mid-reveal");
        assert!(cells >= 1, "nothing had arrived after a frame");
    }

    #[test]
    fn a_long_burst_still_finishes_inside_the_ceiling() {
        // The rate rises to fit rather than the reveal being truncated.
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
    fn a_slower_pace_has_less_on_screen_and_instant_has_all_of_it() {
        let shown = |pace| {
            let mut reveal = burst(200);
            reveal.advance_at(0.1, pace);
            reveal.budget(0).map_or(200, |(_, cells)| cells)
        };
        assert!(shown(Pace::Baud1200) < shown(Pace::Baud4800));
        assert!(shown(Pace::Baud4800) < shown(Pace::Baud9600));
        assert_eq!(shown(Pace::Instant), 200, "instant still typed");
        // The ceiling holds at every pace: a slow pace never makes a log a wall.
        let mut reveal = burst(u16::MAX);
        let mut elapsed = 0.0;
        while !reveal.is_settled() && elapsed < 10.0 {
            reveal.advance_at(1.0 / 60.0, Pace::Baud1200);
            elapsed += 1.0 / 60.0;
        }
        assert!(elapsed <= LONGEST + 0.1, "{elapsed}s at 1200 baud");
    }

    #[test]
    fn a_pace_is_its_own_word() {
        for pace in Pace::ALL {
            assert_eq!(Pace::named(pace.word()), Some(pace));
        }
        assert_eq!(Pace::named("300"), None);
    }

    #[test]
    fn a_short_burst_still_types_itself() {
        // The other end: the ceiling must not make a one-line echo instant.
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
        // Queueing would let a fast player build a backlog, and the screen
        // would fall further behind the world with every command.
        let mut reveal = Reveal::default();
        reveal.observe(3, 300);
        reveal.advance(0.05);
        reveal.observe(5, 40);

        assert_eq!(reveal.total, 40, "the new burst inherited the old length");
        assert_eq!(reveal.shown, 0, "the new burst started part-way through");
        assert_eq!(reveal.from, 3, "the first burst's records are not settled");
    }

    #[test]
    fn records_put_back_on_screen_do_not_type_themselves() {
        let mut reveal = Reveal::default();
        reveal.observe(10, 100);
        reveal.finish();
        reveal.settle_at(16);
        reveal.observe(16, 60);
        assert!(reveal.is_settled(), "the echoes put back typed again");
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
        // The pane skips the head of the stream, and the view indexes from the
        // first record it is handed, not the first that exists.
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
