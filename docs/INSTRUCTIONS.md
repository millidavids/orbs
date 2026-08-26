# How to Play

O.R.B.S. teaches itself from inside. **Type `help` at any prompt** and the orb
explains the room you are standing in and lists every word it will answer to
there. This document is the orientation you read once; `help` is the reference
you use forever.

## Objective

You are a wizard. You have found a computer terminal inside your scrying orb,
and through it you can reach every part of your tower.

Your job is to keep the tower running: brew potions, assemble scrolls, break the
seals a rival has placed on your instruments, and hold the barrier that protects
you. All of it is work you can do by hand — and all of it is work the orb can be
taught to do for you.

**Teaching the orb is the game.** Doing a task by hand once is how you learn it;
writing a spell that does it while you are elsewhere is how you progress.

## Controls

**You play by typing.** There is no mouse input.

| Key | Does |
|---|---|
| **Enter** | Run the line you have typed |
| **Tab** | Complete the word you are typing, or list the choices |
| **↑ / ↓** | Walk back and forward through what you have typed before |
| **PgUp / PgDn** | Scroll the transcript |
| **F2** | Cycle the phosphor colour scheme |
| **F3** | Turn the CRT effect on and off |
| **F4** | Switch focus mode |
| **F5** | Read the screen as a linear stream of sentences |
| **F6** | Write `orbs-parse.tsv`, a trace of how your commands were read |
| **F7** | Cycle the tonal register |
| **F10** | Leave |
| **F12** | Save a screenshot |

Typing `quit` also leaves, and is the discoverable version of `F10`.

**Nothing rewards typing quickly.** The world advances once per second, and a
command you type is queued to the next tick. Racing the clock earns you nothing
anywhere in the game.

## Talking to the Orb

The orb reads plain English, within reason. It strips filler, so all of these
are the same command:

```
grind the sage
grind sage
```

If you misspell a word it will usually still find what you meant, and it echoes
back what it understood on the line beginning `→`. **Read the echo.** It is how
you learn the vocabulary, and it is how you catch the orb doing something other
than what you intended.

If a word is ambiguous, the orb asks rather than guessing.

## Getting Around

Your tower is a filesystem, and the rooms are directories.

- **`attend <room>`** — go and stand somewhere. `attend laboratory`
- **`survey`** — look at where you are. `survey <place>` looks at something
  without walking to it
- **`status`** — everything in flight at once
- **`recall <thing>`** — the manual page for anything: a verb, a potion, a word
  of the spell language. `recall clarity` walks you through a whole brew

**Each room has words of its own**, and they only work there. `grind` is the
laboratory's; `haul` is the sanctum's. `help` in a room lists that room's words
and nothing else.

## The Rooms

| Room | What happens there |
|---|---|
| **laboratory** | Brewing. Reagents through instruments, one stage at a time, each feeding the next |
| **archive** | Research. Walk the stacks for fragments, assemble them into scrolls |
| **lens** | Scrying. Break the seals a rival wizard has placed, and read what spills out |
| **sanctum** | Defence. Assemble a course of wards to hold the barrier, which decays whether or not you are playing |
| **grimoire** | Where your spells are kept |
| **arsenal** | Where finished work goes. The one room reachable from every other |

## Your First Ten Minutes

```
attend laboratory        stand in the room where brewing happens
help                     what this room is, and every word it takes
kindle charcoal          light the athanor. some stages need heat
grind sage               put sage in the mortar and work it
status                   see how long that will take
meditate 9               let time pass until it is done
empty mortar_and_pestle  clear the husks so the mortar can take another load
recall clarity           the whole recipe, start to finish
```

`recall clarity` is the single most useful thing to type early. It names every
stage of the game's flagship potion, in order, with how long each takes.

## Waiting

Work takes time — a grind is eight seconds, a distillation is a minute. You can
sit and watch, or type **`meditate <n>`** to let `n` ticks pass at once.

`meditate` is a convenience for playing alone. It is not a resource and there is
no cost to it.

## Writing Spells

This is what the game is for.

```
scribe morning           open the editor on a new spell called morning
```

Inside the editor, type **`edit`** to start writing, **Escape** to stop, and
**`quit`** to save and close. Two other words are worth knowing:

- **`guide`** — toggles a pane down the right that explains whatever your cursor
  is touching, and lists what may legally come next
- **`interpret`** — shows how the orb reads each of your lines *before* you run
  them. The place to catch a line that says something other than you meant

A spell is written for the room you scribed it in, so it needs no `attend`.

Then, back at the prompt:

- **`invoke <name>`** — run it once. It stops if you leave the room
- **`bind <name>`** — leave it running. It keeps working while you are elsewhere,
  and re-casts itself when it reaches the end

`bind` has to be earned. Do enough work by hand and the orb learns to hold a
spell for you.

### The language

Nine words: `wait`, `repeat`, `if`, `else`, `end`, `until`, `let`, `for`,
`part`. Type **`recall scripting`** for the whole language on one page, including
which questions the room you are standing in can answer.

```
part load(what)
    grind what
    empty mortar_and_pestle
end

load(sage)
load(rock-salt)
```

## Things That Will Go Wrong

- **An instrument refuses a second load** until you `empty` it. The byproduct of
  the last run is still in it.
- **A stage needs heat.** `kindle charcoal` first.
- **A reagent is not what it says it is.** A rival tampers with your stock.
  `verify <place>` checks a room; `purge <name>` fixes what it finds.
- **A log has been poisoned.** Same rival, different surface. `sift` and
  `verify` are how you tell.
- **A spell has faulted.** The tower rail down the right marks the room with
  `‼`. Go and look — going to look is what clears it.

## Reading the Screen

The **tower rail** down the right shows every room at a glance: whether it is
working, what it is working on, whether a spell is bound there, and whether
anything has gone wrong. It is how you know the laboratory needs you while you
are standing in the archive.

A room with a picture — the archive's maze, the lens's seal, the sanctum's board
— draws it beside the transcript whenever there is something to draw.

## Accessibility

- **`F5`** reads the whole screen as a linear stream of sentences rather than a
  grid. This is the route for screen readers.
- **`F3`** turns off the CRT curve entirely; **`F2`** cycles phosphor schemes,
  including lower-contrast and monochrome options.
- **No meaning is carried by colour alone.** Every colour in the game is
  duplicated by a glyph or a word, so the game reads correctly in greyscale.
- **No mechanic requires fast typing.** See *Controls*, above.

Settings do not yet persist between sessions; that arrives with the settings
screen.
