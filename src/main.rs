use std::{env, error::Error, fs, io, mem, sync::Arc};
use std::collections::BTreeMap;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::str::FromStr;

use futures::StreamExt;
use tokio::io::AsyncBufReadExt;
use tokio::sync::Mutex;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::channel::{Attachment, Channel, ChannelType};
use twilight_model::guild::Guild;
use url::Url;

use crate::website_builder::builder::gallery_page_info::{Gallery, GalleryPageInfo, GalleryPictureInfo};

pub mod website_builder;
pub mod thumbnail_download;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let token = env::var("DISCORD_TOKEN")?;

    // Specify intents requesting events about things like new and updated messages in a guild and direct messages.
    let intents = Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

    // Create a single shard.
    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

    // The http client is separate from the gateway, so startup a new
    // one, also use Arc such that it can be cloned to other threads.
    let http = Arc::new(HttpClient::new(token));

    // Since we only care about messages, make the cache only process messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    let state = Arc::new(
        Mutex::new(State::PreReady {})
    );

    let (tx, mut rx): (_, tokio::sync::mpsc::Receiver<()>) = tokio::sync::mpsc::channel(3);
    let tx = Arc::new(tx);
    // Startup the event loop to process each event in the event stream as they
    // come in.
    loop {
        tokio::select! {
            next_event = shard.next_event() => {
                match next_event {
                    Ok(event) => {
                        // Update the cache.
                        cache.update(&event);

                        // Spawn a new task to handle the event
                        tokio::spawn(handle_event(event, Arc::clone(&http), state.clone(), tx.clone()));
                    }
                    Err(source) => {
                        tracing::warn!(?source, "error receiving event");

                        if source.is_fatal() {
                            break;
                        }
                    }
                }
            },

            _ = rx.recv() => {
                break;
            }
        }
    }


    if let State::Done { guilds } = state.lock().await.deref() {
        ask_user_for_guild_channel(guilds.clone(), http.clone()).await;
    } else {
        unreachable!()
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct BasicGuildInfo {
    name: String,
    channels: Vec<Channel>,
}


pub enum State {
    PreReady {},
    Ready {
        total_guilds_to_load: usize,
        guilds: Vec<BasicGuildInfo>,
    },
    Done {
        guilds: Vec<BasicGuildInfo>,
    },
}

mod state_machine {
    use crate::BasicGuildInfo;

    pub struct PreReady;
    pub struct Ready {
        total_guilds_to_load: usize,
        guilds: Vec<BasicGuildInfo>,
    }
    pub struct Done {
        guilds: Vec<BasicGuildInfo>,
    }

    pub struct GalleryBuildingState<S> {
        state: S,
    }

    impl GalleryBuildingState<PreReady> {
        pub fn ready(self, total_guilds_to_load: usize) -> GalleryBuildingState<Ready> {
            GalleryBuildingState {
                state: Ready {
                    total_guilds_to_load,
                    guilds: Vec::new(),
                }
            }
        }
    }

    impl GalleryBuildingState<Ready> {
        pub fn done(self) -> GalleryBuildingState<Done> {
            GalleryBuildingState {
                state: Done {
                    guilds: self.state.guilds
                }
            }
        }
    }
}

async fn handle_event(
    event: Event,
    http: Arc<HttpClient>,
    state: Arc<Mutex<State>>,
    done_sender: Arc<tokio::sync::mpsc::Sender<()>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::Ready(ready) => {
            let total_guilds_to_load = ready.guilds.len();

            let mut state = state.lock().await;

            *state = State::Ready {
                total_guilds_to_load,
                guilds: Vec::new(),
            };
            println!("Bot shard is ready");
        }
        Event::GuildCreate(g) => {
            let mut state = state.lock().await;

            if let State::Ready { total_guilds_to_load, guilds } = state.deref_mut() {
                let Guild { name, channels, .. } = g.0;

                let basic_guild_info = BasicGuildInfo {
                    name,
                    channels,
                };

                guilds.push(basic_guild_info);

                if guilds.len() >= *total_guilds_to_load {
                    let mut guilds = mem::take(guilds);
                    guilds.sort_by(|bgi1, bgi2| bgi1.name.partial_cmp(&bgi2.name).unwrap());
                    *state = State::Done {
                        guilds
                    };
                    done_sender.send(()).await.unwrap();
                }
            } else {
                panic!("Wrong state!")
            }
        }
        _ => {}
    }

    Ok(())
}

