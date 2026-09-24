//! What the tower is known for (DESIGN.md §11.5, §19).
//!
//! [`Experience`](super::Experience) only rises, and past the Ley Line's last
//! station it buys nothing. Renown is what the work is worth after that, and the
//! one number a player can lose. [`lose`] saturates at nought — a signed total
//! would put a sign in every reader for a state nothing uses.
//!
//! Minted on [`Work::sold`](super::Work::sold), not on [`done`](super::done):
//! every completion goes through that door, so minting there would pay for
//! reading a book and pay a siege twice through escrow.
//!
//! Renown draws nothing, so it needs no `RngStream`. Stored rather than derived,
//! because its history is the only thing that knows it.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::session::Scrollback;

/// What one made thing is worth in renown, against what it earned.
///
/// A share of the run's experience, so one authored table prices both numbers.
/// Rounded *up*: the mortar and pestle earns 1, and a floored halving would have
/// the apprenticeship's first instrument mint nothing at all.
#[must_use]
pub const fn worth(earned: u64) -> u64 {
    earned.div_ceil(RENOWN_PER)
}

/// How much experience one renown costs.
/// `pub(crate)` so the siege prices a fight at the same rate, and making a thing
/// and winning a fight cannot drift apart as separate numbers.
pub(crate) const RENOWN_PER: u64 = 2;

/// What the tower is known for.
///
/// The one number that falls. Everything else the tower holds is a faucet or a
/// pool with a floor; a reputation is the only thing a bad night can take away.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Renown(u64);

impl Renown {
    /// Put a total back, for a save.
    ///
    /// Not [`earn`], for `Experience::restore`'s reason: a load that
    /// congratulated you on yesterday's work would be lying in voice.
    pub(crate) const fn restore(&mut self, total: u64) {
        self.0 = total;
    }

    /// The total held.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Add what a making was worth. Silently.
///
/// Called from [`done`](super::done) where the work put something on a shelf.
///
/// A gain is not an event: a line per making took one sweep of the clarity loop
/// from 466 records to 792. The edge that speaks is the rank (`crossed`).
pub fn earn(world: &mut World, gained: u64) {
    if gained == 0 {
        return;
    }
    let before = reached(world);
    world.resource_mut::<Renown>().0 = world.resource::<Renown>().0.saturating_add(gained);
    crossed(world, before);
}

/// Put the total somewhere, for a tester. Both directions, and it says so.
///
/// The only door that lowers renown without a lost siege, and each rank sits
/// behind an hour of play.
#[cfg(debug_assertions)]
pub fn set(world: &mut World, total: u64) {
    let before = reached(world);
    world.resource_mut::<Renown>().0 = total;
    crossed(world, before);
}

/// How many ranks the tower has reached. Nought is a tower nobody has heard of.
///
/// A count, not a name: falling from magister to cunning man leaves a rank in
/// hand, so comparing *names* read a fall as an arrival.
pub(crate) fn reached(world: &World) -> usize {
    let held = world.resource::<Renown>().get();
    world
        .resource::<crate::content::Progression>()
        .renown()
        .iter()
        .take_while(|step| step.at <= held)
        .count()
}

/// Say the title, if the move changed it.
///
/// An edge, not a state: the name is continuous and derived, so it would
/// otherwise be true in silence for ever. Both directions are news, and a fall
/// speaks once rather than once per rank passed.
fn crossed(world: &mut World, before: usize) {
    let after = reached(world);
    if after == before {
        return;
    }
    let climbed = after > before;
    let id = if climbed {
        rank(world)
    } else {
        // Falling: name where the tower landed, and if that is nowhere, the
        // title it just lost — *"they have stopped calling you magister"*.
        rank(world).or_else(|| {
            world
                .resource::<crate::content::Progression>()
                .renown()
                .get(before.saturating_sub(1))
                .map(|step| step.id.clone())
        })
    };
    let Some(id) = id else {
        return;
    };
    let title = world.resource::<Prose>().line(&format!("renown_{id}"), &[]);
    // Three sentences, not two: a fall that still leaves a title must not read
    // as an arrival.
    let key = match (climbed, after) {
        (true, _) => "rank_reached",
        (false, 0) => "rank_lost",
        (false, _) => "rank_fallen",
    };
    let message = world.resource::<Prose>().line(key, &[("name", &title)]);
    let role = if climbed { Role::Success } else { Role::Danger };
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, &id)
        .text(FieldName::Message, &message)
        .role(role)
        .finish();
}

/// Take renown away, to nought and no further, and say so.
///
/// Unlike [`earn`]: a loss happened *to* the player, at a moment they may not
/// have been watching, and §6 forbids a state changing under someone in silence.
/// Saturating rather than signed — owing renown is a state nothing could draw.
pub fn lose(world: &mut World, cost: u64) {
    if cost == 0 {
        return;
    }
    let was = reached(world);
    let (fell, now) = {
        let mut renown = world.resource_mut::<Renown>();
        let before = renown.0;
        renown.0 = renown.0.saturating_sub(cost);
        (before - renown.0, renown.0)
    };
    if fell == 0 {
        return;
    }
    say(world, "renown_lost", fell, now, Role::Danger);
    crossed(world, was);
}

