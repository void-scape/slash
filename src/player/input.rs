use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::Player;
use crate::weapon::TriggerWeapon;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>()
            .add_observer(inject_bindings)
            .add_observer(apply_movement)
            .add_observer(handle_attack);
    }
}

fn inject_bindings(trigger: On<Insert, Player>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(actions!(Player[
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
                bindings![KeyCode::Space, GamepadButton::West],
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
