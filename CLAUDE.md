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

**Status: Phase 0 in progress.** The determinism spine, the Frame boundary, the
parser, the font assets, and the cell renderer are built and the game draws.
See [docs/ROADMAP.md](docs/ROADMAP.md) for what remains.

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

## Technology Stack

- **Rust**, edition 2024, pinned toolchain
- **Bevy `=0.19.0`** — exact pin, upgraded deliberately (one window in Phase 3c)
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
├── orbs/           Bevy frontend: GPU cell renderer, CRT, audio, Steam
├── orbs-tui/       terminal frontend (second-class, cuttable)
└── orbs-balance/   CLI harness driving orbs-sim
```

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

- **Never commit or push without explicit approval.** Ask, then wait.
- Commit messages: no AI/agent attribution of any kind — no `Co-Authored-By`,
  no generated-with footers, no session URLs.
- Branch from `main`; JIRA-style prefixes are not used on this project.

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
```

The `screens` example renders the §4 boot report and a multiplexed siege through
the same public API the frontends use. **It has already found bugs that the full
test suite did not** — an em-dash in DESIGN.md's own boot text that CP437 cannot
draw, and pane content eating a border because a sub-painter was not established.
Add a screen to it whenever a new surface is built.

**Reach for `ORBS_DUMP` first; keep `ORBS_CAPTURE` for what only pixels show.**
The screenshot path needs a composited window, and without one it writes a valid
PNG of a **black rectangle** — the renderer fine, the picture proving nothing.
That is worse than no picture, because it looks like evidence. `ORBS_DUMP` draws
the same frame through the real `paint`, `Sim` and `ScreenLayout` into a `Frame`
nobody rasterises, then prints it with its linear stream beneath:

```bash
ORBS_DUMP="attend alembic; decoct clarity; meditate 25" cargo run -p orbs
ORBS_DUMP=1 ORBS_GRID=160x44 cargo run -p orbs   # the worst-case grid
ORBS_DUMP=1 ORBS_BOOT=post cargo run -p orbs     # a boot stage as text
ORBS_BOOT=0 cargo run -p orbs                    # skip the boot sequence
```

Each `;`-separated line goes through `submit` and a real `step`. Phosphor, the
CRT curve and the blinking caret are frontend enrichment (rule 2) and are not in
a Frame — those still need eyes on a window.

`ORBS_BOOT` takes `dark`, `strike`, `prompt`, `frame` or `post` for the dump, and
`0` to skip the sequence in the running game. Boot happens once per launch, so
without the latter every "see it" pass on anything else costs a four-second wait.

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
- Update [docs/ROADMAP.md](docs/ROADMAP.md) status as phases progress.
