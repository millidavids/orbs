# O.R.B.S. — Roadmap

**Status tracker. Derived from [DESIGN.md](DESIGN.md) §15, which is authoritative.**
If the two disagree, DESIGN.md wins and this file is wrong.

> **Every item carries a "See it" line, and it is a completion gate — not a note.**
> No work item in any phase is done until a person can reach it from the running
> game. An item without a See it line is not started; an item whose line does not
> work is not finished, however green its tests are. DESIGN.md §15, §19.

Last updated: 2026-08-04 · **Phase 0 and 0.5 closed. Phase 1 next; the §15 gate is deferred, not passed**

---

## Shape

Solo, commercial, Steam. **28 months of work against a 24-month target** — that
gap is deliberate and is the cut line's job to close, not a scheduling error to
hide. Release posture: demo first, then full 1.0. No Early Access.

| Phase | Months | Words | Status |
|---|---|---|---|
| 0. Vertical slice | 4 | ~3k | ✅ Closed · numeric gate **deferred** |
| 0.5. Interlude | — | — | ✅ Closed · settings deferred to 5 |
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
| Fidelity tiers | ✅ tier and grid in the border; drag the window | — |
| `Speech` linear stream | ✅ `F5` shows the session pane as a reader hears it | — |
| CP437 repertoire enforcement | ✅ the prompt refuses what it cannot draw | — |
| Parse instrumentation (`ParseLog`, TSV) | ✅ every line traced; `F6` exports | — |
| `sift` / record filtering | ✅ `sift <pattern> orb.log` | — |
| Eldritch + tampered presentation | ✅ `F7` cycles the register; all three faces draw | — |
| Progress bars (`Painter::progress`) | ✅ a brew draws its meter | — |
| Sidebar (`ScreenLayout::sidebar`) | ❌ example only | **Domain panes**, which the brewing plan review cut from this item — §9 opens with two domains at capacity 1, so one is minimised. Needs panes-per-domain to exist first |
| 3- and 4-pane tiling | ❌ the game asks for at most 2 | **Phase 1+ progression.** Gated by *multiplex capacity*, not by domains: §11.5 starts the player at capacity **1** and reaches 3 at ~5 h. §9 keeps panes and capacity as separate unlocks that "must not be conflated" — drawing three panes at t=0 would delete the swap-or-let-it-burn trade the whole focus track is built on |
| `Sim::with_schedule`'s build closure | ⚠️ test-only, and now says so | **Nothing left** — domain systems belong inside `Sim::new`, or the Bevy build, `orbs-tui` and `orbs-balance` each register their own and diverge (§13). Kept because the boundary tests drive it; marked so no frontend reaches for it |
| Replay log (`Sim::submissions`) | ⚠️ written, never read | **Phase 1** — needs a replay command to read it |
| Destruction guard (§7's refusal) | ✅ `purge alembic` refuses; `purge residue-N` works | — |
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
      the framebuffer. Four themes — amber (default), green, muted violet and a
      monochrome light-grey — contrast-solved rather than eyeballed. `ORBS_CAPTURE=1 cargo run -p orbs` screenshots it
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
      inside it (§7), so `decoct` works in the alembic and nowhere else. One
      production slot tower-wide per §11.5's opening capacity of 1, work stored
      as an interval rather than a countdown, and durations deliberately at
      §11.5's routine end because §15's gate is a fifteen-minute scenario —
      DESIGN.md §19. Numbered candidate selection shipped with them, since
      adding nouns is what makes ambiguity reachable and the gate weighs zero
      dead ends above the resolution rate
      **See it:** `attend alembic`, `make a potion of clarity` → `decoct clarity`
      with a meter; `divine sigil-iv` while it brews → refused, naming what holds
      the slot; `decoct nonsense` → a numbered prompt you answer with a digit
- [x] **Log-poisoning sabotage** on brewing logs, via `peruse` / `sift` / `verify`
      — a seeded roll every `DRIFT_INTERVAL` ticks poisons a log; the tell is
      §8.1's **structural** one, a record that lost its tick field, so it is
      spotted by comparing two adjacent lines rather than by reading a warning
      **See it:** ✅ `ORBS_DUMP="attend alembic; decoct clarity; meditate 25;
      decoct warding; meditate 25; verify alembic.log; peruse alembic.log"` —
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
- [ ] **Run the gate** — ⏸ **deferred to Phase 4, not passed.** Blocked on people,
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
      **See it:** ✅ type `survey` in the alembic and watch the listing fill
      across then down. Any keystroke completes it instantly
- [x] **A real boot sequence** — opens on black, then the prompt types itself,
      the pane border draws itself a cell at a time, and a POST card prints
      `O.R.B.S.` in block glyphs a character at a time before checking off
      Blackhearth Games, Rust and Bevy. **14 s, any key skips** — paced to be
      read rather than to be got past: at the first pass's 4.4 s the stages that
      animate were over before they could be followed. The POST is a title card
      rather than a table, so it cannot be mistaken for §4's tower report
      arriving twice. **The world does not tick during it** — `tower::drift`
      rolls once per tick, so the same seed would otherwise build a different
      world depending on how long boot ran
      **See it:** ✅ `cargo run -p orbs`. Or as text:
      `ORBS_DUMP=1 ORBS_BOOT=post cargo run -p orbs`, and `dark`/`prompt`/`frame`
      for the rest. `ORBS_BOOT=0` skips it
- [x] **The tube strike — built, then cut** — a flash and a sweeping band first,
      then the band alone was dropped for reading as a fault, then the flash too.
      It never earned its place: the game is a wizard finding a computer inside a
      scrying orb, and an orb does not power on like a monitor. The screen opens
      black. **Nothing in the game flashes now**, which retires the only
      photosensitivity exposure Phase 0.5 created — DESIGN.md §19 keeps the
      arithmetic, because the *reasoning* was wrong twice and that is worth not
      repeating
      **See it:** ✅ launch the game — it opens black and the prompt types itself
- [ ] **Sticky skip, persisted CRT-off, reduce-motion** — ⏸ **Phase 5**, with
      §15's settings screen. §4 asks for skip to be *"a sticky setting, not a
      per-launch keypress"*; the keypress is the honest half-measure until there
      is anywhere to persist a setting. No `serde`, no `toml`, nothing in the
      workspace serialises anything yet.

      §14's health warning goes with them, and is no longer urgent: with the
      strike cut, nothing in the game flashes at all
      **See it:** turn the tube off, relaunch, and it is still off

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
