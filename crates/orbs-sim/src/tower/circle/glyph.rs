//! The circle's three glyphs, and the words a spell may ask the circle for.

use super::Humour;

/// One of the three places a humour is limned.
///
/// **The keystone takes what the other two answer**; `sunwise` and `widdershins`
/// each take two of the beast's three senses, and which two is drawn with the
/// beast. So the shape of the circle never changes and its wiring does, which is
/// the sanctum's jittered height one room over: a player who has learned *the*
/// circle has to go on reading *this* one.
///
/// # The names
///
/// `dexter` and `sinister` were the first pair — heraldry's own left and right —
/// and `dexter` scores 667 against both `pewter`, a lens sigil, and `enter`, an
/// `attend` synonym. The way round a circle is older than heraldry: `sunwise` and
/// `widdershins` are clean, and they say *which way round* rather than *which
/// hand*, which is the truer word for a ring. `crown` was the keystone's first
/// name and scores 800 against the shell's `cron`. §19 has the sweep.
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
/// **`glyph`, and `hand` was the first word.** It scores 750 against the
/// bailey's `band` and against `and`, which is a word every `if` line may hold.
pub const GLYPHS: &str = "glyph";

/// The group `for each humour` walks.
///
/// **`humour`, and `kind` was the first word.** It scores 750 against `bind` and
/// `find`, and `kin` prefixes `kindle`. A humour is also the older word for a
/// temper's parts, which is what a beast's temper is made of.
pub const HUMOURS: &str = "humour";

/// The reading on the circle: how many rows of the waiting beast's temper are lit.
///
/// **A fact about the temper, which the player can see** — never a fact about
/// the circle, and never a verdict on a glyph. It is what lets a spell that
/// *knows the logic* skip humours that cannot be the keystone: a temper lit on
/// one row of eight is a yoke or an eschew at the keystone, whatever the other
/// two are.
///
/// Always between one and seven while a beast waits, because a temper lit on
/// every row or none is refused when the table is built — so `raise_count` never
/// sees nought, and **its absence means no beast**, which is what `is empty`
/// answers.
///
/// **`fervour`, and two words came first.** `lit` is the forge's, scores 750
/// against `list` and 600 against `light`, and would have broken the naming
/// test's three pinned collisions. `choler` — the humour of temper — survived a
/// sweep of every constant in the crate and then failed the real one at 667
/// against the lens's `closer`, which is a word a spell asks one room over. A
/// scrape is not the sweep; `tests/naming.rs` is.
pub const FERVOUR: &str = "fervour";

/// The words a spell may ask the menagerie for.
///
/// Registered unconditionally in `tower::scene`, whether or not a beast waits,
/// because a spell compiles at **cast** — the sanctum's `readings()` shape.
///
/// **The humours are here because a glyph carries its humour as a reading**
/// (`if the keystone has heed`), which is the lens's socket carrying its sigil.
/// Nothing else is published: not how many rows balked, not which, and no
/// *closer* or *further* between calls. §19 records why — a climb on those
/// stalls on 37% of beasts, because one glyph can mask another, so publishing
/// them would invite a solver that does not end.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    let mut words: Vec<&'static str> = Humour::ALL.into_iter().map(Humour::word).collect();
    words.push(FERVOUR);
    words
}
