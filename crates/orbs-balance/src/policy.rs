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
    /// The menagerie, sung correctly and in time.
    Chanting,
    /// Fight a siege the way `besieging` fights one, then let the next arrive.
    ///
    /// The fifth policy that must read the world, and it reads exactly what the
    /// shipped decision tree reads — `few`, `hurt`, `outnumbered` — because the
    /// point of the column is what the *loop* is worth, not what a cleverer
    /// player might manage.
    Besieging,
    /// Bind charms at the forge, reading the residue the way the table does.
    ///
    /// **The sixth policy that must read the world**, and it reads exactly what
    /// `forging` reads — the three columns' `lit` — because the point of the
    /// column is what the *loop* is worth rather than what a lucky guess is.
    ///
    /// **It is the only policy that spends the tower's one slot on something
    /// other than making a thing**, which is the whole of why the forge needed
    /// one: §10's scarcity for the domain is *"the buff's own lifetime, and the
    /// slot"*, and a claim about competing for the slot that nothing measures is
    /// a claim. It is also the instrument for every number this phase invented —
    /// `QUINTESSENCE_BASE`, `REGEN_TICKS`, `REGEN_PER_ROUND`, `EBBING_AT` and
    /// every `costs`/`lasts`/`takes` in `forge.toml` — each of which is
    /// documented as a placeholder *"and `orbs-balance` is what sweeps it"*.
    Imbuing,
    /// Earn a slot by hand, bind a spell, and then do nothing at all.
    ///
    /// **The only policy that measures the script engine**, which is the point
    /// of it: every other one issues its commands the way a player types them,
    /// [`Scrying`](Self::Scrying) included, so the whole of §8 — the per-step
    /// tick cost, `PATIENCE`, the wait on the production slot, a binding
    /// re-casting a spell that has run off the end — is invisible to this
    /// harness without it.
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
        Self::CHANTING,
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

    /// The lens, swept the way a spell sweeps it.
    ///
    /// **This is the number §19's pricing argument now rests on**, and it moved:
    /// the ward went from 360 codes to 1296 and lost the ratchet, the settle-lock
    /// and the per-socket tally with them, so the whole automated rate is a
    /// different quantity than the one the design was priced against. A sentence
    /// in a decisions log is not an instrument; this is.
    ///
    /// **It reads the deltas and nothing else** — `dial <socket>` bare, `probe`,
    /// and *did `aligned` rise, hold or fall* — because that is the whole of what
    /// `breaking` can ask. It does **not** deduce, so it is not a model of a
    /// player; a player averages 5.15 presses against this policy's ~12, and §19's
    /// *"the harness has no player"* is why the faster of the two is the one
    /// absent from this file.
    ///
    /// **A press takes no slot**, so unlike every policy above it this one never
    /// waits on the tower — which is the balance claim worth watching. A bound
    /// solver is meant to run *beside* a full brewing loop rather than compete
    /// with it, and a `cost` column that starts filling here is that claim
    /// breaking.
    ///
    /// **Read `landed` as presses, not solves.** A `probe` answers with
    /// `{quantity} aligned`, so every press whose figure placed at least one
    /// sigil satisfies [`drive`](crate::drive)'s structural test for finished
    /// work. Nothing in the tally can tell that from a yield without matching
    /// prose, which rule 6 forbids; the column is a diagnostic, and this is what
    /// it is diagnosing here.
    ///
    /// **It still issues its own commands**, so it measures the *loop* and not
    /// the runner — [`BOUND`](Self::BOUND) below is the one that pays §8's
    /// per-step tick. A real bound `breaking` reads ~0.18 against this policy's
    /// 0.268, and that gap is the interpreter.
    const SCRYING: Self = Self {
        name: "scrying",
        gloss: "the lens, swept one socket at a time as a bound spell sweeps it",
        setup: &["attend lens"],
        body: Body::Scrying,
    };

    /// The sanctum, solved by the cyclic rotation `holding` writes.
    ///
    /// **The one policy whose rate is not a constant**, and the reason is the
    /// domain: a course's height comes from how far the walls have slipped, and
    /// its cost is `2^n - 1`. So a tower this policy is keeping up with musters
    /// short courses and earns steadily, and one it is falling behind on musters
    /// tall ones and earns less per tick while it catches up. The column
    /// therefore measures *the loop against the drain*, which is the only
    /// question this domain has.
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

    /// The menagerie, sung correctly and on the beat.
    ///
    /// **It measures the ceiling, not a player.** The driver reads the aperture
    /// and waits for `until` to run out **off the model**, not off a reading —
    /// the circle publishes no `until` to the language any more — so every
    /// syllable is struck, which is what a policy is for. A person misses some
    /// and a solver misses none once concentration is bought, and the number
    /// here is the roof both are under.
    ///
    /// **The one policy that can make the tower *worse*.** Every other loop only
    /// earns; a chant sung badly wears the barrier, so a regression that broke
    /// the timing would show up here as a falling rate *and* as integrity
    /// draining, which is the pair worth watching. ROADMAP's Phase 5 scarcity
    /// note asks for exactly that: *"a sweep is part of the gate rather than an
    /// afterthought"*.
    ///
    /// It issues its own commands, like [`WARDING`](Self::WARDING) and unlike
    /// [`BOUND`](Self::BOUND), so it does not pay §8's per-step tick — and here
    /// that gap is not a detail: a real bound `chanting` cannot keep up at all
    /// until the weave grants a second step, which is the domain's whole
    /// progression hook (§19).
    const CHANTING: Self = Self {
        name: "chanting",
        gloss: "the menagerie, every syllable answered on the beat",
        setup: &["attend menagerie"],
        body: Body::Chanting,
    };

    /// The bailey, fought the way the shipped decision tree fights it.
    ///
    /// **The one policy whose rate is mostly not up to it.** Every other loop
    /// converts time into experience at a rate the loop controls; a siege
    /// converts it at a rate the *dice* control, so this column is a mean over
    /// however many sieges fit in the run rather than a property of the driver.
    /// Read it across seeds or conclude nothing — the archive's `stacks` carries
    /// the same warning for the same reason.
    ///
    /// **It is also the only policy that can lose.** A fallen siege still pays
    /// escrow (§11.5's floor), so the rate never goes to nought — which is the
    /// thing worth watching here: a regression that made sieges unwinnable would
    /// show as a rate that *halved* rather than one that vanished, and halving
    /// is easy to mistake for tuning.
    const BESIEGING: Self = Self {
        name: "besieging",
        gloss: "the bailey, fought as the shipped decision tree fights it",
        // **Stocked at setup, and that is the policy modelling a player.** A
        // driver with an empty arsenal reaches the `few` rung, is told *"there
        // is no troop in the arsenal"*, and asks again on the next tick for
        // ever — measured at 7195 costs in 7200 ticks and a rate of nought.
        // The shipped spell does not have that failure because `hold` sits
        // outside its ladder; the *policy* did, which is CLAUDE.md's *"anything
        // else means the loop has fallen out of phase with the tower"* arriving
        // exactly as documented.
        //
        // `debug_spawn` earns nothing, so this inflates no rate — it buys the
        // arsenal a player would have brewed, which is the thing the column is
        // supposed to be measuring the use of.
        setup: &[
            "attend bailey",
            "debug_spawn troop 60",
            "debug_spawn warding 60",
        ],
        body: Body::Besieging,
    };

    /// [`GRIND`](Self::GRIND)'s loop again, run by a spell instead of by hand.
    ///
    /// **The comparison is the measurement.** The two policies issue the same
    /// two commands for ever, so everything else about them is equal and the gap
    /// between their rates is *exactly* what automation costs: the per-step tick
    /// §8 charges, the tick a bound spell spends re-casting when it runs off the
    /// end, and nothing else. Pointed at any other loop the number would be a
    /// mixture of that and the loop's own shape.
    ///
    /// **That gap is what the language overhaul moves**, and it is why this
    /// exists before the overhaul rather than after it. §19 records the
    /// decision that a step still costs a tick, with the weave tree as the
    /// escape valve; both halves of that are claims about this number, and
    /// nothing in the harness could see it. The other six policies issue their
    /// commands the way a player types them, so `SCRIPT_BUDGET`, `PATIENCE` and
    /// the whole runner were unmeasured.
    ///
    /// **`empty mortar_and_pestle` is in the spell, not beside it.** A binding
    /// re-casts a spell that has run off the end, so a lapping spell has to
    /// clear its own byproduct or the second pass refuses the load and every
    /// pass after it complains about the same husks — the trap CLAUDE.md records
    /// against `first_light`, which is why the shipped spell cannot be used here.
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
