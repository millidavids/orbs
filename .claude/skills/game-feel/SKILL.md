---
name: game-feel
description: >
  Make actions feel satisfying — screen shake, hit-stop/freeze frames, easing and
  tweening, squash & stretch, knockback, damage flash, and layered audio-visual
  feedback. Covers the technique, the tuning numbers that actually work, and
  Bevy 0.19 implementations. Use this skill whenever the user mentions game feel,
  "juice", "make it feel good/punchy/satisfying", screen shake, hit stop, easing,
  squash and stretch, impact frames, camera kick, or polish and feedback on hits,
  jumps, pickups, and deaths — and proactively when reviewing gameplay code that
  applies damage or triggers an impact with no feedback attached.
when_to_use: >
  Triggers: "game feel", "juice", "juicy", "feels floaty", "feels weak",
  "make the hit land", "screen shake", "hit stop", "freeze frame", "easing",
  "tween", "squash and stretch", "knockback", "impact", "polish", "camera shake".
paths:
  - "**/*.rs"
metadata:
  targets_bevy: "0.19"
  verified: "All Bevy code in this skill compiled against bevy 0.19.0 on 2026-08-02."
  concepts_adapted_from: "gamedev-skills/awesome-gamedev-agent-skills (Apache-2.0); snippets rewritten for Bevy."
  author: davidyurek
---

# Game feel (juice)

The gap between a mechanic that *works* and one that feels *good* is feedback.
The underlying hit already registered — feel is everything you layer on top so
the player's body believes it.

## The layering principle

A satisfying impact is never one effect. It is 4-6 cheap effects fired together
within ~100ms. Any single one looks crude; together they read as weight.

A hit landing should typically fire: **hit-stop** (freeze), **screen shake**,
**damage flash**, **knockback**, **particles**, **sound**. Miss two or three and
it feels thin.

## Tuning numbers that work

Start here, then tune by feel. These are the values that read as "punchy"
without becoming nauseating.

| Effect | Starting value | Notes |
|---|---|---|
| Hit-stop | 40-80ms | Light hits 40ms, heavy 100-150ms. Longer reads as lag. |
| Screen shake | 0.2-0.4 trauma, decay ~1.5/s | Scale offset by trauma² for punch |
| Damage flash | 60-120ms | Pure white or red, then fade |
| Knockback | 150-400 units/s | Must decay fast or it feels floaty |
| Squash & stretch | 10-25% | Beyond ~30% reads as cartoon |
| Tween (UI pop) | 150-250ms | `CubicOut` or `BackOut` for overshoot |

**The single highest-value effect is hit-stop.** If you add one thing, add that.

## Screen shake

Use *trauma*, not direct offset: add trauma on impact, decay it continuously,
and scale the offset by trauma squared. Squaring makes small hits subtle and big
hits dramatic, and the decay means overlapping hits accumulate smoothly instead
of fighting each other.

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Trauma(pub f32);

#[derive(Component)]
pub struct ShakeCamera { pub base: Vec3 }

pub fn apply_shake(
    time: Res<Time>,
    mut trauma: ResMut<Trauma>,
    mut q: Query<(&mut Transform, &ShakeCamera)>,
) {
    trauma.0 = (trauma.0 - time.delta_secs() * 1.5).max(0.0);
    let shake = trauma.0 * trauma.0;          // square it
    let t = time.elapsed_secs();

    for (mut tf, cam) in &mut q {
        let ox = (t * 37.0).sin() * (t * 11.0).cos();
        let oy = (t * 43.0).cos() * (t * 13.0).sin();
        tf.translation = cam.base + Vec3::new(ox, oy, 0.0) * shake * 12.0;
    }
}
```

Add trauma with `trauma.0 = (trauma.0 + amount).min(1.0);` — clamping is what
stops a burst of hits from throwing the camera off-screen.

Shake the **camera**, never the world. And offer an accessibility toggle: screen
shake is a common motion-sickness trigger, so ship a slider that scales trauma
to zero.

## Hit-stop

Briefly freeze virtual time so the impact frame lands. The critical detail:
**tick the recovery timer on the real clock**, because virtual time is what you
just froze — otherwise you freeze forever.

```rust
#[derive(Resource, Default)]
pub struct HitStop(pub Timer);

pub fn start_hit_stop(mut stop: ResMut<HitStop>, mut time: ResMut<Time<Virtual>>) {
    stop.0 = Timer::from_seconds(0.08, TimerMode::Once);
    time.set_relative_speed(0.0);
}

