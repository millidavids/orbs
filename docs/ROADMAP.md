# O.R.B.S. — Roadmap

**Status tracker. Derived from [DESIGN.md](DESIGN.md) §15, which is authoritative.**
If the two disagree, DESIGN.md wins and this file is wrong.

> **Every item carries a "See it" line, and it is a completion gate — not a note.**
> No work item in any phase is done until a person can reach it from the running
> game. An item without a See it line is not started; an item whose line does not
> work is not finished, however green its tests are. DESIGN.md §15, §19.

Last updated: 2026-08-03 · **Phase 0 in progress**

---

## Shape

Solo, commercial, Steam. **28 months of work against a 24-month target** — that
gap is deliberate and is the cut line's job to close, not a scheduling error to
hide. Release posture: demo first, then full 1.0. No Early Access.

| Phase | Months | Words | Status |
|---|---|---|---|
| 0. Vertical slice | 4 | ~3k | 🟡 In progress |
| 1. Core loop | 5 | ~15k | ⬜ |
| 2. Siege | 4 | ~15k | ⬜ |
| 3a. Breadth | 4 | ~18k | ⬜ |
| 3b. Remote hosts | 3 | ~12k | ⬜ |
| 3c. Engine upgrade | 1 | — | ⬜ |
| 4. Onboarding + demo | 4 | ~20k | ⬜ |
| 5. Ship | 3 | ~5k | ⬜ |

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

### Playability status of what is already built

The items below were built in architectural-layer order and are tested,
reviewed, and green. Much of it the **game did not call**. This table is the
scope of the retroactive gating pass, and it is what "verified" versus
"asserted" looks like written down.

The prompt closed three rows; the retroactive pass has closed four more.
What is still ❌ or ⚠️ is what remains of it.

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
| Fidelity tiers | ✅ tier and grid in the border; drag the window | — |
| `Speech` linear stream | ❌ captured 60×/s, never surfaced | Retroactive pass |
| CP437 repertoire enforcement | ✅ the prompt refuses what it cannot draw | — |
| Parse instrumentation (`ParseLog`, TSV) | ❌ | Retroactive pass |
| `sift` / record filtering | ✅ `sift <pattern> orb.log` | — |
| Eldritch + tampered presentation | ❌ | Retroactive pass |
| Per-subsystem RNG streams | ❌ nothing rolls yet | **Phase 2** — honest deferral, see below |

**Where a gate is not yet possible, it says so.** Per-subsystem RNG streams
cannot be *seen* until something rolls against them, and the first thing that
does is aberrations in Phase 2. Inventing a debug affordance nobody will maintain
would be worse than naming the phase that gates it.

### Work items

- [x] **Workspace + determinism spine** — five crates; `orbs-sim` on `bevy_ecs`
      standalone; ChaCha8 per-subsystem RNG streams; single-threaded `SimSchedule`;
      `Sim::step()` as the sole entry point. 13 tests green, clippy clean at
      `-D warnings`. Verified `bevy_ecs` unifies to one version across the
      workspace, so types match across the sim/frontend boundary
      **See it:** ✅ `meditate 30` and watch the tick jump; `status` reports the
      seed; the border carries both live
- [x] **`orbs-render` Frame boundary** — cell buffer, semantic styling, layout.
      `Style` carries role/intensity/presentation and never a colour; every frame
      carries a `Speech` linear stream captured at paint time; glyphs are bounded
      to the CP437 repertoire; `Fidelity` reproduces §9's tier table exactly and
      `ScreenLayout` reproduces its 60×15 four-pane figure. 100 tests green,
      clippy clean at `-D warnings`, rustdoc clean at `-D warnings`.
      `cargo run -p orbs-render --example screens` renders the §4 boot report and
      a multiplexed siege for eyeballing
      **See it:** ✅ the game draws two panes from a real `ScreenLayout`, and
      `F4` switches Deep ↔ Wide — which raises fidelity a step and visibly
      changes both the glyph size and the pane count. ⚠️ the linear stream is
      still unsurfaced
