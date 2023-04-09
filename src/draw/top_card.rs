use piet_common::{
    kurbo::{Line, Point, Rect, Size},
    CairoTextLayout, Color, Device, ImageFormat, LineCap, LinearGradient, PietText, RenderContext,
    StrokeStyle, Text, TextLayout, TextLayoutBuilder, UnitPoint,
};
use tracing::{info, instrument};

use super::UserInfoCard;
use crate::{
    draw::{to_png_buffer, Colors, FONT},
    levels::xp::{total_xp_required_for_level, xp_needed_to_level_up},
};

const TITLE_HEIGHT: usize = 60;
const USER_HEIGHT: usize = 32;

struct UserLayout {
    rank: CairoTextLayout,
    name: CairoTextLayout,
    xp: CairoTextLayout,
    stroke: (f64, Color),
    level: CairoTextLayout,
}

#[instrument]
pub async fn gen_top_card(users: &[UserInfoCard], _guild_name: &str) -> anyhow::Result<Vec<u8>> {
    info!("get top_card for users:\n{users:#?}");

    // Some colors
    let colors = Colors::default();

    // Load font
    let mut text = PietText::new();
    let font = text.font_family(FONT).expect("Cannot load font family");
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
    let target_height = users.len() * USER_HEIGHT + TITLE_HEIGHT + 40;

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
    let mut y_offset = TITLE_HEIGHT as f64;
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

        y_offset += USER_HEIGHT as f64;
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
