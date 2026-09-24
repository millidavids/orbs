O.R.B.S.
========
Operational Relic Bewitching System
Blackhearth Studios


Getting Started
---------------
Run orbs.exe (Windows), orbs (Linux), or double-click O.R.B.S..app (macOS).

The game is played entirely by typing. There is no mouse input.

The orb opens on its own menu, which is typed like everything else. Type
"play", then "new", pick how long a game you want, and you are in. Next time,
"play" lists the towers the orb is keeping and a number opens one.

"manual" on that same menu is the whole book: how to play, how every system
works, and how scripting works. "settings" is where the sound, the glass and
the accessibility options live, and it remembers what you choose.

Then type "help" at the prompt. The orb explains where you are standing and
lists every word it will answer to there. Type "help" again after moving to a
new room -- each room has words of its own.

Your first line is:

    attend laboratory

...and then "help" again.

Type "menu" at any time to get back to the orb's menu.


Save Data
---------
Your towers are saved in your operating system's standard application data
folder, in a folder named "orbs":

  Linux     ~/.local/share/orbs/
  macOS     ~/Library/Application Support/orbs/
  Windows   %APPDATA%\orbs\

The orb keeps up to six towers, named "orbs-save.toml" through
"orbs-save-6.toml", with your settings in "orbs-settings.toml" beside them.

They are plain text. You can open them in any editor, read them, and back them
up by copying them somewhere safe.

  NOTE: If you played an earlier build, your towers were saved next to the
  game's executable. The first time this build runs it COPIES them into the
  folder above and leaves the originals alone, so nothing is lost.

  Steam Cloud support arrives in a later update.


Choosing where the save lives
-----------------------------
Set the environment variable ORBS_SAVE before launching:

  ORBS_SAVE=/path/to/my-save.toml     save to that file instead
  ORBS_SAVE=off                       do not save at all this session

On Steam you can set this in the game's Launch Options.


Clearing Progress
-----------------
From the orb's menu, type "play", then "abandon" and the number of the tower --
"abandon 2". It asks first; typing the same words again does it. The tower is
renamed rather than deleted, so you can put it back by hand if you change your
mind.

To clear everything, quit the game and delete the "orbs" folder named above.

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

The orb makes sound: a click under each key, a chime when something finishes,
and a low hum under it all. Every one of those says something the screen says at
the same moment, so turning them off loses nothing. "settings", then "sound":
"voice" is the cues and "hum" is the tube, and they move separately.

All of these are remembered between sessions. Every function key above is also a
row under "settings", and they are the same setting seen twice -- press F3 or
set "crt off", whichever you prefer, and it is still off next launch.

See HEALTH_WARNING.md before playing.


License
-------
O.R.B.S. is free software under the GNU General Public License v3.0 or later.
See LICENSE.

The Spleen bitmap font (c) Frederic Cambus is used under the BSD 2-Clause
license; see CREDITS.md for the full notice and for everything else the game is
built from.
