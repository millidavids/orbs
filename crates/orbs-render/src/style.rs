//! Semantic styling — and one channel that is deliberately not.
//!
//! A [`Style`] says what a cell *means*. Turning that into a phosphor hue, an
//! ANSI index, or a glyph variant is the frontend's job, and each frontend does
//! it differently: the Bevy build has curated phosphor themes, the terminal
//! build has indexed ANSI inherited from the user's terminal (DESIGN.md §13).
//! A concrete colour must never appear in this crate.
//!
//! Two rules from the design are enforced here by the type system rather than by
//! discipline:
//!
//! - **Colour never carries meaning alone** (§14). [`Role`] is carried into the
//!   screen-reader stream alongside the text, so the meaning survives with no
//!   pixels at all.
//! - **Eldritch and sabotage signal vocabularies are disjoint** (§3). Because
//!   [`Presentation`] is an enum, a cell cannot be both, and the tonal system
//!   cannot jam the diagnostic system at peak threat.
//!
//! # The exception, and why it is safe
//!
//! [`Depiction`] is the one channel here that says nothing. It selects a colour
//! ramp and no more: it is how the athanor's meter becomes a picture of a fire
//! rather than a reading of one.
//!
//! That does not weaken the rule above, and the reason is worth stating rather
//! than trusting. §14 forbids colour being the **sole carrier of meaning**; the
//! guarantee that makes it true is that every [`Role`] reaches the linear stream
//! beside its text. A channel carrying *no* meaning has nothing to withhold from
//! a listener, so it cannot break that guarantee — which is why
//! [`Depiction`] is asserted by test to be absent from the stream entirely, and
//! why [`Style::depicted`] refuses to paint over an accent.

/// What a cell *means*, from the accent set reserved strictly for meaning.
///
/// DESIGN.md §4: the base hue carries all ordinary text through intensity
/// variation alone, and accents are never decorative.
///
/// Deliberately **not** `#[non_exhaustive]`. Every consumer is in this
/// workspace, and exhaustive matching is the point: a frontend that gains a new
/// role must be made to handle it, because the failure mode of a missed arm is
/// silently rendering danger as ordinary text.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    /// Ordinary text. Carried by the base hue.
    #[default]
    Normal,
    /// Failure, breach, hostile action.
    Danger,
    /// Mana and arcane expenditure.
    Cost,
    /// Completion.
    Success,
}

/// Relative brightness within the base hue.
///
/// This is the only channel ordinary text is allowed to vary on, which is what
/// keeps the accent triad meaningful.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Intensity {
    /// Recessive: borders, timestamps, inactive panes.
    Dim,
    /// The default weight for body text.
    #[default]
    Normal,
    /// Emphasised: headings, the focused pane, a value that just changed.
    Bright,
}

/// A presentation treatment applied on top of the base style.
///
/// The two non-plain variants are mutually exclusive by design rule (§3), which
/// is why this is an enum and not a set of flags.
///
/// # One face per variant
///
/// The Bevy frontend draws each variant from a different face of the same font
/// family (`assets/fonts/unscii/`), which is why this enum is worth its own
/// channel rather than being folded into [`Role`]:
///
/// | Variant | Face | Native |
/// |---|---|---|
/// | [`Presentation::Plain`] | `unscii-16` | 8×16 |
/// | [`Presentation::Eldritch`] | `unscii-8-fantasy` | 8×8, row-doubled |
/// | [`Presentation::Tampered`] | `unscii-8-mcr` | 8×8, row-doubled |
///
/// All three share metrics and repertoire, so a face swap can never move a cell.
/// The terminal frontend has no such control and renders all three identically —
/// an accepted degradation (§8.1), since `verify` is the authoritative detector
/// on every surface and every frontend.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Presentation {
    /// Rendered faithfully. Carries the prose budget, so it gets the face drawn
    /// at 8×16 rather than a stretched 8×8.
    #[default]
    Plain,
    /// High-threat tonal register (§3). The frontend may treat this cell
    /// unusually — spacing, timing, phosphor behaviour.
    ///
    /// The *model* always records eldritch output faithfully; only the rendering
    /// is affected. Content marked this way must supply an authored spoken
    /// variant (see [`Span::with_spoken`](crate::Span::with_spoken)), or
    /// screen-reader players lose an entire tonal register.
    ///
    /// Never applied to script listings, schedule listings, or log output —
    /// those are corruption-exempt diagnostic surfaces (§3).
    Eldritch,
    /// A sabotage tell (§8.1): the frontend draws a subtly wrong variant of the
    /// glyph.
    ///
    /// A **bonus channel, never the only one.** `verify` is the authoritative
    /// detector on every surface and every frontend; the terminal build cannot
    /// render this at all because it does not control the font, and the design
    /// accepts that as degradation rather than breakage.
    Tampered,
}

