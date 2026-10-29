//! A glance at every domain at once — what the tower rail draws.
//!
//! §9 gives the minimised half one job, *awareness only, not commandable*, so
//! everything here is a reading that says what the domain's own pane would say
//! in fewer words, never more (rule 2).
//!
//! Every slot always, dark until opened: a rail that grew a box at a time would
//! move under the player, and anonymous dark boxes say *there is more* without
//! spending what §11's discovery loop wants.
//!
//! It asks the panel — [`state_at`](super::panel::state_at) and
//! [`instruments_in`](super::panel::instruments_in) — about a room nobody is
//! standing in, rather than answering beside it (§19).

use bevy_ecs::prelude::*;

use super::node::{Name, NodeId, children_of, root};
use super::panel::{Instrument, State};

/// The rooms where work happens, in the order the rail draws them.
///
/// Not the grimoire or the bailey: neither is a room you tend, and the
/// grimoire's box read `idle` for ever. Where a place can be shut is
/// `opened::is_room`'s question. A const table, because these are directory
/// names the parser resolves — decisions, not prose. §10's order, the two
/// opening domains first.
pub const DOMAINS: [&str; 6] = [
    "laboratory",
    "archive",
    "lens",
    "forge",
    "menagerie",
    // `sanctum`, not §10's `battlements/` (§19).
    "sanctum",
];

/// Something a domain wants noticed, latched until the player goes and looks.
///
/// Cleared by [`attend`](crate::execute), not a timer, or it would be told while
/// the player made tea and never again. One per domain: the rail says *something
/// happened here* and the room says what.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// A spell in this domain gave up, or could not read one of its own lines —
    /// otherwise silent from anywhere but its log (§8.1).
    Fault,
    /// Something was found here worth coming back for.
    News,
}

/// Every domain's mark, keyed by directory name.
///
/// A resource, read from a `&World` while a frame is painted.
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

/// Latch a mark on a domain. A fault outranks news, never the other way round:
/// the rail has one glyph to spend.
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

/// Latch a fault on whichever domain a running spell belongs to, by the
/// [`NodeId`] a `Running` carries.
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
    /// Whether the room is open; a sealed one draws as §11.5's dark box.
    pub built: bool,
    /// What the instrument the room is reported as is doing — see
    /// [`reported`](super::reported).
    pub state: State,
    /// What the room is waiting on, the answer `status rates` gives.
    pub wait: super::Wait,
    /// Twelve cells on what is working or waiting: `al 22t`, `at al`, `fr
    /// +salt`, `after mp`, or the forge's charm.
    pub detail: Option<String>,
    /// What a reader hears for [`detail`](Self::detail) when the cells are a
    /// shorthand: the whole sentence `status rates` says.
    pub detail_spoken: Option<String>,
    /// A spell running here, if one is.
    pub spell: Option<String>,
    /// How many cursors of automation are running here, in total.
    ///
    /// Cursors, not spells: a second `invoke` and an `alongside` fork are the
    /// same fact from the rail. Nought when nothing runs.
    pub running: usize,
    /// Something latched for the player's attention.
    pub mark: Option<Mark>,
    /// How far along the room's mastery line is, while it has a station left —
    /// the rail's one progression reading (§11.5).
    pub mastery: Option<super::Progress>,
}

