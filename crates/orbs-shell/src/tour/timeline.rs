//! Where a scene's steps have reached: which key is next, and when.
//!
//! The window hands it real seconds, the world's tick and what is up; it hands
//! back at most one keystroke a frame. A step starts once the one before it has
//! *landed* — the tick after its Enter, so no two lines share a tick and the
//! second refuse — and its pause and `when` allow.

use std::collections::VecDeque;

use super::steps::{Named, Step, Stroke, When};

/// Seconds between typed characters: quick, and still visibly typed.
pub const TYPING: f32 = 0.035;

/// Seconds around a named key, so an arrow or a function key reads as pressed.
pub const KEYING: f32 = 0.15;

/// A scene's steps, and how far they have got.
#[derive(Debug, Clone)]
pub struct Timeline {
    steps: Vec<Step>,
    /// The step to start next.
    next: usize,
    phase: Phase,
    /// Seconds since the scene began.
    clock: f32,
    /// When the last step landed, once it has.
    done_at: Option<f32>,
}

#[derive(Debug, Clone)]
enum Phase {
    /// Waiting to start `next`, `since` seconds after the last landing.
    Waiting { since: f32 },
    /// Typing a step; `wait` seconds until its next stroke.
    Typing {
        strokes: VecDeque<Stroke>,
        wait: f32,
    },
    /// Its Enter is in. The tick is read next frame, after the world has had
    /// the step that ran before the line was submitted.
    Entered,
    /// Waiting for the world's tick to move from `tick`.
    Ticking { tick: u64 },
    /// Every step has landed.
    Done,
}

/// What the window sees this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seen {
    /// The world's tick.
    pub tick: u64,
    /// Whether a line entered now reaches the world: the prompt has the keys
    /// and the world runs. An Enter anywhere else lands as it is pressed, since
    /// no tick would come for it — at the threshold none comes at all.
    pub prompt: bool,
}

/// What a frame of the timeline did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Advance {
    /// The key to press this frame.
    pub press: Option<Stroke>,
    /// The step that landed this frame, by index.
    pub landed: Option<usize>,
}

impl Timeline {
    /// `steps`, none started.
    #[must_use]
    pub fn new(steps: Vec<Step>) -> Self {
        let phase = if steps.is_empty() {
            Phase::Done
        } else {
            Phase::Waiting { since: 0.0 }
        };
        let done_at = steps.is_empty().then_some(0.0);
        Self {
            steps,
            next: 0,
            phase,
            clock: 0.0,
            done_at,
        }
    }

    /// Seconds since the scene began.
    #[must_use]
    pub const fn clock(&self) -> f32 {
        self.clock
    }

    /// When every step had landed, if they all have.
    #[must_use]
    pub const fn done_at(&self) -> Option<f32> {
        self.done_at
    }

    /// The steps it plays.
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Move on by `seconds`, as `seen`, with `up` saying what is on screen.
    pub fn advance(&mut self, seconds: f32, seen: Seen, up: &dyn Fn(When) -> bool) -> Advance {
        self.clock += seconds;
        let mut done = Advance::default();
        if let Phase::Waiting { since } = &mut self.phase {
            *since += seconds;
            let since = *since;
            let step = &self.steps[self.next];
            if since >= step.pause(self.next == 0) && step.when.is_none_or(up) {
                self.phase = Phase::Typing {
                    strokes: step.strokes().unwrap_or_default().into(),
                    wait: 0.0,
                };
                self.next += 1;
            }
        }
        match &mut self.phase {
            Phase::Typing { strokes, wait } => {
                *wait -= seconds;
                if *wait <= 0.0 {
                    match strokes.pop_front() {
                        Some(stroke) => {
                            done.press = Some(stroke);
                            *wait = gap(stroke, strokes.front().copied());
                            if strokes.is_empty() {
                                if stroke == Stroke::Named(Named::Enter) && seen.prompt {
                                    self.phase = Phase::Entered;
                                } else {
                                    done.landed = Some(self.land());
                                }
                            }
                        }
                        // Nothing to type lands at once rather than hold the
                        // scene for good.
                        None => done.landed = Some(self.land()),
                    }
                }
            }
            Phase::Entered => self.phase = Phase::Ticking { tick: seen.tick },
            // Moved, not grown: a line that swaps the tower — the installation's
            // `wake` — lands on a new tower counting from nought.
            Phase::Ticking { tick: entered } if seen.tick != *entered => {
                done.landed = Some(self.land());
            }
            Phase::Waiting { .. } | Phase::Ticking { .. } | Phase::Done => {}
        }
        done
    }

