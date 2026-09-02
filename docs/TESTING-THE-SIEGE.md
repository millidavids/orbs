# Testing the siege — a morning's play

A walkthrough for the bailey (Phase 8), written to be followed top to bottom at a
real keyboard. Every command below has been run; nothing here is from memory.

**Build first**, so the game does not print cargo progress into the window:

```bash
cargo build -p orbs && cargo run -p orbs
```

`ORBS_BOOT=0` skips the thirteen-second boot sequence. Use it every time except
the once you want to see the boot.

---

## 0. The one-line version

You are a wizard whose tower is under siege. A king has lent you six troops and
you have three dice. The enemy tells you what it will do **before it does it**,
you **place your dice** behind the parts of the wall that need them, and you say
`hold` to let a round happen. Nothing moves until you do.

There are four parts of the wall and three dice, so something always goes
without. Choosing what is the game.

While you fight, the enemy rewrites your spells behind your back.

---

## 1. Arrive, and read the board

```
attend bailey
defend
```

You should see, beside the transcript:

```
┌ rampart ──────────────────────────────────┐
│they mean to onslaught                     │   ← what happens NEXT round
│                                           │
│line      --                               │   ← the four parts of the wall
│buckler   --                               │
│succour   --                               │
│sortie    --                               │
│                                           │
│enemy     ████████████████████  21/21   60%│   ← them, and their odds
│───────────────────────────────────────────│
│garrison  ████████████████████  18/18   50%│   ← you, and yours
│                                           │
│coffer    d6 d8 d20                        │   ← your three dice
│round 0, 7 still coming                    │
└───────────────────────────────────────────┘
```

**Five things to check.**

1. The top line names one of `advance`, `onslaught`, `volley`. That is the
   telegraph, and it is what tells you where the dice should go.
2. Four areas, three dice. **One always goes without.**
3. The two bars are *lengths*, not colours — they read the same in greyscale.
4. The `%` on each row is that side's chance to land a hit. **Shown before you
   commit**, which is the fairness rule.
5. `round 0`. Nothing has happened yet.

**Try it on a volley.** Restart with `ORBS_SEED=5` until the top line says
`volley` and look at the `line` row — it should read `moot`. A volley cannot be
answered, so anything behind the line is thrown away, and the board says so
*before* you put a die there.

**Now wait.** Sit for a minute and do nothing. The board must not move — `hold`
is the only thing in the domain that advances the world. If the round counter
ticks up on its own, that is a bug and it is the most important one in the room.

---

## 2. Place your dice — this is the game

```
pledge d20 to buckler
pledge d6 to succour
survey coffer
hold
```

Each pledge tells you its **range before it is rolled**:

```
d20 goes behind the buckler. 1 to 20
d6 goes behind the succour. 1 to 6
```

**That is the whole decision.** The d20 has a huge ceiling and might roll 1; the
d6 is a floor you can lean on. So the question is not *how much* but **where a
bad roll costs least** — a low succour merely heals less, a low buckler is a
round wasted.

After `hold`, each area says what it came to:

```
they onslaught. you lose 2 and take 3
the buckler put 14 on the wall
the succour rolled 4 and put back 4
```

**Four things worth trying deliberately:**

| Type this | What should happen |
|---|---|
| `pledge d20 buckler` twice | `d20 is already pledged to the buckler` |
| `pledge d20 line` on a volley | it is allowed, and warns `wasted on a volley` — and it **still costs**, because you were told before you typed it |
| `pledge buckler d20` | the two the wrong way round — `buckler is not one of your dice` |
| `pledge d10 buckler` | **the ordinary fuzzy matcher**: `d10` is not a die you hold, so it is read as a typo for `d20` and the echo says `→ pledge d20 buckler`. Worth knowing rather than reporting — only `d6`, `d8` and `d20` exist as words, and the other four dice belong to the arsenal's future |

**And check the dice come back.** After a `hold`, `survey coffer` should show all
three again. **The die is not what is scarce — the quintessence is.**

## 2a. Quintessence, which is the decision

Pledging costs. A siege opens with a pool — 24 on a well-kept tower — and it
**never regenerates**: what you are given at `defend` is what the whole fight
gets. A `d20` costs 5, a `d8` 2, a `d6` 1, so filling every row is 8 a round and
the pool buys **three unrestrained rounds of a fight that runs six to thirteen**.

**Declining to pledge costs nothing**, which is the whole point: leaving a row
dark is now a move rather than an omission.

```
attend bailey
defend
survey coffer
pledge d20 buckler
survey coffer
```

The coffer row on the board carries it — each die with its price, then what is
left:

```
coffer    d6 1  d8 2                     19
```

**What to try, and what I most want your judgement on:**

| Type this | What should happen |
|---|---|
| Three full rounds, then pledge again | `d20 would take 5, and you hold 0` |
| `attend sanctum`, `meditate 3600`, then fight | a worn tower opens with **12**, not 24 |
| Spend it all on round one | you fight rounds 4+ with no dice at all — is that interesting, or just flat? |

