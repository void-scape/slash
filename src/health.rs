use crate::{Layer, query::AncestorQuery, weapon::HitEvent};
use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{BLACK, RED},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_message::<DeathEvent>()
        .add_systems(FixedPostUpdate, despawn_dead.in_set(DeathSystems::Despawn))
        .add_observer(on_hit)
        .add_systems(Update, (spawn_health_bars, update_health_bars).chain());

    app.configure_sets(
        FixedPostUpdate,
        (
            DeathSystems::Despawn.after(DeathSystems::Prepare),
            DeathSystems::Prepare.after(PhysicsSystems::Last),
        ),
    );
}

/// Orders death systems in the `FixedPostUpdate` schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum DeathSystems {
    Prepare,
    /// Dead entities are despawned.
    Despawn,
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

#[derive(Default, Component)]
#[require(Sensor)]
pub struct Hitbox;

#[derive(Default, Component)]
#[require(Sensor)]
pub struct Hurtbox;

#[derive(Component)]
#[require(
    Hurtbox,
    CollisionLayers = Self::collision_layers(),
    CollisionEventsEnabled,
)]
pub struct FriendlyHurtbox;

impl FriendlyHurtbox {
    pub fn collision_layers() -> CollisionLayers {
        CollisionLayers::new(
            Layer::FriendlyHurtboxEnemyHitbox,
            Layer::FriendlyHurtboxEnemyHitbox,
        )
    }
}

#[derive(Component)]
#[require(Hitbox, CollisionLayers = Self::collision_layers())]
pub struct FriendlyHitbox;

impl FriendlyHitbox {
    pub fn collision_layers() -> CollisionLayers {
        CollisionLayers::new(
            Layer::FriendlyHitboxEnemyHurtbox,
            Layer::FriendlyHitboxEnemyHurtbox,
        )
    }
}

#[derive(Default, Component)]
#[require(Hurtbox, CollisionLayers = Self::collision_layers(), CollisionEventsEnabled)]
pub struct EnemyHurtbox;

impl EnemyHurtbox {
    pub fn collision_layers() -> CollisionLayers {
        FriendlyHitbox::collision_layers()
    }
}

#[derive(Default, Component)]
#[require(Hitbox, CollisionLayers = Self::collision_layers())]
pub struct EnemyHitbox;

impl EnemyHitbox {
    pub fn collision_layers() -> CollisionLayers {
        FriendlyHurtbox::collision_layers()
    }
}

/// An entity who just died.
///
/// Since [`HitEvent`] is triggered in the `Avian` schedule, there needs to be
/// synchronization between systems that want to observe deaths before they are
/// despawned, thus, this is a `Message`.
#[derive(Message)]
pub struct DeathEvent(pub Entity);

fn on_hit(
    hit: On<HitEvent>,
    mut health: AncestorQuery<(Entity, &mut Health)>,
    damage: Query<&Damage>,
    mut writer: MessageWriter<DeathEvent>,
) -> Result {
    let (root_entity, mut health) = health.get_inclusive_mut(hit.target)?;
    let damage = damage.get(hit.weapon)?;
    health.damage(damage.0);
    if health.is_dead() {
        writer.write(DeathEvent(root_entity));
    }
    Ok(())
}

pub fn despawn_dead(mut commands: Commands, mut reader: MessageReader<DeathEvent>) {
    for event in reader.read() {
        commands
            .entity(event.0)
            .despawn_related::<HealthBars>()
            .despawn();
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
            Name::new("Health bar"),
            HealthBarOf(entity),
            Sprite::from_color(BLACK, Vec2::new(50.0, 5.0)),
            Transform::from_xyz(0.0, 15.0, 1.0),
            children![(
                HealthBarFront,
                Sprite::from_color(RED, Vec2::new(50.0, 5.0)),
                Transform::from_xyz(0.0, 0.0, 1.0),
            )],
        ));
    }
}

fn update_health_bars(
    health: AncestorQuery<&Health, (), HealthBarOf>,
    mut bars: Query<(&mut Transform, &ChildOf), With<HealthBarFront>>,
    mut back_bars: Query<(Entity, &mut Transform), (Without<HealthBarFront>, With<HealthBarOf>)>,
    global_transforms: AncestorQuery<&GlobalTransform, (), HealthBarOf>,
) -> Result {
    for (entity, mut transform) in back_bars.iter_mut() {
        let gt = global_transforms.get(entity)?;
        let newt = gt.compute_transform().translation;
        transform.translation.x = newt.x;
        transform.translation.y = newt.y + 15.0;
    }
    for (mut transform, child_of) in bars.iter_mut() {
        let health = health.get(child_of.0)?;
        transform.scale.x = health.current / health.max;
    }
    Ok(())
}
