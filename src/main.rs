use bevy::prelude::*;
use heron::prelude::*;

const X_RESOLUTION: f32 = 320.0;
const Y_RESOLUTION: f32 = 240.0;

#[cfg(feature = "deepgram")]
mod microphone;

#[derive(PhysicsLayer)]
enum Layer {
    Items,
    Player,
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
    .add_plugins(DefaultPlugins)
    .add_plugin(PhysicsPlugin::default())
    .insert_resource(Gravity::from(Vec3::new(0.0, 0.0, 0.0)))
    .add_startup_system(spawn_blueberry_basket)
    .add_startup_system(setup_camera)
    .add_event::<microphone::SugarSaid>()
    .add_system(handle_sugar_said_event);

    #[cfg(feature = "deepgram")]
    app.add_plugin(microphone::MicrophonePlugin);

    app.run();
}

#[derive(Component)]
pub(crate) struct BlueberryBasket;

fn spawn_blueberry_basket(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("blueberry_basket.png"),
            transform: Transform::from_xyz(0.0, 0.0, 1.0).with_scale(Vec3::splat(1.0)),
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
pub(crate) struct SugarBag;

fn spawn_sugar_bag(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("sugar_bag.png"),
            transform: Transform::from_xyz(-30.0, 0.0, 1.0).with_scale(Vec3::splat(1.0)),
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

fn handle_sugar_said_event(
    sugar_said_event: EventReader<microphone::SugarSaid>,
    commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if !sugar_said_event.is_empty() {
        info!("You said sugar!");
        spawn_sugar_bag(commands, asset_server);
    }
    sugar_said_event.clear();
}
