# Seeing it — every surface, and how to reach it from the running game

**This is the reference half of CLAUDE.md's *"Seeing it"* discipline.** That rule
— *work is not done when it compiles, it is done when it has been looked at* —
stays in CLAUDE.md, because it governs how a session works. What is here is the
accumulated *evidence*: for every domain, surface and instrument, the command
that shows it and what should appear when it does.

**It was in CLAUDE.md and grew to 174k characters**, which is nine tenths of a
file loaded into every session whether or not the work touches a siege. Splitting
it costs one thing and it is worth naming: this is no longer in front of you by
default. **Read the section for whatever you are about to change**, and treat a
See-it line here exactly as if it were still in CLAUDE.md — it is not
supplementary, it is the gate.

Nothing was reworded in the move. Every line below is what CLAUDE.md said.

## How to use it

| You are touching | Read |
|---|---|
| a domain — laboratory, archive, lens, sanctum, menagerie, bailey | that domain's section |
| the spell language, the editor, or highlighting | *A spell's verb is read at cast*, *A spell is highlighted*, *The satchel* |
| a duration, a rate, or anything the economy touches | *`orbs-balance`* — and **run a sweep**, which is the instrument that has caught what tests did not |
| a painter, a record, or the way something reads | *The output style*, *The terminal build* |
| accessibility, colour, or the tube | *Greyscale* |
| the manual, `recall`, or a room's primer | *`recall apprentice`* |

**Three instruments recur and are worth knowing before you need them.**
`scripts/dumps.sh` captures every surface as text and is the gate for a refactor
whose claim is that nothing changed; `scripts/play.sh` plays the game at a real
keyboard; `ink` is the only thing that can see a colour in the terminal build.

---

### The world sabotage surface — a reagent that is not what it says

§8.1's second of four surfaces. A base reagent is **substituted**: its name
changes and its identity does not, so a spell that named it stops working.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_swap; survey dispensary; \
  verify dispensary; verify laboratory" cargo run -p orbs
```
```text
charcoal- = ∞   rock-salt = ∞   sage = ∞     ← the odd one out, on screen
verify dispensary tampered something here is not what it says: charcoal-
verify laboratory sound                      ← one level, deliberately
```

**Endless base stock only, and never fuel.** A swap that could reach
`ground-sage` mid-pipeline destroys work in flight rather than misdirecting — and
it announced itself by breaking a *pipeline* test that has nothing to do with
sabotage, which is how a nuisance that reaches too far shows up. Charcoal was the
same mistake wearing the endless flag: it is named by no recipe, so swapping it
stops every heated stage in every domain rather than one spell.

**`debug_swap` above is the tester's shortcut, and the *ambient* half is what
breaks.** The system runs at one swap an hour and no dump can reach it, which is
how three defects shipped with every See-it line and every test green. They are in
DESIGN.md §19; the gate is a sweep and a test, and the sweep is the one that found
them:

```bash
cargo test -p orbs-sim --test tampering        # settles; never takes the fire
cargo run -p orbs-balance -- sweep --ticks 7200
```

**A swept rate is the only instrument that sees an ambient nuisance at all.** The
one that shipped took the alphabetically-first endless pile — always `charcoal` —
and never expired, so clarity read **0.074 against its pinned 0.140** and a
standing grind loop died for the rest of the session. `--ticks 7200`, not
`--hours 1`: an hour is short enough that one badly-timed swap flags the seed.

**A lie settles back to the truth on its own, and that is a fact about the spell
language.** A spell names things with literals, so it can never `purge sage-` — a
word nobody knew when it was written. `verify` → `purge` is therefore human-only,
and without an expiry an unattended tower had no path back at all. `purge` still
repairs it instantly, which is what keeps finding one worth doing.

**`drift` and `substitution` are two systems, not one branch.** Both draw once
per tick from `RngStream::Threat`, so interleaving them would make which surface
is hit depend on prior draws and invalidate every existing replay. The second is
**appended** to the schedule, never inserted — the same rule a stream index has.
`settling` is appended after both and draws **nothing**: it is a clock reading, so
it cannot perturb the stream. A second draw taken only *when* a swap fires would —
which is why the pile is chosen from the quotient of the roll that already fired.

### `verify` has two forms, and looking is what costs

**Bare is the audit and there is no flag.** §8.1 writes it `verify --all`; the
parser has no flag syntax at all, so the widening is `Slot::optional` — the same
bare-widens shape `survey` and `recall` already have. **Bare-widens is this
game's idiom; reach for it before inventing a second grammar.**

**The audit is Production-class**, so it hangs on `/tower` and answers to the
same tower-wide `CAPACITY` a grind does. An audit means you are not brewing, and
that is the price §8.1 sets rather than an accident of implementation.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; verify; grind sage; meditate 25" cargo run -p orbs
```
```text
the orb turns its attention on the whole tower. 21 ticks
the tower is busy verifying. wait for it, or stop it   ← the grind, refused
name: verify, state: tampered, message: these are not what they say: sanctum.log
```

**`/tower`, never `tower::root`.** The filesystem root is deliberately nameless,
so hanging the run there made the refusal read *"the  is busy verifying"* — a
sentence naming nothing, which is the one thing it exists to avoid.

**The cooldown rations a *surface*, never a target.** One look tires the orb of
logs — all of them — and leaves shelves untouched. That is what makes *which
surface do I inspect first* a decision; per-target would be no rationing at all
with four targets to hand.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; verify laboratory.log; \
  verify laboratory.log; verify dispensary; meditate 21; verify laboratory.log" cargo run -p orbs
```
```text
name: verify, source: laboratory.log, state: sound
the orb is still reading the log of things. 19 ticks   ← the same surface
name: verify, source: dispensary, state: sound         ← the other one, untouched
name: verify, source: laboratory.log, state: sound     ← and back, after the wait
```

**`State` is the verdict and the surface rides `Kind`.** The refusal wrote the
surface into `State` first, where `sound` and `tampered` live, which made a
refusal and an answer indistinguishable to `sift`, to §14 and to any test
counting verdicts. Rule 4, and it was caught by a test counting two answers where
one had been given — **nothing on screen says which field a word came from.**

**The refusal is `Role::Cost`.** A bound spell verifying in a loop is told to
wait without latching a fault on the rail, which is `say_blocked`'s choice for
`say_blocked`'s reason: a wait is the loop working.

**It travels in the save, and the save shape is named pairs.** A cooldown cleared
by quitting is not a cooldown. The obvious `Vec<Option<u64>>` positional on
`Surface::ALL` **does not serialise at all** — TOML has no null and `toml`
answers `unsupported None value` — so it is `[["log", 120]]`, which is also the
readable file §15 asks for and cannot be renumbered by Phase 8's two remaining
surfaces.

```bash
cargo test -p orbs-sim --test auditing     # the game: both forms, the rationing, the save
cargo test -p orbs-sim --lib tower::audit  # the model: the curve and the arithmetic
scripts/play.sh auditing::                 # three scenarios, through a real tmux game
```

### The bailey — a siege on the seven dice, and the enemy attacks the automation

**`defend` lets an enemy arrive; `hold` ends your turn and resolves one round.**
Between the two, `deploy <troop>` and `quaff <potion>` spend the arsenal, and
scrolls keep `wield` because §19 already spends that word on setting a thing
going. All of it is instant except `hold`, which is **the only thing in the
domain that advances the world** — §5.0's *"no per-command tick cost"*, kept.

**It is not an eighth domain.** §10 fixes the count at seven and lists them; the
bailey is the *arsenal's* shape — a real place in the tree, its own log and
verbs, deliberately not a rail box. The rail's seven fixed slots are the argument.

**It takes no production slot**, which this domain needs most: a siege exists to
test the automation, so freezing it would leave the enemy nothing to attack.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; survey enemy; survey garrison; hold" cargo run -p orbs
```
```text
5 come up the road. they mean to onslaught   ← telegraphed, a round ahead
foes = 5   massed   troops = 5   vigour = 15
troops = 6   vigour = 18                     ← the king's contingent. always six
they onslaught. you lose 2 and take 2
```

### The dice are placed, and that is the decision

**Three dice, four parts of the wall, and a pool that pays for them.**
`pledge d20 to buckler` puts one behind something; the die is **rolled when the
round comes**, not when it is pledged, so the board prints its range first —
§5.1's odds-before-the-commitment rule carried from a roll to an allocation.

**Pledging costs quintessence** — §11.5's mana, built at `0.8.15`. A fixed pool
granted on `defend`, sized by the tower's integrity (halved at nothing, whole at
`STANDING`) and raised by the ley line, and **it never regenerates**: §14's rule,
because the screen-reader mode advances siege ticks on player input, so a
per-tick regen would mean *more typing produces more*. A die costs `faces / 4`,
floored at one — `d6` 1, `d8` 2, `d20` 5 — so a full allocation is eight a round
against a base pool of 24. **Three unrestrained rounds of a siege that runs six
to thirteen**, which is what makes leaving a row dark a move.

**Declining to pledge costs nothing.** That is the whole shape: the question
stopped being *which row* and became *is this row worth five*.

```bash
# The pool spending down, and the refusal when it is gone. Three full rounds is
# exactly 24, so the fourth has nothing.
ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; pledge d20 buckler; \
  pledge d8 line; pledge d6 succour; survey coffer" cargo run -p orbs
#   d20 goes behind the buckler for 5. 1 to 20   ->   quintessence = 16
#   ...and after three rounds: d20 would take 5, and you hold 0
```
```text
coffer    d6 1  d8 2                     19    ← each die's price, then what is left
```

**A worn tower gets half a pool**, which is §19's deferred *integrity → siege*
coupling finally taken. The floor is what stops a lost siege spiralling into an
unwinnable next one — §11.5's *"never ruinous, only slower"*.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend sanctum; meditate 3600; attend bailey; defend; \
  survey coffer" cargo run -p orbs      # quintessence = 12, against a kept 24
```

**The coffer publishes only the dice it can pay for, and that is a language
decision rather than a convenience.** The obvious positive sentence is *wrong*:
`watch.rs`'s comparison against another place is **strict**, so `if the coffer has
more quintessence than the d20` excludes the die you can exactly afford, and there
is no *at least as many* comparative. Only the negative spelling is correct. So
affordability is a derived word — the maze's `spoil`/`exit` pattern — and
`if the coffer has d20` means *"I hold it and can afford it"*.

**Both shipped bailey solvers needed no change**, because both already guarded
that way. The numeric comparison still works for real weighing:

```bash
ORBS_BOOT=0 ORBS_GRID=110x36 ORBS_DUMP="attend bailey; scribe thrift" \
ORBS_EDIT=$'edit\nif the enemy has fewer than 40 aim\nhold\nend\nif the coffer has fewer quintessence than the d20\nhold\nend\n<esc>\ninterpret' \
  cargo run -p orbs
#   2 if enemy has 39 or fewer aim
#   5 if coffer has fewer quintessence than d20
```

**`aim` is the odds, and it closes an asymmetry §5.1 did not know it had.**
`Roll::chance` was called only from `Siege::view`, so a hand player could read
`60%` off the board and a bound solver could ask nothing — *show the odds before
the commitment* was true for eyes only. It is published on both bands now.

**`quintessence` is a tolerated naming collision, with its cost written down.**
It shares `qui` with `quickening-scroll`, which is spent *at the same wall* — the
first tolerated collision that shares a room. `wield` is `Workable` and rejects a
`Sense`, so the scroll keeps its abbreviation; `purge` and `verify` take `Any` and
would pick the reading. **§11.5's own `mana` is worse** (750 against `many`, which
is inside `as many … as`) and so is `power` (800 against `tower`).

| area | what it does | when it is worthless |
|---|---|---|
| `line` | added to every attack you make | a volley — you do not swing back |
| `buckler` | added to what the enemy must beat | never |
| `succour` | mettle put back at the end of the round | at full strength |
| `sortie` | mettle spent for damage now | when you cannot afford it |

**The board can never be covered**, which is the whole shape: one row is always
dark and choosing which is the turn. And **each intent makes a different area
urgent**, which is what turns the telegraph from advice into the thing the turn
is spent on.

```text
┌ rampart ──────────────────────────────────┐
│they mean to volley                        │
│line      --               moot            │   ← the warning, *before* you pledge
│buckler   d20           1 to 20            │   ← the gamble, and its range
│succour   d6             1 to 6            │   ← the floor
│sortie    --                               │   ← the one left dark
│enemy     ████████████████████  24/24   50%│
│garrison  ████████████████████  18/18   50%│
│coffer    d8                               │   ← what is left to pledge
└───────────────────────────────────────────┘
```

**Where variance is cheapest is the question, not how much.** A low roll in
succour merely heals less; a low roll behind the buckler is a round wasted. That
is what a *set* of dice buys over three of a kind.

**What a spell reads:** `ceiling` on each area, `moot` where the intent will
waste it, the free dice by name on the `coffer`, and `for each area` to walk the
four. `steadfast` reads the telegraph and allocates by it; `warding_off` walks
the areas generically.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; pledge d20 buckler; \
  pledge d6 succour; survey coffer; hold" cargo run -p orbs
ORBS_BOOT=0 ORBS_DUMP="attend bailey" ORBS_THEN="invoke steadfast; meditate 900" cargo run -p orbs
```

**At three dice, allocating *at all* matters far more than allocating well** —
measured, the two solvers tie on seventeen seeds while *no* allocation loses seed
3 outright. That is a tuning signal rather than a conclusion.

**`mend` could not heal a wounded band, and `quaff mending` was a third of what
it read as.** It capped at `count * VIGOUR` where `wound` derives `count` back
down from `vigour`, so the cap *was* the current strength. Surfaced by `the
succour rolled 4 and put back 0` on screen; the matrix test had passed throughout
because the number moved by one.

**The board draws whenever a siege is running**, not when a word is typed — the
map's rule, the sheet's and the sanctum's. **Strength is bar *length*, never
colour** (§14): the domain *is* a comparison of two quantities, so a hue would
make it invisible in greyscale and in a dump.

```text
┌ rampart ──────────────────────────────────┐
│they mean to onslaught                     │   ← what the turn is spent answering
│enemy     ████████████░░░░░░░░  16/27   60%│   ← the odds, before the commitment
│───────────────────────────────────────────│
│garrison  ████████░░░░░░░░░░░░   7/18   50%│
│round 5, 6 still coming                    │
└───────────────────────────────────────────┘
```

**Show the odds before the commitment and the roll after it.** That is XCOM's
bargain and the rule that keeps a rolled outcome fair — a decision under known
risk is a decision; a surprise is not. `Roll::chance` composes without drawing,
which is what lets the board print it.

**The seven dice are D&D's**, `d4` through `d100`, and a roll is **composed then
resolved**: assembled as a value — die, modifiers, target — and only then drawn.
**This is the piece that cannot be retrofitted.** With the draw at the call site,
every call site has to change to admit a modifier, and there is one per kind of
attack. `d100` is one die, not the percentile pair: that pair exists because
nobody manufactures a hundred-sided solid, and copying it would spend two draws
for one number.

**The draw count is a function of the composed roll, never of the outcome.** A
reroll-on-miss would make the stream depend on its own results, which is §19's
`drift` defect. **A volley draws the garrison's dice and throws them away** for
the same reason — skipping them would make the number of draws depend on the
intent.

**Every roll is in the log with its die and its face**, which is rule 4 doing its
job: `peruse bailey.log` is a genuine postmortem, and a player can see which
potion earned its place.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; hold; peruse bailey.log" cargo run -p orbs
#   enemy strikes - d20 gives 12 against 11, and it tells
```

### The arsenal is worth what your industry is worth

**How much help a thing gives is matched to the *rate* you make it at**, not to
how many you hold. Three words: `fresh` at full strength, `thin` at half, `spent`
and refused.

```bash
ORBS_BOOT=0 ORBS_DUMP="debug_spawn warding 5; survey arsenal; \
  meditate 1200; survey arsenal; meditate 1800; survey arsenal" cargo run -p orbs
#   name: warding, qty: 5, state: fresh
#   name: warding, qty: 5, state: fresh
#   name: warding, qty: 5, state: spent
```

**`debug_spawn` stamps a *full* store**, not one making — its own sentence is
*"the shelf finds it had it all along"*, and the industry behind the shelf is part
of *all along*. A single stamp made a spawned troop bring half its bodies, and a
test of the Ley Line's garrison grant failed on the arsenal's freshness rule
instead of on the grant.

**Read the `qty` column, not the state one.** It is 5 the whole way through.
**Total stock never enters the arithmetic** — that is what makes this buildable
where a perishable arsenal was not, and it is why hoarding cannot beat it: a
thousand wardings and five wardings are the same store if you have made the same
number lately.

**A spent store is refused, and the potion is kept** — the rule `spend_whole`
already sets in this room:

```bash
ORBS_BOOT=0 ORBS_DUMP="debug_spawn warding 1; meditate 3000; attend bailey; \
  defend; quaff warding; survey arsenal" cargo run -p orbs
#   your warding stores are out. make one and come back
#   name: warding, qty: 1, state: spent      ← still on the shelf
```

**Making something that is already `fresh` mints double renown** — the surplus is
sold rather than shelved, so there is never a point at which producing stops being
worth it. That is visible in a sweep rather than in a dump:
`cargo run -p orbs-balance -- sweep --ticks 7200` shows the production policies'
`renown` column roughly doubled while every `xp/tick` is untouched.

**It does not thin while the game is closed**, because a tick is *"one real second
while the window is open"* and nothing catches up. When offline progression lands
and advances the tick, it will — with no change to any of this.

**And a spell can ask which store has run down**, which is the whole answer to the
mechanic — the reply to a thinning arsenal is automation, not vigilance:

```
for each store
    if the store has spent
        ...go and make one
    end
end
```

**`fresh`, `thin` and `spent` are all askable**, and `spent` is the one a keeping
spell acts on. `thin` is the middle of the slope and is reached by production
*spread over time* — one `debug_spawn` cannot make it, because it stamps a full
store at a single tick and those makings all fall out of the window together.

```bash
cargo test -p orbs-sim --test scripting_the_siege a_spell_can_walk
```

Two lookups had to learn about the arsenal for that to work, and both are
Cwd-scoped everywhere else: `group_at` (which set does `for each` walk) and
`watch::find` (what does `has` resolve against). §19's exemption is *"the one room
reachable from every other"* — a spell that can **say** `warding` anywhere but only
**ask** about it in one room is that decision half-built.

### Fame draws a crowd, and `petition` is how you are less famous

**How many come up the road is decided by what the tower is worth.** The floor
never moves — five, at every standing — and what fame lengthens is the *tail*.
So a famous tower can still draw a quiet night, and an unknown one never meets
the worst.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend" cargo run -p orbs
#   5 come up the road. they mean to onslaught

ORBS_BOOT=0 ORBS_DUMP="debug_renown 9000; attend bailey; defend" cargo run -p orbs
#   6 come up the road. they mean to onslaught
```

**Twelve is a written ceiling.** The garrison is six and `outnumbered` is a
*ratio* — `enemy >= garrison * 2` — so twelve is exactly where that reading turns
over at the opening. Past it, the rung three shipped solvers branch on would be
true in every fight.

**`petition` spends standing to shorten the tail**, and it lowers the *ceiling*
rather than subtracting after the draw — at the moment you type it there is no
siege and nothing has been drawn:

```bash
ORBS_BOOT=0 ORBS_GRID=120x45 \
  ORBS_DUMP="debug_renown 9000; attend bailey; petition; petition; petition; defend" \
  cargo run -p orbs
#   word goes out. at most 11 will come, for 7 renown
#   word goes out. at most 9 will come, for 7 renown
#   word goes out. at most 8 will come, for 7 renown
#   5 come up the road. they mean to onslaught
```

**Read that second line again — it is the mechanic explaining itself.** The
ceiling falls 11 → 9 → 8, not 11 → 10 → 9, because the first payment cost a rank
on the way and a lower rank draws a shorter tail on its own. **`petition` retires
itself** rather than becoming a tax on every fight.

Both refusals are free and name their numbers, which is `pledge`'s rule in this
room: `that would take 7 renown, and you hold 0`, and at the bottom
`no fewer than 5 will ever come. keep your renown`.

### A fight moves your standing, and only says so once

**Renown moves on every exchange and the round does not stop to mention it.**
A round is elected by typing `hold` and has just said what the enemy did, so a
sentence per exchange would be the same news twice over, six to thirteen times a
fight. What moves is `(dealt + sortied)` against `(taken + spent) − mended` —
**`mended` is in there because otherwise healing is punished**, and quaffing a
`mending` is the domain's headline move.

**It must pledge to show anything.** An unpledged round trades about evenly and
nets nought, which is why this looked broken before it was written this way:

```bash
ORBS_BOOT=0 ORBS_GRID=120x45 \
  ORBS_DUMP="attend bailey; defend; status; pledge d20 to buckler; hold; status" \
  cargo run -p orbs
#   renown .........   0
#   they onslaught. you lose 1 and take 3      ← no renown line between
#   renown .........   2
```

