//! `summon` and `limn` — the menagerie's two words (DESIGN.md §10).
//!
//! The lens's shape, one room over: `summon` opens a beast and calls it in as
//! `probe` opens and presses, and `limn` turns a glyph as `dial` turns a
//! socket — named, or stepped round the six.
//!
//! Neither word takes the production slot and neither waits, so the menagerie's
//! faucet is additive; `spell::block::begins_work` carries the exemption (§19).
//! A hold puts troops into the arsenal and then the orb earns — `muster`'s
//! order. A balk costs only time (§19).

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::publish::{publish, republish_glyph};
use crate::content::Prose;
use crate::parser::{Intent, Verb};
use crate::session::Scrollback;
use crate::tower::{
    self,
    circle::{Beast, Call, Glyph, Humour},
};

/// The circle's name, which is where a menagerie record is filed and what
/// `progression.toml` prices a hold under.
pub(crate) const CIRCLE: &str = "circle";

/// What a troop is called on a shelf.
pub const TROOP: &str = "troop";

/// `summon` — draw a beast up at the circle, or call the waiting one in.
///
/// Two acts in one word, as `probe` is. One circle and one beast at a time, so
/// there is never anything to name, and the draw is not a call.
///
/// A call is never spent on a guess: `divined` is the augury having read the
/// line (§6), and a reader defeated by a sentence falls back to `summon`, so
/// *"cycle the widdershins"* came back as a call against par with nothing
/// limned.
pub(crate) fn summon(world: &mut World, divined: bool) {
    let Some(circle) = fixture(world) else {
        say(world, Verb::Summon, "summon_nowhere", &[], Role::Cost);
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
    call(world, circle);
}

/// A beast arrives at the circle.
fn arrive(world: &mut World, circle: Entity) {
    // The domain's one draw, here rather than in a system: the circle has no
    // tick system, so `RngStream::Menagerie` advances only when asked and a
    // replay has nothing to reorder.
    //
    // A sealed tower draws lesser beasts until `menagerie_2` opens the whole
    // circle.
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
fn call(world: &mut World, circle: Entity) {
    let Some(mut beast) = world.get_mut::<Beast>(circle) else {
        return;
    };
    let answered = beast.call();
    let calls = beast.calls();
    let troops = beast.troops();
    match answered {
        // A balk is the puzzle, not a dead end — a cost, never a fault.
        //
        // Nothing is republished: nothing a spell can read has moved (§19), and
        // republishing here re-raised every reading with new ids.
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
        Call::Held => hold(world, circle, calls, troops),
    }
}

/// Every row agreed: the beast is held, and the circle is clear for the next.
fn hold(world: &mut World, circle: Entity, calls: u32, troops: u32) {
    let Some(shape) = world.get::<Beast>(circle).map(Beast::shape) else {
        return;
    };
    world.entity_mut(circle).remove::<Beast>();
    publish(world, circle);

    let mut work = tower::Work::event(tower::FIGURE);
    // Through `tower::home`, never a named room: asking the rule stops a held
    // troop and a `debug_spawn`ed one landing in different places (§19).
    let shelf = tower::home(world, TROOP);
    if let Some(shelf) = shelf {
        tower::give(
            world,
            shelf,
            TROOP,
            crate::parser::NounKind::Essence,
            troops,
        );
        // A making, so renown is minted: the stores count makings, and a
        // producer that forgot would deploy troops the arsenal calls thin.
        work = work.making(TROOP);
    }
    let shelved = shelf.is_some();
    // A lesser hold is a quarter of the price — the lesson, not the faucet. One
    // rule for par either way; the circle's shape says what par is.
    let earned = shape.priced(tower::worth_within_par(world, CIRCLE, calls, shape.par()));
    // The sentence says what reached a shelf: announcing troops that never
    // arrived would be the record lying about the world.
    let answering = if shelved {
        world.resource::<Prose>().counted("summon_troops", troops)
    } else {
        world.resource::<Prose>().line("summon_troops_unkept", &[])
    };
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
/// Free and instant, as `dial` is: limning is not the work, holding the beast
/// is, and a player may change two glyphs and call once.
///
/// Bare is what makes a search writable. A spell has no variables, so it cannot
/// name a humour it has not tried; `limn keystone` asks the circle for the next
/// one, and three `repeat 6` loops around that are an odometer.
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
    // Either order, because no glyph shares a word with a humour. A humour
    // first makes the second word the glyph whatever it is, so the refusal
    // names the wrong word: `limn heed laboratory` is told `laboratory` is no
    // glyph.
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
/// Returns whether there was one. A beast inserts no `Working`, so without
/// this arm `stop circle` denied what `if the circle is working` affirmed.
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
/// `Cwd` is right here: a spell's command runs inside `spell::run`'s room swap.
/// The lens paid for a *tick system* reading `Cwd`; the circle has none.
pub(crate) fn fixture(world: &World) -> Option<Entity> {
    super::super::readings::fixture(world, Verb::Summon)
}

/// One authored line about the menagerie.
fn say(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    let message = world.resource::<Prose>().line(key, args);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Message, &message)
        .text(FieldName::Source, CIRCLE)
        .role(role)
        .finish();
}