/// Take renown away without the sentence, for a loss the player is watching.
///
/// A siege round has already said what the enemy did, so [`lose`]'s sentence
/// would retell it six to thirteen times a fight (§19). A rank crossing still
/// speaks through the shared `crossed`, and the siege says the whole movement
/// once on settling. Saturating, exactly as [`lose`] is.
pub fn slip(world: &mut World, cost: u64) {
    if cost == 0 {
        return;
    }
    let was = reached(world);
    world.resource_mut::<Renown>().0 = world.resource::<Renown>().0.saturating_sub(cost);
    crossed(world, was);
}

/// Pay `cost` out of standing, or say there is not enough. Never partial.
///
/// `Quintessence::spend`'s shape: the fiction has no word for a half-paid
/// petition. `false` and nothing moved, so a caller may ask before it commits.
/// Quiet like [`slip`], since what is bought says its own price.
#[must_use]
pub fn spend(world: &mut World, cost: u64) -> bool {
    if cost > world.resource::<Renown>().get() {
        return false;
    }
    slip(world, cost);
    true
}

/// Where a number stands between the tier behind it and the one ahead.
///
/// Both bounds, not just the target: filling from nought would jump the gauge
/// backwards on every crossing — 99% of the way to 150, then 4% to 350.
/// `Default` is empty, which is what a `Panel` holds before its first refresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Toward {
    /// How far past the tier behind.
    pub done: u64,
    /// How far the tier ahead is from the one behind. Never nought.
    pub span: u64,
    /// What is in front of the tower on this track.
    pub ahead: Ahead,
}

/// What a track has left: three answers, written as two.
///
/// `at: Option<u64>` collapsed *nothing more is authored* and *nobody has asked
/// yet*, so a fresh tower's ley gauge drew as the most finished one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ahead {
    /// Not measured yet. Draws and says nothing about where the tower stands.
    #[default]
    Unasked,
    /// The total that reaches the next tier.
    Tier(u64),
    /// Past the last tier: nothing more is authored.
    Nothing,
}

impl Default for Toward {
    fn default() -> Self {
        Self {
            done: 0,
            span: 1,
            ahead: Ahead::Unasked,
        }
    }
}

impl Toward {
    /// A full gauge, for a track with nothing left to reach.
    const FULL: Self = Self {
        done: 1,
        span: 1,
        ahead: Ahead::Nothing,
    };

    /// Where `held` stands among `tiers`, which must ascend.
    ///
    /// One arithmetic for both gauges, so the Ley Line and renown cannot
    /// disagree about what *nearly there* looks like. Past the last tier it
    /// reads full rather than empty.
    #[must_use]
    pub fn among(held: u64, tiers: impl Iterator<Item = u64>) -> Self {
        let mut behind = 0;
        for at in tiers {
            if at > held {
                return Self {
                    done: held.saturating_sub(behind),
                    span: at.saturating_sub(behind).max(1),
                    ahead: Ahead::Tier(at),
                };
            }
            behind = at;
        }
        Self::FULL
    }
}

/// What the tower is called now, and what it is climbing toward.
///
/// `None` below the first rank: inventing a title would spend that rank's
/// arrival. Owned rather than borrowed, because every caller has already let the
/// world's borrow go.
#[must_use]
pub fn rank(world: &World) -> Option<String> {
    let held = world.resource::<Renown>().get();
    world
        .resource::<crate::content::Progression>()
        .renown()
        .iter()
        .take_while(|step| step.at <= held)
        .last()
        .map(|step| step.id.clone())
}

/// Where renown stands against the next rank.
#[must_use]
pub fn toward(world: &World) -> Toward {
    let held = world.resource::<Renown>().get();
    Toward::among(
        held,
        world
            .resource::<crate::content::Progression>()
            .renown()
            .iter()
            .map(|rank| rank.at),
    )
}

