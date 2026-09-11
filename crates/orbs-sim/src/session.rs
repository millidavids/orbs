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
    /// A spell to write out — see [`Sim::write_spell`](crate::Sim::write_spell).
    ///
    /// **In the same queue as commands, deliberately.** A save and a typed line
    /// both land on the next tick, and two queues would mean an ordering between
    /// them that nothing states — so `scribe morning` followed immediately by a
    /// save could apply in either order.
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
    /// On this queue for [`Write`](Self::Write)'s reason: a choice made on a
    /// screen and a line typed at the prompt both land on the next tick, and two
    /// queues would leave an ordering between them that nothing states.
    Take(String),
    /// A tester asking for reagents — see [`execute::debug`](crate::execute).
    ///
    /// **On the same queue, and that is the point.** A debug tool that mutated
    /// the world from inside an input call would land off a tick boundary, which
    /// is the one thing `session` is explicit that nothing may do: the state it
    /// produced could not be reproduced from `(seed, submissions)`, and the tool
    /// meant to help find bugs would be a source of them.
    #[cfg(debug_assertions)]
    Spawn(crate::execute::SpawnOrder),
}

/// Commands resolved but not yet run.
///
/// Drained by [`run_pending`](crate::execute::run_pending) at the start of every tick, which is what keeps
/// "the player typed it" and "the world did it" on opposite sides of a tick
/// boundary.
#[derive(Resource, Debug, Default)]
pub struct Pending(Vec<Queued>);