/// How hot a flame or spark cell is — see [`Depiction::flame`] and
/// [`Depiction::spark`].
///
/// **Four steps, where an earlier design had three.** That argument was that
/// CP437 gives flame two glyphs (`▓` and `█`) and a ramp longer than the glyphs
/// can back is colour variation with nothing underneath it. It no longer holds:
/// the bottom half of the fire is solid `█` by construction and carries *all* of
/// its motion in hue, so the ramp is doing the work alone there and a coarse one
/// reads as a two-tone flicker rather than a glow.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Heat {
    /// Dull: the tip of the flame, and a spark about to go out.
    #[default]
    Ember,
    /// The body of the flame.
    Flame,
    /// Hot, and most of what the base of the fire shifts between.
    Blaze,
    /// The brightest the tube goes — the heart of the fire, at its base.
    Core,
}

/// The colour family a material draws its instrument's bar in.
///
/// **A name, never a value.** This module forbids concrete colours and that
/// still holds: `Green` says *which family*, and what green is on a given tube is
/// the frontend's answer, exactly as it is for [`Role`]. **`orbs-tui` does
/// resolve the same eight names to ANSI indices, and is right** — the prediction
/// this comment used to make is now `orbs-tui`'s `theme::hue`, and the closed
/// set is what makes it right: eight names, eight hues, no fallback, so a ninth
/// tint breaks that build on purpose.
///
/// # It carries a hint, and the hint is never the only carrier
///
/// A tint says what is inside an instrument — sage grinds green, its husks are
/// brown — which is a *hint over* `survey`, not a substitute for it. Three things
/// keep it inside §14:
///
/// - `survey <instrument>` names the contents outright, on every frontend.
/// - The panel names the instrument and its state, and both reach the linear
///   stream.
/// - A tint **never paints over an accent**. `Fouled` draws its label in
///   `Role::Danger` and the accent wins, for the same reason
///   [`Style::depicted`] drops a picture on an accented cell: §4 reserves the
///   triad strictly for meaning.
///
/// DESIGN.md §19 records the widening this represents, and the Phase 11 item that
/// carries the accessibility promise it moved.
///
/// # Eight, and closed
///
/// Deliberately **not** `#[non_exhaustive]`, matching [`Role`] and
/// [`Depiction`]: a frontend that gains a tint must be made to handle it rather
/// than silently drawing a material in the base hue. Authored data selects one
/// *by name* and an unknown name is a load error — never a silent fallback,
/// which is the defect `Recipe::heat` and `craft_of` each already paid for.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tint {
    /// Growing things: herbs, leaves, sage.
    #[default]
    Green,
    /// Spent plant matter: husks, bark, wood.
    Brown,
    /// Stone and salt, and what a fire leaves.
    Grey,
    /// Finished work — the colour of something worth having.
    Gold,
    /// Arcane, and the register the design already reads as magical.
    Violet,
    /// Blood, heat, and the reagents that are dangerous to hold.
    Red,
    /// Water and the things dissolved in it.
    Blue,
    /// Bone, chalk, and anything bleached.
    Bone,
}

