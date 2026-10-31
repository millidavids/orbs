//! Fixing the names in a spell's questions, held to a stricter bar than §6's
//! prompt: nobody is watching a spell resolve.

use crate::parser::{Condition, EXACT, NounKind, NounMatch, Scene, leaf};

use super::reading::Fault;

/// How alike a name must be to what it names, inside a spell.
///
/// 850 is `PREFIX_FLOOR`, where the scale already divides: any prefix of three
/// or more clears it, and a typo only as one wrong character in a long word.
/// At the prompt's 600, a question about absent salt compiled as one about sage.
pub const SPELL_SIMILARITY: u32 = 850;

/// How far ahead of the runner-up a name must be to be the one meant.
///
/// `ground` is an equal prefix of `ground-sage` and `ground-salt`, and
/// `best_match` would pick whichever registered first. The ceiling is `sag` →
/// `sage` at 67; `the_margin_is_below_the_closest_call_the_vocabulary_makes`
/// fails when a new name squeezes it.
pub const SPELL_MARGIN: u32 = 50;

/// Resolve one question's names in place, reporting every name it could not
/// place.
///
/// A missing *place* leaves the question unanswerable (§8's *Referent missing*),
/// named by the runner every cast. A missing *thing* is usually an answer of no
/// — `has ground-sage` is asked before there is any — so it is looked for in the
/// room, then in every name any recipe knows; in neither, it is a typo, and the
/// question cannot be asked. Every name, not the first (§8.1).
pub(super) fn fix(
    condition: &mut Condition,
    scene: &Scene,
    known: &[&str],
    bound: &[String],
) -> Vec<Unplaced> {
    let mut unplaced = Vec::new();
    condition.rename(&mut |kind, name| {
        // A bound name stands for a place known only when the line runs; its
        // value was resolved at the `set`.
        if bound.iter().any(|held| held.eq_ignore_ascii_case(name)) {
            return None;
        }
        let found = match kind {
            NounKind::Place => clearly(scene, NounKind::Place, name),
            // What `is` asks about: a place, or a spell by its exact name. The
            // exact place first, so `laboratory` never reads as
            // `laboratory.spell`; the spell before the fuzzy place, so
            // `sparing` is never fuzzed into a room. A spell keeps `.spell`, so
            // the run need not decide this again (`subject::find_is`).
            NounKind::Stoppable => exactly(scene, NounKind::Place, name)
                .or_else(|| {
                    exactly(
                        scene,
                        NounKind::Script,
                        &crate::content::with_extension(name),
                    )
                })
                .or_else(|| clearly(scene, NounKind::Place, name)),
            _ => clearly(scene, NounKind::Any, name).or_else(|| {
                known
                    .iter()
                    .find(|word| word.eq_ignore_ascii_case(name))
                    .map(|word| (*word).to_owned())
            }),
        };
        if found.is_none() {
            unplaced.push(match kind {
                NounKind::Place | NounKind::Stoppable => Unplaced::Place(name.to_owned()),
                _ => Unplaced::Thing(name.to_owned()),
            });
        }
        found
    });
    unplaced
}

/// The one fault a list of unplaced names adds up to.
///
/// A typo outranks a missing place: one leaves a question that cannot be asked,
/// the other one that cannot be answered. Every offender of that kind is named.
pub(super) fn fault_of(unplaced: &[Unplaced]) -> Option<Fault> {
    let typos: Vec<&str> = unplaced
        .iter()
        .filter_map(|entry| match entry {
            Unplaced::Thing(name) => Some(name.as_str()),
            Unplaced::Place(_) => None,
        })
        .collect();
    if !typos.is_empty() {
        return Some(Fault {
            key: "spell_unreadable_if",
            detail: Some(typos.join(", ")),
        });
    }
    let places: Vec<&str> = unplaced
        .iter()
        .filter_map(|entry| match entry {
            Unplaced::Place(name) => Some(name.as_str()),
            Unplaced::Thing(_) => None,
        })
        .collect();
    (!places.is_empty()).then(|| Fault {
        key: "spell_nowhere",
        detail: Some(places.join(", ")),
    })
}

/// A name in a question that the room could not place, and which kind it was.
pub(super) enum Unplaced {
    /// A place the tower does not have: the question stands, unanswerable.
    Place(String),
    /// A word that names nothing, in the room or the recipes: a typo.
    Thing(String),
}

/// What `name` clearly names in `scene`, or `None` if it is not clear.
///
/// Unclear is nothing near enough, a typo rather than an abbreviation, or two
/// things equally close.
pub(super) fn clearly(scene: &Scene, kind: NounKind, name: &str) -> Option<String> {
    let words: Vec<&str> = name.split_whitespace().collect();
    let found = scene.candidates(kind, &words);
    let best = found.first()?;
    if best.score < SPELL_SIMILARITY {
        return None;
    }
    let winner = leaf(&best.name);

    // Two entries for one word are not a tie: §6.1 registers a `Topic` beside
    // every reagent at the same score.
    let runner_up = found.iter().find(|next| leaf(&next.name) != winner);

    // An exact match is never a coin flip.
    if best.score < EXACT
        && runner_up.is_some_and(|next| best.score.saturating_sub(next.score) < SPELL_MARGIN)
    {
        return None;
    }
    Some(winner.to_owned())
}

/// The leaf a scene registers as exactly `name`, and nothing nearer.
fn exactly(scene: &Scene, kind: NounKind, name: &str) -> Option<String> {
    let best = candidates(scene, kind, name).into_iter().next()?;
    (best.score >= EXACT).then(|| leaf(&best.name).to_owned())
}

/// Every distinct name a scene could answer `name` with, best first — what a
/// tie is detected from, which `best_match` throws away.
#[must_use]
pub fn candidates(scene: &Scene, kind: NounKind, name: &str) -> Vec<NounMatch> {
    let words: Vec<&str> = name.split_whitespace().collect();
    scene.candidates(kind, &words)
}