- [x] **Bevy frontend skeleton** — window opens (Metal verified), `SimPlugin`
      drives `Sim::step()` from `FixedUpdate` at 1 Hz per §5.0, `Screen` resolves
      window pixels to a fidelity tier and grid. `orbs::shell::preview` logs the
      painted Frame each tick as an interim stand-in until the cell renderer
      lands. **`cargo build -p orbs` is now part of the gate at every step**
      **See it:** ✅ `cargo run -p orbs` — the window opens and the sim ticks
- [x] **Parser + instrumentation** — the 16 slice commands (DESIGN.md §6.1) across
      three registers; full input/resolution/candidate-score capture with export.
      Deterministic NLU, no RNG: normalise → fuzzy match → resolve against the
      live scene → score → disambiguate → suggest. `analyse()` keeps every scored
      reading so the gate can cluster failures by cause; `ParseLog::to_tsv()`
      exports one row per candidate. `cargo run -p orbs-sim --example parse -- -i`
      to type at it. 180 workspace tests green
      **See it:** ✅ type at it. ⚠️ the instrumentation export is still gated by
      the retroactive pass
- [x] **Naming pass** for the slice's 16 commands — run against the implemented
      vocabulary, not by eye. Found a canonical collision between the two core
      brewing verbs (`decoct`/`decant`, 667), a `dec-` prefix shared three ways,
      and four names over the length rule. `decant`→`siphon`, `decipher`→`divine`,
      `inscribe`→`scribe`; old words kept as synonyms. Canonical collisions 1→0,
      prefix ambiguity 1→0, synonym collisions 7→3 all claimed. Now enforced
      continuously by `crates/orbs-sim/tests/naming.rs` — DESIGN.md §6.1, §19
      **See it:** ✅ type any register at the prompt and read the canonical echo
      come back
- [x] **Cell-grid text renderer** — glyph atlas + single-mesh quads, integer
      fidelity tiers. One draw call at any grid size; **227 µs to rebuild the
      worst-case 160×45 grid in release**, 1.4% of a 60 Hz frame, so §4's
      cell-index-texture alternative is not needed. No custom shader: the atlas
      carries coverage in alpha and stock `ColorMaterial` multiplies by vertex
      colour. Camera fixed to *physical* pixels so integer scaling survives to
      the framebuffer. Three phosphor themes, contrast-solved rather than
      eyeballed. `ORBS_CAPTURE=1 cargo run -p orbs` screenshots it
      **See it:** ✅ it is what you look at. `F2` cycles themes; drag the window
      and the tier and grid in the border change
- [x] **CRT port** — barrel, scanlines, aperture grille, vignette, chromatic
      aberration, flicker, rounded corners, phosphor glow, desaturation, flash.
      **The shader ported; the surrounding Rust did not exist to port** — Bevy
      0.19 removed `render_graph` entirely and a post-process is now a system in
      the `Core2d` schedule, so ~2,100 lines became ~300. Scanline and grille
      frequencies are cell-derived per §9, so a fidelity-tier change cannot beat
      against the glyph stems. Fully disableable (§14) and §4's peak-threat state
      is reachable on F3. The pass registers `.in_set(Core2dSystems::PostProcess)`;
      an ordering edge alone leaves it unordered against the main pass and the
      blit, which made it flash. Not yet wired to world state — DESIGN.md §19
      **See it:** ✅ `cargo run -p orbs`, `F3` cycles default → peak threat →
      off. **The model item for this rule** — it caught its own bug in ten seconds
