//! The player's side of the world: what they typed, what the orb said back, and
//! what is waiting for the next tick.
//!
//! # Why the scrollback lives here and not in a frontend
//!
//! It is not a display buffer. DESIGN.md §3 forbids unlogged output, so this
//! stream **is** the log — the scrollback, the file a player `peruse`s, the
//! source a pipe stage reads, and the balance harness's transcript are one
//! stream read four ways. A frontend-owned scrollback would mean the first
//! domain has to move it, and until it did there would be two streams that could
//! disagree about what the orb had said. That is the mistake the record model
//! exists to prevent, one layer up.
//!
//! # The two clocks
//!
//! [`Sim::submit`](crate::Sim::submit) resolves **immediately**; the resulting
//! [`Intent`] runs on the **next tick**.
//!
//! Both halves are load-bearing. Ticks are 1 Hz (§5.0), so resolving inside
//! `step()` would put up to a full second between pressing Enter and seeing the
//! echo — and a terminal that takes a second to answer reads as broken, while §6
//! makes the echo the mechanism players learn the canonical vocabulary from.
//! Effects, meanwhile, must land on a tick boundary or replay and offline/online
//! parity stop holding.
//!
//! Determinism survives because a frontend calls `submit` from its own update
//! and `step` from its fixed update, and Bevy runs the fixed loop *first* in a
//! frame: a line submitted in frame F always resolves against world state as of
//! the last completed tick in F. [`Submissions`] records the pairing, which is
//! all a replay needs — and is also what §6's command-anchored `undo` will want.

use bevy_ecs::prelude::*;
use orbs_render::Records;

use crate::parser::Intent;
use crate::tick::Tick;

/// Everything the player has said and been told.
#[derive(Resource, Debug, Default)]
pub struct Scrollback(Records);

impl Scrollback {
    /// The records, for a view to draw or a pipe stage to filter.
    #[must_use]
    pub fn records(&self) -> &Records {
        &self.0
    }

    /// The records, for a command to write into.
    pub fn records_mut(&mut self) -> &mut Records {
        &mut self.0
    }
}

/// Commands resolved but not yet run.
///
/// Drained by [`run_pending`](crate::execute::run_pending) at the start of every tick, which is what keeps
/// "the player typed it" and "the world did it" on opposite sides of a tick
/// boundary.
#[derive(Resource, Debug, Default)]
pub struct Pending(Vec<Intent>);

impl Pending {
    /// Queue an intent for the next tick.
    pub fn push(&mut self, intent: Intent) {
        self.0.push(intent);
    }

    /// How many commands are waiting.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether nothing is waiting.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Take everything waiting.
    pub fn drain(&mut self) -> Vec<Intent> {
        std::mem::take(&mut self.0)
    }
}

/// Who is at the orb.
///
/// §4's framing is *"always inside"* — the player never sees the wizard, because
/// the player **is** the wizard. So the name at the prompt is world state rather
/// than a display setting: it belongs in a save, it is the same in every
/// frontend, and it is one of the few places the game says the player's own word
/// back to them.
///
/// Lives here rather than in `orbs-render` because a name is content, and
/// `orbs-render` decides *where* things appear rather than what they are called.
///
/// # It is world state, so a save outranks the environment
///
/// A frontend seeds this from the environment when it builds a **new** world.
/// That is the only moment it may: once a save exists it carries its own name,
/// and loading it must overwrite this rather than have the current machine's
/// login quietly rename someone else's wizard. Nothing in the sim reads the name
/// — it feeds the prompt and nothing else — so it cannot make two runs from one
/// seed diverge, and this note is here to keep it that way.
#[derive(Resource, Debug, Clone)]
pub struct Wizard {
    name: String,
}

impl Default for Wizard {
    fn default() -> Self {
        Self {
            name: DEFAULT_WIZARD.to_owned(),
        }
    }
}

