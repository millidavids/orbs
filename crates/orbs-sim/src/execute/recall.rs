//! The in-world manual (§6.1): how a thing is made, in the order you make it.
//!
//! Not part of §10.1's loop — it is what you read **before** committing an
//! instrument to a route, which is the half of the exit criterion that says the
//! same goal having two right answers is only a decision if the player can see
//! both.
//!
//! It is also where the retired brewing words land (§19). `make a potion of
//! clarity` resolves here, and for a while it resolved here and *acknowledged* —
//! a dead end, which §15 weighs above the raw resolution rate. This is what makes
//! the newcomer's most natural sentence the tutorial entry point rather than a
//! shrug.
//!
//! # It reads as instructions, not as a table
//!
//! The first version emitted the raw breadth-first walk as five bare columns per
//! row, tiled across the pane. Four things were wrong with it and they compounded:
//!
//! - **Backwards.** The walk starts at the goal, so the first line was the *last*
//!   thing to do. A player reading top-down got the recipe in reverse.
//! - **Unlabelled.** `clarity alembic clarified-draught phlegm 14` gives the
//!   reader five values and no way to tell an input from an output.
//! - **Clipped.** A two-input step does not fit ~60 cells, and tiling cut it.
//! - **Full of noise.** It expanded *every* route to everything, including three
//!   ways to make the `rock-salt` sitting in the dispensary.
//!
//! So: the primary route in **dependency order**, numbered, one authored sentence
//! per step (rule 6), stopping at what the player already has. Alternatives
//! follow, marked, with any step only they need. The fields stay on the record
//! (rule 4) so `sift` and a pipe still work on them.

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
/// The content's longest chain is four, and the `seen` set already stops a cycle
/// — this is the backstop for a content file that grows one, so a malformed
/// recipe table cannot recurse until the stack goes.
const MAX_DEPTH: usize = 16;

