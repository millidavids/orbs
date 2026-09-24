//! The manual, while it is open: which chapter, and how far down it.
//!
//! Nothing here draws — see [`paint`](super::paint) — and nothing here reads a
//! file. The book is handed in when the reader opens, the way the menu's
//! settings rows are.

use orbs_sim::content::Chapter;

/// What the reader is showing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Showing {
    /// The contents: every chapter, by name.
    #[default]
    Contents,
    /// One chapter, `at` lines down.
    Chapter {
        /// Which one, as an index into the book.
        at: usize,
        /// How far down it, in wrapped rows.
        ///
        /// Rows rather than paragraphs: a paragraph is several rows at one
        /// width and one at another, and only the reader knows the width.
        /// `paint` clamps on the way past, as `Editor::top` does.
        top: usize,
    },
}

/// The manual, open.
///
/// The book travels with it: the reader does not read a file, ask a `Sim` or
/// know what a chapter is made of. The frontend assembles one — `manual.toml`
/// plus `prose.toml`'s manual keys — and hands it over, which is
/// `Menu::show_settings`' rule one surface along.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reader {
    /// Every chapter, in the order the contents lists them.
    pub(super) book: Vec<Chapter>,
    /// Which page is showing.
    pub(super) showing: Showing,
    /// What is being typed at it.
    pub(super) command: String,
    /// A chapter it does not have.
    pub(super) complaint: Option<String>,
    /// How many rows the last paint could show at once.
    ///
    /// Written by the painter, the only thing that knows how tall the pane is.
    /// `Editor::scroll_to` is the precedent: a viewport that follows the
    /// content belongs to whoever measures it.
    pub(super) rows: usize,
    /// How many rows the chapter showing came to at the last paint.
    pub(super) height: usize,
    /// How far down the *contents* is.
    ///
    /// Not in `Showing::Contents`, so it survives reading a chapter: coming back
    /// to the top of the list is a book that loses your thumb. The contents used
    /// to be one screen by construction, until nineteen chapters and the 80x22
    /// floor silently hid the last five.
    pub(super) contents_top: usize,
    /// The chapter last wrapped, and at what width.
    ///
    /// A cache: the painter re-wrapped a two-thousand-word chapter sixty times a
    /// second while the reader sat still. Keyed on chapter and width, so a
    /// resize re-wraps and [`restock`](Self::restock) invalidates it with the
    /// book.
    wrapped: Option<Wrapped>,
}

/// One chapter, wrapped to one width.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Wrapped {
    /// Which chapter, as an index into the book.
    at: usize,
    /// The width it was wrapped to.
    width: u16,
    /// Its title, ready to draw.
    title: String,
    /// Its lines, wrapped. An empty entry is a blank row.
    rows: Vec<String>,
}

/// What the reader would like the shell to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Close the manual; put back whatever was there.
    Close,
}

impl Reader {
    /// A reader open on this book, at its contents.
    #[must_use]
    pub fn of(book: Vec<Chapter>) -> Self {
        Self {
            book,
            ..Self::default()
        }
    }

    /// Every chapter, in order.
    #[must_use]
    pub fn book(&self) -> &[Chapter] {
        &self.book
    }

    /// Wrap chapter `at` to `width`, reusing the last wrap when nothing moved.
    ///
    /// Returns the chapter's title, or [`None`] if the book has no such
    /// chapter — which `restock` can produce and the painter answers by drawing
    /// nothing.
    ///
    /// Called by the painter, the only thing that knows the width. See
    /// [`wrapped`](Self::wrapped) for why the answer is kept.
    pub fn wrap_for(&mut self, at: usize, width: u16) -> Option<String> {
        if let Some(had) = &self.wrapped
            && had.at == at
            && had.width == width
        {
            return Some(had.title.clone());
        }
        let entry = self.book.get(at)?;
        let mut rows: Vec<String> = Vec::new();
        for line in &entry.lines {
            if line.is_empty() {
                rows.push(String::new());
                continue;
            }
            rows.extend(orbs_render::Wrap::new(line, width).map(ToOwned::to_owned));
        }
        let title = entry.title.clone();
        self.wrapped = Some(Wrapped {
            at,
            width,
            title: title.clone(),
            rows,
        });
        Some(title)
    }

