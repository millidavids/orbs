//! A glance at every domain at once — what the tower rail draws.
//!
//! DESIGN.md §9 gives the minimised half of the screen one job: *"awareness
//! only, **not commandable**"*. So everything here is a **reading**, never a
//! control, and it says the same things the domain's own pane would say in
//! fewer words — never anything extra, which is rule 2's line.
//!
//! Seven slots always, six dark in a fresh game. A rail that showed only what is
//! open would grow a box at a time with no warning, and six anonymous slots say
//! *there is more* without saying what — which is what §11's discovery loop
//! wants and what naming them would spend. Every room is raised at tick 0, so
//! `built` reads *opened*.
//!
//! It asks the panel rather than answering beside it:
//! [`state_at`](super::panel::state_at) and
//! [`instruments_in`](super::panel::instruments_in) are the derivations the
//! instrument panel draws, called about a room nobody is standing in. §19
//! records the cost of a second answer to *"is this instrument busy"*.

use bevy_ecs::prelude::*;

use super::node::{Name, NodeId, children_of, root};
use super::panel::{Instrument, State};

/// The rooms where work happens, in the order the rail draws them.
///
/// Six, not §10's seven: the grimoire is a kind of play but not a room you
/// *work in* — it raises no instrument, earns nothing and anchors no verb, so
/// its rail box read `idle` for ever. The bailey is absent for the same reason:
/// a siege is fought there and it is still not a room you tend. Where a place
/// can be *shut* is a wider question `opened::is_room` answers.
///
/// A const table, not content: these are directory names the parser resolves and
/// `build.rs` raises — decisions, like `Verb::canonical`'s table. The label is
/// the same word as the path, because §7 makes the filesystem the world.
///
/// The order is §10's table top to bottom, with the two opening domains first so
/// the rail's live half is its top half from the first frame.
pub const DOMAINS: [&str; 6] = [
    "laboratory",
    "archive",
    "lens",
    "forge",
    "menagerie",
    // `sanctum`, where §10's table says `battlements/` (§19): the room is a
    // warding chamber rather than a wall walk, because what defends the tower
    // is arcane and not masonry.
    "sanctum",
];

/// Something a domain wants noticed, latched until the player goes and looks.
///
/// A latch cleared by [`attend`](crate::execute), not a timer: a mark that faded
/// on its own would be something the game told you while you were making tea and
/// never again. Clearing on arrival needs no second clock and cannot drift from
/// what the player has seen. One mark per domain, not a queue — the rail says
/// *something happened here* and the room says what.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// A spell in this domain gave up, or could not read one of its own lines.
    ///
    /// §8's *"scripts always log and never halt"* means a broken spell is
    /// otherwise silent from anywhere but its own log — which is exactly the
    /// case §8.1 says a player must be able to notice without reading.
    Fault,
    /// Something was found here worth coming back for.
    News,
}

/// Every domain's mark, keyed by directory name.
///
/// A resource rather than a component on each domain node, because §9's rail is
/// drawn from a `&World` while a frame is being painted and a sparse map of at
/// most seven entries is cheaper to read than seven component lookups.
#[derive(Resource, Debug, Default, Clone)]
pub struct Marks(std::collections::BTreeMap<String, Mark>);

impl Marks {
    /// Every domain with something to say, and what it is.
    pub fn iter(&self) -> impl Iterator<Item = (&str, Mark)> {
        self.0.iter().map(|(domain, mark)| (domain.as_str(), *mark))
    }

    /// Put a set of marks back, for a save.
    pub(crate) fn restore(&mut self, marks: impl IntoIterator<Item = (String, Mark)>) {
        self.0 = marks.into_iter().collect();
    }

    /// What this domain wants noticed, if anything.
    #[must_use]
    pub fn get(&self, domain: &str) -> Option<Mark> {
        self.0.get(domain).copied()
    }
}

/// Latch a mark on a domain.
///
/// A fault outranks news, never the other way round. Both can happen in one tick
/// — a scrying spell finding a recipe on the lap it gives up on — and the rail
/// has one glyph to spend.
pub fn mark(world: &mut World, domain: &str, mark: Mark) {
    let mut marks = world.resource_mut::<Marks>();
    match marks.0.get(domain) {
        Some(Mark::Fault) if mark == Mark::News => {}
        _ => {
            marks.0.insert(domain.to_owned(), mark);
        }
    }
}

/// Forget a domain's mark — the player has gone and looked.
pub fn clear_mark(world: &mut World, domain: &str) {
    world.resource_mut::<Marks>().0.remove(domain);
}

