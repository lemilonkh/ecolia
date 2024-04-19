use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_turborand::prelude::*;
use std::{collections::HashMap, f32::consts::PI, time::Duration};

// assets
const ANIMATION_COUNT: usize = 12;

// world
const STAGE_SIZE: f32 = 100.0;
const TREE_SPAWN_TIME: f32 = 1.2;

// simulation
const BASE_VELOCITY: f32 = 20.0;
const RUN_ENERGY_DRAIN: f32 = 0.02;
const RUN_HUNGER_DRAIN: f32 = 0.005;
const RUN_THIRST_DRAIN: f32 = 0.01;
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

#[derive(Component, PartialEq, Debug)]
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
            thirst: 1.0,
            hunger: 1.0,
        }
    }
}

#[derive(Component)]
struct CameraMarker;

const ANIMALS: &[&str] = &["Alpaca", "Deer", "Fox", "Husky", "Stag", "Wolf"];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RngPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, (setup, setup_world, add_animals, add_nature))
        .add_systems(
            Update,
            (
                setup_scene_once_loaded,
                keyboard_input,
                mouse_input,
                respawn_trees,
                find_velocity,
                update_animals,
                update_animal_animations,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(80.0, 30.0, 180.0)
                .looking_at(Vec3::new(50.0, 0.0, 50.0), Vec3::Y),
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
struct Animations(HashMap<String, Vec<Handle<AnimationClip>>>);
#[derive(Resource)]
struct CursorTarget(Vec3);
#[derive(Component)]
struct Animal {
    name: String,
}
#[derive(Component)]
struct Selected;
#[derive(Component)]
struct Target(Vec3);
#[derive(Component)]
enum PlantType {
    Tree,
    Bush,
}
#[derive(Component)]
struct TreeSpawner(Timer);

fn add_animals(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut global_rng: ResMut<GlobalRng>,
) {
    let mut rng = RngComponent::from(&mut global_rng);
    let mut animations: HashMap<String, Vec<Handle<AnimationClip>>> = HashMap::new();

    for animal in ANIMALS {
        let file_name = format!("animals/{}.glb", animal);
        // build animation graphs
        let mut animation_clips: Vec<Handle<AnimationClip>> = vec![];
        for i in 0..ANIMATION_COUNT {
            animation_clips.push(assets.load(format!("{}#Animation{}", file_name, i)));
        }
        animations.insert(animal.to_string(), animation_clips);

        let alpaca = assets.load(format!("{}#Scene0", file_name));
        commands.spawn((
            Animal {
                name: animal.to_string(),
            },
            SceneBundle {
                scene: alpaca,
                transform: Transform::from_xyz(rng.f32() * STAGE_SIZE, 0.0, rng.f32() * STAGE_SIZE),
                ..default()
            },
            Vitality::default(),
            Velocity(Vec3::new(0.0, 0.0, 5.0)),
            AnimalState::Idle,
            RngComponent::from(&mut global_rng),
            Target(Vec3::new(
                rng.f32() * STAGE_SIZE,
                0.0,
                rng.f32() * STAGE_SIZE,
            )),
        ));
    }

    commands.insert_resource(Animations(animations));
}

#[derive(Resource)]
struct PlantMeshes(Vec<Handle<Scene>>);

fn add_nature(mut commands: Commands, assets: Res<AssetServer>, mut global_rng: ResMut<GlobalRng>) {
    let mut rng = RngComponent::from(&mut global_rng);
    let mut meshes = vec![];

    for i in 2..=5 {
        let file_name = format!("nature/BirchTree_{}.glb", i);
        let mesh = assets.load(format!("{}#Scene0", file_name));
        meshes.push(mesh.clone());
        for _j in 0..8 {
            commands.spawn((
                SceneBundle {
                    scene: mesh.clone(),
                    transform: Transform::from_xyz(
                        rng.f32() * STAGE_SIZE,
                        0.0,
                        rng.f32() * STAGE_SIZE,
                    )
                    .with_scale(Vec3::splat(2.0)),
                    ..default()
                },
                PlantType::Tree,
            ));
        }
    }

    commands.insert_resource(PlantMeshes(meshes));
    commands.spawn((
        TreeSpawner(Timer::from_seconds(TREE_SPAWN_TIME, TimerMode::Repeating)),
        rng,
    ));
}

fn respawn_trees(
    mut tree_spawner: Query<(&mut TreeSpawner, &mut RngComponent)>,
    time: Res<Time>,
    mut commands: Commands,
    plant_meshes: Res<PlantMeshes>,
) {
    for (mut timer, mut rng) in &mut tree_spawner {
        if timer.0.tick(time.delta()).just_finished() {
            commands.spawn((
                SceneBundle {
                    scene: plant_meshes.0[0].clone(),
                    transform: Transform::from_xyz(
                        rng.f32() * STAGE_SIZE,
                        0.0,
                        rng.f32() * STAGE_SIZE,
                    )
                    .with_scale(Vec3::splat(2.0)),
                    ..default()
                },
                PlantType::Tree,
            ));
        }
    }
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parent_query: Query<&Parent>,
    animals: Query<&Animal>,
) {
    for (entity, mut player) in &mut players {
        let Ok(parent) = parent_query.get(entity) else {
            println!("Missing parent!");
            continue;
        };
        let Ok(grandparent) = parent_query.get(parent.get()) else {
            println!("Missing grandparent!");
            continue;
        };
        let Ok(animal) = animals.get(grandparent.get()) else {
            println!("Missing animals!");
            continue;
        };

        println!(
            "Anims {}: Count {}",
            animal.name,
            animations.0[&animal.name].len()
        );
        player
            .play(animations.0[&animal.name][4].clone_weak())
            .repeat();
        player.seek_to(entity.index() as f32 / 10.0);
    }
}

fn find_velocity(
    mut query: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut AnimalState,
            &Vitality,
            &mut Target,
            &mut RngComponent,
        ),
        With<Animal>,
    >,
    plants: Query<(Entity, &Transform, &PlantType), Without<Animal>>,
    mut commands: Commands,
) {
    for (mut velocity, mut transform, mut state, vitality, mut target, mut rng) in &mut query {
        if *state != AnimalState::Running && *state != AnimalState::Idle {
            continue;
        }

        let to_target = target.0 - transform.translation;
        if to_target.length() < 0.8 {
            *state = AnimalState::Eating;

            // TODO add Wait(Timer) component to entity, execute this in timer system when done
            let mut min_dist = f32::INFINITY;
            let mut closest_plant: Option<Vec3> = None;
            if rng.f32() < 0.5 {
                for (entity, plant_transform, _plant_type) in &plants {
                    if plant_transform.translation == target.0 {
                        commands.entity(entity).despawn_recursive();
                        continue;
                    }
                    let dist = plant_transform
                        .translation
                        .distance_squared(transform.translation);
                    if dist < min_dist {
                        min_dist = dist;
                        closest_plant = Some(plant_transform.translation);
                    }
                }
            } else {
                let transforms = plants
                    .iter()
                    .map(|(_, transform, _)| transform)
                    .collect::<Vec<_>>();
                closest_plant = Some(
                    transforms[(rng.u32(0..100) % transforms.len() as u32) as usize].translation,
                );
            }

            let Some(target_pos) = closest_plant else {
                continue;
            };

            target.0 = target_pos;
            continue;
        }

        *state = AnimalState::Running;
        velocity.0 = to_target.normalize();
        velocity.0 *= BASE_VELOCITY * vitality.energy;
        transform.look_to(velocity.0, Vec3::Y);
        transform.rotate_local_y(PI);
    }
}

fn update_animal_animations(
    mut players: Query<(Entity, &mut AnimationPlayer)>,
    query: Query<(&AnimalState, &Animal, &Vitality)>,
    animations: Res<Animations>,
    parent_query: Query<&Parent>,
) {
    for (entity, mut player) in &mut players {
        let Ok(parent) = parent_query.get(entity) else {
            println!("Missing parent!");
            continue;
        };
        let Ok(grandparent) = parent_query.get(parent.get()) else {
            println!("Missing grandparent!");
            continue;
        };
        let Ok(animal_components) = query.get(grandparent.get()) else {
            println!("Missing animal components!");
            continue;
        };
        let (state, animal, vitality) = animal_components;

        let animation_index: usize = match state {
            AnimalState::Idle => 6,
            AnimalState::Running => 4,
            AnimalState::Eating => 11,
            _ => 8,
        };
        player
            .play_with_transition(
                animations.0[&animal.name][animation_index].clone_weak(),
                Duration::from_millis(250),
            )
            .repeat();
        player.set_speed(vitality.energy);
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
                *state = AnimalState::Dead;
            }
            vitality.hunger = f32::max(
                vitality.hunger - RUN_HUNGER_DRAIN * time.delta_seconds(),
                0.0,
            );
            vitality.thirst = f32::max(
                vitality.thirst - RUN_THIRST_DRAIN * time.delta_seconds(),
                0.0,
            );
        }

        if *state == AnimalState::Eating {
            vitality.energy = f32::min(
                vitality.energy + EAT_ENERGY_GAIN * time.delta_seconds(),
                1.0,
            );
            vitality.hunger = f32::max(vitality.hunger - EAT_DURATION * time.delta_seconds(), 0.0);
            *state = AnimalState::Idle;
        }

        if *state == AnimalState::Drinking {
            vitality.energy = f32::min(
                vitality.energy + DRINK_ENERGY_GAIN * time.delta_seconds(),
                1.0,
            );
            vitality.thirst =
                f32::max(vitality.thirst - DRINK_DURATION * time.delta_seconds(), 0.0);
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
        0.8,
        Color::WHITE,
    );

    if buttons.just_pressed(MouseButton::Left) {
        cursor_target.0 = point;
    }
}
