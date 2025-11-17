use crate::{
    enemy::{Enemy, steering::SteeringTarget},
    health::{EnemyHurtbox, Health},
    player::Player,
    weapon::{Dagger, HitEvent},
};
use avian2d::prelude::{CollisionEventsEnabled, LinearDamping, LinearVelocity, RigidBody};
use bevy::{color::palettes::css::GREEN, prelude::*};
use rand::Rng;
use std::f32::consts::PI;

pub mod coalescence;

pub struct BitsPlugin;

impl Plugin for BitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(coalescence::CoalescencePlugin)
            .add_observer(observe_hit)
            .add_observer(spawn_enemy);
    }
}

const BITS_SPEED: f32 = 500f32;
const INITIAL_SIZE: f32 = 8f32;

fn spawn_enemy(
    gibblet: On<coalescence::CoalesceEvent>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    player: Query<Entity, With<Player>>,
) -> Result {
    commands.entity(gibblet.entity).despawn();
    let transform = transforms.get(gibblet.entity)?;
    let mut entity = commands.spawn((
        Enemy,
        Transform::from_translation(transform.translation()),
        Health::new(4.0),
        children![
            (
                EnemyHurtbox,
                avian2d::prelude::Collider::rectangle(25.0, 25.0),
                Transform::default(),
            ),
            (Dagger, Transform::from_xyz(0.0, 15.0, 0.0))
        ],
    ));
    if let Ok(player) = player.single() {
        entity.insert(SteeringTarget(player));
    }
    Ok(())
}

#[derive(Component)]
#[require(
    coalescence::BitMass(1.0),
    coalescence::CoalesceTimer,
    Transform,
    LinearDamping(4.0),
    RigidBody::Dynamic,
    CollisionEventsEnabled,
    Sprite::from_color(GREEN, Vec2::splat(INITIAL_SIZE))
)]
pub struct Bit;

/// Describes the number of bits an attack will produce.
#[derive(Component)]
pub struct BitProducer(pub usize);

fn observe_hit(
    trigger: On<HitEvent>,
    transforms: Query<&GlobalTransform>,
    weapon: Query<&BitProducer>,
    mut commands: Commands,
) -> Result {
    let Ok(bit_producer) = weapon.get(trigger.weapon) else {
        return Ok(());
    };
    let attacker_trans = transforms.get(trigger.attacker)?.compute_transform();
    let target_trans = transforms.get(trigger.target)?.compute_transform();

    let direction = target_trans.translation - attacker_trans.translation;
    let mut rng = rand::rng();

    for _ in 0..bit_producer.0 {
        let direction = random_direction_in_arc(direction.xy(), PI * 0.75, &mut rng);

        commands.spawn((
            Bit,
            Transform::from_translation(target_trans.translation),
            LinearVelocity(direction * BITS_SPEED * rng.random_range(0.8..1.2)),
        ));
    }

    Ok(())
}

/// Returns a random unit vector whose direction lies within an arc of `arc_radians`
/// centered around the given direction vector.
///
/// `dir` does not have to be normalized; this function normalizes it internally.
/// `arc_radians` is the full width of the arc (e.g. PI/4 is ±PI/8 around dir).
fn random_direction_in_arc(dir: Vec2, arc_radians: f32, rng: &mut impl Rng) -> Vec2 {
    // Normalize the input direction
    let dir = dir.normalize_or_zero();

    // Convert direction to angle
    let base_angle = dir.y.atan2(dir.x); // atan2(y, x)

    // Half-width of the arc
    let half_arc = arc_radians * 0.5;

    // Sample angle uniformly in [base_angle - half_arc, base_angle + half_arc]
    let offset: f32 = rng.random_range(-half_arc..=half_arc);
    let final_angle = base_angle + offset;

    // Convert back to a unit vector
    Vec2 {
        x: final_angle.cos(),
        y: final_angle.sin(),
    }
}
