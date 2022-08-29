use bevy::prelude::*;
use heron::prelude::*;

const X_RESOLUTION: f32 = 640.0;
const Y_RESOLUTION: f32 = 480.0;
const PLAYER_SPEED: f32 = 100.0;

// z-values
const Z_WOODEN_SIGN: f32 = 4.0;
const Z_PLAYER: f32 = 3.0;
const Z_BLUEBERRY_BASKET: f32 = 3.0;
const Z_SUGAR_BAG: f32 = 3.0;
const Z_WOODEN_PLANKS: f32 = 3.0;
const Z_BEAR: f32 = 4.0;
const Z_JAM_JAR: f32 = 5.0;
const Z_BRIDGE: f32 = 2.0;
const Z_LAVA_TILES: f32 = 1.0;

// coordinates of objects in the jam room
const WOODEN_SIGN_JAM_X: f32 = -400.0;
const WOODEN_SIGN_JAM_Y: f32 = -10.0;
const BLUEBERRY_BASKET_X: f32 = -400.0;
const BLUEBERRY_BASKET_Y: f32 = 30.0;
const SUGAR_BAG_INITIAL_X: f32 = -400.0;
const SUGAR_BAG_INITIAL_Y: f32 = 70.0;
const SUGAR_BAG_SPEED: f32 = -15.0;
const JAM_JAR_X: f32 = -400.0;
const JAM_JAR_Y: f32 = 30.0;
const BEAR_X: f32 = -360.0;
const BEAR_Y: f32 = 80.0;
const WOODEN_PLANKS_X: f32 = -360.0;
const WOODEN_PLANKS_Y: f32 = 70.0;

// coordinates of objects in the mentos room
const WOODEN_SIGN_MENTOS_X: f32 = 500.0;
const WOODEN_SIGN_MENTOS_Y: f32 = 70.0;
const COLA_X: f32 = 400.0;
const COLA_Y: f32 = 70.0;
const MENTOS_INITIAL_X: f32 = 400.0;
const MENTOS_INITIAL_Y: f32 = 140.0;
const MENTOS_SPEED: f32 = -15.0;
const ROPE_INITIAL_X: f32 = 425.0;
const ROPE_INITIAL_Y: f32 = 174.0;
const ROPE_FINAL_X: f32 = 425.0;
const ROPE_FINAL_Y: f32 = 130.0;
const ROPE_DROP_SPEED: f32 = -60.0;
const BULLSEYE_X: f32 = 400.0;
const BULLSEYE_Y: f32 = 174.0;

// coordinates of objects in the bridge room
const WOODEN_SIGN_BRIDGE_X: f32 = 0.0;
const WOODEN_SIGN_BRIDGE_Y: f32 = -10.0;
const WOODEN_BRIDGE_X: f32 = 0.0;
const WOODEN_BRIDGE_Y: f32 = 72.0;
const TREASURE_CHEST_X: f32 = 0.0;
const TREASURE_CHEST_Y: f32 = 120.0;

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
    Tiles,
    Npc,
    Items,
    Player,
}

#[derive(Default)]
struct GameState {
    mentos_puzzle_completed: bool,
    sugar_puzzle_completed: bool,
    jam_created: bool,
    bridge_puzzle_completed: bool,
    wooden_planks_collected: bool,
    rope_coil_collected: bool,
    bullseye_just_hit: bool,
    treasure_chest_opened: bool,
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
    .insert_resource(Gravity::from(Vec3::new(0.0, 0.0, 0.0)))
    .add_startup_system(spawn_wall_tiles)
    .add_startup_system(spawn_lava_tiles)
    .add_startup_system(spawn_player)
    .add_system(keyboard_input)
    .add_system(camera_follow_player)
    .add_startup_system(spawn_blueberry_basket)
    .add_startup_system(spawn_wooden_planks)
    .add_startup_system(spawn_bear)
    .add_startup_system(spawn_soda)
    .add_startup_system(spawn_rope_coil)
    .add_startup_system(spawn_bullseye)
    .add_startup_system(spawn_treasure_chest)
    .add_startup_system(spawn_wooden_signs)
    .add_system(puzzle_sign_system)
    .add_startup_system(setup_camera)
    .add_event::<SpeechEvent>()
    .add_system(handle_sugar_said_event)
    .add_system(handle_mentos_said_event)
    .add_system(handle_bridge_said_event)
    .add_system(handle_rope_coil_collected_event)
    .add_system(explode_mentos)
    .add_system(drop_rope)
    .add_system(handle_wooden_planks_collected_event)
    .add_system(move_bear_to_jam_jar)
    .add_system(drop_sugar)
    .add_system(check_treasure_chest_proximity);

