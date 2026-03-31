use bevy::prelude::*;
use bevy::window::WindowResolution;

mod assets;
mod camera;
mod player;
mod rendering;
mod ui;
mod world;

pub const TILE_SIZE: f32 = 16.0;
pub const WORLD_WIDTH: usize = 400;
pub const WORLD_HEIGHT: usize = 200;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Terarria".to_string(),
                    resolution: WindowResolution::new(1280, 720),
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(ClearColor(Color::srgb(0.38, 0.62, 0.88)))
        .add_plugins(world::WorldPlugin)
        .add_plugins(assets::AssetsPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(rendering::RenderingPlugin)
        .add_plugins(ui::UiPlugin)
        .run();
}
