//! Tampering, and how a player finds it.
//!
//! DESIGN.md §8.1 is the whole specification, and its central rule is that
//! **tells are never colour-only** — colour is forbidden as a sole carrier of
//! meaning (§14) and is invisible to a screen reader. Every tampering therefore
//! has two channels:
//!
//! 1. **A structural visual signature** — *"spacing, glyph substitution,
//!    alignment drift, malformed record boundaries."* Perceptible at a glance to
//!    a player who looks.
//! 2. **A command-detectable signature.** `verify <target>` reports tampering on
//!    any one surface instantly and cheaply. This is §5.1's one-command
//!    diagnosis, it is fully accessible, and it makes the *visual* tell a speed
//!    bonus for observant players rather than a requirement.
//!
//! The second is what keeps the first honest. A tell that could only be seen
//! would be a puzzle a blind player cannot play; a tell that could only be
//! `verify`ed would make looking pointless.
//!
//! # Why the log is the surface
//!
//! §15 exercises log poisoning against *brewing* logs using `peruse`, `sift` and
//! `verify` — the log-reading primitives — rather than building the whole
//! scrying domain, so the "siege is empty" risk is covered in Phase 0 at two
//! domains rather than three.
//!
//! # How the tell is built
//!
//! Not by corrupting stored text. §3 is explicit: *"the renderer corrupts it;
//! the model records it faithfully"*, and a poisoned log that had been rewritten
//! could not be recovered, compared, or `verify`ed against anything.
//!
//! Instead a poisoned line is **re-emitted with a field dropped and the tampered
//! presentation set** — §8.1's "malformed record boundaries" literally, which the
//! record model already draws as a gap with no special case. `Presentation::
//! Tampered` is refused nowhere (§3's exemption covers only the *eldritch*
//! register), so the tell survives on exactly the diagnostic surface a player
//! inspects.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, Presentation, RecordKind, Records, Role};

use super::node::{Name, NodeId};
use crate::parser::Verb;
use rand::Rng as _;

use crate::rng::{RngStream, Rngs};
use crate::session::Scrollback;
use crate::tick::Tick;

/// A surface an enemy has interfered with.
///
/// §8.1 names four surfaces — script, schedule, log, entity. Phase 0 exercises
/// the log; the rest arrive with the remaining sabotage surfaces in Phase 1.
#[derive(Component, Debug, Clone, Copy)]
pub struct Poisoned;

/// Mark `target` as tampered with.
pub fn poison(world: &mut World, target: Entity) {
    world.entity_mut(target).insert(Poisoned);
}

/// Whether `target` has been tampered with.
#[must_use]
pub fn poisoned(world: &World, target: Entity) -> bool {
    world.get::<Poisoned>(target).is_some()
}

/// Report on a surface — §8.1's command-detectable signature.
///
/// Cheap and instant on one target. §8.1 makes `verify --all` the expensive form
/// *"scaling with the tower"*, so that auditing everything grows into a real
/// cost exactly as the tower gets complex and "which surface do I inspect first"
/// stays a decision; that arrives with the remaining surfaces in Phase 1.
pub fn verify(world: &mut World, target: Entity) {
    let name = world
        .get::<Name>(target)
        .map_or_else(String::new, |name| name.0.clone());

    // **A place answers for what is standing in it**, which is what makes the
    // *world* a surface rather than only the log. §8.1's world-state row is
    // *"reagents swapped, entity substituted"*, and a substitution is not on the
    // shelf — it is on the pile. Verifying the shelf and being told `sound`
    // while a swapped pile sat in it would be the surface reporting the
    // container instead of the contents.
    //
    // One level, deliberately: `verify` is §5.1's *one command, instantly and
    // cheaply*, and a recursive audit of the whole tree is `verify --all`, which
    // §8.1 prices as Production-class work.
    let substituted: Vec<String> = super::children_of(world, target)
        .into_iter()
        .filter(|node| poisoned(world, *node))
        .filter_map(|node| world.get::<Name>(node).map(|name| name.0.clone()))
        .collect();
    let tampered = poisoned(world, target) || !substituted.is_empty();

    // **Named, not counted.** §8.1's design rule is that sabotage must be
    // engaging to find and never frustrating — *"the skill is knowing which
    // surface to inspect, not deciphering an obscure clue"* — so once the player
    // has inspected the right surface, the answer is the thing itself.
    let message = if substituted.is_empty() {
        String::new()
    } else {
        world
            .resource::<crate::content::Prose>()
            .line("verify_substituted", &[("detail", &substituted.join(", "))])
    };

    let mut scrollback = world.resource_mut::<Scrollback>();
    let mut record = scrollback
        .records_mut()
        .push(RecordKind::Status)
        .text(FieldName::Name, Verb::Verify.canonical())
        .text(FieldName::Source, &name)
        .text(
            FieldName::State,
            if tampered { "tampered" } else { "sound" },
        )
        .role(if tampered {
            Role::Danger
        } else {
            Role::Success
        });
    if !message.is_empty() {
        record = record.text(FieldName::Message, &message);
    }
    record.finish();
}

