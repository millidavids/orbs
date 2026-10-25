//! `summon` and `limn` — the menagerie's two words (DESIGN.md §10).
//!
//! The lens's shape: `summon` as `probe`, `limn` as `dial`. Neither takes the
//! production slot (`spell::block::begins_work`, §19). A hold refills the
//! garrison and then the orb earns — `muster`'s order; a balk costs only time.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use super::publish::{publish, republish_glyph};
use crate::content::Prose;
use crate::parser::{Intent, Verb};
use crate::tower::{
    self,
    circle::{Beast, Call, Glyph, Humour},
};

/// The circle's name, which is where a menagerie record is filed and what
/// `progression.toml` prices a hold under.
pub(crate) const CIRCLE: &str = "circle";

/// What a hold is counted under in the tally and the stores. No longer a thing
/// on a shelf: the bodies join the garrison (§19, *Supply*).
pub const TROOP: &str = "troop";

/// `summon [<demon>]` — draw a beast up at the circle, or call the waiting one
/// in.
///
/// Two acts in one word; the draw is not a call. Bare, a hold refills the
/// garrison; named, it binds that demon (§19, *Supply*). Never spent on a
/// guess: a `divined` line falls back to `summon` when the reader is beaten.
pub(crate) fn summon(intent: &Intent, world: &mut World, divined: bool) {
    let Some(circle) = fixture(world) else {
        say(world, Verb::Summon, "summon_nowhere", &[], Role::Cost);
        return;
    };
    let Ok(demon) = super::binding::named(intent, world) else {
        return;
    };
    if world.get::<Beast>(circle).is_none() {
        arrive(world, circle);
        return;
    }
    if divined {
        say(world, Verb::Summon, "summon_divined", &[], Role::Cost);
        return;
    }
    // A lesser circle binds bodies only. Refused before the call, so it costs
    // no par.
    let lesser = world
        .get::<Beast>(circle)
        .is_some_and(|beast| matches!(beast.shape(), tower::circle::Shape::Lesser(_)));
    if demon.is_some() && lesser {
        say(world, Verb::Summon, "summon_lesser_binds", &[], Role::Cost);
        return;
    }
    call(world, circle, demon.as_deref());
}

/// A beast arrives at the circle.
fn arrive(world: &mut World, circle: Entity) {
    // The domain's one draw, so `RngStream::Menagerie` advances only when
    // asked. A sealed tower draws lesser beasts until `menagerie_2`.
    let whole = world.resource::<tower::Opened>().has(tower::opened::CIRCLE);
    let beast = {
        let mut rngs = world.resource_mut::<crate::rng::Rngs>();
        Beast::draw(&mut rngs, whole)
    };
    let lit = world
        .resource::<Prose>()
        .counted("summon_rows", beast.shape().temper().fervour());
    let key = if whole {
        "summon_opens"
    } else {
        "summon_opens_lesser"
    };
    world.entity_mut(circle).insert(beast);
    publish(world, circle);
    say(
        world,
        Verb::Summon,
        key,
        &[("quantity", &lit)],
        Role::Success,
    );
}

/// Call the waiting beast into the circle as it stands.
fn call(world: &mut World, circle: Entity, demon: Option<&str>) {
    let Some(mut beast) = world.get_mut::<Beast>(circle) else {
        return;
    };
    let answered = beast.call();
    let calls = beast.calls();
    let troops = beast.troops();
    match answered {
        // A balk is a cost, never a fault. Nothing a spell reads moved, so
        // nothing is republished (§19).
        Call::Balked { rows } => {
            let balking = world.resource::<Prose>().counted("summon_rows", rows);
            say(
                world,
                Verb::Summon,
                "summon_balks",
                &[("quantity", &balking), ("count", &calls.to_string())],
                Role::Cost,
            );
        }
        Call::Held => hold(world, circle, calls, troops, demon),
    }
}

