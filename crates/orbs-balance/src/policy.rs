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
//! The loops are the ones §11.5 and §19 already name, not invented workloads:
//! `clarity` is the flagship the 16-experience threshold is derived from,
//! `haste` the chain §19 calls *"the fastest experience in the game"*, and
//! `grind` the *"standing grind loop free, at 1 experience per ~10 ticks for
//! ever"* it flags beside it. The harness puts numbers on those sentences.

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
    /// Bind charms at the forge, reading the residue the way the table does.
    ///
    /// The sixth policy that must read the world, and it reads exactly what
    /// `forging` reads — the three columns' `lit` — because the point is what
    /// the *loop* is worth rather than what a lucky guess is.
    ///
    /// The only policy that spends the tower's one slot on something other than
    /// making a thing, which is why the forge needed one: §10's scarcity for the
    /// domain is *"the buff's own lifetime, and the slot"*, and an unmeasured
    /// claim about competing for the slot is a claim. It is also the instrument
    /// for every number the phase invented — `QUINTESSENCE_BASE`, `REGEN_TICKS`,
    /// `EBBING_AT`, every `costs`/`lasts`/`takes` in `forge.toml`.
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

impl Policy {
    /// Every policy the harness knows, in the order `list` prints them.
    pub const ALL: [Self; 11] = [
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
    ];