/// Swap what a pile of stock says it is, without moving it.
///
/// §8.1's world-state tampering: *"reagents swapped, glyphs corrupted, entity
/// substituted"*, whose tell is *"substituted entities fail ID check →
/// `Referent missing`"*.
///
/// **The name changes and the identity does not.** That is the whole mechanism:
/// a spell that named the reagent resolved it at cast to a stable id, so the id
/// still points here and the *name* no longer matches — which is exactly the
/// missing-referent fault §8's failure taxonomy already reports, arriving for
/// the reason §8.1 says it should rather than through a typo.
///
/// It is the mirror of how a poisoned log works: nothing stored is destroyed,
/// so the tampering is recoverable, comparable and `verify`-able. A swap that
/// deleted the pile would be an enemy taking your sage, which is theft rather
/// than sabotage — §5.1 puts environmental damage in the calm layer and keeps
/// *misdirection* as the thing scrying exists to see through.
pub fn substitute(world: &mut World, node: Entity, as_named: &str) {
    let Some(mut name) = world.get_mut::<Name>(node) else {
        return;
    };
    // **The true name is kept, and without it this was not sabotage but
    // destruction.** `purge` on a poisoned surface removes `Poisoned` and reports
    // `cleansed`, which left the pile permanently misnamed and — being
    // un-poisoned again — eligible for a second swap to `charcoal--`. Three
    // endless reagents in the tower, so three rolls killed every heated recipe in
    // the game: no `kindle charcoal`, therefore no digestion, no distillation, no
    // clarity and none of the three secrets. Unwinnable, quietly, about an hour
    // in.
    //
    // The module header already claimed *"nothing stored is destroyed, so the
    // tampering is recoverable"*. It is true now.
    let was = std::mem::replace(&mut name.0, as_named.to_owned());
    let since = *world.resource::<Tick>();
    world.entity_mut(node).insert(Substituted { was, since });
    world.entity_mut(node).insert(Poisoned);
}

/// The name a pile is *made to claim* when the swap takes it.
///
/// It must be a real word or the tell would read as corruption rather than as
/// substitution. Its own name with a sigil struck through it is the cheapest
/// honest lie: `sage` sitting in the dispensary calling itself something a
/// recipe will not take.
///
/// **One function because the parser reads the shape too.** A lie built this way
/// is a strict *extension* of the truth, which makes it a resolver problem as
/// well as a content one — `parser::scene`'s abbreviation rule would otherwise
/// hand the pile straight back to the spell that named it. Spelling it twice is
/// how that rule would silently come apart again the next time this changes.
#[must_use]
pub fn claimed(was: &str) -> String {
    format!("{was}-")
}

/// What a substituted thing is really called, and when the lie landed.
///
/// The name is held beside the lie rather than instead of it, which is what makes
/// [`restore`] possible — and what makes notice → `verify` → `purge` a repair
/// loop rather than a report.
#[derive(Component, Debug, Clone)]
pub struct Substituted {
    /// The true name.
    pub was: String,
    /// The tick the swap landed, for [`settling`].
    pub since: Tick,
}

/// Give a substituted thing its name back.
///
/// Returns what it is called afterwards, so the sentence about the repair can
/// name the thing that was repaired rather than the lie it was carrying.
pub fn restore(world: &mut World, node: Entity) -> Option<String> {
    let was = world.get::<Substituted>(node)?.was.clone();
    if let Some(mut name) = world.get_mut::<Name>(node) {
        name.0 = was.clone();
    }
    world.entity_mut(node).remove::<Substituted>();
    Some(was)
}

