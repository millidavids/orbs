//! A number partway to its new value (DESIGN.md §19, *number go up*).
//!
//! The shape only. The shell decides when a roll starts and how far it has run;
//! this decides what is drawn at that point, and it steps on
//! [`FLIP_HZ`](crate::FLIP_HZ)'s grid so a digit is held to the same rate as
//! every other animated cell.

use crate::style::Intensity;
use crate::tween;

/// How long a roll takes, in seconds.
///
/// Half a world tick, the crossing's bound: a roll as long as a tick would pace
/// itself by its own length, and automation changes a number most ticks.
pub const ROLL_SECS: f32 = 0.5;

/// How many times a rolling number changes on its way: [`ROLL_SECS`] of
/// [`FLIP_HZ`](crate::FLIP_HZ) ticks.
pub const ROLL_STEPS: u16 = 3;

/// How long a gain's `+N` stays, in seconds: one world tick, the flare's length
/// and the longest an animation here may run.
///
/// Longer than the roll so it can be read. It fades in three weights, a third
/// of a second each.
pub const PLUS_SECS: f32 = 1.0;

/// A number on its way from `from` to `to`, `t` of the way through.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Roll {
    /// What was showing when it started.
    pub from: u64,
    /// The truth.
    pub to: u64,
    /// How far through, `0.0..=1.0`.
    pub t: f32,
}

impl Roll {
    /// A number that is not moving.
    #[must_use]
    pub const fn settled(value: u64) -> Self {
        Self {
            from: value,
            to: value,
            t: 1.0,
        }
    }

    /// Which step of the flip grid the roll has reached, `0..=ROLL_STEPS`.
    ///
    /// Floored, not rounded: the first change lands a whole flip tick after the
    /// roll starts, so a roll restarted mid-way cannot change twice inside one.
    #[must_use]
    pub fn step(self) -> u16 {
        floored(self.t, ROLL_STEPS)
    }

    /// The value to draw: eased toward the truth, front-loaded, and exact at
    /// both ends.
    #[must_use]
    pub fn shown(self) -> u64 {
        self.at_step(self.step())
    }

    /// The value at `step` of the grid.
    fn at_step(self, step: u16) -> u64 {
        if step == 0 {
            return self.from;
        }
        if step >= ROLL_STEPS {
            return self.to;
        }
        let through = f32::from(step) / f32::from(ROLL_STEPS);
        let permille = u128::from(tween::mix(0, 1000, ease(through)));
        let part =
            |gap: u64| -> u64 { u64::try_from(u128::from(gap) * permille / 1000).unwrap_or(gap) };
        if self.to >= self.from {
            self.from + part(self.to - self.from)
        } else {
            self.from - part(self.from - self.to)
        }
    }
}

/// How a `+N` is drawn `t` of the way through [`PLUS_SECS`]: bright, then
/// normal, then dim, then gone. Floored, for [`Roll::step`]'s reason.
#[must_use]
pub fn fading(t: f32) -> Option<Intensity> {
    match floored(t, 3) {
        0 => Some(Intensity::Bright),
        1 => Some(Intensity::Normal),
        2 => Some(Intensity::Dim),
        _ => None,
    }
}

/// Which of `steps` equal steps `t` has reached, `0..=steps`.
fn floored(t: f32, steps: u16) -> u16 {
    // `mix` rounds, so half a step back turns its rounding into a floor.
    tween::mix(0, steps, t - 0.5 / f32::from(steps)).min(steps)
}

