//! The record stream: what commands emit, and the only thing views read.
//!
//! # Storage
//!
//! One `String` arena, one `Vec` of fields, one `Vec` of record headers — the
//! same shape as [`Speech`](crate::Speech), and for the same reason. `ls` on a
//! full `/alembic` is a few hundred records of a few fields each; a `String` per
//! field would be a thousand allocations to draw one directory.
//!
//! A [`Record`] is therefore a borrowed view — an index and a reference, `Copy`
//! and free to pass around — never an owned struct.
//!
//! # Why this is the log
//!
//! DESIGN.md §3: *"Unlogged output is forbidden."* Scrollback, the log files a
//! player greps, the pipe source, and the balance harness's transcript are all
//! the same stream read differently. Giving each its own representation is how
//! you end up with a `sift` that searches something subtly other than what is on
//! screen.

use crate::linear::UtteranceKind;
use crate::record::field::{FieldName, Value};
use crate::record::kind::RecordKind;
use crate::record::outcome::Outcome;
use crate::style::{Intensity, Presentation, Role, Style};

/// A field, stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StoredField {
    name: FieldName,
    payload: Payload,
}

/// A value, stored — text as a range into the arena, numbers inline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Payload {
    Text { start: usize, end: usize },
    Count(u64),
    Tick(u64),
}

/// A record header: what it is, and which fields belong to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StoredRecord {
    kind: RecordKind,
    role: Role,
    presentation: Presentation,
    first: usize,
    len: usize,
    /// The authored linear variant (§3), if one was supplied.
    spoken: Option<(usize, usize)>,
}

/// A sequence of records, in emit order.
#[derive(Debug, Default, Clone)]
pub struct Records {
    text: String,
    fields: Vec<StoredField>,
    records: Vec<StoredRecord>,
    register: Presentation,
}

impl Records {
    /// An empty stream.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The register everything emitted from now on is spoken in.
    ///
    /// DESIGN.md §3 puts the eldritch treatment on *messages*, not on call
    /// sites: it is a property of how the orb is currently speaking, which in
    /// Phase 2 is driven by threat. Setting it once here rather than at every
    /// emit site is what keeps a register change from being a hundred-line diff
    /// that misses four of them.
    ///
    /// Two things still hold, and neither is this function's to override:
    ///
    /// - §3's corruption exemption. A [`RecordKind::LogLine`] refuses eldritch
    ///   however the register is set, so the diagnostic surfaces stay
    ///   trustworthy *as renderings* — see [`RecordKind::allows`].
    /// - The text stays faithful. §3: *"the renderer corrupts it; the model
    ///   records it faithfully."* The register changes the face a frontend
    ///   draws with, never the characters, which is why a register-set record
    ///   needs no separately authored spoken variant — the drawn text already
    ///   is one.
    pub const fn set_register(&mut self, register: Presentation) {
        self.register = register;
    }

    /// The register new records inherit.
    #[must_use]
    pub const fn register(&self) -> Presentation {
        self.register
    }

    /// How many records the stream holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether nothing has been emitted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Empty the stream, keeping every allocation for the next command.
    pub fn clear(&mut self) {
        self.text.clear();
        self.fields.clear();
        self.records.clear();
    }

    /// The record at `index`.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<Record<'_>> {
        (index < self.records.len()).then_some(Record {
            stream: self,
            index,
        })
    }

    /// Every record, in emit order.
    ///
    /// `Clone` on the returned iterator is load-bearing: a table measures its
    /// columns in one pass and draws them in a second, and cloning the iterator
    /// is what lets it do that without collecting into a `Vec` every frame.
    pub fn iter(&self) -> impl Iterator<Item = Record<'_>> + Clone {
        (0..self.records.len()).map(|index| Record {
            stream: self,
            index,
        })
    }

    /// Begin a record. Nothing is stored until [`RecordBuilder::finish`].
    ///
    /// The record inherits the stream's [`register`](Records::set_register).
    pub fn push(&mut self, kind: RecordKind) -> RecordBuilder<'_> {
        let first = self.fields.len();
        let presentation = self.register;
        RecordBuilder {
            stream: self,
            first,
            kind,
            role: Role::Normal,
            presentation,
            faithful: true,
            spoken: None,
        }
    }

    fn intern(&mut self, text: &str) -> (usize, usize) {
        let start = self.text.len();
        self.text.push_str(text);
        (start, self.text.len())
    }
}

/// One unit of command output: a kind, a meaning, and named fields.
#[derive(Debug, Clone, Copy)]
pub struct Record<'a> {
    stream: &'a Records,
    index: usize,
}

