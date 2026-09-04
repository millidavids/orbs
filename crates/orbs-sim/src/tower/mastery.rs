//! Mastery: seven straight lines, one per room (DESIGN.md §11.5, §19).
//!
//! # Per domain, no choices
//!
//! Each room has a line of stations, and a station is a *deed* — brew a
//! clarity, walk the stacks five times, close a figure. A line is walked in
//! order: a station is reached when the one before it is reached and its deed's
//! count in the [`Tally`] is met, and reaching it *is* the grant.
//! Nothing here is ever `take`n, and the screen says so.
//!
//! The choices live on the Ley Line (`tower::ley`), which is where the tree
//! this module used to read has gone.
//!
//! # Reaching is said once, and what it opens is said once
//!
//! [`advance`] runs after every completion, on the tick the work landed. A
//! station reached is a record, so `sift` and the log see it; what it opened is
//! a second record, said only if the thing was shut — a tower restored open
//! that re-reaches the station opening the archive has an archive already.
//!
//! # No stream, no `Submission`
//!
//! A deed is done by work the submissions already record, and reaching is a
//! function of the tally against the authored file. Nothing is drawn and nothing
//! is chosen, so nothing here has to replay on its own.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Progression, Prose};
use crate::session::Scrollback;

use super::Tally;
use super::opened::{Opened, open, opening};

/// Which mastery stations the tower has reached.
///
/// **A list of ids**, as [`Taken`](super::Taken) is, and for the same reasons.
/// Saved so a restore need not replay to know it, and never regenerated from
/// the tally — the tally says a deed is done; this says its station was reached
/// *and said*, which a migrated save must not do twice.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Reached(Vec<String>);

impl Reached {
    /// Put a set back, for a save.
    pub(crate) fn restore(&mut self, ids: Vec<String>) {
        self.0 = ids;
    }

    /// The ids reached, in the order they were reached.
    #[must_use]
    pub fn ids(&self) -> &[String] {
        &self.0
    }

    /// Whether `id` has been reached.
    #[must_use]
    pub fn has(&self, id: &str) -> bool {
        self.0.iter().any(|held| held == id)
    }

    fn hold(&mut self, id: &str) {
        if !self.has(id) {
            self.0.push(id.to_owned());
        }
    }
}

/// Where a station on a line stands.
///
/// **Three states, three words** (§14). Not [`Standing`](super::Standing),
/// which is the fork's vocabulary — a fork node can be open and unchosen, and
/// a station on a line cannot: it is done, it is the one being worked toward,
/// or it is further along.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Walk {
    /// Done.
    Reached,
    /// The one the room is working toward.
    Next,
    /// Further along the line.
    Later,
}

impl Walk {
    /// The word a screen reader hears.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Reached => "reached",
            Self::Next => "next",
            Self::Later => "later",
        }
    }
}

/// One station on a room's line, as the screen draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stop {
    /// The id, which is the content file's word and the prose key.
    pub id: String,
    /// Where it stands.
    pub walk: Walk,
    /// How much of its deed is done, capped at what it asks.
    pub done: u32,
    /// What its deed asks.
    pub needed: u32,
    /// What reaching it opens.
    pub opens: Vec<String>,
}

impl Stop {
    /// What a cursor holds to mean *this* station.
    ///
    /// A mastery id already names one thing across both tracks
    /// (`Progression::check`), so the id serves. It exists so the two tracks
    /// answer the same question the same way — see [`Node::mark`](super::Node::mark),
    /// where the id alone is not enough.
    #[must_use]
    pub fn mark(&self) -> &str {
        &self.id
    }
}

/// One room's line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// The room, from [`DOMAINS`](super::DOMAINS).
    pub domain: &'static str,
    /// Whether the room may be entered at all.
    pub open: bool,
    /// Its stations, in order.
    pub stops: Vec<Stop>,
}

impl Line {
    /// How far along: stations reached, of how many.
    #[must_use]
    pub fn reached(&self) -> (usize, usize) {
        (
            self.stops
                .iter()
                .filter(|stop| stop.walk == Walk::Reached)
                .count(),
            self.stops.len(),
        )
    }

    /// The station being worked toward, if the line is not finished.
    #[must_use]
    pub fn next(&self) -> Option<&Stop> {
        self.stops.iter().find(|stop| stop.walk == Walk::Next)
    }
}

