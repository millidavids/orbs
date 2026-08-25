//! Turning a [`Lexeme`] into a weight, for the two surfaces that highlight.
//!
//! The classification is `orbs-sim`'s — see `parser::lexeme`, and §19 on why a
//! painter must not have a second opinion about the grammar. What is decided
//! here is the other half of rule 2: **how much of the row's own styling a run
//! is allowed to override**, which is a presentation question and belongs on
//! this side of the boundary.
//!
//! It lived in `sheet` while the spell editor was the only surface that
//! highlighted. The prompt is the second.

use orbs_render::{Intensity, Lexeme, Role, Style};

/// One run's style: the row's, with the lexeme's weight if the row has none of
/// its own.
///
/// # The row outranks the run, one level below the accent
///
/// `Style::lexed` already drops the lexeme on an accented cell, so a line
/// `interpret` could not read stays wholly `Role::Danger`. This is the same
/// arbitration a step further in: the line the orb is **executing** is drawn
/// `Intensity::Bright` to say so, and a `the` inside it dimming back down would
/// break the one row-wide signal the gutter marker is paired with.
///
/// So a lexeme's weight applies only where the row is at normal weight — which
/// is every line except the running one.
///
/// This is `Style::depicted`'s rule written at the caller rather than on the
/// type, and deliberately: a fourth field on `Style` costs every cell of every
/// frame — measured at +28 KiB and ~1.2 µs — for a fact two surfaces read.
pub(crate) fn lit(style: Style, kind: Lexeme) -> Style {
    if !takes_syntax(style) {
        return style;
    }
    style.with_intensity(kind.weight())
}

/// Whether a row lets its runs style themselves at all.
///
/// # Ask this, never `lit(style, kind) != style`
///
/// The obvious test is wrong in a way that is easy to ship: `lit` also returns
/// the style unchanged when the run's weight already **matches** the row's,
/// which is every ordinary [`Lexeme::Name`] on an ordinary line. A caller
/// gating on inequality would colour the control words and the filler and
/// silently skip the names — most of a spell.
///
/// So the question is about the *row*, and it is asked in two places now: `lit`
/// resolves the weight, and the painter decides whether to register the hue.
/// They must agree, or a line the orb could not read would keep its red and gain
/// a syntax colour on top of it.
pub(crate) fn takes_syntax(style: Style) -> bool {
    style.role == Role::Normal && style.intensity == Intensity::Normal
}

/// What a lexeme honestly is at the **prompt**, which cannot run a spell.
///
/// # Four of them mean nothing here, and drawing them would lie
///
/// `lex` is lexical and world-free, so it reads `repeat` as a control word,
/// `gathering()` as a call, `is` as grammar and `idle` as a state wherever it
/// finds them. In a spell all four are right. At the prompt none can run:
/// `repeat` is not a verb, a part is a thing only a `.spell` file defines, and
/// a prompt cannot ask a question — there is no line at the prompt where `is
/// idle` is anything but two words the parser will not place.
///
/// Drawing them in their own colours — or `repeat` bright, the weight that says
/// *the orb knows this word* — would be the one thing §14 forbids highlighting
/// to do, which is to carry information. And the information would be false. So
/// they fall back to what any unrecognised word gets, and the player finds out
/// what the orb made of the line from the echo, as they do for every other word.
///
/// **`Verb`, `Name`, `Number`, `Filler` and `Comment` are shared**, because all
/// five mean the same thing in both places: §6's parser strips the same filler
/// and answers to the same verbs whether the line was typed or scribed.
pub(crate) const fn at_prompt(kind: Lexeme) -> Lexeme {
    match kind {
        Lexeme::Control | Lexeme::Call | Lexeme::Grammar | Lexeme::State => Lexeme::None,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run takes the lexeme's weight where the row has none of its own.
    #[test]
    fn a_plain_row_takes_the_runs_weight() {
        assert_eq!(
            lit(Style::default(), Lexeme::Control).intensity,
            Intensity::Bright,
        );
        assert_eq!(
            lit(Style::default(), Lexeme::Filler).intensity,
            Intensity::Dim,
        );
    }

    /// ...and an accented or already-weighted row keeps its own.
    ///
    /// **A fault outranks the highlighting.** §4 reserves the accent triad
    /// strictly for meaning, so decoration may not paint over it — the rule
    /// `Style::depicted` already applies to pictures, here applied to syntax.
    /// Without it a `repeat` inside an unreadable line would draw bright instead
    /// of red, and the one signal saying *this line is broken* would be the one
    /// the eye skipped.
    ///
    /// **So does the row the orb is standing on**, which is drawn bright to say
    /// so; a `the` inside it dimming back down would break the one row-wide
    /// signal the gutter marker is paired with.
    #[test]
    fn the_row_outranks_the_run() {
        let fault = Style::default().with_role(Role::Danger);
        for kind in [Lexeme::Control, Lexeme::Filler] {
            assert_eq!(lit(fault, kind), fault, "a fault lost to {kind:?}");
        }

        let running = Style::default().with_intensity(Intensity::Bright);
        for kind in [Lexeme::Filler, Lexeme::Comment] {
            assert_eq!(
                lit(running, kind),
                running,
                "the executing line lost to {kind:?}",
            );
        }
    }

    /// The prompt cannot run a spell, so it does not draw a spell's words.
    ///
    /// Four of the eight: the scaffolding, a call to a part, and both halves of
    /// a question. None can run at a prompt, and drawing them in their own
    /// colour would say the orb understood something it is about to refuse.
    #[test]
    fn the_prompt_does_not_draw_a_word_it_cannot_run() {
        for spell_only in [
            Lexeme::Control,
            Lexeme::Call,
            Lexeme::Grammar,
            Lexeme::State,
        ] {
            assert_eq!(
                at_prompt(spell_only),
                Lexeme::None,
                "{spell_only:?} kept its meaning at a prompt that has none for it",
            );
            assert_eq!(
                lit(Style::default(), at_prompt(spell_only)).intensity,
                Intensity::Normal,
                "{spell_only:?} was drawn as scaffolding at the prompt",
            );
        }
        // Everything else is the same answer in both places: §6's parser strips
        // the same filler and answers to the same verbs whether the line was
        // typed or scribed.
        for shared in [
            Lexeme::Verb,
            Lexeme::Name,
            Lexeme::Number,
            Lexeme::Filler,
            Lexeme::Comment,
        ] {
            assert_eq!(
                at_prompt(shared),
                shared,
                "{shared:?} was reinterpreted at the prompt",
            );
        }
    }
}
