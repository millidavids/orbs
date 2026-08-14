# O.R.B.S. — Roadmap

**Status tracker. Derived from [DESIGN.md](DESIGN.md) §15, which is authoritative.**
If the two disagree, DESIGN.md wins and this file is wrong.

> **Every item carries a "See it" line, and it is a completion gate — not a note.**
> No work item in any phase is done until a person can reach it from the running
> game. An item without a See it line is not started; an item whose line does not
> work is not finished, however green its tests are. DESIGN.md §15, §19.

Last updated: 2026-08-10 · **Phase 0 and 0.5 closed. Phase 1 in progress; the §15 gate is deferred, not passed**

---

## Shape

Solo, commercial, Steam. **28 months of work against a 24-month target** — that
gap is deliberate and is the cut line's job to close, not a scheduling error to
hide. Release posture: demo first, then full 1.0. No Early Access.

| Phase | Months | Words | Status |
|---|---|---|---|
| 0. Vertical slice | 4 | ~3k | ✅ Closed · numeric gate **deferred** |
| 0.5. Interlude | — | — | ✅ **Closed, every box ticked** · settings moved to 5 |
| 1. Core loop | 5 | ~15k | 🔶 In progress |
| 2. Siege | 4 | ~15k | ⬜ |
| 3a. Breadth | 4 | ~18k | ⬜ |
| 3b. Remote hosts | 3 | ~12k | ⬜ |
| 3c. Engine upgrade | 1 | — | ⬜ |
| 4. Onboarding + demo | 4 | ~20k | ⬜ |
| 5. Ship | 3 | ~5k | ⬜ |
| **Standing** | — | — | ♾ Never closes, and blocks no phase |

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
rather than passed.** There are no external testers yet, and the author cannot
stand in for one — the gate measures whether a person's *own* phrasing reaches
the action they meant, and someone who knows the canonical vocabulary is
measuring their memory. A number that looks like this one but was produced
in-house would be worse than no number.

This is written down rather than left as a status colour because it is the one
Phase 0 claim that is **not** evidenced. Deferred to Phase 4, which is where
onboarding and the demo put non-terminal players in front of the game anyway —
and where the scripted scenario it needs will have to be written.

### Playability status of what is already built

The items below were built in architectural-layer order and are tested,
reviewed, and green. Much of it the **game did not call**. This table is the
scope of the retroactive gating pass, and it is what "verified" versus
"asserted" looks like written down.

The prompt closed three rows; the retroactive pass closed four more; brewing and
sabotage have since closed two. What is still ❌ or ⚠️ is what remains of it —
and every one of those now names a phase rather than an oversight.

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
| Per-subsystem RNG streams | ⚠️ one of six rolls | **Phase 2 for the rest.** Log-poisoning drift rolls `RngStream::Threat`, so the seeded, per-stream machinery is now exercised by the running game rather than only by tests — and the same seed poisons the same log on the same tick. The other five wait for the subsystems that own them |

**Where a gate is not yet possible, it says so.** A sidebar has nothing to
minimise until domain panes exist, and three panes at t=0 would delete the
capacity trade §11.5 spends five hours building. Inventing a debug affordance
nobody will maintain would be worse than naming the phase that gates it.

The RNG row is what that honesty is worth: it read *"❌ nothing rolls yet —
Phase 2"* until log-poisoning shipped and rolled `RngStream::Threat`. A deferral
that names its phase gets revisited when the phase arrives; one that says
"later" does not.

> **This table is derived from the code, not from the item list.** Its first
> version was written by hand from the roadmap and missed five built-and-
> unreachable subsystems outright, because no roadmap item mentioned them. Audit
> it by sweeping the public API of `orbs-render` and `orbs-sim` for anything the
> shipping path never calls, then justifying each miss. A table assembled from
> memory measures the memory.
>
> **And do not let it choose scope.** A row here says a subsystem is unreachable;
> it does not say the game should be changed to reach it. The brewing plan's
> first draft proposed three main panes at t=0 — contradicting §9's capacity-1
> opening — with the stated motivation *"both rows close."* Two of these rows are
> gated by progression that does not exist yet, and the right fix was to correct
> the rows. **A checklist is not a design.**

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
      changes both the glyph size and the pane count. `F5` swaps the session
      pane for what it says, so §14's parity is a keypress rather than a claim
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
      **See it:** ✅ type at it, then `F6` to write `orbs-parse.tsv` — one row
      per candidate, with the verb and argument scores the gate clusters on
- [x] **Naming pass** for the slice's 16 commands — run against the implemented
      vocabulary, not by eye. Found a canonical collision between the two core
      brewing verbs (`decoct`/`decant`, 667), a `dec-` prefix shared three ways,
      and four names over the length rule. `decant`→`siphon`, `decipher`→`research`,
      `inscribe`→`scribe`; old words kept as synonyms. Canonical collisions 1→0,
      prefix ambiguity 1→0, synonym collisions 7→3 all claimed. Now enforced
      continuously by `crates/orbs-sim/tests/naming.rs` — DESIGN.md §6.1, §19
      **See it:** ✅ type any register at the prompt and read the canonical echo
      come back
- [x] **Cell-grid text renderer** — glyph atlas + single-mesh quads. One draw
      call at any grid size; **227 µs to rebuild the worst-case grid in
      release**, 1.4% of a 60 Hz frame, so §4's cell-index-texture alternative is
      not needed. (Measured at 160×45, which was then the largest grid a window
      could produce; the worst case is now the fixed 120×45 — 5400 quads against
      7200, so the headroom only grew.) No custom shader: the atlas carries
      coverage in alpha and stock `ColorMaterial` multiplies by vertex colour.
      The camera letterboxes a fixed 4:3 picture with `ScalingMode::AutoMin`, so
      the mesh is emitted in virtual pixels and scaled once. Four themes — amber
      (default), green, muted violet and a monochrome light-grey —
      contrast-solved rather than eyeballed. `ORBS_CAPTURE=1 cargo run -p orbs` screenshots it
      **See it:** ✅ it is what you look at. `F2` cycles themes; drag the window
      and the `scale` row in the telemetry pane moves while `cols` and `rows`
      hold at 120×45
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
      filters them. `F7` cycles the tonal register through all three typefaces,
      and `peruse orb.log` under it shows §3's exemption holding
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
- [x] **Retroactive playability pass** — everything in the status table above
      that was ⚠️ or ❌ now has a hand on it. Not a cleanup task: until each of
      these ran, the code behind it had been asserted rather than verified —
      DESIGN.md §15, §19.

      **It found two defects no test could have.** `Fidelity::deep` was built,
      tested and never called, so Deep focus was unreachable at any window size
      and the layout could never host a second pane. And the linear view's first
      shape described a frame it had already replaced. Both surfaced from
      building a surface and looking at it — which is the entire argument for
      the gate
  - **~~Fidelity tiers~~ the picture** — resize the window and watch the grid
        *not* re-derive. A readout of cell scale and grid size on screen, so
        §19's fixed 4:3 picture is something you can walk through with a mouse
        instead of read
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
- [x] **Dev ergonomics** — measured, and the premise did not hold. The item
      assumed *"Bevy's compile time is the main friction"*, which is advice
      written for slower machines and older linkers than this project has. On an
      M4 Pro with Xcode `ld-1267`, a rebuild after touching a leaf file is
      **0.7 s** and the whole gate is about **nine seconds**.

      `bevy/dynamic_linking` is available behind `--features fast-compile` and
      is **off by default**: it saves roughly a tenth of a second, inside the
      noise, and costs a flag to remember plus a dev binary laid out differently
      from the one that ships. No fast linker is installed and none is wanted —
      installing LLVM for `lld` would cost more disk than it saves in seconds.
      Numbers and the Linux/Windows caveat in [SETUP.md](SETUP.md)
      **See it:** ✅ `touch crates/orbs/src/shell/prompt.rs && time cargo build -p orbs`
      — under a second, so the typing loop was never waiting on the compiler
- [x] **Brewing + archive** — the two starting domains, both thin. The tower is
      ECS with stable `NodeId`s; a domain's belongings are nameable only from
      inside it (§7), so `decoct` works in the laboratory and nowhere else. One
      production slot tower-wide per §11.5's opening capacity of 1, work stored
      as an interval rather than a countdown, and durations deliberately at
      §11.5's routine end because §15's gate is a fifteen-minute scenario —
      DESIGN.md §19. Numbered candidate selection shipped with them, since
      adding nouns is what makes ambiguity reachable and the gate weighs zero
      dead ends above the resolution rate
      **See it:** `attend laboratory`, `make a potion of clarity` → `decoct clarity`
      with a meter; `research` while it brews → refused, naming what holds
      the slot; `decoct nonsense` → a numbered prompt you answer with a digit
- [x] **Log-poisoning sabotage** on brewing logs, via `peruse` / `sift` / `verify`
      — a seeded roll every `DRIFT_INTERVAL` ticks poisons a log; the tell is
      §8.1's **structural** one, a record that lost its tick field, so it is
      spotted by comparing two adjacent lines rather than by reading a warning
      **See it:** ✅ `ORBS_DUMP="attend laboratory; decoct clarity; meditate 25;
      decoct warding; meditate 25; verify laboratory.log; peruse laboratory.log"` —
      `verify` says `tampered`, and in the `peruse` below it line 1 has lost its
      number while line 2 still has one
- [x] **Boot sequence** — status report reflecting real world state. Built by
      walking the tower, so it cannot go stale: one row per domain with what it
      holds and whether it is sound, plus §8.1's `bound` count. `bound: 0` is
      printed rather than omitted — a missing section leaves a player unable to
      tell *none* from *not shown* — DESIGN.md §19
      **See it:** ✅ `ORBS_DUMP=1 cargo run -p orbs`, or launch the game: the
      report is what is on screen before anything is typed, and it reports the
      tower that actually exists
- [x] **Scaffold tutorial** — folded into the boot report, because §4 wants the
      first screen to be the tower answering for itself and a separate onboarding
      screen would be a second thing to skip. It names the verbs that **work**,
      not the verbs that exist: six of §6.1's sixteen only acknowledge in Phase 0,
      and one of them, `bind`, resolves to `sift` and reports success. Offering
      those would spend the gate's most important metric on Phase 1 — DESIGN.md §19
      **See it:** ✅ a cold launch lists ten verbs and what each one takes; every
      one of them does something
- [x] **Worst-case legibility test** — the minimum window offering tier 2 is
      **1280×704**, derived rather than assumed; there tier 1 is 2× → 80×22 and
      tier 2 is 1× → **160×44**, so a one-character tell is **8 physical pixels
      wide**. The screen renders four panes, a siege, the eldritch register and
      §8.1's one-space tell at that grid, and the tell survives to the Frame
      while the eldritch pane still speaks plainly — DESIGN.md §19
      **See it:** ✅ the last screen of `cargo run -p orbs-render --example screens`.
      ⚠️ **The human read is outstanding**: peak-threat CRT is frontend
      enrichment and is not in a Frame. Size the window to 1280×704, `F4` into
      Deep focus, `F3` to peak threat, `F7` for eldritch, and read the log