/// Every room's line, in [`DOMAINS`](super::DOMAINS) order.
#[must_use]
pub fn mastery(world: &World) -> Vec<Line> {
    let curve = world.resource::<Progression>();
    let tally = world.resource::<Tally>();
    let reached = world.resource::<Reached>();
    let opened = world.resource::<Opened>();
    super::DOMAINS
        .into_iter()
        .map(|domain| {
            let mut seen_next = false;
            let stops = curve
                .line(domain)
                .map(|milestone| {
                    let needed = milestone.done.times();
                    let done = tally.count(&milestone.done.key()).min(needed);
                    let walk = if reached.has(&milestone.id) {
                        Walk::Reached
                    } else if seen_next {
                        Walk::Later
                    } else {
                        seen_next = true;
                        Walk::Next
                    };
                    Stop {
                        id: milestone.id.clone(),
                        walk,
                        done,
                        needed,
                        opens: milestone.opens.clone(),
                    }
                })
                .collect();
            Line {
                domain,
                open: opened.is_open(domain),
                stops,
            }
        })
        .collect()
}

/// How far along one room is: `(reached, of, (done, needed))` toward the next.
///
/// **The rail's reading**, in three numbers rather than a `Line`, because a
/// rail box has one row to put it on. `None` for a line that is finished or a
/// room that has no line.
#[must_use]
pub fn progress(world: &World, domain: &str) -> Option<Progress> {
    mastery(world)
        .into_iter()
        .find(|line| line.domain == domain)?
        .progress()
}

impl Line {
    /// How far along this line is, or `None` once it is finished.
    #[must_use]
    pub fn progress(&self) -> Option<Progress> {
        let (reached, of) = self.reached();
        let next = self.next()?;
        Some(Progress {
            reached,
            of,
            toward: (next.done, next.needed),
        })
    }
}

/// How far along a room's line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Stations reached.
    pub reached: usize,
    /// Stations on the line.
    pub of: usize,
    /// The next station's deed: done, of needed.
    pub toward: (u32, u32),
}

impl Progress {
    /// How much of the next deed is done, as a percentage.
    #[must_use]
    pub fn percent(self) -> u32 {
        let (done, needed) = self.toward;
        if needed == 0 {
            return 100;
        }
        (u64::from(done) * 100 / u64::from(needed)).min(100) as u32
    }
}

/// Reach every station whose deed is done, in line order, and say so.
///
/// Called by [`done`](super::done) after every completion. Cheap: seven lines
/// of a handful of stations, read against a map.
pub fn advance(world: &mut World) {
    let due: Vec<(String, String, Vec<String>)> = {
        let curve = world.resource::<Progression>();
        let tally = world.resource::<Tally>();
        let reached = world.resource::<Reached>();
        let mut due = Vec::new();
        for domain in super::DOMAINS {
            for milestone in curve.line(domain) {
                if reached.has(&milestone.id) {
                    continue;
                }
                if tally.count(&milestone.done.key()) < milestone.done.times() {
                    break;
                }
                due.push((
                    domain.to_owned(),
                    milestone.id.clone(),
                    milestone.opens.clone(),
                ));
            }
        }
        due
    };
    for (domain, id, opens) in due {
        reach(world, &domain, &id, &opens);
    }
}

/// Open what every station already reached opens, without saying so.
///
/// **[`advance`] reaches a station once, ever** — it skips anything already in
/// [`Reached`] — so what a station opens is applied on exactly one tick in the
/// life of a save. A document whose station gained an `opens` after it was
/// written, or one written by a build that applied it wrongly, comes back with
/// the thing still shut and no deed able to open it: the tally is long past what
/// the station asks, so `advance` walks straight past it for ever.
///
/// [`ley::caught_up`](super::ley::caught_up)'s twin, and silent for its reason.
pub(crate) fn caught_up(world: &mut World) {
    let opens: Vec<String> = {
        let curve = world.resource::<Progression>();
        let reached = world.resource::<Reached>();
        super::DOMAINS
            .into_iter()
            .flat_map(|domain| curve.line(domain))
            .filter(|milestone| reached.has(&milestone.id))
            .flat_map(|milestone| milestone.opens.iter().cloned())
            .collect()
    };
    for key in opens {
        open(world, &key);
    }
}

