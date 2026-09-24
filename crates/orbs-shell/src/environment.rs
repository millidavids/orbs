//! The facts a frontend reads from the environment before the world exists: the
//! wizard's name, whether a fresh tower begins sealed, and which readers answer.
//! The seed has a module of its own, `seed`, since a game and an instrument want
//! different ones.
//!
//! All read *outside* the sim: the environment is not deterministic, and
//! reaching for it from a world that must replay from a seed is a habit worth
//! not starting.
//!
//! Here rather than in a frontend's `main` because two rules for deriving the
//! prompt name is the divergence this crate exists to stop — `orbs-tui` reading
//! `USER` itself would agree with the Bevy build on a developer's machine and
//! disagree on Windows, in CI, or with `USER` blank.

/// Who the wizard is, if anything on the machine says.
///
/// `ORBS_WIZARD` first, then the platform's own. `USERNAME` is not optional:
/// §13 ships Windows through Steam, so without it the platform most players are
/// on always falls back to `orbs $ `.
///
/// `ORBS_WIZARD` exists because the alternative was overriding a system
/// variable, which works by accident rather than by intention. The real answer
/// is a settings screen, which §15 puts in Phase 13.
///
/// A blank value is rejected, so an exported-but-empty `USER` falls through to
/// the next candidate instead of naming the wizard nothing.
#[must_use]
pub fn wizard() -> Option<String> {
    ["ORBS_WIZARD", "USER", "USERNAME"]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok())
        .find(|name| !name.trim().is_empty())
}

/// The switch that decides whether a fresh tower begins sealed.
///
/// `1` seals it; `0` opens it; unset takes the caller's default. The game
/// defaults to sealed and a dump to open, for the reason `ORBS_BOOT=0` exists:
/// ~200 See-it lines attend rooms directly (§19).
///
/// Blank falls through to the default, as a blank `USER` does above — reading
/// it as `0` would hand a player the open tower with nothing saying so.
pub const SEALED: &str = "ORBS_SEALED";

/// The switch that decides how long a fresh game is.
///
/// `short`, `medium`, `long` — or `baseline`, the curve exactly as authored.
/// Unset takes the caller's default, split as [`SEALED`] is: the game defaults
/// to medium, a dump to baseline, or all 138 captures would move the day a
/// default changed.
///
/// Blank falls through to the default, for [`SEALED`]'s reason. So does an
/// unrecognised word: a typo should give a player the default game rather than
/// refuse to start one, and there is nowhere this early in the boot to say so.
pub const LENGTH: &str = "ORBS_LENGTH";

/// A fresh tower, sealed or open and long or short as the environment says — or
/// as `sealed` and `length` say when it does not.
///
/// The one caller of these constructors from a frontend: three fresh-world
/// sites across two frontends and the dump, and three readings of one switch is
/// how a rule comes to differ per build.
#[must_use]
pub fn fresh(seed: u64, sealed: bool, length: orbs_sim::content::Length) -> orbs_sim::Sim {
    let wanted = std::env::var(SEALED)
        .ok()
        .and_then(|value| match value.trim() {
            "1" | "true" | "yes" => Some(true),
            "0" | "false" | "no" => Some(false),
            _ => None,
        })
        .unwrap_or(sealed);
    let long = std::env::var(LENGTH)
        .ok()
        .and_then(|value| orbs_sim::content::Length::named(&value))
        .unwrap_or(length);
    match (wanted, long) {
        // An open tower is never a *game*, so it keeps the authored curve
        // whatever the environment says — it is what every dump, example and
        // balance policy measures.
        (false, _) => orbs_sim::Sim::new(seed),
        (true, orbs_sim::content::Length::Baseline) => orbs_sim::Sim::sealed(seed),
        (true, long) => orbs_sim::Sim::begun(seed, long),
    }
}

/// Which reader answers the lines the orb cannot read itself (§6).
const AUGURY: &str = "ORBS_AUGURY";

/// Which reader answers the *spell* lines the orb cannot read itself.
const SCRIVENER: &str = "ORBS_SCRIVENER";

/// The scrivener named by `ORBS_SCRIVENER`, or the trained reader by default.
///
/// On unless turned off, the same shape as [`augury`]: without it a player
/// types `crush the sage` at the prompt, watches it work, puts it in a spell
/// and gets *Referent missing*.
///
/// | | |
/// |---|---|
/// | unset | the trained reader, if this build has one; otherwise none, quietly |
/// | `model` | the same, but says so when there are no weights to load |
/// | `off`, `0`, `none` | no reader. The spell compiles exactly what was typed |
/// | `stub` | [`Copyist::worked`](orbs_sim::Copyist::worked), a fixed table — what a capture pins |
///
/// A capture cannot pin a trained reader: weights are a gitignored artefact
/// that changes on every run, which is why `dumps.sh` runs the prompt's reader
/// as `stub` and `grammar` rather than `model`.
#[must_use]
pub fn scrivener() -> Option<Box<dyn orbs_sim::Scrivener>> {
    match std::env::var(SCRIVENER).unwrap_or_default().trim() {
        "" => copying(false),
        "model" => copying(true),
        "stub" => Some(Box::new(orbs_sim::Copyist::worked())),
        "off" | "0" | "none" => None,
        other => {
            tracing::warn!("scrivener: {other:?} names no reader; running without one");
            None
        }
    }
}