    #[cfg(feature = "deepgram")]
    app.add_plugin(microphone::MicrophonePlugin);

    app.run();
}

#[derive(Component)]
pub(crate) struct Player;

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
            transform: Transform::from_xyz(
                BLUEBERRY_BASKET_X,
                BLUEBERRY_BASKET_Y,
                Z_BLUEBERRY_BASKET,
            ),
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
            transform: Transform::from_xyz(WOODEN_PLANKS_X, WOODEN_PLANKS_Y, Z_WOODEN_PLANKS),
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
            transform: Transform::from_xyz(ROPE_INITIAL_X, ROPE_INITIAL_Y, 1.0),
            ..default()
        })
        .insert(RigidBody::KinematicVelocityBased)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 16.0, 1.0),
            border_radius: None,
        })
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(RopeCoil);
}

#[derive(Component)]
pub(crate) struct Bullseye;

fn spawn_bullseye(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("bullseye.png"),
            transform: Transform::from_xyz(BULLSEYE_X, BULLSEYE_Y, 2.0),
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
                .with_mask(Layer::Items),
        )
        .insert(Bullseye);
}

#[derive(Component)]
pub(crate) struct TreasureChest;

fn spawn_treasure_chest(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("treasure_chest_closed.png"),
            transform: Transform::from_xyz(TREASURE_CHEST_X, TREASURE_CHEST_Y, 1.0),
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
        .insert(TreasureChest);
}

fn spawn_opened_treasure_chest(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("treasure_chest_opened.png"),
            transform: Transform::from_xyz(TREASURE_CHEST_X, TREASURE_CHEST_Y, 1.0),
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
        .insert(TreasureChest);
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
            half_extends: Vec3::new(8.0, 20.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(Soda);
}

fn despawn_soda(
    commands: &mut Commands,
    soda_bottle_query: Query<(Entity, &Transform), With<Soda>>,
) {
    let (soda_bottle_entity, _soda_bottle_transform) = soda_bottle_query.single();
    commands.entity(soda_bottle_entity).despawn_recursive();
}

#[derive(Component)]
pub(crate) struct EmptySoda;

fn spawn_empty_soda(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("soda_bottle_empty.png"),
            transform: Transform::from_xyz(COLA_X, COLA_Y, 1.0),
            ..default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 20.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(EmptySoda);
}

#[derive(Component)]
pub(crate) struct Mentos;

fn spawn_mentos(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning mentos.");
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("mint_package.png"),
            transform: Transform::from_xyz(MENTOS_INITIAL_X, MENTOS_INITIAL_Y, 1.0),
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
        Transform::from_xyz(WOODEN_SIGN_JAM_X, WOODEN_SIGN_JAM_Y, Z_WOODEN_SIGN),
        "The bear is hungry and would like some jam. Here are some blueberries, what else do you need to make jam?",
    );

    spawn_wooden_sign(
        &mut commands,
        &asset_server,
        Transform::from_xyz(WOODEN_SIGN_MENTOS_X, WOODEN_SIGN_MENTOS_Y, Z_WOODEN_SIGN),
        "Pop the bottle cap to hit the bullseye. What could you mix with the soda to do this?",
    );

    spawn_wooden_sign(
        &mut commands,
        &asset_server,
        Transform::from_xyz(WOODEN_SIGN_BRIDGE_X, WOODEN_SIGN_BRIDGE_Y, Z_WOODEN_SIGN),
        "To get the treasure, find some rope and some wood. Then tell me what you can make with them to cross the lava.",
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
            half_extends: Vec3::new(3.0, 16.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Items)
                .with_mask(Layer::Player),
        )
        .insert(WoodenSign(text));
}

