//! Which of the things on screen has the keyboard.
//!
//! One enum and one ordering, shared by both frontends, because the ordering is
//! a *rule* and a second copy of a rule is how the two builds come to disagree.
//! That is `shortcuts.rs`'s argument and this is the same shape.
//!
//! # Why it moved here
//!
//! The Bevy build spent four resources, four run conditions and a four-term
//! predicate on this question, and `shell/input.rs` had already written down the
//! ceiling: *"`wander` is the fourth and it is the last one that goes in here: a
//! fifth surface refactors this first."* The reason it is worth naming rather
//! than living with is in that same comment — **each term is a place to
//! forget**, and a surface added without its term does not fail loudly. It types
//! into an invisible prompt while the player is looking at something else, and
//! the characters arrive later.
//!
//! `orbs-tui` had already solved it, in `surfaces.rs`, whose own doc comment
//! points at the Bevy side and says so. So this is that solution **ported and
//! shared**, not a third expression of it: the terminal keeps its `Surfaces`
//! state machine and asks this for the verdict.
//!
//! # What this does *not* decide
//!
//! How a frontend throws a keystroke away. That genuinely differs and neither
//! way is wrong: a terminal delivers one key at a time to whoever is asking, so
//! declining is enough; Bevy's `MessageReader` carries a cursor per reader, so a
//! system that simply does not run leaves the keys queued and they all arrive at
//! once when it does. §19 records that defect — typing while the transcript was
//! being read, then pressing Escape, put every character into the prompt.

/// What each frontend can answer about its own surfaces.
///
/// **Named fields rather than positional flags**, so the four cannot be
/// transposed at a call site. Both frontends fill this from state they already
/// hold; neither stores a `Focus`, because a stored verdict is one that can go
/// stale against the state it was derived from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Open {
    /// A spell is being edited — `scribe`, then `edit`.
    pub editing: bool,
    /// The progression screen is up — `weave`.
    pub weaving: bool,
    /// The arrow keys are walking the stacks — `wander`.
    pub walking: bool,
    /// The transcript is scrolled back — `unfurl`, or `PgUp`.
    pub reading: bool,
    /// The arrows are answering a chant — `chorus`.
    pub chorusing: bool,
    /// The orb's menu is up — `quit`.
    ///
    /// **The sixth, and the first that is not about the tower.** Everything
    /// above it is a surface *inside* a game; this one is the way out of one, so
    /// it wins the keyboard over all of them — see [`Focus::of`].
    pub menuing: bool,
}

/// The surfaces that can hold the keyboard, in the order they take it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Focus {
    /// The command line. The default, and where everything returns to.
    ///
    /// **The prompt is the default and a surface must be *entered* with a
    /// word** — the editor's `edit`, the weave's `ley`/`mastery`, the map's
    /// `wander`. §19: a surface that grabs the keyboard on open eats the
    /// player's first keystroke.
    #[default]
    Prompt,
    /// The orb's menu, opened by `quit`.
    ///
    /// **First in the order, and the only one that is not a surface of the
    /// tower.** The five below it are places inside a game and cannot coexist;
    /// this is the way out of the game, and it is reached by a word typed at the
    /// prompt — so in principle it opens over nothing. It is ordered first
    /// anyway, because if it ever did tie, the way *out* is the answer a player
    /// meant.
    Menu,
    /// A spell, opened by `scribe`.
    Editor,
    /// The progression screen, opened by `weave`.
    Weave,
    /// The archive's stacks, opened by `wander`.
    Maze,
    /// The menagerie's figure, opened by `chorus`.
    ///
    /// **The fifth, and the one this module was built for.** `shell/input.rs`
    /// pre-committed to a `Focus` owner *before* a fifth surface arrived, and
    /// this is it — added as one variant and one field rather than a fifth term
    /// in a predicate nobody would remember to update.
    Chant,
    /// The transcript, opened by `unfurl`.
    Reading,
}

impl Focus {
    /// Who the next keystroke belongs to.
    ///
    /// **Order matters only because it must be decided.** None of these can be
    /// open at once — the prompt is dead while any of them holds the keyboard,
    /// so nothing can open a second — but a silent tie would be the harder bug
    /// to find, so the newer surface never wins by accident.
    #[must_use]
    pub const fn of(open: Open) -> Self {
        if open.menuing {
            Self::Menu
        } else if open.editing {
            Self::Editor
        } else if open.weaving {
            Self::Weave
        } else if open.walking {
            Self::Maze
        } else if open.chorusing {
            Self::Chant
        } else if open.reading {
            Self::Reading
        } else {
            Self::Prompt
        }
    }