/// One command, at length.
///
/// # Sections, and what each is made of
///
/// The synopsis and description are authored; the ways of saying it are read off
/// `SYNONYMS`. That split is deliberate: `signature()` carries no connectives —
/// `move` is `[Reagent, Place?, Place]` with no `to` in it — so a generated
/// synopsis would read `move reagent place place`, which nobody types. A test
/// keeps the authored line honest instead, by asserting it names every required
/// slot.
///
/// `RecordKind::Message` throughout, not `Entry`: a page is instructions, and
/// `Message` wraps where `Entry` tiles. `recall.rs`'s route walk already makes
/// that argument. Long pages page for free, because `unfurl` searches by record.
fn page(world: &mut World, verb: Verb) {
    let canonical = verb.canonical();

    // The synopsis, and the one line saying what it is for. Both authored; a
    // missing key draws as the key itself, which is how `Prose::line` makes an
    // omission visible rather than blank.
    for key in [
        format!("man_{canonical}_use"),
        format!("man_{canonical}_gloss"),
    ] {
        say(world, canonical, &key);
    }

    // **Joined into one record, not one per authored line.** Prose is capped at
    // 70 cells by the width lint, so a description has to be written in pieces —
    // but emitting those pieces as separate records makes them *hard* line
    // breaks, and a page in a 100-column pane came out as ragged 50-cell strips
    // with a wall of text where its paragraphs should be. One record wraps to
    // whatever the pane actually is.
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

    // **One row per register, not one per phrase.** `attend` has five spellings
    // and four of them are plain, so a row each made the section longer than the
    // description it followed.
    //
    // Read off the table, so it cannot go stale. Arcane first, because that is
    // what the echo teaches and what an expert types.
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
        // **The arrangement is authored**, like every route step: a `Message`
        // draws its message and nothing else, so composing `"arcane  grind"` in
        // Rust would be a frontend decision made in the sim (rule 4) *and* a
        // sentence in source (rule 6). The fields stay fields, so `sift` still
        // works on them.
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
/// A page is read in a transcript pane, not a browser. Eight is already more
/// than fits at the 80x22 floor without scrolling, so the cap is a backstop
/// against an authored run with no end rather than a budget anyone reaches.
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
/// **Not `primer`**, which was the first choice and is already taken *in this
/// module* — [`primer`] is the room's three-line introduction, printed by
/// `help`. One word for two pages in one file is how the next reader merges
/// them, which is the argument `Strand` records against `Cursor`.
///
/// `apprentice` is §12's own word for this — *"diegetic apprenticeship: the orb
/// teaches as a mentor"* — and it reads as the request a player is making rather
/// than as a label. Swept on both axes: clean on similarity, and `app` is a free
/// prefix. `spellcraft` scores 925 against `spell` and shares `spe` with it;
/// `crafting` and `writing` both score 667 against `scripting`, which is the one
/// page it must not be confused with.
const APPRENTICE: &str = "apprentice";

/// The name the primer's worked example gives its spell.
///
/// A word rather than a room's, because the example is followed by typing it: a
/// player who copies the page ends up with `morning.spell`, and a name that
/// changed with the room would make the `invoke` line below wrong the moment
/// they walked next door.
const EXAMPLE: &str = "morning";

/// The room the example falls back to where there is no work to script.
const FALLBACK: &str = "laboratory";

/// How to make a spell, from nothing, in the order it happens.
///
/// # A tutorial, where [`scripting`] is the reference
///
/// That page answers *what may I write* — the words, the question shapes, what
/// this room can name. It is the right page to have open while writing and it
/// teaches nobody how to start, because **no listing of words teaches an
/// order**. `scribe`, `edit`, the lines, `<escape>`, `quit`, `invoke`, and then
/// the log rather than the pane: seven steps, one of which (`quit` being the
/// save) is a thing a player will otherwise discover by losing work.
///
/// §12 calls this the *in-world grimoire* and puts it in the "always" column,
/// beside the apprenticeship that Phase 10 owns. This is the reference half of
/// that, which is why it is a `recall` page and not a scripted sequence.
///
/// # The worked example comes from the room
///
/// [`scripting`]'s third section arriving at the same conclusion: the shape of a
/// spell is the same everywhere and the lines in one are not. Showing `grind
/// sage` to somebody standing in the lens teaches them a room they are not in,
/// and the fix is the one this file already uses twice — build the moving part
/// from where the player is.
///
/// A room with no work to script says whose lines it is borrowing. That is
/// better than silence and better than nothing: the grimoire's own primer
/// already tells you to go where the work is, and this agrees with it.
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
/// # Three sections, and only the last one moves
///
/// The **words** and the **question shapes** are the same in every room, because
/// the grammar is. What changes is what a question can *name*: the laboratory's
/// instruments answer `is idle`, the archive's four ways answer `has passage`,
/// and a player standing in one has no way to discover the other's vocabulary by
/// guessing at it.
///
/// So the third section is built from the room, the same way [`overview`] builds
/// its verb list from [`offered`](super::offered) — one rule, applied twice,
/// rather than a second idea of what *here* means.
fn scripting(world: &mut World) {
    let cwd = world.resource::<tower::Cwd>().0;
    let places: Vec<String> = tower::children_of(world, cwd)
        .into_iter()
        .filter(|node| world.get::<tower::Fixture>(*node).is_some())
        .filter_map(|node| world.get::<tower::Name>(node).map(|name| name.0.clone()))
        .collect();
    // The readings belong to a **set** — a way, a socket — so they are listed
    // where that set is. They resolve everywhere (a solver's `if` names them at
    // cast, when no maze and no ward is open), but naming them in the laboratory
    // would be teaching a word the room can never answer.
    //
    // **This asked `Reading` and then printed the maze's list regardless.** The
    // lens's sockets and sigils carry that marker, so the gate opened in the
    // lens and the page taught `passage wall exit back spoil marks gleaning`
    // while naming none of the six deltas that are the whole of what a lens
    // spell may ask. See [`readings_at`](tower::readings_at).
    let readings = tower::readings_at(world, cwd);

    say(world, SCRIPTING, "recall_scripting");

    section(world, "man_scripting_words");
    // **`SpellWord::shape`, not a copy of it.** This was a byte-identical second
    // table, and the extraction that took the first copy out of `orbs-shell`
    // walked straight past it — so adding `part` meant editing both, and the
    // next word added to the language would have updated one and left this page
    // printing a stale shape. That is the exact failure the extraction was for.
    for word in crate::parser::SpellWord::ALL {
        entry(world, word.canonical(), word.shape(), None);
    }

    section(world, "man_scripting_asking");
    for key in SHAPES {
        let line = world.resource::<Prose>().line(key, &[]);
        entry(world, &line, "", None);
    }

    // **The sets, before the names**, because `for each <set>` is unusable
    // without them and there is nowhere else to find one out: a set is not a
    // place you can `survey` and not a word the scene offers, so a player who
    // has read the word `for` still cannot write a line with it.
    //
    // Derived from the room rather than listed, so a domain that declares a
    // group gets it here the same tick — the same rule `here` below follows.
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
/// `than` sits after `count` because it is the same comparison with the world
/// on both sides — a player who has not met a number in a question yet has no
/// use for one.
const SHAPES: [&str; 5] = [
    "man_scripting_shape_is",
    "man_scripting_shape_has",
    "man_scripting_shape_count",
    "man_scripting_shape_than",
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
    // **What it does, when the row has earned one.** Carried as a field rather
    // than composed into the name: the view decides whether a run draws as a
    // described column or an index, `sift` matches the sentence, and a reader
    // hears it labelled. Absent on a tower-wide verb, which is what keeps that
    // run tiling.
    if let Some(gloss) = describing {
        row = row.text(FieldName::Detail, gloss);
    }
    row.finish();
}

/// The three halves of a room's primer, in the order they are printed.
///
/// Named here rather than spelled out at each site so the prose keys, the emitter
/// and `every_room_a_player_can_stand_in_explains_itself` cannot come to disagree
/// about how many there are — which is how a fourth would get authored and never
/// drawn.
const KEYS: [&str; 3] = ["here", "start", "solve"];

/// What the room you are standing in is, and how its puzzle is worked.
///
/// # Three keys, and they are the three questions
///
/// `man_here_<room>` says what the place *is*, `man_start_<room>` how to begin its
/// puzzle, `man_solve_<room>` how to finish one. What-it-is before how-it-works is
/// the same order [`describe`] uses for a thing, because it is the same split.
///
/// Three rather than two because `every_authored_line_fits_the_worst_case_width`
/// gives 70 cells against §4's 80×22 floor and one first-draft sentence ran to 112.
/// That is not a workaround: start and solve are two instructions, followed at
/// different times.
///
/// # `man_`, not `recall_`
///
/// [`Prose::topics`](crate::content::Prose::topics) decides what is *nameable* by
/// stripping `recall_`, so `recall_here_lens` would register `here_lens` as a
/// subject to ask the orb about — the trap this module already records paying for
/// twice, at `grimoire_step_or` and at `clarity_use`. `man_` is the prefix the
/// manual's own furniture already uses (`man_elsewhere`, `man_group_work`) and is
/// not a noun space.
///
/// # Absent is allowed
///
/// Guarded on [`Prose::has`], so an unbuilt room simply has no primer and the
/// listing prints as it always did. `every_room_a_player_can_stand_in_explains
/// _itself` is what stops that being a silent omission for a room that *is* built.
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

    // The room's own name as the heading, which is why this one is not a `Prose`
    // key like the group headings below: it *is* the place, and authoring seven
    // files' worth of `man_section_<room>` saying the room's name back would be
    // furniture with a translation cost.
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Section)
        .text(FieldName::Kind, &room)
        .finish();
    for message in lines {
        line(world, Verb::Recall.canonical(), &message);
    }
}

