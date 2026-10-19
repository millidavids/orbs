//! The policies that spend scrolls, and what a run held at its end (§19,
//! *Supply*).

use std::collections::BTreeMap;

use orbs_sim::Sim;

use crate::drive::{issue, walk_one};

/// What every shelf and the arsenal held when the run ended, by name.
pub(crate) fn shelved(sim: &Sim) -> BTreeMap<String, u32> {
    use orbs_sim::tower::{Keep, Store, children_of, holdings, root};

    let world = sim.world();
    let mut held: BTreeMap<String, u32> = BTreeMap::new();
    let mut open = vec![root(world)];
    while let Some(node) = open.pop() {
        if world.get::<Store>(node).is_some() || world.get::<Keep>(node).is_some() {
            for (name, units) in holdings(world, node) {
                let total = held.entry(name).or_default();
                *total = total.saturating_add(units);
            }
        }
        open.extend(children_of(world, node));
    }
    held
}

/// What a policy spending scrolls has been handed.
#[derive(Default)]
pub(crate) struct Scrolling {
    pub(crate) given: BTreeMap<String, u32>,
}

impl Scrolling {
    /// Top the arsenal up to `scroll`'s slowest rate, so its supply can run.
    /// Returns whether it stocked any.
    ///
    /// Stocked as it is wanted, so freshness never decides the column.
    fn stock(&mut self, sim: &mut Sim, scroll: &str) -> bool {
        let opens = sim
            .world()
            .resource::<orbs_sim::content::Scrolls>()
            .get(scroll)
            .map_or(0, |calm| calm.rates.slowest().opens);
        let held = orbs_sim::tower::supply::held_in_arsenal(sim.world(), scroll);
        let short = opens.saturating_sub(held);
        if short > 0 {
            *self.given.entry(scroll.to_owned()).or_default() += short;
            issue(sim, &format!("debug_spawn {scroll} {short}"));
        }
        short > 0
    }
}

/// Whether `scroll`'s supply is open.
fn open(sim: &Sim, scroll: &str) -> bool {
    sim.world()
        .resource::<orbs_sim::tower::supply::Supplies>()
        .get(scroll)
        .is_some()
}

/// Keep a quickening supply open on the laboratory. Returns whether it acted.
///
/// Topped up after every draw, so it never runs dry, and opened again if it
/// has. The slowest rate: a raise draws two a time for a third, not a half.
pub(crate) fn quicken_one(sim: &mut Sim, state: &mut Scrolling) -> bool {
    const SCROLL: &str = "quickening-scroll";
    let stocked = state.stock(sim, SCROLL);
    if open(sim, SCROLL) {
        return stocked;
    }
    issue(sim, &format!("wield {SCROLL}"));
    true
}

/// One step of the archive with every maze set to gather.
///
/// The supply opened for the `research` and shut after it, which is how it is
/// meant to be played: left open, it draws while the walk takes thousands of
/// ticks and buys nothing more.
pub(crate) fn glean_one(sim: &mut Sim, state: &mut Scrolling) {
    const SCROLL: &str = "gleaning-scroll";
    if sim.stacks().is_none() {
        state.stock(sim, SCROLL);
        issue(sim, &format!("wield {SCROLL}"));
        issue(sim, "research");
        issue(sim, &format!("stop {SCROLL}"));
        return;
    }
    walk_one(sim);
}
