use bevy::{color::palettes::css::BLUE, prelude::*};

pub mod input;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(input::InputPlugin);
    }
}

/// The player marker component.
#[derive(Component)]
#[require(Transform, Sprite::from_color(BLUE, Vec2::new(20.0, 20.0)))]
pub struct Player;
