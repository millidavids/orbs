//! The three facts a frontend reads from the environment before the world
//! exists: the seed, the wizard's name, and whether a fresh tower begins sealed.
//!
//! All three are read *outside* the sim, deliberately: the environment is not
//! deterministic, and reaching for it from inside a world that must replay
//! identically from a seed is a habit worth not starting.
//!
//! They live here rather than in a frontend's `main` because there are two
//! frontends now, and two rules for deriving the prompt name is exactly the
//! divergence this crate exists to stop — `orbs-tui` reaching for
//! `std::env::var("USER")` on its own would agree with the Bevy build on a
//! developer's machine and disagree on Windows, in CI, or with `USER` blank.

/// The world's seed when nothing overrides it.
///
/// A hex word rather than a round number, so a seed that appears in a bug report
/// is recognisable as *the default* rather than as something the reporter chose.
pub const SEED: u64 = 0x0B5;

/// The seed, or `ORBS_SEED`'s if it names a number.
///
/// **A See-it affordance, not a setting.** Anything the world *generates* — the
/// archive's stacks first, sabotage and sieges later — is one seed's worth of
/// evidence per run, and one sample cannot show a distribution. Three dumps of
/// the same maze looked like proof that randomising it had failed; they were
/// three copies of one seed.
///
/// Tests sweep seeds directly through `Sim::new` and always could. This is the
/// same reach from outside the binary, so a person can look rather than trust a
/// test — which is the whole of §15's gate.
#[must_use]
pub fn seed() -> u64 {
    std::env::var("ORBS_SEED")
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(SEED)
}

/// Who the wizard is, if anything on the machine says.
///
/// `ORBS_WIZARD` first, then the platform's own. **`USERNAME` is not optional
/// politeness**: §13 ships Windows through Steam, so leaving it out would mean
/// the platform most players are on always falls back to `orbs $ ` while the two
/// development platforms quietly look right.
///
/// `ORBS_WIZARD` exists because the alternative was renaming a wizard by
/// overriding a system variable, which works by accident rather than by
/// intention. The real answer is a settings screen, which §15 puts in Phase 13
/// alongside the rest of the options; until then this is the switch.
///
/// A blank value is rejected rather than accepted, so an exported-but-empty
/// `USER` falls through to the next candidate instead of naming the wizard
/// nothing.
#[must_use]
pub fn wizard() -> Option<String> {
    ["ORBS_WIZARD", "USER", "USERNAME"]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok())
        .find(|name| !name.trim().is_empty())
}

/// The switch that decides whether a fresh tower begins sealed.
///
/// `1` seals it; `0` opens it; unset takes the caller's default. **The game
/// defaults to sealed and a dump defaults to open**, for the reason
/// `ORBS_BOOT=0` exists: the dump is an instrument, ~200 See-it lines attend
/// rooms directly, and every one of them keeps meaning what it meant. §19
/// records the trade-off.
///
/// **Blank falls through to the default**, as a blank `USER` does above: an
/// exported-but-empty variable is the shape "unset" most often arrives in, and
/// reading it as `0` would quietly hand a player the open tower — the whole
/// feature off, with nothing on screen saying so.
pub const SEALED: &str = "ORBS_SEALED";

/// The switch that decides how long a fresh game is.
///
/// `short`, `medium`, `long` — or `baseline`, the curve exactly as authored.
/// Unset takes the caller's default, and **the split is `ORBS_SEALED`'s**: the
/// game defaults to medium and **a dump defaults to baseline**, because the dump
/// is an instrument and all 138 of its captures would otherwise move the day a
/// default changed.
///
/// **Blank falls through to the default**, for the reason recorded on [`SEALED`]
/// — an exported-but-empty variable is the shape "unset" most often arrives in,
/// and reading it as anything in particular is how a switch comes to be on when
/// nobody asked.
///
/// **An unrecognised word also falls through**, deliberately: a typo should give
/// a player the default game rather than refuse to start one, and there is
/// nowhere at this point in the boot to say so.
pub const LENGTH: &str = "ORBS_LENGTH";

