use bevy::prelude::*;
use std::collections::HashMap;

use crate::assets::GameTextures;
use crate::camera::MainCamera;
use crate::world::{tile_to_world, GameWorld, TileType};
use crate::{TILE_SIZE, WORLD_HEIGHT, WORLD_WIDTH};
/// Tracks all currently-spawned tile entities by their tile coordinates.
#[derive(Resource, Default)]
pub struct RenderedTiles(pub HashMap<(i32, i32), Entity>);

/// Render buffer (extra tiles rendered beyond the visible viewport).
const BUFFER: i32 = 4;

/// Fallback tint shown for any tile type that has no texture loaded yet.
const MISSING_TEXTURE_COLOR: Color = Color::srgb(1.0, 0.0, 1.0);

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderedTiles>()
            .add_systems(Update, update_tile_rendering);
    }
}

// ---------------------------------------------------------------------------
// Tile rendering system
// ---------------------------------------------------------------------------

fn update_tile_rendering(
    mut commands: Commands,
    mut rendered: ResMut<RenderedTiles>,
    mut world: ResMut<GameWorld>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    windows: Query<&Window>,
    textures: Res<GameTextures>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };

    // Compute visible tile range from camera position
    let cam_pos = camera_transform.translation().truncate();
    let half_w = window.width() * 0.5 + BUFFER as f32 * TILE_SIZE;
    let half_h = window.height() * 0.5 + BUFFER as f32 * TILE_SIZE;

    let min_x = ((cam_pos.x - half_w) / TILE_SIZE).floor() as i32;
    let max_x = ((cam_pos.x + half_w) / TILE_SIZE).ceil() as i32;
    // Note: y-axis is flipped (positive y = up in Bevy, row 0 = top of world)
    let min_y = ((-cam_pos.y - half_h) / TILE_SIZE).floor() as i32;
    let max_y = ((-cam_pos.y + half_h) / TILE_SIZE).ceil() as i32;

    let min_x = min_x.clamp(0, WORLD_WIDTH as i32 - 1);
    let max_x = max_x.clamp(0, WORLD_WIDTH as i32 - 1);
    let min_y = min_y.clamp(0, WORLD_HEIGHT as i32 - 1);
    let max_y = max_y.clamp(0, WORLD_HEIGHT as i32 - 1);

    // Handle dirty tiles: remove old entity so they can be respawned
    let dirty: Vec<(i32, i32)> = world.dirty.drain().collect();
    for (tx, ty) in dirty {
        if let Some(entity) = rendered.0.remove(&(tx, ty)) {
            commands.entity(entity).despawn();
        }
    }

    // Despawn tiles that are out of view
    let out_of_view: Vec<(i32, i32)> = rendered
        .0
        .keys()
        .copied()
        .filter(|&(tx, ty)| tx < min_x || tx > max_x || ty < min_y || ty > max_y)
        .collect();
    for key in out_of_view {
        if let Some(entity) = rendered.0.remove(&key) {
            commands.entity(entity).despawn();
        }
    }

    // Spawn tiles that are newly in view.
    // The `rendered` map is the authoritative list of spawned entities; tiles already
    // in it are skipped, so texture lookup only happens when a tile first enters view
    // or is re-spawned after being mined/placed (via the dirty-tile path above).
    for ty in min_y..=max_y {
        for tx in min_x..=max_x {
            if rendered.0.contains_key(&(tx, ty)) {
                continue;
            }
            let tile = world.get(tx, ty);
            if tile == TileType::Air {
                continue; // Air — no sprite needed
            }
            let pos = tile_to_world(tx, ty);
            let sprite = if let Some(handle) = textures.tiles.get(&tile) {
                // Use the loaded texture for this tile type
                Sprite {
                    image: handle.clone(),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                }
            } else {
                // Fallback: solid tint (should not occur for known tile types)
                Sprite::from_color(
                    tile.color().unwrap_or(MISSING_TEXTURE_COLOR),
                    Vec2::splat(TILE_SIZE - 0.5),
                )
            };
            let entity = commands
                .spawn((sprite, Transform::from_translation(pos.extend(1.0))))
                .id();
            rendered.0.insert((tx, ty), entity);
        }
    }
}