- [x] **Structured-record output model** — linear, semantic, presentation separate.
      `Records` is an arena stream; a `Record` is a borrowed view carrying a kind,
      a role, and named fields whose numbers stay numbers. Four consumers read it
      and none is privileged: `RecordView` draws it, `Record::speak` linearises
      it, `Records::sift` filters it, the harness measures it. §3's corruption
      exemption is enforced by construction — the only presentation accessor is
      already filtered, so eldritch cannot reach a log line and a sabotage tell
      still can. The model lives in `orbs-render` because it is the *interface*
      between the crates; `orbs-sim` gained the dependency and
      `crates/orbs-sim/tests/boundaries.rs` now holds it to rules 1, 2, and 8.
      Fields split into content and annotation, so a machine tag can classify a
      record for a view without being read aloud or drawn. The parser is the
      first producer, and every record it emits says which outcome it is — a
      selectable candidate is not a suggestion, and a forced echo is not a clear
      one. 292 workspace tests green; the third screen of
      `cargo run -p orbs-render --example screens` is one stream drawn three
      ways — DESIGN.md §7, §19
      **See it:** ✅ every line on screen is a record. `sift <pattern> orb.log`
      filters them. ⚠️ the eldritch/tampered presentations remain
> **Everything below is ordered by playability, not by layer** — DESIGN.md §15.
> The first six items were built in layer order and left ~10,000 lines the binary
> called under 40% of. **No item below is done until the "see it" line works.**

- [x] **The prompt** — a real command line in the running game. Not a
      feature in its own right: it is the instrument every item below is verified
      with, and the first thing to call the parser, the record model, and the
      views from the actual binary. Plan independently reviewed before starting;
      decisions in DESIGN.md §19, ordering rationale in §15. Scope:
  - Line buffer driven by `KeyboardInput::text`, **not** `logical_key` — the
        spacebar is `Key::Space`, so a `Key::Character` implementation silently
        loses it and every multi-word command becomes untypeable. Filter through
        `orbs_render::is_renderable`: `text` carries control characters (Enter is
        `"\r"`), and an unrenderable char occupies a column and draws nothing
  - A **viewport** on the input line. At the 80×22 floor the prompt leaves 72
        cells; past that `put_str` clips and `set_cursor` refuses an off-grid
        position, so the player types into a dead line with no caret
  - **Rebind Escape**, which currently quits, and ship a replacement quit in the
        same change — Escape is "clear the line" muscle memory
  - An **outcome-aware line view** in `orbs-render`. `RecordView::lines()` draws
        `meditate` as `meditate count` and an unresolved input as four bare
        words; the `Outcome` annotation is unreachable by it. Marker glyph +
        intensity derived from the record — §19
  - Scrollback and `Scene` live in **`orbs-sim`** behind `Sim::submit()`, which
        resolves immediately and queues the `Intent` for the next `step()`
  - A `prompt` screen in the `screens` example, and a **headless keystroke
        test** — `MinimalPlugins + InputPlugin`, write a `KeyboardInput`, assert
        on the resulting records. §19's whole lesson is that the parser had never
        received a keystroke; this is the test that fixes that permanently
  - Out of scope, stated: selecting a numbered ambiguous candidate, history,
        left/right editing, paste, IME, line wrapping in the scrollback
      **See it:** `cargo run -p orbs`, type `look around` → `survey` comes back on
      the tube; type `xyzzy` → suggestions; type `meditate` → it asks for a count
- [ ] **Retroactive playability pass** ← *immediately after the prompt, before any
      new Phase 0 work.* Everything in the status table above marked ⚠️ or ❌ gets
      a hand. Not a cleanup task: until each of these runs, the code behind it has
      been asserted rather than verified — DESIGN.md §15, §19
  - **Fidelity tiers** — resize the window and watch the grid re-derive. A
        readout of tier, cell scale, and grid size on screen, so §9's table is
        something you can walk through with a mouse instead of read
  - **Frame boundary / layout** — a real multi-pane screen driven by
        `ScreenLayout`, and the Deep ↔ Wide focus switch (§9) on a key. The
        `screens` example already proves both; the game has never drawn either
  - **`Speech` linear stream** — a key that shows it on screen beside the frame
        it describes. §14 makes it first-class and nothing has ever displayed it;
        the parity property the example asserts should be visible live
  - **CP437 enforcement** — the prompt rejects unrenderable input at the buffer
        rather than drawing an invisible hole. Makes the repertoire a thing you
        can bump into
  - **Parse instrumentation** — a key that dumps `ParseLog` to TSV beside the
        binary, and shows the last resolution's candidate scores. §6's *"the
        parser must explain itself"* currently explains itself to a file nobody
        writes
  - **`sift` over the scrollback** — the first pipe stage, on real records.
        `sift <pattern>` filters what is on screen. Proves §7's "operates on
        records, never rendered text" with a hand instead of a test
  - **Eldritch + tampered presentation** — a debug key that flips a record's
        presentation, so the three font faces and §3's corruption exemption are
        visible. The exemption is currently proven only by a `println!` in an
        example
  - **Determinism** — `meditate <n>` advances ticks visibly; seed and tick on
        screen; relaunching with the same seed reproduces the same world
      **See it:** every row of the status table above reads ✅, or names the phase
      that gates it
