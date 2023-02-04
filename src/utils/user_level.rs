use super::levels::{rand_xp, xp_for_level, ANTI_SPAM_DELAY};
use chrono::Utc;

#[derive(Debug)]
pub struct UserLevel {
    pub user_id: i64, // Discord user id, stored as i64 because SQLite does not support 128 bits interger
    pub xp: i64,      // User's xp
    pub level: i64,   // User's level
    pub messages: i64, // User's messages count
    pub last_message: i64, // Timestamp of the last message posted
}

impl UserLevel {
    pub fn new(user_id: i64) -> Self {
        Self {
            user_id,
            xp: 0,
            level: 0,
            messages: 0,
            last_message: 0,
        }
    }

    pub fn gain_xp(&mut self) {
        // Check the time between last and new message.
        // If time is below anti spam constant, return early
        // without adding xp.
        let now: i64 = Utc::now().timestamp();
        if now - self.last_message > ANTI_SPAM_DELAY {
            self.messages += 1;
            self.last_message = now;
            self.xp += rand_xp();
        }
    }

    pub fn level_up(&mut self) -> bool {
        let xp_to_next_level = xp_for_level(self.level + 1);
        if self.xp >= xp_to_next_level {
            self.level += 1;
            true
        } else {
            false
        }
    }
}

impl From<[i64; 5]> for UserLevel {
    fn from(item: [i64; 5]) -> Self {
        Self {
            user_id: item[0],
            xp: item[1],
            level: item[2],
            messages: item[3],
            last_message: item[4],
        }
    }
}