/// Every row agreed: the beast is held, and the circle is clear for the next.
fn hold(world: &mut World, circle: Entity, calls: u32, troops: u32, demon: Option<&str>) {
    let Some((shape, within_par)) = world
        .get::<Beast>(circle)
        .map(|beast| (beast.shape(), beast.within_par()))
    else {
        return;
    };
    world.entity_mut(circle).remove::<Beast>();
    publish(world, circle);

    // A lesser hold is a quarter of the price — the lesson, not the faucet. One
    // rule for par either way; the circle's shape says what par is.
    let earned = shape.priced(tower::worth_within_par(world, CIRCLE, calls, shape.par()));

    // Bound as a demon: into the arsenal, and one making of it. Nothing
    // shelved is no making: renown and the stores count what reached a shelf.
    if let Some(demon) = demon {
        let (answering, shelved) = super::binding::bind(world, demon, within_par);
        say(
            world,
            Verb::Summon,
            "summon_holds",
            &[("quantity", &answering), ("count", &calls.to_string())],
            Role::Success,
        );
        let work = tower::Work::event(tower::FIGURE);
        let work = if shelved { work.making(demon) } else { work };
        tower::done(world, &work, earned);
        return;
    }

    // Still a making, whatever the garrison could take: the hold is a run, and
    // renown and the tally count runs (§19, *Supply*).
    let work = tower::Work::event(tower::FIGURE).making(TROOP);
    let answering = super::enlisting::enlist(world, troops);
    say(
        world,
        Verb::Summon,
        "summon_holds",
        &[("quantity", &answering), ("count", &calls.to_string())],
        Role::Success,
    );
    tower::done(world, &work, earned);
}

/// `limn <glyph> [<humour>]` — limn a glyph; bare, step it round the six.
///
/// Free and instant, as `dial`: holding the beast is the work. Bare makes a
/// search writable — three `repeat 6` loops are an odometer.
pub(crate) fn limn(intent: &Intent, world: &mut World) {
    let Some(circle) = fixture(world) else {
        say(world, Verb::Limn, "summon_nowhere", &[], Role::Cost);
        return;
    };
    if world.get::<Beast>(circle).is_none() {
        say(world, Verb::Limn, "limn_unopened", &[], Role::Cost);
        return;
    }

    // The leaf, for every sentence below: a resolved place arrives as its full
    // path, and prose echoing it puts the tower's internals in a player's line.
    let mut named = intent
        .arguments
        .iter()
        .map(|argument| crate::parser::leaf(&argument.value).to_owned());
    let Some(first) = named.next() else {
        say(world, Verb::Limn, "limn_incomplete", &[], Role::Cost);
        return;
    };
    // Either order, since no glyph shares a word with a humour, so a refusal
    // names the wrong word.
    let (word, humour) = match named.next() {
        Some(second)
            if Glyph::from_word(&first).is_none() && Humour::from_word(&first).is_some() =>
        {
            (second, Some(first))
        }
        second => (first, second),
    };
    let Some(glyph) = Glyph::from_word(&word) else {
        say(
            world,
            Verb::Limn,
            "limn_not_a_glyph",
            &[("name", &word)],
            Role::Cost,
        );
        return;
    };
    // A dark glyph is refused as a cost rather than ignored, so a player who
    // read ahead in `recall` learns the circle is not whole yet.
    if world
        .get::<Beast>(circle)
        .is_some_and(|beast| !beast.shape().lights(glyph))
    {
        say(
            world,
            Verb::Limn,
            "limn_dark",
            &[("name", glyph.word())],
            Role::Cost,
        );
        return;
    }

    let humour = match humour {
        None => world
            .get_mut::<Beast>(circle)
            .map(|mut beast| beast.step(glyph)),
        Some(wanted) => {
            // Both slots are `NounKind::Place`, so two glyphs parse and have
            // to be refused here, as the lens's `dial` does.
            let Some(humour) = Humour::from_word(&wanted) else {
                say(
                    world,
                    Verb::Limn,
                    "limn_not_a_humour",
                    &[("name", &wanted)],
                    Role::Cost,
                );
                return;
            };
            world.get_mut::<Beast>(circle).map(|mut beast| {
                beast.limn(glyph, humour);
                humour
            })
        }
    };
    let Some(humour) = humour else {
        return;
    };
    // Published every time: the glyph's reading is how a spell knows where its
    // search stands. That glyph's alone — nothing else has moved.
    republish_glyph(world, circle, glyph);
    say(
        world,
        Verb::Limn,
        "limn_done",
        &[("name", glyph.word()), ("detail", humour.word())],
        Role::Success,
    );
}

/// Let the waiting beast go, for nothing — what `stop circle` does.
///
/// Returns whether there was one; a beast inserts no `Working`.
pub(crate) fn release(world: &mut World, circle: Entity) -> bool {
    if world.get::<Beast>(circle).is_none() {
        return false;
    }
    world.entity_mut(circle).remove::<Beast>();
    publish(world, circle);
    say(world, Verb::Stop, "summon_released", &[], Role::Cost);
    true
}

/// The circle, where the player — or the spell — is standing.
///
/// `Cwd` is right here: a spell's command runs inside `spell::run`'s room swap,
/// and the circle has no tick system.
fn fixture(world: &World) -> Option<Entity> {
    super::super::readings::fixture(world, Verb::Summon)
}

/// One authored line about the menagerie.
pub(super) fn say(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    super::super::readings::filed(world, CIRCLE, verb, key, args, role);
}
