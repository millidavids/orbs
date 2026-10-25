//! The player's side of the world: what they typed, what the orb said back, and
//! what is waiting for the next tick.
//!
//! The scrollback is here, not in a frontend, because it is the log (§3): the
//! pane, `peruse`, a pipe and the harness read one stream.
//!
//! Two clocks: [`Sim::submit`](crate::Sim::submit) resolves and echoes now, at
//! 1 Hz a second's delay would read as broken; the [`Intent`] runs on the next
//! tick, or replay stops holding. Bevy runs the fixed loop first in a frame, so
//! a line always resolves against the last completed tick, and
//! [`Submissions`] records the pairing.

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
    pub const fn records(&self) -> &Records {
        &self.0
    }

    /// The records, for a command to write into.
    pub const fn records_mut(&mut self) -> &mut Records {
        &mut self.0
    }
}

/// Something the player asked for, waiting for the tick that does it.
#[derive(Debug, Clone)]
pub enum Queued {
    /// A resolved command.
    Command(Intent),
    /// A resolved command the augury read rather than the orb (§6) — see
    /// [`Sim::submit_divined`](crate::Sim::submit_divined).
    ///
    /// Its own variant, since a verb may answer a guess differently — `summon`
    /// will not spend a call on one.
    Divined(Intent),
    /// A spell to write out — see [`Sim::write_spell`](crate::Sim::write_spell).
    ///
    /// In the commands' queue, so a save and a typed line keep their order.
    Write {
        /// The spell's filename, extension included.
        name: String,
        /// What the buffer held, before canonicalisation.
        lines: Vec<String>,
        /// The same lines as the orb read them, which is what compiles.
        ///
        /// Equal to `lines` when no reader was consulted, which is every build
        /// without one and every line a reader abstained on.
        read: Vec<String>,
        /// Which reader made `read` — see `tower::Read::by`.
        by: u64,
    },
    /// A mastery node the player chose — see [`Sim::take`](crate::Sim::take).
    ///
    /// On this queue for [`Write`](Self::Write)'s reason.
    Take(String),
    /// A tester asking for reagents — see [`execute::debug`](crate::execute).
    ///
    /// On the same queue, so a tester's state replays from `(seed,
    /// submissions)`.
    #[cfg(debug_assertions)]
    Spawn(crate::execute::SpawnOrder),
}

/// Commands resolved but not yet run.
///
/// Drained by [`run_pending`](crate::execute::run_pending) at the start of
/// every tick, so typing and doing sit either side of a tick boundary.
#[derive(Resource, Debug, Default)]
pub struct Pending(Vec<Queued>);

impl Pending {
    /// Queue an intent for the next tick.
    pub fn push(&mut self, intent: Intent) {
        self.0.push(Queued::Command(intent));
    }

    /// Queue an intent the augury read, for the next tick.
    pub fn push_divined(&mut self, intent: Intent) {
        self.0.push(Queued::Divined(intent));
    }

    /// Queue a spell to be written on the next tick.
    pub fn write(&mut self, name: String, lines: Vec<String>, read: Vec<String>, by: u64) {
        self.0.push(Queued::Write {
            name,
            lines,
            read,
            by,
        });
    }

    /// The reading the latest queued write of `name` carries, if one is waiting.
    ///
    /// Newer than the node's, since a write lands on the next tick.
    #[must_use]
    pub fn written(&self, name: &str) -> Option<crate::tower::Read> {
        self.0.iter().rev().find_map(|queued| match queued {
            Queued::Write {
                name: queued,
                lines,
                read,
                by,
            } if queued == name => Some(crate::tower::Read::new(lines, read.clone(), Some(*by))),
            _ => None,
        })
    }

    /// Queue a mastery node to be taken on the next tick.
    pub fn take_node(&mut self, id: String) {
        self.0.push(Queued::Take(id));
    }

    /// Queue a tester's spawn for the next tick.
    #[cfg(debug_assertions)]
    pub fn spawn(&mut self, order: crate::execute::SpawnOrder) {
        self.0.push(Queued::Spawn(order));
    }

    /// How many commands are waiting.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether nothing is waiting.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Take everything waiting.
    pub fn drain(&mut self) -> Vec<Queued> {
        std::mem::take(&mut self.0)
    }
}

/// Who is at the orb.
///
/// World state, not a display setting: the player *is* the wizard (§4). So it
/// travels in a save, which outranks the environment a new world is seeded
/// from. Nothing in the sim reads it, so it cannot make two runs diverge.
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
/// Not a placeholder for lore: §4 keeps the wizard unnamed.
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

