//! The record stream: what commands emit, and the only thing views read.
//!
//! One `String` arena, one `Vec` of fields, one `Vec` of record headers — the
//! shape [`Speech`](crate::Speech) uses, for the same reason: `ls` on a full
//! `/laboratory` is a few hundred records, and a `String` per field would be a
//! thousand allocations to draw one directory. So a [`Record`] is a borrowed
//! view, `Copy` and free to pass around, never an owned struct.
//!
//! This is the log. DESIGN.md §3: *"Unlogged output is forbidden."* Scrollback,
//! the log files a player greps, the pipe source and the balance harness's
//! transcript are one stream read differently; a representation each is how
//! `sift` ends up searching something other than what is on screen.

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
    /// Logged, but not drawn in the transcript. See [`RecordBuilder::quiet`].
    quiet: bool,
}

/// A sequence of records, in emit order.
#[derive(Debug, Default, Clone)]
pub struct Records {
    text: String,
    fields: Vec<StoredField>,
    entries: Vec<StoredRecord>,
    register: Presentation,
    /// How many records have **ever** been pushed, across the stream's whole
    /// life — see [`Records::sequence`].
    pushed: u64,
    /// The spell everything emitted from now on is the doing of, if any.
    ///
    /// Set once around an instruction rather than at every emit site, for
    /// [`Records::register`]'s reason: a spell's output is whatever the commands
    /// it ran emitted, and those are the *same* sites a player's typing reaches,
    /// so stamping them all would mean every future one had to remember.
    attributed: Option<String>,
}

impl Records {
    /// An empty stream.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty stream that has already dropped `dropped` records.
    ///
    /// A save carries a bounded tail: the stream grows without bound and `.log`
    /// files are a view over it, so dropping it would empty every log in the
    /// tower. [`sequence`](Self::sequence) cannot be dropped — a running spell's
    /// cursor is a position in it, and [`dropped`](Self::dropped) turns that
    /// back into an index — so a restore opens here with the count of records it
    /// is *not* carrying, then pushes the tail.
    #[must_use]
    pub fn resume(dropped: u64) -> Self {
        Self {
            pushed: dropped,
            ..Self::default()
        }
    }

