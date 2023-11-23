pub mod admin;
pub mod levels;
pub mod misc;
pub mod roulette;
pub mod youtube;

use image::{ImageEncoder, ImageError, RgbaImage};

#[tracing::instrument]
pub fn to_png_buffer(card_buf: &[u8], width: u32, height: u32) -> Result<Vec<u8>, ImageError> {
    let img = RgbaImage::from_vec(width, height, card_buf.to_vec())
        .expect("Cannot create RgbaImage from vec");
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    encoder.write_image(&img, width, height, image::ColorType::Rgba8)?;

    Ok(buf)
}