/// The readings the orb is waiting for the player to pick between.
///
/// §6's numbered prompt, answered with a digit. Emptied the moment anything
/// else is typed, since §6 forbids a modal prompt.
#[derive(Resource, Debug, Default)]
pub struct Choices {
    readings: Vec<Intent>,
    /// The line that raised the question, so a save can ask it again.
    ///
    /// The line, not the readings, which would pin the format to the parser's
    /// types. `analyse` is pure (§19), so asking again gives the same list.
    asked: String,
}

impl Choices {
    /// Offer these readings, best first, and remember what was asked.
    pub fn offer(&mut self, line: &str, readings: Vec<Intent>) {
        self.readings = readings;
        self.asked = line.to_owned();
    }

    /// The line that raised the question, if one is open.
    #[must_use]
    pub fn asked(&self) -> Option<&str> {
        (!self.readings.is_empty()).then_some(self.asked.as_str())
    }

    /// Forget the question.
    pub fn clear(&mut self) {
        self.readings.clear();
        self.asked.clear();
    }

    /// How many are on offer.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.readings.len()
    }

    /// Whether the orb is waiting on an answer.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.readings.is_empty()
    }

    /// The reading `choice` names, counting from **one** as the prompt shows.
    #[must_use]
    pub fn pick(&self, choice: usize) -> Option<&Intent> {
        self.readings.get(choice.checked_sub(1)?)
    }
}

/// Ticks a command asked the world to pass through.
///
/// `meditate` runs inside a tick, so it leaves a request that
/// [`Sim::step`](crate::Sim::step) runs out — inside `step`, so every frontend
/// passes time alike.
#[derive(Resource, Debug, Default)]
pub struct Skip(u64);

impl Skip {
    /// Ask for `ticks` more.
    pub const fn request(&mut self, ticks: u64) {
        self.0 = self.0.saturating_add(ticks);
    }

    /// Take one, if any are owed.
    pub(crate) const fn take(&mut self) -> bool {
        if self.0 == 0 {
            return false;
        }
        self.0 -= 1;
        true
    }

    /// How many ticks are still owed.
    #[must_use]
    pub const fn owed(&self) -> u64 {
        self.0
    }
}

/// One thing the player did that the world has to be able to be told again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Submission {
    /// A line typed at the prompt.
    Typed(String),
    /// A line the augury worked out rather than the orb reading it (§6).
    ///
    /// The canonical form travels: a model need not read a line alike on
    /// another GPU, so it runs once and the echo is the determinism boundary.
    Divined {
        /// Exactly what the player typed. Kept for the transcript and the trace,
        /// never re-read.
        line: String,
        /// The canonical command the augury settled on — what actually replays.
        echo: String,
    },
    /// A spell saved out of the editor.
    ///
    /// The whole text, not the keystrokes, which are no decision and would
    /// couple replay to the editor. One per `:w`.
    Wrote {
        /// The spell's filename.
        name: String,
        /// Its lines, exactly as the buffer held them — *before*
        /// canonicalisation, so replaying re-derives the same canonical form
        /// rather than trusting one recorded alongside it.
        lines: Vec<String>,
        /// The same lines as the orb *read* them, carried rather than
        /// re-derived.
        ///
        /// Carried for `Divined`'s reason: a trained reader may not read alike
        /// elsewhere. Equal to `lines` when nothing read them.
        read: Vec<String>,
        /// Which reader made `read`, so a replayed spell is kept for exactly the
        /// reader the live one was — see `tower::Read::by`.
        by: u64,
    },
    /// One cell of the archive's stacks, walked by hand (§10, §19).
    ///
    /// Not a [`Typed`](Self::Typed) `follow`: it ran at once, after its tick's
    /// step, where a typed line runs at the next. See `Sim::replay`.
    Walked(String),
    /// A mastery node taken on the weave screen.
    ///
    /// The id, not the keystrokes — [`Wrote`](Self::Wrote)'s argument. Runs at
    /// the next tick, like a typed line.
    Took(String),
}

/// Everything the player did, with the tick it landed on.
///
/// A replay needs exactly `(seed, submissions)`; the tick a line landed on
/// cannot be inferred afterwards.
#[derive(Resource, Debug, Default)]
pub struct Submissions(Vec<(Tick, Submission)>);

impl Submissions {
    /// Note that `line` was submitted during `tick`.
    pub fn push(&mut self, tick: Tick, line: &str) {
        self.0.push((tick, Submission::Typed(line.to_owned())));
    }

    /// Note that the augury read `line` as `echo` during `tick`.
    pub fn divined(&mut self, tick: Tick, line: &str, echo: &str) {
        self.0.push((
            tick,
            Submission::Divined {
                line: line.to_owned(),
                echo: echo.to_owned(),
            },
        ));
    }

