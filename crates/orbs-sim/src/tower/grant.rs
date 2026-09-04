//! What a Ley Line fork node grants, and how much (DESIGN.md §11.5, §19).
//!
//! # The id is the contract
//!
//! `progression.toml` says *"ids are decisions, not prose"*: a fork node's id
//! is what a taken node is stored as, what its sentence is keyed by, and — here
//! — what it does. `<stem>_<n>` parses to a [`Grant`] with a tier, and every
//! reader below sums the tiers of the nodes the orb has taken. One parser, no
//! second table to drift; `Progression::check` holds every authored node to it.
//!
//! # Three lanes, and what each may touch
//!
//! **Craft** is the orb — steps, the satchel, cursors, and `haste`, which is
//! struck invariant 3 landing *"as a thing you buy"*: a run a spell issued
//! finishes sooner. **Provision** is the tower's supplies: fuel, the pool, what
//! a siege pays, what a charm costs. **War** is the wall and the dice: an edge on
//! every answering roll, the pool's floor under a worn wall, a troop's worth, a
//! course's mending, how often the calm layer strikes.
//!
//! **A grant never duplicates a charm.** The forge sells *temporary* instrument
//! buffs, and a permanent copy on this line would delete the domain — so
//! nothing here makes an instrument faster or a run yield more. `haste` is
//! narrower than `hurried` on purpose: it is the orb's speed, not the tool's.
//!
//! # Tiers add
//!
//! A player who takes `fuel_1` at one fork and `fuel_2` at a later one holds
//! three tiers of fuel, and each reader multiplies its unit by the tiers held.
//! Reading them as a maximum would refund the second choice in silence, which
//! is `steps_<n>`'s rule and the reason it is the rule everywhere.

use bevy_ecs::prelude::*;

use super::ley::Taken;

/// Which kind of play a grant alters.
///
/// The designer's own three: *"more resources vs better combat vs faster
/// tools"*. The painter orders a fork's siblings by this, so the same lane is
/// always in the same row, and `Progression::check` refuses a fork with two
/// nodes in one lane — a choice is always between kinds of play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lane {
    /// More resources: fuel, the pool, what a siege pays, what a charm costs.
    Provision,
    /// Better combat: the wall and the dice.
    War,
    /// A faster orb: steps, the satchel, cursors, haste.
    Craft,
}

impl Lane {
    /// The word a screen reader hears, and the prose key's suffix.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Provision => "provision",
            Self::War => "war",
            Self::Craft => "craft",
        }
    }
}

/// What a real fork node actually does, with its tier where it has one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grant {
    /// `steps_<n>` — that many more instructions a tick.
    Steps(usize),
    /// `satchel_1` — `queue` and `pull` work at all (§8's channel).
    Satchel,
    /// `cursors_1` — `alongside` works: two places in one spell.
    Cursors,
    /// `haste_<n>` — a run a spell issued finishes [`HASTE_PERCENT`] × n sooner.
    Haste(usize),
    /// `fuel_<n>` — a charcoal burns [`FUEL_PERCENT`] × n longer.
    Fuel(usize),
    /// `edge_<n>` — [`EDGE_BONUS`] × n on every answering roll at the wall.
    Edge(usize),
    /// `pool_<n>` — [`POOL_BONUS`] × n on the quintessence ceiling.
    Pool(usize),
    /// `floor_<n>` — the pool's floor under a worn wall rises [`FLOOR_BONUS`] × n
    /// points.
    Floor(usize),
    /// `escrow_<n>` — a siege pays [`ESCROW_PERCENT`] × n more.
    Escrow(usize),
    /// `garrison_<n>` — a deployed troop is worth [`GARRISON_BONUS`] × n more.
    Garrison(usize),
    /// `mend_<n>` — a finished course mends [`MEND_BONUS`] × n more.
    Mend(usize),
    /// `vigilance_<n>` — the calm layer strikes [`VIGILANCE_PERCENT`] × n less
    /// often.
    Vigilance(usize),
    /// `thrift_<n>` — a charm costs [`THRIFT`] × n less.
    Thrift(usize),
}

