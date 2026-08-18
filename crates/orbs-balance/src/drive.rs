//! Driving a real `Sim` with a synthetic player, and sampling the curve.
//!
//! # It goes through `submit`, like everything else
//!
//! The harness has no privileged entry point: every command a policy issues is
//! a line of text handed to [`Sim::submit`], resolved by the real parser and run
//! on the next tick by the real schedule. That is what makes a sweep and a
//! hand-played session comparable at all — CLAUDE.md's rule 4 in practice, and
//! the reason there is deliberately no `Sim::scrollback_mut`.
//!
//! # A command is issued when the tower is free
//!
//! `CAPACITY` is 1 tower-wide, so a second production command issued while the
//! first is in flight is refused, earns nothing, and quietly makes the sweep
//! measure a policy nobody could play. The driver therefore waits on
//! [`Sim::working`] exactly as a bound spell waits on the slot, and that wait is
//! the *only* timing model here — no typing delay, no reaction time, nothing
//! §19's Phase 0.5 entry refused.

use std::collections::BTreeMap;

use orbs_sim::Sim;

use crate::policy::{Body, Policy};

/// One row of the curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample {
    /// World time, in ticks — one real second with the window open.
    pub tick: u64,
    /// The unlock currency (§11.5). Only ever rises.
    pub experience: u64,
    /// Spells the orb can hold, derived from the total against the curve.
    pub concentration: usize,
}

/// A finished sweep of one policy.
#[derive(Debug, Clone)]
pub struct Run {
    /// Which policy produced it.
    pub policy: &'static str,
    /// The seed the world was built from.
    pub seed: u64,
    /// The curve, one row per sampling interval plus a final row.
    pub samples: Vec<Sample>,
    /// Records the tower stamped with the **cost** accent.
    ///
    /// **The diagnostic a balance harness most needs, and the first draft had no
    /// column for it.** A policy whose commands are refused earns nothing for
    /// them and still spends the ticks, so its rate is real but it is not
    /// measuring the loop anybody wrote — it is measuring a loop that has fallen
    /// out of phase with the tower. Two sweeps were read as balance findings
    /// before this existed, and both were the policy's fault.
    ///
    /// **It counts a scour as well as a refusal, and cannot tell them apart.**
    /// `refuse_busy` and `purge` both stamp [`Role::Cost`](orbs_render::Role) —
    /// §4's accent triad says *"mana and arcane expenditure"*, which a scour
    /// honestly is. Discriminating on the sentence would be matching prose, and
    /// rule 6 puts prose in a file precisely so nothing in Rust depends on its
    /// wording. So the number is *"things that cost something"* and
    /// [`reasons`](Self::reasons) is what tells you which — which is why `--why`
    /// exists rather than a cleverer classifier here.
    pub cost: usize,
    /// Runs that finished successfully — the work the rate is made of.
    pub landed: usize,
    /// Why the tower turned things down, commonest first.
    ///
    /// A count alone says a policy is broken; this says *where*. The sentence is
    /// the authored prose the player would have read, so a reason here is
    /// greppable straight back to `prose.toml`.
    pub reasons: BTreeMap<String, usize>,
}

impl Run {
    /// Experience per tick over the whole run — the number every policy is
    /// compared on.
    ///
    /// Returns 0.0 rather than dividing by nought for a run of no length, which
    /// is reachable with `--ticks 0` and is not worth an error.
    #[must_use]
    pub fn rate(&self) -> f64 {
        let Some(last) = self.samples.last() else {
            return 0.0;
        };
        if last.tick == 0 {
            return 0.0;
        }
        // Both are small enough that f64 is exact here: experience over a
        // 25-hour session is thousands, ticks are tens of thousands.
        #[allow(clippy::cast_precision_loss)]
        {
            last.experience as f64 / last.tick as f64
        }
    }

    /// What the run finished on.
    #[must_use]
    pub fn last(&self) -> Sample {
        self.samples.last().copied().unwrap_or(Sample {
            tick: 0,
            experience: 0,
            concentration: 0,
        })
    }
}

/// Run one policy for `ticks` and return its curve.
///
/// `every` is the sampling interval. A row is always emitted for tick 0 and for
/// the final tick, so a curve is never empty and its ends are always exact.
#[must_use]
pub fn run(policy: Policy, seed: u64, ticks: u64, every: u64) -> Run {
    let mut sim = Sim::new(seed);
    let mut samples = vec![sample(&sim)];

    for line in policy.setup {
        issue(&mut sim, line);
    }

    let mut cycle = 0usize;
    let every = every.max(1);

    while sim.tick().get() < ticks {
        // **Free first, then act.** Asking for the next command while a run is
        // in flight would have it refused and the lap lost; this is the same
        // wait a bound spell performs against the production slot.
        if busy(&sim) {
            sim.step();
        } else {
            match policy.body {
                Body::Cycle(lines) => {
                    let line = lines[cycle % lines.len()];
                    cycle += 1;
                    issue(&mut sim, line);
                }
                Body::Stacks => walk_one(&mut sim),
            }
        }

        if sim.tick().get().is_multiple_of(every) {
            samples.push(sample(&sim));
        }
    }

    // The final row, exact, even when the run ended between intervals.
    let last = sample(&sim);
    if samples.last() != Some(&last) {
        samples.push(last);
    }

    let (cost, landed, reasons) = tally(&sim);
    Run {
        policy: policy.name,
        seed,
        samples,
        cost,
        landed,
        reasons,
    }
}

