# New surface

A surface is a screen that takes the pane and, usually, the keyboard: the spell
editor, the weave, the labyrinth map. It is the category with the most
frontend-only work and the one where **rule 2** bites — a frontend may add
enrichment the other cannot reproduce *provided it carries no information absent
from the Frame*.

Worked examples: the **weave** (`shell/loom.rs` paints, `shell/tapestry.rs`
holds state, `shell/weaving.rs` owns the keyboard), the **spell editor**
(`shell/sheet.rs`, `shell/editor.rs`), and the **labyrinth map**
(`shell/labyrinth.rs`, `shell/wandering.rs`, with the picture itself in
`orbs-render/src/maze.rs` because both frontends need it).

## Where the pieces go

| Concern | Crate | Why |
|---|---|---|
| The picture — what appears and where | `orbs-render` | Both frontends need it, and it must be testable with no GPU |
| The painter — this surface in this pane | `orbs/src/shell/<name>.rs` | Bevy-side layout against a `Rect` |
| The state — what is selected, where the cursor is | `orbs/src/shell/<name>.rs` or `orbs-sim` | **In the sim if a spell can drive it**, in the shell if it is purely a way of looking |
| Keyboard ownership | `orbs/src/shell/<mode>.rs` + `input.rs` | See below |
| Registration | `orbs/src/shell/plugin.rs` | Registration only. No bodies |

The maze is the case worth studying: the *picture* is in `orbs-render` because a
bound spell solves a labyrinth and the terminal frontend must draw it too, while
*who the arrow keys belong to* is purely frontend.

## Keyboard ownership is the hard part

There is no focus system. `input::type_into_line` **runs and declines** —
clearing the reader cursor — while every other surface is gated by `run_if`.
The guard in `input.rs` currently has four terms, and its comment says a fifth
requires a real `Focus` refactor. **If your surface is the fifth, do the
refactor rather than adding a term.**

Two rules learned the hard way (§19):

- **The prompt is the default.** A surface must be *entered* with a word — the
  editor's `edit`, the weave's `ley`/`mastery`, the map's `wander`. A surface
  that grabs the keyboard on open eats the player's first keystroke.
- **Escape must `break`, not `continue`.** A mode that consumes Escape and stays
  is a mode you cannot leave.

If the surface consumes arrows or a raw keystroke stream, give it an env switch
so a dump can drive it — `ORBS_EDIT`, `ORBS_WEAVE`, `ORBS_WALK` are the three
that exist, all newline-separated, and each documents its own segment grammar in
CLAUDE.md.

## Taking the pane

`panel::split` runs **first** so the instrument panel always wins, then the
surface takes what is left. Whichever split runs *second* is the one whose
refusal fires — §19 records getting that backwards.

- **Take columns, not rows**, where you have the choice. The labyrinth takes
  columns because taking rows under a `Top` panel left the transcript five rows.
- **Refuse rather than truncate.** Below a floor, draw a bordered "too small"
  message like the editor and the loom do, or draw nothing — never something
  misleading. `loom::MIN_ROWS`/`MIN_COLS` and `labyrinth::LEAST_BLOCK` are the
  precedents.
- **Or pan.** The map shows the part the reading stands in rather than vanishing,
  which is right at a small grid — but check it at the real one: a session pane
  is **60 columns**, not the grid's 120, because panes tile.

A surface that fills the whole pane and hides the transcript (`wander`, the
editor) is a deliberate mode, not a layout accident. Say which you are building.

## Do not re-derive geometry

The grid is a constant — `orbs_render::GRID`, 120×45 — and the window only
scales it (§19). So:

- **Layout is computed once and never moves.** A resize changes no rectangle. If
  your plan responds to a resize, it is solving a problem that no longer exists.
- **Transcribed geometry rots silently.** `labyrinth.rs`'s `BODIES` table holds
  pane dimensions traced from the layout, and it went stale without a single test
  failing — a wrong rectangle is still a rectangle. If you transcribe a
  measurement, add the assertion that re-derives it.

## Speech is not optional

Every frame carries a `Speech`, and §14 makes it architectural rather than a
nicety: a cell grid read back row by row is box-drawing characters and column
fragments, not sentences. A surface that draws a picture must say what the
picture means — the maze speaks its counts, the weave speaks its nodes.

Check with `F5` in the running game, which shows the session pane as a reader
hears it, and in the linear stream printed under every `ORBS_DUMP`.

## See it

```bash
# The surface, and its linear stream beneath.
ORBS_BOOT=0 ORBS_DUMP="<how you get there>" cargo run -p orbs

# ...driven, if it owns the keyboard.
ORBS_BOOT=0 ORBS_DUMP="..." ORBS_<SWITCH>="<key>\n<key>" cargo run -p orbs

# ...at the authoring floor, where a sentence stops fitting.
ORBS_BOOT=0 ORBS_GRID=80x22 ORBS_DUMP="..." cargo run -p orbs

# ...and in the Frame-only example, which has found bugs the suite did not.
cargo run -p orbs-render --example screens
```

Add a screen to `examples/screens.rs` whenever a new surface is built. It renders
real `Frame`s through the public API with no sim and no GPU, and CLAUDE.md
records it catching an em-dash CP437 cannot draw and a pane eating its own
border.