/// Mark `id` reached, say so, and open what it opens.
///
/// **Two records, not one.** The station is one fact and what it opened is
/// another, and only the second is conditional — see the module header.
pub(crate) fn reach(world: &mut World, domain: &str, id: &str, opens: &[String]) {
    world.resource_mut::<Reached>().hold(id);
    let sentence = world
        .resource::<Prose>()
        .line(&format!("mastery_{id}"), &[]);
    let message = world.resource::<Prose>().line(
        "mastery_reached",
        &[("name", domain), ("detail", &sentence)],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, id)
        .text(FieldName::Source, domain)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();

    opening(world, opens);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;
    use crate::tower::Work;

    fn said(sim: &Sim) -> Vec<String> {
        sim.scrollback()
            .records()
            .iter()
            .filter_map(|record| record.field(FieldName::Message))
            .filter_map(|value| match value {
                orbs_render::Value::Text(text) => Some(text.to_owned()),
                _ => None,
            })
            .collect()
    }

    fn line(sim: &Sim, domain: &str) -> Line {
        mastery(sim.world())
            .into_iter()
            .find(|line| line.domain == domain)
            .expect("no such line")
    }

    #[test]
    fn a_cold_tower_has_a_line_per_room_and_is_working_toward_the_first_station_of_each() {
        let sim = Sim::new(1);
        let lines = mastery(sim.world());
        // Six, since the grimoire left `DOMAINS` (§19): its three stations
        // counted a deed done everywhere else and opened nothing.
        assert_eq!(lines.len(), super::super::DOMAINS.len());
        for line in &lines {
            assert!(!line.stops.is_empty(), "the {} has no line", line.domain);
            assert_eq!(line.reached().0, 0, "the {} started ahead", line.domain);
            assert_eq!(line.stops[0].walk, Walk::Next, "{}", line.domain);
            assert!(
                line.stops[1..].iter().all(|stop| stop.walk == Walk::Later),
                "{}",
                line.domain,
            );
            assert!(line.open, "{} is shut in an open tower", line.domain);
        }
    }

    #[test]
    fn a_deed_done_reaches_the_station_and_says_so() {
        let mut sim = Sim::new(1);
        super::super::done(
            sim.world_mut(),
            &Work::made("alembic", "clarity", true, false),
            8,
        );
        let laboratory = line(&sim, "laboratory");
        assert_eq!(laboratory.stops[0].walk, Walk::Reached);
        assert_eq!(laboratory.stops[1].walk, Walk::Next);
        assert_eq!(laboratory.stops[1].done, 1, "the potion was not counted");
        assert!(
            said(&sim).iter().any(|line| line.contains("laboratory")),
            "reaching was not said: {:?}",
            said(&sim),
        );
    }

    #[test]
    fn a_line_is_walked_in_order_and_a_late_deed_reaches_two_at_once() {
        // Five potions before the first clarity: nothing is reached until the
        // clarity lands, and then both stations do, in order, each said.
        let mut sim = Sim::new(1);
        for _ in 0..5 {
            super::super::done(
                sim.world_mut(),
                &Work::made("alembic", "haste", true, false),
                8,
            );
        }
        assert_eq!(
            line(&sim, "laboratory").reached().0,
            0,
            "a station was skipped to"
        );

        super::super::done(
            sim.world_mut(),
            &Work::made("alembic", "clarity", true, false),
            8,
        );
        let laboratory = line(&sim, "laboratory");
        assert_eq!(
            laboratory.reached().0,
            2,
            "the second station did not follow"
        );
        assert_eq!(
            sim.world().resource::<Reached>().ids(),
            ["laboratory_1", "laboratory_2"],
        );
    }

    #[test]
    fn what_a_station_opens_is_said_only_if_it_was_shut() {
        // `Sim::new` is an open tower, so the archive is open already and
        // reaching the station that opens it must not announce it.
        let mut sim = Sim::new(1);
        super::super::done(
            sim.world_mut(),
            &Work::made("alembic", "clarity", true, false),
            8,
        );
        assert!(
            !said(&sim).iter().any(|line| line.contains("archive")),
            "an open room was announced as opening: {:?}",
            said(&sim),
        );
    }

    #[test]
    fn a_station_is_never_reached_twice() {
        let mut sim = Sim::new(1);
        for _ in 0..3 {
            super::super::done(
                sim.world_mut(),
                &Work::made("alembic", "clarity", true, false),
                8,
            );
        }
        let count = said(&sim)
            .iter()
            .filter(|line| line.contains("laboratory"))
            .count();
        assert_eq!(count, 1, "reaching was said {count} times");
    }

    #[test]
    fn progress_reads_the_next_deed_as_a_percentage() {
        let mut sim = Sim::new(1);
        super::super::done(
            sim.world_mut(),
            &Work::made("alembic", "clarity", true, false),
            8,
        );
        for _ in 0..2 {
            super::super::done(
                sim.world_mut(),
                &Work::made("alembic", "haste", true, false),
                8,
            );
        }
        let laboratory = progress(sim.world(), "laboratory").expect("the line is finished");
        assert_eq!(laboratory.reached, 1);
        assert_eq!(laboratory.toward, (3, 5));
        assert_eq!(laboratory.percent(), 60);
        assert!(progress(sim.world(), "kitchen").is_none());
    }

    #[test]
    fn every_walk_says_a_word_of_its_own() {
        let words: Vec<&str> = [Walk::Reached, Walk::Next, Walk::Later]
            .into_iter()
            .map(Walk::word)
            .collect();
        assert_eq!(words, ["reached", "next", "later"]);
    }
}
