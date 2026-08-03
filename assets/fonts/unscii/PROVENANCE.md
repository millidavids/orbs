# unscii — one face per `Presentation`

Three faces from one family, so `orbs_render::Presentation` has a visual channel
that costs no layout risk.

| `Presentation` | Face | Native size | Shipped as |
|---|---|---|---|
| `Plain` | `unscii-16` | **8×16 by design** | `unscii-16.hex` |
| `Eldritch` | `unscii-8-fantasy` | 8×8 | `unscii-8-fantasy.hex`, row-doubled at atlas build |
| `Tampered` | `unscii-8-mcr` | 8×8 | `unscii-8-mcr.hex`, row-doubled at atlas build |

`Plain` carries the ~88k-word prose budget (§12), so it gets the face that was
actually *drawn* at 8×16 rather than a stretched 8×8 — full vertical resolution
where nearly all the reading happens. The two special registers appear in short
bursts and can afford the doubling.

## What was taken

| | |
|---|---|
| Upstream | <https://github.com/viznut/unscii> |
| Author | Viznut (Ville-Matias Heikkilä) |
| Retrieved | 2026-08-03, `master` |
| Website | <https://viznut.fi/unscii/> |

| File | Glyphs | SHA-256 |
|---|---|---|
| `unscii-16.hex` | 3240 | `2642c8b748fa81f24d76772d70c55faa720d98fadfec8133daf89136c5c8bfb1` |
| `unscii-8-fantasy.hex` | 3191 | `7fbda7ebec089ac10ba9b8888b8ca1cad97e0e84788cb7d2b4fbd5bff1d20cc4` |
| `unscii-8-mcr.hex` | 3191 | `b116a5cb19eeca6ba01f938d86deb8e33f8e7ea260b3cc707382df29b9f8744d` |

Shipped **verbatim as upstream `.hex`**, unstretched, so the checksums can be
re-verified against the source. Row doubling happens in our atlas builder.

```sh
for f in unscii-16 unscii-8-fantasy unscii-8-mcr; do
  curl -sL "https://raw.githubusercontent.com/viznut/unscii/master/fontfiles/$f.hex" \
    | shasum -a 256
done
```

## Licence

**Public domain (CC0)** for the files taken here. Verbatim from the upstream
`README.md`:

> Licensing: You can consider it Public Domain (or CC-0) except for the files
> derived from or containing parts of Roman Czyborra's Unifont project
> (`unifont.hex`, `hex2bdf.pl`, `unscii-16-full.*`) which fall under GPL.

### ⚠ Never take `unscii-16-full`

`unscii-16-full` is **GPL**, not public domain. It is one character away from
the file we do use, and taking it by accident would place a copyleft obligation
on the game. The upstream `Makefile` shows exactly where the boundary is:

```make
unscii-16.hex:      $(SRC)                                     # src/ only — CC0
	./assemble.pl

unscii-16-full.hex: unscii-16.hex unifont.hex fsex-adapted.hex # merges GPL Unifont
	./merge-otherfonts.pl
```

`unscii-16.hex` is assembled purely from the project's own `src/`. Unifont enters
only through `-full`. The three files here are all built from `$(SRC)` and carry
no Unifont content.

`hex2bdf.pl` is likewise GPL. **We do not use it** — the `.hex` format is a
codepoint and a hex bitmap per line, so our atlas builder parses it directly.
That is a deliberate choice, not a coincidence.

### GPL-3.0 compatibility

**Compatible, trivially.** CC0 and public-domain dedication impose no conditions
at all, so there is nothing to conflict with any licence. The FSF lists CC0 as
GPL-compatible.

Note the difference in *form* from the Spleen font next door: Spleen ships a
formal BSD-2 licence text, whereas unscii's dedication is a sentence in its
README. A README dedication from the sole author is normally accepted, but it is
weaker evidence than a licence file, which is why the statement is quoted in full
above and preserved in [`LICENSE.md`](LICENSE.md).

## Known characteristics

- **All three share metrics and repertoire.** Same 8-pixel advance, same glyph
  set. Swapping faces per `Presentation` can never move a cell.
- **All three are missing the same four glyphs** from our repertoire:
  `∙` `⌂` `⌐` `☼`. They need drawing, or borrowing from Spleen. Four glyphs
  × three faces, and the fallback is already in the repo.
- **Box drawing is *not* identical between `unscii-16` and the doubled 8×8
  faces.** Verticals (`│`) and solid blocks (`█ ▀ ▄ ▌ ▐`) match; horizontals
  (`─ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼`) and the dither shades (`░ ▒ ▓`) do not, because a rule
  drawn at 16 rows does not land where a doubled 8-row rule lands.

  Harmless in normal use — borders are always `Plain`. Where it shows is a
  `Tampered` line containing box drawing, and there it works *for* us: §8.1 lists
  "malformed record boundaries" as a sabotage tell, and this produces one for
  free.
- `fantasy` and `mcr` agree with **each other** on every box-drawing glyph, so
  the two special registers stay consistent.
