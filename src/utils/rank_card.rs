use font_kit::font::Font;
use raqote::{DrawOptions, DrawTarget, Image, Point, SolidSource, Source};

const CARD_WIDTH: i32 = 440;
const CARD_HEIGHT: i32 = 160;

pub async fn gen_card(
    username: &str,
    avatar_url: &str,
    level: i64,
    user_xp: i64,
    xp_next_level: i64,
) -> anyhow::Result<()> {
    let url = clean_url(avatar_url);
    let img_bytes = reqwest::get(url).await?.bytes().await?;

    // Create the target and fill with white
    let mut dt = DrawTarget::new(CARD_WIDTH, CARD_HEIGHT);
    dt.clear(SolidSource::from_unpremultiplied_argb(240, 220, 220, 220));

    // Draw a black rectangle inside
    dt.fill_rect(
        5.,
        5.,
        (CARD_WIDTH - 10) as f32,
        (CARD_HEIGHT - 10) as f32,
        &Source::Solid(SolidSource::from_unpremultiplied_argb(250, 35, 35, 35)),
        &DrawOptions::new(),
    );

    // Load the avatar umage from url
    //let file = image::io::Reader::open("assets/images/avatar.png")?.decode()?;
    let file = image::load_from_memory(&img_bytes)?;

    // Transform [u8] to [u32]
    let mut buffer: Vec<u32> = vec![];
    for i in file.as_bytes().chunks(4) {
        buffer.push((i[3] as u32) << 24 | (i[0] as u32) << 16 | (i[1] as u32) << 8 | i[2] as u32)
    }
    // Create an image that will be drawn on the target
    let image = Image {
        width: file.width() as i32,
        height: file.height() as i32,
        data: &buffer,
    };
    let margin = ((CARD_HEIGHT / 2) - (file.height() as i32 / 2)) as f32;
    dt.draw_image_at(margin, margin, &image, &DrawOptions::new());

    // Load font
    let font: Font = font_kit::loader::Loader::from_file(
        &mut std::fs::File::open("assets/fonts/DejaVu Sans Mono Nerd Font Complete.ttf")?,
        0,
    )?;
    let solid_source = Source::Solid(SolidSource::from_unpremultiplied_argb(255, 220, 220, 220));
    dt.draw_text(
        &font,
        30.,
        username,
        Point::new(180., 55.),
        &solid_source,
        &DrawOptions::new(),
    );

    dt.draw_text(
        &font,
        20.,
        &format!("Rank: #{level}"),
        Point::new(185., 90.),
        &solid_source,
        &DrawOptions::new(),
    );

    dt.draw_text(
        &font,
        18.,
        &format!("{user_xp}/{xp_next_level}"),
        Point::new(240., 130.),
        &solid_source,
        &DrawOptions::new(),
    );

    // Write target to file
    dt.write_png("card.png")?;

    Ok(())
}

fn clean_url(url: &str) -> String {
    let mut url = url.to_owned();
    if let Some(index) = url.find("webp") {
        let _  = url.split_off(index);
        url.push_str("png");
    }
    url
}

#[tokio::test]
async fn test_gen_card() {
    let username = String::from("Username#64523");
    let avatar_url = String::from(
        "https://cdn.discordapp.com/avatars/164445708827492353/700d1f83e3d68d6a32dca1269093f81f.webp?size=1024",
    );
    assert!(gen_card(&username, &avatar_url, 2, 137, 255).await.is_ok());
}

#[test]
fn url_cleaned() {
    let dirty = 
        "https://cdn.discordapp.com/avatars/164445708827492353/700d1f83e3d68d6a32dca1269093f81f.webp?size=1024"
    ;
    let clean = String::from(
        "https://cdn.discordapp.com/avatars/164445708827492353/700d1f83e3d68d6a32dca1269093f81f.png",
    );
    assert_eq!(clean_url(dirty), clean);
}
