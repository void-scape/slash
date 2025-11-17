use bevy::{color::palettes::css::BLUE, prelude::*};

pub mod input;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(input::InputPlugin)
            .add_observer(inject_sprite);
    }
}

/// The player marker component.
#[derive(Component)]
#[require(Transform)]
pub struct Player;

fn inject_sprite(trigger: On<Insert, Player>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(Sprite::from_color(BLUE, Vec2::new(50.0, 50.0)));
}
