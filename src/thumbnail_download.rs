use std::collections::VecDeque;
use std::{fs, mem};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::str::FromStr;
use futures::{FutureExt, stream, StreamExt};
use image::ImageFormat;
use image::imageops::FilterType;
use once_cell::sync::Lazy;
use reqwest::Url;

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder().build().unwrap()
});

pub struct ThumbnailDownloader {
    queue: VecDeque<Pin<Box<dyn Future<Output=()>>>>,
}

impl ThumbnailDownloader {
    pub fn new() -> ThumbnailDownloader {
        ThumbnailDownloader {
            queue: Default::default(),
        }
    }

    pub fn queue_download<P: AsRef<Path>>(&mut self, website_root: P, image_url: &str) -> String {
        let image_url = Url::from_str(image_url).unwrap();

        let url_path = image_url.path().trim_start_matches("/");

        let save_path = website_root.as_ref().to_path_buf().join(url_path);

        if !save_path.exists() {
            let save_path = save_path.clone();
            self.queue.push_back(async move {
                println!("Starting download: {:?}", save_path);
                let thumbnail_image = async {
                    let response = CLIENT.get(image_url).send().await.unwrap(); // This can fail if we can't connect to the discord CDN, if we can't connect there isn't much reason continuing anyway
                    let image_bytes = response.bytes().await.unwrap(); // I think we can just unwrap here, I *think* that the only was this panics is if we run out of memory or something else where the best option is to just panic
                    let image = image::load_from_memory(&image_bytes).unwrap();

                    image.resize(250, 250, FilterType::Triangle) // This size here is based off of the values in gallery-style.css .gallery{}
                }.await;

                fs::create_dir_all(save_path.parent().unwrap()).unwrap();

                match thumbnail_image.save_with_format(&save_path, ImageFormat::Jpeg) {
                    Ok(_) => {
                        println!("Successfully saved thumbnail `{}`", save_path.display())
                    }
                    Err(err) => {
                        eprintln!("Error saving thumbnail `{}`: {}", save_path.display(), err)
                    }
                };
            }.boxed())
        } else {
            println!("Skipping already saved thumbnail `{}`", save_path.display())
        }

        save_path.strip_prefix(website_root).unwrap().to_string_lossy().to_string()
    }

    pub async fn download_all(mut self) {
        let queue = mem::take(&mut self.queue);
        let download_queue = stream::iter(queue.into_iter()).buffer_unordered(5).collect::<Vec<_>>();
        download_queue.await;
    }
}
