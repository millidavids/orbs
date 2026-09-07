# O.R.B.S. — Roadmap

**Status tracker. Derived from [DESIGN.md](DESIGN.md) §15, which is authoritative.**
If the two disagree, DESIGN.md wins and this file is wrong.

> **Every item carries a "See it" line, and it is a completion gate — not a note.**
> No work item in any phase is done until a person can reach it from the running
> game. An item without a See it line is not started; an item whose line does not
> work is not finished, however green its tests are. DESIGN.md §15, §19.

## The list is not an order

**This file is a list of work, not a sequence of phases.** An item is open, or it
is done and carries the phase name it was done under. Nothing here claims to come
next, because nothing here ever reliably did.

`0.<phase>.<step>` assumed phases were built in sequence and **four times they
were not**. Each time cost a document-wide renumber whose only product was
keeping a promise the numbers could not keep, and §19 recorded the scheme as
*"still undecided and now overdue"* after the fourth. Deciding it is cheaper than
a fifth pass.

### What the version means now

> **The minor names the large feature in hand. The patch is an iteration within
> it. `1.0` is the release on Steam.**

**Only the referent changed.** It is what the numbers were already doing — Renown
was minor `11` and its steps were patches `0`–`14` — so no tag moves and no
history is rewritten. What the minor stops being is a *phase index*, which is the
part that could not survive work taken out of turn.

**`0.12` is the orb's menu**, and is not "Phase 12". The tower as one machine
keeps that name as a label in this file and stops being a version claim.

**§19's recorded alternative — *"let the minor count phases closed"* — is
superseded** by the rule above and is no longer a live proposal.

### The renumbers, kept as history

**The tags mean what they meant** — `v0.8.x` is the siege, `v0.10.x` is
Progression, `v0.11.x` is Renown — and §19 twice records a mechanical pass
corrupting the tables that say so. Nothing below is edited again.

| pass | what moved | why |
|---|---|---|
| first | Siege 2 → **8**; the five domains take 2–7 | *"If we don't have a series of interesting puzzles, then there really is no game"* |
| second | Enchanting 6 → **9**, and 7 / 9a–c / 10 / 11 up with it | Phase 8 shipped first, and the POST card's number must never go backwards |
| third | Progression → **10**, everything above up one | built out of turn |
| fourth | Renown → **11**, everything above up one | built out of turn |

**Every pass was done highest-first.** One that moved a lower number first
collided two phases into a single number, and a replace-all once corrupted two
*historical* renumber tables in §19. There is no fifth pass: with the minor
naming a feature rather than indexing a phase, there is nothing left to renumber.

**Phase numbers survive in the prose below as names**, not as an order — *"waits
on Breadth's offline catch-up"* says which body of work, and says nothing about
what comes first.

Last updated: 2026-09-07 · **The phases stopped being an order** and this file
became the list above and below; `0.12` is the orb's menu. **Renown closed at
`0.11.14`** — the second number, spendable, an arsenal worth what your industry
is worth, and a siege that moves renown both ways. **The interlude closed at
`0.11.10`** — every screen in the game crosses rather than cuts, the opening
included.

---

## Shape

Solo, commercial, Steam. Release posture: demo first, then full 1.0. No Early
Access.

**The domains come before the siege**, which was the reordering the whole plan
turned on. *"If we don't have a series of interesting puzzles, then there really
is no game"* — five of §10's seven domains were a single line inside a breadth
phase two phases away, and the siege that *consumes* the puzzles was being built
first. That is settled and shipped; it is recorded here because it is why the
numbers below are not consecutive.

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

### Open — and **this table is not a running order**

