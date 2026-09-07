//! What work is worth, and what it buys (DESIGN.md §11.5).
//!
//! Authored in `content/progression.toml`, not here (rule 6). This module knows
//! the *shape* of the two tracks and nothing about their numbers.
//!
//! # Two tracks, two shapes
//!
//! **The Ley Line is the tower's.** One list of stations on total experience. A
//! station is a *step* — `grants` something, and passing it is the grant — or a
//! *fork* — `nodes` to choose one of, taken with `take`. A station may also
//! `opens` something: a room, a recipe, a charm.
//!
//! **Mastery is per domain.** One straight line per room you work in — six, not
//! §10's seven, since the grimoire left `DOMAINS` (§19) — each a list of
//! stations with a [`Deed`] on it. No choices anywhere: a station is reached
//! when its deed is done and the one before it is reached.
//!
//! §19 records that the two names were attached the other way round when the
//! weave shipped — the Ley Line was the no-choice track and Mastery the tree —
//! and why they swapped.
//!
//! # It reaches decisions, so it is not hot-reloadable
//!
//! The opposite of [`Materials`](super::Materials) next door, and for the reason
//! that module states: a tint is read by the panel and by nothing else, so
//! swapping it cannot change what the world does. A **weight** changes what a
//! run earns and a **threshold** gates a verb, so both are decisions — and
//! `Sim::new` is explicit that content reaching a decision loads once, because
//! otherwise `(seed, submissions)` stops replaying.
//!
//! # Anything nobody can name is an error
//!
//! Every name in this file is checked against something at load: `[earns]` keys
//! against the instruments, a `grants` against the closed set below, a fork's
//! nodes against the grant parser, a deed against the recipes and the events,
//! an `opens` against the rooms, the gated recipes and the charms. A name
//! matching nothing would earn or open nothing, which looks exactly like a
//! number someone chose — the indistinguishable-typo problem `materials.toml`
//! records paying for once.
//!
//! **Validated against other content**, so the check takes a [`Catalogue`]
//! rather than reaching for resources, which keeps it a pure function and lets
//! `Sim::new` decide the order.

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use serde::Deserialize;

use super::Deed;

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/progression.toml");

/// The file's name, for an error a writer can act on.
const FILE: &str = "progression.toml";

/// What a ley-line **step** may grant.
///
/// **A closed set, checked at load.** The same rule as `[earns]`'s keys and
/// `materials.toml`'s colours: a `grants` nobody implements would give the step
/// nothing, which reads exactly like a step deliberately authored as a marker —
/// and the file would be correct on its face while the curve quietly stopped
/// half way up.
pub const GRANTS: [&str; 2] = [CONCENTRATION, QUINTESSENCE];

/// What the Ley Line grants first — a spell the orb can hold.
pub const CONCENTRATION: &str = "concentration";

/// ...and what it grants second: a deeper pool to pledge from in a siege.
pub const QUINTESSENCE: &str = "quintessence";

/// What work is worth, and what it buys.
///
/// **`Default` is the built-in table, not an empty one.** `Fuels` and
/// `Materials` both carry the same hand-written impl, and `Materials` records
/// why: a derived `Default` gives an empty map, `Sim` installs it with
/// `init_resource`, and every run silently earns nothing while the file on disk
/// is perfectly correct.
///
/// **Unknown sections fail the load**, which is the rule the tracks are
/// defaulted *for*. With both optional, the old `[concentration] levels = [16]`
/// parsed happily into a tower with no curve at all — every threshold gone, no
/// verb refusing, and the file correct on its face. A misspelled section is the
/// same defect as a misspelled `[earns]` key and gets the same answer.
#[derive(Debug, Clone, Deserialize, Resource)]
#[serde(deny_unknown_fields)]
pub struct Progression {
    /// What a completed run is worth, by what did it.
    earns: BTreeMap<String, u64>,
    /// The tower's line: steps and forks on total experience.
    ///
    /// **Defaulted**, so a file with no track at all is a tower that earns and
    /// buys nothing rather than a load failure. The tests below author `[earns]`
    /// alone for exactly this reason.
    #[serde(default)]
    ley_line: Vec<Station>,
    /// The rooms' lines, one station at a time, in the file's order.
    #[serde(default)]
    mastery: Vec<Milestone>,
    /// What the tower is called, by how much renown it holds.
    ///
    /// **Titles, not gates**, so a rank carries `at` and `id` and nothing else —
    /// there is no `opens` here on purpose. Defaulted like the other two tracks,
    /// so a file with no ranks is a tower nobody has heard of rather than a load
    /// failure.
    #[serde(default)]
    renown: Vec<Rank>,
}

