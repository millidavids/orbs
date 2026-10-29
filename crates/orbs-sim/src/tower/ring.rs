//! When something happened, lately — the one shape a store's freshness and a
//! room's rate are both read from (§19, *the tower speaks up*).

/// Ticks at which something happened, oldest first, bounded by a window and a
/// cap the owner chooses.
///
/// A `Vec`, not a `VecDeque`: the save writes it as a plain list, and the cap
/// is small enough that draining the front costs nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ring(Vec<u64>);

impl Ring {
    /// A ring holding exactly these ticks, for a save.
    #[must_use]
    pub const fn from_ticks(ticks: Vec<u64>) -> Self {
        Self(ticks)
    }

    /// Record `now`, then drop what fell out of `window` and the oldest past
    /// `cap`. The newest are kept: a rate is about *lately*. Returns what the
    /// cap let go while it was still inside the window, for an owner that
    /// counts it.
    pub fn push(&mut self, now: u64, window: u64, cap: usize) -> Option<u64> {
        self.0.push(now);
        bound(&mut self.0, |at| *at, now, window, cap)
    }

    /// How many fall inside `window` of `now`.
    #[must_use]
    pub fn within(&self, now: u64, window: u64) -> usize {
        self.0.iter().filter(|at| inside(**at, now, window)).count()
    }

    /// The pace from the first remembered to the last, or `None` under two.
    ///
    /// Read from the gaps alone, never from now, so a forecast drawn from it
    /// changes only when something is made (§14: progress is not announced).
    #[must_use]
    pub fn pace(&self) -> Option<Pace> {
        let (first, last) = (self.0.first()?, self.0.last()?);
        let made = u64::try_from(self.0.len().checked_sub(1)?).ok()?;
        Pace::of(last.saturating_sub(*first), made)
    }

    /// The ticks, oldest first.
    #[must_use]
    pub fn ticks(&self) -> &[u64] {
        &self.0
    }
}

/// So many made in so many ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pace {
    /// How long they took.
    pub ticks: u64,
    /// How many there were. Never nought.
    pub made: u64,
}

impl Pace {
    /// `made` in `ticks`, if that is a pace: none made is none, and all of
    /// them in one tick would forecast every wait as under a minute.
    #[must_use]
    pub fn of(ticks: u64, made: u64) -> Option<Self> {
        (made > 0 && ticks > 0).then_some(Self { ticks, made })
    }

    /// Ticks until `left` more at this pace, rounded up.
    #[must_use]
    pub const fn eta(self, left: u64) -> u64 {
        left.saturating_mul(self.ticks).div_ceil(self.made)
    }
}

/// Keep `items`, oldest first, to those inside `window` of `now` and the
/// newest `cap` of those: [`Ring`]'s rule, for anything with a tick. Returns
/// the newest the cap let go.
pub fn bound<T>(
    items: &mut Vec<T>,
    at: impl Fn(&T) -> u64,
    now: u64,
    window: u64,
    cap: usize,
) -> Option<T> {
    items.retain(|item| inside(at(item), now, window));
    let excess = items.len().checked_sub(cap).filter(|excess| *excess > 0)?;
    items.drain(..excess).last()
}

/// Whether something at `at` still counts at `now`.
pub(crate) const fn inside(at: u64, now: u64, window: u64) -> bool {
    now.saturating_sub(at) < window
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ring_is_bounded_however_hard_it_is_hammered() {
        let mut ring = Ring::default();
        for now in 0..10_000 {
            ring.push(now, 3_600, 64);
        }
        assert_eq!(ring.ticks().len(), 64);
        assert_eq!(ring.ticks().last(), Some(&9_999), "the newest are kept");
    }

    #[test]
    fn what_falls_out_of_the_window_is_dropped_on_the_next_push() {
        let mut ring = Ring::default();
        ring.push(0, 100, 8);
        ring.push(50, 100, 8);
        assert_eq!(ring.within(120, 100), 1, "0 is out, 50 is in");
        ring.push(140, 100, 8);
        assert_eq!(ring.ticks(), [50, 140]);
    }

    #[test]
    fn two_make_a_pace_and_one_does_not() {
        let mut ring = Ring::default();
        ring.push(10, 3_600, 64);
        assert_eq!(ring.pace(), None);
        ring.push(70, 3_600, 64);
        ring.push(130, 3_600, 64);
        assert_eq!(
            ring.pace(),
            Some(Pace {
                ticks: 120,
                made: 2
            })
        );
    }

    #[test]
    fn the_cap_hands_back_what_it_let_go() {
        let mut ring = Ring::default();
        assert_eq!(ring.push(1, 100, 2), None);
        assert_eq!(ring.push(2, 100, 2), None);
        assert_eq!(ring.push(3, 100, 2), Some(1));
        // The window's drops are not the cap's.
        assert_eq!(ring.push(500, 100, 2), None);
    }

    #[test]
    fn makings_in_one_tick_are_no_pace() {
        let mut ring = Ring::default();
        ring.push(40, 3_600, 64);
        ring.push(40, 3_600, 64);
        assert_eq!(ring.pace(), None);
    }

    #[test]
    fn an_eta_rounds_up() {
        let pace = Pace {
            ticks: 120,
            made: 2,
        };
        assert_eq!(pace.eta(3), 180);
        assert_eq!(Pace { ticks: 10, made: 3 }.eta(1), 4);
        assert_eq!(pace.eta(0), 0);
    }
}
