//! Whether a tower has been chosen yet (DESIGN.md §15, §19).
//!
//! The orb wakes to a menu, not a tower. Launching used to read
//! `orbs-save.toml` and put a world in front of the player before they asked for
//! one, at a length they had not chosen, with no way to see what else was on
//! disk. The menu answering all three existed (`0.12`) and was reachable only
//! from inside a game.
//!
//! So there is a state before the world: [`Threshold::Waiting`]. The menu is the
//! whole screen, nothing ticks and nothing is written — a tower is raised only
//! when one is chosen, arriving through the swap the menu already used.
//!
//! One enum, shared, because it gates two builds. `focus.rs` makes the argument
//! at length: a rule expressed twice is a rule the frontends come to disagree
//! about. Both ask this; neither decides it.
//!
//! Deliberately not in `shell_resources!`, the Bevy build's reset list, since
//! `reset_for_swap` blanks every type in it to its `Default` — and a `Threshold`
//! reset on a swap would put the player back out of the tower the swap had just
//! chosen. Lifted out for the reason `Screen` and `Standing` are, with the swap
//! setting [`Threshold::Playing`] explicitly.
//!
//! `Playing` is the default, deliberately: every headless harness builds an app
//! without this plugin and expects a world it can drive, so an absent or
//! defaulted `Threshold` has to mean *the game is running*, exactly as an absent
//! [`Boot`](crate::Boot) does.

// `bevy_ecs::Resource` is the trait `bevy::prelude::Resource` is — `bevy`
// re-exports this crate and the lockfile holds one copy. See `screen.rs`.
use bevy_ecs::prelude::Resource;

/// The switch that decides whether the orb wakes to its menu.
///
/// `0` lands straight in a tower, the way launching did before this existed;
/// `1` stands at the threshold. Unset takes the caller's default, and the split
/// is [`SEALED`](crate::SEALED)'s: the game waits and an instrument does not.
///
/// The harnesses need the `0` end and it is not optional: `scripts/play.sh`
/// drives the terminal build under `tmux` and `scripts/tui.sh` passes its
/// environment by hand, and both are written to reach a *tower*. A scenario
/// about the laboratory that had to type its way past a menu first would be
/// testing the door.
///
/// The `1` end is for `ORBS_DUMP`, which builds no `App` and so has no
/// `Threshold` resource to read — see `dump::menued`.
///
/// Blank falls through to the default, as for `ORBS_SEALED`: an
/// exported-but-empty variable is the shape "unset" most often arrives in, and
/// reading it as anything in particular is how a switch comes to be on when
/// nobody asked.
pub const THRESHOLD: &str = "ORBS_THRESHOLD";

/// Whether a tower has been chosen.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Threshold {
    /// No tower yet. The menu has the screen, the clock does not run, and
    /// nothing on disk is touched.
    Waiting,
    /// A tower is in front of the player and the game is running.
    ///
    /// The default, for the reason in the module doc: every harness that does
    /// not install this expects a world it can drive.
    #[default]
    Playing,
}

impl Threshold {
    /// Where the environment says to start, or `default` when it says nothing.
    ///
    /// One reading of the switch, here, for the reason [`fresh`](crate::fresh)
    /// is the one reading of `ORBS_SEALED`: there are three callers across two
    /// frontends and the dump, and three readings of one switch is how a rule
    /// comes to differ per build.
    #[must_use]
    pub fn chosen(default: Self) -> Self {
        Self::from_value(std::env::var(THRESHOLD).ok().as_deref(), default)
    }

    /// [`chosen`](Self::chosen) with the lookup already done.
    ///
    /// Split out so it can be tested. `crt::plugin::chosen` gives the reason: a
    /// test setting the variable sets it for every other test in the process,
    /// since `cargo test` runs threads and the environment is process-global.
    #[must_use]
    fn from_value(value: Option<&str>, default: Self) -> Self {
        match value.map(str::trim) {
            Some("1" | "true" | "yes") => Self::Waiting,
            Some("0" | "false" | "no") => Self::Playing,
            // Blank or unrecognised falls through, for the reason `SEALED`
            // gives: a typo should hand the player the ordinary start rather
            // than a screen nobody asked for, and there is nowhere this early
            // in the boot to say so.
            _ => default,
        }
    }

    /// Whether the game is running.
    ///
    /// Named for the run condition that wraps it, so a call site reads as the
    /// thing it is guarding rather than as a comparison — `focus.rs`'s
    /// `is_elsewhere` is the same idea.
    #[must_use]
    pub const fn is_playing(self) -> bool {
        matches!(self, Self::Playing)
    }

    /// Whether the player is still at the door.
    #[must_use]
    pub const fn is_waiting(self) -> bool {
        matches!(self, Self::Waiting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_a_running_game() {
        // Load-bearing, not a convenience: every headless harness builds an app
        // without the threshold plugin and drives a world, so a default of
        // `Waiting` would freeze all of them at a menu they never open.
        assert_eq!(Threshold::default(), Threshold::Playing);
        assert!(Threshold::default().is_playing());
    }

    #[test]
    fn the_two_states_are_complements() {
        for state in [Threshold::Waiting, Threshold::Playing] {
            assert_ne!(
                state.is_playing(),
                state.is_waiting(),
                "{state:?} is both or neither",
            );
        }
    }

    #[test]
    fn the_switch_answers_both_ways_and_otherwise_falls_through() {
        for word in ["1", "true", "yes", " 1 "] {
            assert_eq!(
                Threshold::from_value(Some(word), Threshold::Playing),
                Threshold::Waiting,
                "{word:?} did not stand at the threshold",
            );
        }
        for word in ["0", "false", "no"] {
            assert_eq!(
                Threshold::from_value(Some(word), Threshold::Waiting),
                Threshold::Playing,
                "{word:?} did not skip the threshold",
            );
        }
        // Blank, absent and nonsense all take the caller's default — and both
        // defaults are checked, because a fall-through that quietly picked one
        // would pass a test that only ever asked for the other.
        for value in [None, Some(""), Some("  "), Some("maybe")] {
            for default in [Threshold::Waiting, Threshold::Playing] {
                assert_eq!(
                    Threshold::from_value(value, default),
                    default,
                    "{value:?} did not fall through to {default:?}",
                );
            }
        }
    }
}
