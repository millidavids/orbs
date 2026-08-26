O.R.B.S.
========
Operational Relic Bewitching System
Blackhearth Studios


Getting Started
---------------
Run orbs.exe (Windows), orbs (Linux), or double-click O.R.B.S..app (macOS).

The game is played entirely by typing. There is no mouse input.

Type "help" at the prompt. The orb explains where you are standing and lists
every word it will answer to there. Type "help" again after moving to a new
room -- each room has words of its own.

Your first line is:

    attend laboratory

...and then "help" again.


Save Data
---------
Your progress is saved to a single file named "orbs-save.toml", written next to
the game's executable.

It is plain text. You can open it in any editor, read it, and back it up by
copying it somewhere safe.

  NOTE: Moving the save into the standard per-user application data folder for
  your operating system -- and Steam Cloud support -- arrives with the settings
  screen in a later update. Until then it lives beside the executable.

If the folder the game is installed in is read-only, the save will fail. If that
happens, see "Choosing where the save lives" below.


Choosing where the save lives
-----------------------------
Set the environment variable ORBS_SAVE before launching:

  ORBS_SAVE=/path/to/my-save.toml     save to that file instead
  ORBS_SAVE=off                       do not save at all this session

On Steam you can set this in the game's Launch Options.


Clearing Progress
-----------------
Quit the game, then delete "orbs-save.toml". The next launch starts a fresh
tower.

There is no in-game "clear progress" button yet, and no automatic backup is
kept. If you want to keep your old tower, copy the file somewhere safe before
deleting it.


Screenshots and Diagnostics
---------------------------
Two keys write files, both next to the executable:

  F12   saves a screenshot as "orbs-screenshot.png"
  F6    writes "orbs-parse.tsv" -- a trace of how the orb read your commands

F6 is silent when it succeeds; the file appearing is the confirmation.


Reporting a Bug
---------------
Please include:

  1. The version number. It is printed on the boot screen as "v0.0.0", and
     "status" shows it too.
  2. What you typed and what happened.
  3. "orbs-parse.tsv" (press F6) if the orb misunderstood a command.
  4. Your "orbs-save.toml" if the tower is in a state you cannot get out of.
     It is plain text -- read it first if you would rather not share it.

Send them to support@blackhearthgames.com


Comfort and Accessibility
-------------------------
  F3   turns the curved-CRT effect off entirely. If curved screens make you
       uncomfortable, press this first. The game plays identically without it.
  F2   cycles the phosphor colour scheme, including lower-contrast options.
  F5   reads the screen as a stream of sentences instead of a grid.

Nothing in this game flashes. No mechanic requires fast typing.

Settings are not yet saved between sessions -- you will need to press F3 again
each launch. Persisting them arrives with the settings screen.

See HEALTH_WARNING.md before playing.


License
-------
O.R.B.S. is free software under the GNU General Public License v3.0 or later.
See LICENSE.

The Spleen bitmap font (c) Frederic Cambus is used under the BSD 2-Clause
license; see CREDITS.md for the full notice and for everything else the game is
built from.
