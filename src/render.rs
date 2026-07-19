use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

use crate::World;

/// Writes the field as a dependency-free 24-bit bitmap.
pub fn write_bmp(world: &World, path: impl AsRef<Path>) -> io::Result<()> {
    let width = world.width();
    let height = world.height();
    let pixels = world.rgb8();
    let row_bytes = width * 3;
    let row_padding = (4 - row_bytes % 4) % 4;
    let image_bytes = (row_bytes + row_padding) * height;
    let file_bytes = 54 + image_bytes;

    let file = File::create(path)?;
    let mut output = BufWriter::new(file);
    output.write_all(b"BM")?;
    output.write_all(&(file_bytes as u32).to_le_bytes())?;
    output.write_all(&[0; 4])?;
    output.write_all(&54_u32.to_le_bytes())?;
    output.write_all(&40_u32.to_le_bytes())?;
    output.write_all(&(width as i32).to_le_bytes())?;
    output.write_all(&(height as i32).to_le_bytes())?;
    output.write_all(&1_u16.to_le_bytes())?;
    output.write_all(&24_u16.to_le_bytes())?;
    output.write_all(&0_u32.to_le_bytes())?;
    output.write_all(&(image_bytes as u32).to_le_bytes())?;
    output.write_all(&2_835_i32.to_le_bytes())?;
    output.write_all(&2_835_i32.to_le_bytes())?;
    output.write_all(&0_u32.to_le_bytes())?;
    output.write_all(&0_u32.to_le_bytes())?;

    let padding = [0_u8; 3];
    for y in (0..height).rev() {
        for pixel in &pixels[y * width..(y + 1) * width] {
            output.write_all(&[pixel[2], pixel[1], pixel[0]])?;
        }
        output.write_all(&padding[..row_padding])?;
    }
    output.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    #[test]
    fn bitmap_has_a_valid_header_and_size() {
        let world = World::new(Config {
            width: 25,
            height: 24,
            ..Config::default()
        })
        .unwrap();
        let path = std::env::temp_dir().join(format!(
            "universum-{}-{}.bmp",
            std::process::id(),
            world.width()
        ));
        write_bmp(&world, &path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let expected_row = world.width() * 3 + 1;
        assert_eq!(&bytes[0..2], b"BM");
        assert_eq!(bytes.len(), 54 + expected_row * world.height());
        std::fs::remove_file(path).unwrap();
    }
}
