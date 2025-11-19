#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::{
    bits::coalescence::CoalesceEvent,
    enemy::Dummy,
    health::MaxHealth,
    player::PlayerHurtbox,
    weapon::{ApplyWeaponDurability, WeaponDurability},
};
use avian2d::prelude::*;
#[cfg(feature = "debug")]
use bevy::input::common_conditions::input_toggle_active;
use bevy::{
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use player::Player;

mod bits;
mod enemy;
mod health;
mod physics;
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
            })
            .set(LogPlugin {
                filter: format!(
                    "avian2d::dynamics::rigid_body::mass_properties=off,{DEFAULT_FILTER}"
                ),
                ..Default::default()
            }),
        bevy_rand::prelude::EntropyPlugin::<bevy_rand::prelude::WyRand>::with_seed(
            69u64.to_le_bytes(),
        ),
        bevy_tween::DefaultTweenPlugins,
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
        physics::plugin,
    ))
    .insert_resource(Gravity(Vec2::ZERO));

    // #[cfg(not(feature = "debug"))]
    app.set_error_handler(bevy::ecs::error::warn);

    #[cfg(debug_assertions)]
    app.add_systems(Update, close_on_escape);

    app.add_systems(Startup, (camera, spawn_training_scene))
        .run();
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

#[allow(unused)]
fn spawn_training_scene(mut commands: Commands) {
    let mut root = commands.spawn((Transform::default(), Visibility::default()));
    root.with_child((Dummy, Transform::default()));
    root.with_child((
        Player,
        Transform::from_xyz(0.0, 80.0, 0.0),
        children![weapon::Dagger],
    ));
    level_walls(root);
}

#[allow(unused)]
fn spawn_scene(mut commands: Commands) {
    let mut root = commands.spawn((Transform::default(), Visibility::default()));
    root.with_child((
        Player,
        ApplyWeaponDurability,
        Transform::from_xyz(0.0, -30.0, 0.0),
        MaxHealth(10.0),
        children![
            (weapon::Dagger, WeaponDurability::Hit(3)),
            (
                PlayerHurtbox,
                health::FriendlyHurtbox,
                avian2d::prelude::Collider::rectangle(15.0, 15.0),
                Transform::default(),
            )
        ],
    ));

    root.with_child(GlobalTransform::from(Transform::from_translation(
        Vec3::new(100.0, 100.0, 0.0),
    )))
    .trigger(CoalesceEvent);
    root.with_child(GlobalTransform::from(Transform::from_translation(
        Vec3::new(-100.0, 100.0, 0.0),
    )))
    .trigger(CoalesceEvent);
    root.with_child(GlobalTransform::from(Transform::from_translation(
        Vec3::new(0.0, 0.0, 0.0),
    )))
    .trigger(CoalesceEvent);

    level_walls(root);
}

fn level_walls(mut commands: EntityCommands) {
    // Bottom
    commands.with_child((
        RigidBody::Static,
        Transform::from_xyz(0.0, -HEIGHT / 2.0, 0.0),
        Collider::rectangle(WIDTH, 25.0),
        CollisionLayers::new(Layer::Wall, LayerMask::ALL),
        CollisionEventsEnabled,
        Name::new("Wall"),
    ));
    // Left wall
    commands.with_child((
        RigidBody::Static,
        Transform::from_xyz(-WIDTH / 2.0, 0.0, 0.0),
        Collider::rectangle(25.0, HEIGHT),
        CollisionLayers::new(Layer::Wall, LayerMask::ALL),
        CollisionEventsEnabled,
        Name::new("Wall"),
    ));
    // Right wall
    commands.with_child((
        RigidBody::Static,
        Transform::from_xyz(WIDTH / 2.0, 0.0, 0.0),
        Collider::rectangle(25.0, HEIGHT),
        CollisionLayers::new(Layer::Wall, LayerMask::ALL),
        CollisionEventsEnabled,
        Name::new("Wall"),
    ));
    // Top
    commands.with_child((
        RigidBody::Static,
        Transform::from_xyz(0.0, HEIGHT / 2.0, 0.0),
        Collider::rectangle(WIDTH, 25.0),
        CollisionLayers::new(Layer::Wall, LayerMask::ALL),
        CollisionEventsEnabled,
        Name::new("Wall"),
    ));
}
