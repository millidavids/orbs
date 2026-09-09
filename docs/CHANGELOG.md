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

## [v0.13.19] - 2026-09-09

### Description
In development — a dev log, not patch notes. The orb has stopped quietly
ignoring words it cannot place. A command with a word it does not know now gets
a question back, instead of running something you did not ask for.

### Fixed
- **A word the orb cannot place no longer just disappears.** `verify gibberish`
  used to throw the odd word away and audit the entire tower — twenty-one ticks
  of it. `digest husks` filled the water bath with nothing. In both cases the orb
  knew the verb, could not place what followed it, and went ahead anyway. Now it
  names the command and asks what you meant.
- **A mistyped line will not undo your last one.** When the orb works out what
  you meant and its best reading cannot be done where you are standing, it falls
  through to the next one — and a command that takes no argument at all can
  always be done, so `undo` kept winning. It now only considers readings that
  keep the thing you actually named.

### Changed
- **`look around` is a phrase the orb knows properly.** It used to work by
  accident, as `look` with a word left over; it is written down now, so it keeps
  working for the right reason.

## [v0.13.17] - 2026-09-08

### Description
In development — a dev log, not patch notes. The orb learned to read plain
English: "triturate the sage" works like "grind sage", from a model trained here
on this game's own words. It knows when you are thinking aloud, and the menu can
switch it off.

### Added
- **The orb reads what you meant, not just what it knows.** Type
  `turn the sage into powder`, or `smash the sage`, or `the sage wants
  crushing` — all of them reach the mortar. The orb still echoes back the short
  command it settled on, so the arcane words are learned by seeing them rather
  than by memorising a list.
- **It knows when you are not asking for anything.** `what should i do next` and
  `i wonder where the sage came from` are left alone, even though the second one
  names a reagent. Nine times in ten it declines the sentences that ask for
  nothing, and answers the ones that do.
- **A settings page, and the first thing on it is how the orb reads you.**
  `menu`, then `options`. **Augury** works out what you meant; **plain** answers
  only the words it already knows and suggests when it cannot. The choice takes
  effect on the next line you type and is remembered between sessions — kept with
  your machine rather than inside a tower, so opening another save never changes
  how your keyboard behaves.
- **The terminal build is the whole game now.** It runs the same reader the
  windowed one does. It had been going without, for no better reason than that
  the model used to drag a graphics stack behind it.

### Changed
- **Every command the game has can now be spoken plainly.** Seventeen of the
  forty-six had never been written down in the phrasebook — the whole menagerie,
  bailey and forge among them — so a sentence meaning one of those was quietly
  answered by whichever other command came closest. `send the wolves to the gate`
  used to move a gate.
- **The orb's guess is checked against the room before it acts.** It offers its
  best few readings and the tower takes the first that means something where you
  are standing, so a word that could be two commands is settled by what is
  actually in front of you rather than by the guess alone.
- **The phrasebook doubled**, and reaches for the older register too — `bray`,
  `triturate`, `lixiviate`, `repair to the archive`, `belay that`.

### Fixed
- **A misread no longer costs you a turn silently.** When the orb cannot make
  sense of a line at all it falls back to the way it always worked, suggestions
  and all, instead of running its best guess anyway.

## [v0.12.7] - 2026-09-07

### Description
In development — a dev log, not patch notes. A game has a length now: short,
medium or long, chosen when you begin one. The orb grew a menu that keeps six
towers. Sieges move renown both ways, and the arsenal is worth what you keep
making.

### Added
- **A game has a length, and you choose it when you start one.** Short, medium
  or long. The old climb topped out at ten thousand experience, which was
  reachable by typing — the long haul is twenty-five times that. The early
  stations barely move, so the first hour is the hour it always was; what
  stretches is the far end, where you are meant to be teaching the orb to work
  for you instead of typing every command yourself.
- **The orb has a menu of its own.** Type `menu`. The towers you keep are listed
  there with their wizard, their length and how far each has got; open another,
  raise a new one, or put the orb down.
- **The orb keeps more than one tower.** Six of them. Loading a second leaves the
  first exactly as you left it, written out before the new one arrives.
