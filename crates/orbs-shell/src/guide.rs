//! The scribing guide: what the words mean, beside the words.
//!
//! # Why the editor needs one and the manual is not enough
//!
//! `recall <word>` has answered *what does `repeat` do* since Phase 1, and it is
//! the wrong shape for writing a spell: it is a **command**, so reaching it means
//! leaving the editor, which is the one place a player is when they need it. The
//! language has since grown to nine control words, a question grammar with
//! seventeen comparison spellings, parts, variables and sets — past the size one
//! person holds in their head. A reference you have to close your work to read
//! is a reference nobody reads.
//!
//! So the guide is the same prose, in the same pane, reacting to the caret.
//!
//! # It writes nothing new
//!
//! Every line here comes from `recall_<word>` and `using_<word>` in
//! `prose.toml` — 98 entries that already existed, one definition and one worked
//! example per word, held by `every_word_a_spell_is_written_with_has_a_page`.
//! Rule 6 keeps prose out of Rust and this obeys it by having none of its own:
//! a word authored tomorrow appears here with no code change.
//!
//! # Information, not enrichment
//!
//! Unlike the syntax highlighting beside it, this is **content** — §14 therefore
//! requires it to reach the linear stream, so `sheet` draws it with
//! [`Painter::span`](orbs_render::Painter::span) rather than the silent `glyphs`
//! the buffer uses.

use orbs_render::{Frame, Painter, Pos, Rect, Span, Style, UtteranceKind};
use orbs_sim::parser::Reason;
use orbs_sim::{Prose, Sim};

/// What the guide is showing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Guide {
    /// Nothing is under the caret, so: everything there is to use.
    Vocabulary {
        /// The language's own words.
        control: Vec<Entry>,
        /// The verbs this spell's domain answers to.
        verbs: Vec<Entry>,
    },
    /// The caret is on a word the guide knows, so that word's page.
    Word {
        /// The word itself, as it is written.
        name: String,
        /// What it does.
        does: String,
        /// How it is used.
        using: String,
    },
    /// The caret is part way through a line, so: what may come next.
    ///
    /// The reactive half. `if the mortar_and_pestle is ` has exactly three
    /// answers and a player has no way to know that — the manual says so in a
    /// room they have left to be here.
    Next {
        /// What the grammar allows in this position.
        entries: Vec<Entry>,
    },
}

/// One row of the vocabulary listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The word.
    pub name: String,
    /// What follows it, if the grammar fixes anything — `<count>`, `each <set>`.
    pub shape: String,
}

/// What to show for a buffer with its caret where it is.
///
/// `domain` is the spell's, not the player's: a laboratory spell lists the
/// laboratory's verbs however far away it is being read from, which is the same
/// call `parser::lexeme` makes about highlighting and for the same reason.
#[must_use]
pub fn guide(sim: &Sim, lines: &[String], caret: (usize, usize), domain: &str) -> Guide {
    let prose = sim.prose();
    // A word the manual has a page for beats everything: somebody whose caret is
    // resting on `repeat` is asking what `repeat` does, not what may follow it.
    if let Some(word) = under_caret(lines, caret)
        && let Some(page) = page(prose, &word)
    {
        return page;
    }

    let found = expected(sim, lines, caret, domain);
    // **Part way through a line, the expectation *is* the guide.** At a line
    // start it is the whole vocabulary again, which the listing below says
    // better — with headings, and with the verbs the reference wants.
    if started(lines, caret) && !found.is_empty() {
        return Guide::Next {
            entries: found.expected.iter().map(entry_for).collect(),
        };
    }

    Guide::Vocabulary {
        // **Not all nine.** `until` never opens a line, `end` needs something
        // open and `else` needs an `if` directly above — so a listing of all
        // nine offers three ways to make the orb refuse the line it just
        // suggested. `expect` already knows which; asking it here is what stops
        // the guide keeping a second opinion about the block rules.
        control: found
            .expected
            .iter()
            .filter(|one| one.why == Reason::Word)
            .map(entry_for)
            .collect(),
        verbs: verbs_for(sim, domain),
    }
}

/// Whether the caret has anything before it on its line.
///
/// The line start is where the *listing* belongs and anywhere later is where the
/// *expectation* does. Measured in characters, because that is what a caret is.
fn started(lines: &[String], (row, column): (usize, usize)) -> bool {
    lines.get(row).is_some_and(|line| {
        line.chars()
            .take(column)
            .any(|glyph| !glyph.is_whitespace())
    })
}

