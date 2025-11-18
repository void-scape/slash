use super::Player;
use crate::{
    Layer,
    bits::BitProducer,
    health::FriendlyHitbox,
    physics::{Acceleration, velocity},
    player::OrientationMethod,
    player::PlayerHurtbox,
    weapon::{TriggerWeapon, Weapon, WeaponPickup},
};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_tween::{
    bevy_time_runner::TimeRunnerEnded,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use std::time::Duration;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>()
            .add_systems(Update, end_dash)
            .add_observer(inject_bindings)
            .add_observer(apply_movement)
            .add_observer(stop_movement)
            .add_observer(handle_attack)
            .add_observer(handle_dash)
            .add_observer(handle_pick_up)
            .add_observer(handle_throw)
            .add_observer(handle_aim);
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
            bindings![KeyCode::KeyE, GamepadButton::South],
        ),
        (
            Action::<Throw>::new(),
            Press::default(),
            bindings![KeyCode::KeyC, GamepadButton::LeftTrigger2, GamepadButton::North],
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
            &Acceleration,
        ),
        With<Player>,
    >,
    is_dashing: Query<&Dashing>,
) {
    let (mut retained, mut velocity, max_speed, acceleration) = player.into_inner();
    retained.0 = movement.value;
    if is_dashing.is_empty() && acceleration.0.length_squared() <= 20.0 * 20.0 {
        velocity.0 = movement.value * max_speed.0;
    }
}

fn stop_movement(
    _movement: On<Complete<Move>>,
    mut player: Single<&mut LinearVelocity, With<Player>>,
    is_dashing: Query<&Dashing>,
) {
    if is_dashing.is_empty() {
        player.0 = Vec2::ZERO;
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
    player: Single<(Entity, &RetainedMove)>,
    hurtbox: Single<Entity, With<PlayerHurtbox>>,
) {
    let (player_entity, last_input) = player.into_inner();
    let start = last_input.0 * 4_000.0;
    let end = Vec2::ZERO;

    let animation = commands
        .animation()
        .insert_tween_here(
            Duration::from_secs_f32(0.2),
            EaseKind::QuarticOut,
            player_entity.into_target().with(velocity(start, end)),
        )
        .insert(Dashing)
        .id();

    commands
        .entity(player_entity)
        .remove::<(Acceleration, LinearDamping, MaxLinearSpeed)>()
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
            commands.entity(*player).insert((
                Acceleration::default(),
                Player::LINEAR_DAMPING,
                Player::MAX_SPEED,
            ));
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
    player: Single<(Entity, &GlobalTransform, &Children), With<Player>>,
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
                .insert((ChildOf(player_entity), BitProducer(35)));
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
    player: Single<(&GlobalTransform, &Children), With<Player>>,
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
