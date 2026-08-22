# O.R.B.S. — Operational Relic Bewitching System

**Blackhearth Studios — Game Design Document**
Status: draft 8 · 2026-08-16 · **in production — Phases 0, 0.5 and 1 closed**

Drafts 2, 4, and 6 each incorporate an independent staff-level design review.
Draft 6 settled the tick model as duration-based, removed the offline clamp, gave
sieges a completion economy, separated the eldritch and sabotage signal
vocabularies, and added three provocation sources with a composition cap.

Draft 7 completed the economy session (§11.5) — durations, panes, attention, mana,
escrow, and unattended-siege pressure — and separated domain panes from multiplex
capacity (§9).

**Draft 8 incorporates a fourth review of that economy**, fixing the Focus-slot
reservation rule, the unattended-siege exploit, the mana/duration denomination of
the automation advantage, and the layout model.

---

## 1. Pitch

A wizard sits in his tower and stares into his scrying orb. Inside the orb is a
computer terminal.

O.R.B.S. is a text-only game played entirely through a fantasy command line. You
tend a wizard's tower by navigating a filesystem that *is* your duties — brewing
in `/laboratory`, warding in `/battlements`, spying in `/lens` — and you progress by
writing scripts that let the orb perform your work without you. When you are
ready, you descend into a siege, where an intelligent enemy attacks the
automation you built and you must diagnose and repair it under pressure.

Artless by design: no sprites, no characters, no illustrations. A single curved
CRT glowing in the dark.

## 2. Design pillars

1. **The terminal is the fantasy, not a UI.** Every mechanic should feel like
   both operating a computer and practising magic.
2. **Understanding beats memorisation.** The parser forgives phrasing. Mastery is
   knowing *what to do*, not recalling a flag.
3. **Automation is progression — and automation is attack surface.**
4. **Urgency is consensual.** The idle layer is safe; sieges are entered
   deliberately.
5. **Everything converges in the siege.**

## 3. Tone and voice

A **dynamic register driven by world state**, not a fixed style.

| Register | When | Character |
|---|---|---|
| **Earnest arcane** | Default, ~80% of text | Warm, mythic, sincere. |
| **Sysadmin comedy** | Easter eggs, deprecated commands, obscure corners | Old software with old opinions. Discovered, never pushed. |
| **Eldritch** | Threat high, siege collapsing, forbidden directories | Clipped, cold, wrong. |

Rules:
- Registers shift on a measurable world value, never arbitrarily.
- Comedy is **opt-in through curiosity** and must never puncture a tense moment.
- **Eldritch signals through structure, never colour** — palette-independent (the
  base hue is a player option) and colourblind-safe.
- **The eldritch and sabotage signal vocabularies are disjoint, by rule.** Eldritch
  fires at high threat; sabotage tells (§8.1) must be spotted at high threat. If
  both used glyph corruption and broken alignment, the tonal system would jam the
  diagnostic system at peak difficulty.

  | System | Signals |
  |---|---|
  | **Eldritch** | Pacing, diction, silence, a prompt that answers, delayed or unattributed echo |
  | **Sabotage** | Spacing, alignment drift, glyph substitution, malformed records |

  Draft 6 reserved *"unrequested output"* and *"missing prompts"* to eldritch.
  Duration-actions make asynchronous unrequested output the **normal** case — every
  completed brew and ward emits one, from up to 25 concurrent sources at endgame —
  and §9 mandates a permanent input line, so a missing prompt cannot be a tell
  without breaking a layout invariant. Both signals are retired.

- **Routine completions route to their own pane; the main line is reserved for
  eldritch.** This is what keeps the register legible now that async output is
  ordinary.

- **Script listings, schedule listings, and log output are corruption-exempt
  surfaces.** The renderer never applies eldritch presentation to them. These are
  the diagnostic surfaces; they must always be trustworthy as *renderings*, even
  when their contents are not.
- **Eldritch output is a normal, logged message carrying a presentation flag.**
  The renderer corrupts it; the model records it faithfully. Preserves
  reproducibility (§6) and linearisation (§14). Unlogged output is forbidden.
- **Every eldritch message carries an authored linearised variant**, achieving
  the effect through diction, cadence, and stage direction rather than glyph
  damage. Without this, screen-reader players lose an entire tonal register.

## 4. The screen

**Framing: always inside.** The player never sees the orb, the wizard, or the
tower. CRT curvature and glow imply the sphere. Absolute — it is what makes an
artless game read as intentional rather than cheap.

### Palette

**Curated phosphor themes, selectable in settings** — each a hand-tuned harmony,
not a hue slider.

- **Default: amber.** Warm, classic, and the tube a scrying orb ought to be.
  Muted violet was the original default and remains an option — see §19; it is
  the one theme nobody mistakes for a real terminal, which is a reason to keep it
  and a reason not to open on it.
- *(Bevy frontend only — the terminal frontend inherits the user's terminal theme
  via indexed ANSI, §13.)*
- **Amber** — warm, classic, second default.
- **Green** — canonical terminal.
- All cosmetic; none unlockable (art direction should not be gated behind the
  economy).

Base hue carries all ordinary text through **intensity variation alone**, with a
small accent set reserved strictly for meaning.

| Accent | Reserved for |
|---|---|
| Danger accent | Failure, breach, hostile action |
| Cost accent | Mana and arcane expenditure |
| Success accent | Completion |

Accents are never decorative, **colour never carries meaning alone** (§14), and
each theme defines its own accent triad so contrast holds against its base.

### CRT treatment

Ported from `court_wizard`'s `crt_effect.wgsl`: barrel distortion, scanlines, RGB
subpixel grid, vignette, chromatic aberration, flicker, rounded corners, phosphor
glow, desaturation, screen flash, viewport letterboxing. Effects wire to game
state — vignette pulses as threat rises, flicker on strain, flash on breach.

**A Phase 0 task with real cost — 2,638 lines of surrounding Rust across 15 files
plus a 242-line shader** (of which ~530 lines are the separately-credited
accessibility pipeline):

- **Cross-version.** court_wizard pins Bevy 0.18.1; O.R.B.S. targets 0.19. The
  WGSL ports; the render-graph Rust is Bevy's most volatile surface.
- **Unproven over text.** Barrel distortion + RGB subpixel mask + scanlines over
  1px glyph stems is a moiré and legibility hazard. **Legibility is the product,
  not an aesthetic.**
- **The Phase 0 legibility test must use the genuine worst case**, not a calm
  prompt and not default effect levels: **tier 2 fidelity at the minimum supported
  window**, four panes, a siege in progress, dense log output, **peak-threat CRT
  state (maximum flicker and vignette pulse), and eldritch presentation active**,
  while requiring the tester to spot a single-character sabotage tell. Three
  systems degrade legibility simultaneously at peak threat and the test must
  exercise all three at once.

  It must also establish **the minimum window the picture is legible on**.
  *(Superseded, §19: this read "the minimum window at which tier 2 is offered at
  all" while the grid followed the window. There are no tiers; the number is now
  `MIN_SCALE` and is derived by `screens.rs::minimum_window`, which puts it at
  960×720.)*
- It hardcodes a 16:9 letterbox and a 1080 scanline reference.
  *(Half-superseded, §19: the letterbox is now ours and deliberate, at 4:3. The
  criticism that stands is the hardcoding — the scanline reference re-derives
  against the live cell size.)*

Already built in court_wizard and creditable to §14: `colorblind_correction.wgsl`,
`high_contrast.wgsl`, and an accessibility render pipeline.

### Grid model

**A fixed 120×45 grid in a 4:3 picture, scaled continuously to the window.**
*(§19, superseding the reflow model below.)* The cell is 8×16, so 4:3 forces
`cols : rows = 8 : 3` and the picture is 960×720 at native size. A window decides
only how big a cell is — `min(W/960, H/720)`, letterboxed on the other axis by
`ScalingMode::AutoMin` — so a resize moves no pane, no border and no sentence.
720 divides 720, 1080, 1440 and 2160, so the common display heights are ×1.0,
×1.5, ×2.0 and ×3.0. All layout is still authored against the 80×22 floor, which
survives as the `ORBS_GRID` check.

*The superseded model, kept because §9 and the phase history are written against
it:* **reflow at integer fidelity tiers, with a declared minimum of 80×22.** Cell
size was an integer multiple of the 8×16 bitmap cell and the multiplier was
chosen so tier 1 landed near 80×22 on any common window; engaging multiplexing
dropped it a step, roughly doubling available cells (§9).

Its stated justification was that *"fixed scaling without reflow would make large
accessibility font scales unreadable; this model keeps the floor legible while
letting the orb resolve more detail on demand."* **That cost is real and is now
owed.** Fixing the grid removes the only text-size control the game had, and §19
records a font-scale setting — a second authored grid on the 8:3 line, 80×30 —
as the affordance that replaces it. Until that ships, the game has one text size
per window size and no way to ask for another.

**Text renderer.** New work — court_wizard's `bevy_text`/`bevy_ui` TTF path will
not hold a full grid redrawing under a real-time siege. Single mesh with glyph
atlas and instanced quads, or a custom material driven by a cell-index texture.
Phase 0, critical path.

**Font: CP437-style bitmap at 8×16.** Box-drawing glyphs are native to the
codepage, which matters because every pane border, table, and progress bar in the
game is drawn with them. The 1:2 cell aspect reads well for long prose — the bulk
of this game's content — and is instantly legible as "terminal". Custom glyph
variants within the codepage give sabotage tells (§8.1) a place to live.

### Boot

**The boot sequence is a status report** reflecting real world state — damaged
systems fail to initialise, undiscovered domains show `not found`, corruption
appears as failed checks. **Skip is a sticky setting, not a per-launch keypress**,
and the same content is available any time via `status`.

```
O.R.B.S. v0.9.3  —  cold start

scrying lens ................ [ ok ]
ley-line uplink ............. [ ok ]
grimoire index .............. [ 2841 ]
laboratory ..................... [ ok ]
battlements ................. [ DEGRADED ]
  east_wall integrity 34%
menagerie ................... [ not found ]
archive ..................... [ ok ]

bound:
  night_watch    dusk      [ ok ]
  purge_cycle    hourly    [ DRIFTED ]

3 warnings. the orb warms to your touch.

orbs:~$ _
```

Listing bound scripts here is not decoration — it is the ambient defence against
forgetting what is running unattended (§8.1).

## 5. Time and the world clock

**One canonical clock. Every system reads from it.**

### 5.0 Tick semantics — ticks are world time, not currency

A **tick** is the atomic unit of world time: the clock for drift, upkeep,
triggers, timers, aberrations, and accrual. It is **not a resource**. There is no
tick pool, nothing to run out of, and no energy meter.

- **Issuing a command is free and instant.** No tick cost, no resource cost. A
  fast typist gains nothing over a slow one — which is what §14 and §17 require,
  and what a per-command tick charge would have quietly violated.
- **A failed or ambiguous parse changes nothing at all.**
- **The tower clock advances on wall-clock while the window is open** at 1 tick
  per second (§11.5) — **always, with no pause and no grace.** Pausing to think
  is not punished because **drift and decay rates are slow *per tick***; the
  clock itself is not. That is the whole protection, and it is sufficient.

  An inactivity grace was specified here, built, and **removed** — it was a
  second mechanism for a problem the slow rates already solve, and it cost an
  exploit surface, a special case, and a clock the player could not predict
  (§19).
- **`wait` / `meditate N` fast-forwards** the clock. It is not a time source.

### Scarcity comes from duration, not from currency

Issuing an action is free; **actions take time to complete.** Brewing, warding,
repairing, deciphering, and summoning all occupy their pane for a duration
measured in ticks.

**Concurrency is therefore the real scarcity:**
how many duration-actions you can have in flight at once. That is precisely what
focus panes gate (§9), which means the economy and the focus system are the same
system rather than two bolted together.

This gives every downstream claim its teeth without a currency:

- **Nuisances have real decision content.** Repairing the rats occupies the
  laboratory pane for its duration, during which you are not brewing. "Handle when
  convenient" is a genuine trade-off.
- **Automation wins** because script actions occupy Concentration rather than
  Focus, and because of script-only capabilities and the speed advantage (§8).
  It wins *nothing at all* until the first concentration level is bought, which
  is what makes that upgrade the game's turn rather than a step on a curve.
- **Multiplexing is valuable** because in-flight manual actions reserve their
  Focus slot (§9, invariant 4), so capacity — not pane count — is what caps how
  much you can have going by hand.
- **No spiral.** Repairs advance the clock only as much as any other action, and
  aberration arrival is capped (§5.3).

Durations, arrival rates, and the concurrency curve are specified in §11.5. The
model and its first-pass numbers are settled; the balance harness (§13) sweeps
them in Phase 1.

### Phase A — The Tower (idle / calm)

No forced threat. The player navigates domains, performs duties, researches
discoveries, writes and binds scripts, handles nuisance aberrations, and prepares
for the next siege.

### Offline progression — unlocked, not default

**Initially there is no offline progression.** The tower ticks only while the
window is open. This makes quitting strictly neutral rather than optimal, removes
the exploit surface entirely, and means the hardest-to-balance system does not
exist while the player is still learning.

**Offline accrual is an unlockable capability** — the orb learns to work
unobserved. Once unlocked, bound scripts (and only bound scripts) execute while
the game is closed. The offline economy becomes a *reward* for progression rather
than a baseline assumption, arriving when the player already understands what
their scripts do.

Once unlocked:

- **Bounded accrual window**, extendable by further progression.
- **Full-granularity deterministic simulation** of the bound schedule on return,
  using the same code path as online. At 1 Hz an 8-hour window is 28,800 `step()`
  calls on a small headless sim — milliseconds. Draft 6's "coarse-tick" was
  self-contradictory (a different granularity *is* a different code path) and, at
  this tick rate, unnecessary.
- **Offline production accrues trace**, but offline-provoked sieges are capped at
  one per accrual window and are exempt from backlog escalation (§11.5).
  Otherwise: accruing no trace offline would make offline production strictly
  better than online, re-creating the quit-is-optimal inversion; while accruing it
  unbounded would mean returning from a night away to a suspended-offline backlog,
  punishing absence.
- **One narrow floor: offline execution cannot reduce Integrity below a set
  fraction of its departure value.** Integrity is the only resource whose loss
  reads as punishment for absence.

  Draft 5's broader clamp — "never net-negative in any resource" — is **removed**.
  It duplicated the failure taxonomy (§8 already skips and logs unaffordable
  instructions), it would have let players conjure resources by binding a
  converter and logging off, and it made closing the game strictly better than
  leaving it open. That is the same inversion this section exists to prevent.
- **Drift and aberrations are damped identically whether the window is open and
  idle or closed.** Damping only the offline path would re-create the inversion in
  a different place.
- **The "while you were away" summary is a log file** the player can `cat` and
  `grep` — on-theme, and the primary script-debugging surface.

### Phase B — The Siege (roguelite)

Entered deliberately. Real-time, with genuine failure. Bound scripts execute as an
autobattler; the player diagnoses and repairs under pressure.

- **Pull:** the only source of certain reagents, fragments, and trait knowledge.
  Idle play plateaus without them.
- **Rewards escrow during the siege and settle on exit.** Earnings accrue into
  escrow as they occur. How much you keep depends on how you leave:

  **The kept fraction scales with how far you got**, floored at 20% (§11.5), so
  bailing early after farming a cheap opening is punished specifically while a run
  that fails at 90% still pays nearly in full.

  Draft 5's continuous banking made "enter, farm the easy opening, quit" the
  optimal loop, because there was no loss to dodge. Escrow closes it: bailing
  early is never *negative*, just **unsatisfying**. Meta-progression comes
  markedly faster through success than through losses or abandons.

  A crash is treated as an abandon, so a power cut costs the bonus and a share of
  escrow — a real cost, never a catastrophe.

- **Declining a siege is always allowed**, but unattended sieges compound and
  eventually suspend offline progression (§11.5).
- **The tower is never harmed.** Experimentation remains free.
- **Failure condition varies by siege type** — survival timers, objective
  defence, integrity collapse, cascading systems failure.

### 5.1 Aberrant events — the core of manual play

The model is **incident response**, not combat. You built a reliable system;
reality periodically violates its assumptions; you diagnose and repair.

| | **Nuisance (idle)** | **Adversarial (siege)** |
|---|---|---|
| Source | Environmental, random | An intelligent enemy |
| Example | Rats infest the laboratory; byproduct accumulates; a reagent spoils | Reagents swapped, triggers retimed, glyphs corrupted, logs poisoned |
| Effect | Slows production | Compounds toward failure |
| Surfaces touched | Environmental only — never scripts, schedules, or logs | All four surfaces (§8.1) |
| Response | Repair occupies a pane for a duration | Diagnose and repair under pressure |
| Ignoring | Reduced throughput | Loss |

**Nuisances have real decision content because repairs occupy a pane.** Fixing the
rats blocks the laboratory for its duration, so it trades directly against brewing.
"Handle when convenient" is a genuine prioritisation choice, not a chore list.

**Adversarial aberrations are siege-only.** Trace (§5.3) raises the *rate* of
ordinary environmental nuisances outside a siege, but the enemy never touches
scripts, schedules, or logs in the calm layer. Phase A stays genuinely safe, which
pillar 4 requires, and the four-surface sabotage model stays the siege's
signature.

**In sieges, the enemy attacks your automation.** Your scripts are simultaneously
your greatest strength and your attack surface. The intervention loop is therefore
**debugging**, which makes the scrying domain load-bearing rather than flavour.

**Triage bandwidth, not typing speed — mechanised:** *identifying* any aberration
costs exactly **one command**, never a sequence (§8.1). The binding constraint is
therefore *which surface do I inspect first*, not how fast anyone types. Repair
may take more than one command; diagnosis never does.

**Cost model: issuing commands is free; repairs consume resources and occupy a
pane for their duration.** Responding to sabotage you did not cause is never
punished beyond the damage already done and the attention it steals.

### 5.2 Threat model — procedural trait composition

Threats are assembled from modular traits: resistances, behaviours, arrival
patterns, sabotage methods, timings. Programmable as both threat and counter,
combinatorially deep, directly controllable for difficulty tuning.

**Traits are unknown at first and learned through encounter**, creating a
knowledge-progression layer independent of mechanical progression. Over many
sieges the player builds a model of the trait space and writes defensively rather
than reactively. *The player levels up; the character merely unlocks.*

Learned traits are **recorded in-world** in `lens/observed/` once successfully
identified in the field. The skill is recognising a trait live from partial
evidence; having done so once, the player keeps the note. This removes a memory
tax that would otherwise punish the long breaks idle play invites, and it is
exactly what a game about an orb that writes things down should do.

### 5.3 Trace — how sieges get provoked

**The connective tissue between the calm layer and the roguelite layer.**

Sieges are opt-in, which leaves them motivated only by *pull* (exclusive
resources). **Trace** supplies the missing mechanism:

1. Player activity accrues **trace** — the realm notices you.
2. Trace raises the **rate of environmental nuisances** against your tower. The
   enemy has your scent; it does not yet have your scripts.
3. Sufficient trace **provokes a siege**, which is where progression lives.

The player therefore **provokes their own sieges by doing things they wanted to do
anyway.** Urgency remains consensual — you chose the activity, and you chose how
hard to push — but it arrives organically rather than through a menu button.

### Three trace sources

| Source | Available from | Survives the cut line? |
|---|---|---|
| **Production heat** — the more you brew, ward, and summon, the more the realm notices | Phase 1 | Yes |
| **Scrying depth** — looking hard into hostile feeds draws attention | Phase 0 (scrying is in the slice) | Yes |
| **Infiltration** — connecting to enemy hosts, scaling with time and interference | Phase 9 | **No — cut-line item 3** |

Draft 5 rested this entire loop on infiltration alone, which was both a phase
late (sieges land in Phase 8) and a cut-line dependency. Production heat and
scrying depth are always available and survive every cut; **infiltration becomes
the high-yield amplifier rather than the sole carrier.**

Trace is a **player-controlled difficulty dial**: push for more progression, more
pressure, more frequent sieges. A player who wants a quiet tower produces less,
scries shallowly, and stops connecting — see the plateau caveat in §11.

### Nuisance rate — composition and cap

Nuisance arrival is multiplied by trace (three sources), open pane count (§9), and
accumulated drift (§8). **These compose multiplicatively against a hard ceiling**,
so a player who infiltrates aggressively while multiplexing cannot be swamped by
the product of five curves. The ceiling is a first-class balance constant, and the
composition rule is validated in the balance harness (§13) before any of it ships.

### The loop

*Automate in calm → produce, scry, infiltrate → draw attention → the siege comes →
the enemy attacks the automation → diagnose, repair, harden → automate deeper.*

## 6. The parser — intent resolution

**The most important system in the game.**

### Approach: deterministic NLU

No LLM, local or remote. Intent resolved by a hand-authored pipeline over the
live world model:

1. **Normalise** — lowercase, strip filler.
2. **Fuzzy match** against known vocabulary and *entities that currently exist*.
3. **Resolve against world state.**
4. **Score** candidates; accept a clear winner.
5. **Disambiguate** with a numbered prompt on ties.
6. **Suggest** if nothing scores — never a bare error.

Instant, offline, deterministic, unit-testable, tiny. Its limitation — only
anticipated phrasings resolve — is acceptable because vocabulary is authored and
finite, and disambiguation degrades gracefully.

### Mastery arc: echo, with optional strict mode

Loose input accepted; the canonical command **always echoed back**. Players absorb
syntax by osmosis and graduate to typing it directly. An optional
"arcane/strict" mode requires precision.

```
orbs:~$ go to the castle gates
  → cd /tower/battlements/gates

orbs:/tower/battlements/gates$ start potion
  → brew --recipe=?
  ambiguous. did you mean:
    1) brew --recipe=clarity
    2) brew --recipe=warding
  [1/2]: _
```

**Echo verbosity is a setting (default on)**, tagged as metadata in the linearised
stream so screen readers do not speak every command twice.

### Three registers in, one register out

Every canonical command carries synonyms across **three registers**, all of which
resolve:

| Register | Example input | Serves |
|---|---|---|
| **Shell** | `grep march feed.log` | Terminal natives — existing muscle memory works |
| **Arcane** | `sift march feed.log` | Players who want the fantasy |
| **Plain English** | `search for march in the feed` | Newcomers guessing |

This is nearly free: the parser is synonym-based already, and man pages are
written per *canonical* command, so the synonym layer costs vocabulary entries
rather than prose.

**The canonical form — what the echo shows — is arcane.** Because echo is the
teaching mechanism, whichever register is canonical is the one players absorb.
Making it arcane means the mastery arc is literally learning to speak as a wizard:
you type "grind the sage", the orb answers `wield mortar_and_pestle`, and months
later you are typing `wield` by reflex. The interface teaches magic.

```
orbs:~$ grind the sage
  → wield mortar_and_pestle

orbs:~$ grep march feed.log
  → sift march feed.log
```

**"make a potion of clarity" resolves to the recipe, not to a potion** (§10.1,
§19). There is no single command that brews one — brewing is a pipeline you run,
or a script you wrote. So the newcomer's most natural sentence answers the
question they actually asked:

```
orbs:~$ make a potion of clarity
  → grimoire clarity
```

That is deliberately the *tutorial entry point* rather than a refusal. §15 chose
brewing as the parser-gate subject because *"a shell-naive tester immediately
understands 'make a potion'"* — which requires the phrase to resolve somewhere
useful, not that a verb exist to satisfy it.

**The canonical command set is player-facing API.** Naming is a design task
scheduled before the Phase 1 vocabulary freeze.

### 6.1 Naming — the slice vocabulary

**Canonical names must be short.** Because players graduate to typing the
canonical form (§6), the canonical name is what *expert* players type all day. A
long arcane name makes mastery slower than novicehood — `auspicate --sign=march`
would lose to `grep march` every time — which quietly punishes the exact progression the echo
mechanism exists to create, and edges toward the typing burden §17 disclaims.

**Rule: canonical verbs are one short word, ideally ≤7 characters.** Flavour lives
in the prose, not in the length of things you type a thousand times.

The Phase 0 vocabulary (16 commands), canonical arcane with synonym registers:

| Canonical | Does | Shell synonyms | Plain synonyms |
|---|---|---|---|
| `attend <path>` | Move to a place | `cd` | go, enter, "go to" |
| `survey [path]` | List what is here | `ls`, `dir` | look, list, "what's here" |
| `peruse <file>` | Read a file | `cat`, `less` | read, open, show |
| `sift <pat> <src>` | Filter for matches | `grep` | search, filter, "look for" |
| `status` | Tower overview (the boot report) | — | overview, "how are things" |
| `recall <topic>` | In-world manual | `man`, `help`, `?` | explain, "how do I" |
| `meditate <n>` | Fast-forward the clock | `sleep` | rest, pass — **not `wait`, see §19** |
| `verify <target>` | Detect tampering | `check` | inspect, audit |
| `undo` | Revert the last command | — | revert, "take it back" |
| `meditate <n>` | Fast-forward the clock | `wait`, `sleep` | rest, pass |
| `decoct <essence>` | Brew a potion — **retired in Phase 1, see below** | — | brew, make, mix, distil |
| `empty <place>` | Turn a tool out into the dispensary | — | unload, collect, decant, pour |
| `purge <target>` | Destroy waste or spoilage | `rm` | clean, dump, "get rid of" |
| `research` | Open a way into the stacks | — | divine, decipher, study, translate |
| `scribe <name>` | Open a spell in the editor, making it if new | `vi`, `edit` | inscribe, author |
| `bind <script>` | Attach a script to a trigger | `cron` | schedule, automate |
| `invoke <script>` | Run a script or spell | `run`, `exec`, `./` | cast, do |

Every row resolves from all three registers; the echo always shows column one.

**Three canonical names changed in the Phase 0 naming pass** (§19) —
`decant`→`siphon`, `decipher`→`research`, `inscribe`→`scribe`. Every original stays
in the table as a plain-English synonym, so nothing a player learned stops
working. That is not courtesy: a released word does not stop resolving, it
resolves to whatever it is nearest, and `decant` unclaimed lands on `decoct`.

**`siphon` was retired outright in Phase 1** (§19) and `empty` inherited its
words. Not a rename: the verb had nothing left to do once the per-instrument
verbs could reach into an idle tool, and with it gone the laboratory bench and
the dispensary are one place. `take` was deliberately *not* inherited, which
removed a tolerated collision rather than moving it.

**A fourth changed in Phase 1: `grimoire`→`recall`** (§19), and this one was
**released rather than kept as a synonym.** The rule above protects *shipped*
vocabulary and nothing has shipped — and `grimoire` now names the player's
spellbook at `/grimoire`, so keeping it pointed at the manual would make one word
mean both the reference you read and the book you write in. Typed alone it now
reads as the directory it is.

**Phase 1 adds three and retires one, taking the vocabulary to 18.** The brewing
pipeline (§10.1, §19) needs a way to move a thing, start a tool, and cancel one:

| Canonical | Does | Shell synonyms | Plain synonyms |
|---|---|---|---|
| `move <thing> from <src> to <dst>` | Move a thing between places | `mv` | transfer, transport, relocate |
| `wield <tool>` | Start a tool working | — | use, begin, kindle |
| `stop <tool>` | Cancel a tool, refunding inputs | — | cancel, halt, damp |

**`decoct` is retired.** It meant *"brew a potion"* when brewing was one command
holding a slot for twenty ticks. It is now a pipeline — grind, digest, combine,
distil — and a verb claiming to do all four in one word would be teaching the
player something false, which is exactly what §6's echo mechanism must not do.

**The only single command that brews a potion is a script you wrote.** That is
the design's whole thesis about automation (§8), and a built-in shortcut would
have been the orb doing for free the thing the player is meant to build.

**Its words stay claimed, pointed at `grimoire`.** `decoct`, `brew`, `make`,
`mix` and `distil` all resolve to a recipe lookup, so *"make a potion of
clarity"* answers with how. This is not courtesy — the Phase 0 naming pass
established that **a released word does not stop resolving, it resolves to
whatever it is nearest**, and an unclaimed `decoct` would land on some other
verb silently. Recipes are registered as `Topic` nouns alongside their `Essence`
so the lookup has something to name.

**`move` is the first verb whose slots are not independent.** Its first slot
resolves against the *contents of its second* — `move charcoal from dispensary
to athanor` looks `charcoal` up inside `dispensary`. This is what makes `from`
load-bearing rather than decorative, and it does **not** relax §19's *"you can
only name what is where you are"* for anything else: `purge charcoal` from the
laboratory still fails, and Phase 8's pane addressing is still what relaxes the
rule generally. The two-argument short form `move <thing> <dst>` infers the
source from the current place and needs none of this.

### Disambiguation never blocks during a siege

The numbered prompt is modal, and siege ticks advance on wall-clock. A modal wait
would make ambiguous phrasing cost siege time — exactly the typing pressure §14
forbids.

**In a siege the parser takes its best-scoring candidate, executes it, echoes what
it chose, and offers correction.** Blocking disambiguation is a calm-layer
affordance only.

### Undo

The parser is designed to guess, echo arrives *after* execution, and destructive
verbs are everyday tools (§7). A confident misresolution must be recoverable.

**`undo` reverts the last command**, valid until the next command or ~30 seconds,
whichever comes first. It cancels the in-flight action and refunds its consumed
inputs.

Anchoring undo to the previous *tick* boundary would give a one-second window at
1 Hz (§11.5) — useless, since the echo arrives after execution and the player
typically notices a misresolution several seconds later, once a ten-minute brew
has already claimed their reagents. Command-anchored is the only version that
reaches the mistake it exists to catch.

Destructive commands additionally confirm when the target is not routine.

### Requirements

- Never blocks the frame. Sub-millisecond.
- Every resolution reproducible — full input, resolution, and candidate scores
  logged and exportable. Required by the Phase 0 gate (§15).
- The parser must explain itself.

## 7. The filesystem as world

The directory tree *is* the tower. Navigation is diegetic; paths are places.

```
/tower
├── battlements/     defense — walls, gates, wards
├── laboratory/         brewing — reagents, recipes, potions
├── lens/            scrying — feeds, logs, intelligence, observed/
├── grimoire/        spellcraft — spell composition
├── menagerie/       summoning — familiars and constructs
├── forge/           enchanting — imbuing items and wards
└── archive/         research — fragments, translation, discovery
```

Files represent real state. Hidden and locked branches exist and are discoverable
— where lore and the eldritch register live.

### Remote hosts as places

**The world beyond the tower consists of other machines.** An enemy camp, a rival
wizard's sanctum, a derelict waystation — each a host you `connect` to, with its
own filesystem, permissions, and defences. Reconnaissance and infiltration are
performed by navigating someone else's tree.

Thematically the strongest idea in the design — you never leave the orb, yet the
whole world is reachable — and a **significant scope addition**: a second content
type with its own trees, verbs, failure states, and discovery pacing. Scoped in
§15 and on the cut line.

**Trace risk.** Time connected and depth of interference accrue trace. Rising
trace increases the nuisance rate against your own tower and eventually provokes a
siege (§5.3). Disconnecting is always clean and always available — the tension is
"how long do I dare stay", never "am I trapped".

**Hosts do not retaliate beyond trace.** Trace *is* the whole risk model: no
counter-infiltration, no direct sabotage from a host, no hunting you home.
Retaliation would be a second adversarial system layered on one that already
works, on the phase most likely to be cut (§15), and it would put enemy action in
the calm layer — which §5.1 forbids.

### Shell verbs unlock as discoveries

Shell verbs are found and researched exactly like magic. Start with `cd`/`ls`,
discover `cat`, then `grep`, then pipes. Literacy transfer and progression from
one mechanic, and it paces the vocabulary budget.

```
orbs:~/archive$ decipher fragment_0x1f

  the sigil resolves. it is a sieve — it lets
  through only what matches, and holds back
  the rest.

  learned: grep
    grep <pattern> <file>
    ... | grep <pattern>

  the grimoire grows.
```

### Pipes: strict structure, forgiving stages

A pipeline's *structure* is exact syntax, but each *stage* is fuzzy-resolved
against the verb vocabulary and echo-corrected like any other command:
`ls | find rats` → `ls | grep rats`. This avoids a pedagogical cliff where hours
of "phrasing doesn't matter" abruptly stops being true — which is precisely where
the "parser feels frustrating" risk would fire.

**Commands emit structured records; human presentation is a view over the
record.** Pipes and `grep` operate on records, never on rendered text. This is the
only model that survives the eldritch renderer corrupting output, and it keeps
display off the compatibility surface. Binding on §13.

### Destruction is maintenance

Destructive verbs exist because they are **useful, everyday, and scriptable**.
Alchemical byproduct accumulates in `/laboratory` and must be purged manually or by a
bound cleanup script; spoiled reagents, corrupted glyphs, and dead summons need
disposal. Waste management is an idle mechanic and a nuisance-aberration source.

Catastrophic targets (tower root, live domains) are guarded in character — the orb
refuses, memorably. Destruction is a tool, not a trap.

## 8. Scripting and automation

```bash
orbs:~/grimoire$ cat night_watch.spell
#!/orbs/invoke
# runs each dusk

ward --upon north_gate --strength 3
invoke brew_clarity --qty 2
purge --byproduct --above 60%

if scry --enemy --within 2leagues; then
    alert --priority high
    ward --upon all --emergency
fi

orbs:~/grimoire$ bind night_watch.spell --to dusk
Bound. The orb will remember.
```

**`brew_clarity` is a spell the player wrote, not a command the game ships.**
Earlier drafts of this section had `brew --recipe=clarity --qty 2` here, which
quietly assumed a built-in that brews a whole potion — the exact thing §10.1's
pipeline replaced, and the exact thing the player is supposed to build. There is
no such command. `brew_clarity.spell` holds the twelve-odd lines that clear the
instruments, move the reagents, wield each in turn and siphon the result, and
`night_watch` calls it.

That makes this example better rather than poorer: it shows **scripts composing
scripts**, which is the progression beat between "I automated one chore" and "my
tower runs itself" — and it is the honest picture of why automation wins, since
every line it saves is a line the player once typed.

### Why automation beats doing it by hand

Three mechanisms, all present:

1. **Script-only capabilities.** Some actions cannot be performed manually at all
   — triggers on events you cannot sit and watch for, timing precision a human
   cannot hit, conditionals requiring continuous evaluation (`--within 2leagues`).
   Automation is *qualitatively* necessary, not merely cheaper.
2. **Manual actions occupy a pane for their duration.** Your concurrency is
   bounded, so automation multiplies a scarce thing rather than duplicating a free
   one — a script running the brew does not consume the attention you need
   elsewhere.
3. **Script speed advantage.** A script-executed action *completes faster* than the
   same action typed (invariant 3). The orb does it better than you.

   Denominated in **duration**, not mana. Mana does not bind in the tower (§11.5),
   which is precisely where automation lives — so a mana discount would have had
   teeth nowhere at all.

Together these guarantee automation dominates for a present player — which pillar
3 requires and which draft 3 failed to deliver.

### Cast-time canonicalisation

**Loose phrasing is resolved to canonical commands when a spell is cast, and the
resolved form lives in the program rather than in the file.** A script executes
later, in a different world state, where live-state disambiguation is
unavailable — so the names have to be fixed at a moment the player is present
for, and `invoke` is that moment. Loose phrasing remains a live-prompt
affordance; the diegetic beat, *the orb writes down what you meant*, is what
`interpret` shows in the editor.

**This is amended from bind-time**, which stored the canonical form *in the
spell*. That worked whenever the orb understood a whole line and destroyed the
player's words whenever it understood only part of one — four times, each fixed
by another special case in the rewriter (§19). The file is now exactly what was
typed, and nothing about the requirement above was given up: the program has
always been *"derived, never stored"*, so the reading simply moved into it.

**Referent identity: bound references resolve by stable entity ID, not by name or
path.** If `north_gate` is destroyed and rebuilt, the rebuilt gate is a new entity
and the script reports `Referent missing` rather than silently acting on a
replacement. This is what makes substitution sabotage detectable rather than
invisible.

### Execution model

- **Execution budget — "the orb's attention."** Instructions per tick per script
  are bounded. Solves runaway loops, gives a first-class balance lever, and is
  thematically exact.
- **Trigger contention** resolves by declared priority; ties by bind order.
- **Resource contention** resolves in execution order; starved instructions log
  and degrade per the taxonomy.
- **Scripts may not bind other scripts.** Scripts *may* `invoke` another script,
  with a **call-depth limit of 3**. The Concentration pool alone is not a
  sufficient recursion guard: exhausting it makes every subsequent instruction
  `Budget starved`, which logs at high verbosity only, so all automation would
  stop silently. Depth-limiting makes runaway recursion a loud, diagnosable
  failure.
- **A script action and a manual action may target the same domain
  simultaneously** — they occupy different resources (Concentration vs Focus, §9).
  They contend only for reagents and mana, resolved under the resource-contention
  rule above, with the manual action taking precedence: the player's own hand
  wins over the orb's.
- **Concentration exhaustion is a visible condition**, surfaced in `status` and in
  the sidebar — never a quiet log line. At concentration 0 it is not an exhaustion
  at all but the **starting state**, and `bind` says so in those terms: the orb
  cannot hold a spell for you yet.
- **Instruction dispatch is atomic within a tick; saves are permitted only at tick
  boundaries.** Autosave runs every N ticks and on significant events, not every
  second.

  Draft 6 claimed this "eliminates an entire class of save-corruption bug." With
  duration-actions that is no longer true and the doc should not bank a benefit it
  does not get: a brew can be 47% complete when the player saves. **In-flight
  duration-actions are first-class serialisable entities** with start and
  completion ticks. The bug class is relocated to a well-defined place, not
  removed.

### Authoring-time validation

`bind` always succeeds. When a script references a locked capability or a path
that no longer exists, the orb **warns and proposes the correction** in voice —
"north_gate no longer stands; did you mean north_barbican?" — and flags the script
for review. Never blocking preserves the parser's forgiving philosophy, and the
flag folds into the same script-hygiene loop as drift and sabotage.

### External editing and hot-reload

`.spell` files edited in the player's own editor **hot-reload**. Players keep their
spellbook in vim or VS Code, which makes the filesystem premise literally true and
speaks directly to the Zachtronics half of the audience (§15).

**Files store canonical commands with ID annotations** — `ward --upon
north_gate#7f2a`. This keeps the file human-readable and editable while anchoring
each reference to a stable entity.

**On reload, only lines the player actually changed are re-resolved by name;
untouched lines stay bound by ID.** Without that rule, opening your spellbook and
saving it would silently rebind every reference to current world state — which
would **launder sabotage**, quietly re-pointing a script at an entity the enemy
substituted and ensuring `Referent missing` never fires.

**Reloads queue to the next tick boundary**, the same rule that governs saves, so a
file cannot change under a script mid-execution.

### Failure taxonomy — "scripts always log and never halt"

| Failure | Cause | Behaviour |
|---|---|---|
| **Degraded** | Numeric drift (frost resist vs frost volley) | Executes at reduced effect. Logs a notice. |
| **Referent missing** | Target destroyed, moved, or substituted | Skips, logs a warning, continues. |
| **Resource insufficient** | Mana/reagents exhausted | Skips, logs, continues. |
| **Budget starved** | The orb's attention exhausted this tick | Defers to the next tick, logs at high verbosity only. |
| **Sabotaged** | Enemy tampering (siege only) | Executes *wrongly*. Detectable — see §8.1. |
| **Precondition false** | Conditional not met | Normal control flow, no log. |

Draft 5's **Capability lost** row is removed: nothing in the design can produce
that state. Commands are only ever added to the grimoire (§11), there is no
prestige reset, and drift explicitly never revokes a capability. Enemy-caused
unavailability is **Sabotaged**.

### 8.1 The sabotage surface

The keystone mechanic. The enemy may mutate **four surfaces**:

| Surface | Example | Tell |
|---|---|---|
| **World state** | Reagents swapped, glyphs corrupted, entity substituted | Substituted entities fail ID check → `Referent missing` |
| **Script text** | A flag altered, a target changed, a line reordered | Anomalous formatting — spacing, glyph shape, alignment |
| **Trigger clocks** | A script retimed so it fires uselessly early or late | Schedule listing shows drift from bound time |
| **Logs** | Poisoned lines to misdirect diagnosis | Forged lines render subtly differently from genuine ones |

**Design rule: sabotage must be engaging to find, never frustrating.** The player
should not need to be a senior sysadmin. Every tampering is *visible if you look
at the right surface* — the skill is knowing which surface to inspect, not
deciphering an obscure clue.

**Tells are never colour-only.** Colour is forbidden as a sole carrier of meaning
(§14) and is invisible to screen readers. Every tell therefore has two channels:

1. **A structural visual signature** — spacing, glyph substitution, alignment
   drift, malformed record boundaries. Perceptible at a glance to a player who
   looks.
2. **A command-detectable signature.** `verify <target>` reports tampering on any
   one surface — script, schedule, log, or entity — instantly and cheaply. This is
   the one-command diagnosis of §5.1, it is fully accessible, and it makes the
   *visual* tell a speed bonus for observant players rather than a requirement.

**`verify --all` is Production-class, and its duration scales with the tower.**
Auditing everything takes time proportional to bound scripts plus log volume — so
it grows into a real cost exactly as the tower gets complex. A flat 30-second
audit would be trivially correct to open every siege with, "which surface do I
inspect first" would stop being a decision, and the four-surface model would
collapse on move one.

**`verify <target>` carries a short per-surface cooldown.** There are only four
surfaces; four free instant checks *are* `verify --all` by another name. The
cooldown makes choosing the right surface meaningfully better than sweeping all
of them, which is the decision §5.1 is built on.

**The script-text visual tell is a Bevy-frontend channel only.** It relies on font
control we do not have in two places: files opened in an external editor (§8,
hot-reload) render in that editor's font and saving reformats them, and the
terminal frontend (§13) uses the user's font entirely. `verify` is the
authoritative detector on every surface and every frontend; the visual tell is a
speed bonus for players reading inside the Bevy orb.

**Log poisoning always leaves one trustworthy source** to cross-check against (the
orb's own attention ledger). An unreliable narrator is only fair when truth is
obtainable.

**Scripts the player has forgotten they bound** are covered on three surfaces, all
of which exist already:

1. **The boot report lists every bound script and its health**, so each session
   begins by reminding you what is running unattended.
2. **Sabotage log lines name the affected script and its bind time**, so when
   something does break, the culprit is never anonymous.
3. **`verify --all`** audits every script, schedule, and log on demand.

### Gating

- **Capability unlocks** — conditionals, loops, triggers, bindings, offline
  accrual, and extra focus panes are each discovered and researched. Early scripts
  are linear command lists.
- **Upkeep is paid in Concentration, not mana.** A bound script holds a **whole
  slot** while it is bound, idle or not, so "what is worth automating" is a
  genuine portfolio decision — and at concentration 1 it is the sharpest decision
  in the game, because binding a second spell means letting the first one go.
  Mana upkeep would have been free in practice, since mana does not bind in the
  tower. Explicit from Phase 1.

### Soft drift

Enemies adapt, recipes shift, wards decay. Drift produces **Degraded**, never
**Capability lost**. Maintenance is an invitation, not a punishment.

### Sharing

Scripts are plain text from day one, so sharing works naturally. No infrastructure
at launch; Steam Workshop deferred (§19).

## 9. The focus system — splitting the mind

A **progression axis**, not a UI feature.

### Two separate axes

**Panes exist per domain. Multiplex capacity governs how many are open at once.**
These are different unlocks and must not be conflated.

| Axis | What it is | How it grows |
|---|---|---|
| **Panes** | One per domain — seven total | Unlocked by discovering that activity. Two exist at the start |
| **Multiplex capacity** | How many panes fit in the main window simultaneously | The "splitting your mind" research track |

Pane progression therefore rides on domain discovery and needs no separate
economy; multiplexing is the only thing the focus track itself sells.

### Terminology — three meters, three names

Used consistently throughout; earlier drafts overloaded "attention".

| Name | Meaning | Source |
|---|---|---|
| **Focus** | Manual concurrency — how many duration-actions *you* can have running | Multiplex capacity |
| **Concentration** | Automated concurrency — how many spells you can hold at once | Shared pool (§11.5) |
| **Execution budget** | Instructions per tick per script | Fixed constant (§8) |

**Concentration is denominated in *scripts*, and it starts at zero.** A bound
script holds one slot for as long as it is bound; the concurrent actions it drives
hold fractions of one. Both charges are needed and neither is redundant — the
whole-slot charge is what makes *"what is worth automating"* a portfolio decision
even for an idle script, and the fractional charge is what keeps §11.5's
multiplexing counterweight scaling with concurrency rather than with pane count.

The unit is scripts because that is the thing the player names, holds, and lets
go of. *"I am concentrating on `night_watch`"* is a sentence about a spell, not
about a number of in-flight brews — and starting at **0** makes the first
concentration level a real unlock rather than a raised ceiling.

### Layout — the orb resolves more detail

- **The main window** holds every open pane, all **fully rendered and fully
  functional**, up to multiplex capacity.
- **The sidebar** holds every other unlocked pane, minimised to a single line.
  Awareness only, **not commandable**.
- **One input line, always at the bottom.**

> **Superseded in full by §19's fixed grid.** Everything from here to the end of
> the tier table describes a screen the game no longer has: there is one grid,
> `F4` changes only how the main window divides, and no key or window size
> changes the size of the text. It is kept because the arguments below — four
> panes as the cap, ~60×16 as the workable pane, parity between the two display
> modes — all survive, and because the phase history refers to it.
>
> The replacement in one line: **120×45, always, at `min(W/960, H/720)`.** Four
> panes tile at 60×21, which clears the 60×16 this section calls workable, so the
> mechanism the tiers existed to fund is funded by the grid instead.

**Multiplexing raises the screen's fidelity.** The game begins at a low-fidelity,
large-text grid — a true old terminal. Engaging multiplexing zooms the orb out:
the cell grid gets denser, text gets smaller, and all four panes fit at full
function. Cancelling multiplex returns to the coarse grid. Diegetically, the
wizard focuses harder and the orb resolves more detail.

Fidelity tiers are **integer scale factors** on the 8×16 CP437 cell (§4) —
required anyway, since bitmap fonts must scale by whole pixels to stay crisp:

| Window | Tier 1 (default) | Tier 2 (multiplexed) |
|---|---|---|
| 1920×1080 | 2× → 120×33 | 1× → 240×67 |
| 2560×1440 | 3× → 106×30 | 2× → 160×45 |
| 1280×720 | 1× → 160×45 | — (already finest) |

**Tier 1 aims at ~100×28, not at the 80×22 floor.** The table sat one step
coarser and pinned every window to the floor exactly: a 4K display showed the
same amount of text as a 720p one, in 48×96-pixel glyphs. The floor is a *floor*
— what a window must clear to host the game at all — and using it as the target
made "a bigger window buys a bigger glyph" mean "a bigger window buys nothing
else". A bigger window still buys a bigger glyph; the whole table simply sits one
step down, and the aim is [`DEEP_FOCUS_FLOOR`] so a default window can host §9's
two-pane focus without being resized first.

**At 1280×720 there is no tier 2, and that is fine.** Scale 1 is the finest whole
pixel step, so Deep focus engages at the same fidelity — and it does not need a
finer one, because 160×45 already carries four panes. Tier 2 exists to *buy
cells*; a window that has them already needs nothing bought.

Tier 2 at 1080p gives four panes at roughly **60×16 each** — workable for the
dense log scanning a siege demands, against the ~28×8 a fixed-grid 2×2 layout
would have produced.

Only one fidelity step is needed; four panes is the cap.

### Two multiplex display modes, chosen by the player

Both render paths exist; which one is used is a **setting**, not a heuristic.

| Mode | What it does | Suits |
|---|---|---|
| **Deep focus** (tier 2) | Fidelity rises one step; four full panes at smaller text | Most desktop players |
| **Wide focus** (strips) | Fidelity stays at tier 1; extra panes render as compact strips | Large text, small windows, handhelds, TVs |

**The game picks a default from window size and font scale, and the player can
override it at any time — including mid-siege.** Window size and font scale are
proxies for visual acuity, not measurements of it; the game cannot infer who can
comfortably read tier 2, so it must not decide unilaterally. The setting also
serves players who simply prefer large glyphs, whom a detection-only fallback
would refuse.

**Parity is mandatory.** Both modes grant identical multiplex capacity, identical
panes, identical information, and identical synergies. Only the rendering differs.
If strips ever showed *less*, the setting would become a difficulty choice, and a
player who needs large text would be paying for it in capability.

**Shader consequence.** Scanline density, the RGB subpixel mask, and barrel
distortion must re-derive against the active cell size, or the tier change
produces exactly the moiré §4 identifies as the top legibility hazard. This makes
the CRT port dynamic rather than statically configured, and the zoom transition
itself is a first-class effect — atmospheric, never nauseating, and disableable
per §14.

**Commands route by domain name within the focused set.** `wield alembic` reaches
the laboratory pane, `ward north` reaches the battlements pane, with no switching
between them — because commands are discrete, one input line serves any number of
focused panes.

**This is what multiplexing actually buys.** At capacity 1, a disaster in the
alchemy lab while you are commanding the battlements forces a choice: swap the
laboratory into the main window (losing direct command of the defence) or let it
burn. Higher capacity lets you hold both and command both. Splitting your mind
converts an either/or into an and.

**An in-flight manual action reserves its Focus slot for its full duration.** You
may push its pane to the sidebar — the action keeps running and you lose only
command of it — but **the slot is not freed.** The wizard's mind is still on the
brew.

This is the rule that makes Focus load-bearing. Without it, a player could start a
10-minute brew, swap panes in one second, start another, and saturate all seven
domains inside seven seconds at *any* capacity — making multiplexing worthless and
making sidebar-parking a way to get maximum concurrency at minimum aberration
risk. With it, **manual concurrency equals multiplex capacity**, pane count is
breadth, and capacity is depth.

**A pane holds one production slot and one triage slot.** That is what "partially
locks" means precisely: a 6-minute brew occupies the laboratory's production slot
while a 20-second purge can still run in its triage slot. Only the production slot
consumes Focus.

**Amended in Phase 1: a pane may hold as many production slots as it has
instruments.** The laboratory has four tools (§10), so it can hold four — drawn
from the **same tower-wide pool**, never granted free. §19's *"one production
slot, tower-wide"* is unchanged: the counter stays global, and at capacity 1 you
run one tool and are not deciphering. What changes is only the per-pane cap.

This is a **balance change, not a wording fix**, and it required moving the
counterweight — see below.

### Progression

The player begins with two domain panes and capacity 1. Multiplexing is researched
and unlocked, raising capacity. Thematically, the wizard splits his mind.

### Synergy: all pairs, plus exposure

Beyond command access, co-present panes grant **standing synergies**. An open
brewing pane feeds potions to the siege automatically; an open scrying pane
annotates threats in the combat pane; forge beside battlements auto-repairs wards.

**Synergies require both panes to be in the main window**, not merely unlocked.
Sidebar panes grant awareness only. This is what keeps the multiplexing unlock
meaningful rather than something domain discovery gives away for free.

**All 21 domain pairs are authored.** To make that affordable against the writing
risk (§12), the *mechanics* come from a shared template with per-pair tuning —
only the flavour line is bespoke. Twenty-one distinctive sentences is cheap;
twenty-one bespoke systems is not.

**The multiplexing counterweight is Concentration upkeep per open pane, not
aberration rate.** Synergies grow O(n²) in open panes (three hold three pairs, four hold six)
so the counterweight must keep pace — but nuisance rate is hard-capped
(invariant 5), so at endgame, where max multiplex meets max trace and max drift,
a rate-based counterweight would stop counterweighting at exactly the point it was
designed for. Upkeep is uncapped and scales cleanly: **each concurrent production
action holds a fraction of a Concentration slot**, so multiplexing trades directly
against automation capacity.

The fractional charge is the reason Concentration keeps a per-action component at
all when it is *counted* in scripts. Charging only whole scripts would make this
counterweight blind to concurrency — a player would buy depth 4 and pay the same
as at depth 1 — which is exactly the pane-proxy failure the amendment below
describes, returning in a new unit.

**Amended in Phase 1: upkeep is charged per concurrent action, not per open
pane.** Panes were a proxy for concurrency, and the proxy broke the moment one
pane could hold four production slots (above) — a player would have bought
depth 4 at one pane's upkeep, decoupling depth from exposure, which is the trade
this whole section builds. Charging the thing itself rather than its container is
*more* faithful to the reasoning above, since concurrency is what upkeep was
always pricing.

It also protects a progression track from the cut line. Under the per-pane rule,
cut-line item 7 (*"multiplexing capped at two panes"*, §15) would silently cap
usable capacity at 2, stranding §11.5's 3-at-5h and 4-at-10h unlocks behind a
rendering decision — precisely what the paragraph below warns against. Per-action
upkeep keeps capacity 4 meaningful if that cut is ever taken.

**Concentration is decoupled from multiplex capacity as a progression track.** The
pool grows along **the Ley Line** (§11.5), so capping multiplexing for legibility
reasons (§4, cut-line item 6) does not silently cap the game's core progression
as a side effect of a rendering decision.

**Pane count and content are identical at every fidelity tier and window size.**
Tier 2 renders the same four panes with the same information as the strip-pane
fallback — it renders them more comfortably. A player on a large monitor must
never see *more of the siege* than a player at minimum resolution or maximum
accessibility font scale; they see the same thing, better.

**And splitting your mind raises the aberration rate.** More surface area, more
that goes wrong. Multiplexing is a genuine risk/reward decision rather than a
straight upgrade, and it self-balances: only players whose scripts are solid can
afford the extra chaos.

OS-window pane detaching is deferred — each costs a render target and CRT shader
pass.

## 10. Domains

Seven at launch, with tiered depth to survive solo scale.

| Domain | Path | Minigame form | Depth |
|---|---|---|---|
| **Defense** | `battlements/` | Command pressure at 1 Hz, ward placement | Bespoke |
| **Scrying** | `lens/` | Deduction — parse noisy logs to find truth | Bespoke |
| **Spellcraft** | `grimoire/` | Composition — build spells from components | Bespoke |
| **Brewing** | `laboratory/` | Sequence/recipe puzzle with timing — **see §10.1** | Bespoke |
| **Archive** | `archive/` | Decipherment; powers all discovery | Bespoke |
| **Summoning** | `menagerie/` | Resource allocation → autonomous siege units | Derived |
| **Enchanting** | `forge/` | Sequence + resource cost → persistent buffs | Derived |

**These forms are a table, not a design.** Phase 0 built brewing and archive as
commands with a duration and no decision content, which is what the column
exists to prevent. **Brewing is fleshed out at the head of Phase 1** and is the
worked example the remaining six are cut from — including the rule that decides
what a minigame here may be: outcome follows *what the player chooses given
readable state*, never how fast or precisely they act. §19 has the reasoning and
what it cost to find.

**Archive is bespoke, not derived** — it gates all discovery, is played most, and
stales fastest. Budget fallback: decipherment becomes mostly a resource sink with
occasional authored set-pieces. **That fallback is the plan of record until Phase
9a**: the archive's minigame was considered for the head of Phase 1 and
deliberately left where discovery is, because its own "see it" line — gaining a
verb — *is* the discovery loop, and because what the puzzle should feel like
depends on what it unlocks (§19).

**Scrying is elevated by the aberration model.** Log-parsing is how sabotage is
found. Build it early, alongside the siege prototype.

> **Amended: it is built early and the siege is not.** The five unbuilt domains
> became Phases 2–7 and the siege moved to 8, so "alongside" no longer holds.
> Scrying stays first for the half of the reason that survives — sabotage is
> illegible without it, and the calm layer produces sabotage from Phase 0.

#### What each remaining domain's scarcity is

§10.1 is the worked example and the six rows of its template are in ROADMAP's
*Phases 2–7* preamble. The load-bearing correction, recorded because a first pass
got it wrong twice: **brewing's scarcity is lit *time*, not fuel stock** — fuel is
`Holding::endless` on purpose — and `CAPACITY = 1` is tower-wide, so §5.0's
*"concurrency is the real scarcity"* is a resource every domain already shares.

| Domain | Its scarcity | Its choice |
|---|---|---|
| **Scrying** | the production slot — a read is not a brew | which source to read first |
| **Spellcraft** | Concentration, §11.5's shared pool | what is worth factoring out |
| **Enchanting** | the buff's own lifetime, and the slot | which instrument, and when to re-buff |
| **Summoning** | allocation — a unit is spent stock | what to spend it on, and the unit's standing rule |
| **Defense** | wards, made and consumed | where, given readable approach state |

**Two things are ruled out and both were tried on paper.** A resource cost on
*looking* contradicts §5.1's *"issuing commands is free"* and would push a player
without potions onto the visual tell, which §14 makes a bonus and never a
requirement. And a second fuel supply contradicts `build.rs`'s own reason for
endless charcoal: *"a cold athanor with nothing to burn is a laboratory with
nothing to do."*

**Defense is the one that can still fail the rule.** *Command pressure at 1 Hz*
is a reflex mechanic unless something makes it a decision, and §5.1's answer for
aberrations — *"identifying any aberration costs exactly one command, never a
sequence"* — has no ward-placement analogue yet. Finding one is head-of-phase
work and the phase does not start without it.

### 10.1 Brewing — the worked example

**The laboratory holds five instruments, four of which take Focus.** Brewing is
moving materials through them; each is a place you can `survey`, and the
laboratory's pane draws them as a permanent instrument panel.

| Stage | Instrument | Needs heat | Focus | Leaves behind |
|---|---|---|---|---|
| preprocess | `mortar_and_pestle/` | — | ✅ | husks |
| process | `balneum_mariae/` | **athanor** | ✅ | sediment |
| combine | `flask_and_rod/` | — | ✅ | dregs |
| distil | `alembic/` | **athanor** | ✅ | phlegm |
| *(heat)* | `athanor/` | — | ❌ | ash |

Plus `dispensary/`, a holding area for materials between instruments.

**Four Focus-consuming tools; §11.5's capacity track tops out at four.** The
athanor is infrastructure rather than a stage, which is what makes the arithmetic
exact instead of argued.

**The loop, per tool:** clear it (`purge`), load it (`move`), start it (`wield`),
collect (`siphon`). A running tool is locked — `stop` cancels with a refund, or
you wait. That lock is what makes different recipes need different scripts.

**There is no command that brews a potion.** `decoct` is retired (§6.1): the only
way one line makes a potion is a spell the player wrote, which is §8's whole
argument for automation and would be given away by any built-in shortcut.
*"make a potion of clarity"* resolves to `grimoire clarity` — the recipe — which
is the tutorial entry point rather than a refusal.

**The athanor is a shared, depleting resource.** It takes fuel (charcoal), burns
at a **constant rate while lit**, and heats whichever of the `balneum_mariae` or
`alembic` is working. Fuel is an interval, not a countdown —
`remaining = fuel_at_lighting − (now − lit_at) × RATE`, a pure function of the
tick, per §19. A load-varying rate is forbidden: it would require integrating
over which tools were mounted when.

**Heat is checked when an operation starts, and the run then completes.** The
timing pressure comes from the burn being *time-based*, not from spoilage risk:
the athanor consumes fuel per lit tick whether or not anything is mounted, so
idle lit time is pure waste. Light it once, get both heated stages done inside
that window, damp it. This is a **window at 1 Hz**, never a reflex — and the cost
is fuel, never progress, so §11.5's *"not automating is never ruinous, only
slower"* holds.

**It is also the cleanest argument for automation in the design.** A manual
player walks away with the athanor lit and wastes fuel; a script ends its loop
with `stop athanor` and does not. §8's claim that automation beats doing it by
hand becomes mechanical rather than asserted.

**Damping banks what has not burnt, and that makes a mid-brew move available.**
The two heated stages are *not adjacent* — the unheated `flask_and_rod` sits
between the bath and the still — so burning through the combination is waste.
The efficient play is **light → digest → damp → combine → relight → distil**.

That was not designed. It fell out of the durations and was found when the fire
died in the middle of the end-to-end test. It is the best argument yet that
`stop athanor` is a real move rather than an end-of-script tidy, and it is
exactly the shape of decision ROADMAP asked for in *"timing means windows at
1 Hz — when to advance a stage against everything else wanting the slot, never a
reflex."*

**A charcoal outlasts a brew by a wide margin, deliberately.** The first pass
made one barely cover a single brew, which turned that move from an optimisation
into a tax: damping mid-brew was not a good idea, it was the only idea, and the
fire wanted watching like a third instrument. The pressure is unchanged in kind
and much gentler in degree — idle lit time is still pure waste, so batching and
damping still save fuel. It is now something an **attentive player takes and a
careless one merely pays for in fuel**, never in progress, which is §11.5's
*"not automating is never ruinous, only slower"* holding at the scale of a
single decision. The script that damps is meaningfully ahead over a session
rather than trivially ahead over one brew, and that is the better argument for
writing it.

**Ash accrues at burn-out, not per tick.** Per-tick accrual would spawn nodes
into `Children`, and §19 makes insertion order *the parse* — a live-watched burn
and a `meditate`-collapsed burn must issue identical ids in identical order. A
single spawn at exhaustion is trivially order-identical.

**Variance comes from byproducts, under one rule: every byproduct has at least
one use.** A tool fouled by the last brew must be cleared before use, and a
byproduct that is an *input* to another recipe makes the laboratory's current
state change what is optimal now. A byproduct that is only ever litter would make
`purge` into tidying — the exact feeling this item exists to remove.

**Recipes live one file per instrument** (rule 6), each with a single shape:
ingredient→material, material→material, material+material→material,
material→potion, and the athanor's fuel table. The instrument is implied by which
file a recipe lives in, and a material's **state** decides which recipes can fire
— so the material suggests the next operation rather than the player memorising a
sequence. That is what gives the puzzle repeat-tolerance.

**Capacity 1 is the form that ships.** §11.5 reaches capacity 4 at ~10 h and
capacity is researched, which is Phase 9a — so for Phases 1 and 2 the four stages
are sequential and the decision is *when to light the athanor* and whether both
heated stages fit one window. Capacity 4 is where the laboratory becomes a
pipeline, with the mortar grinding the next brew while the alembic distils this
one.

**On shared engines — build concrete first, extract later.** Draft 3 mandated
generic-engine-first; that is the classic route to the wrong abstraction, because
the generalisation axis is only knowable from the *second* consumer. Instead:
build each bespoke domain concretely and cheaply, and schedule an explicit,
budgeted **extraction task in Phase 9** when the derived domains arrive. If the
derived domains are cut, the extraction is cut with them at zero loss.

## 11. Progression

**Discovery + research.** Commands are found, then understood, then used.

1. **Discover** a fragment — siege, hidden directory, remote host.
2. **Research** it in `archive/`.
3. **Add** it to the grimoire; available to parser and scripts.

Shell verbs, domain verbs, and capabilities (conditionals, loops, triggers,
offline accrual, focus panes) all unlock on this single track.

**Not the only track, since §11.5's experience curve** (**amended §19**).
Concentration and multiplex capacity are bought with work rather than researched
with fragments: what §11 governs is what the player can *say*, and what the curve
governs is how much the orb can *do*. A player between fragments is still
advancing.

**Trait knowledge** is recorded in `lens/observed/` on first successful field
identification (§5.2).

**The quiet-tower path plateaus, and the design says so plainly.** Fragments come
from sieges, hidden directories, and remote hosts; hidden directories are finite.
A player who never provokes and never connects will run out of progression. That
is a deliberate stance — pillar 4 promises the calm layer is *safe*, not that it
is a complete way to play indefinitely. The calm layer carries a slow renewable
fragment trickle so the plateau is soft rather than a wall, but the ceiling is
real and the game should not pretend otherwise.

**Softer since experience** (**amended §19**), and the stance is unchanged: a
quiet player keeps earning, so the tower keeps automating and the numbers keep
moving. What they run out of is *vocabulary* — new verbs, new capabilities, new
domains — which is the ceiling that was always the honest one.

**Vocabulary budget: 45–55 canonical commands.** Cut from 60–80 because vocabulary
is the dominant multiplier on the writing budget (§12). Each canonical command
carries synonyms across three registers (§6), but man pages and failure text are
written per canonical command — so the synonym layer widens what players can
*say* without widening what must be *written*.

## 11.5 Resources and curves

*Structure settled in the economy session. Numbers are first-pass targets for the
balance harness (§13) to sweep, not final values.*

### Anchors

| | |
|---|---|
| **Tick** | 1 real second while the window is open |
| **Soft ending** | 15–25 hours (~54,000–90,000 ticks) |
| **Time to first bound script** | ~2 min — one clarity, brewed by hand (**amended §19**; was 30–45 min) |
| **Time to first siege opportunity** | 1.5–2 h |
| **Siege provocation cadence** | every 20–30 min at a normal push rate |

### Action durations

Range: **10 seconds to 10 minutes.** The rule is that duration tracks
consequence — triage is fast, production is slow.

| Class | Duration | Examples |
|---|---|---|
| Instant | 0 | `ls`, `cat`, `cd`, `status`, `verify <target>` |
| Triage | 10–30s | purge byproduct, patch a glyph, `verify --all` |
| Routine | 1–3 min | ward a gate, decipher a fragment |
| Production | 3–10 min | brew a potion, enchant an item, bind a familiar |

**Everything is worth automating** — not because manual play is punished, but
because automation is *non-blocking*.

### Blocking, and why automation wins

- **A manual duration-action partially locks its pane** — brewing occupies the
  laboratory's brewing functions while inspection, reading, and purging stay
  available. A player at capacity 1 is never fully stuck, which matters most in
  the first hour when they have nowhere to switch to.
- **Script actions lock nothing.** This is automation's headline benefit and it
  needs no arithmetic to feel: not faster, not cheaper — *non-blocking*.
- **Not automating is never ruinous**, only slower. Players who want to do it by
  hand can, until they choose to push far enough that they cannot.

### Focus, panes, and concentration

| Track | Start | Growth |
|---|---|---|
| **Domain panes** (breadth) | 2 of 7 | One per activity discovered; all 7 by ~hour 10 |
| **Focus / multiplex capacity** (depth) | 1 | 2 at ~1.5h, 3 at ~5h, 4 at ~10h |
| **Concentration** (automation) | **0** — everything by hand | **1** at 16 experience — one clarity, brewed by hand; **~8** by the soft ending, along **the Ley Line** (below) |

**Concentration starts at zero, and the first level is the game's turn.** Until it
is bought the tower is worked entirely by hand: the player can `scribe` a spell
and read it back, but the orb cannot hold one. That is deliberate. Pillar 3's
promise is that *teaching the orb to do your work* is the progression, and a
promise the player is handed at minute zero is not a progression — it is a
premise. Buying concentration 1 is the moment the game becomes the game it
advertises, and it should be reachable inside the first hour.

**Concentration 1 is also the sharpest decision in the game**, for exactly as long
as it lasts. One slot means one standing spell, so binding a second is *letting
the first one go* — the portfolio question §8 describes, at its most
uncompromising, before the player has enough slots to stop thinking about it.

**~8 by the soft ending, and the arithmetic is inherited rather than invented.**
The retired Attention pool was denominated in actions and ran 3 → ~25 on the
reasoning that at ~6 min average duration, 25 slots is ~250 actions/hour against
a manual ceiling of 40/hour at Focus 4 — automation dominating by roughly 6×,
which is what pillar 3 promises. §8's `night_watch.spell` issues a ward, a double
brew, a purge and a conditional: three or more concurrent actions from one script.
**~25 actions ÷ ~3 actions per script ≈ 8 scripts**, so the same 6× advantage
survives the change of unit. Growth stays stepped and non-exponential.

Each open pane holds a fraction of a slot (§9), so breadth of attention trades
against depth of automation. **The fractional charge is deferred** (**amended
§19**): a whole slot is drawn per bound spell and nothing is charged per action
in flight, because what the fraction exists to price is multiplexing, and only
one production action may be in flight until multiplex capacity is bought.

### Experience — what work is worth

**Experience is the unlock currency, and it is earned by working.** It
accumulates and is never spent: an upgrade opens when the total passes its
threshold and stays open. One `u64` in a save, no balance to keep, and nothing a
player can spend and then wish they had not.

**Fragments no longer price progression** (**amended §19**). They remain what
`archive/` research consumes — the discovery loop, deciphered from sieges and
hidden directories — but a capability that opens because you *did the work* is
the loop pillar 3 is about, and a capability that opens because you *found a
thing* is a different game wearing the same number.

One completed run earns by the instrument that did it, doubling per tier:

| Instrument | Earns | |
|---|---|---|
| `mortar_and_pestle` | **1** | the cheapest complete run there is |
| `balneum_mariae` | **2** | |
| `flask_and_rod` | **4** | |
| `alembic` | **8** | |
| `research` (archive) | **1** | one of the two opening domains; at zero, half the opening would be dead progression |

Binary, so **each tier of tool is worth every use of the one below** — the next
rung always pays more than grinding the last one forever, without the number
growing fast enough to outrun the curve.

**16 is the first threshold, and it is arrived at rather than picked**: 1 + 2 + 1
+ 4 + 8 is one clarity walked end to end — grind, digest, grind, mix, distil. The
player buys concentration 1 with exactly the potion the tutorial teaches, which
is why the number is derived from the recipe rather than chosen against a clock.

Only work that *succeeded* earns. A scour, a refused command, a run that matched
no recipe, and the debug reagent spawn all earn nothing.

**Content gates are future work, not built.** *"Brew this to open that domain"* is
the shape §11's discovery loop wants, and it is a second axis over experience
rather than a replacement for it.

### The two tracks: the Ley Line and Mastery

**This replaces *"ley-line upgrades and grimoire rank"***, which named two
sources of Concentration and gave neither any mechanical content — inherited
verbatim from the draft-7 economy session, when the pool was still called
Attention, and carried through three amendments untouched. `grimoire rank` is
**dropped** (§19): `/grimoire` is already the directory holding the player's
spells, and a rank of the same name would make one word mean both the book you
write in and a number beside it.

| | |
|---|---|
| **The Ley Line** | The straight path. Predefined steps in order, and **passing one is the grant** — there is no choice in it and no moment where a step is reachable but unheld. Concentration lives here |
| **Mastery** | The branching tree. A tier opens when the total passes it and gives exactly **one** of its nodes, so the tier opening is the grant and *which node* is the decision |

**This is what lets experience stay unspendable while still offering a choice.**
The ROADMAP asks that a player *"be offered something you choose between, rather
than a level that arrives on its own"*, and the obvious way to get that — a
spendable points balance — would contradict *"accumulates and is never spent"*
above and add something a player can spend and then regret. A tier opening costs
nothing; taking one of its nodes closes it. A save carries one `u64` and a list
of taken ids.

**Both are authored in `progression.toml`** and are drawn by `weave` (§6.1), which
is the only surface progression has. A node's **id** is a decision and lives in
that file; its **sentence** is prose and lives in `prose.toml` keyed by the id.

**Nothing is behind a Mastery node yet.** Every one is authored as a marker and
`take` refuses in voice, because the first real node is a balance question and
the screen is not. What it buys is that the *shape* is visible from inside the
game before anything is behind it: a player who reaches 24 watches a tier open
and learns that a choice is coming.

### Mana — a fixed siege budget

Mana regenerates freely in the tower and rarely binds there. **Under siege it is a
fixed pool granted on entry, with no regeneration**, budgeting repairs and
invocations.

Fixed-per-siege rather than regenerating-per-tick is deliberate: §14's
screen-reader mode advances siege ticks on player input, and a per-tick regen
would mean *more typing produces more mana*, inverting the economy for exactly the
players the mode exists to serve. A fixed pool gives identical mana-per-decision
whether ticks come from a clock or a keystroke, so triage pressure survives in
both modes and §14's "preserves every decision" becomes true rather than
aspirational.

**Diagnosis is never mana-costed** — only repairs and invocations are. Issuing
commands remains free (§5.0); what mana budgets is *changing the world*.

### Siege escrow

Rewards accrue into escrow and settle on exit:

| Exit | Settlement |
|---|---|
| **Completed** | Full escrow + 50% completion bonus |
| **Lost / abandoned / crashed** | Escrow × completion fraction, floored at 20% |

Scaling by progress punishes the specific exploit — bailing early after farming a
cheap opening — while a run that fails at 90% still pays nearly in full. Effort is
never wasted; only *cynicism* is.

### Unattended sieges — a flow balance, not an accumulator

**Sieges are never mandatory, and ignoring them is never a hard block.** But they
compound, and the wizard pays to hold them at bay:

| Backlog | Effect |
|---|---|
| 1–2 | Small continuous resource drain; idle yields dip slightly |
| 3–4 | Drain grows; idle yields noticeably reduced |
| **5 (cap)** | **Offline progression suspended** until the backlog drops below 5 |

**The backlog is capped at 5 and provocation is suppressed at cap.** Without a cap
the ladder either plateaus — making decline free forever past 5 — or scales without
bound, which at a 25-minute cadence would make the game unplayable ~2 hours in for
anyone who declines as often as this design expects.

**Backlog decays.** An unattended siege lapses on its own after a period of paid
drain. At a 25-minute cadence with a one-in-three fight rate, net inflow is ~1.6
per hour; a lapse rate of ~1.5 per hour holds equilibrium around 3–4 — pressure
that is felt continuously without ever becoming a wall.

**A siege clears three ways:**

1. **Fought to at least 20% completion** — won or lost.
2. **Dispersed** at a resource cost. Buying peace is legitimate and expensive.
3. **Lapsed** after its drain period.

Entering and immediately quitting does **not** clear a siege. Escrow already pays
≈0 at f≈0, but without the completion threshold, quitting would clear the backlog
free in seconds — which would make dispersal strictly dominated and delete this
entire pressure system. Below 20%, leaving is a decline and the siege stays queued.

### Trace

Trace accrues from production heat, scrying depth, and infiltration (§5.3),
composing multiplicatively against the nuisance-rate ceiling.

**Provocation subtracts the threshold from accumulated trace**, so a declined
siege does not leave trace parked at the threshold re-provoking immediately.

At 13–23.5 eligible hours and a 20–30 minute cadence, the game offers roughly
**26–70 siege opportunities**, of which most players will decline many.

### Sinks — a worked example

Supply without demand cannot be balanced, and the harness (§13) needs something to
sweep against. First-pass targets:

| Sink | Cost | Source |
|---|---|---|
| Capability unlock (a command, a conditional, a pane) | 3–5 fragments | Sieges, hidden dirs, trickle |
| A completed siege | — | yields 2–4 fragments |
| A brewed potion | 2–3 reagents | Reagents from brewing and sieges |
| A siege consumes | 4–8 potions | — |

**~64 research events across the game** (≈50 commands + ~10 capabilities + 5
domain discoveries — **amended §19**: the ~8 concentration steps and 3 multiplex
steps are bought with experience, not researched) at 3–5 fragments each is ~250
fragments.

Fragment supply, first pass:

| Source | Yield | Total |
|---|---|---|
| Sieges fought | 2–4 each, ~20 fought | ~60 |
| Hidden directories | authored, finite | ~80 |
| **Calm-layer trickle** | **6–10 / hour**, rising slightly as domains unlock | **~160** |

**The trickle carries the majority of progression**, which makes its rate a
first-order balance constant rather than a footnote. At ~8/hour a fragment arrives
roughly every 7 minutes — frequent enough that a quiet player always feels
movement, slow enough that sieges remain the efficient path.

The trickle is also what makes the quiet-tower plateau *soft* (§11): a player who
never provokes still advances, just at roughly a third of the rate.

### Cross-check: the unlock cadence corroborates the length

~64 research events over 15–25 hours is **one unlock every 14–23 minutes** — a
healthy drip for an idle game. This is independent evidence that the 45–55
vocabulary budget (§11) and the 15–25 hour length agree with each other; the two
most-argued-over numbers in the document turn out to be mutually consistent.

**And the drip is now two drips**, which is why moving 11 events off it did not
thin it: experience runs on its own clock, so a player between fragments is still
advancing. The cadence above is the *research* rate, not the progression rate.

### Resources

**Ticks are not in this table.** They are world time (§5.0), not a resource. The
scarcity they used to represent is now expressed as **action duration and
concurrency**.

| Resource | Produced by | Consumed by | Role |
|---|---|---|---|
| **Concurrency** | Focus panes (§9) | Duration-actions in flight | The real throttle on what you do by hand |
| **Mana** | Passive regeneration, Ley Line steps | Invocations, script upkeep, repairs | The throttle on action |
| **Reagents** | Brewing, sieges (exclusive tiers) | Potions, enchantments, repairs | Crafting economy |
| **Fragments** | Sieges, hidden directories, remote hosts | Research in `archive/` | Discovery gate |
| **Experience** | Completed runs, weighted by instrument | **Nothing — it only rises** | Progression gate |
| **Integrity** | Repair, warding | Damaged by sieges, decay, aberrations | Tower health, persistent |
| **Concentration** | The Ley Line, against experience (**not** pane count) | Shared pool across all bound scripts | Caps total automation |

**Concentration is a shared pool, not a per-script bound.** Draft 5 described
both; they are different mechanisms with different balance behaviour. A shared
pool makes "what is worth automating" a real portfolio decision, which is the
intended texture.

That it is *counted* in scripts does not make it per-script: one slot is not
reserved for one spell. The pool is shared, a bound script draws a whole slot from
it and its in-flight actions draw fractions, and what the player buys is the size
of the pool.

### Intended shape

- **Time to first bound script:** ~2 min (**amended §19**; was ~30–45 min). The "I
  taught it to do that" moment must land inside the first session, and the anchor
  moved when the threshold was derived from the tutorial potion rather than from a
  clock. Automation arriving early is the game showing its hand, not skipping its
  first act — what carries the pacing afterwards is the rest of the curve, where
  a second slot costs many clarities rather than one.
- **Time to first siege:** ~1.5–2 h, entirely player-elected.
- **Growth:** roughly linear within a tier, stepped at each capability unlock.
  Deliberately *not* exponential — this is an automation game, not a
  numbers-go-up game, and exponential curves would trivialise the mid-game.
- **Prestige/reset layer: none at launch.** Idle games usually need one and
  retrofitting is brutal, but O.R.B.S.'s replay engine is sieges and trait
  knowledge, not resets. Revisit only if playtesting shows the late game flattens.

### Invariants

1. Offline execution cannot reduce Integrity below a set fraction of its departure
   value. No other resource is floored (§5).
2. A failed or ambiguous parse changes no state whatsoever (§5.0).
3. ~~A script-executed action completes in strictly less time than the same action
   issued manually.~~ **Struck** (§19). A bound spell runs at exactly manual speed;
   a *speed* upgrade is later work and, when it lands, it is a thing you buy rather
   than a property scripts are born with. What automation sells is that it runs
   while you are elsewhere and while you sleep, which is the larger multiplier and
   the one pillar 3 actually names.
4. **An in-flight manual action reserves its Focus slot for its full duration**,
   regardless of which pane is displayed (§9).
5. No siege can reduce tower integrity below its pre-siege value (§5).
6. Nuisance arrival rate has a hard ceiling regardless of trace, panes, and drift
   (§5.3). The multiplexing counterweight is therefore Concentration upkeep, not
   nuisance rate (§9).
7. Unattended-siege backlog is capped at 5, decays over time, and clears only at
   ≥20% completion, by dispersal, or by lapse.

## 12. Onboarding and the writing track

**The single largest commercial risk**, starting in Phase 1 — onboarding failures
demand changes to vocabulary, filesystem shape, and gating order, all of which
freeze early.

1. **Diegetic apprenticeship** (first ~hour) — the orb teaches as a mentor. Each
   command arrives with in-world justification. Onboarding is story.
2. **Progressive reveal** (throughout) — the world starts tiny. Three commands,
   two directories.
3. **In-world grimoire** (always) — `help`/`man` as magical reference, plus errors
   that always suggest a next action.

The parser is itself the primary accessibility feature: "look at the gates" gets
somewhere.

### The writing budget

Text-only relocates content cost from art to prose. **This is the production risk
that most threatens the schedule**, and it is worth being blunt about why:
court_wizard shipped 167k lines in seven months, but with **22,985 rows of
third-party LPC art** — its content cost was *externalised*. There is no
equivalent source for 100k words of bespoke reactive prose. The thing that made
the last project fast is unavailable here.

**Mitigation 1 — compose the low-traffic text, author the high-traffic text.**

Draft 5 had this backwards. It proposed composing failure strings — but failure
and suggestion text is the **highest-traffic prose in the game** and the exact
surface where the #1 Critical risk ("parser feels frustrating rather than
magical") fires. Composing there is composition applied at the worst possible
place.

Inverted:

| Author by hand | Compose at runtime |
|---|---|
| Failure and suggestion frames (~50 commands × 5 modes) | Reagent and item descriptions |
| Disambiguation prompts | Ambient flavour and idle lines |
| Apprenticeship and set-pieces | Boot report lines |
| Discovery and lore fragments | Routine status output |

Same word budget, far better placement. §13's structured-record model provides the
composition architecture either way.

**Mitigation 2 — content data format from Phase 1.** All prose in hot-reloadable
TOML/RON keyed by state, never string literals in Rust.

**Mitigation 3 — a word budget per phase**, tracked with a running total in §15,
so slippage is measurable rather than discovered in Phase 11.

**Mitigation 4 — vocabulary size is the dominant multiplier** (§11).

## 13. Technical architecture

### Stack

| | |
|---|---|
| **Language** | Rust, edition 2024 |
| **Toolchain** | Pinned via `rust-toolchain.toml`; stable channel |
| **Engine** | Bevy `=0.19.0` (exact pin — see §16 on version drift) |
| **ECS** | `bevy_ecs` — used standalone in `orbs-sim`, and via `bevy` in the frontend |
| **Rendering** | Custom cell-grid text renderer + ported CRT post-process. **Not** `bevy_text`/`bevy_ui` |
| **Serialisation** | `serde` + `toml` (readable saves, §7) |
| **RNG** | `rand` 0.9, seeded, per-subsystem streams |
| **Errors** | `thiserror` |
| **Logging** | `tracing` |
| **Paths** | `dirs` |
| **CLI (balance harness)** | `clap` |
| **Steam** | `bevy-steamworks`, with `steamworks`/`steamworks-sys` exact-pinned |

### Workspace layout

```
orbs/
├── crates/
│   ├── orbs-sim/       world model, parser, script engine, aberrations,
│   │                   threat composition — no rendering deps, headless
│   ├── orbs-render/    Frame / cell-buffer types, layout, semantic styling.
│   │                   Consumed by BOTH frontends. No backend deps
│   ├── orbs-shell/     The shell BOTH frontends share: painters, surfaces,
│   │                   the headless dump. orbs-sim + orbs-render + bevy_ecs
│   ├── orbs/           Bevy frontend: GPU cell renderer, CRT, audio, Steam
│   ├── orbs-tui/       Terminal frontend: crossterm, raw mode, ANSI
│   └── orbs-balance/   CLI harness driving orbs-sim (§11.5 sweeps)
├── assets/
├── docs/
└── .github/workflows/
```

**`orbs-sim` depends on `bevy_ecs` only — never on `bevy` the engine.** The world
model is ECS throughout; what is excluded is rendering, windowing, assets, and
anything needing a GPU. Standalone `bevy_ecs` is ~90 crates and adds no renderer;
`bevy` is ~340 and adds all of it.

This is what makes balance sweeps, replay, determinism, and accessibility cheap,
and it is the single most important structural decision in the project. It also
means components and systems are shared vocabulary across the sim and the
frontend, rather than two models that must be kept in sync.

### Pluggable frontends — Bevy first, terminal second

**The Bevy build is the product.** It ships on Steam, it carries the marketing,
and it is never allowed to slip for the terminal build. The terminal frontend is
a **deliberate second-class citizen**: wanted, pursued, and cut before the Bevy
build is if the schedule demands it.

**The abstraction, however, is not optional.** `orbs-render` owns the **`Frame`** —
a cell buffer with semantic styling (role tags like *danger*, *cost*, *success*,
*eldritch*, never concrete colours) plus layout. `orbs-sim` produces records;
`orbs-render` turns records into a Frame; a frontend rasterises the Frame.

> **Rule: `orbs-render` decides what appears and where. Frontends decide only how
> a cell is drawn.** A frontend may add enrichment the other cannot reproduce —
> CRT effects, audio, fidelity tiers — provided **that enrichment carries no
> information absent from the Frame.** The moment a frontend conveys something the
> Frame does not, the other frontend is playing a worse game rather than wearing a
> different skin.

This costs almost nothing and buys a great deal: it is the same discipline that
keeps `orbs-sim` headless, it makes any future frontend a drop-in, and it is what
keeps the terminal build a *possibility* rather than a rewrite.

| | `orbs` (Bevy) | `orbs-tui` (terminal) |
|---|---|---|
| Surface | GPU cell renderer, glyph atlas | Alternate screen buffer, raw mode |
| Font | Embedded CP437 8×16 bitmap | The user's terminal font |
| Box drawing | CP437 codepage | Unicode U+2500 block |
| Colour | Curated phosphor themes (§4) | **Indexed ANSI 0–15 — inherits the user's terminal theme** |
| CRT effects | Full (§4) | None |
| Fidelity tiers | Yes (§9) | N/A — the terminal is whatever size the user made it |
| Audio | Full | **Luxury, unscheduled.** Not forced out — a TUI binary could link `rodio` independently of Bevy — but it must never hold up development |
| Steam | Native | Via Steam launch option |

**The terminal build is a full-screen application**, in the manner of the Ubuntu
Steam installer, `htop`, or the Debian installer. It takes over the alternate
screen buffer and draws the whole grid itself. It does **not** shell out, does not
touch the real filesystem, and does not interoperate with the host shell —
`attend /tower/laboratory` navigates the simulated tower exactly as it does under
Bevy. Same sim, same commands, same world; only the rasteriser differs.

**Input is a strength, not a compromise.** O.R.B.S. is a typed command line plus a
few hotkeys, which is precisely what terminals do best — line editing, history,
and completion are easier in raw mode than reimplementing them over Bevy's
keyboard events. The usual terminal input limitations (no key-up, modifier
quirks) bite games needing held keys or gamepads, and do not apply here.

**Consequences accepted:**
- The CRT identity exists only in the Bevy build. That build remains the trailer,
  the capsule, and the marketing.
- §8.1's glyph-substitution and alignment sabotage tells are unreliable under a
  font we do not control. `verify` is already the authoritative detector and the
  visual tell already only a bonus (§8.1), so this degrades rather than breaks.
- Distribution: if the terminal build ships, both binaries go in the Steam depot
  as two launch options — one purchase, two ways to play. If it does not, nothing
  about the Bevy product changes.

### Bevy features

Start from `default-features = false`. The custom text renderer means **`bevy_text`,
`default_font`, `bevy_ui`, and `bevy_ui_render` are not needed** — settings, menus,
and every screen are rendered as terminal content, in-fiction. That is a
substantial saving over court_wizard's list.

**`bevy_sprite_render` is required and was missing from this list.** 0.19 split
the 2D rendering half out of `bevy_sprite`; without it `Mesh2d`, `ColorMaterial`,
and `MeshMaterial2d` do not exist and nothing 2D draws at all.

```toml
bevy = { version = "=0.19.0", default-features = false, features = [
    "std", "async_executor", "multi_threaded",
    "bevy_winit", "bevy_window", "bevy_input_focus",
    "bevy_render", "bevy_core_pipeline", "bevy_sprite", "bevy_sprite_render",
    "bevy_mesh", "bevy_image", "bevy_camera", "bevy_color",
    "bevy_asset", "bevy_state", "bevy_log",
    "bevy_audio", "vorbis",
    "png",
    "serialize",
] }

[target.'cfg(target_os = "linux")'.dependencies]
bevy = { version = "=0.19.0", default-features = false, features = ["x11", "wayland"] }
```

### Build profiles

Following court_wizard, which is proven across a shipped title:

```toml
[profile.dev]
opt-level = 1
debug = "line-tables-only"

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"
```

Dev iteration additionally uses `bevy/dynamic_linking` and a fast linker (lld or
mold). This is the largest quality-of-life lever available and should be set up in
Phase 0, not discovered in Phase 9.

### Distribution — not literally one binary

Earlier drafts claimed "a single self-contained binary." **Steam integration makes
that false**: `libsteam_api.{so,dylib,dll}` ships beside the executable, which is
why court_wizard carries `.cargo/config.toml` rpath flags (`$ORIGIN` on Linux,
`@loader_path` on macOS). O.R.B.S. inherits both the dependency and the flags.

The honest claim is **one executable plus one Steam shim, with all assets
embedded** — still far simpler than the engine + extension + pack arrangement that
ruled Godot out, but not a single file. Assets (the CP437 bitmap font, shaders,
content tables) are compiled in, so there is no loose `assets/` directory at
runtime.

### CI and platforms

Mirroring court_wizard's proven shape:

| Target | Runner | Notes |
|---|---|---|
| `x86_64-pc-windows-gnu` | `ubuntu-latest` | Cross-compiled; no Windows runner needed |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | Needs `libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev` |
| `aarch64` + `x86_64-apple-darwin` | `macos-latest` | Universal via `lipo`, signed and notarised with `rcodesign`, release-only |

macOS is built only for releases because signing and notarisation are slow.
Workflows: `build.yml`, `macos-release.yml`, `release.yml`, `steam-upload.yml`.

### Audio

The design has been silent on audio, which is a gap for a game whose entire output
is text. **A terminal needs a voice**: key clicks, the orb's hum, completion
chimes, alert tones on aberrations, and a low drone that shifts with the tonal
register (§3). This is cheap to produce, disproportionately effective at selling
the CRT fiction, and it is the only sensory channel besides text available.

Audio also carries real accessibility weight — distinct tones for
completion/warning/sabotage give screen-reader players an ambient channel that
does not compete with speech. This applies to the Bevy build; audio in the
terminal build is a luxury (§13, frontend table) and is never a schedule item.

Scope: a small library of synthesised or short sampled cues, no music bed at
launch beyond an optional drone. `bevy_audio` + `vorbis`, already in the feature
list.

### Other

- **Conventions:** inherit court_wizard's — feature-sliced modules, `plugin.rs`
  for registration only, ~300-line file cap, `Message` not `Event`, mandatory
  `run_if` guards. `clippy.toml` with `type-complexity-threshold = 500`.
- **Crash handling:** port court_wizard's crash handler.
- **Structured records everywhere.** Commands emit records; presentation is a view.
  Pipes, `sift`, the eldritch renderer, screen-reader linearisation, and the test
  harness all depend on this.
- **No `async` in `orbs-sim`.** Async brings non-deterministic completion
  ordering and scheduler-dependent interleaving — exactly what breaks replay,
  offline/online parity, and harness-vs-live agreement. `Sim::step(&mut self)`
  makes re-entrancy and cross-thread driving statically impossible. Frontends may
  use their backend's concurrency (Bevy task pools, `crossterm`'s loop) but never
  to drive the sim. No `tokio`: there is no networking. Long work goes to a worker
  thread, not an async runtime.
- **Determinism is architected, not assumed.** Two rules binding from Phase 0,
  because §5 requires offline simulation to match online exactly and §6 requires
  every parse to be reproducible:
  1. **Seeded RNG with per-subsystem streams**, so adding an aberration roll
     cannot perturb the parser's stream.
  2. **`orbs-sim` owns its own `Schedule` and advances through a single explicit
     `step(&mut World, tick)` entry point** that frontends call. The Bevy app and
     its plugin scheduler never drive the sim — a frontend is a caller, not a host.
  3. **The sim schedule runs single-threaded** —
     `Schedule::set_executor(SingleThreadedExecutor::new())` (`ExecutorKind` was
     removed in 0.19). Bevy's multi-threaded executor does not guarantee ordering
     between systems without explicit constraints, which is fatal for a sim that
     must replay identically and match offline to online. At 1 Hz and this entity
     count, parallelism buys nothing and costs the property everything else rests
     on. If the live game and the CLI harness diverged, we would not find out
     until Phase 9.
- **Balance harness (Phase 1):** an `orbs-sim` CLI running a scripted synthetic
  player for N simulated hours, dumping §11.5's curves to CSV. Converts balance
  from vibes into sweeps.
- **Parser instrumentation (Phase 0):** full input, resolution, and candidate-score
  capture with export. Required by the Phase 0 gate.
- **Rendering:** cell-grid text renderer (glyph atlas + instanced quads) with
  integer fidelity tiers (§4, §9), CRT post-process ported from court_wizard and
  made cell-size-aware.
- **Save format:** plain text/TOML — on-theme and debuggable; invites external
  editing, which is accepted (§15).

## 14. Accessibility

**At launch:** CRT fully disableable (motion sickness — court_wizard ships a health
warning), multiple phosphor themes with colourblind-safe accent triads, contrast
options, font size and cell scaling, **multiplex display mode — Deep or Wide focus
(§9)**, no meaning conveyed by colour alone
(including sabotage tells, §8.1), full remapping, echo verbosity toggle, and **no
mechanic requiring fast typing at base difficulty** — mechanised via one-command
diagnosis (§5.1), not merely asserted.

**Post-launch: first-class screen-reader support.** A text-only game is one of very
few genres that can be natively blind-accessible — a real differentiator, a press
hook, an underserved audience.

**The terminal frontend (§13) is the cheapest route to it, but not a launch
commitment.** Terminals are already accessible; Orca, NVDA, and VoiceOver read
terminal output natively, so a shipped `orbs-tui` would deliver much of this
almost for free. But that frontend is second-class and cuttable, so the promise
cannot rest on it. The linearisation architecture is built into `orbs-render`
regardless, which keeps both routes open: a bespoke Bevy accessibility layer, or
the terminal build, whichever the schedule reaches first.

**Constrains architecture now:** linearised representation for panes, tables, and
progress bars; eldritch messages logged and carrying authored linear variants;
echo tagged as metadata; sabotage tells command-detectable, not merely visual.

**Progress announcements: completion only.** At endgame a player may have ~25
in-flight duration-actions; announcing every progress update would be unusable.
Screen readers announce **completions only**, and `status` enumerates everything
in flight on request.

**The real-time siege needs its own accessible mode, decided now.** Text-only
games are natively blind-accessible when they are *turn-based*. Speech is serial
and slow; a multiplexed three-pane siege emitting concurrently against a
wall-clock is not consumable that way. **In screen-reader mode, siege ticks
advance on player input rather than wall-clock.** The siege becomes turn-based for
those players — preserving every decision (triage order, diagnosis, repair) while
removing the one dimension speech cannot serve. Deciding this before the siege is
built is what makes "designing for it is nearly free" actually true.

## 15. Scope and phasing

Solo, commercial, Steam. **Plan for two years.** Release posture: **demo first,
then full 1.0** — no Early Access.

**The slice is brewing + archive — the two starting domains.** Phase 0 must build
what the player actually meets first, or the gate tests a configuration that never
ships. Brewing is the most legible subject for the parser gate (a shell-naive
tester immediately understands "make a potion"); archive is mandatory because
`bind` is gated on progression, so without a second earning domain there is no
bound script inside the first session. (**Amended §19**: `bind` is bought with
*experience* rather than researched with fragments, and the archive is mandatory
for the same reason under either currency — it earns, and half the opening would
be dead progression if it did not.)

**Sabotage is still tested, without a third domain.** Log poisoning is exercised
against *brewing* logs using `peruse`, `sift`, and `verify` — the log-reading
primitives, not the full scrying domain. That covers the "siege is empty" Critical
risk in Phase 0 while keeping the slice at two domains. Scrying becomes the
player's first discovery and lands in Phase 1.

### The playability gate — every step, every phase

> **No work item in any phase is complete until a person can reach it from the
> running game.** Every item carries a **See it** line naming the keystrokes that
> demonstrate it. An item without one is not started; an item whose line does not
> work is not finished, however green its tests are.

This has the same standing as the build gate in `CLAUDE.md`, and applies for the
rest of the project — not as a Phase 0 correction. Phases 1 through 5 add
systems whose failure modes are *felt* rather than asserted: a script that runs
but feels arbitrary, a siege that is correct but not tense, an economy that
balances but bores. Those are invisible to a test suite by construction, and the
only instrument that finds them is a hand on the keyboard.

**Ordering follows from the gate.** Where a phase's items can be sequenced so
that earlier ones become the instrument for later ones, they are. In Phase 0 that
means **the prompt comes before everything else that remains** — a real command
line, with the parser behind it and its output on the tube, is not a milestone in
its own right but the thing domains, sabotage, boot, and the tutorial are all
built *into*.

**Retroactive gating is a work item, not a cleanup.** Anything already built
without a gate gets one before new work continues past the prompt. Code the game
does not call is not verified by being tested; it is only asserted.

#### What this cost, and why the rule exists

Recorded in §19. Phase 0's first six items ran in architectural-layer order —
determinism spine, Frame boundary, parser, cell renderer, CRT port, record model.
Every one was completed, tested, and reviewed. After six of them the binary
answered four keys, **none of them a letter**: ~10,000 lines of Rust, of which
the game called under 40%. A sixteen-command parser with ninety-odd tests had
never received a keystroke, and a record model built to be drawn had never been
drawn.

Nothing there was wrong, and that is the point worth keeping. **Tests prove code
does what it was written to do; they cannot prove it is the code worth writing.**
An API with no callers is unshaped — `report()` needed a parameter no test wanted
and no design document predicted, surfaced by review rather than by use, which is
luck rather than method.

The counter-example is the CRT, the one item that *was* player-gated on arrival:
boot the game, press F3. It flashed, the flashing was visible in ten seconds, and
two wrong diagnoses were falsified by looking rather than by reasoning. No test
in the suite would have caught it, and none was ever written that could have.

### Commercial

- **Price: $4 (provisional).** Deliberately impulse-tier for a premise that is
  hard to convey in a capsule, and a deliberate audience-building play for
  Blackhearth's catalogue. Note the trade honestly: at ~$2.50 net per unit after
  Steam's cut and tax, matching a $14.99 return takes roughly four times the
  units. Revisit at Phase 10 when content volume is visible.
- **Positioning: Duskers + Zachtronics.** Lead with the terminal atmosphere —
  Duskers proved players will type commands into the dark and love it — and
  support with automation depth for the Exapunks/Shenzhen I/O audience. The two
  audiences overlap heavily. The capsule leads on mood; the trailer leads on
  motion, because CRT motion is the hook a static image cannot carry.
- **Saves are readable and editable, achievements unguarded.** Single-player;
  hand-editing a TOML file only affects the person doing it, and readable saves
  are both on-theme and a debugging asset.

| Phase | Goal | Words | Cal. | Exit criterion |
|---|---|---|---|---|
| **0. Vertical slice** | Parser + instrumentation, **the prompt — an interactive command line, built before the domains and used to verify them**, **brewing + archive (the two starting domains)**, **`orbs-render` Frame boundary**, **log-poisoning sabotage on brewing logs**, cell-grid renderer + fidelity tiers, worst-case CRT legibility test, structured-record output model, seeded-RNG + `step()` determinism spine, boot, scaffold tutorial | ~3k | 4 mo | Numeric gate below |
| **1. Core loop** | World clock, script engine + **concentration** + failure taxonomy, remaining sabotage surfaces, 3 domains, minimal apprenticeship, content data format, balance CLI **sweeping §11.5's first-pass numbers**, **scrappy internal `orbs-tui` as a dev tool** | ~15k | 5 mo | A player automates a duty and feels clever; non-terminal testers in the loop |
| **2. Scrying** | The `lens/` domain: deduction over sources that disagree. **World sabotage surface**, **`orbs-balance` sweeping §11.5's first-pass numbers** | ~6k | 3 mo | A player finds which of two accounts is lying, and a spell repeats it |
| **3. Spellcraft** | The `grimoire/` domain: spells composed from reusable parts. **Naming pass (~35 verbs)**, hidden-directory authoring | ~6k | 3 mo | A spell is assembled from parts written earlier |
| **4. Enchanting** | The `forge/` domain (derived): sequence + cost → a decaying buff on an instrument. **Shared-engine extraction**, pulled forward from 9a | ~4k | 2 mo | A buffed instrument is visibly faster and the buff runs out |
| **5. Summoning** | The `menagerie/` domain (derived): allocation → a unit with a standing rule | ~4k | 2 mo | A summoned thing acts without being told to that tick |
| **6. Defense** | The `battlements/` domain: wards under 1 Hz pressure. **Reflex-avoidance mechanism decided first**; **accessible mode decided here, not at 11** | ~5k | 3 mo | A pressure survived by choosing, not by reacting |
| **7. One machine** | Reagents cross domains, **Focus-slot reservation**, pane addressing, the authored edges between domains | ~3k | 2 mo | The tower runs itself across rooms |
| **8. Siege** | Autobattler, trait composition, adversarial aberrations, escrow economy, **unattended-siege backlog + dispersal**, pane addressing, one siege type, drift stub, synergy template | ~15k | 4 mo | Sieges are tense and scripts visibly matter |
| **9a. Breadth** | Discovery/research, full drift, **offline progression + its unlock**. *(The five domains and the shared-engine extraction moved to Phases 2–7.)* | ~4k | 2 mo | Discovery closed |
| **9b. Remote hosts** | The second content type: trees, verbs, infiltration, trace amplification. **`orbs-tui` to ship quality, if the schedule allows** | ~12k | 3 mo | Infiltration loop closed |
| **9c. Engine upgrade** | Bevy version window — whole-codebase, isolated from new-system work | — | 1 mo | Green on all three platforms |
| **10. Onboarding + demo** | Polish apprenticeship, progressive reveal, grimoire, soft ending, **demo + capsule + trailer** | ~20k | 4 mo | A non-terminal player reaches hour two unaided |
| **11. Ship** | Accessibility, screen-reader siege mode, options, Steam (**both launch options**), polish | ~5k | 3 mo | Release |

**Total: 41 months of work against a 24-month target**, where it was 28 before
the domains were given phases of their own. That gap is deliberate and should be
read as the buffer being *already spent* — it is the number to attack with the
cut line, not a scheduling error to hide. §16's "per-phase scope buffers"
mitigation is only real if the calendar exists to measure against.

**The 13 months are an admission, not an inflation.** Five domains and the
extraction task were budgeted at 4 months and ~18k words *combined*, filed as one
line inside a breadth phase; 9a shrinks by exactly that as they leave. Giving
each domain a phase says plainly that the old figure was priced for a table entry
rather than for five minigames — which is the same correction §10 makes about its
own table: *"these forms are a table, not a design"*. A 41-month plan that is
honest is more useful than a 28-month one that was never going to hold, and the
cut line below is where the difference gets argued.

**Phase 9 is split three ways.** Draft 5 loaded five domains, discovery, full
drift, the entire remote-host content type, the engine upgrade, and the extraction
task into one phase. The engine upgrade in particular must not share a phase with
new-system work — it touches everything.

**The terminal frontend is split into two very different costs.**

The **`orbs-render` boundary** lands in Phase 0 and is non-negotiable. It cannot
be retrofitted — if it is not load-bearing while the codebase is small, the Bevy
shell quietly accumulates presentation logic no other frontend can reach, and the
terminal build becomes a rewrite rather than an addition. It is also nearly free,
being the same discipline that keeps `orbs-sim` headless.

A **scrappy internal `orbs-tui`** lands in Phase 1 as a *dev tool*, not a product.
It has no parity, polish, or support obligation. Its job is to prove the boundary
is real and to give the parser and balance work a no-GPU, no-window,
instant-startup, trivially scriptable target. It pays for itself in iteration
speed on the two things Phase 1 is built around, which is why it costs no
additional calendar.

**Ship-quality `orbs-tui`** sits in Phase 9b and is explicitly cuttable. The Bevy
build never waits for it.

**Two things moved into Phase 0.** The naming pass for the slice's 16 commands (now done — §6.1),
because the gate's first-attempt metric substantially measures how *guessable* the
names are, and testing provisional names would invalidate the result. And one
sabotage surface (log poisoning — cheapest and most legible), because §16 claims
Phase 0 de-risks "siege is empty" and it cannot do that with a scrying stub and no
sabotage at all.

**The demo sits in Phase 10** because its purpose is validating onboarding, and a
text-game demo is almost entirely apprenticeship and progressive reveal.

**Running total: ~88k words.** If a phase overruns, the vocabulary budget (§11) is
the first lever.

### Phase 0 gate — numeric, defined now

> **≥ 8 external testers, at least half self-reporting no shell experience,
> complete a 15-minute scripted scenario with expected-intent ground truth.**
> - **≥ 85% of inputs resolve to the intended action on first attempt** (below 70%
>   is a no-go), **and**
> - **≥ 95% of initially-unresolved inputs reach the intended action within two
>   further attempts, with zero dead ends.**

The second metric matters more than the first: a parser at 85% whose failures are
dead ends is worse than one at 75% that always recovers.

The gate measures the parser *given a scaffold tutorial* — a Phase 0 deliverable,
thrown away afterwards — so a failure is diagnosable as parser-vs-onboarding
rather than confounded. Numbers set before results exist; a gate authored after
seeing results is not a gate.

**Read the clustering, not the aggregate.** Eight testers over fifteen minutes is a
small sample with wide confidence intervals around 85%. It is sound for a go/no-go
on a catastrophic outcome (below 70%), but 84% versus 86% is noise. The artefact
to act on is the instrumentation export (§6, §13) showing *which inputs* failed and
how they cluster — that is what tells you whether the problem is a verb name, a
missing synonym, or the scoring function.

### Cut line — decided before it is needed

1. Steam Workshop (already deferred)
2. OS-window pane detaching (already deferred)
3. **Ship-quality `orbs-tui`** — the boundary and the dev-tool build are kept
   regardless; only the polished, supported frontend is dropped
4. **Remote hosts** reduced to read-only scrying feeds rather than navigable
   filesystems
5. Enchanting, then Summoning (derived domains — the extraction task is cut with
   them)
6. Screen-reader support slips further post-launch (architecture retained)
7. Multiplexing capped at two panes (may become the default — see §4)

**Never cut:** the parser, the script engine, the aberration system, one excellent
siege, onboarding.

#### How a domain phase is cut, now that item 5 is two numbered phases

Item 5 used to name two entries inside a breadth phase. They are **Phases 4 and
5** now, sitting *ahead* of the never-cut siege, and cutting a numbered
mid-sequence phase is a different and larger operation — it would re-trigger the
renumber that this restructure just paid for.

So the rule is: **a cut domain phase is emptied, not removed.** Its number stays,
its heading stays, its items become a one-line record of what was dropped and
why, and nothing after it moves. That keeps the cut cheap, keeps every
cross-reference valid, and keeps the decision visible in the document rather than
inferable from a gap in the numbering.

**And the order of the five is itself a hedge.** Scrying and Spellcraft are first
because they are the two whose absence would be felt everywhere — sabotage stays
illegible without one, and the language stops growing without the other. The two
**derived** domains sit at 4 and 5 precisely because they are the cut candidates:
late enough that cutting them costs the least, early enough that the extraction
task they carry lands before Phase 7 needs it. If both are cut, the extraction
moves to 7 and 7 absorbs it.

### Naming pass

The canonical command set is player-facing API and requires a dedicated in-world
naming pass before Phase 1 freezes vocabulary.

**The Phase 0 pass is done** (§19) and is now a test — `crates/orbs-sim/tests/
naming.rs`. Phase 1 adds ~35 commands, which is exactly when a vocabulary drifts
back into collision, so the rules are enforced continuously rather than
re-audited by hand.

## 16. Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Parser feels frustrating rather than magical | **Critical** | Phase 0 gate with recovery metric; instrumentation; forgiving pipe stages |
| Siege is empty once scripts are good | **Critical** | Aberration system prototyped Phase 0; four-surface sabotage; trait novelty |
| Onboarding loses non-terminal players | **Critical** | Three layered approaches from Phase 1; continuous external testing; demo in Phase 10 |
| **Writing volume overruns the schedule** | **Critical** | Composition over authorship; per-phase word budget; vocabulary as first lever. court_wizard's speed came from externalised art — unavailable here |
| Economy is wrong or untuneable | High | §11.5 settled with first-pass numbers and worked sinks; balance CLI sweeps them in Phase 1; one headline scarcity per phase — Focus in the tower, mana in the siege |
| Seven domains + remote hosts spreads thin | High | Concrete-first with a budgeted Phase 9 extraction; cut line |
| CRT over text is illegible | High | Phase 0 worst-case test at min resolution and max font scale |
| Sabotage feels unfair rather than solvable | High | Every surface has a structural tell AND a one-command `verify`; log poisoning always leaves a trustworthy source |
| Bevy 0.19 goes stale over two years | Medium | ~4-month cadence means shipping 5–6 versions behind; one deliberate upgrade window in Phase 9, budgeted |
| Second frontend divides effort | Low-Medium | Bevy is the product and never waits. `orbs-render` owns presentation, so the boundary is cheap and reusable. Ship-quality TUI is cut-line item 3 |
| Solo continuity — illness, burnout, life | Medium | Cut line defined in advance; minimum shippable state named; per-phase scope buffers |
| Market: text-only is hard to sell | Medium | CRT *motion* is the hook — trailer over capsule; demo as primary conversion tool |

## 17. What this game is not

- Not a Linux tutorial. Terminal literacy helps but is never required.
- Not a typing game. Mechanised via one-command diagnosis, not merely asserted.
- Not illustrated. No sprites, no portraits, no key art beyond the screen itself.
- Not multiplayer.

## 18. Live open questions

**Nothing blocks Phase 0.** The slice vocabulary is named (§6.1), the starting
domains are settled (brewing + archive), and the fragment trickle has a rate
(§11.5).

**Blocking Phases 2–7** *(these blocked Phase 1; it closed and they did not, so
they are re-pointed at the phases that now need them rather than quietly dropped)*

1. Trace tuning across all three sources — accrual rates, the nuisance-rate
   composition rule, the ceiling, and the provocation threshold (§5.3).
   **→ Standing.** It accrues with content and has no state in which it is done.
2. Naming pass for the remaining ~35 canonical commands and their synonym sets,
   following §6.1's ≤7-character rule. **→ Phase 3**, ahead of the domains that
   coin the most verbs.
3. Hidden-directory authoring plan — ~80 fragments' worth, placed to pace the
   first ten hours. **→ Phase 3**, which is what spends fragments.
4. ~~What the brewing recipe puzzle actually is.~~ **Done at the head of Phase 1**,
   and §10.1 is the answer: four stages, a shared fire, and *light → digest → damp
   → combine → relight → distil* as a window at 1 Hz. It is now the worked example
   Phases 2–7 are cut from, which is what this item asked for.
5. **Nuisance aberrations are unscheduled in every phase.** §5.1 calls them *"the
   core of manual play"* and Phase 8 lists only the adversarial ones; Phase 0
   shipped log-poisoning drift and nothing else, and `RngStream::Aberration` has
   never been rolled. **→ Phase 2**, and this is the phase that fixes it: scrying
   is deduction over things that have gone wrong, and a domain built to read
   sabotage with nothing generating any is a puzzle with no inputs. Still a gap
   rather than a deferral — it just now has a phase that cannot ship without it.

**Blocking Phase 8**

6. The 21-pair synergy template: which mechanical parameters it exposes, and the
   per-pair tuning values.
7. Siege type definitions, their failure conditions, **and how completion fraction
   is computed for each.** Elapsed/target works for survival timers; objective
   defence, integrity collapse, and cascading failure each need an explicit
   definition or escrow cannot settle.
8. Difficulty-tier definitions within the progression-gated range.

**Commercial, before Phase 10**

9. Wishlist target, and whether $4 survives contact with actual content volume.
   Note that $4 forfeits discount room (a 50% sale is $2) and may read as a
   smallness signal to the Zachtronics-adjacent audience, which does not
   price-shop — Exapunks is $19.99.

## 19. Decisions log

### The save format — a snapshot, with the journal as its instrument (Phase 3, `0.3.14`)

**Nothing in the workspace serialised anything.** ROADMAP said it twice from
opposite directions: *"`Running` is not in any save format, because there is no
save format"*, and the settings item sitting in Phase 11 because *"no `serde`, no
`toml`, nothing in the workspace serialises anything yet."* Close the game and
the tower was gone.

§13 and §15 had already settled most of the shape — `serde` + `toml`, readable
and editable, saves at tick boundaries only, in-flight duration-actions as
first-class serialisable entities — so what follows is the part they left open.

| Question | Decision |
|---|---|
| Snapshot or journal | **Snapshot.** `(seed, submissions)` is exactly true and had been recorded since Phase 0 with no consumer, but a journal is invalidated by any content patch and "editable" would mean editing a command log. §13 and §15 chose TOML state and are not re-litigated |
| So what is the journal for | **A determinism check.** `two_routes_to_one_world_write_the_same_save` reaches one world by replay and by play and requires identical bytes — far more world state than the four message-stream replay tests that already existed, and the first consumer `(seed, submissions)` has ever had |
| What catches a component nobody carried | **The lockstep test, and this was got wrong first.** The journal test runs `capture` on two worlds *neither of which was restored*, so a field `capture` omits is missing from both documents and it passes. `a_loaded_tower_keeps_running_the_same_world` is the instrument, because it is the only one that restores: drop `Burning` and the loaded athanor is cold, the heated stage never lands, and the divergence surfaces in fields the document does carry. The two completeness lints narrow the rest |
| Is there a `save` verb | **No**, and three reasons. §8 already specifies autosave; §19's editor precedent deleted `save` as a word for the same chore one level down; and **`save` and `sage` are one edit apart**, which is the collision that refused `leave` and `exit` as `quit` synonyms |
| How a node is addressed | **By path, never by `NodeId`.** An id is a counter in spawn order, stable only while `build.rs`'s tables are. A path is what the player types, survives a phase that adds a domain, and makes `subject = "/tower/laboratory/dispensary/sage"` legible. It is also what lets the lockstep test compare two documents at all — under id-keying a lived world and a rebuilt one hold different ids for the same nodes |
| Whole tree, or a patch | **Raise first, then adopt.** `Sim::restored` runs `tower::raise` and reconciles by path. An authoritative save is simpler and wrong on a schedule: five domains arrive between here and Phase 9a, and every pre-existing save would open into a tower permanently lacking them. Refusing the old save by format version is not an answer — it deletes it |
| What the RNG carries | **The master seed *and* eight word positions.** `Rngs::master_seed`'s doc already claimed a world could be reconstructed from the seed; it could not. A `ChaCha8Rng` advances in place, and `sabotage`'s two systems each draw once per tick unconditionally, so a rewound stream re-runs the session's whole schedule of drifts and swaps. `the_seed_alone_would_rewind_every_stream` pins it |
| Offline progression | **Not built.** §5 opens *"initially there is no offline progression"* and puts accrual in Phase 9a. The save carries a **departure stamp** — `[away]`, stamped by the frontend because the sim has no wall clock and must not acquire one — so Phase 9a is a policy change rather than a format migration. A departure time cannot be recovered after the fact |

#### `tower::drift` picked its target by query order, and only a rebuilt world could see it

`logs.iter().next()` is archetype order, which in a **lived** world is a function
of which log was poisoned when — inserting `Poisoned` moves an entity between
tables and `swap_remove`s its row — and in a **rebuilt** one is simply spawn
order. Same seed, same tick, different log.

`substitution` fixed exactly this one function down and said so: *"**Sorted by
name, not query order.** …a replay has to swap the *same* pile from the same
seed."* `drift`'s own comment recorded the debt — *"it was the newer of the two
systems and only it got the fix"* — and **nothing had ever rebuilt a world**, so
nothing could observe it. The save format is what made it observable, and it is
sorted now, with the index drawn from the roll that already fired so the shared
`Threat` stream takes no extra draw.

#### The transcript is not a transcript, and dropping it would have half-broken §8.1

The first draft of this work saved no records at all, reasoning that closing a
terminal loses its scrollback. That is wrong here: **a `.log` is a view over the
record stream**, not a file with contents of its own — `tower::node` says so
outright — so dropping the stream empties every log in the tower. Worse,
`sabotage::emit_lines` builds the poisoned tell by *re-emitting existing lines*
with a field struck out, so a restored tower with a `Poisoned` log would answer
`verify` with *tampered* and then show the player nothing at all.

So a **bounded tail** travels (500 records), built in `orbs-sim` over `Records`'s
public API — `orbs-render`'s `[dependencies]` stays empty. What is *not* bounded
is `Records::sequence`, which rides in `[world]`: a running spell's cursor is a
position in that sequence, and `Records::resume` reopens the stream at the count
it is not carrying so `dropped` reports the gap. Drop the counter instead and a
held spell silently stops seeing anything, with every test green.

#### A restored world is the saved world, exactly — so it prints no banner

`Sim::restored` deliberately does **not** call `tower::report`, where
`with_schedule` does. The round-trip test settled it: `report` pushes §4's
condition report onto the stream `restore` has just rebuilt, so a loaded world
carried thirty-odd records the world that wrote it never had.

The contract is worth more than the banner, and what a player sees on waking is
better than a boot report anyway — the save carries the tail of the stream, so
the transcript is the screen they left. Saying *"the orb was dark for three
hours"* is a frontend's line, because the away stamp is a frontend's.

#### A resumed spell verifies its program before walking it

`Running::program` is a derived view rebuilt from the spell's text at every cast,
so the first draft excluded it. But `pc` is a **path into the tree the spell was
cast against**, and two things can have moved since: the spell's own text, since
editing one mid-flight is shipped, and a reagent's name, since §8.1's
substitution surface is the point of the domain. Walk a stale `pc` into a freshly
compiled tree and the orb runs the wrong line, or `program::at` returns nothing
and the run ends with no reason given.

So the save carries a fingerprint of the text the program was compiled from, and
a mismatch **ends the run and says so** — §8's *"scripts always log and never
halt"* forbids halting quietly. A `Bound` spell is cast again on the next tick
from the top, which is the correct recovery and needed no code.

#### The completeness lint keys on `TypeId`, and its reach is its world builder's

A snapshot save has one fatal failure mode and it is silent: a component added in
a later phase that nobody serialises. Every round-trip test still passes, because
neither world has the field.

`every_component_the_world_holds_is_one_the_save_knows_about` walks
`world.archetypes()` and fails **by name**. Two things about it are worth
recording rather than rediscovering:

- **`ComponentInfo::name` is unusable here.** It returns a `DebugName`, which
  without `bevy_utils`'s `debug` feature is the literal string `"<Enable the
  debug feature to see the name>"` — and `crates/orbs` builds Bevy with
  `default-features = false`. A lint keyed on it prints the same placeholder for
  every entry and names nothing, which is the one thing it has to do. It keys on
  `type_id()`, and `orbs-sim` takes `bevy_ecs`'s `debug` feature as a
  **dev-dependency** so the message can carry a name under `cargo test` without
  reaching the shipped binary.
- **It only sees what is present.** Twelve components are conditional — a `Ward`
  exists while a reading is open, a `Quickened` inside a window — so the lint is
  exactly as strong as the world it is pointed at. It is pointed at the same
  builder the lockstep test uses, and
  `the_test_world_actually_holds_everything_it_is_meant_to` guards *that*, having
  already caught the builder reaching its snapshot with **no work in flight at
  all**: the grind was started twenty ticks too early, and every assertion in the
  file had been comparing two worlds where nothing was happening.

#### What a review of the built format found

Three defects, each reproduced before it was fixed, and each invisible to the
suite that was green when they were found:

- **A renamed node changed slot on the way back in.** `sabotage::substitute`
  renames a pile in place — `sage` becomes `sage-` — so its saved path matched
  nothing the raised tower had; it was spawned and appended while the raised
  `sage` was swept. `tower::node` opens by saying anything a player can see must
  come from walking `Children` in insertion order, because §6 resolves noun ties
  to whichever was registered first, so a reload silently changed what an
  ambiguous phrase resolved to. `Children` order is now rebuilt to the
  document's. The test that pins it had to **step to a real ambient swap**: the
  first version used `debug_swap`, which takes `charcoal` — last in the
  dispensary already, so re-appending it was invisible — and passed with the fix
  removed.
- **A record's register was written to the save and never read back.** §8.1's
  poisoned-log tell is `Presentation::Tampered` on every third line, so a reload
  answered `verify` with *tampered* and then drew the log clean.
- **`purge` on a shipped spell was undone by the next load**, in a release build.
  The sweep despawned only `Stock`; a spell is neither stock nor a fixture and
  fell in the gap. It now sweeps anything destructible that is neither
  `Protected` nor `Fixture`, and the cost is stated: a *destructible* thing added
  by a later build is swept the first time an older save opens.

Also corrected: a hand-edited ward could seat a sigil index past the end and
panic on the next `probe` rather than on the way in; `find_by_path("")` answered
with the nameless root, turning three "this path is gone" guards into guards that
silently resolved; and the node's own `NodeId` now travels, because
`spell::advance` orders running spells by it and a re-issued id would interleave
two player-written spells differently.

**The fixture failed `cargo test --release` and nobody had run it.** `debug_spawn`
and the dev-spell shelf are `cfg(debug_assertions)`, so the release world had no
experience, no bound spell and no quickening window — and the round-trip tests
went green on it. Six sibling test files answer that by gating the whole file on
`debug_assertions`; this one does not, because **the save format ships in
release** and that is the half that matters. The debug-only commands are
conditional and the coverage guard asserts only what the profile in hand can
reach.

**A resource lint was missing and the component lint could never have covered
it.** `Choices` — the numbered disambiguation prompt — is a resource and survives
until it is answered, so a save that dropped it left the question on the
transcript with no answer that resolved: §15's dead end, arriving through the
affordance `session` says exists to remove one.

**The save carries the line, not the readings.** An `Intent` is the parser's
resolved form with typed arguments, and putting it in the format would pin the
format against every future parser change — for a question that survives until
the next command. The line costs one string and reproduces the list exactly,
because §19 already settled that the parser's tie-break draws **no randomness**:
ranking is a total order over score, position in `Verb::ALL`, and the canonical
echo, so `analyse` is a pure function of the line and the scene.

That re-ask is its own step in the restore, **after the scene is rebuilt** —
`analyse` takes the `Scene` *resource*, which at the top of a restore is still
the empty default, so asking there resolved against a world that named nothing.
(`spell::compile` is unaffected: `tower::scene_at` computes a scene from the
world rather than reading the resource.)

**Two sorts were not total orders.** `drift` and `substitution` both sort by name
and `sort_unstable` promises nothing for equal keys — so two same-named logs
would fall back to the archetype order the sort exists to remove. Four domains
have four distinct log names today and five more arrive by Phase 9a. `NodeId` is
the tie-break, and it is only safe as one *because* the save now carries it.

**Every interval in the file is a start and a completion tick**, which is §8's
own wording. `Triaging` holds an end and `Burning`/`Quickened` hold a budget, and
one `SpanSave` used to carry whichever the component had in a field called
`ticks` — so the same field name meant a tick in one row and a duration two rows
down. Both round-tripped correctly; it was a trap for the reader, and §15 makes
the reader the criterion.

#### The file, and the three answers it can give (`0.3.15`)

`orbs-shell` owns the path, the write and the wall clock; `orbs-sim` owns the
document and touches no filesystem. Decisions worth keeping:

| Question | Decision |
|---|---|
| Where | `orbs-save.toml`, **beside the binary**, on `TRACE_PATH`'s argument. `dirs` is §13's stack and Phase 11's settings screen is where it arrives with Steam Cloud; until then this is one function rather than a path in two frontends. The cost is named: an install directory can be read-only |
| How | Written to `<path>.writing` and **renamed**. A save lands every sixty ticks for as long as the game is open, so a crash catching a half-written file is not theoretical; the rename turns *"the tower is corrupt"* into *"the tower is one minute stale"*. The scratch file is a **sibling**, because `rename` is only atomic within a filesystem |
| What a dump does | **Neither loads nor saves unless `ORBS_SAVE` names a path** — the opposite of the running game's default. Otherwise every See-it line in CLAUDE.md becomes order-dependent on whether anyone has played in that directory, and `scripts/dumps.sh`'s baseline stops being one. Both it and the played-game suite pin `off` besides |
| A save that will not open | **Kept, not deleted**, and said in voice. A later build may read it — the format refuses a *newer* file precisely so it is not half-read — and the next autosave overwrites it anyway. Losing a tower is bad; losing it silently and destroying the evidence is worse |
| Three answers, not two | `Opened::{New, Restored, Unreadable}`. A first launch and a tower that did not come back both end in a new world and are **not** the same thing to tell a player |

#### Leaving has four doors and only one of them is a word (`0.3.16`)

`quit` is a verb, and `F10`, the window's close button and the terminal's
`Ctrl-C` are not. None of the three touches the `Quitting` flag — `F10` writes an
`AppExit` directly and so does `bevy_window` — so **ordering a save against
`quit_requested` would have covered one exit in four** and looked complete.

The Bevy build reads `AppExit` in `Last`, which is the one place every route has
converged by. The terminal's loop became a function of its own so its caller has
the same single place; its four exits include an `io::Error` off the terminal,
which no flag could have carried at all.

This is `quit`'s own §19 entry read from the other end. That one records the
first attempt checking the flag at `submit` time and doing nothing; this one is
the same mistake one level up — checking the *word* rather than the act.

#### The orb says it remembers, and the sim never reads a clock

Rule 6 puts the words in `prose.toml`; §19 forbids the sim reading a wall clock.
So the frontend measures the gap and hands over a number of seconds, and
`Sim::say_resumed` finds the sentence. **Not inside `Sim::restored`**, which must
hand back the saved world *exactly* — a banner pushed there puts records in the
loaded stream that the world which wrote it never had, and the round-trip test
compares the two documents byte for byte. Saying so is a thing a *session* does,
which is the line `quit` already draws.

The span is deliberately coarse — *"9 hours"*, not *"9 hours, 14 minutes and 3
seconds"*. A precise figure would imply the game had been counting, and §5 is
explicit that it has not: **nothing accrues while the window is closed.** The
sentence says so out loud rather than leaving it to be inferred.

#### `SimPlugin::persist` is a field rather than an environment variable

`ORBS_SAVE=off` exists and would have worked, except that `crates/orbs`'s own
tests build a `SimPlugin` and run it for hundreds of ticks — and under `cargo
test` the working directory is the crate root, so they would have read whatever
save was lying there and written one every sixty ticks. The environment is
per-process and `cargo test` runs threads, so one test's setting is every test's.

A field has neither problem and says what it means at the call site.

#### `cargo test --release` is not a gate, and one file was wrong about that

CLAUDE.md's gate is debug-only, and the only release lines anywhere are two
narrow ones — `--test debug_spawn` and `--test debug_spell` — both there to prove
a *door is shut* in a release build rather than to test the game.

**The convention is per-test, not per-file.** Seven test files carry
`#[cfg(debug_assertions)]` on individual tests and none carries it at the top,
which says the intent plainly: a test that needs a debug door is gated, and
everything else is expected to run in either profile. (An earlier draft of this
entry said six files gated themselves off wholesale. That was wrong, and the
distinction matters — it is the difference between "release is not tested" and
"release is tested except where a door is needed".)

`tests/arsenal.rs` failed in release because its one fixture opened with
`debug_spawn clarified-draught`, so all eight tests had been dead in that profile
since the file was written. **Gating them would have been the wrong fix**: the
arsenal is a shipped room, and a room whose only tests are debug-only is a room
untested in the build that ships.

**The symptom is what makes this class hard to see.** A debug door in a release
build does not fail — it is an ordinary unresolvable line, so the setup silently
does not happen and the assertion fails against a world that was never built.
`fetching` reports *"the potion never reached the arsenal"* and `secrets` reports
*"an unfound recipe read as ready to run"*; neither says the word `debug` and
neither is what is actually wrong.

The chain needs no door: sage, rock-salt and charcoal are all `Holding::endless`,
so §10.1's five stages brew a `clarity` from nothing in about fifty ticks. The
fixture now does that, which also makes it say what its own doc claimed — a
potion *carried out of the room that finished it*, having really been finished
there. A second lap needed the instruments scoured **before** use rather than
after, which is `orbs-balance`'s rule and the reason the second brew silently
refused: `mix` leaves the flask charged, so the next run's `mix` has nowhere to
land.

**Six files were affected in the end**, and the fix is a judgement per test
rather than one sweep. Where the feature ships, the fixture was made real; where
only a door can reach the state, the test is gated.

| File | What was done |
|---|---|
| `arsenal`, `fetching`, `binding` | **Brewed.** All three wanted a `clarified-draught` and reached for `debug_spawn`; §10.1's chain makes one from endless stock in ~50 ticks, so all three now test shipped behaviour in the build that ships it |
| `tampering` | **Waited for the real thing.** Three tests used `debug_swap`; they now step until the *ambient* swap fires, which is what CLAUDE.md says is the half that actually breaks. A fourth was **passing vacuously** in release — nothing was substituted and its `after > before` held anyway |
| `secrets`, `gleaning`, `ward` | **Gated.** A secret needs `debug_learn` and the roll is the mechanic; a scroll is four solved mazes; a broken ward is `debug_ward`. Each has an honest route that is hours long, and in `ward`'s case the coverage is not lost — `a_blind_ladder_breaks_a_ward_through_the_real_verbs` brute-forces one through the real verbs in either profile |

Two things worth keeping from doing it:

- **`debug_swap` is not the ambient surface**, and a test that uses it is testing
  a different pile. The shortcut takes the alphabetically-first endless pile,
  which is the charcoal; the ambient half **exempts fuel**, so charcoal is
  exactly what it can never take. `a_substitution_stops_a_spell_that_named_the_
  reagent` had to be turned around — the swap happens first and the spell is
  written after, naming whatever was actually hit.
- **A whole-crate release run costs one link, not seventeen.** Checking these
  file by file meant re-linking with `lto = "fat"` and `codegen-units = 1` each
  time. `cargo test --release -p orbs-sim` once answers the same question, and
  the debug gate — which is the gate — is 27 seconds.

#### The completeness lint reads two worlds, because one cannot hold everything

Its stated caveat is that it sees only components *present* in the world it is
pointed at. Measured, that blind spot covered six: `triaging`, `banked`,
`bidden`, `substituted`, `taken` and `marks` — four of them carried on faith,
written symmetrically in `capture` and `adopt` and checked by nothing.

Some are mutually exclusive with the busy tower by construction. `damp` requires
`Burning` and removes it, so an athanor cannot be lit and banked at once; a
spell-driven run and a player-driven one compete for the single production slot.
Contorting one fixture to hold everything would have made it hold each thing less
convincingly, so there are two, and both lints read both.

Three words that fixture had to be taught, none of which is what a reader would
guess:

- **Damping has no verb.** It is what `stop athanor` *does* — `pipeline` says so
  where it does it. A bare `damp` is ambiguous, and `damp athanor` fuzzy-matches
  `purge`, which is how the first version of this fixture ended up scouring.
- **`purge` on a log takes no triage slot**, because un-poisoning is instant.
  Only an instrument leaves a `Triaging` in flight.
- **`Bidden` needs a spell to have asked for the run.** The player's own `grind`
  never sets it, so the fixture invokes `first_light` instead.

#### The filesystem stays outside `orbs-sim`, and now a test says so

`orbs-sim` had never touched a file — content is `include_str!`'d and the prose
watcher lives in the frontend — but nothing enforced it, because nothing had ever
been *tempted*. `capture` and `restore` turn a world into a document and back,
and the obvious next line writes it to disk.
`the_sim_never_touches_the_filesystem` joins the three guards already in
`tests/boundaries.rs`, on the same argument §16 uses for the font asset: a rule
that is only written down is a rule that gets broken during a hurried phase.

### `quit` — the way out is a word, like every other way through

**Leaving was reachable only by a key, and no key in this game is
discoverable.** `F10` has always left; the terminal build added `Ctrl-C` because
raw mode makes it ours to answer. Neither is on screen anywhere. §6 makes this a
game played by typing, so a player who has learned twenty-five words for what
happens inside the tower has learned nothing about how to stop — which is
`unfurl`'s argument for `PageUp`, one level up, and the same answer.

It reads right at three depths: `quit` closes the spell editor, `quit` closes the
weave screen, and now `quit` closes the orb. One word meaning *leave the thing
you are in*.

| Question | Decision |
|---|---|
| Where it lives | The sim owns the **decision** and nothing else: a fifth take-once handshake beside `scribe`, `unfurl`, `weave` and `wander`. What leaving *means* is a frontend's, and the two answers have nothing in common — an `AppExit` here, raw mode being put back there |
| Determinism | Untouched. Leaving is not a world event, so `(seed, submissions)` replays identically whether or not anyone quit; a replay simply runs past it, which is what you want when the recording outlives the session |
| When it lands | On the **tick**, like every verb's effect. `submit` echoes and queues; the player sees the echo, then `quit_begins`, then the terminal comes back. A frontend checking at `submit` time would find the flag unset — the first attempt did, and did nothing |
| The tower-wide count | **22 → 23**, and `verb.rs` pins that number with an argument rather than a budget. `quit` is the one word the ceiling was never about: it acts on the *orb*, not the tower, and there is no fixture in any room that leaving the game could be scoped to. The ceiling still stands for the case it was drawn for — a domain verb arriving there is still the argument for building the missing mechanism |
| `leave` and `exit` are refused | Both fuzzy-collide — with `weave` and `edit`, one character each. Neither would *misresolve* (a claimed exact match beats a fuzzy one, which is `("list", "light")`'s rule), but the typo between them lands in an ambiguity prompt, and this is the one prompt that would offer **end the session** beside a verb typed all the time. Plain English arrives as a phrase instead: the collision check walks single words only |
| `audit`/`quit` is pinned | Unavoidable, because `quit` is the canonical name. Tolerated on the same terms: both claimed, two edits apart, different argument shapes — `audit laboratory` still reaches `verify` |
| The manual names no key | `man_quit_*` says what the word does and stops. Which keys exist is a frontend's answer and the two builds disagree, so prose the sim owns must not name `f10` or `ctrl-c` |

### `orbs-shell` — one shell, because there are two frontends now

**The painters needed `orbs-sim` *and* `orbs-render`, and neither may depend on
the other in that direction**, so for four phases they lived in the Bevy crate.
That was correct while there was one frontend, and this log even leaned on it:
the editor-ownership entry below rests its third confirmation on *"`orbs-tui` is
ten lines with nothing to diverge from."*

A real terminal build expires that argument. The choice is one shell or two that
disagree, and the failures would be the quiet kind — two line editors that treat
Tab differently, two rules for deriving the prompt name, two answers to *how many
records is a page*.

| Question | Decision |
|---|---|
| What moved | ~9,100 lines: every painter, `Editor`, `Tapestry`, `Line`, `Bench`, `Screen`, the POST card, the boot clock, the content read path, `seed`/`wizard`, and `ORBS_DUMP` itself |
| What stayed | Everything needing an engine: the camera and the 4:3 letterbox, the CRT, the glyph atlas, the phosphor palette, the system graph, and winit's press/release model with the held-key bookkeeping it forces |
| Dependency | `orbs-sim` + `orbs-render` + **`bevy_ecs`, never `bevy`** — rule 1 as written. The crate with the strictest headless requirement already takes exactly this dependency; it buys the `Resource` derive on the nine types the Bevy build keeps between frames and costs `orbs-tui` nothing, since `bevy_ecs` is in its tree via `orbs-sim` anyway. `tests/boundaries.rs` holds it, plus a third guard: **the shell never names a colour** |
| Not §15's extraction | ROADMAP's Phase 4 *"shared-engine extraction"* is one recipe table driving two **derived domains**, and is cut with them. This is frontends and has nothing to do with it |
| "The axis is only knowable from the second consumer" | §13's own principle, applied with `ORBS_DUMP` as the partial second consumer it always was — a complete headless frontend that built the real `Frame` from the real `Sim` with no `App`. That file is why this was a move rather than a rewrite |
| `pub` at the boundary | 142 items widened. CLAUDE.md's *"`pub` only for Plugin types"* is about a binary crate where `pub` means nothing; here the crate boundary **is** the frontend boundary. `missing_docs` cost nothing — all 142 were already documented — and `must_use_candidate` cost 58 attributes, which is the real price of the widening and was worth paying to keep the gate at strength |
| The gate | **Byte-identical dumps.** 56 screens captured before and after by `scripts/dumps.sh`, `diff -r` clean, 1,119 tests still passing. The only thing that may differ is the POST card's version line, because the step bumps it |

### The terminal build measures its glyph widths rather than assuming them

**CP437's symbols are East Asian *Ambiguous*, not Narrow.** `‼` `►` `☼` `○` `♂`
`♀` `♦` `♠` `Ω` `░` `▒` — under a CJK locale, or a terminal configured
*ambiguous = wide*, any one of them takes two columns. The Bevy build is immune
because it draws an 8×16 bitmap atlas and a glyph is one cell by construction.

A terminal is not, and the consequence is worse than ugly: one double-width cell
shifts the rest of the row **and** desynchronises the per-cell diff, because the
shadow buffer and the screen stop agreeing about which column is which.

So `orbs-tui` prints one at a known column at startup, reads the cursor position
back, and believes the answer — swapping in ASCII for the at-risk glyphs if it
comes back two. A terminal that will not answer is treated as narrow, which is
the common case and costs nothing if wrong on a screen nobody is looking at yet.

### A walk must land after its tick's step, or replay quietly diverges

**The terminal build's loop steps the sim before it drains the keyboard, and the
reason is `Sim::walk` rather than `Sim::submit`.**

A typed line is queued: `submit` records it against the current tick and hands
the command to the next `step`, so it executes at the start of tick N+1 whatever
order a frontend runs in. Loop order changes only *which* tick a wall-clock
keystroke lands on — latency, not replay.

`Sim::walk` does not wait for a clock. It executes immediately, and `Sim::replay`
already states the contract: a `Submission::Walked` *"already ran, during its
tick, **after** that tick's step."* Drain an arrow before stepping and the walk
runs before tick N's step while being recorded against tick N — and a replay
applies it after.

**Nothing fails.** The same number of ticks pass and the same squares are walked,
so a test comparing either is green; the first draft of the test that now guards
this compared ticks and proved nothing. What diverges is the *recording*, and
from then on the log describes a session that did not happen. Bevy gets the order
for free — `MainScheduleOrder` runs `RunFixedMainLoop` before `Update` — and a
hand-rolled loop gets nothing for free.

### What the terminal build gives up, and what it does not

Three degradations, all accepted and none informational (rule 2):

- **`Presentation` renders identically.** Eldritch and Tampered select a *face*
  in the glyph atlas, and the face here is whatever the user's terminal is set
  to. §8.1 already allows this by name: `verify` is the authoritative sabotage
  detector on every surface.
- **A mixed `Wash` takes its first tint.** The flask's `green+bone` band is the
  one region that is two colours becoming one, and sixteen indices cannot
  average. The band is still told apart by *position* — it grows from the fill
  end while both ingredients shrink.
- **No CRT, no phosphor, no blinking caret.** The terminal draws its own cursor,
  which is better: it is the one a screen reader tracks, which is why `Frame`
  keeps the cursor off `Cell` in the first place.

What it does **not** give up is the screen. `orbs-tui --dump` and `ORBS_DUMP`
print the same bytes, through the same painters, from the same `Sim`, and CI
diffs them on every push.

### Scrying is a code-breaker, and the first design deleted its own puzzle

**§10's form for this domain was one line — *"deduction: parse noisy logs to find
truth"* — and §10 says outright that the forms are a table, not a design.** The
mechanic is now a ward: a far wizard's orb sealed with four sigils drawn from
six, none twice, 360 codes. You press figures against it and read how it answers.
The stolen logs survive as the *yield* rather than the mechanic.

#### The vice, and why the obvious design cannot work

Measured over all 360 codes: **"press anything still consistent with the answers
so far" solves in 4.24 presses, and the best play there is manages 4.08.**
Mastermind's entire difficulty is bookkeeping, and a machine gets bookkeeping for
free — so there is no skill above the naive strategy to reward.

And `parser::question` is stateless: `Has`, `Is`, `Not`/`All`/`Any`, every operand
a literal written into the file. A spell's only memory is what the world writes
down. So:

| The world publishes | A spell can solve it | A player has a puzzle |
|---|---|---|
| the last answer only | **no** — two numbers it cannot turn into a guess | yes |
| the surviving candidates | yes, in three lines | **no** — the orb has done it |

**There is no useful middle.** A first design published a standing per sigil; it
buys 0.09 presses out of 4.24, and it means the orb is deducing.

#### The fix: two channels onto one ward

| | The player | A spell |
|---|---|---|
| Reads | `aligned`, `astray` | `closer`/`level`/`further`, `marks`, `settled` |
| Method | deduction | greedy hill-climbing |
| Presses | **4.14** | **22.8** |

Both use the same two verbs. *Did that help* is a fact the world can write down
without inferring anything, and it is enough to hill-climb on — so §8's language
automates a puzzle it could never solve.

**The player still wins, 5.5 to 1**, which is the margin the domain rests on and
is pinned by `deduction_beats_the_ladder`. A bound solver earns less per ward and
runs while you are in the laboratory, which is §8's argument for automation
landing through concurrency rather than speed.

#### The ratchet, and the settle-lock

**A press that does not gain snaps the aperture back.** This is the undo §8's
variable-free language cannot express: without it a blind ladder destroys the
sockets it has already got right and never converges. It never blocks correct
play — putting the right sigil in its own socket always raises `aligned` — so it
costs only the ability to hold a *worse* figure, and it makes a wrong press cost
ticks and nothing else (§11.5's *"cost is the resource, never progress"*).

**A gain settles a socket only when it was the only one that moved**, and both
looser rules were wrong. Settling everything ever touched locked the aperture on
the first gain. Settling everything touched *by this press* is worse and subtler:
with no repeats, dialling a sigil already in play exchanges two sockets, so a gain
of one settles both — and code `[0,1,3,4]` reaches three aligned with its fourth
socket locked wrong, unsolvable by any ladder. One socket moved and `aligned` rose
is **entailed**; that is the only claim the world may make.

A settled socket then **refuses** later dials, which is what bounds the walk. The
improving move a ward always has is provably reachable without disturbing one.

#### Names, and two words that did not survive

**`scry` is not a verb**, and §10's own word for the domain losing to a naming
rule is worth recording. `tests/naming.rs` forbids two canonicals sharing a
three-character prefix, having deleted its last exemption on the grounds that
*"an exemption that outlives its cause is how a guard quietly stops guarding"* —
and `scr` reaches `scribe`. `probe` opens a reading when none is open, which is
`grind`'s move-and-wield idiom; the fixture that existed only to carry the other
verb went with it. `seat` became **`dial`** to the same rule (`sea` reaches
`sift`'s `search`), and `dial` is the better word anyway: a ward is a lock.

**The first six sigils all failed a sweep**, `crown`/`cron` scoring 800 against
`bind`'s shell synonym — above the 750 the pinned set tolerates. They are now
`nitre`, `alum`, `borax`, `quartz`, `pewter`, `ochre`.

**And three readings leaked.** `gained`/`held`/`lost` are `NounKind::Sense`, which
`NounKind::Any` reaches — so `purge grind` fuzzy-matched `gained` and answered
*"there is no gained within reach"* from the laboratory. Ordinary English
participles sit in the way of half the words a player might mistype; comparatives
do not. `open` went the same way for a plainer reason: it is `peruse`'s own shell
synonym. They are `closer`/`level`/`further` and `loose`.

#### Two entanglements the lens pulled apart

**`is_operation` is not "does this take the production slot".** It answers *is
this word scoped to one instrument*, and `spell::block::begins_work` was reading
it as the other question. `dial` is scoped and schedules nothing; left alone a
spell's `dial first nitre` would queue behind a brew and burn `PATIENCE` doing
nothing.

**And the slot question itself is where the lens differs from the archive.** §19
refuses the slot to a maze because a solve is hundreds of ticks and a solver
holding it would starve every other spell. A press is twelve ticks and gives the
slot back between presses — so the lens can honour ROADMAP's stated scarcity,
*"a read is not a brew"*, where the archive could not.

**A ward between presses is `charged`, not `fouled`.** It holds no stock, so the
panel's fall-through read *"holding only what the last run fouled it with"* and
the prism reported a mess it did not have.

#### What a solve pays

`prism = 8`, the alembic's tier, and the yield curve is much gentler than the
first draft's. Halving per press past par lands a 23-press ladder on a floor of 1,
while it *already* pays 5.5× the ticks — the penalty double-counts and drives
automated scrying below the archive's maze. Three quarters beyond par, and the
tick cost does the real work: **0.161 XP/tick hand-played against clarity's 0.170,
0.022 for a bound solver against the maze's 0.006.**

### The world sabotage surface — §8.1's second of four

**A reagent is substituted: the name changes and the identity does not.** That is
the whole mechanism, and it is what §8.1's world-state row already specifies —
*"reagents swapped… substituted entities fail ID check → `Referent missing`"*. A
spell resolved the reagent at cast to a stable id; the id still points at the
pile and the *name* no longer matches, so the spell stops working for exactly the
reason §8.1 says it should.

It is the mirror of how a poisoned log works. **Nothing stored is destroyed**, so
the tampering is recoverable, comparable and `verify`-able — the same argument §3
makes for a poisoned line being re-emitted rather than rewritten.

**Both of §8.1's channels, because either alone is a defect.** `survey` shows the
odd one out on the shelf, carried by the *name* rather than by a colour; `verify`
finds it in one command and **names it**, which §8.1's design rule requires —
*"the skill is knowing which surface to inspect, not deciphering an obscure
clue"*.

**A place answers for what stands in it**, one level deep. Verifying a shelf and
being told `sound` while a swapped pile sat in it would be the surface reporting
the container rather than the contents; recursing the whole tree would be
`verify --all`, which §8.1 prices as Production-class work.

#### Endless base stock only, and the restriction is the design

A first pass took any pile at all and swapped `ground-sage` sitting between a
grind and a digestion. **That does not misdirect a player, it destroys work in
flight** — against §5.1, which keeps environmental damage in the calm layer and
leaves misdirection as the thing to see through, and against §11.5's *"cost is
the resource, never progress"*.

A base reagent is the honest target for the same reason it is endless: the tower
always has more, so what a swap costs is the **spell that named it** and nothing
half-made. It is also the thing a spell names most, which is what makes the
sabotage worth finding.

It announced itself by breaking `meditating_stalls_at_a_stage_boundary` — a test
about the pipeline with nothing to do with sabotage. A nuisance that can reach
into a running brew shows up as noise everywhere, which is the signal that it
reaches too far.

#### A second system, not a branch, and the reason is the stream

`drift` and `substitution` both draw once per tick from `RngStream::Threat`.
Interleaving two rolls inside one system would make *which* surface is hit depend
on how many draws had happened before it — so adding the world surface would
silently invalidate every existing log-poisoning replay. Two systems each drawing
once is one more draw per tick and no reordering of what either sees, and the
second is **appended** to the schedule for the same reason a stream index is
never inserted.

Swaps are four times rarer than log drift: a poisoned log misleads one reading,
and a swapped reagent stops a spell.

**Two of §8.1's four surfaces now ship.** Script text and trigger clocks stay in
Phase 8, where §5.1 puts adversarial aberrations — with no siege they have no
producer, and an item that cannot close does not belong in a numbered phase.

### The solver spell, and the two words it taught the lens

`debug_spell breaking` is 24 rungs, one per socket and sigil, and writing it
found two things the design had assumed rather than checked.

**`is empty` cannot mean *no ward*.** `spell::watch` answers `empty` by asking
whether the node has **children**, and a prism's children are its published
readings — of which there are none until the first press lands. A `breaking`
bounded on `repeat until the prism is empty` ended on its own first instruction,
having pressed once and looked finished. **An open ward reports `working`**, the
same answer the stacks gives for an open maze and for the same reason, and the
bound is `repeat until the prism is idle`. (`charged` was the first answer and is
also wrong: it means *wield this and it runs*, which a reading in progress is
not.)

**A guard that never moves is a ladder that never advances.** Every rung for a
socket asked the same `loose` question, so the first rung fired for ever — and
the first rung is `dial first nitre`, which the opening aperture already holds,
so nothing moved, nothing pressed, and the spell spun for six hundred ticks
having done one press. The fix is a **per-socket tally**: a dial *aimed* at a
socket marks it whether or not anything moved, so the next lap takes the next
rung. That is honest bookkeeping — *attempts on this socket* — and it is the only
way a language with no variables can walk a list.

A settled socket is deliberately **not** marked, because the latch means *stop
looking here*, and a tally that kept climbing on a socket nobody may touch would
tell a ladder it had made progress.

**It keeps going, unlike `threading`.** A maze solver stops at one so a single
walk can be observed; a ward solver is a faucet, and breaking seals while the
player is in the laboratory is the whole reason to bind one.

#### Slot contention, measured

§19 refuses the production slot to a maze because *"a solver holding the tower's
one slot would starve every other spell into `spell_gave_up`"*, and a bound
`breaking` presses for twelve ticks in roughly every thirteen — worse in
occupancy than the case that refusal was written for.

**Measured rather than assumed**, which the plan for this phase insisted on: over
600 ticks the solver does not own the slot outright, and a `grind` issued beside
it still completes. The tower is **contended, not starved** — `PATIENCE` is 120
ticks and a gap arrives far more often than that. A brew beside a bound solver is
slower, which is the trade §5.0 calls *"concurrency is the real scarcity"*
working as intended rather than a defect.

### Discovery lives in two rooms, and the lens is the second

**§11 puts discovery in `archive/`** — *"decipherment; powers all discovery"* —
and the lens now discovers things too. That is not a contradiction, and the split
is what makes it not one:

| The archive discovers | The lens discovers |
|---|---|
| **your own capability** — verbs, hidden directories, what fragments assemble | **other people's knowledge** — what a far wizard knows how to make |

You do not decipher a recipe out of your own shelves. You steal it from somebody
who already had it, which is what a broken ward is for.

#### The spill

A dozen lines of somebody else's laboratory, **quiet** — they go to `lens.log`,
and the transcript gets one sentence saying how many there were. Twelve lines a
solve on the transcript would push the player's own last command off screen in
seconds, which is the argument §19 already makes for a spell's output.

**Generated from real content, so it stays true.** The shapes are authored and
the nouns come from `Recipes`, so a line always names an instrument doing
something it can actually do. A free-for-all of names would print `digest sage`
— plausible, false, and a player who tried it would learn the wrong thing about
their own tower.

**A secret can appear in the spill, and that is the point.** `distil dregs` in
somebody else's log is a recipe you do not have, sitting there being used. It is
a hint rather than a leak, because the word is not in *your* vocabulary until you
find it.

**`RecordKind::Message`, not `Entry`.** An `Entry` speaks as a `TableRow`, which
`Record::is_prose` excludes — so every field draws, and a stolen line came out as
`probe mix phlegm prism`: the orb's own bookkeeping wrapped around somebody
else's log entry.

#### Secrets: content, not player state

`recipes.toml` gains `secret = true`; `tower::Learned` holds what has been found.
Keeping them apart is what lets `Recipes` stay immutable, which is what recipe
replay rests on — and `Learned` is itself reproducible from `(seed, submissions)`,
because a discovery is a seeded roll on a deterministic tick.

**Exactly three questions consult it**, and the restraint is the design:
whether a recipe fires, whether it is *part-way* to firing, and whether its
product is a word the player can say. **Everything else stays unfiltered**, and
one of those would have been a real defect: `execute::scroll` derives a base
reagent as *"in the vocabulary and made by nothing"*, so a filtered `outputs`
would drop an undiscovered potion out of "made" and offer it as an inexhaustible
herb — the same shape as the bug §19 records shipping once, when every byproduct
read as a herb.

**The gate is a set subtraction in `scene_at`, and it has to be there.** `Topics`
is snapshotted once at construction so a prose reload cannot change what a phrase
resolves to — and a secret potion's `recall_` page *is* a prose key, so it is in
that snapshot from tick 0. Gate it at snapshot time and `recall <secret>` answers
before the player has found it, with every other test green because the recipe
still refuses to fire.

**An unfound recipe reads `fouled`, not `charged`.** Honest rather than coy: the
player is holding something that makes nothing, as far as they know. `charged`
would promise a run that can never start.

#### The curve, and the three that ship

`0.04 + 0.04 × solves_since_last`, capped: mean about six solves, **certain by the
25th**. A flat chance leaves a player forty solves in with nothing; a certainty
makes the lottery a queue. It **retires** once all three are found rather than
rolling against an empty pool.

The three are `mending`, `dreaming` and `vigour`, and each is **one step over a
byproduct** — potash, sediment, dregs. That is the point rather than a saving:
§10.1 gives every byproduct one use, and these give three of them a *second*, so
what you find is a use for something you have been throwing away. Each is also
the colour of what it was distilled from, which
`the_flasks_products_are_the_colour_of_what_makes_them` enforces and which caught
all three of a first pass's prettier choices.

**The file's order is the reveal order** — the roll takes the first unfound one
rather than drawing at random, so `recipes.toml` decides what a player meets
first, and a replay reaches the same tower.

### The board — the ward as a sheet, and what it may not show

The map's shape one room over: **not gated on a word**, so a bound solver is
watchable; **columns, never rows**; and it splits *after* the instrument panel,
because whichever runs second is the one whose refusal can fire.

**It refuses rather than panning, and the maze does the opposite.** A maze is 33
squares across and a window centred on the reading is a useful answer; a ward is
ten columns and every row matters equally, so a sheet showing four of six presses
has lost the two the player was about to compare. There is no "where you are" to
centre on.

**It carries nothing the readings lack, and adds only *history*.** Every row is a
press the transcript already reported; what a sheet buys is five of them side by
side, which is what makes deduction possible without a notepad. Nothing is
inferred, because the sim it reads from infers nothing.

**The figure recorded is the one that was sent, not the one it snapped back to.**
A board logging the reverted aperture would say a press answered something the
press never asked.

**Six glyphs, never six colours** (§14). `▪` for a held socket is not in CP437 and
was the first choice; `■` is. `●` for a peg is not either; `•` is. Both were
caught by the board's own repertoire test rather than by a player — the same
class of miss §19 records finding in DESIGN.md's own boot text.

**Spoken once, as a summary.** A reader hearing four sigils and four pegs read out
cell by cell gets box-drawing noise, which is exactly what putting structure on
one channel and content on the other exists to prevent. The per-press detail is
already in the transcript, where it was said as a sentence.

### The tower rail replaces the telemetry pane — Phase 2 item 2

**§9's sidebar was built, tested, and unreachable**, listed in this log under
*"gated by: brewing + archive — nothing to minimise with two panes."* Scrying is
the third domain, so the gate opens — and what it opened into is not the shape
§9 wrote.

**A thin vertical column on the right, one box per domain**, rather than
full-width rows above the input line. Everything §9 argued for survives:
awareness only, never commandable, and it yields before the main window does.
What moved is the axis, and the reason is arithmetic — **a row can hold a name
*or* a state *or* a spell, and a box can hold all three.** A glance at seven
domains wants to know that the alembic is busy *and* that `tending` is what is
keeping it busy, and one line cannot say both.

There is **one implementation**, not two: `ScreenLayout::sidebar` became
`ScreenLayout::rail`, with its tests rewritten to hold the same properties on the
other axis.

| | |
|---|---|
| Columns | **16.** Inset one leaves 14, against `battlements` at 11 plus a mark, `►tending` at 8, `al 22t` at 6 |
| Boxes | **Seven, always.** An unbuilt room is a dim dotted row with no name |
| Below the floor | **Dropped whole**, never narrowed — at 80×22 there is no rail and the session pane is intact |

**The telemetry pane is gone, and `PANES` is 1.** It drew nine developer readings
into fifty-eight columns; five of them are at the rail's foot and the other four
— `seed`, `queued`, `logged` and the rest — were already in `status`. **They
could not have moved there anyway**: `scale`, `cols`, `rows` and `focus` are
`Screen` facts, and `status` lives in the sim, which rules 1 and 2 forbid a
window.

**What one pane bought: the session body went from 58 columns to 102.** The
laboratory's instrument panel flips to `Along::Side`, and — unplanned — **the
archive's maze stops panning**, because the whole 35-column picture now fits
beside a transcript. This log's own note that it pans is amended: it pans below
this grid, and `ORBS_GRID=80x22` is where to see that.

**Seven slots with four dark is foreshadowing, and it is deliberate.** A rail
showing only what exists would grow a box at a time with no warning. Naming the
unbuilt rooms would spend §11's discovery; drawing them anonymously says *there
is more* without saying what. It also keeps the built boxes in fixed positions,
which is what makes a mark findable.

#### Marks: a latch cleared by walking in, never a timer

A mark says *something happened over there*. It is cleared by `attend`ing the
domain and by nothing else — diegetic, needing no second clock, and unable to
drift from what the player has actually seen. **A timer was the alternative and
is worse**: it would clear while the player was making tea, which is the case
idle play is largely made of.

**A fault outranks news**, in either arrival order, because the rail has one
glyph to spend and a player shown the find and not the broken spell has been told
the less useful of the two things.

**Glyphs, not colours** — `‼` and `!`, with the accent carrying the same fact a
second time. A red border and a green exclamation are the obvious design and are
exactly what §14 forbids.

**A refused command is not a fault.** The mark is raised in `say_failure` at
`Role::Danger` only, which is four cases: a spell gave up, named something that
is not there, used a verb it may not, or holds a line the orb cannot read. A
`Cost` there is a spell politely waiting for the production slot, which happens
constantly.

#### Two things it cost, both recorded rather than fixed

**`F4` is visibly inert at one pane.** §9 makes the focus mode overridable at any
time and this log fixes the switch on `F4`; with one pane both tilings are
identical, so the key does nothing until multiplexing returns the second pane in
Phase 9a, when it reclaims its job with no code to change. Reassigning it to
toggle the rail would re-litigate two recorded decisions to buy a key a job for
one phase. ROADMAP's ✅ line claiming *"`F4` changes the split"* is amended to
what remains true, and `f4_switches_focus_without_moving_the_grid` now asks about
an explicit two-pane request — the property is about the tiler, not about today's
pane count.

**`PaneTransition::panes()` is gone** with its one non-test caller. It comes back
with multiplexing, which is the only thing that will make the count vary again.

#### The boxes are ruled off, and drawing the rule changed the layout

A box holds one to four rows of content in a five-row slot, so consecutive
domains ran together in a column of whitespace and which line belonged to which
room was left to the reader. **A horizontal rule closes every box but the
seventh**, whose boundary is the foot's own rule — two rules in adjacent rows is a
thing nobody would author deliberately. The rule belongs to the box *above* the
boundary, so it costs that box a row, and it is silent like the border: structure
writes cells and no speech, and a reader hearing six horizontal lines read out
between seven domains gets box-drawing noise where a sighted player gets
separation for free.

**Drawing it made an existing unevenness visible, which is the interesting part.**
`lay_rail` gave its remainder to the earliest boxes — `tiling::deep`'s rule,
shared deliberately so this crate had one way of splitting leftovers rather than
two. That is right for a pane, whose exact height nobody can see. It is wrong for
a *ruled* box: the first box was one row taller, so one separator sat a row below
the other five and read as a defect. **The remainder now goes to the foot**, where
slack is invisible because the readings are top-aligned. Seven equal boxes, six
evenly spaced rules.

`MIN_RAIL_BOX` went **3 → 4** with it, and the doc it had was already the argument:
*"a name, a state, and the spell running there."* At three, with a row spent on the
rule, the squeezed box kept its name and state and silently dropped `►spell` — the
one row that tells a player a room is automated. The fixed 120×45 grid gives each
box five, so this moves only where the rail yields entirely.

#### `▸` is not in CP437, and the lint that exists could never have said so

The spell marker was `▸` (U+25B8), which is not in the code page, so it drew as
`?` — the row saying a room is automated instead read as *the orb does not know
what is there*, which is the worst available reading of it in the pane whose whole
job is a glance. `►` is CP437 0x10 and is the glyph that was wanted.

**`cp437::is_renderable` never saw it, and would not have.** That lint runs over
authored *prose*, because rule 6 puts every player-facing string in a TOML file —
and a painter's structural glyphs are Rust literals by design, which is the one
category the content pipeline cannot reach. The board's `▪` was the same defect and
this log records it; a second occurrence in the same phase is the argument for a
test rather than for care, so `every_glyph_the_rail_draws_is_in_the_code_page`
asserts the painter's own constants.

**And the spell row said `tending.spe`.** A spell node is named for the file it was
scribed to, so the brief carried `tending.spell` — six columns of extension in a
fourteen-column rail, cut in half by the rail's own truncation. Stripped in
`brief.rs` using the `content::without_extension` that already existed, **not** in
the painter: rule 2 gives a frontend only *how* a cell is drawn, so `orbs-tui` must
not have to know that spells live in files.

### `orbs-balance` is built, and it disagrees with four numbers — Phase 2 item 1

**Built first in the phase, deliberately**, because ROADMAP writes the item as
*"**before** five phases author durations on top of unswept ones"* and because
§15's own rule is *build the instrument before the thing it measures*. It drives
a real `Sim` through `submit` with five synthetic players, samples experience
against ticks, and prints a per-tick rate against a pinned reference.

It found four things on its first clean run. **None of them is a bug**; all four
are the difference between a number argued for in a sentence and a number a
player can reach.

**1. Every rate in §11.5 is a *recipe-tick idealisation*, and the real loop is
about 25% slower.** Clarity's 0.170 is 16 experience over 94 ticks — grind 8,
digest 12, grind 8, mix 10, distil 56 — and counts nothing else. A hand-played
clarity reaches `experience 16` at **tick 123**, which is 0.130, because every
command pays a tick of queue latency and each fouled instrument pays a
`PURGE_TICKS` scour before the next lap can load it. A running loop settles at
**0.140** once the first lap's setup is amortised. The design keeps its
idealisation, which is the right way to *derive* a threshold; `orbs-balance`
pins the reachable number, which is the right way to catch drift.

**2. §10.1's damping is *behind*, not ahead — and the reason is that fuel is
endless.** §10.1 argues the efficient play is *light → digest → damp → combine →
relight → distil*, and this log calls that *"the best argument yet that `stop
athanor` is a real move"*, claiming the damping script is *"meaningfully ahead
over a session"*. Measured, `damped` is **0.1356 against `clarity`'s 0.1400** —
3% behind. Damping does avoid the five lost laps an hour that a guttering fire
costs the careless policy, but it pays two extra commands every lap to save a
resource that `build.rs` deliberately makes `Holding::endless`.

**This is a real gap in the design, not a tuning miss.** ROADMAP's six-row table
gives brewing *"a shared, **depleting** resource — lit time"*, but lit time only
depletes something if fuel is scarce, and it is explicitly not. Either charcoal
stops being endless — which `build.rs` refuses on the grounds that *"a cold
athanor with nothing to burn is a laboratory with nothing to do"* — or lit time
buys something other than fuel, or brewing's stated scarcity is not one. **The
claim is withdrawn until one of those is chosen**; `Policy::DAMPED` stays in the
harness so the day it gets ahead, the table says so.

**3. The haste chain still dominates the flagship, and the risk stands.** This
log already records it at 0.367 against 0.170, a ratio of 2.16. Measured it is
**0.2553 against 0.1400 — 1.82**. Gentler, same shape, still the case that the
shortest chain in the laboratory out-earns the potion the tutorial teaches.

**4. The archive maze is 10–20× below the flagship, and it is seed-dependent.**
`progression.toml` already flags `stacks = 4` as *"a question for the balance
harness rather than for this change."* The harness answers: **0.0067 at seeds 0
and 11, 0.0144 at seed 3**, against clarity's 0.140. It is excluded from the
pinned reference table for that spread — pinning it would pin a seed rather than
a rate.

#### Two things the harness needed that the sim's shape decides

**A policy is a policy, never a script with a stopwatch.** Every command goes
through `Sim::submit` and lands on the next tick like a player's, and the only
timing model is waiting for the tower to be free. That is this log's own
distinction — *"a script encodes a policy; `orbs-balance` sweeps policy against
state"* against *"the harness has no player, so it cannot sweep anything"* — and
it is why there is no typing delay here and never will be.

**`Sim::working()` is the production slot only, and a driver must also watch the
triage slot.** §9 gives a pane one production slot *and* one triage slot; a
`purge` lives in the second and is invisible to `working()`. A driver waiting on
`working()` alone runs straight over its own scour, is told *"you are already
scouring the balneum_mariae"*, and reports 0.072 for a loop worth 0.140. The fix
is to also ask whether any instrument reads `State::Scouring`, which is what the
instrument panel draws, so the two cannot disagree.

**The `cost` column cannot tell a refusal from a scour, and does not pretend
to.** `refuse_busy` and `purge` both stamp `Role::Cost` — §4's accent triad says
*"mana and arcane expenditure"*, which a scour honestly is. Discriminating would
mean matching prose, and rule 6 puts prose in a file precisely so nothing in Rust
depends on its wording. So the number is *things that cost something* and `--why`
prints the sentences, which is where the diagnosis actually lives: **two sweeps
were read as balance findings before that flag existed, and both were the
policy's fault.**

### The harness's first regression catch — the ambient swap was terminal

The four findings above were all *disagreements with prose*. This is the first
thing `orbs-balance` caught that was simply wrong, and it caught it one item
later in the same phase: the world sabotage surface (Phase 2 item 7) shipped, and
the next sweep flagged **all four pinned policies at once**.

| | pinned | measured |
|---|---|---|
| clarity | 0.140 | **0.074** |
| damped | 0.136 | **0.104** |
| haste | 0.255 | **0.116** |
| grind | 0.100 | **0.093** |

The whole test suite was green throughout, and so was every See-it line — because
`debug_swap` targets a pile directly and shows the tell working perfectly. What
nobody had looked at was the *ambient* system running beside it.

**Three defects, each hidden by the next.**

**1. The target was never actually drawn.** `substitution` sorted the endless
piles by name — correctly, because a replay has to swap the same pile from the
same seed — and then took `piles.first()`. So *which* pile was hit was as fixed as
the sort: `charcoal`, then `rock-salt`, then `sage`, in that order, on every seed,
for ever. A determinism fix that quietly became content. The index now comes out
of the quotient of the same roll that decided *whether* to swap, rather than a
second draw — a draw that only happens when the swap fires would move `drift`'s
stream position by a variable amount, which is the exact hazard the two systems
were split apart to avoid.

**2. Fuel was in the pool, and it should never have been.** The module's own
argument for restricting swaps to endless base stock is that a swap must cost
*the spell that named the reagent* and nothing half-made. Charcoal is named by no
recipe at all — it is the tower's power supply, so swapping it stops every heated
stage in every domain at once. Combined with (1) it meant the **first** swap of
every session took the fire: no `kindle`, therefore no digestion, no
distillation, no clarity, and none of the three secrets. Fuel is now exempt, on
the same reasoning that already exempts the non-endless piles.

**3. A swap was permanent, and the repair loop is human-only by construction.**
This is the finding worth keeping, because it is a fact about the *language*
rather than about this module. A spell names things with literals — `parser/
question.rs` has no variables — so a spell can never say *"purge whatever the
dispensary is lying about"*. It would have to name `sage-`, a word nobody knew
when the spell was written. So notice → `verify` → `purge` is reachable **only by
a person reading the screen**, and an unattended tower has no path back at all.

Measured: the standing grind loop fell from 0.100/tick to 0.058 and **stayed
there for the rest of the session**. §5.1 caps aberration arrival *"so repairs
cannot spiral"*; a permanent un-automatable swap does not spiral, it terminates,
which is worse and was never the intent. A lie now settles back to the truth on
its own, and `purge` still repairs it the instant it is found — which is what
keeps the human loop worth running rather than making waiting the better play.

**The two constants are swept, not chosen, and only their ratio matters.** A loop
stalls for as long as the pile it names is lying, so its downtime is `WEARS_OFF /
SWAP_INTERVAL`. At (1200, never) that was total; at (1200, 1800) it was 60% of
every window. **3600 and 300 — one swap an hour, five minutes of trouble — is 8%**,
and at that size every pinned rate came back inside its band on three seeds
without the reference table needing to move. That is the outcome to want: a
nuisance small enough that it does not perturb the numbers the rest of the design
is derived from.

**A swap is also twelve times rarer than a poisoned log now, and it was four.** A
poisoned log misdirects a reading of one; a swapped reagent stops every loop that
named it. Pricing them one step apart said they were the same order of
interruption.

#### The pin was not a pin, and the test that holds it

`report::EXPECTED` is documented as a regression pin, and printed `<-- drifted`
beside a measurement that left its band — which is a pin only for as long as
somebody is reading the column. Nothing failed. `tests/agrees.rs`, named for the
See-it claim *"a sweep's curve and a hand-played session agree"*, drove `Sim` by
hand in both its tests and would have passed with the entire harness deleted: it
asserted the reference numbers were *reachable* and never that the harness reached
them.

Both halves are fixed together, because they are one hole. The crate gained a
`lib.rs` — a binary-only crate cannot be reached from an integration test at all,
which is *why* the file had been written against `Sim` directly — and the test now
runs real policies through `drive::run` and fails when a measurement leaves its
band. Pointed at the tree as it stood, it flagged all four policies immediately.

### The arsenal, and §7's one exemption

**Nothing in the tower could be carried between domains.** `carry`'s destination
lookup wants a `Fixture` child of `cwd` and a domain is neither, so `move clarity
to archive` could not resolve — and neither could any route between two rooms.
That held while nothing a domain made was wanted anywhere else: the laboratory
consumed its own reagents, the archive its own fragments. A **finished** thing
breaks it, because a potion is brewed to be used elsewhere and a scroll is
assembled to be spent elsewhere.

§10's remaining five domains all have the same problem waiting for them, so
`/tower/arsenal` is the standing answer rather than a fix for one pair.

#### A finished potion could not be picked up at all

`move`'s first slot was `NounKind::Reagent`, and `produce::transmute` gives a
`potion = true` output `NounKind::Essence` — so the required slot silently never
filled, and had not since potions existed. It went unnoticed because `empty`
turns an instrument out wholesale and never asks what kind anything is, so the
one route that mattered *inside* the laboratory worked.

`NounKind::Portable` is the fix, on the `Stoppable` pattern: stock, a fragment, a
potion, a scroll. Deliberately **not** `Any`, which reaches places, files, topics
and spells — `move laboratory to arsenal` would resolve at full confidence.

#### The exemption is narrow, and stated rather than assumed

`tower::scene` records that acting on a domain you are not standing in is **Phase
2's** unlock. This does not repeal that. What reaches everywhere is the arsenal's
*contents*, on exactly the terms places, spells and the maze's readings already
have — a spellbook you carry is not a shelf you walk to, and neither is a
bandolier.

What keeps it honest is the door. **Finished work only** — an `Essence` or a
`Scroll` — so the arsenal cannot become a second dispensary and reagents still
belong to the domain that uses them. The question is asked of the **kind**, never
of the name: telling finished work from stock by name would mean the tower
deciding which reagents are waste, which §10.1 refuses outright, because every
byproduct is some other recipe's input. And `reachable` searches the arsenal
**last**, so a reagent in the room always outranks a carried one and nothing an
existing command picks up can change.

#### Nameable is not enough, and that was the whole risk

Registering the contents in the scene makes them nameable from every room — and
`purge` and `verify` take `NounKind::Any`, so both would then *resolve* on a
potion from any room and report it absent. That is §15's dead end arriving
through the affordance meant to remove one, and it is the failure mode this
document already records for spells: a `Script` slot with nothing findable is
*worse* than a dead end, because `bind night_watch` fell through to `sift` and
reported success.

Three lookups had to learn it — `pipeline::reachable`, `pipeline::purge` and
`files::here_or_place` — and `tower::keep` owns the rule so they cannot disagree.
§19 already records what happens when a rule like that has copies: three
hand-built versions of *what is in this instrument* all dropped the stock count,
and nothing noticed until a recipe wanted more than one of something.

#### It is also what lets a spell touch a potion

Not obvious, and worth writing down. A spell is written *for* a domain and
`may_issue` forbids it `attend`ing, so it can only name what is in scope where it
runs. Finished work that lived in the room that made it could never be reached by
automation running anywhere else — so this is pillar 3's access to the crafting
economy, not only a convenience for a player's hands.

#### What came free, and one defect settled

Being a top-level branch gives it `Protected` — so `purge arsenal` refuses in
character, which matters more here than anywhere, since the room holds everything
the player has finished. The boot report walks the world, so it gains a row with
no code. `arsenal.log` fills because `move` already stamps `Path`; §19 records
the archive's log being empty from the day it was built for want of exactly that.

And **`fragment` was two noun kinds at once**: `NounKind::Fragment` from a solved
maze, `Reagent` from `debug_spawn`, which asks `Recipes::kind_of`. `stock::give`
merges by *name*, so the two would have merged into whichever node was found
first, and `move fragment ...` worked on one and not the other. One rule now: the
maze asks the same question everything else does.

#### Every item has a home, and it is a rule rather than a list

`debug_spawn` is how a state worth testing is reached without paying the forty
ticks of grinding it costs, and its value is that **a new item is testable the
moment it is authored**. That has to be structural. A hand-kept list of where
each thing goes is a promise kept until somebody is busy: the next material lands
in whichever room the list happened to say, or in none, and the person who finds
out is the one who assumed the tool was right and went looking for the bug in the
game.

So `tower::home` is a rule *over the content*, and the rule is **where the game
itself would leave it**:

1. **Finished work** — an `Essence` or a `Scroll` — goes to the arsenal. That is
   the room it is for, and the one reachable from every other.
2. **Anything a recipe produces** belongs to the domain that produces it. Dust is
   the archive's leavings even though a mortar will grind it, because the archive
   is where it comes from.
3. **Anything else** — a base reagent, a fuel — belongs to the domain that
   consumes it. Nothing makes sage; the laboratory is where it is wanted.

Within a domain it is the **store**, never an instrument. A shelf is inert: the
thing sits there until it is `move`d or a per-instrument verb reaches in for it.
Dropped straight into a tool it would charge it, and a mortar holding something
no recipe wants reads `fouled` — a state to explain rather than one to test from.

Two lints make it a guarantee instead of a claim.
`every_material_has_a_home_a_move_can_reach` walks every authored material and
fails the build by name if any has nowhere to live, or lives somewhere
`pipeline::reachable` cannot see into. `every_name_the_tool_offers_lands_in_the_
room_it_belongs_to` drives each one through a real `Sim` and checks it arrives
where the rule says — against the rule rather than against a room written down in
the test, because a room written down in a test is the list this replaces.

#### Byproducts are the laboratory's mechanic, and only the laboratory's

`Recipe::leaves` was compulsory, so every recipe had to shed something — and the
lectern's assembly duly shed `dust`, a substance invented to fill a field. It
then had nowhere to go. §10.1's *every byproduct has at least one use* forced a
second mortar recipe grinding dust into the potash that ash already became, and
**that recipe could never fire**: reagents do not cross a domain boundary, so the
archive's dust could not reach the laboratory's mortar. A recipe that only fires
on material a tester spawns is not content.

The chain of repairs it was pulling behind it is the tell. Dust needed the
archive to grow a shelf so it was not trapped in the lectern; it needed a mortar
recipe so it was not litter; it needed a tint, a name and a paragraph about old
paper. None of that was buying anything, because §10.1 builds the waste-has-a-use
loop around **brewing** — where a second route to the same draught can exist and
be interesting — and the archive has no second route to anything.

So `leaves` is optional and the lectern leaves nothing. The mechanic stays where
it works. `every_byproduct_has_at_least_one_use` now polices the byproducts that
exist rather than the absence of one, and two prose lines gained a companion —
`wield_done_clean`, and a `route_leaves` clause composed in beside `route_heat` —
because *"and leaves "* with a hole after it is how §19 says a missing key
surfaces, and it would have surfaced on every scroll the archive ever assembles.

### A thing's page says what it is, not only how it is made

`recall <thing>` walked the recipe graph and stopped. So the manual could tell a
player the five steps to a `clarity` and not one word about what a clarity *was*
— and a `gleaning-scroll`, the thing four walks of the stacks pay for, answered
with its assembly and nothing about unrolling it.

Those are different questions. A route answers *how do I get one*; a player
holding the thing is asking *what is this*, and after that *how do I use it*.
Both are said now, and what it is comes first — the route is the least urgent of
the three to someone with the object already in hand.

**`using_<name>`, deliberately not `recall_<name>_use`.**
`Prose::topics` decides what is nameable by stripping `recall_`, so the suffix
spelling would have registered `clarity_use` as a subject to ask the orb about.
That is the trap `grimoire_step_or` already paid for once, and the file's own
comment warns about it two sections up.

**Where a use is not built, the page says so.** Every potion reads *"nothing
drinks a potion yet. a siege will be what spends them"* — §11.5's resource table
has a siege consuming 4–8 potions and sieges are Phase 8. `undo`'s verb page
already does this, and §15's argument is the same: a page admitting a thing does
nothing is the cheapest way to keep a player out of a dead end.

**Two lints, beside the three the verb pages have.**
`every_material_has_a_page` fails the build by name for a material with no
description; `a_finished_product_says_what_it_is_for` requires a `using_` line on
every potion and scroll. §19 records this class of omission twice already — the
four shard names, and the `dust` a compulsory field invented — and both times a
person asking found it rather than a test.

**It makes every material a `Topic`, which is the same exemption verb pages
have.** *A manual you can only read in the right room has a lock on it*, so
`recall sage` works from the archive. §7's scoping is unharmed and is now stated
more precisely than it was: from the archive `sage` is something to read about
and not something to grind, which is a claim about the **kind** rather than about
the name. The cost, recorded rather than discovered: `purge`/`verify` take
`NounKind::Any` and so can name a material from a room that has none — which was
already true of every recipe *output* and is now true of every input and
byproduct too.

#### Quickening, and the herbs a scroll puts on the shelf

The other two scrolls, and each answered a question the first left open.

**Quickening is a window on the room, and was a one-shot on a run.** The first
version halved what was left of the run in hand and *refused when nothing was
running* — which made it unusable at exactly the moment a player reaches for one.
**Quicken the laboratory, then brew** is the obvious play, and it was the one
thing the scroll could not do; four walks of the stacks also bought a single stage
where a window buys a stretch of work and rewards lining it up.

So it sets `Quickened` on the laboratory for `QUICKENED_TICKS`, and everything
the room starts inside that window takes half as long.

**An interval, like the fire.** `Burning` is *"a pure function of the tick, so
fuel survives `meditate`"*, and speed is the same claim: the state stores when it
began and how long it lasts, nothing ticks it down, and hundreds of ticks
collapsed inside one `step` behave exactly like hundreds watched.

**Read when a run starts, like heat.** §10.1 checks the athanor at `begin` and
lets the run finish even if the fire dies under it, because *"pausing would be
the countdown §19 refused"*. A run started inside the window stays short when the
window closes — which is also what keeps `Working` an interval set once, the
property `meditate` idempotence rests on. A rate applied per tick would be that
countdown wearing a multiplier.

**And what is already running is hurried too — one rule, not two.** The state
means *this room works at double speed*, and a run in flight is something the
room is doing; leaving it alone would make wielding the scroll mid-brew look like
it had done nothing. Halved from **now**, not from the start: halving the whole
interval would refund time already spent and, past the half-way point, land the
end in the past.

`QUICKENED_TICKS` is **300 — five minutes**, since §5.0 makes a tick one real
second. That is squarely in §11.5's Production band (3–10 minutes), which is what
a window covering a *stretch of work* should be measured against, and it is about
three clarities' worth of brewing. Generous, deliberately: a scroll costs four
walks of the stacks, which is thousands of ticks of walking, and a window covering
one stage made that trade absurd. A placeholder like every other duration, and
`orbs-balance` being a stub is why sweeping it is a plan rather than a fact.

**The test that guards it was measuring nothing**, and raising the number is what
showed that. `a_run_started_in_the_window_stays_short_when_it_closes` meditated
25 ticks against a 120-tick window and then asserted a length nothing had any
reason to move — green for the wrong reason. It starts the run *near the end* of
the window now and crosses the boundary while the run is going, checking
`tower::quickened` on both sides so it cannot silently stop closing again.

**The verdant scroll gives one herb, not all of them.** The plan said *unlock the
reagents*; a scroll that did the lot would leave the lectern assembling something
with nothing left to give — a dud draw for ever, which is the dead end this item
exists to close. Three scrolls, three herbs, and the laboratory visibly grows
three times.

**What counts as unlockable is derived.** A base reagent is one the vocabulary
knows that *nothing in the tower makes*, whose home is the laboratory's shelf. So
authoring a fourth herb makes it unlockable the same tick, and the archive's
`fragment` is excluded without being named — `tower::home` already knows which
room a thing belongs to, and asking it beats a second list.

**Getting *made* wrong was visible rather than subtle**, and it is the clearest
argument for the See-it rule in this whole arc. The first version asked
`Recipes::outputs`, which is a recipe's `output` and **not** its `leaves` — so
every byproduct in the game read as a herb, and four scrolls put `dregs`, `ash`
and a `fragment` on the shelf as inexhaustible stock. Every test passed. One dump
showed it in a second.

**Names measured, not chosen.** `steeped-draught` and `settled-draught` collided
at **734** — worse than the 667 that got `decant` renamed to `siphon` — and
became `keen-draught` and `quiet-draught`; `flowering-scroll` sat at 688 against
`gleaning-scroll` and became `verdant-scroll`. The worst remaining pair is 693,
against the 819 (`ground-sage`/`ground-salt`) and 896 (`sage`/`sage-tincture`)
the game already ships and `Scene::knowing` already protects.

#### The stacks and the lectern are two instruments, and were one

The maze opened **on the lectern**, which was also where four fragments became
a scroll. §19 above records that as *"the first instrument that can be doing two
things at once"* and treats it as a curiosity found while fixing `stop`. It was a
design problem wearing one.

- `stop lectern` had to guess which of the two it meant, and the fix was a branch
  that stopped the maze *and then* fell through to the run.
- The panel gave both one row, and one `State`. *"Is a reading open"* and *"is a
  scroll coming together"* were the same question with two answers.
- Its picture was `Craft::Reading` — the maze's explored-cells gauge — so a
  twenty-tick assembly drew a bar measuring something else entirely.
- And a fixture carries exactly one `Operation`, which the lectern spent on
  `research`. That is the recorded reason `follow` is a tower-wide verb wearing a
  domain's coat.

So the archive has **stacks** — an endless library you navigate, which is what a
maze in a library *is* — and the lectern goes back to being one thing: where four
matching fragments are moved out of the cabinet and assembled. `stop stacks`
closes the stacks, `stop lectern` abandons an assembly, and neither reaches the
other. The panel has a row each, with the maze's gauge on one and an honest
progress bar on the other.

**The lectern has no `Operation` now**, and its picture comes from having
*recipes* instead — `craft_of` asks the content rather than matching a name,
which is the pattern that module's own header records paying for twice.

**`[earns]` had to widen, and it was too narrow already.** Its keys were checked
against `recipes.toml`, so an instrument that runs and transforms nothing could
never be priced — which was true of the athanor all along and became true of the
stacks, which earn for every walk finished. The check now takes the recipes'
instruments *plus* the fixtures that carry a verb. Both archive instruments keep
the 4 the lectern paid for both halves, so the split rebalances nothing; a rename
is a bad moment to change a number.

**It did *not* retire `follow`'s debt, and that is worth being clear about.** The
lectern's `Operation` slot is free now, so `follow` could be scoped through it —
and that would be a lie, because `follow` walks the stacks' maze and the lectern
does not answer to it. Two verbs still want to scope to one fixture, which is the
same missing mechanism `verb.rs` names. `follow` and `wander` retire together the
day a fixture can carry a second operation, exactly as recorded.

**One naming note, measured.** `stacks` scores **667** against `status`, over the
600 floor — six letters, two edits. They never compete in the same position: the
first word of a line is a verb and `stacks` is a place, so `attend stacks` and
`survey stacks` resolve against nouns where `status` is not one. What a bare
`stacks` can do is fuzz to `status`, which prints the status and echoes the word
it chose — §6's echo answering exactly the case it exists for. Recorded rather
than renamed, because the word is right for the room.

#### "labyrinth" is retired: the stacks are the instrument *and* the place

The split above gave the archive a fixture named `stacks` and left the thing it
opens called a `labyrinth`, so the game had two words for one object and said
both — *"there are no stacks here to walk a labyrinth in"* was a real line. The
second word is gone. **You `research` at the stacks and you are then in the
stacks**, which is what an endless library means and what the split was for.

The collision that had to be rewritten rather than substituted is the whole
reason this is an entry: a find-and-replace produces *"no stacks here to walk a
stacks in"*, so every sentence naming both was reworded, and the ones treating
the maze as singular (*"walk it"*, *"research opens one"*) became plural
(*"walk them"*, *"research opens them"*). `research_opens` is now *"the page
opens into shelves that do not end"* — the image survives, the noun does not.

**`maze` stays, and only below the waterline.** It is the accurate word for a
spanning tree with one path between any two cells, and `Maze`, `maze.rs` and
Prim's own documentation read worse renamed to a room. So the rule is: the
player and the prose say **stacks**; the algorithm says **maze**. Nothing
player-facing says either "labyrinth" or "maze" any more — the two manual lines
that did (`man_wander_1`, `man_research_1`) were rewritten with the rest.

Renamed with it: `orbs_render::Labyrinth` → `Stacks`, `Painter::labyrinth` →
`Painter::stacks`, `Sim::labyrinth` → `Sim::stacks`, `shell/labyrinth.rs` →
`shell/stacks.rs`, and `research::finish_labyrinth` → `finish_walk`. **The
shipped v0.1.24 changelog block was left alone**: it is the record of an
announcement that already went to GitHub, Discord and Bluesky under the old
word, and it describes a tower where the maze still opened on the lectern.

#### The domains come before the siege, and the phases moved down by six

**"If we don't have a series of interesting puzzles, then there really is no
game."** Two of §10's seven domains were built; the other five were a single line
inside Phase 3a, two phases away, and Phase 2 was the siege — building the thing
that *consumes* the puzzles before the puzzles existed.

So Phase 1 closed, everything from the old Phase 2 down shifted by six, and
**Phases 2–7 are Scrying, Spellcraft, Enchanting, Summoning, Defense, and the
phase that makes them one machine.** Old 2 → 8, 3a/3b/3c → 9a/9b/9c, 4 → 10,
5 → 11. Phases 0, 0.5 and 1 did not move.

**The version scheme moved with it**, since it is `0.<phase>.<step>`. Closing
Phase 1 makes the next release **v0.2.0**, so v0.2.x now means *Scrying* where
under the old numbering it would have meant *Siege*. Shipped `v0.1.x` tags are
untouched and still mean what they meant. And §19's own note that the 1.0 switch
is *"never read as a thirteenth phase"* was written against six numbered phases;
there are eleven now, so the arithmetic it guards against is *twelfth*, not
thirteenth — the guard still holds, but the sentence should not be quoted as a
count.

**Phase 1's exit had two clauses and one was unmet.** *"A player automates a duty
and feels clever"* is met and mechanical — a script ends its loop with `stop
athanor` and a person walks away with the fire lit. *"Non-terminal testers are in
the loop"* needs a person, and **moved to Phase 10 with the item that carries
it**. Ticking it would have been the failure §15's own See-it rule exists to
catch.

**Two scarcities were invented and had to be withdrawn**, and they are recorded
because the mistake is instructive: a first pass gave Enchanting *"one fuel
supply, two fires"* and Scrying *"looking costs a potion"*. Charcoal is
`Holding::endless` by design — *"a cold athanor with nothing to burn is a
laboratory with nothing to do"* — so fuel **stock** was never brewing's scarcity;
lit **time** is. And §5.1 is *"issuing commands is free"*, so a resource cost on
looking is not available to any domain. **The lesson is that the template is
`CAPACITY = 1`**: §5.0 already says concurrency is the real scarcity, and four of
the six new phases spend it rather than minting a currency.

**Three things were nearly stranded by the renumber**, all in the same roadmap
item: reagents crossing a domain, Focus-slot reservation, and pane addressing.
§19 parked the first with *"Phase 2's pane addressing"*; renumbering mechanically
would have sent all three to Phase 8 — *after* the five domains that need them —
and `CAPACITY = 1` would have made Phase 7's exit unreachable by construction.
They are Phase 7's, and the code comments that referenced them point there.

**The renumber was 72 references in the docs and 30 in code and content**, none
of which type-checks, including one player-facing prose line. Three rules made it
safe: `3c` before `3b` before `3a` before bare `3`, or a pass corrupts `3a` into
`9a`; **`Phase A` and `Phase B` are the tower and siege layers and were not
touched**; and §19's Draft-review tables record *what a review said at the time*,
so the row reading *"Phase 3 overloaded → split into 3a/3b/3c"* keeps its original
numbers with a note, because renumbering a log falsifies it.

#### Every spelling of a comparison, and the manual the language never had

Three follow-ons from the audit, each closing something the audit itself opened.

**The comparison had one spelling and swallowed the rest.** `has at least 2 X`
became `has X` — count *and* words gone, no fault raised — because `at` is §6
filler, so the remainder resolved down to the noun. That is the same swallow
counting was added to close, left open for the phrasing nobody tried. There are
now seventeen spellings in one table: `at least`, `at most`, `exactly`,
`or more`, `or fewer`, `or less`, `more than`, `fewer than`, `greater than`,
`less than`, and `=`, `==`, `>`, `>=`, `<`, `<=`, `=<` — the last set both spaced
and written against their number (`>=2`).

**A third bound arrived with the symbols.** `=` is a thing people write without
being taught, and reading it as at-least would be the quiet reinterpretation §6
forbids — so `Bound::Exactly` exists, and `not has exactly 4 X` is how the fourth
question is asked rather than an `!=` to learn. Strict comparators carry an
**offset** instead of variants of their own: `more than 2` is `AtLeast(3)`,
because counts are whole and two directions are easier to answer than four.

**Symbols are accepted at the door and never kept.** The fair copy writes
`has exactly 2 X`, not `has = 2 X`: a spell is a file a player reads back, and
the canonical form should be the one someone who has never seen an operator can
still read.

**And the manual.** Nothing in the game taught the spell language — control words
are outside `Verb::ALL`, so `recall repeat` reached nothing, and the readings are
`NounKind::Sense`, so `recall marks` answered with a message about *other rooms
having words of their own*: a dead end wearing a wrong reason. There is now a
page per control word, a page per reading, and **`recall scripting`**, whose
last section is built from the room the way `help` builds its verb list — the
grammar is the same everywhere, what a question can *name* is not.

`every_word_a_spell_is_written_with_has_a_page` is what stops it rotting.

**One lint was strengthened rather than added.** Both colour checks skipped a
recipe whose input had no wash, so omitting a tint did not fail them — it
switched them off, for the product *and* everything downstream. That is how two
tinctures shipped colourless and took four products' coverage with them. Once a
recipe claims to be a mixture, a missing colour is now a failure that names both
the product and the input.

#### The language audited against seven others: `until`, comparators, `marks`

Asked to audit `.spell` against comparable languages and close whatever gaps stop
it working *in this game*, keeping the rule that state lives on the object and
there are no variables. Seven were read: **Autonauts** and **HyperTalk** and
**AppleScript** (English-like, `repeat until`, non-programmer audiences),
**Inform 7** (rulebooks sorted by specificity), **Factorio** and **Oxygen Not
Included** (comparisons over signals, no variables at all), and the
**Zachtronics** assemblers (complexity from constraint).

**What the audit confirmed was already right**, and it is most of the language.
No variables: Factorio and ONI reach the same answer, and ONI's single exception
is an explicit latch we have no case for. No `shout`/`hear` signal bus, which
Autonauts has and we do not need — a spell leaves a thing on a shelf and another
asks `if the cabinet has 4 fragment`, and signalling *through the world* is what
§8.1 requires anyway, since a bus is hidden state by definition. `wait` before
`if` as the unlock order, which is Autonauts' own. And a per-step tick cost,
which is Zachtronics' whole posture.

**What it found missing was one word.** Three of the seven have `repeat until` as
their *primary* loop, and we had `repeat <n>`, unbounded `repeat`, and no way to
leave either early — so *"walk until the maze closes"* was inexpressible and the
shipped solver said `repeat 20000`, a constant chosen to outlast the longest
walk, spinning uselessly once the maze closed. `until` is the sixth control word
and it earns its place by **deleting** a guessed number rather than adding a
capability.

Its guard is asked **before the first pass and again at the end of each**, which
is Autonauts' rule and the difference between a guard and a do-while: the first
pass is where a spell does damage, so a guard that cannot prevent it is not one.
A question it **cannot answer stops the loop**, which is the opposite of what an
`if` does with the same answer — declining to act is safe, while declining to
*stop* is a spell running for ever on an unanswerable question.

**Refused, and worth recording.** Inform 7 sorts rules by specificity so the most
specific fires first, which would turn our twenty-four-rung ladder into an
unordered set. Elegant, and wrong here: execution order would become *implicit*,
derived from a metric the player cannot see, where §8.1's contract is that a
spell is auditable and the log names the line. An `else`-ladder is worse to write
and better to debug, and the game is about debugging.

**`exit repeat` is deferred rather than refused.** AppleScript and HyperTalk both
have one; with `until` the guard covers the common case, and Autonauts ships
without it. Weighed, not overlooked.

**And a stale number came out of it.** `tests/solver.rs` pinned `worst <= 6500`
against a documented worst of **5123**; pinning the figure as an *equality* made
it report **5699**, which is what it had been measuring all along. A loose bound
cannot notice a stale claim about itself, so the ceiling now sits beside an
equality that says *is the ladder still the ladder*.

#### A spell counts, and a tester is handed a ladder rather than typing one

Asked whether §8's language could automate the archive — solve a maze for its one
fragment, and a gleaning maze for its five. **It already could**, and every part
was verified in the running game before anything was written: one spell file
solves a maze walked for its exit *and* one set to gather, re-opens the stacks
itself, and assembles a scroll. What was missing was proof, reach, and one word.

**The gap was a live defect, not an absence.** `if the cabinet has 4 fragment`
parsed as `if cabinet has fragment` — the number swallowed in silence, with
`interpret` displaying the shorter question. That is *"the orb writes down a
shorter command than it heard"* arriving through the one surface built to catch
it, and a player writing a guard got a branch that fired at one fragment.

So `has` takes a count. **At least, never exactly**, because a guard asks *have I
enough yet* and a walk pays into the cabinet while the spell runs — `has 4` that
went false at five would jam the moment a solver got ahead of it. `has 1 X` is
what a bare `has X` already meant and writes back bare, so every spell written
before this keeps its meaning and its spelling. `has 0 X` is `has no X`, decided
rather than discovered: read literally, "at least nought" is satisfied by an
empty shelf, which is a guard that always fires.

**The count is answered off the node's own `Stock`, and emphatically not through
`tower::holdings`.** `holdings` skips `Nameable(NounKind::Sense)` — correct for a
shelf, fatal here, because every maze reading *is* a `Sense` child. Routing `has`
through it would have answered *no* to `if north has passage` for ever and
deleted the archive's whole automation pillar while looking exactly like reuse.
An independent review caught it in a plan that presented it as a benefit.

**The solver ships to dev builds only, and that is a §12 decision rather than a
convenience.** `spells.toml` ships `first_light` deliberately unfinished — *"it
stops before the interesting part on purpose"* — and a complete twenty-four-rung
ladder handed to a player is the answer to the archive's central puzzle, `back`
included, which §19 above records was only found by measuring twenty mazes. So
`debug_spell` writes it, `cfg(debug_assertions)` like `debug_spawn`, from a
`dev_spells.toml` compiled in under the same `cfg`.

It departs from `debug_spawn` twice, both deliberately:

- **It refuses outside the spell's own domain**, where `debug_spawn`'s headline
  property is *any fixture, from anywhere*. That argument does not survive here:
  `scribe::write` homes a new spell to where the player stands, so writing an
  archive spell from the laboratory produces a file whose every line fails to
  resolve — a broken spell reported as written down.
- **It records the write, not the typed line.** `write_spell` already pushes a
  `Wrote`; recording the line as well would push two submissions for one input
  and a replay would push two more. Recording the *lines* is also the stronger
  guarantee — a replay reproduces what ran even after the file is edited, where a
  recorded name would silently pick up new text. The cost is that the tool's own
  two console lines are absent from a replay, which is the right thing to lose.

**One ladder serves both errands with no errand check at all.** `spoil` and
`exit` are both tiers; a gleaning maze withdraws the way out and an ordinary one
scatters nothing, so the inapplicable tier is simply never true. `if the stacks
has gleaning` remains for a spell wanting to do something *else* per errand. That
was true from the day the spoil rung was written and was **a doc comment** — the
test only ever pointed it at a gleaning maze, so a regression in the exit half
would have left the suite green. It is now asserted across four seeds.

**Two ladders are kept on purpose**, against this codebase's own instinct to fold
duplicates. `tests/solver.rs` pins a 6500-tick budget *to notice the language
getting slower*, and `SCRIPT_BUDGET` is 1, so the four always-false `spoil` rungs
cost a tick per move — 708 moves is +2832 ticks and the pin fails at ~7955. They
hold different properties: one the tick cost of an exit walk, one that a single
text serves both errands. Folding them breaks the first and proves nothing.

**And `threading` does not re-`research`.** A first draft opened the next maze
itself, which made a single walk's yield unobservable — the count keeps climbing,
and a test asserting *five spoils ended the walk* reads fourteen.

#### The typed path and the spell path have to answer the same question

A review of the scrolls found the same shape three times, and it is worth one
entry rather than three fixes: **a rule expressed twice, where only one of the
two expressions was ever exercised.**

`begins_work` was `const fn(Verb)`, so `wield` had one answer for two acts —
charging an instrument *is* a run, spending a scroll is not. `pipeline::wield`
returns before `work::begin` for a scroll, so a **typed** spend was never
charged the production slot, and its comment said as much. A **spell** reaches
`spell::block` first, so `wield quickening-scroll` in a script waited out the
very brew it was written to hurry, burned `PATIENCE` and gave up. *Quicken then
brew* is the obvious play, automating it is the point, and it was the one thing
a spell could not do — while `using_quickening-scroll` told the player *"wield
it anywhere, before a brew or during one"*. Both halves are one predicate now,
`execute::spending`, asked by both paths.

`carry`'s **destination** learned the arsenal exemption and its **source** did
not, so `move clarity to arsenal` worked and `move clarity from arsenal to
alembic` answered *"there is no /tower/arsenal within reach"* — a name resolving
at full confidence and then reporting itself unreachable, which is §15's worst
dead end and exactly what `tower::keep`'s *"nameable is not enough"* note
enumerates. `receiver` is now `addressed` and both ends ask it.

And a quickening scroll spent on the tick a run would land read `left == 0`,
where `(0 / QUICKENED_BY).max(1)` is 1 — so the scroll pushed `ends` a tick
*past* where it already was. `max(1)` was guarding a one-tick run becoming a
no-tick one, which is a different case; the interval may now only ever move
earlier.

**The fourth was the same lesson wearing content.** `mugwort-tincture` and
`valerian-tincture` were authored with no `tint`, and both colour lints
`continue` on an input with no wash — so the omission did not fail a check, it
switched the checks off for `keen-draught`, `quiet-draught`, `insight` and
`stillness` at once. A material with no colour draws in the base hue, which is
the failure mode §19 already calls total and invisible. The lint that skips
rather than fails is the next thing to fix here.

#### The cabinet is where fragments are kept, and the lectern is where four are spent

The `cabinet` was added for the trapped dust, and that reason went with the
byproduct. It has a better one: **it is the archive's shelf, and what the stacks
give up is stock.** A solved maze pays its fragment onto it; the player moves
**four matching fragments** into the lectern to assemble a scroll.

That is the laboratory's own loop, in the archive's words — take from the shelf,
charge the tool, wield — and it is worth having for three reasons beyond
symmetry. `survey cabinet` is where a hoard is, in one place, instead of hidden
inside the instrument that will consume it. The lectern holds a maze *and* an
assembly (§19 records it as the first instrument that can be doing two things at
once), so keeping a growing pile out of it is one fewer thing overlapping. And
the arithmetic is now visible: four is a number a player can see themselves
reaching.

**It also closed a split the tool had opened.** `tower::home` sends a *spawned*
fragment to the cabinet, and a *solved maze* was putting one in the lectern — the
same word in two places depending on how it was got, which is the shape of the
`Fragment`/`Reagent` kind defect one field over. Both paths ask `tower::home`
now, so neither can drift, and so does the test that counts them.

**"Matching" is doing nothing yet, and is the right word anyway.** There is one
generic `fragment`, so any four match. When specific fragments arrive — §19 above
records that as the intended shape, waiting on a reason to prefer one maze over
another — the sentence does not change, and neither does `Recipe::count`, which
already asks for four of *one name*.

**What it costs, stated rather than discovered.** `move` carries one unit, so
assembling is four `move fragment to lectern` and a `wield`. Against four walks
of the stacks that is noise, and it is exactly the tedium the archive is built to
automate — `repeat 4 / move fragment to lectern / end` is four lines of a spell.
If it ever grates, the lever is `move` learning a count, and that is a signature
change with the optional middle slot to think about rather than a quick fix.

Without the cabinet, `empty lectern` would also be a word that can never work in
the room it is offered in.

Measured rather than chosen: 572 against `combine`, its nearest word, where the
resolver's floor is 600. `shelf` and `chest` both land *on* the floor (600,
against `help` and `check`); `press`, `stacks` and `carrel` are over it. `almery`
— a monastic book cupboard — is safest at 429 and was passed over for being a
word nobody types on a first guess, which is §19's own objection to the four
invented shard names.

**It moved the numbered prompt, and the pin is what said so.** Places sort by
full path, so `/tower/archive/cabinet` displaced `east` as the fourth reading a
bare `purge` offers — a fixture added for a reason two rooms away changing what a
player is shown, with nothing on screen saying so.
`a_bare_anything_verb_offers_the_same_four_readings` exists for exactly that and
caught it on the first run.

**And the boundary it exposed is still there**, stated rather than left to be
rediscovered: a *reagent* cannot cross a domain boundary, because the arsenal
takes finished work only. Nothing needs it to today — the archive's stock is
fragments, which the lectern consumes in the room they are won in — but the day a
domain wants another's raw material, this is the decision, and it belongs with
Phase 8's pane addressing rather than with a byproduct.

#### And the tool learned the room

`debug_spawn` grew a destination when the archive gained an instrument it could
not reach; the arsenal is a **domain**, so the fixture test refused it and every
arsenal state went straight back out of a tester's reach — the same gap, one
change later. The rule is not what shape a node is but whether
`pipeline::reachable` can see into it, and it can see into exactly two things: a
fixture where you stand, and the arsenal from anywhere.

**Its door holds for a tester too.** Stock standing in the arsenal is a state no
`move` could produce, which is the same objection this word already answers three
times over — an unknown name, a nought count, a wrong noun kind. A testing tool
that can build impossible worlds is one whose bug reports have to be checked
against the tool first.

**And a `way` is a fixture and is not a shelf**, which the fixture test alone got
wrong. `north` and its three siblings carry `Fixture` so the maze can publish
readings into them, and `research::refresh` despawns everything in a way on the
step after — so a reagent put there is a pile that vanishes with no line saying
so.

Two lints hold the promise that *everything* is reachable, because a promise kept
by hand is one kept until somebody is busy. One drives every offered name through
a real `Sim`; the other checks the direction that rots — a material authored in
`materials.toml` that no recipe names would have a colour, a manual route, and no
way for a tester to hold one, with both files parsing perfectly.

#### The sigils, and the noun kind that went with them

`sigil-iv`, `sigil-ix` and `the-quiet-page` sat on the archive's floor from the
day it was built, and were the last of the `divine` that consumed a fragment.
Once `research` opened the stacks instead, nothing produced them, nothing
consumed them and no prose said what one *was* — which is verbatim the complaint
recorded above against `shard-of-dawn` and its three siblings: *"four invented
names standing in for a decision nobody made"*. Three more of the same, one room
over, and they survived that clean-up because nothing pointed at them.

**`NounKind::Fragment` went with them, and the argument is §15's.** No verb's
signature ever asked for one — `grep` the signature table and there is no
`Fragment` in it — so the kind sorted nothing and gated nothing; and once the
maze's yield became a `Reagent`, nothing in the world was one either. A kind with
no instances and no slot is an API with no callers, which §19 has deleted before:
`Depiction::Tool` and its four solved ramps went for exactly this.

**§11.5's Fragments row is unaffected**, and it is worth saying why. That row is
about the *resource* — what the archive yields and what research consumes — and
the resource is a named material, `fragment`, which is exactly as real as it was.
What is gone is a parser category that never told anything apart. It comes back
the day a slot needs to refuse a reagent while accepting a fragment, and not
before.

**No version bump.** Nothing moved from `[ ]` to `[x]`; this is a correction
folded into the step above, which CLAUDE.md's *Finishing a step* says advances
nothing.

### A scroll does something, and the stacks gain an errand

Four walks of the stacks assembled a `spell-scroll` and that was where the archive
stopped: an object with a name, a colour and no use. §15 weighs the dead-end rate
above the raw resolution rate, and this was the largest one in the game.

**The design document had no theory of what a scroll was for.** It appears once
above, as *the thing four fragments become*. So the mechanism is decided here.

#### `wield`, not a twenty-third word

`verb.rs`'s vocabulary test argues the case and refuses the alternative in
advance: *"22 is a number to defend, not a budget to spend: the next word added
here needs an argument of this shape. `wander` is the last one this reasoning
stretches to."* Spending a scroll is *setting a thing going*, which is what
`wield` already means, so the verb learned a second argument kind —
`NounKind::Workable`, accepting a place **or** a scroll — on the `Stoppable`
pattern that exists for exactly this.

**`empty` split off and kept `PLACE`.** The two shared one signature, and
widening it would have made `empty gleaning-scroll` a sentence the parser accepts
and the executor cannot answer.

**The scroll branch returns before `start`.** `begins_work` is `const fn(Verb)`
and cannot see the argument, so branching in the handler is the only place a
scroll can be kept out of the production pool — and it must be, because at
`CAPACITY = 1` a scroll would otherwise be refused whenever anything was running,
which is exactly when a player reaches for one.

#### What four fragments become is drawn, and that is not the attrition we removed

Nothing about the inputs could decide which scroll comes out — four fragments are
four fragments however they were won — and a lectern that always made the same
thing is §10's objection to the old `research` one level up: a duration with no
decision content.

**This is not the roll that was taken out.** That one was four *distinct*
fragments drawn uniformly and collected into a set: 8.3 solves per scroll, and no
way to aim for the one you lacked. The draw is on the **output** now, every
result is immediately usable, and there is no set to complete.

**The list lives on the recipe, not in three `[[lectern]]` blocks.**
`Recipes::matching` returns the *first* recipe whose inputs match, so three
blocks all wanting four fragments would leave two permanently unreachable with
the content file looking perfectly reasonable. `Recipe::outputs()` is the
counterpart of `inputs()` and takes the same two spellings for the same reason.

**A recipe that makes one thing rolls nothing**, and the guard is not an
optimisation: every completion passes through `transmute`, so an unconditional
draw would advance `RngStream::Archive` on every grind and every distillation —
the cross-subsystem coupling this document already records fixing once, when
solving a maze rolled the laboratory's `Yield` stream and changed a player's brew
yields.

#### The errand is a word on the lectern, not a `State`

The maze gained a modifier — `Errand::Way` or `Errand::Glean` — and a spell has
to be able to ask which, or the player needs two solvers and no way to tell which
maze they are in.

It is published as an ordinary **named child of the stacks**, exactly as a way
publishes `passage` and `back`, so `if the stacks has gleaning` is answered by
the `has` question §8 already has. A `State` variant was the other candidate and
is worse three ways over: `State` is a closed set read by `State::is_busy`, the
panel's `bar_of` and the spell language's `is working` / `is idle` at once, and an
errand is not a state of the *instrument* — the stacks are doing exactly what they
were doing before.

`Errand::ALL` chains onto the readings in `scene_at` beside `back` and `spoil`,
because a condition resolves at **cast** and there is never an errand on at that
moment. That is the same single line the whole solver design already rests on.

**A gleaning maze publishes no exit at all.** Leaving one that did nothing would
be a trap rather than a change: a solver's top rung is `if <way> has exit`, so it
would walk onto that square, find the walk not over, and take the same rung from
the same place for ever. The picture withdraws `Ω` with it — a map offering a way
out that the readings do not is the one mark on screen that lies.

**Five spoils against the four a scroll costs, profitable on purpose.** Gleaning
is what keeps scrolls in circulation and makes automating the maze the engine §10
says the archive is meant to be. It is also the largest balance exposure in the
feature and **nothing sweeps it**: `orbs-balance` is still a stub, so the number
is a placeholder and is written down as one rather than implied to be tuned.

#### Two defects the work uncovered

**`tower::holdings` counted a reading as stock.** A child with no `Stock` reports
one unit, and the lectern is an *instrument* — so an errand parked on it would
have entered the multiset `Recipes::matching` compares. Four fragments plus one
word is not four fragments: the recipe would stop matching and the panel would
read `fouled` for a lectern with nothing wrong with it. The ways got away with
publishing readings only because nothing ever asks them what they hold. Fixed at
the source — `holdings` skips `NounKind::Sense`, which is true everywhere and not
a special case.

**`debug_spawn` could not reach the archive at all.** It walks to the tower's one
`Store`, and that is in the laboratory, so §7 made every archive state
unreachable from the tool that exists to reach states — four fragments on a
lectern could be had by walking the stacks four times and by nothing else.
`tests/solver.rs` had already written the impossibility down in a comment and
worked around it by hand. It now takes a destination (`debug_spawn fragment 4
lectern`), a fixture anywhere, on the same argument the shelf lookup already
makes: requiring the tester to stand in the right room first puts back the
walking the tool exists to skip. A numeric third word is still refused at the
parse, so `debug_spawn sage 2 3` does not become a place called `3`.

### The archive is a maze, and the world holds the search

§10 calls the archive **bespoke** — *"played most, and stales fastest"* — and
named a resource sink only as the budget fallback. It was five entities and a
verb that consumed nothing, produced nothing and could be run on the same sigil
for ever. It is now the stacks: `research` resolves them out of the lectern,
`follow` threads them, the way out gives up a shard, and four shards make a scroll
(`Recipes::matching`, so the assembly half needed **no new mechanism**).

#### The finding the whole design rests on

§8's language has no variables, no counters and no numeric comparison. A maze
solved by *searching* would be the one room in the game that permanently defeats
pillar 3 — you could never teach the orb to do it.

**Unless the maze holds the search's state.** Trémaux's algorithm needs no memory
beyond marks in the passages, so the cells mark themselves and the four ways
publish what is adjacent as ordinary nodes. A solver is then a rule, not a
search, and it is **depth-first search performed physically**: the marks are the
visited set and turning back is the stack pop, because the reading head *is* the
stack pointer.

This is the strongest defence §19's refusal of numeric comparison has, and it
belongs where the next person is tempted: **the language did not need to grow,
the world needed to remember.**

> **Amended by *"a spell counts"* below, and half of it stood.** The world does
> the remembering and always did — `Square::marks` is the visited set, and no
> variable was ever wanted. What was wrong is the other half: the language *did*
> need to grow, because the count the world kept was being read through two words
> (`walked`, `twice`) that could not tell a square walked twice from one walked
> forty times, so *prefer the least-walked way* was a rule the world could answer
> and the language could not ask. A comparison adds no memory. It stops the
> language reading a `u8` through a two-value lens.

#### One thing had to give, and it was the vocabulary

The claim was first made as *"no change at all"*, and that was wrong.
`spell::compile` resolves a condition's names against the room **as it is at that
instant**, and nulls the whole condition for a name it cannot place — a guard
added because `has ground-slat` answered "no" for ever. No cell is `walked` at
the moment a solver is *cast*, which is exactly when its names must resolve, so
every `if` compiled to a branch taking neither half. `bind::stand` recasts every
lap, so the deadness would have changed lap to lap.

The fix is `NounKind::Sense` and a fixed vocabulary the scene always offers —
`passage`, `wall`, `walked`, `twice`, `exit`. **A kind no slot asks for**, so a
reading can never fill a `Reagent` by accident while `Any` still finds it. The
four ways are `Role::Reading` places: they have to be `NounKind::Place` because
that is the only kind the place half resolves against, and being places is why
they need `Protected` and their own exclusion from the instrument panel.

`tests/solver.rs` was written **before the generator, the verb or the picture**,
and pins all of it — including that a misspelled `walkd` is still refused, so
offering a vocabulary did not buy resolution by disabling the guard that made it
necessary.

#### The naming, and what it cost

**`step` scores 750 against `stop`** — over `MIN_SIMILARITY`, and a typo that
stopped a run instead of advancing it would cost the whole maze. **`tread` scores
800 against `read`**, which `peruse` claims. `follow` is 429 against its nearest
and shares no three-character prefix. All three computed before the name was
chosen, which is the practice `wield`/`kindle` set.

**`follow` is the 21st tower-wide verb and is recorded as a debt.** It means
nothing outside the archive and by rights would be an operation scoped to the
lectern — but `Scene::offering` derives scope from the `Operation` component and
a fixture carries exactly one, which the lectern spends on `research`. **Scoping a
second verb to one instrument is the missing mechanism**, and until it exists
this word is global.

#### What the instrument retired

`research`'s completion had no `Message`, a running `research` could not be stopped
(`stop` finds its target through `Fixture`), and `recall archive` reached
nothing. All three were one absence — the archive had no instrument — and the
first and third went when it got a lectern.

**The second did not, and the first telling of this entry said it had.** `research`
inserts no `Working` at all now, because reading takes no production slot — so
`stop lectern` found an instrument, had nothing to stop, and said so. The defect
was not retired; it was made *moot*, which is a different thing and reads the
same from outside. It is fixed properly now: `stop` on a lectern holding a
maze **abandons** it, which is also the answer to a player stuck in one
they cannot solve. A claim that a defect is gone is worth exactly as much as the
test under it, and this one had none. With them went
`DIVINE_TICKS`, `pipeline::work`, `progression::DIVINE` and its escape from
`check`, and the verb-keyed earn: **the archive pays through its instrument now,
like every other room.**

`research` also stopped taking a fragment. It named one while it was a twelve-tick
command that consumed one; it opens the stacks, and there is one place to open
them at.

#### Two shapes borrowed, and what each cost

**The four ways are places you cannot go.** They must be `NounKind::Place`,
because that is the only kind the place half of a spell's question resolves
against — without it `if north has passage` cannot be written at all. But scene
places are attendable, so `attend north` walked into a compass bearing until an
explicit refusal was added. That is the second spatial system this design was
warned against, arriving by default rather than by drift, and it is held off by
one guard on one component.

**The stacks report as `Working` with a meter of floor walked.** They take no
production slot, so `Working` here is the *panel's* state rather than the
component — which is what lets a solver ask `if lectern is working` to know
whether its maze is still open. The meter is the only honest one a maze has: a
brew knows its duration before it starts and a maze does not, because how long it
takes is what the player's rule decides. What can be reported is how much has
been seen, and that only ever grows, which is what a bar must do.

`Craft::Reading` takes `Bar::Read`, which is **meterless** — unlike `Bar::Plain`,
which draws *nothing* when an instrument reports no meter. That defect has
shipped twice in `shell/panel.rs` and is documented there twice; a lectern with
no maze open is exactly the state that would have made it three.

#### And a slot it deliberately does not take

**Opening a maze holds no production slot.** `CAPACITY` is 1 and `PATIENCE` is
120, and a solve is hundreds of ticks — a solver holding the tower's one slot
would starve every other spell into `spell_gave_up`, which is precisely the
bind-it-and-go-and-brew case the design sells. Reading is not a *run*, the same
argument `start` makes for the athanor.

#### A solver that reaches the exit, which nothing had shown

`tests/solver.rs` shipped with the maze and pinned four things, every one of them
about the **cast**: that a Trémaux solver's conditions survive `compile::fix`,
that the readings resolve with no maze open, that `walkd` is still refused, that
the spell writes and casts end to end. Read together they look like proof that a
player can automate the archive. They are not. *Survives the cast* had been
quietly standing in for *reaches the exit*, and the two are different claims —
the pillar the whole domain rests on was the one thing untested.

It was not idle worry. **The obvious solver does not work.** Sixteen `if`s in a
row — take the way out, else an unwalked passage, else the least-walked way back,
four ways per tier — parses, casts with no fault, walks two cells and then
oscillates for ever. The `passage` tier steps into a fresh cell and the `walked`
tier, four lines later *in the same lap*, reads the cell just left and steps
straight back. Guarding the retreat behind *nowhere new to go* moves the pendulum
down a tier rather than removing it: the `walked` rung steps back and the `twice`
rung returns.

**`else` is what fixes it**, and the reason is worth stating plainly because it
is the first real lesson §8's language teaches. A ladder of `if`s is *read* as
"the first line that matches, and then stop" — and it does not mean that. It
means "every line that matches, in order, against a world the earlier lines have
already changed". `else` is how the language says the thing the shape implies:
one move per lap, by construction. Twelve seeds now sweep to a shard, worst 677
ticks against a pinned budget of 900.

This is also the sharpest argument yet for §15's *"tests prove code does what it
was written to do; they cannot prove it is the code worth writing"*. Four green
tests, all correct, all testing the wrong half.

### The map, and the fog being the reading's own knowledge

The archive's maze was complete in the sim and invisible in the game. A bound
solver working for four hundred ticks showed a two-cell gauge creeping up the
panel — §10.1's `bar_of` conceded as much, drawing the plain gauge *because* "a
maze's picture is the map (its own item)". It is now `orbs-render/src/maze.rs`,
placed by `orbs/src/shell/stacks.rs`.

**Three decisions carried the item.**

**The picture lives in `orbs-render` and the sim builds one.** `orbs-sim` depends
on the render crate and never the reverse, so a `Stacks` description crossing
the boundary is the only arrangement available — and it is the better one anyway:
`Maze` keeps its cells private, and the fog is decided in exactly one place
(`Maze::view`) rather than in each frontend's painter. `Instrument`'s `Wash` is
the same shape and the precedent for it.

**A wall is drawn only where the reading has *stood*.** Not a difficulty setting
— it is precisely what the four `survey` readings told the player, so the map
carries nothing the linear stream lacks (rule 2) and a player with squared paper
could have drawn it themselves. Two kinds of knowing had to be kept apart to say
that, and collapsing them loses the interesting one: a cell you have **stood in**
proves its four walls, while a cell a walked neighbour merely **opens onto** is
one you were told about — `north has passage` — so it is on the map with nothing
known about its own walls. The first draft made the second mean the first, which
is defensible right up until you notice the picture then has no way to say *there
is somewhere through there I have not been*, which is the one thing a player
reads a map for.

**Columns, never rows.** The instrument panel takes a side or a strip depending
on the pane's shape, and following it is the obvious thing and wrong: under a
`Top` panel the map would take rows, and 17 rows out of 22 leaves the deep-focus
floor a five-row transcript — a map that ate the thing it exists to be read
beside. One orientation-independent rule covers every grid the game runs at. It
is also why the map splits **after** the panel: whichever runs second is the one
whose refusal can fire, and an instrument row is load-bearing where a map is a
convenience.

Two smaller ones, both recorded because the alternative looks reasonable.
**It refuses rather than truncates** — a maze drawn short is not a smaller maze,
it is a wrong one, which is the same reason `research` bounds the generator's width.
And **it says nothing continuously**: the border title is announced as a heading,
the four ways are what `survey` answers, and the walked count is on the lectern's
own panel row, so a second per-frame utterance would be §14's *"progress
announcements: completion only"* broken by the surface that most wants to break
it.

### `wander` — the 22nd tower-wide verb, and what it is not

Walking a maze meant typing `follow east` fifty to a hundred times. `wander`
gives the arrow keys the stacks.

**The seat and the debt are two different arguments and both have to be made.**
`verb.rs` records that 21 was "a number to defend, not a budget to spend", and
this word is `unfurl`'s case and `follow`'s case at once. The seat is `unfurl`'s:
who owns the arrow keys has no other way to be said, and §6.1's exception is
exactly for a word that makes a mouseless game navigable. The debt is `follow`'s,
unchanged and **not doubled**: this is a domain's word wearing a tower-wide coat
for one reason, that `Scene::offering` derives scope from the `Operation`
component and the lectern spends its only one on `research`. Two words waiting on
one missing mechanism is an argument for building the mechanism. It is not an
argument for a third, and the count now says so.

**Bare `follow` was the alternative, and was declined rather than overlooked.**
It costs no vocabulary — `follow` is already tower-wide and already has a
no-argument branch — but it would make `follow east` and `follow` do
categorically different things, one walking a cell and one seizing the keyboard,
and making the slot optional loses the numbered prompt that a required slot gives
every other verb. Recorded so it is not re-proposed as an oversight.

**It opens no surface.** The map draws whenever a maze is open, which is what
makes a bound solver watchable for nothing, so the word changes only who the
arrows belong to — and walking a maze by hand and watching a spell walk it are
deliberately the same picture.

Three consequences, each of which had a plausible wrong answer.

**An arrow is a submission, not a move.** Pressing right writes `follow east`
into the ordinary stream, so there is no second walking implementation, no
frontend reaching into the world, and `(seed, submissions)` replays a
hand-walked maze exactly as it replays a spell-walked one.

**The queue was never the problem; the tick was.** Three versions went through
`submit`, and the first two are worth keeping because they bracket the answer.
Submitting per keystroke walks at the speed of the *keyboard* — `Pending` is
drained whole at tick start and key repeat is unfiltered, so a held arrow was
about thirty cells at once. Keeping one aim and replacing it, then a bounded
burst, walks at the speed of the *world*, and a maze at 1 Hz is a wait rather
than a minigame. No amount of queueing fixes that, because the queue was solving
the wrong problem.

So `Sim::walk` is a **third entry point**, alongside `submit` and `step`, and it
is the narrowest one that answers the question: it moves the reading and nothing
else. **No tick is consumed** — no brew advances, no fire burns down, no spell
runs — so a player walks as fast as they can press, and standing in a maze costs
world time only in the sense that they are standing there doing it.

Three doors is one more than this document has ever wanted, so it is worth being
explicit about what keeps it honest.

**Replay is not weakened, and the recording is why.** A typed line is recorded
against the tick it was *queued* on and executes at the start of the next; a walk
executes immediately, so it lands after that tick's step. Both are exact, and
`Submission::Walked` is what lets a driver tell them apart rather than guess.
A tick can never hold both kinds, because the prompt is dead while the arrows
have the maze. This is tested rather than argued — a hand-walked maze replays to
the same cell — and the replay driver now lives on `Sim` rather than being
hand-written at each call site, which it was in three places.

**One body, two clocks.** `follow` and `Sim::walk` both go through
`research::tread`, so a hand-walked maze and a spell-walked one cannot disagree
about a wall or about what reaching the exit is worth.

**And the balance question it raises, stated rather than dodged**: walking by
hand is now much faster than a bound solver, which took 677 ticks at worst. That
does not make automation pointless, because the value of a bound spell was never
speed — it is that it works while you are somewhere else brewing. But if the
archive ever needs the two to be closer, the lever is here and this paragraph is
where to look for it.

**Walking takes the pane; watching does not.** Adding the fourth term to
`type_into_line`'s guard makes the prompt dead, and this document already has the
rule from the editor — drawing a caret that cannot accept a keystroke is the
clearest possible lie about where typing goes. The first version kept the
transcript and replaced only the input row, on the argument that a player wants
the running commentary. In practice standing in a maze is a *mode*, and a screen
that still looks like a session is offering something it cannot do. So `wander`
now takes the whole pane like the editor, with the maze centred and the walked
count and the keys beneath it.

**The two views are the split that matters**, and it is worth naming: a spell's
solving stays inline beside a live transcript, because watching and doing are
different activities and only one of them owns the keyboard. The picture is the
same picture; what differs is how much of the screen the player has given up.

The consequence to state rather than discover: **Escape is the only way out**,
since `attend` needs the prompt, so "the player walks out of the archive while
wandering" is unreachable rather than handled.

#### The corridors, and a picture that lied about progress

The map's first version drew an open wall segment as a blank. Every cell has a
wall line on either side of it, so a walked path came out `▒ ▒ ▒ ▒` — mark, gap,
mark, gap — and read as *every other cell has been visited*. It was reported by
looking at it, which is the whole of §15's argument in one line: the code was
correct, the tests were green, and the drawing was saying something false.

An open passage now carries the corridor's own mark, so a walked run is solid.
**The lesser of the two cells' marks**, because a passage has been used at most
as often as the cell it leads to — a corridor claiming `once` between a
once-cell and a five-cell would report a route nobody took.

**And what the reading has only been *told* about draws nothing at all**, which
is the other half of the same lesson and was got wrong the same way. A cell a
walked neighbour opened onto had a `·`, and so did the corridor leading to it —
so every unexplored way out of the region cost **two** dots, and a head with
three ways out sat in a small constellation of them. It was defended as *the
frontier reading as frontier*, and on a screen it read as speckle: the gap in the
wall already says a passage is there, so the dots were the same fact drawn three
times. The frontier is the hole in the outline.

The exit survives the cut because it is a *different* fact rather than more of
the same one, and it is the one thing `Chamber::seen` still decides.

### The manual, and `help` asking a question instead of answering one

`help`, `man` and `?` have been synonyms of `recall` since §6.1's register table.
Typing any of them opened a **numbered prompt** offering `archive`, `brewing`,
`clarified-draught` and `clarity` — the four alphabetically-first manual
subjects, chosen by nothing.

The cause is a parser rule doing exactly what it says. `recall`'s slot was
`Slot::required(NounKind::Topic)`, and a required slot with fillers cannot yield
an argument-less intent: `resolve::collect` pushes one candidate per filler, they
tie, `analyse` returns `Ambiguous`. The scene always has topics, so the
no-argument branch in `execute::recall` was **unreachable code** — which is why
the first draft of the plan for this proposed rewriting it and would have changed
nothing on screen.

So this was never a missing feature. It was §6's *no bare error* failing at the
one command whose whole job is answering the question, and it survived because no
test ever asked what `help` did.

**`recall` takes `TOPIC_OPTIONAL`, and that needs answering rather than
assuming.** §19 records making a slot optional as *declined* for bare `follow`,
on the grounds that it loses the numbered prompt a required slot gives every
other verb. The difference is `survey`: `follow` bare and `follow east` are
categorically different acts — one walks a cell, one seizes the keyboard —
where `survey` bare and `survey alembic` are the **same act at two scopes**,
which is precisely what `recall` and `recall grind` are. And the prompt being
lost was never a disambiguation: nothing had been typed to disambiguate, so it
offered four arbitrary subjects rather than four readings of an input.

**Three fixtures moved, and that is the change proving itself.** `brew` is a
`recall` synonym and was the ambiguity fixture in `parser::trace` and
`tests/parsing.rs` — a bare `brew` now resolves at full confidence, so those
tests were left asserting nothing. What they are *about* is unchanged; the
fixture is `purge`, a required `NounKind::Any`.

**The overview cost no render code.** `RecordKind::Section` stacks and draws as a
`[heading]`; `Entry` tiles across the pane. `survey` already emits exactly that
pair, so a clap-shaped listing was two existing shapes in a new order — and
because the entries are records rather than a formatted string, `sift` still
works on them and §14 hears one utterance per verb instead of a wall of spacing.

**One filter, shared.** `execute::offered` — live, ungated, in scope — was
written out inside `boot.rs`. Two copies of that rule is two chances for the
tutorial a player reads at launch to disagree with the manual they ask for a
minute later. Its scoping half generalises what the report hardcoded: at the
tower root an empty scene offers no operations, so `Scene::offers` and
`!is_operation()` name the same list.

**`Verb::group()` is a table, not a derivation.** The predicates that exist group
by the wrong thing — `is_operation` is about scope, `transmutes` about the
pipeline. What a lost player wants is sorted by what they are trying to do, which
is a judgement. It also gives `is_destructive` its first reader: it had none, and
a test now asserts the `Careful` group and that predicate name the same two
verbs rather than one being trusted.

**Known and left alone**: `research`, `follow` and `wander` list in the
laboratory, where they refuse. They are tower-wide because `Scene::offering`
derives scope from the `Operation` component and a fixture carries one, which the
lectern spends on `research` — the debt §19 already records against `follow`. The
overview makes it *visible* rather than causing it, and it retires when a second
verb can be scoped to an instrument.

#### The manual, written

Five sentences of documentation became 27 pages — ~170 authored lines, and the
first time the game can answer *what does this word do* from inside itself.

**The lints check that a page exists, fits and points somewhere real. They cannot
check it is worth reading**, which is the whole risk of this item and the reason
it was reviewed by reading dumps at the 80x22 floor rather than by reading the
diff. What they do catch is drift: a verb added later fails the completeness lint
as well as `Verb::group`'s wildcard-free match, a synopsis that stops naming its
required slots fails, and a `see also` naming a word the parser lacks fails —
which is worse than pointing nowhere, because a player types it.

**Two pages say what a lint could not.** `undo` is in `Verb::ALL` and not in
`is_live`, so the word resolves and does nothing; its page says so and sends the
reader to `stop`. `bind` is gated at concentration 0; its page says what it
costs. The overview omits both, as `boot.rs` does, because a word that can only
refuse is a dead end in a listing — but the *manual* should explain what exists,
and §15 weighs the dead-end rate above the raw resolution rate.

**`follow` is exempt from the synopsis check**, and the exemption is a debt
rather than an oversight. Its slot is `NounKind::Place` because the place half of
a spell's condition resolves against exactly that kind, which is why the four
ways are places a player cannot stand in. So `follow <place>` is honest about the
implementation and wrong for a player, who is choosing a direction — the
synopsis reads `follow <way>` and the test asserts *that* instead. It goes when
the ways stop needing to be places.

#### A page that read as a wall, and the three things making it one

The first manual pages drew as a solid block. Reported by looking at one, with
every lint green — the lines fitted, drew in CP437 and pointed at real words, and
none of that is the same as reading well. Three causes, and only one of them was
in the manual.

**Prose is capped at 70 cells, so a description is written in pieces — and
emitting those pieces as separate records made them *hard* line breaks.** In a
100-column pane a paragraph written at ~50 came out as ragged strips down the
left. The pieces are joined into one record now and the pane wraps it to whatever
width it actually has, which is what `RecordView` was already for.

**Every record took the same lead**, so `[what it does]` sat flush with its own
body and the page had no hierarchy at all. A `Section` draws at the margin now
and everything else keeps its two cells — outdenting the heading is the same
shape as indenting the content and costs no cells. `survey`'s `[place]` headings
got it too, which is the tell that it was the render layer's problem rather than
the manual's.

**A row per synonym made the vocabulary longer than the description.** `attend`
has five spellings, four of them plain. One row per *register* now, phrases
joined — arcane first, because that is what the echo teaches.

The lesson is the same one §15 keeps making, one layer up: the lints could check
that a page exists, fits and points somewhere real, and none of them could check
that it was *shaped* like a page. That needed eyes on a screen.

#### `NounKind::Command`, and the kind that had to be unreachable

`recall grind` has to resolve, and the obvious way is to register every verb
canonical as a `NounKind::Topic` beside the recipe outputs — which is exactly
what recipes already do. It is wrong, and `NounKind`'s own doc had already said
so about a different word: *"not `Topic`, which `recall` reads: `recall walked`
would resolve and then find no manual entry."*

The reason is that `Any` reaches `Topic`, and three things read `Any`:

- **Tab** offers what a slot accepts, so `purge gr` would have suggested `grind`
  — a word the parser refuses, which is the dead end §15 weighs above the raw
  resolution rate.
- **`spell::compile`** resolves a condition's *thing* names through `Any`, so
  `if the dispensary has grind` would have compiled clean and answered *no* for
  ever. That is verbatim the `has ground-slat` defect `compile::fix` was
  rewritten to kill.
- **The numbered prompt** orders by (kind, value, slot), so inserting 27 nouns
  would have moved which four readings a bare `purge` offers, silently.

So `Command` is a noun kind **`Any` does not accept**, reachable from exactly one
slot kind — `Subject`, which takes `Topic | Command`, the shape `Readable`
(File|Script) and `Stoppable` already have.

**It is the `Sense` argument run backwards**, which is worth noticing. `Sense`
needs `Any` to find it, so a spell's `if` can name a reading that does not exist
yet; `Command` needs `Any` *not* to, so a spell cannot name a verb as a thing.
Two kinds, one mechanism, opposite requirements — which is the case for a slot
kind rather than a wider `Any`.

**The bare-`purge` prompt was pinned before any of this moved.** A test written
in the previous step records the four readings it offers; the guard here is that
they are unchanged. That is the only way a silent reordering could have been
caught, and it had to exist first.

**Pages are readable from anywhere; the overview is scoped.** A manual you can
only read in the room the tool is in has a lock on it, and §7's rule is about
*acting* — places and spells already carry the same exemption. So `recall grind`
answers from the archive while a bare `recall` there does not list `grind`. The
asymmetry is deliberate and is written down because it reads as a bug otherwise.

**Three subjects were retired to make room.** `recall_unfurl`, `recall_research`
and `recall_wander` made those three verbs `Topic` nouns already, so the page
branch would have shadowed a working answer while `complete::nouns` — which does
not dedupe — offered the word twice. Their prose moved into `man_*` and the keys
are gone.

#### The fog is gone, and what it cost to remove

The map drew only what the reading had stood in or beside — exactly what the four
`survey` readings answer, which is what let it claim to carry no information the
linear stream lacked. It now draws the maze **whole** from the moment `research`
opens one.

**The trade is §14's and it is the one asymmetry that rule exists to prevent**: a
sighted player can now see more than a listener can. It is stated here rather
than quietly absorbed. Two things make it survivable. A *spell* still solves the
maze from the four readings alone, so pillar 3 is untouched and the domain's
automation is exactly as reachable as it was. And walking by hand is now a
routing problem rather than a feel-along-the-wall one, which is a better minigame
and the reason for the change.

If the asymmetry does bite, the repair is a **spoken bearing to the exit** — a
listener would then have *more* than the fog ever gave them — rather than a
return to fog. Recorded so the next person reaches for that first.

Unwalked floor still draws as nothing, which is not a remnant of the fog: the
walls around a corridor are on screen, so the corridor is the gap in them, and a
glyph there would be a third way of saying what the wall already says. That is
what the `·` was.

#### 16×16, a denser carve, and the solver that was never a solver

The maze went to **16×11 cells** — 33×23 squares, 192 cells, four times the
floor — and the generator from a recursive backtracker to **randomised Prim's**.
A backtracker carves one long path and turns only when it has to, so its mazes
are a few very long corridors with the odd stub; Prim's grows outward from
everywhere at once and gives short passages, frequent junctions and many small
dead ends. That is the difference between a maze you read at a glance and one
you have to walk.

The first version of this refused to draw at all where the whole picture would
not fit — the rule the map had always had, on the argument that half a maze is
not a smaller maze but a wrong one. That held while a maze was 15 squares and
fitted everywhere; at 33 it meant the map simply **vanished** from the 80×22 and
100×28 floors, which is not honest, it is absent. A player at a small window got
nothing rather than the part of the maze they were standing in.

So a short pane gets a **window centred on the reading**, clamped inside the
maze so it never shows emptiness past the edge, and walking pans it. When the
whole picture fits, the window is the whole picture and nothing moves — the
common case is still a still. Only a keyhole, under nine columns, is refused:
below that there is no junction to read and the transcript is the better use of
them.

**And it broke automation, which is how we found out the solver was never a
solver.**

The four-tier ladder — exit, unwalked, walked, twice — reads like Trémaux and is
not. Trémaux's actual rule is *"when you arrive at a junction you have seen
before **by the passage you came along**, turn back"*, and nothing in §8's
language could say which passage that was. Without it, at a junction where two
ways read alike, a fixed compass order sends the reading back where it came from
and it **cycles for ever**. It solved 7×7 backtracker mazes and nothing harder:
measured across a sweep, 4 of 8 at 16×16, and 1 of 12 on Prim's mazes at 7×7.

Not slow — cycling. Which means the acceptance test that proved a solver reaches
the exit had been proving it about the only mazes the flaw survived, and the
archive's automation pillar was resting on the generator being weak.

**The fix is one word: `back`.** The way the reading last came from, published as
a *second* child on that direction — a way can be `walked` and the way you came
at once, and the two answer different questions, so it is not a fifth `Sense`. A
five-rung ladder that excludes it in the middle and retreats along it last solves
every maze tried: both generators, both sizes, 12 of 12, in at most 708 steps.

This was already written down as a future Mastery unlock — §19's solving ladder
lists *"a heading, and relative senses"* as the rung after auto-marks. It turns
out the *first* rung was never complete without it, and a bigger maze is what
made that visible. §15 again: the tests were green and the code was correct, and
the thing being tested was easier than the thing being claimed.

**Sixteen by eleven rather than sixteen square**, because a character cell is
8×16 pixels: a grid square is a tall rectangle on screen, so equal counts draw as
a portrait maze. 33×23 characters is 264 by 368 pixels against 33×33's 264 by
528.

It came down twice, and the second time was **clipping rather than taste**.
Whatever a pane cannot fit is shown as a window that pans, which is right at a
small grid and reads as *the bottom is cut off* at a large one — the two are the
same code and only one of them is what a player expects. The block a map wants is
`2 × HEIGHT + 3` rows, so that is the number to check against a pane before
reaching for the constant again.

#### A wall is a square, so a step is one character

The maze was 7×7 *cells* with the walls **between** them. That has to draw
`2w+1` characters across — a cell, a wall line, a cell — so one step moved the
reading **two characters**, and it was reported the way it looked: *"I still seem
to be moving two spaces at a time."*

The reading was exactly right, and there is no fix at the drawing end. Dropping
to one character per cell loses the walls entirely: two corridors running side by
side with a wall between them would merge into a block, which is worse than the
complaint. The geometry had to change instead. **A wall is now a square of its
own**, the corridor between two cells is somewhere you stand, and the picture
*is* the grid — 15×15 squares, one character each.

Three things fell out of it, and two are improvements.

**The painter got simpler.** There is no odd/even split any more — no `interior`,
no `segment`, no `corner`, no lesser-of-two-marks rule for a corridor. `cell()`
is an index and a match.

**The meter counts floor, not squares.** Wall is most of the grid and none of the
walk, so counting the whole grid would peg the bar near a third before the
reading had gone anywhere.

**And a solver takes about twice as long** — 1233 ticks at worst across twelve
seeds, against 677 before — because every cell-to-cell move is now two steps.
That is the price of the picture reading correctly, and it is paid by bound
spells, which run unattended, rather than by a player, who walks at the speed of
their own keyboard. If it ever needs to come back down, the lever is `WIDTH`.

#### One generic fragment, and a recipe that can want four of it

The maze yielded one of `shard-of-dawn`, `-noon`, `-dusk`, `-night`, drawn
uniformly, and four distinct ones made a scroll. Two things were wrong with it,
and the second is the one that matters.

**Nothing in the game ever said what a shard was.** No prose, no `recall` topic
— `recall shard-of-dawn` offered *archive*, *brewing*, *clarified-draught*,
*clarity* instead. The four names were invented to fill an array, not decided.
§19 already has the rule that covers this: *"names are not prose… the moment a
fragment needs deciphered text, that text belongs in Phase 1's content file, and
needing it is the signal Phase 1 has been imported early."* A player asking what
one is *is* that signal.

**And collecting a set was attrition with no decision in it.** Four
interchangeable uniform draws is the coupon-collector problem: 4·(1 + ½ + ⅓ + ¼)
≈ **8.3 solves** for one scroll. You could not aim for the one you lacked, and a
maze whose shard you already held was worth exactly as much as one you did not.
That is §10's objection to the *old* `research` — a duration with no decision
content — reappearing one level up, in the collection loop instead of the
command, which is the harder place to see it.

So: one `fragment`, four of it, one generic `spell-scroll`, and no roll at all.
Specific fragments for specific spells is the intended shape and will want
distinct names again — at which point they will also want a reason to prefer one
maze over another, or the attrition comes back with better names on it.

**The cost was `Recipe::count`, and the alternative was worse.** What an
instrument holds is a node per *name* carrying a stock count, so
`inputs = ["fragment", "fragment", "fragment", "fragment"]` reads like it should
work and cannot — four fragments are one entry. The other candidate, expanding a
held stack into one name per unit, breaks something already shipping: the mortar
holding **two** sage against a one-sage recipe reads `charged` and fires, which
is exactly what *"charged a unit at a time, so a run spends a unit"* means. So
the recipe says how many it wants, the match asks for **at least** that many, and
`transmute` spends that many rather than a literal one.

It also collapsed three hand-built copies of *what is in this instrument* into
`tower::holdings` — all three had been dropping the stock count, which is
invisible right up until a recipe wants more than one of something.

#### What the review of the archive found

A `high` review of the whole arc found twelve things. Four are worth recording
because each is a rule already written down being broken somewhere new.

**The readings outlived their maze.** `refresh` ran *before* the solved `Maze`
was removed, so the four ways kept the solved position's readings for ever:
`survey north` answered `passage` with the stacks closed. The cost lands on
exactly the thing the archive is for — a bound solver read them, fired its
`follow` tier every lap and was told *"research first"* for the rest of its
`repeat`. `pipeline::stop` had the order right all along.

**Every one of `follow`'s records was filed under `research`.** Both verbs shared
one `say`, which stamped `FieldName::Name` with `Verb::Research`, so `sift follow
orb.log` returned the echo of the typed line and *not* what happened — for the
archive's most-used word. Rule 4 makes the record the source and every view a
reading of it; a record filed under the wrong verb is that source lying, and the
verb is passed in now.

**Solving a maze rolled from the laboratory's stream.** The shard draw used
`RngStream::Yield`, so walking the stacks changed a player's subsequent brew
yields — the cross-subsystem coupling per-stream RNG exists to prevent, in the
same change that added `RngStream::Archive` and then did not use it here.

**`State::Gathering` is new, and the panel needed a fifth word.** The lectern's
only recipe is an exact match on four distinct shards, so one, two or three of
them matched nothing and fell through to `Fouled` — the panel telling a player
mid-collection that their instrument *will not start*, which is the confusion
that column was built to remove. `Charged` would have been the opposite lie: it
means wield it and it runs. So `Recipes::gathering` asks whether what is held is
a proper sub-multiset of some recipe's inputs, and the flask holding one of two
ingredients gets the same correction for free.

Two smaller ones with the same shape. `stop lectern` abandoned the maze and
`return`ed, leaving a scroll assembly running — the lectern is the first
instrument that can be doing two things at once. And `wander`'s Escape handler
`continue`d rather than breaking, so arrows later in the *same* keyboard batch
still walked the reading after the player had left the mode.

**And three doc blocks had been split by insertion**, each leaving the function
below it undocumented and its own text attached to the wrong thing — one of them
claiming behaviour the body contradicted. §19 already records this exact defect
once, for `dump.rs`'s `woven`/`opened` pair. Inserting a documented item directly
above another one is the shape that causes it, and it is worth checking for by
eye every time.

**And the guard's own prediction has come true.** `type_into_line`'s comment said
the boolean's ceiling was five and that a single `Focus` owner was worth building
before the fifth surface arrived. `wander` is the fourth and is the last one that
goes in as a term: a fifth refactors it first. The reason is that each term is a
place to *forget*, and forgetting one does not fail loudly — it types into an
invisible prompt while the player is looking at something else.


### `weave` — the Ley Line, Mastery, and a surface for progression

Concentration 1 shipped and arrives **on its own**: brew a clarity, read one
line, and nothing was ever chosen. `status` printed `experience 20` and
`concentration 1` and that was the whole of it — two numbers with nothing saying
what they are for, what is next, or what it costs. **The first thing to build was
the surface, not more upgrades.**

#### The two tracks, and what they replace

§11.5 named *"ley-line upgrades and grimoire rank"* as the two sources of
Concentration and **neither had any mechanical content anywhere in the
document**. They were inherited verbatim from the draft-7 economy session, when
the pool was still called Attention, and survived three amendments untouched.
The tree is where they get their first definition, and one of them does not
survive it.

| | |
|---|---|
| **The Ley Line** | The straight path. Predefined steps, and **passing one is the grant** — no choice, and no moment where a step is reachable but unheld. It is `[concentration].levels` with a name and a `grants` on each entry, so `concentration(u64)` is the same reading it always was |
| **Mastery** | The branching tree. A tier opens on a total and gives exactly **one** of its nodes |
| **`grimoire rank` is dropped** | `/grimoire` is already the directory holding the player's spells. A *rank* of the same name would make one word mean the book you write in and a number beside it — the collision §19 already refused when `grimoire` stopped being a verb |
| **No points, and experience is still never spent** | A tier opening costs nothing; taking one of its nodes closes it. That satisfies the ROADMAP's *"offered something you choose between"* without contradicting §11.5's *"accumulates and is never spent"*, and without adding a balance a player can spend and then regret |

#### `weave`, and the name that failed

**`ascend` was the obvious name and does not survive the scorer**: two edits from
`attend` in a six-letter word is **667**, over `MIN_SIMILARITY`, and
`no_two_canonical_names_fuzzy_match_each_other` refuses it. `weave` scores **200**
against `wield` and **400** against `write` — the only other `w` words in the
vocabulary — and `wea` is a free three-character prefix. Computed before the name
was chosen rather than discovered by a failing test, which is the practice §19
records for `wield`/`kindle`.

**No shell synonym, and `tree` in particular is refused.** In a game whose
premise is a filesystem that *is* your duties (§7), `tree` means *list this
directory*: a player who types it means `survey`, and it would resolve at 1000
and take over the screen. That is the call the table already makes for `less` and
`read` against `unfurl` — and no fuzzy test catches a collision of *meaning*, so
it has to be made by hand. `status` has no shell synonym either, so nothing is
owed.

**The twentieth tower-wide verb needs the argument `unfurl` made**, and it is the
opposite case landing in the same place: `unfurl` earned its seat by being the
only way to reach a surface that *already existed*, and this one has no surface
at all. A track nobody can look at is a track nobody is on. **20 is a number to
defend, not a budget to spend** — the test says so beside the count.

**And `may_issue` refuses it**, beside `scribe` and `unfurl`. It opens a whole
screen, which is that objection with more of the window behind it: `repeat 100 /
weave` is a soft-lock. `may_issue` is a `matches!`, so nothing catches this at
compile time and the test drives a real cast.

#### The tree is not an item; it is what content leaves behind

Settled after the screen was built and looked at: **the upgrade tree expands as
the game gains content, rather than landing as a piece of work of its own.** A
node is a row in `progression.toml` and a line in `prose.toml`, and the painter
reads whatever is there — so a domain that ships brings the tier that unlocks it,
a recipe brings its own step, and the curve fills in as there is something to put
on it.

That is a scheduling decision as much as a design one, and it is the honest way
round: a tree authored ahead of the content would be a set of promises about
things nobody has built, and the balance question it exists to answer cannot be
asked until there is something to balance. What was worth building early is the
**surface**, because progression with nowhere to look at it is progression the
player cannot act on.

One thing stays scheduled: **the first real node**, which turns `take` from a
refusal into a grant and brings the pieces v1 left out — the mutator, a
`Submission` variant, the queued effect on a tick boundary. It is the one place
the choose-between mechanic can be seen rather than tested.

#### And a roadmap rule, because this is the second time

**An item that cannot close does not belong in a numbered phase.** Sitting in
one it does not track work — it holds the phase open for ever, and a phase that
can never be finished has stopped being a plan and become a list.

Twice now. The settings item (sticky skip, persisted CRT-off, reduce-motion) sat
open in Phase 0.5 waiting for somewhere to persist a setting, which no Phase 0.5
item builds; it moved to Phase 11, beside the settings screen it depends on. The
upgrade tree then sat open in Phase 1 waiting for content, and had no such phase
to move to — because it is not waiting on one thing, it is waiting on all of
them.

So the ROADMAP gains a **Standing** section, and the rule that sorts into it:
when an item is deferred because a *later phase* builds what it needs, move it to
that phase; when it accretes instead — a little more of it true with every
content item, never all of it true — it goes to Standing. **Nothing in Standing
may block a phase**, which is the property the section exists to guarantee.

#### v1 is read-only, deliberately

Every Mastery node is authored as a marker and `take` refuses in voice. **That is
not a scope cut**: it is what keeps the irreversible-choice machinery — a cursor
identity at commit, a confirm, a `Submission` variant, the queued effect on a
tick boundary that `Sim::write_spell` already has — out of an item with nothing
to commit. It lands with the first real node, which is where it can be exercised.

What ships is the shape, visible from inside the game before anything is behind
it: a player who reaches 24 watches a tier open and learns a choice is coming.

#### The shape: a bar, then two chains running right

**Progression runs rightward, and the screen says so three times over** — the
experience bar fills right, the Ley Line runs right, and Mastery's tiers run
right. A tier's nodes stack **downward**, which is the other axis and the other
meaning: rightward is progress, downward is a choice. The Ley Line, having no
choices, is one node tall everywhere, and that *is* the difference between the
two tracks rather than a special case in the painter.

**The first version was a vertical list and was replaced for being one.** It drew
what you had as a set of rows, and a player reading it could not see that the
thing was a *track* at all — reported as *"the UI is confusing; I thought we were
doing a horizontal progress."* The list was chosen to fit 48 columns and it did
fit; it simply did not say what it was for.

**The bar is measured against a fixed hundred**, and the Ley Line is drawn
underneath it across the same cells — so **a step's position on the line is its
cost**, read against the same scale. A step at 16 stands a sixth of the way along
and the fill either has reached it or has not; the two rows are one picture, and
that is what the fixed scale buys.

It was briefly measured against the *next* threshold instead, which is worse in
the one way that matters: the bar emptied itself every time a step was passed —
at the exact instant the player had earned something, the thing meant to show
progress reset to nothing. A constant scale only ever grows. The hundred is
provisional and deliberately round; the curve does not reach it yet, and when it
does the number becomes derived rather than chosen.

**Both tracks are lines, not rows of glyphs, and both are placed at cost.**
Drawing a step as a separate mark said *"here are some things"*; drawing an
unbroken run with stations standing on it says *"here is a road, and these are
the places along it"*. Mastery is the same road with a fork in it: one trunk, a
branch into the first tier, and a line from each node to **its own** successor —
which is what makes it a tree rather than two rows of unrelated marks, because
what a tree draws is *reachability* and reachability is the lines.

Mastery's tiers were briefly at a fixed stride, five cells apart whatever they
cost. That put a tier at 24 and a tier at 40 side by side and said they were
adjacent, when the second is nearly twice the work. Position is the cheapest true
thing a track can say, and spending it on even spacing is spending it on nothing.

**The names live under the cursor, not on the nodes.** At 48 columns a sentence
cannot sit beside every node, and abbreviating them all would make the screen a
puzzle. A node is a glyph and a total; what it *is* goes in a **details panel**
in the bottom right, for the one thing you are aimed at. That is what lets the
picture fit and the words stay readable at the same time. The panel sits under
the tracks rather than beside them, because a track runs the full width of the
pane and anything alongside one would be sharing cells with the road.

**It says two things, and they are not the same thing.** *Unlocked* is whether
the tower has earned enough to reach a node; *active* is whether what it grants
is in effect. A Ley Line step is both at once — passing one *is* taking it — but
a Mastery node can be unlocked and idle because nobody has chosen it, or unlocked
and idle for ever because a sibling took the tier's one choice. `Standing::Locked`
draws the same for a total not yet reached and a tier already spent, deliberately
— neither can be had — so `unlocked` is a **field on the node rather than a
reading of the glyph**. One says *work more* and the other says *you chose
otherwise*, and a panel that could not tell them apart would send a player to
earn something they have already earned.

#### Three things the screen had to be told about the window

- **The session pane is ~48 columns, not 80.** `DEEP_FOCUS_FLOOR` is 100×28 and
  panes tile side by side above it, so **the 80×22 floor is the *widest*
  single-pane case**, not the narrowest. Everything is authored against 48 — and
  the list version was not, so a sentence ran into the state column and a row
  read *"hold a spell while you are **locked**"*, a phrase in no file.
  `screens.rs` draws it at 48 for this reason.
- **`●` is not in CP437.** The renderer skips what it cannot draw, so "taken"
  would have rendered as *nothing* — silently collapsing the one distinction §14
  says must not be carried by colour alone. `•` (0x07), `○` (0x09), `·` (0xFA)
  and `─` (0xC4) are in the table and were checked against it. Same class as the
  em-dash CLAUDE.md records.
- **A glyph is not a description.** `Painter::span` pushes its literal text into
  the speech stream, so a reader would hear "`○`" and be told nothing. The nodes
  are drawn with `Painter::glyphs`, which writes no speech, and each one
  `announce`s `{total}: {state}` as words — the division `Painter::meter` already
  makes, whose doc says a silent caller *owes* the listener an utterance.

#### The way in is a word, and the arrows wait for it

**`ley` and `mastery` go *into* a track**, place the cursor on its first node and
hand the arrows over; `<esc>` comes back. That is `edit` dropping into the
editor's buffer, and it is the model this screen was corrected to: the first
version let the arrows work immediately, and *"my first key press was being
ignored"* was the report. An arrow at the command line now does nothing on
purpose — a screen where the arrows are sometimes navigation and sometimes
nothing, depending on what you last typed, answers differently to the same key.

**The dropped keystroke was a real bug and a self-inflicted one.**
`input::chord_is_stale` means *the modifier is a ghost — accept this keystroke as
plain text*, and this screen read it as *drop this keystroke*. One swallowed key
every time the screen was opened after a pause, which is exactly when it is
opened. The prompt and the editor both have it the right way round; paraphrasing
a guard instead of copying it is what put it in backwards.

**Typing while aiming was a second dead end, caught by a test.** Browsing
swallowed text the way the editor's reading state does, so a player who aimed at
a node and typed `take` got nothing with nothing saying why. A printable
character now steps back to the command line and **keeps the aim**, which is the
flow the arrows exist for. Still safe, because typing *leaves* browsing and only
`Enter` in browsing commits.

**Every node is framed, and the aimed one is framed differently *and* brightly.**
A node standing on a line needs to read as a station rather than as a break in
it, so all of them draw as `[○]`; the aimed one swaps the pair for `«○»` and
draws its glyph Bright. Two carriers for one fact, and each is doing a job the
other cannot: brightness alone failed outright — an aimed `○` is already Bright
and identical to its sibling — and §14 forbids the difference being colour, so
the frame is what survives greyscale while the brightness is what the eye finds
first. The frame overwrites one cell of the run on each side, which is why every
line is drawn before any node.

**And it is an identity, never an index.** The world ticks behind the screen, so
a tier opening changes what is on it — an index would come to point at a
different node. It unplaces when its id is no longer drawn, which is the rule
extracted from `Editor::reading`: a view that survives its subject lies.

#### And the content file grew a third rule

`progression.toml` already refused an unsorted track and an `[earns]` key naming
no instrument. It now also refuses a `grants` nothing implements and a duplicate
node id — an id is what a taken node is stored as and what its sentence is keyed
by, so two entries sharing one make taking either take both.

**`deny_unknown_fields` was the one that mattered.** Both tracks are
`serde(default)` so a file may omit them, which meant the *old* `[concentration]
levels = [16]` parsed happily into a tower with **no curve at all** — every
threshold gone, no verb refusing, and the file correct on its face. Two tests in
that module were silently exercising an empty track before it was added. The
regression test that made this visible was written **before** the restructure and
run after it, which is the only ordering that could have caught it.

### Experience, Concentration 1, and `bind` — the game's turn, built

§11.5 calls the first Concentration level *"the moment the game becomes the game
it advertises"*. It could not be built without deciding what pays for it, and the
answer turned out to change three numbers and delete one invariant.

#### Experience supersedes fragments as the progression currency

§11.5 priced ~8 concentration steps at 3–5 **fragments** each. Fragments are
*found* — sieges, hidden directories, remote hosts — and a capability that opens
because you found a thing is a different game from the one pillar 3 promises,
which is that **teaching the orb to do your work is the progression**. So
progression runs on **experience**, earned by completing runs, and fragments keep
`archive/` research and nothing else.

| | |
|---|---|
| **It accumulates and is never spent** | A `u64` that only rises; a threshold passed stays passed. One value in a save, no balance to keep, nothing to spend and then regret |
| **Weighted by the instrument that did the work** | mortar **1**, balneum **2**, flask **4**, alembic **8** — binary, so each tier of tool is worth every use of the one below, and the next rung always beats grinding the last one forever |
| **The archive earns too** | `research` is worth 1. It is a duration action in one of the two opening domains; at zero, half the opening would be dead progression |
| **Only work that succeeded** | Not a scour, not a refusal, not a run that matched no recipe, and **not `debug_spawn`** — the one path that makes reagents without work, which is exactly why it must not pay |
| **Content gates are named, not built** | *"brew this to open that domain"* is a second axis over experience, not a replacement for it |

The ~300-fragment derivation and the unlock-cadence cross-check are amended with
it: 11 of the ~75 research events move off the fragment economy, leaving ~64 and
~250. **The drip did not thin, because it became two drips** — a player between
fragments is still earning.

#### 16, and why the anchor moved from 30–45 minutes to two

**16 is arrived at, not picked**: 1 + 2 + 1 + 4 + 8 is one clarity walked end to
end — grind, digest, grind, mix, distil. The player buys concentration 1 with
exactly the potion the tutorial teaches.

That is ~2 minutes, against §11.5's *"time to first bound script: 30–45 min"*, so
the anchor is amended rather than the number. Deriving the threshold from the
recipe and then deriving the clock from the threshold is the right direction: a
30-minute anchor met by inflating the price would be the tutorial potion no
longer buying anything, which is the one property worth keeping. Automation
arriving early is the game showing its hand, not skipping its first act — the
pacing lives in the rest of the curve, where a second slot costs many clarities.

#### Invariant 3 is struck

> ~~A script-executed action completes in strictly less time than the same action
> issued manually.~~

A bound spell runs at **exactly** manual speed. The invariant is deleted rather
than deferred because it was never the argument it looked like: §11.5's own
"Blocking, and why automation wins" already says *"not faster, not cheaper —
non-blocking"*, and the ~6× advantage inherited from the retired Attention pool is
an argument about **concurrency**, which a speed discount does not supply.

What automation sells is that it runs while you are elsewhere and while you sleep.
A speed upgrade is later work, and when it lands it is a thing you buy — which is
strictly better than a property scripts are born with, because it is another rung
on the curve rather than a constant.

#### The fractional Concentration charge is deferred

§9 and §11.5 both have a bound spell draw a whole slot and its in-flight actions
draw fractions. Only the whole slot is built. The fraction exists to price
**multiplexing**, and at `CAPACITY = 1` there is nothing to multiplex; charging a
fraction of a pool of one would be arithmetic with no decision in it. It returns
with multiplex capacity, which is the upgrade it was written for.

#### What `bind` buys, and the `invoke` change that made it buy anything

The obvious plan — narrow `invoke` to the room you are standing in — **does not
work**, and finding that out shaped the rest. `Running` fixes a spell's domain at
cast and nothing read the player's position afterwards, so `attend laboratory;
invoke brewing; attend archive` already ran unattended. Gating the *cast* would
have bought `bind` a walk back and nothing else.

So the change is at the other end: **an invocation ends when the player leaves
the domain it runs in**, said in voice — *"first_light.spell needed you there. it
stops."* A bound spell does not. That is a difference you can feel in one
keystroke, and it makes §19's own earlier reading of `invoke` — that it *"needs
you standing there"* — true rather than asserted.

It does not contradict `scene.rs`'s *"a spell is not a domain, it is the book you
carry"*: a spell stays nameable and castable from anywhere. What needs your
presence is the **running**.

| | |
|---|---|
| **A bound spell stands** | It is cast again when it runs off the end. Otherwise a slot at capacity 1 is held by a spell doing nothing, and *"walk away, come back to work done"* would be false for every spell not already wrapped in a `repeat` |
| **Standing is silent** | One `spell_begun` per lap is the noise cut back from the editor's saves, below. The `bind` the player typed says itself once; the laps say nothing. The work still reports itself |
| **`stop` releases before it un-runs** | Or `stand` puts the spell back on the next tick and the player watches nothing happen |
| **Over capacity is refused, naming what is held** | At concentration 1 this is §11.5's sharpest decision in the game, and it cannot be made by a player who has to go and look up what they are already holding |
| **The hole this leaves** | With a bound spell always running there is **no way to pause one without giving up the slot.** Named rather than solved |

**§8's "`bind` always succeeds" survives**, because it is about *content*: a
spell naming a locked capability or a vanished path is warned about and flagged,
never blocked. Capacity is not content. A refusal that says the orb cannot hold
another spell is the pool being full, which is the mechanic §8 describes two
bullets earlier.

#### Three places that lied by omission until this landed

- **`boot.rs` hardcoded `bound: 0`.** §8.1 makes the boot report the
  forgotten-automation surface; shipping `bind` without wiring the count would
  have made it lie in the one place it exists to tell the truth.
- **Its scaffold list is derived from `is_live`**, so `bind` would have appeared
  in the first thing a new player reads, at concentration 0, where it can only
  refuse — the dead end that list exists to prevent. Hence `is_gated`, which is
  the same filter one step further in as the one that already keeps a domain's
  own verbs off the list.
- **`execute/tests.rs` sampled `bind night_watch`**, and no such spell is in the
  tower. The verb had never been exercised against a real node.

#### `progression.toml` needed an ordering the other content files did not

Its keys are instrument names, so it is validated against **`recipes.toml`**
rather than against anything in Rust — the first cross-content check in the
project, and the reason it loads after recipes. `materials.toml` validates its
colours against an enum and needed no such ordering. It fails the load like the
others rather than falling back, for the same reason: a weight that silently
defaulted to zero would make a whole instrument's work worthless with nothing
saying so.

It sits in the **recipes' tier, not the materials'** — not hot-reloadable. A
weight decides what a run earns and a threshold gates a verb, so both reach
decisions and a mid-session swap would break replay from `(seed, submissions)`.

#### Eight defects the review found, and the one shape six of them share

Six of the eight are the same mistake: **a rule was written for the run the
player starts, and holding makes a run lap.** Every mechanism that reasoned
per-*cast* silently became per-*second*, and every one that reasoned about the
`Bound` *component* missed the things a bound spell sets going.

| Defect | Fix |
|---|---|
| **A held spell's nested `invoke` died the moment the player walked out.** The exemption asked whether the entity wears `Bound`, and only the parent does — a child spell has a `Running` and nothing else, so §8's own worked example (`night_watch` invoking `brew_clarity`) was killed in exactly the walk-away case `bind` exists to sell | `Running::unattended`, a flag set at cast rather than a component read at use. A binding is unattended, a standing recast is, and **anything either casts inherits it** — while a spell a *player* invoked stays attended, and so does everything it invokes, or the child would outlive the parent the player's own departure just ended. The ambient `Depth` resource became `Caller`, carrying both facts, because a second resource beside the first is two things that must be cleared together |
| **`Running::said` reset on every lap**, so the once-per-line rationing on a missing name went back to square one twice a second: a bound spell with one typo reported it **31 times in 60 ticks** | The rationing outlives the run, on `Bound::said` — `finish` copies it out at the end of a lap and `cast` reads it back at the start of the next. `Running` is precisely the thing that cannot carry it, because `finish` removes it between laps. Cleared where `Running::said` is cleared: when the *text* changes |
| **`spell_done` fired on every lap** — *"tending.spell is finished"*, contradicted a tick later, for as long as it is held, with the recast already silent so there was no beginning to match it | Silent when the spell is held. It has not finished; it is being held, and a binding ends when the player says `stop`, which says so in those words |
| **`bind` on a running invocation restarted it from line 1**, silently. §8 makes `invoke` the way to *test* a spell before committing a slot, so `invoke x` then `bind x` is the sequence the design recommends and the one that threw the test away | Already in flight means taken up where it stands. The run is the same run; what changed is who is watching it |
| **`Backspace` in the editor's new reading state edited the buffer.** `type_text` and `enter` both refuse — a reading is a view, not a second buffer — and `backspace` fell straight through, so a player who opened `interpret` to check the orb's reading and tapped it out of habit deleted a character from a spell they could not see change, and the settle clock wrote the damage out | It refuses too. The three states are now exhaustive rather than "not `Command`" |
| **The reading survived the edit it described.** `reading[i]` belongs to `lines[i]`, so inserting a line shifted every reading below it: danger marks painted on the wrong lines and `interpret` showed the wrong sentences until the next settle | `touched()` clears it. Half a second of *no* marks is a pane catching up; half a second of marks on the wrong lines is the pane lying, and the player is looking straight at it while they type |

Two more, neither about laps:

| Defect | Fix |
|---|---|
| **`compile::fix` stopped at the first unplaced name**, so `if the mortr is idle and the dispensary has sagg` reported `mortr` and said nothing about `sagg` — and the player fixed one typo, asked again, and was told about the next one. `watch::every` states the opposite rule from the runtime end: *"the culprit is never anonymous, not that one culprit is enough"* | Every name, with a typo outranking a missing place — the first leaves a question that cannot be *asked*, the second one that cannot be *answered*, and every offender of the winning kind is named |
| **`[concentration].levels` was assumed sorted and nothing checked it.** `concentration` counts with `take_while`, so `[30, 16]` would gate level 2 behind 30 *and* never award level 1 at 16 — a table that reads as authored and behaves as neither, with no verb refusing and nothing to look at | `check` refuses it, beside the instrument-name check. The sort is the meaning of the list, so an unsorted one is malformed rather than unusual |

**One thing the review flagged that was left alone.** `spell_waiting` also fires
once per lap — but it is *true* every lap, unlike `spell_done`, and a spell's
records go to the log rather than the transcript (`prompt.rs` draws what the
player did, not what their spells did). §3 wants the log complete. Six lines per
200 ticks in a surface read with `sift` is the log working.

#### Risks recorded rather than solved

- **`CAPACITY = 1`, so the chains do compete.** The weights were set on the
  reasoning that a player runs instruments in parallel, and only one production
  action may be in flight until multiplex capacity is bought — which reaches 4 at
  ~10 hours. Until then the fastest experience in the game is the **haste** chain
  at 0.367/tick against the flagship clarity's 0.170, and the 4× distillation
  time is what inverted that. The mechanism is unaffected; the numbers are
  content.
- **`meditate 3600` is a fast-forward over the curve** — one keystroke for an
  hour of standing production. Harmless at a threshold of 16; it has to be
  answered before a second one is chosen.
- **Endless base reagents make a standing grind loop free**, at 1 experience per
  ~10 ticks for ever. That is the idle loop working, and it is the other half of
  the same question.
- **`orbs-balance` is a ten-line stub** and is itself an unticked Phase 1 item,
  so "the harness will sweep this" is aspirational rather than a plan.

### The alembic's bubbles get out — overturning "at the surface, never above it"

Asked for directly: *"release bubbles into the air above the face of the rising
progress bar."* The alembic's picture already had bubbles **breaking** at the
surface, and marks *above* it had been refused twice, for two reasons recorded
in `bath.rs`. Both were checked before this was built, and they turn out to be
one reason and one mistake.

**The real one is the layout.** The panel has two: down the side, where each
instrument is a **column** and the bar rises; and across the top, where each is
a **row** and the bar runs rightward. In the row layout *above the face* becomes
*right of the fill* — and the athanor's row sits directly beneath the alembic's,
with its sparks and smoke occupying exactly that region past its own fill
(`fire::plume` draws both). Two adjacent rows, sparse marks past the fill on
both, meaning different things.

**So the bubbles are drawn where above is up, and nowhere else** — `Steep::upward`,
set by the two painter entry points so a caller cannot get it wrong. The picture
now differs by orientation, which the shared `cell` was built to prevent; the
exception is justified because *what is adjacent to it* differs by orientation,
which is the whole of what the objection was ever about. It is one flag wide.

**The second reason does not survive contact.** It was that CP437's only round
glyphs are the fire's sparks. True — `∙ ° ·` — but a bubble and a spark are told
apart by their **ramp**, not their glyph: these draw from the liquid family, in
the tincture's colour, beside a column of embers. The `Spark*` depictions make
the same argument in the other direction, existing as their own variants
*because* they share a ramp with the flame.

**The face itself stays clear**, which is the part of the old decision that was
right and is kept: the level is read off solid-against-blank, and a mark on the
boundary cell would put the picture and the value at odds on the one cell the
value comes from. `fire::plume` pins its own front the same way. So a bubble is
born one cell up, climbs two, four or six, and thins as it goes — `°` while it
still has something in it, `·` for the last of it.

**Each of those three lengths is `STAGE` cells, and `STAGE` is two.** It was
one, and the whole vocabulary then happened inside three cells of the face: the
picture read as a fizz at the surface rather than as something *leaving* it. A
bubble wants enough room to be seen going. Doubling the stage doubles the time a
bubble is alive and therefore roughly doubles how many are in the air at once —
that is a real consequence and it is the reason `ESCAPE_ODDS` is worth
re-looking at if the air ever reads as busy rather than as boiling.

#### The distillation is four times longer, and the bubbles are measured from the face

Two changes, asked for together, and they turn out to be one problem seen from
both ends: *"quadruple the time a distilling takes, and make sure the bubbles
rise faster than the brew itself."*

The alembic's `clarified-draught → clarity` is **56 ticks**, up from 14 — by a
wide margin the longest stage in the pipeline, which is right for the last thing
that happens to a brew and the one worth watching. At fourteen it was over
before the picture had said anything.

**The second half cannot be fixed with a number, and that is the finding.** The
level climbs `bar / ticks` cells a tick, and the side panel's bar is the *full
height of its pane* — so it grows with the window. At the shipped durations the
face outran the bubbles comfortably (a 27-cell bar over 14 ticks is nearly two
cells a tick against the bubbles' one in three), and quadrupling only narrows
that: at 56 ticks and 36 cells it is still ahead. What it looks like is bubbles
being **swallowed by the liquid they just left**.

So the air is measured from the **face** rather than from the bottom of the bar:
a bubble is *defined* as sitting so many cells above the surface, and that number
only grows. The liquid carries them as it rises and they climb away from it on
their own beat. Overtaking is not tuned out, it is unrepresentable —
`the_rising_face_can_never_overtake_a_bubble` holds the phase and moves the
level, which would differ if any of it were anchored in absolute space.

`recall clarity` reads **94 ticks** now, up from 52. The balance harness sweeps
durations (§11.5) and this is a first-pass number like the rest of them.

**Two numbers, both tuned by looking**, which is the only way to tune them:

- `ESCAPE_ODDS` was 5 and is 3. At five, a two-lane column showed a bubble about
  a third of the time and read as a stray artefact rather than an instrument
  working.
- Presence and lifetime are drawn from **divided**, not shifted, parts of one
  hash — `seed / ESCAPE_ODDS % CARRY`, exactly as `fire::spark` does. The first
  version shifted (`seed >> 8`) and `drift_noise` is eight bits wide, so every
  bubble had a lifetime of exactly one cell: a picture that blinked one row above
  the face and never rose. Visible in `screens` the moment it was drawn, and in
  no test that could have been written for it beforehand.

**See it:**

```bash
cargo run -p orbs-render --example screens   # ...eight rises, side by side
ORBS_BOOT=0 ORBS_GRID=80x40 ORBS_FIRE_PHASE=0.4 \
  ORBS_DUMP="attend laboratory; kindle charcoal; debug_spawn clarified-draught; \
    distil clarified-draught; meditate 6" cargo run -p orbs
```

### `debug_spawn` — a tester's door, in the builds a tester runs

Reaching a state worth testing costs forty ticks of grinding, and most of the
bugs in this log were found by reaching one. `debug_spawn <reagent> [count]`
puts reagents in the **dispensary** — the one place things go when they leave a
tool (§19 below) — from wherever the player is standing, because requiring the
walk first puts back part of the cost the tool exists to remove.

**It is not a `Verb`, and that is the whole design.** Every reason is about the
vocabulary it would otherwise join: §6.1's tutorial lists the verbs that *work*
and its most important metric is the dead-end rate, so a word present in some
builds would be counted, offered, and then absent from the one a player has; and
`Verb::ALL` is walked by the completion table, by `recall`, and by a test that
drives every verb through a real `Sim`. So it is matched **exactly**, before the
parser sees the line, in the shape `SpellWord` already uses and for the reason
that module gives — *"checked before the fuzzy matcher ever sees the line."* The
parser's vocabulary does not know it exists, and a mistyped `debug_spaw` is an
ordinary unresolved line rather than a tower that quietly changed.

Three properties it does **not** get to break:

- **It lands on a tick boundary**, through `Pending` like every other effect. A
  debug tool that mutated the world from inside an input call would produce
  state that `(seed, submissions)` cannot reproduce — the thing meant to help
  find bugs becoming a source of them.
- **It spawns known names only.** `Recipes::vocabulary` ∪ the fuels table, so
  `debug_spawn xyzzy` is refused rather than putting a node in the tower that no
  recipe, instrument or `survey` row knows what to do with. A tester who can
  reach a state the game cannot is a tester chasing a bug only their shortcut
  produces. Bare, it lists what there is.
- **It says what it did.** §3 forbids unlogged output and tooling is not exempt.

**A release build has no code for it** (`cfg(debug_assertions)`), but four lines
of prose ship anyway, because `prose.toml` is `include_str!`'d whole. That is
deliberate rather than an oversight: four dead lines cost less than an exemption
from rule 6, and the width, shouting and CP437 lints all read that file. What
matters is that nothing can act on them, which
`the_word_does_nothing_in_a_release_build` asserts — the one test in the file
that is `cfg(not(debug_assertions))`, because a door only ever tried from the
side it opens on is a door nobody has tested.

One consequence worth stating: a session that used it does not replay in a
release build, where the line resolves to nothing. Debug sessions replay in debug
builds, which is where they were recorded.

**See it:**

```bash
ORBS_BOOT=0 ORBS_GRID=100x30 \
  ORBS_DUMP="attend laboratory; debug_spawn ground-sage 3; survey dispensary" \
  cargo run -p orbs          # ...ground-sage = 3 on the shelf
cargo test --release -p orbs-sim --test debug_spawn   # ...and the door is shut
```

### The file is the player's, and `if` learned to say `and` — built

Two reports, one root. *"I really don't like that the game edits a file and
throws lines away. If a file has syntax errors, it should just report it, not
edit the file."* And, a moment earlier, that `or` did not exist.

They are the same problem. The parser read the first `is` in a question, took one
word after it and discarded the rest of the line; the rewriter then wrote what it
had understood into the spell. So:

| typed | file afterwards |
|---|---|
| `if the dispensary has ground-sage or the mortar has ground-sage` | `if dispensary has ground-sage` |
| `if the mortar is idle and the athanor is working` | `if mortar_and_pestle is idle` |

§19 already recorded three bugs of exactly that shape — `grind sage` truncated to
`grind`, `invoke brew_clarity` re-pointed at `first_light.spell`, `wait for the
mortar` rewritten as `scribe mortar` — each closed by adding another special case
to the rewriter. **A fourth was the argument for stopping.**

#### Canonicalisation moves from save to cast, and §8 is amended

`scribe::canonicalise` is gone: `write` stores `lines` exactly as given, spacing
and indentation included. The reading happens in `spell::compile`, which is now
the only route from text to a runnable `Program`.

Nothing was given up to do it. §8's requirement is that *"a script executes later,
in a different world state, where live-state disambiguation is unavailable"* —
and cast is a **better** moment than save for meeting it: the names are fixed
once, in the world the spell is about to run in, with the player standing there
having just typed `invoke`. Command lines were already re-read per execution
(`run_line`), so only the questions changed at all.

`Draft` and `Program` are separate types for this. `program::read` returns a
`Draft`, which nothing can run and nothing can store on `Running`; only `compile`
produces a `Program`. A visibility modifier would not have done it — both callers
are inside the crate — and the two doors this had to close (`invoke`, `scribe`'s
mid-flight reload) are outside the module the constructor lives in.

#### `not`, `and`, `or`, and `either … or …` as a bracket made of words

`not` binds tighter than `and`, which binds tighter than `or`. Where precedence
is not enough, `either … or …` groups:

| typed | means |
|---|---|
| `a and b or c` | `(a and b) or c` |
| `either a or b and c` | `(a or b) and c` |
| `a and either b or c` | `a and (b or c)` |

**The bracket words take operands, not whole sub-questions**, and that is
load-bearing rather than an implementation note. Written the obvious way —
`either` reading a full disjunction — the inner disjunction swallows the `and`
that follows it, `either a or b and c` means `a or (b and c)`, and the only
grouping the language has stops grouping. It was written the obvious way first.

`both … and …` is accepted and **changes no meaning**; `and` already binds
tighter. It is kept because refusing a word a player would reasonably write is
the dead end §6 forbids, and it earns its keep in the *writer*, where it is the
only way to stop a nested `All` flattening into its parent.

A **shared subject** expands: `the dispensary has sage and charcoal` is two
questions about one shelf, `the mortar is idle and empty` two about one mortar.
Only when the operand carries no `is`/`has` of its own — that one rule decides
between a second operand and a second clause, and both readings are tested.

**Everything must be read, or nothing is.** A question that does not consume
every token it was given returns `None`: the line is kept, marked in the editor,
and reported at cast. That rule is the whole fix, and without it the half-read
line simply moves from the writer into the reader, where it is *harder* to see
because nothing rewrites the file to reveal it.

#### The orb accepts an abbreviation, never a typo, and never a coin flip

`similarity("ground-salt", "ground-sage")` is 819, against the prompt's floor of
600. A question written while the salt happened to be absent would have compiled
into a question about the sage — the file saying one thing and the running spell
asking another, with nothing on screen to show it.

So a spell resolves at `SPELL_SIMILARITY = 850`, which is not an arbitrary
number: `fuzzy` scores a prefix of three characters or more at 850 **by
construction**, and a typo only reaches it as a single slip in a long word. The
line is the one `fuzzy` already draws between *"prefixes are intentional"* and
*"typos are accidental"*. At the prompt the player sees the echo; a spell resolves
with nobody watching, so it takes the stricter half.

A floor cannot help when two things are *equally* close — with both products on
the shelf, `ground` is a 931 prefix of each — so `Scene::candidates` keeps the
runner-up and a tie refuses. Command lines in spells already refuse on ambiguity;
this brings questions into line rather than inventing a policy.

**The margin was a tie in a margin's clothes**, and asking whether it should
descend by increments instead is what found it. `SPELL_MARGIN` was `1`, so
`diff < 1` fired only on an exact dead heat. Measured against the shipped
vocabulary that is indistinguishable from a real margin — every reading is a
landslide (`mortar` wins by 652) or a dead heat (`ground` by 0), with nothing in
between — so it would have gone on looking correct until one reagent was named
close to another, and then resolved a twenty-point lead in silence.

It is `50` now, which is the room `sag` leaves: that has to keep reaching `sage`
past `sage-husks`, and wins by 67. A **descending scan** would not have added
anything on top — stopping at the first level where anything matches is
`max(score) ≥ floor` computed slowly, and the levels it would descend through
*are* what the score already means: 1000 exact, ≥850 a prefix by construction,
below that a typo. Requiring uniqueness at each level is a margin again, and a
banded one is worse than a plain one, because 951-vs-949 and 999-vs-951 land on
opposite sides of a band edge in the wrong order.

**And the test written to justify the number found a bug instead.** §6.1 puts a
`Topic` in the scene beside every reagent so `recall ground-sage` reads the
manual — so that word is in the scene *twice*, at identical scores, and the tie
rule was comparing a name against a copy of itself. `ground-sag` was refused as
ambiguous with the thing it names. `clearly` now takes the best entry with a
**different leaf** as the runner-up. Two entries for one word are one candidate.

**A place and a thing fail differently.** A place that is not there makes the
question unanswerable and is named. A thing that is not there is an answer of no
— `if the dispensary has ground-sage` is the commonest question in the game and
it is asked *before* there is any. A word that is in neither the room nor
`Recipes::vocabulary` is a **typo**, and makes the question unreadable rather
than false: `has ground-slat` was otherwise indistinguishable from `has
ground-salt` on an empty shelf, for ever, in silence.

#### An unanswerable question runs neither half

`run.rs` had `answer.unwrap_or(false)`, so a question nobody could answer took
the `else` — for ever, which is *"exactly like the condition being inverted"*,
the words this log already used about the last bug here. Now it runs neither, and
names what it could not place **once per line per casting** rather than once per
evaluation: a bad name inside a `repeat` was a `Role::Danger` record every tick
for as long as the spell ran.

**Strict, not Kleene.** One missing place makes the whole question unanswerable
even where an `or` had already been satisfied. Three-valued logic was considered
and rejected: an instrument that has stopped existing is §8.1's substitution
surface, and a spell carrying on over it is the thing that must not pass quietly.

#### `interpret`, the third editor word

The file no longer shows what the orb heard, and a resolution that fails is
reported while one that succeeds *wrongly* had nowhere to show at all. `interpret`
is where that went: the buffer, line for line, as the orb reads it, on request.
It changes nothing and saves nothing.

Beside it, a line the orb cannot read is drawn in `Role::Danger` with the count on
the status row — **not** a gutter mark: `GUTTER` is five cells, its own doc says a
sixth would cost the 80-column floor, and its one marker cell already holds the
running-line `»`. Two marks in one cell needs a precedence rule that styling the
line does not.

#### What it cost, and what it is held to

~250 lines of rewriter deleted; six tests rewritten (all of them asserted the
rewriting) and 44 added across `tests/questions.rs` and `tests/spells.rs` — a
grammar table of every shape and every refusal, a round-trip property, a sweep
asserting **no name is ever dropped**, six laboratories driven into named states
with a truth table asked of each, and the file held byte-for-byte across five
saves.

Two of those found real bugs on their first run: a `has` operand swallowed the
following clause (`the dispensary has sage the mortar is idle` read as a shelf
holding something called *"sage mortar is idle"*), and an unknown *thing*
answered no rather than being called a typo.

**See it:**

```bash
# what goes in comes out
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe keep" \
ORBS_EDIT="edit\ngrind the sage\nif the mortar is idle and the dispensary has sage\nempty the mortar\nend\n<esc>\nquit" \
ORBS_THEN="peruse keep.spell" cargo run -p orbs

# ...and `interpret` is where you see what it heard
ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend laboratory; scribe check" \
ORBS_EDIT="edit\nmake a potion of clarity\nif the mortr is bare\nsurvey\nend\nxyzzy plugh\n<esc>\ninterpret" \
  cargo run -p orbs

# a spell's own records are in the log, not the pane (`prompt.rs`)
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe broken" \
ORBS_EDIT="edit\nrepeat 5\nif the mortr is idle\ngrind sage\nelse\nsurvey\nend\nend\n<esc>\nquit" \
ORBS_THEN="invoke broken; meditate 20; sift broken orb.log" cargo run -p orbs
```

### `if athanor is idle` was yes while it burned — fixed

Reported from a spell: *"I had `if athanor is idle` which would trigger even
when the athanor was burning charcoal, which is not how I would interpret the
`idle` keyword."* It is not how anyone would.

The cause is a decision one layer down that was right and had one consequence
nobody traced. The fire is a `Burning` component and pointedly **not**
`Working`, because the athanor consumes no Focus and *"nothing that counts the
production pool may see it"* (§10.1 — the athanor is infrastructure, not a
stage). `holds` asked `tower::busy`, which reads `Working` and `Triaging`. So
the fire was invisible to the only two words that ask about it, and **both were
wrong at once**: `is idle` said yes, `is working` said no, and the panel beside
them drew `at burning` throughout.

The two words now ask the **panel's own state** — `State::is_busy`, true for
`Working`, `Scouring` and `Burning`. That ties the spell language to the word on
screen: `at burning` in the pane and `athanor is working` in a spell are one
fact, and a state added later cannot be busy for one and idle for the other. A
`busy()` taught about fire would have been the second answer that can disagree,
and this is the third bug of that exact shape in the spell surface.

`Banked` is deliberately **not** busy — a damped fire is fuel put by, not work in
progress, and a spell waiting for the athanor to be free must not wait on one for
ever. `is empty` stays a question about *contents* rather than the panel's
`State::Empty`, which the athanor never reports: it is cold, or charged, or
alight, and reading that word off the panel would make `if athanor is empty`
false however bare it was.

**See it** — the spell damps the fire, which means it saw it:

```bash
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe tending" \
ORBS_EDIT="edit\nif the athanor is working\nstop the athanor\nelse\nkindle charcoal\nend\n<esc>\nquit" \
ORBS_THEN="kindle charcoal; invoke tending; meditate 4" cargo run -p orbs
#   Progress   laboratory: athanor banked      ← the `if` branch, not the `else`
```

### Saving a spell says nothing

`scribe_done` and `scribe_unclear` are gone, with no replacement. **The buffer
writes itself out after every pause in the typing** (`SETTLE`, 0.6 s), which was
decided below and is right — saving is part of the loop when a spell can be
edited while it runs. What was not traced is that a *sentence per save* is then a
sentence every second or two: reported as a screenshot of one editing session
that had stacked sixteen copies of `first_light.spell: N lines, written down`
and `N lines, 1 the orb could not read` behind the modal, in a transcript whose
whole job is that a player can read back what happened.

Rejected: reporting once when the editor closes. It is one line rather than
sixteen, and it is still an announcement that a save the player did not ask for
and cannot see fail has happened. **Nothing is lost by silence** — a line the orb
could not read is named *individually, with its line number*, by the runner when
the spell is cast (`spell_missing`, `spell_unreadable_if`, `spell_forbidden`),
which is more use than a tally, arrives where it can be acted on, and is said
once. §3 forbids unlogged *output*; a save produces none, and the file it wrote
is `peruse`-able, which is the log.

`spell_reloaded` survives, because *"the orb picked your change up mid-flight"*
is the one thing about a save that a player cannot otherwise see — and it is now
said **once per editing session** rather than once per write, reset by `scribe`.
That keeps it derived from the submission stream alone, so a replay says it in
the same places.

The `Written { text, understood }` pair went with the count that read it, and
`canonicalise` returns plain lines. Its four verbatim branches — a spell naming
a spell, an unreadable line, `attend`, a dropped reagent — differed only in the
flag, so they are one function (`heard`) with the four reasons kept as comments,
because each of them was paid for.

**See it** — one line for the `scribe`, and nothing for the save:

```bash
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe brewing" \
ORBS_EDIT="edit\nkindle charcoal\ngrind the sage\nempty mortar_and_pestle\n<esc>\nquit" \
ORBS_THEN="invoke brewing; meditate 40" cargo run -p orbs
```

### The flask_and_rod — two things becoming one

§10.1's fourth instrument picture, and the only one whose subject is a *pair*.

**The bar is three bands.** Two ingredients above, the mixture growing from the
floor, and both ingredients shrinking at the same rate as it climbs. The
conservation is the mortar's — the vessel is always full of material and what
changes is how much has combined — and the even shrinking is the part that
carries the meaning: one input consumed before the other is touched would read
as *this reagent, then that one*, which is not combining.

**The mixture's colour is the average of its two ingredients**, not a third
authored one. A colour sitting between its neighbours reads as a mixture of
them; a new colour reads as a *substitution*, which is the opposite of what the
instrument does. The average is taken of the two **bases**, so the mixed band
derives its own three-step ramp and roils in its own colour like any other
liquid.

That required the tint channel to carry a pair, and it could — because a tint
lives on the `Frame` per *region* rather than on the `Cell` per position, a
payload costs a few words a frame instead of a byte times 7,040. The decision
below to keep it off the `Cell` is what made this affordable a fortnight later,
which is the usual shape of that kind of decision paying off.

**§14 is satisfied by the one boundary that matters.** Three bands told apart by
colour alone would be alarming if the bands were the reading — they are not. The
reading is where the mixture *ends*, and that boundary is a glyph edge: `█`
below, `▓` above, a whole coverage step. The bands are a hint about contents,
like every other tint.

### A finished bath settles rather than freezing

The at-rest rule — *an instrument that is not working does not move* — was
written about instruments that have **not started**, and it stays absolute for
those. A bath that has *finished* is a different thing: it has just spent its
run over a lit athanor, it is still hot, and liquid that hot does not stop
moving because the timer did.

So `Motion` is three states rather than a `bool`. `Standing` is dead flat;
`Settling` runs at a third the rate with a third the bubbles and never reaches
the brightest step. The tempo is what keeps the rule intact — a bar moving that
slowly is visibly a thing calming rather than a thing running.

**It also earns its place informationally.** `Charged` and `Ready` are the two
at-rest states a player must tell apart, and the meter cannot help: the sim
reports no quantity for either. Before this they were the same still picture at
different fills; now one is moving and one is not.

### Material tints — a hint that lives on the Frame

Every reagent and byproduct has an authored colour family, and an instrument's
bar draws in the colour of what is inside it. Sage grinds pale green; leave the
husks and the same bar turns brown.

**It is a hint over `survey`, not a carrier.** §14 forbids colour being the
*sole* carrier of meaning, and three things keep this on the right side of that:
`survey <instrument>` names the contents outright on every frontend, the panel
names the instrument and its state, and both reach the linear stream. A player
who cannot see the colour loses a convenience.

**It lives on the `Frame`, and the reason is arithmetic.** The obvious home is a
byte on `Style` — and a tint is a property of *what is in an instrument*, so
every cell of one bar carries the same value. Per-cell, that spends a byte on all
7,040 cells of a 160×44 grid to express something that varies across five of
them, and takes `Cell` from 8 bytes to 12 for the second time. It is a
`(Rect, Tint)` side-table instead, cleared with `speech` and `magnified` every
frame.

`Frame::magnified` is the precedent and the argument is the same one: it is
*informational*, so it belongs where both frontends can see it. Handing the Bevy
build a tint beside the Frame would have been the thing rule 2 exists to prevent
— *"the moment a frontend conveys something the Frame does not, the other
frontend is playing a worse game rather than wearing a different skin."*

| | |
|---|---|
| Eight families, closed | `Tint` is fixed in `orbs-render` and the TOML selects one *by name*, exactly as `Role` is a name the frontend resolves. The crate still holds no concrete colour |
| An unknown name fails the load | Not a fallback. An untinted material draws in the base hue too, so a silent one would make a typo indistinguishable from an omission — the defect `Recipe::heat` and `craft_of` have each paid for once |
| One base per tint, ramp derived | Eight numbers to tune rather than twenty-four, and monotonic by construction. The dim step began at 0.66× and was raised to 0.74× because brown fell under the 3.0 contrast floor |
| Hot-reloadable in principle | Unlike recipes and spells, a tint reaches no decision — no verb branches on it — so swapping the file mid-session cannot break replay from `(seed, submissions)` |

**Two things a tint may never paint over.** An accent, because §4 reserves the
triad strictly for meaning — a fouled instrument's label stays red however brown
its husks are. And the fire, because it burns one orange ramp on every tube by
the decision below; fuel is a tinted material like any other and the flame simply
does not consult it. A tinted hearth would put the four-ramp problem back in a
new costume.

**The failure mode is total and silent, so it has its own See-it.** A tint
changes no glyph, so a dump of the frame is identical with it and without — and a
colour reported by the sim that never reaches a cell draws in the base hue, which
is exactly what an untinted material looks like. `ORBS_DUMP` therefore prints the
tinted *regions* beneath the linear stream. That is not decoration: the first
version of this shipped a derived `Default` on `Materials`, so `init_resource`
installed an **empty table** and every material lost its colour while the file on
disk was perfectly correct. The cross-crate test in `shell::panel` caught it;
nothing within any single crate could have.

### The balneum mariae — a level, not a progress bar

§10.1's third instrument picture, and the one that had to solve a problem the
first two did not.

**The bar is the liquid in the vessel.** A gentle digestion draws something out
of a reagent and into a liquid, so the level starts shallow, rises as the
extraction proceeds, and stands full until the tincture is taken.

The alternative — a bar reading *how far along* — fails on the state that matters
most. The sim reports no meter for a charged instrument, so `stand_in` hands it
`0/1`, and a progress bar draws **nothing**: a loaded bath and an empty one look
identical. That is the invisible-state defect this panel was built to remove,
arrived at from a new direction, and it is the second time the answer has been
the same one. The mortar reached it first: *the bar is what is in the bowl*, not
how much has been done to it. **An instrument's bar should describe its contents,
not its progress** — progress is what the contents' state implies.

**All of the motion is in the colour, and that is the whole design.** The glyph
is `█` from the floor of the vessel to its surface at every fill and every phase.
Three things follow, all of them wanted:

| | |
|---|---|
| The value survives greyscale trivially | Solid against blank is the strongest join the alphabet has — the same one the fire's flame front is pinned to. §14 needs no argument here, only the observation |
| Nothing reads as a bubble | Considered and rejected: a mark in the water is a mark *of* something, and a vessel holds one substance. `∙°·` are the fire's sparks besides |
| It is legible at one cell | The panel's horizontal layout gives each instrument a single row, and a picture whose motion is textural needs height. This one does not |

**Its tempo is the slowest on the panel**, at four shared-clock ticks per beat.
That is the signature a *gentle* heat should have — ROADMAP asks that what is
running be legible without reading a word — and it is simultaneously a
photosensitivity argument the bath does not have to make: 0.75 flashes a second
against a floor of 3, where the fire is inside the band by recorded exemption.

**The roil takes the staggered clock, not the shared one.** `shared_tick` exists
for pictures that *travel*; a shimmer that stays put sampled off it turns every
cell of the vessel over on the same instant, which is whole-field modulation —
the hazard the rate cap alone does not cover. The first draft did exactly that
and would have failed `cells_do_not_all_turn_over_together`.

**No pour.** `Bench` carries one load timer driven by one edge, found by
`Craft::Grinding`, so handing it to the bath as well would mean charging the
mortar poured the bath. The bath does not need one: unlike the mortar it is
already a picture at rest, so a load is visible without an animation to announce
it.

### One fire, material tints, and where the accessibility promise went

Three decisions taken together, because the third is the price of the first two
and taking them separately would have hidden that.

**Fire is orange on every tube.** It shipped as four ramps — amber burned orange,
green burned green, violet violet, monochrome white — on the argument that §4's
premise is a single curved CRT whose base hue carries the picture, so an orange
fire on the green phosphor is a colour that tube cannot make. That argument is
about the *tube*; the thing being drawn is a **fire**, and a green fire does not
read as one, it reads as the meter having changed colour. `Depiction` exists
precisely because a picture is worth more than consistency with the surrounding
hue, and four ramps was that concession made and then taken back. One ramp also
has to clear the contrast floor against *four* backgrounds rather than sit inside
one theme's family, which is a stronger property and a smaller table: measured
5.5:1 at the coolest ember against the tightest background, on a floor of 3.0.

**Materials carry a tint.** A reagent or byproduct has an authored colour, and an
instrument's bar draws in it — sage grinds pale green, the husks it leaves are
pale brown, the bath's liquid takes the colour of what is dissolved in it. It is
a **convenience over `survey`**, which remains the authority on what is inside
anything, and it is not the sole carrier of anything: the panel names the
instrument and its state, and the linear stream carries both.

**Both put hue on the monochrome theme, and §4's promise went with it.** That
theme existed so *"a player with a colour vision deficiency loses nothing"*, and
`palette` tested that its base carried no hue at all. An exemption was considered
— tints resolving to base hue on that one theme — and rejected, because it makes
the accessible option also an aesthetic choice: a player who needs the
accommodation has to give up amber to get it.

So the guarantee moves to **Phase 11: colour-vision filters and a true greyscale
mode**, applied as a post-pass over the composited frame in the CRT shader, after
the phosphor and before the barrel. One place catches the fire, the tints and the
accent triad, rather than four palettes each being solved three more times. A
filter is orthogonal to the theme, which is what an accommodation should be.

**This is a promise deferred, not dropped, and the deferral has a cost.** Until
that item lands the game is *less* accessible than it was, and the roadmap item
says so at the top rather than reading like ordinary polish. What still holds in
the meantime: the accent triad remains separable without hue at all
(`palette::the_accent_triad_is_separable_without_hue`), every meter's value is
read off a glyph boundary rather than a colour, and `Depiction` reaches no
utterance. The tints are allowed to collapse under a filter; `danger`, `cost` and
`success` are not.

### Versioning — `0.<phase>.<step>` until release

The workspace version tracks [ROADMAP.md](ROADMAP.md) rather than a public API,
because there is no public API: every crate here is consumed only by this
workspace, so the semver contract has nothing to describe. What a reader wants
from the number before release is *where in the plan is this*, and the phase and
step say exactly that. It is also player-visible — `boot::screen` draws
`v{CARGO_PKG_VERSION}` on the POST card — so it doubles as the thing a tester
quotes in a report.

| Question | Decision |
|---|---|
| Form | `0.<phase>.<step>`, one workspace version inherited by all five crates |
| Phase 0.5 | The interlude gets **no minor of its own** — it is bookkeeping between 0 and 1, and `0.0.5` would collide with a Phase 0 step. Work done there versions under the phase it serves |
| A step is a **completed roadmap item**, not a commit | Commits are not a unit anyone reads; a checked box is. Corrections folded into an item (the `✅` entries under Phase 1) do not advance it — they are the item still being finished |
| Completing a phase | Bumps the **minor** and resets the patch to zero |
| After 1.0 | Ordinary semver, and the switch is **one-way**. Recorded here so the jump from `0.<phase>` to `1.<minor>` is never read as a thirteenth phase |

**Bumping it is part of finishing a step, not a release chore.** CLAUDE.md's
*Finishing a step* makes the three things one action: the box is ticked, the
version moves, and whatever was decided lands in this section. A step that did
only the first of those is a step whose evidence is a checkbox.

**The number is asserted by nothing**, deliberately. There is no test tying it to
`ROADMAP.md`'s checkboxes, and writing one would pin a judgement call — whether an
item counts as done is the question the See-it gate exists to ask, and it is
answered by a person looking at the running game. The convention is therefore a
habit with a written home rather than a check, which is the same standing the
See-it rule itself has.

### Instrument animation — corrections from review

Four decisions came out of an adversarial review of §10.1's animated bars. All
four were confirmed by measurement rather than by argument, and all four had a
green test suite sitting on top of them.

**A travelling pattern wraps on the distance it travels, not on the cycle.** The
plume and the mortar's debris are the only pictures in `orbs-render` that
*correlate* consecutive ticks — that is what makes them move rather than churn —
so they are the only ones the cycle boundary can tear. `shared_tick` runs
`143 → 0`, and a coordinate computed as `step - tick` jumped 143 cells there:
the whole plume re-randomised in one frame, once every twenty-four seconds.
Measured at 24 of 32 plume cells translating across the wrap against 31 of 32
elsewhere, and 26 of 40 for the debris.

`CYCLE_SECS` had asserted the opposite — *"there is no seam there: consecutive
ticks are uncorrelated hashes everywhere"* — which is true for an ordinary cell
and exactly backwards for a drift. `pulse::rising` and `pulse::falling` reduce
the coordinate modulo the pattern's own travel period, so `143` and `−1` are the
same coordinate and the wrap is one more ordinary step. **A picture that
translates must take its coordinate from those two and never subtract a tick by
hand.**

**`Depiction` is flat, and the reason is `Cell`'s size.** It arrived as
`Flame(Heat)` — a payload-carrying enum, which pushed `Style` to a second word
and `Cell` from 8 bytes to 12. A `Frame` holds one `Cell` per grid position and
is reset every frame, so that was +28 KiB and ~1.2 µs per frame at the worst-case
160×44, on *every screen in the game*, for a picture occupying about thirty cells
of one panel. Eleven fieldless variants with `flame()`/`spark()`/`smoke()`
constructors read the same at every call site and cost nothing. `Cell` is now
pinned at 8 bytes by test.

**A picture may never be able to vanish.** Three separate animations learned this
independently: the flare floors the flame at one cell so ignition is not a blank
frame, the pour floors the block at one for the same reason, and the cold
hearth's wisp did not — its noise clears the threshold about three times in eight
over three cells, so every draw in a narrow bar could miss at once. Reduce-motion
pins the phase at zero, which froze that blank for the session. A cold athanor
drawing nothing is indistinguishable from a row the panel forgot, which is the
invisible-state defect §10.1's panel exists to remove.

**The animated bars carry no accent, and `cold` is spoken.** Two halves of the
same rule. The picture bars take `Style::NORMAL` rather than the instrument's
accent — on the fire the accent is not even available, since `Style::depicted`
drops the picture on any accented cell — and nothing is lost, because the label
carries the accent in both layouts and `speak` carries the state into the linear
stream. That second clause was false for one state: `cold` was filtered out of
the utterance while the `Top` layout drew the word and the panel drew a wisp of
smoke for it. §19's rule is that a visual constraint must not become an
informational one, and this failed it in the direction nobody checks. Only
`empty` is filtered now.

### `unfurl` — a word for a key nobody could find

`PageUp` has scrolled the transcript since the transcript existed, and nothing
ever said so: the border advertises `PgDn newest` **only once you are already
scrolled back**, so the affordance announced itself exclusively to players who
had found it. In a game with no mouse and no menus that is no affordance at all,
and a long `survey` was effectively unreadable past the pane's height.

So the way in is a word, like everything else here. `unfurl` hands the transcript
the keyboard, pages back once so the screen visibly changes, and puts
`pgup/pgdn  esc out` in the border — which is the part the player keeps after
they stop needing the word. **Escape means what it means in the editor**: step
out of the mode you are in. It deliberately does not scroll back to the newest
output; someone who read back and pressed Escape wants to type, not to lose
their place.

#### Not `recollect`, and not `scroll`, and not `page`

Three names died to the naming pass, all before shipping, which is the whole
point of it:

- **`recollect`** collides with `recall` on `rec` — and
  `rec_is_pinned_as_a_prefix_before_anything_else_wants_it` names this exact
  scenario a word in advance: *"a future `recipe`, `record` or `recover` would
  build the `dec`-reaches-three-verbs defect, one word at a time and with
  nothing complaining."*
- **`scroll`** made `scr` reach `scribe` *and* `unfurl`, caught by
  `ambiguous_synonym_prefixes_are_known` — which is why that test pins a set
  rather than counting one.
- **bare `page`** fuzzy-matches `purge`. A collision between *read back* and
  *destroy what is in this* is not one to tolerate whichever way it resolves, so
  only the phrases `page up` and `page back` survive — and they name the key,
  which is better than the word they replaced.

`und` and `unf` part at the third character, which is the length the pass
governs.

#### Declining a keystroke means running

Three surfaces can own the keyboard now, and the prompt's guard used to be a run
condition. That is the wrong shape: **a system that does not run keeps its
message cursor**, so every key typed while the editor or the transcript held the
keyboard was still queued — and arrived at the prompt in a burst the moment it
ran again. Typing while reading and then pressing Escape dumped all of it into
the command line.

`type_into_line` runs whenever there are keys and decides for itself, clearing
the reader when it declines. The invariant that exactly one surface consumes a
keystroke now lives in one function rather than in a pair of predicates that had
to stay complements as modes were added.

**Found by a test written to check something else** — the assertion that mattered
was the second one, that the prompt gets the keyboard *back*.

### `survey` reads as a TOML table, tiled — built

`sage reagent` in one flat weight, with no indication of how many, repeated once
per row. Three problems in one line, and the middle one was the model's fault
rather than the view's — there were no counts to show until `Stock` existed.

```text
[reagent]
charcoal    = ∞  ground-sage = 1  husks       = 1  rock-salt   = ∞
sage        = ∞
```

- **`RecordKind::Section`** carries the kind that used to sit on every row, and
  stacks while the entries tile. Saying `reagent` once per line gained nothing
  and is what made a listing read as a wall rather than as a table. It speaks as
  a `Heading`, so §14 gives a listener the grouping the columns give everyone
  else.
- **The brackets are the view's.** The record holds the bare word, so `sift
  reagent` finds it and a reader hears a heading rather than punctuation;
  `[reagent]` is how this surface draws one.
- **Tiled, and never wrapped.** `per_row` is a whole number of strides and a
  stride is the widest entry in the run, so an entry either gets its own column
  or the run stacks. There is no arithmetic that can leave half a name at the
  end of a row, which is a property rather than a bound to tune.

#### The planner measures fields, not the rendered line

The `=` only lines up if every tile pads its name to the run's widest. Measuring
the joined text gives one width for the pair, packs them tight, and leaves the
eye nothing to run down.

**Getting that wrong broke a screen it was not aimed at.** Setting the stride
from the name alone while still drawing the whole line made the cold-launch verb
listing overlap itself — `attend plasurvey plaperuse filsift`. The planner tracks
both widths now and uses whichever the run is actually drawn from.

#### `=` binds an amount, and only an amount

The first version bound the second field whatever it was, and the verb listing
became `stop = place` — a grammar rewritten as an assignment, saying the two are
the same thing. How many of something there are is the one relation `=` reads
correctly, so `FieldName::Quantity` is the one it is used for; everything else
keeps the juxtaposition it had.

The separator is overdrawn dim after the span rather than drawn as a third span
of its own: the span already carried the whole tile into the linear stream, and a
second would put ` = ` in it as an utterance.

### A spell's output goes to the log, not the pane — built

A running spell emits exactly what the same commands typed by hand emit. That is
right, and it buried the transcript: a `repeat` loop pushes a move, a wait, a
yield and an empty every few ticks for as long as it runs, so the player's own
last line scrolled off in seconds.

**No new storage and no second path**, because a log is already a view over the
one record stream (§3, rule 4). `FieldName::Spell` names which spell caused a
record; the transcript draws the records without it, and `peruse laboratory.log`
reads the very ones it declined to draw. That also names the culprit, which §8.1
wants on its own account — a record saying which spell moved the sage is one
`sift` can select.

#### Stamped by the stream, not by the emit sites

`Records::attribute` is set once around a spell's turn, exactly as
`Records::register` is set once for the tonal register. A spell's output *is*
what the ordinary sites emit, so there is nothing at those sites to change — and
changing them all would mean every future site had to remember.

#### The half that was nearly missed

Hiding what the runner emits got most of the way there and left the line a loop
produces **most often**: `the mortar_and_pestle yields ground-sage`. An
instrument completes on its own schedule, ticks later, in `work::land` rather
than in the runner — so the credit has to be carried on the instrument
(`Bidden`) from the moment it is charged and re-applied when the run lands. It is
*taken* rather than read, so a credit cannot be spent on whatever starts that
instrument next.

`everything_a_spell_does_is_credited_to_it` sweeps the whole stream rather than
checking one record, which is what catches this class: the sites a spell reaches
are the ordinary ones, so the property has to hold for records nobody thought
about. Reverting the `Bidden` half reproduces five stray yields.

The player's own line stays: the `invoke` they typed, its echo, and the orb
answering that it has taken the spell up. Those are the command, not its output,
and hiding them would make casting a spell look like nothing happened.

### Stock has a count, and the base reagents are endless — built

The tower held **one** of each reagent, as one entity per name, with no quantity
anywhere. A `repeat` loop over `grind sage` therefore fired exactly once and then
reported an empty mortar for ever — correct behaviour with nothing behind it.
§11.5 wants the laboratory to always have something to do, and a laboratory whose
sage is spent after one grind has nothing.

**`Stock` is a component with two states**, `Endless` and `Counted(u32)`:

- **Endless is not "a very large number."** A count that started high would still
  tick down on the panel and still end, and the promise is that it does not.
- **A variant, not a name check.** `tower::Role` already records what happens
  otherwise — six sites branching on `name == ATHANOR` with nothing binding them
  together. Which stock is inexhaustible is decided once, where the tower is
  built, and §10's five further domains will each have their own.
- **Charcoal is endless too.** It is fuel rather than an ingredient, but a cold
  athanor with nothing to burn is the same stalled laboratory by another route.

**What is *made* is scarce**, and that is where the game is: the base reagents
are the floor, and everything derived from them costs the tower's time.

#### One unit per move

`move sage to mortar_and_pestle` takes one and leaves the rest; a run spends one
of each input. A count nobody can spend part of is a number on a screen — taking
one is what makes it the thing you manage.

#### Merging was already needed, and had nothing to show it

`give` pours onto an existing pile rather than standing a second node beside it.
Grinding twice used to leave **two** nodes both called `ground-sage` in the
dispensary — two `survey` rows under one name, and a name the parser then had to
choose between arbitrarily. That was true before counts existed; counts merely
made it visible.

#### The second copy is what made the first fix look like it worked

`move` and charging an instrument were two copies of the same three lines, and
they drifted the instant counts arrived: `move sage` took a unit while
`grind sage` re-parented the **endless pile itself** into the mortar, where the
run consumed it and `empty` swept it into the store. The tower's inexhaustible
sage was gone for good after one grind — and the `move` path, tested on its own,
looked entirely correct. `hand` is the one function now.

### The screenshot chord froze the prompt, then froze the editor — really fixed

`Cmd+Shift+Ctrl+4` hands the window to macOS's screenshot overlay mid-chord, so
the release for Cmd and Ctrl goes to the overlay. `ButtonInput` believes they are
down forever, every keystroke hits the chord guard, and the field is dead with
nothing on screen to say why.

**The first fix was `forget_held_keys` on `WindowFocused(false)`, and it did not
work.** Reported again a day later, on the editor. Reading the engine says why,
and it is structural rather than a slip:

- `WindowFocused(false)` is the **only** thing that reaches
  `check_keyboard_focus_lost`, which is the only thing that writes
  `KeyboardFocusLost`, which is the only thing that calls `release_all`. Every
  recovery path in Bevy 0.19 hangs off that one event — so our system was a
  second copy of a mechanism the engine already had, not an addition to it.
- **Bevy 0.19 drops winit's `ModifiersChanged` entirely.** That is the OS saying
  what is *actually* held, and it never reaches the app. Verified by grep against
  the pinned source: `bevy_winit` matches it nowhere.

So when a key-up is swallowed without a focus event, there is no mechanism
anywhere in the stack to notice, and no event to hang a third fix on.

#### The evidence is the keystroke itself

`chord_is_stale`: if the key arrived **with text**, the OS is not treating it as
a command, so the held chord is a ghost from a swallowed release. The guard
yields, the character lands, and the stale state is cleared — the field comes
back on the first key typed, with no focus event required.

Platform-independent by construction. A real chord produces no text on macOS,
and a control character on Windows and X11 — which both text fields already
filter, because a control byte in the buffer would occupy a cell and draw
nothing. `a_real_chord_is_still_not_text` pins that: `Cmd+Enter` carries
`text: Some("\r")` and must stay guarded, which is the bug the guard was built
for in the first place.

**Both fields, tested separately.** They share the guard, which is why they
shared the freeze — and a fix tested only on the prompt would have been half a
fix, exactly as the focus hook was. `forget_held_keys` stays: it is correct when
a focus event *does* arrive, and cheaper than waiting for a keystroke.

### An `if` was the one line the orb did not write down — fixed

**Reported as "the `if` block is working inversely"**, from a screenshot, and it
was not inverted. `if mortar is empty` ran the `else` every time because
`mortar` was compared **exactly** against `mortar_and_pestle`, found nothing, and
answered no for ever. `if mortar_and_pestle is empty` had always worked.

Two defects, and the second is the one that made the first survive.

#### Canonicalisation skipped a control word's tail

Every other line goes through §6's matcher at save — that is what turns
`grind the sage` into `grind sage`. A control word was kept verbatim, and so was
everything after it, including the place the question is about. `if` was
therefore the only line in the language where the player's own phrasing had to
match the tower's internal name character for character.

`scribe::written_condition` now resolves the names in a question the way a
command's argument is resolved, and `peruse` reads back
`if mortar_and_pestle is empty`. That is the same lesson the echo teaches at the
prompt, applied to the one line that was exempt from it. The word itself is
still verbatim — the collision `Resolution::InSpell` exists for is unchanged.

#### "No" and "there is nothing here by that name" were the same answer

`holds` returned `bool`. A place the tower does not have is §8's *Referent
missing*, not a false condition, and conflating them is what made this silent:
a spell could take the `else` on every pass for ever without saying one word.
§8's taxonomy is titled *"scripts always log and never halt"*, and this did
neither.

It returns `Option<bool>` now, and the runner says `spell_nowhere` naming the
place — every evaluation, like every other line-level failure. **That is
defence in depth rather than belt-and-braces**: canonicalisation fixes the names
a player writes today, and the runtime line catches a spell whose world has
changed since it was saved, which is exactly §8.1's substitution surface.

#### Why no test caught it

`if` was covered at the *parse* level only — `program.rs` and `spellword.rs` both
test `if the mortar is idle`, and neither ever resolved that name against a
world. A condition that parses and a condition that finds anything are different
claims. The three new tests in `tower::spell::tests` run the spell and assert on
**which branch executed**; asserting on `holds` directly would have agreed with
the bug.

### The orb was writing down a shorter command than it heard — fixed

Reported as *"`grind sage` is truncated to `grind` when I close and reopen the
editor."* It was, and the file really changed: `quit` saves, so every visit
re-canonicalised the spell against whatever happened to be on the shelf at that
moment.

§10.1's per-instrument verbs take their reagent **optionally** — bare `grind`
recharges a mortar that is already loaded, which is deliberate and is what makes
those verbs worth having. The consequence nobody traced: with the sage spent,
`grind sage` resolves to `grind` with no arguments, and the orb wrote `grind`
down as though that were what it heard. Two different commands, swapped in
silence, in a file the player had finished writing.

`scribe::dropped_argument` keeps the line as typed and flags it instead.

**Why it asks about reagents rather than about words.** Canonicalisation is
*supposed* to discard: `make a potion of clarity` becomes `recall clarity`, and
`look around` becomes `survey`. A sweep for "did any word vanish" flags both —
the game's two flagship plain-English phrasings — so that fix would have broken
the tutorial to save the editor.

The distinguishing fact is that a reagent's **name** is a fixed property of the
recipes while its **presence** is not. `sage` names something whether or not any
is on the shelf; `around` names nothing, ever. So the question is whether a
dropped word is in `Recipes::vocabulary`, and that answer does not move when the
laboratory does. The counterweight test asserts the three phrasings still
canonicalise and that none of them is flagged.

#### ...and the guard immediately exposed four lines that fell out of their block

Flagging `grind sage` made it visible that a kept-verbatim line loses its
indentation:

```text
repeat
    if mortar_and_pestle is empty
grind sage                          ← flagged, so un-indented
    else
```

**Four branches keep a line's words** — a spell naming another spell, an
unreadable line, `attend`, and now a dropped reagent — and every one wrote
`line.trim()`, which keeps the words and throws the whitespace away. Two of them
had been doing it since they were written; nothing had put a flagged line inside
a block before.

Structurally harmless, because blocks are delimited by `repeat`/`if`/`else`/`end`
and never by layout. Alarming to look at, which is worse than harmless in a file
whose whole job is being read back — and it contradicted what this module already
says about control words: *"the indentation is the orb's, not the player's."*
That was true of one branch and false of four. `verbatim` is now the one way to
write a kept line, and it takes the block depth.

### The running-line marker moved left, and went green

It replaced the gutter's trailing space, which put it *between* the number and
the code — so the digits shifted a column the instant an invocation reached the
line. It has its own column now, left of the number, and the number sits still.

The gutter is the same five cells: a marker column, three digits, a space.
Three still spells every line of any spell anyone will write, and a sixth column
would come out of the 80-cell floor, where columns are the scarce thing.

Green by `Role::Success` rather than by a colour — the theme's completion accent,
which is green in every theme where green reads and yellow in the one where it
would disappear into the base ramp.

### A prose line drew its own placeholder — fixed, and now guarded

`spell_gave_up` read `{name} waited too long on the {source}`, and `say_failure`
supplies `name`, `detail` and `count`. So a player watching a spell give up read
*"waited too long on the `{source}`, and moved on"*, literally.

`prose.toml`'s header calls this deliberate — *"a placeholder with no matching
field is left as written, visible on screen, so a typo is caught by looking
rather than by silently rendering an empty gap."* It worked exactly as designed:
found by looking, while answering a question about something else.

`no_line_a_spell_can_say_has_a_hole_in_it` is so looking does not have to. It
drives a spell through every failure the runner has — a wait that never lands, a
question about nowhere, a name that is not there, a forbidden verb — and asserts
no message in the **whole stream** contains a `{`. Swept rather than listed: a
list of keys is the thing that goes stale, and a new failure line with a new
placeholder is covered here the day it is written.

### The editor indents blocks as you type — built

Four spaces per level, eight two blocks deep, on every line of every spell was a
tax on writing one. `Enter` inside a block now opens the next line at that
block's depth, `end` and `else` step back out as the word completes, and
`Backspace` in the leading whitespace falls back a whole level.

**The rule is one function, in `orbs-sim`.** `parser::indent_around` says where a
line sits and where the next one starts; `canonicalise` folds it when the orb
writes the file and the editor folds it as you type. That is not tidiness — the
orb re-indents on save, so a buffer that indented differently would make **every
save look like it had moved your work**. `what_the_editor_indents_is_what_the_orb_
writes_down` checks the buffer against the shared function rather than against a
hand-written expectation, which would agree with itself and with neither.

Three judgement calls, and what decided each:

- **`end` moves when the word lands**, not when the line is left. A player types
  it inside the body, and a line that sat one level too deep until save would
  read as the save having moved it. Only lines that *are* control words re-indent
  — a command line jumping while you typed ordinary text would be the editor
  fighting you.
- **`Enter` mid-line indents nothing.** Auto-indent applies when the split leaves
  the line empty, which is "start the next one". Breaking a line mid-text moves
  the tail exactly as it was, because prepending an indent there — or trimming
  what the caret split — stops `Enter` and `Backspace` being each other's
  inverse. `enter_splits_the_line_and_backspace_joins_it_again` already pinned
  that and caught this being written the other way round.
- **`Backspace` outdenting is not a nicety**, it is the other half: `Enter`
  leaves the caret four columns in, and without it getting back out costs the
  four keypresses the indent just saved. It falls back to the previous *multiple*
  of a level, so a hand-spaced line lands on the grid rather than being pushed
  off it.

A blank line is now written out empty rather than as typed, because auto-indent
leaves the caret's level behind on a line nothing was put on — trailing spaces
that draw as nothing and would otherwise go into the file.

### A spell is edited while it runs — built

Four changes, and they are one mechanic. A spell can be opened, altered and
watched *mid-flight*; the orb reads along with you.

#### One line per tick, which turns spell length into a cost

`SCRIPT_BUDGET` was 4 and is now **1**. §8 introduced it as *"the execution
budget — the orb's attention"*, and at four it was only a runaway guard: the
difference between a tight spell and a sloppy one vanished inside a single tick,
so there was nothing for an efficient script to *be better at*.

At one, a spell costs a tick per step and **a shorter spell is a faster spell**.
Two lines that do what three did is a real advantage, and a `repeat` whose body
could have been tightened is paid for every turn. That is the lever §11.5 wants
and the old number did not give.

Everything counts as a step — a command, checking a `wait`, entering a `repeat`,
asking an `if`. Counting only *commands* would make block-heavy spells free,
which is precisely the wrong incentive. It also keeps the budget doing its
original job: entering a block spends a step, so an empty `repeat` cannot spin
inside one tick and hang the game.

#### Saving is debounced, and `save` is not a word any more

The buffer writes itself out a beat after the typing stops (0.6s). `save` and
`discard` are gone from the editor's vocabulary, which is now `edit` and `quit`.

**Because saving stopped being a thing you finish with.** A spell reloads live,
so the loop is: watch it go wrong, fix the line, watch the next pass take it. A
`save` word standing in the middle of that is a chore where the mechanic should
be.

Debounced rather than per-keystroke because a save mid-word would hand a running
invocation a half-typed line to reload from.

Three things follow, and each deleted code:

- **`quit` cannot refuse any more.** It refused on unsaved work, which made
  `discard` load-bearing — without it, unsaved work had no exit. There is no
  reachable unsaved state now, so both the refusal and its escape hatch are gone
  and `quit` simply flushes and closes.
- **`Complaint::Unsaved` and `Outcome::Close` were deleted**, being a refusal and
  an exit nothing could reach.
- **`q!` now means `quit`.** A vim user reaches for it, and "without saving" has
  stopped being a thing that can happen. `w`, `q`, `wq` and `x` stay as the
  unadvertised easter egg; `w` is the only way left to write *without* closing,
  and it is what `ORBS_EDIT` and `ORBS_DUMP` use, having no clock to pause on.

#### A save over a running spell takes hold now, not on the next casting

`Running.program` is re-derived and swapped in place, keeping the position as it
stands.

This is not a violation of §8's *"reloads queue to the next tick boundary, so a
file cannot change under a script mid-execution"* — it **is** that rule. A save
is queued through `Pending` like every other effect, so the swap happens between
steps and never inside one.

**The position is kept rather than remapped**, and nothing else is honest:
matching old lines to new ones is a diff, and a diff that guesses wrong moves a
running spell to a line the player did not point it at. Editing below the marker
behaves exactly as expected; editing above it shifts what runs next — the same
thing that happens when you edit a script somebody is reading aloud from.

It supersedes the *stop → edit → restart* loop recorded below, which is now one
option rather than the only one. `stop` is still there and still needed.

#### The marker, and `line_of` returning `Option`

The editor's gutter marks the line the orb is on with `»`, replacing the
gutter's trailing space rather than taking a column — a gutter that widened when
an invocation started would shift every line sideways under the player's caret.

§14 forbids a fact carried only visually, so the **title says it too**:
`Painter::border` pushes a title to the speech stream as a `Heading`, and
`editor_at_line` is authored prose like everything else.

`line_of` returned `0` for "run off the end", which the two log sites never see —
they report a failure on the step they are executing. The marker does: a spell
whose last line has run keeps `Running` until the tick tidies it up, and the
sentinel put the marker on a line numbered zero, which no file has. It returns
`Option<u64>` now.

**Found by looking, not by a test.** `ORBS_DUMP` also grew the ability to open
the editor from `ORBS_THEN` — the only ordering that can show a spell being
edited while it runs, since the invocation has to be cast first — and the first
version of that replayed `ORBS_EDIT` into the already-written buffer and reported
`9 lines, 1 the orb could not read` for a three-line spell.

### Domain-scoped spells, watchable events, and the first control structures — built

#### A spell belongs to a domain, and `attend` inside one is meaningless

`scribe` takes the domain from where you stand; the spell runs there wherever you
are. **The domain is data on the spell, not a directory.** `/grimoire/laboratory/`
was the obvious shape and it breaks the tower: a directory spawns as a place, so
that `laboratory` collides on the leaf with `/tower/laboratory`, `attend
laboratory` becomes a walk-order coin flip, and `every_place_leaf_is_unique`
fails — the test whose comment says it exists *"rather than the echo quietly
starting to lie."*

`scribe` at `/tower` refuses, in voice. The first refusal in the game about
**where you are** rather than what you named.

**This is a precondition for control flow, not a tidy-up.** Canonicalisation used
to walk a simulated position through a spell's own `attend` lines. A `repeat`
makes that walk execute a body once at authoring time and N times at run time;
an `if` makes it **undecidable** — you cannot know at save time which branch ran,
so you cannot know which room line 9 resolves against. §8 fixes canonicalisation
at authoring time, so the walk and control flow could never both exist.

#### Events are records, once the completion contract is honest

No second stream, no event bus. §3 already forbids unlogged output, so every
consequence is on the record stream — and the argument that settles it is §8.1's:
**a private bus would let a spell react to something the player cannot see or
audit**, and log poisoning would have nothing to bite on. Here, forging the event
and forging the evidence are the same act.

But the records could not answer *"has the mortar finished?"*. Eleven emit sites
put the instrument in whichever field was nearest — `Name` at two, `Path` at two,
`Source` at two more, where that field's own documentation forbids a place — and
`Name` itself meant a verb, an instrument, a product or a **list** of products
depending on who wrote the line. `siphon` announces `ground-sage`, so a spell
reading `Name` to find the instrument would have been asking a reagent.

**`FieldName::At` is the fix**: a completion says where it happened, always, and
the shared `say` helper now *requires* it as a separate argument so a new call
site cannot forget. A test drives a full brew and fails on any event without one;
it caught three sites on the first run.

#### `wait` before `if`, and `wait` had to be taken from `meditate`

Autonauts is the closest prior art — its audience is explicitly non-programmers
and its conditional primitive is *"repeat until hear X"*, a **listen rather than
a branch**. That sets the unlock order: a wait needs no condition vocabulary, no
comparisons, no truthiness, only a noun.

`wait` was `meditate`'s shell synonym. One word cannot be both, so it was
released to the spell vocabulary and `sleep` carries the sense at the prompt —
which it always did better. The tolerated-collision set is **one shorter** than
it was: `("wait", "write")` went with it.

#### Control words are spell-only, and the prompt answers for them

Not in §6's vocabulary and not fuzzy-matched: `wait` collided with `meditate`,
`repeat` reaches `revert` at 667 over a 600 threshold, and adding five words
fuzzily to a vocabulary whose naming pass exists to have no collisions is the
wrong trade.

**That left a hole §6 forbids, and it was not hypothetical.** Before this,
`wait for the mortar` at the prompt **opened the editor on a new empty
`mortar.spell`** — `for`/`the` are filler, `wait`'s `Count` slot cannot take
`mortar`, so the reading lost to `scribe <Name>`, which takes free text.
`repeat 3` resolved to `undo`. A player learning a word in the editor and trying
it at the prompt destroyed something.

`Resolution::InSpell` is `Elsewhere` one step further — *a real word, in the
wrong place, answered honestly*. And because canonicalisation runs **at save**,
the fix had to reach there too: without it the file would have held
`scribe mortar` permanently, with a cheerful *"written down"*.

#### `end` closes every block, and the orb indents its own fair copy

One closing word, chosen for **learnability** rather than diagnosability — the
earlier draft claimed the latter and it is false: counting detects imbalance but
cannot say which block, and cannot detect a misordered close at all. `fi`/`done`
were dropped as generic aliases, because `repeat … fi` would then parse, which is
worse than either scheme.

Indentation is cosmetic and **the orb re-derives it from block depth on write**.
It has to: `anchored` strips leading whitespace from every command line, so a
hand-indented spell could not round-trip a save. Re-indenting makes the file the
orb's fair copy, which is what §8 says a saved spell is.

#### A malformed spell runs

§8 leaves exactly one answer. It **cannot be refused at save** — *"`bind` always
succeeds"*, and a draft you cannot save is a dead end — and it **cannot halt at
cast**, because the taxonomy is titled *"scripts always log and never halt."* So
an unmatched `repeat` is closed at end of file, a stray `end` is dropped, each is
reported once naming the line, and the spell runs.

#### A spell can be called off, which it could not be

`stop` took a `Place`. A spell is a `Script`, so `stop night_watch` never
resolved to the thing it named — and **nothing but running out of program**
removes `Running`, which an unbounded `repeat` never does. A player who wrote one
had a spell working the laboratory for ever with no way to reach it: §6's dead
end, arrived at from a direction the parser could not see.

`stop` now takes a `Stoppable` — a place **or** a script — through the same
slot-kind-as-set machinery `peruse` uses. Spells are checked first, which is safe
rather than a tie-break: a `.spell` in the grimoire and a fixture in a room can
never share a name.

**Stopping the spell does not stop what it started.** `stop mortar_and_pestle` is
still how a run is cancelled; calling a spell off is walking away from it, and
the brew it began finishes exactly as it would if the line had been typed by
hand. Two things to stop, and `stop` reaching both must not conflate them.

The claim this corrects is one made in this very log: *"the budget bounds a tick,
so it wastes itself rather than hanging the game."* True of the frame. False of
the session, which is the part a player experiences.

#### `if` and `else`, and the arithmetic that makes a branch different from a loop

The fourth unlock, and last for the reason §8 gives: it is the only one needing a
vocabulary for **state** rather than for what just happened. Two shapes and no
operators — `if the dispensary has sage`, `if the mortar is idle`. No
comparisons, no booleans, no `and`, no nesting of conditions; each is a thing to
learn that the tower never taught.

> **Superseded twice, and both are recorded below rather than rewritten in.**
> The connectives work added `and`, `or`, `not`, `either`/`both` and nesting;
> *"A spell counts"* adds `has <n>`, which is the first comparison. What survives
> is the **posture** — that each addition is a thing to learn and has to earn the
> teaching — not the list. §19's convention is to mark a decision that moved, so
> a reader arriving at this paragraph is not told the language is smaller than it
> is.

**A question the orb cannot read answers no**, and says so. Guessing would be
worse than refusing here in a way it is not elsewhere: a condition the player did
not write would decide what their laboratory does while they are somewhere else.

The implementation detail worth recording is that **a branch is not a loop with
one turn**. An `if` costs *two* path elements going in — which half, then the
step within it — so walking back out has to pop two. Popping one leaves the path
pointing at the *other* half, and the spell runs both. That is why the runner's
stack records what kind of block each entry is rather than only a count: two
exits that look like arithmetic, and one piece of arithmetic is right for only
one of them.

`else` sits level with its `if` in the orb's fair copy. Indented as body it reads
as a step inside the branch it ends, which is the opposite of what it does.

#### `siphon` is retired, and the bench and the shelf become one place

**The verb had nothing left to do.** `reachable` searches idle instruments, so
`digest ground-sage` takes the mortar's output directly — the pipeline advances
with nothing drawn off first. `siphon` was step three of a four-command loop that
had quietly become two, and a docstring in `reachable` still asserted the
opposite: *"§10.1's loop makes `siphon` mandatory."* Written when the loop was
`move`/`wield`/`siphon`, never revisited when the per-instrument verbs collapsed
charge-and-start.

**And it was the only thing that could put a reagent on the laboratory floor.**
`move`'s destination resolves through `instrument`, which finds fixtures only. So
retiring it does not just remove a verb — it removes a *place*: the loose bench
where products landed. There is one place things go when they leave a tool, and
it is the dispensary.

What is left is a clean three-way split, each doing something the others cannot:

| | |
|---|---|
| the next stage's verb | takes what it needs, leaves the rest |
| `empty` | shelves **everything**, freeing the tool |
| `purge` | **destroys** what is inside |

`empty` inherited `collect`, `decant` and `pour`, because §6.1's rule is that a
released word does not stop resolving — it resolves to whatever it is nearest,
and the two nearest in this room are `purge` and `stop`, which is the pair a
laboratory can least afford to confuse. **`take` was deliberately not
inherited**: it sat one edit from `make` and was only safe while it belonged to a
verb with a different argument kind, so the `("make", "take")` collision is gone
rather than moved. Two entries have now left the tolerated-collision list and
none has joined it.

A trap worth recording, because a test caught it rather than a review: the work
tests' `run` helper was `stage / wait / siphon / purge`, and deleting the siphon
line left `purge` **destroying the product** the helper existed to keep. A verb
whose job is quietly two jobs does not decompose by deletion.

#### The `#id` annotation is withdrawn until something can be substituted

Spells were saved as `siphon mortar_and_pestle#5`, and the `#5` was wrong to be
there. **§8's requirement is about referents that can be destroyed and rebuilt**,
and its own example says so: `ward --upon north_gate#7f2a`. A siege tears a gate
down, the player puts it back, the rebuilt gate is a new entity — and the anchor
is the only thing that can tell, which is what makes §8.1's substitution sabotage
detectable rather than invisible.

Nothing in the tower is like that. `purge` on a place **scours** it rather than
despawning it — *"an instrument is safe by being a place"* — and the sim's four
despawns are spent fuel, an instrument's contents, and loose reagents. The
mortar's identity is fixed for the life of the world, so the annotation guarded
a substitution that cannot occur.

Worse, it was **inert**: the field it fed was `None` at every construction site
and read at none, and at run time the line was re-parsed with `#5` fuzzy-matched
away as noise — verified by writing `siphon mortar_and_pestle#999999` into a
spell and watching it resolve to the mortar anyway. So it was decoration that
*looked* load-bearing, in a file the player has to be able to read, and the
`Referent missing` row it was supposed to support does not work either.

It goes back with wards and gates in Phase 8, where there is something for it to
catch and something to test it against. Generalising it from one example about
one kind of place was the error.

#### Text stays canonical; the program is derived

`Held(Vec<String>)` remains the single source of truth and the program is rebuilt
at every cast. §8's hot-reload is line-anchored — *"only lines the player
actually changed are re-resolved"* — and §8.1's sabotage surface is *"a line
reordered"*. An enemy mutates the text; the program is whatever the text means.

**A new sabotage affordance falls out and §8.1's tell budget was not designed for
it:** moving an `end` changes far more meaning than moving a command line.

#### Three things the walker had to learn

- **A path, not a line number.** `[2, 1]` is the second step inside the third,
  and a `loops` stack holds each open `repeat`'s remaining count — a save with a
  position and no counts would resume every enclosing loop from its first turn.
- **Entering a block spends a budget step.** It looks wasteful and it is the
  guard: a `repeat` whose body spends nothing would otherwise be an unbounded
  loop inside one tick, and the game would stop.
- **An empty body is stepped *past*, not into.** Descending into one puts the
  path where `at` cannot resolve, which the runner reads as the end of the spell
   — so `repeat 2 / end / survey` ended before the survey rather than after it.

#### The cursor is a sequence number

`Records::sequence()` counts everything ever pushed and does not reset on
`clear`. An index into the stream is correct only while nothing truncates it, and
the stream grows without bound against a Phase 9a offline catch-up of ~29k steps
— so the day rotation arrives, every saved cursor would point at the wrong record
and spells would re-fire or skip with no test catching it.

A `wait` reads only what arrived after its cursor, which is what lets a spell
blocked for thirty ticks still see everything that happened in them, **and** what
stops a loop's second turn being satisfied instantly by its first turn's event.

### The script engine and the spell editor — built

Five decisions, in the order they were forced.

#### `grimoire` is a place, not a verb — the manual is `recall`

A grimoire is a wizard's book of **spells**, which is what `/grimoire` now holds.
Using the same word for the in-world manual made one word mean both the
reference you read and the book you write in.

**The word was released rather than re-pointed.** §6.1's *"a released word does
not stop resolving"* rule protects **shipped** vocabulary; nothing has shipped,
and unclaimed `grimoire` now reads as the directory it names — which it is.

The rename deleted a collision rather than mitigating one. `gri` was the
vocabulary's only shared three-character prefix, between `grimoire` and `grind`,
and it had needed a paragraph arguing the clash was survivable because the two
were words in the same room. That paragraph and its test exemption are both
gone. `recall` scores at most **500** against every other canonical and synonym
(threshold 600); the forward risk is the **prefix**, so `rec` is pinned by a test
before a future `recipe` or `repair` wants it.

#### `/grimoire` is a root domain, beside `/tower` and not inside it

The filesystem root is now a **nameless** node holding `/tower` and `/grimoire`.
Nameless because `path_of` collects a segment only where a `Name` is present, so
paths stay `/tower/laboratory` with no special case anywhere — verified by
building it before the tests that depend on it, which is also how three tests
that would have silently stopped covering half the tree were caught.

A sibling rather than a room, because **a spell is a book you carry**. §8 has the
player keeping their spellbook in vim; the fiction that survives that is
something on your person, not a shelf you walk to.

It is a root domain but **not a §9 activity domain**. §9's panes are per
*activity* — the seven you multiplex between — and writing is not one of them.
You do not run the grimoire concurrently with brewing; you go and write, and what
you wrote runs somewhere else. So it adds nothing to the pane count and is not
the third discovery the roadmap reserves for scrying.

**Spells are nameable from anywhere**, the same exemption places have. Without
it `invoke` would work only while standing in the grimoire — the one room with no
laboratory to run a spell in.

#### Saving the buffer is the authoring event, not `bind`

§8: *"`bind` resolves loose phrasing to canonical commands **at authoring time**
and stores the canonical form."* `bind` was the only authoring event when that
was written. `:w` is now that event, so `bind` and `invoke` both run text that is
already arcane. A refinement of §8, in the doc's own words.

Canonicalisation walks a **simulated position** through the spell, because
§10.1's per-instrument verbs only resolve where their instrument is: a spell
opening `attend laboratory` and then grinding is ordinary, and resolving every
line against wherever the *player* stands would refuse the second one.

**Places are anchored by ID; stock never is.** §8's own example is
`ward --upon north_gate#7f2a` — a place. §10.1 despawns an instrument's contents
and respawns the products with fresh IDs every stage, so `grind sage#31` would
produce a spell that works exactly once and reports `Referent missing` for ever
after — passing its first test on the way.

#### The editor is the frontend's, and the save is the sim's

The buffer lives beside the prompt's own line editor, in the Bevy crate. **A
keystroke reaches no decision**: it never enters `Submissions`, never enters
`ParseLog`, and cannot make two runs from one seed diverge. That is the same test
by which line editing was put there originally.

Three things confirmed it rather than merely allowing it: `tests/boundaries.rs`
forbids `orbs-sim` from naming a layout type *by substring*, so a buffer there
could not know its own pane height; `ORBS_DUMP` runs in the frontend and already
reaches into the line editor; and `orbs-tui` is ten lines with nothing to diverge
from.

**One entry per save.** `Submissions` became an enum — `Typed` and
`Wrote { name, lines }` — carrying the buffer as typed rather than as
canonicalised, so improving the canonicaliser cannot silently make an old session
replay into a different world.

**Two states, and it opens in the one that cannot lose your work.** *Command*
takes words — `edit`, `save`, `quit`, `discard` — and *editing* takes keystrokes
into the spell, with `Esc` coming back. Any unambiguous prefix will do, so `e`,
`s` and `q` work for the same reason `sur` means `survey` outside.

Two reasons this beat the always-insert-with-a-`:`-line it replaced. The small
one: `:` is punctuation you have to be told about, and a modal surface that
answers to `edit` and `quit` is the same game as the one around it, while one
answering to `:wq` is a different program wearing its clothes.

The large one: **opening in command state means the first keystroke cannot damage
anything.** A player who does not yet know what this screen is presses a key and
is told what the words are, rather than silently editing a spell they thought
they were reading.

`Esc` has **one meaning in both states** — step back toward the command line —
so it never leaves the editor and never discards. `quit` refuses on unsaved work,
which is what makes `discard` load-bearing rather than a convenience: without it
a dirty buffer would have no exit, and §6's dead end is the failure §15 weighs
above the raw resolution rate.

**`w`, `q`, `wq`, `x` and `q!` also work, and are never advertised.** An easter
egg for the hands that have typed `:wq` ten thousand times: finding it works is a
small gift, and not finding it costs nobody anything. It is a second table,
matched **exactly** while the vocabulary is matched by prefix — one merged list
would resolve `w` and `wq` by whichever was listed first, which is an ordering
nobody would think to check and a save-and-quit that silently only saved. `q!`
earns its place by being what a vim user reaches for at the exact moment `quit`
has just refused them.

*The `:` line shipped first and lasted one question.* It opened only with the
caret at column 0, so typing a line and pressing `:` to save put a colon in the
spell — and `Esc`'s advice to use `:w` could not be followed without pressing
Home. Neither the tests nor `ORBS_DUMP` could see it: the tests drove the command
line directly and the dump split the script itself, so both reached past the one
function that was wrong. The decision now lives on `Editor` and both go through
it.

#### `invoke` is a convenience; automation is still what Concentration buys

§19 above settles that **automation wins nothing at all** until the first
Concentration level. `invoke` runs at concentration 0, so it cannot be allowed to
win anything either — and it does not. It types for you, and typing was already
free: §5.0 makes issuing an action cost only the time the action takes, and §14
forbids any mechanic requiring fast typing.

So an invoked spell takes the **same** durations, occupies the **same**
production slot, and needs you standing there. What `bind` adds is **running
unattended**, and that is the whole of it: §8's speed advantage was struck from
§11.5's invariants when this was built (§19), so `bind` sells one thing rather
than two. It is still the game's turn, because *walk away and come back to work
done* is the sentence pillar 3 is made of — and because "needs you standing
there" was an assertion until `bind` gave it something to be true against.

**Per-action, not whole-spell, and this is forced rather than preferred.** If an
invoked spell held the production slot for its whole run, at `CAPACITY = 1` its
own `grind` would be refused by its own occupancy and it could never do anything.

#### What the runner had to learn that was not obvious

- **A script *waits*; it is not *refused*.** A player at a busy mortar should be
  told so. A script reaching the same line is not making a mistake — it is the
  next stage of a recipe arriving before the last one finished. Refusing it emits
  one complaint per tick for the whole duration of a run that is going perfectly.
- **The predicate asks "would this be refused", not "does this start work".**
  The first version asked the narrower question and `siphon` fell through it —
  `move`, `empty` and `purge` were the same hole. Found by **looking at the
  screen**, not by a test.
- **Blocked is not silent, and not forever.** Said once when the wait starts;
  after `PATIENCE` ticks it becomes a real failure and the spell moves on. §8's
  taxonomy is titled *"scripts always log and never halt"*, and an unbounded
  silent yield is a halt.
- **A spell has its own position.** `attend` writes `Cwd`, so without one a spell
  would teleport the player mid-brew.
- **Three verbs are hazards from a script**: `meditate` runs its whole count
  inside one `step()`, `scribe` would open the editor under the player's hands,
  and `undo` is command-anchored.
- **Finishing removes the component, never despawns the entity** — `Running` is
  worn by the spell node itself, so despawning deleted the spell from the
  grimoire the moment it completed. Caught by a test.

### Concentration replaces the Attention pool — designed, Phase 1

**Automated concurrency is counted in *spells you are holding*, not in actions
they have in flight, and it starts at zero.**

`concentration 0` is the tower worked entirely by hand. The first level — one of
the first upgrades in the game, inside the first hour — buys **one standing
spell**, and the player *concentrates* on it. That is the mechanic and it is also
the fiction: holding a spell is a thing a wizard does, and letting one go to hold
another is a thing a wizard feels.

**Why the unit changed.** The retired **Attention** pool counted concurrent
script *actions* and started at 3 — enough that a player's first script saturated
it, which §11.5 called the game's headline beat and the wrong place for a ceiling.
Counting scripts instead makes the number the one the player actually holds in
their head. *"I am concentrating on `night_watch`"* is a sentence about a spell;
*"I have three action-slots free"* is a sentence about a budget.

**Starting at 0 rather than 1 is the substantive half**, and it moves the game's
turn. Pillar 3 promises that teaching the orb to do your work *is* the
progression; a promise handed over at minute zero is a premise, not a
progression. So automation wins nothing at all until the first level is bought,
and that purchase is the moment the game becomes what it advertises.

| Superseded | Now |
|---|---|
| **Attention**, counted in concurrent script actions | **Concentration**, counted in bound scripts |
| Starts at **3** — the first script saturates it | Starts at **0** — no script runs at all |
| ~25 by the soft ending | **~8** by the soft ending |

**The ceiling is inherited, not re-invented.** §11.5 derived ~25 action-slots from
~6 min average durations giving ~250 actions/hour against a manual 40/hour at
Focus 4 — a 6× advantage, which is what pillar 3 promises. §8's `night_watch.spell`
issues three or more concurrent actions from one script, so ~25 ÷ ~3 ≈ **8
scripts** and the same 6× survives the change of unit.

**A whole slot *and* a fraction, which is not redundant.** A bound script holds
one slot while bound, idle or not — that is what makes "what is worth automating"
a portfolio decision, and at concentration 1 it is the sharpest decision in the
game, because binding a second spell means letting the first one go. Its in-flight
actions additionally hold *fractions* of a slot, because §11.5's multiplexing
counterweight prices **concurrency**: charging only whole scripts would let a
player buy depth 4 and pay what they paid at depth 1, which is precisely the
pane-proxy failure that amendment already fixed once, returning in a new unit.

**Three sections leaned on Attention being per-action** and were checked rather
than assumed: §8's call-depth limit (which exists *because* pool exhaustion fails
silently — still true), §8's "a script action and a manual action occupy different
resources", and §11.5's counterweight. The fractional charge is what keeps the
third one working; the other two only needed the name.

Earlier drafts overloaded the word "attention", which is why §9 fixed three
resource names in the first place. **The word is now free** and used only in its
ordinary English sense — §9's table names Focus, Concentration and Execution
budget, and the history sections below keep "Attention" where they record what a
past draft decided.

### Per-instrument verbs, scoped to their domain — built

§10.1's loop was four commands a stage — `move`, `wield`, `siphon`, `purge` — and
the two in the middle are the ones a player types most. Naming the **operation**
instead of the tool collapses them, and reads as the domain's own language: you
grind sage, you do not move sage into a mortar and then operate the mortar.

```
move sage to mortar_and_pestle          grind sage
wield mortar_and_pestle
```

Five verbs, one per instrument: `grind`, `digest`, `mix`, `distil`, and the
athanor's `kindle`. The athanor is the odd one — lighting a fire is not a *run*,
so it takes no Focus slot and produces nothing to transmute — but it charges and
starts exactly like the others, which is why it belongs with them rather than
under `wield`. `kindle charcoal` is `move charcoal to athanor` and `wield
athanor`; bare `kindle` relights what was banked, which is the last line of every
script loop. `Verb::transmutes()` excludes it explicitly, since `start` returns
at its `HeatSource` branch before a run is ever begun.

**`and` is filler, not a conjunction node.** `mix sage-tincture and ground-salt`
fills `Mix`'s two reagent slots positionally once `and` is dropped — which is
exactly the mechanism `move sage to mortar_and_pestle` has always used, `to` and
`from` being on the same list. A second way to express slot order would be a
second thing to keep in step with the first.

**The verbs are scoped to the domain whose tools they name.** §7 already says you
can only name what is where you are; an instrument's verb is the same claim said
the other way round, and `mix` in the archive means nothing because there is no
flask there. The instrument declares its own verb (`Operation`, a component in
the one place instruments are declared), the scene collects them from the
fixtures present, and the parser skips the rest.

That is not tidiness. It is what keeps the vocabulary safe to grow: §10 puts five
further domains in Phase 9a, each coining the verbs its tools need, and **none of
them can capture another's typo** because they are never candidates at the same
time. Without it, every domain added would widen the collision surface for every
other — the failure the Phase 0 naming pass exists to prevent, arriving by
accretion instead of all at once.

Three things had to be true for the scoping to be an improvement rather than a
trap, and only the first was obvious:

| | |
|---|---|
| **It must still answer.** A scoped-out verb resolving to nothing would have the orb say *"I do not know that word"* about a word the game taught in the room next door. `Resolution::Elsewhere` says `there is nothing here to mix with` |
| **It must beat a fuzzy rival.** `grind` is two edits from `find`, which `sift` claims — so scoping alone made `grind sage` in the archive offer `sift sage archive.log`. Scoping would have *created* the silent misreading it exists to prevent. An exactly-typed out-of-scope verb now outranks any fuzzy reading of an in-scope one |
| **Where you are is evidence.** `grind` shares a prefix with `grimoire` and sits at exactly `MIN_SIMILARITY` from `bind`. In the laboratory — the only room where `grind` is a word — the tool in front of you is the better guess, so an in-domain verb carries a `DOMAIN_BONUS` sized like `PHRASE_BONUS`: enough to settle a tie, never enough to overturn a real difference |

`grind` was kept over the collision-free `crush` on that last point. It costs two
pinned collisions and one shared abbreviation, all three confined to a single
room and all three disambiguated by argument kind as well as by place.

**`wield` and `move` both stay.** `wield` is the general form, it is what a script
writes when the instrument is the variable, and it is the only way to work a tool
no verb has been coined for. `move` is still how you put something down without
starting anything.

**`empty` is the counterpart of `purge`**, and the pair is §10.1's byproduct rule
made typeable: `purge` destroys what you did not mean to make, `empty` turns the
instrument out into the store and keeps it. Husks are the mortar's leavings *and*
the water bath's input, so a loop that can only clear by destroying is a loop that
never finds the second route to a draught — the thing the exit criterion is about.

It is **instant**, unlike `purge`'s four ticks in §9's triage slot. That is not an
oversight: the ticks are the price of *destroying*, and the loop's opening move
being non-free is what §11.5 puts `purge` in the Triage band for. Paying them to
put something on a shelf would be a toll rather than a cost, and `move husks to
dispensary` — which `empty` is the bulk form of — was always instant. What `empty`
buys is not speed but not having to name what is in there.

`clear` was refused as a synonym: one edit from `clean`, which `purge` claims, and
confusing *put this somewhere safe* with *destroy it* is the collision this domain
can least afford — the reasoning that kept `damp` out.

Two smaller things fell out. `Verb::transmutes()` replaced `verb == Wield` in
`finish` — a completed `grind` was releasing the slot, saying nothing, and leaving
the sage whole in the mortar. And `missing` gained an authored line: the
commonest refusal in the game drew as two bare field values, `mix sage`, which
names what was wanted and never says it was refused.

### The idiom pass, and the lints kept from it

A sweep of the whole workspace against `clippy::pedantic` and `clippy::nursery`,
plus the Bevy 0.19 checklist. The outcome that matters is not the fixes — it is
that **the useful lints are now in `Cargo.toml`**, so the standard is enforced
rather than re-derived by hand next time.

Three of the findings were real defects rather than style:

| Defect | Cost |
|---|---|
| `ends_with(".log")` decided which files carry §8.1's sabotage marker — a **byte** comparison, so a `FEED.LOG` added to the content file would spawn silently immune to poisoning, with no symptom until a siege failed to land a tell | `case_sensitive_file_extension_comparisons` |
| `get_or_insert` built the whole `Incomplete` — including a `Vec` allocation — once per vocabulary entry per keystroke, then discarded it because the slot was already filled | `or_fun_call` |
| Six helpers in `execute/` demanded `&mut World` to *read*, which makes them uncallable from anywhere holding a shared borrow for no reason their bodies can point at | `needless_pass_by_ref_mut` |

**Four lints were considered and refused**, which is the more useful half of the
record:

- **`redundant_pub_crate`** fights this project's own visibility rule, which asks
  for `pub(crate)` on shared items regardless of the enclosing module.
- **`unreadable_literal`** would rewrite `0xC0FFEE`, which is a hex *word* and is
  less readable with separators in it.
- **`match_same_arms`** would merge `Verb::signature`'s three `PLACE` arms. Each
  carries its own reasoning for why that verb takes a place — `Siphon`'s records
  that it was a `Vessel` until §10.1 moved the product into the instrument — and
  the comments are worth more than the three saved lines.
- **`suboptimal_flops`** suggests `mul_add`, which changes rounding in the render
  maths for no measured gain.

**Bevy 0.19 came out clean.** No bundles, no `Event` where `Message` is meant, no
`delta_seconds`, no `Parent`, `Single` used with `Option` everywhere exactly one
match is expected, every `Update` system guarded, and `default-features = false`
with a curated feature list. The one note worth leaving is on `Sim::working`,
which walks `iter_entities` because it takes `&self` and a query would need
`&mut`: 0.19 made resources into components on their own entities, so that walk
is now wider than it reads. It is still correct — nothing there carries
`Working` — but the next broad walk that filters on something a resource *could*
have will not be.

### Recipes read as instructions; Tab cycles; the transcript scrolls — built

Three things asked for after playing the laboratory, and one defect the first of
them uncovered.

**The grimoire was a table pretending to be a manual.** It printed `walk_back`'s
raw breadth-first walk: five unlabelled values a row, tiled across the pane,
**in reverse** — the goal's own step first, so a player reading top-down got the
recipe backwards. Four changes, in order of how much each was worth:

| Change | Why |
|---|---|
| **Doing order** | The walk is post-order over the recipe graph now, so a step's inputs are emitted before the step that consumes them and reading top-down is the order you type |
| **Stop at stock** | It expanded every route to everything, including three ways to make the `rock-salt` sitting in the dispensary. The manual is for what you cannot simply pick up |
| **One authored sentence a step** (rule 6) instead of five bare columns, with the byproduct, the instrument, the tick cost and a **heat marker** — forgetting to light the athanor is the commonest way a stage refuses |
| **Alternatives marked**, with any step only they need. Two rows producing `clarified-draught` were previously indistinguishable from two sequential steps |

The primary route is the content file's **first** entry for each output, which
makes `recipes.toml`'s order the designer's recommendation rather than an
accident.

**A line view clipped rather than wrapped.** Found while fitting the above: the
transcript drew one row per record and cut whatever ran past the pane, which is
silent data loss on the surface §14 calls the game's primary output. A pane gives
about 46 cells once its border and §10.1's instrument panel are out, and
`sage-tincture + ground-salt -> clarified-draught` is 47. Wrapping went into
`RecordView` rather than into the recipe text, so every long refusal benefits.

Two things that had to be got right with it: `height` and `draw` **wrap from the
same iterator at the same width**, or the transcript's tail search scrolls the
pane by a row a frame; and an `Input` record starts after the *whole prompt*
while everything else starts after the marker, so the width is per record rather
than one indent for all. A record still **speaks once**, on its first row — a
listener must not hear a different number of things depending on how wide the
window happens to be.

**Tab cycles.** First press extends to the longest common prefix and, when there
is nothing left to extend, lists **without touching the line** — bash's default,
and the least surprising answer to someone who pressed Tab to ask a question.
Every press after that puts the next candidate in, wrapping: readline's
`menu-complete`, and what makes the list an answer rather than a dead end the
player types their way out of. The list marks where the cycle has reached, and
says so aloud.

The cycle holds its own candidate list rather than recomputing per press, so a
tick landing mid-cycle cannot reorder what the player is walking; and it checks
the line still says what it last wrote before overwriting, so a forgotten
cancellation restarts the cycle instead of splicing a candidate into a word.

**The transcript scrolls**, on PageUp/PageDown. Up and Down stay history — a
shell where Up sometimes scrolls and sometimes recalls is one you cannot type in
without looking. Decisions worth recording:

- **In records, not rows.** The tail search already finds the smallest *record*
  skip that fits; rows would need a second, different measure of the same stream.
- **New output does not yank the view back.** The orb answers on its own clock,
  so a completion can land while the player reads history; jumping would lose
  their place, and the record is still in the stream when they return. The border
  says the view is held instead — in the *title*, because `Painter::border`
  announces one as a heading, which is what makes a reader hear it too.
- **Submitting returns to the bottom.** You acted; what you want is the result.
  This is the one place terminal convention is wrong here.
- `ORBS_SCROLL` exists for the same reason `ORBS_LINE` does: the feature is a
  keypress, a dump presses no keys, and §15 asks that it be reachable as text.

### The ten-angle review, and what it found — applied

Ten independent context-free reviews over the brewing + command-line work, each
given only the diff. They found **eleven live bugs**, four of which a player
would hit in the first session, plus a fidelity regression that had made the
default window two tiers finer than the tier table says. Every finding below was
verified against the running game before it was acted on; two were checked and
**rejected**, and are recorded as such.

The pattern worth keeping: the reviews were most valuable where a *test asserted
the bug*. Four of the eleven had green tests over them, and in three cases the
test had been written to pin the very behaviour that was broken.

**Live bugs, in the order a player meets them**

| Bug | Why nothing caught it |
|---|---|
| **`purge dispensary` made the tower unwinnable** — one command despawned `sage`, `rock-salt` and `charcoal`, and no recipe produces sage or charcoal, so the athanor could never be lit again. Against §7's *"destruction is a tool, not a trap"* and §11.5's *"not automating is never ruinous, only slower"*. The dispensary is now `Store + Protected` | Three tests used `purge dispensary` as their *example of a working purge*. It was the canonical demonstration |
| **A `move` could raid an instrument being scoured** — the lock covered `Working` and not `Triaging`, so a reagent could be carried into an instrument four ticks before a purge despawned everything in it. Reagent gone, nothing produced, Focus slot spent, not a word said. `busy()` now returns both | Nothing tested the interleaving. `Triaging` was added later than `Working` and the guards were never revisited |
| **`peruse laboratory.log` was permanently empty** — the filter compared `FieldName::Source` against `"laboratory"`, and nothing writes that: `transmute` puts the *instrument* there and `carry` wrote no `Source` at all. A domain log is now *what happened in that domain* — the room or anything standing in it | `an_empty_file_reads_as_empty_rather_than_as_success` passed **because** the log was always empty. The test asserted the bug |
| **The transcript dropped its newest records** — the tail binary search took `len - rows` as an upper bound on the assumption that a record costs one row. `RecordView` opens every `Input` after the first with a blank line, so the predicate was false at its own bound and the search converged on a skip that overflowed the pane. Bound is now `len` | The comment stated the false premise as a justification. The blank-row change landed later and nobody re-read it |
| **A `wield` matching no recipe said nothing at all** — `finish` `continue`s after `transmute`, so returning quietly made it the one path that ends a command with no record. Echo, then silence, forever | The path was believed unreachable, so no test drove it |
| **`move` did not clear `Product`** — the destination read `ready` on the panel before anything had been wielded, and `siphon` handed the freshly-delivered *input* straight back out | `siphon` clears the marker; nothing checked that `move` does |
| **`purge` worked at a distance** — from `/tower/archive`, `purge mortar_and_pestle` scoured the laboratory's mortar. The cwd lookup compared a *leaf* against a *full path*, so it never matched and every purge reached its target through the tower-wide fallback | The fallback looked deliberate. It was the only path that ever fired |
| **`damp` at exact burnout destroyed the fire's ash** — `Ash` is landed only by `burn`, which queries live `Burning` components, so removing `Burning` on the tick fuel hit zero orphaned the debt forever. Quenching one tick earlier produced ash; on the exact tick it vanished | An off-by-one in a branch nobody sampled |
| **`State::Fouled` had no construction site** — an instrument holding only husks drew as `charged`, the same word as one loaded and ready, and `speak()` filters `Charged` out of its utterance entirely, so a screen-reader user heard nothing. Precisely the confusion §10.1's panel exists to remove | `an_instrument_walks_through_its_states` asserted `State::Charged` there, with the comment *"the husks are still in there"* |
| **The Tab listing was wiped by Tab's own key-release** — `type_into_line` is gated on `on_message::<KeyboardInput>`, and winit sends one for the release too. The listing lived for the ~50 ms a finger was down | The test helper only ever writes `ButtonState::Pressed`. The release path had no coverage at all |
| **`work_busy` rendered "divineing"** — the prose template appended a literal `ing` to `Verb::canonical`. English inflection is not string concatenation; `Verb::participle` now spells it | The authored-line lints check the *template*, never the interpolated result |

**The fidelity regression.** `Fidelity::PREFERRED_GRID` was set to
`DEEP_FOCUS_FLOOR`, which made `DisplayMode::default_for(tier_one.grid(w))` a
**tautology** — `tier_one` aims at exactly that grid, so whenever it succeeds the
answer is `Deep` — and `deep()` then dropped a second tier. 1920×1080 opened at
240×67 with 8×16 pixel glyphs where 120×33 at 16×32 was intended. Aiming tier one
at the Deep floor already bought what the default was reaching for (two panes
hostable from the first frame, so `F4` works immediately); choosing `Deep` as well
spent the affordance twice. The default is now `Wide`.

**Layering, where a frontend had started deciding things**

- **`abbreviate` moved into the sim.** Rule 2 gives a frontend *how* a cell is
  drawn, not *what word appears in it*, and the Bevy build was deriving the
  panel's two-letter labels from English stopwords in its own source — a rule
  existing nowhere else, so `orbs-tui` would have had to reimplement it and the
  two builds could disagree about what `bm` means. Its collision test lived in the
  Bevy crate over a **hardcoded copy** of the instrument list, so renaming an
  instrument left it passing on stale names.
- **The panel's spoken domain is passed in**, not the literal `"laboratory"`.
  §10's Phase 9a adds a second instrumented room; a sighted player would have read
  `/tower/workshop` in the border while a reader heard "laboratory: forge burning".
- **`needs_heat` became `Recipe::heat`.** It was `matches!(instrument,
  "balneum_mariae" | "alembic")` in the executor while `recipes.toml` recorded the
  same fact *as a comment* — so a designer adding a heated instrument would have
  edited the file, read their own note, and shipped a recipe that runs cold.
- **The athanor and dispensary are marker components**, not six `name == ATHANOR`
  comparisons with nothing binding them together.
- **`window_too_small` moved to `prose.toml`** (rule 6 has no frontend exemption)
  and the prompt's `Prose` is reachable through `Sim::prose`.
- **Prose hot-reload no longer moves the parser's nouns.** Every `grimoire_` key
  was a `NounKind::Topic` read live, so renaming one mid-session changed what a
  phrase resolves to — while `set_prose` documents itself as replay-safe *because
  prose reaches no decision*. A noun is a decision; `tower::Topics` snapshots them
  at construction.

**Two findings checked and rejected.** A review proposed caching the
`QueryState`s that `burn` and `finish` build per tick. `cargo run --release -p
orbs-sim --example bench_steps` says rule 8's worst case — 29,000 `step()` calls —
is **121 ms at ~4 µs a step**, so those constructions are noise at this entity
count and the caching would be complexity bought for nothing. The example is kept
as the instrument that says so. A second review claimed `ORBS_DUMP` reported a
tier its grid could not reach; the arithmetic says 80×22 at tier 1 is exactly what
a 640×352 window produces. The *real* defect underneath was different — the dump
pinned a fixed fake window, which stopped being inert when the prompt learned to
spend a second row at a fine tier — and `Fidelity::for_grid` now inverts the real
path instead.

**Structure.** `execute.rs` was 987 lines holding three unrelated concerns with
only the sixteen-arm `match` touching all of them; it is now `execute/` with
`dispatch`, `pipeline`, `grimoire`, `navigate` and `files`. The three TOML loaders
became one `content::load`, `char_index` and the meter arithmetic moved to
`orbs-render` where both callers can share them, and `paint`'s ten positional
parameters became a `View`.

#### The magnified prompt grew when the screen got denser — reversed

`Fidelity::input_rows` spent a second row at `scale ≤ 2`, drawing the prompt at
double size so it kept its pixel height as the tier got finer. On a **1440p
window it did the opposite**: F4 drops scale 3 → 2, and the prompt went 48 px to
**64 px — larger — while every other glyph on screen halved.**

**The mismatch is arithmetic, not taste.** Tiers step by one scale (3 → 2 is ×⅔)
and row-doubling steps by ×2; those agree only on the 2 → 1 step. Doubling at
scale `s` stays within the tier above it exactly while `32s ≤ 16(s + 1)`, which
is `s ≤ 1`. The threshold is now derived from that rather than chosen.

**A constant height was never available.** The reachable heights are `16s` or
`32s` — `{16,32}`, `{32,64}`, `{48,96}`, `{64,128}` — which share no value, so
"the prompt keeps its size" was not a property that could hold. What holds
instead is **monotone**: 32, 32, 48, 64 across scales 1 to 4, never rising as the
tier gets finer, plus never falling below the transcript it sits under.

The cost is the complaint that set the threshold at 2 in the first place: at
scale 2 the prompt is now the transcript's own size, and someone found that
hard to pick out. Size was a blunt instrument for *distinguishability* and it
bought a prompt that grew when the screen got denser; if the prompt needs to
stand out, the tools are the caret, the dim `orbs:~$`, and the rule above it.

Two things fell out. The input line is no longer half-width at scale 2 — a
double-size glyph costs two columns as well as two rows, so **1080p gets its full
typing width back**. And the guarding test was the real failure: it asserted only
that the height landed in a 32–64 px *band*, which every tier satisfies, so the
jump *inside* that band passed for as long as it existed. A range nobody chose is
not a decision. It now asserts monotonicity across the table, the floor against
the body text, and that F4 cannot enlarge the prompt on any real window.

### The prompt becomes a command line — built

Caret editing, history, completion and an inline suggestion. Two independent
reviews and a survey of `rustyline`, `reedline`, GNU readline, fish and zsh; what
follows is what those changed, because most of it was load-bearing.

| Question | Decision |
|---|---|
| **Places echo as their leaf** | The three-argument `move` echo was **already clipped** — `move charcoal /tower/laboratory/dispensary /tower/laboratory` lost its destination at the 80×22 floor. §7 already says *"players say the place, not the path"*; the echo teaching one canonical form and the player saying the leaf were only ever reconcilable one way. **Not** a "shortest unique suffix": `score_against` matches a phrase against the full name or the leaf and nothing between, so `laboratory/alembic` would teach a form the parser rejects. Collisions are **forbidden** instead, by `every_place_leaf_is_unique` — the day a seventh domain wants a second `dispensary`, a test fails rather than the echo quietly starting to lie |
| **A component is looked for on the floor, then in the instruments, then in the dispensary** | The first draft put instruments first, reasoning that a product sits in the tool that made it. It does not: `siphon` moves it to `cwd`, and §10.1's loop makes `siphon` mandatory, so **every mid-pipeline `move` takes its subject off the laboratory floor**. The dispensary is stock and stock is the fallback |
| **A working instrument is never raided** | Found while checking the above. `carry`'s lock was on the *destination* only, so a `move` could pull a reagent out of a running instrument: the run then matched no recipe, returned early, spent the Focus slot and **said nothing**. Live bug, fixed on its own account |
| **`FieldName::Origin`, not `Source`** | `read_file` keys **domain logs** by `Source`, so a `move` carrying `Source = "dispensary"` would be filtered *out* of `peruse laboratory.log` — a record about the laboratory, missing from the laboratory's log. The closed field set made adding one a reviewed act, which is what it is for |
| **The echo stays a restatement of what was said** | It is written at submit and the source is found at execute, a tick later. Filling the source in at submit was tried and refused: `Choices` outlives ticks, so a numbered prompt left open across a brew would execute against an N-tick-old world, and `carry`'s exclusive-source branch would then be *wrong* rather than merely narrow. From→to lives in the completion line, which knows the truth |
| **The caret is reverse video, and needs nothing from `orbs-render`** | A block caret covered the character under it and blinked it. The first plan proposed a per-cell inverse flag — which would have **contradicted §19's own "what a cell holds"** and been the wrong layer besides. The caret is a *quad*, not a cell, so the glyph is simply redrawn on top in the tube's black. Frontend enrichment, exactly as `blink` already is |
| **Completion returns a range, not an append** | `rustyline`'s shape. It matters more here than there: §6's parser resolves `clarty` to `clarity`, and a completer that can only extend is useless the moment the player has typed a near-miss — which is the player this game is built for |
| **Tab's candidate list is Frame content, never a record** | There is deliberately no `scrollback_mut`: the stream *is* the log, and a frontend writing into it makes a session `(seed, submissions)` cannot replay. A Tab press is not a submission. §3's *"unlogged output is forbidden"* governs the orb's output, not the shell's own affordances |
| **The suggestion is spoken, tagged `Hint`** | The first plan had it silent, on the grounds that §14's stream carries what *is*. That argument would equally forbid speaking the half-typed line — **which this codebase already speaks**. §19's own Frame-boundary rule settles it: *"a narrow pane is a visual constraint and must not become an informational one."* Its own kind, so verbosity can drop the most repetitive thing on screen |
| **History is the frontend's, and `Up` filters by prefix** | Not because `Submissions` contains prompt digits — a frontend history has the same problem and needs the same filter — but because `Submissions` is the **replay log**, and reading it for a UI convenience invites someone to tidy it later. Prefix filtering is fish's default and zsh's `history-beginning-search-backward`; the anchor is held for the whole search, or the second `Up` searches the line the first one recalled |
| **No debouncing, no caching, no async** | fish does all three and zsh recommends async, because their corpus is a filesystem and a history of tens of thousands of lines. Ours is a few dozen scene nouns in memory with no I/O — and rule 8 forbids async in the sim regardless. Written down because the prior art all points the other way |

**Deferred, named:** `Ctrl+R` reverse incremental search — every editor surveyed
has it, and against a repeating brew loop it is arguably worth more than Tab —
plus `Delete`, `Ctrl+U`, `Ctrl+W`, word motion, paste, IME and scrollback
wrapping. `Alt` stays unguarded so AltGr can type `@`, which is why `Alt+B`/`F`
insert rather than move.

**`ORBS_LINE` was a prerequisite, not a nicety.** `ORBS_DUMP` submits every
segment, so the input buffer was always empty when the frame was painted — a
caret position, a partial word and a ghost were the three things a dump could not
show, which is every gate in this item.

### Brewing as a tool pipeline — designed, Phase 1 item 1

§10.1 is the design. Five revisions and two independent reviews; what follows is
what the reviews changed, because most of it was load-bearing.

| Question | Decision |
|---|---|
| **Five instruments, four taking Focus** | The athanor is *infrastructure*, not a stage. That is what makes "four tools, four Focus slots" exact rather than argued — §11.5's capacity track tops out at four, and the arithmetic falls out instead of being reached for. Earlier drafts had four stage-tools and had to defend the coincidence |
| **A pane may hold as many production slots as it has instruments** | §9's per-pane cap of 1 is amended. **Not** a reversal of *"one production slot, tower-wide"* below — the counter stays global and the tools draw from it. The first review called this a re-litigation of a settled decision; it is not, and the distinction decides whether the owner is overturning §19 or amending §9 |
| **Attention upkeep moves from per-open-pane to per-concurrent-action** | The real cost of the amendment, and the second review is what found it. §9 makes open panes the *price* of concurrency; four tools in one pane would have bought depth 4 at one pane's upkeep and exposure, decoupling depth from exposure. Charging the action rather than its container is more faithful to §9's own reasoning — concurrency is what upkeep was always pricing — and it keeps capacity 4 meaningful if cut-line item 7 is ever taken |
| **`decoct` is retired, and nothing replaces it** | It went through three answers. First it was repointed at a tool — wrong, because §15 chose brewing to gate the parser *because "a shell-naive tester immediately understands 'make a potion'"*. Then it was kept as an intent-declaring affordance — also wrong, and the owner caught it: a verb that says *brew a potion* while brewing is a four-stage pipeline teaches the player something false, which is the one thing §6's echo mechanism must never do. **The only single command that brews a potion is a spell the player wrote.** A built-in shortcut would hand over free the exact thing §8 says the player is meant to build |
| **`make a potion of clarity` → `grimoire clarity`** | §15's requirement is that the phrase resolve *somewhere useful*, not that a verb exist to satisfy it. Answering a newcomer's most natural sentence with the recipe makes it the tutorial entry point. `decoct`, `brew`, `make`, `mix` and `distil` all stay **claimed** and point here: the Phase 0 naming pass established that a released word does not stop resolving, it resolves to whatever it is nearest |
| **§8's script example was itself the bug** | The design's headline spell contained `brew --recipe=clarity --qty 2` — a built-in doing a whole domain in one line, which no longer exists and arguably never should have. It becomes `invoke brew_clarity --qty 2`, a spell the player wrote. The section is stronger for it: it now shows **scripts composing scripts**, and every line the script saves is a line the player once typed |
| **`move`'s first slot resolves inside its second** | The plan's own headline command did not work: `tower/scene.rs` registers non-place nodes only as children of `cwd`, so `move charcoal from dispensary to athanor` could not name `charcoal`. Three revisions asserted "the scoping rule survives untouched" having only considered materials inside a *tool*, and missed that `from <source>` exists precisely to name what is *not* where you are. Making the source slot scope the material fixes it without relaxing the rule for anything else |
| **Heat is checked at start; the run then completes** | The alternatives were pausing (a countdown, forbidden below) and spoiling (costs progress, breaking *"never ruinous"*). The timing pressure survives because it comes from the burn being **time-based**, not from spoilage risk: idle lit time is pure waste, so you batch both heated stages into one lighting |
| **Constant burn rate** | A load-varying rate would make fuel depend on which tools were mounted when — not a pure function of the tick, and exactly the countdown *"work is an interval"* refused |
| **Ash at burn-out, not per tick** | Per-tick spawning would issue ids into `Children`, and insertion order *is* the parse. A determinism bug no test in the suite would have caught |
| **`balneum_mariae`, not `retort`** | `retort` scored **667** against `revert` (a plain synonym for `undo`), so a bare `retort` could reach `undo` — found only because the second review checked the new *nouns* and not just the new verbs. The rename is also the better instrument: a water bath is *gently* heated, which is what sitting on the athanor should mean. It has the side effect of keeping `retort` a live `Vessel` noun, so the kind is never emptied |
| **`wield` + `kindle`, not `stoke`/`start`/`tend`** | Computed against the real scoring function: `stoke` scores 600 against `stop` and shares its `sto` prefix, `start` prefix-collides with `status`, `tend` scores 667 against `attend`, `exert` 667 against `revert` |
| **Capacity 1 is the form that ships** | Capacity 4 arrives at ~10 h via research, which is Phase 9a. For Phases 1–2 the pipeline is sequential, and the athanor is what makes that a decision rather than a queue. An earlier draft had this backwards — "designed for 4, degrades to 1" — when 1 is the only capacity anyone plays for a year |

**What the reviews cost, and why the practice stays.** Two context-free reviews
found: a headline command that could not parse, the deletion of §15's stated
reason for gating on this domain, a counterweight silently removed, a noun
collision, `is_destructive` having zero callers so a claimed "free" confirmation
did not exist, `purge <tool>` *despawning the tool*, and a fixture collision on
`alembic`. Every one was verified against the code before being accepted, and one
review finding — that this re-litigates *"one production slot, tower-wide"* — was
checked and **rejected**. Reviews are not authorities either.

### The inactivity grace — specified, built, and removed

§5.0 called for a *"~60s inactivity grace so pausing to think is never
punished"*, plus a rule that a pending disambiguation prompt must not reset it
*"otherwise a player could freeze the tower indefinitely by leaving one open"*.
Both are struck. **The clock now runs whenever the window is open, with no
exceptions.**

It was implemented first, which is how the trouble became visible. Two readings
of the grace exist and neither survives:

| Reading | Fails on |
|---|---|
| Idleness **stops** the clock | Deletes the baseline idle loop. §5 makes offline accrual an *unlockable* precisely because until then *"the tower ticks only while the window is open"* — an open window that stops ticking means Phase A accrues nothing at all |
| Idleness **pauses** the clock, resuming after 60 s | Cannot be reconciled with *"advances on wall-clock while the window is open"*, since the clock would be paused through most of ordinary play |

The second reading is the one the "freeze the tower" clause describes, and the
first is the one that sentence contradicts. That the clause only fits the reading
the rest of §5.0 rules out is the tell that the mechanism was the problem.

**Why nothing is lost by removing it.** §5.0 already answers *"pausing to think
is never punished"* in its very next sentence — *"drift and decay rates are slow
per tick; the clock itself is not."* Thirty seconds of thinking costs thirty
ticks, and thirty ticks of drift is nothing. Long absence is answered elsewhere
again, by §5.3's cap on aberration arrival and by damping. The grace was a second
solution to a problem with two solutions already.

**What it cost.** Three things, and they are the general lesson:

1. **It undercut duration-as-scarcity.** §5.0's economy is *"actions take time to
   complete… attention is the real scarcity, expressed as concurrency."* A clock
   that pauses while you deliberate makes a brew cost "twenty seconds of not
   thinking", which is not a cost anyone can reason about.
2. **It created the exploit it then needed a rule for.** The pending-prompt
   clause exists *only* because the grace opens a way to game the clock. Remove
   the grace and the clause has nothing to guard.
3. **It made the clock unpredictable.** "Is the tower running right now?" should
   never be a question in a game whose texture is a world keeping its own time
   while you work.

Typist fairness survives untouched: §5.0 gets *"a fast typist gains nothing over
a slow one"* from there being **no per-command tick cost**, which a wall clock
does not have.

The siege line went with it — *"in a siege, ticks advance on wall-clock without
grace"* has nothing left to distinguish itself from.

### Brewing is gamified at the head of Phase 1; the archive's minigame is not

§10 gives every domain a minigame form and Phase 0 built two of them as
**commands with a duration and no decision content** — `decoct clarity` holds the
slot for twenty ticks and finishes. That is the thing a player would call a
chore, and §5.1's whole model assumes the opposite: manual play is *incident
response*, and a siege drags the player back to it under pressure.

**Why before the script engine.** §8's engine models *a script performing an
action*. If `decoct` later gains stages and decisions, what a script **is**
changes — replay a sequence, make the decisions, or delegate them. Building the
engine against a two-line action is guessing at the thing it operates on.

**Why brewing and not both.** Fleshing out the archive was considered for the
same slot and cut, on three findings:

| | |
|---|---|
| Its "see it" line is **already Phase 9a's** | *"`research` a fragment and gain a verb you did not have."* That needs `Verb::ALL` to stop being a fixed sixteen, the synonym table to stop being `const`, `is_live` to stop being a `const fn`, and the boot tutorial to read all three dynamically — plus §18's unstarted naming pass. That is the discovery loop pulled forward two phases, not a minigame |
| §10 **pre-authorises the cheap version** | *"decipherment becomes mostly a resource sink with occasional authored set-pieces"* — and §7's only depiction of the verb is exactly that. A bespoke puzzle is an ambition increase, not a debt being paid |
| It is better bought later | The archive gates all discovery. What the puzzle should feel like depends on what it unlocks, and nothing is unlockable yet |

**The domain has to be finished before it can be deepened.** `siphon` is dark,
there are no potions, no reagent consumption and no vessel mechanic;
`retort`/`crucible` are nouns nothing reads, and a completed brew yields a node
called `residue-N`. §11.5's Resources table specifies the missing half. A
"quality gradient" was proposed before this was noticed and had **no object to
carry the quality**.

#### Decisions, not execution — which is what keeps the harness working

The open question was whether hand-played brewing should beat scripted brewing.
§8 invariant 3 only says a script *completes faster*, so a quality edge is
compatible with its letter — but §8:710 also says the three mechanisms
*"guarantee automation dominates for a present player — which draft 3 failed to
deliver"*, and §11.5 says *"not automating is never ruinous, only slower."*

The distinction that dissolves it:

| Outcome depends on | Harness | Verdict |
|---|---|---|
| **What the player chooses**, given readable state | A script encodes a policy; `orbs-balance` sweeps policy against state | ✅ |
| **How well the player executes** — speed, precision | The harness has no player, so it cannot sweep anything | ❌ |

The second is the shape §19's Phase 0.5 entry already refused, for the same
reason: a modelled player-time cost would mean *"`orbs-balance` simulating
typewriter delays"*, which is the divergence §13 exists to prevent. It is also
backwards — §8 gives **scripts** the timing precision a human cannot hit.

So a script is never worse at brewing; it is worse at *noticing*. A fixed policy
meets a tower state its author did not anticipate, and a present player does not.
That is the honest version of "manual matters sometimes", it costs the harness
nothing, and it is the same argument §5.1 already makes for nuisances.

#### Two things this surfaced that no phase owns

- **Nuisance aberrations have no roadmap item in any phase.** Phase 8 lists only
  the adversarial ones; Phase 0 shipped log-poisoning drift alone, and
  `RngStream::Aberration` is unrolled. §5.1 calls nuisances *"the core of manual
  play"*, so this is a gap rather than a deferral.
- **The world clock's inactivity grace was unbuilt** — and stayed that way. It
  was built next, examined, and removed; see the entry above. A brewing decision
  window would have been a pending prompt, which is exactly the case the grace
  needed a special rule for, and is part of why there is no longer a grace to
  need one.

### Phase 0.5 — the orb becomes a machine that moves

Three aesthetic items, requested between the vertical slice closing and Phase 1
opening. Every one of them turned out to have a rule already pointing at it.

#### The boot sequence — two screens, and why that is not a duplicate

§4 already makes *the tower's* boot a status report reflecting real world state,
and that report is built. What Phase 0.5 adds is the **machine** waking up in
front of it: dark tube, strike, prompt, the pane border drawing itself, then a
POST naming what the orb is made of.

| Question | Decision |
|---|---|
| **Sequential, not merged** | The POST is the machine; §4's report is the tower. To stop them reading as one screen twice, the POST is a **centred title card** — no pane, no `name qty state` columns — and the border does not exist until the stage that draws it |
| **The versions are real** | `tower/boot.rs` exists because *"a report that could go stale is a lie the player reads first."* A POST printing invented numbers is the same lie one screen earlier. Bevy's is the exact pin, held to `Cargo.toml` by a test; Rust's comes from a build script |
| **The world does not tick during it** | `tower::drift` rolls once per tick, so ticking through a wall-clock animation advances the RNG stream by an amount depending on how long boot took and whether anyone skipped. **The same seed would build a different world.** A correctness fix, not polish |
| **Skip is a keypress, and §4 asked for sticky** | ~~Sticky needs persistence, which does not exist and arrives with §15's Phase 11 settings screen. The keypress is the honest half-measure.~~ **Superseded — the keypress skip is removed entirely (below).** §4's sticky skip still stands and still waits on Phase 11 |
| **Any key, including the bound ones** | ~~`F10` quits, and a player reaching for it during boot means *skip*.~~ **Superseded with the keypress.** Every keyed system is still gated on `booted`, which is now the whole story: until the game is up, a keystroke does nothing at all |

#### The keypress skip is removed

**Any key went straight to the game. It no longer does.** The sequence runs to
the end, every launch, and only `ORBS_BOOT=0` — a development affordance, not a
player-facing one — starts past it.

The reasoning above was about a sequence *"long and meant to be skippable"*. The
owner's judgement is the opposite: at ~14 s it is **character rather than a
wait**, and a keypress skip made the first thing a player ever does to this game
be dismissing it. A CRT warming up is the game introducing itself; an interface
that flinches away the moment you touch a key teaches that its own atmosphere is
an obstacle.

**This does not touch §4's sticky skip**, which is a different mechanism for a
different person: a remembered setting for someone on their fortieth launch,
chosen once. It still waits on Phase 11's settings screen, and `Boot::finished` is
the state it will select.

**What was actually gained by removing it:** the `booted` run condition stops
being a guard against the skip keystroke doing two things at once, and becomes a
single plain rule — until the game is up, a keystroke does nothing. The `F10`
special case in the entry above disappears with the mechanism that needed it.

**Three Rust versions existed and two were wrong.**
`env!("CARGO_PKG_RUST_VERSION")` is **empty** in `crates/orbs` — it does not
inherit `rust-version`. The workspace's `1.95` is a floor, not the compiler in
use. `rust-toolchain.toml` pins a third. A build script capturing `rustc -vV` is
the only one that is true, and this is recorded because the first draft of the
plan confidently proposed the empty one.

#### Panes arrive over time

A pane appearing between one frame and the next reads as a glitch.
`ScreenLayout::transition` interpolates; the frontend owns the clock. Two
defects the tests caught before the screen did:

- **`Rect::EMPTY` is `(0, 0, 0, 0)`,** so a pane lerped from it grows diagonally
  out of the top-left at half height, through the pane beside it. Panes are born
  from an explicit **edge** rectangle instead, which differs per mode: Deep
  slides in from the right, Wide unrolls downward, because §9 puts extra panes in
  different places.
- **Rounding an origin and its extent independently** lets both round up at the
  same instant — `col 147.5, cols 12.5` becomes `col 148, cols 13`, whose right
  edge is a cell past *either* endpoint. The far edges are interpolated and
  subtracted instead.

Transitional layouts **suspend tiling's no-gap, no-overlap and no-zero-area
guarantees** by design; `compute` keeps all of them. What holds instead is
weaker and sufficient: no pane leaves the span of its own two endpoints.

**`F4` adopts the target grid instantly and animates only the split.** Deep focus
raises fidelity a step, so it resizes the whole grid; interpolating between
layouts computed against two different grids is not a meaningful operation, and
animating a grid resize is a different and much larger feature.

#### Output arrives a character at a time — and must never be a mechanic

Requested with a stated motive: *"one of the detriments to running stuff manually
versus scripting it."* The motive is sound and the implementation must not honour
it, because a **modelled** waiting cost would mean:

- `orbs-balance` simulating typewriter delays to stay in step with the live game,
  which is the divergence §13 exists to prevent;
- offline catch-up — ~29k `step()` calls — owing animation time;
- **§9's parity rule inverting.** *"If strips ever showed less, the setting would
  become a difficulty choice, and a player who needs large text would be paying
  for it in capability."* Make waiting a cost and turning the animation off
  becomes a competitive **advantage**, aimed at exactly the players §14 exists
  for. §14's *"no mechanic requiring fast typing"* points the same way.

So it is pure presentation, and the detriment lands anyway: it costs seconds of
attention, and a player who automates stops spending them. That arrives in Phase
1, when a bound script running twenty commands is not a person watching twenty
reveals. **The payoff cannot be felt in 0.5** — there is nothing to script yet.

Constraints that shaped it: §19's echo-immediacy decision still holds, so the
first character lands on the frame the record does; rows never change, so nothing
below a half-arrived line walks down the pane; a half-arrived record is silent,
because §14's stream is whole records in order; and **any keystroke completes
it**, which is what keeps it from ever being a cost.

#### The strike was cut, and nothing in the game flashes

It shipped as a flash plus a bright band sweeping down the tube; the band went
first, for reading as a *fault* rather than as a tube coming on; then the flash
went too. **The screen opens black and the prompt types itself.**

The reason is not safety, it is fiction. The game is a wizard who finds a
computer inside a scrying orb — an orb is *found*, not switched on, and a CRT
power-up beat was borrowing an idea from the wrong object. Nothing that arrived
after it needed it either: the prompt appearing out of black is a better opening
than the prompt appearing after a bang.

**The consequence worth recording: no part of this game flashes any more.** The
only photosensitivity exposure Phase 0.5 ever created is retired, which is also
why §14's health warning stops being urgent — it lands with the Phase 11 settings
screen alongside the persisted CRT toggle rather than ahead of it.

The two entries below are kept because the **reasoning** was wrong twice, in
opposite directions, and that is the part worth not repeating. `CrtSettings`
keeps its `flash` field: §4 reserves it for *flash on breach*, which is Phase 8.

#### The flash limit is per window, not per second of effect

The strike's arithmetic was wrong twice, in opposite directions, and both cost
something.

First it analysed the flash and the sweep **separately** — the mistake §19 already
records from the 19.1 Hz strobe, two constants each looking fine alone.

Then, having combined them, it divided the flash count by the stage's length and
called the result a rate. **That is not what WCAG 2.3.1 bounds.** The limit is
three general flashes occurring *within* any one-second window, and a single
non-repeating flash is **one flash in that window however briefly it lasts.** A
0.25 s strike and a 0.9 s strike are both 1.

Treating it as a rate made short strikes look unsafe — 1 pair over 0.3 s
"computes" to 3.33/s — and that was the stated reason the stage stayed long
enough to stop reading as a tube striking at all. **Being wrong in the cautious
direction is still being wrong**, and here it was the thing making the effect
bad.

The stage length is now a taste decision (250 ms). The safety property is a
different one and is what the test asserts: the flash rises **once**, falls once,
and does not recur — checked over the curve, because a second rise is the change
that would matter.

#### The strike — one soft flash, and a sweep that was cut

Shipped first as a flash **and** a bright band sweeping down the tube. On a
near-black background both are *general flashes* — the band is a rise and a fall
at every pixel it crosses — so the pair cost:

| | |
|---|---|
| Flash | 1 pair |
| Sweep | 1 pair |
| Over 0.6 s | **3.33/s — over WCAG 2.3.1's three-per-second limit** |
| Over 0.9 s | 2.22/s — inside it |

**The first draft analysed the two separately and was wrong to.** That is
precisely how the 19.1 Hz strobe recorded below got through: two constants that
each looked fine on its own.

**The sweep is now cut**, on looking at it — it read as a fault rather than as a
tube striking, which is a different thing from being unsafe and a better reason
to remove it. What remains is one pair with a long decay, and the flash's
amplitude came down from 0.75 to 0.30: additive on a near-black screen, so that
number *is* how bright the tube gets, and the original whited it out.

With one pair over a stage now measured in seconds, the rate is under a quarter
of a flash per second and **the stage length has stopped being a safety
constraint at all.** That is worth stating because the previous entry said the
opposite, and the test that guards it now asserts a floor with an order of
magnitude in hand rather than a value tuned to sit just inside the limit.

**The off switch is still not the safety mechanism, because it is not
persisted.** `F3` works in-session; settings and the health warning §14 records
this product inheriting arrive together in Phase 11.

#### Amber is the default, and there is a fourth theme

§4 opened on muted violet because it *"reads arcane rather than computer."* On a
finished screen that turned out to be the argument against it as a default: amber
is what a real phosphor terminal looked like when it was not green, it is warmer
than either alternative, and a thing a wizard stares into by candlelight should
look warm. Violet stays on the list — it is the one theme nobody mistakes for a
real terminal, which is a reason to keep it and a reason not to open on it.

`Theme::default()` is now `ALL[0]` rather than a constant by name, because the
list's order is what `F2` cycles and a default outside that order makes the first
keypress do nothing visible.

**A fourth theme, monochrome.** Every other theme is one hue at three weights,
which is what a real tube did; this one is neutral text with the accents carrying
all the colour there is. It is the highest contrast the game offers, and its
**base ramp** has no hue in it to lose.

> **It no longer carries the accessibility guarantee**, and the change is
> deliberate — §19. This paragraph used to say monochrome was *"the only theme
> where a player with a colour vision deficiency loses nothing"*, and two later
> decisions retired that: the athanor burns one orange ramp on every tube, and
> materials carry a tint hinting at what is inside an instrument. Both put hue on
> this theme.
>
> The guarantee moved to **Phase 11's colour-vision filters and true greyscale
> mode**, which is a better home for it than a theme ever was: a theme made the
> accommodation an aesthetic choice, so a player who needed it had to give up
> amber to get it. A filter is orthogonal to the theme, which is what an
> accommodation should be. Monochrome is a grey *aesthetic* until that item
> lands.

Its values are **solved, not picked**, and the first attempt failed: a light base
leaves very little luminance headroom above it, and `success` landed 1.22:1 from
body text — inside the greyscale-separation margin §14's tests demand. Darkening
it made it *worse*, because body sat between the two. Lowering the **base** to
0.74 is what bought the accents room. That is the same lesson §19 already records
from the first palette pass and from the CRT overscan: compute the constant.

#### The POST is a logo and three checks

The card names the game in **block glyphs** — CP437 has a full block and the
double box-drawing set, which is what the letterforms are built from, so the
repertoire test covers it like any other text. Centred as a *block* on the widest
row: the rows are not all the same length, and centring each on its own width
shears the letterforms apart by a column.

**The logo prints a character at a time too**, six rows at once — `O`, then the
full stop, then `R`. That is only possible because the letterforms are
**column-separable**: no glyph shares a column with its neighbour. The column
ranges were *measured* rather than counted by eye, and they are irregular (`O` is
nine cells, the other letters eight, the full stops three), so they live in a
table that a test holds to covering the art exactly — no gap, no overlap, one
entry per character of the name. A hand-written table that drifted from the art
would fail silently, drawing a glyph a column off or leaving a sliver never drawn
at all.

The logo also **speaks only as much of the name as is on screen**. §14: what a
reader hears is what the screen says, which during the print is `O.R`.

**The words do not type; the dots do.** Each line is `label ....... ok`, the
label landing whole, its leader filling the way a progress indicator fills, and
`ok` snapping in behind it — then a pause before the next line. A name arriving
one letter at a time reads as a *slow machine*; a leader filling reads as
*something being checked*, which is what a POST line is.

**Every line spans the logo, edge to edge.** The label sits under the logo's left
edge, `ok` ends flush with its right, and the leader is however long the gap
between them happens to be — so the run varies per line, which is what a leader
*is*.

That replaced centring, which made the card twitch: a centred line is positioned
by its own width, and its width grows by two the moment `ok` lands, so every row
shunted sideways at the end of every check. **Anchoring both ends to something
that is not moving is what makes nothing move.** The general form of the mistake
is worth keeping — *centring anything that grows will move it* — and the fix is
always to anchor rather than to compensate.

The game's own version is not on the card. The logo is the game saying its name,
and a version line under a six-row letterform would be the only small text there.

#### The sequence is paced to be read

4.4 s for the whole thing, first time out. The two stages that actually animate —
the frame drawing itself, the dependencies reporting — were over before they
could be followed, which made them decoration rather than a sequence.

**Four times slower.** The judgement being recorded is that a boot sequence
nobody can read is worse than one that takes a beat, because the second at least
works the first time.

*(Written when it was 17.6 s and skippable with any key. The sequence later lost
its opening flash and settled at ~14 s, and the keypress skip was removed
outright — see the entry above. `ORBS_BOOT=0` remains, as a development
affordance rather than a player-facing one.)*

#### A fourth channel on `Style`, for the one thing that means nothing

The athanor's meter is drawn as a fire: flame glyphs in the filled portion, a
plume in the empty one, both on per-theme colour ramps. That needed a colour
family §4 does not have, and §4 describes its palette as closed — base hue at
three weights, plus an accent triad, *"accents are never decorative"*.

**`Depiction` is admitted as an explicit exception, on the grounds that it says
nothing.** §14's rule is that colour is never the *sole carrier of meaning*, and
the guarantee behind it is that every `Role` reaches the linear stream beside its
text. A channel carrying no meaning has nothing to withhold from a listener, so
it cannot break that guarantee. Three things hold it to that:

- `Style::depicted()` yields `None` on any accented cell, so a `Role::Danger`
  cell can never render in flame colours. The rule lives in the accessor rather
  than at each call site because there is one call site *per frontend*, in
  different crates.
- A test asserts a burning meter's linear stream is byte-identical to a plain
  one's.
- The athanor's bar drops to `Role::Normal` while alight. Its meaning is the fill
  boundary and the panel's spoken summary, not an accent.

Rejected: putting it in `Role` (a category error, and it would distort the
greyscale-separability tests the accent triad exists to pass) and in
`Presentation` (which selects a glyph-atlas face and is pinned by
`presentation_never_changes_the_colour`).

**Per-theme ramps, not one fixed orange.** An orange fire on the green phosphor
is a colour that tube cannot make, and `MONOCHROME` — the accessibility theme,
whose entire claim is that there is no hue in it to lose — would have gained one.
Amber's flame ramp *is* orange into yellow because amber's base hue already is.

Two constraints that were not obvious until the values were solved rather than
picked, which is the same lesson the base palette taught:

- **Monochrome has no headroom.** Its `Bright` is `rgb(1.0, 1.0, 1.0)`; nothing
  is hotter than white. Its flame tops out *at* white, and the test compares the
  hottest step against `Normal`, not `Bright` — the obvious phrasing is
  unsatisfiable for the one theme that most needs keeping.
- **Violet's embers had to be raised well past what the eye would pick.** Violet
  luminance is carried almost entirely by its red channel (blue weighs 0.0722
  against green's 0.7152), so a purple that *looks* like a deep ember contrasts
  2.7:1 against that theme's background — under the 3.0 floor — and is
  simultaneously darker than its own smoke. Both tests failed on the first pass.

#### Fire motion, and a deliberate exemption from the photosensitive band

**`FLIP_HZ` is 6 Hz, which is inside the 3–30 Hz band this document elsewhere
says to stay out of.** That is a departure taken knowingly, and this entry exists
so it is never mistaken for an oversight. It shipped at 2.5 Hz first and was
raised on request, because at 2.5 the fire read as a slideshow.

The band is a rule about **flashes covering a substantial share of the visual
field** — W3C puts the threshold near a quarter of it. The 3 Hz floor this
project adopted was calibrated on the *whole tube* flickering at 19.1 Hz. The
athanor's meter is a bar two cells wide: three orders of magnitude less area, and
nowhere near any published threshold. Applying a whole-screen number to it is
conservative rather than correct.

The exemption is conditional on all three of these, and **if any is removed the
rate comes back down with it**:

- **Nothing turns over together.** Every cell's tick boundary is offset by a fixed
  fraction of a tick, so a change is a few cells out of thirty rather than the
  strip as a whole. Whole-field modulation is the hazard; motion is not.
- **Each step is small.** The flame ramp is deliberately compressed, so a flip is
  a hue step rather than an on/off flash.
- **The element stays small.** Two cells wide, one instrument, one room.

It remains **held by construction**: a cell's appearance is a function of its tick
index, and the quantiser advances that index 6 times a second and no faster,
whatever the noise does. The test changed shape rather than number — it asserted
`FLIP_HZ < 3.0`, and now asserts the rate is bounded, known, and obeyed per cell.
An earlier design bounded *aggregate luminance* across the bar instead; that test
is satisfied exactly by an alternating checkerboard, which is the pattern trigger
the same band covers.

**It rides `CrtSettings::on`.** An env var is not a switch a player can reach,
and the entry below is precisely the failure of gating motion on something
indirect. F3 cycles the tube to `OFF`; that now stops the fire too. The
persistent per-effect toggle stays with Phase 11's settings item — and at this
rate that item has a real dependency rather than a nominal one.

#### The fire's shape, and what it cost at the boundary

**Hottest at the base, mellowing into the tip.** The first version put the
brightest cell at the flame *front*, reasoning that the front is where fuel is
being consumed. It read as a bar with a bright edge rather than as a fire; what
an eye expects is a glowing bed fading upward. The bottom half is solid `█` and
carries all its motion in hue; `▓` begins about halfway up and grows commoner
toward the tip, where a real flame breaks up.

`Heat` went from three steps to four for this. The earlier argument was that
CP437 gives flame only two glyphs, so a longer ramp has nothing underneath it —
which stops being true the moment the bottom half is glyph-constant and the ramp
is working alone down there.

**The fill boundary was relaxed and then bought back stronger than it started.**
Letting `▓` reach the flame tip means letting it reach the topmost lit cell,
which is the join, so the rule went from `█`/`░` to `█`-or-`▓` against `░` — a
3:1 coverage step where the plain meter draws 4:1.

Then `░` was given up entirely: the fire meter's empty track is now **blank**,
and `░` became the last of a puff of smoke pittering out. The join is `█` or `▓`
against *nothing*, which is the strongest the alphabet can draw, and it arrived
as a side effect of an aesthetic request rather than by aiming at it. The join
that stays forbidden throughout is `▓` against `▒`: one dither step, which the
CRT's bloom erases, and the meter's *value* is read off this join. The **dark
side is pinned absolutely**, sparks included.

This is the fire meter only. The plain `meter` keeps its `░` track — the other
four instruments are gauges rather than fires, and an empty track there would
read as a missing bar rather than as clear air.

**A puff's height and its age are the same number.** A puff at height `h` left
the fire `h` ticks ago, because it rose a cell a tick to get there. So thinning
smoke by its distance from the fire is not the stationary threshold that killed
an earlier version of the drift — the puff fades *because* it travels. Getting
this wrong once cost a debugging pass; writing it down is cheaper than earning it
again.

**The plume gets a shared clock; the flame keeps its per-cell stagger.** The two
properties genuinely conflict — a translation only reads as one if neighbouring
cells step together, and staggered they never line up. Measured, the drift washed
out to a coin toss. The exchange is safe because `FLIP_HZ` is unaffected and what
moves together is `░` against `▒` on the dim half of the bar, while the bright
half stays decorrelated.

#### The mortar, and what a second animated instrument cost

The athanor's fire was one instrument's picture. The mortar is the second, and
building it is where the shape for the remaining three got settled.

**The other four need no new colours.** Fire needed a whole `Depiction` family
because *fire is orange* — a hue no phosphor theme has. Grinding does not: it
lives in the base hue at `Dim`/`Normal`/`Bright`, which already exists and is
theme-safe. Five instruments each with their own ramps would have been about a
hundred hand-solved colours; the athanor stays the exception.

**The bar is one lump of material, and grinding reduces it.** An instrument's
meter counts ticks elapsed, but drawing that as a filling bar says nothing about
a mortar. A mortar does not fill a container — it *breaks large things into small
ones*. So the bar is a solid block seen edge-on: it gives way at its underside,
the pieces snow down through a working gap, and they collect as a coarse bed at
the bottom.

**The four shades are four states of one substance**, which is what makes the
picture legible with no legend: `█` whole, `▒` and `░` in pieces and in the air,
`▓` broken and settled. That is the whole vocabulary, and its economy is the
point.

Material is conserved on screen. The block's underside sits a *fixed* gap above
the bed, so as the bed rises the block is **eaten rather than pushed** — the bar
is always full of something and what changes is how much of it is broken. A block
that kept its height and rode upward would leave a growing hole at the top, and
the bowl would read as emptying rather than grinding.

That framing is what let the panel draw states it previously could not. A
**loaded** mortar is a solid bar of `█` — the clearest "there is something in
here" the alphabet has — and a **finished** one is all `▓`, held until the tool
is emptied. Neither is a special case; both are states the sim reports
`meter: None` for, exactly like the cold hearth.

**There is no tool in the picture, and that was the third attempt.** The first
two put a pestle in the gap:

1. `╥`, which looks most like the tool — a head with a stem pointing at what it
   is about to hit. It fails on the *horizontal* layout, which §10.1 reaches
   whenever the pane is taller than it is wide and a player reaches with F4:
   there "down" points at the border, and `╥` reads as a stray box-drawing
   character.
2. `■`, direction-neutral so it survives the rotation, with its own cool ramp
   (`Depiction::Tool`) so it read as stone rather than as the reagent it was
   crushing. It worked, and it was still wrong: a mark from outside the fill
   vocabulary reads as an *object visiting the bar* rather than as the material
   changing state. The bar has two cells to say something in, and spending one on
   a tool costs the thing the bar is actually about.

`Depiction::Tool` and its four hand-solved ramps were removed with it. An API
with no callers is unshaped (§15), and keeping a channel against a possible
future use is how a palette grows colours nobody looks at.

**The fall is slower than the clock.** One cell every two ticks rather than one
per tick, which puts every cell in the gap at 1.5 flashes a second — under the
3 Hz floor, so unlike the fire this needs no exemption. At the full rate the
debris streaks rather than falls, so the safe choice is also the better-looking
one.

**The bed creeps between the world's ticks.** §5.0 turns the world at 1 Hz and
the sim reports whole ticks, so a meter read straight off it moves once a second
in one jump — which is what the animation was decorating around rather than
fixing.

The fix is not a faster tick. §5.0's rate is what makes duration scarce and it
governs replay, offline catch-up and the balance harness; changing it to make a
bar look nicer would be the tail wagging the dog. What changed is the recognition
that **the tick count is a *sample*, not the quantity**: an eight-tick grind is
eight seconds of work, and at three and a half seconds it really is
seven-sixteenths done. Drawing between samples is therefore *closer* to the truth
than the sample is, which is the opposite of the trade the fire's flare makes.

`Painter::creeping` does the interpolation and a frontend supplies the `0..1`,
the same division as the flare and the pane tween. The Bevy build reads it from
the **same `Time<Fixed>` the sim steps on**, so it cannot disagree with the sim
about when a tick lands — a self-counted copy would, the first time the clock
hitched, and the symptom would be a bar arriving at a cell just before or after
the tick it belongs to.

`stop` mid-tick is the one case where the prediction was wrong; it corrects on
the next frame. With motion off the bar snaps to whole ticks, which is the honest
fallback: a player who turned animation off gets the sim's own sampling rate.

**Tempo is the organising principle for the rest.** The fire shimmers
continuously and fast; the mortar's debris drifts steadily downward at half that
rate. Each remaining instrument should get its own, so what is running is legible
from across the room without reading a word.

Two measurement traps are worth keeping, because both cost a confusing failure:

- **Transitions and flashes are not the same number.** A flash is a *pair* of
  opposing changes, so the flash rate is half the transition rate. A test
  asserting transitions against the 3 Hz floor failed at 4.58 and the failure was
  the units rather than the design.
- **A phase landing on a half-tick rounds either way.** `shared_tick` rounds, so
  advancing a phase by exactly two ticks can move the index by one instead of
  two. Sampling *on* ticks (`tick / FLIP_HZ`) is the safe point; a tick *centre*
  is precisely the boundary, which is the wrong guess and the one that was made
  first.

**`Craft` moved into the sim.** A frontend picking its picture by matching on the
literal `"mortar_and_pestle"` would re-derive in its own source what
`recipes.toml` already knows, and `orbs-tui` would derive it a second time and
could disagree. `Instrument` now carries what the thing *does*, for the same
reason it already carried its two-letter form. The heat source answers by
component rather than by name, because `heat::source` already refuses to find it
by name — a *reagent* called `athanor` on the floor would have matched.

**The animation clock is now `shell::bench`, not `shell::fire`.** One clock, one
reduce-motion switch, per-instrument views (`burn`, `grind`). Five clocks would
be five places to get the one number with a safety argument attached wrong. The
rename was cheap with one consumer and would not have been with five.

#### Three states a hearth has that a gauge does not

The meter had two readings — how full, and nothing else. A hearth has more, and
the panel exists precisely so a player never has to touch a thing to learn its
state (§10.1).

**A cold hearth smokes.** `State::Cold` reports **no meter at all** — there is no
quantity left — so it drew nothing, and "out" was indistinguishable from "the
panel forgot this row". It now draws a wisp off the bottom, tapering over about
three cells and clear above: the only bar in the game with no quantity behind it,
and the only one drawn past the check that skips every meterless instrument.

**A guttering hearth keeps the last of its orange.** A fire with a handful of
ticks left divides to zero cells, and an empty bar says *out* — wrong, and wrong
in the direction that matters, since "still lit" against "cold" is exactly what
`kindle` turns on. One faint `▓` ember says so. This is **the only place the fire
deliberately shows more than the plain meter would**, and it is always `▓` and
never `█`, so it cannot be misread as a cell of fill. The ember gets the same
pinned blank above it that a real flame front does — without that it sat directly
against `▒`, which is the one join forbidden everywhere else.

**Lighting one flares, and the flame grows up out of the base.** Over one world
tick the fire climbs from a single cell to the full height of its fuel, burning
at the top of the ramp where it has just caught and settling behind itself.
Everything above the front is drawn as **nothing** — fuel that is present but not
alight.

This took three attempts and each failure is worth keeping:

1. A uniform heat boost over the whole flame. The bar lit instantly and the
   flare was a colour that faded. Nothing climbed.
2. A boost applied only below a rising front, with the fuel above it drawn as
   *dark flame* — which is what "present but not alight" literally is. That
   filled the entire bar with dithered orange the instant `kindle` landed, and
   the flare read as a highlight sweeping over an already-full bar.
3. The fuel above the front drawn as nothing at all. The flame grows into empty
   space, which is the thing an eye reads as catching.

**This is the one place the fire meter shows less than its value**, and only for
the tick after ignition. That is a real cost and it is paid deliberately: it is
the same trade `Reveal` makes for arriving text and `ScreenLayout::transition`
makes for arriving panes — a value animating *to* the truth reads better than one
teleporting to it. §14 is unharmed because the linear stream never sees the
animation; the panel's spoken summary says *burning* from the first frame.

**One tick is the duration, and that is not arbitrary.** §5.0 makes a tick one
real second, the bar redraws at frame rate, and the world advances at 1 Hz — so a
tick is the longest an animation can run and still finish before anything it
describes can change. The flare ends exactly as the fuel it is burning ticks down
for the first time.

The boost is four ramp steps, which saturates what has just caught at `Core` and
leaves the flame's own tip cooler. Six would take every caught cell to `Core`, and
a bar at one flat colour reads as a UI flash rather than as something catching
light. The front never starts at zero — at the exact instant of ignition that
blanks the whole bar for a frame, right when the player is looking for something
to have happened — and it is floored at one cell *only when there is fuel*, since
flooring an empty bar invents a flame on a spent athanor.

**Ignition is an edge, and the panel is not a reliable place to watch for one.**
It only carries the room the player is standing in, so "no athanor visible" had
to be distinguished from "athanor out" — otherwise walking out of the laboratory
and back sets off a flare every time you come home. `Fire` also starts seeded
*lit*, so a tower that opens with the athanor already burning does not flare on
its first frame.

**Sparks carry a fixed identity for their whole life.** A spark's drift
coordinate is constant while it travels — position and tick rise together — so a
single hash settles both whether it exists and how high it gets, and it simply
moves. A lifetime read off *distance from the fire* instead is a fixed ceiling
every spark dies at, which reads as a hard edge ruled across the plume. They are
barred from the boundary cell, and a spent athanor throws none: it is still
`Burning` for the tick before it goes out, and sparks off an empty bar would say
there is fuel left at the exact moment the bar says there is none.

#### An accessibility switch something else could flip

`CrtSettings::enabled` was derived as `settings != OFF`. Both `flash` and
`desaturation` are documented in that same struct as *reserved for world state*,
so the moment anything drove one — a breach flash, threat desaturation, this
phase's boot strike — the settings stopped equalling `OFF` and **barrel,
scanlines, grille and vignette all came back for a player who had turned them off
for motion sickness.**

Now explicit state. The lesson generalises: a switch inferred from the absence of
something is not a switch, because anything that adds that something turns it
back on.

#### A blank screen was uploading an empty mesh sixty times a second

Found by running the game, not by any test. Bevy 0.19's slab allocator answers a
zero-vertex mesh with `use-after-free: attempted to copy element data for an
unallocated key`, and **the boot sequence is the first thing in this game to hold
a blank screen for more than one frame** — ~250 errors in 1.3 seconds, none of
them present one commit earlier.

`Assets::get_mut` marks an asset changed whether or not anything is written, so
the fix has two halves: skip the rebuild entirely when there is nothing to draw,
and start the grid with one degenerate transparent triangle rather than nothing.

**The general point.** The gate is `cargo test`, `clippy`, `rustdoc` and
`cargo build` — and all four were green with 250 GPU errors a second scrolling
past. §15's *"work is done when it has been looked at"* covers reading the log,
not only the screen.

### Boot sequence, scaffold tutorial, and what looking at it cost

§4 asks for a boot report that reflects **real world state**, and §15 asks for a
throwaway scaffold tutorial so the gate *"measures the parser rather than the
absence of onboarding"*. They are one screen: the report is built by walking the
tower, and it ends by naming the vocabulary.

| Question | Decision |
|---|---|
| **Walked, not written** | A boot report that could go stale is a lie the player reads first. Every row comes from the world — one per domain with what it holds and whether it is sound, plus §8.1's `bound` count so automation cannot be forgotten |
| **`bound: 0` is printed, not omitted** | The script engine is Phase 1. An absent section leaves a player unable to tell *none* from *not shown*, and this is the line a Phase 1 script slots into |
| **The tutorial names verbs that work, not verbs that exist** | Six of §6.1's sixteen only acknowledge in Phase 0. Offering them spends the gate's most important metric — the dead-end rate — on things nobody has built. `execute::is_live` is where "works" is written down, and a test drives all sixteen through a real `Sim` to keep it honest |
| **Names, not sentences** | The same line §19 already drew: the parser's tables emit facts. §4's *"the orb warms to your touch"* is composed by a Phase 1 content file from exactly these records |

**`bind` was worse than a dead end, and only the running game could show it.**
Phase 0 has no scripts, so `bind`'s only slot is unfillable — and the deliberate
`find`/`bind` collision in the vocabulary table then hands the line to `sift`
unopposed. `bind night_watch` searches the session log and **reports success.**
The resolution is correct and stops being reachable the moment Phase 1 puts a
script in scope; what was wrong was a tutorial that would have taught it. Found
by a test that types every verb, which existed only because the tutorial needed
to know which verbs work.

**A listing is a set, and sets tile.** Sixteen verbs stacked one per line pushed
the boot report's own first row off an 80×22 screen before anyone had typed
anything — using eighteen columns of eighty to do it. `RecordKind::tiles` now
packs a run of listing rows across the pane, and exactly one kind tiles: `Entry`,
whose rows are unordered. Log lines, script lines and schedule rows are read *in
sequence* and tiling would scramble them; a status row wants its value column
aligned with the one above it. A screen reader is unaffected — each record still
speaks separately, in stream order, so the wrap is visual and nothing else.

**The pane had to be taught to ask.** One row per record stopped being true, and
the transcript's "show the tail" arithmetic still assumed it: every packed
listing left that many blank rows at the bottom *while dropping the same number
of records off the top*. `RecordView::height` shares its packing decision with
the drawing code so the two cannot disagree, and the pane widens its window one
record at a time until the next would overflow.

### `ORBS_DUMP` — because a screenshot can lie

CLAUDE.md's working practice is that work is done when it has been *looked at*,
and `ORBS_CAPTURE=1` was how. That path needs a composited window. Run the binary
from a detached shell, or with the display asleep, and it writes a valid PNG of a
**black rectangle** — the renderer fine, the picture proving nothing. That is
worse than no picture, because it looks like evidence, and it cost most of an
afternoon before the same frame was checked another way and found to be correct.

`ORBS_DUMP=1` draws the same frame the game draws — the real `paint`, the real
`Sim`, the real `ScreenLayout` — into a `Frame` nobody rasterises, and prints it
with its linear stream beneath. No `App`, no `DefaultPlugins`, no GPU, no window.
`ORBS_DUMP="attend laboratory; decoct clarity; meditate 25"` types a session first,
through `submit` and a real `step`, so what prints is the world having actually
run. `ORBS_GRID=160x44` picks the grid.

What it cannot show is what rule 2 says is a frontend's alone: phosphor, the CRT
curve, the blinking caret. Those still need eyes on a window — and rule 2 is
exactly the promise that nothing *informational* is among them.

### Worst-case legibility — prepared, not judged

§15's hardest screen: tier 2 at the minimum supported window, four panes, siege
in progress, peak-threat CRT, eldritch active, and a **single-character sabotage
tell** to spot. The item also had to *establish* the minimum window at which
tier 2 is offered, and that number is now derived rather than written down —
§19's standing lesson from four failed attempts at the CRT overscan is *compute
the constant, do not reason about it*.

| | |
|---|---|
| Minimum window offering tier 2 | **1280×704** |
| There, tier 1 | 2× cell → 80×22 |
| There, tier 2 | 1× cell → **160×44** |
| A one-character tell at tier 2 | **8 physical pixels wide** |

Rendered by the last screen of `cargo run -p orbs-render --example screens`. The
one-space tell survives to the Frame at that size, and the eldritch pane still
speaks plainly — which is the property that makes a *visual* legibility failure
survivable rather than exclusionary.

**Two of the six conditions are not in a Frame and cannot be.** Peak-threat CRT
and the phosphor are frontend enrichment under rule 2. The screen establishes
that everything informational survives at the smallest glyph the game ever draws;
the remaining judgement is a person sizing the window to 1280×704, pressing F4,
F3 and F7, and reading a siege log through it. **That has not been done.**

### Brewing + archive — implemented, Phase 0 item 9

The slice's two domains, and the first time the game has a world rather than a
parser with nothing to parse against.

| Question | Decision |
|---|---|
| **The tree is ECS** | Rule 1 makes the world model ECS throughout, and a second representation would be a thing `Scene`, durations and Phase 1's script referents all have to bridge. Nodes carry a **stable `NodeId`**, not an `Entity`: §8 resolves bound references by stable id and writes them into script files as `north_gate#7f2a`, and `Entity` is a generational index that means nothing across a save |
| **You can only name what is where you are** | §7 makes the tree the tower and navigation diegetic, so `decoct clarity` works in `/tower/laboratory` and nowhere else. Places stay nameable everywhere — gating movement on being somewhere would be a lock whose key is behind it. This is the base state §19's **pane addressing** later relaxes in Phase 8: acting at a distance has to *become* possible |
| **The scene walks `Children`, never a query** | §6 breaks scoring ties by registration order, so registration order *is* the parse. Archetype order is not insertion order and an entity moves tables whenever a component is added — so starting a brew would have reordered the noun list, flipped a tie, and changed what a phrase resolves to. Replay would diverge with no test seeing it |
| **Rebuilt per tick, in its own schedule pass** | On-change is a cache-invalidation bug waiting for the first system that mutates without setting a marker. And sharing a pass with whatever a frontend adds through `with_schedule` is an ambiguity rather than an ordering — Bevy's topsort was in fact running the caller's systems first |
| **One production slot, tower-wide** | §11.5 opens at multiplex capacity **1** and §9's fourth invariant reserves it for the action's whole duration, so brewing occupies the tower and you are not also deciphering. A slot per domain would delete the trade the focus track is built on *and* be more code — a counter per domain where the design needs one |
| **Work is an interval, not a countdown** | §8 wants in-flight actions serialisable with start and completion ticks. Comparing against the clock is idempotent, survives `meditate` running hundreds of ticks inside one `step`, and makes the progress fraction a pure function of the tick |
| **Names are not prose** | §19 already set the line: the parser's own tables emit facts, never sentences. `build.rs` names things and contains no sentence. The moment a fragment needs deciphered *text*, that text belongs in Phase 1's content file — and needing it is the signal Phase 1 has been imported early |

**Durations are deliberately short, and this is the number to attack.** §11.5 puts
production actions at 3–10 minutes, which at 1 Hz is 180–600 ticks. §15's gate is
a **fifteen-minute** scripted scenario, so one design-faithful brew would consume
a fifth to two thirds of a tester's whole session — measuring their patience
rather than the parser. Phase 0 therefore sits at §11.5's routine end:

| Action | Ticks | Why |
|---|---|---|
| `decoct` | 20 | Long enough that the slot is felt, short enough to fit a scenario twice |
| `research` | 12 | §10 makes archive the domain played most and returned to between other work |

These are the **first constants `orbs-balance` will sweep** (§11.5, Phase 1). They
are placeholders with a reason, not measurements.

**Numbered candidate selection shipped with the domains, not after them.** Adding
nouns is what makes ambiguity reachable — with one noun in the scene it was
nearly impossible — and §15's gate weighs *"zero dead ends"* above the raw
resolution rate. A numbered prompt that ignores numbers is the worst possible
shape for that metric. An out-of-range answer leaves the question standing;
anything else walks away from it free, because §6 forbids a modal prompt.

**`Record::marker` derives from all three channels.** Kind alone was a lie: a
refusal is a `Completion` — the work *concluded* — so "you cannot brew and
decipher at once" was reported with a success tick until role was folded in.

### Dev ergonomics — measured, and mostly declined

§15 listed `bevy/dynamic_linking` and a fast linker as a Phase 0 item on the
premise that *"Bevy's compile time is the main friction."* Measured on the
development machine, it is not: a rebuild after touching a leaf file is **0.7 s**
and the entire gate — build, clippy, test, doc — is about **nine seconds**.

| Question | Decision |
|---|---|
| `bevy/dynamic_linking` | **Available, off by default**, behind `--features fast-compile`. It saves ~0.1 s on a 0.7–1.1 s loop, which is inside the noise, against a flag to remember and a dev binary laid out differently from the one that ships. Kept rather than deleted because the measurement is machine-specific and Linux or Windows may answer differently |
| A fast linker | **Declined.** None is installed, and Apple's `ld` has been substantially rewritten since the advice was current. Installing LLVM to obtain `lld` would cost more disk than it saves in seconds |
| Why the item existed | It is standard Bevy advice, and standard Bevy advice is written for the slowest machine that might read it. That is the right way to write advice and the wrong way to accept it |

**The rule this establishes.** Performance items are a measurement before they
are a task. This one took four minutes to measure and would have taken an hour
to implement — and implementing it would have left permanent complexity paying
for a tenth of a second. Where a roadmap item asserts a cost, check the cost
first; the item may already be done, or may never have been needed.

That is the same discipline §4's cell-renderer decision used — 227 µs measured
before choosing single-mesh over a cell-index texture — applied to the build
rather than to the frame.

### The retroactive playability pass — done

Every subsystem built before the gate existed is now reachable from the running
game. What the pass cost was small; what it found was not.

| Was unreachable | Now |
|---|---|
| Determinism spine | `meditate <n>` passes time; `status` reports tick and seed; both live in the pane furniture |
| Fidelity tiers | tier and grid on screen — drag the window and §9's table moves |
| Frame boundary, layout | two panes from a real `ScreenLayout`; `F4` switches Deep ↔ Wide |
| `Speech` linear stream | `F5` shows the session pane as a reader hears it |
| CP437 repertoire | the prompt refuses what it cannot draw |
| Parse instrumentation | every line traced with its losing candidates; `F6` exports TSV |
| `sift`, records | `sift <pattern> orb.log` — the scrollback *is* the log (§3), so it has a name |
| Eldritch, tampered | `F7` cycles the register; all three faces draw, and §3's exemption holds where you can see it |

**Two defects surfaced that no test could have caught**, both by building a
surface and looking at it:

- `Fidelity::deep` was built, tested, documented — and never called. Deep focus
  was therefore unreachable at every window size, so the layout could never host
  a second pane and §9's multiplexing was dead code. Its unit tests passed
  throughout; they proved the function did what it was written to do.
- The linear view's first shape read the live frame's stream and showed a single
  empty utterance, because the mirror had already replaced the pane it was
  meant to describe.

Neither is a coding error. Both are the specific failure §15 names: code that is
correct, tested, and *not called*.

**Six rows are deliberately still open**, and finding five of them is the more
useful result. The first status table was written by hand from the roadmap's item
list, and confirming it against the code — sweeping the public API of both
library crates for anything the shipping path never calls — turned up subsystems
no roadmap item had ever named:

| Built and unreachable | Gated by |
|---|---|
| `Painter::progress` — §14 names progress bars specifically | brewing + archive: nothing has a duration yet |
| `ScreenLayout::sidebar` — §9's minimised panes | brewing + archive: nothing to minimise with two panes |
| 3- and 4-pane tiling | brewing + archive |
| `Sim::submissions` — the replay log is written and never read | Phase 1: needs a replay command |
| `Verb::is_destructive` — §7's destruction guard | brewing + archive: `purge` has nothing to destroy |
| Per-subsystem RNG streams | Phase 8: nothing rolls yet |

Every one is gated by content that does not exist rather than by the pass having
been careless, so none of them changes what the pass should have done. What it
changes is **how the table is maintained**: derived from the API, not recalled
from the plan. A checklist assembled from memory measures the memory, which is
the same error one layer up as §15's original layer-ordered item list.

### Deep focus was unreachable — found by the retroactive pass

`Fidelity::deep` was built in the Frame-boundary item, tested, documented as
*"one step finer — the tier Deep-focus multiplexing engages (§9)"* — and **never
called by the game**. The consequence was not cosmetic: Deep focus could not be
reached at any window size, so the layout never had the cells for a second pane.

The mechanism is easy to miss and load-bearing. `Fidelity::tier_one` returns the
largest scale that still fits 80×22, so **a bigger window buys a bigger glyph,
not more cells** — the grid sits near 80×22 at every window size by design (§4).
Multiplexing needs cells, and §9 says where they come from: *"fidelity rises one
step; every pane is drawn at full size in a grid."* Deep focus **is** the tier
step. Without it, `DEEP_FOCUS_FLOOR` at 100×28 was unreachable on any display.

| Question | Decision |
|---|---|
| **Where the switch lives** | `F4`. §9 requires the mode be overridable *"at any time, including mid-siege"*, so a heuristic the player has to fight would be wrong. Window size only picks the default |
| **What survives a resize** | The player's choice. Re-deriving the mode on every window change would silently undo it |
| **At the finest scale** | `deep()` is `None` — there is no smaller whole-pixel step — so Deep focus is unavailable and Wide serves instead, which is what its own documentation already said |
| **Panes at the floor** | Two panes only above `DEEP_FOCUS_FLOOR`. At 80×22 a secondary pane is a four-row strip: a border, a header and one row. Below the floor the readings live in the session's border title instead, so nothing is lost |

Found by building the telemetry pane and looking at the result, not by any test.
The unit tests for `deep()` passed throughout — they proved the function did what
it was written to do, which is exactly the limit §15 records.

### The playability gate — added to every phase

**Supersedes: the original per-phase item lists, which had no player-facing
completion criterion at the step level.** §15 now gates every work item in every
phase on being reachable from the running game.

| Question | Decision |
|---|---|
| **What went wrong** | Phase 0's first six items were completed in layer order — determinism spine, Frame boundary, parser, cell renderer, CRT port, record model — each tested, reviewed, and correct. The result was **~10,000 lines of Rust that the game binary called under 40% of.** `cargo run -p orbs` answered `Escape`, `F2`, `F3` and `F12`, and no letter key at all. The parser was unreachable from the frontend; the record model had never been drawn by the renderer built to draw it |
| **Whose error** | Not a judgment call made and lost — **the lists themselves were the defect.** Every item was phrased "build this subsystem", so following one faithfully could not produce something playable until it ended. A correct process executed against a badly ordered list |
| **Why tests did not catch it** | They were not capable of it. Tests prove code does what it was written to do; they cannot prove it is the code worth writing. Untouched code is *unshaped* code — `report()` needed an `input` parameter no test wanted and no design document predicted, and it surfaced from an adversarial review rather than from a caller. That is luck, not method |
| **Why this is not a Phase 0 rule** | Later phases need it more, not less. Phase 1 asks whether a player *feels clever*; Phase 8 whether a siege is *tense*; Phase 10 whether a non-terminal player reaches hour two unaided. None of those is assertable, all of them are felt, and the phase exit criteria were already written in exactly those terms — the step-level lists simply did not inherit it |
| **The rule** | **No work item in any phase is complete until a person can reach it from the running game.** Every item carries a *See it* line naming the keystrokes. Same standing as the build gate in CLAUDE.md |
| **Retroactive gating is a work item** | Everything already built without a gate gets one, immediately after the prompt and before any new Phase 0 work. Not a cleanup task and not optional: code the game does not call has been asserted, not verified |
| **Where a gate is not yet possible** | Some built code has no honest player surface until later content exists — per-subsystem RNG streams cannot be *seen* until something rolls against them, which is aberrations in Phase 8. Those items name the phase that gates them rather than inventing a debug affordance nobody will maintain |
| **The counter-example worth copying** | The CRT was the one item player-gated on arrival: boot the game, press F3. It flashed, the flashing was obvious in ten seconds, and **two wrong diagnoses were falsified by looking rather than by reasoning.** No test in the suite would have caught it and none could have been written to |
| **What does not change** | The slice is still brewing + archive, the numeric gate is unchanged, no phase boundary moves, and no scope is added. This is an ordering and acceptance-criteria change |

### The tube was eating a column — found by looking

The first screenshot of the working prompt read `rbs:~$`. The barrel warp pushes
pixels outward by `1 + strength * dot(centred, centred)` — largest at the corners
— and the input line sits in the bottom-left corner, so its first glyph was
warped past the edge and masked away.

**A rule 2 violation, not a cosmetic one.** A frontend may add enrichment the
other cannot reproduce *provided it carries no information absent from the
Frame*. A tube that swallows a column carries less, and the terminal build would
have been playing a different game.

It took four attempts. The three failures are worth recording because every one
of them looked right, and two were *committed to a screenshot* that appeared to
prove it. The last of them was reported by the player, not by the author.

| Attempt | Outcome |
|---|---|
| **Shrink the pixel budget before choosing a tier** — a "safe area" of 92% | **Wrong, and shipped for an hour.** `Fidelity::tier_one` returns the *largest* scale that still fits 80×22, so the floor has zero headroom and trimming pixels drops a whole tier: 1280×720 fell from 2× to **1×**, meaning 8×16 physical-pixel glyphs. Smaller text everywhere is a worse defect than a lost column, and it put the game off §9's tier table. The test written alongside it asserted only pixel margins and passed happily |
| **Normalise the warp by the corner's factor**, so the curved image exactly fills the screen | **Wrong, and it made things worse elsewhere.** A radial warp cannot map a rectangle onto a rectangle: whichever boundary point is made exact, the rest move the other way. Making the corners exact pulled the **edge midpoints in by 30.5 pixels** — a whole cell at tier 4 — so the pane's left and right borders were being cut off at mid-height. Traded a clipped glyph in one corner for a clipped column down both sides |
| **Scale the picture *outward* instead of normalising it inward** — a single `OVERSCAN` constant | **The fix.** Normalising can only ever choose which boundary point is exact and which loses; scaling outward makes every one of them safe at once. The screen then runs out of texture near the edges and the mask paints the dark room, which costs nothing and is what a tube looks like anyway. The constant is also the margin control: `0.0` is the minimum safe value, `0.035` gives a 73px surround at 2560 wide. Tunable upward as taste, never downward — below zero it starts cutting cells again |
| **A one-cell gutter on the input line**, charged to `ScreenLayout` | Kept. No longer load-bearing against hard clipping, but the bottom corners are where the warp, the vignette and the rounded bezel all compound, and content sitting flush in one is legible only by luck. *Where content may safely go* is a layout decision. `orbs-tui` pays a cell it does not need — an invisible gutter in a terminal beats divergent layouts between frontends |

Two lessons, both already written down elsewhere and both re-learned here:

- **The symptom leaving a measurement is not a diagnosis.** The safe area made
  the screenshot look right while silently halving the glyph size. This is the
  same error as blaming MSAA for the CRT flashing.
- **Look at the pixels, not at the thumbnail.** After the second attempt the
  downscaled screenshot looked fixed. Cropping the corner and scaling it up
  showed the stroke still clipped. Verifying at the resolution the defect lives
  at is part of looking, not an extra step.
- **Compute the constant; do not reason about it.** Attempt three was chosen
  because normalising for the corners *sounds* like the conservative option, and
  it is the one that cuts content. Four lines of arithmetic printing what each
  divisor costs at each boundary settled it immediately, and would have settled
  it three attempts earlier.

None of the three was reachable by a test. The Frame was correct throughout, the
renderer was correct, and the loss happened in the post-process — which is the
third time the tube has produced a defect only a human eye finds.

### The prompt — settled before implementation

Decided from an independent review of the implementation plan, which
compile-verified every API claim against the pinned Bevy rather than recalling
it. Four of these are load-bearing.

| Question | Decision |
|---|---|
| **Where the scrollback lives** | **In `orbs-sim`, not the frontend.** The record stream *is* the log — the scrollback, the file a player `peruse`s, the pipe source, and the harness transcript are one stream read four ways. A frontend-owned scrollback means the very next item has to move it, and until then there are two streams that can disagree. This is the mistake the record model exists to prevent, one layer up |
| **A second entry point: `Sim::submit()`** | Rule 3 says the sim *advances* only through `step()`. `submit(&mut self, line)` does not advance world time: it pushes the input record, resolves against the sim's own `Scene`, reports the echo, and **queues the `Intent` for the next `step()`**. Effects stay tick-aligned; the echo does not. Recorded here rather than left to be inferred from a diff |
| **Why the echo cannot wait for the tick** | Ticks are 1 Hz. Resolving inside `step()` would put up to a full second between Enter and the echo, and a terminal that takes a second to answer reads as broken. §6 makes the echo the teaching mechanism, so it must be immediate |
| **Determinism is preserved, and why** | Bevy runs `FixedUpdate` **before** `Update` in a frame, so a line submitted in frame F always resolves against world state as of the last completed tick in F. Replay therefore needs only `(seed, [(tick, line)])` — which is also what §6's command-anchored `undo` will need, so the pairing is recorded from the start |
| **The starting world is empty, deliberately** | `Scene::default()`. A hardcoded scene would be a second source of truth for nouns that brewing + archive then deletes, and it would mask the property that makes this a *world*-grounded parser rather than a command parser. §6's "never a bare error" already holds with nothing in the world: `peruse feed.log` answers `Incomplete{missing: File}` — *"peruse needs a file"* — rather than dead-ending |
| **`Speech` is the live screen; `Records` is the history** | Painting only the scrollback tail keeps a reader and a sighted player exactly level, which is rule 2 in both directions. Scrollback history is `Records`'s job and always was. Noted so that nobody later "fixes" `Speech` into a scrollback and breaks parity to do it |
| **No scrollback trimming yet** | Nothing in Phase 0 emits a record without a keystroke behind it; unattended logging arrives with the script engine in Phase 1. A `retain_last(n)` would have to re-base every stored index in the one type four consumers read — a speculative API of exactly the unshaped kind §19 already warns about. If a cap is ever needed, rebuild through the public builder rather than rewriting indices |

**`RecordView::lines()` is not sufficient for this surface.** Drawn through it,
`meditate` renders as `meditate count` and an unresolved input renders as four
bare words in a column — neither of which is the prompt §6 describes, and the
`Outcome` annotation added in the corrections pass above is unreachable by the
only view that would use it. The prompt therefore needs an **outcome-aware line
view in `orbs-render`**: a marker glyph from the CP437 repertoire plus an
intensity, both derived from the record. Glyphs are not prose, so this does not
touch rule 6; and it belongs in `orbs-render` rather than the frontend because
deciding *what appears* is not a frontend's decision (rule 2).

**Selecting an ambiguous candidate is out of scope for this item.** §6's numbered
prompt (`[1/2]: _`) needs input handling that resolves a number against a pending
list rather than against the verb vocabulary. The marker work above must still
make candidates visually distinct from suggestions, or the first ambiguous input
a tester meets looks like the parser malfunctioning.

### Structured-record output model — implemented, Phase 0 item 6

§7 makes this binding on §13 and lists what depends on it: pipes, `sift`, the
eldritch renderer, screen-reader linearisation, and the test harness. All five
now read one stream, and none of them is privileged.

| Question | Decision |
|---|---|
| **Which crate owns the model** | **`orbs-render`.** A record is the *interface* between the two crates, and an interface belongs to whichever side both can depend on. `orbs-render` has no dependencies at all, so `orbs-sim` takes it on for milliseconds of compile time; the reverse would drag the world model, parser, and script engine into the crate whose tests must stay pure presentation. It also already owns the vocabulary a record must carry — `Role`, `Presentation`, `UtteranceKind` |
| **The cost of that direction** | `orbs-sim` can now *name* `Painter`. Rule 2 forbids it from using one, so `crates/orbs-sim/tests/boundaries.rs` reads the crate's own source and fails the build on `Painter`, `Frame`, `ScreenLayout`, `Fidelity`, `async fn`, `.await`, `tokio`, or `bevy` the engine. Same instrument §16 already specifies for keeping `unscii-16-full` out of `assets/` |
| **Numbers stay numbers** | A `Value` is `Text`, `Count`, or `Tick` — never a pre-rendered string. A quantity rendered at emit time cannot be right-aligned by a table, compared by a future `sift --above`, or summed by the harness without being parsed back out of its own presentation, which is the exact inversion rule 4 exists to prevent. Right-aligned numeric columns fall out of this for free |
| **Field names are a closed enum** | Ten names, each with a lower-case label. §6 already fuzzy-resolves player input against closed vocabularies, and a field name becomes one the moment `sift` can be pointed at a column. A closed set also makes a missed case a compile error in every view |
| **§3's corruption exemption is enforced, not documented** | `Record::presentation()` is the *only* presentation accessor and returns the value already filtered through `RecordKind::allows`. There is no unfiltered path, so no call site can forget. Asking for `Eldritch` on a log line is allowed and silently answered `Plain` |
| **The exemption is asymmetric** | `Eldritch` is refused on the three diagnostic surfaces §3 names; `Tampered` is refused nowhere. §3's clause is *"trustworthy as renderings, even when their contents are not"* — the tonal register must not make a player doubt their eyes, but §8.1's structural tell **is** the sabotage on that surface and suppressing it would delete the signal. This asymmetry is what §3 buys by requiring the two vocabularies be disjoint |
| **Linearised tables are correct by construction** | `Record::speak` emits `label: value` from the record's own labels, so §14's *"a reader must never have to reconstruct columns from spacing"* cannot be violated by a view forgetting to pass headers. A single-field record speaks as itself, so prose reads as prose |
| **A missing field draws as a gap** | §8.1 names malformed record boundaries as a structural sabotage signature. No special case implements it: a record short a field simply leaves a hole in its row |
| **`sift` matches field values only** | Never padding, never a truncated tail, never eldritch corruption, and never the authored spoken variant. §7 calls records *"the only model that survives the eldritch renderer corrupting output"* — search over rendered text would go blind exactly when threat is highest. A test proves `sift shade` still finds `nightshade` in a pane too narrow to draw it |
| **`ls` is not a diagnostic surface** | §3 names three — script listings, schedule listings, log output — and only those three are exempt here. Extending the exemption to directory listings is a design decision for this log, not an inference to make in a match arm |
| **The parser is the first producer** | Every arm of `Resolution` reduces to the same record: a canonical echo. `Resolved` emits one, `Ambiguous` one per tied reading (§6's numbered prompt is a list of echoes), `Incomplete` one plus the category of the empty slot, `Unresolved` one per suggestion. Zero authored prose crosses into Rust — rule 6 and §12's second mitigation put the sentence around these facts in a content file in Phase 1 |

**Open question, deliberately not decided.** §3 exempts log, script, and schedule
listings from eldritch corruption but says nothing about the **echo**. Corrupting
it would damage §6's pedagogy — the echo is how players learn the canonical form
— but adding a fourth exempt surface is a design change, so `RecordKind::Echo` is
currently corruptible. Decide before the eldritch renderer ships in Phase 8.

#### Corrections — post-review

The container was built well and then barely used. Every finding was a variation
on one mistake: **facts that a view needs were left implicit in text.**

| Defect | Fix |
|---|---|
| `Resolved`, `Ambiguous` and `Unresolved` all emitted a bare `Echo{Message}`, byte-identical. No view could tell a command that will run from a total parse failure, or a **selectable** numbered prompt (§6 has the player answer it with a number) from a suggestion list that is not selectable. A string in a box — the exact thing records exist to replace | `FieldName::Outcome` carries `resolved` / `forced` / `incomplete` / `candidate` / `unresolved` / `suggestion`. Absence no longer means anything, so a clear echo is positively marked rather than inferred from a missing flag |
| `speak()` decided whether to say labels by **counting fields**, so adding a machine flag flipped prose into recital: a forced echo spoke *"message: survey, state: forced"* | The **kind** decides. §14 requires labels specifically for tables — "a reader must never have to reconstruct columns from spacing" — so `UtteranceKind::TableRow` speaks `label: value` and everything else speaks values alone. The field-count special case is gone |
| Nothing distinguished a field written for a player from one written for a machine, which is what made the defect above possible at all | `FieldName::is_annotation()` — the same content/structure split `Painter` already draws one layer up. `speak()` and the line view use content; `fields()`, `field()` and `sift` see everything, because a view that names an annotation as a column has asked for it |
| The line view joined **every** field, so an echo would have drawn as `"resolved survey"` — an internal token on screen for a player to read. Found while testing the fix above, not by the review | Line views default to content. Only an explicitly named column can draw an annotation |
| `Unresolved` with no suggestions emitted **zero records**, contradicting §6's "never a bare error" — silence is worse than one | `report()` takes the raw input and always emits at least one record. `Resolution::Unresolved` does not carry the input, and the orb cannot say *"I do not know that word"* without naming the word; an empty `Ambiguous` (a parser defect) falls to the same floor rather than inventing a seventh outcome |
| The `screens` example bordered all three panes before drawing any content, so the linear stream was three headings then fourteen unattributed rows — the `sift` results indistinguishable from the listing they were filtered out of | Each pane borders immediately before its own content. §14's parity is an **ordering** property, not only a completeness one, and the example's other screens already did this |

### CRT port — implemented, Phase 0 item 5

§4 budgeted "2,638 lines of surrounding Rust across 15 files plus a 242-line
shader" and warned that "the WGSL ports; the render-graph Rust is Bevy's most
volatile surface". Both halves of that were right, and the second more than
expected.

| Question | Outcome |
|---|---|
| The shader | Ported nearly intact — barrel, scanlines, aperture grille, vignette, chromatic aberration, flicker, rounded corners, phosphor glow, desaturation, flash |
| The surrounding Rust | **Did not survive at all, and did not need to.** Bevy 0.19 removed `bevy_render::render_graph` outright; a post-process is now an ordinary system in the `Core2d` schedule. `ViewNode`, `RenderGraphContext`, and `RenderLabel` have no 0.19 equivalent. The port is ~300 lines rather than ~2,100 |
| Scanlines and grille | **Cell-derived, as §9 requires.** One scanline every eighth of a cell height, one grille stripe per glyph pixel (cell width ÷ 8). Both are integer fractions of a cell, so the pattern lands identically inside every glyph at every fidelity tier instead of beating against the stems. The 1080-line reference the original hardcoded is gone |
| Dropped | The 16:9 letterbox (the grid fills the window) and the channel-change effect (not in §4's list, and a television retuning is the wrong metaphor for a scrying orb) |
| Pipeline format | Keyed on the **view's** texture format rather than a default. 0.19 deprecated `TextureFormat::bevy_default` precisely because a view may or may not be HDR, and guessing is a draw-time validation error rather than a compile error |
| Disableable | `CrtSettings::OFF` is one value and every field reaches zero, per §14. `PEAK_THREAT` — §4's "maximum flicker and vignette pulse" — is reachable on F3, because the legibility test needs the worst case to exist rather than merely be described |
| Not yet wired | Vignette pulse on threat, flash on breach, desaturation on failure. The uniform carries all three; there is no threat system to drive them |

**Two bugs worth recording**, both found by measuring frames rather than by
reading code:

- **The CRT flashed on and off, because the pass had no system set.** 0.19's
  `Core2dSystems` chains `Prepass → MainPass → EarlyPostProcess → PostProcess`,
  and `upscaling` runs `.after(PostProcess)`. Registering the pass with only
  `.after(tonemapping)` creates one ordering edge and leaves it **unordered
  against `main_pass_2d` and against `upscaling`** — so it landed at a different
  point every frame: sometimes curving the grid, sometimes running before the
  grid was drawn, sometimes after the blit to the swapchain. The fix is
  `.in_set(Core2dSystems::PostProcess)`, which is how every built-in effect
  registers (`bloom`, `tonemapping`, `msaa_writeback`).

  **Set membership is not decoration in this schedule.** An ordering constraint
  against one system says nothing about the rest, and the render schedule will
  happily run an unconstrained system anywhere.

  MSAA was blamed first and was not the cause. It stays off on its own merits —
  every edge is a bitmap glyph on an integer grid, so multisampling can only
  soften the font — but disabling it merely changed the odds enough to look
  like a fix under screenshot sampling. Worth recording as a method failure:
  *the symptom disappearing from a measurement is not a diagnosis.*
- **The flicker term was a 19 Hz strobe.** The original modulated whole-screen
  brightness by `sin(time * 120.0)` and commented it "60Hz-ish". It is neither:
  120 rad/s is **19.1 Hz**, in the middle of the 3–30 Hz band that provokes
  photosensitive reactions, on a product that already ships a health warning
  (§14). There is no correct frequency to substitute either — anything fast
  enough to pass for mains hum is above a 60 Hz display's Nyquist limit and
  aliases into noise. The hum is now **spatial**: a faint band rolling down the
  tube at 0.22 Hz, which is what a camera actually catches off a CRT and is far
  below the photosensitive floor.

And one that only a screenshot finds:
the phosphor taps were placed a third of a cell apart, which at a 64-pixel cell
is a 22-pixel offset — every line of text rendered a visible duplicate below
itself. That is a double exposure, not a glow. The taps are now one glyph pixel
out.

**An interaction the legibility test must account for:** the palette's WCAG
contrast ratios (§19, cell renderer) are computed on the *pre-CRT* image. The
tube multiplies everything down — scanlines, grille, and vignette each darken —
so the effective contrast on screen is lower than the palette tests assert. The
worst-case test is what settles whether the defaults survive that.

### Cell renderer — implemented, Phase 0 item 4

| Question | Decision |
|---|---|
| Which of §4's two options | **Single mesh, one quad per visible cell.** Measured at **227 µs for the worst case** — a full 160×45 grid, 7200 quads, rebuilt from scratch in release. That is 1.4% of a 60 Hz frame, so the cell-index-texture alternative is not needed and the simpler path wins |
| Draw calls | **One, at any grid size.** The whole screen is a single mesh with a single material |
| No custom shader | Each vertex carries its colour and Bevy's stock `ColorMaterial` multiplies the sampled texel by it. The atlas stores **white RGB with coverage in alpha**, so `(1,1,1,coverage) × (r,g,b,1)` is "this glyph in this cell's colour" with no WGSL of ours. An `R8Unorm` atlas would have sampled as `(coverage,0,0,1)` and tinted the screen red |
| Blank cells | **Emit nothing.** A space is the commonest glyph on screen by a wide margin, and a quad sampling a fully transparent texel is pure cost |
| Camera | `ScalingMode::Fixed` at the window's **physical** size, so one world unit is one physical pixel. Bevy's default 2D projection works in logical pixels, which on a 2× display stretches every glyph across four physical pixels — a blurred bitmap font, which §4 names as the thing legibility cannot survive |
| Palette | Four themes, `(Role, Intensity) → Color`. `Presentation` deliberately has **no** entry: it selects a face in the atlas, and a tonal register that existed only as a hue would be exactly what §14 forbids |
| Colours are solved, not chosen | The first pass was picked by eye and **failed its own accessibility tests** — muted violet's cost and success accents were 1.19:1 apart, which is the same colour in greyscale. The shipped values satisfy every constraint the tests assert: body ≥4.5:1 on background, dim ≥3:1, a monotonic intensity ramp, and every accent pair ≥1.25:1 from each other and from body text |
| Verifying it draws | `ORBS_CAPTURE=1 cargo run -p orbs` saves a screenshot after 30 frames. A renderer that cannot be checked without a human at the keyboard is one nobody checks |

### Naming pass — Phase 0, done

Run against the implemented vocabulary rather than by eye, which is what turned
up defects the design table had carried since draft 4.

| Finding | Decision |
|---|---|
| **`decoct` and `decant` collide.** Two edits apart, scoring 667 against a 600 threshold, and they are the two core verbs of brewing — a Phase 0 domain. In a siege §6 forbids a blocking prompt, so a near-typo would be resolved by the parser's best guess: brewing when the player meant to collect | **`decant` → `siphon`.** Alchemically exact, six characters, zero collisions. `decant` is **kept as a plain synonym** — releasing it would be worse than the collision, because an unclaimed `decant` resolves to `decoct` |
| **`dec` prefixed three verbs** — `decoct`, `decant`, `decipher` — so the natural abbreviation for the brewing domain meant three different things | **`decipher` → `research`.** Also clears a length violation. No three-character prefix reaches more than one **canonical** name. Across *synonyms* `dec` still reaches three verbs, because the old words are deliberately kept; that prompts, which is the right answer for a genuinely ambiguous abbreviation. `aut` and `ins` are ambiguous for the same reason. All three are pinned by test |
| **Four canonical names exceeded the ≤7 rule**: `grimoire`, `meditate`, `decipher`, `inscribe` | **`inscribe` → `scribe`** (same root, same meaning, two characters shorter) and `decipher` → `research` as above. **`grimoire` and `meditate` are kept**, and the ceiling is codified at **8**: they are the two most in-world names in the set, abbreviation covers the typing cost, and §6.1 already wrote the rule as "ideally" |
| **Seven cross-verb synonym collisions.** `find`/`bind` at 750, `make`/`take` at 750, `decode`/`decoct` at 667, and others | **All kept and claimed.** Dropping them was the pass's own worst mistake and was caught in review: a released word does not stop resolving. Unclaimed, `find` resolved to `bind`, `take` and `decode` to `decoct`, and `write` — deleted by accident — to `meditate`, every one with `Clear` confidence and no prompt. `find`/`bind` at 750 is *worse* than the 667 that justified renaming a canonical verb |
| **An exact verb match could lose to an approximate one.** `take clarity` resolved to `decoct clarity` — brewing — because `take` reaches `make` at 750 and `clarity` is an essence, even though `take` *names* siphon at 1000 | Ranking is now **exactness first, then score**. A word the player actually typed outranks one that merely resembles it; argument fit still decides between readings of equal exactness |
| Result | Canonical collisions **1 → 0**. Canonical three-character prefix ambiguity **1 → 0**. Seven synonym collisions remain and are pinned — that is the correct number, because the fix for a collision is to *claim* both spellings, not to release one |

The rules are enforced by `crates/orbs-sim/tests/naming.rs`. The load-bearing
one is `every_phrase_reaches_the_verb_that_claims_it`: every phrase the
vocabulary claims must, given a fitting argument, reach the verb that claims it.

That test replaced one that compared the synonym list against a set built from
the same synonym list, and was therefore always true. It passed while four words
resolved to the wrong verb — which is the whole argument for review: the pass
stated the right principle in §19 and then broke it four times in the same
commit.

### Project licence — GPL-3.0-or-later

| Question | Decision |
|---|---|
| Licence | **GPL-3.0-or-later.** Declared in the workspace manifest, full text at the repository root |
| Dependency tree | **Audited and compatible.** Every crate in the tree is MIT, Apache-2.0, BSD-2/3, Zlib, ISC, Unicode-3.0, CC0, 0BSD, MIT-0, or Unlicense; all are FSF-listed as GPL-compatible. The only copyleft entry, `r-efi`, offers MIT/Apache-2.0 alternatives and does not build on our targets |
| One-way constraint | Apache-2.0 is compatible with GPL-**3**.0 but **not** GPL-2.0. Bevy is `MIT OR Apache-2.0`, so 3.0-or-later works and 2.0 would not have |
| Assets | Both font licences survive the combination. CC0 imposes nothing; Spleen's BSD-2 notice requirement **persists** and still has to reach the shipped build (Phase 11) |
| Commercial release | Unaffected — §15's demo-then-1.0 Steam posture stands. Selling GPL software is permitted; the obligation is to offer source to those you distribute binaries to |

### Parser corrections — post-review

Found by an adversarial review of every commit, all confirmed by running the code.

| Defect | Fix |
|---|---|
| The missing-argument prompt **rebuilt the argument list from scratch**, discarding slots that had already resolved. `sift march nowhere.log` offered `sift feed.log` — the pattern gone, the file sitting in the pattern's position, which any consumer zipping against the signature reads as the search term | `fill` now returns one entry **per signature slot**, positionally, and reports *which index* is empty. The enumeration writes the candidate into that slot and leaves the rest alone |
| The enumeration filtered with `noun.kind != kind`, which **never honoured `NounKind::Any`** and could never match the synthetic `Pattern`/`Count` kinds. `purge`, `verify`, `rm`, `meditate`, `sift` and their synonyms all fell through to `Unresolved` — the orb answered *"I do not know that word"* and then suggested the exact word typed | `fillers()` honours `Any` and returns nothing for free-text kinds. New `Resolution::Incomplete` names what the slot wants instead of pretending the verb was unknown |
| Unbounded input: 100k characters measured at **605 ms** in release, on a path §6 requires to be sub-millisecond and non-frame-blocking. The score arithmetic also overflowed `u32` | Input capped at 512 characters and 32 words; `fuzzy` refuses words over 64 characters; the score multiply saturates |
| `NounKind::Pattern` was built from the **lowercased, punctuation-trimmed** token stream, so `sift ERROR feed.log` searched for `error` — contradicting its own documentation | Tokens carry `raw` and `matching` forms together as a `Word`, so filtering one cannot desynchronise it from the other. Quoted runs (`sift "march north" feed.log`) now hold together |

**One typeface per `Presentation`.** `Plain` is `unscii-16`, `Eldritch` is
`unscii-8-fantasy`, `Tampered` is `unscii-8-mcr` — three faces of one public-domain
family, sharing metrics and repertoire so a face swap can never move a cell.

| Question | Decision |
|---|---|
| Why | The renderer already carries `Presentation` in every `Cell` and had nothing to *do* with it. Three faces give the tonal register and the sabotage tell a real visual channel for the cost of two extra atlases |
| `Plain` gets the native 8×16 | `unscii-16` is drawn at 8×16; the other two are 8×8 row-doubled. Plain carries the ~88k-word prose budget (§12) and §4 justifies the 1:2 cell partly on prose legibility, so the full-resolution face goes where nearly all the reading happens. The special registers appear in short bursts and can afford the doubling |
| Fits rule 2 | The Frame carries the tag; the frontend decides only how the cell is drawn. `orbs-tui` renders all three identically and loses nothing, exactly as §8.1 already allows for the script-text tell |
| Accessibility | No regression. Eldritch already carries an authored linear variant (§3) and tampering is `verify`-detectable on every frontend (§8.1). The face adds nothing a reader needs |
| Licence | Public domain / CC0. **`unscii-16-full` merges GPL Unifont** and is one word away in name; a test fails the build if it ever appears in `assets/`. Since the project itself is GPL-3.0-or-later this is a *provenance* guard rather than a licence conflict — `PROVENANCE.md` records exactly which files ship, with checksums, and Unifont carries notice obligations nothing here tracks |

**This amends §3's disjointness rule and the amendment is load-bearing.** §3
assigns *glyph substitution* to sabotage and deliberately withholds a glyph
channel from eldritch, because *"if both used glyph corruption and broken
alignment, the tonal system would jam the diagnostic system at peak difficulty."*
Giving eldritch a face hands it exactly that channel.

It survives because §3 *also* makes script listings, schedule listings, and log
output **corruption-exempt** — the renderer never applies eldritch presentation to
them, and those are precisely the surfaces sabotage tells live on. The two can
therefore never appear on the same line, and the jam cannot occur. **Disjointness
is now preserved by surface separation rather than by channel separation.**

That reasoning holds only while the corruption-exempt rule holds. **If a future
change ever lets eldritch presentation reach a diagnostic surface, this amendment
must be reverted, not patched** — at that point the tonal system really would be
competing with the diagnostic system at peak threat, which is the failure §3 was
written to prevent.

Two consequences worth remembering:

- The doubled faces carry half the vertical detail of a native 8×16. The Phase 0
  worst-case legibility test (§4) is what decides whether that is acceptable for
  the amount of eldritch and tampered text actually on screen at peak threat.
  Spleen is retained in `assets/` as the fallback.
- `unscii-16`'s box drawing does **not** match the doubled 8×8 forms — verticals
  and solid blocks agree, horizontals and dither shades do not. Harmless for
  borders, which are always `Plain`; where it shows is a tampered line containing
  box drawing, and there it supplies §8.1's "malformed record boundaries" tell for
  free.

### Font asset — Spleen 8×16, retained as fallback and gap-filler

| Question | Decision |
|---|---|
| Font | **Spleen 2.2.0, `cp437/spleen-8x16-ibm-437.bdf`.** In the repo at `assets/fonts/spleen/` with full provenance and checksums |
| Licence | **BSD-2-Clause**, and **GPL-3.0-compatible** — the FSF lists the 2-clause BSD licence as *"compatible with the GNU GPL"*. Only the 4-clause form is incompatible, and Spleen has no advertising clause. Combining it into a GPL work does **not** discharge its notice requirement |
| Obligation | Binary redistribution must reproduce the copyright notice *"in the documentation and/or other materials"*. The shipped build therefore carries a third-party notice — naturally a `grimoire licences` topic, since every screen is terminal content. **Phase 11 ship task**; the obligation only attaches on distribution |
| Why this file | **Indexed by codepage byte, 0–255, complete.** `cp437_index()` already returns exactly that index, so there is no mapping layer between Frame and atlas. The Unicode-keyed build of the same font is *not* a substitute — it is missing `∟ ► ◄` against our repertoire |
| Rejected | **int10h Px437** (most authentic, but CC BY-SA share-alike on adaptations, and its "raw bitmaps are uncopyrightable" defence is a US-centric interpretation, not settled law, on a product sold worldwide). **Terminus** (OFL's reserved-font-name clause forces a rename once we edit it). **unscii-16** (public domain and uniform 2px weight, but missing `∙ ⌂ ⌐ ☼`) |
| Custom variants | Unrestricted. §4's custom glyph variants for §8.1 sabotage tells are a local edit BSD-2 permits outright — which is why a modify-friendly licence outranked a more authentic look |
| Watch item | Stem weight is **2px vertical, 1px horizontal**. §4 names 1px stems under barrel distortion and the RGB mask as the top moiré hazard, so pane-border horizontals are the thing to check in the worst-case legibility test. Thickening them is a permitted local edit |
| Guarded by | `crates/orbs-render/tests/font.rs` — asserts the asset exists, is 8×16, is a complete codepage, and that every character `cp437_index()` accepts has a non-blank glyph behind it |

### Parser — implemented, Phase 0 item 3

| Question | Decision |
|---|---|
| Tie-breaking | **No RNG.** §6 anticipated `RngStream::Parser` for exact ties, but ranking is now a *total* order — score, then the verb's position in `Verb::ALL`, then the canonical echo. A coin flip would make replay depend on how many times the parser had been called, and "the parser must explain itself" cannot be honoured when the explanation is a coin flip. The stream stays allocated and unused; removing it would renumber the others and invalidate saves |
| Leading filler | Stripped **before** verb matching, while argument filler is stripped **after**. `to`, `for`, `of`, `do`, `it` are filler in an argument and load-bearing in a phrase (`go to`, `look for`, `get rid of`). Safe only because no synonym opens on a filler word, which a test asserts |
| Missing arguments | A required slot with nothing to fill it expands into **one candidate per plausible filler**, so §6's worked example (`brew` → a numbered list of essences) falls out of the ordinary tie machinery instead of needing a special case |
| Scoring weight | Verb counts double against argument. A confident verb with a shaky argument is the better guess, because the argument can be asked about and the verb cannot |
| What is logged | **Every candidate, on every resolution — including successes.** A command that won by four points and one that won by four hundred are the same `Resolved` and very different data. §15 says act on failure *clustering*, and a near-miss is where clustering starts |
| Export format | **TSV, one row per candidate.** No dependency, survives `grep`, pastes into a spreadsheet. Records carry no timing — wall-clock in a sim record would make two runs of one seed differ |
| Argument categories | Slots are typed (`Place`, `File`, `Essence`, …), so `attend clarity` cannot resolve. `Pattern` and `Count` never touch the world; everything else resolves against the live scene, which is what stops the parser promising a brew the sim cannot perform |
| Places answer to leaves | `attend laboratory` reaches `/tower/laboratory`. §7 says paths are places, and players say the place |

### Frame boundary — implemented, Phase 0 item 2

Decisions taken while building `orbs-render`, none of which contradict the
design above; recorded because they are load-bearing and not obvious from §13.

| Question | Decision |
|---|---|
| What a cell holds | A Unicode `char` plus a `Style` of **role / intensity / presentation**. Never a colour, never an ANSI index, never a codepage byte |
| Glyph repertoire vs encoding | The **repertoire** (which glyphs may appear at all) belongs to `orbs-render` — it is the intersection of what both frontends can draw. The **encoding** belongs to the frontend: Bevy maps the `char` to a CP437 atlas index, the TUI writes it out. A glyph outside the repertoire is substituted, not dropped, so one authoring bug cannot shift a row |
| Eldritch and sabotage are one field | `Presentation` is an **enum**, so a cell cannot be both eldritch and tampered. §3 requires the vocabularies stay disjoint; an enum makes the violation unrepresentable rather than a rule to remember |
| Linearisation is always captured | Every frame builds its `Speech` stream whether or not a reader is attached. Gating it would mean the path is exercised only by the players least able to report that it broke. Cost is a memcpy into a reused arena |
| Spoken text travels with drawn text | The paint API takes both at one call site. §3 requires every eldritch message carry an authored linear variant, and a separate "register the spoken form" call is exactly what gets forgotten on the lines that need it. A debug assertion fires on eldritch content with no variant |
| Structure is silent, content speaks | Borders, fills, and padding write cells and no speech; spans, paragraphs, and progress bars write both; `announce` writes speech and no cells. A titled border announces its title, so a pane never loses its identity in the linear stream |
| Truncation is visual only | A span clipped by a narrow pane is still recorded in full for the reader. A narrow pane is a visual constraint and must not become an informational one |
| Progress is integer-only | Meters take `(done, total)` as integers, not a float fraction. Progress is elapsed ticks against a duration (§5.0), and keeping floats out of the render path keeps a deterministic sim rendering deterministically |
| Layout is total, not fallible | A grid below the 80×22 floor lays out smaller rather than erroring. Sub-minimum grids are a normal runtime state — a window mid-drag, a terminal the user shrank — so refusing to run below the floor is frontend policy, not a panic in the layout code |
| The sidebar yields, never the main window | When rows run short, minimised panes are dropped rather than main panes squeezed. §9 says main panes are "fully rendered and fully functional" and the sidebar is awareness only. At the 80×22 floor all seven panes still fit |
| Tier formula | Largest integer scale whose grid still meets 80×22; Deep focus is one step finer. Reproduces all six numbers in §9's table exactly, and the four-pane layout reproduces its "roughly 60×15 each" |
| Deep-focus default floor | **Provisional at 100×28**, pending §4's Phase 0 legibility test, which is what actually establishes the minimum window at which tier 2 is offered. Wide strip height (4 rows) is likewise a first-pass tuning constant |

Two findings from actually rendering these screens rather than only testing them:

- **In-game prose is restricted to CP437 and typographic punctuation is not in
  it.** `—`, `’`, `“`, `…` all render as `?`. The boot header written in §4 above
  fails this. `cp437::first_unrenderable` exists so the Phase 1 content pipeline
  rejects offending lines at load time; against an ~88k-word budget (§12) written
  in ordinary editors, catching this by eye is not a plan.
- **A Wide-focus strip at the 80×22 floor has two content rows** — one focused
  pane at 6 rows and three strips at 4, less two rows of border each. Information
  parity holds because the linear stream is complete and strips scroll, but the
  legibility test (§4) should treat "can a player triage from a two-row strip?"
  as a question it is there to answer.

### Terminal frontend — pursued, second-class (draft 8)

| Question | Decision |
|---|---|
| Status | **Bevy is the product and never waits.** The terminal build is wanted and pursued, but second-class and cuttable |
| Split of costs | **Boundary** (`orbs-render`, Phase 0) is non-negotiable and ~free. **Dev-tool TUI** (Phase 1) has no parity obligation. **Ship-quality TUI** (Phase 9b) is cut-line item 3 |
| What it is | A full-screen raw-mode application (à la the Ubuntu Steam installer). Terminal as framebuffer + keyboard, nothing more |
| What it is **not** | It does not shell out, touch the real filesystem, or interoperate with the host shell. Same sim, same commands, same simulated tower |
| Architecture | New `orbs-render` crate owns the `Frame` (cell buffer + semantic styling + layout). **`orbs-render` decides what appears and where; frontends decide only how a cell is drawn.** Frontend-only enrichment (CRT, audio) may never carry information absent from the Frame |
| Colour | Indexed ANSI 0–15; inherits the user's terminal theme. Phosphor themes are Bevy-only |
| Glyphs | User's terminal font; Unicode U+2500 box drawing instead of CP437 |
| Not present in TUI | CRT effects, fidelity tiers, embedded bitmap font |
| Sabotage tells | Glyph/alignment cues are Bevy-only; `verify` is authoritative everywhere (§8.1) |
| Schedule | Boundary Phase 0 (cannot be retrofitted), dev tool Phase 1, ship quality Phase 9b |
| Cost | Boundary is ~free. Dev-tool TUI absorbed in Phase 1. Ship-quality TUI sits in Phase 9b and is cuttable |
| Dividend | If it ships, it is the cheapest route to screen-reader support — but §14 does not depend on it |
| Distribution | If shipped, both binaries in the Steam depot as two launch options |

### Sim architecture — settled (draft 8)

| Question | Decision |
|---|---|
| `orbs-sim` world model | **`bevy_ecs`, used standalone.** ECS throughout — it is an ECS game |
| Dependency boundary | `bevy_ecs` **only**, never `bevy`. Excluded: rendering, windowing, assets, GPU. (~90 crates vs ~340) |
| Why not plain structs | Shared component/system vocabulary across sim and frontend, no second model to keep in sync; and the `ecs-architecture` skill applies directly |
| Scheduling | `orbs-sim` owns its `Schedule`; frontends call `step(&mut World, tick)`. The Bevy app never drives the sim |
| Executor | **Single-threaded** (`SingleThreadedExecutor::new()`; `ExecutorKind` removed in 0.19). Multi-threaded ordering is non-deterministic without explicit constraints, and determinism is load-bearing for replay, offline parity, and the balance harness |

### Tech stack — pinned (draft 8)

| Item | Decision |
|---|---|
| Language / edition | Rust, edition 2024, pinned toolchain |
| Engine | Bevy `=0.19.0`, exact pin |
| Workspace | `orbs-sim` (headless, no Bevy) + `orbs` (shell) + `orbs-balance` (CLI) |
| Bevy features | `default-features = false`; **no `bevy_text`/`bevy_ui`** — all screens are terminal content |
| Rendering | Custom cell-grid renderer with integer fidelity tiers + cell-size-aware CRT |
| Save / data | `serde` + `toml`, readable and editable |
| RNG | `rand` 0.9, seeded, per-subsystem streams |
| Steam | `bevy-steamworks`, `steamworks` exact-pinned |
| CI | Windows + Linux cross-built on `ubuntu-latest`; macOS universal, signed and notarised, release-only |
| **Distribution** | **One executable + Steam shim, all assets embedded.** The earlier "single self-contained binary" claim was false once Steam was in scope |
| **Audio** | **Added** — key clicks, orb hum, alert tones, register-linked drone. Previously unspecified |

### Loose ends — closed (draft 8)

| Question | Decision |
|---|---|
| Slice command naming | **§6.1** — 16 commands, canonical arcane, ≤7 chars, three synonym registers |
| Canonical name length | Short by rule — long canonical names punish the mastery the echo arc creates |
| Phase 0 slice vs starting domains | Reconciled: both are **brewing + archive**. Sabotage tested via log poisoning on brewing logs, no third domain |
| Fragment trickle rate | **6–10/hour**, ~160 across the game; sieges ~60, hidden dirs ~80 |
| Script invoking script | Permitted, **call-depth limit 3** — the Attention pool alone fails silently |
| Script vs manual, same domain | Both allowed; different resources; contend only for reagents and mana, manual wins |
| Hostile host retaliation | **None.** Trace is the entire risk model |

### The dev spells are on the shelf in a debug build, and absent in a release one

**Reversed deliberately.** §19 previously argued that a dev ladder should reach the
grimoire only when `debug_spell` wrote it, so a tester looking at what the game does
would not see scaffolding on the shelf. In practice the first thing anybody does
with one is cast it, and `debug_spell breaking` before `invoke breaking` was a step
that taught nothing. `raise_grimoire` shelves all three at construction now, under
`cfg(debug_assertions)`.

**A shelved spell carries its own `Domain` from the file**, which is why this can
skip the room check `debug_spell` needs. That check exists because `scribe::write`
homes a *new* spell to wherever the player is standing, so writing an archive spell
from the laboratory would produce a file whose every line fails to resolve. Nothing
shelved is new, so nothing is homed wrongly — `threading` is an archive spell on the
shelf from tick 0 while the player starts in the tower.

`debug_spell` stays for the job that is still its own: handing back a fresh copy
after one has been edited or purged.

**The release guarantee is the half that matters**, and §12 is why — the archive's
central puzzle is one the player is meant to find, which is the whole reason these
live in `dev_spells.toml` rather than `spells.toml`. A release build gets neither
the nodes nor the eighty lines behind them.

#### The old test could not see the thing it forbade

`the_dev_spells_never_reach_the_grimoire_on_their_own` read `said(&sim)` — the
sentences said so far — and asserted no dev spell's name appeared in it. No spell
name appears in that transcript either way, so it passed against a grimoire holding
every ladder. **An absence test that cannot see the thing it forbids is not a
test.** Both directions ask `Sim::spell` now.

#### Two things four scripts broke that one did not

Adding three nouns to the grimoire changed what a *nonexistent* spell name resolves
to, and two tests were resting on there being only one.

**`invoke night_watch` became ambiguous.** It was *unresolved* while
`first_light.spell` stood alone, so the parser said so and that counted as the world
answering; with four scripts it ties between them and returns a numbered prompt of
`Echo` records, which `a_dark_verb_only_acknowledges_and_a_live_one_does_not`
filters out — so `invoke` looked dead. §19 records the same drift at `brew`, and
CLAUDE.md's rule from it stands: reach for `purge` when you want an ambiguity
fixture, rather than depending on how many nouns happen to exist.

**And a forward reference became a fault**, which is the real defect of the two.
§8's own worked example is a spell invoking one the player has not written yet, and
`compile::names_a_spell` quoted such a line rather than resolving it — but it only
covered `Resolved` and `Incomplete`. A forward reference matches no spell well, so
which of those it produces depends on how many spells exist: one, and it resolved
(the verb is weighted double); four, and it came back `Ambiguous`, fell through, and
was reported as `spell_missing`. Shelving the ladders surfaced it; **a player with
four spells of their own would have found it just the same.** The rule is *this line
names a spell*, and a tie between spells is still that.

### A press is instant, and the lens costs the tower nothing

**`PRESS_TICKS` is 0.** A probe used to schedule twelve ticks of work on the prism
through the ordinary production machinery; it now answers on the tick it is typed,
like `dial`.

**This withdraws ROADMAP's stated scarcity for the domain.** *"A read is not a
brew"* was that sentence, and it was the whole of what scrying cost: §19 refused the
production slot to the archive's maze because a solve is hundreds of ticks, and gave
it to a press because twelve is short enough to hand back between presses. There is
now no slot to hand back, and the lens competes with the laboratory for nothing.

Two things follow, and both are balance facts rather than opinions:

- **A bound solver runs beside a full brewing loop with no contention.** Measured,
  the solver earns about **0.07/tick** against clarity's 0.140 — half, and additive,
  where before it was a third of that and took the slot 12 ticks in every 13.
- **`orbs-balance`'s four pins are unmoved** (clarity 0.1376, damped 0.1326, haste
  0.2460, grind 0.0958), because no policy scrys. The harness cannot see this
  change, which is exactly why the number above is stated here.

Hand-play is the part that plainly improves: deduction over four sigils should not
wait twelve seconds per press, and now it does not.

#### It changed what a solver spell's loop does, which no test would have caught

`repeat until the prism is idle` solves the ward in front of it and exits. With a
press *in flight* for twelve ticks the guard was asked before the solve landed, so
the loop never fell out and an invocation lapped for ever — **by accident.** An
instant press lands the solve before the guard is asked, so the loop exits and an
invocation is one ward.

The faucet is therefore the **binding**, which re-casts a spell that has run off the
end. That is the honest shape and the one §8 already describes: an invocation is an
act, a binding is standing automation. `the_solver_spell_keeps_solving_and_never
_goes_quiet` now binds rather than invokes, and asserts the second hour earns like
the first.

`PRESS_TICKS` is kept as a named nought rather than deleted: it is the number the
domain's rates were derived against, and pricing a press again should change one
line rather than reintroduce a concept.

### `dial <socket>` with no sigil — the state that makes the ward scriptable

**The question was whether the lens can really be scripted.** It could, and the
script was twenty-four rungs across eighty lines. That is a spell nobody writes; it
is a spell somebody *generates*. So the domain gained the one piece of state the
language cannot keep for itself.

**The bind: a spell cannot name the sigil it has not tried.** Every operand in
`parser::question` is a literal, so *"put something else in this socket"* has no
expression — the only way to say it was to enumerate all six per socket and use the
per-socket mark count as an index into `SIGILS`. Both halves of that were wrong:

- **A mark is not an index.** `seat` marks a socket for every dial *aimed* at it,
  including one that moved nothing, and a swap marks the far end too — so the count
  outran the sigils actually tried and a socket exhausted its rungs with candidates
  left.
- **Nothing reset it.** A gain moves the baseline the ratchet keeps, so sigils
  rejected against the old figure are worth trying again. The reference ladder in
  `tests/ward.rs` does exactly this with `tried.clear()` — and **that test never ran
  the spell**, so it was proving a Rust loop terminates while the thing a player
  casts went unmeasured.

**`Ward::tried` is the state**: which sigils each socket has been set to since the
last gain. It is what a player with squared paper would keep, which is the test this
log applies to the maze's map. Three rules over it:

| | |
|---|---|
| `seat` records the sigil | **before** the early returns — dialling one a socket already holds *has* tried it, and that is exactly the case where nothing moves |
| a gain clears the sockets that **moved** | not all four; see below |
| `replenish` gives them all back when none has any | the termination guarantee |

**`dial <socket>` takes an optional sigil**, on `recall`'s `TOPIC_OPTIONAL`
precedent: bare and argumented are the same act — turning that dial — at two
scopes. Bare, the ward picks the first untried sigil. The socket publishes
`untried` as a count, so the existing comparison grammar guards it:
`if the first has 1 or more untried`.

**The solver is now four rungs and twenty-four lines**, against twenty-four rungs
and a hundred and five. That is the whole deliverable: a spell a person can write.

#### Three measurements, and the first two corrected the third

**A plateau that was not one.** The first evidence that the old ladder stalled was
seed 1 freezing at 78 experience between `meditate 4800` and `meditate 9600`.
`MAX_MEDITATE` is **3600**: both runs were the same 3600 ticks. A long wait has to
be several commands, and the retraction is recorded here because the finding was
stated before it was checked.

**Clearing all four sockets on a gain cost a quarter of the rate.** Measured over
14400 ticks across six seeds, that version ran 216–258 experience against the
twenty-four rung ladder's 186–440 — it spent its next laps re-trying sigils that
were still wrong. Clearing only the sockets that **moved** brings it to 292–338,
which is a mean of 313 against 323: even, and far more consistent than the spread it
replaced.

**And the old ladder did not stall after all.** With the cap accounted for it earns
linearly. What was true of it is the part that mattered: nobody would write it.

#### The termination guarantee is a proof, not a sample

A four-rung ladder falls through to `wait` when every socket reports no `untried`,
and the loop below it then presses an unchanged aperture for ever — holding the
tower's one production slot and earning nothing. That is the failure a faucet has:
not a crash, silence. Six seeds over 14400 ticks never reached it, and *never
observed* is not *cannot happen*.

`Ward::replenish` gives every **unsettled** socket its candidates back when none has
any left. Settled sockets keep their lock, so the monotonicity the walk rests on —
`aligned` only rises, settled sockets only accumulate — is untouched. Two unit tests
drive **all 360 codes** rather than a handful of seeds: one asserts a socket is
always left to turn, the other that the four-rung shape breaks every code.

### The ward's sheet was ten cells wide in a pane of a hundred

**Reported from play: the ward box is too small to read.** It was, and the size was
only half of it.

The first sheet packed four sigils, two spaces and four pegs into ten cells —
`☼○♂♀  ••○ ` — which is correct, compact, and a wall of symbols. Two things were
missing that no amount of *correct* makes up for:

- **Nothing said which column was which socket.** `dial second borax` names a socket
  by word, so a player had to count along the row before they could type.
- **Nothing said what a glyph was called.** `♦` is `pewter`, and the only place that
  mapping appeared was the transcript the sheet exists to save you re-reading.

It is 39 cells now: a press number, four named socket columns with the glyph centred
under each, and the pegs under an `answer` header — with a two-row legend beneath
pairing every glyph with its word.

```
    first second  third fourth  answer
 1    ☼      ○      ♂      ♀   ○ ○
 2    ☼      ♂      ○      ♀   ○ ○
 3    ☼      ○      ♀      ♂   • ○
───────────────────────────────────────
 →    ☼      ○      ♀      ♂   · · · ·
 ☼ nitre   ○ alum    ♂ borax
 ♀ quartz  ♦ pewter  ♠ ochre
```

**The words come from the sim, through `Ward::view`.** They are content
(`tower::ward::SOCKETS` and `SIGILS`), and `orbs-render` may not depend on
`orbs-sim` — so `Board` carries two name arrays rather than the painter knowing any
of them. It is the same reason the view exists at all.

**Numbered from the history, not from the sheet.** The cap still shows the last
twelve presses of a fifty-one press ladder; numbering those `1..12` would say the
solve had just begun, so the gutter counts the real press.

**It still fits the 80×22 floor** beside a transcript, and still refuses whole
rather than truncating — a row missing its pegs says a press answered nothing.

Four tests pinned the ten-cell strings and are rewritten against the shape rather
than the spelling: every row is exactly `COLS` wide (one short row shifts every peg
beneath it and makes two presses look alike), the header names what `dial` names,
and the legend pairs all six.

### `move` stays in the laboratory, and stops being taught there

**Raised from play: the word should be unnecessary, because calling a stage's own
verb ought to fetch the ingredient.** It already does, and more completely than the
question assumed — `pipeline::reachable` walks every instrument in the room that is
not busy, in raise order, and *then* the store. So `grind sage` fetches from the
shelf and `digest ground-sage` reaches straight into the mortar and takes the one it
needs, leaving the husks. **A whole clarity brews to `experience 16` without the
word `move` appearing once.**

**So it is redundant in the loop and is not redundant in the room**, and those are
different claims. Two things need it and nothing else does them:

- **`move clarity to arsenal`.** Finished work leaving the room that made it is what
  `/tower/arsenal` was built for, this log records that *nothing in the tower could
  be carried between domains at all* before it, and potions are made in the
  laboratory. Removing `move` there would put the exemption back where it started.
- **Reaching `charged`.** `grind sage` is the fetch and the wield in one tick, so
  the bowl never rests at `charged` and never pours. Three animation See-it lines —
  the mortar filling, the bath's vessel, the flask's two-colour mixture — exist only
  because `move` can stop there. No other word can.

**What was actually wrong is that the manual taught the superseded form first.**
`recall move`'s page opened on `move sage to mortar_and_pestle, then wield it` — a
pair of commands no loop has needed since §10.1 gave every instrument a verb. The
page now leads with carrying to the arsenal and demotes loading to a note. The room
primer never mentioned it.

#### The property nothing asserted

`orbs-balance`'s `BY_HAND` brews a clarity with no `move`, which looks like the
claim and is only its weaker half: it `empty`s the mortar before digesting, so the
ground-sage is fetched from the **shelf**. It would keep passing if the tool-to-tool
reach were deleted tomorrow. `tests/fetching.rs` holds the real one, and a mutation
that skips the instrument tier in `reachable` fails it with *"there is no
ground-sage within reach"* — which is what a player would have seen.

**`empty` is not the same question and is still required.** It clears the
*byproduct* the last stage left, so the mortar can take a second load; without it
the brew stalls at `grind rock-salt` with *"the mortar_and_pestle can do nothing with
husks, rock-salt"*. Applying the same fetch-for-me treatment to byproducts is a
larger change and is not this one.

### A verb is scoped by its fixture, not by whether it takes the slot

**Asked for from play: `help` in a domain should only show what works there.** It
was listing `research`, `follow` and `wander` in the laboratory and the lens, where
none of the three can do anything.

The cause is the entanglement this log already recorded. `Scene::offers` asked
`Verb::is_operation()` — which is the **production slot** question — so every verb
that took no slot was offered in every room. §19 called `follow` and `wander` a debt
*"waiting on one missing mechanism"* and said a third occurrence would be the
argument for building it. `research` was the third, and it was worse than the other
two: the stacks already declares `operation: Some(Verb::Research)`, so the content
had said which room it belonged to all along and nothing consulted it.

**`Verb::anchor()` is the mechanism.** It answers *which fixture must stand here for
this verb to mean anything*, and it is a different question from the slot:

| | takes the slot | anchored to a fixture |
|---|---|---|
| `grind` | ✅ | ✅ mortar |
| `dial` | ❌ | ✅ socket |
| `research`, `follow`, `wander` | ❌ | ✅ stacks |
| `move`, `wield`, `empty`, `stop` | ❌ | ❌ — they name their target |

Neither implies the other now, and the lens is where they first came apart.

**Most of it is derived.** A verb that some `Branch` declares is *self-anchored*, so
the content says which room it belongs to and the parser holds no list of rooms.
`every_self_anchored_verb_is_declared_by_a_fixture` checks both directions: a verb
claiming an anchor no fixture declares resolves in **no** room, which is as quiet a
failure as the one this fixed. `follow` and `wander` are the spelled-out exception —
they act on the reading *inside* the stacks, and a fixture carries exactly one
`Operation`, which the stacks had spent.

#### The refusal got the other half of its answer

*"there is nothing here to wander with"* satisfies §6 — it names the verb and says
it does not apply — but it is half an answer. `tower::fixture_of` walks the same
content, so it is now **"there is no stacks here to wander with"**, and a player who
asks in the wrong room is told where to go.

#### Four tests were measuring scope while claiming to measure naming

`naming.rs`'s scene built itself from `is_operation`, with a comment saying its
purpose was *"every word the game has, all live at once"* so the file would not end
up *"measuring the scoping rule instead of the naming"*. That is exactly what it
started doing: `study`, `decipher` and four more came back `Elsewhere`. Both scenes
now fold `Verb::anchor`.

`parsing.rs` needed the opposite in one place. `tower()` stands **nowhere in
particular** on purpose, because `the_retired_brewing_words_all_still_land_somewhere
_deliberate` needs `mix` out of scope to reach `Elsewhere` — so offering everything
there broke it. The two requirements are now two helpers, `tower()` and `anywhere()`,
each documented with which tests need which.

And `an_instruments_verb_is_only_a_word_where_the_instrument_is` asserted the
substring `"nothing here"`, so it failed when the refusal *improved*. A test that
depends on wording is what rule 6 puts prose in a file to prevent; it asks
`Outcome::Unresolved` and the fixture in `FieldName::Source` now.

#### The boot report was listing a word that is not a word there

Its own doc already said domain verbs are left out because *"the report is written at
the tower root, and `grind` is not a word there — sending a tester after it would be
the exact dead end this list exists to avoid."* `research` was in the list anyway,
for the same reason: it takes no slot. The report needed no change, only the correct
predicate; the test restating the rule is where it was fixed.

### `help` explains the room before it lists the words

**Asked for from play: a player who types `help` in a domain is not asking for a
word list.** They are asking what a ward is, or what the stacks are for, or which
verb starts the thing in front of them. `recall` bare answered with twenty-five
verbs in five groups and not one sentence about the room — §6.1 makes it the
in-world manual, and a manual that opens with an index is a reference rather than
an explanation.

So the listing is now second. First is a primer for the room the player is standing
in: **three keys, and they are the three questions.** `man_here_<room>` is what the
place is, `man_start_<room>` is how to begin its puzzle, `man_solve_<room>` is how
to finish one. What-it-is before how-it-works is the order a *thing*'s
`recall_`/`using_` pair already uses, because it is the same split.

**Three rather than two because of the width lint, and that turned out to be
right.** `every_authored_line_fits_the_worst_case_width` gives 70 cells against
§4's 80×22 floor and the first draft ran to 112 in one sentence. Splitting start
from solve is not a workaround for the budget: they are two instructions, followed
at different times. The whole screen — primer and full listing — fits the floor's
nineteen transcript rows exactly.

**`man_`, not `recall_`.** `Prose::topics` strips `recall_` to decide what is
*nameable*, so `recall_here_lens` would register `here_lens` as a subject nobody
authored. That is the trap this log records paying for twice already, at
`grimoire_step_or` and at `clarity_use`. `man_` is the manual's own furniture prefix
and is not a noun space.

The heading is the room's name, and is the one heading here that is not a prose key:
it *is* the place, and authoring seven `man_section_<room>` lines that say the room's
name back would be furniture with a translation cost.

#### Two lints, and the second found a wrong instruction immediately

`every_room_a_player_can_stand_in_explains_itself` drives `Sim::briefs()` and
requires all three keys for every **built** domain, plus `arsenal` and `tower` by
name — those are standable and are not rail domains, so `briefs()` cannot see them.
`built` is what makes it honest: `forge`, `menagerie` and `battlements` are dark,
need nothing yet, and start needing it the day `build.rs` raises them. §10 has five
more rooms coming and each will be built by someone not thinking about the manual,
where the failure is silent — `primer` is guarded on `Prose::has`, so an unauthored
room just prints the old word list.

`a_primer_only_names_words_that_room_actually_offers` checks every verb a primer
names against `offered`, the same filter the listing below it uses, so the two
halves of one screen cannot disagree. §15 weighs the dead-end rate above the raw
resolution rate, and a manual is the worst place to spend it — the player did the
right thing by asking. It caught **`bind`** in the grimoire's first draft: gated at
concentration 0, so a fresh player was being told to type a word the listing
directly below correctly refused to print.

**And it caught availability only, which the grimoire also proves.** `scribe` *is*
offered there and refuses — *"a spell is written for a place. go where the work is
first"* — so the same first draft sent the player at the one thing that room cannot
do, and the test passed. **Looking is what found it**, which is why the See-it line
runs `help` in all six rooms rather than trusting the green. What the lint does hold
is the failure that arrives later and silently: a verb renamed, or moved between
rooms.

### Leaving a surface hands the prompt the middle of a keystroke

**Reported from play: `wander` was sitting in the prompt after leaving the maze.**

You walk the archive's stacks with the arrow keys and press Escape. The maze lets
go of the keyboard on that frame — but a finger is still on the arrow and **key
repeat keeps delivering**. By the next frame the prompt owns the keyboard again, an
arrow at the prompt means *recall history*, and the newest history entry is the
`wander` that opened the maze. So leaving the maze typed the word back in.

**It is not a maze bug.** All four surfaces that can own the keyboard — the editor,
the weave screen, the unfurled transcript and the maze — hand it back the same way,
so escaping any of them on a held key does the same thing. The maze is only where it
was noticed, because it is the one surface a player holds a key *down* in.

#### The fix is narrower than a quiet frame, and deliberately

Swallowing everything for a frame or two would be a race against the player's
key-repeat rate, which is a setting on their machine. What is actually wrong is
exact: **the prompt is being handed the middle of a keystroke whose press it never
saw.** So `HeldOver` records which physical keys were down at the moment the
keyboard changed hands and drops events for exactly those, exactly until they are
released. A fresh press afterwards is a real keystroke and gets through — which is
half the test, because a fix that swallowed the keyboard from the moment a surface
closed would satisfy the other half and break `Up` for ever.

`chord_is_stale` already reasons this way about ghost modifiers, and for the same
reason: a key whose press this surface never saw is not the player talking to it.

**The watch has to be ungated, and that is the part worth remembering.** The obvious
home for it is inside `type_into_line`, which already reads all four surface states
— but that system is gated on `on_message::<KeyboardInput>`, so it never runs on the
frames where a surface owned the keyboard and nobody typed. The edge it needs to see
is precisely the one it cannot: Escape arrives, the surface has already let go, and
there is no previous frame on record saying it ever held on. A first attempt put the
transition there and fixed nothing.

`type_into_line`'s own comment predicted the four-term predicate's ceiling was five
and that a single `Focus` owner was worth building before the fifth surface arrived.
This is not that refactor, but the predicate now lives in one function that both
callers ask, rather than being spelled out twice.

#### The test harness was modelling half a keystroke

`press` — the helper thirty-odd tests type through — wrote a `Pressed` event and no
`Released`, and stamped every keystroke `KeyCode::KeyA` on the stated grounds that
nothing read the field. So `A` stayed held for the rest of every test. That was
invisible until `watch_focus` read `key_code`, at which point one correct fix
produced a false failure in an unrelated test about the transcript.

**`tap`, ten lines away, already knew:** *"Released on the way out, or
`input_just_pressed` sees it held and the next tap of the same key is not a fresh
press."* `press` now writes both halves. A test about which key is physically down
also needs real key codes rather than a shared `KeyA`, so
`leaving_the_maze_leaves_nothing_in_the_prompt` uses a helper that carries them —
without it, every key lands in one bucket and the test passes against a fix that
swallows the keyboard permanently.

### A step is logged, not drawn (Phase 1, §10)

*"The reading goes north"*, once per step, in a maze that is hundreds of steps.
The pane filled with a line-per-press restating what the map had just drawn, and
the copy that scrolled away was the player's own typing.

| Question | Decision |
|---|---|
| The rule | **The map is the report.** Where the screen already shows a fact, the sentence saying it is the same fact twice. `follow`'s success is now `RecordBuilder::quiet` — emitted, stored, `sift`-able, spoken, and filtered out of the transcript |
| Not a deletion | §3 forbids unlogged output. The record is unchanged in every respect except that `Records::drawn` skips it, which is the same *"log, not pane"* rule `FieldName::Spell` already carried. `drawn` now has two reasons to skip: whose doing it was, and whether it is worth drawing |
| A wall stays drawn | Deliberate, and the reason the fix is not *"make `follow` silent"*. A refusal moves nothing, so the map reports nothing at all — a press that did nothing has to say so (§6). Both halves are tests |
| **The log it names was empty** | Found on the way: `files::in_domain` decides a domain's log by matching `Source`, `Path` or `Origin` against the domain and everything standing in it, and the archive's completions set **none of the three**. `peruse archive.log` returned nothing after a walk, and had done since the archive was built. Drawn records got away with it because the pane was a second surface; a quiet one has no second surface, so this had to be fixed before the rest was honest. The lectern now goes in `Source`, as `work::produce` and `work::slot` already file theirs |

The near-miss worth recording: hushing a record whose log never held it would
have moved the steps from *noisy* to *nowhere*, and a test that only checked the
pane would have passed.

**See it:**

```bash
# Steps do not reach the transcript; a wall still does.
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend archive; research; \
  follow west; follow west" cargo run -p orbs

# ...and they are in the log, with the tick each happened on.
ORBS_SEED=3 ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend archive; research; \
  follow west; follow west; peruse archive.log" cargo run -p orbs
```

### Three fixed things in the stacks, made random (Phase 1, §10)

*"The path to the end is always relatively the same."* The maze itself was
already randomised — Prim's, uniform frontier draw — but three things around it
were constants, and between them they made one errand wearing different walls.

| Question | Decision |
|---|---|
| The reading's corner | **Uniform across all four.** It was the top-left cell on every seed |
| The way out | **Drawn against a weighting, not placed.** It was the bottom-right square on every seed, so paired with a fixed start it was the same diagonal every time. Weight is `(reach − away + 1)³` on Manhattan distance from the corner *opposing the start*, with the reading's own cell at weight 0 so the exit is never underfoot |
| Why cubed | Measured, not liked. Over 16×11 it lands within 8 of the opposing corner **63%** of the time, median 7, with a 13% tail past 12. Squared is barely a lean — mid-maze about as often as far — which is *"the errand is always the same length"* wearing different clothes. Six is the old fixed corner with extra steps |
| Prim's seed cell | **Random, and decoupled from both endpoints.** It was cell 0. Prim's grows outward from its seed, so the seed is the centre of a radial structure — growing every maze from the same cell the walk *started* at meant the walk always began by unwinding the oldest, straightest part. This is the change that answers the complaint most directly: the tree was random and the journey was not |
| Replay | Three new draws on the archive's stream, in fixed order, and `CORNERS` is a fixed table because a roll into it has to mean the same corner for a given seed for ever. Per-subsystem streams mean nothing else shifts (§3) |

**`ORBS_SEED` was added to see it, and the omission was load-bearing.** Anything
the world *generates* is one seed's worth of evidence per run, and the binary had
no way to be anything but seed `0x0B5`. Three dumps taken to check the
randomisation came out identical and looked like proof it had failed; they were
three copies of one seed. Tests swept seeds through `Sim::new` and always could —
this is the same reach from outside, so a person can look rather than trust a
test, which is the whole of §15's gate.

**See it:**

```bash
# Two seeds, two corners, two ways out. `☼` the reading, `Ω` the way out.
ORBS_SEED=3  ORBS_BOOT=0 ORBS_DUMP="attend archive; research; wander" cargo run -p orbs
ORBS_SEED=11 ORBS_BOOT=0 ORBS_DUMP="attend archive; research; wander" cargo run -p orbs

# ...and the distribution, which no single dump can show.
cargo test -p orbs-sim --lib research
cargo test -p orbs-sim --test solver   # every swept seed is still solvable
```

### A third test layer: the game, played (Standing, `0.3.11`)

**1,155 tests and not one pressed a key.** They drive `Sim` directly or paint a
`Frame` and read it back, so the event loop, the redraw diff, keyboard ownership
between five surfaces, the clocks measured off `Time`, and colour as a terminal
resolves it were covered by nothing at all. `orbs-tui/src/surfaces.rs` was 302
lines with zero tests.

Every defect found in that frontend was found by a person looking at a screen,
and every one was invisible to the suite. So `crates/orbs-tui/tests/playing/`
runs the real binary under `tmux`, types at it, and reads the screen back — 92
scenarios, `#[ignore]`d, run by `scripts/play.sh` and by a non-blocking CI job.

**Two findings on the first pass, both of which every other layer was blind to.**
The ambiguous-glyph width probe was measuring `‼`, which is East Asian *Neutral*
— one column on every terminal there is — so it reported `narrow` under every
configuration and the whole fallback beneath it was unreachable. And the maze map
ran a full second behind the arrow keys: `Panel` is rebuilt once a tick on the
stated ground that *"the world moves at 1 Hz too"*, which `Sim::walk` — the third
entry point, and the only thing that reaches the world without a tick — had
falsified. The Bevy build never had it, because `walk` takes `Tower` by `&mut`
and `refresh_panel` hangs on `resource_changed`; a bare loop has no such thing,
so it is hand-rolled at the one keystroke that needs it.

**An assertion is scoped to the newest command block, and the two obvious
alternatives were built and measured before that was settled.** A plain substring
search matches *history* — the clarity chain empties the mortar twice, six steps
apart and both on screen, so the second wait returned in 1 ms against the first
one's line and the driver went green on an expectation that was factually wrong.
Counting occurrences instead deadlocks: the pane holds exactly eight command
blocks, so past the ninth repeat each new line pushes an old one off and the
count never rises. Position is the only thing about that pane that is stable.

**A wait that cannot be fooled is worth more than a fast one.** Four of them
failed by *passing*: `does("meditate 20", "meditate")` is satisfied by the echo
before a single tick has passed, and it cost three scenarios — one of which asked
for 7,200 ticks and got none. Anything whose evidence is a clock waits on the
clock.

**Ambient sabotage is chosen away rather than tolerated.** `drift` and
`substitution` both draw from the seeded `Threat` stream, so their schedule is
fixed in tick space: seed 3 poisons a log inside 200 ticks and seed 0 swaps a
reagent inside 7200, while 11 and 42 are quiet through both. Pinning a measured
seed is free and stronger than choosing survivable assertions.

**Found here and then fixed: `orbs-tui` had no boot sequence** — see the entry
below, which the suite is what surfaced.

```bash
scripts/play.sh                     # 95 scenarios, about half a minute
scripts/play.sh routing::           # keyboard ownership, on its own
```

### The terminal build boots too — supersedes "no boot sequence" (Standing, `0.3.12`)

`orbs-tui` skipped §4's sequence for a version, and the argument recorded in its
own module docs was that a dev tool whose charter is *"instant-startup"* (§15)
must not begin by making you wait.

**That was the wrong trade, and the tell is that it was made silently.** The
sequence is the game's opening image — a dark tube, an orb *found* rather than
switched on, the machine taking an inventory of itself — and a second frontend
that quietly omits it is not the same game with a different rasteriser. §13 lets
this build lose *enrichment*; the opening is content.

The argument was really about the **development loop**, and `ORBS_BOOT=0` had
already answered that — on both sides, for exactly this reason. So the skip is
where it belongs: in the switch, not in the build.

**Almost nothing had to be written.** `Boot`, `Stage` and the card were already
`orbs-shell`'s, and `Boot::default` already reads `ORBS_BOOT=0` itself. What the
frontend supplies is what only it can: a `Duration` per frame, a `Frame` to paint
into, and the **engine line** — `crossterm 0.29` where the other build says
`bevy 0.19.0`, because the card is a diegetic inventory of *this* machine and a
shared painter naming Bevy would put a component on the list that is not in the
box.

Three things the loop has to get right, and each is a rule rather than a detail:

- **The tower's clock does not run.** `tower::drift` rolls once per tick, so a
  sim left running through nine and a half seconds of animation advances its RNG
  stream by a wall-clock-dependent number of draws — the same seed reaching a
  different world depending on how fast the machine drew a logo.
  `Stage::world_runs` already said so; the terminal loop now holds `ticked` at
  `now` while booting, so the catch-up does not then replay the sequence as a
  burst of ticks.
- **Nothing is typed, and one thing still is.** Every keyed system in the Bevy
  build is gated on `booted`, and §19 removed the keypress skip. Both hold here.
  *Leaving* is the exception, and a terminal is why: under a window manager there
  is always a way out of an animation, and raw mode takes that away — `Ctrl-C` is
  ours to answer or nobody's. Answering the exit is not a skip; it ends the
  session rather than jumping to the tower.
- **No caret on that screen.** The `Prompt` stage was removed from the sequence
  because an input line that cannot be typed into is an affordance that does not
  work, offered before anything else on screen is.

```bash
cargo run -p orbs-tui                 # ...and watch it, which is the point
scripts/play.sh presentation::        # the sequence, the card, and the still clock
```

### What a review of the played game found (Standing, `0.3.13`)

The third test layer's first review, and the useful half of it was about the
instruments rather than the game. Recorded because several are the *same* defect
wearing different clothes.

**An assertion that can be satisfied by the question is not an assertion.** The
driver waited until the newest block began with `wizard $ <line>`, and then
searched that block — which by construction contained the line just typed. So
`does("move zzz …", "zzz")` passed on its own argument. It is the stale match and
the saturating count a third time: not *an older answer* but *the question,
mistaken for the answer*. `block` drops its header now, and a needle can no
longer be found in the words the test typed.

**A colour assertion that cannot see the thing it names reports coverage it does
not have.** Both `ink` scenarios decoded the wrong columns — the fire test read
the tower rail (the athanor burns at 100-101) and the tint test read the
transcript, where `Role::Success` is already green and roughly seventy `√`
markers sat before any sage moved. Restoring the `dark-red → white` ramp §19
forbids would have passed. The tint scenario now takes a **control**: the same
band, before and after, so green that was not there and then is can only have
come from the material.

**Five verbs, one list, and `quit` reached four of them.** It went into the
vocabulary, `Verb::ALL`, `dispatch::execute` and the tower's own count —
`spell::run::may_issue` was the one place it was missed, so a spell could raise
`Quitting`, and a *bound* spell re-casts every time it runs off the end. The
three verbs barred beside it are barred for seizing the keyboard; this one closed
the game.

**A peek that nothing called.** `quit_requested` reached for `ResMut<Tower>`
unconditionally, stamping the change tick — and its own run condition is
`resource_changed::<Tower>`, so it re-armed itself for ever and dragged
`refresh_panel`, `suggest` and four `open_requested` systems to frame rate.
`Sim::is_quitting` was added as exactly that peek and had zero callers.
`editing::open_requested` records the same regression happening once already.

**A meter is not always a duration.** The rail suffixed every one with `t`, but
two of the three built domains count something else — the stacks count cells and
the prism counts *sigils aligned*, which falls 4 → 1 as the player wins and so
read, on the surface built for a glance, as a job about to finish. `Meter` now
carries a `Unit`, because rule 2 gives a frontend how a cell is drawn and not
what the thing in it is.

**A cliff removed and then moved.** `Board::SHOWN` caps the ward sheet at twelve
presses so it cannot outgrow its pane — and at the 80×22 floor twelve rows is one
too tall, so the whole sheet vanished on the twelfth press. Refusing whole is
right for columns and wrong for rows, because `SHOWN` already makes this a window
on the recent end: ten of fifty-one is the same kind of view as twelve.

**`MIN_RAIL_BOX` was raised for the third time**, 3 → 4 → 5, each time for the
identical symptom one row up: a squeezed box keeps its name and drops the
`►spell` row, which is the only thing saying a room is automated. It survived at
four because the Bevy grid is fixed at 120×45; a terminal's is the window's, and
`tui.sh start 177 38` sits inside the range that was wrong.

**A conditional draw from a shared stream.** `substitution` hoists its roll above
every early return and its comment names the hazard exactly; `drift` — the older
of the two — returned first and drew second, so once every log was poisoned it
stopped drawing and shifted the swap schedule. Adding a seventh domain would have
silently changed every saved session's sabotage.

**A pin measured in one world is a pin on that world.** `agrees.rs` hardcoded
seed 0 while claiming three seeds had been measured in band; they had not — over
seeds 0, 3, 11 and 42 clarity spans 0.1244 to 0.1383 against a 10% band, so seed
3 failed and seed 0 passed on identical code. Sabotage is part of the economy, so
that spread is the game; measuring it once was the error. The pin is held against
the **mean** of four worlds now, which is sensitive to a regression (it moves
every seed) and survives the luck of any one.

**`--why` could not see a line that failed to parse.** Every parser outcome is
`Role::Normal` by design, so `tally` dropped it into neither column — and CLAUDE.md
tells a reader to check `cost` *before* the rate. `landed` was separately counting
every success record rather than completed runs, reporting five before a command
had been issued.

**And three parity gaps in the terminal build**, all of the shape *a comment
claiming parity sat near the code that broke it*: the editor swallowed
`PageUp`/`PageDown` where Bevy pages behind it; `unfurl` took the keyboard
without paging back, so the word did nothing visible; and there was no `HeldOver`,
so leaving a surface on a held arrow leaked key repeat into history recall. The
last has no direct port — a terminal sends no key-release event — so it is a
**repeat-gap window** instead: auto-repeat lands every 30-40 ms and nobody
double-taps an arrow inside 120, so an unbroken run out of a handover is a held
key and the first gap is the release.

### A word only ever matches itself (Phase 1, parser correctness)

`digest ground-sage`, with no ground-sage on the shelf, digested **ground-salt**
— echoed it, moved it into the balneum mariae, and then reported that the bath
could do nothing with it. A silent wrong action, which §6 ranks below a refusal.

| Question | Decision |
|---|---|
| Why did it happen? | The two names differ by two characters in eleven, which `similarity` scores **819** against a `MIN_SIMILARITY` of 600 — comfortably inside the typo band. Only one candidate scored, so it won outright at `Confidence::Clear` and ran without a prompt. Nothing was wrong with the fuzzy scoring; it was being asked the wrong question |
| The rule | **Fuzzy matching is for typos, and a typo is by definition not a word.** A phrase that is itself a name the laboratory knows now only ever matches *exactly*. `Scene` carries that vocabulary (`Scene::knowing`), separately from what is in the room |
| What still works | Typos still fuzz — `ground-slat` is not a word, so it still reaches `ground-salt`. Abbreviations still prefix — `ground-sa` is not a word either. An exact name still matches itself when it is present. All four are tests, because "only ever matches exactly" is a plausible way to break the third |
| Where the vocabulary comes from | `Recipes::substances` — the recipe vocabulary **union the fuels**, because the athanor transforms nothing and so has no recipe, and charcoal is exactly the reagent a tester reaches for first. This was `debug_spawn`'s private helper; it is shared now, and the parser's use is the load-bearing one |
| Why this is a *second* half | `spell::compile::fix` already applied this rule, falling back to the recipe vocabulary so a spell could tell *"there is none here"* from *"you have mistyped something"*. The prompt could not. §19 already records the two halves disagreeing as how the `has ground-slat` defect survived — this is the same disagreement found from the other side |
| **Still owed** | The word is now *dropped* rather than misread: `digest ground-sage` echoes `digest` and reports the bath is empty. Better than the wrong action and still not the answer, which is *"there is no ground-sage here"*. `digest`'s reagent slot is **optional** (bare `digest` is meaningful), so the miss produces no `Incomplete` and nothing reports it. The shape to follow is `Resolution::Elsewhere` and `InSpell` — both exist because *"I do not know that word"* would lie about a word the game taught the player, and a known substance that is out of stock is the same lie about a noun |

**See it:**

```bash
# The defect, refused. The echo shows `digest` — the word was not understood.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_spawn ground-salt 2; \
  digest ground-sage" cargo run -p orbs

# ...and the typo it must not stop forgiving.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_spawn ground-salt 2; \
  digest ground-slat" cargo run -p orbs
```

### A fixed 4:3 picture — supersedes the fidelity tiers (Phase 1, `0.1.24`)

The window used to decide the grid: `tier_one` picked the largest integer cell
scale whose grid still cleared a floor, and the cell *count* fell out of the
division — 1280×720 gave 160×45, 1920×1080 gave 120×33. Every pane, border, the
prompt's row count and the maze viewport were recomputed against a grid that
moved under them, and half a dozen constants existed only to manage that motion.

**The grid is now a constant and the window decides only how big a cell is.**

| Question | Decision |
|---|---|
| Why is 120×45 the grid? | A cell is 8×16, so a 4:3 *picture* forces `cols : rows = 8 : 3` — the aspect is a property of the grid, not something imposed on it. 120×45 is 960×720 at native size: the same 45 rows the game already drew at its default window, 40 columns narrower. Four panes tile at 60×21, past the 60×16 §9 calls workable, so the mechanism the tiers funded is funded by the grid instead |
| Why not 160×60, the next step up? | It was chosen first and reversed on the arithmetic. **720 divides 720, 1080, 1440 and 2160**; 960 divides none of them. At 160×60 the four commonest display heights land on ×0.75, ×1.125, ×1.5 and ×2.25 — 1080p, the modal desktop size, loses 44% of its glyph height against the old model and every size is an irregular fraction. At 120×45 they are ×1.0, ×1.5, ×2.0, ×3.0, three of them pixel-exact |
| Integer steps or continuous fit? | **Continuous.** Integer-only scaling is crisp everywhere but fills the window nowhere: 1080p and 720p land on the same step, so a maximised 1080p window would show the picture at a third of the screen. The cost is that ×1.5 duplicates some pixel columns and not others — a *regular* 2,1,2,1 alternation, masked by the CRT bloom |
| Who does the letterbox? | `ScalingMode::AutoMin { 960, 720 }` on the camera, and nothing else. It shows at least the picture in the window's own aspect and centres it, so the mesh stays in virtual pixels and `grid::build` lost its scale parameter entirely. It re-derives on `WindowResized` by itself, which is why it is set at camera spawn rather than by a guarded system — a projection left unset draws *correctly* at the opening window and wrong everywhere else |
| Where do the bars go? | **Outside the tube — they are the dark room, not part of the glass.** The CRT is still a full-screen pass, but every *shaped* term in it (barrel, vignette, edge mask, bezel) is measured in **tube space**: the 4:3 picture remapped to 0..1, published as `Tube::fill_x`/`fill_y`. Everything outside falls out black for free, with no extra mask and no branch. This shipped for one version measured in window space, which curved the bars along with the phosphor and made the "monitor" whatever rectangle the player had dragged — on a wide window, a letterbox-shaped tube nobody ever built. §4's *"no 16:9 letterbox, the grid fills the window"* is reversed on both halves |
| The rounded bezel had never worked | It rounded the **unwarped** tube rect, whose corners lie outside the visible picture because the barrel warp insets it — so the radius did nothing at any value and the picture kept a hard 90° corner. Now the edge mask *is* the rounded-box SDF: one boundary, aspect-corrected so the corners are round rather than 4:3-elliptical. **A screenshot cropped to a corner is what found this**; the constant looked correct and the arithmetic around it was fine |
| The radius is a computed bound | `0.027` of the short axis — the largest that loses no cell, and rule 2 is why it is a bound rather than a taste. The mask is evaluated on the *texture* coordinate, so a cell's own grid position is what gets tested; the binding one is the session pane's border corner at `(0, 0)`. `0.09` looked right and ate the `d` of the prompt. A test reimplements the shader's SDF and asserts both that no corner cell is lost and that 10% more radius *would* lose one, so the bound is known tight rather than merely safe |
| What did `F4` cost? | Deep focus used to drop a fidelity tier as well as re-dividing the panes. It cannot now; `F4` is purely `tiling::deep` against `tiling::wide`. **§9 sold Wide focus as the large-text mode and that is gone with it** — see the debt row below |
| Sub-native windows | Minification *drops* strokes out of an 8×16 bitmap rather than shrinking it, so below `MIN_SCALE` (960×720) the game draws the "window too small" card. The atlas sampler is `mag: Nearest, min: Linear`, so what is below the floor degrades soft rather than broken — and the missing half-texel inset stays survivable only because minification is unreachable above it |
| `is_hostable` has two halves now | The grid can be below the 80×22 authoring floor, which only `ORBS_GRID` can produce; or the window can be below `MIN_SCALE`, which is what a player reaches by dragging. They used to be the same test, because a small window *was* a small grid |
| `DEEP_FOCUS_FLOOR` kept, the branch removed | The game clears it by construction, so `drive_panes` became `const PANES: u8 = 2` — a condition that cannot fail is a lie in the shape of a test. The constant stays because `ORBS_DUMP` still chooses a pane count for an arbitrary `ORBS_GRID`, and *"two panes only above the floor"* is a live decision there |
| `INPUT_ROWS` is a constant, and it is **1** | It used to be derived: a second row bought back the pixel height a finer tier took away, `32s ≤ 16(s+1) ⇔ s ≤ 1`. With one grid there is no tier to compensate for, so a second row stopped being compensation and became magnification — a prompt twice the transcript's size at *every* window and three times again at 4K. Shipped at 2 and reverted on sight, which is what the risk register predicted. It also cost the player half the line: at 2× the text is written into half the columns, so a 120-column grid gave 60 cells to type into and now gives 118 |
| The magnified path has no caller now | `Frame::set_magnified` and the renderer's 2× pass are reachable only by setting `INPUT_ROWS` back to 2. Kept rather than deleted **because that is the whole retreat** — one constant — and not on the grounds of a future consumer: the font-scale setting this owes would change the *grid* (80×30, also on the 8:3 line) rather than magnify one row, so it is not the mechanism that would revive this. If the prompt size is settled, deleting it is the honest follow-up |
| A resize constraint | **Considered and dropped.** `WindowResizeConstraints` is logical pixels while `WindowResolution::new` is physical, so on a 2× display a floor expressed in it would exceed the initial window and the game could not open at its stated size. `paint_too_small` is the guard, and it is physical-unit correct |
| The debt this owes | §4 justified reflow with *"fixed scaling without reflow would make large accessibility font scales unreadable"*, and that objection is correct. The game now has **one text size per window size and no way to ask for another**. The replacement is a font-scale setting choosing between authored grids on the 8:3 line — 80×30 is the large-text one — which is a Phase 11 settings item, not a tier |

`INITIAL_WINDOW` stays 1280×720: it is ×1.0 exactly, so the game opens at native
cell size *and* with visible bars, exercising the letterbox on every run.

### Fidelity tiers — added draft 8 (superseded above)

| Question | Decision |
|---|---|
| How do 4 panes fit legibly? | **Multiplexing raises screen fidelity.** Tier 1 is a coarse, large-text grid (~80×22); engaging multiplex drops the integer cell multiplier one step, roughly doubling cells so 4 full panes fit. Cancelling returns to tier 1 |
| Why integer steps? | Bitmap fonts must scale by whole pixels to stay crisp |
| What about small windows / large font scales? | **Player-chosen display mode** — Deep focus (tier 2 grid) or Wide focus (strips), auto-defaulted and overridable any time. Identical capacity and information in both; progression is never gated on visual acuity |
| Shader impact | Scanline density, RGB mask, and barrel distortion re-derive against the active cell size — the CRT port becomes dynamic, and the zoom is a first-class effect |

### Draft 7 review — resolved (draft 8)

| Finding | Decision |
|---|---|
| Free pane swapping made multiplex capacity worthless | In-flight manual actions **reserve their Focus slot**; manual concurrency = capacity |
| Capacity 4 unreadable at 80×24 | **Fidelity tiers** — multiplexing zooms the orb out one integer scale step, fitting 4 full panes. Strips remain as a player-selectable Wide focus mode |
| Unattended-siege ladder broken and exploitable | Backlog capped at 5, decays over time, clears only at ≥20% completion / dispersal / lapse |
| Mana non-binding in tower nullified the script discount | Discount re-denominated to **duration**; upkeep re-denominated to **Attention** |
| Attention pool saturated by the first script | Range widened 3 → ~25 |
| Starting domains made first-script target unreachable | **Brewing + archive**; scrying becomes the first discovery |
| `verify --all` trivially cheap at 1 Hz | Production-class, duration scales with tower size; `verify <target>` gains a cooldown |
| `undo` had a one-second window | Anchored to the last **command**, ~30s validity |
| Async completions collided with eldritch vocabulary | "Unrequested output" and "missing prompts" retired from eldritch |
| Nuisance ceiling nullified the multiplexing counterweight | Counterweight is **attention upkeep per open pane**, which is uncapped |
| Screen-reader mode broke siege mana | Mana is a **fixed per-siege pool**, no regeneration |
| Atomicity claim no longer true | In-flight actions are first-class serialisable entities; benefit un-banked |
| Offline trace unpicked branch | Accrues offline; capped at one provocation per window, exempt from escalation |
| Economy had supply but no demand | Worked sink table + the ~300-fragment derivation added (**~250 since experience** — §19) |
| Three meters, two names | **Focus** / **Attention** / **Execution budget**, used consistently |

### Economy session — settled (draft 7)

| Question | Decision |
|---|---|
| Soft ending length | 15–25 hours; play continues after |
| Tick rate | 1 tick = 1 real second |
| Action durations | 10s–10min; duration tracks consequence |
| Blocking model | Manual actions *partially* lock their pane; scripts lock nothing |
| Panes | 7, one per domain; 2 at start; unlocked by activity discovery |
| Multiplex capacity | Separate track — 1 → 2 (~1.5h) → 3 (~5h) → 4 (~10h) |
| Attention pool | ~3 → ~10 concurrent script actions, via ley-line and grimoire rank |
| Mana | Binds in sieges only; freely regenerating in the tower |
| Escrow | Scales with completion fraction, floored at 20%; +50% completion bonus |
| Siege cadence | Provoked every 20–30 min at a normal push rate |
| Declining sieges | Always allowed; compounds; 5+ unattended suspends offline progression |

### Draft 5 review — resolved

| Finding | Decision |
|---|---|
| Tick model contradiction (time vs currency) | **World time.** Commands free to issue; scarcity is action *duration* and concurrency |
| Offline clamp made quitting optimal | Clamp removed; narrow Integrity floor only; drift damped identically online-idle and offline |
| `verify --all` deleted the triage decision | `verify <target>` cheap and instant; `verify --all` slow and pane-occupying |
| Sieges had no downside | Escrow: full payout + bonus on completion, reduced fraction on loss/abandon/crash |
| §5.3 rested on a cut-line item | Three trace sources; production heat and scrying depth survive every cut |
| Trace sabotage broke "no forced threat" | Rate multiplier only; adversarial surfaces remain siege-only |
| Eldritch and sabotage shared a signal vocabulary | Vocabularies made disjoint; diagnostic surfaces corruption-exempt |
| Hot-reload laundered sabotage | The program is re-derived from the text on every cast, so an edited line is read afresh and nothing is laundered by being reloaded. (Superseded the ID-annotated canonical form, which is withdrawn — §19) |
| Panes gated four things including automation | Attention decoupled from pane count; synergy requires focused set; aberration superlinear in panes |
| "Capability lost" state unreachable | Row removed; replaced with Budget starved |
| No undo model | One-step `undo` anchored at tick boundaries |
| Disambiguation blocked during sieges | Siege parser takes best candidate, echoes, offers correction |
| Screen reader vs real-time siege | Screen-reader mode advances siege ticks on player input |
| Determinism asserted, not architected | Seeded per-subsystem RNG; single `step(world, tick)` entry point |
| Composition applied to highest-traffic prose | Inverted — author failure text, compose descriptive text |
| CRT port understated | Corrected to 2,638 lines + 242-line shader |
| No schedule | Calendar added; 28 months against a 24-month target, stated plainly |
| Phase 3 overloaded | Split into 3a breadth / 3b remote hosts / 3c engine upgrade *(those three are 9a/9b/9c since the phases were renumbered; this row records what the review said at the time)* |

### Draft 4 questions — resolved

| Question | Decision |
|---|---|
| Bitmap font and cell aspect | CP437-style 8×16; box-drawing native |
| Vertical slice domain | Brewing *and* scrying, both thin |
| Vocabulary size | 45–55 canonical commands |
| Domain verb naming | All three registers resolve; **arcane is canonical** |
| Economy | Model settled draft 6, numbers settled draft 7 — see the economy table above |
| Pane addressing | Named by domain, routed within the focused set |
| Script validation | Bind always succeeds; warn with suggested fix |
| Synergy matrix | All 21 pairs; shared mechanical template, bespoke flavour |
| Price | $4 provisional; revisit Phase 10 |
| Positioning | Duskers + Zachtronics |
| Saves vs achievements | Readable and editable; achievements unguarded |
| Save timing | Tick boundaries only; scripts atomic within a tick |
| Hot-reload | Yes; the text is the truth and the program is re-derived from it (§19 — canonicalisation moved from save to cast) |
| Forgotten bound scripts | Boot report + named in sabotage logs + `verify --all` |
| Remote host risk | Trace scales with time → sabotage pressure → provokes sieges |

### Draft 2 questions — resolved

| # | Question | Decision |
|---|---|---|
| 1 | Siege intervention model | Aberrant events; one-command diagnosis; §5.1 |
| 2 | Font and cell aspect | **Settled (draft 4): CP437-style 8×16** |
| 3 | Phosphor | Curated themes; muted violet default |
| 4 | Boot sequence | Status report; sticky skip; `status` command |
| 5 | Shell verb scope | Unlocked as discoveries; strict pipes with forgiving stages; destruction = maintenance |
| 6 | Vocabulary budget | ~~60–80~~ → **superseded (draft 7): 45–55 canonical** |
| 7 | Naming pass | Before Phase 1 vocabulary freeze |
| 8 | Multiplex synergy | Authored pairs + rising aberration exposure |
| 9 | Difficulty tiers | Progression-gated range, player choice within it |
| 10 | Siege rewards | Trait knowledge + guaranteed drops. ~~Banked continuously~~ → **superseded (draft 6): progress-scaled escrow** |
| 11 | Steam Workshop | Deferred; scripts plain text so sharing stays possible |
| 12 | Release posture | Demo (Phase 10) then full 1.0. No Early Access |
| 13 | Endgame | Soft ending closes the lore arc; play continues |
| 14 | World beyond the tower | Remote hosts as navigable machines |