/// One thing the tower may be called.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rank {
    /// The renown that earns the title — and, falling back through it, loses it.
    pub at: u64,
    /// The id: a decision, not prose. `renown_<id>` is the name a player reads.
    pub id: String,
}

/// One station of the Ley Line: a step or a fork.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Station {
    /// The total that reaches it.
    pub at: u64,
    /// What passing it gives, for a step: `concentration` or `quintessence`,
    /// the closed set `check` holds it to.
    #[serde(default)]
    pub grants: Option<String>,
    /// What to choose one of, for a fork. **Ordered**, and the order is the
    /// file's; the painter re-orders by lane.
    #[serde(default)]
    pub nodes: Vec<String>,
    /// What passing it opens, if anything. Keys as `tower::opened::Key` reads.
    #[serde(default)]
    pub opens: Vec<String>,
}

impl Station {
    /// Whether this station is a choice.
    #[must_use]
    pub const fn is_fork(&self) -> bool {
        !self.nodes.is_empty()
    }
}

/// One station on a domain's mastery line.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Milestone {
    /// Which room's line this sits on. One of `tower::DOMAINS`.
    pub domain: String,
    /// The id: a decision, not prose. What a reached station is stored as and
    /// what its sentence in `prose.toml` is keyed by.
    pub id: String,
    /// What has to have been done.
    pub done: Deed,
    /// What reaching it opens, if anything.
    #[serde(default)]
    pub opens: Vec<String>,
}

/// Everything the file's names are checked against.
///
/// Four lists rather than four resources, so [`Progression::check`] stays a
/// pure function of content — the same reason it took an instrument list before
/// there were four things to check.
#[derive(Debug, Clone, Copy)]
pub struct Catalogue<'a> {
    /// Everything that runs and may be priced or counted: the recipes'
    /// instruments plus the fixtures that carry a verb.
    pub instruments: &'a [&'a str],
    /// Every product any recipe makes.
    pub outputs: &'a [&'a str],
    /// The products a recipe gates behind a station.
    pub gated: &'a [&'a str],
    /// Every charm the forge can lay.
    pub charms: &'a [&'a str],
}

impl Default for Progression {
    fn default() -> Self {
        Self::builtin()
    }
}

impl Progression {
    /// This curve, stretched to `length`.
    ///
    /// # What moves and what does not
    ///
    /// **Thresholds move; rates do not.** `ley_line`'s and `renown`'s `at`, and
    /// the counts inside each mastery `done`, are stretched by their index on
    /// their own line. [`earns`](Self::earns) is left **exactly** as authored —
    /// it is what a run is *worth*, so stretching it alongside the thresholds
    /// would multiply both sides of the same fraction and cancel the whole
    /// feature, while every rate `orbs-balance` pins would still pass. That is
    /// the failure this method is most likely to be broken by, so a test asserts
    /// `earns` is identical before and after.
    ///
    /// # Each line is indexed on its own
    ///
    /// Mastery is seven lines, not one, and a station's ramp is its position on
    /// **its own room's line** — so every room's first station is unchanged and
    /// every room opens on the schedule it opens on now. Indexing the flat file
    /// order instead would put the menagerie's first deed a third of the way up
    /// the ramp for no reason a player could see.
    ///
    /// Applied before `check`, whose `ascends` gate every length survives: a
    /// strictly increasing sequence times a non-decreasing positive one is
    /// strictly increasing.
    #[must_use]
    pub fn stretched(self, length: crate::content::Length) -> Self {
        let ley = self.ley_line.len();
        let ranks = self.renown.len();
        // How many stations each room's line holds, so a milestone can be ramped
        // against its own line rather than against the file.
        let mut lines: BTreeMap<&str, usize> = BTreeMap::new();
        for stone in &self.mastery {
            *lines.entry(stone.domain.as_str()).or_default() += 1;
        }
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        let mastery = self
            .mastery
            .iter()
            .map(|stone| {
                let count = lines.get(stone.domain.as_str()).copied().unwrap_or(1);
                let index = seen.entry(stone.domain.as_str()).or_default();
                let stretched = Milestone {
                    domain: stone.domain.clone(),
                    id: stone.id.clone(),
                    done: stone.done.stretched(length, *index, count),
                    opens: stone.opens.clone(),
                };
                *index += 1;
                stretched
            })
            .collect();

        Self {
            earns: self.earns,
            ley_line: self
                .ley_line
                .iter()
                .enumerate()
                .map(|(index, station)| Station {
                    at: length.stretch(station.at, index, ley),
                    grants: station.grants.clone(),
                    nodes: station.nodes.clone(),
                    opens: station.opens.clone(),
                })
                .collect(),
            mastery,
            renown: self
                .renown
                .iter()
                .enumerate()
                .map(|(index, rank)| Rank {
                    at: length.stretch(rank.at, index, ranks),
                    id: rank.id.clone(),
                })
                .collect(),
        }
    }

