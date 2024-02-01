use piet_common::Color;

pub mod rank_card;
pub mod top_card;

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
