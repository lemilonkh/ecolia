use bevy::prelude::*;

use crate::{
    plants::{spawn_tree, PlantMeshes},
    CameraMarker, RngResource,
};

#[derive(Resource)]
pub struct CursorTarget(pub Vec3);

#[derive(Component)]
pub struct Selected;

pub fn mouse_input(
    buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<CameraMarker>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
    mut cursor_target: ResMut<CursorTarget>,
    mut commands: Commands,
    plant_meshes: Res<PlantMeshes>,
    mut rng: ResMut<RngResource>,
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
        spawn_tree(point, &mut commands, &plant_meshes.0, &mut rng.0);
    }
}

pub fn keyboard_input(
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
