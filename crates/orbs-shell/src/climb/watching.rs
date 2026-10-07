//! Every number on screen that rolls, the clock, and the views painters ask.

use bevy_ecs::prelude::Resource;
use orbs_sim::Toward;

use super::rise::{Reading, Rise, Rolled, Scale, named, see};
use crate::glance::Panel;

/// The one name every stock is known by: each has a slot of its own, so the
/// name has nothing to tell apart.
const STOCK: &str = "";

/// A number on screen that rolls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Watch {
    /// The Ley Line gauge.
    Ley,
    /// The renown gauge.
    Renown,
    /// What the room in front of the player has made, in all.
    Room,
    /// How much of that a spell did.
    Spells,
    /// The road's count toward the room's next station.
    Road,
    /// What is left to pledge in a siege.
    Coffer,
    /// The garrison's vigour.
    Garrison,
    /// The enemy's vigour.
    Enemy,
}

/// Every rolling number on screen, and how far each has got.
///
/// A short list keyed by [`Watch`] rather than an array indexed by it, so a
/// new variant needs no count kept in step. At most one entry a watch, and the
/// capacity is kept, so it allocates on the first frames and not after.
#[derive(Resource, Debug, Default)]
pub struct Climb {
    rises: Vec<(Watch, Rise)>,
    enabled: bool,
}

impl Climb {
    /// What to draw for a gauge whose truth is `truth`.
    ///
    /// The truth itself unless this climb has watched it change and is still
    /// moving — so a climb that was never advanced draws everything settled.
    #[must_use]
    pub fn toward(&self, watch: Watch, truth: Toward) -> Rolled {
        self.view(watch, truth.done, Reading::tier(truth))
    }

    /// What to draw for a count of the thing called `of`, whose truth is
    /// `truth`.
    #[must_use]
    pub fn count(&self, watch: Watch, of: &str, truth: u64) -> Rolled {
        self.view(watch, truth, Scale::Of(named(of)))
    }

    /// What to draw for a stock with one slot of its own — the siege's coffer
    /// and two sides — whose truth is `truth`.
    #[must_use]
    pub fn stock(&self, watch: Watch, truth: u64) -> Rolled {
        self.count(watch, STOCK, truth)
    }

    fn view(&self, watch: Watch, value: u64, scale: Scale) -> Rolled {
        match self.rise(watch) {
            Some(rise)
                if self.enabled && rise.truth.value == value && rise.truth.scale == scale =>
            {
                rise.showing()
            }
            _ => Rolled::settled(value),
        }
    }

    fn rise(&self, watch: Watch) -> Option<Rise> {
        self.rises
            .iter()
            .find(|(watched, _)| *watched == watch)
            .map(|(_, rise)| *rise)
    }

    /// One watch's edge, kept in the list.
    fn edge(&mut self, watch: Watch, reading: Option<Reading>) {
        let at = self.rises.iter().position(|(watched, _)| *watched == watch);
        let mut slot = at.map(|index| self.rises[index].1);
        see(&mut slot, reading, self.enabled);
        match (at, slot) {
            (Some(index), Some(rise)) => self.rises[index].1 = rise,
            (Some(index), None) => {
                self.rises.swap_remove(index);
            }
            (None, Some(rise)) => self.rises.push((watch, rise)),
            (None, None) => {}
        }
    }

    /// Note what the panel says now, and start a roll for anything that moved.
    pub fn observe(&mut self, panel: &Panel) {
        let ledger = panel.ledger.as_ref();
        let road = panel.line.as_ref().and_then(orbs_sim::Line::next);
        let siege = panel.rampart.as_ref();
        let stock = |value: u32| Reading::count(u64::from(value), STOCK);
        let readings = [
            (Watch::Ley, Reading::toward(panel.station, panel.experience)),
            (Watch::Renown, Reading::toward(panel.rankward, panel.renown)),
            (
                Watch::Room,
                ledger.map(|row| Reading::count(u64::from(row.count), &row.id)),
            ),
            (
                Watch::Spells,
                ledger.map(|row| Reading::count(u64::from(row.by_spell), &row.id)),
            ),
            (
                Watch::Road,
                road.map(|stop| Reading::count(u64::from(stop.done), &stop.id)),
            ),
            (Watch::Coffer, siege.map(|siege| stock(siege.quintessence))),
            (
                Watch::Garrison,
                siege.map(|siege| stock(siege.garrison.vigour)),
            ),
            (Watch::Enemy, siege.map(|siege| stock(siege.enemy.vigour))),
        ];
        for (watch, reading) in readings {
            self.edge(watch, reading);
        }
    }

    /// Carry every roll forward by `seconds`. Forward only, for
    /// `Bench::tick`'s reason.
    pub fn tick(&mut self, seconds: f32) {
        let seconds = seconds.max(0.0);
        for (_, rise) in &mut self.rises {
            rise.tick(seconds);
        }
    }

    /// Turn rolling on or off. Off draws every number as it is.
    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Put everything in flight `fraction` of the way through its roll, for a
    /// dump. The `+N` is posed on its own longer clock at the same moment.
    ///
    /// A dump advances no `Time`, so without this a roll could only be seen in a
    /// window. See `ORBS_ROLL_AT`.
    pub fn pose(&mut self, fraction: f32) {
        self.enabled = true;
        let fraction = if fraction.is_nan() {
            0.0
        } else {
            fraction.clamp(0.0, 1.0)
        };
        for (_, rise) in &mut self.rises {
            rise.pose(fraction);
        }
    }

    /// Carry the climb forward one frame.
    ///
    /// `motion` is whether the frontend allows movement at all; `None` leaves it
    /// as it was, which from `Default` is off.
    pub fn advance(&mut self, delta: f32, motion: Option<bool>, panel: &Panel) {
        if let Some(on) = motion {
            self.set_enabled(on);
        }
        self.observe(panel);
        self.tick(delta);
    }
}
