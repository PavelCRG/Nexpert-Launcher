# Nexpert Playable Client Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Launch a small textured voxel world from the C# launcher using a Rust GPU client.

**Architecture:** WPF owns installation/version selection and starts a separate Rust executable with explicit paths. Rust loads the selected local JAR texture, owns input/render/world state, and reports a stable frame-time baseline. The initial scene is deliberately small; chunk generation and vanilla parity follow the public roadmap.

**Tech Stack:** C# / WPF / .NET 10; Rust stable; wgpu and winit for GPU/window/input; image and ZIP reading for local JAR textures; GitHub Actions for Windows builds.

**Spec:** `docs/superpowers/specs/2026-11-04-nexpert-playable-client-design.md`

## Global Constraints

- Keep Minecraft code, JARs, textures, user worlds, tokens, and private paths out of Git and CI artifacts.
- Read user installation files without modifying them.
- Do not claim FPS improvement until a same-machine baseline exists.
- Keep the desktop launcher in C# and real-time client in Rust.
- Windows is the initial target; non-Windows support is out of scope for the first playable slice.

## Review Focus

- Invalid or unsupported JAR: show a clear error and keep the launcher usable.
- Inherited version metadata: resolve the actual client JAR or report that the version cannot yet launch.
- GPU adapter unavailable: fail clearly without crashing the launcher.
- Window resize/minimize: update the surface safely and avoid a busy loop while minimized.
- Missing or corrupt texture/asset: use a visible fallback and do not write into the source installation.

## File Map

- `engine/Cargo.toml`: pin the window, GPU, ZIP, and image dependencies.
- `engine/src/main.rs`: parse launch arguments and enter the event loop.
- `engine/src/assets.rs`: read one texture from the selected local client JAR.
- `engine/src/render.rs`: GPU device/surface, textured voxel mesh, camera matrices, resize, and frame timing.
- `engine/src/world.rs`: deterministic small voxel scene and visible-face mesh generation.
- `src/Nexpert.Launcher/MainWindow.xaml.cs`: pass the selected JAR, version, and world seed to Rust; report launch errors.
- `.github/workflows/ci.yml`: install Rust and .NET, build both apps, and run format/check commands.
- `docs/ROADMAP.md`: update progress only when each acceptance check is met.

### Task 1: Launch contract and local JAR texture loader

**Interfaces:** launcher passes `--game-root <path> --version <id> --seed <u64>`; Rust loads `assets/minecraft/textures/block/grass_block_top.png` from `<game-root>/versions/<id>/<id>.jar`.

- [ ] Add `args::parse(impl Iterator<Item = String>) -> Result<LaunchArgs, String>` that rejects missing values, invalid seeds, and duplicate keys.
- [ ] Add an assert-based self-check for valid arguments and invalid/missing arguments.
- [ ] Add ZIP and PNG dependencies; decode the texture to RGBA without extracting it to the game folder.
- [ ] Build and run the self-check; verify the local 1.20.1 JAR texture entry is readable.

### Task 2: Rust native window and GPU surface

**Interfaces:** `Renderer::new(window, texture)`, `Renderer::resize(width, height)`, `Renderer::draw(camera, world_mesh) -> FrameStats`.

- [ ] Create a Winit application window and initialize wgpu on resume.
- [ ] Handle adapter/device/surface failures with user-readable errors.
- [ ] Configure and resize the surface; skip zero-sized surfaces while minimized.
- [ ] Verify a clear-color frame opens and closes on Windows.

### Task 3: Textured voxel scene and controls

**Interfaces:** `World::starter(seed) -> World`, `World::mesh() -> Mesh`, `Camera::update(input, delta_seconds)`.

- [ ] Create a small deterministic grass/dirt platform and mesh only exposed faces.
- [ ] Draw the Minecraft grass texture on top faces and dirt on side faces; use visible fallback textures for missing entries.
- [ ] Add mouse look, WASD, jump, and frame-time/FPS display.
- [ ] Verify the player can enter, move around, and close the scene without changing source files.

### Task 4: Integration, benchmark, and release readiness

- [ ] Pass the selected version directory and seed from the launcher to Rust.
- [ ] Show launch errors and exit status in the launcher.
- [ ] Add a fixed benchmark scene and record hardware, settings, average FPS, 1% low, and world-load time.
- [ ] Run .NET build, Rust format/check/tests, and the Windows client smoke check in CI where a GPU runner is available.
- [ ] Update README and ROADMAP with the tested platform and measured results; never label the whole client Horizon-parity complete.
