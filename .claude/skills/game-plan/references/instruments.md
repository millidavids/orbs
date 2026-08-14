# New instrument, material, recipe, or domain

Four sizes of the same thing. A **material** is content only. A **recipe** joins
two materials through an instrument. An **instrument** is a node with a verb, a
state and a picture. A **domain** is a room full of instruments plus the verbs
that reach them.

Worked examples: the **mortar and pestle** for a transforming instrument, the
**athanor** for one that only supplies heat, the **lectern** for one that
assembles rather than transforms, and the **archive** for a whole domain — the
second one built, and the only one built after the conventions settled.

## Material — content only

`crates/orbs-sim/content/materials.toml`:

```toml
[ground-sage]
tint = "green"
```

The eight names in `orbs_render::Tint` are `green`, `brown`, `grey`, `gold`,
`violet`, `red`, `blue`, `bone`. **An unknown name fails the load** rather than
falling back, deliberately: an untinted material draws in the base hue too, so a
silent fallback would make a typo indistinguishable from an omission.

Pick by family, not by prettiness — spent plant matter is brown, what a fire
leaves is grey, finished work is gold. §19 records the lectern's leavings moving
from brown to grey when they stopped being chaff and became dust.

## Recipe — content only

`crates/orbs-sim/content/recipes.toml`, keyed by instrument:

```toml
[[mortar_and_pestle]]
input = "sage"
output = "ground-sage"
leaves = "husks"
ticks = 8
```

- **`count`** (default 1) is how many of `input` are consumed. `matching` asks
  for *at least* that many. The lectern's four fragments are the only use so far,
  and `recall` does not yet print the number — a known gap.
- **`leaves`** is the byproduct, and **§10.1's rule is that every byproduct has a
  use**. If nothing consumes it, add the recipe that grinds it down, or justify
  the litter. Ash and dust both grind to potash for exactly this reason.
- **`heat = true`** requires the athanor lit.

A recipe fires when the instrument holds what it wants and is `wield`ed — or via
the instrument's own verb, which is `move` + `wield` in one word (§19).

## Instrument

| File | What goes in it |
|---|---|
| `tower/build.rs` | raise the node in its domain |
| `tower/panel.rs` | its `State` (`idle`/`charged`/`gathering`/`working`/`fouled`/`ready`) and its column in the instrument panel |
| `content/recipes.toml` | what it does |
| `parser/verb.rs`, `vocabulary.rs`, `dispatch.rs` | its verb — see [verbs.md](verbs.md) |
| `tower/scene.rs` | `offering(verb)` so the word only exists where the tool is |
| `orbs-render/src/<picture>.rs` | its meter, if it wants one |
| `content/prose.toml` | its outcome lines and its manual page |

**Reserve the panel's name column.** `NAME` is 18 and `STATE` is 10 in
`shell/panel.rs`; both have been one too small before, and the symptom is a word
running into the bar rather than an error.

**A state with no meter draws nothing.** `Bar::Plain` is not `meterless()`, so a
new `State` must be routed to a real bar or it is invisible on the panel — §19
records `gathering` nearly shipping that way.

## The picture is the point

§10.1 gives each instrument a *picture* rather than a reading, so a glance says
what a number would. That is `Depiction` — a colour ramp selected by the sim and
resolved by the frontend — and it is the one channel that says nothing itself,
which is why `Style::depicted` guards it from painting over anything meaningful.

**Motion must survive greyscale.** The bath's roil is all colour, so its *level*
is the glyph and only the turbulence is the ramp. A meter whose meaning lives
only in hue fails §14 and fails a dump.

## Domain

A domain is a node under the tower root with children, plus the verbs that reach
them. `tower/build.rs` raises it; `tower/scene.rs` scopes its words; its log is
`<domain>.log` and works by `files::in_domain` matching `Source`, `Path` or
`Origin` against the domain **and everything standing in it**.

**Check the log actually fills.** The archive's was empty from the day it was
built until §19 caught it — every completion set none of those three fields, and
nothing noticed because the pane was a second surface showing the same records.

Domains are also where the vocabulary budget is defended: each coins the verbs
its own tools need, and `Scene::offering` means none of them costs the others a
possible misreading.

## Determinism

Anything rolled needs a **per-subsystem** `RngStream` (`rng.rs`). Adding a
variant means a new index — **never renumber**, because `derive_stream_seed`
folds the index in and renumbering silently remaps every stream, invalidating
every existing replay.

Build generated state **in one go**. A maze that grew as it was walked would
issue `NodeId`s at a rate depending on how the ticks were consumed, so a
live-watched run and a `meditate`-collapsed one would produce different worlds
from one seed.

## See it

```bash
# The instrument working, and its tinted regions under the linear stream.
ORBS_BOOT=0 ORBS_DUMP="attend <domain>; <verb> <material>; meditate 9" cargo run -p orbs

# Straight to a state worth testing, without forty ticks of setup.
ORBS_BOOT=0 ORBS_DUMP="attend <domain>; debug_spawn <material> 3; survey dispensary" cargo run -p orbs

# The recipe, as the game explains it.
ORBS_BOOT=0 ORBS_DUMP="recall <output>" cargo run -p orbs

# A generated world differs between seeds — one dump proves nothing.
ORBS_SEED=3 ORBS_BOOT=0 ORBS_DUMP="..." cargo run -p orbs
```

Animation is an **edge** and a dump runs no systems: the flare, the pour and the
creep each need their own switch (`ORBS_FLARE`, `ORBS_LOAD`, `ORBS_TICK`) or the
effect has no See-it line at all. A colour-only meter needs
`cargo run -p orbs-render --example screens`, which prints the ramp as letters.
