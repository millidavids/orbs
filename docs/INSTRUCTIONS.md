# how to play

**The manual is in the game now.** Launch it and type `manual` at the orb's menu
— before you have a tower, or from `menu` inside one. Nineteen chapters, and it
is the same text this file used to carry, kept where it cannot go stale against
the build.

```bash
cargo run -p orbs        # then: manual
```

It answers what this file answered and rather more: starting, continuing and
abandoning a tower; the prompt and how the orb reads you; the rooms; time; what
the rooms make and what it is for; the ley line; spells; sabotage; sieges; the
screen; the settings; the keys; what to do when something goes wrong; and the
licences. Three of its chapters are *built* from the 578 manual keys `recall`
already holds — every command in the game, every word a spell is written with,
and every room's own three lines — so they cannot drift from what the game
actually does.

Without a window, the same text as plain text:

```bash
ORBS_BOOT=0 ORBS_DUMP=1 ORBS_THRESHOLD=1 ORBS_MENU=manual cargo run -q -p orbs
ORBS_BOOT=0 ORBS_DUMP=1 ORBS_THRESHOLD=1 ORBS_MENU=manual \
  ORBS_MANUAL=orbs cargo run -q -p orbs      # ...and one chapter
```

And in the game, at any prompt, `help` lists every word that works **where you
are standing**, which is different in every room. That is the reference you will
actually use; the manual is the long way round.

---

## why this file is a pointer

It was 199 lines of orientation, and it was **wrong in two places** by the time
the manual shipped: it named `new` and `saves` on the orb's menu, which moved
under `play` at `0.16.1`, and it closed by saying settings do not persist, which
stopped being true at `0.16.5`.

That is the argument for the manual being content rather than a document.
`crates/orbs-sim/content/manual.toml` is hot-reloadable like the rest of the
prose (CLAUDE.md rule 6), it is held by the same drawability and width lints, and
a chapter that names a command the game does not have fails a test. A markdown
file beside the source is held by nobody.

Edit the manual there, not here:

```bash
ORBS_CONTENT=crates/orbs-sim/content cargo run -p orbs
# edit crates/orbs-sim/content/manual.toml while it runs
```

`docs/PLAYER_README.txt` is the short orientation that ships **beside** the
binary, for somebody who has not launched it yet. This file is a signpost for
anybody who finds the repository first.
