use animals::{
    add_animals, find_velocity, process_wait_timer, setup_animations_once_loaded,
    update_animal_animations, update_animals,
};
use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_turborand::prelude::*;
use input::{keyboard_input, mouse_input, CursorTarget};
use plants::{add_nature, respawn_trees};
use std::f32::consts::PI;

mod animals;
mod input;
mod plants;
mod utils;

// assets
const ANIMATION_COUNT: usize = 12;

// world
const STAGE_SIZE: f32 = 100.0;
const TREE_SPAWN_TIME: f32 = 1.2;
const INITIAL_TREE_COUNT: usize = 20;

// simulation
const BASE_VELOCITY: f32 = 20.0;
const RUN_ENERGY_DRAIN: f32 = 0.02;
const RUN_HUNGER_DRAIN: f32 = 0.005;
const RUN_THIRST_DRAIN: f32 = 0.01;
const EAT_ENERGY_GAIN: f32 = 0.2;
const EAT_DURATION: f32 = 2.0;
const DRINK_ENERGY_GAIN: f32 = 0.2;
const DRINK_DURATION: f32 = 2.0;
const CLOSEST_PLANT_PROBABILITY: f32 = 0.6;

#[derive(Component)]
struct CameraMarker;

#[derive(Resource)]
struct RngResource(RngComponent);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RngPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, (setup, setup_world, add_animals, add_nature))
        .add_systems(
            Update,
            (
                setup_animations_once_loaded,
                keyboard_input,
                mouse_input,
                respawn_trees,
                find_velocity,
                update_animals,
                update_animal_animations,
                process_wait_timer,
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
