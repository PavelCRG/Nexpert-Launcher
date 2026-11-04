use std::io::{Read, Seek};

use image::{ImageFormat, RgbaImage};
use zip::ZipArchive;

const MAX_TEXTURE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_TEXTURE_DIMENSION: u32 = 1024;

pub fn read_grass_texture<R: Read + Seek>(jar: R) -> Result<RgbaImage, String> {
    read_block_texture(jar, "grass_block_top.png")
}

#[cfg(test)]
mod pending_tests {
    use super::read_block_texture;
    use image::{ColorType, ImageEncoder, codecs::png::PngEncoder};
    use std::io::{Cursor, Write};
    use zip::{ZipWriter, write::SimpleFileOptions};

    #[test]
    fn reads_named_vanilla_block_texture_from_jar() {
        let mut png = Vec::new();
        PngEncoder::new(&mut png)
            .write_image(&[80, 50, 30, 255], 1, 1, ColorType::Rgba8.into())
            .unwrap();
        let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
        archive
            .start_file(
                "assets/minecraft/textures/block/dirt.png",
                SimpleFileOptions::default(),
            )
            .unwrap();
        archive.write_all(&png).unwrap();
        let texture = read_block_texture(archive.finish().unwrap(), "dirt.png").unwrap();
        assert_eq!(texture.get_pixel(0, 0).0, [80, 50, 30, 255]);
    }
}

pub fn read_block_texture<R: Read + Seek>(jar: R, texture_name: &str) -> Result<RgbaImage, String> {
    let mut archive =
        ZipArchive::new(jar).map_err(|error| format!("Invalid Minecraft JAR: {error}"))?;
    let path = format!("assets/minecraft/textures/block/{texture_name}");
    let mut entry = archive
        .by_name(&path)
        .map_err(|error| format!("Minecraft block texture {texture_name} is missing: {error}"))?;
    if entry.size() > MAX_TEXTURE_BYTES {
        return Err("Minecraft grass texture exceeds the 4 MiB limit".into());
    }

    let mut bytes = Vec::with_capacity(entry.size() as usize);
    entry
        .by_ref()
        .take(MAX_TEXTURE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read Minecraft grass texture: {error}"))?;
    if bytes.len() as u64 > MAX_TEXTURE_BYTES {
        return Err("Minecraft grass texture exceeds the 4 MiB limit".into());
    }
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
        return Err("Minecraft grass texture is not a valid PNG".into());
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    if width == 0 || height == 0 || width > MAX_TEXTURE_DIMENSION || height > MAX_TEXTURE_DIMENSION
    {
        return Err(format!(
            "Minecraft grass texture dimensions are unsupported: {width}x{height}"
        ));
    }

    image::load_from_memory_with_format(&bytes, ImageFormat::Png)
        .map(|image| image.to_rgba8())
        .map_err(|error| format!("Could not decode Minecraft grass texture: {error}"))
}

#[cfg(test)]
mod tests {
    use super::read_grass_texture;
    use image::{ColorType, ImageEncoder, codecs::png::PngEncoder};
    use std::io::{Cursor, Write};
    use zip::{ZipWriter, write::SimpleFileOptions};

    fn jar_with(entry: &str, contents: &[u8]) -> Cursor<Vec<u8>> {
        let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
        archive
            .start_file(entry, SimpleFileOptions::default())
            .unwrap();
        archive.write_all(contents).unwrap();
        archive.finish().unwrap()
    }

    fn one_pixel_png() -> Vec<u8> {
        let mut png = Vec::new();
        PngEncoder::new(&mut png)
            .write_image(&[0, 255, 0, 255], 1, 1, ColorType::Rgba8.into())
            .unwrap();
        png
    }

    #[test]
    fn reads_grass_texture_from_jar_without_extracting_it() {
        let jar = jar_with(
            "assets/minecraft/textures/block/grass_block_top.png",
            &one_pixel_png(),
        );
        let texture = read_grass_texture(jar).unwrap();
        assert_eq!(texture.dimensions(), (1, 1));
        assert_eq!(texture.get_pixel(0, 0).0, [0, 255, 0, 255]);
    }

    #[test]
    fn reports_missing_grass_texture() {
        let jar = jar_with("assets/minecraft/textures/block/dirt.png", &one_pixel_png());
        assert!(read_grass_texture(jar).is_err());
    }
}