    /// Note that a spell was saved during `tick`.
    pub fn wrote(&mut self, tick: Tick, name: &str, lines: &[String], read: &[String], by: u64) {
        self.0.push((
            tick,
            Submission::Wrote {
                name: name.to_owned(),
                lines: lines.to_vec(),
                read: read.to_vec(),
                by,
            },
        ));
    }

    /// Note that a cell was walked by hand during `tick`.
    ///
    /// Unlike [`push`](Self::push) this one has *already happened* by the time
    /// it is recorded. See [`Submission::Walked`].
    pub fn walked(&mut self, tick: Tick, way: &str) {
        self.0.push((tick, Submission::Walked(way.to_owned())));
    }

    /// Note that a mastery node was taken during `tick`.
    ///
    /// Queued rather than already done, like [`wrote`](Self::wrote) — see
    /// [`Submission::Took`].
    pub fn took(&mut self, tick: Tick, id: &str) {
        self.0.push((tick, Submission::Took(id.to_owned())));
    }

    /// Everything, in order.
    #[must_use]
    pub fn all(&self) -> &[(Tick, Submission)] {
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
        sim.submit("attend laboratory");
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
        // here would draw and speak `attend /tower/laboratory 1`.
        assert_eq!(completion.field(FieldName::Tick), None);
        assert_eq!(completion.to_speech(), "attend, /tower/laboratory");
    }

