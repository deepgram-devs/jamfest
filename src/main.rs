use bevy::prelude::*;
use heron::prelude::*;

use std::time::Duration;

const X_RESOLUTION: f32 = 640.0;
const Y_RESOLUTION: f32 = 480.0;
const PLAYER_SPEED: f32 = 20.0;

#[cfg(feature = "deepgram")]
mod microphone;

#[derive(PhysicsLayer)]
enum Layer {
    Items,
    Player,
}

#[derive(Default)]
struct JamStartTimer(Timer);

#[derive(Default)]
struct GameState {
    jam_puzzle_completed: bool,
}

fn setup_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = bevy::render::camera::ScalingMode::FixedVertical(1.0);
    camera.projection.scale = Y_RESOLUTION / 2.0;
    commands.spawn_bundle(camera);
}

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "Jamfest".to_string(),
        width: X_RESOLUTION,
        height: Y_RESOLUTION,
        ..Default::default()
    })
    .insert_resource(bevy::render::texture::ImageSettings::default_nearest())
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .insert_resource(GameState::default())
    .add_plugins(DefaultPlugins)
    .add_plugin(PhysicsPlugin::default())
    .insert_resource(JamStartTimer(Timer::new(Duration::from_secs(2), false)))
    .insert_resource(Gravity::from(Vec3::new(0.0, 0.0, 0.0)))
    .add_startup_system(spawn_player)
    .add_system(keyboard_input)
    .add_startup_system(spawn_blueberry_basket)
    .add_startup_system(spawn_wooden_sign)
    .add_system(jam_puzzle_sign_system)
    .add_startup_system(setup_camera)
    .add_event::<microphone::SugarSaid>()
    .add_system(handle_sugar_said_event)
    .add_system(cook_jam);

    #[cfg(feature = "deepgram")]
    app.add_plugin(microphone::MicrophonePlugin);

    app.run();
}

#[derive(Component)]
pub(crate) struct Player;

// NOTE: we are using a Dynamic body because it... works
// but normally one would "move and slide" a Kinematic body in other engines...
fn spawn_player(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 1.0, 1.0),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -40.0, 1.0).with_scale(Vec3::splat(16.0)),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 8.0, 1.0),
            border_radius: None,
        })
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(Acceleration::from_linear(Vec3::ZERO))
        .insert(PhysicMaterial {
            friction: 1.0,
            density: 10.0,
            ..Default::default()
        })
        .insert(RotationConstraints::lock())
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Player)
                .with_mask(Layer::Items),
        )
        .insert(Player);
}

fn keyboard_input(keys: Res<Input<KeyCode>>, mut query: Query<&mut Velocity, With<Player>>) {
    if keys.pressed(KeyCode::W) {
        for mut velocity in query.iter_mut() {
            velocity.linear.y = PLAYER_SPEED;
        }
    } else if keys.pressed(KeyCode::S) {
        for mut velocity in query.iter_mut() {
            velocity.linear.y = -PLAYER_SPEED;
        }
    } else {
        for mut velocity in query.iter_mut() {
            velocity.linear.y = 0.0;
        }
    }
    if keys.pressed(KeyCode::A) {
        for mut velocity in query.iter_mut() {
            velocity.linear.x = -PLAYER_SPEED;
        }
    } else if keys.pressed(KeyCode::D) {
        for mut velocity in query.iter_mut() {
            velocity.linear.x = PLAYER_SPEED;
        }
    } else {
        for mut velocity in query.iter_mut() {
            velocity.linear.x = 0.0;
        }
    }
}

#[derive(Component)]
pub(crate) struct BlueberryBasket;

