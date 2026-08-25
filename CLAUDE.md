# O.R.B.S. — Project Documentation

## Project Overview

**O.R.B.S. — Operational Relic Bewitching System.** A wizard stares into his
scrying orb and finds a computer terminal inside it. A text-only game played
entirely through a fantasy command line: you tend a wizard's tower by navigating a
filesystem that *is* your duties, and you progress by writing scripts that teach
the orb to do your work. Sieges then test everything you automated — because the
enemy attacks the automation.

Artless by design. No sprites, no characters, no illustrations. A single curved
CRT glowing in the dark.

**Status: Phases 0, 0.5, 1 and 2 closed. Phase 3 (Spellcraft) next.** The
determinism spine, the Frame boundary, the parser, the cell renderer, brewing,
the archive, the lens, the tower rail, the balance harness, the spell engine and
its scripting language are built and the game plays.
See [docs/ROADMAP.md](docs/ROADMAP.md) for what remains.

**Phases 2–7 are §10's five remaining domains** — scrying, spellcraft,
enchanting, summoning, defense — and the phase that makes them one machine. The
siege moved from Phase 2 to **Phase 8**, because a series of puzzles has to exist
before the thing that consumes them. Phase numbers in older notes are six lower
from Phase 2 down; DESIGN.md §19 records the shift.

**The design is authoritative and lives in [docs/DESIGN.md](docs/DESIGN.md)** —
~2,000 lines, eight drafts, four independent staff-level reviews. Read it before
making design decisions. Do not re-litigate settled decisions; §19 is a decisions
log recording what was decided and why, including superseded choices marked as
such.

| Document | Purpose |
|---|---|
| [docs/DESIGN.md](docs/DESIGN.md) | The design. Authoritative for everything |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Phase status checklist — *derived* from DESIGN.md §15 |
| [docs/SETUP.md](docs/SETUP.md) | Toolchain, skills, build/test/CI procedures |
| [docs/CHANGELOG.md](docs/CHANGELOG.md) | Release notes — **and the source the GitHub Release, Discord and Bluesky all read from**. Its format is load-bearing; SETUP.md §4 has the rules |

## Technology Stack

- **Rust**, edition 2024, pinned toolchain
- **Bevy `=0.19.0`** — exact pin, upgraded deliberately (one window in Phase 9c)
- `serde` + `toml` (readable saves), `rand` 0.9 (seeded), `thiserror`, `tracing`,
  `clap` (harness), `bevy-steamworks`
- **No `bevy_text` / `bevy_ui`.** Every screen is terminal content rendered by our
  own cell-grid renderer. This is a build-level fact, not just an aesthetic.

Full stack detail: DESIGN.md §13.

## Workspace Layout

```
crates/
├── orbs-sim/       world model, parser, script engine, aberrations, threats
│                   — ZERO Bevy dependency, headless, fully testable
├── orbs-render/    Frame / cell-buffer, layout, semantic styling
│                   — consumed by every frontend, no backend deps
├── orbs-shell/     the shell both frontends share: painters, surfaces, the dump
│                   — orbs-sim + orbs-render + bevy_ecs; never `bevy`
├── orbs/           Bevy frontend: GPU cell renderer, CRT, audio, Steam
├── orbs-tui/       terminal frontend (second-class, cuttable)
└── orbs-balance/   CLI harness driving orbs-sim
```

**`orbs-shell` decides what a *screen* is; `orbs-render` decides what a *cell*
is.** Where the instrument panel goes, what the rail says about a room nobody is
standing in, what the editor does with Backspace — all of it takes a `Sim` and a
`Screen` and hands back a `Frame`. Both frontends draw that Frame and neither
decides any of it.

It needs `orbs-sim` **and** `orbs-render`, and neither may depend on the other in
that direction, so the painters had nowhere else to live. For four phases that
was fine because there was one frontend; §19 even rests part of its
editor-ownership argument on *"`orbs-tui` is ten lines with nothing to diverge
from."* The moment a second frontend is real, the choice is one shell or two that
disagree.

## Architectural Rules — do not break these

These are load-bearing. Each was arrived at through review and several were
mistakes caught late.

1. **`orbs-sim` depends on `bevy_ecs` only — never on `bevy` the engine.** The
   world model, components, and systems are ECS throughout; what is forbidden is
   rendering, windowing, assets, and anything requiring a GPU. `orbs-sim` must
   compile and test headlessly, in milliseconds, with no window. Balance sweeps,
   replay, determinism, and accessibility all depend on it.

   (`bevy_ecs` standalone is ~90 crates and adds no renderer; `bevy` is ~340 and
   adds all of it. Depend on the former.)
2. **`orbs-render` decides what appears and where; frontends decide only how a
   cell is drawn.** A frontend may add enrichment the other cannot reproduce (CRT,
   audio, fidelity tiers) **provided it carries no information absent from the
   Frame.**
3. **Determinism is architected.** Three parts:
   - Seeded RNG with **per-subsystem streams**, so adding an aberration roll
     cannot perturb the parser's stream.
   - `orbs-sim` owns its own `Schedule` and advances through a single explicit
     `step(&mut World, tick)` entry point that frontends call. The Bevy *app* and
     its plugin scheduler never drive the sim — a frontend is a caller, not a host.
   - **The sim schedule runs single-threaded**
     (`Schedule::set_executor(SingleThreadedExecutor::new())` — note
     `ExecutorKind` was removed in 0.19). Bevy's multi-threaded executor does not
     guarantee ordering between systems lacking explicit constraints, which is
     fatal for a sim that must replay identically and match offline to online. At
     1 Hz with this entity count, parallelism buys nothing and costs the property
     the whole architecture rests on.
4. **Structured records everywhere.** Commands emit records; presentation is a
   view over the record. Pipes, `sift`, the eldritch renderer, screen-reader
   linearisation, and the test harness all depend on this.
5. **All output is linear and semantic**, with presentation applied separately.
   Retrofitting this is impractical; designing for it is nearly free.
6. **Prose lives in hot-reloadable data files** (TOML/RON keyed by state), never
   as string literals in Rust. The writing budget is ~88k words; string literals
   would make it unmanageable by month 20.
7. **The Bevy frontend never waits for the terminal frontend.** Bevy is the Steam
   product.
8. **No `async` in `orbs-sim`. Ever.** Async introduces non-deterministic
   completion ordering and scheduler-dependent interleaving — precisely the two
   things that break replay, offline/online parity, and the balance harness
   matching the live game. `Sim::step()` taking `&mut self` makes re-entrancy and
   cross-thread driving statically impossible; keep it that way.

   Frontends may use whatever their backend requires — Bevy's task pools and
   render threads, `crossterm`'s event loop — but **never to drive the sim.** The
   sim is called from exactly one place, synchronously, per frame.

   Corollaries: no `tokio` anywhere (there is no networking; court_wizard's
   `tokio`/`iroh` are for its multiplayer). Long work goes on a worker thread, not
   an async runtime — offline catch-up is ~29k `step()` calls, which is
   milliseconds. File watching for `.spell` hot-reload uses `notify`'s background
   thread and a channel, with reloads queued to the next tick boundary.

## Code Conventions

Inherited from `court_wizard` (sibling repo, shipped) unless noted.

- **Feature-sliced modules.** Group by *concern*, one file per feature
  (`parsing.rs`, `escrow.rs`), not by file type. Prefer a `damage.rs` holding the
  component + system + constants together over splitting across
  `components.rs`/`systems.rs`/`constants.rs`.
- **`plugin.rs` does Bevy plugin registration ONLY.** System bodies and helpers
  go in sibling files.
- **`mod.rs` does `mod` declarations + `pub use` re-exports ONLY.** No logic, no
  constants, no types.
- **One `plugin.rs` per module.** No per-concern micro-plugins.
- **Files over ~300 lines must be split** unless every line is genuinely cohesive.
- **`styles.rs` is forbidden.** Constants live with their feature.
- **Visibility:** `pub(crate)` for shared items, `pub(in ...)` for subtree-shared,
  private `mod` for `plugin.rs`, `pub` only for Plugin types.
- **Messages, not Events.** `#[derive(Message)]` with a `Message` suffix.
- **Every `Update` system has a `run_if()` guard.** Never run unconditionally.
- **Check for reuse before writing new logic.** Extract shared patterns.
- `clippy.toml` sets `type-complexity-threshold = 500` (Bevy queries are verbose).

## Bevy 0.19 Specifics

**Bevy breaks APIs every minor release and most published Bevy code targets
0.14–0.16.** Do not recall Bevy syntax from memory.

Use the project skill **`bevy-idioms`** (in `.claude/skills/`), which is pinned to
0.19 and ships a compile-verified harness at
`.claude/skills/bevy-idioms/verify/` — run `cargo check` there to validate any
snippet against the pinned version.

Also available: `ecs-architecture` (engine-neutral design), `game-feel` (juice
patterns with verified Bevy snippets). See [docs/SETUP.md](docs/SETUP.md) for the
user-level `rust-skills` install.

## Git Workflow

**All work happens on `dev`. `main` is releases only.**

- **`dev` is the working branch.** Every commit lands here — features,
  corrections, docs, tooling. Push to `dev` freely (with approval); it triggers
  `dev-release.yml`, which is CLAUDE.md's gate and nothing else.
- **`main` is reached only by fast-forwarding `dev`**, never by committing to it
  and never by a merge commit. A push to `main` is a **release**: it tags the
  version, publishes a GitHub Release, and posts to Discord and Bluesky. There is
  no such thing as a quiet push to `main`.
- **A release is opt-in, and the opt-in is a changelog block.** `release.yml`
  reads `docs/CHANGELOG.md` first: no `## [v<version>]` block for the version in
  `Cargo.toml` means "ordinary work", and the run ends green having done nothing.
  This is what lets the per-step version bumps below reach `main` without
  announcing every step of a phase.
- **`/game-release` is how all three happen.** No argument folds the work into
  the open changelog block on `dev`; `consolidate` rewrites that block into the
  net change; `main` fast-forwards and promotes. Read
  `.claude/skills/game-release/SKILL.md` before doing any of it by hand.
- **Never commit or push without explicit approval.** Ask, then wait — and
  approval for one is not approval for the next.
- Commit messages: no AI/agent attribution of any kind — no `Co-Authored-By`,
  no generated-with footers, no session URLs.
- JIRA-style prefixes are not used on this project. A release commit is
  `v<version>: <what shipped>`.

## Working Practice

### The gate — run this after every step, not every phase

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo build -p orbs            # ← the game itself must LINK, not just check
```

**`cargo build -p orbs` is not optional and `cargo check` does not replace it.**
`check` stops at metadata; it will happily pass while the binary fails to link.
The Bevy dependency tree is ~126 crates and is the single most likely thing to
break on a toolchain or version change, so it must be exercised at every step
rather than discovered at the end of a phase.

### Finishing a step — tick the box, bump the version

**A step is not done until three things have happened together**, and they are one
action rather than three:

1. Its checkbox in [docs/ROADMAP.md](docs/ROADMAP.md) is ticked, with a **See it**
   line that works.
2. The workspace version in [Cargo.toml](Cargo.toml) is bumped.
3. Anything decided along the way is in DESIGN.md §19.

**The version is `0.<phase>.<step>` until release** — it tracks the roadmap, not
a public API, because there is no public API: every crate here is consumed only
by this workspace, so semver has nothing to describe yet. Completing a *step*
bumps the patch; completing a *phase* bumps the minor and resets the patch to
zero. DESIGN.md §19 records the rest, including that Phase 0.5 gets no minor of
its own and that the switch to ordinary semver at 1.0 is one-way.

```toml
[workspace.package]
version = "0.1.8"     # phase 1, step 8
```

All five crates inherit it (`version.workspace = true`), so there is exactly one
line to change. **It is player-visible**: `boot::screen` draws
`v{CARGO_PKG_VERSION}` on the POST card, which is what a tester quotes in a
report — so a stale version is a bug report pointing at the wrong build.

```bash
ORBS_DUMP=1 ORBS_BOOT=post cargo run -p orbs | grep 'v0\.'   # see it
```

**A correction folded into a step does not advance it.** The `✅` entries under
Phase 1 are items still being finished, not new steps; they bump nothing. What
advances the number is a box going from `[ ]` to `[x]`.

**Nothing asserts the number**, deliberately — whether an item is *done* is the
judgement the See-it gate exists to make, and a test pinning it would be pinning
that judgement. It is a habit, which is why it is written here next to the gate
rather than left to memory.

### Seeing it

Work is not done when it compiles. It is done when it has been *looked at*.

**Every roadmap item, in every phase, carries a "See it" line, and it has the same
standing as the gate above.** No item is done until a person can reach it from the
running game. An item without a See it line is not started; an item whose line
does not work is not finished, however green its tests are.

This rule exists because the first six Phase 0 items were built in
architectural-layer order and left ~10,000 lines that the binary called under 40%
of — a sixteen-command parser that had never received a keystroke. **Tests prove
code does what it was written to do; they cannot prove it is the code worth
writing**, and an API with no callers is unshaped.

The counter-example is the CRT, the one item player-gated on arrival: boot the
game, press F3. It flashed, that was obvious in ten seconds, and two wrong
diagnoses were falsified by *looking* rather than by reasoning. No test in the
suite would have caught it.

Corollaries:

- **Build the instrument before the thing it measures.** Where items can be
  ordered so earlier ones verify later ones, order them that way.
- **Anything already built without a gate gets one before new work continues.**
  Retroactive gating is a work item, not a cleanup.
- **Where a gate is genuinely impossible yet, name the phase that provides it**
  rather than inventing a debug affordance nobody will maintain.

DESIGN.md §15 and §19 record the correction and what it cost.

```bash
cargo run -p orbs                        # the game — window, sim, cell renderer
ORBS_DUMP=1 cargo run -p orbs            # ...its screen as text, no window, no GPU
ORBS_CAPTURE=1 cargo run -p orbs         # ...or as a screenshot
cargo run -p orbs-render --example screens   # real Frames dumped as text
cargo run -p orbs-balance -- sweep --hours 1 --why   # the economy, measured
```

The `screens` example renders the §4 boot report and a multiplexed siege through
the same public API the frontends use. **It has already found bugs that the full
test suite did not** — an em-dash in DESIGN.md's own boot text that CP437 cannot
draw, and pane content eating a border because a sub-painter was not established.
Add a screen to it whenever a new surface is built.

### The world sabotage surface — a reagent that is not what it says

§8.1's second of four surfaces. A base reagent is **substituted**: its name
changes and its identity does not, so a spell that named it stops working.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_swap; survey dispensary; \
  verify dispensary; verify laboratory" cargo run -p orbs
```
```text
charcoal- = ∞   rock-salt = ∞   sage = ∞     ← the odd one out, on screen
verify dispensary tampered something here is not what it says: charcoal-
verify laboratory sound                      ← one level, deliberately
```

