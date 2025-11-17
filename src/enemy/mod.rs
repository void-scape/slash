use avian2d::prelude::RigidBody;
use bevy::{color::palettes::css::RED, prelude::*};

pub mod steering;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(steering::SteeringPlugin);
    }
}

#[derive(Component)]
#[require(
    Transform,
    RigidBody::Dynamic,
    Sprite::from_color(RED, Vec2::new(50.0, 50.0))
)]
pub struct Enemy;
