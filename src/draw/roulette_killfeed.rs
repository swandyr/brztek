use piet_common::{
    kurbo::{Point, Rect, Size},
    Device, Image, ImageFormat, InterpolationMode, PietText, RenderContext, Text, TextLayout,
    TextLayoutBuilder,
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

    let user_1_layout = text
        .new_text_layout(user_1.to_owned())
        .font(font.clone(), 25.)
        .text_color(colors.white)
        .build()
        .unwrap();
    let pos = {
        let text_size = user_1_layout.image_bounds();
        let x = (COLOR_RECT_WIDTH as f64 / 2.) - (text_size.width() / 2.);
        let y = baseline - text_size.max_y();
        Point::new(x, y)
    };
    println!("point: {pos:?}");
    rc.draw_text(&user_1_layout, pos);

    let user_2_layout = text
        .new_text_layout(user_2.to_owned())
        .font(font, 25.)
        .text_color(colors.white)
        .build()
        .unwrap();
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

#[test]
fn test_gen_kf() {
    let user_1 = "Swich";
    let user_2 = "ek0z";

    assert!(gen_killfeed(&user_1, &user_2).is_ok());
}