impl<'a> Record<'a> {
    fn stored(&self) -> &'a StoredRecord {
        &self.stream.records[self.index]
    }

    /// What this record is.
    #[must_use]
    pub fn kind(&self) -> RecordKind {
        self.stored().kind
    }

    /// What it means (§14: never carried by colour alone).
    #[must_use]
    pub fn role(&self) -> Role {
        self.stored().role
    }

    /// The presentation a renderer may actually apply.
    ///
    /// Already filtered through [`RecordKind::allows`], so §3's corruption
    /// exemption holds at every call site rather than at the ones that
    /// remembered. The unfiltered value is not reachable, on purpose.
    #[must_use]
    pub fn presentation(&self) -> Presentation {
        let stored = self.stored();
        if stored.kind.allows(stored.presentation) {
            stored.presentation
        } else {
            Presentation::Plain
        }
    }

    /// How the command that emitted this record concluded, if it said.
    #[must_use]
    pub fn outcome(&self) -> Option<Outcome> {
        match self.field(FieldName::Outcome)? {
            Value::Text(text) => Outcome::parse(text),
            Value::Count(_) | Value::Tick(_) => None,
        }
    }

    /// The semantic style a view should draw this record in.
    ///
    /// Intensity is *derived*, never set at the emit site — from the outcome
    /// where there is one, and from the kind otherwise. A record's weight on
    /// screen is a property of what it is, and letting each call site choose
    /// would make it a property of who wrote that line.
    #[must_use]
    pub fn style(&self) -> Style {
        let intensity = match (self.outcome(), self.kind()) {
            (Some(outcome), _) => outcome.intensity(),
            // The player's own words, brightest: the one line on screen they
            // are certain they authored.
            (None, RecordKind::Input) => Intensity::Bright,
            _ => Intensity::Normal,
        };
        Style {
            role: self.role(),
            intensity,
            presentation: self.presentation(),
        }
    }

    /// How many fields this record carries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.stored().len
    }

    /// Whether the record carries no fields at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Every field, in the order it was emitted.
    pub fn fields(&self) -> impl Iterator<Item = (FieldName, Value<'a>)> + Clone {
        let stored = self.stored();
        let stream = self.stream;
        stream.fields[stored.first..stored.first + stored.len]
            .iter()
            .map(move |field| {
                let value = match field.payload {
                    Payload::Text { start, end } => Value::Text(&stream.text[start..end]),
                    Payload::Count(number) => Value::Count(number),
                    Payload::Tick(number) => Value::Tick(number),
                };
                (field.name, value)
            })
    }

    /// The first field called `name`.
    ///
    /// First, not only: §8.1 lists duplicated and misordered fields among the
    /// structural signatures of a malformed record, so a record with two
    /// `state`s must stay *representable* — otherwise the tell cannot be built.
    #[must_use]
    pub fn field(&self, name: FieldName) -> Option<Value<'a>> {
        self.fields()
            .find_map(|(field, value)| (field == name).then_some(value))
    }

    /// The authored linear variant, if one was supplied (§3).
    #[must_use]
    pub fn spoken(&self) -> Option<&'a str> {
        let (start, end) = self.stored().spoken?;
        Some(&self.stream.text[start..end])
    }

    /// Every field written for a player — annotations excluded.
    ///
    /// What a reader hears and what a default view draws. See
    /// [`FieldName::is_annotation`].
    pub fn content(&self) -> impl Iterator<Item = (FieldName, Value<'a>)> + Clone {
        self.fields().filter(|(name, _)| !name.is_annotation())
    }

    /// Append what a screen reader should hear for this record.
    ///
    /// 1. An authored variant wins outright — the eldritch register's whole
    ///    accessibility contract (§3).
    /// 2. Otherwise the **kind** decides whether labels are spoken, because the
    ///    kind is what knows whether this row is a table or a sentence:
    ///
    ///    | Utterance | Spoken as | Example |
    ///    |---|---|---|
    ///    | `TableRow` | `label: value` | `"name: sage, state: ready, qty: 12"` |
    ///    | anything else | values alone | `"the ward has failed"` |
    ///
    /// §14 requires the labels *specifically for tables* — "a reader must never
    /// have to reconstruct columns from spacing" — and reciting them elsewhere
    /// turns prose into a form being read out. Deriving it from the kind rather
    /// than from the field count is what stops a machine annotation, or a second
    /// clause, from flipping a sentence into a recital.
    ///
    /// Either way the labels come from the record, so a view cannot forget them.
    pub fn speak(&self, out: &mut String) {
        if let Some(spoken) = self.spoken() {
            out.push_str(spoken);
            return;
        }
        // Labels are for reconstructing columns (§14). A row with one field has
        // no columns, so `message: tick 33` is pure noise — the kind decides
        // whether this *can* be a table, and the field count decides whether it
        // is one.
        let labelled =
            self.kind().utterance() == UtteranceKind::TableRow && self.content().count() > 1;
        for (index, (name, value)) in self.content().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            if labelled {
                out.push_str(name.label());
                out.push_str(": ");
            }
            value.write(out);
        }
    }

    /// Append the record's drawn form: content values, space separated.
    ///
    /// What a line view puts on screen, and what a command re-emitting a record
    /// into the log should carry. Storing [`Record::speak`] there instead would
    /// nest one record's linearisation inside another's field, which reads back
    /// as `message: name: seed, qty: 12648430`.
    pub fn write_line(&self, out: &mut String) {
        for (index, (_, value)) in self.content().enumerate() {
            if index > 0 {
                out.push(' ');
            }
            value.write(out);
        }
    }

    /// The record's drawn form, as an owned string.
    #[must_use]
    pub fn to_line(&self) -> String {
        let mut out = String::new();
        self.write_line(&mut out);
        out
    }

    /// What a screen reader should hear, as an owned string.
    ///
    /// Convenience for tests and one-offs. Drawing code should use
    /// [`Record::speak`] with a buffer it reuses.
    #[must_use]
    pub fn to_speech(&self) -> String {
        let mut out = String::new();
        self.speak(&mut out);
        out
    }
}

