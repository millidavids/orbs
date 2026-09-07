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
