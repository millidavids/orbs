//! A glance at every domain at once — what the tower rail draws.
//!
//! DESIGN.md §9 gives the minimised half of the screen one job: *"awareness
//! only, **not commandable**"*. So everything here is a **reading**, never a
//! control, and it says the same things the domain's own pane would say in
//! fewer words — never anything extra, which is rule 2's line.
//!
//! # Seven, always, and four of them are dark
//!
//! §10 fixes the domain list at seven and §11.5 starts the player with two of
//! them. A rail that showed only what exists would grow a box at a time with no
//! warning, and the arrival of a *room* is the least surprising thing in the
//! game to foreshadow — so all seven slots are drawn and the unbuilt ones are
//! anonymous. Seven slots with four dark says *there is more* without saying
//! what, which is what §11's discovery loop wants and what naming them would
//! spend.
//!
//! # It asks the panel, rather than answering beside it
//!
//! [`state_at`](super::panel::state_at) and
//! [`instruments_in`](super::panel::instruments_in) are the same derivations the
//! instrument panel draws, called about a room the player is not standing in.
//! §19 records the cost of a second answer to *"is this instrument busy"* once
//! already; this is the same trap with a whole room in it.

use bevy_ecs::prelude::*;

use super::node::{Name, NodeId, children_of, root};
use super::panel::{Instrument, State};

/// §10's seven domains, in the order the rail draws them.
///
/// **A const table, not content.** These are directory names the parser resolves
/// and `build.rs` raises — decisions, like `Verb::canonical`'s table, rather than
/// prose. What each one is *called* on screen is the same word, because §7 makes
/// the filesystem the world and a room whose label differs from its path would
/// be two names for one place.
///
/// The order is §10's table read top to bottom, with the two opening domains
/// first so the rail's live half is its top half from the first frame.
pub const DOMAINS: [&str; 7] = [
    "laboratory",
    "archive",
    "lens",
    "grimoire",
    "forge",
    "menagerie",
    "battlements",
];

/// Something a domain wants noticed, latched until the player goes and looks.
///
/// **A latch cleared by [`attend`](crate::execute), not a timer.** A mark that
/// faded on its own would be a thing the game told you about while you were
/// making tea and never again; a mark that clears when you walk into the room is
/// diegetic, needs no second clock, and cannot drift from what the player has
/// actually seen. It is also why this is one mark per domain rather than a
/// queue: the rail says *something happened here*, and the room says what.
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
    /// What this domain wants noticed, if anything.
    #[must_use]
    pub fn get(&self, domain: &str) -> Option<Mark> {
        self.0.get(domain).copied()
    }
}

/// Latch a mark on a domain.
///
/// **A fault outranks news, and never the other way round.** Both can happen in
/// one tick — a scrying spell finding a recipe on the lap it then gives up on —
/// and the rail has one glyph to spend. A player who is shown the good news and
/// not the broken spell has been told the less useful of the two things.
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
    /// Whether the tower has this room yet. Four are `false` until Phase 9a.
    pub built: bool,
    /// What the busiest instrument in it is doing.
    pub state: State,
    /// That instrument's name and how long it has left, when it is working.
    pub detail: Option<String>,
    /// A spell running here, if one is.
    pub spell: Option<String>,
    /// Something latched for the player's attention.
    pub mark: Option<Mark>,
}

/// A glance at all seven domains, in [`DOMAINS`] order.
///
/// **Total**, so the rail's box count never depends on world state — a room that
/// is not built yet is a `Brief` with `built: false` rather than a gap, which is
/// what keeps the seven slots in fixed positions frame to frame. A box that
/// moved when a domain arrived would make the rail unreadable at the one moment
/// it has something to say.
#[must_use]
pub fn briefs(world: &World) -> Vec<Brief> {
    let rooms = rooms_of(world);
    let marks = world.resource::<Marks>();
    let running = running_by_domain(world);

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
                    mark: None,
                };
            };
            let instruments = super::panel::instruments_in(world, node);
            let busiest = busiest(&instruments);
            Brief {
                name,
                built: true,
                state: busiest.map_or(State::Empty, |instrument| instrument.state),
                detail: busiest.and_then(detail_of),
                spell: running
                    .iter()
                    .find(|(had, _)| had == name)
                    .map(|(_, spell)| spell.clone()),
                mark: marks.get(name),
            }
        })
        .collect()
}

/// The instrument whose state the whole room should be reported as.
///
/// **Busy beats idle, and the first busy one wins.** A room with four
/// instruments has one line in the rail, so it reports the thing a player would
/// want to be told — `working` if anything is, rather than `empty` because the
/// mortar happens to be. `State::is_busy` is the same test the panel and the
/// spell language both ask, so the rail cannot disagree with either.
fn busiest(instruments: &[Instrument]) -> Option<&Instrument> {
    instruments
        .iter()
        .find(|instrument| instrument.state.is_busy())
        .or_else(|| instruments.first())
}

