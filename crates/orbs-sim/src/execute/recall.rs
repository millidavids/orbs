//! The in-world manual (§6.1): how a thing is made, in the order you make it.
//!
//! Not part of §10.1's loop — it is read before committing an instrument to a
//! route, because a goal with two right answers is only a decision if the player
//! can see both. It is also where the retired brewing words land (§19): `make a
//! potion of clarity` resolved here and merely *acknowledged* for a while, a
//! dead end, which §15 weighs above the raw resolution rate.
//!
//! The first version emitted the raw breadth-first walk as five bare columns a
//! row: backwards (the walk starts at the goal), unlabelled, clipped by tiling,
//! and full of routes to the `rock-salt` already in the dispensary. So: the
//! primary route in dependency order, numbered, one authored sentence per step
//! (rule 6), stopping at what the player has; alternatives after, marked. The
//! fields stay on the record (rule 4) for `sift` and pipes.

use std::collections::BTreeSet;

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Prose, Recipe, Recipes};
use crate::parser::{Group, Intent, Verb, synonyms_of};
use crate::session::Scrollback;
use crate::tower::{self, Name, Store};

use super::missing;
use super::navigate::root;

/// How deep a route may go before the walk gives up.
///
/// The longest authored chain is four and `seen` already stops a cycle; this is
/// the backstop against a malformed recipe table recursing until the stack goes.
const MAX_DEPTH: usize = 16;

/// One command, at length.
///
/// The synopsis and description are authored; the spellings are read off
/// `SYNONYMS`. `signature()` carries no connectives — `move` is `[Reagent,
/// Place?, Place]` with no `to` — so a generated synopsis would read `move
/// reagent place place`, which nobody types; a test keeps the authored line
/// naming every required slot instead.
///
/// `RecordKind::Message` throughout, not `Entry`: a page is instructions, and
/// `Message` wraps where `Entry` tiles. Long pages page for free, because
/// `unfurl` searches by record.
fn page(world: &mut World, verb: Verb) {
    let canonical = verb.canonical();

    // The synopsis, and the line saying what it is for. Both authored; a missing
    // key draws as the key itself, so an omission is visible rather than blank.
    for key in [
        format!("man_{canonical}_use"),
        format!("man_{canonical}_gloss"),
    ] {
        say(world, canonical, &key);
    }

    // One record, not one per authored line. The width lint caps prose at 70
    // cells so a description is written in pieces, and separate records make
    // those hard breaks — a 100-column pane drew ragged 50-cell strips. One
    // record wraps to whatever the pane is.
    section(world, "man_page_what");
    let body: Vec<String> = (1..=MAX_LINES)
        .map(|line| format!("man_{canonical}_{line}"))
        .take_while(|key| world.resource::<Prose>().has(key))
        .map(|key| world.resource::<Prose>().line(&key, &[]))
        .collect();
    if !body.is_empty() {
        line(world, canonical, &body.join(" "));
    }

    let examples: Vec<String> = (1..=MAX_LINES)
        .map(|n| format!("man_{canonical}_eg{n}"))
        .take_while(|key| world.resource::<Prose>().has(key))
        .collect();
    if !examples.is_empty() {
        section(world, "man_page_like");
        for key in examples {
            say(world, canonical, &key);
        }
    }

    // One row per register, not one per phrase: `attend` has five spellings and
    // four are plain, so a row each made the section longer than the description
    // above it. Read off the table, so it cannot go stale. Arcane first — what
    // the echo teaches and an expert types.
    section(world, "man_page_said");
    let mut by_register: Vec<(crate::parser::Register, Vec<String>)> = Vec::new();
    for (register, phrase) in synonyms_of(verb) {
        match by_register.last_mut() {
            Some((last, phrases)) if *last == register => phrases.push(phrase),
            _ => by_register.push((register, vec![phrase])),
        }
    }
    for (register, phrases) in by_register {
        let phrase = phrases.join(", ");
        // The arrangement is authored, like every route step: composing
        // `"arcane  grind"` in Rust would be a frontend decision made in the sim
        // (rule 4) and a sentence in source (rule 6). The fields stay fields, so
        // `sift` still works on them.
        let said = world
            .resource::<Prose>()
            .line("man_said", &[("kind", register.label()), ("name", &phrase)]);
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Message)
            .text(FieldName::Name, canonical)
            .text(FieldName::Kind, register.label())
            .text(FieldName::Message, &said)
            .finish();
    }

    let also = format!("man_{canonical}_also");
    if world.resource::<Prose>().has(&also) {
        section(world, "man_page_also");
        say(world, canonical, &also);
    }
}

/// How many description or example lines a page may carry.
///
/// Eight already overflows the 80x22 floor, so this is a backstop against an
/// authored run with no end rather than a budget anyone reaches.
const MAX_LINES: usize = 8;

/// A page heading.
fn section(world: &mut World, key: &str) {
    let heading = world.resource::<Prose>().line(key, &[]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Section)
        .text(FieldName::Kind, &heading)
        .finish();
}

/// One line of a page, already composed.
fn line(world: &mut World, canonical: &str, message: &str) {
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Message)
        .text(FieldName::Name, canonical)
        .text(FieldName::Message, message)
        .finish();
}

/// One authored line of a page.
fn say(world: &mut World, canonical: &str, key: &str) {
    let message = world.resource::<Prose>().line(key, &[]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Message)
        .text(FieldName::Name, canonical)
        .text(FieldName::Message, &message)
        .finish();
}

/// The subject that answers *what can I write in a spell, here*.
///
/// A `recall_` key of the same name is what makes it nameable, exactly as a
/// material's page does.
const SCRIPTING: &str = "scripting";

/// The subject that answers *how do I make one at all*.
///
/// Not `primer`: [`primer`] is already the room's three-line introduction in
/// this module, and one word for two pages is how the next reader merges them.
///
/// `apprentice` is §12's own word — *"diegetic apprenticeship: the orb teaches
/// as a mentor"* — and reads as the request rather than a label. Swept clean on
/// similarity with `app` a free prefix; `spellcraft` scores 925 against `spell`,
/// and `crafting` and `writing` both 667 against `scripting`, the one page it
/// must not be confused with.
const APPRENTICE: &str = "apprentice";

/// The name the primer's worked example gives its spell.
///
/// Fixed rather than the room's, because the player types it: a name that
/// changed with the room would make the `invoke` line below wrong the moment
/// they walked next door.
const EXAMPLE: &str = "morning";

/// The room the example falls back to where there is no work to script.
const FALLBACK: &str = "laboratory";

