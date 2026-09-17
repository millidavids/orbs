//! The six humours a glyph can be limned with — the circle's logic gates.

/// What one glyph does with the two things it is given.
///
/// # Six, and all of them symmetric
///
/// These are the six two-input gates whose answer does not care which input is
/// which: AND, NAND, OR, NOR, XOR and XNOR. **Symmetry is what lets the circle
/// draw a glyph's two inputs without saying which is first**, and it is what
/// keeps the puzzle a question about *logic* rather than about wiring order —
/// an implication gate would make every glyph's two wires a second decision.
///
/// # The names, and why they are not the logician's
///
/// `and`, `or` and `not` are the spell language's own connectives
/// (`parser::question`), and `either` and `both` are its brackets — a humour
/// called `or` would be a word no `if` line could ever say. The rest collide
/// with each other by construction: `nand` is one edit from `and`, `nor` from
/// `or`, `xnor` from `xor`. So the canonical register is arcane, as §6 wants
/// anyway, and `recall <humour>` teaches the logician's name beside it.
///
/// **Each second word is the first one negated**: a yoke pulls only when both
/// pull, and to spurn the yoke is to answer whenever they do not; to heed is to
/// answer to either, and to eschew is to answer to neither; to oppose is to
/// answer when they differ, and to mirror is to answer when they agree. §19
/// records the sweep, and the words it rejected on the way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Humour {
    /// Lit when both are lit. What a logician calls AND.
    Yoke,
    /// Lit unless both are lit. NAND.
    Spurn,
    /// Lit when either is lit. OR.
    Heed,
    /// Lit when neither is lit. NOR.
    Eschew,
    /// Lit when exactly one is lit. XOR.
    Oppose,
    /// Lit when both agree. XNOR.
    Mirror,
}

impl Humour {
    /// Every humour, in the order a bare `limn` steps through them.
    ///
    /// **Each pair together**, so stepping a glyph passes a humour and then its
    /// negation — which is the order a player learning the six meets them in, and
    /// the one [`OPENING`](super::OPENING) starts at the head of.
    pub const ALL: [Self; 6] = [
        Self::Yoke,
        Self::Spurn,
        Self::Heed,
        Self::Eschew,
        Self::Oppose,
        Self::Mirror,
    ];

    /// The word a player types and a spell writes.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Yoke => "yoke",
            Self::Spurn => "spurn",
            Self::Heed => "heed",
            Self::Eschew => "eschew",
            Self::Oppose => "oppose",
            Self::Mirror => "mirror",
        }
    }

    /// Read a humour back from its word.
    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|one| one.word() == word)
    }

    /// What this humour answers, given its two inputs.
    #[must_use]
    pub const fn answer(self, one: bool, other: bool) -> bool {
        match self {
            Self::Yoke => one && other,
            Self::Spurn => !(one && other),
            Self::Heed => one || other,
            Self::Eschew => !(one || other),
            Self::Oppose => one != other,
            Self::Mirror => one == other,
        }
    }

    /// The humour after this one, wrapping — what a bare `limn <glyph>` does.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Yoke => Self::Spurn,
            Self::Spurn => Self::Heed,
            Self::Heed => Self::Eschew,
            Self::Eschew => Self::Oppose,
            Self::Oppose => Self::Mirror,
            Self::Mirror => Self::Yoke,
        }
    }

    /// This humour with its answer turned over.
    ///
    /// Used by the proofs and by nothing the world does: it is how a test states
    /// De Morgan without restating the truth tables.
    #[must_use]
    pub const fn negated(self) -> Self {
        match self {
            Self::Yoke => Self::Spurn,
            Self::Spurn => Self::Yoke,
            Self::Heed => Self::Eschew,
            Self::Eschew => Self::Heed,
            Self::Oppose => Self::Mirror,
            Self::Mirror => Self::Oppose,
        }
    }

    /// The humour that answers the same when both its inputs are turned over.
    ///
    /// **De Morgan, as a function.** `heed` over two things is `spurn` over their
    /// opposites, and `yoke` is `eschew`'s; `oppose` and `mirror` do not care. It
    /// is why no circle has one solution: negate both outer glyphs and swap the
    /// keystone for its dual, and every row answers as before.
    #[must_use]
    pub const fn dual(self) -> Self {
        match self {
            Self::Yoke => Self::Eschew,
            Self::Eschew => Self::Yoke,
            Self::Heed => Self::Spurn,
            Self::Spurn => Self::Heed,
            Self::Oppose => Self::Oppose,
            Self::Mirror => Self::Mirror,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepping_six_times_comes_back_and_meets_every_humour_on_the_way() {
        let mut seen = Vec::new();
        let mut at = Humour::Yoke;
        for _ in 0..Humour::ALL.len() {
            seen.push(at);
            at = at.next();
        }
        assert_eq!(at, Humour::Yoke);
        assert_eq!(seen, Humour::ALL, "a bare limn steps in a different order");
    }

    #[test]
    fn every_word_reads_back_as_its_humour() {
        for humour in Humour::ALL {
            assert_eq!(Humour::from_word(humour.word()), Some(humour));
        }
        assert_eq!(Humour::from_word("and"), None);
    }

    #[test]
    fn negation_and_duality_are_what_their_names_say() {
        for humour in Humour::ALL {
            for (one, other) in [(false, false), (false, true), (true, false), (true, true)] {
                assert_eq!(
                    humour.negated().answer(one, other),
                    !humour.answer(one, other),
                    "{humour:?} negated",
                );
                assert_eq!(
                    humour.dual().answer(!one, !other),
                    humour.answer(one, other),
                    "{humour:?}'s dual",
                );
            }
        }
    }

    /// **Symmetric, all six** — the property the circle's drawing rests on.
    #[test]
    fn no_humour_cares_which_input_is_which() {
        for humour in Humour::ALL {
            assert_eq!(humour.answer(true, false), humour.answer(false, true));
        }
    }

    /// Six different gates, not five and a repeat.
    #[test]
    fn the_six_answer_six_different_tables() {
        let tables: std::collections::BTreeSet<[bool; 4]> = Humour::ALL
            .into_iter()
            .map(|humour| {
                [
                    humour.answer(false, false),
                    humour.answer(false, true),
                    humour.answer(true, false),
                    humour.answer(true, true),
                ]
            })
            .collect();
        assert_eq!(tables.len(), Humour::ALL.len());
    }
}
