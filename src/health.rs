use crate::{query::AncestorQuery, weapon::HitEvent};
use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{BLACK, RED},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, despawn_dead)
        .add_observer(on_hit)
        .add_systems(Update, (spawn_health_bars, update_health_bars));
}

#[derive(Clone, Copy, Component)]
pub struct Health {
    max: f32,
    current: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { max, current: max }
    }

    pub fn damage(&mut self, damage: f32) {
        self.current -= damage;
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
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
    pub fn collision_layers() -> CollisionLayers {
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
    let mut health = health.get_inclusive_mut(hit.target)?;
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

#[derive(Component)]
#[relationship_target(relationship = HealthBarOf, linked_spawn)]
pub struct HealthBars(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = HealthBars)]
pub struct HealthBarOf(Entity);

#[derive(Component)]
struct HealthBarFront;

fn spawn_health_bars(mut commands: Commands, bars: Query<Entity, Added<Health>>) {
    for entity in bars.iter() {
        commands.spawn((
            HealthBarOf(entity),
            HealthBarFront,
            Sprite::from_color(RED, Vec2::new(50.0, 5.0)),
            Transform::from_xyz(0.0, 15.0, 2.0),
        ));
        commands.spawn((
            HealthBarOf(entity),
            Sprite::from_color(BLACK, Vec2::new(50.0, 5.0)),
            Transform::from_xyz(0.0, 15.0, 1.0),
        ));
    }
}

fn update_health_bars(
    health: AncestorQuery<&Health, (), HealthBarOf>,
    mut bars: Query<(Entity, &mut Transform), With<HealthBarFront>>,
    mut back_bars: Query<(Entity, &mut Transform), (Without<HealthBarFront>, With<HealthBarOf>)>,
    global_transforms: AncestorQuery<&GlobalTransform, (), HealthBarOf>,
) -> Result {
    for (entity, mut transform) in back_bars.iter_mut() {
        let gt = global_transforms.get(entity)?;
        let newt = gt.compute_transform().translation;
        transform.translation.x = newt.x;
        transform.translation.y = newt.y + 15.0;
    }
    for (entity, mut transform) in bars.iter_mut() {
        let health = health.get(entity)?;
        let gt = global_transforms.get(entity)?;
        transform.scale.x = health.current / health.max;

        let newt = gt.compute_transform().translation;
        transform.translation.x = newt.x;
        transform.translation.y = newt.y + 15.0;
    }
    Ok(())
}
