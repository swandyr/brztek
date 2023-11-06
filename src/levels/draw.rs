use image::{ImageEncoder, ImageError, RgbaImage};
use piet_common::{
    kurbo::{Line, Point, Rect, Size},
    CairoTextLayout, Color, Device, Image, ImageFormat, InterpolationMode, LineCap, LinearGradient,
    PietText, RenderContext, StrokeStyle, Text, TextLayout, TextLayoutBuilder, UnitPoint,
};
use tracing::{info, instrument};

use super::{
    xp_func::{total_xp_required_for_level, xp_needed_to_level_up},
    CARD_FONT, TOP_TITLE_HEIGHT, TOP_USER_HEIGHT,
};

#[derive(Debug, Clone, Copy)]
struct Colors {
    white: Color,
    dark_gray: Color,
    mid_gray: Color,
    light_gray: Color,
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

/// This struct contains information that are printed on the `top_card`
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

//////////////////////////////////////////////////////////////////////////////////////

const CARD_HEIGHT: usize = 128;
const CARD_WIDTH: usize = 440;
const MARGIN: f64 = 16.0;

#[instrument(skip_all)]
pub fn gen_user_card(
    user_info: UserInfoCard,
    profile_picture: (usize, usize, &[u8]),
) -> anyhow::Result<Vec<u8>> {
    info!("Start drawing user card.");

    let (username, rank, level, user_xp, banner_colour) = user_info.tuple();

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
    let gradient_start =
        (user_xp_in_level as f64 / xp_needed_to_level_up as f64).mul_add(width, -50.);
    let rect = Rect::from_origin_size(Point::new(0., 0.), Size::new(gradient_start + 0.5, height));
    rc.fill(rect, &banner_colour);

    let rect = Rect::from_origin_size(Point::new(gradient_start, 0.), Size::new(100., height));
    let gradient = LinearGradient::new(
        UnitPoint::TOP_LEFT,
        UnitPoint::TOP_RIGHT,
        (banner_colour, colors.dark_gray),
    );
    rc.fill(rect, &gradient);

    // Opacity mask
    let rect = Rect::from_origin_size(Point::new(0., 0.), Size::new(width, height));
    rc.fill(rect, &colors.opacity_mask);
    info!("opacity mask applied");

    // Draw profile picture
    let (pp_width, pp_height, image_buf) = profile_picture;
    println!("pp: {} {} {}", pp_width, pp_height, image_buf.len());
    let format = match image_buf.len() {
        36864 => ImageFormat::RgbaSeparate, // 96 * 96 * 4 : 4 bytes per pixels
        _ => ImageFormat::Rgb,              // 3 bytes per pixels
    };
    let image = rc
        .make_image(pp_width, pp_height, image_buf, format)
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
        .font_family(CARD_FONT)
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
        .text_color(match rank {
            1 => colors.gold,
            2 => colors.silver,
            3 => colors.bronze,
            _ => colors.light_gray,
        })
        .build()
        .unwrap();
    let rank_layout = text
        .new_text_layout(format!("#{rank}"))
        .font(font.clone(), 18.)
        .text_color(match rank {
            1 => colors.gold,
            2 => colors.silver,
            3 => colors.bronze,
            _ => colors.white,
        })
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

    // pos is the top-left point of the drawn text rectangle
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

    // bitmap.save_to_file("rank.png").unwrap();

    Ok(buf)
}

#[test]
fn test_gen_card_with_default_pp() {
    let username = String::from("Username");
    let colour = (255, 255, 0);
    let default_file = super::DEFAULT_PP_TESSELATION_VIOLET;
    let bytes = std::fs::read(default_file).unwrap();
    let image = image::load_from_memory(&bytes).unwrap();
    //let image = image.resize(96, 96, image::imageops::FilterType::Gaussian);
    let (image_width, image_height) = (image.width() as usize, image.height() as usize);
    let image_buf = image.into_bytes();

    let user_info = UserInfoCard::new(username, 1, 2, 275, colour);
    assert!(gen_user_card(user_info, (image_width, image_height, &image_buf),).is_ok());
}

//////////////////////////////////////////////////////////////////////////////////////

struct UserLayout {
    rank: CairoTextLayout,
    name: CairoTextLayout,
    xp: CairoTextLayout,
    stroke: (f64, Color),
    level: CairoTextLayout,
}

#[instrument(skip_all)]
pub async fn gen_top_card(users: &[UserInfoCard], _guild_name: &str) -> anyhow::Result<Vec<u8>> {
    info!("get top_card for users:\n{users:#?}");

    // Some colors
    let colors = Colors::default();

    // Load font
    let mut text = PietText::new();
    let font = text
        .font_family(CARD_FONT)
        .expect("Cannot load font family");
    info!("Font loaded");

    let xp_gauge_width = 180_usize;

    // Creates users layouts
    let user_layouts = users
        .iter()
        .map(|user| {
            let (name, rank, level, current_xp, color) = user.tuple();
            // Xp values
            let xp_for_actual_level = total_xp_required_for_level(level);
            let xp_needed_to_level_up = xp_needed_to_level_up(level);
            let user_xp_in_level = current_xp - xp_for_actual_level;

            // Create text layouts
            let rank = text
                .new_text_layout(format!("#{rank}"))
                .font(font.clone(), 18.)
                .text_color(match rank {
                    1 => colors.gold,
                    2 => colors.silver,
                    3 => colors.bronze,
                    _ => colors.white,
                })
                .build()
                .unwrap();
            let name = text
                .new_text_layout(name.to_owned())
                .font(font.clone(), 16.)
                .text_color(colors.white)
                .build()
                .unwrap();
            let total_xp_required_for_next_level = xp_for_actual_level + xp_needed_to_level_up;
            let xp = text
                .new_text_layout(format!("{current_xp}/{total_xp_required_for_next_level}"))
                .font(font.clone(), 12.)
                .text_color(colors.white)
                .build()
                .unwrap();
            let level = text
                .new_text_layout(format!("{level}"))
                .font(font.clone(), 16.)
                .text_color(colors.white)
                .build()
                .unwrap();

            let end_stroke =
                (user_xp_in_level as f64 / xp_needed_to_level_up as f64) * xp_gauge_width as f64;
            let stroke = (end_stroke, color);

            UserLayout {
                rank,
                name,
                xp,
                stroke,
                level,
            }
        })
        .collect::<Vec<UserLayout>>();

    // Get max sizes of layouts
    let rank_layout_max = user_layouts
        .iter()
        .map(|user| user.rank.trailing_whitespace_width() as usize)
        .max()
        .unwrap();
    let name_layout_max = user_layouts
        .iter()
        .map(|user| user.name.trailing_whitespace_width() as usize)
        .max()
        .unwrap();
    let xp_layout_max = user_layouts
        .iter()
        .map(|user| user.xp.trailing_whitespace_width() as usize)
        .max()
        .unwrap();
    let level_layout_max = user_layouts
        .iter()
        .map(|user| user.level.trailing_whitespace_width() as usize)
        .max()
        .unwrap();

    // Calculate image size in function of the size of the `users` Vec
    let target_width = 10
        + rank_layout_max
        + 10
        + name_layout_max
        + 10
        + xp_layout_max
        + 10
        + xp_gauge_width
        + 10
        + level_layout_max
        + 10;
    let target_height = users.len() * TOP_USER_HEIGHT + TOP_TITLE_HEIGHT + 40;

    // Create context
    let mut device = Device::new().expect("Cannot create device");
    let mut bitmap = device
        .bitmap_target(target_width, target_height, 1.0)
        .expect("Cannot create bitmap target");
    let mut rc = bitmap.render_context();
    info!("Render context created");

    let width = target_width as f64;
    let height = target_height as f64;

    // Draw background
    let gradient = LinearGradient::new(
        UnitPoint::TOP_LEFT,
        UnitPoint::BOTTOM_RIGHT,
        (colors.mid_gray, colors.dark_gray, colors.dark_gray),
    );
    let rect = Rect::from_origin_size(Point::new(0., 0.), Size::new(width, height));
    rc.fill(rect, &gradient);

    // Title
    let title_layout = text
        .new_text_layout(format!("Top {}", users.len()))
        .font(font, 45.)
        .text_color(colors.white)
        .build()
        .unwrap();
    let pos = Point::new(
        width - (title_layout.trailing_whitespace_width() + 50.),
        25.,
    );
    rc.draw_text(&title_layout, pos);

    // Draw elements for each users
    //
    // y_offset set where vertically the target is drawn,
    // it is incremented with the USER_HEIGHT constant when all elements
    // of a user are drawn
    let mut y_offset = TOP_TITLE_HEIGHT as f64;
    for user in user_layouts {
        // x_pos tracks the horizontal position to draw elements
        // relatively to the others, by incrementing or decrementing
        let mut x_pos = 10.0;
        rc.draw_text(&user.rank, Point::new(x_pos, 30. + y_offset));

        x_pos += rank_layout_max as f64 + 10.;
        // y offset is actually 'y_offset + difference in text height with rank_layout"
        rc.draw_text(&user.name, Point::new(x_pos, 30. + y_offset + 2.));

        x_pos += name_layout_max as f64 + 10.;
        rc.draw_text(&user.xp, Point::new(x_pos, 24. + y_offset + 10.));

        x_pos += xp_layout_max as f64 + 10.;
        let y_pos = y_offset + 42.5;
        let back_line = Line::new(
            Point::new(x_pos, y_pos),
            Point::new(x_pos + xp_gauge_width as f64, y_pos),
        );
        let front_line = Line::new(
            Point::new(x_pos, y_pos),
            Point::new(x_pos + user.stroke.0, y_pos),
        );
        let stroke_style = StrokeStyle {
            line_cap: LineCap::Round,
            ..Default::default()
        };
        rc.stroke_styled(back_line, &colors.mid_gray, 3., &stroke_style);
        rc.stroke_styled(front_line, &user.stroke.1, 3., &stroke_style);

        x_pos += xp_gauge_width as f64 + 10.;
        rc.draw_text(&user.level, Point::new(x_pos, 30. + y_offset + 2.));

        y_offset += TOP_USER_HEIGHT as f64;
    }

    let card_buf = bitmap
        .to_image_buf(ImageFormat::RgbaPremul)
        .expect("Unable to get image buf");
    let buf = to_png_buffer(
        card_buf.raw_pixels(),
        target_width.try_into()?,
        target_height.try_into()?,
    )?;

    // bitmap.save_to_file("card.png").unwrap();

    Ok(buf)
}

#[tokio::test]
async fn test_gen_top() {
    let users = vec![
        ("EKXZMANE".to_string(), 1, 4, 950, (35, 12, 50)),
        ("Meeeeeeelent".to_string(), 2, 3, 760, (48, 48, 0)),
        ("Bobish".to_string(), 3, 2, 298, (127, 0, 0)),
        ("user".to_string(), 4, 0, 2, (24, 102, 98)),
    ];
    let users = users
        .into_iter()
        .map(|u| UserInfoCard::new(u.0, u.1, u.2, u.3, u.4))
        .collect::<Vec<_>>();
    let guild_name = "The Guild".to_string();
    assert!(gen_top_card(&users, &guild_name).await.is_ok());
}
