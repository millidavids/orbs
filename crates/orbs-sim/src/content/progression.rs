//! What work is worth, and what it buys (DESIGN.md §11.5).
//!
//! Authored in `content/progression.toml`, not here (rule 6). This module knows
//! the *shape* of the curve and nothing about its numbers.
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
//! # An instrument nobody can name is an error
//!
//! The keys under `[earns]` are checked against `recipes.toml`'s own, and a name
//! matching none of them fails the *load*. An instrument missing from this table
//! earns nothing, which looks exactly like an instrument someone has deliberately
//! priced at zero — the same indistinguishable-typo problem `materials.toml`
//! records paying for once, and `Recipe::heat` and `craft_of` before it.
//!
//! **Validated against other content**, which is new: `materials.toml` checks
//! its colours against a Rust enum, and nothing here can. So the check takes the
//! recipes as an argument rather than reaching for a resource, which keeps it a
//! pure function and lets `Sim::new` decide the order.

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use serde::Deserialize;

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/progression.toml");

/// The file's name, for an error a writer can act on.
const FILE: &str = "progression.toml";

/// What a ley-line step may grant.
///
/// **A closed set, checked at load.** The same rule as `[earns]`'s keys and
/// `materials.toml`'s colours: a `grants` nobody implements would give the step
/// nothing, which reads exactly like a step deliberately authored as a marker —
/// and the file would be correct on its face while the curve quietly stopped
/// half way up.
pub const GRANTS: [&str; 1] = [CONCENTRATION];

/// The only thing the Ley Line grants today.
pub const CONCENTRATION: &str = "concentration";

/// What work is worth, and what it buys.
///
/// **`Default` is the built-in table, not an empty one.** `Fuels` and
/// `Materials` both carry the same hand-written impl, and `Materials` records
/// why: a derived `Default` gives an empty map, `Sim` installs it with
/// `init_resource`, and every run silently earns nothing while the file on disk
/// is perfectly correct.
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
    /// The straight path: predefined steps, granted the moment they are passed.
    ///
    /// **Defaulted**, so a file with no track at all is a tower that earns and
    /// buys nothing rather than a load failure. The tests below author `[earns]`
    /// alone for exactly this reason.
    #[serde(default)]
    ley_line: Vec<Step>,
    /// The branching tree: a tier opens, and one node in it may be taken.
    #[serde(default)]
    mastery: Vec<Tier>,
}

/// One step of the Ley Line.
#[derive(Debug, Clone, Deserialize)]
pub struct Step {
    /// The total that opens it.
    pub at: u64,
    /// What passing it gives. One of [`GRANTS`].
    pub grants: String,
}

/// One tier of Mastery: opens together, and gives exactly one of its nodes.
#[derive(Debug, Clone, Deserialize)]
pub struct Tier {
    /// The total that opens it.
    pub at: u64,
    /// The nodes to choose between. **Ordered**, and the order is the file's.
    pub nodes: Vec<String>,
}

impl Default for Progression {
    fn default() -> Self {
        Self::builtin()
    }
}

impl Progression {
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

    /// Check every `[earns]` key against the instruments that exist, and both
    /// tracks against themselves.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) naming what is wrong. See the module
    /// header for why every one of these is a load failure rather than a warning.
    pub fn check(&self, instruments: &[&str]) -> Result<(), super::ContentError> {
        ascends("ley_line", self.ley_line.iter().map(|step| step.at))?;
        ascends("mastery", self.mastery.iter().map(|tier| tier.at))?;

        // **A `grants` nobody implements gives the step nothing**, which reads
        // exactly like a step deliberately authored as a marker — and the curve
        // would stop half way up with the file correct on its face. The same
        // argument as an unknown `[earns]` key below and an unknown tint in
        // `materials.toml`.
        if let Some(step) = self
            .ley_line
            .iter()
            .find(|step| !GRANTS.contains(&step.grants.as_str()))
        {
            return Err(super::ContentError::new(
                FILE,
                format!(
                    "ley_line step at {} grants `{}`, which is nothing. One of: {}",
                    step.at,
                    step.grants,
                    GRANTS.join(", "),
                ),
            ));
        }

        // **A node id names a node, and two of them name one node.** Ids reach
        // decisions — what has been taken is stored by id, and a sentence is
        // keyed by id — so a duplicate makes taking one take both, and makes the
        // second unreachable in the prose. Nothing else in the file would say so.
        let mut seen: Vec<&str> = Vec::new();
        for node in self.mastery.iter().flat_map(|tier| &tier.nodes) {
            if seen.contains(&node.as_str()) {
                return Err(super::ContentError::new(
                    FILE,
                    format!("`{node}` is a mastery node twice, and an id names one node"),
                ));
            }
            seen.push(node);
        }

        let Some(unknown) = self
            .earns
            .keys()
            .find(|name| !instruments.contains(&name.as_str()))
        else {
            return Ok(());
        };
        Err(super::ContentError::new(
            FILE,
            format!(
                "`{unknown}` earns experience and is not an instrument. One of: {}",
                instruments.join(", "),
            ),
        ))
    }

