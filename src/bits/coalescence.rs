use avian2d::prelude::*;
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use std::time::Duration;

pub struct CoalescencePlugin;

impl Plugin for CoalescencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPostUpdate,
            (
                CoalesceTimer::manage_timers,
                coalesce,
                apply_mass,
                attraction,
            )
                .chain()
                .in_set(PhysicsSystems::Last),
        )
        .add_observer(BitMass::insert);
    }
}

const MASS_THRESOLD: f32 = 30.0;

/// Fired when a bit exceeds the mass threshold.
#[derive(EntityEvent)]
pub struct CoalesceEvent(pub Entity);

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
                    .remove::<(CoalesceTimer, ColliderDisabled)>();
            }
        }
    }
}

/// The bit mass is an idealized mass representation, indicating
/// how many bits have "coalesced" on a particular entity.
///
/// The actual avian mass is calculated in terms of this mass.
/// This being immutable makes that easy, as we can synchronize it in
/// an observer.
#[derive(Component, Clone, Copy)]
#[component(immutable)]
pub struct BitMass(pub f32);

impl BitMass {
    fn insert(
        trigger: On<Insert, Self>,
        mut target: Query<(&BitMass, &mut Transform)>,
        mut commands: Commands,
    ) -> Result {
        let (mass, mut transform) = target.get_mut(trigger.entity)?;

        // for now, it's just 1-to-1
        commands.entity(trigger.entity).insert(Mass(mass.0));
        transform.scale = Vec3::splat(1.0 + (1.0 + mass.0).log10());

        if mass.0 >= MASS_THRESOLD {
            commands.entity(trigger.entity).trigger(CoalesceEvent);
        }

        Ok(())
    }
}

#[derive(Component, Default)]
struct TempMass(f32);

fn coalesce(
    bits: Query<(Entity, &BitMass)>,
    collisions: Collisions,
    mut commands: Commands,
    mut has_coalesced: Local<EntityHashSet>,
) {
    has_coalesced.clear();
    for (bit, &BitMass(mass)) in &bits {
        if has_coalesced.contains(&bit) {
            continue;
        }

        for contact_pair in collisions.collisions_with(bit) {
            if !contact_pair.is_touching() {
                continue;
            }

            let other = if contact_pair.collider1 == bit {
                contact_pair.collider2
            } else {
                contact_pair.collider1
            };
            if has_coalesced.contains(&other) {
                continue;
            }

            let Ok((_, &BitMass(other_mass))) = bits.get(other) else {
                continue;
            };

            if mass >= other_mass {
                commands
                    .entity(bit)
                    .entry::<TempMass>()
                    .or_default()
                    .and_modify(move |mut m| m.0 += other_mass);
                has_coalesced.insert(other);
                commands.entity(other).despawn();
            } else {
                commands
                    .entity(other)
                    .entry::<TempMass>()
                    .or_default()
                    .and_modify(move |mut m| m.0 += mass);
                has_coalesced.insert(bit);
                commands.entity(bit).despawn();
            }
        }
    }
}

fn apply_mass(bits: Query<(Entity, &TempMass, &BitMass)>, mut commands: Commands) {
    for (entity, temp_mass, mass) in &bits {
        commands
            .entity(entity)
            .remove::<TempMass>()
            .insert(BitMass(mass.0 + temp_mass.0));
    }
}

fn attraction(
    mut bits: Query<(Entity, &Position, Forces, &BitMass), Without<CoalesceTimer>>,
    other: Query<(Entity, &Position, &BitMass)>,
) {
    for (bit, position, mut forces, mass) in &mut bits {
        let mut impulse = Vec2::ZERO;

        for (other, other_pos, other_mass) in &other {
            if bit == other {
                continue;
            }

            // gmm/r^2
            let mass = mass.0 * other_mass.0;
            let direction = (other_pos.0 - position.0).normalize_or_zero();
            let distance = other_pos.distance(position.0);
            let force = mass / distance.max(0.01);

            impulse += force.min(25.0) * direction;
        }

        forces.apply_force(impulse * 300.0);
    }
}