- [x] **Run the gate** — ⏸ **deferred to Phase 4, not passed.** Blocked on people,
      not on code: it needs ≥ 8 external testers, at least half with no shell
      experience, over a 15-minute scripted scenario with expected-intent ground
      truth. Everything it measures is built and reachable; what is missing is the
      testers and the script. Phase 4 is onboarding and the demo, which puts
      non-terminal players in front of the game regardless, so that is where this
      belongs rather than stalling here
      **See it:** eight external testers sit down and play. That *is* the gate

---

## Phase 0.5 — Interlude

**Aesthetic, and none of it turned out to be only aesthetic.** An interlude
between the closed vertical slice and the core loop: the orb becomes a machine
that moves. Every item found a rule already pointing at it, and two found live
defects — DESIGN.md §19.

- [x] **The CRT's off switch could be flipped back on by something else** —
      `enabled` was derived as `settings != OFF`, and both `flash` and
      `desaturation` are reserved for world state, so driving either resurrected
      barrel, scanlines, grille and vignette for a player who had turned them off
      for motion sickness (§14). Explicit state now, with tests that write the
      fields directly rather than pressing the key
      **See it:** ✅ `F3` to off and it stays off. The defect was latent when it
      was fixed and is latent again now the boot strike is gone — §4's *flash on
      breach* is what will drive `flash` next, in Phase 2
- [x] **Panes arrive over time** — `ScreenLayout::transition` interpolates and
      the frontend owns the clock. Panes are born from an explicit edge
      rectangle, per mode: Deep slides in from the right, Wide unrolls downward.
      Transitional layouts suspend tiling's no-gap/no-overlap guarantees by
      design; `compute` keeps all of them. `F4` adopts the target grid instantly
      and animates only the split, because interpolating between layouts computed
      against two different grids is not a meaningful operation
      **See it:** ✅ `cargo run -p orbs`, press `F4` — the second pane grows in
      from the right instead of appearing. Drag the window across 100×28 and it
      grows and shrinks without strobing, because retargeting reverses rather
      than restarts
- [x] **Command output arrives a character at a time** — the echo, a `sift`
      result and a `peruse` dump print the way a terminal attached to something
      slow prints. 420 chars/second with a 1.4 s ceiling per burst. **Pure
      presentation, deliberately**: a modelled waiting cost would make
      `orbs-balance` simulate typewriter delays and would invert §9's parity rule,
      turning an accessibility setting into a competitive advantage. The
      detriment lands anyway, in Phase 1, when a bound script running twenty
      commands is not a person watching twenty reveals
      **See it:** ✅ type `survey` in the laboratory and watch the listing fill
      across then down. Any keystroke completes it instantly
- [x] **A real boot sequence** — opens on black, then the prompt types itself,
      the pane border draws itself a cell at a time, and a POST card prints
      `O.R.B.S.` in block glyphs a character at a time before checking off
      Blackhearth Games, Rust and Bevy. **14 s, and it does not skip** — paced to
      be read rather than to be got past: at the first pass's 4.4 s the stages
      that animate were over before they could be followed. The POST is a title
      card rather than a table, so it cannot be mistaken for §4's tower report.
      The any-key skip was removed (§19): the sequence is character, and a
      keypress skip made the first thing a player does to the game be dismissing
      it. §4's *sticky* skip is a different mechanism and still waits on Phase 5
      arriving twice. **The world does not tick during it** — `tower::drift`
      rolls once per tick, so the same seed would otherwise build a different
      world depending on how long boot ran
      **See it:** ✅ `cargo run -p orbs`. Or as text:
      `ORBS_DUMP=1 ORBS_BOOT=post cargo run -p orbs`, and `dark`/`frame` for the
      rest. `ORBS_BOOT=0` skips it — the only skip there is.
      **The prompt is gone from these screens**, and the `Prompt` stage with it:
      an input line typed itself before the frame drew, on a screen where nothing
      can be typed because every keyed system is gated on `booted`. The first
      affordance the game showed was one that did not work. Removing the drawing
      alone would have left 1.2 s of black indistinguishable from a slow launch,
      so the stage went too — boot is ~13 s now, not 14
- [x] **The tube strike — built, then cut** — a flash and a sweeping band first,
      then the band alone was dropped for reading as a fault, then the flash too.
      It never earned its place: the game is a wizard finding a computer inside a
      scrying orb, and an orb does not power on like a monitor. The screen opens
      black. **Nothing in the game flashes now**, which retires the only
      photosensitivity exposure Phase 0.5 created — DESIGN.md §19 keeps the
      arithmetic, because the *reasoning* was wrong twice and that is worth not
      repeating
      **See it:** ✅ launch the game — it opens black and the prompt types itself

**Phase 0.5 is closed.** Its one open item — sticky skip, persisted CRT-off,
reduce-motion — was never Phase 0.5 work: it needs somewhere to persist a
setting, and nothing in the workspace serialises anything. It has **moved to
Phase 5** beside §15's settings screen, which is where the thing it depends on
is built. A deferred item parked in a finished phase is a phase that never
finishes.

---

## Phase 1 — Core loop

**Exit:** a player automates a duty and feels clever; non-terminal testers are in
the loop.