/// How long a lie holds before the pile settles back to its own name.
///
/// # Why a swap wears off at all
///
/// **The repair loop is human-only by construction, and that is a fact about the
/// language rather than a gap in this module.** A spell names things with literals
/// (`parser/question.rs` has no variables at all), so a spell cannot say *"purge
/// whatever the dispensary is lying about"* — it would have to name `sage-`, a
/// word nobody knew at the time the spell was written. So `verify` → `purge` is
/// reachable only by a person who reads the lie off the screen and types it.
///
/// Without an expiry, that made an unattended tower terminal rather than
/// harassed: `orbs-balance` measured the standing grind loop falling from
/// 0.100/tick to **0.058 and staying there** — one swap and the loop is dead for
/// the rest of the session, silently, because `grind sage` finds no sage and the
/// refusal blames the mortar. §5.1 caps aberration arrival *"so repairs cannot
/// spiral"*; a permanent un-automatable swap does not spiral, it terminates, which
/// is worse and was never the intent.
///
/// # The number is swept, not chosen
///
/// A loop stalls for as long as the pile it names is lying, so its downtime is
/// `WEARS_OFF / SWAP_INTERVAL` — the one ratio that matters, and the reason these
/// two constants have to be set together. At the first pair (1200, never) an
/// unattended tower lost **42%** of its rate permanently; at (1200, 1800) it lost
/// 60% of every window. `orbs-balance` measured both.
///
/// **300 against `SWAP_INTERVAL`'s 3600 is 8% downtime**, which is a nuisance a
/// player notices and an hour away survives. Five minutes is also short enough
/// that waiting one out is never the *better* play — `purge` repairs it the moment
/// it is found, and that is what keeps the human loop worth running.
const WEARS_OFF: u64 = 300;

/// Let a lie that nobody caught settle back to the truth.
///
/// The passive half of the repair loop, and deliberately **silent**: a pile
/// quietly becoming itself again is the tower settling, not an event, and a line
/// saying so would be a notification for something the player never saw go wrong.
/// What is *not* silent is `verify` while it holds, which is the tell §8.1 asks
/// for.
pub fn settling(now: Res<Tick>, lies: Query<(Entity, &Substituted)>, mut commands: Commands) {
    let settled: Vec<Entity> = lies
        .iter()
        .filter(|(_, lie)| now.get().saturating_sub(lie.since.get()) >= WEARS_OFF)
        .map(|(node, _)| node)
        .collect();

    for node in settled {
        commands.queue(move |world: &mut World| {
            restore(world, node);
            // **`Poisoned` goes too, or the pile is sound and still ineligible.**
            // `substitution` filters on `Without<Poisoned>`, so a settled pile
            // that kept the marker would be truthful *and* permanently immune —
            // three settlings and the surface would have nothing left to touch.
            world.entity_mut(node).remove::<Poisoned>();
        });
    }
}

/// Roughly how many ticks pass between interferences.
///
/// §15's scenario is fifteen minutes, which at 1 Hz is 900 ticks, so this puts a
/// handful in a tester's session — enough that log-poisoning is *met* rather
/// than described, and rare enough that a clean log is still the normal case a
/// tampered one stands out against.
///
/// §5.3 caps aberration arrival so repairs cannot spiral; the full adversarial
/// model is Phase 8, and this is the single surface §15 asks Phase 0 to exercise.
const DRIFT_INTERVAL: u64 = 300;