impl Tint {
    /// Every variant, kept honest by a test rather than by memory. See
    /// [`Depiction::ALL`].
    pub const ALL: [Self; 8] = [
        Self::Green,
        Self::Brown,
        Self::Grey,
        Self::Gold,
        Self::Violet,
        Self::Red,
        Self::Blue,
        Self::Bone,
    ];

    /// The name authored data selects this by.
    ///
    /// Lower-case and single words, because they are typed into a TOML table by
    /// a person and every other authored key in the project reads that way.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Green => "green",
            Self::Brown => "brown",
            Self::Grey => "grey",
            Self::Gold => "gold",
            Self::Violet => "violet",
            Self::Red => "red",
            Self::Blue => "blue",
            Self::Bone => "bone",
        }
    }

    /// The tint a name selects, or `None` if it names nothing.
    ///
    /// **`None` is an error for the caller to report, not a default to
    /// substitute.** A typo that silently drew the base hue would look exactly
    /// like a material nobody had tinted yet.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|tint| tint.name() == name.trim().to_ascii_lowercase())
    }
}

/// A region's colour, which is one family or two being combined.
///
/// **A payload is affordable here and would not be on a [`Cell`](crate::Cell).**
/// This lives in the `Frame`'s tint side-table, one entry per instrument bar, so
/// it costs a handful of words a frame rather than a byte per grid position.
/// That is the whole reason the tint went on the `Frame` rather than into
/// [`Style`], and it is what lets the flask express a thing the other
/// instruments cannot: two materials on their way to becoming one.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Wash {
    /// The family this region draws in.
    pub tint: Tint,
    /// A second, mixed evenly with the first.
    ///
    /// `Some` only for the `flask_and_rod` mid-combination, where the growing
    /// portion of the bar is literally the two inputs becoming one thing. The
    /// frontend averages the two families rather than picking a third, because a
    /// third colour appearing is a *substitution* and the picture is about a
    /// *mixture*.
    pub with: Option<Tint>,
}

impl Wash {
    /// One family, unmixed. What every instrument but the flask draws.
    #[must_use]
    pub const fn plain(tint: Tint) -> Self {
        Self { tint, with: None }
    }

    /// Two families combining.
    ///
    /// Order does not matter to the eye and must not matter here either — the
    /// flask's two inputs are a set, and `mix(a, b)` drawing differently from
    /// `mix(b, a)` would make the bar depend on which reagent the player happened
    /// to put in first.
    #[must_use]
    pub const fn mixing(first: Tint, second: Tint) -> Self {
        Self {
            tint: first,
            with: Some(second),
        }
    }
}

/// How hard a cell of liquid is moving.
///
/// The balneum mariae's bar is a level of liquid, and **all of its motion is
/// here** — the glyph never changes, so the value survives with the colour
/// thrown away and there is nothing in the picture that could be mistaken for a
/// bubble. Three steps, because the liquid is one solid glyph and the ramp is
/// doing the work alone: two would read as a two-tone flicker, and a fourth
/// would be a distinction nobody can see at this brightness.
///
/// Ordered still-to-moving, so `Ord` sorts the way the picture reads.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Roil {
    /// Barely moving: a bath at rest, and the deep of a working one.
    #[default]
    Still,
    /// Turning over.
    Stirred,
    /// The strongest the liquid moves — a gentle heat's idea of vigorous.
    Rolling,
}

/// How thick a smoke cell is.
///
/// Two steps, for the reason [`Heat`]'s *three*-step version was argued on:
/// smoke has `░` and `▒` and nothing else, so a longer ramp would be colour with
/// nothing under it. That argument stopped binding `Heat` when the flame's bottom
/// half became glyph-constant; it still binds this.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Density {
    /// A wisp.
    #[default]
    Thin,
    /// A cell with some body to it, near the fire that made it.
    Thick,
}