/// The room the player is standing in, for the manual's own purposes.
///
/// [`domain_of`](crate::tower::domain_of) returns `None` at `/tower` and at
/// `/grimoire` because neither is somewhere work happens — but both are places a
/// player can stand and ask for help, and the grimoire has a box on the tower rail
/// like any other. So this falls back to the node itself rather than to nothing.
fn here(world: &World) -> String {
    let cwd = world.resource::<crate::tower::Cwd>().0;
    let node = crate::tower::domain_of(world, cwd).unwrap_or(cwd);
    crate::tower::where_at(world, node)
}

/// Everything you can type where you are standing, grouped.
///
/// # Why this is `Section` + `Entry` and not a formatted string
///
/// `RecordKind::Section` stacks and draws as a `[heading]`; `RecordKind::Entry`
/// **tiles**, packing across whatever width the pane has. `survey` already emits
/// exactly this pair, so a clap-shaped listing needed no render code at all —
/// and because the entries are records rather than a string, `sift` still works
/// on them and §14 hears one utterance per verb rather than a wall of spacing.
///
/// # It lists what works *here*
///
/// [`offered`](super::offered) is the boot report's own filter: live, ungated,
/// and in scope. `grind` appears in the laboratory and not in the archive, which
/// is §7 and is the rule Tab already follows — offering a word the parser would
/// refuse is the dead end §15 weighs above the raw resolution rate.
///
/// So the listing is *narrower* than the manual: `recall grind` will answer from
/// anywhere, because a manual you can only read in the right room has a lock on
/// it. That asymmetry is deliberate and recorded in §19.
fn overview(world: &mut World) {
    // **The room first, and the vocabulary second.** A listing of twenty-five
    // words answers *what may I type* and never *what is this place for* — and a
    // player who types `help` in the lens is not usually asking for a word list,
    // they are asking what a ward is and how to open one. §6.1 makes `recall` the
    // in-world manual, and a manual that opens with an index is a reference rather
    // than an explanation.
    primer(world);

    let offered = super::offered(world);

    for group in Group::ALL {
        let members: Vec<Verb> = offered
            .iter()
            .copied()
            .filter(|verb| verb.group() == group)
            .collect();
        // A heading over nothing is furniture. The archive has no spells in it
        // and no destructive verb the room has earned, and printing empty
        // sections would make the overview mostly headings.
        if members.is_empty() {
            continue;
        }

        section(world, group.key());
        // Through the shared [`entry`], which carries the empty-field rule —
        // `status` and `undo` take no argument, and an empty `Kind` draws as
        // trailing blanks and speaks as a labelled silence. It was written out
        // here *and* in `entry`, which is two homes for one rule.
        // **This room's own words are described; the tower-wide ones are
        // indexed.** `grind`, `digest` and `distil` are opaque and a player
        // meets them for the first time in the room that offers them;
        // `status` and `quit` are neither, and they appear under the same
        // heading in every room in the game. So the local ones earn a
        // sentence and the global ones earn a column.
        //
        // The saving is the point rather than a side effect: described
        // throughout, this page is ~35 rows against a floor that fits 19.
        // `recall <verb>` still has the full page for anything here.
        //
        // `anchor` is the existing question *which fixture must stand here* —
        // the same one `Scene::offers` asks to decide what a room offers at
        // all — so a new domain's verbs describe themselves with no list to
        // maintain.
        // **Decided per section, not per verb**, and that is the fix for a real
        // defect rather than a preference. `move` and `wield` sit under *the
        // work* and are tower-wide, so describing only the anchored verbs left
        // the section half described — and a half-described run cannot draw as
        // either shape. It fell back to stacking with the sentence jammed on
        // unpadded, which is worse than what it replaced.
        //
        // A section is the unit a reader sees, so a section is the unit that
        // decides. §3 is the deeper reason: a run where some rows have a second
        // column and some do not is ragged, and raggedness is the vocabulary
        // sabotage owns.
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

    // The two pointers out, and they answer different questions. **The first is
    // the one a lost player needs**: everything above is a word to type *now*,
    // and nothing on the page says the orb can be taught to type them for you —
    // so a player could read `help` in every room and never learn the game has
    // spells in it. §12 puts the in-world grimoire in the "always" column, and a
    // reference nobody can find their way into is not one.
    //
    // The second says *this is the vocabulary here* rather than *this is the
    // vocabulary*, without which somebody standing in the archive would never
    // learn the laboratory has words of its own.
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
        // **The overview, and this branch was unreachable until the slot became
        // optional.** A required slot with fillers never yields an argument-less
        // intent — every filler ties and `analyse` returns `Ambiguous` — so bare
        // `recall` opened a numbered prompt offering the four
        // alphabetically-first subjects. `help`, `man` and `?` all land here, so
        // that was §6's no-bare-error rule failing at the one command whose job
        // is answering the question. See `TOPIC_OPTIONAL`.
        overview(world);
        return;
    };

    // **The manual's own pages, before the recipe walk.** A verb canonical is a
    // `NounKind::Command` and nothing else, so this cannot shadow a recipe — but
    // it is checked first anyway, because the day a recipe is named after a verb
    // the page is what the player meant.
    // **Asked of the whole word list, not of the canonicals.** `walk` is
    // `follow`, `edit` is `scribe`, `make` is `recall` — and a player asks with
    // the word they typed. Before this those three reached no page at all and
    // fuzzed into the archive's readings instead, so `recall edit` explained the
    // maze's way out to somebody asking about the spell editor (§19).
    if let Some(verb) = crate::parser::verb_of_word(&topic) {
        page(world, verb);
        return;
    }

    // **The one page about the language rather than about the tower**, and it is
    // scoped like the overview above: the grammar is the same everywhere, what a
    // question can *name* is not. Before this, nothing in the game taught the
    // spell vocabulary at all — control words are outside `Verb::ALL`, so
    // `recall repeat` reached nothing and a player had no way to find out what an
    // `if` could ask.
    if topic == SCRIPTING {
        scripting(world);
        return;
    }

    // **The tutorial beside the reference**, and the split is the whole reason
    // it is a second page: `scripting` answers *what may I write* and cannot
    // answer *how do I start*, because no listing of words teaches an order.
    if topic == APPRENTICE {
        apprentice(world);
        return;
    }

    // **What it is, before how it is made.** A route answers *how do I get one*;
    // a player holding a potion or a scroll is asking *what is this for*, and the
    // two are different questions. This used to be an `else`: anything with a
    // recipe got the walk and nothing else, so every finished product in the game
    // could tell you its five steps and not one word about what it did.
    let said = describe(world, &topic);

    let plan = plan(world, &topic);
    if plan.is_empty() {
        // Not a recipe either. §6.1 makes `recall` the **in-world manual**, so a
        // subject like `brewing` is answered with authored prose (rule 6) rather
        // than treated as a thing that does not exist.
        if !said {
            missing(Verb::Recall, &topic, world);
        }
        return;
    }

    say_plan(world, &topic, &plan);
}