- [x] **World clock** per DESIGN.md §5.0 — **already correct; the grace was
      removed instead.** The wall-clock mapping and the catch-up clamp shipped in
      Phase 0 and needed nothing. What §5.0 additionally called for was a ~60 s
      inactivity grace; it was built, examined and struck, and §5.0 struck with
      it — DESIGN.md §19.

      The short version: §5.0 already answers *"pausing to think is never
      punished"* in its next sentence — *"drift and decay rates are slow **per
      tick**; the clock itself is not"* — so the grace was a second solution to a
      solved problem. It cost three things: it undercut duration-as-scarcity (a
      clock that pauses while you deliberate makes a brew cost "twenty seconds of
      not thinking"), it created the exploit its own pending-prompt clause
      existed to guard, and it made "is the tower running right now?" a question
      a player could ask.

      **The clock runs whenever the window is open. One rule, no exceptions.**
      **See it:** ✅ launch, touch nothing, and watch `tick` in the border keep
      climbing; `meditate 30` still lands duration actions on the right ticks
- [x] **Brewing, gamified** — §10 gives the domain a minigame form
      (*"sequence/recipe puzzle with timing"*) and Phase 0 built a command
      instead: `decoct clarity` holds the slot for twenty ticks and finishes.
      This is the item that makes manual play *interesting* rather than a chore
      — which matters most in a siege, where §5.1's incident-response loop pulls
      the player back to it under pressure — DESIGN.md §10, §19.

      **Finish the domain before deepening it.** `siphon` is a dark verb, there
      are no potions, no reagent consumption and no vessel mechanic;
      `retort`/`crucible` are nameable nouns nothing reads, and a completed
      `decoct` produces a node called `residue-N`. §11.5's Resources table
      already specifies the missing half. A recipe puzzle cannot be designed over
      a domain with no output to vary.

      **The design is settled — DESIGN.md §10.1**, with §19's entry recording
      what two independent reviews changed. Five instruments
      (`mortar_and_pestle`, `balneum_mariae`, `flask_and_rod`, `alembic`, and the
      `athanor` as shared heat), four of which take Focus. Three new verbs and
      one retirement take §6.1's vocabulary to 18: `move`, `wield`, `stop` in;
      **`decoct` out.**

      **There is no command that brews a potion.** A verb claiming to do all four
      stages at once would teach the player something false, and the only single
      line that makes a potion is a spell they wrote — §8's whole argument.
      `decoct`/`brew`/`make`/`mix`/`distil` stay claimed and resolve to
      `grimoire <recipe>`, so *"make a potion of clarity"* answers with the
      recipe and becomes the tutorial entry point. This also removed a built-in
      that had been sitting in §8's headline script since draft 1.

      Build in this order — each step is independently visible:

  1. ✅ **Content format** (rule 6) — `serde`+`toml`, `orbs_sim::Prose` keyed by
        situation with `{field}` interpolation, compiled in via `include_str!`
        so headless tests need no filesystem. The watcher lives in the *frontend*
        (`orbs/src/sim/content.rs`) because rule 8 forbids async in the sim and
        rule 3 makes a frontend a caller; reloads land in `FixedUpdate`, on a
        tick boundary. Prose hot-reload is **replay-safe** — no line reaches a
        decision. *Recipes will not be, and need content versioned into
        `Sim::submissions`; that arrives with step 2.*
        Tests pin the authored lines to CP437, the 80×22 width, and lower case —
        the first draft shipped an em-dash that drew as `?`, the same defect
        CLAUDE.md already records finding in DESIGN.md's boot text.
        **See it:** ✅ `ORBS_CONTENT=crates/orbs-sim/content cargo run -p orbs`,
        edit `prose.toml`, and the next tick speaks the new line — confirmed in
        the running game, not only in a dump
  2. ✅ **Instruments as places** — the laboratory holds `mortar_and_pestle`,
        `balneum_mariae`, `flask_and_rod`, `alembic`, `athanor` and
        `dispensary`, each a place, so `survey alembic` inspects one from across
        the room while §19's *"you can only name what is where you are"* still
        governs its contents. New `NounKind::Reagent` — one kind for
        ingredients, part-made materials, byproducts and fuel, because §10.1's
        *every byproduct has at least one use* means a kind that sorted waste
        from ingredient would encode a judgement the recipes are meant to keep
        changing. `crucible` removed; `retort` stayed a vessel, so
        `NounKind::Vessel` never empties and the fixtures move onto it.
        **`purge` is now two tiers:** `Protected` (root, live domains) refuses
        outright; any other **place is emptied, never destroyed**. Instruments
        are safe by *being places* rather than by being protected — otherwise
        `purge alembic` deleted the alembic and left a laboratory that could not
        distil, from a verb §7 calls everyday maintenance.
        **See it:** ✅ `ORBS_DUMP="attend laboratory; survey; survey dispensary;
        purge dispensary; purge laboratory"` — six places listed, three reagents
        in the dispensary, the dispensary scoured but intact, the laboratory
        refused. Looking at it also caught the protected refusal reading *"the
        laboratory is the tower itself"*, wording written when only the root
        could reach it.
        *Moved to step 3, where they gain a consumer:* **material states and
        potions.** Both need recipes, and a state nothing reads is a field
        waiting to drift out of step with the transformations meant to define
        it. The `survey`-reports-contents idea is **dropped** — `survey <place>`
        already inspects at a distance, which is what the loop actually needs
  3. ✅ **The vocabulary and the tools that answer it — one slice.** `move`,
        `wield`, `stop`; retire `decoct`; and the tools **run, lock, finish and
        release**. Merged deliberately: retiring `decoct` removes the only live
        brewing verb, so shipping it without working tools would leave the
        laboratory less playable than it is today — a trough with no gate that
        passes. `wield` and the thing it does arrive together.
        Parser work: the source-scoped first slot and its deferred fill, `from`
        added to `FILLER`, the `tra` prefix pin, plain synonyms, `Verb::ALL`
        16→18. Retiring `decoct` repoints five claimed words at `grimoire`,
        registers recipes as `Topic` nouns beside their `Essence`, and touches
        ~22 files including a doctest and the `ORBS_DUMP` examples in CLAUDE.md
        and SETUP.md. `DECOCT_TICKS` goes with it; `work::begin` keeps `research`
        as a caller. One slot each from the global pool; `meditate` **stalls**
        rather than auto-advancing; a finished-but-uncollected tool **releases
        its slot**, or a capacity-1 player who walks away is soft-locked
        **See it:** `move sage from dispensary to mortar_and_pestle`, then
        `wield mortar_and_pestle`; try to move into it and be refused;
        `meditate 600` → finished, not advanced. `make a potion of clarity`
        answers with the recipe
  4. ✅ **The athanor burns, and gates the two heated instruments** — its own
        module (`tower/heat.rs`) and its own content file (`content/fuel.toml`),
        because it is the one instrument that transforms nothing. `Burning` is an
        **interval**, so fuel is a pure function of the tick and survives
        `meditate`; `Banked` is what `stop athanor` preserves. Deliberately not a
        `Working`, so nothing counting the production pool can see it — that is
        what keeps "four instruments, four Focus slots" exact.
        Heat is checked when a run **starts** and the run then completes: pausing
        would be the countdown §19 refused, spoiling would cost progress against
        §11.5's *"never ruinous, only slower"*. Ash lands once, at burn-out, for
        the same determinism reason — per-tick spawning would issue ids into
        `Children` at a rate depending on how the ticks were consumed.
        **The finding:** the two heated stages are not adjacent, so the play is
        light → digest → **damp** → combine → relight → distil, and one charcoal
        covers a whole brew. Found by the fire dying in the end-to-end test;
        recorded in §10.1
        **See it:** ✅ `ORBS_DUMP="attend laboratory; move charcoal to athanor;
        wield athanor; meditate 20; stop athanor; wield athanor; meditate 60;
        survey athanor"` — lights, damps and banks, relights, gutters, and leaves
        ash. And `wield balneum_mariae` cold answers *"wants heat, and the
        athanor is cold"*. **Text, not a bar** — the painter is step 6
  5. ✅ **`siphon`, `stop`'s refund, and the triage slot** — `siphon` is live and
        takes a **place**, not a vessel: §10.1 puts the product in the instrument
        that made it. A `Product` marker separates what you meant to make from
        what you did not, because telling them apart by *name* would mean the
        laboratory deciding which reagents are waste — and §10.1 refuses that,
        since every byproduct is some other recipe's input.
        **Clearing is Triage work now**, not instant: `Triaging` is a separate
        component from `Working` on purpose, so `in_flight()` and `CAPACITY`
        cannot see it. Sharing one type would mean every counter had to remember
        to filter, and the first that forgot would refuse a purge during a brew —
        the exact inversion of §9. `PURGE_TICKS` sits under §11.5's 10–30 s band
        deliberately: at the band's own numbers, clearing four instruments is two
        minutes of a loop whose whole point is not feeling like a chore. A
        placeholder the balance CLI sweeps.
        `stop`'s refund needed no work — the inputs never left the instrument
        **See it:** ✅ `ORBS_DUMP="attend laboratory; grind sage; meditate 10;
        empty mortar_and_pestle; move husks to mortar_and_pestle; purge
        mortar_and_pestle; meditate 5; survey mortar_and_pestle"` — the ground
        sage and its husks come out onto the shelf, and putting the husks back to
        be scoured takes ticks rather than a keystroke.
        **Rewritten when `siphon` was retired** (§19): it read *"onto the
        laboratory floor, the husks stay behind"*, which is two things that
        stopped being true — `empty` turns everything out, and the floor and the
        shelf are one place now
  6. ✅ **The instrument panel** — `Sim::instruments()` reports every fixture
        where the player is standing, so the panel is a property of *where you
        are* rather than something a frontend decides to show. An accessor, not
        records: rule 4 puts command *output* in records, and the panel is the
        world's current state redrawn every frame — a record per instrument per
        tick would bury the scrollback in its own furniture.
        §14 needed a new `Painter::meter`: five bars drawn with `progress` push
        five utterances **a frame**, against *"progress announcements: completion
        only"*. `meter` draws silently and the panel owes **one** summary line
        naming only what is doing something.
        **It earned its rows twice over.** Banked fuel and a fouled instrument
        were each reported as bugs because the only way to see them was to touch
        them, and each was answered with a sentence. A sentence tells you once,
        when you ask; a bar tells you continuously.
        **See it:** ✅ at the 80×22 floor, `attend laboratory` puts six rows above
        the transcript — one filling, one draining, four at rest — and the
        linearised stream is a single `Progress` line. Added to
        `examples/screens.rs`, where `parity` immediately caught the rows being
        drawn with `span` (which speaks): a Wide strip too short for the athanor
        **said one thing less than Deep**, which §9 forbids outright
        *Deferred, and now smaller than it looked:* a **separate** laboratory
        pane above `DEEP_FOCUS_FLOOR`, and telemetry's place in it. The panel
        works at every size inside the session pane, so this is a layout
        refinement rather than the feature
  7. ✅ **Byproducts with uses, and `grimoire` alive** — the variance is a
        **second route to the same draught**: route A spends fresh sage, route B
        spends the husks a grind left behind. Free if you have been grinding,
        unavailable if you have not, and slower — waste costs time, stock costs
        stock. It also puts `weak-tincture` in two places at once: sent to the
        alembic it is haste, sent to the flask it is half a clarity.
        **`grimoire` was dark, which made the retirement of `decoct` a dead end.**
        `make a potion of clarity` resolved to `grimoire clarity` and got a bare
        acknowledgement — §15 weighs the dead-end rate above the raw resolution
        rate, and I had introduced one while calling it the tutorial entry point.
        It now walks the whole route tree backwards from the goal, tolerating the
        cycles in the content (`dregs + sediment → rock-salt → …`) because those
        cycles are what make waste re-enter the pipeline.
        A recipe name answers with routes; any other subject answers with prose,
        and manual topics are **derived from the `grimoire_` keys** so authoring
        an entry makes it nameable without touching Rust
        **See it:** ✅ `ORBS_DUMP="grimoire brewing; attend laboratory; make a
        potion of clarity"` — the manual answers, then the full tree with **two
        routes to `clarified-draught`** listed before a single instrument is
        committed
  8. ✅ **The athanor's bar burns** — the one instrument that literally is a
        fire is drawn as one: flame glyphs in the filled portion, a plume that
        rises through the empty one, on **per-theme colour ramps**. Its meter
        reports fuel *remaining* where the others report ticks elapsed, so the
        flame shrinks and the smoke grows with nothing arranging it.
        This needed a colour family §4's closed palette does not have, so `Style`
        gains a fourth channel, `Depiction` — **the one thing in that module that
        means nothing**, which is exactly what makes it admissible under §14 and
        is asserted by a test that a burning meter's linear stream is identical
        to a plain one's. `Style::depicted()` refuses to paint over an accent.
        The fire is **hottest at its base** and mellows into its tip: the bottom
        half is solid `█` carrying all its motion in hue, `▓` starts about
        halfway up and frays toward the tip, and **sparks** (`∙ ° ·`) come off
        the top and rise through the plume, cooling as they go.
        **The fire meter has no background track.** `░` was the empty fill and
        is now the last of a puff of smoke pittering out, so the plume is sparse
        and thins to nothing as it rises rather than sitting on a solid wall.
        That fell out as a side effect: the boundary had been *relaxed* to let
        `▓` reach the flame tip — `█`-or-`▓` against `░`, a 3:1 coverage step
        where the plain meter draws 4:1 — and emptying the track made it `█` or
        `▓` against nothing, stronger than it began. The forbidden join
        throughout is `▓` against `▒`, one dither step, which the CRT's bloom
        erases. The plain `meter` keeps its `░`: the other four instruments are
        gauges, not fires.
        **`FLIP_HZ = 6` is a recorded exemption from the 3–30 Hz band**, on the
        grounds that the band is about flashes covering a quarter of the visual
        field and this is a bar two cells wide — conditional on three mitigations
        §19 names, any of which being removed brings the rate back down. Motion
        rides `CrtSettings::on`, so the F3 that kills the tube for motion
        sickness kills the shimmer too; the persistent per-effect toggle stays
        with Phase 5's settings item above, which now has a real dependency
        rather than a nominal one.
        **A hearth has three states a gauge does not**, all §19: a **cold** one
        smokes at the bottom and draws no track — it reports no meter at all, so
        it is the one bar in the game with no quantity behind it, and without
        this "out" looked like a row the panel forgot; a **guttering** one keeps
        one faint `▓` ember when its fuel divides to zero cells, because "still
        lit" against "cold" is exactly what `kindle` turns on; and lighting one
        **flares** — the flame grows up out of the base to the full height of its
        fuel over **one world tick**, burning at the top of the ramp where it has
        just caught, with a shower of sparks. Fuel above the front is drawn as
        *nothing*: an earlier pass drew it as dark flame, which is what "present
        but not alight" literally is, and it filled the whole bar with dithered
        orange the instant `kindle` landed. The bar redraws at frame rate and the
        world advances at 1 Hz, so a tick is the longest an animation can run and
        still finish before the fuel it burns ticks down. This is the one place
        the fire shows less than its value, for one tick, on the same trade
        `Reveal` and pane transitions already make — §14 is unharmed because the
        linear stream says *burning* from the first frame.
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
  9. ✅ **The mortar breaks things down** — the second instrument to get a
        picture, and the one that settled the shape for the remaining three. A
        mortar does not fill a container, it **reduces**: the bar is one solid
        block seen edge-on, it gives way at its underside, the pieces snow down
        through a working gap, and they collect as a coarse bed at the bottom.
        **The four shades are four states of one substance** — `█` whole, `▒░` in
        pieces and in the air, `▓` broken and settled — which is what makes it
        legible with no legend.
        The block's underside sits a *fixed* gap above the bed, so it is **eaten
        rather than pushed**: the bar stays full of material and what changes is
        how much of it is broken. So a **loaded** tool is a solid bar of `█` and
        a **finished** one is all `▓` — both states the sim reports `meter: None`
        for, both previously drawing nothing at all, and `ready` holds until the
        tool is emptied.
        **There is no tool in the picture, and that was the third attempt.** `╥`
        looks most like a pestle but is meaningless on the horizontal layout §10.1
        reaches whenever the pane is taller than it is wide; `■` survives the
        rotation and got its own cool ramp so it read as stone — and was still
        wrong, because a mark from outside the fill vocabulary reads as an object
        *visiting* the bar rather than as the material changing state. With two
        cells to say something in, spending one on a tool costs the thing the bar
        is about. `Depiction::Tool` and its four solved ramps went with it: an API
        with no callers is unshaped (§15) — DESIGN.md §19.
        **The fall is slower than the clock**, one cell every two ticks, which
        puts every cell in the gap at 1.5 flashes a second — under the
        photosensitive floor, so unlike the fire this needs no exemption. At the
        full rate the debris streaks rather than falls, so the safe choice is
        also the better-looking one. Tempo is meant to be the signature: the fire
        shimmers fast, the mortar drifts, and the remaining three should each get
        their own so what is running is legible without reading a word.
        Two things moved to keep this honest: `Craft` into the sim, so a frontend
        never matches on `"mortar_and_pestle"`; and the animation clock from
        `shell::fire` to `shell::bench`, because five instruments must not have
        five ideas of how fast a cell may change.
        **The bed creeps between the world's ticks.** The sim turns at 1 Hz and
        reports whole ticks, so a meter read straight off it jumps once a second.
        The fix is not a faster tick — §5.0's rate is what makes duration scarce
        — but the recognition that a tick count is a **sample** of a continuous
        thing: an eight-tick grind is eight seconds of work, and at three and a
        half seconds it really is seven-sixteenths done. Drawing between samples
        is *closer* to the truth than the sample is. `Painter::creeping`
        interpolates and the frontend supplies the fraction, read from the same
        `Time<Fixed>` the sim steps on so the two cannot disagree about when a
        tick lands — DESIGN.md §19.
        **See it:** ✅ `ORBS_DUMP="attend laboratory; grind sage; meditate 3"
        ORBS_FIRE_PHASE=0.33 cargo run -p orbs` — step the phase by two ticks
        (0.00 → 0.33 → 0.67) and the debris falls a cell each time. `meditate 12`
        for the finished bar, which holds full until `empty`.
        `ORBS_TICK=0.0/0.5/1.0` walks the creep **inside one tick**, which a dump
        otherwise cannot show at all: there is no `Time<Fixed>` in it, so every
        bar would sit exactly on a tick boundary — the one jump this removes.
        **The loaded bowl needs the long form**, `move sage to
        mortar_and_pestle`: §19's per-instrument verbs are *move plus wield*, so
        `grind` never leaves an instrument sitting `charged` — see the note under
        that item.
        `cargo run -p orbs-render --example screens` prints the whole stroke and
        the lifecycle side by side, and both orientations
  - **Decisions, not execution.** The outcome may depend on *what the player
        chooses given the tower's state*; it must never depend on how fast or
        precisely they act. §5.1 mechanises "triage bandwidth, not typing speed",
        §14 forbids any mechanic requiring fast typing, and §8 gives *scripts*
        the timing precision a human cannot hit. A choice is also something a
        script can encode and `orbs-balance` can sweep — an execution-quality
        mechanic is not, and §19's Phase 0.5 entry already refused that shape
        once for the same reason
  - **Timing means windows at 1 Hz** — *when* to advance a stage against
        everything else wanting the slot — never a reflex
  - **`meditate` idempotence must survive stages.** §19 chose an interval over
        a countdown so hundreds of ticks inside one `step()` behave identically
        to being watched. A staged action must also resolve with **no player
        present** — that is a `meditate`-and-stage-boundary requirement, *not*
        an offline one: §5 opens *"initially there is no offline progression"*
        and puts it in Phase 3a, covering bound scripts rather than a lit athanor
  - Durations stay placeholders. §19 records `DECOCT_TICKS = 20` as one, and
        the balance CLI later in this phase is what sweeps it — do not hand-tune
  - Out of scope, stated: the archive's minigame (deferred, below), adversarial
        aberrations (siege-only, §5.1), nuisance aberrations (unscheduled in
        every phase — a separate finding, not this item's job)

      **Exit criterion:** ✅ the same recipe, in two different tower states, has
      two different right answers — and the difference is **readable from the
      records before you act**. Not "the second playthrough differs", which a
      coin flip satisfies; the point is that it is still interesting on the
      twentieth brew
      **See it:** ✅ `grimoire clarity` lists both routes to the draught before an
      instrument is committed to either, and
      `the_same_goal_has_two_right_answers_depending_on_what_the_laboratory_holds`
      runs both to the same end — one on fresh sage, one on the husks a grind
      left behind

      **What is *not* done, stated plainly.** Durations are placeholders and the
      balance CLI has not swept one of them; `PURGE_TICKS` and the athanor's
      `ticks` are the two most likely to move. Capacity is still hard-wired to 1,
      so the four-instrument pipeline this was designed around cannot be run
      concurrently until Phase 2 reserves slots and Phase 3a researches them —
      everything shipped here is the capacity-1 game. A separate laboratory
      *pane* above `DEEP_FOCUS_FLOOR` is deferred: the panel works at every size
      inside the session pane, which makes that a layout refinement rather than
      the feature
- [x] ✅ **The grimoire, the spell editor, and the script engine** — three items
      that turned out to be one, built together. DESIGN.md §19 records five
      decisions and six things the runner had to learn.
      - **`grimoire` is a place now, and the manual is `recall`.** The word named
        both the reference you read and the book you write in. Releasing it
        deleted the vocabulary's only shared three-character prefix
        (`gri`: `grimoire`/`grind`) along with the paragraph arguing that clash
        was survivable. `rec` is pinned by a test before anything else wants it
      - **A nameless filesystem root**, holding `/tower` and `/grimoire` as
        siblings — a spell is a book you carry, not a shelf you walk to. Built as
        a throwaway experiment *first*, which caught three tests that would have
        silently stopped covering half the tree
      - **`Held(Vec<String>)`** — the first file in the game with stored text.
        Logs stay a *view* over the record stream (§3) and the two paths do not
        merge
      - **The editor**: always-insert with a `:` line taking `:w`/`:q`/`:wq`/`:q!`.
        Buffer and painter both frontend-side, beside `Line` and `panel.rs` — a
        keystroke reaches no decision, and `boundaries.rs` forbids the sim from
        naming a layout type at all
      - **`:w` canonicalises**, walking a simulated position through the spell so
        location-scoped verbs resolve. Places anchored by ID, stock never —
        *both since superseded: the file is never rewritten and the reading
        happens at cast (§19), and the ID anchor was withdrawn*
      - **`Submissions` is an enum** — one `Wrote` entry per save, carrying the
        buffer as *typed*, with a replay test
      - **The runner**: own cwd, two-tier blocking, budget, verb allow-list,
        `PATIENCE` before a wait becomes a failure

      **See it:** ✅ `ORBS_DUMP="attend laboratory; scribe brewing"`
      `ORBS_EDIT="edit\nkindle charcoal\ngrind the sage\nempty mortar_and_pestle\n<esc>\nquit"`
      `ORBS_THEN="invoke brewing; meditate 40"` — loose phrasing goes into the
      editor and the spell brews ground-sage while the player stands still.
      `peruse` reads it back **as typed**; the orb's reading of it is `interpret`
      in the editor (§19 — the file stopped being rewritten)

      **What is *not* done, stated plainly.** `bind` is still dark — standing
      automation is the Concentration item below. Conditionals, loops and
      triggers are unbuilt (§8 gates them as capability unlocks). §8's
      **verbosity levels** do not exist, so *Budget starved* cannot be reported
      the way §8 describes and the budget is silent when it bites. **A
      half-executed spell is not serialisable yet** — §8:817-826 requires it and
      `Running` is not in any save format, because there is no save format.
      `purge` on a running spell is untested. `invoke` from inside a spell is
      **forbidden rather than depth-limited**: `MAX_DEPTH` exists and nothing
      reads it
- [x] ✅ **Domain-scoped spells, watchable events, and the first control
      structures.** DESIGN.md §19 records nine decisions.
      - A spell carries its **domain as data**, not as a directory —
        `/grimoire/laboratory/` collides on the leaf with `/tower/laboratory`.
        `attend` in a spell is flagged at authoring and refused at cast
      - **`FieldName::At`** — a completion says where it happened, always.
        Eleven sites disagreed; `say` now requires it and a test drives a brew
        and fails on any event without one
      - **`wait for <thing>`**, `repeat [n]`, `end`. Control words are a
        spell-only, exactly-matched table, and `Resolution::InSpell` makes the
        prompt answer for them — before it did, `wait for the mortar` typed at
        the prompt **opened the editor on a new empty `mortar.spell`**
      - `wait` was released from `meditate`'s synonyms; the tolerated-collision
        set is one shorter than it was
      - A malformed spell **runs**: unmatched blocks close at end of file, said
        once. §8 forbids both refusing at save and halting at cast
      **See it:** ✅ write `repeat 2 / kindle charcoal / grind the sage / wait for
      the mortar / empty mortar_and_pestle / end` in the editor, `peruse` it back
      exactly as typed (the buffer's own indentation and all), `interpret` it to
      see the commands read, then `invoke` it and watch the loop run twice
      - **`stop <spell>`** — a spell could not be called off at all: `stop` took
        a `Place`, and nothing but running out of program removed `Running`,
        which an unbounded `repeat` never does. `stop` now takes a place **or** a
        script. Stopping the spell does not stop the brew it started
      - **`if` / `else`**, with two shapes and no operators: `if the dispensary
        has sage`, `if the mortar is idle`. An unreadable question answers **no**
        and says so — guessing would let a condition the player did not write
        decide what their laboratory does while they are elsewhere
      - The **stop → edit → restart** loop holds: the program is derived at cast,
        so saving over a running spell is safe and takes hold next time, which
        `scribe` now says rather than leaving the edit looking ignored

      **Not done:** `when` — §4.2's question is unsettled, whether in-file `when`
      and §8's `bind --to dusk` are one mechanism or two.
      §14's linear view **cannot express nesting**: `FieldName` has no depth and
      `RecordKind::ScriptLine` is still emitted nowhere, so a screen-reader user
      hears a flat sequence. That needs deciding before more block types land
- [x] ✅ **A spell edited while it runs.** Four changes that are one mechanic —
      the orb reads along with you. DESIGN.md §19 records them.
      - **`SCRIPT_BUDGET` is 1**, down from 4. A spell costs a tick per step, so
        **a shorter spell is a faster spell** — the lever §11.5 wants and four did
        not give, because at four the difference between a tight spell and a
        sloppy one vanished inside one tick. Blocks count as steps, or block-heavy
        spells would be free
      - **The buffer saves itself** 0.6s after the typing stops. `save` and
        `discard` are gone; the vocabulary is `edit` and `quit`. `quit` can no
        longer refuse, because there is no reachable unsaved state to refuse over
        — which deleted `Complaint::Unsaved` and `Outcome::Close` with it
      - **A save over a running spell swaps the program in place**, keeping the
        position. Still §8's tick-boundary rule: a save queues through `Pending`,
        so the swap is between steps and never inside one. Remapping the position
        would be a diff, and a diff that guesses wrong moves a spell to a line the
        player did not point it at
      - **The gutter marks the line the orb is on**, and the title says it too —
        §14 forbids a fact carried only by a glyph in a column. `line_of` returns
        `Option` now; its `0` sentinel had been putting the marker on line zero

      **See it:** ✅ `ORBS_DUMP="attend laboratory; scribe brewing"`
      `ORBS_EDIT="edit\nrepeat\nkindle charcoal\ngrind the sage\nempty mortar_and_pestle\nend\n<esc>\nquit"`
      `ORBS_THEN="invoke brewing; meditate 7; scribe brewing"` — the editor
      reopens on the spell mid-flight with `4»` in the gutter and *(the orb is on
      line 4)* on the title. In the running game: `invoke` it, `scribe` it, type,
      stop typing, and watch the next pass take the new line without a keystroke
      asking it to

      **What is *not* done.** The debounce is a **frontend clock**, so `ORBS_DUMP`
      cannot exercise it — a dump advances no `Time`, and saves with `quit` or `w`
      instead. Two `App`-level tests in `shell::editing` cover the wiring headless.
      The marker's *movement* still needs eyes on a window: a dump is a still
- [x] ✅ **The editor indents blocks as you type.** Four spaces per level on every
      line of every spell was a tax on writing one. `Enter` inside a block opens
      the next line at that block's depth, `end` and `else` step back out as the
      word completes, and `Backspace` in the leading whitespace falls back a whole
      level rather than a character. DESIGN.md §19 records the three judgement
      calls.
      - **`parser::indent_around` is the one rule**, folded by `canonicalise` when
        the orb writes the file and by the editor as you type. Not tidiness: the
        orb re-indents on save, so a buffer that indented differently would make
        every save look like it had moved your work
      - **`Enter` mid-line indents nothing** — auto-indent is for "start the next
        line". Prepending an indent to a split tail stops `Enter` and `Backspace`
        being each other's inverse, which an existing test already pinned and
        which caught this being written the other way round first

      **See it:** ✅ `ORBS_DUMP="attend laboratory; scribe nested"`
      `ORBS_EDIT="edit\nrepeat 2\nkindle charcoal\nif the mortar is idle\ngrind the sage\nelse\nempty mortar_and_pestle\nend\nend\n<esc>\nquit"`
      `ORBS_THEN="peruse nested.spell"` — not one space typed, and the file the
      orb writes back matches the buffer line for line
- [x] ✅ **An `if` names places the way every other line does.** Reported from a
      screenshot as *"the `if` block is working inversely"*, and it was not:
      `if mortar is empty` compared `mortar` against `mortar_and_pestle` and
      answered no for ever. DESIGN.md §19 records both halves.
      - **A control word's tail was never canonicalised**, so `if` was the only
        line where the player's phrasing had to match the tower's internal name
        exactly. `peruse` now reads back `if mortar_and_pestle is empty`
      - **`holds` returns `Option<bool>`.** A place the tower does not have is
        §8's *Referent missing*, not a false condition — conflating them is what
        made this silent. The runner names the place it could not find
      - **`if` had only ever been tested as a parse.** A condition that parses
        and one that finds anything are different claims; the new tests run the
        spell and assert on which branch executed

      **See it:** ✅ `ORBS_DUMP="attend laboratory; scribe probe"`
      `ORBS_EDIT="edit\nrepeat 10\nif mortar is empty\ngrind sage\nelse\nempty mortar_and_pestle\nend\nend\n<esc>\nquit"`
      `ORBS_THEN="peruse probe.spell; invoke probe; meditate 40"` — the loop
      alternates grinding and emptying instead of only emptying. The file reads
      back as typed; `interpret` is where the resolved name is now shown (§19)
- [x] ✅ **The orb no longer writes down a shorter command than it heard.**
      Reported as *"`grind sage` is truncated to `grind` when I close and reopen
      the editor"* — and it was, because `quit` saves and every visit
      re-canonicalised the file against whatever was on the shelf. DESIGN.md §19
      records it, plus the marker restyle and a prose line that drew its own
      placeholder.
      - §10.1's verbs take their reagent **optionally** by design, so with the
        sage spent `grind sage` legitimately resolved to bare `grind`. The
        resolver was right; canonicalisation had no business writing it down
      - **The guard asks about reagents, not words**, because canonicalisation is
        *supposed* to discard — `make a potion of clarity` → `recall clarity`,
        `look around` → `survey`. A word sweep flags both, so it would have
        broken the tutorial to save the editor. A reagent's *name* is fixed; its
        *presence* is not
      - **`spell_gave_up` printed `{source}` literally**, which `prose.toml`
        documents as the deliberate way a typo surfaces. It surfaced.
        `no_line_a_spell_can_say_has_a_hole_in_it` sweeps the whole stream for
        `{` rather than checking a list of keys, which is the thing that rots

      **See it:** ✅ `ORBS_DUMP="attend laboratory; grind sage; meditate 20; scribe keep"`
      `ORBS_EDIT="edit\ngrind sage\n<esc>\nquit"`
      `ORBS_THEN="peruse keep.spell"` — the sage is spent before the spell is
      written, and `grind sage` still reads back whole (flagged, not truncated)
- [x] ✅ **Saving says nothing, and a burning athanor is not idle.** Two reports
      from playing, both about the editor. DESIGN.md §19 records them.
      - **The transcript filled with saves.** The buffer writes itself out after
        every pause in the typing, so `{name}: N lines, written down` — and its
        `1 the orb could not read` companion — arrived every second or two;
        sixteen copies behind the modal for one session. Both keys are gone with
        no replacement: a line the orb could not read is named *individually,
        with its number*, by the runner when the spell is cast. `spell_reloaded`
        survives, once per editing session rather than once per write
      - **`if athanor is idle` was yes while it burned.** The fire is `Burning`
        and deliberately not `Working`, so `busy()` could not see it — `is idle`
        said yes and `is working` said no about the same fire, with `at burning`
        drawn on the panel beside them. Both words now ask the panel's own state,
        which makes the word on screen and the word in a spell one fact

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe brewing"`
      `ORBS_EDIT="edit\nkindle charcoal\ngrind the sage\nempty mortar_and_pestle\n<esc>\nquit"`
      `ORBS_THEN="invoke brewing; meditate 40"` — one line for the `scribe` and
      nothing at all for the save. ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory;
      scribe tending"`
      `ORBS_EDIT="edit\nif the athanor is working\nstop the athanor\nelse\nkindle charcoal\nend\n<esc>\nquit"`
      `ORBS_THEN="kindle charcoal; invoke tending; meditate 4"` — the panel reads
      `athanor banked`, so the spell took the `if` and not the `else`
- [x] ✅ **`debug_spawn`, the alembic's bubbles, and a review of both** — three
      things off the back of the spell-language work below. DESIGN.md §19 records
      each; none is a roadmap box, so none advances the version.
      - **`debug_spawn &lt;reagent&gt; [count]`** puts reagents in the dispensary from
        wherever you are standing, so a state that costs forty ticks of grinding
        is one line. **Not a `Verb`** — matched exactly, before the parser,
        absent from `Verb::ALL` and from §6.1's tutorial, whose headline metric a
        build-only word would corrupt. `cfg(debug_assertions)`, and the one test
        that is `cfg(not(...))` runs under `cargo test --release` to assert the
        door is shut. Known names only, on a tick boundary, and it says what it
        did
      - **The alembic throws bubbles into the air above its face**, which had
        been refused twice — one reason was real (in the *sideways* layout
        "above" is rightward, onto the athanor's own sparks) and is now handled
        by drawing them only where the bar runs upward; the other did not survive
        contact. The distillation is **four times longer** (56 ticks), and the
        bubbles are measured from the **face** rather than from the bar's base,
        so a rising level can never overtake them — which no tick count could
        guarantee, since the bar grows with the window
      - **A `/code-review high` found four live bugs**, all reproduced before
        being fixed: `interpret` re-derived a rule `compile` already owned and
        disagreed with it on both halves (every byproduct name painted red on a
        working line; a reagent standing in for a place reported clean and then
        failing at run time); `invoke not_written_yet` read back as
        `invoke first_light.spell`, which is §19's rewriter bug on the surface
        that replaced the rewriter; and `debug_spawn ash 0` spawned a node
        holding nothing that could never be taken from

      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=80x40 ORBS_FIRE_PHASE=0.4
      ORBS_DUMP="attend laboratory; kindle charcoal; debug_spawn
      clarified-draught; distil clarified-draught; meditate 24"` — `°` above the
      face in the `al` column, and the level far enough down to watch it climb.
      ✅ `cargo run -p orbs-render --example screens` for eight rises side by
      side. ✅ `cargo test --release -p orbs-sim --test debug_spawn`
- [x] **The file is the player's, and `if` learned to say `and`** — reported as
      *"I really don't like that the game edits a file and throws lines away"*,
      and it did: the parser read the first `is` in a question and the rewriter
      wrote what it had understood into the spell, so
      `if a has x or b has x` was saved as `if a has x`. §19 had already recorded
      three bugs of that shape, each closed by another special case in the
      rewriter. This closes the class instead. DESIGN.md §8 is amended and §19
      records it.
      - **Canonicalisation moves from save to cast.** `write` stores the lines as
        typed; `spell::compile` is the only route from text to a runnable
        `Program`, and `Draft` is a separate type so nothing can run an
        unresolved one. Cast is the better moment anyway: the names are fixed in
        the world the spell is about to run in, with the player standing there
      - **`not` > `and` > `or`, with `either … or …` as a bracket made of
        words.** The bracket words take *operands*, not sub-questions — written
        the obvious way the inner disjunction swallows the following `and` and
        `either` stops grouping at all. `both` is accepted and inert. A shared
        subject expands: `has sage and charcoal` is two questions about one shelf
      - **Everything must be read, or nothing is.** A question that does not
        consume every token returns `None`, and the line is kept, marked and
        reported. Without that rule the half-read line just moves from the writer
        into the reader, where it is harder to see
      - **A spell resolves at `SPELL_SIMILARITY` = 850, and a tie refuses.**
        `ground-salt` scored 819 against `ground-sage` — over the prompt's floor
        of 600 — so a question could compile into one about a different reagent.
        850 is where `fuzzy` already separates an abbreviation from a typo
      - **An unanswerable question runs neither half** and names what it could not
        place once per line per casting. It used to take the `else`, for ever
      - **`interpret`**, a third editor word: the buffer as the orb reads it, on
        request. A line it cannot read draws in `Role::Danger` with the count on
        the status row
      - **`SPELL_MARGIN` is 50, where it was 1** — which meant "refuse only on an
        exact tie", a margin in name and a tie in behaviour. Nothing in the
        shipped vocabulary lands between 1 and 54, so the two were
        indistinguishable until one reagent was named close to another. 50 is the
        room `sag` leaves, which must keep reaching `sage` past `sage-husks`
      - **Two entries for one word are one candidate.** §6.1 registers a `Topic`
        beside every reagent, so `ground-sage` is in the scene twice at identical
        scores and the tie rule compared a name against a copy of itself —
        `ground-sag` was refused as ambiguous with the thing it names
      - **47 new tests**, three of which found real bugs on their first run: a
        `has` operand swallowed the following clause, an unknown *thing* answered
        no rather than being called a typo, and the duplicate-name refusal above

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
      **bound spells** and starting at **0**, and the currency that buys it.
      Replaces §11.5's Attention pool, which counted actions and started at 3;
      DESIGN.md §19 records the rename, the superseded numbers, and why the ~8
      ceiling is inherited from the old ~25 rather than invented.
      **Concentration 1 is the game's turn**, not a step on a curve: until it is
      bought the tower is worked entirely by hand, so pillar 3's promise is
      something the player *earns* rather than something they are handed at
      minute zero.
      - **Experience supersedes fragments as the progression currency.** Earned
        by completing runs, weighted by the instrument — mortar 1, balneum 2,
        flask 4, alembic 8, `research` 1 — so each tier of tool is worth every use
        of the one below. It accumulates and is never spent. §11.5's ~300-fragment
        derivation and its unlock cadence are amended with it
      - **16 is arrived at, not picked**: 1+2+1+4+8 is one clarity walked end to
        end, so the player buys the level with exactly the potion the tutorial
        teaches. The 30–45 minute anchor moves to ~2 minutes rather than the
        price being inflated to meet it
      - **An invocation ends when you leave its domain**, said in voice; a bound
        spell does not. Narrowing `invoke`'s *cast* would have bought nothing —
        the domain was already fixed at cast and the player's position was never
        read again, so leaving a running invocation was free
      - **A bound spell stands**, cast again when it runs off the end, or a slot
        at capacity 1 is held by a spell doing nothing. The laps are **silent**;
        the `bind` the player typed says itself once
      - **Invariant 3 is struck** — a script action is not faster than a manual
        one. Speed is a later upgrade; what automation sells is running while you
        are elsewhere. The **fractional slot charge** is deferred to multiplex
        capacity, which is what it exists to price
      - `progression.toml` is validated against `recipes.toml` — the first
        cross-content check in the project, and why it loads after recipes
      - **30 new tests** across `tests/progression.rs` and `tests/binding.rs`,
        every one driven through real runs: there is no public way to hand the sim
        experience, so a test that skipped the work would test a state the game
        cannot reach

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
- [x] **`weave` — the Ley Line, Mastery, and a surface for progression.**
      Concentration 1 arrived on its own and `status` printed two numbers with
      nothing saying what they were for. This is the screen, **read-only**: it
      does **not** close the item above, which needs a node worth choosing.
      - **Two tracks, replacing §11.5's placeholder.** *"Ley-line upgrades and
        grimoire rank"* named two sources of Concentration and had no mechanical
        content anywhere. **The Ley Line** is the straight path — passing a step
        *is* the grant — and **Mastery** branches, a tier opening and giving one
        of its nodes. `grimoire rank` is dropped: `/grimoire` is already the
        player's spellbook
      - **No points; experience is still never spent.** A tier opening costs
        nothing and taking one of its nodes closes it, which is a choice without
        a balance to regret
      - **`ascend` failed the scorer at 667 against `attend`** — computed before
        the name was picked. `weave` is 200 against `wield`, 400 against `write`,
        `wea` free. **No `tree` synonym**: in a game whose premise is a
        filesystem, it means *list this directory*, and no fuzzy test catches a
        collision of meaning. **`may_issue` refuses it**, or `repeat 100 / weave`
        is a soft-lock
      - **A bar, then two tracks running right.** Progression runs rightward and
        the screen says so three times: the bar fills right, the Ley Line runs
        right, Mastery's tiers run right. Siblings stack **downward**, which is
        the other meaning — rightward is progress, downward is a choice. **The
        first version was a vertical list and was replaced for being one**: it
        fit, and it did not say what it was for
      - **The bar is a fixed hundred and both tracks are drawn under it at the
        same width**, so a node's position *is* its cost —
        `─────[•]───────` with the fill either past it or not, and Mastery
        forking off a trunk at 24 with a line from each node to *its own*
        successor at 40. Measured against the *next* threshold instead, the bar
        emptied itself the instant the player earned something, which is when it
        should look most like progress; at a fixed stride, a tier at 24 sat five
        cells from a tier at 40 and said they were adjacent
      - **The session pane is ~48 columns, not 80** — panes tile side by side
        above the 100×28 deep-focus floor, so the 80-column floor is the
        *widest* case. What a node *is* lives in a **details panel** in the
        bottom right rather than beside every node, which is what lets the
        picture fit and the words stay readable
      - **The panel says `unlocked` and `active` separately**, because they are
        different questions: a mastery node can be unlocked and idle (nobody
        chose it) or unlocked and idle for ever (a sibling took the tier's one
        choice). `Locked` draws the same for *not earned yet* and *already
        spent* on purpose, so `unlocked` is a field rather than a reading of the
        glyph — one says *work more*, the other says *you chose otherwise*
      - **`●` is not in CP437**, so "taken" would have drawn as nothing and
        collapsed the one distinction §14 says must not be colour-only. `• ○ · ─`,
        checked against the table. Nodes are drawn silently and `announce` their
        total and state as **words**, so a reader is not handed a bare glyph
      - **The way in is a word.** `ley`/`mastery` go into a track and hand the
        arrows over, as `edit` drops into the editor's buffer; an arrow at the
        command line does nothing. Reported as *"my first key press was being
        ignored"* — which was also a **real bug**: `chord_is_stale` means *accept
        this key*, and this screen read it as *drop it*, swallowing one keystroke
        every time it opened after a pause
      - **Every node is framed `[○]`; the aimed one is `«○»` and Bright** — two
        carriers, because brightness alone failed outright (an aimed `○` is
        already Bright and identical to its sibling) and §14 forbids the
        difference being colour. The aim is an identity, not an index, so a tier
        opening under it unplaces it rather than moving it
      - `progression.toml` gains three rules and `deny_unknown_fields`, which is
        the one that mattered: with both tracks defaulted, the **old**
        `[concentration]` section parsed into a tower with no curve at all

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
- [x] **The prompt becomes a command line** — caret editing (←/→, Home/End,
      `Cmd+←/→` because a Mac has no Home key, Escape to clear, insert and delete
      at the caret), history on ↑/↓ **filtered by what is typed** with the prefix
      anchored for the whole search, `parser::complete` in orbs-sim, Tab, and an
      inline suggestion that prefers history over completion.
      **Not on the roadmap when it was asked for**, and taken against a 28-vs-24
      month gap — it displaces nothing yet, but the script engine and the
      remaining sabotage surfaces are what it competes with. DESIGN.md §19 records
      the eleven decisions, including two live bugs the work uncovered: a `move`
      could raid a **running** instrument and say nothing, and the
      three-argument `move` echo was already clipping its own destination.
      Deferred and named: `Ctrl+R`, `Delete`, `Ctrl+U`, `Ctrl+W`, word motion,
      paste, IME, scrollback wrapping
      **See it:** ✅ `ORBS_LINE="wield mo" ORBS_DUMP="attend laboratory"` shows
      the partial line with `rtar_and_pestle` ghosted after the caret and a
      `Hint` in the linearised stream. In the running game: type, walk into the
      middle of a command and edit it, ↑ through the wields, Tab a name
- [x] **Ten-angle review of the brewing + command-line work, applied** — eleven
      live bugs, four of them reachable in a first session: `purge dispensary`
      made the tower **unwinnable**, a `move` could raid an instrument being
      scoured, `peruse <domain>.log` was permanently empty, and the transcript
      dropped its newest records at the 80×22 floor. Plus a fidelity regression
      that opened the default window two tiers finer than §9's table.
      **Four had green tests over them, three of which asserted the bug** — see
      DESIGN.md §19 for the table and for the two findings that were checked and
      rejected.
      **See it:** ✅ `ORBS_DUMP="attend laboratory; grind sage; meditate 12;
      move ground-sage to balneum_mariae"` — the linear stream now says `laboratory:
      mortar_and_pestle fouled, balneum_mariae charged` where it said nothing at
      all, and
      `cargo run -p orbs-sim --example session` runs the whole §10.1 pipeline and
      reads nine lines back out of `laboratory.log`
- [x] **Readable recipes, Tab cycling, and a scrollable transcript** — three
      things a player asked for after using the laboratory.
      `grimoire` now reads as **instructions in doing order** rather than as the
      raw breadth-first walk printed backwards as five unlabelled columns; it
      stops at what the dispensary stocks instead of expanding three ways to make
      the rock-salt you already have. Repeated **Tab cycles** the candidates
      (readline's `menu-complete`), the first press still listing without touching
      the line. **PageUp/PageDown** scroll the transcript, with the border saying
      so — Up/Down stay history.
      The recipe work found a fourth thing: a line view **clipped** anything past
      the pane width instead of wrapping, which is silent loss on §14's primary
      surface. Wrapping went into `RecordView`, so every long refusal benefits.
      **See it:** ✅ `ORBS_DUMP="attend laboratory; grimoire clarity"` — five
      numbered steps, byproducts, heat markers, alternatives, nothing clipped at
      the 80×22 floor. ✅ `ORBS_SCROLL=14 ORBS_DUMP="…"` shows the transcript held
      back with `PgDn newest` in the border. In the running game: type `wield `
      and press Tab three times
- [x] **Per-instrument verbs, scoped to their domain** — `grind sage` is the
      `move` and the `wield` in one, and the instrument is named by the verb
      rather than typed. `mix a and b` charges two, with `and` dropped as filler
      so the slots fill positionally — the mechanism `move x to y` already used.
      Five verbs (`grind`, `digest`, `mix`, `distil`, `kindle`), each declared by
      its own instrument and **only a word where that instrument stands** (§7).
      `kindle` is the odd one — lighting is not a run — but it charges and starts
      like the rest, and bare `kindle` relights what was banked. That is
      what keeps §10's five further domains from widening each other's collision
      surface. Out of its domain a verb still answers — `there is nothing here to
      mix with` — and an exactly-typed one outranks a fuzzy rival, without which
      scoping would have *created* the misreading it prevents.
      DESIGN.md §19 records the three conditions and why `grind` was kept over a
      collision-free alternative.
      **`empty <tool>`** landed with them: the counterpart of `purge`, turning an
      instrument out into the store instead of destroying what it holds. §10.1
      says every byproduct has a use, so a loop that can only clear by destroying
      never finds route B. Instant, unlike `purge` — the four ticks are the price
      of destroying, not of tidying.
      **See it:** ✅ `ORBS_DUMP="attend laboratory; grind sage; meditate 12"` —
      three commands where four were needed, same result. ✅ the same verbs in
      `attend archive` say where they are not
- [x] ✅ **The balneum mariae draws, and the fire stops changing colour** — the
      third of §10.1's five instrument pictures, plus a correction to the first
      two — DESIGN.md §19.

      **The bath is a vessel of liquid, and the level is the reading.** Not *how
      far along*: the sim reports no meter for a charged bath, so a progress bar
      would draw it blank, which is exactly what an empty instrument looks like —
      the invisible-state defect this panel exists to remove, reached from a new
      direction. A charged vessel is a shallow layer, a finished one is full, and
      sediment is a still band lying too low to be either.

      **All of its motion is in the colour.** The glyph is `█` at every fill and
      every phase, so the level survives greyscale on solid-against-blank — the
      strongest join the alphabet has — and nothing in the picture can be
      mistaken for a bubble. Its tempo is four shared-clock ticks per beat, the
      slowest on the panel, which is both the signature a gentle heat should have
      and a photosensitivity argument it does not have to make: 0.75 flashes a
      second against a floor of 3.

      **The fire is one orange ramp on every tube**, where it had been four. A
      green fire does not read as a fire; it reads as the meter having changed
      colour. That put hue on the monochrome theme, and §4's accessibility
      promise moved to Phase 5's colour-vision filters — recorded there and in
      §19, because it is a promise *deferred*, not dropped.

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal;
      grind sage; meditate 9; empty mortar_and_pestle; digest ground-sage;
      meditate 6"` — the vessel filling; step the last `meditate`. ✅ the same
      with `move ground-sage to balneum_mariae` for a charged one, which is a
      shallow layer rather than nothing. ✅ `cargo run -p orbs-render --example
      screens` for the roil as `a`/`b`/`c` — a dump cannot show it, because the
      glyph never changes. ✅ `cargo run -p orbs` and F7 through the themes: the
      hearth burns the same orange on all four
- [x] ✅ **Materials carry a colour, and an instrument's bar draws in it** — the
      panel stops saying *something is in here* and starts saying *what* —
      DESIGN.md §19.

      Sage grinds pale green; leave the husks behind and the same bar turns
      brown; the bath's liquid takes the colour of whatever is dissolved in it.
      Authored in `content/materials.toml` against the eight families fixed in
      `orbs_render::Tint`, and **an unknown name fails the load** — an untinted
      material draws in the base hue too, so a silent fallback would make a typo
      indistinguishable from an omission.

      **It lives on the `Frame`, not on the `Cell`.** A tint is a property of
      what is in an instrument, so every cell of one bar shares it — a per-cell
      byte would spend 7,040 cells of a 160×44 grid to express a value that
      varies across five of them, and take `Cell` from 8 bytes to 12. It is a
      `(Rect, Tint)` side-table beside `Frame::magnified`, which is the same
      shape for the same reason: informational, so both frontends get it.

      **Two things it may never paint over.** An accent — a fouled instrument's
      label stays red however brown its husks are — and the fire, which burns one
      orange ramp on every tube (§19). Fuel is a tinted material like any other;
      the flame simply does not consult it.

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; move sage to
      mortar_and_pestle"` prints `green 2×17` under the linear stream; the same
      run ending `move ground-sage to dispensary` prints `brown`. A dump can
      show the region but not the colour — ✅ `cargo run -p orbs` for that
- [x] ✅ **The flask_and_rod mixes, and a finished bath keeps turning over** —
      the fourth of §10.1's five instrument pictures — DESIGN.md §19.

      **The flask is the only instrument that takes two inputs, and that is its
      whole picture.** The vessel starts as two bands of ingredient and the
      mixture grows from the floor as both are used up together — neither
      consumed before the other is touched, because that would read as one
      reagent then the next rather than as combining. The mixture's colour is
      the **average** of its two ingredients rather than a third authored
      colour: a colour between its neighbours reads as a mixture of them, and a
      new one reads as a substitution.

      That needed the tint channel to carry a *pair*, which is affordable
      because it lives on the `Frame` per region rather than on the `Cell`
      (`Wash`). A payload would have been unthinkable a byte per grid position
      ago.

      **A finished bath settles rather than freezing.** It has just spent its
      run over a lit athanor and is still hot, so it goes on turning over at a
      third the rate with a third the bubbles. `Charged` stays dead flat — the
      at-rest rule was written about instruments that have *not started*, and
      that is still absolute. The difference between the two is what the meter
      cannot express, since the sim reports no quantity for either.

      Also: rock salt is a cool off-white rather than a cream, sage is muted
      (it is on screen more than the other seven tints together), and the ramp
      narrowed from 1.92× to 1.39× per step so a material catching the light
      stops reading as two materials.

      **See it:** ✅ the eleven-command line in CLAUDE.md prints `green`, `bone`
      and `green+bone` as three regions; step its last `meditate` and the
      mixture grows as both bands shrink. ✅ `ORBS_DUMP="...; digest
      ground-sage; meditate 20"` for a bath that has finished and is still
      moving — a dump cannot show that, so ✅ `cargo run -p orbs` for the rate
- [ ] Remaining sabotage surfaces (world, script text, trigger clocks)
      **See it:** `verify` each of the four surfaces and have it name the tampering
- [x] **The archive is a maze, and the world holds the search** — §10 calls the
      domain *bespoke* and *"stales fastest"*, and it was five entities with a
      verb that consumed nothing and produced nothing. `research` now resolves a
      labyrinth out of the lectern, `follow` threads it, the way out gives up a
      fragment, and four fragments make a scroll.
      - **A search, automatable, with no grammar change.** Trémaux needs no
        memory beyond marks in the passages, so the cells mark themselves and the
        four ways publish what is adjacent. That is DFS performed physically —
        the marks are the visited set, the head is the stack pointer
      - **One thing had to give and it was the vocabulary.** `compile` resolves a
        condition's names at *cast*, which is when no cell is `walked` — so every
        `if` compiled to a dead branch. `NounKind::Sense` plus five words the
        scene always offers is the whole fix
      - **`step` scores 750 against `stop`, `tread` 800 against `read`.** `follow`
        is 429. It is the 21st tower-wide verb and is recorded as a **debt**: it
        belongs to the archive, but a fixture carries one `Operation` and the
        lectern spends it on `research`
      - The instrument retired three defects at once — no completion sentence, an
        unstoppable run, no `recall` — and with them `DIVINE_TICKS`,
        `pipeline::work` and `progression::DIVINE`

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; research; survey north; follow east"`
      — a labyrinth resolves, `survey north` shows a reading, the reading moves.
      ✅ `cargo test -p orbs-sim --test solver` — a Trémaux solver's every
      condition survives the cast, the readings resolve with no maze open, and a
      misspelled `walkd` is still refused.
      ⬜ a scroll that does something — the item below
- [x] **The map that fills in** — `orbs-render/src/maze.rs`, CP437-checked, drawn
      by `orbs/src/shell/labyrinth.rs` beside the instrument panel whenever a maze
      is open. Walls only where the reading has *stood*, `▒` walked once, `░`
      finished with, `☼` the head, `Ω` the way out, and blank for floor nobody has
      walked — the walls around it already say a corridor is there. **Columns, never rows**, whichever way the panel runs: taking
      rows under a `Top` panel leaves the deep-focus floor a five-row transcript.
      It refuses rather than truncates, because a maze drawn short is a wrong maze.

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
- [x] **A solver that actually finishes one** — the acceptance test every other
      test in `tests/solver.rs` was standing in for. *Survives the cast* had been
      quietly doing duty for *reaches the exit*, and they are not the same claim:
      the obvious flat ladder of sixteen `if`s parses, casts clean, walks two
      cells and then **oscillates for ever**, because the `passage` tier steps
      into a fresh cell and the `walked` tier steps back out of it four lines
      later in the same lap. `else` is what fixes it — one move per lap by
      construction — and that is now written down where the next person will
      write a solver.
      **See it:** ✅ `cargo test -p orbs-sim --test solver` — twelve seeds, each
      swept to a shard, worst 5123 ticks against a pinned budget of 6500.
- [x] **`wander`** — the arrow keys walking the labyrinth, because nobody solves
      a maze by typing `follow east` a hundred times. The **22nd tower-wide
      verb**, which `verb.rs` argues for rather than merely counts: the seat is
      `unfurl`'s (a surface with no other way in) and the debt is `follow`'s (a
      domain's word wearing a tower-wide coat until a second verb can be scoped
      to an instrument). Both retire together.

      **Walking takes the pane; watching does not.** `wander` covers the session
      pane like the editor — maze centred, walked count and keys beneath — because
      the prompt is dead while the arrows have the keys and a screen that still
      looks like a session offers something it cannot do. A *spell* solving one
      keeps the map inline beside a live transcript: same picture, different
      activity, and only one of them owns the keyboard.

      **An arrow moves the reading on the frame it is pressed.** `Sim::walk` is a
      third entry point beside `submit` and `step`, and consumes **no tick** — so
      a player walks as fast as they can press and no brew advances while they
      do. Three versions went through the prompt's queue first: per-keystroke
      submission walks at the speed of the *keyboard* (thirty cells on one tick
      from a held arrow), and one aim then a bounded burst walk at the speed of
      the *world*, which is a wait rather than a minigame. The queue was solving
      the wrong problem.

      Replay survives because `Submission::Walked` records *when* — a typed line
      executes at the start of the next tick, a walk has already executed — and
      the driver now lives on `Sim` rather than being hand-written at each of the
      three call sites that had a copy.

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
- [x] **A wall is a square, so a step is one character** — three defects, all
      found by looking at the screen with the tests green, and all the same
      geometry underneath. The map drew an open wall segment as a blank, so a
      path read `▒ ▒ ▒ ▒` — *every other cell visited*. Filling the corridors
      fixed that and left `·` dots on every known-but-unwalked square, two per
      unexplored way out. And a cell-to-cell step still moved the reading **two
      characters**, because cells with walls *between* them draw `2w+1` across.

      No drawing fixes the last one: one character per cell loses the walls, and
      two corridors side by side would merge into a block. So the grid changed —
      15×15 squares, a wall is one of them, the corridor between two cells is
      somewhere you stand. The painter lost its odd/even split entirely, and a
      solver takes about twice as long (1233 ticks worst, was 677), which is
      paid by bound spells rather than by a player.
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=160x45 ORBS_DUMP="attend archive;
      research; wander" ORBS_WALK="<right>"` — the head moves **one** character.
      ✅ `cargo run -p orbs-render --example screens` — a maze that reads as a
      maze.
      ✅ `cargo test -p orbs-render maze` and `cargo test -p orbs-sim --lib maze`
- [x] **16×16, a denser carve, and `back`** — the maze is 176 cells in a 33×23 picture, carved by randomised Prim's rather than a recursive backtracker:
      short passages, frequent junctions, many small dead ends, instead of a few
      very long corridors. The map is **refused** at the 80×22 and 100×28 floors
      because 35 rows will not fit there, so See-it lines that want the picture
      ask for `ORBS_GRID=160x45`.

      **It broke automation, and that is how we learnt the solver was never a
      solver.** The four-tier ladder reads like Trémaux and is not — Trémaux
      turns back *by the passage it came along*, and nothing in §8's language
      could say which that was. At a junction where two ways read alike, a fixed
      compass order sends the reading back where it came from and it **cycles**.
      Measured: 4 of 8 at 16×16, and 1 of 12 on Prim's mazes at 7×7. The
      acceptance test had been proving it about the only mazes the flaw survived.

      One word fixes it: `back`, the way last come from, published as a *second*
      child on that direction. A five-rung ladder solves 12 of 12 on both
      generators at both sizes, in at most 708 steps.
      **See it:** ✅ `cargo test -p orbs-sim --test solver` — twelve seeds swept,
      worst 5123 ticks against a pinned 6500.
      ✅ `ORBS_BOOT=0 ORBS_GRID=160x45 ORBS_DUMP="attend archive; research"` — a
      33×23 block, dark but for one mark.
      ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; research; wander"
      ORBS_WALK="<right>\n<right>\n<right>"` at the 80×22 floor — a window on
      the maze rather than no map at all.
- [x] **One generic fragment, not four named ones** — the yield was
      `shard-of-dawn`/`noon`/`dusk`/`night`, drawn at random. Two things were
      wrong. Collecting a set was **coupon-collector attrition** — 4·(1+½+⅓+¼) ≈
      8.3 solves for one scroll, with no decision in it, since you could not aim
      for the one you lacked; that is §10's *"a duration and no decision
      content"* reappearing in the collection loop instead of the command. And
      **nothing in the game ever said what one was**: no prose, no `recall`
      topic, four invented names standing in for a decision nobody made.

      One `fragment`, four of it, one generic `spell-scroll`, and no roll at all.
      Specific fragments for specific spells is the intended shape and will want
      distinct names again; until those spells exist this is the honest
      placeholder. It cost `Recipe::count` — see below.
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; research"` then solve
      and `survey lectern` — `fragment = 1`, and the panel reads `gathering`.
      ✅ `cargo test -p orbs-sim --test solver`
- [x] **A recipe can want more than one of something** — `Recipe::count`,
      defaulting to 1. What an instrument holds is a node per *name* with a stock
      count on it, so `inputs = ["fragment", "fragment", "fragment", "fragment"]`
      reads like it should work and cannot. Expanding a held stack into one name
      per unit was the other candidate and breaks something already shipping: the
      mortar holding **two** sage against a one-sage recipe reads `charged` and
      fires, which is what *"charged a unit at a time, so a run spends a unit"*
      means. So the recipe says how many it wants, the match asks for at least
      that many, and `transmute` spends exactly that many.

      It also collapsed three hand-built copies of "what is in this instrument"
      into `tower::holdings`, all of which dropped the count on the floor —
      invisible until a recipe wanted more than one.
      **See it:** ✅ `cargo test -p orbs-sim --lib recipe`
      ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; move sage to
      mortar_and_pestle; move sage to mortar_and_pestle; wield
      mortar_and_pestle"` — still fires, and leaves one sage behind.
- [x] **The manual, part one: `help` lists what works here** — **moved from
      Phase 4**, where it sat as *"In-world grimoire (`help` / `man`)"*, because
      §15's onboarding risk plan names it as item 3 and the gate it feeds is
      Phase 1's. One box, moved rather than duplicated, as the settings item was
      moved to Phase 5.

      **It began as a parser defect, not a missing feature.** `help`, `man` and
      `?` were already synonyms of `recall`, whose slot was a *required* topic —
      and a required slot with fillers never yields an argument-less intent, so
      every filler tied and `analyse` returned `Ambiguous`. A lost player typing
      `help` was asked to pick between `archive`, `brewing`, `clarified-draught`
      and `clarity`. §6 forbids a bare error; this was that rule failing at the
      one command whose whole job is answering the question, and nothing caught
      it because no test asked what `help` did.

      `recall` now takes `TOPIC_OPTIONAL` — `survey`'s shape, not the bare-`follow`
      shape §19 declined, because bare and argumented are *the same act at two
      scopes* rather than two different acts. The overview is `RecordKind::Section`
      per group and `Entry` per verb, which `survey` already emits and the tiler
      already packs, so it cost no render code. `Verb::group()` is a new const
      table with no wildcard; the listing itself is `execute::offered`, extracted
      so the boot report and the manual cannot drift.

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; help"` — five
      headings at the 80×22 floor, `grind` among them.
      ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; help"` — no `grind`, because it
      does not resolve there.
      ✅ `cargo test -p orbs-sim --lib recall`
- [x] **The manual, part two: `recall <verb>` is a page** — synopsis, what it
      does, examples, the other ways to say it, and see-also. `Section` headings
      over `Message` lines, because a page is instructions and `Message` wraps
      where `Entry` tiles; `unfurl` pages it for free, searching by record.

      **`NounKind::Command`, not `Topic`**, and the difference is three leaks.
      `Any` reaches `Topic`, so 27 canonicals registered there would have put
      verb words into tab completion (`purge grind`), into `compile::fix` — where
      `if the dispensary has grind` would compile clean and answer *no* for ever,
      verbatim the `has ground-slat` defect — and into the bare-`purge` prompt,
      whose order is (kind, value, slot). So the kind is reachable from exactly
      one *slot* kind, `Subject`, and `Any` refuses it. All three are tested by
      driving them, not by asserting `accepts`.

      **The pages are readable anywhere; the overview is not.** Deliberate: a
      manual you can only read in the right room has a lock on it, and places and
      spells already carry the same exemption.

      The synonyms come off `SYNONYMS` so they cannot go stale. The **synopsis is
      authored** — `signature()` holds no connectives, so a generated `move` would
      read `move reagent place place` — with a test asserting it opens with the
      verb and names every required slot.
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; recall grind"` — the
      mortar's page, read from a room the mortar is not in.
      ✅ `cargo test -p orbs-sim --test naming` — the three leaks, driven.
- [x] **The manual, part three: every verb has a page** — 27 of them, ~170 lines
      of authored prose, and the game went from five sentences of documentation
      to a manual you can read from inside it.

      **Three lints hold it up**, because prose is the one thing tests cannot
      judge. `every_verb_has_a_page` requires `_use`, `_gloss` and `_1` for all
      of `Verb::ALL`, so a verb added later fails here as well as in
      `Verb::group`. `a_synopsis_names_every_slot_its_signature_requires` keeps
      the authored synopsis from drifting from the signature it describes.
      `a_page_never_claims_a_word_that_is_not_there` checks every `see also`
      against the vocabulary — a manual pointing at a word the parser lacks is
      worse than one pointing nowhere, because the player types it.

      Two pages say something a lint could not: `undo` says it is **not built**
      and names `stop` instead, and `bind` says what it costs. §15 wants the
      dead-end rate low, and a page that admits a word does nothing is the
      cheapest way to keep a player out of one.

      **`follow` is exempt from the synopsis check**, with the reason written in:
      its slot is a `Place` because the place half of a spell's condition
      resolves against that kind, which is why the four ways are places you
      cannot stand in. `follow <place>` would be honest about the implementation
      and wrong for a player choosing a direction.
      **A page is sections, not a block**, and it took a screenshot to see it:
      the description is joined into one record so the pane wraps it rather than
      the author hard-wrapping at 50; `Section` draws at the margin so a heading
      outdents from its own body; and synonyms are a row per register rather than
      per phrase. `survey`'s headings got the same for free.
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; recall distil"`
      ✅ `cargo test -p orbs-sim --lib recall` — the three lints
      ✅ every verb: `for v in attend survey peruse ...; do ORBS_DUMP="recall $v"`
- [x] **A fixed 4:3 picture — the grid stops following the window** — the window
      used to decide the *cell count*: 1280×720 gave 160×45, 1920×1080 gave
      120×33, so every pane, border and wrapped sentence was recomputed against a
      grid moving under it. Now the grid is **120×45, always**, and the window
      decides only how big a cell is.

      **The aspect is a property of the grid, not something imposed on it.** A
      cell is 8×16, so 4:3 forces `cols : rows = 8 : 3` and a const assertion
      fails the build if a future edit leaves that line. 120×45 is 960×720 at
      native size — and **720 divides 720, 1080, 1440 and 2160**, which is why it
      beat 160×60: the common display heights land on ×1.0, ×1.5, ×2.0 and ×3.0
      instead of ×0.75, ×1.125, ×1.5 and ×2.25.

      **`ScalingMode::AutoMin { 960, 720 }` is the whole letterbox** — one line
      on the camera, which re-derives on resize by itself. So `fit_camera` is
      gone, `grid::build` lost its scale parameter, and the mesh is emitted in
      virtual pixels. `Fidelity` is deleted entirely; `DEEP_FOCUS_FLOOR` is kept
      because `ORBS_DUMP` still chooses a pane count for an arbitrary
      `ORBS_GRID`, but `drive_panes`' branch on it became `const PANES: u8 = 2`.

      **What it cost, recorded rather than buried:** `F4` no longer changes text
      size, so §9's claim that Wide focus is the large-text mode goes with it and
      the game owes a font-scale setting (§19). And a `BODIES` table transcribed
      from traced geometry had gone stale silently — every test over it still
      passed, because a wrong rectangle is still a rectangle — so it now carries
      an assertion against the layout it was traced from.
      **See it:** ✅ `cargo run -p orbs`, then drag the window wide, tall and
      square: the picture stays 4:3 and centred, the bars grow on one axis only,
      **no text reflows**, and the log prints one `scale` line per resize.
      ✅ the telemetry pane's `scale` row moves while `cols`/`rows` hold at
      120×45; `F4` changes the split and neither of the others.
      ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; survey"` — 120×45 is the
      dump's default now, because a dump that is not the game's screen is an
      instrument reading the wrong thing.
      ✅ `ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_DUMP="attend laboratory; survey"` —
      the authoring floor, which is all `ORBS_GRID` is for now.
      ✅ `ORBS_GRID=40x10 ORBS_DUMP=1 cargo run -p orbs` — the too-small screen
      by grid; drag below 960×720 for the same screen by scale.
      ✅ `cargo run -p orbs-render --example screens` — the scale table that
      replaced §9's tier table, and the worst case at the smallest legible glyph.
- [ ] **Scrolls that do something** — `spell-scroll` assembles and is then an
      object with no use, which is §19's third finding against this item conceded
      rather than dodged. Haste for brewing is the cheapest first use, and it is
      also where the generic fragment becomes specific ones.
      **See it:** spend a scroll and watch a brew run shorter
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
- [ ] Soft ending
      **See it:** reach it, and want to keep playing anyway
- [ ] **Demo + capsule + trailer** — the trailer leads on CRT *motion*, which a
      static capsule cannot carry
      **See it:** hand the demo build to someone cold and watch them play
- [ ] Wishlist target set; revisit the $4 price against actual content volume
      **See it:** a real session's length checked against the price

## Phase 5 — Ship

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

      Three filters (protanopia, deuteranopia, tritanopia) plus a true greyscale
      mode, applied as a post-pass over the composited frame in the CRT shader —
      *after* the phosphor and before the barrel, so it catches the fire, the
      tints and the accent triad in one place rather than needing every palette
      to be solved four more times.

      **The accent triad must stay separable under every filter**, which is the
      property `palette::the_accent_triad_is_separable_without_hue` already
      asserts without hue at all; extend it per filter. The material tints are
      allowed to collapse — they are a convenience over `survey`, never the
      only carrier — but `danger`/`cost`/`success` are not.

      **See it:** set each filter in turn with the laboratory on screen and a
      breach in the transcript; the athanor still reads as fire, the accent
      triad still reads as three things, and greyscale mode has no hue anywhere
- [ ] Accessibility pass (see DESIGN.md §14)
      **See it:** play a full session with the CRT off, at every toggle
- [ ] Screen-reader siege mode (ticks advance on player input)
      **See it:** survive a siege with the screen off, by ear
- [ ] **Sticky skip, persisted CRT-off, reduce-motion** — **moved here from
      Phase 0.5**, where it was the one open item and never belonged: §4 asks for
      skip to be *"a sticky setting, not a per-launch keypress"*, and the
      keypress is the honest half-measure until there is anywhere to persist a
      setting. No `serde`, no `toml`, nothing in the workspace serialises
      anything yet — so it lands with the settings screen below rather than
      before it.

      §14's health warning goes with them, and is no longer urgent: with the
      strike cut, nothing in the game flashes at all
      **See it:** turn the tube off, relaunch, and it is still off
- [ ] Options, remapping, all toggles
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

      **One piece of engineering is left in here and should not be lost in the
      accretion: the first real node.** It turns `take` from a refusal into a
      grant, and it carries what the read-only version deliberately left out — the
      mutator, a `Submission` variant, the queued effect on a tick boundary. It is
      also the only place the choose-between mechanic can be *seen* rather than
      tested, so it is the one item in this section with a See-it line worth
      writing down. Schedule it with whichever content first has two things worth
      choosing between.

      Also waiting on content, and named so they are not rediscovered: the
      **fractional slot charge** (it prices multiplexing, so it needs
      multiplexing), a **speed** upgrade to replace struck invariant 3, the first
      **content gate** — *"brew this to open that domain"* — and `SCALE` in
      `loom.rs`, a provisional round hundred that becomes a derived number once
      the curve reaches it.
      **See it:** cross a threshold, be offered two nodes, take one, and watch
      the other close — rather than a level that arrives on its own

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
