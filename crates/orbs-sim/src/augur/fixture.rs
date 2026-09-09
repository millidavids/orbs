//! A reader that answers from a table.

use super::Augur;

/// A reader that answers from a table, for driving the augury without a model.
///
/// **The instrument that keeps this feature visible.** `ORBS_DUMP` builds no
/// app and presses no key, `scripts/dumps.sh` captures surfaces as text, and
/// `scripts/play.sh` drives the terminal build — none of which can hold a GPU,
/// a worker thread or two megabytes of weights. Without a reader they can
/// reach, every divined surface would be gated on one person typing one
/// sentence into one window, which is the blindness CLAUDE.md records the
/// bailey shipping under.
///
/// It is also the fixture the real reader is measured against: deterministic,
/// instant, and identical on every machine.
#[derive(Debug, Default, Clone)]
pub struct Fixture {
    /// Lines and the canonical command each stands for.
    readings: Vec<(String, String)>,
}

impl Fixture {
    /// A reader that knows nothing, and so abstains from everything.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Teach it that `line` means `echo`.
    #[must_use]
    pub fn reading(mut self, line: &str, echo: &str) -> Self {
        self.readings
            .push((line.trim().to_lowercase(), echo.to_owned()));
        self
    }

    /// The worked phrasings, for a dump or a scenario that wants a live augury.
    ///
    /// **A table in Rust, and deliberately a short one.** These are fixtures the
    /// way `Verb::canonical` is a table rather than writing, so rule 6 is not in
    /// play — but the corpus in `content/phrasings.toml` is where phrasings are
    /// *authored*, and [`Grammar`](super::Grammar) is what reads them. What
    /// stays here is the handful `dumps.sh` pins, chosen so a capture cannot
    /// move when a template is reworded.
    ///
    /// Each is a phrasing the deterministic pipeline genuinely cannot read, so
    /// a scenario using one is exercising the augury and not the matcher.
    #[must_use]
    pub fn worked() -> Self {
        Self::new()
            .reading("turn the sage into powder", "grind sage")
            .reading("smash the sage", "grind sage")
            .reading("i need powdered sage", "grind sage")
            .reading("go and have a look at the laboratory", "attend laboratory")
            .reading("what is in here", "survey")
            .reading("tell me about brewing", "recall brewing")
    }
}

impl Augur for Fixture {
    fn read(&self, line: &str) -> Vec<String> {
        let wanted = line.trim().to_lowercase();
        self.readings
            .iter()
            .filter(|(known, _)| *known == wanted)
            .map(|(_, echo)| echo.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fixture_answers_what_it_was_taught() {
        let augur = Fixture::new().reading("smash the sage", "grind sage");
        assert_eq!(augur.read("smash the sage"), vec!["grind sage".to_owned()]);
    }

    #[test]
    fn a_fixture_abstains_from_everything_else() {
        // Abstaining is the common answer and must never be an error. An empty
        // list rather than a refusal: nothing about it is exceptional.
        let augur = Fixture::worked();
        assert!(augur.read("xyzzy plugh").is_empty());
        assert!(Fixture::new().read("smash the sage").is_empty());
    }

    #[test]
    fn a_fixture_does_not_care_about_case_or_spacing() {
        let augur = Fixture::worked();
        assert_eq!(
            augur.read("  Smash The Sage  "),
            vec!["grind sage".to_owned()]
        );
    }

    #[test]
    fn a_fixture_may_be_taught_more_than_one_reading() {
        // The shape a reader that cannot see the world needs: offer both and
        // let the room decide. A table rarely needs it; a grammar always does.
        let augur = Fixture::new()
            .reading("run night_watch", "invoke night_watch")
            .reading("run night_watch", "wield night_watch");
        assert_eq!(augur.read("run night_watch").len(), 2);
    }

    #[test]
    fn every_worked_phrasing_is_one_the_matcher_cannot_read() {
        // **The fixture must exercise the augury, not the matcher.** A phrasing
        // the deterministic pipeline already reads outright would never reach a
        // reader, so a scenario built on it would pass whether or not the seam
        // worked at all — green, and measuring nothing.
        use crate::parser::{Mode, NounKind, Scene, Verb, analyse, is_literal};

        // Everything the worked phrasings name, with every fixture's verb in
        // scope — so a line failing to read outright is about the *phrasing*
        // and not about which room the test happens to stand in.
        let scene = Verb::ALL.into_iter().filter_map(Verb::anchor).fold(
            Scene::new()
                .with(NounKind::Place, "/tower/laboratory")
                .with(NounKind::Reagent, "sage")
                .with(NounKind::Topic, "brewing"),
            Scene::offering,
        );

        for (line, _) in Fixture::worked().readings {
            assert!(!is_literal(&line), "{line:?} never reaches an augur");
            assert!(
                !analyse(&line, &scene, Mode::Calm).reads_outright(),
                "{line:?} is read outright, so the fixture is testing the matcher"
            );
        }
    }
}
