# Nexpert Playable Client Design

## Intent

Build a polished Windows launcher in C# and a separate Minecraft-like voxel client in Rust. Players choose an installed version from common Minecraft launcher layouts; Nexpert reads that player's local game JAR and resource packs. The long-term goal is a fast, playable voxel game informed by Horizon's subsystem-by-subsystem Rust rewrite and measured against vanilla behavior.

## Goals

- Let a player choose a local installation or launcher instance and a usable version.
- Read Minecraft textures from local files at runtime without committing or distributing them.
- Launch a separate Rust client and eventually enter a generated, explorable world.
- Keep heavy game-loop, world, meshing, and render work in Rust; keep launcher UI and install management in C#.
- Make FPS, frame pacing, memory, and world-load time measurable before claiming performance gains.
- Keep the GitHub project readable in English, with a roadmap, CI, issue templates, and release guidance.

## Non-goals for the first playable milestone

- Full Minecraft parity, multiplayer, mods, server hosting, or all historical versions.
- Improving performance of Legacy, Prism, or another third-party client. Nexpert performance applies to its own Rust client.
- Downloading or redistributing Minecraft game files, assets, or authentication tokens.
- Automatically editing user worlds or launcher installations.

## Constraints and choices

- Windows desktop is the first target; launcher is WPF on .NET 10.
- Game client is a separate Rust process. Graphics use a native GPU API; select a maintained wgpu/winit pairing after a build spike.
- Local installs are detected by conventional folder layouts and can be selected manually when a launcher uses a custom layout.
- Game files are read-only inputs. No asset or user-world data is stored in Git, CI artifacts, or crash reports.
- First playable target is one pinned Java version and one small world scene, then support expands through version-specific adapters.
- Match vanilla behavior where practical, but never advertise full parity until the corresponding behavior has explicit coverage.
- CI may build and run checks; fixes are proposed in reviewed pull requests, not silently applied to user installs.

## First playable acceptance

From the launcher, choose a local installation and a supported version, start the Rust window, load a Minecraft grass texture from that version's local JAR, enter a deterministic small voxel scene, move with WASD and mouse, see a frame-time/FPS readout, and exit without modifying the source installation. Record a baseline on the same machine before performance tuning.

## Open implementation choices

The exact GPU API crate versions and initial world dimensions are implementation choices. The renderer must avoid unnecessary CPU copies, keep the frame loop responsive, and use hardware acceleration where available. Platform or driver gaps should produce a clear error.
