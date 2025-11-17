use crate::weapon::Weapon;
use bevy::{color::palettes::css::BLUE, prelude::*, window::PrimaryWindow};

pub mod input;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(input::InputPlugin)
            .add_systems(Update, move_weapon_with_mouse_input);
    }
}

/// The player marker component.
#[derive(Component)]
#[require(
    Transform,
    Sprite::from_color(BLUE, Vec2::new(20.0, 20.0)),
    Name::new("Player")
)]
pub struct Player;

fn move_weapon_with_mouse_input(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    player: Single<(&GlobalTransform, &Children), With<Player>>,
    mut weapons: Query<&mut Transform, With<Weapon>>,
) {
    let (camera, camera_transform) = camera.into_inner();
    if let Some(Ok(cursor_translation)) = window
        .cursor_position()
        .map(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        let (player_transform, children) = player.into_inner();
        let mut iter = weapons.iter_many_mut(children);
        if let Some(mut transform) = iter.fetch_next() {
            let target = cursor_translation - player_transform.translation().xy();
            let normalized_translation = target.normalize_or_zero();

            if normalized_translation != Vec2::ZERO {
                let angle = Vec2::Y.angle_to(normalized_translation);
                transform.rotation = Quat::from_rotation_z(angle);

                let newt = normalized_translation * 15.0;
                transform.translation.x = newt.x;
                transform.translation.y = newt.y;
            }
        }
    }
}
