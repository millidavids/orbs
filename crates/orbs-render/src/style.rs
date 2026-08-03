//! Semantic styling — meaning, never colour.
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

/// The complete semantic style of a cell.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Style {
    /// What the cell means.
    pub role: Role,
    /// How strongly it is drawn within the base hue.
    pub intensity: Intensity,
    /// A treatment applied on top.
    pub presentation: Presentation,
}

impl Style {
    /// Body text: no accent, normal weight, drawn faithfully.
    pub const NORMAL: Self = Self {
        role: Role::Normal,
        intensity: Intensity::Normal,
        presentation: Presentation::Plain,
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
    fn accent_constants_keep_default_intensity() {
        // The accent triad must read against the base hue at ordinary weight;
        // brightening them would make the accent a second signal.
        for style in [Style::DANGER, Style::COST, Style::SUCCESS] {
            assert_eq!(style.intensity, Intensity::Normal);
            assert_eq!(style.presentation, Presentation::Plain);
        }
    }
}
