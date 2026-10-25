//! Driving a real `Sim` with a synthetic player, and sampling the curve.
//!
//! No privileged entry: every command is text through [`Sim::submit`], so a
//! sweep compares with play (rule 4). The driver waits on [`Sim::working`] as
//! a bound spell waits on the slot — the only timing model (§19).

use std::collections::BTreeMap;

use orbs_sim::Sim;
use orbs_sim::tower::circle::{self, Glyph};

use crate::policy::{Body, Policy};
use crate::scrolling::{Scrolling, glean_one, quicken_one, shelved};

/// One row of the curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample {
    /// World time, in ticks — one real second with the window open.
    pub tick: u64,
    /// The unlock currency (§11.5). Only ever rises.
    pub experience: u64,
    /// Spells the orb can hold, derived from the total against the curve.
    pub concentration: usize,
    /// The standing currency (§11.5). Rises on a sale, falls on a bad siege.
    ///
    /// Sampled, or the rank thresholds are unfalsifiable (§19); experience is
    /// no proxy for it.
    pub renown: u64,
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
    /// The key diagnostic: a refused policy measures a loop out of phase. A
    /// scour, a refusal and a spell's wait all stamp
    /// [`Role::Cost`](orbs_render::Role), so read [`reasons`](Self::reasons)
    /// (`--why`), not the number.
    pub cost: usize,
    /// Runs that finished successfully — the work the rate is made of.
    pub landed: usize,
    /// Why the tower turned things down, commonest first.
    ///
    /// A count alone says a policy is broken; this says *where*. The sentence is
    /// the prose the player would have read, so a reason greps straight back to
    /// `prose.toml`.
    pub reasons: BTreeMap<String, usize>,
    /// What every shelf and the arsenal held when the run ended, by name.
    pub held: BTreeMap<String, u32>,
    /// What a policy that spends a supply was handed as it went, by name.
    /// Less [`held`](Self::held), it is what the run spent (§19, *Supply*).
    pub given: BTreeMap<String, u32>,
}

impl Run {
    /// Experience per tick over the whole run — the number every policy is
    /// compared on.
    ///
    /// Returns 0.0 rather than dividing by nought for a run of no length —
    /// reachable with `--ticks 0`, and not worth an error.
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
            renown: 0,
        })
    }
}