fn spawn_wall_tiles(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut coordinates = Vec::new();
    // bridge room
    for x in -10..=10 {
        for y in 10..=11 {
            coordinates.push((x, y));
        }
        for y in -9..=-8 {
            coordinates.push((x, y));
        }
    }

    for x in -10..=-9 {
        for y in -9..=-3 {
            coordinates.push((x, y));
        }
        for y in 3..=11 {
            coordinates.push((x, y));
        }
    }

    for x in 9..=10 {
        for y in -9..=-3 {
            coordinates.push((x, y));
        }
        for y in 3..=11 {
            coordinates.push((x, y));
        }
    }

    // jam room
    for x in -10 - 25..=10 - 25 {
        for y in 10..=11 {
            coordinates.push((x, y));
        }
        for y in -9..=-8 {
            coordinates.push((x, y));
        }
    }

    for x in -10 - 25..=-9 - 25 {
        for y in -9..=11 {
            coordinates.push((x, y));
        }
    }

    for x in 9 - 25..=10 - 25 {
        for y in -9..=-3 {
            coordinates.push((x, y));
        }
        for y in 3..=11 {
            coordinates.push((x, y));
        }
    }

    // mentos room
    for x in -10 + 25..=10 + 25 {
        for y in 10..=11 {
            coordinates.push((x, y));
        }
        for y in -9..=-8 {
            coordinates.push((x, y));
        }
    }

    for x in -10 + 25..=-9 + 25 {
        for y in -9..=-3 {
            coordinates.push((x, y));
        }
        for y in 3..=11 {
            coordinates.push((x, y));
        }
    }

    for x in 9 + 25..=10 + 25 {
        for y in -9..=11 {
            coordinates.push((x, y));
        }
    }

    // hallways
    for x in -15..=-10 {
        for y in 3..=4 {
            coordinates.push((x, y));
        }
        for y in -4..=-3 {
            coordinates.push((x, y));
        }
    }
    for x in -15..=-10 {
        for y in 3..=4 {
            coordinates.push((x, y));
        }
        for y in -4..=-3 {
            coordinates.push((x, y));
        }
    }

    for x in 10..=15 {
        for y in 3..=4 {
            coordinates.push((x, y));
        }
        for y in -4..=-3 {
            coordinates.push((x, y));
        }
    }
    for x in 10..=15 {
        for y in 3..=4 {
            coordinates.push((x, y));
        }
        for y in -4..=-3 {
            coordinates.push((x, y));
        }
    }

    for coordinate in coordinates {
        spawn_wall_tile(
            &mut commands,
            &asset_server,
            Transform::from_xyz(
                coordinate.0 as f32 * 16.0,
                coordinate.1 as f32 * 16.0,
                Z_LAVA_TILES,
            ),
        );
    }
}

fn spawn_wall_tile(commands: &mut Commands, asset_server: &Res<AssetServer>, position: Transform) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("wall_tile.png"),
            transform: position,
            ..default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 8.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Tiles)
                .with_mask(Layer::Player),
        );
}

// We will use this as a hack to keep track of which lava tiles we will replace with
// non-collidable lava tiles. This is how we will simulate walking over the bridge
// when it appears. There is certainly a better way to do this.
#[derive(Component)]
pub(crate) struct LavaTileTracked;

fn spawn_lava_tiles(mut commands: Commands, asset_server: Res<AssetServer>) {
    // non-tracked tiles
    let mut coordinates = Vec::new();
    for x in -8..=-2 {
        for y in 4..=5 {
            coordinates.push((x, y));
        }
    }

    for x in 2..=8 {
        for y in 4..=5 {
            coordinates.push((x, y));
        }
    }

    for coordinate in coordinates {
        spawn_lava_tile(
            &mut commands,
            &asset_server,
            Transform::from_xyz(
                coordinate.0 as f32 * 16.0,
                coordinate.1 as f32 * 16.0,
                Z_LAVA_TILES,
            ),
        );
    }

    // tracked tiles
    let mut coordinates = Vec::new();
    for x in -1..=1 {
        for y in 4..=5 {
            coordinates.push((x, y));
        }
    }

    for coordinate in coordinates {
        spawn_lava_tile_tracked(
            &mut commands,
            &asset_server,
            Transform::from_xyz(
                coordinate.0 as f32 * 16.0,
                coordinate.1 as f32 * 16.0,
                Z_LAVA_TILES,
            ),
        );
    }
}

