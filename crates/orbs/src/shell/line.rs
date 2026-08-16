//! The line the player is typing.
//!
//! The buffer, the caret, the history, and what Tab and the ghost make of them.
//! Nothing here reads a keystroke — [`input`](super::input) does that and calls
//! these — so the whole editor is testable without a window, which is what the
//! `Line`-only tests below rely on.
//!
//! # The caret counts characters, not bytes
//!
//! CP437 admits multi-byte glyphs that occupy exactly one cell (`░`, `Σ`), so a
//! byte caret would split one and panic on the slice. Every helper here counts
//! characters for the same reason, through [`orbs_render::char_index`] — shared
//! with `orbs-sim`'s completer, because the two have to agree about where a word
//! starts or Tab overwrites the wrong bytes.

use bevy::prelude::*;
use orbs_render::char_index;

/// The parser's own limit (§6). Matching it here means the buffer never holds
/// something the parser would truncate — a shorter cap would silently change
/// what a command means, and a longer one would let the player type into a void.
const MAX_INPUT: usize = 512;

/// How many lines of history are kept.
///
/// Far more than a session reaches, and it bounds the memory. History is not
/// persisted — it dies with the process, and giving it a home arrives with
/// Phase 11's settings, where §4's sticky skip is already waiting.
const MAX_HISTORY: usize = 100;

/// The command line as it currently stands.
#[derive(Resource, Debug, Default)]
pub(crate) struct Line {
    text: String,
    /// Where the next character goes, in **characters** from the start.
    caret: usize,
    /// Submitted lines, oldest first.
    history: Vec<String>,
    /// How far back a recall has walked, as an index into `history`.
    recalled: Option<usize>,
    /// The prefix a history search is anchored to.
    ///
    /// Held for the whole search rather than re-read from the line each press:
    /// after one recall the line *is* the recalled command, so an unanchored
    /// second `Up` would search on that and behave differently from the first.
    /// fish and zsh both hold the original prefix; this is that.
    search: Option<String>,
    /// What was being typed before a recall started, restored by walking back.
    draft: String,
    /// A Tab cycle in progress, if the last thing pressed was Tab.
    cycle: Option<Cycle>,
}

/// Where a run of Tab presses has got to.
///
/// Repeated Tab **cycles** through the candidates rather than re-listing them —
/// readline's `menu-complete`, and what a player expects after the first press
/// says there is more than one answer. Holding the candidate list here rather
/// than recomputing it each press is what makes the cycle stable: the scene can
/// change under a player who is mid-cycle (a tick lands, an instrument finishes)
/// and the list they are walking must not reorder beneath them.
#[derive(Debug)]
struct Cycle {
    /// Byte offset where the completable word begins.
    start: usize,
    /// What currently sits there: the word the player typed until the first
    /// advance, then whichever candidate replaced it.
    ///
    /// Kept so [`Line::advance_cycle`] can check the line still says what the
    /// cycle last wrote before overwriting it.
    filled: String,
    /// The candidates, in the order the completer offered them.
    candidates: Vec<String>,
    /// Which one is in the line, or `None` before the first advance.
    ///
    /// The first Tab **lists without changing the line** — bash's default, and
    /// the least surprising thing to do to someone who pressed Tab to ask a
    /// question rather than to make a choice. Choosing starts on the second.
    index: Option<usize>,
}

impl Line {
    /// A line with `text` already in it and the caret at the end.
    ///
    /// For `ORBS_LINE` — the only way a dump can show a partly-typed line.
    pub(crate) fn typed(text: &str) -> Self {
        let text: String = text
            .chars()
            .filter(|c| orbs_render::is_renderable(*c))
            .collect();
        let caret = text.chars().count();
        Self {
            text,
            caret,
            ..Self::default()
        }
    }

    /// What is being typed.
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// The window of the line that fits `width` cells, and the caret's column
    /// within it.
    ///
    /// Without this the line has no viewport: at the 80×22 floor the prompt
    /// leaves 72 cells, past which `put_str` clips silently *and*
    /// [`Frame::set_cursor`](orbs_render::Frame::set_cursor) refuses an off-grid
    /// position — so the player types into a dead line with no caret and no
    /// explanation. `sift "march north" /tower/laboratory/feed.log` is 44
    /// characters, so 72 is not a theoretical limit, and the magnified prompt
    /// halves the budget again.
    ///
    /// It used to show the **tail** and put the caret at the end. With a caret
    /// that can sit anywhere, the window has to follow the *caret* — so it
    /// scrolls left as well as right, and one cell is always reserved so the
    /// caret at the end of a full line still has somewhere to be.
    pub(crate) fn viewport(&self, width: u16) -> (&str, u16) {
        if width == 0 {
            return ("", 0);
        }
        let width = usize::from(width);
        let count = self.text.chars().count();

        // The whole line fits: no window at all.
        if count < width {
            return (&self.text, to_col(self.caret));
        }

        // Otherwise keep the caret inside a window one cell short of the pane,
        // sliding only as far as it must. `visible` is that cell count.
        let visible = width - 1;
        let start = if self.caret <= visible {
            0
        } else {
            (self.caret - visible).min(count.saturating_sub(visible))
        };
        let from = char_index(&self.text, start);
        let to = char_index(&self.text, (start + visible).min(count));
        (&self.text[from..to], to_col(self.caret - start))
    }

