#!/usr/bin/env python3
"""Decode a `tmux capture-pane -e` into glyph-and-colour, cell by cell.

**The instrument a colour bug needs.** `capture-pane -p` throws the colour away
and `-e` hands back escape sequences no one can read down a column, so a fault in
`orbs-tui`'s `theme.rs` — a flame drawn in the wrong ramp, a tint that declined
when it should not have — is invisible to every tool the project already has.
`ORBS_DUMP` cannot see it either: it prints glyphs and a list of tinted regions,
and the terminal build resolves colour somewhere else entirely.

So this walks the SGR state machine and prints, per cell, what the terminal was
actually told to draw. Read-only: it takes a capture on stdin and prints.

    tmux capture-pane -t orbs -p -e | scripts/ink.py --cols 158,161
    scripts/tui.sh ink 158 161

Columns are 0-based and inclusive, counted in *cells* rather than bytes — a
capture is UTF-8 and the game is drawn out of box-drawing and block glyphs.
"""

import argparse
import re
import sys

# The sixteen indexed colours, as `theme.rs` names them. Anything outside 0-15
# is printed as its number: this frontend must only ever emit indexed colour
# (DESIGN.md §19), so a 256-colour or truecolor code appearing here is itself
# the finding.
ANSI = {
    0: "black", 1: "dark-red", 2: "dark-green", 3: "dark-yellow",
    4: "dark-blue", 5: "dark-magenta", 6: "dark-cyan", 7: "grey",
    8: "dark-grey", 9: "red", 10: "green", 11: "yellow",
    12: "blue", 13: "magenta", 14: "cyan", 15: "white",
}

SGR = re.compile(r"\x1b\[([0-9;]*)m")


def cells(line):
    """Every cell of one captured row, as `(glyph, colour, weight)`."""
    out = []
    colour, weight = "default", "plain"
    at = 0
    for match in SGR.finditer(line):
        for glyph in line[at:match.start()]:
            out.append((glyph, colour, weight))
        at = match.end()
        # An empty parameter list is `ESC[m`, which means reset.
        params = [int(p) for p in match.group(1).split(";") if p != ""] or [0]
        index = 0
        while index < len(params):
            code = params[index]
            if code == 0:
                colour, weight = "default", "plain"
            elif code == 1:
                weight = "bold"
            elif code == 2:
                weight = "dim"
            elif code == 22:
                weight = "plain"
            elif code == 39:
                colour = "default"
            elif code == 38 and params[index + 1 : index + 2] == [5]:
                value = params[index + 2]
                colour = ANSI.get(value, str(value))
                index += 2
            elif 30 <= code <= 37:
                colour = ANSI[code - 30]
            elif 90 <= code <= 97:
                colour = ANSI[code - 90 + 8]
            index += 1
    for glyph in line[at:]:
        out.append((glyph, colour, weight))
    return out


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cols",
        default="0,20",
        help="inclusive cell range as FIRST,LAST (0-based)",
    )
    parser.add_argument(
        "--rows", default=None, help="inclusive row range as FIRST,LAST (0-based)"
    )
    parser.add_argument(
        "--skip-blank",
        action="store_true",
        help="hide rows whose cells are all spaces",
    )
    args = parser.parse_args()

    first, last = (int(n) for n in args.cols.split(","))
    rows = range(0, 1 << 16)
    if args.rows:
        lo, hi = (int(n) for n in args.rows.split(","))
        rows = range(lo, hi + 1)

    for number, line in enumerate(sys.stdin.read().splitlines()):
        if number not in rows:
            continue
        row = cells(line)[first : last + 1]
        if not row:
            continue
        if args.skip_blank and all(glyph == " " for glyph, _, _ in row):
            continue
        drawn = " ".join(
            f"{glyph!r}:{colour}" + ("" if weight == "plain" else f"/{weight}")
            for glyph, colour, weight in row
        )
        print(f"{number:>3}  {drawn}")


if __name__ == "__main__":
    main()
