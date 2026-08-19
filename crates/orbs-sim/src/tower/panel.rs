//! The laboratory's instruments, as something a pane can draw.
//!
//! DESIGN.md §10.1 makes the instrument panel a **permanent fixture** of the
//! laboratory's pane rather than part of the command stream: with four
//! instruments running you watch and respond, and a transcript that scrolls the
//! state away is not something you can watch.
//!
//! # Why this is an accessor rather than records
//!
//! Rule 4 puts *command output* in records — one line, one event, scrolled away
//! once read. The panel is the opposite shape: it is the world's **current
//! state**, redrawn every frame, and emitting a record per instrument per tick
//! would bury the scrollback under its own furniture. The existing progress
//! meter took the same route (`Sim::working`), and this generalises it.
//!
//! Rule 2 still holds: what is returned here is *information*, and a frontend
//! decides how a cell is drawn. Nothing the panel shows is available to one
//! frontend and not another, so the terminal build draws the same bars.

use bevy_ecs::prelude::*;

use orbs_render::Wash;

use super::node::{Cwd, Fixture, Name, children_of};
use crate::tick::Tick;

/// A meter, as a fraction.
///
/// `done` over `total`, whatever it measures. The athanor reports fuel
/// **remaining** here where an instrument reports ticks **elapsed**, which is
/// the whole of what makes one bar drain while the others fill — see
/// [`Burning::fuel`](super::Burning::fuel).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Meter {
    /// The filled part.
    pub done: u64,
    /// The whole.
    pub total: u64,
    /// What the two numbers are counting.
    ///
    /// **A meter is not always a duration, and the rail said it was.** Four of
    /// the six things that raise one are ticks — an instrument at work, a scour,
    /// a fire burning, a fire banked — and two are not: the stacks count *cells
    /// explored* and the prism counts *sigils aligned*. `brief.rs` suffixed all
    /// of them with `t`, so the archive reported `st 350t` for 350 unwalked
    /// squares and the lens reported `pr 4t` for four sigils still astray —
    /// counting **down** 4 → 1 as the player won, which reads on the rail as a
    /// job about to finish.
    ///
    /// The unit belongs here rather than in a `match` on the instrument's name
    /// in a frontend, for the same reason [`Instrument::short`] does: rule 2
    /// gives a frontend *how* a cell is drawn, not what the thing in it is.
    pub unit: Unit,
}

/// What a [`Meter`]'s numbers count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Ticks — a duration, and the only one a `t` suffix is honest about.
    Ticks,
    /// Squares of maze.
    Cells,
    /// Sigils in the right socket.
    Sigils,
}

/// What an instrument *does* — the action, not the noun.
///
/// **Named here rather than matched on in a frontend**, for the reason
/// [`Instrument::short`] is: rule 2 gives a frontend *how* a cell is drawn, not
/// what the thing in it is. A frontend picking its picture by comparing against
/// the literal `"mortar_and_pestle"` would be re-deriving, in its own source,
/// knowledge that lives here — and `orbs-tui` would have to derive it a second
/// time and could silently disagree.
///
/// A closed set, like [`State`]: a view that fell through to a default for an
/// unrecognised craft would draw a working instrument as an idle one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Craft {
    /// Crushing one reagent into another. The mortar and pestle.
    Grinding,
    /// Gentle digestion over the athanor. The balneum mariae.
    Digesting,
    /// Two reagents into one. The flask and rod.
    Combining,
    /// A reagent into a potion. The alembic.
    Distilling,
    /// Not an operation at all — shared heat the others draw on. The athanor,
    /// which is why it is the one instrument taking no Focus (§10.1).
    Heating,
    /// Threading the archive's stacks (§10, `tower::maze`).
    Reading,
    /// Pressing figures against a far orb's ward (§10, `tower::ward`).
    ///
    /// Two fixtures share it — the oculus a reading is opened at and the prism
    /// it is pressed at — because a craft names *what a room does*, and both of
    /// those are scrying. What tells them apart on the panel is their state.
    Scrying,
    /// Fragments into a scroll. The archive's lectern.
    ///
    /// **The one craft not named by an [`Operation`](super::Operation)**, because
    /// the lectern has none: four fragments are `move`d in and `wield`ed, which
    /// is how an instrument runs when it has no verb of its own. What says it is
    /// working is that it has *recipes* — the craft is read off the content
    /// rather than off a name.
    Assembling,
    /// A fixture with no recipe of its own.
    Idle,
}

