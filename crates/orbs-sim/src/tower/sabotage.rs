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

use super::node::Name;
use crate::parser::Verb;
use rand::Rng as _;

use crate::rng::{RngStream, Rngs};
use crate::session::Scrollback;

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
    let tampered = poisoned(world, target);

    world
        .resource_mut::<Scrollback>()
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
        })
        .finish();
}

/// Roughly how many ticks pass between interferences.
///
/// §15's scenario is fifteen minutes, which at 1 Hz is 900 ticks, so this puts a
/// handful in a tester's session — enough that log-poisoning is *met* rather
/// than described, and rare enough that a clean log is still the normal case a
/// tampered one stands out against.
///
/// §5.3 caps aberration arrival so repairs cannot spiral; the full adversarial
/// model is Phase 2, and this is the single surface §15 asks Phase 0 to exercise.
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
    logs: Query<Entity, (With<Log>, Without<Poisoned>)>,
    mut commands: Commands,
) {
    let Some(target) = logs.iter().next() else {
        return;
    };
    // An integer draw rather than a ratio helper: the same arithmetic on every
    // platform and every `rand` release, which replay depends on.
    let roll: u64 = rngs.stream(RngStream::Threat).random();
    if roll.is_multiple_of(DRIFT_INTERVAL) {
        commands.entity(target).insert(Poisoned);
    }
}

/// A file that accumulates what a domain did.
#[derive(Component, Debug, Clone, Copy)]
pub struct Log;

/// Which line of a poisoned log carries the visible tell.
///
/// Every third, so a reader has something to compare against — a log where
/// *everything* looked wrong would read as a rendering fault rather than as
/// interference, and §8.1 wants the player to spot the odd one out.
const TELL_EVERY: usize = 3;

/// Copy `lines` into `into` as log output, damaging some if the source is
/// poisoned.
///
/// The damage is structural, never textual: a dropped field and the tampered
/// face. §8.1's "malformed record boundaries" is exactly a record missing a
/// field, which the table view already draws as a gap.
pub fn emit_lines(into: &mut Records, verb: Verb, lines: &[String], tampered: bool) {
    into.push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .count(
            FieldName::Quantity,
            u64::try_from(lines.len()).unwrap_or(u64::MAX),
        )
        .finish();

    for (index, line) in lines.iter().enumerate() {
        let damaged = tampered && index % TELL_EVERY == 0;
        let row = into.push(RecordKind::LogLine);
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
            row.count(
                FieldName::Tick,
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
        emit_lines(&mut records, Verb::Peruse, &lines, tampered);
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
                record.field(FieldName::Tick).is_none(),
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
