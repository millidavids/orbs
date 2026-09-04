//! The five words a player types at a wall (§5.1).
//!
//! **`hold` is the only one that advances the world.** Everything else here is
//! free and instant, which is §5.0's *"no per-command tick cost"* preserved
//! exactly — and it is what makes the siege turn-based rather than merely slow.
//!
//! What each word *costs* is `spending`; what a resolved round *says* is
//! `report`. The `defend` module doc has the rest.

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
pub(in crate::execute) fn defend(world: &mut World) {
    let Some(rampart) = fixture(world) else {
        say(world, Verb::Defend, "defend_nowhere", &[], Role::Cost);
        return;
    };
    if world.get::<Siege>(rampart).is_some_and(Siege::running) {
        say(world, Verb::Defend, "defend_already", &[], Role::Cost);
        return;
    }
    // **A siege against a tower with no wall is not a decision** (§11.5). The
    // sanctum's first station arms it; until then the road stays empty.
    if !world.resource::<tower::Opened>().has(tower::opened::SIEGE) {
        say(world, Verb::Defend, "defend_unarmed", &[], Role::Cost);
        return;
    }

    // **The cadence** (§11.5, `siege::CADENCE`). Without it `defend` is free and
    // unlimited, and `orbs-balance` measured a back-to-back driver at 4.70
    // experience a tick against clarity's 0.140 — thirty-three times the
    // flagship, which says *ignore every other room*. §5.3's trace is what will
    // eventually provoke a siege; until then this stands in for it.
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

    // **A siege grants nothing.** The pool is the tower's, shared with the
    // forge, and the player brings whatever they have — which is what makes
    // enchanting during a siege cost something real, and what stops a fight
    // being a fresh allowance that expires unspent (§19).
    //
    // **The domain's opening draw, and it is here rather than in a system**, for
    // `muster`'s reason: `RngStream::Siege` advances when the player asks for a
    // siege and never on a tick nobody asked for, which is what lets any future
    // siege system be appended to the schedule without shifting a replay.
    let mut siege = {
        let mut rngs = world.resource_mut::<crate::rng::Rngs>();
        Siege::begin(&mut rngs)
    };
    // The Ley Line's `edge`, read once for the whole fight (§11.5).
    siege.edge = tower::grant::edge_bonus(world);
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
/// Returns whether it took the command. **`pipeline::wield` asks this before
/// anything else**, so a scroll reaches the siege when there is one and its
/// ordinary effect when there is not.
///
/// §19 keeps `wield` for scrolls rather than giving them a fourth verb: a player
/// who has spent one in the archive spends one here without learning anything
/// new. What that costs is this interception, because the same word has to mean
/// *set this going* in two places.
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
    // **Only what the wall can actually use.** Anything else falls through, so
    // `wield mortar_and_pestle` still starts an instrument even mid-siege — the
    // bailey has none, but the rule should not depend on that.
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
/// **Free and instant, like everything else on your turn.** The die is not
/// *rolled* here; it is rolled when the round resolves, which is what lets the
/// board print the range before the commitment.
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
    // **The leaf, for every sentence below.** A resolved place arrives as its
    // full path, and prose that echoed it would read *"d20 goes to
    // /tower/bailey/buckler"* — the tower's internals in a line meant for a
    // player. `muster::haul` records the same trap.
    let die_word = crate::parser::leaf(first).to_owned();
    let area_word = crate::parser::leaf(second).to_owned();

    // **Matched rather than re-asked.** Both slots are `NounKind::Place`, so
    // `pledge buckler d20` parses and has to be refused here — and naming which
    // half was wrong beats a generic refusal, because a player who swapped them
    // is one word from right.
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

    // **A die already pledged is refused, and it names where it went.** A
    // silently ignored second pledge would be the worst shape here: the board
    // would look right and the round would resolve weaker than the player read.
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
        // **The decision the domain is about, said out loud.** It names what the
        // die would have cost and what is left, because *"you cannot afford it"*
        // without the two numbers is a refusal a player cannot plan around — and
        // planning around it is the mechanic.
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
    // **The tower pays, and only once the pledge is made.** `Siege::pledge` says
    // whether the pool is enough and takes nothing; the spend is here, so the
    // two refusals above return having touched no resource at all.
    world.resource_mut::<tower::Quintessence>().spend(cost);
    publish(world, rampart);

    // **A pledge the intent will waste is still allowed, and still warned
    // about.** Refusing it would be the game playing for you; saying nothing
    // would be §5.1's fairness rule broken — you are told the odds *before* the
    // commitment, and "this does nothing next round" is the starkest odds there
    // are.
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
            // carries what this decision cost, which is the half a player is
            // weighing at the moment they type it.
            ("kind", &cost.to_string()),
        ],
        if wasted { Role::Cost } else { Role::Success },
    );
}

/// `hold` — end your turn and let one round resolve.
///
/// **The only thing in this domain that moves the world.**
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

    // **Taken out, resolved, put back**, because resolving needs `&mut Siege`
    // and `&mut Rngs` at once and both live in the `World`. A clone of two
    // `Band`s and a short `Vec` is cheaper than threading the stream through the
    // model, and it keeps `Siege::resolve` a pure function of what it is handed —
    // which is what lets `tower::siege`'s tests prove the arithmetic with no
    // `World` at all.
    // **A `whetted` rampart rolls a bigger die, staged like a potion.**
    // `Effect::Upgrade` has shipped unused since the dice were built, with
    // `siege.toml` reserving it in as many words for *"an enchantment"* — so
    // this needs no new machinery at all, only the charm asked for at the same
    // moment a quaffed potion would be.
    //
    // **Staged every round while the charm holds**, not once when it is laid: a
    // charm is a window, and a modifier that survived past it would be a buff
    // that never expired. `resolve` clears `staged`, which is what makes that
    // work without a second rule.
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

    // **A resolved round pays, and waiting inside one does not.** The calm
    // trickle is suspended while a siege runs, so this lump is the only
    // quintessence a fight produces — which means dawdling earns nothing and the
    // one way to more is to advance the siege and take what the enemy does.
    //
    // **A lump per round rather than a rate**, and that is §14 rather than
    // balance: the patient mode advances siege ticks on *player input*, so
    // anything measured in ticks would mean more typing produces more resource
    // for exactly the players that mode exists to serve. This reads no clock, so
    // a patient siege and a played one grant identically.
    let ceiling = tower::ceiling(world);
    world
        .resource_mut::<tower::Quintessence>()
        .restore(tower::REGEN_PER_ROUND, ceiling);

    // **Every roll into the log, with its die and its face.** Rule 4: the same
    // record is the transcript line, the §14 utterance and what `sift` finds, so
    // `peruse bailey.log` is a genuine postmortem — a player can see which
    // potion earned its place, which is what makes the arsenal legible.
    log_rolls(world, &round);
    publish(world, rampart);
    announce(world, &round);

    // **The enemy attacks the automation** (§5.1), and this is the line that
    // makes the premise true. It runs on a resolved round and nowhere else, so
    // the calm layer stays genuinely safe — pillar 4.
    //
    // **Before `settle`**, so the last round of a siege can still sabotage: an
    // enemy that stopped caring the moment it was losing would make the closing
    // rounds the safe ones, which is backwards.
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