/// Interfere with something, occasionally.
///
/// **Rolled from the seeded stream**, which is the first thing in the game to
/// roll at all: per-subsystem streams have existed since the determinism spine
/// and nothing had ever drawn from one, so replay could not be *observed* to
/// hold. Two runs from one seed now poison the same log on the same tick, and
/// the transcript proves it.
///
/// [`RngStream::Threat`] specifically — §19 gives each subsystem its own stream
/// so that adding a roll here cannot perturb the parser's.
pub fn drift(
    mut rngs: ResMut<Rngs>,
    logs: Query<(Entity, &Name, &NodeId), (With<Log>, Without<Poisoned>)>,
    mut commands: Commands,
) {
    // **Drawn before anything can return, and that is the whole shape of the
    // bug this had.** An integer draw rather than a ratio helper: the same
    // arithmetic on every platform and every `rand` release, which replay
    // depends on.
    //
    // The draw used to sit *after* the "is there a log left to poison" check, so
    // once every domain log was poisoned — roughly 1500 unattended ticks — this
    // system stopped drawing and every subsequent `substitution` roll shifted
    // one position along the shared `Threat` stream. `purge` un-poisons a log
    // and shifts it back. The reagent-swap schedule was therefore a function of
    // how many logs existed and when they filled, so **adding a seventh domain
    // would silently change the swaps of every saved session**.
    //
    // `substitution` hoists its own roll for exactly this reason and says so in
    // as many words; it was the newer of the two systems and only it got the
    // fix. Both draw once per tick, unconditionally, for ever.
    let roll: u64 = rngs.stream(RngStream::Threat).random();

    // **Sorted by name, not query order — the other half of the fix above, and
    // it took a save format to make it observable.** `logs.iter().next()` is
    // archetype order, which in a *lived* world is a function of which log was
    // poisoned when (inserting `Poisoned` moves an entity between tables and
    // `swap_remove`s its row) and in a *rebuilt* one is simply spawn order. So a
    // world reloaded from a save poisons a different log than the session that
    // wrote it, from the same seed on the same tick.
    //
    // Nothing had ever rebuilt a world before, so nothing could see it — which
    // is why the comment above says only `substitution` got this fix, and why
    // this is the item that finally pays it.
    let mut surfaces: Vec<(Entity, &Name, NodeId)> = logs
        .iter()
        .map(|(entity, name, id)| (entity, name, *id))
        .collect();
    // **`NodeId` breaks the tie, and without it the sort was not a total order.**
    // `sort_unstable` promises nothing for equal keys, so two same-named logs
    // would fall back to input order — which is the archetype order this sort
    // exists to remove. Four domains have four distinct log names today; five
    // more domains arrive by Phase 9a and nothing forbids two of them holding a
    // `feed.log`. The id is safe as a secondary key because a save carries it.
    surfaces.sort_unstable_by(|(_, a, x), (_, b, y)| a.0.cmp(&b.0).then(x.cmp(y)));

    if !roll.is_multiple_of(DRIFT_INTERVAL) {
        return;
    }

    // **Drawn from the pool, not `first()`**, for the reason `substitution`
    // gives in full: a sort is a determinism fix, and taking its head turns that
    // fix into content — `archive.log` first, every session, every seed. The
    // index comes out of the *same* `roll`, whose quotient is untouched entropy
    // once it has cleared `DRIFT_INTERVAL`, so this costs the shared `Threat`
    // stream no extra draw.
    let index = (roll / DRIFT_INTERVAL) as usize % surfaces.len().max(1);
    let Some((target, _, _)) = surfaces.get(index).copied() else {
        return;
    };
    commands.entity(target).insert(Poisoned);
}

