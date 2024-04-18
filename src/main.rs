use std::f32::consts::PI;

use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};

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

#[derive(Component)]
struct CameraMarker;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, add_animals)
        .add_systems(Startup, setup)
        .add_systems(Update, move_all)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 40.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
            ..default()
        },
        CameraMarker,
    ));

    commands.insert_resource(AmbientLight {
        color: Color::ORANGE,
        brightness: 0.82,
    });

    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            color: Color::BEIGE,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn add_animals(mut commands: Commands, assets: Res<AssetServer>) {
    let alpaca = assets.load("animals/Alpaca.gltf#Scene0");
    commands.spawn((
        SceneBundle {
            scene: alpaca,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Vitality::default(),
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
