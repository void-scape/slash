use avian2d::prelude::{LinearVelocity, PhysicsSystems};
use bevy::prelude::*;

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPostUpdate,
            (
                (target_vector, seperation_vector),
                apply_force_vectors,
                orient_with_velocity,
            )
                .chain()
                .after(PhysicsSystems::StepSimulation),
        );
    }
}

#[derive(Component)]
#[require(TargetVector, SeperationVector)]
pub struct SteerTarget(pub Entity);

#[derive(Debug, Default, PartialEq, Clone, Copy, Component)]
pub struct TargetVector(pub Vec2);

#[derive(Default, Component)]
pub struct SeperationVector(pub Vec2);

fn target_vector(
    mut steering: Query<(&mut TargetVector, &GlobalTransform, &SteerTarget)>,
    targets: Query<&GlobalTransform>,
) -> Result {
    for (mut target_vector, gt, steer_target) in steering.iter_mut() {
        if let Ok(target) = targets.get(steer_target.0) {
            let diff = target.translation().xy() - gt.translation().xy();
            let new_vector = diff.normalize_or_zero();
            if new_vector != Vec2::ZERO {
                target_vector.0 = new_vector;
            }
        }
    }
    Ok(())
}

fn seperation_vector(
    steering: Query<(Entity, &GlobalTransform), With<SeperationVector>>,
    mut seperation_vectors: Query<&mut SeperationVector>,
) -> Result {
    let radius = 100.0;
    let radius_squared = radius * radius;
    for (target_entity, gt) in steering.iter() {
        let target_translation = gt.translation().xy();
        let mut seperation_force = Vec2::ZERO;
        for (neighbor, neighbor_gt) in steering.iter() {
            if neighbor == target_entity {
                continue;
            }
            let neighbor_translation = neighbor_gt.translation().xy();
            if target_translation.distance_squared(neighbor_translation) <= radius_squared {
                let s = (target_translation - neighbor_translation).normalize_or_zero();
                seperation_force += s;
            }
        }
        let mut seperation_vector = seperation_vectors.get_mut(target_entity)?;
        seperation_vector.0 = seperation_force.normalize_or_zero();
    }
    Ok(())
}

fn apply_force_vectors(
    time: Res<Time>,
    mut steering: Query<(&mut LinearVelocity, &TargetVector, &SeperationVector)>,
) {
    let impulse = 20.0;
    let delta = time.delta_secs();
    for (mut velocity, target, seperation) in steering.iter_mut() {
        let force = target.0 * 2.0 + seperation.0 * 1.2;
        let acceleration = force * impulse;

        velocity.0 += acceleration * delta;
        velocity.0 = velocity.0.clamp_length_max(50.0);
    }
}

fn orient_with_velocity(mut steering: Query<(&mut Transform, &LinearVelocity), With<SteerTarget>>) {
    for (mut transform, velocity) in steering.iter_mut() {
        let vel = velocity.0.normalize_or_zero();
        if vel != Vec2::ZERO {
            let angle = Vec2::Y.angle_to(velocity.0.normalize());
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}
