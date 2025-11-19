use avian2d::prelude::*;
use bevy::{
    ecs::{lifecycle::HookContext, system::SystemId, world::DeferredWorld},
    platform::collections::HashMap,
    prelude::*,
};

use crate::{
    HEIGHT, WIDTH,
    bits::BitProducer,
    health::{EnemyHitbox, FriendlyHitbox, Hitbox, Hurtbox},
    physics::acceleration,
};
use bevy_tween::{
    combinator::tween,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use std::{any::TypeId, time::Duration};

pub fn plugin(app: &mut App) {
    app.init_resource::<AttackHandlerRegistry>()
        .add_systems(
            Update,
            (
                weapon_sprite,
                weapon_reach,
                attack_duration,
                attack_cooldown,
                despawn_bullets,
                (finish_throw, remove_weapon_rigidbody).chain(),
            ),
        )
        .add_observer(propogate_trigger_weapon)
        .add_observer(trigger_weapon)
        .add_observer(handle_attack)
        .add_observer(hit_event);
}

// WEAPONS

// GUNS

#[derive(Component)]
#[require(
    Weapon,
    Damage(1.0),
    WeaponReach(15.0),
    WeaponDurability::Fire(3),
    AttackDamage(Damage(1.0)),
    AttackHandler::bullet(),
    AttackCooldown::from_seconds(0.2),
    Collider::rectangle(20.0, 20.0),
    Sprite::from_color(Color::WHITE, Vec2::splat(20.0)),
    Name::new("Pistol")
)]
pub struct Pistol;

// MELEE

#[derive(Component)]
#[require(
    Weapon,
    Damage(1.0),
    WeaponReach(15.0),
    WeaponDurability::Hit(3),
    AttackHandler::melee(),
    AttackDamage(Damage(1.0)),
    AttackDuration::from_seconds(0.1),
    AttackCooldown::from_seconds(0.2),
    Collider::rectangle(50.0, 20.0),
    WeaponSprite("weapons/1.png"),
    Name::new("Dagger")
)]
pub struct Dagger;

#[derive(Component)]
#[require(
    Weapon,
    Damage(1.5),
    WeaponReach(25.0),
    WeaponDurability::Hit(3),
    AttackHandler::melee(),
    AttackDamage(Damage(1.5)),
    AttackDuration::from_seconds(0.2),
    AttackCooldown::from_seconds(0.4),
    Collider::rectangle(35.0, 55.0),
    WeaponSprite("weapons/4.png"),
    Name::new("Broadsword")
)]
pub struct Broadsword;

#[derive(Component)]
#[require(
    Weapon,
    Damage(2.5),
    WeaponReach(30.0),
    WeaponDurability::Hit(3),
    AttackHandler::melee(),
    AttackDamage(Damage(2.5)),
    AttackDuration::from_seconds(0.3),
    AttackCooldown::from_seconds(1.0),
    Collider::rectangle(60.0, 60.0),
    WeaponSprite("weapons/7.png"),
    Name::new("Axe")
)]
pub struct Axe;

// COMPONENTS AND SYSTEMS

#[derive(Default, Component)]
#[require(
    Sensor,
    Transform,
    BitProducer,
    WeaponKnockback,
    LockedAxes::ROTATION_LOCKED,
    // Disable the collider _ON_ the weapon. The weapon's collider should only
    // be enabled when it is thrown by the player.
    ColliderDisabled,
)]
#[cfg_attr(feature = "debug", require(DebugRender::none()))]
pub struct Weapon;

#[derive(Clone, Copy, Component)]
pub struct Damage(pub f32);

/// Number of hits that a weapon can susatin before shattering.
#[derive(Component)]
pub enum WeaponDurability {
    Fire(usize),
    Hit(usize),
}

#[derive(Component)]
struct DecrementDurabilityOnHit(Entity);

#[derive(Clone, Copy, Component)]
pub struct WeaponKnockback(pub f32);

impl Default for WeaponKnockback {
    fn default() -> Self {
        Self(200.0)
    }
}

