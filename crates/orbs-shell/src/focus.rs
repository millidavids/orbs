//! Which of the things on screen has the keyboard.
//!
//! One enum and one ordering, shared by both frontends, because the ordering is
//! a *rule* and a second copy of a rule is how the two builds come to disagree.
//! That is `shortcuts.rs`'s argument and this is the same shape.
//!
//! It moved here because the Bevy build spent four resources, four run
//! conditions and a four-term predicate on the question, and each term is a
//! place to forget: a surface added without its term does not fail loudly, it
//! types into an invisible prompt while the player looks elsewhere.
//! `orbs-tui`'s `surfaces.rs` had already solved it, so this is that solution
//! ported and shared rather than a third expression of it.
//!
//! What it does *not* decide is how a frontend throws a keystroke away. That
//! genuinely differs: a terminal delivers one key at a time to whoever is
//! asking, so declining is enough, where Bevy's `MessageReader` carries a cursor
//! per reader and a system that does not run leaves the keys queued to arrive at
//! once. §19 records that defect.

/// What each frontend can answer about its own surfaces.
///
/// Named fields rather than positional flags, so they cannot be transposed at a
/// call site. Both frontends fill this from state they already hold, and neither
/// stores a `Focus` — a stored verdict can go stale against what it came from.
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
    /// The orb's menu is up — `quit`.
    ///
    /// The first that is not about the tower: everything above it is a surface
    /// *inside* a game and this is the way out of one, so it wins the keyboard
    /// over all of them. See [`Focus::of`].
    pub menuing: bool,
    /// The manual is open — `manual`, from the menu.
    ///
    /// Above the menu, because it opens *over* it: the menu is the only thing
    /// that opens this and stays up behind it, so unlike every other pair here
    /// these two really are open at once and the order is a fact rather than a
    /// tie-break.
    pub reading_manual: bool,
}

/// The surfaces that can hold the keyboard, in the order they take it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Focus {
    /// The command line. The default, and where everything returns to.
    ///
    /// The prompt is the default and a surface must be *entered* with a word —
    /// the editor's `edit`, the weave's `ley`, the map's `wander`. §19: a
    /// surface that grabs the keyboard on open eats the first keystroke.
    #[default]
    Prompt,
    /// The manual, opened from the menu.
    ///
    /// First, above the menu, and for once the order is not a tie-break: the
    /// menu opens this and stays up behind it, so the two really are open
    /// together. Closing the manual gives the keyboard back where it came from.
    Manual,
    /// The orb's menu, opened by `quit`.
    ///
    /// First in the order, and the only one that is not a surface of the tower:
    /// the four below are places inside a game and cannot coexist, where this is
    /// the way out and is reached by a word typed at the prompt. Ordered first
    /// anyway, because if it ever did tie, the way *out* is what a player meant.
    Menu,
    /// A spell, opened by `scribe`.
    Editor,
    /// The progression screen, opened by `weave`.
    Weave,
    /// The archive's stacks, opened by `wander`.
    Maze,
    /// The transcript, opened by `unfurl`.
    ///
    /// There was a `Chant` above this — the menagerie's figure on the arrow keys,
    /// the fifth surface this module was built for. The menagerie is a logic
    /// puzzle now (§19) so the variant went, and the module stays, because the
    /// reason was never the chant: each surface is a term somebody forgets.
    Reading,
}

impl Focus {
    /// Who the next keystroke belongs to.
    ///
    /// Order matters only because it must be decided: none of these can be open
    /// at once, since the prompt is dead while any holds the keyboard, but a
    /// silent tie is the harder bug to find — so the newer surface never wins by
    /// accident.
    #[must_use]
    pub const fn of(open: Open) -> Self {
        if open.reading_manual {
            Self::Manual
        } else if open.menuing {
            Self::Menu
        } else if open.editing {
            Self::Editor
        } else if open.weaving {
            Self::Weave
        } else if open.walking {
            Self::Maze
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
    pub const fn takes_the_pane(self) -> bool {
        matches!(
            self,
            Self::Manual | Self::Menu | Self::Editor | Self::Weave | Self::Maze
        )
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
                    reading_manual: true,
                    ..Open::default()
                },
                Focus::Manual,
            ),
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
            reading: true,
            menuing: true,
            reading_manual: true,
        };
        // The one pair here that is not impossible: the manual is opened from
        // the menu and the menu stays up behind it, so these two really are open
        // together and this is a fact rather than a tie-break.
        assert_eq!(Focus::of(all), Focus::Manual);
        let all = Open {
            reading_manual: false,
            ..all
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
            Focus::Reading
        );
    }

    /// Reading is the one that shares the screen, and the three modes do not.
    #[test]
    fn only_the_modal_surfaces_take_the_pane() {
        assert!(Focus::Manual.takes_the_pane());
        assert!(Focus::Menu.takes_the_pane());
        assert!(Focus::Editor.takes_the_pane());
        assert!(Focus::Weave.takes_the_pane());
        assert!(Focus::Maze.takes_the_pane());
        assert!(!Focus::Reading.takes_the_pane());
        assert!(!Focus::Prompt.takes_the_pane());
    }
}
