# Nexpert Roadmap

Nexpert is a C# Windows launcher for an independent Rust Minecraft Java-style client. Horizon is the reference for the engineering process: build native systems in small slices, compare behavior to a pinned vanilla version, and publish measured results. Nexpert does not modify or accelerate another launcher's Java game. Its Rust client is a separate game implementation and must earn compatibility one subsystem at a time.

## Compatibility target

- First game target: vanilla Minecraft Java Edition 1.20.1 on Windows, using files already installed by the player.
- Import target: recognize common launcher folders and let the player choose any custom game directory. This does not mean every launcher, mod loader, or Minecraft version is already supported.
- Assets: read textures, GUI images, fonts, models, and other licensed game resources from the selected local installation at runtime. Do not put them in GitHub, CI artifacts, or Nexpert releases.
- Rendering: Rust owns the game loop and GPU renderer, with Vulkan as the first backend target. The C# launcher owns install selection, settings, and process lifecycle.
- Parity: every subsystem has a pinned-version reference and automated checks before it is called compatible. FPS goals are decided from measurements on matched hardware and scenes.

## Twenty milestones

1. **Repository and build foundation** — C# WPF launcher, Rust client, lockfiles, English contribution/license docs, CI builds and unit tests. *Done: first local commit exists; current working changes are not yet published.*
2. **Launcher discovery** — find `.minecraft`, Legacy, Prism/MultiMC `instances`, and standard profile folders; allow manual folder selection for other launchers. *Done for common folder shapes; custom layouts still require selection.*
3. **Version and library metadata** — parse version JSON, inheritance, asset index, client JAR and library paths; report unsupported or incomplete installations clearly. *In progress: current prototype supports a direct version JAR layout.*
4. **Read-only asset resolver** — find assets in the JAR and shared indexed object store; test hashes, missing objects, corrupt archives, and path safety without copying files into the repository.
5. **Vulkan renderer and baseline scenes** — choose a Vulkan adapter, render fixed menu/world scenes, report backend/driver, frame time, average FPS and 1% low. Benchmark startup-to-title, startup-to-world, chunk-ready time, and warmed-up camera paths before optimization.
6. **Minecraft UI rendering** — load original GUI textures and bitmap fonts from the selected local files; reproduce nearest-neighbor sampling, alpha blending, UI scale, and resize behavior.
7. **Title screen and menu navigation** — implement the panorama/background, logo placement, button states, focus, keyboard/mouse navigation, accessibility labels, and functional Singleplayer, Multiplayer, Options, and Quit routes.
8. **Options and launcher settings** — display resolution, GUI scale, sensitivity, render distance, sound and key bindings; persist Nexpert settings separately from the source install.
9. **World list and save safety** — list eligible vanilla worlds, read metadata, create a new world, and make versioned atomic saves without overwriting unrelated directories.
10. **Block registries and model baking** — read pinned-version block states, model JSON, texture references, UVs, and rotations; compare parsed and baked output against the selected vanilla version.
11. **Seeded terrain generation** — reproduce the pinned version's random streams, noise, biome selection and terrain pipeline; add differential tests for fixed seeds and coordinates.
12. **Chunk storage, meshing and lighting** — store sections, hide shared faces, bake block models, build lighting, frustum-cull and stream chunks while keeping the render thread responsive.
13. **Player movement and physics** — implement input, mouse capture, collision, gravity, jumping, fluid movement, camera, and parity checks against vanilla traces.
14. **Gameplay HUD and block interaction** — hotbar, inventory, crosshair, ray casting, block break/place, item use, crafting screens and correct state updates.
15. **Entities, sound and world systems** — tick scheduling, entities, drops, health, effects, audio, containers and the gameplay systems needed for survival play.
16. **Measured performance work** — compare Nexpert and vanilla on the same PC, version, saved world/seed, camera path, window size and settings. Run repeated samples; report average and 1% low FPS, median/p99/p99.9 frame times, CPU/GPU frame time, memory, startup-to-title, startup-to-world, and chunk load. Profile slow frames, optimize the demonstrated bottleneck, then publish before/after reports with commit and hardware details.
17. **Resource packs and supported extensions** — resolve pack priority and overrides from folders/ZIPs; add mod-loader support only through explicit adapters and compatibility tests.
18. **Accounts and multiplayer** — implement ownership/authentication and protocol behavior for online-mode servers; keep credentials out of logs and crash reports.
19. **Version parity and regression gate** — expand supported versions deliberately; run subsystem differential suites, replay bug fixtures, and require review for fixes before release.
20. **Windows packaging and public release** — bundle Nexpert code only, use signed updates and rollback, document supported GPU/OS/version combinations, publish checksums, release notes and repeatable benchmark results.

## Current implementation

Milestones 1–2 are in place. Milestone 3 only handles direct `<game-root>/versions/<id>/<id>.jar` installs. A Rust proof-of-renderer now reads local grass, dirt and stone PNGs and opens a tiny generated voxel scene with WASD/mouse input and an FPS/frame-time title. It is a GPU/window/asset-loading smoke slice, not a Minecraft title screen or a compatible Minecraft world. Do not call milestones 5–20 complete yet.

## Performance and compatibility rules

- Never claim that Rust makes Legacy, the official launcher, or another Java client faster. Performance comparisons apply only to Nexpert's own client.
- Keep a stable baseline scene, hardware/driver details, render settings, warm-up, sample duration, average FPS, 1% low and startup/world-load timings.
- Use profiling evidence to select optimizations. Keep parity checks in place when optimizing generation, meshing, rendering, or simulation.
- A green splash screen or a high FPS counter is not proof of Minecraft compatibility or better performance.
- Benchmark method follows Horizon's public approach: same harness/settings/path for the Rust client and vanilla; separate throughput and frame-pacing runs; preserve reproducible machine and report metadata.
