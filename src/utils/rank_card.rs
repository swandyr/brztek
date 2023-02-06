use font_kit::font::Font;
use raqote::{DrawOptions, DrawTarget, Image, Point, SolidSource, Source, Color, PathBuilder, StrokeStyle, GradientStop, Gradient};

const CARD_WIDTH: i32 = 440;
const CARD_HEIGHT: i32 = 160;
const AVATAR_WIDTH: f32 = 128.0;
const AVATAR_HEIGHT: f32 = 128.0;

const FONT_DEJAVU_BLACK: &str = "assets/fonts/DejaVu Sans Mono Nerd Font Complete.ttf";

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
            dark_gray: Color::new(0xfa, 0x23, 0x23, 0x23),
            light_gray: Color::new(0xff, 0x57, 0x57, 0x57),
            yellow: Color::new(0xff, 0xff, 0xcc, 0x00),
        }
    }
}

pub async fn gen_card(
    username: &str,
    avatar_url: Option<String>,
    banner_colour: (u8, u8, u8),
    level: i64,
    user_xp: i64,
    xp_next_level: i64,
) -> anyhow::Result<()> {
    // Request profile picture via HTTP if `avatar_url` is Some().
    // Fallback to a default picture if None.
    let file = if let Some(url) = avatar_url {
        let url = clean_url(url);
        let buffer =reqwest::get(url).await?.bytes().await?;
        image::load_from_memory(&buffer)?
    } else {
        let default_file = DEFAULT_PP_TESSELATION_VIOLET;
        assert!(std::fs::File::open(default_file).is_ok());
        image::io::Reader::open(default_file)?.decode()?
    };

    // Set some colors
    let colors = Colors::default();

    let margin: f32 = (CARD_HEIGHT as f32 / 2.0) - (AVATAR_HEIGHT / 2.0);

    // Create the target and fill with white
    let mut dt = DrawTarget::new(CARD_WIDTH, CARD_HEIGHT);
    dt.clear(SolidSource::from(colors.white));

    let (r, g, b) = banner_colour;
    // Play with some gradients
    let gradient = Source::new_linear_gradient(
        Gradient {
            stops: vec![
                GradientStop {
                    position: 0.0,
                    color: Color::new(0xff, r, g, b),
                },
                GradientStop {
                    position: 0.99,
                    color: colors.dark_gray,
                },
                GradientStop {
                    position: 1.0,
                    color: colors.dark_gray,
                },
            ],
        },
        Point::new(40.0, 0.0),
        Point::new(190.0, 90.0),
        raqote::Spread::Pad,
    );
    let mut pb = PathBuilder::new();
    // pb.rect(5.0, 5.0, (CARD_WIDTH - 10) as f32, (CARD_HEIGHT - 10) as f32);
    pb.rect(0.0, 0.0, CARD_WIDTH as f32, CARD_HEIGHT as f32);
    let path = pb.finish();
    dt.fill(&path, &gradient, &DrawOptions::new());

    // Draw a black rectangle inside
    // dt.fill_rect(
    //     5.0,
    //     5.0,
    //     (CARD_WIDTH - 10) as f32,
    //     (CARD_HEIGHT - 10) as f32,
    //     &Source::Solid(SolidSource::from(colors.dark_gray)),
    //     &DrawOptions::new(),
    // );

    // Transform [u8] to [u32]
    let mut buffer: Vec<u32> = vec![];
    for i in file.as_bytes().chunks(4) {
        buffer.push((i[3] as u32) << 24 | (i[0] as u32) << 16 | (i[1] as u32) << 8 | i[2] as u32);
    }
    let avatar_width = file.width() as f32;
    let _avatar_height = file.height() as f32;
    // Create an image that will be drawn on the target
    let image = Image {
        width: file.width().try_into()?,
        height: file.height().try_into()?,
        data: &buffer,
    };
    
    // dt.draw_image_at(margin, margin, &image, &DrawOptions::new());
    dt.draw_image_with_size_at(AVATAR_WIDTH, AVATAR_HEIGHT, margin, margin, &image, &DrawOptions::new());

    // Load font
    let font: Font = font_kit::loader::Loader::from_file(
        &mut std::fs::File::open(FONT_DEJAVU_BLACK)?,
        0,
    )?;
    let solid_source = Source::Solid(SolidSource::from(colors.white));
    dt.draw_text(
        &font,
        22.0,
        username,
        Point::new(180.0, 35.0),
        &solid_source,
        &DrawOptions::new(),
    );
    dt.draw_text(
        &font,
        17.,
        &format!("Level: #{level}"),
        Point::new(180.0, 60.0),
        &solid_source,
        &DrawOptions::new(),
    );
    dt.draw_text(
        &font,
        15.,
        &format!("{user_xp}/{xp_next_level}"),
        Point::new(260.0, 130.0),
        &solid_source,
        &DrawOptions::new(),
    );

    // Draw xp gauge
    let start = margin.mul_add(2.0, avatar_width); // let start = margin * 2.0 + file.width() as f32;
    let end = CARD_WIDTH as f32 - margin;
    let length = end - start;

    let style = StrokeStyle {
        cap: raqote::LineCap::Round,
        width: 4.0,
        ..Default::default()
    };

    let mut pb = PathBuilder::new();
    pb.move_to(start, 140.);
    pb.line_to(end, 140.);
    let path = pb.finish();

    dt.stroke(
        &path, 
        &Source::Solid(SolidSource::from(colors.light_gray)), 
        &style, 
        &DrawOptions::new());

    let end = (user_xp as f32 / xp_next_level as f32).mul_add(length, start); // let end = (137.0 / 255.0) * length + start;
    let mut pb = PathBuilder::new();
    pb.move_to(start, 140.0);
    pb.line_to(end, 140.0);
    let path = pb.finish();

    dt.stroke(
        &path,
        &Source::Solid(SolidSource::from(colors.yellow)),
        &style,
        &DrawOptions::new(),
    );

    dt.write_png("rank.png")?;
    // ? See for later use `write_png_to_vec: https://github.com/jrmuizel/raqote/pull/180
    
    Ok(())
}

fn clean_url(mut url: String) -> String {
    if let Some(index) = url.find("webp") {
        let _  = url.split_off(index);
        url.push_str("png");
    }
    url
}

#[tokio::test]
async fn test_gen_card_with_url() {
    let username = String::from("Username#64523");
    let avatar_url = String::from(
        "https://cdn.discordapp.com/avatars/164445708827492353/700d1f83e3d68d6a32dca1269093f81f.webp?size=1024",
    );
    let colour = (255, 255, 0);
    assert!(gen_card(&username, Some(avatar_url), colour, 2, 137, 255).await.is_ok());
}

#[tokio::test]
async fn test_gen_card_with_default_pp() {
    let username = String::from("Username#64523");
    let colour = (255, 255, 0);
    assert!(gen_card(&username, None, colour, 2, 137, 255).await.is_ok());
}

#[test]
fn url_cleaned() {
    let dirty = 
        String::from("https://cdn.discordapp.com/avatars/164445708827492353/700d1f83e3d68d6a32dca1269093f81f.webp?size=1024");
    let clean = String::from(
        "https://cdn.discordapp.com/avatars/164445708827492353/700d1f83e3d68d6a32dca1269093f81f.png",
    );
    assert_eq!(clean_url(dirty), clean);
}