/// A depictive treatment: what the cell is a picture *of*.
///
/// **Carries no meaning**, and is the only thing in this module of which that is
/// true. Everything a [`Role`] says survives with no pixels at all, because the
/// role travels into the linear stream beside its text; this says nothing, is
/// spoken nowhere, and a frontend that ignored it entirely would lose no
/// information — only a picture. See this module's header for why that is what
/// makes it safe.
///
/// Deliberately **not** `#[non_exhaustive]`, matching [`Role`]: every consumer is
/// in this workspace, and a frontend that gains a new depiction should be made to
/// handle it rather than silently drawing it as ordinary text.
///
/// # Flat, not nested, and the reason is [`Cell`](crate::Cell)'s size
///
/// The obvious shape is `Flame(Heat)`/`Smoke(Density)`/`Spark(Heat)`. That is a
/// **two**-byte tagged enum, which pushes [`Style`] from 3 bytes to 5 and — past
/// `char`'s 4-byte alignment — [`Cell`](crate::Cell) from 8 to 12. Every screen
/// pays it: `Frame::reset` memsets the whole grid every frame, measured at
/// +1.2 µs a frame and +28 KiB of buffer at 160×44, for a picture that occupies
/// about thirty cells of one instrument panel.
///
/// Spelled out, the discriminant is one byte and `Cell` is exactly what it was.
/// [`Depiction::flame`] and friends keep the nested shape available where it
/// reads better.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Depiction {
    /// Not a picture of anything. Everything on screen but the instruments.
    #[default]
    None,
    /// Burning fuel at [`Heat::Ember`].
    FlameEmber,
    /// Burning fuel at [`Heat::Flame`].
    FlameBody,
    /// Burning fuel at [`Heat::Blaze`].
    FlameBlaze,
    /// Burning fuel at [`Heat::Core`].
    FlameCore,
    /// Smoke at [`Density::Thin`].
    SmokeThin,
    /// Smoke at [`Density::Thick`].
    SmokeThick,
    /// A spark at [`Heat::Ember`].
    ///
    /// Sparks are their own variants rather than flame in the plume, because
    /// they draw from a different glyph set — the small marks `∙ ° ·` rather
    /// than the shade blocks — and folding them in would cost the invariant that
    /// a flame cell is always one of two glyphs. They resolve against the
    /// **same** ramp: a spark is a piece of the fire, so a theme that tuned one
    /// tuned both.
    SparkEmber,
    /// A spark at [`Heat::Flame`].
    SparkBody,
    /// A spark at [`Heat::Blaze`].
    SparkBlaze,
    /// A spark at [`Heat::Core`].
    SparkCore,
    /// Liquid at [`Roil::Still`].
    LiquidStill,
    /// Liquid at [`Roil::Stirred`].
    LiquidStirred,
    /// Liquid at [`Roil::Rolling`].
    LiquidRolling,
    /// What a run has left settled at the bottom of a vessel.
    ///
    /// Its own step rather than the bottom of the liquid ramp: sediment is a
    /// different *substance*, and a bath holding nothing but the last run's grit
    /// must not read as a bath holding a little liquid.
    Sediment,
}

impl Depiction {
    /// Every variant, for consumers that must handle the whole set.
    ///
    /// **Kept honest by a test, not by memory.** `orbs`'s palette walks this to
    /// prove every depiction resolves to a colour — and that walk used to be a
    /// hand-written array in the consumer, which is a list that silently falls
    /// behind the enum the moment a variant is added. `exhaustive` below fails to
    /// *compile* if this misses one.
    pub const ALL: [Self; 15] = [
        Self::None,
        Self::FlameEmber,
        Self::FlameBody,
        Self::FlameBlaze,
        Self::FlameCore,
        Self::SmokeThin,
        Self::SmokeThick,
        Self::SparkEmber,
        Self::SparkBody,
        Self::SparkBlaze,
        Self::SparkCore,
        Self::LiquidStill,
        Self::LiquidStirred,
        Self::LiquidRolling,
        Self::Sediment,
    ];

    /// Burning fuel at a given heat.
    #[must_use]
    pub const fn flame(heat: Heat) -> Self {
        match heat {
            Heat::Ember => Self::FlameEmber,
            Heat::Flame => Self::FlameBody,
            Heat::Blaze => Self::FlameBlaze,
            Heat::Core => Self::FlameCore,
        }
    }