- **A siege moves your renown both ways.** What you deal and what you sortie
  raise it; what you lose and what you spend lower it, less whatever you mended.
  The fight speaks once, on settling, so a round does not narrate itself twice.
- **Renown decides what comes up the road, and you can pay to turn it down.**
  A famous tower draws longer odds; `petition` spends standing to shorten them.
  The floor never moves, so a famous tower can still draw a quiet night and an
  unknown one never meets the worst.
- **A spell can ask what has gone thin.** `for each store` walks the arsenal and
  `if the store has thin` answers, so a thinning arsenal can be met by a spell
  that keeps making things rather than by guesswork.

### Changed
- **The arsenal is worth what your industry is worth.** How much help a thing
  gives is matched to the rate you make it at: full strength while it is fresh,
  half once it goes thin, and refused in voice once it is spent. This replaces
  the old cap on how much of one thing the arsenal would hold — a ceiling you
  hit and stopped at, where this is a reason to keep production going.
- **`quit` asks before it leaves.** It is the one word that cannot be undone, so
  it puts the question and another `quit` answers it. Anything else you type
  says no. `F10`, `Ctrl-C` and the window's close button still leave outright.
- **Six-figure thresholds are shortened on the weave.** At the longest length the
  line runs to 250,000, and two numbers that size ran together into one nobody
  wrote. The exact figure is still there in the details panel and in what a
  screen reader hears.
- **A room's line says the number rather than spelling it out.** It sits beside a
  live count, so "five potions brewed" next to "3 of 8" disagreed with itself the
  moment a game was longer than the shortest one.

## [v0.11.10] - 2026-09-06

### Description
In development — a dev log, not patch notes. Screens move now. Walking between
rooms slides the old instruments away and the new ones back in, tools fly into
the middle and out again, and the orb's own opening does the same.

### Added
- **Screens move instead of cutting.** Walking into another room sends its
  instruments and boards out by the edge they already sit against — the readings
  along the top go upward, the room's own panel goes off to the side — and the
  new room's come back the way they went. What you have typed never moves: it
  did not change, so it stays where it is.
- **Tools open by flying into the middle.** The archive's map, the spell editor
  and the progression screen take the whole panel, so the whole panel gathers
  into a point and the new one grows back out of it.
- **The orb opens itself.** The name spells out a letter at a time, each one
  growing into place, then what it stands for, then what the orb is made of —
  and when the card is done it collapses inward and the tower pushes in around
  it, the rail from the side and the readings from the top.

### Changed
- **The opening is a third shorter.** Three things happening in turn read faster
  than one long one, so the card no longer needs the pauses that were holding it
  open.
- **`F3` stops all of it.** The tube's own switch turns the motion off with
  everything else it turns off, and screens change instantly instead.

## [v0.11.4] - 2026-09-04

### Description
In development — a dev log, not patch notes. The tower has a second number:
renown, earned when you make something and lost when a siege goes badly. Two
bars at the top of the pane warm from red through green as they fill.

### Added
- **Renown, and ten things to be called.** Every potion, scroll, fragment and
  troop you make earns standing as well as experience — the wizard sells what he
  makes, and that is why he makes so much of it. Ten titles run from hedge-wizard
  to remembered, and unlike experience this number can fall: lose enough and the
  tower is called something lesser again.
- **Two progress bars at the top of every pane.** One for the Ley Line and one
  for renown, each measuring the gap between the tier behind you and the tier
  ahead rather than the whole distance to the end — so they fill visibly instead
  of sitting still for an hour. They give up their rows to the transcript when a
  window is short.
- **The bars warm as they fill.** Red through yellow to green, in six steps, so a
  glance tells you roughly how far along you are before you have read a number.
  Green means arrived, not nearly — a bar one step short stays yellow-green.

### Changed
- **The grimoire is no longer a room you tend.** It had a box on the rail that
  read `idle` for ever, because nothing happens there and nothing can. The box
  and its progress line are gone; the spells, the writing and the shelf are all
  exactly where they were.

