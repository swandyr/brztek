use brzthook::{HookListener, Mode};
use poise::serenity_prelude::{self as serenity, ChannelId};
use serde_json::Value;
use std::{
    sync::{mpsc, Arc},
    time::Duration,
};
use time::OffsetDateTime;
use tokio::join;
use tracing::{debug, error, info, instrument, warn};

use super::{constants::EXPIRATION_DAYS, queries};
use crate::{database::Db, Context, Data, Error};

pub fn expiration_check_timer(listener: Arc<HookListener>, db: Arc<Db>) -> Result<(), Error> {
    let rt = tokio::runtime::Runtime::new()?;

    loop {
        info!("Checking for expiration");
        let subs = rt.block_on(async { queries::get_subs_list(&db).await })?;
        let mut resubbed = vec![];
        let now_utc = OffsetDateTime::now_utc();

        for sub in subs {
            if now_utc > sub.expire_on.checked_add(time::Duration::days(-1)).unwrap()
                && !resubbed.contains(&sub.yt_channel_id)
            {
                listener.subscribe(&sub.yt_channel_id, Mode::Subscribe)?;
                let new_expire = now_utc
                    .checked_add(time::Duration::days(EXPIRATION_DAYS))
                    .unwrap();
                rt.block_on(async {
                    queries::update_expire_on(&db, new_expire, &sub.yt_channel_id).await
                })?;

                info!(
                    "Renewed subscription for {}: new expire_on = {}",
                    &sub.yt_channel_id, new_expire
                );
                resubbed.push(sub.yt_channel_id);
            }
        }

        // Check once a day
        std::thread::sleep(Duration::from_secs(24 * 3600));
    }
}
