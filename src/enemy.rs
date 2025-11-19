use crate::{
    Layer,
    bits::{self, coalescence::CoalesceEvent},
    health::{DeathEvent, DeathSystems, EnemyHurtbox, MaxHealth},
    physics::{Acceleration, CustomPhysicsSystems},
    player::Player,
    weapon::{self, Broadsword, Dagger, Pistol, TriggerWeapon, Weapon, WeaponPickup, WeaponReach},
};
use avian2d::prelude::{
    Collider, ColliderOf, CollisionLayers, LockedAxes, MaxLinearSpeed, RigidBody,
};
use bevy::{
    color::palettes::css::{BLUE, GREEN, RED},
    prelude::*,
};
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (orient_to_player, attack))
            .add_systems(
                FixedPostUpdate,
                (
                    drop_weapon_on_death.in_set(DeathSystems::Prepare),
                    ((target_vector, seperation_vector), apply_force_vectors)
                        .chain()
                        .before(CustomPhysicsSystems::Acceleration),
                ),
            )
            .add_observer(spawn_enemy);
    }
}

#[derive(Component)]
#[require(
    Transform,
    RigidBody::Dynamic,
    Name::new("Enemy"),
    CollisionLayers = Self::collision_layers(),
    LockedAxes::ROTATION_LOCKED,
    MaxLinearSpeed(40.0)
)]
pub struct Enemy;

impl Enemy {
    fn collision_layers() -> CollisionLayers {
        CollisionLayers::new(Layer::Empty, Layer::Wall)
    }
}

fn spawn_enemy(
    gibblet: On<CoalesceEvent>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    player: Query<Entity, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) -> Result {
    let transform = transforms.get(gibblet.0)?;
    let mut entity = commands.spawn((
        Enemy,
        Collider::circle(10.0),
        Transform::from_translation(transform.translation()),
    ));
    if let Ok(player) = player.single() {
        entity.insert(SteerTarget(player));
    }
    entity
        .observe(weapon::weapon_knockback)
        .observe(bits::produce_bits);
    sample_enemy_type(entity, &mut rng);
    Ok(())
}

fn sample_enemy_type(mut entity: EntityCommands, rng: &mut impl Rng) {
    let enemy_type = rng.random_range(0..=2);
    match enemy_type {
        0 => {
            let size = 20.0;
            entity.insert((
                Sprite::from_color(RED, Vec2::splat(size)),
                MaxHealth(3.0),
                children![
                    (
                        EnemyHurtbox,
                        avian2d::prelude::Collider::rectangle(size, size),
                        Transform::default(),
                    ),
                    (Dagger, WeaponReach(size * 0.8))
                ],
            ));
        }
        1 => {
            let size = 25.0;
            entity.insert((
                Sprite::from_color(GREEN, Vec2::splat(size)),
                MaxHealth(2.0),
                children![
                    (
                        EnemyHurtbox,
                        avian2d::prelude::Collider::rectangle(size, size),
                        Transform::default(),
                    ),
                    (Pistol, WeaponReach(size * 0.8))
                ],
            ));
        }
        2 => {
            let size = 30.0;
            entity.insert((
                Sprite::from_color(BLUE, Vec2::splat(size)),
                MaxHealth(4.0),
                children![
                    (
                        EnemyHurtbox,
                        avian2d::prelude::Collider::rectangle(size, size),
                        Transform::default(),
                    ),
                    (Broadsword, WeaponReach(size * 1.2))
                ],
            ));
        }
        _ => unreachable!(),
    }
}

fn attack(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    for entity in enemies.iter() {
        if rng.random_bool(0.02) {
            commands.entity(entity).trigger(TriggerWeapon::enemy);
        }
    }
}

fn drop_weapon_on_death(
    mut commands: Commands,
    mut reader: MessageReader<DeathEvent>,
    children: Query<&Children>,
    weapons: Query<Entity, With<Weapon>>,
    transforms: Query<&GlobalTransform>,
) -> Result {
    for event in reader.read() {
        let transform = transforms.get(event.0)?;
        if let Ok(children) = children.get(event.0) {
            let mut iter = weapons.iter_many(children);
            if let Some(weapon_entity) = iter.fetch_next() {
                commands
                    .entity(weapon_entity)
                    .remove::<(ChildOf, ColliderOf)>()
                    .insert((WeaponPickup::default(), transform.compute_transform()));
            }
        }
    }
    Ok(())
}

#[derive(Component)]
#[require(Acceleration, TargetVector, SeperationVector)]
pub struct SteerTarget(pub Entity);

#[derive(Default, Component)]
pub struct TargetVector(pub Vec2);

#[derive(Default, Component)]
pub struct SeperationVector(pub Vec2);

fn target_vector(
    mut steering: Query<(&mut TargetVector, &GlobalTransform, &SteerTarget)>,
    targets: Query<&GlobalTransform>,
) -> Result {
    for (mut target_vector, gt, steer_target) in steering.iter_mut() {
        if let Ok(target) = targets.get(steer_target.0) {
            let diff = target.translation().xy() - gt.translation().xy();
            let new_vector = diff.normalize_or_zero();
            if new_vector != Vec2::ZERO {
                target_vector.0 = new_vector;
            }
        }
    }
    Ok(())
}

fn seperation_vector(
    steering: Query<(Entity, &GlobalTransform), With<SeperationVector>>,
    mut seperation_vectors: Query<&mut SeperationVector>,
) -> Result {
    let radius = 100.0;
    let radius_squared = radius * radius;
    for (target_entity, gt) in steering.iter() {
        let target_translation = gt.translation().xy();
        let mut seperation_force = Vec2::ZERO;
        for (neighbor, neighbor_gt) in steering.iter() {
            if neighbor == target_entity {
                continue;
            }
            let neighbor_translation = neighbor_gt.translation().xy();
            if target_translation.distance_squared(neighbor_translation) <= radius_squared {
                let s = (target_translation - neighbor_translation).normalize_or_zero();
                seperation_force += s;
            }
        }
        let mut seperation_vector = seperation_vectors.get_mut(target_entity)?;
        seperation_vector.0 = seperation_force.normalize_or_zero();
    }
    Ok(())
}

fn apply_force_vectors(mut steering: Query<(&mut Acceleration, &TargetVector, &SeperationVector)>) {
    let impulse = 2.0;
    for (mut acceleration, target, seperation) in steering.iter_mut() {
        let force = target.0 * 2.0 + seperation.0 * 1.2;
        acceleration.0 += force * impulse;
    }
}

fn orient_to_player(
    mut enemies: Query<&mut Transform, With<Enemy>>,
    player: Single<&GlobalTransform, With<Player>>,
) {
    let player_translation = player.translation().xy();
    for mut transform in enemies.iter_mut() {
        let looking_at = player_translation - transform.translation.xy();
        let angle = Vec2::Y.angle_to(looking_at.normalize_or(Vec2::Y));
        transform.rotation = Quat::from_rotation_z(angle);
    }
}
