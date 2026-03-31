use bevy::prelude::*;

use crate::world::{world_to_tile, GameWorld, TileType};
use crate::{TILE_SIZE, WORLD_HEIGHT, WORLD_WIDTH};

/// Marks the player entity.
#[derive(Component)]
pub struct Player;

/// Current velocity of an entity (pixels per second).
#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

/// Physics constants.
const GRAVITY: f32 = -800.0;
const MOVE_SPEED: f32 = 180.0;
const JUMP_SPEED: f32 = 400.0;
const MAX_FALL_SPEED: f32 = -800.0;

/// Player half-extents (the player occupies a 14×28 px box around its centre).
const PLAYER_HALF_W: f32 = 7.0;
const PLAYER_HALF_H: f32 = 14.0;

/// How many seconds between mining ticks when holding LMB.
const MINE_INTERVAL: f32 = 0.15;

/// The block type currently selected for placement.
#[derive(Resource, Default)]
pub struct SelectedBlock(pub TileType);

/// Timer that controls how fast the player can mine.
#[derive(Resource)]
pub struct MineTimer(pub Timer);

impl Default for MineTimer {
    fn default() -> Self {
        let mut t = Timer::from_seconds(MINE_INTERVAL, TimerMode::Repeating);
        t.set_elapsed(std::time::Duration::from_secs_f32(MINE_INTERVAL)); // fire immediately on first press
        Self(t)
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedBlock>()
            .init_resource::<MineTimer>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    player_input,
                    apply_physics,
                    block_interaction,
                    cycle_selected_block,
                )
                    .chain(),
            );
    }
}

// ---------------------------------------------------------------------------
// Startup
// ---------------------------------------------------------------------------

fn spawn_player(mut commands: Commands, world: Res<GameWorld>) {
    let spawn = world.spawn_position();
    commands.spawn((
        Player,
        Velocity::default(),
        Sprite::from_color(Color::srgb(0.85, 0.72, 0.50), Vec2::new(14.0, 28.0)),
        Transform::from_translation(spawn.extend(10.0)),
    ));
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &Transform), With<Player>>,
    world: Res<GameWorld>,
) {
    let Ok((mut vel, transform)) = query.single_mut() else {
        return;
    };

    let pos = transform.translation.truncate();

    // Horizontal movement
    let mut dx = 0.0f32;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dx -= MOVE_SPEED;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dx += MOVE_SPEED;
    }
    vel.0.x = dx;

    // Jump — check all jump keys in one condition
    if (keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyW)
        || keyboard.just_pressed(KeyCode::ArrowUp))
        && is_on_ground(pos, &world)
    {
        vel.0.y = JUMP_SPEED;
    }
}

// ---------------------------------------------------------------------------
// Physics
// ---------------------------------------------------------------------------