### Fixed
- **The rail drew an empty box, and vanished at window sizes where it fits.**
  Removing the grimoire left the tower rail dividing itself into seven slots and
  filling six, so a five-row hole opened between the last room and the readings.
  The same miscount made the rail disappear entirely at several window heights.
- **The archive and the menagerie earned no renown for what they stock.** Both
  put finished work on a shelf — fragments from the stacks, troops from a chant —
  and neither was paid for it, while the laboratory was paid for the same act.
  Troops are what a siege spends.
- **The bars could be mistaken for errors and confirmations.** In the terminal
  build a nearly-empty bar drew in exactly the red used for failures and a full
  one in the green used for success. The whole ramp moved off those colours on
  both builds.
- **A narrow pane stopped telling screen readers where the tower stands.** When a
  window was too tight to draw a bar, the row went silent instead of still saying
  its reading aloud, and left a stray label behind.
- **A long title crowded out the bar it belonged to.** The tower's name is now
  the first thing dropped when a row runs out of room, which is what it was
  always meant to be.
- **A finished track claimed it was one step from a tier that does not exist.**
  The Ley Line reads `nothing more authored` when there is nothing left to
  reach, and now says the same out loud instead of reading out a meaningless
  count.

## [v0.10.6] - 2026-09-02

### Description
In development — a dev log, not patch notes. The tower opens one room at a time
now: a fresh game is a laboratory and nothing else, and brewing your first
clarity is what opens the archive. Progression has two tracks again, and they
swapped natures.

### Added
- **The tower is earned.** A new game starts in the laboratory with six rooms
  dark on the rail. Brew a clarity and the archive opens; walk the stacks and the
  lens does; a warding potion opens the sanctum; the forge comes with the pool it
  spends. Rooms, recipes and charms all arrive this way, and each arrival is
  said.
- **Mastery is a line per room.** Seven straight lines with no choices on them —
  brew five potions, walk the stacks, close a figure — reached in order, and each
  station opens something in that room or the next. The room's own line runs
  under its title, the rail box says how close the next station is, and `recall`
  says it in words.
- **The Ley Line is where the choices are.** Sixteen stations from 16 to ten
  thousand, some of them forks of three lanes: more resources, better combat, or
  a faster orb. Charcoal that burns longer, a deeper pool, an edge on every roll
  at the wall, a siege that pays more, a troop worth more bodies, sabotage that
  comes rarer, charms that cost less, and spells whose work lands sooner — one
  choice per fork, and the line runs to the soft ending.
- **`weave` draws both** — the forks three deep on the Ley Line, and the seven
  rooms' lines — one track at a time below the bar, which is measured against the
  line's end now rather than a round hundred.

### Changed
- **Warding, insight, haste and stillness are earned, not known.** The
  laboratory's line hands them over as potions are brewed, with haste — the
  fastest thing in the game — last but one. Two of the lectern's three scrolls
  are the archive's to earn.
- **A charm is the forge's to earn too**, except the first; the others come
  with the lines of the rooms they bless.
- **A siege waits for a wall.** The road stays empty until the sanctum has
  mustered a course.
- **`status` says the pool and its ceiling**, since nothing else did.
- **The phases moved up one** above nine, for the third time and the same
  reason: progression is Phase 10 now, and the version on the card follows it.

### Fixed
- **The weave's second Ley Line step had no sentence** and printed its own key.
- **The example screens drew a tree the game no longer has.**

## [v0.9.3] - 2026-09-02

### Description
In development — a dev log, not patch notes. The seventh and last room opened: a
forge, where solving a lattice of glyphs binds a charm onto a tool. Charms make
things faster or richer, and cost the same power a siege spends.

### Added
- **The forge, and it is the tower's last room.** Seven were planned and seven
  now stand. It is where you enchant a tool — name the tool and the charm, and a
  lattice of glyphs opens.
- **A puzzle to bind one.** Every glyph must be lit, and touching one flips its
  neighbours as well. Three columns, eight ways to open, and exactly one of them
  works — the bottom row tells you which, if you can read it.