/// Say what `topic` is, and how it is used. Whether anything was said.
///
/// Two keys, and the second is deliberately **not** `recall_<topic>_use`:
/// [`Prose::topics`](crate::content::Prose::topics) decides what is *nameable* by
/// stripping `recall_`, so that spelling would register `clarity_use` as a
/// subject to ask the orb about. It is the trap the route templates already
/// record paying for, where `grimoire_step_or` made `step_or` a topic.
///
/// `using_` is optional and often absent — a byproduct is a thing you have
/// rather than a thing you do — and where it *is* present it may honestly say
/// the use is not built. `undo`'s page does the same, and §15 wants that: a page
/// admitting a word does nothing is the cheapest way to keep a player out of a
/// dead end.
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
/// `RecordKind::Message`, not `Entry`. `Entry` speaks as a `TableRow` — which is
/// right for `survey`, where §14 needs the column labels — and it **tiles**,
/// packing rows across the pane. Both are wrong for an instruction: the sentence
/// *is* the content, it has to wrap rather than be cut, and a reader wants the
/// line rather than five `label: value` pairs. A `Message` carrying fields keeps
/// `sift` working on them (rule 4) while drawing and speaking as prose.
fn say_step(world: &mut World, step: &Step, index: Option<usize>) {
    // Joined once and handed on — `step_line` needs the same string for its
    // sentence, and building it twice was two allocations per step of every
    // `recall` page and every line of the lens's spill.
    let inputs = step.inputs.join(" + ");
    let message = step_line(world, step, index, &inputs);
    // The facts ride along for `sift` and a pipe, under the same names the
    // pipeline's own records use: the instrument is a `Path`, what comes out is
    // the `Name`, what went in is the `Origin`.
    //
    // **Never `Detail`.** A record carrying prose draws its `Message` *and* its
    // `Detail`, by design — `Detail` is secondary prose subordinate to the
    // message — so putting the inputs there printed them in front of the sentence
    // written to explain them. That is the trap `wield`'s refusal fell into, and
    // `Origin` (where a thing came *from*) is what an input actually is.
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
/// **Split out of [`say_step`] so the lens can steal it.** A broken ward spills
/// the far wizard's working into `lens.log`, and it has to read exactly as
/// `recall` reads — a second renderer would be two ways of describing one recipe,
/// which is the class of defect §19 records most often. What differs between the
/// two callers is only the record it lands in.
fn step_line(world: &World, step: &Step, index: Option<usize>, inputs: &str) -> String {
    let prose = world.resource::<Prose>();
    let heat = if step.heat {
        prose.line("route_heat", &[])
    } else {
        String::new()
    };
    // Composed in like the heat clause beside it, and for the same reason: not
    // every step has one. The `+` lives in `route_leaves` rather than in the
    // step line, so a step that leaves nothing does not print a plus with
    // nothing after it.
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
/// The lens's spill, and nothing else. It walks the same [`plan`] the manual
/// does, so a stolen recipe and a read one cannot describe the same thing two
/// ways.
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

    // **The goal itself is never treated as stock.** `recall rock-salt` should
    // answer with how to make it even though the dispensary holds some, which is
    // what the `depth > 0` guard inside `build` is for.
    build(recipes, &stocked, goal, 0, &mut plan, &mut seen, true);
    plan
}

/// Push the steps that make `output`, deepest first.
///
/// **Post-order**, which is what puts the walk in *doing* order: a step's inputs
/// are emitted before the step that consumes them, so reading top-down is the
/// order a player types. The breadth-first walk this replaced emitted the goal's
/// own step first, which is the last thing you do.
///
/// `primary` says whether these steps belong to the route being recommended or
/// to an alternative — an alternative's own sub-steps are only needed if you take
/// it, so they go with it rather than into the numbered list.
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
    // Stop at what the player already has. Without this the walk expanded three
    // ways to make the `rock-salt` sitting in the dispensary, which is true and
    // useless — the manual is for what you cannot simply pick up.
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

    // Every other way to reach the same thing, with whatever only it needs.
    // **File order decides which is primary**: `recipes.toml` is authored, and
    // its first entry for an output is the designer's recommendation.
    for (instrument, recipe) in routes.iter().skip(1) {
        for input in recipe.inputs() {
            build(recipes, stocked, input, depth + 1, plan, seen, false);
        }
        plan.alternates.push(Step::of(output, instrument, recipe));
    }
}

