use crate::{query::AncestorQuery, weapon::HitEvent};
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, despawn_dead).add_observer(on_hit);
}

#[derive(Clone, Copy, Component)]
pub struct Health(f32);

impl Health {
    pub fn new(health: f32) -> Self {
        Self(health)
    }

    pub fn damage(&mut self, damage: f32) {
        self.0 -= damage;
    }

    pub fn is_dead(&self) -> bool {
        self.0 <= 0.0
    }
}

#[derive(Clone, Copy, Component)]
pub struct Damage(pub f32);

#[derive(Default, PhysicsLayer)]
pub enum Layer {
    #[default]
    Empty,
    FriendlyHurtboxEnemyHitbox,
    FriendlyHitboxEnemyHurtbox,
}

#[derive(Default, Component)]
pub struct Hitbox;

#[derive(Default, Component)]
pub struct Hurtbox;

#[derive(Component)]
#[require(
    Hurtbox,
    Sensor,
    CollisionLayers = Self::collision_layers(),
    CollisionEventsEnabled,
)]
pub struct FriendlyHurtbox;

impl FriendlyHurtbox {
    fn collision_layers() -> CollisionLayers {
        CollisionLayers::new(
            Layer::FriendlyHurtboxEnemyHitbox,
            Layer::FriendlyHurtboxEnemyHitbox,
        )
    }
}

#[derive(Component)]
#[require(Hitbox, Sensor, CollisionLayers = Self::collision_layers())]
pub struct FriendlyHitbox;

impl FriendlyHitbox {
    fn collision_layers() -> CollisionLayers {
        CollisionLayers::new(
            Layer::FriendlyHitboxEnemyHurtbox,
            Layer::FriendlyHitboxEnemyHurtbox,
        )
    }
}

#[derive(Component)]
#[require(
    Hurtbox,
    Sensor,
    CollisionLayers = FriendlyHitbox::collision_layers(),
    CollisionEventsEnabled,
)]
pub struct EnemyHurtbox;

#[derive(Component)]
#[require(Hitbox, Sensor, CollisionLayers = FriendlyHurtbox::collision_layers())]
pub struct EnemyHitbox;

fn on_hit(
    hit: On<HitEvent>,
    mut health: AncestorQuery<&mut Health>,
    damage: Query<&Damage>,
) -> Result {
    let mut health = health.get_mut(hit.target)?;
    let damage = damage.get(hit.weapon)?;
    health.damage(damage.0);
    Ok(())
}

fn despawn_dead(mut commands: Commands, health: Query<(Entity, &Health)>) {
    for (entity, health) in health.iter() {
        if health.is_dead() {
            commands.entity(entity).despawn();
        }
    }
}
