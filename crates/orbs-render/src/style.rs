//! Semantic styling — and one channel that is deliberately not.
//!
//! A [`Style`] says what a cell *means*. Turning that into a phosphor hue, an
//! ANSI index or a glyph variant is the frontend's job, and each does it
//! differently — curated phosphor themes in the Bevy build, the user's own
//! terminal's ANSI indices in the other (DESIGN.md §13). A concrete colour must
//! never appear in this crate.
//!
//! Two design rules the types enforce rather than discipline:
//!
//! - Colour never carries meaning alone (§14). [`Role`] rides into the
//!   screen-reader stream beside the text, so meaning survives with no pixels.
//! - Eldritch and sabotage vocabularies are disjoint (§3). [`Presentation`] is
//!   an enum, so the tonal system cannot jam the diagnostic one at peak threat.
//!
//! [`Depiction`] is the exception: it selects a colour ramp and says nothing, so
//! the athanor's meter is a picture of a fire rather than a reading of one. §14
//! forbids colour being the *sole* carrier of meaning, and a channel carrying
//! none has nothing to withhold from a listener — hence the test that it never
//! reaches the linear stream, and [`Style::depicted`] refusing to paint over an
//! accent.

/// What a cell *means*, from the accent set reserved strictly for meaning.
///
/// DESIGN.md §4: the base hue carries all ordinary text through intensity
/// variation alone, and accents are never decorative.
///
/// Deliberately not `#[non_exhaustive]`: every consumer is in this workspace,
/// and a frontend that gains a new role must be made to handle it — a missed arm
/// renders danger as ordinary text.
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
/// An enum, not flags: the two non-plain variants are mutually exclusive by
/// design rule (§3).
///
/// Its own channel rather than folded into [`Role`] because the Bevy frontend
/// draws each variant from a different face of `assets/fonts/unscii/` —
/// `unscii-16` for [`Plain`](Self::Plain), `unscii-8-fantasy` for
/// [`Eldritch`](Self::Eldritch), `unscii-8-mcr` for
/// [`Tampered`](Self::Tampered), the last two row-doubled from 8×8. All three
/// share metrics and repertoire, so a face swap can never move a cell. The
/// terminal frontend renders all three identically — accepted degradation
/// (§8.1), since `verify` is the authoritative detector on every surface.
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
    /// A bonus channel, never the only one — `verify` is the authoritative
    /// detector everywhere, so the terminal build not controlling its font is
    /// degradation rather than breakage.
    Tampered,
}

/// How hot a flame or spark cell is — see [`Depiction::flame`] and
/// [`Depiction::spark`].
///
/// Four steps, not the three an earlier design had. That argument — CP437 gives
/// flame two glyphs, so a longer ramp is colour with nothing under it — stopped
/// holding once the bottom half of the fire became solid `█` by construction,
/// carrying *all* of its motion in hue. The ramp works alone there, and a coarse
/// one reads as a two-tone flicker rather than a glow.
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

/// How far along a gauge's fill a cell sits — see [`Depiction::gauge`].
///
/// The one hue ramp in a game of brightness ramps: red through yellow to green,
/// brightest in the middle, so it has no monotonic-brightness test and could not
/// pass one. Fire, liquid and smoke depict a *substance getting more intense*;
/// this depicts a *distance being closed*, which the eye reads as a journey
/// rather than a temperature.
///
/// Colour is allowed (§14) because the fill length is the information and the
/// hue only reinforces it: the bracket, the pipes and the reading beside it
/// still say how full a gauge is, so in greyscale nothing is lost. Six steps —
/// three would read as a traffic light changing rather than a bar warming, and
/// six is as many as a sixteen-colour terminal can tell apart, so both frontends
/// draw the whole ramp.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fill {
    /// Barely begun.
    #[default]
    Faint,
    /// Under way.
    Low,
    /// About half.
    Middle,
    /// More done than not.
    High,
    /// Nearly there.
    Near,
    /// Full.
    Whole,
}

