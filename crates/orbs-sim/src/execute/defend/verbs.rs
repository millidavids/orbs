//! The five words a player types at a wall (§5.1).
//!
//! `hold` is the only one that advances the world; the rest are free and
//! instant (§5.0), which is what makes the siege turn-based rather than slow.
//!
//! What each word costs is `spending`; what a resolved round says is `report`.

use std::cmp::Ordering;

use bevy_ecs::prelude::*;
use orbs_render::Role;

use super::publish::publish;
use super::report::{announce, log_rolls, settle};
use super::shared::{fixture, say};
use super::spending::spend;
use crate::parser::{Intent, Verb};
use crate::tower::{
    self,
    dice::Die,
    siege::{self, Siege},
};

/// `defend` — stand to the wall and let the enemy arrive.
/// `petition` — spend standing so that fewer come up the road next time.
///
/// The first thing renown buys. It lowers the ceiling rather than subtracting
/// after the draw, which would let a player pay four times and draw the floor
/// anyway. Self-limiting: paying drops your rank, and a lower rank draws a
/// shorter tail.
pub(in crate::execute) fn petition(world: &mut World) {
    let Some(rampart) = fixture(world) else {
        say(world, Verb::Petition, "defend_nowhere", &[], Role::Cost);
        return;
    };
    // Gated with the rest of the bailey: there is nothing to petition against
    // until the sanctum's first station has armed the road.
    if !world.resource::<tower::Opened>().has(tower::opened::SIEGE) {
        say(world, Verb::Petition, "defend_unarmed", &[], Role::Cost);
        return;
    }
    // Refused mid-fight (§19): it buys the *next* siege, and `settle` charged
    // the spend to the fight — a siege that moved +38 reported +31.
    if world.get::<Siege>(rampart).is_some_and(Siege::running) {
        say(world, Verb::Petition, "petition_besieged", &[], Role::Cost);
        return;
    }

    let bought = world.resource::<siege::Petitioned>().get();
    let ranks = tower::renown::reached(world);

    // Refused at the floor, having spent nothing: the tail is already as short
    // as it goes.
    if siege::most_at(ranks, bought) <= siege::FEWEST {
        say(
            world,
            Verb::Petition,
            "petition_least",
            &[("quantity", &siege::FEWEST.to_string())],
            Role::Cost,
        );
        return;
    }

    // The price is quoted from the same expression that charges it — the forge's
    // rule: a room may not quote one number and take another.
    let held = world.resource::<tower::Renown>().get();
    if !tower::renown::spend(world, siege::PETITION_PER_FOE) {
        say(
            world,
            Verb::Petition,
            "petition_short",
            &[
                ("kind", &siege::PETITION_PER_FOE.to_string()),
                ("quantity", &held.to_string()),
            ],
            Role::Cost,
        );
        return;
    }

    world.resource_mut::<siege::Petitioned>().add();
    let now = world.resource::<siege::Petitioned>().get();
    // Ranks read *again*, after the payment: spending can cross a rank (§19),
    // and the pre-payment count quoted a ceiling one too high.
    let most = siege::most_at(tower::renown::reached(world), now);
    say(
        world,
        Verb::Petition,
        "petition_done",
        &[
            ("quantity", &most.to_string()),
            ("kind", &siege::PETITION_PER_FOE.to_string()),
        ],
        Role::Success,
    );
}

pub(in crate::execute) fn defend(world: &mut World) {
    let Some(rampart) = fixture(world) else {
        say(world, Verb::Defend, "defend_nowhere", &[], Role::Cost);
        return;
    };
    if world.get::<Siege>(rampart).is_some_and(Siege::running) {
        say(world, Verb::Defend, "defend_already", &[], Role::Cost);
        return;
    }
    // A siege against a tower with no wall is not a decision (§11.5). The
    // sanctum's first station arms it; until then the road stays empty.
    if !world.resource::<tower::Opened>().has(tower::opened::SIEGE) {
        say(world, Verb::Defend, "defend_unarmed", &[], Role::Cost);
        return;
    }

    // The cadence (§11.5, `siege::CADENCE`). Unlimited, `orbs-balance` measured
    // a back-to-back driver at 4.70 experience a tick against clarity's 0.140.
    // §5.3's trace will eventually provoke a siege; this stands in for it.
    let now = world.resource::<crate::tick::Tick>().get();
    if let Some(clear_at) = world.get::<Siege>(rampart).and_then(|siege| siege.clear_at)
        && now < clear_at
    {
        let left = clear_at - now;
        say(
            world,
            Verb::Defend,
            "defend_too_soon",
            &[("quantity", &left.to_string())],
            Role::Cost,
        );
        return;
    }

    // A siege grants nothing: the player brings the tower's pool (§19).
    //
    // The draw is here rather than in a system, for `muster`'s reason:
    // `RngStream::Siege` advances only when the player asks, so a future siege
    // system can join the schedule without shifting a replay. Size is decided
    // before it by what the tower is worth, and `petition`'s allowance is taken
    // here so a refused `defend` cannot eat what was paid for.
    let ranks = tower::renown::reached(world);
    let bought = world.resource_mut::<siege::Petitioned>().take();
    let most = siege::most_at(ranks, bought);
    let mut siege = {
        let mut rngs = world.resource_mut::<crate::rng::Rngs>();
        Siege::begin_against(&mut rngs, most)
    };
    // The Ley Line's `edge`, read once for the whole fight (§11.5).
    siege.edge = tower::grant::edge_bonus(world);
    // ...and what the tower was worth before a blow was struck, so the settling
    // sentence says what the *fight* came to, not what its last moment did.
    siege.standing = Some(world.resource::<tower::Renown>().get());
    let arrived = siege.enemy.count;
    let intent = siege.intent.word();
    world.entity_mut(rampart).insert(siege);
    publish(world, rampart);

    say(
        world,
        Verb::Defend,
        "defend_opens",
        &[("quantity", &arrived.to_string()), ("state", intent)],
        Role::Success,
    );
}