    #[test]
    fn an_unresolved_line_queues_nothing() {
        let mut sim = Sim::new(1);
        // The orb boots with a report (§4), so the transcript never starts
        // empty — measure from where this line begins rather than from zero.
        let before = sim.scrollback().records().len();
        sim.submit("xyzzy");

        assert!(sim.pending().is_empty());
        assert_eq!(
            sim.scrollback()
                .records()
                .get(before + 1)
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
        // Against the boot report, not against nothing (§4).
        let before = sim.scrollback().records().len();
        for line in ["", "   ", "\t"] {
            sim.submit(line);
        }
        assert_eq!(
            sim.scrollback().records().len(),
            before,
            "a blank line spoke"
        );
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
                (Tick::new(0), Submission::Typed("look around".to_owned())),
                (Tick::new(3), Submission::Typed("survey".to_owned())),
            ],
        );
    }

    #[test]
    fn saving_a_spell_is_one_submission_carrying_the_whole_buffer() {
        // The editor's replay contract: the save enters, once, with the typed
        // lines, never the keystrokes.
        let mut sim = Sim::new(1);
        sim.step_n(2);
        sim.write_spell("morning", &["make a potion of clarity".to_owned()]);

        assert_eq!(
            sim.submissions().all(),
            [(
                Tick::new(2),
                Submission::Wrote {
                    name: "morning.spell".to_owned(),
                    lines: vec!["make a potion of clarity".to_owned()],
                    // Equal to `lines`, and that is the assertion: nothing read
                    // this spell, so the reading is the text — the identity
                    // case every build without a reader is in.
                    read: vec!["make a potion of clarity".to_owned()],
                    by: crate::augur::Verbatim::IDENTITY,
                },
            )],
        );
    }

    #[test]
    fn a_written_spell_replays_from_seed_and_submissions() {
        // A `Wrote` entry nothing replays is a shape with no consumer. See
        // `Sim::replay`.
        let mut live = Sim::new(9);
        live.submit("attend laboratory");
        live.step();
        live.write_spell("morning", &["make a potion of clarity".to_owned()]);
        live.step();

        let mut replayed = Sim::new(9);
        for (tick, submission) in live.submissions().all().to_vec() {
            while replayed.tick() < tick {
                replayed.step();
            }
            replayed.replay(submission);
        }
        while replayed.tick() < live.tick() {
            replayed.step();
        }

        let spell_of = |sim: &Sim| -> Option<Vec<String>> { sim.spell("morning") };
        assert_eq!(spell_of(&live), spell_of(&replayed));
        assert!(spell_of(&live).is_some(), "the spell was never written");

        // And what it compiles from, which `Held` alone cannot show: the
        // reading travels too.
        let reading_of = |sim: &Sim| -> Option<Vec<String>> {
            let world = sim.world();
            world
                .iter_entities()
                .find(|entity| {
                    entity
                        .get::<crate::tower::Name>()
                        .is_some_and(|name| name.0 == crate::content::with_extension("morning"))
                })
                .map(|entity| crate::tower::spell::source(world, entity.id()))
        };
        assert_eq!(
            reading_of(&live),
            reading_of(&replayed),
            "the replayed spell compiles from different lines",
        );
    }

    /// A way that is actually open from where the reading stands.
    fn a_way_out(maze: &orbs_render::Stacks) -> Option<crate::tower::Way> {
        crate::tower::Way::ALL
            .into_iter()
            .enumerate()
            .find_map(|(index, way)| maze.open(index).then_some(way))
    }

    #[test]
    fn stacks_walked_by_hand_replay_to_the_same_cell() {
        // The claim `Sim::walk` makes, tested rather than argued. It is the
        // third entry point and the only one that does not go through the tick,
        // so it is the one that could quietly put a replay a step out — and a
        // maze is the ideal witness, because being one cell wrong is visible.
        let mut live = Sim::new(4);
        live.submit("attend archive");
        live.step();
        live.submit("research");
        live.step();

        // Several presses inside one tick, then a tick, then more — the shape a
        // player actually produces, and the shape a per-tick queue could not.
        for round in 0..3 {
            for _ in 0..4 {
                let Some(maze) = live.stacks() else { break };
                let Some(way) = a_way_out(&maze) else { break };
                live.walk(way);
            }
            let _ = round;
            live.step();
        }

        let mut replayed = Sim::new(4);
        for (tick, submission) in live.submissions().all().to_vec() {
            while replayed.tick() < tick {
                replayed.step();
            }
            replayed.replay(submission);
        }
        while replayed.tick() < live.tick() {
            replayed.step();
        }

        let at = |sim: &Sim| sim.stacks().map(|maze| (maze.at, maze.explored()));
        assert!(at(&live).is_some(), "the walk never opened a maze");
        assert_eq!(at(&live), at(&replayed), "the replay walked somewhere else");
    }

    #[test]
    fn walking_by_hand_costs_no_world_time() {
        // The other half of what the third entry point promises: a player
        // standing in a maze is not a player whose brews are running down. If
        // this ever ticks, `wander` has quietly become a way to pass time.
        let mut sim = Sim::new(4);
        sim.submit("attend archive");
        sim.step();
        sim.submit("research");
        sim.step();

        let before = sim.tick();
        for _ in 0..8 {
            let Some(maze) = sim.stacks() else { break };
            let Some(way) = a_way_out(&maze) else { break };
            sim.walk(way);
        }
        assert_eq!(sim.tick(), before, "walking advanced the world clock");
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

    /// Ask something the orb cannot settle, in the archive where the fragments are.
    ///
    /// `move`'s first slot is required, which is what makes the prompt appear.
    fn asked(seed: u64) -> Sim {
        let mut sim = Sim::new(seed);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("move nonsense");
        sim.step();
        sim
    }

    #[test]
    fn a_number_answers_the_numbered_prompt() {
        // §6 numbers the tied readings and the player answers with a digit.
        // Without this the prompt is rhetorical: the digit resolves against the
        // verb vocabulary as a miss and the player is stuck.
        let mut sim = asked(1);
        assert_eq!(sim.choices().len(), 3, "the orb asked nothing");

        sim.submit("2");
        assert_eq!(sim.pending().len(), 1, "the answer ran nothing");
        assert!(sim.choices().is_empty(), "the question outlived its answer");

        sim.step();
        assert!(
            messages(&sim).iter().any(|line| line == "move rock-salt"),
            "{:?}",
            messages(&sim),
        );
    }

    #[test]
    fn a_number_outside_the_list_leaves_the_question_standing() {
        // §15's gate weighs "zero dead ends" above the raw resolution rate, so a
        // mistyped answer must not throw the question away with it.
        let mut sim = asked(1);
        sim.submit("9");
        sim.step();

        assert!(sim.pending().is_empty(), "an unlisted number ran something");
        assert_eq!(sim.choices().len(), 3, "the question was dropped");

        sim.submit("1");
        sim.step();
        assert!(messages(&sim).iter().any(|line| line.starts_with("move ")));
    }

    #[test]
    fn asking_something_else_walks_away_from_the_question() {
        // §6 forbids a modal prompt, so leaving one unanswered costs nothing.
        let mut sim = asked(1);
        sim.submit("look around");
        assert!(sim.choices().is_empty(), "the prompt was modal");
    }

    #[test]
    fn a_digit_is_only_an_answer_when_something_was_asked() {
        // No verb takes a bare number as its whole input, so there is nothing to
        // collide with — but a digit typed out of the blue must still go through
        // the parser rather than being swallowed.
        let mut sim = Sim::new(1);
        sim.submit("7");
        sim.step();
        assert!(sim.pending().is_empty());
        assert_eq!(
            sim.parse_log().records().len(),
            1,
            "the digit went untraced"
        );
    }

    #[test]
    fn answering_is_not_counted_as_a_phrasing() {
        // A digit is not a test of the parser. Counting it would dilute §15's
        // first metric with inputs that were never phrasing attempts; what the
        // gate wants is the original ambiguous line reaching its intended
        // action, and the selection is the evidence for that.
        let mut sim = asked(1);
        let before = sim.parse_log().records().len();
        sim.submit("2");
        assert_eq!(sim.parse_log().records().len(), before);
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
