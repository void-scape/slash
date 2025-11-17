use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    health::{Damage, EnemyHitbox, FriendlyHitbox, Hitbox, Hurtbox},
    query::AncestorQuery,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (attack_duration, attack_cooldown))
        .add_observer(propogate_trigger_weapon)
        .add_observer(trigger_weapon)
        .add_observer(hit_event);
}

// WEAPONS

#[derive(Component)]
#[require(
    Weapon,
    Damage(1.0),
    AttackDuration::from_seconds(0.1),
    AttackCooldown::from_seconds(0.2),
    Collider::rectangle(50.0, 20.0)
)]
pub struct Dagger;

// COMPONENTS AND SYSTEMS

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
        let weapon = weapons.get(start.collider2)?;
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
    mut weapons: Query<(&Transform, &AttackDuration, &mut AttackCooldown, &Collider), With<Weapon>>,
) {
    if let Ok((transform, duration, mut cooldown, collider)) = weapons.get_mut(trigger.entity) {
        if !cooldown.0.is_finished() {
            return;
        }
        cooldown.0.reset();

        let translation = transform.translation.normalize_or_zero().xy();
        let angle = if translation != Vec2::ZERO {
            Vec2::Y.angle_to(translation)
        } else {
            0.0
        };

        let mut entity = commands.spawn((
            ChildOf(trigger.entity),
            duration.clone(),
            FriendlyHitbox,
            collider.clone(),
            Transform::from_rotation(Quat::from_rotation_z(angle)),
        ));

        if trigger.friendly {
            entity.insert(FriendlyHitbox);
        } else {
            entity.insert(EnemyHitbox);
        }
    }
}
