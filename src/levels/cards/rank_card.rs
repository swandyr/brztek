use piet_common::{
    kurbo::{Point, Rect, Size},
    Color, Device, Image, ImageFormat, InterpolationMode, LinearGradient, PietText, RenderContext,
    Text, TextLayout, TextLayoutBuilder, UnitPoint,
};
use tracing::{debug, info};

use crate::levels::xp::{total_xp_required_for_level, xp_needed_to_level_up};

use super::{to_png_buffer, Colors};

const CARD_HEIGHT: usize = 128;
const CARD_WIDTH: usize = 440;
const MARGIN: f64 = 16.0;

// Move in cards.rs when all is migrated to piet

pub fn gen_user_card(
    username: &str,
    profile_picture: (usize, usize, &[u8]),
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

    let gradient_start =
        (user_xp_in_level as f64 / xp_needed_to_level_up as f64).mul_add(width, -50.);
    let rect = Rect::from_origin_size(Point::new(0., 0.), Size::new(gradient_start + 0.5, height));
    rc.fill(rect, &gradient_colour);

    let rect = Rect::from_origin_size(Point::new(gradient_start, 0.), Size::new(100., height));
    let gradient = LinearGradient::new(
        UnitPoint::TOP_LEFT,
        UnitPoint::TOP_RIGHT,
        (gradient_colour, colors.dark_gray),
    );
    rc.fill(rect, &gradient);

    // Opacity mask
    let rect = Rect::from_origin_size(Point::new(0., 0.), Size::new(width, height));
    rc.fill(rect, &colors.opacity_mask);

    // Draw profile picture
    let (pp_width, pp_height, image_buf) = profile_picture;
    let image = rc
        .make_image(pp_width, pp_height, image_buf, ImageFormat::RgbaSeparate)
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

    let username_layout = text
        .new_text_layout(username.to_owned())
        .font(font.clone(), 24.)
        .text_color(colors.white)
        .build()
        .unwrap();
    let rank_label_layout = text
        .new_text_layout("Rank ")
        .font(font.clone(), 15.)
        .text_color(colors.light_gray)
        .build()
        .unwrap();
    let rank_layout = text
        .new_text_layout(format!("#{rank}"))
        .font(font.clone(), 18.)
        .text_color(colors.white)
        .build()
        .unwrap();
    let level_label_layout = text
        .new_text_layout("Level ")
        .font(font.clone(), 15.)
        .text_color(colors.light_gray)
        .build()
        .unwrap();
    let level_layout = text
        .new_text_layout(format!("{level}"))
        .font(font.clone(), 18.)
        .text_color(colors.white)
        .build()
        .unwrap();
    let xp_label_layout = text
        .new_text_layout("Xp ")
        .font(font.clone(), 12.)
        .text_color(colors.light_gray)
        .build()
        .unwrap();
    let total_xp_required_for_next_level = xp_for_actual_level + xp_needed_to_level_up;
    let xp_layout = text
        .new_text_layout(format!("{user_xp}/{total_xp_required_for_next_level}"))
        .font(font, 15.)
        .text_color(colors.white)
        .build()
        .unwrap();

    // pos it he top-left point of the drawn text rectangle
    let mut pos = Point::new(image.size().width + 2. * MARGIN, MARGIN);
    rc.draw_text(&username_layout, pos);

    let mut baseline = 24. + 18. + 22.;
    pos.y = baseline - rank_label_layout.image_bounds().height();
    rc.draw_text(&rank_label_layout, pos);

    pos.x += rank_label_layout.trailing_whitespace_width();
    pos.y = baseline - rank_layout.image_bounds().height();
    rc.draw_text(&rank_layout, pos);

    baseline += 18. + 2.;
    pos.x = image.size().width + 2. * MARGIN;
    pos.y = baseline - level_label_layout.image_bounds().height();
    rc.draw_text(&level_label_layout, pos);

    pos.x += level_label_layout.trailing_whitespace_width();
    pos.y = baseline - level_layout.image_bounds().height();
    rc.draw_text(&level_layout, pos);

    baseline += 15. + 5.;
    pos.x = image.size().width + 2. * MARGIN;
    pos.y = baseline - xp_label_layout.image_bounds().height();
    rc.draw_text(&xp_label_layout, pos);

    pos.x += xp_label_layout.trailing_whitespace_width();
    pos.y = baseline - xp_layout.image_bounds().height();
    rc.draw_text(&xp_layout, pos);

    let card_buf = bitmap
        .to_image_buf(ImageFormat::RgbaPremul)
        .expect("Unable to get image buffer.");
    let buf = to_png_buffer(card_buf.raw_pixels(), CARD_WIDTH as u32, CARD_HEIGHT as u32)?;
    info!("Card image encoded in PNG and saved in Vec<u8>");

    bitmap.save_to_file("rank.png").unwrap();

    Ok(buf)
}

// #[test]
// fn test_gen_card_with_url() {
//     let username = String::from("Username");
//     let avatar_url = String::from(
//         "https://cdn.discordapp.com/avatars/164445708827492353/700d1f83e3d68d6a32dca1269093f81f.webp?size=1024",
//     );
//     let colour = (255, 255, 0);
//     assert!(gen_user_card(&username, Some(avatar_url), colour, 2, 1, 275).is_ok());
// }

#[test]
fn test_gen_card_with_default_pp() {
    use crate::levels::cards::DEFAULT_PP_TESSELATION_VIOLET;

    let username = String::from("Username");
    let colour = (255, 255, 0);
    let default_file = DEFAULT_PP_TESSELATION_VIOLET;
    let bytes = std::fs::read(default_file).unwrap();
    let image = image::load_from_memory(&bytes).unwrap();
    let image = image.resize(96, 96, image::imageops::FilterType::Gaussian);
    let (image_width, image_height) = (image.width() as usize, image.height() as usize);
    let image_buf = image.into_bytes();
    assert!(gen_user_card(
        &username,
        (image_width, image_height, &image_buf),
        colour,
        2,
        1,
        275
    )
    .is_ok());
}
