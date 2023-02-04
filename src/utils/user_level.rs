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
