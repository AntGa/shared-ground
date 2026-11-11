use bevy_ecs::{component::Component, system::Commands};

#[derive(Component)]
pub struct Player(pub u8);

fn spawn_players(mut commands: Commands) {}
