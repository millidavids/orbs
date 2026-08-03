# Recognising and fixing stale Bevy code

Read this when Bevy code fails to compile, when adapting a snippet found online,
or when upgrading versions.

## Why this file exists

Bevy ships breaking changes every minor release (~3-5 months). The overwhelming
majority of Bevy code in blogs, tutorials, Stack Overflow answers, and model
training data targets **0.14-0.16**. Assume any snippet you did not just compile
is stale.

## Stale-code smell test

If you see any of these, the snippet predates 0.16 and needs rewriting:

| Stale symbol | Status | Replacement |
|---|---|---|
| `SpriteBundle`, `Camera2dBundle`, `NodeBundle`, `TextBundle`, any `*Bundle` | removed in 0.16 | spawn components directly; use `#[require(...)]` |
| `time.delta_seconds()` | renamed in 0.16 | `time.delta_secs()` |
| `time.elapsed_seconds()` | renamed in 0.16 | `time.elapsed_secs()` |
| `Parent` component | renamed | `ChildOf`, read via `.parent()` |
| `.push_children(...)` | replaced | `children![...]` macro, or `.add_children(...)` |
| `#[derive(Event)]` + `EventWriter`/`EventReader` | reworked in 0.17 | `#[derive(Message)]` + `MessageWriter`/`MessageReader`, `.write(...)` |
| `Trigger<E>` in observers | renamed | `On<E>`, with `#[derive(EntityEvent)]` |
| `query.single()` returning the value directly | now fallible | `Single<...>` param, or `single()?` in a `-> Result` system |
| `ExecutorKind::SingleThreaded` | removed in 0.19 | `Schedule::set_executor(SingleThreadedExecutor::new())` |
| `init_non_send_resource` | deprecated in 0.19 | `init_non_send` |

## 0.19-specific: resources are components

The headline change in 0.19. `#[derive(Resource)]` now also implements
`Component`, and resources are stored as components on dedicated entities.

Practical consequences:

1. **A type cannot derive both `Component` and `Resource`.** If you need both
   shapes, define two types.
2. **Very broad queries now match resource entities.** `Query<Entity>` and
   `Query<()>` pick up the entities backing resources, and may report access
   conflicts they did not before. Fix by narrowing the query to the components
   you actually want, which you should be doing anyway.

## Upgrade procedure

1. Bump one minor version at a time. Do not jump 0.16 → 0.19 in one step.
2. Read the official migration guide for **each** hop:
   `https://bevy.org/learn/migration-guides/0-18-to-0-19/` (substitute versions).
3. Compile and fix. The compiler catches the large majority of breakage.
4. Run the game — the rest surfaces as startup panics (scheduling/query
   conflicts), which Bevy reports with a `B####` code.
5. Update the `targets_bevy` field in the `bevy-idioms` skill frontmatter and
   re-verify its snippets against the new version.

## Verifying a snippet before trusting it

The reliable move, given the churn: compile it. A scratch crate with the
project's exact Bevy version and a `cargo check` settles any API question faster
than searching for documentation that may itself be stale.