    /// A spark at a given heat.
    #[must_use]
    pub const fn spark(heat: Heat) -> Self {
        match heat {
            Heat::Ember => Self::SparkEmber,
            Heat::Flame => Self::SparkBody,
            Heat::Blaze => Self::SparkBlaze,
            Heat::Core => Self::SparkCore,
        }
    }

    /// Smoke at a given density.
    #[must_use]
    pub const fn smoke(density: Density) -> Self {
        match density {
            Density::Thin => Self::SmokeThin,
            Density::Thick => Self::SmokeThick,
        }
    }

    /// Liquid moving at a given vigour.
    #[must_use]
    pub const fn liquid(roil: Roil) -> Self {
        match roil {
            Roil::Still => Self::LiquidStill,
            Roil::Stirred => Self::LiquidStirred,
            Roil::Rolling => Self::LiquidRolling,
        }
    }

    /// Where on the flame ramp this sits, if it is drawn from it at all.
    ///
    /// Sparks and flame share the ramp; everything else has its own and answers
    /// `None`.
    #[must_use]
    pub const fn heat(self) -> Option<Heat> {
        match self {
            Self::FlameEmber | Self::SparkEmber => Some(Heat::Ember),
            Self::FlameBody | Self::SparkBody => Some(Heat::Flame),
            Self::FlameBlaze | Self::SparkBlaze => Some(Heat::Blaze),
            Self::FlameCore | Self::SparkCore => Some(Heat::Core),
            Self::None
            | Self::SmokeThin
            | Self::SmokeThick
            | Self::LiquidStill
            | Self::LiquidStirred
            | Self::LiquidRolling
            | Self::Sediment => None,
        }
    }

    /// Where on the liquid ramp this sits, if it is liquid at all.
    ///
    /// [`Depiction::Sediment`] answers `None`: it is drawn from the vessel's
    /// colour but it is not moving and never will be.
    #[must_use]
    pub const fn roil(self) -> Option<Roil> {
        match self {
            Self::LiquidStill => Some(Roil::Still),
            Self::LiquidStirred => Some(Roil::Stirred),
            Self::LiquidRolling => Some(Roil::Rolling),
            _ => None,
        }
    }

    /// Whether this is a cell of a vessel's contents — liquid or what it left.
    #[must_use]
    pub const fn is_liquid(self) -> bool {
        matches!(
            self,
            Self::LiquidStill | Self::LiquidStirred | Self::LiquidRolling
        )
    }

    /// Whether this is a spark rather than the body of the fire.
    #[must_use]
    pub const fn is_spark(self) -> bool {
        matches!(
            self,
            Self::SparkEmber | Self::SparkBody | Self::SparkBlaze | Self::SparkCore
        )
    }

    /// Whether this is the body of the fire.
    #[must_use]
    pub const fn is_flame(self) -> bool {
        matches!(
            self,
            Self::FlameEmber | Self::FlameBody | Self::FlameBlaze | Self::FlameCore
        )
    }

    /// Whether this is smoke.
    #[must_use]
    pub const fn is_smoke(self) -> bool {
        matches!(self, Self::SmokeThin | Self::SmokeThick)
    }

    /// Whether a material's tint must **not** be painted over this picture.
    ///
    /// **One rule, in the crate that owns the vocabulary.** Both frontends
    /// resolve tints and both encoded this independently — `orbs-tui` as three
    /// `if`s, the Bevy build as an exhaustive `match` — under a comment saying
    /// the two had *to* agree, because a tint that declines in one and
    /// resolves in the other means the two builds disagree about what a fouled
    /// instrument looks like. Nothing enforced it: each build's test re-derived
    /// the same predicate locally and compared the function against a copy of
    /// itself, so each proved only that it agreed with itself.
    ///
    /// Fire is never tinted — it is its own light source, and a green flame is
    /// a different substance rather than a hinted one. Sediment is waste, and
    /// §10.1 gives waste one look so it reads as waste at a glance.
    ///
    /// This is a fact about a *picture*, not about a phosphor or an ANSI index,
    /// which is why it belongs here and not in either resolver. It is the same
    /// rule [`Style::depicted`] enforces for the other channel.
    #[must_use]
    pub const fn declines_tint(self) -> bool {
        self.is_flame() || self.is_spark() || self.is_smoke() || matches!(self, Self::Sediment)
    }
}