pub fn weapon_knockback(hit: On<HitEvent>, mut commands: Commands) {
    let start = hit.knockback;
    let end = Vec2::ZERO;
    let animation = commands
        .animation()
        .insert(tween(
            Duration::from_secs_f32(0.2),
            EaseKind::Linear,
            hit.target.into_target().with(acceleration(start, end)),
        ))
        .id();
    // TODO: despawn knockback tweens
    commands.entity(hit.target).add_child(animation);
}

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

fn weapon_reach(
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

#[derive(Component)]
pub struct AttackDamage(pub Damage);

/// Registers an [`AttackHandlerSystem`] for a weapon.
#[derive(Component)]
#[component(on_insert = Self::insert)]
pub struct AttackHandler(TypeId, Option<Box<dyn FnOnce(&mut World) + Send + Sync>>);

impl AttackHandler {
    pub fn new<T, M>(f: T) -> Self
    where
        T: IntoSystem<In<TriggerWeaponData>, Result, M> + Send + Sync + 'static,
    {
        let id = TypeId::of::<T>();
        Self(
            id,
            Some(Box::new(move |world: &mut World| {
                world.resource_scope::<AttackHandlerRegistry, ()>(move |world, mut registry| {
                    registry
                        .0
                        .entry(id)
                        .or_insert_with(|| world.register_system(f));
                })
            })),
        )
    }

    pub fn melee() -> Self {
        Self::new(default_melee_handler)
    }

    pub fn bullet() -> Self {
        Self::new(default_bullet_handler)
    }

    fn insert(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().queue(move |world: &mut World| {
            let mut handler = world.get_mut::<Self>(ctx.entity).unwrap();
            if let Some(command) = handler.1.take() {
                command(world);
            }
        });
    }
}

#[derive(Default, Resource)]
struct AttackHandlerRegistry(HashMap<TypeId, SystemId<In<TriggerWeaponData>, Result>>);

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

pub struct TriggerWeaponData {
    attack_vector: Vec2,
    attack: Entity,
    weapon: Entity,
}

fn trigger_weapon(
    trigger: On<TriggerWeapon>,
    registry: ResMut<AttackHandlerRegistry>,
    mut commands: Commands,
    mut weapons: Query<
        (
            &mut AttackCooldown,
            Option<&mut WeaponDurability>,
            &WeaponKnockback,
            &BitProducer,
            &AttackHandler,
        ),
        With<Weapon>,
    >,
    transforms: Query<&GlobalTransform>,
) -> Result {
    if let Ok((mut cooldown, durability, knockback, bit_producer, handler)) =
        weapons.get_mut(trigger.entity)
    {
        if !cooldown.0.is_finished() {
            return Ok(());
        }
        cooldown.0.reset();

        let mut entity = commands.spawn((*knockback, *bit_producer));
        let attack = if trigger.friendly {
            entity.insert(FriendlyHitbox).id()
        } else {
            entity.insert(EnemyHitbox).id()
        };

        let weapon_transform = transforms.get(trigger.entity)?;
        let rotation = weapon_transform.rotation().to_euler(EulerRot::ZYX).0;
        let attack_vector = Vec2::Y.rotate(Vec2::from_angle(rotation));

        let id = *registry.0.get(&handler.0).unwrap();
        let input = TriggerWeaponData {
            attack_vector,
            attack,
            weapon: trigger.entity,
        };
        commands.queue_handled(
            move |world: &mut World| world.run_system_with(id, input),
            bevy::ecs::error::warn,
        );

        if let Some(durability) = durability {
            match durability.into_inner() {
                WeaponDurability::Fire(durability) => {
                    *durability = durability.saturating_sub(1);
                    if *durability == 0 {
                        commands.entity(trigger.entity).despawn();
                    }
                }
                WeaponDurability::Hit(_) => {
                    commands
                        .entity(attack)
                        .insert(DecrementDurabilityOnHit(trigger.entity));
                }
            }
        }
    }
    Ok(())
}

#[derive(Component)]
struct Melee;

fn default_melee_handler(
    data: In<TriggerWeaponData>,
    mut commands: Commands,
    melee_weapons: Query<(&AttackDamage, &AttackDuration, &Collider)>,
) -> Result {
    let (damage, duration, collider) = melee_weapons.get(data.weapon)?;
    commands.entity(data.attack).insert((
        ChildOf(data.weapon),
        duration.clone(),
        collider.clone(),
        damage.0,
        Melee,
    ));
    Ok(())
}

#[derive(Component)]
struct Bullet;

fn default_bullet_handler(
    data: In<TriggerWeaponData>,
    mut commands: Commands,
    bullet_weapons: Query<(&AttackDamage, &GlobalTransform)>,
) -> Result {
    let (damage, transform) = bullet_weapons.get(data.weapon)?;
    let translation = transform.translation().xy();
    let velocity = LinearVelocity(data.attack_vector * 400.0);
    commands
        .entity(data.attack)
        .insert((
            Transform::from_translation(translation.extend(0.0)),
            Sprite::from_color(Color::WHITE, Vec2::splat(20.0)),
            LockedAxes::ROTATION_LOCKED,
            Collider::circle(10.0),
            RigidBody::Dynamic,
            DestroyOnImpact,
            damage.0,
            velocity,
            Bullet,
        ))
        // `Sensor` causing warnings for some reason. We don't need it
        // since the collision layers exlude the collisions with the
        // player and enemies.
        .remove::<Sensor>();
    Ok(())
}

/// Despawns a hitbox on [`HitEvent`].
#[derive(Component)]
pub struct DestroyOnImpact;

fn handle_attack(
    mut hit: On<HitEvent>,
    mut commands: Commands,
    attacks: Query<(Has<DestroyOnImpact>, Option<&DecrementDurabilityOnHit>)>,
    mut durability: Query<&mut WeaponDurability>,
) {
    if let Some(attacker) = hit.attacker.take() {
        let (destroy, decrement) = attacks.get(attacker).unwrap();
        if destroy {
            commands.entity(attacker).despawn();
        }
        if let Some(DecrementDurabilityOnHit(weapon)) = decrement
            && let Ok(WeaponDurability::Hit(durability)) =
                durability.get_mut(*weapon).as_deref_mut()
        {
            *durability = durability.saturating_sub(1);
            if *durability == 0 {
                commands.entity(*weapon).despawn();
            }
        }
    }
}

#[derive(EntityEvent)]
#[entity_event(auto_propagate)]
pub struct HitEvent {
    #[event_target]
    pub target: Entity,
    /// The hitbox that hit `target`.
    ///
    /// `attacker` is taken and despawned in [`handle_attack`].
    pub attacker: Option<Entity>,
    pub damage: f32,
    /// Observe the root with [`weapon_knockback`] to apply.
    pub knockback: Vec2,
    /// Observe the root with [`produce_bits`](crate::bits::produce_bits) to apply.
    pub bits: usize,
    pub target_translation: Vec2,
    pub attacker_translation: Vec2,
}

fn hit_event(
    start: On<CollisionStart>,
    mut commands: Commands,
    target: Query<&Hurtbox>,
    attacker: Query<(&BitProducer, &WeaponKnockback, &Damage), With<Hitbox>>,
    transforms: Query<&GlobalTransform>,
) -> Result {
    if target.get(start.collider1).is_ok()
        && let Ok((bit_producer, knockback, damage)) = attacker.get(start.collider2)
    {
        let target = start.collider1;
        let attacker = start.collider2;

        let target_transform = transforms.get(target).unwrap();
        let attacker_transform = transforms.get(attacker).unwrap();
        let diff = target_transform.translation().xy() - attacker_transform.translation().xy();

        let damage = damage.0;
        let knockback = diff.normalize_or(Vec2::Y) * knockback.0;
        let bits = bit_producer.0;
        let target_translation = target_transform.translation().xy();
        let attacker_translation = attacker_transform.translation().xy();

        commands.entity(start.collider1).trigger(|target| HitEvent {
            target,
            attacker: Some(start.collider2),
            damage,
            knockback,
            bits,
            target_translation,
            attacker_translation,
        });
    }
    Ok(())
}

fn despawn_bullets(
    mut commands: Commands,
    bullets: Query<(Entity, &GlobalTransform), With<Bullet>>,
) {
    let w = WIDTH / 2.0;
    let h = HEIGHT / 2.0;
    for (entity, gt) in bullets.iter() {
        let translation = gt.translation();
        if translation.x > w || translation.x < -w || translation.y > h || translation.y < -h {
            commands.entity(entity).despawn();
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
