use std::time::Duration;

use crate::{Avian, bits::INITIAL_SIZE};
use avian2d::prelude::{Collider, CollisionStart, PhysicsSystems, Sensor};
use bevy::{ecs::entity::EntityHashSet, prelude::*};

pub struct CoalescencePlugin;

impl Plugin for CoalescencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Avian,
            (CoalesceTimer::manage_timers, coalesce)
                .chain()
                .in_set(PhysicsSystems::Last),
        );
    }
}

/// We'll wait a moment to start checking for coalescence.
#[derive(Component)]
pub struct CoalesceTimer(pub Timer);

impl Default for CoalesceTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(500), TimerMode::Once))
    }
}

impl CoalesceTimer {
    fn manage_timers(
        mut timers: Query<(Entity, &mut CoalesceTimer)>,
        time: Res<Time>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (entity, mut timer) in &mut timers {
            if timer.0.tick(delta).just_finished() {
                commands
                    .entity(entity)
                    .remove::<CoalesceTimer>()
                    .insert((Sensor, Collider::rectangle(INITIAL_SIZE, INITIAL_SIZE)));
            }
        }
    }
}

#[derive(Component)]
pub struct BitMass(pub f32);

fn coalesce(
    bits: Query<&BitMass>,
    mut collisions: MessageReader<CollisionStart>,
    mut commands: Commands,
) {
    let mut handled = EntityHashSet::default();

    for CollisionStart {
        collider1,
        collider2,
        ..
    } in collisions.read()
    {
        if handled.contains(collider1) || handled.contains(collider2) {
            continue;
        }

        let Ok(a) = bits.get(*collider1) else {
            continue;
        };
        let Ok(b) = bits.get(*collider2) else {
            continue;
        };

        let combined_mass = a.0 + b.0;
        commands.entity(*collider1).insert(BitMass(combined_mass));
        commands.entity(*collider2).despawn();
        handled.insert(*collider2);
    }
}