impl Pending {
    /// Queue an intent for the next tick.
    pub fn push(&mut self, intent: Intent) {
        self.0.push(Queued::Command(intent));
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
    /// **Newer than the node's.** A write lands on the next tick, so on the beat
    /// the editor saves and then reads its buffer, this is the reading that is
    /// about to compile and the node's is the one before it.
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

/// The readings the orb is waiting for the player to pick between.
///
/// §6: when several readings score alike in calm mode the orb asks, numbering
/// them, and the player answers with a digit. Without somewhere to hold the list
/// the question is rhetorical — the prompt appears, the digit resolves against
/// the verb vocabulary as a miss, and the player is in a **dead end**. §15's gate
/// calls the dead-end metric more important than the raw resolution rate.
///
/// Emptied the moment anything else is typed: §6 forbids a modal prompt, so
/// walking away from the question by asking a different one has to be free.
#[derive(Resource, Debug, Default)]
pub struct Choices {
    readings: Vec<Intent>,
    /// The line that raised the question, so a save can ask it again.
    ///
    /// **The line, not the readings.** An [`Intent`] is the parser's resolved
    /// form with typed arguments, and putting it in a save would drag the whole
    /// parser type surface into the format and pin it against every future
    /// parser change — for a question that survives until the next command.
    ///
    /// The line costs one string and reproduces the readings exactly, because
    /// §19 settled that the parser's tie-break uses **no randomness**: ranking
    /// is a total order over score, position in `Verb::ALL`, and the canonical
    /// echo. `analyse` is a pure function of the line and the scene, so asking
    /// it again on the way back in gives the same numbered list the player was
    /// looking at.
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
/// `meditate 30` cannot advance the clock where it runs — it *is* running inside
/// a tick — so it leaves a request here and [`Sim::step`](crate::Sim::step) runs
/// it out before returning. Keeping the loop inside `step` rather than in the
/// caller is what makes the Bevy build, the terminal build and the balance
/// harness pass time identically.
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
    /// # Why the canonical form travels, when [`Wrote`](Self::Wrote)'s does not
    ///
    /// `Wrote` records the buffer *before* canonicalisation on the explicit
    /// ground that replaying should re-derive the canonical form rather than
    /// trust one recorded beside it. That is right there and wrong here, and the
    /// difference is the whole reason this variant exists.
    ///
    /// Re-deriving is safe when the derivation is `analyse`, which is pure,
    /// integer-scored and has no RNG. It is **not** safe when a trained model
    /// did the reading: the same line on a different GPU, a different driver or
    /// a different backend need not produce the same spans, so a replay that
    /// re-read the line could diverge from the session it claims to reproduce —
    /// silently, and only on someone else's machine.
    ///
    /// So the model runs exactly once, at the moment the player was there to see
    /// the echo, and what replays is what they saw. **The augury is outside the
    /// determinism boundary; the echo is the boundary.**
    Divined {
        /// Exactly what the player typed. Kept for the transcript and the trace,
        /// never re-read.
        line: String,
        /// The canonical command the augury settled on — what actually replays.
        echo: String,
    },
    /// A spell saved out of the editor.
    ///
    /// # Why the whole text, and not the keystrokes
    ///
    /// Editing is the first thing in the game that takes input without every
    /// keypress being a decision. A cursor moving left reaches nothing, changes
    /// no state the world can see, and must not enter a replay — recording it
    /// would bloat the log by orders of magnitude *and* couple replay to editor
    /// internals, so changing how `Home` behaves would break every saved
    /// session.
    ///
    /// The save is the decision, so the save is the entry. One per `:w`,
    /// carrying what the buffer held.
    Wrote {
        /// The spell's filename.
        name: String,
        /// Its lines, exactly as the buffer held them — **before**
        /// canonicalisation, so replaying re-derives the same canonical form
        /// rather than trusting one recorded alongside it.
        lines: Vec<String>,
        /// The same lines as the orb *read* them, carried rather than re-derived.
        ///
        /// **The exception the doc above describes, and the reason it is one.**
        /// Re-deriving is safe while the derivation is `analyse`, which is
        /// deterministic; it is not safe once a trained reader did part of it,
        /// because a replay on a machine with no weights — or with different
        /// ones — would build a different program from the same submissions.
        /// `Submission::Divined` carries its echo for exactly this reason.
        ///
        /// Equal to `lines` when nothing read them.
        read: Vec<String>,
        /// Which reader made `read`, so a replayed spell is kept for exactly the
        /// reader the live one was — see `tower::Read::by`.
        by: u64,
    },
    /// One cell of the archive's stacks, walked by hand (§10, §19).
    ///
    /// # Why this is not a [`Typed`](Self::Typed) `follow east`
    ///
    /// Because the two run at different moments, and a replay that could not
    /// tell them apart would put this one a tick out. A typed line is *queued*
    /// and executes at the start of the next tick; an arrow in `wander` mode
    /// executes **immediately**, so it lands after the step of the tick it is
    /// recorded against rather than before the next one.
    ///
    /// That is the whole of the difference and it is recoverable from the
    /// variant alone: replay a tick, then apply the walks recorded against it,
    /// in list order. Nothing else about the pairing is ambiguous, because the
    /// prompt is dead while the arrows have the maze — so a tick can never
    /// contain both a walk and a typed line.
    Walked(String),
    /// A mastery node taken on the weave screen.
    ///
    /// # Why the id and not the keystrokes
    ///
    /// [`Wrote`](Self::Wrote)'s argument, one screen along: aiming the cursor
    /// with the arrows reaches nothing and changes no state the world can see,
    /// so recording it would bloat the log and couple replay to the screen's
    /// internals. **The take is the decision, so the take is the entry** — and
    /// the id is what the world stores, so nothing is re-derived on the way in.
    ///
    /// It executes at the start of the next tick, like a typed line: the screen
    /// hands the sim a request and the world answers on its own clock.
    Took(String),
    /// A syllable sung by hand, on the arrow keys.
    ///
    /// [`Walked`](Self::Walked)'s twin: it has **already happened** by the time
    /// it is recorded, because `Sim::sing` does not wait for a clock. Replay a
    /// tick, then apply the syllables recorded against it in list order.
    ///
    /// **The word, and no timing.** The sim grades on the tick a press arrived
    /// in, so *when inside the tick* changes no world state — and `Wrote`'s rule
    /// excludes exactly that.
    Sang(String),
}

/// Everything the player did, with the tick it landed on.
///
/// A replay needs exactly `(seed, submissions)` and nothing else. Recorded from
/// the start even though nothing consumes it yet, because the pairing is
/// unrecoverable after the fact — the tick a line landed on cannot be inferred
/// from the line.
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

    /// Note that a syllable was sung by hand during `tick`.
    ///
    /// Already done by the time it is recorded, like [`walked`](Self::walked).
    pub fn sang(&mut self, tick: Tick, syllable: &str) {
        self.0.push((tick, Submission::Sang(syllable.to_owned())));
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
        // **The editor's replay contract.** Keystrokes reach no decision and
        // never enter this log; the save does, once, with what the buffer held.
        // Recording the *typed* lines rather than the canonical ones is
        // deliberate — a replay re-derives the canonical form, so the
        // canonicaliser changing cannot make an old session replay into a
        // different world while claiming it did not.
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
                    // **Equal to `lines`, and that is the assertion.** Nothing
                    // read this spell, so the reading is the text — which is the
                    // identity case every build without a reader is in.
                    read: vec!["make a potion of clarity".to_owned()],
                    by: crate::augur::Verbatim::IDENTITY,
                },
            )],
        );
    }

    #[test]
    fn a_written_spell_replays_from_seed_and_submissions() {
        // The property `Submission::Wrote` exists for, and the one revision 3 of
        // the plan asserted without testing. A `Wrote` entry nothing replays is
        // a shape with no consumer.
        //
        // See `Sim::replay` for what each variant means about *when*.
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

        // **And what it *compiles* from, which is the half `spell` cannot see.**
        // `Sim::spell` returns `Held`, and `Held` is byte-identical across a
        // replay by construction — the submission carries those very lines. So
        // this test could not have caught a reading that failed to travel, and a
        // session a model had read would have replayed unread into a different
        // program. `Submission::Wrote` carries the reading for that reason, and
        // this is the assertion that says so.
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
        // **The claim `Sim::walk` makes, tested rather than argued.** It is the
        // third entry point and the only one that does not go through the tick,
        // so it is the one that could quietly put a replay a step out — and a
        // maze is the ideal witness, because being one cell wrong is visible
        // rather than subtle.
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
    /// Was `decoct nonsense` against the laboratory's three essences until
    /// `decoct` was retired (§19), then `divine nonsense` against the archive's
    /// three fragments until `divine` stopped taking one — it opens the stacks
    /// now. The dispensary's three reagents are the same shape of
    /// question, and `move` is the verb whose first slot is *required* — which
    /// is what makes the prompt appear at all.
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
