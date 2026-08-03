# unscii — licence

unscii ships no `LICENSE` or `COPYING` file; the dedication lives in the
project's `README.md`. It is reproduced here **verbatim** so the terms travel
with the files rather than living at a URL that may move.

Retrieved 2026-08-03 from <https://github.com/viznut/unscii/blob/master/README.md>.

---

> The .hex format is basically the same as in the Unifont project. Each line
> consists of codepoint:hexbitmap, and the length of the bitmap string
> indicates whether the glyph is 8x8, 8x16 or 16x16.
>
> Licensing: You can consider it Public Domain (or CC-0) except for the files
> derived from or containing parts of Roman Czyborra's Unifont project
> (unifont.hex, hex2bdf.pl, unscii-16-full.*) which fall under GPL. See
> https://savannah.gnu.org/projects/unifont/ for more.
>
> The program code in this directory:
> - Makefile: builds the font files and some other stuff
> - assemble.pl: compiles the files under src/ into .hex files.
> - bm2uns.c: converts arbitrary bitmaps into unscii art (needs SDL_image)
> - bm2uns-prebuild.pl: builds some tables for bm2uns.c
> - checkwidths.pl: checks if the glyph widths in a .hex file match terminal usage
> - doubleheight.pl: stretches the 8x8 glyphs in a .hex file into 8x16
> - hex2bdf.pl: converts a .hex file into X11 .bdf format (from Unifont)
> - makeconverters.pl: makes shell scripts that convert between Unicode and legacy Unscii
> - makevecfonts.ff: a Fontforge script to build ttf/otf/woff fonts from an .svg font
> - merge-otherfonts.pl: fills in the missing glyphs from unifont.hex and fsex-adapted.hex
> - vectorize.c: builds a .svg font file from a .hex file
>
> Other files in this directory:
> - fsex-adapted.hex: Fixedsys Excelsior, an older public domain font with some similarities to Unscii
> - unifont.hex: Unifont, the definitive Unicode bitmap font.

---

## What that means for the three files here

`unscii-16.hex`, `unscii-8-fantasy.hex`, and `unscii-8-mcr.hex` are **not** on
the GPL list. All three are built from the project's own `src/` (see the
`Makefile` excerpt in [`PROVENANCE.md`](PROVENANCE.md)) and contain no Unifont
material.

**Public domain / CC0 carries no conditions**, so there is no attribution
requirement and no conflict with any licence, GPL-3.0 included. Attribution is
given anyway in [`PROVENANCE.md`](PROVENANCE.md) — it costs nothing and the work
deserves it.