/// Everything a store in the tower holds.
///
/// Walked from the root rather than taken from a global query: `tower::node`
/// requires anything a player can see be derived by walking `Children`, because
/// archetype order is not insertion order. The result here is an order-free set,
/// but the walk keeps the rule unbroken rather than arguing the exception.
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
        // **The defect the manual starts from, and nothing caught it.** A
        // required slot with fillers never yields an argument-less intent, so
        // bare `recall` returned `Ambiguous` and offered the four
        // alphabetically-first subjects — and `help`, `man` and `?` all land
        // here. §6 forbids a bare error; asking a lost player to choose between
        // `archive`, `brewing`, `clarified-draught` and `clarity` is that rule
        // failing at the one command whose job is answering the question.
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
        // **What the player asked for, as an ordering.** A twenty-five word index
        // answers *what may I type* and never *what is this place for* — and the
        // first thing on screen is what a lost player reads.
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
        // **A new domain cannot ship without a primer.** §10 has five more rooms
        // coming and each will be built by someone who is not thinking about the
        // manual; the failure mode is silent, because `primer` is guarded on
        // `Prose::has` and an unauthored room simply prints the old word list.
        //
        // Driven from `Sim::briefs()` rather than a list written out here, so the
        // rooms come from the world. `built` is what makes it honest: `forge`
        // and `menagerie` are dark, need nothing yet, and start needing it on
        // the day `build.rs` raises them.
        let sim = Sim::new(1);
        let prose = sim.world().resource::<Prose>();

        // `arsenal` and `tower` are standable and are not rail domains, so they are
        // named — `briefs()` cannot see them. Both are checked by the same rule
        // rather than exempted from it.
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
        // **The half that rots.** A primer is a sentence naming verbs, and a verb
        // moving room — or being renamed, which `tests/naming.rs` exists because it
        // happens — turns an instruction into a dead end. §15 weighs the dead-end
        // rate above the raw resolution rate, and a manual is the worst place to
        // spend it: the player did the right thing by asking.
        //
        // Checked against `offered`, which is the same filter the listing below the
        // primer uses, so the two halves of one screen cannot disagree.
        //
        // **It catches availability, not success, and the grimoire is the proof.**
        // `scribe` is offered there and refuses — *"a spell is written for a place.
        // go where the work is first"* — so the first draft of that primer sent the
        // player at the one thing the room cannot do and this test passed. Looking
        // is what found it (§15), which is why the See-it line runs `help` in every
        // room rather than trusting the green. What this *does* hold is the failure
        // that arrives later and silently: a verb renamed or moved between rooms.
        //
        // It did catch `bind`, which is gated at concentration 0 — an instruction
        // the listing below correctly refuses to print.
        let mut sim = Sim::new(1);
        for room in ["laboratory", "archive", "lens", "grimoire"] {
            run(&mut sim, &format!("attend {room}"));
            let here: Vec<&str> = super::super::offered(sim.world())
                .into_iter()
                .map(Verb::canonical)
                .collect();

            // Both instruction halves. `here` is descriptive and names no verbs,
            // but it is checked too — a description that named one would be making
            // the same promise.
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

    /// **The apprentice's worked example must be lines the room can run.**
    ///
    /// A tutorial is the worst place in the game to print a dead end — the
    /// player did the right thing by asking, and they are about to *type* what
    /// they are shown. `a_primer_only_names_words_that_room_actually_offers`
    /// makes the same argument about a sentence that mentions a verb; this is
    /// the stronger version, because these are not sentences mentioning verbs.
    /// They are commands.
    ///
    /// **Two questions, because resolving is not enough.** `grind sage` typed in
    /// the lens *resolves* — `grind` is a word the game knows and `sage` is
    /// nameable everywhere — and is then refused by `Verb::anchor` with *"there
    /// is no `mortar_and_pestle` here to grind with"*. A lint that only parsed
    /// would have passed the laboratory's whole example printed in the lens,
    /// which is exactly the failure it exists to stop.
    ///
    /// So: the line must **resolve** through the real parser in the real room —
    /// which catches a renamed argument, and renaming is what `tests/naming.rs`
    /// exists because it happens — *and* its verb must be one that room
    /// **offers**, which is `a_primer_only_names_words_that_room_actually_offers`'
    /// bar and the same filter the listing under `help` uses.
    ///
    /// Availability rather than success, for the primer lint's reason: whether a
    /// line then *works* is a question about the tower's state at that moment,
    /// and a manual cannot promise that.
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
        // Both directions, in two rooms. The listing is `execute::offered`, which
        // is the boot report's filter — so this is also what stops the tutorial a
        // player reads at launch disagreeing with the manual a minute later.
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
            // Nothing gated and nothing dark. `bind` refuses at concentration 0
            // and `undo` is not built; either would be a dead end in the first
            // thing a lost player reads.
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
        // A group with no prose draws its own key — `Prose::line` returns the key
        // for a miss, deliberately, so the failure is visible rather than blank.
        // This makes it a test failure instead.
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
        // **The authored line, kept honest.** `signature()` carries no
        // connectives — `move` is `[Reagent, Place?, Place]` with no `to` — so a
        // generated synopsis would read `move reagent place place`, which nobody
        // types. It is written instead, and this is what stops it drifting from
        // the signature it describes.
        let prose = crate::content::Prose::builtin();
        for verb in Verb::ALL {
            let key = format!("man_{}_use", verb.canonical());
            let line = prose.line(&key, &[]);
            assert!(
                line.starts_with(verb.canonical()),
                "{key} does not open with the word it documents: {line:?}",
            );
            // **Three verbs are exempt, and the exemption is a recorded debt.**
            // Both take `Place` slots because the place half of a spell's
            // condition resolves against exactly that kind — which is why the
            // archive's four ways and the lens's sockets and sigils are places
            // you cannot stand in (§19). `follow <place>` and
            // `seat <place> <place>` would be honest about the implementation
            // and wrong for a player, who is choosing a direction, or a dial and
            // a mark.
            //
            // The exemption is still an assertion rather than a skip: each names
            // the word the player is actually choosing, so a synopsis cannot
            // drift into saying nothing. It goes when `Role::Reading` fixtures
            // stop having to be `Place`s.
            if let Some(instead) = match verb {
                Verb::Follow => Some(&["way"][..]),
                Verb::Dial => Some(&["socket", "sigil"][..]),
                // ...and the sanctum's stations, for the same reason. The
                // synopsis is `haul <station> to <station>` rather than
                // `<from> <to>`: the direction is the thing a player must get
                // right, and `to` is §6 filler so the line is also a command
                // that works.
                Verb::Haul => Some(&["station"][..]),
                // ...and the menagerie's syllables, third time. `sing <place>`
                // would be true of the type and useless to a player: what they
                // are choosing is one of four sounds, not somewhere to stand.
                Verb::Sing => Some(&["syllable"][..]),
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
        // **The gap an audit against seven other languages found**, and it was
        // in the docs rather than the code: every one of them ships a reference,
        // and ours had none. Control words are outside `Verb::ALL`, so the verb
        // lint below could never have covered them — `recall repeat` reached
        // nothing at all, and a player had no way to discover what an `if` could
        // ask.
        //
        // The readings are the same shape of hole one noun space over: they are
        // `NounKind::Sense` rather than materials, so `every_material_has_a_page`
        // does not see them either, and `recall marks` answered with a *scoping*
        // message about other rooms — a dead end wearing a wrong reason.
        let prose = crate::content::Prose::builtin();
        for word in crate::parser::SpellWord::ALL {
            assert!(
                prose.has(&format!("recall_{}", word.canonical())),
                "`{}` is a word a spell is written with and the manual cannot \
                 say what it does",
                word.canonical(),
            );
        }
        // **Every domain's readings, not just the archive's.** This asked
        // `maze::readings()` alone, and the three other rooms that publish
        // readings were simply not covered — so the menagerie shipped a whole
        // phase in which `recall next` fell through to the room overview, while
        // the comment above claimed the hole was closed. A lint naming one
        // domain is a lint that stops working the day a second one arrives, and
        // §10 has five more.
        //
        // Listed rather than derived, deliberately: a domain's readings are
        // `Vec<&str>` on its own module and there is no registry of those, so
        // the honest options are a list here or a registry nothing else wants.
        // A new domain adds a line, and this fails by name until it does.
        let readings = crate::tower::maze::readings()
            .into_iter()
            .chain(crate::tower::chant::readings())
            .chain(crate::tower::ward::readings())
            .chain(crate::tower::pylon::readings());
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
        // **The same completeness lint, one noun space over.** Verbs have had one
        // since the manual was written; materials had none, so a reagent could be
        // authored with a colour, a recipe and a route and never a word saying
        // what it *was*. §19 records that failing twice already — the four shard
        // names, and the `dust` a byproduct field invented — and both times what
        // found it was a person asking rather than a test.
        //
        // One key required, not three: a material is a thing, and most of them
        // want a sentence rather than a page. `using_` is where a *use* goes and
        // is optional, because a byproduct is something you have rather than
        // something you do.
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
        // **The pages this change was for.** A potion and a scroll are the two
        // things a player *holds*, and a route told them five steps and nothing
        // about the thing in their hand. Every one of them now has a `using_`
        // line — which for a potion honestly says the drinking is not built, on
        // `undo`'s precedent.
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
        // **The completeness lint**, and what stops the manual rotting as verbs
        // are added: a new verb is already a compile error in `Verb::group`, and
        // this makes it a test failure in the prose too.
        //
        // Three keys are required and the rest are optional, because a page's
        // length should follow what there is to say. `_use` and `_gloss` are the
        // two lines the overview and the header both need; `_1` is the shortest
        // honest answer to *what does it do*.
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
        // Every `see also` names real vocabulary. A manual pointing at a word
        // the parser does not have is worse than one that points nowhere,
        // because the player types it and lands in the dead end §15 weighs
        // heaviest.
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
        // `is_destructive` had no reader at all until the manual wanted one. Two
        // lists that must not drift, so they are checked against each other
        // rather than one being derived and the other trusted.
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
        // `Verb::group` has no wildcard, so a new verb is a compile error there —
        // but a verb put in a group nobody prints would compile and simply not be
        // there. This is what catches that.
        for verb in Verb::ALL {
            assert!(
                Group::ALL.contains(&verb.group()),
                "{} is in a group the overview never prints",
                verb.canonical(),
            );
        }
    }
}
