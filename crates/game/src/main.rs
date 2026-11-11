use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_color(
        Color::srgb(0.9, 0.3, 0.3),
        Vec2::new(32.0, 32.0),
    ));
}