**The fight speaks once, on settling**, and the number is the *whole* fight —
rounds and outcome — because it is measured from what the tower was worth when
the enemy came up the road:

```bash
ORBS_BOOT=0 ORBS_GRID=120x45 \
  ORBS_DUMP="attend bailey; defend; debug_siege; pledge d20 to buckler; hold; status" \
  cargo run -p orbs
#   the enemy breaks and runs. the wall holds, and you earn 105
#   they are singing about it: 37 renown, 37 in all
```

**And a loss costs, scaled by how far short the wall fell.** A spell's records go
to the log rather than the pane, so this one is `peruse`d:

```bash
ORBS_BOOT=0 ORBS_DUMP="debug_renown 200; attend bailey" \
  ORBS_THEN="invoke besieging; meditate 900; peruse bailey.log" cargo run -p orbs
#   defend rampart fallen 40  the wall is carried. 40 of the way, and you keep 28
#   word gets about: the wall cost 31 renown, 169 left
```

**A fresh tower shows none of this**, and that is the floor rather than a bug:
renown saturates at nought, so there is nothing to lose until there is. `debug_renown`
is how to stand somewhere else — every one of the ten ranks sits behind an hour or
a day of play.

**A scroll spent on the wall reaches the wall.** §19 keeps `wield` for scrolls
rather than giving them a fourth verb — a player who has spent one in the archive
spends one here without learning anything new — so `pipeline::wield` asks
`defend::wielded` before anything else. **Only while a siege is running and only
for a scroll `siege.toml` names**, so `wield gleaning-scroll` in the archive and
`wield mortar_and_pestle` anywhere are untouched.

Without that the three scroll rows were dead content, and the failure was worse
than inert: `wield quickening-scroll` in the bailey quietly hurried the
*laboratory*, while `quaff quickening-scroll` refused and pointed the player at
the word that did the wrong thing.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; debug_spawn verdant-scroll; \
  survey garrison; wield verdant-scroll; survey garrison" cargo run -p orbs
#   spears 6 -> verdant-scroll burns away on the wall -> spears 9
```

**What the arsenal is worth is authored** in `crates/orbs-sim/content/siege.toml`
(rule 6), so balancing is a content edit. An item with no entry cannot be spent at
all, which is what stops `quaff sage` resolving and doing nothing quietly — and
the wrong word names the right one.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; debug_spawn troop 2; \
  deploy troop; quaff troop; deploy sage" cargo run -p orbs
#   troop goes down to the line
#   troop is not quaffed or deployed. deploy it
#   sage is worth nothing on a wall
```

**`debug_siege` leaves the enemy one round from breaking** — `debug_course` one
room over, and it republishes for the same reason: a siege's readings are
rewritten by the *round* that follows, so a stale board is what a decision tree
would read.

**`besieging` is the decision tree**, on the grimoire's shelf in a debug build.
Every rung is a word the world publishes, because the language has no arithmetic
and is not getting any:

```
repeat until the enemy has routed
    if the garrison has few
        deploy troop
    else if the garrison has hurt
        quaff mending
    else if the enemy has outnumbered
        quaff warding
    end
    hold
end
```

**`hold` is outside the ladder and runs every lap.** Inside a branch it would end
the turn only on rounds where something was spent, so a siege where nothing is
wrong would never advance — and **`repeat until the enemy has routed`, never `is
empty`**: a band publishes `routed` when it breaks and counts while it stands, so
it is never childless and `is empty` would be false for ever. That is §19's
`repeat until the circle is idle` in a third costume.

### The forge — Lights Out on three columns, and charms that decay

**`imbue <tool> <charm>` opens a lattice; `snap <column>` turns a glyph and its
neighbours; `anneal` lets it fall.** A charm binds only when every glyph is lit.
Only `anneal` takes the tower's one production slot — §10's scarcity for this
domain is *"the buff's own lifetime, **and the slot**"*, so this is the first
room since brewing that competes with brewing.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend forge; imbue mortar_and_pestle hurried; snap belt" cargo run -p orbs
```
```text
┌ lattice ──────────────────┐
│apex     belt     hem      │   ← the three words `snap` takes
│    ·        ·        ·    │
│    ·        ☼        ·    │   ← the grid, as your snaps have left it
│    ☼        ☼        ·    │
│───────────────────────────│   ← the rule. below it is what the last fall left
│    ☼        ·        ·    │   ← the residue, and the whole signal
│hurried, 0 spent           │
└───────────────────────────┘
```

**The residue is the puzzle.** A Lights Out solution is settled entirely by its
top row, and the eight openings leave **eight distinct residues** — measured over
all 512 boards, along with the two facts a solver rests on: every board has
**exactly one** answer, and the residue-to-answer table is **universal**. So the
bottom row tells you which columns to snap, if you know the table.

**Three columns and not nine cells, and the sweep decided that.** Nine unique
place names are not available — `crown` scores 800 against `brown`, `base` 750
against `bare`, `tier` 600 against `tower`. Since the top row settles everything,
columns were all a player ever needed to address.

**A column publishes `lit` and never `dark`.** Absence is what dark means, which
is the tower's own idiom — and `dark` scores 750 against the maze's `marks`.

```bash
# The whole loop, and the payoff: a grind is eight ticks and lands in four.
ORBS_BOOT=0 ORBS_DUMP="attend forge; imbue mortar_and_pestle hurried; \
  snap apex; anneal; meditate 25; attend laboratory; grind sage; meditate 4" cargo run -p orbs
#   every glyph holds. hurried settles onto the mortar_and_pestle
#   the mortar_and_pestle yields ground-sage, and leaves husks  +1

# ...and it decays. Past the charm, the same grind takes eight again.
ORBS_BOOT=0 ORBS_DUMP="attend forge; imbue mortar_and_pestle hurried; \
  snap apex; anneal; meditate 700; attend laboratory; grind sage; meditate 4" cargo run -p orbs
```

**The lookup table lives in the spell, and that is the point.** Publishing the
answer would be the orb solving it for you, which the lens refuses one room over;
publishing the *state* and letting the player hold the rule is §8.1's *"a rule,
not a memory"*. Eight rungs, against `threading`'s twenty-four.

```bash
# `forging` reads the residue, snaps the one right column, and binds in one fall.
ORBS_BOOT=0 ORBS_DUMP="attend forge" \
  ORBS_THEN="invoke forging; meditate 120; peruse forge.log" cargo run -p orbs
#   the glyphs rise for hurried. 6 to bind it
#   apex turns, and its neighbours with it
#   every glyph holds. hurried settles onto the mortar_and_pestle
```

**What made that writable was checked before the design was committed to**:
`Condition::All` already chains across independent subjects, so `if the apex has
no lit and the belt has lit` parses. `mortar is idle and flask is idle` has
round-tripped since the language was built.

**Five charms, each read at a different site**, which is why this is a
composition rather than the `if` `tower::dice` warned about:

| charm | what it does | read at |
|---|---|---|
| `hurried` | work takes half as long | `work::begin` — with the `quickening-scroll`, one call site |
| `fruitful` | a run yields one more, and no more byproduct | `work::produce` |
| `bountiful` | a walk of the stacks pays two fragments | `execute::research` |
| `whetted` | the garrison rolls a bigger die | staged as a `Modifier`, like a potion |
| `shielded` | a sabotage strike lands and is turned aside | `sabotage`, `assault` |

**`shielded` skips *after* selection, never by filtering.** All three sabotage
surfaces choose by modulo over a collection, so removing a charmed node from the
pool would change which node the same roll hits — every seed's world, moved.
Letting the strike land and be turned aside leaves the arithmetic exactly as it
was.

```bash
# The maintenance spell — `ebbing` is the rung, and `has no graced` is the
# cold start. Bound, it keeps a charm alive unattended.
ORBS_BOOT=0 ORBS_DUMP="attend forge" \
  ORBS_THEN="invoke tending_forge; meditate 200; peruse forge.log" cargo run -p orbs
```

```bash
cargo test -p orbs-sim --lib tower::lattice   # the 512 boards, and the three proofs
cargo test -p orbs-sim --lib tower::charm     # the interval, and the clock
cargo test -p orbs-sim --test enchanting      # every charm reaching its number
```

### Quintessence is the tower's, and the siege is one of two rooms spending it

**§11.5's mana, at the scope the design always had it.** Its resource table reads
*"produced by passive regeneration, Ley Line steps"*; the fixed pool granted on
`defend` was Phase 8's narrowing, and Enchanting spending the same resource
brought it back up to the tower. §19 records it as the return it is.

**Integrity and the ley line set the ceiling** — how much the tower may *hold*,
rather than what a siege grants. So repairing the barrier is what buys enchanting
capacity.

```bash
# A worn tower holds half as much. The curve is unchanged; the question it
# answers is what moved.
ORBS_BOOT=0 ORBS_DUMP="attend sanctum; meditate 3600; attend bailey; defend; survey coffer" cargo run -p orbs
```

**In the calm it trickles; under siege only a resolved round pays.** Waiting
inside a turn earns nothing, so the one way to more quintessence is to advance
the fight and take what the enemy does — and that is §14 satisfied by
construction rather than by exception, because a per-round lump reads no clock
where the patient mode advances siege ticks on player input.

**`REGEN_PER_ROUND` is two, and it was four.** At four the reachability sweep
came back saying **`outnumbered` was no longer published on any seed** — a
garrison that can afford its dice every round is never overtaken, so a whole
reading and the solver rungs asking for it went quietly dead. Measured, not
argued.

**Enchanting during a siege is surcharged**, authored in `forge.toml` rather than
branched in code: the pool is shared, so a charm laid mid-fight is dice you
cannot pledge, and the multiplier makes it hurt twice.

### The enemy attacks the automation — §8.1's four surfaces, closed

**`tower::assault` runs on a resolved round and never on a tick**, which is what
keeps Phase A genuinely safe (pillar 4). §5.1: *"the enemy never touches scripts,
schedules, or logs in the calm layer."*

- **Script text** — one line rewritten, kept as a strict extension of the truth
  so it still *looks* like a line. `haul here there-` is the shape.
- **Trigger clocks** — a bound spell's step budget dragged. **The subtlest of the
  four**: the text is perfect, `peruse` shows what the player wrote, and only
  `verify` finds it. **Floored at one step a tick**, because §8's taxonomy is
  *"scripts always log and never halt"* — the enemy may slow the automation and
  may not stop it.

**Misdirection, never theft.** The true lines are kept, so notice → `verify` →
`purge` is a repair loop rather than a report — `substitute`'s rule one surface
over, and `purge` had to learn both new surfaces or it would clear the mark and
leave the spell corrupt.

```bash
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend bailey; defend; hold; hold; hold; \
  hold; verify; meditate 60" cargo run -p orbs
#   something got past the wall. a spell does not read as it was written
#   these are not what they say: coursing.spell, holding.spell
```

**The message never says *what* was touched**, and that is the puzzle: you are
told there is a lie and you spend a look finding it, which is what §8.1's
per-surface cooldown prices.

### The cadence is the load-bearing number, and only a sweep found it

**`defend` was free and unlimited, and `orbs-balance` measured 4.70 experience a
tick** — against clarity's 0.140. Thirty-three times the flagship, with every test
green and every See-it line correct.

**The escrow was not the thing to tune**, and that was the first diagnosis. A
siege paying 105 for thirteen rounds is right; fighting three hundred in two hours
is not. `siege::CADENCE` is 1200 ticks — §11.5's *"every 20–30 min"* — and stands
in until §5.3's trace provokes them.

**It is pinned at 0.128**, and it was nearly left unpinned for the wrong reason.
The first measurement spread 0.043–0.085 and read as dice variance over the ~6
sieges a run fits — `stacks`'s argument. It was not the dice: the driver keyed
*"have I already spent this round"* on the **byte length of a rendered prose
line**, so consecutive rounds collided and it stopped using its arsenal at
random. Keyed on `turns` the spread is **0.1225–0.1313 over five seeds**.

**A wide spread is a hypothesis, not a finding.** Reaching for "it is inherently
noisy" is how a driver bug gets written into the docs as a property of the game.

### Every shipped solver is run, and nothing ran them before

**`dev_spells.toml` ships a solver per domain and then some, and until
`tests/solvers.rs` nothing ran any of them.** Count them with
`grep -c 'name: "' crates/orbs-sim/tests/solvers.rs` — this sentence said
*thirteen* while the answer was sixteen, which is the failure the file warns
about three sections down and then committed twice in its own prose. They were reached only by See-it lines and by whichever
integration test happened to `invoke` one — so a spell could stop compiling, spin
for ever, or silently do nothing with the whole gate green. §19 records that
twice: `chanting` stopped compiling and a person noticed, and `besieging` shipped
with a `repeat until` guard that could never come true on a loss.

Six claims, per solver: it **compiles clean**, **does its work**, **latches no
fault**, **terminates**, **replays**, and **is in the table at all** — the last
being a lint, because a solver added to the file and not to `SOLVERS` is a worked
example nothing runs.

**The evidence is in the log, never the transcript.** §19: the pane draws *"what
the player did, not what their spells did"*, so a test looking for a solver's
output on screen finds nothing and looks broken.

**Two solvers are *meant* to fall short**, and the table says so rather than the
file assuming one budget: `chanting` collapses at one step a tick — that is the
menagerie's whole progression hook — and `ordering` is a producer that stops
after three queues.

```bash
cargo test -p orbs-sim --test solvers      # all thirteen, driven
```

**The bailey has the most solvers of any room, and the differences are the
point** — `grep -c 'domain = "bailey"' crates/orbs-sim/content/dev_spells.toml`,
because this said *four* while the answer was seven, three of which it goes on to
discuss further down.

`besieging` is the ladder; `answering` reads the telegraph and spends far less
for it — on seed 0 it keeps five wardings where `besieging` spends all six;
`sparing` hoards until the line is thin; `bulwark` composes with `part` and `for
each band`, which it can afford because **a siege does not advance until you
`hold`**; `steadfast` allocates by the telegraph, `warding_off` walks the areas
generically, and `sparingly` is the one that declines on purpose. Every other
room charges for looking.

```bash
cargo run -p orbs-balance -- run besieging --ticks 7200 --why
cargo test -p orbs-sim --test besieging      # the game: verbs, readings, escrow, replay
cargo test -p orbs-sim --test scripting_the_siege  # ...and whether a person can script it
cargo test -p orbs-sim --lib tower::siege    # the model: bands, intents, the arithmetic
cargo test -p orbs-sim --lib tower::dice     # the seven, and the odds against the dice
cargo test -p orbs-sim --lib tower::assault  # the calm layer, untouched
scripts/play.sh bailey::                     # four scenarios, through a real tmux game
```

**Read the `cost` column before the rate — it caught two policy defects here
before it caught the game's.** A driver with an empty arsenal asked for a troop it
did not have 6775 times in 7200 ticks; one that ignored the cadence asked for a
siege 7143 times. Both are *"the loop has fallen out of phase with the tower"*.

### The lens — Mastermind, and the orb deduces nothing

**Four sigils of six, repeats allowed, 1296 codes.** `probe` opens a reading if
none is open and presses the aperture; `dial <socket> <sigil>` turns one dial,
`dial <socket>` steps it round the six. **All of it is instant and none of it
costs the tower anything** — a press was twelve ticks of the one production slot,
which is ROADMAP's *"a read is not a brew"*, and that scarcity is withdrawn
(§19). So `meditate` after a probe is no longer needed anywhere, and a bound
solver runs beside a full brewing loop with no contention at all.

**It was 360 codes with no repeats, and everything wrong with the domain came out
of that** (§19, `0.3.22`). With no repeats a socket often cannot take the sigil
you want because another holds it, so a dial had to *exchange* the two — and a
dial that moves two sockets makes `aligned` rising unattributable, which is why
the ward needed a **ratchet**, a **settle-lock** and a per-socket **`untried`**
count to be solvable at all. All three answer *is this position correct?*, which
is the one question a codemaker never answers. They are gone.

**The orb keeps no candidate set and says nothing about any one socket.** That is
the rule the whole domain rests on: Mastermind's entire difficulty is the
bookkeeping, and a machine gets bookkeeping for free.

**Two channels onto one ward, and the spell's is strictly weaker.** A player
reads `aligned` and `astray` as **numbers** and deduces; a spell reads only which
way each moved — `closer`/`level`/`further` and `richer`/`unchanged`/`poorer`. The
deltas are derivable from the counts and not the reverse. **5.15 presses against
11.93.**

```bash
# A ward, pressed and dialled. `survey` both kinds of reading.
# **No `meditate`**: a press answers on the tick it is typed (§19).
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; \
  dial second borax; probe; survey prism; survey second" cargo run -p orbs
```
```text
prism    aligned = 0   astray = 1   level   spent = 2
         steady                     ← the second delta: astray did not move either
second   borax                      ← what is in it. that is the whole answer
```

**`survey second` says `borax` and nothing else, and that is the point.** It used
to add `loose`/`settled` and two tallies; a socket now reports what you put in it,
because whether it is *right* is your bookkeeping.

**A dial moves one socket and only one, and a sigil may sit in two.** This is
what repeats bought and it is the thing to check when touching `seat`:

```bash
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; dial first alum; \
  survey first; survey second" cargo run -p orbs   # both read `alum`
```

**A press stands.** `aligned` falling is now observable, and it is the single most
informative thing the lens says: only a socket that *was* right can make it drop
when it alone moved. That is what the restore rung in `breaking` reads.

**There is no `meditate` in a lens See-it line any more.** Every one of them used
to carry `meditate 13` — one tick past a twelve-tick press — and a press is instant
now. A dump that still waits is testing the wait.

**`scry` is not a verb and `seat` is not either.** `scr` reaches `scribe` and
`sea` reaches `sift`'s `search`, and `tests/naming.rs` allows no prefix
exemption. The domain is still scrying; the words are `probe` and `dial`.

**`tests/naming.rs` sweeps *readings* now, and it failed on its first run.** A
reading is a `NounKind::Sense` and `NounKind::Any` reaches one, so a reading sits
in the way of every room's vocabulary — §19 records `purge grind` fuzzy-matching
`gained`. Nothing swept them until `0.3.22`; five collisions turned up at once.
Two were the ward's new astray triple and are renamed (`fuller`→`richer`,
`steady`→`unchanged`). **The other three were live and are fixed in the resolver
rather than by renaming** — `purge walk` echoed `purge wall`, and `recall edit`
explained the maze's way out to somebody asking about the spell editor.

**Two widenings close the class**, both of rules that already existed:
`Scene::knowing` holds **every verb word**, so a word the game knows can never
fuzz into a noun; and every **one-word synonym** is a `NounKind::Command`, so
`recall walk` reaches `follow`'s page. Reading-vs-reading is still the worse
collision and is still only caught by looking — `thicker`/`thinner` scores 715.

**A known word may still *abbreviate*, and that is the trap.** Adding the verb
words to `knowing` made `invoke check` unreachable — `check` is one of `verify`'s
words and the spell is `check.spell` — and six spell tests went red at once.
Prefixing is not fuzzing, so `Scene::candidates` exempts a phrase that *starts*
the noun's name. Touch `knowing` and run `cargo test -p orbs-sim --lib
tower::spell` before believing it.

**The board draws whenever a reading is open**, not when a word is typed — the
map's rule, and what makes a bound solver watchable. Columns never rows, and it
splits *after* the instrument panel.

```text
┌ ward ─────────────────────────────────┐
│    first second  third fourth  answer │   ← the words `dial` takes
│ 1    ☼      ○      ♂      ♀   ○ ○     │   ← a press: the figure, then its pegs
│ 2    ☼      ♦      ♂      ♀   • ○ ○   │
│ 3    ♦      ♦      ♂      ♀   • ○ ○ ○ │   ← a sigil twice, and four pegs
│───────────────────────────────────────│
│ →    ♦      ♦      ♂      ♀           │   ← the aperture: what goes next
│                                       │
│ ☼ nitre   ○ alum    ♂ borax           │   ← without this, `♦` is unnameable
│ ♀ quartz  ♦ pewter  ♠ ochre           │
└───────────────────────────────────────┘
```

**The aperture row carried `■ · · ·` and does not.** Those were the sockets the
orb had proved right; the trailing blanks are the peg column, kept so every row
is the same shape and a reader can compare two presses down a column.

**39 cells, and it was 10** (§19). The compact version was correct and unreadable:
nothing said which column was which socket, so a player counted along the row
before typing `dial second borax`, and nothing anywhere said `♦` was `pewter`. Both
name tables come from the sim through `Ward::view` — they are content, and
`orbs-render` may not depend on `orbs-sim`.

**The gutter counts the real press, not the row.** The sheet caps at the last
twelve; numbering those `1..12` would say a long solve had just begun. It still
fits the 80×22 floor beside a transcript, and still refuses whole rather than
truncating. Twelve is chosen against hand play — the worst deducing solve over
all 1296 codes is nine presses, so a person's whole reading always fits.

**Six glyphs, not six colours** (§14): the tint is enrichment and the shape
carries the identity, so the sheet reads in a dump. `▪` is not in CP437 and was
the first choice; `■` is. It is spoken **once, as a summary** — a reader hearing
twenty cells read out would get box-drawing noise.

**A broken seal spills the far wizard's log**, quiet — twelve lines into
`lens.log`, one sentence on the transcript. Every line is generated from a real
`(instrument, verb, material)` triple, so it never names an operation that
cannot happen.

```bash
# `debug_ward` makes the answer whatever the aperture holds, so the next press
# breaks it — a See-it line for the *spill* should not open with forty dials.
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; \
  debug_ward; probe; peruse lens.log" cargo run -p orbs