**The question I cannot answer from a sweep:** is 24 the right number? Measured,
the allocation *now matters* — two solver strategies that used to tie across
seventeen seeds now differ on two seeds in five, by nearly threefold. But whether
it **feels** like a decision or like a budget you run out of and then stop
playing is the thing only playing it will say.

## 2b. Take a turn without the dice

```
survey enemy
survey garrison
hold
```

`survey` costs nothing and takes no time. You should get `spears` (how many are
standing), `mettle` (how much fight they have), `foes`, and any of the words
`few`, `hurt`, `outnumbered`, `massed` that currently apply.

`hold` resolves one round and prints one sentence:

```
they onslaught. you lose 2 and take 2
```

Then look at what actually happened:

```
peruse bailey.log
```

Every die, every face:

```
enemy strikes - d20 gives 12 against 11, and it tells
garrison swings - d20 gives 8 against 11, and misses
```

**This is the postmortem.** If a potion did anything, you can see it here. The
seven dice are D&D's — `d4 d6 d8 d10 d12 d20 d100` — and the log always names
which one rolled.

---

## 3. Spend the arsenal

The arsenal starts empty, so stock it. This is a debug word and it earns nothing:

```
debug_spawn troop 3
debug_spawn warding 3
debug_spawn mending 2
debug_spawn quickening-scroll 2
```

Then:

```
deploy troop      ← bodies into the line
quaff warding     ← +3 to every roll you make next round
quaff mending     ← fight put back
wield quickening-scroll   ← scrolls keep `wield`, as everywhere else
```

**Three refusals worth triggering deliberately**, because each one is the game
telling you something rather than an error:

| Type this | It should say |
|---|---|
| `quaff troop` | `troop is not quaffed or deployed. deploy it` |
| `deploy sage` | `sage is worth nothing on a wall` |
| `quaff mending` at full strength | `the line is whole. keep the mending for when it is not` |
| `quaff warding` when they will `volley` | `they volley next, so you will not swing. keep the warding` |

The last two are the interesting ones: **the orb refuses to let you waste a
potion** and keeps it. Check the arsenal afterwards with `survey arsenal` — the
count should not have gone down.

---

## 4. Fight one to the end

Keep going until it stops:

```
hold
hold
hold      ...and so on
```

A win reads:

```
the enemy breaks and runs. the wall holds, and you earn 147
```

A loss reads:

```
the wall is carried. 44 of the way, and you keep 55
```

**Losing still pays.** Escrow is scaled by how far you got with a floor of 20%,
so an evening is never thrown away. Check `status` for the experience.

**Then try to start another straight away:**

```
defend
```

It should refuse — `the road is empty. nothing comes for 1183 yet`. That is the
twenty-minute cadence. Without it a siege is the best experience in the game by
thirty-three times, which is how it measured before the gate existed.

*(If you want another siege now rather than in twenty minutes, restart the game
or use a different seed.)*

---

## 5. The enemy attacks your automation

This is the premise of the whole game and it only happens during a siege.

Fight several rounds and watch for:

```
something got past the wall. a spell does not read as it was written
```

**It never tells you which spell.** Finding out is the puzzle:

```
verify
```

That is the expensive audit — it takes the tower's one work slot for ~20 ticks,
so you cannot brew while it runs. Wait for it (`meditate 40`) and it names names:

```
these are not what they say: bulwark.spell, holding.spell
```

Now go and look, then repair:

```
attend grimoire
peruse holding.spell        ← find the corrupted line; it ends in a `-`
purge holding.spell         ← puts your own words back
```

**The corruption is always an argument, never a keyword** — `haul here there-`,
not `end-`. A line the orb cannot read at all would fault loudly and give the
game away; this one parses, runs, and quietly does nothing.

There is a second, subtler surface. Sometimes you will see:

```
something got past the wall. a spell is not keeping its own time
```

That spell's **text is perfect**. `peruse` shows exactly what you wrote. It is
just running slower. Only `verify` finds it.

**Try the cheap check too**, and notice it costs you:

```
verify bailey.log       ← instant
verify bailey.log       ← "the orb is still reading the log of things. 19 ticks"
verify dispensary       ← still free: shelves are a different surface
```

One look tires the orb of that *kind* of thing. Choosing which surface to
inspect first is the mechanic.

---

## 6. Let a spell fight it

Four solvers ship, and they disagree with each other on purpose. All are on the
grimoire's shelf in a debug build, so no setup is needed.

```
attend bailey
debug_spawn troop 6
debug_spawn warding 6
invoke besieging
meditate 900
peruse bailey.log
status
```

Then run the same thing with each of:

| Spell | What it does differently |
|---|---|
| `besieging` | The plain ladder. Spends the arsenal, **places no dice** |
| `answering` | **Reads the telegraph** for its arsenal spending |
| `sparing` | Hoards. Spends only when the line is thin |
| `bulwark` | Uses `part` and `for each band` — looks at everything before acting, which costs nothing here and nothing else in the tower |
| `steadfast` | **Places dice by the telegraph.** The d20 goes behind the buckler on an onslaught and into succour on a volley, where a bad roll costs least |
| `warding_off` | **`for each area`** — walks the four generically and fills whatever is empty and not `moot`. The shape to copy when you do not know the board |
| `sparingly` | **The thrifty one, and the only spell that declines on purpose.** Walks `for each die`, checks it can afford one, and holds the big die back for the rounds where the enemy's `aim` is actually dangerous |

