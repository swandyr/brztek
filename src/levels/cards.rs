pub mod rank_card;
pub mod top_card;

use image::{ImageEncoder, ImageError, RgbaImage};
use raqote::Color;

use super::xp;

const FONT: &str = "assets/fonts/eurostile font/EurostileBold.ttf";
const DEFAULT_PP_TESSELATION_VIOLET: &str = "assets/images/default-pp/Tessellation-Violet.png";

struct Colors {
    white: Color,
    dark_gray: Color,
    light_gray: Color,
    yellow: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            white: Color::new(0xff, 0xdc, 0xdc, 0xdc),
            dark_gray: Color::new(0xff, 0x23, 0x23, 0x23),
            light_gray: Color::new(0xff, 0x57, 0x57, 0x57),
            yellow: Color::new(0xff, 0xff, 0xcc, 0x00),
        }
    }
}

fn to_png_buffer(card_buf: &[u8], width: u32, height: u32) -> Result<Vec<u8>, ImageError> {
    let mut buffer: Vec<u8> = vec![];
    for i in card_buf.chunks(4) {
        buffer.push(i[2]);
        buffer.push(i[1]);
        buffer.push(i[0]);
        buffer.push(i[3]);
    }

    let img = RgbaImage::from_vec(width, height, buffer).unwrap();
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    encoder.write_image(&img, width, height, image::ColorType::Rgba8)?;

    Ok(buf)
}