/// Per tier of `haste`: how much sooner a scripted run lands, in percent.
pub const HASTE_PERCENT: u64 = 10;
/// Per tier of `fuel`: how much longer a charcoal burns, in percent.
pub const FUEL_PERCENT: u64 = 20;
/// Per tier of `edge`: the bonus on every answering roll.
pub const EDGE_BONUS: i32 = 1;
/// Per tier of `pool`: quintessence on the ceiling.
pub const POOL_BONUS: u32 = 4;
/// Per tier of `floor`: points on the pool's integrity floor.
pub const FLOOR_BONUS: u32 = 10;
/// Per tier of `escrow`: how much more a siege pays, in percent.
pub const ESCROW_PERCENT: u64 = 25;
/// Per tier of `garrison`: extra bodies per troop deployed.
pub const GARRISON_BONUS: u32 = 1;
/// Per tier of `mend`: integrity a finished course puts back on top.
pub const MEND_BONUS: u32 = 3;
/// Per tier of `vigilance`: how much less often the calm layer strikes, in
/// percent.
pub const VIGILANCE_PERCENT: u64 = 25;
/// Per tier of `thrift`: quintessence off a charm's price.
pub const THRIFT: u32 = 2;

/// The stems a fork node's id may begin with, for `Progression::check`'s
/// message.
pub const STEMS: [&str; 13] = [
    "steps",
    "satchel",
    "cursors",
    "haste",
    "fuel",
    "edge",
    "pool",
    "floor",
    "escrow",
    "garrison",
    "mend",
    "vigilance",
    "thrift",
];

impl Grant {
    /// Which lane this grant sits in.
    #[must_use]
    pub const fn lane(self) -> Lane {
        match self {
            Self::Steps(_) | Self::Satchel | Self::Cursors | Self::Haste(_) => Lane::Craft,
            Self::Fuel(_) | Self::Pool(_) | Self::Escrow(_) | Self::Thrift(_) => Lane::Provision,
            Self::Edge(_)
            | Self::Floor(_)
            | Self::Garrison(_)
            | Self::Mend(_)
            | Self::Vigilance(_) => Lane::War,
        }
    }
}

/// What `id` grants, or `None` for a word the orb cannot read.
///
/// **One parser, and `Progression::check` holds every fork node to it** — a
/// typo'd `satchel1` would otherwise be a node that draws and grants nothing.
#[must_use]
pub fn granted(id: &str) -> Option<Grant> {
    match id {
        "satchel_1" => return Some(Grant::Satchel),
        "cursors_1" => return Some(Grant::Cursors),
        _ => {}
    }
    let (stem, tier) = id.rsplit_once('_')?;
    let tier: usize = tier.parse().ok().filter(|tier| *tier > 0)?;
    Some(match stem {
        "steps" => Grant::Steps(tier),
        "haste" => Grant::Haste(tier),
        "fuel" => Grant::Fuel(tier),
        "edge" => Grant::Edge(tier),
        "pool" => Grant::Pool(tier),
        "floor" => Grant::Floor(tier),
        "escrow" => Grant::Escrow(tier),
        "garrison" => Grant::Garrison(tier),
        "mend" => Grant::Mend(tier),
        "vigilance" => Grant::Vigilance(tier),
        "thrift" => Grant::Thrift(tier),
        _ => return None,
    })
}

/// What a fork node grants, in extra spell steps a tick.
#[must_use]
pub fn steps_granted(id: &str) -> Option<usize> {
    match granted(id) {
        Some(Grant::Steps(many)) => Some(many),
        _ => None,
    }
}

/// The tiers held of whichever grant `pick` selects, summed over what the orb
/// has taken.
fn tiers_in(taken: &Taken, pick: impl Fn(Grant) -> Option<usize>) -> usize {
    taken
        .ids()
        .iter()
        .filter_map(|id| granted(id))
        .filter_map(pick)
        .sum()
}

/// The same, read off the world.
///
/// **Nought before the resource exists**, which is `spell::budget`'s rule and
/// for its reason: the quintessence ceiling is read while `Sim::bare` is still
/// raising the tower, and a reader that panicked there would make the order
/// resources are inserted in a load-bearing fact nobody can see.
fn tiers(world: &World, pick: impl Fn(Grant) -> Option<usize>) -> usize {
    world
        .get_resource::<Taken>()
        .map_or(0, |taken| tiers_in(taken, pick))
}

/// How much sooner a run a spell issued lands, in percent.
#[must_use]
pub fn haste_percent(world: &World) -> u64 {
    let tiers = tiers(world, |grant| match grant {
        Grant::Haste(tier) => Some(tier),
        _ => None,
    });
    (HASTE_PERCENT * tiers as u64).min(90)
}

/// How much longer a charcoal burns, in percent.
#[must_use]
pub fn fuel_percent(world: &World) -> u64 {
    FUEL_PERCENT
        * tiers(world, |grant| match grant {
            Grant::Fuel(tier) => Some(tier),
            _ => None,
        }) as u64
}

