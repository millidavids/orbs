//! What a synthetic player does, and why it is a policy rather than a script.
//!
//! DESIGN.md §19 draws the line this module sits on:
//!
//! > | **What the player chooses**, given readable state | A script encodes a
//! >   policy; `orbs-balance` sweeps policy against state | ✅ |
//! > | **How well the player executes** — speed, precision | The harness has no
//! >   player, so it cannot sweep anything | ❌ |
//!
//! So every policy could be a spell, and none models human timing. The loops
//! are the ones §11.5 and §19 name — `clarity`, `haste`, `grind` — given
//! numbers.

/// A synthetic player, run against a real [`Sim`](orbs_sim::Sim).
#[derive(Debug, Clone, Copy)]
pub struct Policy {
    /// What to ask for on the command line.
    pub name: &'static str,
    /// One line saying what it is measuring.
    pub gloss: &'static str,
    /// Issued once, in order, before the body begins.
    pub setup: &'static [&'static str],
    /// What it does for ever after.
    pub body: Body,
}

/// How a policy decides its next command.
#[derive(Debug, Clone, Copy)]
pub enum Body {
    /// A fixed cycle, each line issued when the tower is free to take it.
    ///
    /// A cycle rather than a decision tree, which is honest for these three: the
    /// laboratory's recipes are deterministic and its byproducts are consumed by
    /// the loop that makes them, so a policy that *looked* at the tower would
    /// reach the same line. Where that stops being true it wants a real guard,
    /// which is what [`Body::Stacks`] is.
    Cycle(&'static [&'static str]),
    /// Walk the archive's maze by Trémaux, then open another.
    ///
    /// The one policy that must read the world: a maze is generated per seed and
    /// no fixed sequence of `follow`s can solve two of them.
    Stacks,
    /// Break a ward one socket at a time, then open another.
    ///
    /// The second policy that must read the world, and it reads **less** than
    /// the player does: only which way `aligned` moved on the last press, which
    /// is all `dev_spells.toml`'s `breaking` can ask for either.
    Scrying,
    /// Rotate a course of wards onto the last post, then muster another.
    ///
    /// The third policy that must read the world, and — like the second — it
    /// reads only what a spell can: the parity of the course it was handed, and
    /// which of two stations carries the lesser ward. Both come out of
    /// `Sim::pylon`, which is the same view the board draws from.
    Warding,
    /// Hold beasts at the menagerie's circle the way `taming` holds them.
    ///
    /// The fourth policy that must read the world, and it reads exactly what the
    /// spell reads — whether a beast is waiting, and where each glyph has been
    /// stepped to — because the point of the column is what the *search* is
    /// worth, not what a person who reads the table might manage.
    Taming,
    /// Fight a siege the way `besieging` fights one, then let the next arrive.
    ///
    /// The fifth policy that must read the world, and it reads exactly what the
    /// shipped decision tree reads — `few`, `hurt`, `outnumbered` — because the
    /// point of the column is what the *loop* is worth, not what a cleverer
    /// player might manage.
    Besieging,
    /// The flagship cycle, with a quickening supply kept open on the
    /// laboratory (§19, *Supply*).
    ///
    /// Against `clarity`, what quickening buys; `agrees.rs` weighs it against
    /// the lectern's cost. Scrolls arrive by `debug_spawn`.
    Quickened(&'static [&'static str]),
    /// [`Body::Stacks`] with a gleaning supply opened for every maze.
    ///
    /// What gleaning is worth, as fragments: run against `stacks`, the
    /// difference in fragments held is what the scrolls bought (§19,
    /// *Supply*).
    Gleaned,
    /// Bind charms at the forge, reading the residue the way the table does.
    ///
    /// Reads what `forging` reads. The one policy spending the slot on no
    /// product, measuring §10's scarcity and every number in `forge.toml`.
    Imbuing,
    /// Earn a slot by hand, bind a spell, and then do nothing at all.
    ///
    /// The only policy that measures the script engine, which is the point:
    /// every other one issues commands the way a player types them, so the whole
    /// of §8 — the per-step tick cost, `PATIENCE`, the wait on the production
    /// slot, a binding re-casting a spell that ran off the end — is otherwise
    /// invisible to this harness.
    Bound {
        /// A cycle, played by hand, until the orb can hold a spell.
        ///
        /// There is no way to grant experience: concentration is derived from
        /// work completed and no public API hands the sim a number, so a policy
        /// that wants a slot has to earn one exactly as a player does.
        earning: &'static [&'static str],
        /// What the spell is called.
        name: &'static str,
        /// What it holds, written the way a player would type it.
        lines: &'static [&'static str],
    },
}

