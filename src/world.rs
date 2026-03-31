use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;

use crate::{TILE_SIZE, WORLD_HEIGHT, WORLD_WIDTH};

/// Types of tiles that can exist in the world.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum TileType {
    Air,
    #[default]
    Dirt,
    Grass,
    Stone,
    Bedrock,
    Sand,
    Wood,
    Leaves,
}

impl TileType {
    /// Returns the display color for this tile, or `None` for Air.
    pub fn color(self) -> Option<Color> {
        match self {
            TileType::Air => None,
            TileType::Grass => Some(Color::srgb(0.22, 0.62, 0.12)),
            TileType::Dirt => Some(Color::srgb(0.52, 0.36, 0.16)),
            TileType::Stone => Some(Color::srgb(0.48, 0.48, 0.50)),
            TileType::Bedrock => Some(Color::srgb(0.18, 0.18, 0.20)),
            TileType::Sand => Some(Color::srgb(0.88, 0.82, 0.50)),
            TileType::Wood => Some(Color::srgb(0.44, 0.30, 0.14)),
            TileType::Leaves => Some(Color::srgb(0.12, 0.52, 0.10)),
        }
    }

    /// Returns `true` if this tile blocks movement.
    pub fn is_solid(self) -> bool {
        self != TileType::Air
    }
}

/// The game world: a 2D grid of tiles.
#[derive(Resource)]
pub struct GameWorld {
    pub tiles: Vec<TileType>,
    pub dirty: HashSet<(i32, i32)>,
    pub surface_heights: Vec<usize>,
}

impl GameWorld {
    pub fn new() -> Self {
        Self {
            tiles: vec![TileType::Air; WORLD_WIDTH * WORLD_HEIGHT],
            dirty: HashSet::new(),
            surface_heights: vec![WORLD_HEIGHT / 2; WORLD_WIDTH],
        }
    }

