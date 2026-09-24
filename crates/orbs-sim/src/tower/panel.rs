//! The laboratory's instruments, as something a pane can draw.
//!
//! DESIGN.md §10.1 makes the instrument panel a *permanent fixture* of the
//! laboratory's pane rather than part of the command stream: with four
//! instruments running you watch and respond, and a transcript that scrolls the
//! state away is not something you can watch.
//!
//! An accessor rather than records: rule 4 puts *command output* in records —
//! one line, one event, scrolled away — and the panel is the opposite shape, the
//! world's current state redrawn every frame. A record per instrument per tick
//! would bury the scrollback under its own furniture; `Sim::working` took the
//! same route and this generalises it.
//!
//! Rule 2 still holds: what is returned is *information*, and a frontend decides
//! how a cell is drawn. Nothing here is available to one frontend and not
//! another, so the terminal build draws the same bars.

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
    /// A meter is not always a duration, and the rail said it was. Four of the
    /// six things that raise one are ticks; the stacks count *cells explored*
    /// and the prism *sigils aligned*. `brief.rs` suffixed all of them with `t`,
    /// so the archive reported `st 350t` for 350 unwalked squares and the lens
    /// `pr 4t` for four sigils still astray — counting *down* as the player won,
    /// which reads on the rail as a job about to finish.
    ///
    /// The unit belongs here rather than in a frontend's `match` on the
    /// instrument's name, for the reason [`Instrument::short`] does: rule 2 gives
    /// a frontend *how* a cell is drawn, not what the thing in it is.
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
    /// The one unit that counts *up*. Every other meter measures work left to
    /// do, so the rail prints what remains; this measures a thing it is good to
    /// have more of, and `py 60` for a tower standing at 40 would be backwards.
    /// `brief::detail_of` answers for that and prints a `%`, so the two can never
    /// be read as one another.
    ///
    /// A `Wards` beside this was a defect: the pylon's meter used to be
    /// wards-still-to-haul while a course stood, so the rail's remainder counted
    /// down as a solver won and the barrier reading vanished. See `panel::read`'s
    /// pylon arm.
    Standing,
}

/// What an instrument *does* — the action, not the noun.
///
/// Named here rather than matched on in a frontend, for the reason
/// [`Instrument::short`] is: a frontend comparing against the literal
/// `"mortar_and_pestle"` would re-derive what lives here, and `orbs-tui` would
/// derive it a second time and could silently disagree (rule 2).
///
/// A closed set, like [`State`]: a view falling through to a default for an
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
    /// Decided here, not by a frontend. Rule 2 gives a frontend *how* a cell is
    /// drawn, not what word appears in it, and the Bevy build derived this from
    /// English stopwords in its own source — so `orbs-tui` would have had to
    /// reimplement the heuristic and the two could disagree about what `bm`
    /// means.
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
    /// Two, because `flask_and_rod` combines two (§10.1) and its picture is
    /// *these two becoming one*; every other instrument fills only the first. A
    /// fixed array rather than a `Vec` because this is rebuilt per instrument per
    /// frame — the `holds` list that used to live here went for allocating that
    /// way.
    ///
    /// Decided here rather than by a frontend, as [`Self::short`] and
    /// [`Self::craft`] are: otherwise two frontends each read `materials.toml`
    /// and map contents to a colour, a rule in two places that can disagree.
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
    /// The lectern is why this exists: its only recipe is an exact match on four
    /// distinct shards, so one, two or three fell through to
    /// [`Fouled`](Self::Fouled) — the panel telling a player collecting a set
    /// that their lectern *will not start*. [`Charged`](Self::Charged) is the
    /// opposite lie: it means wield it and it runs, and three shards do not.
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
    /// A spell's `is idle` and `is working` are complements and must never both
    /// answer yes, so there is one question and this is it.
    ///
    /// It was `busy()`, which reads [`Working`](super::Working) and
    /// [`Triaging`](super::Triaging) — right for the four instruments that
    /// consume Focus, but not the athanor: its fire is
    /// [`Burning`](super::Burning), deliberately not `Working`, because nothing
    /// counting the production pool may see it (§10.1). So `if athanor is idle`
    /// answered *yes* while it burned charcoal and `if athanor is working`
    /// answered no at the same moment — a fire in plain view on the panel,
    /// invisible to the only two words that ask about it.
    ///
    /// Asking the panel's own state ties the spell language to the word on
    /// screen, so a state added later cannot be busy for one and idle for the
    /// other.
    ///
    /// `Banked` is not busy: a damped fire is fuel put by, not work in progress,
    /// and a spell waiting for the athanor to be free should not wait for ever.
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
/// Extracted from [`instruments`] rather than written beside it: the tower rail
/// asks the same question about a room the player is not in, and two derivations
/// of *what is in this room* can disagree — the defect [`State::is_busy`] records
/// once. `instruments` is this with `Cwd` supplied.
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
/// The same derivation the pane draws, for anything needing the word without the
/// row — [`spell::holds`](super::spell::holds) asking whether a place is idle.
/// Sameness is the point: a second answer to "is this instrument busy" can
/// disagree, and the last one did (see [`State::is_busy`]).
///
/// A place that is not an instrument answers from what it holds, which makes
/// `if dispensary is empty` a sentence rather than a special case.
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
/// In world order, which is `survey`'s order: the flask's picture puts its two
/// ingredients in bands, and a panel ordering them differently from the
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