/// One lap of the flagship brew, shared by every policy that runs it.
const CLARITY_LAP: &[&str] = &[
    "kindle charcoal",
    "grind sage",
    "empty mortar_and_pestle",
    // Scour before use, never after fouling: the byproduct blocks the *next*
    // lap's load (§10.1), so a purge after the stage that dirtied it cleans an
    // instrument nothing is waiting on and still leaves lap two refused. The
    // first draft did it the other way and read 0.098.
    "purge balneum_mariae",
    "digest ground-sage",
    "siphon balneum_mariae",
    "grind rock-salt",
    "empty mortar_and_pestle",
    "purge flask_and_rod",
    "mix sage-tincture with ground-salt",
    "distil clarified-draught",
    // `empty` takes the potion *and* the phlegm, so there is nothing left for a
    // purge to scour — one after this refuses every lap.
    "empty alembic",
];

impl Policy {
    /// Every policy the harness knows, in the order `list` prints them.
    pub const ALL: [Self; 13] = [
        Self::CLARITY,
        Self::DAMPED,
        Self::HASTE,
        Self::GRIND,
        Self::STACKS,
        Self::SCRYING,
        Self::WARDING,
        Self::TAMING,
        Self::BESIEGING,
        Self::IMBUING,
        Self::BOUND,
        Self::QUICKENED,
        Self::GLEANED,
    ];

