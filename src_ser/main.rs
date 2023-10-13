use std::env;
use std::process::exit;
use std::sync::Arc;

use chrono::Local;
use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::futures::future::join_all;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;

use crate::website::builder::render_page;
use crate::website::builder::gallery_page_info::{Gallery, GalleryPageInfo, GalleryPictureInfo};
use crate::website::write_whole_website_directory;

pub(crate) mod website;

const BOT_GATEWAY_INTENTS: u64 = GatewayIntents::GUILD_MESSAGES.bits() | GatewayIntents::MESSAGE_CONTENT.bits();

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct BotHandler;

#[async_trait]
impl EventHandler for BotHandler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        let guilds = join_all(
            _guilds
                .iter()
                .map(|guild_id| {
                    ctx.http.get_guild(guild_id.0)
                })
        ).await;

        guilds.iter().for_each(|guild| {
            if let Ok(guild) = guild {
                println!("Got guild info: {}", guild.name);
            } else {
                println!("Error getting guild: {:?}", guild.as_ref().unwrap_err())
            }
        });

        let data = ctx.data.read().await;
        let shard_manager = data.get::<ShardManagerContainer>().unwrap();
        shard_manager.lock().await.shutdown_all().await;
    }

    async fn message(&self, _ctx: Context, new_message: Message) {
        println!("Got message: {:?}", new_message);
    }

    // async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
    //     println!("Connected to discord! Collecting guild info...");
    // }
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::from_bits_truncate(BOT_GATEWAY_INTENTS);
    let mut client = Client::builder(token, intents)
        .event_handler(BotHandler)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    println!("Done gathering information.");

    let test_picture_1 = GalleryPictureInfo {
        picture_description: Some("Test Picture Description".to_string()),
        discord_url: "https://cdn.discordapp.com/attachments/1066115155060084809/1066245166303809567/53F1BD5E-DD7E-4834-975A-5189F411789E_1_105_c.jpeg".to_string(),
        thumbnail_url: "https://cdn.discordapp.com/attachments/1066115155060084809/1066245166303809567/53F1BD5E-DD7E-4834-975A-5189F411789E_1_105_c.jpeg".to_string(),
    };

    let test_gallery_1 = Gallery {
        gallery_title: "Gallery 1 Title".to_string(),
        gallery_picture_infos: vec![test_picture_1],
    };

    let test_page = GalleryPageInfo {
        page_title: "The Page title!".to_string(),
        galleries: vec![test_gallery_1],
        guild_built_from: "No Guild".to_string(),
        page_built_time: Local::now().to_string(),
    };

    let test_page_rendered = render_page(&test_page);

    write_whole_website_directory("test_website_done_week2", &test_page_rendered);

    exit(0);
}