- [ ] **Dev ergonomics** — `bevy/dynamic_linking`, fast linker (lld/mold). Moved
      up: it is now paid back every time the prompt is exercised by hand
      **See it:** the edit → `cargo run -p orbs` → typing loop gets visibly faster
- [ ] **Brewing + archive** — the two starting domains, both thin
      **See it:** `cd /alembic`, `ls`, `decoct clarity` — real files, real state,
      records drawn by the pane rather than by an example
- [ ] **Log-poisoning sabotage** on brewing logs, via `peruse` / `sift` / `verify`
      **See it:** `peruse alembic.log` shows a tampered line you can spot by eye,
      and `verify alembic.log` names it
- [ ] **Boot sequence** — status report reflecting real world state, sticky skip
      **See it:** launch the game and the §4 report is the first thing on screen,
      reporting the world that actually exists
- [ ] **Scaffold tutorial** — throwaway, exists so the gate measures the parser
      rather than the absence of onboarding
      **See it:** a cold launch walks you to your first successful command
- [ ] **Worst-case legibility test** — tier 2 at minimum supported window, four
      panes, siege in progress, peak-threat CRT, eldritch active, tester must spot
      a single-character sabotage tell. Also establishes the minimum window at
      which tier 2 is offered
      **See it:** F3 to peak threat at the minimum window, and read a siege log
      through it
- [ ] **Run the gate**
      **See it:** eight external testers sit down and play. That *is* the gate

---

## Phase 1 — Core loop

**Exit:** a player automates a duty and feels clever; non-terminal testers are in
the loop.

- [ ] World-clock implementation per DESIGN.md §5.0
      **See it:** `meditate 30` and watch duration actions land on the right ticks
- [ ] Script engine — bind-time canonicalisation, ID-anchored referents, execution
      budget, failure taxonomy, Attention pool
      **See it:** write a `.spell`, `bind` it, walk away, come back to work done
- [ ] Remaining sabotage surfaces (world, script text, trigger clocks)
      **See it:** `verify` each of the four surfaces and have it name the tampering