/// What an instrument does.
///
/// Read from the entity, never matched on its name: an instrument carries
/// [`Operation`](super::Operation), the verb that charges and starts it, and
/// each craft-bearing verb has an arm below. An earlier version matched four
/// hardcoded strings, the pattern `content/recipe.rs` records paying for once —
/// a renamed instrument fell through to [`Craft::Idle`] and lost its picture
/// with nothing failing.
///
/// Two crafts are not named by a verb and are answered either side of the match:
/// [`Craft::Heating`] by the `HeatSource` component, and [`Craft::Assembling`]
/// by the content, since the lectern has recipes and no operation. So the `_`
/// arm is a real fallback rather than a formality, and a new verb added without
/// an arm here reaches it silently — this is not a compiler-checked map.
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
        // `Kindle` is the heat source's and answered above. Anything else has no
        // operation — the dispensary, the cabinet, and the lectern, whose verb
        // went to the stacks when the maze did.
        //
        // So the fallback asks the content: an instrument with recipes runs
        // whether or not a verb names it, and a fixture with neither is a shelf.
        // Matching `"lectern"` here would be the hardcoded-name pattern this
        // function's header records paying for twice.
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

    // The pylon's meter is always the barrier, drawn course or no — the one
    // fixture whose *idle* state still carries a meter, and whose meter is not a
    // measure of its own run.
    //
    // It measured the course first, which was §19's recorded defect a third
    // time: `brief::detail_of` prints a meter's *remainder*, so
    // wards-still-to-haul counted down as a solver won, and the sanctum read
    // `py 4` to a player who had last seen `py 100` — a barrier about to fail.
    // Worse than the archive's `st 350t` and the lens's `pr 4t`, because the
    // number vanished precisely while a bound solver was working, the one time
    // the player is in another room and glancing. So the board says where the
    // wards are and this says whether the tower is safe; the state word above it
    // still says `working`.
    //
    // Below `Triaging`, and it shipped above it. A scoured pylon reported
    // `Empty` here while `tower::busy` reported `Scouring`, so the rail read
    // `idle`, `if the pylon is idle` answered yes mid-scour — `holding`'s own
    // loop guard — and `would_block` disagreed with all of them. The `Maze` and
    // `Ward` arms sit above `Triaging` too but are not reachable that way; this
    // one is, because `purge pylon` is ordinary to type.
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
/// Each is `Working` because that is what a solver asks: `repeat until the prism
/// is idle` is how a spell says *until the seal gives*. `is empty` cannot do that
/// job — `spell::watch` answers it by asking whether the node has children, a
/// puzzle's children are its published readings, and a ward has none until the
/// first press lands, so a `breaking` bounded on `empty` ended on its first
/// instruction. `Charged` is wrong the same way: it means *wield this and it
/// runs*.
///
/// None of them may be a `Working` component, which `tower::busy` reads: a
/// spell's `summon` would wait on the very beast it was calling, `would_block`
/// answering *"the circle is working"* to the command that finishes the work.
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

        // The archive has two, and it had one: giving it a fixture retired three
        // defects at once — a completion with no sentence, a run `stop` could not
        // reach, and a domain that drew nothing — and the second arrived when the
        // maze moved off the lectern onto the `stacks`. One row each, because *is
        // a reading open* and *is a scroll coming together* were one row with two
        // meanings.
        //
        // The `cabinet` is a `Store` and the panel leaves those out. The property
        // this test encodes — that the panel belongs to *where you are* — is
        // measured at the root, which has no instruments and never will.
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