/// What part of speech a cell belongs to, when it is a line of a spell.
///
/// # Enrichment, and §14 is the reason it may exist at all
///
/// A spell is a file a player reads, and colouring its parts is a legibility aid
/// — never information. §14 requires the file to read correctly with none of it:
/// the words say what they say, `interpret` reports what the orb heard, and a
/// screen reader hears one sentence per line. Take every colour away and nothing
/// is lost but comfort, which is the test a treatment has to pass to be allowed
/// on screen.
///
/// **It obeys the rule a [`Wash`] and a [`Depiction`] already obey: it declines
/// on an accent.** A line `interpret` could not read is drawn in `Role::Danger`,
/// and the fault is the thing the eye must go to; syntax over the top of it
/// would be decoration winning over meaning, which §4 forbids in one sentence.
///
/// **Unlike those two it is not a field on [`Style`]**, and the reason is
/// measured rather than stylistic: `Cell` is pinned at eight bytes because a
/// `Frame` holds 7,040 of them and resets every frame, and a fifth byte on
/// `Style` cost +28 KiB and ~1.2 µs a frame on *every* screen.
/// `a_cell_stays_eight_bytes` is what said so, by failing.
///
/// It reaches a frontend two ways instead, and neither costs a cell anything.
/// The [`Intensity`] is resolved as the run is painted, so weight is already in
/// the `Style`. The **hue** rides `Frame`'s syntax side-table — one
/// `(Rect, Lexeme)` per run, the same shape [`Wash`] uses for an instrument bar
/// and for the same reason: *"a payload is affordable here and would not be on a
/// `Cell`"*. See [`Frame::lit`](crate::Frame::lit).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lexeme {
    /// Not part of a spell, or a part with nothing to say about it.
    #[default]
    None,
    /// One of the language's own words — `repeat`, `if`, `end`, `part`.
    ///
    /// The scaffolding: where a block opens, where it closes, what bounds a loop.
    Control,
    /// A verb the tower answers to — `grind`, `follow`, `probe`.
    Verb,
    /// Something the tower has — a reagent, an instrument, a place, a reading.
    Name,
    /// A count: `repeat 3`, `has 4 fragment`.
    Number,
    /// A `#` line, which the orb never reads.
    Comment,
    /// A call to one of this spell's own parts — `gathering()`.
    ///
    /// Distinct from a verb because it is a name *this file* defines rather than
    /// one the tower offers, and reading a spell means knowing which is which.
    Call,
    /// A word §6 strips before matching — `the`, `a`, `to`.
    ///
    /// Drawn recessively rather than not at all: the player typed it and the file
    /// is theirs (§19), so it stays on screen and gets out of the way.
    Filler,
    /// A word the question grammar fixes in place — `is`, `has`, `be`, `each`.
    ///
    /// **Not [`Filler`](Self::Filler), and telling them apart is why this
    /// exists.** Both are small words the player types between the interesting
    /// ones, and they are opposites: filler is what the orb *strips*, and this is
    /// what it reads to know which question is being asked. Drawn dim, `is` and
    /// `has` looked exactly like the `the` beside them.
    Grammar,
    /// One of the three states a place reports, or a spelling of one.
    ///
    /// `idle`, `free`, `still`, `working`, `busy`, `running`, `empty`, `bare` —
    /// a closed vocabulary that only ever appears as the answer half of an `is`.
    /// A [`Name`](Self::Name) is something the tower *has*; this is something it
    /// is doing, and a question reads better when the two do not look alike.
    State,
}

