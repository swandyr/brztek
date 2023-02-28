use piet_common::{
    kurbo::{Point, Rect, Size},
    Color, Device, Image, ImageFormat, InterpolationMode, LinearGradient, PietText, RenderContext,
    Text, TextLayout, TextLayoutBuilder, UnitPoint,
};
use tracing::info;

use crate::levels::xp::{total_xp_required_for_level, xp_needed_to_level_up};

use super::{clean_url, to_png_buffer, DEFAULT_PP_TESSELATION_VIOLET};

const CARD_HEIGHT: usize = 128;
const CARD_WIDTH: usize = 440;
const MARGIN: f64 = 16.0;

// Move in cards.rs when all is migrated to piet
struct Colors {
    white: Color,
    dark_gray: Color,
    light_gray: Color,
    yellow: Color,
    opacity_mask: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            white: Color::rgba8(0xdc, 0xdc, 0xdc, 0xff),
            dark_gray: Color::rgba8(0x23, 0x23, 0x23, 0xff),
            light_gray: Color::rgba8(0x57, 0x57, 0x57, 0xff),
            yellow: Color::rgba8(0xff, 0xcc, 0x00, 0xff),
            opacity_mask: Color::rgba8(0x00, 0x00, 0x00, 0x44),
        }
    }
}

pub fn gen_user_card(
    username: &str,
    avatar_url: Option<String>,
    banner_colour: (u8, u8, u8),
    level: i64,
    rank: i64,
    user_xp: i64,
) -> anyhow::Result<Vec<u8>> {
    info!("Start drawing user card.");

    // Xp values
    let xp_for_actual_level = total_xp_required_for_level(level);
    let xp_needed_to_level_up = xp_needed_to_level_up(level);
    let user_xp_in_level = user_xp - xp_for_actual_level;

    // Get some colors
    let colors = Colors::default();

    // Create context
    let mut device = Device::new().expect("Cannot create device");
    let mut bitmap = device
        .bitmap_target(CARD_WIDTH, CARD_HEIGHT, 1.0)
        .expect("Cannot create bitmap target");
    let mut rc = bitmap.render_context();
    info!("Render context created.");

    // Draw background
    let width = CARD_WIDTH as f64;
    let height = CARD_HEIGHT as f64;

    let rect = Rect::from_origin_size(Point::new(0., 0.), Size::new(width, height));
    rc.fill(rect, &colors.dark_gray);

    // Draw the user's xp gauge as a background gradient
    let (r, g, b) = banner_colour;
    let gradient_colour = Color::rgba8(r, g, b, 0xff);

    let gradient_width =
        (user_xp_in_level as f64 / xp_needed_to_level_up as f64).mul_add(width, -50.);
    let rect = Rect::from_origin_size(Point::new(0., 0.), Size::new(gradient_width, height));
    rc.fill(rect, &gradient_colour);

    let rect = Rect::from_origin_size(Point::new(gradient_width, 0.), Size::new(100., height));
    let gradient = LinearGradient::new(
        UnitPoint::TOP_LEFT,
        UnitPoint::TOP_RIGHT,
        (gradient_colour, colors.dark_gray),
    );
    rc.fill(rect, &gradient);

    // Opacity mask
    let rect = Rect::from_origin_size(Point::new(0., 0.), Size::new(width, height));
    rc.fill(rect, &colors.opacity_mask);

    // Request profile picture through HTTP if `avatar_url` is Some().
    // Fallback to a default picture if None.
    let profile_picture = if let Some(url) = avatar_url {
        let url = clean_url(url);
        let bytes = reqwest::blocking::get(&url)?.bytes()?;
        info!("Received avater from {url}");
        image::load_from_memory(&bytes)?
    } else {
        let default_file = DEFAULT_PP_TESSELATION_VIOLET;
        let bytes = std::fs::read(default_file)?;
        info!("Loaded defaut avatar");
        image::load_from_memory(&bytes)?
    };
    let (pp_width, pp_height) = (
        profile_picture.width() as usize,
        profile_picture.height() as usize,
    );
    let image_buf = profile_picture.into_bytes();

    let image = rc
        .make_image(pp_width, pp_height, &image_buf, ImageFormat::RgbaPremul)
        .expect("Cannot make image from profile_picture buffer");
    info!("Image created from avatar bytes");

    let rect = Rect::from_origin_size(
        Point::new(MARGIN, MARGIN),
        Size::new(image.size().width, image.size().height),
    );
    rc.draw_image(&image, rect, InterpolationMode::Bilinear);
    info!("Image drawn");

    // Load font
    let mut text = PietText::new();
    let font = text
        .font_family("Akira Expanded")
        .expect("Cannot load font Akira Expanded");
    info!("Font loaded.");

    let mut pos = Point::new(image.size().width + 2. * MARGIN, MARGIN);
    let layout = text
        .new_text_layout(username.to_owned())
        .font(font.clone(), 24.)
        .text_color(colors.white)
        .build()
        .unwrap();
    rc.draw_text(&layout, pos);

    pos.y += layout.image_bounds().height() + 7.;
    let layout = text
        .new_text_layout("Rank: ")
        .font(font.clone(), 18.)
        .text_color(colors.light_gray)
        .build()
        .unwrap();
    rc.draw_text(&layout, pos);

    pos.x += layout.trailing_whitespace_width();
    let layout = text
        .new_text_layout(format!("#{rank}"))
        .font(font.clone(), 18.)
        .text_color(colors.white)
        .build()
        .unwrap();
    rc.draw_text(&layout, pos);

    pos.x = image.size().width + 2. * MARGIN;
    pos.y += layout.image_bounds().height() + 7.;
    let layout = text
        .new_text_layout("Level: ")
        .font(font.clone(), 18.)
        .text_color(colors.light_gray)
        .build()
        .unwrap();
    rc.draw_text(&layout, pos);

    pos.x += layout.trailing_whitespace_width();
    let layout = text
        .new_text_layout(format!("{level}"))
        .font(font.clone(), 18.)
        .text_color(colors.white)
        .build()
        .unwrap();
    rc.draw_text(&layout, pos);

    pos = Point::new(140., 90.);
    let total_xp_required_for_next_level = xp_for_actual_level + xp_needed_to_level_up;
    let layout = text
        .new_text_layout(format!("{user_xp}/{total_xp_required_for_next_level}"))
        .font(font, 15.)
        .text_color(colors.white)
        .build()
        .unwrap();
    rc.draw_text(&layout, pos);

    let card_buf = bitmap
        .to_image_buf(ImageFormat::RgbaPremul)
        .expect("Unable to get image buffer.");
    let buf = to_png_buffer(
        card_buf.raw_pixels(),
        CARD_WIDTH as u32,
        CARD_HEIGHT as u32,
    )?;
    info!("Card image encoded in PNG and saved in Vec<u8>");

    //bitmap.save_to_file("rank.png").unwrap();

    Ok(buf)
}

#[test]
fn test_gen_card_with_url() {
    let username = String::from("Username");
    let avatar_url = String::from(
        "https://cdn.discordapp.com/avatars/164445708827492353/700d1f83e3d68d6a32dca1269093f81f.webp?size=1024",
    );
    let colour = (255, 255, 0);
    assert!(
        gen_user_card(&username, Some(avatar_url), colour, 2, 1, 275)
            .is_ok()
    );
}

#[test]
fn test_gen_card_with_default_pp() {
    let username = String::from("Username#64523");
    let colour = (255, 255, 0);
    assert!(gen_user_card(&username, None, colour, 2, 1, 275)
        .is_ok());
}