    /// Put `text` in at the caret.
    ///
    /// Takes the whole string rather than a character at a time: winit documents
    /// a Windows dead key yielding **two** characters from one press, and
    /// inserting them one at a time at a moving caret would land them reversed.
    pub(crate) fn insert(&mut self, text: &str) {
        // **Truncated to what is left, not dropped whole.** Refusing the entire
        // insert when it would cross `MAX_INPUT` reintroduces the void this cap
        // exists to prevent: with 510 characters typed, a dead key yielding two
        // would do nothing at all, and a paste of any length over the remainder
        // would silently vanish rather than filling the line.
        let room = MAX_INPUT.saturating_sub(self.text.chars().count());
        let glyphs: String = text
            .chars()
            .filter(|c| orbs_render::is_renderable(*c))
            .take(room)
            .collect();
        if glyphs.is_empty() {
            return;
        }
        let at = char_index(&self.text, self.caret);
        self.text.insert_str(at, &glyphs);
        self.caret += glyphs.chars().count();
        self.end_search();
    }

    /// Delete the character before the caret.
    pub(crate) fn backspace(&mut self) {
        if self.caret == 0 {
            return;
        }
        let from = char_index(&self.text, self.caret - 1);
        let to = char_index(&self.text, self.caret);
        self.text.replace_range(from..to, "");
        self.caret -= 1;
        self.end_search();
    }

    /// Move the caret, clamped to the line.
    pub(crate) const fn left(&mut self) {
        self.caret = self.caret.saturating_sub(1);
    }

    /// Move the caret right, clamped to the end of the line.
    pub(crate) fn right(&mut self) {
        self.caret = (self.caret + 1).min(self.text.chars().count());
    }

    /// Caret to the start.
    pub(crate) const fn home(&mut self) {
        self.caret = 0;
    }

    /// Caret to the end.
    pub(crate) fn end(&mut self) {
        self.caret = self.text.chars().count();
    }

    /// Throw the line away.
    pub(crate) fn clear(&mut self) {
        self.text.clear();
        self.caret = 0;
        self.recalled = None;
        self.search = None;
        self.draft.clear();
    }

    /// Take the finished line, resetting everything, and remember it.
    ///
    /// `remember` is false for a bare digit answering §6's numbered prompt: it
    /// is an answer, not a phrasing, and recalling one later would submit it as
    /// a command and land in `ParseLog` as a failed phrasing — diluting the very
    /// metric `answering_is_not_counted_as_a_phrasing` exists to protect.
    pub(crate) fn take(&mut self, remember: bool) -> String {
        let finished = std::mem::take(&mut self.text);
        self.clear();
        if remember && !finished.trim().is_empty() {
            // Consecutive duplicates collapse — bash's `HISTCONTROL=ignoredups`.
            // Running `survey` five times should not cost five presses of `Up`.
            if self.history.last().map(String::as_str) != Some(finished.as_str()) {
                self.history.push(finished.clone());
                if self.history.len() > MAX_HISTORY {
                    self.history.remove(0);
                }
            }
        }
        finished
    }

    /// Walk back through history; `Up`.
    ///
    /// **Filtered by what is already typed**, which is fish's default and zsh's
    /// `history-beginning-search-backward`. Against a loop that runs
    /// `move / wield / siphon / purge` over and over, typing `wield ` and
    /// pressing `Up` should walk the wields — bash's positional walk is the
    /// worse of the two here.
    pub(crate) fn earlier(&mut self) {
        let anchor = self.anchor();
        let upto = self.recalled.unwrap_or(self.history.len());
        let Some(found) = self.history[..upto]
            .iter()
            .rposition(|line| line.starts_with(&anchor))
        else {
            return;
        };
        if self.recalled.is_none() {
            self.draft = self.text.clone();
        }
        self.recalled = Some(found);
        self.replace_with(self.history[found].clone());
    }

