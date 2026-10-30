//! The facts a frontend reads from the environment before the world exists: the
//! wizard's name, whether a fresh tower begins sealed, and which readers answer.
//! The seed has a module of its own, `seed`, since a game and an instrument want
//! different ones.
//!
//! Read outside the sim, which must replay from a seed. Here rather than in
//! each `main`, or the frontends disagree on Windows, in CI or with `USER`
//! blank.

/// Who the wizard of a new tower is, as the settings file says.
///
/// [`wizard_named`] with the file's answer. A frontend holding the choice in
/// memory asks that instead, so a choice it could not write down still counts.
#[must_use]
pub fn wizard() -> Option<String> {
    wizard_named(crate::settings::chosen_name().as_deref())
}

/// Who the wizard of a new tower is: the switch, then the player's choice,
/// then the machine's login.
///
/// `ORBS_WIZARD` first, so a capture matches everywhere; `USERNAME` for
/// Windows (§13). A blank value, or one the menu would refuse, falls through.
#[must_use]
pub fn wizard_named(chosen: Option<&str>) -> Option<String> {
    let (switch, login) = machine_names();
    switch
        .clone()
        .or_else(|| {
            chosen
                .filter(|name| !name.trim().is_empty())
                .map(ToOwned::to_owned)
        })
        .or_else(|| login.clone())
}

/// The name a new tower would take, as the settings row shows it:
/// [`wizard_named`], or the sim's own default when nothing names one.
///
/// Both builds ask this, so the row and the tower cannot disagree on the order.
#[must_use]
pub fn new_wizard(chosen: Option<&str>) -> String {
    wizard_named(chosen).unwrap_or_else(|| orbs_sim::DEFAULT_WIZARD.to_owned())
}

/// `ORBS_WIZARD`, and the machine's login: read once, since nothing changes a running
/// process's environment and the settings row asks every frame a page is up.
fn machine_names() -> &'static (Option<String>, Option<String>) {
    static READ: std::sync::OnceLock<(Option<String>, Option<String>)> = std::sync::OnceLock::new();
    READ.get_or_init(|| {
        (
            switched("ORBS_WIZARD"),
            as_name(switched("USER")).or_else(|| as_name(switched("USERNAME"))),
        )
    })
}

/// A login, if it could be a name. A Windows `John Smith`, or a letter the
/// glass cannot draw, must not reach the prompt by a door the menu guards.
fn as_name(login: Option<String>) -> Option<String> {
    login.and_then(|login| crate::settings::checked_name(&login).ok())
}

/// `key`'s value, unless it is unset or blank.
pub(crate) fn switched(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .filter(|name| !name.trim().is_empty())
}

/// The switch that decides whether a fresh tower begins sealed.
///
/// `1` seals, `0` opens, unset or blank takes the caller's default: sealed for
/// the game, open for a dump, since See-it lines attend rooms directly (§19).
pub const SEALED: &str = "ORBS_SEALED";

/// The switch that decides how long a fresh game is.
///
/// `short`, `medium`, `long` — or `baseline`, the curve exactly as authored.
/// Unset, blank or unrecognised takes the caller's default, split as
/// [`SEALED`] is: medium for the game, baseline for a dump, so captures never
/// move with it.
pub const LENGTH: &str = "ORBS_LENGTH";

