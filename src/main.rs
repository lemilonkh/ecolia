use bevy::prelude::*;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Vitality {
    health: f32,
    energy: f32,
    thirst: f32,
    hunger: f32,
}

impl Default for Vitality {
    fn default() -> Self {
        Vitality {
            health: 1.0,
            energy: 1.0,
            thirst: 0.0,
            hunger: 0.0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, add_animals)
        .add_systems(Update, move_all)
        .run();
}

fn add_animals(mut commands: Commands) {
    commands.spawn((
        Vitality::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Velocity(Vec3::new(1.0, 0.0, 0.0)),
    ));
}

fn move_all(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        if velocity.0.length() > 0.0 {
            transform.translation += velocity.0 * time.delta_seconds();
        }
    }
}
