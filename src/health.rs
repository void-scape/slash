use crate::{Layer, query::AncestorQuery, weapon::HitEvent};
use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{BLACK, RED},
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_message::<DeathEvent>()
        .configure_sets(
            FixedPostUpdate,
            (
                DeathSystems::Despawn.after(DeathSystems::Prepare),
                DeathSystems::Prepare.after(PhysicsSystems::Last),
            ),
        )
        .add_systems(FixedPostUpdate, despawn_dead.in_set(DeathSystems::Despawn))
        .add_systems(Update, (spawn_health_bars, update_health_bars).chain())
        .add_observer(current_health);
}

/// Orders death systems in the `FixedPostUpdate` schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum DeathSystems {
    Prepare,
    /// Dead entities are despawned.
    Despawn,
}

#[derive(Component)]
#[component(on_insert = Self::insert)]
pub struct MaxHealth(pub f32);

impl MaxHealth {
    fn insert(mut world: DeferredWorld, ctx: HookContext) {
        let max = world.get::<Self>(ctx.entity).unwrap().0;
        world
            .commands()
            .entity(ctx.entity)
            .insert_if_new(CurrentHealth(max));
    }
}

#[derive(Component)]
pub struct CurrentHealth(pub f32);

fn current_health(
    mut hit: On<HitEvent>,
    mut health: Query<&mut CurrentHealth>,
    mut writer: MessageWriter<DeathEvent>,
) {
    if let Ok(mut health) = health.get_mut(hit.target) {
        hit.propagate(false);
        health.0 -= hit.damage;
        if health.0 <= 0.0 {
            writer.write(DeathEvent(hit.target));
        }
    }
}

/// An entity who just died.
///
/// Since [`HitEvent`] is triggered in the `Avian` schedule, there needs to be
/// synchronization between systems that want to observe deaths before they are
/// despawned, thus, this is a `Message`.
#[derive(Message)]
pub struct DeathEvent(pub Entity);

pub fn despawn_dead(mut commands: Commands, mut reader: MessageReader<DeathEvent>) {
    for event in reader.read() {
        commands.entity(event.0).despawn();
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

fn spawn_health_bars(mut commands: Commands, bars: Query<Entity, Added<MaxHealth>>) {
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
    health: AncestorQuery<(&CurrentHealth, &MaxHealth), (), HealthBarOf>,
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
        let (current, max) = health.get(child_of.0)?;
        transform.scale.x = current.0 / max.0;
    }
    Ok(())
}

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
