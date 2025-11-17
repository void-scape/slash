use crate::weapon::TriggerWeapon;
use avian2d::prelude::RigidBody;
use bevy::{color::palettes::css::RED, prelude::*};
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

pub mod steering;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(steering::SteeringPlugin)
            .add_systems(Update, attack);
    }
}

#[derive(Component)]
#[require(
    Transform,
    RigidBody::Dynamic,
    Sprite::from_color(RED, Vec2::new(20.0, 20.0)),
    Name::new("Enemy"),
)]
pub struct Enemy;

fn attack(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    for entity in enemies.iter() {
        if rng.random_bool(0.01) {
            commands.entity(entity).trigger(TriggerWeapon::enemy);
        }
    }
}
