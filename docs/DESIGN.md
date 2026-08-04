# O.R.B.S. — Operational Relic Bewitching System

**Blackhearth Studios — Game Design Document**
Status: draft 8 · 2026-08-03 · pre-production, no code written

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
in `/alembic`, warding in `/battlements`, spying in `/lens` — and you progress by
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

- **Default: muted violet.** Distinctive, reads arcane rather than computer.
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

  It must also establish **the minimum window at which tier 2 is offered at all** —
  below that, and at high accessibility font scales, the default multiplex display
  mode switches to Wide focus (§9). The player can override either way.
- It hardcodes a 16:9 letterbox and a 1080 scanline reference.

Already built in court_wizard and creditable to §14: `colorblind_correction.wgsl`,
`high_contrast.wgsl`, and an accessibility render pipeline.

### Grid model

**Reflow at integer fidelity tiers, with a declared minimum of 80×22.** Cell size
is an integer multiple of the 8×16 bitmap cell — required for a crisp bitmap font —
and the multiplier is chosen so tier 1 lands near 80×22 on any common window.
Engaging multiplexing drops the multiplier by one step, roughly doubling available
cells (§9). All layout is authored against the 80×22 floor and must be responsive.

Fixed scaling without reflow would make large accessibility font scales
unreadable; this model keeps the floor legible while letting the orb resolve more
detail on demand.

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
alembic ..................... [ ok ]
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
  per second (§11.5), with a **~60s inactivity grace** so pausing to think is
  never punished. Drift and decay rates are slow *per tick*; the clock itself is
  not.
  The grace timer is **not** reset by a pending disambiguation prompt — otherwise
  a player could freeze the tower indefinitely by leaving one open.
- **`wait` / `meditate N` fast-forwards** the clock. It is not a time source.
- **In a siege, ticks advance on wall-clock** without grace.

### Scarcity comes from duration, not from currency

Issuing an action is free; **actions take time to complete.** Brewing, warding,
repairing, deciphering, and summoning all occupy their pane for a duration
measured in ticks.

**Attention is therefore the real scarcity, and it is expressed as concurrency:**
how many duration-actions you can have in flight at once. That is precisely what
focus panes gate (§9), which means the economy and the focus system are the same
system rather than two bolted together.

This gives every downstream claim its teeth without a currency:

- **Nuisances have real decision content.** Repairing the rats occupies the
  alembic pane for its duration, during which you are not brewing. "Handle when
  convenient" is a genuine trade-off.
- **Automation wins** because script actions occupy Attention rather than Focus,
  and because of script-only capabilities and the speed advantage (§8).
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
| Example | Rats infest the alembic; byproduct accumulates; a reagent spoils | Reagents swapped, triggers retimed, glyphs corrupted, logs poisoned |
| Effect | Slows production | Compounds toward failure |
| Surfaces touched | Environmental only — never scripts, schedules, or logs | All four surfaces (§8.1) |
| Response | Repair occupies a pane for a duration | Diagnose and repair under pressure |
| Ignoring | Reduced throughput | Loss |

**Nuisances have real decision content because repairs occupy a pane.** Fixing the
rats blocks the alembic for its duration, so it trades directly against brewing.
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
| **Infiltration** — connecting to enemy hosts, scaling with time and interference | Phase 3 | **No — cut-line item 3** |

Draft 5 rested this entire loop on infiltration alone, which was both a phase
late (sieges land in Phase 2) and a cut-line dependency. Production heat and
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
you type "make a potion of clarity", the orb answers `decoct --essence=clarity`,
and months later you are typing `decoct` by reflex. The interface teaches magic.

