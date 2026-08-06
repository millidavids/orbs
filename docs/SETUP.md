# O.R.B.S. — Setup

Everything needed to pick this project up on a new machine.

---

## 1. Toolchain

Rust stable, edition 2024. The project pins its toolchain via
`rust-toolchain.toml` (added in Phase 0), so `rustup` will fetch the right
version automatically on first build.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add x86_64-apple-darwin      # macOS universal builds
```

Verify: `rustc --version` — the project was designed against **1.96** (2024
edition).

### Platform build dependencies

**Linux** (also needed by CI):

```sh
sudo apt-get install -y --no-install-recommends \
  libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev pkg-config
```

**macOS** — Xcode Command Line Tools (`xcode-select --install`). Release builds
additionally need a Developer ID certificate and `rcodesign` for notarisation.

**Windows** — cross-compiled from Linux CI via `x86_64-pc-windows-gnu`; no
Windows machine required.

---

## 2. Claude Code skills

Three project skills are **vendored in this repo** at `.claude/skills/` and work
automatically on any machine — no install step:

| Skill | Purpose |
|---|---|
| `bevy-idioms` | Bevy **0.19**-pinned idioms, with a compile-verified harness |
| `ecs-architecture` | Engine-neutral ECS design guidance |
| `game-feel` | Juice patterns with verified Bevy snippets |

One skill is **user-level and must be installed per machine** — it is third-party
(MIT) and too large to vendor sensibly:

```sh
git clone --depth 1 https://github.com/leonardomso/rust-skills.git \
  ~/.claude/skills/rust-skills
```

Then add `paths` gating to its frontmatter so it only activates on Rust work:

```yaml
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
```

> Pinned at commit `fd2a861ab0406a4ac536a55274d14ea6fd1ca9c9` (265 rules, current
> for Rust 1.96). The `paths` gate matters: the skill's index alone is ~9.5k
> tokens per trigger, and without gating it fires on unrelated work.

### Validating Bevy syntax

Bevy breaks APIs every minor release and most published Bevy code targets
0.14–0.16. **Do not trust recalled Bevy syntax.** To check any snippet against the
pinned version:

```sh
cd .claude/skills/bevy-idioms/verify
cargo check          # zero errors = the skill's snippets are accurate
cargo clean          # target/ reaches ~550 MB — always clean up
```

On a Bevy upgrade: bump the version in `verify/Cargo.toml`, fix errors there
first, port fixes back into the skill, then update its `targets_bevy` and
`verified` frontmatter.

---

## 3. Building

```sh
cargo check --workspace          # fast iteration
cargo test -p orbs-sim           # headless sim tests — no GPU, no window
cargo run -p orbs                # Bevy frontend — opens a window, Esc to quit
cargo run -p orbs-tui            # terminal frontend (stub until Phase 1)
cargo run -p orbs-balance -- ... # economy sweeps (stub until Phase 1)
```

### The gate — after every step

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo build -p orbs              # must LINK, not merely check
```

**`cargo check` does not substitute for `cargo build -p orbs`.** `check` stops at
metadata and will pass while the binary fails to link. Bevy is ~126 crates and is
the likeliest thing to break across a toolchain or version change, so it gets
exercised every step rather than at the end of a phase.

Verified working on this toolchain: Metal backend, window creation, 1 Hz
`FixedUpdate` sim driver, 9.5 MB debug binary.

### Seeing it

Compiling is not the same as looking at it.

```sh
cargo run -p orbs                            # the game
ORBS_DUMP=1 cargo run -p orbs                # ...its screen as text, no GPU
ORBS_CAPTURE=1 cargo run -p orbs             # ...or as a screenshot
cargo run -p orbs-render --example screens   # real Frames dumped as text
```

**Prefer `ORBS_DUMP` and keep `ORBS_CAPTURE` for the things only pixels show.**
The screenshot path needs a composited window: run the binary from a detached
shell, or with the display asleep, and it writes a valid PNG of a black
rectangle — the renderer fine, the picture proving nothing. That is worse than no
picture, because it looks like evidence.

`ORBS_DUMP` draws the same frame the game draws — the real `paint`, the real
`Sim`, the real `ScreenLayout` — into a `Frame` nobody rasterises, and prints it
with its linear stream beneath.

```sh
ORBS_DUMP="attend laboratory; move sage to mortar_and_pestle; wield mortar_and_pestle; meditate 12; empty mortar_and_pestle" cargo run -p orbs
ORBS_DUMP=1 ORBS_GRID=160x44 cargo run -p orbs      # the worst-case grid
ORBS_DUMP=1 ORBS_BOOT=post cargo run -p orbs        # one boot stage as text
ORBS_LINE="wield mo" ORBS_DUMP=1 cargo run -p orbs  # ...with a line half-typed
ORBS_BOOT=0 cargo run -p orbs                       # skip the boot sequence
```

`ORBS_BOOT` takes `dark`, `frame` or `post` for a dump, and `0` to skip the
sequence in the running game — boot runs once per launch, so without it every
pass over anything else costs that wait. (`prompt` was a stage until the prompt
was taken out of the boot sequence; `requested_stage` now returns `None` for it,
which silently dumps the **live game screen** instead — a wrong picture that
looks like a right one.)