/// The name the orb answers to before anyone has given it another.
///
/// Not a placeholder to be replaced by lore: §4 keeps the wizard unnamed and
/// unseen, so until a player says otherwise the machine's own name is the honest
/// thing to show.
pub const DEFAULT_WIZARD: &str = "orbs";

impl Wizard {
    /// The name as it appears at the prompt.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Rename the wizard.
    ///
    /// Blank names are refused rather than accepted and rendered as an empty
    /// prompt — a prompt with nothing in front of the `$` reads as a bug.
    pub fn rename(&mut self, name: &str) {
        let name = name.trim();
        if !name.is_empty() {
            self.name = name.to_owned();
        }
    }
}

/// Ticks a command asked the world to pass through.
///
/// `meditate 30` cannot advance the clock where it runs — it *is* running inside
/// a tick — so it leaves a request here and [`Sim::step`](crate::Sim::step) runs
/// it out before returning. Keeping the loop inside `step` rather than in the
/// caller is what makes the Bevy build, the terminal build and the balance
/// harness pass time identically.
#[derive(Resource, Debug, Default)]
pub struct Skip(u64);

impl Skip {
    /// Ask for `ticks` more.
    pub fn request(&mut self, ticks: u64) {
        self.0 = self.0.saturating_add(ticks);
    }

    /// Take one, if any are owed.
    pub(crate) fn take(&mut self) -> bool {
        if self.0 == 0 {
            return false;
        }
        self.0 -= 1;
        true
    }

    /// How many ticks are still owed.
    #[must_use]
    pub fn owed(&self) -> u64 {
        self.0
    }
}

/// Every line the player submitted, with the tick it landed on.
///
/// A replay needs exactly `(seed, submissions)` and nothing else. Recorded from
/// the start even though nothing consumes it yet, because the pairing is
/// unrecoverable after the fact — the tick a line landed on cannot be inferred
/// from the line.
#[derive(Resource, Debug, Default)]
pub struct Submissions(Vec<(Tick, String)>);

impl Submissions {
    /// Note that `line` was submitted during `tick`.
    pub fn push(&mut self, tick: Tick, line: &str) {
        self.0.push((tick, line.to_owned()));
    }

    /// Every submission, in order.
    #[must_use]
    pub fn all(&self) -> &[(Tick, String)] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;
    use orbs_render::{FieldName, Outcome, RecordKind, Value};

    fn messages(sim: &Sim) -> Vec<String> {
        sim.scrollback()
            .records()
            .iter()
            .filter_map(|record| record.field(FieldName::Message))
            .map(|value| value.with_str(str::to_owned))
            .collect()
    }

    #[test]
    fn a_submitted_line_is_echoed_before_any_tick_runs() {
        // §6's echo is the teaching mechanism. Waiting a tick for it would put a
        // full second between Enter and the answer.
        let mut sim = Sim::new(1);
        sim.submit("look around");

        assert_eq!(sim.tick(), Tick::new(0), "submitting must not advance time");
        assert_eq!(messages(&sim), ["look around", "survey"]);
    }

    #[test]
    fn the_effect_waits_for_the_tick_boundary() {
        // `attend` rather than `survey`: a listing verb now emits `Entry` rows
        // for what it found, and a completion is what a verb that *changed*
        // something reports.
        let mut sim = Sim::new(1);
        sim.submit("attend alembic");
        assert_eq!(sim.pending().len(), 1);
        assert!(
            sim.scrollback()
                .records()
                .iter()
                .all(|record| record.kind() != RecordKind::Completion),
            "the effect landed before its tick",
        );

        sim.step();
        assert!(sim.pending().is_empty(), "the queue must drain");

        let completion = sim
            .scrollback()
            .records()
            .iter()
            .find(|record| record.kind() == RecordKind::Completion)
            .expect("a completion");
        assert_eq!(
            completion.field(FieldName::Name),
            Some(Value::Text("attend"))
        );
        // No bare timestamp: a line view joins every content value, so a tick
        // here would draw and speak `attend /tower/alembic 1`.
        assert_eq!(completion.field(FieldName::Tick), None);
        assert_eq!(completion.to_speech(), "attend, /tower/alembic");
    }

