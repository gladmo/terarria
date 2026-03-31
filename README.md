# Terarria

A Terraria-inspired 2D sandbox game built from scratch with [Bevy 0.18.1](https://bevyengine.org/) in Rust.

## Features

- **Procedural world generation** — 400×200 tile world built with FBM value-noise terrain, cave carving, sand patches, and trees
- **Tile types** — Air, Grass, Dirt, Stone, Bedrock, Sand, Wood, Leaves
- **Player physics** — Gravity, AABB collision with tiles, horizontal movement, jumping
- **Block mining** — Hold LMB to continuously dig (rate-limited); mined area updated instantly on screen
- **Block placement** — RMB to place the selected block type next to existing terrain
- **Viewport-culled rendering** — Only tiles visible in the camera frustum are live entities; dirty-tile tracking for instant updates
- **Smooth camera** — Camera follows the player with frame-rate-independent lerp and world-boundary clamping

## Controls

| Input | Action |
|-------|--------|
| A / ← | Move left |
| D / → | Move right |
| Space / W / ↑ | Jump |
| Hold LMB | Mine block |
| RMB | Place block |
| 1 | Select Dirt |
| 2 | Select Stone |
| 3 | Select Sand |
| 4 | Select Wood |
| 5 | Select Leaves |
| 6 | Select Grass |

## Building & Running

```bash
# Install Rust (https://rustup.rs) if not already present, then:
cargo run
```

### Linux prerequisites

```bash
sudo apt-get install -y libwayland-dev libxkbcommon-dev libx11-dev libasound2-dev
```

### Windows / macOS

No extra steps — Bevy's `DefaultPlugins` handles window creation via winit.

## Project Structure

```
src/
  main.rs       — App entry point, window/plugin setup
  world.rs      — Tile types, GameWorld resource, procedural generation
  player.rs     — Player entity, physics, input, block interaction
  camera.rs     — Camera follow system with world-boundary clamping
  rendering.rs  — Viewport-culled tile sprite management
  ui.rs         — On-screen HUD (controls hint + selected block indicator)
```