`ORBS_LINE` puts text in the input buffer. `ORBS_DUMP` submits every segment it
is given, so the buffer is always empty by the time the frame is painted — a
caret position, a partial word and the completion ghost are the three things a
dump cannot otherwise show.

`ORBS_EDIT` types into the spell editor once a `scribe` has opened it —
newline-separated keystrokes, in order. The editor's own two states decide what a
segment is: it opens in **command** state, so the first segment is a word (`edit`
or `quit`, which is the whole vocabulary), `edit` drops into the buffer, and
`<esc>` comes back out.

**There is no `save` word.** The buffer writes itself out a beat after the typing
stops, and that pause is measured off `Time` — which a dump never advances,
having no frames. In a dump, **`quit` is how you save**: it flushes, then closes.
The vim shorthand still separates the two if you need it (`w` writes and stays,
`wq` writes and closes).

`ORBS_THEN` runs commands **after** that session, which is the only way to look
at a spell that was just saved: a save queues its write for the next tick like
every other effect, so a `peruse` inside `ORBS_DUMP` runs before the spell exists.

```sh
# A spell is written *for* a domain, so `scribe` happens from inside one and
# there is no `attend` in the file.
ORBS_DUMP="attend laboratory; scribe brewing" \
ORBS_EDIT="edit\nkindle charcoal\ngrind the sage\nempty mortar_and_pestle\n<esc>\nquit" \
ORBS_THEN="invoke brewing; meditate 40" cargo run -p orbs
```

Each `;`-separated line goes through `submit` and a real `step`, so what prints
is the world having actually run. What it cannot show is what rule 2 says is a
frontend's alone — phosphor, the CRT curve, the blinking caret — and those are
exactly what `ORBS_CAPTURE` and a real window are still for.

`screens` builds the §4 boot report and a multiplexed siege through the same
public API both frontends use, prints them by walking `Frame::rows()` exactly as
a rasteriser would, and prints the linearised (screen-reader) view beside them.
It also asserts §9's Deep/Wide parity rule.

**It has caught bugs the test suite did not** — an em-dash in DESIGN.md's own
boot text that CP437 cannot draw, and pane content overwriting a border because
no sub-painter was established. Add a screen to it whenever a new surface is
built.

### Dev iteration speed

**Measure before changing anything here.** The received advice — dynamic linking
and a faster linker — was written for slower machines and older linkers than
this project has, and on the development machine neither helps.

Measured on an Apple M4 Pro, Xcode `ld-1267`, toolchain 1.96:

| Loop | Default | `--features fast-compile` |
|---|---|---|
| Touch a leaf file in `orbs` | **0.7 s** | 0.6–1.0 s |
| Touch `orbs-render` (rebuilds `orbs` too) | **1.1 s** | 0.9 s |
| Touch `orbs-sim` | **0.9 s** | 0.8 s |
| `cargo test --workspace` | **6.2 s** | — |
| `cargo clippy --workspace --all-targets` | **1.4 s** | — |

So the whole gate is about nine seconds and a rebuild is about one. Dynamic
linking saves roughly a tenth of a second, which is inside the noise, and costs
a flag to remember plus a dev binary laid out differently from the one that
ships. **It is not on by default and should not be turned on without a
measurement showing it helps on your machine.**

```sh
cargo run -p orbs --features fast-compile   # only if you measured a win
```

No fast linker is installed and none is wanted here: Apple's `ld` has been
substantially rewritten since the advice above was current, and installing LLVM
to obtain `lld` would cost more disk than it saves in seconds. On Linux or
Windows the answer may differ — measure there rather than assuming either way.

Build profiles are specified in DESIGN.md §13 (dev `opt-level = 1`, release
`lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`).

---

## 4. CI and release

Mirrors `court_wizard`'s proven workflows:

| Target | Runner | Notes |
|---|---|---|
| `x86_64-pc-windows-gnu` | `ubuntu-latest` | Cross-compiled |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | Needs the apt deps above |
| `aarch64` + `x86_64-apple-darwin` | `macos-latest` | Universal via `lipo`, signed + notarised with `rcodesign`, **release only** |

Workflows: `build.yml`, `macos-release.yml`, `release.yml`, `steam-upload.yml`.

macOS is built only for releases because signing and notarisation are slow.

**Distribution note:** the ship artifact is *not* a single file. Steam integration
means `libsteam_api.{so,dylib,dll}` sits beside the executable, which is why
`.cargo/config.toml` carries rpath flags (`$ORIGIN` on Linux, `@loader_path` on
macOS). Assets are compiled in, so there is no loose `assets/` directory at
runtime.

---

## 5. Resuming work

1. Read [../CLAUDE.md](../CLAUDE.md) — conventions, architectural rules, and where
   everything lives.
2. Read [ROADMAP.md](ROADMAP.md) for phase status.
3. Consult [DESIGN.md](DESIGN.md) for any design question. It is authoritative;
   §19 records what was decided and why, including superseded choices.

**Do not re-litigate settled decisions.** The design went through eight drafts and
four independent staff-level reviews, and several of the least obvious rules exist
because a review caught a real problem — the offline clamp, the Focus-slot
reservation, the `verify --all` cost, and the eldritch/sabotage signal split are
all load-bearing fixes that look arbitrary without their history. §19 has the
history.