| | Months | Words | |
|---|---|---|---|
| [**The orb's menu**](#the-orbs-menu) | — | ~1k | `0.12.x` — **in hand.** A game has a length, and the orb has more than one save |
| [The tower as one machine](#the-tower-as-one-machine) | 2 | ~3k | the dependency web, and two defects it cannot open on top of (§19) |
| [Spellcraft's three](#phase-3--spellcraft) | — | ~1k | the terse register, a typed action at execution, hidden-directory authoring |
| [Enchanting's two](#phase-9--enchanting) | — | ~1k | the shared-engine extraction, split into a refactor and a behaviour box |
| [Breadth](#breadth) | 2 | ~4k | all seven domains, research, full drift, offline progression |
| [Remote hosts](#remote-hosts) | 3 | ~12k | `connect`, infiltration, trace |
| [Engine upgrade](#engine-upgrade) | 1 | — | one Bevy window, isolated from new-system work |
| [Onboarding + demo](#onboarding--demo) | 4 | ~20k | the apprenticeship, the reveal, the soft ending, the capsule |
| [Ship](#ship) | 3 | ~5k | settings, accessibility, store, launch |
| [**Standing**](#standing--work-that-rides-with-the-content) | — | — | ♾ never closes, and blocks nothing |

**~15 months of open work**, against a whole-plan estimate that reached 41 months
on a 24-month target. The gap was already deliberate and *"the cut line's job to
close"*; the [cut line](#cut-line--decided-in-advance) below says how. §16 rates
schedule overrun **Critical**, so the total is carried here rather than dropped
along with the ordering — an unordered list is not a shorter one.

**The five domains were budgeted at 4 months and ~18k words *combined*** while
filed as one line inside Breadth. Giving each its own phase was the admission
that the estimate had been made for a table entry rather than for five minigames,
and it is the single largest reason the total moved.

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
Phase 0 claim that is **not** evidenced. Deferred to Phase 14, which is where
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
| Per-subsystem RNG streams | ⚠️ one of six rolls | **Phase 8 for the rest.** Log-poisoning drift rolls `RngStream::Threat`, so the seeded, per-stream machinery is now exercised by the running game rather than only by tests — and the same seed poisons the same log on the same tick. The other five wait for the subsystems that own them |

**Where a gate is not yet possible, it says so.** A sidebar has nothing to
minimise until domain panes exist, and three panes at t=0 would delete the
capacity trade §11.5 spends five hours building. Inventing a debug affordance
nobody will maintain would be worse than naming the phase that gates it.

The RNG row is what that honesty is worth: it read *"❌ nothing rolls yet —
Phase 8"* until log-poisoning shipped and rolled `RngStream::Threat`. A deferral
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

      ✅ **The output pass** (`0.4.3`) — a correction folded into this item, so
      it **advances no step**. The model was right and the *view* read as a
      config file:
      `[bracketed]` headings over ragged columns, and `distil reagent  kindle
      reagent` running together because the gap between two entries was the same
      two spaces as the gap inside one. Four rules, all in `record/view.rs` and
      all the view's — a heading ruled off beside its words to a fixed stop, a
      slot drawn `<bracketed>`, a run described where its verbs are local to the
      room and tiled where they are the whole tower's, and a blank row opening
      each section. Prose takes a measure of 68 rather than the pane, and
      `status` keeps dotted leaders because its column is the one right-aligned
      thing in the transcript. Details, and the three things it got wrong first,
      in DESIGN.md §19
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; help" cargo run
      -p orbs`, and again with `ORBS_GRID=80x22`. **The page no longer fits the
      floor** and that is recorded rather than fixed — it is a record like any
      other and `PgUp` reaches it
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
- [x] **The audit, and what a look costs** — §8.1 prices free checking exactly:
      *"four free instant checks **are** `verify --all` by another name"*, and a
      flat cheap audit means *"the four-surface model would collapse on move
      one."* Neither half existed. `sabotage.rs` had said the expensive form
      *"arrives with the remaining surfaces in Phase 1"* since Phase 0, and Phase
      1 closed without it, so `verify` was four instant looks that audited the
      tower for nothing. Closed as Phase 1 debt at `0.5.13`, ahead of Phase 8,
      because the siege's step 1 depends on it
      - **Bare is the wide scope, and there is no flag.** §8.1 writes it
        `verify --all`; the parser has no flag syntax at all and gaining one for
        a single word would be a second grammar. Bare-widens is what `survey` and
        `recall` already do, so `Slot::optional` was the whole change
      - **Production-class, so an audit is not a brew.** The run hangs on
        `/tower` and answers to the same tower-wide `CAPACITY` a grind does. The
        duration scales — a base, plus what the orb holds, plus the record
        stream — because §8.1 says it scales with the tower
      - **The cooldown is per *surface*, not per target**, which is what makes
        *which surface do I inspect first* a decision rather than a formality:
        one look tires the orb of logs, or of shelves, and never of both. It
        travels in the save, because a cooldown a player can clear by quitting is
        the shape §19 calls an exploit that then needs its own rule
      - **`State` is the verdict and the surface rides `Kind`.** A refusal
        writing `log` into the field that holds `sound`/`tampered` makes the two
        indistinguishable to `sift`, to §14 and to any test counting answers —
        which is how it was found, by a test counting two where one was given
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; verify;
      grind sage; meditate 25"` — the audit names its duration, `grind` is
      refused with `the tower is busy verifying`, and the verdict lands
      **See it:** ✅ the rationing, and that it is per surface —
      `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; verify laboratory.log;
      verify laboratory.log; verify dispensary; meditate 21;
      verify laboratory.log"` reads `sound`, then `still reading the log`, then
      `sound` for the shelf, then `sound` again once the wait is served
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
- [x] **Run the gate** — ⏸ **deferred to Phase 14, not passed.** Blocked on people,
      not on code: it needs ≥ 8 external testers, at least half with no shell
      experience, over a 15-minute scripted scenario with expected-intent ground
      truth. Everything it measures is built and reachable; what is missing is the
      testers and the script. Phase 14 is onboarding and the demo, which puts
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
      breach* is what will drive `flash` next, in Phase 8
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
      it. §4's *sticky* skip is a different mechanism and still waits on Phase 15
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
Phase 15** beside §15's settings screen, which is where the thing it depends on
is built. A deferred item parked in a finished phase is a phase that never
finishes.

---

## Phase 1 — Core loop ✅ **Closed**

**Exit:** a player automates a duty and feels clever.

**Met, and it is worth saying how.** A spell solves a 176-cell maze on twelve
seeds, brews a clarity end to end, walks the stacks for either errand from one
file, and assembles a scroll — and §8's claim that automation beats doing it by
hand is *mechanical* rather than asserted, because a script ends its loop with
`stop athanor` and a person walks away with the fire lit.

**The exit had a second clause and it is not met.** *"…non-terminal testers are
in the loop"* needs a person: its See-it line is *"someone who has never used a
shell reaches their first bound script while you watch"*. **The clause moves with
its item to Phase 14**, where §15 already puts onboarding validation and the
demo. Ticking it here would be the one thing the See-it gate exists to stop —
§15's own correction is that *"tests prove code does what it was written to do;
they cannot prove it is the code worth writing"*, and no test can prove this.

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
        and manual topics are **derived from the `recall_` keys**
        (`Prose::topics` strips the prefix) so authoring an entry makes it
        nameable without touching Rust
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
        with Phase 15's settings item above, which now has a real dependency
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
        and puts it in Phase 13a, covering bound scripts rather than a lit athanor
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
      concurrently until Phase 8 reserves slots and Phase 13a researches them —
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
      promise moved to Phase 15's colour-vision filters — recorded there and in
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

      **See it:** ✅ the eleven-command line in SEEING-IT.md prints `green`, `bone`
      and `green+bone` as three regions; step its last `meditate` and the
      mixture grows as both bands shrink. ✅ `ORBS_DUMP="...; digest
      ground-sage; meditate 20"` for a bath that has finished and is still
      moving — a dump cannot show that, so ✅ `cargo run -p orbs` for the rate
- [→] **Remaining sabotage surfaces — split, and moved.** The item named four
      surfaces and two of them cannot be built yet: §5.1 makes adversarial
      aberrations **siege-only** — *"the enemy never touches scripts, schedules,
      or logs in the calm layer. Phase A stays genuinely safe, which pillar 4
      requires"* — so **script text and trigger clocks have no producer** until
      the siege, and an item whose See-it line is unreachable holds its phase
      open for ever. The **world** surface goes to Phase 2 with scrying, which is
      what makes it legible. The other two go to Phase 8.
      **See it:** the log surface already works —
      ✅ `ORBS_DUMP="attend laboratory; grind sage; meditate 25; grind rock-salt;
      meditate 25; verify laboratory.log"` reads `state: tampered`
- [x] **The archive is a maze, and the world holds the search** — §10 calls the
      domain *bespoke* and *"stales fastest"*, and it was five entities with a
      verb that consumed nothing and produced nothing. `research` now resolves
      the stacks out of a page, `follow` threads them, the way out gives up a
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
        stacks spends it on `research`
      - The instrument retired three defects at once — no completion sentence, an
        unstoppable run, no `recall` — and with them `DIVINE_TICKS`,
        `pipeline::work` and `progression::DIVINE`

      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend archive; research; survey north; follow east"`
      — the stacks resolve, `survey north` shows a reading, the reading moves.
      ✅ `cargo test -p orbs-sim --test solver` — a Trémaux solver's every
      condition survives the cast, the readings resolve with no maze open, and a
      misspelled `walkd` is still refused.
      ⬜ a scroll that does something — the item below
- [x] **The map that fills in** — `orbs-render/src/maze.rs`, CP437-checked, drawn
      by `orbs/src/shell/stacks.rs` beside the instrument panel whenever a maze
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
      swept to a shard, worst 5699 ticks against a pinned budget of 6500.
- [x] **`wander`** — the arrow keys walking the stacks, because nobody solves
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
      worst 5699 ticks against a pinned 6500. (It said 5123 here for two
      versions; pinning the figure as an equality is what found it.)
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
      and `survey cabinet` — `fragment = 1` on the archive's shelf.
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
      Phase 14**, where it sat as *"In-world grimoire (`help` / `man`)"*, because
      §15's onboarding risk plan names it as item 3 and the gate it feeds is
      Phase 1's. One box, moved rather than duplicated, as the settings item was
      moved to Phase 15.

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
- [x] **The manual, part four: every *item* has a page** — 34 materials, and the
      half of the manual that was missing. `recall <thing>` answered with the
      **route** and nothing else, so a player holding a potion could be told its
      five steps and not one word about what it was for. A route answers *how do
      I get one*; someone holding the thing is asking *what is this*.

      Both are said now, and **what it is comes first**. `using_<name>` is the
      second half — how you spend it — and it is optional, because a byproduct is
      something you have rather than something you do. Where the use is **not
      built** the page says so: every potion's reads *"nothing drinks a potion
      yet. a siege will be what spends them"*, which is `undo`'s precedent and
      §15's argument that a page admitting a thing does nothing is the cheapest
      way to keep a player out of a dead end.

      **Not `recall_<name>_use`**: `Prose::topics` decides what is nameable by
      stripping `recall_`, so that spelling would have registered `clarity_use`
      as a subject — the trap `grimoire_step_or` already paid for.

      Two lints hold it, beside the three that hold the verb pages:
      `every_material_has_a_page` fails the build by name for a material with no
      description, and `a_finished_product_says_what_it_is_for` requires a
      `using_` line on every potion and scroll.

      **It made every material a `Topic`**, which is the same exemption verb
      pages have and for the stated reason — *a manual you can only read in the
      right room has a lock on it*. §7's scoping is unharmed and now says so more
      precisely: from the archive `sage` is something to read about and not
      something to grind, which is a claim about the **kind** and is what
      `what_an_instrument_holds_is_nameable_from_the_room_it_stands_in` asserts.
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="recall clarity"` — what it is, that
      nothing drinks it yet, then the five steps that make one.
      ✅ `ORBS_BOOT=0 ORBS_DUMP="recall gleaning-scroll"` — and how to spend it.
      ✅ `cargo test -p orbs-sim --lib recall` — five lints now
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
      ✅ the **tower rail's foot** shows `scale` moving while `grid` holds at
      120×45 — it was the telemetry pane's row until Phase 2 replaced that pane.
      **The `F4` half of this line is withdrawn**: `PANES` is 1, so both tilings
      are identical and the key is visibly inert until multiplexing returns the
      second pane (§19). The property it held is now asserted against an explicit
      two-pane request instead, because it is about the tiler and not about
      today's pane count.
      ✅ `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; survey"` — 120×45 is the
      dump's default now, because a dump that is not the game's screen is an
      instrument reading the wrong thing.
      ✅ `ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_DUMP="attend laboratory; survey"` —
      the authoring floor, which is all `ORBS_GRID` is for now.
      ✅ `ORBS_GRID=40x10 ORBS_DUMP=1 cargo run -p orbs` — the too-small screen
      by grid; drag below 960×720 for the same screen by scale.
      ✅ `cargo run -p orbs-render --example screens` — the scale table that
      replaced §9's tier table, and the worst case at the smallest legible glyph.
- [x] **Scrolls that do something** — `spell-scroll` assembled and was then an
      object with no use, which is §19's third finding against this item conceded
      rather than dodged. Four steps, and the order is *what can be shown without
      the next thing existing*. DESIGN.md §19 records the decisions.
  1. ✅ **A scroll is a kind, the lectern rolls one, and `wield` spends it on a
        gleaning errand.** Merged deliberately, on brewing step 3's precedent
        (line 527 below): a step that ships a scroll you cannot spend leaves this
        item's dead end open and its See-it line reads *"look at the thing that
        does nothing"*.
        **The spend is `wield`, not a 23rd word.** `verb.rs`'s vocabulary test
        refuses one in advance — *"22 is a number to defend, not a budget to
        spend"* — so the verb learned a second argument kind (`NounKind::Workable`,
        the `Stoppable` pattern) and `empty` kept `PLACE` so `empty
        gleaning-scroll` never parses.
        **What four fragments become is drawn**, not fixed: nothing about the
        inputs could decide it, and a lectern that always made the same thing is
        §10's *"a duration with no decision content"* one level up. The list is
        on the recipe rather than three `[[lectern]]` blocks, because
        `Recipes::matching` returns the **first** match and the other two would
        be unreachable with the file looking reasonable.
        **The errand is the modifier a spell can ask about**, published on the
        stacks as a named child exactly as a way publishes `passage` — so `if
        the stacks has gleaning` works through the `has` question §8 already
        has: no new `State`, no panel change, no new grammar. **One** solver then
        handles both errands. `Errand::ALL` chains onto the readings in
        `scene_at`, because the condition must resolve at *cast*, when there is
        never an errand on.
        **A gleaning maze publishes no exit at all.** An inert one would have a
        solver walk onto it, find the walk not over, and take the same rung from
        the same place for ever — so the picture withdraws `Ω` too, or the map
        would offer a way out the readings do not.
        **Five spoils against a scroll's four, profitable on purpose** — gleaning
        is what keeps scrolls in circulation. It is also the item's largest
        balance exposure and **nothing sweeps it**: `orbs-balance` is a stub, so
        the number is a placeholder, not a tuned one.
        Two defects fixed on the way: **`holdings` counted a reading as stock**,
        so an errand on an instrument would have broken the four-fragment recipe
        and drawn `fouled` for an instrument with nothing wrong with it; and
        **`debug_spawn` could not reach the archive at all** — there is one
        `Store` and it is in the laboratory — so it gained a destination
        (`debug_spawn fragment 4 lectern`), which `tests/solver.rs` had already
        written down as impossible.
        **See it:** ✅ `ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend archive;
        research; debug_spawn gleaning-scroll; wield gleaning-scroll; wander"`
        — five `♦` scattered through the maze and
        **no `Ω`**.
        ✅ the same without `wander` plus `survey stacks` — `reading: gleaning`
        on the stacks, which is the word a spell asks for.
        ✅ `cargo test -p orbs-sim --test gleaning` — nine claims driven through a
        real `Sim`, including one solver taking a different branch under each
        errand, and the lectern still assembling with an errand open.
        ✅ `cargo run -p orbs-render --example screens` — the fourth stacks
        screen, through the real painter.
  2. ✅ **The arsenal — `/tower/arsenal`, and the one room reachable from every
        other.** Nothing in the tower could be carried between domains **at
        all**: `carry`'s destination lookup wants a `Fixture` child of `cwd` and
        a domain is neither, so `move clarity to archive` could not resolve and
        neither could any route between two rooms. §10's remaining five domains
        all have that problem waiting for them, so this is the standing answer
        rather than a fix for one pair — and it is the prerequisite for
        quickening rather than a convenience beside it.

        **A finished potion could not be picked up at all**, and had not been
        able to for as long as there have been potions: `move`'s first slot was
        `Reagent` and `produce::transmute` gives a `potion = true` output
        `Essence`, so the slot silently never filled. Nothing noticed because
        `empty` turns an instrument out wholesale and never asks what kind
        anything is, so the one route that mattered *inside* the laboratory
        worked. `NounKind::Portable` is the fix — the `Stoppable` pattern again —
        and it is **not** `Any`, which would make `move laboratory to arsenal`
        resolve at full confidence.

        **The exemption is narrow and it is stated.** `scene.rs` names acting on
        a domain you are not in as **Phase 8's** unlock, and this does not repeal
        it: what reaches everywhere is the arsenal's *contents*, on exactly the
        terms places, spells and the maze's readings already have. What keeps it
        honest is the door — **finished work only**, asked of the *kind* and
        never of the name, because telling work from stock by name would mean the
        tower deciding which reagents are waste, which §10.1 refuses outright. So
        it cannot become a second dispensary, and `reachable` searches it **last**
        so a reagent in the room always outranks a carried one.

        **Nameable is not enough, and that is the whole risk.** Registering the
        contents in the scene makes them nameable everywhere — and `purge` and
        `verify` take `NounKind::Any`, so both would have *resolved* on a potion
        from any room and then reported it absent: §15's dead end, arriving
        through the affordance meant to remove one. Three lookups had to learn
        it (`pipeline::reachable`, `pipeline::purge`, `files::here_or_place`), and
        `tower::keep` owns the rule so they cannot disagree.

        **It is also what lets a spell touch a potion.** A spell is written *for*
        a domain and `may_issue` forbids it `attend`ing, so finished work living
        in the room that made it could never be reached by automation running
        anywhere else.

        Free from being a top-level branch: `Protected`, so `purge arsenal`
        refuses in character; a row on the boot report; and `arsenal.log`, which
        actually fills because `move` already stamps `Path`.
        Two more defects settled on the way. **`fragment` was two noun kinds at
        once** — `Fragment` from a solved maze, `Reagent` from `debug_spawn` —
        and `stock::give` merges by name, so the two would have merged into
        whichever node was found first. And **`Recipe::leaves` was compulsory**,
        so the lectern shed a `dust` invented to fill the field — which was
        trapped in the instrument that made it, and whose mortar recipe could
        never fire, because reagents do not cross a domain boundary. §10.1 builds
        the waste-has-a-use loop around *brewing*, where a second route to the
        same draught can exist; the archive has no second route to anything. So
        `leaves` is optional and byproducts stay the laboratory's mechanic.
        **And the archive became three fixtures doing one thing each.** The maze
        opened *on the lectern*, which was also where four fragments
        became a scroll — §19 called that "the first instrument that can be doing
        two things at once" and treated it as a curiosity. It was a design
        problem: `stop lectern` had to guess which it meant, the panel gave both
        one row, and a twenty-tick assembly drew the maze's explored-cells gauge.
        So the maze moved to the **stacks** — an endless library you navigate —
        the **cabinet** is the archive's shelf, and the **lectern** is where four
        matching fragments are moved out of the cabinet and assembled. A walk of
        the stacks pays onto the cabinet, so the hoard is on `survey cabinet`
        instead of hidden in the instrument that consumes it, and `debug_spawn`
        already put one there — a split this closed. `[earns]` widened with it:
        its keys were checked against `recipes.toml`, so an instrument that runs
        and transforms nothing could never be priced, which was quietly true of
        the athanor all along. And `tower::home` now derives
        **where every item belongs** from the recipes that name it, so
        `debug_spawn <thing>` puts the thing in the room it is used in with no
        destination named. Three lints hold that: a new item is testable the
        moment it is authored.
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
  3. ✅ **Quickening — a window on the laboratory, not a shot at one run.** For
        `QUICKENED_TICKS` the room works at double speed: everything it starts
        inside the window takes half as long, and whatever is already running is
        hurried too, because the state means *this room is quick* and a run in
        flight is something the room is doing.

        **It was a one-shot and refused when nothing was running**, which made it
        unusable at exactly the moment a player reaches for one — *quicken the
        laboratory, then brew* is the obvious play and was the one thing it could
        not do. Four walks bought a single stage; a window buys a stretch of
        work and rewards lining it up.

        **An interval, like `Burning`**, so it survives `meditate` with no system
        ticking it down. **Read at `begin`, like heat**, so a run started inside
        the window stays short when the window closes — which is what keeps
        `Working` an interval set once, the property `meditate` idempotence rests
        on. A run already going is halved from **now**, not from the start:
        halving the whole interval would refund time already spent.
        **See it:** ✅ an 8-tick grind, twice —
        `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; grind sage; meditate 4"`
        yields nothing, and the same line with
        `debug_spawn quickening-scroll; wield quickening-scroll` in front of it
        yields ground-sage.
        ✅ `cargo test -p orbs-sim --test gleaning` — the short run, the run that
        stays short when the window closes, the in-flight halving and its point,
        and the two clocks agreeing.
  4. ✅ **The verdant scroll, and two potions off ported herbs** — `mugwort`,
        `valerian` and `amber` from `../court_wizard`, whose alchemy is nineteen
        real herbs each with prose saying what it looks like and where it grows.
        Its *effects* do not port; the half that does is a substance a player can
        be told about, so each has a `recall` page.

        **One herb per scroll, not all three** — the plan said unlock *the*
        reagents, and one scroll doing all of it would have left the lectern
        assembling a thing with nothing left to give: a dud draw for ever, which
        is the dead end this item exists to close. Three are each worth having.

        **What is unlockable is derived, never listed.** A base reagent is one the
        vocabulary knows that nothing in the tower makes, whose home is the
        laboratory's shelf — so a fourth herb authored in `recipes.toml` is
        unlockable the same tick with no Rust to touch. Getting *made* wrong was
        visible rather than subtle: the first version asked `Recipes::outputs`,
        which is a recipe's `output` and not its `leaves`, so every byproduct read
        as a herb and four scrolls shelved `dregs`, `ash` and a `fragment` as
        inexhaustible stock. **One dump found it and no test would have.**

        Names measured, not chosen: `keen-draught` and `quiet-draught` replaced
        `steeped`/`settled`, which collided at **734** — worse than the 667 that
        got `decant` renamed — and `verdant-scroll` replaced `flowering` at 688.
        The worst pair now is 693, against the 819 (`ground-sage`/`ground-salt`)
        and 896 (`sage`/`sage-tincture`) already shipping.
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
- [x] **A spell counts, and the archive's solver is a tester's tool** — asked
      whether §8's language could automate the archive for one fragment *and* for
      a gleaning maze's five. **It already could**, and every part was checked in
      the running game first: one file solves both errands, re-opens the stacks,
      and assembles a scroll. What was missing was one word and two guarantees.

      **`has` takes a count, and it closes a live defect rather than an absence.**
      `if the cabinet has 4 fragment` parsed as `if cabinet has fragment` — the
      number swallowed with no fault, and `interpret` showing the shorter
      question, which is §19's *shorter command than it heard* arriving through
      the surface built to catch it. **At least, never exactly**, so a guard does
      not jam when a solver gets ahead of it; `has 1 X` writes back bare and
      `has 0 X` is `has no X`. Answered off the node's own `Stock` and
      deliberately **not** through `tower::holdings`, which skips `Sense` children
      and would have answered *no* to `if north has passage` for ever.

      **`debug_spell` writes a known-good ladder, in debug builds only.** Not a
      player-facing starter spell: §12 has `first_light` stop early *"on purpose"*,
      and a complete twenty-four-rung ladder is the answer to the archive's
      puzzle, `back` included. It refuses outside the spell's own domain — unlike
      `debug_spawn`, because `scribe::write` homes to where you stand — and
      records the *write* rather than the typed line, or one input would log two
      submissions and a replay two more.

      **One ladder, both errands, now asserted.** `spoil` and `exit` are both
      tiers, so no errand check is needed; that was true since the spoil rung was
      written and lived in a **doc comment**, with the test only ever pointing it
      at a gleaning maze. Two ladders are kept on purpose: `tests/solver.rs` pins
      6500 ticks to notice the language getting slower, and four always-false
      `spoil` rungs cost +2832 over 708 moves.
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
      ✅ the same line with a `gleaning-scroll` wielded first — five fragments,
      same file.
      ✅ `cargo test -p orbs-sim --test gleaning` — one ladder over both errands,
      four seeds; `--test questions` for the counted grammar; `--test debug_spell`
      and `cargo test --release -p orbs-sim --test debug_spell` for the door.
- [x] **The spell language, audited against seven others** — asked whether
      `.spell` could do what this game needs, keeping *state lives on the object,
      so no variables*. Read Autonauts, HyperTalk, AppleScript, Inform 7,
      Factorio, Oxygen Not Included and the Zachtronics assemblers.

      **Most of it was already right**, and §19 records why: no variables
      (Factorio and ONI agree), no signal bus (a spell leaves a thing on a shelf
      and another asks about it, which §8.1 requires anyway), `wait` before `if`,
      and a per-step tick cost. Inform 7's automatic rule specificity was
      **refused** — it would make execution order implicit, where §8.1's contract
      is that the log names the line.

      **One word was missing.** Three of the seven have `repeat until` as their
      primary loop; we had a count, an unbounded loop, and no way out of either —
      so the shipped solver said `repeat 20000`, a number guessed to outlast the
      longest walk. `until` is the sixth control word and it *deletes* that
      constant. Guard asked before the first pass and after each, per Autonauts;
      a question it cannot answer **stops** the loop, opposite to `if`.

      **`has` gained comparators, and the maze stopped discarding its own
      count.** `walked` and `twice` were two buckets over `Square::marks`, so a
      square walked nine times read like one walked twice. They are `marks` now
      — `1 or fewer` / `2 or more` — and §19's *"the language did not need to
      grow"* is amended rather than quietly contradicted.

      **And `recall` teaches the language**, which nothing did: control words are
      outside `Verb::ALL`, so `recall repeat` reached nothing at all.
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
- [x] **Every spelling of a comparison, and a manual for the language** — the
      audit above shipped one spelling and swallowed the others: `has at least 2
      marks` became `has marks`, count and words gone with no fault, because `at`
      is filler and the rest resolved to the noun. Seventeen spellings now read,
      including `=`, `>`, `>=`, `<`, `<=` both spaced and glued (`>=2`).

      **`=` brought a third bound**, `exactly`, since reading it as at-least
      would be the quiet reinterpretation §6 forbids. Strict comparators carry an
      offset rather than variants of their own — `more than 2` is at-least-3 —
      and symbols are accepted at the door but never written back: the fair copy
      says `has exactly 2 marks`, which someone who has never seen an operator
      can still read.

      **And the manual the language never had.** Control words sit outside
      `Verb::ALL` and readings are `NounKind::Sense`, so `recall repeat` reached
      nothing and `recall marks` answered with a message about other rooms — a
      dead end wearing a wrong reason. A page per control word, a page per
      reading, and `recall scripting`, whose last section is built from the room.

      **One lint stopped skipping.** Both colour checks passed over a recipe
      whose input had no wash, so omitting a tint switched them off rather than
      failing them — which is how two tinctures shipped colourless and took four
      products' coverage with them.
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

Closing a phase means saying what happened to everything in it. Each moved for a
stated reason rather than being swept somewhere convenient — **and only two of
the nine went to `Standing`**, because Standing's own admission rule is that an
item belongs there when it *cannot close*, and most of these can.

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

**Content data format is ticked rather than moved**, and the residue is named.
Its See-it line — *"edit a line of prose with the game running and see it
change"* — is verbatim a ✅ line above: `Prose` hot-reloads, and rule 6 has held
since Phase 0 with lints enforcing it. What is **not** done is *recipe* reload,
which is deliberately excluded (`content/recipe.rs`) because a recipe is a
decision and swapping one mid-session breaks replay from `(seed, submissions)`
unless the content is versioned into the submission log. That is a real item and
it is Phase 13b's, with remote hosts, where content versioning arrives.
- [x] **Content data format** — hot-reloadable TOML/RON, prose never in Rust
      literals. Recipe reload deliberately excluded; see above.
      **See it:** ✅ edit a line of `prose.toml` with the game running

---

## The domain phases — the rest of the tower

> **This was *"Phases 2–7"* and the range no longer holds.** The domains are
> phases 2, 3, 4, 5 and 9, with 8 (Siege) among them and 10 after — see
> *The renumbers, kept as history* at the top of this file. Those numbers are
> names now, not an order. The section covers the same five domains it always
> did, and all seven are raised.

**Read this before starting any of them.** §10's table gives each domain a
*form* — "deduction", "composition", "allocation" — and says outright that
**these forms are a table, not a design**, because Phase 0 built brewing and
archive as commands with a duration and no decision content, *"which is what the
column exists to prevent"*.

Brewing is the worked example the rest are cut from, and §10.1 says what that
means. Every phase below is measured against six rows:

| | In brewing |
|---|---|
| A shared, **depleting** resource | **lit time** — the athanor burns per lit tick whether or not anything is mounted |
| **A window at 1 Hz, never a reflex** | light → digest → damp → combine → relight → distil |
| Cost is the resource, **never progress** | wasted fuel, so §11.5's *"never ruinous, only slower"* holds |
| The decision is **choice given readable state** | never how fast or precisely the player acts |
| The world publishes readings, so a spell holds a **rule, not a memory** | §8.1 forbids automation driven by hidden state |
| The best play **falls out of the durations** | *"That was not designed."* Damping was found when the fire died mid-test |

**A first draft of these six phases invented two scarcities the shipped game
contradicts**, and the correction is worth keeping because it is the trap:

- *"The forge burns the same charcoal the athanor does — one supply, two fires."*
  Charcoal is `Holding::endless`, and `build.rs` says why: *"a cold athanor with
  nothing to burn is a laboratory with nothing to do."* Fuel **stock** was never
  the scarcity. **Lit time** is.
- *"Looking through the lens costs a potion."* §5.1 is **"issuing commands is
  free; repairs consume resources"**, and §8.1 has already priced looking with a
  per-surface cooldown. A resource cost on *looking* would also push a player
  without potions onto the visual tell, which §14 makes a bonus and never a
  requirement.

**`CAPACITY = 1` is tower-wide**, so §5.0's *"concurrency is the real scarcity"*
is a resource every domain already shares and none of them needs to invent one.
Four of the six below spend it.

---

## Phase 2 — Scrying ✅

**Exit:** a player breaks a far wizard's ward by deduction, and then a spell does
it unattended. **Met** — `probe`/`dial` at the prompt, `debug_spell breaking`
bound in the lens while the player brews.

§10 gave this domain one line — *"deduction: parse noisy logs to find truth"* —
and says outright that the forms are **a table, not a design**. The mechanic is
now a code-breaker: a ward of four sigils drawn from six, **repeats and all**,
1296 codes. The noisy logs survive as the **yield** rather than the mechanic — a
broken seal spills the far wizard's laboratory into `lens.log`, and now and then
it spells out a recipe.

**The exit criterion changed, and §19 records why.** *"Which of two contradicting
accounts is lying"* was the only mechanic ever proposed for the domain and it was
never designed. What replaced it had to survive one hard finding: Mastermind has
**almost no skill ceiling** for a machine — consistent guessing solves in ~5
presses — so an orb that does the bookkeeping has done the whole puzzle, and
`parser::question` is stateless so a spell can only act on what the world writes
down. **Two channels onto one ward** is the resolution: the two counts for a
person, the two *deltas* for a spell, 5.15 presses against 11.93.

**And the ward was rebuilt once, at `0.3.22`.** It drew four sigils with **none
repeated**, which forced a dial to *exchange* two sockets — and an exchange makes
`aligned` rising unattributable, so the orb propped it up with a ratchet, a
settle-lock and a per-socket `untried` count. All three answer *is this position
correct?*, which is the one question a codemaker never answers. Repeats delete the
exchange and all three go with it. §19's *"The lens is Mastermind now"* has the
whole of it; the boxes below are re-verified against the new rules rather than
reopened, because the exit criterion never moved.

**It is cross-referencing *and* detection now.** `verify` caught a poisoned log
from Phase 0; the **world** surface ships here, so a swapped reagent is found the
same way.

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
      → `laboratory ‼`, cleared by going to look. **Unplanned and kept:** the
      archive's maze stops panning, because 102 columns fits the whole picture.
      **Each box is ruled off from the next**, and drawing the rule found two
      defects that whitespace had been hiding — `lay_rail`'s uneven first box, and
      a spell marker (`▸`) that is not in CP437 and drew as `?`. Both in §19. A box
      using all four of its rows, against the rule that closes it:
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
      **No `meditate`, because a press is instant and takes no slot** (§19).
      ROADMAP's own *"a read is not a brew"* scarcity for this domain is
      withdrawn: the lens now competes with the laboratory for nothing.
      `survey prism` reads the two counts and the two deltas; `survey second`
      reads **only what is in it**, because whether a socket is right is the
      player's own bookkeeping. Two channels onto one ward — the counts for a
      person, `closer`/`level`/`further` and `richer`/`unchanged`/`poorer` for a
      spell — and the second is strictly *weaker* than the first.
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
      **Six glyphs, not six colours** (§14) — `▪` failed the CP437 check and
      became `■`. Spoken once as a summary, never cell by cell.
      **The aperture row used to carry `■···` settle marks** and does not: the
      orb no longer decides that a socket is right (§19).
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
      `recall mending` falls through to the bare overview before the discovery
      and prints the full route after it — **unmakeable, unnameable and
      unreadable, all flipping on the same tick.** The alembic reads `fouled` on
      a load it cannot transmute, which is honest rather than coy.
- [x] **A solver spell for the lens.** `debug_spell breaking` — one sweep, four
      socket phases, reading nothing but which way `aligned` moved. It cannot
      deduce; what makes a blind walk converge is that **one socket moves at a
      time**, so a rise can only be that socket arriving. Trémaux's property in a
      second shape.
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
      **It was 24 rungs, then 4, and it is 4 phases of 3** (§19). `dial <socket>`
      takes an optional sigil, so a spell can say *try something else here*
      without naming what — the one sentence the variable-free language could not
      form. What the ratchet and the settle-lock used to do, a **restore rung**
      does explicitly: `aligned` falling means that socket was right, so put the
      opening sigil back and press again.
      **`bind`, not `invoke`, and that is new.** A sweep breaks the ward in front
      of it and stops. A binding re-casts a spell that has run off the end, which
      is the faucet.
      **No contention at all**, where this line used to measure it: a press takes
      no slot, so the solver runs beside a full brewing loop rather than sharing
      12 ticks in every 13 with it. `cargo run -p orbs-balance -- run scrying`
      is what pins the rate.
- [x] **The world sabotage surface**, moved here from Phase 1 — §8.1's second of
      four. A reagent is **substituted**: its name changes and its identity does
      not, so a spell that named it stops working and `verify` names what is
      wrong. Both of §8.1's channels, because either alone is a defect.
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
      **Endless base stock only, and never fuel**, and that restriction is
      load-bearing twice over: a first pass swapped `ground-sage` mid-pipeline,
      which destroys work in flight rather than misdirecting, and also broke a
      pipeline test that has nothing to do with sabotage — which is how a nuisance
      that reaches too far announces itself. Charcoal was the second: it is named
      by no recipe, so swapping it stops every heated stage in every domain rather
      than the one spell that named a reagent.
      **A lie nobody catches settles back to the truth**, and `orbs-balance` is
      what made the case. A spell names things with literals, so it can never
      `purge sage-` — a word nobody knew when it was written — which makes
      `verify` → `purge` human-only and made an unattended tower *terminal* rather
      than harassed: the standing grind loop fell to 0.058/tick and stayed. One
      swap an hour, five minutes of trouble, 8% downtime, every pinned rate back
      inside its band. DESIGN.md §19 has the sweep.
      **See the ambient half**, which no dump can reach — it is an hour of world
      time, and `debug_swap` above is the tester's shortcut past it:
      ```bash
      cargo test -p orbs-sim --test tampering   # settles; never takes the fire
      cargo run -p orbs-balance -- sweep --ticks 7200
      ```
- [x] `orbs-balance` CLI sweeping §11.5's first-pass numbers, moved here from
      Phase 1 — **before** five phases author durations on top of unswept ones.
      Six policies, a per-tick rate against a pinned reference, and `--why` for
      the sentences behind a refusal. **Four findings on its first clean run**,
      recorded in DESIGN.md §19: the design's rates are recipe-tick
      idealisations, §10.1's damping is *behind* not ahead, haste still
      dominates the flagship, and the archive maze is 10–20× below it.
      **And one regression, caught one item later** — the ambient reagent swap
      above, which it flagged on all four pinned policies at once while the whole
      suite stayed green.
      **See it:**
      ```bash
      cargo run -p orbs-balance -- sweep --ticks 7200 --why
      cargo run -p orbs-balance -- run clarity --ticks 1200 --why   # cost 34, all scours
      cargo test -p orbs-balance                                    # the anchor *and* the pins
      ```
      **`--ticks 7200`, not `--hours 1`.** A policy amortises its first lap's setup
      over the run and the sabotage surface costs a few minutes an hour, so at 3600
      one badly-timed swap moves a rate by more than its tolerance band and the
      table flags the seed rather than the game.
      A sweep's `clarity` reads 0.1400 against a hand-played `status` reading
      `experience 16` at tick 123 — 0.130 — and the gap is the first lap's setup.
      **`cost` is the column to read first:** every entry should be a scour the
      policy asked for. Anything else means the loop has fallen out of phase with
      the tower and the rate beside it is measuring nobody's game.
      **`scrying` is the sixth of seven**, added with the ward rebuild at
      `0.3.22` so that
      the lens's automated rate is an instrument reading rather than a sentence
      in §19 — where it had been priced at 0.022 and was 0.07. It reads **0.268**
      now, above the flagship's 0.140, and it is the only policy here that never
      waits on the tower: a press takes no slot, so the number is additive to
      whatever the laboratory is doing rather than an alternative to it.
**Scarcity: none, and this is the one place the phase's plan was withdrawn.**
*"A read occupies the tower for its duration exactly as a brew does"* was the
stated scarcity, priced at twelve ticks a press. `PRESS_TICKS` is 0: a press
answers on the tick it is typed and takes nothing, so a bound solver runs beside
a full brewing loop with no contention at all (§19). §8.1's cooldown still prices
repeated looking, and is what remains.

---

## Phase 3 — Spellcraft

> **Where this stands, and read this first if you are picking it up cold.** The
> save format is closed (three boxes). The **language overhaul** is four boxes
> in of seven, reaching `0.3.20`: `else if`, a bound-spell policy in
> `orbs-balance`, comparison-against-a-place, and variables-and-sets all ship.
> **The language overhaul is closed at `0.3.26`.** Seven boxes: `else if`, a
> bound-spell policy, comparison-against-a-place, variables and sets, parts with
> the budget-from-the-weave wiring, the resolution policy, and the cast-time half
> of builtins. `RecordKind::ScriptLine` went with them at `0.3.23`.
>
> **Two things are deliberately left**, and both are boxes below rather than
> omissions. The **terse register** is planned and unbuilt — nothing needs it.
> And *builtins that act* turned out to be two items, not one: the plan assumed a
> whole `Intent` could be typed at cast, and running `interpret` before writing
> any of it showed that **an argument cannot** — a spell makes its own inputs, so
> `digest ground-sage` reads back at cast as bare `digest`. The verb half shipped;
> the execution round-trip is its own box, with its regression guard already
> written. §19 records the falsification.
>
> **Spellcraft's exit criterion is met**, and it was rewritten to get there: a
> spell factors into named parts and reads as one file. Cross-file part sharing
> is **struck** (§19) — a spell is contained to a single `.spell` file — and *what
> a part costs to hold* went with it, because an in-file part is not held
> separately and so has no price to draw.
>
> **What is left in this phase** is the ~35-verb naming pass, which §18 lists as
> blocking and which goes ahead of the domains that coin the most verbs; the
> hidden-directory authoring plan; and the two language items above that are
> deliberately unbuilt — the terse register and the typed action at execution.
>
> **`0.3.21` and `0.3.22` are not this phase.** They are Phase 2 and Standing
> work — a review of the input paths, and the lens rebuilt as Mastermind — done
> on another machine in parallel and rebased on top. Nothing in the language
> overhaul depends on them; what they cost this phase is the `breaking` solver
> being rewritten against a ward that no longer publishes `untried`, so the
> `else if` box's own example spell is not the one in `dev_spells.toml` any more.
>
> The language boxes are worth reading in §19 before touching any of it — four
> of them turned on a collision or a silent misreport that is not obvious from
> the code, and all four are recorded there with the measurement.

**Exit:** a spell factors into named parts and still reads as one file.

**It was *"assembled from parts the player did not write that session, and the
parts are reusable"***, which assumed a shared library across files. **A spell is
contained to a single `.spell` file** (§19), so reuse is *within* a spell and the
cross-file half is struck rather than deferred.

§10: *"Composition — build spells from components"*. **Composition of spells, not
acquisition of words** — and that distinction is the whole of this phase's scope.
A component is a `part`, and it lives in the spell that uses it.

**Earning language words is a §19-priced deferral and stays deferred.** §19
records that its See-it line *"is already Phase 13a's"* and that delivering it
needs `Verb::ALL` to stop being a fixed 27, the synonym table to stop being
`const`, `is_live` to stop being a `const fn`, and the boot tutorial to read all
three dynamically — *"the discovery loop pulled forward two phases, not a
minigame."* It would also put discovery in the wrong room: §7, §10 and §11 all
put research in `archive/`, which *"powers all discovery"*.

So the grimoire composes **spells out of spell parts** — named, reusable pieces
one spell invokes — which needs no vocabulary change and is what the directory
already holds.

**Three boxes at the head of this phase are not Spellcraft**, and that is stated
rather than smuggled: the save format serves no part of the exit criterion above.
It goes here because Phase 3 is next and because **every phase from here adds
state to it** — Spellcraft's own spell-parts item adds the first. Building the
format after five domains have shipped means retrofitting six phases of state,
and it leaves Phase 15's settings item and Phase 13a's offline progression blocked
until then. §19 records the placement argument in full.

- [x] ✅ **The save document, headless.** `Sim::snapshot` reads a world out as
      plain TOML and `Sim::restored` puts one back, and neither touches a file —
      `orbs-sim` builds the document, a frontend writes it. Nodes are addressed
      by **path**, not `NodeId`; a restore **raises the tower first and then
      adopts it**, so a save written before a domain existed opens into a tower
      that has one. Closes Phase 1's stated debt — *"`Running` is not in any save
      format, because there is no save format."*
      - **The journal became the instrument.** `(seed, submissions)` had been
        recorded since Phase 0 with no consumer; `two_routes_to_one_world_write_
        the_same_save` reaches one world by replay and by play and requires the
        same bytes from both, which is what pins the snapshot as a *complete*
        description rather than a partial one
      - **`tower::drift` picked its target by query order** and a rebuilt world
        picked a different one. Its own comment had recorded the debt — *"only
        [`substitution`] got the fix"* — and nothing had ever rebuilt a world, so
        nothing could see it. Sorted by name, and the target drawn from the roll
        that already fired, exactly as `substitution` does
      - **A resumed spell verifies its program before walking it.** `pc` is a
        path into the tree the spell was cast against, so the save carries a
        fingerprint of the text it compiled and a mismatch ends the run in voice
        rather than resuming into the wrong program
      - **The record tail travels.** A `.log` is a *view* over the record stream,
        not a file with contents, so dropping the stream would empty every log in
        the tower — and §8.1's poisoned-log tell is built by re-emitting existing
        lines, so `verify` would have said *tampered* and shown nothing
      - **An independent review found five reproduced defects**, §19 lists them:
        a renamed pile changed slot on reload and silently changed what an
        ambiguous phrase resolved to; a record's register was written and never
        read back; `purge` on a shipped spell was undone by the next load in a
        release build; a hand-edited ward panicked on the next `probe`; and an
        open numbered prompt came back unanswerable. The fixture also **failed
        `cargo test --release`** and nobody had run it
      **See it:** `cargo test -p orbs-sim --test persistence`, then read one:
      `cargo test -p orbs-sim --test persistence -- --ignored show_a_save
      --nocapture` — the maze is drawn as a maze, a node's path stands where an
      id would have been, and an instrument mid-run says which tick it lands on
- [x] ✅ **The save file, and the tools that must not see it.** `orbs-shell`
      owns the path, the atomic write and the `[away]` wall-clock stamp — which
      is the frontend's because the sim has no clock and §19 forbids it acquiring
      one. `ORBS_SAVE` names a file and **`off` keeps none**.
      - **Written to one side and renamed.** A save lands every sixty ticks for
        as long as the game is open, so a crash catching a half-written file is
        not theoretical; the rename turns *"the tower is corrupt"* into *"the
        tower is one minute stale"*
      - **A dump neither loads nor saves unless asked**, which is the opposite of
        the running game's default. Otherwise every See-it line in CLAUDE.md
        becomes order-dependent on whether anyone has played in that directory,
        and `scripts/dumps.sh`'s baseline stops being one. Both it and the
        played-game suite pin `off` besides
      - **A save that will not open is kept, not deleted**, and says why in the
        log. A later build may read it; the next autosave overwrites it anyway
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
      outranks the seed *and* the environment — `session::Wizard` says a name is
      world state, so `USER` seeds only a new tower.
      - **Every exit path, not only `quit`.** `F10` writes an `AppExit` directly
        and so does the window's close button; the terminal's `Ctrl-C`/`Ctrl-D`
        return straight out of the loop. None touches `Quitting`, so ordering
        against `quit_requested` would have covered **one exit in four**. Bevy
        reads `AppExit` in `Last`; the terminal's loop became a function so its
        caller has one place every route converges on
      - **Autosave every 60 world ticks, in both builds.** Read off the sim's own
        clock, never a counter of the frontend's — one would drift the moment
        `meditate` ran three hundred ticks inside a `step`
      - **Three answers, not two.** *No save* and *a save that would not open*
        both end in a new tower and are not the same thing to tell a player, so
        `Opened` distinguishes them and the orb says which
      - **`SimPlugin::persist` is a field, not an environment variable.** This
        crate's own tests build a plugin and run it for hundreds of ticks, and
        under `cargo test` the working directory is the crate root — they would
        have read and written a save there. `ORBS_SAVE` is process-global and
        `cargo test` runs threads
      **See it** — the terminal build, twice, leaving by a different route each
      time, because a dump has no window and no clock:
      ```bash
      S=/tmp/tui.toml; rm -f "$S"
      tmux new-session -d -s orbs -x 120 -y 45 -e ORBS_SAVE=$S -e ORBS_BOOT=0 \
        target/debug/orbs-tui
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

**The language overhaul, and why it is in this phase.** *"Composition — build
spells from components"* is this phase's whole subject, and the language cannot
hold a component: six control words, no values, no names, and a step that reaches
the world by re-parsing an English line through the NLU every time it runs.
`threading` is the evidence — **six tiers × four ways, unrolled by hand**, because
nothing in the language can say *"the way with the fewest marks."* DESIGN.md §19
records the decisions, including that this **supersedes the seven-language audit's
no-variables premise**, that a step still costs a tick with the weave tree as the
escape valve, and that the lens's automation pin is deleted.

- [x] ✅ **`else if` — a chained `if`, and one `end` closes the ladder.** The
      shape every solver in the game is written in, and each rung used to nest one
      deeper and owe an `end` at the bottom: **49 of `threading`'s 98 lines were
      `end` or `else`**. Now 3 of 52.
      - ⚠ **`breaking` was the second example and is not any more.** It went
        21 → 15 here, and then `0.3.22` rebuilt the ward it solves: with
        `untried` gone there is no per-socket question left to chain on, so the
        shipped spell is four independent phases of three rungs and uses no
        `else if` at all. It is 53 lines and **`for each socket` is what
        collapses it** — left for the box that would do it rather than folded in
        here, because a solver rewritten to prove a language feature is a solver
        nobody measured
      - **A desugaring, not a new `Kind`.** `else if` pushes a chained `if` frame
        and one `end` unwinds the whole chain, so the runner, `step_past`,
        `guard_answers`, the save format and `interpret` need **nothing** — what
        they see is the nested tree that was always written by hand
      - **One reader for a condition.** The tail after `else` goes through the
        same reader a plain `if` uses, so there is one set of complaint keys and
        an unreadable rung runs neither half rather than being guessed at
      - **An unclosed chain complains once**, about the `if` at the top — a
        chained frame is half of a construct someone else opened
      - **The tree test flattens line numbers**, because the two spellings sit on
        different lines by construction and the line is load-bearing: §8.1 needs
        the log to name the player's line, not the desugared one
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
      thing it measures.** Five policies shipped and **not one invoked or bound a
      spell**, so `SCRIPT_BUDGET`, `PATIENCE`, the wait on the production slot and
      the re-cast a binding performs were all unmeasured — and §19's decision that
      a step still costs a tick is a claim about exactly that number.
      - **`bound` is `grind`'s loop again, run by a spell**, so everything about
        the two is equal except who is typing and the gap is purely what
        automation costs. Pointed at any other loop the number would be a mixture
        of that and the loop's own shape
      - **It keeps 0.910 of the hand-played rate, on every seed measured.** The
        absolute rates move with a world's luck at sabotage — 0.0814 on seed 3
        against 0.0910 on seed 0, and `grind` moves with them — while the
        quotient does not move at all, because it is a property of the script
        engine rather than of the tower. So `tests/agrees.rs` **pins the
        quotient** and the table reports the rate
      - **It goes through `Sim::write_spell`**, the editor's own public entry,
        not `debug_spell` — which is `cfg(debug_assertions)`, so a release sweep
        would have measured nothing, and hands back a shipped spell rather than
        one a policy chose
      - **A slot has to be earned.** No public API hands the sim experience, so
        the policy grinds for sixteen by hand exactly as a player does, then
        writes, then binds — three tick boundaries, because each waits on one
      - **A bound policy breaks the `cost` column's rule of thumb**, and that is
        recorded rather than fixed: a spell that *waits* stamps `Role::Cost` too,
        so `bound` carries ~640 healthy costs in a two-hour sweep. `--why` is
        what separates a wait from a refusal
      **See it:**
      ```bash
      cargo run -p orbs-balance -- run bound --ticks 7200 --why
      #   bound  0  7200  655  1  0.0910  656  641
      #   640  tending.spell waits: the mortar_and_pestle is working
      cargo run -p orbs-balance -- run grind --ticks 7200   # 0.1000, the same loop by hand
      ```
- [x] ✅ **A comparison can name another place instead of a number.** The other
      side of `has` was always a literal, so *"the way with the fewest marks"* —
      the sentence `threading` is six tiers unrolled by hand for — could not be
      written. `north has fewer marks than east`, `more … than`, `as many … as`.
      - **Additive, and every existing spell means exactly what it meant.**
        `Quantity::Count(1)` is what a bare `has sage` always was, so the ~110
        behavioural tests kept passing through the change rather than being
        rewritten around it
      - **It reads the published `Sense` children, not the components.** Both
        sides go through one `many_at`, which is the arithmetic
        `tower::build::raise_count` already promised — *"`has 2 or more marks` is
        answered by the same arithmetic that answers `has 4 fragment`"*. §8.1
        keeps its teeth: forging the event and forging the evidence stay one act
      - **Strict against a place, inclusive against a number**, which is English
        rather than an inconsistency: `has 2 or fewer marks` includes two and
        `has fewer marks than east` does not. *At least as many* is deliberately
        absent — `not … fewer … than` says it, the same route the docs give for
        `!=`
      - **A half-written comparative refuses the line.** `north has fewer marks`
        with no `than` used to fall through, hand `fewer` to the thing's name,
        and have fuzzy resolution drop it — silently becoming `north has marks`,
        which answers *yes* where the player's question answers *no*
      - **`more than 1 fragment` still reads as a number**, and it shares its
        first word. Nothing between the comparative and its closer is the whole
        discriminator; getting it wrong refused two rows of the spelling table
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
      **What it does *not* yet do is rewrite `threading`, and that was tried.**
      A walled way publishes no `marks`, so it reads as nought and is the minimum
      of any four — every rung of a true least-walked tier dies. Selecting the
      least-marked way *among the open ones* is a filter over a list, which is
      the next box. §19 records the attempt.
- [x] ✅ **Variables and sets — `let … be …` and `for each …`.** A spell can hold
      an answer and say *"each of these"*, which is what §10's *"composition"*
      needs and what `threading` was 52 unrolled lines for. `roaming` is the
      same maze in **19**, and a better algorithm inside them: a true minimum
      where the ladder had two buckets over the same count.
      - **A variable holds a *name*, and that is the stated ceiling.** Not a
        number, not a list, not an expression — everywhere a name may stand, the
        bound word stands for it. That is exactly what an accumulator needs and
        nothing more
      - **The cursor is named after the set**, so `for each way` binds `way` and
        the body reads `if way has spoil`. `it` was the obvious choice and is
        impossible: `it` is §6 filler, so `follow it` is stripped to `follow`
      - **`let … be`, because `set` is already a `dial` synonym** and a spell
        word is matched *before* the fuzzy matcher — `set second borax` at the
        prompt stopped reaching the lens. `spellword.rs` names three collisions
        it refused to add; this would have been a fourth, on a shipped verb.
        `be` rather than `to` for the same class of reason: `to` is filler
      - **A set is declared by the fixture, not derived from `Role::Reading`** —
        that marker covers the lens's four sockets *and* its six sigils, so a
        loop over it would hand a spell ten things when it asked for four
      - **`compile` had to learn what a spell binds.** A bound name reaches name
        resolution looking exactly like a place the room does not have, so
        without it every correct `for each` raised `spell_nowhere` — a
        `Role::Danger` record, which also latches the rail's fault mark
      - **`interpret` read `follow best` back as `follow west`**, the fuzzy
        matcher finding the nearest place in the room. The runner was always
        right; the one surface built to show a wrong resolution was the liar
      - **Fewer lines is not fewer ticks, and it was measured.** §8 charges a
        step per line, so three passes over four ways cost ~27 steps a move
        where the ladder short-circuits. A five-pass version — the same
        algorithm as `threading` — stopped finishing seed 3 inside 7200 ticks
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
      it, on a stack of descents: `pc` addresses one tree and a part is a
      different tree, so a call keeps the caller's path *and* its open blocks
      whole while the callee walks its own.
      - **`part`, and it was measured rather than picked.** Every candidate was
        scored against every word the game knows and the obvious ones are all
        inside the 600 typo band — `rite` **800** against `write`, `call`
        **750** against `wall`, `step` **750** against `stop`, `make` **750**
        against `take`, `form` **750** against `for`. `part` scores 500, and §10
        already calls the thing a part
      - **A call is punctuation, not a ninth control word.** `gathering()` — so
        the language spends no word on it, and a part may share a name with a
        verb without either meaning two things
      - **Two rules, both said rather than silent**: a definition belongs at the
        top level (`tree` looks no deeper, so a nested one would be unreachable),
        and one name means one part. Five complaint keys in all, each naming its
        line and what the orb did instead
      - **The budget is read, not compiled in.** `spell::budget` reads `Taken`;
        `steps_1` and `steps_2` are authored in `progression.toml` and **ship as
        markers** like every other node, so it answers 1 and the wiring is what
        was built. Additive across tiers, deliberately not a maximum
      - ~~**Variables are shared, not per-descent**~~ — **superseded at
        `0.4.2`**, and by removing its premise rather than by overruling it. The
        reason given was *"a part takes no arguments, so a private store leaves
        it with no way to be told anything at all"*; a part takes arguments now,
        so the store is per-descent and the parameters fill it. This box's
        original `(spell, pc, loops, vars)` shape is what shipped after all
      - ~~**Parts ship unused, by decision**~~ — **superseded at `0.4.2`**:
        `holding` uses one, and the arguments are why. The tick-per-line
        arithmetic that made factoring not worth it has not changed; what
        changed is that a call now *says what it hands over*, which took the
        sanctum's loop body from nine lines to three
      **See it** — a part told what to work on, which is the whole of what
      parameters bought. One body, two reagents, no `let` between them:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; scribe tending" \
      ORBS_EDIT="edit\npart load(what)\ngrind what\nempty mortar_and_pestle\nend\nload(sage)\nload(rock-salt)\n<esc>\nquit" \
      ORBS_THEN="invoke tending; meditate 40; peruse laboratory.log" cargo run -p orbs
      #   ground-sage, then ground-salt, from one run of lines
      ```
      ...and the scope, which is the rule to teach. The part binds its **own**
      `herb` and grinds rock-salt; the caller's is untouched, so the line after
      the call still grinds sage. Under the shared store this ground rock-salt
      twice:
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
      ...the refusals, each on its own line. **Six now, not five** — `gathering(sage)`
      used to be *"a part takes nothing between its brackets"* and is a count
      mismatch instead, and a heading naming one thing twice is its own case:
      ```bash
      ORBS_BOOT=0 ORBS_GRID=110x40 ORBS_DUMP="attend laboratory; scribe broken" \
      ORBS_EDIT="edit\nmissing()\npart gathering()\ngrind sage\nend\npart gathering()\nsurvey\nend\nrepeat 2\npart inner()\nsurvey\nend\nend\ngathering(sage)\npart twice(a, a)\nend\nhauling(a b)\n<esc>\nquit" \
      ORBS_THEN="invoke broken; meditate 3" cargo run -p orbs
      #   1 no such part / 5 named above / 9 outside every block /
      #   13 wrong number / 14 wants a name / 16 a name in each slot
      ```
      Line 15 is a **stray `end`**, and it is the refused heading above it
      working: a heading the orb cannot read opens no block, so the `end` under
      it belongs to nothing. Opening an unnamed part instead would swallow the
      body silently.
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
- [x] ✅ **A resolution policy** (`0.3.25`). `tower::reach` names the four axes
      that are questions about the *world* — scope, kind filter, naming, and
      **ordering, which is a game rule** — and every lookup is now one call that
      says which setting of each it wants.
      - **Three of the ten were byte-identical**: `navigate::find_script`,
        `bind::find` and `invoke::find` were the same walk written out three
        times. They are one call each now
      - **The fifth axis stays at the call site.** Whether a miss is a record, a
        `None` or an entry on a `missing` vec is a question about what the
        player should be told, and that is presentation
      - **`Scope::Fetch` moved out of `execute`** — §10.1's search order is a
        rule about the world, and it has to be *one* rule the moment a spell
        resolves a name at cast and a verb body looks it up at execution
      - **The defaults are §7.** A bare `look(world)` is *inside where you
        stand, any kind, by leaf*, so every widening past that is visible at the
        call site instead of buried in a helper's body
      **See it** — the two behavioural suites, untouched, as the equivalence
      proof; and the policy's own seven tests, which pin each axis:
      ```bash
      cargo test -p orbs-sim --test fetching --test arsenal
      cargo test -p orbs-sim --lib tower::reach
      ```
- [x] ✅ **Builtins that act — and the plan for them was falsified** (`0.3.26`).
      The box below asked for a typed call that constructs an `Intent`; what it
      did **not** say is which half of a line can be typed at cast, and the
      answer turns out to be *the verb only*.
      - **A spell makes its own inputs, so an argument cannot resolve at cast.**
        `digest ground-sage` is written above the line that produces any, so at
        cast the room has none, `analyse` drops the argument, and `interpret`
        reads the line back as bare **`digest`**. Freezing a whole `Intent` at
        cast would have broken every pipeline spell in the game, silently —
        which is the same shape of failure the box warns about, one level up
      - **The verb survives**, because a verb is offered by the fixture standing
        in the room rather than by what is on the shelf. `mix`, `distil` and
        `digest` all read back at cast with their arguments gone and their verb
        intact
      - **So `may_issue` is asked at cast as well as at the line.** It is a
        security boundary and it was answered too late: a `meditate 3600` in an
        untaken branch said **nothing at all**, and its own doc calls a scripted
        `meditate` a hazard — `Sim::step` drains `Skip` in a while-loop, so an
        hour of world time runs inside one step. As well, not instead: a
        boundary with one guard is one a future caster walks around
      - **The round-trip stays, and that is now a decision rather than a
        default** (§19). `run_line` still resolves names against the world as it
        is when the line runs, because that is what a pipeline *is*
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
- [x] ✅ **Syntax highlighting in the spell editor** (`0.3.29`). A spell was flat
      text; its parts now carry weight. `parser::lexeme` splits a line into runs
      — control word, verb, name, number, comment, call, filler — and `sheet`
      draws each in its own.
      - **§4 decided the medium, not taste.** *"Base hue carries all ordinary
        text through **intensity variation alone**, with a small accent set
        reserved strictly for meaning."* A spell is ordinary text, so colour was
        never available: the accent triad is reserved and widening the palette of
        a one-phosphor tube is a §19-sized decision. Three weights is what
        weight gives, and the split they buy is **structure / content / noise**
      - **The classification is a language fact**, so it lives in `orbs-sim`
        beside `read` and `analyse` rather than being re-derived by a painter —
        the shape §19 records going wrong three times over
      - **Lexical and world-free.** A spell keeps its shape read from another
        room, and does not change appearance as its own pipeline fills the shelf
      - **It declines on an accent and on the running line.** A line `interpret`
        could not read stays wholly red; the line the orb is executing stays
        uniformly bright. `Style::depicted`'s rule, twice
      - ~~⚠ **A verb and a name look alike**, because three weights cannot hold
        seven categories and `Control` takes the bright one. Telling them apart
        needs a hue, which is the thing §4 forbids~~ — **fixed at `0.3.35`**,
        which is where §4 was widened to allow exactly that hue
      - **It is not on `Style`, and `a_cell_stays_eight_bytes` is why.** A fifth
        byte cost +28 KiB and ~1.2 µs a frame on every screen for a fact only the
        editor reads; the arbitration lives at that one caller instead
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
      (`0.3.30`). `recall <word>` is a *command*, so reaching the manual meant
      leaving the editor, which is the one place you are when the language has
      outgrown you. A pane down the right of the editor now holds it: the
      vocabulary by default, and the page for the word under the caret when
      there is one. `guide` toggles it, and is the editor's fourth word.
      - **Two families of key, one pane.** A control word reads `recall_<w>` /
        `using_<w>`; a verb reads `man_<v>_gloss` / `man_<v>_use`. Both were
        already authored for `recall`, so the guide added prose for `part` and
        nothing else — rule 6 paying out
      - **The listing is the *spell's* domain, not the player's.**
        `execute::spell_vocabulary` filters by `spell::may_issue` and the scene
        of the domain the file belongs to, so a spell scribed in the archive
        offers `follow` and one in the laboratory does not. A guide that listed
        every verb in the tower would be the overwhelm it exists to prevent
      - **Held as state, refreshed on the keystroke [B2].** `guide()` reaches
        `scene_at`, which rebuilds every recipe, topic and node — ~197 µs.
        Computing it in the painter would run that at 60 Hz, which is the
        correction `offering.rs` and `editing.rs` have each already paid for. So
        `apply_to_editor` gained `&Sim` and the `Guide` lives on the `Editor`
      - **Wrapped, not clipped.** Prose is linted to 70 cells and the pane is 30
      - **`UtteranceKind::Guide`**, because there is no "on change" in the speech
        model — `Frame::reset` clears it every frame by design — so a reader
        needs its own kind to drop, exactly as `Hint` has
      - **It yields whole below 97 columns**, never narrow: 60 for the buffer,
        the gutter, and 30 for the guide. An earlier arithmetic let it show at
        80 and left 43 columns for `threading`'s 62-character lines
      - **Decision:** the toggle is session-scoped and does not persist. A save
        carries no editor state and adding some for a view preference is Phase
        11's settings item (§19)
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
      (`0.3.31`). The prompt's Tab listing, the prompt's ghost and the spell
      editor's Tab were about to be three answers to one question.
      `expect(line, caret, &Situation)` is that question; `complete` is now a
      thin view of it and `Completion` keeps only the prompt's shape.
      - **`Situation { scene, spell, prompt_open, open }`.** `spell` is not
        cosmetic: the language has nine words of its own the prompt cannot run,
        and `may_issue` + `is_live` rule out six verbs a spell may never issue
      - **`Expected { text, why, shape }`, and `kind()` is derived.** `why` is
        a [`Reason`] and `kind` a `Lexeme`; they were one field until the
        question grammar arrived, and `is`/`has`/`be`/`each` are scaffolding to
        a guide and ordinary words to the painter
      - **`SpellWord::shape` moved out of `orbs-shell`**, where it was a second
        table under a comment saying two answers to *what does `for` take* is
        one of them being wrong later
      - ⚠ **The scribing guide is deliberately not this.** It asks *what should
        I teach you* and lists canonical names only; `expect` offers all three of
        §6's registers, because Tab completing `l` to `ls` is the point
      **See it** — the refactor's own gate is that nothing changed, so the
      order test is the instrument:
      ```bash
      cargo test -p orbs-sim --lib parser::complete   # incl. the order pin
      cargo test -p orbs-sim --lib parser::expect
      ```
- [x] ✅ **The guide floats what is expected** (`0.3.32`). The reactive half:
      part way through a line the pane stops listing the vocabulary and says
      what may go where the caret is.
      - **The states belong to `is`, not to `wait`** — and the feature was
        specified the other way round. `wait` stores a *thing* and resolves it
        by scanning the record stream, so `wait for the mortar_and_pestle to be
        idle` waits on a thing called `mortar_and_pestle be idle`, matches
        nothing, burns `PATIENCE` and latches a fault on the rail. That the
        language confused its author while he was specifying the tool meant to
        stop exactly that is the argument for the tool
      - **Three of the nine words cannot open a line**, and the listing offered
        all nine: `until` is `repeat`'s guard, `end` needs something open,
        `else` needs an `if` **directly** above — which is why `open_blocks`
        keeps a stack and not a depth
      - **Touching a word is a page; past it is an expectation.** The space
        after a word used to still name it, which would have made `if ` explain
        `if` while `is ` listed the states, for no reason a player could infer
      - **Things before readings after `has`**, because a scene's things are the
        room's and its readings are not — `Errand::ALL` and the maze's senses
        chain onto every scene, so a laboratory spell was offered `passage` and
        `spoil` above its own reagents
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
- [x] ✅ **Tab completion in the spell editor** (`0.3.33`). The prompt has had
      it since Phase 1 and the editor — the surface where the vocabulary is
      hardest to remember — had `Key::Tab => None`.
      - **`orbs_shell::tabbing` is readline's rules, once.** Extend to the
        longest common prefix, list when that adds nothing, then cycle. A second
        implementation would have drifted: *"the first Tab lists **without**
        changing the line"* is bash's default and the opposite of what a fresh
        one reaches for, and the staleness guard exists because a forgotten
        cancellation otherwise splices a candidate into the middle of a word
      - **A decision, not a mutation.** `tab` hands back what should happen; the
        callers apply it, because a caret is a character count at the prompt and
        a row and a column in the editor
      - **The listing is the guide, which is why this step is small.** The
        prompt needs `Offered` and a reserved layout row; the editor's pane is
        already showing these candidates live as the line is typed. Building the
        guide first turned this step's design work into nothing
      - **Editing state only** — in command state the caret is on `edit`,
        `guide`, `interpret` or `quit`, four words listed on screen a row below
      - **`Expectation::common` is now the only shared-prefix code there is.**
        There were briefly two, and the prompt's *ghost* reads it: a second
        opinion would have the ghost promising text Tab did not take
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
- [x] ✅ **The prompt hues what is typed into it** (`0.3.34`). The spell editor
      got weight at `0.3.29`; the prompt — the surface a player types at
      constantly — stayed flat. Same `parser::lex`, same three weights.
      - **`orbs_shell::lexing` holds the arbitration now**, because two surfaces
        read it. `lit` is unchanged; `lit_at_prompt` is the new half
      - **Two lexemes are suppressed here.** `lex` is lexical, so it reads
        `repeat` as a control word and `gathering()` as a call wherever it finds
        them — and the prompt can run neither. Drawing them bright, the weight
        that says *the orb knows this word*, would make highlighting carry
        information, and the information would be false
      - **The whole line is lexed and a window is drawn** — `Line::window_starts`
        maps the runs. Classification is positional, so a window lexed on its own
        applies *line-start* rules to a fragment
      - ⚠ **The case where that reaches a cell is a scrolled comment**, and it is
        the only one: `Verb` and `Name` both weigh `Normal`, so the tidier
        example draws identically either way. The test is written against the
        comment for that reason
      - **One `Input` utterance, however many runs.** The span draws *and*
        speaks; the runs go on top with the silent `glyphs`. §14's stream is
        rebuilt every frame, so a run-per-word would recite a half-typed command
        one word at a time — `0.3.23`, at the worst possible surface
      - **The ghost is drawn after the runs**, not merely *known* to sit past
        them
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
- [x] ✅ **A spell is coloured, and §4 is widened to allow it** (`0.3.35`).
      Weight was the whole medium at `0.3.29` and its own ⚠ said what that cost:
      *"a verb and a name look alike, because three weights cannot hold seven
      categories."* Seven now, on two axes.
      - **§4 is superseded, deliberately** — see DESIGN §19. The sentence that
        gave ordinary text *"intensity variation alone"* now exempts spell text,
        and the accent triad is untouched: an accent still outranks a hue
      - **Two new lexemes.** `Grammar` (`is`, `has`, `be`, `each`, and the
        comparison spellings) and `State` (the eight spellings of the three).
        Grammar sits at **Normal** weight, which is the point — it read as
        `Filler` and is filler's *opposite*: filler is what §6 strips, grammar
        is what the question turns on
      - **`empty` is a verb and a state**, the only word that is both, so the
        state check sits below the verb check. Above it, `empty
        mortar_and_pestle` would draw its verb as a state
      - **A `Frame` side-table, not a fifth `Style` byte.** `Cell` is pinned at
        eight and a fifth field cost +28 KiB and ~1.2 µs a frame, measured. A
        syntax run is a *region* exactly as an instrument's bar is, so it rides
        `Vec<(Rect, Lexeme)>` — a handful of entries on the one frame with an
        editor open, none on any other screen
      - **Kept apart from `Tint`, which means materials.** The laboratory draws
        an instrument panel two columns from the editor; a verb sharing a colour
        name with a potion in the bar beside it would be one vocabulary meaning
        two things on one screen
      - **`monochrome` declines the palette entirely**, which makes hue a
        setting rather than something imposed — and is the §14 answer for anyone
        who reads colour poorly. Weight still carries the whole reading there
      - **Four of the eight are suppressed at the prompt**, which cannot run a
        spell: `repeat`, a call, and both halves of a question
      - ⚠ **The Bevy palette is one table shared by three themes.** The accents
        are tuned per theme because each sits against a different background;
        these were authored without a window to judge them in. `Phosphor::syntax`
        is per-theme so a pass with eyes on the tube can split them
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
- [x] ✅ **`RecordKind::ScriptLine` gets its first producer** (`0.3.23`). The §3
      exemption had been wired for four phases with **zero** producers, so every
      line of every spell announced itself as `row  tick: 11, message: follow
      north` — a table row that is not one, under a world clock that is not one,
      for a file the player wrote themselves.
      - **The producer is `peruse`/`sift` over a `.spell`**, and the split it
        needed already existed: `tower::Held` is exactly *text a person wrote*
        and its absence is exactly *a view over records*. Decided there rather
        than from the `.spell` suffix, which is a naming convention
      - **`FieldName::Line` — the number was never a tick.** `emit_lines` writes
        `index + 1`, a position in the listing; a log read back at world tick 5
        numbered its three lines 1, 2, 3 and called each a tick. Silent on
        screen, where no label is drawn, and read aloud to the one player who
        cannot check. **Both kinds** were wrong and both are fixed
      - **Prose keeps `Line` and nothing else.** A listing's gutter is how a
        player says *fix line 11*, so answering a §14 defect by dropping it would
        have deleted the affordance it was about
      - **The doubling guard had to learn the second kind.** A listing is written
        back into the stream it reads; `ScriptLine` left unfiltered means
        `peruse orb.log` swallows the last spell anyone opened
      - **The editor had the twin, through the painter** — found by following
        this item's own See-it line literally. `sheet.rs` drew every row as two
        `Painter::span`s and a span is one utterance, so a reader heard `1` …
        `repeat until the stacks is idle` … `2` … per line, for the whole file.
        The comment over the number had claimed the opposite for four phases.
        Announced once and drawn with `glyphs` now; **colour does not decide
        where a sentence ends**
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
      #   line: 1 … line: 3, at world tick 5 — and the boot card's `tick: 0` stands
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
      here from Phase 1 — §18 item 2, listed **blocking**, and it went *ahead* of
      the domains that coin the most verbs. Thirty verbs, all three registers,
      typed at a real prompt and read back.
      - **The lens had no plain-English way in at all.** `syn(Verb::Probe,
        Register::Plain, &["spy", "peek", "try"])` — `words` is *one phrase,
        pre-split*, so that declared the phrase `spy peek try` and nothing else.
        `spy` at the prompt echoed `! spy` and reached nothing, and so did the
        other two. Split into three, swept first: `spy` and `try` collide with
        nothing, `peek` scores 500 against `pewter` — under the 600 band
      - **Two tests watched it happen**, which is the more useful half.
        `every_verb_is_reachable_from_plain_english` asks whether a `Plain` entry
        *exists*, and one did; `every_phrase_reaches_the_verb_that_claims_it`
        drives the declared phrase, and `spy peek try` reaches `probe` perfectly
        well. Both asked about the shape rather than about what a person types.
        `multi_word_plain_synonyms_are_pinned` is the guard
      - **§6.1's own table had drifted, and one row contradicted another.**
        `meditate` appeared **twice** — one row saying *"not `wait`"*, one listing
        `wait` — while the code has neither. `wield` claimed `kindle`, `stop`
        claimed `damp` against the code's `quench`, `decoct` still listed `mix`
        and `distil`, and four rows had missed a word each. Six rows wrong in the
        document DESIGN.md calls authoritative
      - **The table is now a lint.**
        `the_slice_table_in_this_document_matches_the_vocabulary` reads DESIGN.md
        and compares canonical names and shell words against `SYNONYMS`. §19
        records this exact drift three times over — the rail's state words, the
        substitution table, `is_live` — each found by a person reading two things
        side by side, which is what this does now
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
      (§19). Its See-it line was *"two spells share a part, and editing the part
      changes both"*, and **a spell is contained to a single `.spell` file**.
      Composition is within a file, it shipped at `0.3.24`, and the cross-file
      half is not deferred — it is not wanted.
      **See it:** `part gathering()` and `gathering()`, in the `0.3.24` box above
- [ ] Hidden-directory authoring plan (~80 fragments' worth), moved here from
      Phase 1
      **See it:** find one without being told it is there
- [x] ✅ **~~What a part costs to hold~~ — struck with the item above** (§19).
      It put a composed spell's cost on the sidebar because parts were to be held
      *separately* and compete for concentration. An in-file part is not held
      separately — the spell is one file either way — so factoring out costs
      nothing and there is no price to draw. Concentration still prices **spells**,
      which is where §11.5 put the scarcity to begin with.

**Scarcity: concentration.** §11.5's shared pool, one slot at 16 experience. A
composed spell that holds parts competes with everything else the orb holds, so
*what to factor out* is a choice with a price.

---

## Phase 4 — Defense ✅

**Exit:** wards placed against a pressure the player survives by *choosing*, and
then a spell that survives it. **Met** — a course of wards hauled between three
stations by hand, and `holding` bound in the sanctum while the player brews.

> **This was Phase 9, and Enchanting was Phase 4.** They swapped (§19). Nothing
> about the order was load-bearing — Defense depends on the spell language,
> which closed in Phase 3, and on nothing Enchanting or Summoning make — and the
> version is `0.<phase>.<step>` and player-visible on the POST card, so building
> a later phase first would have made the number go backwards when the earlier
> one landed. Phase 12's authored edge *"menagerie → sanctum"* survives the swap
> unchanged.

> **The room is `sanctum/`, and §7's tree and §10's table both said
> `battlements/`.** Both are superseded (§19). The domain shipped at `0.4.0` with
> fortification names — a `rampart`, a `barbican`, a `bastion`, a `redoubt` — and
> the sentence a player typed most was *"haul a ward from the barbican to the
> redoubt"*, which is not a thing a wizard does. The mechanics did not move; the
> fiction did.

§10: *"Command pressure at 1 Hz, ward placement."* **The form in that table is
not what shipped, and the table says it should not be** — it is *"a table, not a
design"*, and this domain is the one §10 singles out as still able to fail
§10.1's rule.

- [x] ✅ **The reflex-avoidance mechanism, decided and recorded in §19 *before*
      anything was built.** It is the **Tower of Hanoi**, and it dissolves the
      problem rather than working around it: there is no clock in the puzzle at
      all, every ward is on screen, and the only thing that can go wrong is
      choosing the wrong pair of stations. *Outcome follows what the player
      chooses given readable state* is then true by construction rather than by
      restraint.
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
- [ ] ⚠ **The accessible mode — moved to Phase 8, not struck.** §14 requires a
      *real-time* surface to have one, and this domain has none: a course waits
      for ever. DESIGN.md fixes the shape of the answer as *"screen-reader mode
      advances **siege** ticks on player input"*, which is Phase 8's surface and
      not this one. Recorded in §19 rather than quietly dropped.

**Scarcity: integrity, which decays.** §10's row says *"wards, made and
consumed"*; §19 argues the re-reading, against the precedent of scrying's
withdrawn *"a read is not a brew"*. The wards themselves are not stock — they are
the puzzle's pieces — and what the domain actually mints and loses is the barrier.

**Neither verb takes the production slot**, which is the lens's decision and not
the archive's, so a bound `holding` runs *beside* a brew rather than instead of
one. `orbs-balance`'s `warding` policy reads **0.1249 on every seed** — the
flattest column in the table, and just under clarity's 0.140.

---

## Phase 9 — Enchanting

**Exit:** a buffed instrument visibly works faster, the panel says so, and the
buff decays.

**Derived** in §10 — cut from brewing's pipeline, so this is `Recipe`, `Working`
and the instrument panel rather than new subsystems. The cheapest of the five,
and the proof that the generalisation §10.1 promises actually holds.

> **A minigame was missing from the first plan, and a reader caught it.**
> `imbue` was one command with a duration and a resource cost, which is exactly
> what §10's Minigame-form column exists to prevent — the column says
> *"**Sequence** + resource cost"* and the sequence was not there. It is a
> **lattice** now: Lights Out on three columns, where snapping a glyph flips its
> neighbours and a charm binds only when every glyph is lit. §19 records the
> shape and what the sweep cost.

> **The version did not become `0.6.x`, and it cannot.** The scheme is
> `0.<phase>.<step>` and the number is drawn on the POST card, so a tester's
> build would have gone *backwards* from `0.8.18` — which is the exact hazard
> §19 cites for swapping Defense and Enchanting in the first place, arriving
> from the other direction now that Phase 8 shipped first. This landed as
> `0.8.19`, and **what the number should say when a phase closes out of order is
> a decision nobody has taken.** Named here rather than settled quietly.

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

> ~~**A second potion sink, and it amends §11.5.**~~ **Struck.** The plan was for
> the forge to spend essences; it spends **quintessence**, shared with the siege,
> which is a decision taken deliberately and recorded in §19. So potions get no
> second sink and §11.5's table is unamended — what changed instead is that
> quintessence came *up* to the tower, which is a bigger amendment to the same
> section and the one §11.5's own resource row always described.
>
> **The count in that paragraph was wrong too**, and the fix landed anyway:
> **nine** prose lines said *"a siege will be what spends them"*, not five — and
> every one had been false since Phase 8 shipped, because `siege.toml` gives six
> of the eight potions a `quaff` entry and a troop a `deploy`. Those nine now say
> what actually spends them. Two — `stillness` and `vigour` — have no sink at all
> and say so.

---

## Phase 5 — Summoning ✅

**Exit:** a chant performed by hand or by a spell yields troops, and a botched
one leaves the tower worse. **Met** at `0.5.6`: a figure is summoned, sung on the
arrows or by a spell, yields troops into the arsenal, and wears the barrier when
it collapses.

> **The exit and the boxes were rewritten at `0.5.0`, and DESIGN.md §19 says
> why.** They read *"a summoned thing acts on its own, and the player did not
> tell it what to do that tick"*, with a **standing rule** box and an automation
> currency to decide. **None of that is built here.** The chant is the activity
> and troops are the product; they are inert until a siege spends them, so
> autonomy moves to **Phase 8** with the thing that gives it something to do.
> §10's *"allocation"* scarcity row and its **Derived** classification are both
> withdrawn — a chant spends nothing, and a real-time surface is not cut from
> brewing's pipeline.

**Bespoke.** The one domain solved by *doing* rather than by choosing, which is a
reversal of §19's `0.4.0` and is argued there rather than assumed.

- [x] ✅ **Head of phase: the §10 reversal, and the keyboard's one owner**
      (`0.5.0`). §10.1's *"never a reflex"* struck with a pointer, the exit and
      the scarcity rewritten above, and the `Focus` refactor `shell/input.rs` had
      been pre-committed to since `wander` — *"a fifth surface refactors this
      first."* One enum in `orbs-shell`, ported from `orbs-tui`, read by both
      frontends. **The obvious gate was vacuous** — a dump builds no `App`, so
      `dumps.sh` never executes the file — and §19 records that as the more
      useful half
      **See it:** `scripts/play.sh` (113 scenarios, a live keyboard), and
      `cargo test -p orbs --bins no_surface_lets_a_keystroke_reach_the_prompt`,
      which opens each of the four surfaces in a real `App` and proves the prompt
      stays empty
- [x] ✅ **The `menagerie/` domain, the chart, and `sing` typed** (`0.5.1`). A
      figure of twelve syllables, one landing per tick; `summon` draws one and
      `sing <syllable>` answers the aperture. Four misses collapse it and the
      barrier pays five, which is the domain's only cost and the reason an
      attempt is free. Neither verb takes the production slot — a `sing` that
      queued behind a brew would arrive *after* the syllable it answered, which
      is a guaranteed miss rather than a wait. **`up`/`down`/`left`/`right` were
      swept out**: `left` scores 750 against the spell language's `let` and
      `right` 800 against `light`
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; sing skyward;
      sing skyward; sing skyward; sing skyward" cargo run -p orbs` — four blind
      misses, and the figure comes apart. **Singing well is not possible yet and
      that is not a defect**: nothing draws the chart, so a hand player is blind
      until the next box. `cargo test -p orbs-sim --test chanting` is the loop
      proved through the real verbs
- [x] ✅ **The board, and the pace that makes the room playable** (`0.5.2`).
      Syllables rise to a rule at the top, eight ahead, lanes named so a player
      can type what they see. **`PACE` was one and is four** — a syllable a tick
      left no room to read, so a spell could not strike one syllable of a figure
      and a person could not either. Singing early is not a strike, which is what
      makes a delay worth computing. `bide <n>` is the language's tenth word;
      `rest` was the first name and is a `meditate` synonym (§19)
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; meditate 2"`
      against the same line without the `meditate` — the figure has risen two
      rows, which is the timing being visible. It fits the 80×22 floor.
      **A `sing` on the tick after `summon` answers *"too soon"*, not a strike**:
      the syllable is three ticks out, and a dump has no clock to wait with. This
      line claimed a strike and said six for the pace, both wrong — a See-it line
      that describes a different game is worse than none
- [x] ✅ **A spell that times its own answer** (`0.5.3`). A two-tick landing
      window and a `bide n` that costs exactly n ticks. **The solver strikes
      twelve of twelve at two instructions a tick and collapses at one**, which
      makes the menagerie the domain that rewards concentration: unautomatable
      until the weave grants a second step. That reverses `0.5.1`'s lookahead
      bound deliberately (§19)
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; peruse
      chanting.spell" cargo run -p orbs` — and note it is *meant* to fail at the
      shipped budget. `cargo test -p orbs-sim --lib tower::chant` holds the pace
      and window arithmetic
      > **`bide until` shipped in this box and was withdrawn at `0.5.8`**, along
      > with the `until` reading it read. A spell that reads its delay off the
      > world computes nothing. The claim above survives the removal and is
      > *asserted* now rather than measured once — see the box below
- [x] ✅ **`steps_1` is takeable — the first real node in the tree** (`0.5.4`).
      The screen returns an id, the shell hands it to `Sim::take`, and it lands
      on the next tick as a recorded `Submission::Took`. **Both sides check every
      rule**: the screen refuses a marker, a locked node and a spent tier, and
      the world re-asks all three rather than trusting a screen's arithmetic.
      The menagerie is what gave it a reason to exist
      **See it:** three distillations to 24, then
      `ORBS_WEAVE="mastery\ntake\nquit"` — *"the orb takes steps_1. it thinks a
      little faster now"*. Then `invoke chanting`: **24 of 24 struck** where at
      one step a tick it collapsed. `cargo test -p orbs-sim --test progression`
      holds the take, the refusals and the replay
- [x] ✅ **The interactive surface, and the accommodation** (`0.5.5`). `chorus`
      hands the arrows to a running figure — the fifth surface, and the one the
      `Focus` refactor was built for. **It does not take the pane**: a figure
      draws beside the transcript, so a player answering syllables still reads
      what the orb says about them. `F9` makes a chant *patient* — the syllable
      waits for the singer rather than the clock — and it reaches the **same
      ceiling**, which is what makes it an accommodation and not a difficulty
      setting (§14). Both keys are bound in both frontends
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; chorus"
      ORBS_CHANT="<up>\n<left>" cargo run -p orbs` — the presses land and read
      *too soon*, because a dump has no clock. Add `ORBS_PATIENT=1` and the same
      presses land cleanly, which is the whole of what the mode does.
      `cargo test -p orbs-sim --test chanting` pins the parity, with the played
      chant as its control
- [x] ✅ **The troop is a material the game knows about** (`0.5.6`). A page, a
      tint, and a name `debug_spawn` can make. **None of `tower::home`'s three
      lints could see it**, and that is the finding: they walk *authored*
      materials, and a material no file declares is a material no lint iterates
      — so a thing the game produced every time somebody sang was unreachable to
      a tester and answered `recall` with the bare overview. `substances` names
      it beside the fuels, which is the same seam and the same reason
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="recall troop; debug_spawn troop 3;
      survey arsenal" cargo run -p orbs`. And check the verdant trap while you
      are there: four `wield verdant-scroll` must not shelve troops on the
      dispensary as an inexhaustible herb

- [x] ✅ **Reviewed, and four blockers fixed** (`0.5.7`). Fifteen findings on a
      phase that shipped green — and **three of the four blockers were in code
      whose own comment cited the rule it was breaking**: `wear_by` republished
      through the `Cwd`-scoped lookup it names §19 for avoiding, `bide until`
      skipped the room swap a helper exists to make, and the board that calls
      itself *"the one picture that moves"* drew four identical frames per
      approach. Key repeat ate the figure, `ChantSave` existed only in a comment,
      and the domain had **none** of the four See-it instruments wired
      **See it:** `cargo run -p orbs-render --example screens` has a figure now;
      `scripts/dumps.sh` captures seven menagerie screens; `scripts/play.sh
      menagerie::` plays five scenarios on a real keyboard; and
      `cargo run -p orbs-balance -- run chanting --ticks 7200` reads **0.243**.
      The bound solver yields **20 troops from another room**, where before the
      fix it yielded none

- [x] ✅ **`bide until` withdrawn — the delay is the player's arithmetic again**
      (`0.5.8`). The shipped solver read its delay off the circle, so it computed
      nothing: the blocking wait this phase rejected by name, rebuilt under
      another name inside the domain built to refuse it. **The `until` reading
      goes too** — with it answerable, `repeat until the circle has 1 until` is
      the same cheat as a one-tick spin. `Delay` is gone as a type, so
      `Kind::Bide` holds a literal and `bide sage` is a complaint rather than
      four billion ticks of silence.
      The solver is a **constant-length** `for each` that binds with `let` and
      sings *outside* the loop — measured at two steps a tick, the old ladder
      strikes **0 of 12**, singing inside the loop strikes **6**, and this
      strikes **12** with no `bide` at all. Two things it exposed are worse than
      the defect: **nothing anywhere tested `bide`**, so the shipped `.spell`
      stopped compiling with the gate green; and `RunningSave` dropped `biding`,
      so a save four ticks into `bide 3600` reloaded into another whole hour.
      `FORMAT` 4 → 5, because a stored spell that still *loads* and no longer
      *reads* is the format-2 failure in a different costume (§19)
      **See it:** `cargo test -p orbs-sim --test chanting the_shipped_solver` —
      the hook as an assertion rather than a claim, collapse at one step and
      close at two, which is deliberately the pair. `cargo test -p orbs-sim --lib
      tower::spell::program::tests::bide_takes_a_count` holds the refusal, and
      `cargo run -p orbs-balance -- run chanting --ticks 7200` still reads
      **0.243** — the harness reads `until` off the model, where the language
      cannot

- [x] ✅ **A satchel, and one spell can hand another a name** (`0.5.9`). §8's
      spells could not tell each other anything — though **two of them already
      ran at once**, ungated: `invoke` from inside a spell inserts a second
      `Running` and the caller does not block. What was missing was the channel.
      A `satchel` in every domain but the arsenal, `queue <name>` as a verb and
      `pull <name> from satchel` as a control word — asymmetric because *pulling
      binds a name*, and a verb cannot. It **yields while empty and never reaches
      `PATIENCE`**: a consumer caught up with its producer is a working pipeline,
      not a fault worth latching `‼` for.
      A **component, not children** — a queue is an ordered multiset and `Stock`
      collapses duplicates — so `spell::watch` gained an arm, without which `if
      the satchel is empty` is true of a full one. And `scene_at` offers only the
      local satchel, which is what makes `build`'s leaf-uniqueness exemption true
      rather than argued: all six shared a leaf, so `queue` filled the
      menagerie's and `survey satchel` read the **laboratory's**, one line apart.
      **`onward`** is one syllable of lookahead, a widening of §19's earlier
      bound. **The menagerie is not this feature's use case and that is
      measured** — identifying one of four lanes costs more than `PACE`, so a
      producer queues 4 of 12 at one step a tick and 6 at two, and no depth fixes
      it (§19)
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend menagerie; debug_take satchel_1;
      queue skyward; queue earthward; queue skyward; survey satchel" cargo run -p
      orbs` — a name twice, in order. `cargo test -p orbs-sim --test satchel`
      holds ten claims, one per way this could have been half-built

- [x] ✅ **`alongside` — two cursors in one spell, and the unlocks** (`0.5.10`).
      The per-position half of `Running` became a `Strand` and the spell holds a
      `Vec` of them. **`Progress::Blocked` had to stop ending the entity's
      tick**: a consumer on an empty `pull` would otherwise end the tick before
      the producer was reached, every tick — the deadlock was by construction.
      `seen` moved to the strand (two cursors sharing a record-stream mark makes
      one satisfy the other's `wait`), strands step **batch in `Vec` order** to
      match `advance`'s law, a finished one is `remove`d rather than
      `swap_remove`d, and `MAX_STRANDS` is 4.
      Done as a **swap** rather than an index at ~110 sites — `Cwd`'s idiom one
      level down — so the runner, `capture`, `adopt` and `invoke` are untouched
      and the whole balance table is unchanged to four decimal places, which is
      the refactor's real proof.
      `satchel_1` at 24 and `cursors_1` at 40, one grant of speed and one of the
      channel per tier; `steps_granted` became `granted() -> Option<Grant>`, and
      `Progression::check` refuses an id that *reads* as a grant and does not
      parse as one. **`cursors_1` sells ergonomics and says so** — two spells were
      always free; what this buys is both halves in one file
      **See it:** `scripts/play.sh satchel::` plays five scenarios on a real
      keyboard, including the fork feeding itself. `cargo test -p orbs-sim --test
      strands` is nine, and `two_cursors_interleave_the_same_way_at_two_steps_a_tick`
      is deliberately written at budget 2 — batch and round-robin are identical
      at one step a tick, so a pin written there pins nothing.
      `debug_take <id>` is why these lines are short: `debug_spawn`'s argument,
      applied to two hundred ticks of laboratory

- [x] ✅ **Worked examples of both forms, and a rail that can count** (`0.5.11`).
      Three dev spells: `ordering` + `milling` are the **two-spell** channel, and
      `coursing` is the **two-cursor** one — `holding` split down the middle,
      solving in `2^n − 1` hauls, which is the same optimum the unsplit solver
      reaches and is what the test asserts. Both are *cast* in tests, which is
      `chanting`'s lesson: a dev spell nobody casts is prose.
      **The rail named one spell per room however many ran there**, and a fork
      was invisible for the same reason one level down. The suffix counts
      *cursors* — from the rail a second `invoke` and an `alongside` are one
      fact — and the name truncates where the count never does, because the rooms
      that earn a `+2` have the longest names in them. **`status` gained a
      `casting` section**, without which the count pointed at nothing; both read
      one walk so they cannot disagree (§19)
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend sanctum; debug_take satchel_1;
      debug_take cursors_1" ORBS_THEN="invoke coursing; meditate 6; status" cargo
      run -p orbs` — the rail reads `►coursing +1` and `status` says
      `coursing  sanctum, on 2 cursors`. `cargo test -p orbs-shell --lib rail`
      holds the truncation, and
      `the_rail_counts_every_cursor_and_status_names_them` holds the pair

- [x] ✅ **`recall apprentice` — the lesson, where `recall scripting` is the
      reference** (`0.5.12`). **Nothing taught a player to make a spell.** That
      page lists the words and teaches nobody how to start, because no listing of
      words teaches an *order* — `scribe`, `edit`, the lines, `<escape>`, `quit`,
      `invoke`, then the log rather than the pane, one of which (*quit is the
      save*) is otherwise learned by losing work.
      §12's in-world grimoire is an **always** item and Phase 14 owns the
      interactive apprenticeship, so this is the reference half and treads on
      neither. The worked example is **built from the room** — `grind sage` in
      the laboratory, `probe` in the lens — and a room with no work to script
      borrows the laboratory's and says whose they are.
      **The way in is `help`**, and that is the load-bearing half: everything on
      the overview is a word to type now, so a player could read it in every room
      and never learn the game has spells in it (§19)
      **See it:** `for room in laboratory archive lens sanctum menagerie
      grimoire; do ORBS_BOOT=0 ORBS_DUMP="attend $room; recall apprentice" cargo
      run -q -p orbs; done` — six rooms, six examples.
      `scripts/play.sh the_apprentice` **follows the page** from `help` to a
      spell that earns, on a real keyboard, which is the gate that matters: a
      tutorial's lines are typed rather than read past.
      `the_apprentice_only_shows_lines_the_room_can_run` asks both questions —
      resolves *and* offered — because `grind sage` in the lens resolves and is
      then refused

**Scarcity: integrity.** A chant costs nothing to attempt; enough mistakes fail
it and wear the pylon. The cost is time, never progress (§11.5) — and it makes
the sanctum the second domain that writes integrity, which is why a sweep is part
of the gate rather than an afterthought.

---

## Phase 10 — Progression

**Exit:** a fresh tower is a laboratory and nothing else, and the rest opens as
the work is done — rooms, recipes and charms along seven mastery lines and one
Ley Line with a choice at each fork — and the line runs to the soft ending.

**The two tracks swapped natures here** (§19). **The Ley Line is the tower's**:
one line on total experience, whose stations are *steps* (concentration,
quintessence — passing is the grant) or *forks* (take one of three lanes —
provision, war, craft — and the one you take shapes how the game plays).
**Mastery is per domain**: seven straight lines, no choices, each station a
deed — brew a clarity, walk the stacks, close a figure — reached in order. Both
may `opens` something, which is how the tower grows: brewing a clarity opens the
archive. Every number is a first pass and `orbs-balance` decides it.

- [x] **Deeds, tallies, and the two tracks re-strung** (`0.10.1`). The plan's
      first two boxes landed as one, because `[[mastery]]` in
      `progression.toml` cannot be both the old tiers and the new lines at once.
      - **`[[ley_line]]` stations carry `grants` (a step) or `nodes` (a fork),
        and optionally `opens`.** The two tiers that were `[[mastery]]` are forks
        at 24 and 40 now; `take` acts on the ley track only. The four real nodes
        keep the totals they had — the lane rule (one node per lane, refused at
        load) lands with the first grant that is not craft, since until then
        every fork is craft against craft.
      - **`[[mastery]]` is seven lines of `domain`, `id`, `done`, `opens`.** A
        deed is `"clarity"`, `{ potions = 5 }`, `{ scrolls = 1 }`,
        `{ at = "stacks", times = 3 }` or `{ event = "figure" }`; every name is
        checked at load against the recipes, the instruments and a closed event
        set. A line is walked in order, a late deed reaches several stations at
        once, and each is said. **Thirty stations ship**, and `haste` is gated
        behind the laboratory's fifth deliberately (§19).
      - **`tower::done` is the one door every completion goes through**: it
        counts the work, credits what it earned, and advances every line — ten
        seams, listed in `tower::tally`. The menagerie counts a figure closed,
        the bailey a siege settled and one held, the grimoire a spell bound, the
        lens a secret found.
      - **`tower::Opened` holds what has been opened** — `domain:`, `recipe:`,
        `charm:`, `siege` — and `Sim::new` opens everything, which is the tower
        every test, dump and policy has always used. `recipes.toml` gains
        `gated`, **per output**: the lectern fires once any of its three scrolls
        is known, and a recipe-level gate would have sealed the gleaning scroll
        behind the station that opens the others.
      - **The weave draws one track at a time below its bar.** `ley` shows the
        forks' siblings stacked under their stations; `mastery` shows seven
        rows, one per room; down means a sibling on one and a room on the other,
        and the status row says which. The bar is measured against the line's
        last station — 56 — rather than a fixed hundred. `take` on a mastery
        station is refused in voice. `debug_reach <id>` reaches a station and
        every earlier one on its line.
      - **Save `FORMAT` 10**: `tally`, `reached` and `opened`, each defaulting to
        the honest reading of a document that never had it — nothing counted,
        nothing reached, everything open.
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
- [x] **Sealed rooms** (`0.10.2`). A fresh game is a laboratory and nothing
      else, and the rest is earned.
      - **`Sim::sealed` is the tower a fresh game builds; `Sim::new` stays
        open.** `Opened::start` is derived from the content — everything no
        station opens is open, so the laboratory and `hurried` are and the
        archive is not — rather than authored a second time. The bailey follows
        the wall. `orbs_shell::fresh` is the one reader of `ORBS_SEALED`: the
        game defaults to sealed, the dump to open, and the scenario suite and
        `scripts/tui.sh` set it to `0` so ~200 See-it lines keep meaning what
        they meant (§19)
      - **One gate, asked of the node.** `tower::sealed_room_of` refuses
        `attend`, `survey <place>` and a path into a shut room in one voice —
        *"the archive is not yours yet"* — and the tower's listing, the boot
        report and the rail's box all read from the same set. `tower::Sealed`
        is the same fact as a marker on every node under a shut room, for the
        two sabotage queries that cannot ask a function: nothing strikes a room
        the player cannot enter
      - **A shut room stays a place the parser knows**, deliberately. Dropped
        from the scene, `attend archive` fuzzed into a numbered prompt offering
        four *other* rooms — §15's dead end, reached by the one word a new
        player will try. What stays hidden is the room's manual page
      - **`[world] sealed` travels with the seed**, because a sealed and an
        open tower with identical submissions diverge at the first
        `attend archive`; `tests/sealed.rs` replays one. A save that never had
        `opened` restores open, and a sealed one stays sealed
      - **The suite plays it.** `Game::sealed` is the scenario runner's one
        door to the start a player gets, and `sealed::` earns its way into the
        archive at a tester's pace
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
      rising. **This line was impossible until `debug_reach` learned to open the
      room it reaches into** (the forge is behind the ley step at 56, which is
      seven distillations); `cargo test -p orbs-sim --test sealed
      the_wall_is_armed_by_the_sanctum_and_a_charm_by_its_station` is the same
      thing held by a test
- [x] **The rail's percentage and the domain's road** (`0.10.4`).
      - **`Brief.mastery`** is the rail's one progression reading: the
        percentage of the room's next deed, absent on a finished line and on a
        shut room. It sits on the **state row's right edge**, because
        `MIN_RAIL_BOX` is name, state, detail, spell and the rule, and a sixth
        row would be dropped exactly when a room is busy *and* automated —
        the state the rail exists to show. Drawn silent: §14's rule for
        progress is *completion only*, and the station reaching is what is said
      - **The road** is the room's own line under the pane's title —
        `laboratory [•]─[○]─[·]─…  five potions brewed · 3 of 5` — one row
        taken off the top of the body *before* the panel and the boards divide
        the rest, so it sits under the title whichever way the panel runs and
        a short pane loses the road rather than the transcript. `Panel.line` is
        the room's line on the tick clock, and `Sim::domain` is the room rather
        than the leaf, so a player standing in the alembic sees the laboratory's
      - **One vocabulary.** `stations.rs` is the glyphs, the frames and the run
        the loom drew, extracted so the road and the weave cannot come to draw
        a reached station two ways. The dump builds its `Panel` through
        `refresh` now rather than a second literal, which is how a tenth
        reading would otherwise have been forgotten
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
- [x] **The line to the soft ending** (`0.10.6`). Sixteen stations from 16 to
      10,000 — nine steps to concentration 8 and seven forks of three lanes —
      and every grant a fork offers is built. **One box rather than ten**,
      because each grant is one `Grant` arm and one site, and the lane rule,
      the node moves and the sweep are one change: the plan's ten boxes would
      have been ten sweeps of a harness that takes no node.
      - **`tower::grant`** is the one parser, the lane, and a reader per
        grant, each summing the tiers taken. Provision: `fuel` (a charcoal
        burns a fifth longer per tier, on what is lit and never on what was
        banked), `pool` (four on the ceiling), `escrow` (a siege pays a quarter
        more, on the pool before either branch so a loss pays more too),
        `thrift` (two off a charm, never below one, at both the price quoted
        and the price taken). War: `edge` (one on every answering roll, named
        on the siege so the board and the log read one number), `floor` (ten
        points on the pool's floor under a worn wall), `garrison` (a troop
        brings one more body), `mend` (a course puts three more back),
        `vigilance` (the calm layer's interval a third longer — a quarter fewer
        strikes — with the draw untouched, so a tower without it keeps every
        replay). Craft: `haste` (a run a *spell* issued lands a tenth sooner
        and a player's own does not — struck invariant 3, bought), and
        `steps_3`
      - **The lane rule is enforced now**: a fork with two nodes in one lane
        fails the load. The four real nodes moved to their lanes' stations —
        `satchel_1` to 40, `steps_2` to 160, `cursors_1` to 400 — and **a node
        held above its fork's total is kept**, not dropped: `ley_line` derives
        *spent* by membership, so the fork reads chosen and nothing the player
        earned is taken away. That is also what lets a tester's `debug_take`
        survive the save round-trip `tests/strands.rs` holds it to (§19)
      - **The line draws on a logarithmic scale.** Sixteen stations growing by
        half each stood in a knot at the left of a linear run to ten thousand;
        equal cells for equal ratios spreads them as the curve does, the bar
        fills to exactly the cell the total has reached, and the totals
        alternate between two rows so five-figure labels do not run together
      - **`status` says the pool and its ceiling**, because nothing else did
        and a grant a person cannot see is a grant that does not exist
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

**✅ Folded in after three reviews of the whole phase.** Thirteen defects, none
of which advances a box; each is in §19 with what it cost.

- **A Ley step's `opens` was never applied**, so the grimoire and the forge —
  the only two rooms hanging off the tower's line rather than a room's — were
  shut for ever in the only tower a player ever gets, and the forge line's
  `fruitful` with them. Every test there was drove the mastery track. `ley::cross`
  runs from `credit` now, `opened::opening` is the one loop both tracks announce
  through, and a restore catches the line up silently so a document written
  before this cannot stay stuck.
  **See it:** two clarities in a sealed tower — *"the grimoire is yours now"*,
  then `attend grimoire` answers; `scripts/play.sh sealed::` plays it.
- **The weave's cursor was an id, and a step's id is what it *grants*** — so all
  eight concentration steps drew aimed at once, the panel read the first one's
  cost, and `→` jumped back to the second station: the line could not be walked
  past its fourth. It is a *mark* now, the total carried with it.
- **The Ley Line could be drawn wider than the pane it was given.** The pass that
  pushed stations apart had nothing pulling them back, so a narrow track clipped
  its tail away in silence while it was still spoken and still walked onto.
  **Latent today** — one pane, and every grid the game accepts leaves it wide
  enough — and live the day Phase 13a's second pane halves it. Frames come off
  before positions go wrong, a backward pass keeps everything inside, and a pane
  too narrow for even that says so.
- **The mastery cursor walked into rooms the player cannot enter**, where nothing
  is drawn and the panel then read out the deed and the room it opens — the
  foreshadowing every other surface withholds.
- **A siege counted before it reported**, so a first win printed what it opened
  above *"the wall holds"*. And a blank `ORBS_SEALED` read as `0`, quietly
  handing a player the open tower.
- **A save could lose what it was never able to name.** `Opened` was replaced
  wholesale on load, so a charm or a gated product a *later* build ships ungated
  was shut for the life of every existing document. Unioned with `Opened::start`
  now. And a mastery station's `opens` lands on exactly one tick ever — `advance`
  skips anything already reached — so both tracks catch up silently on load.
- **A sealed tower and the same tower reloaded were marked differently.** `seal`
  ran before the pylon's integrity and the dice's prices were published, while a
  restore seals last; four nodes differed in the component two sabotage queries
  read. Sealing moved after the publishers, a spawned node inherits its parent's
  mark, and `tests/sealed.rs` compares the two worlds by path.
- **`sealed_room_of` was wrong for the whole grimoire**, which is `/tower`'s
  sibling rather than its child — so twenty nodes carried the marker while the
  function said they were in no shut room.
- **`recall road` printed a raw template.** Every `recall_` key is a manual
  subject the parser knows, and the primer's two new lines took that prefix.
  Renamed, and lint-checked now.
- **`Taken` was restored after the ceiling was read**, so a document with no
  pool row came back four quintessence short per tier taken.
- **A gated product no station opens now fails the load** rather than being
  handed to every tower at tick 0, and `debug_reach` opens the room the line it
  reaches stands in rather than leaving a state no player can reach.

**✅ And the shipping profile was checked for the first time.** The gate builds
and tests in debug; with `debug_assertions` off, release had **18 warnings and 30
failing tests**, all of them from before this phase — a debug word does not exist
there, so every test that drives one fails against a world that was never built,
and the helpers only those tests reach read as dead code. Gated per item with the
same `cfg` their callers carry. **No product code was wrong.** `cargo clippy
--release` and `cargo test --release` are green now; whether they join the gate is
worth deciding rather than assuming. Two more found the same way: the
`bevy-idioms` verify harness could not be run at all (a nested package with no
`[workspace]` table of its own), and 53 broken or ambiguous doc links sat in
private docs that `cargo doc --no-deps` never lints.

**Three findings are left open**, because each is a design call rather than a
defect, and they are named here rather than quietly decided:

1. **A shut room's *interior* is nameable and tab-completable** — `attend ⇥` on
   the first line of a fresh game offers 54 names from inside the six shut rooms,
   including the four unearned charms. The room's own name is nameable on
   purpose (§19), so `attend archive` refuses in voice instead of fuzzing; the
   interiors were never separately decided. Splitting them needs a scene that can
   *resolve* a word without *offering* it.
2. **The shipped spellbook is readable and runnable while the grimoire is shut**
   — `peruse tending_blindly.spell` prints a forge script before the forge
   exists. Gating the spell verbs on the grimoire changes when the game teaches
   scripting, which is an onboarding decision (Phase 14's).
3. **The laboratory's fifth station is behind the archive's third.** `insight`
   needs mugwort and amber, which only a `verdant-scroll` shelves — so
   `laboratory_5`, `laboratory_6`, `haste` and `stillness` all wait on five mazes
   and ~6 scrolls, and nothing in the room says so. Reachable, not a dead end,
   and one of the `at`/`times` numbers already flagged as first-pass.

---

## Phase 11 — Renown

**Exit:** there is a reason to keep making things after the Ley Line stops buying
anything, and it is a number that can go down.

**The problem this answers is measured, not anticipated.** Experience past the
soft ending buys nothing and is consumed by nothing — the weave prints
`14213 of 10000` against a full bar. §11.5 said a prestige layer was *"none at
launch… revisit only if playtesting shows the late game flattens"*; the flat late
game is the current state by construction, so the condition is met without a
playtest.

**Renown is the tower's second number.** Earned by *making* things and by
fighting well, lost by fighting badly, spent to buy a smaller siege, and it sets
how big a siege arrives. It supersedes three recorded decisions and §19 carries
each: *"a second curve would be a second thing to balance and to save"*,
*"§11.5 keeps experience the one number that buys anything"*, and
*"quintessence: the tower's one spendable resource"*.

**What it does *not* supersede is `CAPACITY = 1`** — §19's *"four of the six new
phases spend the existing scarcity rather than minting a currency"*. That is the
strongest argument against this phase and it is answered rather than ignored:
renown does not compete with the focus slot, it prices a fight the slot has
nothing to say about, and every box below either mints on work already counted or
spends on something the arsenal cannot sell.

- [x] **Renown accrues and is seen** (`0.11.1`). The resource, minted at the one
      completion door, saved, and read on `status` and the weave's headings row.
      - **The mint is on *makings*, not on every completion.** `done` is also how
        events are counted, so minting on all twelve seams would pay renown for
        binding a spell and for *settling a siege* — twice, since escrow flows
        through the same call and escrow pays on a loss. `Work::sold` is the
        question, and only `Work::made` sets the key it reads.
      - **Earning is silent; only losing speaks.** The first pass said a line per
        making, and one balance sweep took the clarity loop from 466 records in
        two hours to 792 — *"word gets about"* per potion, for ever. That is
        `credit`'s lesson one file over: a continuous fact is read off `status`
        and only an edge is news. A *loss* is different — it happened to the
        player, possibly unwatched — so it keeps its sentence.
      - **Rounded up, not down.** `mortar_and_pestle` earns 1, so a halving that
        floored would mint nothing for the first instrument the apprenticeship
        teaches, saying that making things does not count.
      - **No `FORMAT` bump.** Nought is the honest reading of a document written
        before renown existed; `cooling` is the precedent.
      - **On the weave, not the rail.** The rail foot's own doc is *"these five
        and no more"* and argues anything already in `status` belongs there;
        experience is off it for that reason and renown has no better claim.
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal;
      debug_spawn clarified-draught 1; distil clarified-draught; meditate 60;
      status"` — `renown ......... 4` under `experience ..... 8`; and
      `ORBS_WEAVE="mastery"` in place of `status` puts `renown 4` at the right of
      the headings row.
- [x] **The siege moves it both ways** (`0.11.11`). Per-exchange in `hold`, the
      outcome in `settle`, after the siege's own sentence.
      - **Five fields, not four.** The box named `dealt` and `sortied` up,
        `taken` and `spent` down — and `mended` is the fifth. `taken` and
        `spent` are vigour lost and `mended` is vigour put back, so charging
        the first two without the third **bills a player twice for damage they
        repaired**, and the play it punishes is quaffing a `mending` or
        pledging the `succour`. `(taken + spent) − mended` is net vigour lost.
      - **Rounds move it quietly; the fight speaks once, on settling.** A round
        is elected by typing `hold` and has just narrated itself, so a sentence
        per exchange is the same news twice over, six to thirteen times. But
        silence *and* no summary would leave the gauge as the only signal —
        and the gauges yield on a short pane, and §14's stream carries records
        rather than gauges. One record a siege, and it is what gives a
        mid-siege rank loss its explanation.
      - **The sentence measures from when the enemy arrived.** `Siege::standing`
        is an opening snapshot, so the number is the *fight's* — a reading taken
        at `settle` reports the stake and silently omits what the rounds cost.
      - **A fall costs a stake scaled by how far short it fell**, `div_ceil` so
        that losing at ninety-nine percent costs *less* rather than *nothing*:
        at 5..=9 foes, plain division truncates every near miss to nought.
      - **`RENOWN_PER_FOE` is `ESCROW_PER_FOE / RENOWN_PER` = 7**, derived so a
        siege's standing is priced at the rate a making's is. A first pass read
        `arrived` as 21 — that is `arrived × VIGOUR`, the enemy's *vigour* — and
        at one per foe a defeat could never cost even the first rank, which is
        the whole point of the box. §19.
      **See it:** a round moves it, and says nothing —
      `ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="attend bailey; defend; status; pledge d20 to buckler; hold; status" cargo run -p orbs`
      → `renown 0` then `you lose 1 and take 3` then `renown 2`, with no line
      between. **It must pledge**: an unpledged round trades about evenly and
      nets nought, which is why the box's original line showed `renown 0` twice
      and demonstrated nothing.
      Then the fight speaking — `debug_siege` in place of the first `status`
      gives `they are singing about it: 37 renown, 37 in all` under
      `the wall holds`. And a loss, in the log where a spell's records go:
      `ORBS_BOOT=0 ORBS_DUMP="debug_renown 200; attend bailey" ORBS_THEN="invoke besieging; meditate 900; peruse bailey.log" cargo run -p orbs`
      → `word gets about: the wall cost 31 renown, 169 left`
- [x] **Ten ranks, and a gauge for each standing** (`0.11.2`). Two bracketed
      bars at the top of every pane — the Ley Line toward its next station and
      renown toward its next title — and the ten names the tower is called.
      - **Titles, not gates.** A rank names where you stand and unlocks nothing,
        which is what makes losing one safe: a rank that gated a capability could
        take it away mid-siege, in the fight that needed it.
      - **Toward the next tier, from the tier behind.** A gauge that filled from
        nought would jump backwards on every crossing — 99% of the way to 150,
        then 4% of the way to 350. `Toward::among` is the one arithmetic and both
        gauges share it, so they cannot disagree about *nearly there*.
      - **A second bar vocabulary, deliberately.** `Painter::gauge` draws
        `[||||    ]` where `meter` draws `█░`, because it measures a *tier* and
        not a run in flight; the brackets give it ends, which a bare fill has
        not. Capped at 40 cells — stretched across a 120-cell pane the bar became
        a rule with a number at the end, and the eye could not tell a third from
        a half.
      - **Three sentences for a rank, not two.** Climbing says one thing; falling
        to a lower title says another; falling out of the ranks says a third. Two
        of them congratulated a player who had just lost two ranks.
      - **The rows yield before the transcript.** §9's main window does not
        yield, so the gauges hand the body back untouched in a short or narrow
        pane, exactly as the road does.
      **See it:** `ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="debug_renown 900;
      attend laboratory"` — `renown [||||...] 100/800  magister` under the ley
      gauge; then `debug_renown 900; debug_renown 100; debug_renown 0` for the
      three sentences in order.
- [x] **Renown sets the siege, and `petition` turns it down** (`0.11.12`).
      Fame lengthens the tail of what comes up the road; `petition` spends
      standing to shorten it again. **The first thing renown buys.**
      - **The floor never moves.** `5` at every standing, with the ceiling
        climbing `9 → 12` over ten ranks. A famous tower can still draw a quiet
        night and an unknown one never meets the worst, which is what keeps
        variance meaningful at both ends instead of squeezing it against the top.
      - **Twelve is a written ceiling, not a drawn one.** The garrison is six and
        `outnumbered` is a *ratio* — `enemy >= garrison * 2` — so twelve is
        exactly where that reading turns over at the opening. Past it the rung
        three shipped solvers branch on would be true in every fight; short of it
        the reading is unreachable until the garrison is thinned.
      - **`petition` lowers the ceiling; it does not subtract after the draw.**
        At the moment the word is typed there is no siege and no draw, so *"the
        enemy is already as small as it goes"* is only answerable against the
        ceiling. Subtracting afterwards would let a player pay four times, draw
        the floor anyway, and lose the lot in silence.
      - **Self-limiting without a second rule**, and it is visible: paying drops
        a rank, and a lower rank draws a shorter tail on its own. Three petitions
        at 9,000 renown take the ceiling `11 → 9 → 8`, not `11 → 10 → 9`.
      - **`begin` took a parameter back**, and §19 records why it is not the one
        that was removed: this one is consumed *inside* the first draw rather
        than sitting before it, so it cannot reorder either draw. `begin(rngs)`
        is still the whole of the old behaviour, so **every existing seed is
        untouched** — renown is nought in every existing siege fixture.
      **See it:** the tail lengthens —
      `ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend" cargo run -p orbs` draws
      `5 come up the road`, and the same line after `debug_renown 9000` draws
      `6`. Then buying it down:
      `ORBS_BOOT=0 ORBS_DUMP="debug_renown 9000; attend bailey; petition; petition; petition; defend" cargo run -p orbs`
      — `word goes out. at most 11 will come, for 7 renown`, then `9`, then `8`.
- [x] **The gauges warm as they fill** (`0.11.4`). Red through yellow to green,
      six steps, so a bar's colour says how close the next tier is before the
      numbers are read.
      - **Colour is a `Depiction`, not a `Role`.** The shell may not resolve a
        colour — a boundary test says so — so the ramp is chosen in
        `orbs-render` and each frontend answers for it from its own theme.
      - **The caller passes no accent.** A depiction is dropped on any accented
        cell, because §4 reserves the triad for meaning; the first pass passed
        `Role::Success` and would have drawn every gauge flat green for ever.
      - **Green is arrival, not approach.** An even sixth at the top would paint
        a bar one short of its tier the same as one that had reached it.
      - **§14 is satisfied because the fill length is the information.** Strip
        every colour and the bar still says how full it is.
      **See it:** `cargo run -p orbs-render --example screens` — the ramp printed
      as `A`–`F` at every tenth, which is the *only* See-it it has: the glyph is
      `|` at every step, so a dump shows a bar filling and proves nothing about
      the warming. On a tube, `ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="attend
      laboratory"` and watch the two rows at the top.
- [x] **The grimoire leaves the rail** (`0.11.3`). No box, no mastery line.
      §10's seven domains are *kinds of play*; `DOMAINS` is the rooms you **work
      in**, and those were the same list by accident rather than by design. The
      grimoire raises no instrument, earns nothing and anchors no verb, so its
      box read `idle` for ever and its three stations counted spells bound — a
      deed done wherever the player is — and opened nothing.
      The decision was already recorded three phases earlier, when `/grimoire`
      became a sibling of `/tower`: *"a root domain but not a §9 activity
      domain… you do not run the grimoire concurrently with brewing"*. The code
      had not caught up. The bailey settles it by comparison: sieges are fought
      there and it has never had a box.
      **What it keeps:** the spells still live there, `scribe` still writes into
      it, it is still protected, and it is still shut until the ley step at 16 —
      which is now `opened::is_room`'s question rather than `DOMAINS`'s, one
      function where the rule had two copies and would have had three.
      **See it:** `ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="attend laboratory"` —
      six boxes on the rail and no grimoire; `ORBS_SEALED=1 ORBS_DUMP="attend
      grimoire"` still refuses in voice, and two clarities still open it.
- [x] **The arsenal is worth what your industry is worth** (`0.11.13`). How much
      help a thing gives is matched to the **rate** you make it at: `fresh`,
      `thin` at half, `spent` and refused in voice.
      **This replaces the cap, which replaced a perishable arsenal.** ~~A cap on
      how much of one name the arsenal takes~~ — **struck**, and §19 carries the
      argument. Its own text admitted the problem was hypothetical (*"which
      nobody has been"*), renown now answers *"why keep making things"*, and a
      cap is a **wall rather than a bottleneck**: a ceiling you hit and stop at,
      which is the opposite of the *"production must be continuous and varied"*
      intent it inherited.
      - **A rate, not a timestamp — and the difference is the whole feature.** A
        first pass keyed this to *when one was last made*, which is the exact
        hole §19 recorded withdrawing perishability for: *"a timer either keeps a
        thousand potions fresh off one restock — cheaper than playing normally"*.
        One making refreshing an unbounded pile **is** that. A rate cannot be
        gamed that way: one restock is a rate of one, which is thin at best.
      - **Total stock never enters the arithmetic**, which is what makes it
        buildable on `Stock::Counted(u32)` untouched — no per-unit batches, no
        rewrite of every reader, no save migration. The thing that was impossible
        is the thing this does not need.
      - **Surplus sells.** Making something already `fresh` mints its renown
        twice — so nothing is ever refused at the *door*, and there is no point
        at which producing stops being worth it. It also dissolves the cap's one
        unanswerable case: a chant gives up to four troops *after* the figure has
        closed, with nowhere to refuse. Under this it simply sells.
      - **A boolean effect has no half.** `advantage` is draw-twice and `upgrade`
        is a bigger die; two shipped spendables are one. A thin store loses them
        outright rather than being spent for nothing, and the rule is authored on
        `Effect::scales` rather than guessed at the call site.
      - **It does not decay while the game is closed, and that needed no code** —
        a tick is *"one real second while the window is open"*. **And it will
        decay across an absence the day Phase 13a's offline catch-up advances the
        tick**, with the same expression. Correct now, correct later.
      - **`debug_spawn` stamps a *full store*, not one making** — and it took
        three failing tests to get that right. Without any stamp every spawned
        thing arrives `spent`; with one stamp it arrives `thin`, which halved a
        spawned troop's bodies and made a test of the **Ley Line's garrison
        grant** fail on the arsenal's freshness rule instead. Its own sentence is
        *"the shelf finds it had it all along"*, and the industry behind the
        shelf is part of *all along*.
      - **The balance driver learned to keep its arsenal rather than fill it** —
        a top-up rung, not a trip to the laboratory, which would have made the
        pinned rate a blend of two domains.
      **See it:** stock it, then watch it go —
      `ORBS_BOOT=0 ORBS_DUMP="debug_spawn warding 5; survey arsenal; meditate 1200; survey arsenal; meditate 1800; survey arsenal" cargo run -p orbs`
      → `state: fresh`, `fresh`, then `spent`, with **`qty: 5` throughout**,
      which is the proof stock is not what is being measured. Then the refusal,
      which keeps the potion:
      `ORBS_BOOT=0 ORBS_DUMP="debug_spawn warding 1; meditate 3000; attend bailey; defend; quaff warding; survey arsenal" cargo run -p orbs`
      → `your warding stores are out. make one and come back`, and `qty: 1` still
      on the shelf after it.
- [x] **A spell can ask what has gone thin** (`0.11.14`). The other half of the
      mechanic above: stores run down with time, and a player who cannot ask
      *which* can only guess. `for each store` walks the arsenal and
      `if the store has thin` answers — so the reply to a thinning arsenal is a
      spell that keeps making things, which is pillar 3 landing as a mechanic
      rather than a promise.
      - **Two Cwd-scoped lookups had to learn about the arsenal**, and neither
        was optional. `group_at` walks the room a spell stands in, and
        `watch::find` resolves a `has` against that room's children — so
        `for each store` found nothing from the bailey, and once it did, the very
        next line could not ask the cursor a question. §19's arsenal exemption is
        *"the one room reachable from every other"* and `scene` already folds its
        contents into the naming scope; this is that decision finished rather
        than extended.
      - **A store's word is republished by a system, because nothing else can.**
        Every other reading is rewritten by the thing that changed it; a store
        changes because *time passed*. It writes only when the word changes, so
        an idle tick issues no `NodeId` and §19's *insertion order is the parse*
        holds between a watched hour and a `meditate`-collapsed one.
      - **The first version of its test passed vacuously**, and that is why the
        test now asserts the loop *body ran* rather than only that the spell
        compiled clean. A `for each` over an empty set complains about nothing
        and does nothing — which is the exact silent defect `scene.rs` says the
        registration loop exists to prevent.
      **See it:** `cargo test -p orbs-sim --test scripting_the_siege a_spell_can_walk`
      — a spell that walks the arsenal, finds what is thin, and acts on it.
- [x] **Docs** (`0.11.14`). Audited item by item rather than assumed, and **three
      of the six were already done** by the steps that earned them — the three
      supersessions, the `CAPACITY = 1` answer and the withdrawn arsenal are all
      in §19's Renown entry, and SEEING-IT gained its *Renown* section at
      `0.11.11`. What was actually missing:
      - **The fourth renumber had no §19 entry**, while ROADMAP's own preamble
        asserted *"§19 records that the scheme has now been bent four times"* —
        and it recorded three. Renown's insertion at 11 pushed
        tower-as-one-machine → 12, breadth/remote/engine → 13a/b/c, onboarding →
        14, ship → 15. Written now, highest-first, with the historical tables
        left on their own numbers.
      - **§11.5's resource table was two rows short**, not one: **Renown**, and
        **Stores** — the arsenal's rate, which is a resource consumed by *time*
        and by nothing else.
      - **§19 said the arsenal cap had shipped, and it never did.** `stock::give`
        is a bare `saturating_add` and always was. A wrong tense in a decisions
        log cost an independent reviewer a false defect report; it is struck, and
        the cap is now superseded rather than pending.
      **See it:** `grep -c "bent four times" docs/ROADMAP.md` and
      `grep -n "moved a fourth time" docs/DESIGN.md` now agree; §11.5's table has
      nine rows; and the struck cap says what replaced it.

---

## Phase 11.5 — Interlude

**Aesthetic, and none of it turned out to be only aesthetic**, which is Phase
0.5's framing because this is Phase 0.5's shape. That one made the orb *a machine
that moves*; this one makes the tower **a place you move through.**

Every screen in the game cut. `attend forge` swapped the laboratory's instruments
for the forge's lattice between one frame and the next, and §19 had already
recorded that exact defect for pane *geometry* in Phase 0.5 — *"a pane appearing
between one frame and the next reads as a glitch."* Pane **content** was never
addressed, because until Phase 10 raised seven domains there was nowhere much to
go. Now there is.

**No minor of its own**, exactly as Phase 0.5 takes none: an interlude carries no
month or word budget, so the table's arithmetic above is untouched and nothing
renumbers. Its steps run `0.11.5`–`0.11.8`. Phase 11 was open when this started,
so those interleave with Renown's — recorded in §19 rather than hidden, and the
alternative was holding this until Phase 11 closed.

- [x] **A screen leaves by a shape** (`0.11.5`) — `orbs_render::passage`, the
      `Frame` pass, and the kept cells a crossing departs from. `Wipe` alone:
      three shapes and a shape-per-change table would be a vocabulary invented
      before anyone had seen a frame of the first one
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
      crossings. `orbs-tui` holds a settled one and does not animate: it has no
      switch a player can reach, and §14 will not have motion without one
      **See it:** ✅ `cargo run -p orbs`, then `attend forge`. Then `F3` to `OFF`
      and `attend laboratory` — it cuts
- [x] **Three shapes, one per kind of change** (`0.11.8`) — `Wipe` for a room,
      `Gather` for a tool taking the pane, `Furl` for `F5`'s mirror
      **See it:** ✅ `attend forge`, then `wander`, then `F5` — three motions for
      three kinds of change
- [x] **The opening is a crossing too** (`0.11.10`) — the first screen change a
      player meets was the last one still cutting. The name arrives **a letter at
      a time, each growing in from its own middle** — `O.`, `R.`, `B.`, `S.` —
      then the subtitle on its own clock, then the report; `Stage::Close` takes
      the card away the same way; and the tower opens with the rail pushing in
      from the right and the gauges and road pushing down from the top. **The
      card is a third shorter** for it: three legible events in a row do not need
      the pauses one slow event did.

      **The box stays through all of it**, which is the decision: the border the
      card drew for itself is the pane the game arrives in, and what moves it is
      the rail narrowing it from the side. Folding it away would mean drawing a
      second one over the hole a frame later.
      **See it:** ✅ `cargo run -p orbs` and watch the opening. As text:
      `ORBS_DUMP=1 ORBS_BOOT=post:0.06`, `close:0.4`, and
      `ORBS_PASSAGE_AT=wake:0.35 ORBS_DUMP="attend laboratory"`

      ✅ **Each region leaves by its own edge** (`0.11.9`) — the gauges and the
      road go *up*, the panel and the board go *right*, and both come back the
      way they went. It crossed one rectangle with a hole in it before, which
      said what not to touch and nothing about direction, so the top strip wiped
      sideways across the screen it was sitting on.
      ✅ **A gather actually converges** — glyphs fly to the middle and back out
      of it, rather than eroding inward from the edges. That needed §19's
      per-cell flash rule re-examined: it forbade *any* motion, by counting a
      glyph travelling past a cell as a flash. Superseded per family — erosion
      keeps the per-cell rule, convergence keeps a field-level envelope, and both
      are asserted.
      ✅ **The new screen no longer flashes whole for a frame** — `Drive` had no
      ordering edge to `Input`, so a `wander` drew the finished maze and only
      then transitioned away from it. Third time a set has been ordered against
      its reader and not its writer; §19 records the set.
      **See it:** ✅ `cargo run -p orbs`, then `wander` — the laboratory flies
      into the middle and the maze flies out of it, with no whole frame between

**What it deliberately does not touch: the transcript, the border, its title, the
tower rail and the prompt.** History did not change when you walked to the forge,
and blanking it would say the session went away; a box that came apart would read
as the *machine* breaking rather than the screen changing, which is what got the
tube strike cut twice.

---

## The orb's menu

**Exit:** a player can leave a game, start another at a different length, and
come back to the first, unharmed.

**The curve was calibrated to one loop, not a tower.** The Ley Line tops out at
10,000 experience — roughly five thousand hand-played commands, which is
genuinely reachable by typing. That makes playing manually barely worse than
automating, and undercuts pillar 3: *automation is progression*. A tower you can
finish by hand is a tower that never has to teach you to write a spell.

So **a game has a length, chosen once when it begins** — and the place to choose
it is a screen the orb has never had.

> **The head is anchored and the tail is stretched.** A flat multiplier would
> break the thing this exists to serve: `progression.toml` says of the first
> station that *"the player does the whole loop by hand once, and the reward is
> not having to do it again"*, and ×6 makes that six hand-brewed clarities before
> the first spell slot. The stretch ramps quadratically from nothing at a line's
> first station to its full factor at the last, which also dissolves the
> room-reveal problem **without a list of exemptions** — a first draft named five
> stations to protect and three of the five were wrong.

> **The tiers are the curve, not the clock.** Each is defined by what the last
> station reads, which is a fact; what that costs in hours is for `orbs-balance`
> to measure. §19 records the tower's automated rate as an open question —
> *"either the line's top or additivity is wrong"* — and `bound` currently
> measures **0.0910 against `grind`'s 0.1000**, so automation *costs* 9%
> throughput and its win is running unattended rather than running faster. A tier
> defined in hours would be a guess wearing a fact's clothes. **Nothing here
> settles that question.**

| Tier | Last ley station |
|---|---|
| Short | 30,000 |
| **Medium** (default) | **60,000** |
| Long | 250,000 |

- [x] **A game has a length** (`0.12.0`) — `Length`, the anchored quadratic ramp,
      and `Progression::stretched` applied before `check` so the load gate sees
      the stretched curve. **`earns` is never scaled** — those are the rates, and
      scaling them cancels the feature exactly while every rate pin still passes,
      which is the one failure that would go entirely silent. Fifteen prose keys
      that spelled their counts out in English (*"five potions brewed"*) now
      interpolate `{count}`, because they are drawn beside a live `3 of 5`
      counter and at any other length the sentence contradicted the number next
      to it
      **See it:** ✅ `ORBS_LENGTH=long ORBS_SEALED=1 ORBS_SAVE=off ORBS_DUMP="weave" cargo run -p orbs`
      reads 250,000 where baseline reads 10,000; and
      `ORBS_LENGTH=medium … ORBS_DUMP="debug_reach laboratory_1; attend laboratory"`
      says *"6 potions brewed"* where baseline says five
- [x] **The length threads the tower** (`0.12.1`) — the private constructor
      funnel, `Sim::measured` and `Sim::begun`, `WorldSave.length` and `FORMAT`
      10 → 11, `ORBS_LENGTH`, and `--length` on `orbs-balance`. **`Sim::restored`
      is the one that would have shipped broken**: it calls `bare`, which
      installs the *unpaced* curve, and `save::restore` never touches
      `Progression` — every save would have loaded back at ×1. **No lint would
      have caught it**: `persistence.rs`'s completeness check does cover
      resources, and it did fire for `Length` — but what it asserts is that a
      resource is *declared* in the document, never that `restore` puts it back,
      so a `restored` world rebuilt at the wrong length passes it green. **The
      default splits like
      `ORBS_SEALED` does**: the game is medium, the dump is baseline, or all 138
      `dumps.sh` captures move
      **See it:** ✅ `cargo run -p orbs-balance -- sweep --hours 1 --length long --why`,
      and a save round-tripped at a length reads the same curve back
- [x] **The weave reads six figures** (`0.12.2`) — at long the last station is
      250,000, and the pane drew `121766250000`: two totals run together into a
      number nobody authored, which is the exact failure the `index % 2` stagger
      was added to fix one scale earlier. Two rows buy `2 * GAP` = **six cells**,
      which is five figures and no more, so a **six-figure total abbreviates**
      (`250k`) rather than staggering a third time. It rounds **up** — a
      threshold labelled lower than it is would say a station is nearer than it
      is — and the details pane, `weave_locked` and `weave_bar`'s spoken form all
      keep the number exact. **Nothing below six figures moves**, so baseline,
      short and medium draw precisely what they drew and the 138 dump captures
      hold. The bar and the track now take one `gauge()` rather than two
      `format!`s that could drift apart and desync the fill from the stations.

      **`along()` needed no length of its own, and the plan had its direction
      backwards.** `scale` is the denominator, so a longer game moves every
      station *left*, not right: the first sits at `ln(17)/ln(10001)` = 0.31 of
      the run at baseline and 0.23 at long. `pack`'s two passes already hold the
      spacing whatever `along` wants, so the picture was never the problem — the
      label was. The correction is in `along`'s docs, and it was settled by
      looking at the pane rather than by arguing
      **See it:** ✅ `ORBS_LENGTH=long ORBS_SEALED=1 ORBS_SAVE=off ORBS_BOOT=0 ORBS_DUMP="weave" cargo run -p orbs`
      — every total clear of its neighbour, `0 of 250k` in the gauge; the same
      line at `baseline` and `medium` is unchanged, cell for cell
- [x] **`menu` opens the screen, and `quit` asks before it leaves** (`0.12.3`,
      corrected at `0.12.7`) — **`quit` opened the menu for one iteration and
      that was wrong.** The argument was `quit`'s own doc: *"the word means leave
      the thing you are in, whichever thing that is."* What it cost was making
      **stopping a two-step operation through a screen the player had not asked
      for**, which is the opposite of what a way out is for. Superseded (§19).

      So `menu` is its own verb — `Verb::ALL` is 46, and its own `Menuing`
      handshake sits beside the other five — and `quit` leaves. **It asks once
      first**: `quit` puts the question, another `quit` answers it, and any other
      command answers *no*. That is the guard the word needs and nothing else in
      the game does, because it is the only one that cannot be undone, waited out
      or repeated away, and the tower is written on the way. **A word, not a
      screen**: a confirmation surface would be one more thing that takes the
      keyboard, and the one below is why that matters.

      `Focus::Menu` is **first** in the ordering — the way out wins any tie — and
      takes the pane; the 11.5 crossing came free, because `Showing::of` picks up
      any pane-taking focus as `Passage::Gather`. **`F10`, `Ctrl-C` and the
      window's close button still leave outright**, without asking: a key that
      says *close this window* should.
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="menu" cargo run -q -p orbs`, with
      `ORBS_MENU="resume"` and `ORBS_MENU="zorb"`; and
      `ORBS_DUMP="quit" ORBS_THEN="quit"` against
      `ORBS_THEN="status; quit"` — the second leaves, the third asks again. Seven
      latches in `scripts/dumps.sh`
- [x] **The menu does not eat the word that opened it** (`0.12.7`) — **a shipped
      defect, reported by a player and reproduced before it was fixed.** Typing
      the word appeared to exit the game outright. `type_into_menu` was gated on
      the menu being open, and **a gated reader keeps its cursor** — so the first
      time it ran it read whatever `Messages` still held, which on the frame the
      menu opened was the word that opened it. Typed straight back in, that word
      reached the menu's own `quit` and wrote an `AppExit`.

      `focus.rs` already states the rule it broke: *"a surface that grabs the
      keyboard on open eats the player's first keystroke."* The fix is
      `type_into_line`'s shape — **run always, decline, and clear** — plus an
      ordering edge putting it *before* `open_requested`, so the opening frame is
      one the reader has already emptied.

      **The instrument was blind to it and looked stable.** `ORBS_DUMP` builds no
      `App` and presses no key, so it drew a perfect menu throughout; the four
      `dumps.sh` latches were green. The gate a keyboard-owning surface actually
      needs is a test that fires a real `KeyboardInput` at the real plugin stack
      **See it:** ✅ `cargo test -p orbs --bins opening_the_menu` — it fails on
      the old ordering and passes on the new
- [x] **More than one save** (`0.12.4`) — `read_from`/`write_to` beneath the
      current pair, a `saves()` listing and `free_slot()`. **No `FORMAT` bump**:
      wizard, seed, tick, length and `away.unix` were all already in the
      document and only ever read by one caller. Slot 1 **is** `orbs-save.toml`,
      so nothing migrates; 2–6 sit beside it. Three constraints held —
      `saves()` reads through a private `load` that **cannot** reach
      `keep_aside`, so a listing can never rename the files it lists;
      `ORBS_SAVE=off` and `=<file>` mean exactly what they meant; and there is
      **no process-global current path**
      **See it:** ✅ `ORBS_SAVE=/tmp/t/orbs-save.toml ORBS_BOOT=0 ORBS_DUMP="quit" ORBS_MENU="saves" cargo run -q -p orbs`
      — the towers listed with wizard, length and experience
- [x] **Swapping the tower safely** (`0.12.5`) — **the dangerous part, and it
      is one line in one position.** Autosave fires every 60 ticks and on
      `AppExit` against whatever path it holds, so a swap that replaced the path
      first would write the tower being *left* into the file of the tower being
      *opened*. `Kept` makes the path travel **with** the `Sim` and they are
      replaced together; the outgoing tower is written before anything else
      moves.

      **The eighteen resources are now one list read twice.** A hand-written
      reset list would drift, and what it drifts into is invisible: nothing
      crashes, the screen just describes the wrong tower. `shell_resources!` is
      registered from and reset from, so adding one does both in the same edit —
      and the two that survive a swap (`Screen`, the window; `Standing`, the menu
      doing the asking) are lifted out and put back rather than left off the
      list. `SimPlugin::build`'s three-way branch was extracted to `raise` rather
      than written a fourth time, prose is re-applied, and the incoming save is
      **read before anything is torn down** so one that will not open leaves the
      player where they were. The terminal build takes the tidier shape the plan
      wanted: a restart loop rebuilding `Session`, which resets every derived
      field by construction
      **See it:** ✅ `cargo test -p orbs --bins menuing` — four tests, and
      `loading_a_second_tower_writes_the_first_to_its_own_file` is the exit
      criterion: load a second tower, and the first is still in its own file with
      its own seed. **That is exactly what autosave would have broken**
- [x] **A new game, and its length chosen** (`0.12.6`) — the three tiers, offered
      by prefix. `baseline` is deliberately not among them: it is the numbers we
      happened to author first rather than a difficulty. Refused *before* the
      length is asked for when the orb is full, rather than after — asking a
      question and then discarding the answer is the dead end §15 names
      **See it:** ✅ `ORBS_SAVE=/tmp/t/orbs-save.toml ORBS_BOOT=0 ORBS_DUMP="quit" ORBS_MENU="new" cargo run -q -p orbs`
      offers them, and `a_new_game_is_begun_at_the_length_that_was_chosen` reads
      the length back off the new world's save

---

## The tower as one machine

*Was **Phase 12**, and the prose below still calls it that — as a name, not a
position. It is not next; nothing is.*

**Exit:** the player leaves, comes back, and the tower ran itself — *across*
rooms, not in one.

Six rooms is not a series. This is the phase that makes the puzzles compose, and
it collects two things that are **currently parked in the siege phase and would
be stranded there** by a mechanical renumber.

> **The web is what this phase is for, and §19 records the research behind it.**
> The tower counts *runs*, not things, so no bottleneck in the game is denominated
> in goods and nothing a player makes is ever what they run out of. The order
> below is not the old order: **pane addressing and slot reservation come first**,
> because a web that must be routed by hand is an elaborate to-do list, which is
> the documented way players fall off this genre. And the crossing itself turned
> out to be **most of the way built** — `keep::admits` already takes
> `Essence | Scroll` by kind from any room, so potions and scrolls cross today.

- [ ] **The calm layer stops poisoning logs** — §5.1 says the idle layer touches
      *"environmental only, never scripts, schedules, or logs"* and
      `sabotage::drift` targets `With<Log>` unguarded in the tick schedule. It
      does not heal, either: the settling path only restores `Substituted` nodes,
      so a poisoned log stays poisoned, and `drift`'s own comment records
      saturation at ~1500 unattended ticks.
      **This phase's exit is "leave, come back, and the tower ran itself"**, and a
      tower that lies in every log after twenty-five minutes away fails it by
      construction. Guard it to the siege, or give it the decay `substitution`
      already has. §19 has the five tests and which two fail
      **See it:** `ORBS_BOOT=0 ORBS_DUMP="meditate 1800; peruse laboratory.log"` —
      a fresh tower, half an hour alone, and nothing in the log is a lie
- [ ] **`stillness` and `vigour` can be spent** — two rows in `siege.toml` and the
      lint that stops it recurring. Both are real potions, gated and prosed, with
      **out-degree zero**: `defend::spending` refuses any name it has no entry
      for, and no other system in the game reads a potion at all. Phase 9 noted
      they *"have no sink at all and say so"*; what it did not say is that nothing
      else could ever spend them
      **See it:** `debug_spawn stillness 1; attend bailey; defend; quaff stillness`
      — and `every_potion_the_alembic_makes_can_be_spent`, which is
      `every_scroll_the_lectern_makes_can_be_spent` one file over and is why the
      scrolls never had this problem
- [ ] Pane addressing — acting on a domain you are not standing in. **First, not
      fourth** (§19)
      **See it:** start a grind from the archive
- [ ] **Focus-slot reservation**, which is the *same* roadmap item as pane
      addressing and moves with it. `CAPACITY = 1` is tower-wide, so this phase's
      exit is unreachable by construction until it rises
      **See it:** two rooms working at once, and the sidebar accounting for both
- [ ] **A made good crosses and another room consumes it.** No new `NounKind`, no
      content file and no save migration — a `warding` essence already reaches the
      sanctum through the arsenal; what is new is the `pylon` having a recipe that
      takes it (Phase 9). A ward set in one **holds** after the course ends
      **See it:** carry a `warding` to the sanctum and set a ward that outlives
      its course
- [ ] **The player can see a bottleneck.** No surface in the game shows a rate or
      a blocker — but the limiting factor is **already computed and discarded**:
      `Blocked::{Instrument, Slot}` carries the blocker's name and state and is
      emitted once as a log line, and `State::Gathering` already knows its missing
      input. One defect rides with it: `brief::busiest` is
      `.find(is_busy).or_else(first)` over `Working | Scouring | Burning`, so a
      laboratory with a fouled alembic and a cold athanor draws as
      `laboratory / idle` — **the rail is most silent exactly when a room is
      stalled**
      **See it:** `status rates` — every room's goods a tick, and a stalled room
      names what it is waiting on. *(Bare, not `--rates`: the parser has no flag
      syntax, which already forced `verify --all` to be respelled.)*
- [ ] **One good, two consumers, three answers — the phase's real exit.** A
      `warding` potion is worth a siege round *or* a ward that holds, never both,
      and the shortage is answerable three ways: brew more (spend the slot), a
      `fruitful` alembic (spend quintessence), or garrison the wall with a troop
      (spend a chant). Quintessence spent on the charm is quintessence not pledged
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

## Phase 8 — Siege ✅

**Exit:** sieges are tense and scripts visibly matter.

**Built at `0.8.1`–`0.8.7`.** The bailey, a turn-based defence resolved on D&D's
seven dice, the arsenal spent on it at last, a decision tree that fights one
unaided, escrow, and the adversarial aberrations that close §8.1's four-surface
model. **Three items are deliberately left** and are named at the foot.

**It is not an eighth domain.** §10 fixes the count at *"seven at launch"* and
lists them; §5 says you *"descend into"* a siege. The bailey is the **arsenal's**
shape — a real place in the tree, with its own log and its own verbs, that is
deliberately not a rail box. The rail's seven slots are the argument as much as
the table is.

- [x] **Autobattler** — `defend` lets an enemy arrive, `hold` ends your turn and
      resolves one round, and both sides roll. §10's rule is met by
      construction: **there is no clock in it**, so outcome follows what the
      player chooses given readable state and never how fast they act. §10.1's
      one exception (the menagerie) is not extended
      - **Turn-based serves the hand first and the script second**, which is the
        argument the first plan missed by resting it on accessibility alone. A
        real-time siege would outrun a decision tree evaluated at
        `SCRIPT_BUDGET`, which is `PACE = 1` in the menagerie (§19) one room over
      - **§5.0's *"no per-command tick cost"* is preserved.** Everything on your
        turn is free and instant; the clock advances on `hold` and nowhere else
      - **It takes no production slot** (§19), which the domain needs most: a
        siege exists to test the automation, so freezing it would leave the enemy
        nothing to attack
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; survey enemy;
      survey garrison; hold"` — the enemy says what it means to do, both bands
      read back, and one round resolves
- [x] **The seven dice, composed before they are resolved** — `d20`, `d100`,
      `d12`, `d10`, `d8`, `d6`, `d4`, in `tower::dice`. A `Roll` is *assembled* —
      which die, what modifies it, what it is against — and only then drawn
      - **This is the piece that cannot be retrofitted**, and it is why it was
        built before anything rolled: with the draw at the call site, every call
        site has to change to admit a modifier, and there is one per kind of
        attack
      - **Each modifier carries its source**, so the log says *why* — rule 4, and
        what makes `peruse bailey.log` a postmortem rather than a list of results
      - **The draw count is a function of the composed roll, never of the
        outcome**, so advantage is replayable where a reroll-on-miss would be
        §19's `drift` defect
      **See it:** ✅ `peruse bailey.log` after a round names the die and the face
      for every roll: `enemy strikes - d20 gives 12 against 11, and it tells`
- [x] **The board** — both bands, the telegraphed intent, and **the odds before
      the commitment**, which is §5.1's fairness rule drawn. Strength is bar
      *length*, never colour (§14), so it reads in greyscale and in a dump
      **See it:** ✅ `cargo run -p orbs-render --example screens` draws it with no
      sim at all; ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend"` draws it
      beside the transcript
- [x] **The arsenal spends, and the ten apology lines retire.** Potions, scrolls
      and the menagerie's troop are pure mathematical advantages applied on your
      turn — authored in `content/siege.toml` (rule 6), so a tuning pass is a
      content edit rather than a recompile per guess
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend;
      debug_spawn troop 2; deploy troop; quaff troop; deploy sage"` — the troop
      joins the line, and both refusals name the way forward
- [x] **The decision tree** — `besieging`, on the grimoire's shelf in a debug
      build. The shape the brief named: *if base troops low, send demons; if
      health low, apply potions; if outnumbered, wield a scroll*. Every rung is a
      reading the world publishes, because the language has no arithmetic and is
      not getting any — the maze's pattern, where `spoil` is a word rather than a
      sum
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; debug_spawn troop 4"
      ORBS_THEN="invoke besieging; meditate 300; peruse bailey.log"` — it fights
      and wins unaided
- [x] **Escrow economy** (progress-scaled, 20% floor, 50% completion bonus),
      exactly §11.5's table. Losing at 60% keeps 60%; bailing at nought still
      pays the floor — *"effort is never wasted; only cynicism is"*
      **See it:** ✅ a lost siege reads `the wall is carried. 22 of the way, and
      you keep 27`
- [x] **Adversarial aberrations across all four surfaces.** §8.1's model is
      closed: logs and world state shipped in Phase 1, and `tower::assault` adds
      **script text** and **trigger clocks** — the two that reach a *script*, and
      the two §19 moved here *"where their producer is"*
      - **Siege-only**, which is what keeps Phase A genuinely safe (pillar 4).
        Nothing in that module runs on a tick; it runs on a resolved round
      - **A retimed spell reads perfectly and stops keeping up**, which makes it
        the subtlest of the four — only `verify` finds it. Floored at one step a
        tick, because §8's taxonomy is *"scripts always log and never halt"*
      - **Misdirection, never theft**: the true lines are kept, so `purge` is a
        repair rather than a report
      **See it:** ✅ `ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend;
      hold; hold; hold; hold; verify; meditate 60"` — *"something got past the
      wall"*, then the audit names the spell it touched
- [x] **The cadence, and it is the single most load-bearing number in the
      domain.** §11.5 puts siege provocation at *"every 20–30 min"*; until §5.3's
      trace provokes them, `siege::CADENCE` stands in for it
      - **`orbs-balance` found this and nothing else would have.** Back to back,
        a driver read **4.70 experience a tick** against clarity's 0.140 —
        thirty-three times the flagship, which says *ignore every other room*
      - **The escrow was not the thing to tune**, and that was the first
        diagnosis. A siege paying 105 for thirteen rounds is right; fighting
        three hundred of them in two hours is not
      **See it:** ✅ `cargo run -p orbs-balance -- run besieging --ticks 7200`
      reads 0.114–0.134 across seeds, under clarity and near warding
      - **This line said 0.043–0.085 for two versions and that was never the
        game**: it was measured while the driver keyed *"have I already spent
        this round"* on the byte length of a rendered prose line, so consecutive
        rounds collided and the policy stopped using its arsenal at random
- [x] **Quintessence — a pool spent to pledge dice** (`0.8.15`). §11.5's mana,
      built at last: a fixed pool granted on `defend`, sized by the tower's
      integrity and raised by the ley line, **and never regenerating** — §14's
      rule, since a per-tick regen would mean *more typing produces more* for
      exactly the players the screen-reader mode serves
      - **The decision now bites, and that is the measurement rather than the
        claim.** Two solvers with opposite allocation strategies used to tie
        across seventeen seeds; they now differ on 2 of 5, and where they differ
        the gap is nearly threefold
      - **Declining to pledge costs nothing**, so leaving an area dark is a move
      - **The coffer publishes only what it can pay for**, which is why neither
        shipped solver needed a line changed — both already asked `if the coffer
        has d20`. The numeric comparison exists for real weighing and is not
        load-bearing, because `has more … than` is *strict* and would refuse the
        die you can exactly afford
      - **`aim` closes an asymmetry**: `Roll::chance` was drawn on the board and
        askable by nobody, so a hand player could see the odds and a spell could
        not
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; pledge d20
      buckler; pledge d8 line; pledge d6 succour; survey coffer"` — three pledges
      take 5, 2 and 1 of 24, and the board's coffer row prints what each costs
      beside what is left. Run three rounds of it and the fourth is refused:
      *"d20 would take 5, and you hold 0"*
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend sanctum; meditate 3600; attend
      bailey; defend; survey coffer"` — a worn tower opens with **12 against a
      kept tower's 24**, which is §19's deferred integrity → siege coupling
- [x] **The far side of a comparison grows an arithmetic** (`0.8.16`). Reverses
      §19 twice — *"it is not getting arithmetic"* and *"an expression tree: no"*
      — and both are struck through with a pointer rather than contradicted
      - **Quintessence is what broke them.** A derived word answers a *fixed*
        ratio (`outnumbered`), and there is no word to publish for *is what I
        hold more than what this costs*: the answer depends on two quantities
        the player is choosing between
      - `[double] <place> [has <thing>] [plus n]` — **words never symbols, one
        operator, no precedence table, no brackets.** Subtraction is absent
        because `A − n > B` is `A > B + n`
      - **`strict` is derived from the grammar, not the variant**, or `than the
        d20` and `than the d20 has quintessence` disagree at equality and
        `plus 0` changes a sentence's meaning
      - **`plus` joins `STOPPERS`**, which is a permanent reservation: nothing in
        the tower may ever be named it
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=110x40 ORBS_DUMP="attend bailey;
      scribe maths"` with `ORBS_EDIT` writing the three forms, then `interpret` —
      each reads back verbatim, which is the surface a swallowed term shows up on
      **See it:** ✅ `ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend bailey;
      recall scripting"` — the manual gains a sixth question shape
- [x] **`sparingly`, the solver the arithmetic was built for** (`0.8.16`). The
      only shipped spell that uses `for each die`, a different reading on the far
      side, or `double` — and the only one that declines to pledge on purpose
      - **Its guard is a double negative and has to be.** `not … fewer … than` is
        the language's own route to *at least as many*; the affirmative is
        *wrong*, because a comparison against a place is strict and would refuse
        the die you can exactly afford
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; debug_spawn troop 6;
      debug_spawn warding 6" ORBS_THEN="invoke sparingly; meditate 600; sift
      pledge bailey.log"` — **nine pledges, three rounds, exactly 24 spent**, and
      then it fights on without dice rather than being refused
- [x] **An `xhigh` review of the whole siege, and the two it found by playing**
      (`0.8.17`). Eleven findings; **seven were a comment, a doc or a count that
      had stopped being true**, and the two that were not working were both found
      by typing a command into the game
      - **The enemy was breaking spells rather than lying to them.** The
        corruptible guard let through any line whose last word is grammar, so
        `part look()` produced three complaints and `is idle-` skipped a whole
        block — the surface announcing itself, which is the opposite of the
        misdirection `assault.rs`'s own header requires
      - **A die's price did not exist until the first bailey verb**, so the
        affordability guard every solver ships answered *yes* on a tower with no
        pool. The sanctum's `integrity` defect, one domain over
      **See it:** ✅ `ORBS_BOOT=0 ORBS_DUMP="attend bailey; survey d20"` — the
      price is 5 with no siege ever opened, where it read *"the d20 holds
      nothing"*
      **See it:** ✅ six rounds on seeds 3, 11, 17 and 42 print **no** `line n:`
      complaint, where a corrupted `part` or `is <state>` printed three
- [x] **Scripting the siege, tested as its own question** (`0.8.18`).
      `tests/scripting_the_siege.rs` — eleven claims that a *person* can write a
      siege spell, which neither the game tests nor the solver tests asked
      - **Every declared reading is reachable.** `scene_at` registers all
        eighteen unconditionally, so a word nothing publishes still compiles and
        reads nought for ever. It had shipped twice; the lint **failed on its
        first run**, which is how you know it is not vacuous
      - **The arithmetic against fixed numbers** — the pool opens at 24 and a
        `d20` costs 5, so `plus`, `double` and the comparison have known answers.
        Tree shape, chained-`plus` refusal, `plus 0` identity and `u32::MAX`
        saturation are each pinned, and none is visible to a round-trip
      - **A spell can tell a win from a loss** — `lifted` says only *over*, and
        which band is `routed` says which way. Both directions asserted
      **See it:** ✅ `cargo test -p orbs-sim --test scripting_the_siege` — eleven
      claims; **42 test binaries** in the workspace now
- [→] **Pane synergies — deferred to Phase 12, not cut.** §18 item 6 and §9's
      *"an open brewing pane feeds potions to the siege automatically"*. It is an
      *enhancement* to a siege that already works, and it is the one item here
      that genuinely wants multiplexing, so it waits for the phase that builds it
- [→] **Traits, `lens/observed/`, and the eldritch renderer.** Procedural trait
      composition (§5.2) and the renderer are plumbed in `orbs-render` with zero
      producers; both are Phase 13-scale content work rather than siege mechanics,
      and the siege is tense without them
- [→] **Unattended-siege backlog, dispersal and decay**, and difficulty tiers.
      All three depend on **provocation** — §5.3's trace deciding *when* a siege
      arrives — which is unbuilt. A backlog of sieges nobody provoked has nothing
      to accumulate, so this waits with the thing it counts

**Scarcity: the arsenal, finally spent.** Every domain has been producing into a
store nothing drew from; this is the phase where six rooms' output becomes a
decision under known risk.

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

- [ ] Bevy version window, isolated from new-system work
      **See it:** the game boots, draws, and plays identically on the new version
- [ ] Green on all three platforms
      **See it:** launch the built artifact on each, not just CI green

---

## Onboarding + demo

*Was **Phase 14**, and much of the file defers to it by that name.*

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
      they are a convenience over `survey`, never the only carrier — but a
      settings screen to reach any of this without an environment variable is
      Phase 15's, and until it lands there is **no in-game control** for the
      accommodation.

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