fn spawn_blueberry_basket(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("blueberry_basket.png"),
            transform: Transform::from_xyz(20.0, 30.0, 1.0).with_scale(Vec3::splat(1.0)),
            ..default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 8.0, 1.0),
            border_radius: None,
        })
        .insert(PhysicMaterial {
            friction: 1.0,
            density: 10.0,
            ..Default::default()
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(BlueberryBasket);
}

#[derive(Component)]
pub(crate) struct WoodenSign;

fn spawn_wooden_sign(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("wooden_sign.png"),
            transform: Transform::from_xyz(0.0, -10.0, 1.0).with_scale(Vec3::splat(1.0)),
            ..default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(3.0, 12.0, 1.0),
            border_radius: None,
        })
        .insert(PhysicMaterial {
            friction: 1.0,
            density: 10.0,
            ..Default::default()
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(WoodenSign);
}

#[derive(Component)]
pub(crate) struct SugarBag;

fn spawn_sugar_bag(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("sugar_bag.png"),
            transform: Transform::from_xyz(-20.0, 30.0, 1.0).with_scale(Vec3::splat(1.0)),
            ..default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 8.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(SugarBag);
}

#[derive(Component)]
pub(crate) struct JamJar;

fn spawn_jam_jar(commands: &mut Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("jam_jar.png"),
            transform: Transform::from_xyz(0.0, 30.0, 1.0).with_scale(Vec3::splat(1.0)),
            ..default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 8.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(JamJar);
}

fn handle_sugar_said_event(
    sugar_said_event: EventReader<microphone::SugarSaid>,
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
) {
    if !sugar_said_event.is_empty() && !game_state.jam_puzzle_completed {
        info!("You said sugar!");
        spawn_sugar_bag(commands, asset_server);
        game_state.jam_puzzle_completed = true;
    }
    sugar_said_event.clear();
}

fn cook_jam(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_state: Res<GameState>,
    mut jam_start_time: ResMut<JamStartTimer>,
    time: Res<Time>,
    sugar_bag_query: Query<Entity, With<SugarBag>>,
    blueberry_basket_query: Query<Entity, With<BlueberryBasket>>,
) {
    if game_state.jam_puzzle_completed {
        jam_start_time.0.tick(time.delta());
        if jam_start_time.0.just_finished() {
            spawn_jam_jar(&mut commands, asset_server);
            despawn_sugar_bag(&mut commands, sugar_bag_query);
            despawn_blueberry_basket(&mut commands, blueberry_basket_query);
            info!("Spawned jam jar");
        }
    }
}

fn despawn_sugar_bag(commands: &mut Commands, sugar_bag_query: Query<Entity, With<SugarBag>>) {
    let sugar_bag_entity = sugar_bag_query.single();
    commands.entity(sugar_bag_entity).despawn_recursive();
}

fn despawn_blueberry_basket(
    commands: &mut Commands,
    blueberry_basket_query: Query<Entity, With<BlueberryBasket>>,
) {
    let blueberry_basket_entity = blueberry_basket_query.single();
    commands.entity(blueberry_basket_entity).despawn_recursive();
}

fn jam_puzzle_sign_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    wooden_sign_query: Query<&Transform, With<WoodenSign>>,
    jam_puzzle_text_query: Query<Entity, With<JamPuzzleText>>,
    asset_server: Res<AssetServer>,
) {
    let player_transform = player_query.single();

    let wooden_sign_transform = wooden_sign_query.single();

    if jam_puzzle_text_query.is_empty()
        && player_transform
            .translation
            .distance(wooden_sign_transform.translation)
            < 40.0
    {
        spawn_jam_puzzle_text(&mut commands, asset_server);
    }
    if !jam_puzzle_text_query.is_empty()
        && player_transform
            .translation
            .distance(wooden_sign_transform.translation)
            > 40.0
    {
        despawn_jam_puzzle_text(&mut commands, jam_puzzle_text_query);
    }
}

fn despawn_jam_puzzle_text(
    commands: &mut Commands,
    jam_puzzle_text_query: Query<Entity, With<JamPuzzleText>>,
) {
    let jam_puzzle_text_entity = jam_puzzle_text_query.single();
    commands.entity(jam_puzzle_text_entity).despawn_recursive();
}

#[derive(Component)]
pub(crate) struct JamPuzzleText;

fn spawn_jam_puzzle_text(commands: &mut Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(
            TextBundle::from_sections([TextSection::new(
                "Say the word sugar.",
                TextStyle {
                    font: asset_server.load("kongtext.ttf"),
                    font_size: 16.0,
                    color: Color::WHITE,
                },
            )])
            // centering the text in the way we'd like was quite difficult
            // and might be worth more thought
            .with_style(Style {
                align_self: AlignSelf::FlexStart,
                justify_content: JustifyContent::Center,
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    ..default()
                },
                ..default()
            }),
        )
        .insert(JamPuzzleText);
}