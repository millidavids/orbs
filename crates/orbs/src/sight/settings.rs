//! What the accommodation is set to, and the uniform it becomes.

use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

/// The player's chosen accommodation (DESIGN.md §14).
///
/// A resource rather than a component on the camera, unlike `CrtSettings`: it is
/// a *setting*, one per session, and nothing in the world may write it.
/// `CrtSettings` is driven by world state (§4) and this must never be.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct Vision(pub(crate) Sight);

/// Where the accommodation is read from before there is a settings screen.
const VAR: &str = "ORBS_SIGHT";

/// How the player has asked to be shown the screen.
///
/// Not a theme, which is the reversal §19 records: a theme made the
/// accommodation an aesthetic choice, so a player who needed it had to give up
/// amber. This is orthogonal to the phosphor.
///
/// Two variants where Phase 13 asks for five. The three deficiency corrections
/// are left out on measurement: daltonisation degrades this palette's accent
/// separation in eleven of twelve theme × deficiency combinations, because the
/// triad is solved in *luminance* and the correction works in *hue* (§19).
/// `render::deficiency` keeps the simulation half.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Sight {
    /// The tube as authored.
    #[default]
    Plain,
    /// Every hue gone.
    ///
    /// The honest accommodation, and the one that *proves* §14's claim: if the
    /// game plays with no hue at all, then no hue was carrying meaning.
    Greyscale,
}

impl Sight {
    /// Every setting, in the order a settings screen offers them.
    pub(crate) const ALL: [Self; 2] = [Self::Plain, Self::Greyscale];

    /// What this is called, for a settings row and for `ORBS_SIGHT`.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Greyscale => "greyscale",
        }
    }

    /// The setting `word` names, if it names one.
    pub(crate) fn parse(word: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|sight| sight.name().eq_ignore_ascii_case(word.trim()))
    }
}

/// What the shader is told.
///
/// Field order must match `sight.wgsl`'s `Sight` struct — the hand-maintained
/// contract `CrtUniform` carries. Nothing checks it, and a mismatch is a silent
/// wrong picture rather than a compile error.
#[derive(ShaderType, Debug, Clone, Copy, PartialEq)]
pub(crate) struct SightUniform {
    /// 1.0 to take the hue out, 0.0 to leave it.
    pub(crate) greyscale: f32,
    pub(crate) _pad0: f32,
    pub(crate) _pad1: f32,
    pub(crate) _pad2: f32,
}

impl SightUniform {
    /// Build the uniform for a setting.
    ///
    /// Takes a [`Sight`] and nothing else, so it cannot see `CrtSettings` — an
    /// accommodation the tube's own off-switch could reach is a defect §19
    /// already records once.
    pub(crate) const fn new(sight: Sight) -> Self {
        Self {
            greyscale: match sight {
                Sight::Plain => 0.0,
                Sight::Greyscale => 1.0,
            },
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }

    /// Whether this pass would change anything.
    ///
    /// The pass is skipped when it would not, so the ordinary case costs no
    /// fullscreen ping-pong at all.
    pub(crate) fn wanted(self) -> bool {
        self.greyscale > 0.0
    }
}

/// Step through the accommodations — `F8`.
///
/// The `F2`/`F3` shape, and here for the same reason §14 gives for those: an
/// accommodation a player cannot reach is not an accommodation. Phase 13's
/// settings screen replaces all three.
pub(super) fn cycle(mut vision: ResMut<Vision>) {
    let at = Sight::ALL.iter().position(|sight| *sight == vision.0);
    let next = Sight::ALL[(at.unwrap_or(0) + 1) % Sight::ALL.len()];
    vision.0 = next;
    info!("sight: {}", next.name());
    // The key and the settings page are one setting seen twice — see
    // `crt::plugin::cycle`. An accommodation that did not survive a relaunch
    // would be the worst one to leave un-sticky.
    crate::shell::remember_setting(crate::shell::SIGHT, next.name());
}

/// What the accommodation opens as: the switch, then what the player chose.
///
/// The chain `crt::seeded` uses and for the same reason — the two accommodation
/// switches must behave alike.
pub(super) fn seeded() -> Sight {
    chosen(
        // An exported-but-empty `ORBS_SIGHT=` is not the switch being set —
        // `save::chosen`'s rule. Discarding a *saved accessibility choice* over
        // an empty variable is the worst version of it.
        std::env::var(VAR)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| orbs_shell::settings::get(crate::shell::SIGHT))
            .as_deref(),
    )
}

/// The rule [`seeded`] applies, without the environment.
///
/// Split out so it can be *tested*: a test that set the variable would set it
/// for every other test in the binary. `crt::chosen` is the sibling — the two
/// must agree about case and about blanks.
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
        warn!("{VAR}: no such sight {value:?} — the tube is unchanged");
        return Sight::Plain;
    };
    sight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unset_variable_leaves_the_tube_alone() {
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
    /// `crt::chosen` has the twin of this test; the two must stay in step.
    #[test]
    fn a_blank_reads_as_unset_rather_than_as_a_typo() {
        assert_eq!(chosen(Some("")), Sight::Plain);
        assert_eq!(chosen(Some("   ")), Sight::Plain);
    }

    /// `F8` must reach every accommodation and come back round.
    ///
    /// A key that cannot reach the state a player needs is not a control.
    /// Asserted on the table rather than through the system, which takes a
    /// `ResMut` a unit test cannot build.
    #[test]
    fn cycling_reaches_every_sight_and_returns() {
        let mut seen = Vec::new();
        let mut at = Sight::Plain;
        for _ in 0..Sight::ALL.len() {
            seen.push(at);
            let index = Sight::ALL
                .iter()
                .position(|s| *s == at)
                .expect("in the table");
            at = Sight::ALL[(index + 1) % Sight::ALL.len()];
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
    fn plain_is_the_identity_and_costs_nothing() {
        let uniform = SightUniform::new(Sight::Plain);
        assert_eq!(uniform.greyscale, 0.0);
        assert!(
            !uniform.wanted(),
            "the ordinary case must not run a fullscreen pass"
        );
    }

    #[test]
    fn greyscale_takes_all_of_it() {
        assert_eq!(SightUniform::new(Sight::Greyscale).greyscale, 1.0);
        assert!(SightUniform::new(Sight::Greyscale).wanted());
    }

    // No test here for "the tube cannot switch the accommodation off". There was
    // one, asserting `new(s) == new(s)` — the reflexivity of a pure function,
    // which holds for every implementation including one that read the tube's
    // state. A test that cannot fail stops anyone writing a real one.
    //
    // The guarantee is structural: `new` takes a `Sight` and nothing else, and
    // `sight::plugin::extract` reads `Vision` and nothing else, so no value of
    // `CrtSettings` is in scope on either path.
    //
    // The evidence is the four-way capture matrix in SEEING-IT's greyscale
    // block, which drives the real binary at both tube states and counts
    // hue-carrying pixels.

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
