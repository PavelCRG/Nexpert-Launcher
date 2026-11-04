use std::{collections::HashMap, env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let options: HashMap<&str, &str> = args
        .chunks_exact(2)
        .map(|pair| (pair[0].as_str(), pair[1].as_str()))
        .collect();
    let game_root = PathBuf::from(options.get("--game-root").ok_or("missing --game-root")?);
    let version = options.get("--version").ok_or("missing --version")?;
    let assets_root = PathBuf::from(
        options
            .get("--assets-root")
            .ok_or("missing --assets-root")?,
    );
    let packs_root = PathBuf::from(options.get("--packs-root").ok_or("missing --packs-root")?);

    let version_dir = game_root.join("versions").join(version);
    if !version_dir.is_dir() {
        return Err(format!("Minecraft version folder not found: {version}").into());
    }
    let jar = version_dir.join(format!("{version}.jar"));
    if !jar.is_file() {
        return Err(format!("Minecraft client JAR not found: {}", jar.display()).into());
    }
    let indexes = assets_root.join("indexes");
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
    let pack_count = if packs_root.is_dir() {
        fs::read_dir(packs_root)?.filter_map(Result::ok).count()
    } else {
        0
    };

    println!("Nexpert Rust client preview");
    println!("Minecraft version: {version}");
    println!("Shared asset indexes found: {index_count}");
    println!("Local resource packs: {pack_count}");
    println!("Game files stay in place: {}", game_root.display());
    // shortcut: asset discovery only; add a renderer and pack resolver when the first playable scene is in scope.
    println!("Renderer: not implemented yet.");
    Ok(())
}