```
orbs:~$ make a potion of clarity
  → decoct --essence=clarity

orbs:~$ grep march feed.log
  → sift march feed.log
```

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
| `grimoire <topic>` | In-world manual | `man`, `help`, `?` | explain, "how do I" |
| `verify <target>` | Detect tampering | `check` | inspect, audit |
| `undo` | Revert the last command | — | revert, "take it back" |
| `meditate <n>` | Fast-forward the clock | `wait`, `sleep` | rest, pass |
| `decoct <essence>` | Brew a potion | — | brew, make, mix, distil |
| `siphon <vessel>` | Collect a finished potion | — | collect, decant, pour |
| `purge <target>` | Destroy waste or spoilage | `rm` | clean, dump, "get rid of" |
| `divine <frag>` | Research a fragment | — | decipher, study, translate |
| `scribe <name>` | Author a script | `vi`, `edit` | inscribe, author |
| `bind <script>` | Attach a script to a trigger | `cron` | schedule, automate |
| `invoke <script>` | Run a script or spell | `run`, `exec`, `./` | cast, do |

Every row resolves from all three registers; the echo always shows column one.

**Three canonical names changed in the Phase 0 naming pass** (§19) —
`decant`→`siphon`, `decipher`→`divine`, `inscribe`→`scribe`. Every original stays
in the table as a plain-English synonym, so nothing a player learned stops
working. That is not courtesy: a released word does not stop resolving, it
resolves to whatever it is nearest, and `decant` unclaimed lands on `decoct`.

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
├── alembic/         brewing — reagents, recipes, potions
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
Alchemical byproduct accumulates in `/alembic` and must be purged manually or by a
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
brew --recipe=clarity --qty 2
purge --byproduct --above 60%

if scry --enemy --within 2leagues; then
    alert --priority high
    ward --upon all --emergency
fi