**Endless base stock only, and never fuel.** A swap that could reach
`ground-sage` mid-pipeline destroys work in flight rather than misdirecting — and
it announced itself by breaking a *pipeline* test that has nothing to do with
sabotage, which is how a nuisance that reaches too far shows up. Charcoal was the
same mistake wearing the endless flag: it is named by no recipe, so swapping it
stops every heated stage in every domain rather than one spell.

**`debug_swap` above is the tester's shortcut, and the *ambient* half is what
breaks.** The system runs at one swap an hour and no dump can reach it, which is
how three defects shipped with every See-it line and every test green. They are in
DESIGN.md §19; the gate is a sweep and a test, and the sweep is the one that found
them:

```bash
cargo test -p orbs-sim --test tampering        # settles; never takes the fire
cargo run -p orbs-balance -- sweep --ticks 7200
```

**A swept rate is the only instrument that sees an ambient nuisance at all.** The
one that shipped took the alphabetically-first endless pile — always `charcoal` —
and never expired, so clarity read **0.074 against its pinned 0.140** and a
standing grind loop died for the rest of the session. `--ticks 7200`, not
`--hours 1`: an hour is short enough that one badly-timed swap flags the seed.

**A lie settles back to the truth on its own, and that is a fact about the spell
language.** A spell names things with literals, so it can never `purge sage-` — a
word nobody knew when it was written. `verify` → `purge` is therefore human-only,
and without an expiry an unattended tower had no path back at all. `purge` still
repairs it instantly, which is what keeps finding one worth doing.

**`drift` and `substitution` are two systems, not one branch.** Both draw once
per tick from `RngStream::Threat`, so interleaving them would make which surface
is hit depend on prior draws and invalidate every existing replay. The second is
**appended** to the schedule, never inserted — the same rule a stream index has.
`settling` is appended after both and draws **nothing**: it is a clock reading, so
it cannot perturb the stream. A second draw taken only *when* a swap fires would —
which is why the pile is chosen from the quotient of the roll that already fired.

### The lens — Mastermind, and the orb deduces nothing

**Four sigils of six, repeats allowed, 1296 codes.** `probe` opens a reading if
none is open and presses the aperture; `dial <socket> <sigil>` turns one dial,
`dial <socket>` steps it round the six. **All of it is instant and none of it
costs the tower anything** — a press was twelve ticks of the one production slot,
which is ROADMAP's *"a read is not a brew"*, and that scarcity is withdrawn
(§19). So `meditate` after a probe is no longer needed anywhere, and a bound
solver runs beside a full brewing loop with no contention at all.

**It was 360 codes with no repeats, and everything wrong with the domain came out
of that** (§19, `0.3.22`). With no repeats a socket often cannot take the sigil
you want because another holds it, so a dial had to *exchange* the two — and a
dial that moves two sockets makes `aligned` rising unattributable, which is why
the ward needed a **ratchet**, a **settle-lock** and a per-socket **`untried`**
count to be solvable at all. All three answer *is this position correct?*, which
is the one question a codemaker never answers. They are gone.

**The orb keeps no candidate set and says nothing about any one socket.** That is
the rule the whole domain rests on: Mastermind's entire difficulty is the
bookkeeping, and a machine gets bookkeeping for free.

**Two channels onto one ward, and the spell's is strictly weaker.** A player
reads `aligned` and `astray` as **numbers** and deduces; a spell reads only which
way each moved — `closer`/`level`/`further` and `richer`/`unchanged`/`poorer`. The
deltas are derivable from the counts and not the reverse. **5.15 presses against
11.93.**

```bash
# A ward, pressed and dialled. `survey` both kinds of reading.
# **No `meditate`**: a press answers on the tick it is typed (§19).
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; \
  dial second borax; probe; survey prism; survey second" cargo run -p orbs
```
```text
prism    aligned = 0   astray = 1   level   spent = 2
         steady                     ← the second delta: astray did not move either
second   borax                      ← what is in it. that is the whole answer
```

**`survey second` says `borax` and nothing else, and that is the point.** It used
to add `loose`/`settled` and two tallies; a socket now reports what you put in it,
because whether it is *right* is your bookkeeping.

**A dial moves one socket and only one, and a sigil may sit in two.** This is
what repeats bought and it is the thing to check when touching `seat`:

```bash
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; dial first alum; \
  survey first; survey second" cargo run -p orbs   # both read `alum`
```

**A press stands.** `aligned` falling is now observable, and it is the single most
informative thing the lens says: only a socket that *was* right can make it drop
when it alone moved. That is what the restore rung in `breaking` reads.

**There is no `meditate` in a lens See-it line any more.** Every one of them used
to carry `meditate 13` — one tick past a twelve-tick press — and a press is instant
now. A dump that still waits is testing the wait.

**`scry` is not a verb and `seat` is not either.** `scr` reaches `scribe` and
`sea` reaches `sift`'s `search`, and `tests/naming.rs` allows no prefix
exemption. The domain is still scrying; the words are `probe` and `dial`.

**`tests/naming.rs` sweeps *readings* now, and it failed on its first run.** A
reading is a `NounKind::Sense` and `NounKind::Any` reaches one, so a reading sits
in the way of every room's vocabulary — §19 records `purge grind` fuzzy-matching
`gained`. Nothing swept them until `0.3.22`; five collisions turned up at once.
Two were the ward's new astray triple and are renamed (`fuller`→`richer`,
`steady`→`unchanged`). **The other three were live and are fixed in the resolver
rather than by renaming** — `purge walk` echoed `purge wall`, and `recall edit`
explained the maze's way out to somebody asking about the spell editor.

**Two widenings close the class**, both of rules that already existed:
`Scene::knowing` holds **every verb word**, so a word the game knows can never
fuzz into a noun; and every **one-word synonym** is a `NounKind::Command`, so
`recall walk` reaches `follow`'s page. Reading-vs-reading is still the worse
collision and is still only caught by looking — `thicker`/`thinner` scores 715.

**A known word may still *abbreviate*, and that is the trap.** Adding the verb
words to `knowing` made `invoke check` unreachable — `check` is one of `verify`'s
words and the spell is `check.spell` — and six spell tests went red at once.
Prefixing is not fuzzing, so `Scene::candidates` exempts a phrase that *starts*
the noun's name. Touch `knowing` and run `cargo test -p orbs-sim --lib
tower::spell` before believing it.

**The board draws whenever a reading is open**, not when a word is typed — the
map's rule, and what makes a bound solver watchable. Columns never rows, and it
splits *after* the instrument panel.

```text
┌ ward ─────────────────────────────────┐
│    first second  third fourth  answer │   ← the words `dial` takes
│ 1    ☼      ○      ♂      ♀   ○ ○     │   ← a press: the figure, then its pegs
│ 2    ☼      ♦      ♂      ♀   • ○ ○   │
│ 3    ♦      ♦      ♂      ♀   • ○ ○ ○ │   ← a sigil twice, and four pegs
│───────────────────────────────────────│
│ →    ♦      ♦      ♂      ♀           │   ← the aperture: what goes next
│                                       │
│ ☼ nitre   ○ alum    ♂ borax           │   ← without this, `♦` is unnameable
│ ♀ quartz  ♦ pewter  ♠ ochre           │
└───────────────────────────────────────┘
```

**The aperture row carried `■ · · ·` and does not.** Those were the sockets the
orb had proved right; the trailing blanks are the peg column, kept so every row
is the same shape and a reader can compare two presses down a column.

**39 cells, and it was 10** (§19). The compact version was correct and unreadable:
nothing said which column was which socket, so a player counted along the row
before typing `dial second borax`, and nothing anywhere said `♦` was `pewter`. Both
name tables come from the sim through `Ward::view` — they are content, and
`orbs-render` may not depend on `orbs-sim`.

**The gutter counts the real press, not the row.** The sheet caps at the last
twelve; numbering those `1..12` would say a long solve had just begun. It still
fits the 80×22 floor beside a transcript, and still refuses whole rather than
truncating. Twelve is chosen against hand play — the worst deducing solve over
all 1296 codes is nine presses, so a person's whole reading always fits.

**Six glyphs, not six colours** (§14): the tint is enrichment and the shape
carries the identity, so the sheet reads in a dump. `▪` is not in CP437 and was
the first choice; `■` is. It is spoken **once, as a summary** — a reader hearing
twenty cells read out would get box-drawing noise.

**A broken seal spills the far wizard's log**, quiet — twelve lines into
`lens.log`, one sentence on the transcript. Every line is generated from a real
`(instrument, verb, material)` triple, so it never names an operation that
cannot happen.

```bash
# `debug_ward` makes the answer whatever the aperture holds, so the next press
# breaks it — a See-it line for the *spill* should not open with forty dials.
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; \
  debug_ward; probe; peruse lens.log" cargo run -p orbs
```
```text
 5 mix phlegm            ← somebody else's laboratory
10 distil dregs          ← a recipe you do not have. that is the hint, not a leak
16 digest ground-mugwort
```

**Three recipes are `secret = true` and have to be found.** `debug_learn` takes
the next one in the file's order without waiting on the roll; `debug_learn
<name>` takes a named one and refuses anything that is not a secret.

```bash
ORBS_BOOT=0 ORBS_DUMP="recall mending; debug_learn; recall mending" cargo run -p orbs
```

**`debug_learn` skips the roll, so it is not a gate for the *discovery* — and the
whole loop the domain exists for needs two hours and a bound solver.** There is
deliberately no way to force a find (`debug_ward` takes no argument), so this is
the only line that shows the log spelling a recipe out, the orb writing it down,
and the rail raising its mark. It is slow and it is the real thing:

```bash
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; debug_spell breaking" \
  ORBS_THEN="invoke breaking; meditate 3600; meditate 3600; \
  sift mending lens.log; recall mending; attend laboratory" cargo run -p orbs
```
```text
probe 2. potash -> mending + phlegm  (alembic, 30t, needs heat) prism
2 probe mending prism and a way to make mending. the orb writes it down
mending: 2 steps, 36 ticks          ← `recall` answers now, and refused before
│lens         !│                    ← the rail's news mark, from another room
```

**The mark is the one part of this that no *short* dump can reach**, because the
chance starts at 4% and climbs — so a dump that solves one ward and looks for a
`!` finds nothing and is not broken. `Mark::News` is set only on a find, which is
the ask: a solve is ordinary, a discovery is news.

**Unmakeable, unnameable and unreadable, all flipping on the same tick** — and
the third is the one that hides. `Topics` is snapshotted once at construction, so
a secret's `recall_` page is in that snapshot from tick 0; the gate is a set
subtraction in `scene_at` and nowhere else. Gate it at snapshot time and `recall
<secret>` answers early with every test green.

**Only three questions consult `Learned`**: does a recipe fire, is it part-way to
firing, and is its product a word the player can say. **Leave everything else
unfiltered** — `execute::scroll`'s verdant unlock derives base reagents as *"in
the vocabulary and made by nothing"*, so a filtered `Recipes::outputs` would
offer an undiscovered potion as an inexhaustible herb.

**`debug_spell breaking` is the solver — one sweep, four socket phases.**

```
probe
if the prism is working              ← ...and the same for second, third, fourth
dial first
probe
if the prism has further             ← it was already right
dial first nitre                     ← restore, and re-press to re-sync
probe
else
repeat until not the prism has level
dial first
probe
end
end
end
```

**`dial <socket>` with no sigil is what makes that possible** (§19). A spell has
no variables, so it cannot name the sigil a socket has not tried — bare, the ward
steps it round. **It was 24 rungs, then 4, and it is 4 phases of 3.**

**Termination is a proof, not a budget.** One socket moves, so `aligned` can only
change because of it; stepping it cyclically reaches the code's sigil within five
and says so on arrival. Over all 1296 codes: **11.93 presses, worst 21**, against
a deducing player's 5.15 and blind guessing's 648.

**The restore rung is worth 1.9 presses** — a socket already right pays two turns
instead of six — and **the re-press after it is load-bearing**: the deltas are
measured against the *previous press*, so a restore with no press leaves the next
phase comparing against a figure that was never sent. 924 of 1296 without it.

Four traps, all worth knowing before writing another lens spell:

- **`is empty` does not mean *no ward*.** `spell::watch` answers `empty` by
  asking whether a node has *children*, and a prism's children are its readings —
  none until the first press lands. An open ward reports **`working`**.
- **`until not ... has level`, never `until ... has closer`.** A ward that gives
  mid-sweep publishes *nothing*, so `has closer` answers a flat no and the loop
  goes round for ever — probing a fresh ward open and dialling into it from the
  middle of a sweep. `level` is what the walk waits to stop seeing, and its
  **absence** stops the loop the same way its answer does. Found by running it.
- **Wrap each phase in `if the prism is working`** for the other half of that:
  the last socket to arrive can be the second, and every phase after it would be
  dialling at nothing. One solve in six.
- **`has closer`, not `is closer`.** `Condition::Is` takes a closed three-word
  vocabulary — idle/free/still, working/busy/running, empty/bare — so a *reading*
  is always asked for with `has`.

**`MAX_MEDITATE` is 3600, so a long run is several commands.** `meditate 9600`
silently becomes 3600, which is how a first pass at measuring this "found" a
plateau that was the cap. §19 records the retraction.

**`invoke` solves one ward; `bind` is the faucet.** A sweep breaks the ward in
front of it and stops. A binding re-casts a spell that has run off the end, which
is the shape §8 already describes: an invocation is an act, a binding is standing
automation.

```bash
# Two `debug_ward` solves buy the first slot, because a press is free now.
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; debug_ward; probe; \
  probe; debug_ward; probe; debug_spell breaking" \
  ORBS_THEN="bind breaking; meditate 3600; status" cargo run -p orbs