/// How to make a spell, from nothing, in the order it happens.
///
/// The tutorial to [`scripting`]'s reference: that page answers *what may I
/// write* and teaches nobody how to start, because no listing of words teaches
/// an order. Seven steps — `scribe`, `edit`, the lines, `<escape>`, `quit`,
/// `invoke`, the log rather than the pane — one of which (`quit` being the save)
/// is otherwise discovered by losing work. §12's *in-world grimoire*, in the
/// "always" column, which is why it is a `recall` page and not a scripted
/// sequence.
///
/// The worked example comes from the room, for [`scripting`]'s reason: a spell's
/// shape is the same everywhere and its lines are not, so showing `grind sage`
/// to somebody standing in the lens teaches a room they are not in. A room with
/// no work to script says whose lines it is borrowing, which is what the
/// grimoire's own primer says too.
fn apprentice(world: &mut World) {
    let cwd = world.resource::<tower::Cwd>().0;
    let here = tower::where_at(world, cwd);
    let borrowed = !world.resource::<Prose>().has(&format!("craft_line_{here}"));
    let room = if borrowed { FALLBACK.to_owned() } else { here };
    let example = |world: &World, key: &str| world.resource::<Prose>().line(key, &[]);

    say(world, APPRENTICE, "recall_apprentice");
    for key in [
        "craft_what_1",
        "craft_what_2",
        "craft_what_3",
        "craft_what_4",
        "craft_what_5",
    ] {
        let text = example(world, key);
        line(world, APPRENTICE, &text);
    }

    section(world, "man_craft_write");
    if borrowed {
        let note = example(world, "craft_elsewhere");
        line(world, APPRENTICE, &note);
    }
    let (first, second) = (
        example(world, &format!("craft_line_{room}")),
        example(world, &format!("craft_then_{room}")),
    );
    for (name, key) in [
        (format!("scribe {EXAMPLE}"), "craft_scribe"),
        ("edit".to_owned(), "craft_edit"),
        (first, "craft_first"),
        (second, "craft_second"),
        ("<escape>".to_owned(), "craft_escape"),
        ("quit".to_owned(), "craft_quit"),
    ] {
        let gloss = example(world, key);
        entry(world, &name, "", Some(&gloss));
    }

    section(world, "man_craft_cast");
    for (name, key) in [
        (format!("invoke {EXAMPLE}"), "craft_invoke"),
        (format!("peruse {room}.log"), "craft_log"),
    ] {
        let gloss = example(world, key);
        entry(world, &name, "", Some(&gloss));
    }

    section(world, "man_craft_again");
    for (name, key) in [
        ("repeat 4", "craft_repeat"),
        ("end", "craft_end"),
        ("repeat until <question>", "craft_until"),
    ] {
        let gloss = example(world, key);
        entry(world, name, "", Some(&gloss));
    }

    section(world, "man_craft_ask");
    let asking = example(world, &format!("craft_ask_{room}"));
    for (name, key) in [
        (asking, "craft_if"),
        ("else".to_owned(), "craft_else"),
        ("interpret".to_owned(), "craft_interpret"),
    ] {
        let gloss = example(world, key);
        entry(world, &name, "", Some(&gloss));
    }

    section(world, "man_craft_keep");
    for (name, key) in [
        (format!("bind {EXAMPLE}"), "craft_bind"),
        (String::new(), "craft_bind_why"),
    ] {
        let gloss = example(world, key);
        entry(world, &name, "", Some(&gloss));
    }

    section(world, "man_craft_next");
    for (name, key) in [
        ("recall scripting", "craft_more_words"),
        ("recall repeat", "craft_more_one"),
        ("guide", "craft_more_guide"),
    ] {
        let gloss = example(world, key);
        entry(world, name, "", Some(&gloss));
    }
}

/// What a spell is made of, and what this room lets one ask about.
///
/// Three sections, and only the last moves. The words and the question shapes
/// are the same in every room because the grammar is; what a question can *name*
/// is not — the laboratory's instruments answer `is idle`, the archive's ways
/// answer `has passage`, and neither is guessable from the other. So the third
/// section is built from the room, the way [`overview`] builds its verb list
/// from [`offered`](super::offered).
fn scripting(world: &mut World) {
    let cwd = world.resource::<tower::Cwd>().0;
    let places: Vec<String> = tower::children_of(world, cwd)
        .into_iter()
        .filter(|node| world.get::<tower::Fixture>(*node).is_some())
        .filter_map(|node| world.get::<tower::Name>(node).map(|name| name.0.clone()))
        .collect();
    // The readings belong to a set — a way, a socket — so they are listed where
    // that set is. They resolve everywhere, but naming them in the laboratory
    // would teach a word the room can never answer.
    //
    // This asked `Reading` and printed the maze's list regardless: the lens's
    // sockets and sigils carry that marker too, so the lens taught `passage wall
    // exit back spoil marks gleaning` and none of its own six deltas. See
    // [`readings_at`](tower::readings_at).
    let readings = tower::readings_at(world, cwd);

    say(world, SCRIPTING, "recall_scripting");

    section(world, "man_scripting_words");
    // `SpellWord::shape`, not a copy of it. This was a byte-identical second
    // table the `orbs-shell` extraction walked past, so adding `part` meant
    // editing both and the next word would have left this page printing a stale
    // shape — the exact failure the extraction was for.
    for word in crate::parser::SpellWord::ALL {
        entry(world, word.canonical(), word.shape(), None);
    }

    section(world, "man_scripting_asking");
    for key in SHAPES {
        let line = world.resource::<Prose>().line(key, &[]);
        entry(world, &line, "", None);
    }

    // The sets before the names, because `for each <set>` is unusable without
    // them and there is nowhere else to find one: a set is not a place you can
    // `survey` and not a word the scene offers. Derived from the room rather
    // than listed, so a domain declaring a group gets it here the same tick —
    // the rule `here` below follows too.
    let sets = tower::groups_at(world, cwd);
    if !sets.is_empty() {
        section(world, "man_scripting_sets");
        for set in sets {
            entry(world, &set, "", None);
        }
    }

    section(world, "man_scripting_here");
    for place in places {
        entry(world, &place, "", None);
    }
    for word in readings {
        entry(world, word, "", None);
    }
}

/// The shapes a question takes, in the order they are worth learning.
///
/// `than` sits after `count`: it is the same comparison with the world on both
/// sides, and is no use to a player who has not met a number in a question.
const SHAPES: [&str; 6] = [
    "man_scripting_shape_is",
    "man_scripting_shape_has",
    "man_scripting_shape_count",
    "man_scripting_shape_than",
    // The far side's own reading, and its two operators. Its own row rather than
    // lengthening `than`'s: one compares a word against itself in two places,
    // the other weighs one thing against another, and a player reaching for the
    // second is not looking in the first's sentence.
    "man_scripting_shape_weigh",
    "man_scripting_shape_join",
];