/// What the language allows where the caret is.
///
/// Through `orbs-sim`, which builds the scene for the **spell's** domain — see
/// [`spell_expect`](orbs_sim::spell_expect). The block stack comes from the
/// lines above, so `else` and `end` appear exactly where they are legal.
fn expected(
    sim: &Sim,
    lines: &[String],
    (row, column): (usize, usize),
    domain: &str,
) -> orbs_sim::parser::Expectation {
    // **A caret that is nowhere asks about a fresh line, not about nothing.**
    // `Editor::refresh` passes `usize::MAX` in command mode, where there is no
    // caret in the buffer at all — and answering with an empty expectation
    // emptied the listing's control column, so the whole `the language` heading
    // vanished the moment the editor was not being typed into. Which is most of
    // the time it is open.
    let Some(line) = lines.get(row) else {
        return orbs_sim::spell_expect(sim.world(), domain, "", 0, &[]);
    };
    let open = orbs_sim::parser::open_blocks(&lines[..row]);
    orbs_sim::spell_expect(sim.world(), domain, line, column, &open)
}

/// One row of a listing: the word, and what follows it if anything does.
fn shown(entry: &Entry) -> String {
    if entry.shape.is_empty() {
        entry.name.clone()
    } else {
        format!("{} {}", entry.name, entry.shape)
    }
}

/// One offered word, as the listing shows it.
fn entry_for(one: &orbs_sim::parser::Expected) -> Entry {
    Entry {
        name: one.text.clone(),
        shape: one.shape.to_owned(),
    }
}

/// The word the caret has reached, if it has reached one.
///
/// # "At the end of", and the start of a word is not it
///
/// A player types `repeat` and the caret finishes *past* the `t` — no character
/// is under it at all. So the page has to answer for a caret at `run.end`.
///
/// A caret at `run.start` is **not** the same thing and deliberately does not
/// match: it is where the caret sits when a spell is merely *opened*, and
/// answering there would show `scribe threading` a page about its first word
/// instead of the listing it is supposed to open on.
///
/// The first version of this got both halves wrong in one predicate —
/// `start <= at && at <= run.end` matched the start and missed the end — under a
/// doc comment claiming the opposite. That is the defect §19 records over and
/// over: a comment claiming parity, sitting next to the code that broke it.
///
/// # The whitespace *after* a word is not this question any more
///
/// It was, for one version: a caret in the run of spaces after `repeat` read as
/// still being on `repeat`. That was right when a page was the only thing the
/// guide could show, and it is wrong now — a player who has typed the space has
/// finished with the word and is asking **what comes next**, which
/// [`guide`] answers from `expect`. Keeping both would mean `if ` explains `if`
/// while `is ` lists the states, for no reason a player could ever infer.
///
/// So the rule is: touching the word is a page, past it is an expectation.
fn under_caret(lines: &[String], (row, column): (usize, usize)) -> Option<String> {
    let line = lines.get(row)?;
    // The caret is a count of characters; the lexer speaks in bytes.
    let at = line
        .char_indices()
        .nth(column)
        .map_or(line.len(), |(index, _)| index);

    let runs = orbs_sim::parser::lex(line);
    // Inside a word, or exactly at its end.
    runs.iter()
        .find(|run| run.start < at && at <= run.end)
        .and_then(|run| line.get(run.start..run.end))
        .map(str::to_lowercase)
}

/// One word's page, if the manual has one for it.
///
/// # Two key families, because the manual has two
///
/// A **control word** is authored as `recall_<word>` and `using_<word>` — a
/// definition and a worked example. A **verb** is authored as `man_<verb>_gloss`
/// and `man_<verb>_use` — what it does, and the shape it takes. Both are read
/// here rather than one being normalised into the other, because the manual is
/// the authority and rewriting its keys to suit this pane would be the tail
/// wagging the dog.
///
/// A word in neither is not a word the manual knows — a part the player named,
/// a reagent, a typo — and the guide falls back to the listing rather than
/// showing an empty page.
fn page(prose: &Prose, word: &str) -> Option<Guide> {
    // **A call names a part, and a part is the player's own word.** The manual
    // has no page for `gathering`, and inventing one would be the guide claiming
    // to know something about a name only this file defines.
    let word = word.strip_suffix("()").unwrap_or(word);

    for (does, using) in [
        (format!("recall_{word}"), format!("using_{word}")),
        (format!("man_{word}_gloss"), format!("man_{word}_use")),
    ] {
        if prose.has(&does) {
            return Some(Guide::Word {
                name: word.to_owned(),
                does: prose.line(&does, &[]),
                // A word may have a definition and no example — `else` and `end`
                // take nothing. An absent line is empty rather than a key.
                using: if prose.has(&using) {
                    prose.line(&using, &[])
                } else {
                    String::new()
                },
            });
        }
    }
    None
}

