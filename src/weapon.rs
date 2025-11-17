use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    health::{Damage, EnemyHitbox, FriendlyHitbox, Hitbox, Hurtbox},
    query::AncestorQuery,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            weapon_sprite,
            weapon_orientation,
            attack_duration,
            attack_cooldown,
            (finish_throw, remove_weapon_rigidbody).chain(),
        ),
    )
    .add_observer(propogate_trigger_weapon)
    .add_observer(trigger_weapon)
    .add_observer(hit_event);
}

// WEAPONS

#[derive(Component)]
#[require(
    Weapon,
    Damage(1.0),
    WeaponReach(15.0),
    AttackDuration::from_seconds(0.1),
    AttackCooldown::from_seconds(0.2),
    Collider::rectangle(50.0, 20.0),
    WeaponSprite("weapons/1.png")
)]
pub struct Dagger;

#[derive(Component)]
#[require(
    Weapon,
    Damage(1.5),
    WeaponReach(25.0),
    AttackDuration::from_seconds(0.2),
    AttackCooldown::from_seconds(0.4),
    Collider::rectangle(35.0, 55.0),
    WeaponSprite("weapons/4.png")
)]
pub struct Broadsword;

#[derive(Component)]
#[require(
    Weapon,
    Damage(2.5),
    WeaponReach(30.0),
    AttackDuration::from_seconds(0.3),
    AttackCooldown::from_seconds(1.0),
    Collider::rectangle(60.0, 60.0),
    WeaponSprite("weapons/7.png")
)]
pub struct Axe;

// COMPONENTS AND SYSTEMS

/// Weapon pickup radius.
#[derive(Component)]
pub struct WeaponPickup(pub f32);

impl Default for WeaponPickup {
    fn default() -> Self {
        WeaponPickup(50.0)
    }
}

/// The seperation between the root transform and the middle of the
/// weapon transform.
#[derive(Component)]
pub struct WeaponReach(pub f32);

fn weapon_orientation(
    mut weapons: Query<
        (&mut Transform, &WeaponReach),
        (With<ChildOf>, Or<(Changed<WeaponReach>, Added<ChildOf>)>),
    >,
) {
    for (mut transform, reach) in weapons.iter_mut() {
        transform.translation.x = 0.0;
        transform.translation.y = reach.0;
        transform.rotation = Quat::default();
    }
}