- **Five charms, and each wears off.** One halves how long a tool takes. One
  makes it yield twice. One makes a walk of the stacks pay two fragments. One
  rolls a bigger die on the wall. The last turns a saboteur's hand aside.
- **A charm can be laid on anything in the tower**, not only on what is in the
  room with you — and a spell can keep one alive while you are elsewhere, because
  the orb will say when a charm is nearly out.

### Changed
- **Power belongs to the tower now, not to a siege.** It used to be handed out
  when the enemy arrived and vanish when they left. It is yours, it comes back on
  its own, and the forge and the wall both draw on it — so a charm bound in the
  quiet is dice you cannot roll later.
- **Enchanting during a siege costs double**, which is the wizard's own fault for
  being at his forge while the wall is under attack.
- **A siege hands you nothing at the gate.** You bring what you have. Seeing a
  round out earns a little back, so waiting about earns nothing and the only way
  to more is to fight.
- **Mending the walls is what buys enchanting.** How much power the tower can
  hold rises with the barrier, so a neglected tower is a smaller one.
- **Binding a charm takes the tower's attention.** Nothing brews while a lattice
  falls, which is what makes keeping a charm alive a real choice against making
  things.

### Fixed
- **Nine potions said nothing would ever drink them.** Six have been spent on a
  wall since the siege arrived, and a troop deployed for as long. They say what
  they do now, and the two nothing spends admit it.
- **A quickening scroll left a mark on every save you took afterwards**, for the
  rest of the session and every session after it.
- **A hurried laboratory now scours quickly too.** It was the one job in the room
  that ignored the scroll.

## [v0.8.18] - 2026-08-31

### Description
In development — a dev log, not patch notes. The premise's last clause is built:
sieges. A sixth room where the enemy attacks the automation you wrote, fought on
dice and paid for from a pool that runs out. Spells learned to do sums.

### Added
- **Sieges, and the room you fight them in.** An enemy comes up the road and
  tells you what it means to do next; you spend what you have and end your turn
  to let a round resolve. There is no clock in it — nothing moves while you read
  the board — and it takes nothing away from the rest of the tower, so a brewing
  spell keeps working while you fight.
- **The dice, and where you put them.** Three of them, four parts of the wall, so
  one part is always dark and choosing which is the whole turn. A die is rolled
  when the round comes rather than when you place it, so the board shows you what
  it could come to before you commit — and shows the roll afterwards.
- **Quintessence, which is what a die costs.** A pool granted when the enemy
  arrives, and it never comes back during a fight. A big die costs five times
  what a small one does, so the question stopped being *which part of the wall*
  and became *is this part worth five*. Declining costs nothing, which is what
  makes leaving a gap a decision.
- **The arsenal is finally spent.** Every potion, scroll and troop the other five
  rooms have been making goes onto a wall. Ten lines of the game used to
  apologise for having nowhere to spend them; they are gone.
- **The enemy attacks your spells, which was always the point.** Mid-siege it
  rewrites a line of a script, or drags a bound spell's clock so it falls behind
  without a word of it being wrong. Nothing is destroyed and nothing is stolen —
  you are lied to, and finding the lie is the game.
- **`verify`, in two forms.** Bare, it audits the whole tower and costs you the
  time a brew would take. Named, it checks one thing and then tires the orb of
  that *kind* of thing for a while — so *which surface do I look at first* is a
  real question rather than four free glances.
- **A lost siege still pays.** What you earn scales with how far you got and
  never falls below a floor. Effort is never wasted.
- **Spells can do arithmetic now.** A question could always compare a reading
  against a number, or the same reading in two places. It can now weigh two
  different things — what you hold against what a die costs — and say *double* or
  *plus two*. Words, never symbols, and there are no brackets to learn.
- **The odds are something a spell can ask about.** The chance each side has to
  land a blow was printed on the board and readable by nobody; a script can read
  it now, so it can decide a wall is not worth defending.

### Changed
- **Keeping the tower repaired now matters to a siege.** A worn tower opens one
  with half the quintessence a kept one does. There is a floor, so neglect is
  expensive and never leaves you unable to win.

