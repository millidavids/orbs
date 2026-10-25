//! The laboratory's instruments, as something a pane can draw.
//!
//! §10.1's permanent panel, not the command stream: a transcript scrolls state
//! away. An accessor, not records, since the panel is current state redrawn
//! every frame. Information only (rule 2), so both builds draw the same bars.

use bevy_ecs::prelude::*;

use orbs_render::Wash;

use super::node::{Cwd, Fixture, Name, children_of};
use crate::tick::Tick;

/// A meter, as a fraction.
///
/// `done` over `total`, whatever it measures. The athanor reports fuel
/// *remaining* where an instrument reports ticks *elapsed*, which is what makes
/// one bar drain while the others fill — see
/// [`Burning::fuel`](super::Burning::fuel).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Meter {
    /// The filled part.
    pub done: u64,
    /// The whole.
    pub total: u64,
    /// What the two numbers are counting.
    ///
    /// Not always ticks, or `st 350t` reads as a job about to finish. Here, not
    /// in a frontend, for [`Instrument::short`]'s reason (rule 2).
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
    /// Rows of a beast's temper the circle last answered rightly.
    ///
    /// Out of the temper's rows — eight, or four at a lesser circle. The rail
    /// prints the remainder, the rows still balking, so it counts *down* as the
    /// circle comes right, like every work meter here. Before the first call it
    /// reads every row, nothing having been answered yet.
    Rows,
    /// How the tower's defences stand, out of [`STANDING`](super::STANDING).
    ///
    /// The one unit that counts up, so `brief::detail_of` prints it with `%`
    /// rather than as a remainder. See `panel::read`'s pylon arm.
    Standing,
}

