use bevy::prelude::*;

use crate::player::Player;

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
    camera_transform.translation.x = new_pos.x;
    camera_transform.translation.y = new_pos.y;
}
