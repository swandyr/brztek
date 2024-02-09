use time::OffsetDateTime;

use super::models::{SubYtChannel, SubYtChannelSQL};
use crate::{
    database::{from_i64, to_i64, Db},
    Error,
};

pub async fn get_sub(
    db: &Db,
    yt_channel_name: &str,
    guild_id: u64,
) -> Result<Option<SubYtChannel>, Error> {
    let guild_id = to_i64(guild_id);
    let response = sqlx::query_as!(
        SubYtChannelSQL,
        r#"SELECT
            yt_channel_name,
            yt_channel_id,
            guild_id,
            post_channel_id,
            expire_on
        FROM yt_sub WHERE yt_channel_name = ? AND guild_id = ?"#,
        yt_channel_name,
        guild_id,
    )
    .fetch_optional(&db.pool)
    .await?;

    let yt_sub = response.map(SubYtChannel::from);

    Ok(yt_sub)
}

pub async fn get_post_channel_ids(db: &Db, yt_channel_id: &str) -> Result<Vec<u64>, Error> {
    let response = sqlx::query!(
        "SELECT post_channel_id FROM yt_sub WHERE yt_channel_id = ?",
        yt_channel_id
    )
    .fetch_all(&db.pool)
    .await?;

    let ids: Vec<_> = response
        .iter()
        .map(|r| from_i64(r.post_channel_id))
        .collect();
    Ok(ids)
}

pub async fn get_subs_list(db: &Db) -> Result<Vec<SubYtChannel>, Error> {
    let response = sqlx::query_as!(
        SubYtChannelSQL,
        r#"SELECT
            yt_channel_name,
            yt_channel_id,
            guild_id,
            post_channel_id,
            expire_on
        FROM yt_sub"#
    )
    .fetch_all(&db.pool)
    .await?;

    let yt_subs = response.into_iter().map(SubYtChannel::from).collect();

    Ok(yt_subs)
}

pub async fn insert_sub(db: &Db, sub: SubYtChannel) -> Result<(), Error> {
    let sub = SubYtChannelSQL::from(sub);

    sqlx::query!(
        "INSERT INTO yt_sub(yt_channel_name, yt_channel_id, guild_id, post_channel_id, expire_on)
            VALUES (?, ?, ?, ?, ?)
        ON CONFLICT (yt_channel_id, guild_id) DO UPDATE SET yt_channel_name = ?",
        sub.yt_channel_name,
        sub.yt_channel_id,
        sub.guild_id,
        sub.post_channel_id,
        sub.expire_on,
        sub.yt_channel_name
    )
    .execute(&db.pool)
    .await?;

    Ok(())
}

pub async fn update_expire_on(
    db: &Db,
    expire_on: OffsetDateTime,
    yt_channel_id: &str,
) -> Result<(), Error> {
    sqlx::query!(
        "UPDATE yt_sub SET expire_on = ? WHERE yt_channel_id = ?",
        expire_on,
        yt_channel_id
    )
    .execute(&db.pool)
    .await?;

    Ok(())
}

pub async fn delete_sub(db: &Db, yt_channel_id: &str, guild_id: u64) -> Result<(), Error> {
    let guild_id = to_i64(guild_id);

    sqlx::query!(
        "DELETE FROM yt_sub WHERE yt_channel_id = ? AND guild_id = ?",
        yt_channel_id,
        guild_id
    )
    .execute(&db.pool)
    .await?;

    Ok(())
}