    /// The rows [`wrap_for`](Self::wrap_for) last produced.
    #[must_use]
    pub fn wrapped(&self) -> &[String] {
        self.wrapped.as_ref().map_or(&[], |had| had.rows.as_slice())
    }

    /// Swap the book under an open reader, keeping the reader's place.
    ///
    /// For `ORBS_CONTENT`, and what makes rule 6 true here: the book is
    /// assembled once at open, so a hot-reloaded `manual.toml` reached an
    /// already-open reader not at all.
    ///
    /// Kept by name, not by index, or a writer who adds or reorders a chapter
    /// while reading one is thrown into whichever chapter inherited the number.
    /// A chapter that is gone falls back to the contents.
    pub fn restock(&mut self, book: Vec<Chapter>) {
        if let Showing::Chapter { at, top } = self.showing.clone() {
            let name = self.book.get(at).map(|chapter| chapter.name.clone());
            self.showing = name
                .and_then(|name| book.iter().position(|chapter| chapter.name == name))
                .map_or(Showing::Contents, |at| Showing::Chapter { at, top });
        }
        self.book = book;
        // The wrap goes with the book it was made from, or the reader paints
        // the old chapter's rows under the new chapter's title.
        self.wrapped = None;
        // The pane re-measures on the next paint and `measured` clamps `top`
        // there, which is the only place the wrapped height is known.
    }

    /// What is showing.
    #[must_use]
    pub const fn showing(&self) -> &Showing {
        &self.showing
    }

    /// The chapter showing, if one is.
    #[must_use]
    pub fn chapter(&self) -> Option<&Chapter> {
        match self.showing {
            Showing::Contents => None,
            Showing::Chapter { at, .. } => self.book.get(at),
        }
    }

    /// How far down the chapter showing is.
    #[must_use]
    pub const fn top(&self) -> usize {
        match self.showing {
            Showing::Contents => 0,
            Showing::Chapter { top, .. } => top,
        }
    }