    /// The Ley Line, in order.
    #[must_use]
    pub fn ley_line(&self) -> &[Step] {
        &self.ley_line
    }

    /// Mastery's tiers, in order.
    #[must_use]
    pub fn mastery(&self) -> &[Tier] {
        &self.mastery
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
    /// text rather than kept beside it.
    /// **Derived from the Ley Line**, which is the same list it always was with
    /// a name and a `grants` on each entry. `check` refuses an unsorted track, so
    /// counting what has been passed is still the whole of it.
    #[must_use]
    pub fn concentration(&self, experience: u64) -> usize {
        self.granting(CONCENTRATION)
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

    /// Every Ley Line total that grants `what`, in order.
    fn granting<'a>(&'a self, what: &'a str) -> impl Iterator<Item = u64> + 'a {
        self.ley_line
            .iter()
            .filter(move |step| step.grants == what)
            .map(|step| step.at)
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

    #[test]
    fn the_builtin_file_parses() {
        let curve = Progression::builtin();
        assert_eq!(curve.earns("mortar_and_pestle"), 1);
        assert_eq!(curve.earns("balneum_mariae"), 2);
        assert_eq!(curve.earns("flask_and_rod"), 4);
        assert_eq!(curve.earns("alembic"), 8);
        // The archive earns through its instrument now, like every other room:
        // the `divine` exception under `[earns]` is gone, and with it the escape
        // it needed from `check`.
        assert_eq!(curve.earns("lectern"), 4);
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
        // **Written before the `[concentration]` → `[ley_line]` restructure, to
        // be run after it.** The whole game is anchored to 16 meaning one
        // clarity: `tests/progression.rs` walks a brew to it, `tests/binding.rs`
        // reaches a slot through it, and `bind` refuses below it. A migration
        // that moved the number by one would leave every one of those still
        // passing against its own new answer, because they each derive from this
        // table rather than pinning it.
        //
        // So this pins the *readings*, exhaustively across the interesting range
        // and independently of how the file is shaped underneath.
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
            None,
            "a second level appeared without being authored",
        );
    }

    #[test]
    fn an_instrument_that_does_not_exist_fails_the_load() {
        // A key nobody can name earns nothing, which is indistinguishable from a
        // number somebody chose. `materials.toml` records paying for this once.
        let curve = Progression::builtin();
        assert!(curve.check(&["mortar_and_pestle"]).is_err());
        assert!(
            curve
                .check(&[
                    "mortar_and_pestle",
                    "balneum_mariae",
                    "flask_and_rod",
                    "alembic",
                    // Both of the archive's, since the maze moved off the
                    // lectern onto the stacks and each earns for its own work.
                    "lectern",
                    "stacks",
                    // The lens's one. Its oculus is deliberately unpriced —
                    // opening a reading is not work — and this table only
                    // refuses keys that name nothing, so an absent name is not
                    // an error.
                    "prism",
                ])
                .is_ok(),
            "the real instruments were rejected",
        );
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
        // gone, no verb refusing, and the file correct on its face. Two tests in
        // this module were silently testing an empty track before
        // `deny_unknown_fields` caught them.
        assert!(
            Progression::parse("[earns]\nmortar_and_pestle = 1\n[concentration]\nlevels = [16]\n")
                .is_err(),
            "a section nothing reads was accepted",
        );
    }

    #[test]
    fn a_curve_that_does_not_ascend_fails_the_load() {
        // **`take_while` stops at the first level it cannot afford**, so
        // `[30, 16]` would gate level 2 behind 30 *and* never award level 1 at
        // 16 — a table that reads as authored and behaves as neither, with no
        // verb refusing and nothing to look at. The sort is the meaning of the
        // list, so an unsorted one is malformed rather than unusual.
        let out_of_order = authored(&ley_line(&[30, 16]));
        assert_eq!(
            out_of_order.concentration(16),
            0,
            "the reading this refuses is not the one that was broken",
        );
        assert!(out_of_order.check(&["mortar_and_pestle"]).is_err());

        // A repeat is the same fault: a second level bought by the same number
        // is a level nobody can work toward.
        assert!(
            authored(&ley_line(&[16, 16]))
                .check(&["mortar_and_pestle"])
                .is_err()
        );

        // ...and Mastery's tiers answer to the same rule, for the same reason.
        assert!(
            authored(
                "[[mastery]]\nat = 40\nnodes = [\"a\"]\n[[mastery]]\nat = 24\nnodes = [\"b\"]\n"
            )
            .check(&["mortar_and_pestle"])
            .is_err(),
            "mastery tiers were allowed to descend",
        );

        assert!(
            Progression::builtin()
                .check(&[
                    "mortar_and_pestle",
                    "balneum_mariae",
                    "flask_and_rod",
                    "alembic",
                    // Both of the archive's, since the maze moved off the
                    // lectern onto the stacks and each earns for its own work.
                    "lectern",
                    "stacks",
                    // The lens's one. Its oculus is deliberately unpriced —
                    // opening a reading is not work — and this table only
                    // refuses keys that name nothing, so an absent name is not
                    // an error.
                    "prism",
                ])
                .is_ok(),
            "the shipped curve does not ascend",
        );
    }

    #[test]
    fn a_step_that_grants_nothing_fails_the_load() {
        // The same argument as an unknown `[earns]` key: a `grants` nobody
        // implements gives the step nothing, which reads exactly like a step
        // authored as a marker — and the track stops half way up with the file
        // correct on its face.
        assert!(
            authored("[[ley_line]]\nat = 16\ngrants = \"cncentration\"\n")
                .check(&["mortar_and_pestle"])
                .is_err(),
            "a typo in `grants` was accepted",
        );
        assert!(
            authored(&ley_line(&[16]))
                .check(&["mortar_and_pestle"])
                .is_ok(),
            "the spelling that works was refused",
        );
    }

    #[test]
    fn a_node_id_names_one_node() {
        // An id is what a taken node is stored as and what its sentence is keyed
        // by, so two entries sharing one make taking either take both — and make
        // the second unreachable in the prose. Across tiers as well as within
        // one, because the id is the key either way.
        assert!(
            authored("[[mastery]]\nat = 24\nnodes = [\"same\", \"same\"]\n")
                .check(&["mortar_and_pestle"])
                .is_err(),
            "a tier repeated an id",
        );
        assert!(
            authored(
                "[[mastery]]\nat = 24\nnodes = [\"same\"]\n[[mastery]]\nat = 40\nnodes = [\"same\"]\n"
            )
            .check(&["mortar_and_pestle"])
            .is_err(),
            "two tiers shared an id",
        );
    }

    #[test]
    fn the_shipped_tree_is_two_tiers_and_nothing_is_takeable() {
        // **The shape is visible from inside the game before anything is behind
        // it**, so a player who reaches 24 sees a tier open and sees that a
        // choice is coming.
        //
        // It asserted that every id began with `tbi`, which was a way of saying
        // *nothing is implemented*. That stopped being the same claim at
        // `0.3.24`: `steps_<n>` nodes are read by `spell::budget`, so the wiring
        // behind them is real while the **taking** is still the weave phase's
        // item. What has to stay true is that nothing can be taken — which is
        // `Taken`'s emptiness, not a spelling rule about ids.
        let curve = Progression::builtin();
        assert_eq!(curve.ley_line().len(), 1, "the ley line grew a step");
        assert_eq!(curve.ley_line()[0].at, 16);
        assert_eq!(curve.mastery().len(), 2);
        assert_eq!(curve.mastery()[0].at, 24);
        assert!(
            crate::tower::Taken::default().ids().is_empty(),
            "a node can be taken now, and this whole tree still ships as markers",
        );
        // Every id is one of the two kinds the game knows how to read. A third
        // spelling would be a node that draws, refuses, and means nothing.
        for node in curve.mastery().iter().flat_map(|tier| &tier.nodes) {
            assert!(
                node.starts_with("tbi") || node.starts_with("steps_"),
                "{node} is neither a marker nor a grant anything reads",
            );
        }
    }
}