fn is_attachment_image(attachment: &Attachment) -> bool {
    attachment.content_type.is_some() && attachment.content_type.as_ref().unwrap().starts_with("image")
}

async fn ask_user_for_guild_channel(basic_guild_infos: Vec<BasicGuildInfo>, http: Arc<HttpClient>) {
    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());

    let chosen_guild_index = loop {
        println!("Select guild to generate gallery.");
        for (i, guild) in basic_guild_infos.iter().enumerate() {
            println!("{: >2}) {}", i, guild.name)
        }
        print!("Enter guild to use (q to quit): ");
        io::stdout().flush().unwrap();

        let mut read_buffer = Vec::new();
        reader.read_until(b'\n', &mut read_buffer).await.unwrap();
        let guild_number_input_string = String::from_utf8(read_buffer.clone()).unwrap();
        if guild_number_input_string.trim() == "q" {
            return;
        }
        if let Ok(guild_number) = usize::from_str(guild_number_input_string.trim()) {
            break guild_number;
        } else {
            println!("Invalid choice: {}", guild_number_input_string);
        }
    };
    println!();

    let chosen_guild = &basic_guild_infos[chosen_guild_index];
    let guild_categories = chosen_guild.channels.iter().filter(|c| c.kind == ChannelType::GuildCategory);
    let mut valid_guild_categories = guild_categories.filter(|guild_category| {
        chosen_guild.channels.iter().any(|guild_channel| {
            guild_channel.parent_id == Some(guild_category.id) && guild_channel.kind == ChannelType::GuildText
        })
    });

    let chosen_category_index = loop {
        for (i, guild_category) in valid_guild_categories.clone().enumerate() {
            println!("{: >2}) {}", i, &guild_category.name.as_deref().unwrap_or("No Category Name"))
        }

        print!("Enter guild category to use: ");
        io::stdout().flush().unwrap();
        let mut read_buffer = String::new();
        reader.read_line(&mut read_buffer).await.unwrap();
        if let Ok(category_index) = usize::from_str(read_buffer.trim()) {
            break category_index;
        } else {
            println!("Invalid choice: {}", read_buffer);
        }
    };
    println!();

    let chosen_category = valid_guild_categories.nth(chosen_category_index).unwrap();

    const WEBSITE_BUILD_DIRECTORY: &str = "built_websites";
    let website_save_dir = PathBuf::from(WEBSITE_BUILD_DIRECTORY).join(&chosen_guild.name).join(chosen_category.name.as_ref().unwrap());

    let category_channels = chosen_guild.channels.iter().filter(|c| c.parent_id == Some(chosen_category.id) && c.kind == ChannelType::GuildText);

    let mut galleries = Vec::new();

    // let thumbnail_downloader = Arc::new(std::sync::Mutex::new(ThumbnailDownloader::new()));

    for channel in category_channels {
        let channel_messages = http.channel_messages(channel.id).await.unwrap().model().await.unwrap();

        if channel_messages.is_empty() {
            continue;
        }

        let author_discord_name = {
            let mut counts = BTreeMap::new();
            for message in channel_messages.iter() {
                if message.attachments.iter().any(is_attachment_image) {
                    *counts.entry(&message.author.name).or_insert(0) += 1;
                }
            }

            if counts.is_empty() {
                continue;
            }

            let max = counts.into_iter().max_by_key(|&(_, count)| count).unwrap();
            max.0.clone()
        };

        let gallery_picture_infos = channel_messages
            .into_iter()
            .rev()
            .flat_map(|message| {
                let picture_description = if message.content.is_empty() {
                    None
                } else {
                    Some(message.content.clone())
                };
                // let thumbnail_downloader = thumbnail_downloader.clone();
                message
                    .attachments
                    .into_iter()
                    .filter(is_attachment_image)
                    .map(move |attachment| {
                        let picture_description = picture_description.clone();
                        let discord_url = attachment.proxy_url;


                        let image_url = Url::from_str(&discord_url).unwrap();
                        let trimmed_discord_url = image_url.path().trim_start_matches('/');
                        let attachment_save_path = PathBuf::from(WEBSITE_BUILD_DIRECTORY).join(trimmed_discord_url);
                        thumbnail_download::queue_download(&attachment_save_path, &image_url);

                        let thumbnail_url = attachment_save_path.strip_prefix(WEBSITE_BUILD_DIRECTORY).unwrap().to_string_lossy().to_string();

                        // let thumbnail_url = thumbnail_downloader.lock().unwrap().queue_download("test_website", &discord_url);
                        GalleryPictureInfo {
                            description: picture_description,
                            discord_url,
                            thumbnail_url,
                        }
                    })
            }).collect::<Vec<_>>();

        let author_name_channel = parse_author_name_from_channel_name(channel.name.as_deref().unwrap_or("No channel name?"), ChannelParseMode::FirstFullLastInitial);

        let gallery_title = format!("{author_name_channel} ({author_discord_name})");

        galleries.push(
            Gallery {
                gallery_title,
                gallery_picture_infos,
            }
        );
    }

    galleries.sort_unstable_by(|g1, g2| g1.gallery_title.cmp(&g2.gallery_title));

    let page_title = format!("{} Photo Galleries", chosen_guild.name);

    let gallery_page_info = GalleryPageInfo {
        page_title,
        galleries,
        guild_built_from: chosen_guild.name.clone(),
        page_built_time: "PAGE BUILT TIME".to_string(),
    };

    let json_rendered = serde_json::to_string(&gallery_page_info).unwrap();
    fs::write("json_rendered.json", json_rendered.to_string()).expect("TODO: panic message");

    // let rendered_page = render_page(&gallery_page_info);
    // write_whole_website_directory(website_save_dir, &rendered_page);

    thumbnail_download::flush_download_queue();

    {
        // let g = Arc::try_unwrap(thumbnail_downloader).unwrap_or_else(|_| panic!("")).into_inner().unwrap();
        // g.download_all().await;
    }
}

