//! Reading the log, filtering it, and checking it has not been tampered with.
//!
//! §3: unlogged output is forbidden, so the record stream **is** the log —
//! `orb.log` is the whole of it, and a domain log is that same stream filtered by
//! where each line happened. One stream read several ways, rather than several
//! streams that can disagree.
//!
//! §7 calls the filter *"the only model that survives the eldritch renderer
//! corrupting output"*: matching runs over field values and never over anything a
//! view put on screen, so a narrow window cannot change what a search returns and
//! a **poisoned** log still yields its text.
//!
//! **No prose here.** Rule 6 and §12 put authored text in content files.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, Record, RecordKind, Sift, Value};

use crate::parser::{Intent, NounKind, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Cwd};

use super::navigate::{find_place, find_script, root};
use super::{LOG, acknowledge, missing};

/// Read a file.
///
/// `orb.log` is the whole stream. A **domain** log is that same stream filtered
/// by where each line happened — see [`domain_names`].
///
/// A file with nothing in it reads back as a count of zero, which is a true
/// answer. It used to report success having read nothing, which is worse than an
/// error — §6 forbids a bare error so a player is never left guessing, and a
/// cheerful completion over an unread file leaves them guessing anyway.
pub(super) fn peruse(intent: &Intent, world: &mut World) {
    // Snapshot first: the stream being read is the stream being written to.
    let tampered = tampered_source(world, intent);
    let lines = read_file(world, intent, None);
    emit(world, Verb::Peruse, &lines, tampered);
}

/// Filter a file.
pub(super) fn sift(intent: &Intent, world: &mut World) {
    let Some(pattern) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Sift, world);
        return;
    };
    let tampered = tampered_source(world, intent);
    let lines = read_file(world, intent, Some(&pattern));
    emit(world, Verb::Sift, &lines, tampered);
}

/// Report whether a surface has been interfered with (§8.1).
///
/// The command-detectable half of every tell. It is what makes the visual
/// signature a *speed bonus for observant players rather than a requirement* —
/// and what makes sabotage playable at all without sight.
pub(super) fn verify(intent: &Intent, world: &mut World) {
    let Some(target) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Verify, world);
        return;
    };
    match here_or_place(world, &target) {
        Some(node) => tower::verify(world, node),
        None => missing(Verb::Verify, &target, world),
    }
}

/// The node `target` names: something where the player stands, or a place.
///
/// `&World`, not `&mut`: it only looks. Asking for exclusive access to read
/// makes a lookup impossible to call from anywhere already holding a shared
/// borrow, for no reason the body can point at.
fn here_or_place(world: &World, target: &str) -> Option<Entity> {
    let cwd = world.resource::<Cwd>().0;
    tower::children_of(world, cwd)
        .into_iter()
        .find(|node| {
            world
                .get::<tower::Name>(*node)
                .is_some_and(|n| n.0 == target)
        })
        .or_else(|| {
            let root = root(world);
            find_place(world, root, target)
        })
        // ...and a spell, wherever it is kept. Scripts are nameable from
        // anywhere (`tower::scene`), so they have to be findable from anywhere
        // or `peruse` resolves and then reads nothing — see [`find_script`].
        .or_else(|| find_script(world, target))
        // ...and whatever the arsenal holds, for exactly the reason above. It is
        // the third thing nameable from everywhere, so it is the third that has
        // to be findable from everywhere: `verify clarity` from the archive
        // resolving and then reporting "no such thing" is §15's dead end,
        // arriving through the exemption meant to remove one.
        .or_else(|| tower::kept(world, target))
}

/// Which file an intent names, if any.
///
/// **By the resolved `kind`, not by a `.log` suffix.** The parser has already
/// proved this argument is a [`NounKind::File`] — `Verb::Peruse`'s signature says
/// so — and re-deriving it from the string threw that work away. The first
/// readable file that is not a `.log` (a `.spell` in Phase 1, a `notes.txt`)
/// resolves cleanly, reaches here, matches nothing, and `peruse` reports a
/// zero-length read: exactly the symptom
/// `an_empty_file_reads_as_empty_rather_than_as_success` exists to make honest,
/// so the suite stays green while the command is broken. Rule 4 — the resolved
/// record is what downstream reads.
fn named_file(intent: &Intent) -> Option<&str> {
    intent
        .arguments
        .iter()
        .find(|argument| NounKind::Readable.accepts(argument.kind))
        .map(|argument| argument.value.as_str())
}

