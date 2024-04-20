use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_turborand::prelude::*;

use crate::{INITIAL_TREE_COUNT, STAGE_SIZE, TREE_SPAWN_TIME};

#[derive(Component)]
pub enum PlantType {
    Tree,
    Bush,
}
#[derive(Component)]
pub struct TreeSpawner(Timer);

#[derive(Resource)]
pub struct PlantMeshes(pub Vec<Handle<Scene>>);

pub fn add_nature(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut global_rng: ResMut<GlobalRng>,
) {
    let mut rng = RngComponent::from(&mut global_rng);
    let mut meshes = vec![];

    for i in 2..=5 {
        let file_name = format!("nature/BirchTree_{}.glb", i);
        let mesh = assets.load(format!("{}#Scene0", file_name));
        meshes.push(mesh.clone());
    }
    for _j in 0..INITIAL_TREE_COUNT {
        spawn_tree(
            Vec3::new(rng.f32() * STAGE_SIZE, 0.0, rng.f32() * STAGE_SIZE),
            &mut commands,
            &meshes,
            &mut rng,
        );
    }

    commands.insert_resource(PlantMeshes(meshes));
    commands.spawn((
        TreeSpawner(Timer::from_seconds(TREE_SPAWN_TIME, TimerMode::Repeating)),
        rng,
    ));
}

pub fn respawn_trees(
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

// helper function
pub fn spawn_tree(
    position: Vec3,
    commands: &mut Commands,
    plant_meshes: &[Handle<Scene>],
    rng: &mut RngComponent,
) {
    commands.spawn((
        SceneBundle {
            scene: plant_meshes[rng.usize(0..plant_meshes.len())].clone(),
            transform: Transform::from_translation(position)
                .with_scale(Vec3::splat(rng.f32() + 1.5))
                .with_rotation(Quat::from_rotation_y(rng.f32() * 2.0 * PI)),
            ..default()
        },
        PlantType::Tree,
    ));
}