```
```text
 5 mix phlegm            ← somebody else's laboratory
10 distil dregs          ← a recipe you do not have. that is the hint, not a leak
16 digest ground-mugwort
```

**Three recipes are `secret = true` and have to be found.** `debug_learn` takes
the next one in the file's order without waiting on the roll; `debug_learn
<name>` takes a named one and refuses anything that is not a secret.

```bash
ORBS_BOOT=0 ORBS_DUMP="recall mending; debug_learn; recall mending" cargo run -p orbs
```

**`debug_learn` skips the roll, so it is not a gate for the *discovery* — and the
whole loop the domain exists for needs two hours and a bound solver.** There is
deliberately no way to force a find (`debug_ward` takes no argument), so this is
the only line that shows the log spelling a recipe out, the orb writing it down,
and the rail raising its mark. It is slow and it is the real thing:

```bash
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; debug_spell breaking" \
  ORBS_THEN="invoke breaking; meditate 3600; meditate 3600; \
  sift mending lens.log; recall mending; attend laboratory" cargo run -p orbs
```
```text
probe 2. potash -> mending + phlegm  (alembic, 30t, needs heat) prism
2 probe mending prism and a way to make mending. the orb writes it down
mending: 2 steps, 36 ticks          ← `recall` answers now, and refused before
│lens         !│                    ← the rail's news mark, from another room
```

**The mark is the one part of this that no *short* dump can reach**, because the
chance starts at 4% and climbs — so a dump that solves one ward and looks for a
`!` finds nothing and is not broken. `Mark::News` is set only on a find, which is
the ask: a solve is ordinary, a discovery is news.

**Unmakeable, unnameable and unreadable, all flipping on the same tick** — and
the third is the one that hides. `Topics` is snapshotted once at construction, so
a secret's `recall_` page is in that snapshot from tick 0; the gate is a set
subtraction in `scene_at` and nowhere else. Gate it at snapshot time and `recall
<secret>` answers early with every test green.

**Only three questions consult `Learned`**: does a recipe fire, is it part-way to
firing, and is its product a word the player can say. **Leave everything else
unfiltered** — `execute::scroll`'s verdant unlock derives base reagents as *"in
the vocabulary and made by nothing"*, so a filtered `Recipes::outputs` would
offer an undiscovered potion as an inexhaustible herb.

**`debug_spell breaking` is the solver — one sweep, four socket phases.**

```
probe
if the prism is working              ← ...and the same for second, third, fourth
    dial first
    probe
    if the prism has further         ← it was already right
        dial first nitre             ← restore, and re-press to re-sync
        probe
    else
        repeat until not the prism has level
            dial first
            probe
        end
    end
end
```

**`dial <socket>` with no sigil is what makes that possible** (§19). A spell has
no variables, so it cannot name the sigil a socket has not tried — bare, the ward
steps it round. **It was 24 rungs, then 4, and it is 4 phases of 3.**

**Termination is a proof, not a budget.** One socket moves, so `aligned` can only
change because of it; stepping it cyclically reaches the code's sigil within five
and says so on arrival. Over all 1296 codes: **11.93 presses, worst 21**, against
a deducing player's 5.15 and blind guessing's 648.

**The restore rung is worth 1.9 presses** — a socket already right pays two turns
instead of six — and **the re-press after it is load-bearing**: the deltas are
measured against the *previous press*, so a restore with no press leaves the next
phase comparing against a figure that was never sent. 924 of 1296 without it.

Four traps, all worth knowing before writing another lens spell:

- **`is empty` does not mean *no ward*.** `spell::watch` answers `empty` by
  asking whether a node has *children*, and a prism's children are its readings —
  none until the first press lands. An open ward reports **`working`**.
- **`until not ... has level`, never `until ... has closer`.** A ward that gives
  mid-sweep publishes *nothing*, so `has closer` answers a flat no and the loop
  goes round for ever — probing a fresh ward open and dialling into it from the
  middle of a sweep. `level` is what the walk waits to stop seeing, and its
  **absence** stops the loop the same way its answer does. Found by running it.
- **Wrap each phase in `if the prism is working`** for the other half of that:
  the last socket to arrive can be the second, and every phase after it would be
  dialling at nothing. One solve in six.
- **`has closer`, not `is closer`.** `Condition::Is` takes a closed three-word
  vocabulary — idle/free/still, working/busy/running, empty/bare — so a *reading*
  is always asked for with `has`.

**`MAX_MEDITATE` is 3600, so a long run is several commands.** `meditate 9600`
silently becomes 3600, which is how a first pass at measuring this "found" a
plateau that was the cap. §19 records the retraction.

**`invoke` solves one ward; `bind` is the faucet.** A sweep breaks the ward in
front of it and stops. A binding re-casts a spell that has run off the end, which
is the shape §8 already describes: an invocation is an act, a binding is standing
automation.

```bash
# Two `debug_ward` solves buy the first slot, because a press is free now.
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="attend lens; probe; debug_ward; probe; \
  probe; debug_ward; probe; debug_spell breaking" \
  ORBS_THEN="bind breaking; meditate 3600; status" cargo run -p orbs

# ...and one ward, watched end to end — fifteen presses on this seed.
ORBS_SEED=3 ORBS_BOOT=0 ORBS_GRID=200x45 \
  ORBS_DUMP="attend lens; debug_spell breaking" \
  ORBS_THEN="invoke breaking; meditate 200; peruse lens.log" cargo run -p orbs
```
→ `experience 646`, and it is **linear** — a faucet that stops is this domain's
failure mode, not a crash.

**It runs beside a brew with no contention at all**, because a press takes no
slot. `orbs-balance`'s `scrying` policy reads **0.268/tick** against clarity's
0.140, and a real bound `breaking` ~0.18 — the gap is §8's interpreter spending a
tick on every `if`, `else` and `end`. **Additive rather than competing**, which is
what makes a rate above the flagship a decision rather than a defect (§19).

```bash
cargo test -p orbs-sim --test ward        # the sweep, solving through the real verbs
cargo test -p orbs-sim --test secrets     # the gate, and the verdant regression
cargo test -p orbs-sim --lib tower::ward  # the model: 1296 codes, the proofs
cargo run -p orbs-render --example screens   # the sheet, no sim and no GPU
cargo run -p orbs-balance -- run scrying --ticks 7200 --why
scripts/play.sh lens::                    # ten scenarios, through a real tmux game
```

**The lens's board is titled `seal` now, not `ward`** (§19). §10 reserves "ward"
for defense and the lens had borrowed it; the prose keys keep their spelling
because `tower::Ward` is not player-facing.

### The sanctum — Hanoi, and there is no clock in it

**Three stations, three to seven wards, and a greater one never rests upon a
lesser.** Raw arcane energy wells up in the `wellspring`; `muster` draws a course
out of it, and it belongs assembled at the `barrier` with the `conduit` between.
`haul <from> <to>` carries the topmost ward. Both verbs are **instant and take no
production slot**, so a bound solver runs beside a full brewing loop — the lens's
decision, not the archive's.

**The room was `battlements/` and the names were masonry** — a `rampart`, a
`barbican`, a `bastion`, a `redoubt` (§19, `0.4.1`). *"Haul a ward from the
barbican to the redoubt"* is not a thing a wizard does, and §7's tree and §10's
table are both superseded by the rename. The mechanics did not move.

**This is §10's reflex-avoidance mechanism and it dissolves the problem rather
than answering it.** ROADMAP would not start the phase without one: *"command
pressure at 1 Hz is a reflex mechanic unless something makes it a decision."*
Hanoi has no clock at all — a course waits for ever, every ward is on screen, and
the only thing that can go wrong is picking the wrong pair of stations.

**Between any two stations exactly one haul is legal**, unless both are empty.
That one fact is what the whole domain is built on: a player choosing a *pair*
has already chosen a move, and a spell needs only to work out which way round it
runs. `a_haul_between_two_stations_is_unique` is the proof.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend sanctum; muster; survey pylon; survey wellspring" cargo run -p orbs
```
```text
integrity = 100     ← on the pylon, and published from tick 0
potency = 1         ← on the wellspring: the ward on top. the least
```

**A station publishes `potency` only while it holds a ward, and that is
load-bearing.** `spell::watch` answers `is empty` by asking whether a node has
children, so a station that always carried a `potency` could never be empty and
the first two rungs of every solver would be dead. It is the maze's *"a walled
way publishes no `marks`"* arrived at backwards — and it is why the emptiness
rungs must come **before** the comparison: an absent reading counts as
**nought**, which makes an empty station the least thing on the board.

**Integrity is the first drain in the game.** Everything else the tower has only
rises. A point every 30 ticks, whether or not anybody is playing; a finished
course puts back 8 a ward. **A worn tower musters a taller course**, which is the
whole of what erosion does today — the siege coupling is Phase 8's (§19).

```bash
# An hour unattended, and the consequence. `MAX_MEDITATE` is 3600, so one command.
ORBS_BOOT=0 ORBS_DUMP="attend sanctum; survey pylon; meditate 3600; \
  survey pylon; muster; survey pylon" cargo run -p orbs
```
```text
integrity = 100  →  integrity = 0
7 wards well up in the wellspring   ← a kept tower gets three
integrity = 0  odd                  ← and `odd` is why that matters
```

**`odd` exists because the cycle direction depends on the parity.** An even
course sends the least ward wellspring → conduit → barrier and an odd one the
other way; get it backwards and the course finishes **in the conduit** with the
barrier no better than it was. The height is jittered precisely so a player
cannot learn their tower's number and stop reading.

**The one refusal that is the puzzle rather than a dead end:**

```bash
ORBS_BOOT=0 ORBS_DUMP="attend sanctum; muster; haul wellspring barrier; \
  haul wellspring barrier; haul wellspring wellspring; haul conduit barrier" cargo run -p orbs
```
```text
the greater ward will not rest upon the lesser   ← the rule, and it costs nothing
the wellspring is where it already is
nothing is resting at the conduit
```

**`debug_course` leaves the drawn course one haul from done** — the completion
is the interesting half and 127 hauls is not a See-it line. It is `debug_ward`
one room over, and it republishes where `debug_ward` does not: a course's
readings are rewritten by the *next haul*, so a stale board would be dialled at.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend sanctum; muster; debug_course; \
  haul conduit barrier; survey pylon; status" cargo run -p orbs
```

**`holding` is the solver, and it is the first spell that needs `part` and `let`
together.** One part, three `let` pairs, and a parity read off the world — the
cyclic rotation in twelve lines. It solves in exactly `2^n − 1` hauls, which is
optimal, and that number is the test: a wrong cycle still *finishes*, just at the
wrong station.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend sanctum; invoke holding; meditate 400" \
  ORBS_THEN="peruse sanctum.log" cargo run -p orbs   # 15 hauls for four wards
```

**`muster` is the spell's first line and it has to be.** Without it the pylon
is idle, `repeat until` is satisfied before the first pass, the loop runs zero
times and the spell ends having done nothing — and bound, it does that for ever.
With it, `bind::stand` musters afresh every lap, which is the faucet.

**The board draws whenever a course is drawn**, not when a word is typed — the
map's rule and the sheet's. Columns never rows, and it splits *after* the
instrument panel. **It is the one picture that still fits the 80×22 floor**,
where the maze pans and the sheet yields: 35 by 12 with its border.

```text
┌ pylon ──────────────────────────┐
│wellspring   conduit    barrier  │   ← the words `haul` takes
│                                 │
│    ███         █                │   ← a ward `n` wide is ward `n`
│   ████        ██                │
│────────── ────────── ────────── │   ← the floor. the stack is a staircase
│        4 wards, 3 hauled        │
└─────────────────────────────────┘
```

**Magnitude is width, never colour** (§14). The puzzle's one rule is about
*relative magnitude*, so a hue would make it invisible in greyscale and invisible
in a dump; the tint is one violet for every ward and is pure enrichment.

**The rail carries `py 100%`, and it is the only meter in the tower that counts
up.** Every other rail detail is work *remaining*, so `detail_of` prints the
remainder and falls silent at nought — which would have taken the sanctum's only
glance away exactly when the barrier was whole. `Unit::Standing` is answered
ahead of that gate, and the `%` is the `t` suffix's job: without it a standing
`py 62` and a remainder `py 4` are the same shape.

**It says the barrier and never the course**, drawn course or no — and it said
the course first, which was §19's rail defect for the third time. A meter of
wards-still-to-haul counts *down* as a solver wins, so the sanctum read `py 4`
and, after `py 100`, that reads as a barrier about to fail. Worse than the
archive's `st 350t` and the lens's `pr 4t`, because the number **vanished** into
course progress exactly while a bound solver was working — the one time you are
in another room and glancing. The board two columns away says where the wards
are; the rail says whether the tower is safe.

**Two ordering defects, both found by looking rather than by testing** (§19), and
both are the same shape — a number written to two surfaces in the wrong order:

- `erode` had to become an **exclusive** system. It wore the resource down and
  left the *reading* alone, so `survey pylon` and every `if the pylon has
  fewer than n integrity` reported a whole barrier while the rail counted down.
- `finish` **mends before it publishes**. The other way round, a finished course
  said `integrity = 0` on the transcript and `ra 56` on the rail, on one tick.

**And one worse than either: the reading did not exist until tick 30.** It was
published only when the number moved, and `watch::many_at` answers an absent
child with **nought** — so `if the pylon has fewer than 60 integrity` was true of
a barrier in perfect repair for the first half-minute of every session.
`Sim::bare` publishes once after the tower is raised. **A guard that fires hardest when
nothing is wrong is the worst shape a guard can have**, and no test that mustered
first could have seen it.

```bash
cargo test -p orbs-sim --test warding      # the game: verbs, readings, the solver
cargo test -p orbs-sim --lib tower::pylon  # the model, and the optimality proof
cargo test -p orbs-sim --lib tower::erosion  # the curve and its two clamps
cargo run -p orbs-render --example screens   # the board, no sim and no GPU
cargo run -p orbs-balance -- run warding --ticks 7200 --why
scripts/play.sh sanctum::              # seven scenarios, through a real tmux game
```

**`warding` reads 0.1249 on every seed** — the flattest column in the table, and
arithmetic rather than luck: a course of `n` costs `2^n` ticks and pays `n − 2`,
so three and four both come out at an eighth. It is **under** clarity's 0.140
where scrying's 0.268 is over, because a finished course also puts the barrier
back and a domain paying twice should not also pay the best rate in the tower.

### The menagerie — a figure sung against the tick, and the one clock in the tower

**Twelve syllables, one landing every four ticks, four lanes.** `summon` draws a
figure; `sing <syllable>` answers the one at the rule; `chorus` hands the arrows
over. A chant costs **nothing** to attempt — what it risks is the barrier, which
loses 5 when a figure collapses, and that risk *is* the price.

**§10.1's *"timing means windows at 1 Hz — never a reflex"* is struck for this
domain and this domain only** (§19). Five rooms are solved by choosing and this
one by doing; the accommodation is `F9`, not an easier chant.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon" cargo run -p orbs
```
```text
┌ figure ────────────────────────────────┐
│ leftward  skyward  earthward rightward │   ← the words `sing` takes
│────────────────────────────────────────│   ← the rule. syllables land here
│                        ▼               │   ← next, and it is directly under
│                                  ►     │
│    ◄                                   │   ← ...and these are further off
│           12 to come, 0 missed         │
└────────────────────────────────────────┘
```

**They rise, and that is the one picture in the game that moves.** A rule at the
top with notes climbing to it is what every rhythm game does, and the reason is
that the line stays put while the eye tracks approach. It was drawn downward
first and looked wrong immediately. **It fits the 80×22 floor**, which the maze
and the ward sheet do not — and it has to, because the domain is unplayable by
hand without it: the aperture moves every four ticks and `survey` costs one.

**Singing early is not a strike**, and that is the whole mechanic. A syllable is
struck only inside a two-tick window at the rule; the right word too soon costs
it exactly as a wrong word does. Without that a solver would answer the moment it
identified the lane and `bide` would have nothing to count.

**`PACE` was one and the domain was unsolvable by a spell.** A question costs a
tick and the figure advanced whenever nothing was sung, so read-then-sing missed
by exactly one, for ever — twelve figures collapsed with no strike at all. It is
four now, which is *shorter* than a four-lane ladder, and that is deliberate:

```bash
# The shipped solver. **It is meant to fail at the shipped budget.**
ORBS_BOOT=0 ORBS_DUMP="attend menagerie; peruse chanting.spell" cargo run -p orbs
```
→ **12 of 12 struck at two instructions a tick, collapses at one.** The
menagerie is the domain that rewards concentration: unautomatable until the
weave grants a second step, solved outright once it does.

**`bide until` is gone, and the `until` *reading* went with it** (§19). The
delay used to be read off the circle, so the solver computed nothing — which is
the blocking-wait shape the domain was designed to refuse, rebuilt under another
name. Removing the word alone would not have fixed it: with the reading still
answerable, `repeat until the circle has 1 until` / `end` is the same cheat
spelled as a one-tick spin. `Chant::until` still answers for the board and for
`orbs-balance`; the *language* cannot ask.

**So the spell has to keep its own time, and the shape is the puzzle.** Three
things, all measured rather than reasoned about:

- **`for each syllable`, never an `else if` ladder.** A ladder short-circuits, so
  the lane found on the first rung is reached three ticks before the one found on
  the fourth, and a `sing` arriving at a different offset each lap cannot sit in
  a two-tick window. At two steps a tick the ladder strikes **nought of twelve**.
- **`let`, and the `sing` *outside* the loop.** Binding the answer and singing
  after the loop closes is what puts the strike at a fixed offset. Singing where
  the lane is found is variable again — 6 of 12.
- **No `bide` at all.** The pass comes out level with `PACE` on its own; `bide 2`
  breaks it. The arithmetic is *"what does my loop already cost"*.

**Nothing tested `bide` before `0.5.8` — not the word, not the count, not the
reading form the whole domain rested on.** That is why the shipped `.spell` could
stop compiling with the gate green, and why the test is the *pair*: collapse at
one step, close at two. Either half alone passes against a spell that never works.

```bash
cargo test -p orbs-sim --test chanting the_shipped_solver   # the hook, asserted
cargo test -p orbs-sim --lib tower::spell::program::tests::bide_takes_a_count
```

**`repeat until the circle is empty`, never `is idle`.** A circle is idle whether
or not a chant runs, so `is idle` is satisfied before the first pass, the loop
runs zero times and the spell does nothing — silently. That cost four
experiments. A running chant publishes readings, so `is empty` is the question
that separates them.

**Two diagnostics that cost more than the bugs**, both worth knowing:

- **`survey` emits `TableRow`s, not `Message`s**, so `peruse <log>` cannot see
  its answer. Three experiments concluded `for each` was broken while it worked
  perfectly. Prove a loop with a verb that *speaks*.
- **`invoke d6` is ambiguous** against the shelved dev spells, so a spell that
  never ran looks exactly like one that ran and did nothing. Name a scratch spell
  distinctly.

```bash
# The arrows. `chorus` takes only the keys — the board already draws beside the
# transcript, so unlike `wander` this surface does not take the pane.
ORBS_BOOT=0 ORBS_DUMP="attend menagerie; summon; chorus" \
  ORBS_CHANT="<up>\n<left>" cargo run -p orbs        # ...both read `too soon`

# §14's accommodation, and the only way to see it as text. A dump has no clock,
# so without this every press lands on one tick and reads `too soon` — which is
# the played mode working and the patient one being invisible.
ORBS_BOOT=0 ORBS_PATIENT=1 ORBS_DUMP="attend menagerie; summon; chorus" \
  ORBS_CHANT="<up>\n<left>" cargo run -p orbs        # ...they land
