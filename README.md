# O.R.B.S.

**Operational Relic Bewitching System** — Blackhearth Studios

A wizard stares into his scrying orb and finds a computer terminal inside it.

O.R.B.S. is a text-only game played entirely through a fantasy command line. You
tend a wizard's tower by navigating a filesystem that *is* your duties — brewing
in `/laboratory`, warding in `/sanctum`, spying in `/lens` — and you progress by
writing scripts that teach the orb to do your work without you. When you are
ready, you descend into a siege, where an intelligent enemy attacks the automation
you built and you must diagnose and repair it under pressure.

Artless by design. No sprites, no characters, no illustrations. A single curved
CRT glowing in the dark.

```
O.R.B.S. v0.9.3  —  cold start

scrying lens ................ [ ok ]
ley-line uplink ............. [ ok ]
grimoire index .............. [ 2841 ]
laboratory .................. [ ok ]
sanctum ..................... [ DEGRADED ]
  barrier integrity 34%
menagerie ................... [ not found ]
archive ..................... [ ok ]

3 warnings. the orb warms to your touch.

orbs:~$ _
```

---

**Status: in production.** Phases 0, 0.5, 1, 2 and 4 are closed and Phase 3 is
met on its exit criterion. The determinism spine, the parser, the cell renderer,
brewing, the archive, the lens, the sanctum, the tower rail, the balance harness,
the spell engine and its scripting language are built, and the game plays.
Phase 5 (Summoning) is next.

Rust · Bevy 0.19 · Windows, macOS, Linux · GPL-3.0-or-later

## For players

| | |
|---|---|
| [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md) | How to play — controls, the rooms, your first ten minutes |
| [docs/PLAYER_README.txt](docs/PLAYER_README.txt) | Save data, screenshots, reporting a bug |
| [docs/HEALTH_WARNING.md](docs/HEALTH_WARNING.md) | Health & safety. **Nothing in this game flashes** |
| [docs/CREDITS.md](docs/CREDITS.md) | What it is built from, and the licences that travel with it |
| [docs/PRIVACY_POLICY.md](docs/PRIVACY_POLICY.md) | No networking, no telemetry, one plain-text save |
| [docs/CHANGELOG.md](docs/CHANGELOG.md) | Release notes |

In the game itself, **`help`** explains the room you are standing in and lists
every word it takes there, and **`recall <anything>`** is the manual page for a
verb, a potion, or a word of the scripting language.

## For developers

| | |
|---|---|
| [docs/DESIGN.md](docs/DESIGN.md) | The design — authoritative for everything. §19 is a decisions log |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Phase status, derived from DESIGN.md §15 |
| [docs/SETUP.md](docs/SETUP.md) | Toolchain, skills, build, test and release |
| [CLAUDE.md](CLAUDE.md) | Conventions, architectural rules, and every See-it line |

### Building

```bash
cargo run -p orbs                    # the game
ORBS_BOOT=0 cargo run -p orbs        # ...skipping the 13-second boot sequence
cargo run -p orbs-tui                # the terminal build
```

### The gate

Run after every step, not every phase:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo build -p orbs                  # the binary must LINK, not just check
```

### Looking at it

Work is not done when it compiles. It is done when it has been *looked at* —
every roadmap item carries a "See it" line, and CLAUDE.md lists the `ORBS_*`
environment switches that reach the parts a still photograph cannot show.

```bash
ORBS_DUMP="attend laboratory; grind sage" cargo run -p orbs   # a screen, as text
cargo run -p orbs-render --example screens                    # every surface, no GPU
cargo run -p orbs-balance -- sweep --ticks 7200               # the economy, measured
scripts/play.sh                                               # 113 scenarios, played
scripts/dumps.sh <dir>                                        # 65 screens, captured
```

### Workspace

```
crates/
├── orbs-sim/       world model, parser, script engine, aberrations, threats
│                   — ZERO Bevy engine dependency, headless, fully testable
├── orbs-render/    Frame / cell-buffer, layout, semantic styling
├── orbs-shell/     the shell both frontends share: painters, surfaces, the dump
├── orbs/           Bevy frontend: GPU cell renderer, CRT
├── orbs-tui/       terminal frontend
└── orbs-balance/   CLI harness driving orbs-sim
```

## Licence

GPL-3.0-or-later. See [LICENSE](LICENSE).

The Spleen bitmap font © Frederic Cambus is used under the BSD 2-Clause licence,
whose notice must travel with any binary distribution — see
[docs/CREDITS.md](docs/CREDITS.md).