- [ ] Third domain (scrying — the player's first discovery)
      **See it:** discover it in play rather than starting with it
- [ ] Minimal apprenticeship + **continuous non-terminal-user playtesting**
      **See it:** a person who has never used a shell reaches their first bound
      script while you watch
- [ ] Content data format — hot-reloadable TOML/RON, prose never in Rust literals
      **See it:** edit a line of prose with the game running and see it change
- [ ] `orbs-balance` CLI sweeping §11.5's first-pass numbers
      **See it:** a sweep's curve and a hand-played session agree
- [ ] Scrappy internal `orbs-tui` as a dev tool (no parity/polish obligation)
      **See it:** play the same save in both frontends and get the same game
- [ ] Trace tuning — accrual rates, composition rule, ceiling, threshold
      **See it:** stay connected too long and feel the pressure arrive
- [ ] Naming pass for the remaining ~35 canonical commands (≤7 chars, §6.1 rule)
      **See it:** type each register at the prompt and read the canonical echo
- [ ] Hidden-directory authoring plan (~80 fragments' worth)
      **See it:** find one without being told it is there

---

## Phase 2 — Siege

**Exit:** sieges are tense and scripts visibly matter.

- [ ] Autobattler
      **See it:** watch one resolve without touching it, and want to have prepared
- [ ] Procedural trait composition
      **See it:** meet a trait you have not seen and recognise it from its parts
- [ ] Adversarial aberrations across all four surfaces
      **See it:** a siege breaks something you automated, and you find which
      **Also gates:** per-subsystem RNG streams — the first thing that rolls
      against them (deferred here from Phase 0's status table)
- [ ] Escrow economy (progress-scaled, 20% floor, 50% completion bonus)
      **See it:** lose a siege at 60% and keep something worth having
- [ ] Unattended-siege backlog + dispersal + decay
      **See it:** miss a siege entirely and come back to a survivable mess
- [ ] Pane addressing and Focus-slot reservation
      **See it:** command a named pane mid-siege without losing the one you watch
- [ ] One siege type, end to end
      **See it:** play it start to finish and be tense
- [ ] Drift stub
      **See it:** a bound script quietly stops matching the world
- [ ] 21-pair synergy template (shared mechanics, bespoke flavour line)
      **See it:** two aberrations combine into something worse than either
- [ ] Siege type definitions + completion-fraction formula for each
      **See it:** the escrow number matches what the siege felt like
- [ ] Difficulty tiers within the progression-gated range
      **See it:** pick the harder one and feel the difference, not just the numbers

---

## Phase 3a — Breadth

- [ ] All 7 domains
      **See it:** run a tower where every branch of the tree does something
- [ ] Discovery / research loop
      **See it:** `divine` a fragment and gain a verb you did not have
- [ ] Full drift
      **See it:** return after a long absence to scripts that have gone subtly wrong
- [ ] Offline progression + its unlock
      **See it:** quit, come back tomorrow, and read what happened while you were out
- [ ] Shared-engine extraction (deferred here deliberately — the generalisation
      axis is only knowable from the second consumer)
      **See it:** the game plays identically before and after — this one is a
      refactor, and its gate is that nothing changes

## Phase 3b — Remote hosts

- [ ] Host filesystems, verbs, infiltration
      **See it:** `connect` somewhere hostile and navigate a tree that is not yours
- [ ] Trace amplification
      **See it:** stay too long and get hunted at home for it
- [ ] Ship-quality `orbs-tui` *if the schedule allows* — **cut-line item 3**
      **See it:** play a full session in a terminal and miss only the tube

## Phase 3c — Engine upgrade

- [ ] Bevy version window, isolated from new-system work
      **See it:** the game boots, draws, and plays identically on the new version
- [ ] Green on all three platforms
      **See it:** launch the built artifact on each, not just CI green

---

## Phase 4 — Onboarding + demo

- [ ] Diegetic apprenticeship, polished
      **See it:** a non-terminal player reaches hour two unaided, watched
- [ ] Progressive reveal throughout
      **See it:** nothing on screen at minute one that has not been earned
- [ ] In-world grimoire (`help` / `man`)
      **See it:** get unstuck from inside the game, without alt-tabbing
- [ ] Soft ending
      **See it:** reach it, and want to keep playing anyway
- [ ] **Demo + capsule + trailer** — the trailer leads on CRT *motion*, which a
      static capsule cannot carry
      **See it:** hand the demo build to someone cold and watch them play
- [ ] Wishlist target set; revisit the $4 price against actual content volume
      **See it:** a real session's length checked against the price

## Phase 5 — Ship

- [ ] Accessibility pass (see DESIGN.md §14)
      **See it:** play a full session with the CRT off, at every toggle
- [ ] Screen-reader siege mode (ticks advance on player input)
      **See it:** survive a siege with the screen off, by ear
- [ ] Options, remapping, all toggles
      **See it:** rebind every key and play with the result
- [ ] Steam integration and depot
      **See it:** install from Steam on a clean machine and launch it
- [ ] Polish
      **See it:** a full playthrough on the shipping build, start to end

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