```

**`F9` is the key and it reaches the same ceiling.** A patient chant and a played
one yield exactly what was sung correctly, so the setting removes the dimension
reflex cannot serve and nothing else — never a difficulty. Bound in **both**
frontends, and not inert in the terminal: it changes what a strike is worth,
which is world state.

**`chorus`, because `per` reaches `peruse`.** `perform` scores nothing against
anything and an abbreviation collision is invisible to a score; its synonym
`conduct` fell to `conjure` the same way. **`left` and `right` are not syllables**
either — 750 against the spell language's `let` and 800 against `light` — which
is why the four are `-ward`. Three of the words this domain wanted were taken by
something a similarity sweep could not see. **Sweep similarity *and* prefixes.**

```bash
cargo test -p orbs-sim --test chanting     # the game: verbs, readings, parity
cargo test -p orbs-sim --lib tower::chant  # the model: the pace and window arithmetic
cargo test -p orbs-sim --test progression  # `steps_1`, its refusals, and the replay
```

### The tower rail — every domain at a glance, and no telemetry pane

**There is one main pane now.** The rail takes 16 columns down the right and
holds one box per domain; the telemetry pane it replaced is gone, and five of its
nine readings live at the rail's foot (`tick`, `held`, `scale`, `grid`, `focus`).
The other four were already in `status`, which is the full answer where the rail
is the glance.

**Seven boxes, always, and four are dark.** A room the tower has not built draws
as a dim dotted row with no name — the slots keep fixed positions so a box never
appears and pushes the others down at the one moment it has news.

**Each box is ruled off from the next**, and the rule belongs to the box *above*
the boundary. The seventh draws none, because the foot opens with a rule of its
own and two in adjacent rows is a mistake nobody would author on purpose.

**So the boxes are all the same height and the spare row goes to the foot** —
which is deliberately *not* `tiling::deep`'s leftovers-to-the-earliest rule that
`lay_rail` used to share. A pane's exact height is invisible; a *ruled* box's is
not, and the first box being one row taller put one separator a row below the
other five. `MIN_RAIL_BOX` went 3 → 4 with it: the rule costs a row, and at three
a squeezed box silently dropped the `►spell` line, which is the one row telling a
player that room is automated.

**`►`, not `▸`.** U+25B8 is not in CP437 and drew as `?`, so the row saying a room
is automated read as the orb not knowing what was there. `is_renderable` never saw
it — that lint is for authored prose, and a painter's glyphs are Rust literals — so
`every_glyph_the_rail_draws_is_in_the_code_page` is what holds it now. The board's
`▪` was the same defect and §19 records it; this is the second, found the same way.

**The spell row says `►tending`, not `►tending.spe`.** A spell node is named for
its file, and six columns of `.spell` in a fourteen-column rail left the rail
cutting the extension in half. Stripped in `brief.rs` with the
`content::without_extension` that already existed, not in the painter: rule 2
means `orbs-tui` must not have to know spells live in files.

```bash
# The rail beside a working laboratory. `mp 8t` is the two-letter form the
# instrument panel uses, because `mortar_and_pestle 8t` truncates to a name with
# no number left on it.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; kindle charcoal; grind sage" cargo run -p orbs

# **A fault, latched.** Stand in the archive and the rail says the laboratory is
# broken — `laboratory ‼`. A refused command is *not* a fault; the four that are
# come from `say_failure` at `Role::Danger` — gave up, missing name, forbidden
# verb, unreadable line.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; scribe broken" \
  ORBS_EDIT="edit\nrepeat 5\nwield zzz\nend\n<esc>\nquit" \
  ORBS_THEN="invoke broken; meditate 20; attend archive" cargo run -p orbs

# ...and cleared by going to look, which is the only thing that clears it.
# Append `; attend laboratory` to the line above and the `‼` is gone.

# The rail **yields whole** below the grid that can host it — no narrow
# fallback. At the floor there is no rail and the readings fall back into the
# session's border title.
ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_DUMP="attend laboratory; grind sage" cargo run -p orbs
```

**`F4` is visibly inert, and that is recorded rather than fixed.** With one pane
both tilings are identical, so the key changes nothing until multiplexing returns
the second pane in Phase 12a. §9 fixes the focus mode on `F4` and §19 fixes the
switch there, so reassigning it to toggle the rail would re-litigate both.

### `orbs-balance` — the economy, looked at rather than argued

**Every duration in the game is a placeholder and this is what sweeps them.** It
drives a real `Sim` through `submit` with six synthetic players and prints
experience per tick against a pinned reference. It found four disagreements with
DESIGN.md on its first clean run (§19), so treat a number in the design as an
*idealisation* until this has been run against it.

**And then it caught a real regression one item later** — the ambient reagent swap,
flagged on all four pinned policies at once while the full suite stayed green and
every See-it line still read correctly. **Run a sweep after anything that touches
the world, not only after anything that touches a duration.**

```bash
cargo run -p orbs-balance -- list
cargo run -p orbs-balance -- sweep --ticks 7200 --why
cargo run -p orbs-balance -- run clarity --ticks 1200 --why
cargo run -p orbs-balance -- sweep --hours 4 --csv /tmp/curves.csv
cargo test -p orbs-balance          # the pins, as a test rather than a column
```

**Two hours is the shortest honest sweep, and an hour was the default.** A policy
amortises its first lap's setup over the run, and the ambient reagent swap
(`tower::sabotage`) costs a few minutes an hour — so at 3600 ticks one badly-timed
swap moves a rate by more than its tolerance band and the table flags the seed
rather than the game.

**A `<-- drifted` marker is only a pin while somebody reads the column, so
`tests/agrees.rs` reads it.** It runs the five pinned policies through the real
`drive::run` and fails when a measurement leaves its band. It exists because the
first version of that file — named for this crate's See-it claim — drove `Sim` by
hand in both its tests and would have passed with the whole harness deleted. The
crate has a `lib.rs` for that reason: a binary-only crate cannot be reached from an
integration test at all, which is *why* it had been written the other way.

**Read the `cost` column before the rate.** Every entry in it should be a scour
the policy asked for; anything else means the loop has fallen out of phase with
the tower, and the rate beside it is measuring a game nobody plays. `--why`
prints the sentences behind it, which is where the diagnosis is — **two sweeps
were read as balance findings before that flag existed, and both were the
policy's fault.**

**A rate here is a *loop* rate and DESIGN.md's is a *recipe-tick* rate.** Clarity
is 0.170 idealised, 0.130 hand-played (`experience 16` at tick 123), 0.140
looped. The gap is a tick of queue latency per command plus a `PURGE_TICKS` scour
per fouled instrument, and it is not a defect in either number.

**...except for `bound`, where a cost is usually the loop working.** A spell that
**waits** stamps `Role::Cost` too — `say_blocked`, once per block — which is the
runner doing exactly what §8 asks of it, so `bound` carries ~640 healthy costs in
a two-hour sweep. Only the sentence tells a wait from a refusal, which is why
`--why` matters more here than anywhere:

```text
640  tending.spell waits: the mortar_and_pestle is working   ← the loop working
  1  the orb is already holding tending.spell                ← the loop broken
```

**A maze is one sample per seed** — `stacks` reads 0.0067 at seeds 0 and 11 and
0.0144 at seed 3, so it is deliberately absent from the pinned table. Sweep it
across several `--seed`s or conclude nothing.

**A ward is not**, and the contrast is worth knowing: `scrying` reads 0.2656 to
0.2703 across the same four seeds, because a two-hour run breaks hundreds of
wards and the draw averages out. It is pinned at **0.268**.

**A siege is in between, and `besieging` is the one row whose `<-- drifted` you
should not chase.** It reads 0.0992, 0.1254, 0.1138, 0.1167 across those seeds —
a **mean of 0.1138** against its pinned 0.114, over a spread that is 23% of the
mean. `--ticks 7200` fits about six sieges, so one badly-timed sabotage moves a
whole siege and the seed shows through. It is pinnable where `stacks` is not
*because* four worlds average, which is exactly what `tests/agrees.rs` does — so
the sweep's single-seed column flags it about half the time and the test stays
green, and **that is the instrument working rather than disagreeing with itself.**
Read `--why` before believing the marker: cadence waits and sabotage notices are
the healthy costs, and anything else means the driver has fallen out of phase.

**Two policies model a *spell* by typing its commands, and neither pays the
interpreter.** `stacks` and `scrying` both issue one command per tick, where a
real bound spell also spends a tick on every `if`, `else` and `end` — `breaking`
measures ~0.18 against the policy's 0.268. That is §19's *"the harness has no
player"* applied to execution: a policy is a ceiling, and the column says what
the loop is worth rather than what the language costs. **`bound` is the one that
pays it**, which is the next paragraph.

**`bound` is what measures the script engine, and nothing did before it.** It is
`grind`'s loop again — the same two commands for ever — run by a bound spell
instead of by hand, so everything about the two is equal except who is typing and
the gap is purely what automation costs. **It keeps 0.910 of the hand-played
rate, on every seed**, because that quotient is a property of the runner rather
than of the world: the absolute rates both move with a world's luck at sabotage
and the ratio does not move at all. So `tests/agrees.rs` pins the **quotient**
and the table reports the rate.

```bash
cargo run -p orbs-balance -- run bound --ticks 7200 --why   # 0.0910
cargo run -p orbs-balance -- run grind --ticks 7200         # 0.1000, by hand
```

**Run it after anything that touches the spell runner** — `SCRIPT_BUDGET`,
`PATIENCE`, `would_block`, the compiler or the language. Those were unmeasured
for four phases and §19 records what that cost.

**Adding a policy: scour *before* use, never after fouling.** The byproduct a
stage leaves blocks the *next* lap's load, so a purge placed after the stage that
dirtied it cleans an instrument nothing is waiting on and still leaves lap two
refused. And a driver must wait on the **triage** slot as well as the production
one — `Sim::working()` reports only the latter, and a policy that runs over its
own scour reports 0.072 for a loop worth 0.140.

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

**The fetch reaches into other instruments, not just the shelf**, and that is what
makes the loop `move`-free rather than merely shorter. `pipeline::reachable` walks
every unbusy instrument in raise order and *then* the store, so `digest ground-sage`
takes it straight out of the mortar and leaves the husks. A whole clarity brews to
`experience 16` without the word.

**`move` stays in the laboratory anyway**, and §19 says why: it is the only way
finished work leaves the room (`move clarity to arsenal`) and the only way to reach
an instrument's `charged` state, which the three animation See-it lines below need
because `grind` never rests there. What changed is the *manual* — `recall move` led
with `move sage to mortar_and_pestle`, which is a loop nobody writes.

```bash
cargo test -p orbs-sim --test fetching   # the tool-to-tool reach, and the brew
```

**`empty` is a different question and is still required.** It clears the byproduct
so the mortar can take a second load; without it a brew stalls at `grind rock-salt`
with *"the mortar_and_pestle can do nothing with husks, rock-salt"*.

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

**A crossing is a fourth edge, and the hardest of them for a dump**, because it
is an edge between *two screens* rather than inside one. A dump paints one frame,
so on its own there is nothing to cross from. `ORBS_PASSAGE_AT` therefore does
something no other switch does: it **holds the last `;`-separated command back**,
paints the screen that command was about to replace, keeps it, runs the command,
and paints again with the crossing posed over the result. What prints is a
crossing between two screens the game can actually reach.

```bash
# The laboratory leaving and the forge arriving. Step the fraction: 0.15 is the
# wake mid-strip, 0.50 the empty beat between the screens, 0.85 the arrival.
ORBS_BOOT=0 ORBS_PASSAGE_AT=0.15 ORBS_DUMP="attend laboratory; attend forge" cargo run -p orbs

# **The transcript is spared, and that is the whole design.** History did not
# change when you walked to the archive, so a crossing that wiped it would say
# the session went away. This is the capture that would catch it.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_PASSAGE_AT=0.30 \
  ORBS_DUMP="attend laboratory; grind sage; attend archive" cargo run -p orbs

# ...and a surface that genuinely replaces the pane, where nothing is spared.
ORBS_BOOT=0 ORBS_PASSAGE_AT=0.30 ORBS_DUMP="attend archive; research; wander" cargo run -p orbs

# The off switch. Byte-identical to the same line with no passage variables.
ORBS_BOOT=0 ORBS_PASSAGE=0 ORBS_PASSAGE_AT=0.30 \
  ORBS_DUMP="attend laboratory; attend forge" cargo run -p orbs
```

**`ORBS_PASSAGE` and `ORBS_PASSAGE_AT` are two switches on purpose**, and it is
`ORBS_FIRE`'s rule restated: *a crossing at zero is a perfectly ordinary
crossing* — it is one of the two endpoints, and it draws the departing screen
exactly. So the fraction cannot double as an off switch, and the off switch
outranks the fraction. `scripts/tui.sh` and the play suite both set
`ORBS_PASSAGE=0`, for the same reason `ORBS_FIRE=0` exists: a harness that types
a command and reads the screen straight back must not be able to catch one
part-way through leaving.

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
first segment is a word (`edit`, `guide`, `interpret` or `quit` — that is the
whole vocabulary), `edit` drops into the buffer, and the token `<esc>` comes back
out.

**The guide is the fourth word and the pane down the right.** By default it lists
the nine control words and then the verbs *this spell's domain* offers — the
**spell's** domain, not the player's, so the same editor in the archive and in
the laboratory shows different verbs. Put the caret **at the end of a word** and
it becomes that word's page instead. `guide` toggles it, session-scoped; nothing
persists it.

**It needs 97 columns and yields whole below them** — 60 for the buffer, the
gutter, and 30 for the guide. At the 80×22 floor it is simply not there, which is
correct: `threading` has 62-character lines and a 43-column buffer is worse than
no guide.

```bash
# The listing, then the page. Only a running editor has a caret to move, so
# this is a `tui.sh` See-it line and `ORBS_DUMP` cannot reach the second half.
scripts/tui.sh start
scripts/tui.sh type 'attend archive' 'scribe threading' 'edit'
scripts/tui.sh see                       # nine control words, then `here you can`
scripts/tui.sh key Right Right Right Right Right Right
scripts/tui.sh see                       # ...`repeat`, its page, wrapped
scripts/tui.sh key Down Down
scripts/tui.sh see                       # ...`follow` — a verb, the other key family
scripts/tui.sh stop
```

**Two key families, and getting the spelling wrong registers a subject nobody
authored.** A control word reads `recall_<w>` / `using_<w>`; a verb reads
`man_<w>_gloss` / `man_<w>_use`. `Prose::topics` strips `recall_` to decide what
is nameable, which is the same `clarity_use` trap the manual already carries.

**The caret must be *past* the word's first glyph**, not at its start: `guide`
matches `run.start < at && at <= run.end`. So entering edit mode — caret at 0,0 —
shows the listing rather than the page for the first word, deliberately.

**Touching a word is a page; past it is what may come next.** Type a space and
the guide stops explaining the word and starts answering `if ` with the places,
`is ` with the eight state spellings, `repeat ` with `until`. `parser::expect` is
what it asks, so **the guide, both Tab completions and the prompt's ghost all
give the same answer**.

```bash
scripts/tui.sh type 'if '                       # the places
scripts/tui.sh type 'the mortar_and_pestle '    # `is`, `has`
scripts/tui.sh type 'is '                       # idle free still working ...
```

**The states belong to `is` and *not* to `wait`.** `wait` stores a **thing** and
resolves it by scanning the record stream, so `wait for the mortar_and_pestle to
be idle` waits on a thing called `mortar_and_pestle be idle`, matches nothing,
burns `PATIENCE` and latches a fault on the rail. `be` is not filler. This is the
trap the guide exists to stop and it caught its own author writing the spec.

**Three of the nine words cannot open a line**, so the listing shows six at the
top of an empty spell: `until` is `repeat`'s guard, `end` needs something open,
`else` needs an `if` **directly** above it. Open an `if` and the other two
appear. `open_blocks` keeps a **stack**, not a depth — a count cannot tell an
`if` inside a `repeat` from a `repeat` inside an `if`.

**Tab completes in the editor, and the guide is its listing.** readline's rules
live once, in `orbs_shell::tabbing`: extend to the longest common prefix, list
when that adds nothing, then cycle. The prompt needs `Offered` and a reserved
row to show candidates; the editor does not, because the pane beside it is
already showing them.

```bash
scripts/tui.sh type 'gri'; scripts/tui.sh key Tab   # -> `grind `
scripts/tui.sh key Tab Tab Tab                      # lists, then cycles
```

**The prompt is highlighted too**, on the same three weights — but `repeat` and
`gathering()` are **suppressed** there, because the prompt cannot run either and
drawing them bright would say the orb knew a word it is about to refuse. Weight
is invisible to `ORBS_DUMP`, so `ink` is the only instrument that can see any of
this.

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

# `has` takes a count, and `interpret` is where you check it survived. This
# **used to print `if cabinet has fragment`** — the number silently swallowed,
# no fault raised, which is §19's "the orb writes down a shorter command than it
# heard" arriving through the one surface built to catch it.
ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend archive; scribe check" \
ORBS_EDIT="edit\nif the cabinet has 4 fragment\nwield lectern\nend\n<esc>\ninterpret" \
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

### `recall apprentice` — the lesson, where `recall scripting` is the reference

**Nothing taught a player how to make a spell.** `recall scripting` lists the
words, the question shapes and what the room can name — the right page to have
open *while* writing, and it teaches nobody how to start, because **no listing of
words teaches an order**. `scribe`, `edit`, the lines, `<escape>`, `quit`,
`invoke`, then the log rather than the pane: seven steps, one of which — *quit is
the save* — a player otherwise learns by losing work.

§12 puts the in-world grimoire in the *"always"* column; Phase 13 owns the
interactive apprenticeship. This is the reference half, which is why it is a
`recall` page and not a scripted sequence.

**The worked example comes from the room.** `recall scripting`'s third section
arriving at the same conclusion: the shape of a spell is the same everywhere and
the lines in one are not, and showing `grind sage` to somebody standing in the
lens teaches them a room they are not in. A room with no work to script borrows
the laboratory's and **says whose they are**.

```bash
for room in laboratory archive lens sanctum menagerie grimoire; do
  ORBS_BOOT=0 ORBS_DUMP="attend $room; recall apprentice" cargo run -q -p orbs; done
```
```text
  grind sage               the first line - exactly what you would have typed
  probe                    ...in the lens, and `research` in the archive
  no work happens here, so these lines are the laboratory's   ← the grimoire
```

**`apprentice`, not `primer`** — `primer` is already this module's word for the
room's three-line intro, and one word for two pages in one file is how the next
reader merges them. It is §12's own term (*"diegetic apprenticeship"*) and reads
as the request a player is making. `spellcraft` scores 925 against `spell` and
shares its prefix; `crafting` and `writing` both score 667 against `scripting`,
which is the one page it must not be confused with.

**The way in is `help`.** Everything on that page is a word to type *now*, and
nothing on it said the orb could be taught to type them for you — so a player
could read `help` in every room and never learn the game has spells in it. A
reference nobody can find their way into is not one.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; help" cargo run -p orbs | grep taught
#   the orb can be taught to do all of it. recall apprentice
```

**Two gates, and the second is the one that matters.**
`the_apprentice_only_shows_lines_the_room_can_run` asks two questions of every
example — does it *resolve* in that room, and is its verb one that room *offers*
— because `grind sage` typed in the lens resolves and is then refused, so a lint
that only parsed would pass the laboratory's whole example printed anywhere.
And `scripts/play.sh the_apprentice` **follows the page**: finds it from `help`,
reads it, and types what it shows, at a real keyboard, ending in a spell that
earns. A tutorial is the one page whose lines a player will type rather than read
past, so the failure mode is a dead end reached by doing exactly the right thing.

**`recall` bare is the manual's overview, and `help`/`man`/`?` all reach it.**
**It opens with a primer for the room and lists the vocabulary second** — a player
who types `help` in the lens is asking what a ward is, not for twenty-five verbs.
The listing is still `execute::offered`, the same filter the boot report uses,
grouped by `Verb::group()`.

**Three prose keys per room**: `man_here_<room>` is what the place is,
`man_start_<room>` how to begin its puzzle, `man_solve_<room>` how to finish one.
`man_`, never `recall_` — `Prose::topics` strips the latter to decide what is
nameable, so `recall_here_lens` would register `here_lens` as a subject nobody
authored (§19; the `clarity_use` trap again).

**A room lists only its own operations**, and it used to list every other room's.
`Scene::offers` asks `Verb::anchor` — *which fixture must stand here* — where it
asked `is_operation`, the **production slot** question, so anything that took no
slot went everywhere: `help` in the laboratory offered `research`, `follow` and
`wander`. §19 records this as the debt those two words were waiting on.

The two questions are separate now, and a verb can be either, both or neither.
Self-anchored verbs are **derived from the `Branch` that declares them**, so a new
domain's verbs scope themselves; `every_self_anchored_verb_is_declared_by_a_fixture`
fails the build both ways, because a verb claiming an anchor nothing declares
resolves in *no* room. And the refusal names the fixture now —
`there is no stacks here to wander with`, from `tower::fixture_of`.

**Check it in every room, and at the floor.** The 70-cell width lint keeps a
single line from wrapping there.

