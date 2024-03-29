use piet_common::Color;
use poise::serenity_prelude::UserId;
use time::OffsetDateTime;

use super::{
    constants::{DELAY_ANTI_SPAM, MAX_XP_GAIN, MIN_XP_GAIN},
    func::xp_func,
};
use crate::database::from_i64;

#[derive(Debug, Clone, Copy, Default)]
pub struct UserLevel {
    pub user_id: UserId,   // Discord user id
    pub xp: i64,           // User's xp
    pub level: i64,        // User's level
    pub rank: i64,         // User's rank
    pub last_message: i64, // Timestamp of the last message posted
}

impl UserLevel {
    pub fn new(user_id: u64) -> Self {
        Self {
            user_id: UserId::from(user_id),
            xp: 0,
            level: 0,
            rank: 0,
            last_message: 0,
        }
    }

    pub fn gain_xp_if_not_spam(&mut self) -> bool {
        // Check the time between last and new message.
        // Return true if below anti_spam setting,
        // else false without adding xp
        let now: i64 = OffsetDateTime::now_utc().unix_timestamp();
        if now - self.last_message > DELAY_ANTI_SPAM {
            self.last_message = now;
            self.xp += xp_func::rand_xp_points(MIN_XP_GAIN, MAX_XP_GAIN);
            true
        } else {
            false
        }
    }

    pub fn has_level_up(&mut self) -> bool {
        let xp_to_next_level = xp_func::total_xp_required_for_level(self.level + 1);
        if self.xp >= xp_to_next_level {
            self.level += 1;
            true
        } else {
            false
        }
    }
}

impl From<(u64, i64, i64, i64, i64)> for UserLevel {
    fn from(item: (u64, i64, i64, i64, i64)) -> Self {
        let (user_id, xp, level, rank, last_message) = item;
        let user_id = UserId::from(user_id);
        Self {
            user_id,
            xp,
            level,
            rank,
            last_message,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct UserSql {
    pub user_id: i64,
    pub guild_id: i64,
    pub xp: i64,
    pub level: i64,
    pub rank: i64,
    pub last_message: i64,
}

impl From<UserSql> for UserLevel {
    fn from(value: UserSql) -> Self {
        Self {
            user_id: UserId::from(from_i64(value.user_id)),
            xp: value.xp,
            level: value.level,
            rank: value.rank,
            last_message: value.last_message,
        }
    }
}

/// This struct contains information that are printed on the `top_card`
#[derive(Debug)]
pub struct UserInfoCard {
    pub name: String,
    pub rank: i64,
    pub level: i64,
    pub current_xp: i64,
    pub colour: Color,
}

impl UserInfoCard {
    pub fn new(name: String, rank: i64, level: i64, current_xp: i64, colour: (u8, u8, u8)) -> Self {
        let colour = Color::rgba8(colour.0, colour.1, colour.2, 0xff);

        Self {
            name,
            rank,
            level,
            current_xp,
            colour,
        }
    }

    pub fn tuple(&self) -> (&str, i64, i64, i64, Color) {
        (
            &self.name,
            self.rank,
            self.level,
            self.current_xp,
            self.colour,
        )
    }
}