/// A fresh tower, sealed or open and long or short as the environment says — or
/// as `sealed` and `length` say when it does not.
///
/// **The one caller of any of these constructors from a frontend.** There are
/// three fresh-world sites across two frontends and the dump, and three readings
/// of one switch is how a rule comes to differ per build.
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
        // whatever the environment says: it is what every dump, example and
        // balance policy measures, and a length on it would be a length nobody
        // asked for on a tower nobody is playing.
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
/// **On unless turned off, and the same shape as [`augury`] for the same
/// reason.** The divergence this closes is a player typing `crush the sage` at
/// the prompt, watching it work, putting it in a spell and getting *Referent
/// missing* — so a reader nobody can reach answers none of it.
///
/// | | |
/// |---|---|
/// | unset | **the trained reader**, if this build has one; otherwise none, quietly |
/// | `model` | the same, but says so when there are no weights to load |
/// | `off`, `0`, `none` | no reader. The spell compiles exactly what was typed |
/// | `stub` | [`Copyist::worked`](orbs_sim::Copyist::worked), a fixed table — what a capture pins |
///
/// A capture cannot pin a trained reader: weights are a gitignored build
/// artefact that changes on every run, so a dump made against them could not be
/// reproduced from a clean checkout. That is the same reason `dumps.sh` runs the
/// prompt's reader as `stub` and `grammar` rather than `model`.
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
/// **On unless turned off, and that is the shipping default.** The reader is
/// what a player gets now: it reads 81.5% of phrasings nothing taught it where
/// the matcher alone reads 3.5%, it costs 436µs on the CPU, and it refuses
/// rather than guessing when a line asks for nothing. §16 rates *"parser feels
/// frustrating rather than magical"* Critical, and a feature nobody can reach
/// answers none of it.
///
/// | | |
/// |---|---|
/// | unset | **the trained reader**, if this build has one; otherwise none, quietly |
/// | `model` | the same, but says so when there are no weights to load |
/// | `off`, `0`, `none` | no reader. [`Sim::submit`], exactly as it was |
/// | `stub` | [`Fixture::worked`], a fixed table — what `dumps.sh` pins |
/// | `grammar` | [`Grammar::builtin`](orbs_sim::Grammar::builtin), the authored templates in `content/phrasings.toml` |
///
/// # Why unset is silent and `model` is not
///
/// **Weights are a build artefact, not source** — gitignored, written by
/// `orbs-augury --example train --features train`. A fresh clone has none, and the
/// default must degrade to the game exactly as it was without complaining about
/// it on every boot. Asking for `model` by name is different: you named a thing
/// that is missing, and silence there would look like the reader working.
///
/// # What still turns it off
///
/// `scripts/dumps.sh` passes `off` from its `run` helper, so a capture stays
/// reproducible from a clean checkout — weights change on every training run and
/// a dump made against them could not be diffed. A block that wants a reader
/// names one after it, and the later value wins.
///
/// **`grammar` is a real reader and not a demonstration.** Measured on
/// phrasings it was never taught it reads 9.4% where the matcher alone reads
/// 15.6%, and the two together reach 25.0% — they overlap on nothing, so it is
/// additive in the plainest sense. `--bench` prints all three.
///
/// # Why a table is a first-class option and not a placeholder
///
/// `ORBS_DUMP` builds no app and presses no key, `scripts/dumps.sh` captures
/// surfaces as text, and `scripts/play.sh` drives the terminal build under
/// `tmux` — none of which can hold a GPU, a worker thread, or megabytes of
/// weights. Without a reader they can reach, every divined surface would be
/// gated on one person typing one sentence into one window, and CLAUDE.md
/// records what that costs: *"a domain built without a block in it is one this
/// instrument is blind to, and the blindness looks exactly like stability."*
///
/// So `stub` stays after a trained reader exists. It is what makes a divined
/// screen diffable, and what a real reader gets measured against.
///
/// An unrecognised value is `off` **and says so**, which it did not have to
/// before: with the reader off by default a typo cost you the feature you were
/// already not getting, and now it costs you the one you were.
///
/// [`Sim::submit`]: orbs_sim::Sim::submit
/// [`Fixture::worked`]: orbs_sim::Fixture::worked
#[must_use]
pub fn augury() -> Option<Box<dyn orbs_sim::Augur>> {
    match std::env::var(AUGURY).unwrap_or_default().trim() {
        // **Answered here so a headless dump can reach it.** `ORBS_DUMP` builds
        // no `App`, so a reader chosen in a Bevy resource is one
        // `scripts/dumps.sh` can never see — which is the blindness the stub
        // exists to avoid, arriving by a different door.
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
/// **Nothing in the workspace lands here now.** `orbs-tui` did, while the reader
/// needed `wgpu` and a terminal program had no business pulling a graphics stack
/// — but inference is `ndarray` and costs 436µs, so `orbs-augury` put the GPU
/// behind its `train` feature and both frontends take the same reader. This arm
/// stays for a build that turns the feature off on purpose.
#[cfg(not(feature = "augury"))]
fn trained(asked: bool) -> Option<Box<dyn orbs_sim::Augur>> {
    if asked {
        tracing::warn!("augury: this build has no trained reader compiled in");
    }
    None
}

/// The compiler that built this binary, captured by `build.rs`.
///
/// `env!("CARGO_PKG_RUST_VERSION")` was the obvious choice and is **empty**
/// here — this crate does not inherit `rust-version` — and the workspace's floor
/// is not the compiler in use, which `rust-toolchain.toml` pins to something
/// else again. Three numbers, only one of them true. The POST card's whole
/// argument is that reporting an invented version would be a lie the player
/// reads first.
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