/// A fresh tower, sealed or open and long or short as the environment says — or
/// as `sealed` and `length` say when it does not.
///
/// The one place a frontend or dump reads these switches.
#[must_use]
pub fn fresh(seed: u64, sealed: bool, length: orbs_sim::content::Length) -> orbs_sim::Sim {
    let wanted = std::env::var(SEALED)
        .ok()
        .as_deref()
        .and_then(switch)
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

/// An on/off switch's value, or [`None`] when it is blank or unrecognised.
///
/// The one reading for every such switch, so they cannot come to accept
/// different words.
pub(crate) fn switch(value: &str) -> Option<bool> {
    match value.trim() {
        "1" | "true" | "yes" => Some(true),
        "0" | "false" | "no" => Some(false),
        _ => None,
    }
}

/// Which reader answers the lines the orb cannot read itself (§6).
const AUGURY: &str = "ORBS_AUGURY";

/// Which reader answers the *spell* lines the orb cannot read itself.
const SCRIVENER: &str = "ORBS_SCRIVENER";

/// The scrivener named by `ORBS_SCRIVENER`, or the trained reader by default.
///
/// On unless turned off, as [`augury`] is, or what works at the prompt fails
/// in a spell.
///
/// | | |
/// |---|---|
/// | unset | the trained reader, if this build has one; otherwise none, quietly |
/// | `model` | the same, but says so when there are no weights to load |
/// | `off`, `0`, `none` | no reader. The spell compiles exactly what was typed |
/// | `stub` | [`Copyist::worked`](orbs_sim::Copyist::worked), a fixed table — what a capture pins |
///
/// A capture cannot pin trained weights, so `dumps.sh` uses `stub`.
#[must_use]
pub(crate) fn scrivener() -> Option<Box<dyn orbs_sim::Scrivener>> {
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

/// The trained spell reader, if this build has one and was built with weights.
///
/// [`trained`]'s sibling; `asked` is whether the player named it.
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
/// Unset is silent, since a player can do nothing about a build without
/// weights; `model` by name is not. `scripts/dumps.sh` passes `off` and a block names a reader after it.
/// `grammar` is a real reader, overlapping the matcher on nothing
/// (`--bench`). `stub` stays so the instruments, which hold no weights, can
/// reach every divined surface. An unrecognised value is `off`, and says so.
///
/// [`Sim::submit`]: orbs_sim::Sim::submit
/// [`Fixture::worked`]: orbs_sim::Fixture::worked
#[must_use]
pub(crate) fn augury() -> Option<Box<dyn orbs_sim::Augur>> {
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

/// The trained reader, if this build has one and was built with weights.
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
/// For a build that turns the feature off on purpose; nothing in the
/// workspace does.
#[cfg(not(feature = "augury"))]
fn trained(asked: bool) -> Option<Box<dyn orbs_sim::Augur>> {
    if asked {
        tracing::warn!("augury: this build has no trained reader compiled in");
    }
    None
}

/// The two readers a session can consult — the prompt's and the spell's — and
/// the rule for when it does.
///
/// Loaded once and kept even under `plain`, so switching drivers never puts a
/// file read on a keystroke.
#[derive(Default)]
pub struct Readers {
    reader: Option<Box<dyn orbs_sim::Augur>>,
    scribe: Option<Box<dyn orbs_sim::Scrivener>>,
}

impl Readers {
    /// Whatever `ORBS_AUGURY` and `ORBS_SCRIVENER` asked for.
    #[must_use]
    pub fn from_environment() -> Self {
        Self::new(augury(), scrivener())
    }

    /// These two, chosen directly.
    #[must_use]
    pub fn new(
        reader: Option<Box<dyn orbs_sim::Augur>>,
        scribe: Option<Box<dyn orbs_sim::Scrivener>>,
    ) -> Self {
        Self { reader, scribe }
    }

    /// The prompt's reader under `driver`, or nothing for `plain`.
    ///
    /// Two switches answering different questions: the environment says what
    /// this build offers (a developer's override), the driver whether the
    /// player wants it consulted.
    #[must_use]
    pub fn reader(&self, driver: crate::Driver) -> Option<&dyn orbs_sim::Augur> {
        match driver {
            crate::Driver::Plain => None,
            crate::Driver::Augury => self.reader.as_deref(),
        }
    }

    /// The spell reader, on exactly the same terms.
    #[must_use]
    pub fn scrivener(&self, driver: crate::Driver) -> Option<&dyn orbs_sim::Scrivener> {
        match driver {
            crate::Driver::Plain => None,
            crate::Driver::Augury => self.scribe.as_deref(),
        }
    }
}

/// The compiler that built this binary, captured by `build.rs`.
///
/// Not `CARGO_PKG_RUST_VERSION` or the toolchain pin, which can differ, so the
/// POST card reports the real one.
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

    #[test]
    fn a_login_the_menu_would_refuse_is_no_name() {
        assert_eq!(as_name(Some("david".to_owned())), Some("david".to_owned()));
        for refused in ["John Smith", "名前", "averyveryverylongname"] {
            assert_eq!(as_name(Some(refused.to_owned())), None, "{refused:?}");
        }
    }
}