# ...and one ward, watched end to end — fifteen presses on this seed.
ORBS_SEED=3 ORBS_BOOT=0 ORBS_GRID=200x45 \
  ORBS_DUMP="attend lens; debug_spell breaking" \
  ORBS_THEN="invoke breaking; meditate 200; peruse lens.log" cargo run -p orbs
```
→ `experience 646`, and it is **linear** — a faucet that stops is this domain's
failure mode, not a crash.

**It runs beside a brew with no contention at all**, because a press takes no
slot. `orbs-balance`'s `scrying` policy reads **0.268/tick** against clarity's
0.140, and a real bound `breaking` ~0.18 — the gap is §8's interpreter spending a
tick on every `if`, `else` and `end`. **Additive rather than competing**, which is
what makes a rate above the flagship a decision rather than a defect (§19).

```bash
cargo test -p orbs-sim --test ward        # the sweep, solving through the real verbs
cargo test -p orbs-sim --test secrets     # the gate, and the verdant regression
cargo test -p orbs-sim --lib tower::ward  # the model: 1296 codes, the proofs
cargo run -p orbs-render --example screens   # the sheet, no sim and no GPU
cargo run -p orbs-balance -- run scrying --ticks 7200 --why
scripts/play.sh lens::                    # ten scenarios, through a real tmux game
```

### The tower rail — every domain at a glance, and no telemetry pane

**There is one main pane now.** The rail takes 16 columns down the right and
holds one box per domain; the telemetry pane it replaced is gone, and five of its
nine readings live at the rail's foot (`tick`, `held`, `scale`, `grid`, `focus`).
The other four were already in `status`, which is the full answer where the rail
is the glance.

**Seven boxes, always, and four are dark.** A room the tower has not built draws
as a dim dotted row with no name — the slots keep fixed positions so a box never
appears and pushes the others down at the one moment it has news.

**Each box is ruled off from the next**, and the rule belongs to the box *above*
the boundary. The seventh draws none, because the foot opens with a rule of its
own and two in adjacent rows is a mistake nobody would author on purpose.

**So the boxes are all the same height and the spare row goes to the foot** —
which is deliberately *not* `tiling::deep`'s leftovers-to-the-earliest rule that
`lay_rail` used to share. A pane's exact height is invisible; a *ruled* box's is
not, and the first box being one row taller put one separator a row below the
other five. `MIN_RAIL_BOX` went 3 → 4 with it: the rule costs a row, and at three
a squeezed box silently dropped the `►spell` line, which is the one row telling a
player that room is automated.

**`►`, not `▸`.** U+25B8 is not in CP437 and drew as `?`, so the row saying a room
is automated read as the orb not knowing what was there. `is_renderable` never saw
it — that lint is for authored prose, and a painter's glyphs are Rust literals — so
`every_glyph_the_rail_draws_is_in_the_code_page` is what holds it now. The board's
`▪` was the same defect and §19 records it; this is the second, found the same way.

**The spell row says `►tending`, not `►tending.spe`.** A spell node is named for
its file, and six columns of `.spell` in a fourteen-column rail left the rail
cutting the extension in half. Stripped in `brief.rs` with the
`content::without_extension` that already existed, not in the painter: rule 2
means `orbs-tui` must not have to know spells live in files.

```bash
# The rail beside a working laboratory. `mp 8t` is the two-letter form the
# instrument panel uses, because `mortar_and_pestle 8t` truncates to a name with
# no number left on it.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; grind sage" cargo run -p orbs

# **A fault, latched.** Stand in the archive and the rail says the laboratory is
# broken — `laboratory ‼`. A refused command is *not* a fault; the four that are
# come from `say_failure` at `Role::Danger` — gave up, missing name, forbidden
# verb, unreadable line.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe broken" \
  ORBS_EDIT="edit\nrepeat 5\nwield zzz\nend\n<esc>\nquit" \
  ORBS_THEN="invoke broken; meditate 20; attend archive" cargo run -p orbs

# ...and cleared by going to look, which is the only thing that clears it.
# Append `; attend laboratory` to the line above and the `‼` is gone.

# The rail **yields whole** below the grid that can host it — no narrow
# fallback. At the floor there is no rail and the readings fall back into the
# session's border title.
ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_DUMP="attend laboratory; grind sage" cargo run -p orbs
```

**`F4` is visibly inert, and that is recorded rather than fixed.** With one pane
both tilings are identical, so the key changes nothing until multiplexing returns
the second pane in Phase 9a. §9 fixes the focus mode on `F4` and §19 fixes the
switch there, so reassigning it to toggle the rail would re-litigate both.

### `orbs-balance` — the economy, looked at rather than argued

**Every duration in the game is a placeholder and this is what sweeps them.** It
drives a real `Sim` through `submit` with six synthetic players and prints
experience per tick against a pinned reference. It found four disagreements with
DESIGN.md on its first clean run (§19), so treat a number in the design as an
*idealisation* until this has been run against it.

**And then it caught a real regression one item later** — the ambient reagent swap,
flagged on all four pinned policies at once while the full suite stayed green and
every See-it line still read correctly. **Run a sweep after anything that touches
the world, not only after anything that touches a duration.**

```bash
cargo run -p orbs-balance -- list
cargo run -p orbs-balance -- sweep --ticks 7200 --why
cargo run -p orbs-balance -- run clarity --ticks 1200 --why
cargo run -p orbs-balance -- sweep --hours 4 --csv /tmp/curves.csv
cargo test -p orbs-balance          # the pins, as a test rather than a column
```

**Two hours is the shortest honest sweep, and an hour was the default.** A policy
amortises its first lap's setup over the run, and the ambient reagent swap
(`tower::sabotage`) costs a few minutes an hour — so at 3600 ticks one badly-timed
swap moves a rate by more than its tolerance band and the table flags the seed
rather than the game.

**A `<-- drifted` marker is only a pin while somebody reads the column, so
`tests/agrees.rs` reads it.** It runs the five pinned policies through the real
`drive::run` and fails when a measurement leaves its band. It exists because the
first version of that file — named for this crate's See-it claim — drove `Sim` by
hand in both its tests and would have passed with the whole harness deleted. The
crate has a `lib.rs` for that reason: a binary-only crate cannot be reached from an
integration test at all, which is *why* it had been written the other way.

**Read the `cost` column before the rate.** Every entry in it should be a scour
the policy asked for; anything else means the loop has fallen out of phase with
the tower, and the rate beside it is measuring a game nobody plays. `--why`
prints the sentences behind it, which is where the diagnosis is — **two sweeps
were read as balance findings before that flag existed, and both were the
policy's fault.**

**A rate here is a *loop* rate and DESIGN.md's is a *recipe-tick* rate.** Clarity
is 0.170 idealised, 0.130 hand-played (`experience 16` at tick 123), 0.140
looped. The gap is a tick of queue latency per command plus a `PURGE_TICKS` scour
per fouled instrument, and it is not a defect in either number.

**...except for `bound`, where a cost is usually the loop working.** A spell that
**waits** stamps `Role::Cost` too — `say_blocked`, once per block — which is the
runner doing exactly what §8 asks of it, so `bound` carries ~640 healthy costs in
a two-hour sweep. Only the sentence tells a wait from a refusal, which is why
`--why` matters more here than anywhere:

```text
640  tending.spell waits: the mortar_and_pestle is working   ← the loop working
  1  the orb is already holding tending.spell                ← the loop broken
```

**A maze is one sample per seed** — `stacks` reads 0.0067 at seeds 0 and 11 and
0.0144 at seed 3, so it is deliberately absent from the pinned table. Sweep it
across several `--seed`s or conclude nothing.

**A ward is not**, and the contrast is worth knowing: `scrying` reads 0.2656 to
0.2703 across the same four seeds, because a two-hour run breaks hundreds of
wards and the draw averages out. It is pinned at **0.268**.

**Two policies model a *spell* by typing its commands, and neither pays the
interpreter.** `stacks` and `scrying` both issue one command per tick, where a
real bound spell also spends a tick on every `if`, `else` and `end` — `breaking`
measures ~0.18 against the policy's 0.268. That is §19's *"the harness has no
player"* applied to execution: a policy is a ceiling, and the column says what
the loop is worth rather than what the language costs. **`bound` is the one that
pays it**, which is the next paragraph.

**`bound` is what measures the script engine, and nothing did before it.** It is
`grind`'s loop again — the same two commands for ever — run by a bound spell
instead of by hand, so everything about the two is equal except who is typing and
the gap is purely what automation costs. **It keeps 0.910 of the hand-played
rate, on every seed**, because that quotient is a property of the runner rather
than of the world: the absolute rates both move with a world's luck at sabotage
and the ratio does not move at all. So `tests/agrees.rs` pins the **quotient**
and the table reports the rate.

```bash
cargo run -p orbs-balance -- run bound --ticks 7200 --why   # 0.0910
cargo run -p orbs-balance -- run grind --ticks 7200         # 0.1000, by hand
```

**Run it after anything that touches the spell runner** — `SCRIPT_BUDGET`,
`PATIENCE`, `would_block`, the compiler or the language. Those were unmeasured
for four phases and §19 records what that cost.

**Adding a policy: scour *before* use, never after fouling.** The byproduct a
stage leaves blocks the *next* lap's load, so a purge placed after the stage that
dirtied it cleans an instrument nothing is waiting on and still leaves lap two
refused. And a driver must wait on the **triage** slot as well as the production
one — `Sim::working()` reports only the latter, and a policy that runs over its
own scour reports 0.072 for a loop worth 0.140.

**A dump has no clocks of its own.** It builds no `App`, so it advances no
`Time` and has no `Time<Fixed>` — every animation would sit at phase zero and
every bar exactly on a tick boundary. `ORBS_FIRE_PHASE` (the animation clock) and
`ORBS_TICK` (position within a world tick) are what make those visible as text;
without them the See-it lines degrade to "it compiles".

**Each instrument has its own verb** (§19, built): `grind`, `digest`, `mix`,
`distil`, and the athanor's `kindle`. `grind sage` *is* `move sage to
mortar_and_pestle` followed by `wield mortar_and_pestle` — naming the operation
rather than the tool collapses the two commands a player types most. `move` and
`wield` still exist and still work; they are simply not how the loop is written
any more, and a dump using them is testing the long way round.

**The fetch reaches into other instruments, not just the shelf**, and that is what
makes the loop `move`-free rather than merely shorter. `pipeline::reachable` walks
every unbusy instrument in raise order and *then* the store, so `digest ground-sage`
takes it straight out of the mortar and leaves the husks. A whole clarity brews to
`experience 16` without the word.

**`move` stays in the laboratory anyway**, and §19 says why: it is the only way
finished work leaves the room (`move clarity to arsenal`) and the only way to reach
an instrument's `charged` state, which the three animation See-it lines below need
because `grind` never rests there. What changed is the *manual* — `recall move` led
with `move sage to mortar_and_pestle`, which is a loop nobody writes.

```bash
cargo test -p orbs-sim --test fetching   # the tool-to-tool reach, and the brew
```

**`empty` is a different question and is still required.** It clears the byproduct
so the mortar can take a second load; without it a brew stalls at `grind rock-salt`
with *"the mortar_and_pestle can do nothing with husks, rock-salt"*.

**Reach for `ORBS_DUMP` first; keep `ORBS_CAPTURE` for what only pixels show.**
The screenshot path needs a composited window, and without one it writes a valid
PNG of a **black rectangle** — the renderer fine, the picture proving nothing.
That is worse than no picture, because it looks like evidence. `ORBS_DUMP` draws
the same frame through the real `paint`, `Sim` and `ScreenLayout` into a `Frame`
nobody rasterises, then prints it with its linear stream beneath.

**A dump draws the game's own grid, 120×45, and `ORBS_GRID` is now only for the
80×22 authoring floor.** The grid stopped following the window (§19), so there is
one grid and a dump showing any other is an instrument reading a screen nobody
has. What a dump still cannot show is the *fit* — it builds no `App`, so it has
no camera and no projection, and the 4:3 letterbox needs a window and eyes.

```bash
ORBS_DUMP="attend laboratory; grind sage; meditate 12; empty mortar_and_pestle" cargo run -p orbs
ORBS_DUMP=1 ORBS_GRID=80x22 cargo run -p orbs    # the 80×22 authoring floor
ORBS_DUMP=1 ORBS_BOOT=post cargo run -p orbs     # a boot stage as text
ORBS_BOOT=0 cargo run -p orbs                    # skip the boot sequence
ORBS_LINE="grind sa" ORBS_DUMP=1 cargo run -p orbs   # ...with a line half-typed
ORBS_SCROLL=14 ORBS_DUMP="..." cargo run -p orbs     # ...scrolled back 14 records
ORBS_FIRE_PHASE=0.33 ORBS_DUMP="..." cargo run -p orbs   # ...an instrument mid-animation
ORBS_TICK=0.5 ORBS_DUMP="..." cargo run -p orbs      # ...half way through a world tick
```

**Three animations are *edges*, and a dump observes none of them.** The flare,
the pour and the creep all start on a frame where something changed, and a dump
runs no systems — so each needs a switch of its own or it is the one part of the
effect with no See-it line at all:

```bash
# The athanor catching. Compare 1 against 0: the flame climbs out of the base.
ORBS_BOOT=0 ORBS_FLARE=1 ORBS_DUMP="attend laboratory; kindle charcoal" cargo run -p orbs

# The mortar filling. **`move`, not `grind`** — `grind sage` is the move and the
# wield in one tick, so the bowl never rests at `charged` and never pours. Step
# ORBS_LOAD 1.0 → 0.5 → 0.0 and the block builds up off the floor of the bowl.
ORBS_BOOT=0 ORBS_LOAD=0.5 \
  ORBS_DUMP="attend laboratory; move sage to mortar_and_pestle" cargo run -p orbs