/// The bonus on every answering roll at the wall.
#[must_use]
pub fn edge_bonus(world: &World) -> i32 {
    EDGE_BONUS
        * i32::try_from(tiers(world, |grant| match grant {
            Grant::Edge(tier) => Some(tier),
            _ => None,
        }))
        .unwrap_or(i32::MAX)
}

/// Quintessence added to the ceiling.
#[must_use]
pub fn pool_bonus(world: &World) -> u32 {
    POOL_BONUS.saturating_mul(
        u32::try_from(tiers(world, |grant| match grant {
            Grant::Pool(tier) => Some(tier),
            _ => None,
        }))
        .unwrap_or(u32::MAX),
    )
}

/// Points added to the pool's floor under a worn wall.
#[must_use]
pub fn floor_bonus(world: &World) -> u32 {
    FLOOR_BONUS.saturating_mul(
        u32::try_from(tiers(world, |grant| match grant {
            Grant::Floor(tier) => Some(tier),
            _ => None,
        }))
        .unwrap_or(u32::MAX),
    )
}

/// How much more a siege pays, in percent.
#[must_use]
pub fn escrow_percent(world: &World) -> u64 {
    ESCROW_PERCENT
        * tiers(world, |grant| match grant {
            Grant::Escrow(tier) => Some(tier),
            _ => None,
        }) as u64
}

/// Extra bodies per troop deployed.
#[must_use]
pub fn garrison_bonus(world: &World) -> u32 {
    GARRISON_BONUS.saturating_mul(
        u32::try_from(tiers(world, |grant| match grant {
            Grant::Garrison(tier) => Some(tier),
            _ => None,
        }))
        .unwrap_or(u32::MAX),
    )
}

/// Integrity a finished course puts back on top of its wards.
#[must_use]
pub fn mend_bonus(world: &World) -> u32 {
    MEND_BONUS.saturating_mul(
        u32::try_from(tiers(world, |grant| match grant {
            Grant::Mend(tier) => Some(tier),
            _ => None,
        }))
        .unwrap_or(u32::MAX),
    )
}

/// How much less often the calm layer strikes, in percent — read off
/// [`Taken`] directly, because the two sabotage systems are queries and cannot
/// ask the world.
#[must_use]
pub fn vigilance_percent(taken: &Taken) -> u64 {
    (VIGILANCE_PERCENT
        * tiers_in(taken, |grant| match grant {
            Grant::Vigilance(tier) => Some(tier),
            _ => None,
        }) as u64)
        .min(75)
}

/// Quintessence off a charm's price.
#[must_use]
pub fn thrift(world: &World) -> u32 {
    THRIFT.saturating_mul(
        u32::try_from(tiers(world, |grant| match grant {
            Grant::Thrift(tier) => Some(tier),
            _ => None,
        }))
        .unwrap_or(u32::MAX),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stem_parses_with_a_tier_and_nothing_else_does() {
        for stem in STEMS {
            let id = format!("{stem}_1");
            assert!(granted(&id).is_some(), "{id} did not parse");
        }
        for bad in [
            "satchel1",
            "steps_",
            "haste_0",
            "fuel",
            "tbi_b",
            "edge_x",
            "steps_two",
        ] {
            assert!(granted(bad).is_none(), "{bad} parsed");
        }
        assert_eq!(granted("steps_2"), Some(Grant::Steps(2)));
        assert_eq!(granted("vigilance_1"), Some(Grant::Vigilance(1)));
    }

    #[test]
    fn every_grant_has_a_lane_and_the_lanes_are_the_three() {
        let lanes: std::collections::BTreeSet<Lane> = STEMS
            .iter()
            .filter_map(|stem| granted(&format!("{stem}_1")))
            .map(Grant::lane)
            .collect();
        assert_eq!(lanes.len(), 3);
        assert_eq!(Grant::Haste(1).lane(), Lane::Craft);
        assert_eq!(Grant::Fuel(1).lane(), Lane::Provision);
        assert_eq!(Grant::Edge(1).lane(), Lane::War);
    }

    #[test]
    fn tiers_add_across_forks() {
        let mut taken = Taken::default();
        taken.hold("fuel_1");
        taken.hold("fuel_2");
        taken.hold("edge_1");
        let fuel = tiers_in(&taken, |grant| match grant {
            Grant::Fuel(tier) => Some(tier),
            _ => None,
        });
        assert_eq!(fuel, 3, "two fuel nodes did not add");
        assert_eq!(vigilance_percent(&taken), 0);
        taken.hold("vigilance_1");
        assert_eq!(vigilance_percent(&taken), VIGILANCE_PERCENT);
    }
}
