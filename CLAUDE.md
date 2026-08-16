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
| [docs/CHANGELOG.md](docs/CHANGELOG.md) | Release notes — **and the source the GitHub Release, Discord and Bluesky all read from**. Its format is load-bearing; SETUP.md §4 has the rules |

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
```

The `screens` example renders the §4 boot report and a multiplexed siege through
the same public API the frontends use. **It has already found bugs that the full
test suite did not** — an em-dash in DESIGN.md's own boot text that CP437 cannot
draw, and pane content eating a border because a sub-painter was not established.
Add a screen to it whenever a new surface is built.

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
first segment is a word (`edit`, `interpret` or `quit` — that is the whole
vocabulary), `edit` drops into the buffer, and the token `<esc>` comes back out.

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
It lists what resolves *where you are standing* — `execute::offered`, the same
filter the boot report uses — grouped by `Verb::group()`. Check it at the floor,
because a 25-verb listing is most of the transcript at 80×22 and that is where it
has to read:

```bash
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; help" cargo run -p orbs
ORBS_BOOT=0 ORBS_DUMP="attend archive; help" cargo run -p orbs   # no grind
```

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

**The session pane is 60 columns, not 120.** Panes tile side by side, so the
grid's width is never this surface's. `ORBS_GRID=80x22` is narrower still and is
where a sentence stops fitting — worth a look, even though the game itself no
longer reaches it.

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
# **It pans** at the game's own grid: a session pane is 58 columns and the whole
# picture wants 35, which does not leave the transcript its floor, so the block
# shows the part the reading is standing in. That is the designed fallback, not a
# defect — see `stacks::split`.
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
tick: 2, message: the reading goes west
tick: 3, message: the reading goes west
```

**An arrow moves the reading immediately and consumes no tick.** `Sim::walk` is a
**third entry point** beside `submit` and `step` — the only thing in the game
that reaches the world without a tick boundary — so a player walks as fast as
they can press and no brew advances while they do. Three versions went through
the prompt's queue first; all were some flavour of too slow, because the queue
was solving the wrong problem.

`ORBS_WALK` therefore steps **nothing**: check `tick` in the telemetry pane after
a long walk and it should be exactly what it was before. If a dump of eight
presses has advanced the clock eight seconds, something has gone back through
`submit`.

**Replay still holds**, and `Submission::Walked` is what makes it hold: it says
*when* — a typed line executes at the start of the next tick, a walk has already
executed. Use `Sim::replay` rather than matching on `Submission` by hand; three
test files had their own copy of that match and they are one now.

**Watching a spell solve it is the point of the map**, and the solver is fifty
lines, so build it rather than typing it. `else` is load-bearing — a flat ladder
of sixteen `if`s casts clean and oscillates for ever (§19):

```bash
python3 -c '
lines = ["edit", "repeat 400"]
for r in ("exit","passage","walked","twice"):
    for w in ("north","east","south","west"):
        lines += [f"if {w} has {r}", f"follow {w}", "else"]
lines += ["end"]*17 + ["<esc>", "quit"]
print("\\n".join(lines), end="")' > /tmp/solver.txt
ORBS_BOOT=0 ORBS_DUMP="attend archive; research; scribe threading" \
  ORBS_EDIT="$(cat /tmp/solver.txt)" \
  ORBS_THEN="invoke threading; meditate 300" cargo run -p orbs
```

**The maze is a 15×15 grid of squares, one character each** — a wall is a square,
not a line between two cells, so **one arrow press moves one character**. It was
cells with the walls between them, which draws `2w+1` across and moved the
reading two characters a step.

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
- the telemetry pane's `scale` row tracks the drag while `cols` and `rows` hold
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

`ORBS_BOOT` takes `dark`, `frame` or `post` for the dump, and `0` to skip the
sequence in the running game. Boot happens once per launch and runs for
thirteen seconds, so without the latter every "see it" pass on anything else
costs that wait. **`0` is the only skip there is** — the keypress skip was
removed (§19), so a player sits through the whole sequence every time.

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
