//! Where the book comes from.
//!
//! Two sources, one of them new. The authored chapters are `manual.toml`'s; the
//! rest are built from the 578 `man_`, `recall_` and `using_` keys `recall` has
//! accumulated since Phase 1, already held by that module's fourteen lints.
//!
//! §12 puts writing at the top of the risk list and budgets ~88k words. The
//! cheapest words are the ones already written, and a second copy would be a
//! verb's page saying one thing in `help` and another in the manual.
//!
//! ⚠ Indentation does not survive. `Wrap::new` trims leading spaces — it must,
//! or a wrapped line would inherit the indent of the one it broke from — so a
//! line cannot be nested by putting spaces in front of it. Internal padding is
//! untouched, which is what makes a two-column row work.
//!
//! So hierarchy is made of columns, blank lines and words, never indentation,
//! in `manual.toml` exactly as in the two chapters built here.

use orbs_sim::Sim;
use orbs_sim::content::Chapter;
use orbs_sim::parser::{Group, SpellWord, Verb};

/// The chapter that lists every command.
const COMMANDS: &str = "commands";

/// The chapter that lists every word a spell is written with.
const LANGUAGE: &str = "language";

/// The chapter that gives every room its own three lines.
const INSIDE: &str = "inside";

/// Every place a player can stand, in the order a tower opens them.
///
/// The six domains and the four rooms that are not — the tower's floor, the
/// grimoire, the arsenal, the bailey — because a player asking *what is behind
/// that door* does not care which has production in it. The three keys exist for
/// all ten; this array is the reading order, not the membership.
const PLACES: [&str; 10] = [
    "tower",
    "laboratory",
    "archive",
    "lens",
    "sanctum",
    "menagerie",
    "forge",
    "grimoire",
    "arsenal",
    "bailey",
];

/// The manual, assembled.
///
/// Called when the reader opens, for the saves listing's reason: it walks the
/// prose once and the answer does not change while somebody is reading it.
///
/// Authored chapters first, then the generated ones: a player who has just found
/// the orb wants *what this is* before *every word in the language*.
#[must_use]
pub fn book(sim: &Sim) -> Vec<Chapter> {
    let mut book = sim.manual().chapters().to_vec();
    book.push(inside(sim));
    book.push(commands(sim));
    book.push(language(sim));
    book
}

/// Every room, in the three lines `man` gives the one you are standing in.
///
/// The depth the authored *rooms* chapter deliberately does not carry: that one
/// is a paragraph read before you have a tower, this is `primer`'s thirty lines
/// — what the place is, how to begin its puzzle, how to finish one. Authoring it
/// again would be a second copy of thirty sentences.
///
/// The difference from `man` is `commands`' difference from `help`: `man`
/// answers *where am I*, not *what is behind a door the tower has not opened*.
fn inside(sim: &Sim) -> Chapter {
    let prose = sim.prose();
    let mut lines = vec![
        prose.line("manual_inside_lead", &[]),
        String::new(),
        prose.line("manual_inside_where", &[]),
        prose.line("manual_inside_sealed", &[]),
        String::new(),
    ];
    for place in PLACES {
        // `primer`'s own three keys and its own order — what-it-is before
        // how-it-works — guarded on `has` for the same reason it is: a room
        // whose primer is unwritten prints nothing rather than a key.
        let said: Vec<String> = ["here", "start", "solve"]
            .into_iter()
            .map(|part| format!("man_{part}_{place}"))
            .filter(|key| prose.has(key))
            .map(|key| prose.line(&key, &[]))
            .collect();
        if said.is_empty() {
            continue;
        }
        // The room's name, on its own line with air around it — the heading
        // shape the authored *rooms* chapter uses, and lower case because the
        // whole game is. `primer` gives the reason it is the room's own name
        // rather than a `man_section_<room>` key saying the name back.
        lines.push(place.to_owned());
        lines.push(String::new());
        lines.extend(said);
        lines.push(String::new());
    }
    Chapter {
        name: INSIDE.to_owned(),
        title: prose.line("manual_inside_title", &[]),
        lines,
    }
}

/// Every command, grouped as `help` groups them.
///
/// The same table and prose `help` uses, not a second listing beside it:
/// `recall.rs` holds fourteen completeness lints over these keys, and a manual
/// with its own copy would be outside all of them.
///
/// The difference from `help` is what this chapter exists for: `help` lists what
/// works where you are standing, which is no answer to *what is there*. This is
/// every verb, including ones whose rooms a sealed tower has not opened.
fn commands(sim: &Sim) -> Chapter {
    let prose = sim.prose();
    let mut lines = vec![
        prose.line("manual_commands_lead", &[]),
        String::new(),
        prose.line("manual_commands_where", &[]),
        String::new(),
    ];
    // Measured, not guessed: a hard-coded column was narrower than
    // `mix <reagent> with <reagent>` and every long synopsis ran into its gloss,
    // which looks like a wrapping bug and is a padding one.
    let column = Verb::ALL
        .into_iter()
        .map(|verb| {
            prose
                .line(&format!("man_{}_use", verb.canonical()), &[])
                .chars()
                .count()
        })
        .max()
        .unwrap_or(0)
        + 2;
    for group in Group::ALL {
        lines.push(prose.line(group.key(), &[]));
        // A blank line, because the indent does not survive. These rows carried
        // two leading spaces to sit under their heading, `Wrap::new` trimmed
        // every one, and the grouping was invisible on screen. See the module
        // header: hierarchy is columns, blank lines and words.
        lines.push(String::new());
        for verb in Verb::ALL.into_iter().filter(|verb| verb.group() == group) {
            let canonical = verb.canonical();
            let synopsis = prose.line(&format!("man_{canonical}_use"), &[]);
            let gloss = prose.line(&format!("man_{canonical}_gloss"), &[]);
            lines.push(format!("{synopsis:<column$}{gloss}"));
        }
        lines.push(String::new());
    }
    Chapter {
        name: COMMANDS.to_owned(),
        title: prose.line("manual_commands_title", &[]),
        lines,
    }
}

