# Nexpert Roadmap

Nexpert is a Windows-first C# launcher and an independent Rust client. It can read local Minecraft Java Edition installations from common launcher layouts. Minecraft game code and assets are not included in this repository; the player supplies local game files.

The roadmap tracks playable outcomes. A task is complete only when its acceptance check works on a real build. Performance claims require a repeatable comparison on the same machine, world, and settings.

## Now

1. **Project foundation** — C# WPF launcher, Rust client crate, English repository docs, and CI. *Done: builds on Windows.*
2. **Import common launchers** — select a launcher root or instance; recognize `instances/`, `profiles/`, `.minecraft/`, `minecraft/`, and `game/`. *Done: common layouts; custom layouts can be selected directly.*
3. **Read local Minecraft textures** — open the selected version JAR locally and preview the vanilla grass texture. *Done: preview path implemented.*
4. **Native Rust game window** — create a resizable GPU-backed window, handle resize/close, and show frame time. *In progress.*
5. **Enter a tiny world** — render a small textured voxel island and move a first-person camera with mouse and WASD. *Acceptance: launch from the C# app, enter the scene, move, and exit cleanly.*

## Next

6. **Chunk data** — store a 16×16×height block section and build a mesh from visible faces only.
7. **Terrain generation** — deterministic seed, height map, grass/dirt/stone layers, and reproducible chunk output.
8. **Player physics** — collision, gravity, jumping, walking, and camera sensitivity settings.
9. **Block interaction** — raycast from camera; mine and place blocks in the loaded area.
10. **World save and load** — versioned local world format with atomic writes and recovery from an interrupted save.
11. **Lighting and sky** — sunlight, basic ambient shading, fog distance, and day/night foundations.
12. **HUD and inventory** — crosshair, hotbar, health placeholder, block selection, and keyboard navigation.

## Later

13. **Resource packs** — resolve pack overrides and load textures from folders and ZIP packs, with vanilla fallback.
14. **Version adapters** — read version metadata, registries, and block definitions; document supported versions explicitly.
15. **Background chunk pipeline** — generate and mesh chunks on worker threads without blocking input or rendering.
16. **Performance work** — collect CPU/GPU timings, memory, frame pacing, and chunk-load latency; optimize measured hot paths.
17. **Account and online play** — Microsoft ownership flow and official online-mode identity before joining authenticated servers.
18. **Launcher installs and updates** — signed manifests, resumable downloads, hashes, rollback, and safe self-update.
19. **Quality and bug workflow** — CI builds, focused tests, sanitized crash reports, triage labels, and fixes reviewed as pull requests.
20. **Public release** — Windows packaging, clean-machine install guide, release notes, benchmark method/results, and supported-version table.

## Current limits

The current Rust client is a command-line preview. It does not render a world yet. The launcher reads local files and lists resource packs but does not apply packs in-game. Importing an installation does not improve the original launcher or its game; performance work applies only to the Nexpert Rust client.
