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

**Status: pre-production. No implementation code has been written.**

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

- **Plans get independently reviewed before being presented.** The design document
  went through four such reviews and each found load-bearing problems; the practice
  is cheap and worth keeping.
- When a decision is made, record it in DESIGN.md §19 rather than only in
  conversation.
- Update [docs/ROADMAP.md](docs/ROADMAP.md) status as phases progress.
