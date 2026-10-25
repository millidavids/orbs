//! Reading the log, filtering it, and checking it has not been tampered with.
//!
//! The record stream **is** the log (§3): `orb.log` is the whole of it, a
//! domain log the same stream filtered by where each line happened. Matching
//! runs over field values, never what a view drew, so the eldritch renderer
//! cannot change a search (§7).

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, Record, RecordKind, Sift, Value};

use crate::parser::{Intent, NounKind, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Cwd};

use super::navigate::{find_place, find_script, root};
use super::{LOG, acknowledge, missing};

/// Read a file.
///
/// `orb.log` is the whole stream; a domain log is that stream filtered by where
/// each line happened — see [`domain_names`].
///
/// An empty file reads back as a count of zero, never as success (§6).
pub(super) fn peruse(intent: &Intent, world: &mut World) {
    // Snapshot first: the stream being read is the stream being written to.
    let tampered = tampered_source(world, intent);
    let mut listing = read_file(world, intent, None);
    // A round can poison a log nothing has been written to yet. It still lies:
    // one forged entry, drawn damaged, so looking finds what `verify` finds.
    if tampered && listing.lines.is_empty() && listing.kind == RecordKind::LogLine {
        listing.lines.push(
            world
                .resource::<crate::content::Prose>()
                .line("log_forged", &[]),
        );
    }
    emit(world, Verb::Peruse, &listing, tampered);
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
    let listing = read_file(world, intent, Some(&pattern));
    emit(world, Verb::Sift, &listing, tampered);
}

/// Report whether a surface has been interfered with (§8.1).
///
/// The command half of every tell, so the visual one is a speed bonus and
/// sabotage is playable without sight.
pub(super) fn verify(intent: &Intent, world: &mut World) {
    // Bare is the audit — §8.1's `verify --all`, spelled the way every other
    // widening here is (see `Verb::signature`). It used to `acknowledge` and do
    // nothing, which was the one form §8.1 actually prices.
    let Some(target) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        super::audit::sweep(world);
        return;
    };
    let Some(node) = here_or_place(world, &target) else {
        missing(Verb::Verify, &target, world);
        return;
    };

    // Charged before the answer, and only for a surface. §8.1 rations *"which
    // surface do I inspect first"*, and a node that is neither a log nor a shelf
    // can never report tampering — see `Surface::of`.
    if let Some(surface) = tower::Surface::of(world, node) {
        let now = *world.resource::<crate::tick::Tick>();
        if let Some(left) = world.resource::<tower::Cooling>().left(surface, now) {
            super::audit::say_cooling(world, surface, left);
            return;
        }
        world.resource_mut::<tower::Cooling>().spend(surface, now);
    }
    tower::verify(world, node);
}

/// The node `target` names: something where the player stands, or a place.
///
/// `&World`, since it only looks.
fn here_or_place(world: &World, target: &str) -> Option<Entity> {
    // Three scopes, widening, and the order is the rule: what is in the room
    // outranks the same name elsewhere in the tower. See `tower::reach`.
    tower::reach::look(world)
        .find(target)
        .or_else(|| {
            let root = root(world);
            find_place(world, root, target)
        })
        // ...and a spell, wherever it is kept. Scripts are nameable from
        // anywhere (`tower::scene`), so they have to be findable from anywhere
        // or `peruse` resolves and then reads nothing — see [`find_script`].
        .or_else(|| find_script(world, target))
        // ...and whatever the arsenal holds: nameable from everywhere, so
        // findable from everywhere (§15).
        .or_else(|| tower::kept(world, target))
}

/// Which file an intent names, if any.
///
/// By the resolved `kind`, not a `.log` suffix: the parser already proved it
/// a [`NounKind::File`] (rule 4).
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

/// What a read produced, and which of §3's diagnostic surfaces it came from.
///
/// Together, since the surface is knowable only here.
struct Listing {
    lines: Vec<String>,
    kind: RecordKind,
}

/// The lines a file holds, optionally filtered.
///
/// Owned, so the borrow ends before anything is written back to the stream.
fn read_file(world: &World, intent: &Intent, pattern: Option<&str>) -> Listing {
    let Some(file) = named_file(intent) else {
        return Listing {
            lines: Vec::new(),
            // Nothing was named, so nothing was read. A log is the surface a
            // bare read defaults to everywhere else in this file.
            kind: RecordKind::LogLine,
        };
    };

    // Stored text (`tower::Held`, what a person wrote) wins and never falls
    // through to the log. It is the script/log split, decided here rather
    // than by the `.spell` suffix.
    if let Some(node) = here_or_place(world, file)
        && let Some(held) = world.get::<tower::Held>(node)
    {
        return Listing {
            lines: held
                .0
                .iter()
                .filter(|line| {
                    pattern.is_none_or(|pattern| orbs_render::contains_ignoring_case(line, pattern))
                })
                .cloned()
                .collect(),
            kind: RecordKind::ScriptLine,
        };
    }

    let domain = (file != LOG).then(|| domain_names(world, file.trim_end_matches(".log")));
    let sift = pattern.map(Sift::new);

    let lines = world
        .resource::<Scrollback>()
        .records()
        .iter()
        // Never its own output, of either listing kind, or the stream doubles
        // every read.
        .filter(|record| !matches!(record.kind(), RecordKind::LogLine | RecordKind::ScriptLine))
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
        .collect();

    Listing {
        lines,
        kind: RecordKind::LogLine,
    }
}

/// A domain's own name plus every fixture standing in it.
///
/// A domain's log is what happened in the domain or anything standing in it —
/// derived from the world, not stamped on each record, since the sites that
/// forgot a room are how it once stayed empty.
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

/// Write a listing back onto the transcript, damaged if the source was poisoned.
fn emit(world: &mut World, verb: Verb, listing: &Listing, tampered: bool) {
    let mut scrollback = world.resource_mut::<Scrollback>();
    tower::emit_lines(
        scrollback.records_mut(),
        verb,
        listing.kind,
        &listing.lines,
        tampered,
    );
}
