//! Reading the screen back: the transcript column, the newest block, the clock.

use crate::play::WIZARD;

/// The transcript column of every row, with the rail and borders removed.
pub fn transcript(screen: &str) -> String {
    let mut out = String::new();
    for row in screen.lines() {
        // `│transcript│rail│` splits to ["", transcript, rail, ""]. The live
        // prompt row has no border and drops out, correctly: it holds a
        // half-typed line the game has not been told about.
        if let Some(cell) = row.split('│').nth(1) {
            // Cut at a corner: a map's `┌───┐` and `└───┘` have no `│` in them,
            // so they stay in this cell glued to the transcript — once giving
            // `a way out └───┘ is in them`, and twelve scenarios gave up. Safe
            // because only `Painter::border` draws a box corner.
            let cell = cell.find(['┌', '└']).map_or(cell, |at| &cell[..at]);
            out.push_str(cell.trim_end());
            out.push('\n');
        }
    }
    out
}

/// Whether the live prompt has given up the line that was typed into it.
///
/// The prompt row is drawn **outside** the pane border and is the only text the
/// game has not been told about. It carries the ghost completion too, so this
/// asks whether the typed line is gone, not whether the row is bare.
pub fn prompt_taken(screen: &str, line: &str) -> bool {
    let header = format!("{WIZARD} $");
    screen
        .lines()
        .rfind(|row| !row.starts_with('│') && row.contains(&header))
        .is_none_or(|row| !row.contains(line))
}

/// The newest command block, header included — what `Game::send` waits for.
pub fn block_with_header(screen: &str) -> String {
    let pane = transcript(screen);
    let header = format!("{WIZARD} $");
    pane.rfind(&header)
        .map_or_else(|| pane.clone(), |at| pane[at..].to_owned())
}

/// What the game said back — the newest block **without the line that was typed**.
///
/// The header has to go, or assertions pass on the echo:
/// `does("sift charcoal laboratory.log", "charcoal")` was satisfied by its own
/// argument, with the log empty and the claim untested. Nine shipped that way.
pub fn block(screen: &str) -> String {
    let full = block_with_header(screen);
    full.split_once('\n')
        .map_or_else(String::new, |(_, body)| body.to_owned())
}

/// One long line, for matching a phrase the pane may have wrapped.
///
/// `RecordView` wraps rather than clips, with a two-cell continuation indent, so
/// a sentence longer than the pane splits mid-phrase. `research`'s answer at
/// 120×45 lands as
///
/// ```text
/// √ the page opens into shelves that do not end. a way out
///     is in them
/// ```
///
/// and a needle as a person reads it matches neither row. Every wait flattens
/// first, so an assertion is about *what the game said* rather than where this
/// width broke the line.
pub fn flatten(text: &str) -> String {
    // Leader runs collapse too: `status` draws `experience ..... 0`, and seven
    // scenarios asserting `experience 0` broke the day it did.
    //
    // A **run**, never a single `.`, so a sentence keeps its full stops and a
    // file its extension: `orbs-save.toml` and `laboratory.log` are real needles.
    let mut out = String::with_capacity(text.len());
    let mut dots = 0usize;
    for ch in text.chars() {
        if ch == '.' {
            dots += 1;
            continue;
        }
        if dots > 0 {
            // Two or more is a leader; one is punctuation and is kept.
            out.push_str(if dots == 1 { "." } else { " " });
            dots = 0;
        }
        out.push(ch);
    }
    if dots == 1 {
        out.push('.');
    } else if dots > 1 {
        out.push(' ');
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The world's clock, wherever this grid happens to draw it.
///
/// Never a plain search for `tick`: the athanor says *"fuel for 600 ticks"* and
/// `status` prints a snapshot `tick N`. Only the two places the *live* reading
/// is drawn — the rail's foot, and the border title where there is no rail.
pub fn parse_tick(screen: &str) -> Option<u64> {
    // From the right, because an inline map adds a pane and a fixed index from
    // the left silently reads blank. The length guard keeps `status`'s snapshot
    // `tick N` from being read as the rail at a grid that has none.
    for row in screen.lines() {
        let cells: Vec<&str> = row.split('│').collect();
        if cells.len() >= 5
            && let Some(cell) = cells.get(cells.len() - 2)
            && let Some(rest) = cell.trim().strip_prefix("tick")
            && let Ok(tick) = rest.trim().parse()
        {
            return Some(tick);
        }
    }
    let title = screen.lines().next()?;
    let at = title.find("tick ")?;
    title[at + "tick ".len()..]
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

/// A pane beside the transcript never becomes part of what the transcript says.
///
/// Pure, and not `#[ignore]`d: every scenario reads the game through this
/// helper, and a defect in it arrived as twelve unrelated-looking failures.
///
/// The rows are the ones `research` answered on when those twelve failed: the
/// map's top edge beside an instrument row, its side beside the echo, and its
/// bottom edge sharing the first row of a wrapped sentence.
#[test]
fn a_pane_beside_the_transcript_does_not_leak_into_it() {
    let screen = "\
│  orb 0 181 cold start      ┌ stacks ──────────┐│le st ││
│→ research                  │██████████████████││   ░░ ││
│√ the page opens into shelves that do not end. a way out  └──────────────────┘│   ░░ ││
│    is in them                                                                │   ░░ ││";
    let read = flatten(&transcript(screen));
    assert!(
        read.contains("a way out is in them"),
        "the map's edge split the sentence: {read:?}",
    );
    assert!(
        !read.contains("stacks"),
        "the map's title was read as transcript: {read:?}",
    );
    assert!(read.contains("orb 0 181 cold start"), "{read:?}");
}
