use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use std::f32::consts::PI;

const ANIMAL_PATH: &str = "animals/Alpaca.gltf";

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
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_world)
        .add_systems(Startup, add_animals)
        .add_systems(Update, move_all)
        .add_systems(Update, (setup_scene_once_loaded, keyboard_input))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(30.0, 30.0, 60.0)
                .looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
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
            first_cascade_far_bound: 200.0, //4.0,
            maximum_distance: 400.0,        //10.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(5000.0, 5000.0)),
        material: materials.add(Color::rgb(0.1, 0.7, 0.2)),
        transform: Transform::from_translation(Vec3::Y / 2.0),
        ..default()
    });
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

fn add_animals(mut commands: Commands, assets: Res<AssetServer>) {
    // build animations graphs
    commands.insert_resource(Animations(vec![
        assets.load(format!("{}#Animation4", ANIMAL_PATH)),
        assets.load(format!("{}#Animation1", ANIMAL_PATH)),
    ]));

    let alpaca = assets.load("animals/Alpaca.gltf#Scene0");
    commands.spawn((
        SceneBundle {
            scene: alpaca,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Vitality::default(),
        Velocity(Vec3::new(0.0, 0.0, 5.0)),
    ));
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        player.play(animations.0[0].clone_weak()).repeat();
    }
}

fn move_all(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        if velocity.0.length() > 0.0 {
            transform.translation += velocity.0 * time.delta_seconds();
        }
    }
}

fn keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for mut player in &mut animation_players {
        if keyboard_input.pressed(KeyCode::Space) {
            if player.is_paused() {
                player.resume();
            } else {
                player.pause();
            }
        }
    }
}
