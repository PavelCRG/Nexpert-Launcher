# Architecture

Nexpert keeps the desktop shell and game client in separate processes. The launcher owns local installation discovery, version selection, settings, and eventually updates. The Rust client owns resource loading and the real-time game loop. The process boundary lets each part be developed and measured independently.

## Current implementation

- `src/Nexpert.Launcher`: WPF UI, common launcher instance-folder discovery (`instances/`, `profiles/`, `.minecraft/`, `minecraft/`, `game/`), local version and resource-pack discovery, vanilla texture preview from the selected JAR, persisted game-folder preference, and process launch.
- `engine`: Rust command-line preview that checks the selected version JAR and optional shared asset indexes.
- Game files remain in the user's selected installation. They are neither copied into the repository nor included in builds.

## Milestones

1. Finish launcher discovery and compatibility for the common instance layouts.
2. Parse the selected asset index and resolve vanilla texture paths without copying asset files.
3. Render one textured block and a controllable camera in Rust.
4. Add chunk storage, meshing, lighting, world loading, and a measured performance baseline.
5. Add Microsoft account ownership checks before any online-mode or official-service integration.
6. Expand version support and compatibility through differential checks against vanilla behavior.

The current preview is milestone 1 only. Listing resource packs does not mean the client can apply them. FPS and load-time improvements must be measured on matched hardware and scenes before being claimed.