    /// What is typed at it.
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }

    /// The chapter it could not find.
    #[must_use]
    pub fn complaint(&self) -> Option<&str> {
        self.complaint.as_deref()
    }

    /// Add typed text to the line.
    pub fn type_text(&mut self, text: &str) {
        for glyph in text.chars().filter(|glyph| !glyph.is_control()) {
            self.command.push(glyph);
        }
    }

    /// Rub out the last character.
    pub fn backspace(&mut self) {
        self.command.pop();
    }

    /// Tell the reader what the pane could hold, and how tall the chapter came
    /// to.
    ///
    /// Called by the painter on its way past, and it also clamps: a chapter
    /// scrolled to the bottom then opened in a taller pane would otherwise show
    /// a page of nothing, the shape `Editor::scroll_to` also guards.
    pub const fn measured(&mut self, rows: usize, height: usize) {
        self.rows = rows;
        self.height = height;
        let furthest = height.saturating_sub(rows);
        match &mut self.showing {
            Showing::Chapter { top, .. } => {
                if *top > furthest {
                    *top = furthest;
                }
            }
            Showing::Contents => {
                if self.contents_top > furthest {
                    self.contents_top = furthest;
                }
            }
        }
    }

    /// How far down the contents is.
    #[must_use]
    pub const fn contents_top(&self) -> usize {
        self.contents_top
    }

    /// Run whatever is on the line.
    ///
    /// An empty line does nothing rather than complaining, as it does at the
    /// prompt and in the menu.
    pub fn enter(&mut self) -> Option<Outcome> {
        let typed = std::mem::take(&mut self.command);
        let typed = typed.trim().to_lowercase();
        self.complaint = None;
        if typed.is_empty() {
            return None;
        }
        if BACK.starts_with(typed.as_str()) || typed.starts_with(BACK) {
            return self.back();
        }
        // Prefix-matched, like the menu's words and the lengths — `content`
        // holds that no two chapters share a first letter.
        let Some(at) = self
            .book
            .iter()
            .position(|chapter| chapter.name.starts_with(&typed))
        else {
            self.complaint = Some(typed);
            return None;
        };
        self.showing = Showing::Chapter { at, top: 0 };
        None
    }

    /// Escape: out of a chapter, then out of the manual.
    ///
    /// One level at a time, which is what Escape means everywhere else in the
    /// game — the menu's pages, the weave's browsing mode, the editor's buffer.
    pub fn escape(&mut self) -> Option<Outcome> {
        self.complaint = None;
        self.command.clear();
        self.back()
    }

    /// One level up: a chapter to the contents, the contents to wherever the
    /// manual was opened from.
    const fn back(&mut self) -> Option<Outcome> {
        match self.showing {
            Showing::Chapter { .. } => {
                self.showing = Showing::Contents;
                None
            }
            Showing::Contents => Some(Outcome::Close),
        }
    }

    /// Scroll `rows` down, or up when `down` is false.
    ///
    /// Clamped to whatever is showing, so paging past the end stops at the end
    /// rather than scrolling into blank pane.
    ///
    /// The contents scrolls too, and used not to: *"one screen by
    /// construction"* was false the moment a sixteenth chapter was added, and
    /// `paint` simply broke out of the loop.
    pub fn scroll(&mut self, rows: usize, down: bool) {
        // `height == 0` means *not measured yet*, not *nothing to show*: a dump
        // runs its whole `ORBS_MANUAL` script before the single paint, so a
        // `<pgdn>` arrives before anything is wrapped, and clamping to nought
        // made it a no-op. A chapter always has lines (`content::manual` holds
        // that), so nought cannot mean an empty one; left unclamped,
        // [`measured`] pulls it back on the way past.
        //
        // [`measured`]: Self::measured
        let furthest = self.height.saturating_sub(self.rows);
        let measured = self.height > 0;
        let moved = |top: usize| {
            if down {
                let moved = top.saturating_add(rows);
                if measured { moved.min(furthest) } else { moved }
            } else {
                top.saturating_sub(rows)
            }
        };
        match &mut self.showing {
            Showing::Chapter { top, .. } => *top = moved(*top),
            Showing::Contents => self.contents_top = moved(self.contents_top),
        }
    }

    /// A screenful, for `PageUp` and `PageDown`.
    ///
    /// One row short of a screen, so the line you were reading at the bottom is
    /// at the top after a page — what a pager does, and what the transcript's
    /// own `page_step` does.
    #[must_use]
    pub const fn page(&self) -> usize {
        if self.rows > 1 { self.rows - 1 } else { 1 }
    }
}

/// The word that steps one level up, on either page.
///
/// Prefix-matched like the menu's, and it outranks a chapter — [`enter`] tests
/// it first, so a chapter sharing a prefix with it cannot be opened by that
/// prefix: `begin` shipped second on the contents page and typing `b` closed the
/// manual instead. It is `entering` now.
///
/// [`the_way_out_cannot_be_shadowed_by_a_chapter`] holds it, checking the
/// *whole* prefix rather than the first letter, because `ba` and `bac` shadowed
/// it too.
///
/// [`enter`]: Reader::enter
/// [`the_way_out_cannot_be_shadowed_by_a_chapter`]: tests::the_way_out_cannot_be_shadowed_by_a_chapter
pub(super) const BACK: &str = "back";

#[cfg(test)]
mod tests {
    use super::*;

    /// The real book, generated chapters and all.
    fn book() -> Vec<Chapter> {
        crate::manual::book(&orbs_sim::Sim::new(1))
    }

    #[test]
    fn the_way_out_cannot_be_shadowed_by_a_chapter() {
        // `enter` tests `BACK.starts_with(typed)` against what the *player
        // typed*, so a chapter is shadowed at every prefix of its name that is
        // also a prefix of `back`. Comparing the chapter's whole name instead
        // finds nothing, which made the first version of this test vacuous.
        for chapter in book() {
            let name = chapter.name.as_str();
            for len in 1..=name.len() {
                let typed = &name[..len];
                assert!(
                    !BACK.starts_with(typed),
                    "typing `{typed}` for `{name}` leaves the manual instead",
                );
            }
        }
    }

