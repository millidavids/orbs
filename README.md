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
 ██████╗    ██████╗    ██████╗    ███████╗
██╔═══██╗   ██╔══██╗   ██╔══██╗   ██╔════╝
██║   ██║   ██████╔╝   ██████╔╝   ███████╗
██║   ██║   ██╔══██╗   ██╔══██╗   ╚════██║
╚██████╔╝██╗██║  ██║██╗██████╔╝██╗███████║██╗
 ╚═════╝ ╚═╝╚═╝  ╚═╝╚═╝╚═════╝ ╚═╝╚══════╝╚═╝
     Operational Relic Bewitching System

blackhearth games ........................ ok
```

---

**Status: in production.** The game plays and its premise is complete: seven
domains, the arsenal carrying their output, and a siege that rewrites the
scripts behind you. The latest feature is `0.23`, *the tour*;
[docs/ROADMAP.md](docs/ROADMAP.md) has what remains.

Rust · Bevy 0.20 · Windows, macOS, Linux · GPL-3.0-or-later

## For players

| | |
|---|---|
| [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md) | How to play — a pointer to the in-game manual, which is where it lives now |
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
| [docs/SEEING-IT.md](docs/SEEING-IT.md) | How to reach every surface from the running game, and every `ORBS_*` switch |
| [CLAUDE.md](CLAUDE.md) | Conventions, architectural rules, and the gate |

### Building

```bash
cargo run -p orbs                    # the game
ORBS_BOOT=0 cargo run -p orbs        # ...skipping the boot card
ORBS_BOOT=0 ORBS_THRESHOLD=0 cargo run -p orbs   # ...and the orb's menu, straight into a tower
cargo run -p orbs-tui                # the terminal build
```

### The gate

Run after every step, not every phase:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --features orbs-augury/train -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo build -p orbs                  # the binary must LINK, not just check
```

### Looking at it

Work is not done when it compiles. It is done when it has been *looked at* —
every roadmap item carries a "See it" line, and docs/SEEING-IT.md lists the
`ORBS_*` environment switches that reach the parts a still photograph cannot
show.

```bash
ORBS_DUMP="attend laboratory; grind sage" cargo run -p orbs   # a screen, as text
cargo run -p orbs-render --example screens                    # every surface, no GPU
cargo run -p orbs-balance -- sweep --ticks 7200               # the economy, measured
scripts/play.sh                                               # 210 scenarios, played
scripts/dumps.sh <dir>                                        # 227 screens, captured
scripts/tour.sh all                                           # every surface, moving — below
```

### The visual showcase

`scripts/tour.sh` plays the game in its real window, one scene at a time: every
domain at work, every puzzle mid-move, every screen crossing to the next, the
boot card, the menus, and every effect the glass has. A scene sets itself up at
once, then types its own commands at the keyboard as a player would, and holds
for about five seconds. 87 scenes, in a debug build.

```bash
scripts/tour.sh                          # list every scene: name, how long, what it shows
scripts/tour.sh all                      # all of them, one after another, in one window
scripts/tour.sh forge                    # a group: every forge.* scene
scripts/tour.sh forge.lattice,lens       # names and groups, comma-separated
scripts/tour.sh forge.lattice            # one scene; when its steps end, it is yours to play
scripts/tour.sh shots <dir> [scenes]     # screenshots, each with its frame as text, then exit
ORBS_TOUR=forge.lattice ORBS_DUMP=1 cargo run -p orbs    # a scene's first frame, as text
```

The groups are `boot`, `orb` (the menu, the manual, the installation),
`laboratory`, `archive`, `lens`, `sanctum`, `menagerie`, `bailey`, `forge`,
`passage` (screens crossing), `progress`, `spell`, `stores`, `sealed`, `ley`,
`glass` (the tube, phosphors, greyscale, strain, afterglow) and `text`.

In the window, **F11** is the next scene and **Shift+F11** the one before — the
same one again at either end, so F11 replays a single scene. **Any other key**
takes the scene over: its typing stops, it stops moving on, and the game is
yours. The title bar names the scene. A tour keeps nothing — no save, no
settings — and starts every scene from the default settings.

`shots` writes `<NN>-<scene>-<seconds>s.png` for each screenshot with the frame
beside it as `.txt`, an `index.txt` that marks a blank capture, and the run's
`stderr.log`, whose errors and warnings it prints. The scenes live in
[crates/orbs-shell/src/tour/scenes.toml](crates/orbs-shell/src/tour/scenes.toml);
`ORBS_TOUR_FILE=<an edited copy> scripts/tour.sh …` plays one without a
rebuild. `cargo test` plays every scene with no window and holds each to what
it claims to show. [docs/SEEING-IT.md](docs/SEEING-IT.md), *The tour*, has the
rest.

### Workspace

```
crates/
├── orbs-sim/       world model, parser, script engine, aberrations, threats
│                   — ZERO Bevy engine dependency, headless, fully testable
├── orbs-render/    Frame / cell-buffer, layout, semantic styling
├── orbs-shell/     the shell both frontends share: painters, surfaces, the dump
├── orbs/           Bevy frontend: GPU cell renderer, CRT, sound
├── orbs-tui/       terminal frontend
├── orbs-augury/    the trained readers; training behind the `train` feature
└── orbs-balance/   CLI harness driving orbs-sim
```

## Licence

GPL-3.0-or-later. See [LICENSE](LICENSE).

The Spleen bitmap font © Frederic Cambus is used under the BSD 2-Clause licence,
whose notice must travel with any binary distribution — see
[docs/CREDITS.md](docs/CREDITS.md).
