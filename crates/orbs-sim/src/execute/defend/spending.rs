//! Spending the arsenal onto the coming round (§5.1).
//!
//! **This is where §7's *"one room reachable from every other"* finally pays.**
//! A potion brewed in the laboratory, a scroll assembled in the archive and a
//! troop sung up in the menagerie all arrive here through the arsenal, and what
//! each is worth is authored in `siege.toml` rather than written in Rust.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use super::shared::{fixture, say};
use crate::parser::{Intent, Verb};
use crate::tower::{self, siege, siege::Siege};

/// Spend an arsenal item onto the coming round.
///
/// **One body for `deploy` and `quaff`**, because they differ only in what they
/// are allowed to spend and what it does. Two copies of *find the siege, find
/// the thing, check it is the right kind, apply it, say so* is two chances to
/// disagree about a refusal.
pub(super) fn spend(intent: &Intent, world: &mut World, verb: Verb) {
    let Some(rampart) = fixture(world) else {
        say(world, verb, "defend_nowhere", &[], Role::Cost);
        return;
    };
    let Some(siege) = world.get::<Siege>(rampart) else {
        say(world, verb, "hold_unopened", &[], Role::Cost);
        return;
    };
    if !siege.running() {
        say(world, verb, "hold_over", &[], Role::Cost);
        return;
    }

    let Some(argument) = intent.arguments.first() else {
        say(world, verb, "spend_incomplete", &[], Role::Cost);
        return;
    };
    let named = crate::parser::leaf(&argument.value).to_owned();

    // **Authored, not inferred** (rule 6): `siege.toml` says what each item does
    // and which word spends it, so a tuning pass is a content edit. An item with
    // no entry cannot be spent at all, which is what stops `quaff sage`
    // resolving and quietly doing nothing.
    let spendables = world.resource::<crate::content::Spendables>();
    let Some(entry) = spendables.get(&named).cloned() else {
        say(
            world,
            verb,
            "spend_useless",
            &[("name", &named)],
            Role::Cost,
        );
        return;
    };
    // **The wrong word is its own refusal**, and it names the right one. A
    // player who typed `quaff troop` is one word from correct, and §6 forbids a
    // bare error.
    let wanted = world
        .resource::<crate::content::Spendables>()
        .verb_for(&named);
    if wanted != Some(verb) {
        let wanted = wanted.map_or("nothing", Verb::canonical).to_owned();
        say(
            world,
            verb,
            "spend_wrong_word",
            &[("name", &named), ("detail", &wanted)],
            Role::Cost,
        );
        return;
    }

    // **A potion that would do nothing is refused rather than drunk.** Spending
    // a scarce heal on a line that is already whole is not an error — §6's bare
    // error is a different thing — but it is a silent loss of the one resource
    // the domain exists to make you weigh, and the next siege arrives on
    // `CADENCE` whether or not you have anything left.
    //
    // So the orb says so and keeps the potion. Found by a matrix test asking
    // whether every authored row *changes* the siege: `mending` did not, because
    // the fixture drank it at full strength.
    // **Against `mustered`, never the current count** — and this guard had it
    // wrong, which is `Band::mend`'s own defect reintroduced at the call site
    // one commit after it was fixed in the model.
    //
    // `wound` leaves `count = ceil(vigour / VIGOUR)`, so `count * VIGOUR >=
    // vigour` always, *with equality whenever vigour is a multiple of three*.
    // The guard therefore fired on a third of all wounded states — including
    // vigour 9 of 18, which is exactly the `hurt` threshold every shipped solver
    // hangs its `quaff` rung on. The potion was refused precisely when it was
    // most needed, and the refusal claimed the line was whole while nine points
    // were missing.
    if entry.kind == "vigour"
        && let Some(siege) = world.get::<Siege>(rampart)
        && siege.garrison.vigour >= siege.mustered * siege::VIGOUR
    {
        say(world, verb, "spend_whole", &[("name", &named)], Role::Cost);
        return;
    }

    // **A roll modifier bought for a round the garrison does not roll in is
    // refused, for the reason a heal at full strength is.** `staged` clears when
    // the round resolves, so a bonus bought before a volley is spent on rolls
    // that never happen — the same silent loss, one intent over.
    //
    // Bodies and fight are *not* refused here, and that is the distinction worth
    // keeping: a volley still hits you. What it does not do is let you hit back.
    if entry.effect().is_some()
        && let Some(siege) = world.get::<Siege>(rampart)
        && !siege.intent.answered()
    {
        say(
            world,
            verb,
            "spend_unanswered",
            &[("name", &named), ("state", siege.intent.word())],
            Role::Cost,
        );
        return;
    }

    // **From the arsenal, which is the one room reachable from every other.**
    // §19 built that exemption for exactly this: a potion brewed in the
    // laboratory has to be spendable in the bailey, and before the arsenal
    // existed it could not leave the room it was made in.
    if !take_one(world, &named) {
        say(world, verb, "spend_none", &[("name", &named)], Role::Cost);
        return;
    }

    // The Ley Line's `garrison`: every troop deployed brings more bodies than
    // the menagerie sang (§11.5). Read before the borrow below.
    let garrison = crate::tower::grant::garrison_bonus(world);
    let mut siege = world
        .get_mut::<Siege>(rampart)
        .expect("the siege was there a moment ago");
    match entry.kind.as_str() {
        "troops" => siege.reinforce(entry.count.saturating_add(garrison)),
        "vigour" => siege.heal(entry.points),
        _ => {
            if let Some(effect) = entry.effect() {
                siege.stage(&named, effect);
            }
        }
    }
    // The borrow ends here; `publish` needs the world back.
    let _ = siege;
    super::publish::publish(world, rampart);

    say(
        world,
        verb,
        match verb {
            Verb::Deploy => "deploy_sent",
            Verb::Wield => "scroll_spent",
            _ => "quaff_drunk",
        },
        &[("name", &named)],
        Role::Success,
    );
}

/// Take one of `named` out of the arsenal, if there is one.
///
/// **Spending is a real withdrawal**, which is what makes the arsenal a
/// decision: a potion drunk here is a potion the next siege does not have, and
/// that is §11.5's *"repairs consume resources"* on the one surface where the
/// player chose to spend them.
fn take_one(world: &mut World, named: &str) -> bool {
    let Some(arsenal) = tower::keep(world) else {
        return false;
    };
    tower::take(world, arsenal, named, 1)
}