    #[test]
    fn every_chapter_opens_by_its_shortest_prefix() {
        // The promise the contents page makes by printing one-word names.
        // Asserted through `enter`, because that is the path a keystroke takes.
        let chapters = book();
        for (at, chapter) in chapters.iter().enumerate() {
            let first = &chapter.name[..1];
            let mut reader = Reader::of(chapters.clone());
            reader.type_text(first);
            assert_eq!(reader.enter(), None, "`{first}` asked to leave");
            assert_eq!(
                *reader.showing(),
                Showing::Chapter { at, top: 0 },
                "`{first}` did not open `{}`",
                chapter.name,
            );
        }
    }

    #[test]
    fn a_reloaded_book_keeps_the_chapter_you_were_reading() {
        // Rule 6's last mile. The book is assembled at open, so an edited
        // `manual.toml` reached an open reader not at all. Restocking has to
        // land the edit *and* keep the reader's place.
        let chapters = book();
        let mut reader = Reader::of(chapters.clone());
        reader.type_text("keys");
        reader.enter();
        let Showing::Chapter { at, .. } = *reader.showing() else {
            panic!("the chapter did not open");
        };
        let name = chapters[at].name.clone();

        // The writer adds a chapter *above* the one being read, so the index
        // moves and the name does not.
        let mut grown = chapters;
        grown.insert(
            0,
            Chapter {
                name: "zzz".to_owned(),
                title: "a new chapter".to_owned(),
                lines: vec!["and its text".to_owned()],
            },
        );
        reader.restock(grown.clone());

        let Showing::Chapter { at: now, .. } = *reader.showing() else {
            panic!("restocking threw the reader out of its chapter");
        };
        assert_eq!(
            grown[now].name, name,
            "restocking followed the index rather than the chapter",
        );
    }

    #[test]
    fn the_wrap_is_reused_and_is_thrown_away_when_it_must_be() {
        // A cache with two keys and one invalidation: a stale wrap must not
        // survive either of the two things that change it.
        let chapters = book();
        let mut reader = Reader::of(chapters.clone());

        let title = reader.wrap_for(0, 60).expect("a chapter");
        let narrow = reader.wrapped().len();
        assert!(narrow > 0, "wrapping produced nothing");

        // The same chapter at the same width is the same answer, not a re-wrap.
        assert_eq!(reader.wrap_for(0, 60).as_deref(), Some(title.as_str()));
        assert_eq!(reader.wrapped().len(), narrow);

        // A resize re-wraps: wider means fewer rows for the same prose.
        reader.wrap_for(0, 100);
        assert!(
            reader.wrapped().len() < narrow,
            "a wider pane produced the same number of rows",
        );

        // ...and a restock throws it away, or the reader paints the old
        // chapter's rows under the new chapter's title.
        reader.wrap_for(0, 60);
        reader.restock(chapters);
        assert!(
            reader.wrapped().is_empty(),
            "a reloaded book kept the old wrap",
        );
    }

    #[test]
    fn a_chapter_deleted_under_the_reader_falls_back_to_the_contents() {
        // A writer who deletes the chapter being read gets the list, not
        // somebody else's text.
        let chapters = book();
        let mut reader = Reader::of(chapters.clone());
        reader.type_text("keys");
        reader.enter();
        assert!(matches!(reader.showing(), Showing::Chapter { .. }));

        let without: Vec<Chapter> = chapters
            .into_iter()
            .filter(|chapter| chapter.name != "keys")
            .collect();
        reader.restock(without);
        assert_eq!(*reader.showing(), Showing::Contents);
    }

    #[test]
    fn a_word_it_does_not_know_is_named_rather_than_swallowed() {
        // §6: never a bare error, and never a silent no-op.
        let mut reader = Reader::of(book());
        reader.type_text("zorb");
        assert_eq!(reader.enter(), None);
        assert_eq!(reader.complaint(), Some("zorb"));
        assert_eq!(*reader.showing(), Showing::Contents);
    }
}
