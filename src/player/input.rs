use super::Player;
use crate::{
    Layer,
    bits::BitEvent,
    enemy::{EnableAttacks, FinisherTarget},
    health::{CurrentHealth, DeathEvent, FriendlyHitbox},
    physics::velocity,
    player::{OrientationMethod, PlayerHurtbox},
    weapon::{TriggerWeapon, Weapon, WeaponPickup},
};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_tween::{
    bevy_time_runner::TimeRunnerEnded,
    interpolate::translation,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use std::time::Duration;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>()
            .add_systems(Update, (end_dash, end_finish))
            .add_observer(inject_bindings)
            .add_observer(apply_movement)
            .add_observer(stop_movement)
            .add_observer(handle_attack)
            .add_observer(handle_dash)
            .add_observer(handle_pick_up)
            .add_observer(handle_throw)
            .add_observer(handle_aim)
            .add_observer(handle_finish);
    }
}

fn inject_bindings(trigger: On<Insert, Player>, mut commands: Commands) {
    commands.entity(trigger.entity).insert(actions!(Player[
        (
            Action::<Move>::new(),
            DeadZone::default(),
            Bindings::spawn((
                Cardinal::wasd_keys(),
                Axial::left_stick(),
            )),
        ),
        (
            Action::<Aim>::new(),
            DeadZone {
                lower_threshold: 0.5,
                ..Default::default()
            },
            SmoothNudge::new(16.0),
            Bindings::spawn((
                Cardinal::arrows(),
                Axial::right_stick(),
            )),
        ),
        (
            Action::<Attack>::new(),
            Press::default(),
            bindings![KeyCode::Space, GamepadButton::West, GamepadButton::RightTrigger2, MouseButton::Left],
        ),
        (
            Action::<Dash>::new(),
            Press::default(),
            bindings![KeyCode::ShiftLeft, GamepadButton::East],
        ),
        (
            Action::<PickUp>::new(),
            Press::default(),
            bindings![KeyCode::KeyF, GamepadButton::South],
        ),
        (
            Action::<Throw>::new(),
            Press::default(),
            bindings![KeyCode::KeyC, GamepadButton::LeftTrigger2, GamepadButton::North],
        ),
        (
            Action::<Finish>::new(),
            Press::default(),
            bindings![KeyCode::KeyE, GamepadButton::RightTrigger],
        ),
    ]));
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;

#[derive(Default, Component)]
pub struct RetainedMove(Vec2);

fn apply_movement(
    movement: On<Fire<Move>>,
    player: Single<
        (
            &mut RetainedMove,
            &mut LinearVelocity,
            &MaxLinearSpeed,
            Has<Dashing>,
            Has<Finishing>,
        ),
        With<Player>,
    >,
) {
    let (mut retained, mut velocity, max_speed, is_dashing, is_finishing) = player.into_inner();
    retained.0 = movement.value;
    if !is_dashing && !is_finishing {
        velocity.0 = movement.value * max_speed.0;
    }
}

fn stop_movement(
    _movement: On<Complete<Move>>,
    mut player: Single<&mut LinearVelocity, (With<Player>, Without<Dashing>, Without<Finishing>)>,
) {
    player.0 = Vec2::ZERO;
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct Aim;

fn handle_aim(
    aim: On<Fire<Aim>>,
    player: Single<(&mut Transform, &mut OrientationMethod), With<Player>>,
) {
    let (mut transform, mut method) = player.into_inner();
    *method = OrientationMethod::Stick;

    let angle = aim.value.normalize_or_zero();
    if angle.length_squared() != 0.0 {
        let angle = Vec2::Y.angle_to(angle);
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

#[derive(InputAction)]
#[action_output(bool)]
struct Attack;

fn handle_attack(
    _attack: On<Fire<Attack>>,
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
) {
    commands.entity(*player).trigger(TriggerWeapon::friendly);
}

#[derive(InputAction)]
#[action_output(bool)]
struct Dash;

#[derive(Component)]
pub struct Dashing;

fn handle_dash(
    _dash: On<Fire<Dash>>,
    mut commands: Commands,
    player: Single<(Entity, &RetainedMove), Without<Finishing>>,
    hurtbox: Single<Entity, With<PlayerHurtbox>>,
) {
    let (player_entity, last_input) = player.into_inner();
    commands.entity(player_entity).insert(Dashing);
    let start = last_input.0 * 1_000.0;
    let end = last_input.0 * Player::MAX_SPEED.0 / 2.0;

    let animation = commands
        .animation()
        .insert_tween_here(
            Duration::from_secs_f32(0.4),
            EaseKind::QuarticOut,
            player_entity.into_target().with(velocity(start, end)),
        )
        .insert(Dashing)
        .id();

    commands
        .entity(player_entity)
        .remove::<(LinearDamping, MaxLinearSpeed)>()
        .add_child(animation);
    commands.entity(*hurtbox).insert(ColliderDisabled);
}

fn end_dash(
    mut commands: Commands,
    dashing: Query<&Dashing>,
    mut ended: MessageReader<TimeRunnerEnded>,
    player: Single<Entity, With<Player>>,
    hurtbox: Single<Entity, With<PlayerHurtbox>>,
) {
    for ended in ended.read() {
        if ended.is_completed() && dashing.contains(ended.entity) {
            commands.entity(ended.entity).despawn();
            commands
                .entity(*player)
                .insert((Player::LINEAR_DAMPING, Player::MAX_SPEED))
                .remove::<Dashing>();
            commands.entity(*hurtbox).remove::<ColliderDisabled>();
        }
    }
}

#[derive(InputAction)]
#[action_output(bool)]
struct PickUp;

fn handle_pick_up(
    _pick_up: On<Fire<PickUp>>,
    mut commands: Commands,
    pickups: Query<(Entity, &GlobalTransform, &WeaponPickup)>,
    player: Single<
        (Entity, &GlobalTransform, &Children),
        (With<Player>, Without<Dashing>, Without<Finishing>),
    >,
    weapons: Query<&Weapon>,
) {
    let (player_entity, player_transform, children) = player.into_inner();
    if children.iter().any(|c| weapons.contains(c)) {
        return;
    }

    for (entity, gt, pickup) in pickups.iter() {
        if gt
            .translation()
            .distance_squared(player_transform.translation())
            <= pickup.0 * pickup.0
        {
            commands
                .entity(entity)
                .remove::<(WeaponPickup, RigidBody)>()
                .insert(ChildOf(player_entity));
            return;
        }
    }
}

#[derive(InputAction)]
#[action_output(bool)]
struct Throw;

fn handle_throw(
    _throw: On<Fire<Throw>>,
    mut commands: Commands,
    player: Single<
        (&GlobalTransform, &Children),
        (With<Player>, Without<Dashing>, Without<Finishing>),
    >,
    weapons: Query<Entity, With<Weapon>>,
) {
    let (player_transform, children) = player.into_inner();
    let rotation = player_transform.rotation().to_euler(EulerRot::ZYX).0;
    let mut layers = FriendlyHitbox::collision_layers();
    layers.filters |= Layer::Wall.to_bits();
    for entity in weapons.iter_many(children) {
        commands
            .entity(entity)
            .remove::<(ChildOf, ColliderDisabled)>()
            .insert((
                player_transform.compute_transform(),
                LinearVelocity(Vec2::Y.rotate(Vec2::from_angle(rotation)) * 1000.0),
                RigidBody::Dynamic,
                FriendlyHitbox,
                layers,
                LinearDamping(3.5),
            ))
            .remove::<Sensor>();
    }
}

#[derive(InputAction)]
#[action_output(bool)]
struct Finish;

#[derive(Clone, Copy, Component)]
pub struct Finishing {
    direction: Vec2,
    target: Entity,
}

fn handle_finish(
    _finish: On<Fire<Finish>>,
    mut commands: Commands,
    player: Single<(Entity, &GlobalTransform), (With<Player>, Without<Dashing>)>,
    hurtbox: Single<Entity, With<PlayerHurtbox>>,
    targets: Query<(Entity, &GlobalTransform), With<FinisherTarget>>,
) {
    let dist = 100.0;
    let (player_entity, transform) = player.into_inner();
    let start = transform.translation();
    let end = targets
        .iter()
        .sort_by::<&GlobalTransform>(|a, b| {
            let dist1 = a.translation().xy().distance_squared(start.xy());
            let dist2 = b.translation().xy().distance_squared(start.xy());
            dist1.total_cmp(&dist2)
        })
        .map(|(e, t)| (e, t.translation().xy()))
        .next();
    if let Some((target, end)) = end
        && end.distance_squared(start.xy()) <= dist * dist
    {
        let finishing = Finishing {
            direction: end - start.xy(),
            target,
        };

        let animation = commands
            .animation()
            .insert_tween_here(
                Duration::from_secs_f32(0.1),
                EaseKind::QuarticOut,
                player_entity
                    .into_target()
                    .with(translation(start, end.extend(start.z))),
            )
            .insert(finishing)
            .id();

        commands
            .entity(player_entity)
            .insert(finishing)
            .add_child(animation);
        commands.entity(*hurtbox).insert(ColliderDisabled);
        commands.entity(target).remove::<EnableAttacks>();
    }
}

fn end_finish(
    mut commands: Commands,
    finishing: Query<&Finishing>,
    mut ended: MessageReader<TimeRunnerEnded>,
    player: Single<(Entity, &GlobalTransform), With<Player>>,
    hurtbox: Single<Entity, With<PlayerHurtbox>>,
    mut health: Query<&mut CurrentHealth>,
    mut death_writer: MessageWriter<DeathEvent>,
    mut bits_writer: MessageWriter<BitEvent>,
) -> Result {
    let (player, player_transform) = player.into_inner();
    for ended in ended.read() {
        if ended.is_completed()
            && let Ok(target) = finishing.get(ended.entity)
        {
            commands.entity(ended.entity).despawn();
            commands.entity(player).remove::<Finishing>();
            commands.entity(*hurtbox).remove::<ColliderDisabled>();
            let mut health = health.get_mut(target.target)?;
            health.0 = 0.0;
            death_writer.write(DeathEvent(target.target));
            bits_writer.write(BitEvent {
                direction: target.direction,
                translation: player_transform.translation().xy(),
                bits: 15,
            });
        }
    }
    Ok(())
}