## [v0.5.12] - 2026-08-29

### Description
In development — a dev log, not patch notes. A fifth room: the menagerie, where
you sing a figure against the clock for troops. Spells learned to hand each
other names and to run in two places at once, and there is a guide to writing
one.

### Added
- **A fifth room — the menagerie.** Summon a figure and twelve syllables travel
  up toward a rule; sing each one as it lands. It is the only room in the tower
  with a clock in it, and the only picture that moves. Four misses and the figure
  comes apart, which costs the barrier — that risk is the whole price of the
  room, because trying costs nothing.
- **Troops**, which is what a chant is for. A figure sung cleanly calls them up
  and they keep themselves in the arsenal. Nothing spends them yet; a siege will.
- **Play it on the arrow keys.** `chorus` hands them to a running figure, and
  unlike the maze it does not take the screen — the figure draws beside the
  transcript, so you still read what the orb says about what you sang.
- **`F9` — a patient chant.** The syllable waits for you instead of for the
  clock. It reaches exactly the same ceiling as playing it in time, which is what
  makes it an accommodation rather than an easier setting.
- **The first thing you can actually buy.** The tree behind `weave` was drawn and
  refused for four phases; `steps_1` now grants what it says, and the orb thinks
  a little faster for the rest of the game.
- **A satchel in every room** — a queue you can look at, and the first thing two
  spells have ever been able to share. One puts names in, another takes them out
  in the order they went.
- **`alongside` — one spell in two places at once.** A part set running on its
  own cursor while the rest of the spell carries on, with its own place in the
  file and its own names.
- **`recall apprentice` — how to make a spell, from nothing.** The seven steps in
  the order they happen, with the example lines taken from whichever room you ask
  in. `help` now points at it, because nothing on that page said the orb could be
  taught to do any of it for you.

### Changed
- **The sidebar says when more than one thing is running in a room.** It named
  one spell however many were there. `status` gained a list of what is casting,
  and where.
- **A spell has to work out its own timing.** `bide until` read the delay
  straight off the world, which left nothing to think about; it is gone, and so
  is the reading behind it. The menagerie's solver now keeps its own time — and
  still cannot keep up until you have bought that second instruction a second.
- **Three worked spells shipped for the two new shapes** — two spells sharing a
  satchel, and one spell running in two places. The second solves a course of
  wards in as few moves as the single-cursor version does.
- **Every command in a listing shows its shape once.** `let <name> be <place>`
  used to draw with a second pair of brackets round the whole thing.

### Fixed
- **The menagerie's readings had no manual pages**, so asking about the word the
  room is built on answered with the room's overview instead.
- **A spell interrupted mid-delay resumed by starting the delay over.** Saving
  four seconds into an hour-long wait gave you another whole hour.
- **A collapsing chant wore the barrier without saying so.** The number on the
  sidebar went down while the room itself kept reporting the tower as whole.

## [v0.4.3] - 2026-08-27

### Description
In development — a dev log, not patch notes. This one is all readability: every
listing the game prints got ruled headings, described commands and room to
breathe, and there is now a greyscale mode that takes all the colour out.

### Added
- **A greyscale mode.** It removes every colour from the picture — including
  from the curved tube, which puts hue back into pixels later in the pass than
  you would expect. Nothing is only ever a colour in this game, so nothing is
  lost by switching it off. `F8` cycles it.
- **The window opens at 1920×1080.** It was 1280×720, which occupied a third of
  a modern display. The text is drawn at one and a half times its old size; the
  4:3 picture and its bars are unchanged.

### Changed
- **Every listing in the game was re-set.** Headings are ruled off beside their
  words instead of wearing `[square brackets]`, a command that takes something
  shows it as `grind <reagent>`, and each section now opens on a blank row. It
  used to read like a configuration file.
- **`help` describes the room you are standing in.** The words a room owns —
  `grind`, `digest`, `distil` — get a line each saying what they do, because
  they are the ones you have not met. The words every room shares stay a compact
  index. Asking about any single one still gives you its full page.
