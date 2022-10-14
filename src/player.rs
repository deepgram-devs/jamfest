use bevy::prelude::*;
use heron::prelude::*;

use crate::{Layer, Z_PLAYER};

const PLAYER_SPEED: f32 = 100.0;

#[derive(Component)]
pub(crate) struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(camera_follow_player)
            .add_system(keyboard_input);
    }
}

// NOTE: we are using a Dynamic body because it... works
// but normally one would "move and slide" a Kinematic body in other engines...
fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("bear_player_1.png"),
            transform: Transform::from_xyz(0.0, -60.0, Z_PLAYER),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(12.0, 16.0, 1.0),
            border_radius: None,
        })
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(RotationConstraints::lock())
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Player)
                .with_mask(Layer::Items)
                .with_mask(Layer::Npc)
                .with_mask(Layer::Tiles),
        )
        .insert(Player);
}

fn keyboard_input(keys: Res<Input<KeyCode>>, mut query: Query<&mut Velocity, With<Player>>) {
    let mut velocity = query.single_mut();
    if keys.pressed(KeyCode::W) {
        velocity.linear.y = PLAYER_SPEED;
    } else if keys.pressed(KeyCode::S) {
        velocity.linear.y = -PLAYER_SPEED;
    } else {
        velocity.linear.y = 0.0;
    }
    if keys.pressed(KeyCode::A) {
        velocity.linear.x = -PLAYER_SPEED;
    } else if keys.pressed(KeyCode::D) {
        velocity.linear.x = PLAYER_SPEED;
    } else {
        velocity.linear.x = 0.0;
    }
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