**Four rooms fit 80×22 whole; the laboratory and the archive do not, and that is
not a defect.** Measured: the laboratory's page wants **26 rows** — it is the one
room whose transcript is narrowed by the instrument panel, *and* the one with
five extra verbs — and the archive is one row over. The primer lines are already
inside the width lint, so the only way to make them fit would be to stop offering
a verb, which is worse than scrolling. `help` is a record like any other and
`PgUp` reaches it.

This block used to claim the page "fills the 80×22 transcript exactly". It never
did for the laboratory, which is the worst case and therefore the one a See-it
line is least likely to be run against:

```bash
for room in laboratory archive lens grimoire arsenal tower; do
  ORBS_BOOT=0 ORBS_DUMP="attend $room; help" cargo run -q -p orbs; done
ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_DUMP="attend lens; help" cargo run -p orbs
```

**Two lints, and neither can see a *refusal*.**
`every_room_a_player_can_stand_in_explains_itself` requires all three keys for
every **built** domain (from `Sim::briefs()`) plus `arsenal` and `tower` by name;
`a_primer_only_names_words_that_room_actually_offers` checks each verb a primer
names against `offered`. The second caught `bind` in the grimoire — gated at
concentration 0. What it *missed* is `scribe`, which is offered in the grimoire and
refuses, so the first draft told the player to do the one thing that room cannot.
**Run the loop above rather than trusting the green** — that is the half only eyes
have.

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
movement. **On the Ley Line left/right walks the stations and up/down picks a
fork's sibling** — rightward is progress, downward is a choice. **On Mastery
up/down picks a room and left/right walks its line** — there are no choices
there, and `take` is refused in voice. One track draws below the bar at a time,
and the word decides which.

**The session pane is 104 columns, not 120.** The tower rail takes 16 off the
right, so the grid's width is never this surface's. It **was 60** while the
second pane held telemetry; a surface authored against that has 44 columns of
slack it did not have. `ORBS_GRID=80x22` drops the rail entirely and is where a
sentence stops fitting — worth a look, even though the game itself no longer
reaches it.

```bash
# Nothing earned: the bar reads `0 of 56` — **against the line's last station,
# not a fixed hundred and not the next threshold** — and the Ley Line's steps
# and forks all draw `[·]` with their totals under them.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" cargo run -p orbs

# The seven rooms' lines. Every first station is `[○]` — the one being worked
# toward — and the rest `[·]`; the panel names the aimed deed and what it opens.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" ORBS_WEAVE="mastery" cargo run -p orbs

# One clarity reaches the laboratory's first station: `«•»`, the panel reading
# *a clarity brewed · 1 of 1 · reached · opens archive*, and the log saying so.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; kindle charcoal; \
  debug_spawn clarified-draught 1; distil clarified-draught; meditate 60; weave" \
  ORBS_WEAVE="mastery" cargo run -p orbs

# A fork **opening**, which is the only place the choose-between shape shows.
# Three alembic runs are 24; the fork's two nodes stack under its station and
# `<down>` aims at the second.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; kindle charcoal; \
  debug_spawn clarified-draught 3; distil clarified-draught; meditate 60; \
  empty alembic; distil clarified-draught; meditate 60; empty alembic; \
  distil clarified-draught; meditate 60; weave" \
  ORBS_WEAVE="ley\n<right>\n<down>" cargo run -p orbs

# `take` on a mastery station — refused in voice, because a station there is
# reached by doing its deed and never taken.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" \
  ORBS_WEAVE="mastery\ntake" cargo run -p orbs

# ...and the arrows before a word, which must move nothing and say what to do.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" \
  ORBS_WEAVE="<down>\n<right>" cargo run -p orbs

# Reach a station without doing its deed — and every earlier one on its line,
# since a line is walked in order. What each opens is opened and said.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_reach laboratory_2; weave" \
  ORBS_WEAVE="mastery" cargo run -p orbs
```

**Every fork node is real, and there are no markers.** `steps_1` and `steps_2`
grant spell steps; `satchel_1` and `cursors_1` grant §8's channel and its second
cursor. A fork node the orb cannot parse fails the load rather than drawing,
being aimed at, and refusing — which is the promise-about-nothing §19 refused
for the old tree.

**`ley::granted` is the one parser**, returning a `Grant` rather than a
`usize` — a boolean squeezed into the step-count parser is a `satchel_1` that
quietly hands out an instruction a tick. `Grant::lane` is derived from it the
same way, so a fork's siblings are always drawn provision, war, craft.

**Thirty mastery stations ship, and `Sim::new` is an open tower.** Every room,
every gated recipe and every charm is open in a dump, a test and a balance
policy — so a station's `opens` is said only in a *sealed* tower (`ORBS_SEALED=1`,
below), and `recall warding` answers here before the station that opens it. The
log line *"the laboratory line advances"* is what to look for; `sift` the log
for it.

**`debug_take <id>` skips the earning and nothing else**, which is `debug_spawn`'s
argument: 24 experience is two hundred ticks of the laboratory before a See-it
line about a gated word can start. Bare, it lists what there is to take.
**`debug_reach <id>` is its twin for a mastery station**, and reaches every
earlier station on the same line too.

## A sealed tower is a laboratory and nothing else

**`ORBS_SEALED=1` starts the dump where a player starts** (§11.5, Phase 10). The
game defaults to sealed and the dump to open — `orbs_shell::fresh` is the one
reader of the switch — so every other line in this file describes a tower with
all seven rooms, and that is deliberate: `Sim::new` is the open tower every test
and balance policy has always used, and a fresh *game* is `Sim::sealed`. What
opens what is `progression.toml`'s `opens`, and `Opened::start` is everything no
station names.

```bash
# Refused in voice from three doors — the room's name, a path into it, and the
# doorway — with the boot report naming the laboratory and the arsenal only.
ORBS_SEALED=1 ORBS_BOOT=0 ORBS_GRID=100x36 \
  ORBS_DUMP="attend archive; attend stacks; survey archive; survey /tower" \
  cargo run -p orbs

# ...and six dark boxes on the rail, which needs the deep grid to be drawn at
# all: the rail takes 16 columns and 100 is not wide enough to give them.
ORBS_SEALED=1 ORBS_BOOT=0 ORBS_GRID=120x45 \
  ORBS_DUMP="attend laboratory; attend archive" cargo run -p orbs

# One clarity opens the archive: the station, then *"the archive is yours now.
# attend it"*, then the room answering.
ORBS_SEALED=1 ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; \
  kindle charcoal; debug_spawn clarified-draught 1; distil clarified-draught; \
  meditate 60; attend archive" cargo run -p orbs

# The wall is armed by the sanctum's first station and the bailey follows it.
ORBS_SEALED=1 ORBS_BOOT=0 ORBS_DUMP="debug_reach laboratory_3; attend bailey; \
  debug_reach sanctum_1; attend bailey" cargo run -p orbs

# Two clarities is sixteen, and sixteen is the tower's own line rather than a
# room's: *"the orb can hold a spell now"*, then *"the grimoire is yours now"*,
# then the room answering. The forge is the same step at 56.
ORBS_SEALED=1 ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; \
  kindle charcoal; debug_spawn clarified-draught 2; distil clarified-draught; \
  meditate 60; empty alembic; distil clarified-draught; meditate 60; \
  attend grimoire" cargo run -p orbs

# Played rather than photographed: the same route, at a tester's pace.
scripts/play.sh sealed::
```

**`recall archive` falls through to the overview in a sealed tower** — the
room's manual page is hidden with the room — but the *word* still resolves,
deliberately: dropped from the scene, `attend archive` fuzzed into a numbered
prompt offering four other rooms, which is §15's dead end by the first word a new
player will try. `attend`'s gate is asked of the node, so `attend stacks` is
refused as *"the archive is not yours yet"* too.

**A charm is gated the same way.** `imbue whetted` is refused in voice until the
sanctum's second station; the forge opens knowing `hurried`.

```bash
# The forge is behind the ley step at 56 — seven distillations — and
# `debug_reach` opens the room the line it reaches stands in, so a dump can
# stand at the lattice: "the forge cannot lay whetted yet", then `hurried` rising.
ORBS_SEALED=1 ORBS_BOOT=0 ORBS_DUMP="debug_reach forge_1; attend forge; \
  imbue mortar_and_pestle whetted; imbue mortar_and_pestle hurried" cargo run -p orbs
```

**`debug_reach <id>` opens whatever had to open first.** It walks the chain — the
sanctum is the laboratory's third station, so `debug_reach sanctum_1` reaches
`laboratory_1..3` on the way. It used to reach a line inside a room the player
could not enter, and the workaround was written into two See-it lines by hand.

**Sabotage never strikes a shut room.** `tower::Sealed` marks every node under
one, and the two calm-layer queries carry `Without<Sealed>`; `tests/sealed.rs`
runs two hours of ticks and finds every shut room's log clean.

## The road under a room's title, and the rail's percentage

**Every open room draws its own mastery line one row under the pane's title**,
in the weave's marks, with the next deed and its count at the far end; and every
open room's rail box carries the percentage of that deed on its state row. Both
read from the same `Line` the weave draws, so the three cannot disagree.

```bash
# Three potions in: the road reads `laboratory [•]─[○]─[·]─[·]─[·]─[·]─── five
# potions brewed · 3 of 5`, and the rail box `burning  60%`. The deep grid, so
# the rail is drawn at all.
ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="attend laboratory; kindle charcoal; \
  debug_spawn clarified-draught 3; distil clarified-draught; meditate 60; \
  empty alembic; distil clarified-draught; meditate 60; empty alembic; \
  distil clarified-draught; meditate 60" cargo run -p orbs

# A finished line: the road ends *"every station reached"* and the rail box
# carries no percentage, because a number about nothing is nothing.
ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="attend laboratory; debug_reach laboratory_6" \
  cargo run -p orbs

# The primer's last line says the same thing in words.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; recall" cargo run -p orbs

# ...and the 80×22 floor, where the sentence at the road's end is the first
# thing to go and `3 of 5` is the last.
ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_DUMP="attend laboratory; debug_reach laboratory_1" \
  cargo run -p orbs
```

**The road yields before the transcript does.** It takes the top row of the
body only while the body keeps six rows after it, and a shut room or a finished
line with nothing to say draws none. The rail's percentage sits on the state
row's right edge because `MIN_RAIL_BOX` has no spare row, and it is silent glyphs
— §14's rule for progress is completion only, and the station reaching is what
gets said.

## The Ley Line to the soft ending, and what its forks grant

**Sixteen stations from 16 to 10,000** — nine steps to concentration 8 and seven
forks — and the run draws on a **logarithmic** scale, because stations growing by
half each stood in a knot at the left of a linear line. Position is still cost;
the bar fills to the cell the total has reached; the totals alternate between two
rows.

```bash
# The whole line, forks three deep — provision, war, craft, top to bottom —
# `40` aimed at its war node, and the panel naming it, its cost and its lane.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="weave" \
  ORBS_WEAVE="ley\n<right>\n<right>\n<down>" cargo run -p orbs

# Each grant, on the surface it moves. `debug_take` holds the node without
# the experience, which is the tool's one job.
ORBS_BOOT=0 ORBS_DUMP="debug_take fuel_1; attend laboratory; kindle charcoal" \
  cargo run -p orbs                          # "fuel for 720 ticks", not 600
ORBS_BOOT=0 ORBS_DUMP="status; debug_take pool_1; status" cargo run -p orbs
                                             # the ceiling row, four higher
ORBS_BOOT=0 ORBS_DUMP="debug_take thrift_1; attend forge; imbue mortar_and_pestle hurried" \
  cargo run -p orbs                          # the price quoted, two lower
ORBS_BOOT=0 ORBS_GRID=120x45 ORBS_DUMP="debug_take edge_1; attend bailey; defend" \
  cargo run -p orbs                          # the board's garrison odds: 55%, not 50%

# The other seven — haste, escrow, garrison, mend, floor, vigilance, steps_3 —
# each against its own number:
cargo test -p orbs-sim --test grants
```

**The line packs itself to the pane it is given.** At every pane the game can
draw today it is framed `[·]`, because there is one pane and the narrowest grid
still leaves it sixty columns. **The narrow case is Phase 11a's**: a second pane
halves the main window to 52 columns, which leaves the track 39 cells where
sixteen framed stations want 48 — so the frames come off and the marks stand two
apart, the aimed one keeping its frame, which is what carries *aimed* without
colour. Narrower than even that needs and the screen says *"the orb needs a
larger window"* rather than drawing a line with its tail clipped off the edge.

**No dump can show it**: `PANES` is 1 and the dump never splits. The gate is
`cargo test -p orbs-shell --lib loom`, which walks every width from the tight
packing to 120 and asserts no station leaves the run and no two share a cell.

**A node held above its fork's total is kept.** `debug_take cursors_1` at nought
experience, or an older save from before the node moved to 400: the node stays
in effect and the fork at 400 reads as chosen, because `ley_line` derives *spent*
by membership. The plan's "drop it and say so" fought the tester's shortcut
across a save and lost (§19).

**`haste` is the orb's speed and never the tool's.** `debug_take haste_1`, then
the same `grind sage` by hand and from a spell: the spell's lands a tenth sooner
and the player's does not — `tests/grants.rs` measures both. That is struck
invariant 3, bought.

**The archive draws a map, and `wander` gives it the arrow keys.** The map is
*not* gated on the word — it draws whenever a maze is open, which is what makes a
bound solver watchable — so `research` alone is enough to see it. `wander` only
decides who the arrows belong to, and `ORBS_WALK` presses them:

```bash
# The maze beside the transcript. **There is no fog** (§19) — every wall is on
# screen from the moment it opens; what fills in as you walk is the marks.
# **The whole picture now fits at the game's own grid**, and used to pan: the
# session pane was 58 columns against a 35-column maze, and the tower rail taking
# the telemetry pane's place left it 102. Panning is still what `stacks::split`
# does below that — `ORBS_GRID=80x22` is where to see it — and it is the designed
# fallback rather than a defect.
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

**Leaving a surface on a *held* key is a case no dump can reach, and it shipped
broken.** Escaping the maze with a finger still on an arrow put `wander` back in
the prompt: key repeat kept arriving, the maze had let go, and an arrow at the
prompt means *recall history*. `HeldOver` drops keys whose press went to another
surface until they are released — see §19, including why the watch that notices
the handoff has to be **ungated** (`type_into_line` is gated on a keystroke
arriving, so the frames it needs to have seen are the ones it never runs on).

It applies to all four surfaces, not just the maze — the editor and the weave
screen hand the keyboard back the same way. **`ORBS_WALK` cannot show any of it**,
because a dump writes no key events and holds nothing down; the gate is
`leaving_the_maze_leaves_nothing_in_the_prompt`, which needs real `KeyCode`s
because the harness's own `press` stamps every key `KeyA`.

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

**A spell asks `if the cabinet has 4 fragment` before it moves any**, and that
guard is what the count was added for. Without it a spell cannot tell one
fragment from four, so the only shape available was `repeat 4 / move / end` fired
blind — which moves whatever is there, wields a short lectern, and says *"the
lectern can do nothing with fragment"* on every lap for ever. `debug_spell
assembling` writes the guarded version.

**At least, never exactly**: `has 4` stays true at five, or the guard jams the
moment a solver gets ahead of it. `has 1 X` is what a bare `has X` already meant
and writes back bare; `has 0 X` is `has no X`.

```bash
# 2 fragments: nothing moves and the log stays quiet. 4 and 5 both assemble.
ORBS_BOOT=0 ORBS_DUMP="attend archive; debug_spawn fragment 2; debug_spell assembling" \
  ORBS_THEN="invoke assembling; meditate 60; peruse archive.log" cargo run -p orbs
```

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
line: 2, message: the reading goes west
line: 3, message: the reading goes west
```

**An arrow moves the reading immediately and consumes no tick.** `Sim::walk` is a
**third entry point** beside `submit` and `step` — the only thing in the game
that reaches the world without a tick boundary — so a player walks as fast as
they can press and no brew advances while they do. Three versions went through
the prompt's queue first; all were some flavour of too slow, because the queue
was solving the wrong problem.

`ORBS_WALK` therefore steps **nothing**: check `tick` at the tower rail's foot after
a long walk and it should be exactly what it was before. If a dump of eight
presses has advanced the clock eight seconds, something has gone back through
`submit`.

**Replay still holds**, and `Submission::Walked` is what makes it hold: it says
*when* — a typed line executes at the start of the next tick, a walk has already
executed. Use `Sim::replay` rather than matching on `Submission` by hand; three
test files had their own copy of that match and they are one now.

**Watching a spell solve it is the point of the map, and `threading` is the
solver.** The ladder is twenty-four rungs across fifty-two lines — it was
ninety-eight before `else if` — and it lives in
`crates/orbs-sim/content/dev_spells.toml`.

**Every dev spell is on the grimoire's shelf from tick 0 in a debug build**, so
`invoke threading`, `scribe threading` and `peruse threading.spell` all work with
no `debug_spell` first. A shelved spell carries its own `Domain` from the file,
which is why it needs no room check — `scribe::write` homes a *new* spell to where
you stand, and nothing shelved is new.

**A release build has none of it**: `raise_grimoire` shelves them under
`cfg(debug_assertions)` and `dev_spells.toml` is `include_str!`'d under the same
`cfg`, so neither the nodes nor the lines exist. Both halves are held —
`the_dev_spells_are_on_the_shelf_from_the_first_tick` and, in release,
`a_release_shelf_holds_only_the_shipped_spells`.

`debug_spell <name>` stays for the other job it does: handing back a **fresh
copy** after one has been edited or purged. It still refuses outside the spell's
own domain, because that path really does write a new file.

```bash
ORBS_BOOT=0 ORBS_DUMP="survey grimoire" cargo run -p orbs      # all five, shelved
ORBS_BOOT=0 ORBS_DUMP="debug_spell" cargo run -p orbs          # what it can rewrite

# An ordinary maze: the way out, and a fragment for reaching it.
ORBS_SEED=11 ORBS_BOOT=0 ORBS_GRID=160x45 \
  ORBS_DUMP="attend archive; research; debug_spell threading" \
  ORBS_THEN="invoke threading; meditate 3600; meditate 3600; survey cabinet" cargo run -p orbs

# The **same file** on the other errand: no way out, five things to gather.
ORBS_SEED=17 ORBS_BOOT=0 ORBS_GRID=160x45 \
  ORBS_DUMP="attend archive; research; debug_spawn gleaning-scroll; \
    wield gleaning-scroll; debug_spell threading" \
  ORBS_THEN="invoke threading; meditate 3600; meditate 3600; survey cabinet" cargo run -p orbs
```

**Do not hand-build the four-tier ladder**, and note that two of its words no
longer exist. `exit`/`passage`/`walked`/`twice` is what §19 calls *"the solver
that was never a solver"* — measured at equal laps and ticks it solved seed 3 and
**failed seed 11**, because without a `back` rung a fixed compass order sends the
reading back where it came from at any junction where two ways read alike, and it
cycles. It also had no `spoil` rung, so it could never glean. That recipe was in
this file for two versions and it was the broken one.

**`walked` and `twice` are gone**; a way reports `marks`, a count, and the
middle tiers compare it — `1 or fewer marks` and `2 or more marks`. They were
two buckets over a `u8` the maze had all along, so a square walked nine times
read exactly like one walked twice.

```bash
# What a way says now. Used to read `back walked`.
ORBS_SEED=3 ORBS_BOOT=0 ORBS_GRID=160x45 \
  ORBS_DUMP="attend archive; research; follow south; follow south; follow north; survey south" \
  cargo run -p orbs
# -> back    marks = 1
```

**The language has twelve control words**: `wait`, `repeat`, `if`, `else`, `end`,
`until`, `let`, `for`, `part`, `bide`, `pull`, `alongside`. **Count them from
`SpellWord::ALL`, never from this sentence** — it said nine while the answer was
eleven.

### The satchel — one spell hands another a name, and `alongside` forks a cursor

**Two spells already ran at once, and that is where to start.** `invoke` from
inside a spell inserts a second `Running` and the caller does not block;
`run::advance` steps every one each tick with its own budget — ungated, at
concentration 0. So §8 never lacked concurrency. It lacked a **channel**, and the
satchel is it.

```bash
# A queue loaded by hand and read back. A name twice, in the order it went in.
ORBS_BOOT=0 ORBS_DUMP="attend menagerie; debug_take satchel_1; queue skyward; \
  queue earthward; queue skyward; survey satchel" cargo run -p orbs
```
```text
queued ───────────────────────────
  skyward    earthward  skyward     ← a count would say `skyward 2` and lose the order
```

**`queue` is a verb and `pull` is a control word, and the asymmetry is load-
bearing.** Pulling *binds a name* and `let` is the only other thing that does; a
verb runs through `execute::dispatch`, which hands back records and touches
nothing a spell holds — so a `pull` verb could empty the satchel and have nowhere
to put what it took. `queue` is a verb because it changes a node, and because a
player who cannot load one by hand cannot watch a consumer drain it.

