//! The greyscale accommodation, as a setting both builds read (§14).
//!
//! What each build does with it is its own — the window runs a shader pass, the
//! terminal sends no hue — but what it is called, what it can be and how it
//! opens are one copy.

/// Where the accommodation is read from before the settings file.
pub const SIGHT_VAR: &str = "ORBS_SIGHT";

/// How the player has asked to be shown the screen.
///
/// Not a theme, which is the reversal §19 records: a theme made the
/// accommodation an aesthetic choice, so a player who needed it had to give up
/// amber. This is orthogonal to the phosphor.
///
/// Two variants where Phase 13 asks for five. The three deficiency corrections
/// are left out on measurement: daltonisation degrades the window's accent
/// separation in eleven of twelve theme × deficiency combinations, because the
/// triad is solved in *luminance* and the correction works in *hue* (§19).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sight {
    /// The screen as authored.
    #[default]
    Plain,
    /// Every hue gone.
    ///
    /// The honest accommodation, and the one that *proves* §14's claim: if the
    /// game plays with no hue at all, then no hue was carrying meaning.
    Greyscale,
}

impl Sight {
    /// Every setting, in the order a settings page offers them and `F8` steps.
    pub const ALL: [Self; 2] = [Self::Plain, Self::Greyscale];

    /// What this is called, for a settings row and for `ORBS_SIGHT`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Greyscale => "greyscale",
        }
    }

    /// The setting `word` names, if it names one.
    #[must_use]
    pub fn parse(word: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|sight| sight.name().eq_ignore_ascii_case(word.trim()))
    }

    /// The next in `F8`'s order, coming back round.
    #[must_use]
    pub fn next(self) -> Self {
        let at = Self::ALL.iter().position(|sight| *sight == self);
        Self::ALL[(at.unwrap_or(0) + 1) % Self::ALL.len()]
    }

    /// What the accommodation opens as: `ORBS_SIGHT`, then what the player
    /// chose.
    #[must_use]
    pub fn seeded() -> Self {
        chosen(
            // An exported-but-empty `ORBS_SIGHT=` is not the switch being set —
            // `save::chosen`'s rule. Discarding a *saved accessibility choice*
            // over an empty variable is the worst version of it.
            std::env::var(SIGHT_VAR)
                .ok()
                .filter(|value| !value.trim().is_empty())
                .or_else(|| crate::settings::get(crate::settings::SIGHT))
                .as_deref(),
        )
    }
}

/// The rule [`Sight::seeded`] applies, without the environment, so it can be
/// tested: a test that set the variable would set it for every other test.
fn chosen(value: Option<&str>) -> Sight {
    // An exported-but-empty variable is *unset*, not a typo. `ORBS_SIGHT= orbs`
    // is the shell idiom for neutralising one, and warning on it would put a
    // line in the log on every launch.
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Sight::Plain;
    };
    let Some(sight) = Sight::parse(value) else {
        // Said rather than silently ignored: a player who mistypes an
        // accommodation should not have to wonder whether it took.
        tracing::warn!("{SIGHT_VAR}: no such sight {value:?} — the screen is unchanged");
        return Sight::Plain;
    };
    sight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unset_variable_leaves_the_screen_alone() {
        assert_eq!(chosen(None), Sight::Plain);
    }

    #[test]
    fn a_named_sight_is_taken() {
        assert_eq!(chosen(Some("greyscale")), Sight::Greyscale);
        assert_eq!(chosen(Some(" greyscale ")), Sight::Greyscale);
        assert_eq!(chosen(Some("GREYSCALE")), Sight::Greyscale);
    }

    #[test]
    fn a_typo_falls_back_rather_than_panicking() {
        assert_eq!(chosen(Some("grayscale")), Sight::Plain);
    }

    /// An exported-but-empty variable is unset, and must not warn.
    ///
    /// The window's `crt::chosen` has the twin of this test; the two must stay
    /// in step.
    #[test]
    fn a_blank_reads_as_unset_rather_than_as_a_typo() {
        assert_eq!(chosen(Some("")), Sight::Plain);
        assert_eq!(chosen(Some("   ")), Sight::Plain);
    }

    /// `F8` must reach every accommodation and come back round. A key that
    /// cannot reach the state a player needs is not a control.
    #[test]
    fn cycling_reaches_every_sight_and_returns() {
        let mut seen = Vec::new();
        let mut at = Sight::Plain;
        for _ in 0..Sight::ALL.len() {
            seen.push(at);
            at = at.next();
        }
        assert_eq!(
            at,
            Sight::Plain,
            "the cycle does not return to where it began"
        );
        for sight in Sight::ALL {
            assert!(seen.contains(&sight), "{} is unreachable", sight.name());
        }
    }

    #[test]
    fn every_name_round_trips_and_a_typo_does_not() {
        for sight in Sight::ALL {
            assert_eq!(Sight::parse(sight.name()), Some(sight));
        }
        assert_eq!(Sight::parse("GREYSCALE"), Some(Sight::Greyscale));
        assert_eq!(Sight::parse("grayscale"), None, "a near miss must not land");
        assert_eq!(Sight::parse(""), None);
    }
}
