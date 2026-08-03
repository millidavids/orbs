//! Every snippet destined for the bevy-idioms skill. If this compiles against
//! bevy 0.19, the skill is accurate.

use bevy::prelude::*;

// ---------- 1. components, required components ----------
pub mod components {
    use super::*;

    #[derive(Component)]
    pub struct Health(pub f32);

    #[derive(Component, Default)]
    pub struct Velocity(pub Vec2);

    // Required components replace bundles (removed in 0.16).
    #[derive(Component)]
    #[require(Transform, Velocity, Health(100.0))]
    pub struct Player;

    #[derive(Component)]
    pub struct Enemy;
}

// ---------- 2. resources ----------
pub mod resources {
    use super::*;

    #[derive(Resource, Default)]
    pub struct Score(pub u32);

    #[derive(Resource)]
    pub struct Config {
        pub gravity: f32,
    }

    impl Default for Config {
        fn default() -> Self {
            Self { gravity: -9.81 }
        }
    }
}

// ---------- 3. systems: queries, filters, commands ----------
pub mod systems {
    use super::components::*;
    use super::resources::*;
    use super::*;

    pub fn movement(time: Res<Time>, mut q: Query<(&mut Transform, &Velocity)>) {
        for (mut tf, vel) in &mut q {
            tf.translation += vel.0.extend(0.0) * time.delta_secs();
        }
    }

    // With/Without filters
    pub fn only_players(q: Query<&Transform, (With<Player>, Without<Enemy>)>) {
        for _tf in &q {}
    }

    // Change detection
    pub fn on_health_change(q: Query<&Health, Changed<Health>>) {
        for h in &q {
            let _ = h.0;
        }
    }

    pub fn on_added(q: Query<Entity, Added<Enemy>>) {
        for _e in &q {}
    }

    // Commands: deferred spawn/despawn
    pub fn spawn_enemy(mut commands: Commands) {
        commands.spawn((Enemy, Transform::default(), Health(50.0)));
    }

    pub fn despawn_dead(mut commands: Commands, q: Query<(Entity, &Health)>) {
        for (entity, health) in &q {
            if health.0 <= 0.0 {
                commands.entity(entity).despawn();
            }
        }
    }

    // Resource access
    pub fn bump_score(mut score: ResMut<Score>, config: Res<Config>) {
        let _ = config.gravity;
        score.0 += 1;
    }

    // Fallible system (Result-returning)
    pub fn fallible(q: Query<&Transform, With<Player>>) -> Result {
        let tf = q.single()?;
        let _ = tf.translation;
        Ok(())
    }

    // Parallel iteration for heavy work
    pub fn parallel_work(mut q: Query<&mut Transform, With<Enemy>>) {
        q.par_iter_mut().for_each(|mut tf| {
            tf.translation.x += 1.0;
        });
    }
}

// ---------- 4. spawning hierarchies ----------
pub mod spawning {
    use super::components::*;
    use super::*;

    pub fn spawn_with_children(mut commands: Commands) {
        commands.spawn((
            Player,
            Transform::default(),
            children![
                (Enemy, Transform::default()),
                (Enemy, Transform::from_xyz(1.0, 0.0, 0.0)),
            ],
        ));
    }

    pub fn read_hierarchy(q: Query<&Children, With<Player>>, parents: Query<&ChildOf>) {
        for children in &q {
            for _child in children.iter() {}
        }
        for child_of in &parents {
            let _parent = child_of.parent();
        }
    }
}

// ---------- 5. events / messages ----------
pub mod events {
    use super::*;

    #[derive(Message)]
    pub struct Scored {
        pub points: u32,
    }

    pub fn send(mut writer: MessageWriter<Scored>) {
        writer.write(Scored { points: 10 });
    }

    pub fn receive(mut reader: MessageReader<Scored>) {
        for msg in reader.read() {
            let _ = msg.points;
        }
    }
}

// ---------- 6. observers ----------
pub mod observers {
    use super::components::*;
    use super::*;

    #[derive(EntityEvent)]
    pub struct Damage {
        pub entity: Entity,
        pub amount: f32,
    }

    pub fn on_damage(damage: On<Damage>, mut q: Query<&mut Health>) {
        if let Ok(mut health) = q.get_mut(damage.entity) {
            health.0 -= damage.amount;
        }
    }

    pub fn register(app: &mut App) {
        app.add_observer(on_damage);
    }
}

// ---------- 7. states ----------
pub mod states {
    use super::*;

    #[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub enum GameState {
        #[default]
        Menu,
        Playing,
        Paused,
    }

    pub fn enter_playing() {}
    pub fn tick_playing() {}

    pub fn register(app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(OnEnter(GameState::Playing), enter_playing)
            .add_systems(Update, tick_playing.run_if(in_state(GameState::Playing)));
    }
}

// ---------- 8. scheduling: sets, ordering, fixed timestep ----------
pub mod scheduling {
    use super::systems::*;
    use super::*;

    #[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
    pub enum GameSet {
        Input,
        Simulation,
        Render,
    }

    pub fn fixed_step(mut q: Query<&mut Transform>, time: Res<Time<Fixed>>) {
        for mut tf in &mut q {
            tf.translation.y -= 9.81 * time.delta_secs();
        }
    }

    pub fn register(app: &mut App) {
        app.configure_sets(
            Update,
            (GameSet::Input, GameSet::Simulation, GameSet::Render).chain(),
        )
        .add_systems(Update, movement.in_set(GameSet::Simulation))
        .add_systems(Update, (despawn_dead, bump_score).chain())
        .add_systems(FixedUpdate, fixed_step);
    }
}

// ---------- 9. plugin organisation ----------
pub mod plugins {
    use super::resources::*;
    use super::systems::*;
    use super::*;

    pub struct GameplayPlugin;

    impl Plugin for GameplayPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<Score>()
                .init_resource::<Config>()
                .add_systems(Startup, spawn_enemy)
                .add_systems(Update, (movement, despawn_dead));
        }
    }
}

pub mod round2;
pub mod round3;
