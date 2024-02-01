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

use super::{constants::YOUTUBE_VIDEO_PREFIX, queries};
use crate::{database::Db, Context, Data, Error};

#[instrument(skip(ctx))]
pub fn listen_loop(
    ctx: serenity::Context,
    db: Arc<Db>,
    listener: Arc<HookListener>,
) -> Result<(), Error> {
    // Start TCP listening in another thread and pass it a `Sender<Notification>
    let (tx, rx) = mpsc::channel();
    listener.listen(&tx);

    info!("Starting webhooks listener");
    // Wait for the sender to transfer data
    loop {
        match rx.recv().expect("recv error") {
            Err(e) => error!("Error in HookListener: {e}"),
            Ok(notification) => {
                let rt = tokio::runtime::Runtime::new()?;

                info!("Got notification from listener");
                let post_channel_ids = rt.block_on(async {
                    queries::get_post_channel_ids(&db, &notification.channel_id)
                        .await
                        .unwrap()
                });

                for id in post_channel_ids {
                    let message = format!("New video from **{}** !!!", &notification.channel_name);
                    let content = format!(
                        "{message}\n{YOUTUBE_VIDEO_PREFIX}{}",
                        &notification.video_id
                    );
                    // Bot disconnect after ChannelId.say(); why ?
                    rt.block_on(async {
                        ChannelId::new(id).say(&ctx.http, &content).await.unwrap();
                    });
                }
            }
        }
    }
    debug!("Quit listener loop");
}
