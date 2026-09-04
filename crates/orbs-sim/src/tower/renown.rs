//! What the tower is known for (DESIGN.md §11.5, §19).
//!
//! # The tower's second number, and the first one that can fall
//!
//! [`Experience`](super::Experience) rises and never falls, and past the Ley
//! Line's last station it buys nothing at all — the weave draws a full bar and a
//! number that keeps climbing. Renown is what the work is worth *after* that: it
//! is minted by making things, moved both ways by a siege, and it is the one
//! number a player can lose.
//!
//! **It falls to nought and no further.** A signed total would put a sign in
//! every reader for a state the game has no use for, so [`lose`] saturates.
//!
//! # Minted on makings, not on completions
//!
//! [`done`](super::done) is the one door every completion goes through — and it
//! carries *events* as well as makings. A siege settling, a spell bound and a
//! secret found all arrive there, so minting on the door itself would pay renown
//! for reading a book, and would pay a siege twice over since escrow flows
//! through the same call and escrow pays on a **loss**. [`Work::sold`](super::Work::sold) is the
//! question this asks instead: did the run put a nameable thing on a shelf.
//!
//! # No stream, and nothing derived
//!
//! Renown draws nothing, so it replays from `(seed, submissions)` with no
//! `RngStream` of its own. Unlike concentration it is *stored* rather than
//! derived, because it is not a function of anything — its history is the only
//! thing that knows it.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::session::Scrollback;

/// What one made thing is worth in renown, against what it earned.
///
/// **A share of the run's experience, rounded up.** Tying it to `[earns]` means
/// one authored table prices both numbers and a tuning pass moves them together.
/// Rounded *up* rather than down because the mortar and pestle earns 1 — the
/// first instrument in the game and the one the apprenticeship teaches — and a
/// halving that floored would have it mint nothing at all, teaching a new player
/// that making things does not count.
#[must_use]
pub const fn worth(earned: u64) -> u64 {
    earned.div_ceil(RENOWN_PER)
}

/// How much experience one renown costs.
const RENOWN_PER: u64 = 2;

/// What the tower is known for.
///
/// **The one number that falls.** Everything else the tower holds is a faucet or
/// a pool with a floor; this is a reputation, and a reputation is the only thing
/// the design has that a bad night can take away.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Renown(u64);

impl Renown {
    /// Put a total back, for a save.
    ///
    /// **Not [`earn`]**, for `Experience::restore`'s reason: a restore is not
    /// the player earning anything, and a load that congratulated you on work you
    /// did yesterday would be reporting a lie in voice.
    pub(crate) const fn restore(&mut self, total: u64) {
        self.0 = total;
    }

