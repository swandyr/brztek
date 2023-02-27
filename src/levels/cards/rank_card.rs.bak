use font_kit::font::Font;
use raqote::{DrawOptions, DrawTarget, Spread, Image, Point, SolidSource, Source, Color, PathBuilder, GradientStop, Gradient};

use super::{
    to_png_buffer,
    xp::{xp_needed_to_level_up, total_xp_required_for_level},
    FONT, DEFAULT_PP_TESSELATION_VIOLET, Colors
};

const CARD_WIDTH: i32 = 440;
const CARD_HEIGHT: i32 = 128;
const AVATAR_WIDTH: f32 = 96.0;
const AVATAR_HEIGHT: f32 = 96.0;

pub async fn gen_user_card(
    username: &str,
    avatar_url: Option<String>,
    banner_colour: (u8, u8, u8),
    level: i64,
    rank: i64,
    user_xp: i64,
) -> anyhow::Result<Vec<u8>> {
    // Xp values
    let xp_for_actual_level = total_xp_required_for_level(level);
    let xp_needed_to_level_up = xp_needed_to_level_up(level);
    let user_xp_in_level = user_xp - xp_for_actual_level;

    // Request profile picture through HTTP if `avatar_url` is Some().
    // Fallback to a default picture if None.
    let profile_picture = if let Some(url) = avatar_url {
        let url = clean_url(url);
        let bytes = reqwest::get(url).await?.bytes().await?;
        image::load_from_memory(&bytes)?
    } else {
        let default_file = DEFAULT_PP_TESSELATION_VIOLET;
        let bytes = std::fs::read(default_file)?;
        image::load_from_memory(&bytes)?
    };


    // Set some colors
    let colors = Colors::default();

    // let margin: f32 = (CARD_HEIGHT as f32 / 2.0) - (AVATAR_HEIGHT / 2.0);
    let margin = 16.0_f32;

    // Create the target
    let mut dt = DrawTarget::new(CARD_WIDTH, CARD_HEIGHT);
    dt.clear(SolidSource::from(colors.white));

    // Draw the user's xp gauge as a background gradient
    let (r, g, b) = banner_colour;
    let color_gradient = Color::new(0xff, r, g, b);

    let end_rect = (user_xp_in_level as f32 / xp_needed_to_level_up as f32).mul_add(CARD_WIDTH as f32, -50.0);
    dt.fill_rect(
        0.0, 
        0.0, 
        end_rect, 
        CARD_HEIGHT as f32, 
        &Source::Solid(SolidSource::from_unpremultiplied_argb(0xff, r, g, b)), 
        &DrawOptions::new());

    let start_gradient = Point::new(end_rect, 90.0);
    let end_gradient = Point::new(end_rect + 100.0, 90.0);

    let gradient = Source::new_linear_gradient(
        Gradient {
            stops: vec![
                GradientStop {
                    position: 0.0,
                    color: color_gradient,
                },
                GradientStop {
                    position: 0.9999,
                    color: colors.dark_gray,
                },
                GradientStop {
                    position: 1.0,
                    color: colors.dark_gray,
                },
            ],
        },
        start_gradient,
        end_gradient,
        Spread::Pad,
    );
    let mut pb = PathBuilder::new();
    pb.rect(0.0, 0.0, CARD_WIDTH as f32, CARD_HEIGHT as f32);
    let path = pb.finish();
    dt.fill(&path, &gradient, &DrawOptions::new());

    // Opacity background for readability
    let (a, r, g, b) = (
        0xc0_u8,
        colors.light_gray.r(),
        colors.light_gray.g(),
        colors.light_gray.b(),
    );

    dt.fill_rect(
        0.0,            // AVATAR_WIDTH + margin,
        0.0,            // margin,
        CARD_WIDTH as f32,     // 250.0,
        CARD_HEIGHT as f32,    // AVATAR_HEIGHT,
        &Source::Solid(SolidSource::from_unpremultiplied_argb(a, r, g, b)),
        &DrawOptions {
            blend_mode: raqote::BlendMode::Darken,
            alpha: 1.0,
            antialias: raqote::AntialiasMode::None,
        });

    // Transform rgba Vec<u8> to argb Vec<u32>
    let mut buffer: Vec<u32> = vec![];
    for i in profile_picture.as_bytes().chunks(4) {
        // RGBA to ARGB
        buffer.push((i[3] as u32) << 24 | (i[0] as u32) << 16 | (i[1] as u32) << 8 | i[2] as u32);
    }

    // Create an image from the avatar and draw it on the target
    let image = Image {
        width: profile_picture.width().try_into()?,
        height: profile_picture.height().try_into()?,
        data: &buffer,
    };    
    
    dt.draw_image_with_size_at(AVATAR_WIDTH, AVATAR_HEIGHT, margin, margin, &image, &DrawOptions::new());

    // Draw texts
    let font: Font = font_kit::loader::Loader::from_file(
        &mut std::fs::File::open(FONT)?,
        0,
    )?;
    let solid_source = Source::Solid(SolidSource::from(colors.white));
    dt.draw_text(
        &font,
        22.0,
        username,
        Point::new(130.0, 40.0),
        &solid_source,
        &DrawOptions::new(),
    );
    dt.draw_text(
        &font, 
        17., 
        &format!("Rank: #{rank}"),
        Point::new(130.0, 60.0),
        &solid_source,
        &DrawOptions::new(),
    );
    dt.draw_text(
        &font,
        17.,
        &format!("Level: {level}"),
        Point::new(130.0, 80.0),
        &solid_source,
        &DrawOptions::new(),
    );
    let total_xp_required_for_next_level = xp_for_actual_level + xp_needed_to_level_up;
    dt.draw_text(
        &font,
        15.,
        &format!("XP: {user_xp}/{total_xp_required_for_next_level}"),
        Point::new(130.0, 100.0),
        &solid_source,
        &DrawOptions::new(),
    );

    // Encode the image data into png and returned in Vec<u8>
    let card_buf = dt.get_data_u8().to_vec();
    let buf = to_png_buffer(&card_buf, CARD_WIDTH as u32, CARD_HEIGHT as u32)?;
    
    Ok(buf)
}

// Change .webp extension to .png and remove parameters from URL
fn clean_url(mut url: String) -> String {
    if let Some(index) = url.find("webp") {
        let _  = url.split_off(index);
        url.push_str("png?size=96");  // Ensure the size of the image to be at max 96x96
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
    assert!(gen_user_card(&username, Some(avatar_url), colour, 2, 1, 354).await.is_ok());
}

#[tokio::test]
async fn test_gen_card_with_default_pp() {
    let username = String::from("Username#64523");
    let colour = (255, 255, 0);
    assert!(gen_user_card(&username, None, colour, 2, 1, 275).await.is_ok());
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
