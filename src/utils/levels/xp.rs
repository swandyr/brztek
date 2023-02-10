use super::user_level::UserLevel;
use rand::prelude::*;

const MIN_XP_GAIN: i64 = 15;
const MAX_XP_GAIN: i64 = 25;
pub const ANTI_SPAM_DELAY: i64 = 30;

pub fn rand_xp(min_gain: i64, max_gain: i64) -> i64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min_gain..=max_gain)
}

pub const fn xp_needed_to_level_up(level: i64) -> i64 {
    // The amount of xp needed from level X to level X+1
    // xp formula used by Mee6: https://github.com/Mee6/Mee6-documentation/blob/master/docs/levels_xp.md
    5 * (level.pow(2)) + (50 * level) + 100
}

pub fn total_xp_required_for_level(level: i64) -> i64 {
    (0..level).map(xp_needed_to_level_up).sum() // https://rust-lang.github.io/rust-clippy/master/index.html#redundant_closures
}

#[test]
fn test_xp_for_level() {
    let level = 4_i64;

    let xp_to_next_level = xp_needed_to_level_up(level);
    assert_eq!(xp_to_next_level, 380_i64);
}

#[test]
fn test_total_xp_for_level() {
    let level = 4_i64;

    let total_xp_required_for_level = total_xp_required_for_level(level);
    assert_eq!(total_xp_required_for_level, 770_i64);
}