/// One instrument, as the panel draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instrument {
    /// What the player types to name it.
    pub name: String,
    /// What it does, for a view choosing how to picture it.
    pub craft: Craft,
    /// Two letters for a narrow column — `mortar_and_pestle` is `mp`.
    ///
    /// **Decided here, not by a frontend.** Rule 2 gives a frontend *how* a cell
    /// is drawn, not *what word appears in it*, and the Bevy build was deriving
    /// this from English stopwords in its own source — a rule that existed
    /// nowhere else, so `orbs-tui` would have had to reimplement the same
    /// heuristic and the two builds could silently disagree about what `bm`
    /// means. The panel module's own header already claimed otherwise: *"nothing
    /// the panel shows is available to one frontend and not another."*
    pub short: String,
    /// One word for its condition — what a reader hears.
    pub state: State,
    /// Its meter, if it has one running.
    pub meter: Option<Meter>,
    /// The colour families of what is inside it, in the order it is held.
    ///
    /// **A hint over `survey`, never a substitute** — see
    /// [`Tint`](orbs_render::Tint). An entry is `None` when nobody has tinted
    /// that material, and the bar draws that part in the base hue.
    ///
    /// **Two, because `flask_and_rod` combines two** (§10.1) and its picture is
    /// precisely *these two becoming one*. Every other instrument fills only the
    /// first. A fixed array rather than a `Vec` because this is rebuilt per
    /// instrument per frame, and the `holds` list that used to live here was
    /// removed for allocating exactly that way.
    ///
    /// **Decided here rather than by a frontend**, exactly as [`Self::short`] and
    /// [`Self::craft`] are: the alternative is two frontends each reading
    /// `materials.toml` and each mapping contents to a colour, which is a rule
    /// living in two places that can disagree.
    pub tints: [Option<Wash>; 2],
}

impl Instrument {
    /// The colour of what is inside, for the instruments that hold one thing.
    #[must_use]
    pub const fn tint(&self) -> Option<Wash> {
        self.tints[0]
    }
}

/// What an instrument is doing.
///
/// A closed set, for the same reason [`FieldName`](orbs_render::FieldName) is:
/// a view that silently skipped a state it did not recognise would draw a
/// working instrument as idle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Nothing in it, nothing to do.
    Empty,
    /// Holding something, not started.
    Charged,
    /// Holding part of what a recipe wants, and waiting for the rest.
    ///
    /// **The lectern is why this exists.** Its only recipe is an exact match on
    /// four distinct shards, so one, two or three of them matched nothing and
    /// fell through to [`Fouled`](Self::Fouled) — the panel telling a player
    /// collecting a set that their lectern *will not start*, which is the exact
    /// confusion the panel was built to remove. [`Charged`](Self::Charged) would
    /// be the opposite lie: it means wield it and it runs, and three shards do
    /// not.
    Gathering,
    /// Running.
    Working,
    /// Being cleared — §9's triage slot.
    Scouring,
    /// Finished, with a product waiting to be siphoned.
    Ready,
    /// Holding only what the last run fouled it with.
    Fouled,
    /// The athanor, alight.
    Burning,
    /// The athanor, damped with fuel kept.
    Banked,
    /// The athanor, cold and empty.
    Cold,
}

impl State {
    /// Whether the instrument is in the middle of something.
    ///
    /// # What `is idle` and `is working` ask, and why they ask it here
    ///
    /// A spell's two words are complements — `if athanor is idle` and
    /// `if athanor is working` must never both answer yes — so there is one
    /// question, asked once, and this is it.
    ///
    /// It was `busy()`, which reads [`Working`](super::Working) and
    /// [`Triaging`](super::Triaging) and is the right answer for the four
    /// instruments that consume Focus. **The athanor is not one of them**: its
    /// fire is [`Burning`](super::Burning), deliberately not `Working`, because
    /// nothing counting the production pool may see it (§10.1 — the athanor is
    /// infrastructure, not a stage). So `if athanor is idle` answered *yes*
    /// while it was burning charcoal, which is how it was reported, and
    /// `if athanor is working` answered no at the same moment — a fire in plain
    /// view on the panel, invisible to the only two words that ask about it.
    ///
    /// Asking the panel's own state instead ties the spell language to the word
    /// on screen: `at burning` in the pane and `athanor is working` in a spell
    /// are now the same fact, and a state added later cannot be busy for one and
    /// idle for the other.
    ///
    /// `Banked` is **not** busy. A damped fire is fuel put by, not work in
    /// progress, and a spell waiting for the athanor to be free should not wait
    /// on it for ever.
    #[must_use]
    pub const fn is_busy(self) -> bool {
        matches!(self, Self::Working | Self::Scouring | Self::Burning)
    }