/// One sentence about the number moving, with the facts beside it.
///
/// `experience.rs`'s `say_gained`, field for field. `Quantity` is what moved;
/// the running total is prose, because a `FieldName` for it would have to pass
/// `save::naming`'s round-trip and be adopted by `experience.rs` too.
fn say(world: &mut World, key: &str, moved: u64, now: u64, role: Role) {
    let message = world.resource::<Prose>().line(
        key,
        &[
            ("quantity", &moved.to_string()),
            ("count", &now.to_string()),
        ],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, "renown")
        .count(FieldName::Quantity, moved)
        .text(FieldName::Message, &message)
        .role(role)
        .finish();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;
    use crate::tower::Work;

    fn total(sim: &Sim) -> u64 {
        sim.world().resource::<Renown>().get()
    }

    fn said(sim: &Sim) -> Vec<String> {
        sim.scrollback()
            .records()
            .iter()
            .filter_map(
                |record| match record.field(orbs_render::FieldName::Message) {
                    Some(orbs_render::Value::Text(text)) => Some(text.to_owned()),
                    _ => None,
                },
            )
            .collect()
    }

    #[test]
    fn a_made_thing_mints_and_a_read_book_does_not() {
        // The distinction the module header is about: `Work::made` put something
        // on a shelf, `Work::at` did not.
        let mut sim = Sim::new(1);
        super::super::done(
            sim.world_mut(),
            &Work::made("alembic", "clarity", true, false),
            8,
        );
        assert_eq!(total(&sim), worth(8), "a brewed potion minted nothing");

        let before = total(&sim);
        super::super::done(sim.world_mut(), &Work::at("stacks"), 4);
        assert_eq!(total(&sim), before, "walking the stacks minted renown");
    }

    #[test]
    fn the_first_instrument_in_the_game_is_worth_something() {
        // The rounding, as a property: `mortar_and_pestle` earns 1, and a
        // floored halving would have the first action in the game mint nought.
        assert_eq!(worth(1), 1);
        assert_eq!(worth(2), 1);
        assert_eq!(worth(8), 4);
    }

    #[test]
    fn it_falls_to_nought_and_no_further() {
        let mut sim = Sim::new(1);
        earn(sim.world_mut(), 10);
        lose(sim.world_mut(), 4);
        assert_eq!(total(&sim), 6);
        lose(sim.world_mut(), 1_000);
        assert_eq!(total(&sim), 0, "renown went below nought");
    }

    #[test]
    fn earning_is_silent_until_it_crosses_a_rank() {
        // A line per making took the clarity policy from 466 records in two
        // hours to 792, so the number accrues quietly — but a rank is an edge.
        let mut sim = Sim::new(1);
        let quiet = sim.scrollback().records().len();

        // Under the first rank, which the shipped file puts at 25.
        earn(sim.world_mut(), 10);
        assert_eq!(
            sim.scrollback().records().len(),
            quiet,
            "earning renown put a line on the transcript",
        );

        // ...and over it, which does.
        earn(sim.world_mut(), 40);
        assert_eq!(
            sim.scrollback().records().len(),
            quiet + 1,
            "crossing the first rank said nothing",
        );
        assert!(said(&sim).iter().any(|line| line.contains("hedge-wizard")));
    }

    #[test]
    fn a_loss_speaks_and_a_fall_through_a_rank_speaks_again() {
        let mut sim = Sim::new(1);
        earn(sim.world_mut(), 200);
        let held = sim.scrollback().records().len();

        // A loss that changes no rank says one thing: the number.
        lose(sim.world_mut(), 10);
        assert_eq!(sim.scrollback().records().len(), held + 1);

        // One that falls through a rank says the number *and* the title.
        lose(sim.world_mut(), 1_000);
        assert_eq!(sim.scrollback().records().len(), held + 3);
        assert!(
            said(&sim)
                .iter()
                .any(|line| line.contains("stopped calling you")),
            "falling to nothing did not say the title was gone: {:?}",
            said(&sim),
        );
    }

    #[test]
    fn a_slip_takes_the_renown_and_says_nothing() {
        // The half of `lose` a siege round needs: the number falls, the
        // transcript does not grow. The fight says the total once, on settling.
        let mut sim = Sim::new(1);
        earn(sim.world_mut(), 200);
        let held = sim.scrollback().records().len();

        slip(sim.world_mut(), 10);
        assert_eq!(total(&sim), 190);
        assert_eq!(
            sim.scrollback().records().len(),
            held,
            "a slip put a line on the transcript",
        );
    }

    #[test]
    fn a_slip_through_a_rank_still_says_the_title() {
        // Losing a name is an edge, and an edge is news however quiet the
        // number is. `crossed` is shared rather than reimplemented.
        let mut sim = Sim::new(1);
        earn(sim.world_mut(), 200);
        let held = sim.scrollback().records().len();

        slip(sim.world_mut(), 1_000);
        assert_eq!(total(&sim), 0, "a slip went below nought");
        assert_eq!(
            sim.scrollback().records().len(),
            held + 1,
            "a slip through a rank said the wrong number of things",
        );
        assert!(
            said(&sim)
                .iter()
                .any(|line| line.contains("stopped calling you")),
            "slipping out of the ranks did not say the title was gone: {:?}",
            said(&sim),
        );
    }

    #[test]
    fn a_move_of_nought_says_nothing() {
        // Nought is not an event in either direction, and a `lose` that hits the
        // floor without taking anything is nought.
        let mut sim = Sim::new(1);
        let before = sim.scrollback().records().len();
        earn(sim.world_mut(), 0);
        lose(sim.world_mut(), 0);
        lose(sim.world_mut(), 5);
        assert_eq!(
            sim.scrollback().records().len(),
            before,
            "nothing moved and something was said",
        );
    }
}
