use bevy::prelude::*;
use heron::prelude::*;

use std::time::Duration;

const X_RESOLUTION: f32 = 640.0;
const Y_RESOLUTION: f32 = 480.0;
const PLAYER_SPEED: f32 = 100.0;

// coordinates of objects in the jam room
const WOODEN_SIGN_JAM_X: f32 = -200.0;
const WOODEN_SIGN_JAM_Y: f32 = -10.0;
const BLUEBERRY_BASKET_X: f32 = -180.0;
const BLUEBERRY_BASKET_Y: f32 = 20.0;
const SUGAR_BAG_X: f32 = -220.0;
const SUGAR_BAG_Y: f32 = 20.0;
const JAM_JAR_X: f32 = -200.0;
const JAM_JAR_Y: f32 = 20.0;
const BEAR_X: f32 = -170.0;
const BEAR_Y: f32 = 50.0;
const WOODEN_PLANKS_X: f32 = -170.0;
const WOODEN_PLANKS_Y: f32 = 50.0;

// coordinates of objects in the mentos room
const WOODEN_SIGN_MENTOS_X: f32 = 200.0;
const WOODEN_SIGN_MENTOS_Y: f32 = -10.0;
// BULLSEYE_X
// BULLSEYE_Y
const COLA_X: f32 = 200.0;
const COLA_Y: f32 = 50.0;
const MENTOS_INITIAL_X: f32 = 200.0;
const MENTOS_INITIAL_Y: f32 = 150.0;
const MENTOS_SPEED: f32 = -20.0;
const ROPE_X: f32 = 250.0;
const ROPE_Y: f32 = 50.0;

// coordinates of objects in the bridge room
const WOODEN_SIGN_BRIDGE_X: f32 = 0.0;
const WOODEN_SIGN_BRIDGE_Y: f32 = -10.0;
// BRIDGE_X
// BRIDGE_Y
// GOAL_X
// GOAL_Y

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpeechEvent {
    Bridge,
    Mentos,
    Sugar,
}
#[cfg(feature = "deepgram")]
mod microphone;

#[derive(PhysicsLayer)]
enum Layer {
    Npc,
    Items,
    Player,
}

#[derive(Default)]
struct JamStartTimer(Timer);

#[derive(Default)]
struct GameState {
    mentos_puzzle_completed: bool,
    sugar_puzzle_completed: bool,
    wooden_planks_collected: bool,
    rope_coil_collected: bool,
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
    .add_system(camera_follow_player)
    .add_startup_system(spawn_blueberry_basket)
    .add_startup_system(spawn_wooden_planks)
    .add_startup_system(spawn_bear)
    .add_startup_system(spawn_soda)
    .add_startup_system(spawn_wooden_signs)
    .add_system(puzzle_sign_system)
    .add_startup_system(setup_camera)
    .add_event::<SpeechEvent>()
    .add_system(handle_sugar_said_event)
    .add_system(handle_mentos_said_event)
    .add_system(handle_rope_coil_collected_event)
    .add_system(explode_mentos)
    .add_system(handle_wooden_planks_collected_event)
    .add_system(move_bear_to_jam_jar)
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
        .insert(RotationConstraints::lock())
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Player)
                .with_mask(Layer::Items)
                .with_mask(Layer::Npc),
        )
        .insert(Player);
}

fn keyboard_input(
    keys: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
    mut speech_events: EventWriter<SpeechEvent>,
) {
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

    if keys.just_pressed(KeyCode::J) {
        info!("Sending sugar speech event triggered by key press");
        speech_events.send(SpeechEvent::Sugar);
    } else if keys.just_pressed(KeyCode::B) {
        info!("Sending bridge speech event triggered by key press");
        speech_events.send(SpeechEvent::Bridge);
    } else if keys.just_pressed(KeyCode::M) {
        info!("Sending mentos speech event triggered by key press");
        speech_events.send(SpeechEvent::Mentos);
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

#[derive(Component)]
pub(crate) struct BlueberryBasket;

fn spawn_blueberry_basket(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("blueberry_basket.png"),
            transform: Transform::from_xyz(BLUEBERRY_BASKET_X, BLUEBERRY_BASKET_Y, 1.0),
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
        .insert(BlueberryBasket);
}

#[derive(Component)]
pub(crate) struct WoodenPlanks;

fn spawn_wooden_planks(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("wooden_planks.png"),
            transform: Transform::from_xyz(WOODEN_PLANKS_X, WOODEN_PLANKS_Y, 1.0),
            ..default()
        })
        .insert(RigidBody::Sensor)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 8.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(WoodenPlanks);
}

#[derive(Component)]
pub(crate) struct RopeCoil;

fn spawn_rope_coil(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("rope_coil.png"),
            transform: Transform::from_xyz(ROPE_X, ROPE_Y, 1.0),
            ..default()
        })
        .insert(RigidBody::Sensor)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 16.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(RopeCoil);
}

