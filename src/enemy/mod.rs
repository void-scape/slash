use crate::{
    Layer,
    bits::coalescence::CoalesceEvent,
    enemy::steering::SteerTarget,
    health::{DeathEvent, DeathSystems, EnemyHurtbox, Health},
    player::Player,
    weapon::{Broadsword, Dagger, Pistol, TriggerWeapon, Weapon, WeaponPickup, WeaponReach},
};
use avian2d::prelude::{Collider, ColliderOf, CollisionLayers, LockedAxes, RigidBody};
use bevy::{
    color::palettes::css::{BLUE, GREEN, RED},
    prelude::*,
};
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

pub mod steering;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(steering::SteeringPlugin)
            .add_systems(Update, attack)
            .add_systems(
                FixedPostUpdate,
                drop_weapon_on_death.in_set(DeathSystems::Prepare),
            )
            .add_observer(spawn_enemy);
    }
}

fn spawn_enemy(
    gibblet: On<CoalesceEvent>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    player: Query<Entity, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) -> Result {
    commands.entity(gibblet.0).despawn();
    let transform = transforms.get(gibblet.0)?;
    let mut entity = commands.spawn((
        Enemy,
        Collider::circle(10.0),
        Transform::from_translation(transform.translation()),
    ));
    if let Ok(player) = player.single() {
        entity.insert(SteerTarget(player));
    }
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
                Health::new(3.0),
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
                Health::new(2.0),
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
                Health::new(4.0),
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

#[derive(Component)]
#[require(
    Transform,
    RigidBody::Dynamic,
    Name::new("Enemy"),
    CollisionLayers = Self::collision_layers(),
    LockedAxes::ROTATION_LOCKED,
)]
pub struct Enemy;

impl Enemy {
    fn collision_layers() -> CollisionLayers {
        CollisionLayers::new(Layer::Empty, Layer::Wall)
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
