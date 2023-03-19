pub mod rank_card;
pub mod top_card;

use image::{ImageEncoder, ImageError, RgbaImage};
use piet_common::Color;
use tracing::instrument;

const FONT: &str = "Akira Expanded"; // Font needs to be installed on the system (https://www.dafont.com/akira-expanded.font)
pub const DEFAULT_PP_TESSELATION_VIOLET: &str = "assets/images/default-pp/Tessellation-Violet.png";

struct Colors {
    white: Color,
    dark_gray: Color,
    mid_gray: Color,
    light_gray: Color,
    _yellow: Color,
    opacity_mask: Color,
    gold: Color,
    silver: Color,
    bronze: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            white: Color::rgba8(0xdc, 0xdc, 0xdc, 0xff),
            dark_gray: Color::rgba8(0x23, 0x23, 0x23, 0xff),
            mid_gray: Color::rgba8(0x57, 0x57, 0x57, 0xff),
            light_gray: Color::rgba8(0xb2, 0xb2, 0xb2, 0xff),
            _yellow: Color::rgba8(0xff, 0xcc, 0x00, 0xff),
            opacity_mask: Color::rgba8(0x00, 0x00, 0x00, 0x44),
            gold: Color::rgba8(0xc9, 0xb0, 0x37, 0xff),
            silver: Color::rgba8(0xb4, 0xb4, 0xb4, 0xff),
            bronze: Color::rgba8(0xad, 0x8a, 0x56, 0xff),
        }
    }
}

#[instrument]
fn to_png_buffer(card_buf: &[u8], width: u32, height: u32) -> Result<Vec<u8>, ImageError> {
    let img = RgbaImage::from_vec(width, height, card_buf.to_vec()).unwrap();
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    encoder.write_image(&img, width, height, image::ColorType::Rgba8)?;

    Ok(buf)
}

/// This struct contains informations that are printed on the top_card
#[derive(Debug)]
pub struct UserInfoCard {
    name: String,
    rank: i64,
    level: i64,
    current_xp: i64,
    colour: Color,
}

impl UserInfoCard {
    pub fn new(name: String, rank: i64, level: i64, current_xp: i64, colour: (u8, u8, u8)) -> Self {
        let colour = Color::rgba8(colour.0, colour.1, colour.2, 0xff);

        Self {
            name,
            rank,
            level,
            current_xp,
            colour,
        }
    }

    fn tuple(&self) -> (&str, i64, i64, i64, Color) {
        (
            &self.name,
            self.rank,
            self.level,
            self.current_xp,
            self.colour,
        )
    }
}
