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

// Change .webp extension to .png and remove parameters from URL
fn clean_url(mut url: String) -> String {
    if let Some(index) = url.find("webp") {
        let _ = url.split_off(index);
        url.push_str("png?size=96"); // Ensure the size of the image to be at max 96x96
    }
    url
}

#[test]
fn url_cleaned() {
    let dirty = 
        String::from("https://cdn.discordapp.com/avatars/164445708827492353/700d1f83e3d68d6a32dca1269093f81f.webp?size=1024");
    let clean = String::from(
        "https://cdn.discordapp.com/avatars/164445708827492353/700d1f83e3d68d6a32dca1269093f81f.png?size=96",
    );
    assert_eq!(clean_url(dirty), clean);
}
