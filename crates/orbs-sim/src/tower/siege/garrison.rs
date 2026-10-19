//! The tower's standing garrison (§19, *Supply*).
//!
//! Troops stopped being an arsenal item. The tower keeps its own line: a most
//! nothing passes, a floor it never falls below, and desertion between. The
//! menagerie refills it and `defend` levies all of it.
//!
//! Time inside a siege neither pays nor costs (§19 `0.17.0`), so desertion
//! pauses while one runs, and a hold made then waits in the reserve.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use super::{ASSIGNED, SPEARS, besieged};
use crate::tower;

/// Where desertion stops: the king's contingent, so the bailey stands without a
/// menagerie (§19, *"A domain stands alone"*).
pub const GARRISON_FLOOR: u32 = ASSIGNED;

/// The most bodies the tower holds before the Ley Line's `garrison` grant.
pub const GARRISON_MOST: u32 = 10;

/// Ticks between desertions, above the floor.
pub const DESERT_EVERY: u64 = 240;

/// The menagerie's fixture the standing garrison is read from.
///
/// Not the arsenal, which every room reaches and every `Any` verb would trip
/// on, and not the circle, whose `is empty` means *no beast waits*.
pub const ROSTER: &str = "roster";

/// The roster's reading when the garrison is at its most.
pub const FULL: &str = "full";

/// Where the roster stands.
const ROSTER_PATH: &str = "/tower/menagerie/roster";

/// The bodies the tower holds, and the ones that came while it fought.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Garrison {
    standing: u32,
    reserve: u32,
}

impl Default for Garrison {
    fn default() -> Self {
        Self {
            standing: GARRISON_FLOOR,
            reserve: 0,
        }
    }
}

impl Garrison {
    /// The bodies standing at home. Nought while they are on the wall.
    #[must_use]
    pub const fn standing(self) -> u32 {
        self.standing
    }

    /// Bodies a hold brought while a siege ran, waiting to join at its end.
    #[must_use]
    pub const fn reserve(self) -> u32 {
        self.reserve
    }

    /// Everyone at home: the standing and the reserve.
    #[must_use]
    pub const fn home(self) -> u32 {
        self.standing.saturating_add(self.reserve)
    }

    /// Put both counts back, for a save.
    pub const fn restore(&mut self, standing: u32, reserve: u32) {
        self.standing = standing;
        self.reserve = reserve;
    }
}

/// The most the tower may hold, its grant included.
#[must_use]
pub fn most_held(world: &World) -> u32 {
    GARRISON_MOST.saturating_add(tower::grant::garrison_bonus(world))
}

/// What the line levied for a siege still being fought, if one is.
fn on_the_wall(world: &mut World) -> Option<u32> {
    world
        .query::<&super::Siege>()
        .iter(world)
        .find(|siege| siege.running())
        .map(|siege| siege.levied)
}

/// Every body the tower holds: at home, and the levied line while a siege
/// runs, which comes home at its end.
///
/// The one count a hold is measured against, the roster reads and `status`
/// prints, so none of them says there is room where a hold would be turned
/// away.
pub fn held(world: &mut World) -> u32 {
    let line = on_the_wall(world).unwrap_or(0);
    world.resource::<Garrison>().home().saturating_add(line)
}

/// Bring `bodies` home from a hold. Returns how many joined and how many were
/// turned away at the most.
///
/// Mid-siege the levied line counts too ([`held`]): a reserve filled past what
/// it leaves room for would be clamped away at `disband` after the player was
/// told the bodies were waiting.
pub fn enlist(world: &mut World, bodies: u32) -> (u32, u32) {
    let most = most_held(world);
    let fighting = on_the_wall(world).is_some();
    let joined = bodies.min(most.saturating_sub(held(world)));
    let mut garrison = world.resource_mut::<Garrison>();
    if fighting {
        garrison.reserve += joined;
    } else {
        garrison.standing += joined;
    }
    (joined, bodies - joined)
}

/// Send everyone standing to the wall, and say how many went.
///
/// Never fewer than the floor: a hand-edited document can say nought, and a
/// siege with no line is no decision.
pub fn levy(world: &mut World) -> u32 {
    let mut garrison = world.resource_mut::<Garrison>();
    let levied = garrison.standing.max(GARRISON_FLOOR);
    garrison.standing = 0;
    levied
}

/// Bring the line home once a siege ends.
///
/// Survivors come home up to what was levied, so a refill past the levy
/// disbands. A refilled casualty comes home as a healed one would. The reserve
/// joins them, and a fallen garrison is reassigned the floor.
pub fn disband(world: &mut World, survivors: u32, levied: u32) {
    let most = most_held(world);
    let mut garrison = world.resource_mut::<Garrison>();
    garrison.standing = survivors
        .min(levied)
        .saturating_add(garrison.reserve)
        .clamp(GARRISON_FLOOR, most.max(GARRISON_FLOOR));
    garrison.reserve = 0;
}

/// One body leaves every [`DESERT_EVERY`] ticks, down to the floor.
///
/// Keyed on the clock, so a watched hour and a `meditate`-collapsed one lose the
/// same bodies. Paused while a siege runs.
pub fn desert(world: &mut World) {
    let now = world.resource::<crate::tick::Tick>().get();
    if now == 0 || !now.is_multiple_of(DESERT_EVERY) || besieged(world) {
        return;
    }
    let mut garrison = world.resource_mut::<Garrison>();
    if garrison.standing > GARRISON_FLOOR {
        garrison.standing -= 1;
    }
}

/// Keep the roster's readings in step with the garrison: `spears` for every
/// body it [`held`], and `full` at the most.
///
/// A system that writes only on a change, `stocktake`'s rule, so an idle tick
/// issues no `NodeId`. Scheduled only when the garrison or the Ley grants
/// moved, the two things it reads besides the levy `defend` makes with them.
pub fn roll_call(world: &mut World) {
    let Some(roster) = tower::find_by_path(world, ROSTER_PATH) else {
        return;
    };
    let held = held(world);
    let mut want: Vec<(&str, Option<u32>)> = Vec::new();
    if held > 0 {
        want.push((SPEARS, Some(held)));
    }
    if held >= most_held(world) {
        want.push((FULL, None));
    }
    tower::sync_readings(world, roster, &[SPEARS, FULL], &want);
}