/// Latch a fault on whichever domain a running spell belongs to.
///
/// Takes the [`NodeId`] a `Running` carries rather than a name, because that is
/// what the runner has to hand and looking the name up here keeps the two from
/// disagreeing about which room a spell is in.
pub fn mark_fault_at(world: &mut World, at: NodeId) {
    let Some(domain) = name_of(world, at) else {
        return;
    };
    mark(world, &domain, Mark::Fault);
}

/// One domain, at a glance.
#[derive(Debug, Clone)]
pub struct Brief {
    /// The directory name, which is also what the rail prints.
    pub name: &'static str,
    /// Whether the tower has this room yet. Four are `false` until Phase 11a.
    pub built: bool,
    /// What the busiest instrument in it is doing.
    pub state: State,
    /// That instrument's name and how long it has left, when it is working.
    pub detail: Option<String>,
    /// A spell running here, if one is.
    pub spell: Option<String>,
    /// How many cursors of automation are running here, in total.
    ///
    /// Cursors, not spells — the unit is the decision. A second `invoke` and an
    /// `alongside` fork are the same fact from the rail's position: *more is
    /// running than the name below can say*. `status` is where the difference
    /// lives, which is the standing split.
    ///
    /// Nought when nothing runs, so `spell.is_some()` and `running > 1` are
    /// separate questions and the painter asks the second only after the first.
    pub running: usize,
    /// Something latched for the player's attention.
    pub mark: Option<Mark>,
    /// How far along the room's mastery line is, while it has a station left.
    ///
    /// The one progression reading on the rail (§11.5): a percentage of the next
    /// station's deed, so a glance says how close the room is to opening
    /// something. Absent on a finished line and on a shut room.
    pub mastery: Option<super::Progress>,
}

/// A glance at all seven domains, in [`DOMAINS`] order.
///
/// Total, so the box count never depends on world state: an unbuilt room is a
/// `Brief` with `built: false` rather than a gap, which keeps the seven slots in
/// fixed positions. A box that moved when a domain arrived would make the rail
/// unreadable at the one moment it has something to say.
#[must_use]
pub fn briefs(world: &World) -> Vec<Brief> {
    let rooms = rooms_of(world);
    let marks = world.resource::<Marks>();
    let running = running_by_domain(world);
    // Once for all seven, rather than once per box: the lines are read from
    // the tally and the content, and nothing about them differs per room.
    let lines = super::mastery(world);

    DOMAINS
        .into_iter()
        .map(|name| {
            let Some(node) = rooms.iter().find(|(had, _)| had == name).map(|(_, at)| *at) else {
                return Brief {
                    name,
                    built: false,
                    state: State::Empty,
                    detail: None,
                    spell: None,
                    running: 0,
                    mark: None,
                    mastery: None,
                };
            };
            let open = world.resource::<super::Opened>().is_open(name);
            let instruments = super::panel::instruments_in(world, node);
            let busiest = busiest(&instruments);
            Brief {
                name,
                // Raised and opened. Every room is raised at tick 0; what a
                // fresh game has not earned draws as §11.5's dark box.
                built: open,
                mastery: if open {
                    lines
                        .iter()
                        .find(|line| line.domain == name)
                        .and_then(super::Line::progress)
                } else {
                    None
                },
                state: busiest.map_or(State::Empty, |instrument| instrument.state),
                detail: busiest.and_then(detail_of),
                spell: running
                    .iter()
                    .find(|(had, _, _)| had == name)
                    .map(|(_, spell, _)| spell.clone()),
                running: running
                    .iter()
                    .find(|(had, _, _)| had == name)
                    .map_or(0, |(_, _, cursors)| *cursors),
                mark: marks.get(name),
            }
        })
        .collect()
}

/// The instrument whose state the whole room should be reported as.
///
/// Busy beats idle, first busy wins: a room with four instruments has one line,
/// so it reports `working` if anything is rather than `empty` because the mortar
/// happens to be. `State::is_busy` is the test the panel and the spell language
/// both ask, so the rail cannot disagree with either.
fn busiest(instruments: &[Instrument]) -> Option<&Instrument> {
    instruments
        .iter()
        .find(|instrument| instrument.state.is_busy())
        .or_else(|| instruments.first())
}