    /// Whether the command line has the keyboard.
    #[must_use]
    pub const fn is_prompt(self) -> bool {
        matches!(self, Self::Prompt)
    }

    /// Whether something other than the command line has it.
    ///
    /// The question `shell/input.rs` actually asks, kept as a word so the call
    /// site reads as the thing it is guarding rather than as a comparison.
    #[must_use]
    pub const fn is_elsewhere(self) -> bool {
        !self.is_prompt()
    }

    /// Whether this surface takes the whole pane, transcript included.
    ///
    /// The menu, the editor, the weave screen and the maze are **deliberate
    /// modes** rather
    /// than layout accidents, and three things follow from it that are otherwise
    /// easy to get wrong: `prompt::paint` returns early for them, `F5`'s linear
    /// mirror does not run over them (§14 — a known hole, recorded rather than
    /// fixed), and a `play.sh` scenario must wait on a *screen* rather than on a
    /// command block, because the answer to the word has painted over the block.
    #[must_use]
    /// **The chant is not one of them, and that is the interesting case.**
    /// `wander` hides the transcript because the maze is too big to sit beside
    /// it; a figure is 42 columns and already draws beside one, so `chorus`
    /// takes the *keys* and nothing else. A player answering syllables can still
    /// read what the orb is saying about them, which is the thing `wander` gives
    /// up and would rather not.
    pub const fn takes_the_pane(self) -> bool {
        matches!(self, Self::Menu | Self::Editor | Self::Weave | Self::Maze)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_open_is_the_prompt() {
        assert_eq!(Focus::of(Open::default()), Focus::Prompt);
        assert!(Focus::of(Open::default()).is_prompt());
    }

    #[test]
    fn each_surface_takes_the_keyboard_from_the_prompt() {
        let cases = [
            (
                Open {
                    menuing: true,
                    ..Open::default()
                },
                Focus::Menu,
            ),
            (
                Open {
                    editing: true,
                    ..Open::default()
                },
                Focus::Editor,
            ),
            (
                Open {
                    weaving: true,
                    ..Open::default()
                },
                Focus::Weave,
            ),
            (
                Open {
                    walking: true,
                    ..Open::default()
                },
                Focus::Maze,
            ),
            (
                Open {
                    chorusing: true,
                    ..Open::default()
                },
                Focus::Chant,
            ),
            (
                Open {
                    reading: true,
                    ..Open::default()
                },
                Focus::Reading,
            ),
        ];
        for (open, want) in cases {
            assert_eq!(Focus::of(open), want, "{open:?}");
            assert!(Focus::of(open).is_elsewhere(), "{open:?}");
        }
    }

    /// The tie-break, pinned in the order the two frontends agreed on.
    ///
    /// This cannot happen — a surface is entered with a word and the prompt is
    /// dead while one is open — but *"a silent tie would be the harder bug to
    /// find"*, so the resolution is a fact rather than an accident of the `if`
    /// chain's order.
    #[test]
    fn an_impossible_tie_resolves_to_the_earlier_surface() {
        let all = Open {
            editing: true,
            weaving: true,
            walking: true,
            chorusing: true,
            reading: true,
            menuing: true,
        };
        // The way *out* wins, which is the one tie whose resolution a player
        // would have an opinion about.
        assert_eq!(Focus::of(all), Focus::Menu);
        let all = Open {
            menuing: false,
            ..all
        };
        assert_eq!(Focus::of(all), Focus::Editor);
        assert_eq!(
            Focus::of(Open {
                editing: false,
                ..all
            }),
            Focus::Weave
        );
        assert_eq!(
            Focus::of(Open {
                editing: false,
                weaving: false,
                ..all
            }),
            Focus::Maze
        );
        assert_eq!(
            Focus::of(Open {
                editing: false,
                weaving: false,
                walking: false,
                ..all
            }),
            Focus::Chant
        );
    }

    /// Reading is the one that shares the screen, and the three modes do not.
    #[test]
    fn only_the_modal_surfaces_take_the_pane() {
        assert!(Focus::Menu.takes_the_pane());
        assert!(Focus::Editor.takes_the_pane());
        assert!(Focus::Weave.takes_the_pane());
        assert!(Focus::Maze.takes_the_pane());
        assert!(!Focus::Reading.takes_the_pane());
        assert!(!Focus::Prompt.takes_the_pane());
        // The chant keeps the transcript, unlike the other three modes.
        assert!(!Focus::Chant.takes_the_pane());
        assert!(Focus::Chant.is_elsewhere(), "it still owns the keyboard");
    }
}