/// What an instrument *does* — the action, not the noun.
///
/// Here, not in a frontend, so two builds cannot disagree (rule 2). A closed
/// set, like [`State`].
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
    /// Carrying wards between three stations (§10, `tower::pylon`).
    Warding,
    /// Binding a charm on a lattice of glyphs (§10, `tower::lattice`).
    Imbuing,
    /// Fragments into a scroll. The archive's lectern.
    ///
    /// The one craft not named by an [`Operation`](super::Operation): the lectern
    /// has none, four fragments being `move`d in and `wield`ed. What says it
    /// works is that it has *recipes* — read off the content, not off a name.
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
    /// Decided here, not by a frontend: rule 2 gives a frontend how a cell is
    /// drawn, not what word is in it.
    pub short: String,
    /// One word for its condition — what a reader hears.
    pub state: State,
    /// Its meter, if it has one running.
    pub meter: Option<Meter>,
    /// The colour families of what is inside it, in the order it is held.
    ///
    /// A hint over `survey`, never a substitute — see
    /// [`Tint`](orbs_render::Tint). An entry is `None` when nobody has tinted
    /// that material, and the bar draws that part in the base hue.
    ///
    /// Two, for `flask_and_rod`'s two becoming one; others fill the first. An
    /// array, since this is rebuilt every frame. Decided here, as
    /// [`Self::short`] is.
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
    /// A lectern with three of its four shards is neither
    /// [`Fouled`](Self::Fouled) nor [`Charged`](Self::Charged).
    Gathering,
    /// Running.
    Working,
    /// Being cleared — §9's triage slot.
    Scouring,
    /// Finished, with a product waiting to be emptied out.
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
    /// The one question behind a spell's `is idle` and `is working`, asked of
    /// the panel's own word so the two agree with the screen. Not `busy()`,
    /// which cannot see an athanor's [`Burning`](super::Burning) fire. `Banked`
    /// is fuel put by, not work.
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
/// What the rail asks of a room nobody is in; [`instruments`] is this with `Cwd`.
#[must_use]
pub fn instruments_in(world: &World, place: Entity) -> Vec<Instrument> {
    let now = *world.resource::<Tick>();

    // The dispensary is not an instrument: a shelf that does nothing, has no
    // meter and is `charged` from the first tick to the last. A row that never
    // changes teaches the eye to skip the panel, which is the one thing a
    // permanent fixture must not do. `survey dispensary` reads a shelf.
    let fixtures: Vec<Entity> = children_of(world, place)
        .into_iter()
        .filter(|node| world.get::<Fixture>(*node).is_some())
        .filter(|node| world.get::<super::Store>(*node).is_none())
        // And not a reading. The archive's four ways are places so a spell can
        // name them (`if north has passage` needs a `NounKind::Place`), and
        // places are fixtures — but a compass bearing has no meter, and four
        // rows reading `empty` for ever is the dispensary's argument again.
        .filter(|node| world.get::<super::Reading>(*node).is_none())
        // And not a satchel, on the dispensary's argument exactly — a shelf in
        // every room, six rows reading `empty` for ever. `survey satchel` reads
        // a queue.
        .filter(|node| world.get::<super::Satchel>(*node).is_none())
        .collect();

    let mut panel = Vec::with_capacity(fixtures.len());
    for node in fixtures {
        let name = world
            .get::<Name>(node)
            .map_or_else(String::new, |name| name.0.clone());
        // No `holds` list: a `String` per held reagent per instrument per frame
        // at 60 Hz, read by nothing in either frontend. What it reached for is
        // now `State::Fouled`.
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
/// The pane's derivation, for [`spell::holds`](super::spell::holds) and anything
/// else wanting the word. A non-instrument answers from what it holds, so `if
/// dispensary is empty` is a sentence.
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

/// The colour families of what is in an instrument, in the order it is held.
///
/// In `survey`'s order, so the flask's bands agree with the transcript. An
/// untinted material keeps its slot as `None`.
fn tints_of(world: &World, node: Entity) -> [Option<Wash>; 2] {
    let materials = world.resource::<crate::content::Materials>();
    let mut held = children_of(world, node)
        .into_iter()
        .filter_map(|held| world.get::<Name>(held))
        .map(|name| materials.wash(&name.0));
    [held.next().flatten(), held.next().flatten()]
}

/// What an instrument does.
///
/// From the entity's [`Operation`](super::Operation), never its name, which lost
/// a renamed instrument its picture. [`Craft::Heating`] comes from `HeatSource`
/// and [`Craft::Assembling`] from having recipes, so the `_` arm is a real
/// fallback — a new verb without an arm reaches it silently.
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
        Some(crate::parser::Verb::Muster) => Craft::Warding,
        Some(crate::parser::Verb::Imbue) => Craft::Imbuing,
        // No operation: an instrument with recipes runs anyway, and a fixture
        // with neither is a shelf.
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
    // An open puzzle is work and says so, whichever puzzle it is — through
    // `puzzle::Open`, matched exhaustively so a new one cannot be missed here
    // (see its doc). Each arm's meter is the only honest measure that puzzle has.
    if let Some(open) = super::puzzle::Open::on(world, node)
        && let Some(meter) = puzzle_meter(world, node, open)
    {
        return (State::Working, Some(meter));
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

    // The pylon's meter is always the barrier, course or no: a remainder of
    // wards-to-haul read as a barrier about to fail while a solver won (§19).
    // Below `Triaging`, or a scoured pylon read `idle` to the rail and to
    // `holding`'s own loop guard.
    if world
        .get::<super::Operation>(node)
        .is_some_and(|operation| operation.0 == crate::parser::Verb::Muster)
    {
        // A drawn course is `Working`, on the ward's reasoning above: a solver's
        // loop is `repeat until the pylon is idle`, and `is empty` cannot do that
        // because the pylon always carries its `integrity` reading.
        let state = if world.get::<super::Course>(node).is_some() {
            State::Working
        } else {
            State::Empty
        };
        return (
            state,
            Some(Meter {
                done: u64::from(world.resource::<super::Integrity>().get()),
                total: u64::from(super::STANDING),
                unit: Unit::Standing,
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
        // Something burnable, not merely something. Testing for any child
        // reported `charged` when the athanor held only the `ash` its burnout
        // left, one tick before `wield athanor` answered "nothing in it to burn".
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
    // Otherwise: does what is in there start anything? This fell through to
    // `Charged`, so an instrument holding only the last run's husks drew the same
    // word as one ready to go, and `speak()` filtered `Charged` out of its
    // utterance — the confusion the panel exists to remove.
    let holding = super::holdings(world, node);
    let recipes = world.resource::<crate::content::Recipes>();
    // A recipe the player has not found reads `fouled`, not `charged`: as far as
    // they know they are holding two things that make nothing, and `charged` for
    // a run that will never start is the lie this column exists to remove.
    let known = super::known(world);
    if recipes.matching(name, &holding, &known).is_some() {
        return (State::Charged, None);
    }
    // Part of a recipe is not leavings. The lectern wants four distinct shards,
    // so one, two or three fell through to `Fouled` — the panel telling a player
    // *collecting a set* that their instrument will not start.
    if recipes.gathering(name, &holding, &known) {
        return (State::Gathering, None);
    }
    (State::Fouled, None)
}

/// How far through an open puzzle is — the meter [`read`] draws while it is
/// open, or `None` for a puzzle whose fixture's meter is about something else.
///
/// `Working`, because a solver asks `repeat until the prism is idle`; `is empty`
/// reads published children, which a ward has none of before its first press.
/// Never a `Working` component, or a spell's `summon` would wait on the beast
/// it is calling.
fn puzzle_meter(world: &World, node: Entity, open: super::puzzle::Open) -> Option<Meter> {
    use super::puzzle::Open;
    match open {
        // Cells walked against cells there are: how long a maze takes is what the
        // player's rule decides.
        Open::Maze => world.get::<super::Maze>(node).map(|maze| {
            let (done, total) = maze.explored();
            Meter {
                done,
                total,
                unit: Unit::Cells,
            }
        }),
        // Sigils placed against sigils there are, by the last press's answer,
        // which falls as well as rises. It was `best()` while the ratchet held
        // the aperture at the best figure sent; with the ratchet gone (§19) a
        // high-water mark would read `3 of 4` over an aperture holding one.
        Open::Ward => world.get::<super::Ward>(node).map(|ward| Meter {
            done: u64::from(ward.last().0),
            total: super::ward::WIDTH as u64,
            unit: Unit::Sigils,
        }),
        // Columns snapped against columns there are. Without this arm the forge's
        // row read `lattice fouled` from the moment a charm was opened — the
        // fallthrough reading published residue as leavings to scour.
        Open::Binding => world.get::<super::lattice::Binding>(node).map(|binding| {
            let snapped = (0..super::lattice::WIDTH)
                .filter(|column| binding.lattice.snapped(*column))
                .count();
            Meter {
                done: snapped as u64,
                total: super::lattice::WIDTH as u64,
                unit: Unit::Sigils,
            }
        }),
        // The pylon's meter is always the barrier, drawn course or no, and `read`
        // gives it below — the one fixture whose meter is not about its own
        // puzzle.
        Open::Course => None,
        // Rows the last call agreed on, of the temper's rows.
        Open::Beast => world.get::<super::circle::Beast>(node).map(|beast| Meter {
            done: u64::from(beast.agreeing()),
            total: u64::try_from(beast.rows()).unwrap_or(u64::MAX),
            unit: Unit::Rows,
        }),
    }
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
        // Pinned against the real world, not a fixture. `craft_of` had no test
        // while it matched hardcoded names, so renaming an instrument in
        // `build.rs` would drop it to `Craft::Idle` and take its picture away
        // with a green suite.
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

        // The lectern and the stacks, a row each; the cabinet is a `Store`. The
        // root, with no instruments, measures that the panel is where you are.
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
        // collide. Against the real panel, not a hardcoded copy: this lived in
        // the Bevy crate over a `const LABORATORY: [&str; 5]`, so renaming an
        // instrument left it passing on stale names.
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

        // The next stage takes the product, which is what `siphon` did before it
        // retired (§19) — still the moment the mortar stops being `Ready`, since
        // what makes it ready is having something worth taking.
        sim.submit("digest ground-sage");
        sim.step();
        // `Fouled`, not `Charged`. The husks match no recipe, so the mortar will
        // not start — and this asserted `Charged`, the same word as an instrument
        // ready to go, while `speak()` filters `Charged` out of its utterance
        // entirely, so a listener heard nothing at all.
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