**A component, not children**, because a queue is an ordered multiset and `Stock`
collapses duplicates. That means `spell::watch` needs its own arm — **without it
`if the satchel is empty` is true of a full satchel**, for ever and silently,
which is the third time §19 records that shape.

**One per domain, all called `satchel`, and `scene_at` offers only the local
one.** That last clause is what makes `build`'s exemption from
`every_place_leaf_is_unique` true rather than argued: every place is registered
by path and §6's matcher takes a last segment, so six satchels meant the word
resolved to whichever was registered *first* — `queue` filled the menagerie's and
`survey satchel` read the **laboratory's** and called it empty, one line apart, on
the first See-it line run. **Check both rooms after touching `scene_at`.**

**`pull` yields while empty and never reaches `PATIENCE`.** A consumer caught up
with its producer is a working pipeline; latching `‼` for it would make the fault
light useless in the room most likely to show it.

```bash
# `alongside` — both halves in one file. The consumer is cursor 0 and blocks on
# an empty satchel until the forked producer fills it.
ORBS_BOOT=0 ORBS_GRID=110x40 \
  ORBS_DUMP="attend menagerie; debug_take satchel_1; debug_take cursors_1; scribe both" \
  ORBS_EDIT=$'edit\npart filling()\nqueue skyward\nqueue earthward\nend\nalongside filling()\nrepeat 2\npull note from satchel\nsurvey note\nend\n<esc>\nquit' \
  ORBS_THEN="invoke both; meditate 12; peruse menagerie.log" cargo run -p orbs
```

**`Progress::Blocked` yields the *cursor*, not the entity, and that is the whole
refactor.** `step_one` used to `return` on a block, ending the spell's tick — so
the consumer above would have ended it before the producer was ever reached, on
every tick. **The deadlock was by construction.**

Five rules that came with it, none of which is the obvious default:

- **`seen` is per-cursor.** It is the record-stream mark a `wait` reads, so two
  cursors sharing one makes A satisfy B's wait — silently, and only for spells
  using `wait`.
- **Batch, not round-robin**: each strand spends its whole budget before the
  next, in `Vec` order, matching `advance` one level up. **The two are identical
  at budget 1**, so a determinism pin written at the shipped budget pins nothing —
  `two_cursors_interleave_the_same_way_at_two_steps_a_tick` earns its second step.
- **`remove`, never `swap_remove`.** Reordering live cursors breaks replay for
  any spell that outlives a fork.
- **The spell ends when every cursor has**, which is what `ended` is for.
- **`MAX_STRANDS` is 4** — `MAX_PARTS`'s argument, and a strand is heavier
  because each spends its own budget.

**The active cursor lives in `Running`'s own fields and the rest are parked.**
`swap_in`/`swap_out` are the only two functions that know, which is `Cwd`'s idiom
one level down and is what kept ~110 call sites untouched. The cost: those fields
mean *the cursor currently stepping*, so `Sim::running_line` and the editor's
gutter get whichever was put back last.

**The menagerie is not this feature's use case, and it was measured.**
Identifying one of four lanes costs six to ten steps against a `PACE` of four, so
a producer queues **4 of 12 at one step a tick and 6 at two**, with duplicates. No
queue depth fixes it — the cost is *identifying*, not seeing far enough — and
that is `the_pace_is_shorter_than_a_four_lane_ladder` working. A version letting
`queue` take a **reading** solved it at budget 1 and is withdrawn as `bide until`
in a new hat (§19).

**Three dev spells are the worked examples, and both forms are shipped.**
`ordering` + `milling` are the *two-spell* channel; `coursing` is the
*two-cursor* one. All three are on the grimoire's shelf in a debug build, so
`invoke` reaches them with no `debug_spell` first.

```bash
# Two spells, one channel. `ordering` queues three reagents and stops; `milling`
# invokes it and grinds what it left behind. Three loads, `+1` each.
ORBS_BOOT=0 ORBS_DUMP="attend laboratory; debug_take satchel_1" \
  ORBS_THEN="invoke milling; meditate 80; peruse laboratory.log" cargo run -p orbs

# One spell, two cursors: `holding` split down the middle. A planner queues the
# station pairs, a mover pulls two and hauls. **15 hauls for four wards** — the
# same 2^n-1 optimum `holding` reaches, which is the point of the test.
ORBS_BOOT=0 ORBS_DUMP="attend sanctum; debug_take satchel_1; debug_take cursors_1" \
  ORBS_THEN="invoke coursing; meditate 400; peruse sanctum.log" cargo run -p orbs
```

**`milling`'s `bide 2` is load-bearing and reads like superstition.** `advance`
snapshots the running list, so `invoke ordering` starts the producer on the
*next* tick — and the guard is `repeat until the satchel is empty`, true of an
empty one. Without the pause the loop runs zero times and the spell ends having
done nothing, silently. That is `repeat until the circle is idle` (§19) through a
different door, and it is the trap a first pipeline falls into.

**`coursing`'s `if the satchel is empty` is backpressure, written by the
player.** The planner is six queues a lap against a mover spending nine steps on
one haul, so ungoverned it fills to `DEPTH` and then refuses once a lap for the
rest of the course. Refilling only when drained is what keeps the log readable.

**The rail counts cursors, not spells** — `►coursing +1` for one forked spell and
`►both +2` for a fork plus a second `invoke`. From the rail they are one fact:
*more is running here than this line can name*. `status` gained a `casting`
section that says which, and both read one walk (`tower::running_spells`) so they
cannot disagree. The name truncates and the count never does.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend sanctum; debug_take satchel_1; debug_take cursors_1" \
  ORBS_THEN="invoke coursing; meditate 6; status" cargo run -p orbs
#   rail: ►coursing +1        status: coursing  sanctum, on 2 cursors
```

**`debug_take <id>` is why these lines are short.** `satchel_1` opens at 24 and
`cursors_1` at 40 — two hundred ticks of laboratory before a line about `queue`
could begin. It grants the real node and refuses a marker. Bare, it lists.

```bash
ORBS_BOOT=0 ORBS_DUMP="attend menagerie; queue skyward; debug_take; \
  debug_take satchel_1; queue skyward" cargo run -p orbs
cargo test -p orbs-sim --test satchel --test strands   # nineteen claims
scripts/play.sh satchel::                              # five, on a real keyboard
```

**`cursors_1` sells ergonomics and the tree says so.** Two spells were always
free; what it buys is both halves in one file. A node implying otherwise would be
selling something the player already has.

**`part between(here, there)` names a run of lines and `between(a, b)` runs it.**
A definition is stepped *past* where it stands — a spell is read top to bottom
and its parts are written among its lines — and a call is a stack of
**descents**, because a `pc` addresses one tree and a part is a different tree.
The brackets may be empty (`part gathering()`) and may be dropped entirely on a
definition that takes nothing; at a call site they are the notation and are
always required.

**A spell is contained to a single `.spell` file, and that is a decision** (§19).
Two spells cannot share a part; `program::tree` looks only inside the running
spell's own body, and `a_part_is_not_reachable_from_another_spell` holds it.
Cross-file sharing is **struck**, not deferred — with it went *"what a part costs
to hold"*, because an in-file part is not held separately and so has no price to
draw. A spell reaching another spell is `invoke`, which is a second `Running`
with its own budget rather than a descent.

```
part load(what)
    grind what
    empty mortar_and_pestle
end

load(sage)               ← one body, two reagents, and no `let` between them
load(rock-salt)
```

**The call is punctuation, and it is the one place a symbol is canonical.**
§19's comparison spellings accept `>=` and write back words because a word exists
to write back to; here none does, so `()` *is* the notation. It also means the
language spends no word on calling — `part` is the only word this cost.

**A definition belongs at the top level and a name means one part.** Both are cut
out and said rather than silently ignored; **six** complaint keys now, each
naming its line. Recursion is bounded at `MAX_PARTS` (8), separate from
`invoke`'s `MAX_DEPTH` — at one step a tick a runaway does not hang the game, it
grows the **save** by a descent a second.

**A part's brackets are the whole of what it can see** (§19, `0.4.2`). This
reversed *"variables are shared, not per-descent"* by removing its premise rather
than by overruling it: that decision rested on *"a part takes no arguments, so a
private store would leave it with no way to be told anything at all"*, and a part
takes arguments now. So `vars` rides the `Descent`, the roadmap's original
`(spell, pc, loops, vars)` shape is what shipped, and `for each way` inside a part
no longer rebinds the caller's `way`.

- **An argument is resolved one level, in the caller's store.**
  `between(wellspring, near)` hands over whatever `near` stands for; a literal
  stands for itself. `substituted`'s rule, at a call site.
- **A wrong count is a refusal, never a name bound to nothing.** There is no
  default and no overload — an unfilled parameter would leave the body asking
  about a name that stands for itself, which resolves against the room and does
  something quietly. Checked at cast *and* in `called`, because §8 hot-reloads a
  definition under a running spell.
- **A parameter may not repeat; an argument may.** `part between(here, here)`
  shadows and is refused; `between(here, here)` is two slots given one name.
- **`bindings` is still file-wide and that is not an inconsistency.** It feeds
  one lint — do not resolve a line naming a variable against the room — where a
  name too many is harmless and a name too few paints a working line red.

**`SCRIPT_BUDGET` is the floor and `spell::budget(world)` is the number.** It
reads `Taken`; `steps_1` and `steps_2` are authored in `progression.toml` and
ship as markers like every other node, so it answers 1 today and what was built
is the wiring. **`holding` is the one dev spell that uses a part** — the
tick-per-line arithmetic still says factoring is slower, and what changed is that
a call now says what it hands over: the sanctum's loop body went nine lines to
three.

```bash
# A part told what to work on. `ground-sage`, then `ground-salt`.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; scribe tending" \
ORBS_EDIT="edit\npart load(what)\ngrind what\nempty mortar_and_pestle\nend\nload(sage)\nload(rock-salt)\n<esc>\nquit" \
ORBS_THEN="invoke tending; meditate 40; peruse laboratory.log" cargo run -p orbs

# The scope. The part binds its **own** `herb` and grinds rock-salt; the
# caller's `herb` is untouched, so the line after the call still grinds sage.
# Under the shared store this ground rock-salt twice.
ORBS_BOOT=0 ORBS_GRID=100x36 ORBS_DUMP="attend laboratory; scribe scoped" \
ORBS_EDIT="edit\npart load()\nlet herb be rock-salt\ngrind herb\nempty mortar_and_pestle\nend\nlet herb be sage\nload()\ngrind herb\n<esc>\nquit" \
ORBS_THEN="invoke scoped; meditate 40; peruse laboratory.log" cargo run -p orbs
#   rock-salt: dispensary to mortar_and_pestle
#   sage: dispensary to mortar_and_pestle

# All six refusals. Line 15 is a **stray `end`** and that is the refused
# heading above it working — an unreadable heading opens no block.
ORBS_BOOT=0 ORBS_GRID=110x40 ORBS_DUMP="attend laboratory; scribe broken" \
ORBS_EDIT="edit\nmissing()\npart gathering()\ngrind sage\nend\npart gathering()\nsurvey\nend\nrepeat 2\npart inner()\nsurvey\nend\nend\ngathering(sage)\npart twice(a, a)\nend\nhauling(a b)\n<esc>\nquit" \
ORBS_THEN="invoke broken; meditate 3" cargo run -p orbs
```

**A call is cut by bracket and comma, never by space, and the lexer is where
that bites.** `between(wellspring, near)` is *two* whitespace-separated words, so
the word loop asked `is_call` of `between(wellspring,` and of `near)`, got no for
both, and drew the line as two ordinary names — the feature absent on screen with
the parser working perfectly. Same shape as §19's `▪`. The name and its brackets
are the call; the arguments draw as **names**, so a call of two places reads as
one. `ink` is the only instrument that can see either half.

```bash
scripts/tui.sh start
scripts/tui.sh type 'attend sanctum' 'scribe hues' 'edit' \
  'part between(here, there)' 'haul here there' 'end' 'between(wellspring, conduit)'
scripts/tui.sh ink 5 34    # `p`,`b`,`(`,`)`:magenta/bold — `here`,`there`:default
scripts/tui.sh stop
```

**Close the part before reading the colours.** An unclosed `part` faults its line
to `Role::Danger` and an accent outranks a hue, so a half-typed heading reads
entirely red and looks exactly like the highlighting never arrived.

### A spell is highlighted, and `ink` is the only thing that can see it

**A spell's parts carry a weight *and* a hue**, on two axes. §4 gave ordinary
text *"intensity variation alone"* and was **widened at `0.3.35`** to exempt
spell text — the language outgrew what three weights can separate, and `is` and
`the` are opposites that read identically. §19 records the supersession.

| weight | |
|---|---|
| **bold** | `repeat` `if` `else` `end` `part`, and a `gathering()` call |
| normal | verbs, names, numbers, grammar, states |
| dim | filler the orb strips, and `#` comments |

| hue | |
|---|---|
| magenta | control words, and a call |
| yellow | verbs |
| dark-yellow | numbers |
| dark-cyan | **grammar** — `is` `has` `be` `each`, the comparison spellings |
| cyan | **states** — the eight spellings of idle/working/empty |
| *the base* | names, filler, comments |

**Weight still carries the whole reading on its own**, and that is load-bearing:
`ORBS_DUMP` has no colour, a greyscale tube is a §14 case, and the `monochrome`
theme declines the palette outright. Take every hue away and a spell reads as it
did before hue existed.

**`Grammar` is the variant that earned the change.** It was `Filler` and is
filler's *opposite* — filler is what §6 strips, grammar is what the question
turns on — so `is` and `has` looked exactly like the `the` beside them. No amount
of weight could have separated them.

**`empty` is a verb and a state**, the only word that is both, so the state check
sits *below* the verb check in `classify`. Above it, `empty mortar_and_pestle`
draws its verb as a state.

**The hue is a `Frame` side-table, never a fifth `Style` byte.** `Cell` is pinned
at eight and a fifth field cost +28 KiB and ~1.2 µs a frame, measured. A syntax
run is a *region* exactly as an instrument's bar is, so it rides
`Vec<(Rect, Lexeme)>` — and is kept **apart from `Tint`**, which means materials:
the laboratory draws an instrument panel two columns from the editor.

**`parser::lexeme` classifies and `sheet` draws.** The classification is a
*language* fact and lives in `orbs-sim` beside `read` and `analyse`; it is
**lexical and world-free**, so a spell keeps its shape read from another room and
does not change appearance as its own pipeline fills the shelf.

**It declines on an accent and on the running line.** A line `interpret` could not
read stays wholly red; the line the orb is executing stays uniformly bright. Same
rule `Style::depicted` has, and the reason `sheet` announces a row once and draws
its runs silently — highlighting multiplies the runs per line by five or six, and
under the old shape every one would have been a separate utterance.

**A dump prints the runs and cannot print the colours**, so there are two gates
and you need both:

```bash
# What the sim classified. The only text gate a hue has.
ORBS_BOOT=0 ORBS_GRID=100x30 ORBS_DUMP="attend laboratory; scribe hues" \
ORBS_EDIT=$'edit\nrepeat 3\nif the mortar_and_pestle is idle\ngrind the sage\nend\nend\n<esc>' \
  cargo run -p orbs | grep -A12 'lit runs'
#   control "repeat" / number "3" / grammar "is" / state "idle" / verb "grind"
```

```bash
# ...and what the terminal was told to draw. `ink` is the only instrument that
# can see either axis, and it is the one to use — see below.
scripts/tui.sh start
scripts/tui.sh type 'attend laboratory' 'scribe hues' 'edit' \
  'repeat 3' 'if the mortar_and_pestle is idle' 'grind the sage' 'end' 'end'
scripts/tui.sh ink 5 40
#   'r':magenta/bold … 'i':dark-cyan … 'i':cyan … 'g':yellow … 't':default/dim
scripts/tui.sh stop
```

**Use `ink`, never a hand-written SGR reader.** crossterm writes every named
colour through the **256-colour** form — `Color::Magenta` is `38;5;13`, not the
`35` an ANSI table predicts — so a parser written against `3x` reports the whole
feature missing. That happened, and the failure is maximally misleading: a broken
reader and a broken feature print identically.

**A well-formed spell, or you are testing the decline.** An unclosed `repeat` or
`if` faults its line to `Role::Danger`, and an accent outranks a hue — so a half-
written spell reads entirely red and looks like the colours never arrived.

### `tower::reach` — one rule for what a name reaches

**Every string→handle lookup goes through it**, and three of them used to be the
same walk written out three times (`navigate::find_script`, `bind::find`,
`invoke::find`). It names the four axes that are questions about the *world*:

| Axis | Settings |
|---|---|
| scope | `In` (children of one node) · `Under` · `Tower` · `Fetch(room)` · `Arsenal` |
| kind | any, or one `NounKind` |
| naming | `Leaf` · `LeafOrPath` (§7: a place answers to both) · `Script` (supplies `.spell`) |
| ordering | **`Fetch` is §10.1's search order and a game rule** — unbusy instruments in raise order, then stores, then the arsenal |

**The fifth axis is not here on purpose.** What a miss *says* — a record, a
`None`, an entry on a `missing` vec — is about what the player is told, and that
is presentation. It stays at the call site.

**A bare `look(world)` is §7's plain rule**: inside where you stand, any kind, by
leaf. Every widening past that is then visible at the call site rather than
buried in a helper's body, which is how *nameable* and *findable* came to
disagree twice (§19).

```bash
cargo test -p orbs-sim --test fetching --test arsenal   # untouched: the proof
cargo test -p orbs-sim --lib tower::reach               # each axis, pinned
```

**A sweep is the other proof.** The fetch order decides what `digest ground-sage`
picks up, so a refactor that moved it would move the economy — run
`cargo run -p orbs-balance -- sweep --ticks 7200` and every rate should be
unchanged to four decimal places.

### A spell's verb is read at cast; its arguments are not

**`may_issue` is asked twice, and that is deliberate.** It is a security boundary
— `attend`, `meditate`, `scribe`, `undo`, `bind`, `unfurl`, `weave`, `wander`,
`quit` — and `run_line` asks it only when a line is *reached*, which for a line
in an untaken branch is never. A `meditate 3600` sat in a spell saying nothing,
and a scripted `meditate` is a hazard: `Sim::step` drains `Skip` in a while-loop,
so an hour of world time runs inside one step.

**But only the verb can be checked at cast, and this is the trap.** A spell makes
its own inputs, so `digest ground-sage` is written above the line that produces
any — at cast the room has none and `interpret` reads the line back as bare
`digest`:

```text
   1 grind sage
   2 empty mortar_and_pestle
   3 digest              ← the reagent is gone. this is correct
```

**Freezing a whole `Intent` at cast would break every pipeline spell in the
game, silently.** The verb survives because a verb is offered by the fixture
standing in the room rather than by what is on the shelf. So `check_commands`
looks at the verb and must say nothing about the arguments — and
`the_cast_check_does_not_fire_on_a_line_that_makes_its_own_input` is what holds
that.

**Lines naming a variable are skipped**, because a variable holds nothing until
the line runs.

**`let <name> be <place>` binds a name, and a variable holds a *name*** — not a
number, not a list, not an expression. Everywhere a name may stand the bound word
stands for it, which is exactly what an accumulator needs and nothing more. The
value resolves against the room **at cast**, like every other name, so `let m be
mortar` binds `mortar_and_pestle`.

**It is `let … be` and not `set … to`, and both halves of that are collisions.**
`set` is already a `dial` synonym and a spell word is matched *before* the fuzzy
matcher, so `set second borax` at the prompt stopped reaching the lens; `to` is
§6 filler and would be invisible to every reader but the one that sees text
before normalisation. `spellword.rs` names three collisions it refused to add —
this would have been a fourth, on a shipped verb.

**`for each <set>` walks a set, and the cursor is named after it.** `for each way`
binds `way`, so the body reads `if way has spoil` and `follow way`. **`it` is
impossible**: it is on the filler list, so `follow it` is stripped to `follow`
before anything sees it.

**A set is declared by the fixture, never derived from `Role::Reading`** — that
marker covers the archive's four ways *and* the lens's four sockets *and* its six
sigils, so a loop over the marker would hand a spell in the lens ten things when
it asked for four. `recall scripting` lists a room's sets, and is the only place
that does:

```bash
for room in archive lens laboratory; do ORBS_BOOT=0 ORBS_GRID=100x40 \
  ORBS_DUMP="attend $room; recall scripting" cargo run -q -p orbs; done
#   archive: way.   lens: socket, sigil.   laboratory: no such section
```

