# Changelog

<!--
  The format is load-bearing and documented in docs/SETUP.md §4, not here.

  Both the release workflow and scripts/post_to_bluesky.py find a version's
  block by matching `## [` at the start of a line, so **this file must contain
  no example headers** — a fenced code sample showing the format was picked up
  as the newest release and posted to Bluesky verbatim. Keep the guidance in
  SETUP.md, where nothing scans it.

  Until 1.0, every `### Description` says the game is in development and reads
  as a dev log rather than as patch notes: it is published as a Bluesky post to
  people who cannot play this yet. That framing switches with the version scheme
  at 1.0 (DESIGN.md §19).
-->

## [v0.1.24] - 2026-08-14

### Description
First dev log for O.R.B.S., a text-only game about tending a wizard's tower from
its command line. In development. This stretch: the orb became a proper 4:3
monitor, the archive grew labyrinths, and the game learned to explain itself.

### Added
- **A manual you can read from inside the game** — `help` lists every word that
  works where you are standing, grouped by what it is for, and `recall <word>`
  opens a page on it: what it does, examples, the other ways to say it, and what
  to look at next. Every verb has one, including the ones that are not built yet,
  which say so rather than leaving you guessing.
- **The archive holds labyrinths** — `research` opens one on the lectern. Walk it
  with the arrow keys after `wander`, or one step at a time with `follow`, or
  teach the orb to solve it for you and watch the map fill in. Four fragments
  from four solved labyrinths make a spell scroll at the lectern.
- **The weave** — a progression screen showing what your work has earned: a ley
  line that runs straight, and a mastery tree that makes you choose between
  siblings at each tier.
- **The orb can hold a spell for you** — enough experience buys concentration,
  and a bound spell keeps running while you walk away. An invoked one stops when
  you leave the room; a bound one does not.
- **Every instrument has a picture rather than a reading** — the athanor's fire
  climbs, the mortar fills, the bath rolls as it digests, and materials carry
  their own colour through the whole pipeline, so a glance tells you what a
  number would have.

### Changed
- **The picture is a fixed 4:3, letterboxed in whatever window you drag** — the
  grid used to be recomputed on every resize, so panes, borders and wrapped
  sentences all moved as you dragged an edge. Nothing reflows now. The common
  display heights all land on whole scaling steps, so the text stays crisp.
- **The tube is the picture, not the window** — the curve, the vignette and the
  rounded bezel belong to the 4:3 area, and the bars either side of it are the
  dark room the monitor stands in rather than part of the glass.
- **The prompt is the size of everything else** — it was drawn at double size at
  every window, which also cost you half the line to type into.
- **Labyrinths differ from one another** — you start in a random corner, the way
  out leans toward the opposing one rather than always sitting in it, and the
  maze's own structure no longer grows outward from the cell you begin on. The
  walk used to be much the same journey however the walls fell.
- **Walking a labyrinth no longer fills the transcript** — a line per step
  restating what the map had just drawn buried your own typing. `peruse
  archive.log` still has every step.
- **The lectern leaves dust** — the archive's waste is old paper and stone rather
  than a granary's threshed husk.

### Fixed
- **`ground-sage` no longer quietly means `ground-salt`** — naming a reagent you
  do not have could resolve to a *different* reagent you do, and then act on it.
  A real name is never read as another real name; typos still resolve.
- **The archive's log was empty** — `peruse archive.log` returned nothing at all,
  however long you had been reading, because nothing filed its records under the
  archive.
- **The rounded corner on the tube had never worked** — at any setting. It was
  rounding a rectangle outside the visible picture.
