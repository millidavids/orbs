//! `summon` and `limn` — the menagerie's two words (DESIGN.md §10).
//!
//! The lens's shape, one room over: `summon` opens a beast the way `probe` opens
//! a ward and then calls it in the way `probe` presses, and `limn` turns a glyph
//! the way `dial` turns a socket — named, or stepped round the six when a spell
//! cannot name what it has not tried.
//!
//! **Neither word takes the production slot**, and neither waits on anything.
//! A beast waits for ever and a call answers on the tick it is typed, so a bound
//! search runs *beside* a brew, which is what makes the menagerie's faucet
//! additive. `spell::block::begins_work` carries the exemption; §19 records it
//! being forgotten once for the sanctum.
//!
//! # What a hold is worth
//!
//! Troops into the arsenal, and then the orb earns — the work, and then what the
//! work bought, which is `muster`'s order and the lens's. A call that balks costs
//! nothing but the call: this domain has no failure, only time (§19).

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
/// **Two acts in one word**, which is `probe`'s: finding a far orb and pressing
/// it, here drawing a beast and calling it. There is one circle and one beast at
/// a time, so there is never anything to name.
///
/// **The draw is not a call.** A beast arrives with every glyph at the opening
/// and nothing answered; the next `summon` is the first call, and the one PAR
/// counts.
///
/// **A call is never spent on a guess.** `divined` is the augury having read
/// the line (§6), and `summon` is what a reader falls back to when a sentence
/// defeats it: the word takes no argument and resolves in any menagerie, so
/// *"cycle the widdershins"* came back as a call against par with nothing
/// limned. A divined `summon` still draws a beast, which costs nothing; it does
/// not call one in, and says what to type instead.
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
    // **The domain's one draw, and it is here rather than in a system.** A beast
    // is drawn once per arrival, so `RngStream::Menagerie` advances only when
    // somebody asks — the circle has no tick system at all, and nothing a
    // replay could reorder.
    //
    // **Which circle is asked of the tower, at the draw.** A sealed tower draws
    // lesser beasts until `menagerie_2` opens the whole circle; a beast already
    // waiting keeps its shape when that happens, so the board never changes
    // under a player mid-read.
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
        // **A balk is not a refusal.** It is the circle answering wrongly, which
        // is the puzzle rather than a dead end — the refused haul's reading, and
        // why it is a cost and never a fault on the rail.
        //
        // **Nothing is republished**, because nothing a spell can read has moved:
        // the temper's `fervour` and each glyph's humour are what they were before
        // the call, and the answer is deliberately not a reading (§19). The board
        // and the rail read the beast itself. Republishing here despawned and
        // re-raised every reading on every call to say the same thing with new
        // ids — which `limn` did too until it republished only its glyph.
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
    // **Through `tower::home`, never to a named room.** A troop is finished work
    // and keeps itself in the arsenal; asking the rule rather than the room is
    // what stops a held troop and a `debug_spawn`ed one landing in different
    // places, which §19 records the archive's fragments doing for one change.
    let shelf = tower::home(world, TROOP);
    if let Some(shelf) = shelf {
        tower::give(
            world,
            shelf,
            TROOP,
            crate::parser::NounKind::Essence,
            troops,
        );
        // **A making**, so the troops arrive fresh and renown is minted for
        // them: the stores count makings, and a producer that forgot this would
        // deploy troops the arsenal still calls thin.
        work = work.making(TROOP);
    }
    let shelved = shelf.is_some();
    // **A lesser hold is a quarter of the price** — the lesson, not the faucet.
    // One rule for par either way; the circle's shape says what par is.
    let earned = shape.priced(tower::worth_within_par(world, CIRCLE, calls, shape.par()));
    // **The sentence says what reached a shelf.** `home` answers `None` only for a
    // tower with no arsenal, which no tower raised today is — but announcing four
    // troops the arsenal never received would be the record lying about the
    // world, and the player could not tell the hold had been lost.
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
/// **Free, and instant**, as `dial` is: limning is not the work, holding the
/// beast is. It is also what lets a player change two glyphs and call once,
/// which is the shape a deduction takes.
///
/// **Bare is what makes a search writable.** A spell has no variables, so it
/// cannot name the humour it has not tried; `limn keystone` asks the circle for
/// the next one instead, and three literal `repeat 6` loops around that are an
/// odometer over every circle there is.
pub(crate) fn limn(intent: &Intent, world: &mut World) {
    let Some(circle) = fixture(world) else {
        say(world, Verb::Limn, "summon_nowhere", &[], Role::Cost);
        return;
    };
    if world.get::<Beast>(circle).is_none() {
        say(world, Verb::Limn, "limn_unopened", &[], Role::Cost);
        return;
    }

    // **The leaf, for every sentence below.** A resolved place arrives as its
    // full path, and prose echoing it would read *"/tower/menagerie/keystone is
    // limned heed"* — the tower's internals in a line meant for a player.
    let mut named = intent
        .arguments
        .iter()
        .map(|argument| crate::parser::leaf(&argument.value).to_owned());
    let Some(first) = named.next() else {
        say(world, Verb::Limn, "limn_incomplete", &[], Role::Cost);
        return;
    };
    // **Either order, because only one can be meant.** No glyph shares a word
    // with a humour, so `limn heed keystone` names exactly what `limn keystone
    // heed` does — and it is the order plain English takes: *"limn the keystone
    // with heed"* and *"give the keystone heed"* are the glyph first, but a
    // player who thinks of the humour first types it first.
    //
    // **A humour first makes the second word the glyph, whatever it is**, so the
    // refusal names the word that is wrong: `limn heed laboratory` is told
    // `laboratory` is no glyph, not that `heed` is. Two glyphs are still refused
    // below by name.
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
    // **A dark glyph is refused, and as a cost** — at a lesser circle only the
    // keystone is part of it, and limning the others would change nothing the
    // beast answers. Said rather than ignored, so a player who read ahead in
    // `recall` learns the circle is not whole yet instead of watching a limn
    // do nothing.
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
            // **Both slots are `NounKind::Place`**, so `limn keystone sunwise`
            // parses and has to be refused here — the lens's `dial` has the same
            // shape. Naming what was wrong beats a generic refusal: a player who
            // named two glyphs is one word from right.
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
    // Published every time, because the glyph's reading is what a spell reads
    // back to know where its search stands — **that glyph's alone**, since
    // nothing else a spell can read has moved.
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
/// **Returns whether there was one**, so `stop` can fall through to its ordinary
/// answer when nothing waits. A beast inserts no `Working`, so without this arm
/// `stop circle` would say the circle is not working while `if the circle is
/// working` said it was — the disagreement the pylon's arm records.
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
/// **`Cwd` is right here, and it is not the lens's bug.** A spell's command runs
/// inside `spell::run`'s room swap, so `Cwd` is the spell's room when a bound
/// search calls `summon`. The lens paid for a *tick system* reading `Cwd`, and the
/// circle has no tick system.
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
