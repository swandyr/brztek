use poise::serenity_prelude::UserId;
use time::OffsetDateTime;

use super::{
    xp_func::{rand_xp_points, total_xp_required_for_level},
    DELAY_ANTI_SPAM, MAX_XP_GAIN, MIN_XP_GAIN,
};

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
            self.xp += rand_xp_points(MIN_XP_GAIN, MAX_XP_GAIN);
            true
        } else {
            false
        }
    }

    pub fn has_level_up(&mut self) -> bool {
        let xp_to_next_level = total_xp_required_for_level(self.level + 1);
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