impl Lexeme {
    /// How strongly this part of a spell is drawn.
    ///
    /// # Weight is the first axis and still carries the reading on its own
    ///
    /// A spell had this and nothing else for four versions, and §19 records why
    /// hue was added beside it rather than instead of it: **a dump has no
    /// colour**, `ORBS_DUMP` is the project's primary instrument, and a greyscale
    /// tube is a §14 accessibility case. So weight keeps the split it always had,
    /// and the hue is a second, finer cut over the top:
    ///
    /// | | |
    /// |---|---|
    /// | **Bright** | the scaffolding — where a block opens, closes, or calls |
    /// | Normal | the content: what it does, to what, and what it is doing |
    /// | Dim | the noise: filler the orb strips, and comments it never reads |
    ///
    /// [`Grammar`](Self::Grammar) sits at **Normal**, which is the whole of what
    /// it was added for. It reads like filler and is its opposite — filler is
    /// what §6 strips, `is` and `has` are what the question turns on — and drawn
    /// dim beside a `the` there was nothing to tell them apart by.
    ///
    /// # One rule, not one per frontend
    ///
    /// Both builds resolve this the same way and neither gets an opinion, which
    /// is [`Depiction::declines_tint`]'s argument applied again: that one was
    /// three `if`s in `orbs-tui` mirroring an exhaustive `match` in the Bevy
    /// build, under a comment saying the two had to agree, with nothing making
    /// them.
    #[must_use]
    pub const fn weight(self) -> Intensity {
        match self {
            Self::Control | Self::Call => Intensity::Bright,
            Self::Verb | Self::Name | Self::Number | Self::Grammar | Self::State | Self::None => {
                Intensity::Normal
            }
            Self::Comment | Self::Filler => Intensity::Dim,
        }
    }
}

/// The complete semantic style of a cell.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Style {
    /// What the cell means.
    pub role: Role,
    /// How strongly it is drawn within the base hue.
    pub intensity: Intensity,
    /// A treatment applied on top.
    pub presentation: Presentation,
    /// What the cell is a picture of, if anything.
    ///
    /// **Frontends must resolve through [`Style::depicted`], never this field.**
    /// The accessor is where "a depiction never paints over an accent" lives, and
    /// a frontend reading the field directly would draw a `Role::Danger` cell in
    /// flame colours — losing the one accent the game most needs legible.
    pub depiction: Depiction,
}

impl Style {
    /// Body text: no accent, normal weight, drawn faithfully, a picture of
    /// nothing.
    pub const NORMAL: Self = Self {
        role: Role::Normal,
        intensity: Intensity::Normal,
        presentation: Presentation::Plain,
        depiction: Depiction::None,
    };

    /// Recessive text — borders, chrome, anything the eye should skip.
    pub const DIM: Self = Self::NORMAL.with_intensity(Intensity::Dim);

    /// Emphasised text.
    pub const BRIGHT: Self = Self::NORMAL.with_intensity(Intensity::Bright);

    /// Failure, breach, hostile action.
    pub const DANGER: Self = Self::NORMAL.with_role(Role::Danger);

    /// Mana and arcane expenditure.
    pub const COST: Self = Self::NORMAL.with_role(Role::Cost);

    /// Completion.
    pub const SUCCESS: Self = Self::NORMAL.with_role(Role::Success);

    /// This style with a different role.
    #[must_use]
    pub const fn with_role(self, role: Role) -> Self {
        Self { role, ..self }
    }

    /// This style with a different intensity.
    #[must_use]
    pub const fn with_intensity(self, intensity: Intensity) -> Self {
        Self { intensity, ..self }
    }

    /// This style with a different presentation treatment.
    #[must_use]
    pub const fn with_presentation(self, presentation: Presentation) -> Self {
        Self {
            presentation,
            ..self
        }
    }

    /// This style as a picture of something.
    #[must_use]
    pub const fn with_depiction(self, depiction: Depiction) -> Self {
        Self { depiction, ..self }
    }