impl Fill {
    /// Every step, lowest first. Kept honest by a test rather than by memory.
    pub const ALL: [Self; 6] = [
        Self::Faint,
        Self::Low,
        Self::Middle,
        Self::High,
        Self::Near,
        Self::Whole,
    ];

    /// Which step `done` out of `total` sits on.
    ///
    /// Integer throughout, like every measurement the render path makes: a float
    /// would be one more thing replay has to trust. `total` of nought is
    /// [`Faint`](Self::Faint) — a gauge measuring nothing has not started.
    #[must_use]
    pub const fn of(done: u32, total: u32) -> Self {
        if total == 0 {
            return Self::Faint;
        }
        // `Whole` is reserved for actually full, and the five below it split
        // what is left evenly. An even sixth at the top would paint a bar one
        // short of its tier the same green as one that had arrived.
        if done >= total {
            return Self::Whole;
        }
        match (done as u64 * 5) / total as u64 {
            0 => Self::Faint,
            1 => Self::Low,
            2 => Self::Middle,
            3 => Self::High,
            _ => Self::Near,
        }
    }
}

/// The colour family a material draws its instrument's bar in.
///
/// A name, never a value: `Green` says *which family*, and what green is on a
/// given tube is the frontend's answer, as it is for [`Role`]. `orbs-tui`
/// resolving the same eight names to ANSI indices in `theme::hue` is fine
/// because the set is closed — eight names, eight hues, no fallback, so a ninth
/// tint breaks that build on purpose.
///
/// A tint hints at what is inside an instrument — sage grinds green, its husks
/// are brown — *over* `survey` rather than instead of it. Three things keep it
/// inside §14: `survey <instrument>` names the contents on every frontend; the
/// panel names the instrument and its state, both of which reach the linear
/// stream; and a tint never paints over an accent, so `Fouled` keeps its
/// `Role::Danger` label, for the same reason [`Style::depicted`] drops a picture
/// on an accented cell. DESIGN.md §19 records the widening and the Phase 13 item
/// carrying the accessibility promise it moved.
///
/// Deliberately not `#[non_exhaustive]`, matching [`Role`] and [`Depiction`]: a
/// frontend that gains a tint must be made to handle it rather than silently
/// drawing a material in the base hue. Authored data selects one *by name* and
/// an unknown name is a load error, not a silent fallback — the defect
/// `Recipe::heat` and `craft_of` each already paid for.
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
    /// `None` is for the caller to report, not to substitute a default for: a
    /// typo drawing the base hue looks exactly like an untinted material.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|tint| tint.name() == name.trim().to_ascii_lowercase())
    }
}