fn apply_physics(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Player>>,
    world: Res<GameWorld>,
) {
    let Ok((mut vel, mut transform)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // Gravity
    vel.0.y = (vel.0.y + GRAVITY * dt).max(MAX_FALL_SPEED);

    // Move X then resolve collisions, then move Y then resolve collisions
    let mut pos = transform.translation.truncate();

    // -- X axis --
    pos.x += vel.0.x * dt;
    if resolve_x_collision(&mut pos, vel.0.x, &world) {
        vel.0.x = 0.0;
    }

    // -- Y axis --
    pos.y += vel.0.y * dt;
    if resolve_y_collision(&mut pos, vel.0.y, &world) {
        vel.0.y = 0.0;
    }

    transform.translation.x = pos.x;
    transform.translation.y = pos.y;
}

// ---------------------------------------------------------------------------
// Collision helpers
// ---------------------------------------------------------------------------

/// Returns `true` if any solid tile would block horizontal movement.
fn resolve_x_collision(pos: &mut Vec2, vel_x: f32, world: &GameWorld) -> bool {
    let left = pos.x - PLAYER_HALF_W;
    let right = pos.x + PLAYER_HALF_W;
    let top = pos.y + PLAYER_HALF_H - 1.0;
    let bottom = pos.y - PLAYER_HALF_H + 1.0;

    let check_x = if vel_x >= 0.0 { right } else { left };

    let ty_top = ((-top) / TILE_SIZE).floor() as i32;
    let ty_bot = ((-bottom) / TILE_SIZE).floor() as i32;
    let tx = (check_x / TILE_SIZE).floor() as i32;

    let mut hit = false;
    for ty in ty_top..=ty_bot {
        if world.get(tx, ty).is_solid() {
            hit = true;
            if vel_x >= 0.0 {
                pos.x = tx as f32 * TILE_SIZE - PLAYER_HALF_W;
            } else {
                pos.x = (tx + 1) as f32 * TILE_SIZE + PLAYER_HALF_W;
            }
            break;
        }
    }
    hit
}

/// Returns `true` if any solid tile blocks vertical movement.
fn resolve_y_collision(pos: &mut Vec2, vel_y: f32, world: &GameWorld) -> bool {
    let left = pos.x - PLAYER_HALF_W + 1.0;
    let right = pos.x + PLAYER_HALF_W - 1.0;
    let top = pos.y + PLAYER_HALF_H;
    let bottom = pos.y - PLAYER_HALF_H;

    let check_y = if vel_y <= 0.0 { bottom } else { top };

    let ty = ((-check_y) / TILE_SIZE).floor() as i32;
    let tx_left = (left / TILE_SIZE).floor() as i32;
    let tx_right = (right / TILE_SIZE).floor() as i32;

    let mut hit = false;
    for tx in tx_left..=tx_right {
        if world.get(tx, ty).is_solid() {
            hit = true;
            if vel_y <= 0.0 {
                pos.y = -(ty as f32 * TILE_SIZE) + PLAYER_HALF_H;
            } else {
                pos.y = -((ty + 1) as f32 * TILE_SIZE) - PLAYER_HALF_H;
            }
            break;
        }
    }
    hit
}

/// Returns `true` if the player is standing on solid ground.
fn is_on_ground(pos: Vec2, world: &GameWorld) -> bool {
    let bottom = pos.y - PLAYER_HALF_H - 1.0;
    let left = pos.x - PLAYER_HALF_W + 1.0;
    let right = pos.x + PLAYER_HALF_W - 1.0;

    let ty = ((-bottom) / TILE_SIZE).floor() as i32;
    let tx_left = (left / TILE_SIZE).floor() as i32;
    let tx_right = (right / TILE_SIZE).floor() as i32;

    for tx in tx_left..=tx_right {
        if world.get(tx, ty).is_solid() {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Block interaction (mine / place)
// ---------------------------------------------------------------------------

const REACH_TILES: f32 = 5.0;

fn block_interaction(
    time: Res<Time>,
    mut mine_timer: ResMut<MineTimer>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    player_query: Query<&Transform, With<Player>>,
    selected: Res<SelectedBlock>,
    mut world: ResMut<GameWorld>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let dist = (world_pos - player_pos).length();
    if dist > REACH_TILES * TILE_SIZE {
        return;
    }

    let (tx, ty) = world_to_tile(world_pos);
    if tx < 0 || ty < 0 || tx >= WORLD_WIDTH as i32 || ty >= WORLD_HEIGHT as i32 {
        return;
    }

    // Mine: hold LMB, fires every MINE_INTERVAL seconds
    if mouse.pressed(MouseButton::Left) {
        mine_timer.0.tick(time.delta());
        if mine_timer.0.just_finished() && world.get(tx, ty).is_solid() {
            world.set(tx, ty, TileType::Air);
        }
    } else {
        // Reset timer when LMB released so next press mines immediately
        mine_timer.0.reset();
        mine_timer
            .0
            .set_elapsed(std::time::Duration::from_secs_f32(MINE_INTERVAL));
    }

    // Place: single click RMB
    if mouse.just_pressed(MouseButton::Right) && !world.get(tx, ty).is_solid() {
        let tile_center = crate::world::tile_to_world(tx, ty);
        let overlap_x = (tile_center.x - player_pos.x).abs() < PLAYER_HALF_W + TILE_SIZE * 0.5;
        let overlap_y = (tile_center.y - player_pos.y).abs() < PLAYER_HALF_H + TILE_SIZE * 0.5;
        if !(overlap_x && overlap_y) {
            world.set(tx, ty, selected.0);
        }
    }
}

// ---------------------------------------------------------------------------
// Block cycling (number keys)
// ---------------------------------------------------------------------------

fn cycle_selected_block(keyboard: Res<ButtonInput<KeyCode>>, mut selected: ResMut<SelectedBlock>) {
    let blocks = [
        TileType::Dirt,
        TileType::Stone,
        TileType::Sand,
        TileType::Wood,
        TileType::Leaves,
        TileType::Grass,
    ];

    let keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
    ];

    for (i, key) in keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            selected.0 = blocks[i];
        }
    }
}

