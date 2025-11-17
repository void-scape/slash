use super::Player;
use crate::{
    bits::BitProducer,
    health::FriendlyHitbox,
    weapon::{TriggerWeapon, Weapon, WeaponPickup},
};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>()
            .add_observer(inject_bindings)
            .add_observer(apply_movement)
            .add_observer(handle_attack)
            .add_observer(handle_pick_up)
            .add_observer(handle_throw);
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
            Action::<Attack>::new(),
            Press::default(),
            bindings![KeyCode::Space, GamepadButton::West, MouseButton::Left],
        ),
        (
            Action::<PickUp>::new(),
            Press::default(),
            bindings![KeyCode::KeyE, GamepadButton::South],
        ),
        (
            Action::<Throw>::new(),
            Press::default(),
            bindings![KeyCode::KeyC, GamepadButton::North],
        ),
    ]));
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;

fn apply_movement(movement: On<Fire<Move>>, mut player: Single<&mut Transform, With<Player>>) {
    player.translation += movement.value.extend(0.0) * 2.5;
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
    for entity in weapons.iter_many(children) {
        commands
            .entity(entity)
            .remove::<(ChildOf, ColliderDisabled)>()
            .insert((
                player_transform.compute_transform(),
                LinearVelocity(Vec2::Y.rotate(Vec2::from_angle(rotation)) * 1000.0),
                RigidBody::Dynamic,
                FriendlyHitbox,
                FriendlyHitbox::collision_layers(),
                LinearDamping(3.5),
            ));
    }
}
