use poise::serenity_prelude::UserId;
use crate::database::from_i64;

#[derive(Debug)]
pub enum ShotKind {
    Normal,
    SelfShot,
    Reverse,
}

#[derive(Debug, Clone, Copy)]
pub struct Roulette {
    pub timestamp: i64,
    pub caller_id: UserId,
    pub target_id: UserId,
    // Is Some if the record has triggered rff, is None if it processed normally
    pub rff_triggered: Option<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(super) struct RouletteSql {
    pub(super) timestamp: i64,
    pub(super)  caller_id: i64,
    pub(super) target_id: i64,
    pub(super) rff_triggered: Option<u8>,
}

impl From<RouletteSql> for Roulette {
    fn from(value: RouletteSql) -> Self {
        Self {
            timestamp: value.timestamp,
            caller_id: UserId::from(from_i64(value.caller_id)),
            target_id: UserId::from(from_i64(value.target_id)),
            rff_triggered: value.rff_triggered,
        }
    }
}