/// Fast, then settling: most of the distance in the first step.
fn ease(x: f32) -> f32 {
    let left = 1.0 - x.clamp(0.0, 1.0);
    1.0 - left * left
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse::FLIP_HZ;

    fn flip_secs() -> f32 {
        1.0 / FLIP_HZ
    }

    #[test]
    fn the_steps_fill_the_roll_exactly() {
        // Not a tuning pair: the step count is the roll's length on the flip
        // grid, and a mismatch would either break the rate or leave a gap.
        let steps = ROLL_SECS * FLIP_HZ;
        assert!(
            steps == f32::from(ROLL_STEPS),
            "{ROLL_SECS}s at {FLIP_HZ}Hz is {steps} steps, not {ROLL_STEPS}",
        );
        const { assert!(ROLL_SECS <= 0.5, "a roll must leave half a tick spare") }
    }

    #[test]
    fn the_ends_are_the_numbers_themselves() {
        // A roll not begun and a roll finished must be indistinguishable from
        // no roll at all, or a settled screen differs from a still one.
        for (from, to) in [(8, 16), (16, 8), (0, 1), (5, 5), (0, u64::MAX)] {
            assert_eq!(Roll { from, to, t: 0.0 }.shown(), from);
            assert_eq!(Roll { from, to, t: 1.0 }.shown(), to);
            assert_eq!(Roll { from, to, t: 7.0 }.shown(), to);
            assert_eq!(Roll { from, to, t: -1.0 }.shown(), from);
            let unread = Roll {
                from,
                to,
                t: f32::NAN,
            };
            assert_eq!(unread.shown(), from);
        }
        assert_eq!(Roll::settled(9).shown(), 9);
    }

    #[test]
    fn it_never_turns_back() {
        for (from, to) in [(0, 25), (25, 0), (3, 1000), (1000, 3)] {
            let mut last = from;
            for n in 0..=240u16 {
                let shown = Roll {
                    from,
                    to,
                    t: f32::from(n) / 240.0,
                }
                .shown();
                if to >= from {
                    assert!(shown >= last, "{from}->{to} fell back to {shown}");
                } else {
                    assert!(shown <= last, "{from}->{to} climbed back to {shown}");
                }
                last = shown;
            }
        }
    }

    #[test]
    fn a_digit_turns_over_at_most_flip_hz_times_a_second() {
        // The photosensitivity guarantee, held the way `pulse` holds it: sampled
        // far above any frame rate, no two changes closer than one flip tick.
        let rate = 240.0_f32;
        let roll = |at: f32| Roll {
            from: 0,
            to: 1000,
            t: at / ROLL_SECS,
        };
        let mut last = roll(0.0).shown();
        let mut changed_at: Option<f32> = None;
        for n in 1..=240u16 {
            let at = f32::from(n) / rate;
            let shown = roll(at).shown();
            if shown != last {
                if let Some(before) = changed_at {
                    assert!(
                        at - before >= flip_secs() - 1.0 / rate,
                        "changed at {before}s and again at {at}s",
                    );
                }
                assert!(
                    at >= flip_secs() - 1.0 / rate,
                    "the first change came {at}s in, inside one flip tick",
                );
                changed_at = Some(at);
                last = shown;
            }
        }
        assert_eq!(last, 1000, "the roll never arrived");
    }

    #[test]
    fn a_plus_fades_and_goes() {
        let weights: Vec<_> = [0.0, 0.2, 0.34, 0.5, 0.67, 0.9, 1.0, 3.0]
            .into_iter()
            .map(fading)
            .collect();
        assert_eq!(
            weights,
            [
                Some(Intensity::Bright),
                Some(Intensity::Bright),
                Some(Intensity::Normal),
                Some(Intensity::Normal),
                Some(Intensity::Dim),
                Some(Intensity::Dim),
                None,
                None,
            ],
        );
        // A third of a second a weight, so it turns over at 3 Hz.
        const { assert!(PLUS_SECS / 3.0 >= 1.0 / FLIP_HZ) }
    }

    #[test]
    fn most_of_the_way_is_covered_first() {
        // Front-loaded, so three steps read as a count settling rather than a
        // number creeping and then jumping.
        let first = Roll {
            from: 0,
            to: 900,
            t: 1.0 / 3.0,
        }
        .shown();
        assert!(first > 450, "the first step covered only {first} of 900");
    }
}
