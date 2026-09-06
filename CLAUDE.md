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

**Status: Phases 0, 0.5, 1, 2, 4, 5 and 8 closed; Phase 3 met on its exit with
three boxes deliberately left. Phase 9 (Enchanting) is met on three of its four
boxes — the shared-engine extraction is the one left. Phase 10 (Progression)
closed at `0.10.6`; Phase 11 (the tower as one machine) remains.** The determinism spine, the Frame boundary, the parser, the
cell renderer, brewing, the archive, the lens, the sanctum, the menagerie, the
bailey, the forge, the tower rail, the balance harness, the spell engine and its
scripting language are built and the game plays.

**§10's seven domains are all raised, and a fresh game opens them one at a
time.** `Sim::new` is the open tower every test, dump and balance policy uses;
`Sim::sealed` is what a player gets — a laboratory, and the rest earned along
the mastery lines — so the rail's dark boxes are back for exactly the rooms
§11.5's breadth track always promised. `ORBS_SEALED=1` shows a dump the sealed
start.

**The phases were renumbered so the version could keep climbing** (§19), four
times now. Enchanting was Phase 6 and is **Phase 9**; **Progression is 10** (the
Ley Line, Mastery, and the tower that opens as you work); **Renown is 11** (the
second number, and a reason to keep making things after the Ley Line stops
buying); the tower-as-one-machine is **12**, breadth/remote/engine are
**13a/13b/13c**, onboarding is **14** and ship is **15**. Phases 0–5, 8, 9 and
10 did not move — they are closed or tagged and their tags mean what they meant.

**The scheme has now been bent four times for this.** `0.<phase>.<step>` assumes
phases are built in order and four times they have not been. §19 records the
alternative — a minor that counts phases *closed* rather than naming the phase —
and that it is **still undecided and now overdue**: the third renumber's own note
said a fourth phase out of turn should settle the scheme instead, and the fourth
renumbered anyway. Each has been cheap only because none of the moved phases
carried a tag, and that luck is not a plan.

**A renumber is done highest-first, always.** A mechanical pass that moved a
lower number first collided two phases into one, and a replace-all once corrupted
two *historical* renumber tables in §19 — those record what the numbers were at
the time and must never be shifted.

**The premise is complete.** *"Sieges then test everything you automated — because
the enemy attacks the automation"* was the last unbuilt clause and closed at
`0.8.7`: six domains produce, the arsenal carries their output across the tower,
and a siege spends it while the enemy rewrites the scripts behind you.
See [docs/ROADMAP.md](docs/ROADMAP.md) for what remains.

**Phases 2–7 are §10's five remaining domains** — scrying, spellcraft, defense,
summoning, enchanting — and the phase that makes them one machine. The siege
moved from Phase 2 to **Phase 8**, because a series of puzzles has to exist
before the thing that consumes them.

**Phase 8 was built before 6 and 7, and nothing gated it** (§19). A domain stands
alone; the arsenal, multiplexing and the weave are *enhancements* to that rather
than prerequisites for it — four shipped domains take no production slot and
`orbs-balance` measures them as **additive**. An earlier plan called the
one-machine phase a hard dependency on the premise that a siege holds the slot;
it does not. **Phase numbers in older notes are stale twice over** — six lower
from Phase 2 down, and then moved again for Enchanting and everything above it;
DESIGN.md §19 records both shifts.

**Defense and Enchanting swapped** (§19), and then Enchanting moved again.
Nothing in defense depended on either derived domain, and the version is
`0.<phase>.<step>` and drawn on the POST card — so building the later phase first
would have made a tester's version number go backwards. It did anyway, when the
siege shipped ahead of the forge, which is what the second renumber above fixes.

**The design is authoritative and lives in [docs/DESIGN.md](docs/DESIGN.md)** —
~2,000 lines, eight drafts, four independent staff-level reviews. Read it before
making design decisions. Do not re-litigate settled decisions; §19 is a decisions
log recording what was decided and why, including superseded choices marked as
such.

| Document | Purpose |
|---|---|
| [docs/DESIGN.md](docs/DESIGN.md) | The design. Authoritative for everything |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Phase status checklist — *derived* from DESIGN.md §15 |
| [docs/SEEING-IT.md](docs/SEEING-IT.md) | **How to reach every surface from the running game.** The reference half of *Seeing it* below — read the section for whatever you are changing |
| [docs/SETUP.md](docs/SETUP.md) | Toolchain, skills, build/test/CI procedures |
| [docs/CHANGELOG.md](docs/CHANGELOG.md) | Release notes — **and the source the GitHub Release, Discord and Bluesky all read from**. Its format is load-bearing; SETUP.md §4 has the rules |

## Technology Stack

- **Rust**, edition 2024, pinned toolchain
- **Bevy `=0.19.0`** — exact pin, upgraded deliberately (one window in Phase 12c)
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

### ...and [docs/SEEING-IT.md](docs/SEEING-IT.md) is how

**Every surface in the game has a See-it line, and they live in that file.** It
was in this one and grew to 174k characters — nine tenths of a document loaded
into every session whether or not the work touches a siege. Nothing was reworded
in the move.

**Read the section for whatever you are about to change.** A See-it line there
has exactly the standing it had here: it is the gate, not supplementary reading.

| You are touching | Read |
|---|---|
| a domain — laboratory, archive, lens, sanctum, menagerie, bailey, forge | that domain's section |
| quintessence, charms, or anything the forge buffs | *The forge*, *Quintessence is the tower's* |
| the spell language, the editor, or highlighting | *A spell's verb is read at cast*, *A spell is highlighted*, *The satchel* |
| a duration, a rate, or anything the economy touches | *`orbs-balance`* — and **run a sweep** |
| a painter, a record, or the way something reads | *The output style*, *The terminal build* |
| a screen changing — a domain, a tool, `F5` | *The passage* — and it is one of the few that genuinely needs a window |
| accessibility, colour, or the tube | *Greyscale* |
| the manual, `recall`, or a room's primer | *`recall apprentice`* |

**Four things from it are worth carrying without looking**, because they are the
ones a session gets wrong before it thinks to check:

- **Run a sweep after anything that touches the world**, not only after a
  duration. `cargo run -p orbs-balance -- sweep --ticks 7200` — an hour is short
  enough that one badly-timed sabotage flags the seed rather than the game, and a
  swept rate is the only instrument that sees an ambient nuisance at all.
- **A spell's records are in the log, never the transcript.** The pane draws what
  the *player* did, so a dump that casts a spell and looks for its output on
  screen finds nothing and looks broken. `peruse` or `sift` the log.
- **`ORBS_DUMP` is a still photograph.** It builds no `App`, advances no clock and
  presses no key, so anything animated, anything on an *edge*, and every
  interactive surface needs a switch of its own or has no gate at all.
- **`scripts/dumps.sh <dir>` captures every surface as text** and is the
  instrument for a refactor whose claim is that nothing changed. **A domain built
  without a block in it is one this instrument is blind to, and the blindness
  looks exactly like stability** — Phase 8 shipped the bailey that way.

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