/// One row under a section: a name, and what follows it.
fn entry(world: &mut World, name: &str, shape: &str, describing: Option<&str>) {
    let mut records = world.resource_mut::<Scrollback>();
    let records = records.records_mut();
    let mut row = records.push(RecordKind::Entry).text(FieldName::Name, name);
    // An empty field is not an absent one — it draws as trailing blanks and
    // speaks as a labelled silence, which is why `overview` guards the same way.
    if !shape.is_empty() {
        row = row.text(FieldName::Kind, shape);
    }
    // What it does, when the row has earned one. A field rather than composed
    // into the name: the view decides between a described column and an index,
    // `sift` matches the sentence, and a reader hears it labelled. Absent on a
    // tower-wide verb, which keeps that run tiling.
    if let Some(gloss) = describing {
        row = row.text(FieldName::Detail, gloss);
    }
    row.finish();
}

/// The three halves of a room's primer, in the order they are printed.
///
/// Named once so the prose keys, the emitter and
/// `every_room_a_player_can_stand_in_explains_itself` cannot disagree about how
/// many there are — which is how a fourth gets authored and never drawn.
const KEYS: [&str; 3] = ["here", "start", "solve"];

/// What the room you are standing in is, and how its puzzle is worked.
///
/// `man_here_<room>` says what the place *is*, `man_start_<room>` how to begin
/// its puzzle, `man_solve_<room>` how to finish one — what-it-is before
/// how-it-works, the order [`describe`] uses for the same split. Three rather
/// than two because start and solve are followed at different times, and because
/// the 70-cell width lint against §4's 80×22 floor cut a 112-cell first draft.
///
/// `man_`, not `recall_`:
/// [`Prose::topics`](crate::content::Prose::topics) strips `recall_` to decide
/// what is *nameable*, so `recall_here_lens` would register `here_lens` as a
/// subject — the trap this module paid for at `grimoire_step_or` and
/// `clarity_use`. `man_` is the manual's own furniture prefix, not a noun space.
///
/// Guarded on [`Prose::has`], so an unbuilt room has no primer;
/// `every_room_a_player_can_stand_in_explains_itself` stops that being a silent
/// omission for a room that *is* built.
fn primer(world: &mut World) {
    let room = here(world);
    if room.is_empty() {
        return;
    }

    let keys = KEYS.map(|part| format!("man_{part}_{room}"));
    let prose = world.resource::<Prose>();
    let lines: Vec<String> = keys
        .iter()
        .filter(|key| prose.has(key))
        .map(|key| prose.line(key, &[]))
        .collect();
    if lines.is_empty() {
        return;
    }

    // The room's own name as the heading, not a `Prose` key like the group
    // headings below: authoring a `man_section_<room>` per room to say the
    // room's name back would be furniture with a translation cost.
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Section)
        .text(FieldName::Kind, &room)
        .finish();
    for message in lines {
        line(world, Verb::Recall.canonical(), &message);
    }
    road(world, &room);
}

/// The room's mastery line, as words, at the end of its primer (§11.5).
///
/// From the same reading the road draws, so asking and glancing agree. A room
/// with no line, or one the player cannot enter, says nothing.
fn road(world: &mut World, room: &str) {
    let Some(line) = tower::mastery(world)
        .into_iter()
        .find(|line| line.domain == room && line.open)
    else {
        return;
    };
    let (reached, of) = line.reached();
    let message = match line.next() {
        Some(next) => {
            let prose = world.resource::<Prose>();
            // The deed's own count, interpolated. Spelling it out — *"five
            // potions brewed"* — disagreed with the live `3 of 5` the moment a
            // game had a length. `counted` rather than `line`, because six deeds
            // ask for one of a thing and *"1 charms laid"* is worse than either.
            let deed = prose.counted(&format!("mastery_{}", next.id), next.needed);
            let count = prose.line(
                "weave_progress",
                &[
                    ("count", &next.done.to_string()),
                    ("quantity", &next.needed.to_string()),
                ],
            );
            prose.line(
                "primer_road",
                &[
                    ("name", room),
                    ("count", &reached.to_string()),
                    ("quantity", &of.to_string()),
                    ("detail", &format!("{deed}, {count}")),
                ],
            )
        }
        None => world
            .resource::<Prose>()
            .line("primer_road_done", &[("name", room)]),
    };
    self::line(world, Verb::Recall.canonical(), &message);
}

/// The room the player is standing in, for the manual's own purposes.
///
/// [`domain_of`](crate::tower::domain_of) returns `None` at `/tower` and at
/// `/grimoire` — no work happens there — but both are places a player can stand
/// and ask for help, so this falls back to the node itself rather than nothing.
fn here(world: &World) -> String {
    let cwd = world.resource::<crate::tower::Cwd>().0;
    let node = crate::tower::domain_of(world, cwd).unwrap_or(cwd);
    crate::tower::where_at(world, node)
}

/// Everything you can type where you are standing, grouped.
///
/// `Section` + `Entry` rather than a formatted string: `Section` stacks as a
/// `[heading]` and `Entry` tiles across the pane, the pair `survey` already
/// emits, so this needed no render code — and as records `sift` works on them
/// and §14 hears one utterance per verb rather than a wall of spacing.
///
/// It lists what works *here*, through [`offered`](super::offered) — the boot
/// report's filter: live, ungated, in scope (§7, the rule Tab follows), because
/// offering a word the parser would refuse is the dead end §15 weighs heaviest.
/// So the listing is narrower than the manual, which answers `recall grind`
/// from anywhere: a manual you can only read in the right room has a lock on
/// it. That asymmetry is recorded in §19.
fn overview(world: &mut World) {
    // The room first, the vocabulary second. Twenty-five words answer *what may
    // I type* and never *what is this place for*, and a player typing `help` in
    // the lens is asking what a ward is. §6.1 makes `recall` the in-world
    // manual, and a manual opening with an index is a reference.
    primer(world);

    let offered = super::offered(world);

    for group in Group::ALL {
        let members: Vec<Verb> = offered
            .iter()
            .copied()
            .filter(|verb| verb.group() == group)
            .collect();
        // A heading over nothing is furniture: the archive has no spells and no
        // destructive verb, and empty sections would be most of the overview.
        if members.is_empty() {
            continue;
        }

        section(world, group.key());
        // Through the shared [`entry`], which owns the empty-field rule: an
        // empty `Kind` draws as trailing blanks and speaks as a labelled
        // silence, and `status` and `undo` take no argument.
        //
        // This room's own words are described; the tower-wide ones are indexed.
        // `grind` is opaque and met in the room offering it, `quit` is neither.
        // Described throughout, this page is ~35 rows against a floor that fits
        // 19, and `recall <verb>` still has the full page. `anchor` asks *which
        // fixture must stand here* — `Scene::offers`'s own question — so a new
        // domain's verbs describe themselves with no list to maintain.
        //
        // Decided per section, not per verb: `move` and `wield` are tower-wide
        // and sit under *the work*, so describing only the anchored verbs left
        // the section half described, and such a run falls back to stacking with
        // the sentence jammed on unpadded. A section is the unit a reader sees,
        // and §3 is the deeper reason — raggedness is sabotage's vocabulary.
        let describes = members.iter().any(|verb| verb.anchor().is_some());
        for verb in members {
            let gloss = describes.then(|| {
                let key = format!("man_{}_gloss", verb.canonical());
                world.resource::<Prose>().line(&key, &[])
            });
            entry(
                world,
                verb.canonical(),
                verb.signature_label(),
                gloss.as_deref(),
            );
        }
    }

    // Two pointers out, answering different questions. The first is the one a
    // lost player needs: nothing above says the orb can be taught to type these
    // for you, so `help` in every room would never reveal the game has spells in
    // it, and §12's "always" reference nobody can find is not one. The second
    // says *this is the vocabulary here*, without which somebody in the archive
    // never learns the laboratory has words of its own.
    for key in ["man_teachable", "man_elsewhere"] {
        let message = world.resource::<Prose>().line(key, &[]);
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Message)
            .text(FieldName::Name, Verb::Recall.canonical())
            .text(FieldName::Message, &message)
            .finish();
    }
}

