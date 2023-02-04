use super::user_level::UserLevel;
use rand::prelude::*;

use crate::utils::db::add

const MIN_XP_GAIN: i64 = 15;
const MAX_XP_GAIN: i64 = 25;
pub const ANTI_SPAM_DELAY: i64 = 30;

fn xp_formula(lvl: i64) -> i64 {
    // xp formula used by Mee6: https://github.com/Mee6/Mee6-documentation/blob/master/docs/levels_xp.md
    5 * (lvl.pow(2)) + (50 * lvl) + 100
}

fn check_if_level_up(user: &UserLevel) -> bool {
    let next_level = user.level + 1;
    let xp_needed = (0..next_level).map(xp_formula).sum(); // https://rust-lang.github.io/rust-clippy/master/index.html#redundant_closures

    user.xp >= xp_needed
}

pub fn gain_xp(user: &mut UserLevel) -> bool {
    let mut rng = rand::thread_rng();
    let gain = rng.gen_range(MIN_XP_GAIN..=MAX_XP_GAIN);

    user.messages += 1;
    user.xp += gain;
    if check_if_level_up(user) {
        user.level += 1;
        return true;
    }

    false
}

#[allow(dead_code)]
pub fn xp_needed_to_level_up(user: &UserLevel) -> i64 {
    xp_formula(user.level) - user.xp
}

pub fn xp_for_level(level: i64) -> i64 {
    (0..=level).map(xp_formula).sum()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn xp_gain_with_level_up() {
        let mut user = UserLevel {
            user_id: 1i64,
            xp: 470,
            level: 2,
            messages: 10,
            last_message: 0,
        };

        let level_up = gain_xp(&mut user);
        let min = 470 + MIN_XP_GAIN;
        let max = 470 + MAX_XP_GAIN;

        assert!(level_up);
        assert!((min..=max).contains(&user.xp));
        assert_eq!(user.level, 3);
    }

    #[test]
    fn xp_gain_without_level_up() {
        let mut user = UserLevel {
            user_id: 1i64,
            xp: 320,
            level: 2,
            messages: 10,
            last_message: 0,
        };

        let level_up = gain_xp(&mut user);
        let min = 320 + MIN_XP_GAIN;
        let max = 320 + MAX_XP_GAIN;

        assert!(!level_up);
        assert!((min..=max).contains(&user.xp));
        assert_eq!(user.level, 2);
    }
}