    /// The word a reader hears and a column shows.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Charged => "charged",
            Self::Gathering => "gathering",
            Self::Working => "working",
            Self::Scouring => "scouring",
            Self::Ready => "ready",
            Self::Fouled => "fouled",
            Self::Burning => "burning",
            Self::Banked => "banked",
            Self::Cold => "cold",
        }
    }
}

/// Every instrument where the player is standing, in the order they were raised.
///
/// Empty anywhere but the laboratory, which is what makes the panel a property
/// of *where you are* rather than a thing the frontend has to decide to show.
#[must_use]
pub fn instruments(world: &World) -> Vec<Instrument> {
    instruments_in(world, world.resource::<Cwd>().0)
}

/// Every instrument standing in one place, whether or not the player is there.
///
/// **Extracted from [`instruments`] rather than written beside it**, because the
/// tower rail asks the same question about a room the player is not in, and two
/// derivations of *what is in this room* are two answers that can disagree — the
/// defect [`State::is_busy`] already records once. `instruments` is now this with
/// `Cwd` supplied, which keeps the panel a property of where you are standing
/// while letting the rail glance elsewhere.
#[must_use]
pub fn instruments_in(world: &World, place: Entity) -> Vec<Instrument> {
    let now = *world.resource::<Tick>();

    // The **dispensary is not an instrument** and is deliberately absent. It is
    // a shelf: it does nothing, it has no meter, and it is `charged` from the
    // first tick to the last — a row that never changes teaches the eye to skip
    // the panel, which is the one thing a permanent fixture must not do.
    // `survey dispensary` is how you read a shelf.
    let fixtures: Vec<Entity> = children_of(world, place)
        .into_iter()
        .filter(|node| world.get::<Fixture>(*node).is_some())
        .filter(|node| world.get::<super::Store>(*node).is_none())
        // **And not a reading.** The archive's four ways are places so a spell
        // can name them (`if north has passage` needs `north` to be a
        // `NounKind::Place`), and being places makes them fixtures — but a
        // compass bearing is not an instrument and has no meter, so four rows
        // reading `empty` for ever would teach the eye to skip the panel, which
        // is the same argument that keeps the dispensary off it.
        .filter(|node| world.get::<super::Reading>(*node).is_none())
        .collect();

    let mut panel = Vec::with_capacity(fixtures.len());
    for node in fixtures {
        let name = world
            .get::<Name>(node)
            .map_or_else(String::new, |name| name.0.clone());
        // No `holds` list. It was built here — a `String` per held reagent per
        // instrument per frame, at 60 Hz — and read by nothing in either
        // frontend. What it was reaching for is now `State::Fouled`, which is
        // the one question the contents were meant to answer.
        let (state, meter) = read(world, node, &name, now);
        panel.push(Instrument {
            short: abbreviate(&name),
            craft: craft_of(world, node),
            tints: tints_of(world, node),
            name,
            state,
            meter,
        });
    }
    panel
}

/// What one place is doing, in the panel's own words.
///
/// The same derivation the pane draws, for anything that needs the word without
/// the row — [`spell::holds`](super::spell::holds) asking whether a place is
/// idle, and nothing else so far. **The same derivation** is the point: a second
/// answer to "is this instrument busy" is a second answer that can disagree, and
/// the last one did — see [`State::is_busy`].
///
/// A place that is not an instrument at all answers from what it holds, which is
/// what makes `if dispensary is empty` a sentence rather than a special case.
#[must_use]
pub fn state_at(world: &World, node: Entity) -> State {
    let now = *world.resource::<Tick>();
    let name = world
        .get::<Name>(node)
        .map_or_else(String::new, |name| name.0.clone());
    read(world, node, &name, now).0
}

