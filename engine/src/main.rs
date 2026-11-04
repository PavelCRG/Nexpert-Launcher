#![cfg_attr(windows, windows_subsystem = "windows")]

mod args;
mod assets;
mod camera;
mod render;
mod world;

use std::{env, error::Error, fs, io::Error as IoError};

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse(env::args().skip(1)).map_err(std::io::Error::other)?;
    let version_dir = args.game_root.join("versions").join(&args.version);
    if !version_dir.is_dir() {
        return Err(format!("Minecraft version folder not found: {}", args.version).into());
    }
    let jar = version_dir.join(format!("{}.jar", args.version));
    if !jar.is_file() {
        return Err(format!("Minecraft client JAR not found: {}", jar.display()).into());
    }
    let grass = assets::read_grass_texture(fs::File::open(&jar)?).map_err(IoError::other)?;
    let textures = [
        grass.clone(),
        assets::read_block_texture(fs::File::open(&jar)?, "grass_block_side.png")
            .unwrap_or_else(|_| grass.clone()),
        assets::read_block_texture(fs::File::open(&jar)?, "dirt.png")
            .unwrap_or_else(|_| grass.clone()),
        assets::read_block_texture(fs::File::open(&jar)?, "stone.png")
            .unwrap_or_else(|_| grass.clone()),
    ];

    let indexes = args.assets_root.join("indexes");
    let index_count = if indexes.is_dir() {
        fs::read_dir(indexes)?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "json")
            })
            .count()
    } else {
        0
    };
    let pack_count = if args.packs_root.is_dir() {
        fs::read_dir(&args.packs_root)?
            .filter_map(Result::ok)
            .count()
    } else {
        0
    };

    println!(
        "Starting Nexpert Rust client for Minecraft {} (seed {})",
        args.version, args.seed
    );
    println!(
        "Found {index_count} asset indexes and {pack_count} local resource packs; game files remain in place."
    );
    let world = world::World::starter(args.seed);
    render::run(world.mesh(), textures).map_err(IoError::other)?;
    Ok(())
}