pub enum ChannelParseMode {
    FullName,
    FirstFullLastInitial,
}

fn parse_author_name_from_channel_name(channel_name: &str, channel_parse_mode: ChannelParseMode) -> String {
    match channel_parse_mode {
        ChannelParseMode::FullName => {
            channel_name
                .split('-')
                .map(|s| {
                    let mut chars = s.chars();
                    let mut string = String::from(chars.next().unwrap().to_ascii_uppercase());
                    string += &*chars.as_str().to_ascii_lowercase();

                    string
                })
                .collect::<Vec<String>>()
                .join(" ")
        }
        ChannelParseMode::FirstFullLastInitial => {
            let channel_name_parts = channel_name.split('-').collect::<Vec<&str>>();
            return if channel_name_parts.len() >= 2 {
                let mut first_name_chars = channel_name_parts[0].chars();
                let first_initial = first_name_chars.next().unwrap().to_ascii_uppercase();
                let rest_of_first_name = first_name_chars.as_str().to_ascii_lowercase();
                let last_initial = channel_name_parts[1].chars().next().unwrap().to_ascii_uppercase();
                format!("{}{} {}.", first_initial, rest_of_first_name, last_initial)
            } else { // If there isn't 2 parts to the name just return the channel name, this means someone didn't name their channel right (Shame!)
                channel_name.to_owned()
            };
        }
    }
}