/// `deploy <name>` — send a summoned troop into the line.
pub(in crate::execute) fn deploy(intent: &Intent, world: &mut World) {
    spend(intent, world, Verb::Deploy);
}

/// `quaff <name>` — spend a potion on the coming round.
pub(in crate::execute) fn quaff(intent: &Intent, world: &mut World) {
    spend(intent, world, Verb::Quaff);
}

/// `wield <scroll>` on the wall — spend it on the siege rather than the tower.
///
/// Returns whether it took the command; `pipeline::wield` asks first. §19 keeps
/// `wield` for scrolls rather than a fourth verb, and this is what that costs.
pub(in crate::execute) fn wielded(intent: &Intent, world: &mut World) -> bool {
    let Some(rampart) = fixture(world) else {
        return false;
    };
    if !world.get::<Siege>(rampart).is_some_and(Siege::running) {
        return false;
    }
    let Some(argument) = intent.arguments.first() else {
        return false;
    };
    let named = crate::parser::leaf(&argument.value).to_owned();
    // Only what the wall can use. Anything else falls through, so `wield
    // mortar_and_pestle` still starts an instrument mid-siege.
    if world
        .resource::<crate::content::Spendables>()
        .verb_for(&named)
        != Some(Verb::Wield)
    {
        return false;
    }
    spend(intent, world, Verb::Wield);
    true
}

/// `pledge <die> to <area>` — put one of your dice behind part of the wall.
///
/// Free and instant, like everything else on your turn. The die is rolled when
/// the round resolves, so the board can print the range before the commitment.
pub(in crate::execute) fn pledge(intent: &Intent, world: &mut World) {
    let Some(rampart) = fixture(world) else {
        say(world, Verb::Pledge, "defend_nowhere", &[], Role::Cost);
        return;
    };
    let Some(siege) = world.get::<Siege>(rampart) else {
        say(world, Verb::Pledge, "hold_unopened", &[], Role::Cost);
        return;
    };
    if !siege.running() {
        say(world, Verb::Pledge, "hold_over", &[], Role::Cost);
        return;
    }

    let mut named = intent.arguments.iter().map(|argument| &argument.value);
    let (Some(first), Some(second)) = (named.next(), named.next()) else {
        say(world, Verb::Pledge, "pledge_incomplete", &[], Role::Cost);
        return;
    };
    // The leaf, for every sentence below: a resolved place arrives as a full
    // path, so echoing it reads *"d20 goes to /tower/bailey/buckler"*.
    let die_word = crate::parser::leaf(first).to_owned();
    let area_word = crate::parser::leaf(second).to_owned();

    // Both slots are `NounKind::Place`, so `pledge buckler d20` parses and has
    // to be refused here — naming which half was wrong, since a player who
    // swapped them is one word from right.
    let (Some(die), Some(area)) = (Die::named(&die_word), siege::Area::named(&area_word)) else {
        let key = if Die::named(&die_word).is_none() {
            "pledge_not_a_die"
        } else {
            "pledge_not_an_area"
        };
        say(
            world,
            Verb::Pledge,
            key,
            &[("name", &die_word), ("detail", &area_word)],
            Role::Cost,
        );
        return;
    };

    // A die already pledged is refused, and named where it went. Ignoring it
    // would leave the board looking right and the round resolving weaker.
    let pool = world.resource::<tower::Quintessence>().get();
    let Some(mut siege) = world.get_mut::<Siege>(rampart) else {
        return;
    };
    let cost = match siege.pledge(die, area, pool) {
        siege::Pledged::Made { cost } => cost,
        siege::Pledged::Spent => {
            let already = siege
                .pledges
                .iter()
                .find(|pledge| pledge.die == die)
                .map_or("nowhere", |pledge| pledge.area.word())
                .to_owned();
            say(
                world,
                Verb::Pledge,
                "pledge_spent",
                &[("name", &die_word), ("state", &already)],
                Role::Cost,
            );
            return;
        }
        // Names the cost and what is left: *"you cannot afford it"* without the
        // two numbers cannot be planned around, and that is the mechanic.
        siege::Pledged::Short { cost } => {
            let left = pool.to_string();
            say(
                world,
                Verb::Pledge,
                "pledge_short",
                &[
                    ("name", &die_word),
                    ("kind", &cost.to_string()),
                    ("detail", &left),
                ],
                Role::Cost,
            );
            return;
        }
    };
    let (low, high) = siege.range(area);
    let wasted = !area.answers(siege.intent);
    let coming = siege.intent.word().to_owned();
    let _ = siege;
    // The tower pays only once the pledge is made — `Siege::pledge` takes
    // nothing, so the two refusals above touch no resource at all.
    world.resource_mut::<tower::Quintessence>().spend(cost);
    publish(world, rampart);

    // A pledge the intent will waste is allowed and warned about: refusing it
    // would be the game playing for you, and silence would break §5.1.
    let state = if wasted { coming } else { high.to_string() };
    say(
        world,
        Verb::Pledge,
        if wasted {
            "pledge_wasted"
        } else {
            "pledge_made"
        },
        &[
            ("name", &die_word),
            ("detail", area.word()),
            ("quantity", &low.to_string()),
            ("state", &state),
            // What it took. The board carries what is *left*; the sentence
            // carries the cost, which is what a player is weighing.
            ("kind", &cost.to_string()),
        ],
        if wasted { Role::Cost } else { Role::Success },
    );
}

