use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::player::Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;
