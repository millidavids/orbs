# credits

## game development

O.R.B.S. was created by David Yurek, under Blackhearth Studios.

## built with

### Rust
O.R.B.S. is written in [Rust](https://www.rust-lang.org/), a language empowering
everyone to build reliable and efficient software.

- **License**: MIT OR Apache-2.0
- **Website**: https://www.rust-lang.org/

### Bevy
The desktop build is built on [Bevy](https://bevyengine.org/), a refreshingly
simple data-driven game engine built in Rust.

- **License**: MIT OR Apache-2.0
- **Website**: https://bevyengine.org/
- **Repository**: https://github.com/bevyengine/bevy

O.R.B.S. uses Bevy without `bevy_text` and without `bevy_ui`. Every screen in the
game - including the boot sequence, the editor, and the menus - is terminal
content drawn by the project's own cell-grid renderer.

## third-party libraries

Beyond Bevy and the Rust standard library, O.R.B.S. depends on:

| Crate | Purpose | License |
|---|---|---|
| `bevy_ecs` | The entity-component system the simulation is built on | MIT OR Apache-2.0 |
| `rand`, `rand_chacha` | Seeded, reproducible randomness | MIT OR Apache-2.0 |
| `serde`, `toml` | Reading content files and writing the save | MIT OR Apache-2.0 |
| `notify` | Watching content files for hot reload | CC0-1.0 |
| `thiserror` | Error types | MIT OR Apache-2.0 |
| `tracing` | Diagnostics | MIT |
| `clap` | The balance harness's command line | MIT OR Apache-2.0 |
| `crossterm` | The terminal build's input and output | MIT |

For complete license information covering every transitive dependency, see the
`Cargo.lock` file, which lists all dependencies and their versions.

## fonts

O.R.B.S. renders every glyph from bitmap fonts rasterised into a texture atlas.
Full provenance - upstream URLs, versions, retrieval dates, and SHA-256 checksums
for every file taken - is recorded in `assets/fonts/*/PROVENANCE.md`.

### Spleen

The primary typeface, used for all ordinary text.

- **Author**: Frederic Cambus
- **Version**: 2.2.0
- **Source**: https://github.com/fcambus/spleen
- **License**: **BSD-2-Clause** - full text in `assets/fonts/spleen/LICENSE`

> **Binary redistribution notice.** The BSD 2-Clause license requires that the
> copyright notice, conditions, and disclaimer be reproduced *"in the
> documentation and/or other materials provided with the distribution."* This
> file, shipped alongside the game, satisfies that condition. Shipping O.R.B.S.
> without the Spleen notice would not.

Only `cp437/spleen-8x16-ibm-437.bdf` was taken, plus its license file. Glyphs
may be modified in future releases; the BSD-2 license permits this.

### unscii

Used for the game's two alternate tonal registers, which change the shape of the
letters without changing their position.

- **Author**: Viznut (Ville-Matias Heikkilä)
- **Source**: https://github.com/viznut/unscii - https://viznut.fi/unscii/
- **License**: **Public domain (CC0)** - dedication quoted in full in
  `assets/fonts/unscii/LICENSE.md`

Three faces are used: `unscii-16`, `unscii-8-fantasy`, and `unscii-8-mcr`. All
three are assembled from unscii's own sources and contain no GNU Unifont
material; `unscii-16-full`, which does and is therefore GPL, is deliberately not
used.

## art, music and sound

**There is none, and that is the design.** O.R.B.S. is artless by intent: no
sprites, no characters, no illustrations. Everything on screen is a character
cell drawn from one of the bitmap fonts above.

The game currently ships without music or sound effects. If audio is added, it
will be credited here.

## special thanks

- The Bevy community for their documentation, examples, and willingness to
  answer questions about a version that changes every release
- The Rust community for the language and the ecosystem
- Frederic Cambus for Spleen, and for shipping a CP437-indexed variant
- Viznut for unscii, and for drawing a 16-pixel face rather than stretching one

---

O.R.B.S. is open source under the GNU General Public License v3.0 or later. See
[LICENSE](../LICENSE) for details.
