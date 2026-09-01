//! The words the bailey publishes for a spell to ask about (§5.1).
//!
//! **The arithmetic the language does not have.** The spell language has
//! at-least / at-most / exactly and place-against-place comparison, and no
//! ratios — so *"if the enemy count is twice that of defenders"* is published as
//! the word [`OUTNUMBERED`] rather than grown into a grammar. That is the maze's
//! pattern (`spoil`, `exit`, `back`), and this file is the vocabulary it needs.
//!
//! **Nearly every name here was chosen against a collision**, and the comments
//! say which — a reading is nameable from *every* room, so it sits in the way of
//! the whole vocabulary and a bad one breaks a word in another domain.

use super::{Intent, POOL};
use crate::tower::dice::Die;

/// The garrison is thin.
pub const FEW: &str = "few";
/// The garrison is wounded.
pub const HURT: &str = "hurt";
/// The enemy has at least twice the garrison's numbers.
pub const OUTNUMBERED: &str = "outnumbered";
/// The enemy is at full strength.
pub const MASSED: &str = "massed";
/// A side is broken.
pub const ROUTED: &str = "routed";
/// The best an area could come to once its dice are rolled.
///
/// **This is the count reading too, and a separate `pledged` was cut.** It said
/// how many dice were on an area, which `ceiling` already implies and `is empty`
/// already answers — and it collided twice over: `pledge` is a *prefix* of
/// `pledged`, so a live verb sat in front of a noun, which is §19's `troops`
/// against `troop` a second time. Vocabulary that answers a question two other
/// words already answer is worth removing rather than renaming.
pub const CEILING: &str = "ceiling";
/// The coming intent will throw away anything pledged here.
///
/// **`moot`, where the obvious word was `wasted`** — 667 against the potion
/// `haste`, which is *spendable in this room*, and 667 against this domain's own
/// `massed`. Two collisions in one word, one of them with something a player
/// types at the same wall.
pub const MOOT: &str = "moot";
/// The siege has ended, either way.
///
/// **`lifted`, where this was `over` — 600-plus against `order`**, which is
/// `deploy`'s own plain synonym one verb away, and against `overview`. A reading
/// is nameable everywhere, so `order troop` in the bailey could fuzz to it.
/// `lifted` is also the word a siege actually ends with.
///
/// **The reading a solver's `repeat until` needs**, and without it the shipped
/// spell looped for ever on a *loss*: `repeat until the enemy has routed` can
/// never come true when the garrison is the side that broke, so the guard stayed
/// false, `hold` was refused every lap, and a bound spell spun for the rest of
/// the session. `Outcome` is the question a loop should ask; `routed` is a
/// question about one band.
pub const LIFTED: &str = "lifted";
/// How many of a side are standing.
///
/// **`spears`, where this was `troops` — 834 against the material `troop`**, and
/// `troop` prefix-matches it outright. A reading is nameable from every room, so
/// `deploy troop` with an empty arsenal resolved to the *reading* and answered
/// *"troops is worth nothing on a wall"* instead of naming the empty arsenal.
/// That is §19's `purge grind` → `gained` defect verbatim.
pub const SPEARS: &str = "spears";
/// How much fight a side has left.
///
/// **`mettle`, where this was `vigour` — an exact 1000 collision with the secret
/// potion of the same name in `recipes.toml`.** Three naming sweeps ran before
/// this shipped and none of them included *materials* in the word list, which is
/// the hole: a reading collides with everything the player can name, not only
/// with verbs. `tests/naming.rs` sweeps them now.
pub const METTLE: &str = "mettle";
/// How many of them there are.
pub const FOES: &str = "foes";
/// How many rounds have resolved.
pub const TURNS: &str = "turns";
/// What is left to spend on dice — and, on a die, what that die costs.
///
/// **One word on two kinds of node, deliberately.** The coffer's is what you
/// hold and a die's is what it takes, so `if the coffer has fewer quintessence
/// than the d20` is a comparison of *the same reading in two places* — the shape
/// the language already had, rather than one it needed.
///
/// **`quintessence`, where §11.5 itself says `mana`.** `mana` scores 750 against
/// `many` — which lives inside `as many … as`, the comparison grammar this
/// reading is written for — and 750 against `man`, a live `recall` synonym. The
/// player's own first word, `power`, scores 800 against `tower`. A reading is
/// nameable from every room, so it sits in the way of the whole vocabulary.
///
/// The cost is one abbreviation: `qui` now reaches this rather than
/// `quickening-scroll`, by 887 to 876. `wield qui` is unaffected — `Workable`
/// rejects a `Sense` — so only `verify`/`purge`, which take `NounKind::Any`, can
/// see both.
pub const QUINTESSENCE: &str = "quintessence";
/// How likely a band's attacks are to tell, as a percentage.
///
/// **The odds a hand player already reads off the board.** `Roll::chance` was
/// called only from `Siege::view`, so the number was on screen and unaskable —
/// an asymmetry with §5.1's own *show the odds before the commitment*, since a
/// bound solver could not see what a person could. Publishing it is what makes
/// *"a 5% chance to be hit is not worth defending against"* a decision a spell
/// can take.
///
/// **`aim`, and it was nearly `peril`** — which fails the *prefix* sweep rather
/// than the similarity one: `per` reaches `peril` at 940 against `peruse`'s 925,
/// so the reading would have stolen a live verb's abbreviation. CLAUDE.md records
/// that exact case (`chorus`, because `per` reaches `peruse`) and closes with
/// **"sweep similarity *and* prefixes"**. `chance` fails too — 667 against
/// `cancel`, a `stop` synonym.
pub const AIM: &str = "aim";

/// Every reading the bailey publishes.
///
/// **Declared, never derived from `Role::Reading`.** `recall scripting` lists a
/// room's readings from `tower::readings_at`, and asking the marker instead
/// shipped a real defect one domain over — the lens taught the *archive's*
/// words. A set is declared by the fixture that owns it.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    let mut out = vec![
        FEW,
        HURT,
        OUTNUMBERED,
        MASSED,
        ROUTED,
        LIFTED,
        SPEARS,
        METTLE,
        FOES,
        TURNS,
        CEILING,
        MOOT,
        // **Appended, never inserted** — `scene.rs:345` states the rule for the
        // sixth time: §6 resolves a noun tie to whichever was registered first,
        // so a word slipped into the middle silently re-resolves a name an
        // existing solver already uses.
        QUINTESSENCE,
        AIM,
    ];
    out.extend(Intent::ALL.into_iter().map(Intent::word));
    // **The dice a wizard actually holds are readings**, because the coffer
    // publishes each free one by name — `if the coffer has d20` is how a solver
    // asks whether it still has the gamble in hand.
    //
    // **`POOL`, not `Die::ALL`.** Listing all seven made four words nameable
    // that nothing can ever publish, and they collide with each other: `d10`
    // against `d100` scores **962**. A die you do not have is not a reading.
    out.extend(POOL.into_iter().map(Die::word));
    out
}
