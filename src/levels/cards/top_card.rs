use font_kit::font::Font;
use raqote::{
    DrawOptions, DrawTarget, Gradient, GradientStop, PathBuilder, Point, SolidSource, Source,
    Spread, StrokeStyle,
};

use super::{
    to_png_buffer,
    xp::{total_xp_required_for_level, xp_needed_to_level_up},
    Colors, FONT,
};

const CARD_WIDTH: i32 = 440;
const TITLE_HEIGHT: i32 = 60;
const USER_HEIGHT: i32 = 40;

pub async fn gen_top_ten_card(
    users: &[(
        String, //username
        i64,    // rank
        i64,    // level
        i64,    // current xp
    )],
    _guild_name: &str,
) -> anyhow::Result<Vec<u8>> {
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

    let font: Font = font_kit::loader::Loader::from_file(&mut std::fs::File::open(FONT)?, 0)?;

    // Create header
    let solid_source = Source::Solid(SolidSource::from(colors.white));
    // dt.draw_text(
    //     &font,
    //     55.0,
    //     guild_name,
    //     Point::new(20.0, 45.0),
    //     &solid_source,
    //     &DrawOptions::new(),
    // );
    let text = format!("Top {}", users.len());
    dt.draw_text(
        &font,
        45.0,
        &text,
        Point::new(260.0, 45.0),
        &solid_source,
        &DrawOptions::new(),
    );

    // Draw elements for each users
    //
    // y_offset set where vertically the darget is drawn,
    // it is incremented with the USER_HEIGHT constant when all elements
    // of a user are drawn
    let mut y_offset = TITLE_HEIGHT as f32;
    for user in users {
        let (name, rank, level, current_xp) = user;
        // Xp values
        let xp_for_actual_level = total_xp_required_for_level(*level);
        let xp_needed_to_level_up = xp_needed_to_level_up(*level);
        let user_xp_in_level = current_xp - xp_for_actual_level;
        println!("total xp for level {level}: {xp_for_actual_level}");
        println!("user xp: {current_xp}");
        println!("xp in his level: {user_xp_in_level}");

        // x_pos tracks the horizontal position to draw elements
        // relatively to the others, by incrementing or decrementing
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
        let total_xp_required_for_next_level = xp_for_actual_level + xp_needed_to_level_up;
        dt.draw_text(
            &font,
            14.0,
            &format!("{current_xp}/{total_xp_required_for_next_level}"),
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

        let end = (user_xp_in_level as f32 / xp_needed_to_level_up as f32).mul_add(length, start);
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

    let card_buf = dt.get_data_u8().to_vec();
    let buf = to_png_buffer(card_buf, CARD_WIDTH as u32, target_height as u32)?;

    Ok(buf)
}

#[tokio::test]
async fn test_gen_top() {
    let users = vec![
        ("EKXZMANE".to_string(), 1, 4, 950),
        ("Meeeeeeelent".to_string(), 2, 3, 760),
        ("Bobish".to_string(), 3, 2, 298),
        ("user".to_string(), 4, 0, 2),
    ];
    let guild_name = "The Guild".to_string();
    assert!(gen_top_ten_card(&users, &guild_name).await.is_ok());
}
