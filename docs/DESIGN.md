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
| `sift <pat> <src>` | Filter for matches | `grep`, `find` | search, filter, "look for" |
| `status` | Tower overview (the boot report) | — | overview, "how are things" |
| `grimoire <topic>` | In-world manual | `man`, `help`, `?` | explain, "how do I" |
| `verify <target>` | Detect tampering | `check` | inspect, audit |
| `undo` | Revert the last command | — | revert, "take it back" |
| `meditate <n>` | Fast-forward the clock | `wait`, `sleep` | rest, pass |
| `decoct <essence>` | Brew a potion | — | brew, make, mix, distil |
| `decant <vessel>` | Collect a finished potion | — | collect, take, pour |
| `purge <target>` | Destroy waste or spoilage | `rm` | clean, dump, "get rid of" |
| `decipher <frag>` | Research a fragment | — | study, translate, decode |
| `inscribe <name>` | Author a script | `vi`, `edit` | write, author |
| `bind <script>` | Attach a script to a trigger | `cron` | schedule, automate |
| `invoke <script>` | Run a script or spell | `run`, `exec`, `./` | cast, do |

Every row resolves from all three registers; the echo always shows column one.

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

```toml
bevy = { version = "=0.19.0", default-features = false, features = [
    "std", "async_executor", "multi_threaded",
    "bevy_winit", "bevy_window", "bevy_input_focus",
    "bevy_render", "bevy_core_pipeline", "bevy_sprite",
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
| **0. Vertical slice** | Parser + instrumentation, **brewing + archive (the two starting domains)**, **`orbs-render` Frame boundary**, **log-poisoning sabotage on brewing logs**, cell-grid renderer + fidelity tiers, worst-case CRT legibility test, structured-record output model, seeded-RNG + `step()` determinism spine, boot, scaffold tutorial | ~3k | 4 mo | Numeric gate below |
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
