//! The circle's three glyphs, and the words a spell may ask the circle for.

use super::Humour;

/// One of the three places a humour is limned.
///
/// The keystone takes what the other two answer; `sunwise` and `widdershins`
/// each take two of the beast's three senses, drawn with the beast. So the shape
/// of the circle never changes and its wiring does, and a player who has learned
/// *the* circle still has to read *this* one.
///
/// The names: `dexter` scored 667 against the lens's `pewter` and against
/// `enter`, and `crown` 800 against the shell's `cron` — so the way round a ring
/// rather than which hand (§19).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Glyph {
    /// Takes what the other two answer. What the beast is held by.
    Keystone,
    /// The glyph on the sunwise side, over two of the senses.
    Sunwise,
    /// The glyph on the widdershins side, over two of the senses.
    Widdershins,
}

impl Glyph {
    /// All three, in the order a save and a spoken line list them.
    pub const ALL: [Self; 3] = [Self::Keystone, Self::Sunwise, Self::Widdershins];

    /// The word a player types and a spell writes.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Keystone => "keystone",
            Self::Sunwise => "sunwise",
            Self::Widdershins => "widdershins",
        }
    }

    /// Read a glyph back from its word.
    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|one| one.word() == word)
    }

    /// Where this glyph sits in [`ALL`](Self::ALL), and in every `[_; 3]` the
    /// circle keeps.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Keystone => 0,
            Self::Sunwise => 1,
            Self::Widdershins => 2,
        }
    }
}

/// The group `for each glyph` walks.
///
/// `hand` was the first word: 750 against the bailey's `band` and against `and`,
/// which every `if` line may hold.
pub const GLYPHS: &str = "glyph";

/// The group `for each humour` walks.
///
/// `kind` was the first word: 750 against `bind` and `find`, and `kin` prefixes
/// `kindle`. A humour is also the older word for a temper's parts.
pub const HUMOURS: &str = "humour";

/// The reading on the circle: how many rows of the waiting beast's temper are lit.
///
/// A fact about the temper the player can see, never a verdict on a glyph — so
/// a spell that knows the logic can skip humours that cannot be the keystone: a
/// temper lit on one row of eight is a yoke or an eschew there.
///
/// Always between one and seven while a beast waits, because a temper lit on
/// every row or none is refused when the table is built — so its absence means
/// no beast, which is what `is empty` answers.
///
/// Two words came first: `lit` is the forge's and scores 750 against `list`,
/// and `choler` survived a scrape of the crate's constants and then failed at
/// 667 against the lens's `closer`. A scrape is not the sweep; `tests/naming.rs`
/// is.
pub const FERVOUR: &str = "fervour";

/// The words a spell may ask the menagerie for.
///
/// Registered unconditionally in `tower::scene`, whether or not a beast waits,
/// because a spell compiles at **cast** — the sanctum's `readings()` shape.
///
/// The humours are here because a glyph carries its humour as a reading (`if the
/// keystone has heed`), as the lens's socket carries its sigil. Nothing else is
/// published: a climb on rows-balked stalls on 37% of beasts, since one glyph
/// can mask another (§19).
#[must_use]
pub fn readings() -> Vec<&'static str> {
    let mut words: Vec<&'static str> = Humour::ALL.into_iter().map(Humour::word).collect();
    words.push(FERVOUR);
    words
}