/// A record under construction.
///
/// `#[must_use]` is the enforcement: dropping the builder without
/// [`RecordBuilder::finish`] leaves the final `Self` unused, which the
/// workspace's `-D warnings` gate turns into a build failure.
#[must_use = "a record is not emitted until finish() is called"]
#[derive(Debug)]
pub struct RecordBuilder<'a> {
    stream: &'a mut Records,
    first: usize,
    kind: RecordKind,
    role: Role,
    presentation: Presentation,
    /// Whether the drawn text is undamaged, so the spoken form is the drawn one.
    ///
    /// True for a register-inherited treatment and false the moment a caller
    /// asks for one explicitly — see [`RecordBuilder::finish`].
    faithful: bool,
    spoken: Option<(usize, usize)>,
}

impl RecordBuilder<'_> {
    /// Add a text field.
    pub fn text(self, name: FieldName, value: &str) -> Self {
        let (start, end) = self.stream.intern(value);
        self.stream.fields.push(StoredField {
            name,
            payload: Payload::Text { start, end },
        });
        self
    }

    /// Add a count. Stays a number; see [`Value`].
    pub fn count(self, name: FieldName, value: u64) -> Self {
        self.stream.fields.push(StoredField {
            name,
            payload: Payload::Count(value),
        });
        self
    }

    /// Add a point in world time, in ticks.
    pub fn tick(self, name: FieldName, value: u64) -> Self {
        self.stream.fields.push(StoredField {
            name,
            payload: Payload::Tick(value),
        });
        self
    }

    /// Set what this record means.
    pub const fn role(mut self, role: Role) -> Self {
        self.role = role;
        self
    }

    /// Record how the command concluded.
    ///
    /// Stored as an annotation, so it classifies the record for a view without
    /// being spoken or drawn as text. See [`Outcome`].
    pub fn outcome(self, outcome: Outcome) -> Self {
        self.text(FieldName::Outcome, outcome.as_str())
    }

    /// Ask for a presentation treatment on this record alone.
    ///
    /// *Ask*, not set: §3's exemption may deny it, and
    /// [`Record::presentation`] is what reports the answer.
    ///
    /// Asking explicitly means the drawn text may itself be damaged — that is
    /// the case the authored spoken variant exists for — so this record now owes
    /// one. Inheriting the stream's [register](Records::set_register) does not,
    /// because a register never alters the characters.
    pub const fn presentation(mut self, presentation: Presentation) -> Self {
        self.presentation = presentation;
        self.faithful = false;
        self
    }

    /// Supply the authored linear variant (§3).
    pub fn spoken(mut self, text: &str) -> Self {
        self.spoken = Some(self.stream.intern(text));
        self
    }

    /// Emit the record.
    ///
    /// # Panics
    ///
    /// In debug builds, if the record will render as
    /// [`Presentation::Eldritch`] without an authored spoken variant. §3 requires
    /// one on *every* eldritch message, and the alternative to catching it at the
    /// emit site is shipping a tonal register screen-reader players cannot hear.
    /// Mirrors the same assertion in [`Painter::span`](crate::Painter::span),
    /// one layer earlier.
    pub fn finish(self) {
        let effective = if self.kind.allows(self.presentation) {
            self.presentation
        } else {
            Presentation::Plain
        };
        debug_assert!(
            effective != Presentation::Eldritch || self.faithful || self.spoken.is_some(),
            "eldritch {:?} record with damaged text and no authored spoken variant",
            self.kind,
        );

        let first = self.first;
        self.stream.records.push(StoredRecord {
            kind: self.kind,
            role: self.role,
            presentation: self.presentation,
            first,
            len: self.stream.fields.len() - first,
            spoken: self.spoken,
        });
    }
}
