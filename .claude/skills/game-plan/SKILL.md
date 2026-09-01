---
name: game-plan
description: >
  Plan and design new features for the O.R.B.S. Bevy game project. Use when the user wants to plan
  a new feature, verb, instrument, domain, screen, recipe, spell-language word, or any gameplay
  addition. Triggers on: "plan a feature", "design a new verb", "add an instrument", "new domain",
  "new screen", "game plan", or any request to architect a new addition to the O.R.B.S. codebase.
  Produces a structured implementation plan with file lists, integration points, and step-by-step
  instructions that follow the project's established conventions.
---

# Game Plan — O.R.B.S. Feature Planning

Plan new features by following established codebase patterns exactly. Use Context7 MCP tools for
Bevy and Rust documentation when needed — and the project's own `bevy-idioms` skill, which is
pinned to 0.19 and ships a compile-verified harness. **Do not recall Bevy syntax from memory.**

## Read these first, in this order

1. **[docs/DESIGN.md](../../../docs/DESIGN.md)** — authoritative for everything. **§19 is a
   decisions log**: check it before proposing anything, because a surprising number of "obvious"
   ideas are recorded there as already tried and reversed, with the reason.
2. **[CLAUDE.md](../../../CLAUDE.md)** — architectural rules 1–8, code conventions, and the gate.
3. **[docs/ROADMAP.md](../../../docs/ROADMAP.md)** — whether this is even the next thing.

## Instructions

When this skill is activated, **immediately enter Plan Mode** using `EnterPlanMode`. Then:

1. **Classify** the feature into one or more categories below.
2. **Read** the relevant reference file(s) for the integration points.
3. **Read the nearest existing example in full.** This codebase documents its decisions in the
   code; a sibling verb or instrument is a better template than anything a reference file can
   summarise.
4. **Check §19** for whether this was already decided.
5. **Review for reuse** — extract shared patterns rather than writing a second copy. §19 is full
   of defects caused by two expressions of one rule disagreeing.
6. **Have the plan reviewed** by a context-free Staff Engineer agent before presenting it
   (CLAUDE.md's standing practice). Give it only the plan file path.

## Feature Categories

| Category | Reference | When to use |
|---|---|---|
| New verb | [verbs.md](references/verbs.md) | Any new word the player can type |
| New instrument | [instruments.md](references/instruments.md) | A thing in a domain that transforms materials over ticks |
| New surface | [surfaces.md](references/surfaces.md) | A screen that takes the pane and the keyboard — editor, weave, map |
| New material or recipe | [instruments.md](references/instruments.md) | Content-only: `recipes.toml` + `materials.toml` |
| New domain | [instruments.md](references/instruments.md) | §10's remaining five. The archive is the worked example; a domain is a node, its instruments, and its verbs |
| New spell-language word | — | `parser/spellword.rs`, `tower/spell/{program,compile,run,watch}.rs`. Rare and load-bearing; read all four before proposing |
| Frontend enrichment | — | CRT, phosphor, audio. **Rule 2**: it may carry no information absent from the Frame |

Features spanning several categories read all the relevant references. A new domain is usually all
three at once.

## The rules that actually catch people

The full set is in CLAUDE.md. These are the ones a plan gets wrong:

- **`orbs-sim` depends on `bevy_ecs`, never `bevy`.** No rendering, no windowing, no assets, no
  GPU. It must compile and test headlessly in milliseconds. `tests/boundaries.rs` enforces it by
  scanning for forbidden names.
- **Determinism is architected.** Any randomness comes from a **per-subsystem** `RngStream` so
  adding a roll cannot perturb another system's sequence. Adding a stream means a new variant with
  a new index — never renumber, or every existing replay is invalidated.
- **No `async` in `orbs-sim`. Ever.** No `tokio`. Long work goes on a worker thread, not a runtime.
- **Structured records everywhere.** Commands emit records; presentation is a view over the record.
  A record is the source for the pane, the log, `sift`, the screen reader and the harness at once.
- **Prose lives in `content/*.toml`, never as string literals in Rust.** The budget is ~88k words.
  The exception is parser *tables* — `Verb::canonical` is a const table, not prose.
- **All output is linear and semantic.** Every frame carries a `Speech`; a cell grid read row by
  row is box-drawing characters, not sentences.
- **Every `Update` system has a `run_if()` guard.**
- **Files over ~300 lines get split**; `plugin.rs` is registration only; `mod.rs` is `mod` + `pub
  use` only; `styles.rs` is forbidden.

## Every plan ends with a See-it line

**This is not optional and it is not a test.** CLAUDE.md: *"Work is not done when it compiles. It
is done when it has been looked at."* An item whose See-it line does not work is not finished,
however green its tests are — the rule exists because ~10,000 lines were once built that the
binary called under 40% of.

So a plan must say **how a person reaches this from the running game**, concretely:

```bash
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; <the thing>" cargo run -p orbs
```

[docs/SEEING-IT.md](../../../docs/SEEING-IT.md) lists every switch — `ORBS_DUMP`, `ORBS_GRID`, `ORBS_SEED`, `ORBS_EDIT`,
`ORBS_THEN`, `ORBS_WALK`, `ORBS_WEAVE`, `ORBS_TICK`, `ORBS_FIRE_PHASE`, `ORBS_LOAD`, `ORBS_FLARE`,
`ORBS_LINE`, `ORBS_SCROLL`, `ORBS_BOOT`, `ORBS_CAPTURE`. **A dump advances no clock and builds no
`App`**, so anything animated, anything on an edge, and anything about the window's *fit* needs
either its own switch or eyes on a window. If a gate is genuinely impossible yet, name the phase
that provides it rather than inventing a debug affordance nobody will maintain.

## Finishing a step is three things at once

A step is not done until all three have happened together:

1. Its checkbox in `docs/ROADMAP.md` is ticked, with a **See it** line that works.
2. The workspace version in `Cargo.toml` is bumped — `0.<phase>.<step>`, patch per step, minor per
   phase. It is player-visible on the POST card.
3. Anything decided along the way is in `DESIGN.md` §19.

A correction folded into an existing step advances nothing.

## Plan Output Format

```
## Feature: [Name]

### Context
Why this is being built — the problem, what prompted it, the intended outcome.
Cite the DESIGN.md section it comes from, and §19 if it revises a decision.

### Summary
One paragraph: the feature and what it changes for a player.

### Files to Create
- `crates/<crate>/src/<path>.rs` — one-line purpose

### Files to Modify
- `crates/<crate>/src/<path>.rs` — the specific change

### Content
- `crates/orbs-sim/content/<file>.toml` — keys added, and roughly what they say

### Implementation Steps
1. ...

### Verification
The gate:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
    cargo build -p orbs

See it:
    ORBS_BOOT=0 ORBS_DUMP="..." cargo run -p orbs

Tests: what is asserted, and what property each one holds.

### Docs
- [ ] ROADMAP item ticked with a working See-it line
- [ ] Cargo.toml version bumped
- [ ] DESIGN.md §19 entry for anything decided
- [ ] docs/SEEING-IT.md gains a section for the new surface — and a switch, if one was added

### Risks
What could be wrong, and what would show it.
```
