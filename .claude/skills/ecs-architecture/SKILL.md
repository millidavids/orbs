---
name: ecs-architecture
description: >
  Design-level guidance for Entity Component System architecture — how to shape
  components and systems, choose component granularity, model relationships and
  state, keep simulation deterministic and testable, and avoid the common
  ECS anti-patterns (god components, entity-as-object, hidden coupling through
  shared mutable state). Engine-neutral: applies to Bevy, hecs, flecs, EnTT,
  Unity DOTS, or a hand-rolled ECS. Use this skill whenever the user is designing
  or refactoring game/simulation architecture, deciding what should be a
  component vs a resource vs a system, asking "how should I structure this",
  or reviewing ECS code for design (not syntax) problems.
when_to_use: >
  Triggers: "how should I model", "should this be a component", "ECS design",
  "refactor my systems", "god component", "my systems are tangled",
  "make this deterministic", "how do I test my game logic", architecture reviews
  of entity/component/system code.
metadata:
  scope: engine-neutral design guidance, no version-specific APIs
  author: davidyurek
---

# ECS architecture

Design guidance, not syntax. For Bevy-specific APIs use the `bevy-idioms` skill.

## The one idea

ECS is **data-oriented**, not object-oriented. An entity is an ID, not an object.
Components are plain data with no behaviour. Systems are functions over sets of
components. The moment you find yourself asking "what methods does this entity
have," you have stopped doing ECS.

The payoff is that behaviour composes by *adding data* rather than by editing a
class. A `Poisoned` component plus a `poison_tick` system adds poison to every
entity in the game — enemies, the player, destructible crates — without touching
any of them.

## Shaping components

**Prefer many small components.** Granularity is what makes queries precise and
parallelism possible. `Position` + `Velocity` + `Health` beats one `Entity`
struct with fifteen fields.

**Split by access pattern, not by concept.** If two fields are always written by
different systems, they belong in different components — that is exactly what
lets those systems run in parallel. If they are always read and written
together, keep them together.

**Marker components are free and underused.** A fieldless `Player`, `Enemy`,
`Dead`, or `Frozen` costs nothing and turns an `if` inside a system into a query
filter, which is both faster and clearer.

**State is often better as a component than a field.** Instead of
`enum State { Idle, Walking, Attacking }` on a big struct, consider `Idle`,
`Walking`, `Attacking` marker components and systems that query for one. Adding
and removing them makes "what is this entity doing" a query rather than a branch.
The tradeoff is structural churn, so avoid this for state that changes every
frame.

**Keep engine handles out of simulation components.** A component holding a
texture handle, a render node, or a GPU resource welds your simulation to the
renderer and usually forces main-thread-only access. Keep a component of plain
data, and let a separate sync system map it to the renderer.

## Shaping systems

**One system, one job.** A system named `update_game` is a smell. Systems should
be small enough that their parameter list documents exactly what they touch.

**Declare the narrowest access that works.** `&T` instead of `&mut T` wherever
possible — read-only access is what lets the scheduler run systems concurrently.

**Order only what must be ordered.** Every ordering constraint is lost
parallelism. Most systems genuinely do not care.

**Communicate through data, not calls.** Systems do not call each other. They
leave components, resources, or events behind for other systems to observe. If
you want system A to "tell" system B something, the answer is a component or an
event, never a function call.

## Relationships

Parent/child hierarchies, inventories, and targeting are all relationships
between entities. Store the entity ID and treat it as a weak reference — the
target can be despawned at any time, so **every lookup must handle absence.** A
component holding `Entity` is a foreign key, not a guarantee.

Beware of the reverse-lookup trap: if you frequently need "all entities pointing
at me," maintain the reverse index as data rather than scanning every frame.

## Determinism

Determinism is what makes a simulation debuggable, replayable, and networkable.
It is far cheaper to preserve from day one than to retrofit.

- Run simulation on a **fixed timestep**, never on frame delta.
- Do not let **iteration order** affect results. ECS storage order is an
  implementation detail and can change when components are added or removed. If
  a system's outcome depends on the order it visits entities, sort explicitly.
- Seed randomness explicitly and store the RNG state as data.
- Keep floating-point work out of cross-platform-critical paths if you need
  bit-exact determinism, or accept that you do not.

## Testing

The great practical advantage of ECS: **simulation logic has no engine
dependency**, so it can be tested headlessly and fast.

Structure the project so the simulation crate does not depend on the renderer or
windowing at all. Then a test is: build a world, insert components, run the
schedule N times, assert on components. No window, no GPU, milliseconds per test.

If your simulation cannot be tested without opening a window, the layering is
wrong — that is the signal to move logic out of render-coupled systems.

## Anti-patterns

**God component.** One component with everything in it. Defeats granularity,
serialises everything that touches it. Split by access pattern.

**Entity as object.** Methods hanging off a wrapper struct, inheritance-shaped
hierarchies, "base entity" components. Model behaviour as systems over data.

**Systems that query everything.** `Query<(&mut A, &mut B, &mut C, &mut D)>`
conflicts with nearly every other system and kills parallelism. Usually a sign
one system is doing several jobs.

**Branch-instead-of-filter.** `for e in all_entities { if !e.is_enemy { continue } }`
should be a query filter. Filters are faster and self-documenting.

**Hidden coupling through a shared mutable resource.** A global `GameData`
resource that half the systems mutate reintroduces exactly the tangle ECS exists
to prevent. Prefer narrow resources, or components on entities.

**Structural churn in hot loops.** Adding/removing components moves entities
between archetypes, which is comparatively expensive. For state toggling many
times per second, prefer a field or a bitflag over add/remove.

**Premature parallelism.** Correct data shapes give you parallelism for free.
Contorting the design to chase it before profiling is wasted effort.

## When ECS is the wrong tool

ECS pays off with many entities sharing behaviour. It is overhead when:

- The game has a handful of bespoke objects (most puzzle and narrative games).
- Logic is dominated by sequential scripted flow rather than per-frame updates.
- The problem is really a document/state tree — UI, dialogue graphs, menus —
  where a plain data structure is simpler and clearer.

Using ECS for the simulation and something simpler for UI and flow is a normal,
healthy split. Do not force everything through the ECS because the engine
offers one.
