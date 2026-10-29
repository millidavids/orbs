# O.R.B.S. — Roadmap

**Status tracker. Derived from [DESIGN.md](DESIGN.md) §15, which is authoritative.**
If the two disagree, DESIGN.md wins and this file is wrong.

> **Every item carries a "See it" line, and it is a completion gate — not a note.**
> No work item in any phase is done until a person can reach it from the running
> game. An item without a See it line is not started; an item whose line does not
> work is not finished, however green its tests are. DESIGN.md §15, §19.

## The list is not an order

**This file is a list of work, not a sequence of phases.** An item is open, or it
is done and carries the phase name it was done under. `0.<phase>.<step>` assumed
phases were built in sequence, and four times they were not (§19).

### What the version means now

> **The minor names the large feature in hand. The patch is an iteration within
> it. `1.0` is the release on Steam.**

No tag moved: Renown was already minor `11` with patches `0`–`14`. **`0.12` is
the orb's menu**, not "Phase 12".

### The renumbers, kept as history

**The tags mean what they meant** — `v0.8.x` is the siege, `v0.10.x` is
Progression, `v0.11.x` is Renown. Nothing below is edited again.

| pass | what moved | why |
|---|---|---|
| first | Siege 2 → **8**; the five domains take 2–7 | *"If we don't have a series of interesting puzzles, then there really is no game"* |
| second | Enchanting 6 → **9**, and 7 / 9a–c / 10 / 11 up with it | Phase 8 shipped first, and the POST card's number must never go backwards |
| third | Progression → **10**, everything above up one | built out of turn |
| fourth | Renown → **11**, everything above up one | built out of turn |

**Every pass was done highest-first**, and there is no fifth. Phase numbers
survive below as names, not as an order.

Last updated: 2026-10-28 · `0.23`, *the tour*, closed at `0.23.4`, after `0.22`,
*every instrument has a tempo*, at `0.22.5`.

---

## Shape

Solo, commercial, Steam. Release posture: demo first, then full 1.0. No Early
Access. **The domains come before the siege**, which is why the numbers below are
not consecutive.

### Done