#[derive(Component)]
struct WeaponSprite(&'static str);

fn weapon_sprite(
    mut commands: Commands,
    server: Res<AssetServer>,
    weapon_sprites: Query<(Entity, &WeaponSprite), Added<WeaponSprite>>,
) {
    for (entity, sprite_path) in weapon_sprites.iter() {
        commands
            .entity(entity)
            .insert(Sprite::from_image(server.load(sprite_path.0)));
    }
}

#[derive(EntityEvent)]
pub struct HitEvent {
    /// The hurtbox this event hits.
    #[event_target]
    pub target: Entity,
    /// The hitbox that hit `target`.
    pub attacker: Entity,
    /// The weapon that `attacker` used to hit `target`.
    pub weapon: Entity,
}

fn hit_event(
    start: On<CollisionStart>,
    mut commands: Commands,
    target: Query<&Hurtbox>,
    attacker: Query<&Hitbox>,
    weapons: AncestorQuery<Entity, With<Weapon>>,
) -> Result {
    if target.get(start.collider1).is_ok() && attacker.get(start.collider2).is_ok() {
        let weapon = weapons.get_inclusive(start.collider2)?;
        commands.entity(start.collider1).trigger(|target| HitEvent {
            target,
            attacker: start.collider2,
            weapon,
        });
    }
    Ok(())
}

#[derive(Default, Component)]
// Disable the collider _ON_ the weapon. `TriggerWeapon` will spawn a hitbox
// and clone this collider.
#[require(Transform, ColliderDisabled)]
#[cfg_attr(feature = "debug", require(DebugRender::none()))]
pub struct Weapon;

#[derive(Clone, Component)]
pub struct AttackDuration(Timer);

impl AttackDuration {
    pub fn from_seconds(duration: f32) -> Self {
        Self(Timer::from_seconds(duration, TimerMode::Once))
    }
}

fn attack_duration(
    mut commands: Commands,
    time: Res<Time>,
    mut durations: Query<(Entity, &mut AttackDuration), Without<Weapon>>,
) {
    for (entity, mut duration) in durations.iter_mut() {
        duration.0.tick(time.delta());
        if duration.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Clone, Component)]
pub struct AttackCooldown(Timer);

impl AttackCooldown {
    pub fn from_seconds(duration: f32) -> Self {
        let mut timer = Timer::from_seconds(duration, TimerMode::Once);
        timer.finish();
        Self(timer)
    }
}

fn attack_cooldown(time: Res<Time>, mut cooldowns: Query<&mut AttackCooldown, With<Weapon>>) {
    for mut duration in cooldowns.iter_mut() {
        if !duration.0.is_finished() {
            duration.0.tick(time.delta());
        }
    }
}

#[derive(Clone, EntityEvent)]
pub struct TriggerWeapon {
    entity: Entity,
    friendly: bool,
}

impl TriggerWeapon {
    pub fn friendly(entity: Entity) -> Self {
        Self {
            entity,
            friendly: true,
        }
    }

    pub fn enemy(entity: Entity) -> Self {
        Self {
            entity,
            friendly: false,
        }
    }
}

fn propogate_trigger_weapon(
    trigger: On<TriggerWeapon>,
    mut commands: Commands,
    children: Query<&Children, Without<Weapon>>,
    weapons: Query<Entity, With<Weapon>>,
) {
    if let Ok(children) = children.get(trigger.entity) {
        let mut iter = weapons.iter_many(children);
        if let Some(weapon_entity) = iter.next() {
            commands
                .entity(weapon_entity)
                .trigger(|entity| TriggerWeapon {
                    entity,
                    friendly: trigger.friendly,
                });
            if iter.next().is_some() {
                error!("entity contains multiple weapons");
            }
        }
    }
}

fn trigger_weapon(
    trigger: On<TriggerWeapon>,
    mut commands: Commands,
    mut weapons: Query<(&AttackDuration, &mut AttackCooldown, &Collider), With<Weapon>>,
) {
    if let Ok((duration, mut cooldown, collider)) = weapons.get_mut(trigger.entity) {
        if !cooldown.0.is_finished() {
            return;
        }
        cooldown.0.reset();

        if trigger.friendly {
            commands.spawn((
                ChildOf(trigger.entity),
                FriendlyHitbox,
                duration.clone(),
                collider.clone(),
                Transform::default(),
            ));
        } else {
            commands.spawn((
                ChildOf(trigger.entity),
                EnemyHitbox,
                duration.clone(),
                collider.clone(),
                Transform::default(),
            ));
        }
    }
}

fn finish_throw(mut commands: Commands, weapons: Query<(Entity, &LinearVelocity), With<Weapon>>) {
    for (entity, velocity) in weapons.iter() {
        if velocity.0.length_squared() < 10.0 * 10.0 {
            commands
                .entity(entity)
                .remove::<(Hitbox, FriendlyHitbox, EnemyHitbox)>()
                .insert((
                    ColliderDisabled,
                    WeaponPickup::default(),
                    CollisionLayers::NONE,
                ));
        }
    }
}

fn remove_weapon_rigidbody(
    mut commands: Commands,
    weapons: Query<(Entity, &LinearVelocity), With<Weapon>>,
) {
    for (entity, velocity) in weapons.iter() {
        if velocity.0.length_squared() < f32::EPSILON * f32::EPSILON {
            commands.entity(entity).remove::<RigidBody>();
        }
    }
}