/// The trained spell reader, if this build has one and this checkout has
/// weights.
///
/// [`trained`]'s sibling, and `asked` means the same thing: whether the player
/// named it, which is the whole difference between the two silences.
#[cfg(feature = "augury")]
fn copying(asked: bool) -> Option<Box<dyn orbs_sim::Scrivener>> {
    match orbs_augury::Copying::cpu() {
        Ok(reader) => Some(Box::new(reader)),
        Err(error) => {
            if asked {
                tracing::warn!("scrivener: no trained reader ({error}); running without one");
            } else {
                tracing::debug!("scrivener: no trained reader ({error})");
            }
            None
        }
    }
}

/// No spell reader at all, for a build with `burn` left out.
#[cfg(not(feature = "augury"))]
fn copying(asked: bool) -> Option<Box<dyn orbs_sim::Scrivener>> {
    if asked {
        tracing::warn!("scrivener: this build has no trained reader compiled in");
    }
    None
}

/// The augury named by `ORBS_AUGURY`, or the trained reader by default.
///
/// On unless turned off, which is the shipping default: it reads 81.5% of
/// phrasings nothing taught it where the matcher alone reads 3.5%, costs 436µs
/// on the CPU, and refuses rather than guessing. §16 rates *"parser feels
/// frustrating rather than magical"* Critical.
///
/// | | |
/// |---|---|
/// | unset | the trained reader, if this build has one; otherwise none, quietly |
/// | `model` | the same, but says so when there are no weights to load |
/// | `off`, `0`, `none` | no reader. [`Sim::submit`], exactly as it was |
/// | `stub` | [`Fixture::worked`], a fixed table — what `dumps.sh` pins |
/// | `grammar` | [`Grammar::builtin`](orbs_sim::Grammar::builtin), the authored templates in `content/phrasings.toml` |
///
/// Unset is silent and `model` is not: weights are a gitignored build artefact,
/// so a fresh clone has none and the default must degrade quietly — but asking
/// for `model` by name and getting silence would look like the reader working.
///
/// `scripts/dumps.sh` passes `off` from its `run` helper so a capture stays
/// reproducible from a clean checkout; a block that wants a reader names one
/// after it, and the later value wins.
///
/// `grammar` is a real reader, not a demonstration: on phrasings it was never
/// taught it reads 9.4% where the matcher alone reads 15.6%, and the two
/// together reach 25.0% — they overlap on nothing. `--bench` prints all three.
///
/// `stub` stays after a trained reader exists, because `ORBS_DUMP`,
/// `scripts/dumps.sh` and `scripts/play.sh` can hold no GPU and no weights.
/// Without a reader they can reach, every divined surface is gated on one
/// person typing one sentence into one window — CLAUDE.md's *"the blindness
/// looks exactly like stability"*. It is also what a real reader is measured
/// against.
///
/// An unrecognised value is `off` and says so: with the reader on by default, a
/// typo now costs you the feature you were getting.
///
/// [`Sim::submit`]: orbs_sim::Sim::submit
/// [`Fixture::worked`]: orbs_sim::Fixture::worked
#[must_use]
pub fn augury() -> Option<Box<dyn orbs_sim::Augur>> {
    match std::env::var(AUGURY).unwrap_or_default().trim() {
        // Answered here so a headless dump can reach it: `ORBS_DUMP` builds no
        // `App`, so a reader chosen in a Bevy resource is one `dumps.sh` could
        // never see.
        "" => trained(false),
        "model" => trained(true),
        "off" | "0" | "none" => None,
        "stub" => Some(Box::new(orbs_sim::Fixture::worked())),
        "grammar" => Some(Box::new(orbs_sim::Grammar::builtin())),
        other => {
            tracing::warn!("augury: {other:?} names no reader; running without one");
            None
        }
    }
}

/// The trained reader, if this build has one and this checkout has weights.
///
/// `asked` is whether the player named it, which is the whole difference between
/// the two silences — see [`augury`].
#[cfg(feature = "augury")]
fn trained(asked: bool) -> Option<Box<dyn orbs_sim::Augur>> {
    match orbs_augury::Reading::cpu() {
        Ok(reader) => Some(Box::new(reader)),
        Err(error) => {
            if asked {
                tracing::warn!("augury: no trained reader ({error}); running without one");
            } else {
                tracing::debug!("augury: no trained reader ({error})");
            }
            None
        }
    }
}

/// No reader at all, for a build with `burn` left out.
///
/// Nothing in the workspace lands here now: `orbs-tui` did while the reader
/// needed `wgpu`, but inference is `ndarray` and the GPU sits behind
/// `orbs-augury`'s `train` feature. This arm is for a build that turns the
/// feature off on purpose.
#[cfg(not(feature = "augury"))]
fn trained(asked: bool) -> Option<Box<dyn orbs_sim::Augur>> {
    if asked {
        tracing::warn!("augury: this build has no trained reader compiled in");
    }
    None
}

/// The compiler that built this binary, captured by `build.rs`.
///
/// `env!("CARGO_PKG_RUST_VERSION")` is empty here, the workspace floor is not
/// the compiler in use, and `rust-toolchain.toml` pins a third number again.
/// The POST card must not report an invented version.
pub const RUSTC: &str = env!("ORBS_RUSTC");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_compiler_it_reports_is_a_real_version() {
        // `build.rs` falls back to `0.0.0` when it cannot ask `rustc` — a value
        // no release could be mistaken for, which is the point. Under `cargo
        // test` it can ask, so anything else here means the capture broke.
        assert_ne!(RUSTC, "0.0.0", "build.rs could not reach rustc");
        assert!(
            RUSTC.chars().next().is_some_and(|c| c.is_ascii_digit()),
            "{RUSTC:?} is not a version",
        );
    }
}