| | Shipped at | |
|---|---|---|
| Phase 0. Vertical slice | — | ✅ numeric gate **deferred** |
| Phase 0.5. Interlude | — | ✅ every box ticked · settings moved to *Ship* |
| Phase 1. Core loop | — | ✅ testers clause moved to *Breadth* |
| **Phase 2. Scrying** `lens/` | — | ✅ every box ticked |
| **Phase 3. Spellcraft** `grimoire/` | `0.3.35` | ✅ exit met · **three boxes left**, open below |
| **Phase 4. Defense** `sanctum/` | `0.4.2` | ✅ swapped with Enchanting (§19) |
| **Phase 5. Summoning** `menagerie/` | — | ✅ |
| Phase 8. Siege | `0.8.7` | ✅ three items deferred, named in the section |
| **Phase 9. Enchanting** `forge/` | `0.9.3` | ✅ three of **five** boxes — the extraction split in two (§19); **two left**, open below |
| **Phase 10. Progression** | `0.10.6` | ✅ every box ticked; the numbers are first-pass and `orbs-balance` decides them |
| **Phase 11. Renown** | `0.11.14` | ✅ every box ticked |
| Phase 11.5. Interlude | `0.11.10` | ✅ every box ticked |
| **The orb's menu** | `0.12.7` | ✅ every box ticked |
| **The augury** | `0.13.19` | ✅ closed |
| **The scrivener** | `0.14.11` | ✅ every box ticked |
| [**The circle**](#the-circle) `menagerie/` | `0.15.7` | ✅ every box ticked · supersedes Phase 5's rhythm game |
| [**The threshold**](#the-threshold--closed-at-01613) | `0.16.13` | ✅ every box ticked · the orb's front door |
| [**Number go up**](#number-go-up) | `0.17.6` | ✅ every box ticked · interleaved with `0.17` |
| [**The installation**](#the-installation) | `0.17.8` | ✅ every box ticked · interleaved with `0.17` · minute one of the apprenticeship |
| [**The orb made yours**](#the-orb-made-yours) | `0.17.15` | ✅ every box ticked · interleaved with `0.17` · settings chosen from research |
| [**Supply**](#supply) | `0.18.7` | ✅ every box ticked · no produced item has individual value |
| [**The terminal kept true**](#the-terminal-kept-true) | `0.19.13` | ✅ every box ticked · every feature since `0.12` played in the terminal; motion, greyscale, and the swap and rasteriser fixes |
| [**The tower swept**](#the-tower-swept) | `0.20.22` | ✅ every box ticked · one copy of each rule, test helpers shared, comments held to the house style |
| [**The tower speaks up**](#the-tower-speaks-up) | `0.21.5` | ✅ every box ticked · a rate, a wait, a forecast and the tower's news; the glass answers and the phosphor lingers · ticks `0.17`'s bottleneck box |
| [**Every instrument has a tempo**](#every-instrument-has-a-tempo) | `0.22.5` | ✅ every box ticked · the prism, the lattice, the pylon, the lectern and the circle draw pictures of their own |
| [**The tour**](#the-tour) | `0.23.4` | ✅ every box ticked · every surface, moving, one scene at a time, in the window and headless; the instrument for judging the look |

### Open — and **this table is not a running order**

| | Months | Words | |
|---|---|---|---|
| [**The tower as one machine**](#the-tower-as-one-machine) — `0.17` | 2 | ~3k | the dependency web, and two defects it cannot open on top of (§19) — the first fixed at `0.17.0` |
| [Spellcraft's three](#phase-3--spellcraft) | — | ~1k | the terse register, a typed action at execution, hidden-directory authoring |
| [Enchanting's two](#phase-9--enchanting) | — | ~1k | the shared-engine extraction, split into a refactor and a behaviour box |
| [Breadth](#breadth) | 2 | ~4k | all seven domains, research, full drift, offline progression |
| [Remote hosts](#remote-hosts) | 3 | ~12k | `connect`, infiltration, trace |
| [Engine upgrade](#engine-upgrade) | 1 | — | one Bevy window, isolated from new-system work |
| [Onboarding + demo](#onboarding--demo) | 4 | ~20k | the apprenticeship, the reveal, the soft ending, the capsule |
| [Ship](#ship) | 3 | ~5k | settings, accessibility, store, launch |
| [**Standing**](#standing--work-that-rides-with-the-content) | — | — | ♾ never closes, and blocks nothing |

**~16 months of open work**, against a whole-plan estimate that reached 41 months
on a 24-month target; the [cut line](#cut-line--decided-in-advance) below says
how the gap closes. §16 rates schedule overrun **Critical**, so the total is
carried rather than dropped with the ordering.

---

## Phase 0 — Vertical slice

**Exit gate is numeric and was set before any results exist:**

> ≥ 8 external testers, at least half self-reporting no shell experience, complete
> a 15-minute scripted scenario with expected-intent ground truth.
> - **≥ 85% of inputs resolve to the intended action on first attempt** (below 70%
>   is a no-go), **and**
> - **≥ 95% of initially-unresolved inputs reach the intended action within two
>   further attempts, with zero dead ends.**
>
> Act on the per-input failure *clustering*, not the aggregate — 8 testers over 15
> minutes has wide confidence intervals, so 84% vs 86% is noise.

**Status: every work item is closed. The gate above has not run, and is deferred
rather than passed** — to Phase 14's onboarding and demo, which put
non-terminal players in front of the game. Someone who knows the vocabulary
measures their memory, so an in-house number would be worse than none.

### Playability status of what is already built

The scope of the retroactive gating pass: built in layer order, tested, and
much of it not called by the game. Every ❌ or ⚠️ left names a phase.

| Built | Reachable from the running game? | Gated by |
|---|---|---|
| Cell renderer, glyph atlas | ✅ it is what you look at | — |
| Phosphor themes | ✅ `F2` | — |
| CRT — all ten effects, peak-threat state | ✅ `F3` | — |
| Determinism spine — seed, tick, `step()` | ✅ `meditate <n>`, `status`, live in the border | — |
| Frame boundary — panes, layout | ✅ two panes from `ScreenLayout`; `F4` switches focus | — |
| Parser — 16 commands, 3 registers | ✅ type at it | — |
| Naming pass — synonyms, canonical echo | ✅ the echo answers in canonical arcane | — |
| Record model, `RecordView` | ✅ every line on screen is a record | — |
| ~~Fidelity tiers~~ → a fixed 4:3 picture | ✅ drag the window: `scale` moves, `cols`/`rows` do not | — |
| `Speech` linear stream | ✅ `F5` shows the session pane as a reader hears it | — |
| CP437 repertoire enforcement | ✅ the prompt refuses what it cannot draw | — |
| Parse instrumentation (`ParseLog`, TSV) | ✅ every line traced; `F6` exports | — |
| `sift` / record filtering | ✅ `sift <pattern> orb.log` | — |
| Eldritch + tampered presentation | ✅ `F7` cycles the register; all three faces draw | — |
| Progress bars (`Painter::progress`) | ✅ a brew draws its meter | — |
| Sidebar (`ScreenLayout::sidebar`) | ❌ example only | **Unblocked** — domain panes are back in scope as step 7 of Phase 1's brewing item (they were cut from it by an earlier review, then restored). §9 opens with two domains at capacity 1, so one is minimised |
| 3- and 4-pane tiling | ❌ the game asks for at most 2 | **Phase 1+ progression.** Gated by *multiplex capacity*, not by domains: §11.5 starts the player at capacity **1** and reaches 3 at ~5 h. §9 keeps panes and capacity as separate unlocks that "must not be conflated" — drawing three panes at t=0 would delete the swap-or-let-it-burn trade the whole focus track is built on |
| `Sim::with_schedule`'s build closure | ⚠️ test-only, and now says so | **Nothing left** — domain systems belong inside `Sim::new`, or the Bevy build, `orbs-tui` and `orbs-balance` each register their own and diverge (§13). Kept because the boundary tests drive it; marked so no frontend reaches for it |
| Replay log (`Sim::submissions`) | ⚠️ written, never read | **Phase 1** — needs a replay command to read it |
| Destruction guard (§7's refusal) | ✅ `purge laboratory` refuses; `purge residue-N` works | — |
| Per-subsystem RNG streams | ⚠️ one of six rolls | **Phase 8 for the rest.** Log-poisoning drift rolls `RngStream::Threat`, so the seeded, per-stream machinery is now exercised by the running game rather than only by tests — and the same seed poisons the same log on the same tick. The other five wait for the subsystems that own them |

**Where a gate is not yet possible, it names the phase** rather than inventing a
debug affordance nobody will maintain.

> **This table is derived from the code**, by sweeping the public API for
> anything the shipping path never calls — the first, from memory, missed five.
> **And it does not choose scope**: a row says a subsystem is unreachable, not
> that the game should reach it (§19).

### Work items

- [x] **Workspace + determinism spine** — `orbs-sim` on `bevy_ecs` alone,
      per-subsystem ChaCha8 streams, a single-threaded schedule, `Sim::step()`
      the sole entry point
      **See it:** ✅ `meditate 30` and watch the tick jump; `status` reports the
      seed; the border carries both live
- [x] **`orbs-render` Frame boundary** — cells with semantic styles and never a
      colour, a `Speech` stream per frame, the CP437 repertoire, `ScreenLayout`.
      `cargo run -p orbs-render --example screens` draws real Frames as text
      **See it:** ✅ the game draws two panes from a real `ScreenLayout`, and
      `F4` switches Deep ↔ Wide — which raises fidelity a step and visibly
      changes both the glyph size and the pane count. `F5` swaps the session
      pane for what it says, so §14's parity is a keypress rather than a claim
- [x] **Bevy frontend skeleton** — `SimPlugin` drives `Sim::step()` from
      `FixedUpdate` at 1 Hz (§5.0). `cargo build -p orbs` joined the gate
      **See it:** ✅ `cargo run -p orbs` — the window opens and the sim ticks
- [x] **Parser + instrumentation** — the 16 slice commands across three
      registers (§6.1), deterministic: normalise → fuzzy match → resolve →
      score → disambiguate → suggest, every scored reading kept for the gate
      **See it:** ✅ type at it, then `F6` to write `orbs-parse.tsv` — one row
      per candidate, with the verb and argument scores the gate clusters on
- [x] **Naming pass** for the slice's 16 commands — measured, not by eye, and
      enforced since by `crates/orbs-sim/tests/naming.rs` (§6.1, §19)
      **See it:** ✅ type any register at the prompt and read the canonical echo
      come back
- [x] **Cell-grid text renderer** — glyph atlas and one mesh, one draw call, a
      worst-case rebuild under 2% of a frame. Four contrast-solved themes.
      `ORBS_CAPTURE=1 cargo run -p orbs` screenshots it
      **See it:** ✅ it is what you look at. `F2` cycles themes; drag the window
      and the `scale` row in the telemetry pane moves while `cols` and `rows`
      hold at 120×45
- [x] **CRT port** — ten effects as a `Core2d` post-process, scanlines and
      grille derived from the cell (§9), fully disableable (§14). The pass sits
      in `Core2dSystems::PostProcess`, or it flashes (§19)
      **See it:** ✅ `cargo run -p orbs`, `F3` cycles default → peak threat →
      off. **The model item for this rule** — it caught its own bug in ten seconds
- [x] **Structured-record output model** — an arena stream of kinded,
      fielded records that four views read, none privileged; §3's exemption by
      construction. It lives in `orbs-render`, the crates' interface, held by
      `tests/boundaries.rs` (§7, §19)
      **See it:** ✅ every line on screen is a record. `sift <pattern> orb.log`
      filters them. `F7` cycles the tonal register through all three typefaces,
      and `peruse orb.log` under it shows §3's exemption holding

      ✅ **The output pass** (`0.4.3`) — a correction folded in, advancing
      nothing: a heading ruled off, a slot `<bracketed>`, local verbs
      described and the tower's tiled, a blank row before each section (§19)
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; help" cargo run
      -p orbs`, and again with `ORBS_GRID=80x22`. **The page no longer fits the
      floor** and that is recorded rather than fixed — it is a record like any
      other and `PgUp` reaches it
> **Everything below is ordered by playability, not by layer** — DESIGN.md §15.
> The first six items were built in layer order and left ~10,000 lines the binary
> called under 40% of. **No item below is done until the "see it" line works.**

- [x] **The prompt** — a real command line, the instrument every item below is
      verified with: `KeyboardInput::text` filtered through the repertoire, a
      viewport, an outcome-aware line view, `Sim::submit` queuing for the next
      `step()`, and a headless keystroke test (§15, §19)
      **See it:** `cargo run -p orbs`, type `look around` → `survey` comes back on
      the tube; type `xyzzy` → suggestions; type `meditate` → it asks for a count
- [x] **Retroactive playability pass** — a hand on everything ⚠️ or ❌ above:
      the picture, the layout and `F4`, `Speech` on `F5`, the repertoire at the
      buffer, `F6`'s trace, `sift`, `F7`'s registers, `meditate`. It found two
      defects no test could have, by looking (§15, §19)
      **See it:** every row of the status table above reads ✅, or names the phase
      that gates it
- [x] **Dev ergonomics** — measured, and the premise did not hold: a leaf
      rebuild is under a second. `--features fast-compile` exists, off by
      default; [SETUP.md](SETUP.md) has the numbers
      **See it:** ✅ `touch crates/orbs/src/shell/prompt.rs && time cargo build -p orbs`
      — under a second, so the typing loop was never waiting on the compiler
- [x] **Brewing + archive** — the two starting domains, thin: stable `NodeId`s,
      names scoped to a domain (§7), one production slot, work as an interval,
      and numbered candidate selection (§19)
      **See it:** `attend laboratory`, `make a potion of clarity` → `decoct clarity`
      with a meter; `research` while it brews → refused, naming what holds
      the slot; `decoct nonsense` → a numbered prompt you answer with a digit
- [x] **Log-poisoning sabotage** on brewing logs, via `peruse` / `sift` / `verify`
      — a seeded roll poisons a log, and the tell is §8.1's structural one
      **See it:** ✅ `ORBS_DUMP="attend laboratory; decoct clarity; meditate 25;
      decoct warding; meditate 25; verify laboratory.log; peruse laboratory.log"` —
      `verify` says `tampered`, and in the `peruse` below it line 1 has lost its
      number while line 2 still has one
- [x] **The audit, and what a look costs** — §8.1's expensive `verify --all`,
      spelled bare: production-class, scaling with the tower, with a cooldown
      per surface that travels in the save. Phase 1 debt, closed at `0.5.13`
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; verify;
      grind sage; meditate 25"` — the audit names its duration, `grind` is
      refused with `the tower is busy verifying`, and the verdict lands
      **See it:** ✅ the rationing, and that it is per surface —
      `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; verify laboratory.log;
      verify laboratory.log; verify dispensary; meditate 21;
      verify laboratory.log"` reads `sound`, then `still reading the log`, then
      `sound` for the shelf, then `sound` again once the wait is served
- [x] **Boot sequence** — a status report walked off the tower, so it cannot go
      stale; `bound: 0` is printed, not omitted (§19)
      **See it:** ✅ `ORBS_DUMP=1 cargo run -p orbs`, or launch the game: the
      report is what is on screen before anything is typed, and it reports the
      tower that actually exists
- [x] **Scaffold tutorial** — folded into the boot report (§4), naming the verbs
      that work rather than the verbs that exist (§19)
      **See it:** ✅ a cold launch lists ten verbs and what each one takes; every
      one of them does something
- [x] **Worst-case legibility test** — four panes, a siege, the eldritch
      register and a one-space tell at the smallest glyph drawn, derived rather
      than assumed; the tell survives to the Frame (§19)
      **See it:** ✅ the last screen of `cargo run -p orbs-render --example screens`.
      ⚠️ **The human read is outstanding**: peak-threat CRT is frontend
      enrichment and is not in a Frame. Size the window to 1280×704, `F4` into
      Deep focus, `F3` to peak threat, `F7` for eldritch, and read the log
- [x] **Run the gate** — ⏸ **deferred to Phase 14, not passed.** Blocked on
      testers and a scripted scenario, not on code
      **See it:** eight external testers sit down and play. That *is* the gate

---

## Phase 0.5 — Interlude

**The orb becomes a machine that moves.** Every item found a rule already
pointing at it, and two found live defects (§19).

- [x] **The CRT's off switch could be flipped back on by something else** —
      `enabled` is explicit state now, not `settings != OFF`, which world-state
      `flash` or `desaturation` could revive (§14)
      **See it:** ✅ `F3` to off and it stays off. The defect was latent when it
      was fixed and is latent again now the boot strike is gone — §4's *flash on
      breach* is what will drive `flash` next, in Phase 8
- [x] **Panes arrive over time** — `ScreenLayout::transition` interpolates and
      the frontend owns the clock: Deep slides in from the right, Wide unrolls
      downward
      **See it:** ✅ `cargo run -p orbs`, press `F4` — the second pane grows in
      from the right instead of appearing. Drag the window across 100×28 and it
      grows and shrinks without strobing, because retargeting reverses rather
      than restarts
- [x] **Command output arrives a character at a time** — pure presentation,
      deliberately: a modelled wait would invert §9's parity rule (§19)
      **See it:** ✅ type `survey` in the laboratory and watch the listing fill
      across then down. Any keystroke completes it instantly
- [x] **A real boot sequence** — black, the border drawing itself, a POST card
      typing `O.R.B.S.` in block glyphs. Paced to be read, with no any-key skip,
      and the world does not tick during it (§19)
      **See it:** ✅ `cargo run -p orbs`. Or as text:
      `ORBS_DUMP=1 ORBS_BOOT=post cargo run -p orbs`, and `dark`/`frame` for the
      rest. `ORBS_BOOT=0` skips it — the only skip there is.
      The prompt is gone from these screens: nothing can be typed during boot
- [x] **The tube strike — built, then cut** — an orb does not power on like a
      monitor. Nothing in the game flashes now (§19)
      **See it:** ✅ launch the game — it opens black and the prompt types itself

**Phase 0.5 is closed.** Its one open item — sticky skip, persisted CRT-off,
reduce-motion — needed somewhere to persist a setting and **moved to *Ship***.
The first two shipped with the threshold at `0.16.5`.

---

## Phase 1 — Core loop ✅ **Closed**

**Exit:** a player automates a duty and feels clever.

**Met.** A spell solves a 176-cell maze on twelve seeds, brews a clarity end to
end, walks the stacks, and assembles a scroll, and automation beating the hand is
mechanical: a script ends with `stop athanor`, a person walks away with the fire
lit.

**The second clause is not met** — *"…non-terminal testers are in the loop"*
needs a person, so it **moved with its item to Phase 14**.

- [x] **World clock** per DESIGN.md §5.0 — already correct; the ~60 s inactivity
      grace was built and struck (§19). **The clock runs whenever the window is
      open.**
      **See it:** ✅ launch, touch nothing, and watch `tick` in the border keep
      climbing; `meditate 30` still lands duration actions on the right ticks
- [x] **Brewing, gamified** — §10's *"sequence/recipe puzzle with timing"*, in
      place of Phase 0's one command. Five instruments (§10.1), `move`, `wield`
      and `stop` in, **`decoct` out**: there is no command that brews a potion,
      and `make a potion of clarity` answers with the recipe (§19).

  1. ✅ **Content format** (rule 6) — `Prose` keyed by situation, compiled in;
        the watcher in the frontend, reloads on a tick boundary. Tests pin the
        lines to CP437, the floor's width, and lower case
        **See it:** ✅ `ORBS_CONTENT=crates/orbs-sim/content cargo run -p orbs`,
        edit `prose.toml`, and the next tick speaks the new line — confirmed in
        the running game, not only in a dump
  2. ✅ **Instruments as places** — six places in the laboratory, one
        `NounKind::Reagent` for everything that is not one, and `purge` in two
        tiers: protected refuses, any other place is emptied, never destroyed
        **See it:** ✅ `ORBS_DUMP="attend laboratory; survey; survey dispensary;
        purge dispensary; purge laboratory"` — six places listed, three reagents
        in the dispensary, the dispensary scoured but intact, the laboratory
        refused
  3. ✅ **The vocabulary and the tools that answer it — one slice.** `move`,
        `wield`, `stop`, and `decoct` retired, with the tools that answer them,
        so the laboratory is never less playable between steps. A finished tool
        releases its slot, or a capacity-1 player who walks away is soft-locked
        **See it:** `move sage from dispensary to mortar_and_pestle`, then
        `wield mortar_and_pestle`; try to move into it and be refused;
        `meditate 600` → finished, not advanced. `make a potion of clarity`
        answers with the recipe
  4. ✅ **The athanor burns, and gates the two heated instruments** — `Burning`
        is an interval, not a `Working`, so the production pool cannot see it.
        Heat is checked when a run starts. The two heated stages are not
        adjacent, so the play is light → digest → damp → combine → relight →
        distil (§10.1)
        **See it:** ✅ `ORBS_DUMP="attend laboratory; move charcoal to athanor;
        wield athanor; meditate 20; stop athanor; wield athanor; meditate 60;
        survey athanor"` — lights, damps and banks, relights, gutters, and leaves
        ash. And `wield balneum_mariae` cold answers *"wants heat, and the
        athanor is cold"*. **Text, not a bar** — the painter is step 6
  5. ✅ **`siphon`, `stop`'s refund, and the triage slot** — a `Product` marker
        rather than a name tells product from byproduct, and clearing is Triage
        work, a component apart from `Working` so no counter can confuse them.
        (`siphon` has since been retired for `empty`, §19)
        **See it:** ✅ `ORBS_DUMP="attend laboratory; grind sage; meditate 10;
        empty mortar_and_pestle; move husks to mortar_and_pestle; purge
        mortar_and_pestle; meditate 5; survey mortar_and_pestle"` — the ground
        sage and its husks come out onto the shelf, and putting the husks back to
        be scoured takes ticks rather than a keystroke
  6. ✅ **The instrument panel** — `Sim::instruments()`, an accessor rather than
        records, since the panel is state redrawn every frame. `Painter::meter`
        draws silently and the panel owes one summary line (§14)
        **See it:** ✅ at the 80×22 floor, `attend laboratory` puts six rows above
        the transcript — one filling, one draining, four at rest — and the
        linearised stream is a single `Progress` line. Added to
        `examples/screens.rs`, where `parity` immediately caught the rows being
        drawn with `span` (which speaks): a Wide strip too short for the athanor
        **said one thing less than Deep**, which §9 forbids outright
  7. ✅ **Byproducts with uses, and `grimoire` alive** — a second route to the
        same draught from a grind's husks, and `grimoire` walking the route tree
        backwards from the goal, so retiring `decoct` left no dead end
        **See it:** ✅ `ORBS_DUMP="grimoire brewing; attend laboratory; make a
        potion of clarity"` — the manual answers, then the full tree with **two
        routes to `clarified-draught`** listed before a single instrument is
        committed
  8. ✅ **The athanor's bar burns** — flame in the filled portion, a sparse plume
        above it, on per-theme ramps through `Style`'s `Depiction`, the one
        channel that means nothing (§14). `FLIP_HZ = 6` is a recorded exemption
        from the 3–30 Hz band. A cold hearth smokes, a guttering one keeps an
        ember, and lighting one flares over one tick (§19)
        **See it:** ✅ `ORBS_DUMP="attend laboratory; kindle charcoal; meditate
        300" ORBS_FIRE_PHASE=0.17 cargo run -p orbs` — step the phase by a tick
        (0.00 → 0.17 → 0.33) and a spark climbs a row each time, cooling `∙` →
        `°`. `ORBS_GRID=80x60` for the horizontal bar, which the 80×22 default
        never reaches. `attend laboratory` alone for the cold wisp; `meditate
        597` for the guttering ember. `ORBS_FLARE=1` against `ORBS_FLARE=0` on a
        half-burned bar shows the spark shower — a dump cannot show the flare's
        colour, so `cargo run -p orbs-render --example screens` prints the heat
        ramp as digits beside the glyphs, which is where `flaring` and `settled`
        are actually distinguishable. **Colour still needs eyes on a window:**
        run the game, `attend laboratory`, `kindle charcoal`, then `F2` through
        all four tubes and `F3` to off
  9. ✅ **The mortar breaks things down** — it reduces rather than fills: four
        shades are four states of one substance, the block eaten from below, no
        tool drawn. The fall is slower than the clock, `Craft` lives in the sim,
        and `Painter::creeping` draws between ticks from the same `Time<Fixed>`
        the sim steps on (§19)
        **See it:** ✅ `ORBS_DUMP="attend laboratory; grind sage; meditate 3"
        ORBS_FIRE_PHASE=0.33 cargo run -p orbs` — step the phase by two ticks
        (0.00 → 0.33 → 0.67) and the debris falls a cell each time. `meditate 12`
        for the finished bar, which holds full until `empty`.
        `ORBS_TICK=0.0/0.5/1.0` walks the creep **inside one tick**, which a dump
        otherwise cannot show at all.
        **The loaded bowl needs the long form**, `move sage to
        mortar_and_pestle`, since `grind` never leaves an instrument `charged`.
        `cargo run -p orbs-render --example screens` prints the whole stroke and
        the lifecycle side by side, and both orientations
  - **Decisions, not execution** — the outcome depends on what the player
        chooses, never on how fast they act (§5.1, §14); timing means windows at
        1 Hz, and a staged action resolves under `meditate` with no one present

      **Exit criterion:** ✅ the same recipe, in two different tower states, has
      two different right answers, readable from the records before you act
      **See it:** ✅ `grimoire clarity` lists both routes to the draught before an
      instrument is committed to either, and
      `the_same_goal_has_two_right_answers_depending_on_what_the_laboratory_holds`
      runs both to the same end — one on fresh sage, one on the husks a grind
      left behind
- [x] ✅ **The grimoire, the spell editor, and the script engine** — three items
      that were one (§19): `grimoire` a place and the manual `recall`, a
      nameless root holding `/tower` and `/grimoire`, `Held` text, the editor
      frontend-side, `Submissions::Wrote` per save, and the runner

      **See it:** ✅ `ORBS_DUMP="attend laboratory; scribe brewing"`
      `ORBS_EDIT="edit\nkindle charcoal\ngrind the sage\nempty mortar_and_pestle\n<esc>\nquit"`
      `ORBS_THEN="invoke brewing; meditate 40"` — loose phrasing goes into the
      editor and the spell brews ground-sage while the player stands still.
      `peruse` reads it back **as typed**; the orb's reading of it is `interpret`
      in the editor (§19 — the file stopped being rewritten)
- [x] ✅ **Domain-scoped spells, watchable events, and the first control
      structures** (§19) — the domain as data, `FieldName::At` on every
      completion, `wait for`, `repeat`, `end`, and a malformed spell that runs
      **See it:** ✅ write `repeat 2 / kindle charcoal / grind the sage / wait for
      the mortar / empty mortar_and_pestle / end` in the editor, `peruse` it back
      exactly as typed (the buffer's own indentation and all), `interpret` it to
      see the commands read, then `invoke` it and watch the loop run twice.
      Also: `stop <spell>`, and `if`/`else`, where a question it cannot read
      answers no and says so
- [x] ✅ **A spell edited while it runs** (§19) — `SCRIPT_BUDGET` is 1, so a
      shorter spell is a faster one; the buffer saves itself; a save swaps the
      program between steps, keeping the position; the gutter and the title
      both mark the line the orb is on

      **See it:** ✅ `ORBS_DUMP="attend laboratory; scribe brewing"`
      `ORBS_EDIT="edit\nrepeat\nkindle charcoal\ngrind the sage\nempty mortar_and_pestle\nend\n<esc>\nquit"`
      `ORBS_THEN="invoke brewing; meditate 7; scribe brewing"` — the editor
      reopens on the spell mid-flight with `4»` in the gutter and *(the orb is on
      line 4)* on the title. In the running game: `invoke` it, `scribe` it, type,
      stop typing, and watch the next pass take the new line without a keystroke
      asking it to. The debounce is a frontend clock a dump cannot run; the
      in-`App` tests cover it
- [x] ✅ **The editor indents blocks as you type** (§19) — one rule,
      `parser::indent_around`; `Enter` mid-line indents nothing, so `Enter` and
      `Backspace` stay each other's inverse

      **See it:** ✅ `ORBS_DUMP="attend laboratory; scribe nested"`
      `ORBS_EDIT="edit\nrepeat 2\nkindle charcoal\nif the mortar is idle\ngrind the sage\nelse\nempty mortar_and_pestle\nend\nend\n<esc>\nquit"`
      `ORBS_THEN="peruse nested.spell"` — not one space typed, and the file the
      orb writes back matches the buffer line for line
- [x] ✅ **An `if` names places the way every other line does** (§19) — its tail
      is resolved like any line, and a place the tower lacks is *Referent
      missing*, not false

      **See it:** ✅ `ORBS_DUMP="attend laboratory; scribe probe"`
      `ORBS_EDIT="edit\nrepeat 10\nif mortar is empty\ngrind sage\nelse\nempty mortar_and_pestle\nend\nend\n<esc>\nquit"`
      `ORBS_THEN="peruse probe.spell; invoke probe; meditate 40"` — the loop
      alternates grinding and emptying instead of only emptying. The file reads
      back as typed; `interpret` is where the resolved name is now shown (§19)
- [x] ✅ **The orb no longer writes down a shorter command than it heard**
      (§19) — the guard asks about reagents, not words; superseded since by a
      file that is never rewritten

      **See it:** ✅ `ORBS_DUMP="attend laboratory; grind sage; meditate 20; scribe keep"`
      `ORBS_EDIT="edit\ngrind sage\n<esc>\nquit"`
      `ORBS_THEN="peruse keep.spell"` — the sage is spent before the spell is
      written, and `grind sage` still reads back whole (flagged, not truncated)
- [x] ✅ **Saving says nothing, and a burning athanor is not idle** (§19) — an
      autosave speaks no line, and `is idle` asks the panel's own state

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe brewing"`
      `ORBS_EDIT="edit\nkindle charcoal\ngrind the sage\nempty mortar_and_pestle\n<esc>\nquit"`
      `ORBS_THEN="invoke brewing; meditate 40"` — one line for the `scribe` and
      nothing at all for the save. ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory;
      scribe tending"`
      `ORBS_EDIT="edit\nif the athanor is working\nstop the athanor\nelse\nkindle charcoal\nend\n<esc>\nquit"`
      `ORBS_THEN="kindle charcoal; invoke tending; meditate 4"` — the panel reads
      `athanor banked`, so the spell took the `if` and not the `else`
- [x] ✅ **`debug_spawn`, the alembic's bubbles, and a review of both** (§19) —
      a debug-build word matched before the parser, never a `Verb`; bubbles
      above the alembic's face where the bar runs upward; and four bugs a review
      found, each reproduced first. No box, so no version

      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=80x40 ORBS_FIRE_PHASE=0.4
      ORBS_DUMP="attend laboratory; kindle charcoal; debug_spawn
      clarified-draught; distil clarified-draught; meditate 24"` — `°` above the
      face in the `al` column, and the level far enough down to watch it climb.
      ✅ `cargo run -p orbs-render --example screens` for eight rises side by
      side. ✅ `cargo test --release -p orbs-sim --test debug_spawn`
- [x] **The file is the player's, and `if` learned to say `and`** — the class of
      rewriter bugs closed by never rewriting (§8, §19). Reading moves to cast,
      through `spell::compile`; `not` > `and` > `or` with `either … or …` as a
      bracket; a question reads every token or none; a spell resolves at
      `SPELL_SIMILARITY` 850 with a margin of 50; an unanswerable question runs
      neither half; `interpret` shows the orb's reading

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe keep"`
      `ORBS_EDIT="edit\ngrind the sage\nif the mortar is idle and the dispensary has sage\nempty the mortar\nend\n<esc>\nquit"`
      `ORBS_THEN="peruse keep.spell"` — what goes in comes out, byte for byte.
      ✅ `ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend laboratory; scribe check"`
      `ORBS_EDIT="edit\nmake a potion of clarity\nif the mortr is bare\nsurvey\nend\nxyzzy plugh\n<esc>\ninterpret"`
      — `recall clarity` where the loose phrasing was, and `2 the orb cannot
      read` on the status row before that. ✅ the same without the trailing
      `interpret` shows the count while you write. ✅ a spell's own records are in
      the log rather than the pane, so
      `ORBS_THEN="invoke broken; meditate 20; sift broken orb.log"` is how the
      runner's report is read back
- [x] **Concentration, experience, and `bind`** — the automation pool, counted in
      bound spells and starting at 0, replacing §11.5's Attention (§19).
      **Concentration 1 is the game's turn**: until it is bought the tower is
      worked by hand. Experience is earned by runs, weighted by instrument, and
      never spent; 16 is one clarity walked end to end. An invocation ends when
      you leave its domain, and a bound spell stands, silently

      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; kindle charcoal; grind sage; meditate 9; empty mortar_and_pestle; digest ground-sage; meditate 14; grind rock-salt; meditate 9; empty mortar_and_pestle; mix sage-tincture with ground-salt; meditate 12; distil clarified-draught; meditate 60; status"`
      — `+8` on the alembic's yield line, then *"the orb can hold a spell now"*,
      then `experience 16` / `concentration 1`.
      ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; invoke first_light; attend archive; meditate 6"`
      — *"first_light.spell needed you there. it stops"*.
      ✅ and the pair, which is the whole point:
      `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; kindle charcoal; debug_spawn clarified-draught 2; distil clarified-draught; meditate 60; empty alembic; distil clarified-draught; meditate 60; scribe tending"`
      `ORBS_EDIT="edit\ngrind sage\nempty mortar_and_pestle\n<esc>\nquit"`
      `ORBS_THEN="bind tending; attend archive; meditate 40; status"` — experience
      climbs 16 → 20 while the player stands in the archive, nothing says *needed
      you there*, and the sidebar reads `held 1 of 1`. ✅ that row is **absent**
      before the turn: `ORBS_DUMP=1 ORBS_GRID=100x36` has no `held` line
- [x] **`weave` — the Ley Line, Mastery, and a surface for progression** (§19).
      Two tracks: the Ley Line, where passing a step is the grant, and Mastery,
      where a tier opens and gives one node. No points. The bar is a fixed
      hundred with both tracks under it, so a node's position is its cost;
      rightward is progress, downward a choice. The panel says `unlocked` and
      `active` separately, the aim is `«○»` and Bright, and the way into a track
      is a word

      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave"` — the bar
      reads `0 of 100`, the ley line draws `─────[·]───` with `16` under its one
      station, and both mastery tiers are `[·]`.
      ✅ after one clarity the station fills to `[•]` and the bar reads
      `16 of 100` — the long brew line from the item above with `weave` on the
      end. The bar's scale does **not** move; the fill does.
      ✅ a tier **opening**, which is the closest v1 gets to the mechanic:
      `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; kindle charcoal; debug_spawn clarified-draught 3; distil clarified-draught; meditate 60; empty alembic; distil clarified-draught; meditate 60; empty alembic; distil clarified-draught; meditate 60; weave"`
      — the tier at 24 becomes `[○]` and the details panel reads `unlocked` /
      `inactive`, which is the pair a single word could not have carried.
      ✅ aim and take, on the same line plus
      `ORBS_WEAVE="mastery\n<down>\ntake"` — `«○»` moves to the sibling below and
      the refusal names `tbi_b`, so typing kept the aim. `<right>` walks to *its
      own* successor at 40, which is the other axis.
      ✅ and the arrows refusing to move before a word:
      `ORBS_WEAVE="<down>\n<right>"` leaves the aim unplaced and the details
      panel reads *"say ley or mastery"*.
      ✅ `ORBS_WEAVE="ley\ntake"` on an earned step says *"concentration is yours
      already"* — not *"nothing behind it yet"*, which is what it said about the
      game's one real grant while the panel beside it read `active`.
      ✅ both widths: `ORBS_GRID=80x22` (one wide pane) and `ORBS_GRID=100x28`
      (the narrowest tiled pane).
      ✅ `cargo run -p orbs-render --example screens` draws it at 48×18 — the
      narrowest *and* shortest the painter accepts — and prints the linear
      stream, where every state appears as a word (`16: taken`, `24: open`)
- [x] **The prompt becomes a command line** — caret editing, history filtered by
      what is typed, `parser::complete`, Tab, and an inline suggestion preferring
      history (§19)
      **See it:** ✅ `ORBS_LINE="wield mo" ORBS_DUMP="attend laboratory"` shows
      the partial line with `rtar_and_pestle` ghosted after the caret and a
      `Hint` in the linearised stream. In the running game: type, walk into the
      middle of a command and edit it, ↑ through the wields, Tab a name
- [x] **Ten-angle review of the brewing + command-line work, applied** — eleven
      live bugs, four reachable in a first session, and three asserted by green
      tests (§19)
      **See it:** ✅ `ORBS_DUMP="attend laboratory; grind sage; meditate 12;
      move ground-sage to balneum_mariae"` — the linear stream now says `laboratory:
      mortar_and_pestle fouled, balneum_mariae charged` where it said nothing at
      all, and
      `cargo run -p orbs-sim --example session` runs the whole §10.1 pipeline and
      reads nine lines back out of `laboratory.log`
- [x] **Readable recipes, Tab cycling, and a scrollable transcript** — a recipe
      in doing order that stops at what is stocked, Tab cycling, PageUp/PageDown,
      and `RecordView` wrapping where it had clipped
      **See it:** ✅ `ORBS_DUMP="attend laboratory; grimoire clarity"` — five
      numbered steps, byproducts, heat markers, alternatives, nothing clipped at
      the 80×22 floor. ✅ `ORBS_SCROLL=14 ORBS_DUMP="…"` shows the transcript held
      back with `PgDn newest` in the border. In the running game: type `wield `
      and press Tab three times
- [x] **Per-instrument verbs, scoped to their domain** — `grind`, `digest`,
      `mix`, `distil`, `kindle`: the `move` and the `wield` in one, each a word
      only where its instrument stands (§7, §19). And `empty <tool>`, the
      instant counterpart of `purge`
      **See it:** ✅ `ORBS_DUMP="attend laboratory; grind sage; meditate 12"` —
      three commands where four were needed, same result. ✅ the same verbs in
      `attend archive` say where they are not
- [x] ✅ **The balneum mariae draws, and the fire stops changing colour** (§19) —
      the bath's level is the reading, all its motion in colour on a solid `█`;
      the fire is one orange ramp on every tube

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal;
      grind sage; meditate 9; empty mortar_and_pestle; digest ground-sage;
      meditate 6"` — the vessel filling; step the last `meditate`. ✅ the same
      with `move ground-sage to balneum_mariae` for a charged one, which is a
      shallow layer rather than nothing. ✅ `cargo run -p orbs-render --example
      screens` for the roil as `a`/`b`/`c` — a dump cannot show it, because the
      glyph never changes. ✅ `cargo run -p orbs` and F7 through the themes: the
      hearth burns the same orange on all four
- [x] ✅ **Materials carry a colour, and an instrument's bar draws in it** (§19) —
      `materials.toml` against `Tint`'s eight families, an unknown name failing
      the load; a `(Rect, Tint)` side-table on the `Frame`, never over an accent
      or the fire

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; move sage to
      mortar_and_pestle"` prints `green 2×17` under the linear stream; the same
      run ending `move ground-sage to dispensary` prints `brown`. A dump can
      show the region but not the colour — ✅ `cargo run -p orbs` for that
- [x] ✅ **The flask_and_rod mixes, and a finished bath keeps turning over**
      (§19) — two bands shrinking together into a mixture the average of their
      colours (`Wash`); a finished bath still turns over, a charged one is flat

      **See it:** ✅ the eleven-command line in SEEING-IT.md prints `green`, `bone`
      and `green+bone` as three regions; step its last `meditate` and the
      mixture grows as both bands shrink. ✅ `ORBS_DUMP="...; digest
      ground-sage; meditate 20"` for a bath that has finished and is still
      moving — a dump cannot show that, so ✅ `cargo run -p orbs` for the rate
- [→] **Remaining sabotage surfaces — split, and moved.** Adversarial aberrations
      are siege-only (§5.1), so script text and trigger clocks went to Phase 8;
      the world surface went to Phase 2 with scrying
      **See it:** the log surface already works —
      ✅ `ORBS_DUMP="attend laboratory; grind sage; meditate 25; grind rock-salt;
      meditate 25; verify laboratory.log"` reads `state: tampered`
- [x] **The archive is a maze, and the world holds the search** — `research`
      resolves the stacks, `follow` threads them, the way out gives up a
      fragment, four make a scroll. The cells mark themselves, so Trémaux is
      automatable with no grammar change; `NounKind::Sense` lets a condition
      name them at cast (§19)

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; research; survey north; follow east"`
      — the stacks resolve, `survey north` shows a reading, the reading moves.
      ✅ `cargo test -p orbs-sim --test solver` — a Trémaux solver's every
      condition survives the cast, the readings resolve with no maze open, and a
      misspelled `walkd` is still refused.
      ⬜ a scroll that does something — the item below
- [x] **The map that fills in** — `orbs-render/src/maze.rs`: walls where the
      reading has stood, `▒` walked, `░` finished, `☼` the head, `Ω` the way out.
      It takes columns, never rows, and refuses rather than truncates

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; research"` — one mark
      alone in the dark.
      ✅ `ORBS_BOOT=0 ORBS_GRID=160x45 ORBS_DUMP="attend archive; research; follow
      east; follow east"` — the fog opening as it goes, at the narrowest grid that
      draws it.
      ✅ `cargo run -p orbs-render --example screens` — three states through the
      real painter rather than a replica, because this picture lives in the render
      crate and can be called rather than imitated.
      ✅ **watching a spell solve it**, which is what the map is for — write the
      four-way solver from `tests/solver.rs`, then
      `ORBS_THEN="invoke threading; meditate 300"`. `▒`, `░` where it backtracked,
      `☼` mid-flight, fog still ahead of it.
- [x] **A solver that actually finishes one** — *survives the cast* had stood in
      for *reaches the exit*. A flat ladder of `if`s oscillates; `else` makes it
      one move per lap
      **See it:** ✅ `cargo test -p orbs-sim --test solver` — twelve seeds, each
      swept to a shard, worst 5699 ticks against a pinned budget of 6500.
- [x] **`wander`** — the arrow keys walking the stacks. Walking takes the pane;
      a spell solving one keeps the map inline. `Sim::walk` is a third entry
      point that consumes no tick, and `Submission::Walked` records when, so
      replay holds (§19)

      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=160x45 ORBS_DUMP="attend archive;
      research; wander" ORBS_WALK="<right>\n<right>\n<down>\n<down>\n<left>"` — the
      pane is the maze, the border reads `walking`, and the footer counts.
      ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; wander"` — refused, and it names
      `research` as the way in.
      ✅ `cargo test -p orbs shell::wandering` — a press moves the reading with no
      tick stepped anywhere in the test, and the keys come back when a spell
      closes the maze.
      ✅ `cargo test -p orbs-sim --lib session` — a hand-walked maze replays to the
      same cell, and walking costs no world time.
- [x] **A wall is a square, so a step is one character** — three defects found
      by looking, one geometry: the grid became squares, a wall one of them, and
      the corridor between two cells somewhere you stand
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=160x45 ORBS_DUMP="attend archive;
      research; wander" ORBS_WALK="<right>"` — the head moves **one** character.
      ✅ `cargo run -p orbs-render --example screens` — a maze that reads as a
      maze.
      ✅ `cargo test -p orbs-render maze` and `cargo test -p orbs-sim --lib maze`
- [x] **16×16, a denser carve, and `back`** — 176 cells carved by randomised
      Prim's, refused below `ORBS_GRID=160x45`. It showed the four-tier ladder was
      never Trémaux, and `back`, the way last come from, fixes it (§19)
      **See it:** ✅ `cargo test -p orbs-sim --test solver` — twelve seeds swept,
      worst 5699 ticks against a pinned 6500. (It said 5123 here for two
      versions; pinning the figure as an equality is what found it.)
      ✅ `ORBS_BOOT=0 ORBS_GRID=160x45 ORBS_DUMP="attend archive; research"` — a
      33×23 block, dark but for one mark.
      ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; research; wander"
      ORBS_WALK="<right>\n<right>\n<right>"` at the 80×22 floor — a window on
      the maze rather than no map at all.
- [x] **One generic fragment, not four named ones** — four named shards drawn at
      random were coupon-collector attrition with no decision in it. One
      `fragment`, four of it, and no roll
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; research"` then solve
      and `survey cabinet` — `fragment = 1` on the archive's shelf.
      ✅ `cargo test -p orbs-sim --test solver`
- [x] **A recipe can want more than one of something** — `Recipe::count`, matched
      as at least and spent exactly; three copies of "what is in this
      instrument" became `tower::holdings`
      **See it:** ✅ `cargo test -p orbs-sim --lib recipe`
      ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; move sage to
      mortar_and_pestle; move sage to mortar_and_pestle; wield
      mortar_and_pestle"` — still fires, and leaves one sage behind.
- [x] **The manual, part one: `help` lists what works here** — moved from Phase
      14. Bare `help` was ambiguous; `recall` takes an optional topic now, and
      `execute::offered` is the one listing the boot report shares

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; help"` — five
      headings at the 80×22 floor, `grind` among them.
      ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; help"` — no `grind`, because it
      does not resolve there.
      ✅ `cargo test -p orbs-sim --lib recall`
- [x] **The manual, part two: `recall <verb>` is a page** — synopsis, gloss,
      examples, synonyms off `SYNONYMS`, see-also. `NounKind::Command`, reachable
      only from `Subject`, so verb words leak nowhere. Pages are readable
      anywhere
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; recall grind"` — the
      mortar's page, read from a room the mortar is not in.
      ✅ `cargo test -p orbs-sim --test naming` — the three leaks, driven.
- [x] **The manual, part three: every verb has a page** — three lints hold it:
      every verb has a page, a synopsis names every required slot, and no see-also
      names a word that is not there. A page admits what is not built
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; recall distil"`
      ✅ `cargo test -p orbs-sim --lib recall` — the three lints
      ✅ every verb: `for v in attend survey peruse ...; do ORBS_DUMP="recall $v"`
- [x] **The manual, part four: every *item* has a page** — what it is first,
      then `using_<name>`, then the route. Two more lints, and every material a
      `Topic`
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="recall clarity"` — what it is, that
      nothing drinks it yet, then the five steps that make one.
      ✅ `ORBS_BOOT=0 ORBS_DUMP="recall gleaning-scroll"` — and how to spend it.
      ✅ `cargo test -p orbs-sim --lib recall` — five lints now
- [x] **A fixed 4:3 picture — the grid stops following the window** — 120×45,
      always; the window decides only how big a cell is, through
      `ScalingMode::AutoMin`. 720 divides the common display heights. `F4` no
      longer changes text size, so the game owes a font-scale setting (§19)
      **See it:** ✅ `cargo run -p orbs`, then drag the window wide, tall and
      square: the picture stays 4:3 and centred, the bars grow on one axis only,
      **no text reflows**, and the log prints one `scale` line per resize.
      ✅ the **tower rail's foot** shows `scale` moving while `grid` holds at
      120×45 — it was the telemetry pane's row until Phase 2 replaced that pane.
      **The `F4` half of this line is withdrawn**: `PANES` is 1, so the key is
      inert until multiplexing returns the second pane (§19)
      ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; survey"` — 120×45 is the
      dump's default now, because a dump that is not the game's screen is an
      instrument reading the wrong thing.
      ✅ `ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_DUMP="attend laboratory; survey"` —
      the authoring floor, which is all `ORBS_GRID` is for now.
      ✅ `ORBS_GRID=40x10 ORBS_DUMP=1 cargo run -p orbs` — the too-small screen
      by grid; drag below 960×720 for the same screen by scale.
      ✅ `cargo run -p orbs-render --example screens` — the scale table that
      replaced §9's tier table, and the worst case at the smallest legible glyph.
- [x] **Scrolls that do something** — four steps, ordered by what can be shown
      without the next thing existing (§19).
  1. ✅ **A scroll is a kind, the lectern rolls one, and `wield` spends it on a
        gleaning errand** — `wield` learned `NounKind::Workable` rather than a
        23rd word; what four fragments become is drawn; the errand is published
        on the stacks, so one solver handles both; a gleaning maze has no exit
        **See it:** ✅ `ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend archive;
        research; debug_spawn gleaning-scroll 4; wield gleaning-scroll; wander"`
        — four `♦` scattered through the maze (five, and one scroll, before
        `0.18.6`) and
        **no `Ω`**.
        ✅ the same without `wander` plus `survey stacks` — `reading: gleaning`
        on the stacks, which is the word a spell asks for.
        ✅ `cargo test -p orbs-sim --test gleaning` — nine claims driven through a
        real `Sim`, including one solver taking a different branch under each
        errand, and the lectern still assembling with an errand open.
        ✅ `cargo run -p orbs-render --example screens` — the fourth stacks
        screen, through the real painter.
  2. ✅ **The arsenal — `/tower/arsenal`, and the one room reachable from every
        other** — finished work only, asked of the kind and never the name,
        searched last; `tower::keep` owns the rule. `NounKind::Portable` lets a
        potion be picked up at all. The archive became three fixtures doing one
        thing each — stacks, cabinet, lectern — and `tower::home` derives where
        every item belongs (§19)
        **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory;
        kindle charcoal; debug_spawn clarified-draught; distil
        clarified-draught; meditate 60; empty alembic; move clarity to arsenal;
        attend archive; survey arsenal"` — a potion brewed in one room, carried
        to a second and listed from a third. The first time anything in the tower
        has crossed a domain.
        ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; move sage to arsenal"` — the
        door, refusing in voice and naming where a reagent does belong.
        ✅ the long line with `attend archive; verify clarity; peruse arsenal.log;
        purge clarity; survey arsenal` on the end — every verb that can now name
        a kept thing reaches it, the log has the move in it, and `purge` acts
        rather than resolving and doing nothing.
        ✅ `cargo test -p orbs-sim --test arsenal` — eight claims, one per way
        this could have been half-built.
  3. ✅ **Quickening — a window on the laboratory, not a shot at one run** — an
        interval read at `begin`, so a run started inside it stays short; a run in
        flight is halved from now
        **See it:** ✅ an 8-tick grind, twice —
        `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; grind sage; meditate 4"`
        yields nothing, and the same line with
        `debug_spawn quickening-scroll 4; wield quickening-scroll` in front of it
        yields ground-sage (one scroll did, before `0.18.6`).
        ✅ `cargo test -p orbs-sim --test gleaning` — the short run, the run that
        stays short when the window closes, the in-flight halving and its point,
        and the two clocks agreeing.
  4. ✅ **The verdant scroll, and two potions off ported herbs** — `mugwort`,
        `valerian` and `amber` from `../court_wizard`, one herb per scroll. What
        is unlockable is derived, never listed; names were measured, not chosen
        **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_spawn
        verdant-scroll 4; wield verdant-scroll; wield verdant-scroll; wield
        verdant-scroll; wield verdant-scroll; survey dispensary"` — three
        reagents become six, one at a time, and the fourth scroll says the shelf
        holds every herb it knows.
        ✅ `recall insight` and `recall stillness` — five steps each, and
        `recall mugwort` says what a mugwort is.
        ✅ the whole route walked in `tests/gleaning.rs`, because a route that
        reads well and cannot be walked is a table rather than content — which is
        exactly what the lectern's `dust` recipe turned out to be.
- [x] **A spell counts, and the archive's solver is a tester's tool** — the
      language already automated both errands. `has` takes a count, at least and
      never exactly, where the number used to be swallowed. `debug_spell` writes a
      known-good ladder in debug builds only, recording the write (§19)
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend archive;
      scribe check" ORBS_EDIT="edit\nif the cabinet has 4 fragment\nwield
      lectern\nend\n<esc>\ninterpret"` — the count survives, where it used to
      vanish.
      ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; debug_spawn fragment 2;
      debug_spell assembling" ORBS_THEN="invoke assembling; meditate 60; peruse
      archive.log"` — the guard holds and the log stays quiet; 4 and 5 both
      assemble.
      ✅ `ORBS_SEED=11 ORBS_BOOT=0 ORBS_GRID=160x45 ORBS_DUMP="attend archive;
      research; debug_spell threading" ORBS_THEN="invoke threading; meditate
      3600; meditate 3600; survey cabinet"` — seed 11 is the one the old
      documented ladder fails.
      ✅ the same line with a gleaning supply opened first — four fragments
      since `0.18.6`, same file.
      ✅ `cargo test -p orbs-sim --test gleaning` — one ladder over both errands,
      four seeds; `--test questions` for the counted grammar; `--test debug_spell`
      and `cargo test --release -p orbs-sim --test debug_spell` for the door.
- [x] **The spell language, audited against seven others** — most of it was
      right (§19). One word was missing: `repeat until`, where a question it
      cannot answer stops the loop. `has` gained comparators, the maze's buckets
      became `marks`, and `recall` teaches the control words
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend archive;
      scribe check" ORBS_EDIT="edit\nrepeat until the stacks is idle\nfollow
      east\nend\n<esc>\nquit" ORBS_THEN="invoke check; meditate 8; peruse
      orb.log"` — zero passes with the stacks shut, and it loops with one open.
      ✅ the same with `until the mortar_and_pestle is working` around a `grind
      sage` in the laboratory — one pass, then *"is finished"*.
      ✅ `ORBS_DUMP="recall repeat; recall until; recall if"` — the manual the
      language never had.
      ✅ `ORBS_SEED=3 ORBS_BOOT=0 ORBS_GRID=160x45 ORBS_DUMP="attend archive;
      research; follow south; follow south; follow north; survey south"` — reads
      `back  marks = 1`, where it read `back walked`.
      ✅ `interpret` on `2 or more` and `1 or fewer`: the first collapses to the
      bare count, the second keeps its words.
      ✅ `cargo test -p orbs-sim --test solver` — the ladder is tier-for-tier
      what it was, pinned to the tick.
- [x] **Every spelling of a comparison, and a manual for the language** —
      seventeen spellings, symbols included, with `exactly` as a third bound and
      words written back. A page per control word and reading, and `recall
      scripting`. A colour lint that skipped a missing tint now fails
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend archive;
      scribe check" ORBS_EDIT="edit\nif east has at least 2 marks\nfollow
      east\nelse\nif east has <= 1 marks and not east has wall\nfollow
      east\nelse\nif east has exactly 0 marks\nfollow
      east\nend\nend\nend\n<esc>\ninterpret"` — `2 marks`, `1 or fewer marks`,
      `exactly 0 marks`. The first used to read `marks`.
      ✅ `ORBS_DUMP="attend archive; recall scripting"` against the same in the
      laboratory — the words and questions are identical, the last section is not.
      ✅ `ORBS_DUMP="attend archive; recall marks; recall back"` — the readings
      say what they mean, including the wall guard `marks` needs.
      ✅ delete a tincture's tint and `cargo test -p orbs-sim --lib material`
      names the product *and* the input, where it used to pass.
### Where Phase 1's nine open items went

Each moved for a stated reason. Only two went to `Standing`, whose rule is that
an item belongs there when it cannot close.

| Item | To | Why |
|---|---|---|
| Third domain (scrying) | **2** | It *is* the domain, and §10 says build it early |
| Sabotage surfaces — the **world/log** half | **2** | §10: log-parsing is how sabotage is found |
| Sabotage surfaces — **script text, trigger clocks** | **8** | §5.1 makes adversarial aberrations **siege-only** — *"the enemy never touches scripts, schedules, or logs in the calm layer"*. With no siege they have no producer, so the item could never close and would hold a phase open for ever |
| `orbs-balance` CLI | **2** | §16 names it the mitigation for the High-severity *"economy is wrong or untuneable"*. Every duration in the game is still a placeholder and five domain phases would author more on top of unswept numbers |
| Naming pass (~35 verbs) | **3** | §18 lists it **blocking**, and Standing *"blocks no phase"*. It goes ahead of Spellcraft, which is where new verbs arrive fastest |
| Hidden-directory authoring | **3** | Fragments are what the grimoire spends |
| Minimal apprenticeship + playtesting | **10** | Needs people; carries Phase 1's second exit clause with it |
| Scrappy `orbs-tui` | **Standing** | Genuinely accretes — a little more true with each frontend change |
| Trace tuning | **Standing** | Accrues with content; no state in which it is *done* |

**Content data format is ticked rather than moved.** Recipe reload is excluded:
a recipe is a decision, and swapping one breaks replay until content is versioned
into the submission log, which arrives with remote hosts.
- [x] **Content data format** — hot-reloadable TOML/RON, prose never in Rust
      literals. Recipe reload deliberately excluded; see above.
      **See it:** ✅ edit a line of `prose.toml` with the game running

---

## The domain phases — the rest of the tower

> **This was *"Phases 2–7"*.** The domains are phases 2, 3, 4, 5 and 9 — names,
> not an order. All seven are raised.

**§10's forms are a table, not a design.** Brewing is the worked example the rest
are cut from (§10.1), and every domain is measured against six rows:

| | In brewing |
|---|---|
| A shared, **depleting** resource | **lit time** — the athanor burns per lit tick whether or not anything is mounted |
| **A window at 1 Hz, never a reflex** | light → digest → damp → combine → relight → distil |
| Cost is the resource, **never progress** | wasted fuel, so §11.5's *"never ruinous, only slower"* holds |
| The decision is **choice given readable state** | never how fast or precisely the player acts |
| The world publishes readings, so a spell holds a **rule, not a memory** | §8.1 forbids automation driven by hidden state |
| The best play **falls out of the durations** | *"That was not designed."* Damping was found when the fire died mid-test |

**The trap is inventing a scarcity the game contradicts.** Fuel stock was never
scarce, lit time is; and looking is priced by §8.1's cooldown, never by a
resource. `CAPACITY = 1` is tower-wide, so concurrency is already shared.

---

## Phase 2 — Scrying ✅

**Exit:** a player breaks a far wizard's ward by deduction, and then a spell does
it unattended. **Met** — `probe`/`dial` at the prompt, `debug_spell breaking`
bound in the lens while the player brews.

**A code-breaker**: a ward of four sigils from six, repeats allowed, 1296 codes.
The noisy logs survive as the yield — a broken seal spills the far wizard's
laboratory into `lens.log`. Two channels onto one ward: the counts for a person,
the deltas for a spell. The ward was rebuilt with repeats at `0.3.22`, which
deleted the exchange and the three props it needed (§19, *"The lens is
Mastermind now"*). The world sabotage surface ships here too.

- [x] **The tower rail** — §9's sidebar, finally reachable, reshaped into a
      vertical column of one box per domain. Replaces the telemetry pane;
      `PANES` is 1 and the session body goes from 58 columns to 102.
      **See it:**
      ```bash
      ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; grind sage" cargo run -p orbs
      ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_DUMP="attend laboratory; grind sage" cargo run -p orbs
      ```
      Seven boxes, four dark. A fault in one room is visible from another:
      ```bash
      ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe broken" \
        ORBS_EDIT="edit\nrepeat 5\nwield zzz\nend\n<esc>\nquit" \
        ORBS_THEN="invoke broken; meditate 20; attend archive" cargo run -p orbs
      ```
      → `laboratory ‼`, cleared by going to look. Each box is ruled off from the
      next (§19). A box using all four of its rows:
      ```bash
      ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; \
        debug_spawn clarified-draught 2; distil clarified-draught; meditate 60; \
        empty alembic; distil clarified-draught; meditate 60; scribe tending" \
      ORBS_EDIT="edit\ngrind sage\nempty mortar_and_pestle\n<esc>\nquit" \
      ORBS_THEN="bind tending; attend archive; meditate 6" cargo run -p orbs
      ```
      ```text
      laboratory      ← name, state, what is busy, what is automating it, then the rule
        burning
        at 138t
        ►tending      ← `►`, and no `.spell`: both were wrong and are in §19
      ──────────────
      ```
- [x] **The `lens/` domain, the ward, `probe` and `dial`.** Four sigils of six,
      **repeats allowed**, 1296 codes. The orb keeps **no candidate set and
      deduces nothing** — see §19, which records *two* designs breaking that
      rule, the second quietly.
      **See it:**
      ```bash
      ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; \
        dial second borax; probe; survey prism; survey second" cargo run -p orbs
      ORBS_BOOT=0 ORBS_DUMP="recall probe; recall dial" cargo run -p orbs
      cargo test -p orbs-sim --test ward
      ```
      No `meditate`: a press is instant and takes no slot (§19). `survey prism`
      reads the counts and the deltas; `survey second` reads only what is in it.
      ```bash
      # A dial moves one socket and only one, and a sigil may sit in two.
      ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; dial first alum; \
        survey first; survey second" cargo run -p orbs
      ```
- [x] **The board** — the ward as a sheet, beside the transcript. Not gated on a
      word, so a bound solver is watchable; columns never rows; refuses rather
      than clipping, because a row missing its pegs says a press answered
      nothing.
      **See it:**
      ```bash
      ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; \
        dial second borax; probe; dial third quartz; probe" cargo run -p orbs
      cargo run -p orbs-render --example screens   # the sheet, no sim and no GPU
      ```
      ```text
      ┌ ward ────┐
      │☼○♂♀  ○○  │   ← each row a press: the figure, then its pegs
      │☼♦♂♀  •○○ │
      │♦♦♂♀  •○○○│   ← a sigil twice, and the full four pegs
      │──────────│
      │♦♦♂♀      │   ← the aperture: what the next press will send
      ```
      Six glyphs, not six colours (§14), spoken once as a summary.
- [x] **The spill, and recipes you do not know yet.** A broken seal writes a
      dozen lines of the far wizard's laboratory into `lens.log` — quiet, so the
      transcript gets one sentence — and rolls for a recipe on a chance that
      climbs each solve and resets when it lands. Three secrets ship, each one
      step over a byproduct, so a discovery gives something you were throwing
      away a second use.
      **See it:**
      ```bash
      ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; \
        debug_ward; probe; peruse lens.log" cargo run -p orbs
      ORBS_BOOT=0 ORBS_DUMP="recall mending; debug_learn; recall mending" cargo run -p orbs
      cargo test -p orbs-sim --test secrets
      ```
      ```text
       5 mix phlegm            ← somebody else's log, generated from real recipes
       7 grind mugwort
      10 distil dregs          ← a recipe you do not have. that is the hint
      ```
      `recall mending` falls through before the discovery and prints the route
      after it — makeable, nameable and readable on the same tick.
- [x] **A solver spell for the lens.** `debug_spell breaking` reads only which
      way `aligned` moved; one socket moves at a time, so a rise can only be that
      socket arriving.
      **See it:**
      ```bash
      ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; debug_ward; probe; \
        probe; debug_ward; probe; debug_spell breaking" \
        ORBS_THEN="bind breaking; meditate 3600; status" cargo run -p orbs

      # ...and one ward, watched: fifteen presses on this seed.
      ORBS_SEED=3 ORBS_BOOT=0 ORBS_GRID=200x45 \
        ORBS_DUMP="attend lens; debug_spell breaking" \
        ORBS_THEN="invoke breaking; meditate 200; peruse lens.log" cargo run -p orbs
      ```
      Four phases of three rungs (§19): bare `dial <socket>` tries something
      else, and a restore rung puts the sigil back when `aligned` falls. `bind`
      makes it the faucet. `cargo run -p orbs-balance -- run scrying` pins the
      rate.
- [x] **The world sabotage surface**, moved here from Phase 1 — a reagent is
      substituted, its name changed and its identity not, so a spell that named
      it stops and `verify` names what is wrong.
      **See it:**
      ```bash
      ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_swap; survey dispensary; \
        verify dispensary; verify laboratory" cargo run -p orbs
      cargo test -p orbs-sim --test tampering
      ```
      ```text
      charcoal- = ∞   rock-salt = ∞   sage = ∞     ← the odd one out, on screen
      verify dispensary tampered something here is not what it says: charcoal-
      verify laboratory sound                      ← one level, deliberately
      ```
      Endless base stock only, never fuel. A lie nobody catches settles back to
      the truth, or an unattended tower is terminal — a sweep showed it (§19).
      **See the ambient half**, which no dump can reach — it is an hour of world
      time, and `debug_swap` above is the tester's shortcut past it:
      ```bash
      cargo test -p orbs-sim --test tampering   # settles; never takes the fire
      cargo run -p orbs-balance -- sweep --ticks 7200
      ```
- [x] `orbs-balance` CLI sweeping §11.5's first-pass numbers, moved here from
      Phase 1 — policies, a per-tick rate against a pinned reference, and `--why`.
      Four findings on its first run, and the ambient swap's regression one item
      later with the suite green (§19)
      **See it:**
      ```bash
      cargo run -p orbs-balance -- sweep --ticks 7200 --why
      cargo run -p orbs-balance -- run clarity --ticks 1200 --why   # cost 34, all scours
      cargo test -p orbs-balance                                    # the anchor *and* the pins
      ```
      **`--ticks 7200`, not `--hours 1`**: at 3600 one badly-timed swap moves a
      rate past its band and the table flags the seed rather than the game.
      **`cost` is the column to read first** — every entry should be a scour the
      policy asked for. `scrying` takes no slot, so its rate is additive.
**Scarcity: none.** A press takes no slot, so a bound solver runs beside a full
brewing loop (§19); §8.1's cooldown still prices repeated looking.

---

## Phase 3 — Spellcraft

> **Where this stands.** The save format is closed (three boxes), and the
> language overhaul closed at `0.3.26` (seven boxes). **Spellcraft's exit is
> met.** Left, deliberately: the terse register (nothing needs it), the typed
> action at execution (an argument cannot be typed at cast, §19), the naming
> pass and hidden-directory authoring. `0.3.21`–`0.3.22` were Phase 2 and
> Standing work rebased in.

**Exit:** a spell factors into named parts and still reads as one file.

A spell is contained to a single `.spell` file (§19), so reuse is within a spell
and cross-file sharing is struck. **Composition of spells, not acquisition of
words**: earning language words stays a §19-priced deferral. The three save
boxes at the head are not Spellcraft; they are here because every later phase
adds state to the format (§19).

- [x] ✅ **The save document, headless.** `Sim::snapshot` reads a world out as
      plain TOML and `Sim::restored` puts one back; nodes by path, and a restore
      raises the tower first and then adopts it. Replay and play must write the
      same bytes. A resumed spell checks a fingerprint, the record tail travels,
      and a review found five defects (§19)
      **See it:** `cargo test -p orbs-sim --test persistence`, then read one:
      `cargo test -p orbs-sim --test persistence -- --ignored show_a_save
      --nocapture` — the maze is drawn as a maze, a node's path stands where an
      id would have been, and an instrument mid-run says which tick it lands on
- [x] ✅ **The save file, and the tools that must not see it.** `orbs-shell` owns
      the path, the atomic write and the `[away]` stamp; `ORBS_SAVE=off` keeps
      none. A dump neither loads nor saves unless asked, and a save that will not
      open is kept
      **See it:** leave mid-brew, come back, and the clock continues rather than
      restarting — `tick = 8` then `tick = 10`, with the first run's transcript
      still on screen. Then `printf 'garbage' > "$S"` and watch the session draw
      a new tower instead of falling over:
      ```bash
      S=/tmp/orbs-save.toml; rm -f "$S"
      ORBS_SAVE=$S ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; \
        grind sage; meditate 4" cargo run -p orbs
      ORBS_SAVE=$S ORBS_BOOT=0 ORBS_DUMP="survey mortar_and_pestle; status" \
        cargo run -p orbs
      cat "$S"
      ```
      ...and the one that matters most, because it is the hazard the step exists
      to close — a save deliberately left in the repository root, and the
      56-screen baseline byte-identical across it:
      ```bash
      ORBS_SAVE=orbs-save.toml ORBS_BOOT=0 ORBS_DUMP="attend archive; research" \
        cargo run -p orbs
      scripts/dumps.sh /tmp/b1 && scripts/dumps.sh /tmp/b2 && diff -r /tmp/b1 /tmp/b2
      ```
- [x] ✅ **Both frontends load, autosave and save on the way out.** A save
      outranks the seed and the environment. Every exit path saves, not only
      `quit`; autosave every 60 world ticks off the sim's clock; `Opened` tells
      no save from an unreadable one
      **See it** — the terminal build, twice, leaving by a different route each
      time, because a dump has no window and no clock:
      ```bash
      S=/tmp/tui.toml; rm -f "$S"
      tmux new-session -d -s orbs -x 120 -y 45 -e ORBS_SAVE=$S -e ORBS_BOOT=0 \
        -e ORBS_SEED=181 target/debug/orbs-tui   # a game's seed is its own since 0.15.5
      # attend laboratory; kindle charcoal; grind sage — then `quit`
      # reopen: the transcript and the lit athanor are both still there
      # leave by F10, and again by Ctrl-C: `tick` climbs 7 → 42 → 67
      ```
      ...and the voices, which a dump *can* reach:
      ```bash
      ORBS_SAVE=$S ORBS_BOOT=0 ORBS_DUMP="status" cargo run -p orbs
      #   the orb remembers. the tower is as you left it
      #   it was dark for 3 minutes. nothing turned while you were gone
      printf 'garbage' > "$S" && ORBS_SAVE=$S ORBS_BOOT=0 ORBS_DUMP="status" cargo run -p orbs
      #   the orb cannot make out what it wrote last. this tower is new
      ```

**The language overhaul.** The language could not hold a component — `threading`
was six tiers × four ways unrolled by hand. This supersedes the seven-language
audit's no-variables premise (§19).

- [x] ✅ **`else if` — a chained `if`, and one `end` closes the ladder** —
      `threading` went from 49 lines of `end`/`else` in 98 to 3 in 52. A
      desugaring, not a new `Kind`, so nothing downstream changed
      **See it** — three branches and one `end`, read back by `interpret`:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend archive; scribe ladder" \
        ORBS_EDIT="edit\nif north has exit\nfollow north\nelse if east has exit\nfollow east\nelse\nfollow west\nend\n<esc>\ninterpret" \
        cargo run -p orbs
      ```
      ...and the 52-line solver still walking out of three different mazes:
      ```bash
      for s in 3 11 17; do ORBS_SEED=$s ORBS_BOOT=0 \
        ORBS_DUMP="attend archive; research; debug_spell threading" \
        ORBS_THEN="invoke threading; meditate 3600; meditate 3600; survey cabinet" \
        cargo run -q -p orbs; done      # `fragment 1` each time
      ```
- [x] ✅ **A bound-spell policy in `orbs-balance` — the instrument, before the
      thing it measures.** `bound` is `grind`'s loop run by a spell, so the gap
      is what automation costs: 0.910 of the hand rate on every seed, and
      `tests/agrees.rs` pins the quotient. A waiting spell stamps `Role::Cost`
      too, so `--why` separates a wait from a refusal
      **See it:**
      ```bash
      cargo run -p orbs-balance -- run bound --ticks 7200 --why
      #   bound  0  7200  655  1  0.0910  656  641
      #   640  tending.spell waits: the mortar_and_pestle is working
      cargo run -p orbs-balance -- run grind --ticks 7200   # 0.1000, the same loop by hand
      ```
- [x] ✅ **A comparison can name another place instead of a number** — `north has
      fewer marks than east`, `more … than`, `as many … as`. Additive; both sides
      read the published `Sense` children through one `many_at`; strict against a
      place, inclusive against a number; a half-written comparative refuses
      **See it** — the same spell against three worlds, moving only when the
      cabinet really has more:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=110x34 \
        ORBS_DUMP="attend archive; debug_spawn fragment 4; debug_spawn fragment 1 lectern; scribe weighing" \
        ORBS_EDIT="edit\nif the cabinet has more fragment than the lectern\nmove fragment to lectern\nend\n<esc>\nquit" \
        ORBS_THEN="invoke weighing; meditate 6; survey lectern" cargo run -p orbs
      #   4 vs 1 -> lectern 2.   2 vs 2 -> lectern 2.   1 vs 3 -> lectern 3.
      ```
      ...and the manual, which teaches it beside the other question shapes:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend archive; recall scripting" cargo run -p orbs
      ORBS_BOOT=0 ORBS_DUMP="recall marks" cargo run -p orbs
      ```
      A walled way reads as nought, so `threading` waited for the next box (§19).
- [x] ✅ **Variables and sets — `let … be …` and `for each …`** — `roaming` is
      the maze in 19 lines with a true minimum. A variable holds a name and
      nothing more; the cursor is named after the set; `let … be` because `set`
      is a `dial` synonym; a set is declared by the fixture. Fewer lines is not
      fewer ticks (§19)
      **See it** — the solver, and it must walk out of three different mazes:
      ```bash
      ORBS_BOOT=0 ORBS_DUMP="peruse roaming.spell" cargo run -p orbs
      for s in 3 11 17; do ORBS_SEED=$s ORBS_BOOT=0 \
        ORBS_DUMP="attend archive; research" \
        ORBS_THEN="invoke roaming; meditate 3600; meditate 3600; survey cabinet" \
        cargo run -q -p orbs; done      # `fragment 1` each time
      ```
      ...and the manual, which now says which sets a room has — a `for each` is
      unwritable without that, and nowhere else says it:
      ```bash
      for room in archive lens laboratory; do ORBS_BOOT=0 ORBS_GRID=100x40 \
        ORBS_DUMP="attend $room; recall scripting" cargo run -q -p orbs; done
      #   archive: way.   lens: socket, sigil.   laboratory: no such section
      ORBS_BOOT=0 ORBS_DUMP="recall let; recall for" cargo run -p orbs
      ```
- [x] ✅ **Parts, and `SCRIPT_BUDGET` becoming a number the weave tree sets**
      (`0.3.24`). `part gathering()` names a run of lines and `gathering()` runs
      it on a stack of descents. `part` was measured; a call is punctuation; a
      definition is top-level and one name means one part. `spell::budget` reads
      the weave. Since `0.4.2` a part takes arguments and its store is
      per-descent (§19)
      **See it** — a part told what to work on, which is the whole of what
      parameters bought. One body, two reagents, no `let` between them:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; scribe tending" \
      ORBS_EDIT="edit\npart load(what)\ngrind what\nempty mortar_and_pestle\nend\nload(sage)\nload(rock-salt)\n<esc>\nquit" \
      ORBS_THEN="invoke tending; meditate 40; peruse laboratory.log" cargo run -p orbs
      #   ground-sage, then ground-salt, from one run of lines
      ```
      ...and the scope: the part binds its own `herb`, and the caller's is
      untouched:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; scribe scoped" \
      ORBS_EDIT="edit\npart load()\nlet herb be rock-salt\ngrind herb\nempty mortar_and_pestle\nend\nlet herb be sage\nload()\ngrind herb\n<esc>\nquit" \
      ORBS_THEN="invoke scoped; meditate 40; peruse laboratory.log" cargo run -p orbs
      #   rock-salt: dispensary to mortar_and_pestle
      #   sage: dispensary to mortar_and_pestle
      ```
      **See it** — a definition and its call, read back by `interpret`:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend laboratory; scribe tending" \
      ORBS_EDIT="edit\npart gathering()\ngrind sage\nempty mortar_and_pestle\nend\nrepeat 2\ngathering()\nend\n<esc>\ninterpret" \
        cargo run -p orbs
      ```
      ...and the same file run, which grinds **twice** from one body:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; scribe tending" \
      ORBS_EDIT="edit\npart gathering()\ngrind sage\nempty mortar_and_pestle\nend\nrepeat 2\ngathering()\nend\n<esc>\nquit" \
      ORBS_THEN="invoke tending; meditate 40; peruse laboratory.log" cargo run -p orbs
      ```
      ...the six refusals, each on its own line:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=110x40 ORBS_DUMP="attend laboratory; scribe broken" \
      ORBS_EDIT="edit\nmissing()\npart gathering()\ngrind sage\nend\npart gathering()\nsurvey\nend\nrepeat 2\npart inner()\nsurvey\nend\nend\ngathering(sage)\npart twice(a, a)\nend\nhauling(a b)\n<esc>\nquit" \
      ORBS_THEN="invoke broken; meditate 3" cargo run -p orbs
      #   1 no such part / 5 named above / 9 outside every block /
      #   13 wrong number / 14 wants a name / 16 a name in each slot
      ```
      Line 15 is a stray `end`: a heading the orb cannot read opens no block.
      ...runaway recursion, bounded and loud rather than silent:
      ```bash
      ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe deep" \
      ORBS_EDIT="edit\npart spiral()\nspiral()\nend\nspiral()\n<esc>\nquit" \
      ORBS_THEN="invoke deep; meditate 30; peruse orb.log" cargo run -p orbs
      #   "deep.spell calls spiral too deep to follow. it stops there and goes on"
      ```
      ...and the steps-per-tick nodes on the weave screen, refusing like every
      other node:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" \
        ORBS_WEAVE="mastery\ntake" cargo run -p orbs
      ```
- [x] ✅ **A resolution policy** (`0.3.25`) — `tower::reach` names four axes:
      scope, kind filter, naming, and ordering. Every lookup is one call saying
      which setting of each; what a miss tells the player stays at the call site;
      the defaults are §7
      **See it** — the two behavioural suites, untouched, as the equivalence
      proof; and the policy's own seven tests, which pin each axis:
      ```bash
      cargo test -p orbs-sim --test fetching --test arsenal
      cargo test -p orbs-sim --lib tower::reach
      ```
- [x] ✅ **Builtins that act — and the plan for them was falsified** (`0.3.26`).
      Only the verb can be typed at cast: a spell makes its own inputs, so an
      argument cannot resolve then. `may_issue` is asked at cast as well as at
      the line, and the round-trip in `run_line` stays, as a decision (§19)
      **See it** — a forbidden verb in a branch that never runs, refused when the
      spell is cast:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend laboratory; scribe risky" \
      ORBS_EDIT="edit\nif the dispensary has quartz\nmeditate 3600\nend\nsurvey\n<esc>\nquit" \
      ORBS_THEN="invoke risky; meditate 8" cargo run -p orbs
      #   "risky.spell line 2: that is a word the orb will not take from a spell"
      ```
      ...and the pipeline that must **not** be refused by the same check:
      ```bash
      cargo test -p orbs-sim --lib tower::spell::tests
      #   the_cast_check_does_not_fire_on_a_line_that_makes_its_own_input
      #   a_spell_that_should_wait_still_waits_rather_than_being_refused
      ```
- [ ] ⚠ **A typed action at execution — re-scoped out of the box above, not
      done.** What remains of *"builtins that act"* is replacing the NLU
      round-trip inside `run_line` with a direct `Intent` construction. It is
      **not** blocked and it is **not** small: argument resolution lives inside
      `analyse`, and that is exactly where `would_block`'s `NounKind` filter and
      `pipeline::carry`'s slot numbering come from.
      - **A flat `fn(Verb, &[Value])` breaks `would_block` *silently*.**
        `touches()` filters on `argument.kind == NounKind::Place`
        (`spell/block.rs`), so a flat list makes it return empty, `would_block`
        returns `None`, and **every spell that used to wait starts being
        refused** — `waiting_since` never set, `PATIENCE` never tripped, most
        tests still green. `begins_work` fails the same way via `spending`, and
        `pipeline::carry`'s `slot(0)/slot(1)/slot(2)` reintroduces the
        renumbering bug §19 records
      - So a typed call **constructs an `Intent`**. The win is directness and
        correctness, not deleting the round-trip
      - **The regression guard already exists**:
        `a_spell_that_should_wait_still_waits_rather_than_being_refused` was
        written at `0.3.26` for exactly this, and it fails from the outside
      **See it:** a spell that should wait still waits rather than being refused
- [x] ✅ **Syntax highlighting in the spell editor** (`0.3.29`) — `parser::lexeme`
      splits a line into runs, a language fact in `orbs-sim`, lexical and
      world-free. Three weights: structure, content, noise. It declines on an
      accent and on the running line, and is not on `Style` (§19)
      **See it** — `ink` is the only instrument in the project that can show
      weight, so it is the gate:
      ```bash
      scripts/tui.sh start
      scripts/tui.sh type 'attend archive' 'scribe threading'
      scripts/tui.sh ink 6 40
      #   `repeat`/`until` bold, `the` dim, `stacks is idle` normal
      scripts/tui.sh stop
      ```
      ```bash
      cargo test -p orbs-sim --lib parser::lexeme   # the classification
      cargo test -p orbs-shell --lib sheet          # the weights, and the declines
      ```
- [x] ✅ **The scribing guide — the manual, in the room where you need it**
      (`0.3.30`) — a pane beside the editor: the spell's domain's vocabulary, or
      the page for the word under the caret. `guide` toggles it. Held on the
      `Editor` and refreshed on the keystroke, never in the painter; it yields
      whole below 97 columns
      **See it** — the two halves are the listing and the page, and only a
      running editor has a caret to move:
      ```bash
      scripts/tui.sh start
      scripts/tui.sh type 'attend archive' 'scribe threading' 'edit'
      scripts/tui.sh see        # the nine control words, then `here you can`
      scripts/tui.sh key Right Right Right Right Right Right
      scripts/tui.sh see        # ...`repeat` — its page, wrapped
      scripts/tui.sh key Down Down
      scripts/tui.sh see        # ...`follow` — a verb, the other key family
      scripts/tui.sh stop
      ```
      ```bash
      cargo test -p orbs-shell --lib guide     # both families, and the caret rule
      cargo test -p orbs-shell --lib sheet     # the split, and the yield
      scripts/play.sh spells::
      ```
- [x] ✅ **`parser::expect` — one answer to *what may come next***
      (`0.3.31`) — the prompt's Tab, its ghost and the editor's Tab were about to
      be three answers to one question; `expect(line, caret, &Situation)` is it.
      The scribing guide deliberately is not: it teaches canonical names only
      **See it** — the refactor's own gate is that nothing changed, so the
      order test is the instrument:
      ```bash
      cargo test -p orbs-sim --lib parser::complete   # incl. the order pin
      cargo test -p orbs-sim --lib parser::expect
      ```
- [x] ✅ **The guide floats what is expected** (`0.3.32`) — part way through a
      line the pane says what may go where the caret is. The states belong to
      `is`, not `wait`; three control words cannot open a line; touching a word
      is a page and past it an expectation; things before readings
      **See it** — walk one question end to end:
      ```bash
      scripts/tui.sh start
      scripts/tui.sh type 'attend laboratory' 'scribe reactive' 'edit'
      scripts/tui.sh type 'if '                 # the places
      scripts/tui.sh type 'the mortar_and_pestle '   # `is`, `has`
      scripts/tui.sh type 'is '                 # the eight state spellings
      scripts/tui.sh stop
      ```
      ```bash
      cargo test -p orbs-sim --lib parser::expect   # the table, and the legality
      cargo test -p orbs-sim --lib question         # WORDS agrees with `read`
      cargo test -p orbs-shell --lib guide
      ```
- [x] ✅ **Tab completion in the spell editor** (`0.3.33`) — `orbs_shell::tabbing`
      is readline's rules once, handing back a decision the caller applies; the
      guide is the listing; `Expectation::common` is the one shared-prefix rule
      **See it** — `ORBS_DUMP` writes no key events, so this is `tui.sh` only:
      ```bash
      scripts/tui.sh start
      scripts/tui.sh type 'attend laboratory' 'scribe tabbing' 'edit'
      scripts/tui.sh type 'gri'
      scripts/tui.sh key Tab          # -> `grind `
      scripts/tui.sh type 'sa'
      scripts/tui.sh key Tab          # -> `grind sage `
      #  ...and the cycle, where there is more than one answer:
      scripts/tui.sh key Tab Tab Tab  # lists, then `idle`, then `free`
      scripts/tui.sh stop
      ```
      ```bash
      cargo test -p orbs-shell --lib tabbing   # every rule, both directions
      ```
- [x] ✅ **The prompt hues what is typed into it** (`0.3.34`) — the same lexer
      through `orbs_shell::lexing`, with what the prompt cannot run suppressed.
      The whole line is lexed and a window drawn; one `Input` utterance however
      many runs (§14)
      **See it** — weight is invisible to `ORBS_DUMP`, so `ink` is the gate:
      ```bash
      scripts/tui.sh start
      scripts/tui.sh type 'attend laboratory'
      scripts/tui.sh ink 45 45     # `move` plain, `the`/`to` dim, `sage` plain
      scripts/tui.sh stop
      ```
      ```bash
      cargo test -p orbs-shell --lib prompt   # the runs, the scroll, the utterance
      cargo test -p orbs-shell --lib lexing   # the arbitration, both surfaces
      ```
- [x] ✅ **A spell is coloured, and §4 is widened to allow it** (`0.3.35`) — seven
      categories on two axes, spell text exempted from "intensity alone" and the
      accent triad untouched (§19). `Grammar` and `State` lexemes; a
      `Vec<(Rect, Lexeme)>` side-table on the `Frame`, apart from `Tint`;
      `monochrome` declines the palette
      **See it** — `ORBS_DUMP` prints the runs now, which is the only text gate
      there is for a colour:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend laboratory; scribe hues" \
      ORBS_EDIT=$'edit\nrepeat 3\nif the mortar_and_pestle is idle\ngrind the sage\nend\nend\n# a note\n<esc>' \
        cargo run -p orbs | grep -A14 'lit runs'
      #   control "repeat" / number "3" / grammar "is" / state "idle" ...
      ```
      ```bash
      # ...and the colours themselves, which only `ink` can see.
      scripts/tui.sh start
      scripts/tui.sh type 'attend laboratory' 'scribe hues' 'edit' \
        'repeat 3' 'if the mortar_and_pestle is idle' 'grind the sage' 'end' 'end'
      scripts/tui.sh ink 5 40
      #   `repeat` magenta/bold, `is` dark-cyan, `idle` cyan, `grind` yellow,
      #   `the` default/dim, `mortar_and_pestle` default
      scripts/tui.sh stop
      ```
      ```bash
      cargo test -p orbs-sim  --lib parser::lexeme   # the two new classifications
      cargo test -p orbs-tui  --bins theme           # the palette, and the decline
      cargo test -p orbs-tui  --bins a_lit_run       # ...all the way to the wire
      ```
- [ ] **The terse register — planned, deliberately not built here.** §8's gating
      ladder earns a second spelling the way it earns conditionals, but it
      doubles what the parser, the manual and `interpret` must each cover, and
      `interpret` and `run_line` are already two expressions of one rule that
      have disagreed twice. Nothing above needs it.
      **See it:** the same spell written both ways, read back identically
- [x] ✅ **`RecordKind::ScriptLine` gets its first producer** (`0.3.23`) —
      `peruse`/`sift` over held text. `FieldName::Line`, because the number was
      never a tick; the doubling guard learned the kind; the editor announces a
      row once. Colour does not decide where a sentence ends
      **See it** — the same twelve lines, before and after `F5`:
      ```bash
      scripts/tui.sh start
      scripts/tui.sh type 'attend archive' 'peruse threading.spell'
      scripts/tui.sh key F5     # `text 11, follow north` — was `row  tick: 11, …`
      scripts/tui.sh key F5     # ...and the pane's gutter is unchanged
      scripts/tui.sh stop
      ```
      ...the log's own numbering, which was the quieter half:
      ```bash
      ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend archive; research; \
        follow west; follow west; peruse archive.log" cargo run -p orbs
      #   line: 1 … line: 4, at world tick 5 — and the boot card's `tick: 0` stands
      ```
      ...and the editor, which needs a dump because `F5` cannot reach it:
      ```bash
      ORBS_BOOT=0 ORBS_DUMP="attend archive; scribe threading" cargo run -p orbs
      #   `Text  1 repeat until the stacks is idle` — one utterance, not two
      ```
- [ ] **`F5` over the three modal surfaces.** Opened by the box above, which
      found two §14 defects and could only show one of them through the key
      built to show exactly this. `prompt::paint` returns early for the editor,
      the loom and the maze, so the linear mirror never runs over any of them —
      and they are the three surfaces that take the *whole* pane, which is to say
      the three where a reader has nothing else to fall back on.
      **See it:** `scribe` a spell, press `F5`, and read the file back
- [x] ✅ **Naming pass for the remaining canonical commands** (`0.3.28`), moved
      here from Phase 1 — §18 item 2, listed blocking. Thirty verbs, three
      registers, typed and read back. The lens had no plain-English way in
      (`spy peek try` was one phrase); §6.1's table had drifted six rows and is
      now a lint against `SYNONYMS` (§19)
      **See it** — the three words that did not work, echoing the canonical form:
      ```bash
      for w in spy peek try probe; do
        ORBS_BOOT=0 ORBS_DUMP="attend lens; $w" cargo run -q -p orbs; done
      #   every one echoes `probe`; `spy` used to echo `! spy`
      ```
      ```bash
      cargo test -p orbs-sim --test naming     # 21 rules, including the table
      ```
- [x] ✅ **Spell parts: named, composable** — ~~invoked across files~~ **struck**
      (§19): a spell is one file, and composition within it shipped at `0.3.24`
      **See it:** `part gathering()` and `gathering()`, in the `0.3.24` box above
- [ ] Hidden-directory authoring plan (~80 fragments' worth), moved here from
      Phase 1
      **See it:** find one without being told it is there
- [x] ✅ **~~What a part costs to hold~~ — struck with the item above** (§19): an
      in-file part is not held separately, so there is no price to draw.

**Scarcity: concentration**, which prices spells — §11.5's shared pool, one slot
at 16 experience.

---

## Phase 4 — Defense ✅

**Exit:** wards placed against a pressure the player survives by *choosing*, and
then a spell that survives it. **Met** — a course of wards hauled between three
stations by hand, and `holding` bound in the sanctum while the player brews.

> **This was Phase 9, and Enchanting was Phase 4** — they swapped (§19). The room
> is `sanctum/`, where §7 and §10 said `battlements/`: the mechanics did not
> move, the fiction did (§19).

- [x] ✅ **The reflex-avoidance mechanism, decided and recorded in §19 *before*
      anything was built.** The **Tower of Hanoi**: no clock in the puzzle, every
      ward on screen, so outcome follows choice by construction.
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend sanctum; muster; haul wellspring barrier; haul wellspring barrier" cargo run -p orbs`
      — the second haul is refused in voice, and nothing anywhere is timed
- [x] ✅ **The `sanctum/` domain, its stations, and its wards.** A `pylon` that
      draws a course up, three stations — `wellspring`, `conduit`, `barrier` —
      and two verbs. Each station publishes `potency` **only while it holds a
      ward**, so an empty one answers `is empty`; the pylon publishes `odd` and
      `integrity`.
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend sanctum; muster; survey pylon; survey wellspring" cargo run -p orbs`
- [x] ✅ **Integrity, which wears down on its own — the first drain in the
      game.** Everything else the tower has is a faucet. A point every 30 ticks,
      and a finished course puts it back; a worn tower musters a *taller* course,
      so neglect is expensive and never ruinous (§11.5).
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend sanctum; survey pylon; meditate 3600; survey pylon; muster; survey pylon" cargo run -p orbs`
      — 100 → 0, then a course of six or seven where a kept tower gets three
- [x] ✅ **A re-warding spell, which is what hands off to Phase 8.** `holding`
      is the cyclic Hanoi rotation: one `part`, three `let` pairs, and a parity
      read off the world. It solves in exactly `2^n − 1` hauls — optimal — and
      bound it musters afresh every lap.
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend sanctum; invoke holding; meditate 400" ORBS_THEN="peruse sanctum.log" cargo run -p orbs`
- [ ] ⚠ **The accessible mode — moved to Phase 8, not struck.** §14 requires one
      of a *real-time* surface, and a course waits for ever; *"screen-reader mode
      advances siege ticks on player input"* is Phase 8's surface (§19).

**Scarcity: integrity, which decays** (§19). Neither verb takes the production
slot, so a bound `holding` runs beside a brew.

---

## Phase 9 — Enchanting

**Exit:** a buffed instrument visibly works faster, the panel says so, and the
buff decays.

**Derived** in §10 — cut from brewing's pipeline. The minigame is a **lattice**:
Lights Out on three columns, a charm binding only when every glyph is lit (§19).
It landed as `0.8.19`, before the version stopped naming phases.

- [x] ✅ **The `forge/` domain and a buff that lands on an instrument** — the
      seventh and last of §10's domains, so every box on the rail is lit. `imbue`
      opens a lattice, `snap` turns a column, `anneal` lets it fall; five charms,
      each read at a different site
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend forge; imbue mortar_and_pestle hurried; snap apex; anneal; meditate 25; attend laboratory; grind sage; meditate 4" cargo run -p orbs`
      — the grind lands in four ticks where it takes eight
- [x] ✅ **Decay, and the panel row that shows it draining** — a charm is an
      interval, so nothing ticks it down and `meditate 3600` inside one `step`
      behaves like 3,600 watched ticks
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend forge; imbue mortar_and_pestle hurried; snap apex; anneal; meditate 700; attend laboratory; grind sage; meditate 4" cargo run -p orbs`
      — past the charm, the same grind takes eight again
- [ ] **The shared-engine extraction**, pulled forward from Phase 13a — §10.1
      already says it belongs *"when the derived domains arrive"*, and they arrive
      here. **Its gate is that nothing changes**, which is Phase 13a's wording and
      the right one for a refactor
      **See it:** `scripts/dumps.sh` byte-identical before and after
- [ ] **The `pylon` and the `circle` gain a recipe table and a store** — a
      separate box, because it is new behaviour and the extraction's gate is that
      there is none. Merging the two makes the refactor unverifiable, which is how
      the duplicate box below got its contradictory gate in the first place.
      `build.rs` already calls a room without a store a defect — *"a room with an
      instrument and nowhere to set anything down is a room where `empty` is a
      word that can never work"* — and four of seven rooms are in that state.
      **This is what every supply edge in Phase 12 is waiting on** (§19)
      **See it:** `attend sanctum`, `survey pylon` — it names what it would take

> **The extraction is written twice and the two disagree.** It is this box and
> Phase 13a's, whose See-it line is *"the game plays identically before and
> after — this one is a refactor, and its gate is that nothing changes."* One
> asks for new behaviour and one for none. They are now one refactor here and one
> behaviour box beside it; 13a's entry points at these rather than repeating them.
- [x] ✅ **A maintenance spell** — `tending_forge`, and the eight-rung lookup
      table lives in it rather than in the world. §8.1's *"a rule, not a
      memory"*: the columns publish the residue and the spell holds the mapping
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend forge" ORBS_THEN="invoke forging; meditate 120; peruse forge.log" cargo run -p orbs`

**Scarcity: the buff's own lifetime, and the slot.** Maintaining a buff competes
with making things, so idle buff-time is waste exactly as idle lit time is.

> ~~**A second potion sink.**~~ **Struck**: the forge spends **quintessence**,
> shared with the siege (§19).

---

## Phase 5 — Summoning ✅

> **The chant below was replaced at `0.15.0`** — the menagerie is a circle of
> logic gates now; see [The circle](#the-circle) and DESIGN.md §19. This section
> is the record of what was built.

**Exit:** a chant performed by hand or by a spell yields troops, and a botched
one leaves the tower worse. **Met** at `0.5.6`. Rewritten at `0.5.0`: autonomy
moved to Phase 8, and §10's *"allocation"* row was withdrawn (§19).

- [x] ✅ **Head of phase: the §10 reversal, and the keyboard's one owner**
      (`0.5.0`) — one `Focus` enum in `orbs-shell`, read by both frontends. The
      obvious gate was vacuous: a dump builds no `App` (§19)
      **See it:** `scripts/play.sh` (113 scenarios, a live keyboard), and
      `cargo test -p orbs --bins no_surface_lets_a_keystroke_reach_the_prompt`,
      which opens each of the four surfaces in a real `App` and proves the prompt
      stays empty
- [x] ✅ **The `menagerie/` domain, the chart, and `sing` typed** (`0.5.1`) — a
      figure of twelve syllables; four misses collapse it and the barrier pays.
      Neither verb takes the slot
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; sing skyward;
      sing skyward; sing skyward; sing skyward" cargo run -p orbs` — four blind
      misses, and the figure comes apart. `cargo test -p orbs-sim --test
      chanting` is the loop proved through the real verbs
- [x] ✅ **The board, and the pace that makes the room playable** (`0.5.2`) —
      syllables rise to a rule, eight ahead; `PACE` is four; `bide <n>` is the
      language's tenth word (§19)
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; meditate 2"`
      against the same line without the `meditate` — the figure has risen two
      rows, which is the timing being visible. It fits the 80×22 floor. A `sing`
      on the tick after `summon` answers *"too soon"*: a dump has no clock
- [x] ✅ **A spell that times its own answer** (`0.5.3`) — the solver strikes
      twelve of twelve at two steps a tick and collapses at one, so the
      menagerie rewards concentration (§19)
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; peruse
      chanting.spell" cargo run -p orbs` — and note it is *meant* to fail at the
      shipped budget. `cargo test -p orbs-sim --lib tower::chant` holds the pace
      and window arithmetic
      > `bide until` shipped here and was withdrawn at `0.5.8`
- [x] ✅ **`steps_1` is takeable — the first real node in the tree** (`0.5.4`) —
      the screen hands an id to `Sim::take`, recorded as `Submission::Took`; both
      sides check every rule
      **See it:** three distillations to 24, then
      `ORBS_WEAVE="mastery\ntake\nquit"` — *"the orb takes steps_1. it thinks a
      little faster now"*. Then `invoke chanting`: **24 of 24 struck** where at
      one step a tick it collapsed. `cargo test -p orbs-sim --test progression`
      holds the take, the refusals and the replay
- [x] ✅ **The interactive surface, and the accommodation** (`0.5.5`) — `chorus`
      hands the arrows to a running figure beside the transcript. `F9` makes a
      chant patient and reaches the same ceiling, so it is an accommodation
      (§14)
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; chorus"
      ORBS_CHANT="<up>\n<left>" cargo run -p orbs` — the presses land and read
      *too soon*, because a dump has no clock. Add `ORBS_PATIENT=1` and the same
      presses land cleanly, which is the whole of what the mode does.
      `cargo test -p orbs-sim --test chanting` pins the parity, with the played
      chant as its control
- [x] ✅ **The troop is a material the game knows about** (`0.5.6`) — a page, a
      tint, a `debug_spawn` name; `substances` names it, since a lint walks only
      authored materials
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="recall troop; debug_spawn troop 3;
      survey arsenal" cargo run -p orbs`. And check the verdant trap while you
      are there: four `wield verdant-scroll` must not shelve troops on the
      dispensary as an inexhaustible herb

- [x] ✅ **Reviewed, and four blockers fixed** (`0.5.7`) — fifteen findings on a
      green phase, three blockers in code whose own comment cited the rule it
      broke, and none of the four See-it instruments wired (§19)
      **See it:** `cargo run -p orbs-render --example screens` has a figure now;
      `scripts/dumps.sh` captures seven menagerie screens; `scripts/play.sh
      menagerie::` plays five scenarios on a real keyboard; and
      `cargo run -p orbs-balance -- run chanting --ticks 7200` reads **0.243**.
      The bound solver yields **20 troops from another room**, where before the
      fix it yielded none

- [x] ✅ **`bide until` withdrawn — the delay is the player's arithmetic again**
      (`0.5.8`) — a spell reading its delay off the world computes nothing.
      `Kind::Bide` holds a literal; the solver is a constant-length `for each`
      that sings outside the loop. `RunningSave` keeps `biding`, `FORMAT` 4 → 5
      (§19)
      **See it:** `cargo test -p orbs-sim --test chanting the_shipped_solver` —
      the hook as an assertion rather than a claim, collapse at one step and
      close at two, which is deliberately the pair. `cargo test -p orbs-sim --lib
      tower::spell::program::tests::bide_takes_a_count` holds the refusal, and
      `cargo run -p orbs-balance -- run chanting --ticks 7200` still reads
      **0.243** — the harness reads `until` off the model, where the language
      cannot

- [x] ✅ **A satchel, and one spell can hand another a name** (`0.5.9`) — `queue
      <name>` as a verb and `pull <name> from satchel` as a control word, since
      pulling binds a name. It yields while empty and never reaches `PATIENCE`.
      A component, not children; only the local satchel is offered (§19)
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; debug_take satchel_1;
      queue skyward; queue earthward; queue skyward; survey satchel" cargo run -p
      orbs` — a name twice, in order. `cargo test -p orbs-sim --test satchel`
      holds ten claims, one per way this could have been half-built

- [x] ✅ **`alongside` — two cursors in one spell, and the unlocks** (`0.5.10`).
      A `Running` holds a `Vec` of `Strand`s, stepped in order, at most four; a
      blocked strand no longer ends the entity's tick. Done as a swap, so the
      balance table is unchanged to four places. `cursors_1` sells ergonomics
      **See it:** `scripts/play.sh satchel::` plays five scenarios on a real
      keyboard, including the fork feeding itself. `cargo test -p orbs-sim --test
      strands` is nine, and `two_cursors_interleave_the_same_way_at_two_steps_a_tick`
      is deliberately written at budget 2 — batch and round-robin are identical
      at one step a tick, so a pin written there pins nothing.
      `debug_take <id>` is why these lines are short: `debug_spawn`'s argument,
      applied to two hundred ticks of laboratory

- [x] ✅ **Worked examples of both forms, and a rail that can count** (`0.5.11`).
      `ordering` + `milling` are the two-spell channel and `coursing` the
      two-cursor one, all cast in tests. The rail counts cursors and truncates
      the name, never the count; `status` gained a `casting` section from the
      same walk (§19)
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend sanctum; debug_take satchel_1;
      debug_take cursors_1" ORBS_THEN="invoke coursing; meditate 6; status" cargo
      run -p orbs` — the rail reads `►coursing +1` and `status` says
      `coursing  sanctum, on 2 cursors`. `cargo test -p orbs-shell --lib rail`
      holds the truncation, and
      `the_rail_counts_every_cursor_and_status_names_them` holds the pair

- [x] ✅ **`recall apprentice` — the lesson, where `recall scripting` is the
      reference** (`0.5.12`) — nothing taught the order of making a spell. The
      worked example is built from the room, and the way in is `help` (§19)
      **See it:** `for room in laboratory archive lens sanctum menagerie
      grimoire; do ORBS_BOOT=0 ORBS_DUMP="attend $room; recall apprentice" cargo
      run -q -p orbs; done` — six rooms, six examples.
      `scripts/play.sh the_apprentice` **follows the page** from `help` to a
      spell that earns, on a real keyboard, which is the gate that matters: a
      tutorial's lines are typed rather than read past.
      `the_apprentice_only_shows_lines_the_room_can_run` asks both questions —
      resolves *and* offered — because `grind sage` in the lens resolves and is
      then refused

**Scarcity: integrity.** A chant costs nothing to attempt; enough mistakes wear
the pylon. Time, never progress (§11.5).

---

## Phase 10 — Progression

**Exit:** a fresh tower is a laboratory and nothing else, and the rest opens as
the work is done — rooms, recipes and charms along seven mastery lines and one
Ley Line with a choice at each fork — and the line runs to the soft ending.

**The two tracks swapped natures here** (§19). **The Ley Line is the tower's**:
one line on total experience, of steps (passing is the grant) and forks (one of
three lanes). **Mastery is per domain**: a straight line of deeds per room,
reached in order (six since the grimoire left the rail). Either may open something, which is how the tower grows. Every number is
a first pass and `orbs-balance` decides it.

- [x] **Deeds, tallies, and the two tracks re-strung** (`0.10.1`) — `[[ley_line]]`
      stations grant or fork; `[[mastery]]` is seven lines of deeds checked at
      load, thirty stations. `tower::done` is the one door every completion goes
      through. `tower::Opened` holds what has been opened, and `Sim::new` opens
      everything. The weave draws one track at a time; `debug_reach <id>`. Save
      `FORMAT` 10 (§19)
      **See it:** `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory;
      kindle charcoal; debug_spawn clarified-draught 1; distil clarified-draught;
      meditate 60; weave" ORBS_WEAVE="mastery"` — the laboratory's first station
      reads `«•»`, the panel *"a clarity brewed · 1 of 1 · reached · opens
      archive"*, and the log says *"the laboratory line advances: a clarity
      brewed"*. Three distillations and `ORBS_WEAVE="ley\n<right>\n<down>"` —
      the fork at 24 open, its second node aimed. `ORBS_WEAVE="mastery\ntake"` —
      *"laboratory_1 is reached by doing, not by taking"*. `cargo test -p
      orbs-sim --lib tower::mastery` holds the order, the double reach and the
      once-only announcement
- [x] **Sealed rooms** (`0.10.2`) — `Sim::sealed` is a fresh game, `Sim::new`
      stays open, and `Opened::start` is derived from the content.
      `orbs_shell::fresh` is the one reader of `ORBS_SEALED`. One gate,
      `tower::sealed_room_of`, with `tower::Sealed` marking nodes for sabotage.
      A shut room stays nameable so `attend archive` refuses in voice; `[world]
      sealed` travels with the seed (§19)
      **See it:** `ORBS_SEALED=1 ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="attend
      laboratory; attend archive"` — *"the archive is not yours yet"*, the boot
      report naming the laboratory and the arsenal only, and six dark boxes on
      the rail. Then `ORBS_SEALED=1 ORBS_BOOT=0 ORBS_DUMP="attend laboratory;
      kindle charcoal; debug_spawn clarified-draught 1; distil clarified-draught;
      meditate 60; attend archive"` — *"the archive is yours now. attend it"*
      and the room answers. `scripts/play.sh sealed::` plays the same thing.
      `cargo test -p orbs-sim --test sealed` holds every door, the replay and
      the migration
- [x] **Charms open on the forge line** (`0.10.3`) — and on the line of the
      room a charm blesses: `fruitful` with the forge's first station,
      `bountiful` with the archive's fourth, `shielded` with the lens's second,
      `whetted` with the sanctum's second. `imbue` refuses an unearned charm in
      voice, and the forge opens knowing `hurried`, its flagship
      **See it:** `ORBS_SEALED=1 ORBS_BOOT=0 ORBS_DUMP="debug_reach forge_1;
      attend forge; imbue mortar_and_pestle whetted; imbue mortar_and_pestle
      hurried"` — *"the forge cannot lay whetted yet"*, then the flagship charm
      rising. `debug_reach` opens the room it reaches into; `cargo test -p orbs-sim --test sealed
      the_wall_is_armed_by_the_sanctum_and_a_charm_by_its_station` is the same
      thing held by a test
- [x] **The rail's percentage and the domain's road** (`0.10.4`) — the rail's
      one progression reading, silent, on the state row's right edge; the road
      under the pane's title, taken off the body first. `stations.rs` is the one
      vocabulary the road and the weave draw with
      **See it:** `ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="attend laboratory;
      kindle charcoal; debug_spawn clarified-draught 3; distil clarified-draught;
      meditate 60; empty alembic; distil clarified-draught; meditate 60; empty
      alembic; distil clarified-draught; meditate 60"` — the road under the
      title with `five potions brewed · 3 of 5` at its end, and the laboratory's
      rail box reading `burning  60%`
- [x] **`recall` says where you stand** (`0.10.5`). The room's primer ends
      with its line as words — *"laboratory line: one of six reached. next,
      five potions brewed, 3 of 5"* — composed from the same reading the road
      draws, so a player who asks is told what a glance would show
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; recall"` — the
      primer's last line; `debug_reach laboratory_6; recall` — *"every
      station reached"*
- [x] **The line to the soft ending** (`0.10.6`) — sixteen stations from 16 to
      10,000, nine steps and seven forks of three lanes, every grant built.
      `tower::grant` is the one parser and a reader per grant. The lane rule
      fails the load; a node held above its fork's total is kept. The line draws
      on a logarithmic scale, and `status` says the pool and its ceiling (§19)
      **See it:** `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave"
      ORBS_WEAVE="ley\n<right>\n<right>\n<down>"` — sixteen stations, the
      forks three deep, `40` aimed at its war node and the panel reading *"a
      worn wall keeps more pool · costs 40 · war"*. Then, each grant on the
      surface it moves: `ORBS_DUMP="debug_take fuel_1; attend laboratory;
      kindle charcoal"` — *"fuel for 720 ticks"* where it was 600;
      `ORBS_DUMP="status; debug_take pool_1; status"` — the ceiling four
      higher; `ORBS_GRID=120x45 ORBS_DUMP="debug_take edge_1; attend bailey;
      defend"` — the board's garrison odds 55% where they were 50%;
      `cargo test -p orbs-sim --test grants` holds the other seven, each
      against its own number

**✅ Folded in after three reviews of the whole phase** — thirteen defects, each
in §19. The worst: a Ley step's `opens` was never applied, so the grimoire and
the forge were shut for ever in a player's tower.
  **See it:** two clarities in a sealed tower — *"the grimoire is yours now"*,
  then `attend grimoire` answers; `scripts/play.sh sealed::` plays it.

**✅ And the shipping profile was checked for the first time**: release had 18
warnings and 30 failing tests, all debug-word tests and their helpers, now gated
by the same `cfg`. No product code was wrong.

**Three findings are left open**, as design calls:

1. **A shut room's interior is nameable and tab-completable.** Splitting it needs
   a scene that can resolve a word without offering it.
2. **The shipped spellbook is readable while the grimoire is shut** — an
   onboarding decision (Phase 14's).
3. **The laboratory's fifth station is behind the archive's third** — `insight`
   needs herbs only a `verdant-scroll` shelves, and nothing in the room says so.

---

## Phase 11 — Renown

**Exit:** there is a reason to keep making things after the Ley Line stops buying
anything, and it is a number that can go down.

Experience past the soft ending bought nothing, so §11.5's condition for a
prestige layer was met by construction. **Renown is the tower's second number**:
earned by making things and fighting well, lost by fighting badly, spent to buy
a smaller siege, and it sets how big a siege arrives. It supersedes three
recorded decisions and does not touch `CAPACITY = 1` (§19).

- [x] **Renown accrues and is seen** (`0.11.1`) — minted on makings only, at the
      one completion door, rounded up. Earning is silent and only losing speaks.
      No `FORMAT` bump; on `status` and the weave, not the rail
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal;
      debug_spawn clarified-draught 1; distil clarified-draught; meditate 60;
      status"` — `renown ......... 4` under `experience ..... 8`; and
      `ORBS_WEAVE="mastery"` in place of `status` puts `renown 4` at the right of
      the headings row.
- [x] **The siege moves it both ways** (`0.11.11`) — per exchange in `hold`,
      net of what was mended; the fight speaks once, on settling, measured from
      when the enemy arrived. A fall costs a stake scaled by how far short it
      fell; `RENOWN_PER_FOE` is derived (§19)
      **See it:** a round moves it, and says nothing —
      `ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="attend bailey; defend; status; pledge d20 to buckler; hold; status" cargo run -p orbs`
      → `renown 0` then `you lose 1 and take 3` then `renown 2`, with no line
      between. It must pledge: an unpledged round nets nought.
      Then the fight speaking — `debug_siege` in place of the first `status`
      gives `they are singing about it: 37 renown, 37 in all` under
      `the wall holds`. And a loss, in the log where a spell's records go:
      `ORBS_BOOT=0 ORBS_DUMP="debug_renown 200; attend bailey" ORBS_THEN="invoke besieging; meditate 900; peruse bailey.log" cargo run -p orbs`
      → `word gets about: the wall cost 31 renown, 169 left`
- [x] **Ten ranks, and a gauge for each standing** (`0.11.2`) — two bracketed
      bars at the top of every pane. Titles, not gates, so losing one is safe.
      `Toward::among` fills from the tier behind; `Painter::gauge` measures a
      tier, capped at 40 cells; three sentences for a rank; the rows yield
      before the transcript
      **See it:** `ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="debug_renown 900;
      attend laboratory"` — `renown [||||...] 100/800  magister` under the ley
      gauge; then `debug_renown 900; debug_renown 100; debug_renown 0` for the
      three sentences in order.
- [x] **Renown sets the siege, and `petition` turns it down** (`0.11.12`).
      Fame lengthens the tail of what comes up the road; `petition` spends
      standing to lower the ceiling. The floor never moves; twelve is where
      `outnumbered` turns over; paying drops a rank, so it limits itself. Every
      existing seed is untouched (§19)
      **See it:** the tail lengthens —
      `ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend" cargo run -p orbs` draws
      `5 come up the road`, and the same line after `debug_renown 9000` draws
      `6`. Then buying it down:
      `ORBS_BOOT=0 ORBS_DUMP="debug_renown 9000; attend bailey; petition; petition; petition; defend" cargo run -p orbs`
      — `word goes out. at most 11 will come, for 7 renown`, then `9`, then `8`.
- [x] **The gauges warm as they fill** (`0.11.4`) — red through yellow to green
      in six steps, a `Depiction` and never an accent. Green is arrival, and the
      fill length carries the information (§14)
      **See it:** `cargo run -p orbs-render --example screens` — the ramp printed
      as `A`–`F` at every tenth, which is the *only* See-it it has: the glyph is
      `|` at every step, so a dump shows a bar filling and proves nothing about
      the warming. On a tube, `ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="attend
      laboratory"` and watch the two rows at the top.
- [x] **The grimoire leaves the rail** (`0.11.3`) — no box, no mastery line:
      `DOMAINS` is the rooms you work in, and the grimoire raises no instrument
      (§19). It keeps its spells and its seal, now `opened::is_room`'s question
      **See it:** `ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="attend laboratory"` —
      six boxes on the rail and no grimoire; `ORBS_SEALED=1 ORBS_DUMP="attend
      grimoire"` still refuses in voice, and two clarities still open it.
- [x] **The arsenal is worth what your industry is worth** (`0.11.13`). How much
      help a thing gives is matched to the rate you make it at: `fresh`, `thin`
      at half, `spent` and refused. It replaced the cap, which was struck (§19).
      A rate, not a timestamp; total stock never enters it; surplus sells.
      `debug_spawn` stamps a full store. (Superseded by *Supply* at `0.18`)
      **See it:** stock it, then watch it go —
      `ORBS_BOOT=0 ORBS_DUMP="debug_spawn warding 5; survey arsenal; meditate 1200; survey arsenal; meditate 1800; survey arsenal" cargo run -p orbs`
      → `state: fresh`, `fresh`, then `spent`, with **`qty: 5` throughout**,
      which is the proof stock is not what is being measured. Then the refusal,
      which keeps the potion:
      `ORBS_BOOT=0 ORBS_DUMP="debug_spawn warding 1; meditate 3000; attend bailey; defend; quaff warding; survey arsenal" cargo run -p orbs`
      → `your warding stores are out. make more and come back`, and `qty: 1` still
      on the shelf after it.
- [x] **A spell can ask what has gone thin** (`0.11.14`) — `for each store`
      walks the arsenal and `if the store has thin` answers. Two room-scoped
      lookups learned the arsenal; a system republishes a store's word only when
      it changes; the test asserts the body ran
      **See it:** `cargo test -p orbs-sim --test scripting_the_siege a_spell_can_walk`
      — a spell that walks the arsenal, finds what is thin, and acts on it.
- [x] **Docs** (`0.11.14`) — the fourth renumber's §19 entry, §11.5's two missing
      rows, and §19's claim that the arsenal cap shipped, which it never did.
      **See it:** `grep -n "moved a fourth time" docs/DESIGN.md`; §11.5's table
      has nine rows; and the struck cap says what replaced it.

---

## Phase 11.5 — Interlude

**The tower becomes a place you move through.** Every screen cut; pane geometry
had been fixed in Phase 0.5, and pane content never was. No minor of its own; its
steps interleave with Renown's (§19).

- [x] **A screen leaves by a shape** (`0.11.5`) — `orbs_render::passage`, the
      `Frame` pass, and the kept cells a crossing departs from. `Wipe` alone, first
      **See it:** ✅ `cargo run -p orbs-render --example screens` — the "A
      crossing" blocks, eight fractions in a row. It caught its own demo glyphs
      on the first run: `◇` is outside CP437 and the arriving half printed `???`
- [x] **The shell keeps the last screen and knows what changed** (`0.11.6`) —
      `Passing`, `Showing`, and the two switches. The room and not the leaf, the
      surfaces that take the pane and not the ones that only take the keys
      **See it:** ✅
      `ORBS_BOOT=0 ORBS_PASSAGE_AT=0.30 ORBS_DUMP="attend laboratory; attend forge" cargo run -p orbs`
      — and `scripts/dumps.sh` byte-identical without it, which is the real gate
- [x] **The tower plays it, and `F3` stops it** (`0.11.7`) — the Bevy clock, the
      motion switch both clocks now share, and the one-tick floor between
      crossings. `orbs-tui` held a settled one until `0.19` gave it a switch
      **See it:** ✅ `cargo run -p orbs`, then `attend forge`. Then `F3` to `OFF`
      and `attend laboratory` — it cuts
- [x] **Three shapes, one per kind of change** (`0.11.8`) — `Wipe` for a room,
      `Gather` for a tool taking the pane, `Furl` for `F5`'s mirror
      **See it:** ✅ `attend forge`, then `wander`, then `F5` — three motions for
      three kinds of change
- [x] **The opening is a crossing too** (`0.11.10`) — the first screen change a
      player meets was the last one still cutting. The name grows in a letter at
      a time, `Stage::Close` takes the card away the same way, and the tower
      opens with the rail and the gauges pushing in. The box stays throughout
      **See it:** ✅ `cargo run -p orbs` and watch the opening. As text:
      `ORBS_DUMP=1 ORBS_BOOT=post:0.06`, `close:0.4`, and
      `ORBS_PASSAGE_AT=wake:0.35 ORBS_DUMP="attend laboratory"`

      ✅ **Each region leaves by its own edge** (`0.11.9`) — the gauges and road
      up, the panel and board right.
      ✅ **A gather actually converges**, with the flash rule split per family
      (§19). ✅ **The new screen no longer flashes whole for a frame** — `Drive`
      is ordered after `Input` (§19).
      **See it:** ✅ `cargo run -p orbs`, then `wander` — the laboratory flies
      into the middle and the maze flies out of it, with no whole frame between

**What it deliberately does not touch: the transcript, the border, its title, the
tower rail and the prompt** — a box that came apart would read as the machine
breaking.

---

## The circle

**Exit:** the menagerie is solved by choosing — a beast's temper is read off the
board and the circle limned to hold it, by hand or by a spell — and the chant,
its keyboard surface and its clock are gone.

**The rule** (§19): a beast's **temper** is a truth table over three senses, and
the **circle** is three glyphs in drawn wiring, each limned with one of six
**humours** — the six symmetric logic gates. `summon` draws and calls a beast in;
`limn <glyph> <humour>` sets a gate. A hold brings troops; a balk costs the call.
It **supersedes Phase 5's rhythm game** and makes summoning's scarcity time.

- [x] **The circle replaces the chant** (`0.15.0`) — `tower::circle`, proved
      exhaustively; `summon` and `limn` publish each humour and the temper's
      `fervour`, and nothing about a call. The board is a truth table written
      sideways. `taming` is an odometer over every circle. The chant's words,
      surface and tick system are gone; `FORMAT` 12 (§19)
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; limn keystone
      heed; limn sunwise oppose; summon" cargo run -p orbs` — the board beside the
      transcript, the answer row under the temper, and `▲` under the rows that
      balk (seed 181, re-measured at `0.15.4` when beasts became generated
      circuits: rows 1, 2, 6, 7 and 8, *"the beast balks at 5 rows, on call 1"*).
      `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; debug_circle; summon;
      survey arsenal" cargo run -p orbs` — *"the circle holds. 4 troops answer"*.
      `cargo test -p orbs-sim --test taming` is the domain through real verbs,
      including a bound search holding beasts while the player is in the
      laboratory; `cargo run -p orbs-balance -- run taming --ticks 7200` reads
      **0.0378** on seed 0 against a pin of 0.042 — ten seeds' mean — inside
      `taming`'s own thirty-five-percent band, which those seeds measured at
      0.0319 to 0.0550
- [x] **A spell that knows the logic** (`0.15.1`) — `winnowing`: the keystone
      narrowed on `fervour`, sunwise halved by De Morgan; about half `taming`'s
      calls. `taming` stays the pinned floor
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; invoke winnowing"
      ORBS_THEN="meditate 400; peruse menagerie.log" cargo run -p orbs` — *"the
      circle holds. 3 troops answer, on call 86"*, where the same line with
      `taming` and `meditate 900` holds seed 181's beast on call 170.
      `cargo test -p orbs-sim --test taming` casts both over 24 seeds and every
      rung
- [x] **The lesser circle** (`0.15.2`) — in a sealed tower the first beasts are
      the keystone alone over two senses, *which humour is this?*, until
      `menagerie_2` opens the whole circle. `Shape` is `Lesser` or `Whole` and a
      temper carries its senses, so one set of verbs, readings, board and save
      serves both. `FORMAT` 13
      **See it:** `ORBS_SEALED=1 ORBS_BOOT=0 ORBS_DUMP="debug_reach lens_1;
      attend menagerie; summon; limn sunwise heed; limn keystone mirror; summon"
      cargo run -p orbs` — *"a lesser beast gathers, lit on 3 rows. which humour
      is it?"*, *"sunwise is dark in a lesser circle"*, and a board of one line
      and four columns with `▲` under rows 1, 2 and 3.
      `cargo test -p orbs-sim --test lesser` holds five and finds the sixth whole
- [x] **Reviewed** (`0.15.3`) — no blocker; one major, and an event deed may be
      `fixed` now. §19 lists the rest
      **See it:** `cargo test -p orbs-sim --test lesser
      the_whole_circle_opens_after_five_lesser_beasts_at_every_length`;
      `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; limn oppose sunwise"
      cargo run -p orbs` — *"sunwise is limned oppose"*; `scripts/play.sh
      menagerie::`, five scenarios on a real terminal
- [x] **The seed builds the circuit** (`0.15.4`) — wiring, turned wires and a
      limning drawn from the seed; a temper is what the circuit answers.
      `circuit::drawable` is every valid circuit, and the spoken summary names
      each lit row's senses. `FORMAT` 14
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon" cargo run -p
      orbs` — *"sunwise yoke ← blood, ~breath"* and *"over blood and turned
      breath … lit on rows 2 (breath), 3 (bone), 5 (blood), 7 (blood bone)"*;
      `ORBS_SEED=3` for another beast. `cargo test -p orbs-sim --lib
      tower::circle` pins the space against an outside count; `cargo test -p
      orbs-shell --lib circle` holds two hundred beasts from the words alone
- [x] **Every new game its own seed** (`0.15.5`) — `orbs_shell::new_game_seed`
      from the wall clock unless `ORBS_SEED` names one; `orbs_shell::seed` stays
      the instruments'. Nothing in the sim reads the clock
      **See it:** two `orbs-tui` launches under tmux with no `ORBS_SEED` print
      two seeds in the condition report — *"orb 0 82177 cold start"*, *"orb 0
      211335 cold start"* — the loop in SEEING-IT's *`ORBS_SEED`* section.
      `cargo test -p orbs-shell --lib seed`; `cargo test -p orbs --bin orbs
      menuing` builds a new game from the seed it was asked for
- [x] **The circle, tested as a whole domain** (`0.15.6`) — 108 tests by what
      each holds: exhaustive, sufficient (board alone, speech alone),
      consistent (a seeded fuzz), deterministic, varied
      **See it:** `cargo test -p orbs-sim --test circle -- --nocapture` —
      *"214 holds, 2102 answer rows, 2102 calls, 5623 steps away"* from the fuzz
      in a debug build;
      `cargo test -p orbs-sim --lib tower::circle`; `scripts/play.sh
      menagerie::`, seven scenarios
- [x] **Reviewed a second time** (`0.15.7`) — fourteen corrections (§19), three
      of them rules: a call against par is never spent on a guess; a word an
      optional last slot cannot use is asked about, not dropped; every sentence
      the circle teaches a reader is one a reader is shown
      **See it:** `ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon;
      call the beast in; summon" cargo run -p orbs` — *"≈ summon"*, *"the orb will
      not call the beast in on a guess. type summon to call it"*, then *"the beast
      balks at 4 rows, on call 1"*; `ORBS_AUGURY=off ORBS_BOOT=0
      ORBS_DUMP="attend menagerie; summon; limn keystone xyzzy" cargo run -p orbs`
      — *"¿ limn keystone"*, and the keystone still at `yoke`;
      `cargo test -p orbs-sim --test augury circle`

---

## The scrivener

**Exit:** a phrasing that works at the prompt works inside a `.spell`, the file
on disk is still exactly what the player typed, and a replay builds the same
program without a model in the room.

**The divergence this closes**: `crush the sage` works at the prompt and is
*Referent missing* in a spell. A spell's strictness is about resolution, not
reading (§19).

> **The reading is derived and the file is not.** `Held` stays byte-exact and
> `Read` sits beside it: discardable, rebuilt per line, and what `spell::compile`
> compiles.

- [x] **The seam, the cache, and the identity case** (`0.14.0`) — `Scrivener`
      beside `Augur`, `Read` beside `Held`, and `spell::source` the one place a
      route to a `Program` asks what to compile. No model yet. Sabotage writes
      `Read` as well as `Held`; replay carries the reading in
      `Submission::Wrote`; fingerprinted per line. 156 captures, none moved
      **See it:** ✅ `scripts/dumps.sh /tmp/after && diff -r /tmp/before /tmp/after`
      — and `cargo test -p orbs-sim --test scrivener`

- [x] **`interpret` shows the reading** (`0.14.1`) — `Reading::was` carries the
      player's own line wherever a reader changed it, and the editor draws `≈` on
      that row — a glyph, not a colour (§14). It derives over the buffer, not the
      stored reading. `Copyist` is the fixed reader the captures pin
      **See it:** ✅ `ORBS_SCRIVENER=stub ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe loose" ORBS_EDIT=$'edit\nwork the sage down\nhang on ten ticks\n<esc>\ninterpret' cargo run -p orbs`
      — `≈ grind sage` and `≈ bide 10`; drop the switch and both stay as typed

- [x] **The corpus** (`0.14.2`) — `content/spellings.toml`, **11 templates
      expanding to 19,050 lines**, plus 119 that must come back untouched. The
      same format and loader as the prompt's; control flow only, since command
      lines are the prompt's vocabulary. It cannot be grown by adding nouns.
      `spell::reads_cleanly` came out of writing it
      **See it:** ✅ `cargo run -p orbs-sim --example parse -- --spells`
      — 119 of 119 sound, and **941 lines that parse into the wrong meaning**,
      which is the class a re-parse can never catch

- [x] **The model** (`0.14.3`) — a second reader, sharing the encoder, the
      vocabulary, the batching and the trainer, with its own head. A class is a
      template, not a spell word; a count must be present in the line; a name is
      the player's own word; a span is numbered by the canonical, not the
      phrasing (§19)
      **See it:** ✅ `cargo run --release -p orbs-augury --example train --features train -- --spells`
      then `cargo run --release -p orbs-augury --example measure --features train -- --spells`

- [x] **Live wiring** (`0.14.4`) — both frontends hold both readers under one
      driver setting; `ORBS_SCRIVENER` takes `ORBS_AUGURY`'s values. Both editors
      read on open and on the settle beat, where only the dump had passed a
      scrivener before
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe loose" ORBS_EDIT=$'edit\nwork the sage down\nhang on ten ticks\ndo that three times\n<esc>\ninterpret' cargo run -p orbs`
      — `≈ grind sage`, `≈ bide 10`, `≈ repeat 3`; `ORBS_SCRIVENER=off` leaves all three as typed

- [x] **The trials** (`0.14.5`) — `content/spell_trials.toml`: **300 lines and 8 whole
      spells the scrivener was never taught**, and `examples/trials.rs`, a report
      by shape and style. **Read as its opposite: 0**, and a test holds it there.
      The holdout was the wrong thing to tune against; the tests that need no
      model keep the trials honest. A review folded in after it (§19, *The
      review*)
      **See it:** ✅ `cargo run --release -p orbs-augury --example trials`
      — and `cargo test -p orbs-augury --test trials`

---

## The augury

**Exit:** the orb acts on a sentence it has never been able to read, echoes what
it heard, and no phrasing that resolved before resolves differently.

**§16 rates *"parser feels frustrating rather than magical"* Critical.** A flat
synonym list cannot hold a reordering: `grind sage` resolves and `turn the sage
into powder` cannot.

**DESIGN.md §6's *"No LLM, local or remote"* is retired** (§19, *the augury*).
The deterministic pipeline answers first and wins whenever it has an answer, so
the augury cannot regress a phrasing that works today.

> **Nothing is pulled from the internet.** No pretrained weights, embeddings or
> tokenizer, and no other model's outputs in the weights (§19). The model is
> trained here from the game's own content, which is why it stays small and why
> the corpus is the largest item below.

- [x] **The seam, and a reader that needs no GPU** (`0.13.0`) — three tiers
      (`parser::is_literal`, `Analysis::reads_outright`, `Sim::submit_reading`),
      `Confidence::Divined`, `Submission::Divined`, the `Augur` trait with
      `Fixture`, and `ORBS_AUGURY`. Tier one asks whether the parser explained
      the whole line, through `Candidate::leftover`. `Fixture` stays for the
      instruments that hold no weights (§19)
      **See it:** ✅ `ORBS_AUGURY=stub ORBS_BOOT=0 ORBS_DUMP="attend laboratory; turn the sage into powder" cargo run -p orbs`
      answers `≈ grind sage` and moves the sage into the mortar; drop
      `ORBS_AUGURY` and the same line is `!` exactly as before. `dumps.sh` gains
      four captures, and `augury_typed` is **byte-identical** to the same dump
      with no reader — the additive claim as a diff rather than an argument

- [x] **The corpus, and the first number anyone has had** (`0.13.1`) —
      `content/phrasings.toml`, `content::Phrasings` expanding templates over a
      `Scene`, and `--bench`. Today's parser read 15.6% of the holdout. A
      coverage lint, not §15's gate. `say` teaches, `holdout` is taught to
      nothing and is all that is reported; slot spans fall out of generation
      **See it:** ✅ `cargo run -p orbs-sim --example parse -- --bench` — two
      rates, and every miss printed under the command it should have reached

- [x] **The grammar baseline, and it is a reader rather than a yardstick**
      (`0.13.2`) — `augur::Grammar` from the `say` templates and
      `ORBS_AUGURY=grammar`. It and the matcher overlapped on nothing, 15.6% +
      9.4% = 25.0%. A reader never sees the world: it answers text and
      `parser::resolve` binds it
      **See it:** ✅ `cargo run -p orbs-sim --example parse -- --bench`, and
      `ORBS_AUGURY=grammar ORBS_BOOT=0 ORBS_DUMP="attend laboratory; work the sage down" cargo run -p orbs`
      answers `≈ grind sage`

- [x] **Span binding — superseded, not built** (`0.13.18`) — `Augur::read`
      returns canonical text, not byte offsets, so the alignment problem moved
      inside `orbs-augury`
      **See it:** the seam's signature — one line, `Vec<String>`

- [x] **The GPU question, settled before anything was built on it** (`0.13.3`) —
      `crates/orbs-augury` with `burn 0.21`. The graphics stack unifies, `tokio`
      never compiles, and a device shares. The trap: a device built with default
      limits computes zeros silently, so adopting one is checked with a known
      matmul (§19)
      **See it:** ✅ `cargo run -p orbs-augury --example device --features train` computes on the
      GPU twice and then prints the trap;
      `cargo tree -d | grep -E '^(wgpu|naga)'` prints nothing

- [x] **The vocabulary, and the sizing it settles** (`0.13.4`) —
      `orbs_augury::Vocabulary`, assembled from the game's own tables — the
      provenance rule as code. Sorted and pinned; unknown words hashed by
      character trigram
      **See it:** ✅ `cargo test -p orbs-augury the_game_speaks -- --nocapture`
      prints `known 318 + 64 buckets = 382 rows`

- [x] **The corpus, grown — and today's parser reads 1.6%** (`0.13.5`) — 14
      templates to 30. The 15.6% was an artefact of a small holdout. The
      provenance rule was corrected: it protects the artefact, not who typed a
      phrasing (§19)
      **See it:** ✅ `cargo run -p orbs-sim --example parse -- --bench`

- [x] **A reader answers with candidates, and the room decides** (`0.13.6`) —
      `Augur::read` returns up to four readings best-first, and `submit_reading`
      takes the first that resolves, failing that the first that gives a
      deliberate answer
      **See it:** ✅ `cargo run -p orbs-sim --example parse -- --bench`

- [x] **The corpus expands over the content, and the model is unblocked**
      (`0.13.7`) — `content::corpus_scene` expands over every reagent the game
      names, not the one on the shelf, and the grammar keeps function words:
      11,243 corpus examples, 99.1% on its own corpus
      **See it:** ✅ `cargo run -p orbs-sim --example parse -- --bench`

- [x] **What the reader is asked to predict** (`0.13.8`) — `augury::Sample`:
      `<cls>` plus the sentence, one verb class plus a reject, and a BIO tag per
      word by slot position, not noun kind. `REJECT` is a class, not a threshold
      **See it:** ✅ `cargo test -p orbs-augury the_whole_corpus_encodes -- --nocapture`

- [x] **The model, trained, and it reads 44.1%** (`0.13.9`) — `Reader` (343,862
      parameters), `Batch`, `examples/train.rs`, `Trained` implementing `Augur`,
      and `examples/measure.rs` — nine times the matcher on the same holdout.
      More templates raise the verb rate; more nouns do not. The trainer keeps
      the best holdout, and `burn-train` is deliberately absent (§19)
      **See it:** ✅ `cargo run --release -p orbs-augury --example train --features train`, then
      `cargo run --release -p orbs-augury --example measure --features train`

- [x] **The reject class, and enough negatives to mean it** (`0.13.10`) —
      `[[refusal]]` in `phrasings.toml`, 682 negatives from content; 94% refused
      correctly, and reading improved with it. `measure` reports the two
      populations apart
      **See it:** ✅ `cargo run --release -p orbs-augury --example measure --features train`

- [x] **The reader reaches a player** (`0.13.11`) — `ORBS_AUGURY=model`,
      `orbs-augury` behind an optional feature on `orbs-shell`. Inference runs on
      the CPU — the GPU is eight times slower for a batch of one — which retired
      the worker thread, the deadline and the pondering indicator. `model` is
      not in `dumps.sh`, since weights are gitignored (§19)
      **See it:** ✅ `ORBS_AUGURY=model ORBS_BOOT=0 ORBS_DUMP="attend laboratory; smash the sage" cargo run -p orbs`
      answers `≈ grind sage`; `let me read the feed.log` is refused by tier one
      as a spell word and never reaches the reader

- [x] **Five times the phrasings, and the verb rate follows** (`0.13.12`) —
      234 say-lines to 1,009: verb classification 54.5% → 72.3%, as `0.13.9`
      predicted. The register went archaic on purpose — `triturate`, `cohobate`,
      `lixiviate` (§19)
      **See it:** ✅ `ORBS_AUGURY=model ORBS_BOOT=0 ORBS_DUMP="attend laboratory; triturate the sage" cargo run -p orbs`

- [x] **Slot-to-verb conditioning, and the instrument fault it exposed**
      (`0.13.13`) — the verb head reads a per-tag pooled summary alongside
      `<cls>`, the checkpoint criterion weighs refusals as well as verbs, and
      **training is seeded** — unseeded runs carried ±12 points of variance,
      wider than any effect measured, so the architecture change cannot be
      credited. Verbs and refusals traded between adjacent passes (§19)
      **See it:** ✅ `cargo run --release -p orbs-augury --example train --features train -- --epochs 4`
      twice — the two runs print identical figures

- [x] **A separate head for "is this a command at all" — and the diagnosis that
      was wrong** (`0.13.14`) — `Reader::command`, a two-output head off the same
      pooled encoding. The shared softmax was never the mechanism; the 34:1 class
      balance was, and weighting the command loss fixed it. The head was kept on
      design grounds, not its measured effect (§19)
      **See it:** ✅ `ORBS_AUGURY=model ORBS_BOOT=0 ORBS_DUMP="attend laboratory; what should i do next; triturate the sage" cargo run -p orbs`
      — the question falls through to the deterministic path, the command does not

- [x] **The seventeen verbs nobody had written down, and the corpus that was
      47% one verb** (`0.13.15`) — templates for every verb in `Verb::ALL`, every
      existing entry doubled, `Phrasings::corpus_capped`, both losses weighted.
      Seventeen verbs had no templates, and the corpus was 47% `move` because an
      expansion is the product of its slots. Three tests hold both now
      **See it:** ✅ `ORBS_AUGURY=model ORBS_BOOT=0 ORBS_DUMP="attend laboratory; send the wolves to the gate; triturate the sage" cargo run -p orbs`
      — the first was a confident `move gate`, and is now `haul` answering *"there
      is no wellspring here"*

- [x] **The reader a player actually gets** (`0.13.16`) — `ORBS_AUGURY` unset
      means the trained reader. A clone with no weights degrades silently;
      `SimPlugin` installs no reader under `cargo test`; `scripts/dumps.sh` passes
      `off`, so the captures do not show the shipping default — `--example
      measure` gates the reader instead
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; triturate the sage" cargo run -p orbs`
      — no switch set, and the mortar turns

- [x] **The player chooses the driver, and the terminal gets the whole game**
      (`0.13.17`) — the driver is a menu word, `augury` or `plain`, remembered in
      `orbs-settings.toml` beside the saves, never in one. `wgpu` went behind a
      `train` feature, so `orbs-tui` holds the same reader and the gate asks for
      `--features orbs-augury/train`
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU="options" cargo run -p orbs`
      — and in the terminal, `menu`, `options`, `plain`, `resume`, then type a
      sentence: it falls through to the matcher

- [x] **A verb whose argument explained nothing asks, rather than running**
      (`0.13.18`) — `verify gibberish` ran a whole-tower audit with the word
      discarded. It is `Resolution::Incomplete` now; naming the instrument you
      operate is exempt (§19)
      **See it:** ✅ `ORBS_AUGURY=off ORBS_BOOT=0 ORBS_DUMP="attend laboratory; verify gibberish; move charcoal to athanor; light athanor; survey" cargo run -p orbs`
      — the first asks, the second lights, the third lists

- [x] **A reading that throws away what the tagger found** (`0.13.19`) — the
      reader no longer offers a verb that cannot hold the spans it located, where
      a bare `undo` had outranked `digest husks`
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; digest husks" cargo run -p orbs`
      — *"there is no husks within reach"*, not `undo`

- [x] **A verb that takes no argument no longer swallows one** (`0.14.6`) —
      `status gibberish` ran `status` and `undo gibberish` acknowledged, the word
      thrown away. It is `Resolution::TakesNothing` now, `Incomplete`'s other
      half: the orb names the verb and the words it could not use, a reader sees
      the line first, and a place (`wander archive`) or filler (`status please`)
      is not a stray word
      **See it:** ✅ `ORBS_AUGURY=off ORBS_BOOT=0 ORBS_DUMP="status gibberish" cargo run -p orbs`
      — *"status takes nothing - 'gibberish' is not something it can use"*

- [x] **The reading that uses every word runs first** (`0.14.7`) — and the
      class was **24** corpus lines, not the 3,146 this box claimed. The bench
      sorts its misreads by cause now: 3,119 were a wrong reading ahead of the
      right one in the reader's own order, and the box's example, `survey
      feed.log`, had been `Incomplete` since `0.13.18`. `parser::reading_to_run`
      asks a reader's readings what `reads_outright` asks a typed line, and
      `Sim::submit_reading` and the bench both choose with it. With `0.14.6`
      beside it, 3,146 misreads became 2,822
      **See it:** ✅ `cargo run --release -p orbs-sim --example parse -- --bench` —
      *"the one that ran left words it could not use"* 24 → 12

- [x] **The model — shipped, and one plank of it dropped** (`0.13.18`).
      `orbs-augury`, trained on the GPU, inferring on `ndarray` at 436µs. The
      consolidated action set was never built; `0.13.15` spent that lever on
      templates instead
      **See it:** ✅ `cargo run --release -p orbs-augury --example measure --features train`

- [x] **Settings in the menu** — done at `0.13.17`, under the entry above: the
      driver, not the backend
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="menu" ORBS_MENU="options" cargo run -p orbs`

- [x] **Live wiring — one quarter built, three quarters retired by measurement**
      (`0.13.18`). The reader is on by default in both frontends; at 436µs a line
      there is no slow state to paint. Boot warm-up was measured separately, below
      **See it:** ✅ `cargo run -p orbs`, then type `turn the sage into powder`

- [x] **Boot warm-up — measured, and not needed** (`0.14.8`). Both readers load
      in 8.4ms warm in release and 21ms in dev, so nothing moves behind the POST
      card
      **See it:** ✅ `cargo test --release -p orbs-augury --test loading -- --ignored --nocapture`
      *(drop `--release` for the dev profile, which is what `cargo run -p orbs` builds)*

- [x] **An exact word before a near one, and each sentence taught once, to one
      command** (`0.14.9`) — three corpus defects: a near word claiming another
      template's word, 154 sentences taught as two commands, and every reagent
      sentence made twice. Misreads 2,822 → 65
      **See it:** ✅ `cargo run --release -p orbs-sim --example parse -- --bench` —
      *"0 sentences are taught as two commands"*, and under the outranked cause,
      which command stood in front of which; in the game,
      `ORBS_AUGURY=grammar ORBS_BOOT=0 ORBS_DUMP="attend laboratory; still the sage" cargo run -p orbs`
      — *"≈ distil sage"*, which was grinding it

- [x] **Several seeds, so a retrain can be told from its luck** (`0.14.10`) —
      `scripts/seeds.sh` trains both readers over five seeds and `--compare` says
      whether a change clears the baseline's range. One unchanged tree ranged
      47.1%–77.0% across seeds, wider than every single-run comparison before it
      **See it:** ✅ `scripts/seeds.sh baseline`, then `cat target/seeds/baseline/summary.txt`
      — half an hour on the GPU; after a change, `scripts/seeds.sh <label>` and
      `scripts/seeds.sh --compare baseline <label>`

- [x] **The reader's table holds what is taught, and both readers are
      retrained** (`0.14.11`) — two of `0.14.5`'s three table changes, and a
      retrained pair chosen by a rule fixed before the run was read
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; triturate the sage" cargo run -p orbs`
      — the shipping default reads it again; the numbers are
      `cargo run --release -p orbs-augury --example measure --features train`
      (and `-- --spells`) and `cargo run --release -p orbs-augury --example trials`

- [ ] **Refusals the spell reader is shown** — the third table change. The
      spell file's refusals are canonical statements the gate keeps from the
      reader, and they are the population `measure --spells` holds to 100%
      untouched, so the sentences about the tower a reader *would* meet need a
      population of their own. `0.14.5` replaced the one with the other in a
      single run and could not tell what it measured
      **See it:** `scripts/seeds.sh refusals --spells <label>`, then
      `scripts/seeds.sh --compare <label> refusals` — the *nothing* and
      *hazard* trial styles, and *read as their opposite* at 0

- [ ] **A `{count}` in the spell corpus** — `repeat 3` only ever says *three*
      and `bide 10` only *ten*, so *"run it seven times"* is a shape the head has
      never met. The count is read from the line, so it is the class that fails;
      a slot has to carry a number word in the sentence and its digit in the
      canonical, and teach the tagger nothing
      **See it:** `cargo run --release -p orbs-augury --example trials` — the
      *repeat*, *bide* and *no-count* rows, over seeds

---

## The threshold — **closed at `0.16.13`**

Eleven boxes, and three more for what two `/code-review max` passes found — the
second pass's findings include three defects the first round's fixes introduced.

**Exit:** the orb wakes to a menu rather than a tower, and every tower a player
plays is one they chose — a new game at a length they picked, or one of the six
on disk. Settings and the manual are reachable before any of it. **Met.**

What went differently from the plan is in §19: the manual's depth is a generated
chapter, the new surfaces went to `scripts/dumps.sh`, and the licence table had
been wrong about copyleft. Four things were found by looking rather than testing.

> **One menu, two stances — not two menus.** `Stance::{Threshold, InTower}` is a
> field on the screen that already exists. The stance gates the way out: at the
> threshold the menu cannot be closed.

- [x] **The threshold exists, and it is a closed room** (`0.16.0`) —
      `Threshold::{Waiting, Playing}` shared by both frontends, `ORBS_THRESHOLD`,
      and a `playing` gate over the clock, the autosave and the way out. No save
      is read, so nothing unplayed can be written. `Outcome::Begin` carries an
      optional path, and `Session::answered` runs once per pass — found by
      playing it. A dump is blind to all of it; the gate is
      `routing::the_threshold_reaches_a_tower`
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 cargo run -p orbs`
      — and `ORBS_MENU="resume"` for the refusal, `ORBS_MENU="new"` for the
      lengths reached with saving off; then `scripts/play.sh routing::the_threshold`
      for the half a picture cannot show
- [x] **One menu, two stances** (`0.16.1`) — `resume` · `play` · `settings` ·
      `quit`, with `resume` off the threshold's copy. `Page::Play` numbers the
      towers with `new` under them; `options` became `settings`. The split into
      `pages.rs` came first, checked by `diff -r`
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="play" cargo run -p orbs`
      — with `ORBS_SAVE` pointing at a directory of towers they are numbered with
      `new` beneath, so a returning player types `play`, `1`
- [x] **A tower can be abandoned, and a broken one says so** (`0.16.2`) —
      `abandon <slot>` asks once, and any other line answers no. The only word in
      the menu not prefix-matched; the file is renamed, never unlinked. An
      unreadable slot is listed as what it is and refused by name
      **See it:** ✅ `ORBS_SAVE=<dir>/orbs-save.toml ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU='play\nabandon 2' cargo run -p orbs`
      asks; `'play\nabandon 2\nabandon 2'` does it and the file is
      `orbs-save-2.toml.abandoned`; with a corrupt file in a slot, `'play\n3'`
      says the orb cannot read it and leaves the file alone
- [x] **Saves and settings move to the OS app-data directory** (`0.16.3`) —
      `<app-data>/orbs/`, resolved `ORBS_SAVE` → app-data → working directory,
      since a Steam install directory can be read-only (§13). `create_dir_all`
      before the first write; a marker file, not per-slot copy-if-absent;
      `chosen` takes the directory as a parameter so no test reaches real
      towers. Copied, never moved
      **See it:** ✅ with a sandboxed `HOME` and towers in the working directory,
      one launch puts them in `~/.local/share/orbs/` with `.migrated` beside
      them, leaves the originals untouched, and a second launch resurrects
      nothing
- [x] **The settings store** (`0.16.4`) — `settings/` split into `values` (what a
      page is made of), `store` (the file) and the typed readers. Each row says
      what it is set to; a word steps a setting and a word with a value names
      one. A frontend builds the rows and the menu draws them, so a build
      without a tube offers no `tube` page. The file is a map of words, keeping
      keys it does not know

      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU='settings\ntube\ncrt off' cargo run -p orbs`
      — and the same in `orbs-tui`, which offers two pages rather than three
- [x] **The frontends obey** (`0.16.5`) — the tube, the phosphor, the
      accommodation, the focus mode, the linear stream and §4's sticky boot skip
      seed from the store and write back through their keys. The chain is `env →
      stored → default`, so the instruments' switches outrank a preference.
      Reduce-motion was left for later (§19). Nothing is kept until a frontend
      opts in, so a test never writes a real settings file
      **See it:** ✅ set `skip on` in `settings`, `habits`; relaunch **without**
      `ORBS_BOOT=0` and the boot sequence does not play — with a fresh profile it
      still does
- [x] **The manual reader** (`0.16.6`) — a sixth `Focus`, a contents page, a
      chapter wrapped to the pane and scrolled within it, `Key::PageUp`/
      `PageDown`, and `ORBS_MANUAL`, on the menu's top page in both stances. The
      first surface to want a key another owns, so both builds guard the
      transcript's paging. `content/manual.toml` is its own file because a manual
      is paragraphs the reader wraps
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" ORBS_MANUAL="orbs" cargo run -p orbs`
      — and at `ORBS_GRID=80x22` a chapter says *pgdn for more*; then
      `scripts/play.sh routing::the_manual_pages` for the paging a capture
      cannot take a measurement for
- [x] **The generated chapters** (`0.16.7`) — *every command there is* and *every
      word a spell is written with*, assembled from existing keys, with no new
      prose. Indentation does not survive wrapping, so hierarchy is columns
      measured from the prose
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" ORBS_MANUAL="commands" cargo run -p orbs`
      — every verb in the game, in `help`'s own groups, columns aligned; and
      `ORBS_MANUAL="language"` for the twelve words and their worked examples
- [x] **The authored chapters** (`0.16.8`) — `manual.toml`, its own `ORBS_CONTENT` leg
      (rule 6 is not satisfied by `include_str!`), wrap-aware lints, and the
      licences chapter. Sixteen authored chapters and a third generated one,
      `inside`, from the room primers — nineteen in all
      **See it:** `ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU="manual" cargo run -p orbs`
      — the contents page at the floor; `ORBS_MANUAL="entering"`, `"yours"`,
      `"upkeep"`, `"inside"` for the new ones. The reload leg:
      `ORBS_CONTENT=crates/orbs-sim/content cargo run -p orbs`, then edit a line
      of `manual.toml` while it runs and watch the log say `reloaded manual.toml`
      **once** — see §19 on the watcher that used to say it every tick
- [x] **A voice for the orb** (`0.16.9`) — synthesised cues chosen from a
      record's `kind`/`role`/`outcome`, rule 2 as a signature. (The hum it
      shipped with was removed at `0.17.18`)
      **See it:** `ORBS_SOUND=1 cargo run -p orbs 2>&1 | grep '^sound:'` — every
      cue named as it plays, which is the half that works with no speakers; then
      type, brew, and hear the difference between a completion and a refusal
      with the screen off
- [x] **The sound page, and audio as accommodation** (`0.16.10`) — volumes persisted, and
      §14's distinct completion / warning / sabotage tones. Volumes are words,
      not numbers; `wrong` is the sabotage tone. The page arrived with the thing
      it controls
      **See it:** `ORBS_BOOT=0 ORBS_DUMP=1 ORBS_THRESHOLD=1 ORBS_SAVE=off ORBS_MENU='settings\nsound' cargo run -p orbs`
      — the page; add `\nvoice quiet\nhum off` to see both forms (cycle, and name
      a value). Then put `hum = "off"` in `orbs-settings.toml` beside your saves
      and `ORBS_SOUND=1 cargo run -p orbs 2>&1 | grep '^sound:'` — it comes back
      `hum (plain, off)`. By ear: `verify` a tampered surface and hear the beat
      against a completion's rise
- [x] **The review, and fifteen corrections** (`0.16.11`) — a `/code-review max`
      over the whole feature, with the gate green throughout. Two could lose a
      player's work: a dump could rename a real save, and `migrate()` tested slot
      1 alone. The worst four share one shape — an enum grew and a `_ =>` arm
      absorbed it (§19)
      **See it:** the two that matter, as reproductions —
      `XDG_DATA_HOME=/tmp/p ORBS_BOOT=0 ORBS_DUMP=1 ORBS_THRESHOLD=1 ORBS_MENU=play cargo run -p orbs`
      says *no towers yet* rather than listing a real profile's games, and
      `ORBS_MENU=$'play\nabandon 1\nabandon 1'` leaves every file where it was;
      then a working directory holding `orbs-save-2.toml` and no slot 1 migrates
      on the next launch
- [x] **The second review, and fifteen more** (`0.16.12`) — the same pass again
      over the corrected tree. Three were defects the first round's fixes
      introduced, among them an empty `ORBS_SAVE=` letting a dump overwrite a real
      tower again (§19)
      **See it:** `ORBS_SAVE= XDG_DATA_HOME=/tmp/p ORBS_BOOT=0 ORBS_DUMP="attend archive" cargo run -p orbs`
      leaves the profile's save byte-identical; `cargo test -p orbs manual`
      holds the two-surface keyboard; and
      `ORBS_BOOT=0 ORBS_DUMP=1 ORBS_THRESHOLD=1 ORBS_SAVE=off ORBS_MENU=$'play\nnew\nback' cargo run -p orbs`
      lands on the play page rather than the top one
- [x] **The review's remainder** (`0.16.13`) — the findings the second pass
      listed below its own cap: the manual re-wrapped every line every paint, now
      cached on `(chapter, width)`; the key click fired for modifiers and
      through the boot card; and the changelog block's version did not match the
      manifest, so a release would have published nothing
      **See it:** `cargo test -p orbs sound::keys` and
      `cargo test -p orbs-shell manual::state` hold the click and the cache;
      `ORBS_BOOT=0 ORBS_DUMP=1 ORBS_THRESHOLD=1 ORBS_SAVE=off ORBS_GRID=80x22 ORBS_MENU=manual ORBS_MANUAL=language cargo run -p orbs`
      draws the longest chapter at the floor, which is what used to be re-wrapped
      every frame

---

## The orb's menu

**Exit:** a player can leave a game, start another at a different length, and
come back to the first, unharmed.

**The curve was calibrated to one loop, not a tower** — 10,000 experience is
reachable by typing, which undercuts pillar 3. So **a game has a length, chosen
once when it begins.**

> **The head is anchored and the tail is stretched**, quadratically from nothing
> at a line's first station, so the first spell slot still costs one clarity.
> **The tiers are the curve, not the clock**: each is what the last station
> reads, and what it costs in hours is `orbs-balance`'s to measure (§19).

| Tier | Last ley station |
|---|---|
| Short | 30,000 |
| **Medium** (default) | **60,000** |
| Long | 250,000 |

- [x] **A game has a length** (`0.12.0`) — `Length`, the anchored quadratic ramp,
      and `Progression::stretched` applied before `check`. `earns` is never
      scaled, or the feature cancels silently; fifteen prose keys interpolate
      `{count}`
      **See it:** ✅ `ORBS_LENGTH=long ORBS_SEALED=1 ORBS_SAVE=off ORBS_DUMP="weave" cargo run -p orbs`
      reads 250,000 where baseline reads 10,000; and
      `ORBS_LENGTH=medium … ORBS_DUMP="debug_reach laboratory_1; attend laboratory"`
      says *"6 potions brewed"* where baseline says five
- [x] **The length threads the tower** (`0.12.1`) — the private constructor
      funnel, `Sim::measured` and `Sim::begun`, `WorldSave.length` and `FORMAT`
      10 → 11, `ORBS_LENGTH`, and `--length` on `orbs-balance`. `Sim::restored`
      would have loaded every save at ×1, and no lint could see it. The default
      splits like `ORBS_SEALED`: the game is medium, the dump baseline
      **See it:** ✅ `cargo run -p orbs-balance -- sweep --hours 1 --length long --why`,
      and a save round-tripped at a length reads the same curve back
- [x] **The weave reads six figures** (`0.12.2`) — at long the last station is
      250,000 and two totals ran together. A six-figure total abbreviates
      (`250k`), rounding up, and every other surface keeps it exact; the bar and
      the track share one `gauge()`
      **See it:** ✅ `ORBS_LENGTH=long ORBS_SEALED=1 ORBS_SAVE=off ORBS_BOOT=0 ORBS_DUMP="weave" cargo run -p orbs`
      — every total clear of its neighbour, `0 of 250k` in the gauge; the same
      line at `baseline` and `medium` is unchanged, cell for cell
- [x] **`menu` opens the screen, and `quit` asks before it leaves** (`0.12.3`,
      corrected at `0.12.7`) — `quit` opened the menu for one iteration, which
      was wrong (§19). `menu` is its own verb, and `quit` asks once — a word, not
      a screen — and any other command answers no. `Focus::Menu` is first in the
      ordering. `F10`, `Ctrl-C` and the close button leave outright
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="menu" cargo run -q -p orbs`, with
      `ORBS_MENU="resume"` and `ORBS_MENU="zorb"`; and
      `ORBS_DUMP="quit" ORBS_THEN="quit"` against
      `ORBS_THEN="status; quit"` — the second leaves, the third asks again. Seven
      latches in `scripts/dumps.sh`
- [x] **The menu does not eat the word that opened it** (`0.12.7`) — **a shipped
      defect, reported by a player.** A gated reader keeps its cursor, so the
      menu read the word that opened it. The fix is run always, decline, and
      clear, ordered before `open_requested`. A dump was blind to it; an in-`App`
      test is the gate
      **See it:** ✅ `cargo test -p orbs --bins opening_the_menu` — it fails on
      the old ordering and passes on the new
- [x] **More than one save** (`0.12.4`) — `read_from`/`write_to` beneath the
      current pair, a `saves()` listing and `free_slot()`. Slot 1 is
      `orbs-save.toml`; a listing can never rename what it lists; there is no
      process-global current path
      **See it:** ✅ `ORBS_SAVE=/tmp/t/orbs-save.toml ORBS_BOOT=0 ORBS_DUMP="quit" ORBS_MENU="saves" cargo run -q -p orbs`
      — the towers listed with wizard, length and experience
- [x] **Swapping the tower safely** (`0.12.5`) — `Kept` makes the path travel
      with the `Sim`, so autosave cannot write the tower being left into the
      file of the one being opened. The resources are one list, registered and
      reset from; the incoming save is read before anything is torn down
      **See it:** ✅ `cargo test -p orbs --bins menuing` — four tests, and
      `loading_a_second_tower_writes_the_first_to_its_own_file` is the exit
      criterion: load a second tower, and the first is still in its own file with
      its own seed. **That is exactly what autosave would have broken**
- [x] **A new game, and its length chosen** (`0.12.6`) — the three tiers, offered
      by prefix; `baseline` is not a difficulty. A full orb is refused before the
      length is asked
      **See it:** ✅ `ORBS_SAVE=/tmp/t/orbs-save.toml ORBS_BOOT=0 ORBS_DUMP="quit" ORBS_MENU="new" cargo run -q -p orbs`
      offers them, and `a_new_game_is_begun_at_the_length_that_was_chosen` reads
      the length back off the new world's save

---

## The tower as one machine

*Was **Phase 12** — a name, not a position.*

**Exit:** the player leaves, comes back, and the tower ran itself — *across*
rooms, not in one.

> **The web is what this phase is for** (§19). Nothing a player makes is ever
> what they run out of. **Pane addressing and slot reservation come first**, since
> a web routed by hand is a to-do list; potions and scrolls already cross through
> `keep::admits`.

- [x] **The calm layer stops poisoning logs, and a siege waits** (`0.17.0`) —
      §5.1 says the idle layer touches *"environmental only, never scripts,
      schedules, or logs"*, and `sabotage::drift` poisoned a log every ~300
      ticks. **The enemy acts only when a round resolves**, an accessibility
      commitment. `drift` is gone; a log is `assault::strike`'s third surface,
      and what a siege breaks stays broken until purged (`0.17.1`). Format 15
      (§19)
      **See it:** `ORBS_BOOT=0 ORBS_SEED=3 ORBS_DUMP="meditate 1800; verify; meditate 40" cargo run -p orbs`
      reads `sound` where it named all five logs. The same line with
      `attend bailey; defend; meditate 3600; survey rampart` reads turn 0, and
      `cargo test -p orbs-sim --test besieging siege` holds both halves
- [x] **`stillness` and `vigour` can be spent** (`0.17.1`). Both had **out-degree
      zero**. Stillness ends a siege with no penalty, so it is the dearest thing
      the laboratory makes and only a hand drinks it; vigour raises how much
      fight the garrison holds. The tower stays broken after a siege until
      purged (§19)
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; hold; hold; debug_spawn stillness 3; quaff stillness; survey rampart" cargo run -p orbs`
      reads `stilled` and the rampart `lifted`. `…; debug_spawn vigour 6; quaff vigour; survey garrison`
      grows the garrison bar from 13/18 to 13/21. Both spawn the slowest rate's
      threshold since *Supply* (`0.18.3`): one does nothing. And
      `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; debug_spawn quiet-draught 1; debug_spawn gleaning-scroll 1; move quiet-draught to flask_and_rod; move gleaning-scroll to flask_and_rod; wield flask_and_rod; meditate 22; recall stillness" cargo run -p orbs`
      makes the draught and reads the 600-tick, 20-quintessence route.
      `every_potion_the_alembic_makes_can_be_spent` keeps it from recurring
- [ ] Pane addressing — acting on a domain you are not standing in. **First, not
      fourth** (§19)
      **See it:** start a grind from the archive
- [ ] **Focus-slot reservation**, which is the *same* roadmap item as pane
      addressing and moves with it. `CAPACITY = 1` is tower-wide, so this phase's
      exit is unreachable by construction until it rises
      **See it:** two rooms working at once, and the sidebar accounting for both
- [ ] **A made good crosses and another room consumes it.** No new `NounKind`, no
      content file and no save migration — a `warding` essence already reaches the
      sanctum through the arsenal; what is new is the `pylon` drawing a warding
      **supply** (*Supply*, `0.18.3`: never one potion) while a ward holds, so a
      ward set **holds** after the course ends for as long as it is fed
      **See it:** a warding supply open on the sanctum, and a ward that outlives
      its course until the supply runs dry
- [x] **The player can see a bottleneck** (done under *the tower speaks up*,
      `0.21.0`–`0.21.1`). No surface in the game shows a rate or
      a blocker — but the limiting factor is **already computed and discarded**:
      `Blocked::{Instrument, Slot}` carries the blocker's name and state and is
      emitted once as a log line, and `State::Gathering` already knows its missing
      input. One defect rides with it: `brief::busiest` is
      `.find(is_busy).or_else(first)` over `Working | Scouring | Burning`, so a
      laboratory with a fouled alembic and a cold athanor draws as
      `laboratory / idle` — **the rail is most silent exactly when a room is
      stalled**
      **See it:** `status rates` — every room's goods ~~a tick~~ an hour, and a
      stalled room names what it is waiting on. *(Bare, not `--rates`: the
      parser has no flag syntax, which already forced `verify --all` to be
      respelled.)* The lines that work are *the tower speaks up*'s first two
      boxes'
- [ ] **One good, two consumers, three answers — the phase's real exit.** A
      `warding` potion is worth a siege round *or* a ward that holds, never both,
      and the shortage is answerable three ways: brew more (spend the slot), a
      `fruitful` alembic (spend quintessence), or a fuller standing garrison
      (spend holds at the circle — *Supply* moved troops off the shelf, `0.18.1`).
      Quintessence spent on the charm is quintessence not pledged
      to dice, so **the new bottleneck plugs into the one the game already has**
      rather than standing beside it
      **See it:** `orbs-balance` reads a policy that routes potions by which of
      the two is lower
- [ ] **The edges between domains, authored deliberately** — laboratory → lens,
      archive → grimoire, forge → laboratory, menagerie → sanctum
      **See it:** `recall` a domain and read what feeds it and what it feeds
- [ ] **The web and §9's pane synergies are reconciled** — a §19 entry, not code.
      §9 authors standing cross-domain effects from co-presence and this phase
      defers all 21 pairs; *"forge beside the sanctum auto-repairs wards"* is an
      edge the web also draws. **A pair must not be paid twice**
      **See it:** the §19 entry names which mechanism owns which pair
- [ ] **A domain's floor is still free** — the reachability test. Rate is the
      wrong instrument: only nine of eleven policies pin, `TOLERANCE` is 10%,
      `drive::run` uses the *open* tower, and a Mastery station that comes to
      require a bought ceiling gates progression while every rate holds still
      **See it:** `cargo test -p orbs-sim --test floors` — every Mastery line
      walked to its last station with no good ever crossing a room
- [ ] One spell that runs the whole tower
      **See it:** bind it, walk away for an hour, come back to work done in four
      rooms

**Scarcity: the slot, finally contested.** Every domain has been spending one
slot alone; this is the phase where they compete for it, which is what §5.0 means
by *"the economy and the focus system are the same system"*.

> **Reagent crossing is deliberately *not* here.** Two drafts made `ground-salt`
> the contested good and this phase's exit; `rock-salt` is `Holding::endless`, so
> the shortage cannot exist, and the relief route costs two more grinds than it
> saves. It returns as texture once made goods have proved the shape, and its rule
> is **derived** — a good is a material another domain's recipe consumes — never a
> list of names in a file, which is the judgement `keep::admits` refuses (§19).

---

## Supply

**No produced item has individual value.** A potion, scroll or demon does
something only while the arsenal holds enough of it, and while it does, it
drains. Each authors rates — a threshold, a ration, a benefit — and the item's
verb opens or raises one. Draining steps a supply down; freshness caps it. The
garrison stops being an arsenal item and becomes the tower's, refilled by the
menagerie, which can bind demons instead. §19 *Supply* has the rules.

**`0.18`, every box ticked at `0.18.7`.** The one-machine boxes above resume
under whatever minor is in hand. **The readers were retrained in the box that
changes their vocabulary**, `0.18.4`; `weights_on_disk_are_this_vocabularys`
fails the gate whenever they are stale.

- [x] **A run yields a batch, and the tower still counts runs** (`0.18.0`) —
      `yields` on a recipe; a distillation makes three. Experience, renown, the
      tally and the stores still count the run, so every pinned rate holds.
      Fruitful adds one to the batch. The laboratory's ledger and mastery lines
      say *brews*, since one brew is no longer one potion. §19
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; debug_spawn clarified-draught; distil clarified-draught; meditate 60; survey alembic; status" cargo run -p orbs`
      reads `the alembic yields 3 clarity`, `clarity = 3` and
      `brews finished 1`. `cargo test -p orbs-sim --test batches`
- [x] **A standing garrison, and `deploy troop` retired** (`0.18.1`) — a most
      of ten nothing passes, a floor of six the bailey keeps without a
      menagerie, and desertion between, paused in a siege. `defend` levies all
      of it; survivors come home up to the levy, and refills past it disband.
      A bare `summon` refills it, and a hold mid-siege waits in the reserve.
      Read off the menagerie's `roster` and in `status`. Format 17 turns held
      troops into bodies. §19
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; debug_circle; summon; survey roster; attend bailey; defend; survey garrison" cargo run -p orbs`
      reads `4 join the garrison`, the roster `full` and `spears = 10`, and the
      garrison `spears = 10`. `ORBS_SEALED=1 ORBS_BOOT=0 ORBS_DUMP="debug_reach sanctum_1; status" cargo run -p orbs`
      reads `garrison 6` and `garrison most 10`.
      `cargo test -p orbs-sim --test garrison`
- [x] **Freshness renamed, and `most` retired** (`0.18.2`) — `Supply` is
      `Freshness`. Stillness brews three a run (two until a second review),
      may be brewed while some is held, and goes stale like anything else. §19
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; debug_spawn stillness 1; debug_spawn stilling-draught; distil stilling-draught; meditate 600; survey alembic" cargo run -p orbs`
      reads `the alembic yields 3 stillness` where it read *"one stillness is
      all the tower keeps"*. `cargo test -p orbs-sim --test stilling`
- [x] **Items are standing supplies in the siege** (`0.18.3`) — a row's rates
      are checked at load; the verb opens a supply and raises it; each round it
      draws, slows as it drains, and runs dry. Thin runs only the slowest rate and
      nothing halves; a siege reads freshness as at `defend`. Fortify is a level.
      `stop` closes one. The board's `drawing` row, and `draw`/`drained` for a
      spell. Format 18. §19
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; debug_spawn warding 5; quaff warding; debug_spawn warding 10; quaff warding; quaff warding; hold; survey arsenal; stop warding; peruse bailey.log" cargo run -p orbs`
      reads `you hold 5 warding. it flows from 6`, `the warding flows, 2 a
      round`, `flows faster, 3 a round`, the draws in the log, `qty: 9`, and
      `you stop the warding`.
      `cargo test -p orbs-sim --test supply`
- [x] **The circle's second function: demons** (`0.18.4`) — `summon <demon>`
      binds a hold into the arsenal: four within par, three past it. Golem
      refills the line, familiar heals, effigy fortifies; `deploy <demon>` opens
      one. A lesser circle binds bodies only and is refused before the call.
      Format 19. The readers are retrained here. §19
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; debug_circle; summon golem; summon; debug_circle; summon golem; survey arsenal; attend bailey; defend; deploy golem; survey arsenal" cargo run -p orbs`
      reads `4 golem bound to the arsenal`, `golem = 8`, `the golem flows, 2 a
      round` and `golem = 6`. `ORBS_AUGURY=model ORBS_BOOT=0 ORBS_DUMP="attend menagerie; I want the next beast as a golem; turn this one into an effigy" cargo run -p orbs`
      reads `summon golem` and `summon effigy`, phrasings the reader never saw;
      `ORBS_AUGURY=model ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; debug_spawn warding 6; quaff warding; enough of the warding" cargo run -p orbs`
      reads `stop warding`. `cargo test -p orbs-sim --test demons`
- [x] **The scrolls measured first** (`0.18.5`) — `orbs-balance` learns
      `quickened` and `gleaned`, against today's one-shot scrolls and a lectern
      at one. Quickening pays 33 experience a scroll; gleaning loses at a
      lectern of one. §19
      **See it:** `cargo run -p orbs-balance -- run quickened --ticks 7200 --why`
      reads about 0.249 against clarity's 0.140 and `quickening-scroll: 24
      given, 24 spent`; `run gleaned` reads `fragment: 49 held`.
      `cargo test -p orbs-balance --test agrees quickening`
- [x] **Quickening and gleaning as calm supplies** (`0.18.6`) — `wield` opens
      one from four held and draws at once, then every 300 ticks while open,
      used or not; the word again raises it. A window of 301 ticks quickens the
      laboratory, by a half or a third. While gleaning is open the stacks
      gather. Paused in a siege, which takes `wield` while it runs. The lectern
      yields two. §19
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_spawn quickening-scroll 8; wield quickening-scroll; kindle charcoal; debug_spawn clarified-draught; distil clarified-draught; meditate 29; meditate 2400; survey arsenal" cargo run -p orbs`
      reads `unrolls, 1 every 300 ticks`, `works 2 times quick`, a 56-tick
      distillation landing inside 29, `runs dry and rolls itself up`, and
      `qty: 3`. `ORBS_BOOT=0 ORBS_DUMP="attend archive; debug_spawn gleaning-scroll 4; wield gleaning-scroll; research; stop gleaning-scroll" cargo run -p orbs`
      reads `the next stacks you open will gather 4`, then `the stacks gather
      now`. `cargo test -p orbs-sim --test calm --test gleaning`
- [x] **Verdant is cultivation** (`0.18.7`) — each draw grows the next herb;
      at three it flowers for good, endless like sage. Growth is kept when the
      supply runs dry and travels in the save; it rolls itself up once every
      herb has flowered. §19
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_spawn verdant-scroll 6; wield verdant-scroll; meditate 900; survey dispensary" cargo run -p orbs`
      reads `amber flowers on the shelf, and stays`, then `runs dry`, and
      `amber` on the shelf, endless. `cargo test -p orbs-sim --test gleaning verdant`
- [x] ~~**The readers, retrained** (`0.18.8`)~~ — **folded into `0.18.4`**, the
      one box that changes the vocabulary: the demons' names, `deploy
      {essence}`, `stop {essence}` and binding at the circle. A gate cannot
      be green with the weights on disk stale, and `0.18.6`–`0.18.7` add no
      word a reader is taught. §19

---

## The terminal kept true

**A feature is not shipped until the terminal build plays it.** `0.12`–`0.18`
were built against the window and mostly reached `orbs-tui` through the shell;
an audit found what did not. §19 *The terminal kept true* has the findings and
the order.

**`0.19`, every box ticked at `0.19.13`.** One patch per box. `0.17` stays
paused, and resumes under whatever minor is in hand.

- [x] **`drive.rs` split by concern; nothing changes** (`0.19.0`) — the loop
      alone in `drive.rs`; `carried`, `hangup`, `keymap`, `session/`, and
      `surfaces/` and `blit/` as folders. The session keeps one `Carried`. A dump
      never runs this code, so the proof is the played suite and a byte-for-byte
      snapshot of the wire. §19
      **See it:** `scripts/play.sh` — 149 passed, as before the split;
      `cargo test -p orbs-tui --bins a_frame_reaches_the_wire`;
      `wc -l crates/orbs-tui/src/*.rs crates/orbs-tui/src/*/*.rs` — nothing over
      300 but `theme.rs`, which box 7 splits.
- [x] **The played harness gets a builder and a kept folder** (`0.19.1`) —
      `Game::launch()` and its knobs; `Kept`, a folder for towers and settings
      that outlives the games launched into it. `play.rs` split into `launch`,
      `transcript` and `probes`. §19
      **See it:** `scripts/play.sh kept::` — `motion off` set at one launch
      reads `[off]` at the next, and §14's warning is said at the first launch
      and not the second.
- [x] **Supply, played** (`0.19.2`) — fifteen scenarios across `supply`,
      `garrison` and `demons`: a potion's rates, `stop` and the board's
      `drawing` row; spent stock; one supply an item; a spell's `has no draw`;
      a batch moved whole; the lectern's two; the levy, desertion and a full
      garrison; golems bound and deployed; a plain-English `stop`. §19
      **See it:** `scripts/play.sh supply::`, `scripts/play.sh garrison::`,
      `scripts/play.sh demons::` — 15 passed.
- [x] **Sieges, standing and the ledger, played** (`0.19.3`) — ten scenarios
      across `besieged`, `standing` and `ledger`: stillness ending a siege and
      refused to a spell; vigour raising the wall; a brew's quintessence in
      `recall`; a lie that outlasts the siege until `purge`; the coffer counting
      down; `petition`; a win sung once; `status`'s ledger and a room's spells'
      share. §19
      **See it:** `scripts/play.sh besieged::`, `scripts/play.sh standing::`,
      `scripts/play.sh ledger::` — 10 passed.
- [x] **The front door and the settings, played** (`0.19.4`) — thirteen
      scenarios across `kept`, `threshold` and `settings`: abandoning, loading
      another tower, a chosen name; the manual and `back` before there is a
      tower, `F6` writing nothing there, the boot card playing once, a length's
      ley line, a game's own seed; `skip` holding, `echo changed`, `motion off`
      stilling the fire, the menu's secrets. §19
      **See it:** `scripts/play.sh kept::`, `scripts/play.sh threshold::`,
      `scripts/play.sh settings::` — 15 passed with box 2's two.
- [x] **Readers and refusals, played** (`0.19.5`) — seven scenarios across
      `readers` and `refusals`: the augury reading a line and `reading plain`
      leaving it unasked; a spell line read and marked `≈`; a guessed `summon`
      refused; `winnowing` holding a beast; a leftover word named; an
      unplaceable word asked about; `for each store` reaching every spent store.
      §19
      **See it:** `scripts/play.sh readers::`, `scripts/play.sh refusals::` —
      7 passed.
- [x] **The rasteriser draws what it was told** (`0.19.6`) — `▲` and `←`
      narrowed, held by a scan of the painters' source; colour before weight,
      so `NO_COLOR` keeps dim and bold; `theme` split, resolving in the
      window's order. §19
      **See it:** `cargo test -p orbs-tui --bins scan` and
      `scripts/play.sh sight::` — both fail with either fix taken out, which
      was tried. The fallback itself: `target/debug/orbs-tui` run directly in a
      VTE terminal set to ambiguous-wide, `attend menagerie`, `summon`, `limn
      keystone heed`, `summon` — the marks row stays under its numbers.
- [x] **Every key a terminal sends** (`0.19.7`) — Shift+Tab completes as Tab;
      PgUp and PgDn page nothing at the threshold; the menu reads the register
      per keystroke, so `F7` silences its jokes at once; every shared `Key` is
      matched with no `_` arm, so a new one fails to compile. §19
      **See it:** `scripts/play.sh keys::` — `BTab` after `atte` writes
      `attend`; `F7` with the menu up and `xyzzy` is refused.
      `cargo test -p orbs-tui --bins mapping paging secret` — the secret test
      fails with the per-keystroke refresh taken out, which was tried.
- [x] **A swap replaces the tower and nothing else** (`0.19.8`) — opening a
      save is one function (`orbs_shell::tower_from`, `loaded`); both builds
      write the tower being left before reading, so loading the tower you are
      in keeps where you are; a load never raises a tower, and a refused one
      re-lists the slots; **a new tower never takes the slot of the tower being
      left** (`Raise::begin` asks again); one watchdog per process. §19
      **See it:** `scripts/play.sh swapping::` — six scenarios, and three of
      them fail with the slot or the watchdog fix taken out, which was tried.
      `cargo test -p orbs -- menuing` holds the window's three; `cargo run -p
      orbs`, raise a tower, `menu` → `play` → `new` before sixty ticks, then
      `menu` → `play` → `1` returns the first tower where it was.
- [x] **`ORBS_MOTION`** (`0.19.9`) — `0` still, `1` moving, blank to the row;
      it seeds the `motion` row at launch and the row wins after, read once in
      each build (`settings::motion_wanted`). The played suite defaults to `0`
      and the scenarios about motion opt in; `tui.sh` passes `0`, `dumps.sh`
      pins it blank. §19
      **See it:** `ORBS_MOTION=0 ORBS_BOOT=0 ORBS_SAVE=off ORBS_MENU='settings\naccess' target/debug/orbs-tui --dump menu`
      reads `motion anything moving [off]`, and `[on]` with `1` or blank; the
      same with `-p orbs`'s `ORBS_DUMP=menu`. `scripts/play.sh` stays green, the
      fire scenarios included.
- [x] **Numbers roll and screens cross, in the terminal** (`0.19.10`) — the
      `motion` row is the terminal's one switch; `Climb` and `Passing` are
      driven on it, the crossing through a shared `Passing::drive` the window
      calls too. `passing.rs` split into a folder. §19
      **See it:** `ORBS_MOTION=1 ORBS_PASSAGE=1 scripts/tui.sh start`, then
      `type 'debug_renown 20'` — `0/25  +20` rolling to `20/25` — and `type
      'attend forge'`, the strip wiping across. `scripts/play.sh motion::`;
      `cargo test -p orbs-tui --bins session`.
- [x] **Text types itself, and `pace`, in the terminal** (`0.19.11`) — one
      shared `Reveal::drive` (notice, then type or land whole) both builds
      call; `settings::PACE` and `settings::pace()` in the shell; the `pace` row
      in the terminal, carried across a swap; any keystroke finishes it; the
      echo row settles what it puts back. `reveal.rs` split into a folder. §19
      **See it:** `ORBS_MOTION=1 scripts/tui.sh start`, then `menu`,
      `settings`, `habits`, `pace 1200`, back out, and `type 'recall clarity'`
      — it types over a second and a half, and any key lands it.
      `scripts/play.sh motion::`; `cargo test -p orbs-tui --bins session`.
- [x] **Greyscale in the terminal: `F8` and `sight`** (`0.19.12`) — `Sight`
      and `settings::SIGHT` move to the shell, read by both builds; the
      terminal's `sight` row, `F8` and `ORBS_SIGHT` take every hue to one of the
      terminal's greys by lightness and keep the weights; carried across a swap
      and written to the file. §19
      **See it:** `scripts/tui.sh start`, `type 'attend laboratory' 'move sage
      to mortar_and_pestle'`, `ink 89 103` reads green; `key F8`; `ink 89 103`
      reads grey. `scripts/play.sh sight::` — six scenarios.
- [x] **`unfurl` steps one record, in both builds** (`0.19.13`) — one shared
      `orbs_shell::apply_to_reading`: the arrows one record, `Home` and `End`
      the ends, `Escape` out. The window called its page system on the arrows
      and bound no `Home`/`End`; it now runs one `input_just_pressed` system a
      key through the table. The manual and `recall unfurl` say so. §19
      **See it:** `scripts/tui.sh start`, a few `recall clarity`, `unfurl`,
      then `key Up`, `key Home` (the session's first line), `key End`.
      `cargo run -p orbs`, the same keys. `scripts/play.sh unfurling::`;
      `cargo test -p orbs -- arrows_step`.
      ✅ **Reviewed** (`0.19.13`) — fourteen findings, three a player could
      meet: with the installation on, a second tower's wake blamed another orb
      for the orb's own tower; `Home` went one past the top, so the next key
      looked dead; and loading the tower in hand after a failed write read an
      older save over it. A refused load now says why, a tower opened from the
      menu no longer types its history out again, and no unit test moves with
      `ORBS_PASSAGE`, `NO_COLOR` or `ORBS_SAVE` exported. §19
      **See it:** `scripts/play.sh swapping::`; `env NO_COLOR=1 ORBS_SAVE=off
      ORBS_PASSAGE=0 cargo test --workspace`.

---

## The tower swept

**The same game with one copy of each rule.** Measured at the start: 50k lines of
shipped code with almost no copy-paste, 35k lines of comments on it, 57k of tests
with no shared helpers. The waste is two expressions of one rule, which §19
records more defects from than anything else. §19 *The tower swept* has the
audit, the order and what was deliberately left apart.

**`0.20`, one patch per box.** A refactor box's gate is that nothing changes:
`scripts/dumps.sh` byte-identical apart from the POST card's version.

- [x] **What the audit found broken** (`0.20.0`) — the forge credited a lit
      fall before saying it, so its mastery line printed above the charm; the
      archive's own records carried no `Source`, so `archive.log` held neither
      the opening nor the solve. Two dev-only: `dev_spells.toml` errors named
      `spells.toml`, and the window's content reload on a swap was a second copy
      of `orbs_shell::load` that never said the directory was missing. §19
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend forge; imbue mortar_and_pestle
      hurried; snap apex; anneal; meditate 25" cargo run -p orbs` — *every glyph
      holds* above *the forge line advances*. `ORBS_SEED=1 ORBS_BOOT=0
      ORBS_DUMP="attend archive; debug_spell threading; research"
      ORBS_THEN="invoke threading; meditate 3600; meditate 3600; meditate 3600;
      sift fragment archive.log" cargo run -p orbs` — *the stacks give up
      fragment*. `cargo test -p orbs-sim --lib every_domain_says_its_work`.
- [x] **Test helpers, shared** (`0.20.1`) — `tests/common` and `src/testing.rs`
      replace the copies of `run`, `said`, `ever_said`, the replay loop and the
      save round trip; a played game starts only through `Option<Game>`, so
      216 hand-written tmux guards went. §19
      **See it:** `cargo test --workspace` passes the same 2,545; break
      `common::said` and 215 fail. `scripts/play.sh` — *216 passed*, no
      `SKIPPED`
- [x] **One plumbing for the puzzle domains** (`0.20.2`) — `execute/settle.rs`'s
      per-puzzle table holds `refresh` and `stop`'s `abandon` too; five domain
      `say`s are one `readings::filed`; the mastery lines, the renown ranks, the
      scene's vocabularies and `land`'s special completions are each walked
      once. §19
      **See it:** `scripts/dumps.sh` byte-identical; `cargo run -p orbs-balance --
      sweep --ticks 7200 --why` identical
- [x] **`build.rs` reads its names from the models** (`0.20.3`) — the branches
      built by five `const fn`s from the models' own names, 1,557 lines to
      1,245; one walk where three were; three dead items gone. §19
      **See it:** as above
- [x] **Debug words and handshakes** (`0.20.4`) — one `DebugWord` parser and
      the bodies in `execute/debug/`, so `sim.rs` loses 500 lines; one
      `Handshake<M>`, one `say_save`, one `submit_with`. §19
      **See it:** as above
- [x] **One shell for both frontends** (`0.20.5`) — the editor, the weave, the
      menu, the swap, the readers, the rows, autosave and the prefix rule
      written once in `orbs-shell`; the window's tests out of `plugin.rs`. §19
      **See it:** dumps and `cargo run -p orbs-render --example screens`
      identical; `scripts/play.sh` — *216 passed*; `cargo test -p orbs --bin
      orbs shell::` passes
- [x] **`orbs-shell` internals** (`0.20.6`) — one painter frame and typed line
      for the menu and the manual, one `Split`, no `serde`; the command line and
      the board loop left, and §19 says why
      **See it:** as above
- [x] **The window's internals** (`0.20.7`) — one fullscreen pass for the tube
      and sight, `Modal<T>`, one chord check, one transcript-key system, one
      `pair!` for the two-word settings; `crt/plugin.rs` registration only. §19
      **See it:** dumps and screens identical; `scripts/play.sh` — *216 passed*;
      `cargo test -p orbs --bin orbs shell::` passes. SEEING-IT's greyscale
      matrix with `ORBS_CRT=peak` added, captured twice before as a control: off
      is pixel-identical, on differs by the control's own 1–2 levels, greyscale
      has no hue in any state
- [x] **One test per property** (`0.20.8`) — duplicates deleted, near-identical
      tests made tables; every removal named beside the test that still holds it.
      §19
      **See it:** `cargo test --workspace` passes 2,539 where it passed 2,547;
      `scripts/play.sh` — *210 passed*, no `SKIPPED`; §19 pairs each removal
      with its survivor

**The comment pass** holds every comment to CLAUDE.md's *a line or two, why not
what*. A paraphrase of §19 becomes a pointer, and a why with no home there moves
into it. One box per area, none over ~5k comment lines. **See it, for each:**
`scripts/only-comments.py` passes and its negative control fails; `cargo test
--doc` counts the same.

- [x] **`scripts/only-comments.py`, then `tower/spell` and `tower/siege`** (`0.20.9`)
      — 3,044 comment lines to 1,760. §19
      **See it:** `base=$(scripts/only-comments.py snapshot)` before a box and
      `scripts/only-comments.py check "$base"` after; `scripts/only-comments.py
      self-test` — *15 cases, 0 judged wrongly*
- [x] **The rest of `tower/`, first half** (`0.20.10`) — `assault` to `ley`,
      2,809 comment lines to 1,850. §19
- [x] **The rest of `tower/`, second half** (`0.20.11`) — `mastery` to
      `work/`, 3,439 comment lines to 2,624. §19
- [x] **`parser/`** (`0.20.12`) — 3,342 comment lines to 1,898; three docs
      that sat on the wrong item moved back. §19
- [x] **`execute/`** (`0.20.13`) — 3,345 comment lines to 2,253; a stale
      module table and `research.rs`'s pull measurement moved into §19. §19
- [x] **`save/`, `content/` and the top of `orbs-sim`** (`0.20.14`) — 4,202
      comment lines to 3,251; `play.sh` and the probes stop leaving tmux
      servers and sockets behind. §19
      **See it:** `scripts/play.sh probes::`, then `ls /tmp/tmux-$(id -u)` — no
      live server, and one socket per probe however many runs
- [x] **`orbs-shell`, first half** (`0.20.15`) — `bench` to `menu/`, 3,998
      comment lines to 3,416. §19
- [x] **`orbs-shell`, second half** (`0.20.16`) — `panel` to `transition`,
      3,814 comment lines to 3,183. §19
- [x] **`orbs-render`** (`0.20.17`) — 5,130 comment lines to 4,052; a stale
      `[reagent]` heading doc corrected. §19
- [x] **`orbs`** (`0.20.18`) — 4,286 comment lines to 3,254. §19
- [x] **`orbs-tui`, `orbs-augury`, `orbs-balance`** (`0.20.19`) — 3,885
      comment lines to 3,012; the measurements the margins carried moved into
      §19. §19
- [x] **The tests' comments** (`0.20.20`) — 5,981 comment lines to 5,053; two
      docs on the wrong item moved back, and a stale nine-line doc on three tests
      cut to two lines each.
      §19
- [x] **Manifests, content TOML and shaders** (`0.20.21`) — 3,153 comment
      lines to 1,663; eleven stale claims corrected and three docs moved back
      to the item they describe. §19

- [x] **Docs** (`0.20.22`) — closed ROADMAP sections and CLAUDE.md lose the
      narrative §19 already holds; ROADMAP 7,548 lines to about 3,890. Stale
      claims in DESIGN §1–18, CLAUDE.md and the game-plan skill corrected, and
      SEEING-IT gains *Proving nothing changed*. §19
      **See it:** every section's box and See-it counts unchanged; every anchor
      resolves

---

## The tower speaks up

**Progress a player can read: a rate, a wait, a forecast, and the tower's own
news.** Every surface said where the tower stood; none said how fast, what a
stalled room was waiting on, or when the next station would come — and what a
spell earned never reached the screen. The look rides on it: the glass answers
the tower, and the phosphor keeps what just changed. §19 *The tower speaks up*.

**`0.21`, one patch per box.** Each fact has one source the rail, `status
rates`, the road and the sweep all read.

- [x] **`status rates`** (`0.21.0`) — every open room's rate over the last hour
      of ticks, what it waits on, and what holds the tower's one slot. A
      `Throughput` ring per tally key, fed at the tally's one door and saved
      with no format bump; `tower::wait_of` ranks a room's instruments so a
      stalled one says why, and a held spell says what holds it on all four
      paths. `status` takes one closed word; the sweep's `--why` reads the
      same yields. §19
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal;
      debug_spawn clarified-draught 3; distil clarified-draught; meditate 60;
      empty alembic; distil clarified-draught; meditate 60; status rates" cargo
      run -p orbs` reads *brews finished: 55 an hour · the alembic is ready.
      empty it* and *slot  free. nothing holds it*. `…; grind sage; status
      rates` names the grind holding the slot. `status zorb` reads *status
      takes only rates*. `cargo test -p orbs-sim --test rates`
- [x] **The rail says what a room waits on, and the forge's charm** (`0.21.1`)
      — `brief` reads `wait::reported` and `wait_in`, the function `status
      rates` reads, so a laboratory with a fouled alembic and a cold athanor
      reads `fouled`, not `idle`. The detail row is the wait in twelve cells —
      `at mp`, `fr +tincture`, `after mp`, `bides 12t` — heard as the sentence
      `status rates` says; the forge's names the charm nearest lapsing by the
      word the forge publishes. A cold hearth alone is not a wait. Ticks *the
      tower as one machine*'s **The player can see a bottleneck**. §19
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; grind sage;
      meditate 9; attend archive" cargo run -p orbs` — the laboratory box reads
      `ready  0%` / `at mp`, and the reader hears *the mortar_and_pestle is
      ready. empty it*. `ORBS_BOOT=0 ORBS_DUMP="attend forge; imbue
      mortar_and_pestle hurried; snap apex; anneal; meditate 20; attend
      laboratory" cargo run -p orbs` — the forge box reads `graced 598t`.
      `scripts/dumps.sh` captures `rail_stalled` and `rail_charm`
- [x] **The road and the ley gauge forecast** (`0.21.2`) — `~4 min` to the
      room's next station, from its deed's pace between makings, and to the
      next Ley station, from the pace experience was earned at. Both read
      gaps, never the clock, so a forecast moves only when something is made;
      both take only blank space and are said as drawn. Renown gets none. §19
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal;
      debug_spawn clarified-draught 3; distil clarified-draught; meditate 60;
      empty alembic; distil clarified-draught; meditate 60" cargo run -p orbs`
      reads `0/8  ~2 min` on the ley gauge and `5 brews finished · 2 of 5 · ~4
      min` on the road. `ORBS_GRID=80x22` keeps the road's and drops the
      ley's, which has no blank beside the room's column. `scripts/dumps.sh`
      captures `forecast` and `forecast_floor`
- [x] **The tower's news reaches you, whoever did the work** (`0.21.3`) — a
      station, a room opened, a rank, a level and a fork becoming choosable
      carry `RecordBuilder::news`, so `drawn()` shows them even when a spell
      earned them; they keep `Spell`, so `sift` names the culprit. The chime is
      the success cue, not a yield's. A station or a room reached while the
      player stands elsewhere latches `!` on its box. A fork announces itself
      once. §19
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal;
      debug_spawn clarified-draught 1; scribe brewing"
      ORBS_EDIT=$'edit\ndistil clarified-draught\n<esc>\nquit'
      ORBS_THEN="invoke brewing; meditate 70" cargo run -p orbs` reads *the
      laboratory line advances: a clarity brewed* with no yield around it.
      `…; distil clarified-draught; attend archive; meditate 60` puts
      `laboratory   !` on the rail. A sealed tower's third brew reads *the ley
      line forks ahead. weave, and choose a lane*. `cargo test -p orbs-sim
      --test news`; `scripts/dumps.sh` captures `news_by_spell`, `news_away`
      and `fork_opened`
- [x] **The glass answers the tower** (`0.21.4`, window only) — the corners
      draw in as a siege turns against the garrison (the enemy's margin, so an
      even fight is nothing), and colour drains to 0.15 while a spell's fault is
      latched. Two levels read from the shell's `Panel`, ramped over 0.6 s,
      applied to the extracted copy so `F3` still cycles; `edge` governs both.
      `flash` is deleted. §19
      **See it:** from a scratch folder, `ORBS_STRAIN=0.9,1 ORBS_SAVE=off
      ORBS_BOOT=0 ORBS_THRESHOLD=0 ORBS_CAPTURE=1 timeout 25 <repo>/target/debug/orbs`
      beside the same line without `ORBS_STRAIN`: mean saturation 8.7 → 7.3,
      the corners darker, no error on stderr. Live, `attend bailey; defend;
      hold; hold` tightens the corners as the enemy band outgrows the garrison
      — that ramp needs eyes on a window
- [x] **Afterglow** (`0.21.5`, window only) — the phosphor keeps what just
      changed as a soft fade, under half after a frame and gone in about a
      tenth of a second (shortened from a quarter on the owner's look, which
      read it as ghosting), `max(now, before × decay)`,
      before the tube so a trail curves and is scan-lined. `fullscreen.rs`
      gains an optional history: one owned texture, recreated zeroed on any
      change of size or format, forgotten whenever the pass stops. Rides the
      `glow` row (*bloom, afterglow*); off with motion, the tube or
      `ORBS_AFTERGLOW=0`. §19
      **See it:** `cargo run -p orbs`, `debug_renown 20` — the digits trail as
      they roll; `wander` and `Esc` — the gather trails. `picture` → `glow
      off`, `access` → `motion off` or `F3` to `OFF`: no trail. `ORBS_CAPTURE`
      runs with and without it, through boot, tube off and motion off, with no
      validation error on stderr. The trail itself needs eyes on a window

**Every box ticked at `0.21.5`.** *Every instrument has a tempo* was carried to
`0.22`, below.

---

## Every instrument has a tempo

**Five rooms' panels are flat meters; the laboratory's four instruments are
pictures.** Each of the rest gets one, describing its *contents* rather than its
progress (§19, *the balneum mariae*), with a tempo of its own so what is running
is legible from across the room. Carried from *the tower speaks up*; §19 has the
tempo table each box builds from.

**`0.22`, one patch per box.** The five share one `orbs-render` module,
`figure`, a file per picture and one test file sweeping all five for the rules
that make a picture safe: the join survives greyscale, a resting figure is
still, a working one moves, no cell nears three flashes a second, and no moment
turns half the bar. One `Bar::Figure` arm in `orbs-shell/src/panel.rs`, one
`Bench::rite`, one `screens` section, a `dumps.sh` capture each. **Every box
ticked at `0.22.5`.**

- [x] **`Craft::Summoning`, and one figure arm for all five** (`0.22.0`) — the
      circle reports its own craft rather than `Idle`, and `bar_of` maps each
      puzzle craft to its figure while it works, plain otherwise
      **See it:** `cargo test -p orbs-shell panel` and `cargo test -p
      orbs-render figure`
- [x] **The prism** (`0.22.1`) — aligned sigils solid in the gauge's hue, a `▒`
      focus climbing the open sockets one cell per four ticks
      **See it:** `ORBS_SEED=3 ORBS_BOOT=0 ORBS_FIRE_PHASE=9 ORBS_DUMP="attend
      lens; probe; dial second borax; probe" cargo run -p orbs` — the `pr`
      column carries `▒▒` in its open part; step the phase by `0.67` and it
      climbs a cell
- [x] **The lattice** (`0.22.2`) — snapped columns burning, one cell in eight
      lifting to blaze each six ticks; cooler on `anneal`, ember lifting to
      body each eight. Hue only, as the athanor's body is
      **See it:** `cargo run -p orbs-render --example screens` — *The puzzle
      instruments*, both lattice strips. `ORBS_BOOT=0 ORBS_DUMP="attend forge;
      imbue mortar_and_pestle hurried; snap apex; snap belt; snap hem" cargo
      run -p orbs` — the `la` column solid
- [x] **The pylon** (`0.22.3`) — the wall in its standing's gauge hue, at rest
      too; while a course is up its top course turns `█`↔`▓` each nine ticks
      **See it:** `ORBS_BOOT=0 ORBS_FIRE_PHASE=2 ORBS_DUMP="attend sanctum;
      muster; haul wellspring barrier" cargo run -p orbs` — the `py` column's
      top `▓▓`; at phase `0` it is `██`
- [x] **The lectern** (`0.22.4`) — the scroll so far, its three seams open `▓`
      and knitting from the base one per sixteen ticks; a finished scroll
      stands whole, where it drew nothing
      **See it:** `ORBS_BOOT=0 ORBS_FIRE_PHASE=2 ORBS_DUMP="debug_spawn fragment
      4 lectern; attend archive; wield lectern; meditate 6" cargo run -p orbs`
      — a `▓▓` seam in the `le` column; at phase `40` it has knit. The
      `ledger` capture's ready lectern is a solid gold column
- [x] **The circle** (`0.22.5`) — agreeing rows solid in the gauge's hue,
      balking ones pulling `░`↔`▒` in place, a quarter of them each six ticks
      **See it:** `ORBS_BOOT=0 ORBS_FIRE_PHASE=2 ORBS_DUMP="attend menagerie;
      summon; limn keystone heed; limn sunwise oppose; summon" cargo run -p
      orbs` — the `ci` column's open part flecked `▒`

---

## The tour

**Every surface in the game, moving, one scene at a time** — in the window for
a person to watch and take over, as screenshots with a text copy for Claude to
read, and headless in `cargo test`. `dumps.sh` is the still photograph; this is
the film. §19, *the tour*, has the decisions and where each `dumps.sh` section
landed. **Every box ticked at `0.23.4`**, 87 scenes. The README's *The
visual showcase* has the commands.

**`0.23`, one patch per box.** A scene is data in
`orbs-shell/src/tour/scenes.toml`: a tower, a setup run at once through the
dump's own driver, steps typed at the real keyboard, and `expect`/`ends` checked
against the world so no scene claims a state it never reached. A debug build's
instrument: the setups use the debug words.

- [x] **The catalogue, staged headless** (`0.23.0`) — `orbs_shell::tour`: the
      scenes, `stage`, the `expect` check; `drive` and the editor replay moved
      from `dump.rs` to `script.rs`, one copy for both
      **See it:** `ORBS_TOUR=list cargo run -p orbs` — twelve scenes;
      `ORBS_TOUR=forge.lattice ORBS_DUMP=1 cargo run -p orbs` — the forge with
      the lattice half snapped; `scripts/dumps.sh` before and after, `diff -r`
      empty
- [x] **One scene in the window, and its screenshots** (`0.23.1`) — the steps
      as real keystrokes, a person's key taking the scene, nothing kept; each
      shot a PNG and the frame as text
      **See it:** `scripts/tour.sh laboratory.kindle` — `kindle charcoal` types
      itself and the fire flares up the athanor's column; press any key and the
      scene is yours. `scripts/tour.sh shots <dir> laboratory.kindle` — three
      PNGs, each with its frame as `.txt`, and `index.txt`
- [x] **The tour** (`0.23.2`) — playlists, `F11` and `Shift+F11`, the title,
      everything reset between scenes, `ORBS_TOUR_FILE`, the whole catalogue
      played in-App by `cargo test`
      **See it:** `scripts/tour.sh laboratory,forge` — one window, one scene
      after another, the title naming each. `scripts/tour.sh shots <dir>
      laboratory.grind,passage.wipe` — the second scene's shots catch the
      crossing to the forge mid-wipe. `cargo test -p orbs tour`
- [x] **The boot card and the threshold as scenes** (`0.23.3`) — the card
      replayed mid-session, a blank frame's mesh, the threshold reopened; the
      orb's menu, manual, installation, wake, `quit` and a resumed tower
      **See it:** `scripts/tour.sh shots <dir> boot,orb` — 33 shots, none
      `BLANK`, the card's letters arriving mid-session, and `stderr.log` holds
      nothing but the driver's notice. `scripts/tour.sh boot.card` to watch it
- [x] **The whole catalogue** (`0.23.4`) — every room, puzzle, crossing,
      surface and effect, and the tests that hold it whole
      **See it:** `scripts/tour.sh all` — 87 scenes, boot card to a whole
      cycle, in one window. `scripts/tour.sh shots <dir> all` — none `BLANK`,
      `stderr.log` clean. `cargo test -p orbs-shell tour` holds the coverage,
      every instrument state included; `cargo test -p orbs tour` plays every
      scene through the keyboard and paints every frame at 120×45 and 80×22,
      failing on any text cut (§19, *The tour checks what it shows*)

---

## Number go up

**Progress a player can watch.** Every number on screen snapped, nothing showed
a lifetime count, and what a spell did was never counted at all. Aesthetic
first, and like both interludes before it, not only aesthetic: the tally learns
whose hand did the work.

**Its boxes bump `0.17.x`** and interleave with *the tower as one machine*
(§19). **Every box ticked at `0.17.6`.**

- [x] **A number rolls to the truth** (`0.17.2`) — `orbs_render::Roll` and
      `orbs_shell::Climb`. Both gauges' digits roll and their bars fill, half a
      tick, three steps on the flip grid. Speech, colour and width are the
      truth's; a crossing cuts; a number that leaves the screen is forgotten, so
      coming back is never a gain. F3 stops it, and `orbs-tui` never rolls
      **See it:** `ORBS_BOOT=0 ORBS_ROLL_AT=0.4 ORBS_DUMP="debug_renown 0; debug_renown 20" cargo run -p orbs`
      reads `11/25` on its way to `20/25` and speaks `20 of 25`.
      `cargo run -p orbs-render --example screens` — *A number climbing*.
      `scripts/dumps.sh` byte-identical without the switch
- [x] **A gain says how much, and a station lands** (`0.17.3`) — a fading
      `+N` of the whole total earned, silent, in the blank after the reading;
      and one beat across a tier however many it crossed: the tier left fills,
      shows *arrived*, and the bar lands in the next. Into the top it stays full
      **See it:** `ORBS_BOOT=0 ORBS_ROLL_AT=0.5 ORBS_DUMP="debug_renown 20; debug_renown 30" cargo run -p orbs`
      reads `25/25  hedge-wizard  +10`; `0.1` still reads `20/25`, `1.0` lands
      on `5/35`. `debug_renown 400` is one beat with `+380`
- [x] **The tally learns whose hand, and `status` keeps a ledger** (`0.17.4`) —
      a spell's work counted a second time under `spell:`, read off the
      attribution `done` already ran inside; `[[ledger]]` rows authored with a
      deed's spellings and no number; and a `ledger` section in `status`. No
      `FORMAT` bump, and the sweep is unmoved
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend archive; debug_spawn fragment 8; debug_spell assembling" ORBS_THEN="invoke assembling; meditate 120; status" cargo run -p orbs`
      reads `scrolls copied 1`, `scrolls copied by your spells 1`,
      `done by your spells 1`.
      `cargo test -p orbs-sim --test ledger`
- [x] **The room counts what it made** (`0.17.5`) — the room's first ledger row
      and its spell share at the right of the gauge strip, rolling with `+N`.
      It reserves its width first and draws only where every bar keeps 24 cells
      and its title, otherwise it drops whole. The road's `n of m` rolls too.
      The strip's *"these two and nothing else"* is superseded (§19)
      **See it:** `ORBS_BOOT=0 ORBS_ROLL_AT=0.4 ORBS_DUMP="attend archive; debug_spawn fragment 8; debug_spell assembling; invoke assembling; meditate 30" cargo run -p orbs`
      reads `scrolls copied 0 +1` and `your spells' share 0 +1` on their way to
      `1`; `1.0` arrives. `attend laboratory` reads `potions brewed 0`.
      `scripts/dumps.sh` captures `roll_column`
- [x] **The siege board's stock rolls** (`0.17.6`) — the coffer and both sides'
      vigour, drawn from a rolled copy and spoken from the truth. `chance`
      snaps; the pylon and the rail stay still, and §19 says why
      **See it:** `ORBS_BOOT=0 ORBS_ROLL_AT=0.4 ORBS_DUMP="attend bailey; defend; pledge d20 to buckler" cargo run -p orbs`
      reads the coffer at `22` on its way from `24` to `19`. `…; pledge d20 to sortie; hold`
      rolls both bands — the enemy `15 → 12 → 9`, the garrison `18 → 16 → 14`

---

## The installation

**A new tower is installed before it wakes.** The wizard initialises the orb.
After `new` and a length, the player stands in a **practice tower**: the real
sealed tower from the same seed, used through the real prompt, and thrown away
afterwards. A strip where the gauges sit teaches the prompt and the way around in
five steps, a line each. Prompt and navigation only, a few minutes at most; the
potion and the spell stay with *Onboarding + demo*.

**Its boxes bump `0.17.x`**, for *Number go up*'s reason. **Every box ticked at
`0.17.8`.**

- [x] **A new tower is installed first** (`0.17.7`). `new` and a length reach
      the practice tower, `Sim::sandbox`: `begun` with no opening report, so the
      report is read once, in the real tower. It is kept nowhere; the slot
      `new` found travels in `orbs_shell::Raise` and is checked again at the
      wake.

      Five steps: `help`, `survey`, `attend`, plain words, `status`. Each is
      finished by a line that resolved, judged on `ParseLog` from the step's own
      start, so one line finishes one step. `wake` at the end, or `skip` at
      any step, raises the real tower from the same seed at tick 0. Neither
      reaches a world; a bare Enter is only a line. The F5 mirror pins the
      strip above the transcript's tail, or `help`'s answer would push the
      lesson out of §14's route.

      Found by playing it: a terminal swap left the old screen's glyphs behind
      (§19)
      **See it:** `ORBS_INSTALL=1 ORBS_BOOT=0 ORBS_DUMP="help; survey" cargo run -p orbs`
      reads `step 3 of 5`. `…="help; survey; cd laboratory"` reads `step 4 of
      5`, not 5. `…="help; survey; attend laboratory; go to the tower; status"`
      reads `the orb is initialised` and `try  wake`; `…; status; wake; status"`
      and `…="help; skip; status"` are the real tower.
      `scripts/play.sh installing::` plays what a dump cannot: the menu around
      it, and F5. `scripts/dumps.sh` adds `install_*`, with every earlier
      capture byte-identical
- [x] **`install` in settings** (`0.17.8`). A habits row beside `skip`, on by
      default; off, `new` raises the tower at once. `ORBS_INSTALL` outranks it:
      `0` never installs, `1` always does, blank falls through
      **See it:** `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_THRESHOLD=1 ORBS_DUMP=1 ORBS_MENU='settings\nhabits' cargo run -p orbs`
      shows `install  setup on a new orb  [on]`.
      `scripts/play.sh installing::the_setting` turns it off at the threshold
      and gets a tower. The `menu_habits` and `menu_plain` captures gain the row

---

## The orb made yours

**Settings chosen from research, filtered through the design.** About 190
options from accessibility guidelines (GAG, Xbox XAGs, APX), terminal emulators
and comparable games, cut to what §3, §4, §14 and §19 allow. The rejected half
is in DESIGN.md §19 with its reasons, so it is not argued again.

**Its boxes bump `0.17.x`**, for *Number go up*'s reason; several are *Ship*
items done early. **Every box ticked at `0.17.15`.** What research turned up and
this does not build is catalogued in DESIGN.md §19.

- [x] **The manual told the truth again** — a correction, no bump. `yours`
      said `tube off` (the row is `crt`), colour-vision `sight` modes that do
      not exist, and a `focus` description backwards against `DisplayMode`;
      `keys` left out F8 and F12. And the terminal build's `skip` row read the
      file, so with nowhere to write it snapped `skip on` back to `off`: its
      rows and its settings are one `Carried` now, with a test that every row
      is honoured
- [x] **`motion`, its own switch, and §14's health warning** (`0.17.9`). An
      access row in both builds. In the window, motion is the row *and* the
      tube — either saying still is enough, and neither can bring motion back.
      In the terminal it is the row alone, since there is no tube. Off stills
      the fire, the flare, the crossings, rolling numbers, the hum band, the
      caret, text typing itself, pane changes and the boot card, which arrives
      whole and leaves by a cut. The warning is said once per machine, under
      the threshold's menu, and only where settings are kept
      **See it:** `cargo run -p orbs`, `settings` → `access` → `motion off`
      with the tube on: the athanor's fire is still, `attend` cuts, text lands
      whole and the caret stops blinking. In a terminal, start `orbs-tui` with
      `ORBS_SAVE` at an empty folder: the warning is under the menu once, and
      not on the next launch. `scripts/dumps.sh` moves `menu_access` alone
- [x] **The `picture` page** (`0.17.10`) — `curve`, `lines`, `glow`,
      `flicker`, `mask`, `split`, `edge`, each `off`, `soft` or `full`, in the
      window build only. Ceilings on whatever `crt` chose, never a second tube:
      `F3` and `crt` still pick the state, and `crt::effective` is the one place
      the state, the ceilings and stillness meet. The manual's *"the shape
      without the flicker"* is true now. A letter two rows share names neither
      — `Menu::set` took the first match
      **See it:** `cargo run -p orbs`, `settings` → `picture` → `flicker off`:
      the curve stays and the band stops. With `curve off`, `lines off` and
      `edge off` in the settings file, `ORBS_CAPTURE=1` draws a flat glass with
      square corners and the glow intact. `scripts/dumps.sh` moves
      `menu_settings` and `menu_unsettable`, which list the new page
- [x] **Legibility** (`0.17.11`), window build only. **`contrast high`**
      lifts ordinary and spell text to 7:1 on every theme, keeps the ramp's
      steps at 1.25:1, and leaves the accent triad alone. **`weight heavy`** is
      the VGA's own bold, generated from the shipped fonts, with the shades and
      box drawing exempt and no two glyphs merged. **`scale whole`** fits the
      picture to a whole multiple, with `Screen` holding the fit so the camera
      and the tube read one scale. **`pace`** is the reveal in baud — `1200`,
      `4800`, `9600`, `instant` — and stillness still forces instant
      **See it:** muted violet with `contrast high`, `weight heavy` and `scale
      whole` in a scratch settings file, under `ORBS_CAPTURE=1`: the dim chrome
      is brighter, every stroke a pixel bolder, the accents unchanged, and the
      log says `scale 2.000` where the same window fills at `2.250`. `pace 1200`
      types an echo slowly enough to watch. `scripts/dumps.sh` moves the four
      settings pages that gained a row
- [x] **`echo`** (`0.17.12`), both builds — §6's and §14's *echo verbosity is
      a setting*, promised since the design and never built. `changed` leaves
      out an echo only when it resolved cleanly and repeats the line word for
      word; a different reading, a best guess and an unknown word all still
      echo. One predicate in `Records::drawn`, so the pane, the reveal, the
      scroll, the cues and the plain stream cannot disagree, and nothing leaves
      the stream — a log or a spell sees every echo
      **See it:** a terminal session with `echo = "changed"` in a scratch
      settings file: `survey` has no echo, `look around` still reads `→
      survey`, and `frobnicate` still says the orb does not know it. With
      `always`, `survey` echoes too. `scripts/dumps.sh` moves `menu_access`
- [x] **The voice** (`0.17.13`), window build only. **`keys off`** silences
      the click and nothing else, and leaves no held-over burst for when it
      comes back on. **`mains 60`** builds the hum on 60, 120 and 84 Hz, each a
      whole number of cycles in the bed's half-second lap. **`away`** — `off`,
      `dim`, `hush`, `both` — clouds the glass over 0.6 s through the sight
      pass, so it holds with the tube off, and quiets the *hum* only: the cues
      are §14's channel and how a finished brew reaches a player in another
      window
      **See it:** `mains = "60"` in a scratch settings file and `ORBS_SOUND=1`
      prints `sound: hum (plain, full, 60 Hz)`. `keys off`, then type: no
      `sound: key` lines, and a finished brew still chimes. `away both`, then
      alt-tab: the glass eases dark and the hum stops; come back and both
      return. The alt-tab needs a person, since a launched window takes focus.
      `scripts/dumps.sh` moves the sound and habits pages
- [x] **The wizard's name** (`0.17.14`), both builds. A habits row that takes
      free text — the first: one word, sixteen letters, drawable on the glass,
      case kept. It names *new* towers; a save outranks the machine, so a
      standing tower keeps its name. The word alone puts it back to the
      default, which stays the machine's login. A refusal says what a name can
      be. The order is `ORBS_WIZARD`, then the choice, then the login, and each
      frontend asks the choice it holds rather than the file, so a name chosen
      where nothing can be written still counts
      **See it:** `orbs-tui` with `ORBS_SAVE` at an empty folder: `settings` →
      `habits` shows `name … [<your login>]`; `name Morgan le Fay` says *a name
      is one word*; `name Morgana`, then `play` → `new`, and the prompt reads
      `Morgana $`. `scripts/dumps.sh` moves `menu_habits` and `menu_plain`, which
      show `ORBS_WIZARD`'s name rather than whoever ran them
- [x] **Secrets** (`0.17.15`) — §3's sysadmin comedy, discovered and never
      pushed. A row can answer words it never offers: whole, never by prefix,
      never cycled. **Phosphor part numbers** on the tube page — `theme p3` is
      amber, `p1` and `p31` green, `p4` monochrome, and `theme p7` a radar tube
      no list shows, held to every palette floor the others are and back after
      a relaunch. **`hum eleven`** and `voice eleven` are `full`, and the orb
      says it does not go to eleven. At the menu, **`xyzzy`**, `plugh`, `hello
      sailor`, `sudo`, `make me a sandwich`, `sudo make me a sandwich`, and
      **`moon`**, which reads the moon's phase from the clock and names all
      hallows' eve and a Friday the 13th. All silent while the tower speaks
      eldritch. A dump reads no clock unless `ORBS_DATE` names a day
      **See it:** `ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DATE=2025-03-14 ORBS_DUMP="menu" ORBS_MENU="moon" cargo run -p orbs`
      reads *the moon is full*; without `ORBS_DATE`, *the orb cannot see the
      sky from here*. `ORBS_MENU='settings\ntube\ntheme p7'` reads `[p7]` and
      *the orb knows its phosphors by number*. `scripts/dumps.sh` gains
      `menu_picture`, `menu_xyzzy`, `menu_moon`, `menu_moon_full`, `menu_p7`,
      `menu_eleven` and `menu_name_spaced`, with every earlier capture
      unchanged
- [x] **The review's corrections** — no bump. A `/code-review xhigh` found
      fourteen; thirteen are fixed and one is declined with its reason, all in
      DESIGN.md §19. The ones a player meets: `pace` held again once echoes were
      hidden, `echo always` no longer re-typed the old echoes, a row's secrets
      went silent in the eldritch register, `keys off` kept Enter, the name
      refusal named the letter instead of drawing `?`, and `scale whole` sat on
      whole pixels in an odd window
      **See it:** `scripts/dumps.sh` matches the box's capture but for the
      version, and `scripts/play.sh` plays all 149. `scale whole` in a window
      dragged to an odd width, under `ORBS_CAPTURE=1`: every glyph pixel the
      same size
- [x] **The hum is gone** (`0.17.16`) — the owner's call. The bed under
      everything is removed, not turned down, and with it the `hum` and
      `mains` rows and `away`'s `hush` and `both`. The orb sounds only when
      something happens; the cues and the click are unchanged. DESIGN.md §19,
      *The hum is gone*
      **See it:** `cargo run -p orbs` with the sound up: silence until a key.
      `settings` → `sound` lists `voice` and `keys`, and `away` takes `off` or
      `dim`. `scripts/dumps.sh` moves `menu_volumes`, `menu_eleven` and the
      sound and picture pages
- [x] **Each kind of sound on or off** (`0.17.18`) — the owner's ask. Under
      `voice`, which stays the volume: `keys` (typing and Enter), `chimes`,
      `alarms` and `refusals`, each `on` or `off`. One gate, in `ring`
      **See it:** `settings` → `sound` lists all five. `keys off` under
      `ORBS_SOUND=1`, then type: `sound: key (off)` and `sound: enter (off)`,
      and a finished brew still prints `sound: done`. `scripts/dumps.sh` moves
      the sound page, `menu_volumes` and `menu_eleven`

---

## Phase 8 — Siege ✅

**Exit:** sieges are tense and scripts visibly matter.

**Built at `0.8.1`–`0.8.7`.** The bailey, a turn-based defence resolved on D&D's
seven dice, the arsenal spent on it at last, a decision tree that fights one
unaided, escrow, and the adversarial aberrations that close §8.1's four-surface
model. **Three items are deliberately left** and are named at the foot.

**It is not an eighth domain** (§10, §5). The bailey is the arsenal's shape: a
real place with its own log and verbs, and no rail box.

- [x] **Autobattler** — `defend` lets an enemy arrive, `hold` ends your turn and
      resolves one round. No clock in it; turn-based serves the hand and the
      script; everything on your turn is free; it takes no production slot (§19)
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; survey enemy;
      survey garrison; hold"` — the enemy says what it means to do, both bands
      read back, and one round resolves
- [x] **The seven dice, composed before they are resolved** — `d20`, `d100`,
      `d12`, `d10`, `d8`, `d6`, `d4`, in `tower::dice`. A `Roll` is assembled
      before it is drawn, each modifier carrying its source, and the draw count
      is a function of the roll, never the outcome
      **See it:** ✅ `peruse bailey.log` after a round names the die and the face
      for every roll: `enemy strikes - d20 gives 12 against 11, and it tells`
- [x] **The board** — both bands, the telegraphed intent, and **the odds before
      the commitment**, which is §5.1's fairness rule drawn. Strength is bar
      *length*, never colour (§14), so it reads in greyscale and in a dump
      **See it:** ✅ `cargo run -p orbs-render --example screens` draws it with no
      sim at all; ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend"` draws it
      beside the transcript
- [x] **The arsenal spends, and the ten apology lines retire.** ~~Potions,
      scrolls and the menagerie's troop are pure mathematical advantages applied
      on your turn~~ — **superseded by *Supply* (`0.18.1`, `0.18.3`)**: troops are
      the tower's standing garrison, and no single potion or scroll does
      anything; each is a supply that draws every round. Authored in
      `content/siege.toml` (rule 6), so a tuning pass is a content edit rather
      than a recompile per guess
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend;
      debug_spawn troop 2; deploy troop; quaff troop; deploy sage"` — the troop
      joins the line, and both refusals name the way forward
- [x] **The decision tree** — `besieging`, on the grimoire's shelf in a debug
      build — *if base troops low, send demons; if health low, apply potions; if
      outnumbered, wield a scroll*. Every rung is a reading the world publishes
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; debug_spawn warding 12;
      debug_spawn clarity 12; debug_spawn golem 12" ORBS_THEN="invoke besieging;
      meditate 300; peruse bailey.log"` — it fights and wins unaided. Supplies
      since *Supply* (`0.18.3`); it spawned four troops before
- [x] **Escrow economy** (progress-scaled, 20% floor, 50% completion bonus),
      exactly §11.5's table. Losing at 60% keeps 60%; bailing at nought still
      pays the floor — *"effort is never wasted; only cynicism is"*
      **See it:** ✅ a lost siege reads `the wall is carried. 22 of the way, and
      you keep 27`
- [x] **Adversarial aberrations across all four surfaces.** §8.1's model is
      closed: logs and world state shipped in Phase 1, and `tower::assault` adds
      **script text** and **trigger clocks**, siege-only and on a resolved round.
      A retimed spell reads perfectly and stops keeping up; misdirection, never
      theft, so `purge` repairs
      **See it:** ✅ `ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend;
      hold; hold; hold; hold; verify; meditate 60"` — *"something got past the
      wall"*, then the audit names the spell it touched
- [x] **The cadence, and it is the single most load-bearing number in the
      domain.** `siege::CADENCE` stands in for §5.3's provocation. Back to back,
      sieges paid thirty-three times the flagship; `orbs-balance` found it, and
      the cadence, not the escrow, was the thing to tune (§19)
      **See it:** ✅ `cargo run -p orbs-balance -- run besieging --ticks 7200`
      reads 0.114–0.134 across seeds, under clarity and near warding
- [x] **Quintessence — a pool spent to pledge dice** (`0.8.15`). §11.5's mana,
      built at last: a fixed pool granted on `defend`, sized by integrity and
      raised by the ley line, never regenerating (§14). The decision now bites;
      the coffer publishes only what it can pay for; `aim` lets a spell ask the
      odds the board draws
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; pledge d20
      buckler; pledge d8 line; pledge d6 succour; survey coffer"` — three pledges
      take 5, 2 and 1 of 24, and the board's coffer row prints what each costs
      beside what is left. Run three rounds of it and the fourth is refused:
      *"d20 would take 5, and you hold 0"*
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend sanctum; meditate 3600; attend
      bailey; defend; survey coffer"` — a worn tower opens with **12 against a
      kept tower's 24**, which is §19's deferred integrity → siege coupling
- [x] **The far side of a comparison grows an arithmetic** (`0.8.16`). Reverses
      §19 twice, struck with a pointer: quintessence needs *is what I hold more
      than this costs*. `[double] <place> [has <thing>] [plus n]` — words, one
      operator, no brackets; `strict` derived from the grammar; `plus` reserved
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=110x40 ORBS_DUMP="attend bailey;
      scribe maths"` with `ORBS_EDIT` writing the three forms, then `interpret` —
      each reads back verbatim, which is the surface a swallowed term shows up on
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend bailey;
      recall scripting"` — the manual gains a sixth question shape
- [x] **`sparingly`, the solver the arithmetic was built for** (`0.8.16`). The
      only shipped spell that uses `for each die` or `double`, and the only one
      that declines to pledge on purpose. Its guard is a double negative, since a
      comparison against a place is strict
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; debug_spawn warding 6"
      ORBS_THEN="invoke sparingly; meditate 600; sift pledge bailey.log"` —
      **twelve pledges, four rounds, 32 spent**: the pool of 24 and what it
      regenerated between rounds. Then it fights on without dice rather than
      being refused. It read nine and 24 when it spawned six troops, before
      *Supply* (`0.18.1`)
- [x] **An `xhigh` review of the whole siege, and the two it found by playing**
      (`0.8.17`). Eleven findings, seven of them a comment, doc or count no
      longer true. The two found by playing: the enemy broke spells rather than
      lying to them, and a die's price did not exist until the first bailey verb
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; survey d20"` — the
      price is 5 with no siege ever opened, where it read *"the d20 holds
      nothing"*
      **See it:** ✅ six rounds on seeds 3, 11, 17 and 42 print **no** `line n:`
      complaint, where a corrupted `part` or `is <state>` printed three
- [x] **Scripting the siege, tested as its own question** (`0.8.18`).
      `tests/scripting_the_siege.rs` — eleven claims that a person can write a
      siege spell: every declared reading reachable, the arithmetic against
      fixed numbers, and a spell telling a win from a loss
      **See it:** ✅ `cargo test -p orbs-sim --test scripting_the_siege` — eleven
      claims
- [→] **Pane synergies — deferred to Phase 12, not cut** — an enhancement that
      wants multiplexing
- [→] **Traits, `lens/observed/`, and the eldritch renderer** — plumbed with no
      producers; Breadth-scale content work
- [→] **Unattended-siege backlog, dispersal and decay**, and difficulty tiers —
      all wait on provocation, §5.3's trace, which is unbuilt

**Scarcity: the arsenal, finally spent.** Six rooms' output becomes a decision
under known risk.

## Breadth

*Was **Phase 13a**.*

- [ ] All 7 domains
      **See it:** run a tower where every branch of the tree does something
- [ ] Discovery / research loop — **and the archive's minigame with it.** §10
      makes decipherment bespoke, and it was considered for the head of Phase 1
      alongside brewing and **deliberately left here** — DESIGN.md §19:
  - Its "see it" line *is* this item. Gaining a verb needs `Verb::ALL` to
        stop being a fixed sixteen, the synonym table to stop being `const`,
        `execute::is_live` to stop being a `const fn`, and the boot tutorial to
        read all three dynamically — plus §18's unstarted naming pass to have
        named the verb that arrives
  - §10 pre-authorises the cheap version — *"decipherment becomes mostly a
        resource sink with occasional authored set-pieces"* — and §7's only
        depiction of the verb is exactly that: read the fragment, learn `grep`,
        the grimoire grows. A bespoke puzzle is an ambition **increase**, not a
        debt, and it is better bought when it is known what it gates
      **See it:** `research` a fragment and gain a verb you did not have
- [ ] Full drift
      **See it:** return after a long absence to scripts that have gone subtly wrong
- [ ] Offline progression + its unlock
      **See it:** quit, come back tomorrow, and read what happened while you were out
> **~~Shared-engine extraction~~ — moved to Phase 9, not done.** It was written
> *twice*, here and there, with contradictory gates: Phase 9's asked for *"one
> recipe table drives two domains"* — new behaviour — and this one for *"the game
> plays identically before and after."* Phase 9 now carries both halves as two
> boxes, a refactor whose gate is that nothing changes and a behaviour box beside
> it, because merging them makes the refactor unverifiable. No box is ticked by
> this move; the work is open, one phase earlier. §19

- [ ] **The arsenal is typed on more than one axis.** Every item resolves to a
      global scalar today, so the only question is which number is largest —
      which is how `clarity` became *"strictly the best thing in the arsenal…
      which collapses the decision this whole file exists to create."* An optional
      `area` on an arsenal row scopes it to the wall the siege already types:
      `warding` to the buckler, `mending` to the succour, a troop to the line or
      sortie. **Repricing cannot fix a one-axis system** — some number is always
      largest — and this is the fix that does not nerf anything (§19)
      **See it:** `defend` until the telegraph reads `volley`, then spend a
      `Line`-scoped item — refused, because *"a volley cannot be answered."* Not
      `warding`: the buckler is *"the only area that is never wasted"*
- [ ] **A settled siege drops a `sigil`, and the lens reads it.** The one new
      material in the whole web. Reading it writes trait knowledge to
      `lens/observed/`, which §5.2 already designs, and that names the next
      siege's telegraph before it lands — so fighting compounds rather than merely
      paying, and **the laboratory stops being a cut vertex**
      **See it:** fight one to a settlement, then `peruse lens/observed/<name>` —
      `peruse`, not `cat`; there is no `cat` verb
- [ ] **Garrison upkeep, and the driver change it cannot ship without.** A troop
      spent per round is what gives the menagerie's *rate* a reason to exist.
      It answers the perishable arsenal's first objection — a decrement is not a
      timestamp, and `Stock::Counted` supports it — and **inherits the fourth
      verbatim**: *"it would have starved the `besieging` balance policy, which
      stocks once at setup."* That pin is the most fragile number in the table.
      The penalty is escrow fraction and garrison strength, **never integrity**,
      per §11.5 invariant 5
      **See it:** `orbs-balance -- sweep --policy besieging --why` still additive
- [ ] **The durations spread.** §11.5 authors four classes across 0–600 ticks and
      the shipped recipes run 6–56 — this document already calls that *"the number
      to attack."* Domains on different clocks is what lets the ten-minute player
      and the overnight player optimise different rooms rather than the same room
      more slowly. A tuning pass, not a design change
      **See it:** `orbs-balance -- sweep --hours 1 --why` across every policy, with
      the archive's loop and the laboratory's an order of magnitude apart
- [ ] **The Ley Line keeps offering choices past its last station.** The forks
      stop at `at = 10000` and the weave then draws a full bar and a climbing
      number. Twenty-five Mastery stations sit outside the cap so the plateau is
      real rather than a wall — what ends is *choice*. Repeating the three lanes
      at widening intervals answers it with no new system
      **See it:** pass the last authored station and be offered a fork anyway

## Remote hosts

*Was **Phase 13b**.*

- [ ] Host filesystems, verbs, infiltration
      **See it:** `connect` somewhere hostile and navigate a tree that is not yours
- [ ] Trace amplification
      **See it:** stay too long and get hunted at home for it
- [ ] Ship-quality `orbs-tui` *if the schedule allows* — **cut-line item 3**
      **See it:** play a full session in a terminal and miss only the tube

## Engine upgrade

*Was **Phase 13c**.*

- [x] Bevy version window, isolated from new-system work (`0.17.17`) — Bevy
      0.20, `bevy_ecs` 0.20, Rust 1.98, `rand` 0.10, `toml` 1, `dirs` 7, the
      way `court_wizard` moved the day before. DESIGN.md §19, *Bevy 0.20*
      **See it:** `ORBS_CAPTURE=1` with `motion = "off"` in a scratch settings
      file, tube on and off: the stills match 0.19's pixel for pixel.
      `orbs-balance sweep --ticks 7200` matches byte for byte, and
      `rng::every_stream_draws_what_it_always_has` holds the values. The POST
      card reads `bevy 0.20.0` and `rust 1.98.0`, which is all `dumps.sh` moves
- [ ] **`burn` 0.22**, held out of the window above. The burn on Bevy's `wgpu
      30`, and a rewrite: no backend generic, the shipped weights converted to
      its format, and `ndarray` swapped for its new CPU backend with inference
      measured again. Until then a `train` build compiles two `wgpu`s
      **See it:** `cargo tree -d | grep -E '^(wgpu|naga)'` empty again, and
      the augury trials reading what they read before
- [ ] Green on all three platforms
      **See it:** launch the built artifact on each, not just CI green

---

## Onboarding + demo

*Was **Phase 14**, and much of the file defers to it by that name.*

- [ ] Diegetic apprenticeship, polished. Minute one's prompt and navigation
      are [the installation](#the-installation) (`0.17.8`); the first potion
      and the first spell are still this item's
      **See it:** a non-terminal player reaches hour two unaided, watched
- [ ] Progressive reveal throughout
      **See it:** nothing on screen at minute one that has not been earned
- [ ] Soft ending
      **See it:** reach it, and want to keep playing anyway
- [ ] **Demo + capsule + trailer** — the trailer leads on CRT *motion*, which a
      static capsule cannot carry
      **See it:** hand the demo build to someone cold and watch them play
- [ ] Wishlist target set; revisit the $4 price against actual content volume
      **See it:** a real session's length checked against the price

## Ship

*Was **Phase 15**, and much of the file defers to it by that name.*

- [ ] **Colour-vision filters and a real monochrome mode** — ⚠ **this item now
      carries a promise that used to be carried by a theme.** §4's monochrome
      phosphor existed so *"a player with a colour vision deficiency loses
      nothing"*, and `palette` tested that its base carried no hue at all. Two
      decisions retired that: the athanor burns one orange ramp on every tube,
      and materials carry a tint that hints at what is inside an instrument
      (DESIGN.md §19). Monochrome is now a grey *aesthetic*, not a guarantee.

      A theme was the wrong place for the guarantee anyway — it made the
      accessible option also an aesthetic choice, so a player who wanted amber
      had to give up the accommodation to get it. A **filter** is orthogonal to
      the theme, which is what an accommodation should be.

      ~~Three filters (protanopia, deuteranopia, tritanopia) plus a true
      greyscale mode, applied as a post-pass over the composited frame in the CRT
      shader — *after* the phosphor and before the barrel.~~ **Both halves of
      that are superseded, and the greyscale half is built.**

      **The three correction filters are not being built**, on measurement rather
      than on schedule. Daltonisation degrades this palette's accent separation
      in eleven of twelve theme × deficiency combinations, because the triad is
      solved in *luminance* and the correction works in *hue*; DESIGN.md §19
      carries the numbers. What was kept is the **simulation**, as
      `render::deficiency` under `#[cfg(test)]`, pointed at
      `palette::the_accent_triad_survives_every_deficiency` — which failed on its
      first run and is why monochrome's `cost` moved.

      **"Before the barrel" is not a place that exists.** `crt.wgsl` applies the
      barrel *first* — it computes the sampling coordinate — and the grille, the
      aberration and the flash all put hue back into a pixel that had none. The
      pass is **last**, and it is a separate pass rather than part of the tube
      because `crt.wgsl` early-returns when the tube is off and an accommodation
      F3 could switch off is §19's own anti-pattern.

      Still owed: the material tints are allowed to collapse under greyscale —
      they are a convenience over `survey`, never the only carrier.
      ~~A settings screen to reach any of this without an environment variable is
      Phase 15's, and until it lands there is **no in-game control** for the
      accommodation.~~ **Discharged at `0.16.5`:** `settings` → `access` → `sight`
      sets it from inside the game and remembers it, and `ORBS_SIGHT` still
      outranks it so a See-it line stays reproducible.

      **See it:** set each filter in turn with the laboratory on screen and a
      breach in the transcript; the athanor still reads as fire, the accent
      triad still reads as three things, and greyscale mode has no hue anywhere
- [ ] Accessibility pass (see DESIGN.md §14)
      **See it:** play a full session with the CRT off, at every toggle
- [ ] Screen-reader siege mode (ticks advance on player input)
      **See it:** survive a siege with the screen off, by ear
- [x] **Sticky skip and persisted CRT-off** — **moved here from Phase 0.5**,
      where it was the one open item and never belonged: §4 asks for skip to be
      *"a sticky setting, not a per-launch keypress"*, and the keypress was the
      honest half-measure until there was anywhere to persist a setting.
      ~~No `serde`, no `toml`, nothing in the workspace serialises anything yet~~
      — that reasoning stopped being true when the save landed, and both shipped
      at `0.16.5` with the threshold's `habits` and `tube` pages. Every function
      key that changes something now writes it down, so `F3` and the `crt` row
      are one setting seen twice
      **See it:** turn the tube off, relaunch, and it is still off; or
      `settings` → `habits` → `skip on`, relaunch, and the boot sequence is gone
- [x] **Reduce-motion as a setting of its own** — done at `0.17.9`, in
      [*The orb made yours*](#the-orb-made-yours), with the health warning. What
      follows is the reasoning it was held to. Split out of the item above,
      which shipped without it **deliberately**. Motion is gated on the tube
      today (`motion::advance` stops the crossings, the fire and the flare when
      any tube state is off) and §14 is served by that. A separate row would be
      a switch a player could turn *on* while the tube said off, which is the one
      state §14 must not reach — so it needs the crossings separated from
      `Passing::enabled`'s one input first. §14's health warning goes here too,
      and is no longer urgent: with the strike cut, nothing in the game flashes
      **See it:** ask for no motion with the tube *on*, and get none
- [ ] **Remapping** — the rest of *"Options, remapping, all toggles"*; the
      options and the toggles shipped at `0.16.4`–`0.16.5` and `0.16.10`. This
      half needs a binding table, a conflict rule and a capture surface, none of
      which the menu needed to be good, and was scoped out of the threshold on
      those grounds
      **See it:** rebind every key and play with the result
- [ ] Steam integration and depot
      **See it:** install from Steam on a clean machine and launch it
- [ ] Polish
      **See it:** a full playthrough on the shipping build, start to end

---

## Standing — work that rides with the content

**Nothing here is scheduled, and nothing here may block a phase.** That is the
whole reason the section exists.

An item that *cannot close* does not belong in a numbered phase. Sitting in one,
it does not track work — it holds the phase open for ever, and a phase that can
never be finished stops being a plan and becomes a list. This has now happened
twice: the settings item sat open in Phase 0.5 waiting for somewhere to persist a
setting, and the upgrade tree sat open in Phase 1 waiting for content that no
Phase 1 item produces. The first was moved to the phase that builds what it
needs; the second had no such phase, because it is not waiting on one thing — it
is waiting on **all of them**.

So: when an item is deferred because a *later phase* builds what it needs, move
it to that phase. When it accretes instead — a little more of it true with every
content item, never all of it true — move it here.

- [ ] **The upgrade tree grows with the content.** The surface is built
      (`weave`, Phase 1) and a node is a row in `progression.toml` plus a line in
      `prose.toml` — **no Rust**. So the tree is not a thing to schedule; it is
      what every later content item leaves behind. A domain that ships brings the
      tier that unlocks it, a recipe brings its own step, and the curve fills in
      as there is something to put on it.

      ✅ **The first real node** landed at `0.5.4` — `take` became a grant, with
      the mutator, the `Submission` variant and the queued effect it was named
      as needing.
      ✅ **The first content gate and the derived scale** landed at `0.10.1`, in
      Phase 10: *"brew this to open that"* is a mastery station's `opens`, and
      the bar is measured against the line's last station.

      Still waiting on content, and named so they are not rediscovered: the
      **fractional slot charge** (it prices multiplexing, so it needs
      multiplexing), and the **speed** upgrade to replace struck invariant 3,
      which is Phase 10's `haste_1`.
      **See it:** cross a threshold, be offered two nodes, take one, and watch
      the other close — rather than a level that arrives on its own

- [ ] **Trace tuning accrues with the content.** §18's first blocking question,
      re-pointed here when Phase 1 closed and it did not: accrual rates across
      all three of §5.3's sources, the nuisance-rate composition rule, the
      ceiling, and the provocation threshold. Every one of those is a number
      *about* content, so each domain that ships moves it and none of them
      finishes it — production heat is a rate per thing brewed, and there is no
      state in which the last thing has been brewed.

      **`orbs-balance` is the instrument, not a playtest.** §5.3 calls trace *"a
      player-controlled difficulty dial"*, which means the thing to tune is a
      *curve* rather than a value, and a curve is swept rather than felt.
      **See it:** produce hard for an hour and watch the nuisance rate climb,
      then stop and watch it settle — the dial moving in both directions

- [ ] **`orbs-balance` learns to see goods, one edge at a time.** The harness
      measures **experience a tick** and the whole of §19's dependency web moves
      **goods** — so the instrument is currently blind to the economy the web
      builds, and the blindness looks exactly like stability, which is the failure
      CLAUDE.md already records for a domain shipped without a block in
      `scripts/dumps.sh`.

      It belongs here rather than in Phase 12 because it accretes: a goods-a-tick
      readout is one change, but *a policy per edge* is not a thing that finishes
      — every edge that ships brings one, and there is no state in which the last
      edge has shipped. The same argument the upgrade tree makes above.

      **The two are different questions and both are needed.** A rate says whether
      a room produces; a *balance* says whether the room downstream is starving.
      §19's exit for Phase 12 — one good, two consumers, three answers — cannot be
      read off an experience rate at all.
      **See it:** `cargo run -p orbs-balance -- sweep --why` with a goods column
      beside the experience one, and a consumer that runs dry named in it

- [ ] **Scrappy `orbs-tui` is a little more true with each frontend change.**
      Moved here when Phase 1 closed. It is a **dev tool, not a product** — §15
      gives it *"no parity, polish, or support obligation"*, and its job is to
      prove the `orbs-render` boundary is real and to give the parser and balance
      work a no-GPU, no-window, instant-startup, trivially scriptable target.
      Ship-quality `orbs-tui` is a different item and stays in Phase 13b as
      cut-line item 3.

      It accretes for a specific reason: **every new surface either goes through
      the Frame or quietly does not**, and the terminal build is the only thing
      that can tell the difference. A painter that reaches into the Bevy crate
      compiles, tests green, and looks right in a dump; it simply cannot be drawn
      anywhere else. So this item is never *done* — it is one more screen true
      each time a screen is built.

      **The dividend is that a screen can be driven rather than described.**
      `ORBS_DUMP` is a still photograph and now needs eighteen environment
      variables to pose; a terminal build is the running game under `tmux
      send-keys`, which is a test harness for everything that moves.

      ✅ **The boundary is real, and `orbs-shell` is what made it so** (`0.3.1`).
      ~9,100 lines of painters and surfaces moved out of the Bevy crate into a
      shell both frontends share, under a gate of 56 byte-identical dumps. §19
      records what moved and why `bevy_ecs` is allowed there.
      ✅ **It draws** (`0.3.2`) — the same `Frame`, in ANSI, with a shadow-buffer
      diff and a startup probe for ambiguous glyph widths.
      ✅ **It plays** (`0.3.3`) — a clock, the shared key table, and the
      step-before-drain rule a walk's replay depends on.
      ✅ **All four surfaces** (`0.3.4`) — editor, weave screen, maze, unfurled
      transcript.
      ✅ **Every key the Bevy build binds** (`0.3.7`) — `F4` `F5` `F6` `F7`
      `F10`, and `PgUp`/`PgDn` without `unfurl`. It shipped once without them
      while the border drew `F4 deep` on every frame; the rules now live in
      `orbs_shell::shortcuts` so a second copy cannot go missing.
      ✅ **The fire burns orange** (`0.3.8`) — the ramp ran `dark-red → white`
      against §19's *"orange on every tube"*. Four heats out of two ANSI hues,
      using the weight axis. `scripts/ink.py` is what found it and is the only
      instrument in the project that can see a resolved colour.
      ✅ **`quit`** (`0.3.9`) — the way out is a word, in both builds. §19
      records the tower-wide count going 22 → 23 and why `leave` and `exit` are
      refused.
      ✅ **Resizing reflows instead of scrambling** (`0.3.10`) — the shadow
      buffer told the truth about itself and a lie about the screen, so every
      cell the new frame left blank kept its old glyph.
      ✅ **The game is played, as a test suite** (`0.3.11`) — 92 scenarios that
      type at a real terminal and read the screen back, plus the first unit tests
      `surfaces.rs` has ever had. It found the width probe measuring a glyph that
      cannot vary, and the maze map running a whole second behind the arrow keys.
      ✅ **It boots** (`0.3.12`) — §4's sequence runs here too, naming `crossterm`
      where the other build names `bevy`. It was skipped on an "instant-startup"
      argument that was really about the development loop, which `ORBS_BOOT=0`
      already answered. The world's clock stays still through it, which is a
      determinism rule and not a nicety.
      ✅ **The input paths nobody had pressed** (`0.3.21`) — a second review,
      run after `0.3.13` shipped. Backspace was dead on any terminal sending
      `^H`, key repeat was filtered out entirely, the boot card ignored the
      "too small" floor, a signal left the terminal in raw mode, and `ORBS_DUMP`
      drew the opposite focus mode to the game. Plus the duplication behind
      them: three key tables shared, the tint rule made one, `plugin.rs` split
      to registration only. §19 records the set.
      ✅ **Reviewed, and the instruments fixed first** (`0.3.13`) — nine defects,
      several of them in the suite rather than the game: assertions satisfied by
      the command they typed, colour scenarios decoding the wrong columns, a
      spell that could end the session, a rail calling sigils seconds, a ward
      sheet vanishing at the authoring floor, and three keyboard-parity gaps
      whose comments all claimed parity. §19 records the set.

      What is left is what accretes: every screen built after this either goes
      through the Frame or quietly does not, and this is the only thing that can
      tell the difference.
      **See it:** `scripts/tui.sh start`, then
      `scripts/tui.sh type 'attend laboratory'` and `scripts/tui.sh see` — the
      tower, played and read back, with no window anywhere
      **...and driven:** `scripts/play.sh` plays the whole game in half a minute;
      `scripts/play.sh brewing::` plays one room of it
      **...and it opens properly:** `cargo run -p orbs-tui` — a dark tube, then
      the card printing its own name, then the tower

---

## Cut line — decided in advance

Drop in this order under schedule pressure:

1. Steam Workshop (already deferred)
2. OS-window pane detaching (already deferred)
3. Ship-quality `orbs-tui` (boundary and dev-tool build kept regardless)
4. Remote hosts → read-only scrying feeds
5. Enchanting, then Summoning (extraction task cut with them)
6. Screen-reader support slips further post-launch (architecture retained)
7. Multiplexing capped at two panes

**Never cut:** the parser, the script engine, the aberration system, one excellent
siege, onboarding.

---

## Live open questions

Tracked in DESIGN.md §18. **Nothing currently blocks Phase 0.**