pub fn tick_hit_stop(
    real: Res<Time<Real>>,                    // REAL clock, not virtual
    mut stop: ResMut<HitStop>,
    mut time: ResMut<Time<Virtual>>,
) {
    if stop.0.tick(real.delta()).just_finished() {
        time.set_relative_speed(1.0);
    }
}
```

A partial freeze (`set_relative_speed(0.15)`) often feels better than a full stop
for rapid attacks, since it keeps animation readable.

## Easing and tweening

Never move UI or feedback elements linearly — linear motion is the single
clearest tell of an unpolished game. Bevy ships easing curves:

```rust
#[derive(Component)]
pub struct Tween {
    pub from: Vec3,
    pub to: Vec3,
    pub timer: Timer,
    pub ease: EaseFunction,
}

pub fn drive_tweens(time: Res<Time>, mut q: Query<(&mut Transform, &mut Tween)>) {
    for (mut tf, mut tween) in &mut q {
        tween.timer.tick(time.delta());
        let curve = EasingCurve::new(tween.from, tween.to, tween.ease);
        if let Some(p) = curve.sample(tween.timer.fraction()) {
            tf.translation = p;
        }
    }
}
```

Which curve to reach for:

- **`CubicOut`** — the default for almost everything entering. Fast start, gentle
  settle.
- **`BackOut`** — overshoot then settle. Pickups, UI pops, anything celebratory.
- **`ElasticOut`** — heavy bounce. Use sparingly; it gets annoying fast.
- **`CubicIn`** — things leaving or being destroyed.
- **Linear** — only for genuinely constant motion like a conveyor.

## Squash and stretch

Volume preservation is what sells it: as one axis compresses, the other expands.
Landing squashes vertically; launching stretches vertically.

```rust
pub fn apply_squash(time: Res<Time>, mut q: Query<(&mut Transform, &mut Squash)>) {
    for (mut tf, mut squash) in &mut q {
        squash.timer.tick(time.delta());
        let k = (1.0 - squash.timer.fraction()) * squash.strength;
        tf.scale = Vec3::new(1.0 + k, 1.0 - k, 1.0);   // wider as it flattens
    }
}
```

## Damage flash

```rust
pub fn damage_flash(time: Res<Time>, mut q: Query<(&mut Sprite, &mut Flash)>) {
    for (mut sprite, mut flash) in &mut q {
        flash.0.tick(time.delta());
        let k = 1.0 - flash.0.fraction();
        sprite.color = Color::srgb(1.0, 1.0 - k, 1.0 - k);
    }
}
```

Remove the `Flash` component when the timer finishes so the sprite returns to its
base colour and the query stops matching.

## Knockback

Apply as an impulse to velocity, then let normal damping decay it. Applying it to
*position* directly is what makes knockback feel like teleporting.

```rust
// Helper, called from a damage observer — not a system itself.
pub fn knockback(q: &mut Query<&mut Velocity>, entity: Entity, dir: Vec2, force: f32) {
    if let Ok(mut vel) = q.get_mut(entity) {
        vel.0 += dir.normalize_or_zero() * force;
    }
}
```

## Wiring it together

An observer on a damage event is the natural place to fire the whole stack, since
it runs immediately and is already targeted at the entity that got hit. Add
trauma, start hit-stop, insert `Flash` and `Squash` components, apply knockback,
spawn particles, play the sound — all from one place.

## Audio is half of feel

Visual feedback without sound feels broken, and it is the cheapest win available.

- **Vary pitch randomly** (±10-15%) on repeated sounds or they turn to machine-gun
  fatigue.
- **Layer by weight**: a light hit gets one sample, a heavy hit gets the same
  sample plus a low thump.
- **Sound leads the visual.** If anything, play audio a frame early — late audio
  reads as lag.

## Diagnosing "it feels off"

| Symptom | Usual cause |
|---|---|
| Floaty | Gravity too low, or knockback/velocity decays too slowly |
| Weak, unsatisfying hits | No hit-stop. Add it first |
| Sluggish, unresponsive | Input buffering missing, or animation blocks control |
| Nauseating | Shake too strong or decays too slowly; camera follows too tightly |
| Cheap / flat | Only one feedback layer firing — stack more |
| Janky motion | Linear easing, or simulation not interpolated to render frames |

## Restraint

Juice is seasoning. Every effect firing on every action produces noise, and
constant screen shake is exhausting rather than exciting. Reserve the heavy stack
for moments that matter — the boss hit, the level clear — and keep routine
actions light. Contrast is what makes the big moments feel big.