fn spawn_lava_tile(commands: &mut Commands, asset_server: &Res<AssetServer>, position: Transform) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("lava_tile.png"),
            transform: position,
            ..default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 8.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Tiles)
                .with_mask(Layer::Player),
        );
}

fn spawn_lava_tile_tracked(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Transform,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("lava_tile.png"),
            transform: position,
            ..default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, 8.0, 1.0),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layer::Tiles)
                .with_mask(Layer::Player),
        )
        .insert(LavaTileTracked);
}

fn spawn_lava_tiles_non_collidable(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let mut coordinates = Vec::new();
    for x in -1..=1 {
        for y in 4..=5 {
            coordinates.push((x, y));
        }
    }

    for coordinate in coordinates {
        spawn_lava_tile_non_collidable(
            commands,
            &asset_server,
            Transform::from_xyz(
                coordinate.0 as f32 * 16.0,
                coordinate.1 as f32 * 16.0,
                Z_LAVA_TILES,
            ),
        );
    }
}

fn spawn_lava_tile_non_collidable(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Transform,
) {
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("lava_tile.png"),
        transform: position,
        ..default()
    });
}

#[derive(Component)]
pub(crate) struct SugarBag;

fn spawn_sugar_bag(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("sugar_bag.png"),
            transform: Transform::from_xyz(SUGAR_BAG_INITIAL_X, SUGAR_BAG_INITIAL_Y, Z_SUGAR_BAG),
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
        .insert(SugarBag);
}

#[derive(Component)]
pub(crate) struct JamJar;

fn spawn_jam_jar(commands: &mut Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("jam_jar.png"),
            transform: Transform::from_xyz(JAM_JAR_X, JAM_JAR_Y, Z_JAM_JAR),
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
pub(crate) struct WoodenBridge;

fn spawn_wooden_bridge(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("wooden_bridge.png"),
            transform: Transform::from_xyz(WOODEN_BRIDGE_X, WOODEN_BRIDGE_Y, Z_BRIDGE),
            ..default()
        })
        .insert(WoodenBridge);
}

#[derive(Component)]
pub(crate) struct Bear;