/// Every word a spell is written with.
///
/// Built from `recall_<word>` and `using_<word>` — the 98 entries
/// `every_word_a_spell_is_written_with_has_a_page` already holds, and the same
/// ones the editor's scribing guide draws beside the caret. A word authored
/// tomorrow appears here with no code change, which is `guide.rs`'s own rule.
fn language(sim: &Sim) -> Chapter {
    let prose = sim.prose();
    let mut lines = vec![prose.line("manual_language_lead", &[]), String::new()];
    // Two columns rather than an indent, for the module doc's reason: `Wrap`
    // trims leading spaces, so a nested line is not nested once wrapped.
    let column = SpellWord::ALL
        .into_iter()
        .map(|word| word.canonical().chars().count())
        .max()
        .unwrap_or(0)
        + 2;
    for word in SpellWord::ALL {
        let name = word.canonical();
        let says = prose.line(&format!("recall_{name}"), &[]);
        lines.push(format!("{name:<column$}{says}"));
        let using = format!("using_{name}");
        if prose.has(&using) {
            // The worked example, marked rather than indented — `man_page_like`
            // is the word `recall`'s own pages use for it.
            lines.push(format!(
                "{}: {}",
                prose.line("man_page_like", &[]),
                prose.line(&using, &[]),
            ));
        }
        lines.push(String::new());
    }
    Chapter {
        name: LANGUAGE.to_owned(),
        title: prose.line("manual_language_title", &[]),
        lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_chapter_the_contents_names_is_one_the_book_can_open() {
        // The completeness shape `recall.rs` uses fourteen times: a chapter that
        // is listed and cannot be opened is the dead end §15 weighs heaviest,
        // and a generated one can become that by a key being renamed.
        let sim = Sim::new(1);
        for chapter in book(&sim) {
            assert!(
                !chapter.title.is_empty() && !chapter.title.starts_with("manual_"),
                "{} has no authored title — the prose key fell through",
                chapter.name,
            );
            assert!(
                !chapter.lines.is_empty(),
                "{} is listed with nothing in it",
                chapter.name,
            );
        }
    }

    #[test]
    fn the_commands_chapter_names_every_verb_in_the_game() {
        // Every verb, not the ones that work where you stand — the whole
        // difference from `help`, which cannot mention a room a sealed tower has
        // not opened.
        let sim = Sim::new(1);
        let chapter = commands(&sim);
        let text = chapter.lines.join("\n");
        for verb in Verb::ALL {
            let canonical = verb.canonical();
            assert!(
                text.contains(canonical),
                "the commands chapter never names `{canonical}`",
            );
        }
    }

    #[test]
    fn every_room_a_player_can_stand_in_is_behind_one_of_the_doors() {
        // The lint `recall.rs` holds over `primer`, one surface over. A room
        // built after this — §10 has none left, but a siege stage or a second
        // courtyard would — must either gain the three keys or be left off
        // `PLACES` deliberately, rather than quietly missing from the chapter.
        let sim = Sim::new(1);
        let chapter = inside(&sim);
        let text = chapter.lines.join("\n");
        for place in PLACES {
            assert!(
                text.contains(place),
                "the inside chapter never opens the door to `{place}`",
            );
            assert!(
                sim.prose().has(&format!("man_here_{place}")),
                "`{place}` is listed with no primer — `man` cannot describe it either",
            );
        }
    }

    #[test]
    fn no_generated_line_leans_on_an_indent_that_will_not_survive() {
        // The module header's rule, as a test. `commands` did exactly this for a
        // version: every verb carried two leading spaces, `Wrap` trimmed them,
        // and the group headings were invisible. Internal padding is what a
        // column is made of and is untouched.
        let sim = Sim::new(1);
        for chapter in [commands(&sim), language(&sim), inside(&sim)] {
            for line in &chapter.lines {
                assert!(
                    !line.starts_with(' '),
                    "{} indents a line that `Wrap` will flatten: {line:?}",
                    chapter.name,
                );
            }
        }
    }

    #[test]
    fn no_generated_line_is_a_prose_key_that_fell_through() {
        // `Prose::line` returns the key when it is missing — *"deliberately
        // visible rather than empty"* — so a renamed key shows up as
        // `man_grind_use` on a player's screen. It is a test rather than a hope
        // because nothing else reads these two chapters.
        let sim = Sim::new(1);
        for chapter in [commands(&sim), language(&sim), inside(&sim)] {
            for line in &chapter.lines {
                let said = line.trim();
                assert!(
                    !said.starts_with("man_")
                        && !said.starts_with("recall_")
                        && !said.starts_with("using_")
                        && !said.starts_with("manual_"),
                    "{} has a prose key that fell through: {line:?}",
                    chapter.name,
                );
            }
        }
    }

    #[test]
    fn the_generated_chapters_do_not_collide_with_the_authored_ones() {
        // The contents page is prefix-matched, so `c` and `l` have to mean one
        // chapter each. `content::manual` holds it for the authored ones; this
        // holds it across the join.
        let sim = Sim::new(1);
        let names: Vec<String> = book(&sim).into_iter().map(|chapter| chapter.name).collect();
        let mut firsts: Vec<char> = names
            .iter()
            .filter_map(|name| name.chars().next())
            .collect();
        firsts.sort_unstable();
        let before = firsts.len();
        firsts.dedup();
        assert_eq!(
            before,
            firsts.len(),
            "two chapters share a first letter: {names:?}",
        );
    }
}