/// `hold` — end your turn and let one round resolve.
///
/// The only thing in this domain that moves the world.
pub(in crate::execute) fn hold(world: &mut World) {
    let Some(rampart) = fixture(world) else {
        say(world, Verb::Hold, "defend_nowhere", &[], Role::Cost);
        return;
    };
    let Some(siege) = world.get::<Siege>(rampart) else {
        say(world, Verb::Hold, "hold_unopened", &[], Role::Cost);
        return;
    };
    if !siege.running() {
        say(world, Verb::Hold, "hold_over", &[], Role::Cost);
        return;
    }

    // Taken out, resolved, put back, because resolving needs `&mut Siege` and
    // `&mut Rngs` at once. Cloning keeps `Siege::resolve` pure, so
    // `tower::siege`'s tests need no `World`.
    //
    // A `whetted` rampart rolls a bigger die, staged every round while the charm
    // holds rather than once when it is laid: `resolve` clears `staged`, so a
    // modifier cannot outlive the window.
    if tower::charmed(world, rampart, tower::charm::Kind::Whetted)
        && let Some(mut siege) = world.get_mut::<Siege>(rampart)
    {
        siege.stage(
            tower::charm::Kind::Whetted.word(),
            tower::Effect::Upgrade(tower::Die::D12),
        );
    }

    let mut siege = world
        .get::<Siege>(rampart)
        .cloned()
        .expect("the siege was there a moment ago");
    let round = {
        let mut rngs = world.resource_mut::<crate::rng::Rngs>();
        siege.resolve(&mut rngs)
    };
    *world
        .get_mut::<Siege>(rampart)
        .expect("the siege is still there") = siege;

    // A resolved round pays and waiting inside one does not: the calm trickle is
    // suspended while a siege runs. A lump per round rather than a rate, for
    // §14 — the patient mode advances siege ticks on player input, so a rate
    // would pay for typing.
    let ceiling = tower::ceiling(world);
    world
        .resource_mut::<tower::Quintessence>()
        .restore(tower::REGEN_PER_ROUND, ceiling);

    // Every roll into the log, with its die and its face. Rule 4: one record is
    // the transcript line, the §14 utterance and what `sift` finds.
    log_rolls(world, &round);
    publish(world, rampart);
    announce(world, &round);

    // What the exchange was worth in standing (§11.5, §19). After `announce`, so
    // the round says what happened before what it cost.
    //
    // All five numbers: `mended` is vigour put back, and netting it out bills a
    // player twice for damage they repaired. The sortie's trade survives netting
    // deliberately — `sortied` is `sortie / 2` against `spent`'s `sortie / 3`.
    let up = u64::from(round.dealt) + u64::from(round.sortied);
    let down = u64::from(
        round
            .taken
            .saturating_add(round.spent)
            .saturating_sub(round.mended),
    );
    match up.cmp(&down) {
        // Quietly, both ways: the round has just narrated itself, so
        // `renown::lose`'s sentence would retell it. `settle` says it once.
        Ordering::Greater => tower::renown::earn(world, up - down),
        Ordering::Less => tower::renown::slip(world, down - up),
        Ordering::Equal => {}
    }

    // The enemy attacks the automation (§5.1). It runs on a resolved round and
    // nowhere else, so the calm layer stays safe — pillar 4. Before `settle`,
    // so the last round can still sabotage rather than being the safe one.
    if let Some(reached) = tower::assault::strike(world) {
        say(
            world,
            Verb::Hold,
            match reached {
                tower::Reached::Script => "assault_script",
                tower::Reached::Clock => "assault_clock",
            },
            &[],
            Role::Danger,
        );
    }

    if let Some(outcome) = round.outcome {
        settle(world, rampart, outcome);
    }
}