#[derive(Component)]
pub(crate) struct Soda;

fn spawn_soda(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("soda_bottle.png"),
            transform: Transform::from_xyz(COLA_X, COLA_Y, 1.0),
            ..default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 16.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(Soda);
}

#[derive(Component)]
pub(crate) struct Mentos;

fn spawn_mentos(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.75, 0.75, 0.75),
                ..default()
            },
            // texture: asset_server.load("sugar_bag.png"),
            transform: Transform::from_xyz(MENTOS_INITIAL_X, MENTOS_INITIAL_Y, 1.0)
                .with_scale(Vec3::new(4.0, 16.0, 1.0)),
            ..default()
        })
        .insert(RigidBody::KinematicVelocityBased)
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 8.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(Mentos);
}

#[derive(Component)]
pub(crate) struct WoodenSign(&'static str);

fn spawn_wooden_signs(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_wooden_sign(
        &mut commands,
        &asset_server,
        Transform::from_xyz(WOODEN_SIGN_JAM_X, WOODEN_SIGN_JAM_Y, 1.0),
        "The bear is hungry and would like some jam. Here are some blueberries, what else do you need to make jam?",
    );

    spawn_wooden_sign(
        &mut commands,
        &asset_server,
        Transform::from_xyz(WOODEN_SIGN_MENTOS_X, WOODEN_SIGN_MENTOS_Y, 1.0),
        "Say mentos.",
    );

    spawn_wooden_sign(
        &mut commands,
        &asset_server,
        Transform::from_xyz(WOODEN_SIGN_BRIDGE_X, WOODEN_SIGN_BRIDGE_Y, 1.0),
        "Say bridge.",
    );
}

fn spawn_wooden_sign(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Transform,
    text: &'static str,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("wooden_sign.png"),
            transform: position,
            ..default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(3.0, 12.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(WoodenSign(text));
}

#[derive(Component)]
pub(crate) struct SugarBag;

fn spawn_sugar_bag(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("sugar_bag.png"),
            transform: Transform::from_xyz(SUGAR_BAG_X, SUGAR_BAG_Y, 1.0),
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
            transform: Transform::from_xyz(JAM_JAR_X, JAM_JAR_Y, 1.0),
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

#[derive(Component)]
pub(crate) struct Bear;

fn spawn_bear(mut commands: Commands) {
    commands
        // TODO use an actual bear sprite
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                ..default()
            },
            transform: Transform::from_xyz(BEAR_X, BEAR_Y, 2.0).with_scale(Vec3::splat(8.0)),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 8.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Npc)
                .with_mask(Layer::Player),
        )
        // giving the bear infinite mass so that you can't push it
        .insert(PhysicMaterial {
            friction: 1.0,
            density: f32::MAX,
            ..Default::default()
        })
        .insert(Bear);
}

fn handle_sugar_said_event(
    mut speech_events: EventReader<SpeechEvent>,
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
    player_query: Query<&Transform, With<Player>>,
    blueberry_basket_query: Query<&Transform, With<BlueberryBasket>>,
) {
    if game_state.sugar_puzzle_completed {
        return;
    }

    let player_transform = player_query.single();
    let blueberry_basket_transform = blueberry_basket_query.single();

    if player_transform
        .translation
        .distance(blueberry_basket_transform.translation)
        < 50.0
    {
        let sugar_said = speech_events
            .iter()
            .any(|event| *event == SpeechEvent::Sugar);
        if sugar_said {
            info!("You said sugar!");
            spawn_sugar_bag(commands, asset_server);
            game_state.sugar_puzzle_completed = true;
        }
        speech_events.clear();
    }
}

fn handle_mentos_said_event(
    mut speech_events: EventReader<SpeechEvent>,
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
    player_query: Query<&Transform, With<Player>>,
    soda_query: Query<&Transform, With<Soda>>,
) {
    if game_state.mentos_puzzle_completed {
        return;
    }

    let player_transform = player_query.single();
    let soda_transform = soda_query.single();

    if player_transform
        .translation
        .distance(soda_transform.translation)
        < 50.0
    {
        let mentos_said = speech_events
            .iter()
            .any(|event| *event == SpeechEvent::Mentos);
        if mentos_said {
            info!("You said mentos!");
            spawn_mentos(commands, asset_server);
            game_state.mentos_puzzle_completed = true;
        }
        speech_events.clear();
    }
}

