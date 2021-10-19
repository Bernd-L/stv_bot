use futures::stream::StreamExt;
use std::{env, error::Error, fmt::Debug, time::Duration};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::{thread::AutoArchiveDuration, Channel, GuildChannel},
    gateway::{payload::ThreadUpdate, Intents},
    id::ChannelId,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let token = env::var("DISCORD_TOKEN_STV")?;

    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    let scheme = ShardScheme::Auto;

    // Use intents to only receive guild message events.
    let (cluster, mut events) =
        Cluster::builder(token.to_owned(), Intents::GUILD_MESSAGES | Intents::GUILDS)
            .shard_scheme(scheme)
            .build()
            .await?;

    // Start up the cluster.
    let cluster_spawn = cluster.clone();

    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // HTTP is separate from the gateway, so create a new client.
    let http = HttpClient::new(token);

    // Since we only care about new messages, make the cache only
    // cache new messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache with the event.
        cache.update(&event);

        tokio::spawn(handle_event(shard_id, event, http.clone()));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: HttpClient,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) if msg.content == "!ping" => {
            // http.create_message(msg.channel_id)
            let r = http
                .create_message(ChannelId(900041355202547786))
                .content("Pong!")?
                .exec();

            debug_log(&http, "Got the future").await?;
            let r = r.await;

            debug_log(&http, &r).await?;
        }
        Event::MessageCreate(msg) if msg.content == "!ua" => {
            debug_log(&http, "Unarchiving").await?;

            // Unarchive the thread
            http.update_thread(ChannelId(900061500453060638))
                .archived(false)
                .locked(false)
                .auto_archive_duration(AutoArchiveDuration::Day)
                .exec()
                .await?;

            // // Unarchive the thread
            // let r = http
            //     .update_channel(ChannelId(900061500453060638))
            //     // .archived(false)
            //     // .locked(false)
            //     .name("1234")?
            //     .exec()
            //     .await;

            // debug_log(&http, &format!("{:#?}", &r)).await?;

            http.create_message(ChannelId(900041355202547786))
                .content("Unarchived")?
                .exec()
                .await?;
        }
        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
        }
        Event::ThreadUpdate(ThreadUpdate(Channel::Guild(GuildChannel::PublicThread(thread)))) => {
            // // http.create_message(thread.id)
            // http.create_message(ChannelId(900041355202547786))
            //     // .content(&format!("Archived={}", thread.thread_metadata.archived))?
            //     .content(&format!("```json\n{:#?}\n```", thread))?
            //     .exec()
            //     .await?;
        }
        _ => {}
    }

    Ok(())
}

async fn debug_log(
    http: &HttpClient,
    thing: impl Debug,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    http.create_message(ChannelId(900106000319799306))
        .content(&format!("```json\n{:#?}\n```", thing))?
        .exec()
        .await?;

    Ok(())
}