/// `al 22t` — what is working and how much of it is left.
///
/// The two-letter form, not the name: the rail has fourteen columns and
/// `mortar_and_pestle 8t` is twenty, so truncating loses the number — the half a
/// glance wants. [`Instrument::short`] is the panel's own abbreviation, so both
/// name a tool the same way.
fn detail_of(instrument: &Instrument) -> Option<String> {
    let meter = instrument.meter?;
    let left = meter.total.saturating_sub(meter.done);
    // The one unit that counts up, answered before the gate below. Every other
    // meter measures work left, so the rail prints the remainder and falls
    // silent at none — which for integrity would read `py 60` at 40 and take the
    // sanctum's only glance away when the barrier is whole.
    //
    // The `%` is what stops it being read as a remainder, which is the `t`
    // suffix's job one arm down: without it `py 62` here and `py 4` below are
    // the same shape, and the sanctum said two unrelated things under one
    // prefix.
    if meter.unit == super::panel::Unit::Standing {
        return Some(format!("{} {}%", instrument.short, meter.done));
    }
    // `t` only when it is a duration. It suffixed everything, so the archive
    // read `st 350t` for 350 unwalked squares and the lens `pr 4t` for four
    // sigils astray — a number that counts *down* as the player wins, reading
    // as a job about to finish on the one surface meant for a glance.
    (left > 0).then(|| match meter.unit {
        super::panel::Unit::Ticks => format!("{} {left}t", instrument.short),
        _ => format!("{} {left}", instrument.short),
    })
}

/// Every room a domain could be, by name.
///
/// `/tower`'s children *and* the filesystem root's: §7 puts `/grimoire` beside
/// `/tower` rather than inside it, but §10 counts spellcraft as a domain — so
/// walking only `/tower` would leave the grimoire's box dark with the directory
/// sitting right there. [`root`] is the nameless node above both, so `tower`
/// itself and any future sibling are filtered by [`DOMAINS`], not by position.
fn rooms_of(world: &World) -> Vec<(String, Entity)> {
    let named = |node: Entity| world.get::<Name>(node).map(|name| (name.0.clone(), node));

    let outside = children_of(world, root(world));
    let inside = outside
        .iter()
        .find(|node| named(**node).is_some_and(|(name, _)| name == TOWER))
        .map(|tower| children_of(world, *tower))
        .unwrap_or_default();

    outside
        .into_iter()
        .chain(inside)
        .filter_map(named)
        .collect()
}

/// The room most domains hang under (§7). The grimoire is its sibling.
const TOWER: &str = "tower";

/// Which spell is running in each domain, and how many cursors in total.
///
/// The first spell names the room and the rest are counted, which is the rail's
/// shape everywhere — `busiest` picks one instrument out of four for the same
/// reason. It counted nothing before and kept only the first, so a second
/// `invoke` in one room was invisible: the rail said `►tending` whether one
/// spell ran there or three.
fn running_by_domain(world: &World) -> Vec<(String, String, usize)> {
    let mut found: Vec<(String, String, usize)> = Vec::new();
    for one in running_spells(world) {
        if let Some((_, _, already)) = found.iter_mut().find(|(had, _, _)| *had == one.domain) {
            *already = already.saturating_add(one.cursors);
            continue;
        }
        found.push((one.domain, one.spell, one.cursors));
    }
    found
}

/// One spell part-way through, as a surface wants to report it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cast {
    /// What a player would type to invoke it — no `.spell`.
    pub spell: String,
    /// The domain it runs in.
    pub domain: String,
    /// How many cursors it is running on. One unless it has forked.
    pub cursors: usize,
}

/// Every spell running anywhere, with where and on how many cursors.
///
/// One walk of `Running`, read by two surfaces: the rail folds it by domain into
/// a name and a count, `status` lists it whole. Two walks would be two answers
/// to *what is running*, and they would disagree in exactly the case that
/// matters — the rail's `+2` is meaningless unless something can say what the
/// two are.
///
/// Sorted by spell name, so a listing does not reorder between two frames of one
/// tick: `Running` is queried through the ECS and archetype order is no promise.
#[must_use]
pub fn running_spells(world: &World) -> Vec<Cast> {
    let Some(mut query) = world.try_query::<&super::spell::Running>() else {
        return Vec::new();
    };
    let mut found: Vec<Cast> = Vec::new();
    for state in query.iter(world) {
        let (Some(domain), Some(spell)) = (name_of(world, state.at), name_of(world, state.spell))
        else {
            continue;
        };
        // The invocable name, not the filename: a spell node is named for its
        // file, so this reported `tending.spell` and the fourteen-column rail
        // cut it to `tending.spe`. Stripped here rather than in the painter,
        // because rule 2 gives a frontend only *how* a cell is drawn — and
        // through `content::without_extension`, so the rule lives in one place.
        found.push(Cast {
            spell: crate::content::without_extension(&spell).to_owned(),
            domain,
            // Cursors, not spells: a forked spell is one `Running` wearing
            // several `Strand`s.
            cursors: state.strands.len().max(1),
        });
    }
    found.sort_by(|left, right| left.spell.cmp(&right.spell));
    found
}