fn explode_mentos(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut mentos_query: Query<(Entity, &Transform, &mut Velocity), With<Mentos>>,
    soda_query: Query<&Transform, With<Soda>>,
) {
    if let Ok((mentos, mentos_transform, mut mentos_velocity)) = mentos_query.get_single_mut() {
        let soda = soda_query.single();
        let difference = mentos_transform.translation - soda.translation;
        let distance = difference.length();
        if distance < 20.0 {
            commands.entity(mentos).despawn_recursive();
            spawn_rope_coil(commands, asset_server)
        } else {
            let new_velocity = difference.normalize() * MENTOS_SPEED;
            *mentos_velocity = Velocity::from_linear(new_velocity);
        }
    }
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
    if game_state.sugar_puzzle_completed {
        jam_start_time.0.tick(time.delta());
        if jam_start_time.0.just_finished() {
            spawn_jam_jar(&mut commands, asset_server);
            despawn_sugar_bag(&mut commands, sugar_bag_query);
            despawn_blueberry_basket(&mut commands, blueberry_basket_query);
            info!("Spawned jam jar");
        }
    }
}

fn move_bear_to_jam_jar(
    jam_jar_query: Query<&Transform, With<JamJar>>,
    mut bear_query: Query<(&Transform, &mut Velocity), With<Bear>>,
) {
    if let Ok(jam_jar) = jam_jar_query.get_single() {
        let (bear_transform, mut bear_velocity) = bear_query.single_mut();
        let difference = jam_jar.translation - bear_transform.translation;
        let distance = difference.length();
        if distance > 20.0 {
            let direction = difference.normalize();
            *bear_velocity = Velocity::from_linear(direction * 10.0);
        } else {
            // Also start bear eating jam animation and allow user to pick up wooden planks.
            // That or make the bear collision box really big or something :shrug:
            // Actually, should allow picking up the wood as soon as the bear has cleared the wood.
            *bear_velocity = Velocity::from_linear(Vec3::ZERO);
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

fn puzzle_sign_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    wooden_sign_query: Query<(&WoodenSign, &Transform)>,
    puzzle_text_query: Query<Entity, With<PuzzleText>>,
    asset_server: Res<AssetServer>,
) {
    let player_transform = player_query.single();

    let mut found_wooden_sign = None;

    for (wooden_sign, wooden_sign_transform) in wooden_sign_query.iter() {
        if player_transform
            .translation
            .distance(wooden_sign_transform.translation)
            < 40.0
        {
            found_wooden_sign = Some(wooden_sign);
        }
    }

    match found_wooden_sign {
        Some(wooden_sign) => {
            if puzzle_text_query.is_empty() {
                spawn_puzzle_text(&mut commands, asset_server, wooden_sign.0);
            }
        }
        None => {
            despawn_puzzle_text(&mut commands, puzzle_text_query);
        }
    }
}

fn despawn_puzzle_text(
    commands: &mut Commands,
    puzzle_text_query: Query<Entity, With<PuzzleText>>,
) {
    for puzzle_text_entity in puzzle_text_query.iter() {
        commands.entity(puzzle_text_entity).despawn_recursive();
    }
}

#[derive(Component)]
pub(crate) struct PuzzleText;

fn spawn_puzzle_text(commands: &mut Commands, asset_server: Res<AssetServer>, text: &'static str) {
    commands
        .spawn_bundle(
            TextBundle::from_sections([TextSection::new(
                text,
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
                max_size: Size {
                    width: Val::Px(X_RESOLUTION),
                    height: Val::Auto,
                },
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    ..default()
                },

                ..default()
            }),
        )
        .insert(PuzzleText);
}

fn handle_rope_coil_collected_event(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut game_state: ResMut<GameState>,
    rope_coil_query: Query<Entity, With<RopeCoil>>,
    player_query: Query<Entity, With<Player>>,
) {
    let player = player_query.single();

    if let Ok(rope_coil) = rope_coil_query.get_single() {
        for event in events.iter() {
            let (e1, e2) = event.rigid_body_entities();
            if (e1 == rope_coil || e2 == rope_coil) && (e1 == player || e2 == player) {
                game_state.rope_coil_collected = true;
                commands.entity(rope_coil).despawn_recursive();
                info!("Collected rope coil!");
                break;
            }
        }
    }
}

fn handle_wooden_planks_collected_event(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut game_state: ResMut<GameState>,
    wooden_planks_query: Query<(Entity, &Transform), With<WoodenPlanks>>,
    player_query: Query<Entity, With<Player>>,
    bear_query: Query<&Transform, With<Bear>>,
) {
    let player_entity = player_query.single();
    let bear_transform = bear_query.single();

    if let Ok((wooden_planks_entity, wooden_planks_transform)) = wooden_planks_query.get_single() {
        // this feels a little bit hacky, but essentially, we don't want
        // the player to be able to pick up the wood if the bear is too close to the wood
        if wooden_planks_transform
            .translation
            .distance(bear_transform.translation)
            < 16.0
        {
            return;
        }

        for event in events.iter() {
            let (e1, e2) = event.rigid_body_entities();
            if (e1 == wooden_planks_entity || e2 == wooden_planks_entity)
                && (e1 == player_entity || e2 == player_entity)
            {
                game_state.wooden_planks_collected = true;
                commands.entity(wooden_planks_entity).despawn_recursive();
                info!("Collected wooden planks!");
                break;
            }
        }
    }
}
