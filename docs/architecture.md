# Architecture

Nexpert consists of a C# WPF launcher and an independent native game client written in Rust. The launcher discovers local game installations, selects a version and starts the Rust process with explicit paths. Rust owns asset parsing, UI/game rendering, input, simulation, world storage, and the game loop. The first rendering target is Vulkan through wgpu.

## The Horizon reference

Horizon is a standalone Rust reimplementation with its own launcher and game client; it does not speed up an existing Java Minecraft process. Its useful engineering pattern is staged subsystem work with explicit vanilla behavior targets, differential tests, a parity ledger, and profiling. Its benchmark guide compares the Rust client and vanilla with the same world, camera path, settings and statistics code; it records startup/title/world/chunk times and frame distributions. A separate pacing run reports tail latency and slow-frame causes. Nexpert follows that method while keeping a C# launcher and initially targeting Windows plus vanilla 1.20.1. We use Horizon as an architectural reference, not as a source of copied code or bundled Minecraft assets.

## Asset boundary

The player supplies a local Minecraft installation from any discovered or manually selected launcher folder. Nexpert reads the selected version metadata, JAR and asset index as read-only inputs. GUI images, fonts, block models, terrain and gameplay code are separate concerns: finding a texture file alone cannot make a Minecraft menu or world work. Every asset lookup has a version-aware path and a safe fallback; no Minecraft game files enter GitHub releases, CI, or Nexpert-owned storage.

Launcher import is an adapter boundary: the initial catalog targets Official Minecraft, Legacy, Prism/MultiMC, Modrinth and CurseForge layouts, with a manual game-directory option for other launchers. Each adapter resolves a selected version to explicit metadata, client JAR, libraries and assets directory. Forge/Fabric/NeoForge profiles can be detected before their runtime/classpath support is implemented; detecting them must not claim their Java mods run in the Rust game.

The title-screen path must resolve the version's shared assets index as well as its client JAR. Some client JAR entries such as the panorama faces are 1×1 placeholders; the full images live in the launcher's indexed object store. The asset preparation phase verifies the selected index and reports gaps before the game opens its menus.

## Performance boundary

Nexpert's Rust client can be tuned independently from the launcher, but the original Java game is untouched. FPS and frame-time values are diagnostics until a stable same-machine baseline exists. Performance changes must be profiled on CPU/GPU and checked against the same parity fixtures before they are accepted.

Each performance report should pin the build commit, game version, machine/GPU/driver, window size, graphics settings and camera path. Separate startup/world-loading measurements from steady-state FPS and frame pacing. Record repeated samples and show median and tail percentiles so a high average cannot conceal stutter. Compare with the locally installed vanilla game under matched settings; do not make a “Rust is faster” claim from a counter in one scene. Horizon's method is documented at [benchmark](https://github.com/horizon-rust/horizon/blob/main/docs/benchmark.md) and [graphics](https://github.com/horizon-rust/horizon/blob/main/docs/graphics.md).

## Current implementation

- `src/Nexpert.Launcher`: WPF UI, discovery of common launcher folder layouts, version/resource-pack listing, a local grass texture preview, and a Rust process launch with exit/error reporting.
- `engine/src`: Rust argument validation, direct version-JAR texture decoding, a deterministic tiny voxel scene, camera controls, and wgpu/winit rendering.
- The current client is a renderer smoke slice. It does not implement the Minecraft title screen, vanilla world generation, chunk streaming, collision, block interaction, saves, authentication, or multiplayer.
- `docs/ROADMAP.md` defines the twenty staged deliverables and acceptance boundaries.