/// Show how a thing is made, and every way there is to make it.
pub(super) fn recall(intent: &Intent, world: &mut World) {
    let Some(topic) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        // The overview. Unreachable until the slot became optional: a required
        // slot with fillers never yields an argument-less intent, so bare
        // `recall` — and `help`, `man` and `?` — opened a numbered prompt of the
        // four alphabetically-first subjects, which is §6's no-bare-error rule
        // failing at the one command whose job is answering. See
        // `TOPIC_OPTIONAL`.
        overview(world);
        return;
    };

    // The manual's own pages, before the recipe walk: a verb canonical cannot
    // shadow a recipe, but the day a recipe is named after a verb the page is
    // what the player meant.
    //
    // Asked of the whole word list, not the canonicals — a player asks with the
    // word they typed. `walk`, `edit` and `make` reached no page and fuzzed into
    // the archive's readings, so `recall edit` explained the maze's way out to
    // somebody asking about the spell editor (§19).
    if let Some(verb) = crate::parser::verb_of_word(&topic) {
        page(world, verb);
        return;
    }

    // The one page about the language rather than the tower, scoped like the
    // overview above: the grammar is the same everywhere, what a question can
    // *name* is not. Control words are outside `Verb::ALL`, so before this
    // `recall repeat` reached nothing and nothing taught the spell vocabulary.
    if topic == SCRIPTING {
        scripting(world);
        return;
    }

    // The tutorial beside the reference, and the split is why it is a second
    // page: `scripting` answers *what may I write*, never *how do I start*,
    // because no listing of words teaches an order.
    if topic == APPRENTICE {
        apprentice(world);
        return;
    }

    // What it is, before how it is made. A route answers *how do I get one*; a
    // player holding a potion is asking *what is this for*. This was an `else`,
    // so every finished product could tell you its five steps and not one word
    // about what it did.
    let said = describe(world, &topic);

    let plan = plan(world, &topic);
    if plan.is_empty() {
        // Not a recipe either. §6.1 makes `recall` the in-world manual, so a
        // subject like `brewing` gets authored prose (rule 6) rather than being
        // treated as a thing that does not exist.
        if !said {
            missing(Verb::Recall, &topic, world);
        }
        return;
    }

    say_plan(world, &topic, &plan);
}

/// Say what `topic` is, and how it is used. Whether anything was said.
///
/// The second key is deliberately not `recall_<topic>_use`:
/// [`Prose::topics`](crate::content::Prose::topics) strips `recall_` to decide
/// what is *nameable*, so that spelling would make `clarity_use` a subject — the
/// trap `grimoire_step_or` already paid for.
///
/// `using_` is optional and often absent — a byproduct is a thing you have, not
/// a thing you do — and where present it may honestly say the use is not built,
/// as `undo`'s page does. §15 wants that: admitting a word does nothing is the
/// cheapest way out of a dead end.
fn describe(world: &mut World, topic: &str) -> bool {
    let prose = world.resource::<Prose>();
    let what = format!("recall_{topic}");
    let how = format!("using_{topic}");
    let lines: Vec<String> = [what, how]
        .into_iter()
        .filter(|key| prose.has(key))
        .map(|key| prose.line(&key, &[]))
        .collect();
    if lines.is_empty() {
        return false;
    }
    for message in lines {
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Message)
            .text(FieldName::Name, topic)
            .text(FieldName::Message, &message)
            .finish();
    }
    true
}

/// Write the plan out, a record per step.
fn say_plan(world: &mut World, goal: &str, plan: &Plan) {
    let steps = plan.steps.len();
    let ticks: u64 = plan.steps.iter().map(|step| step.ticks).sum();

    let heading = world.resource::<Prose>().line(
        "route_line",
        &[
            ("name", goal),
            ("count", &steps.to_string()),
            ("ticks", &ticks.to_string()),
        ],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Message)
        .text(FieldName::Name, goal)
        .count(FieldName::Quantity, to_count(steps))
        .count(FieldName::Remaining, ticks)
        .text(FieldName::Message, &heading)
        .role(Role::Success)
        .finish();

    for (index, step) in plan.steps.iter().enumerate() {
        say_step(world, step, Some(index + 1));
    }
    for step in &plan.alternates {
        say_step(world, step, None);
    }
}

/// One step, as a record carrying both its facts and its sentence.
///
/// `RecordKind::Message`, not `Entry`. `Entry` speaks as a `TableRow` and tiles
/// rows across the pane, and both are wrong for an instruction: the sentence
/// *is* the content, it has to wrap rather than be cut, and a reader wants the
/// line rather than five `label: value` pairs. A `Message` carrying fields keeps
/// `sift` working on them (rule 4).
fn say_step(world: &mut World, step: &Step, index: Option<usize>) {
    // Joined once and handed on: `step_line` needs the same string, and building
    // it twice cost two allocations per step of every page and every spill line.
    let inputs = step.inputs.join(" + ");
    let message = step_line(world, step, index, &inputs);
    // The facts ride along for `sift` and a pipe, under the pipeline's own
    // names: the instrument is a `Path`, the output a `Name`, the inputs an
    // `Origin`.
    //
    // Never `Detail`. A record carrying prose draws its `Message` and its
    // `Detail`, so the inputs printed in front of the sentence written to
    // explain them — the trap `wield`'s refusal fell into. `Origin` (where a
    // thing came *from*) is what an input is.
    let mut scrollback = world.resource_mut::<Scrollback>();
    let mut record = scrollback
        .records_mut()
        .push(RecordKind::Message)
        .text(FieldName::Name, &step.output)
        .text(FieldName::Origin, &inputs)
        .text(FieldName::Path, &step.instrument)
        .text(FieldName::State, &step.leaves)
        .count(FieldName::Remaining, step.ticks)
        .text(FieldName::Message, &message);
    if let Some(index) = index {
        record = record.count(FieldName::Quantity, to_count(index));
    }
    record.finish();
}