/// Run one policy for `ticks` and return its curve.
///
/// `every` is the sampling interval. A row is always emitted for tick 0 and the
/// final tick, so a curve is never empty and its ends are exact.
#[must_use]
pub fn run(
    policy: Policy,
    seed: u64,
    ticks: u64,
    every: u64,
    length: orbs_sim::content::Length,
) -> Run {
    // An open tower at the asked-for length. `Sim::begun` is a sealed *game*,
    // which would shut every room a policy needs.
    let mut sim = Sim::measured(seed, length);
    let mut samples = vec![sample(&sim)];

    for line in policy.setup {
        issue(&mut sim, line);
    }

    let mut cycle = 0usize;
    let mut sweep = Sweep::default();
    let mut fighting = Fighting::default();
    let mut scrolling = Scrolling::default();
    let every = every.max(1);

    while sim.tick().get() < ticks {
        // Free first, then act: asking for the next command while a run is in
        // flight has it refused and loses the lap. The wait a bound spell
        // performs against the production slot.
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
                Body::Taming => tame_one(&mut sim),
                Body::Besieging => fight_one(&mut sim, &mut fighting),
                Body::Imbuing => bind_one(&mut sim),
                Body::Gleaned => glean_one(&mut sim, &mut scrolling),
                Body::Quickened(lines) => {
                    if !quicken_one(&mut sim, &mut scrolling) {
                        let line = lines[cycle % lines.len()];
                        cycle += 1;
                        issue(&mut sim, line);
                    }
                }
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
    let held = shelved(&sim);
    let given = scrolling.given;
    Run {
        policy: policy.name,
        seed,
        samples,
        cost,
        landed,
        reasons,
        held,
        given,
    }
}

/// Count what the tower turned down against what it finished.
///
/// By §4's accent triad — [`Role::Cost`], [`Role::Success`], [`Role::Danger`]
/// — so the count matches the screen.
///
/// [`Role::Cost`]: orbs_render::Role::Cost
/// [`Role::Success`]: orbs_render::Role::Success
/// [`Role::Danger`]: orbs_render::Role::Danger
fn tally(sim: &Sim) -> (usize, usize, BTreeMap<String, usize>) {
    use orbs_render::{FieldName, Outcome, RecordKind, Role, Value};

    let (mut cost, mut landed) = (0, 0);
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();

    for record in sim.scrollback().records().iter() {
        // An unresolved line costs a tick but is `Role::Normal`, so count it
        // here, or a reagent swap stopping `grind sage` hides from `--why`.
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
            // A `Completion` that yielded experience, not any success — what the
            // rate is made of — read from a kind and a field, not prose.
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
/// A submitted line is queued and executed at the start of the next tick
/// (`orbs_sim::session`), so a caller that does not step never sees its effect —
/// the commonest way to write a sweep that measures nothing.
pub(crate) fn issue(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

/// Whether the tower has anything in flight or queued.
///
/// The triage slot too, since [`Sim::working`] reports production only (§9),
/// read through the panel's own [`State::Scouring`].
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
        renown: sim.renown(),
    }
}

/// Get a spell written and bound, one step at a time.
///
/// Returns whether the player's hands were busy this step — `false` means there
/// is still experience to earn and the caller should play the earning cycle.
///
/// Three visits, each on a tick boundary: earn the slot by work, write, bind.
/// Through `Sim::write_spell`, the editor's entry, so the session replays —
/// never `debug_spell`, which is debug-only.
fn hand_over(sim: &mut Sim, name: &str, lines: &[&str]) -> bool {
    // Compared by filename, because that is what the tower holds: `bound()`
    // answers `tending.spell` where a policy names `tending`, so a bare `==`
    // never matched and the driver re-issued `bind` every free tick — 1,920
    // refusals in a two-hour sweep.
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
/// Opens a maze if there is none, otherwise issues a single `follow`. The tiers
/// are the solver spell's rungs rather than a second idea about how a maze is
/// solved:
///
/// 1. a spoil to pick up, 2. the way out, 3. floor nobody has walked,
/// 4. the least-walked way that is not where we came from, 5. back.
///
/// The `back` rung makes it a solver (§19).
pub(crate) fn walk_one(sim: &mut Sim) {
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
/// Spelled out rather than imported, as `orbs-sim/tests/ward.rs` does: a policy
/// is a thing a player types, so a rename breaking the *player's* vocabulary
/// would still compile against `tower::ward::SOCKETS` and go on measuring a game
/// nobody can play.
const SOCKETS: [&str; 4] = ["first", "second", "third", "fourth"];
const OPENING: [&str; 4] = ["nitre", "alum", "borax", "quartz"];

/// Where a besieging policy has got to.
///
/// Two facts that were one `usize`. `fight_one` used the shared `cycle` counter
/// as a round marker during a siege *and* as an idle tick count between them, so
/// a finished siege left its last round number behind and the "every 64 ticks"
/// `defend` retry fired 51 to 58 ticks later. `Sweep` below is the precedent: a
/// policy that needs state gets a named one.
#[derive(Debug, Default, Clone, Copy)]
struct Fighting {
    /// The round a spend has already been made on, so the ladder runs once.
    spent_on: usize,
    /// The round the arsenal was last topped up on.
    ///
    /// Separate from [`spent_on`](Self::spent_on): a restock sharing the spend
    /// marker *consumes* the round's one command, so the ladder never evaluated
    /// on a restock round — over half of all rounds spent shopping, in the policy
    /// whose job is to measure fighting.
    restocked_on: usize,
    /// Ticks spent waiting for the road, between sieges.
    idle: usize,
}

/// Where a scrying policy has got to in its sweep.
///
/// Four sockets and a flag: §8's language has no variables, so a policy carrying
/// more state would be measuring a spell nobody can write.
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
/// one word — and otherwise turns the socket in hand and presses. The three
/// outcomes are the spell's three rungs:
///
/// 1. `aligned` fell, so this socket *was* right: put the opening sigil back and
///    press again, which re-syncs the baseline the next delta is measured from.
/// 2. `aligned` rose, so this socket has arrived: move to the next.
/// 3. neither: turn it again.
///
/// The deltas come from the board, `Ward::press`'s arithmetic, never the prose
/// (rule 6).
fn press_one(sim: &mut Sim, sweep: &mut Sweep) {
    // No reading open — either the first lap, or the last one gave. `probe`
    // opens the next and presses it, which is how a bound spell laps.
    if sim.ward().is_none() {
        *sweep = Sweep::default();
        issue(sim, "probe");
        return;
    }

    // Unreachable — a socket's cyclic walk always arrives — but a spin here
    // would be silent, so it starts over rather than pressing a still aperture.
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
/// A policy is a thing a *player* types, so a rename breaking the player's
/// vocabulary would still compile against `tower::pylon::STATIONS` — `SOCKETS`
/// above, for the same reason.
const STATIONS: [&str; 3] = ["wellspring", "conduit", "barrier"];

/// How often the besieging driver puts more in the arsenal, in rounds.
///
/// Stores are a rate, so an arsenal has to be kept rather than filled. One
/// restock every four rounds — well inside `tower::WINDOW` at a siege's pace.
///
/// A player automating this would bind a brewing spell; this is the cheapest
/// thing that models one without dragging the laboratory into a bailey number.
const RESTOCK_EVERY: usize = 4;

/// One step of the circle's search, taken the way `taming` takes it.
///
/// The spell's odometer — widdershins, then sunwise, then the keystone — read
/// off `Sim::beast`. Stateless, or it measures a spell nobody can write.
fn tame_one(sim: &mut Sim) {
    if sim.beast().is_none() {
        issue(sim, "summon");
        return;
    }
    issue(sim, "limn widdershins");
    issue(sim, "summon");
    // Wrapped means back at the opening, which the spell's inner loop reaches on
    // its sixth step — the moment it falls through to the next `limn`. A beast
    // held on that call has no circle, so neither test passes and the next call
    // draws another.
    if wrapped(sim, Glyph::Widdershins) {
        issue(sim, "limn sunwise");
        if wrapped(sim, Glyph::Sunwise) {
            issue(sim, "limn keystone");
        }
    }
}

/// Whether a glyph has stepped back round to where every beast opens it, with a
/// beast still waiting.
///
/// The opening is read from the circle, not spelled out here: where a search
/// wraps is nothing a player types, and a local copy of `OPENING` could drift,
/// leaving the outer glyphs unstepped and the column silently measuring a search
/// that cannot hold most beasts.
fn wrapped(sim: &Sim, glyph: Glyph) -> bool {
    sim.beast()
        .is_some_and(|beast| beast.humour(glyph) == circle::OPENING[glyph.index()])
}

/// One turn of a siege, decided the way `besieging` decides one.
///
/// The same ladder, in the same order. A policy that played better than the
/// shipped spell would measure a player nobody is, and one that played worse
/// would blame the domain for the driver. §19's *"the harness has no player"*,
/// applied to a decision tree.
fn fight_one(sim: &mut Sim, state: &mut Fighting) {
    let Some(board) = sim.rampart() else {
        issue(sim, "defend");
        return;
    };
    // A finished siege leaves its board up as a postmortem, so *"there is a
    // board"* is not *"there is a fight"* — `defend` starts the next one.
    if board.enemy.troops == 0 || board.garrison.troops == 0 {
        // Back off, or §11.5's cadence fills `cost` with refusals; on its own
        // counter, so 64 means 64.
        state.idle = state.idle.wrapping_add(1);
        if state.idle.is_multiple_of(64) {
            issue(sim, "defend");
        } else {
            issue(sim, "survey rampart");
        }
        return;
    }
    // The dice first, the domain's central decision, by `steadfast`'s rule, and
    // only what it can pay for.
    if let Some((die, _)) = board
        .coffer
        .iter()
        .find(|(_, cost)| *cost <= board.quintessence)
    {
        let volley = board.intent == "volley";
        let area = match die.as_str() {
            // The d20 where a bad roll costs least — behind the wall when they
            // come hard, into succour when you cannot swing back anyway.
            "d20" if volley => "succour",
            "d20" => "buckler",
            "d8" => "buckler",
            _ => "succour",
        };
        issue(sim, &format!("pledge {die} to {area}"));
        return;
    }

    // One spend a round, marked even if refused, as the spell's `hold` outside
    // its ladder does. Keyed on `turns`, never a rendered line's length.
    let round = usize::try_from(board.turns).unwrap_or(0) + 1;

    // The top-up, outside the spend budget and unmarked (§19), by
    // `debug_spawn`, so the rate measures one domain.
    if state.restocked_on != round && round.is_multiple_of(RESTOCK_EVERY) {
        state.restocked_on = round;
        issue(sim, "debug_spawn warding 6");
        return;
    }

    if state.spent_on == round {
        issue(sim, "hold");
        return;
    }
    state.spent_on = round;

    // The ladder, as the spell runs it. No bodies rung: the tower levies its
    // standing garrison at `defend`. And a potion is a supply that draws on its
    // own once open, so the rung opens it only when it is not flowing (§19,
    // *Supply*).
    let flowing = sim
        .world()
        .resource::<orbs_sim::tower::supply::Supplies>()
        .get("warding")
        .is_some();
    if board.garrison.vigour * 2 <= board.garrison.full && !flowing {
        // `warding`, where the spell writes `mending`: `mending` is a
        // `secret = true` recipe, so naming it would measure a player further
        // through the game than this column is about.
        issue(sim, "quaff warding");
        return;
    }
    issue(sim, "hold");
}

/// One step of the forge's loop, reading the residue as the shipped table does.
///
/// What a spell reads, never the answer: `forging`'s eight rungs, held in step
/// by `the_shipped_table_is_the_answer_the_arithmetic_gives`.
fn bind_one(sim: &mut Sim) {
    // Wait when the pool is short rather than asking and being refused. Without
    // this the driver spent 6,236 of 7,200 ticks on *"hurried would take 6
    // quintessence, and you hold 4"*, measuring a game nobody plays. A player
    // short of quintessence waits for it; so does this.
    let held = sim
        .world()
        .resource::<orbs_sim::tower::Quintessence>()
        .get();
    let wanted = sim
        .world()
        .resource::<orbs_sim::content::Charms>()
        .cost(orbs_sim::tower::charm::Kind::Hurried, false);
    if sim.lattice().is_none() && held < wanted {
        issue(sim, "meditate 30");
        return;
    }

    let Some(board) = sim.lattice() else {
        // Nothing open, so open one. `hurried` every lap, so the column measures
        // one charm's economy rather than an average over five.
        issue(sim, "imbue mortar_and_pestle hurried");
        return;
    };
    // A fall in flight: let the slot run out rather than spinning. `issue`
    // advances the clock, which is what keeps `while sim.tick() < ticks` ending.
    let residue: Vec<bool> = board.residue.clone();
    let wanted: &[usize] = match (residue.first(), residue.get(1), residue.get(2)) {
        (Some(true), Some(true), Some(true)) => &[],
        (Some(true), Some(true), Some(false)) => &[1, 2],
        (Some(true), Some(false), Some(true)) => &[0, 1, 2],
        (Some(true), Some(false), Some(false)) => &[0],
        (Some(false), Some(true), Some(true)) => &[0, 1],
        (Some(false), Some(true), Some(false)) => &[0, 2],
        (Some(false), Some(false), Some(true)) => &[2],
        _ => &[1],
    };
    // Snap the columns the table names that are not already snapped, one a tick,
    // then fall. Asking the board rather than counting laps is what keeps this
    // correct when a fall springs back and the presses clear.
    for column in wanted {
        if !board.snapped.get(*column).copied().unwrap_or(false) {
            let word = orbs_sim::tower::lattice::COLUMNS[*column];
            issue(sim, &format!("snap {word}"));
            return;
        }
    }
    issue(sim, "anneal");
}

/// One turn of the cyclic solution, or a fresh course when none is drawn.
///
/// Three pairs in rotation. Between any two stations exactly one haul is legal,
/// so the policy never searches — it picks the pair whose turn it is and asks the
/// course which way round the haul runs, which is what `dev_spells.toml`'s
/// `holding` does with `let` and a part.
///
/// The sim's rotation and direction, via `between`, so the harness cannot
/// drift into a wrong-station solve. `STATIONS` is written out, as a player
/// types it.
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

    // `None` starts the rotation over rather than returning. Both stations empty
    // is unreachable in phase with the board — but `run` only advances the clock
    // inside `issue`, so a bare `return` is a spin with no tick and
    // `while sim.tick() < ticks` never ends. `press_one` guards the same hang.
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
        // The gap `--why` was built to close and could not see. Every parser
        // outcome is `Role::Normal` — `parser/report.rs` reserves the triad for
        // danger, cost and success — so an unresolved command fell into neither
        // column, and *"read the `cost` column before the rate"* read a number
        // that excluded it.
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
        // `landed` counted every `Role::Success` record, so a sim that had issued
        // no commands still reported finished runs — against a field documented
        // as *"the work the rate is made of"*.
        let sim = Sim::new(0);
        let (_, landed, _) = tally(&sim);
        assert_eq!(landed, 0, "the tower reported finished work at tick zero");
    }
}
