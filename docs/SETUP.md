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

*(Phase 0 creates the workspace; these are the intended commands.)*

```sh
cargo check --workspace          # fast iteration
cargo test -p orbs-sim           # headless sim tests — no GPU, no window
cargo run -p orbs                # Bevy frontend
cargo run -p orbs-tui            # terminal frontend
cargo run -p orbs-balance -- ... # economy sweeps
```

### Dev iteration speed

Bevy's compile time is the main friction. Set both up early:

- `bevy/dynamic_linking` in the dev profile
- A fast linker — `lld` or `mold`

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