/// Swap a reagent somewhere in the tower, occasionally — §8.1's world surface.
///
/// **A second system rather than a branch inside [`drift`]**, and the reason is
/// the stream. Both draw from [`RngStream::Threat`], and interleaving two rolls
/// in one system would make *which* surface is hit depend on how many draws had
/// happened before — so adding the world surface would silently change every
/// existing log-poisoning replay. Two systems each drawing once per tick is one
/// more draw per tick and no reordering of what either one sees.
///
/// **Rarer than log drift, deliberately.** A poisoned log misdirects a
/// diagnosis; a swapped reagent stops a bound spell, which is a bigger
/// interruption — §5.1 caps aberration arrival so repairs cannot spiral, and
/// this is the more expensive of the two to repair.
pub fn substitution(
    mut rngs: ResMut<Rngs>,
    fuels: Res<crate::content::Fuels>,
    stock: Query<(Entity, &Name, &NodeId, &super::Stock), Without<Poisoned>>,
    mut commands: Commands,
) {
    // **Endless base stock only, and this is the load-bearing restriction.**
    //
    // A first pass took any pile at all, and swapped `ground-sage` sitting
    // between a grind and a digestion — which does not misdirect a player, it
    // destroys work in flight. §5.1 keeps environmental damage in the calm layer
    // and leaves *misdirection* as the thing to see through, and §11.5's rule is
    // that a cost is the resource and never progress.
    //
    // A base reagent is the honest target for the same reason it is endless: the
    // tower always has more, so what a swap costs is the **spell that named it**
    // and nothing that was half-made. It is also what a spell names most, which
    // is what makes the sabotage worth finding.
    //
    // It broke `meditating_stalls_at_a_stage_boundary` on the way in, which is a
    // test about the pipeline and not about sabotage — a nuisance that can reach
    // into a running brew is one that shows up as noise everywhere.
    // **The roll first, unconditionally, and the order is the whole reason this
    // is a separate system.** The doc above promises *one more draw per tick and
    // no reordering of what either one sees* — and drawing *after* an early
    // return breaks it: once every endless pile is poisoned this system stops
    // drawing, and every subsequent `drift` roll shifts one position along the
    // shared stream. Adding or removing an endless reagent in `build.rs` would
    // then silently change the log-poisoning schedule, which is exactly the
    // hazard the split was built to avoid.
    let roll: u64 = rngs.stream(RngStream::Threat).random();

    let mut piles: Vec<(Entity, &Name, NodeId)> = stock
        .iter()
        .filter(|(_, _, _, stock)| matches!(stock, super::Stock::Endless))
        // **Fuel is exempt, and the reason is the paragraph above.** A swap is
        // honest because it costs *the spell that named the reagent* and nothing
        // half-made — and charcoal is named by no recipe at all. It is the
        // tower's power supply, so swapping it stops every heated stage in every
        // domain at once, which is the same "reaches into work in flight"
        // objection that already exempts the non-endless piles.
        //
        // `orbs-balance` is what found this, on the first sweep after the surface
        // shipped: `charcoal` sorts before `rock-salt` and `sage`, so the *first*
        // swap of every session took the fire and clarity's rate fell from 0.140
        // to 0.074 — half the laboratory, silently, on every seed.
        .filter(|(_, name, _, _)| fuels.get(&name.0).is_none())
        .map(|(node, name, id, _)| (node, name, *id))
        .collect();
    // **Sorted by name, not query order.** `tower::node` records archetype order
    // as a defect that changes what a phrase resolves to with no test catching
    // it, and a replay has to swap the *same* pile from the same seed.
    // Total, for the reason `drift` gives above: `sort_unstable` leaves equal
    // keys in input order, and input order here is the archetype order the sort
    // exists to remove.
    piles.sort_unstable_by(|(_, a, x), (_, b, y)| a.0.cmp(&b.0).then(x.cmp(y)));

    if !roll.is_multiple_of(SWAP_INTERVAL) {
        return;
    }

    // **Drawn from the pool, not `first()`.** The sort above is for replay and is
    // right; taking its head made *which* pile is hit as fixed as the sort —
    // every session, every seed, in alphabetical order for ever. That is a
    // determinism fix that quietly became content.
    //
    // The index comes out of the *same* `roll` rather than a second draw, because
    // a draw that only happens when the swap fires would move `drift`'s stream
    // position by a variable amount — the hazard this system was split out to
    // avoid. `roll` cleared `SWAP_INTERVAL`, so its quotient is untouched entropy.
    let index = (roll / SWAP_INTERVAL) as usize % piles.len().max(1);
    let Some((target, name, _)) = piles.get(index).copied() else {
        return;
    };

    let claimed = claimed(&name.0);
    commands.queue(move |world: &mut World| {
        substitute(world, target, &claimed);
    });
}

/// Roughly how many ticks pass between reagent swaps.
///
/// **Twelve times rarer than [`DRIFT_INTERVAL`], and it was four.** A poisoned log
/// misleads a reading of one; a swapped reagent stops every loop that named it, so
/// the two are not the same order of interruption and pricing them one step apart
/// said they were. One an hour, paired with [`WEARS_OFF`]'s five minutes — see
/// there for the ratio, which is the number that was actually swept.
const SWAP_INTERVAL: u64 = 3600;

/// A file that accumulates what a domain did.
#[derive(Component, Debug, Clone, Copy)]
pub struct Log;

/// Which line of a poisoned log carries the visible tell.
///
/// Every third, so a reader has something to compare against — a log where
/// *everything* looked wrong would read as a rendering fault rather than as
/// interference, and §8.1 wants the player to spot the odd one out.
const TELL_EVERY: usize = 3;