/// One step's sentence, composed and not yet emitted.
///
/// Split out of [`say_step`] so the lens can steal it: a broken ward spills the
/// far wizard's working into `lens.log` and it has to read exactly as `recall`
/// reads, where a second renderer would be two ways of describing one recipe.
/// Only the record it lands in differs.
fn step_line(world: &World, step: &Step, index: Option<usize>, inputs: &str) -> String {
    let prose = world.resource::<Prose>();
    let heat = if step.heat {
        prose.line("route_heat", &[])
    } else {
        String::new()
    };
    // Composed in like the heat clause beside it, for the same reason: not every
    // step has one. The `+` lives in `route_leaves`, so a step that leaves
    // nothing does not print a plus with nothing after it.
    let leaves = if step.leaves.is_empty() {
        String::new()
    } else {
        prose.line("route_leaves", &[("state", &step.leaves)])
    };
    prose.line(
        if index.is_some() {
            "route_step"
        } else {
            "route_step_or"
        },
        &[
            (
                "index",
                &index.map_or_else(String::new, |n| format!("{n}.")),
            ),
            ("detail", inputs),
            ("name", &step.output),
            ("state", &leaves),
            ("source", &step.instrument),
            ("ticks", &step.ticks.to_string()),
            ("kind", &heat),
        ],
    )
}

/// How to make `goal`, as the sentences `recall` would print.
///
/// The lens's spill, and nothing else. Walks the same [`plan`] the manual does,
/// so a stolen recipe and a read one cannot describe it two ways.
pub(super) fn route_lines(world: &World, goal: &str) -> Vec<String> {
    plan(world, goal)
        .steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let inputs = step.inputs.join(" + ");
            step_line(world, step, Some(index + 1), &inputs)
        })
        .collect()
}

/// One thing to do, in the order it is done.
struct Step {
    /// What this step makes.
    output: String,
    /// Where it happens.
    instrument: String,
    /// What goes in.
    inputs: Vec<String>,
    /// The byproduct it also leaves — §10.1 gives every one of them a use, and
    /// showing it here is how a player finds the alternative route below.
    leaves: String,
    /// How long it takes.
    ticks: u64,
    /// Whether the athanor must be lit, from the recipe's own `heat` key.
    heat: bool,
}

impl Step {
    fn of(output: &str, instrument: &str, recipe: &Recipe) -> Self {
        Self {
            output: output.to_owned(),
            instrument: instrument.to_owned(),
            inputs: recipe.inputs().into_iter().map(ToOwned::to_owned).collect(),
            leaves: recipe.leaves.clone().unwrap_or_default(),
            ticks: recipe.ticks,
            heat: recipe.heat,
        }
    }
}

/// A route to something, and the other ways there are to reach it.
#[derive(Default)]
struct Plan {
    /// The primary route, in the order the steps are performed.
    steps: Vec<Step>,
    /// Every other route to a step in it, plus anything only they need.
    alternates: Vec<Step>,
}

impl Plan {
    const fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// Work out how to make `goal` from what the tower already stocks.
fn plan(world: &World, goal: &str) -> Plan {
    let stocked = stocked(world);
    let recipes = world.resource::<Recipes>();
    let mut plan = Plan::default();
    let mut seen = BTreeSet::new();

    // The goal itself is never stock: `recall rock-salt` answers with how to
    // make it although the dispensary holds some — the `depth > 0` guard inside
    // `build`.
    build(recipes, &stocked, goal, 0, &mut plan, &mut seen, true);
    plan
}

/// Push the steps that make `output`, deepest first.
///
/// Post-order, which puts the walk in *doing* order: a step's inputs come before
/// the step that consumes them, so top-down is the order a player types. The
/// breadth-first walk this replaced emitted the goal's own step first, which is
/// the last thing you do.
///
/// `primary` says whether these steps belong to the recommended route or to an
/// alternative — an alternative's sub-steps are only needed if you take it, so
/// they go with it rather than into the numbered list.
fn build(
    recipes: &Recipes,
    stocked: &BTreeSet<String>,
    output: &str,
    depth: usize,
    plan: &mut Plan,
    seen: &mut BTreeSet<String>,
    primary: bool,
) {
    if depth >= MAX_DEPTH || !seen.insert(output.to_owned()) {
        return;
    }
    // Stop at what the player has. Without it the walk gave three ways to make
    // the `rock-salt` sitting in the dispensary: true, and useless.
    if depth > 0 && stocked.contains(output) {
        return;
    }

    let routes = recipes.routes(output);
    let Some((instrument, recipe)) = routes.first() else {
        return;
    };

    for input in recipe.inputs() {
        build(recipes, stocked, input, depth + 1, plan, seen, primary);
    }
    let step = Step::of(output, instrument, recipe);
    if primary {
        plan.steps.push(step);
    } else {
        plan.alternates.push(step);
    }

    // Every other way to reach the same thing, with whatever only it needs. File
    // order decides which is primary: `recipes.toml` is authored, and its first
    // entry for an output is the designer's recommendation.
    for (instrument, recipe) in routes.iter().skip(1) {
        for input in recipe.inputs() {
            build(recipes, stocked, input, depth + 1, plan, seen, false);
        }
        plan.alternates.push(Step::of(output, instrument, recipe));
    }
}

/// Everything a store in the tower holds.
///
/// Walked from the root rather than queried: `tower::node` requires anything a
/// player can see be derived by walking `Children`, because archetype order is
/// not insertion order. The set here is order-free, but the walk keeps the rule.
fn stocked(world: &World) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut stack = vec![root(world)];
    while let Some(node) = stack.pop() {
        if world.get::<Store>(node).is_some() {
            for held in tower::children_of(world, node) {
                if let Some(name) = world.get::<Name>(held) {
                    names.insert(name.0.clone());
                }
            }
        }
        stack.extend(tower::children_of(world, node));
    }
    names
}

