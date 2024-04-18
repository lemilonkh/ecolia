use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use std::{f32::consts::PI, time::Duration};

const ANIMAL_PATH: &str = "animals/Alpaca.gltf";
const BASE_VELOCITY: f32 = 20.0;
const RUN_ENERGY_DRAIN: f32 = 0.1;
const EAT_ENERGY_GAIN: f32 = 0.2;
const EAT_DURATION: f32 = 2.0;
const DRINK_ENERGY_GAIN: f32 = 0.2;
const DRINK_DURATION: f32 = 2.0;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Vitality {
    health: f32,
    energy: f32,
    thirst: f32,
    hunger: f32,
}

#[derive(Component, PartialEq)]
enum AnimalState {
    Idle,
    Running,
    Eating,
    Drinking,
    Attacking,
    Dead,
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
        .add_systems(Update, (find_velocity, update_animals))
        .add_systems(
            Update,
            (
                setup_scene_once_loaded,
                keyboard_input,
                mouse_input,
                update_animal_animations,
            ),
        )
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
            translation: Vec3::new(0.0, 100.0, 0.0),
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
    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(5000.0, 5000.0)),
        material: materials.add(Color::rgb(0.1, 0.7, 0.2)),
        transform: Transform::from_translation(Vec3::ZERO),
        ..default()
    });

    commands.insert_resource(CursorTarget(Vec3::ZERO));
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);
#[derive(Resource)]
struct CursorTarget(Vec3);

fn add_animals(mut commands: Commands, assets: Res<AssetServer>) {
    // build animations graphs
    commands.insert_resource(Animations(vec![
        assets.load(format!("{}#Animation6", ANIMAL_PATH)), // idle
        assets.load(format!("{}#Animation4", ANIMAL_PATH)), // run
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
        AnimalState::Idle,
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

fn find_velocity(
    mut query: Query<(&mut Velocity, &mut Transform, &mut AnimalState, &Vitality)>,
    cursor_target: Res<CursorTarget>,
) {
    for (mut velocity, mut transform, mut state, vitality) in &mut query {
        let to_target = cursor_target.0 - transform.translation;
        if to_target.length() < 0.1 {
            if *state == AnimalState::Running {
                *state = AnimalState::Idle;
            }
            return;
        } else {
            *state = AnimalState::Running;
        }

        velocity.0 = to_target.normalize();
        velocity.0 *= BASE_VELOCITY * vitality.energy;
        transform.look_to(velocity.0, Vec3::Y);
        transform.rotate_local_y(PI);
    }
}

fn update_animal_animations(
    mut query: Query<&AnimalState>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animations: Res<Animations>,
) {
    // TODO how to associate players to entities?
    let Ok(mut player) = animation_players.get_single_mut() else {
        return;
    };
    for state in query.iter() {
        let animation_index: usize = match state {
            AnimalState::Idle => 0,
            AnimalState::Running => 1,
            _ => 2,
        };
        player
            .play_with_transition(
                animations.0[animation_index].clone_weak(),
                Duration::from_millis(250),
            )
            .repeat();
    }
}

fn update_animals(
    mut query: Query<(&mut Transform, &mut Vitality, &Velocity, &mut AnimalState)>,
    time: Res<Time>,
) {
    for (mut transform, mut vitality, velocity, mut state) in &mut query {
        if *state == AnimalState::Running && velocity.0.length() > 0.0 {
            transform.translation += velocity.0 * time.delta_seconds();
            vitality.energy = f32::max(
                vitality.energy - RUN_ENERGY_DRAIN * time.delta_seconds(),
                0.0,
            );
            if vitality.energy == 0.0 {
                *state = AnimalState::Idle;
            }
        }

        if *state == AnimalState::Eating {
            vitality.energy += f32::min(EAT_ENERGY_GAIN * time.delta_seconds(), 1.0);
            vitality.hunger -= f32::max(EAT_DURATION * time.delta_seconds(), 0.0);
            *state = AnimalState::Idle;
        }

        if *state == AnimalState::Drinking {
            vitality.energy += f32::min(DRINK_ENERGY_GAIN * time.delta_seconds(), 1.0);
            vitality.thirst -= f32::max(DRINK_DURATION * time.delta_seconds(), 0.0);
            *state = AnimalState::Idle;
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

fn mouse_input(
    buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<CameraMarker>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
    mut cursor_target: ResMut<CursorTarget>,
) {
    let (camera, camera_transform) = camera_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // calculate ray pointing from camera into world based on mouse position
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // calculate if and where ray is hitting ground plane
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Y)) else {
        return;
    };
    let point = ray.get_point(distance);

    gizmos.circle(
        point + Vec3::Y * 0.01,
        Direction3d::new_unchecked(Vec3::Y),
        0.2,
        Color::WHITE,
    );

    if buttons.just_pressed(MouseButton::Left) {
        cursor_target.0 = point;
    }
}
