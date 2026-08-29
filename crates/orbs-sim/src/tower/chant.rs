//! The menagerie's chant — a figure sung against the tick (§10, `menagerie/`).
//!
//! A chart of syllables travels toward the circle, one landing on each tick. You
//! answer each with `sing <syllable>` before it lands; miss too many and the
//! chant collapses and the pylon wears for it. Finish it and the circle yields
//! troops, as many as the chant was sung cleanly.
//!
//! # Why this is a clock when nothing else in the tower is
//!
//! §10.1 said *"timing means windows at 1 Hz — never a reflex"*, and that clause
//! is struck as of this phase (§19). Five domains are solved by choosing and this
//! one is solved by *doing*; the accommodation for anyone who does not want that
//! is a paused mode reaching the same ceiling, not an easier chant.
//!
//! **What is here is world-shaped and clockless all the same.** A syllable lands
//! on a *tick*, and this module knows nothing finer. `orbs-sim` gains no time
//! source, which is architectural rule 1 and rule 3 together — and a real-time
//! press, when Phase 5's surface lands, is graded against the tick it arrived on
//! rather than against a clock this module would have to keep.
//!
//! # The shape a spell has to solve
//!
//! The chart is visible ahead, so a spell is not guessing what comes — it is
//! working out *when*. One instruction a tick means reading and singing cannot
//! happen on the same one, and that is the whole puzzle. §19 records the draft
//! that bounded lookahead to a single tick and so dissolved the arithmetic it was
//! trying to protect.

use bevy_ecs::prelude::*;
use rand::Rng;

use crate::rng::{RngStream, Rngs};

/// How many syllables a chant runs to.
///
/// Twelve is a figure a person can hold: long enough that a lapse costs
/// something, short enough that a failed one is a minute rather than an evening.
/// It is also two more than [`TOLERANCE`] allows to be missed by a factor of
/// four, so a chant cannot be blundered through.
pub const LENGTH: usize = 12;

/// How many syllables may be missed before the chant collapses.
///
/// **Three, and it is a tolerance rather than a life count.** §11.5's *"never
/// ruinous, only slower"* wants a failed chant to cost time, and the way to make
/// that true is to let a good performance survive a slip. A player who misses
/// four has stopped performing the figure and is guessing.
pub const TOLERANCE: u32 = 3;

/// What a collapsed chant takes off the barrier.
///
/// **Bounded, and small against [`erosion::STANDING`](super::erosion::STANDING).**
/// Five points is 150 ticks of ordinary erosion — enough that failing matters and
/// far from ruinous, which is the §11.5 line this domain has to stay the right
/// side of. It is also the reason a chant costs *nothing* to attempt: the risk is
/// the price, so an attempt is never a resource decision.
pub const WEAR: u32 = 5;

/// How many ticks a syllable takes to reach the rule.
///
/// # The number that makes the domain solvable at all
///
/// It was **one**, and a spell could not strike a single syllable of a chant. A
/// question costs a tick and the figure advanced on every tick nothing was sung,
/// so a read was always followed by an advance and a read-then-sing missed by
/// exactly one, for ever. §19 records it; the shipped `probe_it` test spell
/// collapsed twelve figures out of twelve without one strike.
///
/// **Four, and it is deliberately less than the worst ladder.** A spell picking
/// a syllable out of four spends up to four ticks deciding and one singing, so
/// at one instruction a tick it *cannot* keep up — and that is the point. The
/// weave's `steps_1` and `steps_2` buy extra instructions a tick, so **the
/// menagerie is the domain that rewards concentration**: unautomatable early,
/// solvable once the orb can read and answer in the same second.
///
/// §19 records this reversing an earlier decision of mine that bounded the
/// budget's effect deliberately. The budget is the unlock here, not a leak.
///
/// It is also what makes the room playable by hand. One arrow a second is a
/// typing test; one every four is a figure a person can read off the board and
/// answer without hurrying.
pub const PACE: u32 = 4;

