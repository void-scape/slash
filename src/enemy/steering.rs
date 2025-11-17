use crate::Avian;
use avian2d::prelude::{LinearVelocity, Physics, PhysicsSystems};
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Avian,
            (steer_homing, apply_heading_velocity)
                .chain()
                .before(PhysicsSystems::StepSimulation),
        );
    }
}

/// Facilitates "steering" behavior, giving enemies a feeling of momentum.
#[derive(Debug, PartialEq, Clone, Copy, Component)]
#[require(TurnSpeed)]
pub struct Heading {
    pub direction: f32,
    pub speed: f32,
}

impl Default for Heading {
    fn default() -> Self {
        Self {
            direction: 0.0,
            speed: 50.0,
        }
    }
}

#[derive(Component)]
#[require(Heading)]
pub struct SteeringTarget(pub Entity);

/// The turning speed for headings.
#[derive(Clone, Copy, PartialEq, Component)]
pub struct TurnSpeed(pub f32);

impl Default for TurnSpeed {
    fn default() -> Self {
        Self(300.0)
    }
}

impl Heading {
    pub fn steer_towards(&mut self, time: &Time<Physics>, turn_speed: f32, from: Vec2, to: Vec2) {
        let desired_direction = (to - from).normalize();
        let desired_angle = desired_direction.y.atan2(desired_direction.x);

        let mut angle_diff = (desired_angle - self.direction) % TAU;
        if angle_diff > PI {
            angle_diff = PI - angle_diff;
        } else if angle_diff < -PI {
            angle_diff = -PI - angle_diff;
        };

        self.direction += angle_diff * turn_speed * time.delta_secs();
        self.direction %= TAU;
    }
}

fn steer_homing(
    mut homing: Query<(
        Entity,
        &GlobalTransform,
        &SteeringTarget,
        &mut Heading,
        &TurnSpeed,
    )>,
    targets: Query<&GlobalTransform>,
    time: Res<Time<Physics>>,
    mut commands: Commands,
) {
    let delta = time.delta_secs();

    for (entity, transform, target, mut heading, turn_speed) in homing.iter_mut() {
        let Ok(target) = targets.get(target.0) else {
            commands.entity(entity).remove::<SteeringTarget>();
            continue;
        };

        let transform = transform.compute_transform();
        let target = target.compute_transform();

        heading.steer_towards(
            &time,
            turn_speed.0 * delta,
            transform.translation.xy(),
            target.translation.xy(),
        );
    }
}

fn apply_heading_velocity(mut homing: Query<(&mut Transform, &Heading, &mut LinearVelocity)>) {
    for (mut transform, heading, mut velocity) in homing.iter_mut() {
        velocity.0.x = heading.speed * heading.direction.cos();
        velocity.0.y = heading.speed * heading.direction.sin();

        let new_rotation = Quat::from_rotation_z(heading.direction - PI / 2.0);
        transform.rotation = new_rotation;
    }
}