    /// Look one up by name.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|policy| policy.name == name)
    }

    /// The flagship: sage and salt to a potion of clarity, end to end.
    ///
    /// 16 experience over 94 ticks, which is the number §11.5's first threshold
    /// is derived from — grind 1, digest 2, grind 1, mix 4, distil 8. If this
    /// rate stops matching a hand-played session, one of the two has drifted.
    ///
    /// `empty mortar_and_pestle` between grinds is not tidying: a charged
    /// instrument refuses a second load, so a loop without it stalls on lap two.
    ///
    /// `kindle charcoal` is in the body, not the setup, and the first draft had
    /// it the other way round: a charcoal burns 600 ticks against a lap of ~120,
    /// so a fire lit once goes out five laps in and every heated stage refuses
    /// silently. The sweep read 0.079 against the design's 0.170 and looked like
    /// a balance finding. This is the careless player;
    /// [`DAMPED`](Self::DAMPED) is the attentive one.
    const CLARITY: Self = Self {
        name: "clarity",
        gloss: "the flagship brew, relit every lap and never damped",
        setup: &["attend laboratory"],
        body: Body::Cycle(&[
            "kindle charcoal",
            "grind sage",
            "empty mortar_and_pestle",
            // Scour before use, never after fouling: the byproduct blocks the
            // *next* lap's load (§10.1), so a purge after the stage that dirtied
            // it cleans an instrument nothing is waiting on and still leaves lap
            // two refused. The first draft did it the other way and read 0.098.
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
    /// Checks a design claim rather than adding a workload. §10.1 argues the
    /// athanor's two heated stages are not adjacent — `flask_and_rod` sits
    /// between them — so efficient play is light → digest → damp → combine →
    /// relight → distil, and §19 wants that *"meaningfully ahead over a session
    /// rather than trivially ahead over one brew"*. Fuel is `Holding::endless`,
    /// so damping can only show up here, as ticks, or nowhere at all.
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
    /// The number §19's pricing argument rests on, and it moved: the ward went
    /// from 360 codes to 1296 and lost the ratchet, the settle-lock and the
    /// per-socket tally, so the automated rate is a different quantity than the
    /// one the design was priced against.
    ///
    /// It reads the deltas and nothing else — `dial <socket>` bare, `probe`, and
    /// which way `aligned` moved — because that is all `breaking` can ask. It
    /// does not deduce, so it is not a model of a player: a player averages 5.15
    /// presses against this policy's ~12.
    ///
    /// A press takes no slot, so unlike every policy above it this one never
    /// waits on the tower — which is the balance claim worth watching. A `cost`
    /// column that starts filling here is a bound solver competing with brewing
    /// rather than running beside it.
    ///
    /// Read `landed` as presses, not solves: a `probe` answers with
    /// `{quantity} aligned`, so any press placing one sigil satisfies
    /// [`drive`](crate::drive)'s structural test for finished work.
    ///
    /// It still issues its own commands, so it measures the *loop* and not the
    /// runner — [`BOUND`](Self::BOUND) pays §8's per-step tick. A real bound
    /// `breaking` reads ~0.18 against this policy's 0.268, and the gap is the
    /// interpreter.
    const SCRYING: Self = Self {
        name: "scrying",
        gloss: "the lens, swept one socket at a time as a bound spell sweeps it",
        setup: &["attend lens"],
        body: Body::Scrying,
    };

    /// The sanctum, solved by the cyclic rotation `holding` writes.
    ///
    /// The one policy whose rate is not a constant: a course's height comes from
    /// how far the walls have slipped and its cost is `2^n - 1`, so a tower this
    /// is keeping up with musters short courses and one it is behind on musters
    /// tall ones. The column measures *the loop against the drain*.
    ///
    /// It issues its own commands, like [`STACKS`](Self::STACKS) and
    /// [`SCRYING`](Self::SCRYING) and unlike [`BOUND`](Self::BOUND), so it does
    /// not pay §8's per-step tick. A real bound `holding` spends six or seven
    /// steps a haul against this one's one, and the gap is the interpreter.
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
    /// It does not deduce, so it is not a model of a player —
    /// [`SCRYING`](Self::SCRYING)'s rule, and it matters more here: a person who
    /// reads the temper holds a beast in one call, where this tries every circle
    /// and averages 68. What it prints is what a bound search is worth, which is
    /// what the economy sees (§19).
    ///
    /// It issues its own commands, so it does not pay §8's per-step tick for the
    /// spell's guards and loop entries — this is the search's ceiling at one
    /// command a tick, and a bound `taming` sits below it.
    const TAMING: Self = Self {
        name: "taming",
        gloss: "the menagerie, every circle tried in order until one holds",
        setup: &["attend menagerie"],
        body: Body::Taming,
    };

    /// The bailey, fought the way the shipped decision tree fights it.
    ///
    /// The one policy whose rate is mostly not up to it: a siege converts time
    /// into experience at a rate the *dice* control, so this column is a mean
    /// over however many sieges fit in the run. Read it across seeds or conclude
    /// nothing — `stacks` carries the same warning.
    ///
    /// Also the only policy that can lose. A fallen siege still pays escrow
    /// (§11.5's floor), so a regression that made sieges unwinnable shows as a
    /// rate that *halves* rather than vanishes — and halving is easy to mistake
    /// for tuning.
    const BESIEGING: Self = Self {
        name: "besieging",
        gloss: "the bailey, fought as the shipped decision tree fights it",
        // Stocked at setup, which is the policy modelling a player: an empty
        // arsenal reaches the `few` rung, is told *"there is no troop in the
        // arsenal"*, and asks again every tick — 7195 costs in 7200 ticks and a
        // rate of nought. `debug_spawn` earns nothing, so it inflates no rate;
        // it buys the arsenal a player would have brewed.
        setup: &[
            "attend bailey",
            "debug_spawn troop 60",
            "debug_spawn warding 60",
        ],
        body: Body::Besieging,
    };

    /// [`GRIND`](Self::GRIND)'s loop again, run by a spell instead of by hand.
    ///
    /// The comparison is the measurement: the two policies issue the same two
    /// commands for ever, so the gap between their rates is exactly what
    /// automation costs — §8's per-step tick, plus the tick a bound spell spends
    /// re-casting off the end. Pointed at any other loop it would be a mixture
    /// of that and the loop's own shape.
    ///
    /// That gap is what the language overhaul moves, which is why this exists
    /// before the overhaul. §19 records the decision that a step still costs a
    /// tick with the weave tree as the escape valve; both halves are claims
    /// about this number, and nothing in the harness could see it.
    ///
    /// `empty mortar_and_pestle` is in the spell, not beside it: a binding
    /// re-casts a spell that ran off the end, so a lapping spell must clear its
    /// own byproduct or every pass after the first complains about the same
    /// husks — the trap CLAUDE.md records against `first_light`.
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
