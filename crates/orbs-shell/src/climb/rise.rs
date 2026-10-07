//! One number: what it said, where its roll began, and what to draw now.

use orbs_render::{Fill, Intensity, PLUS_SECS, ROLL_SECS, Roll, fading};
use orbs_sim::{Ahead, Toward};

/// What a number is measured against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Scale {
    /// A gauge's tier. A change is a crossing, never a roll: `15/16 → 2/8`
    /// rolled in one scale would count *down*, a gain drawn as a loss.
    Tier { span: u64, ahead: Ahead },
    /// A count of one named thing — a room's row, a station. A change of name
    /// is a different number, so it settles rather than rolls.
    Of(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Reading {
    /// What is drawn.
    pub(super) value: u64,
    pub(super) scale: Scale,
    /// The total whose rise is a gain — experience, not the tier-relative
    /// `done`, so a `+N` says what was earned.
    pub(super) whole: u64,
}

impl Reading {
    /// `None` for a track nobody has measured, which is not a reading of nought.
    pub(super) fn toward(toward: Toward, whole: u64) -> Option<Self> {
        (toward.ahead != Ahead::Unasked).then_some(Self {
            value: toward.done,
            scale: Self::tier(toward),
            whole,
        })
    }

    pub(super) const fn tier(toward: Toward) -> Scale {
        Scale::Tier {
            span: toward.span,
            ahead: toward.ahead,
        }
    }

    /// A count of the thing called `of`.
    pub(super) fn count(value: u64, of: &str) -> Self {
        Self {
            value,
            scale: Scale::Of(named(of)),
            whole: value,
        }
    }
}

/// A name as an identity, without keeping the string.
///
/// FNV-1a. Only ever compared with the same name a frame later, so a collision
/// would cost one roll across a change of room, and the names are a handful.
pub(super) fn named(name: &str) -> u64 {
    name.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// A crossing in progress: the tier being left, filled to its end.
#[derive(Debug, Clone, Copy)]
struct Beat {
    /// The span of the tier being left.
    span: u64,
    /// Where the tower stood in it, for its colour.
    stood: u64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Rise {
    /// What the panel last said.
    pub(super) truth: Reading,
    /// What was showing when the roll began.
    from: u64,
    /// Seconds since it began, held at [`ROLL_SECS`] once arrived.
    elapsed: f32,
    beat: Option<Beat>,
    /// The gain a `+N` is showing, nought for none.
    plus: u64,
    /// Seconds since the `+N` appeared, held at [`PLUS_SECS`] once gone.
    plus_elapsed: f32,
}

impl Rise {
    const fn settled(truth: Reading) -> Self {
        Self {
            truth,
            from: truth.value,
            elapsed: ROLL_SECS,
            beat: None,
            plus: 0,
            plus_elapsed: PLUS_SECS,
        }
    }

    fn rolling(self) -> bool {
        self.elapsed < ROLL_SECS
    }

    fn roll(self) -> Roll {
        Roll {
            from: self.from,
            to: self.truth.value,
            t: self.elapsed / ROLL_SECS,
        }
    }

    /// What is on screen now, a beat's tier included.
    pub(super) fn showing(self) -> Rolled {
        let plus = (self.plus > 0)
            .then(|| fading(self.plus_elapsed / PLUS_SECS))
            .flatten()
            .map(|weight| (self.plus, weight));
        // Arrived is the truth exactly, `from` included: a reading reserves its
        // width from `from`, and a stale one would pad a settled row for ever.
        if !self.rolling() {
            return Rolled {
                plus,
                ..Rolled::settled(self.truth.value)
            };
        }
        let rolled = Rolled {
            shown: self.roll().shown(),
            from: self.from,
            span: None,
            hue: None,
            plus,
        };
        let Some(beat) = self.beat else {
            return rolled;
        };
        // The tier left as it stood, then arrived, then landed. The first flip
        // tick changes nothing, like any roll's, so a crossing seen just after a
        // digit turned cannot turn another inside one tick.
        match self.roll().step() {
            0 => Rolled {
                shown: self.from,
                span: Some(beat.span),
                hue: Some(Fill::of(count(beat.stood), count(beat.span))),
                ..rolled
            },
            1 | 2 => Rolled {
                shown: beat.span,
                span: Some(beat.span),
                hue: Some(Fill::Whole),
                ..rolled
            },
            _ => rolled,
        }
    }

    /// Carry it forward by `seconds`.
    pub(super) fn tick(&mut self, seconds: f32) {
        self.elapsed = (self.elapsed + seconds).min(ROLL_SECS);
        self.plus_elapsed = (self.plus_elapsed + seconds).min(PLUS_SECS);
    }

    /// Put it `fraction` of the way through its roll, and its `+N` the same
    /// number of seconds into its own longer clock.
    pub(super) fn pose(&mut self, fraction: f32) {
        self.elapsed = fraction * ROLL_SECS;
        if self.plus > 0 {
            self.plus_elapsed = fraction * ROLL_SECS;
        }
    }
}

/// One number's edge.
///
/// First sight settles, and so does a number that left the screen and came
/// back: it is evicted on the frame it is absent, unlike the fire's
/// `observe_hearth`, because a room worked by a spell while the player was
/// elsewhere would otherwise roll on their return — a view change drawn as a
/// gain.
pub(super) fn see(slot: &mut Option<Rise>, reading: Option<Reading>, enabled: bool) {
    let Some(now) = reading else {
        *slot = None;
        return;
    };
    let Some(rise) = slot.as_mut() else {
        *slot = Some(Rise::settled(now));
        return;
    };
    if rise.truth == now {
        return;
    }
    let was = *rise;
    let same_thing = match (was.truth.scale, now.scale) {
        (Scale::Of(before), Scale::Of(after)) => before == after,
        (Scale::Tier { .. }, Scale::Tier { .. }) => true,
        _ => false,
    };
    if !enabled || !same_thing {
        *rise = Rise::settled(now);
        return;
    }

    let gained = now.whole.saturating_sub(was.truth.whole);
    if gained > 0 {
        // Summed while the roll it belongs to is still moving, so a hitch that
        // lands two gains at once says their total. After that a gain is news
        // of its own: summing for as long as the `+N` stays would count up for
        // ever under a spell that earns every tick.
        let showing = was.rolling() && was.plus > 0;
        rise.plus = if showing { was.plus + gained } else { gained };
        rise.plus_elapsed = 0.0;
    } else if now.whole < was.truth.whole {
        // A loss has its own record; a `+N` still fading would contradict it.
        rise.plus = 0;
        rise.plus_elapsed = PLUS_SECS;
    }

    rise.truth = now;
    rise.beat = None;
    if was.beat.is_some() && was.rolling() {
        // Mid-beat, cut: a second crossing folded into the first would draw an
        // arrival that has already been superseded.
        rise.from = now.value;
        rise.elapsed = ROLL_SECS;
    } else if was.truth.scale == now.scale {
        // Retargeted from what is showing, so a change mid-roll carries on from
        // where the eye is rather than jumping back to the old start.
        rise.from = was.showing().shown;
        rise.elapsed = 0.0;
    } else if let (Scale::Tier { span, .. }, true) = (was.truth.scale, gained > 0) {
        // One beat however many tiers were crossed: the tier left is the one on
        // screen, and every tier between was never drawn.
        rise.beat = Some(Beat {
            span,
            stood: was.truth.value,
        });
        rise.from = was.showing().shown;
        rise.elapsed = 0.0;
    } else {
        // A fall across a tier, or off the top of the track: cut.
        rise.from = now.value;
        rise.elapsed = ROLL_SECS;
    }
}

/// A count as a gauge takes it.
fn count(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// What a painter draws for one rolling number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rolled {
    /// The value to draw now.
    pub shown: u64,
    /// Where the roll started — what a reading reserves its width for, so the
    /// row does not move as the digits change.
    pub from: u64,
    /// The span `shown` is out of, while a crossing still draws the tier it is
    /// leaving. `None` is the truth's.
    pub span: Option<u64>,
    /// The fill's colour, while a crossing draws the tier it is leaving and
    /// then arrives. `None` is the truth's.
    pub hue: Option<Fill>,
    /// A gain, and the weight its `+N` is drawn at while it fades.
    pub plus: Option<(u64, Intensity)>,
}

impl Rolled {
    /// A number that is not moving.
    #[must_use]
    pub const fn settled(value: u64) -> Self {
        Self {
            shown: value,
            from: value,
            span: None,
            hue: None,
            plus: None,
        }
    }

    /// The wider of the value shown at the start and the truth, for a reading
    /// that must not move as its digits turn.
    #[must_use]
    pub fn widest(self, truth: u64) -> u64 {
        self.from.max(truth)
    }
}