    /// The total held.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Add what a making was worth. **Silently.**
///
/// Called from [`done`](super::done) where the work put something on a shelf.
///
/// **A gain is not an event, and the balance harness proved it.** The first
/// pass said a line per making; a single policy sweep took the clarity loop from
/// 466 records in two hours to 792, one *"word gets about"* per potion, for ever.
/// That is the shape `credit` already refuses next door — a level is announced at
/// the edge that buys it and never on the way there, because the fact is
/// continuous and only the crossing is news.
///
/// So the number accrues quietly and is read off the gauge at the top of every
/// pane. **The edge that speaks is the rank**, which `crossed` below is for.
pub fn earn(world: &mut World, gained: u64) {
    if gained == 0 {
        return;
    }
    let before = reached(world);
    world.resource_mut::<Renown>().0 = world.resource::<Renown>().0.saturating_add(gained);
    crossed(world, before);
}

/// Put the total somewhere, for a tester. **Both directions, and it says so.**
///
/// The only door that can *lower* renown without a lost siege, which is what a
/// rank being lost needs: every one of the ten sits behind an hour of play, and
/// falling back through one otherwise means losing on purpose.
#[cfg(debug_assertions)]
pub fn set(world: &mut World, total: u64) {
    let before = reached(world);
    world.resource_mut::<Renown>().0 = total;
    crossed(world, before);
}

/// How many ranks the tower has reached. Nought is a tower nobody has heard of.
///
/// **A count, not a name, and the direction depends on it.** Falling from
/// magister to cunning man leaves a rank in hand, so comparing *names* said the
/// tower had been promoted — a fall reads as an arrival unless something counts.
fn reached(world: &World) -> usize {
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
/// **An edge, not a state**, which is `credit`'s rule for concentration: the
/// name is continuous and derived, so it would otherwise be true in silence for
/// ever. Both directions are news — a title gained and a title lost are the two
/// things about renown a player most wants to hear.
///
/// **A fall names what you are called now, or that you are called nothing.** A
/// player who drops two ranks hears it once, about where they landed, rather
/// than once per rank passed: the tower has one name and the news is what it is.
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
    // Three sentences, not two. A fall that still leaves a title is not an
    // arrival and must not read as one — *"they are calling you cunning man
    // now"* congratulated a player who had just lost two ranks.
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

/// Take renown away, to nought and no further, and **say so**.
///
/// **The asymmetry with [`earn`] is deliberate.** A gain is expected, frequent
/// and already on screen; a loss is something that happened *to* the player, at
/// a moment they may not have been watching, and §6 forbids a state changing
/// under someone in silence. One sentence per loss is rare because losses are.
///
/// **Saturating rather than signed.** A tower can be disgraced to nothing and
/// that is the floor; owing renown is a state nothing reads and nothing could
/// draw.
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

/// Where a number stands between the tier behind it and the one ahead.
///
/// **Both bounds, not just the target.** A gauge that filled from nought to the
/// next threshold would jump backwards every time one was passed — 99% of the
/// way to 150, then 4% of the way to 350. Measuring from the tier *behind* is
/// what makes it fill smoothly and empty exactly once, when a tier is crossed.
/// **`Default` is an empty gauge with nothing ahead**, which is what a `Panel`
/// holds before its first refresh — not a full one, because a bar that started
/// full and emptied on the first tick would report the opposite of the truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Toward {
    /// How far past the tier behind.
    pub done: u64,
    /// How far the tier ahead is from the one behind. Never nought.
    pub span: u64,
    /// What is in front of the tower on this track.
    pub ahead: Ahead,
}

/// What a track has left, which is **three** answers and was written as two.
///
/// `at: Option<u64>` collapsed *nothing more is authored* and *nobody has asked
/// yet* into the same `None`, and every reader branched on `is_some`. So a
/// `Panel` before its first refresh — the state `Toward::default` exists for —
/// drew a fresh tower's ley gauge as an empty bar labelled `nothing more
/// authored`, and spoke the same to a screen reader: the emptiest possible tower
/// reported as the most finished one.
///
/// It was unreachable, because every frontend refreshes before it paints. It was
/// also unrepresentable-apart, which is the part worth fixing — the guard was
/// four call sites' worth of habit rather than anything the type enforced.
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
    /// **The one arithmetic, used by both gauges**, so the Ley Line and renown
    /// cannot come to disagree about what *nearly there* looks like. Past the
    /// last tier it reads full rather than empty: nothing more is authored, which
    /// §19 is careful to distinguish from *finished*, but a bar that emptied at
    /// the top of the game would say the opposite of what happened.
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
/// `None` for the name below the first rank: a tower nobody has heard of has no
/// title, and inventing one would spend the first rank's arrival.
/// **Owned rather than borrowed**, because every caller is a painter that has
/// already let the world's borrow go — and the alternative a first pass reached
/// for, leaking the id to get a `'static`, would leak once a frame for ever.
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
/// **The fields carry the facts and the message carries the sentence**, on one
/// record — `experience.rs`'s `say_gained` is the shape, field for field.
///
/// `Quantity` is what **moved**. Where the total now *stands* is interpolated
/// into the message and is not a field, because there is no `FieldName` for a
/// running total and this is not the place to invent one: the variant would have
/// to earn its way through `save::naming`'s round-trip and be adopted by
/// `experience.rs` too, or the two siblings would describe the same kind of fact
/// differently. So `sift --field qty` can ask what was gained or lost, and the
/// total is prose. Rule 4 would prefer both to be fields; that is a change to
/// make for both numbers at once, not silently for one.
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
        // **The rounding, as a property.** `mortar_and_pestle` earns 1, and a
        // halving that floored would mint nought — so the apprenticeship's first
        // action would teach that making things does not count.
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
        // **The property the balance harness bought.** A line per making took the
        // clarity policy from 466 records in two hours to 792, so the number
        // accrues quietly — but a *rank* is an edge, and an edge is news.
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