# The bed creeping *between* ticks rather than jumping once a second.
ORBS_BOOT=0 ORBS_TICK=0.5 ORBS_DUMP="attend laboratory; grind sage; meditate 3" cargo run -p orbs
```

**The balneum mariae's See-it line is longer than the others, and it has to be.**
The bath needs the athanor alight — `heat = true` in `recipes.toml` — and it takes
its input from the mortar, so reaching a *working* bath means running the first
two stages first. There is no shortcut:

```bash
# A vessel filling with tincture. Step the meditate to watch the level rise.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; grind sage; \
  meditate 9; empty mortar_and_pestle; digest ground-sage; meditate 6" cargo run -p orbs

# ...and the three states the sim reports no meter for, which is where a vessel
# picture earns its keep. `move`, not `digest`, to stop at charged.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; grind sage; \
  meditate 9; empty mortar_and_pestle; move ground-sage to balneum_mariae" cargo run -p orbs
```

**`ORBS_DUMP` cannot show the bath moving, and that is by design.** All of the
roil is in the colour — the glyph is `█` at every fill and every phase, which is
what makes the level survive greyscale — so a dump of it is a solid bar that
proves nothing. The `screens` example prints the ramp steps as `a`/`b`/`c`, and
that is the only text See-it there is for it:

```bash
cargo run -p orbs-render --example screens   # ...the roil, as letters
```

**A material's tint is pure colour too, so a dump prints the *regions*.** A tint
changes no glyph, which makes it the one thing on the panel whose failure is
total and invisible: a colour reported by the sim that never reaches a cell draws
in the base hue and looks exactly like a material nobody has tinted yet. Every
dump that has one lists it under the linear stream:

```bash
# sage grinds green; leave the husks behind and the same bar turns brown.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; move sage to mortar_and_pestle" cargo run -p orbs
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; grind sage; meditate 9; \
  move ground-sage to dispensary" cargo run -p orbs
```
```text
-- tinted regions (DESIGN.md §19) --
  green    30×1 at 29,1        ← and `brown` after the husks are all that is left
```

Colours are authored in `crates/orbs-sim/content/materials.toml` against the
eight names in `orbs_render::Tint`. **An unknown name fails the load** rather
than falling back, because an untinted material draws in the base hue too — a
silent fallback would make a typo indistinguishable from an omission.

**The flask prints three regions, and one of them is two colours.** Its bar is
the only place a region is a *mixture* — `green+bone` below is the two
ingredients becoming one thing — and reaching it means running the first three
stages, because the flask combines what the mortar and the bath hand it:

```bash
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; \
  grind sage; meditate 9; empty mortar_and_pestle; digest ground-sage; \
  meditate 14; siphon balneum_mariae; grind rock-salt; meditate 9; \
  empty mortar_and_pestle; mix sage-tincture with ground-salt; meditate 5" \
  cargo run -p orbs
```
```text
flask_and_rod  working  ▓▓████████████████▓▓▓▓▓▓▓▓▓▓▓▓
  green      6×1 at 47,3      ← the two ingredients, shrinking together
  bone       6×1 at 53,3
  green+bone 18×1 at 29,3     ← the mixture, growing from the fill end
```

Step the last `meditate` and the mixture grows while both bands shrink at the
same rate — neither ingredient is consumed before the other is touched, which is
what combining means.

**The spell editor has two channels of its own.** `ORBS_EDIT` types into the
editor a `scribe` opened — newline-separated keystrokes, in order. **The editor's
own three states decide what a segment is**: it opens in *command* state so the
first segment is a word (`edit`, `guide`, `interpret` or `quit` — that is the
whole vocabulary), `edit` drops into the buffer, and the token `<esc>` comes back
out.

**The guide is the fourth word and the pane down the right.** By default it lists
the nine control words and then the verbs *this spell's domain* offers — the
**spell's** domain, not the player's, so the same editor in the archive and in
the laboratory shows different verbs. Put the caret **at the end of a word** and
it becomes that word's page instead. `guide` toggles it, session-scoped; nothing
persists it.

**It needs 97 columns and yields whole below them** — 60 for the buffer, the
gutter, and 30 for the guide. At the 80×22 floor it is simply not there, which is
correct: `threading` has 62-character lines and a 43-column buffer is worse than
no guide.

```bash
# The listing, then the page. Only a running editor has a caret to move, so
# this is a `tui.sh` See-it line and `ORBS_DUMP` cannot reach the second half.
scripts/tui.sh start
scripts/tui.sh type 'attend archive' 'scribe threading' 'edit'
scripts/tui.sh see                       # nine control words, then `here you can`
scripts/tui.sh key Right Right Right Right Right Right
scripts/tui.sh see                       # ...`repeat`, its page, wrapped
scripts/tui.sh key Down Down
scripts/tui.sh see                       # ...`follow` — a verb, the other key family
scripts/tui.sh stop
```

**Two key families, and getting the spelling wrong registers a subject nobody
authored.** A control word reads `recall_<w>` / `using_<w>`; a verb reads
`man_<w>_gloss` / `man_<w>_use`. `Prose::topics` strips `recall_` to decide what
is nameable, which is the same `clarity_use` trap the manual already carries.

**The caret must be *past* the word's first glyph**, not at its start: `guide`
matches `run.start < at && at <= run.end`. So entering edit mode — caret at 0,0 —
shows the listing rather than the page for the first word, deliberately.

**Touching a word is a page; past it is what may come next.** Type a space and
the guide stops explaining the word and starts answering `if ` with the places,
`is ` with the eight state spellings, `repeat ` with `until`. `parser::expect` is
what it asks, so **the guide, both Tab completions and the prompt's ghost all
give the same answer**.

```bash
scripts/tui.sh type 'if '                       # the places
scripts/tui.sh type 'the mortar_and_pestle '    # `is`, `has`
scripts/tui.sh type 'is '                       # idle free still working ...
```

**The states belong to `is` and *not* to `wait`.** `wait` stores a **thing** and
resolves it by scanning the record stream, so `wait for the mortar_and_pestle to
be idle` waits on a thing called `mortar_and_pestle be idle`, matches nothing,
burns `PATIENCE` and latches a fault on the rail. `be` is not filler. This is the
trap the guide exists to stop and it caught its own author writing the spec.

**Three of the nine words cannot open a line**, so the listing shows six at the
top of an empty spell: `until` is `repeat`'s guard, `end` needs something open,
`else` needs an `if` **directly** above it. Open an `if` and the other two
appear. `open_blocks` keeps a **stack**, not a depth — a count cannot tell an
`if` inside a `repeat` from a `repeat` inside an `if`.

**Tab completes in the editor, and the guide is its listing.** readline's rules
live once, in `orbs_shell::tabbing`: extend to the longest common prefix, list
when that adds nothing, then cycle. The prompt needs `Offered` and a reserved
row to show candidates; the editor does not, because the pane beside it is
already showing them.

```bash
scripts/tui.sh type 'gri'; scripts/tui.sh key Tab   # -> `grind `
scripts/tui.sh key Tab Tab Tab                      # lists, then cycles
```

**The prompt is highlighted too**, on the same three weights — but `repeat` and
`gathering()` are **suppressed** there, because the prompt cannot run either and
drawing them bright would say the orb knew a word it is about to refuse. Weight
is invisible to `ORBS_DUMP`, so `ink` is the only instrument that can see any of
this.

**There is no `save`.** The buffer writes itself out a beat after the typing
stops, and that pause is measured off `Time`, which a dump never advances. So in
a dump, **`quit` is how you save** — it flushes, then closes. The vim shorthand
still works if you want one without the other: `w` writes and stays, `wq` does
both.

`ORBS_THEN` runs commands *after* the editing session — needed because a save
queues its write for the next tick like every other effect, so a `peruse` inside
`ORBS_DUMP` runs before the spell exists and offers the other readables instead.
That looks exactly like a bug and is not one.

```bash
# The whole loop: write it, save it, cast it. A spell is written *for* a domain
# (`scribe` from inside one), so there is no `attend` in the file.
ORBS_DUMP="attend laboratory; scribe brewing" \
ORBS_EDIT="edit\nkindle charcoal\ngrind the sage\nempty mortar_and_pestle\n<esc>\nquit" \
ORBS_THEN="invoke brewing; meditate 40" cargo run -p orbs

# ...and what the file holds, which is **exactly** what was typed (§19). The orb
# never rewrites a spell; `peruse` gives you your own words back.
ORBS_DUMP="attend laboratory; scribe morning" \
ORBS_EDIT="edit\nmake a potion of clarity\n<esc>\nquit" \
ORBS_THEN="peruse morning.spell" cargo run -p orbs

# `interpret` is where the orb's reading lives now — the third editor word, and
# the only place a *wrong* resolution can be seen before it runs. The status row
# counts what it cannot read; drop the trailing `interpret` to see that instead.
ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend laboratory; scribe check" \
ORBS_EDIT="edit\nmake a potion of clarity\nif the mortr is bare\nsurvey\nend\nxyzzy plugh\n<esc>\ninterpret" \
  cargo run -p orbs

# `has` takes a count, and `interpret` is where you check it survived. This
# **used to print `if cabinet has fragment`** — the number silently swallowed,
# no fault raised, which is §19's "the orb writes down a shorter command than it
# heard" arriving through the one surface built to catch it.
ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend archive; scribe check" \
ORBS_EDIT="edit\nif the cabinet has 4 fragment\nwield lectern\nend\n<esc>\ninterpret" \
  cargo run -p orbs

# Editing a spell while it runs — the marker in the gutter is the orb's place
# in the file, and the save that lands a beat later is picked up mid-flight.
ORBS_DUMP="attend laboratory; invoke brewing; meditate 3; scribe brewing" \
  cargo run -p orbs
```

**`debug_spawn <name> [count] [place]` skips the setup.** A state worth testing
costs forty ticks of grinding to reach; this puts stock straight where you want
it, from wherever you are standing. Known names only (`Recipes::vocabulary` plus
the fuels), and bare it lists them. It is **not a verb** — matched exactly,
before the parser, absent from `Verb::ALL` and from the tutorial — and it is
`cfg(debug_assertions)`, so a release build has no code for it at all.

**It lands where the thing belongs, so the common case needs no destination.**
`tower::home` is a rule over the content, not a list: finished work goes to the
arsenal, anything a recipe *produces* goes to the domain that produces it, and
anything else to the domain that consumes it — always the **store**, never an
instrument, because a shelf is inert and a charged tool is a state to explain
rather than one to test from.

```bash
# From the archive, with no destination named. Each lands in its own room.
ORBS_BOOT=0 ORBS_DUMP="attend archive; debug_spawn fragment 4; debug_spawn clarity; \
  debug_spawn sage; survey cabinet; survey arsenal" cargo run -p orbs
```
```text
survey cabinet   reagent  fragment 4     ← made in the archive, so it stays there
survey arsenal   essence  clarity 1      ← finished work keeps itself
                                         ← the sage went to the dispensary
```

**Three lints make that a guarantee rather than a claim**, which is the point:
a new item has to be testable the moment it is authored.
`every_material_has_a_home_a_move_can_reach` fails the build by name if an
authored material has nowhere to live or lives somewhere `reachable` cannot see;
`every_name_the_tool_offers_lands_in_the_room_it_belongs_to` drives each one
through a real `Sim` and checks it arrives where the rule says;
`every_material_the_game_has_is_one_the_tool_can_make` catches the direction that
rots — a material with a tint that no recipe names, with both files parsing
perfectly.

```bash
# Bare, to see the whole list.
ORBS_BOOT=0 ORBS_DUMP="debug_spawn" cargo run -p orbs

ORBS_BOOT=0 ORBS_GRID=100x30 \
  ORBS_DUMP="attend laboratory; debug_spawn ground-sage 3; survey dispensary" \
  cargo run -p orbs

# The gate itself, from the side that cannot be tested in a debug build.
cargo test --release -p orbs-sim --test debug_spawn
```

**The third slot is a *shelf*, not any place**, and the rule is what
`pipeline::reachable` can see into: an instrument or store anywhere in the tower,
or the arsenal. Three things are refused, each because the state it would build
is one the game cannot reach on its own — which is what this word already refuses
for an unknown name, a nought count and a wrong noun kind:

```bash
ORBS_BOOT=0 ORBS_DUMP="debug_spawn fragment 4 lectern; debug_spawn clarity 1 arsenal; \
  debug_spawn sage 1 arsenal; debug_spawn sage 1 north; debug_spawn sage 1 laboratory" \
  cargo run -p orbs
```
```text
the shelf finds it had fragment all along          ← any instrument, from any room
the shelf finds it had clarity all along           ← the arsenal, which is a domain
the arsenal keeps finished work. sage is not any   ← its door holds for a tester too
there is no shelf here to put sage on              ← a *way* is a fixture and is not
there is no shelf here to put sage on              ← an ordinary domain never was
```

**A `way` is the one that surprises.** `north` and its three siblings carry
`Fixture` so the maze can publish readings into them — and `research::refresh`
despawns *everything* in a way on the step after, so a reagent put there is a
pile that vanishes with no line saying so. A tester chasing that would be chasing
the tool.

**A spell's own records are in the log, not in the pane.** `prompt.rs` draws
*"what the player did, not what their spells did"* — a `repeat` loop would
otherwise push the player's last line off screen in seconds — so a dump that
casts a spell and looks for its output on the transcript finds nothing and looks
broken. `sift` or `peruse` the log, which is the same stream read another way:

```bash
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe broken" \
ORBS_EDIT="edit\nrepeat 5\nif the mortr is idle\ngrind sage\nelse\nsurvey\nend\nend\n<esc>\nquit" \
ORBS_THEN="invoke broken; meditate 20; sift broken orb.log" cargo run -p orbs
```

**`bind` costs 16 experience, and there is no way to grant it.** Concentration is
derived from work completed, `debug_spawn` deliberately earns nothing, and no
public API hands the sim a number — so reaching a slot in a dump means running
two real distillations, or brewing one clarity end to end. The short route needs
`kindle charcoal` (the alembic wants heat) and `empty alembic` between runs (a
charged instrument will not take a second load):

```bash
# The curve: one clarity by hand. `+1` … `+8` on the yield lines, then
# "the orb can hold a spell now", then experience 16 / concentration 1.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; kindle charcoal; \
  grind sage; meditate 9; empty mortar_and_pestle; digest ground-sage; \
  meditate 14; grind rock-salt; meditate 9; empty mortar_and_pestle; \
  mix sage-tincture with ground-salt; meditate 12; distil clarified-draught; \
  meditate 60; status" cargo run -p orbs