    /// The curve compiled into the binary.
    ///
    /// # Panics
    ///
    /// If the built-in file is malformed — a build-time authoring error, covered
    /// by `the_builtin_file_parses`.
    #[must_use]
    pub fn builtin() -> Self {
        super::load::builtin(FILE, BUILTIN)
    }

    /// Parse a progression file.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of
    /// the expected shape.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        super::load::parse(FILE, text)
    }

    /// Check every name in the file against what exists, and both tracks
    /// against themselves.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) naming what is wrong. See the module
    /// header for why every one of these is a load failure rather than a warning.
    pub fn check(&self, catalogue: &Catalogue<'_>) -> Result<(), super::ContentError> {
        ascends("ley_line", self.ley_line.iter().map(|station| station.at))?;
        ascends("renown", self.renown.iter().map(|rank| rank.at))?;
        self.check_stations()?;
        self.check_milestones(catalogue)?;
        self.check_ids()?;
        self.check_opens(catalogue)?;
        self.check_gated(catalogue)?;

        let Some(unknown) = self
            .earns
            .keys()
            .find(|name| !catalogue.instruments.contains(&name.as_str()))
        else {
            return Ok(());
        };
        Err(super::ContentError::new(
            FILE,
            format!(
                "`{unknown}` earns experience and is not an instrument. One of: {}",
                catalogue.instruments.join(", "),
            ),
        ))
    }

    /// A station is a step or a fork, never both and never neither.
    fn check_stations(&self) -> Result<(), super::ContentError> {
        for station in &self.ley_line {
            match (&station.grants, station.is_fork()) {
                (Some(_), true) => {
                    return Err(super::ContentError::new(
                        FILE,
                        format!(
                            "ley_line station at {} both grants and forks; a station is one or the other",
                            station.at,
                        ),
                    ));
                }
                (None, false) => {
                    return Err(super::ContentError::new(
                        FILE,
                        format!(
                            "ley_line station at {} neither grants nor forks; a station is one or the other",
                            station.at,
                        ),
                    ));
                }
                // **A `grants` nobody implements gives the step nothing**, which
                // reads exactly like a step deliberately authored as a marker —
                // and the curve would stop half way up with the file correct on
                // its face.
                (Some(grants), false) if !GRANTS.contains(&grants.as_str()) => {
                    return Err(super::ContentError::new(
                        FILE,
                        format!(
                            "ley_line step at {} grants `{grants}`, which is nothing. One of: {}",
                            station.at,
                            GRANTS.join(", "),
                        ),
                    ));
                }
                _ => {}
            }
            // **Every node of a fork grants something, and the parser decides.**
            // There are no markers: a node nothing implements would draw, be
            // aimed at, and be refused — which is a promise about a thing
            // nobody has built, the shape §19 refused for the tree.
            if let Some(node) = station
                .nodes
                .iter()
                .find(|node| crate::tower::granted(node).is_none())
            {
                return Err(super::ContentError::new(
                    FILE,
                    format!(
                        "`{node}` at {} is a fork node the orb cannot parse. A node is \
                         satchel_1, cursors_1, or <stem>_<n> with a stem among: {}",
                        station.at,
                        crate::tower::grant::STEMS.join(", "),
                    ),
                ));
            }
            // **One node per lane.** A fork is a choice between kinds of play —
            // more resources, better combat, a faster orb — and two craft
            // nodes at one fork would be a choice inside one kind, which is not
            // what the designer asked the line to offer.
            let mut lanes: Vec<crate::tower::Lane> = Vec::new();
            for node in &station.nodes {
                let Some(lane) = crate::tower::granted(node).map(crate::tower::Grant::lane) else {
                    continue;
                };
                if lanes.contains(&lane) {
                    return Err(super::ContentError::new(
                        FILE,
                        format!(
                            "the fork at {} holds two {} nodes, and a fork offers one per lane",
                            station.at,
                            lane.word(),
                        ),
                    ));
                }
                lanes.push(lane);
            }
        }
        Ok(())
    }

    /// Every milestone sits on a real room and asks for something countable.
    fn check_milestones(&self, catalogue: &Catalogue<'_>) -> Result<(), super::ContentError> {
        for milestone in &self.mastery {
            if !crate::tower::DOMAINS.contains(&milestone.domain.as_str()) {
                return Err(super::ContentError::new(
                    FILE,
                    format!(
                        "`{}` is on the `{}` line, and there is no such room. One of: {}",
                        milestone.id,
                        milestone.domain,
                        crate::tower::DOMAINS.join(", "),
                    ),
                ));
            }
            if let Err(why) = milestone
                .done
                .check(catalogue.outputs, catalogue.instruments)
            {
                return Err(super::ContentError::new(
                    FILE,
                    format!("`{}`: {why}", milestone.id),
                ));
            }
        }
        Ok(())
    }

    /// An id names one thing, across both tracks.
    ///
    /// Ids reach decisions — what has been taken or reached is stored by id, and
    /// a sentence is keyed by id — so a duplicate makes taking one take both, and
    /// makes the second unreachable in the prose. A step's id is its grant, so
    /// those are in the set too.
    fn check_ids(&self) -> Result<(), super::ContentError> {
        let mut seen: Vec<&str> = GRANTS.to_vec();
        let ids = self
            .ley_line
            .iter()
            .flat_map(|station| station.nodes.iter())
            .chain(self.mastery.iter().map(|milestone| &milestone.id))
            .chain(self.renown.iter().map(|rank| &rank.id));
        for id in ids {
            if seen.contains(&id.as_str()) {
                return Err(super::ContentError::new(
                    FILE,
                    format!("`{id}` names two things, and an id names one"),
                ));
            }
            seen.push(id);
        }
        Ok(())
    }

    /// Every gated product is opened by some station.
    ///
    /// **The other direction, and the half that was missing.** `check_opens`
    /// refuses a key that names nothing; nothing refused a *product* that no key
    /// names. Because `Opened::start` is *everything no station opens*, such a
    /// product is not unreachable — it is handed to the player at tick 0, with
    /// the file correct on its face and every test green. `gated = true` says
    /// *earned*, so a station losing its `opens` line is the
    /// indistinguishable-typo failure this module's header is about, arriving
    /// from the side the check did not cover.
    fn check_gated(&self, catalogue: &Catalogue<'_>) -> Result<(), super::ContentError> {
        let opened: Vec<&str> = self
            .ley_line
            .iter()
            .flat_map(|station| station.opens.iter())
            .chain(
                self.mastery
                    .iter()
                    .flat_map(|milestone| milestone.opens.iter()),
            )
            .map(String::as_str)
            .collect();
        for product in catalogue.gated {
            let key = crate::tower::recipe_key(product);
            if !opened.iter().any(|open| *open == key) {
                return Err(super::ContentError::new(
                    FILE,
                    format!(
                        "`{product}` is gated and no station opens it, so every tower \
                         starts holding it. Give a station `opens = [\"{key}\"]`, or take \
                         `gated` off the recipe"
                    ),
                ));
            }
        }
        Ok(())
    }

    /// Every `opens` names something that can be opened.
    fn check_opens(&self, catalogue: &Catalogue<'_>) -> Result<(), super::ContentError> {
        let opens = self
            .ley_line
            .iter()
            .map(|station| (station.at.to_string(), &station.opens))
            .chain(
                self.mastery
                    .iter()
                    .map(|milestone| (milestone.id.clone(), &milestone.opens)),
            );
        for (who, keys) in opens {
            for key in keys {
                let Some(parsed) = crate::tower::opened::Key::parse(key) else {
                    return Err(super::ContentError::new(
                        FILE,
                        format!(
                            "`{who}` opens `{key}`, which is not a room, a recipe, a charm or the wall"
                        ),
                    ));
                };
                let (known, of): (bool, &[&str]) = match &parsed {
                    // **`is_room`, not `DOMAINS`.** A station may open a room
                    // that is not one you *work* in — the bailey and the
                    // grimoire are both shut until they are earned and neither
                    // has a mastery line. Checking the narrower list refused the
                    // ley step at 16 that opens the grimoire.
                    crate::tower::opened::Key::Domain(name) => {
                        (crate::tower::opened::is_room(name), &crate::tower::DOMAINS)
                    }
                    crate::tower::opened::Key::Recipe(name) => {
                        (catalogue.gated.contains(&name.as_str()), catalogue.gated)
                    }
                    crate::tower::opened::Key::Charm(name) => {
                        (catalogue.charms.contains(&name.as_str()), catalogue.charms)
                    }
                    crate::tower::opened::Key::Siege => (true, &[]),
                };
                if !known {
                    return Err(super::ContentError::new(
                        FILE,
                        format!(
                            "`{who}` opens `{key}`, which nothing can open. One of: {}",
                            of.join(", "),
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    /// The Ley Line, in order.
    #[must_use]
    pub fn ley_line(&self) -> &[Station] {
        &self.ley_line
    }

    /// Every mastery station, in the file's order.
    #[must_use]
    pub fn mastery(&self) -> &[Milestone] {
        &self.mastery
    }

    /// Every rank, lowest first.
    #[must_use]
    pub fn renown(&self) -> &[Rank] {
        &self.renown
    }

    /// One room's line, in order.
    pub fn line<'a>(&'a self, domain: &'a str) -> impl Iterator<Item = &'a Milestone> + 'a {
        self.mastery
            .iter()
            .filter(move |milestone| milestone.domain == domain)
    }

    /// What one completed run at `named` is worth.
    ///
    /// Zero for anything unlisted, which after [`check`](Self::check) can only
    /// be something with no recipes at all.
    #[must_use]
    pub fn earns(&self, named: &str) -> u64 {
        self.earns.get(named).copied().unwrap_or_default()
    }

    /// How many spells the orb can hold at `experience`.
    ///
    /// **Derived, never stored.** The level is a function of one number against
    /// this table, so a save carries the number and nothing can fall out of step
    /// with it — the same shape as a spell's `Program` being derived from its
    /// text rather than kept beside it. `check` refuses an unsorted track, so
    /// counting what has been passed is the whole of it.
    #[must_use]
    pub fn concentration(&self, experience: u64) -> usize {
        self.granting(CONCENTRATION)
            .take_while(|needed| *needed <= experience)
            .count()
    }

    /// How many ley steps granting quintessence have been passed.
    ///
    /// **`concentration`'s twin, and deliberately a second method rather than a
    /// public `granting`.** The steps' grants are a closed set (`GRANTS`), so
    /// every reader of it is a named question about a named grant — exposing
    /// the iterator would invite a caller to ask about a word the set does not
    /// hold and silently get nought.
    #[must_use]
    pub fn quintessence(&self, experience: u64) -> usize {
        self.granting(QUINTESSENCE)
            .take_while(|needed| *needed <= experience)
            .count()
    }

    /// What the next level of concentration costs, if there is one.
    ///
    /// The number a player is working toward. `None` at the top of the table,
    /// which is *"nothing more is authored yet"* rather than *"you are finished"*.
    #[must_use]
    pub fn next_concentration(&self, experience: u64) -> Option<u64> {
        self.granting(CONCENTRATION)
            .find(|needed| *needed > experience)
    }

    /// Every step total that grants `what`, in order.
    fn granting<'a>(&'a self, what: &'a str) -> impl Iterator<Item = u64> + 'a {
        self.ley_line
            .iter()
            .filter(move |station| station.grants.as_deref() == Some(what))
            .map(|station| station.at)
    }
}

/// Refuse a track whose totals do not strictly ascend.
///
/// **The sort is the meaning of the list**, because every reading of it counts
/// with `take_while` and stops at the first total it cannot afford. `[30, 16]`
/// gates the second step behind 30 *and* never awards the first at 16 — a track
/// that reads as authored and behaves as neither, with no verb refusing and
/// nothing to look at.
fn ascends(track: &str, totals: impl Iterator<Item = u64>) -> Result<(), super::ContentError> {
    let totals: Vec<u64> = totals.collect();
    let Some(pair) = totals.windows(2).find(|two| {
        let [first, second] = two else { return false };
        second <= first
    }) else {
        return Ok(());
    };
    Err(super::ContentError::new(
        FILE,
        format!(
            "{track} must ascend, and {} does not follow {}: {totals:?}",
            pair[1], pair[0],
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped tower's names, as `Sim::new` hands them in.
    const INSTRUMENTS: [&str; 10] = [
        "mortar_and_pestle",
        "balneum_mariae",
        "flask_and_rod",
        "alembic",
        "athanor",
        "lectern",
        "stacks",
        "prism",
        "pylon",
        "lattice",
    ];

    fn catalogue<'a>(
        outputs: &'a [&'a str],
        gated: &'a [&'a str],
        charms: &'a [&'a str],
    ) -> Catalogue<'a> {
        Catalogue {
            instruments: &INSTRUMENTS,
            outputs,
            gated,
            charms,
        }
    }

    /// The shipped file, checked against the shipped content — the same call
    /// `Sim::new` makes, so a test here fails before a tower does.
    fn shipped_check(curve: &Progression) -> Result<(), super::super::ContentError> {
        let recipes = super::super::Recipes::builtin();
        let outputs = recipes.outputs();
        let gated = recipes.gated();
        let charms = super::super::Charms::builtin();
        let charms: Vec<&str> = charms.names().collect();
        curve.check(&catalogue(&outputs, &gated, &charms))
    }

    #[test]
    fn the_builtin_file_parses() {
        let curve = Progression::builtin();
        assert_eq!(curve.earns("mortar_and_pestle"), 1);
        assert_eq!(curve.earns("balneum_mariae"), 2);
        assert_eq!(curve.earns("flask_and_rod"), 4);
        assert_eq!(curve.earns("alembic"), 8);
        assert_eq!(curve.earns("lectern"), 4);
        shipped_check(&curve).expect("the shipped file does not check");
    }

    #[test]
    fn one_clarity_is_the_first_threshold() {
        // **The number the whole curve is anchored to**, spelled out here rather
        // than trusted: grind sage, digest, grind salt, mix, distil. If a weight
        // moves and the threshold does not, this is what says so.
        let curve = Progression::builtin();
        let clarity = curve.earns("mortar_and_pestle")
            + curve.earns("balneum_mariae")
            + curve.earns("mortar_and_pestle")
            + curve.earns("flask_and_rod")
            + curve.earns("alembic");

        assert_eq!(clarity, 16);
        assert_eq!(curve.concentration(clarity - 1), 0, "it arrived early");
        assert_eq!(curve.concentration(clarity), 1, "one clarity is not enough");
        assert_eq!(curve.next_concentration(0), Some(16));
    }

    #[test]
    fn the_shipped_curve_reads_the_same_at_every_total_that_matters() {
        // The whole game is anchored to 16 meaning one clarity: `tests/
        // progression.rs` walks a brew to it, `tests/binding.rs` reaches a slot
        // through it, and `bind` refuses below it. So this pins the *readings*,
        // exhaustively across the interesting range and independently of how
        // the file is shaped underneath.
        let curve = Progression::builtin();
        for total in 0..=40 {
            let expected = usize::from(total >= 16);
            assert_eq!(
                curve.concentration(total),
                expected,
                "concentration moved at {total}",
            );
        }
        assert_eq!(curve.next_concentration(0), Some(16));
        assert_eq!(curve.next_concentration(15), Some(16));
        assert_eq!(
            curve.next_concentration(16),
            Some(96),
            "the second level is not where the line puts it",
        );
    }

    #[test]
    fn an_instrument_that_does_not_exist_fails_the_load() {
        // A key nobody can name earns nothing, which is indistinguishable from a
        // number somebody chose. `materials.toml` records paying for this once.
        let curve = Progression::builtin();
        let recipes = super::super::Recipes::builtin();
        let outputs = recipes.outputs();
        let gated = recipes.gated();
        let charms = super::super::Charms::builtin();
        let charms: Vec<&str> = charms.names().collect();
        let short = Catalogue {
            instruments: &["mortar_and_pestle"],
            outputs: &outputs,
            gated: &gated,
            charms: &charms,
        };
        assert!(curve.check(&short).is_err());
    }

    /// An `[earns]` table and whatever else the test needs.
    fn authored(rest: &str) -> Progression {
        Progression::parse(&format!("[earns]\nmortar_and_pestle = 1\n{rest}"))
            .expect("the test authored invalid TOML")
    }

    /// One ley-line step per total, all granting concentration.
    fn ley_line(totals: &[u64]) -> String {
        totals
            .iter()
            .map(|at| format!("[[ley_line]]\nat = {at}\ngrants = \"concentration\"\n"))
            .collect()
    }

    /// A check against a tower with one potion and no charms.
    fn small(curve: &Progression) -> Result<(), super::super::ContentError> {
        curve.check(&catalogue(&["clarity"], &[], &[]))
    }

    #[test]
    fn levels_are_counted_rather_than_looked_up() {
        let curve = authored(&ley_line(&[10, 30, 90]));

        for (experience, expected) in [(0, 0), (9, 0), (10, 1), (29, 1), (30, 2), (900, 3)] {
            assert_eq!(curve.concentration(experience), expected, "at {experience}");
        }
        assert_eq!(curve.next_concentration(10), Some(30));
        assert_eq!(curve.next_concentration(900), None, "past the last level");
    }

    #[test]
    fn the_section_this_replaced_fails_the_load_rather_than_being_ignored() {
        // **The migration hazard, pinned.** Both tracks are `serde(default)` so a
        // file may omit them — which meant the old `[concentration] levels = [16]`
        // parsed happily into a tower with *no curve at all*: every threshold
        // gone, no verb refusing, and the file correct on its face.
        assert!(
            Progression::parse("[earns]\nmortar_and_pestle = 1\n[concentration]\nlevels = [16]\n")
                .is_err(),
            "a section nothing reads was accepted",
        );
        // ...and the tiers the old tree was written as, which the forks replaced.
        assert!(
            Progression::parse(
                "[earns]\nmortar_and_pestle = 1\n[[mastery]]\nat = 24\nnodes = [\"a\"]\n"
            )
            .is_err(),
            "a mastery tier parsed as a station",
        );
    }

    #[test]
    fn a_curve_that_does_not_ascend_fails_the_load() {
        // **`take_while` stops at the first level it cannot afford**, so
        // `[30, 16]` would gate level 2 behind 30 *and* never award level 1 at
        // 16 — a table that reads as authored and behaves as neither.
        let out_of_order = authored(&ley_line(&[30, 16]));
        assert_eq!(
            out_of_order.concentration(16),
            0,
            "the reading this refuses is not the one that was broken",
        );
        assert!(small(&out_of_order).is_err());

        // A repeat is the same fault: a second level bought by the same number
        // is a level nobody can work toward.
        assert!(small(&authored(&ley_line(&[16, 16]))).is_err());
        shipped_check(&Progression::builtin()).expect("the shipped curve does not ascend");
    }

    #[test]
    fn a_step_that_grants_nothing_fails_the_load() {
        assert!(
            small(&authored(
                "[[ley_line]]\nat = 16\ngrants = \"cncentration\"\n"
            ))
            .is_err(),
            "a typo in `grants` was accepted",
        );
        assert!(
            small(&authored(&ley_line(&[16]))).is_ok(),
            "the spelling that works was refused",
        );
    }

    #[test]
    fn a_station_is_a_step_or_a_fork_and_never_both() {
        assert!(
            small(&authored(
                "[[ley_line]]\nat = 16\ngrants = \"concentration\"\nnodes = [\"steps_1\"]\n"
            ))
            .is_err(),
            "a station that both grants and forks was accepted",
        );
        assert!(
            small(&authored("[[ley_line]]\nat = 16\n")).is_err(),
            "a station that does nothing was accepted",
        );
    }

    #[test]
    fn a_fork_node_the_orb_cannot_parse_fails_the_load() {
        // **There are no markers.** A node nothing implements would draw, be
        // aimed at, and be refused — a promise about a thing nobody has built.
        assert!(
            small(&authored(
                "[[ley_line]]\nat = 24\nnodes = [\"steps_1\", \"tbi_b\"]\n"
            ))
            .is_err(),
            "a marker was accepted on a fork",
        );
        assert!(
            small(&authored("[[ley_line]]\nat = 24\nnodes = [\"satchel1\"]\n")).is_err(),
            "a near-miss grant was accepted",
        );
        assert!(
            small(&authored(
                "[[ley_line]]\nat = 24\nnodes = [\"fuel_1\", \"steps_1\"]\n"
            ))
            .is_ok(),
        );
    }

    #[test]
    fn a_gated_product_no_station_opens_fails_the_load() {
        // **The half `check_opens` does not cover.** A key naming nothing is
        // refused; a *product* nothing names was not — and because
        // `Opened::start` is everything no station opens, the product is then
        // handed to every tower at tick 0, with the file correct on its face.
        // A station losing its `opens` line looks exactly like a designer
        // deciding the thing should be free.
        let curve = authored(&ley_line(&[16]));
        let gated = curve.check(&catalogue(&["clarity", "warding"], &["warding"], &[]));
        assert!(gated.is_err(), "a gated product nothing opens was accepted");

        let opened = authored(
            "[[ley_line]]\nat = 16\ngrants = \"concentration\"\nopens = [\"recipe:warding\"]\n",
        );
        assert!(
            opened
                .check(&catalogue(&["clarity", "warding"], &["warding"], &[]))
                .is_ok(),
            "a gated product a station does open was refused",
        );
    }

    #[test]
    fn a_fork_offers_one_node_per_lane() {
        assert!(
            small(&authored(
                "[[ley_line]]\nat = 24\nnodes = [\"steps_1\", \"haste_1\"]\n"
            ))
            .is_err(),
            "two craft nodes on one fork were accepted",
        );
        assert!(
            small(&authored(
                "[[ley_line]]\nat = 24\nnodes = [\"fuel_1\", \"edge_1\", \"steps_1\"]\n"
            ))
            .is_ok(),
            "one node per lane was refused",
        );
    }

    #[test]
    fn an_id_names_one_thing_across_both_tracks() {
        assert!(
            small(&authored(
                "[[ley_line]]\nat = 24\nnodes = [\"fuel_1\", \"fuel_1\"]\n"
            ))
            .is_err(),
            "a fork repeated an id",
        );
        assert!(
            small(&authored(
                "[[ley_line]]\nat = 24\nnodes = [\"steps_1\"]\n[[ley_line]]\nat = 40\nnodes = [\"steps_1\"]\n"
            ))
            .is_err(),
            "two forks shared an id",
        );
        assert!(
            small(&authored(
                "[[ley_line]]\nat = 24\nnodes = [\"steps_1\"]\n\
                 [[mastery]]\ndomain = \"laboratory\"\nid = \"steps_1\"\ndone = \"clarity\"\n"
            ))
            .is_err(),
            "a mastery station borrowed a fork node's id",
        );
        assert!(
            small(&authored(
                "[[mastery]]\ndomain = \"laboratory\"\nid = \"concentration\"\ndone = \"clarity\"\n"
            ))
            .is_err(),
            "a mastery station borrowed a step's id",
        );
    }

    #[test]
    fn a_milestone_sits_on_a_real_room_and_asks_for_something_countable() {
        assert!(
            small(&authored(
                "[[mastery]]\ndomain = \"kitchen\"\nid = \"kitchen_1\"\ndone = \"clarity\"\n"
            ))
            .is_err(),
            "a line for a room that does not exist was accepted",
        );
        assert!(
            small(&authored(
                "[[mastery]]\ndomain = \"laboratory\"\nid = \"laboratory_1\"\ndone = \"clarty\"\n"
            ))
            .is_err(),
            "a deed naming no product was accepted",
        );
        assert!(
            small(&authored(
                "[[mastery]]\ndomain = \"laboratory\"\nid = \"laboratory_1\"\ndone = \"clarity\"\n"
            ))
            .is_ok(),
        );
    }

    #[test]
    fn an_opens_names_something_that_can_be_opened() {
        // The first station is here to satisfy `check_gated`, which wants the
        // catalogue's one gated product opened by *something*: this test is
        // about the other direction, and without it every case would fail for
        // the other check's reason.
        let curve = |opens: &str| {
            authored(&format!(
                "[[mastery]]\ndomain = \"laboratory\"\nid = \"laboratory_1\"\n\
                 done = \"clarity\"\nopens = [\"recipe:warding\"]\n\
                 [[mastery]]\ndomain = \"laboratory\"\nid = \"laboratory_2\"\n\
                 done = \"clarity\"\nopens = [\"{opens}\"]\n"
            ))
        };
        let check =
            |curve: &Progression| curve.check(&catalogue(&["clarity"], &["warding"], &["hurried"]));
        assert!(check(&curve("domain:archive")).is_ok());
        assert!(
            check(&curve("domain:kitchen")).is_err(),
            "a room that does not exist"
        );
        assert!(check(&curve("recipe:warding")).is_ok());
        assert!(
            check(&curve("recipe:clarity")).is_err(),
            "a recipe nothing gates"
        );
        assert!(check(&curve("charm:hurried")).is_ok());
        assert!(
            check(&curve("charm:hurrying")).is_err(),
            "a charm that does not exist"
        );
        assert!(check(&curve("siege")).is_ok());
        assert!(check(&curve("wall")).is_err(), "a key of no kind");
    }

    #[test]
    fn the_shipped_line_is_stations_and_every_fork_node_is_real() {
        let curve = Progression::builtin();
        let steps: Vec<(u64, &str)> = curve
            .ley_line()
            .iter()
            .filter_map(|station| Some((station.at, station.grants.as_deref()?)))
            .collect();
        assert_eq!(
            steps,
            vec![
                (16, CONCENTRATION),
                (56, QUINTESSENCE),
                (96, CONCENTRATION),
                (256, CONCENTRATION),
                (640, CONCENTRATION),
                (1600, CONCENTRATION),
                (4000, CONCENTRATION),
                (8000, CONCENTRATION),
                (10_000, CONCENTRATION),
            ]
        );
        let forks: Vec<u64> = curve
            .ley_line()
            .iter()
            .filter(|station| station.is_fork())
            .map(|station| station.at)
            .collect();
        assert_eq!(forks, vec![24, 40, 160, 400, 1000, 2500, 6400]);
        // The soft ending is concentration 8, and the line runs to it.
        assert_eq!(
            curve.concentration(10_000),
            8,
            "the line does not reach the soft ending"
        );
        assert_eq!(curve.concentration(9_999), 7);
        for node in curve.ley_line().iter().flat_map(|station| &station.nodes) {
            assert!(
                crate::tower::granted(node).is_some(),
                "{node} is a fork node nothing reads",
            );
        }
        // Every room has a line, and the laboratory's begins with the potion
        // the tutorial teaches.
        for domain in crate::tower::DOMAINS {
            assert!(
                curve.line(domain).next().is_some(),
                "the {domain} has no line",
            );
        }
        assert_eq!(
            curve.line("laboratory").next().map(|m| m.done.key()),
            Some("made:clarity".to_owned()),
        );
    }
}
