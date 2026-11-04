use std::{collections::HashMap, path::PathBuf};

pub struct LaunchArgs {
    pub game_root: PathBuf,
    pub version: String,
    pub seed: u64,
    pub assets_root: PathBuf,
    pub packs_root: PathBuf,
}

pub fn parse(args: impl Iterator<Item = String>) -> Result<LaunchArgs, String> {
    let mut values = HashMap::new();
    let mut args = args;
    while let Some(key) = args.next() {
        if !matches!(
            key.as_str(),
            "--game-root" | "--version" | "--seed" | "--assets-root" | "--packs-root"
        ) {
            return Err(format!("unknown option: {key}"));
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {key}"))?;
        if values.insert(key.clone(), value).is_some() {
            return Err(format!("duplicate option: {key}"));
        }
    }

    let game_root = PathBuf::from(values.remove("--game-root").ok_or("missing --game-root")?);
    let version = values.remove("--version").ok_or("missing --version")?;
    let seed = values
        .remove("--seed")
        .ok_or("missing --seed")?
        .parse()
        .map_err(|_| "--seed must be an unsigned integer")?;
    let assets_root = values
        .remove("--assets-root")
        .map(PathBuf::from)
        .unwrap_or_else(|| game_root.join("assets"));
    let packs_root = values
        .remove("--packs-root")
        .map(PathBuf::from)
        .unwrap_or_else(|| game_root.join("resourcepacks"));

    if version.is_empty() {
        return Err("--version cannot be empty".into());
    }
    Ok(LaunchArgs {
        game_root,
        version,
        seed,
        assets_root,
        packs_root,
    })
}

#[cfg(test)]
mod tests {
    use super::{LaunchArgs, parse};
    use std::path::PathBuf;

    fn parse_args(values: &[&str]) -> Result<LaunchArgs, String> {
        parse(values.iter().map(|value| value.to_string()))
    }

    #[test]
    fn parses_game_root_version_and_seed() {
        let parsed = parse_args(&[
            "--game-root",
            "D:/Minecraft/game",
            "--version",
            "1.20.1",
            "--seed",
            "42",
        ])
        .unwrap();

        assert_eq!(parsed.game_root, PathBuf::from("D:/Minecraft/game"));
        assert_eq!(parsed.version, "1.20.1");
        assert_eq!(parsed.seed, 42);
    }

    #[test]
    fn rejects_missing_required_values() {
        assert!(parse_args(&["--game-root", "D:/Minecraft", "--version", "1.20.1"]).is_err());
    }

    #[test]
    fn rejects_duplicate_flags() {
        assert!(
            parse_args(&[
                "--game-root",
                "D:/one",
                "--game-root",
                "D:/two",
                "--version",
                "1.20.1",
                "--seed",
                "1",
            ])
            .is_err()
        );
    }

    #[test]
    fn rejects_non_numeric_seed() {
        assert!(
            parse_args(&[
                "--game-root",
                "D:/Minecraft",
                "--version",
                "1.20.1",
                "--seed",
                "world",
            ])
            .is_err()
        );
    }

    #[test]
    fn rejects_unknown_flags() {
        assert!(
            parse_args(&[
                "--game-root",
                "D:/Minecraft",
                "--version",
                "1.20.1",
                "--seed",
                "1",
                "--unsafe",
                "true",
            ])
            .is_err()
        );
    }
}
