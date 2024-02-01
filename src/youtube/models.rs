use crate::database::{from_i64, to_i64};

#[derive(Debug, Clone)]
pub struct SubYtChannel {
    pub yt_channel_name: String,
    pub yt_channel_id: String,
    pub guild_id: u64,
    pub post_channel_id: u64,
    pub expire_on: time::OffsetDateTime,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct YtVideo {
    pub author_name: String,
    pub author_id: String,
    pub video_id: String,
    pub video_title: String,
}

pub struct SubYtChannelSQL {
    pub yt_channel_name: String,
    pub yt_channel_id: String,
    pub guild_id: i64,
    pub post_channel_id: i64,
    pub expire_on: time::OffsetDateTime,
}

impl From<SubYtChannel> for SubYtChannelSQL {
    fn from(value: SubYtChannel) -> Self {
        Self {
            yt_channel_name: value.yt_channel_name,
            yt_channel_id: value.yt_channel_id,
            guild_id: to_i64(value.guild_id),
            post_channel_id: to_i64(value.post_channel_id),
            expire_on: value.expire_on,
        }
    }
}

impl From<SubYtChannelSQL> for SubYtChannel {
    fn from(value: SubYtChannelSQL) -> Self {
        Self {
            yt_channel_name: value.yt_channel_name,
            yt_channel_id: value.yt_channel_id,
            guild_id: from_i64(value.guild_id),
            post_channel_id: from_i64(value.post_channel_id),
            expire_on: value.expire_on,
        }
    }
}