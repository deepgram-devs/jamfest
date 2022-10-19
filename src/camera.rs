use bevy::prelude::*;

use crate::{Player, Y_RESOLUTION};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_camera)
            .add_system(camera_follow_player);
    }
}

fn setup_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = bevy::render::camera::ScalingMode::FixedVertical(1.0);
    camera.projection.scale = Y_RESOLUTION / 2.0;
    commands.spawn_bundle(camera);
}

fn camera_follow_player(
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let mut camera = camera_query.single_mut();
    let player = player_query.single();
    camera.translation.x = player.translation.x;
    camera.translation.y = player.translation.y;
}