- **`status` lines its numbers up.** Dotted leaders, and the values in one
  column, so `2` and `181` end in the same place.
- **Prose wraps at a comfortable measure** rather than running the full width of
  the pane, which in the laboratory was about 40% too long a line. Logs and
  spells are exempt: those are shown to you exactly as they are.
- **The website wears the same clothes.** Its headings are ruled to a fixed
  column like the game's, and its sections open on the same gap.

### Fixed
- **One theme's accent colours were too close under colour blindness.** The
  monochrome theme's "cost" and "danger" marks sat almost on top of each other
  for a deuteranopic reader. Solved and pinned, along with the other three
  themes, against all three kinds of colour blindness.

## [v0.4.2] - 2026-08-26

### Description
In development — a dev log, not patch notes. Spells can now hand a named run of
lines what to work on, which took the sanctum's solver from nine lines a lap to
three. Plus a review of that room, and the documents a player needs.

### Added
- **A part takes arguments.** `part between(here, there)`, called as
  `between(wellspring, conduit)`. Before this a part could only read names the
  rest of the spell had set, so telling one what to do meant three `let` lines
  above every call — and every part in a spell was reaching into the same pot of
  names. The sanctum's solver was nine lines of that per lap; it is three calls
  now.
- **What a part can see is what is in its brackets.** Names it was not handed are
  not visible to it, and names it sets are its own — so a loop inside a part can
  no longer quietly overwrite the one the caller was walking. Reading a part is
  a local act: its brackets are the list.
- **The documents a player actually needs.** How to play, health and safety, a
  privacy policy, and full credits for everything the game is built from. The
  health notice is worth reading if flashing imagery affects you: nothing in
  this game flashes, and the one effect that could cause discomfort — the curved
  CRT — turns off with a single key.

### Fixed
- **Looking at something empty now answers.** `survey` on a place with nothing
  in it said *nothing at all* — the orb meeting a typed command with silence,
  which it does nowhere else. It had been that way for four phases without being
  noticed, because the rooms that existed are never empty; the sanctum made it
  the common case, since two of its three stations are bare for most of a solve
  and looking at one is close to the first thing you type in there.
- **A spell warding the tower no longer waits for the laboratory.** It was
  queuing behind whatever was brewing and eventually giving up — which is the
  opposite of the point, since the whole reason to bind one is so the barrier
  holds while you do something else.
- **The rail now always tells you how the barrier stands.** It used to switch to
  counting wards left to haul while a spell was working, so the number fell as
  the spell *won* — and it fell out of sight exactly when you were in another
  room and wanted it most.
- **A neglected tower no longer always draws the same course.** At its very worst
  it drew seven wards every time, which made the one thing the puzzle asks you to
  read a foregone conclusion.
- **`stop pylon` works.** The refusal told you to type it and it did nothing.
- **A pylon being cleared no longer reports itself idle** to the sidebar, the
  panel and your spells while it is plainly busy.
- **A save that has lost a ward no longer jams the room.** It came back as a
  course that could never be finished and could never be replaced; now it simply
  is not there, and you can draw a fresh one.

## [v0.4.1] - 2026-08-25

### Description
In development — a dev log, not patch notes. The tower can defend itself now: a
fourth room, a puzzle with no clock in it, and the first number in the game that
goes the wrong way on its own.

### Added
- **The sanctum, where the wizard wards his tower.** Raw arcane energy wells up
  in a wellspring; you draw it through a conduit one ward at a time and assemble
  it into a barrier. A greater ward will not rest upon a lesser. That is the
  whole of the rule, and everything about the puzzle follows from it.
- **`muster` draws a course of wards up; `haul` carries one between stations.**
  Both answer on the tick you type them and neither occupies the tower, so you
  can work the barrier while something is brewing downstairs.
- **Integrity — the first thing the tower can lose.** Every other number here
  only climbs. The barrier fades on its own, a little every half-minute, whether
  or not you are playing; finishing a course puts it back. The rail says how it
  stands from any room.
- **A tower you have neglected is more work to put right.** A barrier at its
  worst musters seven wards where a whole one musters three, which is sixteen
  times the hauling — expensive, and never impossible.