**The comparison worth making.** Run `besieging` and `steadfast` on
`ORBS_SEED=3`: `besieging` places no dice and **loses** (83 experience);
`steadfast` places them and wins (189).

**And the one quintessence changed.** `steadfast` and `warding_off` used to tie
on every seed tried — allocating *at all* mattered and *where* did not. With a
pool to spend they now differ on two seeds in five, and by nearly threefold where
they do:

```
seed 11  steadfast 83    warding_off 147
seed 42  steadfast 56    warding_off 168
```

So *where* is a real decision now. **Whether it is an interesting one is the
thing I want your read on** — a sweep can tell you the numbers moved apart and
cannot tell you whether choosing felt good.

**`sparingly` is the one to read before writing your own.** It is the worked
example for the arithmetic and uses three things nothing else does:

```
for each die
    if not the coffer has fewer quintessence than die
        if the enemy has more than 55 aim
            pledge die to buckler
```

`for each die` walks the three rather than naming them; `than die` reads what
*that die* costs; and `aim` is the odds, which until now were drawn on the board
and askable by nobody. **The guard is a double negative on purpose** — `not …
fewer … than` is the language's route to *at least as many*, and the affirmative
is wrong rather than clumsy, because a comparison against a place is strict and
would refuse the die you can exactly afford.

**What to compare:** `survey arsenal` afterwards. On seed 0 (`ORBS_SEED=0`),
`besieging` finishes with 1 warding left and `answering` with 5 — both win. That
gap is the telegraph paying for itself.

```bash
ORBS_SEED=0 ORBS_BOOT=0 cargo run -p orbs
```

**Read the spells themselves:**

```
peruse answering.spell
```

---

## 7. It runs beside everything else

The point of the domain is that it tests your automation, so your automation has
to be *running*.

```
attend bailey
defend
hold
attend laboratory
grind sage
```

The grind must work. A siege takes **no production slot** — if you see
`the tower is busy`, that is a bug.

---

## 8. Where the game explains itself

Everything above is in the game. Check these read well:

```
attend bailey
help                  ← the room, then its verbs
recall defend         ← and quaff, deploy, hold
recall volley         ← every reading has a page
recall few
recall outnumbered
recall apprentice     ← writing your first bailey spell
recall scripting      ← what a spell here can name
```

`recall scripting` should list `band` under *sets*, and `rampart`, `garrison`,
`enemy`, `satchel` under *nameable here*.

---

## What I would most like you to judge

1. **Does placing the dice feel like a decision?** This is the big one. Measured,
   *where* you put them barely changes the outcome at three dice — allocating at
   all is what matters. If it should be a real choice, the pool wants to be
   bigger or the four areas want to differ more sharply than they do.
2. **Is a hand-played siege worth doing twice?** If it is only scaffolding for
   the spell, the domain has failed its own exit criterion. Section 4 is the test.
3. **Is the telegraph legible?** Do you find yourself reading the top line and
   changing where the d20 goes?
4. **Is one round in three too much sabotage?** It is tuned at `ODDS = 3` and
   that is a guess, not a measurement.
5. **Is `sortie` ever worth it?** It deals half its roll and costs a third —
   favourable, but paid in the thing keeping you alive. I am least sure of this
   one.

---

## Known gaps, so you do not report them as bugs

- **Three Phase 8 items are deliberately unbuilt** and named in `docs/ROADMAP.md`:
  pane synergies (waits on Phase 10's multiplexing), procedural traits and the
  eldritch renderer, and the unattended-siege backlog with difficulty tiers
  (waits on §5.3's trace provoking sieges rather than `defend` doing it).
- **`defend` is how a siege starts.** In the finished game trace provokes one;
  the cadence stands in for that until then.
- **The rail has no bailey box.** §10 fixes the domain count at seven and the
  siege is not one of them, so it does not get a slot. The board and the log are
  where it speaks.
- **`F4` does nothing visible** anywhere in the game until Phase 9a returns the
  second pane.
- **`orbs-balance` flags `besieging <-- drifted` at some seeds, and that is not a
  bug.** Two hours fits about six sieges, so one badly-timed sabotage moves the
  rate: 0.0992 / 0.1254 / 0.1138 / 0.1167 across seeds 0, 3, 11 and 42, a mean of
  0.1138 against the pinned 0.114. The pin is read against the **mean of four
  worlds** by `cargo test -p orbs-balance`; the sweep's column shows one.

## If something goes wrong

```bash
cargo run -p orbs 2>&1 | grep -iE "error|panic|warn"
```

The gate is green as of `v0.8.18`: fmt, clippy, all tests, docs, a real `orbs`
build, 131 played scenarios, and every balance rate at its pinned value.