    /// Look one up by name.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|policy| policy.name == name)
    }

    /// The flagship: sage and salt to a potion of clarity, end to end.
    ///
    /// 16 experience over 94 ticks, §11.5's first threshold. `empty` between
    /// grinds, or lap two stalls; `kindle` in the body, or the fire dies five
    /// laps in. The careless player; [`DAMPED`](Self::DAMPED) is attentive.
    const CLARITY: Self = Self {
        name: "clarity",
        gloss: "the flagship brew, relit every lap and never damped",
        setup: &["attend laboratory"],
        body: Body::Cycle(CLARITY_LAP),
    };

    /// [`CLARITY`](Self::CLARITY) with a quickening supply kept open.
    ///
    /// Stocked as it is wanted, so every scroll is fresh and the run counts
    /// what it was handed (`Run::given`).
    const QUICKENED: Self = Self {
        name: "quickened",
        gloss: "the flagship brew with a quickening supply kept open on it",
        setup: &["attend laboratory"],
        body: Body::Quickened(CLARITY_LAP),
    };

    /// [`STACKS`](Self::STACKS) with every maze set to gather.
    const GLEANED: Self = Self {
        name: "gleaned",
        gloss: "the archive's maze with a gleaning supply opened for every one",
        setup: &["attend archive"],
        body: Body::Gleaned,
    };

    /// The same brew, played the way §10.1 says is better.
    ///
    /// Checks §10.1's claim that damping between heated stages pays over a
    /// session (§19); with endless fuel it shows only as ticks.
    const DAMPED: Self = Self {
        name: "damped",
        gloss: "the same brew, damping between heated stages — §10.1's claimed better play",
        setup: &["attend laboratory"],
        body: Body::Cycle(&[
            "grind sage",
            "empty mortar_and_pestle",
            "purge balneum_mariae",
            "kindle charcoal",
            "digest ground-sage",
            "siphon balneum_mariae",
            "stop athanor",
            "grind rock-salt",
            "empty mortar_and_pestle",
            "purge flask_and_rod",
            "mix sage-tincture with ground-salt",
            "kindle charcoal",
            "distil clarified-draught",
            "empty alembic",
            "stop athanor",
        ]),
    };

    /// The short chain §19 flags as the fastest experience in the game.
    ///
    /// Grind sage for the husks, digest the husks to a weak tincture, distil
    /// that. 11 experience over 30 ticks against clarity's 16 over 94 — §19
    /// records the ratio as a risk, because the flagship ought not to be the
    /// slow way to earn.
    const HASTE: Self = Self {
        name: "haste",
        gloss: "the short chain §19 flags as the fastest experience in the game",
        setup: &["attend laboratory"],
        body: Body::Cycle(&[
            "kindle charcoal",
            "grind sage",
            "empty mortar_and_pestle",
            "purge balneum_mariae",
            "digest husks",
            "siphon balneum_mariae",
            "distil weak-tincture",
            "empty alembic",
        ]),
    };

    /// The mortar, for ever, on endless stock.
    ///
    /// §19: *"Endless base reagents make a standing grind loop free, at 1
    /// experience per ~10 ticks for ever."* The floor every other policy must
    /// beat, and the one that needs no fire.
    const GRIND: Self = Self {
        name: "grind",
        gloss: "the standing mortar loop on endless stock — the rate floor",
        setup: &["attend laboratory"],
        body: Body::Cycle(&["grind sage", "empty mortar_and_pestle"]),
    };

    /// The archive, walked the way a spell walks it.
    ///
    /// `follow`, never `Sim::walk`: an arrow moves the reading immediately and
    /// costs no tick, so a hand-walked maze has an unbounded rate, where a spell
    /// issues one `follow` per tick. §19's *"the harness has no player"* settles
    /// which of the two this measures.
    const STACKS: Self = Self {
        name: "stacks",
        gloss: "the archive's maze, walked by Trémaux as a spell would walk it",
        setup: &["attend archive"],
        body: Body::Stacks,
    };

    /// The lens, swept the way a spell sweeps it.
    ///
    /// §19's pricing number, reading only what `breaking` can: no deduction,
    /// ~12 presses against a player's 5.15. A press takes no slot, so a filling
    /// `cost` column means competing with brewing. `landed` counts presses.
    /// Its own commands, so the loop, not the interpreter: a bound `breaking`
    /// reads ~0.18.
    const SCRYING: Self = Self {
        name: "scrying",
        gloss: "the lens, swept one socket at a time as a bound spell sweeps it",
        setup: &["attend lens"],
        body: Body::Scrying,
    };

    /// The sanctum, solved by the cyclic rotation `holding` writes.
    ///
    /// The loop against the drain: a course's height follows the walls'
    /// slippage. Its own commands, as [`SCRYING`](Self::SCRYING), so no §8
    /// per-step tick.
    const WARDING: Self = Self {
        name: "warding",
        gloss: "the sanctum, rotated as the cyclic solver rotates it",
        setup: &["attend sanctum"],
        body: Body::Warding,
    };

    /// The forge, solved the way the shipped table solves it.
    const IMBUING: Self = Self {
        name: "imbuing",
        gloss: "the forge, reading the residue as the eight-rung table reads it",
        setup: &["attend forge"],
        body: Body::Imbuing,
    };

    /// The menagerie, searched the way `taming` searches it.
    ///
    /// No deduction, as [`SCRYING`](Self::SCRYING): 68 calls where a reader of
    /// the temper takes one — what the economy sees (§19). A ceiling over a
    /// bound `taming`.
    const TAMING: Self = Self {
        name: "taming",
        gloss: "the menagerie, every circle tried in order until one holds",
        setup: &["attend menagerie"],
        body: Body::Taming,
    };

    /// The bailey, fought the way the shipped decision tree fights it.
    ///
    /// The dice set the rate, so read it across seeds. The only one that can
    /// lose, and escrow pays a fall (§11.5), so unwinnable sieges halve the
    /// rate rather than zero it.
    const BESIEGING: Self = Self {
        name: "besieging",
        gloss: "the bailey, fought as the shipped decision tree fights it",
        // Stocked at setup, which is the policy modelling a player: an empty
        // arsenal is refused every tick it asks. `debug_spawn` earns nothing,
        // so it inflates no rate; it buys the arsenal a player would have
        // brewed. No troops: the bailey fights at its standing floor, the
        // menagerie being another column (§19, *Supply*).
        setup: &["attend bailey", "debug_spawn warding 60"],
        body: Body::Besieging,
    };

    /// [`GRIND`](Self::GRIND)'s loop again, run by a spell instead of by hand.
    ///
    /// Same commands, so the gap is what automation costs: §8's per-step tick
    /// and the re-cast (§19). `empty` in the spell, since a re-cast spell must
    /// clear its own byproduct.
    const BOUND: Self = Self {
        name: "bound",
        gloss: "the grind loop again, bound as a spell — what automation costs",
        setup: &["attend laboratory"],
        body: Body::Bound {
            earning: &["grind sage", "empty mortar_and_pestle"],
            name: "tending",
            lines: &["grind sage", "empty mortar_and_pestle"],
        },
    };
}
