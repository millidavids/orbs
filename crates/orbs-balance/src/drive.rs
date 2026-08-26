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
    ///
    /// **A bound policy breaks the rule of thumb this column came with.**
    /// *"Every entry should be a scour the policy asked for"* holds for a
    /// synthetic player typing and not for one watching a spell: a spell that
    /// **waits** stamps `Role::Cost` too — once per block, from `say_blocked` —
    /// which is the runner doing exactly what §8 asks of it. `bound` carries
    /// ~640 of them in a two-hour sweep and is perfectly healthy. Read `--why`
    /// rather than the number: a wait is the loop working, a refusal is the loop
    /// out of phase with the tower, and only the sentence separates them.
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
    let mut sweep = Sweep::default();
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
                Body::Scrying => press_one(&mut sim, &mut sweep),
                Body::Warding => haul_one(&mut sim, &mut cycle),
                Body::Bound {
                    earning,
                    name,
                    lines,
                } => {
                    if !hand_over(&mut sim, name, lines) {
                        let line = earning[cycle % earning.len()];
                        cycle += 1;
                        issue(&mut sim, line);
                    }
                }
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
    use orbs_render::{FieldName, Outcome, RecordKind, Role, Value};

    let (mut cost, mut landed) = (0, 0);
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();

    for record in sim.scrollback().records().iter() {
        // **A line that never resolved costs the policy a tick and used to be
        // invisible here.** Every parser outcome is `Role::Normal` by design —
        // `parser/report.rs` says the accent triad is danger, cost and success,
        // and *"a parser needing one more word is none of those"* — so a
        // command the tower could not read at all fell through the arm below
        // and contributed to neither column.
        //
        // That is precisely the misdiagnosis `--why` exists to prevent: the
        // downstream shape of an ambient reagent swap is `grind sage` ceasing to
        // parse, and the reader was shown a `cost` column that could not see it
        // and a reasons list pointing somewhere else entirely.
        let unread = matches!(
            record.outcome(),
            Some(Outcome::Unresolved | Outcome::Incomplete | Outcome::Candidate)
        );
        if unread {
            cost += 1;
            if let Some(Value::Text(why)) = record.field(FieldName::Message) {
                *reasons.entry(why.to_owned()).or_default() += 1;
            }
            continue;
        }
        match record.role() {
            Role::Cost | Role::Danger => {
                cost += 1;
                if let Some(Value::Text(why)) = record.field(FieldName::Message) {
                    *reasons.entry(why.to_owned()).or_default() += 1;
                }
            }
            // **A `Completion` that earned something**, not any success. This
            // counted every `Role::Success` record, so `attend`, `kindle`,
            // `empty` and each concentration gain all inflated it — `sweep
            // --ticks 0` reported five landed runs before a command had been
            // issued, against a field documented as *"runs that finished
            // successfully — the work the rate is made of"*.
            //
            // The rate is experience over ticks, so *the work the rate is made
            // of* is precisely a run that yielded experience. Both halves are
            // structural — a record kind and a counted field — so neither
            // depends on prose the way a match on the message would.
            Role::Success
                if record.kind() == RecordKind::Completion
                    && matches!(
                        record.field(FieldName::Quantity),
                        Some(Value::Count(earned)) if earned > 0
                    ) =>
            {
                landed += 1;
            }
            Role::Success | Role::Normal => {}
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

/// Get a spell written and bound, one step at a time.
///
/// Returns whether the player's hands were busy this step — `false` means there
/// is still experience to earn and the caller should play the earning cycle.
///
/// # Three visits, because each waits on a tick boundary
///
/// A slot has to be **earned**: concentration is derived from work completed,
/// `debug_spawn` deliberately earns nothing, and no public API hands the sim a
/// number, so the first stretch of this policy is a player grinding for sixteen
/// experience exactly as CLAUDE.md's own See-it line does.
///
/// Then the write, which is queued through `Pending` like every other effect and
/// lands on the next tick — so `bind` cannot be issued in the same breath, and a
/// driver that tried would name a spell the tower does not have yet.
///
/// # It goes through `write_spell`, not through a debug door
///
/// `Sim::write_spell` is the editor's own public entry: it records the
/// submission, so a swept session still replays, and it homes the spell to where
/// the policy is standing. `debug_spell` would have worked and would have been
/// wrong twice over — it is `cfg(debug_assertions)`, so a release sweep would
/// have measured nothing, and it hands back a *shipped* spell rather than one a
/// policy chose.
fn hand_over(sim: &mut Sim, name: &str, lines: &[&str]) -> bool {
    // **Compared by filename**, because that is what the tower holds. `bound()`
    // answers `tending.spell` where a policy names `tending`, so a bare `==`
    // never matched and the driver re-issued `bind` on every free tick — 1,920
    // refusals in a two-hour sweep, and a `cost` column reading four times the
    // work done. `with_extension` is the same normalisation every verb that
    // names a spell already goes through.
    let filename = orbs_sim::content::with_extension(name);

    // Standing automation. There is nothing left for a player to do, which is
    // the whole claim this policy measures — so burn the tick and watch.
    if sim.bound().contains(&filename) {
        sim.step();
        return true;
    }
    if sim.concentration() == 0 {
        return false;
    }
    if sim.spell(name).is_none() {
        let lines: Vec<String> = lines.iter().map(|line| (*line).to_string()).collect();
        sim.write_spell(name, &lines);
        sim.step();
        return true;
    }
    issue(sim, &format!("bind {name}"));
    true
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

/// The four sockets and the four sigils the aperture opens on.
///
/// **Spelled out rather than imported**, exactly as `orbs-sim/tests/ward.rs`
/// spells them out: a policy is a thing a player types, so a rename that broke
/// the *player's* vocabulary would still compile against `tower::ward::SOCKETS`
/// and this sweep would go on measuring a game nobody can play.
const SOCKETS: [&str; 4] = ["first", "second", "third", "fourth"];
const OPENING: [&str; 4] = ["nitre", "alum", "borax", "quartz"];

/// Where a scrying policy has got to in its sweep.
///
/// **Four sockets and a flag is the whole of it**, which is the point: §8's
/// language has no variables, so a policy carrying more state than this would be
/// measuring a spell nobody can write.
#[derive(Debug, Default, Clone, Copy)]
struct Sweep {
    /// The socket being worked, left to right.
    socket: usize,
    /// Whether the last press said this socket was already right.
    restore: bool,
}

/// One turn-and-press of the lens, chosen the way `breaking` chooses it.
///
/// Opens a reading if there is none — `probe` finds a far orb and presses it in
/// one word — and otherwise turns the socket in hand and presses. Three
/// outcomes, and they are the spell's three rungs:
///
/// 1. `aligned` fell, so this socket *was* right: put the opening sigil back and
///    press again, which re-syncs the baseline the next delta is measured from.
/// 2. `aligned` rose, so this socket has arrived: move to the next.
/// 3. neither: turn it again.
///
/// **The deltas are recomputed from the board rather than read off a reading**,
/// because `Sim` publishes the words to a *spell* and the numbers to a *record*.
/// Comparing the last two attempts is the same arithmetic `Ward::press` does and
/// cannot drift from it, whereas matching `the ward is closer` would be matching
/// prose — which rule 6 puts in a file precisely so nothing in Rust depends on it.
fn press_one(sim: &mut Sim, sweep: &mut Sweep) {
    // No reading open — either the first lap, or the last one gave. `probe`
    // opens the next and presses it, which is how a bound spell laps.
    if sim.ward().is_none() {
        *sweep = Sweep::default();
        issue(sim, "probe");
        return;
    }

    // Unreachable by the sweep's own proof — a socket's cyclic walk always
    // arrives — and a spin here would be silent, so it starts over rather than
    // pressing an aperture nothing is moving.
    if sweep.socket >= SOCKETS.len() {
        *sweep = Sweep::default();
    }
    let at = sweep.socket;

    if sweep.restore {
        issue(sim, &format!("dial {} {}", SOCKETS[at], OPENING[at]));
        issue(sim, "probe");
        sweep.restore = false;
        sweep.socket += 1;
        return;
    }

    issue(sim, &format!("dial {}", SOCKETS[at]));
    issue(sim, "probe");

    // The ward gave on that press. The next call opens another.
    let Some(board) = sim.ward() else { return };
    let mut recent = board.attempts.iter().rev();
    let (Some(now), Some(before)) = (recent.next(), recent.next()) else {
        return;
    };
    if now.aligned < before.aligned {
        sweep.restore = true;
    } else if now.aligned > before.aligned {
        sweep.socket += 1;
    }
}

/// The three stations, spelled out rather than imported.
///
/// A policy is a thing a *player* types, so a rename that broke the player's
/// vocabulary would still compile against `tower::pylon::STATIONS` — the same
/// reason `SOCKETS` above is written out.
const STATIONS: [&str; 3] = ["wellspring", "conduit", "barrier"];

/// One turn of the cyclic solution, or a fresh course when none is drawn.
///
/// **The whole algorithm is three pairs in rotation.** Between any two stations
/// exactly one haul is legal, so the policy never has to search — it picks the
/// pair whose turn it is and asks the course which way round the haul runs. That
/// is exactly what `dev_spells.toml`'s `holding` does with `let` and a part, and
/// no more: the parity comes off the course's own height, and the direction off
/// two `potency`s.
///
/// **The rotation and the direction are the sim's, not a copy of them.**
/// `tower::pylon::cycle` and `Course::between` are both `pub`, and `between`'s
/// own doc says it is kept *"even though nothing in the game calls it"* — this is
/// the caller it was waiting for. Transcribing either here would leave the
/// harness able to drift into measuring a slower, wrong-station solve while
/// `tower::pylon`'s optimality tests stayed green, and the code itself records
/// that the failure is invisible: *"getting this backwards still finishes — in
/// the conduit"*. `STATIONS` above stays written out for the opposite reason,
/// which its own doc gives: it is a thing a **player** types.
fn haul_one(sim: &mut Sim, cycle: &mut usize) {
    // No course drawn — either the first lap, or the last one finished.
    // `muster` draws the next, which is how a bound spell laps.
    let Some(course) = sim.course() else {
        *cycle = 0;
        issue(sim, "muster");
        return;
    };

    let pairs = orbs_sim::tower::pylon::cycle(course.height());
    let (a, b) = pairs[*cycle % pairs.len()];
    *cycle += 1;

    // **`None` starts the rotation over rather than returning.** Both stations
    // empty is unreachable while the rotation is in phase with the board — but
    // `run` only advances the clock inside `issue`, so a bare `return` here is a
    // spin with no tick, and `while sim.tick() < ticks` would never end. That is
    // the hang `press_one` guards against in as many words one function up.
    let Some((from, to)) = course.between(a, b) else {
        *cycle = 0;
        issue(sim, "muster");
        return;
    };
    issue(sim, &format!("haul {} {}", STATIONS[from], STATIONS[to]));
}

#[cfg(test)]
mod tests {
    use super::tally;
    use orbs_sim::Sim;

    #[test]
    fn a_line_the_tower_cannot_read_at_all_is_counted_and_named() {
        // **The gap `--why` was built to close, and could not see.** Every
        // parser outcome is `Role::Normal` — `parser/report.rs` reserves the
        // accent triad for danger, cost and success — so a command the tower
        // never resolved fell into neither column, and CLAUDE.md's instruction
        // to *"read the `cost` column before the rate"* was reading a number
        // that could not include it.
        let mut sim = Sim::new(0);
        sim.submit("xyzzy plugh");
        sim.step();

        let (cost, _, reasons) = tally(&sim);
        assert!(
            cost > 0,
            "an unreadable line cost the policy a tick and was tallied as free",
        );
        assert!(
            !reasons.is_empty(),
            "the line was counted but `--why` has nothing to say about it",
        );
    }

    #[test]
    fn nothing_lands_before_any_work_is_done() {
        // `landed` counted every `Role::Success` record, so a sim that had
        // issued no commands at all still reported finished runs — against a
        // field documented as *"the work the rate is made of"*.
        let sim = Sim::new(0);
        let (_, landed, _) = tally(&sim);
        assert_eq!(landed, 0, "the tower reported finished work at tick zero");
    }
}
