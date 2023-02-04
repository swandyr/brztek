use super::user_level::UserLevel;
use rand::prelude::*;

const MIN_XP_GAIN: i64 = 15;
const MAX_XP_GAIN: i64 = 25;
pub const ANTI_SPAM_DELAY: i64 = 30;

fn xp_formula(lvl: i64) -> i64 {
    // xp formula used by Mee6: https://github.com/Mee6/Mee6-documentation/blob/master/docs/levels_xp.md
    5 * (lvl.pow(2)) + (50 * lvl) + 100
}

pub fn rand_xp() -> i64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(MIN_XP_GAIN..=MAX_XP_GAIN)
}

#[allow(dead_code)]
pub fn xp_needed_to_level_up(user: &UserLevel) -> i64 {
    xp_formula(user.level) - user.xp
}

pub fn xp_for_level(level: i64) -> i64 {
    (0..=level).map(xp_formula).sum() // https://rust-lang.github.io/rust-clippy/master/index.html#redundant_closures
}
