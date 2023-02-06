use font_kit::font::Font;
use raqote::{
    Color, DrawOptions, DrawTarget, Gradient, GradientStop, PathBuilder, Point, SolidSource,
    Source, Spread, StrokeStyle,
};

const CARD_WIDTH: i32 = 440;
const TITLE_HEIGHT: i32 = 60;
const USER_HEIGHT: i32 = 40;
const AVATAR_HEIGHT: i32 = 42;

const FONT_DEJAVU_BLACK: &str = "assets/fonts/DejaVu Sans Mono Nerd Font Complete.ttf";

const DEFAULT_PP_TESSELATION_VIOLET: &str = "assets/images/default-pp/Tesselation-Violet.png";

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

pub async fn gen_top_ten_card(
    users: &[(
        String, //username
        // Option<&str>, // avatar url
        i64, // rank
        i64, // level
        i64, // current xp
        i64, // xp for next level
    )],
) -> anyhow::Result<()> {
    // Some colors
    let colors = Colors::default();

    // Calculate image height in function of the size of `users` Vec
    let target_height = users.len() as i32 * USER_HEIGHT + TITLE_HEIGHT;

    // Create the target
    let mut dt = DrawTarget::new(CARD_WIDTH, target_height);
    dt.clear(SolidSource::from(colors.white));

    // Create a gradient and fill the target
    let gradient = Source::new_linear_gradient(
        Gradient {
            stops: vec![
                GradientStop {
                    position: 0.0,
                    color: colors.light_gray,
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
        Point::new(190.0, target_height as f32 / 2.0),
        Spread::Pad,
    );
    let mut pb = PathBuilder::new();
    pb.rect(0.0, 0.0, CARD_WIDTH as f32, target_height as f32);
    let path = pb.finish();
    dt.fill(&path, &gradient, &DrawOptions::new());

    // Load font
    let font: Font =
        font_kit::loader::Loader::from_file(&mut std::fs::File::open(FONT_DEJAVU_BLACK)?, 0)?;

    // Create header
    let solid_source = Source::Solid(SolidSource::from(colors.white));
    dt.draw_text(
        &font,
        45.0,
        "Top 10",
        Point::new(260.0, 45.0),
        &solid_source,
        &DrawOptions::new(),
    );

    let mut y_offset = TITLE_HEIGHT as f32;
    for user in users {
        let (name, rank, level, current_xp, next_xp) = user;

        let mut x_pos = 10.0;
        dt.draw_text(
            &font,
            22.0,
            &format!("#{rank}"),
            Point::new(x_pos, 30.0 + y_offset),
            &solid_source,
            &DrawOptions::new(),
        );

        x_pos += 35.0;
        dt.draw_text(
            &font,
            20.0,
            name,
            Point::new(x_pos, 30.0 + y_offset),
            &solid_source,
            &DrawOptions::new(),
        );

        x_pos += 165.0;
        dt.draw_text(
            &font,
            14.0,
            &format!("{current_xp}/{next_xp}"),
            Point::new(x_pos, 24.0 + y_offset),
            &solid_source,
            &DrawOptions::new(),
        );

        x_pos += 200.0;
        dt.draw_text(
            &font,
            20.0,
            &format!("{level}"),
            Point::new(x_pos, 30.0 + y_offset),
            &solid_source,
            &DrawOptions::new(),
        );

        // Draw xp gauge
        let start = x_pos - 220.0;
        let end = start + 200.0;
        let length = end - start;

        let style = StrokeStyle {
            cap: raqote::LineCap::Round,
            width: 3.0,
            ..Default::default()
        };

        let mut pb = PathBuilder::new();
        pb.move_to(start, 30.0 + y_offset);
        pb.line_to(end, 30.0 + y_offset);
        let path = pb.finish();
        dt.stroke(
            &path,
            &Source::Solid(SolidSource::from(colors.light_gray)),
            &style,
            &DrawOptions::new(),
        );

        let end = (*current_xp as f32 / *next_xp as f32).mul_add(length, start);
        let mut pb = PathBuilder::new();
        pb.move_to(start, 30.0 + y_offset);
        pb.line_to(end, 30.0 + y_offset);
        let path = pb.finish();
        dt.stroke(
            &path,
            &Source::Solid(SolidSource::from(colors.yellow)),
            &style,
            &DrawOptions::new(),
        );

        y_offset += USER_HEIGHT as f32;
    }

    // Save to file
    dt.write_png("top_ten.png")?;
    Ok(())
}

#[tokio::test]
async fn test_gen_top() {
    let users = vec![
        ("user1".to_string(), 1, 4, 475, 770),
        ("user2".to_string(), 2, 3, 320, 435),
    ];
    assert!(gen_top_ten_card(&users).await.is_ok());
}