/// How many ticks wide the landing is.
///
/// **A window, not an instant.** A syllable may be answered on the tick it lands
/// or the one before it. Two things need this and neither is a concession:
///
/// - **A spell's decision costs a variable number of ticks** — one rung of a
///   ladder or four — so an instant target would mean a different `bide` per
///   branch and an arithmetic that depends on the runner's exact accounting.
///   With a window the same delay serves every branch.
/// - **A person cannot hit a single named second** reliably, and being asked to
///   is a reflex test rather than a reading one. §10.1's clause is struck for
///   this domain (§19) and that is not a licence to make it unfair.
///
/// Two rather than three: at [`PACE`] four, a three-tick window would leave only
/// one tick where singing is *early*, and a target you can hardly miss is not a
/// target.
pub const WINDOW: u32 = 2;

/// A window as wide as the approach would mean nothing was ever early, and
/// [`Strike::Early`] is the rule the whole timing puzzle rests on.
///
/// **A compile-time assertion rather than a test**, because it is a relationship
/// between two constants: a test asserting it is a test clippy correctly calls
/// constant, and this fails the build instead of a run.
const _: () = assert!(WINDOW < PACE);

/// The reading a spell asks the circle for: which syllable is at the aperture.
pub const NEXT: &str = "next";

/// The reading that counts syllables still to come.
pub const REMAINING: &str = "remaining";

/// The reading on the syllable *behind* the aperture — §8's one tick of
/// lookahead.
///
/// **This is what makes a satchel worth having in this room.** Without it a
/// producer has nothing to run ahead on: only the aperture publishes
/// [`NEXT`], so a spell reading the world would enqueue the same
/// syllable every lap while the consumer waited on the one it had already sung.
///
/// **`onward`, not `after`.** A sweep of both axes puts `after` at exactly
/// `MIN_SIMILARITY` against `enter`, which is an `attend` synonym — and a
/// reading is a `NounKind::Sense`, which `NounKind::Any` reaches, so it sits in
/// the way of every room's vocabulary rather than only this one's. `onward` is
/// clean on similarity and prefix, and it is the same `-ward` the four syllables
/// already carry.
pub const ONWARD: &str = "onward";

/// The words a spell may ask the menagerie for.
///
/// The sanctum's `readings()` is the shape: registered unconditionally in
/// `tower::scene`, whether or not a chant is running, because a spell compiles
/// at **cast** — when none of them is true of anything — and `bind::stand`
/// recasts every lap.
///
/// # There is deliberately no `until`
///
/// It was here, counting ticks to the landing, and it was the reading a solver
/// was written against — *"so the delay is read off the world rather than
/// hard-coded from [`PACE`]"*. That is exactly what made the domain trivial, and
/// removing `bide until` alone would not have fixed it: with the reading still
/// answerable, `repeat until the circle has 1 until` / `end` is the same cheat
/// spelled as a one-tick spin, and it lands the strike just as reliably.
///
/// **A spell must now count `PACE` itself**, which is the arithmetic the domain
/// exists for. The number has not left the model — [`Chant::until`] still
/// answers it, the board still draws the approach, and `orbs-balance`'s driver
/// still reads it, because a harness measuring the ceiling is not a player.
/// What left is the *language's* ability to ask.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    vec![NEXT, REMAINING, ONWARD]
}

/// One syllable of a chant.
///
/// # The names, and why they are not `up`/`down`/`left`/`right`
///
/// **`left` scores 750 against `let` and `right` 800 against `light`.** The first
/// is the spell language's own binding word, which would have put a syllable in
/// the way of every `let` a player typed; the second is a `kindle` synonym. Both
/// were found by sweeping rather than by playing, which is the lesson §19 draws
/// from `chant` itself scoring 600 three ways.
///
/// The `-ward` set is clean, reads as one family, and prefixes usefully: three
/// characters is `MIN_PREFIX`, so `sing sky` reaches [`Skyward`](Self::Skyward)
/// and nothing else. A spell writes the whole word because a spell is read more
/// than it is typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Syllable {
    /// Sung high. `▲`.
    Skyward,
    /// Sung low. `▼`.
    Earthward,
    /// Sung to the left hand. `◄`.
    Leftward,
    /// Sung to the right hand. `►`.
    Rightward,
}

impl Syllable {
    /// Every syllable, in the order the lanes are drawn.
    pub const ALL: [Self; 4] = [
        Self::Leftward,
        Self::Skyward,
        Self::Earthward,
        Self::Rightward,
    ];