/// Copy `lines` into `into` as a listing, damaging some if the source is
/// poisoned.
///
/// The damage is structural, never textual: a dropped field and the tampered
/// face. §8.1's "malformed record boundaries" is exactly a record missing a
/// field, which the table view already draws as a gap.
///
/// # `kind` is which of §3's diagnostic surfaces this is
///
/// [`RecordKind::LogLine`] for a view over the record stream,
/// [`RecordKind::ScriptLine`] for a file the player wrote. Both are exempt from
/// the eldritch treatment; what differs is how §14 says them. A log line is
/// fielded and speaks as `label: value`; a **script** line is the player's own
/// sentence and speaks as itself, which is why the kind had to reach this far
/// rather than being decided at the painter.
///
/// It had no second producer for four phases, and the cost was audible rather
/// than visible: every line of every spell announced itself as a `row`.
pub fn emit_lines(
    into: &mut Records,
    verb: Verb,
    kind: RecordKind,
    lines: &[String],
    tampered: bool,
) {
    into.push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .count(
            FieldName::Quantity,
            u64::try_from(lines.len()).unwrap_or(u64::MAX),
        )
        .finish();

    for (index, line) in lines.iter().enumerate() {
        let damaged = tampered && index % TELL_EVERY == 0;
        let row = into.push(kind);
        // The source field is what a well-formed line carries. Dropping it is
        // the malformed boundary — and it is dropped rather than blanked so the
        // record genuinely lacks it.
        // A well-formed line is numbered. The damaged one loses its number —
        // §8.1's "malformed record boundaries" and "alignment drift" in one, and
        // the oldest tell in any log: the sequence skips.
        let row = if damaged {
            row.presentation(Presentation::Tampered)
                .spoken(line)
                .text(FieldName::Message, line)
        } else {
            // **`Line`, and this was `Tick` for both kinds.** The number is
            // `index + 1` — a position in this listing — and never the tick the
            // line happened at, which is not carried here at all. A log read
            // back at world tick 5 numbered its three lines 1, 2, 3 and called
            // each one a tick, so the only field on the record was a fabricated
            // time. It is silent on screen, where no label is drawn, and a
            // reader heard every one of them.
            row.count(
                FieldName::Line,
                u64::try_from(index + 1).unwrap_or(u64::MAX),
            )
            .text(FieldName::Message, line)
        };
        row.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(tampered: bool) -> Records {
        let mut records = Records::new();
        let lines: Vec<String> = (0..6).map(|n| format!("line {n}")).collect();
        emit_lines(
            &mut records,
            Verb::Peruse,
            RecordKind::LogLine,
            &lines,
            tampered,
        );
        records
    }

    #[test]
    fn a_sound_log_reads_clean() {
        let records = read(false);
        assert!(
            records
                .iter()
                .filter(|record| record.kind() == RecordKind::LogLine)
                .all(|record| record.presentation() == Presentation::Plain),
            "an untouched log showed a tell",
        );
    }

    #[test]
    fn a_poisoned_log_shows_a_structural_tell_and_keeps_its_text() {
        // §8.1: the signature is structural — spacing, glyph substitution,
        // alignment drift, malformed record boundaries. §3: the model records
        // faithfully and only the rendering is damaged, so the words survive
        // and `sift` can still find them.
        let records = read(true);
        let lines: Vec<_> = records
            .iter()
            .filter(|record| record.kind() == RecordKind::LogLine)
            .collect();

        let damaged: Vec<_> = lines
            .iter()
            .filter(|record| record.presentation() == Presentation::Tampered)
            .collect();
        assert!(!damaged.is_empty(), "a poisoned log showed nothing");
        assert!(
            damaged.len() < lines.len(),
            "everything looked wrong, so nothing stands out",
        );

        for record in &damaged {
            assert!(
                record.field(FieldName::Line).is_none(),
                "the boundary was not malformed",
            );
        }
        // The words are intact — §3's model-records-faithfully.
        assert!(
            records
                .sift(&orbs_render::Sift::new("line 0"))
                .next()
                .is_some(),
            "poisoning destroyed the text",
        );
    }

    #[test]
    fn the_tampered_face_survives_on_a_log_line() {
        // §3's corruption exemption covers the *eldritch* register only. A
        // sabotage tell on a diagnostic surface is the whole point — suppressing
        // it there would delete the signal on the surface a player inspects.
        let records = read(true);
        assert!(
            records
                .iter()
                .any(|record| record.presentation() == Presentation::Tampered),
            "the exemption swallowed the tell",
        );
    }
}
