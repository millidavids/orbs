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

## [v0.3.13] - 2026-08-19

### Description
In development — a dev log, not patch notes. The orb learned to break another
wizard's seal, every room got its own line on a rail down the side of the
screen, and the whole game now runs in a plain terminal — boot sequence and all.

### Added
- **Another wizard's orb, sealed with four sigils of six.** Press it and it tells
  you how many are in the right socket and how many are merely present. Turn a
  dial, press again, and narrow it down. The orb keeps no list of what is still
  possible — working that out is the puzzle, and handing it to the machine would
  be handing away the game.
- **Three recipes nobody taught you.** A broken seal spills the far wizard's
  working log, and somewhere in it is a way to make something you had no recipe
  for. Until you find it the orb cannot make it, cannot name it, and has no page
  about it.
- **A rail down the side of the screen**, one box per room, so you can see the
  laboratory working while standing in the archive. It says what each room is
  doing, marks a room that has news, and marks one where something has gone
  wrong — and going to look is what clears it.
- **Spells can drive the lens.** A four-rung spell solves any seal, because a
  socket can be told to try the next sigil it has not tried yet without your
  having to name which one that is.
- **The whole game runs in a terminal.** Same tower, same words, same screens —
  no window, no graphics card. Every surface works: writing spells, walking the
  stacks with the arrow keys, the progression screen, reading back through the
  transcript. It opens the way the other one does, with the orb waking up.
- **A way to stop playing.** `quit` ends the session in both builds, and so do
  the keys you would expect. Nothing on screen used to say how to leave.

### Changed
- **Reading a seal costs nothing.** A press used to take twelve seconds of the
  one thing your laboratory could be doing instead. Now a solver can run beside
  a full brewing loop without either waiting on the other.
- **The instrument panel and the maze got out of each other's way** on small
  screens: where the pictures no longer fit, the one that yields does so cleanly
  instead of half-drawing.

### Fixed
- **A spell could end your session.** `quit` written into a spell closed the
  game, and a spell you had bound would do it again every time it looped.
- **The rail called things seconds that were not seconds.** The archive reported
  unwalked shelves and the lens reported sigils, both with a `t` on the end — so
  the lens counted *down* as you won, which read as a job about to finish.
- **The seal's working sheet vanished** on a small screen once you had pressed
  twelve times — exactly when a long solve most needed it. It now shows fewer
  rows rather than none.
- **A room running a spell could stop saying so** at some window heights, which
  is the one line telling you that room is automated.
- **Reading back through the transcript** now works from behind an open spell
  editor, steps one entry at a time with the arrow keys, and actually moves the
  moment you ask for it.
- **Letting go of an arrow key** on the way out of the stacks no longer types
  into the prompt.

## [v0.2.0] - 2026-08-16

### Description
In development — a dev log, not patch notes. Scrolls do something now, the tower
got an arsenal so finished work can leave the room it was made in, and spells
learned to loop until a question is answered instead of guessing a number.

### Added
- **Three scrolls, and each one does something.** Four scraps from the stacks
  make one. A gleaning scroll sets the shelves to gather — five things scattered
  in the dark and no way out. A quickening scroll makes the laboratory work at
  double speed for five minutes, whether or not anything is brewing yet. A
  verdant scroll makes the shelf remember a herb it has never held.
- **The arsenal** — one room reachable from every other, and the first place
  anything could be carried between rooms. It keeps finished work only: a potion
  brewed in the laboratory can be carried there and listed from the archive, and
  a handful of sage is turned away and told where it belongs.
- **Two new potions and three new herbs.** Amber, mugwort and valerian arrive on
  the shelf one verdant scroll at a time, and between them they reach insight
  and stillness.
- **Spells can loop until something is true.** `repeat until the stacks is idle`
  runs no times if they already are, and stops the moment they close — where
  before you had to guess a number of laps and hope.
- **Spells can count, and compare.** `if the cabinet has 4 fragment` waits until
  there are four. `2 or more`, `at least 2`, `1 or fewer`, `exactly 2` and the
  symbols for them all read, so you can write it however you think of it.
- **The manual covers the spell language.** Every word a spell is written with
  has a page, and so does every reading a maze publishes. `recall scripting`
  lists what you can write *in the room you are standing in* — the words and the
  questions are the same everywhere, what you can name is not.
- **Every material tells you what it is.** `recall` says what a thing is and how
  it is used before it says how to make it, because someone holding a potion is
  not asking for its five steps.

### Changed
- **The archive is three things instead of one.** The stacks are the shelves you
  walk, the cabinet is where scraps are kept, and the lectern is where four of
  them become a scroll. Each says what it is doing on its own row.
- **A way says how many times it has been walked**, as a number. It used to say
  only "walked" or "twice", so a square crossed nine times looked exactly like
  one crossed twice and a solver could not prefer the quieter path.
- **"Labyrinth" is gone; it is the stacks.** One name for one thing — you
  research at the stacks and you are then in the stacks.
- **A gleaning run pays five scraps against the four a scroll costs**, so
  gleaning is what keeps scrolls in circulation.

### Fixed
- **A number in a question was silently thrown away.** `if the cabinet has 4
  fragment` was read as "has any fragment", with nothing said about it.
- **A spell stopped when you walked into another room.** A loop's question was
  answered about wherever *you* were standing rather than where the spell was,
  so a spell left running while you did something else quietly gave up.
- **A mistyped loop ran for ever.** A bound the orb could not read produced an
  endless loop while telling you the loop had stopped.
- **Two tinctures had no colour**, so they and everything made from them drew in
  the wrong shade.

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