orbs:~/grimoire$ bind night_watch.spell --to dusk
Bound. The orb will remember.
```

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

### Bind-time canonicalisation

**`bind` resolves loose phrasing to canonical commands at authoring time and
stores the canonical form.** A script executes later, in a different world state,
where live-state disambiguation is unavailable. Sound engineering, and a good
diegetic beat: *the orb writes down what you meant.* Loose phrasing remains a
live-prompt affordance only.

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
  with a **call-depth limit of 3**. The Attention pool alone is not a sufficient
  recursion guard: exhausting it makes every subsequent instruction `Budget
  starved`, which logs at high verbosity only, so all automation would stop
  silently. Depth-limiting makes runaway recursion a loud, diagnosable failure.
- **A script action and a manual action may target the same domain
  simultaneously** — they occupy different resources (Attention vs Focus, §9).
  They contend only for reagents and mana, resolved under the resource-contention
  rule above, with the manual action taking precedence: the player's own hand
  wins over the orb's.
- **Attention exhaustion is a visible condition**, surfaced in `status` and in the
  sidebar — never a quiet log line.
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
- **Upkeep is paid in Attention, not mana.** A bound script holds a fraction of
  the Attention pool even while idle, so "what is worth automating" is a genuine
  portfolio decision. Mana upkeep would have been free in practice, since mana
  does not bind in the tower. Explicit from Phase 1.

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
| **Attention** | Automated concurrency — how many script actions can run at once | Shared pool (§11.5) |
| **Execution budget** | Instructions per tick per script | Fixed constant (§8) |

### Layout — the orb resolves more detail

- **The main window** holds every open pane, all **fully rendered and fully
  functional**, up to multiplex capacity.
- **The sidebar** holds every other unlocked pane, minimised to a single line.
  Awareness only, **not commandable**.
- **One input line, always at the bottom.**

**Multiplexing raises the screen's fidelity.** The game begins at a low-fidelity,
large-text grid — a true old terminal. Engaging multiplexing zooms the orb out:
the cell grid gets denser, text gets smaller, and all four panes fit at full
function. Cancelling multiplex returns to the coarse grid. Diegetically, the
wizard focuses harder and the orb resolves more detail.

Fidelity tiers are **integer scale factors** on the 8×16 CP437 cell (§4) —
required anyway, since bitmap fonts must scale by whole pixels to stay crisp:

| Window | Tier 1 (default) | Tier 2 (multiplexed) |
|---|---|---|
| 1920×1080 | 3× → 80×22 | 2× → 120×33 |
| 2560×1440 | 4× → 80×22 | 3× → 106×30 |
| 1280×720 | 2× → 80×22 | 1× → 160×45 |

Tier 1 lands near 80×22 on every common window. Tier 2 at 1080p gives four panes
at roughly **60×15 each** — workable for the dense log scanning a siege demands,
against the ~28×8 a fixed-grid 2×2 layout would have produced.

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

**Commands route by domain name within the focused set.** `decoct haste` reaches
the alembic pane, `ward north` reaches the battlements pane, with no switching
between them — because commands are discrete, one input line serves any number of
focused panes.

**This is what multiplexing actually buys.** At capacity 1, a disaster in the
alchemy lab while you are commanding the battlements forces a choice: swap the
alembic into the main window (losing direct command of the defence) or let it
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
locks" means precisely: a 6-minute brew occupies the alembic's production slot
while a 20-second purge can still run in its triage slot. Only the production slot
consumes Focus.

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

**The multiplexing counterweight is attention upkeep per open pane, not aberration
rate.** Synergies grow O(n²) in open panes (three hold three pairs, four hold six)
so the counterweight must keep pace — but nuisance rate is hard-capped
(invariant 5), so at endgame, where max multiplex meets max trace and max drift,
a rate-based counterweight would stop counterweighting at exactly the point it was
designed for. Upkeep is uncapped and scales cleanly: **each open pane holds a
fraction of the Attention pool**, so multiplexing trades directly against
automation capacity.

**Attention is decoupled from multiplex capacity as a progression track.** The
pool grows via ley-line upgrades and grimoire rank, so capping multiplexing for
legibility reasons (§4, cut-line item 6) does not silently cap the game's core
progression as a side effect of a rendering decision.

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
| **Brewing** | `alembic/` | Sequence/recipe puzzle with timing | Bespoke |
| **Archive** | `archive/` | Decipherment; powers all discovery | Bespoke |
| **Summoning** | `menagerie/` | Resource allocation → autonomous siege units | Derived |
| **Enchanting** | `forge/` | Sequence + resource cost → persistent buffs | Derived |

**Archive is bespoke, not derived** — it gates all discovery, is played most, and
stales fastest. Budget fallback: decipherment becomes mostly a resource sink with
occasional authored set-pieces.

**Scrying is elevated by the aberration model.** Log-parsing is how sabotage is
found. Build it early, alongside the siege prototype.

**On shared engines — build concrete first, extract later.** Draft 3 mandated
generic-engine-first; that is the classic route to the wrong abstraction, because
the generalisation axis is only knowable from the *second* consumer. Instead:
build each bespoke domain concretely and cheaply, and schedule an explicit,
budgeted **extraction task in Phase 3** when the derived domains arrive. If the
derived domains are cut, the extraction is cut with them at zero loss.

## 11. Progression

**Discovery + research.** Commands are found, then understood, then used.

1. **Discover** a fragment — siege, hidden directory, remote host.
2. **Research** it in `archive/`.
3. **Add** it to the grimoire; available to parser and scripts.

Shell verbs, domain verbs, and capabilities (conditionals, loops, triggers,
offline accrual, focus panes) all unlock on this single track.

**Trait knowledge** is recorded in `lens/observed/` on first successful field
identification (§5.2).

**The quiet-tower path plateaus, and the design says so plainly.** Fragments come
from sieges, hidden directories, and remote hosts; hidden directories are finite.
A player who never provokes and never connects will run out of progression. That
is a deliberate stance — pillar 4 promises the calm layer is *safe*, not that it
is a complete way to play indefinitely. The calm layer carries a slow renewable
fragment trickle so the plateau is soft rather than a wall, but the ceiling is
real and the game should not pretend otherwise.

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
| **Time to first bound script** | 30–45 min |
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
  alembic's brewing functions while inspection, reading, and purging stay
  available. A player at capacity 1 is never fully stuck, which matters most in
  the first hour when they have nowhere to switch to.
- **Script actions lock nothing.** This is automation's headline benefit and it
  needs no arithmetic to feel: not faster, not cheaper — *non-blocking*.
- **Not automating is never ruinous**, only slower. Players who want to do it by
  hand can, until they choose to push far enough that they cannot.

### Focus, panes, and attention

| Track | Start | Growth |
|---|---|---|
| **Domain panes** (breadth) | 2 of 7 | One per activity discovered; all 7 by ~hour 10 |
| **Focus / multiplex capacity** (depth) | 1 | 2 at ~1.5h, 3 at ~5h, 4 at ~10h |
| **Attention pool** (automation) | 3 concurrent script actions | **~25** by the soft ending, via ley-line upgrades and grimoire rank |

**The attention pool starts at 3 and must climb steeply.** The `night_watch.spell`
example in §8 issues a ward, a double brew, a purge, and a conditional — three or
more concurrent duration-actions from a single script. A 3-slot pool is saturated
by the player's *first* script, which is the game's headline emotional beat; a
ceiling is the wrong reward for it. At ~6 min average duration, 25 slots is ~250
actions/hour against a manual ceiling of 40/hour at Focus 4 — automation dominates
by roughly 6×, which is what pillar 3 promises. Growth stays stepped and
non-exponential.

Each open pane holds a fraction of the pool (§9), so breadth of attention trades
against depth of automation.

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

**~75 research events across the game** (≈50 commands + ~10 capabilities + ~7
attention steps + 3 multiplex + 5 domain discoveries) at 3–5 fragments each is
~300 fragments.

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

~75 research events over 15–25 hours is **one unlock every 12–20 minutes** — a
healthy drip for an idle game. This is independent evidence that the 45–55
vocabulary budget (§11) and the 15–25 hour length agree with each other; the two
most-argued-over numbers in the document turn out to be mutually consistent.

### Resources

**Ticks are not in this table.** They are world time (§5.0), not a resource. The
scarcity they used to represent is now expressed as **action duration and
concurrency**.

| Resource | Produced by | Consumed by | Role |
|---|---|---|---|
| **Concurrency** | Focus panes (§9) | Duration-actions in flight | The real throttle on attention |
| **Mana** | Passive regeneration, ley-line upgrades | Invocations, script upkeep, repairs | The throttle on action |
| **Reagents** | Brewing, sieges (exclusive tiers) | Potions, enchantments, repairs | Crafting economy |
| **Fragments** | Sieges, hidden directories, remote hosts | Research in `archive/` | Progression gate |
| **Integrity** | Repair, warding | Damaged by sieges, decay, aberrations | Tower health, persistent |
| **Attention** | Ley-line upgrades, grimoire rank (**not** pane count) | Shared pool across all bound scripts | Caps total automation |

**Attention is a shared pool, not a per-script bound.** Draft 5 described both;
they are different mechanisms with different balance behaviour. A shared pool
makes "what is worth automating" a real portfolio decision, which is the intended
texture.

### Intended shape

- **Time to first bound script:** ~30–45 min. The "I taught it to do that" moment
  must land inside the first session.
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
3. **A script-executed action completes in strictly less time than the same action
   issued manually.** Denominated in duration, the currency the economy actually
   runs on — a mana discount would bite nowhere, since mana does not bind in the
   tower, which is where automation lives.
4. **An in-flight manual action reserves its Focus slot for its full duration**,
   regardless of which pane is displayed (§9).
5. No siege can reduce tower integrity below its pre-siege value (§5).
6. Nuisance arrival rate has a hard ceiling regardless of trace, panes, and drift
   (§5.3). The multiplexing counterweight is therefore attention upkeep, not
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
so slippage is measurable rather than discovered in Phase 5.

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
`attend /tower/alembic` navigates the simulated tower exactly as it does under
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
Phase 0, not discovered in Phase 3.

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
     until Phase 3.
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
`bind` is a researched capability, so without it there is no bound script at the
30–45 minute target.

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
  units. Revisit at Phase 4 when content volume is visible.
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
| **1. Core loop** | World clock, script engine + attention pool + failure taxonomy, remaining sabotage surfaces, 3 domains, minimal apprenticeship, content data format, balance CLI **sweeping §11.5's first-pass numbers**, **scrappy internal `orbs-tui` as a dev tool** | ~15k | 5 mo | A player automates a duty and feels clever; non-terminal testers in the loop |
| **2. Siege** | Autobattler, trait composition, adversarial aberrations, escrow economy, **unattended-siege backlog + dispersal**, pane addressing, one siege type, drift stub, synergy template | ~15k | 4 mo | Sieges are tense and scripts visibly matter |
| **3a. Breadth** | All 7 domains, discovery/research, full drift, **offline progression + its unlock**, shared-engine extraction | ~18k | 4 mo | Every domain playable |
| **3b. Remote hosts** | The second content type: trees, verbs, infiltration, trace amplification. **`orbs-tui` to ship quality, if the schedule allows** | ~12k | 3 mo | Infiltration loop closed |
| **3c. Engine upgrade** | Bevy version window — whole-codebase, isolated from new-system work | — | 1 mo | Green on all three platforms |
| **4. Onboarding + demo** | Polish apprenticeship, progressive reveal, grimoire, soft ending, **demo + capsule + trailer** | ~20k | 4 mo | A non-terminal player reaches hour two unaided |
| **5. Ship** | Accessibility, screen-reader siege mode, options, Steam (**both launch options**), polish | ~5k | 3 mo | Release |

**Total: 28 months of work against a 24-month target.** That gap is deliberate and
should be read as the buffer being *already spent* — it is the number to attack
with the cut line, not a scheduling error to hide. §16's "per-phase scope buffers"
mitigation is only real if the calendar exists to measure against.

**Phase 3 is split three ways.** Draft 5 loaded five domains, discovery, full
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

**Ship-quality `orbs-tui`** sits in Phase 3b and is explicitly cuttable. The Bevy
build never waits for it.

**Two things moved into Phase 0.** The naming pass for the slice's 16 commands (now done — §6.1),
because the gate's first-attempt metric substantially measures how *guessable* the
names are, and testing provisional names would invalidate the result. And one
sabotage surface (log poisoning — cheapest and most legible), because §16 claims
Phase 0 de-risks "siege is empty" and it cannot do that with a scrying stub and no
sabotage at all.

**The demo sits in Phase 4** because its purpose is validating onboarding, and a
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
| Onboarding loses non-terminal players | **Critical** | Three layered approaches from Phase 1; continuous external testing; demo in Phase 4 |
| **Writing volume overruns the schedule** | **Critical** | Composition over authorship; per-phase word budget; vocabulary as first lever. court_wizard's speed came from externalised art — unavailable here |
| Economy is wrong or untuneable | High | §11.5 settled with first-pass numbers and worked sinks; balance CLI sweeps them in Phase 1; one headline scarcity per phase — Focus in the tower, mana in the siege |
| Seven domains + remote hosts spreads thin | High | Concrete-first with a budgeted Phase 3 extraction; cut line |
| CRT over text is illegible | High | Phase 0 worst-case test at min resolution and max font scale |
| Sabotage feels unfair rather than solvable | High | Every surface has a structural tell AND a one-command `verify`; log poisoning always leaves a trustworthy source |
| Bevy 0.19 goes stale over two years | Medium | ~4-month cadence means shipping 5–6 versions behind; one deliberate upgrade window in Phase 3, budgeted |
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

**Blocking Phase 1**

1. Trace tuning across all three sources — accrual rates, the nuisance-rate
   composition rule, the ceiling, and the provocation threshold (§5.3).
2. Naming pass for the remaining ~35 canonical commands and their synonym sets,
   following §6.1's ≤7-character rule.
3. Hidden-directory authoring plan — ~80 fragments' worth, placed to pace the
   first ten hours.

**Blocking Phase 2**

4. The 21-pair synergy template: which mechanical parameters it exposes, and the
   per-pair tuning values.
5. Siege type definitions, their failure conditions, **and how completion fraction
   is computed for each.** Elapsed/target works for survival timers; objective
   defence, integrity collapse, and cascading failure each need an explicit
   definition or escrow cannot settle.
6. Difficulty-tier definitions within the progression-gated range.

**Commercial, before Phase 4**

7. Wishlist target, and whether $4 survives contact with actual content volume.
   Note that $4 forfeits discount room (a 50% sale is $2) and may read as a
   smallness signal to the Zachtronics-adjacent audience, which does not
   price-shop — Exapunks is $19.99.

## 19. Decisions log

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
`ORBS_DUMP="attend alembic; decoct clarity; meditate 25"` types a session first,
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
| **You can only name what is where you are** | §7 makes the tree the tower and navigation diegetic, so `decoct clarity` works in `/tower/alembic` and nowhere else. Places stay nameable everywhere — gating movement on being somewhere would be a lock whose key is behind it. This is the base state §19's **pane addressing** later relaxes in Phase 2: acting at a distance has to *become* possible |
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
| `divine` | 12 | §10 makes archive the domain played most and returned to between other work |

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
| Per-subsystem RNG streams | Phase 2: nothing rolls yet |

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
| **Why this is not a Phase 0 rule** | Later phases need it more, not less. Phase 1 asks whether a player *feels clever*; Phase 2 whether a siege is *tense*; Phase 4 whether a non-terminal player reaches hour two unaided. None of those is assertable, all of them are felt, and the phase exit criteria were already written in exactly those terms — the step-level lists simply did not inherit it |
| **The rule** | **No work item in any phase is complete until a person can reach it from the running game.** Every item carries a *See it* line naming the keystrokes. Same standing as the build gate in CLAUDE.md |
| **Retroactive gating is a work item** | Everything already built without a gate gets one, immediately after the prompt and before any new Phase 0 work. Not a cleanup task and not optional: code the game does not call has been asserted, not verified |
| **Where a gate is not yet possible** | Some built code has no honest player surface until later content exists — per-subsystem RNG streams cannot be *seen* until something rolls against them, which is aberrations in Phase 2. Those items name the phase that gates them rather than inventing a debug affordance nobody will maintain |
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
currently corruptible. Decide before the eldritch renderer ships in Phase 2.

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
| Palette | Three phosphor themes, `(Role, Intensity) → Color`. `Presentation` deliberately has **no** entry: it selects a face in the atlas, and a tonal register that existed only as a hue would be exactly what §14 forbids |
| Colours are solved, not chosen | The first pass was picked by eye and **failed its own accessibility tests** — muted violet's cost and success accents were 1.19:1 apart, which is the same colour in greyscale. The shipped values satisfy every constraint the tests assert: body ≥4.5:1 on background, dim ≥3:1, a monotonic intensity ramp, and every accent pair ≥1.25:1 from each other and from body text |
| Verifying it draws | `ORBS_CAPTURE=1 cargo run -p orbs` saves a screenshot after 30 frames. A renderer that cannot be checked without a human at the keyboard is one nobody checks |

### Naming pass — Phase 0, done

Run against the implemented vocabulary rather than by eye, which is what turned
up defects the design table had carried since draft 4.

| Finding | Decision |
|---|---|
| **`decoct` and `decant` collide.** Two edits apart, scoring 667 against a 600 threshold, and they are the two core verbs of brewing — a Phase 0 domain. In a siege §6 forbids a blocking prompt, so a near-typo would be resolved by the parser's best guess: brewing when the player meant to collect | **`decant` → `siphon`.** Alchemically exact, six characters, zero collisions. `decant` is **kept as a plain synonym** — releasing it would be worse than the collision, because an unclaimed `decant` resolves to `decoct` |
| **`dec` prefixed three verbs** — `decoct`, `decant`, `decipher` — so the natural abbreviation for the brewing domain meant three different things | **`decipher` → `divine`.** Also clears a length violation. No three-character prefix reaches more than one **canonical** name. Across *synonyms* `dec` still reaches three verbs, because the old words are deliberately kept; that prompts, which is the right answer for a genuinely ambiguous abbreviation. `aut` and `ins` are ambiguous for the same reason. All three are pinned by test |
| **Four canonical names exceeded the ≤7 rule**: `grimoire`, `meditate`, `decipher`, `inscribe` | **`inscribe` → `scribe`** (same root, same meaning, two characters shorter) and `decipher` → `divine` as above. **`grimoire` and `meditate` are kept**, and the ceiling is codified at **8**: they are the two most in-world names in the set, abbreviation covers the typing cost, and §6.1 already wrote the rule as "ideally" |
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
| Assets | Both font licences survive the combination. CC0 imposes nothing; Spleen's BSD-2 notice requirement **persists** and still has to reach the shipped build (Phase 5) |
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
| Obligation | Binary redistribution must reproduce the copyright notice *"in the documentation and/or other materials"*. The shipped build therefore carries a third-party notice — naturally a `grimoire licences` topic, since every screen is terminal content. **Phase 5 ship task**; the obligation only attaches on distribution |
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
| Places answer to leaves | `attend alembic` reaches `/tower/alembic`. §7 says paths are places, and players say the place |

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
| Split of costs | **Boundary** (`orbs-render`, Phase 0) is non-negotiable and ~free. **Dev-tool TUI** (Phase 1) has no parity obligation. **Ship-quality TUI** (Phase 3b) is cut-line item 3 |
| What it is | A full-screen raw-mode application (à la the Ubuntu Steam installer). Terminal as framebuffer + keyboard, nothing more |
| What it is **not** | It does not shell out, touch the real filesystem, or interoperate with the host shell. Same sim, same commands, same simulated tower |
| Architecture | New `orbs-render` crate owns the `Frame` (cell buffer + semantic styling + layout). **`orbs-render` decides what appears and where; frontends decide only how a cell is drawn.** Frontend-only enrichment (CRT, audio) may never carry information absent from the Frame |
| Colour | Indexed ANSI 0–15; inherits the user's terminal theme. Phosphor themes are Bevy-only |
| Glyphs | User's terminal font; Unicode U+2500 box drawing instead of CP437 |
| Not present in TUI | CRT effects, fidelity tiers, embedded bitmap font |
| Sabotage tells | Glyph/alignment cues are Bevy-only; `verify` is authoritative everywhere (§8.1) |
| Schedule | Boundary Phase 0 (cannot be retrofitted), dev tool Phase 1, ship quality Phase 3b |
| Cost | Boundary is ~free. Dev-tool TUI absorbed in Phase 1. Ship-quality TUI sits in Phase 3b and is cuttable |
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

### Fidelity tiers — added draft 8

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
| Economy had supply but no demand | Worked sink table + the ~300-fragment derivation added |
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
| Hot-reload laundered sabotage | ID-annotated canonical form; only changed lines re-resolve by name |
| Panes gated four things including automation | Attention decoupled from pane count; synergy requires focused set; aberration superlinear in panes |
| "Capability lost" state unreachable | Row removed; replaced with Budget starved |
| No undo model | One-step `undo` anchored at tick boundaries |
| Disambiguation blocked during sieges | Siege parser takes best candidate, echoes, offers correction |
| Screen reader vs real-time siege | Screen-reader mode advances siege ticks on player input |
| Determinism asserted, not architected | Seeded per-subsystem RNG; single `step(world, tick)` entry point |
| Composition applied to highest-traffic prose | Inverted — author failure text, compose descriptive text |
| CRT port understated | Corrected to 2,638 lines + 242-line shader |
| No schedule | Calendar added; 28 months against a 24-month target, stated plainly |
| Phase 3 overloaded | Split into 3a breadth / 3b remote hosts / 3c engine upgrade |

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
| Price | $4 provisional; revisit Phase 4 |
| Positioning | Duskers + Zachtronics |
| Saves vs achievements | Readable and editable; achievements unguarded |
| Save timing | Tick boundaries only; scripts atomic within a tick |
| Hot-reload | Yes, with re-canonicalisation |
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
| 12 | Release posture | Demo (Phase 4) then full 1.0. No Early Access |
| 13 | Endgame | Soft ending closes the lore arc; play continues |
| 14 | World beyond the tower | Remote hosts as navigable machines |
