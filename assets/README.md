# Assets

**Every asset here carries a `PROVENANCE.md` recording where it came from, under
what licence, and what that licence obliges us to do.**

This is not bureaucracy. O.R.B.S. is a paid product on Steam, and the cost of
discovering an unclear licence at submission time is the release date. The rule
is cheap now and impossible to retrofit once the directory has a dozen files of
uncertain origin.

| Asset | Licence | Obligation on the shipped build |
|---|---|---|
| [`fonts/unscii/`](fonts/unscii/) | Public domain / CC0 | **None.** Attribution given anyway |
| [`fonts/spleen/`](fonts/spleen/) | BSD-2-Clause | Reproduce the copyright notice and disclaimer in the shipped documentation |

The project ships under **GPL-3.0-or-later** (see the root `LICENSE`). Both asset
licences are GPL-compatible — CC0 imposes no conditions at all, and the FSF lists
2-clause BSD as *"compatible with the GNU GPL"*. Spleen's notice requirement
survives the combination and still has to reach the shipped build.

## Which font does what

**`unscii/` is the shipping font** — three faces of one family, one per
`orbs_render::Presentation`, so the tonal and diagnostic registers get a visual
channel that cannot disturb layout.

**`spleen/` is kept deliberately**, for two jobs:

1. **Gap-filler.** All three unscii faces are missing `∙ ⌂ ⌐ ☼`; Spleen has them.
2. **Fallback.** unscii's `Eldritch` and `Tampered` faces are 8×8 row-doubled, so
   they carry half the vertical detail of a native 8×16. If the Phase 0
   worst-case legibility test (DESIGN.md §4) rejects that, Spleen is a complete,
   crisp, already-vetted codepage to fall back to. Both are consumed by the same
   atlas builder, so the swap is cheap.

> ⚠ **Never add `unscii-16-full`.** It merges GPL Unifont, unlike the CC0
> `unscii-16` we ship, and its name differs by one word.
>
> The project itself is GPL-3.0-or-later, so this is no longer a licence
> *conflict* — it is a **provenance** one. `PROVENANCE.md` records exactly which
> files ship, with checksums; taking a different file silently makes that record
> false, and Unifont carries its own notice and source obligations that nothing
> here tracks. `crates/orbs/src/render/glyphs.rs` fails the build if it appears.

## Third-party notices

The BSD-2-Clause obligation is satisfied by shipping the notice **with the
binary** — a `LICENSES.txt` in the Steam depot, an in-game credits screen, or
both. Since every screen in this game is terminal content (§13), the natural
home is a `grimoire licences` topic, which costs a prose file rather than a
UI screen.

**Not yet wired up.** Tracked as a Phase 5 ship task; the obligation only
attaches on distribution, and nothing is distributed yet.
