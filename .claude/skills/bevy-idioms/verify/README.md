# Verification harness

Every code snippet in `bevy-idioms` and `game-feel` lives here as compilable
source. This is how the skills avoid silently rotting across Bevy releases.

## Verify the skill against the current Bevy

```sh
cd ~/.claude/skills/bevy-idioms/verify
cargo check
```

Zero errors means the skills' snippets are accurate for the Bevy version in
`Cargo.toml`.

## On a Bevy upgrade

1. Bump the version in `verify/Cargo.toml`.
2. `cargo check`.
3. Fix each error here first — the compiler tells you exactly what changed.
4. Port the fixes back into `../SKILL.md`, `../references/`, and
   `../../game-feel/SKILL.md`.
5. Update the `targets_bevy` and `verified` fields in both skills' frontmatter.
6. Add any renamed API to the stale-symbol table in
   `../references/migration-0-19.md`.

## Layout

| File | Covers |
|---|---|
| `src/lib.rs` | components, required components, resources, systems, queries, filters, commands, hierarchies, messages, observers, states, scheduling, plugins |
| `src/round2.rs` | ParamSet, `Single`, `Local`, exclusive world access, run conditions, 2D setup, gizmos |
| `src/round3.rs` | game-feel: screen shake, hit-stop, easing, squash & stretch, knockback, damage flash |

Last verified: **bevy 0.19.0**, 2026-08-02.
