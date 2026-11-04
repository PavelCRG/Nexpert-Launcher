# Nexpert Launcher

An unofficial Windows launcher and experimental Rust client for locally installed Minecraft Java Edition files. The launcher is written in C# with WPF; the client is written in Rust.

> Nexpert is not an official Minecraft product and is not approved by or associated with Mojang or Microsoft. Minecraft is a trademark of Mojang AB. This repository does not include Minecraft code, assets, or game files. Users must provide a local game installation and are responsible for complying with its terms.

## Current status

The early playable milestone discovers local game versions and resource packs, reads vanilla block textures from the selected local client JAR, and opens a small Rust-rendered voxel island with WASD and mouse-look controls. Enter captures the mouse; Escape closes the game. The window title shows a live FPS and frame-time estimate. This is a small prototype world, not Minecraft parity: it has no chunks, collision, block interaction, saves, account login, multiplayer, or resource-pack application. The shown FPS is not a benchmark and no performance improvement is claimed.

## Requirements

- Windows 10 or newer
- .NET 10 SDK with the Windows Desktop workload
- Rust stable toolchain
- A local Minecraft Java Edition installation

## Build

```powershell
dotnet build Nexpert.slnx
cargo build --manifest-path engine/Cargo.toml
```

Run `src/Nexpert.Launcher/bin/Release/net10.0-windows/Nexpert.Launcher.exe`, choose a game's `versions/` folder or a launcher folder containing `instances/` or `profiles/`, select an installed version, and press **Play World**. Common `game/`, `.minecraft/`, and `minecraft/` layouts are recognized. Select a custom instance folder directly when its launcher uses another layout. The selected folder is stored in the current user's Local AppData. The Rust client reads vanilla block textures directly from the selected version JAR. Controls: WASD to move, Space/Shift to fly up/down, Enter to capture the mouse, Escape to exit.

The launcher reads version and resource-pack names from the chosen folder. It does not copy those files into this repository or download Minecraft files.

## Repository layout

```text
src/Nexpert.Launcher/  C# WPF desktop launcher
engine/                Rust client workspace
docs/                  Architecture and milestone notes
.github/workflows/     Build checks
```

Project direction and implementation status: [roadmap](docs/ROADMAP.md), [architecture](docs/architecture.md), [playable client design](docs/superpowers/specs/2026-11-04-nexpert-playable-client-design.md), and [implementation plan](docs/superpowers/plans/2026-11-04-nexpert-playable-client.md).

## Contributing

Open an issue with the launcher version, Windows version, steps to reproduce, and sanitized logs. Never attach access tokens, account data, private world saves, or copyrighted game files. Pull requests should keep licensed game files out of the repository.

## License

The original Nexpert source is MIT licensed. Minecraft and its assets are not covered by this license.
