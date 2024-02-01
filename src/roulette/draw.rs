use piet_common::{
    kurbo::{Point, Rect, Size},
    CairoText, CairoTextLayout, Color, Device, FontFamily, Image, ImageFormat, InterpolationMode,
    PietText, RenderContext, Text, TextLayout, TextLayoutBuilder,
};
use tracing::{debug, info, instrument};

use crate::util::to_png_buffer;
use crate::Error;
use crate::roulette::models::ShotKind;

const KILLFEED_FONT: &str = "Coolvetica"; // Font needs to be installed on the system (https://www.dafont.com/akira-expanded.font)
const TEMPLATE_NORMAL_PATH: &str = "assets/images/killfeed.png";
const TEMPLATE_REVERSE_PATH: &str = "assets/images/killfeed_reverse.png";
const TEMPLATE_SELF_PATH: &str = "assets/images/killfeed_self.png";
const HEIGHT: usize = 32;
const WIDTH: usize = 395;
const COLOR_RECT_WIDTH: usize = 156;

#[derive(Debug, Clone, Copy)]
struct Colors {
    white: Color,
    // kf_orange: Color,
    // kf_blue: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            white: Color::rgba8(0xdc, 0xdc, 0xdc, 0xff),
            // kf_orange: Color::rgba8(0xf3, 0x73, 0x20, 0xff),
            // kf_blue: Color::rgba8(0x1b, 0x91, 0xf0, 0xff),
        }
    }
}

#[instrument]
pub fn gen_killfeed(user_1: &str, user_2: &str, kind: ShotKind) -> Result<Vec<u8>, Error> {
    info!("Draw killfeed {user_1} -> {user_2}");

    // Create context
    let mut device = Device::new().expect("Cannot create device");
    let mut bitmap = device
        .bitmap_target(WIDTH, HEIGHT, 1.0)
        .expect("Cannot create bitmap target");
    let mut rc = bitmap.render_context();
    debug!("Render context created");

    let template_buf = {
        let bytes = std::fs::read(match kind {
            ShotKind::Normal => &TEMPLATE_NORMAL_PATH,
            ShotKind::SelfShot => &TEMPLATE_SELF_PATH,
            ShotKind::Reverse => &TEMPLATE_REVERSE_PATH,
        })?;
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
    let baseline = HEIGHT as f64 - 8.0;

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
        let metrics = user_1_layout.line_metric(0).unwrap_or_default();
        let y = baseline - metrics.baseline;
        Point::new(x, y)
    };
    rc.draw_text(&user_1_layout, pos);

    let user_2_layout = text_layout_with_max_size(
        &mut text,
        font,
        colors.white,
        user_2,
        (COLOR_RECT_WIDTH - 4) as f64,
    );
    let pos = {
        let text_size = user_2_layout.image_bounds();
        let x = (WIDTH as f64 - (COLOR_RECT_WIDTH as f64 / 2.)) - (text_size.width() / 2.);
        let metrics = user_2_layout.line_metric(0).unwrap_or_default();
        let y = baseline - metrics.baseline;
        Point::new(x, y)
    };
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

    loop {
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
    }
}

#[test]
fn test_gen_kf_short() {
    let user_1 = "Swich";
    let user_2 = "Night";

    assert!(gen_killfeed(user_1, user_2, ShotKind::Normal).is_ok());
}

#[test]
fn test_gen_kf_with_long_name() {
    let user_1 = "a pretty long username, but very long long";
    let _user_1 = "Swich";
    let user_2 = "@K_limero91 ou @ChaK_lim";

    assert!(gen_killfeed(user_1, user_2, ShotKind::Normal).is_ok());
}
