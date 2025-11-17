use avian2d::prelude::RigidBody;
use bevy::{color::palettes::css::RED, prelude::*};

pub mod steering;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(steering::SteeringPlugin)
            .add_observer(inject_sprite);
    }
}

#[derive(Component)]
#[require(Transform, RigidBody::Dynamic)]
pub struct Enemy;

fn inject_sprite(trigger: On<Insert, Enemy>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(Sprite::from_color(RED, Vec2::new(50.0, 50.0)));
}
