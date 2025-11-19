use crate::weapon::HitEvent;
use avian2d::prelude::*;
use bevy::{color::palettes::css::GREEN, prelude::*};
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;
use std::f32::consts::PI;

pub mod coalescence;

pub struct BitsPlugin;

impl Plugin for BitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(coalescence::CoalescencePlugin)
            .add_message::<BitEvent>()
            .add_systems(Update, handle_bit_events);
    }
}

const BITS_SPEED: f32 = 500f32;
const INITIAL_SIZE: f32 = 8f32;

#[derive(Component)]
#[require(
    coalescence::BitMass(1.0),
    coalescence::CoalesceTimer,
    RigidBody::Dynamic,
    Collider = Collider::rectangle(INITIAL_SIZE, INITIAL_SIZE),
    CollisionEventsEnabled,
    LinearDamping(4.0),
    ColliderDisabled,
    Sensor,
    Sprite::from_color(GREEN, Vec2::splat(INITIAL_SIZE))
)]
pub struct Bit;

/// Describes the number of bits an attack will produce.
#[derive(Default, Clone, Copy, Component)]
pub struct BitProducer(pub usize);

#[derive(Message)]
pub struct BitEvent {
    pub direction: Vec2,
    pub translation: Vec2,
    pub bits: usize,
}

fn handle_bit_events(
    mut commands: Commands,
    mut reader: MessageReader<BitEvent>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    for event in reader.read() {
        for _ in 0..event.bits {
            let direction = random_direction_in_arc(event.direction, PI * 0.75, &mut rng);
            commands.spawn((
                Bit,
                Transform::from_translation(event.translation.extend(0.0)),
                LinearVelocity(direction * BITS_SPEED * rng.random_range(0.8..1.2)),
            ));
        }
    }
}

pub fn produce_bits(hit: On<HitEvent>, mut writer: MessageWriter<BitEvent>) {
    writer.write(BitEvent {
        direction: hit.target_translation - hit.attacker_translation,
        translation: hit.target_translation,
        bits: hit.bits,
    });
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