    /// Walk forward through history; `Down`. Past the newest, the draft returns.
    pub(crate) fn later(&mut self) {
        let Some(from) = self.recalled else {
            return;
        };
        let anchor = self.anchor();
        match self.history[from + 1..]
            .iter()
            .position(|line| line.starts_with(&anchor))
        {
            Some(offset) => {
                let found = from + 1 + offset;
                self.recalled = Some(found);
                self.replace_with(self.history[found].clone());
            }
            None => {
                self.recalled = None;
                let draft = std::mem::take(&mut self.draft);
                self.replace_with(draft);
                self.search = None;
            }
        }
    }

    /// The prefix this search is anchored to, taken once at the first `Up`.
    fn anchor(&mut self) -> String {
        self.search.get_or_insert_with(|| self.text.clone()).clone()
    }

    /// Swap the line's contents, caret to the end.
    fn replace_with(&mut self, text: String) {
        self.text = text;
        self.caret = self.text.chars().count();
    }

    /// Typing ends a history search — the anchor was the *old* prefix.
    fn end_search(&mut self) {
        self.recalled = None;
        self.search = None;
    }

    /// What the line would become if the ghost were accepted.
    ///
    /// **A chain, history first**, which is fish's order and zsh's default
    /// (`ZSH_AUTOSUGGEST_STRATEGY=(history)`, with completion opt-in and
    /// documented as slow). History is the cheap half and usually the right one:
    /// a twelve-command brew loop is a player repeating themselves, and what
    /// they are about to type is nearly always what they typed before.
    ///
    /// Only ever offered with the caret at the end — a suggestion trailing text
    /// the player is editing in the middle would sit in the wrong place.
    ///
    /// Called once per change rather than per frame; see
    /// [`Ghost`](super::input::Ghost) for what owns the result.
    pub(crate) fn ghost(&self, scene: &orbs_sim::parser::Scene, prompt_open: bool) -> String {
        if self.text.is_empty() || self.caret != self.text.chars().count() {
            return String::new();
        }
        if let Some(earlier) = self
            .history
            .iter()
            .rev()
            .find(|line| line.starts_with(&self.text) && line.as_str() != self.text)
        {
            return earlier[self.text.len()..].to_owned();
        }
        let completion = orbs_sim::parser::complete(&self.text, self.caret, scene, prompt_open);
        let common = completion.common();
        let typed = &self.text[completion.replaces];
        common.strip_prefix(typed).unwrap_or_default().to_owned()
    }

    /// Take a completion: replace the partial word with `insert`.
    fn apply(&mut self, replaces: core::ops::Range<usize>, insert: &str) {
        self.text.replace_range(replaces.clone(), insert);
        let upto = replaces.start + insert.len();
        self.caret = self.text[..upto].chars().count();
        self.end_search();
    }

    /// Tab: extend as far as every candidate agrees, then cycle.
    ///
    /// - **First press** extends to the longest common prefix, which is free
    ///   progress (GNU readline's `compute_lcd_of_matches`). A lone candidate
    ///   finishes with a trailing space and there is nothing left to choose.
    /// - **When extending adds nothing** — the candidates share no more than what
    ///   is already typed — it lists them and leaves the line alone. That is
    ///   bash's default, and the least surprising answer to someone who pressed
    ///   Tab to ask a question rather than to make a choice.
    /// - **Every press after that** puts the next candidate in the line,
    ///   wrapping. This is readline's `menu-complete`, and it is what makes the
    ///   list an answer rather than a dead end the player types their way out of.
    ///
    /// Returns the candidate list while a cycle is running, so the caller can
    /// show it with [`cycling`](Self::cycling) marking where the player is.
    pub(crate) fn tab(
        &mut self,
        scene: &orbs_sim::parser::Scene,
        prompt_open: bool,
    ) -> Vec<String> {
        if let Some(candidates) = self.advance_cycle() {
            return candidates;
        }

        let completion = orbs_sim::parser::complete(&self.text, self.caret, scene, prompt_open);
        if completion.is_empty() {
            // §6 forbids a bare error, and this is not even a failed command.
            self.cycle = None;
            return Vec::new();
        }
        let common = completion.common();
        let typed = self.text[completion.replaces.clone()].to_owned();

        if common.len() > typed.len() {
            // There is agreement left to spend. Take it and stop — a second Tab
            // re-enters here, finds nothing further shared, and starts cycling.
            let lone = completion.candidates.len() == 1;
            let mut insert = common;
            if lone {
                insert.push(' ');
            }
            self.cycle = None;
            self.apply(completion.replaces, &insert);
            return Vec::new();
        }

        // Nothing left to extend. One candidate means the word is already whole,
        // so finish it; more than one starts the cycle on the first of them.
        if completion.candidates.len() == 1 {
            self.cycle = None;
            let mut insert = completion.candidates[0].clone();
            insert.push(' ');
            self.apply(completion.replaces, &insert);
            return Vec::new();
        }

        // Arm the cycle over what is already typed, and list. The line is not
        // touched until the next press.
        let candidates = completion.candidates;
        self.cycle = Some(Cycle {
            start: completion.replaces.start,
            filled: typed,
            candidates: candidates.clone(),
            index: None,
        });
        candidates
    }

