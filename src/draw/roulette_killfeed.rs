use piet_common::{
    kurbo::{Point, Rect, Size},
    CairoText, CairoTextLayout, Color, Device, FontFamily, Image, ImageFormat, InterpolationMode,
    PietText, RenderContext, Text, TextLayout, TextLayoutBuilder,
};
use tracing::{debug, info, instrument};

use super::{to_png_buffer, Colors, KILLFEED_FONT};

const TEMPLATE_PATH: &str = "assets/images/killfeed.png";
const HEIGHT: usize = 32;
const WIDTH: usize = 395;
const COLOR_RECT_WIDTH: usize = 156;

#[instrument]
pub fn gen_killfeed(user_1: &str, user_2: &str) -> anyhow::Result<Vec<u8>> {
    info!("Draw killfeed {user_1} -> {user_2}");

    // Create context
    let mut device = Device::new().expect("Cannot create device");
    let mut bitmap = device
        .bitmap_target(WIDTH, HEIGHT, 1.0)
        .expect("Cannot create bitmap target");
    let mut rc = bitmap.render_context();
    debug!("Render context created");

    let template_buf = {
        let bytes = std::fs::read(&TEMPLATE_PATH)?;
        image::load_from_memory(&bytes)?.into_bytes()
    };
    let image = rc
        .make_image(WIDTH, HEIGHT, &template_buf, ImageFormat::RgbaSeparate)
        .expect("Cannot make image from killfeed template buffer");
    let rect = Rect::from_origin_size(
        Point::new(0., 0.),
        Size::new(image.size().width, image.size().height),
    );
    rc.draw_image(&image, rect, InterpolationMode::Bilinear);
    debug!("Image created from template buffer");

    // Load font
    let mut text = PietText::new();
    let font = text
        .font_family(KILLFEED_FONT)
        .expect("Cannot load font Adobe Helvetica");
    debug!("Font loaded");

    let colors = Colors::default();
    let baseline = HEIGHT as f64 - 6.5;

    let user_1_layout = text_layout_with_max_size(
        &mut text,
        font.clone(),
        colors.white,
        user_1,
        (COLOR_RECT_WIDTH - 4) as f64,
    );
    let pos = {
        let text_size = user_1_layout.image_bounds();
        let x = (COLOR_RECT_WIDTH as f64 / 2.) - (text_size.width() / 2.);
        let y = baseline - text_size.max_y();
        Point::new(x, y)
    };
    println!("point: {pos:?}");
    rc.draw_text(&user_1_layout, pos);

    let user_2_layout = text_layout_with_max_size(
        &mut text,
        font.clone(),
        colors.white,
        user_2,
        (COLOR_RECT_WIDTH - 4) as f64,
    );
    let pos = {
        let text_size = user_2_layout.image_bounds();
        let x = (WIDTH as f64 - (COLOR_RECT_WIDTH as f64 / 2.)) - (text_size.width() / 2.);
        let y = baseline - text_size.max_y();
        Point::new(x, y)
    };
    println!("point: {pos:?}");
    rc.draw_text(&user_2_layout, pos);

    let kf_buf = bitmap
        .to_image_buf(ImageFormat::RgbaPremul)
        .expect("Unable to get image buffer");
    let buf = to_png_buffer(kf_buf.raw_pixels(), WIDTH as u32, HEIGHT as u32)?;

    // bitmap.save_to_file("kf.png").unwrap();

    Ok(buf)
}

#[instrument]
fn text_layout_with_max_size(
    text: &mut CairoText,
    font: FontFamily,
    color: Color,
    string: &str,
    max_size: f64,
) -> CairoTextLayout {
    let mut font_height = 25.0;

    let text_layout = loop {
        let layout = text
            .new_text_layout(string.to_owned())
            .font(font.clone(), font_height)
            .text_color(color)
            .build()
            .unwrap();

        if layout.image_bounds().width() > max_size && font_height > 10.0 {
            font_height -= 0.5;
            continue;
        }

        break layout;
    };

    text_layout
}

#[test]
fn test_gen_kf() {
    let user_1 = "Swich";
    let user_2 = "ek0z";

    assert!(gen_killfeed(&user_1, &user_2).is_ok());
}

#[test]
fn test_gen_kf_with_long_name() {
    let user_1 = "a pretty long username, but very long long";
    let user_2 = "@K_limero91 ou @ChaK_lim";

    assert!(gen_killfeed(&user_1, &user_2).is_ok());
}
