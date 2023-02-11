use super::xp::{rand_xp, total_xp_required_for_level};
use chrono::Utc;

use crate::utils::config::XpSettings;

#[derive(Debug, Default)]
pub struct UserLevel {
    pub user_id: u64,      // Discord user id
    pub xp: i64,           // User's xp
    pub level: i64,        // User's level
    pub messages: i64,     // User's messages count
    pub last_message: i64, // Timestamp of the last message posted
}

impl UserLevel {
    pub const fn new(user_id: u64) -> Self {
        Self {
            user_id,
            xp: 0,
            level: 0,
            messages: 0,
            last_message: 0,
        }
    }

    pub fn gain_xp_if_not_spam(&mut self, xp_settings: XpSettings) -> bool {
        // Check the time between last and new message.
        // Return true if below anti_spam setting,
        // else false without adding xp
        let now: i64 = Utc::now().timestamp();
        if now - self.last_message > xp_settings.delay_anti_spam {
            self.messages += 1;
            self.last_message = now;
            self.xp += rand_xp(xp_settings.min_xp_gain, xp_settings.max_xp_gain);
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
        let (user_id, xp, level, messages, last_message) = item;
        Self {
            user_id,
            xp,
            level,
            messages,
            last_message,
        }
    }
}