/// A node's name, by the id that survives a save.
fn name_of(world: &World, id: NodeId) -> Option<String> {
    let mut query = world.try_query::<(&NodeId, &Name)>()?;
    query
        .iter(world)
        .find(|(had, _)| **had == id)
        .map(|(_, name)| name.0.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;
    use crate::tower::siege;

    /// Nodes [`rooms_of`] reaches that are deliberately not domains.
    ///
    /// Spelled out with reasons rather than filtered by shape, because each is a
    /// different kind of not-a-room and a rule broad enough to cover all four
    /// would also cover a domain somebody forgot to register.
    const NOT_ROOMS: [&str; 4] = [
        // Where the spells live, and not a room you work in (§19): it raises no
        // instrument, earns nothing and anchors no verb. Still a place, still
        // `Protected`, still shut until the ley step at 16 (`opened::is_room`).
        "grimoire",
        // The container every other room hangs under (§7).
        "tower",
        // A keep, not a domain pane — §19's arsenal entry is explicit that it is
        // the one room reachable from every other, which is what makes it not
        // one of the seven.
        "arsenal",
        // A place you descend into, not an eighth domain: §10 fixes the count at
        // seven and the siege is not among them. The arsenal's shape exactly — a
        // real place with its own log and verbs, deliberately not a rail box.
        // Seven fixed slots is what stops a box appearing and pushing the others
        // down at the one moment it has news.
        siege::BAILEY,
    ];

    #[test]
    fn every_domain_the_tower_raises_has_a_slot_in_the_rail() {
        // The drift this catches: a domain added to `build.rs` and not to
        // `DOMAINS` would be a room with no box, invisible from the rail for
        // ever, with every test still green.
        let sim = Sim::new(1);
        for (name, _) in rooms_of(sim.world()) {
            if NOT_ROOMS.contains(&name.as_str()) || name.contains('.') {
                continue;
            }
            assert!(
                DOMAINS.contains(&name.as_str()),
                "the tower has `{name}` and the rail has no box for it",
            );
        }
    }

    #[test]
    fn the_grimoire_has_no_box_and_is_still_a_room_that_can_be_shut() {
        // This test asserted the opposite for three phases (§19): the rail
        // carried a grimoire box because §10 lists spellcraft among the seven,
        // and it read `idle` for ever. What the room kept is what made it worth
        // having — the spells live there and `scribe` writes into it.
        let sim = Sim::new(1);
        assert!(
            !briefs(sim.world())
                .iter()
                .any(|brief| brief.name == "grimoire"),
            "the grimoire is back on the rail",
        );
        assert!(
            super::super::opened::is_room("grimoire"),
            "the grimoire stopped being a room that can be shut",
        );
        assert!(
            sim.is_open("grimoire"),
            "an open tower does not hold the grimoire",
        );
    }

    #[test]
    fn the_rail_always_has_six_boxes_and_the_unbuilt_ones_are_anonymous() {
        let sim = Sim::new(1);
        let briefs = briefs(sim.world());
        // Six, and the grimoire is the one that left (§19): §10's seven are
        // kinds of play, this list is the rooms you *work in*.
        assert_eq!(briefs.len(), 6, "the rail is not six rooms");

        let built: Vec<_> = briefs.iter().filter(|brief| brief.built).collect();
        assert!(
            built.iter().any(|brief| brief.name == "laboratory"),
            "the laboratory is not on the rail",
        );
        // Every one is built now; the forge was the last. This asserted the
        // opposite for six phases, which was right while rooms were still
        // arriving. Kept rather than deleted, because a domain that stopped
        // being raised should fail loudly here rather than quietly draw dark.
        assert!(
            briefs.iter().all(|brief| brief.built),
            "a room reads as unbuilt, but every one is raised now",
        );
    }

    #[test]
    fn a_mark_latches_and_a_visit_clears_it() {
        let mut sim = Sim::new(1);
        mark(sim.world_mut(), "laboratory", Mark::Fault);
        assert_eq!(
            world_mark(&sim, "laboratory"),
            Some(Mark::Fault),
            "the mark did not latch",
        );

        sim.submit("attend laboratory");
        sim.step();
        assert_eq!(
            world_mark(&sim, "laboratory"),
            None,
            "walking into the room did not clear the mark",
        );
    }

    #[test]
    fn a_fault_outranks_news_whichever_order_they_arrive_in() {
        // One glyph, two things to say. Being shown the find and not the broken
        // spell is the worse of the two ways to be wrong.
        let mut sim = Sim::new(1);
        mark(sim.world_mut(), "lens", Mark::News);
        mark(sim.world_mut(), "lens", Mark::Fault);
        assert_eq!(world_mark(&sim, "lens"), Some(Mark::Fault));

        mark(sim.world_mut(), "lens", Mark::News);
        assert_eq!(world_mark(&sim, "lens"), Some(Mark::Fault), "news won");
    }

    fn world_mark(sim: &Sim, domain: &str) -> Option<Mark> {
        sim.world().resource::<Marks>().get(domain)
    }
}