**The *readings* that page lists are keyed on the set too**, and asking
`Role::Reading` instead shipped a real defect: the gate opened wherever any
child carried the marker and then printed `maze::readings()` regardless, so the
**lens** taught `passage wall exit back spoil marks gleaning` and named none of
the ward's six deltas — the whole of what a lens spell may ask. `tower::readings_at`
walks `groups_at` now. Check the `[nameable here]` block in all three rooms above
when touching either.

**`repeat until <question>` is the bound a loop should have** —
`repeat 20000` was a guessed constant chosen to outlast the longest walk, and the
surplus laps spun doing nothing once the maze closed. The guard is asked before
the first pass *and* at the end of each, so `repeat until <already true>` runs
zero times; a question it cannot answer **stops** the loop, which is the opposite
of `if`'s rule and deliberate.

**A comparison has seventeen spellings and one canonical form.** `has 2 X`,
`at least 2`, `2 or more`, `more than 1`, `>= 2`, `>=2` all mean at-least;
`at most 2`, `2 or fewer`, `fewer than 3`, `<= 2`, `<3` mean at-most; `exactly 2`
and `= 2` mean exactly. **Symbols are accepted and never written back** — the
fair copy is words, so a player who has never seen an operator can read it.
`interpret` is where you check a spelling survived, because the ones that did not
used to vanish in silence.

**`else if` chains, and one `end` closes the whole ladder.** A desugaring rather
than a `Kind` — the runner, the save format and `interpret` see the nested tree
that was always written by hand — so nothing downstream knows about it. It took
`threading` from 98 lines to 52: **49 of the 98 were `end` or `else`**.

**And the other side of a comparison can be a place**: `north has fewer marks
than east`, `more … than`, `as many … as`. Both sides go through one read of a
named child's `Stock`, which is the arithmetic `raise_count` already promised.

**The far side takes an arithmetic** (`0.8.16`), and §19 records this as the
reversal it is — the log said *"it is not getting arithmetic"* and *"an
expression tree: no"*, and both are struck through. The shape is
`[double] <place> [has <thing>] [plus n]`:

```
if the coffer has fewer quintessence than the d20 has quintessence
if the enemy has more spears than double the garrison
if the garrison has fewer mettle than the enemy has mettle plus 6
```

**Quintessence is what broke the old rule**, and it is worth knowing why the
maze's pattern stopped scaling: a derived word answers a *fixed* ratio, and there
is no word to publish for *is what I hold more than what this costs* — the answer
depends on two quantities the player is choosing between.

**Words never symbols, one operator, no precedence table, no brackets.** An
expression sits only on the far side, reads left to right, and cannot contain a
comparison — so there is nothing to bracket. **Subtraction is deliberately
absent**: `A − n > B` is `A > B + n`, and there is no clean word for it (`less`
is already `BOUNDS` grammar, `minus` scores 667 against `minute`).

**`plus` is a `STOPPERS` word now, which is a permanent reservation** — nothing
in the tower may ever be named it. `double` is not, because it is consumed ahead
of the place it modifies rather than being something a span runs through.

**Strict against a place, inclusive against a number** — `has 2 or fewer marks`
includes two and `has fewer marks than east` does not. That is English, not an
inconsistency, and *at least as many* is deliberately absent because
`not … fewer … than` says it.

**`strict` is asked of the grammar, never of the variant** — *"is the far side a
world read"*, not *"is it `Elsewhere`"*. Written the second way the three
expression variants all fall to inclusive, so `than the d20` and `than the d20
has quintessence` disagree at equality and **`plus 0` changes a sentence's
meaning**. That is the whole of why the affirmative affordability sentence is
unwritable: `has more quintessence than the d20` is strict, so it refuses the die
you can *exactly* afford, and the correct spelling is the double negative. The
coffer publishes only affordable dice so that a player never has to know this.

**Absent is nought, and the arithmetic makes that sharper rather than safer.**
`the enemy has mettle plus 6` is **6** when the enemy is routed and publishes no
`mettle` at all — the operator does not know it is adding to nothing. There is no
design fix, only the knowledge: **test every new comparison against an absent
reading**, because the wrong answer is a plausible number rather than an error.

**A comparison alone cannot pick "the way with the fewest marks", and the rung
that looks like it does is wrong.** A walled or unwalked way publishes no `marks`
node, so it counts as **nought** and is the minimum of any four — `threading`
rewritten that way solves no maze at all. Least-of-the-*open*-ways is a **filter
then a minimum**, which is what `let` and `for each` are for and what `roaming`
does:

```
let best be north                ← the seed. it may be walled; the next pass fixes it
for each way
    if way has no wall           ← any open way — the dead-end fallback
        let best be way
    end
end
for each way
    if way has no wall and no back        ← prefer one that is not where we came from
...
for each way
    if way has no wall and no back and fewer marks than best   ← the true minimum
```

**Later loops override earlier ones, so priority reads bottom-up.** The `passage`
tier `threading` needs is subsumed: an unwalked way publishes no `marks` at all,
which counts as nought, so it is already the minimum.

**Fewer lines is not fewer ticks.** `roaming` is 19 lines against `threading`'s
52 and costs ~27 steps a move where the ladder short-circuits at the first rung
that fires. A **five**-pass version — adding `exit` and `spoil`, which makes it
the same algorithm as `threading` — was measured at ~45 steps and stopped
finishing seed 3 inside 7200 ticks. Both ship, and the pair is the lesson: §19's
step-cost decision as a number rather than an argument.

```bash
ORBS_BOOT=0 ORBS_DUMP="peruse roaming.spell" cargo run -p orbs
for s in 3 11 17; do ORBS_SEED=$s ORBS_BOOT=0 ORBS_DUMP="attend archive; research" \
  ORBS_THEN="invoke roaming; meditate 3600; meditate 3600; survey cabinet" \
  cargo run -q -p orbs; done      # `fragment 1` each time
cargo test -p orbs-sim --test naming_things
```

```bash
# The comparison, and the two cases where it must not fire.
ORBS_BOOT=0 ORBS_GRID=110x34 \
  ORBS_DUMP="attend archive; debug_spawn fragment 4; debug_spawn fragment 1 lectern; scribe weighing" \
  ORBS_EDIT="edit\nif the cabinet has more fragment than the lectern\nmove fragment to lectern\nend\n<esc>\nquit" \
  ORBS_THEN="invoke weighing; meditate 6; survey lectern" cargo run -p orbs
#   4 vs 1 -> lectern 2.   2 vs 2 -> lectern 2.   1 vs 3 -> lectern 3.
```

**`recall` now teaches the language.** Nothing did before — control words are
outside `Verb::ALL` and readings are `NounKind::Sense`, so `recall repeat`
reached nothing and `recall marks` answered with a message about other rooms.

```bash
ORBS_BOOT=0 ORBS_DUMP="recall repeat; recall until; recall marks" cargo run -p orbs

# The page whose last section is the room. Run it in both and compare: the words
# and the question shapes are identical, what you can *name* is not.
ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend archive; recall scripting" cargo run -p orbs
ORBS_BOOT=0 ORBS_GRID=100x40 ORBS_DUMP="attend laboratory; recall scripting" cargo run -p orbs
```

**One ladder serves both errands, and no errand check is involved.** `spoil` and
`exit` are both tiers: a gleaning maze withdraws the way out and an ordinary one
scatters nothing, so the tier that does not apply is simply never true. `if the
stacks has gleaning` exists for a spell that wants to do something *else* per
errand. `cargo test -p orbs-sim --test gleaning` holds it across four seeds.

**`threading` solves the maze in front of it and stops.** It does not re-`research`
— a ladder whose fragment count keeps climbing cannot be observed for one walk.

**The maze is 176 cells in a 33×23 picture of squares, one character each** — a
wall is a square, not a line between two cells, so **one arrow press moves one
character**. It was cells with the walls between them, which draws `2w+1` across
and moved the reading two characters a step. At 33 wide the whole picture wants
`ORBS_GRID=160x45`; at the game's own 120×45 the inline map pans instead.

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
- the **tower rail's foot** tracks the drag on its `scale` row while `grid` holds
  — it was the telemetry pane's row until the rail replaced that pane
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

**The POST card names what the machine is made of, and each frontend answers for
itself.** `ORBS_BOOT=post` is halfway through the card, where only the studio has
typed itself — so the two version lines had no See-it line at all until
`:<fraction>` was added. The engine line is the one thing on that card that
differs between the builds, and each holds its own pin to its own manifest with a
test:

```bash
ORBS_BOOT=post:1 ORBS_DUMP=1 cargo run -q -p orbs        # ...bevy 0.19.0
ORBS_BOOT=post:1 cargo run -q -p orbs-tui -- --dump 1    # ...crossterm 0.29
```

`ORBS_BOOT` takes `dark`, `frame` or `post` for the dump — each with an optional
`:<fraction>` naming how far through — and `0` to skip the sequence in the
running game. Boot happens once per launch and runs for
thirteen seconds, so without the latter every "see it" pass on anything else
costs that wait. **`0` is the only skip there is** — the keypress skip was
removed (§19), so a player sits through the whole sequence every time.

### The terminal build — the game, driven and read back

**`ORBS_DUMP` is a still photograph and this is the running game.** A dump builds
no `App`, advances no clock and presses no keys, so every animated thing, every
*edge* and every interactive surface needs an environment variable of its own.
**Count them rather than quoting a number** — `grep -rhoE '"ORBS_[A-Z_]+"'
crates/ | sort -u` — this sentence has said eighteen through two versions in
which the answer was twenty-one, then twenty-three, and is now twenty-five.
`orbs-tui` needs none: it is the same `Frame`
through the same painters, with a real clock and a real keyboard, under `tmux`.

```bash
scripts/tui.sh start                      # 120x45 — `orbs_render::GRID`
scripts/tui.sh start 177 38               # ...or any size
scripts/tui.sh type 'attend laboratory' 'kindle charcoal' 'grind sage'
scripts/tui.sh key Down Down Left         # arrows and named keys — `key F5`
scripts/tui.sh wait 20                    # let the world run
scripts/tui.sh see                        # the screen, as text
scripts/tui.sh ink 158 159                # ...and what colour each cell is
scripts/tui.sh stop
```

**`ink` is the one instrument nothing else in the project has.** `ORBS_DUMP`
prints glyphs and a list of tinted *regions*; the terminal build resolves colour
somewhere else entirely (`orbs-tui/src/theme.rs`), so a fire drawn in the wrong
ramp is invisible to every other tool. `see -e` hands back escape sequences
nobody can read down a column. `ink` walks the SGR state machine and prints, per
cell, what the terminal was actually told to draw:

```text
  7  '█':dark-yellow/dim      ← the athanor, at its cool end
 30  '█':yellow/bold          ← ...and its core, twenty rows down
```

**It found a real one on its first use.** The flame ran `dark-red → white`, which
is red at the cool end and colourless at the hot one, against §19's *"the athanor
burns orange on every tube by decision"* and `ember.rs`'s `#CC7024 → #FFE375`.
Sixteen indices hold two colours in that family, so the other two steps come from
the **weight** axis — dim-dark, dark, bright, bold-bright. `every_ramp_climbs`
and `the_fire_is_orange_all_the_way_up` hold it now.

...which is `tmux` and nothing else, if you would rather see it:

```bash
cargo build -p orbs-tui                   # **build first** — `cargo run` prints
tmux new-session -d -s orbs -x 120 -y 45 \#   its progress into the pane
  target/debug/orbs-tui
tmux send-keys -t orbs 'attend laboratory' Enter
tmux capture-pane -t orbs -p              # `-e` keeps the colour escapes
tmux kill-session -t orbs
```

**Resize it while it runs** — that is a screen worth looking at, and it was
broken until `blit::Screen::resize` learned to wipe. The layout reflows, the rail
drops into the border title when the columns run short, and below 80×22 the
"orb needs a larger window" card takes over:

```bash
for s in 170x45 60x18 140x40 90x24 120x45; do
  tmux resize-window -t orbs -x "${s%x*}" -y "${s#*x}"; sleep 1.6
  tmux capture-pane -t orbs -p | head -2
done
```

**Pace typing to the tick, or commands batch.** `Sim::submit` queues a line for
the *next* tick, so two lines sent inside one second land on the same tick with
no time passing between them — `empty mortar_and_pestle` and the `grind` that
needed it arrive together and the second refuses. `tui.sh type` sleeps 1.1 s per
line; `tui.sh key` does not, because `Sim::walk` is the third entry point and
spends no world time at all. **Check `tick` at the rail's foot after a long
walk**: eight arrow presses must not have advanced it eight seconds.

**Escape followed by a letter is two keystrokes and a terminal cannot say so.**
Escape is one byte, `\x1b`, with no terminator; crossterm reads the pty in one
syscall, so `\x1b` and anything behind it in that read come back as
`Alt+<letter>` — the Escape **gone**, the letter delivered. Closing the editor and
typing `quit` fast enough put `quit` in the buffer as a line of the spell.

`drive::escape_prefixed` splits the pair back apart. It answers **only for
`KeyCode::Char`**, because that is the one shape an `\x1b <byte>` pair builds; a
real `Alt+Left` arrives CSI-encoded and is left alone, or a player would leave
the editor by pressing word-left. Suspect this whenever a key "does nothing" in
`orbs-tui` and works with a `sleep` in front of it — and note `ORBS_DUMP` writes
no key events, so it can never see any of it.

```bash
cargo test -p orbs-tui --bins escaping   # both directions of the split
```

**All four surfaces work, and three of them are keys rather than words.**

```bash
scripts/tui.sh start
scripts/tui.sh type 'attend archive' 'research' 'wander'
scripts/tui.sh key Down Down Left         # the reading walks; `▒` marks the trail
scripts/tui.sh key Escape                 # ...and the prompt has the keys back
```

- **the editor** — `scribe <name>`, then `edit`, the lines, `Escape`, `quit`.
  There is still no `save`: stop typing and the buffer writes itself out a beat
  later, which is the same `Editor::settle` the Bevy build runs.
- **the weave screen** — `weave` opens it and **`quit` closes it**; `Escape` only
  returns to command mode, exactly as on the other side.
- **the maze** — `wander` hands over the arrows, `Escape` gives them back.
- **the transcript** — `unfurl`, then `PgUp`/`PgDn`, `Escape` out. It pages by
  *records*, measured with `orbs_shell::page_step`, which both frontends share.
  **`PgUp` works without `unfurl` too**, in both builds: the word exists because
  the *key* could not be discovered, not because the key needs permission.

**Every key the Bevy build binds, the terminal build binds** — and it shipped
once without them, while `prompt.rs` drew `F4 deep` into the border on every
frame. A key the screen offers and the build ignores is §19's *"the first
affordance the game showed was one that did not work"*, so the rules behind them
live in `orbs_shell::shortcuts` where a second copy cannot go missing.

| Key | Does | In a terminal |
|---|---|---|
| `F4` | flips the focus mode | the border's own hint moves; **the tiling does not**, in either build, until Phase 12a returns the second pane |
| `F5` | §14's linear stream | the pane describes itself instead of drawing — the accessibility route, and this is the build §14 calls the cheapest one |
| `F6` | writes `orbs-parse.tsv` | silent on success in both builds; the file appearing is the confirmation |
| `F7` | cycles the tonal register | **visibly inert** — `Presentation` picks a glyph-atlas *face* and a terminal has the user's. The world still moves, and `F6`'s `register` column shows it |
| `F9` | §14's patient chant | **not inert**, unlike `F7` and `F8`: it changes what a strike is worth, which is world state, so a chant sung patiently in a terminal reaches the same troops as one under Bevy |
| `F10` | leaves | as it does under Bevy. `Ctrl-C` and `Ctrl-D` also do, because raw mode makes them ours to answer |

**`F5` is inert over the editor, the loom and the maze, and that is a known
hole rather than a rule.** `prompt::paint` returns early for all three modal
surfaces, so the mirror never runs on the three screens that take the *whole*
pane. Reach their stream with a dump instead — it is the same `Speech`:

```bash
ORBS_BOOT=0 ORBS_DUMP="attend archive; scribe threading" cargo run -p orbs
```

**Read the stream, not only the screen, whenever you touch a painter.** A row
drawn as two `Painter::span`s is *two utterances*, and nothing on screen says so
— the editor spoke `1` and `attend laboratory` separately for four phases with
the comment above it claiming otherwise. `span` speaks, `glyphs` is silent, and
`announce` says a whole row that had to be drawn in several styles. A run split
for **colour** is not a sentence boundary (§19, `0.3.23`).

**...and `quit` is the word for it**, in both builds. Every key above was
undiscoverable — §6 makes this a game played by typing, and nothing on screen
said how to stop. It is the fifth take-once handshake beside `scribe`, `unfurl`,
`weave` and `wander`: the sim records the decision, the frontend decides what
leaving means. **It lands on the tick**, like every verb's effect — a frontend
checking at `submit` time finds the flag unset, which is what the first attempt
did.

`logout`, `put it down` and `stop playing` reach it too. **`leave` and `exit` do
not, and §19 records why**: both fuzzy-collide (with `weave` and `edit`), and
that is the one ambiguity prompt that would offer ending the session beside a
verb people type constantly.

### `quit` asks once before it leaves

**It is the one word in the game that cannot be undone.** Everything else can be
waited out, repeated or reversed; this ends the session and writes the tower on
the way — so a `quit` meant for the spell editor, typed one surface too high,
would end the game instead of closing a buffer.

So it asks, **and the answer is another `quit`**. Any other command answers *no*
and the question is gone: it lasts exactly one line, so a `quit` thought better
of cannot end a session three commands later.

**A word, not a keypress, and not a screen.** A confirmation surface would be one
more thing that takes the keyboard — and one that opens *because a keystroke
arrived* reads the keystrokes that opened it, which this project has now paid for
once (§19, and the section below).

**`F10`, `Ctrl-C` and the window's close button still leave outright**, without
asking. A key that says *close this window* should close it; the question guards
the word, which is the one you can typo.

```bash
ORBS_BOOT=0 ORBS_DUMP="quit" cargo run -q -p orbs                       # it asks
ORBS_BOOT=0 ORBS_DUMP="quit" ORBS_THEN="quit" cargo run -q -p orbs      # ...and goes
ORBS_BOOT=0 ORBS_DUMP="quit" ORBS_THEN="status; quit" cargo run -q -p orbs  # asks again
scripts/tui.sh start && scripts/tui.sh type 'quit' 'quit'   # the session ends
```

### `menu` opens the orb's own screen

**A word of its own, and it used to be `quit`.** For one iteration `quit` opened
this and leaving was a choice made here, on the argument that the word means
*leave the thing you are in*. **Superseded** (§19): leaving the game and stepping
out to a screen are two things, and one word for both made stopping a two-step
operation through a screen the player had not asked for.

`Focus::Menu` takes the pane and is **first** in the ordering — the way out wins
any tie, which is the answer a player meant.

```bash
ORBS_BOOT=0 ORBS_DUMP="menu" cargo run -q -p orbs                     # the menu
ORBS_BOOT=0 ORBS_DUMP="menu" ORBS_MENU="resume" cargo run -q -p orbs  # ...and back
ORBS_BOOT=0 ORBS_DUMP="menu" ORBS_MENU="zorb" cargo run -q -p orbs    # §6: it says so
```

> **A dump could not have caught the defect this shipped with.** `type_into_menu`
> was gated on the menu being open, and a reader that does not run keeps its
> cursor — so the first time it ran it read whatever `Messages` still held, which
> on the frame the menu opened was the word that opened it. Typed back in, that
> word reached the menu's own `quit` and left the orb, which looked exactly like
> `quit` never having stopped ending the session.
>
> `ORBS_DUMP` builds no `App` and presses no key, so it drew a perfect menu
> throughout. **The gate for a surface that takes the keyboard is a test that
> fires a real `KeyboardInput` at the real plugin stack** —
> `opening_the_menu_does_not_eat_the_word_that_opened_it`, in `shell/plugin.rs`.
> The fix is the rule `focus.rs` already states: *"a surface that grabs the
> keyboard on open eats the player's first keystroke."*

**`ORBS_MENU` is the latch**, `\n`-separated, one word per segment with Enter
implied — `ORBS_WEAVE`'s shape minus the arrows, because the menu has nothing to
walk. `<esc>` leaves it. **A `quit` typed at the menu inside a dump prints the
menu**: a dump hosts no process, so there is nothing to end, and what it shows is
the screen the player was looking at when they left.

The menu's `quit` does **not** ask twice. The question guards the word typed at
the *prompt*, where it can be a typo for something else; a `quit` chosen from a
list headed *what now?* is already the second step.

### The menu's three pages, and more than one tower

`resume` · `saves` · `new` · `quit`, then `back` on either inner page. Every
choice is a word, prefix-matched like the editor's and the weave's — `r`, `s`,
`n`, `q`, `b`, and `s`/`m`/`l` for the lengths. **Escape steps one page and never
leaves the orb**: the one irreversible choice here is always typed.

**A dump keeps no save** (`ORBS_SAVE=off`, which `dumps.sh` pins), so `saves` and
`new` say so rather than drawing an empty list. Point it at a real path to see
them:

