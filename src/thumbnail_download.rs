use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use image::ImageFormat;
use image::imageops::FilterType;
use once_cell::sync::Lazy;
use reqwest::Url;

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder().build().unwrap()
});

pub async fn download_image<P: AsRef<Path>>(website_root: P, image_url: &str) -> String {
    let image_url = Url::from_str(image_url).unwrap();

    let url_path = image_url.path().trim_start_matches("/");

    let save_path = website_root.as_ref().to_path_buf().join(url_path);

    if !save_path.exists() {
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
    } else {
        println!("Skipping already saved thumbnail `{}`", save_path.display())
    }

    save_path.strip_prefix(website_root).unwrap().to_string_lossy().to_string()
}

pub struct ThumbnailDownloader {
    queue: VecDeque<(Url, PathBuf)>,
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
            self.queue.push_back((image_url, save_path.clone()))
        } else {
            println!("Skipping already saved thumbnail `{}`", save_path.display())
        }

        save_path.strip_prefix(website_root).unwrap().to_string_lossy().to_string()
    }

    pub async fn download_all(mut self) {
        while let Some((next_download_url, next_download_save_as)) = self.queue.pop_front() {
            println!("Starting download: {}", next_download_save_as.display());
            let thumbnail_image = async {
                let response = CLIENT.get(next_download_url).send().await.unwrap(); // This can fail if we can't connect to the discord CDN, if we can't connect there isn't much reason continuing anyway
                let image_bytes = response.bytes().await.unwrap(); // I think we can just unwrap here, I *think* that the only was this panics is if we run out of memory or something else where the best option is to just panic
                let image = image::load_from_memory(&image_bytes).unwrap();

                image.resize(250, 250, FilterType::Triangle) // This size here is based off of the values in gallery-style.css .gallery{}
            }.await;

            fs::create_dir_all(next_download_save_as.parent().unwrap()).unwrap();

            match thumbnail_image.save_with_format(&next_download_save_as, ImageFormat::Jpeg) {
                Ok(_) => {
                    println!("Successfully saved thumbnail `{}`", next_download_save_as.display())
                }
                Err(err) => {
                    eprintln!("Error saving thumbnail `{}`: {}", next_download_save_as.display(), err)
                }
            };
        }
    }
}