/// A glance at every domain, in [`DOMAINS`] order.
///
/// Total, so the boxes stay in fixed positions whatever the world holds.
#[must_use]
pub fn briefs(world: &World) -> Vec<Brief> {
    let rooms = rooms_of(world);
    let marks = world.resource::<Marks>();
    let running = running_by_domain(world);
    let casts = running_spells(world);
    let lines = super::mastery(world);

    DOMAINS
        .into_iter()
        .map(|name| {
            let Some(node) = rooms.iter().find(|(had, _)| had == name).map(|(_, at)| *at) else {
                return Brief {
                    name,
                    built: false,
                    state: State::Empty,
                    wait: super::Wait::Nothing,
                    detail: None,
                    detail_spoken: None,
                    spell: None,
                    running: 0,
                    mark: None,
                    mastery: None,
                };
            };
            let open = world.resource::<super::Opened>().is_open(name);
            let instruments = super::panel::instruments_in(world, node);
            let reported = super::reported(&instruments);
            let wait = super::wait_in(&instruments, name, &casts);
            let (detail, detail_spoken) = detail(world, name, reported, &wait);
            Brief {
                name,
                built: open,
                mastery: if open {
                    lines
                        .iter()
                        .find(|line| line.domain == name)
                        .and_then(super::Line::progress)
                } else {
                    None
                },
                state: reported.map_or(State::Empty, |instrument| instrument.state),
                wait,
                detail,
                detail_spoken,
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

/// The detail row, and what a reader hears for it when it is shorthand.
///
/// The wait decides: a held spell says what holds it; an instrument says its
/// meter (`al 22t`) or else its wait (`at al`). The forge, with nothing else to
/// say, names the charm nearest lapsing, by the word the forge publishes.
fn detail(
    world: &World,
    room: &str,
    reported: Option<&Instrument>,
    wait: &super::Wait,
) -> (Option<String>, Option<String>) {
    let prose = world.resource::<crate::content::Prose>();
    let metered = match wait {
        super::Wait::At { .. } => reported.and_then(detail_of),
        super::Wait::Held { .. } | super::Wait::Nothing => None,
    };
    if let Some(drawn) = metered {
        return (Some(drawn), None);
    }
    if let Some(drawn) = super::wait::short(prose, wait) {
        return (Some(drawn), Some(super::wait_sentence(prose, wait)));
    }
    if room == super::charm::FORGE
        && let Some((kind, left)) = super::charm::soonest(world)
    {
        let state = if left <= super::charm::EBBING_AT {
            super::charm::EBBING
        } else {
            super::charm::GRACED
        };
        let count = left.to_string();
        let drawn = prose.line("brief_charm", &[("state", state), ("count", &count)]);
        let spoken = prose.line(
            "brief_charm_said",
            &[("name", kind.word()), ("state", state), ("count", &count)],
        );
        return (Some(drawn), Some(spoken));
    }
    (None, None)
}

/// `al 22t` — what is working and how much of it is left.
///
/// [`Instrument::short`], because fourteen columns cannot fit a name and the
/// number is the half a glance wants.
fn detail_of(instrument: &Instrument) -> Option<String> {
    let meter = instrument.meter?;
    // What a fire has left, not what it has burnt: one rule, `status rates`'s.
    let left = instrument.remaining()?;
    // Integrity counts up, so it prints what is done, with `%` so it does not
    // read as a remainder.
    if meter.unit == super::panel::Unit::Standing {
        return Some(format!("{} {}%", instrument.short, meter.done));
    }
    // `t` only on a duration: a count of squares or sigils is not a job about to
    // finish.
    (left > 0).then(|| match meter.unit {
        super::panel::Unit::Ticks => format!("{} {left}t", instrument.short),
        _ => format!("{} {left}", instrument.short),
    })
}

/// Every room a domain could be, by name: `/tower`'s children and the
/// filesystem root's, filtered by [`DOMAINS`] rather than by position.
pub(super) fn rooms_of(world: &World) -> Vec<(String, Entity)> {
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
/// The first names the room and the rest are counted, so a second `invoke` is
/// not invisible.
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
    /// What it yielded on last turn, if it yielded.
    pub held: Option<super::spell::Hold>,
}

/// Every spell running anywhere, with where and on how many cursors.
///
/// One walk for the rail and `status`, so the rail's `+2` and the listing that
/// says what the two are cannot disagree. Sorted by name, since archetype order
/// is no promise.
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
        // The invocable name, stripped here: rule 2 gives a frontend only how a
        // cell is drawn.
        found.push(Cast {
            spell: crate::content::without_extension(&spell).to_owned(),
            domain,
            cursors: state.strands.len().max(1),
            held: state.held.clone(),
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

    /// Nodes [`rooms_of`] reaches that are deliberately not domains, each named
    /// with its reason, so a rule broad enough to cover them cannot also cover a
    /// domain somebody forgot to register.
    const NOT_ROOMS: [&str; 4] = [
        // Where the spells live; still shut until its ley step
        // (`opened::is_room`).
        "grimoire",
        // The container every other room hangs under (§7).
        "tower",
        // The keep, reachable from every room.
        "arsenal",
        // A real place with its own log and verbs, and not a room you tend.
        siege::BAILEY,
    ];

    #[test]
    fn every_domain_the_tower_raises_has_a_slot_in_the_rail() {
        // A domain added to `build.rs` and not to `DOMAINS` would have no box.
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
        assert_eq!(briefs.len(), 6, "the rail is not six rooms");

        let built: Vec<_> = briefs.iter().filter(|brief| brief.built).collect();
        assert!(
            built.iter().any(|brief| brief.name == "laboratory"),
            "the laboratory is not on the rail",
        );
        // In an open tower every room is built; one that stopped being raised
        // should fail here rather than quietly draw dark.
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
        // Shown the find and not the broken spell is the worse way to be wrong.
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

    #[test]
    fn a_fire_on_the_rail_counts_its_fuel_left() {
        // The rail and `status rates` read one rule; the rail once said `at 12t`
        // of a fire with 588 ticks to burn.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("kindle charcoal");
        sim.step();
        for _ in 0..10 {
            sim.step();
        }
        let laboratory = briefs(sim.world())
            .into_iter()
            .find(|brief| brief.name == "laboratory")
            .and_then(|brief| brief.detail);
        let left: Option<u64> = laboratory
            .as_deref()
            .and_then(|detail| detail.strip_prefix("at "))
            .and_then(|count| count.strip_suffix('t'))
            .and_then(|count| count.parse().ok());
        assert!(left.is_some_and(|left| left > 500), "{laboratory:?}");
    }
}
