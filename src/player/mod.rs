use crate::{
    Layer,
    physics::Acceleration,
    player::input::{Dashing, RetainedMove},
};
use avian2d::prelude::{
    Collider, CollisionLayers, LinearDamping, LockedAxes, MaxLinearSpeed, RigidBody,
};
use bevy::{
    color::palettes::css::BLUE, input::mouse::MouseMotion, prelude::*, window::PrimaryWindow,
};

pub mod input;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(input::InputPlugin)
            .add_systems(Update, orient_player_with_mouse_input);
    }
}

/// The player marker component.
#[derive(Component)]
#[require(
    Transform,
    RigidBody::Dynamic,
    Sprite::from_color(BLUE, Vec2::new(20.0, 20.0)),
    Name::new("Player"),
    CollisionLayers = Self::collision_layers(),
    Collider::circle(7.5),
    LockedAxes::ROTATION_LOCKED,
    OrientationMethod,
    Acceleration,
    LinearDamping = Self::LINEAR_DAMPING,
    MaxLinearSpeed = Self::MAX_SPEED,
    RetainedMove,
)]
pub struct Player;

impl Player {
    pub const LINEAR_DAMPING: LinearDamping = LinearDamping(100.0);
    pub const MAX_SPEED: MaxLinearSpeed = MaxLinearSpeed(200.0);
    pub fn collision_layers() -> CollisionLayers {
        CollisionLayers::new(Layer::Empty, Layer::Wall)
    }
}

#[derive(Component)]
pub struct PlayerHurtbox;

#[derive(Component, Default)]
pub enum OrientationMethod {
    #[default]
    Stick,
    Mouse,
}

fn orient_player_with_mouse_input(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    player: Single<(&mut Transform, &mut OrientationMethod), With<Player>>,
    mut motion: MessageReader<MouseMotion>,
    is_dashing: Query<&Dashing>,
) {
    if !is_dashing.is_empty() {
        return;
    }

    let (mut player_transform, mut orientation) = player.into_inner();
    if let OrientationMethod::Stick = *orientation {
        if motion.read().last().is_none() {
            *orientation = OrientationMethod::Mouse;
        } else {
            return;
        }
    }

    let (camera, camera_transform) = camera.into_inner();
    if let Some(Ok(cursor_translation)) = window
        .cursor_position()
        .map(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        let target = cursor_translation - player_transform.translation.xy();
        let normalized_translation = target.normalize_or_zero();

        if normalized_translation != Vec2::ZERO {
            let angle = Vec2::Y.angle_to(normalized_translation);
            player_transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}
