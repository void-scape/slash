#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use avian2d::prelude::{Gravity, PhysicsLayer};
use bevy::prelude::*;
use player::Player;

use crate::bits::coalescence::CoalesceEvent;
#[cfg(feature = "debug")]
use bevy::input::common_conditions::input_toggle_active;

mod bits;
mod enemy;
mod health;
mod player;
mod query;
mod weapon;

pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 720.0;

fn main() {
    let mut app = App::default();

    app.add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (WIDTH as u32, HEIGHT as u32).into(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        bevy_rand::prelude::EntropyPlugin::<bevy_rand::prelude::WyRand>::with_seed(
            69u64.to_le_bytes(),
        ),
        #[cfg(feature = "debug")]
        (
            bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
            bevy_inspector_egui::quick::WorldInspectorPlugin::default()
                .run_if(input_toggle_active(false, KeyCode::KeyI)),
        ),
    ))
    .add_plugins((
        avian2d::PhysicsPlugins::default().with_length_unit(2.0),
        #[cfg(feature = "debug")]
        avian2d::debug_render::PhysicsDebugPlugin,
        bevy_enhanced_input::EnhancedInputPlugin,
        player::PlayerPlugin,
        enemy::EnemyPlugin,
        bits::BitsPlugin,
        health::plugin,
        weapon::plugin,
    ))
    .insert_resource(Gravity(Vec2::ZERO));

    // #[cfg(not(feature = "debug"))]
    app.set_error_handler(bevy::ecs::error::warn);

    #[cfg(debug_assertions)]
    app.add_systems(Update, close_on_escape);

    app.add_systems(Startup, (camera, spawn_scene)).run();
}

#[derive(Default, PhysicsLayer)]
pub enum Layer {
    #[default]
    Empty,
    Wall,
    FriendlyHurtboxEnemyHitbox,
    FriendlyHitboxEnemyHurtbox,
}

#[cfg(debug_assertions)]
fn close_on_escape(input: Res<ButtonInput<KeyCode>>, mut writer: MessageWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        writer.write(AppExit::Success);
    }
}

#[cfg(not(debug_assertions))]
pub fn name(_: impl Into<std::borrow::Cow<'static, str>>) -> () {}
#[cfg(debug_assertions)]
pub fn name(name: impl Into<std::borrow::Cow<'static, str>>) -> Name {
    Name::new(name)
}

fn camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_scene(mut commands: Commands) {
    commands.spawn((
        Player,
        health::Health::new(10.0),
        children![
            (weapon::Pistol, bits::BitProducer(35)),
            (
                health::FriendlyHurtbox,
                avian2d::prelude::Collider::rectangle(15.0, 15.0),
                Transform::default(),
            )
        ],
    ));

    commands
        .spawn(GlobalTransform::from(Transform::from_translation(
            Vec3::new(50.0, 50.0, 0.0),
        )))
        .trigger(CoalesceEvent);
    commands
        .spawn(GlobalTransform::from(Transform::from_translation(
            Vec3::new(-50.0, 50.0, 0.0),
        )))
        .trigger(CoalesceEvent);

    // LEVEL WALLS

    // Bottom
    use avian2d::prelude::*;
    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(0.0, -HEIGHT / 2.0, 0.0),
        Collider::rectangle(WIDTH, 25.0),
        CollisionLayers::new(Layer::Wall, LayerMask::ALL),
        CollisionEventsEnabled,
        Name::new("Wall"),
    ));
    // Left wall
    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(-WIDTH / 2.0, 0.0, 0.0),
        Collider::rectangle(25.0, HEIGHT),
        CollisionLayers::new(Layer::Wall, LayerMask::ALL),
        CollisionEventsEnabled,
        Name::new("Wall"),
    ));
    // Right wall
    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(WIDTH / 2.0, 0.0, 0.0),
        Collider::rectangle(25.0, HEIGHT),
        CollisionLayers::new(Layer::Wall, LayerMask::ALL),
        CollisionEventsEnabled,
        Name::new("Wall"),
    ));
    // Top
    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(0.0, HEIGHT / 2.0, 0.0),
        Collider::rectangle(WIDTH, 25.0),
        CollisionLayers::new(Layer::Wall, LayerMask::ALL),
        CollisionEventsEnabled,
        Name::new("Wall"),
    ));
}
