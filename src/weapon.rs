use avian2d::prelude::*;
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    HEIGHT, WIDTH,
    health::{Damage, EnemyHitbox, FriendlyHitbox, Hitbox, Hurtbox},
};

pub fn plugin(app: &mut App) {
    app.add_message::<HitEvent>()
        .add_systems(
            Update,
            (
                weapon_sprite,
                weapon_orientation,
                attack_duration,
                attack_cooldown,
                despawn_bullets,
                (finish_throw, remove_weapon_rigidbody).chain(),
            ),
        )
        .add_systems(PostUpdate, destroy_on_impact)
        .add_observer(propogate_trigger_weapon)
        .add_observer(trigger_weapon)
        .add_observer(hit_event);
}

// WEAPONS

// GUNS

#[derive(Component)]
#[require(
    Weapon,
    Damage(1.0),
    BulletGenerator::basic(),
    WeaponReach(15.0),
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
    Melee,
    Damage(1.0),
    WeaponReach(15.0),
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
    Melee,
    Damage(1.5),
    WeaponReach(25.0),
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
    Melee,
    Damage(2.5),
    WeaponReach(30.0),
    AttackDuration::from_seconds(0.3),
    AttackCooldown::from_seconds(1.0),
    Collider::rectangle(60.0, 60.0),
    WeaponSprite("weapons/7.png"),
    Name::new("Axe")
)]
pub struct Axe;

// COMPONENTS AND SYSTEMS

#[derive(Default, Component)]
pub struct Melee;

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub struct BulletGenerator(Box<dyn FnMut(EntityCommands, LinearVelocity) + Send + Sync>);

impl BulletGenerator {
    pub fn basic() -> Self {
        Self(Box::new(|mut entity, mut normalized_velocity| {
            normalized_velocity.0 *= 400.0;
            entity
                .insert((
                    Sprite::from_color(Color::WHITE, Vec2::splat(20.0)),
                    normalized_velocity,
                    DestroyOnImpact,
                    Collider::circle(10.0),
                    LockedAxes::ROTATION_LOCKED,
                ))
                // `Sensor` causing warnings for some reason. We don't need it
                // since the collision layers exlude the collisions with the
                // player and enemies.
                .remove::<Sensor>();
        }))
    }
}

#[derive(Component)]
pub struct DestroyOnImpact;

fn destroy_on_impact(
    mut commands: Commands,
    mut reader: MessageReader<HitEvent>,
    destroy: Query<&DestroyOnImpact>,
) {
    for event in reader.read() {
        if destroy.get(event.attacker).is_ok() {
            commands.entity(event.attacker).despawn();
        }
    }
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

#[derive(EntityEvent, Message)]
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
    attacker: Query<&AttackSource, With<Hitbox>>,
    mut writer: MessageWriter<HitEvent>,
) -> Result {
    if target.get(start.collider1).is_ok()
        && let Ok(source) = attacker.get(start.collider2)
    {
        let weapon = source.0;
        commands.entity(start.collider1).trigger(|target| HitEvent {
            target,
            attacker: start.collider2,
            weapon,
        });
        writer.write(HitEvent {
            target: start.collider1,
            attacker: start.collider2,
            weapon,
        });
    }
    Ok(())
}

#[derive(Default, Component)]
// Disable the collider _ON_ the weapon. `TriggerWeapon` will spawn a hitbox
// and clone this collider.
#[require(Transform, Sensor, ColliderDisabled, LockedAxes::ROTATION_LOCKED)]
#[cfg_attr(feature = "debug", require(DebugRender::none()))]
#[component(on_insert = Self::insert)]
pub struct Weapon;

impl Weapon {
    fn insert(mut world: DeferredWorld, ctx: HookContext) {
        // Insert `AttackSource` into the weapon so that `hit_event` can find
        // the weapon source when it collides with hurtboxes.
        world
            .commands()
            .entity(ctx.entity)
            .insert(AttackSource(ctx.entity));
    }
}

/// Points to the weapon that triggered this attack.
#[derive(Component)]
#[require(Sensor)]
pub struct AttackSource(pub Entity);

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
    mut melee_weapons: Query<
        (&AttackDuration, &mut AttackCooldown, &Collider),
        (With<Weapon>, With<Melee>),
    >,
    mut bullet_generators: Query<
        (&mut AttackCooldown, &mut BulletGenerator),
        (With<Weapon>, Without<Melee>),
    >,
    transforms: Query<&GlobalTransform>,
) -> Result {
    if let Ok((duration, mut cooldown, collider)) = melee_weapons.get_mut(trigger.entity) {
        if !cooldown.0.is_finished() {
            return Ok(());
        }
        cooldown.0.reset();

        if trigger.friendly {
            commands.spawn((
                ChildOf(trigger.entity),
                AttackSource(trigger.entity),
                FriendlyHitbox,
                duration.clone(),
                collider.clone(),
                Transform::default(),
            ));
        } else {
            commands.spawn((
                ChildOf(trigger.entity),
                AttackSource(trigger.entity),
                EnemyHitbox,
                duration.clone(),
                collider.clone(),
                Transform::default(),
            ));
        }
    } else if let Ok((mut cooldown, mut generator)) = bullet_generators.get_mut(trigger.entity) {
        if !cooldown.0.is_finished() {
            return Ok(());
        }
        cooldown.0.reset();

        let gt = transforms.get(trigger.entity)?;
        let translation = gt.translation().xy();
        let entity = if trigger.friendly {
            commands.spawn((
                Bullet,
                AttackSource(trigger.entity),
                FriendlyHitbox,
                RigidBody::Dynamic,
                Transform::from_translation(translation.extend(0.0)),
            ))
        } else {
            commands.spawn((
                Bullet,
                AttackSource(trigger.entity),
                EnemyHitbox,
                RigidBody::Dynamic,
                Transform::from_translation(translation.extend(0.0)),
            ))
        };
        let rotation = gt.rotation().to_euler(EulerRot::ZYX).0;
        let normalized_velocity = LinearVelocity(Vec2::Y.rotate(Vec2::from_angle(rotation)));
        (generator.0)(entity, normalized_velocity);
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
