//! Which step the installation is on, and when the real tower is raised.
//!
//! The practice tower is a real sealed [`Sim`] that the frontend holds as its
//! tower while this runs, so the prompt, the parser and the echo are the game's
//! own. This only watches. DESIGN.md §19, *The orb is installed before the tower
//! wakes*.

use std::path::PathBuf;

use orbs_sim::content::Length;
use orbs_sim::parser::{Outcome, ParseRecord, Register, Verb};
use orbs_sim::{Augur, Sim};

use super::paint::Said;
use crate::save::Keep;

/// `0` never installs, `1` always does, and anything else leaves it to the
/// `install` setting. In a dump, `1` poses the installation and nothing else
/// does.
pub const INSTALL: &str = "ORBS_INSTALL";

/// The word that ends the installation from any step. Matched whole, never by
/// prefix, and never submitted.
pub const SKIP: &str = "skip";

/// The word that ends it once every step is done, matched as [`SKIP`] is. A
/// word rather than a bare Enter, which a player still practising presses out
/// of habit.
pub const WAKE: &str = "wake";

/// One thing the orb learns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// `help`: a word typed and Enter pressed.
    Listening,
    /// `survey`: what is here.
    Sight,
    /// `attend`: rooms, and the pane's title saying which.
    Paths,
    /// Plain words or a shell's, and the echo saying what they meant.
    Speech,
    /// `status`.
    Report,
}

impl Step {
    /// In the order they are taught.
    pub const ALL: [Self; 5] = [
        Self::Listening,
        Self::Sight,
        Self::Paths,
        Self::Speech,
        Self::Report,
    ];

    /// Its part of every `install_<key>_*` prose key.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Listening => "listening",
            Self::Sight => "sight",
            Self::Paths => "paths",
            Self::Speech => "speech",
            Self::Report => "report",
        }
    }

    const fn wants(self) -> Wants {
        match self {
            Self::Listening => Wants::Verb(Verb::Recall),
            Self::Sight => Wants::Verb(Verb::Survey),
            Self::Paths => Wants::Elsewhere,
            Self::Speech => Wants::OwnWords,
            Self::Report => Wants::Verb(Verb::Status),
        }
    }
}

/// What finishes a step.
#[derive(Debug, Clone, Copy)]
enum Wants {
    /// A line that resolved to this verb.
    Verb(Verb),
    /// Standing anywhere but where the installation began.
    Elsewhere,
    /// A line that resolved from plain words or a shell's, or that the augury
    /// read.
    OwnWords,
}

impl Wants {
    /// Whether this record finishes the step.
    ///
    /// Resolved first, because an `Incomplete` or `Elsewhere` record also
    /// carries a verb and a register, and `divined` is stamped on a reading that
    /// ran nothing.
    fn met_by(self, record: &ParseRecord) -> bool {
        if !matches!(record.outcome, Outcome::Resolved | Outcome::Forced) {
            return false;
        }
        match self {
            // `ParseRecord` keeps no verb, and the echo leads with its canonical
            // form.
            Self::Verb(verb) => {
                record
                    .echo
                    .as_deref()
                    .and_then(|echo| echo.split_whitespace().next())
                    == Some(verb.canonical())
            }
            Self::OwnWords => {
                record.divined || matches!(record.register, Some(Register::Plain | Register::Shell))
            }
            Self::Elsewhere => false,
        }
    }
}

/// The tower `new` asked for, held until the installation ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raise {
    /// The slot it will be kept in, or [`None`] for a session that keeps
    /// nothing.
    pub keep: Option<Keep>,
    /// How long it is to be.
    pub length: Length,
    /// Its seed, which the practice tower shares, so `status` reads the same
    /// before and after.
    pub seed: u64,
    /// The wizard's name, or [`None`] to keep the sim's own. Held so both
    /// towers' prompts read the same.
    pub wizard: Option<String>,
}

impl Raise {
    /// The practice tower: this tower before it has said anything.
    #[must_use]
    pub fn sandbox(&self) -> Sim {
        self.named(Sim::sandbox(self.seed, self.length))
    }

    /// The tower itself, at tick 0, whether or not anything was installed.
    #[must_use]
    pub fn tower(&self) -> Sim {
        self.named(Sim::begun(self.seed, self.length))
    }

    fn named(&self, mut sim: Sim) -> Sim {
        if let Some(wizard) = &self.wizard {
            sim.rename(wizard);
        }
        sim
    }

    /// What `new` opens on: the practice tower, kept nowhere whatever slot `new`
    /// found, or the tower itself when nothing is installed. One copy for both
    /// frontends and the dump.
    ///
    /// The slot is asked again first, on both paths: the menu read it from the
    /// disk, where a tower raised since its last autosave is not yet, and the
    /// swap has just written that tower there. The trade is silent, since the
    /// tower in it is this orb's own.
    #[must_use]
    pub fn begin(mut self, installs: bool) -> Begun {
        let free = self.keep.as_ref().map(Keep::still_free);
        if !installs {
            let mut sim = self.tower();
            if free == Some(None) {
                sim.say_save_failed();
            }
            return Begun {
                sim,
                install: None,
                keep: free.flatten().as_ref().map(Keep::path),
            };
        }
        // With no slot left the asked one is kept, so the wake finds it taken
        // and says so in the real tower rather than the practice one.
        if let Some(Some(slot)) = free {
            self.keep = Some(slot);
        }
        let sim = self.sandbox();
        let install = Install::new(self, &sim);
        Begun {
            sim,
            install: Some(install),
            keep: None,
        }
    }