```bash
mkdir -p /tmp/t
ORBS_SAVE=/tmp/t/orbs-save.toml ORBS_BOOT=0 ORBS_DUMP="menu" ORBS_MENU="saves" \
  cargo run -q -p orbs     # the towers, numbered — the dump wrote slot 1 itself
ORBS_SAVE=/tmp/t/orbs-save.toml ORBS_BOOT=0 ORBS_DUMP="menu" ORBS_MENU="new" \
  cargo run -q -p orbs     # short / medium / long
```

**Slot 1 *is* `orbs-save.toml`**, so nothing a player already had has moved;
2–6 sit beside it as `orbs-save-2.toml` and so on, and `ORBS_SAVE=<file>` names
slot 1 wherever you point it.

**The swap itself a dump cannot show** — it builds no `App` and hosts no session,
so `Load` and `Begin` leave the menu on screen. It is tested where it can be
exercised:

```bash
cargo test -p orbs --bins menuing
```

**`loading_a_second_tower_writes_the_first_to_its_own_file` is the exit criterion
of the whole feature.** Autosave writes to whatever path it holds, so a swap that
replaced the path before writing would put the tower you *left* into the file of
the tower you *opened*. `Kept` makes the two one operation, and that test is what
says so.

`F2` (phosphor) and `F12` (screenshot) are **not** bound, and cannot be: there
are no themes to cycle and no pixels to capture. That is rule 2 working — the
terminal loses enrichment and no information.

**`--dump` is the boundary proof, and it is in CI.** Both binaries reach
`orbs_shell::dump_script` through the same painters from the same `Sim`, so:

```bash
ORBS_WIZARD=wizard ORBS_BOOT=0 sh -c '
  diff <(ORBS_DUMP="attend laboratory; grind sage" target/debug/orbs) \
       <(target/debug/orbs-tui --dump "attend laboratory; grind sage")'
```

...prints nothing, or the shell has grown a frontend-shaped hole in it. Every
`ORBS_*` switch that reaches the *shell* works against `orbs-tui` too, so a
See-it line written for one frontend runs against the other unchanged.

**Two do not, and they are the only two: `ORBS_SIGHT` and `ORBS_CRT`.** Both name
render passes, and a terminal has none — they are read in `crates/orbs` and are
silently inert in the other build. A See-it line using either does not move
across frontends, and this sentence exists because the paragraph above used to
say *every* without qualification.

**`scripts/dumps.sh <dir>` captures every surface the game can draw** — 98
screens — and is the instrument for a refactor whose gate is that nothing
changes. Run it before and after, then `diff -r`. It pins `ORBS_WIZARD`, because
a baseline that varies with who ran it is not a baseline.

**The bailey was absent from it until `0.8.15`**, and the way that surfaced is
the lesson: quintessence changed the board, the diff came back clean, and *clean*
was the bug. Phase 8 shipped a whole domain — the densest picture in the game —
without a capture, so nothing in this file could have told you when it moved.
**A domain built without a `dumps.sh` block is a domain this instrument is blind
to**, and the blindness looks exactly like stability.

**Count it, do not quote it.** This line said 56 for two versions and the number
was 58; a plan written against it said 52; it then said 65 while the answer was
73. `ls <dir> | wc -l` takes a second and the figure is only ever used to notice
a screen that stopped being captured.

### `scripts/play.sh` — the game, played, as a test suite

**131 scenarios that type at a real terminal and read the screen back** — and
count them with `scripts/play.sh 2>&1 | tail -1` rather than trusting this
number, which has been stale twice. This is
the third layer: `orbs-sim`'s tests prove the rules and `orbs-render`'s prove the
picture, and neither presses a key. Run it **after anything touching the sim, the
shell, or the loop** — `orbs-balance`'s standing, for the same reason.

```bash
scripts/play.sh                     # all of it, ~1 minute at six threads
scripts/play.sh routing::           # one area
scripts/play.sh -- --test-threads 1 # one at a time, to watch it play
```

**Not in the gate.** Every scenario is `#[ignore]`d, so `cargo test --workspace`
skips them; CI runs them in a **separate non-blocking job**. Without `tmux` they
print SKIPPED and pass, so read the output rather than the exit code somewhere
new.

**One test target, `tests/playing/main.rs`, and the layout is load-bearing.**
Cargo runs test *binaries* one after another and parallelises only *within* one,
so nine files would be slower than one. `tests/playing.rs` **does not compile** —
a crate root resolves `mod play;` against `tests/`, not `tests/playing/`; cargo
auto-discovers `tests/*/main.rs` instead and leaves the siblings as modules.

**Assertions are scoped to the newest command block, never to the screen**, and
both of the obvious alternatives were tried and measured:

- a plain substring search **matches history** — the clarity chain empties the
  mortar twice, six steps apart and both visible, so the second wait returned in
  **1 ms** against the first one's line and the driver went green on an
  assertion that was factually wrong;
- counting occurrences instead **deadlocks** — the pane holds exactly **8
  command blocks** at 120×45, so from the ninth repeat each new line pushes an
  old one off and the count never rises again.

Four more things the driver has to do, each of which failed loudly first:

- **`send-keys -l`, with Enter on its own call.** Without the literal flag tmux
  reads a word as a *key name* wherever one matches: `end` becomes the End key,
  `up` an arrow, `home` Home. `end` closes every `repeat` and `if` in the spell
  language. `scripts/tui.sh` had this bug for two versions.
- **Flatten before matching.** `RecordView` wraps rather than clips, so
  `research`'s answer lands as `…a way out` / `    is in them` and a needle
  written the way a person reads it matches neither row.
- **`opens()`, not `does()`, for `scribe`/`weave`/`wander`.** They take the whole
  pane, transcript included, so the answer to the word is a *screen* and a
  block-scoped wait waits for something that has been painted over.
- **`meditates(n)`, never `does("meditate 20", "meditate")`.** The prompt echoes
  `→ meditate 20` at once, so waiting for the word is satisfied before a single
  tick passes. It caught three scenarios, one of which asked for 7,200 ticks and
  got none.

**Seeds are chosen, not tolerated.** `drift` poisons a log at 1/300 per tick and
`substitution` renames a base reagent at 1/3600, both from the seeded `Threat`
stream — so the schedule is fixed in tick space. Measured: seed 3 poisons a log
inside 200 ticks and seed 0 swaps a reagent inside 7200; **11 and 42 are quiet
through both**, and `play::QUIET` is 11.

**Both frontends boot** (§19, `0.3.12`). `orbs-tui` skipped §4's sequence for a
version on an "instant-startup" argument that was really about the development
loop — which `ORBS_BOOT=0` already answered. The clock, the stages and the card
are `orbs-shell`'s; the terminal supplies a frame and the engine line, so the
card reads `crossterm 0.29` where the other says `bevy 0.19.0`.

**The world does not tick through it**, and that is a determinism rule rather
than a nicety: `tower::drift` rolls once per tick, so a sim running through nine
and a half seconds of animation would reach a different world on the same seed
depending on how fast the machine drew a logo. Nothing is typed during boot
either — except **leaving**, which a terminal needs because raw mode makes
`Ctrl-C` ours to answer or nobody's.

```bash
cargo run -p orbs-tui              # ...and watch it. 9.6s, then the tower
ORBS_BOOT=0 cargo run -p orbs-tui  # ...or don't
```

**Colour is indexed ANSI 0–15 and inherits the user's terminal theme** (§19), so
there is no palette to tune here and no contrast to verify — the numbers belong
to whoever configured the terminal. What holds §14 up instead is the other half
of the rule: the glyph carries the identity. Three degradations are accepted and
recorded — `Presentation` renders identically, a mixed `Wash` takes its first
tint, and there is no CRT.

**Ambiguous-width glyphs are a correctness item, not polish.** CP437's symbols —
`‼ ► ☼ ○ ♂ ♀ ♦ ♠ Ω ░ ▒` — are East Asian *Ambiguous*, so a CJK locale or a
terminal set to `ambiguous = wide` gives them two columns, which shifts the row
**and** invalidates the per-cell diff. `orbs-tui` measures rather than assumes:
it prints one at a known column and reads the cursor back, then swaps in ASCII if
the answer is two. `cargo test -p orbs-tui` holds the table.

### The passage — a screen leaving, and what it does not touch

**Play it. This one genuinely needs a window**, and the switches above are the
still photographs that go with it:

```bash
cargo run -p orbs
# attend forge      — the laboratory's instruments wipe out, the forge's wipe in
# attend laboratory — and back
# attend alembic    — and *nothing*, because the room did not change
# wander / <Esc>    — the maze takes the whole pane, transcript included
# F5 / F5           — the linear mirror, on the same terms
```

**Then press `F3` until the tube is `OFF` and do all of it again.** Every one of
them must **cut**. §14 makes motion disableable and DESIGN.md §19 records the
trap that goes with it — three animations independently learned that *"a picture
may never be able to vanish"*, because reduce-motion pins a phase and froze a
blank. With the tube off there is no crossing to freeze and no kept screen at
all, which is a thing to check rather than assume.

**Six things to look at, in the order they are easy to get wrong:**

1. **The transcript does not move** when you change rooms. That is the whole
   design — history did not change, and blanking it says the session went away.
   The boards and the instrument panel are what cross.
2. **The two regions go different ways.** The gauges and the road along the top
   leave **upward**; the instrument panel and the room's board leave
   **rightward**, out past the tower rail. Each by the edge it already sits
   against, so neither crosses the text between them.
3. **They come back the way they went.** Out to the right, in from the right —
   not out right and in from the left, which is what it did first and reads as
   two unrelated motions.
4. **The border, its title and the tower rail do not move either.** The title
   says `forge` while the strip below still shows the laboratory, for the length
   of a crossing; that is the intended order and is recorded in §19 rather than
   fixed. A box that came apart would read as the machine breaking.
5. **A tool gathers instead.** `wander`, `edit` and `weave` replace the whole
   pane, so the screen **flies into the middle**, winks out, and the next one
   flies back out of the same point. One convergence, not one per region — three
   piles converging on three centres is what the union did before `whole` was
   made to stand in for the parts.
6. **`attend` in a bound spell does not strobe the pane.** A crossing may not
   begin within a world tick of the last one, which is what caps whole-region
   flashes at one a second. Nothing you can type gets near it; a spell walking
   the tower is the case the floor exists for.

**The boot sequence is a crossing too**, and it is the first one anybody meets:

```bash
cargo run -p orbs        # and watch the opening, which is now three motions

ORBS_DUMP=1 ORBS_BOOT=post:0.04 cargo run -p orbs   # `O.` growing in
ORBS_DUMP=1 ORBS_BOOT=post:0.09 cargo run -p orbs   # ...`O.` whole, `R.` growing
ORBS_DUMP=1 ORBS_BOOT=post:0.19 cargo run -p orbs   # ...and `S.`, the last of them
ORBS_DUMP=1 ORBS_BOOT=post:0.31 cargo run -p orbs   # then what it stands for
ORBS_DUMP=1 ORBS_BOOT=close:0.4 cargo run -p orbs   # the card collapsing
ORBS_DUMP=1 ORBS_BOOT=close:1.0 cargo run -p orbs   # ...down to one cell
ORBS_BOOT=0 ORBS_PASSAGE_AT=wake:0.35 \
  ORBS_DUMP="attend laboratory" cargo run -p orbs   # the tower opening
```

1. **The name arrives a letter at a time** — `O.`, then `R.`, then `B.`, then
   `S.` — each one growing in from its own middle by `Gather`'s arriving half,
   the same motion `wander` uses. **Only the one in flight moves**: the letters
   behind it stand whole and the ones ahead are not drawn yet, which is the thing
   to check and the thing a single frame cannot show. Then the **subtitle**, on
   its own clock, after the letters rather than alongside them.
2. **`Stage::Close` takes the card away** the same way, and **the box stays**.
   That is the thing to check: the border the card drew for itself is the pane the
   game arrives in, so folding it away would mean drawing a second one over the
   hole a frame later.
3. **The tower opens** — the rail pushes in from the right and narrows the pane
   to make room, and the gauges and road push down from the top. `wake` is the
   only crossing that moves the rail, and a dump reaches it no other way: it is
   started by a system on the one frame the sequence hands over.

**And one that is a defect if you see it:** the new screen must never appear
*whole* for a frame before the motion starts. It did — `ShellSystems::Drive` had
no edge to `ShellSystems::Input`, so the crossing could be driven before the
system that opened the surface, and a `wander` drew the finished maze and then
transitioned away from it. §19 records it.

**The shape, as text, at eight fractions in a row:**

```bash
cargo run -p orbs-render --example screens    # the "A crossing" block
```

That is the only place the *motion* can be read rather than inferred — a dump
poses one frame, and one frame of a wipe says nothing about which way it went.
It is also what caught the first draft's demo glyphs: `◇` and `◆` are outside
CP437, so the arriving half printed `???` and nothing else in the project would
have shown it.

### The output style — a heading is ruled, a slot is `<bracketed>`

**Three rules, all in `record/view.rs`, and all of them the *view's*** — records
carry bare words so `sift reagent` still finds one (rule 4).

1. **A heading is ruled off beside the words**, not under them, so hierarchy
   costs no row. The rule stops at a fixed column — never more than half the
   pane — because drawn to the pane edge the rules become the strongest marks on
   screen and the content goes to mush.
2. **A slot draws `<like this>`.** Not decoration: tiled bare, `distil reagent`
   and `kindle reagent` run together because the gap *between* entries is the
   same two spaces as the gap *inside* one. `>` terminates the cell.
3. **Described where local, indexed where global.** A run whose records carry
   `FieldName::Detail` takes one row each with a second column; a run without one
   tiles. `Verb::anchor` decides, so a new domain describes its own words with no
   list to maintain.

4. **A section opens with a blank row**, which `Input` has always had and which
   `opens_with_a_gap` now answers for both. A rule names the boundary and the gap
   gives the eye somewhere to land; pressed against the previous listing, a ruled
   heading looks like part of it.

```bash
ORBS_BOOT=0 ORBS_SAVE=off ORBS_DUMP="attend laboratory; help" cargo run -p orbs
ORBS_BOOT=0 ORBS_SAVE=off ORBS_GRID=80x22 ORBS_DUMP="attend laboratory; help" cargo run -p orbs
```
```text
the work ─────────────────────────
  grind <reagent>   crush a reagent in the mortar     ← this room's own
  distil <reagent>  draw a potion off in the alembic
                                                      ← the gap, not the rule,
finding your way ─────────────────                       is what separates them
  attend <place>  survey <place>  peruse <file>       ← every room's, indexed
```

**`opens_with_a_gap` is asked in `height` *and* in `draw_lines`**, and it is one
`const fn` for that reason — a blank row changes the row count, which is the one
thing those two may never disagree about. DESIGN.md §19 records that this was
refused once on a measurement that had expired: the help page stopped fitting the
80×22 floor exactly when the listing gained descriptions, so the blank rows make
a scrolling page longer rather than breaking a fit. **Re-measure before honouring
an objection of that shape** — nothing but running it can tell a live one from a
stale one.

**A leader bridges to a *right-aligned* column and to nothing else.** The boot
card's `name ....... ok` works because `ok` is flush right. Against a
left-aligned column the run length is set by the *near* column's raggedness —
`status` took eleven dots and `digest <reagent>` none — so the eye lands on the
dots. `status` keeps leaders; listings take a plain gap.

```text
tick ...........   2      ← the one right-aligned column in the transcript,
seed ........... 181        so 181 and 2 end in the same place
```

**`RecordKind::Status` has five emit sites and only one is `status`.** The
cold-start report is a contiguous run of seven and `verify` is another — so the
reading column is guarded by a **shape test** (two or more records, each exactly
a name and a numeric quantity), never by the kind alone. Boot is mixed and fails
on its first row. Check `ORBS_BOOT=0 ORBS_DUMP="verify laboratory"` after
touching it: the report must be untouched.

**Prose wraps to a measure of 68, not to the pane** — 86 cells in the laboratory
is 40% past the comfortable line length. Scoped to `Message`: a log or a `.spell`
listing re-wrapped would be a line the game had reformatted, on the surfaces
whose whole job is fidelity. It never binds at the 80×22 floor, whose body is 62.

**`play::flatten` collapses leader runs as well as whitespace.** Seven scenarios
asserting `experience 0` broke the day `status` gained dots; all seven were about
the reading rather than its padding. A **run** of dots only — a single `.` is
kept, or `orbs-save.toml` stops matching.

**The measure and the draw must change together.** `Tiling::plan` measures
through `signature_of` because `draw_tiled` draws through it. Measuring the bare
form sets the stride two cells short and overlaps the tiles, which is §19's
`attend plasurvey plaperuse filsift` defect.

**The squint test, and the CRT shader is already the blur:**

```bash
timeout 60 env ORBS_BOOT=0 ORBS_SAVE=off ORBS_CAPTURE=1 ./target/debug/orbs
python3 -c "from PIL import Image, ImageFilter; \
  Image.open('orbs-screenshot.png').filter(ImageFilter.GaussianBlur(4)).save('/tmp/squint.png')"
```

If the rules are the strongest shapes, there are too many. **~50 of the 65
captured dumps move when a heading changes** — headings are in nearly every
screen with a listing — so byte-identity is a gate for a pure extraction only.

### Greyscale — §14's accommodation, and the switch nothing else can flip

**`ORBS_SIGHT=greyscale` takes every hue out of the picture**, and it is a
separate render pass that runs *after* the tube. Both halves are decisions.

**Last, not "before the barrel".** §19 originally placed the filter *"after the
phosphor and before the barrel"*, and there is no such place: `crt.wgsl` applies
the barrel **first** — it computes the sampling coordinate — and three later
terms put hue back into a pixel that had none (the aperture grille attenuates
R/G/B per column, the aberration is a coloured fringe, the flash is additive).
A pass upstream of those would be undone by them.

**A separate pass, because `crt.wgsl` early-returns when the tube is off.** A
filter below that guard would switch off with `F3` — §19's *"a switch inferred
from the absence of something is not a switch"*. `SightUniform::new` takes a
`Sight` and nothing else, so there is no path from the tube to the accommodation.

**`ORBS_CAPTURE` writes its screenshot at frame 30 and then keeps running** —
nothing sends `AppExit`. So every capture line needs a `timeout`, and one
without it hangs the shell rather than failing. This is the whole reason the
loop below reads the way it does; the first draft of it omitted the timeout and
blocked on its first iteration.

```bash
# The four-way matrix. Greyscale must be grey in **both** tube states.
for s in plain greyscale; do for t in default off; do
  timeout 60 env ORBS_BOOT=0 ORBS_SAVE=off ORBS_SIGHT=$s ORBS_CRT=$t \
    ORBS_CAPTURE=1 ./target/debug/orbs >/dev/null 2>&1
  python3 -c "
from PIL import Image
im=Image.open('orbs-screenshot.png').convert('RGB')
print('$s/$t', sum(1 for p in im.getdata() if p[0]!=p[1] or p[1]!=p[2]))"
done; done
```
```text
plain/default      3180776     ← the control. without it the check is vacuous
plain/off          4665600
greyscale/default        0
greyscale/off            0     ← the property the whole design turns on
```

**`ORBS_CAPTURE`, not `ORBS_DUMP`, and they cannot be combined** — `orbs_shell::dump`
returns before the `App` is built, so a dump has no render passes at all. This is
one of the few things only pixels can show.

**`ORBS_CRT` takes `default`, `peak` or `off`** and seeds what `F3` cycles. It
exists so the tube-off case is reachable without a keypress, which is the case
§14 most needs checked automatically.

**These two are the first `ORBS_*` switches that are Bevy-only.** Every other one
works against `orbs-tui` as well, and the boundary claim further up this file
says so — but a terminal has no render passes and no phosphor, so `ORBS_SIGHT`
and `ORBS_CRT` are silently inert there. A See-it line using either does not move
across frontends. Greyscale in the terminal build is the user's own colour
scheme, which is §19's standing answer for that frontend.

**The three correction filters are deliberately not built** (§19). Daltonisation
was measured against this palette and degrades the accent triad in **eleven of
twelve** theme × deficiency combinations, because the triad is solved in
*luminance* and daltonisation redistributes *hue*. `render::deficiency` keeps the
simulation half, `#[cfg(test)]`, and points it at
`the_accent_triad_survives_every_deficiency` — which found monochrome's danger
and cost sitting 1.06:1 apart under deuteranopia and is why that theme's `cost`
moved.

```bash
cargo test -p orbs --bins render::palette   # the floors, including the deficiency one
```

**Solve a palette constant in floats, not on a 0–255 grid.** The first fix for
monochrome was solved as integers, landed a rounding step away, and failed
`accents_are_distinguishable_from_body_text` at 1.191 against a floor of 1.2.
These margins are smaller than one step of 1/255.