    /// The word a player types and a spell writes.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Skyward => "skyward",
            Self::Earthward => "earthward",
            Self::Leftward => "leftward",
            Self::Rightward => "rightward",
        }
    }

    /// The glyph the board draws.
    ///
    /// **CP437, and checked by a test rather than by eye** — §19 records `▪` and
    /// `►` shipping as `?` because a painter's glyphs are Rust literals and
    /// `is_renderable` only lints authored prose. These four are the arrows the
    /// code page has.
    #[must_use]
    pub const fn glyph(self) -> char {
        match self {
            Self::Skyward => '▲',
            Self::Earthward => '▼',
            Self::Leftward => '◄',
            Self::Rightward => '►',
        }
    }

    /// Read a syllable back from its word.
    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|one| one.word() == word)
    }
}

/// What answering a syllable was worth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strike {
    /// The right syllable, on the tick it landed.
    Struck,
    /// The wrong syllable, or none before it landed.
    Missed,
    /// The right syllable, too soon.
    ///
    /// Counted against the singer exactly as a miss is — what separates it is
    /// only what the orb *says*, because a player who sang the right word early
    /// has made a different mistake from one who sang the wrong word and needs to
    /// be told which.
    Early,
    /// Nothing was sung and the syllable is still on its way.
    Travelling,
}

/// A chant in progress: the chart, and how far through it the singer is.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Chant {
    /// The figure, in the order it lands.
    chart: Vec<Syllable>,
    /// Which syllable is at the aperture. Equals `chart.len()` when done.
    at: usize,
    /// How each answered syllable went, oldest first: `true` struck.
    ///
    /// **A sequence, not two counters**, and the board is why. Two tallies can
    /// say *two struck, one missed* and cannot say **which** — so a board drawing
    /// pegs from counts would show a run that never happened. It is also what the
    /// save needs, for the same reason: a restored chant has to come back the one
    /// that was being sung.
    sung: Vec<bool>,
    /// How many ticks the syllable at the aperture has been travelling.
    ///
    /// Reaches [`PACE`] and the syllable lands. Held here rather than derived
    /// from a start tick because a chant can be saved mid-approach, and a tick
    /// stamp restored into a world whose clock has moved on would land the whole
    /// remaining figure at once.
    travelled: u32,
}

impl Chant {
    /// Draw a fresh figure.
    ///
    /// **Uniform over the four, independently** — no attempt to avoid runs. A
    /// figure that never repeats a syllable is one a player can partly predict,
    /// and predicting it is the thing this domain is not about; the same
    /// argument the ward makes for allowing repeats (§19).
    #[must_use]
    pub fn draw(rngs: &mut Rngs) -> Self {
        let rng = rngs.stream(RngStream::Menagerie);
        let chart = (0..LENGTH)
            .map(|_| Syllable::ALL[rng.random_range(0..Syllable::ALL.len())])
            .collect();
        Self {
            chart,
            at: 0,
            sung: Vec::new(),
            travelled: 0,
        }
    }

    /// Rebuild one from a save, without moving the stream.
    ///
    /// The ward's rule: a restore reads the figure rather than rolling it, or
    /// loading a tower would perturb every later draw in the session.
    #[must_use]
    pub const fn restored(
        chart: Vec<Syllable>,
        at: usize,
        sung: Vec<bool>,
        travelled: u32,
    ) -> Self {
        Self {
            chart,
            at,
            sung,
            travelled,
        }
    }

    /// How far the syllable at the aperture has travelled, out of [`PACE`].
    #[must_use]
    pub const fn travelled(&self) -> u32 {
        self.travelled
    }

    /// How many ticks until the syllable at the aperture lands.
    ///
    /// **What a spell is really asking**, and it is published as a reading so a
    /// solver can be written against it rather than against a constant it has to
    /// be told. Nought means it lands on this tick.
    #[must_use]
    pub const fn until(&self) -> u32 {
        PACE.saturating_sub(self.travelled).saturating_sub(1)
    }

    /// How each answered syllable went, oldest first.
    #[must_use]
    pub fn sung(&self) -> &[bool] {
        &self.sung
    }