/// The verbs a spell written for `domain` may issue.
///
/// **Two filters, and both are the spell's rather than the player's.** A verb is
/// listed when its fixture stands in that domain — `grind` in the laboratory,
/// `follow` in the archive — *and* when a spell is allowed to issue it at all.
/// The second is why `attend`, `meditate` and `scribe` are absent: `may_issue`
/// refuses them, and a guide that offered a word the runner then refused would be
/// teaching the player a line that cannot work.
fn verbs_for(sim: &Sim, domain: &str) -> Vec<Entry> {
    orbs_sim::spell_vocabulary(sim.world(), domain)
        .into_iter()
        .map(|verb| Entry {
            name: verb.canonical().to_owned(),
            shape: verb.signature_label().to_owned(),
        })
        .collect()
}

/// Draw the guide into `pane`.
///
/// # Every row is announced, and each is one utterance
///
/// This is **content**, not decoration: §14 requires a definition a sighted
/// player can read to reach a listener too. So rows go through
/// [`Painter::span`](orbs_render::Painter::span) rather than the silent `glyphs`
/// the buffer uses — and each wrapped row is announced whole, because a sentence
/// broken across three rows is still one sentence and three utterances would be
/// the `0.3.23` defect wearing a different pane.
///
/// They carry [`UtteranceKind::Guide`], which exists so a reader can drop the
/// lot: the stream is rebuilt every frame, so a standing reference with no kind
/// of its own would be recited continuously.
pub fn paint(frame: &mut Frame, pane: Rect, guide: &Guide, prose: &Prose) {
    let mut painter = frame.painter(pane);
    let area = painter.area();
    painter.border(area, Some(&prose.line("guide_title", &[])), Style::DIM);

    let inside = area.cols.saturating_sub(2);
    let mut row = area.row.saturating_add(1);
    let last = area.row.saturating_add(area.rows).saturating_sub(2);
    let at = area.col.saturating_add(1);

    let write = |painter: &mut Painter<'_>, row: &mut u16, text: &str, style: Style| {
        for line in orbs_render::Wrap::new(text, inside) {
            if *row > last {
                return;
            }
            painter.span(
                Pos::new(at, *row),
                &Span::new(line)
                    .with_style(style)
                    .with_kind(UtteranceKind::Guide),
            );
            *row = row.saturating_add(1);
        }
    };

    match guide {
        Guide::Word { name, does, using } => {
            write(&mut painter, &mut row, name, Style::BRIGHT);
            write(&mut painter, &mut row, does, Style::NORMAL);
            if !using.is_empty() {
                row = row.saturating_add(1);
                write(&mut painter, &mut row, using, Style::DIM);
            }
        }
        Guide::Next { entries } => {
            write(
                &mut painter,
                &mut row,
                &prose.line("guide_next", &[]),
                Style::BRIGHT,
            );
            for entry in entries {
                write(&mut painter, &mut row, &shown(entry), Style::NORMAL);
            }
        }
        Guide::Vocabulary { control, verbs } => {
            for (heading, entries) in [
                ("guide_words", control.as_slice()),
                ("guide_verbs", verbs.as_slice()),
            ] {
                if entries.is_empty() {
                    continue;
                }
                write(
                    &mut painter,
                    &mut row,
                    &prose.line(heading, &[]),
                    Style::BRIGHT,
                );
                for entry in entries {
                    write(&mut painter, &mut row, &shown(entry), Style::NORMAL);
                }
                row = row.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tower, for the verb listing.
    fn tower() -> Sim {
        Sim::new(1)
    }

    fn lines(text: &[&str]) -> Vec<String> {
        text.iter().map(|line| (*line).to_owned()).collect()
    }

    /// At the top of an empty spell, the words that may actually open a line.
    ///
    /// **Six, not nine, and it listed nine for a version.** `until` never opens
    /// a line — it is `repeat`'s guard, written on `repeat`'s own line — and
    /// `end` and `else` need something open above them. A guide listing all
    /// nine offers three ways to make the orb refuse the line it just suggested,
    /// which is the dead end §15 weighs above the resolution rate, taught.
    #[test]
    fn it_lists_only_the_words_that_can_open_a_line_here() {
        let guide = guide(&tower(), &lines(&[""]), (0, 0), "laboratory");
        let Guide::Vocabulary { control, verbs } = guide else {
            panic!("an empty line named a word");
        };
        let words: Vec<&str> = control.iter().map(|one| one.name.as_str()).collect();
        // `bide` opens a line like `wait` does — a delay is a statement, not a
        // guard — so it belongs here and the three that cannot open one still do
        // not. `pull` is a statement too, and `let`'s sibling: it binds a name.
        assert_eq!(
            words,
            [
                "wait",
                "repeat",
                "if",
                "let",
                "for",
                "part",
                "bide",
                "pull",
                "alongside",
            ]
        );
        assert!(
            verbs.iter().any(|entry| entry.name == "grind"),
            "the laboratory's own verbs are missing: {verbs:?}",
        );
    }

    /// ...and `else` and `end` arrive as soon as they are legal.
    ///
    /// The reactive half of the listing: a player learns `else` exists by
    /// opening an `if`, which is where they would want it.
    #[test]
    fn a_word_appears_in_the_listing_once_it_is_legal() {
        let inside = lines(&["if the mortar_and_pestle is idle", ""]);
        let guide = guide(&tower(), &inside, (1, 0), "laboratory");
        let Guide::Vocabulary { control, .. } = guide else {
            panic!("an empty line named a word");
        };
        let words: Vec<&str> = control.iter().map(|one| one.name.as_str()).collect();
        assert!(words.contains(&"else"), "{words:?}");
        assert!(words.contains(&"end"), "{words:?}");
        assert!(
            !words.contains(&"until"),
            "`until` still never opens a line: {words:?}",
        );
    }

    /// Part way through a line, the guide says what may come next.
    ///
    /// This is the ask: `if the mortar_and_pestle is ` has exactly three
    /// answers and nothing on screen said so.
    #[test]
    fn part_way_through_a_line_it_says_what_may_follow() {
        let asking = lines(&["if the mortar_and_pestle is "]);
        let column = asking[0].chars().count();
        let Guide::Next { entries } = guide(&tower(), &asking, (0, column), "laboratory") else {
            panic!("a half-written question fell back to the listing");
        };
        let words: Vec<&str> = entries.iter().map(|one| one.name.as_str()).collect();
        assert_eq!(
            words,
            [
                "idle", "free", "still", "working", "busy", "running", "empty", "bare"
            ],
        );
    }

    /// ...and a word with a page of its own still wins.
    ///
    /// Somebody whose caret is resting on `repeat` is asking what `repeat`
    /// does, not what may follow it.
    #[test]
    fn a_word_with_a_page_beats_what_may_follow_it() {
        let typing = lines(&["repeat"]);
        assert!(
            matches!(
                guide(&tower(), &typing, (0, 6), "laboratory"),
                Guide::Word { .. }
            ),
            "the page lost to the expectation",
        );
    }

    #[test]
    fn it_lists_the_spells_domain_and_not_the_players() {
        // A spell is written *for* a room and runs there however far away it is
        // being edited from, so the listing follows the spell.
        let sim = tower();
        let archive = guide(&sim, &lines(&[""]), (0, 0), "archive");
        let Guide::Vocabulary { verbs, .. } = archive else {
            panic!("expected a listing");
        };
        let named: Vec<&str> = verbs.iter().map(|entry| entry.name.as_str()).collect();
        assert!(named.contains(&"follow"), "the archive's verbs: {named:?}");
        assert!(
            !named.contains(&"grind"),
            "a laboratory verb was offered to an archive spell: {named:?}",
        );
    }

    #[test]
    fn a_verb_a_spell_may_not_issue_is_never_offered() {
        // `may_issue` refuses these, so a guide naming one would be teaching a
        // line that cannot work — worse than a listing that is short.
        let sim = tower();
        let Guide::Vocabulary { verbs, .. } = guide(&sim, &lines(&[""]), (0, 0), "laboratory")
        else {
            panic!("expected a listing");
        };
        let named: Vec<&str> = verbs.iter().map(|entry| entry.name.as_str()).collect();
        for refused in ["attend", "meditate", "scribe", "weave", "quit"] {
            assert!(
                !named.contains(&refused),
                "{refused} was offered: {named:?}"
            );
        }
    }

    #[test]
    fn the_caret_at_the_end_of_a_word_opens_its_page() {
        // **The case that matters**, because it is what typing looks like: the
        // caret is past the last character rather than on it. A guide that
        // answered only for a caret strictly inside a word would react to
        // arrowing around and not to writing.
        let sim = tower();
        let text = lines(&["repeat 3"]);
        let Guide::Word { name, does, using } = guide(&sim, &text, (0, 6), "laboratory") else {
            panic!("the caret at the end of `repeat` named no word");
        };
        assert_eq!(name, "repeat");
        assert!(!does.is_empty() && !using.is_empty(), "an empty page");
    }

    #[test]
    fn it_answers_for_a_verb_as_well_as_a_control_word() {
        let sim = tower();
        let text = lines(&["grind sage"]);
        let Guide::Word { name, .. } = guide(&sim, &text, (0, 5), "laboratory") else {
            panic!("the caret at the end of `grind` named no word");
        };
        assert_eq!(name, "grind");
    }

    /// Opening a spell shows the listing, not a page about its first word.
    ///
    /// `Editor::open` starts the caret at `(0, 0)`, and a rule that matched the
    /// *start* of a word would greet every `scribe threading` with a page about
    /// `repeat`. The ask is that the guide opens on what you can use.
    #[test]
    fn opening_a_spell_opens_on_the_listing() {
        let sim = tower();
        let text = lines(&["repeat until the stacks is idle", "follow north"]);
        assert!(
            matches!(
                guide(&sim, &text, (0, 0), "archive"),
                Guide::Vocabulary { .. }
            ),
            "opening a spell showed a word page instead of the vocabulary",
        );
    }

    /// Touching the word is a page; past it is an expectation.
    ///
    /// **The space used to still name the word**, on the argument that the
    /// moment a player most wants `repeat`'s page is right after typing it. It
    /// is not: what they want then is `repeat`'s *argument*, which is the thing
    /// the guide could not say at all until `expect` existed.
    ///
    /// Keeping the old rule alongside the new one would have made `if `
    /// explain `if` while `is ` listed the states, for no reason a player could
    /// ever infer — the difference being only whether the manual happens to
    /// have a page for the word.
    #[test]
    fn touching_a_word_is_a_page_and_past_it_is_what_follows() {
        let sim = tower();
        let text = lines(&["repeat "]);

        let Guide::Word { name, .. } = guide(&sim, &text, (0, 6), "laboratory") else {
            panic!("the caret at the end of `repeat` lost its page");
        };
        assert_eq!(name, "repeat");

        let Guide::Next { entries } = guide(&sim, &text, (0, 7), "laboratory") else {
            panic!("the caret past `repeat ` did not ask what follows");
        };
        let words: Vec<&str> = entries.iter().map(|one| one.name.as_str()).collect();
        assert_eq!(words, ["until"]);
    }

    /// ...and the same, one line up, for the word that started this feature.
    #[test]
    fn the_space_after_if_lists_the_places_to_ask_about() {
        let sim = tower();
        let Guide::Next { entries } = guide(&sim, &lines(&["if "]), (0, 3), "laboratory") else {
            panic!("`if ` showed a page instead of what it takes");
        };
        assert!(
            entries.iter().any(|one| one.name == "mortar_and_pestle"),
            "the laboratory's instruments are not offered: {entries:?}",
        );
    }

    #[test]
    fn a_part_the_player_named_has_no_page_and_says_so_by_listing() {
        // The manual has no entry for `gathering`, and inventing one would be the
        // guide claiming to know a name only this file defines. It falls back to
        // the vocabulary rather than showing an empty page.
        let sim = tower();
        let text = lines(&["gathering()"]);
        assert!(
            matches!(
                guide(&sim, &text, (0, 11), "laboratory"),
                Guide::Vocabulary { .. }
            ),
            "a player's own part was given a manual page",
        );
    }
}
