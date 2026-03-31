use bevy::prelude::*;

use crate::player::Player;
use crate::{TILE_SIZE, WORLD_HEIGHT, WORLD_WIDTH};

/// Marker for the main 2D camera.
#[derive(Component)]
pub struct MainCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, follow_player);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

fn follow_player(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    windows: Query<&Window>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let target = player_transform.translation.truncate();
    let current = camera_transform.translation.truncate();

    // Smooth camera follow using actual delta time
    let lerp_speed = 5.0;
    let new_pos = current.lerp(target, (lerp_speed * time.delta_secs()).min(1.0));

    // Clamp camera so it never shows outside the world boundaries
    let world_w = WORLD_WIDTH as f32 * TILE_SIZE;
    let world_h = WORLD_HEIGHT as f32 * TILE_SIZE;

    let (half_vp_w, half_vp_h) = if let Ok(window) = windows.single() {
        (window.width() * 0.5, window.height() * 0.5)
    } else {
        (640.0, 360.0)
    };

    let clamped_x = new_pos
        .x
        .clamp(half_vp_w, (world_w - half_vp_w).max(half_vp_w));
    // Bevy Y is positive up; world rows go down so camera Y is negative
    let clamped_y = new_pos
        .y
        .clamp((-world_h + half_vp_h).min(-half_vp_h), -half_vp_h);

    camera_transform.translation.x = clamped_x;
    camera_transform.translation.y = clamped_y;
}
