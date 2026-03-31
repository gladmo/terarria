use bevy::prelude::*;
use std::collections::HashMap;

use crate::world::TileType;

/// Holds pre-loaded texture handles for all tile types and the player.
#[derive(Resource)]
pub struct GameTextures {
    pub player: Handle<Image>,
    pub tiles: HashMap<TileType, Handle<Image>>,
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_game_textures);
    }
}

fn load_game_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut tiles = HashMap::new();

    let tile_paths = [
        (TileType::Grass, "textures/tile_grass.png"),
        (TileType::Dirt, "textures/tile_dirt.png"),
        (TileType::Stone, "textures/tile_stone.png"),
        (TileType::Bedrock, "textures/tile_bedrock.png"),
        (TileType::Sand, "textures/tile_sand.png"),
        (TileType::Wood, "textures/tile_wood.png"),
        (TileType::Leaves, "textures/tile_leaves.png"),
    ];

    for (tile_type, path) in &tile_paths {
        tiles.insert(*tile_type, asset_server.load(*path));
    }

    commands.insert_resource(GameTextures {
        player: asset_server.load("textures/player.png"),
        tiles,
    });
}
