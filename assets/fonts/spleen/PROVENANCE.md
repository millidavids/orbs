# Spleen 8×16 — CP437

The bitmap font the Bevy cell renderer rasterises from (DESIGN.md §4).

## What was taken

| | |
|---|---|
| Upstream | <https://github.com/fcambus/spleen> |
| Author | Frederic Cambus |
| Version | **2.2.0** |
| Retrieved | 2026-08-03 |
| Release archive | `https://github.com/fcambus/spleen/releases/download/2.2.0/spleen-2.2.0.tar.gz` |
| Archive SHA-256 | `ec42925c6b56d2138c862b2f97147c872e472f674bf03423417d827a08d69a89` |

Only one file was taken from the archive, plus its licence:

| File | Upstream path | SHA-256 |
|---|---|---|
| `spleen-8x16-ibm-437.bdf` | `cp437/spleen-8x16-ibm-437.bdf` | `61f76fc920f18508d203fd5f90b3a9f8b023568ad83ac58409033a17142d1f24` |
| `LICENSE` | `LICENSE` | `f33fe8679d5b2abecc4f1313ce6c6bfa58262964de5f7bca146596a7318047af` |

To re-fetch and verify:

```sh
curl -sL -o spleen.tar.gz \
  https://github.com/fcambus/spleen/releases/download/2.2.0/spleen-2.2.0.tar.gz
shasum -a 256 spleen.tar.gz   # must match the archive hash above
tar xzf spleen.tar.gz
shasum -a 256 spleen-2.2.0/cp437/spleen-8x16-ibm-437.bdf
```

## Licence

**BSD-2-Clause.** Verbatim text in [`LICENSE`](LICENSE).

### GPL-3.0 compatibility

**Compatible.** The FSF lists the 2-clause BSD licence ("FreeBSD License") as
*"a lax, permissive non-copyleft free software license, compatible with the GNU
GPL"* — <https://www.gnu.org/licenses/license-list.html>.

Only the original **4-clause** BSD licence is GPL-incompatible, because of the
advertising clause. Spleen's licence has no advertising clause; it is the
2-clause form, reproduced in full alongside this file so the claim can be checked
rather than trusted.

### What the licence obliges

BSD-2 attaches two conditions, and the second is the one that reaches a shipped
game:

1. **Source redistribution** must retain the copyright notice, conditions, and
   disclaimer. Satisfied by keeping `LICENSE` in this directory.
2. **Binary redistribution** must reproduce the same notice *"in the
   documentation and/or other materials provided with the distribution."*
   **The shipped Steam build must therefore carry the Spleen notice** — see
   [`../../README.md`](../../README.md).

Nothing here restricts modification, which matters: §4 wants custom glyph
variants inside the codepage as a home for §8.1's sabotage tells, so the font
*will* be edited. A licence with a rename requirement (OFL) or a share-alike
clause (CC BY-SA) would have made that a recurring negotiation.

The project ships under GPL-3.0-or-later. That does **not** discharge condition 2
— combining BSD-2 material into a GPL work keeps the BSD notice requirement
intact, so the notice still has to reach the shipped build.

## Why this file rather than the others

The archive ships two 8×16 BDFs. This one is **indexed by codepage byte, 0–255,
with no gaps** — verified by counting `ENCODING` records, all 256 present.

The Unicode-keyed `spleen-8x16.bdf` is *not* a substitute: checked against the
256-entry repertoire in `crates/orbs-render/src/cp437.rs`, it is missing
`U+221F ∟`, `U+25BA ►`, and `U+25C4 ◄`. (For comparison, unscii-16 was missing
four others: `∙ ⌂ ⌐ ☼`.)

Byte indexing is also the exact shape the renderer wants:
`orbs_render::cp437_index(char) -> Option<u8>` already returns the atlas index,
so there is no mapping layer between the Frame and the glyph.

## Known characteristics

- `FONTBOUNDINGBOX 8 16 0 -4` — matches `orbs_render::CELL_WIDTH` / `CELL_HEIGHT`.
- **Stem weight is 2px vertical, 1px horizontal.** §4 names 1px stems under
  barrel distortion and the RGB subpixel mask as the top moiré and legibility
  hazard, so the horizontal rules of every pane border are the thing to watch in
  the Phase 0 worst-case legibility test. If they break up, thickening the
  box-drawing horizontals to 2px is a local edit the licence permits outright.