/// `al 22t` — what is working and how much of it is left.
///
/// **The two-letter form, not the name.** The rail has fourteen columns and
/// `mortar_and_pestle 8t` is twenty; truncating gives `mortar_and_p`, which has
/// lost the number — the half a glance actually wants. [`Instrument::short`] is
/// the abbreviation the instrument panel already uses in its own narrow column,
/// so the rail and the panel name a tool the same way.
fn detail_of(instrument: &Instrument) -> Option<String> {
    let meter = instrument.meter?;
    let left = meter.total.saturating_sub(meter.done);
    // **`t` only when it is a duration.** This suffixed everything, so the
    // archive read `st 350t` for 350 unwalked squares and the lens `pr 4t` for
    // four sigils still astray — which counts *down* as the player wins and so
    // reads, on the one surface meant for a glance, as a job about to finish.
    // Two of the three built domains were glanceably wrong.
    (left > 0).then(|| match meter.unit {
        super::panel::Unit::Ticks => format!("{} {left}t", instrument.short),
        _ => format!("{} {left}", instrument.short),
    })
}

/// Every room a domain could be, by name.
///
/// **`/tower`'s children *and* the filesystem root's**, and the second half is
/// not tidiness. §7 puts `/grimoire` beside `/tower` rather than inside it — it
/// is where the player's spells live, not a room in the tower — but §10 lists
/// spellcraft as one of the seven domains. Walking only `/tower` would leave the
/// grimoire's box dark for ever with the directory sitting right there, which is
/// the rail lying about a room that exists.
///
/// [`root`] is the nameless node above both, so `tower` itself and any future
/// sibling are filtered by [`DOMAINS`] rather than by position.
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

/// Which spell, if any, is running in each domain.
fn running_by_domain(world: &World) -> Vec<(String, String)> {
    let Some(mut query) = world.try_query::<&super::spell::Running>() else {
        return Vec::new();
    };
    let mut found: Vec<(String, String)> = Vec::new();
    for state in query.iter(world) {
        let (Some(domain), Some(spell)) = (name_of(world, state.at), name_of(world, state.spell))
        else {
            continue;
        };
        if !found.iter().any(|(had, _)| *had == domain) {
            // **The invocable name, not the filename.** A spell node is named for
            // the file it was scribed to, so this reported `tending.spell` — which
            // is six columns of extension in a fourteen-column rail, and the rail
            // cut it to `tending.spe`. The word a player would type is `tending`.
            //
            // Stripped here rather than in the painter: rule 2 gives a frontend
            // only *how* a cell is drawn, so `orbs-tui` must not have to know that
            // spells live in files. `peruse` still wants the full name and still
            // has it — nothing else reads this.
            //
            // `content::without_extension`, not a `strip_suffix` of its own: the
            // rule for what a spell is called already exists in one place.
            let bare = crate::content::without_extension(&spell).to_owned();
            found.push((domain, bare));
        }
    }
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

    /// Nodes [`rooms_of`] reaches that are deliberately not domains.
    ///
    /// Spelled out with reasons rather than filtered by shape, because each is a
    /// different kind of not-a-room and a rule broad enough to cover all three
    /// would also cover a domain somebody forgot to register.
    const NOT_ROOMS: [&str; 2] = [
        // The container every other room hangs under (§7).
        "tower",
        // A keep, not a domain pane — §19's arsenal entry is explicit that it is
        // the one room reachable from every other, which is what makes it not
        // one of the seven.
        "arsenal",
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
    fn the_grimoire_is_on_the_rail_even_though_it_is_not_in_the_tower() {
        // §7 puts `/grimoire` *beside* `/tower` rather than inside it, and §10
        // lists spellcraft as one of the seven domains. Walking only `/tower`
        // left its box dark for ever with the directory sitting right there —
        // the rail saying a room does not exist while the player writes files
        // into it.
        let sim = Sim::new(1);
        let briefs = briefs(sim.world());
        let grimoire = briefs
            .iter()
            .find(|brief| brief.name == "grimoire")
            .expect("the grimoire has no box");
        assert!(grimoire.built, "the grimoire reads as unbuilt");
    }

    #[test]
    fn the_rail_always_has_seven_boxes_and_the_unbuilt_ones_are_anonymous() {
        let sim = Sim::new(1);
        let briefs = briefs(sim.world());
        assert_eq!(briefs.len(), 7, "the rail is not seven domains");

        let built: Vec<_> = briefs.iter().filter(|brief| brief.built).collect();
        assert!(
            built.iter().any(|brief| brief.name == "laboratory"),
            "the laboratory is not on the rail",
        );
        assert!(
            briefs.iter().any(|brief| !brief.built),
            "every domain reads as built, so the rail promises nothing",
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