    /// Step a running cycle on, if there is one and the line still matches it.
    ///
    /// The guard is not paranoia. `Line` is edited from several places, and a
    /// stale `start` would splice a candidate into the middle of a word. Checking
    /// that what sits at the recorded span **is** the candidate that was put
    /// there makes a forgotten cancellation harmless — the cycle simply restarts
    /// rather than corrupting the line.
    fn advance_cycle(&mut self) -> Option<Vec<String>> {
        let cycle = self.cycle.as_mut()?;
        let span = cycle.start..cycle.start.checked_add(cycle.filled.len())?;
        if self.text.get(span.clone()) != Some(cycle.filled.as_str()) {
            self.cycle = None;
            return None;
        }

        let next = match cycle.index {
            None => 0,
            Some(index) => (index + 1) % cycle.candidates.len(),
        };
        let text = cycle.candidates.get(next)?.clone();
        cycle.index = Some(next);
        cycle.filled = text.clone();
        let candidates = cycle.candidates.clone();

        self.text.replace_range(span.clone(), &text);
        let upto = span.start + text.len();
        self.caret = self.text[..upto].chars().count();
        Some(candidates)
    }

    /// Which candidate the cycle has put in the line, for the list to mark.
    ///
    /// `None` while the list is merely being shown, which is the first press.
    pub(crate) fn cycling(&self) -> Option<usize> {
        self.cycle.as_ref().and_then(|cycle| cycle.index)
    }

    /// Abandon any Tab cycle. Anything that is not another Tab ends it.
    pub(crate) fn end_cycle(&mut self) {
        self.cycle = None;
    }
}

fn to_col(count: usize) -> u16 {
    u16::try_from(count).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_of(text: &str) -> Line {
        Line::typed(text)
    }

    #[test]
    fn a_short_line_is_shown_whole_with_the_caret_after_it() {
        let line = line_of("survey");
        assert_eq!(line.viewport(40), ("survey", 6));
    }

    #[test]
    fn a_long_line_scrolls_and_keeps_its_caret_on_screen() {
        // The failure this exists to prevent: past the pane width the text stops
        // appearing and `set_cursor` refuses an off-grid position, so the player
        // types into a dead line.
        let line = line_of("abcdefghij");
        let (visible, caret) = line.viewport(5);
        assert_eq!(visible, "ghij", "the tail must stay visible");
        assert_eq!(caret, 4, "the caret must stay inside the viewport");
        assert!(usize::from(caret) < 5);
    }

    #[test]
    fn the_caret_never_leaves_the_grid_at_any_length() {
        for length in 0..200usize {
            let line = line_of(&"x".repeat(length));
            for width in [1u16, 8, 72] {
                let (visible, caret) = line.viewport(width);
                assert!(caret < width, "len {length} width {width} caret {caret}");
                assert!(visible.chars().count() < usize::from(width) + 1);
            }
        }
    }

    #[test]
    fn a_zero_width_viewport_is_empty_rather_than_a_panic() {
        assert_eq!(line_of("survey").viewport(0), ("", 0));
    }

    #[test]
    fn a_multibyte_glyph_is_not_split() {
        // `░` is three bytes and one cell. Slicing the tail by byte offset would
        // panic mid-character; the viewport has to seek by `char_indices`.
        let line = line_of("░░░░░");
        let (visible, caret) = line.viewport(3);
        assert_eq!(visible, "░░", "one cell is reserved for the caret");
        assert_eq!(caret, 2);
    }

    #[test]
    fn an_over_long_insert_fills_the_line_rather_than_doing_nothing() {
        // The cap used to drop the whole insert, which reintroduces the void it
        // exists to prevent — a paste longer than the remaining room vanished.
        let mut line = Line::typed(&"x".repeat(MAX_INPUT - 2));
        line.insert("abcdef");
        assert_eq!(line.text().chars().count(), MAX_INPUT);
    }
}