    /// The step just finished has landed: the next waits, or the scene's steps
    /// are done. Its index.
    fn land(&mut self) -> usize {
        let landed = self.next - 1;
        if self.next < self.steps.len() {
            self.phase = Phase::Waiting { since: 0.0 };
        } else {
            self.phase = Phase::Done;
            self.done_at = Some(self.clock);
        }
        landed
    }
}

/// Seconds after `pressed` before `next`.
fn gap(pressed: Stroke, next: Option<Stroke>) -> f32 {
    let named = |stroke: Stroke| matches!(stroke, Stroke::Named(_));
    if named(pressed) || next.is_some_and(named) {
        KEYING
    } else {
        TYPING
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A frame at the prompt, with every surface up or none.
    fn frame(timeline: &mut Timeline, seconds: f32, tick: u64, up: bool) -> Advance {
        timeline.advance(seconds, Seen { tick, prompt: true }, &|_| up)
    }

    fn typed(line: &str) -> Step {
        Step {
            line: Some(line.to_owned()),
            ..Step::default()
        }
    }

    #[test]
    fn a_line_is_typed_a_key_a_frame_and_lands_on_the_tick_after_it() {
        let mut timeline = Timeline::new(vec![typed("ok"), typed("go")]);
        // The first step waits out its pause.
        assert_eq!(frame(&mut timeline, 1.0, 0, true).press, None);
        let pressed = |advance: Advance| advance.press;
        assert_eq!(
            pressed(frame(&mut timeline, 0.3, 0, true)),
            Some(Stroke::Char('o'))
        );
        // However long a frame, one key.
        assert_eq!(
            pressed(frame(&mut timeline, 5.0, 0, true)),
            Some(Stroke::Char('k'))
        );
        assert_eq!(
            pressed(frame(&mut timeline, 5.0, 0, true)),
            Some(Stroke::Named(Named::Enter))
        );
        // The tick the line went in on is read next frame; only a later one
        // lands it, so the second line cannot share its tick.
        assert_eq!(frame(&mut timeline, 5.0, 3, true), Advance::default());
        assert_eq!(frame(&mut timeline, 5.0, 3, true), Advance::default());
        assert_eq!(frame(&mut timeline, 0.1, 4, true).landed, Some(0));
        // The second waits its own pause after that.
        assert_eq!(frame(&mut timeline, 0.5, 4, true).press, None);
        assert_eq!(
            frame(&mut timeline, 0.6, 4, true).press,
            Some(Stroke::Char('g'))
        );
        assert_eq!(timeline.done_at(), None);
    }

    #[test]
    fn a_line_that_swaps_the_tower_lands_on_the_new_one() {
        // `wake` raises a tower counting from nought: its tick moves, but down.
        let mut timeline = Timeline::new(vec![typed("w")]);
        let mut landed = None;
        for tick in [9, 9, 9, 9, 0] {
            let advance = frame(&mut timeline, 1.0, tick, true);
            landed = landed.or(advance.landed);
        }
        assert_eq!(landed, Some(0));
    }

    #[test]
    fn an_enter_a_surface_takes_lands_as_it_is_pressed() {
        // The menu, the editor, the threshold: no tick comes for their lines.
        let mut timeline = Timeline::new(vec![typed("k")]);
        let away = Seen {
            tick: 0,
            prompt: false,
        };
        let mut landed = None;
        for _ in 0..4 {
            let advance = timeline.advance(1.0, away, &|_| true);
            landed = landed.or(advance.landed);
        }
        assert_eq!(landed, Some(0));
        assert!(timeline.done_at().is_some());
    }

    #[test]
    fn a_step_with_nothing_to_type_still_lands() {
        let empty = Step {
            keys: Some(String::new()),
            ..Step::default()
        };
        let mut timeline = Timeline::new(vec![empty]);
        for _ in 0..3 {
            frame(&mut timeline, 1.0, 0, true);
        }
        assert!(timeline.done_at().is_some(), "an empty step held the scene");
    }

    #[test]
    fn a_step_waits_for_what_it_needs_up() {
        let step = Step {
            keys: Some("<down>".to_owned()),
            when: Some(When::Editor),
            ..Step::default()
        };
        let mut timeline = Timeline::new(vec![step]);
        for _ in 0..10 {
            assert_eq!(frame(&mut timeline, 1.0, 0, false), Advance::default());
        }
        let pressed = frame(&mut timeline, 0.1, 0, true);
        assert_eq!(pressed.press, Some(Stroke::Named(Named::Down)));
        // No Enter, so it lands as it is pressed.
        assert_eq!(pressed.landed, Some(0));
        assert!(timeline.done_at().is_some());
    }
}