    /// Write one out for a save.
    pub(crate) fn to_save(&self) -> crate::save::ChantSave {
        crate::save::ChantSave {
            chart: self.chart.iter().map(|one| one.word().to_owned()).collect(),
            at: self.at,
            sung: self.sung.clone(),
            travelled: self.travelled,
        }
    }

    /// Put one back.
    ///
    /// **Clamps rather than trusts**, on `Course::from_save`'s precedent: §15
    /// invites hand-editing, so every field here is one a person can get wrong.
    /// A word that names no syllable drops the whole figure rather than
    /// substituting one — a chant with a syllable nobody can sing is
    /// unfinishable, where no chant at all is a `summon` away from fine.
    pub(crate) fn from_save(save: &crate::save::ChantSave) -> Option<Self> {
        let chart: Option<Vec<Syllable>> = save
            .chart
            .iter()
            .map(|word| Syllable::from_word(word))
            .collect();
        let chart = chart?;
        if chart.is_empty() {
            return None;
        }
        // `at` past the end is a finished figure, which is not a thing to
        // restore; `travelled` past the pace would land the syllable the instant
        // the world resumed.
        let at = save.at.min(chart.len());
        let mut sung = save.sung.clone();
        sung.truncate(at);
        Some(Self {
            chart,
            at,
            sung,
            travelled: save.travelled.min(PACE.saturating_sub(1)),
        })
    }

    /// The figure, for the board and the save.
    #[must_use]
    pub fn chart(&self) -> &[Syllable] {
        &self.chart
    }

    /// How far through it the singer is.
    #[must_use]
    pub const fn at(&self) -> usize {
        self.at
    }

    /// How many landed cleanly, and how many did not.
    ///
    /// **Derived from [`sung`](Self::sung) rather than counted alongside it**, so
    /// the two can never disagree — the defect §19 records most often is one rule
    /// written twice.
    #[must_use]
    pub fn tally(&self) -> (u32, u32) {
        let struck = self.sung.iter().filter(|one| **one).count();
        let missed = self.sung.len() - struck;
        (
            u32::try_from(struck).unwrap_or(u32::MAX),
            u32::try_from(missed).unwrap_or(u32::MAX),
        )
    }

    /// The syllable at the aperture, if the chant is still running.
    #[must_use]
    pub fn next(&self) -> Option<Syllable> {
        self.chart.get(self.at).copied()
    }

    /// The syllable behind the aperture — one deep, and no more.
    ///
    /// # Why the chart is not simply visible
    ///
    /// §19 wanted the whole chart *"visible ahead, as any rhythm game's is"*,
    /// and struck a version that bounded a spell to a single tick because *"it
    /// dissolves the puzzle it was protecting"*. This is a **widening** of what
    /// exists rather than a return to that: a spell could see nothing at all
    /// beyond the aperture, and can now see one.
    ///
    /// One is what the pipeline needs and it is the smallest amount that makes
    /// the producer/consumer split *necessary* rather than decorative — the
    /// producer identifies this while the consumer sings [`next`](Self::next).
    /// Going deeper is the direction §19 already points and is a later decision,
    /// not a refusal.
    #[must_use]
    pub fn onward(&self) -> Option<Syllable> {
        self.chart.get(self.at.saturating_add(1)).copied()
    }