    pub fn get(&self, x: i32, y: i32) -> TileType {
        if x < 0 || y < 0 || x >= WORLD_WIDTH as i32 || y >= WORLD_HEIGHT as i32 {
            return TileType::Bedrock;
        }
        self.tiles[y as usize * WORLD_WIDTH + x as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, tile: TileType) {
        if x < 0 || y < 0 || x >= WORLD_WIDTH as i32 || y >= WORLD_HEIGHT as i32 {
            return;
        }
        self.tiles[y as usize * WORLD_WIDTH + x as usize] = tile;
        self.dirty.insert((x, y));
    }

    /// Returns the player spawn position in Bevy world coordinates.
    pub fn spawn_position(&self) -> Vec2 {
        let tx = WORLD_WIDTH / 2;
        let ty = self.surface_heights[tx];
        tile_to_world(tx as i32, ty as i32 - 2)
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts tile coordinates to Bevy world-space center position.
pub fn tile_to_world(tx: i32, ty: i32) -> Vec2 {
    Vec2::new(
        tx as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        -(ty as f32 * TILE_SIZE + TILE_SIZE * 0.5),
    )
}

/// Converts a Bevy world-space position to tile coordinates.
pub fn world_to_tile(pos: Vec2) -> (i32, i32) {
    let tx = (pos.x / TILE_SIZE).floor() as i32;
    let ty = ((-pos.y) / TILE_SIZE).floor() as i32;
    (tx, ty)
}

// ---------------------------------------------------------------------------
// Simple value noise helpers for terrain generation
// ---------------------------------------------------------------------------

fn hash2(x: i64, seed: u64) -> f64 {
    let mut h: u64 = seed.wrapping_add(x as u64 * 0x9e3779b97f4a7c15);
    h ^= h >> 30;
    h = h.wrapping_mul(0xbf58476d1ce4e5b9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94d049bb133111eb);
    h ^= h >> 31;
    (h & 0x00FF_FFFF) as f64 / 0x00FF_FFFF as f64
}

fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Smooth 1-D value noise in the range [0, 1].
fn smooth_noise(x: f64, seed: u64) -> f64 {
    let x0 = x.floor() as i64;
    let t = smoothstep(x - x.floor());
    hash2(x0, seed) * (1.0 - t) + hash2(x0 + 1, seed) * t
}

/// Fractional Brownian motion layering of smooth_noise.
fn fbm(x: f64, seed: u64, octaves: u32) -> f64 {
    let mut value = 0.0f64;
    let mut amplitude = 0.5f64;
    let mut frequency = 1.0f64;
    let mut max_value = 0.0f64;
    for i in 0..octaves {
        value += smooth_noise(x * frequency, seed.wrapping_add(i as u64 * 6364136223846793005)) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    value / max_value
}

// ---------------------------------------------------------------------------
// Cave generation
// ---------------------------------------------------------------------------

fn cave_noise(x: f64, y: f64, seed: u64) -> f64 {
    // 2D value noise via two 1D hashes
    let x0 = x.floor() as i64;
    let y0 = y.floor() as i64;
    let tx = smoothstep(x - x.floor());
    let ty = smoothstep(y - y.floor());
    let s00 = hash2(x0.wrapping_mul(73856093) ^ y0.wrapping_mul(19349663), seed);
    let s10 = hash2((x0+1).wrapping_mul(73856093) ^ y0.wrapping_mul(19349663), seed);
    let s01 = hash2(x0.wrapping_mul(73856093) ^ (y0+1).wrapping_mul(19349663), seed);
    let s11 = hash2((x0+1).wrapping_mul(73856093) ^ (y0+1).wrapping_mul(19349663), seed);
    let top = s00 * (1.0 - tx) + s10 * tx;
    let bot = s01 * (1.0 - tx) + s11 * tx;
    top * (1.0 - ty) + bot * ty
}

// ---------------------------------------------------------------------------
// World generation
// ---------------------------------------------------------------------------

fn generate_world(world: &mut GameWorld) {
    let seed: u64 = rand::rng().random();

    let surface_base = (WORLD_HEIGHT as f64 * 0.35) as usize;
    let surface_amplitude = 25.0f64;

    // Generate height map
    for x in 0..WORLD_WIDTH {
        let nx = x as f64 * 0.008;
        let height_frac = fbm(nx, seed, 5);
        let surface_y = (surface_base as f64 + (height_frac - 0.5) * surface_amplitude * 2.0)
            .clamp(5.0, WORLD_HEIGHT as f64 - 10.0) as usize;
        world.surface_heights[x] = surface_y;
    }

    // Smooth surface heights slightly
    let heights = world.surface_heights.clone();
    for x in 1..WORLD_WIDTH - 1 {
        world.surface_heights[x] = (heights[x - 1] + heights[x] + heights[x + 1]) / 3;
    }

    // Fill tiles
    for x in 0..WORLD_WIDTH {
        let surface_y = world.surface_heights[x];
        for y in 0..WORLD_HEIGHT {
            let tile = if y < surface_y {
                TileType::Air
            } else if y == surface_y {
                TileType::Grass
            } else if y <= surface_y + 4 {
                TileType::Dirt
            } else if y >= WORLD_HEIGHT - 3 {
                TileType::Bedrock
            } else {
                TileType::Stone
            };
            world.tiles[y * WORLD_WIDTH + x] = tile;
        }
    }

    // Carve caves (only below dirt layer, above bedrock)
    let cave_threshold = 0.78;
    let cave_seed = seed.wrapping_add(1);
    for x in 0..WORLD_WIDTH {
        let surface_y = world.surface_heights[x];
        for y in (surface_y + 6)..(WORLD_HEIGHT - 4) {
            let cx = x as f64 * 0.05;
            let cy = y as f64 * 0.05;
            let n1 = cave_noise(cx, cy, cave_seed);
            let n2 = cave_noise(cx + 100.0, cy + 100.0, cave_seed.wrapping_add(12345));
            if n1 > cave_threshold && n2 > cave_threshold {
                world.tiles[y * WORLD_WIDTH + x] = TileType::Air;
            }
        }
    }

    // Add small sand patches near surface
    let sand_seed = seed.wrapping_add(2);
    for x in 0..WORLD_WIDTH {
        let surface_y = world.surface_heights[x];
        let nx = x as f64 * 0.03;
        let sand_chance = smooth_noise(nx, sand_seed);
        if sand_chance > 0.72 {
            // Place a small sand patch
            let patch_size = ((sand_chance - 0.72) * 20.0) as usize + 2;
            for dy in 0..patch_size.min(4) {
                let ty = surface_y + dy;
                if ty < WORLD_HEIGHT - 3 {
                    world.tiles[ty * WORLD_WIDTH + x] = TileType::Sand;
                }
            }
        }
    }

    // Generate some trees
    let tree_seed = seed.wrapping_add(3);
    let mut x = 5usize;
    while x < WORLD_WIDTH - 5 {
        let nx = x as f64 * 0.2;
        let chance = smooth_noise(nx, tree_seed);
        if chance > 0.65 {
            let surface_y = world.surface_heights[x];
            if world.tiles[surface_y * WORLD_WIDTH + x] == TileType::Grass {
                let tree_height = 4 + (chance * 4.0) as usize;
                for h in 1..=tree_height {
                    if surface_y >= h {
                        world.tiles[(surface_y - h) * WORLD_WIDTH + x] = TileType::Wood;
                    }
                }
                // Leaves
                let top = surface_y.saturating_sub(tree_height);
                for ly in 0..4usize {
                    let lwidth: i32 = 2 - ly as i32 / 2;
                    for lx in -lwidth..=lwidth {
                        let wx = x as i32 + lx;
                        let wy = top as i32 + ly as i32 - 1;
                        if wx >= 0 && wx < WORLD_WIDTH as i32 && wy >= 0 && wy < WORLD_HEIGHT as i32
                            && world.tiles[wy as usize * WORLD_WIDTH + wx as usize] == TileType::Air
                        {
                            world.tiles[wy as usize * WORLD_WIDTH + wx as usize] = TileType::Leaves;
                        }
                    }
                }
            }
            x += 6;
        } else {
            x += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let mut world = GameWorld::new();
        generate_world(&mut world);
        app.insert_resource(world);
    }
}