/// A two-letter form of an instrument's name, for a narrow column.
///
/// The initial of each meaningful part — `mortar_and_pestle` is `mp`,
/// `flask_and_rod` is `fr` — and the first two letters when there is only one
/// part, which is what keeps `alembic` and `athanor` apart as `al` and `at`.
/// Both rules are needed: initials alone would make them both `a`.
fn abbreviate(name: &str) -> String {
    let parts: Vec<&str> = name
        .split('_')
        .filter(|part| !matches!(*part, "and" | "of" | "the"))
        .collect();

    if parts.len() >= 2 {
        return parts
            .iter()
            .filter_map(|part| part.chars().next())
            .take(2)
            .collect();
    }
    name.chars().take(2).collect()
}

/// What an instrument does.
///
/// **Read from the entity, never matched on its name.** Every instrument already
/// carries [`Operation`](super::Operation) — the verb that charges and starts it,
/// put there by `build::raise` — and those five verbs are exactly the five
/// crafts. Reading it makes this a closed, compiler-checked map.
///
/// An earlier version matched four hardcoded strings with a silent fallthrough,
/// which is the pattern `content/recipe.rs` records having already paid for once:
/// `heat` was a `matches!` over two instrument names in Rust while the TOML noted
/// it in a comment, *"so a designer adding a heated instrument would have edited
/// this file, read their own note, and shipped a recipe that silently runs
/// cold."* A renamed instrument would have fallen through to [`Craft::Idle`] here
/// and quietly lost its picture, with nothing failing.
/// The colour families of what is in an instrument, in the order it is held.
///
/// **In world order, which is `survey`'s order.** The flask's picture puts its
/// two ingredients in bands, and a panel that ordered them differently from the
/// transcript would be two views of one vessel disagreeing about which reagent
/// is which.
///
/// Untinted materials keep their slot as `None` rather than being skipped, so
/// one uncoloured thing in a flask does not silently promote the other into its
/// band.
fn tints_of(world: &World, node: Entity) -> [Option<Wash>; 2] {
    let materials = world.resource::<crate::content::Materials>();
    let mut held = children_of(world, node)
        .into_iter()
        .filter_map(|held| world.get::<Name>(held))
        .map(|name| materials.wash(&name.0));
    [held.next().flatten(), held.next().flatten()]
}

fn craft_of(world: &World, node: Entity) -> Craft {
    // The heat source answers first, and by component: `heat::source` already
    // refuses to find it by name because a *reagent* called `athanor` lying on
    // the floor would have matched.
    if world.get::<super::HeatSource>(node).is_some() {
        return Craft::Heating;
    }
    match world.get::<super::Operation>(node).map(|verb| verb.0) {
        Some(crate::parser::Verb::Grind) => Craft::Grinding,
        Some(crate::parser::Verb::Digest) => Craft::Digesting,
        Some(crate::parser::Verb::Mix) => Craft::Combining,
        Some(crate::parser::Verb::Distil) => Craft::Distilling,
        Some(crate::parser::Verb::Research) => Craft::Reading,
        Some(crate::parser::Verb::Probe) => Craft::Scrying,
        // `Kindle` is the heat source's, and it answered above. Anything else has
        // no operation — which is the dispensary and the cabinet, and is *also*
        // the lectern, whose verb went to the stacks when the maze did.
        //
        // **So the fallback asks the content.** An instrument with recipes is an
        // instrument that runs, whether or not a verb names it, and a fixture
        // with neither is a shelf. Matching `"lectern"` here would be the
        // hardcoded-name pattern this function's own header records paying for
        // twice.
        _ => {
            let named = world.get::<super::Name>(node);
            let has_recipes = named.is_some_and(|name| {
                !world
                    .resource::<crate::content::Recipes>()
                    .for_instrument(&name.0)
                    .is_empty()
            });
            if has_recipes {
                Craft::Assembling
            } else {
                Craft::Idle
            }
        }
    }
}