# What `bind` buys, and it only reads as a pair. An **invocation** ends when you
# walk out: "first_light.spell needed you there. it stops".
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; invoke first_light; attend archive; \
  meditate 6" cargo run -p orbs

# A **binding** does not. Experience climbs while the player stands in the
# archive, and the sidebar reads `held 1 of 1` — a row that is *absent* until
# the orb has been taught to hold one.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; kindle charcoal; \
  debug_spawn clarified-draught 2; distil clarified-draught; meditate 60; \
  empty alembic; distil clarified-draught; meditate 60; scribe tending" \
ORBS_EDIT="edit\ngrind sage\nempty mortar_and_pestle\n<esc>\nquit" \
ORBS_THEN="bind tending; attend archive; meditate 40; status" cargo run -p orbs
```

**The spell has to empty its own mortar.** A standing spell is cast again every
time it runs off the end, so `first_light` bound would foul the mortar on its
second pass and complain about it for ever — `grind sage` then `empty
mortar_and_pestle` is the shortest loop that can actually lap.

**`recall` bare is the manual's overview, and `help`/`man`/`?` all reach it.**
**It opens with a primer for the room and lists the vocabulary second** — a player
who types `help` in the lens is asking what a ward is, not for twenty-five verbs.
The listing is still `execute::offered`, the same filter the boot report uses,
grouped by `Verb::group()`.

**Three prose keys per room**: `man_here_<room>` is what the place is,
`man_start_<room>` how to begin its puzzle, `man_solve_<room>` how to finish one.
`man_`, never `recall_` — `Prose::topics` strips the latter to decide what is
nameable, so `recall_here_lens` would register `here_lens` as a subject nobody
authored (§19; the `clarity_use` trap again).

**A room lists only its own operations**, and it used to list every other room's.
`Scene::offers` asks `Verb::anchor` — *which fixture must stand here* — where it
asked `is_operation`, the **production slot** question, so anything that took no
slot went everywhere: `help` in the laboratory offered `research`, `follow` and
`wander`. §19 records this as the debt those two words were waiting on.

The two questions are separate now, and a verb can be either, both or neither.
Self-anchored verbs are **derived from the `Branch` that declares them**, so a new
domain's verbs scope themselves; `every_self_anchored_verb_is_declared_by_a_fixture`
fails the build both ways, because a verb claiming an anchor nothing declares
resolves in *no* room. And the refusal names the fixture now —
`there is no stacks here to wander with`, from `tower::fixture_of`.

**Check it in every room, and at the floor.** The 70-cell width lint keeps a
single line from wrapping there.

**Four rooms fit 80×22 whole; the laboratory and the archive do not, and that is
not a defect.** Measured: the laboratory's page wants **26 rows** — it is the one
room whose transcript is narrowed by the instrument panel, *and* the one with
five extra verbs — and the archive is one row over. The primer lines are already
inside the width lint, so the only way to make them fit would be to stop offering
a verb, which is worse than scrolling. `help` is a record like any other and
`PgUp` reaches it.

This block used to claim the page "fills the 80×22 transcript exactly". It never
did for the laboratory, which is the worst case and therefore the one a See-it
line is least likely to be run against:

```bash
for room in laboratory archive lens grimoire arsenal tower; do
  ORBS_BOOT=0 ORBS_DUMP="attend $room; help" cargo run -q -p orbs; done
ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_DUMP="attend lens; help" cargo run -p orbs
```

**Two lints, and neither can see a *refusal*.**
`every_room_a_player_can_stand_in_explains_itself` requires all three keys for
every **built** domain (from `Sim::briefs()`) plus `arsenal` and `tower` by name;
`a_primer_only_names_words_that_room_actually_offers` checks each verb a primer
names against `offered`. The second caught `bind` in the grimoire — gated at
concentration 0. What it *missed* is `scribe`, which is offered in the grimoire and
refuses, so the first draft told the player to do the one thing that room cannot.
**Run the loop above rather than trusting the green** — that is the half only eyes
have.

**A bare verb that needs an argument still asks.** `recall` is the exception and
`TOPIC_OPTIONAL` is why (§19): it is `survey`'s shape, because bare and
argumented are the same act at two scopes. If you need an *ambiguity* fixture in
a test, use `purge` — `brew` was one until `recall`'s slot changed, and three
tests were silently left asserting nothing.

**`weave` opens the progression screen, and `ORBS_WEAVE` types at it.**
Newline-separated like `ORBS_EDIT`, but **every segment is a whole thing** — a
word, or one of `<up>` / `<down>` / `<left>` / `<right>` / `<esc>`. There is no
buffer, so nothing is typed a character at a time and Enter is implied at the end
of a segment and nowhere else.

**The arrows do nothing until a word has gone into a track.** `ley` or `mastery`
is what hands them over, the way `edit` drops into the editor's buffer — so a
dump that opens the screen and presses an arrow is testing the refusal, not the
movement. **Left/right walks the track; up/down picks between a tier's
siblings** — rightward is progress, downward is a choice.

**The session pane is 104 columns, not 120.** The tower rail takes 16 off the
right, so the grid's width is never this surface's. It **was 60** while the
second pane held telemetry; a surface authored against that has 44 columns of
slack it did not have. `ORBS_GRID=80x22` drops the rail entirely and is where a
sentence stops fitting — worth a look, even though the game itself no longer
reaches it.

```bash
# Nothing earned: the bar reads `0 of 100`, the ley line's one station draws
# `[·]` (untaken) with its cost `16` under it, and mastery's tier is `[·]` too.
# **The bar is against a fixed SCALE of 100, not against the next threshold** —
# this line used to claim `0 of 16` and an `opens at`, and neither was ever on
# screen. A See-it line that describes a different screen is worse than none.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" cargo run -p orbs

# A tier **opening**, which is the only place the choose-between shape shows.
# Three alembic runs are 24. `empty alembic` between them, as always.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; kindle charcoal; \
  debug_spawn clarified-draught 3; distil clarified-draught; meditate 60; \
  empty alembic; distil clarified-draught; meditate 60; empty alembic; \
  distil clarified-draught; meditate 60; weave" cargo run -p orbs

# Aim with the arrows, then type the word — `«○»` moves and the refusal names
# the node you aimed at, which is what proves typing kept the aim.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" \
  ORBS_WEAVE="mastery\n<down>\ntake" cargo run -p orbs

# ...and the arrows before a word, which must move nothing and say what to do.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" \
  ORBS_WEAVE="<down>\n<right>" cargo run -p orbs
```

**Nothing is takeable yet and that is deliberate** — every Mastery node is
authored as a marker, so `take` always refuses in voice. A dump looking for a
node to change state is looking for the next item.

**The archive draws a map, and `wander` gives it the arrow keys.** The map is
*not* gated on the word — it draws whenever a maze is open, which is what makes a
bound solver watchable — so `research` alone is enough to see it. `wander` only
decides who the arrows belong to, and `ORBS_WALK` presses them:

```bash
# The maze beside the transcript. **There is no fog** (§19) — every wall is on
# screen from the moment it opens; what fills in as you walk is the marks.
# **The whole picture now fits at the game's own grid**, and used to pan: the
# session pane was 58 columns against a 35-column maze, and the tower rail taking
# the telemetry pane's place left it 102. Panning is still what `stacks::split`
# does below that — `ORBS_GRID=80x22` is where to see it — and it is the designed
# fallback rather than a defect.
ORBS_BOOT=0 ORBS_DUMP="attend archive; research" cargo run -p orbs

# ...opening as it goes. `▒` walked once, `░` finished with, `☼` the reading,
# `Ω` the way out once a walked cell is beside it.
ORBS_BOOT=0 ORBS_DUMP="attend archive; research; follow east; follow east" \
  cargo run -p orbs

# The arrows, and **the only place the whole maze is on screen at once**:
# `wander` takes the whole pane — maze centred, count and keys beneath, no
# transcript and no prompt. A spell solving one keeps the map inline, panning.
ORBS_BOOT=0 ORBS_DUMP="attend archive; research; wander" \
  ORBS_WALK="<right>\n<right>\n<down>\n<down>\n<left>" cargo run -p orbs
```

**Leaving a surface on a *held* key is a case no dump can reach, and it shipped
broken.** Escaping the maze with a finger still on an arrow put `wander` back in
the prompt: key repeat kept arriving, the maze had let go, and an arrow at the
prompt means *recall history*. `HeldOver` drops keys whose press went to another
surface until they are released — see §19, including why the watch that notices
the handoff has to be **ungated** (`type_into_line` is gated on a keystroke
arriving, so the frames it needs to have seen are the ones it never runs on).

It applies to all four surfaces, not just the maze — the editor and the weave
screen hand the keyboard back the same way. **`ORBS_WALK` cannot show any of it**,
because a dump writes no key events and holds nothing down; the gate is
`leaving_the_maze_leaves_nothing_in_the_prompt`, which needs real `KeyCode`s
because the harness's own `press` stamps every key `KeyA`.

**`ORBS_SEED` is how you see a *second* maze.** The reading starts in a random
corner and the way out is drawn against a weighting that leans on the opposing
one (§19), so one dump is one sample and proves nothing about either. Without
this the binary is seed `0x0B5` for ever: three dumps taken to check the
randomisation came back identical and read as a failure, and they were three
copies of the same seed.

```bash
ORBS_SEED=3  ORBS_BOOT=0 ORBS_DUMP="attend archive; research; wander" cargo run -p orbs
ORBS_SEED=11 ORBS_BOOT=0 ORBS_DUMP="attend archive; research; wander" cargo run -p orbs
```

It applies to the whole world, not just the archive — everything generated is
one seed's worth of evidence per run. The *distribution* is not something a dump
can show at all, so it is a test: `cargo test -p orbs-sim --lib research`.

**A scroll is `wield`ed, and the archive's loop is the laboratory's.** A walk of
the stacks pays its fragment onto the **cabinet**, the archive's shelf; the player
moves **four matching fragments** into the lectern and wields it. `move` carries
one unit, so that is four `move`s — noise against four walks of the stacks, and
four lines of a spell if it grates.

**A spell asks `if the cabinet has 4 fragment` before it moves any**, and that
guard is what the count was added for. Without it a spell cannot tell one
fragment from four, so the only shape available was `repeat 4 / move / end` fired
blind — which moves whatever is there, wields a short lectern, and says *"the
lectern can do nothing with fragment"* on every lap for ever. `debug_spell
assembling` writes the guarded version.

**At least, never exactly**: `has 4` stays true at five, or the guard jams the
moment a solver gets ahead of it. `has 1 X` is what a bare `has X` already meant
and writes back bare; `has 0 X` is `has no X`.

```bash
# 2 fragments: nothing moves and the log stays quiet. 4 and 5 both assemble.
ORBS_BOOT=0 ORBS_DUMP="attend archive; debug_spawn fragment 2; debug_spell assembling" \
  ORBS_THEN="invoke assembling; meditate 60; peruse archive.log" cargo run -p orbs
```

**Where a fragment lands is `tower::home`, asked by both paths**, so a spawned
one and a won one cannot end up in different rooms. That split existed for one
change and is the reason the tests ask the rule rather than naming a room.

`debug_spawn`'s third slot is still there for when you want the fragments
*already* in the lectern. **The count is positional and required**, so
`debug_spawn fragment lectern` is refused rather than read as one fragment
somewhere.

```bash
# The whole loop: assemble a scroll, open the stacks, spend it on them.
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend archive; \
  research; debug_spawn gleaning-scroll; wield gleaning-scroll; wander" \
  cargo run -p orbs
```

**Five `♦` and no `Ω`, and the missing `Ω` is the point.** A gleaning errand
withdraws the way out rather than leaving one that does nothing: a solver's top
rung is `if <way> has exit`, so an inert exit would have it walk onto that square
and take the same rung for ever. **`wander` is how you see the whole maze** — the
grid is fixed at 120×45 and the map pans inside a pane, so a dump at any other
`ORBS_GRID` is an instrument reading a screen nobody has.

**The errand is a word on the stacks, and that is what a spell asks.** `if the
stacks has gleaning` compiles at cast like every other reading (`Errand::ALL`
chains onto them in `scene_at`), so **one** solver reads which maze it is in and
swaps its top rung from `exit` to `spoil`:

```bash
ORBS_BOOT=0 ORBS_DUMP="attend archive; research; debug_spawn gleaning-scroll; \
  wield gleaning-scroll; survey stacks" \
  cargo run -p orbs
```
```text
Heading    reading
TableRow   gleaning       ← the word a spell asks for, on the stacks
```

A scroll's own draw and the errand-aware solver are both things a dump cannot
show — one is a distribution, the other is four thousand ticks of walking — so
they are tests: `cargo test -p orbs-sim --test gleaning`.

**There are three scrolls and the lectern draws between them.** `gleaning` sets
the errand above; `quickening` sets a **window** in which the laboratory works at
double speed; and `verdant` puts one base reagent the laboratory has never had on
its shelf, endlessly.

**Quickening never refuses for want of something to hurry** — it was a one-shot
on the run in hand, which made it unusable at exactly the moment a player reaches
for one. It is an interval like `Burning`, read at `begin` like heat, so a run
started inside the window stays short when the window closes. What is already
running is hurried too, halved from *now*.

```bash
# An 8-tick grind, twice. Without the scroll `meditate 4` yields nothing.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_spawn quickening-scroll; \
  wield quickening-scroll; grind sage; meditate 4" cargo run -p orbs

# Three reagents become six, one scroll at a time. The fourth says so.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_spawn verdant-scroll 4; \
  wield verdant-scroll; wield verdant-scroll; wield verdant-scroll; \
  wield verdant-scroll; survey dispensary" cargo run -p orbs