/// A region's colour, which is one family or two being combined.
///
/// A payload is affordable here and would not be on a [`Cell`](crate::Cell):
/// this lives in the `Frame`'s tint side-table, one entry per instrument bar, so
/// it costs a handful of words a frame rather than a byte per grid position.
/// That is why the tint went on the `Frame` rather than into [`Style`], and it
/// is what lets the flask say what the other instruments cannot — two materials
/// on their way to becoming one.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Wash {
    /// The family this region draws in.
    pub tint: Tint,
    /// A second, mixed evenly with the first.
    ///
    /// `Some` only for the `flask_and_rod` mid-combination, where the growing
    /// portion of the bar is the two inputs becoming one thing. The frontend
    /// averages the two families rather than picking a third: a third colour
    /// reads as a *substitution*, and the picture is about a *mixture*.
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
    /// The flask's two inputs are a set, so order must not matter here either:
    /// `mix(a, b)` drawing differently from `mix(b, a)` would make the bar
    /// depend on which reagent the player happened to put in first.
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
/// The balneum mariae's bar is a level of liquid and all of its motion is here:
/// the glyph never changes, so the value survives the colour being thrown away
/// and nothing in the picture can be mistaken for a bubble. Three steps, since
/// one solid glyph leaves the ramp working alone — two would read as a two-tone
/// flicker, a fourth is invisible at this brightness.
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
/// Two steps, on the argument [`Heat`] was once held to: smoke has `░` and `▒`
/// and nothing else, so a longer ramp would be colour with nothing under it.
/// The flame's bottom half became glyph-constant and escaped it; smoke has not.
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
/// The only thing in this module carrying no meaning: it is spoken nowhere, and
/// a frontend ignoring it entirely would lose a picture and no information. See
/// the module header for why that makes it safe.
///
/// Deliberately not `#[non_exhaustive]`, matching [`Role`]: a frontend that
/// gains a depiction must be made to handle it rather than silently drawing it
/// as ordinary text.
///
/// Flat rather than the obvious `Flame(Heat)`/`Smoke(Density)`/`Spark(Heat)`,
/// because that is a two-byte tagged enum: it pushes [`Style`] from 3 bytes to 5
/// and, past `char`'s 4-byte alignment, [`Cell`](crate::Cell) from 8 to 12.
/// `Frame::reset` memsets the whole grid every frame, so every screen pays
/// +1.2 µs and +28 KiB at 160×44 for a picture occupying thirty cells of one
/// panel. Spelled out, the discriminant is one byte. [`Depiction::flame`] and
/// friends keep the nested shape where it reads better.
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
    /// Their own variants rather than flame in the plume: they draw from the
    /// small marks `∙ ° ·` rather than the shade blocks, and folding them in
    /// would cost the invariant that a flame cell is one of two glyphs. Same
    /// ramp, though — a spark is a piece of the fire, so tuning one tunes both.
    SparkEmber,
    /// A spark at [`Heat::Flame`].
    SparkBody,
    /// A spark at [`Heat::Blaze`].
    SparkBlaze,
    /// A spark at [`Heat::Core`].
    SparkCore,
    /// A gauge's fill at [`Fill::Faint`].
    GaugeFaint,
    /// A gauge's fill at [`Fill::Low`].
    GaugeLow,
    /// A gauge's fill at [`Fill::Middle`].
    GaugeMiddle,
    /// A gauge's fill at [`Fill::High`].
    GaugeHigh,
    /// A gauge's fill at [`Fill::Near`].
    GaugeNear,
    /// A gauge's fill at [`Fill::Whole`].
    GaugeWhole,
    /// Liquid at [`Roil::Still`].
    LiquidStill,
    /// Liquid at [`Roil::Stirred`].
    LiquidStirred,
    /// Liquid at [`Roil::Rolling`].
    LiquidRolling,
    /// What a run has left settled at the bottom of a vessel.
    ///
    /// Its own step rather than the bottom of the liquid ramp: a bath holding
    /// nothing but the last run's grit must not read as one holding a little
    /// liquid.
    Sediment,
}

