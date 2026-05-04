# Space Invaders WebGPU 🚀

A modern, bleeding-edge recreation of the classic **Space Invaders**, built from the ground up to run entirely in the web browser leveraging **WebAssembly** and **WebGPU**. 

## Tech Stack
- **Language**: Rust 🦀
- **Engine**: Bevy Engine 0.14
- **Graphics API**: WebGPU (via `wgpu`)
- **Compilation Target**: WebAssembly (`wasm32-unknown-unknown`)
- **Bundler**: Trunk

## Features
- ⚡ **Insane Performance**: Runs natively in the browser via WebAssembly with zero JavaScript game loop overhead.
- 🎨 **WebGPU Rendering**: Utilizes the modern WebGPU standard for high-performance graphics, allowing us to push neon bloom effects and thousands of active entities at 60+ FPS.
- 🌌 **Visual Modes**:
  - **Lightweight Mode**: A classic retro-arcade feel, optimized for battery life and low-end devices.
  - **Intensive Mode (WebGPU)**: Features HDR cameras, real-time post-processing bloom, and glowing neon visuals!

---

## 🛠️ Build and Run Instructions

### Prerequisites
1. **Rust**: Installed via [rustup](https://rustup.rs/).
2. **WASM Target**: You need the WebAssembly target installed.
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
3. **Trunk**: A zero-config WASM web application bundler for Rust.
   ```bash
   cargo install trunk
   ```

### Running Locally
To build the WASM and serve the game locally, simply run:
```bash
trunk serve
```
Then open your browser to **http://127.0.0.1:8081**.

### Building for Production
To create an optimized production build (which minifies the WASM and generates the final HTML/CSS assets):
```bash
trunk build --release
```
The optimized web application will be output to the `/dist` directory, ready to be hosted on GitHub Pages, Vercel, Netlify, or any static file server.

## Architecture

This project leverages Bevy's **Entity-Component-System (ECS)** architecture.
- **Entities**: Players, Enemies, and Bullets are separate entities.
- **Components**: Transform (position), Velocity, Collider, and BulletType tag entities with their properties.
- **Systems**: Isolated functions (`player_movement`, `enemy_shooting`, `collision_system`) run every frame, iterating over entities that match specific component queries.

The graphics pipeline uses Bevy's built-in `Camera2dBundle` with `hdr: true` and `BloomSettings` to achieve the visually intensive neon glowing effect seen in modern retro-revival games.

## License
MIT