```

**`recall <thing>` is a page, not a route.** It says what the thing is, then how
it is used, then how it is made — in that order, because someone holding a potion
is not asking for its five steps. `using_<name>` is the second half and is
**deliberately not** `recall_<name>_use`: `Prose::topics` strips `recall_` to
decide what is nameable, so that spelling would register `clarity_use` as a
subject.

```bash
ORBS_BOOT=0 ORBS_DUMP="recall clarity; recall gleaning-scroll" cargo run -p orbs
```
```text
a potion of clear sight, and the laboratory's flagship work
nothing drinks a potion yet. a siege will be what spends them   ← honest, per `undo`
clarity: 5 steps, 94 ticks
1. sage -> ground-sage + husks  (mortar_and_pestle, 8t)
...
```

**Two lints keep it complete**: `every_material_has_a_page` fails by name for a
material with no description, and `a_finished_product_says_what_it_is_for`
requires a `using_` line on every potion and scroll. A material added without a
page does not ship.

**Every material is a `Topic` because of this**, on the same exemption verb pages
have — *a manual you can only read in the right room has a lock on it*. So §7's
scoping is a claim about the **kind**: from the archive `sage` is something to
read about and not something to grind. A test asserting a name is *absent* from
another room wants `things()`, not `names()`.

**What a verdant scroll may unlock is derived, never listed** — a base reagent is
one the vocabulary knows that *nothing in the tower makes*, whose home is the
laboratory's shelf. Author a fourth herb in `recipes.toml` and it is unlockable
the same tick. **Check `survey dispensary` after four scrolls when touching
this**: the first version asked `Recipes::outputs`, which is a recipe's `output`
and not its `leaves`, so every byproduct read as a herb and `dregs`, `ash` and a
`fragment` were shelved as inexhaustible stock. The suite was green throughout.

**`/tower/arsenal` is the one room reachable from every other, and before it
nothing could be carried between domains at all.** `move`'s destination wants a
fixture where you are standing, so a potion made in the laboratory could not go
anywhere; and a finished potion could not be picked up at all, because the slot
was `Reagent` and a potion is an `Essence` (§19). Both are fixed, and the
exemption is narrow: the arsenal takes **finished work only**.

```bash
# A potion brewed in one room, carried to a second, listed from a third.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; kindle charcoal; \
  debug_spawn clarified-draught; distil clarified-draught; meditate 60; \
  empty alembic; move clarity to arsenal; attend archive; survey arsenal" \
  cargo run -p orbs

# The door. A reagent is refused, and told where it does belong.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; move sage to arsenal" cargo run -p orbs
```

**Nameable is not enough, and that is what to check when touching this.**
`purge` and `verify` take `NounKind::Any`, so they can now *name* a potion from
any room — and a verb that names what it cannot reach says *"there is no clarity
within reach"*, which is the one answer that is false. Put
`attend archive; verify clarity; peruse arsenal.log; purge clarity; survey
arsenal` on the end of the line above: none of them may say that, and the `purge`
must actually empty the room rather than resolving and doing nothing.

`cargo test -p orbs-sim --test arsenal` holds all of it, one claim per way this
could have been half-built.

**A step is in the log, not the transcript** (§19). The map already shows the
reading move, so `follow`'s success is `quiet` — emitted, stored and spoken as
ever, and filtered out of the pane by `Records::drawn`. A dump looking for *"the
reading goes"* on the transcript will not find it and is not broken; a **wall**
is still drawn, because nothing moves and the map reports nothing.

```bash
ORBS_SEED=3 ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend archive; research; \
  follow west; follow west; peruse archive.log" cargo run -p orbs
```
```text
line: 2, message: the reading goes west
line: 3, message: the reading goes west
```

**An arrow moves the reading immediately and consumes no tick.** `Sim::walk` is a
**third entry point** beside `submit` and `step` — the only thing in the game
that reaches the world without a tick boundary — so a player walks as fast as
they can press and no brew advances while they do. Three versions went through
the prompt's queue first; all were some flavour of too slow, because the queue
was solving the wrong problem.

`ORBS_WALK` therefore steps **nothing**: check `tick` at the tower rail's foot after
a long walk and it should be exactly what it was before. If a dump of eight
presses has advanced the clock eight seconds, something has gone back through
`submit`.

**Replay still holds**, and `Submission::Walked` is what makes it hold: it says
*when* — a typed line executes at the start of the next tick, a walk has already
executed. Use `Sim::replay` rather than matching on `Submission` by hand; three
test files had their own copy of that match and they are one now.

**Watching a spell solve it is the point of the map, and `threading` is the
solver.** The ladder is twenty-four rungs across fifty-two lines — it was
ninety-eight before `else if` — and it lives in
`crates/orbs-sim/content/dev_spells.toml`.

**Every dev spell is on the grimoire's shelf from tick 0 in a debug build**, so
`invoke threading`, `scribe threading` and `peruse threading.spell` all work with
no `debug_spell` first. A shelved spell carries its own `Domain` from the file,
which is why it needs no room check — `scribe::write` homes a *new* spell to where
you stand, and nothing shelved is new.

**A release build has none of it**: `raise_grimoire` shelves them under
`cfg(debug_assertions)` and `dev_spells.toml` is `include_str!`'d under the same
`cfg`, so neither the nodes nor the lines exist. Both halves are held —
`the_dev_spells_are_on_the_shelf_from_the_first_tick` and, in release,
`a_release_shelf_holds_only_the_shipped_spells`.

`debug_spell <name>` stays for the other job it does: handing back a **fresh
copy** after one has been edited or purged. It still refuses outside the spell's
own domain, because that path really does write a new file.

```bash
ORBS_BOOT=0 ORBS_DUMP="survey grimoire" cargo run -p orbs      # all five, shelved
ORBS_BOOT=0 ORBS_DUMP="debug_spell" cargo run -p orbs          # what it can rewrite

# An ordinary maze: the way out, and a fragment for reaching it.
ORBS_SEED=11 ORBS_BOOT=0 ORBS_GRID=160x45 \
  ORBS_DUMP="attend archive; research; debug_spell threading" \
  ORBS_THEN="invoke threading; meditate 3600; meditate 3600; survey cabinet" cargo run -p orbs

# The **same file** on the other errand: no way out, five things to gather.
ORBS_SEED=17 ORBS_BOOT=0 ORBS_GRID=160x45 \
  ORBS_DUMP="attend archive; research; debug_spawn gleaning-scroll; \
    wield gleaning-scroll; debug_spell threading" \
  ORBS_THEN="invoke threading; meditate 3600; meditate 3600; survey cabinet" cargo run -p orbs
```

**Do not hand-build the four-tier ladder**, and note that two of its words no
longer exist. `exit`/`passage`/`walked`/`twice` is what §19 calls *"the solver
that was never a solver"* — measured at equal laps and ticks it solved seed 3 and
**failed seed 11**, because without a `back` rung a fixed compass order sends the
reading back where it came from at any junction where two ways read alike, and it
cycles. It also had no `spoil` rung, so it could never glean. That recipe was in
this file for two versions and it was the broken one.

**`walked` and `twice` are gone**; a way reports `marks`, a count, and the
middle tiers compare it — `1 or fewer marks` and `2 or more marks`. They were
two buckets over a `u8` the maze had all along, so a square walked nine times
read exactly like one walked twice.

```bash
# What a way says now. Used to read `back walked`.
ORBS_SEED=3 ORBS_BOOT=0 ORBS_GRID=160x45 \
  ORBS_DUMP="attend archive; research; follow south; follow south; follow north; survey south" \
  cargo run -p orbs
# -> back    marks = 1
```

**The language has nine control words**: `wait`, `repeat`, `if`, `else`, `end`,
`until`, `let`, `for`, `part`.

**`part gathering()` names a run of lines and `gathering()` runs it.** A
definition is stepped *past* where it stands — a spell is read top to bottom and
its parts are written among its lines — and a call is a stack of **descents**,
because a `pc` addresses one tree and a part is a different tree.

**A spell is contained to a single `.spell` file, and that is a decision** (§19).
Two spells cannot share a part; `program::tree` looks only inside the running
spell's own body, and `a_part_is_not_reachable_from_another_spell` holds it.
Cross-file sharing is **struck**, not deferred — with it went *"what a part costs
to hold"*, because an in-file part is not held separately and so has no price to
draw. A spell reaching another spell is `invoke`, which is a second `Running`
with its own budget rather than a descent.

```
part gathering()
    grind sage
    empty mortar_and_pestle
end

repeat 2
    gathering()          ← two grinds, from one body
end
```

**The call is punctuation, and it is the one place a symbol is canonical.**
§19's comparison spellings accept `>=` and write back words because a word exists
to write back to; here none does, so `()` *is* the notation. It also means the
language spends no word on calling — `part` is the only word this cost.

**A definition belongs at the top level and a name means one part.** Both are cut
out and said rather than silently ignored; five complaint keys in all, each
naming its line. Recursion is bounded at `MAX_PARTS` (8), separate from
`invoke`'s `MAX_DEPTH` — at one step a tick a runaway does not hang the game, it
grows the **save** by a descent a second.

**Variables are shared, not per-descent** — a deliberate deviation from the
roadmap's `(spell, pc, loops, vars)` shape (§19). A part takes no arguments, so a
private store would leave it with no way to be told anything at all. The cost:
`for each way` inside a part rebinds the caller's `way`.

**`SCRIPT_BUDGET` is the floor and `spell::budget(world)` is the number.** It
reads `Taken`; `steps_1` and `steps_2` are authored in `progression.toml` and
ship as markers like every other node, so it answers 1 today and what was built
is the wiring. **Parts ship unused by decision** — with every line costing a tick,
factoring into a part is slower than not, and no dev spell was rewritten.

```bash
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; scribe tending" \
ORBS_EDIT="edit\npart gathering()\ngrind sage\nempty mortar_and_pestle\nend\nrepeat 2\ngathering()\nend\n<esc>\nquit" \
ORBS_THEN="invoke tending; meditate 40; peruse laboratory.log" cargo run -p orbs
```

### A spell is highlighted, and `ink` is the only thing that can see it

**A spell's parts carry a weight *and* a hue**, on two axes. §4 gave ordinary
text *"intensity variation alone"* and was **widened at `0.3.35`** to exempt
spell text — the language outgrew what three weights can separate, and `is` and
`the` are opposites that read identically. §19 records the supersession.

| weight | |
|---|---|
| **bold** | `repeat` `if` `else` `end` `part`, and a `gathering()` call |
| normal | verbs, names, numbers, grammar, states |
| dim | filler the orb strips, and `#` comments |

| hue | |
|---|---|
| magenta | control words, and a call |
| yellow | verbs |
| dark-yellow | numbers |
| dark-cyan | **grammar** — `is` `has` `be` `each`, the comparison spellings |
| cyan | **states** — the eight spellings of idle/working/empty |
| *the base* | names, filler, comments |

**Weight still carries the whole reading on its own**, and that is load-bearing:
`ORBS_DUMP` has no colour, a greyscale tube is a §14 case, and the `monochrome`
theme declines the palette outright. Take every hue away and a spell reads as it
did before hue existed.

**`Grammar` is the variant that earned the change.** It was `Filler` and is
filler's *opposite* — filler is what §6 strips, grammar is what the question
turns on — so `is` and `has` looked exactly like the `the` beside them. No amount
of weight could have separated them.

**`empty` is a verb and a state**, the only word that is both, so the state check
sits *below* the verb check in `classify`. Above it, `empty mortar_and_pestle`
draws its verb as a state.

**The hue is a `Frame` side-table, never a fifth `Style` byte.** `Cell` is pinned
at eight and a fifth field cost +28 KiB and ~1.2 µs a frame, measured. A syntax
run is a *region* exactly as an instrument's bar is, so it rides
`Vec<(Rect, Lexeme)>` — and is kept **apart from `Tint`**, which means materials:
the laboratory draws an instrument panel two columns from the editor.

**`parser::lexeme` classifies and `sheet` draws.** The classification is a
*language* fact and lives in `orbs-sim` beside `read` and `analyse`; it is
**lexical and world-free**, so a spell keeps its shape read from another room and
does not change appearance as its own pipeline fills the shelf.

**It declines on an accent and on the running line.** A line `interpret` could not
read stays wholly red; the line the orb is executing stays uniformly bright. Same
rule `Style::depicted` has, and the reason `sheet` announces a row once and draws
its runs silently — highlighting multiplies the runs per line by five or six, and
under the old shape every one would have been a separate utterance.

**A dump prints the runs and cannot print the colours**, so there are two gates
and you need both:

```bash
# What the sim classified. The only text gate a hue has.
ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend laboratory; scribe hues" \
ORBS_EDIT=$'edit\nrepeat 3\nif the mortar_and_pestle is idle\ngrind the sage\nend\nend\n<esc>' \
  cargo run -p orbs | grep -A12 'lit runs'
#   control "repeat" / number "3" / grammar "is" / state "idle" / verb "grind"
```

```bash
# ...and what the terminal was told to draw. `ink` is the only instrument that
# can see either axis, and it is the one to use — see below.
scripts/tui.sh start
scripts/tui.sh type 'attend laboratory' 'scribe hues' 'edit' \
  'repeat 3' 'if the mortar_and_pestle is idle' 'grind the sage' 'end' 'end'
scripts/tui.sh ink 5 40
#   'r':magenta/bold … 'i':dark-cyan … 'i':cyan … 'g':yellow … 't':default/dim
scripts/tui.sh stop
```

**Use `ink`, never a hand-written SGR reader.** crossterm writes every named
colour through the **256-colour** form — `Color::Magenta` is `38;5;13`, not the
`35` an ANSI table predicts — so a parser written against `3x` reports the whole
feature missing. That happened, and the failure is maximally misleading: a broken
reader and a broken feature print identically.

**A well-formed spell, or you are testing the decline.** An unclosed `repeat` or
`if` faults its line to `Role::Danger`, and an accent outranks a hue — so a half-
written spell reads entirely red and looks like the colours never arrived.

### `tower::reach` — one rule for what a name reaches

**Every string→handle lookup goes through it**, and three of them used to be the
same walk written out three times (`navigate::find_script`, `bind::find`,
`invoke::find`). It names the four axes that are questions about the *world*:

| Axis | Settings |
|---|---|
| scope | `In` (children of one node) · `Under` · `Tower` · `Fetch(room)` · `Arsenal` |
| kind | any, or one `NounKind` |
| naming | `Leaf` · `LeafOrPath` (§7: a place answers to both) · `Script` (supplies `.spell`) |
| ordering | **`Fetch` is §10.1's search order and a game rule** — unbusy instruments in raise order, then stores, then the arsenal |

**The fifth axis is not here on purpose.** What a miss *says* — a record, a
`None`, an entry on a `missing` vec — is about what the player is told, and that
is presentation. It stays at the call site.

**A bare `look(world)` is §7's plain rule**: inside where you stand, any kind, by
leaf. Every widening past that is then visible at the call site rather than
buried in a helper's body, which is how *nameable* and *findable* came to
disagree twice (§19).

```bash
cargo test -p orbs-sim --test fetching --test arsenal   # untouched: the proof
cargo test -p orbs-sim --lib tower::reach               # each axis, pinned
```

**A sweep is the other proof.** The fetch order decides what `digest ground-sage`
picks up, so a refactor that moved it would move the economy — run
`cargo run -p orbs-balance -- sweep --ticks 7200` and every rate should be
unchanged to four decimal places.

### A spell's verb is read at cast; its arguments are not

**`may_issue` is asked twice, and that is deliberate.** It is a security boundary
— `attend`, `meditate`, `scribe`, `undo`, `bind`, `unfurl`, `weave`, `wander`,
`quit` — and `run_line` asks it only when a line is *reached*, which for a line
in an untaken branch is never. A `meditate 3600` sat in a spell saying nothing,
and a scripted `meditate` is a hazard: `Sim::step` drains `Skip` in a while-loop,
so an hour of world time runs inside one step.

**But only the verb can be checked at cast, and this is the trap.** A spell makes
its own inputs, so `digest ground-sage` is written above the line that produces
any — at cast the room has none and `interpret` reads the line back as bare
`digest`:

```text
   1 grind sage
   2 empty mortar_and_pestle
   3 digest              ← the reagent is gone. this is correct
```

**Freezing a whole `Intent` at cast would break every pipeline spell in the
game, silently.** The verb survives because a verb is offered by the fixture
standing in the room rather than by what is on the shelf. So `check_commands`
looks at the verb and must say nothing about the arguments — and
`the_cast_check_does_not_fire_on_a_line_that_makes_its_own_input` is what holds
that.

**Lines naming a variable are skipped**, because a variable holds nothing until
the line runs.

**`let <name> be <place>` binds a name, and a variable holds a *name*** — not a
number, not a list, not an expression. Everywhere a name may stand the bound word
stands for it, which is exactly what an accumulator needs and nothing more. The
value resolves against the room **at cast**, like every other name, so `let m be
mortar` binds `mortar_and_pestle`.

**It is `let … be` and not `set … to`, and both halves of that are collisions.**
`set` is already a `dial` synonym and a spell word is matched *before* the fuzzy
matcher, so `set second borax` at the prompt stopped reaching the lens; `to` is
§6 filler and would be invisible to every reader but the one that sees text
before normalisation. `spellword.rs` names three collisions it refused to add —
this would have been a fourth, on a shipped verb.

**`for each <set>` walks a set, and the cursor is named after it.** `for each way`
binds `way`, so the body reads `if way has spoil` and `follow way`. **`it` is
impossible**: it is on the filler list, so `follow it` is stripped to `follow`
before anything sees it.

**A set is declared by the fixture, never derived from `Role::Reading`** — that
marker covers the archive's four ways *and* the lens's four sockets *and* its six
sigils, so a loop over the marker would hand a spell in the lens ten things when
it asked for four. `recall scripting` lists a room's sets, and is the only place
that does:

```bash
for room in archive lens laboratory; do ORBS_BOOT=0 ORBS_GRID=100x40 \
  ORBS_DUMP="attend $room; recall scripting" cargo run -q -p orbs; done
#   archive: way.   lens: socket, sigil.   laboratory: no such section
```

**The *readings* that page lists are keyed on the set too**, and asking
`Role::Reading` instead shipped a real defect: the gate opened wherever any
child carried the marker and then printed `maze::readings()` regardless, so the
**lens** taught `passage wall exit back spoil marks gleaning` and named none of
the ward's six deltas — the whole of what a lens spell may ask. `tower::readings_at`
walks `groups_at` now. Check the `[nameable here]` block in all three rooms above
when touching either.

**`repeat until <question>` is the bound a loop should have** —
`repeat 20000` was a guessed constant chosen to outlast the longest walk, and the
surplus laps spun doing nothing once the maze closed. The guard is asked before
the first pass *and* at the end of each, so `repeat until <already true>` runs
zero times; a question it cannot answer **stops** the loop, which is the opposite
of `if`'s rule and deliberate.

**A comparison has seventeen spellings and one canonical form.** `has 2 X`,
`at least 2`, `2 or more`, `more than 1`, `>= 2`, `>=2` all mean at-least;
`at most 2`, `2 or fewer`, `fewer than 3`, `<= 2`, `<3` mean at-most; `exactly 2`
and `= 2` mean exactly. **Symbols are accepted and never written back** — the
fair copy is words, so a player who has never seen an operator can read it.
`interpret` is where you check a spelling survived, because the ones that did not
used to vanish in silence.

**`else if` chains, and one `end` closes the whole ladder.** A desugaring rather
than a `Kind` — the runner, the save format and `interpret` see the nested tree
that was always written by hand — so nothing downstream knows about it. It took
`threading` from 98 lines to 52: **49 of the 98 were `end` or `else`**.

**And the other side of a comparison can be a place**: `north has fewer marks
than east`, `more … than`, `as many … as`. Both sides go through one read of a
named child's `Stock`, which is the arithmetic `raise_count` already promised.

**Strict against a place, inclusive against a number** — `has 2 or fewer marks`
includes two and `has fewer marks than east` does not. That is English, not an
inconsistency, and *at least as many* is deliberately absent because
`not … fewer … than` says it.

**A comparison alone cannot pick "the way with the fewest marks", and the rung
that looks like it does is wrong.** A walled or unwalked way publishes no `marks`
node, so it counts as **nought** and is the minimum of any four — `threading`
rewritten that way solves no maze at all. Least-of-the-*open*-ways is a **filter
then a minimum**, which is what `let` and `for each` are for and what `roaming`
does:

```
let best be north            ← the seed. it may be walled; the next pass fixes it
for each way
if way has no wall           ← any open way — the dead-end fallback
let best be way
end
end
for each way
if way has no wall and no back        ← prefer one that is not where we came from
...
for each way
if way has no wall and no back and fewer marks than best     ← the true minimum
```

**Later loops override earlier ones, so priority reads bottom-up.** The `passage`
tier `threading` needs is subsumed: an unwalked way publishes no `marks` at all,
which counts as nought, so it is already the minimum.

**Fewer lines is not fewer ticks.** `roaming` is 19 lines against `threading`'s
52 and costs ~27 steps a move where the ladder short-circuits at the first rung
that fires. A **five**-pass version — adding `exit` and `spoil`, which makes it
the same algorithm as `threading` — was measured at ~45 steps and stopped
finishing seed 3 inside 7200 ticks. Both ship, and the pair is the lesson: §19's
step-cost decision as a number rather than an argument.

```bash
ORBS_BOOT=0 ORBS_DUMP="peruse roaming.spell" cargo run -p orbs
for s in 3 11 17; do ORBS_SEED=$s ORBS_BOOT=0 ORBS_DUMP="attend archive; research" \
  ORBS_THEN="invoke roaming; meditate 3600; meditate 3600; survey cabinet" \
  cargo run -q -p orbs; done      # `fragment 1` each time
cargo test -p orbs-sim --test naming_things
```

```bash
# The comparison, and the two cases where it must not fire.
ORBS_BOOT=0 ORBS_GRID=110x34 \
  ORBS_DUMP="attend archive; debug_spawn fragment 4; debug_spawn fragment 1 lectern; scribe weighing" \
  ORBS_EDIT="edit\nif the cabinet has more fragment than the lectern\nmove fragment to lectern\nend\n<esc>\nquit" \
  ORBS_THEN="invoke weighing; meditate 6; survey lectern" cargo run -p orbs
#   4 vs 1 -> lectern 2.   2 vs 2 -> lectern 2.   1 vs 3 -> lectern 3.
```

**`recall` now teaches the language.** Nothing did before — control words are
outside `Verb::ALL` and readings are `NounKind::Sense`, so `recall repeat`
reached nothing and `recall marks` answered with a message about other rooms.

```bash
ORBS_BOOT=0 ORBS_DUMP="recall repeat; recall until; recall marks" cargo run -p orbs

# The page whose last section is the room. Run it in both and compare: the words
# and the question shapes are identical, what you can *name* is not.
ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend archive; recall scripting" cargo run -p orbs
ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend laboratory; recall scripting" cargo run -p orbs
```

**One ladder serves both errands, and no errand check is involved.** `spoil` and
`exit` are both tiers: a gleaning maze withdraws the way out and an ordinary one
scatters nothing, so the tier that does not apply is simply never true. `if the
stacks has gleaning` exists for a spell that wants to do something *else* per
errand. `cargo test -p orbs-sim --test gleaning` holds it across four seeds.

**`threading` solves the maze in front of it and stops.** It does not re-`research`
— a ladder whose fragment count keeps climbing cannot be observed for one walk.

**The maze is 176 cells in a 33×23 picture of squares, one character each** — a
wall is a square, not a line between two cells, so **one arrow press moves one
character**. It was cells with the walls between them, which draws `2w+1` across
and moved the reading two characters a step. At 33 wide the whole picture wants
`ORBS_GRID=160x45`; at the game's own 120×45 the inline map pans instead.

**The inline map refuses rather than truncating**, and it takes **columns, never
rows** — taking rows under a `Top` panel leaves the deep-focus floor a five-row
transcript. `panel::split` runs first so the instrument panel always wins the
pane. If a grid is too small the map simply is not there, which is correct and
not a bug.

**A walked path is a solid run of `▒`**, because the corridor squares are walked
too. A dump showing marks with gaps between them is showing a regression to the
old cells-and-wall-lines geometry, not a maze.

Each `;`-separated line goes through `submit` and a real `step`. Phosphor, the
CRT curve and the blinking caret are frontend enrichment (rule 2) and are not in
a Frame — those still need eyes on a window.

**So does the 4:3 fit, and it is the one thing here with no text gate at all.**
The grid is fixed at 120×45 and the window only scales it (§19), so what a resize
changes is a *projection* — and a dump builds no `App`, so it has no camera and
no projection to change. Drag the window instead:

```bash
cargo run -p orbs      # then drag it wide, tall, and square
```

- the picture stays 4:3 and centred; bars grow on one axis, never both
- **no text reflows** — the same words stay on the same rows throughout, which is
  the whole point and is what a resize used to break
- the **tower rail's foot** tracks the drag on its `scale` row while `grid` holds
  — it was the telemetry pane's row until the rail replaced that pane
- one `window … -> scale … -> grid 120×45` line per resize in the log

`ORBS_CAPTURE=1` earns its keep here for once, because the bars are pixels and
nothing else: a real window does composite under a normal desktop session, and
this is the case where the black rectangle would be the *answer* rather than the
failure. Check the picture is 4:3 by measuring it, not by trusting it.

**And crop a corner before believing the tube.** The CRT's shaped terms — barrel,
vignette, edge mask, rounded bezel — are all in *tube* space, so they belong to
the 4:3 picture and the bars are the dark room. A full-window screenshot is too
small to show whether the corner is actually round: the bezel spent a version
rounding the unwarped tube rect, whose corners are outside the picture, so it
did nothing at any radius and nobody saw. One crop settled it.

```python
python3 -c "
from PIL import Image
im = Image.open('orbs-screenshot.png')
im.crop((240,840,700,1080)).resize((1380,720), Image.NEAREST).save('/tmp/corner.png')"
```

**The POST card names what the machine is made of, and each frontend answers for
itself.** `ORBS_BOOT=post` is halfway through the card, where only the studio has
typed itself — so the two version lines had no See-it line at all until
`:<fraction>` was added. The engine line is the one thing on that card that
differs between the builds, and each holds its own pin to its own manifest with a
test:

```bash
ORBS_BOOT=post:1 ORBS_DUMP=1 cargo run -q -p orbs        # ...bevy 0.19.0
ORBS_BOOT=post:1 cargo run -q -p orbs-tui -- --dump 1    # ...crossterm 0.29
```

`ORBS_BOOT` takes `dark`, `frame` or `post` for the dump — each with an optional
`:<fraction>` naming how far through — and `0` to skip the sequence in the
running game. Boot happens once per launch and runs for
thirteen seconds, so without the latter every "see it" pass on anything else
costs that wait. **`0` is the only skip there is** — the keypress skip was
removed (§19), so a player sits through the whole sequence every time.

### The terminal build — the game, driven and read back

**`ORBS_DUMP` is a still photograph and this is the running game.** A dump builds
no `App`, advances no clock and presses no keys, so every animated thing, every
*edge* and every interactive surface needs an environment variable of its own —
there are **eighteen** of them now. `orbs-tui` needs none: it is the same `Frame`
through the same painters, with a real clock and a real keyboard, under `tmux`.

```bash
scripts/tui.sh start                      # 120x45 — `orbs_render::GRID`
scripts/tui.sh start 177 38               # ...or any size
scripts/tui.sh type 'attend laboratory' 'kindle charcoal' 'grind sage'
scripts/tui.sh key Down Down Left         # arrows and named keys — `key F5`
scripts/tui.sh wait 20                    # let the world run
scripts/tui.sh see                        # the screen, as text
scripts/tui.sh ink 158 159                # ...and what colour each cell is
scripts/tui.sh stop
```

**`ink` is the one instrument nothing else in the project has.** `ORBS_DUMP`
prints glyphs and a list of tinted *regions*; the terminal build resolves colour
somewhere else entirely (`orbs-tui/src/theme.rs`), so a fire drawn in the wrong
ramp is invisible to every other tool. `see -e` hands back escape sequences
nobody can read down a column. `ink` walks the SGR state machine and prints, per
cell, what the terminal was actually told to draw:

```text
  7  '█':dark-yellow/dim      ← the athanor, at its cool end
 30  '█':yellow/bold          ← ...and its core, twenty rows down
```

**It found a real one on its first use.** The flame ran `dark-red → white`, which
is red at the cool end and colourless at the hot one, against §19's *"the athanor
burns orange on every tube by decision"* and `ember.rs`'s `#CC7024 → #FFE375`.
Sixteen indices hold two colours in that family, so the other two steps come from
the **weight** axis — dim-dark, dark, bright, bold-bright. `every_ramp_climbs`
and `the_fire_is_orange_all_the_way_up` hold it now.

...which is `tmux` and nothing else, if you would rather see it:

```bash
cargo build -p orbs-tui                   # **build first** — `cargo run` prints
tmux new-session -d -s orbs -x 120 -y 45 \#   its progress into the pane
  target/debug/orbs-tui
tmux send-keys -t orbs 'attend laboratory' Enter
tmux capture-pane -t orbs -p              # `-e` keeps the colour escapes
tmux kill-session -t orbs
```

**Resize it while it runs** — that is a screen worth looking at, and it was
broken until `blit::Screen::resize` learned to wipe. The layout reflows, the rail
drops into the border title when the columns run short, and below 80×22 the
"orb needs a larger window" card takes over:

```bash
for s in 170x45 60x18 140x40 90x24 120x45; do
  tmux resize-window -t orbs -x "${s%x*}" -y "${s#*x}"; sleep 1.6
  tmux capture-pane -t orbs -p | head -2
done
```

**Pace typing to the tick, or commands batch.** `Sim::submit` queues a line for
the *next* tick, so two lines sent inside one second land on the same tick with
no time passing between them — `empty mortar_and_pestle` and the `grind` that
needed it arrive together and the second refuses. `tui.sh type` sleeps 1.1 s per
line; `tui.sh key` does not, because `Sim::walk` is the third entry point and
spends no world time at all. **Check `tick` at the rail's foot after a long
walk**: eight arrow presses must not have advanced it eight seconds.

**Escape followed by a letter is two keystrokes and a terminal cannot say so.**
Escape is one byte, `\x1b`, with no terminator; crossterm reads the pty in one
syscall, so `\x1b` and anything behind it in that read come back as
`Alt+<letter>` — the Escape **gone**, the letter delivered. Closing the editor and
typing `quit` fast enough put `quit` in the buffer as a line of the spell.

`drive::escape_prefixed` splits the pair back apart. It answers **only for
`KeyCode::Char`**, because that is the one shape an `\x1b <byte>` pair builds; a
real `Alt+Left` arrives CSI-encoded and is left alone, or a player would leave
the editor by pressing word-left. Suspect this whenever a key "does nothing" in
`orbs-tui` and works with a `sleep` in front of it — and note `ORBS_DUMP` writes
no key events, so it can never see any of it.

```bash
cargo test -p orbs-tui --bins escaping   # both directions of the split
```

**All four surfaces work, and three of them are keys rather than words.**

```bash
scripts/tui.sh start
scripts/tui.sh type 'attend archive' 'research' 'wander'
scripts/tui.sh key Down Down Left         # the reading walks; `▒` marks the trail
scripts/tui.sh key Escape                 # ...and the prompt has the keys back
```

- **the editor** — `scribe <name>`, then `edit`, the lines, `Escape`, `quit`.
  There is still no `save`: stop typing and the buffer writes itself out a beat
  later, which is the same `Editor::settle` the Bevy build runs.
- **the weave screen** — `weave` opens it and **`quit` closes it**; `Escape` only
  returns to command mode, exactly as on the other side.
- **the maze** — `wander` hands over the arrows, `Escape` gives them back.
- **the transcript** — `unfurl`, then `PgUp`/`PgDn`, `Escape` out. It pages by
  *records*, measured with `orbs_shell::page_step`, which both frontends share.
  **`PgUp` works without `unfurl` too**, in both builds: the word exists because
  the *key* could not be discovered, not because the key needs permission.

**Every key the Bevy build binds, the terminal build binds** — and it shipped
once without them, while `prompt.rs` drew `F4 deep` into the border on every
frame. A key the screen offers and the build ignores is §19's *"the first
affordance the game showed was one that did not work"*, so the rules behind them
live in `orbs_shell::shortcuts` where a second copy cannot go missing.

| Key | Does | In a terminal |
|---|---|---|
| `F4` | flips the focus mode | the border's own hint moves; **the tiling does not**, in either build, until Phase 9a returns the second pane |
| `F5` | §14's linear stream | the pane describes itself instead of drawing — the accessibility route, and this is the build §14 calls the cheapest one |
| `F6` | writes `orbs-parse.tsv` | silent on success in both builds; the file appearing is the confirmation |
| `F7` | cycles the tonal register | **visibly inert** — `Presentation` picks a glyph-atlas *face* and a terminal has the user's. The world still moves, and `F6`'s `register` column shows it |
| `F10` | leaves | as it does under Bevy. `Ctrl-C` and `Ctrl-D` also do, because raw mode makes them ours to answer |

**`F5` is inert over the editor, the loom and the maze, and that is a known
hole rather than a rule.** `prompt::paint` returns early for all three modal
surfaces, so the mirror never runs on the three screens that take the *whole*
pane. Reach their stream with a dump instead — it is the same `Speech`:

```bash
ORBS_BOOT=0 ORBS_DUMP="attend archive; scribe threading" cargo run -p orbs
```

**Read the stream, not only the screen, whenever you touch a painter.** A row
drawn as two `Painter::span`s is *two utterances*, and nothing on screen says so
— the editor spoke `1` and `attend laboratory` separately for four phases with
the comment above it claiming otherwise. `span` speaks, `glyphs` is silent, and
`announce` says a whole row that had to be drawn in several styles. A run split
for **colour** is not a sentence boundary (§19, `0.3.23`).

**...and `quit` is the word for it**, in both builds. Every key above was
undiscoverable — §6 makes this a game played by typing, and nothing on screen
said how to stop. It is the fifth take-once handshake beside `scribe`, `unfurl`,
`weave` and `wander`: the sim records the decision, the frontend decides what
leaving means. **It lands on the tick**, like every verb's effect — a frontend
checking at `submit` time finds the flag unset, which is what the first attempt
did.

`logout`, `put it down` and `stop playing` reach it too. **`leave` and `exit` do
not, and §19 records why**: both fuzzy-collide (with `weave` and `edit`), and
that is the one ambiguity prompt that would offer ending the session beside a
verb people type constantly.

```bash
scripts/tui.sh start && scripts/tui.sh type 'quit'   # the session ends
ORBS_BOOT=0 ORBS_DUMP="quit" cargo run -q -p orbs    # ...and the same word here
```

`F2` (phosphor) and `F12` (screenshot) are **not** bound, and cannot be: there
are no themes to cycle and no pixels to capture. That is rule 2 working — the
terminal loses enrichment and no information.

**`--dump` is the boundary proof, and it is in CI.** Both binaries reach
`orbs_shell::dump_script` through the same painters from the same `Sim`, so:

```bash
ORBS_WIZARD=wizard ORBS_BOOT=0 sh -c '
  diff <(ORBS_DUMP="attend laboratory; grind sage" target/debug/orbs) \
       <(target/debug/orbs-tui --dump "attend laboratory; grind sage")'
```

...prints nothing, or the shell has grown a frontend-shaped hole in it. Every
`ORBS_*` switch works against `orbs-tui` too, so a See-it line written for one
frontend runs against the other unchanged.

**`scripts/dumps.sh <dir>` captures every surface the game can draw** — 58
screens — and is the instrument for a refactor whose gate is that nothing
changes. Run it before and after, then `diff -r`. It pins `ORBS_WIZARD`, because
a baseline that varies with who ran it is not a baseline.

**Count it, do not quote it.** This line said 56 for two versions and the number
was 58; a plan written against it said 52. `ls <dir> | wc -l` takes a second and
the figure is only ever used to notice a screen that stopped being captured.

### `scripts/play.sh` — the game, played, as a test suite

**92 scenarios that type at a real terminal and read the screen back.** This is
the third layer: `orbs-sim`'s tests prove the rules and `orbs-render`'s prove the
picture, and neither presses a key. Run it **after anything touching the sim, the
shell, or the loop** — `orbs-balance`'s standing, for the same reason.

```bash
scripts/play.sh                     # all of it, ~1 minute at six threads
scripts/play.sh routing::           # one area
scripts/play.sh -- --test-threads 1 # one at a time, to watch it play
```

**Not in the gate.** Every scenario is `#[ignore]`d, so `cargo test --workspace`
skips them; CI runs them in a **separate non-blocking job**. Without `tmux` they
print SKIPPED and pass, so read the output rather than the exit code somewhere
new.

**One test target, `tests/playing/main.rs`, and the layout is load-bearing.**
Cargo runs test *binaries* one after another and parallelises only *within* one,
so nine files would be slower than one. `tests/playing.rs` **does not compile** —
a crate root resolves `mod play;` against `tests/`, not `tests/playing/`; cargo
auto-discovers `tests/*/main.rs` instead and leaves the siblings as modules.

**Assertions are scoped to the newest command block, never to the screen**, and
both of the obvious alternatives were tried and measured:

- a plain substring search **matches history** — the clarity chain empties the
  mortar twice, six steps apart and both visible, so the second wait returned in
  **1 ms** against the first one's line and the driver went green on an
  assertion that was factually wrong;
- counting occurrences instead **deadlocks** — the pane holds exactly **8
  command blocks** at 120×45, so from the ninth repeat each new line pushes an
  old one off and the count never rises again.

Four more things the driver has to do, each of which failed loudly first:

- **`send-keys -l`, with Enter on its own call.** Without the literal flag tmux
  reads a word as a *key name* wherever one matches: `end` becomes the End key,
  `up` an arrow, `home` Home. `end` closes every `repeat` and `if` in the spell
  language. `scripts/tui.sh` had this bug for two versions.
- **Flatten before matching.** `RecordView` wraps rather than clips, so
  `research`'s answer lands as `…a way out` / `    is in them` and a needle
  written the way a person reads it matches neither row.
- **`opens()`, not `does()`, for `scribe`/`weave`/`wander`.** They take the whole
  pane, transcript included, so the answer to the word is a *screen* and a
  block-scoped wait waits for something that has been painted over.
- **`meditates(n)`, never `does("meditate 20", "meditate")`.** The prompt echoes
  `→ meditate 20` at once, so waiting for the word is satisfied before a single
  tick passes. It caught three scenarios, one of which asked for 7,200 ticks and
  got none.

**Seeds are chosen, not tolerated.** `drift` poisons a log at 1/300 per tick and
`substitution` renames a base reagent at 1/3600, both from the seeded `Threat`
stream — so the schedule is fixed in tick space. Measured: seed 3 poisons a log
inside 200 ticks and seed 0 swaps a reagent inside 7200; **11 and 42 are quiet
through both**, and `play::QUIET` is 11.

**Both frontends boot** (§19, `0.3.12`). `orbs-tui` skipped §4's sequence for a
version on an "instant-startup" argument that was really about the development
loop — which `ORBS_BOOT=0` already answered. The clock, the stages and the card
are `orbs-shell`'s; the terminal supplies a frame and the engine line, so the
card reads `crossterm 0.29` where the other says `bevy 0.19.0`.

**The world does not tick through it**, and that is a determinism rule rather
than a nicety: `tower::drift` rolls once per tick, so a sim running through nine
and a half seconds of animation would reach a different world on the same seed
depending on how fast the machine drew a logo. Nothing is typed during boot
either — except **leaving**, which a terminal needs because raw mode makes
`Ctrl-C` ours to answer or nobody's.

```bash
cargo run -p orbs-tui              # ...and watch it. 9.6s, then the tower
ORBS_BOOT=0 cargo run -p orbs-tui  # ...or don't
```

**Colour is indexed ANSI 0–15 and inherits the user's terminal theme** (§19), so
there is no palette to tune here and no contrast to verify — the numbers belong
to whoever configured the terminal. What holds §14 up instead is the other half
of the rule: the glyph carries the identity. Three degradations are accepted and
recorded — `Presentation` renders identically, a mixed `Wash` takes its first
tint, and there is no CRT.

**Ambiguous-width glyphs are a correctness item, not polish.** CP437's symbols —
`‼ ► ☼ ○ ♂ ♀ ♦ ♠ Ω ░ ▒` — are East Asian *Ambiguous*, so a CJK locale or a
terminal set to `ambiguous = wide` gives them two columns, which shifts the row
**and** invalidates the per-cell diff. `orbs-tui` measures rather than assumes:
it prints one at a known column and reads the cursor back, then swaps in ASCII if
the answer is two. `cargo test -p orbs-tui` holds the table.

### Read the log, not only the screen

**The gate can be entirely green while the game prints 250 GPU errors a second.**
That is not hypothetical: the boot sequence was the first thing to hold a blank
screen for more than one frame, Bevy 0.19 answers a zero-vertex mesh with
`use-after-free`, and `cargo test`, `clippy`, `rustdoc` and `cargo build` all
passed throughout. §15's "work is done when it has been looked at" includes
looking at stderr.

```bash
cargo run -p orbs 2>&1 | grep -iE "error|panic|warn"
```

### Editing shaders

**`embedded_asset!` does not make cargo rebuild when the `.wgsl` changes.** Edit
a shader on its own and the binary keeps the old one, silently — a change that
appears to do nothing, and a diagnostic that appears to prove the opposite of
what is true. Touch a `.rs` file in the same crate, or `touch
crates/orbs/src/crt/plugin.rs`, to force the re-embed.

### Other

- **Plans get independently reviewed before being presented.** The design document
  went through four such reviews and each found load-bearing problems; the practice
  is cheap and worth keeping.
- When a decision is made, record it in DESIGN.md §19 rather than only in
  conversation.
- Update [docs/ROADMAP.md](docs/ROADMAP.md) status as phases progress — and bump
  the version with it. See *Finishing a step* above.