/// A count as a record value, saturating rather than wrapping.
fn to_count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    /// Every record the orb has drawn, as its rendered line.
    fn drawn(sim: &Sim) -> Vec<(RecordKind, String)> {
        sim.scrollback()
            .records()
            .iter()
            .map(|record| (record.kind(), record.to_line()))
            .collect()
    }

    fn run(sim: &mut Sim, line: &str) {
        sim.submit(line);
        sim.step();
    }

    #[test]
    fn a_bare_recall_lists_the_vocabulary_rather_than_asking_which_topic() {
        // The defect the manual starts from. A required slot with fillers never
        // yields an argument-less intent, so bare `recall` — and `help`, `man`
        // and `?` — returned `Ambiguous` and offered the four
        // alphabetically-first subjects. §6 forbids a bare error, and this is
        // that rule failing at the one command whose job is answering.
        let mut sim = Sim::new(1);
        run(&mut sim, "attend laboratory");
        let before = sim.scrollback().records().len();
        run(&mut sim, "help");

        assert!(
            sim.choices().is_empty(),
            "`help` asked which topic instead of answering",
        );
        let after: Vec<_> = drawn(&sim).split_off(before);
        assert!(
            after.iter().any(|(kind, _)| *kind == RecordKind::Section),
            "no headings: {after:?}",
        );
        assert!(
            after.iter().any(|(_, line)| line.starts_with("grind")),
            "the laboratory's own work was not listed: {after:?}",
        );
    }

    #[test]
    fn help_explains_the_room_before_it_lists_the_words() {
        // What the player asked for, as an ordering: an index never answers
        // *what is this place for*, and the top of the screen is what is read.
        let mut sim = Sim::new(1);
        run(&mut sim, "attend lens");
        let before = sim.scrollback().records().len();
        run(&mut sim, "help");

        let after: Vec<_> = drawn(&sim).split_off(before);
        let heading = after
            .iter()
            .position(|(kind, _)| *kind == RecordKind::Section)
            .expect("the manual printed no headings at all");
        assert_eq!(
            after[heading].1.trim(),
            "lens",
            "the first heading is not the room: {after:?}",
        );

        // The instruction half, and it must come before any group heading — those
        // are the index, and the index is what this reordering demotes.
        let first_group = after
            .iter()
            .skip(heading + 1)
            .position(|(kind, _)| *kind == RecordKind::Section)
            .expect("the vocabulary listing has gone");
        let primer: Vec<&String> = after
            .iter()
            .skip(heading + 1)
            .take(first_group)
            .map(|(_, line)| line)
            .collect();
        assert!(
            primer.iter().any(|line| line.contains("probe")),
            "the room's own puzzle is not explained before the word list: {primer:?}",
        );
    }

    #[test]
    fn every_room_a_player_can_stand_in_explains_itself() {
        // A new domain cannot ship without a primer, and the failure is silent:
        // `primer` is guarded on `Prose::has`, so an unauthored room simply
        // prints the old word list.
        //
        // Driven from `Sim::briefs()` so the rooms come from the world. `built`
        // is what makes it honest: a dark room needs nothing until `build.rs`
        // raises it.
        let sim = Sim::new(1);
        let prose = sim.world().resource::<Prose>();

        // `arsenal` and `tower` are standable and not rail domains, so
        // `briefs()` cannot see them: named, and held to the same rule.
        let extra = ["arsenal", "tower"];
        let rooms: Vec<String> = sim
            .briefs()
            .into_iter()
            .filter(|brief| brief.built)
            .map(|brief| brief.name.to_owned())
            .chain(extra.into_iter().map(str::to_owned))
            .collect();
        assert!(rooms.len() > extra.len(), "no rooms were found to check");

        for room in rooms {
            for key in KEYS.map(|part| format!("man_{part}_{room}")) {
                assert!(
                    prose.has(&key),
                    "`help` in the {room} explains nothing: prose.toml has no `{key}`",
                );
            }
        }
    }

    #[test]
    fn a_primer_only_names_words_that_room_actually_offers() {
        // The half that rots. A primer names verbs, and a verb moving room — or
        // being renamed, which `tests/naming.rs` exists because it happens —
        // turns an instruction into a dead end where the player did the right
        // thing by asking. Checked against `offered`, the filter the listing
        // below the primer uses, so one screen's two halves cannot disagree.
        //
        // Availability, not success: `scribe` is offered in the grimoire and
        // refuses — *"go where the work is first"* — so that primer's first
        // draft sent the player at the one thing the room cannot do and this
        // passed. Looking found it (§15), hence the See-it line running `help`
        // in every room. What it does hold is the later, silent failure: a verb
        // renamed or moved. It did catch `bind`, gated at concentration 0.
        let mut sim = Sim::new(1);
        for room in ["laboratory", "archive", "lens", "grimoire"] {
            run(&mut sim, &format!("attend {room}"));
            let here: Vec<&str> = super::super::offered(sim.world())
                .into_iter()
                .map(Verb::canonical)
                .collect();

            // Both instruction halves. `here` names no verbs and is checked
            // anyway — a description that named one makes the same promise.
            for key in KEYS.map(|part| format!("man_{part}_{room}")) {
                let said = sim.world().resource::<Prose>().line(&key, &[]);
                for word in said.split(|glyph: char| !glyph.is_ascii_lowercase()) {
                    // Only words that *are* verbs somewhere are candidates; the rest
                    // of the sentence is English and is not this test's business.
                    if !Verb::ALL.iter().any(|verb| verb.canonical() == word) {
                        continue;
                    }
                    assert!(
                        here.contains(&word),
                        "`{key}` tells the player to type `{word}`, which the {room} \
                         does not offer",
                    );
                }
            }
        }
    }

    /// The apprentice's worked example must be lines the room can run.
    ///
    /// A tutorial is the worst place to print a dead end, and these are not
    /// sentences mentioning verbs — they are commands the player is about to
    /// type. The stronger version of
    /// `a_primer_only_names_words_that_room_actually_offers`.
    ///
    /// Two questions, because resolving is not enough: `grind sage` in the lens
    /// resolves and is then refused by `Verb::anchor`, so a lint that only
    /// parsed would pass the laboratory's example printed in the lens. The line
    /// must resolve through the real parser in the real room — catching a
    /// renamed argument — *and* its verb must be one that room offers, which is
    /// the filter the listing under `help` uses. Availability rather than
    /// success, for the primer lint's reason: whether a line then *works*
    /// depends on the tower's state at that moment.
    #[test]
    fn the_apprentice_only_shows_lines_the_room_can_run() {
        use crate::parser::{Mode, Resolution, analyse};

        let mut sim = Sim::new(1);
        for room in ["laboratory", "archive", "lens", "sanctum", "menagerie"] {
            run(&mut sim, &format!("attend {room}"));
            let cwd = sim.world().resource::<tower::Cwd>().0;
            let scene = crate::tower::scene_at(sim.world(), cwd);
            let here = super::super::offered(sim.world());
            for part in ["line", "then"] {
                let key = format!("craft_{part}_{room}");
                let prose = sim.world().resource::<Prose>();
                assert!(
                    prose.has(&key),
                    "the {room} has no `{key}`, so the apprentice borrows another \
                     room's lines in a room that has its own",
                );
                let said = prose.line(&key, &[]);
                let Resolution::Resolved { intent, .. } =
                    analyse(&said, &scene, Mode::Calm).resolution
                else {
                    panic!("`{key}` shows `{said}`, which the {room} cannot read");
                };
                assert!(
                    here.contains(&intent.verb),
                    "`{key}` shows `{said}`, and the {room} does not offer \
                     `{}`",
                    intent.verb.canonical(),
                );
            }
        }
    }

    #[test]
    fn the_overview_lists_what_resolves_here_and_nothing_else() {
        // Both directions, in two rooms. The listing is `execute::offered`, the
        // boot report's filter — so this also stops the launch tutorial
        // disagreeing with the manual a minute later.
        for (place, wanted, unwanted) in [
            ("laboratory", "grind", "kindle-nothing"),
            ("archive", "research", "grind"),
        ] {
            let mut sim = Sim::new(1);
            run(&mut sim, &format!("attend {place}"));
            let before = sim.scrollback().records().len();
            run(&mut sim, "recall");
            let after: Vec<_> = drawn(&sim).split_off(before);

            let listed: Vec<&str> = after
                .iter()
                .filter(|(kind, _)| *kind == RecordKind::Entry)
                .filter_map(|(_, line)| line.split_whitespace().next())
                .collect();
            assert!(listed.contains(&wanted), "{place}: no {wanted}: {listed:?}");
            assert!(
                !listed.contains(&unwanted),
                "{place}: offered {unwanted}, which would not resolve here",
            );
            // Nothing gated, nothing dark: `bind` refuses at concentration 0 and
            // `undo` is not built, and either is a dead end in the first thing a
            // lost player reads.
            assert!(!listed.contains(&"bind"), "{place}: offered a gated verb");
            assert!(!listed.contains(&"undo"), "{place}: offered a dark verb");
        }
    }

    #[test]
    fn every_word_for_the_manual_reaches_the_same_answer() {
        // `help`, `man` and `?` are synonyms of `recall` (§6.1's registers), and
        // the overview is the answer a player gets from all four spellings.
        let mut baseline = Sim::new(1);
        run(&mut baseline, "recall");
        let expected = drawn(&baseline);

        for word in ["help", "man", "?"] {
            let mut sim = Sim::new(1);
            run(&mut sim, word);
            assert_eq!(
                drawn(&sim).len(),
                expected.len(),
                "`{word}` answered differently from `recall`",
            );
        }
    }

    #[test]
    fn every_group_the_table_names_has_a_heading_authored() {
        // A group with no prose draws its own key — `Prose::line` returns the
        // key for a miss, so the failure is visible. This makes it a test one.
        let prose = crate::content::Prose::builtin();
        for group in Group::ALL {
            assert!(
                prose.has(group.key()),
                "{} has no authored heading",
                group.key(),
            );
        }
        assert!(prose.has("man_elsewhere"));
    }

    #[test]
    fn the_manual_registers_no_subjects_of_its_own() {
        // `Prose::topics` strips `recall_` to decide what is *nameable*, which is
        // how `step_or` once became a subject nobody authored (§19). The manual's
        // own furniture is under `man_`, so none of it may appear.
        let prose = crate::content::Prose::builtin();
        let leaked: Vec<&str> = prose
            .topics()
            .into_iter()
            .filter(|topic| topic.starts_with("group_") || *topic == "elsewhere")
            .collect();
        assert!(leaked.is_empty(), "the manual leaked subjects: {leaked:?}");
    }

    #[test]
    fn a_page_a_verb_has_is_the_page_it_gets() {
        // Only three are written so far, so this is not the completeness lint —
        // it is the shape. A page names itself, says what it is for, and reads
        // its synonyms off the table rather than out of prose.
        let mut sim = Sim::new(1);
        run(&mut sim, "attend archive");
        let before = sim.scrollback().records().len();
        run(&mut sim, "recall grind");
        let lines: Vec<String> = drawn(&sim)
            .split_off(before)
            .into_iter()
            .map(|(_, l)| l)
            .collect();
        let page = lines.join("\n");

        // Readable from the archive, where `grind` itself does not resolve. A
        // manual you can only read in the right room has a lock on it.
        assert!(page.contains("grind <reagent>"), "no synopsis: {page}");
        assert!(page.contains("crush a reagent"), "no gloss: {page}");
        // Off `SYNONYMS`, so it cannot go stale when a word is added.
        assert!(page.contains("arcane: grind"), "no registers: {page}");
        assert!(page.contains("plain: crush"), "no plain form: {page}");
    }

    #[test]
    fn a_synopsis_names_every_slot_its_signature_requires() {
        // The authored line, kept honest. `signature()` carries no connectives —
        // `move` is `[Reagent, Place?, Place]` with no `to` — so a generated
        // synopsis reads `move reagent place place`, which nobody types. This is
        // what stops the written one drifting from the signature it describes.
        let prose = crate::content::Prose::builtin();
        for verb in Verb::ALL {
            let key = format!("man_{}_use", verb.canonical());
            let line = prose.line(&key, &[]);
            assert!(
                line.starts_with(verb.canonical()),
                "{key} does not open with the word it documents: {line:?}",
            );
            // Exempt, and the exemption is a recorded debt. These take `Place`
            // slots because the place half of a spell's condition resolves
            // against that kind, which is why the archive's four ways and the
            // lens's sockets and sigils are places you cannot stand in (§19).
            // `follow <place>` would be honest about the implementation and
            // wrong for a player, who is choosing a direction.
            //
            // Still an assertion rather than a skip: each names the word the
            // player is choosing, so a synopsis cannot drift into saying
            // nothing. It goes when `Role::Reading` fixtures stop having to be
            // `Place`s.
            if let Some(instead) = match verb {
                Verb::Follow => Some(&["way"][..]),
                Verb::Dial => Some(&["socket", "sigil"][..]),
                // ...and the sanctum's stations. `haul <station> to <station>`
                // rather than `<from> <to>`: the direction is what a player must
                // get right, and `to` is §6 filler so the line also works typed.
                Verb::Haul => Some(&["station"][..]),
                // ...and the bailey's die and area, fourth time. `pledge <place>
                // <place>` is true of the type and useless: they are choosing
                // *which die* and *which part of the wall*, and swapping the two
                // is the verb's commonest mistake.
                Verb::Pledge => Some(&["die", "area"][..]),
                // ...and the menagerie's glyph and humour, third time: one of
                // three glyphs and one of six gates, and neither is somewhere to
                // stand.
                Verb::Limn => Some(&["glyph", "humour"][..]),
                // ...and the forge's tool and charm, fifth time: any tool in the
                // tower and one of five charms, and "place" says nothing about
                // which way round they go.
                Verb::Imbue => Some(&["tool", "charm"][..]),
                // ...and its columns, sixth. A column is a thing you snap, not
                // somewhere to stand.
                Verb::Snap => Some(&["column"][..]),
                _ => None,
            } {
                for wanted in instead {
                    assert!(line.contains(wanted), "{key} lost its {wanted}: {line:?}",);
                }
                continue;
            }
            for slot in verb.signature().iter().filter(|slot| slot.required) {
                assert!(
                    line.contains(slot.kind.label()),
                    "{key} never names its required {}: {line:?}",
                    slot.kind.label(),
                );
            }
        }
    }

    #[test]
    fn every_word_a_spell_is_written_with_has_a_page() {
        // The gap an audit against seven other languages found: every one ships
        // a reference and ours had none. Control words are outside `Verb::ALL`,
        // so the verb lint below could not cover them and `recall repeat`
        // reached nothing at all.
        //
        // The readings are the same hole one noun space over: `NounKind::Sense`
        // rather than materials, so `every_material_has_a_page` misses them too
        // and `recall marks` answered with a *scoping* message about other rooms
        // — a dead end wearing a wrong reason.
        let prose = crate::content::Prose::builtin();
        for word in crate::parser::SpellWord::ALL {
            assert!(
                prose.has(&format!("recall_{}", word.canonical())),
                "`{}` is a word a spell is written with and the manual cannot \
                 say what it does",
                word.canonical(),
            );
        }
        // Every domain's readings, not just the archive's. This asked
        // `maze::readings()` alone and three rooms went uncovered, so the
        // menagerie shipped a phase where `recall next` fell through to the room
        // overview; it then listed four and the bailey went uncovered.
        //
        // Listed rather than derived: a domain's readings are `Vec<&str>` on its
        // own module with no registry, so the options are a list here or a
        // registry nothing else wants. The hand-written list is the weakness — a
        // domain not added here ships a vocabulary the manual cannot reach, with
        // this test green.
        let readings = crate::tower::maze::readings()
            .into_iter()
            .chain(crate::tower::circle::readings())
            .chain(crate::tower::ward::readings())
            .chain(crate::tower::pylon::readings())
            .chain(crate::tower::siege::readings());
        for reading in readings {
            assert!(
                prose.has(&format!("recall_{reading}")),
                "`{reading}` is a word a solver's `if` names and the manual \
                 cannot say what it means",
            );
        }
        // And the page that ties them to a room.
        assert!(prose.has("recall_scripting"));
    }

    #[test]
    fn every_material_has_a_page() {
        // The same completeness lint, one noun space over. Materials had none,
        // so a reagent could be authored with a colour, a recipe and a route
        // and never a word saying what it *was* — §19 records that failing
        // twice, and both times a person found it rather than a test.
        //
        // One key required, not three: a material is a thing, and most want a
        // sentence rather than a page. `using_` is optional, because a byproduct
        // is something you have rather than something you do.
        let prose = crate::content::Prose::builtin();
        for material in crate::content::Materials::builtin().names() {
            assert!(
                prose.has(&format!("recall_{material}")),
                "`{material}` is authored in materials.toml and the manual cannot \
                 say what it is",
            );
        }
    }

    #[test]
    fn a_finished_product_says_what_it_is_for() {
        // The pages this change was for. A potion and a scroll are what a player
        // *holds*, and a route told them five steps and nothing about the thing
        // in their hand. Each has a `using_` line now — which for a potion says
        // the drinking is not built, on `undo`'s precedent.
        let prose = crate::content::Prose::builtin();
        let recipes = crate::content::Recipes::builtin();
        let mut checked = 0;
        for made in recipes.outputs() {
            if !matches!(
                recipes.kind_of(made),
                crate::parser::NounKind::Essence | crate::parser::NounKind::Scroll
            ) {
                continue;
            }
            checked += 1;
            assert!(
                prose.has(&format!("using_{made}")),
                "`{made}` is finished work and its page does not say how it is used",
            );
        }
        assert!(checked > 0, "no finished product was actually checked");
    }

    #[test]
    fn every_verb_has_a_page() {
        // The completeness lint, and what stops the manual rotting as verbs are
        // added: a new verb is already a compile error in `Verb::group`, and
        // this makes it a prose failure too.
        //
        // Three keys required and the rest optional, so a page's length follows
        // what there is to say. `_use` and `_gloss` are what the overview and
        // the header need; `_1` is the shortest honest *what does it do*.
        let prose = crate::content::Prose::builtin();
        for verb in Verb::ALL {
            let canonical = verb.canonical();
            for suffix in ["use", "gloss", "1"] {
                let key = format!("man_{canonical}_{suffix}");
                assert!(prose.has(&key), "{canonical} has no {key}");
            }
        }
    }

    #[test]
    fn a_page_never_claims_a_word_that_is_not_there() {
        // Every `see also` names real vocabulary. Pointing at a word the parser
        // does not have is worse than pointing nowhere: the player types it and
        // lands in the dead end §15 weighs heaviest.
        let prose = crate::content::Prose::builtin();
        let known: Vec<&str> = Verb::ALL.iter().map(|verb| verb.canonical()).collect();
        for verb in Verb::ALL {
            let key = format!("man_{}_also", verb.canonical());
            if !prose.has(&key) {
                continue;
            }
            for word in prose.line(&key, &[]).split(',') {
                // `recall <topic>` entries point at a subject, not a verb; the
                // first word is what has to exist.
                let Some(first) = word.split_whitespace().next() else {
                    continue;
                };
                assert!(
                    known.contains(&first),
                    "{key} points at {first:?}, which is not a word",
                );
            }
        }
    }

    #[test]
    fn the_careful_group_is_exactly_the_destructive_verbs() {
        // `is_destructive` had no reader until the manual wanted one. Two lists
        // that must not drift, so each is checked against the other.
        for verb in Verb::ALL {
            assert_eq!(
                verb.group() == Group::Careful,
                verb.is_destructive(),
                "{} is grouped and marked differently",
                verb.canonical(),
            );
        }
    }

    #[test]
    fn no_verb_is_left_out_of_the_overview() {
        // `Verb::group` has no wildcard, so a new verb is a compile error — but
        // a verb in a group nobody prints compiles and is simply not there.
        for verb in Verb::ALL {
            assert!(
                Group::ALL.contains(&verb.group()),
                "{} is in a group the overview never prints",
                verb.canonical(),
            );
        }
    }
}