- **A spell can hold the barrier for you.** `holding` reads how tall the course
  is, works out which way round to rotate it, and does it — and bound, it draws a
  fresh course every time it finishes. It is the first spell that needs both a
  named part and remembered names to say what it means.

### Changed
- The lens's board is titled `seal` rather than `ward`. The word belongs to the
  defences, and the lens already called the thing a seal everywhere else.

## [v0.3.35] - 2026-08-24

### Description
In development — a dev log, not patch notes. The tower saves and loads now, the
spell language grew variables, conditional loops and reusable parts, and the
editor grew a guide that explains the word you are on and colours what you type.

### Added
- **The tower is saved, and closing the game no longer loses it.** It writes
  itself out as readable text every minute or so, and picks up where you left
  off. A save from an older build is refused rather than half-read — a tower
  that opens and is quietly wrong is worse than one that says it cannot.
- **A guide beside the spell editor.** By default it lists the language's own
  words and the verbs the room you are writing for answers to. Rest the cursor
  on a word and it becomes that word's page; move past it and the guide starts
  telling you what may come next instead — the places a question can ask about,
  then the answers that question accepts.
- **Tab finishes a word while you write a spell**, the way it always has at the
  prompt: it takes you as far as every candidate agrees, lists them when it
  cannot, then walks the list.
- **Spells can remember a name, and walk a set.** `let best be north` gives a
  name to a place, and `for each way` walks the four ways in turn without your
  naming them. Together they let a spell pick the least-walked way out of
  whichever ones are open, which no fixed ladder of questions could do.
- **A run of lines can be given a name and reused.** Write `part gathering()`
  above a few lines and `gathering()` runs them, as many times as you like.
- **Loops can stop when a question says so** — `repeat until the stacks is
  idle` — instead of guessing a number big enough to outlast the work.
- **A ladder of questions without the nesting.** `else if` chains, and one `end`
  closes the whole thing.
- **Spells can count.** `if the cabinet has 4 fragment` waits for four, and two
  places can be compared against each other — `if north has fewer marks than
  east` — in about seventeen ways of saying it, all written back in plain words.

### Changed
- **A spell is coloured now, not just weighted.** Its parts are drawn in
  different colours as well as different weights: the language's own words, the
  verbs, the numbers, and — the one that matters most — the small words a
  question turns on, which used to look exactly like the filler the orb throws
  away. The monochrome theme leaves it all in one colour on purpose, and the
  words still read correctly with every colour taken away.
- **The far wizard's seal lets a sigil repeat**, which takes it from 360
  combinations to 1296 and makes it the puzzle it was shaped like. Turning a
  dial now moves one socket and only that socket, so what changes afterwards is
  attributable — and three props that existed to paper over the old ambiguity
  are gone, along with the counting the orb was doing on your behalf.
- **Every verb reaches the plain-English words that claim it.** The lens had no
  plain way in at all: `spy`, `peek` and `try` were registered as one phrase
  nobody could type, so all three reached nothing.

### Fixed
- **Backspace did nothing on a great many terminals** — anything configured the
  way PuTTY ships. In a game played entirely by typing, a typo was
  uncorrectable short of clearing the line.
- **Holding a key down moved exactly one square**, and deleted exactly one
  character, on terminals that report a held key at all.
- **Escape followed quickly by a letter lost the Escape.** Closing the spell
  editor and typing `quit` fast enough put `quit` in the buffer as a line of the
  spell.
- **The cursor sat on the word telling you how to start.** In the editor's
  command row it rested on the `i` of `edit`, the first word on the row.
- **The orb called a good line unreadable.** Naming a part — the exact form the
  manual teaches — was reported as a line it could not read, while running
  perfectly well.
- **The lens's own scripting page listed the archive's words**, naming none of
  the six the lens actually answers with.
- **The boot sequence ran off the edge of a small window** for its whole
  thirteen seconds, instead of saying the window was too small.
- **Quitting could leave the terminal unusable** if the game was killed rather
  than asked to stop.

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