    /// The register everything emitted from now on is spoken in.
    ///
    /// DESIGN.md §3 puts the eldritch treatment on *messages*, not on call
    /// sites: it is how the orb is currently speaking, driven in Phase 8 by
    /// threat. Setting it once keeps a register change from being a
    /// hundred-line diff that misses four of them.
    ///
    /// Two things hold, and neither is this function's to override:
    ///
    /// - §3's corruption exemption. A [`RecordKind::LogLine`] refuses eldritch
    ///   however the register is set, so the diagnostic surfaces stay
    ///   trustworthy *as renderings* — see [`RecordKind::allows`].
    /// - The text stays faithful. §3: *"the renderer corrupts it; the model
    ///   records it faithfully."* The register changes the face, never the
    ///   characters, so a register-set record needs no authored spoken variant.
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
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing has been emitted.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// How many records have **ever** been pushed.
    ///
    /// A sequence number, not a length. A watching script keeps a cursor into
    /// the stream, and the obvious cursor — an index, `len()` — is correct only
    /// while the stream is never truncated. It never is today, but it grows
    /// without bound and §5's Phase 11a offline catch-up is ~29k steps, so the
    /// day someone adds rotation every saved cursor would point at the wrong
    /// record and spells would re-fire or skip events with no test catching it.
    /// This does not reset, so a cursor survives a truncation.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.pushed
    }

    /// Every record a transcript should draw: the ones nobody's spell caused.
    ///
    /// One definition, because four callers measure this stream. As a closure in
    /// the frontend's paint it left paging, the scroll clamp and the typewriter
    /// reveal counting the *whole* stream while the pane drew a subset — two
    /// units for one `Scroll::back`, so `PageUp` jumped whole screens with a
    /// spell running. Anything needing "what the player sees" asks here.
    ///
    /// Two reasons to skip, one rule: a record is undrawn because it was a
    /// spell's rather than the player's ([`FieldName::Spell`]), or because the
    /// screen already says it ([`RecordBuilder::quiet`]). Neither is a deletion.
    pub fn drawn(&self) -> impl Iterator<Item = Record<'_>> + Clone {
        self.iter()
            .filter(|record| record.field(FieldName::Spell).is_none() && !record.is_quiet())
    }

    /// How many records a transcript would draw.
    #[must_use]
    pub fn drawn_len(&self) -> usize {
        self.drawn().count()
    }

    /// Every drawn record pushed at or after `sequence`.
    ///
    /// [`drawn`](Self::drawn) with a watermark, for anything that reacts to the
    /// stream rather than painting it — the audio cues are the first. A
    /// [`sequence`](Self::sequence) rather than an index, because an index is
    /// broken by [`clear`](Self::clear), which empties the stream between
    /// commands, and by a restore, which opens with a truncated tail.
    ///
    /// A `sequence` from the future yields nothing rather than panicking — a
    /// swapped tower can rewind the count, and the caller's answer is to take
    /// the new mark and say nothing.
    pub fn since(&self, sequence: u64) -> impl Iterator<Item = Record<'_>> + Clone {
        let dropped = self.dropped();
        let from = usize::try_from(sequence.saturating_sub(dropped)).unwrap_or(usize::MAX);
        self.drawn().skip_while(move |record| record.index < from)
    }

    /// Attribute everything pushed from now on to `spell`, or to nobody.
    ///
    /// Paired with a clear, always — a runner that returned without clearing
    /// would attribute the player's own next line to a spell, and the transcript
    /// would stop showing them their own typing.
    pub fn attribute(&mut self, spell: Option<&str>) {
        self.attributed = spell.map(ToOwned::to_owned);
    }

    /// Which spell is being credited, if any.
    #[must_use]
    pub fn attributed(&self) -> Option<&str> {
        self.attributed.as_deref()
    }

    /// How many records have been dropped off the front, if any.
    ///
    /// `sequence() - len()`. What a cursor subtracts to find its index.
    #[must_use]
    pub const fn dropped(&self) -> u64 {
        self.pushed.saturating_sub(self.entries.len() as u64)
    }

    /// Empty the stream, keeping every allocation for the next command.
    ///
    /// `pushed` survives, so a cursor taken before a clear does not silently
    /// start pointing at new records — see [`sequence`](Self::sequence).
    pub fn clear(&mut self) {
        self.text.clear();
        self.fields.clear();
        self.entries.clear();
    }

    /// The record at `index`.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<Record<'_>> {
        (index < self.entries.len()).then_some(Record {
            stream: self,
            index,
        })
    }

    /// Every record, in emit order.
    ///
    /// `Clone` on the returned iterator is load-bearing: a table measures its
    /// columns in one pass and draws them in a second, and cloning lets it do
    /// that without collecting into a `Vec` every frame.
    pub fn iter(&self) -> impl Iterator<Item = Record<'_>> + Clone {
        (0..self.entries.len()).map(|index| Record {
            stream: self,
            index,
        })
    }

    /// Begin a record. Nothing is stored until [`RecordBuilder::finish`].
    ///
    /// The record inherits the stream's [`register`](Records::set_register).
    pub const fn push(&mut self, kind: RecordKind) -> RecordBuilder<'_> {
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
            quiet: false,
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
        &self.stream.entries[self.index]
    }

    /// What this record is.
    #[must_use]
    pub fn kind(&self) -> RecordKind {
        self.stored().kind
    }

    /// Whether this was logged without being drawn — see
    /// [`RecordBuilder::quiet`].
    #[must_use]
    pub fn is_quiet(&self) -> bool {
        self.stored().quiet
    }

    /// What it means (§14: never carried by colour alone).
    #[must_use]
    pub fn role(&self) -> Role {
        self.stored().role
    }

    /// The presentation a renderer may actually apply.
    ///
    /// Already filtered through [`RecordKind::allows`], so §3's corruption
    /// exemption holds at every call site rather than the ones that remembered.
    /// The unfiltered value is unreachable, on purpose.
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

    /// The glyph a prompt draws in front of this line.
    ///
    /// Derived from all three channels the record carries, because any one alone
    /// is a lie somewhere: a refusal is a [`RecordKind::Completion`] — the work
    /// *concluded* — so on kind alone "you cannot brew and decipher at once"
    /// wore a success's `√`.
    #[must_use]
    pub fn marker(&self) -> Option<char> {
        if let Some(outcome) = self.outcome() {
            return Some(outcome.marker());
        }
        match (self.kind(), self.role()) {
            // §5.0's refusals cost the player something they wanted, and §4
            // reserves the accent triad for exactly that.
            (RecordKind::Completion, Role::Cost | Role::Danger) => Some('¬'),
            (kind, _) => kind.marker(),
        }
    }

    /// The semantic style a view should draw this record in.
    ///
    /// Intensity is *derived*, never set at the emit site — from the outcome
    /// where there is one, from the kind otherwise. A record's weight on screen
    /// is a property of what it is; letting each call site choose would make it
    /// a property of who wrote that line.
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
            // A record is a reading, never a picture. Only the athanor's meter
            // depicts anything, and it is painted rather than logged.
            depiction: crate::style::Depiction::None,
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
    /// First, not only: §8.1 lists duplicated and misordered fields among a
    /// malformed record's structural signatures, so two `state`s must stay
    /// *representable* or the tell cannot be built.
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
    /// 1. An authored variant wins outright — §3's accessibility contract for
    ///    the eldritch register.
    /// 2. Otherwise the kind decides whether labels are spoken, because the kind
    ///    knows whether this row is a table or a sentence:
    ///
    ///    | Utterance | Spoken as | Example |
    ///    |---|---|---|
    ///    | `TableRow` | `label: value` | `"name: sage, state: ready, qty: 12"` |
    ///    | anything else | values alone | `"the ward has failed"` |
    ///
    /// §14 requires labels *specifically for tables* — "a reader must never have
    /// to reconstruct columns from spacing" — and reciting them elsewhere reads
    /// prose out as a form. Taking it from the kind rather than the field count
    /// stops an annotation, or a second clause, flipping a sentence into a
    /// recital. Either way the labels come from the record.
    pub fn speak(&self, out: &mut String) {
        if let Some(spoken) = self.spoken() {
            out.push_str(spoken);
            return;
        }
        // Labels are for reconstructing columns (§14), and a row with one field
        // has none, so `message: tick 33` is noise. The kind decides whether
        // this *can* be a table; the field count decides whether it is one.
        let labelled =
            self.kind().utterance() == UtteranceKind::TableRow && self.content().count() > 1;
        let prose = self.is_prose();
        for (index, (name, value)) in self.presented(prose).enumerate() {
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
    /// into the log should carry. [`Record::speak`] there would nest one
    /// record's linearisation inside another's field, reading back as
    /// `message: name: seed, qty: 12648430`.
    pub fn write_line(&self, out: &mut String) {
        let prose = self.is_prose();
        for (index, (_, value)) in self.presented(prose).enumerate() {
            if index > 0 {
                out.push(' ');
            }
            value.write(out);
        }
    }

    /// Whether this record carries authored prose that speaks *for* its facts.
    ///
    /// The presentation half of rule 4. A record may carry both: a refusal holds
    /// `name`, `state` and `source` so `sift` and a pipe can filter it, and an
    /// authored `message` so a player reads a sentence. Drawing both put the
    /// sentence next to the values it explains — `decoct decoct laboratory the
    /// laboratory is busy…`.
    ///
    /// A `TableRow` is excluded by construction: it *is* columns and §14
    /// requires their labels, so prose winning there would delete what a
    /// listener needs to reconstruct the row.
    fn is_prose(&self) -> bool {
        self.kind().utterance() != UtteranceKind::TableRow
            && self.field(FieldName::Message).is_some()
    }

    /// The fields a view should show, given whether this record is prose.
    ///
    /// Facts stay on the record either way — this narrows what is *drawn and
    /// spoken*, never what is stored or searched. `sift` matches a field's own
    /// value and never comes through here (see [`sift`](super::sift)).
    ///
    /// [`FieldName::Line`] is prose's one exception, because it is a listing's
    /// gutter: prose keeps what a person would read aloud, which is wrong for a
    /// numbered file, where the number is how a player names the line they mean.
    /// Every other field a prose record carries classifies it rather than
    /// saying anything.
    fn presented(&self, prose: bool) -> impl Iterator<Item = (FieldName, Value<'a>)> + Clone {
        self.content().filter(move |(name, _)| {
            !prose
                || matches!(
                    name,
                    FieldName::Line | FieldName::Message | FieldName::Detail
                )
        })
    }

    /// The record's drawn form, as an owned string.
    #[must_use]
    pub fn to_line(&self) -> String {
        let mut out = String::new();
        self.write_line(&mut out);
        out
    }

    /// Every field, prose or not — a search result, not a drawn line.
    ///
    /// [`matches`](Self::matches) runs over all field values and never over
    /// rendered text (rule 4, §7: *"the only model that survives the eldritch
    /// renderer corrupting output"*), but [`to_line`](Self::to_line) draws a
    /// prose record as its sentence alone — so `sift wield orb.log` returned
    /// rows matching `Name = "wield"` that drew as *"the `balneum_mariae` is
    /// empty"*, a filter that looks broken because the term is nowhere in the
    /// result. Narrowing `matches` instead would make what a pipe can find
    /// depend on what a view shows, the coupling rule 4 forbids, so the *result*
    /// widens and only for search.
    #[must_use]
    pub fn to_line_verbatim(&self) -> String {
        let mut out = String::new();
        for (index, (_, value)) in self.presented(false).enumerate() {
            if index > 0 {
                out.push(' ');
            }
            value.write(&mut out);
        }
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
    quiet: bool,
}

impl RecordBuilder<'_> {
    /// Log this, but keep it out of the transcript.
    ///
    /// For a fact the screen is already showing. The archive's `follow` emitted
    /// *"the reading goes north"* on every step of a hundreds-of-steps maze, so
    /// walking one buried the pane in a line per press restating what the map
    /// had just drawn — and scrolled the player's own typing away.
    ///
    /// Not a deletion: §3 forbids unlogged output, and this is emitted, stored,
    /// `sift`-able and spoken as before, so `peruse archive.log` reads every
    /// step. It is [`FieldName::Spell`]'s "log, not pane" rule for the other
    /// reason there is to invoke it — that one is *whose* doing, this one is
    /// *worth drawing*. Only where something else on screen already says this: a
    /// refusal is never quiet, and `follow` into a wall stays drawn because
    /// nothing moves and the map reports nothing at all.
    pub const fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

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
    /// [`Record::presentation`] reports the answer. Asking explicitly means the
    /// drawn text may itself be damaged, which is what the authored spoken
    /// variant exists for, so this record now owes one. Inheriting the stream's
    /// [register](Records::set_register) does not: a register never alters the
    /// characters.
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
    /// In debug builds, if the record will render as [`Presentation::Eldritch`]
    /// without an authored spoken variant. §3 requires one on *every* eldritch
    /// message, and not catching it at the emit site means shipping a register
    /// screen-reader players cannot hear. Mirrors the assertion in
    /// [`Painter::span`](crate::Painter::span), one layer earlier.
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

        // Stamped here rather than by the caller: everything a spell does goes
        // through the ordinary emit sites, so the stream is the only place that
        // reliably knows.
        if let Some(spell) = self.stream.attributed.clone() {
            let (start, end) = self.stream.intern(&spell);
            self.stream.fields.push(StoredField {
                name: FieldName::Spell,
                payload: Payload::Text { start, end },
            });
        }

        let first = self.first;
        self.stream.pushed = self.stream.pushed.saturating_add(1);
        self.stream.entries.push(StoredRecord {
            kind: self.kind,
            role: self.role,
            presentation: self.presentation,
            first,
            len: self.stream.fields.len() - first,
            spoken: self.spoken,
            quiet: self.quiet,
        });
    }
}