impl Depiction {
    /// Every variant, for consumers that must handle the whole set.
    ///
    /// `orbs`'s palette walks this to prove every depiction resolves to a
    /// colour. That walk used to be a hand-written array in the consumer, which
    /// falls behind the enum the moment a variant is added; the test below fails
    /// to *compile* if this misses one.
    pub const ALL: [Self; 21] = [
        Self::None,
        Self::GaugeFaint,
        Self::GaugeLow,
        Self::GaugeMiddle,
        Self::GaugeHigh,
        Self::GaugeNear,
        Self::GaugeWhole,
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

    /// A gauge's fill at a given step.
    #[must_use]
    pub const fn gauge(fill: Fill) -> Self {
        match fill {
            Fill::Faint => Self::GaugeFaint,
            Fill::Low => Self::GaugeLow,
            Fill::Middle => Self::GaugeMiddle,
            Fill::High => Self::GaugeHigh,
            Fill::Near => Self::GaugeNear,
            Fill::Whole => Self::GaugeWhole,
        }
    }

    /// Whether this is a gauge's fill.
    #[must_use]
    pub const fn is_gauge(self) -> bool {
        matches!(
            self,
            Self::GaugeFaint
                | Self::GaugeLow
                | Self::GaugeMiddle
                | Self::GaugeHigh
                | Self::GaugeNear
                | Self::GaugeWhole
        )
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
            | Self::Sediment
            | Self::GaugeFaint
            | Self::GaugeLow
            | Self::GaugeMiddle
            | Self::GaugeHigh
            | Self::GaugeNear
            | Self::GaugeWhole => None,
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
    /// One rule, in the crate that owns the vocabulary. Both frontends encoded
    /// it independently, each proving only that it agreed with itself, while a
    /// tint declining in one and resolving in the other means the two builds
    /// disagree about what a fouled instrument looks like.
    ///
    /// Fire is never tinted — it is its own light source, and a green flame is a
    /// different substance rather than a hinted one. Sediment is waste, and
    /// §10.1 gives waste one look so it reads as waste at a glance.
    ///
    /// A fact about a *picture* rather than a phosphor or an ANSI index, which
    /// is why it belongs here and not in either resolver — the same rule
    /// [`Style::depicted`] enforces for the other channel.
    #[must_use]
    pub const fn declines_tint(self) -> bool {
        // A gauge is not a material either: it stands at the top of the pane, so
        // a tint it picked up would say the fill meant something about sage.
        self.is_flame()
            || self.is_spark()
            || self.is_smoke()
            || self.is_gauge()
            || matches!(self, Self::Sediment)
    }
}

/// What part of speech a cell belongs to, when it is a line of a spell.
///
/// Enrichment, allowed by §14 because the file reads correctly with none of it:
/// the words say what they say, `interpret` reports what the orb heard, and a
/// screen reader hears one sentence per line. Take the colour away and only
/// comfort is lost.
///
/// It declines on an accent, as a [`Wash`] and a [`Depiction`] do. A line
/// `interpret` could not read is drawn in `Role::Danger` and is where the eye
/// must go; syntax over the top would be decoration winning over meaning (§4).
///
/// Unlike those two it is not a field on [`Style`], for a measured reason:
/// `Cell` is pinned at eight bytes because a `Frame` holds 7,040 of them and
/// resets every frame, and a fifth byte on `Style` cost +28 KiB and ~1.2 µs a
/// frame on *every* screen — `a_cell_stays_eight_bytes` said so by failing.
///
/// It reaches a frontend two other ways, neither costing a cell anything. The
/// [`Intensity`] is resolved as the run is painted, so weight is already in the
/// `Style`; the hue rides `Frame`'s syntax side-table, one `(Rect, Lexeme)` per
/// run, the shape [`Wash`] uses for an instrument bar and for the same reason.
/// See [`Frame::lit`](crate::Frame::lit).
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
    /// Not [`Filler`](Self::Filler), which is its opposite: filler is what the
    /// orb *strips*, this is what it reads to know which question is being
    /// asked. Drawn dim, `is` and `has` looked like the `the` beside them.
    Grammar,
    /// One of the three states a place reports, or a spelling of one.
    ///
    /// `idle`, `free`, `still`, `working`, `busy`, `running`, `empty`, `bare` —
    /// a closed vocabulary appearing only as the answer half of an `is`. A
    /// [`Name`](Self::Name) is something the tower *has*; this is something it
    /// is doing, and a question reads better when the two do not look alike.
    State,
}

impl Lexeme {
    /// How strongly this part of a spell is drawn.
    ///
    /// Weight is the first axis and still carries the reading alone — a dump has
    /// no colour, `ORBS_DUMP` is the project's primary instrument, and a
    /// greyscale tube is a §14 accessibility case — so §19 records hue being
    /// added beside it rather than instead of it. Bright is the scaffolding
    /// (where a block opens, closes or calls), Normal the content, Dim the noise
    /// the orb strips or never reads.
    ///
    /// [`Grammar`](Self::Grammar) sits at Normal, which is what it was added
    /// for: drawn dim beside a `the` there was nothing to tell them apart by.
    ///
    /// One rule, not one per frontend — [`Depiction::declines_tint`]'s argument
    /// again.
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
    /// Frontends must resolve through [`Style::depicted`], never this field: the
    /// accessor is where "a depiction never paints over an accent" lives, and
    /// reading the field directly draws a `Role::Danger` cell in flame colours.
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