    /// The tower itself, once the installation ends, in the slot `begin` chose —
    /// or the lowest free one if another orb wrote that slot meanwhile, which is
    /// said, as is having none left.
    #[must_use]
    pub fn wake(&self) -> Begun {
        let mut sim = self.tower();
        let keep = self.keep.as_ref().and_then(|asked| {
            let free = asked.still_free();
            match &free {
                None => sim.say_save_failed(),
                Some(slot) if slot != asked => sim.say_save_moved(slot.slot),
                Some(_) => {}
            }
            free
        });
        Begun {
            sim,
            install: None,
            keep: keep.as_ref().map(Keep::path),
        }
    }
}

/// The tower a frontend puts in front of the player after `new` or a wake.
pub struct Begun {
    /// The practice tower or the real one.
    pub sim: Sim,
    /// The installation, while `sim` is the practice tower.
    pub install: Option<Install>,
    /// Its slot, or [`None`] for a tower kept nowhere.
    pub keep: Option<PathBuf>,
}

/// Where the installation has reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Install {
    raise: Raise,
    /// Where the player stood when it began. `paths` is done anywhere else.
    home: String,
    /// How many steps are done.
    done: usize,
    /// The first `ParseLog` record the step in hand may be judged on, so one
    /// line finishes one step.
    from: usize,
    /// The strip's words, so a frame draws them without building them.
    said: Said,
}

impl Install {
    /// Begin, over the practice tower `raise` built.
    #[must_use]
    pub fn new(raise: Raise, sim: &Sim) -> Self {
        Self {
            raise,
            home: sim.location(),
            done: 0,
            from: sim.parse_log().records().len(),
            said: Said::new(0, sim.prose()),
        }
    }

    /// The step in hand, or [`None`] when every one is done.
    #[must_use]
    pub fn step(&self) -> Option<Step> {
        Step::ALL.get(self.done).copied()
    }

    /// Whether every step is done.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.step().is_none()
    }

    /// The tower that will be raised.
    #[must_use]
    pub const fn raise(&self) -> &Raise {
        &self.raise
    }

    pub(super) const fn said(&self) -> &Said {
        &self.said
    }

    /// Finish whatever the practice tower now shows the player has done.
    ///
    /// Several steps can finish in one call, but each on its own line: a record
    /// that finishes one is behind `from` for the next.
    pub fn observe(&mut self, sim: &Sim) {
        self.advance(sim);
        // Once a tick rather than once a frame, and still in time for a prose
        // reload.
        self.said = Said::new(self.done, sim.prose());
    }

    fn advance(&mut self, sim: &Sim) {
        let records = sim.parse_log().records();
        while let Some(step) = self.step() {
            match step.wants() {
                Wants::Elsewhere => {
                    if sim.location() == self.home {
                        return;
                    }
                    self.finish(records.len());
                }
                wants => {
                    let found = records
                        .iter()
                        .enumerate()
                        .skip(self.from)
                        .find(|(_, record)| wants.met_by(record));
                    let Some((at, _)) = found else {
                        self.from = records.len();
                        return;
                    };
                    self.finish(at + 1);
                }
            }
        }
    }

    const fn finish(&mut self, from: usize) {
        self.done += 1;
        self.from = from;
    }

    /// Take a line the player finished at the practice tower's prompt.
    ///
    /// The one path for a line during installation, shared by both frontends
    /// and the dump. `skip`, or `wake` once every step is done, wakes the tower
    /// and reaches no world. Anything else is submitted, so a question left
    /// unanswered costs nothing (§6).
    pub fn submit(
        &mut self,
        sim: &mut Sim,
        line: &str,
        augur: Option<&dyn Augur>,
    ) -> Option<Raise> {
        let typed = line.trim();
        if typed.eq_ignore_ascii_case(SKIP)
            || (self.is_complete() && typed.eq_ignore_ascii_case(WAKE))
        {
            return Some(self.raise.clone());
        }
        sim.submit_with(line, augur);
        self.observe(sim);
        None
    }
}

/// Whether a new tower is installed first: `ORBS_INSTALL`, then the setting.
#[must_use]
pub fn wanted(setting: bool) -> bool {
    from_value(std::env::var(INSTALL).ok().as_deref(), setting)
}

/// [`wanted`] with the lookup done, so it can be tested without touching a
/// process-global environment.
///
/// Blank or unrecognised falls through to the setting: an exported-but-empty
/// variable is how "unset" usually arrives.
#[must_use]
pub fn from_value(value: Option<&str>, setting: bool) -> bool {
    value
        .and_then(crate::environment::switch)
        .unwrap_or(setting)
}

/// Whether a dump poses the installation. Only when asked: the dump has no
/// settings, and its default must stay the open tower every capture expects.
#[must_use]
pub fn posed() -> bool {
    from_value(std::env::var(INSTALL).ok().as_deref(), false)
}
