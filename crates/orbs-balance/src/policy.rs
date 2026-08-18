//! What a synthetic player does, and why it is a policy rather than a script.
//!
//! DESIGN.md §19 draws the line this module sits on:
//!
//! > | **What the player chooses**, given readable state | A script encodes a
//! >   policy; `orbs-balance` sweeps policy against state | ✅ |
//! > | **How well the player executes** — speed, precision | The harness has no
//! >   player, so it cannot sweep anything | ❌ |
//!
//! So every policy here is something a player could also write as a spell, and
//! nothing here models a human's typing speed, reaction time or attention. A
//! command is issued when the tower is free to take it, which is the only timing
//! rule the game itself has.
//!
//! # The loops are the ones §11.5 and §19 already name
//!
//! These are not invented workloads. `clarity` is the flagship the 16-experience
//! threshold is derived from; `haste` is the chain §19 records as *"the fastest
//! experience in the game"* and flags as a risk; `grind` is the *"standing grind
//! loop free, at 1 experience per ~10 ticks for ever"* it flags beside it. The
//! harness exists to put numbers on those three sentences.

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
    /// **A cycle rather than a decision tree, and that is honest for these
    /// three.** The laboratory's recipes are deterministic and its byproducts
    /// are consumed by the same loop that makes them, so a policy that *looks*
    /// at the tower would reach the same line the cycle already names. Where
    /// that stops being true — a policy choosing between §10.1's two routes to
    /// the same draught — it wants a real guard, and that is what
    /// [`Body::Stacks`] already is.
    Cycle(&'static [&'static str]),
    /// Walk the archive's maze by Trémaux, then open another.
    ///
    /// The one policy that must read the world: a maze is generated per seed and
    /// no fixed sequence of `follow`s can solve two of them.
    Stacks,
}

impl Policy {
    /// Every policy the harness knows, in the order `list` prints them.
    pub const ALL: [Self; 5] = [
        Self::CLARITY,
        Self::DAMPED,
        Self::HASTE,
        Self::GRIND,
        Self::STACKS,
    ];

    /// Look one up by name.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|policy| policy.name == name)
    }

    /// The flagship: sage and salt to a potion of clarity, end to end.
    ///
    /// **16 experience over 94 ticks**, which is the number §11.5's first
    /// threshold is derived from rather than chosen against — grind 1, digest 2,
    /// grind 1, mix 4, distil 8. If this policy's rate ever stops matching a
    /// hand-played session, one of the two has drifted, and that is the whole
    /// point of the See-it line on this item.
    ///
    /// `empty mortar_and_pestle` between grinds is not tidying: a charged
    /// instrument refuses a second load, so a loop without it stalls on its
    /// second lap. CLAUDE.md records the same trap for a bound spell.
    ///
    /// **`kindle charcoal` is in the body, not the setup, and the first draft of
    /// this file had it the other way round.** A charcoal burns 600 ticks
    /// (`fuel.toml`) against a lap of roughly 120, so a fire lit once goes out
    /// five laps in and every heated stage refuses silently for the rest of the
    /// hour. The sweep read 0.079 against the design's 0.170 and looked like a
    /// balance finding; it was a policy nobody could play. This is the careless
    /// player — relight every lap, never damp — and [`DAMPED`](Self::DAMPED) is
    /// the attentive one beside it.
    const CLARITY: Self = Self {
        name: "clarity",
        gloss: "the flagship brew, relit every lap and never damped",
        setup: &["attend laboratory"],
        body: Body::Cycle(&[
            "kindle charcoal",
            "grind sage",
            "empty mortar_and_pestle",
            // **Scour before use, never after fouling.** The byproduct a stage
            // leaves is what blocks the *next* lap's load — §10.1's rule that a
            // tool fouled by the last brew must be cleared before use — so a
            // purge placed after the stage that dirtied it cleans an instrument
            // nothing is waiting on and still leaves lap two refused. The first
            // draft did it the other way round and the sweep read 0.098.
            "purge balneum_mariae",
            "digest ground-sage",
            "siphon balneum_mariae",
            "grind rock-salt",
            "empty mortar_and_pestle",
            "purge flask_and_rod",
            "mix sage-tincture with ground-salt",
            "distil clarified-draught",
            // `empty` takes the potion *and* the phlegm, so there is nothing
            // left for a purge to scour — one after this refuses every lap.
            "empty alembic",
        ]),
    };

    /// The same brew, played the way §10.1 says is better.
    ///
    /// **This policy exists to check a design claim, not to add a workload.**
    /// §10.1 argues that the athanor's two heated stages are *not adjacent* —
    /// the unheated `flask_and_rod` sits between the bath and the still — so the
    /// efficient play is **light → digest → damp → combine → relight → distil**,
    /// and §19 calls that *"the best argument yet that `stop athanor` is a real
    /// move rather than an end-of-script tidy"*, adding that the damping script
    /// should be *"meaningfully ahead over a session rather than trivially ahead
    /// over one brew."*
    ///
    /// That is a falsifiable sentence about a rate, and the harness is what
    /// falsifies it. Fuel is `Holding::endless`, so damping cannot show up as
    /// stock saved — it can only show up here, as ticks, or nowhere at all.
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
    /// that. **11 experience over 30 ticks** against clarity's 16 over 94 — and
    /// §19 records the ratio as a risk rather than a feature, because the
    /// flagship ought not to be the slow way to earn. Measuring it is the first
    /// thing this harness owes.
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
    /// **`follow`, never `Sim::walk`.** An arrow moves the reading immediately
    /// and consumes no tick, so a hand-walked maze costs no world time at all
    /// and its rate is unbounded; a spell issues one `follow` per tick and pays
    /// hundreds. §19's *"the harness has no player"* settles which of the two
    /// this measures.
    const STACKS: Self = Self {
        name: "stacks",
        gloss: "the archive's maze, walked by Trémaux as a spell would walk it",
        setup: &["attend archive"],
        body: Body::Stacks,
    };
}
