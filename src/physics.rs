use avian2d::prelude::{LinearVelocity, PhysicsSystems};
use bevy::prelude::*;
use bevy_tween::{BevyTweenRegisterSystems, component_tween_system, prelude::Interpolator};

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedPostUpdate,
        apply_acceleration
            .after(PhysicsSystems::StepSimulation)
            .in_set(CustomPhysicsSystems::Acceleration),
    )
    .add_tween_systems((
        component_tween_system::<AccelerationTween>(),
        component_tween_system::<VelocityTween>(),
    ));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum CustomPhysicsSystems {
    Acceleration,
}

#[derive(Default, Component)]
pub struct Acceleration(pub Vec2);

fn apply_acceleration(mut velocities: Query<(&mut LinearVelocity, &mut Acceleration)>) {
    for (mut velocity, mut acceleration) in velocities.iter_mut() {
        velocity.0 += acceleration.0;
        acceleration.0 = Vec2::ZERO;
    }
}

#[derive(Component)]
pub struct AccelerationTween {
    start: Vec2,
    end: Vec2,
}

pub fn acceleration(start: Vec2, end: Vec2) -> AccelerationTween {
    AccelerationTween { start, end }
}

impl Interpolator for AccelerationTween {
    type Item = Acceleration;
    fn interpolate(
        &self,
        item: &mut Self::Item,
        value: bevy_tween::interpolate::CurrentValue,
        _: bevy_tween::interpolate::PreviousValue,
    ) {
        item.0 = self.start.lerp(self.end, value)
    }
}

#[derive(Component)]
pub struct VelocityTween {
    start: Vec2,
    end: Vec2,
}

pub fn velocity(start: Vec2, end: Vec2) -> VelocityTween {
    VelocityTween { start, end }
}

impl Interpolator for VelocityTween {
    type Item = LinearVelocity;
    fn interpolate(
        &self,
        item: &mut Self::Item,
        value: bevy_tween::interpolate::CurrentValue,
        _: bevy_tween::interpolate::PreviousValue,
    ) {
        item.0 = self.start.lerp(self.end, value)
    }
}