/// What one instrument is doing, and how far through.
fn read(world: &World, node: Entity, name: &str, now: Tick) -> (State, Option<Meter>) {
    if let Some(work) = world.get::<super::Working>(node) {
        let (done, total) = work.progress(now);
        return (
            State::Working,
            Some(Meter {
                done,
                total,
                unit: Unit::Ticks,
            }),
        );
    }
    // **Walking the stacks is work, and says so** — even though it takes no
    // production slot. `if stacks is working` is how a solver asks whether its
    // maze is still open, and the meter is cells walked against cells there are:
    // the only honest measure a maze has, because how long it takes is what the
    // player's rule decides.
    if let Some(maze) = world.get::<super::Maze>(node) {
        let (done, total) = maze.explored();
        return (
            State::Working,
            Some(Meter {
                done,
                total,
                unit: Unit::Cells,
            }),
        );
    }
    // **An open ward is work, and says so** — the same answer the stacks gives
    // for an open maze, and for the same two reasons.
    //
    // It is what a solver asks: `repeat until the prism is idle` is how a spell
    // says *until the seal gives*, and `is empty` cannot do that job because
    // `spell::watch` answers it by asking whether the node has **children** —
    // the prism's children are its published readings, and there are none until
    // the first press lands. A `breaking` bounded on `empty` ended on its first
    // instruction, having pressed once.
    //
    // `Charged` was the first answer and is wrong for the same reason: it means
    // *wield this and it runs*, which is not what a reading in progress is.
    //
    // The meter is sigils placed against sigils there are — the only honest
    // measure a ward has, because how many presses it takes is what the player's
    // deduction decides, exactly as a maze's length is what their rule decides.
    if let Some(ward) = world.get::<super::Ward>(node) {
        return (
            State::Working,
            // **`best`, not `last().0`.** The meter is *progress*, and the last
            // press's answer goes down as well as up — a gauge that fell back
            // whenever a guess did not help would be reporting the guess rather
            // than the reading.
            Some(Meter {
                done: u64::from(ward.best()),
                total: super::ward::WIDTH as u64,
                unit: Unit::Sigils,
            }),
        );
    }
    if let Some(triage) = world.get::<super::Triaging>(node) {
        let total = triage.ends.get().saturating_sub(triage.started.get());
        let done = now.get().saturating_sub(triage.started.get()).min(total);
        return (
            State::Scouring,
            Some(Meter {
                done,
                total,
                unit: Unit::Ticks,
            }),
        );
    }

    // The heat source answers on its own terms: it runs no operation, so its
    // meter is fuel rather than progress, and it drains.
    if world.get::<super::HeatSource>(node).is_some() {
        if let Some(fire) = super::burning(world, node) {
            let (done, total) = fire.fuel(now);
            return (
                State::Burning,
                Some(Meter {
                    done,
                    total,
                    unit: Unit::Ticks,
                }),
            );
        }
        let banked = super::banked(world, node);
        if banked > 0 {
            return (
                State::Banked,
                Some(Meter {
                    done: banked,
                    total: banked,
                    unit: Unit::Ticks,
                }),
            );
        }
        // **Something burnable**, not merely something. Testing for any child at
        // all reported `charged` when the athanor held only the `ash` its own
        // burnout had left, one tick before `wield athanor` answered "nothing in
        // it to burn" — the panel and the verb contradicting each other on
        // screen at the same time.
        let fuels = world.resource::<crate::content::Fuels>();
        let holds_fuel = children_of(world, node).into_iter().any(|held| {
            world
                .get::<Name>(held)
                .is_some_and(|name| fuels.get(&name.0).is_some())
        });
        return (
            if holds_fuel {
                State::Charged
            } else {
                State::Cold
            },
            None,
        );
    }

    let held = children_of(world, node);
    if held.is_empty() {
        return (State::Empty, None);
    }
    // A product waiting is `ready`.
    if held
        .iter()
        .any(|held| world.get::<super::Product>(*held).is_some())
    {
        return (State::Ready, None);
    }
    // Otherwise: does what is in there start anything? **`Fouled` had no
    // construction site at all** — this fell through to `Charged`, so an
    // instrument holding only the husks of the last run drew the same word as one
    // charged and ready to go, and `speak()` filtered `Charged` out of its
    // utterance entirely. That is precisely the confusion the panel exists to
    // remove: "a fouled instrument read as *it will not start*".
    let holding = super::holdings(world, node);
    let recipes = world.resource::<crate::content::Recipes>();
    // **A recipe the player has not found reads `fouled`, not `charged`**, and
    // that is honest rather than coy: they are holding two things that make
    // nothing, as far as they know. A panel saying `charged` for a run that will
    // never start is the exact lie this column exists to remove.
    let learned = world.resource::<super::Learned>();
    if recipes.matching(name, &holding, learned).is_some() {
        return (State::Charged, None);
    }
    // **Part of a recipe is not leavings.** The lectern wants four distinct
    // shards, so one, two or three of them matched nothing and fell through to
    // `Fouled` — the panel telling a player *collecting a set* that their
    // instrument will not start, which is the confusion this whole column exists
    // to remove.
    if recipes.gathering(name, &holding, learned) {
        return (State::Gathering, None);
    }
    (State::Fouled, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    fn panel(sim: &mut Sim) -> Vec<Instrument> {
        instruments(sim.world_mut())
    }

    fn state_of(sim: &mut Sim, want: &str) -> State {
        panel(sim)
            .into_iter()
            .find(|instrument| instrument.name == want)
            .unwrap_or_else(|| panic!("no {want}"))
            .state
    }

    #[test]
    fn every_instrument_reports_the_craft_it_actually_does() {
        // **Pinned against the real world, not against a fixture.** `craft_of`
        // had no test at all while it matched hardcoded names, so renaming an
        // instrument in `build.rs` would have dropped it to `Craft::Idle` and
        // silently taken its picture away with a green suite — exactly what
        // moved `abbreviate`'s test into this file.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();

        let crafts: Vec<(String, Craft)> = panel(&mut sim)
            .into_iter()
            .map(|instrument| (instrument.name, instrument.craft))
            .collect();
        assert_eq!(
            crafts,
            vec![
                ("mortar_and_pestle".to_owned(), Craft::Grinding),
                ("balneum_mariae".to_owned(), Craft::Digesting),
                ("flask_and_rod".to_owned(), Craft::Combining),
                ("alembic".to_owned(), Craft::Distilling),
                ("athanor".to_owned(), Craft::Heating),
            ],
        );
    }

    #[test]
    fn the_panel_is_a_fact_about_where_you_stand() {
        // The panel belongs to *where you are*, so a frontend never has to
        // decide whether to show it.
        let mut sim = Sim::new(1);
        sim.step();
        assert!(panel(&mut sim).is_empty(), "the tower root has instruments");

        // **The archive has two**, and it had one: giving it a fixture at all is
        // what retired three defects at once — a completion with no sentence, a
        // run `stop` could not reach, and a domain that drew nothing — and the
        // second arrived when the maze moved off the lectern onto the `stacks`.
        // One row each, which is the point: *is a reading open* and *is a scroll
        // coming together* were one row with two meanings.
        //
        // The `cabinet` is not counted. It is a `Store`, and the panel leaves
        // those out because a shelf reading `charged` from the first tick to the
        // last teaches the eye to skip the panel.
        //
        // The property this test actually encodes — that the panel belongs to
        // *where you are* — is measured at the root, which has no instruments
        // and never will.
        sim.submit("attend archive");
        sim.step();
        assert_eq!(
            panel(&mut sim).len(),
            2,
            "the archive lost the lectern or the stacks",
        );

        sim.submit("attend tower");
        sim.step();
        assert!(panel(&mut sim).is_empty(), "the root has instruments");
    }

    #[test]
    fn the_laboratory_reports_its_instruments_in_the_order_they_were_raised() {
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        let names: Vec<String> = panel(&mut sim)
            .into_iter()
            .map(|instrument| instrument.name)
            .collect();
        assert_eq!(
            names,
            [
                "mortar_and_pestle",
                "balneum_mariae",
                "flask_and_rod",
                "alembic",
                "athanor",
            ]
        );
    }

    #[test]
    fn the_dispensary_is_not_on_the_panel() {
        // It is a shelf: no meter, nothing to do, and `charged` from the first
        // tick to the last. A row that never changes teaches the eye to skip the
        // panel, which is the one thing a permanent fixture must not do.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        assert!(
            !panel(&mut sim)
                .into_iter()
                .any(|instrument| instrument.name == "dispensary"),
            "the shelf is taking a row"
        );
    }

    #[test]
    fn every_instrument_abbreviates_to_two_distinct_letters() {
        // `alembic` and `athanor` both start with `a`, so initials alone would
        // collide — which in a two-letter column is indistinguishable.
        //
        // Against the **real panel**, not a hardcoded copy of the instrument
        // list. This test lived in the Bevy crate and asserted over a
        // `const LABORATORY: [&str; 5]`, so renaming an instrument in `build.rs`
        // left it passing on stale names and adding a colliding one went
        // unnoticed anywhere.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();

        let short: Vec<String> = panel(&mut sim)
            .into_iter()
            .map(|instrument| instrument.short)
            .collect();
        assert_eq!(short, ["mp", "bm", "fr", "al", "at"]);

        let mut unique = short.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), short.len(), "two instruments share a label");
    }

    #[test]
    fn an_instrument_walks_through_its_states() {
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        assert_eq!(state_of(&mut sim, "mortar_and_pestle"), State::Empty);

        sim.submit("move sage to mortar_and_pestle");
        sim.step();
        assert_eq!(state_of(&mut sim, "mortar_and_pestle"), State::Charged);

        sim.submit("wield mortar_and_pestle");
        sim.step();
        assert_eq!(state_of(&mut sim, "mortar_and_pestle"), State::Working);

        sim.step_n(20);
        assert_eq!(state_of(&mut sim, "mortar_and_pestle"), State::Ready);

        // **The next stage takes the product**, which is what `siphon` used to
        // do before it retired (§19) — and it is still the moment the mortar
        // stops being `Ready`, because what makes it ready is having something
        // worth taking in it.
        sim.submit("digest ground-sage");
        sim.step();
        // **`Fouled`, not `Charged`.** The husks left behind match no recipe, so
        // the mortar will not start — and this asserted `Charged`, the same word
        // as an instrument loaded and ready to go, while `State::Fouled` had no
        // construction site anywhere in the workspace. The panel exists to stop
        // "a fouled instrument read as *it will not start*"; it was saying the
        // opposite, and `speak()` filters `Charged` out of its utterance
        // entirely, so a screen-reader user heard nothing at all.
        assert_eq!(
            state_of(&mut sim, "mortar_and_pestle"),
            State::Fouled,
            "the husks are still in there and start nothing"
        );

        sim.submit("purge mortar_and_pestle");
        sim.step();
        assert_eq!(state_of(&mut sim, "mortar_and_pestle"), State::Scouring);

        sim.step_n(super::super::PURGE_TICKS);
        assert_eq!(state_of(&mut sim, "mortar_and_pestle"), State::Empty);
    }

    #[test]
    fn a_working_instrument_reports_a_filling_meter() {
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("move sage to mortar_and_pestle");
        sim.step();
        sim.submit("wield mortar_and_pestle");
        sim.step();

        let early = panel(&mut sim)[0].meter.expect("a meter");
        sim.step_n(4);
        let later = panel(&mut sim)[0].meter.expect("a meter");

        assert_eq!(early.total, later.total);
        assert!(later.done > early.done, "the meter did not fill");
    }

    #[test]
    fn the_athanor_reports_a_draining_meter() {
        // The one bar that runs the other way. Same `Meter`, same fraction — it
        // is handed fuel remaining rather than ticks elapsed.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("move charcoal to athanor");
        sim.step();
        sim.submit("wield athanor");
        sim.step();

        let fire = |sim: &mut Sim| {
            panel(sim)
                .into_iter()
                .find(|instrument| instrument.name == super::super::ATHANOR)
                .expect("the athanor")
        };

        let early = fire(&mut sim).meter.expect("a meter");
        sim.step_n(6);
        let later = fire(&mut sim).meter.expect("a meter");

        assert_eq!(early.total, later.total);
        assert!(later.done < early.done, "the fuel meter did not drain");
    }

    #[test]
    fn the_athanor_reports_banked_fuel_rather_than_looking_cold() {
        // The defect that made damping read as losing the charcoal: banked fuel
        // has no node, so without this the panel shows an empty, cold athanor.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        assert_eq!(state_of(&mut sim, "athanor"), State::Cold);

        sim.submit("move charcoal to athanor");
        sim.step();
        sim.submit("wield athanor");
        sim.step();
        sim.step_n(5);
        sim.submit("stop athanor");
        sim.step();

        assert_eq!(state_of(&mut sim, "athanor"), State::Banked);
        assert!(
            panel(&mut sim)
                .into_iter()
                .any(|instrument| instrument.meter.is_some()),
            "banked fuel is invisible again"
        );
    }
}