    /// The depiction a frontend should actually draw — **the only correct way to
    /// read [`Style::depiction`]**.
    ///
    /// Yields [`Depiction::None`] on any cell carrying an accent, whatever the
    /// field says. §4 reserves the accent triad strictly for meaning and a
    /// depiction means nothing, so where the two collide the accent wins; the
    /// alternative is a `Role::Danger` cell rendered in flame colours, which is
    /// a breach drawn as decoration.
    ///
    /// Enforced here rather than at each call site because there is one call site
    /// *per frontend*, in different crates, and the Bevy build getting it right
    /// would say nothing about `orbs-tui`.
    #[must_use]
    pub const fn depicted(self) -> Depiction {
        match self.role {
            Role::Normal => self.depiction,
            Role::Danger | Role::Cost | Role::Success => Depiction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_style_is_plain_body_text() {
        assert_eq!(Style::default(), Style::NORMAL);
    }

    #[test]
    fn builders_compose_without_disturbing_other_channels() {
        let style = Style::DANGER
            .with_intensity(Intensity::Bright)
            .with_presentation(Presentation::Tampered);

        assert_eq!(style.role, Role::Danger);
        assert_eq!(style.intensity, Intensity::Bright);
        assert_eq!(style.presentation, Presentation::Tampered);
    }

    #[test]
    fn nothing_is_a_picture_of_anything_by_default() {
        // A new channel on `Style` must not silently restyle the entire game.
        // Every constant here is drawn on most of the cells on screen.
        for style in [
            Style::NORMAL,
            Style::DIM,
            Style::BRIGHT,
            Style::DANGER,
            Style::COST,
            Style::SUCCESS,
        ] {
            assert_eq!(style.depiction, Depiction::None);
            assert_eq!(style.depicted(), Depiction::None);
        }
    }

    #[test]
    fn every_depiction_is_in_all() {
        // **The match is the test.** Adding a variant to `Depiction` makes this
        // fail to compile, which is the only thing that keeps `ALL` from
        // silently falling behind — and a stale `ALL` is worse than none,
        // because the palette's coverage walk would go on passing while no
        // longer covering everything.
        for depiction in Depiction::ALL {
            let named = match depiction {
                Depiction::None
                | Depiction::FlameEmber
                | Depiction::FlameBody
                | Depiction::FlameBlaze
                | Depiction::FlameCore
                | Depiction::SmokeThin
                | Depiction::SmokeThick
                | Depiction::SparkEmber
                | Depiction::SparkBody
                | Depiction::SparkBlaze
                | Depiction::SparkCore
                | Depiction::LiquidStill
                | Depiction::LiquidStirred
                | Depiction::LiquidRolling
                | Depiction::Sediment => true,
            };
            assert!(named, "{depiction:?}");
        }

        // ...and no duplicates, which a copy-paste into the array would make and
        // the match above would happily allow.
        let mut seen = std::collections::BTreeSet::new();
        for depiction in Depiction::ALL {
            assert!(seen.insert(depiction), "{depiction:?} is in ALL twice");
        }
    }

    #[test]
    fn a_depiction_never_paints_over_an_accent() {
        // §4 reserves the accent triad strictly for meaning, and a depiction
        // means nothing. A `Role::Danger` cell rendered in flame colours is a
        // breach drawn as decoration — so the accessor drops the picture rather
        // than the accent, whatever the field holds.
        let blaze = Depiction::flame(Heat::Blaze);
        assert_eq!(Style::NORMAL.with_depiction(blaze).depicted(), blaze);

        for accent in [Style::DANGER, Style::COST, Style::SUCCESS] {
            assert_eq!(
                accent.with_depiction(blaze).depicted(),
                Depiction::None,
                "{:?} let a depiction overwrite it",
                accent.role,
            );
        }
    }

    #[test]
    fn accent_constants_keep_default_intensity() {
        // The accent triad must read against the base hue at ordinary weight;
        // brightening them would make the accent a second signal.
        for style in [Style::DANGER, Style::COST, Style::SUCCESS] {
            assert_eq!(style.intensity, Intensity::Normal);
            assert_eq!(style.presentation, Presentation::Plain);
        }
    }
}
