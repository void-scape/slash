#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;

pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 720.0;

fn main() {
    let mut app = App::default();

    app.add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (WIDTH as u32, HEIGHT as u32).into(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        bevy_rand::prelude::EntropyPlugin::<bevy_rand::prelude::WyRand>::with_seed(
            69u64.to_le_bytes(),
        ),
    ))
    .add_plugins((
        avian2d::PhysicsPlugins::default().with_length_unit(2.0),
        #[cfg(feature = "debug")]
        avian2d::debug_render::PhysicsDebugPlugin,
    ));

    #[cfg(debug_assertions)]
    app.add_systems(Update, close_on_escape);

    app.add_systems(Startup, camera).run();
}

#[cfg(debug_assertions)]
fn close_on_escape(input: Res<ButtonInput<KeyCode>>, mut writer: MessageWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        writer.write(AppExit::Success);
    }
}

#[cfg(not(debug_assertions))]
pub fn name(_: impl Into<std::borrow::Cow<'static, str>>) -> () {}
#[cfg(debug_assertions)]
pub fn name(name: impl Into<std::borrow::Cow<'static, str>>) -> Name {
    Name::new(name)
}

fn camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