/// Count what the tower turned down against what it finished.
///
/// `Role` is the discriminator rather than any field: `refuse_busy` and every
/// other refusal in `tower/work/slot.rs` stamps [`Role::Cost`], a completion
/// stamps [`Role::Success`], and a breach stamps [`Role::Danger`]. That is the
/// accent triad §4 already commits to, so counting on it cannot drift from what
/// the player sees on screen.
///
/// [`Role::Cost`]: orbs_render::Role::Cost
/// [`Role::Success`]: orbs_render::Role::Success
/// [`Role::Danger`]: orbs_render::Role::Danger
fn tally(sim: &Sim) -> (usize, usize, BTreeMap<String, usize>) {
    use orbs_render::{FieldName, Role, Value};

    let (mut cost, mut landed) = (0, 0);
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();

    for record in sim.scrollback().records().iter() {
        match record.role() {
            Role::Cost | Role::Danger => {
                cost += 1;
                if let Some(Value::Text(why)) = record.field(FieldName::Message) {
                    *reasons.entry(why.to_owned()).or_default() += 1;
                }
            }
            Role::Success => landed += 1,
            Role::Normal => {}
        }
    }
    (cost, landed, reasons)
}

/// Hand one line to the sim and let it land.
///
/// A submitted line is queued and executed at the **start of the next tick**
/// (`orbs_sim::session`), so a caller that does not step never sees its effect —
/// which is the commonest way to write a sweep that measures nothing.
fn issue(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

/// Whether the tower has anything in flight or queued.
///
/// **Three things, and the triage slot is the one that was missed.**
/// [`Sim::working`] reports the *production* slot only, so a driver waiting on it
/// alone runs straight over a `purge` — §9 gives a pane one production slot *and
/// one triage slot*, and a scour lives in the second. The symptom was a policy
/// scouring twice a lap and being told *"you are already scouring the
/// `balneum_mariae`"*, with the sweep reading 0.072 for a loop worth 0.140.
///
/// A scour is visible on the panel as [`State::Scouring`], which is the same
/// accessor the instrument panel draws from — so this cannot disagree with what
/// a player would see.
///
/// [`State::Scouring`]: orbs_sim::tower::State::Scouring
fn busy(sim: &Sim) -> bool {
    use orbs_sim::tower::State;

    sim.working().is_some()
        || !sim.pending().is_empty()
        || sim
            .instruments()
            .iter()
            .any(|instrument| instrument.state == State::Scouring)
}

fn sample(sim: &Sim) -> Sample {
    Sample {
        tick: sim.tick().get(),
        experience: sim.experience(),
        concentration: sim.concentration(),
    }
}

/// One step of the archive, chosen the way `threading` chooses it.
///
/// Opens a maze if there is none, otherwise issues a single `follow`. Tiers, in
/// order, and they are the solver spell's rungs rather than a second idea about
/// how a maze is solved:
///
/// 1. a spoil to pick up, 2. the way out, 3. floor nobody has walked,
/// 4. the least-walked way that is not where we came from, 5. back.
///
/// **The `back` rung is what makes it a solver.** §19 records the version
/// without one — *"the solver that was never a solver"* — which solved one seed
/// and cycled for ever on another, because a fixed compass order sends the
/// reading back where it came from at any junction where two ways read alike.
fn walk_one(sim: &mut Sim) {
    let Some(maze) = sim.stacks() else {
        issue(sim, "research");
        return;
    };

    let width = usize::from(maze.width.max(1));
    let at = maze.at;
    // `Way::ALL` order — north, east, south, west — which `Stacks::open` also
    // indexes. The two must agree, and this is the only place that assumes it.
    let neighbour = |way: usize| -> Option<usize> {
        let (x, y) = (at % width, at / width);
        match way {
            0 => y.checked_sub(1).map(|y| y * width + x),
            1 => (x + 1 < width).then_some(at + 1),
            2 => Some(at + width),
            _ => x.checked_sub(1).map(|x| y * width + x),
        }
    };

    let mut best: Option<(u8, usize)> = None;
    let mut chosen = None;
    for way in 0..4 {
        if !maze.open(way) {
            continue;
        }
        let Some(next) = neighbour(way) else { continue };
        if maze.spoils.contains(&next) || maze.exit == Some(next) {
            chosen = Some(way);
            break;
        }
        let marks = maze
            .squares
            .get(next)
            .map_or(u8::MAX, |square| square.marks);
        if best.is_none_or(|(fewest, _)| marks < fewest) {
            best = Some((marks, way));
        }
    }

    let way = chosen.or_else(|| best.map(|(_, way)| way));
    match way {
        // `Way::ALL`'s own words, which is what `follow` parses.
        Some(0) => issue(sim, "follow north"),
        Some(1) => issue(sim, "follow east"),
        Some(2) => issue(sim, "follow south"),
        Some(_) => issue(sim, "follow west"),
        // Walled in on all four sides is unreachable in a carved maze, but a
        // step that does nothing would spin the driver for ever. Burn the tick.
        None => sim.step(),
    }
}