    /// How many syllables are still to come, this one included.
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.chart.len().saturating_sub(self.at)
    }

    /// Whether the figure has run out.
    #[must_use]
    pub const fn is_done(&self) -> bool {
        self.at >= self.chart.len()
    }

    /// Whether it collapsed rather than finished.
    ///
    /// **Asked at any point, not only at the end**, so the chant can be cut
    /// short the moment it is unrecoverable rather than making the singer finish
    /// a figure that is already lost.
    #[must_use]
    pub fn has_collapsed(&self) -> bool {
        self.tally().1 > TOLERANCE
    }

    /// Answer the syllable at the aperture, and move on.
    ///
    /// Returns `None` when there was nothing to answer.
    ///
    /// # Early is not a strike, and that is the whole mechanic
    ///
    /// A syllable is struck only on the tick it **lands** — [`until`](Self::until)
    /// nought. Singing the right word too soon is [`Early`](Strike::Early): it
    /// costs the syllable exactly as a wrong word does, and it is why a spell has
    /// to work out a delay rather than singing the moment it has identified the
    /// lane.
    ///
    /// Without this the domain has no puzzle in it at all — a solver would read
    /// the aperture and answer immediately, and `bide` would have nothing to
    /// count. It is also the half a *player* feels: the figure is a rhythm to
    /// keep, not a queue to empty.
    pub fn strike(&mut self, sung: Syllable, patient: bool) -> Option<Strike> {
        let wanted = self.next()?;
        // Inside the window, which is the last [`WINDOW`] ticks of the approach
        // — **unless the figure is being sung patiently**, where there is no
        // window because there is no clock. See [`Chant::travel`].
        if !patient && self.until() >= WINDOW {
            self.consume(false);
            return Some(Strike::Early);
        }
        let struck = sung == wanted;
        self.consume(struck);
        Some(if struck {
            Strike::Struck
        } else {
            Strike::Missed
        })
    }

    /// A tick passed with nothing sung.
    ///
    /// The syllable travels one row nearer; when it has come [`PACE`] rows it
    /// lands unanswered and is a miss. Returns what happened, or `None` when the
    /// chant was already over.
    ///
    /// # Patiently, which is §14's accommodation
    ///
    /// A figure sung `patient` does not travel at all: the syllable waits at the
    /// rule until it is answered, and [`strike`](Self::strike) drops the window
    /// with it. **The ceiling is unchanged** — a patient chant and a played one
    /// both yield what was sung correctly — so what the setting removes is the
    /// *dimension* speech and reflex cannot serve, and nothing else. §14 fixes
    /// that shape for the siege (*"ticks advance on player input"*) and this is
    /// the same shape, one domain earlier.
    ///
    /// **Never a difficulty setting.** A patient chant cannot score higher than
    /// a played one either; the two reach the same number by different roads,
    /// which is the only arrangement under which the accommodation is one.
    pub fn travel(&mut self, patient: bool) -> Option<Strike> {
        if self.is_done() {
            return None;
        }
        if patient {
            return Some(Strike::Travelling);
        }
        self.travelled += 1;
        // **`travelled >= PACE`, not `until() == 0`.** The tick where `until` is
        // nought is the beat itself — the one tick a syllable *can* be struck on
        // — so landing there would take it away before it could be answered.
        // That was the first version and it made every syllable unstrikeable.
        if self.travelled < PACE {
            return Some(Strike::Travelling);
        }
        self.consume(false);
        Some(Strike::Missed)
    }

    /// Take the syllable at the aperture off the figure.
    fn consume(&mut self, struck: bool) {
        self.sung.push(struck);
        self.at += 1;
        self.travelled = 0;
    }

    /// What a finished chant yields.
    ///
    /// **Quality is how much of the figure was sung, and nothing else.** A
    /// paused chant and a real-time one reach the same ceiling by being
    /// *correct*; what real time changes is how hard correct is. §19 records why
    /// that is the only shape under which the accommodation is an accommodation
    /// rather than a difficulty setting.
    ///
    /// A collapsed chant yields nothing at all.
    #[must_use]
    pub fn troops(&self) -> u32 {
        if self.has_collapsed() {
            return 0;
        }
        // One troop per three syllables sung cleanly: a whole figure is four,
        // and a chant scraping past `TOLERANCE` is three.
        self.tally().0 / 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn figure(of: &[Syllable]) -> Chant {
        Chant::restored(of.to_vec(), 0, Vec::new(), 0)
    }

    /// Let the syllable at the aperture travel all the way in, unanswered.
    ///
    /// The whole of what changed when [`PACE`] stopped being one: a tick is no
    /// longer a syllable, so a test that wants the *next* one has to say how
    /// many ticks that is rather than assuming.
    fn landed(chant: &mut Chant) -> Option<Strike> {
        let mut last = None;
        for _ in 0..PACE {
            last = chant.travel(false);
        }
        last
    }

    /// Carry the syllable at the aperture to its landing tick, then sing it.
    fn sing_on_the_beat(chant: &mut Chant, syllable: Syllable) -> Option<Strike> {
        for _ in 0..PACE - 1 {
            chant.travel(false);
        }
        chant.strike(syllable, false)
    }

    /// **The second half of the name was never asserted.** The body checked the
    /// length and nothing about the contents, so a `draw` that lost its
    /// randomness and returned twelve `Skyward`s would pass this *and*
    /// `the_same_seed_draws_the_same_figure` — leaving a fully predictable
    /// figure, which is exactly what [`Chant::draw`]'s own doc argues against.
    ///
    /// Across a handful of seeds rather than one, because a single twelve-draw
    /// can legitimately miss a lane: the draw is uniform and independent, not a
    /// shuffle.
    #[test]
    fn a_drawn_chant_is_the_stated_length_and_uses_every_syllable_somewhere() {
        let mut seen = Vec::new();
        for seed in 0..8 {
            let chant = Chant::draw(&mut Rngs::from_seed(seed));
            assert_eq!(chant.chart().len(), LENGTH);
            assert_eq!(chant.remaining(), LENGTH);
            assert!(!chant.is_done());
            for one in chant.chart() {
                if !seen.contains(one) {
                    seen.push(*one);
                }
            }
        }
        assert_eq!(
            seen.len(),
            Syllable::ALL.len(),
            "eight figures used only {seen:?} — the draw has lost a lane",
        );
    }

    /// The draw is a *seeded* one, so the same seed is the same figure.
    #[test]
    fn the_same_seed_draws_the_same_figure() {
        let one = Chant::draw(&mut Rngs::from_seed(11));
        let two = Chant::draw(&mut Rngs::from_seed(11));
        assert_eq!(one.chart(), two.chart());
        assert_ne!(Chant::draw(&mut Rngs::from_seed(12)).chart(), one.chart());
    }

    #[test]
    fn striking_the_aperture_advances_it_and_missing_does_too() {
        let mut chant = figure(&[Syllable::Skyward, Syllable::Leftward]);
        assert_eq!(chant.next(), Some(Syllable::Skyward));
        assert_eq!(
            sing_on_the_beat(&mut chant, Syllable::Skyward),
            Some(Strike::Struck),
        );
        assert_eq!(chant.next(), Some(Syllable::Leftward));
        assert_eq!(
            sing_on_the_beat(&mut chant, Syllable::Skyward),
            Some(Strike::Missed),
        );
        assert!(chant.is_done());
        assert_eq!(chant.tally(), (1, 1));
        assert_eq!(
            chant.strike(Syllable::Skyward, false),
            None,
            "a finished chant",
        );
    }

    /// A syllable travels for [`PACE`] ticks, and only the last of them lands it.
    ///
    /// **The defect this test is named after.** With a pace of one, a tick *was*
    /// a syllable — so a spell that spent a tick reading always sang into the
    /// next one and could never strike anything at all.
    #[test]
    fn a_syllable_travels_before_it_lands() {
        let mut chant = figure(&[Syllable::Skyward; 2]);
        assert_eq!(chant.until(), PACE - 1);
        for tick in 1..PACE {
            assert_eq!(chant.travel(false), Some(Strike::Travelling), "tick {tick}");
            assert_eq!(chant.remaining(), 2, "it landed early");
        }
        assert_eq!(chant.travel(false), Some(Strike::Missed), "it never landed");
        assert_eq!(chant.remaining(), 1);
    }

    /// **Early is not a strike**, and it costs the syllable just the same.
    #[test]
    fn the_right_syllable_sung_too_soon_is_not_a_strike() {
        let mut chant = figure(&[Syllable::Skyward; 2]);
        assert_eq!(chant.strike(Syllable::Skyward, false), Some(Strike::Early));
        // **It costs the syllable**, exactly as a wrong word does. Leaving it on
        // the board would make the figure solvable by singing the same word every
        // tick until it happened to land, which is no puzzle at all.
        assert_eq!(chant.tally(), (0, 1));
        assert_eq!(chant.remaining(), 1, "an early sing kept its syllable");

        // ...and on the beat it is a strike.
        assert_eq!(
            sing_on_the_beat(&mut chant, Syllable::Skyward),
            Some(Strike::Struck),
        );
    }

    /// A figure left alone runs itself out, one landing per [`PACE`] ticks.
    #[test]
    fn a_figure_nobody_sings_runs_itself_out() {
        let mut chant = figure(&[Syllable::Skyward; 2]);
        assert_eq!(landed(&mut chant), Some(Strike::Missed));
        assert_eq!(landed(&mut chant), Some(Strike::Missed));
        assert_eq!(chant.travel(false), None, "nothing left to miss");
        assert_eq!(chant.tally(), (0, 2));
    }

    #[test]
    fn collapse_is_one_past_the_tolerance_and_yields_nothing() {
        let mut chant = figure(&[Syllable::Skyward; LENGTH]);
        for _ in 0..TOLERANCE {
            landed(&mut chant);
        }
        assert!(!chant.has_collapsed(), "at the tolerance, still going");
        landed(&mut chant);
        assert!(chant.has_collapsed());
        assert_eq!(chant.troops(), 0, "a collapsed chant yields nothing");
    }

    /// The ceiling, and that it is reached by being *correct in time*.
    #[test]
    fn a_whole_figure_sung_cleanly_yields_the_most_troops() {
        let mut chant = figure(&[Syllable::Skyward; LENGTH]);
        for _ in 0..LENGTH {
            sing_on_the_beat(&mut chant, Syllable::Skyward);
        }
        assert!(chant.is_done());
        assert!(!chant.has_collapsed());
        assert_eq!(chant.troops(), 4);

        // ...and one that survives the tolerance yields less, not nothing.
        let mut scraped = figure(&[Syllable::Skyward; LENGTH]);
        for _ in 0..TOLERANCE {
            landed(&mut scraped);
        }
        while !scraped.is_done() {
            sing_on_the_beat(&mut scraped, Syllable::Skyward);
        }
        assert_eq!(scraped.troops(), 3);
    }

    /// **The pace is deliberately shorter than the worst ladder**, and this test
    /// asserted the opposite for one commit.
    ///
    /// Testing four lanes costs four ticks at one instruction a tick, and a
    /// syllable is [`PACE`] ticks from the rule — so a solver reaches the first
    /// lanes in time and not the last, and the figure collapses. **That is the
    /// design**: the weave's `steps_1` buys a second instruction a tick, and at
    /// two the same spell strikes twelve of twelve. The menagerie is the domain
    /// that rewards concentration (§19).
    ///
    /// My first version of this required `PACE > lanes` — the room comfortable
    /// at one step a tick — which would have made the concentration unlock buy
    /// nothing here at all.
    #[test]
    fn the_pace_is_shorter_than_a_four_lane_ladder() {
        let lanes = u32::try_from(Syllable::ALL.len()).expect("four");
        assert!(
            PACE <= lanes,
            "a solver would keep up at one step a tick, so concentration buys \
             nothing in this room",
        );
    }

    #[test]
    fn a_syllable_reads_back_from_its_own_word() {
        for one in Syllable::ALL {
            assert_eq!(Syllable::from_word(one.word()), Some(one), "{one:?}");
        }
        assert_eq!(Syllable::from_word("northward"), None);
    }

    /// §19's `▪` and `►` defects, twice over: a painter's glyphs are Rust
    /// literals, so `is_renderable` never sees them and only a test does.
    #[test]
    fn every_syllable_glyph_is_in_the_code_page() {
        for one in Syllable::ALL {
            assert!(
                orbs_render::is_renderable(one.glyph()),
                "{one:?} draws {:?}, which CP437 has no cell for",
                one.glyph(),
            );
        }
    }

    // **There is no collision test here on purpose.** A syllable is a
    // `Role::Reading` node, and `tests/naming.rs` already sweeps every reading
    // against every word the game knows — CLAUDE.md records that sweep failing
    // on its first run and turning up five collisions at once. A second,
    // hand-listed copy here would be the weaker of the two and would rot the
    // moment a word was added elsewhere.
    //
    // What that sweep caught for this domain, before any of it was built:
    // `left` scores **750** against the spell language's `let` and `right`
    // **800** against `light`. The `-ward` set is what came back clean.
}