    #[test]
    fn an_unresolved_line_queues_nothing() {
        let mut sim = Sim::new(1);
        sim.submit("xyzzy");

        assert!(sim.pending().is_empty());
        assert_eq!(
            sim.scrollback()
                .records()
                .get(1)
                .expect("the parser's answer")
                .outcome(),
            Some(Outcome::Unresolved),
        );
    }

    #[test]
    fn a_blank_line_does_nothing_at_all() {
        // Every real shell redraws the prompt. `report` correctly refuses to be
        // silent at its own layer, so the filtering belongs here.
        let mut sim = Sim::new(1);
        for line in ["", "   ", "\t"] {
            sim.submit(line);
        }
        assert!(sim.scrollback().records().is_empty());
        assert!(sim.submissions().all().is_empty());
    }

    #[test]
    fn every_submission_is_paired_with_the_tick_it_landed_on() {
        // A replay needs `(seed, submissions)` and nothing else.
        let mut sim = Sim::new(1);
        sim.submit("look around");
        sim.step_n(3);
        sim.submit("survey");

        assert_eq!(
            sim.submissions().all(),
            [
                (Tick::new(0), "look around".to_owned()),
                (Tick::new(3), "survey".to_owned()),
            ],
        );
    }

    #[test]
    fn the_same_seed_and_the_same_typing_produce_the_same_world() {
        // The property the whole architecture rests on, now exercised through
        // the path a player actually drives.
        let transcript = ["look around", "meditate", "xyzzy", "look around"];
        let mut a = Sim::new(0xC0FFEE);
        let mut b = Sim::new(0xC0FFEE);
        for sim in [&mut a, &mut b] {
            for line in transcript {
                sim.submit(line);
                sim.step();
            }
        }
        assert_eq!(messages(&a), messages(&b));
        assert_eq!(a.submissions().all(), b.submissions().all());
    }

    #[test]
    fn every_line_typed_is_traced_with_the_readings_that_lost() {
        // §6: "the parser must explain itself." The Phase 0 gate acts on the
        // *clustering* of failures by cause, so knowing a line failed is not
        // enough — the losing candidates are what say whether the miss was the
        // verb or the argument, and they are unrecoverable after the fact.
        let mut sim = Sim::new(1);
        for line in ["look around", "meditate", "xyzzy"] {
            sim.submit(line);
            sim.step();
        }

        let log = sim.parse_log();
        assert_eq!(log.records().len(), 3, "a line went untraced");
        assert_eq!(log.resolved(), 1);
        assert_eq!(log.incomplete(), 1);
        assert_eq!(log.unresolved(), 1);

        // The scores are the point. An aggregate pass rate cannot say whether a
        // miss was the verb or the argument; these columns can, which is what
        // makes the gate's clustering requirement actionable.
        let tsv = log.to_tsv();
        let header = tsv.lines().next().expect("a header");
        for column in ["verb_score", "arg_score", "suggestions", "outcome"] {
            assert!(header.contains(column), "{column} missing from {header}");
        }

        let resolved = tsv
            .lines()
            .find(|line| line.contains("look around"))
            .expect("the resolved line");
        let fields: Vec<&str> = resolved.split('\t').collect();
        let scores = header.split('\t').zip(&fields);
        for (column, value) in scores {
            if column == "verb_score" || column == "arg_score" {
                assert!(
                    value.parse::<u32>().is_ok_and(|score| score > 0),
                    "{column} was {value:?}",
                );
            }
        }
    }

    #[test]
    fn a_blank_line_is_not_traced_either() {
        // It is not an input the gate should count against the parser.
        let mut sim = Sim::new(1);
        sim.submit("   ");
        assert!(sim.parse_log().records().is_empty());
    }
}