/// Whether the named file has been poisoned (§8.1).
fn tampered_source(world: &World, intent: &Intent) -> bool {
    named_file(intent).is_some_and(|file| {
        here_or_place(world, file).is_some_and(|node| tower::poisoned(world, node))
    })
}

/// The lines a file holds, optionally filtered.
///
/// Owned because the borrow has to end before anything can be written back into
/// the same stream — reading and writing one log is the normal case here.
fn read_file(world: &World, intent: &Intent, pattern: Option<&str>) -> Vec<String> {
    let Some(file) = named_file(intent) else {
        return Vec::new();
    };

    // **Stored text wins, and does not fall through.** A `.spell` holds lines a
    // player wrote (`tower::Held`); everything else is a *view* over the record
    // stream. Falling through on an empty spell would search the log for a file
    // name and report whatever it found, which is the same class of defect as
    // `bind` falling through to `sift` — a wrong answer wearing a right one's
    // clothes. An empty spell reads as empty, which is true.
    if let Some(node) = here_or_place(world, file)
        && let Some(held) = world.get::<tower::Held>(node)
    {
        return held
            .0
            .iter()
            .filter(|line| {
                pattern.is_none_or(|pattern| orbs_render::contains_ignoring_case(line, pattern))
            })
            .cloned()
            .collect();
    }

    let domain = (file != LOG).then(|| domain_names(world, file.trim_end_matches(".log")));
    let sift = pattern.map(Sift::new);

    world
        .resource::<Scrollback>()
        .records()
        .iter()
        // Never its own output, or each run would match everything the last one
        // emitted and the stream would double every time.
        .filter(|record| record.kind() != RecordKind::LogLine)
        .filter(|record| domain.as_ref().is_none_or(|names| in_domain(record, names)))
        .filter(|record| sift.as_ref().is_none_or(|sift| record.matches(sift)))
        // A search shows the fields it matched on; a read shows the drawn line.
        // See `Record::to_line_verbatim`.
        .map(|record| {
            if sift.is_some() {
                record.to_line_verbatim()
            } else {
                record.to_line()
            }
        })
        .collect()
}

/// A domain's own name plus every fixture standing in it.
///
/// **The domain log was permanently empty**, and the filter looked right: it
/// compared `FieldName::Source` against `"laboratory"`. Nothing in §10.1's
/// pipeline ever writes that — `transmute` puts the *instrument* in `Source`, and
/// `carry` writes no `Source` at all — so `peruse laboratory.log` returned zero
/// lines after a full brew, and the one test covering it passed by never brewing
/// first.
///
/// A domain's log is what happened **in** that domain, which is the domain itself
/// or anything standing in it. Derived from the world rather than stamped on each
/// record, because the alternative is every `push` site remembering to carry a
/// room it does not otherwise care about — and the sites that forgot are exactly
/// how this broke.
fn domain_names(world: &World, domain: &str) -> Vec<String> {
    let mut names = vec![domain.to_owned()];
    let Some(cwd) = world.get_resource::<Cwd>().map(|cwd| cwd.0) else {
        return names;
    };
    // Walk from where the tree begins: a log is readable from wherever it is
    // held, and §7 makes a domain nameable from anywhere.
    let at = tower::filesystem_root(world, cwd);
    if let Some(place) = find_place(world, at, domain) {
        names.extend(
            tower::children_of(world, place)
                .into_iter()
                .filter_map(|node| world.get::<tower::Name>(node).map(|name| name.0.clone())),
        );
    }
    names
}

/// Whether a record happened in a domain naming any of `names`.
fn in_domain(record: &Record<'_>, names: &[String]) -> bool {
    [FieldName::Source, FieldName::Path, FieldName::Origin]
        .into_iter()
        .filter_map(|field| record.field(field))
        .any(|value| match value {
            Value::Text(text) => names.iter().any(|name| name == crate::parser::leaf(text)),
            _ => false,
        })
}

/// Write `lines` back as log output, damaged if the source was poisoned.
fn emit(world: &mut World, verb: Verb, lines: &[String], tampered: bool) {
    let mut scrollback = world.resource_mut::<Scrollback>();
    tower::emit_lines(scrollback.records_mut(), verb, lines, tampered);
}