    /// The depiction a frontend should draw — the only correct way to read
    /// [`Style::depiction`].
    ///
    /// Yields [`Depiction::None`] on any cell carrying an accent, whatever the
    /// field says: §4 reserves the triad strictly for meaning and a depiction
    /// means nothing, so the accent wins rather than a breach being drawn as
    /// decoration.
    ///
    /// Enforced here rather than at each call site because there is one call
    /// site *per frontend*, in different crates, and the Bevy build getting it
    /// right would say nothing about `orbs-tui`.
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
        // The match is the test: a new variant fails to compile here. A stale
        // `ALL` is worse than none — the palette's coverage walk would go on
        // passing while no longer covering everything.
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
                | Depiction::Sediment
                | Depiction::GaugeFaint
                | Depiction::GaugeLow
                | Depiction::GaugeMiddle
                | Depiction::GaugeHigh
                | Depiction::GaugeNear
                | Depiction::GaugeWhole => true,
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
    fn a_gauge_warms_evenly_and_only_a_full_bar_reads_full() {
        // Properties over the whole range, not hand-picked indices: a spot check
        // would pass on a ramp that jumped somewhere nobody looked.
        let steps: Vec<Fill> = (0..=100).map(|done| Fill::of(done, 100)).collect();

        // Monotonic: a bar that filled further must never cool.
        assert!(steps.windows(2).all(|pair| pair[0] <= pair[1]));

        // Every step is reached, so no colour in the ramp is unreachable.
        let seen: std::collections::BTreeSet<Fill> = steps.iter().copied().collect();
        assert_eq!(
            seen.len(),
            Fill::ALL.len(),
            "a step never appears: {seen:?}"
        );

        // The ends say what they mean.
        assert_eq!(steps[0], Fill::Faint);
        assert_eq!(steps[100], Fill::Whole);

        // A bar one short of full must not read as finished — green is the
        // arrival, not the approach.
        assert_eq!(Fill::of(99, 100), Fill::Near);
        assert_eq!(Fill::of(11, 12), Fill::Near);

        // The five below full split evenly, so no step is a sliver.
        let widest = Fill::ALL
            .iter()
            .map(|step| steps.iter().filter(|seen| *seen == step).count())
            .max()
            .unwrap_or_default();
        assert!(widest <= 21, "one step swallowed the ramp: {widest}");

        // Degenerate inputs answer rather than panic: nothing measured has not
        // started, and past full is full.
        assert_eq!(Fill::of(0, 0), Fill::Faint);
        assert_eq!(Fill::of(99, 12), Fill::Whole);
    }

    #[test]
    fn every_fill_is_a_depiction_of_its_own() {
        // A ramp with two steps resolving to one picture would be a bar that
        // stalled at one colour while the number underneath it moved.
        let seen: std::collections::BTreeSet<Depiction> =
            Fill::ALL.into_iter().map(Depiction::gauge).collect();
        assert_eq!(seen.len(), Fill::ALL.len());
        assert!(seen.iter().all(|depiction| depiction.is_gauge()));
        // And a gauge is never mistaken for the fire, which shares no step.
        assert!(seen.iter().all(|depiction| depiction.heat().is_none()));
    }

    #[test]
    fn a_depiction_never_paints_over_an_accent() {
        // §4 reserves the triad strictly for meaning, so the accessor drops the
        // picture rather than the accent, whatever the field holds.
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
