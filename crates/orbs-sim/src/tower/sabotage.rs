//! Tampering, and how a player finds it.
//!
//! DESIGN.md §8.1 is the specification, and its central rule is that tells are
//! never colour-only (§14). So every tampering has two channels: a structural
//! visual signature — *"spacing, glyph substitution, alignment drift, malformed
//! record boundaries"* — and `verify <target>`, §5.1's one-command diagnosis. A
//! seen-only tell is a puzzle a blind player cannot play; a `verify`-only one
//! makes looking pointless.
//!
//! The log is the surface because §15 exercises log poisoning against *brewing*
//! logs with `peruse`, `sift` and `verify`.
//!
//! The tell is not corrupted stored text. §3: *"the renderer corrupts it; the
//! model records it faithfully"* — a rewritten log could not be recovered,
//! compared or `verify`ed. A poisoned line is re-emitted with a field dropped
//! and the tampered presentation set, which the record model already draws as a
//! gap. `Presentation::Tampered` is refused nowhere: §3's exemption covers only
//! the *eldritch* register.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, Presentation, RecordKind, Records, Role};

use super::charm;
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
/// Cheap and instant on one target. §8.1 prices `verify --all` as the expensive
/// form *"scaling with the tower"*, so "which surface do I inspect first" stays
/// a decision.
pub fn verify(world: &mut World, target: Entity) {
    let name = world
        .get::<Name>(target)
        .map_or_else(String::new, |name| name.0.clone());

    // A place answers for what is standing in it, which is what makes the
    // *world* a surface rather than only the log: §8.1's world-state row is
    // *"reagents swapped, entity substituted"*, and a substitution is on the
    // pile, not the shelf.
    //
    // One level, deliberately: `verify` is §5.1's *one command, instantly and
    // cheaply*, and the recursive audit is `verify --all`.
    let substituted: Vec<String> = super::children_of(world, target)
        .into_iter()
        .filter(|node| poisoned(world, *node))
        .filter_map(|node| world.get::<Name>(node).map(|name| name.0.clone()))
        .collect();
    let tampered = poisoned(world, target) || !substituted.is_empty();

    // Named, not counted. §8.1: *"the skill is knowing which surface to
    // inspect, not deciphering an obscure clue"*.
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
/// The name changes and the identity does not: a spell resolved the reagent to
/// a stable id at cast, so the id still points here and the *name* no longer
/// matches — §8's missing-referent fault, arriving for the reason §8.1 says it
/// should rather than through a typo.
///
/// Like a poisoned log, nothing stored is destroyed, so the tampering is
/// recoverable and `verify`-able. Deleting the pile would be theft rather than
/// sabotage: §5.1 keeps *misdirection* as the thing scrying sees through.
pub fn substitute(world: &mut World, node: Entity, as_named: &str) {
    let Some(mut name) = world.get_mut::<Name>(node) else {
        return;
    };
    // The true name is kept, or this is destruction rather than sabotage.
    // `purge` removes `Poisoned` and reports `cleansed`, which left the pile
    // permanently misnamed and eligible for a second swap to `charcoal--`.
    // Three endless reagents, so three rolls killed every heated recipe in the
    // game: unwinnable, quietly, about an hour in.
    let was = std::mem::replace(&mut name.0, as_named.to_owned());
    let since = *world.resource::<Tick>();
    world.entity_mut(node).insert(Substituted { was, since });
    world.entity_mut(node).insert(Poisoned);
}

/// The name a pile is *made to claim* when the swap takes it.
///
/// A real word, or the tell reads as corruption rather than substitution: its
/// own name with a sigil struck through it is the cheapest honest lie.
///
/// One function because the parser reads the shape too — the lie is a strict
/// *extension* of the truth, so `parser::scene`'s abbreviation rule would
/// otherwise hand the pile straight back to the spell that named it.
#[must_use]
pub fn claimed(was: &str) -> String {
    format!("{was}-")
}

/// What a substituted thing is really called, and when the lie landed.
///
/// The true name is held beside the lie rather than instead of it, which makes
/// [`restore`] possible — and notice → `verify` → `purge` a repair loop rather
/// than a report.
#[derive(Component, Debug, Clone)]
pub struct Substituted {
    /// The true name.
    pub was: String,
    /// The tick the swap landed, for [`settling`].
    pub since: Tick,
}

/// Give a substituted thing its name back.
///
/// Returns what it is called afterwards, so the repair sentence can name the
/// thing rather than the lie it was carrying.
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
/// The repair loop is human-only by construction: a spell names things with
/// literals, so it cannot say *"purge whatever the dispensary is lying about"*
/// — it would have to name `sage-`, a word nobody knew when the spell was
/// written. Only a person who reads the lie off the screen can repair it.
///
/// Without an expiry an unattended tower was terminal rather than harassed:
/// `orbs-balance` measured the standing grind loop falling from 0.100/tick to
/// 0.058 and staying there. §5.1 caps aberration arrival *"so repairs cannot
/// spiral"*.
///
/// The number is swept, not chosen. Downtime is `WEARS_OFF / SWAP_INTERVAL`, so
/// the two constants are set together: (1200, never) lost an unattended tower
/// 42% of its rate permanently, (1200, 1800) lost 60% of every window. 300
/// against 3600 is 8% — short enough that waiting one out is never better than
/// `purge`.
const WEARS_OFF: u64 = 300;

/// Let a lie that nobody caught settle back to the truth.
///
/// The passive half of the repair loop, and deliberately silent: a line saying
/// so would announce something the player never saw go wrong. `verify` while
/// the lie holds is the tell §8.1 asks for.
pub fn settling(now: Res<Tick>, lies: Query<(Entity, &Substituted)>, mut commands: Commands) {
    let settled: Vec<Entity> = lies
        .iter()
        .filter(|(_, lie)| now.get().saturating_sub(lie.since.get()) >= WEARS_OFF)
        .map(|(node, _)| node)
        .collect();

    for node in settled {
        commands.queue(move |world: &mut World| {
            restore(world, node);
            // `Poisoned` goes too, or the pile is sound and still ineligible:
            // `substitution` filters on `Without<Poisoned>`, so a settled pile
            // keeping the marker would be truthful *and* permanently immune.
            world.entity_mut(node).remove::<Poisoned>();
        });
    }
}

/// Roughly how many ticks pass between interferences.
///
/// §15's scenario is fifteen minutes — 900 ticks at 1 Hz — so this puts a
/// handful in a tester's session: log-poisoning is *met* rather than described,
/// and a clean log is still the normal case a tampered one stands out against.
///
/// §5.3 caps aberration arrival so repairs cannot spiral.
const DRIFT_INTERVAL: u64 = 300;

/// Interfere with something, occasionally.
///
/// Rolled from the seeded stream — the first thing in the game to roll at all.
/// [`RngStream::Threat`] specifically: §19 gives each subsystem its own stream
/// so a roll here cannot perturb the parser's.
pub fn drift(
    mut rngs: ResMut<Rngs>,
    // Never a log in a room the player cannot enter: a strike there latches a
    // rail mark on a box drawn dark, a fault nobody can walk in and find.
    // `tower::Sealed` marks the shut rooms for exactly this query.
    logs: Query<(Entity, &Name, &NodeId), (With<Log>, Without<Poisoned>, Without<super::Sealed>)>,
    // Not a query filter, and it cannot be. A charm has no expiry system — it
    // is an interval, so a lapsed one is still a present component — and
    // `Without<Charmed>` would shield a log for ever after its first charm.
    charmed: Query<&charm::Charmed>,
    now: Res<crate::tick::Tick>,
    taken: Res<super::Taken>,
    mut commands: Commands,
) {
    // Drawn before anything can return. An integer draw rather than a ratio
    // helper: the same arithmetic on every platform and every `rand` release,
    // which replay depends on.
    //
    // The draw used to sit *after* the "is there a log left to poison" check,
    // so once every domain log was poisoned this system stopped drawing and
    // every subsequent `substitution` roll shifted one position along the
    // shared `Threat` stream — the swap schedule was a function of how many
    // logs existed. Both systems now draw once per tick, unconditionally.
    let roll: u64 = rngs.stream(RngStream::Threat).random();

    // Sorted by name, not query order — the other half of the fix above.
    // `logs.iter().next()` is archetype order, which in a *lived* world depends
    // on which log was poisoned when and in a *rebuilt* one is spawn order, so
    // a world reloaded from a save poisoned a different log than the session
    // that wrote it, from the same seed on the same tick.
    let mut surfaces: Vec<(Entity, &Name, NodeId)> = logs
        .iter()
        .map(|(entity, name, id)| (entity, name, *id))
        .collect();
    // `NodeId` breaks the tie, or the sort is not a total order:
    // `sort_unstable` promises nothing for equal keys, so two same-named logs
    // fall back to the archetype order this sort exists to remove. The id is
    // safe as a secondary key because a save carries it.
    surfaces.sort_unstable_by(|(_, a, x), (_, b, y)| a.0.cmp(&b.0).then(x.cmp(y)));

    // The Ley Line's `vigilance` widens the interval, so the calm layer strikes
    // less often by exactly the tiers taken. The draw is unchanged, so a tower
    // with no vigilance keeps every replay it ever had.
    let interval = vigilant_interval(DRIFT_INTERVAL, super::grant::vigilance_percent(&taken));
    if !roll.is_multiple_of(interval) {
        return;
    }

    // Drawn from the pool, not `first()`: taking the sort's head turns a
    // determinism fix into content — `archive.log` first, every session, every
    // seed. The index reuses `roll`, whose quotient is untouched entropy once it
    // has cleared the interval, so the shared stream costs no extra draw.
    let index = usize::try_from(roll / interval).unwrap_or(usize::MAX) % surfaces.len().max(1);
    let Some((target, _, _)) = surfaces.get(index).copied() else {
        return;
    };
    // A `shielded` log is struck and holds, rather than never being picked.
    // Filtering charmed nodes *out of the pool* would change which log the same
    // roll hits — every seed's world, moved, by a thing the player did. The
    // charm holds, it does not hide.
    if charmed
        .get(target)
        .is_ok_and(|held| held.left(charm::Kind::Shielded, *now) > 0)
    {
        return;
    }
    commands.entity(target).insert(Poisoned);
}

/// How many ticks apart the calm layer strikes, under `less` percent of
/// vigilance.
///
/// `300` at nought; `400` at a quarter less, because a quarter fewer strikes
/// is an interval a third longer. Integer, exact, and never nought.
#[must_use]
pub const fn vigilant_interval(base: u64, less: u64) -> u64 {
    let less = if less > 99 { 99 } else { less };
    let widened = base * 100 / (100 - less);
    if widened == 0 { 1 } else { widened }
}

/// Swap a reagent somewhere in the tower, occasionally — §8.1's world surface.
///
/// A second system rather than a branch inside [`drift`], because of the
/// stream: both draw from [`RngStream::Threat`], and interleaving two rolls in
/// one system would make *which* surface is hit depend on how many draws had
/// happened before. Two systems drawing once per tick each reorder nothing.
///
/// Rarer than log drift: a poisoned log misdirects a diagnosis, a swapped
/// reagent stops a bound spell, and this is the more expensive to repair.
pub fn substitution(
    mut rngs: ResMut<Rngs>,
    fuels: Res<crate::content::Fuels>,
    stock: Query<
        (Entity, &Name, &NodeId, &super::Stock),
        (Without<Poisoned>, Without<super::Sealed>),
    >,
    mut commands: Commands,
) {
    // Endless base stock only. A first pass took any pile at all and swapped
    // `ground-sage` sitting between a grind and a digestion, which destroys work
    // in flight rather than misdirecting — §5.1 keeps environmental damage in
    // the calm layer, and §11.5's rule is that a cost is the resource and never
    // progress.
    //
    // A base reagent is the honest target for the same reason it is endless:
    // the tower always has more, so a swap costs the *spell that named it* and
    // nothing half-made. It is also what a spell names most.
    //
    // The roll comes first, unconditionally, and the order is the whole reason
    // this is a separate system: drawing *after* an early return means that
    // once every endless pile is poisoned this system stops drawing and every
    // subsequent `drift` roll shifts one position along the shared stream.
    let roll: u64 = rngs.stream(RngStream::Threat).random();

    let mut piles: Vec<(Entity, &Name, NodeId)> = stock
        .iter()
        .filter(|(_, _, _, stock)| matches!(stock, super::Stock::Endless))
        // Fuel is exempt, for the paragraph above: charcoal is named by no
        // recipe at all, and it is the tower's power supply, so swapping it
        // stops every heated stage in every domain at once.
        //
        // `orbs-balance` found this on the first sweep after the surface
        // shipped: `charcoal` sorts before `rock-salt` and `sage`, so the first
        // swap of every session took the fire and clarity's rate fell from
        // 0.140 to 0.074 — half the laboratory, on every seed.
        .filter(|(_, name, _, _)| fuels.get(&name.0).is_none())
        .map(|(node, name, id, _)| (node, name, *id))
        .collect();
    // Sorted by name, not query order: a replay has to swap the *same* pile from
    // the same seed. Total, for the reason `drift` gives above — `sort_unstable`
    // leaves equal keys in the archetype order the sort exists to remove.
    piles.sort_unstable_by(|(_, a, x), (_, b, y)| a.0.cmp(&b.0).then(x.cmp(y)));

    if !roll.is_multiple_of(SWAP_INTERVAL) {
        return;
    }

    // Drawn from the pool, not `first()`: taking the sort's head made *which*
    // pile is hit alphabetical for ever, on every seed. The index reuses `roll`
    // rather than a second draw, because a draw that only happens when the swap
    // fires would move `drift`'s stream position by a variable amount.
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
/// Twelve times rarer than [`DRIFT_INTERVAL`], and it was four: a poisoned log
/// misleads one reading, a swapped reagent stops every loop that named it, so
/// pricing them a step apart said they were the same order of interruption. One
/// an hour, paired with [`WEARS_OFF`] — see there for the swept ratio.
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
/// `kind` says which of §3's diagnostic surfaces this is —
/// [`RecordKind::LogLine`] for a view over the record stream,
/// [`RecordKind::ScriptLine`] for a file the player wrote. What differs is how
/// §14 says them: a log line is fielded and speaks as `label: value`, a script
/// line speaks as itself, which is why the kind reaches this far rather than
/// being decided at the painter. With one producer for four phases the cost was
/// audible rather than visible: every line of every spell announced itself as a
/// `row`.
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
        // The source field is dropped rather than blanked, so the record
        // genuinely lacks it. The damaged line loses its number too — §8.1's
        // "malformed record boundaries", and the oldest tell in any log: the
        // sequence skips.
        let row = if damaged {
            row.presentation(Presentation::Tampered)
                .spoken(line)
                .text(FieldName::Message, line)
        } else {
            // `Line`, and this was `Tick` for both kinds. The number is a
            // position in this listing, never the tick the line happened at, so
            // a log read back at world tick 5 numbered its lines 1, 2, 3 and
            // called each one a tick. Silent on screen; a reader heard every one.
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
        // §8.1: the signature is structural. §3: the model records faithfully
        // and only the rendering is damaged, so the words survive and `sift`
        // can still find them.
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
        // §3's corruption exemption covers the *eldritch* register only:
        // suppressing a sabotage tell would delete the signal on the surface a
        // player inspects.
        let records = read(true);
        assert!(
            records
                .iter()
                .any(|record| record.presentation() == Presentation::Tampered),
            "the exemption swallowed the tell",
        );
    }
}