fn spawn_bear(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("bear_npc_1.png"),
            transform: Transform::from_xyz(BEAR_X, BEAR_Y, Z_BEAR),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(12.0, 16.0, 1.0),
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

fn handle_bridge_said_event(
    mut speech_events: EventReader<SpeechEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
    player_query: Query<&Transform, With<Player>>,
    treasure_chest_query: Query<&Transform, With<TreasureChest>>,
    lava_tile_tracked_query: Query<Entity, With<LavaTileTracked>>,
) {
    if game_state.bridge_puzzle_completed
        || !game_state.rope_coil_collected
        || !game_state.wooden_planks_collected
    {
        return;
    }

    let player_transform = player_query.single();
    let treasure_chest_transform = treasure_chest_query.single();

    if player_transform
        .translation
        .distance(treasure_chest_transform.translation)
        < 200.0
    {
        let bridge_said = speech_events
            .iter()
            .any(|event| *event == SpeechEvent::Bridge);
        if bridge_said {
            info!("You said bridge!");
            spawn_wooden_bridge(&mut commands, &asset_server);
            despawn_lava_tiles_tracked(&mut commands, lava_tile_tracked_query);
            spawn_lava_tiles_non_collidable(&mut commands, &asset_server);
            game_state.bridge_puzzle_completed = true;
        }
        speech_events.clear();
    }
}

fn despawn_lava_tiles_tracked(
    commands: &mut Commands,
    lava_tile_tracked_query: Query<Entity, With<LavaTileTracked>>,
) {
    for lava_tile in lava_tile_tracked_query.iter() {
        info!("Despawning a tracked lava tile!");
        commands.entity(lava_tile).despawn_recursive();
    }
}

fn check_treasure_chest_proximity(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    treasure_chest_query: Query<(Entity, &Transform), With<TreasureChest>>,
    mut game_state: ResMut<GameState>,
) {
    if game_state.treasure_chest_opened {
        return;
    }

    let player_transform = player_query.single();
    let (_, treasure_chest_transform) = treasure_chest_query.single();

    if player_transform
        .translation
        .distance(treasure_chest_transform.translation)
        < 40.0
    {
        despawn_treasure_chest(&mut commands, treasure_chest_query);
        spawn_win_text(&mut commands, &asset_server);
        spawn_opened_treasure_chest(commands, asset_server);
        game_state.treasure_chest_opened = true;
    }
}

fn despawn_treasure_chest(
    commands: &mut Commands,
    treasure_chest_query: Query<(Entity, &Transform), With<TreasureChest>>,
) {
    let (treasure_chest_entity, _) = treasure_chest_query.single();
    commands.entity(treasure_chest_entity).despawn_recursive();
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
        < 200.0
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
        < 200.0
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
    mut game_state: ResMut<GameState>,
    mut mentos_query: Query<(Entity, &Transform, &mut Velocity), With<Mentos>>,
    soda_query: Query<(Entity, &Transform), With<Soda>>,
) {
    if let Ok((mentos, mentos_transform, mut mentos_velocity)) = mentos_query.get_single_mut() {
        let (_soda_entity, soda_transform) = soda_query.single();
        let difference = mentos_transform.translation - soda_transform.translation;
        let distance = difference.length();
        if distance < 20.0 {
            commands.entity(mentos).despawn_recursive();
            info!("Setting bullseye_just_hit to true.");
            game_state.bullseye_just_hit = true;
            despawn_soda(&mut commands, soda_query);
            spawn_empty_soda(commands, asset_server);
        } else {
            let new_velocity = difference.normalize() * MENTOS_SPEED;
            *mentos_velocity = Velocity::from_linear(new_velocity);
        }
    }
}

fn drop_rope(
    mut game_state: ResMut<GameState>,
    mut rope_coil_query: Query<(&Transform, &mut Velocity), With<RopeCoil>>,
) {
    if !game_state.bullseye_just_hit {
        return;
    }

    if let Ok((rope_coil_transform, mut rope_coil_velocity)) = rope_coil_query.get_single_mut() {
        let difference =
            rope_coil_transform.translation - Vec3::new(ROPE_FINAL_X, ROPE_FINAL_Y, 1.0);
        let distance = difference.length();
        if distance < 5.0 {
            *rope_coil_velocity = Velocity::from_linear(Vec3::ZERO);
            game_state.bullseye_just_hit = false;
        } else {
            let new_velocity = difference.normalize() * ROPE_DROP_SPEED;
            *rope_coil_velocity = Velocity::from_linear(new_velocity);
        }
    }
}

fn drop_sugar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
    mut sugar_bag_query: Query<(Entity, &Transform, &mut Velocity), With<SugarBag>>,
    mut blueberry_basket_query: Query<(Entity, &Transform), With<BlueberryBasket>>,
) {
    if !game_state.sugar_puzzle_completed || game_state.jam_created {
        return;
    }

    if let Ok((_, sugar_bag_transform, mut sugar_bag_velocity)) = sugar_bag_query.get_single_mut() {
        if let Ok((_, blueberry_basket_transform)) = blueberry_basket_query.get_single_mut() {
            let difference =
                sugar_bag_transform.translation - blueberry_basket_transform.translation;
            let distance = difference.length();
            if distance < 5.0 {
                despawn_sugar_bag(&mut commands, sugar_bag_query);
                despawn_blueberry_basket(&mut commands, blueberry_basket_query);
                spawn_jam_jar(&mut commands, asset_server);
                game_state.jam_created = true;
            } else {
                let new_velocity = difference.normalize() * SUGAR_BAG_SPEED;
                *sugar_bag_velocity = Velocity::from_linear(new_velocity);
            }
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

fn despawn_sugar_bag(
    commands: &mut Commands,
    sugar_bag_query: Query<(Entity, &Transform, &mut Velocity), With<SugarBag>>,
) {
    let (sugar_bag_entity, _, _) = sugar_bag_query.single();
    commands.entity(sugar_bag_entity).despawn_recursive();
}

fn despawn_blueberry_basket(
    commands: &mut Commands,
    blueberry_basket_query: Query<(Entity, &Transform), With<BlueberryBasket>>,
) {
    let (blueberry_basket_entity, _) = blueberry_basket_query.single();
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

fn spawn_win_text(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.spawn_bundle(
        TextBundle::from_sections([TextSection::new(
            "YOU WIN!",
            TextStyle {
                font: asset_server.load("kongtext.ttf"),
                font_size: 32.0,
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
                top: Val::Auto,
                bottom: Val::Auto,
            },

            ..default()
        }),
    );
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
